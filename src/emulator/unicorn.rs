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
    #[cfg(feature = "unicorn")]
    trampoline_ranges: Vec<(u32, u32)>,
    #[cfg(feature = "unicorn")]
    trampoline_jumps: Vec<(u32, u32)>,
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
    pub indirect_call_probe: Option<UnicornIndirectCallProbe>,
    pub memory_write_probe: Option<UnicornMemoryWriteProbe>,
    pub interrupt_probe: Option<UnicornInterruptProbe>,
    pub invalid_instruction_probe: Option<UnicornInvalidInstructionProbe>,
    pub last_code: Vec<UnicornLastCode>,
    pub last_blocks: Vec<UnicornLastBlock>,
    pub blocked_get_message: Option<UnicornBlockedGetMessage>,
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
pub struct UnicornIndirectCallProbe {
    pub pc: u32,
    pub ra: u32,
    pub sp: u32,
    pub instruction: u32,
    pub register: u32,
    pub register_name: &'static str,
    pub target: u32,
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
pub struct UnicornInterruptProbe {
    pub pc: u32,
    pub ra: u32,
    pub sp: u32,
    pub intno: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornInvalidInstructionProbe {
    pub pc: u32,
    pub ra: u32,
    pub sp: u32,
    pub instruction: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornLastCode {
    pub pc: u32,
    pub ra: u32,
    pub sp: u32,
    pub instruction: Option<u32>,
    pub next_instruction: Option<u32>,
    pub direct_jump_target: Option<u32>,
    pub direct_jump_target_instruction: Option<u32>,
    pub direct_jump_target_in_trampoline: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornLastBlock {
    pub pc: u32,
    pub size: u32,
    pub ra: u32,
    pub sp: u32,
    pub instruction: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornBlockedGetMessage {
    pub thread_id: u32,
    pub hwnd: Option<u32>,
    pub min_msg: u32,
    pub max_msg: u32,
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
const IMPORT_TRAP_ARG_COUNT: usize = 12;
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
#[cfg(feature = "unicorn")]
const IMAGE_SCN_MEM_EXECUTE: u32 = 0x2000_0000;
#[cfg(feature = "unicorn")]
const MIPS_NOP: u32 = 0x0000_0000;

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
            #[cfg(feature = "unicorn")]
            trampoline_ranges: Vec::new(),
            #[cfg(feature = "unicorn")]
            trampoline_jumps: Vec::new(),
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
            #[cfg(feature = "unicorn")]
            {
                let trampoline_patch = patch_mips_unicorn_trampolines(dll, load_base, &mut mapped)?;
                if let Some(range) = trampoline_patch.range {
                    self.trampoline_ranges.push(range);
                }
                self.trampoline_jumps.extend(trampoline_patch.jumps);
            }
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
        #[cfg(feature = "unicorn")]
        {
            let trampoline_patch =
                patch_mips_unicorn_trampolines(image, image.image_base(), &mut mapped)?;
            if let Some(range) = trampoline_patch.range {
                self.trampoline_ranges.push(range);
            }
            self.trampoline_jumps.extend(trampoline_patch.jumps);
        }
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
            .dispatch_trap(kernel, memory, thread_id, address, args.to_vec())
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

        let indirect_call_probe = Rc::new(RefCell::new(None));
        let indirect_call_probe_hook = Rc::clone(&indirect_call_probe);
        let last_code = Rc::new(RefCell::new(Vec::<UnicornLastCode>::new()));
        let last_code_hook = Rc::clone(&last_code);
        let trampoline_ranges = self.trampoline_ranges.clone();
        let trampoline_jumps = self.trampoline_jumps.clone();
        uc.add_code_hook(1, 0, move |uc, address, _size| {
            let pc = address as u32;
            let instruction = read_unicorn_u32(uc, pc);
            let next_instruction = read_unicorn_u32(uc, pc.wrapping_add(4));
            let direct_jump_target =
                instruction.and_then(|instruction| decode_direct_jump_target(pc, instruction));
            let direct_jump_target_instruction =
                direct_jump_target.and_then(|target| read_unicorn_u32(uc, target));
            let sentinel_target =
                instruction
                    .zip(next_instruction)
                    .and_then(|(instruction, next_instruction)| {
                        decode_trampoline_sentinel_target(instruction, next_instruction)
                    });
            if let Some((register, target)) = instruction
                .and_then(decode_jr_register)
                .and_then(|register| read_mips_gpr(uc, register).map(|target| (register, target)))
            {
                if let Some((_, trampoline_target)) = trampoline_jumps
                    .iter()
                    .find(|(origin, _)| *origin == target)
                {
                    let _ = write_mips_gpr(uc, register, *trampoline_target);
                }
            }
            if let Some(target) = sentinel_target {
                if target_in_ranges(target, &trampoline_ranges) {
                    let _ = uc.reg_write(RegisterMIPS::PC, u64::from(target));
                    return;
                }
            }
            let direct_jump_target_in_trampoline = direct_jump_target
                .is_some_and(|target| target_in_ranges(target, &trampoline_ranges));
            {
                let mut last_code = last_code_hook.borrow_mut();
                if last_code.len() == 12 {
                    last_code.remove(0);
                }
                last_code.push(UnicornLastCode {
                    pc,
                    ra: read_mips_reg(uc, RegisterMIPS::RA),
                    sp: read_mips_reg(uc, RegisterMIPS::SP),
                    instruction,
                    next_instruction,
                    direct_jump_target,
                    direct_jump_target_instruction,
                    direct_jump_target_in_trampoline,
                });
            }
            if let (Some(instruction), Some(next_instruction), Some(target)) =
                (instruction, next_instruction, direct_jump_target)
            {
                if is_patched_trampoline_jump(
                    instruction,
                    next_instruction,
                    target,
                    &trampoline_ranges,
                ) {
                    let _ = uc.reg_write(RegisterMIPS::PC, u64::from(target));
                    return;
                }
            }
            let Some(instruction) = instruction else {
                return;
            };
            let Some(register) = decode_indirect_call_register(instruction) else {
                return;
            };
            let Some(target) = read_mips_gpr(uc, register) else {
                return;
            };
            if target != 0 && target >= 0x0001_0000 {
                return;
            }
            *indirect_call_probe_hook.borrow_mut() = Some(UnicornIndirectCallProbe {
                pc,
                ra: read_mips_reg(uc, RegisterMIPS::RA),
                sp: read_mips_reg(uc, RegisterMIPS::SP),
                instruction,
                register,
                register_name: mips_gpr_name(register),
                target,
            });
        })
        .map_err(|err| Error::Backend(format!("install indirect-call probe: {err:?}")))?;

        let last_blocks = Rc::new(RefCell::new(Vec::<UnicornLastBlock>::new()));
        let last_blocks_hook = Rc::clone(&last_blocks);
        uc.add_block_hook(1, 0, move |uc, address, size| {
            let mut last_blocks = last_blocks_hook.borrow_mut();
            if last_blocks.len() == 16 {
                last_blocks.remove(0);
            }
            let pc = address as u32;
            last_blocks.push(UnicornLastBlock {
                pc,
                size: size as u32,
                ra: read_mips_reg(uc, RegisterMIPS::RA),
                sp: read_mips_reg(uc, RegisterMIPS::SP),
                instruction: read_unicorn_u32(uc, pc),
            });
        })
        .map_err(|err| Error::Backend(format!("install block trace hook: {err:?}")))?;

        let memory_write_probe = Rc::new(RefCell::new(None));
        let blocked_get_message = Rc::new(RefCell::new(None));
        let blocked_get_message_hook = Rc::clone(&blocked_get_message);
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
                        sp = format_args!("0x{:08x}", read_mips_reg(uc, RegisterMIPS::SP)),
                        "import trap"
                    );
                }
                let args = read_mips_import_args(uc);
                if trap.as_ref().is_some_and(|trap| {
                    try_block_empty_get_message(
                        unsafe { &mut *kernel_ptr },
                        trap.module_kind,
                        trap.ordinal,
                        &args,
                        &blocked_get_message_hook,
                    )
                }) {
                    let _ = uc.emu_stop();
                    return;
                }
                let mut memory = UnicornGuestMemory {
                    uc,
                    memory_write_probe: Some(Rc::clone(&import_memory_write_probe)),
                };
                if trap.as_ref().is_some_and(|trap| {
                    try_enter_dispatch_message_callout(
                        unsafe { &*kernel_ptr },
                        memory.uc,
                        trap.module_kind,
                        trap.ordinal,
                        &args,
                    )
                }) {
                    return;
                }
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

        let interrupt_probe = Rc::new(RefCell::new(None));
        let interrupt_probe_hook = Rc::clone(&interrupt_probe);
        uc.add_intr_hook(move |uc, intno| {
            *interrupt_probe_hook.borrow_mut() = Some(UnicornInterruptProbe {
                pc: read_mips_reg(uc, RegisterMIPS::PC),
                ra: read_mips_reg(uc, RegisterMIPS::RA),
                sp: read_mips_reg(uc, RegisterMIPS::SP),
                intno: intno as u32,
            });
            let _ = uc.emu_stop();
        })
        .map_err(|err| Error::Backend(format!("install interrupt probe: {err:?}")))?;

        let invalid_instruction_probe = Rc::new(RefCell::new(None));
        let invalid_instruction_probe_hook = Rc::clone(&invalid_instruction_probe);
        uc.add_insn_invalid_hook(move |uc| {
            let pc = read_mips_reg(uc, RegisterMIPS::PC);
            *invalid_instruction_probe_hook.borrow_mut() = Some(UnicornInvalidInstructionProbe {
                pc,
                ra: read_mips_reg(uc, RegisterMIPS::RA),
                sp: read_mips_reg(uc, RegisterMIPS::SP),
                instruction: read_unicorn_u32(uc, pc),
            });
            false
        })
        .map_err(|err| Error::Backend(format!("install invalid-instruction probe: {err:?}")))?;

        let entry = self
            .entry
            .ok_or_else(|| Error::Backend("no PE entry point has been loaded".to_owned()))?;
        let result = uc.emu_start(u64::from(entry), 0, 0, 0);
        self.last_debug = Some(capture_debug_snapshot(
            &uc,
            &self.import_traps,
            memory_fault.borrow().clone(),
            function_pointer_probe.borrow().clone(),
            indirect_call_probe.borrow().clone(),
            memory_write_probe.borrow().clone(),
            interrupt_probe.borrow().clone(),
            invalid_instruction_probe.borrow().clone(),
            last_code.borrow().clone(),
            last_blocks.borrow().clone(),
            blocked_get_message.borrow().clone(),
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
        if self
            .last_debug
            .as_ref()
            .is_some_and(|snapshot| snapshot.interrupt_probe.is_some())
        {
            let snapshot = self
                .last_debug
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| "register snapshot unavailable".to_owned());
            return Err(Error::Backend(format!(
                "Unicorn run stopped on guest CPU exception; {snapshot}"
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

#[cfg(feature = "unicorn")]
fn patch_mips_unicorn_trampolines(
    image: &PeImage,
    load_base: u32,
    mapped: &mut Vec<u8>,
) -> Result<MipsTrampolinePatchResult> {
    let mut patches = Vec::new();
    for section in &image.sections {
        if section.characteristics & IMAGE_SCN_MEM_EXECUTE == 0 {
            continue;
        }
        let section_size = section.virtual_size.max(section.size_of_raw_data);
        let start = section.virtual_address;
        let Some(end) = start.checked_add(section_size) else {
            return Err(Error::InvalidArgument(format!(
                "{} section {} overflows",
                image.path, section.name
            )));
        };
        let mut rva = start;
        while rva.checked_add(8).is_some_and(|next| next <= end) {
            let instruction = read_mapped_word(mapped, rva, &image.path)?;
            let delay_slot = read_mapped_word(mapped, rva + 4, &image.path)?;
            if let Some(branch) = decode_mips_branch_likely(instruction) {
                let pc = load_base.wrapping_add(rva);
                patches.push(MipsUnicornPatch {
                    rva,
                    pc,
                    kind: MipsUnicornPatchKind::BranchLikely { branch, delay_slot },
                });
            } else if let Some(branch) = decode_mips_normal_branch(instruction) {
                let pc = load_base.wrapping_add(rva);
                patches.push(MipsUnicornPatch {
                    rva,
                    pc,
                    kind: MipsUnicornPatchKind::Branch { branch, delay_slot },
                });
            } else if let Some(target) =
                decode_mips_jal_target(load_base.wrapping_add(rva), instruction)
            {
                let pc = load_base.wrapping_add(rva);
                patches.push(MipsUnicornPatch {
                    rva,
                    pc,
                    kind: MipsUnicornPatchKind::Jal { target, delay_slot },
                });
            }
            rva = rva
                .checked_add(4)
                .ok_or_else(|| Error::InvalidArgument("section scan overflow".to_owned()))?;
        }
    }

    if patches.is_empty() {
        return Ok(MipsTrampolinePatchResult::default());
    }

    let aligned_len = align_up_4k(mapped.len() as u32)? as usize;
    if mapped.len() < aligned_len {
        mapped.resize(aligned_len, 0);
    }
    let mut stub_rva = aligned_len as u32;
    let mut trampoline_jumps = Vec::with_capacity(patches.len());
    for patch in patches {
        let stub_pc = load_base.wrapping_add(stub_rva);
        let stub_words = match patch.kind {
            MipsUnicornPatchKind::BranchLikely { branch, delay_slot } => {
                branch_likely_stub_words(patch.pc, branch, delay_slot, stub_pc)?
            }
            MipsUnicornPatchKind::Branch { branch, delay_slot } => {
                normal_branch_stub_words(patch.pc, branch, delay_slot, stub_pc)?
            }
            MipsUnicornPatchKind::Jal { target, delay_slot } => {
                jal_stub_words(patch.pc, target, delay_slot, stub_pc)?
            }
        };
        write_mapped_word(
            mapped,
            patch.rva,
            encode_mips_lui(26, stub_pc >> 16),
            &image.path,
        )?;
        write_mapped_word(
            mapped,
            patch.rva + 4,
            encode_mips_ori(26, 26, stub_pc & 0xffff),
            &image.path,
        )?;
        trampoline_jumps.push((patch.pc, stub_pc));

        let stub_offset = stub_rva as usize;
        let stub_end = stub_offset
            .checked_add(stub_words.len() * 4)
            .ok_or_else(|| Error::InvalidArgument("branch-likely stub overflow".to_owned()))?;
        if mapped.len() < stub_end {
            mapped.resize(stub_end, 0);
        }
        for (index, word) in stub_words.into_iter().enumerate() {
            let offset = stub_offset + index * 4;
            mapped[offset..offset + 4].copy_from_slice(&word.to_le_bytes());
        }
        stub_rva = stub_rva
            .checked_add((stub_end - stub_offset) as u32)
            .ok_or_else(|| Error::InvalidArgument("branch-likely stub RVA overflow".to_owned()))?;
    }
    let final_len = align_up_4k(stub_rva)? as usize;
    if mapped.len() < final_len {
        mapped.resize(final_len, 0);
    }
    let range_base = load_base.wrapping_add(aligned_len as u32);
    let range_size = final_len
        .checked_sub(aligned_len)
        .and_then(|size| u32::try_from(size).ok())
        .ok_or_else(|| Error::InvalidArgument("branch trampoline range overflow".to_owned()))?;
    Ok(MipsTrampolinePatchResult {
        range: Some((range_base, range_size)),
        jumps: trampoline_jumps,
    })
}

#[cfg(feature = "unicorn")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct MipsTrampolinePatchResult {
    range: Option<(u32, u32)>,
    jumps: Vec<(u32, u32)>,
}

#[cfg(feature = "unicorn")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MipsUnicornPatch {
    rva: u32,
    pc: u32,
    kind: MipsUnicornPatchKind,
}

#[cfg(feature = "unicorn")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MipsUnicornPatchKind {
    BranchLikely {
        branch: MipsBranchLikely,
        delay_slot: u32,
    },
    Branch {
        branch: MipsBranchLikely,
        delay_slot: u32,
    },
    Jal {
        target: u32,
        delay_slot: u32,
    },
}

#[cfg(feature = "unicorn")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MipsBranchLikely {
    rs: u32,
    rt: u32,
    target: u32,
    inverse_branch: MipsBranch,
    link: bool,
}

#[cfg(feature = "unicorn")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MipsBranch {
    Beq,
    Bne,
    Blez,
    Bgtz,
    Bltz,
    Bgez,
}

#[cfg(feature = "unicorn")]
fn decode_mips_branch_likely(instruction: u32) -> Option<MipsBranchLikely> {
    let opcode = instruction >> 26;
    let rs = (instruction >> 21) & 0x1f;
    let rt = (instruction >> 16) & 0x1f;
    let imm = instruction as u16 as i16 as i32;
    let target = (4i32.wrapping_add(imm.wrapping_shl(2))) as u32;
    let relative_target = |pc: u32| pc.wrapping_add(target);

    let inverse_branch = match opcode {
        0x14 => MipsBranch::Bne,
        0x15 => MipsBranch::Beq,
        0x16 => MipsBranch::Bgtz,
        0x17 => MipsBranch::Blez,
        0x01 => match rt {
            0x02 => MipsBranch::Bgez,
            0x03 => MipsBranch::Bltz,
            0x12 => MipsBranch::Bgez,
            0x13 => MipsBranch::Bltz,
            _ => return None,
        },
        _ => return None,
    };
    let link = opcode == 0x01 && matches!(rt, 0x12 | 0x13);
    Some(MipsBranchLikely {
        rs,
        rt,
        target: relative_target(0),
        inverse_branch,
        link,
    })
}

#[cfg(feature = "unicorn")]
fn decode_mips_normal_branch(instruction: u32) -> Option<MipsBranchLikely> {
    let opcode = instruction >> 26;
    let rs = (instruction >> 21) & 0x1f;
    let rt = (instruction >> 16) & 0x1f;
    let imm = instruction as u16 as i16 as i32;
    let target = (4i32.wrapping_add(imm.wrapping_shl(2))) as u32;

    let inverse_branch = match opcode {
        0x04 => MipsBranch::Bne,
        0x05 => MipsBranch::Beq,
        0x06 => MipsBranch::Bgtz,
        0x07 => MipsBranch::Blez,
        0x01 => match rt {
            0x00 => MipsBranch::Bgez,
            0x01 => MipsBranch::Bltz,
            _ => return None,
        },
        _ => return None,
    };
    Some(MipsBranchLikely {
        rs,
        rt,
        target,
        inverse_branch,
        link: false,
    })
}

#[cfg(feature = "unicorn")]
fn decode_mips_jal_target(pc: u32, instruction: u32) -> Option<u32> {
    let opcode = instruction >> 26;
    if opcode != 0x03 {
        return None;
    }
    let index = instruction & 0x03ff_ffff;
    Some((pc.wrapping_add(4) & 0xf000_0000) | (index << 2))
}

#[cfg(feature = "unicorn")]
fn decode_trampoline_sentinel_target(instruction: u32, next_instruction: u32) -> Option<u32> {
    let opcode = instruction >> 26;
    let rt = (instruction >> 16) & 0x1f;
    if opcode != 0x0f || rt != 26 {
        return None;
    }
    let next_opcode = next_instruction >> 26;
    let next_rs = (next_instruction >> 21) & 0x1f;
    let next_rt = (next_instruction >> 16) & 0x1f;
    if next_opcode != 0x0d || next_rs != 26 || next_rt != 26 {
        return None;
    }
    Some(((instruction & 0xffff) << 16) | (next_instruction & 0xffff))
}

#[cfg(feature = "unicorn")]
fn is_patched_trampoline_jump(
    instruction: u32,
    next_instruction: u32,
    target: u32,
    trampoline_ranges: &[(u32, u32)],
) -> bool {
    let opcode = instruction >> 26;
    opcode == 0x02 && next_instruction == MIPS_NOP && target_in_ranges(target, trampoline_ranges)
}

#[cfg(feature = "unicorn")]
fn target_in_ranges(target: u32, ranges: &[(u32, u32)]) -> bool {
    ranges.iter().any(|(base, size)| {
        let end = base.saturating_add(*size);
        target >= *base && target < end
    })
}

#[cfg(feature = "unicorn")]
fn jal_stub_words(pc: u32, target: u32, delay_slot: u32, stub_pc: u32) -> Result<Vec<u32>> {
    let link_address = pc.wrapping_add(8);
    Ok(vec![
        encode_mips_lui(31, link_address >> 16),
        encode_mips_ori(31, 31, link_address & 0xffff),
        delay_slot,
        encode_mips_jump(stub_pc.wrapping_add(12), target)?,
        MIPS_NOP,
    ])
}

#[cfg(feature = "unicorn")]
fn branch_likely_stub_words(
    pc: u32,
    mut branch: MipsBranchLikely,
    delay_slot: u32,
    stub_pc: u32,
) -> Result<Vec<u32>> {
    branch.target = pc.wrapping_add(branch.target);
    let fallthrough = pc.wrapping_add(8);
    let false_path_index = if branch.link { 7 } else { 5 };
    let false_path_pc = stub_pc.wrapping_add(false_path_index * 4);

    let mut words = vec![
        encode_mips_cond_branch(
            branch.inverse_branch,
            branch.rs,
            branch.rt,
            stub_pc,
            false_path_pc,
        )?,
        MIPS_NOP,
    ];
    if branch.link {
        let link_address = pc.wrapping_add(8);
        words.push(encode_mips_lui(31, link_address >> 16));
        words.push(encode_mips_ori(31, 31, link_address & 0xffff));
    }
    let true_jump_pc = stub_pc.wrapping_add((words.len() as u32 + 1) * 4);
    words.extend([
        delay_slot,
        encode_mips_jump(true_jump_pc, branch.target)?,
        MIPS_NOP,
        encode_mips_jump(false_path_pc, fallthrough)?,
        MIPS_NOP,
    ]);
    Ok(words)
}

#[cfg(feature = "unicorn")]
fn normal_branch_stub_words(
    pc: u32,
    mut branch: MipsBranchLikely,
    delay_slot: u32,
    stub_pc: u32,
) -> Result<Vec<u32>> {
    branch.target = pc.wrapping_add(branch.target);
    let fallthrough = pc.wrapping_add(8);
    let false_path_pc = stub_pc.wrapping_add(20);

    Ok(vec![
        encode_mips_cond_branch(
            branch.inverse_branch,
            branch.rs,
            branch.rt,
            stub_pc,
            false_path_pc,
        )?,
        MIPS_NOP,
        delay_slot,
        encode_mips_jump(stub_pc.wrapping_add(12), branch.target)?,
        MIPS_NOP,
        delay_slot,
        encode_mips_jump(false_path_pc.wrapping_add(4), fallthrough)?,
        MIPS_NOP,
    ])
}

#[cfg(feature = "unicorn")]
fn encode_mips_cond_branch(
    branch: MipsBranch,
    rs: u32,
    rt: u32,
    pc: u32,
    target: u32,
) -> Result<u32> {
    let offset = branch_offset(pc, target)?;
    let instruction = match branch {
        MipsBranch::Beq => (0x04 << 26) | (rs << 21) | (rt << 16),
        MipsBranch::Bne => (0x05 << 26) | (rs << 21) | (rt << 16),
        MipsBranch::Blez => (0x06 << 26) | (rs << 21),
        MipsBranch::Bgtz => (0x07 << 26) | (rs << 21),
        MipsBranch::Bltz => (0x01 << 26) | (rs << 21),
        MipsBranch::Bgez => (0x01 << 26) | (rs << 21) | (0x01 << 16),
    };
    Ok(instruction | u32::from(offset as u16))
}

#[cfg(feature = "unicorn")]
fn branch_offset(pc: u32, target: u32) -> Result<i16> {
    let delta = target as i64 - pc.wrapping_add(4) as i64;
    if delta % 4 != 0 {
        return Err(Error::InvalidArgument(format!(
            "unaligned MIPS branch target 0x{target:08x}"
        )));
    }
    let offset = delta / 4;
    i16::try_from(offset).map_err(|_| {
        Error::InvalidArgument(format!(
            "MIPS branch target 0x{target:08x} is out of trampoline range from 0x{pc:08x}"
        ))
    })
}

#[cfg(feature = "unicorn")]
fn encode_mips_jump(pc: u32, target: u32) -> Result<u32> {
    if pc.wrapping_add(4) & 0xf000_0000 != target & 0xf000_0000 {
        return Err(Error::InvalidArgument(format!(
            "MIPS jump target 0x{target:08x} is outside direct jump region from 0x{pc:08x}"
        )));
    }
    Ok((0x02 << 26) | ((target >> 2) & 0x03ff_ffff))
}

#[cfg(feature = "unicorn")]
fn encode_mips_lui(rt: u32, imm: u32) -> u32 {
    (0x0f << 26) | (rt << 16) | (imm & 0xffff)
}

#[cfg(feature = "unicorn")]
fn encode_mips_ori(rt: u32, rs: u32, imm: u32) -> u32 {
    (0x0d << 26) | (rs << 21) | (rt << 16) | (imm & 0xffff)
}

#[cfg(feature = "unicorn")]
fn read_mapped_word(mapped: &[u8], rva: u32, path: &str) -> Result<u32> {
    let offset = rva as usize;
    let end = offset
        .checked_add(4)
        .ok_or_else(|| Error::InvalidArgument(format!("{path} mapped read overflows")))?;
    let bytes = mapped.get(offset..end).ok_or_else(|| {
        Error::InvalidArgument(format!(
            "{path} mapped read RVA 0x{rva:08x} is outside image"
        ))
    })?;
    Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
}

#[cfg(feature = "unicorn")]
fn write_mapped_word(mapped: &mut [u8], rva: u32, value: u32, path: &str) -> Result<()> {
    let offset = rva as usize;
    let end = offset
        .checked_add(4)
        .ok_or_else(|| Error::InvalidArgument(format!("{path} mapped write overflows")))?;
    let bytes = mapped.get_mut(offset..end).ok_or_else(|| {
        Error::InvalidArgument(format!(
            "{path} mapped write RVA 0x{rva:08x} is outside image"
        ))
    })?;
    bytes.copy_from_slice(&value.to_le_bytes());
    Ok(())
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
        if let Some(probe) = self.indirect_call_probe.as_ref() {
            write!(
                f,
                " indirect_pc=0x{:08x} indirect_ra=0x{:08x} indirect_sp=0x{:08x} indirect_insn=0x{:08x} indirect_reg=${}({}) indirect_target=0x{:08x}",
                probe.pc,
                probe.ra,
                probe.sp,
                probe.instruction,
                probe.register,
                probe.register_name,
                probe.target
            )?;
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
        if let Some(probe) = self.interrupt_probe.as_ref() {
            write!(
                f,
                " interrupt_pc=0x{:08x} interrupt_ra=0x{:08x} interrupt_sp=0x{:08x} interrupt_no={}",
                probe.pc, probe.ra, probe.sp, probe.intno
            )?;
        }
        if let Some(probe) = self.invalid_instruction_probe.as_ref() {
            write!(
                f,
                " invalid_pc=0x{:08x} invalid_ra=0x{:08x} invalid_sp=0x{:08x}",
                probe.pc, probe.ra, probe.sp
            )?;
            if let Some(instruction) = probe.instruction {
                write!(f, " invalid_insn=0x{instruction:08x}")?;
            }
        }
        if !self.last_code.is_empty() {
            write!(f, " last_code=[")?;
            for (index, code) in self.last_code.iter().enumerate() {
                if index != 0 {
                    write!(f, ",")?;
                }
                write!(
                    f,
                    "0x{:08x}/ra=0x{:08x}/sp=0x{:08x}",
                    code.pc, code.ra, code.sp
                )?;
                if let Some(instruction) = code.instruction {
                    write!(f, "/insn=0x{instruction:08x}")?;
                }
                if let Some(next_instruction) = code.next_instruction {
                    write!(f, "/next=0x{next_instruction:08x}")?;
                }
                if let Some(target) = code.direct_jump_target {
                    write!(f, "/jt=0x{target:08x}")?;
                    if code.direct_jump_target_in_trampoline {
                        write!(f, "/jt_trampoline=true")?;
                    }
                    if let Some(target_instruction) = code.direct_jump_target_instruction {
                        write!(f, "/jt_insn=0x{target_instruction:08x}")?;
                    } else {
                        write!(f, "/jt_insn=<unreadable>")?;
                    }
                }
            }
            write!(f, "]")?;
        }
        if !self.last_blocks.is_empty() {
            write!(f, " last_blocks=[")?;
            for (index, block) in self.last_blocks.iter().enumerate() {
                if index != 0 {
                    write!(f, ",")?;
                }
                write!(
                    f,
                    "0x{:08x}/size={}/ra=0x{:08x}/sp=0x{:08x}",
                    block.pc, block.size, block.ra, block.sp
                )?;
                if let Some(instruction) = block.instruction {
                    write!(f, "/insn=0x{instruction:08x}")?;
                }
            }
            write!(f, "]")?;
        }
        if let Some(blocked) = self.blocked_get_message.as_ref() {
            write!(
                f,
                " blocked_get_message thread_id={} hwnd={} min_msg=0x{:08x} max_msg=0x{:08x}",
                blocked.thread_id,
                blocked
                    .hwnd
                    .map(|hwnd| format!("0x{hwnd:08x}"))
                    .unwrap_or_else(|| "<any>".to_owned()),
                blocked.min_msg,
                blocked.max_msg
            )?;
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
fn read_mips_import_args<D>(uc: &unicorn_engine::Unicorn<'_, D>) -> Vec<u32> {
    let mut args = Vec::with_capacity(IMPORT_TRAP_ARG_COUNT);
    args.extend([
        read_mips_reg(uc, unicorn_engine::RegisterMIPS::A0),
        read_mips_reg(uc, unicorn_engine::RegisterMIPS::A1),
        read_mips_reg(uc, unicorn_engine::RegisterMIPS::A2),
        read_mips_reg(uc, unicorn_engine::RegisterMIPS::A3),
    ]);

    let sp = read_mips_reg(uc, unicorn_engine::RegisterMIPS::SP);
    for stack_index in 0..IMPORT_TRAP_ARG_COUNT.saturating_sub(4) {
        let offset = 16 + (stack_index as u32 * 4);
        let value = sp
            .checked_add(offset)
            .and_then(|addr| read_unicorn_u32(uc, addr))
            .unwrap_or(0);
        args.push(value);
    }
    args
}

#[cfg(feature = "unicorn")]
fn try_block_empty_get_message(
    kernel: &mut CeKernel,
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    args: &[u32],
    blocked: &std::rc::Rc<std::cell::RefCell<Option<UnicornBlockedGetMessage>>>,
) -> bool {
    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll
        || ordinal != Some(crate::ce::coredll_ordinals::ORD_GET_MESSAGE_W)
    {
        return false;
    }

    let hwnd = args.get(1).copied().filter(|hwnd| *hwnd != 0);
    let min_msg = args.get(2).copied().unwrap_or(0);
    let max_msg = args.get(3).copied().unwrap_or(0);
    let thread_id = 1;
    kernel.pump_timers_to_gwe(thread_id);
    if kernel
        .gwe
        .peek_message_filtered(
            thread_id,
            hwnd,
            min_msg,
            max_msg,
            crate::ce::gwe::PeekFlags::empty(),
        )
        .is_some()
    {
        return false;
    }

    *blocked.borrow_mut() = Some(UnicornBlockedGetMessage {
        thread_id,
        hwnd,
        min_msg,
        max_msg,
    });
    true
}

#[cfg(feature = "unicorn")]
fn try_enter_dispatch_message_callout<D>(
    kernel: &CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    args: &[u32],
) -> bool {
    use unicorn_engine::RegisterMIPS;

    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll
        || ordinal != Some(crate::ce::coredll_ordinals::ORD_DISPATCH_MESSAGE_W)
    {
        return false;
    }

    let msg_ptr = args.first().copied().unwrap_or(0);
    let Some(hwnd) = read_unicorn_u32(uc, msg_ptr) else {
        return false;
    };
    let Some(msg) = read_unicorn_u32(uc, msg_ptr.wrapping_add(4)) else {
        return false;
    };
    if msg == crate::ce::gwe::WM_QUIT {
        return false;
    }
    let Some(wparam) = read_unicorn_u32(uc, msg_ptr.wrapping_add(8)) else {
        return false;
    };
    let Some(lparam) = read_unicorn_u32(uc, msg_ptr.wrapping_add(12)) else {
        return false;
    };
    let Some(window) = kernel.gwe.window(hwnd) else {
        return false;
    };
    let wndproc = window.wndproc;
    if wndproc == 0 {
        return false;
    }

    tracing::debug!(
        target: "ce.gwe",
        msg_ptr = format_args!("0x{msg_ptr:08x}"),
        hwnd = format_args!("0x{hwnd:08x}"),
        msg = format_args!("0x{msg:08x}"),
        wparam = format_args!("0x{wparam:08x}"),
        lparam = format_args!("0x{lparam:08x}"),
        class = window.class_name.as_str(),
        wndproc = format_args!("0x{wndproc:08x}"),
        ra = format_args!("0x{:08x}", read_mips_reg(uc, RegisterMIPS::RA)),
        "DispatchMessageW guest wndproc callout"
    );

    let writes = [
        uc.reg_write(RegisterMIPS::A0, u64::from(hwnd)),
        uc.reg_write(RegisterMIPS::A1, u64::from(msg)),
        uc.reg_write(RegisterMIPS::A2, u64::from(wparam)),
        uc.reg_write(RegisterMIPS::A3, u64::from(lparam)),
        uc.reg_write(RegisterMIPS::T9, u64::from(wndproc)),
        uc.reg_write(RegisterMIPS::PC, u64::from(wndproc)),
    ];
    writes.into_iter().all(|write| write.is_ok())
}

#[cfg(feature = "unicorn")]
fn capture_debug_snapshot<D>(
    uc: &unicorn_engine::Unicorn<'_, D>,
    traps: &ImportTrapTable,
    memory_fault: Option<UnicornMemoryFault>,
    function_pointer_probe: Option<UnicornFunctionPointerProbe>,
    indirect_call_probe: Option<UnicornIndirectCallProbe>,
    memory_write_probe: Option<UnicornMemoryWriteProbe>,
    interrupt_probe: Option<UnicornInterruptProbe>,
    invalid_instruction_probe: Option<UnicornInvalidInstructionProbe>,
    last_code: Vec<UnicornLastCode>,
    last_blocks: Vec<UnicornLastBlock>,
    blocked_get_message: Option<UnicornBlockedGetMessage>,
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
        indirect_call_probe,
        memory_write_probe,
        interrupt_probe,
        invalid_instruction_probe,
        last_code,
        last_blocks,
        blocked_get_message,
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
fn decode_indirect_call_register(instruction: u32) -> Option<u32> {
    let opcode = instruction >> 26;
    let function = instruction & 0x3f;
    if opcode != 0 || (function != 0x08 && function != 0x09) {
        return None;
    }
    Some((instruction >> 21) & 0x1f)
}

#[cfg(feature = "unicorn")]
fn decode_jr_register(instruction: u32) -> Option<u32> {
    let opcode = instruction >> 26;
    let function = instruction & 0x3f;
    if opcode != 0 || function != 0x08 {
        return None;
    }
    Some((instruction >> 21) & 0x1f)
}

#[cfg(feature = "unicorn")]
fn decode_direct_jump_target(pc: u32, instruction: u32) -> Option<u32> {
    let opcode = instruction >> 26;
    if opcode != 0x02 && opcode != 0x03 {
        return None;
    }
    let index = instruction & 0x03ff_ffff;
    Some((pc.wrapping_add(4) & 0xf000_0000) | (index << 2))
}

#[cfg(feature = "unicorn")]
fn read_mips_gpr<D>(uc: &unicorn_engine::Unicorn<'_, D>, register: u32) -> Option<u32> {
    use unicorn_engine::RegisterMIPS;

    match register {
        0 => Some(0),
        1 => Some(read_mips_reg(uc, RegisterMIPS::AT)),
        2 => Some(read_mips_reg(uc, RegisterMIPS::V0)),
        3 => Some(read_mips_reg(uc, RegisterMIPS::V1)),
        4 => Some(read_mips_reg(uc, RegisterMIPS::A0)),
        5 => Some(read_mips_reg(uc, RegisterMIPS::A1)),
        6 => Some(read_mips_reg(uc, RegisterMIPS::A2)),
        7 => Some(read_mips_reg(uc, RegisterMIPS::A3)),
        8 => Some(read_mips_reg(uc, RegisterMIPS::T0)),
        9 => Some(read_mips_reg(uc, RegisterMIPS::T1)),
        10 => Some(read_mips_reg(uc, RegisterMIPS::T2)),
        11 => Some(read_mips_reg(uc, RegisterMIPS::T3)),
        12 => Some(read_mips_reg(uc, RegisterMIPS::T4)),
        13 => Some(read_mips_reg(uc, RegisterMIPS::T5)),
        14 => Some(read_mips_reg(uc, RegisterMIPS::T6)),
        15 => Some(read_mips_reg(uc, RegisterMIPS::T7)),
        16 => Some(read_mips_reg(uc, RegisterMIPS::S0)),
        17 => Some(read_mips_reg(uc, RegisterMIPS::S1)),
        18 => Some(read_mips_reg(uc, RegisterMIPS::S2)),
        19 => Some(read_mips_reg(uc, RegisterMIPS::S3)),
        20 => Some(read_mips_reg(uc, RegisterMIPS::S4)),
        21 => Some(read_mips_reg(uc, RegisterMIPS::S5)),
        22 => Some(read_mips_reg(uc, RegisterMIPS::S6)),
        23 => Some(read_mips_reg(uc, RegisterMIPS::S7)),
        24 => Some(read_mips_reg(uc, RegisterMIPS::T8)),
        25 => Some(read_mips_reg(uc, RegisterMIPS::T9)),
        28 => Some(read_mips_reg(uc, RegisterMIPS::GP)),
        29 => Some(read_mips_reg(uc, RegisterMIPS::SP)),
        30 => Some(read_mips_reg(uc, RegisterMIPS::FP)),
        31 => Some(read_mips_reg(uc, RegisterMIPS::RA)),
        _ => None,
    }
}

#[cfg(feature = "unicorn")]
fn write_mips_gpr<D>(
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    register: u32,
    value: u32,
) -> Option<()> {
    use unicorn_engine::RegisterMIPS;

    let register = match register {
        0 => return Some(()),
        1 => RegisterMIPS::AT,
        2 => RegisterMIPS::V0,
        3 => RegisterMIPS::V1,
        4 => RegisterMIPS::A0,
        5 => RegisterMIPS::A1,
        6 => RegisterMIPS::A2,
        7 => RegisterMIPS::A3,
        8 => RegisterMIPS::T0,
        9 => RegisterMIPS::T1,
        10 => RegisterMIPS::T2,
        11 => RegisterMIPS::T3,
        12 => RegisterMIPS::T4,
        13 => RegisterMIPS::T5,
        14 => RegisterMIPS::T6,
        15 => RegisterMIPS::T7,
        16 => RegisterMIPS::S0,
        17 => RegisterMIPS::S1,
        18 => RegisterMIPS::S2,
        19 => RegisterMIPS::S3,
        20 => RegisterMIPS::S4,
        21 => RegisterMIPS::S5,
        22 => RegisterMIPS::S6,
        23 => RegisterMIPS::S7,
        24 => RegisterMIPS::T8,
        25 => RegisterMIPS::T9,
        28 => RegisterMIPS::GP,
        29 => RegisterMIPS::SP,
        30 => RegisterMIPS::FP,
        31 => RegisterMIPS::RA,
        _ => return None,
    };
    uc.reg_write(register, u64::from(value)).ok()
}

#[cfg(feature = "unicorn")]
fn mips_gpr_name(register: u32) -> &'static str {
    match register {
        0 => "zero",
        1 => "at",
        2 => "v0",
        3 => "v1",
        4 => "a0",
        5 => "a1",
        6 => "a2",
        7 => "a3",
        8 => "t0",
        9 => "t1",
        10 => "t2",
        11 => "t3",
        12 => "t4",
        13 => "t5",
        14 => "t6",
        15 => "t7",
        16 => "s0",
        17 => "s1",
        18 => "s2",
        19 => "s3",
        20 => "s4",
        21 => "s5",
        22 => "s6",
        23 => "s7",
        24 => "t8",
        25 => "t9",
        26 => "k0",
        27 => "k1",
        28 => "gp",
        29 => "sp",
        30 => "fp",
        31 => "ra",
        _ => "?",
    }
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

#[cfg(all(test, feature = "unicorn"))]
mod unicorn_tests {
    use unicorn_engine::{
        RegisterMIPS, Unicorn,
        unicorn_const::{Arch, Mode, Prot},
    };

    #[test]
    fn unicorn_executes_relocated_high_address_jal_with_delay_slot() {
        let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();
        uc.mem_map(0x6002_4000, 0x0000_7000, Prot::ALL).unwrap();
        write_u32(&mut uc, 0x6002_4218, 0x0c00_a6cc);
        write_u32(&mut uc, 0x6002_421c, 0x02c0_2025);
        write_u32(&mut uc, 0x6002_9b30, 0x27bd_ffe0);
        write_u32(&mut uc, 0x6002_9b34, 0x03e0_0008);
        write_u32(&mut uc, 0x6002_9b38, 0x0000_0000);
        uc.reg_write(RegisterMIPS::SP, 0x7ffd_fee8).unwrap();
        uc.reg_write(RegisterMIPS::S6, 0x6004_ed38).unwrap();

        uc.emu_start(0x6002_4218, 0, 0, 3).unwrap();

        assert_eq!(uc.reg_read(RegisterMIPS::PC).unwrap(), 0x6002_9b34);
        assert_eq!(uc.reg_read(RegisterMIPS::RA).unwrap(), 0x6002_4220);
        assert_eq!(uc.reg_read(RegisterMIPS::A0).unwrap(), 0x6004_ed38);
        assert_eq!(uc.reg_read(RegisterMIPS::SP).unwrap(), 0x7ffd_fec8);
    }

    #[test]
    fn unicorn_executes_relocated_high_address_jal_with_trace_hook() {
        let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();
        uc.mem_map(0x6002_4000, 0x0000_7000, Prot::ALL).unwrap();
        write_u32(&mut uc, 0x6002_4218, 0x0c00_a6cc);
        write_u32(&mut uc, 0x6002_421c, 0x02c0_2025);
        write_u32(&mut uc, 0x6002_9b30, 0x27bd_ffe0);
        write_u32(&mut uc, 0x6002_9b34, 0x03e0_0008);
        write_u32(&mut uc, 0x6002_9b38, 0x0000_0000);
        uc.reg_write(RegisterMIPS::SP, 0x7ffd_fee8).unwrap();
        uc.reg_write(RegisterMIPS::S6, 0x6004_ed38).unwrap();

        uc.add_code_hook(1, 0, |uc, address, _size| {
            let pc = address as u32;
            let instruction = super::read_unicorn_u32(uc, pc);
            let _next_instruction = super::read_unicorn_u32(uc, pc.wrapping_add(4));
            let direct_jump_target = instruction
                .and_then(|instruction| super::decode_direct_jump_target(pc, instruction));
            let _direct_jump_target_instruction =
                direct_jump_target.and_then(|target| super::read_unicorn_u32(uc, target));
            let _ra = super::read_mips_reg(uc, RegisterMIPS::RA);
            let _sp = super::read_mips_reg(uc, RegisterMIPS::SP);
        })
        .unwrap();

        uc.emu_start(0x6002_4218, 0, 0, 3).unwrap();

        assert_eq!(uc.reg_read(RegisterMIPS::PC).unwrap(), 0x6002_9b34);
        assert_eq!(uc.reg_read(RegisterMIPS::RA).unwrap(), 0x6002_4220);
        assert_eq!(uc.reg_read(RegisterMIPS::A0).unwrap(), 0x6004_ed38);
        assert_eq!(uc.reg_read(RegisterMIPS::SP).unwrap(), 0x7ffd_fec8);
    }

    #[test]
    fn unicorn_executes_jal_immediately_after_jr_return_target() {
        let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();
        uc.mem_map(0x6002_4000, 0x0000_7000, Prot::ALL).unwrap();
        write_u32(&mut uc, 0x6002_9b28, 0x03e0_0008);
        write_u32(&mut uc, 0x6002_9b2c, 0xac43_0010);
        write_u32(&mut uc, 0x6002_4218, 0x0c00_a6cc);
        write_u32(&mut uc, 0x6002_421c, 0x02c0_2025);
        write_u32(&mut uc, 0x6002_9b30, 0x27bd_ffe0);
        write_u32(&mut uc, 0x6002_9b34, 0x03e0_0008);
        write_u32(&mut uc, 0x6002_9b38, 0x0000_0000);
        uc.reg_write(RegisterMIPS::RA, 0x6002_4218).unwrap();
        uc.reg_write(RegisterMIPS::SP, 0x7ffd_fee8).unwrap();
        uc.reg_write(RegisterMIPS::S6, 0x6004_ed38).unwrap();
        uc.reg_write(RegisterMIPS::V0, 0x3000_22f0).unwrap();
        uc.reg_write(RegisterMIPS::V1, 1).unwrap();
        uc.mem_map(0x3000_2000, 0x1000, Prot::ALL).unwrap();

        uc.emu_start(0x6002_9b28, 0, 0, 5).unwrap();

        assert_eq!(uc.reg_read(RegisterMIPS::PC).unwrap(), 0x6002_9b34);
        assert_eq!(uc.reg_read(RegisterMIPS::RA).unwrap(), 0x6002_4220);
        assert_eq!(uc.reg_read(RegisterMIPS::A0).unwrap(), 0x6004_ed38);
        assert_eq!(uc.reg_read(RegisterMIPS::SP).unwrap(), 0x7ffd_fec8);
        let mut slot = [0; 4];
        uc.mem_read(0x3000_2300, &mut slot).unwrap();
        assert_eq!(u32::from_le_bytes(slot), 1);
    }

    #[test]
    fn unicorn_executes_jal_after_jr_return_target_with_trace_hook() {
        let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();
        uc.mem_map(0x6002_4000, 0x0000_7000, Prot::ALL).unwrap();
        uc.mem_map(0x3000_2000, 0x1000, Prot::ALL).unwrap();
        uc.mem_map(0x7ffd_f000, 0x1000, Prot::ALL).unwrap();
        write_u32(&mut uc, 0x6002_9b18, 0x8c43_0010);
        write_u32(&mut uc, 0x6002_9b1c, 0x8fbf_0010);
        write_u32(&mut uc, 0x6002_9b20, 0x2463_0001);
        write_u32(&mut uc, 0x6002_9b24, 0x27bd_0018);
        write_u32(&mut uc, 0x6002_9b28, 0x03e0_0008);
        write_u32(&mut uc, 0x6002_9b2c, 0xac43_0010);
        write_u32(&mut uc, 0x6002_4218, 0x0c00_a6cc);
        write_u32(&mut uc, 0x6002_421c, 0x02c0_2025);
        write_u32(&mut uc, 0x6002_9b30, 0x27bd_ffe0);
        write_u32(&mut uc, 0x6002_9b34, 0x03e0_0008);
        write_u32(&mut uc, 0x6002_9b38, 0x0000_0000);
        uc.reg_write(RegisterMIPS::SP, 0x7ffd_fed0).unwrap();
        uc.reg_write(RegisterMIPS::S6, 0x6004_ed38).unwrap();
        uc.reg_write(RegisterMIPS::V0, 0x3000_22f0).unwrap();
        write_u32(&mut uc, 0x3000_2300, 0);
        write_u32(&mut uc, 0x7ffd_fee0, 0x6002_4218);

        uc.add_code_hook(1, 0, |uc, address, _size| {
            let pc = address as u32;
            let instruction = super::read_unicorn_u32(uc, pc);
            let _next_instruction = super::read_unicorn_u32(uc, pc.wrapping_add(4));
            let direct_jump_target = instruction
                .and_then(|instruction| super::decode_direct_jump_target(pc, instruction));
            let _direct_jump_target_instruction =
                direct_jump_target.and_then(|target| super::read_unicorn_u32(uc, target));
            let _ra = super::read_mips_reg(uc, RegisterMIPS::RA);
            let _sp = super::read_mips_reg(uc, RegisterMIPS::SP);
        })
        .unwrap();

        uc.emu_start(0x6002_9b18, 0, 0, 9).unwrap();

        assert_eq!(uc.reg_read(RegisterMIPS::PC).unwrap(), 0x6002_9b34);
        assert_eq!(uc.reg_read(RegisterMIPS::RA).unwrap(), 0x6002_4220);
        assert_eq!(uc.reg_read(RegisterMIPS::A0).unwrap(), 0x6004_ed38);
        assert_eq!(uc.reg_read(RegisterMIPS::SP).unwrap(), 0x7ffd_fec8);
        let mut slot = [0; 4];
        uc.mem_read(0x3000_2300, &mut slot).unwrap();
        assert_eq!(u32::from_le_bytes(slot), 1);
    }

    #[test]
    fn unicorn_executes_mfc_return_site_and_nested_target_prologue() {
        let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();
        uc.mem_map(0x6002_4000, 0x0003_0000, Prot::ALL).unwrap();
        uc.mem_map(0x3000_2000, 0x1000, Prot::ALL).unwrap();
        uc.mem_map(0x7ffd_f000, 0x1000, Prot::ALL).unwrap();
        write_u32(&mut uc, 0x6002_9b18, 0x8c43_0010);
        write_u32(&mut uc, 0x6002_9b1c, 0x8fbf_0010);
        write_u32(&mut uc, 0x6002_9b20, 0x2463_0001);
        write_u32(&mut uc, 0x6002_9b24, 0x27bd_0018);
        write_u32(&mut uc, 0x6002_9b28, 0x03e0_0008);
        write_u32(&mut uc, 0x6002_9b2c, 0xac43_0010);
        write_u32(&mut uc, 0x6002_4218, 0x0c00_a6cc);
        write_u32(&mut uc, 0x6002_421c, 0x02c0_2025);
        write_u32(&mut uc, 0x6002_4220, 0x0802_5400);
        write_u32(&mut uc, 0x6002_4224, 0x0000_0000);
        write_u32(&mut uc, 0x6002_9b30, 0x27bd_ffe0);
        write_u32(&mut uc, 0x6002_9b34, 0xafbf_0018);
        write_u32(&mut uc, 0x6002_9b38, 0xafbe_0010);
        write_u32(&mut uc, 0x6002_9b3c, 0xafb7_0014);
        write_u32(&mut uc, 0x6002_9b40, 0x0c01_3b27);
        write_u32(&mut uc, 0x6002_9b44, 0x0080_b825);
        write_u32(&mut uc, 0x6004_ec9c, 0x03e0_0008);
        write_u32(&mut uc, 0x6004_eca0, 0x0000_0000);
        uc.reg_write(RegisterMIPS::SP, 0x7ffd_fed0).unwrap();
        uc.reg_write(RegisterMIPS::S6, 0x6004_ed38).unwrap();
        uc.reg_write(RegisterMIPS::V0, 0x3000_22f0).unwrap();
        write_u32(&mut uc, 0x3000_2300, 0);
        write_u32(&mut uc, 0x7ffd_fee0, 0x6002_4218);

        uc.emu_start(0x6002_9b18, 0, 0, 15).unwrap();

        assert_eq!(uc.reg_read(RegisterMIPS::RA).unwrap(), 0x6002_9b48);
        assert_eq!(uc.reg_read(RegisterMIPS::S7).unwrap(), 0x6004_ed38);
    }

    #[test]
    fn unicorn_executes_mfc_return_site_with_run_diagnostics() {
        let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();
        uc.mem_map(0x6002_4000, 0x0003_0000, Prot::ALL).unwrap();
        uc.mem_map(0x3000_2000, 0x1000, Prot::ALL).unwrap();
        uc.mem_map(0x7ffd_f000, 0x1000, Prot::ALL).unwrap();
        write_u32(&mut uc, 0x6002_9b18, 0x8c43_0010);
        write_u32(&mut uc, 0x6002_9b1c, 0x8fbf_0010);
        write_u32(&mut uc, 0x6002_9b20, 0x2463_0001);
        write_u32(&mut uc, 0x6002_9b24, 0x27bd_0018);
        write_u32(&mut uc, 0x6002_9b28, 0x03e0_0008);
        write_u32(&mut uc, 0x6002_9b2c, 0xac43_0010);
        write_u32(&mut uc, 0x6002_4218, 0x0c00_a6cc);
        write_u32(&mut uc, 0x6002_421c, 0x02c0_2025);
        write_u32(&mut uc, 0x6002_4220, 0x0802_5400);
        write_u32(&mut uc, 0x6002_4224, 0x0000_0000);
        write_u32(&mut uc, 0x6002_9b30, 0x27bd_ffe0);
        write_u32(&mut uc, 0x6002_9b34, 0xafbf_0018);
        write_u32(&mut uc, 0x6002_9b38, 0xafbe_0010);
        write_u32(&mut uc, 0x6002_9b3c, 0xafb7_0014);
        write_u32(&mut uc, 0x6002_9b40, 0x0c01_3b27);
        write_u32(&mut uc, 0x6002_9b44, 0x0080_b825);
        write_u32(&mut uc, 0x6004_ec9c, 0x03e0_0008);
        write_u32(&mut uc, 0x6004_eca0, 0x0000_0000);
        uc.reg_write(RegisterMIPS::SP, 0x7ffd_fed0).unwrap();
        uc.reg_write(RegisterMIPS::S6, 0x6004_ed38).unwrap();
        uc.reg_write(RegisterMIPS::V0, 0x3000_22f0).unwrap();
        write_u32(&mut uc, 0x3000_2300, 0);
        write_u32(&mut uc, 0x7ffd_fee0, 0x6002_4218);

        uc.add_code_hook(1, 0, |uc, address, _size| {
            let pc = address as u32;
            let instruction = super::read_unicorn_u32(uc, pc);
            let _next_instruction = super::read_unicorn_u32(uc, pc.wrapping_add(4));
            let direct_jump_target = instruction
                .and_then(|instruction| super::decode_direct_jump_target(pc, instruction));
            let _direct_jump_target_instruction =
                direct_jump_target.and_then(|target| super::read_unicorn_u32(uc, target));
            let _ra = super::read_mips_reg(uc, RegisterMIPS::RA);
            let _sp = super::read_mips_reg(uc, RegisterMIPS::SP);
        })
        .unwrap();
        uc.add_block_hook(1, 0, |uc, address, _size| {
            let _pc = address as u32;
            let _ra = super::read_mips_reg(uc, RegisterMIPS::RA);
            let _sp = super::read_mips_reg(uc, RegisterMIPS::SP);
        })
        .unwrap();
        uc.add_mem_hook(
            unicorn_engine::unicorn_const::HookType::MEM_UNMAPPED
                | unicorn_engine::unicorn_const::HookType::MEM_PROT,
            1,
            0,
            |_uc, _access, _address, _size, _value| false,
        )
        .unwrap();
        uc.add_intr_hook(|uc, _intno| {
            let _ = uc.emu_stop();
        })
        .unwrap();
        uc.add_insn_invalid_hook(|_uc| false).unwrap();

        uc.emu_start(0x6002_9b18, 0, 0, 15).unwrap();

        assert_eq!(uc.reg_read(RegisterMIPS::RA).unwrap(), 0x6002_9b48);
        assert_eq!(uc.reg_read(RegisterMIPS::S7).unwrap(), 0x6004_ed38);
    }

    #[test]
    fn unicorn_executes_direct_jump_immediately_after_jr_return_target() {
        let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();
        uc.mem_map(0x6002_4000, 0x0009_0000, Prot::ALL).unwrap();
        uc.mem_map(0x3000_2000, 0x1000, Prot::ALL).unwrap();
        write_u32(&mut uc, 0x6002_9b28, 0x03e0_0008);
        write_u32(&mut uc, 0x6002_9b2c, 0xac43_0010);
        write_u32(&mut uc, 0x6002_4218, 0x0802_6c1b);
        write_u32(&mut uc, 0x6002_421c, 0x0000_0000);
        write_u32(&mut uc, 0x6009_b06c, 0x3c1f_6002);
        write_u32(&mut uc, 0x6009_b070, 0x37ff_4220);
        write_u32(&mut uc, 0x6009_b074, 0x02c0_2025);
        write_u32(&mut uc, 0x6009_b078, 0x0800_a6cc);
        write_u32(&mut uc, 0x6009_b07c, 0x0000_0000);
        uc.reg_write(RegisterMIPS::RA, 0x6002_4218).unwrap();
        uc.reg_write(RegisterMIPS::SP, 0x7ffd_fee8).unwrap();
        uc.reg_write(RegisterMIPS::S6, 0x6004_ed38).unwrap();
        uc.reg_write(RegisterMIPS::V0, 0x3000_22f0).unwrap();
        uc.reg_write(RegisterMIPS::V1, 1).unwrap();

        uc.emu_start(0x6002_9b28, 0, 0, 7).unwrap();

        assert_eq!(uc.reg_read(RegisterMIPS::RA).unwrap(), 0x6002_4220);
        assert_eq!(uc.reg_read(RegisterMIPS::A0).unwrap(), 0x6004_ed38);
        let mut slot = [0; 4];
        uc.mem_read(0x3000_2300, &mut slot).unwrap();
        assert_eq!(u32::from_le_bytes(slot), 1);
    }

    #[test]
    fn unicorn_honors_pc_redirect_from_code_hook() {
        let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();
        uc.mem_map(0x6002_4000, 0x0009_0000, Prot::ALL).unwrap();
        write_u32(&mut uc, 0x6002_4218, 0x0802_6c1b);
        write_u32(&mut uc, 0x6002_421c, 0x0000_0000);
        write_u32(&mut uc, 0x6009_b06c, 0x3c1f_6002);
        write_u32(&mut uc, 0x6009_b070, 0x37ff_4220);

        uc.add_code_hook(0x6002_4218, 0x6002_4218, |uc, _address, _size| {
            uc.reg_write(RegisterMIPS::PC, 0x6009_b06c).unwrap();
        })
        .unwrap();
        uc.emu_start(0x6002_4218, 0, 0, 3).unwrap();

        assert_eq!(uc.reg_read(RegisterMIPS::RA).unwrap(), 0x6002_4220);
    }

    #[test]
    fn branch_likely_trampoline_annuls_delay_slot_when_condition_is_false() {
        let pc = 0x6002_4220;
        let stub_pc = 0x6002_a000;
        let branch = super::decode_mips_branch_likely(0x0683_0003).unwrap();
        let words = super::branch_likely_stub_words(pc, branch, 0x2442_0001, stub_pc).unwrap();

        let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();
        uc.mem_map(0x6002_4000, 0x0000_7000, Prot::ALL).unwrap();
        write_words(&mut uc, stub_pc, &words);
        uc.reg_write(RegisterMIPS::S4, u64::MAX).unwrap();
        uc.reg_write(RegisterMIPS::V0, 41).unwrap();

        uc.emu_start(u64::from(stub_pc), 0, 0, 4).unwrap();

        assert_eq!(uc.reg_read(RegisterMIPS::PC).unwrap(), u64::from(pc + 8));
        assert_eq!(uc.reg_read(RegisterMIPS::V0).unwrap(), 41);
    }

    #[test]
    fn branch_likely_trampoline_runs_delay_slot_when_condition_is_true() {
        let pc = 0x6002_4220;
        let stub_pc = 0x6002_a000;
        let branch = super::decode_mips_branch_likely(0x0683_0003).unwrap();
        let words = super::branch_likely_stub_words(pc, branch, 0x2442_0001, stub_pc).unwrap();

        let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();
        uc.mem_map(0x6002_4000, 0x0000_7000, Prot::ALL).unwrap();
        write_words(&mut uc, stub_pc, &words);
        uc.reg_write(RegisterMIPS::S4, 0).unwrap();
        uc.reg_write(RegisterMIPS::V0, 41).unwrap();

        uc.emu_start(u64::from(stub_pc), 0, 0, 5).unwrap();

        assert_eq!(uc.reg_read(RegisterMIPS::PC).unwrap(), u64::from(pc + 16));
        assert_eq!(uc.reg_read(RegisterMIPS::V0).unwrap(), 42);
    }

    #[test]
    fn normal_branch_trampoline_runs_delay_slot_on_both_paths() {
        let pc = 0x6002_9b64;
        let stub_pc = 0x6002_a000;
        let branch = super::decode_mips_normal_branch(0x12e0_0018).unwrap();
        let words = super::normal_branch_stub_words(pc, branch, 0x2442_0001, stub_pc).unwrap();

        let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();
        uc.mem_map(0x6002_4000, 0x0000_7000, Prot::ALL).unwrap();
        write_words(&mut uc, stub_pc, &words);
        uc.reg_write(RegisterMIPS::S7, 0).unwrap();
        uc.reg_write(RegisterMIPS::V0, 41).unwrap();
        uc.emu_start(u64::from(stub_pc), 0, 0, 5).unwrap();
        assert_eq!(uc.reg_read(RegisterMIPS::PC).unwrap(), 0x6002_9bc8);
        assert_eq!(uc.reg_read(RegisterMIPS::V0).unwrap(), 42);

        let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();
        uc.mem_map(0x6002_4000, 0x0000_7000, Prot::ALL).unwrap();
        write_words(&mut uc, stub_pc, &words);
        uc.reg_write(RegisterMIPS::S7, 1).unwrap();
        uc.reg_write(RegisterMIPS::V0, 41).unwrap();
        uc.emu_start(u64::from(stub_pc), 0, 0, 5).unwrap();
        assert_eq!(uc.reg_read(RegisterMIPS::PC).unwrap(), u64::from(pc + 8));
        assert_eq!(uc.reg_read(RegisterMIPS::V0).unwrap(), 42);
    }

    #[test]
    fn jal_trampoline_sets_link_and_runs_delay_slot() {
        let pc = 0x6002_4218;
        let stub_pc = 0x6002_a000;
        let target = 0x6002_9b30;
        let words = super::jal_stub_words(pc, target, 0x02c0_2025, stub_pc).unwrap();

        let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();
        uc.mem_map(0x6002_4000, 0x0000_7000, Prot::ALL).unwrap();
        write_words(&mut uc, stub_pc, &words);
        uc.reg_write(RegisterMIPS::S6, 0x6004_ed38).unwrap();

        uc.emu_start(u64::from(stub_pc), 0, 0, 5).unwrap();

        assert_eq!(uc.reg_read(RegisterMIPS::PC).unwrap(), u64::from(target));
        assert_eq!(uc.reg_read(RegisterMIPS::RA).unwrap(), u64::from(pc + 8));
        assert_eq!(uc.reg_read(RegisterMIPS::A0).unwrap(), 0x6004_ed38);
    }

    fn write_u32(uc: &mut Unicorn<'_, ()>, address: u64, value: u32) {
        uc.mem_write(address, &value.to_le_bytes()).unwrap();
    }

    fn write_words(uc: &mut Unicorn<'_, ()>, address: u32, words: &[u32]) {
        for (index, word) in words.iter().enumerate() {
            write_u32(uc, u64::from(address) + index as u64 * 4, *word);
        }
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
