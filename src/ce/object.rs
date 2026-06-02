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
    FindFile(FindFileObject),
    Device(DeviceSession),
    Window(u32),
    WaveOut(u32),
    FileMapping(FileMappingObject),
    CriticalSection(CriticalSectionObject),
    Thread(ThreadObject),
    Process(ProcessObject),
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
pub struct FindFileObject {
    pub guest_pattern: String,
    pub find_id: u32,
}

#[derive(Debug, Clone)]
pub struct FileMappingObject {
    pub name: Option<String>,
    pub size: u32,
    pub protect: u32,
    pub file_id: Option<u32>,
    pub data: Vec<u8>,
    pub view_base: Option<u32>,
    pub view_size: u32,
    pub view_offset: u32,
}

#[derive(Debug, Clone)]
pub struct CriticalSectionObject {
    pub guest_ptr: u32,
}

#[derive(Debug, Clone)]
pub struct ThreadObject {
    pub thread_id: u32,
    pub start_address: u32,
    pub parameter: u32,
    pub exit_code: u32,
    pub priority: i32,
    pub signaled: bool,
    pub suspend_count: u32,
}

#[derive(Debug, Clone)]
pub struct ProcessObject {
    pub process_id: u32,
    pub exit_code: u32,
    pub signaled: bool,
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
        if let Some(name) = name.as_deref() {
            if let Some((handle, _)) = self.objects.iter().find(|(_, object)| {
                matches!(object, KernelObject::Event(event) if event.name.as_deref() == Some(name))
            }) {
                return *handle;
            }
        }
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

    pub fn create_thread(
        &mut self,
        thread_id: u32,
        start_address: u32,
        parameter: u32,
        suspended: bool,
    ) -> u32 {
        self.insert(KernelObject::Thread(ThreadObject {
            thread_id,
            start_address,
            parameter,
            exit_code: 259,
            priority: 0,
            signaled: false,
            suspend_count: u32::from(suspended),
        }))
    }

    pub fn create_file_mapping(
        &mut self,
        name: Option<String>,
        size: u32,
        protect: u32,
        file_id: Option<u32>,
    ) -> u32 {
        if let Some(name) = name.as_deref() {
            if let Some((handle, _)) = self.objects.iter().find(|(_, object)| {
                matches!(
                    object,
                    KernelObject::FileMapping(mapping) if mapping.name.as_deref() == Some(name)
                )
            }) {
                return *handle;
            }
        }
        self.insert(KernelObject::FileMapping(FileMappingObject {
            name,
            size,
            protect,
            file_id,
            data: vec![0; size as usize],
            view_base: None,
            view_size: 0,
            view_offset: 0,
        }))
    }

    pub fn create_process(&mut self, process_id: u32) -> u32 {
        self.insert(KernelObject::Process(ProcessObject {
            process_id,
            exit_code: 259,
            signaled: false,
        }))
    }

    pub fn file_mapping(&self, handle: u32) -> Result<&FileMappingObject> {
        match self.get(handle)? {
            KernelObject::FileMapping(mapping) => Ok(mapping),
            _ => Err(Error::InvalidHandle(handle)),
        }
    }

    pub fn file_mapping_mut(&mut self, handle: u32) -> Result<&mut FileMappingObject> {
        match self.get_mut(handle)? {
            KernelObject::FileMapping(mapping) => Ok(mapping),
            _ => Err(Error::InvalidHandle(handle)),
        }
    }

    pub fn has_file_mapping_view(&self, base: u32) -> bool {
        self.objects.values().any(|object| {
            matches!(
                object,
                KernelObject::FileMapping(FileMappingObject {
                    view_base: Some(view_base),
                    ..
                }) if *view_base == base
            )
        })
    }

    pub fn file_mapping_by_view(&self, base: u32) -> Option<&FileMappingObject> {
        self.objects.values().find_map(|object| match object {
            KernelObject::FileMapping(mapping) if mapping.view_base == Some(base) => Some(mapping),
            _ => None,
        })
    }

    pub fn file_mapping_by_view_mut(&mut self, base: u32) -> Option<&mut FileMappingObject> {
        self.objects.values_mut().find_map(|object| match object {
            KernelObject::FileMapping(mapping) if mapping.view_base == Some(base) => Some(mapping),
            _ => None,
        })
    }

    pub fn file_mappings_mut(&mut self) -> impl Iterator<Item = &mut FileMappingObject> {
        self.objects.values_mut().filter_map(|object| match object {
            KernelObject::FileMapping(mapping) => Some(mapping),
            _ => None,
        })
    }

    pub fn file_mappings(&self) -> impl Iterator<Item = &FileMappingObject> {
        self.objects.values().filter_map(|object| match object {
            KernelObject::FileMapping(mapping) => Some(mapping),
            _ => None,
        })
    }

    pub fn mark_thread_exited(&mut self, handle: u32, exit_code: u32) -> bool {
        let Ok(KernelObject::Thread(thread)) = self.get_mut(handle) else {
            return false;
        };
        thread.exit_code = exit_code;
        thread.signaled = true;
        true
    }

    pub fn suspend_thread(&mut self, handle: u32) -> Option<u32> {
        let Ok(KernelObject::Thread(thread)) = self.get_mut(handle) else {
            return None;
        };
        let previous = thread.suspend_count;
        thread.suspend_count = thread.suspend_count.saturating_add(1);
        Some(previous)
    }

    pub fn resume_thread(&mut self, handle: u32) -> Option<u32> {
        let Ok(KernelObject::Thread(thread)) = self.get_mut(handle) else {
            return None;
        };
        let previous = thread.suspend_count;
        thread.suspend_count = thread.suspend_count.saturating_sub(1);
        Some(previous)
    }

    pub fn thread_priority(&self, handle: u32) -> Option<i32> {
        let Ok(KernelObject::Thread(thread)) = self.get(handle) else {
            return None;
        };
        Some(thread.priority)
    }

    pub fn thread_priority_by_id(&self, thread_id: u32) -> Option<i32> {
        self.objects.values().find_map(|object| match object {
            KernelObject::Thread(thread) if thread.thread_id == thread_id => Some(thread.priority),
            _ => None,
        })
    }

    pub fn set_thread_priority(&mut self, handle: u32, priority: i32) -> bool {
        let Ok(KernelObject::Thread(thread)) = self.get_mut(handle) else {
            return false;
        };
        thread.priority = priority;
        true
    }

    pub fn thread_start(&self, handle: u32) -> Option<(u32, u32, u32)> {
        let Ok(KernelObject::Thread(thread)) = self.get(handle) else {
            return None;
        };
        (!thread.signaled && thread.suspend_count == 0).then_some((
            thread.thread_id,
            thread.start_address,
            thread.parameter,
        ))
    }

    pub fn mark_process_exited(&mut self, handle: u32, exit_code: u32) -> bool {
        let Ok(KernelObject::Process(process)) = self.get_mut(handle) else {
            return false;
        };
        process.exit_code = exit_code;
        process.signaled = true;
        true
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
            | KernelObject::FindFile(_)
            | KernelObject::Device(_)
            | KernelObject::Window(_)
            | KernelObject::WaveOut(_)
            | KernelObject::FileMapping(_)
            | KernelObject::CriticalSection(_) => WaitResult::Object0,
            KernelObject::Thread(thread) if thread.signaled => WaitResult::Object0,
            KernelObject::Process(process) if process.signaled => WaitResult::Object0,
            _ if timeout_ms == 0 => WaitResult::Timeout,
            _ => WaitResult::Timeout,
        }
    }

    pub fn is_wait_ready(&self, handle: u32, thread_id: u32) -> Option<bool> {
        let object = self.get(handle).ok()?;
        Some(match object {
            KernelObject::Event(event) => event.signaled,
            KernelObject::Mutex(mutex) => {
                mutex.owner_thread.is_none() || mutex.owner_thread == Some(thread_id)
            }
            KernelObject::Thread(thread) => thread.signaled,
            KernelObject::Process(process) => process.signaled,
            KernelObject::File(_)
            | KernelObject::FindFile(_)
            | KernelObject::Device(_)
            | KernelObject::Window(_)
            | KernelObject::WaveOut(_)
            | KernelObject::FileMapping(_)
            | KernelObject::CriticalSection(_) => true,
        })
    }
}
