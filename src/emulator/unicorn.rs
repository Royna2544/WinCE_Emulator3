use crate::{
    ce::{
        coredll::CoredllGuestMemory,
        framebuffer::{Framebuffer, VirtualFramebuffer},
        kernel::CeKernel,
    },
    emulator::{
        imports::{
            DYNAMIC_COREDLL_PROC_TRAP_BASE, ExternalImportTable, IMPORT_TRAP_BASE,
            IMPORT_TRAP_PAGE_SIZE, ImportTrapTable, import_trap_code_page,
            patch_pe_coredll_imports, patch_pe_imports,
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
    entry_image_base: Option<u32>,
    stack_top: Option<u32>,
    mapped_blobs: Vec<MappedBlob>,
    import_traps: ImportTrapTable,
    resource_strings: Vec<MappedResourceString>,
    resources: Vec<MappedResource>,
    #[cfg(feature = "unicorn")]
    trampoline_ranges: Vec<(u32, u32)>,
    #[cfg(feature = "unicorn")]
    trampoline_jumps: Vec<MipsTrampolineJump>,
    last_debug: Option<UnicornDebugSnapshot>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UnicornRunLimits {
    pub instruction_limit: usize,
    pub wall_clock_limit_ms: u64,
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
    pub indirect_call_probe: Option<UnicornIndirectCallProbe>,
    pub host_wall_clock_stop: Option<UnicornHostWallClockStop>,
    pub interrupt_probe: Option<UnicornInterruptProbe>,
    pub invalid_instruction_probe: Option<UnicornInvalidInstructionProbe>,
    pub last_calls: Vec<UnicornLastCall>,
    pub last_imports: Vec<UnicornLastImport>,
    pub last_messages: Vec<UnicornLastMessage>,
    pub last_wndproc_returns: Vec<UnicornWndProcReturn>,
    pub last_code: Vec<UnicornLastCode>,
    pub last_blocks: Vec<UnicornLastBlock>,
    pub import_counts: Vec<UnicornImportCount>,
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
pub struct UnicornHostWallClockStop {
    pub pc: u32,
    pub ra: u32,
    pub sp: u32,
    pub instruction: Option<u32>,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornInterruptProbe {
    pub pc: u32,
    pub ra: u32,
    pub sp: u32,
    pub intno: u32,
    pub last_code_pc: Option<u32>,
    pub last_code_instruction: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornInvalidInstructionProbe {
    pub pc: u32,
    pub ra: u32,
    pub sp: u32,
    pub instruction: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornLastCall {
    pub pc: u32,
    pub target: u32,
    pub kind: &'static str,
    pub ra: u32,
    pub sp: u32,
    pub a0: u32,
    pub a1: u32,
    pub a2: u32,
    pub a3: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornLastImport {
    pub pc: u32,
    pub module: String,
    pub kind: crate::emulator::imports::ImportModuleKind,
    pub ordinal: Option<u32>,
    pub name: Option<String>,
    pub a0: u32,
    pub a1: u32,
    pub a2: u32,
    pub a3: u32,
    pub sp: u32,
    pub result: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornImportCount {
    pub module: String,
    pub ordinal: Option<u32>,
    pub name: Option<String>,
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct UnicornImportCountKey {
    module: String,
    ordinal: Option<u32>,
    name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornLastMessage {
    pub ordinal: u32,
    pub msg_ptr: u32,
    pub filter_hwnd: Option<u32>,
    pub min_msg: u32,
    pub max_msg: u32,
    pub flags: Option<u32>,
    pub result: Option<u32>,
    pub message: Option<UnicornMessageRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornMessageRecord {
    pub hwnd: u32,
    pub msg: u32,
    pub wparam: u32,
    pub lparam: u32,
    pub time_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornWndProcReturn {
    pub source: &'static str,
    pub hwnd: u32,
    pub msg: u32,
    pub wparam: u32,
    pub lparam: u32,
    pub wndproc: u32,
    pub return_pc: u32,
    pub result: u32,
    pub class_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornLastCode {
    pub pc: u32,
    pub ra: u32,
    pub sp: u32,
    pub sp_return_slot: Option<u32>,
    pub instruction: Option<u32>,
    pub next_instruction: Option<u32>,
    pub direct_jump_target: Option<u32>,
    pub direct_jump_target_instruction: Option<u32>,
    pub direct_jump_target_in_trampoline: bool,
    pub direct_jump_trampoline_origin: Option<u32>,
    pub current_trampoline_origin: Option<u32>,
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
const GUEST_STACK_MIN_RESERVE: u32 = 0x0010_0000;
const GUEST_HEAP_ARENA_BASE: u32 = 0x3000_0000;
const GUEST_HEAP_ARENA_SIZE: u32 = 0x0100_0000;
#[cfg(feature = "unicorn")]
const UNICORN_TRACE_LIMIT: usize = 256;
#[cfg(feature = "unicorn")]
const UNICORN_CODE_TRACE_SAMPLE_INTERVAL: u32 = 64;
#[cfg(feature = "unicorn")]
const IMPORT_TRAP_ARG_COUNT: usize = 12;
#[cfg(feature = "unicorn")]
const THREAD_EXIT_STUB_ADDR: u32 =
    IMPORT_TRAP_BASE + IMPORT_TRAP_PAGE_SIZE - crate::emulator::imports::IMPORT_TRAP_STRIDE;
#[cfg(feature = "unicorn")]
const CREATE_WINDOW_RETURN_STUB_ADDR: u32 =
    GUEST_THREAD_RETURN_STUB_ADDR - crate::emulator::imports::IMPORT_TRAP_STRIDE;
#[cfg(feature = "unicorn")]
const GUEST_THREAD_RETURN_STUB_ADDR: u32 =
    THREAD_EXIT_STUB_ADDR - crate::emulator::imports::IMPORT_TRAP_STRIDE;
#[cfg(feature = "unicorn")]
const WNDPROC_RETURN_STUB_ADDR: u32 =
    CREATE_WINDOW_RETURN_STUB_ADDR - crate::emulator::imports::IMPORT_TRAP_STRIDE;
const RESERVED_IMPORT_TRAP_STUB_BYTES: u32 = crate::emulator::imports::IMPORT_TRAP_STRIDE * 4;
#[cfg(feature = "unicorn")]
const CREATESTRUCTW_SIZE: u32 = 48;
#[cfg(feature = "unicorn")]
const WM_INITDIALOG: u32 = 0x0110;
#[cfg(feature = "unicorn")]
const IMAGE_SCN_MEM_EXECUTE: u32 = 0x2000_0000;
#[cfg(feature = "unicorn")]
const MIPS_NOP: u32 = 0x0000_0000;

#[cfg(feature = "unicorn")]
#[derive(Debug, Clone, PartialEq, Eq)]
struct CreateWindowReturn {
    return_pc: u32,
    hwnd: u32,
    wndproc: u32,
    lparam: u32,
    class_name: Option<String>,
}

#[cfg(feature = "unicorn")]
#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingWndProcReturn {
    source: &'static str,
    hwnd: u32,
    msg: u32,
    wparam: u32,
    lparam: u32,
    wndproc: u32,
    return_pc: u32,
    class_name: Option<String>,
    api_result: Option<u32>,
    dialog_result_hwnd: Option<u32>,
    finalize_destroy: bool,
    send_thread_id: Option<u32>,
    send_timeout_result_ptr: Option<u32>,
}

#[cfg(feature = "unicorn")]
#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingGuestThreadReturn {
    creator_thread_id: u32,
    worker_thread_id: u32,
    thread_handle: u32,
    return_pc: u32,
    creator_regs: [u32; 32],
}

#[cfg(feature = "unicorn")]
#[derive(Debug, Clone, PartialEq, Eq)]
struct SuspendedGuestThread {
    thread_id: u32,
    regs: [u32; 32],
    pc: u32,
}

#[cfg(feature = "unicorn")]
#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockedGuestThread {
    thread_id: u32,
    thread_handle: u32,
    regs: [u32; 32],
    return_pc: u32,
    msg_ptr: u32,
    hwnd: Option<u32>,
    min_msg: u32,
    max_msg: u32,
}

#[cfg(feature = "unicorn")]
#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockedWaitThread {
    thread_id: u32,
    thread_handle: u32,
    wait_handle: u32,
    regs: [u32; 32],
    return_pc: u32,
}

#[cfg(feature = "unicorn")]
type GuestThreadStackSlots = std::rc::Rc<std::cell::RefCell<std::collections::BTreeMap<u32, u32>>>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct MappedResourceString {
    module: u32,
    id: u32,
    text: String,
    data_ptr: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MappedResource {
    module: u32,
    name: u32,
    name_string: Option<String>,
    kind: u32,
    data_ptr: u32,
    size: u32,
}

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
            entry_image_base: None,
            stack_top: None,
            mapped_blobs: Vec::new(),
            import_traps: ImportTrapTable::new(),
            resource_strings: Vec::new(),
            resources: Vec::new(),
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
        let stack_size = align_up_4k(
            image
                .optional_header
                .size_of_stack_reserve
                .max(GUEST_STACK_MIN_RESERVE),
        )?;
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
        self.entry_image_base = Some(image.image_base());
        self.register_image_resource_strings(image, image.image_base())?;
        self.import_traps.merge(traps);
        self.mapped_blobs.push(MappedBlob {
            base: image.image_base(),
            bytes: mapped,
        });
        for (path, load_base, mapped) in loaded_dlls {
            if let Some(dll) = dlls.iter().find(|dll| dll.path == path) {
                self.register_image_resource_strings(dll, load_base)?;
            }
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

    fn register_image_resource_strings(&mut self, image: &PeImage, load_base: u32) -> Result<()> {
        for string in image.resource_strings()? {
            self.resource_strings.push(MappedResourceString {
                module: load_base,
                id: string.id,
                text: string.text,
                data_ptr: Some(load_base.wrapping_add(string.data_rva)),
            });
        }
        for resource in image.resource_data_entries()? {
            self.resources.push(MappedResource {
                module: load_base,
                name: resource.name,
                name_string: resource.name_string,
                kind: resource.kind,
                data_ptr: load_base.wrapping_add(resource.data_rva),
                size: resource.size,
            });
        }
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

    pub fn run_until_import_trap(&mut self, kernel: &mut CeKernel) -> Result<()> {
        let mut framebuffer = VirtualFramebuffer::default_primary()?;
        self.run_until_import_trap_with_framebuffer(kernel, &mut framebuffer)
    }

    pub fn run_until_import_trap_with_framebuffer(
        &mut self,
        kernel: &mut CeKernel,
        framebuffer: &mut dyn Framebuffer,
    ) -> Result<()> {
        self.run_until_import_trap_with_framebuffer_limit(kernel, framebuffer, 0)
    }

    pub fn run_until_import_trap_with_framebuffer_limit(
        &mut self,
        kernel: &mut CeKernel,
        framebuffer: &mut dyn Framebuffer,
        instruction_limit: usize,
    ) -> Result<()> {
        self.run_until_import_trap_with_framebuffer_limits(
            kernel,
            framebuffer,
            UnicornRunLimits {
                instruction_limit,
                wall_clock_limit_ms: 0,
            },
        )
    }

    pub fn run_until_import_trap_with_framebuffer_limits(
        &mut self,
        kernel: &mut CeKernel,
        framebuffer: &mut dyn Framebuffer,
        limits: UnicornRunLimits,
    ) -> Result<()> {
        let info = framebuffer.info();
        kernel.remote.set_framebuffer_size(info.width, info.height);
        for string in &self.resource_strings {
            kernel.resources.register_string(
                string.module,
                string.id,
                string.text.clone(),
                string.data_ptr,
            );
        }
        for resource in &self.resources {
            kernel.resources.register(
                resource.module,
                resource
                    .name_string
                    .as_ref()
                    .map(|name| crate::ce::resource::ResourceId::Name(name.clone()))
                    .unwrap_or(crate::ce::resource::ResourceId::Integer(
                        resource.name as u16,
                    )),
                crate::ce::resource::ResourceId::Integer(resource.kind as u16),
                resource.data_ptr,
                resource.size,
            );
        }
        #[cfg(not(feature = "unicorn"))]
        let _ = limits;
        #[cfg(feature = "unicorn")]
        {
            return self.run_with_unicorn(kernel, framebuffer, limits);
        }

        #[cfg(not(feature = "unicorn"))]
        Err(Error::Backend(
            "built without the `unicorn` feature; core state is ready but CPU execution is disabled"
                .to_owned(),
        ))
    }

    #[cfg(feature = "unicorn")]
    fn write_process_entry_context<D>(
        &self,
        kernel: &CeKernel,
        uc: &mut unicorn_engine::Unicorn<'_, D>,
    ) -> Result<()> {
        use unicorn_engine::RegisterMIPS;

        const STACK_COMMAND_LINE_OFFSET: u32 = 0x800;
        const SW_SHOWNORMAL: u32 = 1;

        let Some(hinstance) = self.entry_image_base else {
            return Ok(());
        };
        let Some(stack_top) = self.stack_top else {
            return Err(Error::Backend(
                "PE entry context needs a mapped guest stack".to_owned(),
            ));
        };
        let command_line = stack_top
            .checked_sub(STACK_COMMAND_LINE_OFFSET)
            .ok_or_else(|| Error::Backend("guest command-line pointer underflow".to_owned()))?;

        // CE/MFC WinMain receives the application command line in A2.
        let mut command_line_bytes = Vec::new();
        for unit in kernel.process_command_line().encode_utf16() {
            command_line_bytes.extend_from_slice(&unit.to_le_bytes());
        }
        command_line_bytes.extend_from_slice(&0u16.to_le_bytes());
        uc.mem_write(u64::from(command_line), &command_line_bytes)
            .map_err(|err| Error::Backend(format!("write guest command line: {err:?}")))?;
        for (register, value, name) in [
            (RegisterMIPS::A0, hinstance, "A0/hInstance"),
            (RegisterMIPS::A1, 0, "A1/hPrevInstance"),
            (RegisterMIPS::A2, command_line, "A2/lpCmdLine"),
            (RegisterMIPS::A3, SW_SHOWNORMAL, "A3/nCmdShow"),
        ] {
            uc.reg_write(register, u64::from(value))
                .map_err(|err| Error::Backend(format!("set guest {name}: {err:?}")))?;
        }

        Ok(())
    }

    #[cfg(feature = "unicorn")]
    fn run_with_unicorn(
        &mut self,
        kernel: &mut CeKernel,
        framebuffer: &mut dyn Framebuffer,
        limits: UnicornRunLimits,
    ) -> Result<()> {
        use std::{
            cell::{Cell, RefCell},
            collections::BTreeMap,
            rc::Rc,
            time::{Duration, Instant},
        };
        use unicorn_engine::{
            RegisterMIPS, Unicorn,
            unicorn_const::{Arch, HookType, Mode},
        };

        let framebuffer_info = framebuffer.info();
        tracing::debug!(
            target: "ce.framebuffer",
            width = framebuffer_info.width,
            height = framebuffer_info.height,
            stride = framebuffer_info.stride,
            format = ?framebuffer_info.format,
            dirty_rects = framebuffer.dirty_rects().len(),
            "virtual framebuffer attached"
        );

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
        self.write_process_entry_context(kernel, &mut uc)?;

        let indirect_call_probe = Rc::new(RefCell::new(None));
        let indirect_call_probe_hook = Rc::clone(&indirect_call_probe);
        let last_code = Rc::new(RefCell::new(Vec::<UnicornLastCode>::new()));
        let last_code_hook = Rc::clone(&last_code);
        let last_code_probe = Rc::new(RefCell::new(None));
        let last_code_probe_hook = Rc::clone(&last_code_probe);
        let code_trace_counter = Rc::new(Cell::new(0u32));
        let code_trace_counter_hook = Rc::clone(&code_trace_counter);
        let host_wall_clock_stop = Rc::new(RefCell::new(None));
        let host_wall_clock_stop_hook = Rc::clone(&host_wall_clock_stop);
        let host_wall_clock_limit = (limits.wall_clock_limit_ms != 0)
            .then(|| Duration::from_millis(limits.wall_clock_limit_ms));
        let host_wall_clock_started = Instant::now();
        let host_wall_clock_counter = Rc::new(RefCell::new(0u32));
        let host_wall_clock_counter_hook = Rc::clone(&host_wall_clock_counter);
        let last_calls = Rc::new(RefCell::new(Vec::<UnicornLastCall>::new()));
        let last_calls_hook = Rc::clone(&last_calls);
        let trampoline_ranges = self.trampoline_ranges.clone();
        let trampoline_jumps = self.trampoline_jumps.clone();
        uc.add_code_hook(1, 0, move |uc, address, _size| {
            let pc = address as u32;
            let code_trace_index = code_trace_counter_hook.get().wrapping_add(1);
            code_trace_counter_hook.set(code_trace_index);
            let instruction = read_unicorn_u32(uc, pc);
            *last_code_probe_hook.borrow_mut() = Some((pc, instruction));
            let next_instruction = read_unicorn_u32(uc, pc.wrapping_add(4));
            if let Some(limit) = host_wall_clock_limit {
                let mut counter = host_wall_clock_counter_hook.borrow_mut();
                *counter = counter.wrapping_add(1);
                if *counter & 0x0fff == 0 && host_wall_clock_started.elapsed() >= limit {
                    *host_wall_clock_stop_hook.borrow_mut() = Some(UnicornHostWallClockStop {
                        pc,
                        ra: read_mips_reg(uc, RegisterMIPS::RA),
                        sp: read_mips_reg(uc, RegisterMIPS::SP),
                        instruction,
                        elapsed_ms: host_wall_clock_started.elapsed().as_millis() as u64,
                    });
                    let _ = uc.emu_stop();
                    return;
                }
            }
            let direct_jump_target =
                instruction.and_then(|instruction| decode_direct_jump_target(pc, instruction));
            let direct_jump_target_instruction =
                direct_jump_target.and_then(|target| read_unicorn_u32(uc, target));
            if let (Some(instruction), Some(target)) = (instruction, direct_jump_target) {
                if instruction >> 26 == 0x03 {
                    push_unicorn_last_call(&last_calls_hook, uc, pc, target, "jal");
                }
            }
            if let Some((register, target)) = instruction
                .and_then(decode_jalr_register)
                .and_then(|register| read_mips_gpr(uc, register).map(|target| (register, target)))
            {
                let kind = mips_gpr_name(register);
                push_unicorn_last_call(&last_calls_hook, uc, pc, target, kind);
            }
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
                if let Some(trampoline) = trampoline_jumps
                    .iter()
                    .find(|trampoline| trampoline.origin == target)
                {
                    let _ = write_mips_gpr(uc, register, trampoline.stub);
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
            let direct_jump_trampoline_origin = direct_jump_target.and_then(|target| {
                trampoline_jumps
                    .iter()
                    .find_map(|trampoline| (trampoline.stub == target).then_some(trampoline.origin))
            });
            let current_trampoline_origin = trampoline_jumps.iter().find_map(|trampoline| {
                trampoline
                    .stub
                    .checked_add(trampoline.byte_len)
                    .filter(|end| pc >= trampoline.stub && pc < *end)
                    .map(|_| trampoline.origin)
            });
            let should_trace_code = code_trace_index % UNICORN_CODE_TRACE_SAMPLE_INTERVAL == 0
                || direct_jump_target_in_trampoline
                || direct_jump_trampoline_origin.is_some()
                || current_trampoline_origin.is_some();
            if should_trace_code {
                let ra = read_mips_reg(uc, RegisterMIPS::RA);
                let sp = read_mips_reg(uc, RegisterMIPS::SP);
                let mut last_code = last_code_hook.borrow_mut();
                if last_code.len() == UNICORN_TRACE_LIMIT {
                    last_code.remove(0);
                }
                last_code.push(UnicornLastCode {
                    pc,
                    ra,
                    sp,
                    sp_return_slot: sp
                        .checked_add(0x10)
                        .and_then(|addr| read_unicorn_u32(uc, addr)),
                    instruction,
                    next_instruction,
                    direct_jump_target,
                    direct_jump_target_instruction,
                    direct_jump_target_in_trampoline,
                    direct_jump_trampoline_origin,
                    current_trampoline_origin,
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
            if last_blocks.len() == UNICORN_TRACE_LIMIT {
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

        let blocked_get_message = Rc::new(RefCell::new(None));
        let blocked_get_message_hook = Rc::clone(&blocked_get_message);
        let last_imports = Rc::new(RefCell::new(Vec::<UnicornLastImport>::new()));
        let last_imports_hook = Rc::clone(&last_imports);
        let import_counts = Rc::new(RefCell::new(BTreeMap::<UnicornImportCountKey, u64>::new()));
        let import_counts_hook = Rc::clone(&import_counts);
        let last_messages = Rc::new(RefCell::new(Vec::<UnicornLastMessage>::new()));
        let last_messages_hook = Rc::clone(&last_messages);
        let last_wndproc_returns = Rc::new(RefCell::new(Vec::<UnicornWndProcReturn>::new()));
        let last_wndproc_returns_hook = Rc::clone(&last_wndproc_returns);
        let pending_wndproc_returns = Rc::new(RefCell::new(Vec::<PendingWndProcReturn>::new()));
        let pending_wndproc_returns_hook = Rc::clone(&pending_wndproc_returns);
        let create_window_returns = Rc::new(RefCell::new(Vec::<CreateWindowReturn>::new()));
        let create_window_returns_hook = Rc::clone(&create_window_returns);
        let current_thread_id = Rc::new(RefCell::new(1u32));
        let current_thread_id_hook = Rc::clone(&current_thread_id);
        let pending_guest_thread_returns =
            Rc::new(RefCell::new(Vec::<PendingGuestThreadReturn>::new()));
        let pending_guest_thread_returns_hook = Rc::clone(&pending_guest_thread_returns);
        let blocked_guest_thread = Rc::new(RefCell::new(None::<BlockedGuestThread>));
        let blocked_guest_thread_hook = Rc::clone(&blocked_guest_thread);
        let blocked_wait_threads = Rc::new(RefCell::new(Vec::<BlockedWaitThread>::new()));
        let blocked_wait_threads_hook = Rc::clone(&blocked_wait_threads);
        let suspended_guest_thread = Rc::new(RefCell::new(None::<SuspendedGuestThread>));
        let suspended_guest_thread_hook = Rc::clone(&suspended_guest_thread);
        let running_guest_thread = Rc::new(RefCell::new(None::<(u32, u32)>));
        let running_guest_thread_hook = Rc::clone(&running_guest_thread);
        let guest_thread_stack_slots = Rc::new(RefCell::new(std::collections::BTreeMap::new()));
        let guest_thread_stack_slots_hook = Rc::clone(&guest_thread_stack_slots);
        let traps = self.import_traps.clone();
        let kernel_ptr = kernel as *mut CeKernel;
        let framebuffer_ptr = framebuffer as *mut dyn Framebuffer;
        let stack_top = self.stack_top.unwrap_or(0);
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
                if address == CREATE_WINDOW_RETURN_STUB_ADDR {
                    let Some(callout) = create_window_returns_hook.borrow_mut().pop() else {
                        let _ = uc.emu_stop();
                        return;
                    };
                    record_wndproc_return(
                        &last_wndproc_returns_hook,
                        UnicornWndProcReturn {
                            source: "CreateWindowExW/WM_CREATE",
                            hwnd: callout.hwnd,
                            msg: crate::ce::gwe::WM_CREATE,
                            wparam: 0,
                            lparam: callout.lparam,
                            wndproc: callout.wndproc,
                            return_pc: callout.return_pc,
                            result: read_mips_reg(uc, RegisterMIPS::V0),
                            class_name: callout.class_name,
                        },
                    );
                    let writes = [
                        uc.reg_write(RegisterMIPS::V0, u64::from(callout.hwnd)),
                        uc.reg_write(RegisterMIPS::PC, u64::from(callout.return_pc)),
                        uc.reg_write(RegisterMIPS::RA, u64::from(callout.return_pc)),
                    ];
                    if writes.into_iter().any(|write| write.is_err()) {
                        let _ = uc.emu_stop();
                    }
                    return;
                }
                if address == WNDPROC_RETURN_STUB_ADDR {
                    let Some(callout) = pending_wndproc_returns_hook.borrow_mut().pop() else {
                        let _ = uc.emu_stop();
                        return;
                    };
                    let result = read_mips_reg(uc, RegisterMIPS::V0);
                    if let Some(thread_id) = callout.send_thread_id {
                        unsafe { &mut *kernel_ptr }.gwe.end_send_message(thread_id);
                    }
                    if let Some(result_ptr) = callout.send_timeout_result_ptr {
                        let _ = uc.mem_write(u64::from(result_ptr), &result.to_le_bytes());
                    }
                    if callout.msg == crate::ce::gwe::WM_PAINT {
                        unsafe { &mut *kernel_ptr }
                            .gwe
                            .validate_window(callout.hwnd);
                    }
                    if callout.finalize_destroy {
                        let time_ms = unsafe { &*kernel_ptr }.timers.tick_count();
                        unsafe { &mut *kernel_ptr }
                            .gwe
                            .destroy_window(callout.hwnd, time_ms);
                    }
                    record_wndproc_return(
                        &last_wndproc_returns_hook,
                        UnicornWndProcReturn {
                            source: callout.source,
                            hwnd: callout.hwnd,
                            msg: callout.msg,
                            wparam: callout.wparam,
                            lparam: callout.lparam,
                            wndproc: callout.wndproc,
                            return_pc: callout.return_pc,
                            result,
                            class_name: callout.class_name,
                        },
                    );
                    let api_result = callout.api_result.or_else(|| {
                        callout
                            .dialog_result_hwnd
                            .and_then(|hwnd| unsafe { &*kernel_ptr }.gwe.dialog_result(hwnd))
                    });
                    if let Some(api_result) = api_result {
                        let _ = uc.reg_write(RegisterMIPS::V0, u64::from(api_result));
                    }
                    let writes = [
                        uc.reg_write(RegisterMIPS::PC, u64::from(callout.return_pc)),
                        uc.reg_write(RegisterMIPS::RA, u64::from(callout.return_pc)),
                    ];
                    if writes.into_iter().any(|write| write.is_err()) {
                        let _ = uc.emu_stop();
                    }
                    return;
                }
                if address == GUEST_THREAD_RETURN_STUB_ADDR {
                    let exit_code = read_mips_reg(uc, RegisterMIPS::V0);
                    if let Some(callout) = pending_guest_thread_returns_hook.borrow_mut().pop() {
                        unsafe { &mut *kernel_ptr }
                            .mark_guest_thread_exited(callout.thread_handle, exit_code);
                        release_guest_thread_stack_slot(
                            &guest_thread_stack_slots_hook,
                            callout.worker_thread_id,
                        );
                        *current_thread_id_hook.borrow_mut() = callout.creator_thread_id;
                        *running_guest_thread_hook.borrow_mut() = None;
                        restore_mips_gprs(uc, &callout.creator_regs);
                        let writes = [
                            uc.reg_write(RegisterMIPS::V0, u64::from(callout.thread_handle)),
                            uc.reg_write(RegisterMIPS::PC, u64::from(callout.return_pc)),
                            uc.reg_write(RegisterMIPS::RA, u64::from(callout.return_pc)),
                        ];
                        tracing::debug!(
                            target: "ce.imports",
                            thread_id = callout.worker_thread_id,
                            handle = format_args!("0x{:08x}", callout.thread_handle),
                            exit_code = format_args!("0x{exit_code:08x}"),
                            return_pc = format_args!("0x{:08x}", callout.return_pc),
                            "guest thread returned"
                        );
                        if writes.into_iter().any(|write| write.is_err()) {
                            let _ = uc.emu_stop();
                        }
                        return;
                    }
                    let Some((worker_thread_id, thread_handle)) =
                        running_guest_thread_hook.borrow_mut().take()
                    else {
                        let _ = uc.emu_stop();
                        return;
                    };
                    unsafe { &mut *kernel_ptr }.mark_guest_thread_exited(thread_handle, exit_code);
                    release_guest_thread_stack_slot(
                        &guest_thread_stack_slots_hook,
                        worker_thread_id,
                    );
                    let Some(suspended) = suspended_guest_thread_hook.borrow_mut().take() else {
                        let _ = uc.emu_stop();
                        return;
                    };
                    *current_thread_id_hook.borrow_mut() = suspended.thread_id;
                    restore_suspended_thread(uc, &suspended);
                    tracing::debug!(
                        target: "ce.imports",
                        thread_id = worker_thread_id,
                        handle = format_args!("0x{thread_handle:08x}"),
                        exit_code = format_args!("0x{exit_code:08x}"),
                        resume_thread_id = suspended.thread_id,
                        "resumed creator after guest thread returned"
                    );
                    return;
                }
                let trap = traps
                    .trap_at(address)
                    .cloned()
                    .or_else(|| crate::emulator::imports::dynamic_coredll_proc_trap(address));
                if trap.is_none() {
                    return;
                }
                if let Some(trap) = trap.as_ref() {
                    let a0 = read_mips_reg(uc, RegisterMIPS::A0);
                    let a1 = read_mips_reg(uc, RegisterMIPS::A1);
                    let a2 = read_mips_reg(uc, RegisterMIPS::A2);
                    let a3 = read_mips_reg(uc, RegisterMIPS::A3);
                    let sp = read_mips_reg(uc, RegisterMIPS::SP);
                    *import_counts_hook
                        .borrow_mut()
                        .entry(UnicornImportCountKey {
                            module: trap.module_name.clone(),
                            ordinal: trap.ordinal,
                            name: trap.name.clone(),
                        })
                        .or_insert(0) += 1;
                    {
                        let mut imports = last_imports_hook.borrow_mut();
                        if imports.len() == UNICORN_TRACE_LIMIT {
                            imports.remove(0);
                        }
                        imports.push(UnicornLastImport {
                            pc: address,
                            module: trap.module_name.clone(),
                            kind: trap.module_kind,
                            ordinal: trap.ordinal,
                            name: trap.name.clone(),
                            a0,
                            a1,
                            a2,
                            a3,
                            sp,
                            result: None,
                        });
                    }
                    tracing::debug!(
                        target: "ce.imports",
                        pc = format_args!("0x{address:08x}"),
                        module = trap.module_name.as_str(),
                        kind = ?trap.module_kind,
                        ordinal = trap.ordinal,
                        name = trap.name.as_deref().unwrap_or("<ordinal>"),
                        a0 = format_args!("0x{a0:08x}"),
                        a1 = format_args!("0x{a1:08x}"),
                        a2 = format_args!("0x{a2:08x}"),
                        a3 = format_args!("0x{a3:08x}"),
                        sp = format_args!("0x{sp:08x}"),
                        "import trap"
                    );
                }
                let args = read_mips_import_args(uc);
                let active_thread_id = *current_thread_id_hook.borrow();
                if trap.as_ref().is_some_and(|trap| {
                    try_block_empty_get_message(
                        unsafe { &mut *kernel_ptr },
                        uc,
                        trap.module_kind,
                        trap.ordinal,
                        &args,
                        active_thread_id,
                        &blocked_get_message_hook,
                        &blocked_guest_thread_hook,
                        &pending_guest_thread_returns_hook,
                        &current_thread_id_hook,
                        &suspended_guest_thread_hook,
                        &running_guest_thread_hook,
                        &last_messages_hook,
                    )
                }) {
                    return;
                }
                if trap.as_ref().is_some_and(|trap| {
                    try_block_wait_for_single_object(
                        unsafe { &mut *kernel_ptr },
                        uc,
                        trap.module_kind,
                        trap.ordinal,
                        &args,
                        active_thread_id,
                        &blocked_wait_threads_hook,
                        &pending_guest_thread_returns_hook,
                        &current_thread_id_hook,
                        &suspended_guest_thread_hook,
                        &running_guest_thread_hook,
                    )
                }) {
                    return;
                }
                if trap.as_ref().is_some_and(|trap| {
                    try_exit_guest_thread_callout(
                        unsafe { &mut *kernel_ptr },
                        uc,
                        trap.module_kind,
                        trap.ordinal,
                        &args,
                        &pending_guest_thread_returns_hook,
                        &current_thread_id_hook,
                        &suspended_guest_thread_hook,
                        &running_guest_thread_hook,
                        &guest_thread_stack_slots_hook,
                    )
                }) {
                    return;
                }
                let mut memory = UnicornGuestMemory { uc };
                if let Some(setjmp_result) = trap.as_ref().and_then(|trap| {
                    try_handle_setjmp_longjmp(
                        &mut memory,
                        trap.module_kind,
                        trap.ordinal,
                        trap.name.as_deref(),
                        &args,
                    )
                }) {
                    if let Some(trap) = trap.as_ref() {
                        if let Some(import) = last_imports_hook
                            .borrow_mut()
                            .iter_mut()
                            .rev()
                            .find(|import| import.pc == address && import.result.is_none())
                        {
                            import.result = Some(setjmp_result.result);
                        }
                        tracing::debug!(
                            target: "ce.imports",
                            pc = format_args!("0x{address:08x}"),
                            module = trap.module_name.as_str(),
                            kind = ?trap.module_kind,
                            ordinal = trap.ordinal,
                            name = trap.name.as_deref().unwrap_or("<ordinal>"),
                            result = format_args!("0x{:08x}", setjmp_result.result),
                            "import trap return"
                        );
                    }
                    if !setjmp_result.jumped {
                        let _ = memory
                            .uc
                            .reg_write(RegisterMIPS::V0, u64::from(setjmp_result.result));
                    }
                    return;
                }
                if trap.as_ref().is_some_and(|trap| {
                    try_enter_dispatch_message_callout(
                        unsafe { &*kernel_ptr },
                        memory.uc,
                        trap.module_kind,
                        trap.ordinal,
                        &args,
                        &pending_wndproc_returns_hook,
                    )
                }) {
                    return;
                }
                if trap.as_ref().is_some_and(|trap| {
                    try_enter_send_message_callout(
                        unsafe { &mut *kernel_ptr },
                        memory.uc,
                        trap.module_kind,
                        trap.ordinal,
                        &args,
                        &pending_wndproc_returns_hook,
                    )
                }) {
                    return;
                }
                if trap.as_ref().is_some_and(|trap| {
                    try_enter_def_window_proc_callout(
                        unsafe { &mut *kernel_ptr },
                        memory.uc,
                        trap.module_kind,
                        trap.ordinal,
                        &args,
                        &pending_wndproc_returns_hook,
                    )
                }) {
                    return;
                }
                if trap.as_ref().is_some_and(|trap| {
                    try_enter_def_dlg_proc_callout(
                        unsafe { &*kernel_ptr },
                        memory.uc,
                        trap.module_kind,
                        trap.ordinal,
                        &args,
                        &pending_wndproc_returns_hook,
                    )
                }) {
                    return;
                }
                if trap.as_ref().is_some_and(|trap| {
                    try_enter_destroy_window_callout(
                        unsafe { &mut *kernel_ptr },
                        memory.uc,
                        trap.module_kind,
                        trap.ordinal,
                        &args,
                        &pending_wndproc_returns_hook,
                    )
                }) {
                    return;
                }
                if trap.as_ref().is_some_and(|trap| {
                    try_enter_update_window_callout(
                        unsafe { &*kernel_ptr },
                        memory.uc,
                        trap.module_kind,
                        trap.ordinal,
                        &args,
                        &pending_wndproc_returns_hook,
                    )
                }) {
                    return;
                }
                if trap.as_ref().is_some_and(|trap| {
                    try_enter_is_dialog_message_callout(
                        unsafe { &*kernel_ptr },
                        memory.uc,
                        trap.module_kind,
                        trap.ordinal,
                        &args,
                        &pending_wndproc_returns_hook,
                    )
                }) {
                    return;
                }
                if trap.as_ref().is_some_and(|trap| {
                    try_enter_call_window_proc_callout(
                        unsafe { &mut *kernel_ptr },
                        memory.uc,
                        trap.module_kind,
                        trap.ordinal,
                        &args,
                        &pending_wndproc_returns_hook,
                    )
                }) {
                    return;
                }
                if trap.as_ref().is_some_and(|trap| {
                    try_enter_create_thread_callout(
                        unsafe { &mut *kernel_ptr },
                        memory.uc,
                        trap.module_kind,
                        trap.ordinal,
                        &args,
                        active_thread_id,
                        stack_top,
                        &current_thread_id_hook,
                        &running_guest_thread_hook,
                        &guest_thread_stack_slots_hook,
                        &pending_guest_thread_returns_hook,
                    )
                }) {
                    return;
                }
                let Some(import_return) = traps.dispatch_trap_registers_with_framebuffer(
                    unsafe { &mut *kernel_ptr },
                    &mut memory,
                    Some(unsafe { &mut *framebuffer_ptr }),
                    active_thread_id,
                    address,
                    args.clone(),
                ) else {
                    let _ = memory.uc.emu_stop();
                    return;
                };
                let result = import_return.v0;
                let _ = map_kernel_memory_allocations(
                    memory.uc,
                    unsafe { &*kernel_ptr },
                    &mut mapped_kernel_memory_hook.borrow_mut(),
                );
                if trap.as_ref().is_some_and(|trap| {
                    trap.module_kind == crate::emulator::imports::ImportModuleKind::Coredll
                        && trap.ordinal == Some(crate::ce::coredll_ordinals::ORD_CREATE_PROCESS_W)
                        && result != 0
                }) {
                    let _ = run_pending_process_launches(
                        memory.uc,
                        unsafe { &mut *kernel_ptr },
                        limits.instruction_limit,
                    );
                    let _ = sync_file_mapping_views_to_unicorn(memory.uc, unsafe { &*kernel_ptr });
                }
                if let Some(trap) = trap.as_ref() {
                    if let Some(import) = last_imports_hook
                        .borrow_mut()
                        .iter_mut()
                        .rev()
                        .find(|import| import.pc == address && import.result.is_none())
                    {
                        import.result = Some(result);
                    }
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
                    record_message_import(
                        memory.uc,
                        trap.module_kind,
                        trap.ordinal,
                        &args,
                        Some(result),
                        &last_messages_hook,
                    );
                }
                if trap.as_ref().is_some_and(|trap| {
                    try_enter_create_window_create_callout(
                        unsafe { &mut *kernel_ptr },
                        memory.uc,
                        trap.module_kind,
                        trap.ordinal,
                        &args,
                        result,
                        &mapped_kernel_memory_hook,
                        &create_window_returns_hook,
                    )
                }) {
                    return;
                }
                if trap.as_ref().is_some_and(|trap| {
                    try_enter_dialog_init_callout(
                        unsafe { &*kernel_ptr },
                        memory.uc,
                        trap.module_kind,
                        trap.ordinal,
                        &args,
                        result,
                        &pending_wndproc_returns_hook,
                    )
                }) {
                    return;
                }
                if trap.as_ref().is_some_and(|trap| {
                    try_enter_resumed_thread_callout(
                        unsafe { &*kernel_ptr },
                        memory.uc,
                        trap.module_kind,
                        trap.ordinal,
                        &args,
                        result,
                        active_thread_id,
                        stack_top,
                        &current_thread_id_hook,
                        &suspended_guest_thread_hook,
                        &running_guest_thread_hook,
                        &guest_thread_stack_slots_hook,
                    )
                }) {
                    return;
                }
                let _ = memory.uc.reg_write(RegisterMIPS::V0, u64::from(result));
                if let Some(v1) = import_return.v1 {
                    let _ = memory.uc.reg_write(RegisterMIPS::V1, u64::from(v1));
                }
                if try_resume_blocked_wait(
                    unsafe { &mut *kernel_ptr },
                    memory.uc,
                    active_thread_id,
                    &current_thread_id_hook,
                    &blocked_wait_threads_hook,
                    &suspended_guest_thread_hook,
                    &running_guest_thread_hook,
                ) {
                    return;
                }
                if try_resume_blocked_get_message(
                    unsafe { &mut *kernel_ptr },
                    memory.uc,
                    active_thread_id,
                    &current_thread_id_hook,
                    &blocked_guest_thread_hook,
                    &suspended_guest_thread_hook,
                    &running_guest_thread_hook,
                ) {
                    return;
                }
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
        let interrupt_last_code_probe = Rc::clone(&last_code_probe);
        uc.add_intr_hook(move |uc, intno| {
            let last_code = *interrupt_last_code_probe.borrow();
            *interrupt_probe_hook.borrow_mut() = Some(UnicornInterruptProbe {
                pc: read_mips_reg(uc, RegisterMIPS::PC),
                ra: read_mips_reg(uc, RegisterMIPS::RA),
                sp: read_mips_reg(uc, RegisterMIPS::SP),
                intno: intno as u32,
                last_code_pc: last_code.map(|(pc, _)| pc),
                last_code_instruction: last_code.and_then(|(_, instruction)| instruction),
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
        let result = uc.emu_start(u64::from(entry), 0, 0, limits.instruction_limit);
        self.last_debug = Some(capture_debug_snapshot(
            &uc,
            &self.import_traps,
            memory_fault.borrow().clone(),
            indirect_call_probe.borrow().clone(),
            host_wall_clock_stop.borrow().clone(),
            interrupt_probe.borrow().clone(),
            invalid_instruction_probe.borrow().clone(),
            last_calls.borrow().clone(),
            last_imports.borrow().clone(),
            last_messages.borrow().clone(),
            last_wndproc_returns.borrow().clone(),
            last_code.borrow().clone(),
            last_blocks.borrow().clone(),
            import_count_snapshot(&import_counts.borrow()),
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
                let _ = sync_file_mapping_views_from_unicorn(&mut uc, kernel);
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
        let _ = sync_file_mapping_views_from_unicorn(&mut uc, kernel);
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
        let halfword_jump_table_ranges =
            mips_halfword_jump_table_ranges(mapped, load_base, start, end, &image.path)?;
        let mut rva = start;
        while rva.checked_add(8).is_some_and(|next| next <= end) {
            if mips_patch_rva_overlaps_data_ranges(rva, &halfword_jump_table_ranges) {
                rva = rva
                    .checked_add(4)
                    .ok_or_else(|| Error::InvalidArgument("section scan overflow".to_owned()))?;
                continue;
            }
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
                if delay_slot == MIPS_NOP && pc.wrapping_add(branch.target) == pc.wrapping_add(8) {
                    write_mapped_word(mapped, rva, MIPS_NOP, &image.path)?;
                } else if delay_slot != MIPS_NOP || is_unconditional_taken_branch(branch) {
                    patches.push(MipsUnicornPatch {
                        rva,
                        pc,
                        kind: MipsUnicornPatchKind::Branch { branch, delay_slot },
                    });
                }
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
        let stub_offset = stub_rva as usize;
        let stub_end = stub_offset
            .checked_add(stub_words.len() * 4)
            .ok_or_else(|| Error::InvalidArgument("branch-likely stub overflow".to_owned()))?;
        trampoline_jumps.push(MipsTrampolineJump {
            origin: patch.pc,
            stub: stub_pc,
            byte_len: (stub_words.len() * 4) as u32,
        });
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
    jumps: Vec<MipsTrampolineJump>,
}

#[cfg(feature = "unicorn")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct MipsTrampolineJump {
    origin: u32,
    stub: u32,
    byte_len: u32,
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
fn is_unconditional_taken_branch(branch: MipsBranchLikely) -> bool {
    branch.rs == 0 && branch.rt == 0 && branch.inverse_branch == MipsBranch::Bne
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
fn mips_halfword_jump_table_ranges(
    mapped: &[u8],
    load_base: u32,
    start: u32,
    end: u32,
    path: &str,
) -> Result<Vec<(u32, u32)>> {
    let mut ranges = Vec::new();
    let mut rva = start;
    while rva.checked_add(32).is_some_and(|next| next <= end) {
        let Some(range) =
            decode_mips_halfword_jump_table_range(mapped, load_base, start, end, rva, path)?
        else {
            rva = rva
                .checked_add(4)
                .ok_or_else(|| Error::InvalidArgument("section scan overflow".to_owned()))?;
            continue;
        };
        ranges.push(range);
        rva = rva
            .checked_add(4)
            .ok_or_else(|| Error::InvalidArgument("section scan overflow".to_owned()))?;
    }
    Ok(ranges)
}

#[cfg(feature = "unicorn")]
fn decode_mips_halfword_jump_table_range(
    mapped: &[u8],
    load_base: u32,
    section_start: u32,
    section_end: u32,
    rva: u32,
    path: &str,
) -> Result<Option<(u32, u32)>> {
    let lui = read_mapped_word(mapped, rva, path)?;
    let addiu = read_mapped_word(mapped, rva + 4, path)?;
    let sll = read_mapped_word(mapped, rva + 8, path)?;
    let addu_index = read_mapped_word(mapped, rva + 12, path)?;
    let lh = read_mapped_word(mapped, rva + 16, path)?;
    let addu_target = read_mapped_word(mapped, rva + 20, path)?;
    let jr = read_mapped_word(mapped, rva + 24, path)?;
    let delay_slot = read_mapped_word(mapped, rva + 28, path)?;

    let Some(base_register) = decode_mips_lui_rt(lui) else {
        return Ok(None);
    };
    if !is_mips_addiu_same_register(addiu, base_register) {
        return Ok(None);
    }
    let Some((index_register, selector_register)) = decode_mips_sll_by_one(sll) else {
        return Ok(None);
    };
    if !is_mips_addu(addu_index, index_register, index_register, base_register) {
        return Ok(None);
    }
    if !is_mips_lh_same_register(lh, index_register) {
        return Ok(None);
    }
    if !is_mips_addu(addu_target, base_register, base_register, index_register) {
        return Ok(None);
    }
    if !is_mips_jr(jr, base_register) || delay_slot != MIPS_NOP {
        return Ok(None);
    }

    let table_pc = ((lui & 0xffff) << 16).wrapping_add(addiu as u16 as i16 as i32 as u32);
    let Some(table_rva) = table_pc.checked_sub(load_base) else {
        return Ok(None);
    };
    if table_rva != rva + 32 || table_rva >= section_end {
        return Ok(None);
    }
    let Some(entry_count) = find_mips_halfword_jump_table_entry_count(
        mapped,
        section_start,
        rva,
        selector_register,
        path,
    )?
    else {
        return Ok(None);
    };
    let byte_len = entry_count.saturating_mul(2);
    if byte_len == 0 {
        return Ok(None);
    }
    let Some(table_end) = table_rva.checked_add(byte_len) else {
        return Ok(None);
    };
    if table_end > section_end {
        return Ok(None);
    }
    Ok(Some((table_rva, byte_len)))
}

#[cfg(feature = "unicorn")]
fn find_mips_halfword_jump_table_entry_count(
    mapped: &[u8],
    section_start: u32,
    setup_rva: u32,
    selector_register: u32,
    path: &str,
) -> Result<Option<u32>> {
    let search_start = setup_rva.saturating_sub(64).max(section_start);
    let mut cursor = setup_rva;
    while cursor >= search_start + 4 {
        cursor -= 4;
        let instruction = read_mapped_word(mapped, cursor, path)?;
        if instruction >> 26 == 0x0b
            && ((instruction >> 21) & 0x1f) == selector_register
            && (instruction & 0xffff) != 0
        {
            return Ok(Some(instruction & 0xffff));
        }
    }
    Ok(None)
}

#[cfg(feature = "unicorn")]
fn mips_patch_rva_overlaps_data_ranges(rva: u32, ranges: &[(u32, u32)]) -> bool {
    ranges.iter().any(|(start, len)| {
        let end = start.saturating_add(*len);
        rva < end && rva.saturating_add(8) > *start
    })
}

#[cfg(feature = "unicorn")]
fn decode_mips_lui_rt(instruction: u32) -> Option<u32> {
    (instruction >> 26 == 0x0f).then_some((instruction >> 16) & 0x1f)
}

#[cfg(feature = "unicorn")]
fn is_mips_addiu_same_register(instruction: u32, register: u32) -> bool {
    instruction >> 26 == 0x09
        && ((instruction >> 21) & 0x1f) == register
        && ((instruction >> 16) & 0x1f) == register
}

#[cfg(feature = "unicorn")]
fn decode_mips_sll_by_one(instruction: u32) -> Option<(u32, u32)> {
    let opcode = instruction >> 26;
    let rs = (instruction >> 21) & 0x1f;
    let rt = (instruction >> 16) & 0x1f;
    let rd = (instruction >> 11) & 0x1f;
    let shamt = (instruction >> 6) & 0x1f;
    let funct = instruction & 0x3f;
    (opcode == 0 && rs == 0 && shamt == 1 && funct == 0).then_some((rd, rt))
}

#[cfg(feature = "unicorn")]
fn is_mips_addu(instruction: u32, rd: u32, rs: u32, rt: u32) -> bool {
    instruction >> 26 == 0
        && ((instruction >> 21) & 0x1f) == rs
        && ((instruction >> 16) & 0x1f) == rt
        && ((instruction >> 11) & 0x1f) == rd
        && (instruction & 0x3f) == 0x21
}

#[cfg(feature = "unicorn")]
fn is_mips_lh_same_register(instruction: u32, register: u32) -> bool {
    instruction >> 26 == 0x21
        && ((instruction >> 21) & 0x1f) == register
        && ((instruction >> 16) & 0x1f) == register
        && (instruction & 0xffff) == 0
}

#[cfg(feature = "unicorn")]
fn is_mips_jr(instruction: u32, register: u32) -> bool {
    instruction >> 26 == 0
        && ((instruction >> 21) & 0x1f) == register
        && (instruction & 0x001f_ffff) == 0x08
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
    if next >= DYNAMIC_COREDLL_PROC_TRAP_BASE.saturating_sub(RESERVED_IMPORT_TRAP_STUB_BYTES) {
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
            MemoryPerms::READ | MemoryPerms::WRITE,
            "heap allocation",
        )?;
    }
    for allocation in kernel.memory.virtual_allocations() {
        let newly_mapped = map_guest_range(
            uc,
            mapped,
            allocation.base,
            allocation.size,
            virtual_allocation_perms(allocation.protect),
            "virtual allocation",
        )?;
        if newly_mapped && !allocation.initial_bytes.is_empty() {
            uc.mem_write(u64::from(allocation.base), &allocation.initial_bytes)
                .map_err(|err| {
                    Error::Backend(format!(
                        "seed virtual allocation 0x{:08x}: {err:?}",
                        allocation.base
                    ))
                })?;
        }
    }
    Ok(())
}

#[cfg(feature = "unicorn")]
fn sync_file_mapping_views_from_unicorn<D>(
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    kernel: &mut CeKernel,
) -> Result<()> {
    let mut updates = Vec::new();
    for mapping in kernel.handles.file_mappings() {
        let Some(base) = mapping.view_base else {
            continue;
        };
        let size = mapping.view_size.min(mapping.size) as usize;
        let mut bytes = vec![0; size];
        uc.mem_read(u64::from(base), &mut bytes).map_err(|err| {
            Error::Backend(format!(
                "read mapped view 0x{base:08x} before process launch: {err:?}"
            ))
        })?;
        updates.push((base, mapping.view_offset as usize, bytes));
    }
    for (base, offset, bytes) in updates {
        if let Some(mapping) = kernel.handles.file_mapping_by_view_mut(base) {
            let end = offset.saturating_add(bytes.len());
            if end > mapping.data.len() {
                mapping.data.resize(end, 0);
            }
            mapping.data[offset..end].copy_from_slice(&bytes);
        }
        kernel.memory.set_virtual_initial_bytes(base, bytes);
    }
    Ok(())
}

#[cfg(feature = "unicorn")]
fn sync_file_mapping_views_to_unicorn<D>(
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    kernel: &CeKernel,
) -> Result<()> {
    for mapping in kernel.handles.file_mappings() {
        let Some(base) = mapping.view_base else {
            continue;
        };
        let start = mapping.view_offset as usize;
        let end = start
            .saturating_add(mapping.view_size as usize)
            .min(mapping.data.len());
        if start >= end {
            continue;
        }
        uc.mem_write(u64::from(base), &mapping.data[start..end])
            .map_err(|err| Error::Backend(format!("write mapped view 0x{base:08x}: {err:?}")))?;
    }
    Ok(())
}

#[cfg(feature = "unicorn")]
fn run_pending_process_launches<D>(
    parent_uc: &mut unicorn_engine::Unicorn<'_, D>,
    kernel: &mut CeKernel,
    instruction_limit: usize,
) -> Result<()> {
    let launches = kernel.take_pending_process_launches();
    if launches.is_empty() {
        return Ok(());
    }
    sync_file_mapping_views_from_unicorn(parent_uc, kernel)?;
    for launch in launches {
        let Some(path) = resolve_process_launch_path(kernel, &launch)? else {
            kernel.mark_process_launch_exited(&launch, u32::MAX);
            continue;
        };
        let image = PeImage::inspect(&path)?;
        let saved_base = kernel.process_module_base();
        let saved_path = kernel.process_module_path().to_owned();
        let saved_host_path = kernel.process_module_host_path().cloned();
        let saved_command_line = kernel.process_command_line().to_owned();

        kernel.set_process_module_base(image.image_base());
        let child_module_path = kernel
            .host_path_to_guest_mount(&path)
            .unwrap_or_else(|| path.to_string_lossy().replace('/', "\\"));
        kernel.set_process_module_path(child_module_path);
        kernel.set_process_module_host_path(path.clone());
        kernel.set_process_command_line(launch.command_line.clone().unwrap_or_default());

        let mut child = UnicornMips::new()?;
        child.load_pe_image(&image)?;
        let mut child_framebuffer = VirtualFramebuffer::default_primary()?;
        child.run_until_import_trap_with_framebuffer_limit(
            kernel,
            &mut child_framebuffer,
            instruction_limit,
        )?;
        let exit_code = child
            .last_debug_snapshot()
            .and_then(|snapshot| {
                snapshot
                    .encoded_kernel_exit
                    .as_ref()
                    .map(|exit| exit.exit_code)
            })
            .unwrap_or(0);
        kernel.mark_process_launch_exited(&launch, exit_code);

        kernel.set_process_module_base(saved_base);
        kernel.set_process_module_path(saved_path);
        if let Some(saved_host_path) = saved_host_path {
            kernel.set_process_module_host_path(saved_host_path);
        }
        kernel.set_process_command_line(saved_command_line);
    }
    Ok(())
}

#[cfg(feature = "unicorn")]
fn resolve_process_launch_path(
    kernel: &CeKernel,
    launch: &crate::ce::kernel::PendingProcessLaunch,
) -> Result<Option<std::path::PathBuf>> {
    let raw_path = if let Some(application) = launch
        .application
        .as_deref()
        .filter(|path| !path.is_empty())
    {
        application.to_owned()
    } else if let Some(token) = first_command_line_token(launch.command_line.as_deref()) {
        token
    } else {
        return Ok(None);
    };
    let separator = std::path::MAIN_SEPARATOR.to_string();
    let relative = raw_path.replace('\\', &separator);
    let relative_path = std::path::Path::new(&relative);
    let Some(parent_exe) = kernel.process_module_host_path() else {
        return Ok(None);
    };
    let Some(parent_dir) = parent_exe.parent() else {
        return Ok(None);
    };
    let direct = parent_dir.join(relative_path);
    if direct.exists() {
        return Ok(Some(direct));
    }
    let Some(file_name) = relative_path.file_name() else {
        return Ok(None);
    };
    let Some(search_root) = parent_dir.parent() else {
        return Ok(None);
    };
    for entry in std::fs::read_dir(search_root).map_err(|source| Error::Io {
        path: search_root.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| Error::Io {
            path: search_root.to_path_buf(),
            source,
        })?;
        if !entry.file_type().map(|ty| ty.is_dir()).unwrap_or(false) {
            continue;
        }
        let candidate = entry.path().join(file_name);
        if candidate.exists() {
            return Ok(Some(candidate));
        }
    }
    Ok(None)
}

#[cfg(feature = "unicorn")]
fn first_command_line_token(command_line: Option<&str>) -> Option<String> {
    let command_line = command_line?.trim();
    if command_line.is_empty() {
        return None;
    }
    if let Some(rest) = command_line.strip_prefix('"') {
        return rest.find('"').map(|end| rest[..end].to_owned());
    }
    command_line
        .split_whitespace()
        .next()
        .map(|token| token.to_owned())
}

#[cfg(feature = "unicorn")]
fn map_guest_range<D>(
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    mapped: &mut Vec<(u32, u32)>,
    base: u32,
    size: u32,
    perms: MemoryPerms,
    label: &str,
) -> Result<bool> {
    let first_page = base & !0xfff;
    let page_end = base
        .checked_add(size.max(1))
        .and_then(|end| end.checked_add(0xfff))
        .map(|end| end & !0xfff)
        .ok_or_else(|| Error::InvalidArgument(format!("{label} range overflow")))?;
    let mut page_base = first_page;
    let mut mapped_any = false;
    while page_base < page_end {
        if mapped.iter().any(|(mapped_base, mapped_size)| {
            page_base >= *mapped_base && page_base < mapped_base.saturating_add(*mapped_size)
        }) {
            page_base = page_base
                .checked_add(0x1000)
                .ok_or_else(|| Error::InvalidArgument(format!("{label} page overflow")))?;
            continue;
        }
        uc.mem_map(u64::from(page_base), 0x1000, unicorn_perms(perms))
            .map_err(|err| {
                Error::Backend(format!("map {label} page 0x{page_base:08x}: {err:?}"))
            })?;
        mapped.push((page_base, 0x1000));
        mapped_any = true;
        page_base = page_base
            .checked_add(0x1000)
            .ok_or_else(|| Error::InvalidArgument(format!("{label} page overflow")))?;
    }
    Ok(mapped_any)
}

#[cfg(feature = "unicorn")]
fn virtual_allocation_perms(protect: u32) -> MemoryPerms {
    const PAGE_READONLY: u32 = 0x02;
    const PAGE_READWRITE: u32 = 0x04;
    const PAGE_WRITECOPY: u32 = 0x08;
    const PAGE_EXECUTE: u32 = 0x10;
    const PAGE_EXECUTE_READ: u32 = 0x20;
    const PAGE_EXECUTE_READWRITE: u32 = 0x40;
    const PAGE_EXECUTE_WRITECOPY: u32 = 0x80;

    match protect & 0xff {
        PAGE_READONLY => MemoryPerms::READ,
        PAGE_READWRITE | PAGE_WRITECOPY => MemoryPerms::READ | MemoryPerms::WRITE,
        PAGE_EXECUTE => MemoryPerms::EXEC,
        PAGE_EXECUTE_READ => MemoryPerms::READ | MemoryPerms::EXEC,
        PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY => {
            MemoryPerms::READ | MemoryPerms::WRITE | MemoryPerms::EXEC
        }
        _ => MemoryPerms::READ | MemoryPerms::WRITE,
    }
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
        if let Some(stop) = self.host_wall_clock_stop.as_ref() {
            write!(
                f,
                " host_wall_clock_stop_ms={} host_stop_pc=0x{:08x} host_stop_ra=0x{:08x} host_stop_sp=0x{:08x}",
                stop.elapsed_ms, stop.pc, stop.ra, stop.sp
            )?;
            if let Some(instruction) = stop.instruction {
                write!(f, " host_stop_insn=0x{instruction:08x}")?;
            }
        }
        if !self.import_counts.is_empty() {
            write!(f, " import_counts=[")?;
            for (index, count) in self.import_counts.iter().enumerate() {
                if index != 0 {
                    write!(f, ",")?;
                }
                write!(f, "{}/count={}", count.module, count.count)?;
                if let Some(ordinal) = count.ordinal {
                    write!(f, "/ord={ordinal}")?;
                }
                if let Some(name) = count.name.as_deref() {
                    write!(f, "/name={name}")?;
                }
            }
            write!(f, "]")?;
        }
        if let Some(probe) = self.interrupt_probe.as_ref() {
            write!(
                f,
                " interrupt_pc=0x{:08x} interrupt_ra=0x{:08x} interrupt_sp=0x{:08x} interrupt_no={}",
                probe.pc, probe.ra, probe.sp, probe.intno
            )?;
            if let Some(last_pc) = probe.last_code_pc {
                write!(f, " interrupt_last_pc=0x{last_pc:08x}")?;
            }
            if let Some(instruction) = probe.last_code_instruction {
                write!(f, " interrupt_last_insn=0x{instruction:08x}")?;
            }
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
        if !self.last_imports.is_empty() {
            write!(f, " last_imports=[")?;
            for (index, import) in self.last_imports.iter().enumerate() {
                if index != 0 {
                    write!(f, ",")?;
                }
                write!(f, "0x{:08x}/{:?}/{}", import.pc, import.kind, import.module)?;
                if let Some(ordinal) = import.ordinal {
                    write!(f, "/ord={ordinal}")?;
                }
                if let Some(name) = import.name.as_deref() {
                    write!(f, "/name={name}")?;
                }
                write!(
                    f,
                    "/a0=0x{:08x}/a1=0x{:08x}/a2=0x{:08x}/a3=0x{:08x}/sp=0x{:08x}",
                    import.a0, import.a1, import.a2, import.a3, import.sp
                )?;
                if let Some(result) = import.result {
                    write!(f, "/ret=0x{result:08x}")?;
                } else {
                    write!(f, "/ret=<pending>")?;
                }
            }
            write!(f, "]")?;
        }
        if !self.last_calls.is_empty() {
            write!(f, " last_calls=[")?;
            for (index, call) in self.last_calls.iter().enumerate() {
                if index != 0 {
                    write!(f, ",")?;
                }
                write!(
                    f,
                    "0x{:08x}->0x{:08x}/{}/ra=0x{:08x}/sp=0x{:08x}/a0=0x{:08x}/a1=0x{:08x}/a2=0x{:08x}/a3=0x{:08x}",
                    call.pc,
                    call.target,
                    call.kind,
                    call.ra,
                    call.sp,
                    call.a0,
                    call.a1,
                    call.a2,
                    call.a3
                )?;
            }
            write!(f, "]")?;
        }
        if !self.last_messages.is_empty() {
            write!(f, " last_messages=[")?;
            for (index, message) in self.last_messages.iter().enumerate() {
                if index != 0 {
                    write!(f, ",")?;
                }
                let name = if message.ordinal == crate::ce::coredll_ordinals::ORD_GET_MESSAGE_W {
                    "GetMessageW"
                } else {
                    "PeekMessageW"
                };
                write!(
                    f,
                    "{name}/msg_ptr=0x{:08x}/filter={}/range=0x{:08x}..0x{:08x}",
                    message.msg_ptr,
                    message
                        .filter_hwnd
                        .map(|hwnd| format!("0x{hwnd:08x}"))
                        .unwrap_or_else(|| "<any>".to_owned()),
                    message.min_msg,
                    message.max_msg
                )?;
                if let Some(flags) = message.flags {
                    write!(f, "/flags=0x{flags:08x}")?;
                }
                if let Some(result) = message.result {
                    write!(f, "/ret=0x{result:08x}")?;
                } else {
                    write!(f, "/ret=<blocked>")?;
                }
                if let Some(record) = message.message.as_ref() {
                    write!(
                        f,
                        "/msg_hwnd=0x{:08x}/msg=0x{:08x}/w=0x{:08x}/l=0x{:08x}/time={}",
                        record.hwnd, record.msg, record.wparam, record.lparam, record.time_ms
                    )?;
                }
            }
            write!(f, "]")?;
        }
        if !self.last_wndproc_returns.is_empty() {
            write!(f, " last_wndproc_returns=[")?;
            for (index, record) in self.last_wndproc_returns.iter().enumerate() {
                if index != 0 {
                    write!(f, ",")?;
                }
                write!(
                    f,
                    "{}/hwnd=0x{:08x}/msg=0x{:08x}/w=0x{:08x}/l=0x{:08x}/wndproc=0x{:08x}/return_pc=0x{:08x}/ret=0x{:08x}",
                    record.source,
                    record.hwnd,
                    record.msg,
                    record.wparam,
                    record.lparam,
                    record.wndproc,
                    record.return_pc,
                    record.result
                )?;
                if let Some(class_name) = record.class_name.as_deref() {
                    write!(f, "/class={class_name}")?;
                }
            }
            write!(f, "]")?;
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
                if let Some(sp_return_slot) = code.sp_return_slot {
                    write!(f, "/sp10=0x{sp_return_slot:08x}")?;
                }
                if let Some(instruction) = code.instruction {
                    write!(f, "/insn=0x{instruction:08x}")?;
                }
                if let Some(next_instruction) = code.next_instruction {
                    write!(f, "/next=0x{next_instruction:08x}")?;
                }
                if let Some(origin) = code.current_trampoline_origin {
                    write!(f, "/tramp_origin=0x{origin:08x}")?;
                }
                if let Some(target) = code.direct_jump_target {
                    write!(f, "/jt=0x{target:08x}")?;
                    if code.direct_jump_target_in_trampoline {
                        write!(f, "/jt_trampoline=true")?;
                    }
                    if let Some(origin) = code.direct_jump_trampoline_origin {
                        write!(f, "/jt_origin=0x{origin:08x}")?;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SetjmpLongjmpTrapResult {
    result: u32,
    jumped: bool,
}

#[cfg(feature = "unicorn")]
const MIPS_JMPBUF_RETURN_PC_SLOT: u32 = 0;
#[cfg(feature = "unicorn")]
const MIPS_JMPBUF_SP_SLOT: u32 = 1;
#[cfg(feature = "unicorn")]
const MIPS_JMPBUF_FP_SLOT: u32 = 2;
#[cfg(feature = "unicorn")]
const MIPS_JMPBUF_RA_SLOT: u32 = 3;
#[cfg(feature = "unicorn")]
const MIPS_JMPBUF_GP_SLOT: u32 = 4;
#[cfg(feature = "unicorn")]
const MIPS_JMPBUF_S0_SLOT: u32 = 5;

#[cfg(feature = "unicorn")]
fn try_handle_setjmp_longjmp<D>(
    memory: &mut UnicornGuestMemory<'_, '_, D>,
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    name: Option<&str>,
    args: &[u32],
) -> Option<SetjmpLongjmpTrapResult> {
    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll {
        return None;
    }

    let is_setjmp = ordinal == Some(crate::ce::coredll_ordinals::ORD_SETJMP)
        || name.is_some_and(|name| name.eq_ignore_ascii_case("_setjmp"));
    if is_setjmp {
        let env = args.first().copied().unwrap_or(0);
        if save_mips_jmp_buf(memory, env).is_err() {
            let _ = memory.uc.emu_stop();
        }
        return Some(SetjmpLongjmpTrapResult {
            result: 0,
            jumped: false,
        });
    }

    let is_longjmp = ordinal == Some(crate::ce::coredll_ordinals::ORD_LONGJMP)
        || name.is_some_and(|name| name.eq_ignore_ascii_case("longjmp"));
    if !is_longjmp {
        return None;
    }

    let env = args.first().copied().unwrap_or(0);
    let value = match args.get(1).copied().unwrap_or(1) {
        0 => 1,
        value => value,
    };
    if restore_mips_jmp_buf(memory, env, value).is_err() {
        let _ = memory.uc.emu_stop();
    }
    Some(SetjmpLongjmpTrapResult {
        result: value,
        jumped: true,
    })
}

#[cfg(feature = "unicorn")]
fn save_mips_jmp_buf<D>(memory: &mut UnicornGuestMemory<'_, '_, D>, env: u32) -> Result<()> {
    use unicorn_engine::RegisterMIPS;

    write_mips_jmp_buf_slot(
        memory,
        env,
        MIPS_JMPBUF_RETURN_PC_SLOT,
        read_mips_reg(memory.uc, RegisterMIPS::RA),
    )?;
    write_mips_jmp_buf_slot(
        memory,
        env,
        MIPS_JMPBUF_SP_SLOT,
        read_mips_reg(memory.uc, RegisterMIPS::SP),
    )?;
    write_mips_jmp_buf_slot(
        memory,
        env,
        MIPS_JMPBUF_FP_SLOT,
        read_mips_reg(memory.uc, RegisterMIPS::FP),
    )?;
    write_mips_jmp_buf_slot(
        memory,
        env,
        MIPS_JMPBUF_RA_SLOT,
        read_mips_reg(memory.uc, RegisterMIPS::RA),
    )?;
    write_mips_jmp_buf_slot(
        memory,
        env,
        MIPS_JMPBUF_GP_SLOT,
        read_mips_reg(memory.uc, RegisterMIPS::GP),
    )?;
    for register in 16..=23 {
        let value = read_mips_gpr(memory.uc, register).unwrap_or(0);
        write_mips_jmp_buf_slot(memory, env, MIPS_JMPBUF_S0_SLOT + (register - 16), value)?;
    }
    tracing::debug!(
        target: "ce.crt",
        env = format_args!("0x{env:08x}"),
        return_pc = format_args!("0x{:08x}", read_mips_reg(memory.uc, RegisterMIPS::RA)),
        "saved MIPS _setjmp buffer"
    );
    Ok(())
}

#[cfg(feature = "unicorn")]
fn restore_mips_jmp_buf<D>(
    memory: &mut UnicornGuestMemory<'_, '_, D>,
    env: u32,
    value: u32,
) -> Result<()> {
    use unicorn_engine::RegisterMIPS;

    let return_pc = read_mips_jmp_buf_slot(memory, env, MIPS_JMPBUF_RETURN_PC_SLOT)?;
    let sp = read_mips_jmp_buf_slot(memory, env, MIPS_JMPBUF_SP_SLOT)?;
    let fp = read_mips_jmp_buf_slot(memory, env, MIPS_JMPBUF_FP_SLOT)?;
    let ra = read_mips_jmp_buf_slot(memory, env, MIPS_JMPBUF_RA_SLOT)?;
    let gp = read_mips_jmp_buf_slot(memory, env, MIPS_JMPBUF_GP_SLOT)?;
    for register in 16..=23 {
        let slot = MIPS_JMPBUF_S0_SLOT + (register - 16);
        let saved = read_mips_jmp_buf_slot(memory, env, slot)?;
        write_mips_gpr(memory.uc, register, saved)
            .ok_or_else(|| Error::Backend(format!("restore MIPS register ${register}")))?;
    }
    memory
        .uc
        .reg_write(RegisterMIPS::SP, u64::from(sp))
        .map_err(|err| Error::Backend(format!("restore MIPS SP: {err:?}")))?;
    memory
        .uc
        .reg_write(RegisterMIPS::FP, u64::from(fp))
        .map_err(|err| Error::Backend(format!("restore MIPS FP: {err:?}")))?;
    memory
        .uc
        .reg_write(RegisterMIPS::RA, u64::from(ra))
        .map_err(|err| Error::Backend(format!("restore MIPS RA: {err:?}")))?;
    memory
        .uc
        .reg_write(RegisterMIPS::GP, u64::from(gp))
        .map_err(|err| Error::Backend(format!("restore MIPS GP: {err:?}")))?;
    memory
        .uc
        .reg_write(RegisterMIPS::V0, u64::from(value))
        .map_err(|err| Error::Backend(format!("restore MIPS V0: {err:?}")))?;
    memory
        .uc
        .reg_write(RegisterMIPS::PC, u64::from(return_pc))
        .map_err(|err| Error::Backend(format!("restore MIPS PC: {err:?}")))?;
    tracing::debug!(
        target: "ce.crt",
        env = format_args!("0x{env:08x}"),
        return_pc = format_args!("0x{return_pc:08x}"),
        value = format_args!("0x{value:08x}"),
        "restored MIPS longjmp buffer"
    );
    Ok(())
}

#[cfg(feature = "unicorn")]
fn read_mips_jmp_buf_slot<M: CoredllGuestMemory>(memory: &M, env: u32, slot: u32) -> Result<u32> {
    memory.read_u32(jmp_buf_slot_addr(env, slot)?)
}

#[cfg(feature = "unicorn")]
fn write_mips_jmp_buf_slot<M: CoredllGuestMemory>(
    memory: &mut M,
    env: u32,
    slot: u32,
    value: u32,
) -> Result<()> {
    memory.write_u32(jmp_buf_slot_addr(env, slot)?, value)
}

#[cfg(feature = "unicorn")]
fn jmp_buf_slot_addr(env: u32, slot: u32) -> Result<u32> {
    env.checked_add(slot.checked_mul(4).unwrap_or(u32::MAX))
        .ok_or_else(|| Error::InvalidArgument("MIPS jmp_buf slot overflow".to_owned()))
}

#[cfg(feature = "unicorn")]
fn try_block_empty_get_message<D>(
    kernel: &mut CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    args: &[u32],
    thread_id: u32,
    blocked: &std::rc::Rc<std::cell::RefCell<Option<UnicornBlockedGetMessage>>>,
    blocked_thread: &std::rc::Rc<std::cell::RefCell<Option<BlockedGuestThread>>>,
    pending_returns: &std::rc::Rc<std::cell::RefCell<Vec<PendingGuestThreadReturn>>>,
    current_thread_id: &std::rc::Rc<std::cell::RefCell<u32>>,
    suspended_thread: &std::rc::Rc<std::cell::RefCell<Option<SuspendedGuestThread>>>,
    running_thread: &std::rc::Rc<std::cell::RefCell<Option<(u32, u32)>>>,
    last_messages: &std::rc::Rc<std::cell::RefCell<Vec<UnicornLastMessage>>>,
) -> bool {
    use unicorn_engine::RegisterMIPS;

    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll
        || ordinal != Some(crate::ce::coredll_ordinals::ORD_GET_MESSAGE_W)
    {
        return false;
    }

    let hwnd = args.get(1).copied().filter(|hwnd| *hwnd != 0);
    let min_msg = args.get(2).copied().unwrap_or(0);
    let max_msg = args.get(3).copied().unwrap_or(0);
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
    record_message_import(uc, module_kind, ordinal, args, None, last_messages);

    let mut pending_returns = pending_returns.borrow_mut();
    if let Some(callout) = pending_returns.pop() {
        *blocked_thread.borrow_mut() = Some(BlockedGuestThread {
            thread_id,
            thread_handle: callout.thread_handle,
            regs: capture_mips_gprs(uc),
            return_pc: read_mips_reg(uc, RegisterMIPS::RA),
            msg_ptr: args.first().copied().unwrap_or(0),
            hwnd,
            min_msg,
            max_msg,
        });
        *running_thread.borrow_mut() = None;
        *current_thread_id.borrow_mut() = callout.creator_thread_id;
        restore_mips_gprs(uc, &callout.creator_regs);
        let writes = [
            uc.reg_write(RegisterMIPS::V0, u64::from(callout.thread_handle)),
            uc.reg_write(RegisterMIPS::PC, u64::from(callout.return_pc)),
            uc.reg_write(RegisterMIPS::RA, u64::from(callout.return_pc)),
        ];
        if writes.into_iter().any(|write| write.is_err()) {
            let _ = uc.emu_stop();
        }
        return true;
    }

    let running = *running_thread.borrow();
    if let Some((_, thread_handle)) = running {
        *blocked_thread.borrow_mut() = Some(BlockedGuestThread {
            thread_id,
            thread_handle,
            regs: capture_mips_gprs(uc),
            return_pc: read_mips_reg(uc, RegisterMIPS::RA),
            msg_ptr: args.first().copied().unwrap_or(0),
            hwnd,
            min_msg,
            max_msg,
        });
        *running_thread.borrow_mut() = None;
        if let Some(suspended) = suspended_thread.borrow_mut().take() {
            *current_thread_id.borrow_mut() = suspended.thread_id;
            restore_suspended_thread(uc, &suspended);
            return true;
        }
    }
    let _ = uc.emu_stop();
    true
}

#[cfg(feature = "unicorn")]
fn try_block_wait_for_single_object<D>(
    kernel: &mut CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    args: &[u32],
    thread_id: u32,
    blocked_waits: &std::rc::Rc<std::cell::RefCell<Vec<BlockedWaitThread>>>,
    pending_returns: &std::rc::Rc<std::cell::RefCell<Vec<PendingGuestThreadReturn>>>,
    current_thread_id: &std::rc::Rc<std::cell::RefCell<u32>>,
    suspended_thread: &std::rc::Rc<std::cell::RefCell<Option<SuspendedGuestThread>>>,
    running_thread: &std::rc::Rc<std::cell::RefCell<Option<(u32, u32)>>>,
) -> bool {
    use unicorn_engine::RegisterMIPS;

    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll
        || ordinal != Some(crate::ce::coredll_ordinals::ORD_WAIT_FOR_SINGLE_OBJECT)
    {
        return false;
    }

    let wait_handle = args.first().copied().unwrap_or(0);
    let timeout = args.get(1).copied().unwrap_or(0);
    if timeout == 0 || kernel.is_wait_ready(wait_handle, thread_id) != Some(false) {
        return false;
    }

    let regs = capture_mips_gprs(uc);
    let return_pc = read_mips_reg(uc, RegisterMIPS::RA);
    if let Some(callout) = pending_returns.borrow_mut().pop() {
        blocked_waits.borrow_mut().push(BlockedWaitThread {
            thread_id,
            thread_handle: callout.thread_handle,
            wait_handle,
            regs,
            return_pc,
        });
        *running_thread.borrow_mut() = None;
        *current_thread_id.borrow_mut() = callout.creator_thread_id;
        restore_mips_gprs(uc, &callout.creator_regs);
        let writes = [
            uc.reg_write(RegisterMIPS::V0, u64::from(callout.thread_handle)),
            uc.reg_write(RegisterMIPS::PC, u64::from(callout.return_pc)),
            uc.reg_write(RegisterMIPS::RA, u64::from(callout.return_pc)),
        ];
        if writes.into_iter().any(|write| write.is_err()) {
            let _ = uc.emu_stop();
        }
        return true;
    }

    let running = *running_thread.borrow();
    if let Some((_, thread_handle)) = running {
        blocked_waits.borrow_mut().push(BlockedWaitThread {
            thread_id,
            thread_handle,
            wait_handle,
            regs,
            return_pc,
        });
        *running_thread.borrow_mut() = None;
        if let Some(suspended) = suspended_thread.borrow_mut().take() {
            *current_thread_id.borrow_mut() = suspended.thread_id;
            restore_suspended_thread(uc, &suspended);
            return true;
        }
    }
    false
}

#[cfg(feature = "unicorn")]
fn try_resume_blocked_wait<D>(
    kernel: &mut CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    active_thread_id: u32,
    current_thread_id: &std::rc::Rc<std::cell::RefCell<u32>>,
    blocked_waits: &std::rc::Rc<std::cell::RefCell<Vec<BlockedWaitThread>>>,
    suspended_thread: &std::rc::Rc<std::cell::RefCell<Option<SuspendedGuestThread>>>,
    running_thread: &std::rc::Rc<std::cell::RefCell<Option<(u32, u32)>>>,
) -> bool {
    use unicorn_engine::RegisterMIPS;

    if running_thread.borrow().is_some() {
        return false;
    }

    let ready_index = {
        let blocked_waits = blocked_waits.borrow();
        blocked_waits
            .iter()
            .enumerate()
            .filter(|(_, blocked)| {
                blocked.thread_id != active_thread_id
                    && kernel.is_wait_ready(blocked.wait_handle, blocked.thread_id) == Some(true)
            })
            .max_by_key(|(index, blocked)| {
                (
                    kernel.thread_priority_by_id(blocked.thread_id),
                    std::cmp::Reverse(*index),
                )
            })
            .map(|(index, _)| index)
    };
    let Some(index) = ready_index else {
        return false;
    };
    let blocked = blocked_waits.borrow_mut().remove(index);
    let wait_result = kernel.wait_for_single_object(blocked.wait_handle, 0, blocked.thread_id);

    let mut current = SuspendedGuestThread {
        thread_id: active_thread_id,
        regs: capture_mips_gprs(uc),
        pc: read_mips_reg(uc, RegisterMIPS::RA),
    };
    current.regs[2] = read_mips_reg(uc, RegisterMIPS::V0);
    *suspended_thread.borrow_mut() = Some(current);

    let mut regs = blocked.regs;
    regs[2] = wait_result;
    restore_mips_gprs(uc, &regs);
    let writes = [
        uc.reg_write(RegisterMIPS::PC, u64::from(blocked.return_pc)),
        uc.reg_write(RegisterMIPS::RA, u64::from(blocked.return_pc)),
    ];
    if writes.into_iter().any(|write| write.is_err()) {
        let _ = uc.emu_stop();
        return true;
    }
    *current_thread_id.borrow_mut() = blocked.thread_id;
    *running_thread.borrow_mut() = Some((blocked.thread_id, blocked.thread_handle));
    true
}

#[cfg(feature = "unicorn")]
fn try_exit_guest_thread_callout<D>(
    kernel: &mut CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    args: &[u32],
    pending_returns: &std::rc::Rc<std::cell::RefCell<Vec<PendingGuestThreadReturn>>>,
    current_thread_id: &std::rc::Rc<std::cell::RefCell<u32>>,
    suspended_thread: &std::rc::Rc<std::cell::RefCell<Option<SuspendedGuestThread>>>,
    running_thread: &std::rc::Rc<std::cell::RefCell<Option<(u32, u32)>>>,
    stack_slots: &GuestThreadStackSlots,
) -> bool {
    use unicorn_engine::RegisterMIPS;

    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll
        || ordinal != Some(crate::ce::coredll_ordinals::ORD_EXIT_THREAD)
    {
        return false;
    }

    let exit_code = args.first().copied().unwrap_or(0);
    if let Some(callout) = pending_returns.borrow_mut().pop() {
        kernel.mark_guest_thread_exited(callout.thread_handle, exit_code);
        release_guest_thread_stack_slot(stack_slots, callout.worker_thread_id);
        *current_thread_id.borrow_mut() = callout.creator_thread_id;
        *running_thread.borrow_mut() = None;
        restore_mips_gprs(uc, &callout.creator_regs);
        let writes = [
            uc.reg_write(RegisterMIPS::V0, u64::from(callout.thread_handle)),
            uc.reg_write(RegisterMIPS::PC, u64::from(callout.return_pc)),
            uc.reg_write(RegisterMIPS::RA, u64::from(callout.return_pc)),
        ];
        if writes.into_iter().any(|write| write.is_err()) {
            let _ = uc.emu_stop();
        }
        return true;
    }

    let Some((worker_thread_id, thread_handle)) = running_thread.borrow_mut().take() else {
        let _ = uc.emu_stop();
        return true;
    };
    kernel.mark_guest_thread_exited(thread_handle, exit_code);
    release_guest_thread_stack_slot(stack_slots, worker_thread_id);
    let Some(suspended) = suspended_thread.borrow_mut().take() else {
        let _ = uc.emu_stop();
        return true;
    };
    *current_thread_id.borrow_mut() = suspended.thread_id;
    restore_suspended_thread(uc, &suspended);
    true
}

#[cfg(feature = "unicorn")]
fn try_enter_create_thread_callout<D>(
    kernel: &mut CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    args: &[u32],
    creator_thread_id: u32,
    process_stack_top: u32,
    current_thread_id: &std::rc::Rc<std::cell::RefCell<u32>>,
    running_thread: &std::rc::Rc<std::cell::RefCell<Option<(u32, u32)>>>,
    stack_slots: &GuestThreadStackSlots,
    pending_returns: &std::rc::Rc<std::cell::RefCell<Vec<PendingGuestThreadReturn>>>,
) -> bool {
    use unicorn_engine::RegisterMIPS;

    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll
        || ordinal != Some(crate::ce::coredll_ordinals::ORD_CREATE_THREAD)
    {
        return false;
    }

    let start_address = args.get(2).copied().unwrap_or(0);
    if start_address == 0 || process_stack_top == 0 {
        kernel.threads.set_last_error(
            creator_thread_id,
            crate::ce::thread::ERROR_INVALID_PARAMETER,
        );
        let _ = uc.reg_write(RegisterMIPS::V0, 0);
        return true;
    }
    let parameter = args.get(3).copied().unwrap_or(0);
    let suspended = args.get(4).copied().unwrap_or(0) & 0x0000_0004 != 0;
    let thread_id_ptr = args.get(5).copied().unwrap_or(0);
    let (thread_handle, worker_thread_id) =
        kernel.create_guest_thread(start_address, parameter, suspended);
    if thread_id_ptr != 0
        && uc
            .mem_write(u64::from(thread_id_ptr), &worker_thread_id.to_le_bytes())
            .is_err()
    {
        let _ = kernel.close_handle(thread_handle);
        kernel.threads.set_last_error(
            creator_thread_id,
            crate::ce::thread::ERROR_INVALID_PARAMETER,
        );
        let _ = uc.reg_write(RegisterMIPS::V0, 0);
        return true;
    }
    kernel.threads.set_last_error(creator_thread_id, 0);
    if suspended {
        let _ = uc.reg_write(RegisterMIPS::V0, u64::from(thread_handle));
        return true;
    }

    let creator_regs = capture_mips_gprs(uc);
    let return_pc = read_mips_reg(uc, RegisterMIPS::RA);
    pending_returns.borrow_mut().push(PendingGuestThreadReturn {
        creator_thread_id,
        worker_thread_id,
        thread_handle,
        return_pc,
        creator_regs,
    });
    *current_thread_id.borrow_mut() = worker_thread_id;
    *running_thread.borrow_mut() = Some((worker_thread_id, thread_handle));
    let stack_slot = assign_guest_thread_stack_slot(stack_slots, worker_thread_id);
    let worker_stack = guest_thread_stack_top(process_stack_top, stack_slot);
    let writes = [
        uc.reg_write(RegisterMIPS::PC, u64::from(start_address)),
        uc.reg_write(RegisterMIPS::RA, u64::from(GUEST_THREAD_RETURN_STUB_ADDR)),
        uc.reg_write(RegisterMIPS::A0, u64::from(parameter)),
        uc.reg_write(RegisterMIPS::SP, u64::from(worker_stack)),
        uc.reg_write(RegisterMIPS::T9, u64::from(start_address)),
    ];
    tracing::debug!(
        target: "ce.imports",
        creator_thread_id,
        worker_thread_id,
        stack_slot,
        handle = format_args!("0x{thread_handle:08x}"),
        start = format_args!("0x{start_address:08x}"),
        parameter = format_args!("0x{parameter:08x}"),
        stack = format_args!("0x{worker_stack:08x}"),
        return_pc = format_args!("0x{return_pc:08x}"),
        "enter guest thread"
    );
    if writes.into_iter().any(|write| write.is_err()) {
        let _ = uc.emu_stop();
    }
    true
}

#[cfg(feature = "unicorn")]
fn try_enter_resumed_thread_callout<D>(
    kernel: &CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    args: &[u32],
    result: u32,
    creator_thread_id: u32,
    process_stack_top: u32,
    current_thread_id: &std::rc::Rc<std::cell::RefCell<u32>>,
    suspended_thread: &std::rc::Rc<std::cell::RefCell<Option<SuspendedGuestThread>>>,
    running_thread: &std::rc::Rc<std::cell::RefCell<Option<(u32, u32)>>>,
    stack_slots: &GuestThreadStackSlots,
) -> bool {
    use unicorn_engine::RegisterMIPS;

    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll
        || ordinal != Some(crate::ce::coredll_ordinals::ORD_RESUME_THREAD)
        || result == u32::MAX
        || process_stack_top == 0
        || running_thread.borrow().is_some()
    {
        return false;
    }
    let thread_handle = args.first().copied().unwrap_or(0);
    let Some((worker_thread_id, start_address, parameter)) =
        kernel.guest_thread_start(thread_handle)
    else {
        return false;
    };

    let mut creator = SuspendedGuestThread {
        thread_id: creator_thread_id,
        regs: capture_mips_gprs(uc),
        pc: read_mips_reg(uc, RegisterMIPS::RA),
    };
    creator.regs[2] = result;
    *suspended_thread.borrow_mut() = Some(creator);
    *current_thread_id.borrow_mut() = worker_thread_id;
    *running_thread.borrow_mut() = Some((worker_thread_id, thread_handle));

    let worker_stack = guest_thread_stack_top(
        process_stack_top,
        assign_guest_thread_stack_slot(stack_slots, worker_thread_id),
    );
    let writes = [
        uc.reg_write(RegisterMIPS::PC, u64::from(start_address)),
        uc.reg_write(RegisterMIPS::RA, u64::from(GUEST_THREAD_RETURN_STUB_ADDR)),
        uc.reg_write(RegisterMIPS::A0, u64::from(parameter)),
        uc.reg_write(RegisterMIPS::SP, u64::from(worker_stack)),
        uc.reg_write(RegisterMIPS::T9, u64::from(start_address)),
    ];
    if writes.into_iter().any(|write| write.is_err()) {
        let _ = uc.emu_stop();
    }
    true
}

#[cfg(feature = "unicorn")]
fn try_resume_blocked_get_message<D>(
    kernel: &mut CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    active_thread_id: u32,
    current_thread_id: &std::rc::Rc<std::cell::RefCell<u32>>,
    blocked_thread: &std::rc::Rc<std::cell::RefCell<Option<BlockedGuestThread>>>,
    suspended_thread: &std::rc::Rc<std::cell::RefCell<Option<SuspendedGuestThread>>>,
    running_thread: &std::rc::Rc<std::cell::RefCell<Option<(u32, u32)>>>,
) -> bool {
    use unicorn_engine::RegisterMIPS;

    let Some(blocked) = blocked_thread.borrow().as_ref().cloned() else {
        return false;
    };
    if active_thread_id == blocked.thread_id {
        return false;
    }
    kernel.pump_timers_to_gwe(blocked.thread_id);
    let Some(message) = kernel.gwe.get_message_filtered(
        blocked.thread_id,
        blocked.hwnd,
        blocked.min_msg,
        blocked.max_msg,
    ) else {
        return false;
    };
    if write_unicorn_message(uc, blocked.msg_ptr, &message).is_err() {
        let _ = uc.emu_stop();
        return true;
    }

    let mut current = SuspendedGuestThread {
        thread_id: active_thread_id,
        regs: capture_mips_gprs(uc),
        pc: read_mips_reg(uc, RegisterMIPS::RA),
    };
    current.regs[2] = read_mips_reg(uc, RegisterMIPS::V0);
    *suspended_thread.borrow_mut() = Some(current);

    let mut regs = blocked.regs;
    regs[2] = u32::from(message.msg != crate::ce::gwe::WM_QUIT);
    restore_mips_gprs(uc, &regs);
    let writes = [
        uc.reg_write(RegisterMIPS::PC, u64::from(blocked.return_pc)),
        uc.reg_write(RegisterMIPS::RA, u64::from(blocked.return_pc)),
    ];
    if writes.into_iter().any(|write| write.is_err()) {
        let _ = uc.emu_stop();
        return true;
    }
    *current_thread_id.borrow_mut() = blocked.thread_id;
    *running_thread.borrow_mut() = Some((blocked.thread_id, blocked.thread_handle));
    *blocked_thread.borrow_mut() = None;
    true
}

#[cfg(feature = "unicorn")]
fn restore_suspended_thread<D>(
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    suspended: &SuspendedGuestThread,
) {
    use unicorn_engine::RegisterMIPS;

    restore_mips_gprs(uc, &suspended.regs);
    let _ = uc.reg_write(RegisterMIPS::PC, u64::from(suspended.pc));
}

#[cfg(feature = "unicorn")]
fn write_unicorn_message<D>(
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    addr: u32,
    message: &crate::ce::gwe::Message,
) -> Result<()> {
    let mut bytes = [0u8; 28];
    bytes[0..4].copy_from_slice(&message.hwnd.to_le_bytes());
    bytes[4..8].copy_from_slice(&message.msg.to_le_bytes());
    bytes[8..12].copy_from_slice(&message.wparam.to_le_bytes());
    bytes[12..16].copy_from_slice(&message.lparam.to_le_bytes());
    bytes[16..20].copy_from_slice(&message.time_ms.to_le_bytes());
    uc.mem_write(u64::from(addr), &bytes)
        .map_err(|err| Error::Backend(format!("write resumed MSG 0x{addr:08x}: {err:?}")))
}

#[cfg(feature = "unicorn")]
fn guest_thread_stack_top(process_stack_top: u32, thread_id: u32) -> u32 {
    let offset = 0x0002_0000u32.saturating_mul(thread_id.max(1));
    process_stack_top.wrapping_sub(offset) & !0x7
}

#[cfg(feature = "unicorn")]
fn assign_guest_thread_stack_slot(stack_slots: &GuestThreadStackSlots, thread_id: u32) -> u32 {
    let mut stack_slots = stack_slots.borrow_mut();
    if let Some(slot) = stack_slots.get(&thread_id).copied() {
        return slot;
    }

    let mut slot = 1;
    while stack_slots.values().any(|used| *used == slot) {
        slot += 1;
    }
    stack_slots.insert(thread_id, slot);
    slot
}

#[cfg(feature = "unicorn")]
fn release_guest_thread_stack_slot(stack_slots: &GuestThreadStackSlots, thread_id: u32) {
    stack_slots.borrow_mut().remove(&thread_id);
}

#[cfg(feature = "unicorn")]
fn capture_mips_gprs<D>(uc: &unicorn_engine::Unicorn<'_, D>) -> [u32; 32] {
    let mut regs = [0; 32];
    for register in 1..32 {
        regs[register as usize] = read_mips_gpr(uc, register).unwrap_or(0);
    }
    regs
}

#[cfg(feature = "unicorn")]
fn restore_mips_gprs<D>(uc: &mut unicorn_engine::Unicorn<'_, D>, regs: &[u32; 32]) {
    for register in 1..32 {
        let _ = write_mips_gpr(uc, register, regs[register as usize]);
    }
}

#[cfg(feature = "unicorn")]
fn record_message_import<D>(
    uc: &unicorn_engine::Unicorn<'_, D>,
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    args: &[u32],
    result: Option<u32>,
    last_messages: &std::rc::Rc<std::cell::RefCell<Vec<UnicornLastMessage>>>,
) {
    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll {
        return;
    }
    let Some(ordinal) = ordinal else {
        return;
    };
    if ordinal != crate::ce::coredll_ordinals::ORD_GET_MESSAGE_W
        && ordinal != crate::ce::coredll_ordinals::ORD_PEEK_MESSAGE_W
    {
        return;
    }

    let msg_ptr = args.first().copied().unwrap_or(0);
    let message = result
        .filter(|result| *result != 0)
        .and_then(|_| read_unicorn_message(uc, msg_ptr));
    let mut messages = last_messages.borrow_mut();
    if messages.len() == UNICORN_TRACE_LIMIT {
        messages.remove(0);
    }
    messages.push(UnicornLastMessage {
        ordinal,
        msg_ptr,
        filter_hwnd: args.get(1).copied().filter(|hwnd| *hwnd != 0),
        min_msg: args.get(2).copied().unwrap_or(0),
        max_msg: args.get(3).copied().unwrap_or(0),
        flags: (ordinal == crate::ce::coredll_ordinals::ORD_PEEK_MESSAGE_W)
            .then(|| args.get(4).copied().unwrap_or(0)),
        result,
        message,
    });
}

#[cfg(feature = "unicorn")]
fn read_unicorn_message<D>(
    uc: &unicorn_engine::Unicorn<'_, D>,
    msg_ptr: u32,
) -> Option<UnicornMessageRecord> {
    Some(UnicornMessageRecord {
        hwnd: read_unicorn_u32(uc, msg_ptr)?,
        msg: read_unicorn_u32(uc, msg_ptr.wrapping_add(4))?,
        wparam: read_unicorn_u32(uc, msg_ptr.wrapping_add(8))?,
        lparam: read_unicorn_u32(uc, msg_ptr.wrapping_add(12))?,
        time_ms: read_unicorn_u32(uc, msg_ptr.wrapping_add(16))?,
    })
}

#[cfg(feature = "unicorn")]
fn record_wndproc_return(
    last_wndproc_returns: &std::rc::Rc<std::cell::RefCell<Vec<UnicornWndProcReturn>>>,
    record: UnicornWndProcReturn,
) {
    tracing::debug!(
        target: "ce.gwe",
        source = record.source,
        hwnd = format_args!("0x{:08x}", record.hwnd),
        msg = format_args!("0x{:08x}", record.msg),
        wparam = format_args!("0x{:08x}", record.wparam),
        lparam = format_args!("0x{:08x}", record.lparam),
        wndproc = format_args!("0x{:08x}", record.wndproc),
        return_pc = format_args!("0x{:08x}", record.return_pc),
        result = format_args!("0x{:08x}", record.result),
        class = record.class_name.as_deref().unwrap_or("<unknown>"),
        "guest wndproc return"
    );
    let mut returns = last_wndproc_returns.borrow_mut();
    if returns.len() == UNICORN_TRACE_LIMIT {
        returns.remove(0);
    }
    returns.push(record);
}

#[cfg(feature = "unicorn")]
fn is_guest_wndproc(wndproc: u32) -> bool {
    wndproc != 0 && wndproc != crate::ce::gwe::DEFAULT_WNDPROC
}

#[cfg(feature = "unicorn")]
fn try_enter_dispatch_message_callout<D>(
    kernel: &CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    args: &[u32],
    pending_returns: &std::rc::Rc<std::cell::RefCell<Vec<PendingWndProcReturn>>>,
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
    if !is_guest_wndproc(wndproc) {
        return false;
    }
    let return_pc = read_mips_reg(uc, RegisterMIPS::RA);

    tracing::debug!(
        target: "ce.gwe",
        msg_ptr = format_args!("0x{msg_ptr:08x}"),
        hwnd = format_args!("0x{hwnd:08x}"),
        msg = format_args!("0x{msg:08x}"),
        wparam = format_args!("0x{wparam:08x}"),
        lparam = format_args!("0x{lparam:08x}"),
        class = window.class_name.as_str(),
        wndproc = format_args!("0x{wndproc:08x}"),
        ra = format_args!("0x{return_pc:08x}"),
        "DispatchMessageW guest wndproc callout"
    );

    pending_returns.borrow_mut().push(PendingWndProcReturn {
        source: "DispatchMessageW",
        hwnd,
        msg,
        wparam,
        lparam,
        wndproc,
        return_pc,
        class_name: Some(window.class_name.clone()),
        api_result: None,
        dialog_result_hwnd: None,
        finalize_destroy: false,
        send_thread_id: None,
        send_timeout_result_ptr: None,
    });
    let writes = [
        uc.reg_write(RegisterMIPS::A0, u64::from(hwnd)),
        uc.reg_write(RegisterMIPS::A1, u64::from(msg)),
        uc.reg_write(RegisterMIPS::A2, u64::from(wparam)),
        uc.reg_write(RegisterMIPS::A3, u64::from(lparam)),
        uc.reg_write(RegisterMIPS::RA, u64::from(WNDPROC_RETURN_STUB_ADDR)),
        uc.reg_write(RegisterMIPS::T9, u64::from(wndproc)),
        uc.reg_write(RegisterMIPS::PC, u64::from(wndproc)),
    ];
    if writes.into_iter().all(|write| write.is_ok()) {
        true
    } else {
        let _ = pending_returns.borrow_mut().pop();
        false
    }
}

#[cfg(feature = "unicorn")]
fn try_enter_send_message_callout<D>(
    kernel: &mut CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    args: &[u32],
    pending_returns: &std::rc::Rc<std::cell::RefCell<Vec<PendingWndProcReturn>>>,
) -> bool {
    use unicorn_engine::RegisterMIPS;

    let is_send_message = ordinal == Some(crate::ce::coredll_ordinals::ORD_SEND_MESSAGE_W);
    let is_send_message_timeout =
        ordinal == Some(crate::ce::coredll_ordinals::ORD_SEND_MESSAGE_TIMEOUT);
    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll
        || (!is_send_message && !is_send_message_timeout)
    {
        return false;
    }

    let hwnd = args.first().copied().unwrap_or(0);
    if !kernel.gwe.is_window(hwnd) {
        return false;
    }
    let Some(window) = kernel.gwe.window(hwnd) else {
        return false;
    };
    let wndproc = window.wndproc;
    let target_thread_id = window.thread_id;
    let class_name = window.class_name.clone();
    if !is_guest_wndproc(wndproc) {
        return false;
    }
    let msg = args.get(1).copied().unwrap_or(0);
    let wparam = args.get(2).copied().unwrap_or(0);
    let lparam = args.get(3).copied().unwrap_or(0);
    let result_ptr = is_send_message_timeout
        .then(|| args.get(6).copied().unwrap_or(0))
        .filter(|ptr| *ptr != 0);
    let return_pc = read_mips_reg(uc, RegisterMIPS::RA);

    tracing::debug!(
        target: "ce.gwe",
        source = if is_send_message_timeout { "SendMessageTimeout" } else { "SendMessageW" },
        hwnd = format_args!("0x{hwnd:08x}"),
        msg = format_args!("0x{msg:08x}"),
        wparam = format_args!("0x{wparam:08x}"),
        lparam = format_args!("0x{lparam:08x}"),
        class = class_name.as_str(),
        wndproc = format_args!("0x{wndproc:08x}"),
        ra = format_args!("0x{return_pc:08x}"),
        "SendMessage guest wndproc callout"
    );

    kernel.gwe.begin_send_message(target_thread_id);
    pending_returns.borrow_mut().push(PendingWndProcReturn {
        source: if is_send_message_timeout {
            "SendMessageTimeout"
        } else {
            "SendMessageW"
        },
        hwnd,
        msg,
        wparam,
        lparam,
        wndproc,
        return_pc,
        class_name: Some(class_name),
        api_result: None,
        dialog_result_hwnd: None,
        finalize_destroy: false,
        send_thread_id: Some(target_thread_id),
        send_timeout_result_ptr: result_ptr,
    });
    let writes = [
        uc.reg_write(RegisterMIPS::A0, u64::from(hwnd)),
        uc.reg_write(RegisterMIPS::A1, u64::from(msg)),
        uc.reg_write(RegisterMIPS::A2, u64::from(wparam)),
        uc.reg_write(RegisterMIPS::A3, u64::from(lparam)),
        uc.reg_write(RegisterMIPS::RA, u64::from(WNDPROC_RETURN_STUB_ADDR)),
        uc.reg_write(RegisterMIPS::T9, u64::from(wndproc)),
        uc.reg_write(RegisterMIPS::PC, u64::from(wndproc)),
    ];
    if writes.into_iter().all(|write| write.is_ok()) {
        true
    } else {
        let _ = pending_returns.borrow_mut().pop();
        kernel.gwe.end_send_message(target_thread_id);
        false
    }
}

#[cfg(feature = "unicorn")]
fn try_enter_def_window_proc_callout<D>(
    kernel: &mut CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    args: &[u32],
    pending_returns: &std::rc::Rc<std::cell::RefCell<Vec<PendingWndProcReturn>>>,
) -> bool {
    use unicorn_engine::RegisterMIPS;

    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll
        || ordinal != Some(crate::ce::coredll_ordinals::ORD_DEF_WINDOW_PROC_W)
    {
        return false;
    }

    let hwnd = args.first().copied().unwrap_or(0);
    let msg = args.get(1).copied().unwrap_or(0);
    let wparam = args.get(2).copied().unwrap_or(0);
    let lparam = args.get(3).copied().unwrap_or(0);
    if msg == crate::ce::gwe::WM_CLOSE {
        return enter_destroy_window_wm_destroy_callout(
            kernel,
            uc,
            hwnd,
            "DefWindowProcW/WM_CLOSE",
            0,
            pending_returns,
        );
    }

    let result = crate::ce::gwe::default_send_message_result(msg, wparam, lparam);
    let _ = uc.reg_write(RegisterMIPS::V0, u64::from(result));
    true
}

#[cfg(feature = "unicorn")]
fn try_enter_def_dlg_proc_callout<D>(
    kernel: &CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    args: &[u32],
    pending_returns: &std::rc::Rc<std::cell::RefCell<Vec<PendingWndProcReturn>>>,
) -> bool {
    use unicorn_engine::RegisterMIPS;

    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll
        || ordinal != Some(crate::ce::coredll_ordinals::ORD_DEF_DLG_PROC_W)
    {
        return false;
    }

    let hwnd = args.first().copied().unwrap_or(0);
    let dlgproc = kernel
        .gwe
        .get_window_long(hwnd, crate::ce::gwe::DWL_DLGPROC)
        .unwrap_or(0);
    if !is_guest_wndproc(dlgproc) {
        return false;
    }
    let msg = args.get(1).copied().unwrap_or(0);
    let wparam = args.get(2).copied().unwrap_or(0);
    let lparam = args.get(3).copied().unwrap_or(0);
    let class_name = kernel
        .gwe
        .window(hwnd)
        .map(|window| window.class_name.clone());
    let return_pc = read_mips_reg(uc, RegisterMIPS::RA);

    tracing::debug!(
        target: "ce.gwe",
        hwnd = format_args!("0x{hwnd:08x}"),
        msg = format_args!("0x{msg:08x}"),
        wparam = format_args!("0x{wparam:08x}"),
        lparam = format_args!("0x{lparam:08x}"),
        dlgproc = format_args!("0x{dlgproc:08x}"),
        ra = format_args!("0x{return_pc:08x}"),
        "DefDlgProcW guest dlgproc callout"
    );

    pending_returns.borrow_mut().push(PendingWndProcReturn {
        source: "DefDlgProcW",
        hwnd,
        msg,
        wparam,
        lparam,
        wndproc: dlgproc,
        return_pc,
        class_name,
        api_result: None,
        dialog_result_hwnd: None,
        finalize_destroy: false,
        send_thread_id: None,
        send_timeout_result_ptr: None,
    });
    let writes = [
        uc.reg_write(RegisterMIPS::A0, u64::from(hwnd)),
        uc.reg_write(RegisterMIPS::A1, u64::from(msg)),
        uc.reg_write(RegisterMIPS::A2, u64::from(wparam)),
        uc.reg_write(RegisterMIPS::A3, u64::from(lparam)),
        uc.reg_write(RegisterMIPS::RA, u64::from(WNDPROC_RETURN_STUB_ADDR)),
        uc.reg_write(RegisterMIPS::T9, u64::from(dlgproc)),
        uc.reg_write(RegisterMIPS::PC, u64::from(dlgproc)),
    ];
    if writes.into_iter().all(|write| write.is_ok()) {
        true
    } else {
        let _ = pending_returns.borrow_mut().pop();
        false
    }
}

#[cfg(feature = "unicorn")]
fn try_enter_destroy_window_callout<D>(
    kernel: &mut CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    args: &[u32],
    pending_returns: &std::rc::Rc<std::cell::RefCell<Vec<PendingWndProcReturn>>>,
) -> bool {
    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll
        || ordinal != Some(crate::ce::coredll_ordinals::ORD_DESTROY_WINDOW)
    {
        return false;
    }

    let hwnd = args.first().copied().unwrap_or(0);
    enter_destroy_window_wm_destroy_callout(
        kernel,
        uc,
        hwnd,
        "DestroyWindow/WM_DESTROY",
        1,
        pending_returns,
    )
}

#[cfg(feature = "unicorn")]
fn enter_destroy_window_wm_destroy_callout<D>(
    kernel: &mut CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    hwnd: u32,
    source: &'static str,
    api_result: u32,
    pending_returns: &std::rc::Rc<std::cell::RefCell<Vec<PendingWndProcReturn>>>,
) -> bool {
    use unicorn_engine::RegisterMIPS;

    let Some(window) = kernel.gwe.window(hwnd) else {
        return false;
    };
    let wndproc = window.wndproc;
    let class_name = window.class_name.clone();
    if !is_guest_wndproc(wndproc) {
        let destroyed = kernel.gwe.destroy_window(hwnd, kernel.timers.tick_count());
        let result = if api_result == 0 {
            0
        } else {
            u32::from(destroyed)
        };
        let _ = uc.reg_write(RegisterMIPS::V0, u64::from(result));
        return true;
    }

    let return_pc = read_mips_reg(uc, RegisterMIPS::RA);
    tracing::debug!(
        target: "ce.gwe",
        hwnd = format_args!("0x{hwnd:08x}"),
        wndproc = format_args!("0x{wndproc:08x}"),
        ra = format_args!("0x{return_pc:08x}"),
        "DestroyWindow guest WM_DESTROY callout"
    );

    pending_returns.borrow_mut().push(PendingWndProcReturn {
        source,
        hwnd,
        msg: crate::ce::gwe::WM_DESTROY,
        wparam: 0,
        lparam: 0,
        wndproc,
        return_pc,
        class_name: Some(class_name),
        api_result: Some(api_result),
        dialog_result_hwnd: None,
        finalize_destroy: true,
        send_thread_id: None,
        send_timeout_result_ptr: None,
    });
    let writes = [
        uc.reg_write(RegisterMIPS::A0, u64::from(hwnd)),
        uc.reg_write(RegisterMIPS::A1, u64::from(crate::ce::gwe::WM_DESTROY)),
        uc.reg_write(RegisterMIPS::A2, 0),
        uc.reg_write(RegisterMIPS::A3, 0),
        uc.reg_write(RegisterMIPS::RA, u64::from(WNDPROC_RETURN_STUB_ADDR)),
        uc.reg_write(RegisterMIPS::T9, u64::from(wndproc)),
        uc.reg_write(RegisterMIPS::PC, u64::from(wndproc)),
    ];
    if writes.into_iter().all(|write| write.is_ok()) {
        true
    } else {
        let _ = pending_returns.borrow_mut().pop();
        false
    }
}

#[cfg(feature = "unicorn")]
fn try_enter_is_dialog_message_callout<D>(
    kernel: &CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    args: &[u32],
    pending_returns: &std::rc::Rc<std::cell::RefCell<Vec<PendingWndProcReturn>>>,
) -> bool {
    use unicorn_engine::RegisterMIPS;

    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll
        || ordinal != Some(crate::ce::coredll_ordinals::ORD_IS_DIALOG_MESSAGE_W)
    {
        return false;
    }

    let hwnd = args.first().copied().unwrap_or(0);
    let msg_ptr = args.get(1).copied().unwrap_or(0);
    let Some(msg_hwnd) = read_unicorn_u32(uc, msg_ptr) else {
        return false;
    };
    let Some(msg) = read_unicorn_u32(uc, msg_ptr.wrapping_add(4)) else {
        return false;
    };
    if msg != crate::ce::gwe::WM_COMMAND {
        return false;
    }
    let Some(wparam) = read_unicorn_u32(uc, msg_ptr.wrapping_add(8)) else {
        return false;
    };
    let Some(lparam) = read_unicorn_u32(uc, msg_ptr.wrapping_add(12)) else {
        return false;
    };
    let target = if msg_hwnd != 0 { msg_hwnd } else { hwnd };
    let Some(window) = kernel.gwe.window(target) else {
        return false;
    };
    let wndproc = window.wndproc;
    if !is_guest_wndproc(wndproc) {
        return false;
    }
    let return_pc = read_mips_reg(uc, RegisterMIPS::RA);

    tracing::debug!(
        target: "ce.gwe",
        hwnd = format_args!("0x{target:08x}"),
        msg = format_args!("0x{msg:08x}"),
        wparam = format_args!("0x{wparam:08x}"),
        lparam = format_args!("0x{lparam:08x}"),
        class = window.class_name.as_str(),
        wndproc = format_args!("0x{wndproc:08x}"),
        ra = format_args!("0x{return_pc:08x}"),
        "IsDialogMessageW guest dialog proc callout"
    );

    pending_returns.borrow_mut().push(PendingWndProcReturn {
        source: "IsDialogMessageW",
        hwnd: target,
        msg,
        wparam,
        lparam,
        wndproc,
        return_pc,
        class_name: Some(window.class_name.clone()),
        api_result: Some(1),
        dialog_result_hwnd: None,
        finalize_destroy: false,
        send_thread_id: None,
        send_timeout_result_ptr: None,
    });
    let writes = [
        uc.reg_write(RegisterMIPS::A0, u64::from(target)),
        uc.reg_write(RegisterMIPS::A1, u64::from(msg)),
        uc.reg_write(RegisterMIPS::A2, u64::from(wparam)),
        uc.reg_write(RegisterMIPS::A3, u64::from(lparam)),
        uc.reg_write(RegisterMIPS::RA, u64::from(WNDPROC_RETURN_STUB_ADDR)),
        uc.reg_write(RegisterMIPS::T9, u64::from(wndproc)),
        uc.reg_write(RegisterMIPS::PC, u64::from(wndproc)),
    ];
    if writes.into_iter().all(|write| write.is_ok()) {
        true
    } else {
        let _ = pending_returns.borrow_mut().pop();
        false
    }
}

#[cfg(feature = "unicorn")]
fn try_enter_update_window_callout<D>(
    kernel: &CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    args: &[u32],
    pending_returns: &std::rc::Rc<std::cell::RefCell<Vec<PendingWndProcReturn>>>,
) -> bool {
    use unicorn_engine::RegisterMIPS;

    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll
        || ordinal != Some(crate::ce::coredll_ordinals::ORD_UPDATE_WINDOW)
    {
        return false;
    }

    let hwnd = args.first().copied().unwrap_or(0);
    if kernel.gwe.update_rect(hwnd).is_none() {
        return false;
    }
    let Some(window) = kernel.gwe.window(hwnd) else {
        return false;
    };
    let wndproc = window.wndproc;
    if !is_guest_wndproc(wndproc) {
        return false;
    }
    let msg = crate::ce::gwe::WM_PAINT;
    let return_pc = read_mips_reg(uc, RegisterMIPS::RA);

    tracing::debug!(
        target: "ce.gwe",
        hwnd = format_args!("0x{hwnd:08x}"),
        msg = format_args!("0x{msg:08x}"),
        class = window.class_name.as_str(),
        wndproc = format_args!("0x{wndproc:08x}"),
        ra = format_args!("0x{return_pc:08x}"),
        "UpdateWindow guest WM_PAINT callout"
    );

    pending_returns.borrow_mut().push(PendingWndProcReturn {
        source: "UpdateWindow/WM_PAINT",
        hwnd,
        msg,
        wparam: 0,
        lparam: 0,
        wndproc,
        return_pc,
        class_name: Some(window.class_name.clone()),
        api_result: Some(1),
        dialog_result_hwnd: None,
        finalize_destroy: false,
        send_thread_id: None,
        send_timeout_result_ptr: None,
    });
    let writes = [
        uc.reg_write(RegisterMIPS::A0, u64::from(hwnd)),
        uc.reg_write(RegisterMIPS::A1, u64::from(msg)),
        uc.reg_write(RegisterMIPS::A2, 0),
        uc.reg_write(RegisterMIPS::A3, 0),
        uc.reg_write(RegisterMIPS::RA, u64::from(WNDPROC_RETURN_STUB_ADDR)),
        uc.reg_write(RegisterMIPS::T9, u64::from(wndproc)),
        uc.reg_write(RegisterMIPS::PC, u64::from(wndproc)),
    ];
    if writes.into_iter().all(|write| write.is_ok()) {
        true
    } else {
        let _ = pending_returns.borrow_mut().pop();
        false
    }
}

#[cfg(feature = "unicorn")]
fn try_enter_dialog_init_callout<D>(
    kernel: &CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    args: &[u32],
    hwnd: u32,
    pending_returns: &std::rc::Rc<std::cell::RefCell<Vec<PendingWndProcReturn>>>,
) -> bool {
    use unicorn_engine::RegisterMIPS;

    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll {
        return false;
    }
    let is_create =
        ordinal == Some(crate::ce::coredll_ordinals::ORD_CREATE_DIALOG_INDIRECT_PARAM_W);
    let is_modal = ordinal == Some(crate::ce::coredll_ordinals::ORD_DIALOG_BOX_INDIRECT_PARAM_W);
    if !is_create && !is_modal {
        return false;
    }
    if hwnd == 0 {
        return false;
    }
    let Some(window) = kernel.gwe.window(hwnd) else {
        return false;
    };
    let wndproc = window.wndproc;
    if !is_guest_wndproc(wndproc) {
        return false;
    }
    let init_param = args.get(4).copied().unwrap_or(0);
    let msg = WM_INITDIALOG;
    let return_pc = read_mips_reg(uc, RegisterMIPS::RA);
    let source = if is_modal {
        "DialogBoxIndirectParamW/WM_INITDIALOG"
    } else {
        "CreateDialogIndirectParamW/WM_INITDIALOG"
    };

    tracing::debug!(
        target: "ce.gwe",
        hwnd = format_args!("0x{hwnd:08x}"),
        msg = format_args!("0x{msg:08x}"),
        lparam = format_args!("0x{init_param:08x}"),
        class = window.class_name.as_str(),
        wndproc = format_args!("0x{wndproc:08x}"),
        ra = format_args!("0x{return_pc:08x}"),
        "dialog init guest proc callout"
    );

    pending_returns.borrow_mut().push(PendingWndProcReturn {
        source,
        hwnd,
        msg,
        wparam: 0,
        lparam: init_param,
        wndproc,
        return_pc,
        class_name: Some(window.class_name.clone()),
        api_result: is_create.then_some(hwnd),
        dialog_result_hwnd: is_modal.then_some(hwnd),
        finalize_destroy: false,
        send_thread_id: None,
        send_timeout_result_ptr: None,
    });
    let writes = [
        uc.reg_write(RegisterMIPS::A0, u64::from(hwnd)),
        uc.reg_write(RegisterMIPS::A1, u64::from(msg)),
        uc.reg_write(RegisterMIPS::A2, 0),
        uc.reg_write(RegisterMIPS::A3, u64::from(init_param)),
        uc.reg_write(RegisterMIPS::RA, u64::from(WNDPROC_RETURN_STUB_ADDR)),
        uc.reg_write(RegisterMIPS::T9, u64::from(wndproc)),
        uc.reg_write(RegisterMIPS::PC, u64::from(wndproc)),
    ];
    if writes.into_iter().all(|write| write.is_ok()) {
        true
    } else {
        let _ = pending_returns.borrow_mut().pop();
        false
    }
}

#[cfg(feature = "unicorn")]
fn try_enter_create_window_create_callout<D>(
    kernel: &mut CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    args: &[u32],
    hwnd: u32,
    mapped_kernel_memory: &std::rc::Rc<std::cell::RefCell<Vec<(u32, u32)>>>,
    pending_returns: &std::rc::Rc<std::cell::RefCell<Vec<CreateWindowReturn>>>,
) -> bool {
    use unicorn_engine::RegisterMIPS;

    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll
        || ordinal != Some(crate::ce::coredll_ordinals::ORD_CREATE_WINDOW_EX_W)
        || hwnd == 0
    {
        return false;
    }

    let Some(window) = kernel.gwe.window(hwnd) else {
        return false;
    };
    let wndproc = window.wndproc;
    if !is_guest_wndproc(wndproc) {
        return false;
    }
    let class_name = window.class_name.clone();
    let return_pc = read_mips_reg(uc, RegisterMIPS::RA);
    let Some(create_struct) = kernel.memory.heap_alloc(
        crate::ce::memory::PROCESS_HEAP_HANDLE,
        crate::ce::memory::HEAP_ZERO_MEMORY,
        CREATESTRUCTW_SIZE,
    ) else {
        return false;
    };
    if map_kernel_memory_allocations(uc, kernel, &mut mapped_kernel_memory.borrow_mut()).is_err() {
        return false;
    }
    let bytes = create_structw_bytes(args);
    if uc.mem_write(u64::from(create_struct), &bytes).is_err() {
        return false;
    }

    tracing::debug!(
        target: "ce.gwe",
        hwnd = format_args!("0x{hwnd:08x}"),
        msg = format_args!("0x{:08x}", crate::ce::gwe::WM_CREATE),
        lparam = format_args!("0x{create_struct:08x}"),
        class = class_name.as_str(),
        wndproc = format_args!("0x{wndproc:08x}"),
        return_pc = format_args!("0x{return_pc:08x}"),
        "CreateWindowExW guest WM_CREATE callout"
    );

    pending_returns.borrow_mut().push(CreateWindowReturn {
        return_pc,
        hwnd,
        wndproc,
        lparam: create_struct,
        class_name: Some(class_name),
    });
    let writes = [
        uc.reg_write(RegisterMIPS::A0, u64::from(hwnd)),
        uc.reg_write(RegisterMIPS::A1, u64::from(crate::ce::gwe::WM_CREATE)),
        uc.reg_write(RegisterMIPS::A2, 0),
        uc.reg_write(RegisterMIPS::A3, u64::from(create_struct)),
        uc.reg_write(RegisterMIPS::RA, u64::from(CREATE_WINDOW_RETURN_STUB_ADDR)),
        uc.reg_write(RegisterMIPS::T9, u64::from(wndproc)),
        uc.reg_write(RegisterMIPS::PC, u64::from(wndproc)),
    ];
    if writes.into_iter().all(|write| write.is_ok()) {
        true
    } else {
        let _ = pending_returns.borrow_mut().pop();
        false
    }
}

#[cfg(feature = "unicorn")]
fn create_structw_bytes(args: &[u32]) -> [u8; CREATESTRUCTW_SIZE as usize] {
    let fields = [
        args.get(11).copied().unwrap_or(0),
        args.get(10).copied().unwrap_or(0),
        args.get(9).copied().unwrap_or(0),
        args.get(8).copied().unwrap_or(0),
        args.get(7).copied().unwrap_or(0),
        args.get(6).copied().unwrap_or(0),
        args.get(5).copied().unwrap_or(0),
        args.get(4).copied().unwrap_or(0),
        args.get(3).copied().unwrap_or(0),
        args.get(2).copied().unwrap_or(0),
        args.get(1).copied().unwrap_or(0),
        args.first().copied().unwrap_or(0),
    ];
    let mut bytes = [0; CREATESTRUCTW_SIZE as usize];
    for (index, value) in fields.into_iter().enumerate() {
        bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
    }
    bytes
}

#[cfg(feature = "unicorn")]
fn try_enter_call_window_proc_callout<D>(
    kernel: &mut CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    args: &[u32],
    pending_returns: &std::rc::Rc<std::cell::RefCell<Vec<PendingWndProcReturn>>>,
) -> bool {
    use unicorn_engine::RegisterMIPS;

    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll
        || ordinal != Some(crate::ce::coredll_ordinals::ORD_CALL_WINDOW_PROC_W)
    {
        return false;
    }

    let wndproc = args.first().copied().unwrap_or(0);
    if wndproc == 0 {
        return false;
    }
    let hwnd = args.get(1).copied().unwrap_or(0);
    let msg = args.get(2).copied().unwrap_or(0);
    let wparam = args.get(3).copied().unwrap_or(0);
    let lparam = args.get(4).copied().unwrap_or(0);
    if wndproc == crate::ce::gwe::DEFAULT_WNDPROC {
        if msg == crate::ce::gwe::WM_CLOSE {
            return enter_destroy_window_wm_destroy_callout(
                kernel,
                uc,
                hwnd,
                "CallWindowProcW(DEFAULT)/WM_CLOSE",
                0,
                pending_returns,
            );
        }
        let result = crate::ce::gwe::default_send_message_result(msg, wparam, lparam);
        let _ = uc.reg_write(RegisterMIPS::V0, u64::from(result));
        return true;
    }
    let return_pc = read_mips_reg(uc, RegisterMIPS::RA);

    tracing::debug!(
        target: "ce.gwe",
        hwnd = format_args!("0x{hwnd:08x}"),
        msg = format_args!("0x{msg:08x}"),
        wparam = format_args!("0x{wparam:08x}"),
        lparam = format_args!("0x{lparam:08x}"),
        wndproc = format_args!("0x{wndproc:08x}"),
        ra = format_args!("0x{return_pc:08x}"),
        "CallWindowProcW guest wndproc callout"
    );

    pending_returns.borrow_mut().push(PendingWndProcReturn {
        source: "CallWindowProcW",
        hwnd,
        msg,
        wparam,
        lparam,
        wndproc,
        return_pc,
        class_name: None,
        api_result: None,
        dialog_result_hwnd: None,
        finalize_destroy: false,
        send_thread_id: None,
        send_timeout_result_ptr: None,
    });
    let writes = [
        uc.reg_write(RegisterMIPS::A0, u64::from(hwnd)),
        uc.reg_write(RegisterMIPS::A1, u64::from(msg)),
        uc.reg_write(RegisterMIPS::A2, u64::from(wparam)),
        uc.reg_write(RegisterMIPS::A3, u64::from(lparam)),
        uc.reg_write(RegisterMIPS::RA, u64::from(WNDPROC_RETURN_STUB_ADDR)),
        uc.reg_write(RegisterMIPS::T9, u64::from(wndproc)),
        uc.reg_write(RegisterMIPS::PC, u64::from(wndproc)),
    ];
    if writes.into_iter().all(|write| write.is_ok()) {
        true
    } else {
        let _ = pending_returns.borrow_mut().pop();
        false
    }
}

#[cfg(feature = "unicorn")]
fn push_unicorn_last_call<D>(
    last_calls: &std::rc::Rc<std::cell::RefCell<Vec<UnicornLastCall>>>,
    uc: &unicorn_engine::Unicorn<'_, D>,
    pc: u32,
    target: u32,
    kind: &'static str,
) {
    use unicorn_engine::RegisterMIPS;

    let mut last_calls = last_calls.borrow_mut();
    if last_calls.len() == UNICORN_TRACE_LIMIT {
        last_calls.remove(0);
    }
    last_calls.push(UnicornLastCall {
        pc,
        target,
        kind,
        ra: read_mips_reg(uc, RegisterMIPS::RA),
        sp: read_mips_reg(uc, RegisterMIPS::SP),
        a0: read_mips_reg(uc, RegisterMIPS::A0),
        a1: read_mips_reg(uc, RegisterMIPS::A1),
        a2: read_mips_reg(uc, RegisterMIPS::A2),
        a3: read_mips_reg(uc, RegisterMIPS::A3),
    });
}

#[cfg(feature = "unicorn")]
fn capture_debug_snapshot<D>(
    uc: &unicorn_engine::Unicorn<'_, D>,
    traps: &ImportTrapTable,
    memory_fault: Option<UnicornMemoryFault>,
    indirect_call_probe: Option<UnicornIndirectCallProbe>,
    host_wall_clock_stop: Option<UnicornHostWallClockStop>,
    interrupt_probe: Option<UnicornInterruptProbe>,
    invalid_instruction_probe: Option<UnicornInvalidInstructionProbe>,
    last_calls: Vec<UnicornLastCall>,
    last_imports: Vec<UnicornLastImport>,
    last_messages: Vec<UnicornLastMessage>,
    last_wndproc_returns: Vec<UnicornWndProcReturn>,
    last_code: Vec<UnicornLastCode>,
    last_blocks: Vec<UnicornLastBlock>,
    import_counts: Vec<UnicornImportCount>,
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
        indirect_call_probe,
        host_wall_clock_stop,
        interrupt_probe,
        invalid_instruction_probe,
        last_calls,
        last_imports,
        last_messages,
        last_wndproc_returns,
        last_code,
        last_blocks,
        import_counts,
        blocked_get_message,
        thread_exit_reached,
        encoded_kernel_exit: None,
    }
}

fn import_count_snapshot(
    counts: &std::collections::BTreeMap<UnicornImportCountKey, u64>,
) -> Vec<UnicornImportCount> {
    const IMPORT_COUNT_SNAPSHOT_LIMIT: usize = 16;

    let mut counts = counts
        .iter()
        .map(|(key, count)| UnicornImportCount {
            module: key.module.clone(),
            ordinal: key.ordinal,
            name: key.name.clone(),
            count: *count,
        })
        .collect::<Vec<_>>();
    counts.sort_by(|lhs, rhs| {
        rhs.count
            .cmp(&lhs.count)
            .then_with(|| lhs.module.cmp(&rhs.module))
            .then_with(|| lhs.ordinal.cmp(&rhs.ordinal))
            .then_with(|| lhs.name.cmp(&rhs.name))
    });
    counts.truncate(IMPORT_COUNT_SNAPSHOT_LIMIT);
    counts
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
        Ok(())
    }

    fn read_bytes(&self, addr: u32, out: &mut [u8]) -> Result<()> {
        self.uc
            .mem_read(u64::from(addr), out)
            .map_err(|err| Error::Backend(format!("read_bytes 0x{addr:08x}: {err:?}")))?;
        Ok(())
    }

    fn write_bytes(&mut self, addr: u32, bytes: &[u8]) -> Result<()> {
        self.uc
            .mem_write(u64::from(addr), bytes)
            .map_err(|err| Error::Backend(format!("write_bytes 0x{addr:08x}: {err:?}")))?;
        Ok(())
    }

    fn fill_bytes(&mut self, addr: u32, value: u8, len: u32) -> Result<()> {
        const FILL_CHUNK: usize = 4096;

        let chunk = [value; FILL_CHUNK];
        let mut remaining = len as usize;
        let mut cursor = addr;
        while remaining != 0 {
            let count = remaining.min(FILL_CHUNK);
            self.uc
                .mem_write(u64::from(cursor), &chunk[..count])
                .map_err(|err| Error::Backend(format!("fill_bytes 0x{cursor:08x}: {err:?}")))?;
            cursor = cursor.wrapping_add(count as u32);
            remaining -= count;
        }
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
    fn create_structw_bytes_match_ce_sdk_layout() {
        let args = [
            0x0000_00a1,
            0x1000_0001,
            0x1000_0002,
            0x0000_00a4,
            10,
            20,
            300,
            200,
            0x0002_0000,
            0x0000_1234,
            0x0040_0000,
            0x3000_0008,
        ];
        let bytes = super::create_structw_bytes(&args);
        let field = |offset: usize| {
            u32::from_le_bytes(
                bytes[offset..offset + 4]
                    .try_into()
                    .expect("aligned u32 field"),
            )
        };

        assert_eq!(bytes.len(), 48);
        assert_eq!(field(0), 0x3000_0008);
        assert_eq!(field(4), 0x0040_0000);
        assert_eq!(field(8), 0x0000_1234);
        assert_eq!(field(12), 0x0002_0000);
        assert_eq!(field(16), 200);
        assert_eq!(field(20), 300);
        assert_eq!(field(24), 20);
        assert_eq!(field(28), 10);
        assert_eq!(field(32), 0x0000_00a4);
        assert_eq!(field(36), 0x1000_0002);
        assert_eq!(field(40), 0x1000_0001);
        assert_eq!(field(44), 0x0000_00a1);
    }

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

    #[test]
    fn trampoline_scan_skips_halfword_jump_table_data() {
        let mut mapped = vec![0; 0x80];
        write_vec_u32(&mut mapped, 0x10, 0x2c43_0005);
        write_vec_u32(&mut mapped, 0x20, 0x3c07_0000);
        write_vec_u32(&mut mapped, 0x24, 0x24e7_0040);
        write_vec_u32(&mut mapped, 0x28, 0x0002_3040);
        write_vec_u32(&mut mapped, 0x2c, 0x00c7_3021);
        write_vec_u32(&mut mapped, 0x30, 0x84c6_0000);
        write_vec_u32(&mut mapped, 0x34, 0x00e6_3821);
        write_vec_u32(&mut mapped, 0x38, 0x00e0_0008);
        write_vec_u32(&mut mapped, 0x3c, 0x0000_0000);
        let table_entries = [0x000c_u16, 0x29b8, 0x243c, 0x16b0, 0x2154];
        for (index, entry) in table_entries.into_iter().enumerate() {
            let offset = 0x40 + index * 2;
            mapped[offset..offset + 2].copy_from_slice(&entry.to_le_bytes());
        }

        let ranges =
            super::mips_halfword_jump_table_ranges(&mapped, 0, 0, mapped.len() as u32, "test")
                .unwrap();

        assert_eq!(ranges, vec![(0x40, 10)]);
        assert!(!super::mips_patch_rva_overlaps_data_ranges(0x38, &ranges));
        assert!(super::mips_patch_rva_overlaps_data_ranges(0x3c, &ranges));
        assert!(super::mips_patch_rva_overlaps_data_ranges(0x44, &ranges));
        assert!(super::mips_patch_rva_overlaps_data_ranges(0x48, &ranges));
        assert!(!super::mips_patch_rva_overlaps_data_ranges(0x4c, &ranges));
    }

    fn write_u32(uc: &mut Unicorn<'_, ()>, address: u64, value: u32) {
        uc.mem_write(address, &value.to_le_bytes()).unwrap();
    }

    fn write_vec_u32(mapped: &mut [u8], offset: usize, value: u32) {
        mapped[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn write_words(uc: &mut Unicorn<'_, ()>, address: u32, words: &[u32]) {
        for (index, word) in words.iter().enumerate() {
            write_u32(uc, u64::from(address) + index as u64 * 4, *word);
        }
    }
}
