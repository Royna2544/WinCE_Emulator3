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
    rx: Vec<u8>,
    tx: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceIoControlResult {
    pub success: bool,
    pub bytes_returned: u32,
    pub output: Vec<u8>,
}

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

    pub fn write_file(&mut self, bytes: &[u8]) -> u32 {
        self.tx.extend_from_slice(bytes);
        bytes.len() as u32
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
