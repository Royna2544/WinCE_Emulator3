use std::{
    collections::BTreeMap,
    fmt, fs,
    io::{Read, Seek, SeekFrom, Write},
    path::{Component, Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use crate::{
    config::{MountConfig, StorageConfig},
    error::{Error, Result},
};

pub const GENERIC_READ: u32 = 0x8000_0000;
pub const GENERIC_WRITE: u32 = 0x4000_0000;

pub const CREATE_NEW: u32 = 1;
pub const CREATE_ALWAYS: u32 = 2;
pub const OPEN_EXISTING: u32 = 3;
pub const OPEN_ALWAYS: u32 = 4;
pub const TRUNCATE_EXISTING: u32 = 5;

pub const FILE_ATTRIBUTE_READONLY: u32 = 0x0000_0001;
pub const FILE_ATTRIBUTE_HIDDEN: u32 = 0x0000_0002;
pub const FILE_ATTRIBUTE_SYSTEM: u32 = 0x0000_0004;
pub const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x0000_0010;
pub const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x0000_0020;
pub const FILE_ATTRIBUTE_TEMPORARY: u32 = 0x0000_0100;
const CE_VOLUME_ATTRIBUTE_READONLY: u32 = 0x0000_0001;
const CE_VOLUME_ATTRIBUTE_HIDDEN: u32 = 0x0000_0002;
const CE_VOLUME_ATTRIBUTE_REMOVABLE: u32 = 0x0000_0004;
const CE_VOLUME_ATTRIBUTE_SYSTEM: u32 = 0x0000_0008;
const CE_VOLUME_FLAG_STORE: u32 = 0x0000_0020;
const CE_VOLUME_FLAG_RAMFS: u32 = 0x0000_0040;
const CE_VOLUME_DEFAULT_BLOCK_SIZE: u32 = 4096;
const AFS_FLAG_HIDDEN: u32 = 0x0001;
const AFS_FLAG_SYSTEM: u32 = 0x0020;
const AFS_FLAG_PERMANENT: u32 = 0x0040;
const READ_ONLY_MEMORY_BACKING_MIN_BYTES: usize = 1024 * 1024;
const READ_ONLY_MEMORY_BACKING_MAX_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct HostFileSystem {
    root: PathBuf,
    mounts: BTreeMap<String, FileMount>,
    mount_order: Vec<String>,
    object_store: ObjectStore,
    root_relative_mount: Option<String>,
    next_id: u32,
    open_files: BTreeMap<u32, OpenFile>,
    open_finds: BTreeMap<u32, OpenFind>,
    file_locks: BTreeMap<PathBuf, Vec<FileLock>>,
    read_only_cache: BTreeMap<PathBuf, ReadOnlyCacheEntry>,
    io_stats: FileIoStats,
}

#[derive(Debug, Clone)]
pub struct FileMount {
    pub name: Option<String>,
    pub device_name: Option<String>,
    pub bus_name: Option<String>,
    pub guest_root: String,
    pub host_root: Option<PathBuf>,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub writable: bool,
    pub removable: bool,
    pub system: bool,
    pub hidden: bool,
    pub interface_classes: Vec<String>,
    pub registry_roots: Vec<String>,
    pub registry_subkey: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInterfaceAdvertisementSpec {
    pub owner: String,
    pub interfaces: Vec<DeviceInterfaceClassAdvertisementSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInterfaceClassAdvertisementSpec {
    pub class: String,
    pub name: String,
}

#[derive(Debug, Clone)]
struct GuestVolumePath {
    normalized_path: String,
    volume_key: String,
    is_mount_root: bool,
}

impl GuestVolumePath {
    fn is_mount_root(&self) -> bool {
        self.is_mount_root
    }
}

#[derive(Debug, Clone)]
struct ReadOnlyCacheEntry {
    bytes: Arc<Vec<u8>>,
    modified: Option<SystemTime>,
    len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectStore {
    pub total_bytes: u64,
    pub free_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiskSpace {
    pub total_bytes: u64,
    pub free_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeInfo {
    pub attributes: u32,
    pub flags: u32,
    pub block_size: u32,
    pub store_name: String,
    pub partition_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountTableVolumeInfo {
    pub name: String,
    pub flags: u32,
}

#[derive(Debug, Clone)]
pub struct OpenFile {
    pub id: u32,
    pub guest_path: String,
    pub host_path: PathBuf,
    cursor: usize,
    backing: FileBacking,
    read_cache: Vec<u8>,
    read_cache_start: usize,
    file_len: usize,
    desired_access: u32,
    writable: bool,
    dirty: bool,
    eof: bool,
}

impl OpenFile {
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn file_len(&self) -> usize {
        self.file_len
    }

    pub fn is_eof(&self) -> bool {
        self.eof
    }

    pub fn is_memory_backed(&self) -> bool {
        matches!(
            self.backing,
            FileBacking::Memory(_) | FileBacking::ReadOnlyMemory(_)
        )
    }

    pub fn is_host_file_backed(&self) -> bool {
        matches!(self.backing, FileBacking::HostFile(_))
    }

    pub fn memory_len(&self) -> usize {
        match &self.backing {
            FileBacking::Memory(bytes) => bytes.len(),
            FileBacking::ReadOnlyMemory(bytes) => bytes.len(),
            FileBacking::HostFile(_) => 0,
        }
    }

    fn read_into<F>(&mut self, requested: usize, write: &mut F) -> Result<(u32, bool)>
    where
        F: FnMut(&[u8]) -> Result<()>,
    {
        match &mut self.backing {
            FileBacking::Memory(bytes) => {
                if self.cursor >= self.file_len {
                    write(&[])?;
                    if requested != 0 {
                        self.eof = true;
                    }
                    return Ok((0, false));
                }
                let start = self.cursor;
                let end = self.cursor.saturating_add(requested).min(self.file_len);
                write(&bytes[start..end])?;
                self.cursor = end;
                self.eof = end - start < requested;
                Ok(((end - start) as u32, false))
            }
            FileBacking::ReadOnlyMemory(bytes) => {
                if self.cursor >= self.file_len {
                    write(&[])?;
                    if requested != 0 {
                        self.eof = true;
                    }
                    return Ok((0, false));
                }
                let start = self.cursor;
                let end = self.cursor.saturating_add(requested).min(self.file_len);
                write(&bytes[start..end])?;
                self.cursor = end;
                self.eof = end - start < requested;
                Ok(((end - start) as u32, false))
            }
            FileBacking::HostFile(file) => {
                let transferred = read_cached_host_file_into(
                    &self.host_path,
                    file,
                    &mut self.read_cache,
                    &mut self.read_cache_start,
                    self.cursor,
                    requested,
                    write,
                )?;
                self.cursor = self.cursor.saturating_add(transferred as usize);
                self.eof = transferred as usize != requested;
                Ok((transferred, true))
            }
        }
    }

    fn read_at(&mut self, offset: usize, requested: usize) -> Result<(Vec<u8>, bool)> {
        match &mut self.backing {
            FileBacking::Memory(bytes) => {
                if offset >= self.file_len {
                    return Ok((Vec::new(), false));
                }
                let end = offset.saturating_add(requested).min(self.file_len);
                Ok((bytes[offset..end].to_vec(), false))
            }
            FileBacking::ReadOnlyMemory(bytes) => {
                if offset >= self.file_len {
                    return Ok((Vec::new(), false));
                }
                let end = offset.saturating_add(requested).min(self.file_len);
                Ok((bytes[offset..end].to_vec(), false))
            }
            FileBacking::HostFile(file) => {
                let mut bytes = Vec::with_capacity(requested);
                let mut append = |chunk: &[u8]| -> Result<()> {
                    bytes.extend_from_slice(chunk);
                    Ok(())
                };
                read_cached_host_file_into(
                    &self.host_path,
                    file,
                    &mut self.read_cache,
                    &mut self.read_cache_start,
                    offset,
                    requested,
                    &mut append,
                )?;
                Ok((bytes, true))
            }
        }
    }

    fn write_current(&mut self, bytes: &[u8]) -> Result<()> {
        let offset = self.cursor;
        self.write_at(offset, bytes)?;
        self.cursor = offset.saturating_add(bytes.len());
        self.eof = false;
        Ok(())
    }

    fn write_at(&mut self, offset: usize, bytes: &[u8]) -> Result<()> {
        match &mut self.backing {
            FileBacking::Memory(data) => {
                let end = offset.saturating_add(bytes.len());
                if end > data.len() {
                    data.resize(end, 0);
                }
                data[offset..end].copy_from_slice(bytes);
                self.file_len = data.len();
            }
            FileBacking::ReadOnlyMemory(_) => {
                return Err(Error::AccessDenied(format!(
                    "file is read-only: {}",
                    self.guest_path
                )));
            }
            FileBacking::HostFile(file) => {
                file.seek(SeekFrom::Start(offset as u64))
                    .map_err(|source| Error::Io {
                        path: self.host_path.clone(),
                        source,
                    })?;
                file.write_all(bytes).map_err(|source| Error::Io {
                    path: self.host_path.clone(),
                    source,
                })?;
                self.file_len = self.file_len.max(offset.saturating_add(bytes.len()));
                self.read_cache.clear();
                self.read_cache_start = 0;
            }
        }
        self.dirty = true;
        self.eof = false;
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        match &mut self.backing {
            FileBacking::Memory(data) => {
                if let Some(parent) = self.host_path.parent() {
                    fs::create_dir_all(parent).map_err(|source| Error::Io {
                        path: parent.to_path_buf(),
                        source,
                    })?;
                }
                fs::write(&self.host_path, &*data).map_err(|source| Error::Io {
                    path: self.host_path.clone(),
                    source,
                })?;
                self.file_len = data.len();
            }
            FileBacking::ReadOnlyMemory(_) => {}
            FileBacking::HostFile(file) => {
                if self.writable {
                    file.sync_all().map_err(|source| Error::Io {
                        path: self.host_path.clone(),
                        source,
                    })?;
                }
            }
        }
        self.dirty = false;
        Ok(())
    }
}

enum FileBacking {
    Memory(Vec<u8>),
    ReadOnlyMemory(Arc<Vec<u8>>),
    HostFile(fs::File),
}

impl Clone for FileBacking {
    fn clone(&self) -> Self {
        match self {
            Self::Memory(bytes) => Self::Memory(bytes.clone()),
            Self::ReadOnlyMemory(bytes) => Self::ReadOnlyMemory(Arc::clone(bytes)),
            Self::HostFile(file) => {
                Self::HostFile(file.try_clone().expect("clone open host file handle"))
            }
        }
    }
}

impl fmt::Debug for FileBacking {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Memory(bytes) => f
                .debug_tuple("Memory")
                .field(&format_args!("{} bytes", bytes.len()))
                .finish(),
            Self::ReadOnlyMemory(bytes) => f
                .debug_tuple("ReadOnlyMemory")
                .field(&format_args!("{} bytes", bytes.len()))
                .finish(),
            Self::HostFile(_) => f.write_str("HostFile(<open>)"),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FileIoStats {
    pub host_file_open_count: u64,
    pub host_file_read_count: u64,
    pub host_file_read_bytes: u64,
    pub memory_backed_open_count: u64,
    pub max_read_request: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindData {
    pub attributes: u32,
    pub file_size: u64,
    pub file_name: String,
}

#[derive(Debug, Clone)]
pub struct OpenFind {
    pub id: u32,
    pub guest_pattern: String,
    pub entries: Vec<FindData>,
    pub cursor: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileLock {
    owner_file_id: u32,
    start: u64,
    finish: u64,
    exclusive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileLockStatus {
    Success,
    Conflict,
    InvalidParameter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileIoResult {
    pub success: bool,
    pub bytes_transferred: u32,
}

impl HostFileSystem {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            mounts: BTreeMap::new(),
            mount_order: Vec::new(),
            object_store: ObjectStore {
                total_bytes: 256 * 1024 * 1024,
                free_bytes: 128 * 1024 * 1024,
            },
            root_relative_mount: None,
            next_id: 1,
            open_files: BTreeMap::new(),
            open_finds: BTreeMap::new(),
            file_locks: BTreeMap::new(),
            read_only_cache: BTreeMap::new(),
            io_stats: FileIoStats::default(),
        }
    }

    pub fn from_storage(storage: StorageConfig) -> Self {
        let mut fs = Self::new(storage.root.host_root);
        fs.object_store = ObjectStore {
            total_bytes: storage.object_store.total_bytes(),
            free_bytes: storage.object_store.free_bytes(),
        };
        for mount in storage.mounts {
            fs.mount(mount);
        }
        fs
    }

    pub fn mount_guest_root(&mut self, guest_root: &str, host_root: impl Into<PathBuf>) {
        self.mount(MountConfig {
            name: None,
            device_name: None,
            bus_name: None,
            guest_root: guest_root.to_owned(),
            host_root: Some(host_root.into()),
            total_mbytes: 8192,
            free_mbytes: 4096,
            writable: true,
            removable: true,
            system: false,
            hidden: false,
            interface_classes: Vec::new(),
            registry_roots: Vec::new(),
            registry_subkey: None,
        });
    }

    pub fn register_fsdmgr_mount_name(&mut self, mount_name: &str) -> Result<(String, bool)> {
        let normalized_base = normalize_fsdmgr_mount_name(mount_name)?;
        for suffix in 1..=9 {
            let candidate = if suffix == 1 {
                normalized_base.clone()
            } else {
                format!("{normalized_base}{suffix}")
            };
            if self.mounts.contains_key(&candidate) {
                continue;
            }
            let guest_root = format!("\\{}", candidate.replace('/', "\\"));
            self.mount(MountConfig {
                name: None,
                device_name: None,
                bus_name: None,
                guest_root: guest_root.clone(),
                host_root: None,
                total_mbytes: 8192,
                free_mbytes: 4096,
                writable: true,
                removable: true,
                system: false,
                hidden: false,
                interface_classes: Vec::new(),
                registry_roots: Vec::new(),
                registry_subkey: None,
            });
            return Ok((guest_root, true));
        }
        Err(Error::OutOfStructures(format!(
            "no FSDMGR mount-name slots left for {mount_name}"
        )))
    }

    pub fn existing_fsdmgr_mount_root(&self, mount_name: &str) -> Result<Option<String>> {
        let normalized_base = normalize_fsdmgr_mount_name(mount_name)?;
        if normalized_base.is_empty() {
            return Err(Error::InvalidArgument("empty FSDMGR mount name".to_owned()));
        }
        Ok(self
            .mounts
            .contains_key(&normalized_base)
            .then(|| format!("\\{}", normalized_base.replace('/', "\\"))))
    }

    pub fn registry_paths_for_guest_root(&self, guest_root: &str) -> Vec<String> {
        let guest_root = normalize_guest_path(guest_root);
        let Some(mount) = self.mounts.get(&guest_root) else {
            return Vec::new();
        };
        mount
            .registry_roots
            .iter()
            .filter(|root| !root.trim().is_empty())
            .map(|root| match mount.registry_subkey.as_deref() {
                Some(subkey) if !subkey.trim().is_empty() => {
                    format!(
                        "{}\\{}",
                        root.trim_end_matches('\\'),
                        subkey.trim_matches('\\')
                    )
                }
                _ => root.trim_end_matches('\\').to_owned(),
            })
            .collect()
    }

    pub fn copy_mount_registry_profile(&mut self, source_root: &str, target_root: &str) {
        let source_root = normalize_guest_path(source_root);
        let target_root = normalize_guest_path(target_root);
        let Some(source) = self.mounts.get(&source_root).cloned() else {
            return;
        };
        let Some(target) = self.mounts.get_mut(&target_root) else {
            return;
        };
        target.registry_roots = source.registry_roots;
        target.registry_subkey = source.registry_subkey;
    }

    pub fn unmount_guest_root(&mut self, guest_root: &str) -> Option<FileMount> {
        let guest_root = normalize_guest_path(guest_root);
        let mount = self.mounts.get(&guest_root)?;
        if !mount.removable {
            return None;
        }
        self.mount_order.retain(|entry| entry != &guest_root);
        if self.root_relative_mount.as_deref() == Some(guest_root.as_str()) {
            self.root_relative_mount = None;
        }
        self.mounts.remove(&guest_root)
    }

    pub fn mount(&mut self, mount: MountConfig) {
        let guest_root = normalize_guest_path(&mount.guest_root);
        if guest_root.is_empty() {
            return;
        }
        let total_bytes = mount.total_bytes();
        let free_bytes = mount.free_bytes();
        let host_root = mount
            .host_root
            .or_else(|| Some(default_mount_host_root(&self.root, &guest_root)));
        let writable = mount.writable;
        if !self.mounts.contains_key(&guest_root) {
            self.mount_order.push(guest_root.clone());
        }
        self.mounts.insert(
            guest_root.clone(),
            FileMount {
                name: mount.name,
                device_name: mount.device_name,
                bus_name: mount.bus_name,
                guest_root,
                host_root,
                total_bytes,
                free_bytes,
                writable,
                removable: mount.removable,
                system: mount.system,
                hidden: mount.hidden,
                interface_classes: mount.interface_classes,
                registry_roots: mount.registry_roots,
                registry_subkey: mount.registry_subkey,
            },
        );
    }

    pub fn device_interface_advertisement_specs(&self) -> Vec<DeviceInterfaceAdvertisementSpec> {
        self.mounts_in_order()
            .filter_map(device_interface_advertisement_spec)
            .collect()
    }

    pub fn device_interface_advertisement_specs_for_guest_root(
        &self,
        guest_root: &str,
    ) -> Vec<DeviceInterfaceAdvertisementSpec> {
        let guest_root = normalize_guest_path(guest_root);
        self.mounts
            .get(&guest_root)
            .and_then(device_interface_advertisement_spec)
            .into_iter()
            .collect()
    }

    pub fn object_store(&self) -> ObjectStore {
        self.object_store
    }

    pub fn disk_space_for_path(&self, guest_path: Option<&str>) -> DiskSpace {
        if let Some(mount) = guest_path.and_then(|path| self.mount_for_guest_path(path)) {
            return DiskSpace {
                total_bytes: mount.total_bytes,
                free_bytes: mount.free_bytes.min(mount.total_bytes),
            };
        }
        DiskSpace {
            total_bytes: self.object_store.total_bytes,
            free_bytes: self
                .object_store
                .free_bytes
                .min(self.object_store.total_bytes),
        }
    }

    pub fn volume_info_for_path(&self, guest_path: Option<&str>) -> VolumeInfo {
        if let Some(mount) = guest_path.and_then(|path| self.mount_for_guest_path(path)) {
            let mut attributes = 0;
            if !mount.writable {
                attributes |= CE_VOLUME_ATTRIBUTE_READONLY;
            }
            if mount.hidden {
                attributes |= CE_VOLUME_ATTRIBUTE_HIDDEN;
            }
            if mount.removable {
                attributes |= CE_VOLUME_ATTRIBUTE_REMOVABLE;
            }
            if mount.system {
                attributes |= CE_VOLUME_ATTRIBUTE_SYSTEM;
            }
            let name = mount
                .name
                .as_deref()
                .map(str::to_owned)
                .unwrap_or_else(|| volume_name_from_guest_root(&mount.guest_root));
            return VolumeInfo {
                attributes,
                flags: CE_VOLUME_FLAG_STORE,
                block_size: CE_VOLUME_DEFAULT_BLOCK_SIZE,
                store_name: name.clone(),
                partition_name: name,
            };
        }
        VolumeInfo {
            attributes: 0,
            flags: CE_VOLUME_FLAG_RAMFS,
            block_size: CE_VOLUME_DEFAULT_BLOCK_SIZE,
            store_name: "ObjectStore".to_owned(),
            partition_name: "ObjectStore".to_owned(),
        }
    }

    pub fn mount_table_volume_info_for_guest_root(
        &self,
        guest_root: &str,
    ) -> Option<MountTableVolumeInfo> {
        let guest_root = normalize_guest_path(guest_root);
        let mount = self.mounts.get(&guest_root)?;
        let mut flags = 0;
        if mount.hidden {
            flags |= AFS_FLAG_HIDDEN;
        }
        if mount.system {
            flags |= AFS_FLAG_SYSTEM;
        }
        if !mount.removable {
            flags |= AFS_FLAG_PERMANENT;
        }
        Some(MountTableVolumeInfo {
            name: volume_name_from_guest_root(&mount.guest_root),
            flags,
        })
    }

    pub fn io_stats(&self) -> FileIoStats {
        self.io_stats
    }

    pub fn volume_root_for_guest_path(&self, guest_path: &str) -> Option<String> {
        let volume_key = self.volume_for_guest_path(guest_path).volume_key;
        (!volume_key.is_empty()).then(|| format!("\\{}", volume_key.replace('/', "\\")))
    }

    pub fn set_root_relative_guest_path(&mut self, guest_path: &str) {
        let normalized = normalize_guest_path(guest_path);
        self.root_relative_mount = self
            .mount_for_normalized_path(&normalized)
            .map(|mount| mount.guest_root.clone());
    }

    pub fn host_path_to_guest_mount(&self, host_path: &Path) -> Option<String> {
        for mount in self.mounts_in_order() {
            let Some(host_root) = mount.host_root.as_ref() else {
                continue;
            };
            let Some(relative) = strip_host_prefix(host_path, host_root) else {
                continue;
            };
            let mut guest_path = format!("\\{}", mount.guest_root.replace('/', "\\"));
            for component in relative.components() {
                let Component::Normal(part) = component else {
                    continue;
                };
                guest_path.push('\\');
                guest_path.push_str(&part.to_string_lossy());
            }
            return Some(guest_path);
        }
        None
    }

    pub fn create_file_w(
        &mut self,
        guest_path: &str,
        desired_access: u32,
        creation_disposition: u32,
    ) -> Result<u32> {
        let mount = self.mount_for_guest_path(guest_path);
        if desired_access & GENERIC_WRITE != 0 && mount.is_some_and(|mount| !mount.writable) {
            return Err(Error::AccessDenied(format!(
                "guest mount is read-only: {guest_path}"
            )));
        }
        let host_path = self.translate_guest_path(guest_path)?;
        let exists = host_path.exists();
        let is_directory = host_path.is_dir();
        let requested_writable = desired_access & GENERIC_WRITE != 0 && !is_directory;

        let (backing, file_len, writable) = if is_directory {
            match creation_disposition {
                OPEN_EXISTING | OPEN_ALWAYS => (FileBacking::Memory(Vec::new()), 0, false),
                _ => {
                    return Err(Error::InvalidArgument(format!(
                        "cannot create or truncate directory: {guest_path}"
                    )));
                }
            }
        } else {
            match creation_disposition {
                CREATE_NEW if exists => {
                    return Err(Error::InvalidArgument(format!(
                        "file already exists: {guest_path}"
                    )));
                }
                CREATE_NEW | CREATE_ALWAYS => {
                    self.invalidate_read_only_cache(&host_path);
                    let file =
                        create_host_file(&host_path, creation_disposition == CREATE_NEW, true)?;
                    (FileBacking::HostFile(file), 0, requested_writable)
                }
                OPEN_EXISTING if !exists => {
                    return Err(Error::InvalidArgument(format!(
                        "file does not exist: {guest_path}"
                    )));
                }
                OPEN_EXISTING | OPEN_ALWAYS if exists => {
                    let metadata = fs::metadata(&host_path).map_err(|source| Error::Io {
                        path: host_path.clone(),
                        source,
                    })?;
                    let file_len = metadata.len().try_into().unwrap_or(usize::MAX);
                    if !requested_writable {
                        if let Some(backing) =
                            self.read_only_memory_backing(&host_path, &metadata, file_len)?
                        {
                            (backing, file_len, false)
                        } else {
                            let (file, writable) =
                                open_existing_host_file(&host_path, requested_writable)?;
                            (FileBacking::HostFile(file), file_len, writable)
                        }
                    } else {
                        self.invalidate_read_only_cache(&host_path);
                        let (file, writable) =
                            open_existing_host_file(&host_path, requested_writable)?;
                        (FileBacking::HostFile(file), file_len, writable)
                    }
                }
                OPEN_ALWAYS => {
                    self.invalidate_read_only_cache(&host_path);
                    let file = create_host_file(&host_path, false, false)?;
                    (FileBacking::HostFile(file), 0, requested_writable)
                }
                TRUNCATE_EXISTING if exists && requested_writable => {
                    self.invalidate_read_only_cache(&host_path);
                    let file = create_host_file(&host_path, false, true)?;
                    (FileBacking::HostFile(file), 0, true)
                }
                TRUNCATE_EXISTING if !exists => {
                    return Err(Error::InvalidArgument(format!(
                        "file does not exist: {guest_path}"
                    )));
                }
                TRUNCATE_EXISTING => {
                    return Err(Error::InvalidArgument(format!(
                        "file is not writable: {guest_path}"
                    )));
                }
                _ => {
                    return Err(Error::InvalidArgument(format!(
                        "unsupported creation disposition {creation_disposition}"
                    )));
                }
            }
        };
        let is_memory_backed = matches!(
            backing,
            FileBacking::Memory(_) | FileBacking::ReadOnlyMemory(_)
        );
        let is_host_file_backed = matches!(backing, FileBacking::HostFile(_));

        let id = self.next_id;
        self.next_id += 1;
        self.open_files.insert(
            id,
            OpenFile {
                id,
                guest_path: guest_path.to_owned(),
                host_path,
                cursor: 0,
                backing,
                read_cache: Vec::new(),
                read_cache_start: 0,
                file_len,
                desired_access,
                writable,
                dirty: matches!(
                    creation_disposition,
                    CREATE_NEW | CREATE_ALWAYS | TRUNCATE_EXISTING
                ),
                eof: false,
            },
        );
        if is_host_file_backed {
            self.io_stats.host_file_open_count += 1;
        }
        if is_memory_backed {
            self.io_stats.memory_backed_open_count += 1;
        }
        Ok(id)
    }

    pub fn read_file(&mut self, id: u32, requested: u32) -> Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(requested as usize);
        self.read_file_into(id, requested, |chunk| {
            bytes.extend_from_slice(chunk);
            Ok(())
        })?;
        Ok(bytes)
    }

    pub fn read_file_into<F>(&mut self, id: u32, requested: u32, mut write: F) -> Result<u32>
    where
        F: FnMut(&[u8]) -> Result<()>,
    {
        let (transferred, host_read) = {
            let file = self.open_file_mut(id)?;
            file.read_into(requested as usize, &mut write)?
        };
        self.io_stats.max_read_request = self.io_stats.max_read_request.max(requested);
        if host_read {
            self.io_stats.host_file_read_count += 1;
            self.io_stats.host_file_read_bytes += u64::from(transferred);
        }
        Ok(transferred)
    }

    pub fn read_at(&mut self, id: u32, offset: usize, requested: usize) -> Result<Vec<u8>> {
        let (bytes, host_read) = {
            let file = self.open_file_mut(id)?;
            file.read_at(offset, requested)?
        };
        self.io_stats.max_read_request = self.io_stats.max_read_request.max(requested as u32);
        if host_read {
            self.io_stats.host_file_read_count += 1;
            self.io_stats.host_file_read_bytes += bytes.len() as u64;
        }
        Ok(bytes)
    }

    pub fn read_guest_file(&self, guest_path: &str) -> Result<Vec<u8>> {
        let host_path = self.translate_guest_path(guest_path)?;
        fs::read(&host_path).map_err(|source| Error::Io {
            path: host_path,
            source,
        })
    }

    pub fn file_attributes_w(&self, guest_path: &str) -> Result<FindData> {
        let normalized = normalize_guest_path(guest_path);
        if let Some(entry) = self.root_mount_entry(guest_path, &normalized) {
            return Ok(entry);
        }

        let host_path = self.translate_guest_path(guest_path)?;
        let file_name = host_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut data = find_data_from_path(&host_path, file_name)?;
        self.apply_mount_attributes(&normalized, &mut data);
        Ok(data)
    }

    pub fn create_directory_w(&self, guest_path: &str) -> Result<()> {
        if self.volume_for_guest_path(guest_path).is_mount_root() {
            return Err(Error::AlreadyExists(format!(
                "guest mount root already exists: {guest_path}"
            )));
        }
        if self
            .mount_for_guest_path(guest_path)
            .is_some_and(|mount| !mount.writable)
        {
            return Err(Error::AccessDenied(format!(
                "guest mount is read-only: {guest_path}"
            )));
        }
        let host_path = self.translate_guest_path(guest_path)?;
        fs::create_dir_all(&host_path).map_err(|source| Error::Io {
            path: host_path,
            source,
        })
    }

    pub fn remove_directory_w(&self, guest_path: &str) -> Result<()> {
        if self.volume_for_guest_path(guest_path).is_mount_root() {
            return Err(Error::AccessDenied(format!(
                "cannot remove guest mount root: {guest_path}"
            )));
        }
        if self
            .mount_for_guest_path(guest_path)
            .is_some_and(|mount| !mount.writable)
        {
            return Err(Error::AccessDenied(format!(
                "guest mount is read-only: {guest_path}"
            )));
        }
        let host_path = self.translate_guest_path(guest_path)?;
        fs::remove_dir(&host_path).map_err(|source| Error::Io {
            path: host_path,
            source,
        })
    }

    pub fn delete_file_w(&mut self, guest_path: &str) -> Result<()> {
        if self.volume_for_guest_path(guest_path).is_mount_root() {
            return Err(Error::InvalidArgument(format!(
                "cannot delete guest mount root as a file: {guest_path}"
            )));
        }
        if self
            .mount_for_guest_path(guest_path)
            .is_some_and(|mount| !mount.writable)
        {
            return Err(Error::AccessDenied(format!(
                "guest mount is read-only: {guest_path}"
            )));
        }
        let host_path = self.translate_guest_path(guest_path)?;
        self.invalidate_read_only_cache(&host_path);
        fs::remove_file(&host_path).map_err(|source| Error::Io {
            path: host_path,
            source,
        })
    }

    pub fn move_file_w(&mut self, existing_path: &str, new_path: &str) -> Result<()> {
        let existing_volume = self.volume_for_guest_path(existing_path);
        let new_volume = self.volume_for_guest_path(new_path);
        if existing_volume.is_mount_root() {
            return Err(Error::AccessDenied(format!(
                "cannot rename guest mount point: {existing_path}"
            )));
        }
        if new_volume.is_mount_root() {
            return Err(Error::AlreadyExists(format!(
                "cannot rename over guest mount point: {new_path}"
            )));
        }
        let existing = self.translate_guest_path(existing_path)?;
        let new = self.translate_guest_path(new_path)?;
        self.invalidate_read_only_cache(&existing);
        self.invalidate_read_only_cache(&new);
        let existing_metadata = existing.metadata().map_err(|source| Error::Io {
            path: existing.clone(),
            source,
        })?;
        let cross_volume = existing_volume.volume_key != new_volume.volume_key;
        let existing_readonly = self
            .mount_for_normalized_path(&existing_volume.normalized_path)
            .is_some_and(|mount| !mount.writable);
        let new_readonly = self
            .mount_for_normalized_path(&new_volume.normalized_path)
            .is_some_and(|mount| !mount.writable);
        if new_readonly || (!cross_volume && existing_readonly) {
            return Err(Error::AccessDenied(format!(
                "guest mount is read-only: {existing_path} -> {new_path}"
            )));
        }
        if cross_volume && existing_metadata.is_dir() {
            return Err(Error::NotSameDevice(format!(
                "cannot move directory across guest volumes: {existing_path} -> {new_path}"
            )));
        }
        if let Some(parent) = new.parent() {
            fs::create_dir_all(parent).map_err(|source| Error::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        if cross_volume {
            if new.exists() {
                return Err(Error::AlreadyExists(format!(
                    "destination exists: {new_path}"
                )));
            }
            fs::copy(&existing, &new).map_err(|source| Error::Io {
                path: existing.clone(),
                source,
            })?;
            if !existing_readonly {
                let _ = fs::remove_file(&existing);
            }
            return Ok(());
        }
        fs::rename(&existing, &new).map_err(|source| Error::Io {
            path: existing,
            source,
        })
    }

    pub fn delete_and_rename_file_w(&mut self, old_path: &str, new_path: &str) -> Result<()> {
        let old_volume = self.volume_for_guest_path(old_path);
        let new_volume = self.volume_for_guest_path(new_path);
        if old_volume.is_mount_root() || new_volume.is_mount_root() {
            return Err(Error::AccessDenied(format!(
                "cannot delete and rename guest mount point: {old_path} -> {new_path}"
            )));
        }
        if old_volume.volume_key != new_volume.volume_key {
            return Err(Error::NotSameDevice(format!(
                "cannot delete and rename across guest volumes: {old_path} -> {new_path}"
            )));
        }
        if self
            .mount_for_normalized_path(&old_volume.normalized_path)
            .is_some_and(|mount| !mount.writable)
        {
            return Err(Error::AccessDenied(format!(
                "guest mount is read-only: {old_path} -> {new_path}"
            )));
        }
        let old = self.translate_guest_path(old_path)?;
        let new = self.translate_guest_path(new_path)?;
        self.invalidate_read_only_cache(&old);
        self.invalidate_read_only_cache(&new);
        let new_metadata = new.metadata().map_err(|source| Error::Io {
            path: new.clone(),
            source,
        })?;
        if new_metadata.is_dir() {
            return Err(Error::InvalidArgument(format!(
                "replacement source is a directory: {new_path}"
            )));
        }
        fs::remove_file(&old).map_err(|source| Error::Io {
            path: old.clone(),
            source,
        })?;
        fs::rename(&new, &old).map_err(|source| Error::Io { path: new, source })
    }

    pub fn copy_file_w(
        &mut self,
        existing_path: &str,
        new_path: &str,
        fail_if_exists: bool,
    ) -> Result<()> {
        if self
            .mount_for_guest_path(new_path)
            .is_some_and(|mount| !mount.writable)
        {
            return Err(Error::AccessDenied(format!(
                "guest mount is read-only: {new_path}"
            )));
        }
        let existing = self.translate_guest_path(existing_path)?;
        let new = self.translate_guest_path(new_path)?;
        self.invalidate_read_only_cache(&new);
        if fail_if_exists && new.exists() {
            return Err(Error::InvalidArgument(format!(
                "destination exists: {new_path}"
            )));
        }
        if let Some(parent) = new.parent() {
            fs::create_dir_all(parent).map_err(|source| Error::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::copy(&existing, &new)
            .map(|_| ())
            .map_err(|source| Error::Io {
                path: existing,
                source,
            })
    }

    pub fn set_file_attributes_w(&self, guest_path: &str, attributes: u32) -> Result<()> {
        if self.volume_for_guest_path(guest_path).is_mount_root() {
            return Err(Error::AccessDenied(format!(
                "cannot set attributes on guest mount root: {guest_path}"
            )));
        }
        if self
            .mount_for_guest_path(guest_path)
            .is_some_and(|mount| !mount.writable)
        {
            return Err(Error::AccessDenied(format!(
                "guest mount is read-only: {guest_path}"
            )));
        }
        let host_path = self.translate_guest_path(guest_path)?;
        let mut permissions = fs::metadata(&host_path)
            .map_err(|source| Error::Io {
                path: host_path.clone(),
                source,
            })?
            .permissions();
        permissions.set_readonly(attributes & FILE_ATTRIBUTE_READONLY != 0);
        fs::set_permissions(&host_path, permissions).map_err(|source| Error::Io {
            path: host_path,
            source,
        })
    }

    pub fn write_file(&mut self, id: u32, bytes: &[u8]) -> Result<FileIoResult> {
        let host_path = {
            let file = self.open_file_mut(id)?;
            if !file.writable {
                return Ok(FileIoResult {
                    success: false,
                    bytes_transferred: 0,
                });
            }

            file.write_current(bytes)?;
            file.host_path.clone()
        };
        self.invalidate_read_only_cache(&host_path);
        Ok(FileIoResult {
            success: true,
            bytes_transferred: bytes.len() as u32,
        })
    }

    pub fn write_at(&mut self, id: u32, offset: usize, bytes: &[u8]) -> Result<FileIoResult> {
        let host_path = {
            let file = self.open_file_mut(id)?;
            if !file.writable {
                return Ok(FileIoResult {
                    success: false,
                    bytes_transferred: 0,
                });
            }

            file.write_at(offset, bytes)?;
            file.host_path.clone()
        };
        self.invalidate_read_only_cache(&host_path);
        Ok(FileIoResult {
            success: true,
            bytes_transferred: bytes.len() as u32,
        })
    }

    pub fn set_file_pointer(&mut self, id: u32, position: usize) -> Result<usize> {
        let file = self.open_file_mut(id)?;
        file.cursor = position;
        file.eof = false;
        Ok(file.cursor)
    }

    pub fn lock_file_range(
        &mut self,
        id: u32,
        start: u64,
        length: u64,
        exclusive: bool,
    ) -> Result<FileLockStatus> {
        let Some(finish) = file_lock_finish(start, length) else {
            return Ok(FileLockStatus::InvalidParameter);
        };
        let file = self.open_file(id)?;
        if file.desired_access & (GENERIC_READ | GENERIC_WRITE) == 0 {
            return Ok(FileLockStatus::InvalidParameter);
        }
        let host_path = file.host_path.clone();
        let locks = self.file_locks.entry(host_path).or_default();
        let conflicts = locks.iter().any(|lock| {
            file_lock_ranges_overlap(start, finish, lock.start, lock.finish)
                && (lock.exclusive || exclusive)
        });
        if conflicts {
            return Ok(FileLockStatus::Conflict);
        }
        locks.push(FileLock {
            owner_file_id: id,
            start,
            finish,
            exclusive,
        });
        Ok(FileLockStatus::Success)
    }

    pub fn unlock_file_range(
        &mut self,
        id: u32,
        start: u64,
        length: u64,
    ) -> Result<FileLockStatus> {
        let Some(finish) = file_lock_finish(start, length) else {
            return Ok(FileLockStatus::InvalidParameter);
        };
        let host_path = self.open_file(id)?.host_path.clone();
        let Some(locks) = self.file_locks.get_mut(&host_path) else {
            return Ok(FileLockStatus::InvalidParameter);
        };
        let Some(index) = locks.iter().position(|lock| {
            lock.owner_file_id == id && lock.start == start && lock.finish == finish
        }) else {
            return Ok(FileLockStatus::InvalidParameter);
        };
        locks.remove(index);
        if locks.is_empty() {
            self.file_locks.remove(&host_path);
        }
        Ok(FileLockStatus::Success)
    }

    pub fn unlock_file_ranges_owned_by_id(&mut self, id: u32) -> Result<FileLockStatus> {
        self.open_file(id)?;
        self.unlock_file_ranges_owned_by_any_path(id);
        Ok(FileLockStatus::Success)
    }

    pub fn test_file_lock_range(
        &self,
        id: u32,
        start: u64,
        length: u64,
        read: bool,
    ) -> Result<FileLockStatus> {
        let Some(finish) = file_lock_finish(start, length) else {
            return Ok(FileLockStatus::InvalidParameter);
        };
        let file = self.open_file(id)?;
        let Some(locks) = self.file_locks.get(&file.host_path) else {
            return Ok(FileLockStatus::Success);
        };
        let conflicts = locks.iter().any(|lock| {
            lock.owner_file_id != id
                && file_lock_ranges_overlap(start, finish, lock.start, lock.finish)
                && (!read || lock.exclusive)
        });
        Ok(if conflicts {
            FileLockStatus::Conflict
        } else {
            FileLockStatus::Success
        })
    }

    pub fn file_cursor(&self, id: u32) -> Result<usize> {
        Ok(self.open_file(id)?.cursor)
    }

    pub fn file_is_eof(&self, id: u32) -> Result<bool> {
        Ok(self.open_file(id)?.is_eof())
    }

    pub fn file_size(&self, id: u32) -> Result<usize> {
        Ok(self.open_file(id)?.file_len)
    }

    /// Truncates or extends the file to the current file-pointer position,
    /// filling any extension with zeros.  Returns `false` if the file is not
    /// open for writing.
    pub fn set_end_of_file(&mut self, id: u32) -> Result<bool> {
        let host_path = {
            let file = self.open_file_mut(id)?;
            if !file.writable {
                return Ok(false);
            }
            let new_len = file.cursor;
            match &mut file.backing {
                FileBacking::Memory(data) => {
                    data.resize(new_len, 0);
                }
                FileBacking::ReadOnlyMemory(_) => {
                    return Ok(false);
                }
                FileBacking::HostFile(f) => {
                    f.set_len(new_len as u64).map_err(|source| Error::Io {
                        path: file.host_path.clone(),
                        source,
                    })?;
                }
            }
            file.file_len = new_len;
            file.read_cache.clear();
            file.read_cache_start = 0;
            file.dirty = true;
            file.host_path.clone()
        };
        self.invalidate_read_only_cache(&host_path);
        Ok(true)
    }

    /// Returns the host file attributes (FILE_ATTRIBUTE_*) for the file
    /// backing the given open file id.
    pub fn file_attributes_by_id(&self, id: u32) -> Result<u32> {
        let file = self.open_file(id)?;
        let host = &file.host_path;
        let meta = host.metadata().map_err(|source| Error::Io {
            path: host.clone(),
            source,
        })?;
        let mut attr = FILE_ATTRIBUTE_ARCHIVE;
        if meta.permissions().readonly() {
            attr |= FILE_ATTRIBUTE_READONLY;
        }
        Ok(attr)
    }

    /// Returns (creation_time, last_access_time, last_write_time) as Windows
    /// FILETIME values (100-ns intervals since 1601-01-01).  Falls back to
    /// zero for timestamps the host OS does not provide.
    pub fn file_times_by_id(&self, id: u32) -> Result<(u64, u64, u64)> {
        let file = self.open_file(id)?;
        let meta = file.host_path.metadata().map_err(|source| Error::Io {
            path: file.host_path.clone(),
            source,
        })?;
        const EPOCH_DIFF_100NS: u64 = 116_444_736_000_000_000;
        let system_time_to_filetime = |st: std::time::SystemTime| -> u64 {
            match st.duration_since(std::time::UNIX_EPOCH) {
                Ok(d) => {
                    let secs = d.as_secs();
                    let nanos = d.subsec_nanos() as u64;
                    EPOCH_DIFF_100NS
                        .saturating_add(secs.saturating_mul(10_000_000))
                        .saturating_add(nanos / 100)
                }
                Err(_) => 0,
            }
        };
        let write_time = meta.modified().map(system_time_to_filetime).unwrap_or(0);
        let access_time = meta
            .accessed()
            .map(system_time_to_filetime)
            .unwrap_or(write_time);
        let create_time = {
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                let ft = meta.creation_time();
                ft
            }
            #[cfg(not(windows))]
            {
                0u64
            }
        };
        Ok((create_time, access_time, write_time))
    }

    pub fn flush(&mut self, id: u32) -> Result<()> {
        let file = self.open_file_mut(id)?;
        if file.dirty {
            file.flush()?;
        }
        Ok(())
    }

    pub fn close(&mut self, id: u32) -> Result<()> {
        self.flush(id)?;
        let removed = self
            .open_files
            .remove(&id)
            .ok_or(Error::InvalidHandle(id))?;
        self.unlock_file_ranges_owned_by(id, &removed.host_path);
        Ok(())
    }

    pub fn duplicate_open_file(&mut self, id: u32) -> Result<u32> {
        let mut duplicate = self
            .open_files
            .get(&id)
            .ok_or(Error::InvalidHandle(id))?
            .clone();
        let duplicate_id = self.next_id;
        self.next_id += 1;
        duplicate.id = duplicate_id;
        if duplicate.is_host_file_backed() {
            self.io_stats.host_file_open_count += 1;
        }
        if duplicate.is_memory_backed() {
            self.io_stats.memory_backed_open_count += 1;
        }
        self.open_files.insert(duplicate_id, duplicate);
        Ok(duplicate_id)
    }

    pub fn find_first_file_w(&mut self, guest_pattern: &str) -> Result<(u32, FindData)> {
        let entries = self.find_matches(guest_pattern)?;
        let Some(first) = entries.first().cloned() else {
            return Err(Error::InvalidArgument(format!(
                "no files match: {guest_pattern}"
            )));
        };

        let id = self.next_id;
        self.next_id += 1;
        self.open_finds.insert(
            id,
            OpenFind {
                id,
                guest_pattern: guest_pattern.to_owned(),
                entries,
                cursor: 0,
            },
        );
        Ok((id, first))
    }

    pub fn find_next_file_w(&mut self, id: u32) -> Result<Option<FindData>> {
        let find = self
            .open_finds
            .get_mut(&id)
            .ok_or(Error::InvalidHandle(id))?;
        find.cursor = find.cursor.saturating_add(1);
        Ok(find.entries.get(find.cursor).cloned())
    }

    pub fn find_close(&mut self, id: u32) -> Result<()> {
        self.open_finds
            .remove(&id)
            .ok_or(Error::InvalidHandle(id))
            .map(|_| ())
    }

    pub fn duplicate_find(&mut self, id: u32) -> Result<u32> {
        let mut duplicate = self
            .open_finds
            .get(&id)
            .ok_or(Error::InvalidHandle(id))?
            .clone();
        let duplicate_id = self.next_id;
        self.next_id += 1;
        duplicate.id = duplicate_id;
        self.open_finds.insert(duplicate_id, duplicate);
        Ok(duplicate_id)
    }

    pub fn open_file(&self, id: u32) -> Result<&OpenFile> {
        self.open_files.get(&id).ok_or(Error::InvalidHandle(id))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn host_path_for_guest(&self, guest_path: &str) -> Result<PathBuf> {
        self.translate_guest_path(guest_path)
    }

    pub fn routes_security_path(&self, guest_path: &str) -> Result<bool> {
        let normalized = normalize_guest_path(guest_path);
        if normalized.is_empty() {
            return if is_root_relative_path(guest_path) {
                Ok(true)
            } else {
                Err(Error::InvalidArgument("empty guest path".to_owned()))
            };
        }
        if self.mount_for_normalized_path(&normalized).is_some() {
            return Ok(true);
        }
        self.translate_guest_path(guest_path).map(|_| true)
    }

    fn open_file_mut(&mut self, id: u32) -> Result<&mut OpenFile> {
        self.open_files.get_mut(&id).ok_or(Error::InvalidHandle(id))
    }

    fn read_only_memory_backing(
        &mut self,
        host_path: &Path,
        metadata: &fs::Metadata,
        file_len: usize,
    ) -> Result<Option<FileBacking>> {
        if !(READ_ONLY_MEMORY_BACKING_MIN_BYTES..=READ_ONLY_MEMORY_BACKING_MAX_BYTES)
            .contains(&file_len)
        {
            return Ok(None);
        }

        let modified = metadata.modified().ok();
        if let Some(entry) = self.read_only_cache.get(host_path) {
            if entry.len == file_len && entry.modified == modified {
                return Ok(Some(FileBacking::ReadOnlyMemory(Arc::clone(&entry.bytes))));
            }
        }

        let bytes = fs::read(host_path).map_err(|source| Error::Io {
            path: host_path.to_path_buf(),
            source,
        })?;
        let entry = ReadOnlyCacheEntry {
            len: bytes.len(),
            modified,
            bytes: Arc::new(bytes),
        };
        let backing = FileBacking::ReadOnlyMemory(Arc::clone(&entry.bytes));
        self.read_only_cache.insert(host_path.to_path_buf(), entry);
        Ok(Some(backing))
    }

    fn invalidate_read_only_cache(&mut self, host_path: &Path) {
        self.read_only_cache.remove(host_path);
    }

    fn unlock_file_ranges_owned_by(&mut self, id: u32, host_path: &Path) {
        if let Some(locks) = self.file_locks.get_mut(host_path) {
            locks.retain(|lock| lock.owner_file_id != id);
            if locks.is_empty() {
                self.file_locks.remove(host_path);
            }
        }
    }

    fn unlock_file_ranges_owned_by_any_path(&mut self, id: u32) {
        self.file_locks.retain(|_, locks| {
            locks.retain(|lock| lock.owner_file_id != id);
            !locks.is_empty()
        });
    }

    fn translate_guest_path(&self, guest_path: &str) -> Result<PathBuf> {
        let normalized = normalize_guest_path(guest_path);

        if normalized.is_empty() {
            if is_root_relative_path(guest_path) {
                return Ok(self.root.clone());
            }
            return Err(Error::InvalidArgument("empty guest path".to_owned()));
        }

        for mount in self.mounts_longest_first() {
            let remainder = mount_remainder(&normalized, &mount.guest_root);
            if let Some(remainder) = remainder {
                let Some(host_root) = mount.host_root.as_ref() else {
                    return Err(Error::InvalidArgument(format!(
                        "virtual guest mount has no host files: {guest_path}"
                    )));
                };
                return join_normalized_host_path(host_root, remainder, guest_path);
            }
        }

        if is_root_relative_path(guest_path) {
            if let Some(host_root) = self.root_relative_host_root() {
                return join_normalized_host_path(host_root, &normalized, guest_path);
            }
        }

        join_normalized_host_path(&self.root, &normalized, guest_path)
    }

    fn find_matches(&self, guest_pattern: &str) -> Result<Vec<FindData>> {
        let normalized = normalize_guest_path(guest_pattern);
        if self.is_root_namespace_pattern(guest_pattern, &normalized) {
            return Ok(self.root_namespace_entries(&normalized));
        }
        if let Some(entry) = self.root_mount_entry(guest_pattern, &normalized) {
            return Ok(vec![entry]);
        }
        if self
            .mount_for_normalized_path(&normalized)
            .is_some_and(|mount| mount.host_root.is_none())
        {
            return Ok(Vec::new());
        }

        let host_pattern = self.translate_guest_path(guest_pattern)?;
        let pattern_name = host_pattern
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        tracing::debug!(
            target: "ce.file",
            guest_pattern,
            host_pattern = %host_pattern.display(),
            pattern_name,
            "translated find pattern"
        );

        let system_volume = self
            .mount_for_normalized_path(&normalized)
            .is_some_and(|mount| mount.system);

        if has_wildcards(&pattern_name) {
            let dir = host_pattern.parent().unwrap_or_else(|| Path::new("."));
            let mut entries = Vec::new();
            for entry in fs::read_dir(dir).map_err(|source| Error::Io {
                path: dir.to_path_buf(),
                source,
            })? {
                let entry = entry.map_err(|source| Error::Io {
                    path: dir.to_path_buf(),
                    source,
                })?;
                let file_name = entry.file_name().to_string_lossy().into_owned();
                if wildcard_match(&pattern_name, &file_name) {
                    let mut data = find_data_from_path(&entry.path(), file_name)?;
                    if system_volume {
                        data.attributes |= FILE_ATTRIBUTE_SYSTEM;
                    }
                    entries.push(data);
                }
            }
            entries.sort_by(|lhs, rhs| lhs.file_name.cmp(&rhs.file_name));
            tracing::debug!(
                target: "ce.file",
                guest_pattern,
                dir = %dir.display(),
                pattern_name,
                matches = entries.len(),
                "enumerated find pattern"
            );
            return Ok(entries);
        }

        if !host_pattern.exists() {
            tracing::debug!(
                target: "ce.file",
                guest_pattern,
                host_pattern = %host_pattern.display(),
                "find pattern had no exact match"
            );
            return Ok(Vec::new());
        }
        let file_name = host_pattern
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        tracing::debug!(
            target: "ce.file",
            guest_pattern,
            host_pattern = %host_pattern.display(),
            "find pattern matched exact path"
        );
        let mut data = find_data_from_path(&host_pattern, file_name)?;
        if system_volume {
            data.attributes |= FILE_ATTRIBUTE_SYSTEM;
        }
        Ok(vec![data])
    }

    fn root_namespace_entries(&self, pattern: &str) -> Vec<FindData> {
        let mut entries = Vec::new();
        let mut seen_names = Vec::new();
        for mount in self.mounts_in_order() {
            if mount.hidden {
                continue;
            }
            let file_name = mount
                .guest_root
                .rsplit('/')
                .next()
                .unwrap_or(&mount.guest_root);
            if !pattern.is_empty() && !wildcard_match(pattern, file_name) {
                continue;
            }
            seen_names.push(file_name.to_ascii_lowercase());
            entries.push(mount_root_find_data(mount, file_name.to_owned()));
        }
        if let Ok(host_entries) = root_host_find_data(&self.root, pattern) {
            for entry in host_entries {
                let lower_name = entry.file_name.to_ascii_lowercase();
                if seen_names.iter().any(|seen| seen == &lower_name) {
                    continue;
                }
                seen_names.push(lower_name);
                entries.push(entry);
            }
        }
        tracing::debug!(
            target: "ce.file",
            guest_pattern = pattern,
            matches = entries.len(),
            "enumerated root mount namespace"
        );
        entries
    }

    fn root_relative_host_root(&self) -> Option<&Path> {
        let guest_root = self.root_relative_mount.as_ref()?;
        self.mounts_in_order()
            .find(|mount| mount.guest_root.eq_ignore_ascii_case(guest_root))
            .and_then(|mount| mount.host_root.as_deref())
    }

    fn mount_for_guest_path(&self, guest_path: &str) -> Option<&FileMount> {
        let normalized = normalize_guest_path(guest_path);
        self.mount_for_normalized_path(&normalized)
    }

    fn mount_for_normalized_path(&self, normalized: &str) -> Option<&FileMount> {
        self.mounts_longest_first()
            .find(|mount| mount_remainder(normalized, &mount.guest_root).is_some())
    }

    fn volume_for_guest_path(&self, guest_path: &str) -> GuestVolumePath {
        let normalized_path = normalize_guest_path(guest_path);
        let volume_key = self
            .mount_for_normalized_path(&normalized_path)
            .map(|mount| mount.guest_root.clone())
            .unwrap_or_default();
        GuestVolumePath {
            is_mount_root: !volume_key.is_empty()
                && normalized_path.eq_ignore_ascii_case(&volume_key),
            normalized_path,
            volume_key,
        }
    }

    fn is_root_namespace_pattern(&self, guest_pattern: &str, normalized: &str) -> bool {
        normalized.is_empty()
            || is_root_relative_path(guest_pattern)
                && !normalized.contains('/')
                && has_wildcards(normalized)
                && self.root_mount_entry(guest_pattern, normalized).is_none()
    }

    fn root_mount_entry(&self, guest_pattern: &str, normalized: &str) -> Option<FindData> {
        if !is_root_relative_path(guest_pattern)
            || normalized.contains('/')
            || has_wildcards(normalized)
        {
            return None;
        }

        self.mounts_in_order()
            .find(|mount| mount.guest_root.eq_ignore_ascii_case(normalized))
            .map(|mount| {
                let file_name = mount
                    .guest_root
                    .rsplit('/')
                    .next()
                    .unwrap_or(&mount.guest_root)
                    .to_owned();
                mount_root_find_data(mount, file_name)
            })
    }

    fn apply_mount_attributes(&self, normalized: &str, data: &mut FindData) {
        if self
            .mount_for_normalized_path(normalized)
            .is_some_and(|mount| mount.system)
        {
            data.attributes |= FILE_ATTRIBUTE_SYSTEM;
        }
    }

    fn mounts_in_order(&self) -> impl Iterator<Item = &FileMount> {
        self.mount_order
            .iter()
            .filter_map(|guest_root| self.mounts.get(guest_root))
    }

    fn mounts_longest_first(&self) -> impl Iterator<Item = &FileMount> {
        let mut mounts: Vec<_> = self.mounts_in_order().collect();
        mounts.sort_by(|lhs, rhs| {
            rhs.guest_root
                .len()
                .cmp(&lhs.guest_root.len())
                .then_with(|| lhs.guest_root.cmp(&rhs.guest_root))
        });
        mounts.into_iter()
    }
}

fn mount_root_find_data(mount: &FileMount, file_name: String) -> FindData {
    let mut attributes = FILE_ATTRIBUTE_DIRECTORY;
    if mount.removable {
        attributes |= FILE_ATTRIBUTE_TEMPORARY;
    }
    if mount.system {
        attributes |= FILE_ATTRIBUTE_SYSTEM;
    }
    if mount.hidden {
        attributes |= FILE_ATTRIBUTE_HIDDEN;
    }
    FindData {
        attributes,
        file_size: 0,
        file_name,
    }
}

fn root_host_find_data(root: &Path, pattern: &str) -> Result<Vec<FindData>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(root).map_err(|source| Error::Io {
        path: root.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| Error::Io {
            path: root.to_path_buf(),
            source,
        })?;
        let file_name = entry.file_name().to_string_lossy().into_owned();
        if !pattern.is_empty() && !wildcard_match(pattern, &file_name) {
            continue;
        }
        entries.push(find_data_from_path(&entry.path(), file_name)?);
    }
    entries.sort_by(|lhs, rhs| lhs.file_name.cmp(&rhs.file_name));
    Ok(entries)
}

fn volume_name_from_guest_root(guest_root: &str) -> String {
    guest_root
        .trim_matches('/')
        .rsplit('/')
        .find(|part| !part.is_empty())
        .unwrap_or("ObjectStore")
        .to_owned()
}

fn device_interface_advertisement_spec(
    mount: &FileMount,
) -> Option<DeviceInterfaceAdvertisementSpec> {
    if mount.interface_classes.is_empty() {
        return None;
    }
    let device_name = mount
        .device_name
        .as_deref()
        .or(mount.name.as_deref())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| volume_name_from_guest_root(&mount.guest_root));
    let legacy_name = device_name.trim_matches(['\\', '/']).to_owned();
    let bus_name = mount
        .bus_name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(|name| name.trim_matches(['\\', '/']).to_owned());
    let default_device_path = format!("\\StoreMgr\\{legacy_name}");
    let interfaces: Vec<_> = mount
        .interface_classes
        .iter()
        .filter_map(|entry| {
            device_interface_class_advertisement_spec(
                entry,
                &default_device_path,
                &legacy_name,
                bus_name.as_deref(),
            )
        })
        .collect();
    if interfaces.is_empty() {
        return None;
    }
    Some(DeviceInterfaceAdvertisementSpec {
        owner: mount.guest_root.clone(),
        interfaces,
    })
}

fn device_interface_class_advertisement_spec(
    entry: &str,
    default_device_path: &str,
    legacy_name: &str,
    bus_name: Option<&str>,
) -> Option<DeviceInterfaceClassAdvertisementSpec> {
    let entry = entry.trim();
    if entry.is_empty() {
        return None;
    }
    let (class, name) = if let Some((class, explicit_name)) = entry.split_once('=') {
        let class = class.trim();
        let explicit_name = explicit_name.trim();
        if class.is_empty() || explicit_name.is_empty() {
            return None;
        }
        let name = if explicit_name.eq_ignore_ascii_case("%d") {
            format!("$device\\{legacy_name}")
        } else if explicit_name.eq_ignore_ascii_case("%l") {
            legacy_name.to_owned()
        } else if explicit_name.eq_ignore_ascii_case("%b") {
            format!("$bus\\{}", bus_name?)
        } else {
            explicit_name.to_owned()
        };
        (class, name)
    } else {
        (entry, default_device_path.to_owned())
    };
    Some(DeviceInterfaceClassAdvertisementSpec {
        class: class.to_owned(),
        name,
    })
}

fn strip_host_prefix(host_path: &Path, host_root: &Path) -> Option<PathBuf> {
    if let Ok(relative) = host_path.strip_prefix(host_root) {
        return Some(relative.to_path_buf());
    }

    let canonical_path = fs::canonicalize(host_path).ok()?;
    let canonical_root = fs::canonicalize(host_root).ok()?;
    canonical_path
        .strip_prefix(canonical_root)
        .ok()
        .map(Path::to_path_buf)
}

fn default_mount_host_root(root: &Path, guest_root: &str) -> PathBuf {
    let mut path = root.to_path_buf();
    for part in guest_root.split('/').filter(|part| !part.is_empty()) {
        path.push(part);
    }
    path
}

fn normalize_guest_path(guest_path: &str) -> String {
    guest_path
        .trim()
        .trim_start_matches('\\')
        .trim_start_matches('/')
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_owned()
}

fn normalize_fsdmgr_mount_name(mount_name: &str) -> Result<String> {
    let mut mount_name = mount_name.trim();
    while mount_name.starts_with('\\') || mount_name.starts_with('/') {
        mount_name = &mount_name[1..];
    }
    mount_name = mount_name.trim_matches(['\\', '/']);
    if mount_name.is_empty() {
        return Err(Error::InvalidArgument("empty FSDMGR mount name".to_owned()));
    }
    let normalized = normalize_guest_path(mount_name);
    if normalized.is_empty() {
        return Err(Error::InvalidArgument("empty FSDMGR mount name".to_owned()));
    }
    Ok(normalized)
}

fn is_root_relative_path(guest_path: &str) -> bool {
    guest_path
        .trim_start()
        .chars()
        .next()
        .is_some_and(|ch| ch == '\\' || ch == '/')
}

fn mount_remainder<'a>(normalized: &'a str, mount_root: &str) -> Option<&'a str> {
    if normalized.eq_ignore_ascii_case(mount_root) {
        return Some("");
    }
    if normalized.len() <= mount_root.len() {
        return None;
    }
    let Some(prefix) = normalized.get(..mount_root.len()) else {
        return None;
    };
    if prefix.eq_ignore_ascii_case(mount_root) {
        normalized
            .get(mount_root.len()..)
            .and_then(|rest| rest.strip_prefix('/'))
    } else {
        None
    }
}

fn join_normalized_host_path(root: &Path, normalized: &str, guest_path: &str) -> Result<PathBuf> {
    if normalized.is_empty() {
        return Ok(root.to_path_buf());
    }

    let mut relative = PathBuf::new();
    for component in Path::new(normalized).components() {
        match component {
            Component::Normal(part) => relative.push(part),
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                return Err(Error::InvalidArgument(format!(
                    "guest path escapes file root: {guest_path}"
                )));
            }
        }
    }

    Ok(root.join(relative))
}

fn find_data_from_path(path: &Path, file_name: String) -> Result<FindData> {
    let metadata = fs::metadata(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut attributes = if metadata.is_dir() {
        FILE_ATTRIBUTE_DIRECTORY
    } else {
        FILE_ATTRIBUTE_ARCHIVE
    };
    if metadata.permissions().readonly() {
        attributes |= FILE_ATTRIBUTE_READONLY;
    }

    Ok(FindData {
        attributes,
        file_size: if metadata.is_file() {
            metadata.len()
        } else {
            0
        },
        file_name,
    })
}

fn has_wildcards(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?')
}

fn wildcard_match(pattern: &str, name: &str) -> bool {
    if pattern == "*" || pattern == "*.*" {
        return true;
    }

    let pattern = pattern.to_ascii_lowercase();
    let name = name.to_ascii_lowercase();
    let pattern = pattern.as_bytes();
    let name = name.as_bytes();
    let mut matches = vec![vec![false; name.len() + 1]; pattern.len() + 1];
    matches[0][0] = true;

    for pattern_index in 1..=pattern.len() {
        if pattern[pattern_index - 1] == b'*' {
            matches[pattern_index][0] = matches[pattern_index - 1][0];
        }
    }

    for pattern_index in 1..=pattern.len() {
        for name_index in 1..=name.len() {
            matches[pattern_index][name_index] = match pattern[pattern_index - 1] {
                b'*' => {
                    matches[pattern_index - 1][name_index] || matches[pattern_index][name_index - 1]
                }
                b'?' => matches[pattern_index - 1][name_index - 1],
                byte => byte == name[name_index - 1] && matches[pattern_index - 1][name_index - 1],
            };
        }
    }

    matches[pattern.len()][name.len()]
}

fn open_existing_host_file(host_path: &Path, requested_writable: bool) -> Result<(fs::File, bool)> {
    if requested_writable {
        match fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(host_path)
        {
            Ok(file) => return Ok((file, true)),
            Err(source) if source.kind() == std::io::ErrorKind::PermissionDenied => {}
            Err(source) => {
                return Err(Error::Io {
                    path: host_path.to_path_buf(),
                    source,
                });
            }
        }
    }

    fs::File::open(host_path)
        .map(|file| (file, false))
        .map_err(|source| Error::Io {
            path: host_path.to_path_buf(),
            source,
        })
}

fn file_lock_finish(start: u64, length: u64) -> Option<u64> {
    if length == 0 {
        return None;
    }
    start.checked_add(length - 1)
}

fn file_lock_ranges_overlap(
    first_start: u64,
    first_finish: u64,
    second_start: u64,
    second_finish: u64,
) -> bool {
    first_finish >= second_start && first_start <= second_finish
}

fn create_host_file(host_path: &Path, create_new: bool, truncate: bool) -> Result<fs::File> {
    if let Some(parent) = host_path.parent() {
        fs::create_dir_all(parent).map_err(|source| Error::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let mut options = fs::OpenOptions::new();
    options.read(true).write(true);
    if create_new {
        options.create_new(true);
    } else {
        options.create(true);
    }
    if truncate {
        options.truncate(true);
    }
    options.open(host_path).map_err(|source| Error::Io {
        path: host_path.to_path_buf(),
        source,
    })
}

const HOST_READ_CACHE_CHUNK: usize = 64 * 1024;

fn read_cached_host_file_into<F>(
    path: &Path,
    file: &mut fs::File,
    cache: &mut Vec<u8>,
    cache_start: &mut usize,
    offset: usize,
    len: usize,
    mut write: F,
) -> Result<u32>
where
    F: FnMut(&[u8]) -> Result<()>,
{
    if len > HOST_READ_CACHE_CHUNK {
        cache.clear();
        *cache_start = 0;
        return read_open_host_file_into(path, file, offset, len, write);
    }

    let mut position = offset;
    let mut remaining = len;
    let mut transferred = 0usize;
    while remaining != 0 {
        let cache_end = cache_start.saturating_add(cache.len());
        if position < *cache_start || position >= cache_end {
            cache.clear();
            *cache_start = position;
            file.seek(SeekFrom::Start(position as u64))
                .map_err(|source| Error::Io {
                    path: path.to_path_buf(),
                    source,
                })?;
            cache.resize(HOST_READ_CACHE_CHUNK, 0);
            let read = file.read(cache).map_err(|source| Error::Io {
                path: path.to_path_buf(),
                source,
            })?;
            cache.truncate(read);
            if read == 0 {
                break;
            }
        }

        let cache_offset = position.saturating_sub(*cache_start);
        let available = cache.len().saturating_sub(cache_offset);
        if available == 0 {
            break;
        }
        let chunk_len = remaining.min(available);
        write(&cache[cache_offset..cache_offset + chunk_len])?;
        position = position.saturating_add(chunk_len);
        remaining -= chunk_len;
        transferred += chunk_len;
    }

    Ok(transferred as u32)
}

fn read_open_host_file_into<F>(
    path: &Path,
    file: &mut fs::File,
    offset: usize,
    len: usize,
    mut write: F,
) -> Result<u32>
where
    F: FnMut(&[u8]) -> Result<()>,
{
    file.seek(SeekFrom::Start(offset as u64))
        .map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;

    let mut remaining = len;
    let mut transferred = 0usize;
    let mut buffer = vec![0u8; HOST_READ_CACHE_CHUNK.min(len.max(1))];
    while remaining != 0 {
        let chunk_len = remaining.min(buffer.len());
        let read = file
            .read(&mut buffer[..chunk_len])
            .map_err(|source| Error::Io {
                path: path.to_path_buf(),
                source,
            })?;
        if read == 0 {
            break;
        }
        write(&buffer[..read])?;
        transferred += read;
        remaining -= read;
    }
    Ok(transferred as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_parent_dir_escape() {
        let mut fs = HostFileSystem::new(".");
        assert!(
            fs.create_file_w("../outside.txt", GENERIC_READ, OPEN_ALWAYS)
                .is_err()
        );
    }

    #[test]
    fn root_find_is_empty_without_configured_mounts() {
        let root =
            std::env::temp_dir().join(format!("wince_file_empty_root_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let mut fs = HostFileSystem::new(&root);
        assert!(fs.find_first_file_w("\\").is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn root_find_lists_configured_ce_mount_points_and_object_store_entries() {
        let root =
            std::env::temp_dir().join(format!("wince_file_root_find_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("Documents")).unwrap();
        fs::create_dir_all(root.join("SDMMC Disk")).unwrap();
        fs::write(root.join("object.txt"), b"object").unwrap();
        let mut fs = HostFileSystem::new(&root);
        fs.mount(MountConfig {
            name: Some("sdmmc".to_owned()),
            device_name: None,
            bus_name: None,
            guest_root: "\\SDMMC Disk".to_owned(),
            host_root: None,
            total_mbytes: 8192,
            free_mbytes: 4096,
            writable: false,
            removable: true,
            system: false,
            hidden: false,
            interface_classes: Vec::new(),
            registry_roots: Vec::new(),
            registry_subkey: None,
        });
        fs.mount(MountConfig {
            name: Some("resident_flash".to_owned()),
            device_name: None,
            bus_name: None,
            guest_root: "\\ResidentFlash".to_owned(),
            host_root: None,
            total_mbytes: 2048,
            free_mbytes: 1024,
            writable: false,
            removable: false,
            system: false,
            hidden: false,
            interface_classes: Vec::new(),
            registry_roots: Vec::new(),
            registry_subkey: None,
        });
        let (_id, data) = fs.find_first_file_w("\\").unwrap();
        assert_eq!(
            data.attributes,
            FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_TEMPORARY
        );
        assert_eq!(data.file_name, "SDMMC Disk");
        let next = fs.find_next_file_w(_id).unwrap().unwrap();
        assert_eq!(next.file_name, "ResidentFlash");
        assert_eq!(next.attributes, FILE_ATTRIBUTE_DIRECTORY);
        let next = fs.find_next_file_w(_id).unwrap().unwrap();
        assert_eq!(next.file_name, "Documents");
        assert_eq!(next.attributes, FILE_ATTRIBUTE_DIRECTORY);
        let next = fs.find_next_file_w(_id).unwrap().unwrap();
        assert_eq!(next.file_name, "object.txt");
        assert_eq!(next.attributes, FILE_ATTRIBUTE_ARCHIVE);
        assert!(fs.find_next_file_w(_id).unwrap().is_none());
        let (_id, data) = fs.find_first_file_w("\\S*").unwrap();
        assert_eq!(data.file_name, "SDMMC Disk");
        assert!(fs.find_next_file_w(_id).unwrap().is_none());
        let (_id, data) = fs.find_first_file_w("\\D*").unwrap();
        assert_eq!(data.file_name, "Documents");
        let (_id, data) = fs.find_first_file_w("\\SDMMC Disk").unwrap();
        assert_eq!(data.file_name, "SDMMC Disk");
        assert_eq!(
            data.attributes,
            FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_TEMPORARY
        );
        assert!(fs.find_first_file_w("\\SDMMC Disk\\*").is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn root_directory_can_be_opened_as_an_existing_readonly_handle() {
        let root =
            std::env::temp_dir().join(format!("wince_file_root_open_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        let mut fs = HostFileSystem::new(&root);
        let id = fs
            .create_file_w("\\", GENERIC_READ | GENERIC_WRITE, OPEN_EXISTING)
            .unwrap();
        assert_eq!(fs.file_size(id).unwrap(), 0);
        assert!(fs.read_file(id, 16).unwrap().is_empty());
        let write = fs.write_file(id, b"ignored").unwrap();
        assert!(!write.success);
        assert_eq!(write.bytes_transferred, 0);
        assert!(fs.close(id).is_ok());
        assert!(
            fs.create_file_w("\\", GENERIC_WRITE, CREATE_ALWAYS)
                .is_err()
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn open_always_new_writable_file_creates_host_backing_immediately() {
        let root =
            std::env::temp_dir().join(format!("wince_file_open_always_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        let mut fs = HostFileSystem::new(&root);
        let id = fs
            .create_file_w("\\iNaviData\\SDLock.dat", GENERIC_WRITE, OPEN_ALWAYS)
            .unwrap();

        assert!(fs.open_file(id).unwrap().is_host_file_backed());
        assert!(root.join("iNaviData").join("SDLock.dat").is_file());
        assert!(fs.close(id).is_ok());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn readonly_host_files_stream_without_preloading_contents() {
        let root = std::env::temp_dir().join(format!("wince_file_stream_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = root.join("large.bin");
        let bytes: Vec<u8> = (0..=255).cycle().take(256 * 1024).collect();
        fs::write(&path, &bytes).unwrap();

        let mut fs = HostFileSystem::new(&root);
        let id = fs
            .create_file_w("\\large.bin", GENERIC_READ, OPEN_EXISTING)
            .unwrap();
        let open = fs.open_file(id).unwrap();
        assert!(open.is_host_file_backed());
        assert_eq!(open.memory_len(), 0);
        assert_eq!(open.file_len(), bytes.len());

        let first = fs.read_file(id, 17).unwrap();
        assert_eq!(first, bytes[..17]);
        let mut streamed = Vec::new();
        let copied = fs
            .read_file_into(id, 70 * 1024, |chunk| {
                streamed.extend_from_slice(chunk);
                Ok(())
            })
            .unwrap();
        assert_eq!(copied as usize, 70 * 1024);
        assert_eq!(streamed, bytes[17..17 + 70 * 1024]);
        assert_eq!(fs.open_file(id).unwrap().cursor(), 17 + 70 * 1024);

        fs.close(id).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn readwrite_existing_host_files_do_not_preload_contents() {
        let root = std::env::temp_dir().join(format!(
            "wince_file_readwrite_stream_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = root.join("large-rw.bin");
        let mut file = fs::File::create(&path).unwrap();
        file.set_len(2 * 1024 * 1024).unwrap();
        file.seek(SeekFrom::Start(1024 * 1024)).unwrap();
        file.write_all(b"rw-window").unwrap();
        drop(file);

        let mut fs = HostFileSystem::new(&root);
        let id = fs
            .create_file_w(
                "\\large-rw.bin",
                GENERIC_READ | GENERIC_WRITE,
                OPEN_EXISTING,
            )
            .unwrap();
        let open = fs.open_file(id).unwrap();
        assert!(open.is_host_file_backed());
        assert_eq!(open.memory_len(), 0);
        assert_eq!(open.file_len(), 2 * 1024 * 1024);

        fs.set_file_pointer(id, 1024 * 1024).unwrap();
        assert_eq!(fs.read_file(id, 9).unwrap(), b"rw-window");
        fs.write_file(id, b"-written").unwrap();
        fs.close(id).unwrap();
        let mut verify = fs::File::open(&path).unwrap();
        verify
            .seek(SeekFrom::Start(1024 * 1024 + b"rw-window".len() as u64))
            .unwrap();
        let mut written = [0u8; 8];
        verify.read_exact(&mut written).unwrap();
        assert_eq!(&written, b"-written");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn readwrite_existing_readonly_host_files_fall_back_to_read_handle() {
        let root = std::env::temp_dir().join(format!(
            "wince_file_readwrite_readonly_stream_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = root.join("readonly-rw.bin");
        fs::write(&path, b"readonly-window").unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&path, permissions).unwrap();

        let mut fs = HostFileSystem::new(&root);
        let id = fs
            .create_file_w(
                "\\readonly-rw.bin",
                GENERIC_READ | GENERIC_WRITE,
                OPEN_EXISTING,
            )
            .unwrap();
        let open = fs.open_file(id).unwrap();
        assert!(open.is_host_file_backed());
        assert!(!open.writable);
        assert_eq!(open.memory_len(), 0);
        assert_eq!(open.file_len(), b"readonly-window".len());
        assert_eq!(fs.read_file(id, 15).unwrap(), b"readonly-window");
        let write = fs.write_file(id, b"ignored").unwrap();
        assert!(!write.success);
        assert_eq!(write.bytes_transferred, 0);

        fs.close(id).unwrap();
        let mut permissions = std::fs::metadata(&path).unwrap().permissions();
        permissions.set_readonly(false);
        std::fs::set_permissions(&path, permissions).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn separate_host_file_handles_have_independent_cursors() {
        let root = std::env::temp_dir().join(format!(
            "wince_file_independent_cursors_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("cursor.bin"), b"abcdef").unwrap();

        let mut fs = HostFileSystem::new(&root);
        let first = fs
            .create_file_w("\\cursor.bin", GENERIC_READ, OPEN_EXISTING)
            .unwrap();
        let second = fs
            .create_file_w("\\cursor.bin", GENERIC_READ, OPEN_EXISTING)
            .unwrap();

        assert_eq!(fs.read_file(first, 2).unwrap(), b"ab");
        assert_eq!(fs.read_file(second, 3).unwrap(), b"abc");
        assert_eq!(fs.read_file(first, 2).unwrap(), b"cd");
        assert_eq!(fs.open_file(first).unwrap().cursor(), 4);
        assert_eq!(fs.open_file(second).unwrap().cursor(), 3);

        fs.close(first).unwrap();
        fs.close(second).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn host_file_read_file_into_chunks_large_requests() {
        let root = std::env::temp_dir().join(format!("wince_file_chunked_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let bytes: Vec<u8> = (0..=255).cycle().take(150 * 1024).collect();
        fs::write(root.join("chunked.bin"), &bytes).unwrap();

        let mut fs = HostFileSystem::new(&root);
        let id = fs
            .create_file_w("\\chunked.bin", GENERIC_READ, OPEN_EXISTING)
            .unwrap();
        let mut chunks = Vec::new();
        let mut streamed = Vec::new();
        let transferred = fs
            .read_file_into(id, bytes.len() as u32, |chunk| {
                chunks.push(chunk.len());
                streamed.extend_from_slice(chunk);
                Ok(())
            })
            .unwrap();

        assert_eq!(transferred as usize, bytes.len());
        assert_eq!(streamed, bytes);
        assert_eq!(chunks, vec![64 * 1024, 64 * 1024, 22 * 1024]);
        let stats = fs.io_stats();
        assert_eq!(stats.host_file_read_count, 1);
        assert_eq!(stats.host_file_read_bytes, bytes.len() as u64);
        assert_eq!(stats.max_read_request, bytes.len() as u32);

        fs.close(id).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn read_only_medium_host_files_use_shared_memory_backing() {
        let root =
            std::env::temp_dir().join(format!("wince_file_read_cache_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let mut bytes: Vec<u8> = (0..=255)
            .cycle()
            .take(READ_ONLY_MEMORY_BACKING_MIN_BYTES + 4096)
            .collect();
        fs::write(root.join("pack.bin"), &bytes).unwrap();

        let mut fs = HostFileSystem::new(&root);
        let first = fs
            .create_file_w("\\pack.bin", GENERIC_READ, OPEN_EXISTING)
            .unwrap();
        let second = fs
            .create_file_w("\\pack.bin", GENERIC_READ, OPEN_EXISTING)
            .unwrap();

        assert!(fs.open_file(first).unwrap().is_memory_backed());
        assert!(fs.open_file(second).unwrap().is_memory_backed());
        assert_eq!(fs.open_file(first).unwrap().memory_len(), bytes.len());
        assert_eq!(fs.read_file(first, 16).unwrap(), &bytes[..16]);
        assert_eq!(
            fs.read_at(second, bytes.len() - 16, 16).unwrap(),
            bytes[bytes.len() - 16..]
        );
        let stats = fs.io_stats();
        assert_eq!(stats.memory_backed_open_count, 2);
        assert_eq!(stats.host_file_open_count, 0);
        assert_eq!(stats.host_file_read_count, 0);

        fs.close(first).unwrap();
        fs.close(second).unwrap();
        let writer = fs
            .create_file_w("\\pack.bin", GENERIC_READ | GENERIC_WRITE, OPEN_EXISTING)
            .unwrap();
        fs.write_at(writer, 0, b"Z").unwrap();
        fs.close(writer).unwrap();
        bytes[0] = b'Z';

        let reread = fs
            .create_file_w("\\pack.bin", GENERIC_READ, OPEN_EXISTING)
            .unwrap();
        assert!(fs.open_file(reread).unwrap().is_memory_backed());
        assert_eq!(fs.read_file(reread, 1).unwrap(), b"Z");

        fs.close(reread).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn set_file_pointer_then_read_file_works_on_host_file_backing() {
        let root = std::env::temp_dir().join(format!("wince_file_seek_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("seek.bin"), b"0123456789").unwrap();

        let mut fs = HostFileSystem::new(&root);
        let id = fs
            .create_file_w("\\seek.bin", GENERIC_READ, OPEN_EXISTING)
            .unwrap();
        fs.set_file_pointer(id, 4).unwrap();
        assert_eq!(fs.read_file(id, 3).unwrap(), b"456");
        assert_eq!(fs.open_file(id).unwrap().cursor(), 7);

        fs.close(id).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn mount_without_host_root_inherits_default_root_backing() {
        let root =
            std::env::temp_dir().join(format!("wince_file_inherited_mount_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("Windows")).unwrap();
        fs::write(root.join("Windows").join("shell.txt"), b"shell").unwrap();

        let mut fs = HostFileSystem::new(&root);
        fs.mount(MountConfig {
            name: Some("windows".to_owned()),
            device_name: None,
            bus_name: None,
            guest_root: "\\Windows".to_owned(),
            host_root: None,
            total_mbytes: 2048,
            free_mbytes: 1024,
            writable: false,
            removable: false,
            system: true,
            hidden: false,
            interface_classes: Vec::new(),
            registry_roots: Vec::new(),
            registry_subkey: None,
        });

        let (_id, data) = fs.find_first_file_w("\\Windows").unwrap();
        assert_eq!(data.file_name, "Windows");
        assert_eq!(
            data.attributes,
            FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_SYSTEM
        );
        let (_id, data) = fs.find_first_file_w("\\Windows\\*").unwrap();
        assert_eq!(data.file_name, "shell.txt");
        assert_eq!(
            data.attributes,
            FILE_ATTRIBUTE_ARCHIVE | FILE_ATTRIBUTE_SYSTEM
        );
        assert!(
            fs.create_file_w("\\Windows\\x.txt", GENERIC_WRITE, CREATE_ALWAYS)
                .is_err()
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn explicit_mount_host_root_overrides_default_root_backing() {
        let root = std::env::temp_dir().join(format!(
            "wince_file_mount_override_root_{}",
            std::process::id()
        ));
        let override_root = std::env::temp_dir().join(format!(
            "wince_file_mount_override_sd_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&override_root);
        fs::create_dir_all(root.join("SDMMC Disk")).unwrap();
        fs::create_dir_all(&override_root).unwrap();
        fs::write(root.join("SDMMC Disk").join("which.txt"), b"default").unwrap();
        fs::write(override_root.join("which.txt"), b"override").unwrap();

        let mut fs = HostFileSystem::new(&root);
        fs.mount(MountConfig {
            name: Some("sdmmc".to_owned()),
            device_name: None,
            bus_name: None,
            guest_root: "\\SDMMC Disk".to_owned(),
            host_root: Some(override_root.clone()),
            total_mbytes: 8192,
            free_mbytes: 4096,
            writable: true,
            removable: true,
            system: false,
            hidden: false,
            interface_classes: Vec::new(),
            registry_roots: Vec::new(),
            registry_subkey: None,
        });
        let id = fs
            .create_file_w("\\SDMMC Disk\\which.txt", GENERIC_READ, OPEN_EXISTING)
            .unwrap();
        assert_eq!(fs.read_file(id, 16).unwrap(), b"override");

        fs.close(id).unwrap();
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(override_root).unwrap();
    }

    #[test]
    fn mount_matching_tolerates_non_ascii_nonmatching_paths() {
        assert_eq!(mount_remainder("翽䨼y젌〆\u{17}", "Windows"), None);
    }

    #[test]
    fn root_relative_paths_probe_process_mount_backing() {
        let root = std::env::temp_dir().join(format!("wince_file_mount_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("iNaviData")).unwrap();

        let mut fs = HostFileSystem::new(".");
        fs.mount_guest_root("\\ResidentFlash", &root);
        assert!(fs.find_first_file_w("\\iNaviData").is_err());
        fs.set_root_relative_guest_path("\\ResidentFlash\\INavi\\INavi.exe");
        let (_id, data) = fs.find_first_file_w("\\iNaviData").unwrap();
        assert_eq!(data.attributes, FILE_ATTRIBUTE_DIRECTORY);
        assert_eq!(data.file_name, "iNaviData");

        fs::remove_dir_all(root).unwrap();
    }
}
