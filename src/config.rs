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
    #[serde(default = "default_object_store")]
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
    guest_root: String,
    host_root: Option<PathBuf>,
    total_mbytes: Option<u64>,
    free_mbytes: Option<u64>,
    writable: Option<bool>,
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
                .map(MountConfig::from_toml)
                .collect();
            Self {
                object_store: parsed.object_store.unwrap_or_else(default_object_store),
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
    fn from_toml(raw: MountToml) -> Self {
        let (default_total, default_free) =
            default_mount_capacity(&raw.guest_root, raw.name.as_deref());
        let writable = raw.writable.unwrap_or(true);
        Self {
            name: raw.name,
            guest_root: raw.guest_root,
            host_root: raw.host_root,
            total_mbytes: raw.total_mbytes.unwrap_or(default_total),
            free_mbytes: raw.free_mbytes.unwrap_or(default_free),
            writable,
        }
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

fn default_mount_capacity(guest_root: &str, name: Option<&str>) -> (u64, u64) {
    let normalized = guest_root.trim_matches(['\\', '/']);
    if normalized.eq_ignore_ascii_case("SDMMC Disk")
        || name.is_some_and(|name| name.eq_ignore_ascii_case("sdmmc"))
    {
        (8192, 4096)
    } else {
        (2048, 1024)
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
    fn mount_toml_defaults_capacity_and_forces_hostless_read_only() {
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

[[mounts]]
name = "windows"
guest_root = "\\Windows"
writable = true
"#,
        )
        .unwrap();

        let storage = StorageConfig::load_or_default(Some(&path)).unwrap();
        assert_eq!(storage.object_store.total_mbytes, 64);
        assert_eq!(storage.object_store.free_mbytes, 32);
        assert_eq!(storage.mounts.len(), 2);
        assert_eq!(storage.mounts[0].total_mbytes, 8192);
        assert_eq!(storage.mounts[0].free_mbytes, 4096);
        assert!(storage.mounts[0].writable);
        assert_eq!(storage.mounts[1].total_mbytes, 2048);
        assert_eq!(storage.mounts[1].free_mbytes, 1024);
        assert!(!storage.mounts[1].writable);

        let _ = fs::remove_file(path);
    }
}
