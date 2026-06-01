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
}

fn normalize_device_name(name: &str) -> String {
    name.trim().trim_end_matches(':').to_ascii_uppercase()
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
