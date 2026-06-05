use std::collections::BTreeMap;

use crate::{
    ce::devices::DeviceSession,
    error::{Error, Result},
};

pub const MAX_CE_PRIORITY_LEVELS: i32 = 256;
pub const MAX_WIN32_PRIORITY_LEVELS: u32 = 8;
pub const THREAD_PRIORITY_TIME_CRITICAL: u32 = 0;
pub const THREAD_PRIORITY_NORMAL: u32 = 3;
pub const CE_THREAD_PRIORITY_NORMAL: i32 =
    MAX_CE_PRIORITY_LEVELS - MAX_WIN32_PRIORITY_LEVELS as i32 + THREAD_PRIORITY_NORMAL as i32;
pub const MAX_SUSPEND_COUNT: u32 = 127;
pub const MUTEX_MAX_LOCK_COUNT: u32 = 0x7fff;

#[derive(Debug, Clone)]
pub enum KernelObject {
    Event(EventObject),
    Mutex(MutexObject),
    Semaphore(SemaphoreObject),
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
    pub lock_count: u32,
}

#[derive(Debug, Clone)]
pub struct SemaphoreObject {
    pub name: Option<String>,
    pub count: i32,
    pub maximum: i32,
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
    pub views: Vec<FileMappingView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileMappingView {
    pub base: u32,
    pub size: u32,
    pub offset: u32,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitMultipleResult {
    Object(u32),
    Timeout,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadSuspendResult {
    Previous(u32),
    InvalidHandle,
    SignalRefused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadResumeResult {
    Previous(u32),
    InvalidHandle,
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

    pub fn describe_handle(&self, handle: u32) -> String {
        match self.get(handle) {
            Ok(KernelObject::Event(event)) => format!(
                "event(name={},manual={},signaled={})",
                event.name.as_deref().unwrap_or("<unnamed>"),
                event.manual_reset,
                event.signaled
            ),
            Ok(KernelObject::Mutex(mutex)) => format!(
                "mutex(name={},owner={},locks={})",
                mutex.name.as_deref().unwrap_or("<unnamed>"),
                mutex
                    .owner_thread
                    .map(|thread| thread.to_string())
                    .unwrap_or_else(|| "none".to_owned()),
                mutex.lock_count
            ),
            Ok(KernelObject::Semaphore(semaphore)) => format!(
                "semaphore(name={},count={},max={})",
                semaphore.name.as_deref().unwrap_or("<unnamed>"),
                semaphore.count,
                semaphore.maximum
            ),
            Ok(KernelObject::File(file)) => {
                format!("file(id={},path={})", file.file_id, file.guest_path)
            }
            Ok(KernelObject::FindFile(find)) => {
                format!("find(id={},pattern={})", find.find_id, find.guest_pattern)
            }
            Ok(KernelObject::Device(_)) => "device".to_owned(),
            Ok(KernelObject::Window(hwnd)) => format!("window(hwnd=0x{hwnd:08x})"),
            Ok(KernelObject::WaveOut(id)) => format!("waveout(id={id})"),
            Ok(KernelObject::FileMapping(mapping)) => format!(
                "mapping(name={},size={},views={})",
                mapping.name.as_deref().unwrap_or("<unnamed>"),
                mapping.size,
                mapping.views.len()
            ),
            Ok(KernelObject::CriticalSection(cs)) => {
                format!("critical_section(ptr=0x{:08x})", cs.guest_ptr)
            }
            Ok(KernelObject::Thread(thread)) => format!(
                "thread(id={},signaled={},suspend={},exit=0x{:08x})",
                thread.thread_id, thread.signaled, thread.suspend_count, thread.exit_code
            ),
            Ok(KernelObject::Process(process)) => format!(
                "process(id={},signaled={},exit=0x{:08x})",
                process.process_id, process.signaled, process.exit_code
            ),
            Err(_) => "invalid".to_owned(),
        }
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

    pub fn open_event(&self, name: &str) -> Option<u32> {
        self.objects
            .iter()
            .find_map(|(handle, object)| match object {
                KernelObject::Event(event) if event.name.as_deref() == Some(name) => Some(*handle),
                _ => None,
            })
    }

    pub fn create_mutex(&mut self, name: Option<String>, initial_owner: Option<u32>) -> u32 {
        self.create_mutex_with_status(name, initial_owner).0
    }

    pub fn create_mutex_with_status(
        &mut self,
        name: Option<String>,
        initial_owner: Option<u32>,
    ) -> (u32, bool) {
        if let Some(name) = name.as_deref() {
            if let Some((handle, _)) = self.objects.iter().find(|(_, object)| {
                matches!(object, KernelObject::Mutex(mutex) if mutex.name.as_deref() == Some(name))
            }) {
                return (*handle, true);
            }
        }
        let handle = self.insert(KernelObject::Mutex(MutexObject {
            name,
            owner_thread: initial_owner,
            lock_count: u32::from(initial_owner.is_some()),
        }));
        (handle, false)
    }

    pub fn create_semaphore(
        &mut self,
        name: Option<String>,
        initial_count: i32,
        maximum_count: i32,
    ) -> Option<u32> {
        if maximum_count <= 0 || initial_count < 0 || initial_count > maximum_count {
            return None;
        }
        if let Some(name) = name.as_deref() {
            if let Some((handle, _)) = self.objects.iter().find(|(_, object)| {
                matches!(
                    object,
                    KernelObject::Semaphore(semaphore)
                        if semaphore.name.as_deref() == Some(name)
                )
            }) {
                return Some(*handle);
            }
        }
        Some(self.insert(KernelObject::Semaphore(SemaphoreObject {
            name,
            count: initial_count,
            maximum: maximum_count,
        })))
    }

    pub fn release_semaphore(&mut self, handle: u32, release_count: i32) -> Option<i32> {
        if release_count <= 0 {
            return None;
        }
        let Ok(KernelObject::Semaphore(semaphore)) = self.get_mut(handle) else {
            return None;
        };
        let previous = semaphore.count;
        if semaphore.count.saturating_add(release_count) > semaphore.maximum {
            return None;
        }
        semaphore.count += release_count;
        Some(previous)
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
            priority: CE_THREAD_PRIORITY_NORMAL,
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
            views: Vec::new(),
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
                KernelObject::FileMapping(mapping) if mapping.views.iter().any(|view| view.base == base)
            )
        })
    }

    pub fn file_mapping_view(&self, base: u32) -> Option<(&FileMappingObject, FileMappingView)> {
        self.objects.values().find_map(|object| match object {
            KernelObject::FileMapping(mapping) => mapping
                .views
                .iter()
                .copied()
                .find(|view| view.base == base)
                .map(|view| (mapping, view)),
            _ => None,
        })
    }

    pub fn file_mapping_by_view_mut(
        &mut self,
        base: u32,
    ) -> Option<(&mut FileMappingObject, FileMappingView)> {
        self.objects.values_mut().find_map(|object| match object {
            KernelObject::FileMapping(mapping) => mapping
                .views
                .iter()
                .copied()
                .find(|view| view.base == base)
                .map(|view| (mapping, view)),
            _ => None,
        })
    }

    pub fn remove_file_mapping_view(&mut self, base: u32) -> Option<FileMappingView> {
        self.objects.values_mut().find_map(|object| match object {
            KernelObject::FileMapping(mapping) => mapping
                .views
                .iter()
                .position(|view| view.base == base)
                .map(|index| mapping.views.remove(index)),
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

    pub fn thread_id(&self, handle: u32) -> Option<u32> {
        let Ok(KernelObject::Thread(thread)) = self.get(handle) else {
            return None;
        };
        Some(thread.thread_id)
    }

    pub fn thread_exit_code(&self, handle: u32) -> Option<u32> {
        let Ok(KernelObject::Thread(thread)) = self.get(handle) else {
            return None;
        };
        Some(thread.exit_code)
    }

    pub fn suspend_thread(&mut self, handle: u32) -> ThreadSuspendResult {
        let Ok(KernelObject::Thread(thread)) = self.get_mut(handle) else {
            return ThreadSuspendResult::InvalidHandle;
        };
        Self::suspend_thread_object(thread)
    }

    pub fn suspend_thread_by_id(&mut self, thread_id: u32) -> Option<ThreadSuspendResult> {
        let thread = self.objects.values_mut().find_map(|object| match object {
            KernelObject::Thread(thread) if thread.thread_id == thread_id => Some(thread),
            _ => None,
        })?;
        Some(Self::suspend_thread_object(thread))
    }

    fn suspend_thread_object(thread: &mut ThreadObject) -> ThreadSuspendResult {
        let previous = thread.suspend_count;
        if previous == MAX_SUSPEND_COUNT {
            return ThreadSuspendResult::SignalRefused;
        }
        thread.suspend_count += 1;
        ThreadSuspendResult::Previous(previous)
    }

    pub fn resume_thread(&mut self, handle: u32) -> ThreadResumeResult {
        let Ok(KernelObject::Thread(thread)) = self.get_mut(handle) else {
            return ThreadResumeResult::InvalidHandle;
        };
        Self::resume_thread_object(thread)
    }

    pub fn resume_thread_by_id(&mut self, thread_id: u32) -> Option<ThreadResumeResult> {
        let thread = self.objects.values_mut().find_map(|object| match object {
            KernelObject::Thread(thread) if thread.thread_id == thread_id => Some(thread),
            _ => None,
        })?;
        Some(Self::resume_thread_object(thread))
    }

    fn resume_thread_object(thread: &mut ThreadObject) -> ThreadResumeResult {
        let previous = thread.suspend_count;
        if previous > 0 {
            thread.suspend_count -= 1;
        }
        ThreadResumeResult::Previous(previous)
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
        if !(0..MAX_CE_PRIORITY_LEVELS).contains(&priority) {
            return false;
        }
        let Ok(KernelObject::Thread(thread)) = self.get_mut(handle) else {
            return false;
        };
        thread.priority = priority;
        true
    }

    pub fn set_thread_priority_by_id(&mut self, thread_id: u32, priority: i32) -> Option<bool> {
        if !(0..MAX_CE_PRIORITY_LEVELS).contains(&priority) {
            return Some(false);
        }
        let thread = self.objects.values_mut().find_map(|object| match object {
            KernelObject::Thread(thread) if thread.thread_id == thread_id => Some(thread),
            _ => None,
        })?;
        thread.priority = priority;
        Some(true)
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

    pub fn process_exit_code(&self, handle: u32) -> Option<u32> {
        let Ok(KernelObject::Process(process)) = self.get(handle) else {
            return None;
        };
        Some(process.exit_code)
    }

    pub fn process_id(&self, handle: u32) -> Option<u32> {
        let Ok(KernelObject::Process(process)) = self.get(handle) else {
            return None;
        };
        Some(process.process_id)
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
        if mutex.owner_thread != Some(thread_id) {
            return false;
        }
        if mutex.lock_count > 1 {
            mutex.lock_count -= 1;
        } else {
            mutex.lock_count = 0;
            mutex.owner_thread = None;
        }
        true
    }

    pub fn mutex_lock_count(&self, handle: u32) -> Option<u32> {
        let Ok(KernelObject::Mutex(mutex)) = self.get(handle) else {
            return None;
        };
        Some(mutex.lock_count)
    }

    pub fn is_mutex(&self, handle: u32) -> bool {
        matches!(self.get(handle), Ok(KernelObject::Mutex(_)))
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
                if mutex.owner_thread == Some(thread_id) {
                    if mutex.lock_count == MUTEX_MAX_LOCK_COUNT {
                        return WaitResult::Failed;
                    }
                    mutex.lock_count += 1;
                    return WaitResult::Object0;
                }
                mutex.owner_thread = Some(thread_id);
                mutex.lock_count = 1;
                WaitResult::Object0
            }
            KernelObject::Semaphore(semaphore) if semaphore.count > 0 => {
                semaphore.count -= 1;
                WaitResult::Object0
            }
            KernelObject::File(_)
            | KernelObject::FindFile(_)
            | KernelObject::Device(_)
            | KernelObject::Window(_)
            | KernelObject::WaveOut(_)
            | KernelObject::FileMapping(_)
            | KernelObject::CriticalSection(_) => WaitResult::Failed,
            KernelObject::Thread(thread) if thread.signaled => WaitResult::Object0,
            KernelObject::Process(process) if process.signaled => WaitResult::Object0,
            _ if timeout_ms == 0 => WaitResult::Timeout,
            _ => WaitResult::Timeout,
        }
    }

    pub fn wait_for_any_object(&mut self, handles: &[u32], thread_id: u32) -> WaitMultipleResult {
        if handles
            .iter()
            .any(|handle| self.is_wait_ready(*handle, thread_id).is_none())
        {
            return WaitMultipleResult::Failed;
        }

        let Some((index, handle)) = handles
            .iter()
            .enumerate()
            .find(|(_, handle)| self.is_wait_ready(**handle, thread_id) == Some(true))
        else {
            return WaitMultipleResult::Timeout;
        };

        match self.wait_for_single_object(*handle, 0, thread_id) {
            WaitResult::Object0 => WaitMultipleResult::Object(index as u32),
            WaitResult::Timeout => WaitMultipleResult::Timeout,
            WaitResult::Failed => WaitMultipleResult::Failed,
        }
    }

    pub fn is_wait_ready(&self, handle: u32, thread_id: u32) -> Option<bool> {
        let object = self.get(handle).ok()?;
        Some(match object {
            KernelObject::Event(event) => event.signaled,
            KernelObject::Mutex(mutex) => {
                mutex.owner_thread.is_none() || mutex.owner_thread == Some(thread_id)
            }
            KernelObject::Semaphore(semaphore) => semaphore.count > 0,
            KernelObject::Thread(thread) => thread.signaled,
            KernelObject::Process(process) => process.signaled,
            KernelObject::File(_)
            | KernelObject::FindFile(_)
            | KernelObject::Device(_)
            | KernelObject::Window(_)
            | KernelObject::WaveOut(_)
            | KernelObject::FileMapping(_)
            | KernelObject::CriticalSection(_) => return None,
        })
    }
}

pub fn win32_thread_priority_to_ce(priority: u32) -> Option<i32> {
    (priority < MAX_WIN32_PRIORITY_LEVELS)
        .then_some(MAX_CE_PRIORITY_LEVELS - MAX_WIN32_PRIORITY_LEVELS as i32 + priority as i32)
}

pub fn ce_thread_priority_to_win32(priority: i32) -> Option<u32> {
    if !(0..MAX_CE_PRIORITY_LEVELS).contains(&priority) {
        return None;
    }
    let base = MAX_CE_PRIORITY_LEVELS - MAX_WIN32_PRIORITY_LEVELS as i32;
    Some(if priority < base {
        THREAD_PRIORITY_TIME_CRITICAL
    } else {
        (priority - base) as u32
    })
}
