use crate::{
    ce::{
        audio::{AudioSystem, MmResult, WaveBuffer, WaveFormat},
        cemath::CeMath,
        devices::{DeviceIoControlResult, DeviceNamespace},
        file::{FileIoResult, GENERIC_READ, GENERIC_WRITE, HostFileSystem, OPEN_EXISTING},
        gwe::{Gwe, Message},
        object::{FileObject, HandleTable, KernelObject, WaitResult},
        registry::Registry,
        remote::{CeRemote, RemoteStatus, WM_LBUTTONDOWN, WM_MOUSEMOVE, make_lparam},
        timer::{TimerSystem, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT},
    },
    config::RuntimeConfig,
    error::Result,
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
        }
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
        if let KernelObject::File(file) = object {
            self.files.close(file.file_id)?;
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
        let hwnd = self
            .gwe
            .create_window_ex(thread_id, class_name, title, parent, id, style, ex_style);
        self.handles.insert(KernelObject::Window(hwnd));
        hwnd
    }

    pub fn get_message_w(&mut self, thread_id: u32) -> Option<Message> {
        self.pump_timers_to_gwe(thread_id);
        self.gwe.get_message(thread_id)
    }

    pub fn post_message_w(&mut self, hwnd: u32, msg: u32, wparam: u32, lparam: u32) -> bool {
        let time_ms = self.timers.tick_count();
        self.gwe
            .post_message_for_window(hwnd, Message::new(hwnd, msg, wparam, lparam, time_ms))
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
}
