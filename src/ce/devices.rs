use std::collections::BTreeMap;

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
        _input: &[u8],
        output_capacity: u32,
    ) -> DeviceIoControlResult {
        match self.backend {
            DeviceBackend::Stub | DeviceBackend::Win32Com => DeviceIoControlResult {
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

    pub fn tx_bytes(&self) -> &[u8] {
        &self.tx
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
}
