use crate::{
    ce::{coredll::CoredllGuestMemory, kernel::CeKernel},
    emulator::{
        imports::{
            ExternalImportTable, IMPORT_TRAP_BASE, IMPORT_TRAP_PAGE_SIZE, ImportTrapTable,
            import_trap_code_page, patch_pe_coredll_imports, patch_pe_imports,
        },
        memory::{MemoryMap, MemoryPerms},
    },
    error::{Error, Result},
    pe::PeImage,
};

#[derive(Debug, Clone)]
pub struct UnicornMips {
    memory: MemoryMap,
    entry: Option<u32>,
    stack_top: Option<u32>,
    mapped_blobs: Vec<MappedBlob>,
    import_traps: ImportTrapTable,
    last_debug: Option<UnicornDebugSnapshot>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UnicornDebugSnapshot {
    pub pc: u32,
    pub ra: u32,
    pub sp: u32,
    pub v0: u32,
    pub v1: u32,
    pub a0: u32,
    pub a1: u32,
    pub a2: u32,
    pub a3: u32,
    pub t9: u32,
    pub trap_address: Option<u32>,
    pub trap_name: Option<String>,
    pub trap_ordinal: Option<u32>,
    pub memory_fault: Option<UnicornMemoryFault>,
    pub function_pointer_probe: Option<UnicornFunctionPointerProbe>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornMemoryFault {
    pub access: String,
    pub address: u32,
    pub size: usize,
    pub value: i64,
    pub pc: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornFunctionPointerProbe {
    pub pc: u32,
    pub slot: u32,
    pub value: u32,
}

const USER_KDATA_PAGE_BASE: u32 = 0x0000_5000;
const USER_KDATA_PAGE_SIZE: u32 = 0x0000_1000;
const USER_KDATA_BASE: u32 = 0x0000_5800;
const USER_KDATA_SYSHANDLE_OFFSET: u32 = 0x0000_0004;
const SYS_HANDLE_CURRENT_THREAD: usize = 1;
const SYS_HANDLE_CURRENT_PROCESS: usize = 2;
const GUEST_HEAP_ARENA_BASE: u32 = 0x3000_0000;
const GUEST_HEAP_ARENA_SIZE: u32 = 0x0100_0000;
#[cfg(feature = "unicorn")]
const MAIN_DESTRUCTOR_BRANCH_PC: u32 = 0x0048_f9cc;
#[cfg(feature = "unicorn")]
const MAIN_DESTRUCTOR_JALR_PC: u32 = 0x0048_f9d4;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct MappedBlob {
    base: u32,
    bytes: Vec<u8>,
}

impl UnicornMips {
    pub fn new() -> Result<Self> {
        Ok(Self {
            memory: MemoryMap::default(),
            entry: None,
            stack_top: None,
            mapped_blobs: Vec::new(),
            import_traps: ImportTrapTable::new(),
            last_debug: None,
        })
    }

    pub fn map_region(
        &mut self,
        base: u32,
        size: u32,
        perms: MemoryPerms,
        name: &str,
    ) -> Result<()> {
        self.memory.map(base, size, perms, name)
    }

    pub fn set_entry(&mut self, entry: u32) {
        self.entry = Some(entry);
    }

    pub fn memory(&self) -> &MemoryMap {
        &self.memory
    }

    pub fn import_traps(&self) -> &ImportTrapTable {
        &self.import_traps
    }

    pub fn last_debug_snapshot(&self) -> Option<&UnicornDebugSnapshot> {
        self.last_debug.as_ref()
    }

    pub fn load_pe_image(&mut self, image: &PeImage) -> Result<()> {
        self.load_pe_image_with_dlls(image, &[])
    }

    pub fn load_pe_image_with_dlls(&mut self, image: &PeImage, dlls: &[PeImage]) -> Result<()> {
        let mut external = ExternalImportTable::default();
        let mut loaded_dlls = Vec::new();
        let mut next_dll_base = 0x6000_0000u32;
        let mut next_trap_base = IMPORT_TRAP_BASE;

        for dll in dlls {
            let load_base = if ranges_overlap(
                image.image_base(),
                image.optional_header.size_of_image,
                dll.image_base(),
                dll.optional_header.size_of_image,
            ) {
                let load_base = next_dll_base;
                next_dll_base = next_dll_base
                    .checked_add(align_up_4k(dll.optional_header.size_of_image)?)
                    .and_then(|base| base.checked_add(0x0010_0000))
                    .ok_or_else(|| Error::InvalidArgument("DLL load base overflow".to_owned()))?;
                load_base
            } else {
                dll.image_base()
            };
            let mut mapped = dll.mapped_image_at(load_base)?;
            let traps = patch_pe_coredll_imports(
                dll,
                &mut mapped,
                &crate::ce::coredll::CoredllExportTable::default(),
                next_trap_base,
            )?;
            next_trap_base = advance_trap_base(next_trap_base, traps.len())?;
            self.import_traps.merge(traps);
            external.add_pe_image(module_file_name(&dll.path), dll, load_base);
            loaded_dlls.push((dll.path.clone(), load_base, mapped));
        }

        let mut mapped = image.mapped_image()?;
        let traps = patch_pe_imports(
            image,
            &mut mapped,
            &crate::ce::coredll::CoredllExportTable::default(),
            next_trap_base,
            &external,
        )?;
        self.map_region(
            image.image_base(),
            align_up_4k(mapped.len() as u32)?,
            MemoryPerms::READ | MemoryPerms::WRITE | MemoryPerms::EXEC,
            "pe-image",
        )?;
        for (path, load_base, mapped) in &loaded_dlls {
            self.map_region(
                *load_base,
                align_up_4k(mapped.len() as u32)?,
                MemoryPerms::READ | MemoryPerms::WRITE | MemoryPerms::EXEC,
                &format!("dll:{path}"),
            )?;
        }
        if !self
            .memory
            .contains_range(IMPORT_TRAP_BASE, IMPORT_TRAP_PAGE_SIZE)
        {
            self.map_region(
                IMPORT_TRAP_BASE,
                IMPORT_TRAP_PAGE_SIZE,
                MemoryPerms::READ | MemoryPerms::EXEC,
                "ce-import-traps",
            )?;
        }
        if !self
            .memory
            .contains_range(USER_KDATA_PAGE_BASE, USER_KDATA_PAGE_SIZE)
        {
            self.map_region(
                USER_KDATA_PAGE_BASE,
                USER_KDATA_PAGE_SIZE,
                MemoryPerms::READ | MemoryPerms::WRITE,
                "ce-user-kdata",
            )?;
        }
        if !self
            .memory
            .contains_range(GUEST_HEAP_ARENA_BASE, GUEST_HEAP_ARENA_SIZE)
        {
            self.map_region(
                GUEST_HEAP_ARENA_BASE,
                GUEST_HEAP_ARENA_SIZE,
                MemoryPerms::READ | MemoryPerms::WRITE,
                "ce-heap-arena",
            )?;
        }
        let stack_size = align_up_4k(image.optional_header.size_of_stack_reserve.max(0x10000))?;
        let stack_top = IMPORT_TRAP_BASE
            .checked_sub(0x10000)
            .ok_or_else(|| Error::InvalidArgument("guest stack top underflow".to_owned()))?;
        let stack_base = stack_top
            .checked_sub(stack_size)
            .ok_or_else(|| Error::InvalidArgument("guest stack base underflow".to_owned()))?;
        self.map_region(
            stack_base,
            stack_size,
            MemoryPerms::READ | MemoryPerms::WRITE,
            "guest-stack",
        )?;
        self.stack_top = Some(stack_top);
        self.entry = Some(image.entry_point_va());
        self.import_traps.merge(traps);
        self.mapped_blobs.push(MappedBlob {
            base: image.image_base(),
            bytes: mapped,
        });
        for (_path, load_base, mapped) in loaded_dlls {
            self.mapped_blobs.push(MappedBlob {
                base: load_base,
                bytes: mapped,
            });
        }
        self.mapped_blobs.push(MappedBlob {
            base: USER_KDATA_PAGE_BASE,
            bytes: user_kdata_page(),
        });
        let trap_page = import_trap_code_page(&self.import_traps);
        self.mapped_blobs.push(MappedBlob {
            base: IMPORT_TRAP_BASE,
            bytes: trap_page,
        });
        Ok(())
    }

    pub fn dispatch_import_trap<M: CoredllGuestMemory>(
        &self,
        kernel: &mut CeKernel,
        memory: &mut M,
        thread_id: u32,
        address: u32,
        args: [u32; 4],
    ) -> Option<u32> {
        self.import_traps
            .dispatch_trap(kernel, memory, thread_id, address, args)
    }

    pub fn run_until_import_trap(&mut self, _kernel: &mut CeKernel) -> Result<()> {
        #[cfg(feature = "unicorn")]
        {
            return self.run_with_unicorn(_kernel);
        }

        #[cfg(not(feature = "unicorn"))]
        Err(Error::Backend(
            "built without the `unicorn` feature; core state is ready but CPU execution is disabled"
                .to_owned(),
        ))
    }

    #[cfg(feature = "unicorn")]
    fn run_with_unicorn(&mut self, kernel: &mut CeKernel) -> Result<()> {
        use std::{cell::RefCell, rc::Rc};
        use unicorn_engine::{
            RegisterMIPS, Unicorn,
            unicorn_const::{Arch, HookType, Mode},
        };

        let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN)
            .map_err(|err| Error::Backend(format!("{err:?}")))?;
        for region in self.memory.regions() {
            uc.mem_map(
                u64::from(region.base),
                u64::from(region.size),
                unicorn_perms(region.perms),
            )
            .map_err(|err| Error::Backend(format!("map {}: {err:?}", region.name)))?;
        }
        for blob in &self.mapped_blobs {
            uc.mem_write(u64::from(blob.base), &blob.bytes)
                .map_err(|err| Error::Backend(format!("write 0x{:08x}: {err:?}", blob.base)))?;
        }
        if let Some(stack_top) = self.stack_top {
            uc.reg_write(RegisterMIPS::SP, u64::from(stack_top))
                .map_err(|err| Error::Backend(format!("set guest SP: {err:?}")))?;
        }

        let traps = self.import_traps.clone();
        let kernel_ptr = kernel as *mut CeKernel;
        let mapped_kernel_memory = Rc::new(RefCell::new(vec![(
            GUEST_HEAP_ARENA_BASE,
            GUEST_HEAP_ARENA_SIZE,
        )]));
        let mapped_kernel_memory_hook = Rc::clone(&mapped_kernel_memory);
        uc.add_code_hook(
            u64::from(IMPORT_TRAP_BASE),
            u64::from(IMPORT_TRAP_BASE + IMPORT_TRAP_PAGE_SIZE - 1),
            move |uc, address, _size| {
                let address = address as u32;
                if traps.trap_at(address).is_none() {
                    return;
                }
                let trap = traps.trap_at(address).cloned();
                if let Some(trap) = trap.as_ref() {
                    tracing::debug!(
                        target: "ce.imports",
                        pc = format_args!("0x{address:08x}"),
                        module = trap.module_name.as_str(),
                        kind = ?trap.module_kind,
                        ordinal = trap.ordinal,
                        name = trap.name.as_deref().unwrap_or("<ordinal>"),
                        a0 = format_args!("0x{:08x}", read_mips_reg(uc, RegisterMIPS::A0)),
                        a1 = format_args!("0x{:08x}", read_mips_reg(uc, RegisterMIPS::A1)),
                        a2 = format_args!("0x{:08x}", read_mips_reg(uc, RegisterMIPS::A2)),
                        a3 = format_args!("0x{:08x}", read_mips_reg(uc, RegisterMIPS::A3)),
                        "import trap"
                    );
                }
                let args = [
                    read_mips_reg(uc, RegisterMIPS::A0),
                    read_mips_reg(uc, RegisterMIPS::A1),
                    read_mips_reg(uc, RegisterMIPS::A2),
                    read_mips_reg(uc, RegisterMIPS::A3),
                ];
                let mut memory = UnicornGuestMemory { uc };
                let Some(result) =
                    traps.dispatch_trap(unsafe { &mut *kernel_ptr }, &mut memory, 1, address, args)
                else {
                    return;
                };
                let _ = map_kernel_memory_allocations(
                    memory.uc,
                    unsafe { &*kernel_ptr },
                    &mut mapped_kernel_memory_hook.borrow_mut(),
                );
                if let Some(trap) = trap.as_ref() {
                    tracing::debug!(
                        target: "ce.imports",
                        pc = format_args!("0x{address:08x}"),
                        module = trap.module_name.as_str(),
                        kind = ?trap.module_kind,
                        ordinal = trap.ordinal,
                        name = trap.name.as_deref().unwrap_or("<ordinal>"),
                        result = format_args!("0x{result:08x}"),
                        "import trap return"
                    );
                }
                let _ = memory.uc.reg_write(RegisterMIPS::V0, u64::from(result));
            },
        )
        .map_err(|err| Error::Backend(format!("install import hook: {err:?}")))?;

        let function_pointer_probe = Rc::new(RefCell::new(None));
        let function_pointer_probe_hook = Rc::clone(&function_pointer_probe);
        uc.add_code_hook(
            u64::from(MAIN_DESTRUCTOR_BRANCH_PC),
            u64::from(MAIN_DESTRUCTOR_JALR_PC),
            move |uc, address, _size| {
                let pc = address as u32;
                if pc != MAIN_DESTRUCTOR_BRANCH_PC && pc != MAIN_DESTRUCTOR_JALR_PC {
                    return;
                }
                *function_pointer_probe_hook.borrow_mut() = Some(UnicornFunctionPointerProbe {
                    pc,
                    slot: read_mips_reg(uc, RegisterMIPS::FP),
                    value: read_mips_reg(uc, RegisterMIPS::V0),
                });
            },
        )
        .map_err(|err| Error::Backend(format!("install function-pointer probe: {err:?}")))?;

        let memory_fault = Rc::new(RefCell::new(None));
        let memory_fault_hook = Rc::clone(&memory_fault);
        uc.add_mem_hook(
            HookType::MEM_UNMAPPED | HookType::MEM_PROT,
            1,
            0,
            move |uc, access, address, size, value| {
                *memory_fault_hook.borrow_mut() = Some(UnicornMemoryFault {
                    access: format!("{access:?}"),
                    address: address as u32,
                    size,
                    value,
                    pc: read_mips_reg(uc, RegisterMIPS::PC),
                });
                false
            },
        )
        .map_err(|err| Error::Backend(format!("install memory fault hook: {err:?}")))?;

        let entry = self
            .entry
            .ok_or_else(|| Error::Backend("no PE entry point has been loaded".to_owned()))?;
        let result = uc.emu_start(u64::from(entry), 0, 0, 0);
        self.last_debug = Some(capture_debug_snapshot(
            &uc,
            &self.import_traps,
            memory_fault.borrow().clone(),
            function_pointer_probe.borrow().clone(),
        ));
        result.map_err(|err| {
            let snapshot = self
                .last_debug
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| "register snapshot unavailable".to_owned());
            Error::Backend(format!("Unicorn run failed: {err:?}; {snapshot}"))
        })
    }
}

fn ranges_overlap(lhs_base: u32, lhs_size: u32, rhs_base: u32, rhs_size: u32) -> bool {
    let lhs_end = lhs_base.saturating_add(lhs_size);
    let rhs_end = rhs_base.saturating_add(rhs_size);
    lhs_base < rhs_end && rhs_base < lhs_end
}

fn module_file_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

fn user_kdata_page() -> Vec<u8> {
    let mut page = vec![0; USER_KDATA_PAGE_SIZE as usize];
    write_user_kdata_handle(&mut page, SYS_HANDLE_CURRENT_THREAD, 1);
    write_user_kdata_handle(&mut page, SYS_HANDLE_CURRENT_PROCESS, 1);
    page
}

fn write_user_kdata_handle(page: &mut [u8], index: usize, value: u32) {
    let offset =
        (USER_KDATA_BASE - USER_KDATA_PAGE_BASE + USER_KDATA_SYSHANDLE_OFFSET) as usize + index * 4;
    page[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn advance_trap_base(current: u32, trap_count: usize) -> Result<u32> {
    let bytes = u32::try_from(trap_count)
        .ok()
        .and_then(|count| count.checked_mul(crate::emulator::imports::IMPORT_TRAP_STRIDE))
        .ok_or_else(|| Error::InvalidArgument("import trap count overflow".to_owned()))?;
    let next = current
        .checked_add(bytes)
        .ok_or_else(|| Error::InvalidArgument("import trap base overflow".to_owned()))?;
    let trap_end = IMPORT_TRAP_BASE
        .checked_add(IMPORT_TRAP_PAGE_SIZE)
        .ok_or_else(|| Error::InvalidArgument("import trap page overflow".to_owned()))?;
    if next >= trap_end {
        return Err(Error::InvalidArgument(
            "import trap page is full".to_owned(),
        ));
    }
    Ok(next)
}

#[cfg(feature = "unicorn")]
fn map_kernel_memory_allocations<D>(
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    kernel: &CeKernel,
    mapped: &mut Vec<(u32, u32)>,
) -> Result<()> {
    for allocation in kernel.memory.allocations() {
        map_guest_range(
            uc,
            mapped,
            allocation.ptr,
            allocation.actual_size,
            "heap allocation",
        )?;
    }
    for allocation in kernel.memory.virtual_allocations() {
        map_guest_range(
            uc,
            mapped,
            allocation.base,
            allocation.size,
            "virtual allocation",
        )?;
    }
    Ok(())
}

#[cfg(feature = "unicorn")]
fn map_guest_range<D>(
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    mapped: &mut Vec<(u32, u32)>,
    base: u32,
    size: u32,
    label: &str,
) -> Result<()> {
    let first_page = base & !0xfff;
    let page_end = base
        .checked_add(size.max(1))
        .and_then(|end| end.checked_add(0xfff))
        .map(|end| end & !0xfff)
        .ok_or_else(|| Error::InvalidArgument(format!("{label} range overflow")))?;
    let mut page_base = first_page;
    while page_base < page_end {
        if mapped.iter().any(|(mapped_base, mapped_size)| {
            page_base >= *mapped_base && page_base < mapped_base.saturating_add(*mapped_size)
        }) {
            page_base = page_base
                .checked_add(0x1000)
                .ok_or_else(|| Error::InvalidArgument(format!("{label} page overflow")))?;
            continue;
        }
        uc.mem_map(
            u64::from(page_base),
            0x1000,
            unicorn_perms(MemoryPerms::READ | MemoryPerms::WRITE),
        )
        .map_err(|err| Error::Backend(format!("map {label} page 0x{page_base:08x}: {err:?}")))?;
        mapped.push((page_base, 0x1000));
        page_base = page_base
            .checked_add(0x1000)
            .ok_or_else(|| Error::InvalidArgument(format!("{label} page overflow")))?;
    }
    Ok(())
}

impl std::fmt::Display for UnicornDebugSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "pc=0x{:08x} ra=0x{:08x} sp=0x{:08x} v0=0x{:08x} v1=0x{:08x} a0=0x{:08x} a1=0x{:08x} a2=0x{:08x} a3=0x{:08x} t9=0x{:08x}",
            self.pc,
            self.ra,
            self.sp,
            self.v0,
            self.v1,
            self.a0,
            self.a1,
            self.a2,
            self.a3,
            self.t9
        )?;
        if let Some(trap_address) = self.trap_address {
            write!(f, " trap=0x{trap_address:08x}")?;
            if let Some(ordinal) = self.trap_ordinal {
                write!(f, " ordinal={ordinal}")?;
            }
            if let Some(name) = self.trap_name.as_deref() {
                write!(f, " name={name}")?;
            }
        }
        if let Some(fault) = self.memory_fault.as_ref() {
            write!(
                f,
                " fault={} addr=0x{:08x} size={} value=0x{:x} fault_pc=0x{:08x}",
                fault.access, fault.address, fault.size, fault.value, fault.pc
            )?;
        }
        if let Some(probe) = self.function_pointer_probe.as_ref() {
            write!(
                f,
                " funcptr_pc=0x{:08x} funcptr_slot=0x{:08x} funcptr_value=0x{:08x}",
                probe.pc, probe.slot, probe.value
            )?;
        }
        Ok(())
    }
}

fn align_up_4k(size: u32) -> Result<u32> {
    size.checked_add(0xfff)
        .map(|size| size & !0xfff)
        .ok_or_else(|| Error::InvalidArgument("mapping size overflow".to_owned()))
}

#[cfg(feature = "unicorn")]
fn unicorn_perms(perms: MemoryPerms) -> unicorn_engine::unicorn_const::Prot {
    use unicorn_engine::unicorn_const::Prot;

    let mut out = Prot::NONE;
    if perms.contains(MemoryPerms::READ) {
        out |= Prot::READ;
    }
    if perms.contains(MemoryPerms::WRITE) {
        out |= Prot::WRITE;
    }
    if perms.contains(MemoryPerms::EXEC) {
        out |= Prot::EXEC;
    }
    out
}

#[cfg(feature = "unicorn")]
fn read_mips_reg<D>(
    uc: &unicorn_engine::Unicorn<'_, D>,
    register: unicorn_engine::RegisterMIPS,
) -> u32 {
    uc.reg_read(register).unwrap_or(0) as u32
}

#[cfg(feature = "unicorn")]
fn capture_debug_snapshot<D>(
    uc: &unicorn_engine::Unicorn<'_, D>,
    traps: &ImportTrapTable,
    memory_fault: Option<UnicornMemoryFault>,
    function_pointer_probe: Option<UnicornFunctionPointerProbe>,
) -> UnicornDebugSnapshot {
    use unicorn_engine::RegisterMIPS;

    let pc = read_mips_reg(uc, RegisterMIPS::PC);
    let trap = traps.trap_at(pc);
    UnicornDebugSnapshot {
        pc,
        ra: read_mips_reg(uc, RegisterMIPS::RA),
        sp: read_mips_reg(uc, RegisterMIPS::SP),
        v0: read_mips_reg(uc, RegisterMIPS::V0),
        v1: read_mips_reg(uc, RegisterMIPS::V1),
        a0: read_mips_reg(uc, RegisterMIPS::A0),
        a1: read_mips_reg(uc, RegisterMIPS::A1),
        a2: read_mips_reg(uc, RegisterMIPS::A2),
        a3: read_mips_reg(uc, RegisterMIPS::A3),
        t9: read_mips_reg(uc, RegisterMIPS::T9),
        trap_address: trap.map(|trap| trap.address),
        trap_name: trap.and_then(|trap| trap.name.clone()),
        trap_ordinal: trap.and_then(|trap| trap.ordinal),
        memory_fault,
        function_pointer_probe,
    }
}

#[cfg(feature = "unicorn")]
struct UnicornGuestMemory<'a, 'uc, D> {
    uc: &'a mut unicorn_engine::Unicorn<'uc, D>,
}

#[cfg(feature = "unicorn")]
impl<D> CoredllGuestMemory for UnicornGuestMemory<'_, '_, D> {
    fn read_u8(&self, addr: u32) -> Result<u8> {
        let mut bytes = [0; 1];
        self.uc
            .mem_read(u64::from(addr), &mut bytes)
            .map_err(|err| Error::Backend(format!("read_u8 0x{addr:08x}: {err:?}")))?;
        Ok(bytes[0])
    }

    fn write_u8(&mut self, addr: u32, value: u8) -> Result<()> {
        self.uc
            .mem_write(u64::from(addr), &[value])
            .map_err(|err| Error::Backend(format!("write_u8 0x{addr:08x}: {err:?}")))
    }

    fn read_u32(&self, addr: u32) -> Result<u32> {
        let mut bytes = [0; 4];
        self.uc
            .mem_read(u64::from(addr), &mut bytes)
            .map_err(|err| Error::Backend(format!("read_u32 0x{addr:08x}: {err:?}")))?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn write_u32(&mut self, addr: u32, value: u32) -> Result<()> {
        self.uc
            .mem_write(u64::from(addr), &value.to_le_bytes())
            .map_err(|err| Error::Backend(format!("write_u32 0x{addr:08x}: {err:?}")))
    }

    fn read_u16(&self, addr: u32) -> Result<u16> {
        let mut bytes = [0; 2];
        self.uc
            .mem_read(u64::from(addr), &mut bytes)
            .map_err(|err| Error::Backend(format!("read_u16 0x{addr:08x}: {err:?}")))?;
        Ok(u16::from_le_bytes(bytes))
    }

    fn write_u16(&mut self, addr: u32, value: u16) -> Result<()> {
        self.uc
            .mem_write(u64::from(addr), &value.to_le_bytes())
            .map_err(|err| Error::Backend(format!("write_u16 0x{addr:08x}: {err:?}")))
    }
}
