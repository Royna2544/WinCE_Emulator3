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
    pub memory_write_probe: Option<UnicornMemoryWriteProbe>,
    pub thread_exit_reached: bool,
    pub encoded_kernel_exit: Option<EncodedKernelExit>,
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
    pub slot_value: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornMemoryWriteProbe {
    pub pc: u32,
    pub ra: u32,
    pub sp: u32,
    pub address: u32,
    pub size: usize,
    pub value: i64,
    pub slot_value_after: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedKernelExit {
    pub target: u32,
    pub api_set: u32,
    pub method: u32,
    pub process: u32,
    pub exit_code: u32,
    pub caller: u32,
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
const THREAD_EXIT_STUB_ADDR: u32 =
    IMPORT_TRAP_BASE + IMPORT_TRAP_PAGE_SIZE - crate::emulator::imports::IMPORT_TRAP_STRIDE;
#[cfg(feature = "unicorn")]
const MAIN_DESTRUCTOR_BRANCH_PC: u32 = 0x0048_f9cc;
#[cfg(feature = "unicorn")]
const MAIN_DESTRUCTOR_JALR_PC: u32 = 0x0048_f9d4;
#[cfg(feature = "unicorn")]
const MAIN_DESTRUCTOR_BAD_SLOT: u32 = 0x3000_2390;
#[cfg(feature = "unicorn")]
const MAIN_DESTRUCTOR_TABLE_WATCH_BASE: u32 = 0x3000_2000;
#[cfg(feature = "unicorn")]
const MAIN_DESTRUCTOR_TABLE_WATCH_END: u32 = 0x3000_24ff;

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
        uc.reg_write(RegisterMIPS::RA, u64::from(THREAD_EXIT_STUB_ADDR))
            .map_err(|err| Error::Backend(format!("set guest RA: {err:?}")))?;

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
                let slot = read_mips_reg(uc, RegisterMIPS::FP);
                *function_pointer_probe_hook.borrow_mut() = Some(UnicornFunctionPointerProbe {
                    pc,
                    slot,
                    value: read_mips_reg(uc, RegisterMIPS::V0),
                    slot_value: read_unicorn_u32(uc, slot),
                });
            },
        )
        .map_err(|err| Error::Backend(format!("install function-pointer probe: {err:?}")))?;

        let memory_write_probe = Rc::new(RefCell::new(None));
        let traps = self.import_traps.clone();
        let kernel_ptr = kernel as *mut CeKernel;
        let mapped_kernel_memory = Rc::new(RefCell::new(vec![(
            GUEST_HEAP_ARENA_BASE,
            GUEST_HEAP_ARENA_SIZE,
        )]));
        let mapped_kernel_memory_hook = Rc::clone(&mapped_kernel_memory);
        let import_memory_write_probe = Rc::clone(&memory_write_probe);
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
                let mut memory = UnicornGuestMemory {
                    uc,
                    memory_write_probe: Some(Rc::clone(&import_memory_write_probe)),
                };
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

        let thread_exit_reached = Rc::new(RefCell::new(false));
        let thread_exit_reached_hook = Rc::clone(&thread_exit_reached);
        uc.add_code_hook(
            u64::from(THREAD_EXIT_STUB_ADDR),
            u64::from(THREAD_EXIT_STUB_ADDR),
            move |uc, _address, _size| {
                *thread_exit_reached_hook.borrow_mut() = true;
                let _ = uc.emu_stop();
            },
        )
        .map_err(|err| Error::Backend(format!("install thread-exit hook: {err:?}")))?;

        let memory_write_probe_hook = Rc::clone(&memory_write_probe);
        uc.add_mem_hook(
            HookType::MEM_WRITE,
            u64::from(MAIN_DESTRUCTOR_TABLE_WATCH_BASE),
            u64::from(MAIN_DESTRUCTOR_TABLE_WATCH_END),
            move |uc, _access, address, size, value| {
                let address = address as u32;
                let end = address.saturating_add(size as u32);
                let touches_bad_slot = address < MAIN_DESTRUCTOR_BAD_SLOT.saturating_add(4)
                    && end > MAIN_DESTRUCTOR_BAD_SLOT;
                let writes_low_pointer = size == 4 && value as u32 == 0x0001_0000;
                if !touches_bad_slot && !writes_low_pointer {
                    return true;
                }
                *memory_write_probe_hook.borrow_mut() = Some(UnicornMemoryWriteProbe {
                    pc: read_mips_reg(uc, RegisterMIPS::PC),
                    ra: read_mips_reg(uc, RegisterMIPS::RA),
                    sp: read_mips_reg(uc, RegisterMIPS::SP),
                    address,
                    size,
                    value,
                    slot_value_after: read_unicorn_u32(uc, MAIN_DESTRUCTOR_BAD_SLOT),
                });
                true
            },
        )
        .map_err(|err| Error::Backend(format!("install memory-write probe: {err:?}")))?;

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
            memory_write_probe.borrow().clone(),
            *thread_exit_reached.borrow(),
        ));
        if let Err(err) = result {
            let decoded_exit = self
                .last_debug
                .as_ref()
                .and_then(|snapshot| self.decode_encoded_kernel_exit(snapshot));
            if let Some(exit) = decoded_exit {
                if let Some(snapshot) = self.last_debug.as_mut() {
                    snapshot.encoded_kernel_exit = Some(exit);
                }
                return Ok(());
            }
            let snapshot = self
                .last_debug
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| "register snapshot unavailable".to_owned());
            return Err(Error::Backend(format!(
                "Unicorn run failed: {err:?}; {snapshot}"
            )));
        }
        Ok(())
    }

    #[cfg(feature = "unicorn")]
    fn decode_encoded_kernel_exit(
        &self,
        snapshot: &UnicornDebugSnapshot,
    ) -> Option<EncodedKernelExit> {
        if snapshot.pc != 0 || snapshot.ra < 12 {
            return None;
        }
        let load_pc = snapshot.ra.wrapping_sub(12);
        let call_pc = snapshot.ra.wrapping_sub(8);
        let load = self.read_mapped_u32(load_pc)?;
        let call = self.read_mapped_u32(call_pc)?;
        let target_reg = decode_addiu_zero(load)?;
        if decode_jalr_register(call)? != target_reg.0 {
            return None;
        }
        let decoded = decode_old_mips_kernel_call(target_reg.1)?;
        Some(EncodedKernelExit {
            target: target_reg.1,
            api_set: decoded.0,
            method: decoded.1,
            process: snapshot.a0,
            exit_code: snapshot.a1,
            caller: call_pc,
        })
    }

    #[cfg(feature = "unicorn")]
    fn read_mapped_u32(&self, address: u32) -> Option<u32> {
        for blob in &self.mapped_blobs {
            let Some(offset) = address.checked_sub(blob.base).map(|offset| offset as usize) else {
                continue;
            };
            let end = offset.checked_add(4)?;
            if end <= blob.bytes.len() {
                return Some(u32::from_le_bytes(blob.bytes[offset..end].try_into().ok()?));
            }
        }
        None
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
            if let Some(slot_value) = probe.slot_value {
                write!(f, " funcptr_slot_value=0x{slot_value:08x}")?;
            }
        }
        if let Some(probe) = self.memory_write_probe.as_ref() {
            write!(
                f,
                " write_pc=0x{:08x} write_ra=0x{:08x} write_sp=0x{:08x} write_addr=0x{:08x} write_size={} write_value=0x{:x}",
                probe.pc, probe.ra, probe.sp, probe.address, probe.size, probe.value
            )?;
            if let Some(slot_value_after) = probe.slot_value_after {
                write!(f, " write_slot_after=0x{slot_value_after:08x}")?;
            }
        }
        if self.thread_exit_reached {
            write!(f, " thread_exit_reached=true")?;
        }
        if let Some(exit) = self.encoded_kernel_exit.as_ref() {
            write!(
                f,
                " encoded_kernel_exit target=0x{:08x} api_set={} method={} process=0x{:08x} exit_code=0x{:08x} caller=0x{:08x}",
                exit.target, exit.api_set, exit.method, exit.process, exit.exit_code, exit.caller
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
fn read_unicorn_u32<D>(uc: &unicorn_engine::Unicorn<'_, D>, address: u32) -> Option<u32> {
    let mut bytes = [0; 4];
    uc.mem_read(u64::from(address), &mut bytes)
        .ok()
        .map(|()| u32::from_le_bytes(bytes))
}

#[cfg(feature = "unicorn")]
fn capture_debug_snapshot<D>(
    uc: &unicorn_engine::Unicorn<'_, D>,
    traps: &ImportTrapTable,
    memory_fault: Option<UnicornMemoryFault>,
    function_pointer_probe: Option<UnicornFunctionPointerProbe>,
    memory_write_probe: Option<UnicornMemoryWriteProbe>,
    thread_exit_reached: bool,
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
        memory_write_probe,
        thread_exit_reached,
        encoded_kernel_exit: None,
    }
}

#[cfg(feature = "unicorn")]
fn decode_addiu_zero(instruction: u32) -> Option<(u32, u32)> {
    let opcode = instruction >> 26;
    let rs = (instruction >> 21) & 0x1f;
    if opcode != 0x09 || rs != 0 {
        return None;
    }
    let rt = (instruction >> 16) & 0x1f;
    let imm = instruction as u16 as i16 as i32 as u32;
    Some((rt, imm))
}

#[cfg(feature = "unicorn")]
fn decode_jalr_register(instruction: u32) -> Option<u32> {
    let opcode = instruction >> 26;
    let function = instruction & 0x3f;
    if opcode != 0 || function != 0x09 {
        return None;
    }
    Some((instruction >> 21) & 0x1f)
}

#[cfg(feature = "unicorn")]
fn decode_old_mips_kernel_call(target: u32) -> Option<(u32, u32)> {
    const OLD_FIRST_METHOD: u32 = 0xffff_fc02;
    const API_CALL_SCALE: u32 = 4;
    const API_SET_SHIFT: u32 = 8;
    const CURRENT_PROCESS_API_SET: u32 = 2;
    const PROC_TERMINATE_METHOD: u32 = 2;

    if target > OLD_FIRST_METHOD {
        return None;
    }
    let delta = OLD_FIRST_METHOD.wrapping_sub(target);
    if delta % API_CALL_SCALE != 0 {
        return None;
    }
    let encoded = delta / API_CALL_SCALE;
    let api_set = encoded >> API_SET_SHIFT;
    let method = encoded & ((1 << API_SET_SHIFT) - 1);
    (api_set == CURRENT_PROCESS_API_SET && method == PROC_TERMINATE_METHOD)
        .then_some((api_set, method))
}

#[cfg(feature = "unicorn")]
struct UnicornGuestMemory<'a, 'uc, D> {
    uc: &'a mut unicorn_engine::Unicorn<'uc, D>,
    memory_write_probe: Option<std::rc::Rc<std::cell::RefCell<Option<UnicornMemoryWriteProbe>>>>,
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
            .map_err(|err| Error::Backend(format!("write_u8 0x{addr:08x}: {err:?}")))?;
        self.record_write(addr, 1, i64::from(value));
        Ok(())
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
            .map_err(|err| Error::Backend(format!("write_u32 0x{addr:08x}: {err:?}")))?;
        self.record_write(addr, 4, i64::from(value));
        Ok(())
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
            .map_err(|err| Error::Backend(format!("write_u16 0x{addr:08x}: {err:?}")))?;
        self.record_write(addr, 2, i64::from(value));
        Ok(())
    }
}

#[cfg(feature = "unicorn")]
impl<D> UnicornGuestMemory<'_, '_, D> {
    fn record_write(&self, address: u32, size: usize, value: i64) {
        let Some(probe) = self.memory_write_probe.as_ref() else {
            return;
        };
        let end = address.saturating_add(size as u32);
        let touches_bad_slot =
            address < MAIN_DESTRUCTOR_BAD_SLOT.saturating_add(4) && end > MAIN_DESTRUCTOR_BAD_SLOT;
        let writes_low_pointer = size == 4 && value as u32 == 0x0001_0000;
        if !touches_bad_slot && !writes_low_pointer {
            return;
        }
        *probe.borrow_mut() = Some(UnicornMemoryWriteProbe {
            pc: read_mips_reg(self.uc, unicorn_engine::RegisterMIPS::PC),
            ra: read_mips_reg(self.uc, unicorn_engine::RegisterMIPS::RA),
            sp: read_mips_reg(self.uc, unicorn_engine::RegisterMIPS::SP),
            address,
            size,
            value,
            slot_value_after: read_unicorn_u32(self.uc, MAIN_DESTRUCTOR_BAD_SLOT),
        });
    }
}
