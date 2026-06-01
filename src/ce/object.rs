use std::collections::BTreeMap;

use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub enum KernelObject {
    Event(EventObject),
    Mutex(MutexObject),
    File(FileObject),
    Device(String),
    Window(u32),
    WaveOut(u32),
}

#[derive(Debug, Clone)]
pub struct EventObject {
    pub name: Option<String>,
    pub manual_reset: bool,
    pub signaled: bool,
}

#[derive(Debug, Clone)]
pub struct MutexObject {
    pub name: Option<String>,
    pub owner_thread: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct FileObject {
    pub guest_path: String,
}

#[derive(Debug, Clone)]
pub struct HandleTable {
    next: u32,
    objects: BTreeMap<u32, KernelObject>,
}

impl Default for HandleTable {
    fn default() -> Self {
        Self {
            next: 0x100,
            objects: BTreeMap::new(),
        }
    }
}

impl HandleTable {
    pub fn insert(&mut self, object: KernelObject) -> u32 {
        let handle = self.next;
        self.next += 4;
        self.objects.insert(handle, object);
        handle
    }

    pub fn get(&self, handle: u32) -> Result<&KernelObject> {
        self.objects
            .get(&handle)
            .ok_or(Error::InvalidHandle(handle))
    }

    pub fn get_mut(&mut self, handle: u32) -> Result<&mut KernelObject> {
        self.objects
            .get_mut(&handle)
            .ok_or(Error::InvalidHandle(handle))
    }

    pub fn close(&mut self, handle: u32) -> Result<KernelObject> {
        self.objects
            .remove(&handle)
            .ok_or(Error::InvalidHandle(handle))
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }
}
