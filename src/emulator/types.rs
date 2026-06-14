use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoadedPeModuleInfo {
    pub name: String,
    pub base: u32,
    pub guest_path: Option<String>,
    pub host_path: Option<std::path::PathBuf>,
    pub image_size: u32,
    pub entry_point: u32,
    pub dependencies: Vec<String>,
    pub tls_callbacks: Vec<u32>,
    pub load_flags: u32,
    pub dynamic: bool,
    pub exports_by_name: HashMap<String, u32>,
    pub exports_by_ordinal: HashMap<u32, u32>,
    pub forwarders_by_name: HashMap<String, String>,
    pub forwarders_by_ordinal: HashMap<u32, String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UnicornRunLimits {
    pub instruction_limit: usize,
    pub wall_clock_limit_ms: u64,
    pub stop_pc: Option<u32>,
    pub live_pump: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UnicornDebugSnapshot {
    pub pc: u32,
    pub pc_region: Option<String>,
    pub ra: u32,
    pub ra_region: Option<String>,
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
    pub trap_handle: Option<UnicornWaitHandleSnapshot>,
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
    pub runtime_loader_stats: crate::ce::kernel::RuntimeLoaderStats,
    pub active_timers: Vec<UnicornTimerSnapshot>,
    pub active_blocked_waits: Vec<UnicornBlockedWaitSnapshot>,
    pub recent_file_open_ops: Vec<crate::ce::kernel::FileTraceRecord>,
    pub recent_file_ops: Vec<crate::ce::kernel::FileTraceRecord>,
    pub recent_process_ops: Vec<crate::ce::kernel::ProcessTraceRecord>,
    pub recent_event_ops: Vec<crate::ce::kernel::EventTraceRecord>,
    pub recent_message_ops: Vec<crate::ce::kernel::MessageTraceRecord>,
    pub last_messages: Vec<UnicornLastMessage>,
    pub last_wndproc_returns: Vec<UnicornWndProcReturn>,
    pub last_wndproc_call_traces: Vec<UnicornWndProcCallTrace>,
    pub last_mfc_dispatch: Vec<UnicornMfcDispatchTrace>,
    pub last_inavi_display: Vec<UnicornInaviDisplayTrace>,
    pub last_inavi_controller: Vec<UnicornInaviControllerTrace>,
    pub inavi_render_milestones: Vec<UnicornInaviControllerTrace>,
    pub presentation_imports: Vec<UnicornLastImport>,
    pub window_imports: Vec<UnicornLastImport>,
    pub guest_entry_traces: Vec<UnicornGuestEntryTrace>,
    pub last_code: Vec<UnicornLastCode>,
    pub last_blocks: Vec<UnicornLastBlock>,
    pub import_counts: Vec<UnicornImportCount>,
    pub z_order: Vec<u32>,
    pub windows: Vec<UnicornWindowSnapshot>,
    pub heap_allocation_count: usize,
    pub heap_allocation_bytes: u64,
    pub virtual_allocation_count: usize,
    pub virtual_allocation_bytes: u64,
    pub blocked_get_message: Option<UnicornBlockedGetMessage>,
    pub cross_process_send_yield: Option<UnicornCrossProcessSendYield>,
    pub thread_state: Option<String>,
    pub thread_exit_reached: bool,
    pub encoded_kernel_exit: Option<EncodedKernelExit>,
}

impl UnicornDebugSnapshot {
    pub fn summary(&self) -> String {
        let mut parts = vec![
            format_address_with_region("pc", self.pc, self.pc_region.as_deref()),
            format_address_with_region("ra", self.ra, self.ra_region.as_deref()),
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
        if let Some(handle) = &self.trap_handle {
            parts.push(format!(
                "trap_handle=0x{:08x}:{}",
                handle.handle, handle.description
            ));
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
        if self.runtime_loader_stats.load_attempt_count != 0
            || self.runtime_loader_stats.successful_map_count != 0
            || self.runtime_loader_stats.dependency_load_count != 0
            || self.runtime_loader_stats.export_lookup_count != 0
            || self.runtime_loader_stats.forwarded_export_count != 0
            || self.runtime_loader_stats.tls_callback_count != 0
            || self.runtime_loader_stats.dllmain_attach_count != 0
            || self.runtime_loader_stats.dllmain_detach_count != 0
            || self.runtime_loader_stats.loud_failure_count != 0
        {
            parts.push(format!(
                "loader=load:{}/map:{} dep:{} getproc:{}/miss:{} fwd:{} tls:{} dllmain:{}/{} fail:{}",
                self.runtime_loader_stats.load_attempt_count,
                self.runtime_loader_stats.successful_map_count,
                self.runtime_loader_stats.dependency_load_count,
                self.runtime_loader_stats.export_lookup_count,
                self.runtime_loader_stats.export_lookup_miss_count,
                self.runtime_loader_stats.forwarded_export_count,
                self.runtime_loader_stats.tls_callback_count,
                self.runtime_loader_stats.dllmain_attach_count,
                self.runtime_loader_stats.dllmain_detach_count,
                self.runtime_loader_stats.loud_failure_count
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
            || self.scheduler_stats.serial_event_signal_count != 0
            || self.scheduler_stats.send_reply_signal_count != 0
        {
            parts.push(format!(
                "sched=wait:{}/{}/{} sleep:{} yield:{} ok:{} timeout:{} fail:{} block:{} wake:{} reg:{}/{} maxreg:{} sig:{} cand:{} msgsig:{} msgcand:{} sersig:{} sercand:{} serevsig:{} serevcand:{} sendsig:{} sendcand:{} maxpend:{}",
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
                self.scheduler_stats.serial_event_signal_count,
                self.scheduler_stats.serial_event_wake_candidate_count,
                self.scheduler_stats.send_reply_signal_count,
                self.scheduler_stats.send_reply_wake_candidate_count,
                self.scheduler_stats.max_pending_wakes
            ));
        }
        if !self.active_timers.is_empty() {
            let mut timer_summary = String::new();
            for (index, timer) in self.active_timers.iter().take(8).enumerate() {
                if index != 0 {
                    timer_summary.push(';');
                }
                timer_summary.push_str(&format!(
                    "id=0x{:x}/thr={}/hwnd=0x{:08x}/msg=0x{:x}/cb=0x{:08x}/due={}/period={}",
                    timer.id,
                    timer.thread_id,
                    timer.hwnd.unwrap_or_default(),
                    timer.message,
                    timer.callback.unwrap_or_default(),
                    timer.due_ms,
                    timer.period_ms.unwrap_or_default()
                ));
            }
            if self.active_timers.len() > 8 {
                timer_summary.push_str(&format!(";+{} more", self.active_timers.len() - 8));
            }
            parts.push(format!("timers=[{timer_summary}]"));
        }
        if !self.active_blocked_waits.is_empty() {
            let mut waits_summary = String::new();
            for (index, wait) in self.active_blocked_waits.iter().take(16).enumerate() {
                if index != 0 {
                    waits_summary.push(';');
                }
                waits_summary.push_str(&format!(
                    "id={}/thr={}/kind={}/timeout={}/handles=",
                    wait.id, wait.thread_id, wait.kind, wait.timeout_ms
                ));
                if wait.handles.is_empty() {
                    waits_summary.push_str("none");
                } else {
                    for (handle_index, handle) in wait.handles.iter().take(4).enumerate() {
                        if handle_index != 0 {
                            waits_summary.push('|');
                        }
                        waits_summary
                            .push_str(&format!("0x{:08x}:{}", handle.handle, handle.description));
                    }
                    if wait.handles.len() > 4 {
                        waits_summary.push_str(&format!("|+{} more", wait.handles.len() - 4));
                    }
                }
            }
            if self.active_blocked_waits.len() > 16 {
                waits_summary.push_str(&format!(";+{} more", self.active_blocked_waits.len() - 16));
            }
            parts.push(format!("blocked_waits=[{waits_summary}]"));
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
            if !indirect.stack_words.is_empty() {
                let mut stack_summary = String::new();
                for (index, (offset, value)) in indirect.stack_words.iter().enumerate() {
                    if index != 0 {
                        stack_summary.push(',');
                    }
                    match value {
                        Some(value) => {
                            stack_summary.push_str(&format!("+0x{offset:x}=0x{value:08x}"));
                        }
                        None => stack_summary.push_str(&format!("+0x{offset:x}=<unreadable>")),
                    }
                }
                parts.push(format!("indirect_stack=[{stack_summary}]"));
            }
        }
        if !self.guest_entry_traces.is_empty() {
            let mut entries = String::new();
            for (index, entry) in self.guest_entry_traces.iter().rev().take(8).enumerate() {
                if index != 0 {
                    entries.push(';');
                }
                entries.push_str(&format!(
                    "{} pc=0x{:08x}/ra=0x{:08x}/sp=0x{:08x}/a0=0x{:08x}/a1=0x{:08x}",
                    entry.label, entry.pc, entry.ra, entry.sp, entry.a0, entry.a1
                ));
                if !entry.stack_words.is_empty() {
                    entries.push('[');
                    for (word_index, (offset, value)) in entry.stack_words.iter().enumerate() {
                        if word_index != 0 {
                            entries.push(',');
                        }
                        match value {
                            Some(value) => {
                                entries.push_str(&format!("+0x{offset:x}=0x{value:08x}"));
                            }
                            None => entries.push_str(&format!("+0x{offset:x}=<unreadable>")),
                        }
                    }
                    entries.push(']');
                }
            }
            parts.push(format!("guest_entries=[{entries}]"));
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
        if let Some(yielded) = &self.cross_process_send_yield {
            parts.push(format!(
                "cross_process_send=sender:{}:{} target:{}:{} hwnd=0x{:08x} msg=0x{:04x}",
                yielded.sender_process_id,
                yielded.sender_thread_id,
                yielded.target_process_id,
                yielded.target_thread_id,
                yielded.hwnd,
                yielded.msg
            ));
        }
        if let Some(thread_state) = &self.thread_state {
            parts.push(format!("threads={thread_state}"));
        }
        if let Some(exit) = &self.encoded_kernel_exit {
            parts.push(format!(
                "encoded_exit=api{}.{} process=0x{:08x} code=0x{:08x}",
                exit.api_set, exit.method, exit.process, exit.exit_code
            ));
        }
        if let Some(interrupt) = &self.interrupt_probe {
            parts.push(format!(
                "interrupt={} pc=0x{:08x} ra=0x{:08x} sp=0x{:08x} last_pc={} last_insn={}",
                interrupt.intno,
                interrupt.pc,
                interrupt.ra,
                interrupt.sp,
                interrupt
                    .last_code_pc
                    .map(|pc| format!("0x{pc:08x}"))
                    .unwrap_or_else(|| "none".to_owned()),
                interrupt
                    .last_code_instruction
                    .map(|instruction| format!("0x{instruction:08x}"))
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
    pub stack_words: Vec<(u32, Option<u32>)>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UnicornGuestEntryTrace {
    pub label: &'static str,
    pub pc: u32,
    pub ra: u32,
    pub sp: u32,
    pub a0: u32,
    pub a1: u32,
    pub a2: u32,
    pub a3: u32,
    pub t9: u32,
    pub stack_words: Vec<(u32, Option<u32>)>,
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
pub struct UnicornCrossProcessSendYield {
    pub send_id: u64,
    pub sender_thread_id: u32,
    pub sender_process_id: u32,
    pub target_thread_id: u32,
    pub target_process_id: u32,
    pub hwnd: u32,
    pub msg: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornInterruptProbe {
    pub pc: u32,
    pub ra: u32,
    pub sp: u32,
    pub v0: u32,
    pub v1: u32,
    pub a0: u32,
    pub a1: u32,
    pub a2: u32,
    pub a3: u32,
    pub s0: u32,
    pub s1: u32,
    pub s2: u32,
    pub s3: u32,
    pub s4: u32,
    pub s5: u32,
    pub s6: u32,
    pub s7: u32,
    pub fp: u32,
    pub t9: u32,
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
    pub detail: Option<String>,
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
pub struct UnicornBlockedWaitSnapshot {
    pub id: u64,
    pub thread_id: u32,
    pub thread_handle: u32,
    pub kind: String,
    pub wait_started_ms: u32,
    pub timeout_ms: u32,
    pub handles: Vec<UnicornWaitHandleSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornWaitHandleSnapshot {
    pub handle: u32,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornTimerSnapshot {
    pub id: u32,
    pub thread_id: u32,
    pub hwnd: Option<u32>,
    pub message: u32,
    pub callback: Option<u32>,
    pub due_ms: u32,
    pub period_ms: Option<u32>,
}

impl From<crate::ce::timer::KernelTimer> for UnicornTimerSnapshot {
    fn from(timer: crate::ce::timer::KernelTimer) -> Self {
        Self {
            id: timer.id,
            thread_id: timer.thread_id,
            hwnd: timer.hwnd,
            message: timer.message,
            callback: timer.callback,
            due_ms: timer.due_ms,
            period_ms: timer.period_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornWindowSnapshot {
    pub hwnd: u32,
    pub thread_id: u32,
    pub process_id: u32,
    pub class_name: String,
    pub title: String,
    pub visible: bool,
    pub being_destroyed: bool,
    pub destroyed: bool,
    pub parent: Option<u32>,
    pub owner: Option<u32>,
    pub style: u32,
    pub ex_style: u32,
    pub update_pending: bool,
    pub erase_pending: bool,
    pub pending_move: bool,
    pub pending_size: bool,
    pub rect: crate::ce::gwe::Rect,
    pub client_rect: crate::ce::gwe::Rect,
    pub update_rect: crate::ce::gwe::Rect,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicornMappedBlobRange {
    pub name: String,
    pub base: u32,
    pub size: u32,
}

pub(super) fn format_address_with_region(
    label: &str,
    address: u32,
    region: Option<&str>,
) -> String {
    match region {
        Some(region) => format!("{label}=0x{address:08x}({region})"),
        None => format!("{label}=0x{address:08x}(unknown)"),
    }
}

// The unicorn feature provides a detailed Display impl; this stub covers non-unicorn builds.
#[cfg(not(feature = "unicorn"))]
impl std::fmt::Display for UnicornDebugSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.summary())
    }
}
