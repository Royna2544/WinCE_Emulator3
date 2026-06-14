mod accelerometer;
mod i2c_bus;
mod light_sensor;
mod magnetometer;
mod nand_uuid;
mod pic_controller;

use std::collections::BTreeMap;
#[cfg(windows)]
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    os::windows::io::AsRawHandle,
    sync::{Arc, Mutex},
};

use serde::Deserialize;
use serde_json::Value;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceConfigFile {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub defaults: DeviceDefaults,
    #[serde(default)]
    pub devices: Vec<DeviceConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceDefaults {
    #[serde(default = "default_baud")]
    pub baud: u32,
    #[serde(default = "default_mode")]
    pub mode: String,
}

impl Default for DeviceDefaults {
    fn default() -> Self {
        Self {
            baud: default_baud(),
            mode: default_mode(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceConfig {
    pub guest: String,
    #[serde(rename = "type")]
    pub kind: DeviceKind,
    pub backend: DeviceBackend,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub remote_gps: bool,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind {
    Serial,
    IoctlDevice,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceBackend {
    Win32Com,
    Stub,
    Accelerometer,
    I2cBus,
    LightSensor,
    Magnetometer,
    NandUuid,
    PicController,
}

#[derive(Debug, Clone)]
pub struct DeviceNamespace {
    devices: BTreeMap<String, DeviceConfig>,
    defaults: DeviceDefaults,
}

#[derive(Debug, Clone)]
pub struct DeviceSession {
    pub guest_name: String,
    pub kind: DeviceKind,
    pub backend: DeviceBackend,
    pub host: Option<String>,
    dcb: CommDcb,
    comm_timeouts: CommTimeouts,
    comm_mask: u32,
    rx: Vec<u8>,
    tx: Vec<u8>,
    runtime: DeviceRuntime,
    #[cfg(windows)]
    host_serial: Option<Win32ComPort>,
}

#[derive(Debug, Clone)]
enum DeviceRuntime {
    None,
    Accelerometer(accelerometer::Accelerometer),
    I2cBus(i2c_bus::I2cBus),
    LightSensor(light_sensor::LightSensor),
    Magnetometer(magnetometer::Magnetometer),
    NandUuid(nand_uuid::NandUuid),
    PicController(pic_controller::PicController),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceIoControlResult {
    pub success: bool,
    pub bytes_returned: u32,
    pub output: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CommTimeouts {
    pub read_interval_timeout: u32,
    pub read_total_timeout_multiplier: u32,
    pub read_total_timeout_constant: u32,
    pub write_total_timeout_multiplier: u32,
    pub write_total_timeout_constant: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommDcb {
    bytes: [u8; Self::SIZE],
}

impl Default for CommDcb {
    fn default() -> Self {
        let mut dcb = Self {
            bytes: [0; Self::SIZE],
        };
        dcb.set_u32(0, Self::SIZE as u32);
        dcb.set_u32(4, 9600);
        dcb.set_u32(8, 1);
        dcb.bytes[18] = 8;
        dcb.bytes[19] = 0;
        dcb.bytes[20] = 0;
        dcb
    }
}

impl CommDcb {
    pub const SIZE: usize = 28;

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        let mut dcb = Self {
            bytes: [0; Self::SIZE],
        };
        dcb.bytes.copy_from_slice(&bytes[..Self::SIZE]);
        Some(dcb)
    }

    pub fn bytes(&self) -> &[u8; Self::SIZE] {
        &self.bytes
    }

    fn set_u32(&mut self, offset: usize, value: u32) {
        self.bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}

pub const PURGE_TXCLEAR: u32 = 0x0004;
pub const PURGE_RXCLEAR: u32 = 0x0008;

impl DeviceNamespace {
    pub fn from_config(config: DeviceConfigFile) -> Self {
        let devices = config
            .devices
            .into_iter()
            .filter(|device| device.enabled)
            .map(|device| (normalize_device_name(&device.guest), device))
            .collect();

        Self {
            devices,
            defaults: config.defaults,
        }
    }

    pub fn open(&self, guest_name: &str) -> Result<DeviceSession> {
        let normalized = normalize_device_name(guest_name);
        let config = self.devices.get(&normalized).ok_or_else(|| {
            tracing::debug!(
                target: "ce.devices",
                device = guest_name,
                "device open: not configured"
            );
            Error::MissingDevice(guest_name.to_owned())
        })?;
        tracing::debug!(
            target: "ce.devices",
            device = guest_name,
            backend = ?config.backend,
            "device open"
        );

        #[cfg(windows)]
        let host_serial = open_host_serial_if_configured(config, &self.defaults);

        Ok(DeviceSession {
            guest_name: config.guest.clone(),
            kind: config.kind.clone(),
            backend: config.backend.clone(),
            host: config.host.clone(),
            dcb: CommDcb::default(),
            comm_timeouts: CommTimeouts::default(),
            comm_mask: 0,
            rx: Vec::new(),
            tx: Vec::new(),
            runtime: DeviceRuntime::from_config(config),
            #[cfg(windows)]
            host_serial,
        })
    }

    pub fn default_baud(&self) -> u32 {
        self.defaults.baud
    }

    pub fn default_mode(&self) -> &str {
        &self.defaults.mode
    }

    pub fn enabled_names(&self) -> Vec<String> {
        self.devices
            .values()
            .map(|device| device.guest.clone())
            .collect()
    }

    pub fn remote_gps_target(&self) -> Option<String> {
        let mut fallback = None;
        for device in self.devices.values() {
            if device.kind == DeviceKind::Serial && device.remote_gps {
                return Some(device.guest.clone());
            }
        }
        for device in self.devices.values() {
            if device.kind != DeviceKind::Serial {
                continue;
            }
            if device.backend == DeviceBackend::Win32Com {
                return Some(device.guest.clone());
            }
            fallback.get_or_insert_with(|| device.guest.clone());
        }
        fallback
    }
}

impl DeviceRuntime {
    fn from_config(config: &DeviceConfig) -> Self {
        match &config.backend {
            DeviceBackend::Accelerometer => {
                Self::Accelerometer(accelerometer::Accelerometer::new())
            }
            DeviceBackend::I2cBus => Self::I2cBus(i2c_bus::I2cBus::new_for_guest(&config.guest)),
            DeviceBackend::LightSensor => Self::LightSensor(light_sensor::LightSensor::new()),
            DeviceBackend::Magnetometer => Self::Magnetometer(magnetometer::Magnetometer::new()),
            DeviceBackend::NandUuid => Self::NandUuid(nand_uuid::NandUuid::new()),
            DeviceBackend::PicController => {
                Self::PicController(pic_controller::PicController::new())
            }
            _ => Self::None,
        }
    }
}

impl DeviceSession {
    pub fn is_serial(&self) -> bool {
        self.kind == DeviceKind::Serial
    }

    pub fn rx_len(&self) -> usize {
        self.rx.len()
    }

    pub fn accepts_remote_serial_target(&self, target: &str) -> bool {
        let normalized_target = normalize_device_name(target);
        normalize_device_name(&self.guest_name) == normalized_target
            || self
                .host
                .as_deref()
                .is_some_and(|host| normalize_device_name(host) == normalized_target)
    }

    pub fn read_file(&mut self, requested: u32) -> Vec<u8> {
        let count = (requested as usize).min(self.rx.len());
        self.rx.drain(..count).collect()
    }

    pub fn dcb(&self) -> CommDcb {
        self.dcb
    }

    pub fn set_dcb(&mut self, dcb: CommDcb) {
        self.dcb = dcb;
        #[cfg(windows)]
        if let Some(host) = &self.host_serial {
            host.configure_dcb(self.dcb);
        }
    }

    pub fn comm_timeouts(&self) -> CommTimeouts {
        self.comm_timeouts
    }

    pub fn set_comm_timeouts(&mut self, timeouts: CommTimeouts) {
        self.comm_timeouts = timeouts;
    }

    pub fn empty_read_timeout_ms(&self, requested: u32) -> Option<u32> {
        if !self.is_serial() {
            return Some(0);
        }
        let timeouts = self.comm_timeouts;
        if timeouts.read_interval_timeout == u32::MAX
            && timeouts.read_total_timeout_multiplier == 0
            && timeouts.read_total_timeout_constant == 0
        {
            return Some(0);
        }
        let total = u64::from(timeouts.read_total_timeout_constant).saturating_add(
            u64::from(timeouts.read_total_timeout_multiplier).saturating_mul(u64::from(requested)),
        );
        if total != 0 {
            return Some(total.min(u64::from(u32::MAX - 1)) as u32);
        }
        None
    }

    pub fn comm_mask(&self) -> u32 {
        self.comm_mask
    }

    pub fn set_comm_mask(&mut self, mask: u32) {
        self.comm_mask = mask;
    }

    pub fn write_file(&mut self, bytes: &[u8]) -> u32 {
        self.tx.extend_from_slice(bytes);
        #[cfg(windows)]
        if let Some(host) = &self.host_serial {
            let _ = host.write(bytes);
        }
        bytes.len() as u32
    }

    pub fn purge_comm(&mut self, flags: u32) {
        if flags & PURGE_RXCLEAR != 0 {
            self.rx.clear();
        }
        if flags & PURGE_TXCLEAR != 0 {
            self.tx.clear();
        }
    }

    pub fn queue_lengths(&self) -> (u32, u32) {
        (self.rx.len() as u32, self.tx.len() as u32)
    }

    pub fn device_io_control(
        &mut self,
        ioctl_code: u32,
        input: &[u8],
        output_capacity: u32,
    ) -> DeviceIoControlResult {
        self.device_io_control_with_output_buffer(
            ioctl_code,
            input,
            output_capacity,
            output_capacity > 0,
        )
    }

    pub fn device_io_control_with_output_buffer(
        &mut self,
        ioctl_code: u32,
        input: &[u8],
        output_capacity: u32,
        output_buffer_present: bool,
    ) -> DeviceIoControlResult {
        tracing::debug!(
            target: "ce.devices",
            device = self.guest_name.as_str(),
            ioctl = format_args!("0x{ioctl_code:08x}"),
            in_len = input.len(),
            input = %format_ioctl_bytes(input),
            out_cap = output_capacity,
            "DeviceIoControl"
        );
        let result = match &mut self.runtime {
            DeviceRuntime::Accelerometer(sensor) => {
                sensor.device_io_control(ioctl_code, input, output_capacity)
            }
            DeviceRuntime::I2cBus(bus) => bus.device_io_control(ioctl_code, input, output_capacity),
            DeviceRuntime::LightSensor(sensor) => {
                sensor.device_io_control(ioctl_code, input, output_capacity)
            }
            DeviceRuntime::Magnetometer(sensor) => {
                sensor.device_io_control(ioctl_code, input, output_capacity, output_buffer_present)
            }
            DeviceRuntime::NandUuid(device) => {
                device.device_io_control(ioctl_code, input, output_capacity)
            }
            DeviceRuntime::PicController(device) => {
                device.device_io_control(ioctl_code, input, output_capacity)
            }
            DeviceRuntime::None => DeviceIoControlResult {
                success: false,
                bytes_returned: 0,
                output: Vec::new(),
            },
        };
        tracing::debug!(
            target: "ce.devices",
            device = self.guest_name.as_str(),
            ioctl = format_args!("0x{ioctl_code:08x}"),
            success = result.success,
            bytes_returned = result.bytes_returned,
            output = %format_ioctl_bytes(&result.output),
            "DeviceIoControl result"
        );
        result
    }

    pub fn apply_remote_imu(&mut self, imu: &Value) {
        match &mut self.runtime {
            DeviceRuntime::Accelerometer(sensor) => {
                let x = scaled_imu_component(
                    imu,
                    &["ax", "accelX", "accelerometerX", "x"],
                    &["accel", "accelerometer"],
                    "x",
                    256.0,
                );
                let y = scaled_imu_component(
                    imu,
                    &["ay", "accelY", "accelerometerY", "y"],
                    &["accel", "accelerometer"],
                    "y",
                    256.0,
                );
                let z = scaled_imu_component(
                    imu,
                    &["az", "accelZ", "accelerometerZ", "z"],
                    &["accel", "accelerometer"],
                    "z",
                    256.0,
                );
                if let (Some(x), Some(y), Some(z)) = (x, y, z) {
                    sensor.set_axes(x, y, z);
                }
            }
            DeviceRuntime::Magnetometer(sensor) => {
                let x = scaled_imu_component(
                    imu,
                    &["mx", "magX", "magnetometerX"],
                    &["mag", "magnetometer"],
                    "x",
                    1.0,
                );
                let y = scaled_imu_component(
                    imu,
                    &["my", "magY", "magnetometerY"],
                    &["mag", "magnetometer"],
                    "y",
                    1.0,
                );
                let z = scaled_imu_component(
                    imu,
                    &["mz", "magZ", "magnetometerZ"],
                    &["mag", "magnetometer"],
                    "z",
                    1.0,
                );
                if let (Some(x), Some(y), Some(z)) = (x, y, z) {
                    sensor.set_field(x, y, z);
                }
            }
            _ => {}
        }
    }

    pub fn enqueue_rx(&mut self, bytes: &[u8]) {
        self.rx.extend_from_slice(bytes);
    }

    pub fn poll_host_serial(&mut self, max_bytes: usize) -> usize {
        #[cfg(windows)]
        {
            let Some(host) = &self.host_serial else {
                return 0;
            };
            let bytes = host.read_available(max_bytes);
            let count = bytes.len();
            self.rx.extend_from_slice(&bytes);
            count
        }
        #[cfg(not(windows))]
        {
            let _ = max_bytes;
            0
        }
    }

    pub fn tx_bytes(&self) -> &[u8] {
        &self.tx
    }
}

fn scaled_imu_component(
    imu: &Value,
    names: &[&str],
    containers: &[&str],
    axis: &str,
    scale: f64,
) -> Option<i16> {
    names
        .iter()
        .find_map(|name| imu.get(*name).and_then(Value::as_f64))
        .or_else(|| {
            containers.iter().find_map(|container| {
                imu.get(*container)
                    .and_then(|value| value.get(axis))
                    .and_then(Value::as_f64)
            })
        })
        .map(|value| clamp_i16((value * scale).round()))
}

fn clamp_i16(value: f64) -> i16 {
    value.clamp(f64::from(i16::MIN), f64::from(i16::MAX)) as i16
}

#[cfg(windows)]
#[derive(Clone)]
struct Win32ComPort {
    inner: Arc<Mutex<Win32ComPortInner>>,
}

#[cfg(windows)]
struct Win32ComPortInner {
    host_name: String,
    file: std::fs::File,
}

#[cfg(windows)]
impl std::fmt::Debug for Win32ComPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let host_name = self
            .inner
            .lock()
            .ok()
            .map(|inner| inner.host_name.clone())
            .unwrap_or_else(|| "<locked>".to_owned());
        f.debug_struct("Win32ComPort")
            .field("host_name", &host_name)
            .finish_non_exhaustive()
    }
}

#[cfg(windows)]
fn open_host_serial_if_configured(
    config: &DeviceConfig,
    defaults: &DeviceDefaults,
) -> Option<Win32ComPort> {
    if config.kind != DeviceKind::Serial || config.backend != DeviceBackend::Win32Com {
        return None;
    }
    let host = config.host.as_deref()?.trim();
    if host.is_empty() {
        return None;
    }
    Win32ComPort::open(host, defaults.baud)
}

#[cfg(windows)]
impl Win32ComPort {
    fn open(host_name: &str, baud: u32) -> Option<Self> {
        let path = win32_com_path(host_name);
        let file = OpenOptions::new().read(true).write(true).open(&path).ok()?;
        let port = Self {
            inner: Arc::new(Mutex::new(Win32ComPortInner {
                host_name: host_name.to_owned(),
                file,
            })),
        };
        port.configure_timeouts();
        port.configure_line(baud, 8, 0, 0);
        Some(port)
    }

    fn read_available(&self, max_bytes: usize) -> Vec<u8> {
        if max_bytes == 0 {
            return Vec::new();
        }
        let mut buffer = vec![0; max_bytes.min(4096)];
        let Ok(mut inner) = self.inner.lock() else {
            return Vec::new();
        };
        match inner.file.read(&mut buffer) {
            Ok(count) => {
                buffer.truncate(count);
                buffer
            }
            Err(_) => Vec::new(),
        }
    }

    fn write(&self, bytes: &[u8]) -> usize {
        let Ok(mut inner) = self.inner.lock() else {
            return 0;
        };
        inner.file.write(bytes).unwrap_or_default()
    }

    fn configure_dcb(&self, dcb: CommDcb) {
        let baud = u32::from_le_bytes(dcb.bytes()[4..8].try_into().unwrap_or([0; 4]));
        let byte_size = dcb.bytes()[18];
        let parity = dcb.bytes()[19];
        let stop_bits = dcb.bytes()[20];
        self.configure_line(baud, byte_size, parity, stop_bits);
    }

    fn configure_timeouts(&self) {
        use windows::Win32::Devices::Communication::{COMMTIMEOUTS, SetCommTimeouts};
        use windows::Win32::Foundation::HANDLE;

        let Ok(inner) = self.inner.lock() else {
            return;
        };
        let handle = HANDLE(inner.file.as_raw_handle());
        let timeouts = COMMTIMEOUTS {
            ReadIntervalTimeout: u32::MAX,
            ReadTotalTimeoutMultiplier: 0,
            ReadTotalTimeoutConstant: 0,
            WriteTotalTimeoutMultiplier: 0,
            WriteTotalTimeoutConstant: 100,
        };
        let _ = unsafe { SetCommTimeouts(handle, &timeouts) };
    }

    fn configure_line(&self, baud: u32, byte_size: u8, parity: u8, stop_bits: u8) {
        use windows::Win32::Devices::Communication::{DCB, GetCommState, SetCommState};
        use windows::Win32::Foundation::HANDLE;

        let Ok(inner) = self.inner.lock() else {
            return;
        };
        let handle = HANDLE(inner.file.as_raw_handle());
        let mut host_dcb = DCB::default();
        host_dcb.DCBlength = std::mem::size_of::<DCB>() as u32;
        if unsafe { GetCommState(handle, &mut host_dcb) }.is_err() {
            return;
        }
        if baud != 0 {
            host_dcb.BaudRate = baud;
        }
        if byte_size != 0 {
            host_dcb.ByteSize = byte_size;
        }
        host_dcb.Parity = windows::Win32::Devices::Communication::DCB_PARITY(parity);
        host_dcb.StopBits = windows::Win32::Devices::Communication::DCB_STOP_BITS(stop_bits);
        let _ = unsafe { SetCommState(handle, &host_dcb) };
    }
}

#[cfg(windows)]
fn win32_com_path(host_name: &str) -> String {
    let trimmed = host_name.trim();
    if trimmed.starts_with(r"\\.\") {
        trimmed.to_owned()
    } else {
        format!(r"\\.\{}", trimmed.trim_end_matches(':'))
    }
}

fn normalize_device_name(name: &str) -> String {
    name.trim()
        .trim_start_matches("\\\\.\\")
        .trim_end_matches(':')
        .to_ascii_uppercase()
}

fn default_baud() -> u32 {
    9600
}

fn default_mode() -> String {
    "8N1".to_owned()
}

fn format_ioctl_bytes(bytes: &[u8]) -> String {
    const MAX_BYTES: usize = 16;
    let mut out = String::new();
    for (index, byte) in bytes.iter().copied().take(MAX_BYTES).enumerate() {
        if index > 0 {
            out.push(' ');
        }
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    if bytes.len() > MAX_BYTES {
        out.push_str(" ...");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_accepts_ce_colon_suffix() {
        let namespace = DeviceNamespace::from_config(DeviceConfigFile {
            version: 1,
            defaults: DeviceDefaults::default(),
            devices: vec![DeviceConfig {
                guest: "COM7:".to_owned(),
                kind: DeviceKind::Serial,
                backend: DeviceBackend::Stub,
                host: None,
                remote_gps: false,
                enabled: true,
                note: None,
            }],
        });

        assert_eq!(namespace.open("com7").unwrap().guest_name, "COM7:");
    }

    #[test]
    fn remote_gps_target_uses_guest_name_for_win32_backed_serial() {
        let namespace = DeviceNamespace::from_config(DeviceConfigFile {
            version: 1,
            defaults: DeviceDefaults::default(),
            devices: vec![DeviceConfig {
                guest: "COM7:".to_owned(),
                kind: DeviceKind::Serial,
                backend: DeviceBackend::Win32Com,
                host: Some("COM21".to_owned()),
                remote_gps: false,
                enabled: true,
                note: None,
            }],
        });

        assert_eq!(namespace.remote_gps_target().as_deref(), Some("COM7:"));
    }

    #[test]
    fn remote_gps_target_prefers_explicit_serial_flag() {
        let namespace = DeviceNamespace::from_config(DeviceConfigFile {
            version: 1,
            defaults: DeviceDefaults::default(),
            devices: vec![
                DeviceConfig {
                    guest: "COM1:".to_owned(),
                    kind: DeviceKind::Serial,
                    backend: DeviceBackend::Win32Com,
                    host: Some("COM21".to_owned()),
                    remote_gps: false,
                    enabled: true,
                    note: None,
                },
                DeviceConfig {
                    guest: "COM7:".to_owned(),
                    kind: DeviceKind::Serial,
                    backend: DeviceBackend::Stub,
                    host: None,
                    remote_gps: true,
                    enabled: true,
                    note: None,
                },
            ],
        });

        assert_eq!(namespace.remote_gps_target().as_deref(), Some("COM7:"));
    }

    #[test]
    fn win32_com_without_host_still_opens_as_serial_session() {
        let namespace = DeviceNamespace::from_config(DeviceConfigFile {
            version: 1,
            defaults: DeviceDefaults::default(),
            devices: vec![DeviceConfig {
                guest: "COM7:".to_owned(),
                kind: DeviceKind::Serial,
                backend: DeviceBackend::Win32Com,
                host: None,
                remote_gps: false,
                enabled: true,
                note: None,
            }],
        });

        let mut session = namespace.open("COM7:").unwrap();
        assert!(session.is_serial());
        assert_eq!(session.read_file(64), Vec::<u8>::new());
        assert_eq!(session.write_file(b"$PUBX"), 5);
    }

    #[cfg(windows)]
    #[test]
    fn win32_com_path_uses_device_prefix() {
        assert_eq!(win32_com_path("COM21"), r"\\.\COM21");
        assert_eq!(win32_com_path(r"\\.\COM21"), r"\\.\COM21");
        assert_eq!(win32_com_path("COM7:"), r"\\.\COM7");
    }

    #[test]
    fn sensor_backends_handle_decoded_ioctl_contracts() {
        let namespace = DeviceNamespace::from_config(DeviceConfigFile {
            version: 1,
            defaults: DeviceDefaults::default(),
            devices: vec![
                DeviceConfig {
                    guest: "SMB1:".to_owned(),
                    kind: DeviceKind::IoctlDevice,
                    backend: DeviceBackend::Accelerometer,
                    host: None,
                    remote_gps: false,
                    enabled: true,
                    note: None,
                },
                DeviceConfig {
                    guest: "LSD1:".to_owned(),
                    kind: DeviceKind::IoctlDevice,
                    backend: DeviceBackend::LightSensor,
                    host: None,
                    remote_gps: false,
                    enabled: true,
                    note: None,
                },
                DeviceConfig {
                    guest: "MFS1:".to_owned(),
                    kind: DeviceKind::IoctlDevice,
                    backend: DeviceBackend::Magnetometer,
                    host: None,
                    remote_gps: false,
                    enabled: true,
                    note: None,
                },
                DeviceConfig {
                    guest: "I2C2:".to_owned(),
                    kind: DeviceKind::IoctlDevice,
                    backend: DeviceBackend::I2cBus,
                    host: None,
                    remote_gps: false,
                    enabled: true,
                    note: None,
                },
                DeviceConfig {
                    guest: "I2C3:".to_owned(),
                    kind: DeviceKind::IoctlDevice,
                    backend: DeviceBackend::I2cBus,
                    host: None,
                    remote_gps: false,
                    enabled: true,
                    note: None,
                },
                DeviceConfig {
                    guest: "I2C4:".to_owned(),
                    kind: DeviceKind::IoctlDevice,
                    backend: DeviceBackend::I2cBus,
                    host: None,
                    remote_gps: false,
                    enabled: true,
                    note: None,
                },
            ],
        });

        let mut accel = namespace.open("SMB1:").unwrap();
        let xyz = accel.device_io_control(accelerometer::IOCTL_SMB380_READ_ACCEL_XYZT, &[], 6);
        assert!(xyz.success);
        assert_eq!(xyz.bytes_returned, 6);
        assert_eq!(&xyz.output[4..6], &256i16.to_le_bytes());

        let range_set = accel.device_io_control(accelerometer::IOCTL_SMB380_SET_RANGE, &[3], 0);
        assert!(range_set.success);
        let range_get = accel.device_io_control(accelerometer::IOCTL_SMB380_GET_RANGE, &[], 1);
        assert_eq!(range_get.output, vec![3]);

        let mut light = namespace.open("LSD1:").unwrap();
        assert!(
            light
                .device_io_control(light_sensor::IOCTL_LSD_SET_CONTROL, &[0x7f], 0)
                .success
        );
        let lux = light.device_io_control(light_sensor::IOCTL_LSD_READ_LUX, &[], 2);
        assert!(lux.success);
        assert_eq!(lux.bytes_returned, 2);

        let mut mag = namespace.open("MFS1:").unwrap();
        assert!(
            mag.device_io_control(
                magnetometer::IOCTL_MFS_WRITE_REGISTERS,
                &[0, 0x40, 0, 0xaa],
                0
            )
            .success
        );
        let mag_read =
            mag.device_io_control(magnetometer::IOCTL_MFS_READ_REGISTERS, &[0, 0x40, 1, 0], 1);
        assert_eq!(mag_read.output, vec![0xaa]);
        let mag_no_output_buffer =
            mag.device_io_control(magnetometer::IOCTL_MFS_READ_REGISTERS, &[0, 0x40, 1, 0], 0);
        assert!(!mag_no_output_buffer.success);
        let mag_zero_capacity_with_output_buffer = mag.device_io_control_with_output_buffer(
            magnetometer::IOCTL_MFS_READ_REGISTERS,
            &[0, 0x40, 1, 0],
            0,
            true,
        );
        assert!(mag_zero_capacity_with_output_buffer.success);
        assert_eq!(mag_zero_capacity_with_output_buffer.bytes_returned, 1);
        assert_eq!(mag_zero_capacity_with_output_buffer.output, vec![0xaa]);

        let mut i2c = namespace.open("I2C2:").unwrap();
        assert!(
            i2c.device_io_control(i2c_bus::IOCTL_I2C_WRITE, &[0x10, 0x33], 0)
                .success
        );
        let i2c_read = i2c.device_io_control(i2c_bus::IOCTL_I2C_READ, &[0x10], 1);
        assert_eq!(i2c_read.output, vec![0x33]);
        assert!(
            !i2c.device_io_control(i2c_bus::IOCTL_I2C_GIO_I2C2_TRANSFER, &[0x10, 0x44], 1)
                .success
        );

        let mut i2c3 = namespace.open("I2C3:").unwrap();
        assert!(
            i2c3.device_io_control(i2c_bus::IOCTL_I2C_GIO_I2C2_TRANSFER, &[0x10, 0x44], 1)
                .success
        );
        let i2c_transfer_read = i2c3.device_io_control(i2c_bus::IOCTL_I2C_READ, &[0x10], 1);
        assert_eq!(i2c_transfer_read.output, vec![0x44]);

        let mut i2c4 = namespace.open("I2C4:").unwrap();
        assert!(
            !i2c4
                .device_io_control(i2c_bus::IOCTL_I2C_WRITE_READ, &[0x10, 0x55], 1)
                .success
        );
    }

    #[test]
    fn remote_imu_updates_sensor_backend_reads() {
        let namespace = DeviceNamespace::from_config(DeviceConfigFile {
            version: 1,
            defaults: DeviceDefaults::default(),
            devices: vec![
                DeviceConfig {
                    guest: "SMB1:".to_owned(),
                    kind: DeviceKind::IoctlDevice,
                    backend: DeviceBackend::Accelerometer,
                    host: None,
                    remote_gps: false,
                    enabled: true,
                    note: None,
                },
                DeviceConfig {
                    guest: "MFS1:".to_owned(),
                    kind: DeviceKind::IoctlDevice,
                    backend: DeviceBackend::Magnetometer,
                    host: None,
                    remote_gps: false,
                    enabled: true,
                    note: None,
                },
            ],
        });

        let mut accel = namespace.open("SMB1:").unwrap();
        accel.apply_remote_imu(&serde_json::json!({
            "ax": 0.5,
            "ay": -0.25,
            "az": 1.0
        }));
        let xyz = accel.device_io_control(accelerometer::IOCTL_SMB380_READ_ACCEL_XYZT, &[], 6);
        assert!(xyz.success);
        assert_eq!(&xyz.output[0..2], &128i16.to_le_bytes());
        assert_eq!(&xyz.output[2..4], &(-64i16).to_le_bytes());
        assert_eq!(&xyz.output[4..6], &256i16.to_le_bytes());

        let image = accel.device_io_control(accelerometer::IOCTL_SMB380_GET_IMAGE, &[], 8);
        assert!(image.success);
        assert_eq!(&image.output[2..4], &128i16.to_le_bytes());
        assert_eq!(&image.output[4..6], &(-64i16).to_le_bytes());
        assert_eq!(&image.output[6..8], &256i16.to_le_bytes());

        let mut mag = namespace.open("MFS1:").unwrap();
        mag.apply_remote_imu(&serde_json::json!({
            "magnetometer": {
                "x": 30.0,
                "y": -5.0,
                "z": 42.0
            }
        }));
        let field = mag.device_io_control(magnetometer::IOCTL_MFS_READ_REGISTERS, &[0, 0, 6, 0], 6);
        assert!(field.success);
        assert_eq!(&field.output[0..2], &30i16.to_le_bytes());
        assert_eq!(&field.output[2..4], &(-5i16).to_le_bytes());
        assert_eq!(&field.output[4..6], &42i16.to_le_bytes());
    }

    #[test]
    fn nand_uuid_backend_handles_decoded_ioctl_contracts() {
        let namespace = DeviceNamespace::from_config(DeviceConfigFile {
            version: 1,
            defaults: DeviceDefaults::default(),
            devices: vec![DeviceConfig {
                guest: "UID1:".to_owned(),
                kind: DeviceKind::IoctlDevice,
                backend: DeviceBackend::NandUuid,
                host: None,
                remote_gps: false,
                enabled: true,
                note: None,
            }],
        });

        let mut uid = namespace.open("UID1:").unwrap();
        let uuid = uid.device_io_control(nand_uuid::IOCTL_NAND_UPD_READ_UUID, &[], 4);
        assert!(uuid.success);
        assert_eq!(uuid.bytes_returned, 4);

        let uuid_again = uid.device_io_control(nand_uuid::IOCTL_NAND_UPD_READ_UUID, &[], 4);
        assert_eq!(uuid_again.output, uuid.output);

        let sector = 7u32.to_le_bytes();
        let sector_uuid =
            uid.device_io_control(nand_uuid::IOCTL_NAND_UPD_READ_UUID_BY_SECTORNUM, &sector, 4);
        assert!(sector_uuid.success);
        assert_eq!(sector_uuid.bytes_returned, 4);

        let sector_uuid_16 = uid.device_io_control(
            nand_uuid::IOCTL_NAND_UPD_READ_UUID_BY_SECTORNUM,
            &sector,
            16,
        );
        assert!(sector_uuid_16.success);
        assert_eq!(sector_uuid_16.bytes_returned, 16);
        assert_eq!(&sector_uuid_16.output[0..4], &sector_uuid.output);

        let sector_write_input = [sector.as_slice(), b"0123456789abcdef".as_slice()].concat();
        let sector_write = uid.device_io_control(
            nand_uuid::IOCTL_NAND_UPD_WRITE_UUID_BY_SECTORNUM,
            &sector_write_input,
            0,
        );
        assert!(sector_write.success);
        let sector_uuid_written = uid.device_io_control(
            nand_uuid::IOCTL_NAND_UPD_READ_UUID_BY_SECTORNUM,
            &sector,
            16,
        );
        assert_eq!(sector_uuid_written.output, b"0123456789abcdef");

        let load = uid.device_io_control(
            nand_uuid::IOCTL_NAND_CPU_LOAD_CONTROL,
            &42u32.to_le_bytes(),
            0,
        );
        assert!(load.success);
        assert!(
            !uid.device_io_control(nand_uuid::IOCTL_NAND_UPD_READ_UUID, &[], 2)
                .success
        );
        assert!(!uid.device_io_control(0xdead_beef, &[], 4).success);
    }

    #[test]
    fn pic_controller_backend_handles_decoded_ioctl_contracts() {
        let namespace = DeviceNamespace::from_config(DeviceConfigFile {
            version: 1,
            defaults: DeviceDefaults::default(),
            devices: vec![DeviceConfig {
                guest: "PIC1:".to_owned(),
                kind: DeviceKind::IoctlDevice,
                backend: DeviceBackend::PicController,
                host: None,
                remote_gps: false,
                enabled: true,
                note: None,
            }],
        });

        let mut pic = namespace.open("PIC1:").unwrap();
        assert!(
            pic.device_io_control(pic_controller::IOCTL_DEVICE_PIC_READ_VERSION, &[], 1)
                .success
        );
        assert!(
            pic.device_io_control(pic_controller::IOCTL_NANDUUID_MICOM_RESET_STAGE, &[], 0)
                .success
        );
        assert!(
            pic.device_io_control(pic_controller::IOCTL_NANDUUID_MICOM_RESET_ACK, &[], 0)
                .success
        );

        let read = pic.device_io_control(
            pic_controller::IOCTL_DEVICE_PIC_I2C_SET_EEPROM_COMMAND_READ,
            &[0x10, 0],
            1,
        );
        assert!(read.success);
        assert_eq!(read.bytes_returned, 1);

        let display =
            pic.device_io_control(pic_controller::IOCTL_DEVICE_PIC_I2C_DISPLAY_STATE, &[2], 0);
        assert!(display.success);
        assert!(!pic.device_io_control(0xdead_beef, &[], 0).success);
    }
}
