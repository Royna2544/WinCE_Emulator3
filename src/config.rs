use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::{
    ce::{devices::DeviceConfigFile, registry::RegistryDump},
    error::{Error, Result},
};

pub const DEFAULT_REGISTRY_PATH: &str = "registry.reg";
pub const DEFAULT_DEVICES_PATH: &str = "serial_devices.json";
pub const DEFAULT_MOUNT_CONFIG_PATH: &str = "mounts.toml";

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeConfig {
    pub registry: RegistryDump,
    pub devices: DeviceConfigFile,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub root: RootConfig,
    pub object_store: ObjectStoreConfig,
    #[serde(default)]
    pub mounts: Vec<MountConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RootConfig {
    pub host_root: PathBuf,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ObjectStoreConfig {
    pub total_mbytes: u64,
    pub free_mbytes: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MountConfig {
    pub name: Option<String>,
    pub device_name: Option<String>,
    pub guest_root: String,
    pub host_root: Option<PathBuf>,
    pub total_mbytes: u64,
    pub free_mbytes: u64,
    pub writable: bool,
    pub removable: bool,
    pub system: bool,
    pub hidden: bool,
    pub interface_classes: Vec<String>,
    pub registry_roots: Vec<String>,
    pub registry_subkey: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct StorageToml {
    root: Option<RootToml>,
    object_store: Option<ObjectStoreConfig>,
    #[serde(default)]
    mounts: Vec<MountToml>,
}

#[derive(Debug, Clone, Deserialize)]
struct RootToml {
    host_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
struct MountToml {
    name: Option<String>,
    device_name: Option<String>,
    guest_root: Option<String>,
    host_root: Option<PathBuf>,
    total_mbytes: Option<u64>,
    free_mbytes: Option<u64>,
    writable: Option<bool>,
    removable: Option<bool>,
    system: Option<bool>,
    hidden: Option<bool>,
    #[serde(default)]
    interface_classes: Vec<String>,
    #[serde(default)]
    registry_roots: Vec<String>,
    registry_subkey: Option<String>,
}

impl RuntimeConfig {
    pub fn load_default() -> Result<Self> {
        Self::load(DEFAULT_REGISTRY_PATH, DEFAULT_DEVICES_PATH)
    }

    pub fn load_default_with_mounts() -> Result<Self> {
        Self::load_with_mounts(
            DEFAULT_REGISTRY_PATH,
            DEFAULT_DEVICES_PATH,
            Some(DEFAULT_MOUNT_CONFIG_PATH),
        )
    }

    pub fn load(registry_path: impl AsRef<Path>, devices_path: impl AsRef<Path>) -> Result<Self> {
        Self::load_with_mounts(registry_path, devices_path, Option::<&Path>::None)
    }

    pub fn load_with_mounts(
        registry_path: impl AsRef<Path>,
        devices_path: impl AsRef<Path>,
        mount_path: Option<impl AsRef<Path>>,
    ) -> Result<Self> {
        Ok(Self {
            registry: read_registry(registry_path.as_ref())?,
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
                root: RootConfig {
                    host_root: parsed
                        .root
                        .and_then(|root| root.host_root)
                        .unwrap_or_else(|| PathBuf::from(".")),
                },
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
                root: RootConfig {
                    host_root: PathBuf::from("."),
                },
                object_store: default_object_store(),
                mounts: Vec::new(),
            }
        };
        storage.finalize_root();
        Ok(storage)
    }

    fn finalize_root(&mut self) {
        if !self.root.host_root.is_dir() {
            self.root.host_root = PathBuf::from(".");
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
            device_name: raw.device_name,
            guest_root: raw.guest_root?,
            host_root: raw.host_root,
            total_mbytes: raw.total_mbytes?,
            free_mbytes: raw.free_mbytes?,
            writable: raw.writable?,
            removable: raw.removable.unwrap_or(false),
            system: raw.system.unwrap_or(false),
            hidden: raw.hidden.unwrap_or(false),
            interface_classes: raw.interface_classes,
            registry_roots: raw.registry_roots,
            registry_subkey: raw.registry_subkey,
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

fn read_registry(path: &Path) -> Result<RegistryDump> {
    let bytes = fs::read(path).map_err(|source| Error::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let extension_is_reg = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("reg"));
    let text = String::from_utf8_lossy(&bytes);
    let starts_as_regedit = text
        .trim_start_matches('\u{feff}')
        .trim_start()
        .starts_with("REGEDIT4")
        || text
            .trim_start_matches('\u{feff}')
            .trim_start()
            .starts_with("Windows Registry Editor");

    if extension_is_reg || starts_as_regedit {
        RegistryDump::from_regedit(&text, Some(path.display().to_string()))
    } else {
        serde_json::from_slice(&bytes).map_err(|source| Error::Json {
            path: path.to_path_buf(),
            source,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_storage_has_object_store_but_no_mounts() {
        let storage = StorageConfig::load_or_default(Option::<&Path>::None).unwrap();

        assert_eq!(storage.object_store.total_mbytes, 256);
        assert_eq!(storage.object_store.free_mbytes, 128);
        assert_eq!(storage.root.host_root, PathBuf::from("."));
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
device_name = "DSK1:"
guest_root = "\\SDMMC Disk"
host_root = "D:\\INAVI\\SDMMC"
total_mbytes = 8192
free_mbytes = 4096
writable = true
removable = true
system = false
hidden = false
interface_classes = ["{A32942B7-920C-486b-B0E6-92A702A99B35}"]
registry_roots = [
    "HKLM\\System\\StorageManager\\Profiles\\SDMemory",
    "HKLM\\System\\StorageManager",
]
registry_subkey = "FATFS"

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
        assert_eq!(storage.root.host_root, PathBuf::from("."));
        assert_eq!(storage.mounts.len(), 1);
        assert_eq!(storage.mounts[0].total_mbytes, 8192);
        assert_eq!(storage.mounts[0].free_mbytes, 4096);
        assert_eq!(storage.mounts[0].device_name.as_deref(), Some("DSK1:"));
        assert_eq!(
            storage.mounts[0].interface_classes,
            vec!["{A32942B7-920C-486b-B0E6-92A702A99B35}".to_owned()]
        );
        assert_eq!(
            storage.mounts[0].registry_roots,
            vec![
                r"HKLM\System\StorageManager\Profiles\SDMemory".to_owned(),
                r"HKLM\System\StorageManager".to_owned()
            ]
        );
        assert_eq!(storage.mounts[0].registry_subkey.as_deref(), Some("FATFS"));
        assert!(storage.mounts[0].writable);
        assert!(storage.mounts[0].removable);
        assert!(!storage.mounts[0].system);
        assert!(!storage.mounts[0].hidden);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn mount_toml_accepts_valid_root_host_root() {
        let root =
            std::env::temp_dir().join(format!("wince-mount-config-root-{}", std::process::id()));
        let path = root.with_extension("toml");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(
            &path,
            format!(
                r#"
[object_store]
total_mbytes = 64
free_mbytes = 32

[root]
host_root = "{}"
"#,
                root.display().to_string().replace('\\', "\\\\")
            ),
        )
        .unwrap();

        let storage = StorageConfig::load_or_default(Some(&path)).unwrap();
        assert_eq!(storage.root.host_root, root);

        let _ = fs::remove_file(path);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn mount_toml_falls_back_for_missing_root_host_root() {
        let missing = std::env::temp_dir().join(format!(
            "wince-mount-config-missing-root-{}",
            std::process::id()
        ));
        let path = missing.with_extension("toml");
        let _ = fs::remove_dir_all(&missing);
        fs::write(
            &path,
            format!(
                r#"
[object_store]
total_mbytes = 64
free_mbytes = 32

[root]
host_root = "{}"
"#,
                missing.display().to_string().replace('\\', "\\\\")
            ),
        )
        .unwrap();

        let storage = StorageConfig::load_or_default(Some(&path)).unwrap();
        assert_eq!(storage.root.host_root, PathBuf::from("."));

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

    #[test]
    fn runtime_config_loads_regedit_registry_file() {
        let root = std::env::temp_dir().join(format!(
            "wince-runtime-regedit-config-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let registry_path = root.join(DEFAULT_REGISTRY_PATH);
        let devices_path = root.join(DEFAULT_DEVICES_PATH);
        fs::write(
            &registry_path,
            r#"
REGEDIT4

[HKEY_LOCAL_MACHINE\Ident]
"Name"="nav"
"#,
        )
        .unwrap();
        fs::write(&devices_path, r#"{"devices":[]}"#).unwrap();

        let config = RuntimeConfig::load(&registry_path, &devices_path).unwrap();
        assert!(config.registry.keys.contains_key("hklm\\Ident"));

        let _ = fs::remove_dir_all(root);
    }
}
