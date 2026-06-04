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
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct UnicornMips {
    memory: MemoryMap,
    entry: Option<u32>,
    entry_image_base: Option<u32>,
    stack_top: Option<u32>,
    mapped_blobs: Vec<MappedBlob>,
    loaded_modules: Vec<LoadedPeModuleInfo>,
    import_traps: ImportTrapTable,
    resource_strings: Vec<MappedResourceString>,
    resources: Vec<MappedResource>,
    #[cfg(feature = "unicorn")]
    trampoline_ranges: Vec<(u32, u32)>,
    #[cfg(feature = "unicorn")]
    trampoline_jumps: Vec<MipsTrampolineJump>,
    last_debug: Option<UnicornDebugSnapshot>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoadedPeModuleInfo {
    pub name: String,
    pub base: u32,
    pub exports_by_name: HashMap<String, u32>,
    pub exports_by_ordinal: HashMap<u32, u32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UnicornRunLimits {
    pub instruction_limit: usize,
    pub wall_clock_limit_ms: u64,
    pub stop_pc: Option<u32>,
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
    pub trap_module_kind: Option<crate::emulator::imports::ImportModuleKind>,
    pub trap_module_name: Option<String>,
    pub trap_name: Option<String>,
    pub trap_ordinal: Option<u32>,
    pub memory_fault: Option<UnicornMemoryFault>,
    pub indirect_call_probe: Option<UnicornIndirectCallProbe>,
    pub host_wall_clock_stop: Option<UnicornHostWallClockStop>,
    pub interrupt_probe: Option<UnicornInterruptProbe>,
    pub invalid_instruction_probe: Option<UnicornInvalidInstructionProbe>,
    pub pc_stop: Option<UnicornPcStop>,
    pub last_calls: Vec<UnicornLastCall>,
    pub last_imports: Vec<UnicornLastImport>,
    pub import_milestones: Vec<UnicornLastImport>,
    pub file_io_stats: crate::ce::file::FileIoStats,
    pub scheduler_stats: crate::ce::scheduler::SchedulerStats,
    pub gwe_stats: crate::ce::gwe::GweStats,
    pub recent_file_open_ops: Vec<crate::ce::kernel::FileTraceRecord>,
    pub recent_file_ops: Vec<crate::ce::kernel::FileTraceRecord>,
    pub last_messages: Vec<UnicornLastMessage>,
    pub last_wndproc_returns: Vec<UnicornWndProcReturn>,
    pub last_wndproc_call_traces: Vec<UnicornWndProcCallTrace>,
    pub last_mfc_dispatch: Vec<UnicornMfcDispatchTrace>,
    pub last_inavi_display: Vec<UnicornInaviDisplayTrace>,
    pub last_inavi_controller: Vec<UnicornInaviControllerTrace>,
    pub inavi_render_milestones: Vec<UnicornInaviControllerTrace>,
    pub last_code: Vec<UnicornLastCode>,
    pub last_blocks: Vec<UnicornLastBlock>,
    pub import_counts: Vec<UnicornImportCount>,
    pub heap_allocation_count: usize,
    pub heap_allocation_bytes: u64,
    pub virtual_allocation_count: usize,
    pub virtual_allocation_bytes: u64,
    pub blocked_get_message: Option<UnicornBlockedGetMessage>,
    pub thread_exit_reached: bool,
    pub encoded_kernel_exit: Option<EncodedKernelExit>,
}

impl UnicornDebugSnapshot {
    pub fn summary(&self) -> String {
        let mut parts = vec![
            format!("pc=0x{:08x}", self.pc),
            format!("ra=0x{:08x}", self.ra),
            format!("sp=0x{:08x}", self.sp),
            format!("v0=0x{:08x}", self.v0),
            format!("a0=0x{:08x}", self.a0),
            format!("a1=0x{:08x}", self.a1),
        ];
        if let Some(trap_address) = self.trap_address {
            let mut trap = format!("trap=0x{trap_address:08x}");
            if let Some(module) = self.trap_module_name.as_deref() {
                trap.push_str(&format!("/{module}"));
            }
            if let Some(ordinal) = self.trap_ordinal {
                trap.push_str(&format!("@{ordinal}"));
            }
            if let Some(name) = self.trap_name.as_deref() {
                trap.push_str(&format!("/{name}"));
            }
            parts.push(trap);
        }
        if let Some(stop) = &self.host_wall_clock_stop {
            parts.push(format!("wall_stop={}ms", stop.elapsed_ms));
        }
        if self.heap_allocation_count != 0 || self.heap_allocation_bytes != 0 {
            parts.push(format!(
                "heap_live={}/{}B",
                self.heap_allocation_count, self.heap_allocation_bytes
            ));
        }
        if self.virtual_allocation_count != 0 || self.virtual_allocation_bytes != 0 {
            parts.push(format!(
                "virtual_live={}/{}B",
                self.virtual_allocation_count, self.virtual_allocation_bytes
            ));
        }
        if self.file_io_stats.host_file_open_count != 0
            || self.file_io_stats.host_file_read_count != 0
            || self.file_io_stats.memory_backed_open_count != 0
        {
            parts.push(format!(
                "file_io=host_open:{} host_read:{}/{}B mem_open:{} max_read:{}",
                self.file_io_stats.host_file_open_count,
                self.file_io_stats.host_file_read_count,
                self.file_io_stats.host_file_read_bytes,
                self.file_io_stats.memory_backed_open_count,
                self.file_io_stats.max_read_request
            ));
        }
        if self.scheduler_stats.wait_single_count != 0
            || self.scheduler_stats.wait_multiple_count != 0
            || self.scheduler_stats.msg_wait_count != 0
            || self.scheduler_stats.sleep_count != 0
            || self.scheduler_stats.yield_count != 0
            || self.scheduler_stats.object_signal_count != 0
            || self.scheduler_stats.message_input_signal_count != 0
            || self.scheduler_stats.serial_read_signal_count != 0
            || self.scheduler_stats.send_reply_signal_count != 0
        {
            parts.push(format!(
                "sched=wait:{}/{}/{} sleep:{} yield:{} ok:{} timeout:{} fail:{} block:{} wake:{} reg:{}/{} maxreg:{} sig:{} cand:{} msgsig:{} msgcand:{} sersig:{} sercand:{} sendsig:{} sendcand:{} maxpend:{}",
                self.scheduler_stats.wait_single_count,
                self.scheduler_stats.wait_multiple_count,
                self.scheduler_stats.msg_wait_count,
                self.scheduler_stats.sleep_count,
                self.scheduler_stats.yield_count,
                self.scheduler_stats.wait_acquired_count,
                self.scheduler_stats.wait_timeout_count,
                self.scheduler_stats.wait_failed_count,
                self.scheduler_stats.wait_block_count,
                self.scheduler_stats.wait_wake_count,
                self.scheduler_stats.waiter_register_count,
                self.scheduler_stats.waiter_remove_count,
                self.scheduler_stats.max_registered_waits,
                self.scheduler_stats.object_signal_count,
                self.scheduler_stats.object_wake_candidate_count,
                self.scheduler_stats.message_input_signal_count,
                self.scheduler_stats.message_input_wake_candidate_count,
                self.scheduler_stats.serial_read_signal_count,
                self.scheduler_stats.serial_read_wake_candidate_count,
                self.scheduler_stats.send_reply_signal_count,
                self.scheduler_stats.send_reply_wake_candidate_count,
                self.scheduler_stats.max_pending_wakes
            ));
        }
        if self.gwe_stats.send_transaction_count != 0 {
            parts.push(format!(
                "gwe=send:{} done:{} timeout:{} dead:{} maxq:{}",
                self.gwe_stats.send_transaction_count,
                self.gwe_stats.send_transaction_completed_count,
                self.gwe_stats.send_transaction_timeout_count,
                self.gwe_stats.send_transaction_receiver_terminated_count,
                self.gwe_stats.max_sent_queue_depth
            ));
        }
        if let Some(indirect) = &self.indirect_call_probe {
            parts.push(format!(
                "indirect={} pc=0x{:08x} target=0x{:08x}",
                indirect.register_name, indirect.pc, indirect.target
            ));
        }
        if let Some(blocked) = &self.blocked_get_message {
            parts.push(format!(
                "blocked_get_message=thread:{} hwnd={}",
                blocked.thread_id,
                blocked
                    .hwnd
                    .map(|hwnd| format!("0x{hwnd:08x}"))
                    .unwrap_or_else(|| "any".to_owned())
            ));
        }
        if let Some(exit) = &self.encoded_kernel_exit {
            parts.push(format!(
                "encoded_exit=api{}.{} process=0x{:08x} code=0x{:08x}",
                exit.api_set, exit.method, exit.process, exit.exit_code
            ));
        }
        if let Some(interrupt) = &self.interrupt_probe {
            parts.push(format!(
                "interrupt={} last_pc={}",
                interrupt.intno,
                interrupt
                    .last_code_pc
                    .map(|pc| format!("0x{pc:08x}"))
                    .unwrap_or_else(|| "none".to_owned())
            ));
        }
        if let Some(invalid) = &self.invalid_instruction_probe {
            parts.push(format!(
                "invalid_instruction pc=0x{:08x} insn={}",
                invalid.pc,
                invalid
                    .instruction
                    .map(|insn| format!("0x{insn:08x}"))
                    .unwrap_or_else(|| "unreadable".to_owned())
            ));
        }
        if let Some(stop) = &self.pc_stop {
            parts.push(format!("pc_stop=0x{:08x}", stop.pc));
        }
        parts.join(" ")
    }
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
pub struct UnicornPcStop {
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
    pub target_module_kind: Option<crate::emulator::imports::ImportModuleKind>,
    pub target_module_name: Option<String>,
    pub target_name: Option<String>,
    pub target_ordinal: Option<u32>,
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
    pub ra: u32,
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
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornImportCount {
    pub module: String,
    pub ordinal: Option<u32>,
    pub name: Option<String>,
    pub count: u64,
    pub max_a0: u32,
    pub max_a1: u32,
    pub max_a2: u32,
    pub max_a3: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct UnicornImportCountKey {
    module: String,
    ordinal: Option<u32>,
    name: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct UnicornImportStats {
    count: u64,
    max_a0: u32,
    max_a1: u32,
    max_a2: u32,
    max_a3: u32,
}

impl UnicornImportStats {
    fn record(&mut self, a0: u32, a1: u32, a2: u32, a3: u32) {
        self.count = self.count.saturating_add(1);
        self.max_a0 = self.max_a0.max(a0);
        self.max_a1 = self.max_a1.max(a1);
        self.max_a2 = self.max_a2.max(a2);
        self.max_a3 = self.max_a3.max(a3);
    }
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
    pub return_pc_trampoline_origin: Option<u32>,
    pub result: u32,
    pub class_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornWndProcCallTrace {
    pub source: &'static str,
    pub hwnd: u32,
    pub msg: u32,
    pub wparam: u32,
    pub lparam: u32,
    pub wndproc: u32,
    pub return_pc: u32,
    pub return_pc_trampoline_origin: Option<u32>,
    pub result: u32,
    pub class_name: Option<String>,
    pub calls: Vec<UnicornLastCall>,
    pub imports: Vec<UnicornLastImport>,
    pub code: Vec<UnicornLastCode>,
    pub readiness_code: Vec<UnicornLastCode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornMfcDispatchTrace {
    pub pc: u32,
    pub label: &'static str,
    pub ra: u32,
    pub sp: u32,
    pub v0: u32,
    pub a0: u32,
    pub a1: u32,
    pub a2: u32,
    pub a3: u32,
    pub s5: u32,
    pub s6: u32,
    pub s7: u32,
    pub fp: u32,
    pub sp10: Option<u32>,
    pub this_ptr: Option<u32>,
    pub hwnd: Option<u32>,
    pub msg: Option<u32>,
    pub wparam: Option<u32>,
    pub lparam: Option<u32>,
    pub vtable_slot_98: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornInaviDisplayTrace {
    pub pc: u32,
    pub label: &'static str,
    pub ra: u32,
    pub sp: u32,
    pub v0: u32,
    pub a0: u32,
    pub a1: u32,
    pub a2: u32,
    pub a3: u32,
    pub this_ptr: Option<u32>,
    pub hwnd: Option<u32>,
    pub msg: Option<u32>,
    pub wparam: Option<u32>,
    pub lparam: Option<u32>,
    pub field_20: Option<u32>,
    pub field_44: Option<u32>,
    pub field_e8: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornInaviControllerTrace {
    pub pc: u32,
    pub label: &'static str,
    pub instruction: Option<u32>,
    pub ra: u32,
    pub sp: u32,
    pub v0: u32,
    pub a0: u32,
    pub a1: u32,
    pub a2: u32,
    pub a3: u32,
    pub s0: u32,
    pub s2: u32,
    pub s3: u32,
    pub s4: u32,
    pub s5: u32,
    pub s6: u32,
    pub s7: u32,
    pub fp: u32,
    pub sp10: Option<u32>,
    pub sp48: Option<u32>,
    pub controller: Option<u32>,
    pub hwnd: Option<u32>,
    pub msg: Option<u32>,
    pub wparam: Option<u32>,
    pub lparam: Option<u32>,
    pub classifier: Option<u32>,
    pub selected_obj: Option<u32>,
    pub selected_vtable: Option<u32>,
    pub selected_target: Option<u32>,
    pub paint_base: Option<u32>,
    pub paint_gate: Option<u32>,
    pub paint_render_obj: Option<u32>,
    pub paint_render_target: Option<u32>,
    pub render_surface: Option<u32>,
    pub render_enabled: Option<u32>,
    pub render_size_target: Option<u32>,
    pub render_resize_target: Option<u32>,
    pub render_flush_obj: Option<u32>,
    pub render_flush_target: Option<u32>,
    pub render_poll_result: Option<u32>,
    pub render_dim_ptr: Option<u32>,
    pub render_dim_w: Option<u32>,
    pub render_dim_h: Option<u32>,
    pub aux_base: Option<u32>,
    pub aux_slot_10ec_value: Option<u32>,
    pub aux_slot_10f0: Option<u32>,
    pub aux_slot_10f0_vtable: Option<u32>,
    pub aux_inline_10f8: Option<u32>,
    pub aux_inline_10f8_vtable: Option<u32>,
    pub aux_link_ee4: Option<u32>,
    pub aux_init_flag_edc: Option<u32>,
    pub aux_vtable_source: Option<u32>,
    pub aux_vtable_source_value: Option<u32>,
    pub aux_store_addr: Option<u32>,
    pub aux_store_value: Option<u32>,
    #[cfg(feature = "trace")]
    pub query_thunk_slot: Option<u32>,
    #[cfg(feature = "trace")]
    pub query_thunk_target: Option<u32>,
    #[cfg(feature = "trace")]
    pub resource_text: Option<String>,
    #[cfg(feature = "trace")]
    pub resource_format_text: Option<String>,
    #[cfg(feature = "trace")]
    pub resource_aux_text: Option<String>,
    #[cfg(feature = "trace")]
    pub resource_arg_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornLastCode {
    pub pc: u32,
    pub ra: u32,
    pub sp: u32,
    pub v0: u32,
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
    pub queue_status: u32,
    pub next_timer_due_ms: Option<u32>,
    pub timers: Vec<UnicornTimerSnapshot>,
    pub z_order: Vec<u32>,
    pub windows: Vec<UnicornWindowSnapshot>,
    pub queues: Vec<UnicornQueueSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornTimerSnapshot {
    pub id: u32,
    pub hwnd: Option<u32>,
    pub message: u32,
    pub due_ms: u32,
    pub period_ms: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornWindowSnapshot {
    pub hwnd: u32,
    pub thread_id: u32,
    pub process_id: u32,
    pub class_name: String,
    pub title: String,
    pub visible: bool,
    pub destroyed: bool,
    pub update_pending: bool,
    pub erase_pending: bool,
    pub rect: crate::ce::gwe::Rect,
    pub wndproc: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornQueueSnapshot {
    pub thread_id: u32,
    pub messages: Vec<UnicornMessageRecord>,
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
const GUEST_HEAP_SPILLOVER_GRANULARITY: u32 = 0x0010_0000;
#[cfg(feature = "unicorn")]
const UNICORN_TRACE_LIMIT: usize = 256;
#[cfg(feature = "unicorn")]
const UNICORN_TB_CACHE_FLUSH_INTERVAL: u32 = 0x0004_0000;
#[cfg(all(feature = "unicorn", feature = "trace"))]
const UNICORN_IMPORT_MILESTONE_LIMIT: usize = 1024;
#[cfg(feature = "unicorn")]
const UNICORN_WNDPROC_TRACE_LIMIT: usize = 32;
#[cfg(feature = "unicorn")]
const UNICORN_WNDPROC_TRACE_CALL_LIMIT: usize = 96;
#[cfg(feature = "unicorn")]
const UNICORN_WNDPROC_TRACE_IMPORT_LIMIT: usize = 64;
#[cfg(feature = "unicorn")]
const UNICORN_WNDPROC_TRACE_CODE_LIMIT: usize = 256;
#[cfg(feature = "unicorn")]
const UNICORN_WNDPROC_READINESS_TRACE_LIMIT: usize = 64;
#[cfg(all(feature = "unicorn", feature = "trace"))]
const UNICORN_MFC_DISPATCH_TRACE_LIMIT: usize = 64;
#[cfg(all(feature = "unicorn", feature = "trace"))]
const UNICORN_INAVI_DISPLAY_TRACE_LIMIT: usize = 96;
#[cfg(all(feature = "unicorn", feature = "trace"))]
const UNICORN_INAVI_CONTROLLER_TRACE_LIMIT: usize = 128;
#[cfg(all(feature = "unicorn", feature = "trace"))]
const UNICORN_INAVI_RENDER_MILESTONE_LIMIT: usize = 4096;
#[cfg(feature = "unicorn")]
const UNICORN_CODE_TRACE_SAMPLE_INTERVAL: u32 = 64;
#[cfg(feature = "unicorn")]
const UNICORN_BLOCK_TRACE_SAMPLE_INTERVAL: u32 = 16;
#[cfg(feature = "unicorn")]
const MIPS_JUMP_TABLE_SELECTOR_SEARCH_BACK: u32 = 512;
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
struct SendMessageRestoreContext {
    sender_thread_id: u32,
    receiver_thread_id: u32,
    send_id: u64,
    wait_id: u64,
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
    destroy_root_hwnd: Option<u32>,
    remaining_destroy_callouts: Vec<DestroyWndProcCallout>,
    send_thread_id: Option<u32>,
    send_timeout_result_ptr: Option<u32>,
    send_restore: Option<SendMessageRestoreContext>,
}

#[cfg(feature = "unicorn")]
#[derive(Debug, Clone, PartialEq, Eq)]
struct DestroyWndProcCallout {
    hwnd: u32,
    wndproc: u32,
    class_name: Option<String>,
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
    thread_handle: Option<u32>,
    regs: [u32; 32],
    pc: u32,
}

#[cfg(feature = "unicorn")]
#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockedGuestThread {
    wait_id: u64,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockedWaitKind {
    Kernel,
    Sleep,
    MsgWait {
        wake_mask: u32,
        input_available: bool,
    },
    SerialRead {
        handle: u32,
        buffer: u32,
        requested: u32,
        transferred_ptr: u32,
    },
    SendMessage {
        send_id: u64,
        receiver_thread_id: u32,
        result_ptr: Option<u32>,
        previous_running_thread: Option<(u32, u32)>,
    },
}

#[cfg(feature = "unicorn")]
#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockedWaitThread {
    wait_id: u64,
    thread_id: u32,
    thread_handle: u32,
    wait_handles: Vec<u32>,
    kind: BlockedWaitKind,
    wait_started_ms: u32,
    timeout_ms: u32,
    regs: [u32; 32],
    return_pc: u32,
}

#[cfg(feature = "unicorn")]
fn scheduler_blocked_wait_kind(
    kind: BlockedWaitKind,
) -> crate::ce::scheduler::SchedulerBlockedWaitKind {
    match kind {
        BlockedWaitKind::Kernel => crate::ce::scheduler::SchedulerBlockedWaitKind::Kernel,
        BlockedWaitKind::Sleep => crate::ce::scheduler::SchedulerBlockedWaitKind::Sleep,
        BlockedWaitKind::MsgWait {
            wake_mask,
            input_available,
        } => crate::ce::scheduler::SchedulerBlockedWaitKind::MsgWait {
            wake_mask,
            input_available,
        },
        BlockedWaitKind::SerialRead { handle, .. } => {
            crate::ce::scheduler::SchedulerBlockedWaitKind::SerialRead { handle }
        }
        BlockedWaitKind::SendMessage { send_id, .. } => {
            crate::ce::scheduler::SchedulerBlockedWaitKind::SendMessage { send_id }
        }
    }
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
    name: String,
    base: u32,
    bytes: Vec<u8>,
}

#[cfg(feature = "unicorn")]
#[derive(Debug, Clone)]
struct MappedCodeIndex {
    blobs: Vec<MappedBlob>,
    page_to_blob: HashMap<u32, usize>,
}

#[cfg(feature = "unicorn")]
impl MappedCodeIndex {
    fn new(blobs: Vec<MappedBlob>) -> Self {
        let mut page_to_blob = HashMap::new();
        for (index, blob) in blobs.iter().enumerate() {
            if blob.bytes.is_empty() {
                continue;
            }
            let first_page = blob.base >> 12;
            let last_page = blob
                .base
                .saturating_add(blob.bytes.len().saturating_sub(1) as u32)
                >> 12;
            for page in first_page..=last_page {
                page_to_blob.entry(page).or_insert(index);
            }
        }
        Self {
            blobs,
            page_to_blob,
        }
    }

    fn read_u32(&self, address: u32) -> Option<u32> {
        let blob = self.blobs.get(*self.page_to_blob.get(&(address >> 12))?)?;
        let offset = address.checked_sub(blob.base)? as usize;
        let end = offset.checked_add(4)?;
        if end <= blob.bytes.len() {
            Some(u32::from_le_bytes(blob.bytes[offset..end].try_into().ok()?))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornMappedBlobRange {
    pub name: String,
    pub base: u32,
    pub size: u32,
}

impl UnicornMips {
    pub fn new() -> Result<Self> {
        Ok(Self {
            memory: MemoryMap::default(),
            entry: None,
            entry_image_base: None,
            stack_top: None,
            mapped_blobs: Vec::new(),
            loaded_modules: Vec::new(),
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

    pub fn loaded_modules(&self) -> &[LoadedPeModuleInfo] {
        &self.loaded_modules
    }

    pub fn last_debug_snapshot(&self) -> Option<&UnicornDebugSnapshot> {
        self.last_debug.as_ref()
    }

    pub fn mapped_blob_ranges(&self) -> Vec<UnicornMappedBlobRange> {
        self.mapped_blobs
            .iter()
            .map(|blob| UnicornMappedBlobRange {
                name: blob.name.clone(),
                base: blob.base,
                size: blob.bytes.len() as u32,
            })
            .collect()
    }

    pub fn read_mapped_bytes(&self, address: u32, len: usize) -> Option<Vec<u8>> {
        for blob in &self.mapped_blobs {
            let Some(offset) = address.checked_sub(blob.base).map(|offset| offset as usize) else {
                continue;
            };
            let end = offset.checked_add(len)?;
            if end <= blob.bytes.len() {
                return Some(blob.bytes[offset..end].to_vec());
            }
        }
        None
    }

    pub fn load_pe_image(&mut self, image: &PeImage) -> Result<()> {
        self.load_pe_image_with_dlls(image, &[])
    }

    pub fn load_pe_image_with_dlls(&mut self, image: &PeImage, dlls: &[PeImage]) -> Result<()> {
        let mut external = ExternalImportTable::default();
        let mut loaded_dlls = Vec::new();
        #[cfg(feature = "unicorn")]
        let mut trampoline_blobs: Vec<(String, u32, Vec<u8>)> = Vec::new();
        let mut next_dll_base = 0x6000_0000u32;
        #[cfg(feature = "unicorn")]
        let mut next_trampoline_base = 0x5000_0000u32;
        let mut next_trap_base = IMPORT_TRAP_BASE;
        let mut occupied_image_ranges = vec![(
            image.image_base(),
            align_up_4k(image.optional_header.size_of_image)?,
        )];
        self.loaded_modules.clear();

        for dll in dlls {
            let dll_size = align_up_4k(dll.optional_header.size_of_image)?;
            let mut load_base = choose_dll_load_base(
                dll.image_base(),
                dll_size,
                &occupied_image_ranges,
                &mut next_dll_base,
            )?;
            let (mapped, traps, trampoline_patch, mapped_size) = loop {
                let mut mapped = dll.mapped_image_at(load_base)?;
                let traps = patch_pe_coredll_imports(
                    dll,
                    &mut mapped,
                    &crate::ce::coredll::CoredllExportTable::default(),
                    next_trap_base,
                )?;
                #[cfg(feature = "unicorn")]
                let trampoline_patch = {
                    let trampoline_base = allocate_relocated_dll_base(
                        0x0010_0000,
                        &occupied_image_ranges,
                        &mut next_trampoline_base,
                    )?;
                    Some(patch_mips_unicorn_trampolines(
                        dll,
                        load_base,
                        &mut mapped,
                        Some(trampoline_base),
                    )?)
                };
                #[cfg(not(feature = "unicorn"))]
                let trampoline_patch: Option<MipsTrampolinePatchResult> = None;
                let mapped_size = align_up_4k(mapped.len() as u32)?;
                if !range_overlaps_any(load_base, mapped_size, &occupied_image_ranges) {
                    break (mapped, traps, trampoline_patch, mapped_size);
                }
                load_base = allocate_relocated_dll_base(
                    mapped_size,
                    &occupied_image_ranges,
                    &mut next_dll_base,
                )?;
            };
            occupied_image_ranges.push((load_base, mapped_size));
            next_trap_base = advance_trap_base(next_trap_base, traps.len())?;
            self.import_traps.merge(traps);
            external.add_pe_image(module_file_name(&dll.path), dll, load_base);
            #[cfg(not(feature = "unicorn"))]
            let _ = trampoline_patch;
            #[cfg(feature = "unicorn")]
            if let Some(mut trampoline_patch) = trampoline_patch {
                if let Some(range) = trampoline_patch.range {
                    self.trampoline_ranges.push(range);
                    if let Some(bytes) = trampoline_patch.external_mapped.take() {
                        occupied_image_ranges.push((range.0, range.1));
                        trampoline_blobs.push((format!("trampoline:{}", dll.path), range.0, bytes));
                    }
                }
                self.trampoline_jumps.extend(trampoline_patch.jumps);
            }
            self.loaded_modules.push(loaded_module_info(dll, load_base));
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
                patch_mips_unicorn_trampolines(image, image.image_base(), &mut mapped, None)?;
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
        #[cfg(feature = "unicorn")]
        for (name, base, mapped) in &trampoline_blobs {
            self.map_region(
                *base,
                align_up_4k(mapped.len() as u32)?,
                MemoryPerms::READ | MemoryPerms::EXEC,
                name,
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
            name: format!("image:{}", image.path),
            base: image.image_base(),
            bytes: mapped,
        });
        for (path, load_base, mapped) in loaded_dlls {
            if let Some(dll) = dlls.iter().find(|dll| dll.path == path) {
                self.register_image_resource_strings(dll, load_base)?;
            }
            self.mapped_blobs.push(MappedBlob {
                name: format!("dll:{path}"),
                base: load_base,
                bytes: mapped,
            });
        }
        #[cfg(feature = "unicorn")]
        for (name, base, mapped) in trampoline_blobs {
            self.mapped_blobs.push(MappedBlob {
                name,
                base,
                bytes: mapped,
            });
        }
        self.mapped_blobs.push(MappedBlob {
            name: "user-kdata".to_owned(),
            base: USER_KDATA_PAGE_BASE,
            bytes: user_kdata_page(),
        });
        let trap_page = import_trap_code_page(&self.import_traps);
        self.mapped_blobs.push(MappedBlob {
            name: "ce-import-traps".to_owned(),
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
                stop_pc: None,
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
        update_user_kdata_current_ids(&mut uc, 1, kernel.current_process_id())?;

        let indirect_call_probe = Rc::new(RefCell::new(None));
        let indirect_call_probe_hook = Rc::clone(&indirect_call_probe);
        let last_code = Rc::new(RefCell::new(Vec::<UnicornLastCode>::new()));
        let last_code_hook = Rc::clone(&last_code);
        let last_readiness_code = Rc::new(RefCell::new(Vec::<UnicornLastCode>::new()));
        let last_readiness_code_hook = Rc::clone(&last_readiness_code);
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
        let progress_file =
            std::env::var_os("WINCE_EMU_PROGRESS_FILE").map(std::path::PathBuf::from);
        let progress_interval_ms = std::env::var("WINCE_EMU_PROGRESS_INTERVAL_MS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value != 0)
            .unwrap_or(5000);
        let progress_last_ms = Rc::new(RefCell::new(None::<u64>));
        let progress_last_ms_hook = Rc::clone(&progress_last_ms);
        let full_trace_enabled = std::env::var_os("WINCE_EMU_FULL_TRACE").is_some();
        let fast_start_enabled = std::env::var_os("WINCE_EMU_FAST_START").is_some();
        let stop_pc = limits.stop_pc;
        let pc_stop = Rc::new(RefCell::new(None));
        let pc_stop_hook = Rc::clone(&pc_stop);
        let last_calls = Rc::new(RefCell::new(Vec::<UnicornLastCall>::new()));
        let last_calls_hook = Rc::clone(&last_calls);
        let last_mfc_dispatch = Rc::new(RefCell::new(Vec::<UnicornMfcDispatchTrace>::new()));
        #[cfg(feature = "trace")]
        let last_mfc_dispatch_hook = Rc::clone(&last_mfc_dispatch);
        let last_inavi_display = Rc::new(RefCell::new(Vec::<UnicornInaviDisplayTrace>::new()));
        #[cfg(feature = "trace")]
        let last_inavi_display_hook = Rc::clone(&last_inavi_display);
        let last_inavi_controller =
            Rc::new(RefCell::new(Vec::<UnicornInaviControllerTrace>::new()));
        #[cfg(feature = "trace")]
        let last_inavi_controller_hook = Rc::clone(&last_inavi_controller);
        let inavi_render_milestones =
            Rc::new(RefCell::new(Vec::<UnicornInaviControllerTrace>::new()));
        #[cfg(feature = "trace")]
        let inavi_render_milestones_hook = Rc::clone(&inavi_render_milestones);
        let late_inavi_init_dialog_posted = Rc::new(Cell::new(false));
        let late_inavi_init_dialog_posted_hook = Rc::clone(&late_inavi_init_dialog_posted);
        let mapped_code = MappedCodeIndex::new(self.mapped_blobs.clone());
        let trampoline_ranges = self.trampoline_ranges.clone();
        let trampoline_jumps = self.trampoline_jumps.clone();
        let trampoline_pages = trampoline_pages_for_ranges(&trampoline_ranges);
        let trampoline_stub_by_origin = trampoline_stub_by_origin(&trampoline_jumps);
        let trampoline_origin_by_stub = trampoline_origin_by_stub(&trampoline_jumps);
        let kernel_ptr = kernel as *mut CeKernel;
        if !fast_start_enabled {
            uc.add_code_hook(1, 0, move |uc, address, _size| {
            let pc = address as u32;
            let code_trace_index = code_trace_counter_hook.get().wrapping_add(1);
            code_trace_counter_hook.set(code_trace_index);
            if code_trace_index % UNICORN_TB_CACHE_FLUSH_INTERVAL == 0 {
                let _ = uc.ctl_remove_cache(0, u64::MAX);
            }
            let sampled_code_trace = code_trace_index % UNICORN_CODE_TRACE_SAMPLE_INTERVAL == 0;
            let instruction = read_unicorn_code_u32(uc, &mapped_code, pc);
            *last_code_probe_hook.borrow_mut() = Some((pc, instruction));
            if stop_pc == Some(pc) {
                *pc_stop_hook.borrow_mut() = Some(UnicornPcStop {
                    pc,
                    ra: read_mips_reg(uc, RegisterMIPS::RA),
                    sp: read_mips_reg(uc, RegisterMIPS::SP),
                    instruction,
                });
                let _ = uc.emu_stop();
                return;
            }
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
            let direct_jump_target_in_trampoline = direct_jump_target
                .is_some_and(|target| target_in_trampoline_pages(target, &trampoline_pages));
            if let (Some(instruction), Some(target)) = (instruction, direct_jump_target) {
                if full_trace_enabled && instruction >> 26 == 0x03 {
                    push_unicorn_last_call(&last_calls_hook, uc, pc, target, "jal");
                }
            }
            if let Some((register, target)) = instruction
                .and_then(decode_jalr_register)
                .and_then(|register| read_mips_gpr(uc, register).map(|target| (register, target)))
            {
                if full_trace_enabled {
                    let kind = mips_gpr_name(register);
                    push_unicorn_last_call(&last_calls_hook, uc, pc, target, kind);
                }
            }
            let sentinel_target = instruction.and_then(|instruction| {
                if !is_trampoline_sentinel_first_word(instruction) {
                    return None;
                }
                let next_instruction =
                    read_unicorn_code_u32(uc, &mapped_code, pc.wrapping_add(4))?;
                decode_trampoline_sentinel_target(instruction, next_instruction)
            });
            if let Some((register, target)) = instruction
                .and_then(decode_jr_register)
                .and_then(|register| read_mips_gpr(uc, register).map(|target| (register, target)))
            {
                if let Some(stub) = trampoline_stub_by_origin.get(&target).copied() {
                    let _ = write_mips_gpr(uc, register, stub);
                }
            }
            if let Some(target) = sentinel_target {
                if target_in_trampoline_pages(target, &trampoline_pages) {
                    let _ = uc.reg_write(RegisterMIPS::PC, u64::from(target));
                    return;
                }
            }
            let direct_jump_trampoline_origin = direct_jump_target
                .filter(|_| direct_jump_target_in_trampoline)
                .and_then(|target| trampoline_origin_by_stub.get(&target).copied());
            let current_trampoline_origin = ((full_trace_enabled || progress_file.is_some())
                && sampled_code_trace
                && target_in_ranges(pc, &trampoline_ranges))
            .then(|| {
                trampoline_jumps.iter().find_map(|trampoline| {
                    trampoline
                        .stub
                        .checked_add(trampoline.byte_len)
                        .filter(|end| pc >= trampoline.stub && pc < *end)
                        .map(|_| trampoline.origin)
                })
            })
            .flatten();
            if let Some(path) = progress_file.as_ref() {
                let elapsed_ms = host_wall_clock_started.elapsed().as_millis() as u64;
                let mut last_ms = progress_last_ms_hook.borrow_mut();
                if last_ms
                    .map(|last| elapsed_ms.saturating_sub(last) >= progress_interval_ms)
                    .unwrap_or(true)
                {
                    *last_ms = Some(elapsed_ms);
                    let progress = format!(
                        "elapsed_ms={elapsed_ms} hooks={code_trace_index} pc=0x{pc:08x} ra=0x{:08x} sp=0x{:08x} v0=0x{:08x} a0=0x{:08x} a1=0x{:08x} a2=0x{:08x} a3=0x{:08x} insn={} current_trampoline_origin={} direct_jump_target={}\n",
                        read_mips_reg(uc, RegisterMIPS::RA),
                        read_mips_reg(uc, RegisterMIPS::SP),
                        read_mips_reg(uc, RegisterMIPS::V0),
                        read_mips_reg(uc, RegisterMIPS::A0),
                        read_mips_reg(uc, RegisterMIPS::A1),
                        read_mips_reg(uc, RegisterMIPS::A2),
                        read_mips_reg(uc, RegisterMIPS::A3),
                        instruction
                            .map(|value| format!("0x{value:08x}"))
                            .unwrap_or_else(|| "none".to_owned()),
                        current_trampoline_origin
                            .map(|value| format!("0x{value:08x}"))
                            .unwrap_or_else(|| "none".to_owned()),
                        direct_jump_target
                            .map(|value| format!("0x{value:08x}"))
                            .unwrap_or_else(|| "none".to_owned())
                    );
                    let _ = std::fs::write(path, progress);
                }
            }
            #[cfg(feature = "trace")]
            {
                if full_trace_enabled {
                    record_mfc_dispatch_trace(&last_mfc_dispatch_hook, uc, pc);
                    record_inavi_display_trace(&last_inavi_display_hook, uc, pc);
                }
            }
            maybe_post_late_inavi_init_dialog(
                unsafe { &mut *kernel_ptr },
                uc,
                pc,
                &late_inavi_init_dialog_posted_hook,
            );
            repair_inavi_aux_touch_alias(uc, pc);
            #[cfg(feature = "trace")]
            {
                if full_trace_enabled {
                    record_inavi_controller_trace(
                        &last_inavi_controller_hook,
                        &inavi_render_milestones_hook,
                        uc,
                        pc,
                    );
                }
            }
            let code_record = || UnicornLastCode {
                pc,
                ra: read_mips_reg(uc, RegisterMIPS::RA),
                sp: read_mips_reg(uc, RegisterMIPS::SP),
                v0: read_mips_reg(uc, RegisterMIPS::V0),
                sp_return_slot: read_mips_reg(uc, RegisterMIPS::SP)
                    .checked_add(0x10)
                    .and_then(|addr| read_unicorn_u32(uc, addr)),
                instruction,
                next_instruction: read_unicorn_code_u32(uc, &mapped_code, pc.wrapping_add(4)),
                direct_jump_target,
                direct_jump_target_instruction: direct_jump_target
                    .filter(|_| direct_jump_target_in_trampoline)
                        .and_then(|target| read_unicorn_code_u32(uc, &mapped_code, target)),
                direct_jump_target_in_trampoline,
                direct_jump_trampoline_origin,
                current_trampoline_origin,
            };
            if full_trace_enabled
                && (is_inavi_readiness_probe_pc(pc)
                    || current_trampoline_origin.is_some_and(is_inavi_readiness_probe_pc))
            {
                let mut readiness = last_readiness_code_hook.borrow_mut();
                if readiness.len() == UNICORN_WNDPROC_READINESS_TRACE_LIMIT {
                    readiness.remove(0);
                }
                readiness.push(code_record());
            }
            let should_trace_code = full_trace_enabled
                && (sampled_code_trace
                    || direct_jump_target_in_trampoline
                    || direct_jump_trampoline_origin.is_some()
                    || is_inavi_readiness_probe_pc(pc));
            if should_trace_code {
                let mut last_code = last_code_hook.borrow_mut();
                if last_code.len() == UNICORN_TRACE_LIMIT {
                    last_code.remove(0);
                }
                last_code.push(code_record());
            }
            if let (Some(instruction), Some(next_instruction), Some(target)) = (
                instruction,
                direct_jump_target
                    .filter(|_| direct_jump_target_in_trampoline)
                    .and_then(|_| read_unicorn_code_u32(uc, &mapped_code, pc.wrapping_add(4))),
                direct_jump_target,
            ) {
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
            if target != 0 && target >= 0x0001_0000 && target % 4 == 0 {
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
        } else {
            let fast_mapped_code = Rc::new(MappedCodeIndex::new(self.mapped_blobs.clone()));
            let fast_trampoline_ranges = Rc::new(self.trampoline_ranges.clone());
            for trampoline in &self.trampoline_jumps {
                let origin = trampoline.origin;
                let mapped_code = Rc::clone(&fast_mapped_code);
                let trampoline_ranges = Rc::clone(&fast_trampoline_ranges);
                uc.add_code_hook(
                    u64::from(origin),
                    u64::from(origin),
                    move |uc, address, _size| {
                        let pc = address as u32;
                        let Some(instruction) = read_unicorn_code_u32(uc, &mapped_code, pc) else {
                            return;
                        };
                        let Some(target) = decode_direct_jump_target(pc, instruction)
                            .filter(|target| target_in_ranges(*target, &trampoline_ranges))
                        else {
                            return;
                        };
                        let Some(next_instruction) =
                            read_unicorn_code_u32(uc, &mapped_code, pc.wrapping_add(4))
                        else {
                            return;
                        };
                        if is_patched_trampoline_jump(
                            instruction,
                            next_instruction,
                            target,
                            &trampoline_ranges,
                        ) {
                            let _ = uc.reg_write(RegisterMIPS::PC, u64::from(target));
                        }
                    },
                )
                .map_err(|err| {
                    Error::Backend(format!(
                        "install fast-start trampoline origin hook 0x{origin:08x}: {err:?}"
                    ))
                })?;
            }
            for (base, size) in &self.trampoline_ranges {
                let start = *base;
                let end = base.saturating_add(size.saturating_sub(1));
                let mapped_code = Rc::clone(&fast_mapped_code);
                let trampoline_ranges = Rc::clone(&fast_trampoline_ranges);
                uc.add_code_hook(
                    u64::from(start),
                    u64::from(end),
                    move |uc, address, _size| {
                        let pc = address as u32;
                        let Some(instruction) = read_unicorn_code_u32(uc, &mapped_code, pc) else {
                            return;
                        };
                        if is_trampoline_sentinel_first_word(instruction) {
                            let Some(next_instruction) =
                                read_unicorn_code_u32(uc, &mapped_code, pc.wrapping_add(4))
                            else {
                                return;
                            };
                            let Some(target) =
                                decode_trampoline_sentinel_target(instruction, next_instruction)
                            else {
                                return;
                            };
                            if target_in_ranges(target, &trampoline_ranges) {
                                let _ = uc.reg_write(RegisterMIPS::PC, u64::from(target));
                            }
                        }
                    },
                )
                .map_err(|err| {
                    Error::Backend(format!(
                        "install fast-start trampoline hook 0x{start:08x}-0x{end:08x}: {err:?}"
                    ))
                })?;
            }
        }

        let last_blocks = Rc::new(RefCell::new(Vec::<UnicornLastBlock>::new()));
        if full_trace_enabled {
            let last_blocks_hook = Rc::clone(&last_blocks);
            let block_trace_counter = Rc::new(Cell::new(0u32));
            let block_trace_counter_hook = Rc::clone(&block_trace_counter);
            uc.add_block_hook(1, 0, move |uc, address, size| {
                let counter = block_trace_counter_hook.get().wrapping_add(1);
                block_trace_counter_hook.set(counter);
                if counter % UNICORN_BLOCK_TRACE_SAMPLE_INTERVAL != 0 {
                    return;
                }
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
        }

        let blocked_get_message = Rc::new(RefCell::new(None));
        let blocked_get_message_hook = Rc::clone(&blocked_get_message);
        let last_imports = Rc::new(RefCell::new(Vec::<UnicornLastImport>::new()));
        let last_imports_hook = Rc::clone(&last_imports);
        let import_milestones = Rc::new(RefCell::new(Vec::<UnicornLastImport>::new()));
        #[cfg(feature = "trace")]
        let import_milestones_hook = Rc::clone(&import_milestones);
        let import_counts = Rc::new(RefCell::new(BTreeMap::<
            UnicornImportCountKey,
            UnicornImportStats,
        >::new()));
        let import_counts_hook = Rc::clone(&import_counts);
        let last_messages = Rc::new(RefCell::new(Vec::<UnicornLastMessage>::new()));
        let last_messages_hook = Rc::clone(&last_messages);
        let last_wndproc_returns = Rc::new(RefCell::new(Vec::<UnicornWndProcReturn>::new()));
        let last_wndproc_returns_hook = Rc::clone(&last_wndproc_returns);
        let last_wndproc_call_traces = Rc::new(RefCell::new(Vec::<UnicornWndProcCallTrace>::new()));
        let last_wndproc_call_traces_hook = Rc::clone(&last_wndproc_call_traces);
        let last_calls_for_wndproc_hook = Rc::clone(&last_calls);
        let last_imports_for_wndproc_hook = Rc::clone(&last_imports);
        let last_code_for_wndproc_hook = Rc::clone(&last_code);
        let last_readiness_code_for_wndproc_hook = Rc::clone(&last_readiness_code);
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
        let self_suspended_guest_threads = Rc::new(RefCell::new(std::collections::BTreeMap::<
            u32,
            SuspendedGuestThread,
        >::new()));
        let self_suspended_guest_threads_hook = Rc::clone(&self_suspended_guest_threads);
        let running_guest_thread = Rc::new(RefCell::new(None::<(u32, u32)>));
        let running_guest_thread_hook = Rc::clone(&running_guest_thread);
        let guest_thread_stack_slots = Rc::new(RefCell::new(std::collections::BTreeMap::new()));
        let guest_thread_stack_slots_hook = Rc::clone(&guest_thread_stack_slots);
        let traps = self.import_traps.clone();
        let framebuffer_ptr = framebuffer as *mut dyn Framebuffer;
        let stack_top = self.stack_top.unwrap_or(0);
        let mapped_kernel_memory = Rc::new(RefCell::new(KernelMemoryMappings::new()));
        let mapped_kernel_memory_hook = Rc::clone(&mapped_kernel_memory);
        uc.add_code_hook(
            u64::from(IMPORT_TRAP_BASE),
            u64::from(IMPORT_TRAP_BASE + IMPORT_TRAP_PAGE_SIZE - 1),
            move |uc, address, _size| {
                let address = address as u32;
                if address == CREATE_WINDOW_RETURN_STUB_ADDR {
                    if handle_create_window_return_stub(
                        unsafe { &mut *kernel_ptr },
                        uc,
                        &create_window_returns_hook,
                        &last_wndproc_returns_hook,
                    )
                    .is_err()
                    {
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
                    if let Some(restore) = callout.send_restore.as_ref() {
                        let _ = unsafe { &mut *kernel_ptr }
                            .complete_active_sent_message(restore.receiver_thread_id, result);
                    } else if let Some(thread_id) = callout.send_thread_id {
                        let kernel = unsafe { &mut *kernel_ptr };
                        if kernel.gwe.active_sent_message_id(thread_id).is_some() {
                            let _ = kernel.complete_active_sent_message(thread_id, result);
                        } else {
                            kernel.gwe.end_send_message(thread_id);
                        }
                    }
                    if let Some(result_ptr) = callout.send_timeout_result_ptr {
                        let _ = uc.mem_write(u64::from(result_ptr), &result.to_le_bytes());
                    }
                    if should_trace_wndproc_message(callout.msg) {
                        record_wndproc_call_trace(
                            &last_wndproc_call_traces_hook,
                            UnicornWndProcCallTrace {
                                source: callout.source,
                                hwnd: callout.hwnd,
                                msg: callout.msg,
                                wparam: callout.wparam,
                                lparam: callout.lparam,
                                wndproc: callout.wndproc,
                                return_pc: callout.return_pc,
                                return_pc_trampoline_origin: None,
                                result,
                                class_name: callout.class_name.clone(),
                                calls: snapshot_recent_unicorn_calls(
                                    &last_calls_for_wndproc_hook,
                                    UNICORN_WNDPROC_TRACE_CALL_LIMIT,
                                ),
                                imports: snapshot_recent_unicorn_imports(
                                    &last_imports_for_wndproc_hook,
                                    UNICORN_WNDPROC_TRACE_IMPORT_LIMIT,
                                ),
                                code: snapshot_recent_unicorn_code(
                                    &last_code_for_wndproc_hook,
                                    UNICORN_WNDPROC_TRACE_CODE_LIMIT,
                                ),
                                readiness_code: snapshot_recent_unicorn_code(
                                    &last_readiness_code_for_wndproc_hook,
                                    UNICORN_WNDPROC_READINESS_TRACE_LIMIT,
                                ),
                            },
                        );
                    }
                    if matches!(
                        callout.msg,
                        crate::ce::gwe::WM_DESTROY | crate::ce::gwe::WM_NCDESTROY
                    ) {
                        unsafe { &mut *kernel_ptr }
                            .gwe
                            .record_destroy_lifecycle_message(callout.hwnd, callout.msg);
                    }
                    if callout.msg == crate::ce::gwe::WM_WINDOWPOSCHANGED {
                        let _ = unsafe { &mut *kernel_ptr }
                            .release_message_pointer_payload(callout.lparam);
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
                            return_pc_trampoline_origin: None,
                            result,
                            class_name: callout.class_name.clone(),
                        },
                    );
                    if callout.finalize_destroy && !callout.remaining_destroy_callouts.is_empty() {
                        let mut remaining = callout.remaining_destroy_callouts;
                        let next = remaining.remove(0);
                        let next_hwnd = next.hwnd;
                        let next_wndproc = next.wndproc;
                        pending_wndproc_returns_hook
                            .borrow_mut()
                            .push(PendingWndProcReturn {
                                source: callout.source,
                                hwnd: next_hwnd,
                                msg: crate::ce::gwe::WM_DESTROY,
                                wparam: 0,
                                lparam: 0,
                                wndproc: next_wndproc,
                                return_pc: callout.return_pc,
                                class_name: next.class_name,
                                api_result: callout.api_result,
                                dialog_result_hwnd: callout.dialog_result_hwnd,
                                finalize_destroy: true,
                                destroy_root_hwnd: callout.destroy_root_hwnd,
                                remaining_destroy_callouts: remaining,
                                send_thread_id: None,
                                send_timeout_result_ptr: None,
                                send_restore: None,
                            });
                        if write_wndproc_call_registers(
                            uc,
                            next_hwnd,
                            crate::ce::gwe::WM_DESTROY,
                            0,
                            0,
                            next_wndproc,
                            WNDPROC_RETURN_STUB_ADDR,
                        ) {
                            return;
                        }
                        let _ = pending_wndproc_returns_hook.borrow_mut().pop();
                        let _ = uc.emu_stop();
                        return;
                    }
                    if callout.finalize_destroy {
                        let destroy_root = callout.destroy_root_hwnd.unwrap_or(callout.hwnd);
                        let time_ms = unsafe { &*kernel_ptr }.timers.tick_count();
                        unsafe { &mut *kernel_ptr }
                            .gwe
                            .destroy_window(destroy_root, time_ms);
                    }
                    let api_result = callout.api_result.or_else(|| {
                        callout
                            .dialog_result_hwnd
                            .and_then(|hwnd| unsafe { &*kernel_ptr }.gwe.dialog_result(hwnd))
                    });
                    if let Some(api_result) = api_result {
                        let _ = uc.reg_write(RegisterMIPS::V0, u64::from(api_result));
                    }
                    if let Some(restore) = callout.send_restore {
                        let completed = unsafe { &mut *kernel_ptr }
                            .take_completed_send_message_result(restore.send_id)
                            .unwrap_or(result);
                        let Some(index) = blocked_wait_threads_hook
                            .borrow()
                            .iter()
                            .position(|blocked| blocked.wait_id == restore.wait_id)
                        else {
                            let _ = uc.emu_stop();
                            return;
                        };
                        let blocked = blocked_wait_threads_hook.borrow_mut().remove(index);
                        let _ = unsafe { &mut *kernel_ptr }.remove_blocked_waiter(blocked.wait_id);
                        let previous_running_thread = match blocked.kind {
                            BlockedWaitKind::SendMessage {
                                previous_running_thread,
                                ..
                            } => previous_running_thread,
                            _ => None,
                        };
                        restore_mips_gprs(uc, &blocked.regs);
                        *current_thread_id_hook.borrow_mut() = restore.sender_thread_id;
                        let _ = update_user_kdata_current_ids(
                            uc,
                            restore.sender_thread_id,
                            unsafe { &*kernel_ptr }.current_process_id(),
                        );
                        *running_guest_thread_hook.borrow_mut() = previous_running_thread;
                        let writes = [
                            uc.reg_write(RegisterMIPS::V0, u64::from(completed)),
                            uc.reg_write(RegisterMIPS::PC, u64::from(blocked.return_pc)),
                            uc.reg_write(RegisterMIPS::RA, u64::from(blocked.return_pc)),
                        ];
                        if writes.into_iter().any(|write| write.is_err()) {
                            let _ = uc.emu_stop();
                        }
                        return;
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
                        let _ = update_user_kdata_current_ids(
                            uc,
                            callout.creator_thread_id,
                            unsafe { &*kernel_ptr }.current_process_id(),
                        );
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
                    activate_suspended_thread(
                        uc,
                        unsafe { &*kernel_ptr },
                        &current_thread_id_hook,
                        &running_guest_thread_hook,
                        &suspended,
                    );
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
                    let ra = read_mips_reg(uc, RegisterMIPS::RA);
                    let trace_name =
                        trace_import_name(trap.module_kind, trap.ordinal, trap.name.as_deref());
                    import_counts_hook
                        .borrow_mut()
                        .entry(UnicornImportCountKey {
                            module: trap.module_name.clone(),
                            ordinal: trap.ordinal,
                            name: trace_name.clone(),
                        })
                        .or_default()
                        .record(a0, a1, a2, a3);
                    {
                        let mut imports = last_imports_hook.borrow_mut();
                        if imports.len() == UNICORN_TRACE_LIMIT {
                            imports.remove(0);
                        }
                        let import = UnicornLastImport {
                            pc: address,
                            ra,
                            module: trap.module_name.clone(),
                            kind: trap.module_kind,
                            ordinal: trap.ordinal,
                            name: trace_name.clone(),
                            a0,
                            a1,
                            a2,
                            a3,
                            sp,
                            result: None,
                            detail: None,
                        };
                        imports.push(import.clone());
                        #[cfg(feature = "trace")]
                        {
                            if is_import_milestone(trap.module_kind, trap.ordinal) {
                                let mut milestones = import_milestones_hook.borrow_mut();
                                if milestones.len() == UNICORN_IMPORT_MILESTONE_LIMIT {
                                    milestones.remove(0);
                                }
                                milestones.push(import);
                            }
                        }
                    }
                    tracing::debug!(
                        target: "ce.imports",
                        pc = format_args!("0x{address:08x}"),
                        ra = format_args!("0x{ra:08x}"),
                        module = trap.module_name.as_str(),
                        kind = ?trap.module_kind,
                        ordinal = trap.ordinal,
                        name = trace_name.as_deref().unwrap_or("<ordinal>"),
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
                    try_block_sleep(
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
                        &self_suspended_guest_threads_hook,
                        &running_guest_thread_hook,
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
                    try_block_wait_for_multiple_objects(
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
                    try_block_msg_wait_for_multiple_objects_ex(
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
                    try_block_serial_read_file(
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
                        active_thread_id,
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
                        active_thread_id,
                        &current_thread_id_hook,
                        &running_guest_thread_hook,
                        &blocked_wait_threads_hook,
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
                    let name =
                        trace_import_name(trap.module_kind, trap.ordinal, trap.name.as_deref());
                    let detail = import_detail_after_return(
                        unsafe { &*kernel_ptr },
                        memory.uc,
                        trap.module_kind,
                        trap.ordinal,
                        &args,
                        result,
                        active_thread_id,
                    );
                    if let Some(import) = last_imports_hook
                        .borrow_mut()
                        .iter_mut()
                        .rev()
                        .find(|import| import.pc == address && import.result.is_none())
                    {
                        import.name = name.clone();
                        import.result = Some(result);
                        import.detail = detail.clone();
                    }
                    #[cfg(feature = "trace")]
                    {
                        if is_import_milestone(trap.module_kind, trap.ordinal)
                            && let Some(import) = import_milestones_hook
                                .borrow_mut()
                                .iter_mut()
                                .rev()
                                .find(|import| import.pc == address && import.result.is_none())
                        {
                            import.name = name.clone();
                            import.result = Some(result);
                            import.detail = detail.clone();
                        }
                    }
                    tracing::debug!(
                        target: "ce.imports",
                        pc = format_args!("0x{address:08x}"),
                        module = trap.module_name.as_str(),
                        kind = ?trap.module_kind,
                        ordinal = trap.ordinal,
                        name = trace_import_name(
                            trap.module_kind,
                            trap.ordinal,
                            trap.name.as_deref(),
                        )
                        .as_deref()
                        .unwrap_or("<ordinal>"),
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
                        &self_suspended_guest_threads_hook,
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
        #[cfg(feature = "trace")]
        if full_trace_enabled {
            let render_map_global_watch = Rc::clone(&inavi_render_milestones);
            uc.add_mem_hook(
                HookType::MEM_WRITE,
                0x0081_5550,
                0x0081_55c0,
                move |uc, _access, address, size, value| {
                    record_render_map_global_write(
                        &render_map_global_watch,
                        uc,
                        address as u32,
                        size,
                        value,
                    );
                    true
                },
            )
            .map_err(|err| Error::Backend(format!("install render-map watch hook: {err:?}")))?;
        }
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
            &self.trampoline_jumps,
            memory_fault.borrow().clone(),
            indirect_call_probe.borrow().clone(),
            host_wall_clock_stop.borrow().clone(),
            interrupt_probe.borrow().clone(),
            invalid_instruction_probe.borrow().clone(),
            pc_stop.borrow().clone(),
            last_calls.borrow().clone(),
            last_imports.borrow().clone(),
            import_milestones.borrow().clone(),
            kernel.file_io_stats(),
            kernel.scheduler_stats(),
            kernel.gwe_stats(),
            kernel.recent_file_open_ops().to_vec(),
            kernel.recent_file_ops().to_vec(),
            last_messages.borrow().clone(),
            last_wndproc_returns.borrow().clone(),
            last_wndproc_call_traces.borrow().clone(),
            last_mfc_dispatch.borrow().clone(),
            last_inavi_display.borrow().clone(),
            last_inavi_controller.borrow().clone(),
            inavi_render_milestones.borrow().clone(),
            last_code.borrow().clone(),
            last_blocks.borrow().clone(),
            import_count_snapshot(&import_counts.borrow()),
            kernel.memory.allocations().count(),
            kernel
                .memory
                .allocations()
                .map(|allocation| u64::from(allocation.actual_size))
                .sum(),
            kernel.memory.virtual_allocations().count(),
            kernel
                .memory
                .virtual_allocations()
                .map(|allocation| u64::from(allocation.size))
                .sum(),
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

fn range_overlaps_any(base: u32, size: u32, ranges: &[(u32, u32)]) -> bool {
    ranges
        .iter()
        .any(|(other_base, other_size)| ranges_overlap(base, size, *other_base, *other_size))
}

fn choose_dll_load_base(
    preferred_base: u32,
    image_size: u32,
    occupied_ranges: &[(u32, u32)],
    next_dll_base: &mut u32,
) -> Result<u32> {
    if !range_overlaps_any(preferred_base, image_size, occupied_ranges) {
        return Ok(preferred_base);
    }

    allocate_relocated_dll_base(image_size, occupied_ranges, next_dll_base)
}

fn allocate_relocated_dll_base(
    image_size: u32,
    occupied_ranges: &[(u32, u32)],
    next_dll_base: &mut u32,
) -> Result<u32> {
    let mut candidate = align_up_4k(*next_dll_base)?;
    while range_overlaps_any(candidate, image_size, occupied_ranges) {
        candidate = candidate
            .checked_add(image_size)
            .and_then(|base| base.checked_add(0x0010_0000))
            .ok_or_else(|| Error::InvalidArgument("DLL load base overflow".to_owned()))?;
        candidate = align_up_4k(candidate)?;
    }
    *next_dll_base = candidate
        .checked_add(image_size)
        .and_then(|base| base.checked_add(0x0010_0000))
        .ok_or_else(|| Error::InvalidArgument("DLL load base overflow".to_owned()))?;
    Ok(candidate)
}

fn module_file_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

fn loaded_module_info(image: &PeImage, load_base: u32) -> LoadedPeModuleInfo {
    let mut exports_by_name = HashMap::new();
    let mut exports_by_ordinal = HashMap::new();
    if let Some(exports) = image.exports.as_ref() {
        for export in &exports.functions {
            if export.rva == 0 || export.forwarder.is_some() {
                continue;
            }
            let va = load_base.wrapping_add(export.rva);
            exports_by_ordinal.insert(export.ordinal, va);
            if let Some(name) = export.name.as_deref() {
                exports_by_name.insert(crate::ce::kernel::normalize_symbol_name(name), va);
            }
        }
    }
    LoadedPeModuleInfo {
        name: module_file_name(&image.path).to_owned(),
        base: load_base,
        exports_by_name,
        exports_by_ordinal,
    }
}

fn user_kdata_page() -> Vec<u8> {
    let mut page = vec![0; USER_KDATA_PAGE_SIZE as usize];
    write_user_kdata_handle(&mut page, SYS_HANDLE_CURRENT_THREAD, 1);
    write_user_kdata_handle(&mut page, SYS_HANDLE_CURRENT_PROCESS, 1);
    page
}

fn user_kdata_handle_address(index: usize) -> u32 {
    USER_KDATA_BASE + USER_KDATA_SYSHANDLE_OFFSET + index as u32 * 4
}

fn write_user_kdata_handle(page: &mut [u8], index: usize, value: u32) {
    let offset = user_kdata_handle_address(index).saturating_sub(USER_KDATA_PAGE_BASE) as usize;
    page[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

#[cfg(feature = "unicorn")]
fn update_user_kdata_current_ids<D>(
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    thread_id: u32,
    process_id: u32,
) -> Result<()> {
    uc.mem_write(
        u64::from(user_kdata_handle_address(SYS_HANDLE_CURRENT_THREAD)),
        &thread_id.to_le_bytes(),
    )
    .map_err(|err| Error::Backend(format!("write current thread id to KData: {err:?}")))?;
    uc.mem_write(
        u64::from(user_kdata_handle_address(SYS_HANDLE_CURRENT_PROCESS)),
        &process_id.to_le_bytes(),
    )
    .map_err(|err| Error::Backend(format!("write current process id to KData: {err:?}")))?;
    Ok(())
}

#[cfg(feature = "unicorn")]
fn patch_mips_unicorn_trampolines(
    image: &PeImage,
    load_base: u32,
    mapped: &mut Vec<u8>,
    external_stub_base: Option<u32>,
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
        let mut jump_table_data_ranges =
            mips_halfword_jump_table_ranges(mapped, load_base, start, end, &image.path)?;
        jump_table_data_ranges.extend(mips_byte_jump_table_ranges(
            mapped,
            load_base,
            start,
            end,
            &image.path,
        )?);
        let mut rva = start;
        while rva.checked_add(8).is_some_and(|next| next <= end) {
            if mips_patch_rva_overlaps_data_ranges(rva, &jump_table_data_ranges) {
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
    let mut stub_bytes = Vec::new();
    if external_stub_base.is_none() && mapped.len() < aligned_len {
        mapped.resize(aligned_len, 0);
    }
    let mut stub_rva = if external_stub_base.is_some() {
        0
    } else {
        aligned_len as u32
    };
    let mut trampoline_jumps = Vec::with_capacity(patches.len());
    for patch in patches {
        let stub_pc = external_stub_base
            .unwrap_or(load_base)
            .wrapping_add(stub_rva);
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
        let stub_offset = if external_stub_base.is_some() {
            stub_rva as usize
        } else {
            stub_rva as usize
        };
        let stub_end = stub_offset
            .checked_add(stub_words.len() * 4)
            .ok_or_else(|| Error::InvalidArgument("branch-likely stub overflow".to_owned()))?;
        trampoline_jumps.push(MipsTrampolineJump {
            origin: patch.pc,
            stub: stub_pc,
            byte_len: (stub_words.len() * 4) as u32,
        });
        let target_bytes = if external_stub_base.is_some() {
            &mut stub_bytes
        } else {
            &mut *mapped
        };
        if target_bytes.len() < stub_end {
            target_bytes.resize(stub_end, 0);
        }
        for (index, word) in stub_words.into_iter().enumerate() {
            let offset = stub_offset + index * 4;
            target_bytes[offset..offset + 4].copy_from_slice(&word.to_le_bytes());
        }
        stub_rva = stub_rva
            .checked_add((stub_end - stub_offset) as u32)
            .ok_or_else(|| Error::InvalidArgument("branch-likely stub RVA overflow".to_owned()))?;
    }
    let (range_base, range_size, external_mapped) = if let Some(stub_base) = external_stub_base {
        let final_len = align_up_4k(stub_rva)? as usize;
        if stub_bytes.len() < final_len {
            stub_bytes.resize(final_len, 0);
        }
        (stub_base, final_len as u32, Some(stub_bytes))
    } else {
        let final_len = align_up_4k(stub_rva)? as usize;
        if mapped.len() < final_len {
            mapped.resize(final_len, 0);
        }
        let range_base = load_base.wrapping_add(aligned_len as u32);
        let range_size = final_len
            .checked_sub(aligned_len)
            .and_then(|size| u32::try_from(size).ok())
            .ok_or_else(|| Error::InvalidArgument("branch trampoline range overflow".to_owned()))?;
        (range_base, range_size, None)
    };
    Ok(MipsTrampolinePatchResult {
        range: Some((range_base, range_size)),
        jumps: trampoline_jumps,
        external_mapped,
    })
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct MipsTrampolinePatchResult {
    range: Option<(u32, u32)>,
    jumps: Vec<MipsTrampolineJump>,
    external_mapped: Option<Vec<u8>>,
}

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
fn trampoline_pages_for_ranges(ranges: &[(u32, u32)]) -> HashSet<u32> {
    let mut pages = HashSet::new();
    for (base, size) in ranges {
        if *size == 0 {
            continue;
        }
        let first_page = base >> 12;
        let last_page = base.saturating_add(size.saturating_sub(1)) >> 12;
        for page in first_page..=last_page {
            pages.insert(page);
        }
    }
    pages
}

#[cfg(feature = "unicorn")]
fn target_in_trampoline_pages(target: u32, pages: &HashSet<u32>) -> bool {
    pages.contains(&(target >> 12))
}

#[cfg(feature = "unicorn")]
fn trampoline_stub_by_origin(jumps: &[MipsTrampolineJump]) -> HashMap<u32, u32> {
    jumps
        .iter()
        .map(|jump| (jump.origin, jump.stub))
        .collect::<HashMap<_, _>>()
}

#[cfg(feature = "unicorn")]
fn trampoline_origin_by_stub(jumps: &[MipsTrampolineJump]) -> HashMap<u32, u32> {
    jumps
        .iter()
        .map(|jump| (jump.stub, jump.origin))
        .collect::<HashMap<_, _>>()
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
fn mips_byte_jump_table_ranges(
    mapped: &[u8],
    load_base: u32,
    start: u32,
    end: u32,
    path: &str,
) -> Result<Vec<(u32, u32)>> {
    let mut ranges = Vec::new();
    let mut rva = start;
    while rva.checked_add(28).is_some_and(|next| next <= end) {
        let Some(range) =
            decode_mips_byte_jump_table_range(mapped, load_base, start, end, rva, path)?
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
fn decode_mips_byte_jump_table_range(
    mapped: &[u8],
    load_base: u32,
    section_start: u32,
    section_end: u32,
    rva: u32,
    path: &str,
) -> Result<Option<(u32, u32)>> {
    let lui = read_mapped_word(mapped, rva, path)?;
    let addiu = read_mapped_word(mapped, rva + 4, path)?;
    let addu_index = read_mapped_word(mapped, rva + 8, path)?;
    let lb = read_mapped_word(mapped, rva + 12, path)?;
    let addu_target = read_mapped_word(mapped, rva + 16, path)?;
    let jr = read_mapped_word(mapped, rva + 20, path)?;
    let delay_slot = read_mapped_word(mapped, rva + 24, path)?;

    let Some(base_register) = decode_mips_lui_rt(lui) else {
        return Ok(None);
    };
    if !is_mips_addiu_same_register(addiu, base_register) {
        return Ok(None);
    }
    let Some((index_register, selector_register)) =
        decode_mips_addu_with_base(addu_index, base_register)
    else {
        return Ok(None);
    };
    if !is_mips_lb_same_register(lb, index_register) {
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
    if table_rva != rva + 28 || table_rva >= section_end {
        return Ok(None);
    }
    let Some(entry_count) =
        find_mips_jump_table_entry_count(mapped, section_start, rva, selector_register, path)?
    else {
        return Ok(None);
    };
    if entry_count == 0 {
        return Ok(None);
    }
    let Some(table_end) = table_rva.checked_add(entry_count) else {
        return Ok(None);
    };
    if table_end > section_end {
        return Ok(None);
    }
    Ok(Some((table_rva, entry_count)))
}

#[cfg(feature = "unicorn")]
fn find_mips_halfword_jump_table_entry_count(
    mapped: &[u8],
    section_start: u32,
    setup_rva: u32,
    selector_register: u32,
    path: &str,
) -> Result<Option<u32>> {
    find_mips_jump_table_entry_count(mapped, section_start, setup_rva, selector_register, path)
}

#[cfg(feature = "unicorn")]
fn find_mips_jump_table_entry_count(
    mapped: &[u8],
    section_start: u32,
    setup_rva: u32,
    selector_register: u32,
    path: &str,
) -> Result<Option<u32>> {
    let search_start = setup_rva
        .saturating_sub(MIPS_JUMP_TABLE_SELECTOR_SEARCH_BACK)
        .max(section_start);
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
fn decode_mips_addu_with_base(instruction: u32, base_register: u32) -> Option<(u32, u32)> {
    if instruction >> 26 != 0 || (instruction & 0x3f) != 0x21 {
        return None;
    }
    let rs = (instruction >> 21) & 0x1f;
    let rt = (instruction >> 16) & 0x1f;
    let rd = (instruction >> 11) & 0x1f;
    if rt == base_register {
        Some((rd, rs))
    } else if rs == base_register {
        Some((rd, rt))
    } else {
        None
    }
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
fn is_mips_lb_same_register(instruction: u32, register: u32) -> bool {
    instruction >> 26 == 0x20
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
    let mut words = vec![
        encode_mips_lui(31, link_address >> 16),
        encode_mips_ori(31, 31, link_address & 0xffff),
        delay_slot,
    ];
    append_mips_jump_sequence(&mut words, stub_pc.wrapping_add(12), target)?;
    Ok(words)
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
    let prefix_len = if branch.link { 4 } else { 2 };
    let true_jump_pc = stub_pc.wrapping_add((prefix_len + 1) * 4);
    let true_jump_len = mips_jump_sequence_len(true_jump_pc, branch.target)?;
    let false_path_pc = stub_pc.wrapping_add((prefix_len + 1 + true_jump_len as u32) * 4);

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
    words.push(delay_slot);
    append_mips_jump_sequence(&mut words, true_jump_pc, branch.target)?;
    append_mips_jump_sequence(&mut words, false_path_pc, fallthrough)?;
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
    let true_jump_pc = stub_pc.wrapping_add(12);
    let true_jump_len = mips_jump_sequence_len(true_jump_pc, branch.target)?;
    let false_path_pc = stub_pc.wrapping_add((3 + true_jump_len as u32) * 4);

    let mut words = vec![
        encode_mips_cond_branch(
            branch.inverse_branch,
            branch.rs,
            branch.rt,
            stub_pc,
            false_path_pc,
        )?,
        MIPS_NOP,
        delay_slot,
    ];
    append_mips_jump_sequence(&mut words, true_jump_pc, branch.target)?;
    words.push(delay_slot);
    append_mips_jump_sequence(&mut words, false_path_pc.wrapping_add(4), fallthrough)?;
    Ok(words)
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
fn mips_jump_sequence_len(pc: u32, target: u32) -> Result<usize> {
    if pc.wrapping_add(4) & 0xf000_0000 == target & 0xf000_0000 {
        Ok(2)
    } else {
        Ok(4)
    }
}

#[cfg(feature = "unicorn")]
fn append_mips_jump_sequence(words: &mut Vec<u32>, pc: u32, target: u32) -> Result<()> {
    if mips_jump_sequence_len(pc, target)? == 2 {
        words.push(encode_mips_jump(pc, target)?);
        words.push(MIPS_NOP);
    } else {
        words.push(encode_mips_lui(26, target >> 16));
        words.push(encode_mips_ori(26, 26, target & 0xffff));
        words.push(encode_mips_jr(26));
        words.push(MIPS_NOP);
    }
    Ok(())
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
fn encode_mips_jr(rs: u32) -> u32 {
    (rs << 21) | 0x08
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
#[derive(Debug, Clone)]
struct KernelMemoryMappings {
    ranges: Vec<(u32, u32)>,
    virtual_pages: HashSet<u32>,
    heap_spill_cursor: u32,
    heap_generation: u64,
    virtual_generation: u64,
}

#[cfg(feature = "unicorn")]
impl KernelMemoryMappings {
    fn new() -> Self {
        Self {
            ranges: vec![(GUEST_HEAP_ARENA_BASE, GUEST_HEAP_ARENA_SIZE)],
            virtual_pages: HashSet::new(),
            heap_spill_cursor: GUEST_HEAP_ARENA_BASE + GUEST_HEAP_ARENA_SIZE,
            heap_generation: u64::MAX,
            virtual_generation: u64::MAX,
        }
    }
}

#[cfg(feature = "unicorn")]
fn map_kernel_memory_allocations<D>(
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    kernel: &CeKernel,
    mapped: &mut KernelMemoryMappings,
) -> Result<()> {
    if mapped.heap_generation != kernel.memory.heap_generation() {
        map_heap_spillover(uc, kernel, mapped)?;
        mapped.heap_generation = kernel.memory.heap_generation();
    }

    if mapped.virtual_generation != kernel.memory.virtual_generation() {
        reclaim_stale_virtual_memory_pages(uc, kernel, mapped)?;
        for allocation in kernel.memory.virtual_allocations() {
            let newly_mapped = map_guest_range(
                uc,
                mapped,
                allocation.base,
                allocation.size,
                virtual_allocation_perms(allocation.protect),
                "virtual allocation",
            )?;
            mapped.virtual_pages.extend(newly_mapped.iter().copied());
            if !newly_mapped.is_empty() && !allocation.initial_bytes.is_empty() {
                uc.mem_write(u64::from(allocation.base), &allocation.initial_bytes)
                    .map_err(|err| {
                        Error::Backend(format!(
                            "seed virtual allocation 0x{:08x}: {err:?}",
                            allocation.base
                        ))
                    })?;
            }
        }
        mapped.virtual_generation = kernel.memory.virtual_generation();
    }

    Ok(())
}

#[cfg(feature = "unicorn")]
fn map_heap_spillover<D>(
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    kernel: &CeKernel,
    mapped: &mut KernelMemoryMappings,
) -> Result<()> {
    let high_water = kernel.memory.heap_high_water_mark();
    if high_water <= mapped.heap_spill_cursor {
        return Ok(());
    }
    let base = mapped.heap_spill_cursor;
    let mapped_end = high_water
        .checked_add(GUEST_HEAP_SPILLOVER_GRANULARITY - 1)
        .map(|end| end & !(GUEST_HEAP_SPILLOVER_GRANULARITY - 1))
        .ok_or_else(|| Error::InvalidArgument("heap spillover cursor overflow".to_owned()))?;
    let size = mapped_end
        .checked_sub(base)
        .ok_or_else(|| Error::InvalidArgument("heap spillover range underflow".to_owned()))?;
    map_guest_aligned_range(
        uc,
        mapped,
        base,
        size,
        MemoryPerms::READ | MemoryPerms::WRITE,
        "heap spillover",
    )?;
    mapped.heap_spill_cursor = mapped_end
        .checked_add(0xfff)
        .map(|end| end & !0xfff)
        .ok_or_else(|| Error::InvalidArgument("heap spillover cursor overflow".to_owned()))?;
    Ok(())
}

#[cfg(feature = "unicorn")]
fn map_guest_aligned_range<D>(
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    mapped: &mut KernelMemoryMappings,
    base: u32,
    size: u32,
    perms: MemoryPerms,
    label: &str,
) -> Result<()> {
    if size == 0 {
        return Ok(());
    }
    if base & 0xfff != 0 || size & 0xfff != 0 {
        return Err(Error::InvalidArgument(format!(
            "{label} range must be page aligned"
        )));
    }
    if mapped.ranges.iter().any(|(mapped_base, mapped_size)| {
        base >= *mapped_base
            && base.saturating_add(size) <= mapped_base.saturating_add(*mapped_size)
    }) {
        return Ok(());
    }
    uc.mem_map(u64::from(base), u64::from(size), unicorn_perms(perms))
        .map_err(|err| Error::Backend(format!("map {label} 0x{base:08x}+0x{size:x}: {err:?}")))?;
    mapped.ranges.push((base, size));
    Ok(())
}

#[cfg(feature = "unicorn")]
fn reclaim_stale_virtual_memory_pages<D>(
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    kernel: &CeKernel,
    mapped: &mut KernelMemoryMappings,
) -> Result<()> {
    let live_pages = live_virtual_memory_pages(kernel)?;
    let stale_pages = mapped
        .virtual_pages
        .iter()
        .copied()
        .filter(|page| !live_pages.contains(page))
        .collect::<Vec<_>>();
    for page in stale_pages {
        uc.mem_unmap(u64::from(page), 0x1000).map_err(|err| {
            Error::Backend(format!("unmap stale virtual page 0x{page:08x}: {err:?}"))
        })?;
        mapped.virtual_pages.remove(&page);
        mapped
            .ranges
            .retain(|(base, size)| *base != page || *size != 0x1000);
    }
    Ok(())
}

#[cfg(feature = "unicorn")]
fn live_virtual_memory_pages(kernel: &CeKernel) -> Result<HashSet<u32>> {
    let mut pages = HashSet::new();
    for allocation in kernel.memory.virtual_allocations() {
        insert_guest_range_pages(
            &mut pages,
            allocation.base,
            allocation.size,
            "virtual allocation",
        )?;
    }
    Ok(pages)
}

#[cfg(feature = "unicorn")]
fn insert_guest_range_pages(
    pages: &mut HashSet<u32>,
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
        pages.insert(page_base);
        page_base = page_base
            .checked_add(0x1000)
            .ok_or_else(|| Error::InvalidArgument(format!("{label} page overflow")))?;
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
        let saved_process_id = kernel.current_process_id();

        kernel.set_process_module_base(image.image_base());
        let child_module_path = kernel
            .host_path_to_guest_mount(&path)
            .unwrap_or_else(|| path.to_string_lossy().replace('/', "\\"));
        kernel.set_process_module_path(child_module_path);
        kernel.set_process_module_host_path(path.clone());
        kernel.set_process_command_line(launch.command_line.clone().unwrap_or_default());
        kernel.set_current_process_id(launch.process_id);

        let child_result = (|| -> Result<u32> {
            let mut child = UnicornMips::new()?;
            child.load_pe_image(&image)?;
            let mut child_framebuffer = VirtualFramebuffer::default_primary()?;
            child.run_until_import_trap_with_framebuffer_limit(
                kernel,
                &mut child_framebuffer,
                instruction_limit,
            )?;
            Ok(child
                .last_debug_snapshot()
                .and_then(|snapshot| {
                    snapshot
                        .encoded_kernel_exit
                        .as_ref()
                        .map(|exit| exit.exit_code)
                })
                .unwrap_or(0))
        })();

        kernel.set_process_module_base(saved_base);
        kernel.set_process_module_path(saved_path);
        if let Some(saved_host_path) = saved_host_path {
            kernel.set_process_module_host_path(saved_host_path);
        }
        kernel.set_process_command_line(saved_command_line);
        kernel.set_current_process_id(saved_process_id);

        let exit_code = child_result?;
        kernel.mark_process_launch_exited(&launch, exit_code);
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
    mapped: &mut KernelMemoryMappings,
    base: u32,
    size: u32,
    perms: MemoryPerms,
    label: &str,
) -> Result<Vec<u32>> {
    let first_page = base & !0xfff;
    let page_end = base
        .checked_add(size.max(1))
        .and_then(|end| end.checked_add(0xfff))
        .map(|end| end & !0xfff)
        .ok_or_else(|| Error::InvalidArgument(format!("{label} range overflow")))?;
    let mut page_base = first_page;
    let mut newly_mapped = Vec::new();
    while page_base < page_end {
        if mapped.ranges.iter().any(|(mapped_base, mapped_size)| {
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
        mapped.ranges.push((page_base, 0x1000));
        newly_mapped.push(page_base);
        page_base = page_base
            .checked_add(0x1000)
            .ok_or_else(|| Error::InvalidArgument(format!("{label} page overflow")))?;
    }
    Ok(newly_mapped)
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

fn write_inavi_controller_traces(
    f: &mut std::fmt::Formatter<'_>,
    traces: &[UnicornInaviControllerTrace],
) -> std::fmt::Result {
    for (index, trace) in traces.iter().enumerate() {
        if index != 0 {
            write!(f, ",")?;
        }
        write!(
            f,
            "{}@0x{:08x}/ra=0x{:08x}/sp=0x{:08x}/v0=0x{:08x}/a0=0x{:08x}/a1=0x{:08x}/a2=0x{:08x}/a3=0x{:08x}/s0=0x{:08x}/s2=0x{:08x}/s3=0x{:08x}/s4=0x{:08x}/s5=0x{:08x}/s6=0x{:08x}/s7=0x{:08x}/fp=0x{:08x}",
            trace.label,
            trace.pc,
            trace.ra,
            trace.sp,
            trace.v0,
            trace.a0,
            trace.a1,
            trace.a2,
            trace.a3,
            trace.s0,
            trace.s2,
            trace.s3,
            trace.s4,
            trace.s5,
            trace.s6,
            trace.s7,
            trace.fp
        )?;
        if let Some(instruction) = trace.instruction {
            write!(f, "/insn=0x{instruction:08x}")?;
        }
        if let Some(sp10) = trace.sp10 {
            write!(f, "/sp10=0x{sp10:08x}")?;
        }
        if let Some(sp48) = trace.sp48 {
            write!(f, "/sp48=0x{sp48:08x}")?;
        }
        if let Some(controller) = trace.controller {
            write!(f, "/controller=0x{controller:08x}")?;
        }
        if let Some(hwnd) = trace.hwnd {
            write!(f, "/hwnd=0x{hwnd:08x}")?;
        }
        if let Some(msg) = trace.msg {
            write!(f, "/msg=0x{msg:08x}")?;
        }
        if let Some(wparam) = trace.wparam {
            write!(f, "/w=0x{wparam:08x}")?;
        }
        if let Some(lparam) = trace.lparam {
            write!(f, "/l=0x{lparam:08x}")?;
        }
        if let Some(classifier) = trace.classifier {
            write!(f, "/class=0x{classifier:08x}")?;
        }
        if let Some(selected_obj) = trace.selected_obj {
            write!(f, "/obj=0x{selected_obj:08x}")?;
        }
        if let Some(selected_vtable) = trace.selected_vtable {
            write!(f, "/vt=0x{selected_vtable:08x}")?;
        }
        if let Some(selected_target) = trace.selected_target {
            write!(f, "/target=0x{selected_target:08x}")?;
        }
        if let Some(paint_base) = trace.paint_base {
            write!(f, "/paint_base=0x{paint_base:08x}")?;
        }
        if let Some(paint_gate) = trace.paint_gate {
            write!(f, "/paint_gate=0x{paint_gate:08x}")?;
        }
        if let Some(paint_render_obj) = trace.paint_render_obj {
            write!(f, "/paint_obj=0x{paint_render_obj:08x}")?;
        }
        if let Some(paint_render_target) = trace.paint_render_target {
            write!(f, "/paint_target=0x{paint_render_target:08x}")?;
        }
        if let Some(render_surface) = trace.render_surface {
            write!(f, "/render_surface=0x{render_surface:08x}")?;
        }
        if let Some(render_enabled) = trace.render_enabled {
            write!(f, "/render_enabled=0x{render_enabled:08x}")?;
        }
        if let Some(render_size_target) = trace.render_size_target {
            write!(f, "/render_size_target=0x{render_size_target:08x}")?;
        }
        if let Some(render_resize_target) = trace.render_resize_target {
            write!(f, "/render_resize_target=0x{render_resize_target:08x}")?;
        }
        if let Some(render_flush_obj) = trace.render_flush_obj {
            write!(f, "/render_flush_obj=0x{render_flush_obj:08x}")?;
        }
        if let Some(render_flush_target) = trace.render_flush_target {
            write!(f, "/render_flush_target=0x{render_flush_target:08x}")?;
        }
        if let Some(render_poll_result) = trace.render_poll_result {
            write!(f, "/render_poll=0x{render_poll_result:08x}")?;
        }
        if let Some(render_dim_ptr) = trace.render_dim_ptr {
            write!(f, "/dim_ptr=0x{render_dim_ptr:08x}")?;
        }
        if let Some(render_dim_w) = trace.render_dim_w {
            write!(f, "/dim_w=0x{render_dim_w:08x}")?;
        }
        if let Some(render_dim_h) = trace.render_dim_h {
            write!(f, "/dim_h=0x{render_dim_h:08x}")?;
        }
        if let Some(aux_base) = trace.aux_base {
            write!(f, "/aux_base=0x{aux_base:08x}")?;
        }
        if let Some(aux_slot_10ec_value) = trace.aux_slot_10ec_value {
            write!(f, "/aux10ec_value=0x{aux_slot_10ec_value:08x}")?;
        }
        if let Some(aux_slot_10f0) = trace.aux_slot_10f0 {
            write!(f, "/aux10f0=0x{aux_slot_10f0:08x}")?;
        }
        if let Some(aux_slot_10f0_vtable) = trace.aux_slot_10f0_vtable {
            write!(f, "/aux10f0_vt=0x{aux_slot_10f0_vtable:08x}")?;
        }
        if let Some(aux_inline_10f8) = trace.aux_inline_10f8 {
            write!(f, "/aux10f8=0x{aux_inline_10f8:08x}")?;
        }
        if let Some(aux_inline_10f8_vtable) = trace.aux_inline_10f8_vtable {
            write!(f, "/aux10f8_vt=0x{aux_inline_10f8_vtable:08x}")?;
        }
        if let Some(aux_link_ee4) = trace.aux_link_ee4 {
            write!(f, "/aux_ee4=0x{aux_link_ee4:08x}")?;
        }
        if let Some(aux_init_flag_edc) = trace.aux_init_flag_edc {
            write!(f, "/aux_edc=0x{aux_init_flag_edc:08x}")?;
        }
        if let Some(aux_vtable_source) = trace.aux_vtable_source {
            write!(f, "/aux_vt_src=0x{aux_vtable_source:08x}")?;
        }
        if let Some(aux_vtable_source_value) = trace.aux_vtable_source_value {
            write!(f, "/aux_vt_src_value=0x{aux_vtable_source_value:08x}")?;
        }
        if let Some(aux_store_addr) = trace.aux_store_addr {
            write!(f, "/aux_store_addr=0x{aux_store_addr:08x}")?;
        }
        if let Some(aux_store_value) = trace.aux_store_value {
            write!(f, "/aux_store_value=0x{aux_store_value:08x}")?;
        }
        #[cfg(feature = "trace")]
        {
            if let Some(query_thunk_slot) = trace.query_thunk_slot {
                write!(f, "/query_slot=0x{query_thunk_slot:08x}")?;
            }
            if let Some(query_thunk_target) = trace.query_thunk_target {
                write!(f, "/query_target=0x{query_thunk_target:08x}")?;
            }
            if let Some(resource_text) = trace.resource_text.as_deref() {
                write!(f, "/text={}", format_trace_string(resource_text))?;
            }
            if let Some(resource_format_text) = trace.resource_format_text.as_deref() {
                write!(f, "/format={}", format_trace_string(resource_format_text))?;
            }
            if let Some(resource_aux_text) = trace.resource_aux_text.as_deref() {
                write!(f, "/aux_text={}", format_trace_string(resource_aux_text))?;
            }
            if let Some(resource_arg_text) = trace.resource_arg_text.as_deref() {
                write!(f, "/arg_text={}", format_trace_string(resource_arg_text))?;
            }
        }
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
            if let Some(kind) = self.trap_module_kind {
                write!(f, " trap_kind={kind:?}")?;
            }
            if let Some(module) = self.trap_module_name.as_deref() {
                write!(f, " trap_module={module}")?;
            }
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
                write!(
                    f,
                    "/max_a0=0x{:08x}/max_a1=0x{:08x}/max_a2=0x{:08x}/max_a3=0x{:08x}",
                    count.max_a0, count.max_a1, count.max_a2, count.max_a3
                )?;
            }
            write!(f, "]")?;
        }
        write!(
            f,
            " heap_live_count={} heap_live_bytes={} virtual_live_count={} virtual_live_bytes={} file_host_open_count={} file_host_read_count={} file_host_read_bytes={} file_memory_backed_open_count={} file_max_read_request={} sched_wait_single_count={} sched_wait_multiple_count={} sched_msg_wait_count={} sched_sleep_count={} sched_yield_count={} sched_wait_acquired_count={} sched_wait_timeout_count={} sched_wait_failed_count={} sched_wait_block_count={} sched_wait_wake_count={} sched_waiter_register_count={} sched_waiter_remove_count={} sched_object_signal_count={} sched_object_wake_candidate_count={} sched_message_input_signal_count={} sched_message_input_wake_candidate_count={} sched_serial_read_signal_count={} sched_serial_read_wake_candidate_count={} sched_send_reply_signal_count={} sched_send_reply_wake_candidate_count={} sched_max_registered_waits={} sched_max_pending_wakes={} gwe_send_transaction_count={} gwe_send_completed_count={} gwe_send_timeout_count={} gwe_send_receiver_terminated_count={} gwe_max_sent_queue_depth={}",
            self.heap_allocation_count,
            self.heap_allocation_bytes,
            self.virtual_allocation_count,
            self.virtual_allocation_bytes,
            self.file_io_stats.host_file_open_count,
            self.file_io_stats.host_file_read_count,
            self.file_io_stats.host_file_read_bytes,
            self.file_io_stats.memory_backed_open_count,
            self.file_io_stats.max_read_request,
            self.scheduler_stats.wait_single_count,
            self.scheduler_stats.wait_multiple_count,
            self.scheduler_stats.msg_wait_count,
            self.scheduler_stats.sleep_count,
            self.scheduler_stats.yield_count,
            self.scheduler_stats.wait_acquired_count,
            self.scheduler_stats.wait_timeout_count,
            self.scheduler_stats.wait_failed_count,
            self.scheduler_stats.wait_block_count,
            self.scheduler_stats.wait_wake_count,
            self.scheduler_stats.waiter_register_count,
            self.scheduler_stats.waiter_remove_count,
            self.scheduler_stats.object_signal_count,
            self.scheduler_stats.object_wake_candidate_count,
            self.scheduler_stats.message_input_signal_count,
            self.scheduler_stats.message_input_wake_candidate_count,
            self.scheduler_stats.serial_read_signal_count,
            self.scheduler_stats.serial_read_wake_candidate_count,
            self.scheduler_stats.send_reply_signal_count,
            self.scheduler_stats.send_reply_wake_candidate_count,
            self.scheduler_stats.max_registered_waits,
            self.scheduler_stats.max_pending_wakes,
            self.gwe_stats.send_transaction_count,
            self.gwe_stats.send_transaction_completed_count,
            self.gwe_stats.send_transaction_timeout_count,
            self.gwe_stats.send_transaction_receiver_terminated_count,
            self.gwe_stats.max_sent_queue_depth
        )?;
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
        if let Some(stop) = self.pc_stop.as_ref() {
            write!(
                f,
                " pc_stop=0x{:08x} pc_stop_ra=0x{:08x} pc_stop_sp=0x{:08x}",
                stop.pc, stop.ra, stop.sp
            )?;
            if let Some(instruction) = stop.instruction {
                write!(f, " pc_stop_insn=0x{instruction:08x}")?;
            }
        }
        if !self.last_imports.is_empty() {
            write!(f, " last_imports=[")?;
            write_unicorn_import_records(f, &self.last_imports)?;
            write!(f, "]")?;
        }
        if !self.import_milestones.is_empty() {
            write!(f, " import_milestones=[")?;
            write_unicorn_import_records(f, &self.import_milestones)?;
            write!(f, "]")?;
        }
        if !self.recent_file_ops.is_empty() {
            write!(f, " recent_file_ops=[")?;
            write_file_trace_records(f, &self.recent_file_ops)?;
            write!(f, "]")?;
        }
        if !self.recent_file_open_ops.is_empty() {
            write!(f, " recent_file_open_ops=[")?;
            write_file_trace_records(f, &self.recent_file_open_ops)?;
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
                    "0x{:08x}->0x{:08x}/{}/ra=0x{:08x}/sp=0x{:08x}",
                    call.pc, call.target, call.kind, call.ra, call.sp
                )?;
                write_call_target_import(f, call)?;
                write!(
                    f,
                    "/a0=0x{:08x}/a1=0x{:08x}/a2=0x{:08x}/a3=0x{:08x}",
                    call.a0, call.a1, call.a2, call.a3
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
                    "{}/hwnd=0x{:08x}/msg=0x{:08x}/w=0x{:08x}/l=0x{:08x}/wndproc=0x{:08x}/return_pc=0x{:08x}",
                    record.source,
                    record.hwnd,
                    record.msg,
                    record.wparam,
                    record.lparam,
                    record.wndproc,
                    record.return_pc
                )?;
                if let Some(origin) = record.return_pc_trampoline_origin {
                    write!(f, "/return_tramp_origin=0x{origin:08x}")?;
                }
                write!(f, "/ret=0x{:08x}", record.result)?;
                if let Some(class_name) = record.class_name.as_deref() {
                    write!(f, "/class={class_name}")?;
                }
            }
            write!(f, "]")?;
        }
        if !self.last_wndproc_call_traces.is_empty() {
            write!(f, " last_wndproc_call_traces=[")?;
            for (index, trace) in self.last_wndproc_call_traces.iter().enumerate() {
                if index != 0 {
                    write!(f, ",")?;
                }
                write!(
                    f,
                    "{}/hwnd=0x{:08x}/msg=0x{:08x}/w=0x{:08x}/l=0x{:08x}/wndproc=0x{:08x}/return_pc=0x{:08x}",
                    trace.source,
                    trace.hwnd,
                    trace.msg,
                    trace.wparam,
                    trace.lparam,
                    trace.wndproc,
                    trace.return_pc
                )?;
                if let Some(origin) = trace.return_pc_trampoline_origin {
                    write!(f, "/return_tramp_origin=0x{origin:08x}")?;
                }
                write!(f, "/ret=0x{:08x}", trace.result)?;
                if let Some(class_name) = trace.class_name.as_deref() {
                    write!(f, "/class={class_name}")?;
                }
                write!(f, "/calls=[")?;
                for (call_index, call) in trace.calls.iter().enumerate() {
                    if call_index != 0 {
                        write!(f, ";")?;
                    }
                    write!(
                        f,
                        "0x{:08x}->0x{:08x}/{}/ra=0x{:08x}/sp=0x{:08x}",
                        call.pc, call.target, call.kind, call.ra, call.sp
                    )?;
                    write_call_target_import(f, call)?;
                    write!(
                        f,
                        "/a0=0x{:08x}/a1=0x{:08x}/a2=0x{:08x}/a3=0x{:08x}",
                        call.a0, call.a1, call.a2, call.a3
                    )?;
                }
                write!(f, "]")?;
                write!(f, "/imports=[")?;
                for (import_index, import) in trace.imports.iter().enumerate() {
                    if import_index != 0 {
                        write!(f, ";")?;
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
                    if let Some(detail) = import.detail.as_deref() {
                        write!(f, "/detail={}", format_trace_string(detail))?;
                    }
                }
                write!(f, "]")?;
                write!(f, "/code=[")?;
                for (code_index, code) in trace.code.iter().enumerate() {
                    if code_index != 0 {
                        write!(f, ";")?;
                    }
                    write!(
                        f,
                        "0x{:08x}/ra=0x{:08x}/sp=0x{:08x}/v0=0x{:08x}",
                        code.pc, code.ra, code.sp, code.v0
                    )?;
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
                    }
                }
                write!(f, "]")?;
                write!(f, "/probes=[")?;
                for (code_index, code) in trace.readiness_code.iter().enumerate() {
                    if code_index != 0 {
                        write!(f, ";")?;
                    }
                    write!(
                        f,
                        "0x{:08x}/ra=0x{:08x}/sp=0x{:08x}/v0=0x{:08x}",
                        code.pc, code.ra, code.sp, code.v0
                    )?;
                    if let Some(origin) = code.current_trampoline_origin {
                        write!(f, "/tramp_origin=0x{origin:08x}")?;
                    }
                    if let Some(instruction) = code.instruction {
                        write!(f, "/insn=0x{instruction:08x}")?;
                    }
                    if let Some(next_instruction) = code.next_instruction {
                        write!(f, "/next=0x{next_instruction:08x}")?;
                    }
                }
                write!(f, "]")?;
            }
            write!(f, "]")?;
        }
        if !self.last_mfc_dispatch.is_empty() {
            write!(f, " last_mfc_dispatch=[")?;
            for (index, trace) in self.last_mfc_dispatch.iter().enumerate() {
                if index != 0 {
                    write!(f, ",")?;
                }
                write!(
                    f,
                    "{}@0x{:08x}/ra=0x{:08x}/sp=0x{:08x}/v0=0x{:08x}/a0=0x{:08x}/a1=0x{:08x}/a2=0x{:08x}/a3=0x{:08x}/s5=0x{:08x}/s6=0x{:08x}/s7=0x{:08x}/fp=0x{:08x}",
                    trace.label,
                    trace.pc,
                    trace.ra,
                    trace.sp,
                    trace.v0,
                    trace.a0,
                    trace.a1,
                    trace.a2,
                    trace.a3,
                    trace.s5,
                    trace.s6,
                    trace.s7,
                    trace.fp
                )?;
                if let Some(sp10) = trace.sp10 {
                    write!(f, "/sp10=0x{sp10:08x}")?;
                }
                if let Some(this_ptr) = trace.this_ptr {
                    write!(f, "/this=0x{this_ptr:08x}")?;
                }
                if let Some(hwnd) = trace.hwnd {
                    write!(f, "/hwnd=0x{hwnd:08x}")?;
                }
                if let Some(msg) = trace.msg {
                    write!(f, "/msg=0x{msg:08x}")?;
                }
                if let Some(wparam) = trace.wparam {
                    write!(f, "/w=0x{wparam:08x}")?;
                }
                if let Some(lparam) = trace.lparam {
                    write!(f, "/l=0x{lparam:08x}")?;
                }
                if let Some(slot) = trace.vtable_slot_98 {
                    write!(f, "/vtable98=0x{slot:08x}")?;
                }
            }
            write!(f, "]")?;
        }
        if !self.last_inavi_display.is_empty() {
            write!(f, " last_inavi_display=[")?;
            for (index, trace) in self.last_inavi_display.iter().enumerate() {
                if index != 0 {
                    write!(f, ",")?;
                }
                write!(
                    f,
                    "{}@0x{:08x}/ra=0x{:08x}/sp=0x{:08x}/v0=0x{:08x}/a0=0x{:08x}/a1=0x{:08x}/a2=0x{:08x}/a3=0x{:08x}",
                    trace.label,
                    trace.pc,
                    trace.ra,
                    trace.sp,
                    trace.v0,
                    trace.a0,
                    trace.a1,
                    trace.a2,
                    trace.a3
                )?;
                if let Some(this_ptr) = trace.this_ptr {
                    write!(f, "/this=0x{this_ptr:08x}")?;
                }
                if let Some(hwnd) = trace.hwnd {
                    write!(f, "/hwnd=0x{hwnd:08x}")?;
                }
                if let Some(msg) = trace.msg {
                    write!(f, "/msg=0x{msg:08x}")?;
                }
                if let Some(wparam) = trace.wparam {
                    write!(f, "/w=0x{wparam:08x}")?;
                }
                if let Some(lparam) = trace.lparam {
                    write!(f, "/l=0x{lparam:08x}")?;
                }
                if let Some(field_20) = trace.field_20 {
                    write!(f, "/field20=0x{field_20:08x}")?;
                }
                if let Some(field_44) = trace.field_44 {
                    write!(f, "/field44=0x{field_44:08x}")?;
                }
                if let Some(field_e8) = trace.field_e8 {
                    write!(f, "/fielde8=0x{field_e8:08x}")?;
                }
            }
            write!(f, "]")?;
        }
        if !self.inavi_render_milestones.is_empty() {
            write!(f, " inavi_render_milestones=[")?;
            write_inavi_controller_traces(f, &self.inavi_render_milestones)?;
            write!(f, "]")?;
        }
        if !self.last_inavi_controller.is_empty() {
            write!(f, " last_inavi_controller=[")?;
            write_inavi_controller_traces(f, &self.last_inavi_controller)?;
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
                    "0x{:08x}/ra=0x{:08x}/sp=0x{:08x}/v0=0x{:08x}",
                    code.pc, code.ra, code.sp, code.v0
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
            write!(f, " queue_status=0x{:08x}", blocked.queue_status)?;
            if let Some(delay_ms) = blocked.next_timer_due_ms {
                write!(f, " next_timer_due_ms={delay_ms}")?;
            }
            if !blocked.timers.is_empty() {
                write!(f, " timers=[")?;
                for (index, timer) in blocked.timers.iter().enumerate() {
                    if index != 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "id=0x{:08x}", timer.id)?;
                    if let Some(hwnd) = timer.hwnd {
                        write!(f, "/hwnd=0x{hwnd:08x}")?;
                    }
                    write!(f, "/msg=0x{:08x}/due={}", timer.message, timer.due_ms)?;
                    if let Some(period_ms) = timer.period_ms {
                        write!(f, "/period={period_ms}")?;
                    }
                }
                write!(f, "]")?;
            }
            if !blocked.z_order.is_empty() {
                write!(f, " z_order=[")?;
                for (index, hwnd) in blocked.z_order.iter().enumerate() {
                    if index != 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "0x{hwnd:08x}")?;
                }
                write!(f, "]")?;
            }
            if !blocked.windows.is_empty() {
                write!(f, " windows=[")?;
                for (index, window) in blocked.windows.iter().enumerate() {
                    if index != 0 {
                        write!(f, ";")?;
                    }
                    write!(
                        f,
                        "0x{:08x}/tid={}/pid={}/class={}/title={}/vis={}/dead={}/upd={}/erase={}/rect={},{}-{},{}",
                        window.hwnd,
                        window.thread_id,
                        window.process_id,
                        format_trace_string(&window.class_name),
                        format_trace_string(&window.title),
                        window.visible,
                        window.destroyed,
                        window.update_pending,
                        window.erase_pending,
                        window.rect.left,
                        window.rect.top,
                        window.rect.right,
                        window.rect.bottom,
                    )?;
                    if window.wndproc != 0 {
                        write!(f, "/wndproc=0x{:08x}", window.wndproc)?;
                    }
                }
                write!(f, "]")?;
            }
            if !blocked.queues.is_empty() {
                write!(f, " queues=[")?;
                for (queue_index, queue) in blocked.queues.iter().enumerate() {
                    if queue_index != 0 {
                        write!(f, ";")?;
                    }
                    write!(f, "tid={}/msgs=", queue.thread_id)?;
                    write!(f, "[")?;
                    for (message_index, message) in queue.messages.iter().enumerate() {
                        if message_index != 0 {
                            write!(f, ",")?;
                        }
                        write!(
                            f,
                            "0x{:08x}:0x{:08x}/w=0x{:08x}/l=0x{:08x}",
                            message.hwnd, message.msg, message.wparam, message.lparam
                        )?;
                    }
                    write!(f, "]")?;
                }
                write!(f, "]")?;
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

fn write_call_target_import(
    f: &mut std::fmt::Formatter<'_>,
    call: &UnicornLastCall,
) -> std::fmt::Result {
    if let Some(module_kind) = call.target_module_kind {
        write!(f, "/target_module={module_kind:?}")?;
    }
    if let Some(module_name) = call.target_module_name.as_deref() {
        write!(f, "/target_dll={module_name}")?;
    }
    if let Some(ordinal) = call.target_ordinal {
        write!(f, "/target_ord={ordinal}")?;
    }
    if let Some(name) = call.target_name.as_deref() {
        write!(f, "/target_name={name}")?;
    }
    Ok(())
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

#[cfg(all(feature = "unicorn", feature = "trace"))]
fn read_unicorn_u8<D>(uc: &unicorn_engine::Unicorn<'_, D>, address: u32) -> Option<u8> {
    let mut bytes = [0; 1];
    uc.mem_read(u64::from(address), &mut bytes)
        .ok()
        .map(|()| bytes[0])
}

#[cfg(all(feature = "unicorn", feature = "trace"))]
fn read_unicorn_i16<D>(uc: &unicorn_engine::Unicorn<'_, D>, address: u32) -> Option<i16> {
    let mut bytes = [0; 2];
    uc.mem_read(u64::from(address), &mut bytes)
        .ok()
        .map(|()| i16::from_le_bytes(bytes))
}

#[cfg(feature = "unicorn")]
fn read_unicorn_code_u32<D>(
    uc: &unicorn_engine::Unicorn<'_, D>,
    mapped_code: &MappedCodeIndex,
    address: u32,
) -> Option<u32> {
    mapped_code
        .read_u32(address)
        .or_else(|| read_unicorn_u32(uc, address))
}

#[cfg(feature = "unicorn")]
fn is_trampoline_sentinel_first_word(instruction: u32) -> bool {
    let opcode = instruction >> 26;
    let rt = (instruction >> 16) & 0x1f;
    opcode == 0x0f && rt == 26
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
    kernel.drain_remote_input_to_thread_window(thread_id, hwnd);
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

    const MAX_GET_MESSAGE_TIMER_WAIT_MS: u32 = 5_000;
    if let Some(delay_ms) = kernel.timers.next_due_delay_ms() {
        if delay_ms <= MAX_GET_MESSAGE_TIMER_WAIT_MS {
            if delay_ms != 0 {
                kernel.timers.sleep_ms(delay_ms);
            }
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
        }
    }

    *blocked.borrow_mut() = Some(UnicornBlockedGetMessage {
        thread_id,
        hwnd,
        min_msg,
        max_msg,
        queue_status: kernel.gwe.peek_queue_status(thread_id, u32::MAX),
        next_timer_due_ms: kernel.timers.next_due_delay_ms(),
        timers: kernel
            .timers
            .pending_timers()
            .into_iter()
            .map(|timer| UnicornTimerSnapshot {
                id: timer.id,
                hwnd: timer.hwnd,
                message: timer.message,
                due_ms: timer.due_ms,
                period_ms: timer.period_ms,
            })
            .collect(),
        z_order: kernel.gwe.z_order_snapshot(),
        windows: kernel
            .gwe
            .windows_snapshot()
            .into_iter()
            .map(|window| UnicornWindowSnapshot {
                hwnd: window.hwnd,
                thread_id: window.thread_id,
                process_id: window.process_id,
                class_name: window.class_name,
                title: window.title,
                visible: window.visible,
                destroyed: window.destroyed,
                update_pending: window.update_pending,
                erase_pending: window.erase_pending,
                rect: window.rect,
                wndproc: window.wndproc,
            })
            .collect(),
        queues: kernel
            .gwe
            .queue_snapshot()
            .into_iter()
            .map(|(thread_id, messages)| UnicornQueueSnapshot {
                thread_id,
                messages: messages
                    .into_iter()
                    .map(|message| UnicornMessageRecord {
                        hwnd: message.hwnd,
                        msg: message.msg,
                        wparam: message.wparam,
                        lparam: message.lparam,
                        time_ms: message.time_ms,
                    })
                    .collect(),
            })
            .collect(),
    });
    record_message_import(uc, module_kind, ordinal, args, None, last_messages);

    let mut pending_returns = pending_returns.borrow_mut();
    if let Some(callout) = pending_returns.pop() {
        let wait_id = kernel.register_blocked_waiter(
            thread_id,
            callout.thread_handle,
            Vec::new(),
            crate::ce::scheduler::SchedulerBlockedWaitKind::GetMessage {
                hwnd,
                min_msg,
                max_msg,
            },
            kernel.timers.tick_count(),
            crate::ce::timer::INFINITE,
        );
        *blocked_thread.borrow_mut() = Some(BlockedGuestThread {
            wait_id,
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
        let _ = update_user_kdata_current_ids(
            uc,
            callout.creator_thread_id,
            kernel.current_process_id(),
        );
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
        let wait_id = kernel.register_blocked_waiter(
            thread_id,
            thread_handle,
            Vec::new(),
            crate::ce::scheduler::SchedulerBlockedWaitKind::GetMessage {
                hwnd,
                min_msg,
                max_msg,
            },
            kernel.timers.tick_count(),
            crate::ce::timer::INFINITE,
        );
        *blocked_thread.borrow_mut() = Some(BlockedGuestThread {
            wait_id,
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
            activate_suspended_thread(uc, kernel, current_thread_id, running_thread, &suspended);
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
    kernel.record_blocked_single_wait(timeout);
    let wait_started_ms = kernel.timers.tick_count();

    let regs = capture_mips_gprs(uc);
    let return_pc = read_mips_reg(uc, RegisterMIPS::RA);
    if let Some(callout) = pending_returns.borrow_mut().pop() {
        let kind = BlockedWaitKind::Kernel;
        let wait_handles = vec![wait_handle];
        let wait_id = kernel.register_blocked_waiter(
            thread_id,
            callout.thread_handle,
            wait_handles.clone(),
            scheduler_blocked_wait_kind(kind),
            wait_started_ms,
            timeout,
        );
        blocked_waits.borrow_mut().push(BlockedWaitThread {
            wait_id,
            thread_id,
            thread_handle: callout.thread_handle,
            wait_handles,
            kind,
            wait_started_ms,
            timeout_ms: timeout,
            regs,
            return_pc,
        });
        *running_thread.borrow_mut() = None;
        *current_thread_id.borrow_mut() = callout.creator_thread_id;
        let _ = update_user_kdata_current_ids(
            uc,
            callout.creator_thread_id,
            kernel.current_process_id(),
        );
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
        let kind = BlockedWaitKind::Kernel;
        let wait_handles = vec![wait_handle];
        let wait_id = kernel.register_blocked_waiter(
            thread_id,
            thread_handle,
            wait_handles.clone(),
            scheduler_blocked_wait_kind(kind),
            wait_started_ms,
            timeout,
        );
        blocked_waits.borrow_mut().push(BlockedWaitThread {
            wait_id,
            thread_id,
            thread_handle,
            wait_handles,
            kind,
            wait_started_ms,
            timeout_ms: timeout,
            regs,
            return_pc,
        });
        *running_thread.borrow_mut() = None;
        if let Some(suspended) = suspended_thread.borrow_mut().take() {
            activate_suspended_thread(uc, kernel, current_thread_id, running_thread, &suspended);
            return true;
        }
    }
    false
}

#[cfg(feature = "unicorn")]
fn try_block_sleep<D>(
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
    self_suspended_threads: &std::rc::Rc<
        std::cell::RefCell<std::collections::BTreeMap<u32, SuspendedGuestThread>>,
    >,
    running_thread: &std::rc::Rc<std::cell::RefCell<Option<(u32, u32)>>>,
) -> bool {
    use unicorn_engine::RegisterMIPS;

    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll {
        return false;
    }

    let timeout = match ordinal {
        Some(crate::ce::coredll_ordinals::ORD_SLEEP) => {
            match crate::ce::timer::ce_sleep_request(args.first().copied().unwrap_or(0)) {
                crate::ce::timer::CeSleepRequest::Bounded(timeout) => timeout,
                crate::ce::timer::CeSleepRequest::Yield => {
                    return try_yield_sleep(
                        kernel,
                        uc,
                        thread_id,
                        pending_returns,
                        current_thread_id,
                        suspended_thread,
                        running_thread,
                    );
                }
                crate::ce::timer::CeSleepRequest::Suspend => {
                    return try_suspend_sleep(
                        kernel,
                        uc,
                        thread_id,
                        pending_returns,
                        current_thread_id,
                        suspended_thread,
                        self_suspended_threads,
                        running_thread,
                    );
                }
            }
        }
        Some(crate::ce::coredll_ordinals::ORD_SLEEP_TILL_TICK) => 1,
        _ => return false,
    };

    kernel.record_blocked_thread_sleep(timeout);
    let wait_started_ms = kernel.timers.tick_count();
    let regs = capture_mips_gprs(uc);
    let return_pc = read_mips_reg(uc, RegisterMIPS::RA);
    let kind = BlockedWaitKind::Sleep;

    if let Some(callout) = pending_returns.borrow_mut().pop() {
        let wait_id = kernel.register_blocked_waiter(
            thread_id,
            callout.thread_handle,
            Vec::new(),
            scheduler_blocked_wait_kind(kind),
            wait_started_ms,
            timeout,
        );
        blocked_waits.borrow_mut().push(BlockedWaitThread {
            wait_id,
            thread_id,
            thread_handle: callout.thread_handle,
            wait_handles: Vec::new(),
            kind,
            wait_started_ms,
            timeout_ms: timeout,
            regs,
            return_pc,
        });
        *running_thread.borrow_mut() = None;
        *current_thread_id.borrow_mut() = callout.creator_thread_id;
        let _ = update_user_kdata_current_ids(
            uc,
            callout.creator_thread_id,
            kernel.current_process_id(),
        );
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
        let wait_id = kernel.register_blocked_waiter(
            thread_id,
            thread_handle,
            Vec::new(),
            scheduler_blocked_wait_kind(kind),
            wait_started_ms,
            timeout,
        );
        blocked_waits.borrow_mut().push(BlockedWaitThread {
            wait_id,
            thread_id,
            thread_handle,
            wait_handles: Vec::new(),
            kind,
            wait_started_ms,
            timeout_ms: timeout,
            regs,
            return_pc,
        });
        *running_thread.borrow_mut() = None;
        if let Some(suspended) = suspended_thread.borrow_mut().take() {
            activate_suspended_thread(uc, kernel, current_thread_id, running_thread, &suspended);
            return true;
        }
    }
    false
}

#[cfg(feature = "unicorn")]
fn try_suspend_sleep<D>(
    kernel: &mut CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    thread_id: u32,
    pending_returns: &std::rc::Rc<std::cell::RefCell<Vec<PendingGuestThreadReturn>>>,
    current_thread_id: &std::rc::Rc<std::cell::RefCell<u32>>,
    suspended_thread: &std::rc::Rc<std::cell::RefCell<Option<SuspendedGuestThread>>>,
    self_suspended_threads: &std::rc::Rc<
        std::cell::RefCell<std::collections::BTreeMap<u32, SuspendedGuestThread>>,
    >,
    running_thread: &std::rc::Rc<std::cell::RefCell<Option<(u32, u32)>>>,
) -> bool {
    use unicorn_engine::RegisterMIPS;

    let capture = |thread_handle| {
        let mut suspended = SuspendedGuestThread {
            thread_id,
            thread_handle: Some(thread_handle),
            regs: capture_mips_gprs(uc),
            pc: read_mips_reg(uc, RegisterMIPS::RA),
        };
        suspended.regs[2] = 0;
        self_suspended_threads
            .borrow_mut()
            .insert(thread_handle, suspended);
    };

    if let Some(callout) = pending_returns.borrow_mut().pop() {
        if !matches!(
            kernel.suspend_thread(callout.thread_handle),
            crate::ce::object::ThreadSuspendResult::Previous(_)
        ) {
            return false;
        }
        capture(callout.thread_handle);

        let mut creator_regs = callout.creator_regs;
        creator_regs[2] = callout.thread_handle;
        restore_mips_gprs(uc, &creator_regs);
        *current_thread_id.borrow_mut() = callout.creator_thread_id;
        let _ = update_user_kdata_current_ids(
            uc,
            callout.creator_thread_id,
            kernel.current_process_id(),
        );
        *running_thread.borrow_mut() = None;
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

    let Some((_, thread_handle)) = *running_thread.borrow() else {
        return false;
    };
    if !matches!(
        kernel.suspend_thread(thread_handle),
        crate::ce::object::ThreadSuspendResult::Previous(_)
    ) {
        return false;
    }
    capture(thread_handle);
    *running_thread.borrow_mut() = None;
    if let Some(suspended) = suspended_thread.borrow_mut().take() {
        activate_suspended_thread(uc, kernel, current_thread_id, running_thread, &suspended);
    } else {
        let _ = uc.emu_stop();
    }
    true
}

#[cfg(feature = "unicorn")]
fn try_yield_sleep<D>(
    kernel: &mut CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    thread_id: u32,
    pending_returns: &std::rc::Rc<std::cell::RefCell<Vec<PendingGuestThreadReturn>>>,
    current_thread_id: &std::rc::Rc<std::cell::RefCell<u32>>,
    suspended_thread: &std::rc::Rc<std::cell::RefCell<Option<SuspendedGuestThread>>>,
    running_thread: &std::rc::Rc<std::cell::RefCell<Option<(u32, u32)>>>,
) -> bool {
    use unicorn_engine::RegisterMIPS;

    let mut yielding = SuspendedGuestThread {
        thread_id,
        thread_handle: running_thread
            .borrow()
            .and_then(|(id, handle)| (id == thread_id).then_some(handle)),
        regs: capture_mips_gprs(uc),
        pc: read_mips_reg(uc, RegisterMIPS::RA),
    };
    yielding.regs[2] = 0;

    if let Some(callout) = pending_returns.borrow_mut().pop() {
        kernel.record_thread_yield();
        yielding.thread_handle = Some(callout.thread_handle);
        *suspended_thread.borrow_mut() = Some(yielding);

        let mut creator_regs = callout.creator_regs;
        creator_regs[2] = callout.thread_handle;
        restore_mips_gprs(uc, &creator_regs);
        *current_thread_id.borrow_mut() = callout.creator_thread_id;
        let _ = update_user_kdata_current_ids(
            uc,
            callout.creator_thread_id,
            kernel.current_process_id(),
        );
        *running_thread.borrow_mut() = None;
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

    let Some(resume) = suspended_thread.borrow_mut().take() else {
        return false;
    };

    kernel.record_thread_yield();
    *suspended_thread.borrow_mut() = Some(yielding);
    activate_suspended_thread(uc, kernel, current_thread_id, running_thread, &resume);
    true
}

#[cfg(feature = "unicorn")]
fn try_block_wait_for_multiple_objects<D>(
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
        || ordinal != Some(crate::ce::coredll_ordinals::ORD_WAIT_FOR_MULTIPLE_OBJECTS)
    {
        return false;
    }

    let count = args.first().copied().unwrap_or(0);
    let handles_ptr = args.get(1).copied().unwrap_or(0);
    let wait_all = args.get(2).copied().unwrap_or(0) != 0;
    let timeout = args.get(3).copied().unwrap_or(0);
    if count == 0 || count > 64 || handles_ptr == 0 || wait_all || timeout == 0 {
        return false;
    }

    let mut wait_handles = Vec::with_capacity(count as usize);
    for index in 0..count {
        let Some(handle) = read_unicorn_u32(uc, handles_ptr.wrapping_add(index * 4)) else {
            return false;
        };
        wait_handles.push(handle);
    }
    let mut all_waitable = true;
    let mut any_ready = false;
    for handle in &wait_handles {
        match kernel.is_wait_ready(*handle, thread_id) {
            Some(true) => any_ready = true,
            Some(false) => {}
            None => {
                all_waitable = false;
                break;
            }
        }
    }
    if !all_waitable || any_ready {
        return false;
    }

    kernel.record_blocked_multiple_wait(count, timeout);
    let wait_started_ms = kernel.timers.tick_count();
    let regs = capture_mips_gprs(uc);
    let return_pc = read_mips_reg(uc, RegisterMIPS::RA);
    if let Some(callout) = pending_returns.borrow_mut().pop() {
        let kind = BlockedWaitKind::Kernel;
        let wait_id = kernel.register_blocked_waiter(
            thread_id,
            callout.thread_handle,
            wait_handles.clone(),
            scheduler_blocked_wait_kind(kind),
            wait_started_ms,
            timeout,
        );
        blocked_waits.borrow_mut().push(BlockedWaitThread {
            wait_id,
            thread_id,
            thread_handle: callout.thread_handle,
            wait_handles,
            kind,
            wait_started_ms,
            timeout_ms: timeout,
            regs,
            return_pc,
        });
        *running_thread.borrow_mut() = None;
        *current_thread_id.borrow_mut() = callout.creator_thread_id;
        let _ = update_user_kdata_current_ids(
            uc,
            callout.creator_thread_id,
            kernel.current_process_id(),
        );
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
        let kind = BlockedWaitKind::Kernel;
        let wait_id = kernel.register_blocked_waiter(
            thread_id,
            thread_handle,
            wait_handles.clone(),
            scheduler_blocked_wait_kind(kind),
            wait_started_ms,
            timeout,
        );
        blocked_waits.borrow_mut().push(BlockedWaitThread {
            wait_id,
            thread_id,
            thread_handle,
            wait_handles,
            kind,
            wait_started_ms,
            timeout_ms: timeout,
            regs,
            return_pc,
        });
        *running_thread.borrow_mut() = None;
        if let Some(suspended) = suspended_thread.borrow_mut().take() {
            activate_suspended_thread(uc, kernel, current_thread_id, running_thread, &suspended);
            return true;
        }
    }
    false
}

#[cfg(feature = "unicorn")]
fn try_block_msg_wait_for_multiple_objects_ex<D>(
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

    const MWMO_WAITALL: u32 = 0x0001;
    const MWMO_INPUTAVAILABLE: u32 = 0x0004;
    const MAXIMUM_WAIT_OBJECTS: u32 = 64;

    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll
        || ordinal != Some(crate::ce::coredll_ordinals::ORD_MSG_WAIT_FOR_MULTIPLE_OBJECTS_EX)
    {
        return false;
    }

    let count = args.first().copied().unwrap_or(0);
    let handles_ptr = args.get(1).copied().unwrap_or(0);
    let timeout = args.get(2).copied().unwrap_or(0);
    let wake_mask = args.get(3).copied().unwrap_or(0);
    let flags = args.get(4).copied().unwrap_or(0);
    let input_available = flags & MWMO_INPUTAVAILABLE != 0;
    if count > MAXIMUM_WAIT_OBJECTS
        || (count != 0 && handles_ptr == 0)
        || flags & MWMO_WAITALL != 0
        || timeout == 0
    {
        return false;
    }

    let mut wait_handles = Vec::with_capacity(count as usize);
    for index in 0..count {
        let Some(handle) = read_unicorn_u32(uc, handles_ptr.wrapping_add(index * 4)) else {
            return false;
        };
        wait_handles.push(handle);
    }

    let mut all_waitable = true;
    let mut any_ready = false;
    for handle in &wait_handles {
        match kernel.is_wait_ready(*handle, thread_id) {
            Some(true) => any_ready = true,
            Some(false) => {}
            None => {
                all_waitable = false;
                break;
            }
        }
    }
    if !all_waitable || any_ready {
        return false;
    }

    kernel.pump_timers_to_gwe(thread_id);
    let has_input = if input_available {
        kernel.gwe.has_queue_input(thread_id, wake_mask)
    } else {
        kernel.gwe.has_new_queue_input(thread_id, wake_mask)
    };
    if has_input {
        return false;
    }

    kernel.record_blocked_msg_wait(count, timeout);
    let wait_started_ms = kernel.timers.tick_count();
    let regs = capture_mips_gprs(uc);
    let return_pc = read_mips_reg(uc, RegisterMIPS::RA);
    let kind = BlockedWaitKind::MsgWait {
        wake_mask,
        input_available,
    };
    if let Some(callout) = pending_returns.borrow_mut().pop() {
        let wait_id = kernel.register_blocked_waiter(
            thread_id,
            callout.thread_handle,
            wait_handles.clone(),
            scheduler_blocked_wait_kind(kind),
            wait_started_ms,
            timeout,
        );
        blocked_waits.borrow_mut().push(BlockedWaitThread {
            wait_id,
            thread_id,
            thread_handle: callout.thread_handle,
            wait_handles,
            kind,
            wait_started_ms,
            timeout_ms: timeout,
            regs,
            return_pc,
        });
        *running_thread.borrow_mut() = None;
        *current_thread_id.borrow_mut() = callout.creator_thread_id;
        let _ = update_user_kdata_current_ids(
            uc,
            callout.creator_thread_id,
            kernel.current_process_id(),
        );
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
        let wait_id = kernel.register_blocked_waiter(
            thread_id,
            thread_handle,
            wait_handles.clone(),
            scheduler_blocked_wait_kind(kind),
            wait_started_ms,
            timeout,
        );
        blocked_waits.borrow_mut().push(BlockedWaitThread {
            wait_id,
            thread_id,
            thread_handle,
            wait_handles,
            kind,
            wait_started_ms,
            timeout_ms: timeout,
            regs,
            return_pc,
        });
        *running_thread.borrow_mut() = None;
        if let Some(suspended) = suspended_thread.borrow_mut().take() {
            activate_suspended_thread(uc, kernel, current_thread_id, running_thread, &suspended);
            return true;
        }
    }
    false
}

#[cfg(feature = "unicorn")]
fn try_block_serial_read_file<D>(
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
        || ordinal != Some(crate::ce::coredll_ordinals::ORD_READ_FILE)
    {
        return false;
    }

    let handle = args.first().copied().unwrap_or(0);
    let buffer = args.get(1).copied().unwrap_or(0);
    let requested = args.get(2).copied().unwrap_or(0);
    let transferred_ptr = args.get(3).copied().unwrap_or(0);
    if requested == 0
        || buffer == 0
        || !kernel.is_serial_device_handle(handle)
        || kernel.serial_read_ready(handle)
    {
        return false;
    }

    kernel.record_blocked_single_wait(crate::ce::timer::INFINITE);
    let wait_started_ms = kernel.timers.tick_count();
    let kind = BlockedWaitKind::SerialRead {
        handle,
        buffer,
        requested,
        transferred_ptr,
    };
    let regs = capture_mips_gprs(uc);
    let return_pc = read_mips_reg(uc, RegisterMIPS::RA);

    if let Some(callout) = pending_returns.borrow_mut().pop() {
        let wait_id = kernel.register_blocked_waiter(
            thread_id,
            callout.thread_handle,
            Vec::new(),
            scheduler_blocked_wait_kind(kind),
            wait_started_ms,
            crate::ce::timer::INFINITE,
        );
        blocked_waits.borrow_mut().push(BlockedWaitThread {
            wait_id,
            thread_id,
            thread_handle: callout.thread_handle,
            wait_handles: Vec::new(),
            kind,
            wait_started_ms,
            timeout_ms: crate::ce::timer::INFINITE,
            regs,
            return_pc,
        });
        *running_thread.borrow_mut() = None;
        *current_thread_id.borrow_mut() = callout.creator_thread_id;
        let _ = update_user_kdata_current_ids(
            uc,
            callout.creator_thread_id,
            kernel.current_process_id(),
        );
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
        let wait_id = kernel.register_blocked_waiter(
            thread_id,
            thread_handle,
            Vec::new(),
            scheduler_blocked_wait_kind(kind),
            wait_started_ms,
            crate::ce::timer::INFINITE,
        );
        blocked_waits.borrow_mut().push(BlockedWaitThread {
            wait_id,
            thread_id,
            thread_handle,
            wait_handles: Vec::new(),
            kind,
            wait_started_ms,
            timeout_ms: crate::ce::timer::INFINITE,
            regs,
            return_pc,
        });
        *running_thread.borrow_mut() = None;
        if let Some(suspended) = suspended_thread.borrow_mut().take() {
            activate_suspended_thread(uc, kernel, current_thread_id, running_thread, &suspended);
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

    let blocked_msg_wait_thread_ids: Vec<u32> = blocked_waits
        .borrow()
        .iter()
        .filter(|blocked| matches!(blocked.kind, BlockedWaitKind::MsgWait { .. }))
        .map(|blocked| blocked.thread_id)
        .collect();
    for thread_id in blocked_msg_wait_thread_ids {
        kernel.pump_timers_to_gwe(thread_id);
    }
    kernel.expire_timed_out_send_messages();

    let now_ms = kernel.timers.tick_count();
    let Some(ready_wait_id) =
        kernel.select_ready_blocked_waiter(active_thread_id, now_ms, |blocked, kernel| {
            scheduler_blocked_wait_is_ready(blocked, kernel)
        })
    else {
        return false;
    };
    let Some(index) = blocked_waits
        .borrow()
        .iter()
        .position(|blocked| blocked.wait_id == ready_wait_id)
    else {
        return false;
    };
    let blocked = blocked_waits.borrow_mut().remove(index);
    let _ = kernel.remove_blocked_waiter(blocked.wait_id);
    let has_ready_handle = blocked_wait_has_ready_handle(&blocked, kernel);
    let serial_read_ready = blocked_serial_read_ready(&blocked, kernel);
    let send_message_ready = blocked_send_message_ready(&blocked, kernel);
    let message_input_ready =
        !has_ready_handle && !serial_read_ready && blocked_msg_wait_has_input(&blocked, kernel);
    let sleep_timed_out = matches!(blocked.kind, BlockedWaitKind::Sleep)
        && blocked_wait_timed_out(&blocked, kernel.timers.tick_count());
    let wait_result = if sleep_timed_out {
        0
    } else if has_ready_handle {
        if blocked.wait_handles.len() == 1 {
            kernel.wait_for_single_object_without_scheduler_record(
                blocked.wait_handles[0],
                0,
                blocked.thread_id,
            )
        } else {
            kernel.wait_for_multiple_objects_without_scheduler_record(
                &blocked.wait_handles,
                false,
                blocked.thread_id,
            )
        }
    } else if message_input_ready {
        consume_blocked_msg_wait_input(&blocked, kernel);
        crate::ce::timer::WAIT_OBJECT_0 + blocked.wait_handles.len() as u32
    } else if serial_read_ready {
        complete_blocked_serial_read(&blocked, kernel, uc)
    } else if send_message_ready {
        complete_blocked_send_message(&blocked, kernel, uc)
    } else if blocked_wait_timed_out(&blocked, kernel.timers.tick_count()) {
        crate::ce::timer::WAIT_TIMEOUT
    } else {
        crate::ce::timer::WAIT_TIMEOUT
    };
    match blocked.kind {
        BlockedWaitKind::Kernel => kernel.record_resumed_wait(wait_result),
        BlockedWaitKind::Sleep => kernel.record_resumed_thread_sleep(),
        BlockedWaitKind::MsgWait { .. } if message_input_ready => {
            kernel.record_resumed_msg_wait_input();
        }
        BlockedWaitKind::MsgWait { .. } => kernel.record_resumed_msg_wait_result(wait_result),
        BlockedWaitKind::SerialRead { .. } => kernel.record_resumed_wait(if wait_result != 0 {
            crate::ce::timer::WAIT_OBJECT_0
        } else {
            crate::ce::timer::WAIT_FAILED
        }),
        BlockedWaitKind::SendMessage { .. } => kernel.record_resumed_wait(if send_message_ready {
            crate::ce::timer::WAIT_OBJECT_0
        } else {
            crate::ce::timer::WAIT_TIMEOUT
        }),
    }

    let mut current = SuspendedGuestThread {
        thread_id: active_thread_id,
        thread_handle: running_thread
            .borrow()
            .and_then(|(id, handle)| (id == active_thread_id).then_some(handle)),
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
    let _ = update_user_kdata_current_ids(uc, blocked.thread_id, kernel.current_process_id());
    *running_thread.borrow_mut() = match blocked.kind {
        BlockedWaitKind::SendMessage {
            previous_running_thread,
            ..
        } => previous_running_thread,
        _ => Some((blocked.thread_id, blocked.thread_handle)),
    };
    true
}

#[cfg(feature = "unicorn")]
fn blocked_wait_timed_out(blocked: &BlockedWaitThread, now_ms: u32) -> bool {
    blocked.timeout_ms != crate::ce::timer::INFINITE
        && now_ms.wrapping_sub(blocked.wait_started_ms) >= blocked.timeout_ms
}

#[cfg(feature = "unicorn")]
fn blocked_wait_has_ready_handle(blocked: &BlockedWaitThread, kernel: &CeKernel) -> bool {
    blocked
        .wait_handles
        .iter()
        .any(|handle| kernel.is_wait_ready(*handle, blocked.thread_id) == Some(true))
}

#[cfg(feature = "unicorn")]
fn blocked_serial_read_ready(blocked: &BlockedWaitThread, kernel: &CeKernel) -> bool {
    match blocked.kind {
        BlockedWaitKind::SerialRead { handle, .. } => kernel.serial_read_ready(handle),
        _ => false,
    }
}

#[cfg(feature = "unicorn")]
fn blocked_send_message_ready(blocked: &BlockedWaitThread, kernel: &CeKernel) -> bool {
    match blocked.kind {
        BlockedWaitKind::SendMessage { send_id, .. } => kernel.sent_message_result_ready(send_id),
        _ => false,
    }
}

#[cfg(feature = "unicorn")]
fn complete_blocked_send_message<D>(
    blocked: &BlockedWaitThread,
    kernel: &mut CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
) -> u32 {
    let BlockedWaitKind::SendMessage {
        send_id,
        result_ptr,
        ..
    } = blocked.kind
    else {
        return 0;
    };
    let result = kernel
        .take_completed_send_message_result(send_id)
        .unwrap_or(0);
    if let Some(result_ptr) = result_ptr {
        let _ = uc.mem_write(u64::from(result_ptr), &result.to_le_bytes());
    }
    result
}

#[cfg(feature = "unicorn")]
fn complete_blocked_serial_read<D>(
    blocked: &BlockedWaitThread,
    kernel: &mut CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
) -> u32 {
    let BlockedWaitKind::SerialRead {
        handle,
        buffer,
        requested,
        transferred_ptr,
    } = blocked.kind
    else {
        return crate::ce::timer::WAIT_FAILED;
    };

    let mut memory = UnicornGuestMemory { uc };
    let mut cursor = buffer;
    let mut write_failed = false;
    let transferred = match kernel.read_file_into(handle, requested, |bytes| {
        if memory.write_bytes(cursor, bytes).is_err() {
            write_failed = true;
            return Err(Error::InvalidArgument(
                "serial ReadFile guest buffer is not writable".to_owned(),
            ));
        }
        cursor = cursor.wrapping_add(bytes.len() as u32);
        Ok(())
    }) {
        Ok(transferred) => transferred,
        Err(_) => {
            kernel.threads.set_last_error(
                blocked.thread_id,
                if write_failed {
                    crate::ce::thread::ERROR_INVALID_PARAMETER
                } else {
                    crate::ce::thread::ERROR_INVALID_HANDLE
                },
            );
            if transferred_ptr != 0 {
                let _ = memory.write_u32(transferred_ptr, 0);
            }
            return 0;
        }
    };
    if transferred_ptr != 0 && memory.write_u32(transferred_ptr, transferred).is_err() {
        kernel.threads.set_last_error(
            blocked.thread_id,
            crate::ce::thread::ERROR_INVALID_PARAMETER,
        );
        return 0;
    }
    kernel.threads.set_last_error(blocked.thread_id, 0);
    1
}

#[cfg(feature = "unicorn")]
fn scheduler_blocked_wait_has_ready_handle(
    blocked: &crate::ce::scheduler::SchedulerBlockedWait,
    kernel: &CeKernel,
) -> bool {
    blocked
        .wait_handles
        .iter()
        .any(|handle| kernel.is_wait_ready(*handle, blocked.thread_id) == Some(true))
}

#[cfg(feature = "unicorn")]
fn blocked_msg_wait_has_input(blocked: &BlockedWaitThread, kernel: &CeKernel) -> bool {
    match blocked.kind {
        BlockedWaitKind::Kernel => false,
        BlockedWaitKind::Sleep => false,
        BlockedWaitKind::SerialRead { .. } => false,
        BlockedWaitKind::SendMessage { .. } => false,
        BlockedWaitKind::MsgWait {
            wake_mask,
            input_available,
        } => {
            if input_available {
                kernel.gwe.has_queue_input(blocked.thread_id, wake_mask)
            } else {
                kernel.gwe.has_new_queue_input(blocked.thread_id, wake_mask)
            }
        }
    }
}

#[cfg(feature = "unicorn")]
fn scheduler_blocked_msg_wait_has_input(
    blocked: &crate::ce::scheduler::SchedulerBlockedWait,
    kernel: &CeKernel,
) -> bool {
    match blocked.kind {
        crate::ce::scheduler::SchedulerBlockedWaitKind::Kernel
        | crate::ce::scheduler::SchedulerBlockedWaitKind::Sleep
        | crate::ce::scheduler::SchedulerBlockedWaitKind::SerialRead { .. }
        | crate::ce::scheduler::SchedulerBlockedWaitKind::SendMessage { .. } => false,
        crate::ce::scheduler::SchedulerBlockedWaitKind::GetMessage {
            hwnd,
            min_msg,
            max_msg,
        } => kernel
            .gwe
            .has_message_filtered(blocked.thread_id, hwnd, min_msg, max_msg),
        crate::ce::scheduler::SchedulerBlockedWaitKind::MsgWait {
            wake_mask,
            input_available,
        } => {
            if input_available {
                kernel.gwe.has_queue_input(blocked.thread_id, wake_mask)
            } else {
                kernel.gwe.has_new_queue_input(blocked.thread_id, wake_mask)
            }
        }
    }
}

#[cfg(feature = "unicorn")]
fn consume_blocked_msg_wait_input(blocked: &BlockedWaitThread, kernel: &mut CeKernel) {
    let BlockedWaitKind::MsgWait {
        wake_mask,
        input_available,
    } = blocked.kind
    else {
        return;
    };
    if !input_available {
        kernel
            .gwe
            .clear_new_queue_input(blocked.thread_id, wake_mask);
    }
}

#[cfg(feature = "unicorn")]
fn scheduler_blocked_wait_is_ready(
    blocked: &crate::ce::scheduler::SchedulerBlockedWait,
    kernel: &CeKernel,
) -> bool {
    scheduler_blocked_wait_has_ready_handle(blocked, kernel)
        || scheduler_blocked_msg_wait_has_input(blocked, kernel)
        || match blocked.kind {
            crate::ce::scheduler::SchedulerBlockedWaitKind::SerialRead { handle } => {
                kernel.serial_read_ready(handle)
            }
            crate::ce::scheduler::SchedulerBlockedWaitKind::SendMessage { send_id } => {
                kernel.sent_message_result_ready(send_id)
            }
            _ => false,
        }
}

#[cfg(all(test, feature = "unicorn"))]
fn select_ready_blocked_wait_index(
    blocked_waits: &[BlockedWaitThread],
    active_thread_id: u32,
    now_ms: u32,
    mut has_ready_handle: impl FnMut(&BlockedWaitThread) -> bool,
    mut thread_priority: impl FnMut(u32) -> i32,
) -> Option<usize> {
    blocked_waits
        .iter()
        .enumerate()
        .filter(|(_, blocked)| {
            blocked.thread_id != active_thread_id
                && (has_ready_handle(blocked) || blocked_wait_timed_out(blocked, now_ms))
        })
        .min_by_key(|(index, blocked)| (thread_priority(blocked.thread_id), *index))
        .map(|(index, _)| index)
}

#[cfg(all(test, feature = "unicorn"))]
mod wait_scheduler_tests {
    use super::{
        BlockedWaitKind, BlockedWaitThread, blocked_msg_wait_has_input, blocked_wait_timed_out,
        select_ready_blocked_wait_index,
    };
    use crate::{ce::gwe::QS_POSTMESSAGE, config::RuntimeConfig};

    fn blocked_wait(start: u32, timeout: u32) -> BlockedWaitThread {
        BlockedWaitThread {
            wait_id: 1,
            thread_id: 1,
            thread_handle: 0x100,
            wait_handles: vec![0x104],
            kind: BlockedWaitKind::Kernel,
            wait_started_ms: start,
            timeout_ms: timeout,
            regs: [0; 32],
            return_pc: 0,
        }
    }

    fn blocked_wait_for_thread(thread_id: u32, wait_handle: u32) -> BlockedWaitThread {
        BlockedWaitThread {
            wait_id: u64::from(thread_id),
            thread_id,
            thread_handle: 0x100 + thread_id,
            wait_handles: vec![wait_handle],
            kind: BlockedWaitKind::Kernel,
            wait_started_ms: 0,
            timeout_ms: crate::ce::timer::INFINITE,
            regs: [0; 32],
            return_pc: 0,
        }
    }

    #[test]
    fn blocked_wait_timeout_uses_wrapping_tick_elapsed_time() {
        let wait = blocked_wait(u32::MAX - 5, 10);
        assert!(!blocked_wait_timed_out(&wait, 3));
        assert!(blocked_wait_timed_out(&wait, 4));
    }

    #[test]
    fn blocked_wait_timeout_respects_infinite_timeout() {
        let wait = blocked_wait(10, crate::ce::timer::INFINITE);
        assert!(!blocked_wait_timed_out(&wait, 10_000));
    }

    #[test]
    fn ready_blocked_wait_selection_uses_ce_lower_numeric_priority_first() {
        let waits = vec![
            blocked_wait_for_thread(2, 0x200),
            blocked_wait_for_thread(3, 0x204),
            blocked_wait_for_thread(4, 0x208),
        ];
        let selected = select_ready_blocked_wait_index(
            &waits,
            1,
            0,
            |_| true,
            |thread_id| match thread_id {
                2 => 150,
                3 => 10,
                4 => 80,
                _ => 0,
            },
        );
        assert_eq!(selected, Some(1));
    }

    #[test]
    fn ready_blocked_wait_selection_is_fifo_within_same_priority() {
        let waits = vec![
            blocked_wait_for_thread(2, 0x200),
            blocked_wait_for_thread(3, 0x204),
        ];
        let selected = select_ready_blocked_wait_index(&waits, 1, 0, |_| true, |_| 20);
        assert_eq!(selected, Some(0));
    }

    #[test]
    fn ready_blocked_wait_selection_skips_currently_active_thread() {
        let waits = vec![
            blocked_wait_for_thread(2, 0x200),
            blocked_wait_for_thread(3, 0x204),
        ];
        let selected = select_ready_blocked_wait_index(&waits, 2, 0, |_| true, |_| 20);
        assert_eq!(selected, Some(1));
    }

    #[test]
    fn ready_blocked_wait_selection_checks_all_multiple_wait_handles() {
        let mut multi = blocked_wait_for_thread(2, 0x200);
        multi.wait_handles.push(0x204);
        let waits = vec![multi];
        let selected = select_ready_blocked_wait_index(
            &waits,
            1,
            0,
            |blocked| blocked.wait_handles.iter().any(|handle| *handle == 0x204),
            |_| 20,
        );
        assert_eq!(selected, Some(0));
    }

    #[test]
    fn blocked_msg_wait_wakes_on_queue_input() {
        let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
        let mut kernel = crate::ce::kernel::CeKernel::boot(config);
        let thread_id = 9;
        let mut blocked = blocked_wait_for_thread(thread_id, 0x200);
        blocked.kind = BlockedWaitKind::MsgWait {
            wake_mask: QS_POSTMESSAGE,
            input_available: false,
        };

        assert!(!blocked_msg_wait_has_input(&blocked, &kernel));
        kernel
            .gwe
            .post_message(thread_id, crate::ce::gwe::Message::new(0, 0x400, 0, 0, 10));
        assert!(blocked_msg_wait_has_input(&blocked, &kernel));
    }

    #[test]
    fn user_kdata_page_exposes_current_thread_and_process_ids() {
        let page = super::user_kdata_page();
        let thread_offset = super::user_kdata_handle_address(super::SYS_HANDLE_CURRENT_THREAD)
            .saturating_sub(super::USER_KDATA_PAGE_BASE) as usize;
        let process_offset = super::user_kdata_handle_address(super::SYS_HANDLE_CURRENT_PROCESS)
            .saturating_sub(super::USER_KDATA_PAGE_BASE) as usize;

        assert_eq!(
            u32::from_le_bytes(page[thread_offset..thread_offset + 4].try_into().unwrap()),
            1
        );
        assert_eq!(
            u32::from_le_bytes(page[process_offset..process_offset + 4].try_into().unwrap()),
            1
        );
    }
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
        let _ = update_user_kdata_current_ids(
            uc,
            callout.creator_thread_id,
            kernel.current_process_id(),
        );
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
    activate_suspended_thread(uc, kernel, current_thread_id, running_thread, &suspended);
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
    let _ = update_user_kdata_current_ids(uc, worker_thread_id, kernel.current_process_id());
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
    self_suspended_threads: &std::rc::Rc<
        std::cell::RefCell<std::collections::BTreeMap<u32, SuspendedGuestThread>>,
    >,
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
    if result == 1
        && let Some(resumed) = self_suspended_threads.borrow_mut().remove(&thread_handle)
    {
        let mut creator = SuspendedGuestThread {
            thread_id: creator_thread_id,
            thread_handle: running_thread
                .borrow()
                .and_then(|(id, handle)| (id == creator_thread_id).then_some(handle)),
            regs: capture_mips_gprs(uc),
            pc: read_mips_reg(uc, RegisterMIPS::RA),
        };
        creator.regs[2] = result;
        *suspended_thread.borrow_mut() = Some(creator);
        activate_suspended_thread(uc, kernel, current_thread_id, running_thread, &resumed);
        return true;
    }
    let Some((worker_thread_id, start_address, parameter)) =
        kernel.guest_thread_start(thread_handle)
    else {
        return false;
    };

    let mut creator = SuspendedGuestThread {
        thread_id: creator_thread_id,
        thread_handle: None,
        regs: capture_mips_gprs(uc),
        pc: read_mips_reg(uc, RegisterMIPS::RA),
    };
    creator.regs[2] = result;
    *suspended_thread.borrow_mut() = Some(creator);
    *current_thread_id.borrow_mut() = worker_thread_id;
    let _ = update_user_kdata_current_ids(uc, worker_thread_id, kernel.current_process_id());
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
    let now_ms = kernel.timers.tick_count();
    let Some(ready_wait_id) =
        kernel.select_ready_blocked_waiter(active_thread_id, now_ms, |blocked, kernel| {
            scheduler_blocked_wait_is_ready(blocked, kernel)
        })
    else {
        return false;
    };
    if ready_wait_id != blocked.wait_id {
        return false;
    }
    let Some(message) = kernel.gwe.get_message_filtered(
        blocked.thread_id,
        blocked.hwnd,
        blocked.min_msg,
        blocked.max_msg,
    ) else {
        return false;
    };
    let _ = kernel.remove_blocked_waiter(blocked.wait_id);
    if write_unicorn_message(uc, blocked.msg_ptr, &message).is_err() {
        let _ = uc.emu_stop();
        return true;
    }
    kernel.record_resumed_msg_wait_input();

    let mut current = SuspendedGuestThread {
        thread_id: active_thread_id,
        thread_handle: running_thread
            .borrow()
            .and_then(|(id, handle)| (id == active_thread_id).then_some(handle)),
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
    let _ = update_user_kdata_current_ids(uc, blocked.thread_id, kernel.current_process_id());
    *running_thread.borrow_mut() = Some((blocked.thread_id, blocked.thread_handle));
    *blocked_thread.borrow_mut() = None;
    true
}

#[cfg(feature = "unicorn")]
fn activate_suspended_thread<D>(
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    kernel: &CeKernel,
    current_thread_id: &std::rc::Rc<std::cell::RefCell<u32>>,
    running_thread: &std::rc::Rc<std::cell::RefCell<Option<(u32, u32)>>>,
    suspended: &SuspendedGuestThread,
) {
    use unicorn_engine::RegisterMIPS;

    *current_thread_id.borrow_mut() = suspended.thread_id;
    let _ = update_user_kdata_current_ids(uc, suspended.thread_id, kernel.current_process_id());
    *running_thread.borrow_mut() = suspended
        .thread_handle
        .map(|handle| (suspended.thread_id, handle));
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
fn record_wndproc_call_trace(
    last_wndproc_call_traces: &std::rc::Rc<std::cell::RefCell<Vec<UnicornWndProcCallTrace>>>,
    trace: UnicornWndProcCallTrace,
) {
    let mut traces = last_wndproc_call_traces.borrow_mut();
    if traces.len() == UNICORN_WNDPROC_TRACE_LIMIT {
        traces.remove(0);
    }
    traces.push(trace);
}

#[cfg(all(feature = "unicorn", feature = "trace"))]
fn record_mfc_dispatch_trace<D>(
    traces: &std::rc::Rc<std::cell::RefCell<Vec<UnicornMfcDispatchTrace>>>,
    uc: &unicorn_engine::Unicorn<'_, D>,
    pc: u32,
) {
    use unicorn_engine::RegisterMIPS;

    let Some(label) = mfc_dispatch_probe_label(pc) else {
        return;
    };
    let sp = read_mips_reg(uc, RegisterMIPS::SP);
    let s6 = read_mips_reg(uc, RegisterMIPS::S6);
    let s7 = read_mips_reg(uc, RegisterMIPS::S7);
    let fp = read_mips_reg(uc, RegisterMIPS::FP);
    let sp10 = sp
        .checked_add(0x10)
        .and_then(|addr| read_unicorn_u32(uc, addr));
    let this_ptr = match label {
        "route_entry" => Some(read_mips_reg(uc, RegisterMIPS::A0)),
        "vtable_call" => Some(s7),
        "message_map_result" => Some(s7),
        _ => None,
    };
    let vtable_slot_98 = this_ptr
        .and_then(|ptr| read_unicorn_u32(uc, ptr))
        .and_then(|vtable| read_unicorn_u32(uc, vtable.wrapping_add(0x98)));
    let trace = UnicornMfcDispatchTrace {
        pc,
        label,
        ra: read_mips_reg(uc, RegisterMIPS::RA),
        sp,
        v0: read_mips_reg(uc, RegisterMIPS::V0),
        a0: read_mips_reg(uc, RegisterMIPS::A0),
        a1: read_mips_reg(uc, RegisterMIPS::A1),
        a2: read_mips_reg(uc, RegisterMIPS::A2),
        a3: read_mips_reg(uc, RegisterMIPS::A3),
        s5: read_mips_reg(uc, RegisterMIPS::S5),
        s6,
        s7,
        fp,
        sp10,
        this_ptr,
        hwnd: match label {
            "afx_wndproc" | "dispatch_entry" | "lookup_entry" => {
                Some(read_mips_reg(uc, RegisterMIPS::A0))
            }
            "lookup_return" => Some(read_mips_reg(uc, RegisterMIPS::A1)),
            "route_entry" | "message_map_result" | "vtable_call" => {
                Some(read_mips_reg(uc, RegisterMIPS::A1))
            }
            _ => None,
        },
        msg: match label {
            "afx_wndproc" | "dispatch_entry" => Some(read_mips_reg(uc, RegisterMIPS::A1)),
            "route_entry" | "message_map_result" | "vtable_call" => {
                Some(read_mips_reg(uc, RegisterMIPS::A2))
            }
            _ => None,
        },
        wparam: match label {
            "afx_wndproc" => Some(read_mips_reg(uc, RegisterMIPS::A2)),
            "dispatch_entry" | "route_entry" | "message_map_result" | "vtable_call" => {
                Some(read_mips_reg(uc, RegisterMIPS::A3))
            }
            _ => None,
        },
        lparam: match label {
            "afx_wndproc" => Some(read_mips_reg(uc, RegisterMIPS::A3)),
            "route_entry" => sp10,
            "message_map_result" | "vtable_call" => sp10,
            _ => None,
        },
        vtable_slot_98,
    };
    let mut traces = traces.borrow_mut();
    if traces.len() == UNICORN_MFC_DISPATCH_TRACE_LIMIT {
        traces.remove(0);
    }
    traces.push(trace);
}

#[cfg(all(feature = "unicorn", feature = "trace"))]
fn mfc_dispatch_probe_label(pc: u32) -> Option<&'static str> {
    match pc {
        0x6004_eba8 => Some("afx_wndproc"),
        0x6002_5204 => Some("dispatch_entry"),
        0x6002_50e8 => Some("lookup_entry"),
        0x6002_5114 => Some("lookup_return"),
        0x6002_4d94 => Some("route_entry"),
        0x6002_4e3c => Some("message_map_result"),
        0x6002_4ea0 => Some("vtable_call"),
        _ => None,
    }
}

#[cfg(all(feature = "unicorn", feature = "trace"))]
fn record_inavi_display_trace<D>(
    traces: &std::rc::Rc<std::cell::RefCell<Vec<UnicornInaviDisplayTrace>>>,
    uc: &unicorn_engine::Unicorn<'_, D>,
    pc: u32,
) {
    use unicorn_engine::RegisterMIPS;

    let Some(label) = inavi_display_probe_label(pc) else {
        return;
    };
    let sp = read_mips_reg(uc, RegisterMIPS::SP);
    let a0 = read_mips_reg(uc, RegisterMIPS::A0);
    let s7 = read_mips_reg(uc, RegisterMIPS::S7);
    let fp = read_mips_reg(uc, RegisterMIPS::FP);
    let this_ptr = match label {
        "child_wndproc_entry" | "renderer_init_entry" | "renderer_destroy_entry" => Some(a0),
        "child_forward_controller" | "child_default_no_controller" | "child_return" => Some(s7),
        "child_controller_result"
        | "child_controller_unhandled"
        | "child_classifier_fallback"
        | "child_fallback_class" => Some(s7),
        "renderer_init_after_base"
        | "renderer_init_store"
        | "renderer_init_success"
        | "renderer_clear" => Some(fp),
        _ => None,
    };
    let sp10 = sp
        .checked_add(0x10)
        .and_then(|addr| read_unicorn_u32(uc, addr));
    let trace = UnicornInaviDisplayTrace {
        pc,
        label,
        ra: read_mips_reg(uc, RegisterMIPS::RA),
        sp,
        v0: read_mips_reg(uc, RegisterMIPS::V0),
        a0,
        a1: read_mips_reg(uc, RegisterMIPS::A1),
        a2: read_mips_reg(uc, RegisterMIPS::A2),
        a3: read_mips_reg(uc, RegisterMIPS::A3),
        this_ptr,
        hwnd: this_ptr
            .and_then(|ptr| ptr.checked_add(0x20))
            .and_then(|addr| read_unicorn_u32(uc, addr)),
        msg: match label {
            "child_wndproc_entry" | "renderer_init_entry" | "renderer_destroy_entry" => {
                Some(read_mips_reg(uc, RegisterMIPS::A1))
            }
            "child_forward_controller" | "child_default_no_controller" | "child_return" => Some(fp),
            "child_controller_result"
            | "child_controller_unhandled"
            | "child_classifier_fallback"
            | "child_fallback_class" => Some(fp),
            _ => None,
        },
        wparam: match label {
            "child_wndproc_entry" | "renderer_init_entry" | "renderer_destroy_entry" => {
                Some(read_mips_reg(uc, RegisterMIPS::A2))
            }
            "child_forward_controller" | "child_default_no_controller" | "child_return" => {
                Some(read_mips_reg(uc, RegisterMIPS::S5))
            }
            "child_controller_result"
            | "child_controller_unhandled"
            | "child_classifier_fallback"
            | "child_fallback_class" => Some(read_mips_reg(uc, RegisterMIPS::S5)),
            _ => None,
        },
        lparam: match label {
            "child_wndproc_entry" | "renderer_init_entry" | "renderer_destroy_entry" => {
                Some(read_mips_reg(uc, RegisterMIPS::A3))
            }
            "child_forward_controller" | "child_default_no_controller" | "child_return" => {
                Some(read_mips_reg(uc, RegisterMIPS::S6))
            }
            "child_controller_result"
            | "child_controller_unhandled"
            | "child_classifier_fallback"
            | "child_fallback_class" => Some(read_mips_reg(uc, RegisterMIPS::S6)),
            "renderer_init_store" => sp10,
            _ => None,
        },
        field_20: this_ptr
            .and_then(|ptr| ptr.checked_add(0x20))
            .and_then(|addr| read_unicorn_u32(uc, addr)),
        field_44: this_ptr
            .and_then(|ptr| ptr.checked_add(0x44))
            .and_then(|addr| read_unicorn_u32(uc, addr)),
        field_e8: this_ptr
            .and_then(|ptr| ptr.checked_add(0xe8))
            .and_then(|addr| read_unicorn_u32(uc, addr)),
    };
    let mut traces = traces.borrow_mut();
    if traces.len() == UNICORN_INAVI_DISPLAY_TRACE_LIMIT {
        traces.remove(0);
    }
    traces.push(trace);
}

#[cfg(all(feature = "unicorn", feature = "trace"))]
fn inavi_display_probe_label(pc: u32) -> Option<&'static str> {
    match pc {
        0x0001_2860 => Some("child_wndproc_entry"),
        0x0001_2890 => Some("child_forward_controller"),
        0x0001_28a4 => Some("child_controller_result"),
        0x0001_28c8 => Some("child_controller_unhandled"),
        0x0001_28d0 => Some("child_classifier_fallback"),
        0x0001_28e0 => Some("child_fallback_class"),
        0x0001_28f4 => Some("child_default_no_controller"),
        0x0001_2908 => Some("child_return"),
        0x0001_2924 => Some("renderer_init_entry"),
        0x0001_2940 => Some("renderer_init_after_base"),
        0x0001_2998 => Some("renderer_init_store"),
        0x0001_29b8 => Some("renderer_init_success"),
        0x0001_2a04 => Some("renderer_destroy_entry"),
        0x0001_2a38 => Some("renderer_clear"),
        _ => None,
    }
}

#[cfg(feature = "unicorn")]
fn repair_inavi_aux_touch_alias<D>(uc: &mut unicorn_engine::Unicorn<'_, D>, pc: u32) {
    use unicorn_engine::RegisterMIPS;

    if pc != 0x0006_39ec {
        return;
    }
    let aux_base = read_mips_reg(uc, RegisterMIPS::A2);
    let Some(alias_addr) = aux_base.checked_add(0x10f0) else {
        return;
    };
    if read_unicorn_u32(uc, alias_addr).unwrap_or(0) != 0 {
        return;
    }
    let Some(inline_addr) = aux_base.checked_add(0x10f8) else {
        return;
    };
    if read_unicorn_u32(uc, inline_addr).unwrap_or(0) == 0 {
        return;
    }
    let _ = uc.mem_write(u64::from(alias_addr), &inline_addr.to_le_bytes());
}

#[cfg(feature = "unicorn")]
fn maybe_post_late_inavi_init_dialog<D>(
    kernel: &mut CeKernel,
    uc: &unicorn_engine::Unicorn<'_, D>,
    pc: u32,
    posted: &std::cell::Cell<bool>,
) {
    use unicorn_engine::RegisterMIPS;

    if posted.get() || pc != 0x0001_29b8 {
        return;
    }
    let this_ptr = read_mips_reg(uc, RegisterMIPS::FP);
    let Some(hwnd) = this_ptr
        .checked_add(0x20)
        .and_then(|addr| read_unicorn_u32(uc, addr))
        .filter(|hwnd| *hwnd != 0)
    else {
        return;
    };
    let time_ms = kernel.timers.tick_count();
    let message = crate::ce::gwe::Message::new(hwnd, WM_INITDIALOG, 0, 0, time_ms);
    if kernel.gwe.post_message_for_window(hwnd, message) {
        posted.set(true);
    }
}

#[cfg(all(feature = "unicorn", feature = "trace"))]
fn record_inavi_controller_trace<D>(
    traces: &std::rc::Rc<std::cell::RefCell<Vec<UnicornInaviControllerTrace>>>,
    milestones: &std::rc::Rc<std::cell::RefCell<Vec<UnicornInaviControllerTrace>>>,
    uc: &unicorn_engine::Unicorn<'_, D>,
    pc: u32,
) {
    use unicorn_engine::RegisterMIPS;

    let Some(label) = inavi_controller_probe_label(pc) else {
        return;
    };
    let sp = read_mips_reg(uc, RegisterMIPS::SP);
    let v0 = read_mips_reg(uc, RegisterMIPS::V0);
    let v1 = read_mips_reg(uc, RegisterMIPS::V1);
    let a0 = read_mips_reg(uc, RegisterMIPS::A0);
    let a1 = read_mips_reg(uc, RegisterMIPS::A1);
    let a2 = read_mips_reg(uc, RegisterMIPS::A2);
    let a3 = read_mips_reg(uc, RegisterMIPS::A3);
    let s0 = read_mips_reg(uc, RegisterMIPS::S0);
    let s2 = read_mips_reg(uc, RegisterMIPS::S2);
    let s1 = read_mips_reg(uc, RegisterMIPS::S1);
    let s3 = read_mips_reg(uc, RegisterMIPS::S3);
    let s4 = read_mips_reg(uc, RegisterMIPS::S4);
    let s5 = read_mips_reg(uc, RegisterMIPS::S5);
    let s6 = read_mips_reg(uc, RegisterMIPS::S6);
    let s7 = read_mips_reg(uc, RegisterMIPS::S7);
    let fp = read_mips_reg(uc, RegisterMIPS::FP);
    let t0 = read_mips_reg(uc, RegisterMIPS::T0);
    let t8 = read_mips_reg(uc, RegisterMIPS::T8);
    let t4 = read_mips_reg(uc, RegisterMIPS::T4);
    let t6 = read_mips_reg(uc, RegisterMIPS::T6);
    let sp10 = sp
        .checked_add(0x10)
        .and_then(|addr| read_unicorn_u32(uc, addr));
    let sp48 = sp
        .checked_add(0x48)
        .and_then(|addr| read_unicorn_u32(uc, addr));
    let controller = if label.starts_with("resource_ready_") {
        if label == "resource_ready_entry" {
            Some(a0)
        } else {
            Some(fp)
        }
    } else {
        match label {
            "router_entry" | "classifier_call" => Some(a0),
            _ => Some(s6),
        }
    };
    let selected_obj = match label {
        "selected_object" | "vtable_load" | "vtable_call" | "vtable_return" => Some(a0),
        "wm_size_render_vtable" | "wm_size_render_call" => Some(a0),
        _ => None,
    };
    let selected_vtable = match label {
        "vtable_call" => Some(t6),
        "wm_size_render_call" => Some(read_mips_reg(uc, RegisterMIPS::T7)),
        _ => selected_obj.and_then(|obj| read_unicorn_u32(uc, obj)),
    };
    let selected_target = match label {
        "vtable_call" => Some(t4),
        "wm_size_render_vtable" => {
            selected_vtable.and_then(|vtable| read_unicorn_u32(uc, vtable.wrapping_add(0xf0)))
        }
        "wm_size_render_call" => Some(read_mips_reg(uc, RegisterMIPS::T6)),
        _ => selected_vtable.and_then(|vtable| read_unicorn_u32(uc, vtable)),
    };
    let paint_this = match label {
        "paint_entry" => Some(a0),
        "render_lifecycle_entry" => Some(a0),
        "render_lifecycle_after_base"
        | "render_lifecycle_before_full_resize"
        | "render_lifecycle_full_resize_call" => Some(fp),
        "paint_after_begin"
        | "paint_gate_check"
        | "paint_render_obj_check"
        | "paint_render_vtable"
        | "paint_render_call"
        | "paint_end" => Some(fp),
        _ => None,
    };
    let paint_base = paint_this
        .and_then(|this_ptr| this_ptr.checked_add(0x8))
        .and_then(|addr| read_unicorn_u32(uc, addr));
    let paint_gate = paint_base
        .and_then(|base| base.checked_add(0x3_931c))
        .and_then(|addr| read_unicorn_u32(uc, addr));
    let paint_render_obj = paint_base
        .and_then(|base| base.checked_add(0x10ec))
        .and_then(|addr| read_unicorn_u32(uc, addr));
    let paint_render_target = paint_render_obj
        .and_then(|obj| read_unicorn_u32(uc, obj))
        .and_then(|vtable| read_unicorn_u32(uc, vtable.wrapping_add(0x150)));
    let render_this = match label {
        "render_entry"
        | "render_resize_entry"
        | "render_size_entry"
        | "render_full_resize_obj"
        | "render_full_resize_call"
        | "wm_size_render_vtable"
        | "wm_size_render_call" => Some(a0),
        "render_surface_gate"
        | "render_enabled_gate"
        | "render_loop_call"
        | "render_after_loop"
        | "render_flush_call"
        | "render_return"
        | "render_ctor_surface_zero"
        | "render_ctor_enabled_zero"
        | "render_surface_clear"
        | "render_dim_width_check"
        | "render_dim_height_check"
        | "render_surface_create_call"
        | "render_surface_store"
        | "render_surface_after_store" => Some(fp),
        _ => None,
    };
    let render_surface = render_this
        .and_then(|this_ptr| this_ptr.checked_add(0xb8))
        .and_then(|addr| read_unicorn_u32(uc, addr));
    let render_enabled = render_this
        .and_then(|this_ptr| this_ptr.checked_add(0x8d68))
        .and_then(|addr| read_unicorn_u8(uc, addr))
        .map(u32::from);
    let render_vtable = render_this.and_then(|this_ptr| read_unicorn_u32(uc, this_ptr));
    let render_size_target =
        render_vtable.and_then(|vtable| read_unicorn_u32(uc, vtable.wrapping_add(0xf0)));
    let render_resize_target =
        render_vtable.and_then(|vtable| read_unicorn_u32(uc, vtable.wrapping_add(0xf4)));
    let render_flush_obj = render_this
        .and_then(|this_ptr| this_ptr.checked_add(0x91a8))
        .and_then(|addr| read_unicorn_u32(uc, addr));
    let render_flush_target = render_flush_obj
        .and_then(|obj| read_unicorn_u32(uc, obj))
        .and_then(|vtable| read_unicorn_u32(uc, vtable.wrapping_add(0x1c)));
    let render_poll_result =
        (label == "render_after_loop").then_some(read_mips_reg(uc, RegisterMIPS::V0));
    let render_dim_ptr = match label {
        "render_dim_width_check" | "render_dim_height_check" => {
            Some(read_mips_reg(uc, RegisterMIPS::V0))
        }
        "render_size_return" => Some(read_mips_reg(uc, RegisterMIPS::A0)),
        _ => None,
    };
    let render_dim_offset = if label == "render_size_return" { 8 } else { 0 };
    let render_dim_w = render_dim_ptr
        .and_then(|ptr| ptr.checked_add(render_dim_offset))
        .and_then(|addr| read_unicorn_u32(uc, addr))
        .or_else(|| {
            (label == "render_dim_width_check").then_some(read_mips_reg(uc, RegisterMIPS::T7))
        })
        .or_else(|| {
            (label == "wm_size_render_call").then_some(read_mips_reg(uc, RegisterMIPS::A1))
        });
    let render_dim_h = render_dim_ptr
        .and_then(|ptr| ptr.checked_add(render_dim_offset.wrapping_add(4)))
        .and_then(|addr| read_unicorn_u32(uc, addr))
        .or_else(|| {
            (label == "render_dim_height_check").then_some(read_mips_reg(uc, RegisterMIPS::T6))
        })
        .or_else(|| {
            (label == "wm_size_render_call").then_some(read_mips_reg(uc, RegisterMIPS::A2))
        });
    let aux_base = match label {
        "aux_update_loaded_base"
        | "aux_update_state_gate"
        | "aux_update_compare0"
        | "aux_update_compare1"
        | "aux_update_after_ctor"
        | "aux_update_skip_ctor" => {
            Some(read_mips_reg(uc, RegisterMIPS::S7)).filter(|value| *value != 0)
        }
        "aux_ctor_dispatch" | "aux_ctor_entry" | "aux_lazy_init_entry" => Some(a0),
        "aux_ctor_setup"
        | "aux_ctor_store_ptr"
        | "aux_ctor_store_vtable"
        | "aux_lazy_init_done" => Some(fp),
        "aux_mouse_slot_load" | "aux_mouse_slot_deref" => {
            Some(read_mips_reg(uc, RegisterMIPS::A2)).filter(|value| *value != 0)
        }
        _ => None,
    };
    let aux_slot_10f0 = aux_base.and_then(|base| base.checked_add(0x10f0));
    let aux_slot_10ec_value = aux_base
        .and_then(|base| base.checked_add(0x10ec))
        .and_then(|addr| read_unicorn_u32(uc, addr));
    let aux_slot_10f0_vtable = aux_slot_10f0.and_then(|addr| read_unicorn_u32(uc, addr));
    let aux_inline_10f8 = aux_base.and_then(|base| base.checked_add(0x10f8));
    let aux_inline_10f8_vtable = aux_inline_10f8.and_then(|addr| read_unicorn_u32(uc, addr));
    let aux_link_ee4 = aux_base
        .and_then(|base| base.checked_add(0xee4))
        .and_then(|addr| read_unicorn_u32(uc, addr));
    let aux_init_flag_edc = aux_base
        .and_then(|base| base.checked_add(0xedc))
        .and_then(|addr| read_unicorn_u32(uc, addr));
    let aux_vtable_source = aux_base
        .and_then(|base| base.checked_add(0x194))
        .and_then(|addr| read_unicorn_u32(uc, addr));
    let aux_vtable_source_value = aux_vtable_source
        .and_then(|source| source.checked_add(4))
        .and_then(|addr| read_unicorn_u32(uc, addr));
    let aux_store_addr = match label {
        "aux_ctor_store_ptr" => Some(read_mips_reg(uc, RegisterMIPS::V0)),
        "aux_ctor_store_vtable" => Some(read_mips_reg(uc, RegisterMIPS::T6).wrapping_add(8)),
        _ => None,
    };
    let aux_store_value = match label {
        "aux_ctor_store_ptr" => Some(read_mips_reg(uc, RegisterMIPS::T6)),
        "aux_ctor_store_vtable" => Some(read_mips_reg(uc, RegisterMIPS::T7)),
        _ => None,
    };
    let query_thunk_slot = match label {
        "app_query_thunk_entry" | "app_query_thunk_target" => read_unicorn_u32(uc, 0x0052_5068),
        _ => None,
    };
    let query_thunk_target = match label {
        "app_query_thunk_entry" => query_thunk_slot,
        "app_query_thunk_target" => Some(t0),
        _ => None,
    };
    let resource_text = match label {
        "resource_module_after_getmodule" => sp
            .checked_add(0x18)
            .and_then(|addr| read_unicorn_wide_z(uc, addr, 260)),
        "resource_module_string_init_return"
        | "resource_module_string_assign_return"
        | "resource_module_findslash_return" => sp
            .checked_add(0x10)
            .and_then(|addr| read_unicorn_u32(uc, addr))
            .and_then(|ptr| read_unicorn_wide_z(uc, ptr, 260)),
        "resource_module_slice_return" => {
            read_unicorn_u32(uc, v0).and_then(|ptr| read_unicorn_wide_z(uc, ptr, 260))
        }
        "resource_module_format_call" => read_unicorn_wide_z(uc, fp, 260),
        "resource_module_format_return" | "resource_module_success" => {
            read_unicorn_wide_z(uc, fp, 260)
        }
        "resource_596b4_helper_return" => sp
            .checked_add(0x18)
            .and_then(|addr| read_unicorn_wide_z(uc, addr, 260)),
        "resource_596b4_after_set" => read_unicorn_wide_z(uc, 0x0079_c490, 260),
        "resource_59718_base_return" | "resource_59718_source_ready" => {
            read_unicorn_wide_z(uc, v0, 260)
        }
        "resource_59718_format_call" => read_unicorn_wide_z(uc, a2, 260),
        "resource_59718_path_ready"
        | "resource_59718_lookup_call"
        | "resource_59718_lookup_return" => sp
            .checked_add(0x10)
            .and_then(|addr| read_unicorn_wide_z(uc, addr, 260)),
        "resource_state_source_return"
        | "resource_state_populate_return"
        | "resource_state_set_call" => sp
            .checked_add(0x1c)
            .and_then(|addr| read_unicorn_wide_z(uc, addr, 260)),
        "resource_state_set_entry" => read_unicorn_wide_z(uc, a3, 260),
        "resource_lookup_entry" => {
            read_unicorn_wide_z(uc, read_mips_reg(uc, RegisterMIPS::A1), 260)
        }
        "resource_lookup_after_source"
        | "resource_lookup_after_check"
        | "resource_lookup_success"
        | "resource_lookup_fail" => read_unicorn_wide_z(uc, s7, 260),
        "resource_table_open_entry" => read_unicorn_wide_z(uc, a2, 260),
        "render_context_sky_lookup_call"
        | "render_context_sky_lookup_return"
        | "render_context_sky_fail"
        | "render_context_sky_success"
        | "render_context_sky_return" => sp
            .checked_add(0x10)
            .and_then(|addr| read_unicorn_wide_z(uc, addr, 260)),
        "render_context_db_lookup_entry" => read_unicorn_wide_z(uc, a1, 260),
        "render_context_db_lookup_state"
        | "render_context_db_lookup_alloc"
        | "render_context_db_lookup_verify"
        | "render_context_db_lookup_success" => read_unicorn_wide_z(uc, s5, 260),
        _ => None,
    };
    let resource_format_text = match label {
        "resource_module_format_call" | "resource_module_format_return" => {
            read_unicorn_wide_z(uc, 0x005b_b0d4, 260)
        }
        "resource_59718_format_call" => read_unicorn_wide_z(uc, a1, 260),
        _ => None,
    };
    let resource_aux_text = match label {
        "resource_596b4_after_set" => sp
            .checked_add(0x18)
            .and_then(|addr| read_unicorn_wide_z(uc, addr, 260)),
        "resource_module_format_return" | "resource_module_success" => sp
            .checked_add(0x18)
            .and_then(|addr| read_unicorn_wide_z(uc, addr, 260)),
        _ => None,
    };
    let resource_arg_text = match label {
        "resource_module_string_init_return"
        | "resource_module_string_assign_return"
        | "resource_module_findslash_return" => sp
            .checked_add(0x10)
            .map(|addr| resource_pointer_preview(uc, addr, 260)),
        "resource_module_slice_setup" => Some(format!(
            "index={}/source={}",
            v0,
            sp.checked_add(0x10)
                .map(|addr| resource_pointer_preview(uc, addr, 260))
                .unwrap_or_else(|| "source=<overflow>".to_owned())
        )),
        "resource_module_slice_return" => Some(format!(
            "ret={}/slice={}",
            resource_pointer_preview(uc, v0, 260),
            sp.checked_add(0x14)
                .map(|addr| resource_pointer_preview(uc, addr, 260))
                .unwrap_or_else(|| "slice=<overflow>".to_owned())
        )),
        "resource_module_format_call" => Some(resource_pointer_preview(uc, a2, 260)),
        "resource_module_format_return" | "resource_module_success" => {
            Some(resource_pointer_preview(uc, a2, 260))
        }
        "resource_mode_search_entry" => Some(format!(
            "list=0x{a0:08x}/count={}/out=0x{a1:08x}",
            read_unicorn_u32(uc, a0.wrapping_add(4)).unwrap_or_default()
        )),
        "resource_mode_search_count" => Some(format!("count={v0}/out=0x{s6:08x}")),
        "resource_mode_record_read" => Some(format!(
            "index={}/count={}/kind={}/raw0=0x{:08x}/raw1=0x{:08x}",
            s7 as i16,
            read_unicorn_u32(uc, fp.wrapping_add(4)).unwrap_or_default(),
            read_unicorn_i16(uc, sp.wrapping_add(0x18)).unwrap_or_default(),
            read_unicorn_u32(uc, sp.wrapping_add(0x18)).unwrap_or_default(),
            read_unicorn_u32(uc, sp.wrapping_add(0x1c)).unwrap_or_default()
        )),
        "resource_mode_search_return" => Some(format!(
            "result=0x{v0:08x}/mode={}/submode={}/path={}",
            read_unicorn_i16(uc, s6).unwrap_or_default(),
            read_unicorn_i16(uc, s6.wrapping_add(2)).unwrap_or_default(),
            resource_pointer_preview(uc, s6.wrapping_add(4), 260)
        )),
        "resource_state_source_return" => Some(format!(
            "mode={}/submode={}/path={}",
            read_unicorn_i16(uc, sp.wrapping_add(0x18)).unwrap_or_default(),
            read_unicorn_i16(uc, sp.wrapping_add(0x1a)).unwrap_or_default(),
            sp.checked_add(0x1c)
                .map(|addr| resource_pointer_preview(uc, addr, 260))
                .unwrap_or_else(|| "path=<overflow>".to_owned())
        )),
        "resource_state_populate_return" => Some(format!(
            "result=0x{v0:08x}/mode={}/submode={}/path={}",
            read_unicorn_i16(uc, sp.wrapping_add(0x18)).unwrap_or_default(),
            read_unicorn_i16(uc, sp.wrapping_add(0x1a)).unwrap_or_default(),
            sp.checked_add(0x1c)
                .map(|addr| resource_pointer_preview(uc, addr, 260))
                .unwrap_or_else(|| "path=<overflow>".to_owned())
        )),
        "resource_state_acquire_return" => Some(format!(
            "state=0x{v0:08x}/mode={}/submode={}/ready={}",
            read_unicorn_i16(uc, v0).unwrap_or_default(),
            read_unicorn_i16(uc, v0.wrapping_add(2)).unwrap_or_default(),
            read_unicorn_u8(uc, v0.wrapping_add(0x20c)).unwrap_or_default()
        )),
        "resource_state_set_call" => Some(format!(
            "state=0x{a0:08x}/mode={}/submode={}/path={}",
            a1 as i16,
            a2 as i16,
            sp.checked_add(0x1c)
                .map(|addr| resource_pointer_preview(uc, addr, 260))
                .unwrap_or_else(|| "path=<overflow>".to_owned())
        )),
        "resource_state_set_entry" => Some(format!(
            "state=0x{a0:08x}/mode={}/submode={}/path={}",
            a1 as i16,
            a2 as i16,
            resource_pointer_preview(uc, a3, 260)
        )),
        "resource_state_after_mode" => Some(format!(
            "state=0x{a0:08x}/mode={}/submode={}",
            read_unicorn_i16(uc, a0).unwrap_or_default(),
            read_unicorn_i16(uc, a0.wrapping_add(2)).unwrap_or_default()
        )),
        "resource_state_after_pair" | "resource_state_set_return" => Some(format!(
            "state=0x{a0:08x}/mode={}/submode={}/ready={}",
            read_unicorn_i16(uc, a0).unwrap_or_default(),
            read_unicorn_i16(uc, a0.wrapping_add(2)).unwrap_or_default(),
            read_unicorn_u8(uc, a0.wrapping_add(0x20c)).unwrap_or_default()
        )),
        "resource_state_set_done" => Some(format!(
            "state=0x{a0:08x}/mode={}/submode={}/ready={}",
            read_unicorn_i16(uc, a0).unwrap_or_default(),
            read_unicorn_i16(uc, a0.wrapping_add(2)).unwrap_or_default(),
            read_unicorn_u8(uc, a0.wrapping_add(0x20c)).unwrap_or_default()
        )),
        "resource_lookup_entry" => Some(format!(
            "state=0x{a0:08x}/path={}",
            resource_pointer_preview(uc, a1, 260)
        )),
        "resource_lookup_after_source" => Some(format!(
            "table=0x{v0:08x}/mode={}/path={}",
            read_unicorn_i16(uc, fp).unwrap_or_default(),
            resource_pointer_preview(uc, s7, 260)
        )),
        "resource_lookup_after_check" => Some(format!(
            "check_result=0x{v0:08x}/ready={}/path={}",
            read_unicorn_u8(uc, fp.wrapping_add(0x20c)).unwrap_or_default(),
            resource_pointer_preview(uc, s7, 260)
        )),
        "resource_lookup_success" | "resource_lookup_fail" => Some(format!(
            "ready={}/path={}",
            read_unicorn_u8(uc, fp.wrapping_add(0x20c)).unwrap_or_default(),
            resource_pointer_preview(uc, s7, 260)
        )),
        "resource_table_open_entry" => Some(format!(
            "table=0x{a0:08x}/mode={}/path={}",
            a1 as i16,
            resource_pointer_preview(uc, a2, 260)
        )),
        "render_context_sky_entry" => Some(format!("global=0x{a0:08x}/index={a1}")),
        "render_context_sky_lookup_call" => Some(format!(
            "manager=0x{a0:08x}/key={}/out=0x{fp:08x}",
            sp.checked_add(0x10)
                .map(|addr| resource_pointer_preview(uc, addr, 260))
                .unwrap_or_else(|| "key=<overflow>".to_owned())
        )),
        "render_context_sky_lookup_return"
        | "render_context_sky_fail"
        | "render_context_sky_success"
        | "render_context_sky_return" => Some(format!(
            "status=0x{v0:08x}/key={}/out=0x{fp:08x}/out_value=0x{:08x}",
            sp.checked_add(0x10)
                .map(|addr| resource_pointer_preview(uc, addr, 260))
                .unwrap_or_else(|| "key=<overflow>".to_owned()),
            read_unicorn_u32(uc, fp).unwrap_or_default()
        )),
        "render_context_db_lookup_entry" => Some(format!(
            "manager=0x{a0:08x}/key={}/out=0x{a2:08x}",
            resource_pointer_preview(uc, a1, 260)
        )),
        "render_context_db_lookup_state"
        | "render_context_db_lookup_alloc"
        | "render_context_db_lookup_verify"
        | "render_context_db_lookup_success" => Some(format!(
            "manager=0x{s7:08x}/key={}/out=0x{s1:08x}/out_value=0x{:08x}",
            resource_pointer_preview(uc, s5, 260),
            read_unicorn_u32(uc, s1).unwrap_or_default()
        )),
        "resource_table_after_create" => Some(format!(
            "handle=0x{fp:08x}/valid={}",
            (fp != u32::MAX) as u8
        )),
        "resource_table_header_count" => Some(format!(
            "count={}/bytes={}",
            read_unicorn_i16(uc, sp.wrapping_add(0x20)).unwrap_or_default(),
            read_unicorn_u32(uc, sp.wrapping_add(0x24)).unwrap_or_default()
        )),
        "resource_table_record_read" => Some(format!(
            "index={}/want_mode={}/record_mode={}/bytes={}",
            s5 as i16,
            s3 as i16,
            read_unicorn_i16(uc, sp.wrapping_add(0x38)).unwrap_or_default(),
            read_unicorn_u32(uc, sp.wrapping_add(0x24)).unwrap_or_default()
        )),
        "resource_table_record_match" => Some(format!(
            "index={}/offset={}/size={}/bytes={}",
            s5 as i16,
            read_unicorn_u32(uc, sp.wrapping_add(0x3c)).unwrap_or_default(),
            read_unicorn_u32(uc, sp.wrapping_add(0x40)).unwrap_or_default(),
            read_unicorn_u32(uc, sp.wrapping_add(0x24)).unwrap_or_default()
        )),
        "resource_table_data_seek" => Some(format!("seek_result=0x{v0:08x}/expected=0x{a1:08x}")),
        "resource_table_data_read" => Some(format!(
            "buffer=0x{:08x}/size={}/bytes={}",
            read_unicorn_u32(uc, s6).unwrap_or_default(),
            read_unicorn_u32(uc, sp.wrapping_add(0x40)).unwrap_or_default(),
            read_unicorn_u32(uc, sp.wrapping_add(0x24)).unwrap_or_default()
        )),
        "resource_table_field_count" => {
            let buffer = read_unicorn_u32(uc, s6).unwrap_or_default();
            Some(format!(
                "buffer=0x{buffer:08x}/field_count={}/raw0=0x{:08x}",
                read_unicorn_i16(uc, buffer.wrapping_add(2)).unwrap_or_default(),
                read_unicorn_u32(uc, buffer).unwrap_or_default()
            ))
        }
        "resource_table_field_key" => {
            let buffer = v0;
            let offset = v1;
            Some(format!(
                "buffer=0x{buffer:08x}/offset={offset}/key={}/next=0x{:08x}",
                read_mips_reg(uc, RegisterMIPS::T4) as i16,
                buffer.wrapping_add(offset.wrapping_add(2))
            ))
        }
        "resource_table_field_insert_return" => Some(format!(
            "entry_key={}/entry_ptr=0x{:08x}/found=0x{:08x}/created={}",
            read_unicorn_i16(uc, sp.wrapping_add(0x28)).unwrap_or_default(),
            read_unicorn_u32(uc, sp.wrapping_add(0x2c)).unwrap_or_default(),
            read_unicorn_u32(uc, sp.wrapping_add(0x30)).unwrap_or_default(),
            read_unicorn_u8(uc, sp.wrapping_add(0x34)).unwrap_or_default()
        )),
        "resource_table_field_type" => Some(format!(
            "offset={}/type={}/field_index={}/field_count={}",
            v1,
            read_mips_reg(uc, RegisterMIPS::A3) as i16,
            s7 as i16,
            read_unicorn_i16(uc, sp.wrapping_add(0x20)).unwrap_or_default()
        )),
        "resource_table_field_advance" => Some(format!(
            "next_offset={}/field_index={}/field_count={}",
            v1,
            s7 as i16,
            read_unicorn_i16(uc, sp.wrapping_add(0x20)).unwrap_or_default()
        )),
        "resource_table_field_bad_type" => Some(format!(
            "offset={}/bad_type={}/field_index={}/field_count={}",
            v1,
            read_mips_reg(uc, RegisterMIPS::A3) as i16,
            s7 as i16,
            read_unicorn_i16(uc, sp.wrapping_add(0x20)).unwrap_or_default()
        )),
        "resource_tree_insert_entry" => Some(format!(
            "tree=0x{a0:08x}/out=0x{a1:08x}/key_ptr=0x{a2:08x}/key={}/value_ptr=0x{:08x}",
            read_unicorn_i16(uc, a2).unwrap_or_default(),
            read_unicorn_u32(uc, sp.wrapping_add(0x50)).unwrap_or_default()
        )),
        "resource_tree_insert_link" => Some(format!(
            "tree=0x{s7:08x}/parent=0x{s6:08x}/node=0x{fp:08x}/out=0x{s5:08x}"
        )),
        "resource_tree_insert_return" => Some(format!(
            "out=0x{s5:08x}/node=0x{:08x}/tree_count={}",
            read_unicorn_u32(uc, s5).unwrap_or_default(),
            read_unicorn_u32(uc, s7.wrapping_add(0x10)).unwrap_or_default()
        )),
        "resource_table_success" | "resource_table_fail" => Some(format!(
            "table_buffer=0x{:08x}/handle=0x{fp:08x}",
            read_unicorn_u32(uc, s6).unwrap_or_default()
        )),
        "map_asset_ready_entry" => Some(format!(
            "controller=0x{a0:08x}/state=0x{:08x}",
            read_unicorn_u32(uc, a0.wrapping_add(8)).unwrap_or_default()
        )),
        "map_asset_probe_entry" => Some(format!(
            "state=0x{a0:08x}/name={}",
            resource_pointer_preview(uc, a1, 260)
        )),
        "map_asset_probe_open_return" => Some(format!(
            "open_result=0x{v0:08x}/name={}",
            resource_pointer_preview(uc, fp, 260)
        )),
        "map_asset_probe_return" => Some(format!(
            "result=0x{v0:08x}/query={}/state={}",
            sp.checked_add(0x10)
                .map(|addr| resource_pointer_preview(uc, addr, 260))
                .unwrap_or_else(|| "query=<overflow>".to_owned()),
            sp.checked_add(0x18)
                .map(|addr| resource_pointer_preview(uc, addr, 260))
                .unwrap_or_else(|| "state=<overflow>".to_owned())
        )),
        "map_asset_probe_fail" => Some(format!(
            "query={}/state={}",
            sp.checked_add(0x10)
                .map(|addr| resource_pointer_preview(uc, addr, 260))
                .unwrap_or_else(|| "query=<overflow>".to_owned()),
            sp.checked_add(0x18)
                .map(|addr| resource_pointer_preview(uc, addr, 260))
                .unwrap_or_else(|| "state=<overflow>".to_owned())
        )),
        "map_asset_metric_call" => Some(format!(
            "state={}",
            sp.checked_add(0x18)
                .map(|addr| resource_pointer_preview(uc, addr, 260))
                .unwrap_or_else(|| "state=<overflow>".to_owned())
        )),
        "map_asset_metric_entry" => {
            Some(format!("state={}", resource_pointer_preview(uc, a0, 260)))
        }
        "map_asset_metric_value" => Some(format!(
            "metric=0x{fp:08x}/state={}",
            resource_pointer_preview(uc, sp.wrapping_add(0x10), 260)
        )),
        "map_asset_probe_exit" => Some(format!("result=0x{v0:08x}")),
        "map_asset_ready_success" => Some(format!(
            "stored_metric=0x{:08x}/state=0x{:08x}",
            read_unicorn_u32(
                uc,
                read_unicorn_u32(uc, fp.wrapping_add(8))
                    .unwrap_or_default()
                    .wrapping_add(0x462a0)
            )
            .unwrap_or_default(),
            read_unicorn_u32(uc, fp.wrapping_add(8)).unwrap_or_default()
        )),
        "map_prefix_static_init_entry" => Some(format!(
            "flag=0x{:02x}/prefix={}",
            read_unicorn_u8(uc, 0x0079_c464).unwrap_or_default(),
            resource_pointer_preview(uc, 0x0079_c468u32.wrapping_add(0x230), 260)
        )),
        "map_prefix_static_init_ready" => Some(format!(
            "flag=0x{:02x}/prefix={}",
            read_unicorn_u8(uc, 0x0079_c464).unwrap_or_default(),
            resource_pointer_preview(uc, 0x0079_c468u32.wrapping_add(0x230), 260)
        )),
        "map_prefix_object_getter" => Some(format!(
            "object=0x0079c468/prefix={}",
            resource_pointer_preview(uc, 0x0079_c468u32.wrapping_add(0x230), 260)
        )),
        "map_prefix_ctor_entry" => Some(format!(
            "object=0x{a0:08x}/prefix={}",
            resource_pointer_preview(uc, a0.wrapping_add(0x230), 260)
        )),
        "map_prefix_setter_entry" => Some(format!(
            "object=0x{a0:08x}/source={}/old_prefix={}",
            resource_pointer_preview(uc, a1, 260),
            resource_pointer_preview(uc, a0.wrapping_add(0x230), 260)
        )),
        "map_prefix_setter_return" => Some(format!(
            "object=0x{:08x}/prefix={}",
            read_mips_reg(uc, RegisterMIPS::A0).wrapping_sub(0x230),
            resource_pointer_preview(uc, read_mips_reg(uc, RegisterMIPS::A0), 260)
        )),
        "map_prefix_getter_entry" => Some(format!(
            "object=0x{a0:08x}/prefix={}",
            resource_pointer_preview(uc, a0.wrapping_add(0x230), 260)
        )),
        "map_prefix_builder_entry" => Some(format!(
            "object=0x{a0:08x}/source={}/slot438={}",
            resource_pointer_preview(uc, a1, 260),
            resource_pointer_preview(uc, a0.wrapping_add(0x438), 260)
        )),
        "map_prefix_source_module_path" => Some(format!(
            "buffer={}/fp={}/v23={}",
            resource_pointer_preview(uc, sp.wrapping_add(0x258), 260),
            resource_pointer_preview(uc, fp, 260),
            resource_pointer_preview(uc, s7, 260)
        )),
        "map_prefix_set_from_module" | "map_prefix_set_from_fallback" => Some(format!(
            "source={}/prefix={}",
            resource_pointer_preview(uc, fp, 260),
            resource_pointer_preview(uc, 0x0079_c468u32.wrapping_add(0x230), 260)
        )),
        "map_prefix_set_from_stack" | "map_prefix_set_from_resolved" => Some(format!(
            "source={}/prefix={}",
            resource_pointer_preview(uc, sp.wrapping_add(0x460), 260),
            resource_pointer_preview(uc, 0x0079_c468u32.wrapping_add(0x230), 260)
        )),
        "map_prefix_after_set" => Some(format!(
            "prefix={}",
            resource_pointer_preview(uc, 0x0079_c468u32.wrapping_add(0x230), 260)
        )),
        "rsimage_stream_read_entry" => Some(format!(
            "stream=0x{a0:08x}/buffer=0x{a1:08x}/requested={a2}"
        )),
        "rsimage_stream_read_before_callback" => Some(format!(
            "stream=0x{fp:08x}/callback_obj=0x{:08x}/callback=0x{:08x}/requested={}",
            read_unicorn_u32(uc, fp.wrapping_add(0x94)).unwrap_or_default(),
            read_unicorn_u32(uc, 0x0052_5360).unwrap_or_default(),
            read_mips_reg(uc, RegisterMIPS::S7)
        )),
        "rsimage_stream_read_after_callback" => Some(format!(
            "result=0x{v0:08x}/actual={}/requested={}",
            read_unicorn_u32(uc, sp.wrapping_add(0x18)).unwrap_or_default(),
            read_mips_reg(uc, RegisterMIPS::S7)
        )),
        "rsimage_stream_read_short" => Some(format!(
            "stream=0x{fp:08x}/actual={}/requested={}",
            read_unicorn_u32(uc, sp.wrapping_add(0x18)).unwrap_or_default(),
            read_mips_reg(uc, RegisterMIPS::S7)
        )),
        "rsimage_stream_read_return" => Some(format!(
            "stream=0x{fp:08x}/actual={}/requested={}",
            read_unicorn_u32(uc, sp.wrapping_add(0x18)).unwrap_or_default(),
            read_mips_reg(uc, RegisterMIPS::S7)
        )),
        "map_sort_partition_entry" => Some(format!(
            "base=0x{a0:08x}/end=0x{a1:08x}/pivot_lo=0x{a2:08x}/pivot_hi=0x{a3:08x}/callback=0x{:08x}/span_bytes={}",
            read_unicorn_u32(uc, sp.wrapping_add(0x38)).unwrap_or_default(),
            a1.wrapping_sub(a0)
        )),
        "map_sort_forward_probe" => Some(format!(
            "cursor=0x{fp:08x}/end=0x{s7:08x}/pivot_lo=0x{s5:08x}/pivot_hi=0x{s6:08x}/callback=0x{:08x}/result=0x{v0:08x}/left=0x{:08x}{:08x}",
            read_unicorn_u32(uc, sp.wrapping_add(0x38)).unwrap_or_default(),
            read_unicorn_u32(uc, fp).unwrap_or_default(),
            read_unicorn_u32(uc, fp.wrapping_add(4)).unwrap_or_default()
        )),
        "map_sort_backward_probe" => Some(format!(
            "cursor=0x{s7:08x}/begin=0x{fp:08x}/pivot_lo=0x{s5:08x}/pivot_hi=0x{s6:08x}/callback=0x{:08x}/result=0x{v0:08x}/right=0x{:08x}{:08x}",
            read_unicorn_u32(uc, sp.wrapping_add(0x38)).unwrap_or_default(),
            read_unicorn_u32(uc, s7).unwrap_or_default(),
            read_unicorn_u32(uc, s7.wrapping_add(4)).unwrap_or_default()
        )),
        "map_sort_partition_return" => Some(format!(
            "cursor=0x{v0:08x}/begin=0x{fp:08x}/end=0x{s7:08x}/callback=0x{:08x}",
            read_unicorn_u32(uc, sp.wrapping_add(0x38)).unwrap_or_default()
        )),
        "null_object_wrapper_a_entry"
        | "null_object_wrapper_a_call"
        | "null_object_wrapper_b_entry"
        | "null_object_wrapper_b_call"
        | "null_object_target_entry"
        | "null_object_target_before_call"
        | "null_object_target_fault_store" => Some(format!(
            "a0=0x{a0:08x}/a1=0x{a1:08x}/a2=0x{a2:08x}/a3=0x{a3:08x}/fp=0x{fp:08x}/s6=0x{s6:08x}/s7=0x{s7:08x}/ra=0x{:08x}",
            read_mips_reg(uc, RegisterMIPS::RA)
        )),
        "render_slot_helper_call"
        | "render_slot_after_helper"
        | "render_slot_query_return"
        | "render_slot_aux_create_call"
        | "render_slot_aux_store"
        | "render_slot_check"
        | "render_slot_alloc_call"
        | "render_slot_ctor_call"
        | "render_slot_store" => Some(format!(
            "base=0x{s7:08x}/slot=0x{s6:08x}/slot_value=0x{s5:08x}/v0=0x{v0:08x}/a0=0x{a0:08x}/a1=0x{a1:08x}/a2=0x{a2:08x}/a3=0x{a3:08x}/ra=0x{:08x}",
            read_mips_reg(uc, RegisterMIPS::RA)
        )),
        "render_use_slot_entry"
        | "render_use_slot_make_text"
        | "render_use_slot_load"
        | "render_use_slot_call"
        | "render_use_slot_after_call"
        | "render_slot_guard_entry"
        | "render_slot_guard_load"
        | "render_slot_guard_call" => Some(format!(
            "base=0x{s6:08x}/text=0x{fp:08x}/slot=0x{a2:08x}/slot_value=0x{a0:08x}/v0=0x{v0:08x}/a1=0x{a1:08x}/ra=0x{:08x}",
            read_mips_reg(uc, RegisterMIPS::RA)
        )),
        "render_resize_preflight_call"
        | "render_resize_preflight_result"
        | "render_resize_context_source_call"
        | "render_resize_context_query_call"
        | "render_resize_context_query_result"
        | "render_resize_context_null_gate"
        | "render_resize_context_validate_call"
        | "render_resize_context_validate_result"
        | "render_resize_abort" => Some(format!(
            "this=0x{s7:08x}/ctx=0x{s5:08x}/v0=0x{v0:08x}/a0=0x{a0:08x}/a1=0x{a1:08x}/a2=0x{a2:08x}/a3=0x{a3:08x}/ra=0x{:08x}",
            read_mips_reg(uc, RegisterMIPS::RA)
        )),
        "render_map_entry"
        | "render_map_offset_load"
        | "render_map_arg_load"
        | "render_map_pointer_compute"
        | "render_map_pointer_deref" => {
            let offset_addr = s4.wrapping_add(0x555c);
            let offset = read_unicorn_u32(uc, offset_addr).unwrap_or_default();
            let arg_addr = fp.wrapping_add(s5);
            let arg = read_unicorn_u32(uc, arg_addr).unwrap_or_default();
            let computed = s0.wrapping_add(offset);
            let computed_value = read_unicorn_u32(uc, computed).unwrap_or_default();
            Some(format!(
                "s0=0x{s0:08x}/s4=0x{s4:08x}/offset_addr=0x{offset_addr:08x}/offset=0x{offset:08x}/computed=0x{computed:08x}/computed_value=0x{computed_value:08x}/s5=0x{s5:08x}/arg_addr=0x{arg_addr:08x}/arg=0x{arg:08x}/fp=0x{fp:08x}/s7=0x{s7:08x}/ra=0x{:08x}",
                read_mips_reg(uc, RegisterMIPS::RA)
            ))
        }
        "render_map_static_init_entry"
        | "render_map_vector_reset_entry"
        | "render_map_lazy_entry"
        | "render_map_lazy_flag_store"
        | "render_map_lazy_init_call"
        | "render_map_lazy_return"
        | "render_map_vector_ctor_entry"
        | "render_map_vector_ctor_after_insert"
        | "render_map_vector_ctor_return"
        | "render_map_vector_destroy_entry"
        | "render_map_vector_destroy_return"
        | "render_map_vector_insert_entry"
        | "render_map_vector_insert_return"
        | "render_map_vector_alloc_call"
        | "render_map_vector_after_alloc"
        | "render_map_vector_store_begin"
        | "render_map_vector_store_return" => {
            let flag = read_unicorn_u8(uc, 0x0081_5550).unwrap_or_default();
            let vector = read_unicorn_u32(uc, 0x0081_555c).unwrap_or_default();
            let vector_end = read_unicorn_u32(uc, 0x0081_5560).unwrap_or_default();
            let vector_cap = read_unicorn_u32(uc, 0x0081_5564).unwrap_or_default();
            let vector_len = vector_end.wrapping_sub(vector) / 8;
            let vector_capacity = vector_cap.wrapping_sub(vector) / 8;
            Some(format!(
                "flag=0x{flag:02x}/vec=0x{vector:08x}/end=0x{vector_end:08x}/cap=0x{vector_cap:08x}/len={vector_len}/capacity={vector_capacity}/a0=0x{a0:08x}/a1=0x{a1:08x}/a2=0x{a2:08x}/a3=0x{a3:08x}/v0=0x{v0:08x}/fp=0x{fp:08x}/ra=0x{:08x}",
                read_mips_reg(uc, RegisterMIPS::RA)
            ))
        }
        "main_init_entry" => Some(format!(
            "app=0x{a0:08x}/caller_ra=0x{:08x}",
            read_mips_reg(uc, RegisterMIPS::RA)
        )),
        "main_init_singleton_call" => Some(format!(
            "app=0x{:08x}/mutex_handle=0x{:08x}",
            read_mips_reg(uc, RegisterMIPS::S6),
            read_unicorn_u32(uc, read_mips_reg(uc, RegisterMIPS::S6).wrapping_add(0xb4))
                .unwrap_or_default()
        )),
        "main_init_singleton_return" => Some(format!(
            "result=0x{v0:08x}/app=0x{:08x}/mutex_handle=0x{:08x}",
            read_mips_reg(uc, RegisterMIPS::S6),
            read_unicorn_u32(uc, read_mips_reg(uc, RegisterMIPS::S6).wrapping_add(0xb4))
                .unwrap_or_default()
        )),
        "main_init_abort_after_singleton" => Some(format!(
            "result=0x{v0:08x}/app=0x{:08x}",
            read_mips_reg(uc, RegisterMIPS::S6)
        )),
        "main_init_continue_after_singleton" => Some(format!(
            "result=0x{v0:08x}/app=0x{:08x}",
            read_mips_reg(uc, RegisterMIPS::S6)
        )),
        "main_init_return" => Some(format!(
            "result=0x{v0:08x}/app=0x{:08x}/child=0x{:08x}",
            read_mips_reg(uc, RegisterMIPS::S6),
            read_unicorn_u32(uc, read_mips_reg(uc, RegisterMIPS::S6).wrapping_add(0x20))
                .unwrap_or_default()
        )),
        "singleton_entry" => Some(format!(
            "app=0x{a0:08x}/stored_mutex=0x{:08x}",
            read_unicorn_u32(uc, a0.wrapping_add(0xb4)).unwrap_or_default()
        )),
        "singleton_after_create_mutex" => Some(format!(
            "handle=0x{v0:08x}/app=0x{fp:08x}/stored_mutex=0x{:08x}",
            read_unicorn_u32(uc, fp.wrapping_add(0xb4)).unwrap_or_default()
        )),
        "singleton_after_get_last_error" => Some(format!(
            "last_error=0x{v0:08x}/app=0x{fp:08x}/stored_mutex=0x{:08x}",
            read_unicorn_u32(uc, fp.wrapping_add(0xb4)).unwrap_or_default()
        )),
        "singleton_existing_window_branch" => Some(format!(
            "last_error=0x{v0:08x}/app=0x{fp:08x}/stored_mutex=0x{:08x}",
            read_unicorn_u32(uc, fp.wrapping_add(0xb4)).unwrap_or_default()
        )),
        "singleton_existing_return" => Some(format!(
            "result=1/app=0x{fp:08x}/stored_mutex=0x{:08x}",
            read_unicorn_u32(uc, fp.wrapping_add(0xb4)).unwrap_or_default()
        )),
        "singleton_new_return" => Some(format!(
            "result=0/app=0x{fp:08x}/stored_mutex=0x{:08x}",
            read_unicorn_u32(uc, fp.wrapping_add(0xb4)).unwrap_or_default()
        )),
        "auth_proc_call" | "auth_proc_loop_call" | "auth_proc_loop_return" => Some(format!(
            "a0=0x{a0:08x}/a0_text={}/a1=0x{a1:08x}/a1_text={}/static_615874={}/static_615878={}/v0=0x{v0:08x}",
            resource_pointer_preview(uc, a0, 128),
            resource_pointer_preview(uc, a1, 128),
            resource_pointer_preview(uc, 0x0061_5874, 128),
            resource_pointer_preview(uc, 0x0061_5878, 128)
        )),
        "dynamic_loader_thunk_entry" | "dynamic_loader_thunk_jump" => Some(format!(
            "slot=0x{:08x}/t0=0x{t0:08x}/t8=0x{t8:08x}/a0=0x{a0:08x}/a0_text={}/a1=0x{a1:08x}/a1_text={}",
            read_unicorn_u32(uc, 0x0052_538c).unwrap_or_default(),
            resource_pointer_preview(uc, a0, 128),
            resource_pointer_preview(uc, a1, 128)
        )),
        _ => None,
    };
    let trace = UnicornInaviControllerTrace {
        pc,
        label,
        instruction: read_unicorn_u32(uc, pc),
        ra: read_mips_reg(uc, RegisterMIPS::RA),
        sp,
        v0: read_mips_reg(uc, RegisterMIPS::V0),
        a0,
        a1: read_mips_reg(uc, RegisterMIPS::A1),
        a2: read_mips_reg(uc, RegisterMIPS::A2),
        a3: read_mips_reg(uc, RegisterMIPS::A3),
        s0,
        s2,
        s3,
        s4,
        s5,
        s6,
        s7,
        fp,
        sp10,
        sp48,
        controller,
        hwnd: match label {
            "router_entry" => Some(read_mips_reg(uc, RegisterMIPS::A1)),
            _ if label.starts_with("resource_ready_") => None,
            _ => Some(s2),
        },
        msg: match label {
            "router_entry" => Some(read_mips_reg(uc, RegisterMIPS::A2)),
            "classifier_entry" => Some(read_mips_reg(uc, RegisterMIPS::A1)),
            _ if label.starts_with("resource_ready_") => None,
            _ => Some(fp),
        },
        wparam: match label {
            "router_entry" => Some(read_mips_reg(uc, RegisterMIPS::A3)),
            "classifier_entry" => Some(read_mips_reg(uc, RegisterMIPS::A2)),
            _ if label.starts_with("resource_ready_") => None,
            _ => Some(s4),
        },
        lparam: match label {
            "router_entry" => sp10,
            _ if label.starts_with("resource_ready_") => None,
            _ => sp48,
        },
        classifier: match label {
            "router_entry" | "classifier_call" => None,
            "classifier_entry" => None,
            "classifier_return" => Some(read_mips_reg(uc, RegisterMIPS::V0)),
            _ if label.starts_with("resource_ready_") => None,
            _ => Some(s7),
        },
        selected_obj,
        selected_vtable,
        selected_target,
        paint_base,
        paint_gate,
        paint_render_obj,
        paint_render_target,
        render_surface,
        render_enabled,
        render_size_target,
        render_resize_target,
        render_flush_obj,
        render_flush_target,
        render_poll_result,
        render_dim_ptr,
        render_dim_w,
        render_dim_h,
        aux_base,
        aux_slot_10ec_value,
        aux_slot_10f0,
        aux_slot_10f0_vtable,
        aux_inline_10f8,
        aux_inline_10f8_vtable,
        aux_link_ee4,
        aux_init_flag_edc,
        aux_vtable_source,
        aux_vtable_source_value,
        aux_store_addr,
        aux_store_value,
        #[cfg(feature = "trace")]
        query_thunk_slot,
        #[cfg(feature = "trace")]
        query_thunk_target,
        #[cfg(feature = "trace")]
        resource_text,
        #[cfg(feature = "trace")]
        resource_format_text,
        #[cfg(feature = "trace")]
        resource_aux_text,
        #[cfg(feature = "trace")]
        resource_arg_text,
    };
    let keep_table_record_sample = if label == "resource_table_record_read" {
        let index = i32::from(s5 as i16);
        let count = read_unicorn_i16(uc, sp.wrapping_add(0x20))
            .map(i32::from)
            .unwrap_or_default();
        let record_mode = read_unicorn_i16(uc, sp.wrapping_add(0x38)).unwrap_or_default();
        (0..4).contains(&index) || record_mode == s3 as i16 || (count > 0 && index >= count - 3)
    } else {
        false
    };
    let keep_mode_record_sample = if label == "resource_mode_record_read" {
        let index = i32::from(s7 as i16);
        let count = read_unicorn_u32(uc, fp.wrapping_add(4))
            .map(|value| value as i32)
            .unwrap_or_default();
        (0..4).contains(&index) || (count > 0 && index >= count - 3)
    } else {
        false
    };
    if is_inavi_render_milestone_label(label)
        || keep_table_record_sample
        || keep_mode_record_sample
        || is_inavi_controller_focus_trace(&trace)
    {
        let mut milestones = milestones.borrow_mut();
        if milestones.len() == UNICORN_INAVI_RENDER_MILESTONE_LIMIT {
            milestones.remove(0);
        }
        milestones.push(trace.clone());
    }
    let mut traces = traces.borrow_mut();
    if traces.len() == UNICORN_INAVI_CONTROLLER_TRACE_LIMIT {
        traces.remove(0);
    }
    traces.push(trace);
}

#[cfg(all(feature = "unicorn", feature = "trace"))]
fn record_render_map_global_write<D>(
    milestones: &std::rc::Rc<std::cell::RefCell<Vec<UnicornInaviControllerTrace>>>,
    uc: &unicorn_engine::Unicorn<'_, D>,
    address: u32,
    size: usize,
    value: i64,
) {
    use unicorn_engine::RegisterMIPS;

    let pc = read_mips_reg(uc, RegisterMIPS::PC);
    let sp = read_mips_reg(uc, RegisterMIPS::SP);
    let trace = UnicornInaviControllerTrace {
        pc,
        label: "render_map_global_write",
        instruction: read_unicorn_u32(uc, pc),
        ra: read_mips_reg(uc, RegisterMIPS::RA),
        sp,
        v0: read_mips_reg(uc, RegisterMIPS::V0),
        a0: read_mips_reg(uc, RegisterMIPS::A0),
        a1: read_mips_reg(uc, RegisterMIPS::A1),
        a2: read_mips_reg(uc, RegisterMIPS::A2),
        a3: read_mips_reg(uc, RegisterMIPS::A3),
        s0: read_mips_reg(uc, RegisterMIPS::S0),
        s2: read_mips_reg(uc, RegisterMIPS::S2),
        s3: read_mips_reg(uc, RegisterMIPS::S3),
        s4: read_mips_reg(uc, RegisterMIPS::S4),
        s5: read_mips_reg(uc, RegisterMIPS::S5),
        s6: read_mips_reg(uc, RegisterMIPS::S6),
        s7: read_mips_reg(uc, RegisterMIPS::S7),
        fp: read_mips_reg(uc, RegisterMIPS::FP),
        sp10: sp
            .checked_add(0x10)
            .and_then(|addr| read_unicorn_u32(uc, addr)),
        sp48: sp
            .checked_add(0x48)
            .and_then(|addr| read_unicorn_u32(uc, addr)),
        controller: None,
        hwnd: None,
        msg: None,
        wparam: None,
        lparam: None,
        classifier: None,
        selected_obj: None,
        selected_vtable: None,
        selected_target: None,
        paint_base: None,
        paint_gate: None,
        paint_render_obj: None,
        paint_render_target: None,
        render_surface: None,
        render_enabled: None,
        render_size_target: None,
        render_resize_target: None,
        render_flush_obj: None,
        render_flush_target: None,
        render_poll_result: None,
        render_dim_ptr: None,
        render_dim_w: None,
        render_dim_h: None,
        aux_base: None,
        aux_slot_10ec_value: None,
        aux_slot_10f0: None,
        aux_slot_10f0_vtable: None,
        aux_inline_10f8: None,
        aux_inline_10f8_vtable: None,
        aux_link_ee4: None,
        aux_init_flag_edc: None,
        aux_vtable_source: None,
        aux_vtable_source_value: None,
        aux_store_addr: Some(address),
        aux_store_value: Some(value as u32),
        query_thunk_slot: None,
        query_thunk_target: None,
        resource_text: None,
        resource_format_text: None,
        resource_aux_text: None,
        resource_arg_text: Some(format!(
            "addr=0x{address:08x}/size={size}/value=0x{:08x}/flag=0x{:02x}/bytes=0x{:02x}{:02x}{:02x}{:02x}{:02x}/vec=0x{:08x}/end=0x{:08x}/cap=0x{:08x}/slots={:08x},{:08x},{:08x},{:08x},{:08x}",
            value as u32,
            read_unicorn_u8(uc, 0x0081_5550).unwrap_or_default(),
            read_unicorn_u8(uc, 0x0081_5554).unwrap_or_default(),
            read_unicorn_u8(uc, 0x0081_5555).unwrap_or_default(),
            read_unicorn_u8(uc, 0x0081_5556).unwrap_or_default(),
            read_unicorn_u8(uc, 0x0081_5557).unwrap_or_default(),
            read_unicorn_u8(uc, 0x0081_5558).unwrap_or_default(),
            read_unicorn_u32(uc, 0x0081_555c).unwrap_or_default(),
            read_unicorn_u32(uc, 0x0081_5560).unwrap_or_default(),
            read_unicorn_u32(uc, 0x0081_5564).unwrap_or_default(),
            read_unicorn_u32(uc, 0x0081_5588).unwrap_or_default(),
            read_unicorn_u32(uc, 0x0081_558c).unwrap_or_default(),
            read_unicorn_u32(uc, 0x0081_5590).unwrap_or_default(),
            read_unicorn_u32(uc, 0x0081_5594).unwrap_or_default(),
            read_unicorn_u32(uc, 0x0081_5598).unwrap_or_default(),
        )),
    };
    let mut milestones = milestones.borrow_mut();
    if milestones.len() == UNICORN_INAVI_RENDER_MILESTONE_LIMIT {
        milestones.remove(0);
    }
    milestones.push(trace);
}

#[cfg(all(feature = "unicorn", feature = "trace"))]
fn is_inavi_render_milestone_label(label: &str) -> bool {
    label.starts_with("render_")
        || label.starts_with("wm_size_")
        || label.starts_with("paint_")
        || label.starts_with("init_dialog_")
        || label.starts_with("app_query_")
        || label.starts_with("query_5237_")
        || label.starts_with("map_asset_")
        || label.starts_with("map_prefix_")
        || label.starts_with("map_sort_")
        || label.starts_with("rsimage_")
        || label.starts_with("singleton_")
        || label.starts_with("main_init_")
        || label.starts_with("null_object_")
        || label.starts_with("auth_")
        || label.starts_with("dynamic_loader_")
        || label.starts_with("resource_ready_")
        || label.starts_with("resource_module_")
        || label.starts_with("resource_596b4_")
        || label.starts_with("resource_59718_")
        || matches!(
            label,
            "resource_mode_search_entry"
                | "resource_mode_search_count"
                | "resource_mode_search_return"
        )
        || label.starts_with("resource_state_")
        || label.starts_with("resource_lookup_")
        || matches!(
            label,
            "resource_table_open_entry"
                | "resource_table_after_create"
                | "resource_table_header_count"
                | "resource_table_record_match"
                | "resource_table_data_seek"
                | "resource_table_data_read"
                | "resource_table_field_count"
                | "resource_table_field_key"
                | "resource_table_field_insert_return"
                | "resource_table_field_type"
                | "resource_table_field_advance"
                | "resource_table_field_bad_type"
                | "resource_tree_insert_entry"
                | "resource_tree_insert_link"
                | "resource_tree_insert_return"
                | "resource_table_success"
                | "resource_table_fail"
        )
}

#[cfg(all(feature = "unicorn", feature = "trace"))]
fn is_inavi_controller_focus_trace(trace: &UnicornInaviControllerTrace) -> bool {
    matches!(
        trace.label,
        "router_entry"
            | "classifier_entry"
            | "classifier_return"
            | "jump_table"
            | "select_bucket0"
            | "select_bucket1"
            | "select_bucket2"
            | "select_bucket3"
            | "select_bucket4"
            | "select_bucket5"
            | "selected_object"
            | "vtable_load"
            | "vtable_call"
            | "vtable_return"
            | "router_return"
    ) && trace.msg == Some(0x5237)
}

#[cfg(all(feature = "unicorn", feature = "trace"))]
fn inavi_controller_probe_label(pc: u32) -> Option<&'static str> {
    match pc {
        0x0001_a448 => Some("classifier_entry"),
        0x0001_a558 => Some("router_entry"),
        0x0001_a598 => Some("classifier_call"),
        0x0001_a5a0 => Some("classifier_return"),
        0x0001_a6f0 => Some("jump_table"),
        0x0001_a714 => Some("select_bucket0"),
        0x0001_a728 => Some("select_bucket1"),
        0x0001_a73c => Some("select_bucket2"),
        0x0001_a750 => Some("select_bucket3"),
        0x0001_a764 => Some("select_bucket4"),
        0x0001_a778 => Some("select_bucket5"),
        0x0001_a788 => Some("selected_object"),
        0x0001_a790 => Some("vtable_load"),
        0x0001_a7a8 => Some("vtable_call"),
        0x0001_a7b0 => Some("vtable_return"),
        0x0001_a7b8 => Some("router_return"),
        0x0001_ad3c => Some("resource_state_set_entry"),
        0x0001_ad48 => Some("resource_state_after_mode"),
        0x0001_ad50 => Some("resource_state_after_pair"),
        0x0001_ad88 => Some("resource_state_set_return"),
        0x0002_c6e8 => Some("paint_entry"),
        0x0002_c70c => Some("paint_after_begin"),
        0x0002_c720 => Some("paint_gate_check"),
        0x0002_c728 => Some("paint_render_obj_check"),
        0x0002_c734 => Some("paint_render_vtable"),
        0x0002_c73c => Some("paint_render_call"),
        0x0002_c748 => Some("paint_end"),
        0x0002_bac8 => Some("init_dialog_surface_call"),
        0x0002_bc34 => Some("init_dialog_surface_entry"),
        0x0002_bc70 => Some("init_dialog_after_text_setup"),
        0x0002_bc78 => Some("init_dialog_after_text_lookup"),
        0x0002_bc80 => Some("init_dialog_after_window_query"),
        0x0002_bc88 => Some("init_dialog_before_tick_delta"),
        0x0002_bcc0 => Some("init_dialog_before_query_5237"),
        0x0002_bcd0 => Some("init_dialog_query_5237_call"),
        0x0002_bcd8 => Some("init_dialog_resource_check"),
        0x0002_bcec => Some("init_dialog_query_56d0_call"),
        0x0002_bcfc => Some("init_dialog_after_resource"),
        0x0002_bda0 => Some("render_lifecycle_precall"),
        0x0002_bddc => Some("render_lifecycle_full_call"),
        0x0002_bde4 => Some("render_lifecycle_full_return"),
        0x0001_360c => Some("app_query_thunk_entry"),
        0x0001_3614 => Some("app_query_thunk_target"),
        0x0005_7ac8 => Some("query_5237_entry"),
        0x0005_7b08 => Some("query_5237_ready_call"),
        0x0005_7b10 => Some("query_5237_ready_return"),
        0x0005_7b18 => Some("query_5237_success"),
        0x0005_7b20 => Some("query_5237_fail"),
        0x0005_7b24 => Some("query_5237_fail_zero"),
        0x0005_9484 => Some("resource_state_source_return"),
        0x0005_9490 => Some("resource_state_populate_return"),
        0x0005_9498 => Some("resource_state_acquire_return"),
        0x0005_94a4 => Some("resource_state_set_call"),
        0x0005_94ac => Some("resource_state_set_done"),
        0x0001_ad94 => Some("resource_lookup_entry"),
        0x0001_adb0 => Some("resource_lookup_after_source"),
        0x0001_adc0 => Some("resource_lookup_after_check"),
        0x0001_adc8 => Some("resource_lookup_success"),
        0x0001_ade4 => Some("resource_lookup_fail"),
        0x0006_bd18 => Some("resource_table_open_entry"),
        0x0006_bddc => Some("resource_table_after_create"),
        0x0006_be38 => Some("resource_table_header_count"),
        0x0006_be64 => Some("resource_table_record_read"),
        0x0006_be90 => Some("resource_table_fail"),
        0x0006_beac => Some("resource_table_record_match"),
        0x0006_bee8 => Some("resource_table_data_seek"),
        0x0006_bf28 => Some("resource_table_data_read"),
        0x0006_bf3c => Some("resource_table_field_count"),
        0x0006_bf8c => Some("resource_table_field_key"),
        0x0006_bfb4 => Some("resource_table_field_insert_return"),
        0x0006_bfd8 => Some("resource_table_field_type"),
        0x0006_c05c => Some("resource_table_field_advance"),
        0x0006_c0b4 => Some("resource_table_field_bad_type"),
        0x0006_c080 => Some("resource_table_success"),
        0x0006_c1a8 => Some("resource_tree_insert_entry"),
        0x0006_c3ec => Some("resource_tree_insert_link"),
        0x0006_c40c => Some("resource_tree_insert_return"),
        0x0005_8790 => Some("resource_ready_entry"),
        0x0005_87ec => Some("resource_ready_after_589dc"),
        0x0005_87fc => Some("resource_ready_pass_589dc"),
        0x0005_8834 => Some("resource_ready_after_58fac"),
        0x0005_88a0 => Some("resource_ready_after_58b1c"),
        0x0005_88ec => Some("resource_ready_after_58c3c"),
        0x0005_8904 => Some("resource_ready_after_5b068"),
        0x0005_891c => Some("resource_ready_after_58984"),
        0x0005_892c => Some("resource_ready_return"),
        0x0005_b8cc => Some("map_asset_ready_entry"),
        0x0005_b91c => Some("map_asset_probe_return"),
        0x0005_b924 => Some("map_asset_probe_fail"),
        0x0005_b948 => Some("map_asset_metric_call"),
        0x0005_b974 => Some("map_asset_ready_success"),
        0x0005_89fc => Some("resource_ready_after_59430"),
        0x0005_8a04 => Some("resource_ready_fail_59430"),
        0x0005_8a34 => Some("resource_ready_after_594f8"),
        0x0005_8a3c => Some("resource_ready_fail_594f8"),
        0x0005_8a7c => Some("resource_ready_after_59718"),
        0x0005_8a84 => Some("resource_ready_fail_59718"),
        0x0005_8ab4 => Some("resource_ready_after_59778"),
        0x0005_8abc => Some("resource_ready_fail_59778"),
        0x0005_8ad8 => Some("resource_ready_after_595b8"),
        0x0005_8ae0 => Some("resource_ready_fail_595b8"),
        0x0005_8b08 => Some("resource_ready_pass_inner"),
        0x0005_96e8 => Some("resource_596b4_helper_return"),
        0x0005_9704 => Some("resource_596b4_after_set"),
        0x0005_9718 => Some("resource_59718_entry"),
        0x0005_973c => Some("resource_59718_base_return"),
        0x0005_9744 => Some("resource_59718_source_ready"),
        0x0005_9750 => Some("resource_59718_format_call"),
        0x0005_9758 => Some("resource_59718_path_ready"),
        0x0005_9764 => Some("resource_59718_lookup_call"),
        0x0005_976c => Some("resource_59718_lookup_return"),
        0x0029_9544 => Some("resource_mode_search_entry"),
        0x0029_9570 => Some("resource_mode_search_count"),
        0x0029_9594 => Some("resource_mode_record_read"),
        0x0029_9778 => Some("resource_mode_search_return"),
        0x0012_9564 => Some("resource_module_after_getmodule"),
        0x0012_956c => Some("resource_module_string_init_return"),
        0x0012_9580 => Some("resource_module_string_assign_return"),
        0x0012_958c => Some("resource_module_findslash_return"),
        0x0012_95b0 => Some("resource_module_slice_setup"),
        0x0012_95bc => Some("resource_module_slice_return"),
        0x0012_95c8 => Some("resource_module_format_call"),
        0x0012_95d0 => Some("resource_module_format_return"),
        0x0012_95e0 => Some("resource_module_success"),
        0x0006_c838 => Some("map_prefix_static_init_entry"),
        0x0006_c870 => Some("map_prefix_static_init_ready"),
        0x0006_c87c => Some("map_prefix_object_getter"),
        0x0006_c888 => Some("map_prefix_ctor_entry"),
        0x0006_c9d8 => Some("map_prefix_setter_entry"),
        0x0006_c9f4 => Some("map_prefix_setter_return"),
        0x0006_ca00 => Some("map_prefix_getter_entry"),
        0x0006_ca08 => Some("map_prefix_builder_entry"),
        0x0012_97c0 => Some("map_prefix_source_module_path"),
        0x0012_980c => Some("map_prefix_set_from_module"),
        0x0012_985c => Some("map_prefix_set_from_fallback"),
        0x0012_9944 => Some("map_prefix_set_from_stack"),
        0x0012_9a00 => Some("map_prefix_set_from_resolved"),
        0x0012_9814 | 0x0012_9864 | 0x0012_994c | 0x0012_9a08 => Some("map_prefix_after_set"),
        0x0013_fe30 => Some("map_asset_probe_entry"),
        0x0013_fe80 => Some("map_asset_probe_open_return"),
        0x0013_ff38 => Some("map_asset_probe_exit"),
        0x0013_ff54 => Some("map_asset_metric_entry"),
        0x0013_ffd4 => Some("map_asset_metric_value"),
        0x0030_7d18 => Some("rsimage_stream_read_entry"),
        0x0030_7d44 => Some("rsimage_stream_read_before_callback"),
        0x0030_7d58 => Some("rsimage_stream_read_after_callback"),
        0x0030_7d74 => Some("rsimage_stream_read_short"),
        0x0030_7d84 => Some("rsimage_stream_read_return"),
        0x000b_8714 => Some("map_sort_partition_entry"),
        0x000b_8764 => Some("map_sort_forward_probe"),
        0x000b_8784 => Some("map_sort_backward_probe"),
        0x000b_87f4 => Some("map_sort_partition_return"),
        0x0001_199c => Some("singleton_entry"),
        0x0001_19cc => Some("singleton_after_create_mutex"),
        0x0001_19e8 => Some("singleton_after_get_last_error"),
        0x0001_19f4 => Some("singleton_existing_window_branch"),
        0x0001_1a28 => Some("singleton_existing_return"),
        0x0001_1a40 => Some("singleton_new_return"),
        0x0001_1cc8 => Some("main_init_entry"),
        0x0001_1d20 => Some("main_init_singleton_call"),
        0x0001_1d28 => Some("main_init_singleton_return"),
        0x0001_1d30 => Some("main_init_abort_after_singleton"),
        0x0001_1d88 => Some("main_init_continue_after_singleton"),
        0x0001_1ea0 => Some("main_init_return"),
        0x0025_3a58 => Some("null_object_wrapper_a_entry"),
        0x0025_3a64 => Some("null_object_wrapper_a_call"),
        0x0025_49a0 => Some("null_object_wrapper_b_entry"),
        0x0025_49b8 => Some("null_object_wrapper_b_call"),
        0x0025_6634 => Some("null_object_target_entry"),
        0x0025_6658 => Some("null_object_target_before_call"),
        0x0025_6660 => Some("null_object_target_fault_store"),
        0x0010_4220 => Some("render_slot_helper_call"),
        0x0010_4228 => Some("render_slot_after_helper"),
        0x0010_4270 => Some("render_slot_query_return"),
        0x0010_4284 => Some("render_slot_aux_create_call"),
        0x0010_42a0 => Some("render_slot_aux_store"),
        0x0010_42a4 => Some("render_slot_check"),
        0x0010_42cc => Some("render_slot_alloc_call"),
        0x0010_42dc => Some("render_slot_ctor_call"),
        0x0010_42ec => Some("render_slot_store"),
        0x0011_efc8 => Some("render_use_slot_entry"),
        0x0011_f024 => Some("render_use_slot_make_text"),
        0x0011_f02c => Some("render_use_slot_load"),
        0x0011_f030 => Some("render_use_slot_call"),
        0x0011_f038 => Some("render_use_slot_after_call"),
        0x0011_f0b8 => Some("render_slot_guard_entry"),
        0x0011_f0d0 => Some("render_slot_guard_load"),
        0x0011_f0f8 => Some("render_slot_guard_call"),
        0x0010_3f9c => Some("render_resize_preflight_call"),
        0x0010_3fa4 => Some("render_resize_preflight_result"),
        0x0010_4010 => Some("render_resize_context_source_call"),
        0x0010_4020 => Some("render_resize_context_query_call"),
        0x0010_4028 => Some("render_resize_context_query_result"),
        0x0010_402c => Some("render_resize_context_null_gate"),
        0x0010_4034 => Some("render_resize_context_validate_call"),
        0x0010_403c => Some("render_resize_context_validate_result"),
        0x0010_4384 => Some("render_resize_abort"),
        0x0023_46a8 => Some("render_context_sky_entry"),
        0x0023_4738 => Some("render_context_sky_lookup_call"),
        0x0023_4740 => Some("render_context_sky_lookup_return"),
        0x0023_4778 => Some("render_context_sky_fail"),
        0x0023_4780 => Some("render_context_sky_success"),
        0x0023_479c => Some("render_context_sky_return"),
        0x0030_8d5c => Some("render_context_db_lookup_entry"),
        0x0030_8de0 => Some("render_context_db_lookup_state"),
        0x0030_8edc => Some("render_context_db_lookup_alloc"),
        0x0030_8f20 => Some("render_context_db_lookup_verify"),
        0x0030_8fac => Some("render_context_db_lookup_success"),
        0x0026_f664 => Some("render_map_static_init_entry"),
        0x0026_f688 => Some("render_map_vector_reset_entry"),
        0x0026_f6cc => Some("render_map_lazy_entry"),
        0x0026_f6f4 => Some("render_map_lazy_flag_store"),
        0x0026_f6fc => Some("render_map_lazy_init_call"),
        0x0026_f704 => Some("render_map_lazy_return"),
        0x0026_f7c0 => Some("render_map_entry"),
        0x0026_f7d0 => Some("render_map_offset_load"),
        0x0026_f7d8 => Some("render_map_arg_load"),
        0x0026_f7dc => Some("render_map_pointer_compute"),
        0x0026_f7e4 => Some("render_map_pointer_deref"),
        0x0026_f894 => Some("render_map_vector_ctor_entry"),
        0x0026_f8c0 => Some("render_map_vector_ctor_after_insert"),
        0x0026_f8c8 => Some("render_map_vector_ctor_return"),
        0x0026_f988 => Some("render_map_vector_destroy_entry"),
        0x0026_f9b8 => Some("render_map_vector_destroy_return"),
        0x0026_f9c8 => Some("render_map_vector_insert_entry"),
        0x0026_f9f0 => Some("render_map_vector_insert_return"),
        0x0026_fa78 => Some("render_map_vector_alloc_call"),
        0x0026_fa98 => Some("render_map_vector_after_alloc"),
        0x0026_fb70 => Some("render_map_vector_store_begin"),
        0x0026_fb7c => Some("render_map_vector_store_return"),
        0x0020_17a4 => Some("auth_proc_call"),
        0x0020_17c8 => Some("auth_proc_loop_call"),
        0x0020_17d0 => Some("auth_proc_loop_return"),
        0x0049_6a44 => Some("dynamic_loader_thunk_entry"),
        0x0049_6a4c => Some("dynamic_loader_thunk_jump"),
        0x0002_d158 => Some("wm_size_entry"),
        0x0002_d198 => Some("wm_size_render_vtable"),
        0x0002_d1a0 => Some("wm_size_render_call"),
        0x0003_0e1c => Some("render_lifecycle_entry"),
        0x0003_0fa0 => Some("render_lifecycle_after_base"),
        0x0003_1140 => Some("render_lifecycle_before_full_resize"),
        0x0003_1188 => Some("render_full_resize_load_obj"),
        0x0003_118c => Some("render_full_resize_obj"),
        0x0003_1194 => Some("render_full_resize_call"),
        0x0006_3708 => Some("aux_mouse_slot_load"),
        0x0006_370c => Some("aux_mouse_slot_deref"),
        0x0006_4b58 => Some("aux_full_resize_slot_load"),
        0x0006_4b64 => Some("aux_full_resize_slot_call"),
        0x0006_39ec => Some("aux_mouse_slot_load"),
        0x0006_39f0 => Some("aux_mouse_slot_deref"),
        0x004b_1c8c => Some("aux_update_before_call"),
        0x004b_2104 => Some("aux_update_entry"),
        0x004b_2144 => Some("aux_update_loaded_base"),
        0x004b_21dc => Some("aux_update_state_gate"),
        0x004b_21e4 => Some("aux_update_compare0"),
        0x004b_21f4 => Some("aux_update_compare1"),
        0x004b_2214 => Some("aux_update_after_ctor"),
        0x004b_23a8 => Some("aux_update_skip_ctor"),
        0x0010_23e0 => Some("render_ctor_surface_zero"),
        0x0010_28a8 => Some("render_ctor_enabled_zero"),
        0x0010_33e4 => Some("render_resize_entry"),
        0x0010_4878 => Some("render_surface_clear"),
        0x0010_4890 => Some("render_dim_width_check"),
        0x0010_48a8 => Some("render_dim_height_check"),
        0x0010_4904 => Some("render_surface_create_call"),
        0x0010_4910 => Some("render_surface_store"),
        0x0010_4954 => Some("render_surface_after_store"),
        0x0010_518c => Some("render_entry"),
        0x0010_51ac => Some("render_surface_gate"),
        0x0010_51c0 => Some("render_enabled_gate"),
        0x0010_5244 => Some("render_loop_call"),
        0x0010_524c => Some("render_after_loop"),
        0x0010_525c => Some("render_flush_call"),
        0x0010_5264 => Some("render_return"),
        0x0011_ce60 => Some("render_size_entry"),
        0x0011_ce7c => Some("render_size_return"),
        0x0014_b30c => Some("render_poll_call"),
        0x0014_b314 => Some("render_poll_return"),
        0x004b_2204 => Some("aux_ctor_dispatch"),
        0x004b_23e4 => Some("aux_ctor_entry"),
        0x004b_2410 => Some("aux_ctor_setup"),
        0x004b_24f0 => Some("aux_ctor_store_ptr"),
        0x004b_2648 => Some("aux_ctor_store_vtable"),
        0x004b_264c => Some("aux_lazy_init_entry"),
        0x004b_2724 => Some("aux_lazy_init_done"),
        _ => None,
    }
}

#[cfg(feature = "unicorn")]
fn should_trace_wndproc_message(msg: u32) -> bool {
    msg >= crate::ce::gwe::WM_USER || (cfg!(feature = "trace") && is_display_lifecycle_message(msg))
}

#[cfg(feature = "unicorn")]
fn is_display_lifecycle_message(msg: u32) -> bool {
    matches!(
        msg,
        crate::ce::gwe::WM_CREATE
            | crate::ce::gwe::WM_NCCREATE
            | crate::ce::gwe::WM_DESTROY
            | crate::ce::gwe::WM_MOVE
            | crate::ce::gwe::WM_SIZE
            | crate::ce::gwe::WM_PAINT
            | crate::ce::gwe::WM_SHOWWINDOW
            | crate::ce::gwe::WM_WINDOWPOSCHANGED
            | crate::ce::gwe::WM_NCDESTROY
            | crate::ce::gwe::WM_COMMAND
            | crate::ce::gwe::WM_TIMER
            | 0x0200
            | 0x0201
            | 0x0202
    )
}

#[cfg(feature = "unicorn")]
fn is_guest_wndproc(wndproc: u32) -> bool {
    wndproc != 0 && wndproc != crate::ce::gwe::DEFAULT_WNDPROC
}

#[cfg(feature = "unicorn")]
fn should_direct_call_send_message_wndproc(
    active_thread_id: u32,
    active_process_id: u32,
    target_thread_id: u32,
    target_process_id: u32,
    wndproc: u32,
) -> bool {
    active_thread_id == target_thread_id
        && active_process_id == target_process_id
        && is_guest_wndproc(wndproc)
}

#[cfg(feature = "unicorn")]
fn should_receiver_context_send_message_wndproc(
    active_thread_id: u32,
    active_process_id: u32,
    target_thread_id: u32,
    target_process_id: u32,
    wndproc: u32,
) -> bool {
    active_thread_id != target_thread_id
        && active_process_id == target_process_id
        && is_guest_wndproc(wndproc)
}

#[cfg(feature = "unicorn")]
fn try_enter_dispatch_message_callout<D>(
    kernel: &CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    args: &[u32],
    active_thread_id: u32,
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
        destroy_root_hwnd: None,
        remaining_destroy_callouts: Vec::new(),
        send_thread_id: kernel
            .gwe
            .in_send_message(active_thread_id)
            .then_some(active_thread_id),
        send_timeout_result_ptr: None,
        send_restore: None,
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
    active_thread_id: u32,
    current_thread_id: &std::rc::Rc<std::cell::RefCell<u32>>,
    running_thread: &std::rc::Rc<std::cell::RefCell<Option<(u32, u32)>>>,
    blocked_waits: &std::rc::Rc<std::cell::RefCell<Vec<BlockedWaitThread>>>,
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
    let target_process_id = window.process_id;
    let class_name = window.class_name.clone();
    let active_process_id = kernel.current_process_id();
    let msg = args.get(1).copied().unwrap_or(0);
    let wparam = args.get(2).copied().unwrap_or(0);
    let lparam = args.get(3).copied().unwrap_or(0);
    let result_ptr = is_send_message_timeout
        .then(|| args.get(6).copied().unwrap_or(0))
        .filter(|ptr| *ptr != 0);
    let return_pc = read_mips_reg(uc, RegisterMIPS::RA);
    let source = if is_send_message_timeout {
        "SendMessageTimeout"
    } else {
        "SendMessageW"
    };

    let should_direct = should_direct_call_send_message_wndproc(
        active_thread_id,
        active_process_id,
        target_thread_id,
        target_process_id,
        wndproc,
    );
    let should_receiver_context = should_receiver_context_send_message_wndproc(
        active_thread_id,
        active_process_id,
        target_thread_id,
        target_process_id,
        wndproc,
    );
    if !should_direct && !should_receiver_context {
        if is_guest_wndproc(wndproc) {
            tracing::debug!(
                target: "ce.gwe",
                source,
                hwnd = format_args!("0x{hwnd:08x}"),
                class = class_name.as_str(),
                wndproc = format_args!("0x{wndproc:08x}"),
                active_thread_id,
                active_process_id,
                target_thread_id,
                target_process_id,
                "SendMessage guest wndproc not entered"
            );
        }
        return false;
    }

    tracing::debug!(
        target: "ce.gwe",
        source,
        hwnd = format_args!("0x{hwnd:08x}"),
        msg = format_args!("0x{msg:08x}"),
        wparam = format_args!("0x{wparam:08x}"),
        lparam = format_args!("0x{lparam:08x}"),
        class = class_name.as_str(),
        wndproc = format_args!("0x{wndproc:08x}"),
        ra = format_args!("0x{return_pc:08x}"),
        "SendMessage guest wndproc callout"
    );

    let send_restore = if should_receiver_context {
        let timeout_ms = is_send_message_timeout.then(|| args.get(5).copied().unwrap_or(0));
        let Some(send_id) = kernel.begin_cross_thread_send_message_w(
            active_thread_id,
            hwnd,
            msg,
            wparam,
            lparam,
            timeout_ms,
        ) else {
            return false;
        };
        if !kernel
            .gwe
            .activate_sent_message_for_receiver(target_thread_id, send_id)
        {
            return false;
        }
        let sender_regs = capture_mips_gprs(uc);
        let previous_running_thread = *running_thread.borrow();
        let thread_handle = previous_running_thread
            .filter(|(thread_id, _)| *thread_id == active_thread_id)
            .map(|(_, handle)| handle)
            .unwrap_or(0);
        let wait_started_ms = kernel.timers.tick_count();
        let timeout_for_wait = timeout_ms.unwrap_or(crate::ce::timer::INFINITE);
        let kind = BlockedWaitKind::SendMessage {
            send_id,
            receiver_thread_id: target_thread_id,
            result_ptr,
            previous_running_thread,
        };
        let wait_id = kernel.register_blocked_waiter(
            active_thread_id,
            thread_handle,
            Vec::new(),
            scheduler_blocked_wait_kind(kind),
            wait_started_ms,
            timeout_for_wait,
        );
        blocked_waits.borrow_mut().push(BlockedWaitThread {
            wait_id,
            thread_id: active_thread_id,
            thread_handle,
            wait_handles: Vec::new(),
            kind,
            wait_started_ms,
            timeout_ms: timeout_for_wait,
            regs: sender_regs,
            return_pc,
        });
        *current_thread_id.borrow_mut() = target_thread_id;
        let _ = update_user_kdata_current_ids(uc, target_thread_id, kernel.current_process_id());
        *running_thread.borrow_mut() = None;
        Some(SendMessageRestoreContext {
            sender_thread_id: active_thread_id,
            receiver_thread_id: target_thread_id,
            send_id,
            wait_id,
        })
    } else {
        kernel.gwe.begin_send_message(target_thread_id);
        None
    };

    pending_returns.borrow_mut().push(PendingWndProcReturn {
        source,
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
        destroy_root_hwnd: None,
        remaining_destroy_callouts: Vec::new(),
        send_thread_id: should_direct.then_some(target_thread_id),
        send_timeout_result_ptr: result_ptr,
        send_restore,
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
        if let Some(callout) = pending_returns.borrow_mut().pop() {
            if let Some(restore) = callout.send_restore {
                let _ = kernel.complete_active_sent_message(restore.receiver_thread_id, 0);
                let _ = kernel.take_completed_send_message_result(restore.send_id);
                if let Some(index) = blocked_waits
                    .borrow()
                    .iter()
                    .position(|blocked| blocked.wait_id == restore.wait_id)
                {
                    let blocked = blocked_waits.borrow_mut().remove(index);
                    let _ = kernel.remove_blocked_waiter(blocked.wait_id);
                    let previous_running_thread = match blocked.kind {
                        BlockedWaitKind::SendMessage {
                            previous_running_thread,
                            ..
                        } => previous_running_thread,
                        _ => None,
                    };
                    restore_mips_gprs(uc, &blocked.regs);
                    *running_thread.borrow_mut() = previous_running_thread;
                }
                *current_thread_id.borrow_mut() = restore.sender_thread_id;
                let _ = update_user_kdata_current_ids(
                    uc,
                    restore.sender_thread_id,
                    kernel.current_process_id(),
                );
            } else if let Some(thread_id) = callout.send_thread_id {
                kernel.gwe.end_send_message(thread_id);
            }
        }
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

    let result = default_window_proc_result(&mut kernel.gwe, hwnd, msg, wparam, lparam);
    let _ = uc.reg_write(RegisterMIPS::V0, u64::from(result));
    true
}

#[cfg(feature = "unicorn")]
fn default_window_proc_result(
    gwe: &mut crate::ce::gwe::Gwe,
    hwnd: u32,
    msg: u32,
    wparam: u32,
    lparam: u32,
) -> u32 {
    if msg == crate::ce::gwe::WM_PAINT {
        let _ = gwe.validate_window(hwnd);
    }
    crate::ce::gwe::default_send_message_result(msg, wparam, lparam)
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
        destroy_root_hwnd: None,
        remaining_destroy_callouts: Vec::new(),
        send_thread_id: None,
        send_timeout_result_ptr: None,
        send_restore: None,
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

    let Some(mut callouts) = collect_destroy_wndproc_callouts(kernel, hwnd) else {
        return false;
    };
    if callouts.is_empty() {
        let time_ms = kernel.timers.tick_count();
        let destroyed = kernel.gwe.destroy_window(hwnd, time_ms);
        let result = if api_result == 0 {
            0
        } else {
            u32::from(destroyed)
        };
        let _ = uc.reg_write(RegisterMIPS::V0, u64::from(result));
        return true;
    }

    let first = callouts.remove(0);
    let return_pc = read_mips_reg(uc, RegisterMIPS::RA);
    tracing::debug!(
        target: "ce.gwe",
        hwnd = format_args!("0x{:08x}", first.hwnd),
        root = format_args!("0x{hwnd:08x}"),
        remaining = callouts.len(),
        wndproc = format_args!("0x{:08x}", first.wndproc),
        ra = format_args!("0x{return_pc:08x}"),
        "DestroyWindow guest WM_DESTROY callout"
    );

    pending_returns.borrow_mut().push(PendingWndProcReturn {
        source,
        hwnd: first.hwnd,
        msg: crate::ce::gwe::WM_DESTROY,
        wparam: 0,
        lparam: 0,
        wndproc: first.wndproc,
        return_pc,
        class_name: first.class_name,
        api_result: Some(api_result),
        dialog_result_hwnd: None,
        finalize_destroy: true,
        destroy_root_hwnd: Some(hwnd),
        remaining_destroy_callouts: callouts,
        send_thread_id: None,
        send_timeout_result_ptr: None,
        send_restore: None,
    });
    if write_wndproc_call_registers(
        uc,
        first.hwnd,
        crate::ce::gwe::WM_DESTROY,
        0,
        0,
        first.wndproc,
        WNDPROC_RETURN_STUB_ADDR,
    ) {
        true
    } else {
        let _ = pending_returns.borrow_mut().pop();
        false
    }
}

#[cfg(feature = "unicorn")]
fn collect_destroy_wndproc_callouts(
    kernel: &mut CeKernel,
    hwnd: u32,
) -> Option<Vec<DestroyWndProcCallout>> {
    let targets = kernel.gwe.window_and_descendants(hwnd)?;
    let mut callouts = Vec::new();
    for target in targets.into_iter().rev() {
        let Some(window) = kernel.gwe.window(target) else {
            continue;
        };
        if window.destroyed || window.destroy_message_sent {
            continue;
        }
        let wndproc = window.wndproc;
        let class_name = Some(window.class_name.clone());
        if is_guest_wndproc(wndproc) {
            callouts.push(DestroyWndProcCallout {
                hwnd: target,
                wndproc,
                class_name,
            });
        } else {
            let _ = kernel
                .gwe
                .record_destroy_lifecycle_message(target, crate::ce::gwe::WM_DESTROY);
        }
    }
    Some(callouts)
}

#[cfg(feature = "unicorn")]
fn write_wndproc_call_registers<D>(
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    hwnd: u32,
    msg: u32,
    wparam: u32,
    lparam: u32,
    wndproc: u32,
    return_stub: u32,
) -> bool {
    use unicorn_engine::RegisterMIPS;

    [
        uc.reg_write(RegisterMIPS::A0, u64::from(hwnd)),
        uc.reg_write(RegisterMIPS::A1, u64::from(msg)),
        uc.reg_write(RegisterMIPS::A2, u64::from(wparam)),
        uc.reg_write(RegisterMIPS::A3, u64::from(lparam)),
        uc.reg_write(RegisterMIPS::RA, u64::from(return_stub)),
        uc.reg_write(RegisterMIPS::T9, u64::from(wndproc)),
        uc.reg_write(RegisterMIPS::PC, u64::from(wndproc)),
    ]
    .into_iter()
    .all(|write| write.is_ok())
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
        destroy_root_hwnd: None,
        remaining_destroy_callouts: Vec::new(),
        send_thread_id: None,
        send_timeout_result_ptr: None,
        send_restore: None,
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
        destroy_root_hwnd: None,
        remaining_destroy_callouts: Vec::new(),
        send_thread_id: None,
        send_timeout_result_ptr: None,
        send_restore: None,
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
        destroy_root_hwnd: None,
        remaining_destroy_callouts: Vec::new(),
        send_thread_id: None,
        send_timeout_result_ptr: None,
        send_restore: None,
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
    mapped_kernel_memory: &std::rc::Rc<std::cell::RefCell<KernelMemoryMappings>>,
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

    let callout = CreateWindowReturn {
        return_pc,
        hwnd,
        wndproc,
        lparam: create_struct,
        class_name: Some(class_name),
    };
    if write_create_window_wndproc_call_registers(uc, &callout) {
        pending_returns.borrow_mut().push(callout);
        true
    } else {
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
fn handle_create_window_return_stub<D>(
    kernel: &mut CeKernel,
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    pending_returns: &std::rc::Rc<std::cell::RefCell<Vec<CreateWindowReturn>>>,
    last_wndproc_returns: &std::rc::Rc<std::cell::RefCell<Vec<UnicornWndProcReturn>>>,
) -> std::result::Result<(), ()> {
    use unicorn_engine::RegisterMIPS;

    let Some(callout) = pending_returns.borrow_mut().pop() else {
        return Err(());
    };
    let result = read_mips_reg(uc, RegisterMIPS::V0);
    record_wndproc_return(
        last_wndproc_returns,
        UnicornWndProcReturn {
            source: "CreateWindowExW/WM_CREATE",
            hwnd: callout.hwnd,
            msg: crate::ce::gwe::WM_CREATE,
            wparam: 0,
            lparam: callout.lparam,
            wndproc: callout.wndproc,
            return_pc: callout.return_pc,
            return_pc_trampoline_origin: None,
            result,
            class_name: callout.class_name.clone(),
        },
    );

    if result == u32::MAX {
        let _ = kernel
            .gwe
            .destroy_window(callout.hwnd, kernel.timers.tick_count());
        return_create_window_result(uc, callout.return_pc, 0)
    } else {
        return_create_window_result(uc, callout.return_pc, callout.hwnd)
    }
}

#[cfg(feature = "unicorn")]
fn return_create_window_result<D>(
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    return_pc: u32,
    result: u32,
) -> std::result::Result<(), ()> {
    use unicorn_engine::RegisterMIPS;

    let writes = [
        uc.reg_write(RegisterMIPS::V0, u64::from(result)),
        uc.reg_write(RegisterMIPS::PC, u64::from(return_pc)),
        uc.reg_write(RegisterMIPS::RA, u64::from(return_pc)),
    ];
    if writes.into_iter().all(|write| write.is_ok()) {
        Ok(())
    } else {
        Err(())
    }
}

#[cfg(feature = "unicorn")]
fn write_create_window_wndproc_call_registers<D>(
    uc: &mut unicorn_engine::Unicorn<'_, D>,
    callout: &CreateWindowReturn,
) -> bool {
    use unicorn_engine::RegisterMIPS;

    let writes = [
        uc.reg_write(RegisterMIPS::A0, u64::from(callout.hwnd)),
        uc.reg_write(RegisterMIPS::A1, u64::from(crate::ce::gwe::WM_CREATE)),
        uc.reg_write(RegisterMIPS::A2, 0),
        uc.reg_write(RegisterMIPS::A3, u64::from(callout.lparam)),
        uc.reg_write(RegisterMIPS::RA, u64::from(CREATE_WINDOW_RETURN_STUB_ADDR)),
        uc.reg_write(RegisterMIPS::T9, u64::from(callout.wndproc)),
        uc.reg_write(RegisterMIPS::PC, u64::from(callout.wndproc)),
    ];
    writes.into_iter().all(|write| write.is_ok())
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
        let result = default_window_proc_result(&mut kernel.gwe, hwnd, msg, wparam, lparam);
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
        destroy_root_hwnd: None,
        remaining_destroy_callouts: Vec::new(),
        send_thread_id: None,
        send_timeout_result_ptr: None,
        send_restore: None,
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
        target_module_kind: None,
        target_module_name: None,
        target_name: None,
        target_ordinal: None,
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
    trampoline_jumps: &[MipsTrampolineJump],
    memory_fault: Option<UnicornMemoryFault>,
    indirect_call_probe: Option<UnicornIndirectCallProbe>,
    host_wall_clock_stop: Option<UnicornHostWallClockStop>,
    interrupt_probe: Option<UnicornInterruptProbe>,
    invalid_instruction_probe: Option<UnicornInvalidInstructionProbe>,
    pc_stop: Option<UnicornPcStop>,
    mut last_calls: Vec<UnicornLastCall>,
    last_imports: Vec<UnicornLastImport>,
    import_milestones: Vec<UnicornLastImport>,
    file_io_stats: crate::ce::file::FileIoStats,
    scheduler_stats: crate::ce::scheduler::SchedulerStats,
    gwe_stats: crate::ce::gwe::GweStats,
    recent_file_open_ops: Vec<crate::ce::kernel::FileTraceRecord>,
    recent_file_ops: Vec<crate::ce::kernel::FileTraceRecord>,
    last_messages: Vec<UnicornLastMessage>,
    mut last_wndproc_returns: Vec<UnicornWndProcReturn>,
    mut last_wndproc_call_traces: Vec<UnicornWndProcCallTrace>,
    last_mfc_dispatch: Vec<UnicornMfcDispatchTrace>,
    last_inavi_display: Vec<UnicornInaviDisplayTrace>,
    last_inavi_controller: Vec<UnicornInaviControllerTrace>,
    inavi_render_milestones: Vec<UnicornInaviControllerTrace>,
    last_code: Vec<UnicornLastCode>,
    last_blocks: Vec<UnicornLastBlock>,
    import_counts: Vec<UnicornImportCount>,
    heap_allocation_count: usize,
    heap_allocation_bytes: u64,
    virtual_allocation_count: usize,
    virtual_allocation_bytes: u64,
    blocked_get_message: Option<UnicornBlockedGetMessage>,
    thread_exit_reached: bool,
) -> UnicornDebugSnapshot {
    use unicorn_engine::RegisterMIPS;

    let pc = read_mips_reg(uc, RegisterMIPS::PC);
    let trap = traps.trap_at(pc);
    annotate_call_import_targets(&mut last_calls, traps);
    for trace in &mut last_wndproc_call_traces {
        annotate_call_import_targets(&mut trace.calls, traps);
    }
    annotate_wndproc_return_trampolines(&mut last_wndproc_returns, trampoline_jumps);
    annotate_wndproc_call_trace_trampolines(&mut last_wndproc_call_traces, trampoline_jumps);
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
        trap_module_kind: trap.map(|trap| trap.module_kind),
        trap_module_name: trap.map(|trap| trap.module_name.clone()),
        trap_name: trap.and_then(|trap| trap.name.clone()),
        trap_ordinal: trap.and_then(|trap| trap.ordinal),
        memory_fault,
        indirect_call_probe,
        host_wall_clock_stop,
        interrupt_probe,
        invalid_instruction_probe,
        pc_stop,
        last_calls,
        last_imports,
        import_milestones,
        file_io_stats,
        scheduler_stats,
        gwe_stats,
        recent_file_open_ops,
        recent_file_ops,
        last_messages,
        last_wndproc_returns,
        last_wndproc_call_traces,
        last_mfc_dispatch,
        last_inavi_display,
        last_inavi_controller,
        inavi_render_milestones,
        last_code,
        last_blocks,
        import_counts,
        heap_allocation_count,
        heap_allocation_bytes,
        virtual_allocation_count,
        virtual_allocation_bytes,
        blocked_get_message,
        thread_exit_reached,
        encoded_kernel_exit: None,
    }
}

#[cfg(feature = "unicorn")]
fn annotate_wndproc_return_trampolines(
    returns: &mut [UnicornWndProcReturn],
    trampoline_jumps: &[MipsTrampolineJump],
) {
    for record in returns {
        record.return_pc_trampoline_origin =
            mips_trampoline_origin_for_pc(record.return_pc, trampoline_jumps);
    }
}

#[cfg(feature = "unicorn")]
fn annotate_wndproc_call_trace_trampolines(
    traces: &mut [UnicornWndProcCallTrace],
    trampoline_jumps: &[MipsTrampolineJump],
) {
    for trace in traces {
        trace.return_pc_trampoline_origin =
            mips_trampoline_origin_for_pc(trace.return_pc, trampoline_jumps);
    }
}

#[cfg(feature = "unicorn")]
fn snapshot_recent_unicorn_calls(
    last_calls: &std::rc::Rc<std::cell::RefCell<Vec<UnicornLastCall>>>,
    limit: usize,
) -> Vec<UnicornLastCall> {
    let calls = last_calls.borrow();
    let start = calls.len().saturating_sub(limit);
    calls[start..].to_vec()
}

#[cfg(feature = "unicorn")]
fn snapshot_recent_unicorn_imports(
    last_imports: &std::rc::Rc<std::cell::RefCell<Vec<UnicornLastImport>>>,
    limit: usize,
) -> Vec<UnicornLastImport> {
    let imports = last_imports.borrow();
    let start = imports.len().saturating_sub(limit);
    imports[start..].to_vec()
}

#[cfg(feature = "unicorn")]
fn snapshot_recent_unicorn_code(
    last_code: &std::rc::Rc<std::cell::RefCell<Vec<UnicornLastCode>>>,
    limit: usize,
) -> Vec<UnicornLastCode> {
    let code = last_code.borrow();
    let start = code.len().saturating_sub(limit);
    code[start..].to_vec()
}

#[cfg(feature = "unicorn")]
fn import_detail_after_return<D>(
    kernel: &CeKernel,
    uc: &unicorn_engine::Unicorn<'_, D>,
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    args: &[u32],
    result: u32,
    thread_id: u32,
) -> Option<String> {
    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll {
        return None;
    }
    match ordinal {
        Some(crate::ce::coredll_ordinals::ORD_LOAD_LIBRARY_W)
        | Some(crate::ce::coredll_ordinals::ORD_LOAD_LIBRARY_EX_W) => {
            let name_ptr = args.first().copied().unwrap_or(0);
            let last_error = kernel.threads.get_last_error(thread_id);
            let mut parts = vec![
                format!("name_ptr=0x{name_ptr:08x}"),
                format!("handle=0x{result:08x}"),
                format!("last_error={last_error}"),
            ];
            if let Some(name) = import_pointer_or_wide_arg(uc, name_ptr) {
                parts.push(format!("name={name:?}"));
            }
            if ordinal == Some(crate::ce::coredll_ordinals::ORD_LOAD_LIBRARY_EX_W) {
                parts.push(format!(
                    "file=0x{:08x}",
                    args.get(1).copied().unwrap_or_default()
                ));
                parts.push(format!(
                    "flags=0x{:08x}",
                    args.get(2).copied().unwrap_or_default()
                ));
            }
            Some(parts.join("/"))
        }
        Some(crate::ce::coredll_ordinals::ORD_GET_PROC_ADDRESS_W) => {
            let module = args.first().copied().unwrap_or(0);
            let name_ptr = args.get(1).copied().unwrap_or(0);
            let last_error = kernel.threads.get_last_error(thread_id);
            let mut parts = vec![
                format!("module=0x{module:08x}"),
                format!("name_ptr=0x{name_ptr:08x}"),
                format!("proc=0x{result:08x}"),
                format!("last_error={last_error}"),
            ];
            if let Some(name) = import_pointer_or_wide_arg(uc, name_ptr) {
                parts.push(format!("name={name:?}"));
            }
            Some(parts.join("/"))
        }
        Some(crate::ce::coredll_ordinals::ORD_GET_PROC_ADDRESS_A) => {
            let module = args.first().copied().unwrap_or(0);
            let name_ptr = args.get(1).copied().unwrap_or(0);
            let last_error = kernel.threads.get_last_error(thread_id);
            let mut parts = vec![
                format!("module=0x{module:08x}"),
                format!("name_ptr=0x{name_ptr:08x}"),
                format!("proc=0x{result:08x}"),
                format!("last_error={last_error}"),
            ];
            if name_ptr <= 0xffff && name_ptr != 0 {
                parts.push(format!("name=#{name_ptr}"));
            } else if let Some(name) = read_unicorn_narrow_z(uc, name_ptr, 128) {
                parts.push(format!("name={name:?}"));
            }
            Some(parts.join("/"))
        }
        Some(crate::ce::coredll_ordinals::ORD_FREE_LIBRARY) => {
            let module = args.first().copied().unwrap_or(0);
            let last_error = kernel.threads.get_last_error(thread_id);
            Some(format!(
                "module=0x{module:08x}/ok={result}/last_error={last_error}"
            ))
        }
        Some(crate::ce::coredll_ordinals::ORD_GET_MODULE_FILE_NAME_W) => {
            let module = args.first().copied().unwrap_or(0);
            let buffer = args.get(1).copied().unwrap_or(0);
            let max_chars = args.get(2).copied().unwrap_or(0).min(260);
            let mut parts = vec![
                format!("module=0x{module:08x}"),
                format!("buffer=0x{buffer:08x}"),
                format!("max={max_chars}"),
            ];
            if let Some(path) = read_unicorn_wide_z(uc, buffer, max_chars as usize) {
                parts.push(format!("path={path:?}"));
            }
            Some(parts.join("/"))
        }
        Some(crate::ce::coredll_ordinals::ORD_FIND_RESOURCE)
        | Some(crate::ce::coredll_ordinals::ORD_FIND_RESOURCE_W) => {
            let module = args.first().copied().unwrap_or(0);
            let name = args.get(1).copied().unwrap_or(0);
            let kind = args.get(2).copied().unwrap_or(0);
            Some(format!(
                "module=0x{module:08x}/name={}/type={}",
                import_resource_arg_detail(uc, name),
                import_resource_arg_detail(uc, kind)
            ))
        }
        Some(crate::ce::coredll_ordinals::ORD_LOAD_RESOURCE)
        | Some(crate::ce::coredll_ordinals::ORD_SIZEOF_RESOURCE) => {
            let module = args.first().copied().unwrap_or(0);
            let resource = args.get(1).copied().unwrap_or(0);
            Some(format!("module=0x{module:08x}/resource=0x{resource:08x}"))
        }
        Some(crate::ce::coredll_ordinals::ORD_LOAD_STRING_W) => {
            let module = args.first().copied().unwrap_or(0);
            let id = args.get(1).copied().unwrap_or(0);
            let buffer = args.get(2).copied().unwrap_or(0);
            let max_chars = args.get(3).copied().unwrap_or(0).min(260);
            let mut parts = vec![
                format!("module=0x{module:08x}"),
                format!("id=0x{id:08x}"),
                format!("buffer=0x{buffer:08x}"),
                format!("max={max_chars}"),
            ];
            if let Some(text) = read_unicorn_wide_z(uc, buffer, max_chars as usize) {
                parts.push(format!("text={text:?}"));
            }
            Some(parts.join("/"))
        }
        Some(crate::ce::coredll_ordinals::ORD_LOAD_MENU_W) => {
            let module = args.first().copied().unwrap_or(0);
            let name = args.get(1).copied().unwrap_or(0);
            Some(format!(
                "module=0x{module:08x}/name={}",
                import_resource_arg_detail(uc, name)
            ))
        }
        Some(crate::ce::coredll_ordinals::ORD_REGISTER_CLASS_W) => {
            let class_ptr = args.first().copied().unwrap_or(0);
            let wndproc = read_unicorn_u32(uc, class_ptr.wrapping_add(4)).unwrap_or(0);
            let class_name_ptr = read_unicorn_u32(uc, class_ptr.wrapping_add(36)).unwrap_or(0);
            let mut parts = vec![
                format!("class_ptr=0x{class_ptr:08x}"),
                format!("wndproc=0x{wndproc:08x}"),
                format!("class_name_ptr=0x{class_name_ptr:08x}"),
            ];
            if let Some(class_name) = read_unicorn_wide_z(uc, class_name_ptr, 128) {
                parts.push(format!("class={class_name:?}"));
            }
            Some(parts.join("/"))
        }
        Some(crate::ce::coredll_ordinals::ORD_CREATE_WINDOW_EX_W) => {
            let class_ptr = args.get(1).copied().unwrap_or(0);
            let title_ptr = args.get(2).copied().unwrap_or(0);
            let style = args.get(3).copied().unwrap_or(0);
            let x = args.get(4).copied().unwrap_or(0) as i32;
            let y = args.get(5).copied().unwrap_or(0) as i32;
            let width = args.get(6).copied().unwrap_or(0) as i32;
            let height = args.get(7).copied().unwrap_or(0) as i32;
            let parent = args.get(8).copied().unwrap_or(0);
            let id = args.get(9).copied().unwrap_or(0);
            let mut parts = vec![
                format!("ex_style=0x{:08x}", args.first().copied().unwrap_or(0)),
                format!("class_ptr=0x{class_ptr:08x}"),
                format!("title_ptr=0x{title_ptr:08x}"),
                format!("style=0x{style:08x}"),
                format!("rect={x},{y},{width},{height}"),
                format!("parent=0x{parent:08x}"),
                format!("id=0x{id:08x}"),
            ];
            if let Some(class_name) = import_pointer_or_wide_arg(uc, class_ptr) {
                parts.push(format!("class={class_name:?}"));
            }
            if let Some(title) = import_pointer_or_wide_arg(uc, title_ptr) {
                parts.push(format!("title={title:?}"));
            }
            Some(parts.join("/"))
        }
        Some(crate::ce::coredll_ordinals::ORD_CREATE_MUTEX_W) => {
            let attributes = args.first().copied().unwrap_or(0);
            let initial_owner = args.get(1).copied().unwrap_or(0);
            let name_ptr = args.get(2).copied().unwrap_or(0);
            let last_error = kernel.threads.get_last_error(thread_id);
            let mut parts = vec![
                format!("attributes=0x{attributes:08x}"),
                format!("initial_owner={initial_owner}"),
                format!("name_ptr=0x{name_ptr:08x}"),
                format!("handle=0x{result:08x}"),
                format!("last_error={last_error}"),
            ];
            if let Some(name) = import_pointer_or_wide_arg(uc, name_ptr) {
                parts.push(format!("name={name:?}"));
            }
            Some(parts.join("/"))
        }
        Some(crate::ce::coredll_ordinals::ORD_RELEASE_MUTEX) => {
            let handle = args.first().copied().unwrap_or(0);
            let last_error = kernel.threads.get_last_error(thread_id);
            Some(format!(
                "handle=0x{handle:08x}/ok={result}/last_error={last_error}"
            ))
        }
        Some(crate::ce::coredll_ordinals::ORD_FIND_WINDOW_W) => {
            let class_ptr = args.first().copied().unwrap_or(0);
            let title_ptr = args.get(1).copied().unwrap_or(0);
            let last_error = kernel.threads.get_last_error(thread_id);
            let mut parts = vec![
                format!("class_ptr=0x{class_ptr:08x}"),
                format!("title_ptr=0x{title_ptr:08x}"),
                format!("hwnd=0x{result:08x}"),
                format!("last_error={last_error}"),
            ];
            if let Some(class_name) = import_pointer_or_wide_arg(uc, class_ptr) {
                parts.push(format!("class={class_name:?}"));
            }
            if let Some(title) = import_pointer_or_wide_arg(uc, title_ptr) {
                parts.push(format!("title={title:?}"));
            }
            Some(parts.join("/"))
        }
        Some(crate::ce::coredll_ordinals::ORD_GET_DC) => {
            let hwnd = args.first().copied().unwrap_or(0);
            let last_error = kernel.threads.get_last_error(thread_id);
            Some(format!(
                "hwnd=0x{hwnd:08x}/hdc=0x{result:08x}/last_error={last_error}"
            ))
        }
        Some(crate::ce::coredll_ordinals::ORD_RELEASE_DC) => {
            let hwnd = args.first().copied().unwrap_or(0);
            let hdc = args.get(1).copied().unwrap_or(0);
            let last_error = kernel.threads.get_last_error(thread_id);
            Some(format!(
                "hwnd=0x{hwnd:08x}/hdc=0x{hdc:08x}/ok={result}/last_error={last_error}"
            ))
        }
        Some(crate::ce::coredll_ordinals::ORD_CREATE_COMPATIBLE_DC) => {
            let hdc = args.first().copied().unwrap_or(0);
            let last_error = kernel.threads.get_last_error(thread_id);
            Some(format!(
                "source_hdc=0x{hdc:08x}/mem_hdc=0x{result:08x}/last_error={last_error}"
            ))
        }
        Some(crate::ce::coredll_ordinals::ORD_CREATE_DIBSECTION) => {
            let hdc = args.first().copied().unwrap_or(0);
            let info = args.get(1).copied().unwrap_or(0);
            let bits_out = args.get(3).copied().unwrap_or(0);
            let section = args.get(4).copied().unwrap_or(0);
            let offset = args.get(5).copied().unwrap_or(0);
            let bits = (bits_out != 0)
                .then(|| read_unicorn_u32(uc, bits_out))
                .flatten()
                .unwrap_or(0);
            let header_size = read_unicorn_u32(uc, info).unwrap_or(0);
            let width = read_unicorn_u32(uc, info.wrapping_add(4)).unwrap_or(0) as i32;
            let height = read_unicorn_u32(uc, info.wrapping_add(8)).unwrap_or(0) as i32;
            let planes_bits = read_unicorn_u32(uc, info.wrapping_add(12)).unwrap_or(0);
            let compression = read_unicorn_u32(uc, info.wrapping_add(16)).unwrap_or(0);
            let color_entries = kernel
                .resources
                .bitmap(result)
                .map(|bitmap| bitmap.color_table.len())
                .unwrap_or(0);
            let last_error = kernel.threads.get_last_error(thread_id);
            Some(format!(
                "hdc=0x{hdc:08x}/info=0x{info:08x}/header={header_size}/width={width}/height={height}/planes={}/bpp={}/compression={compression}/colors={color_entries}/bits_out=0x{bits_out:08x}/bits=0x{bits:08x}/section=0x{section:08x}/offset={offset}/bitmap=0x{result:08x}/last_error={last_error}",
                planes_bits & 0xffff,
                (planes_bits >> 16) & 0xffff
            ))
        }
        Some(crate::ce::coredll_ordinals::ORD_SELECT_OBJECT) => {
            let hdc = args.first().copied().unwrap_or(0);
            let object = args.get(1).copied().unwrap_or(0);
            let last_error = kernel.threads.get_last_error(thread_id);
            Some(format!(
                "hdc=0x{hdc:08x}/object=0x{object:08x}/previous=0x{result:08x}/last_error={last_error}"
            ))
        }
        Some(crate::ce::coredll_ordinals::ORD_TRANSPARENT_IMAGE) => {
            let dst = args.first().copied().unwrap_or(0);
            let dst_x = args.get(1).copied().unwrap_or(0) as i32;
            let dst_y = args.get(2).copied().unwrap_or(0) as i32;
            let dst_width = args.get(3).copied().unwrap_or(0) as i32;
            let dst_height = args.get(4).copied().unwrap_or(0) as i32;
            let src = args.get(5).copied().unwrap_or(0);
            let src_x = args.get(6).copied().unwrap_or(0) as i32;
            let src_y = args.get(7).copied().unwrap_or(0) as i32;
            let src_width = args.get(8).copied().unwrap_or(0) as i32;
            let src_height = args.get(9).copied().unwrap_or(0) as i32;
            let transparent = args.get(10).copied().unwrap_or(0);
            let src_bitmap = kernel.resources.selected_bitmap(src).unwrap_or(0);
            let dst_bitmap = kernel.resources.selected_bitmap(dst).unwrap_or(0);
            let src_bitmap_detail = kernel
                .resources
                .bitmap(src_bitmap)
                .map(|bitmap| {
                    format!(
                        "src_bitmap=0x{src_bitmap:08x}:{}/{}x{}x{}:bits=0x{:08x}:stride={}",
                        if bitmap.top_down {
                            "top-down"
                        } else {
                            "bottom-up"
                        },
                        bitmap.width,
                        bitmap.height,
                        bitmap.bits_pixel,
                        bitmap.bits_ptr,
                        bitmap.width_bytes
                    )
                })
                .unwrap_or_else(|| format!("src_bitmap=0x{src_bitmap:08x}:none"));
            let dst_bitmap_detail = kernel
                .resources
                .bitmap(dst_bitmap)
                .map(|bitmap| {
                    format!(
                        "dst_bitmap=0x{dst_bitmap:08x}:{}/{}x{}x{}:bits=0x{:08x}:stride={}",
                        if bitmap.top_down {
                            "top-down"
                        } else {
                            "bottom-up"
                        },
                        bitmap.width,
                        bitmap.height,
                        bitmap.bits_pixel,
                        bitmap.bits_ptr,
                        bitmap.width_bytes
                    )
                })
                .unwrap_or_else(|| format!("dst_bitmap=0x{dst_bitmap:08x}:none"));
            let last_error = kernel.threads.get_last_error(thread_id);
            Some(format!(
                "dst=0x{dst:08x}/dst_rect={dst_x},{dst_y},{dst_width},{dst_height}/dst_memdc={}/src=0x{src:08x}/src_rect={src_x},{src_y},{src_width},{src_height}/src_memdc={}/transparent=0x{transparent:08x}/{dst_bitmap_detail}/{src_bitmap_detail}/ok={result}/last_error={last_error}",
                kernel.resources.is_memory_dc(dst),
                kernel.resources.is_memory_dc(src)
            ))
        }
        Some(crate::ce::coredll_ordinals::ORD_BIT_BLT)
        | Some(crate::ce::coredll_ordinals::ORD_STRETCH_BLT) => {
            let dst = args.first().copied().unwrap_or(0);
            let dst_x = args.get(1).copied().unwrap_or(0) as i32;
            let dst_y = args.get(2).copied().unwrap_or(0) as i32;
            let width = args.get(3).copied().unwrap_or(0) as i32;
            let height = args.get(4).copied().unwrap_or(0) as i32;
            let src = args.get(5).copied().unwrap_or(0);
            let src_x = args.get(6).copied().unwrap_or(0) as i32;
            let src_y = args.get(7).copied().unwrap_or(0) as i32;
            let rop = args.get(8).copied().unwrap_or(0);
            let last_error = kernel.threads.get_last_error(thread_id);
            Some(format!(
                "dst=0x{dst:08x}/dst_rect={dst_x},{dst_y},{width},{height}/src=0x{src:08x}/src_origin={src_x},{src_y}/rop=0x{rop:08x}/ok={result}/last_error={last_error}"
            ))
        }
        Some(crate::ce::coredll_ordinals::ORD_STRETCH_DIBITS)
        | Some(crate::ce::coredll_ordinals::ORD_SET_DIBITS_TO_DEVICE) => {
            let hdc = args.first().copied().unwrap_or(0);
            let bits = args.get(9).copied().unwrap_or(0);
            let info = args.get(10).copied().unwrap_or(0);
            let header_size = read_unicorn_u32(uc, info).unwrap_or(0);
            let width = read_unicorn_u32(uc, info.wrapping_add(4)).unwrap_or(0) as i32;
            let height = read_unicorn_u32(uc, info.wrapping_add(8)).unwrap_or(0) as i32;
            let planes_bits = read_unicorn_u32(uc, info.wrapping_add(12)).unwrap_or(0);
            let last_error = kernel.threads.get_last_error(thread_id);
            Some(format!(
                "hdc=0x{hdc:08x}/bits=0x{bits:08x}/info=0x{info:08x}/header={header_size}/width={width}/height={height}/planes={}/bpp={}/result={result}/last_error={last_error}",
                planes_bits & 0xffff,
                (planes_bits >> 16) & 0xffff
            ))
        }
        Some(crate::ce::coredll_ordinals::ORD_CREATE_FILE_W) => {
            let path_ptr = args.first().copied().unwrap_or(0);
            let path = if result != u32::MAX {
                kernel.path_for_handle(result)
            } else {
                read_unicorn_wide_z(uc, path_ptr, 260)
            };
            let access = args.get(1).copied().unwrap_or(0);
            let disposition = args.get(4).copied().unwrap_or(0);
            let mut parts = vec![
                format!("access=0x{access:08x}"),
                format!("disposition={disposition}"),
            ];
            if let Some(path) = path {
                parts.push(format!("path={path:?}"));
            }
            Some(parts.join("/"))
        }
        Some(crate::ce::coredll_ordinals::ORD_READ_FILE) => {
            let handle = args.first().copied().unwrap_or(0);
            let requested = args.get(2).copied().unwrap_or(0);
            let transferred = args
                .get(3)
                .copied()
                .filter(|ptr| *ptr != 0)
                .and_then(|ptr| read_unicorn_u32(uc, ptr));
            let mut parts = vec![
                format!("handle=0x{handle:08x}"),
                format!("requested={requested}"),
            ];
            if let Some(transferred) = transferred {
                parts.push(format!("transferred={transferred}"));
            }
            if let Some(path) = kernel.path_for_handle(handle) {
                parts.push(format!("path={path:?}"));
            }
            Some(parts.join("/"))
        }
        Some(crate::ce::coredll_ordinals::ORD_SET_FILE_POINTER) => {
            let handle = args.first().copied().unwrap_or(0);
            let method = args.get(3).copied().unwrap_or(0);
            let high = args
                .get(2)
                .copied()
                .filter(|ptr| *ptr != 0)
                .and_then(|ptr| read_unicorn_u32(uc, ptr))
                .unwrap_or(0);
            let position = ((high as u64) << 32) | result as u64;
            let mut parts = vec![
                format!("handle=0x{handle:08x}"),
                format!("method={method}"),
                format!("position=0x{position:08x}"),
            ];
            if let Some(path) = kernel.path_for_handle(handle) {
                parts.push(format!("path={path:?}"));
            }
            Some(parts.join("/"))
        }
        Some(crate::ce::coredll_ordinals::ORD_WCSNCPY) => {
            let dest = args.first().copied().unwrap_or(0);
            let src = args.get(1).copied().unwrap_or(0);
            let count = args.get(2).copied().unwrap_or(0);
            let mut parts = vec![format!("count={count}")];
            if let Some(dest_preview) = read_unicorn_wide_z(uc, dest, 32) {
                parts.push(format!("dest={dest_preview:?}"));
            }
            if let Some(src_preview) = read_unicorn_wide_z(uc, src, 32) {
                parts.push(format!("src={src_preview:?}"));
            }
            if let Some(pointer) = read_unicorn_u32(uc, src) {
                parts.push(format!("src_word=0x{pointer:08x}"));
                if let Some(deref_preview) = read_unicorn_wide_z(uc, pointer, 32) {
                    parts.push(format!("src_deref={deref_preview:?}"));
                }
            }
            Some(parts.join("/"))
        }
        Some(crate::ce::coredll_ordinals::ORD_MEMMOVE) => {
            let dest = args.first().copied().unwrap_or(0);
            let src = args.get(1).copied().unwrap_or(0);
            let count = args.get(2).copied().unwrap_or(0);
            let mut parts = vec![format!("count={count}")];
            if let Some(dest_preview) = read_unicorn_wide_z(uc, dest, 32) {
                parts.push(format!("dest_wide={dest_preview:?}"));
            }
            if let Some(src_preview) = read_unicorn_wide_z(uc, src, 32) {
                parts.push(format!("src_wide={src_preview:?}"));
            }
            Some(parts.join("/"))
        }
        Some(crate::ce::coredll_ordinals::ORD_WTOL) => {
            let text = args.first().copied().unwrap_or(0);
            let preview = read_unicorn_wide_z(uc, text, 64).unwrap_or_default();
            Some(format!("text_ptr=0x{text:08x}/text={preview:?}"))
        }
        Some(crate::ce::coredll_ordinals::ORD_ISWCTYPE) => {
            let wch = args.first().copied().unwrap_or(0);
            let ctype = args.get(1).copied().unwrap_or(0);
            Some(format!("wch=0x{:04x}/ctype=0x{ctype:08x}", wch & 0xffff))
        }
        Some(crate::ce::coredll_ordinals::ORD_WSPRINTF_W)
        | Some(crate::ce::coredll_ordinals::ORD_SWPRINTF) => {
            let dest = args.first().copied().unwrap_or(0);
            let format = args.get(1).copied().unwrap_or(0);
            let mut parts = vec![
                format!("dest=0x{dest:08x}"),
                format!("format=0x{format:08x}"),
            ];
            if let Some(format_preview) = read_unicorn_wide_z(uc, format, 128) {
                parts.push(format!("fmt={format_preview:?}"));
            }
            for (index, arg) in args.iter().skip(2).take(4).enumerate() {
                parts.push(format!("arg{index}=0x{arg:08x}"));
                #[cfg(feature = "trace")]
                parts.push(format!(
                    "arg{index}_text={}",
                    resource_pointer_preview(uc, *arg, 128)
                ));
            }
            if let Some(output) = read_unicorn_wide_z(uc, dest, 128) {
                parts.push(format!("out={output:?}"));
            }
            Some(parts.join("/"))
        }
        Some(crate::ce::coredll_ordinals::ORD_WVSPRINTF_W)
        | Some(crate::ce::coredll_ordinals::ORD_VSWPRINTF) => {
            let dest = args.first().copied().unwrap_or(0);
            let format = args.get(1).copied().unwrap_or(0);
            let va_list = args.get(2).copied().unwrap_or(0);
            let mut parts = vec![
                format!("dest=0x{dest:08x}"),
                format!("format=0x{format:08x}"),
                format!("va=0x{va_list:08x}"),
            ];
            if let Some(format_preview) = read_unicorn_wide_z(uc, format, 128) {
                parts.push(format!("fmt={format_preview:?}"));
            }
            for index in 0..4 {
                if let Some(arg) = read_unicorn_u32(uc, va_list.wrapping_add(index * 4)) {
                    parts.push(format!("va{index}=0x{arg:08x}"));
                    #[cfg(feature = "trace")]
                    parts.push(format!(
                        "va{index}_text={}",
                        resource_pointer_preview(uc, arg, 128)
                    ));
                }
            }
            if let Some(output) = read_unicorn_wide_z(uc, dest, 128) {
                parts.push(format!("out={output:?}"));
            }
            Some(parts.join("/"))
        }
        Some(crate::ce::coredll_ordinals::ORD_WCSRCHR) => {
            let string = args.first().copied().unwrap_or(0);
            let needle = args.get(1).copied().unwrap_or(0);
            let preview = read_unicorn_wide_z(uc, string, 32).unwrap_or_default();
            Some(format!("needle=0x{needle:04x}/string={preview:?}"))
        }
        Some(crate::ce::coredll_ordinals::ORD_INPUT_DEBUG_CHAR_W) => {
            let string = args.first().copied().unwrap_or(0);
            read_unicorn_wide_z(uc, string, 32).map(|preview| format!("string={preview:?}"))
        }
        _ => None,
    }
}

#[cfg(all(feature = "unicorn", feature = "trace"))]
fn is_import_milestone(
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
) -> bool {
    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll {
        return false;
    }
    matches!(
        ordinal,
        Some(crate::ce::coredll_ordinals::ORD_LOAD_LIBRARY_W)
            | Some(crate::ce::coredll_ordinals::ORD_LOAD_LIBRARY_EX_W)
            | Some(crate::ce::coredll_ordinals::ORD_GET_PROC_ADDRESS_W)
            | Some(crate::ce::coredll_ordinals::ORD_GET_PROC_ADDRESS_A)
            | Some(crate::ce::coredll_ordinals::ORD_FREE_LIBRARY)
            | Some(crate::ce::coredll_ordinals::ORD_GET_MODULE_FILE_NAME_W)
            | Some(crate::ce::coredll_ordinals::ORD_FIND_RESOURCE)
            | Some(crate::ce::coredll_ordinals::ORD_FIND_RESOURCE_W)
            | Some(crate::ce::coredll_ordinals::ORD_LOAD_RESOURCE)
            | Some(crate::ce::coredll_ordinals::ORD_SIZEOF_RESOURCE)
            | Some(crate::ce::coredll_ordinals::ORD_LOAD_STRING_W)
            | Some(crate::ce::coredll_ordinals::ORD_LOAD_MENU_W)
            | Some(crate::ce::coredll_ordinals::ORD_REGISTER_CLASS_W)
            | Some(crate::ce::coredll_ordinals::ORD_CREATE_WINDOW_EX_W)
            | Some(crate::ce::coredll_ordinals::ORD_CREATE_MUTEX_W)
            | Some(crate::ce::coredll_ordinals::ORD_RELEASE_MUTEX)
            | Some(crate::ce::coredll_ordinals::ORD_FIND_WINDOW_W)
            | Some(crate::ce::coredll_ordinals::ORD_GET_DC)
            | Some(crate::ce::coredll_ordinals::ORD_RELEASE_DC)
            | Some(crate::ce::coredll_ordinals::ORD_CREATE_COMPATIBLE_DC)
            | Some(crate::ce::coredll_ordinals::ORD_CREATE_DIBSECTION)
            | Some(crate::ce::coredll_ordinals::ORD_SELECT_OBJECT)
            | Some(crate::ce::coredll_ordinals::ORD_TRANSPARENT_IMAGE)
            | Some(crate::ce::coredll_ordinals::ORD_BIT_BLT)
            | Some(crate::ce::coredll_ordinals::ORD_STRETCH_BLT)
            | Some(crate::ce::coredll_ordinals::ORD_STRETCH_DIBITS)
            | Some(crate::ce::coredll_ordinals::ORD_SET_DIBITS_TO_DEVICE)
            | Some(crate::ce::coredll_ordinals::ORD_WSPRINTF_W)
            | Some(crate::ce::coredll_ordinals::ORD_SWPRINTF)
            | Some(crate::ce::coredll_ordinals::ORD_WVSPRINTF_W)
            | Some(crate::ce::coredll_ordinals::ORD_VSWPRINTF)
    )
}

#[cfg(feature = "unicorn")]
fn import_resource_arg_detail<D>(uc: &unicorn_engine::Unicorn<'_, D>, ptr: u32) -> String {
    import_pointer_or_wide_arg(uc, ptr)
        .map(|value| format!("{value:?}"))
        .unwrap_or_else(|| format!("0x{ptr:08x}"))
}

#[cfg(feature = "unicorn")]
fn import_pointer_or_wide_arg<D>(uc: &unicorn_engine::Unicorn<'_, D>, ptr: u32) -> Option<String> {
    if ptr == 0 {
        return None;
    }
    if ptr <= 0xffff {
        return Some(format!("#{ptr}"));
    }
    read_unicorn_wide_z(uc, ptr, 128)
}

#[cfg(feature = "unicorn")]
fn read_unicorn_wide_z<D>(
    uc: &unicorn_engine::Unicorn<'_, D>,
    ptr: u32,
    max_units: usize,
) -> Option<String> {
    if ptr == 0 {
        return None;
    }
    let mut units = Vec::new();
    for index in 0..max_units {
        let addr = ptr.wrapping_add((index as u32).wrapping_mul(2));
        let mut bytes = [0u8; 2];
        uc.mem_read(u64::from(addr), &mut bytes).ok()?;
        let unit = u16::from_le_bytes(bytes);
        if unit == 0 {
            break;
        }
        units.push(unit);
    }
    String::from_utf16(&units).ok()
}

#[cfg(feature = "unicorn")]
fn read_unicorn_narrow_z<D>(
    uc: &unicorn_engine::Unicorn<'_, D>,
    ptr: u32,
    max_bytes: usize,
) -> Option<String> {
    if ptr == 0 {
        return None;
    }
    let mut bytes = Vec::new();
    for index in 0..max_bytes {
        let addr = ptr.wrapping_add(index as u32);
        let mut byte = [0u8; 1];
        uc.mem_read(u64::from(addr), &mut byte).ok()?;
        if byte[0] == 0 {
            break;
        }
        bytes.push(byte[0]);
    }
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(all(feature = "unicorn", feature = "trace"))]
fn resource_pointer_preview<D>(
    uc: &unicorn_engine::Unicorn<'_, D>,
    ptr: u32,
    max_units: usize,
) -> String {
    let mut parts = vec![format!("ptr=0x{ptr:08x}")];
    if let Some(wide) = read_unicorn_wide_z(uc, ptr, max_units) {
        parts.push(format!("wide={wide:?}"));
    }
    if let Some(narrow) = read_unicorn_narrow_z(uc, ptr, max_units.saturating_mul(2)) {
        parts.push(format!("narrow={narrow:?}"));
    }
    if let Some(deref) = read_unicorn_u32(uc, ptr) {
        parts.push(format!("deref=0x{deref:08x}"));
        if let Some(wide) = read_unicorn_wide_z(uc, deref, max_units) {
            parts.push(format!("deref_wide={wide:?}"));
        }
        if let Some(narrow) = read_unicorn_narrow_z(uc, deref, max_units.saturating_mul(2)) {
            parts.push(format!("deref_narrow={narrow:?}"));
        }
    }
    parts.join("/")
}

fn format_trace_string(value: &str) -> String {
    format!("{value:?}")
}

fn write_unicorn_import_records(
    f: &mut std::fmt::Formatter<'_>,
    imports: &[UnicornLastImport],
) -> std::fmt::Result {
    for (index, import) in imports.iter().enumerate() {
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
            "/ra=0x{:08x}/a0=0x{:08x}/a1=0x{:08x}/a2=0x{:08x}/a3=0x{:08x}/sp=0x{:08x}",
            import.ra, import.a0, import.a1, import.a2, import.a3, import.sp
        )?;
        if let Some(result) = import.result {
            write!(f, "/ret=0x{result:08x}")?;
        } else {
            write!(f, "/ret=<pending>")?;
        }
        if let Some(detail) = import.detail.as_deref() {
            write!(f, "/detail={}", format_trace_string(detail))?;
        }
    }
    Ok(())
}

fn write_file_trace_records(
    f: &mut std::fmt::Formatter<'_>,
    records: &[crate::ce::kernel::FileTraceRecord],
) -> std::fmt::Result {
    for (index, op) in records.iter().enumerate() {
        if index != 0 {
            write!(f, ",")?;
        }
        write!(f, "{}", op.op)?;
        if let Some(handle) = op.handle {
            write!(f, "/h=0x{handle:08x}")?;
        }
        if let Some(path) = op.path.as_deref() {
            write!(f, "/path={}", format_trace_string(path))?;
        }
        if let Some(preview) = op.preview.as_deref() {
            write!(f, "/preview={}", format_trace_string(preview))?;
        }
        if let Some(requested) = op.requested {
            write!(f, "/req=0x{requested:08x}")?;
        }
        if let Some(transferred) = op.transferred {
            write!(f, "/xfer=0x{transferred:08x}")?;
        }
        if let Some(position) = op.position {
            write!(f, "/pos=0x{position:08x}")?;
        }
        if let Some(result) = op.result {
            write!(f, "/ret=0x{result:08x}")?;
        }
        if let Some(error) = op.error.as_deref() {
            write!(f, "/err={}", format_trace_string(error))?;
        }
    }
    Ok(())
}

#[cfg(feature = "unicorn")]
fn trace_import_name(
    module_kind: crate::emulator::imports::ImportModuleKind,
    ordinal: Option<u32>,
    name: Option<&str>,
) -> Option<String> {
    if let Some(name) = name {
        return Some(name.to_owned());
    }
    if module_kind != crate::emulator::imports::ImportModuleKind::Coredll {
        return None;
    }
    crate::ce::coredll_ordinals::lookup(ordinal?).map(|export| export.name.to_owned())
}

#[cfg(feature = "unicorn")]
fn is_inavi_readiness_probe_pc(pc: u32) -> bool {
    matches!(
        pc,
        0x0005_87ec
            | 0x0005_8834
            | 0x0005_88a0
            | 0x0005_88ec
            | 0x0005_8904
            | 0x0005_891c
            | 0x0005_89fc
            | 0x0005_8a04
            | 0x0005_8a34
            | 0x0005_8a3c
            | 0x0005_8a7c
            | 0x0005_8a84
            | 0x0005_8ab4
            | 0x0005_8abc
            | 0x0005_8ad8
            | 0x0005_8ae0
            | 0x0005_8b08
            | 0x0005_96e8
            | 0x0005_9704
            | 0x0005_9718
            | 0x0005_973c
            | 0x0005_9744
            | 0x0005_9750
            | 0x0005_9758
            | 0x0005_9764
            | 0x0005_976c
            | 0x0012_9564
            | 0x0012_95d0
            | 0x0012_95e0
            | 0x0001_ad94
            | 0x0001_adb0
            | 0x0001_adc0
            | 0x0001_adc8
            | 0x0001_ade4
    )
}

#[cfg(feature = "unicorn")]
fn annotate_call_import_targets(calls: &mut [UnicornLastCall], traps: &ImportTrapTable) {
    for call in calls {
        let Some(trap) = traps.trap_at(call.target) else {
            continue;
        };
        call.target_module_kind = Some(trap.module_kind);
        call.target_module_name = Some(trap.module_name.clone());
        call.target_name = trap.name.clone();
        call.target_ordinal = trap.ordinal;
    }
}

#[cfg(feature = "unicorn")]
fn mips_trampoline_origin_for_pc(pc: u32, trampoline_jumps: &[MipsTrampolineJump]) -> Option<u32> {
    trampoline_jumps.iter().find_map(|trampoline| {
        let end = trampoline.stub.checked_add(trampoline.byte_len)?;
        (pc >= trampoline.stub && pc < end).then_some(trampoline.origin)
    })
}

fn import_count_snapshot(
    counts: &std::collections::BTreeMap<UnicornImportCountKey, UnicornImportStats>,
) -> Vec<UnicornImportCount> {
    const IMPORT_COUNT_SNAPSHOT_LIMIT: usize = 256;

    let mut counts = counts
        .iter()
        .map(|(key, stats)| UnicornImportCount {
            module: key.module.clone(),
            ordinal: key.ordinal,
            name: key.name.clone(),
            count: stats.count,
            max_a0: stats.max_a0,
            max_a1: stats.max_a1,
            max_a2: stats.max_a2,
            max_a3: stats.max_a3,
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
    use crate::{ce::gwe::GWL_WNDPROC, ce::kernel::CeKernel, config::RuntimeConfig};
    use std::{cell::RefCell, rc::Rc};
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
    fn create_window_callout_returns_hwnd_or_null_after_wm_create() {
        let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
        let mut kernel = CeKernel::boot(config);
        let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();
        let thread_id = 9;
        let return_pc = 0x0040_2000;
        let wndproc = 0x0001_3570;
        let hwnd = kernel.create_window_ex_w(thread_id, "CREATE_OK", "", None, 0, 0, 0);
        kernel.gwe.set_window_long(hwnd, GWL_WNDPROC, wndproc);
        let lparam = 0x3000_0100;
        let pending = Rc::new(RefCell::new(vec![super::CreateWindowReturn {
            return_pc,
            hwnd,
            wndproc,
            lparam,
            class_name: Some("CREATE_OK".to_owned()),
        }]));
        let returns = Rc::new(RefCell::new(Vec::new()));

        uc.reg_write(RegisterMIPS::V0, 0).unwrap();
        assert!(
            super::handle_create_window_return_stub(&mut kernel, &mut uc, &pending, &returns,)
                .is_ok()
        );
        assert!(pending.borrow().is_empty());
        assert_eq!(returns.borrow().len(), 1);
        assert_eq!(returns.borrow()[0].source, "CreateWindowExW/WM_CREATE");
        assert_eq!(returns.borrow()[0].msg, crate::ce::gwe::WM_CREATE);
        assert_eq!(returns.borrow()[0].result, 0);
        assert_eq!(uc.reg_read(RegisterMIPS::V0).unwrap() as u32, hwnd);
        assert_eq!(uc.reg_read(RegisterMIPS::PC).unwrap() as u32, return_pc);

        let failed = kernel.create_window_ex_w(thread_id, "CREATE_FAIL", "", None, 0, 0, 0);
        kernel.gwe.set_window_long(failed, GWL_WNDPROC, wndproc);
        let pending = Rc::new(RefCell::new(vec![super::CreateWindowReturn {
            return_pc,
            hwnd: failed,
            wndproc,
            lparam,
            class_name: Some("CREATE_FAIL".to_owned()),
        }]));
        uc.reg_write(RegisterMIPS::V0, u64::from(u32::MAX)).unwrap();
        assert!(
            super::handle_create_window_return_stub(&mut kernel, &mut uc, &pending, &returns,)
                .is_ok()
        );
        assert!(pending.borrow().is_empty());
        assert!(!kernel.gwe.is_window(failed));
        assert_eq!(uc.reg_read(RegisterMIPS::V0).unwrap() as u32, 0);
        assert_eq!(uc.reg_read(RegisterMIPS::PC).unwrap() as u32, return_pc);
    }

    #[test]
    fn destroy_wndproc_callouts_are_guest_child_first() {
        let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
        let mut kernel = CeKernel::boot(config);
        let thread_id = 1;
        let parent = kernel.create_window_ex_w(thread_id, "PARENT", "", None, 0, 0, 0);
        let child = kernel.create_window_ex_w(thread_id, "CHILD", "", Some(parent), 1, 0, 0);
        let grandchild =
            kernel.create_window_ex_w(thread_id, "GRANDCHILD", "", Some(child), 2, 0, 0);
        let default_sibling =
            kernel.create_window_ex_w(thread_id, "DEFAULT", "", Some(parent), 3, 0, 0);
        kernel.gwe.set_window_long(parent, GWL_WNDPROC, 0x0010_1000);
        kernel.gwe.set_window_long(child, GWL_WNDPROC, 0x0010_2000);
        kernel
            .gwe
            .set_window_long(grandchild, GWL_WNDPROC, 0x0010_3000);

        let callouts = super::collect_destroy_wndproc_callouts(&mut kernel, parent).unwrap();
        let hwnds: Vec<u32> = callouts.iter().map(|callout| callout.hwnd).collect();
        let wndprocs: Vec<u32> = callouts.iter().map(|callout| callout.wndproc).collect();

        assert_eq!(hwnds, vec![grandchild, child, parent]);
        assert_eq!(wndprocs, vec![0x0010_3000, 0x0010_2000, 0x0010_1000]);
        assert!(
            kernel
                .gwe
                .window(default_sibling)
                .is_some_and(|window| window.destroy_message_sent)
        );
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
    fn beqzl_trampoline_matches_inavi_layout_loop() {
        let pc = 0x0024_fa48;
        let stub_pc = 0x00a8_9000;
        let branch = super::decode_mips_branch_likely(0x5060_fff7).unwrap();
        let words = super::branch_likely_stub_words(pc, branch, 0x2442_0001, stub_pc).unwrap();

        let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();
        uc.mem_map(0x0024_f000, 0x0000_2000, Prot::ALL).unwrap();
        uc.mem_map(0x00a8_9000, 0x0000_1000, Prot::ALL).unwrap();
        write_words(&mut uc, stub_pc, &words);
        uc.reg_write(RegisterMIPS::V1, 1).unwrap();
        uc.reg_write(RegisterMIPS::V0, 41).unwrap();
        uc.emu_start(u64::from(stub_pc), 0, 0, 4).unwrap();
        assert_eq!(uc.reg_read(RegisterMIPS::PC).unwrap(), u64::from(pc + 8));
        assert_eq!(uc.reg_read(RegisterMIPS::V0).unwrap(), 41);

        let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();
        uc.mem_map(0x0024_f000, 0x0000_2000, Prot::ALL).unwrap();
        uc.mem_map(0x00a8_9000, 0x0000_1000, Prot::ALL).unwrap();
        write_words(&mut uc, stub_pc, &words);
        uc.reg_write(RegisterMIPS::V1, 0).unwrap();
        uc.reg_write(RegisterMIPS::V0, 41).unwrap();
        uc.emu_start(u64::from(stub_pc), 0, 0, 5).unwrap();
        assert_eq!(uc.reg_read(RegisterMIPS::PC).unwrap(), u64::from(pc - 0x20));
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
    fn jal_trampoline_reaches_far_high_address_target() {
        let pc = 0x0005_7000;
        let stub_pc = 0x0005_a000;
        let target = 0x1000_832c;
        let words = super::jal_stub_words(pc, target, 0x02c0_2025, stub_pc).unwrap();

        let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();
        uc.mem_map(0x0005_a000, 0x0000_1000, Prot::ALL).unwrap();
        uc.mem_map(0x1000_8000, 0x0000_1000, Prot::ALL).unwrap();
        write_words(&mut uc, stub_pc, &words);
        write_u32(&mut uc, u64::from(target), 0x2442_0001);
        uc.reg_write(RegisterMIPS::S6, 0x1234_5678).unwrap();
        uc.reg_write(RegisterMIPS::V0, 41).unwrap();

        uc.emu_start(u64::from(stub_pc), 0, 0, 8).unwrap();

        assert_eq!(uc.reg_read(RegisterMIPS::PC).unwrap() as u32, target + 4);
        assert_eq!(uc.reg_read(RegisterMIPS::RA).unwrap(), u64::from(pc + 8));
        assert_eq!(uc.reg_read(RegisterMIPS::A0).unwrap(), 0x1234_5678);
        assert_eq!(uc.reg_read(RegisterMIPS::V0).unwrap(), 42);
    }

    #[test]
    fn jal_trampoline_builds_explorer_high_address_jump_sequence() {
        let words =
            super::jal_stub_words(0x0005_7000, 0xffff_832c, 0x02c0_2025, 0x0005_a000).unwrap();

        assert!(words.contains(&super::encode_mips_lui(26, 0xffff)));
        assert!(words.contains(&super::encode_mips_ori(26, 26, 0x832c)));
        assert!(words.contains(&super::encode_mips_jr(26)));
    }

    #[test]
    fn branch_trampoline_builds_far_high_address_jump_sequences() {
        let pc = 0x0005_7000;
        let stub_pc = 0x0005_a000;
        let branch = super::MipsBranchLikely {
            rs: 0,
            rt: 0,
            target: 0xffff_832c_u32.wrapping_sub(pc),
            inverse_branch: super::MipsBranch::Bne,
            link: false,
        };

        let words = super::normal_branch_stub_words(pc, branch, 0x2442_0001, stub_pc).unwrap();

        assert!(words.contains(&super::encode_mips_lui(26, 0xffff)));
        assert!(words.contains(&super::encode_mips_ori(26, 26, 0x832c)));
        assert!(words.contains(&super::encode_mips_jr(26)));
    }

    #[test]
    fn dll_load_base_allocator_relocates_only_colliding_images() {
        let occupied = vec![(0x0001_0000, 0x0004_0000), (0x6000_0000, 0x0002_0000)];
        let mut next_dll_base = 0x6000_0000;

        assert_eq!(
            super::choose_dll_load_base(0x6200_0000, 0x0003_0000, &occupied, &mut next_dll_base)
                .unwrap(),
            0x6200_0000
        );

        assert_eq!(
            super::choose_dll_load_base(0x0002_0000, 0x0003_0000, &occupied, &mut next_dll_base)
                .unwrap(),
            0x6013_0000
        );
        assert_eq!(next_dll_base, 0x6026_0000);
    }

    #[test]
    fn wndproc_trace_marks_return_pc_trampoline_origin() {
        let mut returns = vec![
            super::UnicornWndProcReturn {
                source: "SendMessageW",
                hwnd: 0x20000,
                msg: 0x5236,
                wparam: 0,
                lparam: 0,
                wndproc: 0x6004_eba8,
                return_pc: 0x008b_7b70,
                return_pc_trampoline_origin: None,
                result: 0,
                class_name: None,
            },
            super::UnicornWndProcReturn {
                source: "SendMessageW",
                hwnd: 0x20004,
                msg: 0x56d0,
                wparam: 0,
                lparam: 0,
                wndproc: 0x6004_eba8,
                return_pc: 0x0002_bcf4,
                return_pc_trampoline_origin: None,
                result: 0,
                class_name: None,
            },
        ];
        let jumps = [super::MipsTrampolineJump {
            origin: 0x0004_3e38,
            stub: 0x008b_7b6c,
            byte_len: 0x14,
        }];

        super::annotate_wndproc_return_trampolines(&mut returns, &jumps);

        assert_eq!(returns[0].return_pc_trampoline_origin, Some(0x0004_3e38));
        assert_eq!(returns[1].return_pc_trampoline_origin, None);
    }

    #[test]
    fn default_window_proc_consumes_paint_but_plain_wndproc_return_does_not() {
        let mut gwe = crate::ce::gwe::Gwe::default();
        let hwnd =
            gwe.create_window_ex(1, "STATIC", "paint", None, 0, crate::ce::gwe::WS_VISIBLE, 0);

        assert!(gwe.update_rect(hwnd).is_some());
        assert_eq!(
            crate::ce::gwe::default_send_message_result(crate::ce::gwe::WM_PAINT, 0, 0),
            0
        );
        assert!(gwe.update_rect(hwnd).is_some());

        assert_eq!(
            super::default_window_proc_result(&mut gwe, hwnd, crate::ce::gwe::WM_PAINT, 0, 0),
            0
        );
        assert!(gwe.update_rect(hwnd).is_none());
    }

    #[test]
    fn send_message_direct_wndproc_requires_same_thread_context() {
        assert!(super::should_direct_call_send_message_wndproc(
            1,
            0,
            1,
            0,
            0x0001_3570
        ));
        assert!(!super::should_direct_call_send_message_wndproc(
            1,
            0,
            2,
            0,
            0x0001_3570
        ));
        assert!(!super::should_direct_call_send_message_wndproc(
            1,
            0,
            1,
            0x42,
            0x0001_3570
        ));
        assert!(!super::should_direct_call_send_message_wndproc(
            1, 0, 1, 0, 0
        ));
        assert!(!super::should_direct_call_send_message_wndproc(
            1,
            0,
            1,
            0,
            crate::ce::gwe::DEFAULT_WNDPROC
        ));
    }

    #[test]
    fn send_message_receiver_context_requires_same_process_guest_wndproc() {
        assert!(super::should_receiver_context_send_message_wndproc(
            1,
            0,
            2,
            0,
            0x0001_3570
        ));
        assert!(!super::should_receiver_context_send_message_wndproc(
            1,
            0,
            1,
            0,
            0x0001_3570
        ));
        assert!(!super::should_receiver_context_send_message_wndproc(
            1,
            0,
            2,
            0x42,
            0x0001_3570
        ));
        assert!(!super::should_receiver_context_send_message_wndproc(
            1, 0, 2, 0, 0
        ));
        assert!(!super::should_receiver_context_send_message_wndproc(
            1,
            0,
            2,
            0,
            crate::ce::gwe::DEFAULT_WNDPROC
        ));
    }

    #[test]
    fn send_message_callout_enters_cross_thread_receiver_context() {
        let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
        let mut kernel = CeKernel::boot(config);
        let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();
        let sender_thread = 1;
        let receiver_thread = 2;
        let wndproc: u32 = 0x0001_3570;
        let return_pc: u32 = 0x0040_1000;
        let hwnd = kernel.create_window_ex_w(receiver_thread, "CROSS_SEND", "", None, 0, 0, 0);
        kernel.gwe.set_window_long(hwnd, GWL_WNDPROC, wndproc);
        uc.reg_write(RegisterMIPS::RA, u64::from(return_pc))
            .unwrap();
        uc.reg_write(RegisterMIPS::S0, 0x7777_0001).unwrap();

        let current_thread_id = Rc::new(RefCell::new(sender_thread));
        let running_thread = Rc::new(RefCell::new(Some((sender_thread, 0x0000_0120))));
        let blocked_waits = Rc::new(RefCell::new(Vec::new()));
        let pending = Rc::new(RefCell::new(Vec::new()));
        assert!(super::try_enter_send_message_callout(
            &mut kernel,
            &mut uc,
            crate::emulator::imports::ImportModuleKind::Coredll,
            Some(crate::ce::coredll_ordinals::ORD_SEND_MESSAGE_W),
            &[hwnd, crate::ce::gwe::WM_USER + 88, 0x55, 0x66],
            sender_thread,
            &current_thread_id,
            &running_thread,
            &blocked_waits,
            &pending,
        ));

        assert_eq!(*current_thread_id.borrow(), receiver_thread);
        assert_eq!(*running_thread.borrow(), None);
        assert_eq!(uc.reg_read(RegisterMIPS::A0).unwrap() as u32, hwnd);
        assert_eq!(
            uc.reg_read(RegisterMIPS::A1).unwrap() as u32,
            crate::ce::gwe::WM_USER + 88
        );
        assert_eq!(uc.reg_read(RegisterMIPS::A2).unwrap() as u32, 0x55);
        assert_eq!(uc.reg_read(RegisterMIPS::A3).unwrap() as u32, 0x66);
        assert_eq!(uc.reg_read(RegisterMIPS::PC).unwrap() as u32, wndproc);
        assert_eq!(
            uc.reg_read(RegisterMIPS::RA).unwrap() as u32,
            super::WNDPROC_RETURN_STUB_ADDR
        );
        let pending = pending.borrow();
        assert_eq!(pending.len(), 1);
        let restore = pending[0]
            .send_restore
            .as_ref()
            .expect("cross-thread send restore context");
        assert_eq!(restore.sender_thread_id, sender_thread);
        assert_eq!(restore.receiver_thread_id, receiver_thread);
        let blocked_waits = blocked_waits.borrow();
        assert_eq!(blocked_waits.len(), 1);
        assert_eq!(blocked_waits[0].wait_id, restore.wait_id);
        assert_eq!(blocked_waits[0].thread_id, sender_thread);
        assert_eq!(blocked_waits[0].thread_handle, 0x120);
        assert_eq!(blocked_waits[0].regs[16], 0x7777_0001);
        let super::BlockedWaitKind::SendMessage {
            send_id,
            receiver_thread_id: blocked_receiver,
            previous_running_thread,
            ..
        } = blocked_waits[0].kind
        else {
            panic!("sender should park as a SendMessage blocked wait");
        };
        assert_eq!(send_id, restore.send_id);
        assert_eq!(blocked_receiver, receiver_thread);
        assert_eq!(previous_running_thread, Some((sender_thread, 0x120)));
        assert!(kernel.blocked_waiter(restore.wait_id).is_some());
        assert!(kernel.gwe.in_send_message(receiver_thread));
        assert_eq!(
            kernel.gwe.active_sent_message_id(receiver_thread),
            Some(restore.send_id)
        );
        assert!(kernel.gwe.sent_message(restore.send_id).is_some());
    }

    #[test]
    fn send_message_blocked_wait_resume_restores_sender_context() {
        let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
        let mut kernel = CeKernel::boot(config);
        let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();
        let sender_thread = 11;
        let receiver_thread = 12;
        let sender_handle = 0x0000_0220;
        let result_ptr = 0x0001_0040;
        let return_pc = 0x0040_2200;
        uc.mem_map(0x0001_0000, 0x1000, Prot::ALL).unwrap();
        uc.mem_write(u64::from(result_ptr), &0u32.to_le_bytes())
            .unwrap();
        uc.reg_write(RegisterMIPS::S0, 0xaaaa_0001).unwrap();
        uc.reg_write(RegisterMIPS::V0, 0xeeee_0002).unwrap();
        uc.reg_write(RegisterMIPS::RA, 0x0040_1200).unwrap();

        let hwnd = kernel
            .gwe
            .create_window(receiver_thread, "SendResume", "send");
        let send_id = kernel
            .begin_cross_thread_send_message_w(
                sender_thread,
                hwnd,
                crate::ce::gwe::WM_USER + 212,
                0,
                0,
                Some(1000),
            )
            .expect("queued send");
        let wait_started_ms = kernel.timers.tick_count();
        let mut sender_regs = [0u32; 32];
        sender_regs[16] = 0x7777_0011;
        let kind = super::BlockedWaitKind::SendMessage {
            send_id,
            receiver_thread_id: receiver_thread,
            result_ptr: Some(result_ptr),
            previous_running_thread: Some((sender_thread, sender_handle)),
        };
        let wait_id = kernel.register_blocked_waiter(
            sender_thread,
            sender_handle,
            Vec::new(),
            super::scheduler_blocked_wait_kind(kind),
            wait_started_ms,
            crate::ce::timer::INFINITE,
        );
        let blocked_waits = Rc::new(RefCell::new(vec![super::BlockedWaitThread {
            wait_id,
            thread_id: sender_thread,
            thread_handle: sender_handle,
            wait_handles: Vec::new(),
            kind,
            wait_started_ms,
            timeout_ms: crate::ce::timer::INFINITE,
            regs: sender_regs,
            return_pc,
        }]));
        assert!(
            kernel
                .gwe
                .activate_sent_message_for_receiver(receiver_thread, send_id)
        );
        assert_eq!(
            kernel.complete_active_sent_message(receiver_thread, 0x2468_ace0),
            Some(send_id)
        );

        let current_thread_id = Rc::new(RefCell::new(receiver_thread));
        let suspended = Rc::new(RefCell::new(None));
        let running_thread = Rc::new(RefCell::new(None));
        assert!(super::try_resume_blocked_wait(
            &mut kernel,
            &mut uc,
            receiver_thread,
            &current_thread_id,
            &blocked_waits,
            &suspended,
            &running_thread,
        ));

        assert_eq!(*current_thread_id.borrow(), sender_thread);
        assert_eq!(
            *running_thread.borrow(),
            Some((sender_thread, sender_handle))
        );
        assert!(blocked_waits.borrow().is_empty());
        assert!(kernel.blocked_waiter(wait_id).is_none());
        assert_eq!(uc.reg_read(RegisterMIPS::S0).unwrap() as u32, 0x7777_0011);
        assert_eq!(uc.reg_read(RegisterMIPS::V0).unwrap() as u32, 0x2468_ace0);
        assert_eq!(uc.reg_read(RegisterMIPS::PC).unwrap() as u32, return_pc);
        assert_eq!(uc.reg_read(RegisterMIPS::RA).unwrap() as u32, return_pc);
        let mut result_bytes = [0; 4];
        uc.mem_read(u64::from(result_ptr), &mut result_bytes)
            .unwrap();
        assert_eq!(u32::from_le_bytes(result_bytes), 0x2468_ace0);
        assert!(kernel.take_completed_send_message_result(send_id).is_none());
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

    #[test]
    fn trampoline_scan_finds_halfword_jump_table_after_long_setup() {
        let mut mapped = vec![0; 0x180];
        write_vec_u32(&mut mapped, 0x10, 0x2c6b_001d);
        let setup = 0x118;
        write_vec_u32(&mut mapped, setup, 0x3c06_0000);
        write_vec_u32(&mut mapped, setup + 4, 0x24c6_0138);
        write_vec_u32(&mut mapped, setup + 8, 0x0003_2840);
        write_vec_u32(&mut mapped, setup + 12, 0x00a6_2821);
        write_vec_u32(&mut mapped, setup + 16, 0x84a5_0000);
        write_vec_u32(&mut mapped, setup + 20, 0x00c5_3021);
        write_vec_u32(&mut mapped, setup + 24, 0x00c0_0008);
        write_vec_u32(&mut mapped, setup + 28, 0x0000_0000);
        for index in 0..0x1d {
            let entry = (0x40 + index * 4) as u16;
            let offset = 0x138 + index * 2;
            mapped[offset..offset + 2].copy_from_slice(&entry.to_le_bytes());
        }

        let ranges =
            super::mips_halfword_jump_table_ranges(&mapped, 0, 0, mapped.len() as u32, "test")
                .unwrap();

        assert_eq!(ranges, vec![(0x138, 0x3a)]);
        assert!(super::mips_patch_rva_overlaps_data_ranges(0x138, &ranges));
        assert!(super::mips_patch_rva_overlaps_data_ranges(0x170, &ranges));
        assert!(!super::mips_patch_rva_overlaps_data_ranges(0x174, &ranges));
    }

    #[test]
    fn trampoline_scan_skips_byte_jump_table_data() {
        let mut mapped = vec![0; 0x80];
        write_vec_u32(&mut mapped, 0x10, 0x2ce6_0006);
        write_vec_u32(&mut mapped, 0x20, 0x3c05_0000);
        write_vec_u32(&mut mapped, 0x24, 0x24a5_003c);
        write_vec_u32(&mut mapped, 0x28, 0x00e5_2021);
        write_vec_u32(&mut mapped, 0x2c, 0x8084_0000);
        write_vec_u32(&mut mapped, 0x30, 0x00a4_2821);
        write_vec_u32(&mut mapped, 0x34, 0x00a0_0008);
        write_vec_u32(&mut mapped, 0x38, 0x0000_0000);
        mapped[0x3c..0x42].copy_from_slice(&[0x08, 0x14, 0x20, 0x2c, 0x38, 0x44]);

        let ranges =
            super::mips_byte_jump_table_ranges(&mapped, 0, 0, mapped.len() as u32, "test").unwrap();

        assert_eq!(ranges, vec![(0x3c, 6)]);
        assert!(!super::mips_patch_rva_overlaps_data_ranges(0x34, &ranges));
        assert!(super::mips_patch_rva_overlaps_data_ranges(0x38, &ranges));
        assert!(super::mips_patch_rva_overlaps_data_ranges(0x3c, &ranges));
        assert!(super::mips_patch_rva_overlaps_data_ranges(0x40, &ranges));
        assert!(!super::mips_patch_rva_overlaps_data_ranges(0x44, &ranges));
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
