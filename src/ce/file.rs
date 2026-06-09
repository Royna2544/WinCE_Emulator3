use std::{
    collections::BTreeMap,
    fmt, fs,
    io::{Read, Seek, SeekFrom, Write},
    path::{Component, Path, PathBuf},
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
    io_stats: FileIoStats,
}

#[derive(Debug, Clone)]
pub struct FileMount {
    pub name: Option<String>,
    pub guest_root: String,
    pub host_root: Option<PathBuf>,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub writable: bool,
    pub removable: bool,
    pub system: bool,
    pub hidden: bool,
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
        matches!(self.backing, FileBacking::Memory(_))
    }

    pub fn is_host_file_backed(&self) -> bool {
        matches!(self.backing, FileBacking::HostFile(_))
    }

    pub fn memory_len(&self) -> usize {
        match &self.backing {
            FileBacking::Memory(bytes) => bytes.len(),
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
    HostFile(fs::File),
}

impl Clone for FileBacking {
    fn clone(&self) -> Self {
        match self {
            Self::Memory(bytes) => Self::Memory(bytes.clone()),
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
            guest_root: guest_root.to_owned(),
            host_root: Some(host_root.into()),
            total_mbytes: 8192,
            free_mbytes: 4096,
            writable: true,
            removable: true,
            system: false,
            hidden: false,
        });
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
                guest_root,
                host_root,
                total_bytes,
                free_bytes,
                writable,
                removable: mount.removable,
                system: mount.system,
                hidden: mount.hidden,
            },
        );
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

    pub fn io_stats(&self) -> FileIoStats {
        self.io_stats
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
            return Err(Error::InvalidArgument(format!(
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
                    (FileBacking::Memory(Vec::new()), 0, requested_writable)
                }
                OPEN_EXISTING if !exists => {
                    return Err(Error::InvalidArgument(format!(
                        "file does not exist: {guest_path}"
                    )));
                }
                OPEN_EXISTING | OPEN_ALWAYS if exists => {
                    let file_len = fs::metadata(&host_path)
                        .map(|metadata| metadata.len())
                        .unwrap_or_default()
                        .try_into()
                        .unwrap_or(usize::MAX);
                    let (file, writable) = open_existing_host_file(&host_path, requested_writable)?;
                    (FileBacking::HostFile(file), file_len, writable)
                }
                OPEN_ALWAYS => (FileBacking::Memory(Vec::new()), 0, requested_writable),
                TRUNCATE_EXISTING if exists && requested_writable => {
                    (FileBacking::Memory(Vec::new()), 0, true)
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
        let is_memory_backed = matches!(backing, FileBacking::Memory(_));
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
        find_data_from_path(&host_path, file_name)
    }

    pub fn create_directory_w(&self, guest_path: &str) -> Result<()> {
        if self
            .mount_for_guest_path(guest_path)
            .is_some_and(|mount| !mount.writable)
        {
            return Err(Error::InvalidArgument(format!(
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
        if self
            .mount_for_guest_path(guest_path)
            .is_some_and(|mount| !mount.writable)
        {
            return Err(Error::InvalidArgument(format!(
                "guest mount is read-only: {guest_path}"
            )));
        }
        let host_path = self.translate_guest_path(guest_path)?;
        fs::remove_dir(&host_path).map_err(|source| Error::Io {
            path: host_path,
            source,
        })
    }

    pub fn delete_file_w(&self, guest_path: &str) -> Result<()> {
        if self
            .mount_for_guest_path(guest_path)
            .is_some_and(|mount| !mount.writable)
        {
            return Err(Error::InvalidArgument(format!(
                "guest mount is read-only: {guest_path}"
            )));
        }
        let host_path = self.translate_guest_path(guest_path)?;
        fs::remove_file(&host_path).map_err(|source| Error::Io {
            path: host_path,
            source,
        })
    }

    pub fn move_file_w(&self, existing_path: &str, new_path: &str) -> Result<()> {
        if self
            .mount_for_guest_path(existing_path)
            .is_some_and(|mount| !mount.writable)
            || self
                .mount_for_guest_path(new_path)
                .is_some_and(|mount| !mount.writable)
        {
            return Err(Error::InvalidArgument(format!(
                "guest mount is read-only: {existing_path} -> {new_path}"
            )));
        }
        let existing = self.translate_guest_path(existing_path)?;
        let new = self.translate_guest_path(new_path)?;
        if let Some(parent) = new.parent() {
            fs::create_dir_all(parent).map_err(|source| Error::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::rename(&existing, &new).map_err(|source| Error::Io {
            path: existing,
            source,
        })
    }

    pub fn copy_file_w(
        &self,
        existing_path: &str,
        new_path: &str,
        fail_if_exists: bool,
    ) -> Result<()> {
        if self
            .mount_for_guest_path(new_path)
            .is_some_and(|mount| !mount.writable)
        {
            return Err(Error::InvalidArgument(format!(
                "guest mount is read-only: {new_path}"
            )));
        }
        let existing = self.translate_guest_path(existing_path)?;
        let new = self.translate_guest_path(new_path)?;
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
        if self
            .mount_for_guest_path(guest_path)
            .is_some_and(|mount| !mount.writable)
        {
            return Err(Error::InvalidArgument(format!(
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
        let file = self.open_file_mut(id)?;
        if !file.writable {
            return Ok(FileIoResult {
                success: false,
                bytes_transferred: 0,
            });
        }

        file.write_current(bytes)?;
        Ok(FileIoResult {
            success: true,
            bytes_transferred: bytes.len() as u32,
        })
    }

    pub fn write_at(&mut self, id: u32, offset: usize, bytes: &[u8]) -> Result<FileIoResult> {
        let file = self.open_file_mut(id)?;
        if !file.writable {
            return Ok(FileIoResult {
                success: false,
                bytes_transferred: 0,
            });
        }

        file.write_at(offset, bytes)?;
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
        let file = self.open_file_mut(id)?;
        if !file.writable {
            return Ok(false);
        }
        let new_len = file.cursor;
        match &mut file.backing {
            FileBacking::Memory(data) => {
                data.resize(new_len, 0);
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
        let write_time = meta
            .modified()
            .map(system_time_to_filetime)
            .unwrap_or(0);
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
        self.open_files
            .remove(&id)
            .ok_or(Error::InvalidHandle(id))
            .map(|_| ())
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

    pub fn open_file(&self, id: u32) -> Result<&OpenFile> {
        self.open_files.get(&id).ok_or(Error::InvalidHandle(id))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn host_path_for_guest(&self, guest_path: &str) -> Result<PathBuf> {
        self.translate_guest_path(guest_path)
    }

    fn open_file_mut(&mut self, id: u32) -> Result<&mut OpenFile> {
        self.open_files.get_mut(&id).ok_or(Error::InvalidHandle(id))
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
                    entries.push(find_data_from_path(&entry.path(), file_name)?);
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
        Ok(vec![find_data_from_path(&host_pattern, file_name)?])
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
            guest_root: "\\SDMMC Disk".to_owned(),
            host_root: None,
            total_mbytes: 8192,
            free_mbytes: 4096,
            writable: false,
            removable: true,
            system: false,
            hidden: false,
        });
        fs.mount(MountConfig {
            name: Some("resident_flash".to_owned()),
            guest_root: "\\ResidentFlash".to_owned(),
            host_root: None,
            total_mbytes: 2048,
            free_mbytes: 1024,
            writable: false,
            removable: false,
            system: false,
            hidden: false,
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
            guest_root: "\\Windows".to_owned(),
            host_root: None,
            total_mbytes: 2048,
            free_mbytes: 1024,
            writable: false,
            removable: false,
            system: true,
            hidden: false,
        });

        let (_id, data) = fs.find_first_file_w("\\Windows").unwrap();
        assert_eq!(data.file_name, "Windows");
        assert_eq!(
            data.attributes,
            FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_SYSTEM
        );
        let (_id, data) = fs.find_first_file_w("\\Windows\\*").unwrap();
        assert_eq!(data.file_name, "shell.txt");
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
            guest_root: "\\SDMMC Disk".to_owned(),
            host_root: Some(override_root.clone()),
            total_mbytes: 8192,
            free_mbytes: 4096,
            writable: true,
            removable: true,
            system: false,
            hidden: false,
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
