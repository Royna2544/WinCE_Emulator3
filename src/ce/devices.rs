mod accelerometer;
mod i2c_bus;
mod light_sensor;
mod magnetometer;

use std::collections::BTreeMap;
#[cfg(windows)]
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    os::windows::io::AsRawHandle,
    sync::{Arc, Mutex},
};

use serde::Deserialize;

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
    #[serde(rename = "NANDUUID_RETURN")]
    NandUuidReturn,
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
        let config = self
            .devices
            .get(&normalized)
            .ok_or_else(|| Error::MissingDevice(guest_name.to_owned()))?;

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
            runtime: DeviceRuntime::from_backend(&config.backend),
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
            if device.kind != DeviceKind::Serial {
                continue;
            }
            if device.backend == DeviceBackend::Win32Com
                && let Some(host) = &device.host
                && !host.is_empty()
            {
                return Some(host.clone());
            }
            fallback.get_or_insert_with(|| device.guest.clone());
        }
        fallback
    }
}

impl DeviceRuntime {
    fn from_backend(backend: &DeviceBackend) -> Self {
        match backend {
            DeviceBackend::Accelerometer => {
                Self::Accelerometer(accelerometer::Accelerometer::new())
            }
            DeviceBackend::I2cBus => Self::I2cBus(i2c_bus::I2cBus::new()),
            DeviceBackend::LightSensor => Self::LightSensor(light_sensor::LightSensor::new()),
            DeviceBackend::Magnetometer => Self::Magnetometer(magnetometer::Magnetometer::new()),
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
        match &mut self.runtime {
            DeviceRuntime::Accelerometer(sensor) => {
                return sensor.device_io_control(ioctl_code, input, output_capacity);
            }
            DeviceRuntime::I2cBus(bus) => {
                return bus.device_io_control(ioctl_code, input, output_capacity);
            }
            DeviceRuntime::LightSensor(sensor) => {
                return sensor.device_io_control(ioctl_code, input, output_capacity);
            }
            DeviceRuntime::Magnetometer(sensor) => {
                return sensor.device_io_control(ioctl_code, input, output_capacity);
            }
            DeviceRuntime::None => {}
        }

        match self.backend {
            DeviceBackend::Stub | DeviceBackend::Win32Com => DeviceIoControlResult {
                success: false,
                bytes_returned: 0,
                output: Vec::new(),
            },
            DeviceBackend::Accelerometer
            | DeviceBackend::I2cBus
            | DeviceBackend::LightSensor
            | DeviceBackend::Magnetometer => DeviceIoControlResult {
                success: false,
                bytes_returned: 0,
                output: Vec::new(),
            },
            DeviceBackend::NandUuidReturn => {
                let mut output = ioctl_code.to_le_bytes().to_vec();
                output.extend_from_slice(self.guest_name.as_bytes());
                output.truncate(output_capacity as usize);
                DeviceIoControlResult {
                    success: true,
                    bytes_returned: output.len() as u32,
                    output,
                }
            }
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
                enabled: true,
                note: None,
            }],
        });

        assert_eq!(namespace.open("com7").unwrap().guest_name, "COM7:");
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
                    enabled: true,
                    note: None,
                },
                DeviceConfig {
                    guest: "LSD1:".to_owned(),
                    kind: DeviceKind::IoctlDevice,
                    backend: DeviceBackend::LightSensor,
                    host: None,
                    enabled: true,
                    note: None,
                },
                DeviceConfig {
                    guest: "MFS1:".to_owned(),
                    kind: DeviceKind::IoctlDevice,
                    backend: DeviceBackend::Magnetometer,
                    host: None,
                    enabled: true,
                    note: None,
                },
                DeviceConfig {
                    guest: "I2C2:".to_owned(),
                    kind: DeviceKind::IoctlDevice,
                    backend: DeviceBackend::I2cBus,
                    host: None,
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

        let mut i2c = namespace.open("I2C2:").unwrap();
        assert!(
            i2c.device_io_control(i2c_bus::IOCTL_I2C_WRITE, &[0x10, 0x33], 0)
                .success
        );
        let i2c_read = i2c.device_io_control(i2c_bus::IOCTL_I2C_READ, &[0x10], 1);
        assert_eq!(i2c_read.output, vec![0x33]);
    }
}
