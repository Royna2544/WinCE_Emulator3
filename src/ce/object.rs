use std::collections::{BTreeMap, BTreeSet};

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
    DeviceSearch(DeviceSearchObject),
    Volume(VolumeObject),
    Store(StoreObject),
    StoreSearch(StoreSearchObject),
    Partition(PartitionObject),
    PartitionSearch(PartitionSearchObject),
    FileChangeNotification(FileChangeNotificationObject),
    DeviceNotification(DeviceNotificationObject),
    MessageQueue(MessageQueueHandleObject),
    Device(DeviceSession),
    Window(u32),
    WaveOut(u32),
    PowerRequirement(PowerRequirementObject),
    PowerNotification(PowerNotificationObject),
    PowerRelationship(PowerRelationshipObject),
    FileMapping(FileMappingObject),
    CriticalSection(CriticalSectionObject),
    ApiSet(ApiSetObject),
    ApiHandle(ApiHandleObject),
    Thread(ThreadHandleObject),
    Process(ProcessHandleObject),
}

#[derive(Debug, Clone)]
pub struct EventObject {
    pub name: Option<String>,
    pub manual_reset: bool,
    pub signaled: bool,
    pub data: u32,
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
    pub notify_change_pending: bool,
}

#[derive(Debug, Clone)]
pub struct FindFileObject {
    pub guest_pattern: String,
    pub find_id: u32,
}

#[derive(Debug, Clone)]
pub struct DeviceSearchObject {
    pub handles: Vec<u32>,
    pub cursor: usize,
}

#[derive(Debug, Clone)]
pub struct VolumeObject {
    pub owner_process_id: u32,
    pub guest_root: String,
    pub disk_ptr: Option<u32>,
    pub fsd_volume_context: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct StoreObject {
    pub guest_root: String,
}

#[derive(Debug, Clone)]
pub struct StoreSearchObject {
    pub guest_roots: Vec<String>,
    pub cursor: usize,
}

#[derive(Debug, Clone)]
pub struct PartitionObject {
    pub store_guest_root: String,
    pub partition_name: String,
}

#[derive(Debug, Clone)]
pub struct PartitionSearchObject {
    pub store_guest_root: String,
    pub partition_names: Vec<String>,
    pub cursor: usize,
}

#[derive(Debug, Clone)]
pub struct FileChangeNotificationObject {
    pub owner_process_id: u32,
    pub watch_path: String,
    pub volume_root: Option<String>,
    pub recursive: bool,
    pub notify_filter: u32,
    pub signaled: bool,
    pub pending_signal_count: usize,
    pub pending: Vec<FileChangeRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChangeRecord {
    pub action: u32,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct DeviceNotificationObject {
    pub class_guid: [u8; 16],
    pub message_queue: u32,
    pub all: bool,
}

#[derive(Debug, Clone)]
pub struct MessageQueueHandleObject {
    pub queue_id: u32,
    pub read_access: bool,
}

#[derive(Debug, Clone)]
pub struct PowerRequirementObject {
    pub device_name: Option<String>,
    pub device_state: u32,
    pub device_flags: u32,
    pub system_state: Option<String>,
    pub system_flags: u32,
}

#[derive(Debug, Clone)]
pub struct PowerNotificationObject {
    pub message_queue: u32,
    pub flags: u32,
}

#[derive(Debug, Clone)]
pub struct PowerRelationshipObject {
    pub parent: Option<String>,
    pub child: Option<String>,
    pub capabilities: Option<Vec<u8>>,
    pub flags: u32,
}

#[derive(Debug, Clone)]
pub struct FileMappingObject {
    pub name: Option<String>,
    pub size: u32,
    pub protect: u32,
    pub file_id: Option<u32>,
    backing: MappingBacking,
    pub views: Vec<FileMappingView>,
    pub closed: bool,
}

/// A mapping has exactly one byte authority. Legacy DIB/injected views use
/// copied bytes; public pointer-mapped views use MemorySystem's stable slab.
#[derive(Debug, Clone)]
pub enum MappingBacking {
    Legacy(Vec<u8>),
    Slab,
}

impl FileMappingObject {
    pub fn legacy_data(&self) -> Option<&[u8]> {
        match &self.backing {
            MappingBacking::Legacy(data) => Some(data),
            MappingBacking::Slab => None,
        }
    }

    pub fn legacy_data_mut(&mut self) -> Option<&mut Vec<u8>> {
        match &mut self.backing {
            MappingBacking::Legacy(data) => Some(data),
            MappingBacking::Slab => None,
        }
    }

    pub fn is_slab(&self) -> bool {
        matches!(self.backing, MappingBacking::Slab)
    }

    /// Call only after the first public view's canonical pages have passed
    /// preflight. Dropping the legacy Vec is irreversible; no live legacy view
    /// may still depend on it.
    pub fn transition_to_slab(&mut self) -> bool {
        if !matches!(self.backing, MappingBacking::Legacy(_)) || !self.views.is_empty() {
            return false;
        }
        self.backing = MappingBacking::Slab;
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileMappingView {
    pub base: u32,
    pub size: u32,
    pub offset: u32,
    /// Process that created the view. Cross-process coherence syncs a view's
    /// guest pages from its owner's CPU only; other processes' copies of the
    /// same pages are stale snapshots and must not overwrite the mapping data.
    pub process_id: u32,
    /// Public slab views retain the issuing birth. Legacy injected/DIB views
    /// have no birth and keep their old PID-only compatibility path.
    pub process_birth: Option<ProcessBirthKey>,
}

#[derive(Debug, Clone)]
pub struct CriticalSectionObject {
    pub guest_ptr: u32,
}

#[derive(Debug, Clone)]
pub struct ApiSetObject {
    pub name: [u8; 4],
    pub methods: Vec<u32>,
    pub signatures: Vec<u64>,
    pub registered_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ApiHandleObject {
    pub api_set_handle: u32,
    pub data: u32,
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

/// Identity of one canonical thread record. Public handles are aliases to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThreadBirthKey {
    thread_id: u32,
    incarnation: u64,
}

#[derive(Debug, Clone)]
pub struct ThreadHandleObject {
    thread_id: u32,
    incarnation: u64,
}

impl ThreadHandleObject {
    pub fn thread_id(&self) -> u32 {
        self.thread_id
    }
}

#[derive(Debug, Clone)]
struct ThreadRecord {
    object: ThreadObject,
    incarnation: u64,
    aliases: BTreeSet<u32>,
    execution_live: bool,
}

#[derive(Debug, Clone)]
pub struct ProcessObject {
    pub process_id: u32,
    pub exit_code: u32,
    pub signaled: bool,
    pub debugger_thread_id: Option<u32>,
    incarnation: u64,
    execution_live: bool,
    aliases: BTreeSet<u32>,
}

/// An immutable identity issued by the canonical process-birth directory.
/// A selected PID alone is not sufficient to own process-scoped VM metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessBirthKey {
    process_id: u32,
    incarnation: u64,
}

impl ProcessBirthKey {
    pub fn process_id(self) -> u32 {
        self.process_id
    }

    pub fn incarnation(self) -> u64 {
        self.incarnation
    }
}

/// A public handle names a birth; mutable process state lives only in HandleTable.
#[derive(Debug, Clone)]
pub struct ProcessHandleObject {
    process_id: u32,
    incarnation: u64,
}

impl ProcessHandleObject {
    pub fn process_id(&self) -> u32 {
        self.process_id
    }
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
    processes: BTreeMap<u32, ProcessObject>,
    next_process_incarnation: u64,
    threads: BTreeMap<u32, ThreadRecord>,
    next_thread_incarnation: u64,
}

impl Default for HandleTable {
    fn default() -> Self {
        Self {
            next: 0x100,
            objects: BTreeMap::new(),
            processes: BTreeMap::new(),
            next_process_incarnation: 1,
            threads: BTreeMap::new(),
            next_thread_incarnation: 1,
        }
    }
}

impl HandleTable {
    pub fn insert(&mut self, object: KernelObject) -> u32 {
        self.try_insert(object)
            .expect("public handle allocation failed")
    }

    /// Publish one public alias only after its canonical target and handle ID validate.
    pub fn try_insert(&mut self, object: KernelObject) -> Result<u32> {
        let process_alias = match &object {
            KernelObject::Process(alias) => {
                if !self
                    .processes
                    .get(&alias.process_id)
                    .is_some_and(|process| process.incarnation == alias.incarnation)
                {
                    return Err(Error::InvalidHandle(alias.process_id));
                }
                Some(alias.process_id)
            }
            _ => None,
        };
        let thread_alias = match &object {
            KernelObject::Thread(alias) => {
                if !self
                    .threads
                    .get(&alias.thread_id)
                    .is_some_and(|thread| thread.incarnation == alias.incarnation)
                {
                    return Err(Error::InvalidHandle(alias.thread_id));
                }
                Some(alias.thread_id)
            }
            _ => None,
        };
        let handle = self.next;
        self.next = self
            .next
            .checked_add(4)
            .ok_or_else(|| Error::OutOfStructures("public handle ID exhausted".to_owned()))?;
        self.objects.insert(handle, object);
        if let Some(process_id) = process_alias {
            self.processes
                .get_mut(&process_id)
                .expect("validated process birth")
                .aliases
                .insert(handle);
        }
        if let Some(thread_id) = thread_alias {
            self.threads
                .get_mut(&thread_id)
                .expect("validated thread birth")
                .aliases
                .insert(handle);
        }
        Ok(handle)
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
        let object = self
            .objects
            .remove(&handle)
            .ok_or(Error::InvalidHandle(handle))?;
        if let KernelObject::Process(alias) = &object {
            if let Some(process) = self.processes.get_mut(&alias.process_id)
                && process.incarnation == alias.incarnation
            {
                process.aliases.remove(&handle);
                self.retire_process_if_unreferenced(alias.process_id);
            }
        }
        if let KernelObject::Thread(alias) = &object {
            if let Some(thread) = self.threads.get_mut(&alias.thread_id)
                && thread.incarnation == alias.incarnation
            {
                thread.aliases.remove(&handle);
                self.retire_thread_if_unreferenced(alias.thread_id);
            }
        }
        Ok(object)
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (u32, &mut KernelObject)> {
        self.objects
            .iter_mut()
            .map(|(handle, object)| (*handle, object))
    }

    pub fn iter(&self) -> impl Iterator<Item = (u32, &KernelObject)> {
        self.objects
            .iter()
            .map(|(handle, object)| (*handle, object))
    }

    pub fn describe_handle(&self, handle: u32) -> String {
        match self.get(handle) {
            Ok(KernelObject::Event(event)) => format!(
                "event(name={},manual={},signaled={},data=0x{:08x})",
                event.name.as_deref().unwrap_or("<unnamed>"),
                event.manual_reset,
                event.signaled,
                event.data
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
            Ok(KernelObject::DeviceSearch(search)) => format!(
                "device_search(cursor={},entries={})",
                search.cursor,
                search.handles.len()
            ),
            Ok(KernelObject::Volume(volume)) => format!(
                "volume(owner={},root={},disk={})",
                volume.owner_process_id,
                volume.guest_root,
                volume
                    .disk_ptr
                    .map(|disk| format!("0x{disk:08x}"))
                    .unwrap_or_else(|| "none".to_owned())
            ),
            Ok(KernelObject::Store(store)) => format!("store(root={})", store.guest_root),
            Ok(KernelObject::StoreSearch(search)) => format!(
                "store_search(cursor={},entries={})",
                search.cursor,
                search.guest_roots.len()
            ),
            Ok(KernelObject::Partition(partition)) => format!(
                "partition(store={},name={})",
                partition.store_guest_root, partition.partition_name
            ),
            Ok(KernelObject::PartitionSearch(search)) => format!(
                "partition_search(store={},cursor={},entries={})",
                search.store_guest_root,
                search.cursor,
                search.partition_names.len()
            ),
            Ok(KernelObject::FileChangeNotification(notification)) => format!(
                "file_change(owner={},path={},recursive={},filter=0x{:08x},signaled={})",
                notification.owner_process_id,
                notification.watch_path,
                notification.recursive,
                notification.notify_filter,
                notification.signaled
            ),
            Ok(KernelObject::DeviceNotification(notification)) => format!(
                "device_notification(queue=0x{:08x},all={},class={:02x?})",
                notification.message_queue, notification.all, notification.class_guid
            ),
            Ok(KernelObject::MessageQueue(queue)) => format!(
                "msg_queue(id={},access={})",
                queue.queue_id,
                if queue.read_access { "read" } else { "write" }
            ),
            Ok(KernelObject::Device(_)) => "device".to_owned(),
            Ok(KernelObject::Window(hwnd)) => format!("window(hwnd=0x{hwnd:08x})"),
            Ok(KernelObject::WaveOut(id)) => format!("waveout(id={id})"),
            Ok(KernelObject::PowerRequirement(requirement)) => format!(
                "power_requirement(device={},state={},device_flags=0x{:08x},system={},system_flags=0x{:08x})",
                requirement.device_name.as_deref().unwrap_or("<none>"),
                requirement.device_state,
                requirement.device_flags,
                requirement.system_state.as_deref().unwrap_or("<none>"),
                requirement.system_flags
            ),
            Ok(KernelObject::PowerNotification(notification)) => format!(
                "power_notification(queue=0x{:08x},flags=0x{:08x})",
                notification.message_queue, notification.flags
            ),
            Ok(KernelObject::PowerRelationship(relationship)) => format!(
                "power_relationship(parent={},child={},caps={},flags=0x{:08x})",
                relationship.parent.as_deref().unwrap_or("<none>"),
                relationship.child.as_deref().unwrap_or("<none>"),
                relationship.capabilities.as_ref().map_or(0, Vec::len),
                relationship.flags
            ),
            Ok(KernelObject::FileMapping(mapping)) => format!(
                "mapping(name={},size={},views={},closed={})",
                mapping.name.as_deref().unwrap_or("<unnamed>"),
                mapping.size,
                mapping.views.len(),
                mapping.closed
            ),
            Ok(KernelObject::CriticalSection(cs)) => {
                format!("critical_section(ptr=0x{:08x})", cs.guest_ptr)
            }
            Ok(KernelObject::ApiSet(api_set)) => format!(
                "api_set(name={},methods={},registered={})",
                String::from_utf8_lossy(&api_set.name),
                api_set.methods.len(),
                api_set
                    .registered_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "none".to_owned())
            ),
            Ok(KernelObject::ApiHandle(api_handle)) => format!(
                "api_handle(set=0x{:08x},data=0x{:08x})",
                api_handle.api_set_handle, api_handle.data
            ),
            Ok(KernelObject::Thread(alias)) => self
                .thread_for_alias(alias)
                .map(|record| {
                    let thread = &record.object;
                    format!(
                        "thread(id={},signaled={},suspend={},exit=0x{:08x})",
                        thread.thread_id, thread.signaled, thread.suspend_count, thread.exit_code
                    )
                })
                .unwrap_or_else(|| "invalid thread birth".to_owned()),
            Ok(KernelObject::Process(alias)) => self
                .process_for_alias(alias)
                .map(|process| {
                    format!(
                        "process(id={},signaled={},exit=0x{:08x})",
                        process.process_id, process.signaled, process.exit_code
                    )
                })
                .unwrap_or_else(|| "invalid process birth".to_owned()),
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
            data: 0,
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

    pub fn named_event_signaled(&self, name: &str) -> Option<bool> {
        self.objects.iter().find_map(|(_, object)| match object {
            KernelObject::Event(event) if event.name.as_deref() == Some(name) => {
                Some(event.signaled)
            }
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
        assert!(
            !self.threads.contains_key(&thread_id),
            "thread TID must not alias an existing birth"
        );
        let incarnation = self.next_thread_incarnation;
        self.next_thread_incarnation = self
            .next_thread_incarnation
            .checked_add(1)
            .expect("thread incarnation exhausted");
        self.threads.insert(
            thread_id,
            ThreadRecord {
                object: ThreadObject {
                    thread_id,
                    start_address,
                    parameter,
                    exit_code: 259,
                    priority: CE_THREAD_PRIORITY_NORMAL,
                    signaled: false,
                    suspend_count: u32::from(suspended),
                },
                incarnation,
                aliases: BTreeSet::new(),
                execution_live: true,
            },
        );
        self.insert(KernelObject::Thread(ThreadHandleObject {
            thread_id,
            incarnation,
        }))
    }

    /// Undo a birth whose only alias has not yet been returned to the guest.
    /// A failed CreateThread output write must not leave an execution-live
    /// record that a later OpenThread can discover.
    pub fn discard_unpublished_thread(&mut self, handle: u32, thread_id: u32) -> bool {
        let Some(KernelObject::Thread(alias)) = self.objects.get(&handle) else {
            return false;
        };
        if alias.thread_id != thread_id {
            return false;
        }
        let Some(record) = self.threads.get(&thread_id) else {
            return false;
        };
        if record.incarnation != alias.incarnation
            || record.aliases.len() != 1
            || !record.aliases.contains(&handle)
            || !record.execution_live
        {
            return false;
        }
        self.objects.remove(&handle);
        self.threads.remove(&thread_id);
        true
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
            backing: MappingBacking::Legacy(if file_id.is_some() {
                Vec::new()
            } else {
                vec![0; size as usize]
            }),
            views: Vec::new(),
            closed: false,
        }))
    }

    fn register_process_birth(&mut self, process_id: u32) -> u64 {
        assert!(
            !self.processes.contains_key(&process_id),
            "process PID must not alias an existing birth"
        );
        let incarnation = self.next_process_incarnation;
        self.next_process_incarnation = self
            .next_process_incarnation
            .checked_add(1)
            .expect("process incarnation exhausted");
        self.processes.insert(
            process_id,
            ProcessObject {
                process_id,
                exit_code: 259,
                signaled: false,
                debugger_thread_id: None,
                incarnation,
                execution_live: true,
                aliases: BTreeSet::new(),
            },
        );
        incarnation
    }

    pub fn register_boot_process(&mut self, process_id: u32) {
        self.register_process_birth(process_id);
    }

    pub fn create_process(&mut self, process_id: u32) -> u32 {
        let incarnation = self.register_process_birth(process_id);
        self.insert(KernelObject::Process(ProcessHandleObject {
            process_id,
            incarnation,
        }))
    }

    pub fn open_process(&mut self, process_id: u32) -> Option<u32> {
        let process = self.processes.get(&process_id)?;
        // A termination request can precede the scheduler's execution cleanup.
        // Its birth remains pinned internally, but after the last public alias
        // closes the terminated PID must not become externally reopenable.
        if process.signaled && process.aliases.is_empty() {
            return None;
        }
        let incarnation = process.incarnation;
        self.try_insert(KernelObject::Process(ProcessHandleObject {
            process_id,
            incarnation,
        }))
        .ok()
    }

    pub fn process_snapshots(&self) -> Vec<ProcessObject> {
        self.processes
            .values()
            .filter(|process| !process.signaled || !process.aliases.is_empty())
            .cloned()
            .collect()
    }

    pub fn process_birth_key(&self, process_id: u32) -> Option<ProcessBirthKey> {
        let process = self.processes.get(&process_id)?;
        Some(ProcessBirthKey {
            process_id,
            incarnation: process.incarnation,
        })
    }

    pub fn process_birth_key_for_handle(&self, handle: u32) -> Option<ProcessBirthKey> {
        let KernelObject::Process(alias) = self.get(handle).ok()? else {
            return None;
        };
        self.process_for_alias(alias)?;
        Some(ProcessBirthKey {
            process_id: alias.process_id,
            incarnation: alias.incarnation,
        })
    }

    pub fn contains_process_birth(&self, key: ProcessBirthKey) -> bool {
        self.process_birth_key(key.process_id) == Some(key)
    }

    fn process_for_alias(&self, alias: &ProcessHandleObject) -> Option<&ProcessObject> {
        self.processes
            .get(&alias.process_id)
            .filter(|process| process.incarnation == alias.incarnation)
    }

    fn process_for_handle(&self, handle: u32) -> Option<&ProcessObject> {
        let KernelObject::Process(alias) = self.get(handle).ok()? else {
            return None;
        };
        self.process_for_alias(alias)
    }

    fn retire_process_if_unreferenced(&mut self, process_id: u32) {
        if self
            .processes
            .get(&process_id)
            .is_some_and(|process| !process.execution_live && process.aliases.is_empty())
        {
            self.processes.remove(&process_id);
        }
    }

    fn thread_for_alias(&self, alias: &ThreadHandleObject) -> Option<&ThreadRecord> {
        self.threads
            .get(&alias.thread_id)
            .filter(|record| record.incarnation == alias.incarnation)
    }

    fn thread_for_handle(&self, handle: u32) -> Option<&ThreadRecord> {
        let KernelObject::Thread(alias) = self.get(handle).ok()? else {
            return None;
        };
        self.thread_for_alias(alias)
    }

    fn thread_for_handle_mut(&mut self, handle: u32) -> Option<&mut ThreadRecord> {
        let KernelObject::Thread(alias) = self.get(handle).ok()? else {
            return None;
        };
        let key = ThreadBirthKey {
            thread_id: alias.thread_id,
            incarnation: alias.incarnation,
        };
        self.threads
            .get_mut(&key.thread_id)
            .filter(|record| record.incarnation == key.incarnation)
    }

    fn retire_thread_if_unreferenced(&mut self, thread_id: u32) {
        if self
            .threads
            .get(&thread_id)
            .is_some_and(|record| !record.execution_live && record.aliases.is_empty())
        {
            self.threads.remove(&thread_id);
        }
    }

    pub fn thread_birth_key_for_handle(&self, handle: u32) -> Option<ThreadBirthKey> {
        let KernelObject::Thread(alias) = self.get(handle).ok()? else {
            return None;
        };
        self.thread_for_alias(alias)?;
        Some(ThreadBirthKey {
            thread_id: alias.thread_id,
            incarnation: alias.incarnation,
        })
    }

    pub fn thread_birth_key_by_id(&self, thread_id: u32) -> Option<ThreadBirthKey> {
        let record = self.threads.get(&thread_id)?;
        Some(ThreadBirthKey {
            thread_id,
            incarnation: record.incarnation,
        })
    }

    pub fn contains_thread_birth(&self, key: ThreadBirthKey) -> bool {
        self.thread_birth_key_by_id(key.thread_id) == Some(key)
    }

    pub fn open_thread(&mut self, thread_id: u32) -> Option<u32> {
        let key = self.thread_birth_key_by_id(thread_id)?;
        self.try_insert(KernelObject::Thread(ThreadHandleObject {
            thread_id: key.thread_id,
            incarnation: key.incarnation,
        }))
        .ok()
    }

    pub fn process_alias_handles(&self, process_id: u32) -> Vec<u32> {
        self.processes
            .get(&process_id)
            .map(|process| process.aliases.iter().copied().collect())
            .unwrap_or_default()
    }

    pub fn finish_process_execution_by_id(
        &mut self,
        process_id: u32,
        exit_code: u32,
    ) -> Option<Vec<u32>> {
        let process = self.processes.get_mut(&process_id)?;
        process.exit_code = exit_code;
        process.signaled = true;
        process.execution_live = false;
        let aliases = process.aliases.iter().copied().collect();
        self.retire_process_if_unreferenced(process_id);
        Some(aliases)
    }

    pub fn process_signaled_by_id(&self, process_id: u32) -> Option<bool> {
        self.processes
            .get(&process_id)
            .map(|process| process.signaled)
    }

    pub fn process_exit_code_by_id(&self, process_id: u32) -> Option<u32> {
        self.processes
            .get(&process_id)
            .map(|process| process.exit_code)
    }

    pub fn thread_snapshots(&self) -> Vec<ThreadObject> {
        // Preserve the existing public-snapshot boundary until execution and
        // wait pins are migrated. A live record with no alias is internal only.
        self.threads
            .values()
            .filter(|record| !record.aliases.is_empty())
            .map(|record| record.object.clone())
            .collect()
    }

    pub fn file_mapping(&self, handle: u32) -> Result<&FileMappingObject> {
        match self.get(handle)? {
            KernelObject::FileMapping(mapping) if !mapping.closed => Ok(mapping),
            _ => Err(Error::InvalidHandle(handle)),
        }
    }

    pub fn file_mapping_mut(&mut self, handle: u32) -> Result<&mut FileMappingObject> {
        match self.get_mut(handle)? {
            KernelObject::FileMapping(mapping) if !mapping.closed => Ok(mapping),
            _ => Err(Error::InvalidHandle(handle)),
        }
    }

    pub fn close_file_mapping_handle_with_live_views(&mut self, handle: u32) -> Result<bool> {
        match self.get_mut(handle)? {
            KernelObject::FileMapping(mapping) if mapping.closed => {
                Err(Error::InvalidHandle(handle))
            }
            KernelObject::FileMapping(mapping) if !mapping.views.is_empty() => {
                mapping.closed = true;
                Ok(true)
            }
            KernelObject::FileMapping(_) => Ok(false),
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
                .find(|view| view.base == base && view.process_birth.is_none())
                .map(|view| (mapping, view)),
            _ => None,
        })
    }

    pub fn file_mapping_view_for_birth(
        &self,
        birth: ProcessBirthKey,
        base: u32,
    ) -> Option<(u32, &FileMappingObject, FileMappingView)> {
        self.objects
            .iter()
            .find_map(|(handle, object)| match object {
                KernelObject::FileMapping(mapping) => mapping
                    .views
                    .iter()
                    .copied()
                    .find(|view| view.base == base && view.process_birth == Some(birth))
                    .map(|view| (*handle, mapping, view)),
                _ => None,
            })
    }

    pub fn remove_file_mapping_view_for_birth(
        &mut self,
        birth: ProcessBirthKey,
        base: u32,
    ) -> Option<(u32, FileMappingView)> {
        let (handle, view) = self
            .objects
            .iter_mut()
            .find_map(|(handle, object)| match object {
                KernelObject::FileMapping(mapping) => mapping
                    .views
                    .iter()
                    .position(|view| view.base == base && view.process_birth == Some(birth))
                    .map(|index| (*handle, mapping.views.remove(index))),
                _ => None,
            })?;
        if matches!(
            self.objects.get(&handle),
            Some(KernelObject::FileMapping(mapping)) if mapping.closed && mapping.views.is_empty()
        ) {
            let _ = self.objects.remove(&handle);
        }
        Some((handle, view))
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
                .find(|view| view.base == base && view.process_birth.is_none())
                .map(|view| (mapping, view)),
            _ => None,
        })
    }

    pub fn remove_file_mapping_view(&mut self, base: u32) -> Option<FileMappingView> {
        let (handle, view) = self
            .objects
            .iter_mut()
            .find_map(|(handle, object)| match object {
                KernelObject::FileMapping(mapping) => mapping
                    .views
                    .iter()
                    .position(|view| view.base == base && view.process_birth.is_none())
                    .map(|index| (*handle, mapping.views.remove(index))),
                _ => None,
            })?;
        if matches!(
            self.objects.get(&handle),
            Some(KernelObject::FileMapping(mapping)) if mapping.closed && mapping.views.is_empty()
        ) {
            let _ = self.objects.remove(&handle);
        }
        Some(view)
    }

    pub fn remove_file_mapping_legacy_view_for_pid(
        &mut self,
        process_id: u32,
        base: u32,
    ) -> Option<FileMappingView> {
        let (handle, view) = self
            .objects
            .iter_mut()
            .find_map(|(handle, object)| match object {
                KernelObject::FileMapping(mapping) => mapping
                    .views
                    .iter()
                    .position(|view| {
                        view.base == base
                            && view.process_id == process_id
                            && view.process_birth.is_none()
                    })
                    .map(|index| (*handle, mapping.views.remove(index))),
                _ => None,
            })?;
        if matches!(
            self.objects.get(&handle),
            Some(KernelObject::FileMapping(mapping)) if mapping.closed && mapping.views.is_empty()
        ) {
            let _ = self.objects.remove(&handle);
        }
        Some(view)
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

    pub fn has_file_mapping_object(&self, handle: u32) -> bool {
        matches!(
            self.objects.get(&handle),
            Some(KernelObject::FileMapping(_))
        )
    }

    pub fn mark_thread_exited(&mut self, handle: u32, exit_code: u32) -> bool {
        let Some(thread_id) = self.thread_id(handle) else {
            return false;
        };
        self.mark_thread_exited_by_id(thread_id, exit_code)
    }

    pub fn mark_thread_exited_by_id(&mut self, thread_id: u32, exit_code: u32) -> bool {
        let Some(record) = self.threads.get_mut(&thread_id) else {
            return false;
        };
        if !record.execution_live {
            return true;
        }
        record.object.exit_code = exit_code;
        record.object.signaled = true;
        record.execution_live = false;
        self.retire_thread_if_unreferenced(thread_id);
        true
    }

    pub fn thread_execution_live_by_id(&self, thread_id: u32) -> bool {
        self.threads
            .get(&thread_id)
            .is_some_and(|record| record.execution_live)
    }

    pub fn thread_id(&self, handle: u32) -> Option<u32> {
        Some(self.thread_for_handle(handle)?.object.thread_id)
    }

    pub fn thread_snapshot_for_handle(&self, handle: u32) -> Option<ThreadObject> {
        Some(self.thread_for_handle(handle)?.object.clone())
    }

    pub fn thread_handle_by_id(&self, thread_id: u32) -> Option<u32> {
        self.threads.get(&thread_id)?.aliases.iter().next().copied()
    }

    pub fn thread_exit_code(&self, handle: u32) -> Option<u32> {
        Some(self.thread_for_handle(handle)?.object.exit_code)
    }

    pub fn suspend_thread(&mut self, handle: u32) -> ThreadSuspendResult {
        let Some(record) = self.thread_for_handle_mut(handle) else {
            return ThreadSuspendResult::InvalidHandle;
        };
        Self::suspend_thread_object(&mut record.object)
    }

    pub fn suspend_thread_by_id(&mut self, thread_id: u32) -> Option<ThreadSuspendResult> {
        let record = self.threads.get_mut(&thread_id)?;
        Some(Self::suspend_thread_object(&mut record.object))
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
        let Some(record) = self.thread_for_handle_mut(handle) else {
            return ThreadResumeResult::InvalidHandle;
        };
        Self::resume_thread_object(&mut record.object)
    }

    pub fn resume_thread_by_id(&mut self, thread_id: u32) -> Option<ThreadResumeResult> {
        let record = self.threads.get_mut(&thread_id)?;
        Some(Self::resume_thread_object(&mut record.object))
    }

    fn resume_thread_object(thread: &mut ThreadObject) -> ThreadResumeResult {
        let previous = thread.suspend_count;
        if previous > 0 {
            thread.suspend_count -= 1;
        }
        ThreadResumeResult::Previous(previous)
    }

    pub fn thread_priority(&self, handle: u32) -> Option<i32> {
        Some(self.thread_for_handle(handle)?.object.priority)
    }

    pub fn thread_priority_by_id(&self, thread_id: u32) -> Option<i32> {
        Some(self.threads.get(&thread_id)?.object.priority)
    }

    pub fn set_thread_priority(&mut self, handle: u32, priority: i32) -> bool {
        if !(0..MAX_CE_PRIORITY_LEVELS).contains(&priority) {
            return false;
        }
        let Some(record) = self.thread_for_handle_mut(handle) else {
            return false;
        };
        record.object.priority = priority;
        true
    }

    pub fn set_thread_priority_by_id(&mut self, thread_id: u32, priority: i32) -> Option<bool> {
        if !(0..MAX_CE_PRIORITY_LEVELS).contains(&priority) {
            return Some(false);
        }
        let record = self.threads.get_mut(&thread_id)?;
        record.object.priority = priority;
        Some(true)
    }

    pub fn thread_start(&self, handle: u32) -> Option<(u32, u32, u32)> {
        let thread = &self.thread_for_handle(handle)?.object;
        (!thread.signaled && thread.suspend_count == 0).then_some((
            thread.thread_id,
            thread.start_address,
            thread.parameter,
        ))
    }

    pub fn mark_process_exited(&mut self, handle: u32, exit_code: u32) -> bool {
        let Some(process_id) = self.process_id(handle) else {
            return false;
        };
        self.finish_process_execution_by_id(process_id, exit_code)
            .is_some()
    }

    /// TerminateProcess requests death and signals the object, but the machine
    /// retains execution ownership until its actual child-completion boundary.
    pub fn request_process_termination_by_id(
        &mut self,
        process_id: u32,
        exit_code: u32,
    ) -> Option<Vec<u32>> {
        let process = self.processes.get_mut(&process_id)?;
        process.exit_code = exit_code;
        process.signaled = true;
        Some(process.aliases.iter().copied().collect())
    }

    pub fn process_exit_code(&self, handle: u32) -> Option<u32> {
        self.process_for_handle(handle)
            .map(|process| process.exit_code)
    }

    pub fn process_id(&self, handle: u32) -> Option<u32> {
        self.process_for_handle(handle)
            .map(|process| process.process_id)
    }

    pub fn process_id_exists(&self, process_id: u32) -> bool {
        self.processes
            .get(&process_id)
            .is_some_and(|process| !process.signaled || !process.aliases.is_empty())
    }

    pub fn process_debugger_thread_id(&self, process_id: u32) -> Option<u32> {
        self.processes
            .get(&process_id)
            .and_then(|process| process.debugger_thread_id)
    }

    pub fn set_process_debugger_thread_id(
        &mut self,
        process_id: u32,
        debugger_thread_id: Option<u32>,
    ) -> bool {
        let Some(process) = self.processes.get_mut(&process_id) else {
            return false;
        };
        process.debugger_thread_id = debugger_thread_id;
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

    pub fn get_event_data(&self, handle: u32) -> Option<u32> {
        let Ok(KernelObject::Event(event)) = self.get(handle) else {
            return None;
        };
        Some(event.data)
    }

    pub fn set_event_data(&mut self, handle: u32, data: u32) -> bool {
        let Ok(KernelObject::Event(event)) = self.get_mut(handle) else {
            return false;
        };
        event.data = data;
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
        if let Ok(KernelObject::Process(alias)) = self.get(handle) {
            return match self.process_for_alias(alias) {
                Some(process) if process.signaled => WaitResult::Object0,
                Some(_) => WaitResult::Timeout,
                None => WaitResult::Failed,
            };
        }
        if let Ok(KernelObject::Thread(alias)) = self.get(handle) {
            return match self.thread_for_alias(alias) {
                Some(thread) if thread.object.signaled => WaitResult::Object0,
                Some(_) => WaitResult::Timeout,
                None => WaitResult::Failed,
            };
        }
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
            KernelObject::FileChangeNotification(notification) if notification.signaled => {
                WaitResult::Object0
            }
            KernelObject::File(_)
            | KernelObject::FindFile(_)
            | KernelObject::DeviceSearch(_)
            | KernelObject::Volume(_)
            | KernelObject::Store(_)
            | KernelObject::StoreSearch(_)
            | KernelObject::Partition(_)
            | KernelObject::PartitionSearch(_)
            | KernelObject::Device(_)
            | KernelObject::DeviceNotification(_)
            | KernelObject::MessageQueue(_)
            | KernelObject::Window(_)
            | KernelObject::WaveOut(_)
            | KernelObject::PowerRequirement(_)
            | KernelObject::PowerNotification(_)
            | KernelObject::PowerRelationship(_)
            | KernelObject::FileMapping(_)
            | KernelObject::CriticalSection(_)
            | KernelObject::ApiSet(_)
            | KernelObject::ApiHandle(_) => WaitResult::Failed,
            KernelObject::Thread(_) => WaitResult::Failed,
            KernelObject::Process(_) => WaitResult::Failed,
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
            KernelObject::FileChangeNotification(notification) => notification.signaled,
            KernelObject::Thread(alias) => self.thread_for_alias(alias)?.object.signaled,
            KernelObject::Process(alias) => self.process_for_alias(alias)?.signaled,
            KernelObject::File(_)
            | KernelObject::FindFile(_)
            | KernelObject::DeviceSearch(_)
            | KernelObject::Volume(_)
            | KernelObject::Store(_)
            | KernelObject::StoreSearch(_)
            | KernelObject::Partition(_)
            | KernelObject::PartitionSearch(_)
            | KernelObject::Device(_)
            | KernelObject::DeviceNotification(_)
            | KernelObject::MessageQueue(_)
            | KernelObject::Window(_)
            | KernelObject::WaveOut(_)
            | KernelObject::PowerRequirement(_)
            | KernelObject::PowerNotification(_)
            | KernelObject::PowerRelationship(_)
            | KernelObject::FileMapping(_)
            | KernelObject::CriticalSection(_)
            | KernelObject::ApiSet(_)
            | KernelObject::ApiHandle(_) => return None,
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

#[cfg(test)]
mod alias_tests {
    use super::*;

    #[test]
    fn checked_thread_alias_exhaustion_publishes_no_reference() {
        let mut table = HandleTable::default();
        let original = table.create_thread(7, 0x1000, 0, false);
        let alias = table.get(original).unwrap().clone();
        let before_len = table.len();
        table.next = u32::MAX - 1;

        assert!(matches!(
            table.try_insert(alias),
            Err(Error::OutOfStructures(_))
        ));
        assert_eq!(table.len(), before_len);
        assert_eq!(table.thread_handle_by_id(7), Some(original));
        assert_eq!(table.thread_snapshots().len(), 1);
    }

    #[test]
    fn retired_thread_birth_rejects_stale_key_after_tid_reuse() {
        let mut table = HandleTable::default();
        let original = table.create_thread(7, 0x1000, 0, false);
        let old = table.thread_birth_key_for_handle(original).unwrap();
        assert!(table.mark_thread_exited(original, 0));
        table.close(original).unwrap();
        assert!(!table.contains_thread_birth(old));

        let replacement = table.create_thread(7, 0x2000, 0, false);
        let new = table.thread_birth_key_for_handle(replacement).unwrap();
        assert_ne!(old, new);
        assert!(!table.contains_thread_birth(old));
        assert!(table.contains_thread_birth(new));
    }
}
