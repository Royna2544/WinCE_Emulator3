use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Seek, SeekFrom},
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

#[derive(Debug, Clone)]
pub struct OpenFile {
    pub id: u32,
    pub guest_path: String,
    pub host_path: PathBuf,
    cursor: usize,
    data: Vec<u8>,
    streamed_readonly: bool,
    file_len: usize,
    writable: bool,
    dirty: bool,
}

impl OpenFile {
    pub fn cursor(&self) -> usize {
        self.cursor
    }
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
        }
    }

    pub fn from_storage(root: impl Into<PathBuf>, storage: StorageConfig) -> Self {
        let mut fs = Self::new(root);
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

    pub fn mount(&mut self, mount: MountConfig) {
        let guest_root = normalize_guest_path(&mount.guest_root);
        if guest_root.is_empty() {
            return;
        }
        let total_bytes = mount.total_bytes();
        let free_bytes = mount.free_bytes();
        let host_root = mount.host_root;
        let writable = mount.writable && host_root.is_some();
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
        let writable = desired_access & GENERIC_WRITE != 0 && !is_directory;

        let (data, streamed_readonly, file_len) = if is_directory {
            match creation_disposition {
                OPEN_EXISTING | OPEN_ALWAYS => (Vec::new(), false, 0),
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
                CREATE_NEW | CREATE_ALWAYS => (Vec::new(), false, 0),
                OPEN_EXISTING if !exists => {
                    return Err(Error::InvalidArgument(format!(
                        "file does not exist: {guest_path}"
                    )));
                }
                OPEN_EXISTING | OPEN_ALWAYS if writable => {
                    let data = fs::read(&host_path).unwrap_or_default();
                    let file_len = data.len();
                    (data, false, file_len)
                }
                OPEN_EXISTING | OPEN_ALWAYS => {
                    let file_len = fs::metadata(&host_path)
                        .map(|metadata| metadata.len())
                        .unwrap_or_default()
                        .try_into()
                        .unwrap_or(usize::MAX);
                    (Vec::new(), true, file_len)
                }
                TRUNCATE_EXISTING if exists && writable => (Vec::new(), false, 0),
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

        let id = self.next_id;
        self.next_id += 1;
        self.open_files.insert(
            id,
            OpenFile {
                id,
                guest_path: guest_path.to_owned(),
                host_path,
                cursor: 0,
                data,
                streamed_readonly,
                file_len,
                writable,
                dirty: matches!(
                    creation_disposition,
                    CREATE_NEW | CREATE_ALWAYS | TRUNCATE_EXISTING
                ),
            },
        );
        Ok(id)
    }

    pub fn read_file(&mut self, id: u32, requested: u32) -> Result<Vec<u8>> {
        let file = self.open_file_mut(id)?;
        let requested = requested as usize;
        if file.streamed_readonly {
            if file.cursor >= file.file_len {
                return Ok(Vec::new());
            }
            let start = file.cursor;
            let end = start.saturating_add(requested).min(file.file_len);
            let bytes = read_host_file_range(&file.host_path, start, end - start)?;
            file.cursor = start + bytes.len();
            return Ok(bytes);
        }
        if file.cursor >= file.file_len {
            return Ok(Vec::new());
        }
        let end = file.cursor.saturating_add(requested).min(file.file_len);
        let bytes = file.data[file.cursor..end].to_vec();
        file.cursor = end;
        Ok(bytes)
    }

    pub fn read_file_into<F>(&mut self, id: u32, requested: u32, mut write: F) -> Result<u32>
    where
        F: FnMut(&[u8]) -> Result<()>,
    {
        let file = self.open_file_mut(id)?;
        let requested = requested as usize;
        if file.streamed_readonly {
            if file.cursor >= file.file_len {
                write(&[])?;
                return Ok(0);
            }
            let start = file.cursor;
            let end = start.saturating_add(requested).min(file.file_len);
            let transferred =
                read_host_file_range_into(&file.host_path, start, end - start, &mut write)?;
            file.cursor = start + transferred as usize;
            return Ok(transferred);
        }
        if file.cursor >= file.file_len {
            write(&[])?;
            return Ok(0);
        }
        let start = file.cursor;
        let end = file.cursor.saturating_add(requested).min(file.file_len);
        write(&file.data[start..end])?;
        file.cursor = end;
        Ok((end - start) as u32)
    }

    pub fn read_at(&self, id: u32, offset: usize, requested: usize) -> Result<Vec<u8>> {
        let file = self.open_file(id)?;
        if file.streamed_readonly {
            if offset >= file.file_len {
                return Ok(Vec::new());
            }
            let end = offset.saturating_add(requested).min(file.file_len);
            return read_host_file_range(&file.host_path, offset, end - offset);
        }
        if offset >= file.file_len {
            return Ok(Vec::new());
        }
        let end = offset.saturating_add(requested).min(file.file_len);
        Ok(file.data[offset..end].to_vec())
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

        let end = file.cursor + bytes.len();
        if end > file.data.len() {
            file.data.resize(end, 0);
        }
        file.data[file.cursor..end].copy_from_slice(bytes);
        file.cursor = end;
        file.file_len = file.data.len();
        file.dirty = true;
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

        let end = offset.saturating_add(bytes.len());
        if end > file.data.len() {
            file.data.resize(end, 0);
        }
        file.data[offset..end].copy_from_slice(bytes);
        file.file_len = file.data.len();
        file.dirty = true;
        Ok(FileIoResult {
            success: true,
            bytes_transferred: bytes.len() as u32,
        })
    }

    pub fn set_file_pointer(&mut self, id: u32, position: usize) -> Result<usize> {
        let file = self.open_file_mut(id)?;
        file.cursor = position;
        Ok(file.cursor)
    }

    pub fn file_size(&self, id: u32) -> Result<usize> {
        Ok(self.open_file(id)?.file_len)
    }

    pub fn flush(&mut self, id: u32) -> Result<()> {
        let file = self.open_file_mut(id)?;
        if file.dirty {
            if let Some(parent) = file.host_path.parent() {
                fs::create_dir_all(parent).map_err(|source| Error::Io {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
            fs::write(&file.host_path, &file.data).map_err(|source| Error::Io {
                path: file.host_path.clone(),
                source,
            })?;
            file.dirty = false;
            file.file_len = file.data.len();
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
            entries.push(mount_root_find_data(mount, file_name.to_owned()));
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

fn read_host_file_range(path: &Path, offset: usize, len: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(len);
    read_host_file_range_into(path, offset, len, |chunk| {
        bytes.extend_from_slice(chunk);
        Ok(())
    })?;
    Ok(bytes)
}

fn read_host_file_range_into<F>(path: &Path, offset: usize, len: usize, mut write: F) -> Result<u32>
where
    F: FnMut(&[u8]) -> Result<()>,
{
    const STREAM_CHUNK: usize = 64 * 1024;

    let mut file = fs::File::open(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })?;
    file.seek(SeekFrom::Start(offset as u64))
        .map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;

    let mut remaining = len;
    let mut transferred = 0usize;
    let mut buffer = vec![0u8; STREAM_CHUNK.min(len.max(1))];
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
        let mut fs = HostFileSystem::new(".");
        assert!(fs.find_first_file_w("\\").is_err());
    }

    #[test]
    fn root_find_lists_configured_ce_mount_points() {
        let mut fs = HostFileSystem::new(".");
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
        let (_id, data) = fs.find_first_file_w("\\S*").unwrap();
        assert_eq!(data.file_name, "SDMMC Disk");
        let (_id, data) = fs.find_first_file_w("\\SDMMC Disk").unwrap();
        assert_eq!(data.file_name, "SDMMC Disk");
        assert_eq!(
            data.attributes,
            FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_TEMPORARY
        );
        assert!(fs.find_first_file_w("\\SDMMC Disk\\*").is_err());
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
        assert!(open.streamed_readonly);
        assert!(open.data.is_empty());
        assert_eq!(open.file_len, bytes.len());

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
    fn hostless_mount_is_empty_and_read_only() {
        let mut fs = HostFileSystem::new(".");
        fs.mount(MountConfig {
            name: Some("windows".to_owned()),
            guest_root: "\\Windows".to_owned(),
            host_root: None,
            total_mbytes: 2048,
            free_mbytes: 1024,
            writable: true,
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
        assert!(fs.find_first_file_w("\\Windows\\*").is_err());
        assert!(
            fs.create_file_w("\\Windows\\x.txt", GENERIC_WRITE, CREATE_ALWAYS)
                .is_err()
        );
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
