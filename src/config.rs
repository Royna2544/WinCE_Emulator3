use std::{fs, path::Path};

use serde::Deserialize;

use crate::{
    ce::{devices::DeviceConfigFile, registry::RegistryDump},
    error::{Error, Result},
};

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeConfig {
    pub registry: RegistryDump,
    pub devices: DeviceConfigFile,
}

impl RuntimeConfig {
    pub fn load(registry_path: impl AsRef<Path>, devices_path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            registry: read_json(registry_path.as_ref())?,
            devices: read_json(devices_path.as_ref())?,
        })
    }
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).map_err(|source| Error::Read {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_slice(&bytes).map_err(|source| Error::Json {
        path: path.to_path_buf(),
        source,
    })
}
