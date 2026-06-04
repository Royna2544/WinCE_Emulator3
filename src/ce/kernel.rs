use crate::{
    ce::{
        audio::{AudioSystem, MmResult, WaveBuffer, WaveFormat},
        cemath::CeMath,
        com::ComSystem,
        devices::{DeviceIoControlResult, DeviceNamespace},
        file::{
            FileIoResult, FileIoStats, FindData, GENERIC_READ, GENERIC_WRITE, HostFileSystem,
            OPEN_EXISTING,
        },
        gwe::{
            Gwe, GweStats, HWND_BROADCAST, HWND_TOP, Message, MessagePointerPayload, Point, Rect,
            SMF_NULL, SWP_HIDEWINDOW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, WA_ACTIVE,
            WA_INACTIVE, WM_ACTIVATE, WM_CANCELMODE, WM_ENABLE, WM_KILLFOCUS, WM_MOVE, WM_SETFOCUS,
            WM_SHOWWINDOW, WM_SIZE, WM_WINDOWPOSCHANGED, WindowPos,
        },
        memory::{MemorySystem, PROCESS_HEAP_HANDLE},
        object::{FileObject, FindFileObject, HandleTable, KernelObject, WaitResult},
        registry::Registry,
        remote::{CeRemote, RemoteStatus, WM_LBUTTONDOWN, WM_MOUSEMOVE, make_lparam},
        resource::ResourceSystem,
        scheduler::{Scheduler, SchedulerStats, SchedulerWaitKind, SchedulerWakeReason},
        thread::ThreadSystem,
        timer::{TimerSystem, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT},
    },
    config::RuntimeConfig,
    error::Result,
};

use std::{collections::BTreeMap, path::PathBuf};

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
    pending_process_launches: Vec<PendingProcessLaunch>,
    next_process_id: u32,
    crt_rand_state: u32,
    crt_strtok_next_by_thread: BTreeMap<u32, u32>,
    recent_file_ops: Vec<FileTraceRecord>,
    recent_file_open_ops: Vec<FileTraceRecord>,
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

const FILE_TRACE_LIMIT: usize = 512;
const FILE_TRACE_PREVIEW_LIMIT: usize = 64;

fn wait_result_to_wake_reason(result: u32) -> SchedulerWakeReason {
    if result == WAIT_FAILED {
        SchedulerWakeReason::Failed
    } else if result == WAIT_TIMEOUT {
        SchedulerWakeReason::Timeout
    } else {
        SchedulerWakeReason::ObjectSignaled
    }
}

impl CeKernel {
    pub fn boot(config: RuntimeConfig) -> Self {
        Self {
            registry: Registry::from_dump(config.registry),
            devices: DeviceNamespace::from_config(config.devices),
            files: HostFileSystem::from_storage(".", config.storage),
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
            current_process_id: 0,
            pending_process_launches: Vec::new(),
            next_process_id: 0x42,
            crt_rand_state: 1,
            crt_strtok_next_by_thread: BTreeMap::new(),
            recent_file_ops: Vec::new(),
            recent_file_open_ops: Vec::new(),
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
        launch
    }

    pub fn take_pending_process_launches(&mut self) -> Vec<PendingProcessLaunch> {
        std::mem::take(&mut self.pending_process_launches)
    }

    pub fn mark_process_launch_exited(&mut self, launch: &PendingProcessLaunch, exit_code: u32) {
        self.handles
            .mark_process_exited(launch.process_handle, exit_code);
        self.handles
            .mark_thread_exited(launch.thread_handle, exit_code);
    }

    pub fn pump_timers_to_gwe(&mut self, thread_id: u32) {
        self.expire_timed_out_send_messages();
        for timer in self.timers.due_timers() {
            if let Some(hwnd) = timer.hwnd {
                let message = crate::ce::gwe::Message::new(
                    hwnd,
                    timer.message,
                    timer.id,
                    0,
                    self.timers.tick_count(),
                );
                self.gwe.post_message(thread_id, message);
            }
        }
    }

    pub fn expire_timed_out_send_messages(&mut self) -> Vec<u64> {
        self.gwe
            .expire_timed_out_sent_messages(self.timers.tick_count())
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

    pub fn recent_file_ops(&self) -> &[FileTraceRecord] {
        &self.recent_file_ops
    }

    pub fn recent_file_open_ops(&self) -> &[FileTraceRecord] {
        &self.recent_file_open_ops
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
            preview: match (start_position, end_position) {
                (Some(start), Some(end)) => Some(format!("pos={start}..{end}")),
                _ => None,
            },
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
                KernelObject::File(file) => {
                    self.files
                        .read_file_into(file.file_id, requested, |bytes| write(bytes))
                }
                KernelObject::Device(device) => {
                    let bytes = device.read_file(requested);
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
            preview: match (start_position, end_position) {
                (Some(start), Some(end)) => Some(format!("pos={start}..{end}")),
                _ => None,
            },
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
        let position = self.file_position(handle)?;
        let size = self.get_file_size(handle)?;
        Ok(position >= size)
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
        self.handles.create_event(name, manual_reset, initial_state)
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
        self.handles.mark_thread_exited(handle, exit_code)
    }

    pub fn guest_thread_id(&self, handle: u32) -> Option<u32> {
        self.handles.thread_id(handle)
    }

    pub fn guest_thread_exit_code(&self, handle: u32) -> Option<u32> {
        self.handles.thread_exit_code(handle)
    }

    pub fn process_exit_code(&self, handle: u32) -> Option<u32> {
        self.handles.process_exit_code(handle)
    }

    pub fn process_id(&self, handle: u32) -> Option<u32> {
        self.handles.process_id(handle)
    }

    pub fn terminate_process(&mut self, handle: u32, exit_code: u32) -> bool {
        self.handles.mark_process_exited(handle, exit_code)
    }

    pub fn suspend_thread(&mut self, handle: u32) -> Option<u32> {
        self.handles.suspend_thread(handle)
    }

    pub fn resume_thread(&mut self, handle: u32) -> Option<u32> {
        self.handles.resume_thread(handle)
    }

    pub fn thread_priority(&self, handle: u32) -> Option<i32> {
        self.handles.thread_priority(handle)
    }

    pub fn thread_priority_by_id(&self, thread_id: u32) -> i32 {
        self.handles.thread_priority_by_id(thread_id).unwrap_or(0)
    }

    pub fn set_thread_priority(&mut self, handle: u32, priority: i32) -> bool {
        self.handles.set_thread_priority(handle, priority)
    }

    pub fn guest_thread_start(&self, handle: u32) -> Option<(u32, u32, u32)> {
        self.handles.thread_start(handle)
    }

    pub fn set_event(&mut self, handle: u32) -> bool {
        self.handles.set_event(handle)
    }

    pub fn reset_event(&mut self, handle: u32) -> bool {
        self.handles.reset_event(handle)
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
        self.handles.release_semaphore(handle, release_count)
    }

    pub fn release_mutex(&mut self, handle: u32, thread_id: u32) -> bool {
        self.handles.release_mutex(handle, thread_id)
    }

    pub fn wait_for_single_object(&mut self, handle: u32, timeout_ms: u32, thread_id: u32) -> u32 {
        self.scheduler
            .record_wait_attempt(SchedulerWaitKind::Single, 1, timeout_ms);
        let result = match self
            .handles
            .wait_for_single_object(handle, timeout_ms, thread_id)
        {
            WaitResult::Object0 => WAIT_OBJECT_0,
            WaitResult::Timeout => WAIT_TIMEOUT,
            WaitResult::Failed => WAIT_FAILED,
        };
        self.scheduler
            .record_wait_result(wait_result_to_wake_reason(result));
        result
    }

    pub fn record_blocked_single_wait(&mut self, timeout_ms: u32) {
        self.scheduler
            .record_wait_attempt(SchedulerWaitKind::Single, 1, timeout_ms);
        self.scheduler.record_blocked_wait();
    }

    pub fn record_resumed_single_wait(&mut self, result: u32) {
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
        match self
            .handles
            .wait_for_single_object(handle, timeout_ms, thread_id)
        {
            WaitResult::Object0 => WAIT_OBJECT_0,
            WaitResult::Timeout => WAIT_TIMEOUT,
            WaitResult::Failed => WAIT_FAILED,
        }
    }

    pub fn is_wait_ready(&self, handle: u32, thread_id: u32) -> Option<bool> {
        self.handles.is_wait_ready(handle, thread_id)
    }

    pub fn wait_for_multiple_objects(
        &mut self,
        handles: &[u32],
        wait_all: bool,
        timeout_ms: u32,
        thread_id: u32,
    ) -> u32 {
        let result = if handles.is_empty() || wait_all {
            WAIT_FAILED
        } else if let Some((index, handle)) = handles
            .iter()
            .enumerate()
            .find(|(_, handle)| self.handles.is_wait_ready(**handle, thread_id) == Some(true))
        {
            let _ = self.handles.wait_for_single_object(*handle, 0, thread_id);
            WAIT_OBJECT_0 + index as u32
        } else if handles
            .iter()
            .any(|handle| self.handles.is_wait_ready(*handle, thread_id).is_none())
        {
            WAIT_FAILED
        } else {
            WAIT_TIMEOUT
        };
        self.record_multiple_wait_attempt(handles.len() as u32, timeout_ms, result);
        result
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
        let hwnd = self.gwe.create_window_ex_with_process_and_rect(
            thread_id,
            self.current_process_id,
            class_name,
            title,
            parent,
            id,
            style,
            ex_style,
            rect,
        );
        self.handles.insert(KernelObject::Window(hwnd));
        self.post_window_rect_messages(
            hwnd,
            Some(Rect::default()),
            self.gwe.get_window_rect(hwnd),
            HWND_TOP,
            0,
        );
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
        let was_visible = self.gwe.is_window_visible(hwnd);
        let previous = self.gwe.show_window(hwnd, visible);
        if was_visible != visible {
            self.post_window_message(hwnd, WM_SHOWWINDOW, u32::from(visible), 0);
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
        if self.gwe.update_rect(hwnd).is_some() {
            let _ = self.send_message_w(hwnd, crate::ce::gwe::WM_PAINT, 0, 0);
        }
        true
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
        let was_visible = self.gwe.is_window_visible(hwnd);
        let moved = self
            .gwe
            .set_window_pos(hwnd, insert_after, x, y, width, height, flags);
        if moved {
            let after = self.gwe.get_window_rect(hwnd);
            let is_visible = self.gwe.is_window_visible(hwnd);
            self.post_window_visibility_message(hwnd, was_visible, is_visible);
            self.post_window_rect_messages(
                hwnd,
                before,
                after,
                insert_after.unwrap_or(HWND_TOP),
                flags,
            );
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
        if !self.gwe.is_window(hwnd) {
            return None;
        }
        let was_enabled = self.gwe.enable_window(hwnd, enabled);
        if was_enabled != enabled {
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
            self.post_window_rect_messages(hwnd, before, self.gwe.get_window_rect(hwnd), 0, 0);
        }
        moved
    }

    pub fn set_focus(&mut self, hwnd: Option<u32>) -> Option<u32> {
        if hwnd.is_some_and(|hwnd| !self.gwe.is_window(hwnd)) {
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
        if hwnd.is_some_and(|hwnd| !self.gwe.is_window(hwnd)) {
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

    fn clear_focus_and_activation_within(&mut self, hwnd: u32) {
        if self.gwe.focus_is_within(hwnd) {
            let _ = self.set_focus(None);
        }
        if self.gwe.active_window_is_within(hwnd) {
            let _ = self.activate_window(None);
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
        self.pump_timers_to_gwe(thread_id);
        self.drain_remote_input_to_thread_window(thread_id, None);
        self.gwe.get_message(thread_id)
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
            HWND_BROADCAST => self
                .gwe
                .post_broadcast_message(msg, wparam, lparam, time_ms),
            0 => {
                self.gwe
                    .post_thread_message(thread_id, msg, wparam, lparam, time_ms);
                true
            }
            hwnd => self
                .gwe
                .post_message_for_window(hwnd, Message::new(hwnd, msg, wparam, lparam, time_ms)),
        }
    }

    pub fn post_thread_message_w(
        &mut self,
        target_thread_id: u32,
        msg: u32,
        wparam: u32,
        lparam: u32,
    ) -> bool {
        self.gwe.post_thread_message(
            target_thread_id,
            msg,
            wparam,
            lparam,
            self.timers.tick_count(),
        );
        true
    }

    pub fn send_message_w(&mut self, hwnd: u32, msg: u32, wparam: u32, lparam: u32) -> Option<u32> {
        let target_thread = self.gwe.window(hwnd).map(|window| window.thread_id)?;
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
        self.gwe.queue_send_message_for_window(
            Some(caller_thread_id),
            hwnd,
            Message::new(hwnd, msg, wparam, lparam, self.timers.tick_count()),
            SMF_NULL,
            timeout_ms,
        )
    }

    pub fn take_completed_send_message_result(&mut self, send_id: u64) -> Option<u32> {
        self.gwe.take_completed_sent_message_result(send_id)
    }

    pub fn destroy_window(&mut self, hwnd: u32) -> bool {
        let Some(targets) = self.gwe.window_and_descendants(hwnd) else {
            return false;
        };
        for target in targets.iter().rev().copied() {
            if self
                .gwe
                .window(target)
                .is_some_and(|window| !window.destroyed && !window.destroy_message_sent)
            {
                let _ = self.send_message_w(target, crate::ce::gwe::WM_DESTROY, 0, 0);
            }
        }
        self.gwe.destroy_window(hwnd, self.timers.tick_count())
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
            return self
                .gwe
                .post_broadcast_message(msg, wparam, lparam, self.timers.tick_count());
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
        self.gwe.queue_sent_message_for_window(
            hwnd,
            Message::new(hwnd, msg, wparam, lparam, self.timers.tick_count()),
        )
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
                .filter(|thread_id| self.gwe.in_send_message(*thread_id))
        } else {
            None
        }
        .or_else(|| (thread_id != 0 && self.gwe.in_send_message(thread_id)).then_some(thread_id));
        let result = if sent_context_thread.is_some() {
            self.gwe
                .send_message(message.hwnd, message.msg, message.wparam, message.lparam)
        } else {
            self.send_message_w(message.hwnd, message.msg, message.wparam, message.lparam)
        }
        .unwrap_or(0);
        if let Some(sent_context_thread) = sent_context_thread {
            self.gwe
                .complete_active_sent_message(sent_context_thread, result);
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

    pub fn set_timer(
        &mut self,
        hwnd: Option<u32>,
        requested_id: Option<u32>,
        period_ms: u32,
    ) -> u32 {
        self.timers
            .set_timer(hwnd, requested_id, period_ms, crate::ce::gwe::WM_TIMER)
    }

    pub fn kill_timer(&mut self, id: u32) -> bool {
        self.timers.kill_timer(id)
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
        self.remote.dispatch_control_message(message, gps_target)
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
                continue;
            };
            let client = self.gwe.screen_to_client(target, point).unwrap_or(point);
            let wparam = if event.message == WM_LBUTTONDOWN || event.message == WM_MOUSEMOVE {
                1
            } else {
                0
            };
            self.gwe.post_message(
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
            posted += 1;
        }

        let key_target = hwnd
            .or_else(|| self.gwe.get_capture())
            .or_else(|| self.gwe.get_active_window());
        for event in key_events {
            let Some(key_target) = key_target.filter(|hwnd| self.gwe.is_window(*hwnd)) else {
                continue;
            };
            self.gwe.post_message(
                thread_id,
                Message::new(key_target, event.message, event.vk, 1, time_ms)
                    .with_source(crate::ce::gwe::MSGSRC_HARDWARE_KEYBOARD),
            );
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

    fn post_window_rect_messages(
        &mut self,
        hwnd: u32,
        before: Option<Rect>,
        after: Option<Rect>,
        insert_after: u32,
        flags: u32,
    ) {
        let (Some(before), Some(after)) = (before, after) else {
            return;
        };
        if before == after {
            return;
        }
        let lparam = self
            .gwe
            .window_pos_for_rect(hwnd, after, insert_after, flags)
            .map(|payload| self.queue_window_pos_payload(payload))
            .unwrap_or(0);
        self.post_window_message(hwnd, WM_WINDOWPOSCHANGED, 0, lparam);
        if before.left != after.left || before.top != after.top {
            self.post_window_message(hwnd, WM_MOVE, 0, make_lparam_i16(after.left, after.top));
        }
        if before.width() != after.width() || before.height() != after.height() {
            self.post_window_message(
                hwnd,
                WM_SIZE,
                0,
                make_lparam_i16(after.width(), after.height()),
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
        self.gwe.post_message(window.thread_id, message);
    }
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

fn is_file_open_trace(op: &str) -> bool {
    matches!(op, "CreateFileW" | "CreateFileWArg" | "FindFirstFileW")
}

fn make_lparam_i16(low: i32, high: i32) -> u32 {
    ((high as u32) & 0xffff) << 16 | ((low as u32) & 0xffff)
}
