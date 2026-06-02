use crate::{
    ce::{
        audio::{AudioSystem, MmResult, WaveBuffer, WaveFormat},
        cemath::CeMath,
        com::ComSystem,
        devices::{DeviceIoControlResult, DeviceNamespace},
        file::{
            FileIoResult, FindData, GENERIC_READ, GENERIC_WRITE, HostFileSystem, OPEN_EXISTING,
        },
        gwe::{
            Gwe, HWND_BROADCAST, Message, Rect, WM_MOVE, WM_SHOWWINDOW, WM_SIZE,
            WM_WINDOWPOSCHANGED,
        },
        memory::MemorySystem,
        object::{FileObject, FindFileObject, HandleTable, KernelObject, WaitResult},
        registry::Registry,
        remote::{CeRemote, RemoteStatus, WM_LBUTTONDOWN, WM_MOUSEMOVE, make_lparam},
        resource::ResourceSystem,
        thread::ThreadSystem,
        timer::{TimerSystem, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT},
    },
    config::RuntimeConfig,
    error::Result,
};

use std::path::PathBuf;

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
    pub resources: ResourceSystem,
    pub com: ComSystem,
    pub memory: MemorySystem,
    process_module_base: u32,
    process_module_path: String,
    process_module_host_path: Option<PathBuf>,
    process_command_line: String,
    pending_process_launches: Vec<PendingProcessLaunch>,
    next_process_id: u32,
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

impl CeKernel {
    pub fn boot(config: RuntimeConfig) -> Self {
        Self {
            registry: Registry::from_dump(config.registry),
            devices: DeviceNamespace::from_config(config.devices),
            files: HostFileSystem::new("."),
            handles: HandleTable::default(),
            gwe: Gwe::default(),
            audio: AudioSystem::default(),
            math: CeMath,
            timers: TimerSystem::default(),
            remote: CeRemote::default(),
            threads: ThreadSystem::default(),
            resources: ResourceSystem::default(),
            com: ComSystem::default(),
            memory: MemorySystem::default(),
            process_module_base: 0,
            process_module_path: "\\FakeCE\\process.exe".to_owned(),
            process_module_host_path: None,
            process_command_line: String::new(),
            pending_process_launches: Vec::new(),
            next_process_id: 0x42,
        }
    }

    pub fn set_process_module_base(&mut self, base: u32) {
        self.process_module_base = base;
    }

    pub fn process_module_base(&self) -> u32 {
        self.process_module_base
    }

    pub fn set_process_module_path(&mut self, path: impl Into<String>) {
        self.process_module_path = path.into();
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

    pub fn set_file_root(&mut self, root: impl Into<std::path::PathBuf>) {
        self.files = HostFileSystem::new(root);
    }

    pub fn mount_guest_root(&mut self, guest_root: &str, host_root: impl Into<std::path::PathBuf>) {
        self.files.mount_guest_root(guest_root, host_root);
    }

    pub fn create_file_w(
        &mut self,
        path: &str,
        desired_access: u32,
        creation_disposition: u32,
    ) -> Result<u32> {
        if let Ok(session) = self.devices.open(path) {
            return Ok(self.handles.insert(KernelObject::Device(session)));
        }

        let file_id = self
            .files
            .create_file_w(path, desired_access, creation_disposition)?;
        Ok(self.handles.insert(KernelObject::File(FileObject {
            guest_path: path.to_owned(),
            file_id,
        })))
    }

    pub fn open_existing_readonly(&mut self, path: &str) -> Result<u32> {
        self.create_file_w(path, GENERIC_READ, OPEN_EXISTING)
    }

    pub fn open_existing_readwrite(&mut self, path: &str) -> Result<u32> {
        self.create_file_w(path, GENERIC_READ | GENERIC_WRITE, OPEN_EXISTING)
    }

    pub fn read_file(&mut self, handle: u32, requested: u32) -> Result<Vec<u8>> {
        match self.handles.get_mut(handle)? {
            KernelObject::File(file) => self.files.read_file(file.file_id, requested),
            KernelObject::Device(device) => Ok(device.read_file(requested)),
            _ => Ok(Vec::new()),
        }
    }

    pub fn read_file_at(&self, file_id: u32, offset: usize, requested: usize) -> Result<Vec<u8>> {
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
        match self.handles.get_mut(handle)? {
            KernelObject::File(file) => self.files.write_file(file.file_id, bytes),
            KernelObject::Device(device) => Ok(FileIoResult {
                success: true,
                bytes_transferred: device.write_file(bytes),
            }),
            _ => Ok(FileIoResult {
                success: false,
                bytes_transferred: 0,
            }),
        }
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
        let KernelObject::File(file) = self.handles.get(handle)? else {
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
            return Err(crate::error::Error::InvalidArgument(
                "negative file pointer".to_owned(),
            ));
        }
        self.files.set_file_pointer(file_id, position as usize)
    }

    pub fn get_file_size(&self, handle: u32) -> Result<usize> {
        let KernelObject::File(file) = self.handles.get(handle)? else {
            return Err(crate::error::Error::InvalidHandle(handle));
        };
        self.files.file_size(file.file_id)
    }

    pub fn flush_file_buffers(&mut self, handle: u32) -> Result<bool> {
        let KernelObject::File(file) = self.handles.get(handle)? else {
            return Err(crate::error::Error::InvalidHandle(handle));
        };
        self.files.flush(file.file_id)?;
        Ok(true)
    }

    pub fn find_first_file_w(&mut self, pattern: &str) -> Result<(u32, FindData)> {
        let (find_id, data) = self.files.find_first_file_w(pattern)?;
        let handle = self.handles.insert(KernelObject::FindFile(FindFileObject {
            guest_pattern: pattern.to_owned(),
            find_id,
        }));
        Ok((handle, data))
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

    pub fn suspend_thread(&mut self, handle: u32) -> Option<u32> {
        self.handles.suspend_thread(handle)
    }

    pub fn resume_thread(&mut self, handle: u32) -> Option<u32> {
        self.handles.resume_thread(handle)
    }

    pub fn thread_priority(&self, handle: u32) -> Option<i32> {
        self.handles.thread_priority(handle)
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

    pub fn release_mutex(&mut self, handle: u32, thread_id: u32) -> bool {
        self.handles.release_mutex(handle, thread_id)
    }

    pub fn wait_for_single_object(&mut self, handle: u32, timeout_ms: u32, thread_id: u32) -> u32 {
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
        if handles.is_empty() {
            return WAIT_FAILED;
        }
        if wait_all {
            if handles
                .iter()
                .all(|handle| self.handles.is_wait_ready(*handle, thread_id) == Some(true))
            {
                for handle in handles {
                    let _ = self.handles.wait_for_single_object(*handle, 0, thread_id);
                }
                WAIT_OBJECT_0
            } else if timeout_ms == 0 {
                WAIT_TIMEOUT
            } else {
                WAIT_TIMEOUT
            }
        } else {
            for (index, handle) in handles.iter().enumerate() {
                if self.handles.is_wait_ready(*handle, thread_id) == Some(true) {
                    let _ = self.handles.wait_for_single_object(*handle, 0, thread_id);
                    return WAIT_OBJECT_0 + index as u32;
                }
            }
            if handles
                .iter()
                .any(|handle| self.handles.is_wait_ready(*handle, thread_id).is_none())
            {
                WAIT_FAILED
            } else {
                WAIT_TIMEOUT
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
        let hwnd = self.gwe.create_window_ex_with_rect(
            thread_id, class_name, title, parent, id, style, ex_style, rect,
        );
        self.handles.insert(KernelObject::Window(hwnd));
        hwnd
    }

    pub fn show_window(&mut self, hwnd: u32, visible: bool) -> bool {
        if !self.gwe.is_window(hwnd) {
            return false;
        }
        let was_visible = self.gwe.is_window_visible(hwnd);
        let previous = self.gwe.show_window(hwnd, visible);
        if was_visible != visible {
            self.post_window_message(hwnd, WM_SHOWWINDOW, u32::from(visible), 0);
        }
        previous
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
            self.post_window_rect_messages(hwnd, before, after);
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
            self.post_window_rect_messages(hwnd, before, self.gwe.get_window_rect(hwnd));
        }
        moved
    }

    pub fn get_message_w(&mut self, thread_id: u32) -> Option<Message> {
        self.pump_timers_to_gwe(thread_id);
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

    pub fn send_message_w(&mut self, hwnd: u32, msg: u32, wparam: u32, lparam: u32) -> Option<u32> {
        self.gwe.send_message(hwnd, msg, wparam, lparam)
    }

    pub fn dispatch_message_w(&mut self, message: Message) -> u32 {
        if message.msg == crate::ce::gwe::WM_QUIT {
            return message.wparam;
        }
        self.send_message_w(message.hwnd, message.msg, message.wparam, message.lparam)
            .unwrap_or(0)
    }

    pub fn message_pump_step(&mut self, thread_id: u32) -> MessagePumpResult {
        let Some(message) = self.get_message_w(thread_id) else {
            return MessagePumpResult::Idle;
        };
        if message.msg == crate::ce::gwe::WM_QUIT {
            return MessagePumpResult::Quit(message.wparam);
        }
        MessagePumpResult::Dispatched(self.dispatch_message_w(message))
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

        let time_ms = self.timers.tick_count();
        let touch_events = self.remote.drain_touch_events();
        let key_events = self.remote.drain_key_events();
        let mut posted = 0;

        for event in touch_events {
            let wparam = if event.message == WM_LBUTTONDOWN || event.message == WM_MOUSEMOVE {
                1
            } else {
                0
            };
            self.gwe.post_message(
                thread_id,
                Message::new(
                    hwnd,
                    event.message,
                    wparam,
                    make_lparam(event.x, event.y),
                    time_ms,
                ),
            );
            posted += 1;
        }

        for event in key_events {
            self.gwe.post_message(
                thread_id,
                Message::new(hwnd, event.message, event.vk, 1, time_ms),
            );
            posted += 1;
        }

        posted
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

    fn post_window_rect_messages(&mut self, hwnd: u32, before: Option<Rect>, after: Option<Rect>) {
        let (Some(before), Some(after)) = (before, after) else {
            return;
        };
        if before == after {
            return;
        }
        self.post_window_message(hwnd, WM_WINDOWPOSCHANGED, 0, 0);
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

    fn post_window_message(&mut self, hwnd: u32, msg: u32, wparam: u32, lparam: u32) {
        let Some(window) = self.gwe.window(hwnd) else {
            return;
        };
        let message = Message::new(hwnd, msg, wparam, lparam, self.timers.tick_count());
        self.gwe.post_message(window.thread_id, message);
    }
}

fn make_lparam_i16(low: i32, high: i32) -> u32 {
    ((high as u32) & 0xffff) << 16 | ((low as u32) & 0xffff)
}
