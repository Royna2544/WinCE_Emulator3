use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::{
    ce::{devices::DeviceConfigFile, registry::RegistryDump},
    error::{Error, Result},
};

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeConfig {
    pub registry: RegistryDump,
    pub devices: DeviceConfigFile,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub object_store: ObjectStoreConfig,
    #[serde(default)]
    pub mounts: Vec<MountConfig>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ObjectStoreConfig {
    pub total_mbytes: u64,
    pub free_mbytes: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MountConfig {
    pub name: Option<String>,
    pub guest_root: String,
    pub host_root: Option<PathBuf>,
    pub total_mbytes: u64,
    pub free_mbytes: u64,
    pub writable: bool,
    pub removable: bool,
    pub system: bool,
    pub hidden: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct StorageToml {
    object_store: Option<ObjectStoreConfig>,
    #[serde(default)]
    mounts: Vec<MountToml>,
}

#[derive(Debug, Clone, Deserialize)]
struct MountToml {
    name: Option<String>,
    guest_root: Option<String>,
    host_root: Option<PathBuf>,
    total_mbytes: Option<u64>,
    free_mbytes: Option<u64>,
    writable: Option<bool>,
    removable: Option<bool>,
    system: Option<bool>,
    hidden: Option<bool>,
}

impl RuntimeConfig {
    pub fn load(registry_path: impl AsRef<Path>, devices_path: impl AsRef<Path>) -> Result<Self> {
        Self::load_with_mounts(registry_path, devices_path, Option::<&Path>::None)
    }

    pub fn load_with_mounts(
        registry_path: impl AsRef<Path>,
        devices_path: impl AsRef<Path>,
        mount_path: Option<impl AsRef<Path>>,
    ) -> Result<Self> {
        Ok(Self {
            registry: read_json(registry_path.as_ref())?,
            devices: read_json(devices_path.as_ref())?,
            storage: StorageConfig::load_or_default(mount_path)?,
        })
    }
}

impl StorageConfig {
    pub fn load_or_default(mount_path: Option<impl AsRef<Path>>) -> Result<Self> {
        let mut storage = if let Some(path) = mount_path {
            let path = path.as_ref();
            let bytes = fs::read(path).map_err(|source| Error::Read {
                path: path.to_path_buf(),
                source,
            })?;
            let text = std::str::from_utf8(&bytes).map_err(|source| {
                Error::InvalidArgument(format!(
                    "{} is not valid UTF-8 TOML: {source}",
                    path.display()
                ))
            })?;
            let parsed: StorageToml = toml::from_str(text).map_err(|source| Error::Toml {
                path: path.to_path_buf(),
                source,
            })?;
            let mounts = parsed
                .mounts
                .into_iter()
                .filter_map(MountConfig::from_toml)
                .collect();
            Self {
                object_store: parsed.object_store.ok_or_else(|| {
                    Error::InvalidArgument(format!(
                        "{} is missing required [object_store]",
                        path.display()
                    ))
                })?,
                mounts,
            }
        } else {
            Self {
                object_store: default_object_store(),
                mounts: Vec::new(),
            }
        };
        storage.finalize_mounts();
        Ok(storage)
    }

    fn finalize_mounts(&mut self) {
        for mount in &mut self.mounts {
            if mount.host_root.is_none() {
                mount.writable = false;
            }
        }
    }
}

impl ObjectStoreConfig {
    pub fn total_bytes(self) -> u64 {
        mbytes_to_bytes(self.total_mbytes)
    }

    pub fn free_bytes(self) -> u64 {
        mbytes_to_bytes(self.free_mbytes)
    }
}

impl MountConfig {
    fn from_toml(raw: MountToml) -> Option<Self> {
        Some(Self {
            name: Some(raw.name?),
            guest_root: raw.guest_root?,
            host_root: raw.host_root,
            total_mbytes: raw.total_mbytes?,
            free_mbytes: raw.free_mbytes?,
            writable: raw.writable?,
            removable: raw.removable.unwrap_or(false),
            system: raw.system.unwrap_or(false),
            hidden: raw.hidden.unwrap_or(false),
        })
    }

    pub fn total_bytes(&self) -> u64 {
        mbytes_to_bytes(self.total_mbytes)
    }

    pub fn free_bytes(&self) -> u64 {
        mbytes_to_bytes(self.free_mbytes)
    }
}

fn default_object_store() -> ObjectStoreConfig {
    ObjectStoreConfig {
        total_mbytes: 256,
        free_mbytes: 128,
    }
}

fn mbytes_to_bytes(mbytes: u64) -> u64 {
    mbytes.saturating_mul(1024 * 1024)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_storage_has_object_store_but_no_mounts() {
        let storage = StorageConfig::load_or_default(Option::<&Path>::None).unwrap();

        assert_eq!(storage.object_store.total_mbytes, 256);
        assert_eq!(storage.object_store.free_mbytes, 128);
        assert!(storage.mounts.is_empty());
    }

    #[test]
    fn mount_toml_skips_entries_missing_required_fields() {
        let path =
            std::env::temp_dir().join(format!("wince-mount-config-{}.toml", std::process::id()));
        fs::write(
            &path,
            r#"
[object_store]
total_mbytes = 64
free_mbytes = 32

[[mounts]]
name = "sdmmc"
guest_root = "\\SDMMC Disk"
host_root = "D:\\INAVI\\SDMMC"
total_mbytes = 8192
free_mbytes = 4096
writable = true
removable = true
system = false
hidden = false

[[mounts]]
name = "windows"
guest_root = "\\Windows"
writable = true

[[mounts]]
name = "resident_flash"
guest_root = "\\ResidentFlash"
total_mbytes = 2048
free_mbytes = 1024

[[mounts]]
guest_root = "\\Nameless"
host_root = "D:\\INAVI\\Nameless"
total_mbytes = 512
free_mbytes = 256
writable = true
"#,
        )
        .unwrap();

        let storage = StorageConfig::load_or_default(Some(&path)).unwrap();
        assert_eq!(storage.object_store.total_mbytes, 64);
        assert_eq!(storage.object_store.free_mbytes, 32);
        assert_eq!(storage.mounts.len(), 1);
        assert_eq!(storage.mounts[0].total_mbytes, 8192);
        assert_eq!(storage.mounts[0].free_mbytes, 4096);
        assert!(storage.mounts[0].writable);
        assert!(storage.mounts[0].removable);
        assert!(!storage.mounts[0].system);
        assert!(!storage.mounts[0].hidden);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn mount_toml_requires_object_store() {
        let path = std::env::temp_dir().join(format!(
            "wince-mount-config-no-store-{}.toml",
            std::process::id()
        ));
        fs::write(
            &path,
            r#"
[[mounts]]
name = "sdmmc"
guest_root = "\\SDMMC Disk"
host_root = "D:\\INAVI\\SDMMC"
total_mbytes = 8192
free_mbytes = 4096
writable = true
"#,
        )
        .unwrap();

        let err = StorageConfig::load_or_default(Some(&path)).unwrap_err();
        assert!(err.to_string().contains("missing required [object_store]"));

        let _ = fs::remove_file(path);
    }
}
