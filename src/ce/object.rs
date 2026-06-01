use std::collections::BTreeMap;

use crate::{
    ce::devices::DeviceSession,
    error::{Error, Result},
};

#[derive(Debug, Clone)]
pub enum KernelObject {
    Event(EventObject),
    Mutex(MutexObject),
    File(FileObject),
    Device(DeviceSession),
    Window(u32),
    WaveOut(u32),
    CriticalSection(CriticalSectionObject),
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
    pub file_id: u32,
}

#[derive(Debug, Clone)]
pub struct CriticalSectionObject {
    pub guest_ptr: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitResult {
    Object0,
    Timeout,
    Failed,
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

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn create_event(
        &mut self,
        name: Option<String>,
        manual_reset: bool,
        initial_state: bool,
    ) -> u32 {
        self.insert(KernelObject::Event(EventObject {
            name,
            manual_reset,
            signaled: initial_state,
        }))
    }

    pub fn create_mutex(&mut self, name: Option<String>, initial_owner: Option<u32>) -> u32 {
        self.insert(KernelObject::Mutex(MutexObject {
            name,
            owner_thread: initial_owner,
        }))
    }

    pub fn set_event(&mut self, handle: u32) -> bool {
        let Ok(KernelObject::Event(event)) = self.get_mut(handle) else {
            return false;
        };
        event.signaled = true;
        true
    }

    pub fn reset_event(&mut self, handle: u32) -> bool {
        let Ok(KernelObject::Event(event)) = self.get_mut(handle) else {
            return false;
        };
        event.signaled = false;
        true
    }

    pub fn release_mutex(&mut self, handle: u32, thread_id: u32) -> bool {
        let Ok(KernelObject::Mutex(mutex)) = self.get_mut(handle) else {
            return false;
        };
        if mutex.owner_thread == Some(thread_id) || mutex.owner_thread.is_none() {
            mutex.owner_thread = None;
            true
        } else {
            false
        }
    }

    pub fn wait_for_single_object(
        &mut self,
        handle: u32,
        timeout_ms: u32,
        thread_id: u32,
    ) -> WaitResult {
        let Ok(object) = self.get_mut(handle) else {
            return WaitResult::Failed;
        };

        match object {
            KernelObject::Event(event) if event.signaled => {
                if !event.manual_reset {
                    event.signaled = false;
                }
                WaitResult::Object0
            }
            KernelObject::Mutex(mutex)
                if mutex.owner_thread.is_none() || mutex.owner_thread == Some(thread_id) =>
            {
                mutex.owner_thread = Some(thread_id);
                WaitResult::Object0
            }
            KernelObject::File(_)
            | KernelObject::Device(_)
            | KernelObject::Window(_)
            | KernelObject::WaveOut(_)
            | KernelObject::CriticalSection(_) => WaitResult::Object0,
            _ if timeout_ms == 0 => WaitResult::Timeout,
            _ => WaitResult::Timeout,
        }
    }
}
