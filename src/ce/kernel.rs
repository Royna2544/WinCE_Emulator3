use crate::{
    ce::{
        audio::{AudioSystem, MmResult, WaveBuffer, WaveFormat},
        cemath::CeMath,
        com::ComSystem,
        devices::{DeviceIoControlResult, DeviceNamespace, PURGE_RXCLEAR},
        file::{
            FileIoResult, FileIoStats, FindData, GENERIC_READ, GENERIC_WRITE, HostFileSystem,
            OPEN_EXISTING,
        },
        gwe::{
            Gwe, GweStats, HWND_BROADCAST, HWND_TOP, Message, MessagePointerPayload, PeekFlags,
            Point, Rect, SMF_NULL, SWP_HIDEWINDOW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
            SWP_NOZORDER, SWP_SHOWWINDOW, WA_ACTIVE, WA_INACTIVE, WM_ACTIVATE, WM_CANCELMODE,
            WM_ENABLE, WM_KILLFOCUS, WM_MOVE, WM_SETFOCUS, WM_SHOWWINDOW, WM_SIZE,
            WM_WINDOWPOSCHANGED, WindowPos,
        },
        memory::{MemorySystem, PROCESS_HEAP_HANDLE},
        object::{
            CE_THREAD_PRIORITY_NORMAL, FileObject, FindFileObject, HandleTable, KernelObject,
            MAX_SUSPEND_COUNT, ThreadResumeResult, ThreadSuspendResult, WaitMultipleResult,
            WaitResult, ce_thread_priority_to_win32, win32_thread_priority_to_ce,
        },
        registry::Registry,
        remote::{CeRemote, RemoteStatus, WM_LBUTTONDOWN, WM_MOUSEMOVE, make_lparam},
        resource::ResourceSystem,
        scheduler::{
            Scheduler, SchedulerBlockedWait, SchedulerBlockedWaitKind, SchedulerStats,
            SchedulerWaitKind, SchedulerWakeReason,
        },
        thread::ThreadSystem,
        timer::{TimerSystem, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT},
    },
    config::RuntimeConfig,
    error::{Error, Result},
};

use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessagePumpResult {
    Dispatched(u32),
    Quit(u32),
    Idle,
}

#[derive(Debug, Clone)]
pub struct CeKernel {
    pub registry: Registry,
    pub devices: DeviceNamespace,
    pub files: HostFileSystem,
    pub handles: HandleTable,
    pub gwe: Gwe,
    pub audio: AudioSystem,
    pub math: CeMath,
    pub timers: TimerSystem,
    pub remote: CeRemote,
    pub threads: ThreadSystem,
    pub scheduler: Scheduler,
    pub resources: ResourceSystem,
    pub com: ComSystem,
    pub memory: MemorySystem,
    process_module_base: u32,
    process_module_path: String,
    process_module_host_path: Option<PathBuf>,
    process_command_line: String,
    current_process_id: u32,
    current_process_exit_code: u32,
    current_process_signaled: bool,
    thread_priority_overrides: BTreeMap<u32, i32>,
    thread_suspend_counts: BTreeMap<u32, u32>,
    pending_process_launches: Vec<PendingProcessLaunch>,
    next_process_id: u32,
    loaded_modules: BTreeMap<String, LoadedModule>,
    crt_rand_state: u32,
    crt_strtok_next_by_thread: BTreeMap<u32, u32>,
    recent_file_ops: Vec<FileTraceRecord>,
    recent_file_open_ops: Vec<FileTraceRecord>,
    recent_process_ops: Vec<ProcessTraceRecord>,
    recent_event_ops: Vec<EventTraceRecord>,
    recent_message_ops: Vec<MessageTraceRecord>,
    pulsed_wait_handles: BTreeMap<u64, u32>,
    comm_event_mask_changed_waits: BTreeSet<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedModule {
    pub name: String,
    pub base: u32,
    pub exports_by_name: BTreeMap<String, u32>,
    pub exports_by_ordinal: BTreeMap<u32, u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingProcessLaunch {
    pub application: Option<String>,
    pub command_line: Option<String>,
    pub process_handle: u32,
    pub thread_handle: u32,
    pub process_id: u32,
    pub thread_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTraceRecord {
    pub op: &'static str,
    pub handle: Option<u32>,
    pub path: Option<String>,
    pub preview: Option<String>,
    pub requested: Option<u32>,
    pub transferred: Option<u32>,
    pub position: Option<u64>,
    pub result: Option<u32>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessTraceRecord {
    pub op: &'static str,
    pub application: Option<String>,
    pub command_line: Option<String>,
    pub path: Option<String>,
    pub process_handle: Option<u32>,
    pub thread_handle: Option<u32>,
    pub process_id: Option<u32>,
    pub thread_id: Option<u32>,
    pub result: Option<u32>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventTraceRecord {
    pub op: &'static str,
    pub handle: Option<u32>,
    pub name: Option<String>,
    pub manual_reset: Option<bool>,
    pub signaled: Option<bool>,
    pub result: Option<bool>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageTraceRecord {
    pub op: &'static str,
    pub thread_id: u32,
    pub hwnd: Option<u32>,
    pub msg: Option<u32>,
    pub wparam: Option<u32>,
    pub lparam: Option<u32>,
    pub screen_pos: Option<u32>,
    pub source: Option<u32>,
    pub result: Option<u32>,
    pub detail: Option<String>,
}

const FILE_TRACE_LIMIT: usize = 512;
const FILE_TRACE_PREVIEW_LIMIT: usize = 64;
const PROCESS_TRACE_LIMIT: usize = 128;
const EVENT_TRACE_LIMIT: usize = 256;
const MESSAGE_TRACE_LIMIT: usize = 512;

pub fn normalize_module_name(name: &str) -> String {
    name.trim()
        .trim_end_matches('\0')
        .trim_end_matches(".dll")
        .trim_end_matches(".DLL")
        .to_ascii_lowercase()
}

pub fn normalize_symbol_name(name: &str) -> String {
    name.trim_start_matches('_')
        .split('@')
        .next()
        .unwrap_or(name)
        .to_ascii_lowercase()
}

fn wait_result_to_wake_reason(result: u32) -> SchedulerWakeReason {
    if result == WAIT_FAILED {
        SchedulerWakeReason::Failed
    } else if result == WAIT_TIMEOUT {
        SchedulerWakeReason::Timeout
    } else {
        SchedulerWakeReason::ObjectSignaled
    }
}

const MAXIMUM_WAIT_OBJECTS: usize = 64;
pub const CE_SYS_HANDLE_BASE: u32 = 64;
pub const CE_SH_CURTHREAD: u32 = 1;
pub const CE_SH_CURPROC: u32 = 2;
pub const CE_CURRENT_THREAD_PSEUDO_HANDLE: u32 = CE_SYS_HANDLE_BASE + CE_SH_CURTHREAD;
pub const CE_CURRENT_PROCESS_PSEUDO_HANDLE: u32 = CE_SYS_HANDLE_BASE + CE_SH_CURPROC;
pub const STILL_ACTIVE: u32 = 259;

impl CeKernel {
    pub fn boot(config: RuntimeConfig) -> Self {
        Self {
            registry: Registry::from_dump(config.registry),
            devices: DeviceNamespace::from_config(config.devices),
            files: HostFileSystem::from_storage(config.storage),
            handles: HandleTable::default(),
            gwe: Gwe::default(),
            audio: AudioSystem::default(),
            math: CeMath,
            timers: TimerSystem::default(),
            remote: CeRemote::default(),
            threads: ThreadSystem::default(),
            scheduler: Scheduler::default(),
            resources: ResourceSystem::default(),
            com: ComSystem::default(),
            memory: MemorySystem::default(),
            process_module_base: 0,
            process_module_path: "\\FakeCE\\process.exe".to_owned(),
            process_module_host_path: None,
            process_command_line: String::new(),
            current_process_id: 1,
            current_process_exit_code: STILL_ACTIVE,
            current_process_signaled: false,
            thread_priority_overrides: BTreeMap::new(),
            thread_suspend_counts: BTreeMap::new(),
            pending_process_launches: Vec::new(),
            next_process_id: 0x42,
            loaded_modules: BTreeMap::new(),
            crt_rand_state: 1,
            crt_strtok_next_by_thread: BTreeMap::new(),
            recent_file_ops: Vec::new(),
            recent_file_open_ops: Vec::new(),
            recent_process_ops: Vec::new(),
            recent_event_ops: Vec::new(),
            recent_message_ops: Vec::new(),
            pulsed_wait_handles: BTreeMap::new(),
            comm_event_mask_changed_waits: BTreeSet::new(),
        }
    }

    pub fn crt_srand(&mut self, seed: u32) {
        self.crt_rand_state = seed;
    }

    pub fn crt_rand(&mut self) -> u32 {
        self.crt_rand_state = self
            .crt_rand_state
            .wrapping_mul(214013)
            .wrapping_add(2531011);
        (self.crt_rand_state >> 16) & 0x7fff
    }

    pub fn crt_strtok_next(&self, thread_id: u32) -> u32 {
        self.crt_strtok_next_by_thread
            .get(&thread_id)
            .copied()
            .unwrap_or(0)
    }

    pub fn crt_set_strtok_next(&mut self, thread_id: u32, ptr: u32) {
        if ptr == 0 {
            self.crt_strtok_next_by_thread.remove(&thread_id);
        } else {
            self.crt_strtok_next_by_thread.insert(thread_id, ptr);
        }
    }

    pub fn set_process_module_base(&mut self, base: u32) {
        self.process_module_base = base;
    }

    pub fn process_module_base(&self) -> u32 {
        self.process_module_base
    }

    pub fn register_loaded_module(
        &mut self,
        name: impl Into<String>,
        base: u32,
        exports_by_name: BTreeMap<String, u32>,
        exports_by_ordinal: BTreeMap<u32, u32>,
    ) {
        let name = name.into();
        let exports_by_name = exports_by_name
            .into_iter()
            .map(|(name, address)| (normalize_symbol_name(&name), address))
            .collect();
        self.loaded_modules.insert(
            normalize_module_name(&name),
            LoadedModule {
                name,
                base,
                exports_by_name,
                exports_by_ordinal,
            },
        );
    }

    pub fn loaded_module_handle(&self, name: &str) -> Option<u32> {
        self.loaded_modules
            .get(&normalize_module_name(name))
            .map(|module| module.base)
    }

    pub fn is_loaded_module_handle(&self, module: u32) -> bool {
        self.loaded_modules
            .values()
            .any(|loaded| loaded.base == module)
    }

    pub fn resolve_loaded_module_proc_by_name(&self, module: u32, name: &str) -> Option<u32> {
        let symbol = normalize_symbol_name(name);
        self.loaded_modules
            .values()
            .find(|loaded| loaded.base == module)?
            .exports_by_name
            .get(&symbol)
            .copied()
    }

    pub fn resolve_loaded_module_proc_by_ordinal(&self, module: u32, ordinal: u32) -> Option<u32> {
        self.loaded_modules
            .values()
            .find(|loaded| loaded.base == module)?
            .exports_by_ordinal
            .get(&ordinal)
            .copied()
    }

    pub fn set_process_module_path(&mut self, path: impl Into<String>) {
        let path = path.into();
        self.files.set_root_relative_guest_path(&path);
        self.process_module_path = path;
    }

    pub fn process_module_path(&self) -> &str {
        &self.process_module_path
    }

    pub fn set_process_module_host_path(&mut self, path: impl Into<PathBuf>) {
        self.process_module_host_path = Some(path.into());
    }

    pub fn process_module_host_path(&self) -> Option<&PathBuf> {
        self.process_module_host_path.as_ref()
    }

    pub fn set_process_command_line(&mut self, command_line: impl Into<String>) {
        self.process_command_line = command_line.into();
    }

    pub fn process_command_line(&self) -> &str {
        &self.process_command_line
    }

    pub fn set_current_process_id(&mut self, process_id: u32) {
        self.current_process_id = process_id;
    }

    pub fn current_process_id(&self) -> u32 {
        self.current_process_id
    }

    pub fn is_current_thread_pseudo_handle(handle: u32) -> bool {
        handle == CE_CURRENT_THREAD_PSEUDO_HANDLE
    }

    pub fn is_current_process_pseudo_handle(handle: u32) -> bool {
        handle == CE_CURRENT_PROCESS_PSEUDO_HANDLE
    }

    pub fn queue_process_launch(
        &mut self,
        application: Option<String>,
        command_line: Option<String>,
    ) -> PendingProcessLaunch {
        let process_id = self.next_process_id;
        self.next_process_id = self.next_process_id.saturating_add(1);
        let thread_id = self.threads.allocate_guest_thread_id();
        let process_handle = self.handles.create_process(process_id);
        let thread_handle = self.handles.create_thread(thread_id, 0, 0, false);
        let launch = PendingProcessLaunch {
            application,
            command_line,
            process_handle,
            thread_handle,
            process_id,
            thread_id,
        };
        self.pending_process_launches.push(launch.clone());
        self.record_process_trace(ProcessTraceRecord {
            op: "CreateProcessWQueued",
            application: launch.application.clone(),
            command_line: launch.command_line.clone(),
            path: None,
            process_handle: Some(process_handle),
            thread_handle: Some(thread_handle),
            process_id: Some(process_id),
            thread_id: Some(thread_id),
            result: Some(1),
            error: None,
        });
        launch
    }

    pub fn take_pending_process_launches(&mut self) -> Vec<PendingProcessLaunch> {
        std::mem::take(&mut self.pending_process_launches)
    }

    pub fn mark_process_launch_exited(&mut self, launch: &PendingProcessLaunch, exit_code: u32) {
        self.destroy_process_windows(launch.process_id, launch.thread_id);
        if self
            .handles
            .mark_process_exited(launch.process_handle, exit_code)
        {
            self.queue_object_wake_candidates(launch.process_handle);
        }
        if self
            .handles
            .mark_thread_exited(launch.thread_handle, exit_code)
        {
            self.queue_object_wake_candidates(launch.thread_handle);
        }
        self.record_process_trace(ProcessTraceRecord {
            op: "CreateProcessExited",
            application: launch.application.clone(),
            command_line: launch.command_line.clone(),
            path: None,
            process_handle: Some(launch.process_handle),
            thread_handle: Some(launch.thread_handle),
            process_id: Some(launch.process_id),
            thread_id: Some(launch.thread_id),
            result: Some(exit_code),
            error: None,
        });
    }

    fn destroy_process_windows(&mut self, process_id: u32, thread_id: u32) {
        let hwnds = self
            .gwe
            .windows_snapshot()
            .into_iter()
            .filter(|window| {
                !window.destroyed
                    && window.hwnd != crate::ce::gwe::DESKTOP_HWND
                    && (window.process_id == process_id || window.thread_id == thread_id)
            })
            .map(|window| window.hwnd)
            .collect::<Vec<_>>();
        for hwnd in &hwnds {
            let _ = self.gwe.destroy_window(*hwnd, self.timers.tick_count());
        }
        if !hwnds.is_empty() {
            self.timers.remove_window_timers(&hwnds);
        }
    }

    pub fn pump_timers_to_gwe(&mut self, thread_id: u32) {
        self.expire_timed_out_send_messages();
        for timer in self.timers.due_timers() {
            let target_thread_id = if timer.thread_id == 0 {
                thread_id
            } else {
                timer.thread_id
            };
            let message = crate::ce::gwe::Message::new(
                timer.hwnd.unwrap_or(0),
                timer.message,
                timer.id,
                timer.callback.unwrap_or(0),
                self.timers.tick_count(),
            );
            self.post_gwe_message(target_thread_id, message);
        }
    }

    pub fn expire_timed_out_send_messages(&mut self) -> Vec<u64> {
        let expired = self
            .gwe
            .expire_timed_out_sent_messages(self.timers.tick_count());
        for send_id in &expired {
            self.queue_send_reply_wake_candidates(*send_id);
        }
        expired
    }

    pub fn set_file_root(&mut self, root: impl Into<std::path::PathBuf>) {
        self.files = HostFileSystem::new(root);
    }

    pub fn mount_guest_root(&mut self, guest_root: &str, host_root: impl Into<std::path::PathBuf>) {
        self.files.mount_guest_root(guest_root, host_root);
    }

    pub fn host_path_to_guest_mount(&self, host_path: &std::path::Path) -> Option<String> {
        self.files.host_path_to_guest_mount(host_path)
    }

    pub fn host_path_for_guest(&self, guest_path: &str) -> Result<std::path::PathBuf> {
        self.files.host_path_for_guest(guest_path)
    }

    pub fn recent_file_ops(&self) -> &[FileTraceRecord] {
        &self.recent_file_ops
    }

    pub fn recent_file_open_ops(&self) -> &[FileTraceRecord] {
        &self.recent_file_open_ops
    }

    pub fn recent_process_ops(&self) -> &[ProcessTraceRecord] {
        &self.recent_process_ops
    }

    pub fn recent_event_ops(&self) -> &[EventTraceRecord] {
        &self.recent_event_ops
    }

    pub fn recent_message_ops(&self) -> &[MessageTraceRecord] {
        &self.recent_message_ops
    }

    pub fn file_io_stats(&self) -> FileIoStats {
        self.files.io_stats()
    }

    pub fn scheduler_stats(&self) -> SchedulerStats {
        self.scheduler.stats()
    }

    pub fn gwe_stats(&self) -> GweStats {
        self.gwe.stats()
    }

    pub fn record_file_trace(&mut self, record: FileTraceRecord) {
        self.push_file_trace(record);
    }

    pub fn record_process_trace(&mut self, record: ProcessTraceRecord) {
        if self.recent_process_ops.len() == PROCESS_TRACE_LIMIT {
            self.recent_process_ops.remove(0);
        }
        self.recent_process_ops.push(record);
    }

    fn record_event_trace(&mut self, record: EventTraceRecord) {
        if self.recent_event_ops.len() == EVENT_TRACE_LIMIT {
            self.recent_event_ops.remove(0);
        }
        self.recent_event_ops.push(record);
    }

    fn record_message_trace(&mut self, record: MessageTraceRecord) {
        if self.recent_message_ops.len() == MESSAGE_TRACE_LIMIT {
            self.recent_message_ops.remove(0);
        }
        self.recent_message_ops.push(record);
    }

    fn record_message_op(
        &mut self,
        op: &'static str,
        thread_id: u32,
        message: &Message,
        result: Option<u32>,
        detail: Option<String>,
    ) {
        let source = if message.source == crate::ce::gwe::MSGSRC_UNKNOWN {
            crate::ce::gwe::MSGSRC_SOFTWARE_POST
        } else {
            message.source
        };
        let screen_pos = message.mouse_pos_at_post.or_else(|| {
            matches!(
                message.msg,
                crate::ce::gwe::WM_MOUSEMOVE
                    | crate::ce::gwe::WM_LBUTTONDOWN
                    | crate::ce::gwe::WM_LBUTTONUP
            )
            .then_some(message.lparam)
        });
        self.record_message_trace(MessageTraceRecord {
            op,
            thread_id,
            hwnd: Some(message.hwnd),
            msg: Some(message.msg),
            wparam: Some(message.wparam),
            lparam: Some(message.lparam),
            screen_pos,
            source: Some(source),
            result,
            detail,
        });
    }

    fn push_file_trace(&mut self, record: FileTraceRecord) {
        if is_file_open_trace(record.op) {
            if self.recent_file_open_ops.len() == FILE_TRACE_LIMIT {
                self.recent_file_open_ops.remove(0);
            }
            self.recent_file_open_ops.push(record.clone());
        }
        if self.recent_file_ops.len() == FILE_TRACE_LIMIT {
            self.recent_file_ops.remove(0);
        }
        self.recent_file_ops.push(record);
    }

    pub fn path_for_handle(&self, handle: u32) -> Option<String> {
        match self.handles.get(handle).ok()? {
            KernelObject::File(file) => Some(file.guest_path.clone()),
            KernelObject::Device(device) => Some(device.guest_name.clone()),
            _ => None,
        }
    }

    pub fn is_serial_device_handle(&self, handle: u32) -> bool {
        matches!(
            self.handles.get(handle),
            Ok(KernelObject::Device(device)) if device.is_serial()
        )
    }

    pub fn serial_read_ready(&self, handle: u32) -> bool {
        let Ok(KernelObject::Device(device)) = self.handles.get(handle) else {
            return false;
        };
        if !device.is_serial() {
            return false;
        }
        device.rx_len() != 0
            || (device.accepts_remote_serial_target(&self.remote_gps_target())
                && self.remote.serial_byte_count() != 0)
    }

    pub fn serial_empty_read_timeout_ms(&self, handle: u32, requested: u32) -> Option<u32> {
        match self.handles.get(handle) {
            Ok(KernelObject::Device(device)) => device.empty_read_timeout_ms(requested),
            _ => Some(0),
        }
    }

    pub fn get_comm_timeouts(&self, handle: u32) -> Result<crate::ce::devices::CommTimeouts> {
        match self.handles.get(handle)? {
            KernelObject::Device(device) if device.is_serial() => Ok(device.comm_timeouts()),
            _ => Err(Error::InvalidHandle(handle)),
        }
    }

    pub fn get_comm_dcb(&self, handle: u32) -> Result<crate::ce::devices::CommDcb> {
        match self.handles.get(handle)? {
            KernelObject::Device(device) if device.is_serial() => Ok(device.dcb()),
            _ => Err(Error::InvalidHandle(handle)),
        }
    }

    pub fn set_comm_dcb(&mut self, handle: u32, dcb: crate::ce::devices::CommDcb) -> Result<()> {
        match self.handles.get_mut(handle)? {
            KernelObject::Device(device) if device.is_serial() => {
                device.set_dcb(dcb);
                Ok(())
            }
            _ => Err(Error::InvalidHandle(handle)),
        }
    }

    pub fn set_comm_timeouts(
        &mut self,
        handle: u32,
        timeouts: crate::ce::devices::CommTimeouts,
    ) -> Result<()> {
        match self.handles.get_mut(handle)? {
            KernelObject::Device(device) if device.is_serial() => {
                device.set_comm_timeouts(timeouts);
                Ok(())
            }
            _ => Err(Error::InvalidHandle(handle)),
        }
    }

    pub fn get_comm_mask(&self, handle: u32) -> Result<u32> {
        match self.handles.get(handle)? {
            KernelObject::Device(device) if device.is_serial() => Ok(device.comm_mask()),
            _ => Err(Error::InvalidHandle(handle)),
        }
    }

    pub fn set_comm_mask(&mut self, handle: u32, mask: u32) -> Result<()> {
        let wait_ids = self.scheduler.serial_event_waiter_ids_for_handle(handle);
        match self.handles.get_mut(handle)? {
            KernelObject::Device(device) if device.is_serial() => {
                device.set_comm_mask(mask);
                if !wait_ids.is_empty() {
                    self.comm_event_mask_changed_waits
                        .extend(wait_ids.iter().copied());
                    self.scheduler.queue_serial_event_wake_candidates(handle);
                }
                Ok(())
            }
            _ => Err(Error::InvalidHandle(handle)),
        }
    }

    pub fn serial_comm_event_ready(&self, handle: u32) -> bool {
        const EV_RXCHAR: u32 = 0x0001;
        self.get_comm_mask(handle)
            .is_ok_and(|mask| mask & EV_RXCHAR != 0)
            && self.serial_read_ready(handle)
    }

    pub fn serial_comm_event_value(&self, handle: u32) -> u32 {
        const EV_RXCHAR: u32 = 0x0001;
        if self.serial_comm_event_ready(handle) {
            EV_RXCHAR
        } else {
            0
        }
    }

    pub fn purge_comm(&mut self, handle: u32, flags: u32) -> Result<()> {
        let target = self.remote_gps_target();
        let clear_remote_rx = flags & PURGE_RXCLEAR != 0
            && matches!(
                self.handles.get(handle),
                Ok(KernelObject::Device(device))
                    if device.is_serial() && device.accepts_remote_serial_target(&target)
            );
        match self.handles.get_mut(handle)? {
            KernelObject::Device(device) if device.is_serial() => {
                device.purge_comm(flags);
                if clear_remote_rx {
                    let _ = self.remote.read_serial_bytes(usize::MAX);
                }
                Ok(())
            }
            _ => Err(Error::InvalidHandle(handle)),
        }
    }

    pub fn comm_queue_lengths(&self, handle: u32) -> Result<(u32, u32)> {
        let target = self.remote_gps_target();
        match self.handles.get(handle)? {
            KernelObject::Device(device) if device.is_serial() => {
                let (mut rx_len, tx_len) = device.queue_lengths();
                if device.accepts_remote_serial_target(&target) {
                    rx_len = rx_len.saturating_add(self.remote.serial_byte_count() as u32);
                }
                Ok((rx_len, tx_len))
            }
            _ => Err(Error::InvalidHandle(handle)),
        }
    }

    fn drain_remote_serial_to_handle(&mut self, handle: u32, max_bytes: usize) -> usize {
        if max_bytes == 0 {
            return 0;
        }
        let target = self.remote_gps_target();
        let should_drain = matches!(
            self.handles.get(handle),
            Ok(KernelObject::Device(device))
                if device.is_serial() && device.accepts_remote_serial_target(&target)
        );
        if !should_drain || self.remote.serial_byte_count() == 0 {
            return 0;
        }
        let bytes = self.remote.read_serial_bytes(max_bytes);
        let count = bytes.len();
        if count == 0 {
            return 0;
        }
        if let Ok(KernelObject::Device(device)) = self.handles.get_mut(handle) {
            device.enqueue_rx(&bytes);
        }
        count
    }

    pub fn poll_host_serial_to_handle(&mut self, handle: u32, max_bytes: usize) -> usize {
        if max_bytes == 0 {
            return 0;
        }
        match self.handles.get_mut(handle) {
            Ok(KernelObject::Device(device)) if device.is_serial() => {
                device.poll_host_serial(max_bytes)
            }
            _ => 0,
        }
    }

    pub fn create_file_w(
        &mut self,
        path: &str,
        desired_access: u32,
        creation_disposition: u32,
    ) -> Result<u32> {
        if let Ok(session) = self.devices.open(path) {
            let handle = self.handles.insert(KernelObject::Device(session));
            self.push_file_trace(FileTraceRecord {
                op: "CreateFileW",
                handle: Some(handle),
                path: Some(path.to_owned()),
                preview: None,
                requested: Some(desired_access),
                transferred: None,
                position: Some(u64::from(creation_disposition)),
                result: Some(handle),
                error: None,
            });
            return Ok(handle);
        }

        let file_id = match self
            .files
            .create_file_w(path, desired_access, creation_disposition)
        {
            Ok(file_id) => file_id,
            Err(err) => {
                self.push_file_trace(FileTraceRecord {
                    op: "CreateFileW",
                    handle: None,
                    path: Some(path.to_owned()),
                    preview: None,
                    requested: Some(desired_access),
                    transferred: None,
                    position: Some(u64::from(creation_disposition)),
                    result: Some(u32::MAX),
                    error: Some(err.to_string()),
                });
                return Err(err);
            }
        };
        let handle = self.handles.insert(KernelObject::File(FileObject {
            guest_path: path.to_owned(),
            file_id,
        }));
        self.push_file_trace(FileTraceRecord {
            op: "CreateFileW",
            handle: Some(handle),
            path: Some(path.to_owned()),
            preview: None,
            requested: Some(desired_access),
            transferred: None,
            position: Some(u64::from(creation_disposition)),
            result: Some(handle),
            error: None,
        });
        Ok(handle)
    }

    pub fn open_existing_readonly(&mut self, path: &str) -> Result<u32> {
        self.create_file_w(path, GENERIC_READ, OPEN_EXISTING)
    }

    pub fn open_existing_readwrite(&mut self, path: &str) -> Result<u32> {
        self.create_file_w(path, GENERIC_READ | GENERIC_WRITE, OPEN_EXISTING)
    }

    pub fn read_file(&mut self, handle: u32, requested: u32) -> Result<Vec<u8>> {
        self.poll_host_serial_to_handle(handle, requested as usize);
        self.drain_remote_serial_to_handle(handle, requested as usize);
        let path = self.path_for_handle(handle);
        let start_position = match self.handles.get(handle) {
            Ok(KernelObject::File(file)) => self
                .files
                .open_file(file.file_id)
                .ok()
                .map(|file| file.cursor() as u64),
            _ => None,
        };
        let result = match self.handles.get_mut(handle) {
            Ok(object) => match object {
                KernelObject::File(file) => self.files.read_file(file.file_id, requested),
                KernelObject::Device(device) => Ok(device.read_file(requested)),
                _ => Ok(Vec::new()),
            },
            Err(err) => Err(err),
        };
        let end_position = match self.handles.get(handle) {
            Ok(KernelObject::File(file)) => self
                .files
                .open_file(file.file_id)
                .ok()
                .map(|file| file.cursor() as u64),
            _ => None,
        };
        self.push_file_trace(FileTraceRecord {
            op: "ReadFile",
            handle: Some(handle),
            path,
            preview: file_read_trace_preview(
                start_position,
                end_position,
                result.as_deref().unwrap_or(&[]),
            ),
            requested: Some(requested),
            transferred: result.as_ref().ok().map(|bytes| bytes.len() as u32),
            position: start_position,
            result: result.as_ref().ok().map(|_| 1),
            error: result.as_ref().err().map(ToString::to_string),
        });
        result
    }

    pub fn read_file_into<F>(&mut self, handle: u32, requested: u32, mut write: F) -> Result<u32>
    where
        F: FnMut(&[u8]) -> Result<()>,
    {
        self.poll_host_serial_to_handle(handle, requested as usize);
        self.drain_remote_serial_to_handle(handle, requested as usize);
        let path = self.path_for_handle(handle);
        let start_position = match self.handles.get(handle) {
            Ok(KernelObject::File(file)) => self
                .files
                .open_file(file.file_id)
                .ok()
                .map(|file| file.cursor() as u64),
            _ => None,
        };
        let mut preview_bytes = Vec::new();
        let result = match self.handles.get_mut(handle) {
            Ok(object) => match object {
                KernelObject::File(file) => {
                    self.files.read_file_into(file.file_id, requested, |bytes| {
                        if preview_bytes.len() < FILE_TRACE_PREVIEW_LIMIT {
                            let remaining = FILE_TRACE_PREVIEW_LIMIT - preview_bytes.len();
                            preview_bytes.extend_from_slice(&bytes[..bytes.len().min(remaining)]);
                        }
                        write(bytes)
                    })
                }
                KernelObject::Device(device) => {
                    let bytes = device.read_file(requested);
                    preview_bytes
                        .extend_from_slice(&bytes[..bytes.len().min(FILE_TRACE_PREVIEW_LIMIT)]);
                    let transferred = bytes.len() as u32;
                    write(&bytes).map(|_| transferred)
                }
                _ => write(&[]).map(|_| 0),
            },
            Err(err) => Err(err),
        };
        let end_position = match self.handles.get(handle) {
            Ok(KernelObject::File(file)) => self
                .files
                .open_file(file.file_id)
                .ok()
                .map(|file| file.cursor() as u64),
            _ => None,
        };
        self.push_file_trace(FileTraceRecord {
            op: "ReadFile",
            handle: Some(handle),
            path,
            preview: file_read_trace_preview(start_position, end_position, &preview_bytes),
            requested: Some(requested),
            transferred: result.as_ref().ok().copied(),
            position: start_position,
            result: result.as_ref().ok().map(|_| 1),
            error: result.as_ref().err().map(ToString::to_string),
        });
        result
    }

    pub fn read_file_at(
        &mut self,
        file_id: u32,
        offset: usize,
        requested: usize,
    ) -> Result<Vec<u8>> {
        self.files.read_at(file_id, offset, requested)
    }

    pub fn read_guest_file(&self, path: &str) -> Result<Vec<u8>> {
        self.files.read_guest_file(path)
    }

    pub fn file_attributes_w(&self, path: &str) -> Result<FindData> {
        self.files.file_attributes_w(path)
    }

    pub fn create_directory_w(&self, path: &str) -> Result<()> {
        self.files.create_directory_w(path)
    }

    pub fn remove_directory_w(&self, path: &str) -> Result<()> {
        self.files.remove_directory_w(path)
    }

    pub fn delete_file_w(&self, path: &str) -> Result<()> {
        self.files.delete_file_w(path)
    }

    pub fn move_file_w(&self, existing_path: &str, new_path: &str) -> Result<()> {
        self.files.move_file_w(existing_path, new_path)
    }

    pub fn copy_file_w(
        &self,
        existing_path: &str,
        new_path: &str,
        fail_if_exists: bool,
    ) -> Result<()> {
        self.files
            .copy_file_w(existing_path, new_path, fail_if_exists)
    }

    pub fn set_file_attributes_w(&self, path: &str, attributes: u32) -> Result<()> {
        self.files.set_file_attributes_w(path, attributes)
    }

    pub fn write_file(&mut self, handle: u32, bytes: &[u8]) -> Result<FileIoResult> {
        let path = self.path_for_handle(handle);
        let result = match self.handles.get_mut(handle) {
            Ok(object) => match object {
                KernelObject::File(file) => self.files.write_file(file.file_id, bytes),
                KernelObject::Device(device) => Ok(FileIoResult {
                    success: true,
                    bytes_transferred: device.write_file(bytes),
                }),
                _ => Ok(FileIoResult {
                    success: false,
                    bytes_transferred: 0,
                }),
            },
            Err(err) => Err(err),
        };
        self.push_file_trace(FileTraceRecord {
            op: "WriteFile",
            handle: Some(handle),
            path,
            preview: file_trace_preview(bytes),
            requested: Some(bytes.len() as u32),
            transferred: result.as_ref().ok().map(|io| io.bytes_transferred),
            position: None,
            result: result.as_ref().ok().map(|io| u32::from(io.success)),
            error: result.as_ref().err().map(ToString::to_string),
        });
        result
    }

    pub fn write_file_at(
        &mut self,
        file_id: u32,
        offset: usize,
        bytes: &[u8],
    ) -> Result<FileIoResult> {
        self.files.write_at(file_id, offset, bytes)
    }

    pub fn set_file_pointer(
        &mut self,
        handle: u32,
        distance: i64,
        move_method: u32,
    ) -> Result<usize> {
        let path = self.path_for_handle(handle);
        let KernelObject::File(file) = self.handles.get(handle)? else {
            self.push_file_trace(FileTraceRecord {
                op: "SetFilePointer",
                handle: Some(handle),
                path,
                preview: None,
                requested: Some(move_method),
                transferred: None,
                position: None,
                result: Some(u32::MAX),
                error: Some("invalid handle".to_owned()),
            });
            return Err(crate::error::Error::InvalidHandle(handle));
        };
        let file_id = file.file_id;
        let current = self.files.open_file(file_id)?.cursor() as i64;
        let size = self.files.file_size(file_id)? as i64;
        let position = match move_method {
            0 => distance,
            1 => current.saturating_add(distance),
            2 => size.saturating_add(distance),
            _ => {
                return Err(crate::error::Error::InvalidArgument(
                    "bad move method".to_owned(),
                ));
            }
        };
        if position < 0 {
            self.push_file_trace(FileTraceRecord {
                op: "SetFilePointer",
                handle: Some(handle),
                path,
                preview: None,
                requested: Some(move_method),
                transferred: None,
                position: None,
                result: Some(u32::MAX),
                error: Some("negative file pointer".to_owned()),
            });
            return Err(crate::error::Error::InvalidArgument(
                "negative file pointer".to_owned(),
            ));
        }
        let result = self.files.set_file_pointer(file_id, position as usize);
        self.push_file_trace(FileTraceRecord {
            op: "SetFilePointer",
            handle: Some(handle),
            path,
            preview: Some(format!("distance={distance} method={move_method}")),
            requested: Some(move_method),
            transferred: None,
            position: result.as_ref().ok().map(|position| *position as u64),
            result: result.as_ref().ok().map(|position| *position as u32),
            error: result.as_ref().err().map(ToString::to_string),
        });
        result
    }

    pub fn get_file_size(&self, handle: u32) -> Result<usize> {
        let KernelObject::File(file) = self.handles.get(handle)? else {
            return Err(crate::error::Error::InvalidHandle(handle));
        };
        self.files.file_size(file.file_id)
    }

    pub fn file_position(&self, handle: u32) -> Result<usize> {
        let KernelObject::File(file) = self.handles.get(handle)? else {
            return Err(crate::error::Error::InvalidHandle(handle));
        };
        Ok(self.files.open_file(file.file_id)?.cursor())
    }

    pub fn file_is_eof(&self, handle: u32) -> Result<bool> {
        let KernelObject::File(file) = self.handles.get(handle)? else {
            return Err(crate::error::Error::InvalidHandle(handle));
        };
        self.files.file_is_eof(file.file_id)
    }

    pub fn flush_file_buffers(&mut self, handle: u32) -> Result<bool> {
        let KernelObject::File(file) = self.handles.get(handle)? else {
            return Err(crate::error::Error::InvalidHandle(handle));
        };
        self.files.flush(file.file_id)?;
        Ok(true)
    }

    pub fn find_first_file_w(&mut self, pattern: &str) -> Result<(u32, FindData)> {
        let (find_id, data) = match self.files.find_first_file_w(pattern) {
            Ok(result) => result,
            Err(err) => {
                self.push_file_trace(FileTraceRecord {
                    op: "FindFirstFileW",
                    handle: None,
                    path: Some(pattern.to_owned()),
                    preview: None,
                    requested: None,
                    transferred: None,
                    position: None,
                    result: Some(u32::MAX),
                    error: Some(err.to_string()),
                });
                return Err(err);
            }
        };
        let handle = self.handles.insert(KernelObject::FindFile(FindFileObject {
            guest_pattern: pattern.to_owned(),
            find_id,
        }));
        self.push_file_trace(FileTraceRecord {
            op: "FindFirstFileW",
            handle: Some(handle),
            path: Some(pattern.to_owned()),
            preview: Some(format!("{} attr=0x{:08x}", data.file_name, data.attributes)),
            requested: None,
            transferred: Some(data.file_size as u32),
            position: None,
            result: Some(handle),
            error: None,
        });
        Ok((handle, data))
    }

    pub fn find_next_file_w(&mut self, handle: u32) -> Result<Option<FindData>> {
        let KernelObject::FindFile(find) = self.handles.get(handle)?.clone() else {
            return Err(crate::error::Error::InvalidHandle(handle));
        };
        let guest_pattern = find.guest_pattern.clone();
        let result = self.files.find_next_file_w(find.find_id);
        match &result {
            Ok(Some(data)) => self.push_file_trace(FileTraceRecord {
                op: "FindNextFileW",
                handle: Some(handle),
                path: Some(guest_pattern.clone()),
                preview: Some(format!("{} attr=0x{:08x}", data.file_name, data.attributes)),
                requested: None,
                transferred: Some(data.file_size as u32),
                position: None,
                result: Some(1),
                error: None,
            }),
            Ok(None) => self.push_file_trace(FileTraceRecord {
                op: "FindNextFileW",
                handle: Some(handle),
                path: Some(guest_pattern.clone()),
                preview: None,
                requested: None,
                transferred: None,
                position: None,
                result: Some(0),
                error: None,
            }),
            Err(err) => self.push_file_trace(FileTraceRecord {
                op: "FindNextFileW",
                handle: Some(handle),
                path: Some(guest_pattern),
                preview: None,
                requested: None,
                transferred: None,
                position: None,
                result: Some(0),
                error: Some(err.to_string()),
            }),
        }
        result
    }

    pub fn find_close(&mut self, handle: u32) -> Result<bool> {
        let KernelObject::FindFile(find) = self.handles.get(handle)?.clone() else {
            return Err(crate::error::Error::InvalidHandle(handle));
        };
        self.files.find_close(find.find_id)?;
        self.handles.close(handle)?;
        Ok(true)
    }

    pub fn device_io_control(
        &mut self,
        handle: u32,
        ioctl_code: u32,
        input: &[u8],
        output_capacity: u32,
    ) -> Result<DeviceIoControlResult> {
        match self.handles.get_mut(handle)? {
            KernelObject::Device(device) => {
                Ok(device.device_io_control(ioctl_code, input, output_capacity))
            }
            _ => Ok(DeviceIoControlResult {
                success: false,
                bytes_returned: 0,
                output: Vec::new(),
            }),
        }
    }

    pub fn close_handle(&mut self, handle: u32) -> Result<bool> {
        let object = self.handles.get(handle)?.clone();
        match object {
            KernelObject::File(file) => self.files.close(file.file_id)?,
            KernelObject::FindFile(find) => self.files.find_close(find.find_id)?,
            KernelObject::Event(event) if event.name.is_some() => return Ok(true),
            KernelObject::FileMapping(mapping) if mapping.name.is_some() => return Ok(true),
            _ => {}
        }
        self.handles.close(handle)?;
        Ok(true)
    }

    pub fn create_event_w(
        &mut self,
        name: Option<String>,
        manual_reset: bool,
        initial_state: bool,
    ) -> u32 {
        let handle = self
            .handles
            .create_event(name.clone(), manual_reset, initial_state);
        self.record_event_trace(EventTraceRecord {
            op: "CreateEventW",
            handle: Some(handle),
            name,
            manual_reset: Some(manual_reset),
            signaled: Some(initial_state),
            result: Some(handle != 0),
            detail: Some(self.describe_handle(handle)),
        });
        handle
    }

    pub fn open_event_w(&mut self, name: &str) -> Option<u32> {
        let handle = self.handles.open_event(name);
        self.record_event_trace(EventTraceRecord {
            op: "OpenEventW",
            handle,
            name: Some(name.to_owned()),
            manual_reset: None,
            signaled: None,
            result: Some(handle.is_some()),
            detail: handle.map(|handle| self.describe_handle(handle)),
        });
        handle
    }

    pub fn create_guest_thread(
        &mut self,
        start_address: u32,
        parameter: u32,
        suspended: bool,
    ) -> (u32, u32) {
        let thread_id = self.threads.allocate_guest_thread_id();
        let handle = self
            .handles
            .create_thread(thread_id, start_address, parameter, suspended);
        (handle, thread_id)
    }

    pub fn mark_guest_thread_exited(&mut self, handle: u32, exit_code: u32) -> bool {
        let success = self.handles.mark_thread_exited(handle, exit_code);
        if success {
            self.queue_object_wake_candidates(handle);
        }
        success
    }

    pub fn guest_thread_id(&self, handle: u32) -> Option<u32> {
        self.handles.thread_id(handle)
    }

    pub fn guest_thread_handle_by_id(&self, thread_id: u32) -> Option<u32> {
        self.handles.thread_handle_by_id(thread_id)
    }

    pub fn guest_thread_id_for_handle(&self, handle: u32, current_thread_id: u32) -> Option<u32> {
        if Self::is_current_thread_pseudo_handle(handle) {
            return Some(current_thread_id);
        }
        self.handles.thread_id(handle)
    }

    pub fn guest_thread_exit_code(&self, handle: u32) -> Option<u32> {
        self.handles.thread_exit_code(handle)
    }

    pub fn guest_thread_exit_code_for_handle(
        &self,
        handle: u32,
        _current_thread_id: u32,
    ) -> Option<u32> {
        if Self::is_current_thread_pseudo_handle(handle) {
            return Some(STILL_ACTIVE);
        }
        self.handles.thread_exit_code(handle)
    }

    pub fn process_exit_code(&self, handle: u32) -> Option<u32> {
        self.handles.process_exit_code(handle)
    }

    pub fn process_exit_code_for_handle(&self, handle: u32) -> Option<u32> {
        if Self::is_current_process_pseudo_handle(handle) {
            return Some(self.current_process_exit_code);
        }
        self.handles.process_exit_code(handle)
    }

    pub fn process_id(&self, handle: u32) -> Option<u32> {
        self.handles.process_id(handle)
    }

    pub fn process_id_for_handle(&self, handle: u32) -> Option<u32> {
        if Self::is_current_process_pseudo_handle(handle) {
            return Some(self.current_process_id);
        }
        self.handles.process_id(handle)
    }

    pub fn terminate_process(&mut self, handle: u32, exit_code: u32) -> bool {
        if Self::is_current_process_pseudo_handle(handle) {
            self.current_process_exit_code = exit_code;
            self.current_process_signaled = true;
            self.queue_object_wake_candidates(handle);
            return true;
        }
        let success = self.handles.mark_process_exited(handle, exit_code);
        if success {
            self.queue_object_wake_candidates(handle);
        }
        success
    }

    pub fn suspend_thread(&mut self, handle: u32) -> ThreadSuspendResult {
        self.handles.suspend_thread(handle)
    }

    pub fn suspend_thread_for_handle(
        &mut self,
        handle: u32,
        current_thread_id: u32,
    ) -> ThreadSuspendResult {
        if Self::is_current_thread_pseudo_handle(handle) {
            if let Some(result) = self.handles.suspend_thread_by_id(current_thread_id) {
                return result;
            }
            return self.suspend_thread_by_id_fallback(current_thread_id);
        }
        self.suspend_thread(handle)
    }

    fn suspend_thread_by_id_fallback(&mut self, thread_id: u32) -> ThreadSuspendResult {
        let previous = self
            .thread_suspend_counts
            .get(&thread_id)
            .copied()
            .unwrap_or(0);
        if previous == MAX_SUSPEND_COUNT {
            return ThreadSuspendResult::SignalRefused;
        }
        self.thread_suspend_counts.insert(thread_id, previous + 1);
        ThreadSuspendResult::Previous(previous)
    }

    pub fn resume_thread(&mut self, handle: u32) -> ThreadResumeResult {
        self.handles.resume_thread(handle)
    }

    pub fn resume_thread_for_handle(
        &mut self,
        handle: u32,
        current_thread_id: u32,
    ) -> ThreadResumeResult {
        if Self::is_current_thread_pseudo_handle(handle) {
            if let Some(result) = self.handles.resume_thread_by_id(current_thread_id) {
                return result;
            }
            return self.resume_thread_by_id_fallback(current_thread_id);
        }
        self.resume_thread(handle)
    }

    fn resume_thread_by_id_fallback(&mut self, thread_id: u32) -> ThreadResumeResult {
        let previous = self
            .thread_suspend_counts
            .get(&thread_id)
            .copied()
            .unwrap_or(0);
        if previous > 1 {
            self.thread_suspend_counts.insert(thread_id, previous - 1);
        } else {
            self.thread_suspend_counts.remove(&thread_id);
        }
        ThreadResumeResult::Previous(previous)
    }

    pub fn thread_priority(&self, handle: u32) -> Option<i32> {
        self.handles.thread_priority(handle)
    }

    pub fn thread_priority_for_handle(&self, handle: u32, current_thread_id: u32) -> Option<i32> {
        if Self::is_current_thread_pseudo_handle(handle) {
            return Some(self.thread_priority_by_id(current_thread_id));
        }
        self.handles.thread_priority(handle)
    }

    pub fn thread_win32_priority(&self, handle: u32) -> Option<u32> {
        self.handles
            .thread_priority(handle)
            .and_then(ce_thread_priority_to_win32)
    }

    pub fn thread_win32_priority_for_handle(
        &self,
        handle: u32,
        current_thread_id: u32,
    ) -> Option<u32> {
        self.thread_priority_for_handle(handle, current_thread_id)
            .and_then(ce_thread_priority_to_win32)
    }

    pub fn thread_priority_by_id(&self, thread_id: u32) -> i32 {
        if let Some(priority) = self.thread_priority_overrides.get(&thread_id) {
            return *priority;
        }
        self.handles
            .thread_priority_by_id(thread_id)
            .unwrap_or(CE_THREAD_PRIORITY_NORMAL)
    }

    pub fn set_thread_ce_priority(&mut self, handle: u32, priority: i32) -> bool {
        self.handles.set_thread_priority(handle, priority)
    }

    pub fn set_thread_ce_priority_for_handle(
        &mut self,
        handle: u32,
        priority: i32,
        current_thread_id: u32,
    ) -> bool {
        if Self::is_current_thread_pseudo_handle(handle) {
            return self.set_thread_ce_priority_by_id(current_thread_id, priority);
        }
        self.set_thread_ce_priority(handle, priority)
    }

    fn set_thread_ce_priority_by_id(&mut self, thread_id: u32, priority: i32) -> bool {
        if let Some(success) = self.handles.set_thread_priority_by_id(thread_id, priority) {
            if success {
                self.thread_priority_overrides.remove(&thread_id);
            }
            return success;
        }
        if !(0..crate::ce::object::MAX_CE_PRIORITY_LEVELS).contains(&priority) {
            return false;
        }
        self.thread_priority_overrides.insert(thread_id, priority);
        true
    }

    pub fn set_thread_win32_priority(&mut self, handle: u32, priority: u32) -> bool {
        let Some(priority) = win32_thread_priority_to_ce(priority) else {
            return false;
        };
        self.handles.set_thread_priority(handle, priority)
    }

    pub fn set_thread_win32_priority_for_handle(
        &mut self,
        handle: u32,
        priority: u32,
        current_thread_id: u32,
    ) -> bool {
        let Some(priority) = win32_thread_priority_to_ce(priority) else {
            return false;
        };
        self.set_thread_ce_priority_for_handle(handle, priority, current_thread_id)
    }

    pub fn guest_thread_start(&self, handle: u32) -> Option<(u32, u32, u32)> {
        self.handles.thread_start(handle)
    }

    pub fn set_event(&mut self, handle: u32) -> bool {
        let success = self.handles.set_event(handle);
        if success {
            self.queue_object_wake_candidates(handle);
        }
        self.record_event_trace(EventTraceRecord {
            op: "SetEvent",
            handle: Some(handle),
            name: None,
            manual_reset: None,
            signaled: None,
            result: Some(success),
            detail: Some(self.describe_handle(handle)),
        });
        success
    }

    pub fn reset_event(&mut self, handle: u32) -> bool {
        let success = self.handles.reset_event(handle);
        self.record_event_trace(EventTraceRecord {
            op: "ResetEvent",
            handle: Some(handle),
            name: None,
            manual_reset: None,
            signaled: None,
            result: Some(success),
            detail: Some(self.describe_handle(handle)),
        });
        success
    }

    pub fn pulse_event(&mut self, handle: u32) -> bool {
        let (manual_reset, wait_ids) = match self.handles.get(handle) {
            Ok(KernelObject::Event(event)) => {
                let mut wait_ids = self.scheduler.waiter_ids_for_handle(handle);
                if !event.manual_reset {
                    wait_ids.truncate(1);
                }
                (event.manual_reset, wait_ids)
            }
            _ => {
                self.record_event_trace(EventTraceRecord {
                    op: "PulseEvent",
                    handle: Some(handle),
                    name: None,
                    manual_reset: None,
                    signaled: None,
                    result: Some(false),
                    detail: Some(self.describe_handle(handle)),
                });
                return false;
            }
        };

        let success = self.handles.set_event(handle);
        if success {
            self.scheduler
                .queue_pending_wake_ids(wait_ids.iter().copied());
            for wait_id in wait_ids {
                self.pulsed_wait_handles.insert(wait_id, handle);
            }
            let _ = self.handles.reset_event(handle);
        }
        self.record_event_trace(EventTraceRecord {
            op: "PulseEvent",
            handle: Some(handle),
            name: None,
            manual_reset: Some(manual_reset),
            signaled: Some(false),
            result: Some(success),
            detail: Some(self.describe_handle(handle)),
        });
        success
    }

    pub fn create_mutex_w(
        &mut self,
        name: Option<String>,
        initial_owner_thread: Option<u32>,
    ) -> u32 {
        self.handles.create_mutex(name, initial_owner_thread)
    }

    pub fn create_mutex_w_with_status(
        &mut self,
        name: Option<String>,
        initial_owner_thread: Option<u32>,
    ) -> (u32, bool) {
        self.handles
            .create_mutex_with_status(name, initial_owner_thread)
    }

    pub fn create_semaphore_w(
        &mut self,
        name: Option<String>,
        initial_count: i32,
        maximum_count: i32,
    ) -> Option<u32> {
        self.handles
            .create_semaphore(name, initial_count, maximum_count)
    }

    pub fn release_semaphore(&mut self, handle: u32, release_count: i32) -> Option<i32> {
        let previous = self.handles.release_semaphore(handle, release_count)?;
        self.queue_object_wake_candidates(handle);
        Some(previous)
    }

    pub fn release_mutex(&mut self, handle: u32, thread_id: u32) -> bool {
        let previous_lock_count = self.handles.mutex_lock_count(handle);
        let success = self.handles.release_mutex(handle, thread_id);
        if success && previous_lock_count == Some(1) {
            self.queue_object_wake_candidates(handle);
        }
        success
    }

    pub fn is_mutex_handle(&self, handle: u32) -> bool {
        self.handles.is_mutex(handle)
    }

    fn wait_for_single_object_core(
        &mut self,
        handle: u32,
        timeout_ms: u32,
        thread_id: u32,
    ) -> WaitResult {
        if Self::is_current_thread_pseudo_handle(handle) {
            return if timeout_ms == 0 {
                WaitResult::Timeout
            } else {
                WaitResult::Timeout
            };
        }
        if Self::is_current_process_pseudo_handle(handle) {
            return if self.current_process_signaled {
                WaitResult::Object0
            } else {
                WaitResult::Timeout
            };
        }
        self.handles
            .wait_for_single_object(handle, timeout_ms, thread_id)
    }

    fn wait_for_any_object_core(&mut self, handles: &[u32], thread_id: u32) -> WaitMultipleResult {
        if handles
            .iter()
            .any(|handle| self.is_wait_ready(*handle, thread_id).is_none())
        {
            return WaitMultipleResult::Failed;
        }

        let Some((index, handle)) = handles
            .iter()
            .enumerate()
            .find(|(_, handle)| self.is_wait_ready(**handle, thread_id) == Some(true))
        else {
            return WaitMultipleResult::Timeout;
        };

        match self.wait_for_single_object_core(*handle, 0, thread_id) {
            WaitResult::Object0 => WaitMultipleResult::Object(index as u32),
            WaitResult::Timeout => WaitMultipleResult::Timeout,
            WaitResult::Failed => WaitMultipleResult::Failed,
        }
    }

    pub fn wait_for_single_object(&mut self, handle: u32, timeout_ms: u32, thread_id: u32) -> u32 {
        self.scheduler
            .record_wait_attempt(SchedulerWaitKind::Single, 1, timeout_ms);
        let result = match self.wait_for_single_object_core(handle, timeout_ms, thread_id) {
            WaitResult::Object0 => WAIT_OBJECT_0,
            WaitResult::Timeout => WAIT_TIMEOUT,
            WaitResult::Failed => WAIT_FAILED,
        };
        if result == WAIT_FAILED {
            self.threads
                .set_last_error(thread_id, crate::ce::thread::ERROR_INVALID_HANDLE);
        }
        let wait_trace = match self.handles.get(handle).ok() {
            Some(KernelObject::Process(process)) => Some(ProcessTraceRecord {
                op: "WaitForSingleObjectProcess",
                application: None,
                command_line: None,
                path: None,
                process_handle: Some(handle),
                thread_handle: None,
                process_id: Some(process.process_id),
                thread_id: None,
                result: Some(result),
                error: Some(format!("exit=0x{:08x}", process.exit_code)),
            }),
            Some(KernelObject::Thread(thread)) => Some(ProcessTraceRecord {
                op: "WaitForSingleObjectThread",
                application: None,
                command_line: None,
                path: None,
                process_handle: None,
                thread_handle: Some(handle),
                process_id: None,
                thread_id: Some(thread.thread_id),
                result: Some(result),
                error: Some(format!("exit=0x{:08x}", thread.exit_code)),
            }),
            _ => None,
        };
        if let Some(record) = wait_trace {
            self.record_process_trace(record);
        }
        self.scheduler
            .record_wait_result(wait_result_to_wake_reason(result));
        result
    }

    pub fn record_blocked_single_wait(&mut self, timeout_ms: u32) {
        self.scheduler
            .record_wait_attempt(SchedulerWaitKind::Single, 1, timeout_ms);
        self.scheduler.record_blocked_wait();
    }

    pub fn record_blocked_multiple_wait(&mut self, handle_count: u32, timeout_ms: u32) {
        self.scheduler
            .record_wait_attempt(SchedulerWaitKind::Multiple, handle_count, timeout_ms);
        self.scheduler.record_blocked_wait();
    }

    pub fn record_blocked_msg_wait(&mut self, handle_count: u32, timeout_ms: u32) {
        self.scheduler
            .record_wait_attempt(SchedulerWaitKind::MsgWait, handle_count, timeout_ms);
        self.scheduler.record_blocked_wait();
    }

    pub fn record_blocked_thread_sleep(&mut self, timeout_ms: u32) {
        self.scheduler
            .record_wait_attempt(SchedulerWaitKind::Sleep, 0, timeout_ms);
        self.scheduler.record_blocked_wait();
    }

    pub fn record_thread_yield(&mut self) {
        self.scheduler.record_thread_yield();
    }

    pub fn record_resumed_single_wait(&mut self, result: u32) {
        self.scheduler
            .record_wait_wake(wait_result_to_wake_reason(result));
    }

    pub fn record_resumed_wait(&mut self, result: u32) {
        self.scheduler
            .record_wait_wake(wait_result_to_wake_reason(result));
    }

    pub fn record_msg_wait_result(&mut self, handle_count: u32, timeout_ms: u32, result: u32) {
        self.scheduler
            .record_wait_attempt(SchedulerWaitKind::MsgWait, handle_count, timeout_ms);
        self.scheduler
            .record_wait_result(wait_result_to_wake_reason(result));
    }

    pub fn record_msg_wait_input(&mut self, handle_count: u32, timeout_ms: u32) {
        self.scheduler
            .record_wait_attempt(SchedulerWaitKind::MsgWait, handle_count, timeout_ms);
        self.scheduler
            .record_wait_result(SchedulerWakeReason::MessageInput);
    }

    pub fn record_resumed_msg_wait_input(&mut self) {
        self.scheduler
            .record_wait_wake(SchedulerWakeReason::MessageInput);
    }

    pub fn record_resumed_msg_wait_result(&mut self, result: u32) {
        self.scheduler
            .record_wait_wake(wait_result_to_wake_reason(result));
    }

    pub fn record_resumed_thread_sleep(&mut self) {
        self.scheduler
            .record_wait_wake(SchedulerWakeReason::Timeout);
    }

    pub fn register_blocked_waiter(
        &mut self,
        thread_id: u32,
        thread_handle: u32,
        wait_handles: Vec<u32>,
        kind: SchedulerBlockedWaitKind,
        wait_started_ms: u32,
        timeout_ms: u32,
    ) -> u64 {
        self.scheduler.register_blocked_wait(
            thread_id,
            thread_handle,
            wait_handles,
            kind,
            wait_started_ms,
            timeout_ms,
        )
    }

    pub fn remove_blocked_waiter(&mut self, wait_id: u64) -> Option<SchedulerBlockedWait> {
        self.pulsed_wait_handles.remove(&wait_id);
        self.comm_event_mask_changed_waits.remove(&wait_id);
        self.scheduler.remove_blocked_wait(wait_id)
    }

    pub fn blocked_waiter(&self, wait_id: u64) -> Option<&SchedulerBlockedWait> {
        self.scheduler.blocked_wait(wait_id)
    }

    pub fn blocked_waiters(&self) -> impl Iterator<Item = &SchedulerBlockedWait> {
        self.scheduler.blocked_waits()
    }

    pub fn describe_handle(&self, handle: u32) -> String {
        if Self::is_current_thread_pseudo_handle(handle) {
            "current_thread_pseudo".to_owned()
        } else if Self::is_current_process_pseudo_handle(handle) {
            format!(
                "current_process_pseudo(signaled={},exit=0x{:08x})",
                self.current_process_signaled, self.current_process_exit_code
            )
        } else {
            self.handles.describe_handle(handle)
        }
    }

    pub fn pulsed_wait_handle(&self, wait_id: u64) -> Option<u32> {
        self.pulsed_wait_handles.get(&wait_id).copied()
    }

    pub fn comm_event_mask_changed_wait(&self, wait_id: u64) -> bool {
        self.comm_event_mask_changed_waits.contains(&wait_id)
    }

    pub fn take_comm_event_mask_changed_wait(&mut self, wait_id: u64) -> bool {
        self.comm_event_mask_changed_waits.remove(&wait_id)
    }

    fn queue_object_wake_candidates(&mut self, handle: u32) {
        let wait_ids = self.scheduler.waiter_ids_for_handle(handle);
        self.scheduler.queue_pending_wake_ids(wait_ids);
    }

    pub fn queue_serial_read_wake_candidates(&mut self, handle: u32) -> usize {
        self.scheduler.queue_serial_read_wake_candidates(handle)
    }

    pub fn queue_serial_event_wake_candidates(&mut self, handle: u32) -> usize {
        self.scheduler.queue_serial_event_wake_candidates(handle)
    }

    pub fn queue_all_serial_read_wake_candidates(&mut self) -> usize {
        self.scheduler.queue_all_serial_read_wake_candidates()
    }

    pub fn queue_all_serial_event_wake_candidates(&mut self) -> usize {
        self.scheduler.queue_all_serial_event_wake_candidates()
    }

    pub fn queue_message_wake_candidates(&mut self, thread_id: u32) -> usize {
        self.scheduler.queue_message_wake_candidates(thread_id)
    }

    pub fn queue_send_reply_wake_candidates(&mut self, send_id: u64) -> usize {
        self.scheduler.queue_send_reply_wake_candidates(send_id)
    }

    pub fn sent_message_result_ready(&self, send_id: u64) -> bool {
        self.gwe.sent_message_result_ready(send_id)
    }

    pub fn select_ready_blocked_waiter(
        &self,
        active_thread_id: u32,
        now_ms: u32,
        mut is_ready: impl FnMut(&SchedulerBlockedWait, &Self) -> bool,
    ) -> Option<u64> {
        self.scheduler.select_ready_blocked_wait_id(
            active_thread_id,
            now_ms,
            |wait| is_ready(wait, self),
            |thread_id| self.thread_priority_by_id(thread_id),
        )
    }

    pub fn record_multiple_wait_attempt(
        &mut self,
        handle_count: u32,
        timeout_ms: u32,
        result: u32,
    ) {
        self.scheduler
            .record_wait_attempt(SchedulerWaitKind::Multiple, handle_count, timeout_ms);
        self.scheduler
            .record_wait_result(wait_result_to_wake_reason(result));
    }

    pub fn wait_for_single_object_without_scheduler_record(
        &mut self,
        handle: u32,
        timeout_ms: u32,
        thread_id: u32,
    ) -> u32 {
        match self.wait_for_single_object_core(handle, timeout_ms, thread_id) {
            WaitResult::Object0 => WAIT_OBJECT_0,
            WaitResult::Timeout => WAIT_TIMEOUT,
            WaitResult::Failed => {
                self.threads
                    .set_last_error(thread_id, crate::ce::thread::ERROR_INVALID_HANDLE);
                WAIT_FAILED
            }
        }
    }

    pub fn is_wait_ready(&self, handle: u32, thread_id: u32) -> Option<bool> {
        if Self::is_current_thread_pseudo_handle(handle) {
            return Some(false);
        }
        if Self::is_current_process_pseudo_handle(handle) {
            return Some(self.current_process_signaled);
        }
        self.handles.is_wait_ready(handle, thread_id)
    }

    pub fn wait_for_multiple_objects(
        &mut self,
        handles: &[u32],
        wait_all: bool,
        timeout_ms: u32,
        thread_id: u32,
    ) -> u32 {
        let result = if handles.is_empty() || wait_all || handles.len() > MAXIMUM_WAIT_OBJECTS {
            self.threads
                .set_last_error(thread_id, crate::ce::thread::ERROR_INVALID_PARAMETER);
            WAIT_FAILED
        } else {
            match self.wait_for_any_object_core(handles, thread_id) {
                WaitMultipleResult::Object(index) => WAIT_OBJECT_0 + index,
                WaitMultipleResult::Timeout => WAIT_TIMEOUT,
                WaitMultipleResult::Failed => {
                    self.threads
                        .set_last_error(thread_id, crate::ce::thread::ERROR_INVALID_HANDLE);
                    WAIT_FAILED
                }
            }
        };
        self.record_multiple_wait_attempt(handles.len() as u32, timeout_ms, result);
        result
    }

    pub fn wait_for_multiple_objects_without_scheduler_record(
        &mut self,
        handles: &[u32],
        wait_all: bool,
        thread_id: u32,
    ) -> u32 {
        if handles.is_empty() || wait_all || handles.len() > MAXIMUM_WAIT_OBJECTS {
            self.threads
                .set_last_error(thread_id, crate::ce::thread::ERROR_INVALID_PARAMETER);
            return WAIT_FAILED;
        }
        match self.wait_for_any_object_core(handles, thread_id) {
            WaitMultipleResult::Object(index) => WAIT_OBJECT_0 + index,
            WaitMultipleResult::Timeout => WAIT_TIMEOUT,
            WaitMultipleResult::Failed => {
                self.threads
                    .set_last_error(thread_id, crate::ce::thread::ERROR_INVALID_HANDLE);
                WAIT_FAILED
            }
        }
    }

    pub fn create_window_ex_w(
        &mut self,
        thread_id: u32,
        class_name: &str,
        title: &str,
        parent: Option<u32>,
        id: u32,
        style: u32,
        ex_style: u32,
    ) -> u32 {
        self.create_window_ex_w_with_rect(
            thread_id,
            class_name,
            title,
            parent,
            id,
            style,
            ex_style,
            Rect::default(),
        )
    }

    pub fn create_window_ex_w_with_rect(
        &mut self,
        thread_id: u32,
        class_name: &str,
        title: &str,
        parent: Option<u32>,
        id: u32,
        style: u32,
        ex_style: u32,
        rect: Rect,
    ) -> u32 {
        self.create_window_ex_w_with_parent_owner_and_rect(
            thread_id, class_name, title, parent, None, id, style, ex_style, rect,
        )
    }

    pub fn create_window_ex_w_with_parent_owner_and_rect(
        &mut self,
        thread_id: u32,
        class_name: &str,
        title: &str,
        parent: Option<u32>,
        owner: Option<u32>,
        id: u32,
        style: u32,
        ex_style: u32,
        rect: Rect,
    ) -> u32 {
        let hwnd = self
            .gwe
            .create_window_ex_with_process_parent_owner_and_rect(
                thread_id,
                self.current_process_id,
                class_name,
                title,
                parent,
                owner,
                id,
                style,
                ex_style,
                rect,
            );
        self.handles.insert(KernelObject::Window(hwnd));
        hwnd
    }

    pub fn show_window(&mut self, hwnd: u32, visible: bool) -> bool {
        self.show_window_with_activation(hwnd, visible, visible)
    }

    pub fn show_window_with_activation(
        &mut self,
        hwnd: u32,
        visible: bool,
        activate: bool,
    ) -> bool {
        if !self.gwe.is_window(hwnd) {
            return false;
        }
        let before = self.gwe.get_window_rect(hwnd);
        let was_direct_visible = self.direct_window_visible(hwnd);
        let previous = self.gwe.show_window(hwnd, visible);
        let is_direct_visible = self.direct_window_visible(hwnd);
        if was_direct_visible != is_direct_visible {
            self.post_window_message(hwnd, WM_SHOWWINDOW, u32::from(is_direct_visible), 0);
            let mut flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER;
            if is_direct_visible {
                flags |= SWP_SHOWWINDOW;
                if !activate {
                    flags |= SWP_NOACTIVATE;
                }
            } else {
                flags |= SWP_HIDEWINDOW | SWP_NOACTIVATE;
            }
            self.post_window_rect_messages(
                hwnd,
                before,
                self.gwe.get_window_rect(hwnd),
                HWND_TOP,
                flags,
                true,
            );
            if is_direct_visible {
                self.post_pending_size_move(hwnd);
            }
        }
        if visible && activate {
            let target = self.top_level_window(hwnd);
            let _ = self.activate_window(Some(target));
        } else if !visible {
            self.clear_focus_and_activation_within(hwnd);
        }
        previous
    }

    pub fn update_window(&mut self, hwnd: u32) -> bool {
        if !self.gwe.is_window(hwnd) {
            return false;
        }
        if let Some(update) = self.gwe.update_rect(hwnd) {
            if !self.gwe.is_window_visible(hwnd) {
                return true;
            }
            if update.erase {
                let hdc = 0x0200_0000 | (hwnd & 0x00ff_ffff);
                if !self.erase_window_background(hwnd, hdc) {
                    return false;
                }
            }
            let _ = self.send_message_w(hwnd, crate::ce::gwe::WM_PAINT, 0, 0);
        }
        true
    }

    pub fn erase_window_background(&mut self, hwnd: u32, hdc: u32) -> bool {
        if !self.gwe.is_window(hwnd) {
            return false;
        }
        let Some(result) = self.send_message_w(hwnd, crate::ce::gwe::WM_ERASEBKGND, hdc, 0) else {
            return false;
        };
        if result != 0 {
            self.gwe.clear_update_erase(hwnd)
        } else {
            true
        }
    }

    pub fn set_window_pos(
        &mut self,
        hwnd: u32,
        insert_after: Option<u32>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        flags: u32,
    ) -> bool {
        let before = self.gwe.get_window_rect(hwnd);
        let was_visible = self.direct_window_visible(hwnd);
        let moved = self
            .gwe
            .set_window_pos(hwnd, insert_after, x, y, width, height, flags);
        if moved {
            let after = self.gwe.get_window_rect(hwnd);
            let is_visible = self.direct_window_visible(hwnd);
            self.post_window_visibility_message(hwnd, was_visible, is_visible);
            self.post_window_rect_messages(
                hwnd,
                before,
                after,
                insert_after.unwrap_or(HWND_TOP),
                flags,
                flags & (SWP_SHOWWINDOW | SWP_HIDEWINDOW) != 0 || flags & SWP_NOZORDER == 0,
            );
            if flags & SWP_SHOWWINDOW != 0 && self.direct_window_visible(hwnd) {
                self.post_pending_size_move(hwnd);
            }
            if flags & (SWP_NOACTIVATE | SWP_HIDEWINDOW) == 0 {
                let target = self.top_level_window(hwnd);
                let _ = self.activate_window(Some(target));
            } else if flags & SWP_HIDEWINDOW != 0 {
                self.clear_focus_and_activation_within(hwnd);
            }
        }
        moved
    }

    pub fn enable_window(&mut self, hwnd: u32, enabled: bool) -> Option<bool> {
        let (was_enabled, changed) = self.set_window_enabled_state(hwnd, enabled)?;
        if changed {
            if !enabled {
                self.post_window_message(hwnd, WM_CANCELMODE, 0, 0);
            }
            self.post_window_message(hwnd, WM_ENABLE, u32::from(enabled), 0);
            if !enabled {
                self.clear_focus_and_activation_within(hwnd);
            }
        }
        Some(was_enabled)
    }

    pub fn set_window_enabled_state(&mut self, hwnd: u32, enabled: bool) -> Option<(bool, bool)> {
        if !self.gwe.is_window(hwnd) {
            return None;
        }
        let was_enabled = self.gwe.enable_window(hwnd, enabled);
        let changed = was_enabled != enabled;
        Some((was_enabled, changed))
    }

    pub fn set_parent(&mut self, hwnd: u32, parent: Option<u32>) -> Option<Option<u32>> {
        let previous = self.gwe.set_parent(hwnd, parent)?;
        if !self.gwe.is_window_visible(hwnd) || !self.gwe.is_window_enabled(hwnd) {
            self.clear_focus_and_activation_within(hwnd);
        }
        Some(previous)
    }

    pub fn bring_window_to_top(&mut self, hwnd: u32) -> bool {
        let moved =
            self.gwe
                .set_window_pos(hwnd, Some(HWND_TOP), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        if moved {
            let target = self.top_level_window(hwnd);
            let _ = self.activate_window(Some(target));
        }
        moved
    }

    pub fn move_window(
        &mut self,
        hwnd: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        repaint: bool,
    ) -> bool {
        let before = self.gwe.get_window_rect(hwnd);
        let moved = self.gwe.move_window(hwnd, x, y, width, height, repaint);
        if moved {
            self.post_window_rect_messages(
                hwnd,
                before,
                self.gwe.get_window_rect(hwnd),
                0,
                0,
                true,
            );
        }
        moved
    }

    pub fn set_focus(&mut self, hwnd: Option<u32>) -> Option<u32> {
        if hwnd.is_some_and(|hwnd| !self.gwe.is_window(hwnd) || !self.gwe.is_window_enabled(hwnd)) {
            return None;
        }
        let previous = self.gwe.set_focus(hwnd);
        if previous != hwnd {
            if let Some(previous_hwnd) = previous {
                self.post_window_message(previous_hwnd, WM_KILLFOCUS, hwnd.unwrap_or(0), 0);
            }
            if let Some(hwnd) = hwnd {
                let _ = self.activate_window(Some(hwnd));
                self.post_window_message(hwnd, WM_SETFOCUS, previous.unwrap_or(0), 0);
            }
        }
        previous
    }

    pub fn activate_window(&mut self, hwnd: Option<u32>) -> Option<u32> {
        if hwnd.is_some_and(|hwnd| !self.gwe.is_window(hwnd) || !self.gwe.is_window_enabled(hwnd)) {
            return None;
        }
        let previous = self.gwe.set_active_window(hwnd);
        if previous != hwnd {
            if let Some(previous_hwnd) = previous {
                self.post_window_message(
                    previous_hwnd,
                    WM_ACTIVATE,
                    WA_INACTIVE,
                    hwnd.unwrap_or(0),
                );
            }
            if let Some(hwnd) = hwnd {
                self.post_window_message(hwnd, WM_ACTIVATE, WA_ACTIVE, previous.unwrap_or(0));
            }
        }
        previous
    }

    pub(crate) fn clear_focus_and_activation_within(&mut self, hwnd: u32) {
        if self.gwe.focus_is_within(hwnd) {
            let _ = self.set_focus(None);
        }
        if self.gwe.active_window_is_within(hwnd) {
            let _ = self.activate_window(None);
        }
        if self.gwe.keyboard_target_is_within(hwnd) {
            self.gwe.clear_keyboard_targets_within(hwnd);
        }
    }

    pub(crate) fn clear_destroyed_window_focus_and_activation(&mut self, hwnd: u32) {
        if self.gwe.focus_is_within(hwnd) {
            let _ = self.set_focus(None);
        }
        if self.gwe.active_window_is_within(hwnd) {
            let _ = self.gwe.set_active_window(None);
        }
        if self.gwe.keyboard_target_is_within(hwnd) {
            self.gwe.clear_keyboard_targets_within(hwnd);
        }
    }

    fn top_level_window(&self, hwnd: u32) -> u32 {
        let mut current = hwnd;
        while let Some(parent) = self.gwe.get_parent(current) {
            current = parent;
        }
        current
    }

    pub fn get_message_w(&mut self, thread_id: u32) -> Option<Message> {
        self.get_message_w_filtered(thread_id, None, 0, 0)
    }

    pub fn get_message_w_filtered(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
    ) -> Option<Message> {
        self.expire_timed_out_send_messages();
        self.drain_remote_input_to_thread_window(thread_id, hwnd);
        if let Some(message) = self
            .gwe
            .get_message_filtered(thread_id, hwnd, min_msg, max_msg)
        {
            self.clear_timer_message_pending(thread_id, &message);
            self.record_message_op(
                "get_message",
                thread_id,
                &message,
                Some(1),
                Some(format_filter_detail(hwnd, min_msg, max_msg)),
            );
            return Some(message);
        }
        self.pump_timers_to_gwe(thread_id);
        let message = self
            .gwe
            .get_message_filtered(thread_id, hwnd, min_msg, max_msg);
        if let Some(message) = message.as_ref() {
            self.clear_timer_message_pending(thread_id, message);
            self.record_message_op(
                "get_message",
                thread_id,
                message,
                Some(1),
                Some(format_filter_detail(hwnd, min_msg, max_msg)),
            );
        }
        message
    }

    pub fn peek_message_w_filtered(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
        flags: PeekFlags,
    ) -> Option<Message> {
        self.expire_timed_out_send_messages();
        self.drain_remote_input_to_thread_window(thread_id, hwnd);
        if let Some(message) = self
            .gwe
            .peek_message_filtered(thread_id, hwnd, min_msg, max_msg, flags)
        {
            if flags.contains(PeekFlags::REMOVE) {
                self.clear_timer_message_pending(thread_id, &message);
            }
            let op = if flags.contains(PeekFlags::REMOVE) {
                "peek_message_remove"
            } else {
                "peek_message"
            };
            self.record_message_op(
                op,
                thread_id,
                &message,
                Some(1),
                Some(format_filter_detail(hwnd, min_msg, max_msg)),
            );
            return Some(message);
        }
        self.pump_timers_to_gwe(thread_id);
        let message = self
            .gwe
            .peek_message_filtered(thread_id, hwnd, min_msg, max_msg, flags);
        if let Some(message) = message.as_ref() {
            if flags.contains(PeekFlags::REMOVE) {
                self.clear_timer_message_pending(thread_id, message);
            }
            let op = if flags.contains(PeekFlags::REMOVE) {
                "peek_message_remove"
            } else {
                "peek_message"
            };
            self.record_message_op(
                op,
                thread_id,
                message,
                Some(1),
                Some(format_filter_detail(hwnd, min_msg, max_msg)),
            );
        }
        message
    }

    pub fn post_message_w(&mut self, hwnd: u32, msg: u32, wparam: u32, lparam: u32) -> bool {
        self.post_message_w_for_thread(0, hwnd, msg, wparam, lparam)
    }

    pub fn post_message_w_for_thread(
        &mut self,
        thread_id: u32,
        hwnd: u32,
        msg: u32,
        wparam: u32,
        lparam: u32,
    ) -> bool {
        let time_ms = self.timers.tick_count();
        match hwnd {
            HWND_BROADCAST => {
                let target_threads: Vec<u32> = self
                    .gwe
                    .windows_snapshot()
                    .into_iter()
                    .filter(|window| !window.destroyed && window.parent.is_none())
                    .map(|window| window.thread_id)
                    .collect();
                let target_messages: Vec<(u32, Message)> = self
                    .gwe
                    .windows_snapshot()
                    .into_iter()
                    .filter(|window| !window.destroyed && window.parent.is_none())
                    .map(|window| {
                        (
                            window.thread_id,
                            Message::new(window.hwnd, msg, wparam, lparam, time_ms),
                        )
                    })
                    .collect();
                let posted = self
                    .gwe
                    .post_broadcast_message(msg, wparam, lparam, time_ms);
                if posted {
                    for (target_thread, message) in &target_messages {
                        self.record_message_op(
                            "post_message",
                            *target_thread,
                            message,
                            Some(1),
                            Some("broadcast".to_owned()),
                        );
                    }
                    for target_thread in target_threads {
                        self.queue_message_wake_candidates(target_thread);
                    }
                }
                posted
            }
            0 => {
                let message = Message::new(0, msg, wparam, lparam, time_ms);
                self.gwe.post_message(thread_id, message.clone());
                self.record_message_op(
                    "post_message",
                    thread_id,
                    &message,
                    Some(1),
                    Some("thread".to_owned()),
                );
                self.queue_message_wake_candidates(thread_id);
                true
            }
            hwnd => {
                let target_thread = self.gwe.window(hwnd).map(|window| window.thread_id);
                let message = Message::new(hwnd, msg, wparam, lparam, time_ms);
                let posted = self.gwe.post_message_for_window(hwnd, message.clone());
                if posted {
                    if let Some(target_thread) = target_thread {
                        self.record_message_op(
                            "post_message",
                            target_thread,
                            &message,
                            Some(1),
                            Some("window".to_owned()),
                        );
                        self.queue_message_wake_candidates(target_thread);
                    }
                }
                posted
            }
        }
    }

    pub fn post_keybd_message_for_thread(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        virtual_key: u32,
        key_down: bool,
        lparam: u32,
        characters: &[u32],
    ) -> bool {
        let time_ms = self.timers.tick_count();
        let target_hwnd = hwnd
            .or_else(|| self.gwe.get_keyboard_target(thread_id))
            .or_else(|| self.gwe.get_focus())
            .or_else(|| self.gwe.get_active_window());
        let target_thread = match target_hwnd {
            Some(hwnd) => match self.gwe.window(hwnd).filter(|window| !window.destroyed) {
                Some(window) => window.thread_id,
                None => return false,
            },
            None => thread_id,
        };
        let hwnd_value = target_hwnd.unwrap_or(0);
        let key_msg = if key_down {
            crate::ce::gwe::WM_KEYDOWN
        } else {
            crate::ce::gwe::WM_KEYUP
        };
        let key_message = Message {
            hwnd: hwnd_value,
            msg: key_msg,
            wparam: virtual_key,
            lparam,
            time_ms,
            source: crate::ce::gwe::MSGSRC_HARDWARE_KEYBOARD,
            mouse_pos_at_post: None,
        };
        self.gwe.post_message(target_thread, key_message.clone());
        self.record_message_op(
            "post_message",
            target_thread,
            &key_message,
            Some(1),
            Some("keyboard".to_owned()),
        );
        if key_down {
            for character in characters
                .iter()
                .copied()
                .filter(|character| *character != 0)
            {
                let char_message = Message {
                    hwnd: hwnd_value,
                    msg: crate::ce::gwe::WM_CHAR,
                    wparam: character,
                    lparam,
                    time_ms,
                    source: crate::ce::gwe::MSGSRC_HARDWARE_KEYBOARD,
                    mouse_pos_at_post: None,
                };
                self.gwe.post_message(target_thread, char_message.clone());
                self.record_message_op(
                    "post_message",
                    target_thread,
                    &char_message,
                    Some(1),
                    Some("keyboard".to_owned()),
                );
            }
        }
        self.queue_message_wake_candidates(target_thread);
        true
    }

    pub fn set_keyboard_target(&mut self, thread_id: u32, hwnd: Option<u32>) -> Option<u32> {
        self.gwe.set_keyboard_target(thread_id, hwnd)
    }

    pub fn post_thread_message_w(
        &mut self,
        target_thread_id: u32,
        msg: u32,
        wparam: u32,
        lparam: u32,
    ) -> bool {
        let message = Message::new(0, msg, wparam, lparam, self.timers.tick_count());
        self.gwe.post_message(target_thread_id, message.clone());
        self.record_message_op(
            "post_message",
            target_thread_id,
            &message,
            Some(1),
            Some("thread".to_owned()),
        );
        self.queue_message_wake_candidates(target_thread_id);
        true
    }

    pub fn send_message_w(&mut self, hwnd: u32, msg: u32, wparam: u32, lparam: u32) -> Option<u32> {
        let target_thread = self
            .gwe
            .window(hwnd)
            .filter(|window| !window.destroyed)
            .map(|window| window.thread_id)?;
        self.gwe.begin_send_message(target_thread);
        let result = self.gwe.send_message(hwnd, msg, wparam, lparam);
        self.gwe.end_send_message(target_thread);
        result
    }

    pub fn begin_cross_thread_send_message_w(
        &mut self,
        caller_thread_id: u32,
        hwnd: u32,
        msg: u32,
        wparam: u32,
        lparam: u32,
        timeout_ms: Option<u32>,
    ) -> Option<u64> {
        let target_thread = self
            .gwe
            .window(hwnd)
            .filter(|window| !window.destroyed)
            .map(|window| window.thread_id)?;
        if target_thread == caller_thread_id {
            return None;
        }
        let flags = timeout_ms.map_or(SMF_NULL, |_| crate::ce::gwe::SMF_TIMEOUT);
        let send_id = self.gwe.queue_send_message_for_window(
            Some(caller_thread_id),
            hwnd,
            Message::new(hwnd, msg, wparam, lparam, self.timers.tick_count()),
            flags,
            timeout_ms,
        );
        if send_id.is_some() {
            self.record_message_trace(MessageTraceRecord {
                op: "queue_send_message",
                thread_id: target_thread,
                hwnd: Some(hwnd),
                msg: Some(msg),
                wparam: Some(wparam),
                lparam: Some(lparam),
                screen_pos: None,
                source: Some(crate::ce::gwe::MSGSRC_SOFTWARE_SEND),
                result: Some(1),
                detail: Some(format!("sender_thread={caller_thread_id}")),
            });
            self.queue_message_wake_candidates(target_thread);
        }
        send_id
    }

    pub fn take_completed_send_message_result(&mut self, send_id: u64) -> Option<u32> {
        self.gwe.take_completed_sent_message_result(send_id)
    }

    pub fn complete_active_sent_message(&mut self, thread_id: u32, result: u32) -> Option<u64> {
        let was_ready = self
            .gwe
            .active_sent_message_id(thread_id)
            .is_some_and(|send_id| self.gwe.sent_message_result_ready(send_id));
        let send_id = self.gwe.complete_active_sent_message(thread_id, result)?;
        if !was_ready {
            self.queue_send_reply_wake_candidates(send_id);
        }
        Some(send_id)
    }

    pub fn reply_message(&mut self, thread_id: u32, result: u32) -> bool {
        let Some(send_id) = self.gwe.reply_message(thread_id, result) else {
            return false;
        };
        self.queue_send_reply_wake_candidates(send_id);
        true
    }

    pub fn destroy_window(&mut self, hwnd: u32) -> bool {
        let Some(targets) = self.gwe.window_and_descendants(hwnd) else {
            return false;
        };
        self.clear_destroyed_window_focus_and_activation(hwnd);
        let doomed_send_ids = self.gwe.sent_message_ids_for_windows(&targets);
        for target in targets.iter().rev().copied() {
            if self
                .gwe
                .window(target)
                .is_some_and(|window| !window.destroyed && !window.destroy_message_sent)
            {
                let _ = self.send_message_w(target, crate::ce::gwe::WM_DESTROY, 0, 0);
            }
        }
        let destroyed = self.gwe.destroy_window(hwnd, self.timers.tick_count());
        if destroyed {
            self.timers.remove_window_timers(&targets);
            for send_id in doomed_send_ids {
                self.queue_send_reply_wake_candidates(send_id);
            }
        }
        destroyed
    }

    pub fn send_notify_message_w(
        &mut self,
        caller_thread_id: u32,
        hwnd: u32,
        msg: u32,
        wparam: u32,
        lparam: u32,
    ) -> bool {
        if hwnd == HWND_BROADCAST {
            let targets: Vec<(u32, u32)> = self
                .gwe
                .windows_snapshot()
                .into_iter()
                .filter(|window| !window.destroyed && window.parent.is_none())
                .map(|window| (window.hwnd, window.thread_id))
                .collect();
            let posted =
                self.gwe
                    .post_broadcast_message(msg, wparam, lparam, self.timers.tick_count());
            if posted {
                for (target_hwnd, target_thread) in targets {
                    self.record_message_trace(MessageTraceRecord {
                        op: "send_notify_message",
                        thread_id: target_thread,
                        hwnd: Some(target_hwnd),
                        msg: Some(msg),
                        wparam: Some(wparam),
                        lparam: Some(lparam),
                        screen_pos: None,
                        source: Some(crate::ce::gwe::MSGSRC_SOFTWARE_POST),
                        result: Some(1),
                        detail: Some(format!("sender_thread={caller_thread_id}/broadcast")),
                    });
                    self.queue_message_wake_candidates(target_thread);
                }
            }
            return posted;
        }
        let Some(target_thread) = self
            .gwe
            .window(hwnd)
            .filter(|window| !window.destroyed)
            .map(|window| window.thread_id)
        else {
            return false;
        };
        if target_thread == caller_thread_id {
            return self.send_message_w(hwnd, msg, wparam, lparam).is_some();
        }
        let queued = self
            .gwe
            .queue_send_message_for_window(
                None,
                hwnd,
                Message::new(hwnd, msg, wparam, lparam, self.timers.tick_count()),
                crate::ce::gwe::SMF_SENDER_NO_WAIT | crate::ce::gwe::SMF_NOTIFY_MESSAGE,
                None,
            )
            .is_some();
        if queued {
            self.record_message_trace(MessageTraceRecord {
                op: "send_notify_message",
                thread_id: target_thread,
                hwnd: Some(hwnd),
                msg: Some(msg),
                wparam: Some(wparam),
                lparam: Some(lparam),
                screen_pos: None,
                source: Some(crate::ce::gwe::MSGSRC_SOFTWARE_SEND),
                result: Some(1),
                detail: Some(format!("sender_thread={caller_thread_id}")),
            });
            self.queue_message_wake_candidates(target_thread);
        }
        queued
    }

    pub fn post_quit_message(&mut self, thread_id: u32, exit_code: u32) {
        self.gwe
            .post_quit_message(thread_id, exit_code, self.timers.tick_count());
        self.queue_message_wake_candidates(thread_id);
    }

    pub fn dispatch_message_w(&mut self, message: Message) -> u32 {
        self.dispatch_message_w_for_thread(0, message)
    }

    pub fn dispatch_message_w_for_thread(&mut self, thread_id: u32, message: Message) -> u32 {
        if message.msg == crate::ce::gwe::WM_QUIT {
            return message.wparam;
        }
        let sent_context_thread = if message.source == crate::ce::gwe::MSGSRC_SOFTWARE_SEND {
            self.gwe
                .window(message.hwnd)
                .map(|window| window.thread_id)
                .filter(|thread_id| self.gwe.active_sent_message_id(*thread_id).is_some())
        } else {
            None
        }
        .or_else(|| {
            (thread_id != 0 && self.gwe.active_sent_message_id(thread_id).is_some())
                .then_some(thread_id)
        });
        let result = if sent_context_thread.is_some() {
            self.gwe
                .send_message(message.hwnd, message.msg, message.wparam, message.lparam)
        } else {
            self.send_message_w(message.hwnd, message.msg, message.wparam, message.lparam)
        }
        .unwrap_or(0);
        if let Some(sent_context_thread) = sent_context_thread {
            self.complete_active_sent_message(sent_context_thread, result);
        }
        if message.msg == WM_WINDOWPOSCHANGED {
            self.release_message_pointer_payload(message.lparam);
        }
        result
    }

    pub fn message_pointer_payload(&self, ptr: u32) -> Option<MessagePointerPayload> {
        self.gwe.message_pointer_payload(ptr)
    }

    pub fn release_message_pointer_payload(&mut self, ptr: u32) -> Option<MessagePointerPayload> {
        let payload = self.gwe.take_message_pointer_payload(ptr)?;
        let _ = self.memory.heap_free(PROCESS_HEAP_HANDLE, 0, ptr);
        Some(payload)
    }

    pub fn message_pump_step(&mut self, thread_id: u32) -> MessagePumpResult {
        let Some(message) = self.get_message_w(thread_id) else {
            return MessagePumpResult::Idle;
        };
        if message.msg == crate::ce::gwe::WM_QUIT {
            return MessagePumpResult::Quit(message.wparam);
        }
        MessagePumpResult::Dispatched(self.dispatch_message_w_for_thread(thread_id, message))
    }

    fn post_gwe_message(&mut self, thread_id: u32, message: Message) {
        let trace_message = message.clone();
        self.gwe.post_message(thread_id, message);
        self.record_message_op("post_message", thread_id, &trace_message, Some(1), None);
        self.queue_message_wake_candidates(thread_id);
    }

    pub fn set_timer(
        &mut self,
        hwnd: Option<u32>,
        requested_id: Option<u32>,
        period_ms: u32,
    ) -> u32 {
        let thread_id = hwnd
            .and_then(|hwnd| self.gwe.window_thread_process_id(hwnd))
            .map(|(thread_id, _)| thread_id)
            .unwrap_or(0);
        self.set_timer_for_thread(thread_id, hwnd, requested_id, period_ms, None)
    }

    pub fn set_timer_for_thread(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        requested_id: Option<u32>,
        period_ms: u32,
        callback: Option<u32>,
    ) -> u32 {
        self.timers.set_timer(
            thread_id,
            hwnd,
            requested_id,
            period_ms,
            crate::ce::gwe::WM_TIMER,
            callback,
        )
    }

    pub fn kill_timer(&mut self, hwnd: Option<u32>, id: u32) -> bool {
        let thread_id = hwnd
            .and_then(|hwnd| self.gwe.window_thread_process_id(hwnd))
            .map(|(thread_id, _)| thread_id)
            .unwrap_or(0);
        self.kill_timer_for_thread(thread_id, hwnd, id)
    }

    pub fn kill_timer_for_thread(&mut self, thread_id: u32, hwnd: Option<u32>, id: u32) -> bool {
        self.timers.kill_timer(thread_id, hwnd, id)
    }

    pub fn remote_gps_target(&self) -> String {
        self.devices.remote_gps_target().unwrap_or_default()
    }

    pub fn remote_status(&self) -> RemoteStatus {
        self.remote.status(self.remote_gps_target())
    }

    pub fn remote_status_json(&self) -> serde_json::Value {
        self.remote.status_json(self.remote_gps_target())
    }

    pub fn dispatch_remote_control_message(
        &mut self,
        message: &serde_json::Value,
    ) -> serde_json::Value {
        let gps_target = self.remote_gps_target();
        let serial_before = self.remote.serial_byte_count();
        let response = self.remote.dispatch_control_message(message, gps_target);
        if self.remote.serial_byte_count() > serial_before {
            self.queue_all_serial_read_wake_candidates();
            self.queue_all_serial_event_wake_candidates();
        }
        response
    }

    pub fn read_remote_serial_bytes(&mut self, max_bytes: usize) -> Vec<u8> {
        self.remote.read_serial_bytes(max_bytes)
    }

    pub fn drain_remote_input_to_gwe(&mut self, thread_id: u32, hwnd: u32) -> usize {
        if !self.gwe.is_window(hwnd) {
            return 0;
        }
        self.drain_remote_input_to_target(thread_id, Some(hwnd), false)
    }

    fn drain_remote_input_to_target(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        hit_test_touches: bool,
    ) -> usize {
        let time_ms = self.timers.tick_count();
        let touch_events = self.remote.drain_touch_events();
        let key_events = self.remote.drain_key_events();
        let mut posted = 0;

        for event in touch_events {
            let point = Point {
                x: event.x,
                y: event.y,
            };
            let target = if hit_test_touches {
                self.gwe
                    .get_capture()
                    .or_else(|| self.gwe.window_from_point_for_thread(thread_id, point))
                    .or(hwnd)
            } else {
                hwnd
            };
            let Some(target) = target.filter(|hwnd| self.gwe.is_window(*hwnd)) else {
                self.record_message_trace(MessageTraceRecord {
                    op: "remote_touch_drop",
                    thread_id,
                    hwnd,
                    msg: Some(event.message),
                    wparam: None,
                    lparam: None,
                    screen_pos: Some(make_lparam(point.x, point.y)),
                    source: None,
                    result: Some(0),
                    detail: Some(if hit_test_touches {
                        "no hit-test target".to_owned()
                    } else {
                        "no target window".to_owned()
                    }),
                });
                continue;
            };
            if event.message == WM_LBUTTONDOWN {
                let _ = self.set_focus(Some(target));
            }
            let client = self.gwe.screen_to_client(target, point).unwrap_or(point);
            let wparam = if event.message == WM_LBUTTONDOWN || event.message == WM_MOUSEMOVE {
                1
            } else {
                0
            };
            self.post_gwe_message(
                thread_id,
                Message::new(
                    target,
                    event.message,
                    wparam,
                    make_lparam(client.x, client.y),
                    time_ms,
                )
                .with_mouse_pos(make_lparam(point.x, point.y)),
            );
            self.record_message_trace(MessageTraceRecord {
                op: "remote_touch_target",
                thread_id,
                hwnd: Some(target),
                msg: Some(event.message),
                wparam: Some(wparam),
                lparam: Some(make_lparam(client.x, client.y)),
                screen_pos: Some(make_lparam(point.x, point.y)),
                source: Some(crate::ce::gwe::MSGSRC_SOFTWARE_POST),
                result: Some(1),
                detail: Some(if hit_test_touches {
                    "hit-test".to_owned()
                } else {
                    "explicit-target".to_owned()
                }),
            });
            posted += 1;
        }

        let key_target = hwnd
            .or_else(|| self.gwe.get_capture())
            .or_else(|| self.gwe.get_active_window());
        for event in key_events {
            let Some(key_target) = key_target.filter(|hwnd| self.gwe.is_window(*hwnd)) else {
                self.record_message_trace(MessageTraceRecord {
                    op: "remote_key_drop",
                    thread_id,
                    hwnd: key_target,
                    msg: Some(event.message),
                    wparam: Some(event.vk),
                    lparam: Some(1),
                    screen_pos: None,
                    source: Some(crate::ce::gwe::MSGSRC_HARDWARE_KEYBOARD),
                    result: Some(0),
                    detail: Some("no target window".to_owned()),
                });
                continue;
            };
            self.post_gwe_message(
                thread_id,
                Message::new(key_target, event.message, event.vk, 1, time_ms)
                    .with_source(crate::ce::gwe::MSGSRC_HARDWARE_KEYBOARD),
            );
            self.record_message_trace(MessageTraceRecord {
                op: "remote_key_target",
                thread_id,
                hwnd: Some(key_target),
                msg: Some(event.message),
                wparam: Some(event.vk),
                lparam: Some(1),
                screen_pos: None,
                source: Some(crate::ce::gwe::MSGSRC_HARDWARE_KEYBOARD),
                result: Some(1),
                detail: None,
            });
            posted += 1;
        }

        posted
    }

    pub fn drain_remote_input_to_thread_window(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
    ) -> usize {
        let hwnd = hwnd
            .filter(|hwnd| self.gwe.is_window(*hwnd))
            .or_else(|| self.gwe.get_capture())
            .or_else(|| self.gwe.get_active_window());
        self.drain_remote_input_to_target(thread_id, hwnd, true)
    }

    pub fn drain_remote_input_to_active_window(&mut self) -> usize {
        let hwnd = self
            .gwe
            .get_capture()
            .or_else(|| self.gwe.get_active_window())
            .filter(|hwnd| self.gwe.is_window(*hwnd));
        let Some(hwnd) = hwnd else {
            return 0;
        };
        let Some((thread_id, _process_id)) = self.gwe.window_thread_process_id(hwnd) else {
            return 0;
        };
        self.drain_remote_input_to_target(thread_id, Some(hwnd), true)
    }

    pub fn wave_out_open(&mut self, format: WaveFormat) -> std::result::Result<u32, MmResult> {
        self.audio.wave_out_open(format)
    }

    pub fn wave_out_write(&mut self, id: u32, buffer: WaveBuffer) -> MmResult {
        self.audio.wave_out_write(id, buffer)
    }

    fn post_window_visibility_message(&mut self, hwnd: u32, before: bool, after: bool) {
        if before != after {
            self.post_window_message(hwnd, WM_SHOWWINDOW, u32::from(after), 0);
        }
    }

    fn direct_window_visible(&self, hwnd: u32) -> bool {
        self.gwe
            .window(hwnd)
            .is_some_and(|window| window.visible && window.style & crate::ce::gwe::WS_VISIBLE != 0)
    }

    fn post_window_rect_messages(
        &mut self,
        hwnd: u32,
        before: Option<Rect>,
        after: Option<Rect>,
        insert_after: u32,
        flags: u32,
        force_window_pos_changed: bool,
    ) {
        let (Some(before), Some(after)) = (before, after) else {
            return;
        };
        if before != after || force_window_pos_changed {
            let lparam = self
                .gwe
                .window_pos_for_rect(hwnd, after, insert_after, flags)
                .map(|payload| self.queue_window_pos_payload(payload))
                .unwrap_or(0);
            self.post_window_message(hwnd, WM_WINDOWPOSCHANGED, 0, lparam);
        }
        let moved = before.left != after.left || before.top != after.top;
        let sized = before.width() != after.width() || before.height() != after.height();
        if moved || sized {
            if self.direct_window_visible(hwnd) {
                self.post_size_move(hwnd, after, moved, sized);
            } else {
                let _ = self.gwe.mark_pending_size_move(hwnd, moved, sized);
            }
        }
    }

    fn post_pending_size_move(&mut self, hwnd: u32) {
        let Some((rect, moved, sized)) = self.gwe.take_pending_size_move(hwnd) else {
            return;
        };
        self.post_size_move(hwnd, rect, moved, sized);
    }

    fn post_size_move(&mut self, hwnd: u32, rect: Rect, moved: bool, sized: bool) {
        if moved {
            self.post_window_message(hwnd, WM_MOVE, 0, make_lparam_i16(rect.left, rect.top));
        }
        if sized {
            self.post_window_message(
                hwnd,
                WM_SIZE,
                0,
                make_lparam_i16(rect.width(), rect.height()),
            );
        }
    }

    fn queue_window_pos_payload(&mut self, payload: WindowPos) -> u32 {
        let Some(ptr) = self.memory.heap_alloc(PROCESS_HEAP_HANDLE, 0, 28) else {
            return 0;
        };
        if self
            .gwe
            .insert_message_pointer_payload(ptr, MessagePointerPayload::WindowPos(payload))
        {
            ptr
        } else {
            let _ = self.memory.heap_free(PROCESS_HEAP_HANDLE, 0, ptr);
            0
        }
    }

    fn post_window_message(&mut self, hwnd: u32, msg: u32, wparam: u32, lparam: u32) {
        let Some(window) = self.gwe.window(hwnd) else {
            return;
        };
        let message = Message::new(hwnd, msg, wparam, lparam, self.timers.tick_count());
        self.post_gwe_message(window.thread_id, message);
    }

    fn clear_timer_message_pending(&mut self, thread_id: u32, message: &Message) {
        if message.msg != crate::ce::gwe::WM_TIMER {
            return;
        }
        let hwnd = (message.hwnd != 0).then_some(message.hwnd);
        let _ = self
            .timers
            .clear_pending_message(thread_id, hwnd, message.wparam);
    }
}

fn format_filter_detail(hwnd: Option<u32>, min_msg: u32, max_msg: u32) -> String {
    format!(
        "filter_hwnd={} min=0x{min_msg:08x} max=0x{max_msg:08x}",
        hwnd.map_or_else(|| "any".to_owned(), |hwnd| format!("0x{hwnd:08x}"))
    )
}

fn file_trace_preview(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }
    let mut preview = String::new();
    for &byte in bytes.iter().take(FILE_TRACE_PREVIEW_LIMIT) {
        match byte {
            b'\r' => preview.push_str("\\r"),
            b'\n' => preview.push_str("\\n"),
            b'\t' => preview.push_str("\\t"),
            0x20..=0x7e => preview.push(byte as char),
            _ => preview.push_str(&format!("\\x{byte:02x}")),
        }
    }
    if bytes.len() > FILE_TRACE_PREVIEW_LIMIT {
        preview.push_str("...");
    }
    Some(preview)
}

fn file_read_trace_preview(
    start_position: Option<u64>,
    end_position: Option<u64>,
    bytes: &[u8],
) -> Option<String> {
    let mut parts = Vec::new();
    if let (Some(start), Some(end)) = (start_position, end_position) {
        parts.push(format!("pos={start}..{end}"));
    }
    if let Some(preview) = file_trace_preview(bytes) {
        parts.push(format!("bytes={preview}"));
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}

fn is_file_open_trace(op: &str) -> bool {
    matches!(op, "CreateFileW" | "CreateFileWArg" | "FindFirstFileW")
}

fn make_lparam_i16(low: i32, high: i32) -> u32 {
    ((high as u32) & 0xffff) << 16 | ((low as u32) & 0xffff)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ce::gwe::{WM_ACTIVATE, WM_KILLFOCUS, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_SETFOCUS, WS_CHILD},
        config::RuntimeConfig,
    };

    #[test]
    fn destroy_cleanup_clears_dead_active_without_posting_inactive_app_window() -> Result<()> {
        let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
        let mut kernel = CeKernel::boot(config);
        let thread_id = 1;
        let parent = kernel.create_window_ex_w(thread_id, "TOP", "", None, 0, 0, 0);
        let child = kernel.create_window_ex_w(thread_id, "CHILD", "", Some(parent), 0, WS_CHILD, 0);

        assert_eq!(kernel.set_focus(Some(child)), None);
        assert_eq!(kernel.gwe.get_message(thread_id).unwrap().msg, WM_ACTIVATE);
        assert_eq!(kernel.gwe.get_message(thread_id).unwrap().msg, WM_SETFOCUS);
        assert_eq!(kernel.gwe.set_active_window(Some(child)), Some(child));

        kernel.clear_destroyed_window_focus_and_activation(child);

        let kill_focus = kernel.gwe.get_message(thread_id).unwrap();
        assert_eq!(kill_focus.hwnd, child);
        assert_eq!(kill_focus.msg, WM_KILLFOCUS);
        assert_eq!(kill_focus.wparam, 0);
        assert!(!kernel.gwe.active_window_is_within(child));
        assert_eq!(kernel.gwe.get_active_window(), Some(parent));
        assert!(kernel.gwe.get_message(thread_id).is_none());

        Ok(())
    }

    #[test]
    fn remote_input_active_window_drain_posts_mouse_messages() -> Result<()> {
        let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
        let mut kernel = CeKernel::boot(config);
        let thread_id = 1;
        let hwnd = kernel.create_window_ex_w(thread_id, "ACTIVE", "", None, 0, 0, 0);
        assert_eq!(kernel.gwe.set_active_window(Some(hwnd)), None);
        kernel.remote.set_framebuffer_size(800, 480);
        kernel.remote.enqueue_touch("tap", 10, 20).unwrap();

        assert_eq!(kernel.drain_remote_input_to_active_window(), 2);

        let mut messages = Vec::new();
        while let Some(message) = kernel.gwe.get_message(thread_id) {
            messages.push((message.hwnd, message.msg));
        }
        assert!(messages.contains(&(hwnd, WM_LBUTTONDOWN)));
        assert!(messages.contains(&(hwnd, WM_LBUTTONUP)));
        Ok(())
    }
}
