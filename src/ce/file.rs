use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

use crate::error::{Error, Result};

pub const GENERIC_READ: u32 = 0x8000_0000;
pub const GENERIC_WRITE: u32 = 0x4000_0000;

pub const CREATE_NEW: u32 = 1;
pub const CREATE_ALWAYS: u32 = 2;
pub const OPEN_EXISTING: u32 = 3;
pub const OPEN_ALWAYS: u32 = 4;
pub const TRUNCATE_EXISTING: u32 = 5;

#[derive(Debug, Clone)]
pub struct HostFileSystem {
    root: PathBuf,
    next_id: u32,
    open_files: BTreeMap<u32, OpenFile>,
}

#[derive(Debug, Clone)]
pub struct OpenFile {
    pub id: u32,
    pub guest_path: String,
    pub host_path: PathBuf,
    cursor: usize,
    data: Vec<u8>,
    writable: bool,
    dirty: bool,
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
            next_id: 1,
            open_files: BTreeMap::new(),
        }
    }

    pub fn create_file_w(
        &mut self,
        guest_path: &str,
        desired_access: u32,
        creation_disposition: u32,
    ) -> Result<u32> {
        let host_path = self.translate_guest_path(guest_path)?;
        let exists = host_path.exists();
        let writable = desired_access & GENERIC_WRITE != 0;

        let data = match creation_disposition {
            CREATE_NEW if exists => {
                return Err(Error::InvalidArgument(format!(
                    "file already exists: {guest_path}"
                )));
            }
            CREATE_NEW | CREATE_ALWAYS => Vec::new(),
            OPEN_EXISTING if !exists => {
                return Err(Error::InvalidArgument(format!(
                    "file does not exist: {guest_path}"
                )));
            }
            OPEN_EXISTING | OPEN_ALWAYS => fs::read(&host_path).unwrap_or_default(),
            TRUNCATE_EXISTING if exists && writable => Vec::new(),
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
        let end = file.cursor.saturating_add(requested).min(file.data.len());
        let bytes = file.data[file.cursor..end].to_vec();
        file.cursor = end;
        Ok(bytes)
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
        file.dirty = true;
        Ok(FileIoResult {
            success: true,
            bytes_transferred: bytes.len() as u32,
        })
    }

    pub fn set_file_pointer(&mut self, id: u32, position: usize) -> Result<usize> {
        let file = self.open_file_mut(id)?;
        file.cursor = position.min(file.data.len());
        Ok(file.cursor)
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
        let normalized = guest_path
            .trim()
            .trim_start_matches('\\')
            .trim_start_matches('/')
            .replace('\\', "/");

        if normalized.is_empty() {
            return Err(Error::InvalidArgument("empty guest path".to_owned()));
        }

        let mut relative = PathBuf::new();
        for component in Path::new(&normalized).components() {
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

        Ok(self.root.join(relative))
    }
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
}
