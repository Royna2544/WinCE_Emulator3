use crate::{
    ce::{
        audio::{AudioSystem, MmResult, WaveBuffer, WaveFormat},
        cemath::CeMath,
        com::ComSystem,
        devices::{DeviceIoControlResult, DeviceNamespace, PURGE_RXCLEAR},
        file::{
            CREATE_ALWAYS, CREATE_NEW, DeviceInterfaceAdvertisementSpec, FILE_ATTRIBUTE_DIRECTORY,
            FileIoResult, FileIoStats, FileLockStatus, FindData, GENERIC_READ, GENERIC_WRITE,
            HostFileSystem, OPEN_ALWAYS, OPEN_EXISTING, TRUNCATE_EXISTING,
        },
        framebuffer::{Framebuffer, FramebufferBackingStore, FramebufferInfo, FramebufferRect},
        gwe::{
            FileChangeNotificationMessage, Gwe, GweStats, HWND_BROADCAST, HWND_TOP, Message,
            MessagePointerPayload, NotifyIconMessage, PeekFlags, Point, Rect, SMF_NULL,
            SWP_HIDEWINDOW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW,
            ShellNotificationMessage, WA_ACTIVE, WA_INACTIVE, WM_ACTIVATE, WM_CANCELMODE,
            WM_CAPTURECHANGED, WM_ENABLE, WM_FILECHANGEINFO, WM_HANDLESHELLNOTIFYICON,
            WM_KILLFOCUS, WM_MOVE, WM_NOTIFY, WM_SETFOCUS, WM_SHOWWINDOW, WM_SIZE,
            WM_WINDOWPOSCHANGED, WindowDeliveryOwner, WindowPos,
        },
        keybd::KeybdSystem,
        memory::{MemorySystem, PROCESS_HEAP_HANDLE},
        nled::NledSystem,
        object::{
            ApiHandleObject, ApiSetObject, CE_THREAD_PRIORITY_NORMAL, DeviceNotificationObject,
            FileChangeNotificationObject, FileChangeRecord, FileObject, FindFileObject,
            HandleTable, KernelObject, MAX_SUSPEND_COUNT, MAX_WIN32_PRIORITY_LEVELS,
            MessageQueueHandleObject, ProcessBirthKey, TaskFrameError, TaskFrameRef, TaskWaitPin,
            ThreadBirthKey, ThreadObject, ThreadResumeResult, ThreadSuspendResult, VolumeObject,
            WaitMultipleResult, WaitResult, ce_thread_priority_to_win32,
            win32_thread_priority_to_ce,
        },
        registry::{Registry, RegistryValue},
        remote::{
            CeRemote, RemoteInputPayload, RemoteStatus, WM_LBUTTONDOWN, WM_MOUSEMOVE, make_lparam,
        },
        resource::ResourceSystem,
        scheduler::{
            Scheduler, SchedulerBlockedWait, SchedulerBlockedWaitKind, SchedulerPublishError,
            SchedulerStats, SchedulerWaitKind, SchedulerWakeReason, WaitObjectKey,
        },
        shell::{
            NotifyIconData, ShellChangeNotifyRegistration, ShellNotificationCallbackMethod,
            ShellNotificationCallbackRecord, ShellNotificationRecord, ShellSystem,
        },
        thread::{
            ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, ERROR_FILE_NOT_FOUND, ERROR_INVALID_HANDLE,
            ERROR_INVALID_PARAMETER, ERROR_NOT_ENOUGH_MEMORY, ERROR_NOT_SUPPORTED,
            ERROR_OUT_OF_STRUCTURES, ERROR_SUCCESS, ThreadSystem,
        },
        timer::{TimerSystem, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT},
    },
    config::RuntimeConfig,
    error::{Error, Result},
    remote_server::RemoteServer,
};

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt::Write as _,
    path::PathBuf,
};

pub const FSDMGR_INTERNAL_PROCESS_ID: u32 = 0xffff_fffe;
const FSDMGR_SYNTHETIC_DISK_SECTOR_SIZE: u32 = 512;
const FSDMGR_FLS_SIGNATURE: &[u8; 8] = b"MSFLSH50";
const FSDMGR_FLS_HEADER_SIZE: usize = 16;
const FSDMGR_FLS_RESERVED_ENTRY_SIZE: usize = 16;
const FSDMGR_FLS_REGION_ENTRY_SIZE: usize = 28;
const IOCTL_DISK_FORMAT_VOLUME: u32 = 0x0007_0220;
const IOCTL_DISK_SCAN_VOLUME: u32 = 0x0007_0224;
const IOCTL_DISK_SET_STANDBY_TIMER: u32 = 0x0007_1c18;
const IOCTL_DISK_STANDBY_NOW: u32 = 0x0007_1c1c;
const IOCTL_DISK_DELETE_CLUSTER: u32 = 0x0007_1c40;
const IOCTL_DISK_READ_CDROM: u32 = 0x0007_1c44;
const IOCTL_DISK_WRITE_CDROM: u32 = 0x0007_1c48;
const IOCTL_DISK_DELETE_SECTORS: u32 = 0x0007_1c4c;
const IOCTL_DISK_FLUSH_CACHE: u32 = 0x0007_1c54;
const IOCTL_DISK_INITIALIZED: u32 = 0x0007_1c10;
const IOCTL_DISK_FORMAT_MEDIA: u32 = 0x0007_5c14;
const DISK_IOCTL_INITIALIZED: u32 = 4;
const DISK_IOCTL_FORMAT_MEDIA: u32 = 6;
const ERROR_BUSY: u32 = 170;
const ERROR_NO_PROC_SLOTS: u32 = 89;
const RREXF_REQUEST_EXCLUSIVE: u32 = 0x0001;
pub const CE_DEFAULT_THREAD_QUANTUM_MS: u32 = 100;
pub const CE_NUM_API_SETS: u32 = 128;
const CE_METHOD_MASK: u32 = 0x00ff;
const CE_REGISTER_APISET_TYPE: u32 = 0x8000_0000;
const CE_SH_WIN32: u32 = 0;
const CE_SH_CURTOKEN: u32 = 3;
const CE_HT_EVENT: u32 = 4;
const CE_HT_MUTEX: u32 = 5;
const CE_HT_APISET: u32 = 6;
const CE_HT_CRITSEC: u32 = 12;
const CE_HT_SEMAPHORE: u32 = 13;
const CE_SH_FIRST_EXT_API_SET: u32 = 112;
const CE_SH_FIRST_EXT_HAPI_SET: u32 = 64;
const CE_SH_FIRST_OS_API_SET: u32 = 80;
const CE_SH_GDI: u32 = CE_SH_FIRST_OS_API_SET;
const CE_SH_WMGR: u32 = CE_SH_FIRST_OS_API_SET + 1;
const CE_SH_COMM: u32 = CE_SH_FIRST_OS_API_SET + 3;
const CE_SH_FILESYS_APIS: u32 = CE_SH_FIRST_OS_API_SET + 4;
const CE_SH_SHELL: u32 = CE_SH_FIRST_OS_API_SET + 5;
const CE_SH_DEVMGR_APIS: u32 = CE_SH_FIRST_OS_API_SET + 6;
const CE_BUILTIN_READY_API_SETS: &[u32] = &[
    CE_SH_WIN32,
    CE_SH_CURTHREAD,
    CE_SH_CURPROC,
    CE_SH_CURTOKEN,
    CE_HT_EVENT,
    CE_HT_MUTEX,
    CE_HT_APISET,
    CE_HT_CRITSEC,
    CE_HT_SEMAPHORE,
    CE_SH_GDI,
    CE_SH_WMGR,
    CE_SH_COMM,
    CE_SH_FILESYS_APIS,
    CE_SH_SHELL,
    CE_SH_DEVMGR_APIS,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessagePumpResult {
    Dispatched(u32),
    Quit(u32),
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RemoteTouchTargeting {
    Explicit,
    DesktopHitTest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RemoteInputDrain {
    posted: usize,
    target_thread_ids: Vec<u32>,
    detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FsdmgrCacheEntry {
    disk_ptr: u32,
    block_size: u32,
    disable_delete: bool,
    disable_flush: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FsdmgrVolumeLock {
    volume_handle: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FsdmgrFmdBlockLockRange {
    start_block: u32,
    block_count: u32,
}

#[derive(Debug, Clone, Default)]
struct IormResourceDomain {
    minimum: u32,
    count: u32,
    valid: BTreeSet<u32>,
    shareable: BTreeSet<u32>,
    use_counts: BTreeMap<u32, u16>,
    exclusive: BTreeSet<u32>,
}

fn iorm_domain_range(
    domain: &IormResourceDomain,
    id: u32,
    len: u32,
) -> Option<std::ops::Range<u32>> {
    if len == 0 || id < domain.minimum {
        return None;
    }
    let start = id.checked_sub(domain.minimum)?;
    let end = start.checked_add(len)?;
    (start < domain.count && end <= domain.count).then_some(start..end)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RemoteServerControlDrain {
    pub handled: usize,
    pub target_thread_ids: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceInterfaceAdvertisement {
    pub class_guid: [u8; 16],
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum DeviceInterfaceAdvertisementOwner {
    Global,
    Mount(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageQueueOptions {
    pub flags: u32,
    pub max_messages: u32,
    pub max_message_bytes: u32,
    pub read_access: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageQueueInfo {
    pub flags: u32,
    pub max_messages: u32,
    pub max_message_bytes: u32,
    pub current_messages: u32,
    pub max_queue_messages: u32,
    pub readers: u16,
    pub writers: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageQueueRead {
    pub bytes: Vec<u8>,
    pub flags: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageQueueReadStatus {
    Message(MessageQueueRead),
    Empty,
    BufferTooSmall(MessageQueueRead),
    Broken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageQueueWriteStatus {
    Written,
    Full,
    MessageTooLarge,
    Broken,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MessageQueueMessage {
    bytes: Vec<u8>,
    flags: u32,
}

#[derive(Debug, Clone)]
struct MessageQueueState {
    name: Option<String>,
    flags: u32,
    max_messages: u32,
    max_message_bytes: u32,
    alert: Option<MessageQueueMessage>,
    messages: VecDeque<MessageQueueMessage>,
    max_queue_messages: u32,
    readers: u16,
    writers: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingOverlappedSerialRead {
    pub handle: u32,
    pub buffer: u32,
    pub requested: u32,
    pub transferred_ptr: u32,
    pub event_handle: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledAppAtTime {
    pub app_name: String,
    pub system_time: [u16; 8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledAppAtEvent {
    pub app_name: String,
    pub event: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserNotificationTrigger {
    pub trigger_type: u32,
    pub event: u32,
    pub application: Option<String>,
    pub arguments: Option<String>,
    pub start_time: [u16; 8],
    pub end_time: [u16; 8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserNotificationAction {
    pub action_flags: u32,
    pub dialog_title: Option<String>,
    pub dialog_text: Option<String>,
    pub sound: Option<String>,
    pub max_sound: u32,
    pub expansion: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserNotificationRecord {
    pub handle: u32,
    pub trigger: UserNotificationTrigger,
    pub action: Option<UserNotificationAction>,
    pub status: u32,
}

#[derive(Debug, Clone)]
pub struct CeKernel {
    pub registry: Registry,
    pub devices: DeviceNamespace,
    pub files: HostFileSystem,
    pub handles: HandleTable,
    pub gwe: Gwe,
    pub audio: AudioSystem,
    pub math: CeMath,
    pub timers: TimerSystem,
    pub remote: CeRemote,
    pub threads: ThreadSystem,
    pub scheduler: Scheduler,
    pub resources: ResourceSystem,
    pub shell: ShellSystem,
    pub com: ComSystem,
    pub memory: MemorySystem,
    pub keybd: KeybdSystem,
    pub nled: NledSystem,
    pub remote_server: Option<RemoteServer>,
    process_module_base: u32,
    process_module_path: String,
    process_module_host_path: Option<PathBuf>,
    process_name_guest_ptrs: BTreeMap<ProcessBirthKey, (String, u32)>,
    module_heap_info_guest_ptrs: BTreeMap<ProcessBirthKey, u32>,
    typed_payload_recipients: BTreeMap<(ProcessBirthKey, u32), u32>,
    delivered_typed_payload_messages: BTreeMap<(ProcessBirthKey, u32, u32), Message>,
    process_command_line: String,
    command_line_guest_ptrs: BTreeMap<ProcessBirthKey, (String, u32)>,
    process_current_directory: Option<String>,
    process_stdio_paths: BTreeMap<(u32, u32), String>,
    process_show_cmd: u32,
    time_zone_normal_bias: i32,
    time_zone_daylight_bias: i32,
    time_zone_in_daylight: bool,
    current_process_id: u32,
    selected_process_context_generation: Option<u64>,
    current_process_exit_code: u32,
    current_process_signaled: bool,
    lowest_scheduled_win32_priority: u32,
    thread_priority_overrides: BTreeMap<u32, i32>,
    thread_suspend_counts: BTreeMap<u32, u32>,
    pending_process_launches: Vec<PendingProcessLaunch>,
    next_process_id: u32,
    loaded_modules: BTreeMap<String, LoadedModule>,
    next_loaded_module_order: u64,
    next_font_mem_resource_handle: u32,
    live_font_mem_resources: BTreeSet<u32>,
    font_resource_counts: BTreeMap<String, u32>,
    crt_rand_state: u32,
    crt_strtok_next_by_thread: BTreeMap<u32, u32>,
    crt_wcstok_next_by_thread: BTreeMap<u32, u32>,
    recent_file_ops: VecDeque<FileTraceRecord>,
    recent_file_open_ops: VecDeque<FileTraceRecord>,
    recent_process_ops: Vec<ProcessTraceRecord>,
    recent_event_ops: Vec<EventTraceRecord>,
    recent_message_ops: Vec<MessageTraceRecord>,
    recent_device_ops: Vec<DeviceTraceRecord>,
    device_interface_advertisements: BTreeSet<DeviceInterfaceAdvertisement>,
    device_interface_advertisement_owners:
        BTreeMap<DeviceInterfaceAdvertisement, BTreeSet<DeviceInterfaceAdvertisementOwner>>,
    message_queues: BTreeMap<u32, MessageQueueState>,
    named_message_queues: BTreeMap<String, u32>,
    next_message_queue_id: u32,
    fsdmgr_disk_sectors: BTreeMap<(u32, u32), Vec<u8>>,
    fsdmgr_disk_info_overrides: BTreeMap<u32, [u32; 6]>,
    fsdmgr_fmd_xip_modes: BTreeMap<u32, bool>,
    fsdmgr_fmd_block_locks: BTreeMap<u32, Vec<FsdmgrFmdBlockLockRange>>,
    fsdmgr_fmd_block_statuses: BTreeMap<(u32, u32), u32>,
    fsdmgr_fmd_sector_sizes: BTreeMap<u32, u32>,
    fsdmgr_fmd_region_tables: BTreeMap<u32, Vec<[u32; 7]>>,
    fsdmgr_fmd_reserved_regions: BTreeMap<(u32, [u8; 8]), Vec<u8>>,
    fsdmgr_fmd_interface_disk: Option<u32>,
    afs_mount_slots: BTreeMap<u32, AfsMountSlot>,
    next_afs_mount_index: u32,
    fsdmgr_volume_locks: BTreeMap<u32, FsdmgrVolumeLock>,
    afs_file_system_functions: BTreeMap<u32, u32>,
    afs_mounted_fs_notifications: BTreeMap<u32, Vec<u32>>,
    next_fsdmgr_volume_lock: u32,
    file_system_notification_callback: Option<u32>,
    scheduled_apps_at_time: BTreeMap<String, ScheduledAppAtTime>,
    scheduled_apps_at_event: BTreeMap<(u32, String), ScheduledAppAtEvent>,
    handled_app_notifications: BTreeSet<String>,
    user_notifications: BTreeMap<u32, UserNotificationRecord>,
    next_user_notification_handle: u32,
    modal_dialog_results: BTreeMap<(u32, u32), u32>,
    system_power_state_name: Option<String>,
    system_power_state_flags: u32,
    device_power_states: BTreeMap<String, u32>,
    live_pump_timeout_stop_tick: Option<u32>,
    runtime_loader_stats: RuntimeLoaderStats,
    pulsed_wait_handles: BTreeMap<u64, u32>,
    comm_event_mask_changed_waits: BTreeSet<u64>,
    pending_overlapped_serial_reads: BTreeMap<u32, PendingOverlappedSerialRead>,
    font_families: Vec<CeFontFamily>,
    fsdmgr_caches: Vec<Option<FsdmgrCacheEntry>>,
    iorm_resource_domains: BTreeMap<u32, IormResourceDomain>,
    thread_quantums: BTreeMap<u32, u32>,
    display_gamma_value: u32,
    display_rotation: u32,
    display_lcdcon3_high_nibble: u8,
    display_perf_timings: Vec<DisplayPerfTiming>,
    display_perf_unhandled: u32,
    window_backing_stores: BTreeMap<u32, FramebufferBackingStore>,
    api_sets: BTreeMap<u32, u32>,
}

/// Failure to prepare an ordered object wait before it becomes visible to the
/// scheduler. No wait record is published and any earlier task pins unwind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitPrepError {
    InvalidHandle { index: usize, handle: u32 },
    StaleTarget { index: usize, pin: TaskWaitPin },
    PinExhausted { index: usize, pin: TaskWaitPin },
}

/// An owner changed between capture and publication. This is an emulator
/// handoff failure, not a guest wait result or CE LastError value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitPublishError {
    MissingOwnerBirth {
        thread_id: u32,
    },
    StaleOwnerBirth {
        expected: ThreadBirthKey,
        current: Option<ThreadBirthKey>,
    },
    ExitedOwnerBirth {
        owner: ThreadBirthKey,
    },
}

/// Failures before a finite Sleep frame becomes guest-visible. Native callers
/// treat these as backend handoff failures, never as a CE Sleep return.
#[derive(Debug)]
pub enum FiniteSleepPublishError {
    Owner(WaitPublishError),
    Frame(TaskFrameError),
    Scheduler(SchedulerPublishError),
}

impl std::fmt::Display for WaitPublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingOwnerBirth { thread_id } => {
                write!(f, "thread {thread_id} has no captured wait-owner birth")
            }
            Self::StaleOwnerBirth { expected, current } => {
                write!(
                    f,
                    "wait-owner birth {expected:?} is stale (current {current:?})"
                )
            }
            Self::ExitedOwnerBirth { owner } => {
                write!(f, "wait-owner birth {owner:?} has exited")
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ModalDialogWaitOwner {
    Birth(ThreadBirthKey),
    // Only selected when this numeric TID has no canonical live birth.
    LegacyUnknownTid(u32),
}

impl ModalDialogWaitOwner {
    fn matches(self, wait: &SchedulerBlockedWait) -> bool {
        match self {
            Self::Birth(birth) => {
                wait.thread_id == birth.thread_id() && wait.owner_birth == Some(birth)
            }
            Self::LegacyUnknownTid(thread_id) => {
                wait.thread_id == thread_id && wait.owner_birth.is_none()
            }
        }
    }
}

impl WaitPrepError {
    pub fn last_error(self) -> u32 {
        match self {
            Self::InvalidHandle { .. } | Self::StaleTarget { .. } => ERROR_INVALID_HANDLE,
            Self::PinExhausted { .. } => ERROR_NOT_ENOUGH_MEMORY,
        }
    }
}

/// A wait whose ordered target references are locked but whose scheduler
/// record has not yet been published. Dropping it releases every acquired
/// reference, including duplicate slots, in reverse acquisition order.
pub struct PreparedTaskWait<'a> {
    kernel: &'a mut CeKernel,
    thread_id: u32,
    owner_birth: Option<ThreadBirthKey>,
    thread_handle: u32,
    wait_handles: Vec<u32>,
    wait_keys: Vec<WaitObjectKey>,
    pinned_targets: Vec<Option<TaskWaitPin>>,
    kind: SchedulerBlockedWaitKind,
    wait_started_ms: u32,
    timeout_ms: u32,
    published: bool,
}

impl PreparedTaskWait<'_> {
    /// Canonical birth captured before this wait can be published.
    pub(crate) fn owner_birth(&self) -> Option<ThreadBirthKey> {
        self.owner_birth
    }

    /// Allows the caller to finish its existing native handoff while the
    /// unpublished wait keeps its exact target births pinned.
    pub fn kernel_mut(&mut self) -> &mut CeKernel {
        self.kernel
    }

    /// Require the captured owner to still be the exact live task before a
    /// native caller mutates its prior continuation or pending return.
    pub fn preflight_owner_birth(&self) -> std::result::Result<ThreadBirthKey, WaitPublishError> {
        let Some(owner_birth) = self.owner_birth else {
            return Err(WaitPublishError::MissingOwnerBirth {
                thread_id: self.thread_id,
            });
        };
        self.kernel
            .validate_live_wait_owner_birth(owner_birth, self.thread_id)
    }

    /// Native publication repeats the exact live-owner check at the insertion
    /// boundary. An error drops this guard and releases every acquired pin.
    pub fn try_publish(mut self) -> std::result::Result<u64, WaitPublishError> {
        let owner_birth = self.preflight_owner_birth()?;
        let wait_id = self
            .kernel
            .scheduler
            .register_blocked_wait_for_birth_with_keys_and_pins(
                owner_birth,
                self.thread_handle,
                std::mem::take(&mut self.wait_handles),
                std::mem::take(&mut self.wait_keys),
                std::mem::take(&mut self.pinned_targets),
                self.kind,
                self.wait_started_ms,
                self.timeout_ms,
            );
        self.published = true;
        Ok(wait_id)
    }

    /// Legacy fixture publication, including deliberately unknown owners.
    /// Production native callers must use `try_publish` instead.
    /// Transfers the prepared pins into the scheduler record exactly once.
    /// Aligned slots have already been built by preparation, so scheduler
    /// registration has no recoverable validation failure at this boundary.
    pub fn publish(mut self) -> u64 {
        let wait_handles = std::mem::take(&mut self.wait_handles);
        let wait_keys = std::mem::take(&mut self.wait_keys);
        let pinned_targets = std::mem::take(&mut self.pinned_targets);
        let wait_id = match self.owner_birth {
            Some(birth) => self
                .kernel
                .scheduler
                .register_blocked_wait_for_birth_with_keys_and_pins(
                    birth,
                    self.thread_handle,
                    wait_handles,
                    wait_keys,
                    pinned_targets,
                    self.kind,
                    self.wait_started_ms,
                    self.timeout_ms,
                ),
            None => self
                .kernel
                .scheduler
                .register_legacy_blocked_wait_with_keys_and_pins(
                    self.thread_id,
                    self.thread_handle,
                    wait_handles,
                    wait_keys,
                    pinned_targets,
                    self.kind,
                    self.wait_started_ms,
                    self.timeout_ms,
                ),
        };
        self.published = true;
        wait_id
    }
}

impl Drop for PreparedTaskWait<'_> {
    fn drop(&mut self) {
        if !self.published {
            for pin in self.pinned_targets.iter().flatten().rev() {
                let released = self.kernel.handles.release_wait_target(*pin);
                if !std::thread::panicking() {
                    assert!(released, "prepared task wait lost an acquired pin");
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct AfsMountSlot {
    name: String,
    guest_root: String,
    owner_process_id: u32,
    volume_handle: Option<u32>,
}

const FIRST_USER_AFS_INDEX: u32 = 2;
const INVALID_MOUNT_INDEX: u32 = u32::MAX;
const ERROR_INVALID_INDEX: u32 = 1413;

/// Font family entry for CE system font enumeration (EnumFontFamiliesExW).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CeFontFamily {
    /// Face name (UTF-8, max 31 chars when encoded to UTF-16).
    pub face_name: String,
    pub charset: u8,
    pub pitch_and_family: u8,
    pub is_truetype: bool,
}

fn ce_system_font_families() -> Vec<CeFontFamily> {
    // CE 6.0 Korean device: gulim.ttc Korean faces + CE system Western faces.
    const HANGUL: u8 = 129;
    const ANSI: u8 = 0;
    vec![
        CeFontFamily {
            face_name: "\u{AD74}\u{B9BC}".to_owned(),
            charset: HANGUL,
            pitch_and_family: 0x22,
            is_truetype: true,
        }, // 굴림
        CeFontFamily {
            face_name: "\u{AD74}\u{B9BC}\u{CC54}".to_owned(),
            charset: HANGUL,
            pitch_and_family: 0x31,
            is_truetype: true,
        }, // 굴림체
        CeFontFamily {
            face_name: "\u{B3CB}\u{C6C0}".to_owned(),
            charset: HANGUL,
            pitch_and_family: 0x22,
            is_truetype: true,
        }, // 돋움
        CeFontFamily {
            face_name: "\u{B3CB}\u{C6C0}\u{CC54}".to_owned(),
            charset: HANGUL,
            pitch_and_family: 0x31,
            is_truetype: true,
        }, // 돋움체
        CeFontFamily {
            face_name: "\u{BC14}\u{D0D5}".to_owned(),
            charset: HANGUL,
            pitch_and_family: 0x12,
            is_truetype: true,
        }, // 바탕
        CeFontFamily {
            face_name: "\u{BC14}\u{D0D5}\u{CC54}".to_owned(),
            charset: HANGUL,
            pitch_and_family: 0x11,
            is_truetype: true,
        }, // 바탕체
        CeFontFamily {
            face_name: "Tahoma".to_owned(),
            charset: ANSI,
            pitch_and_family: 0x22,
            is_truetype: true,
        },
        CeFontFamily {
            face_name: "Arial".to_owned(),
            charset: ANSI,
            pitch_and_family: 0x22,
            is_truetype: true,
        },
        CeFontFamily {
            face_name: "Courier New".to_owned(),
            charset: ANSI,
            pitch_and_family: 0x31,
            is_truetype: true,
        },
    ]
}

fn framebuffer_rect_from_gwe_rect(info: FramebufferInfo, rect: Rect) -> Option<FramebufferRect> {
    let rect = rect.normalized();
    let left = rect.left.max(0).min(info.width as i32) as u32;
    let top = rect.top.max(0).min(info.height as i32) as u32;
    let right = rect.right.max(0).min(info.width as i32) as u32;
    let bottom = rect.bottom.max(0).min(info.height as i32) as u32;
    (right > left && bottom > top).then_some(FramebufferRect::new(
        left,
        top,
        right - left,
        bottom - top,
    ))
}

// CE 6.0 NLS enumeration data for the Korean device image (locale.txt in
// PUBLIC\COMMON\OAK\FILES): English (0409) and Korean (0412) locales.

/// EnumSystemLocalesW strings: 8-hex-digit LCIDs of locales in the OS image.
pub fn ce_nls_system_locales() -> Vec<String> {
    vec!["00000409".to_owned(), "00000412".to_owned()]
}

/// EnumSystemCodePagesW strings: code pages in the Korean OS image.
pub fn ce_nls_system_code_pages() -> Vec<String> {
    vec!["437".to_owned(), "949".to_owned(), "1252".to_owned()]
}

/// EnumUILanguagesW strings: 4-hex-digit UI language ids (Korean image).
pub fn ce_nls_ui_languages() -> Vec<String> {
    vec!["0412".to_owned()]
}

/// Resolve an LCID to one of the image's locale tables: Korean (0x12) or
/// English (0x09); LANG_NEUTRAL/system/user defaults map to Korean.
fn ce_nls_resolve_locale(locale: u32) -> Option<u16> {
    match locale as u16 & 0x03ff {
        0x0000 | 0x0012 => Some(0x0412),
        0x0009 => Some(0x0409),
        _ => None,
    }
}

const KO_SHORT_DATES: &[&str] = &["yyyy-MM-dd", "yy-MM-dd", "yy-M-d", "yyyy-M-d"];
const KO_LONG_DATES: &[&str] = &[
    "yyyy'\u{B144}' M'\u{C6D4}' d'\u{C77C}' dddd",
    "yyyy'\u{B144}' M'\u{C6D4}' d'\u{C77C}'",
    "yy'\u{B144}' M'\u{C6D4}' d'\u{C77C}' dddd",
    "yy'\u{B144}' M'\u{C6D4}' d'\u{C77C}'",
    "yyyy'\u{B144}' MM'\u{C6D4}' dd'\u{C77C}' dddd",
    "yyyy'\u{B144}' MM'\u{C6D4}' dd'\u{C77C}'",
];
const KO_YEAR_MONTH: &[&str] = &["yyyy'\u{B144}' M'\u{C6D4}'"];
const KO_TIME_FORMATS: &[&str] = &["tt h:mm:ss", "tt hh:mm:ss", "H:mm:ss", "HH:mm:ss"];

const EN_SHORT_DATES: &[&str] = &[
    "M/d/yyyy",
    "M/d/yy",
    "MM/dd/yy",
    "MM/dd/yyyy",
    "yy/MM/dd",
    "yyyy-MM-dd",
    "dd-MMM-yy",
];
const EN_LONG_DATES: &[&str] = &[
    "dddd, MMMM dd, yyyy",
    "MMMM dd, yyyy",
    "dddd, dd MMMM, yyyy",
    "dd MMMM, yyyy",
];
const EN_YEAR_MONTH: &[&str] = &["MMMM, yyyy"];
const EN_TIME_FORMATS: &[&str] = &["h:mm:ss tt", "hh:mm:ss tt", "H:mm:ss", "HH:mm:ss"];

/// EnumDateFormatsW strings for a locale and DATE_* flag (0 = short).
pub fn ce_nls_date_formats(locale: u32, flags: u32) -> Option<Vec<String>> {
    const DATE_SHORTDATE: u32 = 0x0001;
    const DATE_LONGDATE: u32 = 0x0002;
    const DATE_YEARMONTH: u32 = 0x0008;
    let lang = ce_nls_resolve_locale(locale)?;
    let table = match flags {
        0 | DATE_SHORTDATE => {
            if lang == 0x0412 {
                KO_SHORT_DATES
            } else {
                EN_SHORT_DATES
            }
        }
        DATE_LONGDATE => {
            if lang == 0x0412 {
                KO_LONG_DATES
            } else {
                EN_LONG_DATES
            }
        }
        DATE_YEARMONTH => {
            if lang == 0x0412 {
                KO_YEAR_MONTH
            } else {
                EN_YEAR_MONTH
            }
        }
        _ => return None,
    };
    Some(table.iter().map(|s| (*s).to_owned()).collect())
}

/// EnumTimeFormatsW strings for a locale; TIME_NOSECONDS strips ":ss".
pub fn ce_nls_time_formats(locale: u32, flags: u32) -> Option<Vec<String>> {
    const TIME_NOSECONDS: u32 = 0x0002;
    let lang = ce_nls_resolve_locale(locale)?;
    let table = if lang == 0x0412 {
        KO_TIME_FORMATS
    } else {
        EN_TIME_FORMATS
    };
    let mut formats: Vec<String> = match flags {
        0 => table.iter().map(|s| (*s).to_owned()).collect(),
        TIME_NOSECONDS => table.iter().map(|s| s.replace(":ss", "")).collect(),
        _ => return None,
    };
    formats.dedup();
    Some(formats)
}

/// EnumCalendarInfoW strings: calendars from locale.txt IOPTIONALCALENDAR
/// (Korean: Gregorian-Korean 1, Gregorian-English 2, Tangun era 5) and the
/// CALENDAR table format pictures (calendar 1 defers to locale defaults).
pub fn ce_nls_calendar_info(locale: u32, calendar: u32, cal_type: u32) -> Option<Vec<String>> {
    const ENUM_ALL_CALENDARS: u32 = 0xffff_ffff;
    const CAL_ICALINTVALUE: u32 = 0x0001;
    const CAL_SCALNAME: u32 = 0x0002;
    const CAL_SSHORTDATE: u32 = 0x0005;
    const CAL_SLONGDATE: u32 = 0x0006;
    const CAL_SYEARMONTH: u32 = 0x002f;
    const CAL_NOUSEROVERRIDE: u32 = 0x8000_0000;

    let lang = ce_nls_resolve_locale(locale)?;
    // (calendar id, native calendar name) per locale.txt IOPTIONALCALENDAR.
    let calendars: &[(u32, &str)] = if lang == 0x0412 {
        &[
            (1, "\u{C11C}\u{AE30} (\u{D55C}\u{AE00})"),
            (2, "\u{C11C}\u{AE30} (\u{C601}\u{C5B4})"),
            (5, "\u{B2E8}\u{AE30}"),
        ]
    } else {
        &[(1, "Gregorian Calendar")]
    };
    let selected: Vec<(u32, &str)> = if calendar == ENUM_ALL_CALENDARS {
        calendars.to_vec()
    } else {
        calendars
            .iter()
            .copied()
            .filter(|(id, _)| *id == calendar)
            .collect()
    };
    if selected.is_empty() {
        return None;
    }

    let mut out = Vec::new();
    for (id, name) in selected {
        match cal_type & !CAL_NOUSEROVERRIDE {
            CAL_ICALINTVALUE => out.push(id.to_string()),
            CAL_SCALNAME => out.push(name.to_owned()),
            CAL_SSHORTDATE => match id {
                // Calendar 1 pictures are \x0000 in the CALENDAR table: locale default.
                1 => out.extend(ce_nls_date_formats(locale, 0x0001)?),
                2 => out.extend(EN_SHORT_DATES[..4].iter().map(|s| (*s).to_owned())),
                5 => out.push("gg yyyy-MM-dd".to_owned()),
                _ => return None,
            },
            CAL_SLONGDATE => match id {
                1 => out.extend(ce_nls_date_formats(locale, 0x0002)?),
                2 => out.extend(EN_LONG_DATES[..2].iter().map(|s| (*s).to_owned())),
                5 => out.extend(
                    [
                        "gg yyyy'\u{B144}' M'\u{C6D4}' d'\u{C77C}' dddd",
                        "gg yyyy'\u{B144}' M'\u{C6D4}' d'\u{C77C}'",
                        "gg yyyy'\u{B144}' MM'\u{C6D4}' dd'\u{C77C}' dddd",
                        "gg yyyy'\u{B144}' MM'\u{C6D4}' dd'\u{C77C}'",
                    ]
                    .iter()
                    .map(|s| (*s).to_owned()),
                ),
                _ => return None,
            },
            CAL_SYEARMONTH => match id {
                1 => out.extend(ce_nls_date_formats(locale, 0x0008)?),
                2 => out.extend(EN_YEAR_MONTH.iter().map(|s| (*s).to_owned())),
                5 => out.push("gg yyyy'\u{B144}' M'\u{C6D4}'".to_owned()),
                _ => return None,
            },
            _ => return None,
        }
    }
    Some(out)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RuntimeLoaderStats {
    pub load_attempt_count: u64,
    pub successful_map_count: u64,
    pub dependency_load_count: u64,
    pub export_lookup_count: u64,
    pub export_lookup_miss_count: u64,
    pub forwarded_export_count: u64,
    pub tls_callback_count: u64,
    pub dllmain_attach_count: u64,
    pub dllmain_detach_count: u64,
    pub loud_failure_count: u64,
}

const DONT_RESOLVE_DLL_REFERENCES_FLAG: u32 = 0x0000_0001;
const LOAD_LIBRARY_AS_DATAFILE_FLAG: u32 = 0x0000_0002;
const NO_IMPORT_LOAD_FLAGS: u32 = DONT_RESOLVE_DLL_REFERENCES_FLAG | LOAD_LIBRARY_AS_DATAFILE_FLAG;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedModule {
    pub name: String,
    pub base: u32,
    pub guest_path: Option<String>,
    pub host_path: Option<PathBuf>,
    pub image_size: u32,
    pub entry_point: u32,
    pub exports_by_name: BTreeMap<String, u32>,
    pub exports_by_ordinal: BTreeMap<u32, u32>,
    pub forwarders_by_name: BTreeMap<String, String>,
    pub forwarders_by_ordinal: BTreeMap<u32, String>,
    pub dependencies: Vec<String>,
    pub tls_callbacks: Vec<u32>,
    pub ref_count: u32,
    pub load_flags: u32,
    pub dynamic: bool,
    pub unload_pending: bool,
    pub thread_library_calls_disabled: bool,
    load_order: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedModuleMetadata {
    pub guest_path: Option<String>,
    pub host_path: Option<PathBuf>,
    pub image_size: u32,
    pub entry_point: u32,
    pub dependencies: Vec<String>,
    pub tls_callbacks: Vec<u32>,
    pub forwarders_by_name: BTreeMap<String, String>,
    pub forwarders_by_ordinal: BTreeMap<u32, String>,
    pub ref_count: u32,
    pub load_flags: u32,
    pub dynamic: bool,
    pub thread_library_calls_disabled: bool,
}

impl Default for LoadedModuleMetadata {
    fn default() -> Self {
        Self {
            guest_path: None,
            host_path: None,
            image_size: 0,
            entry_point: 0,
            dependencies: Vec::new(),
            tls_callbacks: Vec::new(),
            forwarders_by_name: BTreeMap::new(),
            forwarders_by_ordinal: BTreeMap::new(),
            ref_count: 1,
            load_flags: 0,
            dynamic: false,
            thread_library_calls_disabled: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedModuleSnapshot {
    pub name: String,
    pub base: u32,
    pub ref_count: u32,
    pub dynamic: bool,
    pub unload_pending: bool,
    pub guest_path: Option<String>,
    pub host_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedModuleExportSnapshot {
    pub name: String,
    pub base: u32,
    pub exports_by_name: BTreeMap<String, u32>,
    pub exports_by_ordinal: BTreeMap<u32, u32>,
    pub forwarders_by_name: BTreeMap<String, String>,
    pub forwarders_by_ordinal: BTreeMap<u32, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreeLibraryResult {
    InvalidHandle,
    Pinned,
    StillReferenced { ref_count: u32 },
    UnloadPending,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingProcessLaunch {
    pub application: Option<String>,
    pub command_line: Option<String>,
    pub current_directory: Option<String>,
    pub show_cmd: Option<u32>,
    pub process_handle: u32,
    pub thread_handle: u32,
    pub process_id: u32,
    pub thread_id: u32,
    /// Present for an original launch; legacy trace/parked reconstruction may
    /// not be able to recover these immutable identities after alias closure.
    pub process_birth: Option<ProcessBirthKey>,
    pub primary_thread_birth: Option<ThreadBirthKey>,
}

/// Proof returned only after the exact process birth's stopped exit has
/// passed framebuffer preflight and canonical completion. A termination
/// request or a reused numeric PID cannot mint this witness.
#[derive(Debug)]
pub struct AcceptedProcessExit {
    birth: ProcessBirthKey,
}

impl AcceptedProcessExit {
    pub fn birth(&self) -> ProcessBirthKey {
        self.birth
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolhelpProcessSnapshot {
    pub process_id: u32,
    pub default_heap: u32,
    pub thread_count: u32,
    pub priority_base: i32,
    pub flags: u32,
    pub memory_base: u32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolhelpThreadSnapshot {
    pub thread_id: u32,
    pub owner_process_id: u32,
    pub base_priority: i32,
    pub current_priority: i32,
    pub flags: u32,
    pub current_process_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolhelpModuleSnapshot {
    pub module_id: u32,
    pub process_id: u32,
    pub ref_count: u32,
    pub base: u32,
    pub size: u32,
    pub flags: u32,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentProcessState {
    pub process_id: u32,
    pub exit_code: u32,
    pub signaled: bool,
}

/// Provenance for the Kernel's mutable selected-process context. This is not
/// a restoration token or a machine/checkpoint-lineage identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SelectedProcessContextStamp {
    birth: ProcessBirthKey,
    generation: u64,
}

impl SelectedProcessContextStamp {
    pub(crate) fn birth(self) -> ProcessBirthKey {
        self.birth
    }

    pub(crate) fn generation(self) -> u64 {
        self.generation
    }
}

fn toolhelp_process_basename(path: &str) -> String {
    let name = path
        .rsplit(|ch| ch == '\\' || ch == '/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(path);
    if name.is_empty() {
        "process.exe".to_owned()
    } else {
        name.to_owned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTraceRecord {
    pub op: &'static str,
    pub handle: Option<u32>,
    pub path: Option<String>,
    pub preview: Option<String>,
    pub requested: Option<u32>,
    pub transferred: Option<u32>,
    pub position: Option<u64>,
    pub result: Option<u32>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessTraceRecord {
    pub op: &'static str,
    pub application: Option<String>,
    pub command_line: Option<String>,
    pub path: Option<String>,
    pub process_handle: Option<u32>,
    pub thread_handle: Option<u32>,
    pub process_id: Option<u32>,
    pub thread_id: Option<u32>,
    pub result: Option<u32>,
    pub error: Option<String>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayPerfTiming {
    pub rop_code: u32,
    pub c_gpe: u32,
    pub dw_gpe_time: u32,
    pub c_emul: u32,
    pub dw_emul_time: u32,
    pub c_hardware: u32,
    pub dw_hardware_time: u32,
    pub dw_wait_time: u32,
    pub blt_params: [u32; 8],
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DisplayPerfBltParams {
    pub src_in_video_mem: bool,
    pub dest_in_video_mem: bool,
    pub solid_rgb: Option<[u8; 3]>,
    pub stretch: bool,
    pub transparent: bool,
}

impl DisplayPerfTiming {
    fn new(rop_code: u32) -> Self {
        Self {
            rop_code,
            c_gpe: 0,
            dw_gpe_time: 0,
            c_emul: 0,
            dw_emul_time: 0,
            c_hardware: 0,
            dw_hardware_time: 0,
            dw_wait_time: 0,
            blt_params: [0; 8],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventTraceRecord {
    pub op: &'static str,
    pub handle: Option<u32>,
    pub name: Option<String>,
    pub manual_reset: Option<bool>,
    pub signaled: Option<bool>,
    pub result: Option<bool>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageTraceRecord {
    pub op: &'static str,
    pub thread_id: u32,
    pub hwnd: Option<u32>,
    pub msg: Option<u32>,
    pub wparam: Option<u32>,
    pub lparam: Option<u32>,
    pub screen_pos: Option<u32>,
    pub source: Option<u32>,
    pub result: Option<u32>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceTraceRecord {
    pub op: &'static str,
    pub handle: u32,
    pub device: Option<String>,
    pub backend: Option<String>,
    pub ioctl_code: u32,
    pub input_len: u32,
    pub input_preview: Option<String>,
    pub output_capacity: u32,
    pub success: bool,
    pub bytes_returned: u32,
    pub output_preview: Option<String>,
    pub detail: Option<String>,
}

const FILE_TRACE_LIMIT: usize = 512;
const FILE_TRACE_PREVIEW_LIMIT: usize = 64;
const PROCESS_TRACE_LIMIT: usize = 128;
const EVENT_TRACE_LIMIT: usize = 65536;
const MESSAGE_TRACE_LIMIT: usize = 2048;
const DEVICE_TRACE_LIMIT: usize = 512;
const SHCNE_RENAMEITEM: u32 = 0x0000_0001;
const SHCNE_CREATE: u32 = 0x0000_0002;
const SHCNE_DELETE: u32 = 0x0000_0004;
const SHCNE_MKDIR: u32 = 0x0000_0008;
const SHCNE_RMDIR: u32 = 0x0000_0010;
const SHCNE_DRIVEREMOVED: u32 = 0x0000_0080;
const SHCNE_DRIVEADD: u32 = 0x0000_0100;
const WM_DEVICECHANGE: u32 = 0x0219;
const DBT_DEVICEARRIVAL: u32 = 0x8000;
const DBT_DEVICEREMOVECOMPLETE: u32 = 0x8004;
const SHCNE_ATTRIBUTES: u32 = 0x0000_0800;
const SHCNE_UPDATEITEM: u32 = 0x0000_2000;
const SHCNF_IDLIST: u32 = 0x0000_0000;
const SHCNF_PATHW: u32 = 0x0000_0005;
const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
const FILE_NOTIFY_CHANGE_DIR_NAME: u32 = 0x0000_0002;
const FILE_NOTIFY_CHANGE_ATTRIBUTES: u32 = 0x0000_0004;
const FILE_NOTIFY_CHANGE_SIZE: u32 = 0x0000_0008;
const FILE_NOTIFY_CHANGE_LAST_WRITE: u32 = 0x0000_0010;
const FILE_NOTIFY_CHANGE_LAST_ACCESS: u32 = 0x0000_0020;
const FILE_NOTIFY_CHANGE_CREATION: u32 = 0x0000_0040;
const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
const FILE_ACTION_ADDED: u32 = 0x0000_0001;
const FILE_ACTION_REMOVED: u32 = 0x0000_0002;
const FILE_ACTION_MODIFIED: u32 = 0x0000_0003;
const FILE_ACTION_RENAMED_OLD_NAME: u32 = 0x0000_0004;
const FILE_ACTION_RENAMED_NEW_NAME: u32 = 0x0000_0005;
const FILE_ACTION_CHANGE_COMPLETED: u32 = 0x0001_0000;
const FILECHANGENOTIFY_BASE_SIZE: usize = 40;

pub fn normalize_module_name(name: &str) -> String {
    name.trim()
        .trim_end_matches('\0')
        .trim_end_matches(".dll")
        .trim_end_matches(".DLL")
        .to_ascii_lowercase()
}

pub fn normalize_symbol_name(name: &str) -> String {
    name.trim_start_matches('_')
        .split('@')
        .next()
        .unwrap_or(name)
        .to_ascii_lowercase()
}

fn wait_result_to_wake_reason(result: u32) -> SchedulerWakeReason {
    if result == WAIT_FAILED {
        SchedulerWakeReason::Failed
    } else if result == WAIT_TIMEOUT {
        SchedulerWakeReason::Timeout
    } else {
        SchedulerWakeReason::ObjectSignaled
    }
}

fn append_trace_detail(base: String, extra: Option<String>) -> String {
    match extra {
        Some(extra) if !extra.is_empty() => format!("{base}/{extra}"),
        _ => base,
    }
}

const MAXIMUM_WAIT_OBJECTS: usize = 64;
pub const CE_SYS_HANDLE_BASE: u32 = 64;
pub const CE_SH_CURTHREAD: u32 = 1;
pub const CE_SH_CURPROC: u32 = 2;
pub const CE_CURRENT_THREAD_PSEUDO_HANDLE: u32 = CE_SYS_HANDLE_BASE + CE_SH_CURTHREAD;
pub const CE_CURRENT_PROCESS_PSEUDO_HANDLE: u32 = CE_SYS_HANDLE_BASE + CE_SH_CURPROC;
pub const STILL_ACTIVE: u32 = 259;
pub const SW_SHOWNORMAL: u32 = 1;

fn seed_testime_sample_dictionary(registry: &mut Registry) {
    const ROOT: &str = r"HKLM\SOFTWARE\Microsoft\testime\Windows\testime.DIC";

    let digits = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
    for (start, reading) in digits.iter().enumerate() {
        let candidates: Vec<String> = (0..5)
            .map(|offset| digits[(start + offset) % digits.len()].to_string())
            .collect();
        seed_testime_sample_candidates(registry, ROOT, &reading.to_string(), candidates);
    }

    for ch in 'A'..='Z' {
        seed_testime_sample_candidates(
            registry,
            ROOT,
            &ch.to_string(),
            vec![ch.to_string(), ch.to_ascii_lowercase().to_string()],
        );
    }

    seed_testime_sample_candidates(registry, ROOT, "a", vec!["a".to_owned(), "aa".to_owned()]);
    // testime.reg contains two identical [...\b\b] sections; the later default
    // value wins in regedit import before TESTIME's lowercase-section guard hides it.
    seed_testime_sample_candidates(registry, ROOT, "b", vec!["bbbbb".to_owned()]);
    for ch in 'c'..='z' {
        seed_testime_sample_candidates(
            registry,
            ROOT,
            &ch.to_string(),
            vec![ch.to_ascii_uppercase().to_string(), ch.to_string()],
        );
    }
}

fn seed_testime_sample_candidates(
    registry: &mut Registry,
    root: &str,
    reading: &str,
    candidates: Vec<String>,
) {
    let path = format!(r"{root}\{reading}");
    for candidate in candidates {
        registry.set_value(&path, &candidate, RegistryValue::string(candidate.clone()));
    }
}

fn parse_guid_string(value: &str) -> Option<[u8; 16]> {
    let trimmed = value.trim();
    let trimmed = trimmed
        .strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))
        .unwrap_or(trimmed);
    let mut parts = trimmed.split('-');
    let data1 = u32::from_str_radix(parts.next()?, 16).ok()?;
    let data2 = u16::from_str_radix(parts.next()?, 16).ok()?;
    let data3 = u16::from_str_radix(parts.next()?, 16).ok()?;
    let data4_high = parts.next()?;
    let data4_low = parts.next()?;
    if parts.next().is_some() || data4_high.len() != 4 || data4_low.len() != 12 {
        return None;
    }
    let mut guid = [0u8; 16];
    guid[0..4].copy_from_slice(&data1.to_le_bytes());
    guid[4..6].copy_from_slice(&data2.to_le_bytes());
    guid[6..8].copy_from_slice(&data3.to_le_bytes());
    parse_guid_bytes(data4_high, &mut guid[8..10])?;
    parse_guid_bytes(data4_low, &mut guid[10..16])?;
    Some(guid)
}

fn parse_guid_bytes(text: &str, out: &mut [u8]) -> Option<()> {
    if text.len() != out.len() * 2 {
        return None;
    }
    for (index, chunk) in text.as_bytes().chunks_exact(2).enumerate() {
        let chunk = std::str::from_utf8(chunk).ok()?;
        out[index] = u8::from_str_radix(chunk, 16).ok()?;
    }
    Some(())
}

fn encode_devdetail_payload(class_guid: [u8; 16], name: &str, attached: bool) -> Vec<u8> {
    let name_units: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let name_bytes = (name_units.len() * 2) as u32;
    let mut payload = Vec::with_capacity(28 + name_units.len() * 2);
    payload.extend_from_slice(&class_guid);
    payload.extend_from_slice(&0u32.to_le_bytes());
    payload.extend_from_slice(&u32::from(attached).to_le_bytes());
    payload.extend_from_slice(&name_bytes.to_le_bytes());
    for unit in name_units {
        payload.extend_from_slice(&unit.to_le_bytes());
    }
    payload
}

fn font_resource_key(path: &str) -> String {
    path.replace('/', "\\")
        .trim_end_matches('\\')
        .to_ascii_lowercase()
}

fn normalize_power_device_name(name: &str) -> String {
    name.trim().trim_end_matches(':').to_ascii_lowercase()
}

fn ce_implicit_call_address(set_id: u32, method: u32) -> u32 {
    const FIRST_METHOD_ARM: u32 = 0xf102_0000;
    const APISET_SHIFT: u32 = 8;
    const APICALL_SCALE_ARM: u32 = 4;
    let encoded = (set_id << APISET_SHIFT) | method;
    FIRST_METHOD_ARM.wrapping_sub(encoded.wrapping_mul(APICALL_SCALE_ARM))
}

impl CeKernel {
    const MSGQUEUE_MSGALERT: u32 = 0x0000_0001;
    const MSGQUEUE_NOPRECOMMIT: u32 = 0x0000_0001;
    const MSGQUEUE_ALLOW_BROKEN: u32 = 0x0000_0002;
    const MSGQUEUE_VALID_FLAGS: u32 = Self::MSGQUEUE_NOPRECOMMIT | Self::MSGQUEUE_ALLOW_BROKEN;
    const MSGQUEUE_MAX_NODE_SIZE: u32 = 0x0010_0000;
    const MSGQUEUE_MAX_NUM_NODES: u32 = 0x0010_0000;

    pub fn boot(config: RuntimeConfig) -> Self {
        let mut memory = MemorySystem::default();
        memory.configure_guest_physical_ram_bytes(config.guest_physical_ram_bytes);
        let mut registry = Registry::from_dump(config.registry);
        seed_testime_sample_dictionary(&mut registry);
        let files = HostFileSystem::from_storage(config.storage);
        let mut kernel = Self {
            registry,
            devices: DeviceNamespace::from_config(config.devices),
            files,
            handles: HandleTable::default(),
            gwe: Gwe::default(),
            audio: AudioSystem::default(),
            math: CeMath,
            timers: TimerSystem::default(),
            remote: CeRemote::default(),
            threads: ThreadSystem::default(),
            scheduler: Scheduler::default(),
            resources: ResourceSystem::default(),
            shell: ShellSystem::default(),
            com: ComSystem::default(),
            memory,
            keybd: KeybdSystem::default(),
            nled: NledSystem::default(),
            remote_server: None,
            process_module_base: 0,
            process_module_path: "\\FakeCE\\process.exe".to_owned(),
            process_module_host_path: None,
            process_name_guest_ptrs: BTreeMap::new(),
            module_heap_info_guest_ptrs: BTreeMap::new(),
            typed_payload_recipients: BTreeMap::new(),
            delivered_typed_payload_messages: BTreeMap::new(),
            process_command_line: String::new(),
            command_line_guest_ptrs: BTreeMap::new(),
            process_current_directory: None,
            process_stdio_paths: BTreeMap::new(),
            process_show_cmd: SW_SHOWNORMAL,
            time_zone_normal_bias: 480,
            time_zone_daylight_bias: 420,
            time_zone_in_daylight: false,
            current_process_id: 1,
            selected_process_context_generation: Some(1),
            current_process_exit_code: STILL_ACTIVE,
            current_process_signaled: false,
            lowest_scheduled_win32_priority: MAX_WIN32_PRIORITY_LEVELS - 1,
            thread_priority_overrides: BTreeMap::new(),
            thread_suspend_counts: BTreeMap::new(),
            pending_process_launches: Vec::new(),
            next_process_id: 0x42,
            loaded_modules: BTreeMap::new(),
            next_loaded_module_order: 0,
            next_font_mem_resource_handle: 0x5f00_0001,
            live_font_mem_resources: BTreeSet::new(),
            font_resource_counts: BTreeMap::new(),
            crt_rand_state: 1,
            crt_strtok_next_by_thread: BTreeMap::new(),
            crt_wcstok_next_by_thread: BTreeMap::new(),
            recent_file_ops: VecDeque::new(),
            recent_file_open_ops: VecDeque::new(),
            recent_process_ops: Vec::new(),
            recent_event_ops: Vec::new(),
            recent_message_ops: Vec::new(),
            recent_device_ops: Vec::new(),
            device_interface_advertisements: BTreeSet::new(),
            device_interface_advertisement_owners: BTreeMap::new(),
            message_queues: BTreeMap::new(),
            named_message_queues: BTreeMap::new(),
            next_message_queue_id: 1,
            fsdmgr_disk_sectors: BTreeMap::new(),
            fsdmgr_disk_info_overrides: BTreeMap::new(),
            fsdmgr_fmd_xip_modes: BTreeMap::new(),
            fsdmgr_fmd_block_locks: BTreeMap::new(),
            fsdmgr_fmd_block_statuses: BTreeMap::new(),
            fsdmgr_fmd_sector_sizes: BTreeMap::new(),
            fsdmgr_fmd_region_tables: BTreeMap::new(),
            fsdmgr_fmd_reserved_regions: BTreeMap::new(),
            fsdmgr_fmd_interface_disk: None,
            afs_mount_slots: BTreeMap::new(),
            next_afs_mount_index: FIRST_USER_AFS_INDEX,
            fsdmgr_volume_locks: BTreeMap::new(),
            afs_file_system_functions: BTreeMap::new(),
            afs_mounted_fs_notifications: BTreeMap::new(),
            next_fsdmgr_volume_lock: 0x6d00_0001,
            file_system_notification_callback: None,
            scheduled_apps_at_time: BTreeMap::new(),
            scheduled_apps_at_event: BTreeMap::new(),
            handled_app_notifications: BTreeSet::new(),
            user_notifications: BTreeMap::new(),
            next_user_notification_handle: 0x6e00_0001,
            modal_dialog_results: BTreeMap::new(),
            system_power_state_name: None,
            system_power_state_flags: 0x0001_0000,
            device_power_states: BTreeMap::new(),
            live_pump_timeout_stop_tick: None,
            runtime_loader_stats: RuntimeLoaderStats::default(),
            pulsed_wait_handles: BTreeMap::new(),
            comm_event_mask_changed_waits: BTreeSet::new(),
            pending_overlapped_serial_reads: BTreeMap::new(),
            font_families: ce_system_font_families(),
            fsdmgr_caches: Vec::new(),
            iorm_resource_domains: BTreeMap::new(),
            thread_quantums: BTreeMap::new(),
            display_gamma_value: 2330,
            display_rotation: 0,
            display_lcdcon3_high_nibble: 0,
            display_perf_timings: Vec::new(),
            display_perf_unhandled: 0,
            window_backing_stores: BTreeMap::new(),
            api_sets: BTreeMap::new(),
        };
        kernel.handles.register_boot_process(1);
        let boot_birth = kernel
            .handles
            .process_birth_key(1)
            .expect("boot process birth");
        kernel
            .handles
            .register_boot_thread(1, boot_birth)
            .expect("boot thread birth");
        let boot_primary = kernel.handles.thread_birth_key_by_id(1).unwrap();
        assert!(
            kernel
                .handles
                .bind_primary_thread_to_process(boot_birth, boot_primary)
        );
        kernel.publish_configured_mount_device_interfaces(true);
        kernel
    }

    pub fn runtime_loader_stats(&self) -> RuntimeLoaderStats {
        self.runtime_loader_stats
    }

    pub fn set_time_zone_biases(&mut self, normal_bias: i32, daylight_bias: i32) {
        self.time_zone_normal_bias = normal_bias;
        self.time_zone_daylight_bias = daylight_bias;
    }

    pub fn set_daylight_time(&mut self, in_daylight: bool) {
        self.time_zone_in_daylight = in_daylight;
    }

    pub fn active_time_zone_bias(&self) -> i32 {
        if self.time_zone_in_daylight {
            self.time_zone_daylight_bias
        } else {
            self.time_zone_normal_bias
        }
    }

    pub fn create_api_set(
        &mut self,
        name: [u8; 4],
        methods: Vec<u32>,
        signatures: Vec<u64>,
    ) -> u32 {
        self.handles.insert(KernelObject::ApiSet(ApiSetObject {
            name,
            methods,
            signatures,
            registered_id: None,
        }))
    }

    pub fn register_api_set(&mut self, api_set_handle: u32, mut set_id: u32) -> u32 {
        let handle_based = (set_id & CE_REGISTER_APISET_TYPE) != 0;
        set_id &= !CE_REGISTER_APISET_TYPE;

        let Ok(KernelObject::ApiSet(api_set)) = self.handles.get(api_set_handle) else {
            return ERROR_INVALID_HANDLE;
        };
        if api_set.registered_id.is_some() {
            return ERROR_INVALID_PARAMETER;
        }

        let limit = if handle_based {
            CE_SH_FIRST_OS_API_SET
        } else {
            CE_NUM_API_SETS
        };
        if set_id == 0 {
            let base = if handle_based {
                CE_SH_FIRST_EXT_HAPI_SET
            } else {
                CE_SH_FIRST_EXT_API_SET
            };
            let Some(free_id) =
                (base..limit).find(|candidate| !self.api_sets.contains_key(candidate))
            else {
                return ERROR_NO_PROC_SLOTS;
            };
            set_id = free_id;
        }

        let valid_slot = if handle_based {
            set_id < CE_SH_FIRST_OS_API_SET
        } else {
            (CE_SH_FIRST_OS_API_SET..CE_NUM_API_SETS).contains(&set_id)
        };
        if !valid_slot {
            return ERROR_NO_PROC_SLOTS;
        }
        if self.api_sets.contains_key(&set_id) {
            return ERROR_INVALID_PARAMETER;
        }

        let Ok(KernelObject::ApiSet(api_set)) = self.handles.get_mut(api_set_handle) else {
            return ERROR_INVALID_HANDLE;
        };
        api_set.registered_id = Some(set_id);
        self.api_sets.insert(set_id, api_set_handle);
        ERROR_SUCCESS
    }

    pub fn query_api_set_id(&self, name: [u8; 4]) -> Option<u32> {
        self.api_sets.iter().find_map(|(set_id, handle)| {
            matches!(
                self.handles.get(*handle),
                Ok(KernelObject::ApiSet(api_set)) if api_set.name == name
            )
            .then_some(*set_id)
        })
    }

    pub fn is_api_set_ready(&self, set_id: u32) -> bool {
        CE_BUILTIN_READY_API_SETS.contains(&set_id) || self.api_sets.contains_key(&set_id)
    }

    pub fn wait_for_api_ready(&self, set_id: u32, _timeout_ms: u32) -> u32 {
        if set_id >= CE_NUM_API_SETS {
            WAIT_FAILED
        } else if self.is_api_set_ready(set_id) {
            WAIT_OBJECT_0
        } else {
            WAIT_TIMEOUT
        }
    }

    pub fn get_api_address(&self, set_id: u32, method: u32) -> u32 {
        if !self.is_api_set_ready(set_id) || method > CE_METHOD_MASK {
            return 0;
        }
        ce_implicit_call_address(set_id, method)
    }

    pub fn create_api_handle(&mut self, api_set_handle: u32, data: u32) -> Result<u32> {
        if !matches!(self.handles.get(api_set_handle)?, KernelObject::ApiSet(_)) {
            return Err(Error::InvalidHandle(api_set_handle));
        }
        Ok(self
            .handles
            .insert(KernelObject::ApiHandle(ApiHandleObject {
                api_set_handle,
                data,
            })))
    }

    pub fn verify_api_handle(&self, api_set_handle: u32, handle: u32) -> Result<u32> {
        if !matches!(self.handles.get(api_set_handle)?, KernelObject::ApiSet(_)) {
            return Err(Error::InvalidHandle(api_set_handle));
        }
        let KernelObject::ApiHandle(api_handle) = self.handles.get(handle)? else {
            return Err(Error::InvalidHandle(handle));
        };
        if api_handle.api_set_handle == api_set_handle {
            Ok(api_handle.data)
        } else {
            Err(Error::InvalidHandle(handle))
        }
    }

    pub fn register_file_system_notification_callback(&mut self, callback: u32) {
        self.file_system_notification_callback = (callback != 0).then_some(callback);
    }

    pub fn file_system_notification_callback(&self) -> Option<u32> {
        self.file_system_notification_callback
    }

    pub fn schedule_app_at_time(&mut self, app_name: String, system_time: [u16; 8]) {
        let key = app_name.to_ascii_lowercase();
        self.scheduled_apps_at_time.insert(
            key,
            ScheduledAppAtTime {
                app_name,
                system_time,
            },
        );
    }

    pub fn remove_scheduled_app_at_time(&mut self, app_name: &str) -> bool {
        self.scheduled_apps_at_time
            .remove(&app_name.to_ascii_lowercase())
            .is_some()
    }

    pub fn scheduled_app_at_time(&self, app_name: &str) -> Option<&ScheduledAppAtTime> {
        self.scheduled_apps_at_time
            .get(&app_name.to_ascii_lowercase())
    }

    pub fn schedule_app_at_event(&mut self, app_name: String, event: u32) {
        let key = app_name.to_ascii_lowercase();
        self.scheduled_apps_at_event
            .insert((event, key), ScheduledAppAtEvent { app_name, event });
    }

    pub fn remove_scheduled_app_events(&mut self, app_name: &str) -> bool {
        let key = app_name.to_ascii_lowercase();
        let before = self.scheduled_apps_at_event.len();
        self.scheduled_apps_at_event
            .retain(|(_, app_key), _| app_key != &key);
        self.scheduled_apps_at_event.len() != before
    }

    pub fn scheduled_app_at_event(
        &self,
        app_name: &str,
        event: u32,
    ) -> Option<&ScheduledAppAtEvent> {
        self.scheduled_apps_at_event
            .get(&(event, app_name.to_ascii_lowercase()))
    }

    pub fn scheduled_apps_for_event(&self, event: u32) -> Vec<ScheduledAppAtEvent> {
        self.scheduled_apps_at_event
            .iter()
            .filter_map(|((scheduled_event, _), record)| {
                (*scheduled_event == event).then_some(record.clone())
            })
            .collect()
    }

    pub fn record_handled_app_notifications(&mut self, app_name: String) {
        self.handled_app_notifications
            .insert(app_name.to_ascii_lowercase());
    }

    pub fn handled_app_notifications(&self, app_name: &str) -> bool {
        self.handled_app_notifications
            .contains(&app_name.to_ascii_lowercase())
    }

    pub fn set_user_notification(
        &mut self,
        handle: u32,
        trigger: UserNotificationTrigger,
        action: Option<UserNotificationAction>,
    ) -> Option<u32> {
        let handle = if handle == 0 {
            let next = self.next_user_notification_handle;
            self.next_user_notification_handle = self.next_user_notification_handle.wrapping_add(1);
            next
        } else if self.user_notifications.contains_key(&handle) {
            handle
        } else {
            return None;
        };
        self.user_notifications.insert(
            handle,
            UserNotificationRecord {
                handle,
                trigger,
                action,
                status: 0,
            },
        );
        Some(handle)
    }

    pub fn clear_user_notification(&mut self, handle: u32) -> bool {
        self.user_notifications.remove(&handle).is_some()
    }

    pub fn user_notification(&self, handle: u32) -> Option<&UserNotificationRecord> {
        self.user_notifications.get(&handle)
    }

    pub fn user_notification_handles(&self) -> Vec<u32> {
        self.user_notifications.keys().copied().collect()
    }

    pub fn set_system_power_state(&mut self, name: Option<String>, flags: u32) {
        self.system_power_state_name = name.filter(|name| !name.is_empty());
        self.system_power_state_flags = flags;
    }

    pub fn system_power_state(&self) -> (Option<&str>, u32) {
        (
            self.system_power_state_name.as_deref(),
            self.system_power_state_flags,
        )
    }

    pub fn set_device_power_state(&mut self, guest_name: &str, state: u32) -> Result<()> {
        if self.devices.device_keys(guest_name).is_none() {
            return Err(Error::MissingDevice(guest_name.to_owned()));
        }
        self.device_power_states
            .insert(normalize_power_device_name(guest_name), state);
        Ok(())
    }

    pub fn get_device_power_state(&self, guest_name: &str) -> Result<u32> {
        if self.devices.device_keys(guest_name).is_none() {
            return Err(Error::MissingDevice(guest_name.to_owned()));
        }
        Ok(self
            .device_power_states
            .get(&normalize_power_device_name(guest_name))
            .copied()
            .unwrap_or(0))
    }

    pub fn resource_create_list(&mut self, resource_id: u32, minimum: u32, count: u32) -> u32 {
        if self.iorm_resource_domains.contains_key(&resource_id) {
            return ERROR_ALREADY_EXISTS;
        }
        if count == 0 {
            return ERROR_INVALID_PARAMETER;
        }
        self.iorm_resource_domains.insert(
            resource_id,
            IormResourceDomain {
                minimum,
                count,
                ..IormResourceDomain::default()
            },
        );
        ERROR_SUCCESS
    }

    pub fn resource_destroy_list(&mut self, resource_id: u32) -> u32 {
        let Some(domain) = self.iorm_resource_domains.get(&resource_id) else {
            return ERROR_FILE_NOT_FOUND;
        };
        if domain
            .valid
            .iter()
            .any(|item| domain.use_counts.get(item).copied().unwrap_or(0) != 0)
        {
            return ERROR_BUSY;
        }
        self.iorm_resource_domains.remove(&resource_id);
        ERROR_SUCCESS
    }

    pub fn resource_release(&mut self, resource_id: u32, id: u32, len: u32) -> u32 {
        let Some(domain) = self.iorm_resource_domains.get_mut(&resource_id) else {
            return ERROR_FILE_NOT_FOUND;
        };
        let Some(range) = iorm_domain_range(domain, id, len) else {
            return ERROR_INVALID_PARAMETER;
        };
        for item in range.clone() {
            if domain.valid.contains(&item)
                && domain.use_counts.get(&item).copied().unwrap_or(0) == 0
            {
                return ERROR_INVALID_PARAMETER;
            }
        }
        for item in range {
            if domain.valid.insert(item) {
                domain.use_counts.insert(item, 0);
            } else {
                let current = domain.use_counts.get(&item).copied().unwrap_or(0);
                if current == 1 {
                    domain.exclusive.remove(&item);
                }
                domain.use_counts.insert(item, current.saturating_sub(1));
            }
        }
        ERROR_SUCCESS
    }

    pub fn resource_mark_as_shareable(
        &mut self,
        resource_id: u32,
        id: u32,
        len: u32,
        shareable: bool,
    ) -> u32 {
        let Some(domain) = self.iorm_resource_domains.get_mut(&resource_id) else {
            return ERROR_INVALID_PARAMETER;
        };
        let Some(range) = iorm_domain_range(domain, id, len) else {
            return ERROR_INVALID_PARAMETER;
        };
        for item in range.clone() {
            if !domain.valid.contains(&item)
                || domain.use_counts.get(&item).copied().unwrap_or(0) != 0
            {
                return ERROR_INVALID_PARAMETER;
            }
        }
        for item in range {
            if shareable {
                domain.shareable.insert(item);
            } else {
                domain.shareable.remove(&item);
            }
        }
        ERROR_SUCCESS
    }

    pub fn resource_request_ex(&mut self, resource_id: u32, id: u32, len: u32, flags: u32) -> u32 {
        if flags & !RREXF_REQUEST_EXCLUSIVE != 0 {
            return ERROR_INVALID_PARAMETER;
        }
        let Some(domain) = self.iorm_resource_domains.get_mut(&resource_id) else {
            return ERROR_FILE_NOT_FOUND;
        };
        let Some(range) = iorm_domain_range(domain, id, len) else {
            return ERROR_INVALID_PARAMETER;
        };
        let exclusive = flags & RREXF_REQUEST_EXCLUSIVE != 0;
        for item in range.clone() {
            if !domain.valid.contains(&item) {
                return ERROR_INVALID_PARAMETER;
            }
            let use_count = domain.use_counts.get(&item).copied().unwrap_or(0);
            if use_count != 0
                && (!domain.shareable.contains(&item)
                    || exclusive
                    || domain.exclusive.contains(&item))
            {
                return ERROR_BUSY;
            }
        }
        for item in range {
            let use_count = domain.use_counts.get(&item).copied().unwrap_or(0);
            domain.use_counts.insert(item, use_count.saturating_add(1));
            if exclusive {
                domain.exclusive.insert(item);
            }
        }
        ERROR_SUCCESS
    }

    pub fn advertise_device_interface(&mut self, class_guid: [u8; 16], name: String, add: bool) {
        self.advertise_device_interface_with_owner(
            class_guid,
            name,
            DeviceInterfaceAdvertisementOwner::Global,
            add,
        );
    }

    fn advertise_device_interface_with_owner(
        &mut self,
        class_guid: [u8; 16],
        name: String,
        owner: DeviceInterfaceAdvertisementOwner,
        add: bool,
    ) {
        let advertisement = DeviceInterfaceAdvertisement { class_guid, name };
        if add {
            let owners = self
                .device_interface_advertisement_owners
                .entry(advertisement.clone())
                .or_default();
            let was_unowned = owners.is_empty();
            let inserted_owner = owners.insert(owner);
            if inserted_owner && was_unowned {
                self.device_interface_advertisements
                    .insert(advertisement.clone());
                self.queue_device_interface_notification(&advertisement, true);
            }
        } else {
            let should_remove = self
                .device_interface_advertisement_owners
                .get_mut(&advertisement)
                .is_some_and(|owners| {
                    owners.remove(&owner);
                    owners.is_empty()
                });
            if should_remove {
                self.device_interface_advertisement_owners
                    .remove(&advertisement);
                let removed = self.device_interface_advertisements.remove(&advertisement);
                debug_assert!(removed);
                self.queue_device_interface_notification(&advertisement, false);
            }
        }
    }

    pub fn advertised_device_interfaces(&self) -> &BTreeSet<DeviceInterfaceAdvertisement> {
        &self.device_interface_advertisements
    }

    pub fn enum_device_interface_advertisement(
        &self,
        index: u32,
    ) -> Option<DeviceInterfaceAdvertisement> {
        self.device_interface_advertisements
            .iter()
            .nth(index as usize)
            .cloned()
    }

    pub fn create_message_queue(
        &mut self,
        name: Option<String>,
        options: MessageQueueOptions,
    ) -> Result<(u32, bool)> {
        if options.max_message_bytes == 0 {
            return Err(Error::InvalidArgument(
                "message queue max message size is zero".to_owned(),
            ));
        }
        if options.max_message_bytes > Self::MSGQUEUE_MAX_NODE_SIZE {
            return Err(Error::InvalidArgument(
                "message queue max message size exceeds CE limit".to_owned(),
            ));
        }
        if options.max_messages > Self::MSGQUEUE_MAX_NUM_NODES {
            return Err(Error::InvalidArgument(
                "message queue max messages exceeds CE limit".to_owned(),
            ));
        }
        if options.flags & !Self::MSGQUEUE_VALID_FLAGS != 0 {
            return Err(Error::InvalidArgument(
                "message queue flags contain unsupported bits".to_owned(),
            ));
        }
        let normalized_name = name.filter(|name| !name.is_empty());
        let queue_id = if let Some(name) = normalized_name.as_ref() {
            if let Some(queue_id) = self.named_message_queues.get(name).copied() {
                queue_id
            } else {
                let queue_id = self.allocate_message_queue(normalized_name.clone(), options);
                self.named_message_queues.insert(name.clone(), queue_id);
                queue_id
            }
        } else {
            self.allocate_message_queue(None, options)
        };
        let existed = self
            .message_queues
            .get(&queue_id)
            .and_then(|queue| queue.name.as_ref())
            .is_some_and(|queue_name| {
                normalized_name
                    .as_ref()
                    .is_some_and(|name| name.eq_ignore_ascii_case(queue_name))
            })
            && normalized_name
                .as_ref()
                .and_then(|name| self.named_message_queues.get(name))
                .copied()
                == Some(queue_id)
            && self
                .message_queues
                .get(&queue_id)
                .is_some_and(|queue| queue.readers != 0 || queue.writers != 0);
        let handle = self.open_message_queue_endpoint(queue_id, options.read_access)?;
        Ok((handle, existed && normalized_name.is_some()))
    }

    fn allocate_message_queue(
        &mut self,
        name: Option<String>,
        options: MessageQueueOptions,
    ) -> u32 {
        let queue_id = self.next_message_queue_id;
        self.next_message_queue_id = self.next_message_queue_id.wrapping_add(1).max(1);
        self.message_queues.insert(
            queue_id,
            MessageQueueState {
                name,
                flags: options.flags,
                max_messages: if options.max_messages == 0 {
                    16
                } else {
                    options.max_messages
                },
                max_message_bytes: options.max_message_bytes,
                alert: None,
                messages: VecDeque::new(),
                max_queue_messages: 0,
                readers: 0,
                writers: 0,
            },
        );
        queue_id
    }

    pub fn open_message_queue(
        &mut self,
        source_queue_handle: u32,
        options: MessageQueueOptions,
    ) -> Result<u32> {
        let queue_id = self.message_queue_id(source_queue_handle)?;
        if options.max_message_bytes != 0 {
            let Some(queue) = self.message_queues.get(&queue_id) else {
                return Err(Error::InvalidHandle(source_queue_handle));
            };
            if options.max_message_bytes < queue.max_message_bytes {
                return Err(Error::InvalidArgument(
                    "message queue open max message size is too small".to_owned(),
                ));
            }
        }
        self.open_message_queue_endpoint(queue_id, options.read_access)
    }

    fn open_message_queue_endpoint(&mut self, queue_id: u32, read_access: bool) -> Result<u32> {
        let Some(queue) = self.message_queues.get_mut(&queue_id) else {
            return Err(Error::InvalidHandle(queue_id));
        };
        if read_access {
            queue.readers = queue.readers.saturating_add(1);
        } else {
            queue.writers = queue.writers.saturating_add(1);
        }
        Ok(self
            .handles
            .insert(KernelObject::MessageQueue(MessageQueueHandleObject {
                queue_id,
                read_access,
            })))
    }

    pub fn close_message_queue(&mut self, handle: u32) -> Result<()> {
        let KernelObject::MessageQueue(endpoint) = self.handles.get(handle)?.clone() else {
            return Err(Error::InvalidHandle(handle));
        };
        self.handles.close(handle)?;
        self.close_message_queue_endpoint(endpoint.queue_id, endpoint.read_access);
        Ok(())
    }

    fn close_message_queue_endpoint(&mut self, queue_id: u32, read_access: bool) {
        let Some(queue) = self.message_queues.get_mut(&queue_id) else {
            return;
        };
        if read_access {
            queue.readers = queue.readers.saturating_sub(1);
        } else {
            queue.writers = queue.writers.saturating_sub(1);
        }
        if queue.readers == 0 && queue.writers == 0 {
            if let Some(name) = queue.name.clone() {
                self.named_message_queues.remove(&name);
            }
            self.message_queues.remove(&queue_id);
        }
        self.queue_message_queue_wake_candidates(queue_id);
    }

    pub fn message_queue_info(&self, handle: u32) -> Result<MessageQueueInfo> {
        let queue_id = self.message_queue_id(handle)?;
        let Some(queue) = self.message_queues.get(&queue_id) else {
            return Err(Error::InvalidHandle(handle));
        };
        Ok(MessageQueueInfo {
            flags: queue.flags,
            max_messages: queue.max_messages,
            max_message_bytes: queue.max_message_bytes,
            current_messages: queue.messages.len() as u32,
            max_queue_messages: queue.max_queue_messages,
            readers: queue.readers,
            writers: queue.writers,
        })
    }

    pub fn read_message_queue(
        &mut self,
        handle: u32,
        buffer_bytes: u32,
    ) -> Result<MessageQueueReadStatus> {
        let KernelObject::MessageQueue(endpoint) = self.handles.get(handle)?.clone() else {
            return Err(Error::InvalidHandle(handle));
        };
        if !endpoint.read_access {
            return Err(Error::InvalidHandle(handle));
        }
        let Some(queue) = self.message_queues.get_mut(&endpoint.queue_id) else {
            return Err(Error::InvalidHandle(handle));
        };
        if queue.writers == 0 && queue.flags & Self::MSGQUEUE_ALLOW_BROKEN == 0 {
            return Ok(MessageQueueReadStatus::Broken);
        }
        let Some(message) = queue.alert.as_ref().or_else(|| queue.messages.front()) else {
            return Ok(MessageQueueReadStatus::Empty);
        };
        if buffer_bytes < message.bytes.len() as u32 {
            let mut message = queue
                .alert
                .take()
                .or_else(|| queue.messages.pop_front())
                .expect("readable message exists");
            message.bytes.truncate(buffer_bytes as usize);
            let message = MessageQueueRead {
                bytes: message.bytes,
                flags: message.flags,
            };
            self.queue_message_queue_wake_candidates(endpoint.queue_id);
            return Ok(MessageQueueReadStatus::BufferTooSmall(message));
        }
        let message = queue
            .alert
            .take()
            .or_else(|| queue.messages.pop_front())
            .expect("readable message exists");
        let message = MessageQueueRead {
            bytes: message.bytes,
            flags: message.flags,
        };
        self.queue_message_queue_wake_candidates(endpoint.queue_id);
        Ok(MessageQueueReadStatus::Message(message))
    }

    pub fn message_queue_read_would_block(&self, handle: u32) -> Result<bool> {
        let KernelObject::MessageQueue(endpoint) = self.handles.get(handle)?.clone() else {
            return Err(Error::InvalidHandle(handle));
        };
        if !endpoint.read_access {
            return Err(Error::InvalidHandle(handle));
        }
        let Some(queue) = self.message_queues.get(&endpoint.queue_id) else {
            return Err(Error::InvalidHandle(handle));
        };
        if queue.writers == 0 && queue.flags & Self::MSGQUEUE_ALLOW_BROKEN == 0 {
            return Ok(false);
        }
        Ok(queue.alert.is_none() && queue.messages.is_empty())
    }

    pub fn write_message_queue(
        &mut self,
        handle: u32,
        bytes: Vec<u8>,
        flags: u32,
    ) -> Result<MessageQueueWriteStatus> {
        let KernelObject::MessageQueue(endpoint) = self.handles.get(handle)?.clone() else {
            return Err(Error::InvalidHandle(handle));
        };
        if endpoint.read_access {
            return Err(Error::InvalidHandle(handle));
        }
        self.write_message_queue_by_id(endpoint.queue_id, bytes, flags)
    }

    pub fn message_queue_write_would_block(
        &self,
        handle: u32,
        bytes_len: u32,
        flags: u32,
    ) -> Result<bool> {
        let KernelObject::MessageQueue(endpoint) = self.handles.get(handle)?.clone() else {
            return Err(Error::InvalidHandle(handle));
        };
        if endpoint.read_access {
            return Err(Error::InvalidHandle(handle));
        }
        let Some(queue) = self.message_queues.get(&endpoint.queue_id) else {
            return Err(Error::InvalidHandle(handle));
        };
        if bytes_len as usize > queue.max_message_bytes as usize {
            return Ok(false);
        }
        if queue.readers == 0 && queue.flags & Self::MSGQUEUE_ALLOW_BROKEN == 0 {
            return Ok(false);
        }
        if flags & Self::MSGQUEUE_MSGALERT != 0 && queue.alert.is_none() {
            return Ok(false);
        }
        Ok(queue.messages.len() >= queue.max_messages as usize)
    }

    fn write_message_queue_by_id(
        &mut self,
        queue_id: u32,
        bytes: Vec<u8>,
        flags: u32,
    ) -> Result<MessageQueueWriteStatus> {
        let Some(queue) = self.message_queues.get_mut(&queue_id) else {
            return Err(Error::InvalidHandle(queue_id));
        };
        if bytes.len() > queue.max_message_bytes as usize {
            return Ok(MessageQueueWriteStatus::MessageTooLarge);
        }
        if queue.readers == 0 && queue.flags & Self::MSGQUEUE_ALLOW_BROKEN == 0 {
            return Ok(MessageQueueWriteStatus::Broken);
        }
        if flags & Self::MSGQUEUE_MSGALERT != 0 && queue.alert.is_none() {
            queue.alert = Some(MessageQueueMessage { bytes, flags });
            self.queue_message_queue_wake_candidates(queue_id);
            return Ok(MessageQueueWriteStatus::Written);
        }
        if queue.messages.len() >= queue.max_messages as usize {
            return Ok(MessageQueueWriteStatus::Full);
        }
        queue
            .messages
            .push_back(MessageQueueMessage { bytes, flags });
        queue.max_queue_messages = queue.max_queue_messages.max(queue.messages.len() as u32);
        self.queue_message_queue_wake_candidates(queue_id);
        Ok(MessageQueueWriteStatus::Written)
    }

    fn queue_message_queue_wake_candidates(&mut self, queue_id: u32) {
        let handles: Vec<_> = self
            .handles
            .iter()
            .filter_map(|(handle, object)| match object {
                KernelObject::MessageQueue(endpoint) if endpoint.queue_id == queue_id => {
                    Some(handle)
                }
                _ => None,
            })
            .collect();
        for handle in handles {
            self.queue_object_wake_candidates(handle);
        }
    }

    fn message_queue_id(&self, handle: u32) -> Result<u32> {
        let KernelObject::MessageQueue(endpoint) = self.handles.get(handle)? else {
            return Err(Error::InvalidHandle(handle));
        };
        Ok(endpoint.queue_id)
    }

    fn message_queue_wait_ready(&self, endpoint: &MessageQueueHandleObject) -> Option<bool> {
        let queue = self.message_queues.get(&endpoint.queue_id)?;
        if endpoint.read_access {
            if queue.writers == 0 && queue.flags & Self::MSGQUEUE_ALLOW_BROKEN == 0 {
                Some(true)
            } else {
                Some(queue.alert.is_some() || !queue.messages.is_empty())
            }
        } else if queue.readers == 0 && queue.flags & Self::MSGQUEUE_ALLOW_BROKEN == 0 {
            Some(true)
        } else {
            Some(queue.messages.len() < queue.max_messages as usize)
        }
    }

    pub fn request_device_notifications(
        &mut self,
        class_guid: [u8; 16],
        message_queue: u32,
        all: bool,
    ) -> Result<u32> {
        let queue_id = self.message_queue_id(message_queue)?;
        let handle =
            self.handles
                .insert(KernelObject::DeviceNotification(DeviceNotificationObject {
                    class_guid,
                    message_queue,
                    all,
                }));
        if all {
            let advertisements: Vec<_> = self
                .device_interface_advertisements
                .iter()
                .filter(|advertisement| advertisement.class_guid == class_guid)
                .cloned()
                .collect();
            for advertisement in advertisements {
                let payload =
                    encode_devdetail_payload(advertisement.class_guid, &advertisement.name, true);
                let _ = self.write_message_queue_by_id(queue_id, payload, 0);
            }
        }
        Ok(handle)
    }

    pub fn stop_device_notifications(&mut self, handle: u32) -> Result<DeviceNotificationObject> {
        let KernelObject::DeviceNotification(notification) = self.handles.get(handle)?.clone()
        else {
            return Err(Error::InvalidHandle(handle));
        };
        self.handles.close(handle)?;
        Ok(notification)
    }

    fn queue_device_interface_notification(
        &mut self,
        advertisement: &DeviceInterfaceAdvertisement,
        attached: bool,
    ) {
        let subscriptions: Vec<_> = self
            .handles
            .iter()
            .filter_map(|(_, object)| match object {
                KernelObject::DeviceNotification(notification)
                    if notification.class_guid == advertisement.class_guid =>
                {
                    Some(notification.message_queue)
                }
                _ => None,
            })
            .collect();
        let payload =
            encode_devdetail_payload(advertisement.class_guid, &advertisement.name, attached);
        for queue_handle in subscriptions {
            if let Ok(queue_id) = self.message_queue_id(queue_handle) {
                let _ = self.write_message_queue_by_id(queue_id, payload.clone(), 0);
            }
        }
    }

    fn publish_configured_mount_device_interfaces(&mut self, add: bool) {
        let specs = self.files.device_interface_advertisement_specs();
        self.publish_mount_device_interface_specs(specs, add);
    }

    fn publish_mount_device_interface_specs(
        &mut self,
        specs: Vec<DeviceInterfaceAdvertisementSpec>,
        add: bool,
    ) {
        for spec in specs {
            let owner = DeviceInterfaceAdvertisementOwner::Mount(spec.owner);
            for interface in spec.interfaces {
                if let Some(class_guid) = parse_guid_string(&interface.class) {
                    self.advertise_device_interface_with_owner(
                        class_guid,
                        interface.name,
                        owner.clone(),
                        add,
                    );
                }
            }
        }
    }

    pub fn display_gamma_value(&self) -> u32 {
        self.display_gamma_value
    }

    pub fn set_display_gamma_value(&mut self, value: u32) {
        self.display_gamma_value = value.clamp(1000, 3000);
    }

    pub fn display_rotation(&self) -> u32 {
        self.display_rotation
    }

    pub fn set_display_rotation(&mut self, value: u32) {
        self.display_rotation = value;
    }

    pub fn display_backlight_enabled(&self) -> bool {
        self.display_lcdcon3_high_nibble & 1 == 0
    }

    pub fn set_display_backlight_enabled(&mut self, enabled: bool) {
        if enabled {
            self.display_lcdcon3_high_nibble &= !1;
        } else {
            self.display_lcdcon3_high_nibble |= 1;
        }
    }

    pub fn display_contrast_value(&self) -> u32 {
        15 - u32::from(self.display_lcdcon3_high_nibble & 0x0f)
    }

    pub fn set_display_contrast_value(&mut self, value: i32) -> u32 {
        let contrast = value.clamp(0, 15) as u8;
        self.display_lcdcon3_high_nibble = 15 - contrast;
        u32::from(contrast)
    }

    pub fn increase_display_contrast(&mut self) -> u32 {
        if self.display_lcdcon3_high_nibble > 0 {
            self.display_lcdcon3_high_nibble -= 1;
        }
        self.display_contrast_value()
    }

    pub fn decrease_display_contrast(&mut self) -> u32 {
        if self.display_lcdcon3_high_nibble < 15 {
            self.display_lcdcon3_high_nibble += 1;
        }
        self.display_contrast_value()
    }

    pub fn reset_display_contrast(&mut self) -> u32 {
        self.display_lcdcon3_high_nibble = 0;
        0
    }

    pub fn clear_display_perf_timings(&mut self) {
        self.display_perf_timings.clear();
        self.display_perf_unhandled = 0;
    }

    pub fn display_perf_unhandled(&self) -> u32 {
        self.display_perf_unhandled
    }

    pub fn display_perf_timing_bytes(&self) -> Vec<u8> {
        const DISPPERF_TIMING_ROWS: usize = 32;
        let mut bytes = Vec::with_capacity(DISPPERF_TIMING_ROWS * 64);
        for index in 0..DISPPERF_TIMING_ROWS {
            if let Some(timing) = self.display_perf_timings.get(index) {
                bytes.extend_from_slice(&timing.rop_code.to_le_bytes());
                bytes.extend_from_slice(&timing.c_gpe.to_le_bytes());
                bytes.extend_from_slice(&timing.dw_gpe_time.to_le_bytes());
                bytes.extend_from_slice(&timing.c_emul.to_le_bytes());
                bytes.extend_from_slice(&timing.dw_emul_time.to_le_bytes());
                bytes.extend_from_slice(&timing.c_hardware.to_le_bytes());
                bytes.extend_from_slice(&timing.dw_hardware_time.to_le_bytes());
                bytes.extend_from_slice(&timing.dw_wait_time.to_le_bytes());
                for param in timing.blt_params {
                    bytes.extend_from_slice(&param.to_le_bytes());
                }
            } else {
                bytes.extend_from_slice(&[0; 64]);
            }
        }
        bytes
    }

    pub fn record_display_perf_gpe(&mut self, rop_code: u32, stretch: bool, transparent: bool) {
        self.record_display_perf_gpe_with_params(
            rop_code,
            DisplayPerfBltParams {
                stretch,
                transparent,
                ..DisplayPerfBltParams::default()
            },
        );
    }

    pub fn record_display_perf_gpe_with_solid_color(
        &mut self,
        rop_code: u32,
        stretch: bool,
        transparent: bool,
        solid_rgb: Option<[u8; 3]>,
    ) {
        self.record_display_perf_gpe_with_params(
            rop_code,
            DisplayPerfBltParams {
                solid_rgb,
                stretch,
                transparent,
                ..DisplayPerfBltParams::default()
            },
        );
    }

    pub fn record_display_perf_gpe_with_params(
        &mut self,
        rop_code: u32,
        params: DisplayPerfBltParams,
    ) {
        const DISPPERF_TIMING_ROWS: usize = 32;
        const PARAM_SRCINVIDMEM: usize = 2;
        const PARAM_DESTINVIDMEM: usize = 3;
        const PARAM_COLORBLACK: usize = 4;
        const PARAM_COLORWHITE: usize = 5;
        const PARAM_STRETCH: usize = 6;
        const PARAM_TRANSPARENT: usize = 7;
        let Some(index) = self
            .display_perf_timings
            .iter()
            .position(|timing| timing.rop_code == rop_code)
            .or_else(|| {
                (self.display_perf_timings.len() < DISPPERF_TIMING_ROWS).then(|| {
                    self.display_perf_timings
                        .push(DisplayPerfTiming::new(rop_code));
                    self.display_perf_timings.len() - 1
                })
            })
        else {
            self.display_perf_unhandled = self.display_perf_unhandled.saturating_add(1);
            return;
        };
        let timing = &mut self.display_perf_timings[index];
        timing.c_gpe = timing.c_gpe.saturating_add(1);
        timing.dw_gpe_time = timing.dw_gpe_time.saturating_add(1);
        if params.src_in_video_mem {
            timing.blt_params[PARAM_SRCINVIDMEM] =
                timing.blt_params[PARAM_SRCINVIDMEM].saturating_add(1);
        }
        if params.dest_in_video_mem {
            timing.blt_params[PARAM_DESTINVIDMEM] =
                timing.blt_params[PARAM_DESTINVIDMEM].saturating_add(1);
        }
        if params.stretch {
            timing.blt_params[PARAM_STRETCH] = timing.blt_params[PARAM_STRETCH].saturating_add(1);
        }
        if params.transparent {
            timing.blt_params[PARAM_TRANSPARENT] =
                timing.blt_params[PARAM_TRANSPARENT].saturating_add(1);
        }
        match params.solid_rgb {
            Some([0, 0, 0]) => {
                timing.blt_params[PARAM_COLORBLACK] =
                    timing.blt_params[PARAM_COLORBLACK].saturating_add(1);
            }
            Some([0xff, 0xff, 0xff]) => {
                timing.blt_params[PARAM_COLORWHITE] =
                    timing.blt_params[PARAM_COLORWHITE].saturating_add(1);
            }
            _ => {}
        }
    }

    pub fn should_stop_after_live_pump_timeout_slice(&mut self, slice_ms: u32) -> bool {
        if slice_ms == 0 {
            return true;
        }
        let now = self.timers.tick_count();
        let Some(started) = self.live_pump_timeout_stop_tick else {
            self.live_pump_timeout_stop_tick = Some(now);
            return false;
        };
        if now.wrapping_sub(started) < slice_ms {
            return false;
        }
        self.live_pump_timeout_stop_tick = Some(now);
        true
    }

    pub fn record_runtime_loader_load_attempt(&mut self) {
        self.runtime_loader_stats.load_attempt_count += 1;
    }

    pub fn record_runtime_loader_successful_map(&mut self) {
        self.runtime_loader_stats.successful_map_count += 1;
    }

    pub fn record_runtime_loader_dependency_load(&mut self) {
        self.runtime_loader_stats.dependency_load_count += 1;
    }

    pub fn record_runtime_loader_export_lookup(&mut self, found: bool) {
        self.runtime_loader_stats.export_lookup_count += 1;
        if !found {
            self.runtime_loader_stats.export_lookup_miss_count += 1;
        }
    }

    pub fn record_runtime_loader_forwarded_export(&mut self) {
        self.runtime_loader_stats.forwarded_export_count += 1;
    }

    pub fn record_runtime_loader_tls_callback(&mut self) {
        self.runtime_loader_stats.tls_callback_count += 1;
    }

    pub fn record_runtime_loader_dllmain_attach(&mut self) {
        self.runtime_loader_stats.dllmain_attach_count += 1;
    }

    pub fn record_runtime_loader_dllmain_detach(&mut self) {
        self.runtime_loader_stats.dllmain_detach_count += 1;
    }

    pub fn record_runtime_loader_loud_failure(&mut self) {
        self.runtime_loader_stats.loud_failure_count += 1;
    }

    pub fn crt_srand(&mut self, seed: u32) {
        self.crt_rand_state = seed;
    }

    pub fn crt_rand(&mut self) -> u32 {
        self.crt_rand_state = self
            .crt_rand_state
            .wrapping_mul(214013)
            .wrapping_add(2531011);
        (self.crt_rand_state >> 16) & 0x7fff
    }

    pub fn crt_strtok_next(&self, thread_id: u32) -> u32 {
        self.crt_strtok_next_by_thread
            .get(&thread_id)
            .copied()
            .unwrap_or(0)
    }

    pub fn crt_set_strtok_next(&mut self, thread_id: u32, ptr: u32) {
        if ptr == 0 {
            self.crt_strtok_next_by_thread.remove(&thread_id);
        } else {
            self.crt_strtok_next_by_thread.insert(thread_id, ptr);
        }
    }

    pub fn crt_wcstok_next(&self, thread_id: u32) -> u32 {
        self.crt_wcstok_next_by_thread
            .get(&thread_id)
            .copied()
            .unwrap_or(0)
    }

    pub fn crt_set_wcstok_next(&mut self, thread_id: u32, ptr: u32) {
        if ptr == 0 {
            self.crt_wcstok_next_by_thread.remove(&thread_id);
        } else {
            self.crt_wcstok_next_by_thread.insert(thread_id, ptr);
        }
    }

    pub fn set_process_module_base(&mut self, base: u32) {
        self.invalidate_selected_process_context();
        self.process_module_base = base;
    }

    pub fn process_module_base(&self) -> u32 {
        self.process_module_base
    }

    pub fn register_loaded_module(
        &mut self,
        name: impl Into<String>,
        base: u32,
        exports_by_name: BTreeMap<String, u32>,
        exports_by_ordinal: BTreeMap<u32, u32>,
    ) {
        self.register_loaded_module_with_metadata(
            name,
            base,
            exports_by_name,
            exports_by_ordinal,
            LoadedModuleMetadata::default(),
        );
    }

    pub fn register_loaded_module_with_metadata(
        &mut self,
        name: impl Into<String>,
        base: u32,
        exports_by_name: BTreeMap<String, u32>,
        exports_by_ordinal: BTreeMap<u32, u32>,
        metadata: LoadedModuleMetadata,
    ) {
        let name = name.into();
        let exports_by_name = exports_by_name
            .into_iter()
            .map(|(name, address)| (normalize_symbol_name(&name), address))
            .collect();
        let forwarders_by_name = metadata
            .forwarders_by_name
            .into_iter()
            .map(|(name, forwarder)| (normalize_symbol_name(&name), forwarder))
            .collect();
        let load_order = self.next_loaded_module_order;
        self.next_loaded_module_order = self.next_loaded_module_order.saturating_add(1);
        self.loaded_modules.insert(
            normalize_module_name(&name),
            LoadedModule {
                name,
                base,
                guest_path: metadata.guest_path,
                host_path: metadata.host_path,
                image_size: metadata.image_size,
                entry_point: metadata.entry_point,
                exports_by_name,
                exports_by_ordinal,
                forwarders_by_name,
                forwarders_by_ordinal: metadata.forwarders_by_ordinal,
                dependencies: metadata.dependencies,
                tls_callbacks: metadata.tls_callbacks,
                ref_count: metadata.ref_count.max(1),
                load_flags: metadata.load_flags,
                dynamic: metadata.dynamic,
                unload_pending: false,
                thread_library_calls_disabled: metadata.thread_library_calls_disabled,
                load_order,
            },
        );
    }

    pub fn loaded_module_handle(&self, name: &str) -> Option<u32> {
        self.loaded_modules
            .get(&normalize_module_name(name))
            .filter(|module| !module.unload_pending)
            .map(|module| module.base)
    }

    pub fn is_loaded_module_handle(&self, module: u32) -> bool {
        self.loaded_modules
            .values()
            .any(|loaded| loaded.base == module && !loaded.unload_pending)
    }

    pub fn disable_thread_library_calls_for_module(&mut self, module: u32) -> bool {
        let Some(loaded) = self
            .loaded_modules
            .values_mut()
            .find(|loaded| loaded.base == module && !loaded.unload_pending)
        else {
            return false;
        };
        loaded.thread_library_calls_disabled = true;
        true
    }

    pub fn retain_loaded_module_by_name(&mut self, name: &str) -> Option<u32> {
        let module = self.loaded_modules.get_mut(&normalize_module_name(name))?;
        if module.unload_pending {
            return None;
        }
        module.ref_count = module.ref_count.saturating_add(1);
        Some(module.base)
    }

    pub fn retain_loaded_module_by_name_for_load(
        &mut self,
        name: &str,
        requested_flags: u32,
    ) -> Option<u32> {
        let module = self.loaded_modules.get_mut(&normalize_module_name(name))?;
        if module.unload_pending {
            return None;
        }
        module.ref_count = module.ref_count.saturating_add(1);
        if (module.load_flags & NO_IMPORT_LOAD_FLAGS) == DONT_RESOLVE_DLL_REFERENCES_FLAG
            && (requested_flags & NO_IMPORT_LOAD_FLAGS) == 0
        {
            module.load_flags &= !DONT_RESOLVE_DLL_REFERENCES_FLAG;
        }
        module.load_flags |= requested_flags & NO_IMPORT_LOAD_FLAGS;
        module.load_flags |= requested_flags & !NO_IMPORT_LOAD_FLAGS;
        Some(module.base)
    }

    pub fn release_loaded_module(&mut self, module: u32) -> FreeLibraryResult {
        let Some((_name, loaded)) = self
            .loaded_modules
            .iter_mut()
            .find(|(_name, loaded)| loaded.base == module && !loaded.unload_pending)
        else {
            return FreeLibraryResult::InvalidHandle;
        };
        if !loaded.dynamic {
            return FreeLibraryResult::Pinned;
        }
        if loaded.ref_count > 1 {
            loaded.ref_count -= 1;
            return FreeLibraryResult::StillReferenced {
                ref_count: loaded.ref_count,
            };
        }
        loaded.ref_count = 0;
        loaded.unload_pending = true;
        FreeLibraryResult::UnloadPending
    }

    pub fn loaded_module_snapshots(&self) -> Vec<LoadedModuleSnapshot> {
        self.loaded_modules
            .values()
            .map(|module| LoadedModuleSnapshot {
                name: module.name.clone(),
                base: module.base,
                ref_count: module.ref_count,
                dynamic: module.dynamic,
                unload_pending: module.unload_pending,
                guest_path: module.guest_path.clone(),
                host_path: module.host_path.clone(),
            })
            .collect()
    }

    pub fn loaded_module_export_snapshots(&self) -> Vec<LoadedModuleExportSnapshot> {
        self.loaded_modules
            .values()
            .filter(|module| !module.unload_pending)
            .map(|module| LoadedModuleExportSnapshot {
                name: module.name.clone(),
                base: module.base,
                exports_by_name: module.exports_by_name.clone(),
                exports_by_ordinal: module.exports_by_ordinal.clone(),
                forwarders_by_name: module.forwarders_by_name.clone(),
                forwarders_by_ordinal: module.forwarders_by_ordinal.clone(),
            })
            .collect()
    }

    pub fn loaded_module_by_handle(&self, module: u32) -> Option<LoadedModule> {
        self.loaded_modules
            .values()
            .find(|loaded| loaded.base == module && !loaded.unload_pending)
            .cloned()
    }

    pub fn loaded_modules_for_thread_notifications(&self) -> Vec<LoadedModule> {
        let mut modules: Vec<LoadedModule> = self
            .loaded_modules
            .values()
            .filter(|loaded| {
                !loaded.unload_pending
                    && !loaded.thread_library_calls_disabled
                    && (loaded.load_flags & NO_IMPORT_LOAD_FLAGS) == 0
            })
            .cloned()
            .collect();
        modules.sort_by_key(|module| module.load_order);
        modules
    }

    pub fn loaded_modules_for_process_detach(&self) -> Vec<LoadedModule> {
        let mut modules: Vec<LoadedModule> = self
            .loaded_modules
            .values()
            .filter(|loaded| {
                !loaded.unload_pending
                    && loaded.ref_count != 0
                    && (loaded.load_flags & NO_IMPORT_LOAD_FLAGS) == 0
            })
            .cloned()
            .collect();
        modules.sort_by_key(|module| module.load_order);
        modules
    }

    pub fn release_process_detach_loaded_modules(&mut self) -> usize {
        let modules = self.loaded_modules_for_process_detach();
        let mut released = 0usize;
        for module in modules {
            for _ in 0..module.ref_count {
                match self.release_loaded_module(module.base) {
                    FreeLibraryResult::StillReferenced { .. }
                    | FreeLibraryResult::UnloadPending => {
                        released += 1;
                    }
                    FreeLibraryResult::Pinned | FreeLibraryResult::InvalidHandle => {}
                }
            }
        }
        released
    }

    pub fn font_families(&self) -> &[CeFontFamily] {
        &self.font_families
    }

    pub fn add_font_mem_resource_handle(&mut self) -> u32 {
        let handle = self.next_font_mem_resource_handle;
        self.next_font_mem_resource_handle = self.next_font_mem_resource_handle.wrapping_add(1);
        self.live_font_mem_resources.insert(handle);
        handle
    }

    pub fn remove_font_mem_resource_handle(&mut self, handle: u32) -> bool {
        self.live_font_mem_resources.remove(&handle)
    }

    pub fn add_font_resource_ref(&mut self, path: &str) {
        let key = font_resource_key(path);
        let count = self.font_resource_counts.entry(key).or_insert(0);
        *count = count.saturating_add(1);
    }

    pub fn remove_font_resource_ref(&mut self, path: &str) -> bool {
        let key = font_resource_key(path);
        let Some(count) = self.font_resource_counts.get_mut(&key) else {
            return false;
        };
        if *count == 0 {
            self.font_resource_counts.remove(&key);
            return false;
        }
        *count -= 1;
        if *count == 0 {
            self.font_resource_counts.remove(&key);
        }
        true
    }

    pub fn loaded_module_for_address(&self, addr: u32) -> Option<&LoadedModule> {
        self.loaded_modules.values().find(|m| {
            !m.unload_pending && addr >= m.base && addr < m.base.saturating_add(m.image_size)
        })
    }

    pub fn resolve_loaded_module_proc_by_name(&self, module: u32, name: &str) -> Option<u32> {
        let symbol = normalize_symbol_name(name);
        let loaded = self
            .loaded_modules
            .values()
            .find(|loaded| loaded.base == module && !loaded.unload_pending)?;
        if loaded.load_flags & LOAD_LIBRARY_AS_DATAFILE_FLAG != 0 {
            return None;
        }
        loaded.exports_by_name.get(&symbol).copied()
    }

    pub fn resolve_loaded_module_proc_by_ordinal(&self, module: u32, ordinal: u32) -> Option<u32> {
        let loaded = self
            .loaded_modules
            .values()
            .find(|loaded| loaded.base == module && !loaded.unload_pending)?;
        if loaded.load_flags & LOAD_LIBRARY_AS_DATAFILE_FLAG != 0 {
            return None;
        }
        loaded.exports_by_ordinal.get(&ordinal).copied()
    }

    pub fn set_process_module_path(&mut self, path: impl Into<String>) {
        self.invalidate_selected_process_context();
        let path = path.into();
        self.files.set_root_relative_guest_path(&path);
        self.process_module_path = path;
    }

    pub fn process_module_path(&self) -> &str {
        &self.process_module_path
    }

    pub fn process_name_guest_ptr_for(&self, owner: ProcessBirthKey) -> Option<u32> {
        self.process_name_guest_ptrs
            .get(&owner)
            .and_then(|(text, ptr)| (text == &self.process_module_path).then_some(*ptr))
    }

    pub fn set_process_name_guest_ptr_for(&mut self, owner: ProcessBirthKey, ptr: u32) {
        self.process_name_guest_ptrs
            .insert(owner, (self.process_module_path.clone(), ptr));
    }

    pub fn module_heap_info_guest_ptr_for(&self, owner: ProcessBirthKey) -> Option<u32> {
        self.module_heap_info_guest_ptrs.get(&owner).copied()
    }

    pub fn set_module_heap_info_guest_ptr_for(&mut self, owner: ProcessBirthKey, ptr: u32) {
        self.module_heap_info_guest_ptrs.insert(owner, ptr);
    }

    pub fn set_process_module_host_path(&mut self, path: impl Into<PathBuf>) {
        self.invalidate_selected_process_context();
        self.process_module_host_path = Some(path.into());
    }

    pub fn process_module_host_path(&self) -> Option<&PathBuf> {
        self.process_module_host_path.as_ref()
    }

    pub fn set_process_command_line(&mut self, command_line: impl Into<String>) {
        self.invalidate_selected_process_context();
        self.process_command_line = command_line.into();
    }

    pub fn process_command_line(&self) -> &str {
        &self.process_command_line
    }

    /// Return (or lazily allocate) the guest-side command-line wide-string pointer.
    /// Allocates a LMEM block just large enough to hold the null-terminated UTF-16
    /// command line on first call.  Returns 0 only when the memory system is full.
    pub fn command_line_guest_ptr_for(&self, owner: ProcessBirthKey) -> Option<u32> {
        self.command_line_guest_ptrs
            .get(&owner)
            .and_then(|(text, ptr)| (text == &self.process_command_line).then_some(*ptr))
    }

    pub fn set_command_line_guest_ptr_for(&mut self, owner: ProcessBirthKey, ptr: u32) {
        self.command_line_guest_ptrs
            .insert(owner, (self.process_command_line.clone(), ptr));
    }

    pub fn set_process_current_directory(&mut self, directory: Option<String>) {
        self.invalidate_selected_process_context();
        self.process_current_directory = directory;
    }

    pub fn process_current_directory(&self) -> Option<&str> {
        self.process_current_directory.as_deref()
    }

    pub fn set_process_show_cmd(&mut self, show_cmd: u32) {
        self.invalidate_selected_process_context();
        self.process_show_cmd = show_cmd;
    }

    pub fn process_show_cmd(&self) -> u32 {
        self.process_show_cmd
    }

    pub fn set_current_process_id(&mut self, process_id: u32) {
        self.invalidate_selected_process_context();
        self.current_process_id = process_id;
    }

    pub fn current_process_id(&self) -> u32 {
        self.current_process_id
    }

    /// Resolve the mutable selector through the canonical process-birth table.
    /// VM callers must not infer ownership from a bare PID or fall back to PID 1.
    pub fn current_process_birth_key(&self) -> Option<ProcessBirthKey> {
        self.handles.process_birth_key(self.current_process_id)
    }

    /// Capture only a canonical birth, never an unregistered numeric PID.
    pub(crate) fn selected_process_context_stamp(&self) -> Option<SelectedProcessContextStamp> {
        Some(SelectedProcessContextStamp {
            birth: self.current_process_birth_key()?,
            generation: self.selected_process_context_generation?,
        })
    }

    /// Re-resolve the birth on every check: HandleTable is currently public,
    /// so a PID may acquire a new birth without calling a selector setter.
    pub(crate) fn selected_process_context_matches(
        &self,
        stamp: SelectedProcessContextStamp,
    ) -> bool {
        self.selected_process_context_generation == Some(stamp.generation)
            && self.current_process_birth_key() == Some(stamp.birth)
    }

    fn invalidate_selected_process_context(&mut self) {
        // Existing setters are infallible. Preserve their behavior on overflow,
        // but permanently poison provenance rather than wrap or saturate it.
        self.selected_process_context_generation = self
            .selected_process_context_generation
            .and_then(|generation| generation.checked_add(1));
    }

    pub fn stdio_path(&self, id: u32) -> Option<&str> {
        if id >= 3 {
            return None;
        }
        self.process_stdio_paths
            .get(&(self.current_process_id, id))
            .map(String::as_str)
    }

    pub fn set_stdio_path(&mut self, id: u32, path: Option<String>) -> bool {
        if id >= 3 {
            return false;
        }
        let key = (self.current_process_id, id);
        if let Some(path) = path {
            self.process_stdio_paths.insert(key, path);
        } else {
            self.process_stdio_paths.remove(&key);
        }
        true
    }

    pub fn is_primary_thread_for_current_process(&self, thread_id: u32) -> bool {
        let Some(owner) = self.current_process_birth_key() else {
            return false;
        };
        if !self.handles.process_execution_live_by_birth(owner) {
            return false;
        }
        let Some(primary) = self.handles.process_primary_thread_birth(owner) else {
            return false;
        };
        primary.thread_id() == thread_id
            && self.handles.thread_birth_key_by_id(thread_id) == Some(primary)
            && self.handles.live_thread_creation_owner_by_id(thread_id) == Some(owner)
    }

    pub fn toolhelp_process_snapshots(&self) -> Vec<ToolhelpProcessSnapshot> {
        let mut snapshots = vec![ToolhelpProcessSnapshot {
            process_id: self.current_process_id,
            default_heap: crate::ce::memory::PROCESS_HEAP_HANDLE,
            thread_count: 1,
            priority_base: crate::ce::object::THREAD_PRIORITY_NORMAL as i32,
            flags: 0,
            memory_base: self.process_module_base,
            name: toolhelp_process_basename(&self.process_module_path),
        }];
        let mut process_ids = vec![self.current_process_id];
        for process in self.handles.process_snapshots() {
            if process_ids.contains(&process.process_id) {
                continue;
            }
            let name = self
                .pending_process_launches
                .iter()
                .find(|launch| launch.process_id == process.process_id)
                .and_then(|launch| launch.application.as_deref())
                .map(toolhelp_process_basename)
                .unwrap_or_else(|| format!("process-{}.exe", process.process_id));
            process_ids.push(process.process_id);
            snapshots.push(ToolhelpProcessSnapshot {
                process_id: process.process_id,
                default_heap: crate::ce::memory::PROCESS_HEAP_HANDLE,
                thread_count: 1,
                priority_base: crate::ce::object::THREAD_PRIORITY_NORMAL as i32,
                flags: 0,
                memory_base: 0,
                name,
            });
        }
        snapshots
    }

    pub fn toolhelp_thread_snapshots(&self, current_thread_id: u32) -> Vec<ToolhelpThreadSnapshot> {
        let current_owner = self
            .handles
            .thread_creation_owner_by_id(current_thread_id)
            .map(ProcessBirthKey::process_id)
            // Older host-only Toolhelp fixtures supply a synthetic caller TID
            // without a canonical birth. Preserve that fixture boundary; a
            // registered thread never falls back to the mutable selector.
            .or_else(|| {
                self.handles
                    .thread_birth_key_by_id(current_thread_id)
                    .is_none()
                    .then_some(self.current_process_id)
            })
            .unwrap_or(0);
        let mut snapshots = vec![ToolhelpThreadSnapshot {
            thread_id: current_thread_id,
            owner_process_id: current_owner,
            base_priority: self.thread_priority_by_id(current_thread_id),
            current_priority: self.thread_priority_by_id(current_thread_id),
            flags: 0,
            current_process_id: current_owner,
        }];
        let mut thread_ids = vec![current_thread_id];
        for thread in self.handles.thread_snapshots() {
            if thread_ids.contains(&thread.thread_id) {
                continue;
            }
            thread_ids.push(thread.thread_id);
            snapshots.push(self.toolhelp_thread_snapshot_from_object(&thread));
        }
        snapshots
    }

    fn toolhelp_thread_snapshot_from_object(
        &self,
        thread: &ThreadObject,
    ) -> ToolhelpThreadSnapshot {
        let owner_process_id = self
            .handles
            .thread_creation_owner_by_id(thread.thread_id)
            .map(ProcessBirthKey::process_id)
            .unwrap_or(0);
        let flags = if thread.suspend_count != 0 {
            6
        } else if thread.signaled {
            4
        } else {
            0
        };
        ToolhelpThreadSnapshot {
            thread_id: thread.thread_id,
            owner_process_id,
            base_priority: thread.priority,
            current_priority: self.thread_priority_by_id(thread.thread_id),
            flags,
            current_process_id: owner_process_id,
        }
    }

    pub fn toolhelp_module_snapshots(&self, process_id: u32) -> Vec<ToolhelpModuleSnapshot> {
        self.loaded_modules
            .values()
            .filter(|module| !module.unload_pending)
            .map(|module| {
                let path = module
                    .guest_path
                    .clone()
                    .unwrap_or_else(|| module.name.clone());
                ToolhelpModuleSnapshot {
                    module_id: module.base,
                    process_id,
                    ref_count: module.ref_count,
                    base: module.base,
                    size: module.image_size,
                    flags: 0,
                    name: toolhelp_process_basename(&module.name),
                    path,
                }
            })
            .collect()
    }

    pub fn current_process_state(&self) -> CurrentProcessState {
        CurrentProcessState {
            process_id: self.current_process_id,
            exit_code: self.current_process_exit_code,
            signaled: self.current_process_signaled,
        }
    }

    pub fn set_current_process_state(&mut self, state: CurrentProcessState) {
        self.invalidate_selected_process_context();
        self.current_process_id = state.process_id;
        self.current_process_exit_code = state.exit_code;
        self.current_process_signaled = state.signaled;
    }

    pub fn reset_current_process_exit_state(&mut self) {
        self.invalidate_selected_process_context();
        self.current_process_exit_code = STILL_ACTIVE;
        self.current_process_signaled = false;
    }

    pub fn is_current_thread_pseudo_handle(handle: u32) -> bool {
        handle == CE_CURRENT_THREAD_PSEUDO_HANDLE
    }

    pub fn is_current_process_pseudo_handle(handle: u32) -> bool {
        handle == CE_CURRENT_PROCESS_PSEUDO_HANDLE
    }

    pub fn queue_process_launch(
        &mut self,
        application: Option<String>,
        command_line: Option<String>,
    ) -> PendingProcessLaunch {
        self.queue_process_launch_with_options(application, command_line, None, None)
    }

    pub fn queue_process_launch_with_show(
        &mut self,
        application: Option<String>,
        command_line: Option<String>,
        show_cmd: Option<u32>,
    ) -> PendingProcessLaunch {
        self.queue_process_launch_with_options(application, command_line, None, show_cmd)
    }

    pub fn queue_process_launch_with_options(
        &mut self,
        application: Option<String>,
        command_line: Option<String>,
        current_directory: Option<String>,
        show_cmd: Option<u32>,
    ) -> PendingProcessLaunch {
        let process_id = self.next_process_id;
        self.next_process_id = self
            .next_process_id
            .checked_add(1)
            .expect("process PID exhausted before launch publication");
        let thread_id = self.threads.allocate_guest_thread_id();
        let process_handle = self.handles.create_process(process_id);
        let child_birth = self
            .handles
            .process_birth_key_for_handle(process_handle)
            .expect("new child process birth");
        let thread_handle = self
            .handles
            .try_create_thread_in_birth(child_birth, thread_id, 0, 0, false)
            .expect("child primary thread birth allocation failed");
        let primary_thread_birth = self
            .handles
            .thread_birth_key_for_handle(thread_handle)
            .expect("new child primary thread birth");
        assert!(
            self.handles
                .bind_primary_thread_to_process(child_birth, primary_thread_birth)
        );
        let launch = PendingProcessLaunch {
            application,
            command_line,
            current_directory,
            show_cmd,
            process_handle,
            thread_handle,
            process_id,
            thread_id,
            process_birth: Some(child_birth),
            primary_thread_birth: Some(primary_thread_birth),
        };
        self.pending_process_launches.push(launch.clone());
        self.record_process_trace(ProcessTraceRecord {
            op: "CreateProcessWQueued",
            application: launch.application.clone(),
            command_line: launch.command_line.clone(),
            path: None,
            process_handle: Some(process_handle),
            thread_handle: Some(thread_handle),
            process_id: Some(process_id),
            thread_id: Some(thread_id),
            result: Some(1),
            error: None,
            detail: Some(format!("show_cmd={show_cmd:?}")),
        });
        launch
    }

    pub fn take_pending_process_launches(&mut self) -> Vec<PendingProcessLaunch> {
        std::mem::take(&mut self.pending_process_launches)
    }

    /// A published child launch has an independent exact Task birth even
    /// before a CPU exists. Cloning it requires whole-machine relineage.
    pub fn has_pending_process_launches(&self) -> bool {
        !self.pending_process_launches.is_empty()
    }

    /// Return unprocessed, already-published launches to the front after a
    /// recoverable stopped-exit refusal. Their original birth witnesses stay
    /// intact and later launches retain their relative order.
    pub fn restore_pending_process_launches(&mut self, mut launches: Vec<PendingProcessLaunch>) {
        launches.append(&mut self.pending_process_launches);
        self.pending_process_launches = launches;
    }

    pub fn mark_process_launch_exited(
        &mut self,
        launch: &PendingProcessLaunch,
        exit_code: u32,
    ) -> bool {
        self.mark_process_launch_exited_with_framebuffer(launch, exit_code, None)
    }

    pub fn mark_process_launch_exited_with_framebuffer(
        &mut self,
        launch: &PendingProcessLaunch,
        exit_code: u32,
        framebuffer: Option<&mut dyn Framebuffer>,
    ) -> bool {
        self.try_mark_process_launch_exited_with_framebuffer(launch, exit_code, framebuffer)
            .unwrap_or(false)
    }

    /// Complete one stopped, exact-birth process only after every saved
    /// window image can be restored to the current framebuffer. A recovery
    /// error leaves the process and all its exit effects untouched.
    pub fn try_mark_process_launch_exited_with_framebuffer(
        &mut self,
        launch: &PendingProcessLaunch,
        exit_code: u32,
        framebuffer: Option<&mut dyn Framebuffer>,
    ) -> Result<bool> {
        self.try_mark_process_launch_exited_with_witness(launch, exit_code, framebuffer)
            .map(|accepted| accepted.is_some())
    }

    /// Return a one-use retirement witness only after this exact stopped
    /// birth's exit effects and canonical completion have succeeded.
    pub fn try_mark_process_launch_exited_with_witness(
        &mut self,
        launch: &PendingProcessLaunch,
        exit_code: u32,
        framebuffer: Option<&mut dyn Framebuffer>,
    ) -> Result<Option<AcceptedProcessExit>> {
        // An original launch retains its exact birth even after both public
        // aliases close. Reject a delayed completion after PID/TID reuse
        // before it can touch windows, VM, or another process's workers.
        // A reconstructed launch without this witness cannot authorize any
        // cleanup, even its old primary-only path: its PID may have been reused.
        let Some(launch_birth) = launch.process_birth else {
            return Ok(None);
        };
        let Some(exiting_birth) = self.handles.process_birth_key(launch.process_id) else {
            return Ok(None);
        };
        if launch_birth != exiting_birth
            || !self.handles.process_execution_live_by_birth(exiting_birth)
        {
            return Ok(None);
        }
        // The process object retains its exact primary birth after that
        // thread retires. A live worker reusing its numeric TID cannot take
        // the process exit code, and a supplied birth is only a consistency
        // witness against this canonical owner.
        let Some(primary) = self.handles.process_primary_thread_birth(exiting_birth) else {
            return Ok(None);
        };
        if launch.thread_id != primary.thread_id()
            || launch
                .primary_thread_birth
                .is_some_and(|supplied| supplied != primary)
        {
            return Ok(None);
        }
        let exit_code = self
            .handles
            .process_termination_requested_exit_code_by_birth(exiting_birth)
            .unwrap_or(exit_code);
        let cleanup_thread_id = if self.handles.thread_birth_key_by_id(primary.thread_id())
            == Some(primary)
            && self
                .handles
                .thread_creation_owner_by_id(primary.thread_id())
                == Some(exiting_birth)
        {
            primary.thread_id()
        } else {
            0
        };
        self.try_destroy_process_windows_with_backing_restore_for_birth(
            exiting_birth,
            cleanup_thread_id,
            framebuffer,
        )?;
        self.retire_process_host_metadata_for_birth(exiting_birth);
        let completed = self.complete_process_execution_by_birth(exiting_birth, exit_code);
        debug_assert!(completed, "preflighted process birth must complete");
        self.record_process_trace(ProcessTraceRecord {
            op: "CreateProcessExited",
            application: launch.application.clone(),
            command_line: launch.command_line.clone(),
            path: None,
            process_handle: Some(launch.process_handle),
            thread_handle: Some(launch.thread_handle),
            process_id: Some(launch.process_id),
            thread_id: Some(launch.thread_id),
            result: Some(exit_code),
            error: None,
            detail: Some(format!("show_cmd={:?}", launch.show_cmd)),
        });
        Ok(completed.then_some(AcceptedProcessExit {
            birth: exiting_birth,
        }))
    }

    /// One stopped process owns this exact birth before host completion may
    /// retire its windows. The framebuffer check and restore are shared with
    /// child completion; a missing or incompatible destination leaves every
    /// exit effect untouched so the stopped CPU can be recovered.
    pub fn try_terminate_stopped_current_process_for_birth_with_framebuffer(
        &mut self,
        expected_birth: ProcessBirthKey,
        exit_code: u32,
        framebuffer: Option<&mut dyn Framebuffer>,
    ) -> Result<bool> {
        if self.current_process_birth_key() != Some(expected_birth)
            || !self.handles.process_execution_live_by_birth(expected_birth)
            || self
                .handles
                .process_signaled_by_id(expected_birth.process_id())
                != Some(false)
        {
            return Ok(false);
        }
        if self
            .handles
            .process_primary_thread_birth(expected_birth)
            .is_none()
        {
            return Ok(false);
        }
        let effective_exit_code = self
            .handles
            .process_termination_requested_exit_code_by_birth(expected_birth)
            .unwrap_or(exit_code);
        self.try_destroy_process_windows_with_backing_restore_for_birth(
            expected_birth,
            0,
            framebuffer,
        )?;
        self.retire_process_host_metadata_for_birth(expected_birth);
        let completed =
            self.complete_process_execution_by_birth(expected_birth, effective_exit_code);
        if !completed {
            return Err(Error::Backend(
                "preflighted stopped process birth failed canonical Task/VM completion".to_owned(),
            ));
        }
        self.invalidate_selected_process_context();
        self.current_process_exit_code = effective_exit_code;
        self.current_process_signaled = true;
        Ok(true)
    }

    fn retire_process_host_metadata_for_birth(&mut self, owner: ProcessBirthKey) {
        self.release_message_pointer_payloads_for_owner(owner);
        self.delivered_typed_payload_messages
            .retain(|(birth, _, _), _| *birth != owner);
        self.module_heap_info_guest_ptrs.remove(&owner);
        self.process_name_guest_ptrs.remove(&owner);
        self.command_line_guest_ptrs.remove(&owner);
    }

    fn try_destroy_process_windows_with_backing_restore_for_birth(
        &mut self,
        exiting_birth: ProcessBirthKey,
        cleanup_thread_id: u32,
        mut framebuffer: Option<&mut dyn Framebuffer>,
    ) -> Result<()> {
        let backing_store_targets =
            self.process_window_backing_store_targets_for_birth(exiting_birth);
        // Keep one exclusive pixel lease across preflight and commit. The
        // destination length cannot change between validation and copying,
        // and no window/send/wait/process mutation precedes this full check.
        let destination_info = framebuffer.as_ref().map(|target| target.info());
        let mut destination_pixels = framebuffer.as_mut().map(|target| target.pixels_mut());
        let mut prepared_restores = Vec::new();
        for hwnd in backing_store_targets.iter().rev().copied() {
            let Some(store) = self.window_backing_stores.get(&hwnd) else {
                continue;
            };
            let Some(info) = destination_info else {
                return Err(Error::HostExitRecoveryRequired {
                    process_id: exiting_birth.process_id(),
                    incarnation: exiting_birth.incarnation(),
                    hwnd: Some(hwnd),
                    reason: "saved window backing has no destination framebuffer",
                });
            };
            let destination_len = destination_pixels.as_ref().map_or(0, |pixels| pixels.len());
            let Some(prepared) = store.prepare_restore(info, destination_len) else {
                return Err(Error::HostExitRecoveryRequired {
                    process_id: exiting_birth.process_id(),
                    incarnation: exiting_birth.incarnation(),
                    hwnd: Some(hwnd),
                    reason: "saved window backing is incompatible with the destination framebuffer",
                });
            };
            prepared_restores.push(prepared);
        }
        self.destroy_process_windows_for_birth(exiting_birth, cleanup_thread_id);
        if let Some(pixels) = destination_pixels.as_deref_mut() {
            for prepared in &prepared_restores {
                assert!(
                    prepared.apply_to(&mut *pixels),
                    "the held framebuffer lease must match its preflight length"
                );
            }
        }
        drop(destination_pixels);
        if let Some(framebuffer) = framebuffer.as_mut() {
            for prepared in &prepared_restores {
                framebuffer.mark_dirty(prepared.rect());
            }
        }
        self.discard_window_backing_stores(&backing_store_targets);
        Ok(())
    }

    /// Complete one stopped process CPU's canonical Task and VM lifetime.
    /// TerminateProcess remains a request; it must not call this while a guest
    /// or native CPU continuation can still execute for the process.
    fn complete_process_execution_by_birth(
        &mut self,
        owner: ProcessBirthKey,
        exit_code: u32,
    ) -> bool {
        if !self.handles.contains_process_birth(owner) {
            return false;
        }
        let Some(primary) = self.handles.process_primary_thread_birth(owner) else {
            return false;
        };
        for thread in self.handles.live_thread_births_for_owner(owner) {
            let thread_exit_code = if thread == primary { exit_code } else { 0 };
            // Canonical thread completion below cancels only this exact
            // birth's registered waits before dispatching its exit wake.
            if !self.mark_guest_thread_exited_by_birth(thread, thread_exit_code) {
                return false;
            }
        }
        let was_signaled = self.handles.process_signaled_by_id(owner.process_id());
        if self
            .handles
            .finish_process_execution_by_birth(owner, exit_code)
            .is_none()
        {
            return false;
        }
        // A native mapping pin keeps its backing alive until checked native
        // owner close; only canonical, exact-birth VM records retire here.
        self.memory.retire_fixed_private_ranges_for_owner(owner);
        self.memory.retire_writable_image_runs_for_owner(owner);
        self.memory.retire_heap_state_for_owner(owner);
        if was_signaled == Some(false) {
            self.queue_object_wake_candidates_for_key(WaitObjectKey::Process(owner));
        }
        true
    }

    /// A host exit with a verified process birth must not claim a window from
    /// an older incarnation of the same public PID.
    fn process_window_targets_for_birth(&self, owner: ProcessBirthKey) -> Vec<u32> {
        self.gwe.process_window_targets_for_birth(owner)
    }

    /// Include an already-destroyed HWND so its captured backing can be
    /// restored, but only if its immutable creation owner is this exit birth.
    fn process_window_backing_store_targets_for_birth(&self, owner: ProcessBirthKey) -> Vec<u32> {
        self.gwe
            .windows_snapshot()
            .into_iter()
            .filter(|window| {
                window.hwnd != crate::ce::gwe::DESKTOP_HWND
                    && window.delivery_owner().process_birth() == Some(owner)
            })
            .map(|window| window.hwnd)
            .collect()
    }

    fn destroy_process_windows_for_birth(&mut self, owner: ProcessBirthKey, thread_id: u32) {
        let hwnds = self.process_window_targets_for_birth(owner);
        self.gwe.terminate_sent_messages_from_process_birth(owner);
        let pending_receiver_sends = self
            .gwe
            .sent_message_ids_for_windows(&hwnds)
            .into_iter()
            .filter(|send_id| !self.gwe.sent_message_result_ready(*send_id))
            .collect::<Vec<_>>();
        for hwnd in &hwnds {
            self.record_window_lifecycle_trace(
                "destroy_window_begin",
                thread_id,
                Some(*hwnd),
                Some(1),
                Some(format!(
                    "reason=process_exit/process_id={}",
                    owner.process_id()
                )),
            );
        }
        let retired = self
            .gwe
            .destroy_process_windows_for_birth(owner, self.timers.tick_count());
        debug_assert_eq!(hwnds, retired, "one birth-selected GWE target set");
        for send_id in pending_receiver_sends {
            if self.gwe.sent_message_result_ready(send_id) {
                self.queue_send_reply_wake_candidates(send_id);
            }
        }
        if !retired.is_empty() {
            self.release_discarded_message_pointer_payloads();
            self.release_message_pointer_payloads_for_windows(&retired);
        }
        for hwnd in &retired {
            self.record_window_lifecycle_trace(
                "destroy_window_end",
                thread_id,
                Some(*hwnd),
                Some(1),
                Some(format!(
                    "reason=process_exit/process_id={}",
                    owner.process_id()
                )),
            );
        }
        if !retired.is_empty() {
            self.shell.remove_windows_state(&retired);
            self.timers.remove_window_timers(&retired);
        }
    }

    pub fn pump_timers_to_gwe(&mut self, thread_id: u32) {
        self.expire_timed_out_send_messages();
        self.expire_shell_notifications();
        for timer in self.timers.due_candidates() {
            if let Some(birth) = timer.owner_birth
                && !self.is_live_thread_birth(birth)
            {
                // A stale due candidate must not publish to a replacement TID
                // or remain stuck in the store with its pending bit set.
                self.timers.cancel_timer_instance(&timer);
                continue;
            }
            if !self.timers.activate_due_timer(&timer) {
                continue;
            }
            let target_thread_id = if timer.thread_id == 0 {
                thread_id
            } else {
                timer.thread_id
            };
            let mut message = crate::ce::gwe::Message::new(
                timer.hwnd.unwrap_or(0),
                timer.message,
                timer.id,
                timer.callback.unwrap_or(0),
                self.timers.tick_count(),
            );
            if let Some(birth) = timer.owner_birth {
                message.timer_origin = Some(crate::ce::timer::TimerMessageOrigin {
                    birth,
                    id: timer.id,
                    instance: timer.instance,
                });
            }
            self.record_message_op(
                "timer_due",
                target_thread_id,
                &message,
                Some(1),
                Some(format!(
                    "timer_thread={}/hwnd={}/id=0x{:08x}/period_ms={}/callback=0x{:08x}/target_thread={}",
                    timer.thread_id,
                    format_optional_hwnd(timer.hwnd),
                    timer.id,
                    timer.period_ms.unwrap_or(0),
                    timer.callback.unwrap_or(0),
                    target_thread_id
                )),
            );
            if let Some(birth) = timer.owner_birth {
                // The due candidate was validated before its pending transition;
                // this exact-birth queue route is infallible and never wakes B.
                self.gwe
                    .post_thread_message_for_birth(birth, message.clone());
                self.record_message_op("post_message", target_thread_id, &message, Some(1), None);
                self.scheduler
                    .queue_message_wake_candidates_for_birth(birth);
            } else {
                self.post_gwe_message(target_thread_id, message);
            }
        }
    }

    pub fn expire_timed_out_send_messages(&mut self) -> Vec<u64> {
        let expired = self
            .gwe
            .expire_timed_out_sent_messages(self.timers.tick_count());
        for send_id in &expired {
            self.queue_send_reply_wake_candidates(*send_id);
        }
        expired
    }

    pub fn timeout_send_message(&mut self, send_id: u64) -> bool {
        if !self.gwe.timeout_sent_message(send_id) {
            return false;
        }
        self.queue_send_reply_wake_candidates(send_id);
        true
    }

    pub fn terminate_sent_messages_to_receiver(&mut self, receiver_thread_id: u32) -> Vec<u64> {
        let terminated = self
            .gwe
            .terminate_sent_messages_to_receiver(receiver_thread_id);
        for send_id in &terminated {
            self.queue_send_reply_wake_candidates(*send_id);
        }
        terminated
    }

    pub fn set_file_root(&mut self, root: impl Into<std::path::PathBuf>) {
        self.files = HostFileSystem::new(root);
    }

    pub fn mount_guest_root(&mut self, guest_root: &str, host_root: impl Into<std::path::PathBuf>) {
        self.files.mount_guest_root(guest_root, host_root);
        self.post_shell_file_change_notifications(SHCNE_DRIVEADD, Some(guest_root), None);
        self.signal_file_change_notifications(SHCNE_DRIVEADD, Some(guest_root), None);
        // Broadcast WM_DEVICECHANGE(DBT_DEVICEARRIVAL) to all top-level windows.
        self.send_notify_message_w(
            0,
            crate::ce::gwe::HWND_BROADCAST,
            WM_DEVICECHANGE,
            DBT_DEVICEARRIVAL,
            0,
        );
    }

    pub fn unmount_guest_root(&mut self, guest_root: &str) -> bool {
        let advertised_interfaces = self
            .files
            .device_interface_advertisement_specs_for_guest_root(guest_root);
        if self.files.unmount_guest_root(guest_root).is_some() {
            self.drop_fsdmgr_volume_locks_for_guest_root(guest_root);
            self.publish_mount_device_interface_specs(advertised_interfaces, false);
            self.post_shell_file_change_notifications(SHCNE_DRIVEREMOVED, Some(guest_root), None);
            self.signal_file_change_notifications(SHCNE_DRIVEREMOVED, Some(guest_root), None);
            // Signal all change notification handles watching any path under the removed volume
            // (CE FSDMGR signals these regardless of their specific notify_filter).
            self.signal_file_change_notifications_for_removed_mount(guest_root);
            // Broadcast WM_DEVICECHANGE(DBT_DEVICEREMOVECOMPLETE) to all top-level windows.
            self.send_notify_message_w(
                0,
                crate::ce::gwe::HWND_BROADCAST,
                WM_DEVICECHANGE,
                DBT_DEVICEREMOVECOMPLETE,
                0,
            );
            true
        } else {
            false
        }
    }

    pub fn create_volume_handle_for_guest_root(&mut self, guest_root: &str) -> Result<u32> {
        let guest_root = normalize_shell_change_path(&canonical_shell_change_path(guest_root));
        match self.files.volume_root_for_guest_path(&guest_root) {
            Some(volume_root) if volume_root.eq_ignore_ascii_case(&guest_root) => {
                Ok(self.handles.insert(KernelObject::Volume(VolumeObject {
                    owner_process_id: self.current_process_id,
                    guest_root,
                    disk_ptr: None,
                    fsd_volume_context: None,
                })))
            }
            _ => Err(Error::InvalidArgument(format!(
                "volume root is not mounted: {guest_root}"
            ))),
        }
    }

    pub fn fsdmgr_register_volume(
        &mut self,
        disk_ptr: u32,
        mount_name: &str,
        fsd_volume_context: u32,
    ) -> Result<u32> {
        if disk_ptr == 0 {
            return Err(Error::InvalidArgument(
                "null FSDMGR disk pointer".to_owned(),
            ));
        }
        if self.fsdmgr_volume_handle_for_disk(disk_ptr).is_some() {
            return Err(Error::AlreadyExists(format!(
                "FSDMGR disk pointer 0x{disk_ptr:08x} is already registered"
            )));
        }
        let existing_guest_root = self
            .files
            .existing_fsdmgr_mount_root(mount_name)
            .map_err(|err| match err {
                Error::InvalidArgument(_) => err,
                other => Error::InvalidArgument(format!(
                    "invalid FSDMGR mount name {mount_name}: {other}"
                )),
            })?
            .filter(|guest_root| self.volume_handle_for_guest_root(guest_root).is_none());
        let (guest_root, created) = if let Some(guest_root) = existing_guest_root {
            (guest_root, false)
        } else {
            let base_guest_root = self
                .files
                .existing_fsdmgr_mount_root(mount_name)
                .ok()
                .flatten();
            let (guest_root, created) =
                self.files
                    .register_fsdmgr_mount_name(mount_name)
                    .map_err(|err| match err {
                        Error::InvalidArgument(_) | Error::OutOfStructures(_) => err,
                        other => Error::InvalidArgument(format!(
                            "invalid FSDMGR mount name {mount_name}: {other}"
                        )),
                    })?;
            if let Some(base_guest_root) = base_guest_root.as_deref() {
                self.files
                    .copy_mount_registry_profile(base_guest_root, &guest_root);
            }
            (guest_root, created)
        };
        let handle = self.handles.insert(KernelObject::Volume(VolumeObject {
            owner_process_id: self.current_process_id,
            guest_root: guest_root.clone(),
            disk_ptr: Some(disk_ptr),
            fsd_volume_context: (fsd_volume_context != 0).then_some(fsd_volume_context),
        }));
        if created {
            self.post_shell_file_change_notifications(SHCNE_DRIVEADD, Some(&guest_root), None);
            self.signal_file_change_notifications(SHCNE_DRIVEADD, Some(&guest_root), None);
            self.send_notify_message_w(
                0,
                crate::ce::gwe::HWND_BROADCAST,
                WM_DEVICECHANGE,
                DBT_DEVICEARRIVAL,
                0,
            );
        }
        Ok(handle)
    }

    pub fn register_afs_name(&mut self, mount_name: &str) -> (u32, u32) {
        if mount_name.trim().is_empty() {
            return (INVALID_MOUNT_INDEX, ERROR_INVALID_PARAMETER);
        }
        if let Some((index, _)) = self
            .afs_mount_slots
            .iter()
            .find(|(_, slot)| slot.name.eq_ignore_ascii_case(mount_name))
        {
            return (*index, ERROR_ALREADY_EXISTS);
        }
        if matches!(
            self.files.existing_fsdmgr_mount_root(mount_name),
            Ok(Some(_))
        ) {
            return (INVALID_MOUNT_INDEX, ERROR_ALREADY_EXISTS);
        }

        let index = match self.allocate_afs_mount_index() {
            Some(index) => index,
            None => return (INVALID_MOUNT_INDEX, ERROR_OUT_OF_STRUCTURES),
        };
        let (guest_root, _) = match self.files.register_fsdmgr_mount_name(mount_name) {
            Ok(result) => result,
            Err(Error::AlreadyExists(_)) => return (INVALID_MOUNT_INDEX, ERROR_ALREADY_EXISTS),
            Err(Error::OutOfStructures(_)) => {
                return (INVALID_MOUNT_INDEX, ERROR_OUT_OF_STRUCTURES);
            }
            Err(Error::InvalidArgument(_)) => {
                return (INVALID_MOUNT_INDEX, ERROR_INVALID_PARAMETER);
            }
            Err(_) => return (INVALID_MOUNT_INDEX, ERROR_INVALID_PARAMETER),
        };
        self.afs_mount_slots.insert(
            index,
            AfsMountSlot {
                name: mount_name.trim().trim_matches(['\\', '/']).to_owned(),
                guest_root,
                owner_process_id: self.current_process_id,
                volume_handle: None,
            },
        );
        (index, ERROR_SUCCESS)
    }

    pub fn register_afs_ex(
        &mut self,
        index: u32,
        api_set_handle: u32,
        volume_context: u32,
        flags: u32,
    ) -> u32 {
        if api_set_handle == 0 || api_set_handle == u32::MAX {
            return ERROR_INVALID_HANDLE;
        }
        let Some(slot) = self.afs_mount_slots.get_mut(&index) else {
            return ERROR_FILE_NOT_FOUND;
        };
        if slot.owner_process_id != self.current_process_id {
            return ERROR_ACCESS_DENIED;
        }
        if slot.volume_handle.is_some() {
            return ERROR_ALREADY_EXISTS;
        }
        let guest_root = slot.guest_root.clone();
        self.files.update_mount_table_flags(&guest_root, flags);
        let handle = self.handles.insert(KernelObject::Volume(VolumeObject {
            owner_process_id: self.current_process_id,
            guest_root: guest_root.clone(),
            disk_ptr: None,
            fsd_volume_context: (volume_context != 0).then_some(volume_context),
        }));
        if let Some(slot) = self.afs_mount_slots.get_mut(&index) {
            slot.volume_handle = Some(handle);
        }
        if flags & 0x0001 == 0 {
            self.post_shell_file_change_notifications(SHCNE_DRIVEADD, Some(&guest_root), None);
            self.signal_file_change_notifications(SHCNE_DRIVEADD, Some(&guest_root), None);
            self.send_notify_message_w(0, HWND_BROADCAST, WM_DEVICECHANGE, DBT_DEVICEARRIVAL, 0);
        }
        ERROR_SUCCESS
    }

    pub fn deregister_afs(&mut self, index: u32) -> u32 {
        let Some(slot) = self.afs_mount_slots.remove(&index) else {
            return ERROR_INVALID_INDEX;
        };
        if slot.owner_process_id != self.current_process_id {
            self.afs_mount_slots.insert(index, slot);
            return ERROR_ACCESS_DENIED;
        }
        if let Some(handle) = slot.volume_handle {
            self.drop_fsdmgr_volume_locks_for_handle(handle);
            let _ = self.handles.close(handle);
        }
        self.files.update_mount_table_flags(&slot.guest_root, 0);
        self.unmount_guest_root(&slot.guest_root);
        ERROR_SUCCESS
    }

    fn allocate_afs_mount_index(&mut self) -> Option<u32> {
        for _ in 0..0x1000 {
            let index = self.next_afs_mount_index.max(FIRST_USER_AFS_INDEX);
            self.next_afs_mount_index = self.next_afs_mount_index.saturating_add(1);
            if !self.afs_mount_slots.contains_key(&index) {
                return Some(index);
            }
        }
        None
    }

    fn volume_handle_for_guest_root(&self, guest_root: &str) -> Option<u32> {
        self.handles
            .iter()
            .find_map(|(handle, object)| match object {
                KernelObject::Volume(volume)
                    if volume.guest_root.eq_ignore_ascii_case(guest_root) =>
                {
                    Some(handle)
                }
                _ => None,
            })
    }

    pub fn fsdmgr_volume_handle_for_disk(&self, disk_ptr: u32) -> Option<u32> {
        if disk_ptr == 0 {
            return None;
        }
        self.handles
            .iter()
            .find_map(|(handle, object)| match object {
                KernelObject::Volume(volume) if volume.disk_ptr == Some(disk_ptr) => Some(handle),
                _ => None,
            })
    }

    pub fn fsdmgr_registry_value(&self, disk_ptr: u32, value_name: &str) -> Option<RegistryValue> {
        let guest_root = self.handles.iter().find_map(|(_, object)| match object {
            KernelObject::Volume(volume) if volume.disk_ptr == Some(disk_ptr) => {
                Some(volume.guest_root.as_str())
            }
            _ => None,
        })?;
        self.files
            .registry_paths_for_guest_root(guest_root)
            .into_iter()
            .find_map(|path| self.registry.query_value(&path, value_name).ok().cloned())
    }

    pub fn fsdmgr_async_enter_volume(&mut self, volume_handle: u32) -> Result<(u32, u32)> {
        self.volume_root_for_handle(volume_handle)?;
        let lock_handle = self.allocate_fsdmgr_volume_lock(volume_handle);
        Ok((lock_handle, volume_handle))
    }

    pub fn fsdmgr_async_exit_volume(&mut self, lock_handle: u32, lock_data: u32) -> Result<()> {
        if lock_handle == 0 || lock_data == 0 {
            return Err(Error::InvalidArgument("null FSDMGR volume lock".to_owned()));
        }
        let Some(volume_lock) = self.fsdmgr_volume_locks.get(&lock_handle).copied() else {
            return Err(Error::InvalidHandle(lock_handle));
        };
        if volume_lock.volume_handle != lock_data {
            return Err(Error::InvalidArgument(
                "mismatched FSDMGR volume lock data".to_owned(),
            ));
        }
        self.volume_root_for_handle(volume_lock.volume_handle)?;
        self.fsdmgr_volume_locks.remove(&lock_handle);
        Ok(())
    }

    fn allocate_fsdmgr_volume_lock(&mut self, volume_handle: u32) -> u32 {
        loop {
            let lock_handle = self.next_fsdmgr_volume_lock;
            self.next_fsdmgr_volume_lock = self
                .next_fsdmgr_volume_lock
                .wrapping_add(1)
                .max(0x6d00_0001);
            if !self.fsdmgr_volume_locks.contains_key(&lock_handle) {
                self.fsdmgr_volume_locks
                    .insert(lock_handle, FsdmgrVolumeLock { volume_handle });
                return lock_handle;
            }
        }
    }

    fn drop_fsdmgr_volume_locks_for_handle(&mut self, volume_handle: u32) {
        self.fsdmgr_volume_locks
            .retain(|_, volume_lock| volume_lock.volume_handle != volume_handle);
        self.afs_file_system_functions.remove(&volume_handle);
        self.afs_mounted_fs_notifications.remove(&volume_handle);
    }

    fn drop_fsdmgr_volume_locks_for_guest_root(&mut self, guest_root: &str) {
        let volume_handles: BTreeSet<u32> = self
            .handles
            .iter()
            .filter_map(|(handle, object)| match object {
                KernelObject::Volume(volume)
                    if volume.guest_root.eq_ignore_ascii_case(guest_root) =>
                {
                    Some(handle)
                }
                _ => None,
            })
            .collect();
        self.fsdmgr_volume_locks
            .retain(|_, volume_lock| !volume_handles.contains(&volume_lock.volume_handle));
        for volume_handle in volume_handles {
            self.afs_file_system_functions.remove(&volume_handle);
            self.afs_mounted_fs_notifications.remove(&volume_handle);
        }
    }

    pub fn register_afs_file_system_function(&mut self, volume_handle: u32, callback: u32) -> u32 {
        match self.volume_root_for_handle(volume_handle) {
            Ok(_) => {
                if callback == 0 {
                    self.afs_file_system_functions.remove(&volume_handle);
                } else {
                    self.afs_file_system_functions
                        .insert(volume_handle, callback);
                }
                ERROR_SUCCESS
            }
            Err(Error::AccessDenied(_)) => ERROR_ACCESS_DENIED,
            Err(_) => ERROR_INVALID_HANDLE,
        }
    }

    pub fn afs_file_system_function(&self, volume_handle: u32) -> Option<u32> {
        self.afs_file_system_functions.get(&volume_handle).copied()
    }

    pub fn notify_afs_mounted_fs(&mut self, volume_handle: u32, flags: u32) -> u32 {
        match self.volume_root_for_handle(volume_handle) {
            Ok(_) => {
                self.afs_mounted_fs_notifications
                    .entry(volume_handle)
                    .or_default()
                    .push(flags);
                ERROR_SUCCESS
            }
            Err(Error::AccessDenied(_)) => ERROR_ACCESS_DENIED,
            Err(_) => ERROR_INVALID_HANDLE,
        }
    }

    pub fn afs_mounted_fs_notifications(&self, volume_handle: u32) -> &[u32] {
        self.afs_mounted_fs_notifications
            .get(&volume_handle)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn close_all_file_handles_for_volume(
        &mut self,
        volume_handle: u32,
        _process_handle: u32,
    ) -> u32 {
        match self.volume_root_for_handle(volume_handle) {
            Ok(_) => ERROR_SUCCESS,
            Err(Error::AccessDenied(_)) => ERROR_ACCESS_DENIED,
            Err(_) => ERROR_INVALID_HANDLE,
        }
    }

    pub fn fsdmgr_create_cache(&mut self, disk_ptr: u32, block_size: u32) -> Option<u32> {
        if disk_ptr == 0 || block_size == 0 {
            return None;
        }
        let entry = FsdmgrCacheEntry {
            disk_ptr,
            block_size,
            disable_delete: false,
            disable_flush: false,
        };
        if let Some((index, slot)) = self
            .fsdmgr_caches
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.is_none())
        {
            *slot = Some(entry);
            return Some(index as u32);
        }
        let cache_id = u32::try_from(self.fsdmgr_caches.len()).ok()?;
        self.fsdmgr_caches.push(Some(entry));
        Some(cache_id)
    }

    pub fn fsdmgr_delete_cache(&mut self, cache_id: u32) -> u32 {
        let Some(slot) = self.fsdmgr_caches.get_mut(cache_id as usize) else {
            return ERROR_INVALID_PARAMETER;
        };
        if slot.is_none() {
            return ERROR_INVALID_PARAMETER;
        }
        *slot = None;
        ERROR_SUCCESS
    }

    fn fsdmgr_cache_entry(&self, cache_id: u32) -> Option<&FsdmgrCacheEntry> {
        self.fsdmgr_caches
            .get(cache_id as usize)
            .and_then(Option::as_ref)
    }

    fn fsdmgr_cache_entry_mut(&mut self, cache_id: u32) -> Option<&mut FsdmgrCacheEntry> {
        self.fsdmgr_caches
            .get_mut(cache_id as usize)
            .and_then(Option::as_mut)
    }

    pub fn fsdmgr_cache_block_size(&self, cache_id: u32) -> Option<u32> {
        self.fsdmgr_cache_entry(cache_id)
            .map(|entry| entry.block_size)
    }

    pub fn fsdmgr_cached_read(
        &self,
        cache_id: u32,
        block_num: u32,
        blocks: u32,
    ) -> Result<Vec<u8>> {
        let Some(entry) = self.fsdmgr_cache_entry(cache_id) else {
            return Err(Error::InvalidArgument(format!(
                "invalid FSDMGR cache id {cache_id}"
            )));
        };
        self.fsdmgr_read_disk(entry.disk_ptr, block_num, blocks, entry.block_size)
    }

    pub fn fsdmgr_cached_write(
        &mut self,
        cache_id: u32,
        block_num: u32,
        blocks: u32,
        bytes: &[u8],
    ) -> u32 {
        let Some(entry) = self.fsdmgr_cache_entry(cache_id).copied() else {
            return ERROR_INVALID_PARAMETER;
        };
        self.fsdmgr_write_disk(entry.disk_ptr, block_num, blocks, entry.block_size, bytes)
    }

    pub fn fsdmgr_resize_cache(&self, _cache_id: u32) -> u32 {
        ERROR_SUCCESS
    }

    pub fn fsdmgr_flush_cache(&mut self, cache_id: u32) -> u32 {
        let Some(entry) = self.fsdmgr_cache_entry_mut(cache_id) else {
            return ERROR_INVALID_PARAMETER;
        };
        entry.disable_flush = true;
        ERROR_SUCCESS
    }

    pub fn fsdmgr_sync_cache(&self, _cache_id: u32) -> u32 {
        ERROR_SUCCESS
    }

    pub fn fsdmgr_invalidate_cache(&self, _cache_id: u32) -> u32 {
        ERROR_SUCCESS
    }

    pub fn fsdmgr_cache_io_control(&mut self, cache_id: u32, ioctl: u32) -> u32 {
        let Some(entry) = self.fsdmgr_cache_entry_mut(cache_id) else {
            return ERROR_INVALID_PARAMETER;
        };
        if ioctl == IOCTL_DISK_DELETE_SECTORS {
            entry.disable_delete = true;
            return 1;
        }
        0
    }

    pub fn fsdmgr_read_disk(
        &self,
        disk_ptr: u32,
        sector: u32,
        sectors: u32,
        bytes_per_sector: u32,
    ) -> Result<Vec<u8>> {
        if disk_ptr == 0 || bytes_per_sector == 0 {
            return Err(Error::InvalidArgument(
                "invalid FSDMGR disk read parameters".to_owned(),
            ));
        }
        let byte_len = sectors
            .checked_mul(bytes_per_sector)
            .ok_or_else(|| Error::InvalidArgument("FSDMGR disk read size overflow".to_owned()))?;
        let mut out = vec![0; byte_len as usize];
        let copy_len = FSDMGR_SYNTHETIC_DISK_SECTOR_SIZE.min(bytes_per_sector) as usize;
        for relative_sector in 0..sectors {
            let sector_key = sector.wrapping_add(relative_sector);
            if let Some(stored) = self.fsdmgr_disk_sectors.get(&(disk_ptr, sector_key)) {
                let start = (relative_sector * bytes_per_sector) as usize;
                out[start..start + copy_len].copy_from_slice(&stored[..copy_len]);
            }
        }
        Ok(out)
    }

    pub fn fsdmgr_write_disk(
        &mut self,
        disk_ptr: u32,
        sector: u32,
        sectors: u32,
        bytes_per_sector: u32,
        bytes: &[u8],
    ) -> u32 {
        if disk_ptr == 0 || bytes_per_sector == 0 {
            return ERROR_INVALID_PARAMETER;
        }
        let Some(expected_len) = sectors
            .checked_mul(bytes_per_sector)
            .map(|len| len as usize)
        else {
            return ERROR_INVALID_PARAMETER;
        };
        if bytes.len() < expected_len {
            return ERROR_INVALID_PARAMETER;
        }
        let copy_len = FSDMGR_SYNTHETIC_DISK_SECTOR_SIZE.min(bytes_per_sector) as usize;
        for relative_sector in 0..sectors {
            let start = (relative_sector * bytes_per_sector) as usize;
            let sector_key = sector.wrapping_add(relative_sector);
            let mut stored = vec![0; FSDMGR_SYNTHETIC_DISK_SECTOR_SIZE as usize];
            stored[..copy_len].copy_from_slice(&bytes[start..start + copy_len]);
            if stored.iter().all(|byte| *byte == 0) {
                self.fsdmgr_disk_sectors.remove(&(disk_ptr, sector_key));
            } else {
                self.fsdmgr_disk_sectors
                    .insert((disk_ptr, sector_key), stored);
            }
        }
        ERROR_SUCCESS
    }

    pub fn fsdmgr_delete_disk_sectors(&mut self, disk_ptr: u32, sector: u32, sectors: u32) -> u32 {
        if disk_ptr == 0 {
            return ERROR_INVALID_PARAMETER;
        }
        for relative_sector in 0..sectors {
            self.fsdmgr_disk_sectors
                .remove(&(disk_ptr, sector.wrapping_add(relative_sector)));
        }
        ERROR_SUCCESS
    }

    pub fn fsdmgr_format_disk(&mut self, disk_ptr: u32) -> u32 {
        if disk_ptr == 0 {
            return ERROR_INVALID_PARAMETER;
        }
        self.fsdmgr_disk_sectors
            .retain(|(stored_disk_ptr, _), _| *stored_disk_ptr != disk_ptr);
        ERROR_SUCCESS
    }

    pub fn fsdmgr_disk_name(&self, disk_ptr: u32) -> Option<String> {
        if disk_ptr == 0 {
            return None;
        }
        let name = self
            .handles
            .iter()
            .find_map(|(_, object)| match object {
                KernelObject::Volume(volume) if volume.disk_ptr == Some(disk_ptr) => {
                    Some(volume.guest_root.trim_matches('\\').to_owned())
                }
                _ => None,
            })
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| "Storage Card".to_owned());
        Some(name)
    }

    pub fn fsdmgr_disk_info(&self, disk_ptr: u32) -> Option<[u32; 6]> {
        if disk_ptr == 0 {
            return None;
        }
        if let Some(info) = self.fsdmgr_disk_info_overrides.get(&disk_ptr) {
            return Some(*info);
        }
        let total_sectors = 0x0002_0000;
        Some([
            total_sectors.max(1),
            FSDMGR_SYNTHETIC_DISK_SECTOR_SIZE,
            1,
            1,
            total_sectors.max(1),
            0,
        ])
    }

    pub fn fsdmgr_disk_info_override(&self, disk_ptr: u32) -> Option<[u32; 6]> {
        self.fsdmgr_disk_info_overrides.get(&disk_ptr).copied()
    }

    pub fn fsdmgr_set_disk_info(&mut self, disk_ptr: u32, info: [u32; 6]) -> u32 {
        if disk_ptr == 0 {
            return ERROR_INVALID_PARAMETER;
        }
        self.fsdmgr_disk_info_overrides.insert(disk_ptr, info);
        ERROR_SUCCESS
    }

    pub fn fsdmgr_set_fmd_xip_mode(&mut self, disk_ptr: u32, enabled: bool) -> u32 {
        if self.fsdmgr_disk_info(disk_ptr).is_none() {
            return ERROR_INVALID_PARAMETER;
        }
        self.fsdmgr_fmd_xip_modes.insert(disk_ptr, enabled);
        ERROR_SUCCESS
    }

    pub fn fsdmgr_fmd_xip_mode(&self, disk_ptr: u32) -> Option<bool> {
        self.fsdmgr_disk_info(disk_ptr)?;
        Some(
            self.fsdmgr_fmd_xip_modes
                .get(&disk_ptr)
                .copied()
                .unwrap_or(false),
        )
    }

    pub fn fsdmgr_set_fmd_block_lock(
        &mut self,
        disk_ptr: u32,
        start_block: u32,
        block_count: u32,
        locked: bool,
    ) -> u32 {
        if self.fsdmgr_disk_info(disk_ptr).is_none() {
            return ERROR_INVALID_PARAMETER;
        }
        if locked {
            if block_count != 0 {
                self.fsdmgr_fmd_block_locks
                    .entry(disk_ptr)
                    .or_default()
                    .push(FsdmgrFmdBlockLockRange {
                        start_block,
                        block_count,
                    });
            }
        } else {
            self.fsdmgr_fmd_block_locks.remove(&disk_ptr);
        }
        ERROR_SUCCESS
    }

    pub fn fsdmgr_fmd_block_locked(&self, disk_ptr: u32, block: u32) -> Option<bool> {
        self.fsdmgr_disk_info(disk_ptr)?;
        Some(
            self.fsdmgr_fmd_block_locks
                .get(&disk_ptr)
                .is_some_and(|ranges| {
                    ranges.iter().any(|range| {
                        range.block_count != 0
                            && block.wrapping_sub(range.start_block) < range.block_count
                    })
                }),
        )
    }

    pub fn fsdmgr_fmd_block_status(&self, disk_ptr: u32, block: u32) -> Option<u32> {
        const BLOCK_STATUS_UNKNOWN: u32 = 0x01;
        const BLOCK_STATUS_READONLY: u32 = 0x04;

        let info = self.fsdmgr_disk_info(disk_ptr)?;
        if block >= info[0] {
            return Some(BLOCK_STATUS_UNKNOWN);
        }
        let mut status = self
            .fsdmgr_fmd_block_statuses
            .get(&(disk_ptr, block))
            .copied()
            .unwrap_or(0);
        if self
            .fsdmgr_fmd_block_locked(disk_ptr, block)
            .unwrap_or(false)
        {
            status |= BLOCK_STATUS_READONLY;
        }
        Some(status)
    }

    pub fn fsdmgr_set_fmd_block_status(&mut self, disk_ptr: u32, block: u32, status: u32) -> u32 {
        const BLOCK_STATUS_BAD: u32 = 0x02;
        const BLOCK_STATUS_READONLY: u32 = 0x04;
        const BLOCK_STATUS_RESERVED: u32 = 0x08;
        const BLOCK_STATUS_XIP: u32 = 0x10;

        let Some(info) = self.fsdmgr_disk_info(disk_ptr) else {
            return ERROR_INVALID_PARAMETER;
        };
        if block >= info[0] {
            return ERROR_INVALID_PARAMETER;
        }
        let status = status
            & (BLOCK_STATUS_BAD | BLOCK_STATUS_READONLY | BLOCK_STATUS_RESERVED | BLOCK_STATUS_XIP);
        if status != 0 {
            *self
                .fsdmgr_fmd_block_statuses
                .entry((disk_ptr, block))
                .or_default() |= status;
        }
        ERROR_SUCCESS
    }

    pub fn fsdmgr_set_fmd_sector_size(&mut self, disk_ptr: u32, sector_size: u32) -> u32 {
        if self.fsdmgr_disk_info(disk_ptr).is_none() {
            return ERROR_INVALID_PARAMETER;
        }
        self.fsdmgr_fmd_sector_sizes.insert(disk_ptr, sector_size);
        ERROR_SUCCESS
    }

    pub fn fsdmgr_fmd_sector_size(&self, disk_ptr: u32) -> Option<u32> {
        self.fsdmgr_disk_info(disk_ptr)?;
        Some(
            self.fsdmgr_fmd_sector_sizes
                .get(&disk_ptr)
                .copied()
                .unwrap_or(FSDMGR_SYNTHETIC_DISK_SECTOR_SIZE),
        )
    }

    pub fn fsdmgr_set_fmd_interface_disk(&mut self, disk_ptr: u32) -> u32 {
        if self.fsdmgr_disk_info(disk_ptr).is_none() {
            return ERROR_INVALID_PARAMETER;
        }
        self.fsdmgr_fmd_interface_disk = Some(disk_ptr);
        ERROR_SUCCESS
    }

    pub fn fsdmgr_fmd_interface_disk(&self) -> Option<u32> {
        let disk_ptr = self.fsdmgr_fmd_interface_disk?;
        self.fsdmgr_disk_info(disk_ptr)?;
        Some(disk_ptr)
    }

    pub fn fsdmgr_set_fmd_region_table(&mut self, disk_ptr: u32, regions: Vec<[u32; 7]>) -> u32 {
        if self.fsdmgr_disk_info(disk_ptr).is_none() {
            return ERROR_INVALID_PARAMETER;
        }
        let previous_regions = self.fsdmgr_fmd_region_tables.get(&disk_ptr).cloned();
        self.fsdmgr_fmd_region_tables.insert(disk_ptr, regions);
        let status = self.fsdmgr_sync_fmd_flash_layout_sector(disk_ptr);
        if status != ERROR_SUCCESS {
            if let Some(previous_regions) = previous_regions {
                self.fsdmgr_fmd_region_tables
                    .insert(disk_ptr, previous_regions);
            } else {
                self.fsdmgr_fmd_region_tables.remove(&disk_ptr);
            }
            return status;
        }
        ERROR_SUCCESS
    }

    pub fn fsdmgr_fmd_region_count(&self, disk_ptr: u32) -> Option<u32> {
        self.fsdmgr_disk_info(disk_ptr)?;
        Some(
            self.fsdmgr_fmd_region_tables
                .get(&disk_ptr)
                .map(|regions| regions.len() as u32)
                .unwrap_or(0),
        )
    }

    pub fn fsdmgr_fmd_region_table(&self, disk_ptr: u32) -> Option<Vec<[u32; 7]>> {
        self.fsdmgr_disk_info(disk_ptr)?;
        Some(
            self.fsdmgr_fmd_region_tables
                .get(&disk_ptr)
                .cloned()
                .unwrap_or_default(),
        )
    }

    pub fn fsdmgr_fmd_block_sector_span(
        &self,
        disk_ptr: u32,
        block: u32,
    ) -> Option<std::result::Result<(u32, u32), u32>> {
        let info = self.fsdmgr_disk_info(disk_ptr)?;
        let regions = self
            .fsdmgr_fmd_region_tables
            .get(&disk_ptr)
            .cloned()
            .unwrap_or_default();
        if !regions.is_empty() {
            let mut region_start_sector = 0u32;
            for region in regions {
                let start_phys_block = region[1];
                let num_phys_blocks = region[2];
                let sectors_per_block = region[4];
                let Some(region_end_block) = start_phys_block.checked_add(num_phys_blocks) else {
                    return Some(Err(ERROR_INVALID_PARAMETER));
                };
                let Some(region_sectors) = num_phys_blocks.checked_mul(sectors_per_block) else {
                    return Some(Err(ERROR_INVALID_PARAMETER));
                };
                if block >= start_phys_block && block < region_end_block {
                    if sectors_per_block == 0 {
                        return Some(Err(ERROR_INVALID_PARAMETER));
                    }
                    let Some(relative_sector) = block
                        .checked_sub(start_phys_block)
                        .and_then(|relative_block| relative_block.checked_mul(sectors_per_block))
                    else {
                        return Some(Err(ERROR_INVALID_PARAMETER));
                    };
                    let Some(start_sector) = region_start_sector.checked_add(relative_sector)
                    else {
                        return Some(Err(ERROR_INVALID_PARAMETER));
                    };
                    return Some(Ok((start_sector, sectors_per_block)));
                }
                let Some(next_region_start_sector) =
                    region_start_sector.checked_add(region_sectors)
                else {
                    return Some(Err(ERROR_INVALID_PARAMETER));
                };
                region_start_sector = next_region_start_sector;
            }
            return Some(Err(ERROR_INVALID_PARAMETER));
        }

        if block >= info[0] {
            return Some(Err(ERROR_INVALID_PARAMETER));
        }
        let bytes_per_sector = info[1].max(1);
        let sectors_per_block = (self
            .fsdmgr_fmd_sector_size(disk_ptr)
            .unwrap_or(bytes_per_sector)
            .max(bytes_per_sector)
            / bytes_per_sector)
            .max(1);
        let Some(start_sector) = block.checked_mul(sectors_per_block) else {
            return Some(Err(ERROR_INVALID_PARAMETER));
        };
        Some(Ok((start_sector, sectors_per_block)))
    }

    pub fn fsdmgr_fmd_physical_sector_addr(
        &self,
        disk_ptr: u32,
        sector: u32,
    ) -> Option<std::result::Result<u32, u32>> {
        let info = self.fsdmgr_disk_info(disk_ptr)?;
        let regions = self
            .fsdmgr_fmd_region_tables
            .get(&disk_ptr)
            .cloned()
            .unwrap_or_default();
        if regions.is_empty() {
            if sector >= info[0] {
                return Some(Err(ERROR_INVALID_PARAMETER));
            }
            return Some(Ok(sector));
        }

        let data_bytes_per_sector = info[1].max(1);
        let mut region_start_sector = 0u32;
        for region in regions {
            let start_phys_block = region[1];
            let num_phys_blocks = region[2];
            let sectors_per_block = region[4];
            if sectors_per_block == 0 || num_phys_blocks == 0 {
                return Some(Err(ERROR_INVALID_PARAMETER));
            }
            let bytes_per_block = if region[5] != 0 {
                region[5]
            } else {
                let Some(bytes_per_block) = sectors_per_block.checked_mul(data_bytes_per_sector)
                else {
                    return Some(Err(ERROR_INVALID_PARAMETER));
                };
                bytes_per_block
            };
            let Some(region_sectors) = num_phys_blocks.checked_mul(sectors_per_block) else {
                return Some(Err(ERROR_INVALID_PARAMETER));
            };
            let Some(region_end_sector) = region_start_sector.checked_add(region_sectors) else {
                return Some(Err(ERROR_INVALID_PARAMETER));
            };
            if sector >= region_start_sector && sector < region_end_sector {
                let sector_offset_in_region = sector - region_start_sector;
                let block_offset = sector_offset_in_region / sectors_per_block;
                let sector_offset_in_block = sector_offset_in_region % sectors_per_block;
                let Some(block_id) = start_phys_block.checked_add(block_offset) else {
                    return Some(Err(ERROR_INVALID_PARAMETER));
                };
                let Some(block_addr) = block_id.checked_mul(bytes_per_block) else {
                    return Some(Err(ERROR_INVALID_PARAMETER));
                };
                let Some(sector_offset) = sector_offset_in_block.checked_mul(data_bytes_per_sector)
                else {
                    return Some(Err(ERROR_INVALID_PARAMETER));
                };
                let Some(sector_addr) = block_addr.checked_add(sector_offset) else {
                    return Some(Err(ERROR_INVALID_PARAMETER));
                };
                return Some(Ok(sector_addr));
            }
            region_start_sector = region_end_sector;
        }
        Some(Err(ERROR_INVALID_PARAMETER))
    }

    pub fn fsdmgr_read_fmd_reserved_region(
        &mut self,
        disk_ptr: u32,
        name: [u8; 8],
        start: u32,
        len: u32,
    ) -> Option<std::result::Result<Vec<u8>, u32>> {
        self.fsdmgr_disk_info(disk_ptr)?;
        let Some(end) = start.checked_add(len) else {
            return Some(Err(ERROR_INVALID_PARAMETER));
        };
        let mut bytes = vec![0; len as usize];
        if let Some(stored) = self.fsdmgr_fmd_reserved_regions.get(&(disk_ptr, name)) {
            let copy_start = start.min(stored.len() as u32) as usize;
            let copy_end = end.min(stored.len() as u32) as usize;
            if copy_end > copy_start {
                bytes[..copy_end - copy_start].copy_from_slice(&stored[copy_start..copy_end]);
            }
        }
        Some(Ok(bytes))
    }

    pub fn fsdmgr_write_fmd_reserved_region(
        &mut self,
        disk_ptr: u32,
        name: [u8; 8],
        start: u32,
        bytes: &[u8],
    ) -> u32 {
        if self.fsdmgr_disk_info(disk_ptr).is_none() {
            return ERROR_INVALID_PARAMETER;
        }
        let Some(end) = start.checked_add(bytes.len() as u32) else {
            return ERROR_INVALID_PARAMETER;
        };
        let stored = self
            .fsdmgr_fmd_reserved_regions
            .entry((disk_ptr, name))
            .or_default();
        if stored.len() < end as usize {
            stored.resize(end as usize, 0);
        }
        stored[start as usize..end as usize].copy_from_slice(bytes);
        let status = self.fsdmgr_sync_fmd_flash_layout_sector(disk_ptr);
        if status != ERROR_SUCCESS {
            return status;
        }
        ERROR_SUCCESS
    }

    pub fn fsdmgr_fmd_reserved_entries(&self, disk_ptr: u32) -> Option<Vec<([u8; 8], u32)>> {
        self.fsdmgr_disk_info(disk_ptr)?;
        let block_size = self
            .fsdmgr_fmd_sector_size(disk_ptr)
            .unwrap_or(FSDMGR_SYNTHETIC_DISK_SECTOR_SIZE)
            .max(1);
        let entries = self
            .fsdmgr_fmd_reserved_regions
            .iter()
            .filter_map(|((stored_disk, name), bytes)| {
                (*stored_disk == disk_ptr).then(|| {
                    let blocks = (bytes.len() as u32).saturating_add(block_size - 1) / block_size;
                    (*name, blocks)
                })
            })
            .collect();
        Some(entries)
    }

    pub fn fsdmgr_fmd_reserved_count(&self, disk_ptr: u32) -> Option<u32> {
        self.fsdmgr_disk_info(disk_ptr)?;
        Some(
            self.fsdmgr_fmd_reserved_regions
                .keys()
                .filter(|(stored_disk, _)| *stored_disk == disk_ptr)
                .count() as u32,
        )
    }

    fn fsdmgr_sync_fmd_flash_layout_sector(&mut self, disk_ptr: u32) -> u32 {
        let Some(reserved_entries) = self.fsdmgr_fmd_reserved_entries(disk_ptr) else {
            return ERROR_INVALID_PARAMETER;
        };
        let regions = self
            .fsdmgr_fmd_region_tables
            .get(&disk_ptr)
            .cloned()
            .unwrap_or_default();
        let Some(reserved_bytes) = reserved_entries
            .len()
            .checked_mul(FSDMGR_FLS_RESERVED_ENTRY_SIZE)
        else {
            return ERROR_INVALID_PARAMETER;
        };
        let Some(region_bytes) = regions.len().checked_mul(FSDMGR_FLS_REGION_ENTRY_SIZE) else {
            return ERROR_INVALID_PARAMETER;
        };
        let Some(layout_bytes) = FSDMGR_FLS_HEADER_SIZE
            .checked_add(reserved_bytes)
            .and_then(|len| len.checked_add(region_bytes))
        else {
            return ERROR_INVALID_PARAMETER;
        };
        if layout_bytes > FSDMGR_SYNTHETIC_DISK_SECTOR_SIZE as usize {
            return ERROR_INVALID_PARAMETER;
        }

        let mut sector = vec![0; FSDMGR_SYNTHETIC_DISK_SECTOR_SIZE as usize];
        sector[..FSDMGR_FLS_SIGNATURE.len()].copy_from_slice(FSDMGR_FLS_SIGNATURE);
        sector[8..12].copy_from_slice(&(reserved_bytes as u32).to_le_bytes());
        sector[12..16].copy_from_slice(&(region_bytes as u32).to_le_bytes());

        let mut offset = FSDMGR_FLS_HEADER_SIZE;
        for (name, block_count) in reserved_entries {
            sector[offset..offset + 8].copy_from_slice(&name);
            sector[offset + 8..offset + 12].copy_from_slice(&0u32.to_le_bytes());
            sector[offset + 12..offset + 16].copy_from_slice(&block_count.to_le_bytes());
            offset += FSDMGR_FLS_RESERVED_ENTRY_SIZE;
        }
        for region in regions {
            for value in region {
                sector[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
                offset += 4;
            }
        }
        self.fsdmgr_disk_sectors.insert((disk_ptr, 0), sector);
        ERROR_SUCCESS
    }

    pub fn fsdmgr_write_fmd_raw_blocks(
        &mut self,
        disk_ptr: u32,
        start_block: u32,
        block_count: u32,
        bytes: &[u8],
    ) -> u32 {
        let Some(info) = self.fsdmgr_disk_info(disk_ptr) else {
            return ERROR_INVALID_PARAMETER;
        };
        let block_size = info[1];
        if block_count == 0 || block_size == 0 {
            return ERROR_INVALID_PARAMETER;
        }
        let Some(end_block) = block_count
            .checked_sub(1)
            .and_then(|last_offset| start_block.checked_add(last_offset))
        else {
            return ERROR_INVALID_PARAMETER;
        };
        if start_block >= info[0] || end_block >= info[0] {
            return ERROR_INVALID_PARAMETER;
        }
        let Some(required_bytes) = block_count
            .checked_mul(block_size)
            .map(|required| required as usize)
        else {
            return ERROR_INVALID_PARAMETER;
        };
        if bytes.len() < required_bytes {
            return ERROR_INVALID_PARAMETER;
        }
        self.fsdmgr_write_disk(disk_ptr, start_block, block_count, block_size, bytes)
    }

    pub fn fsdmgr_disk_io_control_status(&mut self, disk_ptr: u32, ioctl: u32) -> u32 {
        if disk_ptr == 0 {
            return ERROR_INVALID_PARAMETER;
        }
        match ioctl {
            1
            | 0x0007_1c00
            | DISK_IOCTL_INITIALIZED
            | IOCTL_DISK_INITIALIZED
            | DISK_IOCTL_FORMAT_MEDIA
            | IOCTL_DISK_FORMAT_MEDIA
            | IOCTL_DISK_FORMAT_VOLUME
            | IOCTL_DISK_SCAN_VOLUME
            | IOCTL_DISK_DELETE_SECTORS
            | IOCTL_DISK_FLUSH_CACHE => ERROR_SUCCESS,
            IOCTL_DISK_SET_STANDBY_TIMER
            | IOCTL_DISK_STANDBY_NOW
            | IOCTL_DISK_DELETE_CLUSTER
            | IOCTL_DISK_READ_CDROM
            | IOCTL_DISK_WRITE_CDROM => ERROR_NOT_SUPPORTED,
            2 | 3 | 0x0007_4008 | 0x0007_800c => ERROR_NOT_SUPPORTED,
            _ => ERROR_NOT_SUPPORTED,
        }
    }

    fn ensure_volume_owner(&self, volume: &VolumeObject, handle: u32) -> Result<()> {
        if volume.owner_process_id != self.current_process_id {
            return Err(Error::AccessDenied(format!(
                "process {} cannot access volume handle 0x{handle:08x} owned by process {}",
                self.current_process_id, volume.owner_process_id
            )));
        }
        Ok(())
    }

    pub fn unmount_volume_handle(&mut self, handle: u32) -> Result<bool> {
        let KernelObject::Volume(volume) = self.handles.get(handle)?.clone() else {
            return Err(Error::InvalidHandle(handle));
        };
        self.ensure_volume_owner(&volume, handle)?;
        self.unmount_guest_root(&volume.guest_root);
        self.drop_fsdmgr_volume_locks_for_handle(handle);
        self.handles.close(handle)?;
        Ok(true)
    }

    pub fn volume_root_for_handle(&self, handle: u32) -> Result<String> {
        let KernelObject::Volume(volume) = self.handles.get(handle)? else {
            return Err(Error::InvalidHandle(handle));
        };
        self.ensure_volume_owner(volume, handle)?;
        Ok(volume.guest_root.clone())
    }

    pub fn host_path_to_guest_mount(&self, host_path: &std::path::Path) -> Option<String> {
        self.files.host_path_to_guest_mount(host_path)
    }

    pub fn host_path_for_guest(&self, guest_path: &str) -> Result<std::path::PathBuf> {
        self.files.host_path_for_guest(guest_path)
    }

    pub fn recent_file_ops(&self) -> Vec<FileTraceRecord> {
        self.recent_file_ops.iter().cloned().collect()
    }

    pub fn recent_file_open_ops(&self) -> Vec<FileTraceRecord> {
        self.recent_file_open_ops.iter().cloned().collect()
    }

    pub fn recent_process_ops(&self) -> &[ProcessTraceRecord] {
        &self.recent_process_ops
    }

    pub fn recent_event_ops(&self) -> &[EventTraceRecord] {
        &self.recent_event_ops
    }

    pub fn recent_message_ops(&self) -> &[MessageTraceRecord] {
        &self.recent_message_ops
    }

    pub fn recent_device_ops(&self) -> &[DeviceTraceRecord] {
        &self.recent_device_ops
    }

    pub fn device_debug_text(&self) -> String {
        let mut out = String::new();
        let status = self.remote_status();
        let enabled_names = self.devices.enabled_names();
        let _ = writeln!(
            out,
            "  device config: enabled={} names={}",
            enabled_names.len(),
            if enabled_names.is_empty() {
                "none".to_owned()
            } else {
                enabled_names.join(", ")
            }
        );
        let _ = writeln!(
            out,
            "  remote: gps_target={} queued_serial_bytes={} queued_touch_events={} queued_key_events={} imu={}",
            if status.gps_target.is_empty() {
                "<none>"
            } else {
                status.gps_target.as_str()
            },
            status.queued_serial_bytes,
            status.queued_touch_events,
            status.queued_key_events,
            self.remote.imu_state()
        );

        let mut devices = self
            .handles
            .iter()
            .filter_map(|(handle, object)| match object {
                KernelObject::Device(device) => Some((handle, device)),
                _ => None,
            })
            .peekable();
        let _ = writeln!(out, "  open device handles:");
        if devices.peek().is_none() {
            let _ = writeln!(out, "    none");
        } else {
            let gps_target = self.remote_gps_target();
            for (handle, device) in devices {
                let (rx, tx) = device.queue_lengths();
                let remote_serial =
                    if !gps_target.is_empty() && device.accepts_remote_serial_target(&gps_target) {
                        " remote-gps-target"
                    } else {
                        ""
                    };
                let serial_detail = if device.is_serial() {
                    let timeouts = device.comm_timeouts();
                    format!(
                        " comm_mask=0x{:08x} timeouts=ri:{}/rtm:{}/rtc:{}/wtm:{}/wtc:{}",
                        device.comm_mask(),
                        timeouts.read_interval_timeout,
                        timeouts.read_total_timeout_multiplier,
                        timeouts.read_total_timeout_constant,
                        timeouts.write_total_timeout_multiplier,
                        timeouts.write_total_timeout_constant
                    )
                } else {
                    String::new()
                };
                let _ = writeln!(
                    out,
                    "    0x{handle:08x} {} kind={:?} backend={:?} host={} serial={} rx={} tx={}{}{}",
                    device.guest_name,
                    device.kind,
                    device.backend,
                    device.host.as_deref().unwrap_or("<none>"),
                    device.is_serial(),
                    rx,
                    tx,
                    serial_detail,
                    remote_serial
                );
            }
        }
        out
    }

    pub fn handle_debug_text(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(
            out,
            "  handles: count={} blocked_waiters={}",
            self.handles.len(),
            self.scheduler.waiter_count()
        );
        if self.handles.is_empty() {
            let _ = writeln!(out, "    none");
        } else {
            for (handle, object) in self.handles.iter() {
                let waiters = self.scheduler.waiter_ids_for_handle(handle);
                let waiter_text = if waiters.is_empty() {
                    "none".to_owned()
                } else {
                    waiters
                        .iter()
                        .map(|wait_id| wait_id.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                };
                let ready_text = self
                    .scheduler
                    .blocked_waits()
                    .filter(|wait| wait.wait_handles.contains(&handle))
                    .map(|wait| {
                        format!(
                            "id{}:tid{}:{}",
                            wait.id,
                            wait.thread_id,
                            self.is_wait_ready(handle, wait.thread_id)
                                .map(|ready| ready.to_string())
                                .unwrap_or_else(|| "invalid".to_owned())
                        )
                    })
                    .collect::<Vec<_>>();
                let ready_text = if ready_text.is_empty() {
                    "none".to_owned()
                } else {
                    ready_text.join(",")
                };
                let _ = writeln!(
                    out,
                    "    0x{handle:08x} {} waiters={} ready_by_waiter={}",
                    self.handle_debug_description(handle, object),
                    waiter_text,
                    ready_text
                );
            }
        }

        out.push_str("  blocked waits:\n");
        let waits = self.scheduler.blocked_waits().collect::<Vec<_>>();
        if waits.is_empty() {
            let _ = writeln!(out, "    none");
        } else {
            for wait in waits {
                let handles = wait
                    .wait_handles
                    .iter()
                    .map(|handle| format!("0x{handle:08x}:{}", self.describe_handle(*handle)))
                    .collect::<Vec<_>>()
                    .join("|");
                let _ = writeln!(
                    out,
                    "    id={} tid={} thread_handle=0x{:08x} kind={:?} timeout={} started={} handles=[{}]",
                    wait.id,
                    wait.thread_id,
                    wait.thread_handle,
                    wait.kind,
                    wait.timeout_ms,
                    wait.wait_started_ms,
                    handles
                );
            }
        }
        out
    }

    fn handle_debug_description(&self, handle: u32, object: &KernelObject) -> String {
        match object {
            KernelObject::Thread(_) => self
                .handles
                .thread_snapshot_for_handle(handle)
                .map(|thread| {
                    format!(
                        "thread(id={},start=0x{:08x},param=0x{:08x},priority={},signaled={},suspend={},exit=0x{:08x})",
                        thread.thread_id,
                        thread.start_address,
                        thread.parameter,
                        thread.priority,
                        thread.signaled,
                        thread.suspend_count,
                        thread.exit_code
                    )
                })
                .unwrap_or_else(|| "invalid thread birth".to_owned()),
            KernelObject::Device(device) => {
                let (rx, tx) = device.queue_lengths();
                format!(
                    "device(name={},kind={:?},backend={:?},host={},serial={},rx={},tx={})",
                    device.guest_name,
                    device.kind,
                    device.backend,
                    device.host.as_deref().unwrap_or("<none>"),
                    device.is_serial(),
                    rx,
                    tx
                )
            }
            KernelObject::FileMapping(mapping) => format!(
                "mapping(name={},size={},protect=0x{:08x},file_id={},views={},closed={})",
                mapping.name.as_deref().unwrap_or("<unnamed>"),
                mapping.size,
                mapping.protect,
                mapping
                    .file_id
                    .map(|file_id| file_id.to_string())
                    .unwrap_or_else(|| "none".to_owned()),
                mapping.views.len(),
                mapping.closed
            ),
            KernelObject::Process(_) => self.handles.describe_handle(handle),
            _ => self.handles.describe_handle(handle),
        }
    }

    pub fn file_io_stats(&self) -> FileIoStats {
        self.files.io_stats()
    }

    pub fn scheduler_stats(&self) -> SchedulerStats {
        self.scheduler.stats()
    }

    pub fn gwe_stats(&self) -> GweStats {
        self.gwe.stats()
    }

    pub fn record_file_trace(&mut self, record: FileTraceRecord) {
        self.push_file_trace(record);
    }

    pub fn record_process_trace(&mut self, record: ProcessTraceRecord) {
        if self.recent_process_ops.len() == PROCESS_TRACE_LIMIT {
            self.recent_process_ops.remove(0);
        }
        self.recent_process_ops.push(record);
    }

    fn record_event_trace(&mut self, record: EventTraceRecord) {
        if self.recent_event_ops.len() == EVENT_TRACE_LIMIT {
            self.recent_event_ops.remove(0);
        }
        self.recent_event_ops.push(record);
    }

    pub fn record_wait_event_trace(
        &mut self,
        op: &'static str,
        handle: Option<u32>,
        result: Option<bool>,
        detail: String,
    ) {
        self.record_event_trace(EventTraceRecord {
            op,
            handle,
            name: None,
            manual_reset: None,
            signaled: None,
            result,
            detail: Some(detail),
        });
    }

    fn record_message_trace(&mut self, record: MessageTraceRecord) {
        if self.recent_message_ops.len() == MESSAGE_TRACE_LIMIT {
            self.recent_message_ops.remove(0);
        }
        self.recent_message_ops.push(record);
    }

    fn record_device_trace(&mut self, record: DeviceTraceRecord) {
        if self.recent_device_ops.len() == DEVICE_TRACE_LIMIT {
            self.recent_device_ops.remove(0);
        }
        self.recent_device_ops.push(record);
    }

    pub fn record_comm_trace(
        &mut self,
        op: &'static str,
        handle: u32,
        input_len: u32,
        input_preview: Option<String>,
        output_capacity: u32,
        success: bool,
        bytes_returned: u32,
        output_preview: Option<String>,
        detail: Option<String>,
    ) {
        let (device, backend) = match self.handles.get(handle) {
            Ok(KernelObject::Device(device)) => (
                Some(device.guest_name.clone()),
                Some(format!("{:?}", device.backend)),
            ),
            _ => (None, None),
        };
        self.record_device_trace(DeviceTraceRecord {
            op,
            handle,
            device,
            backend,
            ioctl_code: 0,
            input_len,
            input_preview,
            output_capacity,
            success,
            bytes_returned,
            output_preview,
            detail,
        });
    }

    pub fn record_window_lifecycle_trace(
        &mut self,
        op: &'static str,
        thread_id: u32,
        hwnd: Option<u32>,
        result: Option<u32>,
        extra: Option<String>,
    ) {
        let detail = match (
            hwnd.and_then(|hwnd| self.gwe.window(hwnd).map(window_lifecycle_detail)),
            extra,
        ) {
            (Some(detail), Some(extra)) => Some(format!("{detail}/{extra}")),
            (Some(detail), None) => Some(detail),
            (None, extra) => extra,
        };
        self.record_message_trace(MessageTraceRecord {
            op,
            thread_id,
            hwnd,
            msg: None,
            wparam: None,
            lparam: None,
            screen_pos: None,
            source: None,
            result,
            detail,
        });
    }

    fn record_message_op(
        &mut self,
        op: &'static str,
        thread_id: u32,
        message: &Message,
        result: Option<u32>,
        detail: Option<String>,
    ) {
        let source = if message.source == crate::ce::gwe::MSGSRC_UNKNOWN {
            crate::ce::gwe::MSGSRC_SOFTWARE_POST
        } else {
            message.source
        };
        let screen_pos = message.mouse_pos_at_post.or_else(|| {
            matches!(
                message.msg,
                crate::ce::gwe::WM_MOUSEMOVE
                    | crate::ce::gwe::WM_LBUTTONDOWN
                    | crate::ce::gwe::WM_LBUTTONUP
            )
            .then_some(message.lparam)
        });
        self.record_message_trace(MessageTraceRecord {
            op,
            thread_id,
            hwnd: Some(message.hwnd),
            msg: Some(message.msg),
            wparam: Some(message.wparam),
            lparam: Some(message.lparam),
            screen_pos,
            source: Some(source),
            result,
            detail,
        });
    }

    fn push_file_trace(&mut self, record: FileTraceRecord) {
        if is_file_open_trace(record.op) {
            if self.recent_file_open_ops.len() == FILE_TRACE_LIMIT {
                self.recent_file_open_ops.pop_front();
            }
            self.recent_file_open_ops.push_back(record.clone());
        }
        if self.recent_file_ops.len() == FILE_TRACE_LIMIT {
            self.recent_file_ops.pop_front();
        }
        self.recent_file_ops.push_back(record);
    }

    pub fn path_for_handle(&self, handle: u32) -> Option<String> {
        match self.handles.get(handle).ok()? {
            KernelObject::File(file) => Some(file.guest_path.clone()),
            KernelObject::Device(device) => Some(device.guest_name.clone()),
            _ => None,
        }
    }

    pub fn is_serial_device_handle(&self, handle: u32) -> bool {
        matches!(
            self.handles.get(handle),
            Ok(KernelObject::Device(device)) if device.is_serial()
        )
    }

    pub fn serial_read_ready(&self, handle: u32) -> bool {
        let Ok(KernelObject::Device(device)) = self.handles.get(handle) else {
            return false;
        };
        if !device.is_serial() {
            return false;
        }
        device.rx_len() != 0
            || (device.accepts_remote_serial_target(&self.remote_gps_target())
                && self.remote.serial_byte_count() != 0)
    }

    pub fn register_overlapped_serial_read(
        &mut self,
        overlapped_ptr: u32,
        read: PendingOverlappedSerialRead,
    ) {
        self.pending_overlapped_serial_reads
            .insert(overlapped_ptr, read);
    }

    pub fn pending_overlapped_serial_read(
        &self,
        handle: u32,
        overlapped_ptr: u32,
    ) -> Option<PendingOverlappedSerialRead> {
        self.pending_overlapped_serial_reads
            .get(&overlapped_ptr)
            .copied()
            .filter(|read| read.handle == handle)
    }

    pub fn complete_overlapped_serial_read_into<F>(
        &mut self,
        handle: u32,
        overlapped_ptr: u32,
        mut write: F,
    ) -> Result<Option<(PendingOverlappedSerialRead, u32)>>
    where
        F: FnMut(&[u8]) -> Result<()>,
    {
        let Some(read) = self.pending_overlapped_serial_read(handle, overlapped_ptr) else {
            return Ok(None);
        };
        let transferred = self.read_file_into(read.handle, read.requested, |bytes| write(bytes))?;
        if transferred == 0 {
            return Ok(None);
        }
        self.pending_overlapped_serial_reads.remove(&overlapped_ptr);
        Ok(Some((read, transferred)))
    }

    fn signal_pending_overlapped_serial_reads(&mut self, handle: u32) -> usize {
        if !self.serial_read_ready(handle) {
            return 0;
        }
        let event_handles = self
            .pending_overlapped_serial_reads
            .values()
            .filter(|read| read.handle == handle)
            .map(|read| read.event_handle)
            .collect::<Vec<_>>();
        let mut signaled = 0;
        for event_handle in event_handles {
            if self.set_event(event_handle) {
                signaled += 1;
            }
        }
        signaled
    }

    pub fn serial_empty_read_timeout_ms(&self, handle: u32, requested: u32) -> Option<u32> {
        match self.handles.get(handle) {
            Ok(KernelObject::Device(device)) => device.empty_read_timeout_ms(requested),
            _ => Some(0),
        }
    }

    pub fn get_comm_timeouts(&self, handle: u32) -> Result<crate::ce::devices::CommTimeouts> {
        match self.handles.get(handle)? {
            KernelObject::Device(device) if device.is_serial() => Ok(device.comm_timeouts()),
            _ => Err(Error::InvalidHandle(handle)),
        }
    }

    pub fn get_comm_dcb(&self, handle: u32) -> Result<crate::ce::devices::CommDcb> {
        match self.handles.get(handle)? {
            KernelObject::Device(device) if device.is_serial() => Ok(device.dcb()),
            _ => Err(Error::InvalidHandle(handle)),
        }
    }

    pub fn set_comm_dcb(&mut self, handle: u32, dcb: crate::ce::devices::CommDcb) -> Result<()> {
        match self.handles.get_mut(handle)? {
            KernelObject::Device(device) if device.is_serial() => {
                device.set_dcb(dcb);
                Ok(())
            }
            _ => Err(Error::InvalidHandle(handle)),
        }
    }

    pub fn set_comm_timeouts(
        &mut self,
        handle: u32,
        timeouts: crate::ce::devices::CommTimeouts,
    ) -> Result<()> {
        match self.handles.get_mut(handle)? {
            KernelObject::Device(device) if device.is_serial() => {
                device.set_comm_timeouts(timeouts);
                Ok(())
            }
            _ => Err(Error::InvalidHandle(handle)),
        }
    }

    pub fn get_comm_mask(&self, handle: u32) -> Result<u32> {
        match self.handles.get(handle)? {
            KernelObject::Device(device) if device.is_serial() => Ok(device.comm_mask()),
            _ => Err(Error::InvalidHandle(handle)),
        }
    }

    pub fn set_comm_mask(&mut self, handle: u32, mask: u32) -> Result<()> {
        let wait_ids = self.scheduler.serial_event_waiter_ids_for_handle(handle);
        match self.handles.get_mut(handle)? {
            KernelObject::Device(device) if device.is_serial() => {
                device.set_comm_mask(mask);
                if !wait_ids.is_empty() {
                    self.comm_event_mask_changed_waits
                        .extend(wait_ids.iter().copied());
                    self.scheduler.queue_serial_event_wake_candidates(handle);
                }
                Ok(())
            }
            _ => Err(Error::InvalidHandle(handle)),
        }
    }

    pub fn serial_comm_event_ready(&self, handle: u32) -> bool {
        const EV_RXCHAR: u32 = 0x0001;
        self.get_comm_mask(handle)
            .is_ok_and(|mask| mask & EV_RXCHAR != 0)
            && self.serial_read_ready(handle)
    }

    pub fn serial_comm_event_value(&self, handle: u32) -> u32 {
        const EV_RXCHAR: u32 = 0x0001;
        if self.serial_comm_event_ready(handle) {
            EV_RXCHAR
        } else {
            0
        }
    }

    pub fn purge_comm(&mut self, handle: u32, flags: u32) -> Result<()> {
        let target = self.remote_gps_target();
        let clear_remote_rx = flags & PURGE_RXCLEAR != 0
            && matches!(
                self.handles.get(handle),
                Ok(KernelObject::Device(device))
                    if device.is_serial() && device.accepts_remote_serial_target(&target)
            );
        match self.handles.get_mut(handle)? {
            KernelObject::Device(device) if device.is_serial() => {
                device.purge_comm(flags);
                if clear_remote_rx {
                    let _ = self.remote.read_serial_bytes(usize::MAX);
                }
                Ok(())
            }
            _ => Err(Error::InvalidHandle(handle)),
        }
    }

    pub fn comm_queue_lengths(&self, handle: u32) -> Result<(u32, u32)> {
        let target = self.remote_gps_target();
        match self.handles.get(handle)? {
            KernelObject::Device(device) if device.is_serial() => {
                let (mut rx_len, tx_len) = device.queue_lengths();
                if device.accepts_remote_serial_target(&target) {
                    rx_len = rx_len.saturating_add(self.remote.serial_byte_count() as u32);
                }
                Ok((rx_len, tx_len))
            }
            _ => Err(Error::InvalidHandle(handle)),
        }
    }

    pub(crate) fn drain_remote_serial_to_handle(&mut self, handle: u32, max_bytes: usize) -> usize {
        if max_bytes == 0 {
            return 0;
        }
        let target = self.remote_gps_target();
        let should_drain = matches!(
            self.handles.get(handle),
            Ok(KernelObject::Device(device))
                if device.is_serial() && device.accepts_remote_serial_target(&target)
        );
        if !should_drain || self.remote.serial_byte_count() == 0 {
            return 0;
        }
        let bytes = self.remote.read_serial_bytes(max_bytes);
        let count = bytes.len();
        if count == 0 {
            return 0;
        }
        if let Ok(KernelObject::Device(device)) = self.handles.get_mut(handle) {
            device.enqueue_rx(&bytes);
        }
        self.signal_pending_overlapped_serial_reads(handle);
        self.record_comm_trace(
            "SerialRemoteDrain",
            handle,
            max_bytes as u32,
            None,
            0,
            true,
            count as u32,
            file_trace_preview(&bytes),
            Some(format!("queued_remote_to_rx={count}")),
        );
        count
    }

    fn drain_remote_serial_to_matching_open_device(&mut self) -> usize {
        if self.remote.serial_byte_count() == 0 {
            return 0;
        }
        let target = self.remote_gps_target();
        if target.is_empty() {
            return 0;
        }
        let Some(handle) = self
            .handles
            .iter()
            .find_map(|(handle, object)| match object {
                KernelObject::Device(device)
                    if device.is_serial() && device.accepts_remote_serial_target(&target) =>
                {
                    Some(handle)
                }
                _ => None,
            })
        else {
            return 0;
        };
        self.drain_remote_serial_to_handle(handle, usize::MAX)
    }

    pub fn poll_host_serial_to_handle(&mut self, handle: u32, max_bytes: usize) -> usize {
        if max_bytes == 0 {
            return 0;
        }
        match self.handles.get_mut(handle) {
            Ok(KernelObject::Device(device)) if device.is_serial() => {
                device.poll_host_serial(max_bytes)
            }
            _ => 0,
        }
    }

    pub fn create_file_w(
        &mut self,
        path: &str,
        desired_access: u32,
        creation_disposition: u32,
    ) -> Result<u32> {
        if self.devices.is_configured_device_name(path) {
            let session = self.devices.open(path)?;
            let handle = self.handles.insert(KernelObject::Device(session));
            self.drain_remote_serial_to_handle(handle, usize::MAX);
            self.push_file_trace(FileTraceRecord {
                op: "CreateFileW",
                handle: Some(handle),
                path: Some(path.to_owned()),
                preview: None,
                requested: Some(desired_access),
                transferred: None,
                position: Some(u64::from(creation_disposition)),
                result: Some(handle),
                error: None,
            });
            return Ok(handle);
        }

        let existed_before = self.files.file_attributes_w(path).is_ok();
        let file_id = match self
            .files
            .create_file_w(path, desired_access, creation_disposition)
        {
            Ok(file_id) => file_id,
            Err(err) => {
                self.push_file_trace(FileTraceRecord {
                    op: "CreateFileW",
                    handle: None,
                    path: Some(path.to_owned()),
                    preview: None,
                    requested: Some(desired_access),
                    transferred: None,
                    position: Some(u64::from(creation_disposition)),
                    result: Some(u32::MAX),
                    error: Some(err.to_string()),
                });
                return Err(err);
            }
        };
        let handle = self.handles.insert(KernelObject::File(FileObject {
            guest_path: path.to_owned(),
            file_id,
            notify_change_pending: false,
        }));
        self.push_file_trace(FileTraceRecord {
            op: "CreateFileW",
            handle: Some(handle),
            path: Some(path.to_owned()),
            preview: None,
            requested: Some(desired_access),
            transferred: None,
            position: Some(u64::from(creation_disposition)),
            result: Some(handle),
            error: None,
        });
        let create_event = matches!(
            creation_disposition,
            CREATE_NEW | CREATE_ALWAYS | OPEN_ALWAYS
        ) && !existed_before;
        let update_event =
            matches!(creation_disposition, CREATE_ALWAYS | TRUNCATE_EXISTING) && existed_before;
        if create_event {
            self.post_shell_file_change_notifications(SHCNE_CREATE, Some(path), None);
            self.signal_file_change_notifications(SHCNE_CREATE, Some(path), None);
        } else if update_event {
            self.post_shell_file_change_notifications(SHCNE_UPDATEITEM, Some(path), None);
            self.signal_file_change_notifications(SHCNE_UPDATEITEM, Some(path), None);
        }
        Ok(handle)
    }

    pub fn open_existing_readonly(&mut self, path: &str) -> Result<u32> {
        self.create_file_w(path, GENERIC_READ, OPEN_EXISTING)
    }

    pub fn open_existing_readwrite(&mut self, path: &str) -> Result<u32> {
        self.create_file_w(path, GENERIC_READ | GENERIC_WRITE, OPEN_EXISTING)
    }

    pub fn read_file(&mut self, handle: u32, requested: u32) -> Result<Vec<u8>> {
        self.poll_host_serial_to_handle(handle, requested as usize);
        self.drain_remote_serial_to_handle(handle, requested as usize);
        let path = self.path_for_handle(handle);
        let start_position = match self.handles.get(handle) {
            Ok(KernelObject::File(file)) => self
                .files
                .open_file(file.file_id)
                .ok()
                .map(|file| file.cursor() as u64),
            _ => None,
        };
        let result = match self.handles.get_mut(handle) {
            Ok(object) => match object {
                KernelObject::File(file) => self.files.read_file(file.file_id, requested),
                KernelObject::Device(device) => Ok(device.read_file(requested)),
                _ => Ok(Vec::new()),
            },
            Err(err) => Err(err),
        };
        let end_position = match self.handles.get(handle) {
            Ok(KernelObject::File(file)) => self
                .files
                .open_file(file.file_id)
                .ok()
                .map(|file| file.cursor() as u64),
            _ => None,
        };
        self.push_file_trace(FileTraceRecord {
            op: "ReadFile",
            handle: Some(handle),
            path,
            preview: file_read_trace_preview(
                start_position,
                end_position,
                result.as_deref().unwrap_or(&[]),
            ),
            requested: Some(requested),
            transferred: result.as_ref().ok().map(|bytes| bytes.len() as u32),
            position: start_position,
            result: result.as_ref().ok().map(|_| 1),
            error: result.as_ref().err().map(ToString::to_string),
        });
        if self.is_serial_device_handle(handle) {
            let transferred = result
                .as_ref()
                .ok()
                .map(|bytes| bytes.len() as u32)
                .unwrap_or(0);
            self.record_comm_trace(
                "ReadFile",
                handle,
                requested,
                None,
                requested,
                result.is_ok(),
                transferred,
                result
                    .as_ref()
                    .ok()
                    .and_then(|bytes| file_trace_preview(bytes)),
                None,
            );
        }
        result
    }

    pub fn read_file_into<F>(&mut self, handle: u32, requested: u32, mut write: F) -> Result<u32>
    where
        F: FnMut(&[u8]) -> Result<()>,
    {
        self.poll_host_serial_to_handle(handle, requested as usize);
        self.drain_remote_serial_to_handle(handle, requested as usize);
        let path = self.path_for_handle(handle);
        let start_position = match self.handles.get(handle) {
            Ok(KernelObject::File(file)) => self
                .files
                .open_file(file.file_id)
                .ok()
                .map(|file| file.cursor() as u64),
            _ => None,
        };
        let mut preview_bytes = Vec::new();
        let result = match self.handles.get_mut(handle) {
            Ok(object) => match object {
                KernelObject::File(file) => {
                    self.files.read_file_into(file.file_id, requested, |bytes| {
                        if preview_bytes.len() < FILE_TRACE_PREVIEW_LIMIT {
                            let remaining = FILE_TRACE_PREVIEW_LIMIT - preview_bytes.len();
                            preview_bytes.extend_from_slice(&bytes[..bytes.len().min(remaining)]);
                        }
                        write(bytes)
                    })
                }
                KernelObject::Device(device) => {
                    let bytes = device.read_file(requested);
                    preview_bytes
                        .extend_from_slice(&bytes[..bytes.len().min(FILE_TRACE_PREVIEW_LIMIT)]);
                    let transferred = bytes.len() as u32;
                    write(&bytes).map(|_| transferred)
                }
                _ => write(&[]).map(|_| 0),
            },
            Err(err) => Err(err),
        };
        let end_position = match self.handles.get(handle) {
            Ok(KernelObject::File(file)) => self
                .files
                .open_file(file.file_id)
                .ok()
                .map(|file| file.cursor() as u64),
            _ => None,
        };
        self.push_file_trace(FileTraceRecord {
            op: "ReadFile",
            handle: Some(handle),
            path,
            preview: file_read_trace_preview(start_position, end_position, &preview_bytes),
            requested: Some(requested),
            transferred: result.as_ref().ok().copied(),
            position: start_position,
            result: result.as_ref().ok().map(|_| 1),
            error: result.as_ref().err().map(ToString::to_string),
        });
        if self.is_serial_device_handle(handle) {
            self.record_comm_trace(
                "ReadFile",
                handle,
                requested,
                None,
                requested,
                result.is_ok(),
                result.as_ref().ok().copied().unwrap_or(0),
                file_trace_preview(&preview_bytes),
                Some("read_file_into".to_owned()),
            );
        }
        result
    }

    pub fn read_file_at(
        &mut self,
        file_id: u32,
        offset: usize,
        requested: usize,
    ) -> Result<Vec<u8>> {
        self.files.read_at(file_id, offset, requested)
    }

    pub fn read_file_handle_at(
        &mut self,
        handle: u32,
        offset: usize,
        requested: usize,
    ) -> Result<Vec<u8>> {
        let path = self.path_for_handle(handle);
        let KernelObject::File(file) = self.handles.get(handle)? else {
            return Err(crate::error::Error::InvalidHandle(handle));
        };
        let result = self.files.read_at(file.file_id, offset, requested);
        self.push_file_trace(FileTraceRecord {
            op: "ReadFileScatter",
            handle: Some(handle),
            path,
            preview: result
                .as_deref()
                .ok()
                .and_then(|bytes| file_read_trace_preview(Some(offset as u64), None, bytes)),
            requested: Some(requested as u32),
            transferred: result.as_ref().ok().map(|bytes| bytes.len() as u32),
            position: Some(offset as u64),
            result: result.as_ref().ok().map(|_| 1),
            error: result.as_ref().err().map(ToString::to_string),
        });
        result
    }

    pub fn read_guest_file(&self, path: &str) -> Result<Vec<u8>> {
        self.files.read_guest_file(path)
    }

    pub fn file_attributes_w(&self, path: &str) -> Result<FindData> {
        self.files.file_attributes_w(path)
    }

    pub fn create_directory_w(&mut self, path: &str) -> Result<()> {
        let result = self.files.create_directory_w(path);
        if result.is_ok() {
            self.post_shell_file_change_notifications(SHCNE_MKDIR, Some(path), None);
            self.signal_file_change_notifications(SHCNE_MKDIR, Some(path), None);
        }
        result
    }

    pub fn remove_directory_w(&mut self, path: &str) -> Result<()> {
        let result = self.files.remove_directory_w(path);
        if result.is_ok() {
            self.post_shell_file_change_notifications(SHCNE_RMDIR, Some(path), None);
            self.signal_file_change_notifications(SHCNE_RMDIR, Some(path), None);
        }
        result
    }

    pub fn delete_file_w(&mut self, path: &str) -> Result<()> {
        let result = self.files.delete_file_w(path);
        if result.is_ok() {
            self.post_shell_file_change_notifications(SHCNE_DELETE, Some(path), None);
            self.signal_file_change_notifications(SHCNE_DELETE, Some(path), None);
        }
        self.push_file_trace(FileTraceRecord {
            op: "DeleteFileW",
            handle: None,
            path: Some(path.to_owned()),
            preview: None,
            requested: None,
            transferred: None,
            position: None,
            result: Some(u32::from(result.is_ok())),
            error: result.as_ref().err().map(ToString::to_string),
        });
        result
    }

    pub fn move_file_w(&mut self, existing_path: &str, new_path: &str) -> Result<()> {
        let existing_was_dir = self
            .files
            .file_attributes_w(existing_path)
            .ok()
            .is_some_and(|data| data.attributes & FILE_ATTRIBUTE_DIRECTORY != 0);
        let cross_volume = self.files.volume_root_for_guest_path(existing_path)
            != self.files.volume_root_for_guest_path(new_path);
        let same_parent = shell_change_parent_path(existing_path)
            .eq_ignore_ascii_case(&shell_change_parent_path(new_path));
        let result = self.files.move_file_w(existing_path, new_path);
        if result.is_ok() {
            if cross_volume {
                self.post_shell_file_change_notifications(SHCNE_CREATE, Some(new_path), None);
                self.signal_file_change_notifications(SHCNE_CREATE, Some(new_path), None);
                if self.files.file_attributes_w(existing_path).is_err() {
                    self.post_shell_file_change_notifications(
                        SHCNE_DELETE,
                        Some(existing_path),
                        None,
                    );
                    self.signal_file_change_notifications(SHCNE_DELETE, Some(existing_path), None);
                }
            } else {
                self.post_shell_file_change_notifications(
                    SHCNE_RENAMEITEM,
                    Some(existing_path),
                    Some(new_path),
                );
                self.signal_file_move_notifications(
                    existing_path,
                    new_path,
                    existing_was_dir,
                    same_parent,
                );
            }
        }
        self.push_file_trace(FileTraceRecord {
            op: "MoveFileW",
            handle: None,
            path: Some(existing_path.to_owned()),
            preview: Some(format!("new_path={new_path}")),
            requested: None,
            transferred: None,
            position: None,
            result: Some(u32::from(result.is_ok())),
            error: result.as_ref().err().map(ToString::to_string),
        });
        result
    }

    pub fn delete_and_rename_file_w(&mut self, old_path: &str, new_path: &str) -> Result<()> {
        let result = self.files.delete_and_rename_file_w(old_path, new_path);
        if result.is_ok() {
            self.post_shell_file_change_notifications(SHCNE_DELETE, Some(old_path), None);
            self.signal_file_change_notifications(SHCNE_DELETE, Some(old_path), None);
            self.post_shell_file_change_notifications(
                SHCNE_RENAMEITEM,
                Some(new_path),
                Some(old_path),
            );
            self.signal_file_move_notifications(
                new_path,
                old_path,
                false,
                shell_change_parent_path(new_path)
                    .eq_ignore_ascii_case(&shell_change_parent_path(old_path)),
            );
        }
        self.push_file_trace(FileTraceRecord {
            op: "DeleteAndRenameFileW",
            handle: None,
            path: Some(old_path.to_owned()),
            preview: Some(format!("new_path={new_path}")),
            requested: None,
            transferred: None,
            position: None,
            result: Some(u32::from(result.is_ok())),
            error: result.as_ref().err().map(ToString::to_string),
        });
        result
    }

    pub fn copy_file_w(
        &mut self,
        existing_path: &str,
        new_path: &str,
        fail_if_exists: bool,
    ) -> Result<()> {
        let result = self
            .files
            .copy_file_w(existing_path, new_path, fail_if_exists);
        if result.is_ok() {
            self.post_shell_file_change_notifications(SHCNE_CREATE, Some(new_path), None);
            self.signal_file_change_notifications(SHCNE_CREATE, Some(new_path), None);
        }
        result
    }

    pub fn set_file_attributes_w(&mut self, path: &str, attributes: u32) -> Result<()> {
        let result = self.files.set_file_attributes_w(path, attributes);
        if result.is_ok() {
            self.post_shell_file_change_notifications(SHCNE_ATTRIBUTES, Some(path), None);
            self.signal_file_change_notifications(SHCNE_ATTRIBUTES, Some(path), None);
        }
        result
    }

    pub fn write_file(&mut self, handle: u32, bytes: &[u8]) -> Result<FileIoResult> {
        let path = self.path_for_handle(handle);
        let result = match self.handles.get_mut(handle) {
            Ok(object) => match object {
                KernelObject::File(file) => {
                    let result = self.files.write_file(file.file_id, bytes);
                    if result
                        .as_ref()
                        .is_ok_and(|io| io.success && io.bytes_transferred != 0)
                    {
                        file.notify_change_pending = true;
                    }
                    result
                }
                KernelObject::Device(device) => Ok(FileIoResult {
                    success: true,
                    bytes_transferred: device.write_file(bytes),
                }),
                _ => Ok(FileIoResult {
                    success: false,
                    bytes_transferred: 0,
                }),
            },
            Err(err) => Err(err),
        };
        self.push_file_trace(FileTraceRecord {
            op: "WriteFile",
            handle: Some(handle),
            path: path.clone(),
            preview: file_trace_preview(bytes),
            requested: Some(bytes.len() as u32),
            transferred: result.as_ref().ok().map(|io| io.bytes_transferred),
            position: None,
            result: result.as_ref().ok().map(|io| u32::from(io.success)),
            error: result.as_ref().err().map(ToString::to_string),
        });
        if self.is_serial_device_handle(handle) {
            let success = result.as_ref().is_ok_and(|io| io.success);
            let transferred = result
                .as_ref()
                .ok()
                .map(|io| io.bytes_transferred)
                .unwrap_or(0);
            self.record_comm_trace(
                "WriteFile",
                handle,
                bytes.len() as u32,
                file_trace_preview(bytes),
                0,
                success,
                transferred,
                None,
                None,
            );
        }
        if result
            .as_ref()
            .is_ok_and(|io| io.success && io.bytes_transferred != 0)
            && let Some(path) = path.as_deref()
            && path.starts_with('\\')
        {
            self.post_shell_file_change_notifications(SHCNE_UPDATEITEM, Some(path), None);
            self.signal_file_change_notifications(SHCNE_UPDATEITEM, Some(path), None);
        }
        result
    }

    pub fn write_file_at(
        &mut self,
        file_id: u32,
        offset: usize,
        bytes: &[u8],
    ) -> Result<FileIoResult> {
        self.files.write_at(file_id, offset, bytes)
    }

    pub fn write_file_handle_at(
        &mut self,
        handle: u32,
        offset: usize,
        bytes: &[u8],
    ) -> Result<FileIoResult> {
        let path = self.path_for_handle(handle);
        let KernelObject::File(file) = self.handles.get(handle)? else {
            return Err(crate::error::Error::InvalidHandle(handle));
        };
        let file_id = file.file_id;
        let result = self.files.write_at(file_id, offset, bytes);
        if result
            .as_ref()
            .is_ok_and(|io| io.success && io.bytes_transferred != 0)
            && let Ok(KernelObject::File(file)) = self.handles.get_mut(handle)
        {
            file.notify_change_pending = true;
        }
        self.push_file_trace(FileTraceRecord {
            op: "WriteFileGather",
            handle: Some(handle),
            path: path.clone(),
            preview: file_trace_preview(bytes),
            requested: Some(bytes.len() as u32),
            transferred: result.as_ref().ok().map(|io| io.bytes_transferred),
            position: Some(offset as u64),
            result: result.as_ref().ok().map(|io| u32::from(io.success)),
            error: result.as_ref().err().map(ToString::to_string),
        });
        if result
            .as_ref()
            .is_ok_and(|io| io.success && io.bytes_transferred != 0)
            && let Some(path) = path.as_deref()
            && path.starts_with('\\')
        {
            self.post_shell_file_change_notifications(SHCNE_UPDATEITEM, Some(path), None);
            self.signal_file_change_notifications(SHCNE_UPDATEITEM, Some(path), None);
        }
        result
    }

    pub fn set_file_pointer(
        &mut self,
        handle: u32,
        distance: i64,
        move_method: u32,
    ) -> Result<usize> {
        let path = self.path_for_handle(handle);
        let KernelObject::File(file) = self.handles.get(handle)? else {
            self.push_file_trace(FileTraceRecord {
                op: "SetFilePointer",
                handle: Some(handle),
                path,
                preview: None,
                requested: Some(move_method),
                transferred: None,
                position: None,
                result: Some(u32::MAX),
                error: Some("invalid handle".to_owned()),
            });
            return Err(crate::error::Error::InvalidHandle(handle));
        };
        let file_id = file.file_id;
        let current = self.files.open_file(file_id)?.cursor() as i64;
        let size = self.files.file_size(file_id)? as i64;
        let position = match move_method {
            0 => distance,
            1 => current.saturating_add(distance),
            2 => size.saturating_add(distance),
            _ => {
                return Err(crate::error::Error::InvalidArgument(
                    "bad move method".to_owned(),
                ));
            }
        };
        if position < 0 {
            self.push_file_trace(FileTraceRecord {
                op: "SetFilePointer",
                handle: Some(handle),
                path,
                preview: None,
                requested: Some(move_method),
                transferred: None,
                position: None,
                result: Some(u32::MAX),
                error: Some("negative file pointer".to_owned()),
            });
            return Err(crate::error::Error::InvalidArgument(
                "negative file pointer".to_owned(),
            ));
        }
        let result = self.files.set_file_pointer(file_id, position as usize);
        self.push_file_trace(FileTraceRecord {
            op: "SetFilePointer",
            handle: Some(handle),
            path,
            preview: Some(format!("distance={distance} method={move_method}")),
            requested: Some(move_method),
            transferred: None,
            position: result.as_ref().ok().map(|position| *position as u64),
            result: result.as_ref().ok().map(|position| *position as u32),
            error: result.as_ref().err().map(ToString::to_string),
        });
        result
    }

    pub fn get_file_size(&self, handle: u32) -> Result<usize> {
        let KernelObject::File(file) = self.handles.get(handle)? else {
            return Err(crate::error::Error::InvalidHandle(handle));
        };
        self.files.file_size(file.file_id)
    }

    pub fn set_end_of_file(&mut self, handle: u32) -> Result<bool> {
        let KernelObject::File(file) = self.handles.get(handle)? else {
            return Err(crate::error::Error::InvalidHandle(handle));
        };
        self.files.set_end_of_file(file.file_id)
    }

    pub fn file_attributes_by_handle(&self, handle: u32) -> Result<u32> {
        let KernelObject::File(file) = self.handles.get(handle)? else {
            return Err(crate::error::Error::InvalidHandle(handle));
        };
        self.files.file_attributes_by_id(file.file_id)
    }

    /// Returns (creation_time, last_access_time, last_write_time) as Windows FILETIME values.
    pub fn get_file_time(&self, handle: u32) -> Result<(u64, u64, u64)> {
        let KernelObject::File(file) = self.handles.get(handle)? else {
            return Err(crate::error::Error::InvalidHandle(handle));
        };
        self.files.file_times_by_id(file.file_id)
    }

    pub fn file_position(&self, handle: u32) -> Result<usize> {
        let KernelObject::File(file) = self.handles.get(handle)? else {
            return Err(crate::error::Error::InvalidHandle(handle));
        };
        Ok(self.files.open_file(file.file_id)?.cursor())
    }

    pub fn file_is_eof(&self, handle: u32) -> Result<bool> {
        let KernelObject::File(file) = self.handles.get(handle)? else {
            return Err(crate::error::Error::InvalidHandle(handle));
        };
        self.files.file_is_eof(file.file_id)
    }

    pub fn flush_file_buffers(&mut self, handle: u32) -> Result<bool> {
        let KernelObject::File(file) = self.handles.get(handle)? else {
            return Err(crate::error::Error::InvalidHandle(handle));
        };
        self.files.flush(file.file_id)?;
        Ok(true)
    }

    pub fn find_first_file_w(&mut self, pattern: &str) -> Result<(u32, FindData)> {
        let (find_id, data) = match self.files.find_first_file_w(pattern) {
            Ok(result) => result,
            Err(err) => {
                self.push_file_trace(FileTraceRecord {
                    op: "FindFirstFileW",
                    handle: None,
                    path: Some(pattern.to_owned()),
                    preview: None,
                    requested: None,
                    transferred: None,
                    position: None,
                    result: Some(u32::MAX),
                    error: Some(err.to_string()),
                });
                return Err(err);
            }
        };
        let handle = self.handles.insert(KernelObject::FindFile(FindFileObject {
            guest_pattern: pattern.to_owned(),
            find_id,
        }));
        self.push_file_trace(FileTraceRecord {
            op: "FindFirstFileW",
            handle: Some(handle),
            path: Some(pattern.to_owned()),
            preview: Some(format!("{} attr=0x{:08x}", data.file_name, data.attributes)),
            requested: None,
            transferred: Some(data.file_size as u32),
            position: None,
            result: Some(handle),
            error: None,
        });
        Ok((handle, data))
    }

    pub fn find_next_file_w(&mut self, handle: u32) -> Result<Option<FindData>> {
        let KernelObject::FindFile(find) = self.handles.get(handle)?.clone() else {
            return Err(crate::error::Error::InvalidHandle(handle));
        };
        let guest_pattern = find.guest_pattern.clone();
        let result = self.files.find_next_file_w(find.find_id);
        match &result {
            Ok(Some(data)) => self.push_file_trace(FileTraceRecord {
                op: "FindNextFileW",
                handle: Some(handle),
                path: Some(guest_pattern.clone()),
                preview: Some(format!("{} attr=0x{:08x}", data.file_name, data.attributes)),
                requested: None,
                transferred: Some(data.file_size as u32),
                position: None,
                result: Some(1),
                error: None,
            }),
            Ok(None) => self.push_file_trace(FileTraceRecord {
                op: "FindNextFileW",
                handle: Some(handle),
                path: Some(guest_pattern.clone()),
                preview: None,
                requested: None,
                transferred: None,
                position: None,
                result: Some(0),
                error: None,
            }),
            Err(err) => self.push_file_trace(FileTraceRecord {
                op: "FindNextFileW",
                handle: Some(handle),
                path: Some(guest_pattern),
                preview: None,
                requested: None,
                transferred: None,
                position: None,
                result: Some(0),
                error: Some(err.to_string()),
            }),
        }
        result
    }

    pub fn find_close(&mut self, handle: u32) -> Result<bool> {
        let KernelObject::FindFile(find) = self.handles.get(handle)?.clone() else {
            return Err(crate::error::Error::InvalidHandle(handle));
        };
        self.files.find_close(find.find_id)?;
        self.handles.close(handle)?;
        Ok(true)
    }

    pub fn find_first_change_notification_w(
        &mut self,
        path: &str,
        recursive: bool,
        notify_filter: u32,
    ) -> Result<u32> {
        self.find_first_change_notification_w_for_process(
            path,
            recursive,
            notify_filter,
            self.current_process_id,
        )
    }

    pub fn find_first_change_notification_w_for_process(
        &mut self,
        path: &str,
        recursive: bool,
        notify_filter: u32,
        owner_process_id: u32,
    ) -> Result<u32> {
        let path = canonical_shell_change_path(path);
        let data = self.files.file_attributes_w(&path)?;
        if data.attributes & FILE_ATTRIBUTE_DIRECTORY == 0 {
            return Err(Error::InvalidArgument(format!(
                "change notification path is not a directory: {path}"
            )));
        }
        let handle = self.handles.insert(KernelObject::FileChangeNotification(
            FileChangeNotificationObject {
                owner_process_id,
                watch_path: normalize_shell_change_path(&path),
                volume_root: self.files.volume_root_for_guest_path(&path),
                recursive,
                notify_filter,
                signaled: false,
                pending_signal_count: 0,
                pending: Vec::new(),
            },
        ));
        self.push_file_trace(FileTraceRecord {
            op: "FindFirstChangeNotificationW",
            handle: Some(handle),
            path: Some(path),
            preview: Some(format!(
                "recursive={recursive} filter=0x{notify_filter:08x}"
            )),
            requested: Some(notify_filter),
            transferred: None,
            position: None,
            result: Some(handle),
            error: None,
        });
        Ok(handle)
    }

    fn ensure_file_change_notification_owner(
        &self,
        notification: &FileChangeNotificationObject,
        handle: u32,
    ) -> Result<()> {
        if notification.owner_process_id != self.current_process_id {
            return Err(Error::AccessDenied(format!(
                "process {} cannot access notification handle 0x{handle:08x} owned by process {}",
                self.current_process_id, notification.owner_process_id
            )));
        }
        Ok(())
    }

    fn ensure_internal_file_change_notification_owner(
        &self,
        notification: &FileChangeNotificationObject,
        handle: u32,
    ) -> Result<()> {
        if notification.owner_process_id != FSDMGR_INTERNAL_PROCESS_ID {
            return Err(Error::AccessDenied(format!(
                "internal FSDMGR notification access denied for handle 0x{handle:08x} owned by process {}",
                notification.owner_process_id
            )));
        }
        Ok(())
    }

    pub fn find_next_change_notification(&mut self, handle: u32) -> Result<bool> {
        self.find_next_change_notification_impl(handle, false)
    }

    pub fn find_next_change_notification_internal(&mut self, handle: u32) -> Result<bool> {
        self.find_next_change_notification_impl(handle, true)
    }

    fn find_next_change_notification_impl(&mut self, handle: u32, internal: bool) -> Result<bool> {
        {
            let KernelObject::FileChangeNotification(notification) = self.handles.get(handle)?
            else {
                return Err(Error::InvalidHandle(handle));
            };
            if internal {
                self.ensure_internal_file_change_notification_owner(notification, handle)?;
            } else {
                self.ensure_file_change_notification_owner(notification, handle)?;
            }
        }
        let KernelObject::FileChangeNotification(notification) = self.handles.get_mut(handle)?
        else {
            return Err(Error::InvalidHandle(handle));
        };
        if notification.pending_signal_count > 0 {
            notification.pending_signal_count -= 1;
        }
        // CE NotifyReset(h, NULL) consumes one pending signal count but does not
        // unlink queued NOTFILEINFO detail records. Those records remain hidden
        // until a later signal lets CeGetFileNotificationInfo drain the list.
        notification.signaled = notification.pending_signal_count > 0;
        Ok(true)
    }

    pub fn file_change_notification_records(&self, handle: u32) -> Result<Vec<FileChangeRecord>> {
        self.file_change_notification_records_impl(handle, false)
    }

    pub fn file_change_notification_records_internal(
        &self,
        handle: u32,
    ) -> Result<Vec<FileChangeRecord>> {
        self.file_change_notification_records_impl(handle, true)
    }

    fn file_change_notification_records_impl(
        &self,
        handle: u32,
        internal: bool,
    ) -> Result<Vec<FileChangeRecord>> {
        let KernelObject::FileChangeNotification(notification) = self.handles.get(handle)? else {
            return Err(Error::InvalidHandle(handle));
        };
        if internal {
            self.ensure_internal_file_change_notification_owner(notification, handle)?;
        } else {
            self.ensure_file_change_notification_owner(notification, handle)?;
        }
        if notification.pending_signal_count == 0 {
            return Ok(Vec::new());
        }
        Ok(notification.pending.clone())
    }

    pub fn drain_file_change_notification_records(
        &mut self,
        handle: u32,
        count: usize,
    ) -> Result<bool> {
        self.drain_file_change_notification_records_impl(handle, count, true)
    }

    pub fn drain_file_change_notification_records_without_requeue(
        &mut self,
        handle: u32,
        count: usize,
    ) -> Result<bool> {
        self.drain_file_change_notification_records_impl(handle, count, false)
    }

    pub fn drain_file_change_notification_records_without_requeue_internal(
        &mut self,
        handle: u32,
        count: usize,
    ) -> Result<bool> {
        self.drain_file_change_notification_records_impl_with_access(handle, count, false, true)
    }

    fn drain_file_change_notification_records_impl(
        &mut self,
        handle: u32,
        count: usize,
        requeue: bool,
    ) -> Result<bool> {
        self.drain_file_change_notification_records_impl_with_access(handle, count, requeue, false)
    }

    fn drain_file_change_notification_records_impl_with_access(
        &mut self,
        handle: u32,
        count: usize,
        requeue: bool,
        internal: bool,
    ) -> Result<bool> {
        {
            let KernelObject::FileChangeNotification(notification) = self.handles.get(handle)?
            else {
                return Err(Error::InvalidHandle(handle));
            };
            if internal {
                self.ensure_internal_file_change_notification_owner(notification, handle)?;
            } else {
                self.ensure_file_change_notification_owner(notification, handle)?;
            }
        }
        let KernelObject::FileChangeNotification(notification) = self.handles.get_mut(handle)?
        else {
            return Err(Error::InvalidHandle(handle));
        };
        let drain_count = count.min(notification.pending.len());
        notification.pending.drain(..drain_count);
        notification.pending_signal_count = notification
            .pending_signal_count
            .saturating_sub(drain_count);
        if notification.pending_signal_count == 0 {
            notification.pending.clear();
        }
        notification.signaled = requeue && notification.pending_signal_count > 0;
        let still_pending = notification.signaled;
        if still_pending {
            self.queue_object_wake_candidates(handle);
        }
        Ok(still_pending)
    }

    pub fn requeue_file_change_notification_if_pending(&mut self, handle: u32) -> Result<bool> {
        self.requeue_file_change_notification_if_pending_impl(handle, false)
    }

    pub fn requeue_file_change_notification_if_pending_internal(
        &mut self,
        handle: u32,
    ) -> Result<bool> {
        self.requeue_file_change_notification_if_pending_impl(handle, true)
    }

    fn requeue_file_change_notification_if_pending_impl(
        &mut self,
        handle: u32,
        internal: bool,
    ) -> Result<bool> {
        {
            let KernelObject::FileChangeNotification(notification) = self.handles.get(handle)?
            else {
                return Err(Error::InvalidHandle(handle));
            };
            if internal {
                self.ensure_internal_file_change_notification_owner(notification, handle)?;
            } else {
                self.ensure_file_change_notification_owner(notification, handle)?;
            }
        }
        let KernelObject::FileChangeNotification(notification) = self.handles.get_mut(handle)?
        else {
            return Err(Error::InvalidHandle(handle));
        };
        notification.signaled = notification.pending_signal_count > 0;
        let still_pending = notification.signaled;
        if still_pending {
            self.queue_object_wake_candidates(handle);
        }
        Ok(still_pending)
    }

    pub fn clear_file_change_notification(&mut self, handle: u32) -> Result<bool> {
        self.find_next_change_notification(handle)
    }

    pub fn clear_file_change_notification_internal(&mut self, handle: u32) -> Result<bool> {
        self.find_next_change_notification_internal(handle)
    }

    pub fn find_close_change_notification(&mut self, handle: u32) -> Result<bool> {
        let is_notification = match self.handles.get(handle)? {
            KernelObject::FileChangeNotification(notification) => {
                self.ensure_file_change_notification_owner(notification, handle)?;
                true
            }
            _ => false,
        };
        self.close_handle(handle)?;
        if is_notification {
            Ok(true)
        } else {
            Err(Error::InvalidHandle(handle))
        }
    }

    pub fn find_close_change_notification_internal(&mut self, handle: u32) -> Result<bool> {
        {
            let KernelObject::FileChangeNotification(notification) = self.handles.get(handle)?
            else {
                return Err(Error::InvalidHandle(handle));
            };
            self.ensure_internal_file_change_notification_owner(notification, handle)?;
        }
        self.handles.close(handle)?;
        Ok(true)
    }

    pub fn device_io_control(
        &mut self,
        handle: u32,
        ioctl_code: u32,
        input: &[u8],
        output_capacity: u32,
    ) -> Result<DeviceIoControlResult> {
        self.device_io_control_with_output_buffer(
            handle,
            ioctl_code,
            input,
            output_capacity,
            output_capacity > 0,
        )
    }

    pub fn device_io_control_with_output_buffer(
        &mut self,
        handle: u32,
        ioctl_code: u32,
        input: &[u8],
        output_capacity: u32,
        output_buffer_present: bool,
    ) -> Result<DeviceIoControlResult> {
        let remote_imu = self.remote.imu_state().clone();
        let (device, backend, detail, result) = match self.handles.get_mut(handle)? {
            KernelObject::Device(device) => {
                let device_name = device.guest_name.clone();
                let backend = format!("{:?}", device.backend);
                device.apply_remote_imu(&remote_imu);
                let result = device.device_io_control_with_output_buffer(
                    ioctl_code,
                    input,
                    output_capacity,
                    output_buffer_present,
                );
                (Some(device_name), Some(backend), None, result)
            }
            other => (
                None,
                None,
                Some(format!("non-device handle: {other:?}")),
                DeviceIoControlResult {
                    success: false,
                    bytes_returned: 0,
                    output: Vec::new(),
                },
            ),
        };
        self.record_device_trace(DeviceTraceRecord {
            op: "DeviceIoControl",
            handle,
            device,
            backend,
            ioctl_code,
            input_len: input.len() as u32,
            input_preview: file_trace_preview(input),
            output_capacity,
            success: result.success,
            bytes_returned: result.bytes_returned,
            output_preview: file_trace_preview(&result.output),
            detail,
        });
        Ok(result)
    }

    pub fn is_file_handle(&self, handle: u32) -> Result<bool> {
        Ok(matches!(self.handles.get(handle)?, KernelObject::File(_)))
    }

    pub fn lock_file_range(
        &mut self,
        handle: u32,
        start: u64,
        length: u64,
        exclusive: bool,
    ) -> Result<FileLockStatus> {
        let KernelObject::File(file) = self.handles.get(handle)? else {
            return Err(Error::InvalidHandle(handle));
        };
        self.files
            .lock_file_range(file.file_id, start, length, exclusive)
    }

    pub fn unlock_file_range(
        &mut self,
        handle: u32,
        start: u64,
        length: u64,
    ) -> Result<FileLockStatus> {
        let KernelObject::File(file) = self.handles.get(handle)? else {
            return Err(Error::InvalidHandle(handle));
        };
        self.files.unlock_file_range(file.file_id, start, length)
    }

    pub fn unlock_file_ranges_owned_by_handle(&mut self, handle: u32) -> Result<FileLockStatus> {
        let KernelObject::File(file) = self.handles.get(handle)? else {
            return Err(Error::InvalidHandle(handle));
        };
        self.files.unlock_file_ranges_owned_by_id(file.file_id)
    }

    pub fn file_handle_has_locks(&self, handle: u32) -> Result<bool> {
        let KernelObject::File(file) = self.handles.get(handle)? else {
            return Err(Error::InvalidHandle(handle));
        };
        self.files.file_has_locks(file.file_id)
    }

    pub fn test_file_lock_range(
        &self,
        handle: u32,
        start: u64,
        length: u64,
        read: bool,
    ) -> Result<FileLockStatus> {
        let KernelObject::File(file) = self.handles.get(handle)? else {
            return Err(Error::InvalidHandle(handle));
        };
        self.files
            .test_file_lock_range(file.file_id, start, length, read)
    }

    pub fn file_cursor(&self, handle: u32) -> Result<usize> {
        let KernelObject::File(file) = self.handles.get(handle)? else {
            return Err(Error::InvalidHandle(handle));
        };
        self.files.file_cursor(file.file_id)
    }

    pub fn close_handle(&mut self, handle: u32) -> Result<bool> {
        let object = self.handles.get(handle)?.clone();
        let detail = self.describe_handle(handle);
        self.pending_overlapped_serial_reads
            .retain(|_, read| read.handle != handle && read.event_handle != handle);
        let name = match &object {
            KernelObject::Event(event) => event.name.clone(),
            KernelObject::Mutex(mutex) => mutex.name.clone(),
            KernelObject::Semaphore(semaphore) => semaphore.name.clone(),
            KernelObject::FileMapping(mapping) => mapping.name.clone(),
            _ => None,
        };
        let trace_close = matches!(
            &object,
            KernelObject::Event(_)
                | KernelObject::Mutex(_)
                | KernelObject::Semaphore(_)
                | KernelObject::FileMapping(_)
        );
        match object {
            KernelObject::File(file) => {
                let close_result = self.files.close(file.file_id);
                self.push_file_trace(FileTraceRecord {
                    op: "CloseHandle",
                    handle: Some(handle),
                    path: Some(file.guest_path.clone()),
                    preview: Some(format!("file_id=0x{:08x}", file.file_id)),
                    requested: Some(file.file_id),
                    transferred: None,
                    position: None,
                    result: Some(u32::from(close_result.is_ok())),
                    error: close_result.as_ref().err().map(ToString::to_string),
                });
                close_result?;
                if file.notify_change_pending {
                    self.signal_file_change_notification_path(
                        FILE_NOTIFY_CHANGE_ATTRIBUTES
                            | FILE_NOTIFY_CHANGE_SIZE
                            | FILE_NOTIFY_CHANGE_LAST_WRITE
                            | FILE_NOTIFY_CHANGE_LAST_ACCESS
                            | FILE_NOTIFY_CHANGE_CREATION,
                        FILE_ACTION_CHANGE_COMPLETED,
                        &file.guest_path,
                    );
                }
            }
            KernelObject::FindFile(find) => self.files.find_close(find.find_id)?,
            KernelObject::Volume(volume) => {
                self.ensure_volume_owner(&volume, handle)?;
                self.unmount_guest_root(&volume.guest_root);
                self.drop_fsdmgr_volume_locks_for_handle(handle);
            }
            KernelObject::FileChangeNotification(notification) => {
                self.ensure_file_change_notification_owner(&notification, handle)?;
            }
            KernelObject::MessageQueue(endpoint) => {
                self.handles.close(handle)?;
                self.close_message_queue_endpoint(endpoint.queue_id, endpoint.read_access);
                return Ok(true);
            }
            KernelObject::ApiSet(api_set) => {
                if let Some(set_id) = api_set.registered_id {
                    self.api_sets.remove(&set_id);
                }
            }
            KernelObject::Event(event) if event.name.is_some() => {
                self.record_event_trace(EventTraceRecord {
                    op: "CloseHandle",
                    handle: Some(handle),
                    name,
                    manual_reset: None,
                    signaled: None,
                    result: Some(true),
                    detail: Some(format!("preserved {detail}")),
                });
                return Ok(true);
            }
            KernelObject::FileMapping(mapping) if mapping.name.is_some() => {
                self.record_event_trace(EventTraceRecord {
                    op: "CloseHandle",
                    handle: Some(handle),
                    name,
                    manual_reset: None,
                    signaled: None,
                    result: Some(true),
                    detail: Some(format!("preserved {detail}")),
                });
                return Ok(true);
            }
            KernelObject::FileMapping(mapping) if !mapping.views.is_empty() => {
                self.handles
                    .close_file_mapping_handle_with_live_views(handle)?;
                self.record_event_trace(EventTraceRecord {
                    op: "CloseHandle",
                    handle: Some(handle),
                    name,
                    manual_reset: None,
                    signaled: None,
                    result: Some(true),
                    detail: Some(format!("closed handle, preserved live views for {detail}")),
                });
                return Ok(true);
            }
            _ => {}
        }
        let was_file_mapping = self.handles.has_file_mapping_object(handle);
        self.handles.close(handle)?;
        if was_file_mapping {
            self.memory.forget_shared_mapping(handle);
        }
        if trace_close {
            self.record_event_trace(EventTraceRecord {
                op: "CloseHandle",
                handle: Some(handle),
                name,
                manual_reset: None,
                signaled: None,
                result: Some(true),
                detail: Some(detail),
            });
        }
        Ok(true)
    }

    pub fn duplicate_handle(
        &mut self,
        handle: u32,
        target_process_id: u32,
        close_source: bool,
    ) -> Result<u32> {
        let object = self.handles.get(handle)?.clone();
        if matches!(object, KernelObject::Thread(_) | KernelObject::Process(_)) {
            if !self.handles.process_id_exists(target_process_id) {
                return Err(Error::InvalidHandle(target_process_id));
            }
            let duplicate = self.handles.try_insert(object)?;
            if close_source {
                self.close_handle(handle)?;
            }
            return Ok(duplicate);
        }
        let duplicate_object = match object {
            KernelObject::File(file) => {
                let duplicate_file_id = self.files.duplicate_open_file(file.file_id)?;
                KernelObject::File(FileObject {
                    guest_path: file.guest_path,
                    file_id: duplicate_file_id,
                    notify_change_pending: file.notify_change_pending,
                })
            }
            KernelObject::FindFile(find) => {
                let duplicate_find_id = self.files.duplicate_find(find.find_id)?;
                KernelObject::FindFile(FindFileObject {
                    guest_pattern: find.guest_pattern,
                    find_id: duplicate_find_id,
                })
            }
            KernelObject::Volume(mut volume) => {
                self.ensure_volume_owner(&volume, handle)?;
                volume.owner_process_id = target_process_id;
                KernelObject::Volume(volume)
            }
            KernelObject::FileChangeNotification(mut notification) => {
                self.ensure_file_change_notification_owner(&notification, handle)?;
                notification.owner_process_id = target_process_id;
                KernelObject::FileChangeNotification(notification)
            }
            other => other,
        };
        let duplicate = self.handles.insert(duplicate_object);
        if close_source {
            if matches!(self.handles.get(handle)?, KernelObject::Volume(_)) {
                self.handles.close(handle)?;
            } else {
                self.close_handle(handle)?;
            }
        }
        Ok(duplicate)
    }

    pub fn create_event_w(
        &mut self,
        name: Option<String>,
        manual_reset: bool,
        initial_state: bool,
    ) -> u32 {
        self.create_event_w_with_trace_detail(name, manual_reset, initial_state, None)
    }

    pub fn create_event_w_with_trace_detail(
        &mut self,
        name: Option<String>,
        manual_reset: bool,
        initial_state: bool,
        trace_detail: Option<String>,
    ) -> u32 {
        let handle = self
            .handles
            .create_event(name.clone(), manual_reset, initial_state);
        let detail = append_trace_detail(self.describe_handle(handle), trace_detail);
        self.record_event_trace(EventTraceRecord {
            op: "CreateEventW",
            handle: Some(handle),
            name,
            manual_reset: Some(manual_reset),
            signaled: Some(initial_state),
            result: Some(handle != 0),
            detail: Some(detail),
        });
        handle
    }

    pub fn open_event_w(&mut self, name: &str) -> Option<u32> {
        let handle = self.handles.open_event(name);
        self.record_event_trace(EventTraceRecord {
            op: "OpenEventW",
            handle,
            name: Some(name.to_owned()),
            manual_reset: None,
            signaled: None,
            result: Some(handle.is_some()),
            detail: handle.map(|handle| self.describe_handle(handle)),
        });
        handle
    }

    pub fn is_named_event_signaled_w(&mut self, name: &str) -> bool {
        let signaled = self.handles.named_event_signaled(name);
        self.record_event_trace(EventTraceRecord {
            op: "IsNamedEventSignaled",
            handle: None,
            name: Some(name.to_owned()),
            manual_reset: None,
            signaled,
            result: Some(signaled.unwrap_or(false)),
            detail: None,
        });
        signaled.unwrap_or(false)
    }

    pub fn create_guest_thread(
        &mut self,
        start_address: u32,
        parameter: u32,
        suspended: bool,
    ) -> (u32, u32) {
        self.try_create_guest_thread(start_address, parameter, suspended)
            .expect("guest thread birth allocation failed")
    }

    pub fn try_create_guest_thread(
        &mut self,
        start_address: u32,
        parameter: u32,
        suspended: bool,
    ) -> Result<(u32, u32)> {
        // Compatibility path for host-created test workers. Guest CreateThread
        // passes the creator TID to the checked path below.
        let owner = self.current_process_birth_key().ok_or_else(|| {
            Error::InvalidArgument("selected process has no live birth".to_owned())
        })?;
        let thread_id = self.threads.allocate_guest_thread_id();
        let handle = self.handles.try_create_thread_in_birth(
            owner,
            thread_id,
            start_address,
            parameter,
            suspended,
        )?;
        Ok((handle, thread_id))
    }

    pub fn try_create_guest_thread_for_creator(
        &mut self,
        creator_thread_id: u32,
        start_address: u32,
        parameter: u32,
        suspended: bool,
    ) -> Result<(u32, u32)> {
        let owner = self
            .handles
            .live_thread_creation_owner_by_id(creator_thread_id)
            .filter(|owner| self.handles.process_birth_can_create_threads(*owner))
            .ok_or_else(|| {
                Error::InvalidArgument(format!(
                    "creator TID {creator_thread_id} has no live process birth"
                ))
            })?;
        let thread_id = self.threads.allocate_guest_thread_id();
        let handle = self.handles.try_create_thread_in_birth(
            owner,
            thread_id,
            start_address,
            parameter,
            suspended,
        )?;
        Ok((handle, thread_id))
    }

    pub fn mark_guest_thread_exited(&mut self, handle: u32, exit_code: u32) -> bool {
        let Some(key) = self.handles.thread_birth_key_for_handle(handle) else {
            return false;
        };
        self.mark_guest_thread_exited_by_birth(key, exit_code)
    }

    pub fn mark_guest_thread_exited_by_id(&mut self, thread_id: u32, exit_code: u32) -> bool {
        let Some(key) = self.handles.thread_birth_key_by_id(thread_id) else {
            return false;
        };
        self.mark_guest_thread_exited_by_birth(key, exit_code)
    }

    pub fn live_thread_birth_key_by_id(&self, thread_id: u32) -> Option<ThreadBirthKey> {
        let birth = self.handles.thread_birth_key_by_id(thread_id)?;
        self.handles
            .thread_execution_live_by_id(thread_id)
            .then_some(birth)
    }

    pub fn is_live_thread_birth(&self, birth: ThreadBirthKey) -> bool {
        self.live_thread_birth_key_by_id(birth.thread_id()) == Some(birth)
    }

    pub(crate) fn mark_guest_thread_exited_by_birth(
        &mut self,
        key: ThreadBirthKey,
        exit_code: u32,
    ) -> bool {
        if !self.handles.contains_thread_birth(key) {
            return false;
        }
        let was_live = self.is_live_thread_birth(key);
        let success = self.handles.mark_thread_exited_by_birth(key, exit_code);
        if success && was_live {
            // Retire only input published to this incarnation. Numeric TID
            // cleanup would also destroy a replacement task's queue.
            self.gwe.retire_thread_birth(key);
            for send_id in self.gwe.terminate_sent_messages_to_birth(key) {
                self.queue_send_reply_wake_candidates(send_id);
            }
            // Explicit legacy HWND fixtures have no frozen window birth, but
            // their finite-Sleep send still carries an exact frame owner.
            for send_id in self.gwe.terminate_deferred_sent_messages_to_birth(key) {
                self.queue_send_reply_wake_candidates(send_id);
            }
            self.release_discarded_message_pointer_payloads();
            self.timers.remove_thread_timers_for_birth(key);
            // A completed owner cannot resume its frame. Cancel through the
            // Kernel removal path so every acquired target pin is released;
            // birth matching must precede numeric TID reuse.
            let wait_ids = self.scheduler.waiter_ids_for_birth(key);
            for wait_id in wait_ids {
                let _ = self.remove_blocked_waiter(wait_id);
            }
            // A live birth only retires its own sends. Numeric legacy fixture
            // sends remain explicitly unowned and cannot gain birth authority.
            self.gwe.terminate_sent_messages_from_sender_birth(key);
            self.queue_object_wake_candidates_for_key(WaitObjectKey::Thread(key));
        }
        success
    }

    /// Roll back a newly created worker before its handle is made public.
    pub fn discard_unpublished_guest_thread(&mut self, handle: u32, thread_id: u32) -> bool {
        self.handles.discard_unpublished_thread(handle, thread_id)
    }

    pub fn guest_thread_id(&self, handle: u32) -> Option<u32> {
        self.handles.thread_id(handle)
    }

    pub fn guest_thread_handle_by_id(&self, thread_id: u32) -> Option<u32> {
        self.handles.thread_handle_by_id(thread_id)
    }

    pub fn guest_thread_id_for_handle(&self, handle: u32, current_thread_id: u32) -> Option<u32> {
        if Self::is_current_thread_pseudo_handle(handle) {
            return Some(current_thread_id);
        }
        self.handles.thread_id(handle)
    }

    pub fn guest_thread_creation_owner_for_handle(
        &self,
        handle: u32,
        current_thread_id: u32,
    ) -> Option<u32> {
        let birth = if Self::is_current_thread_pseudo_handle(handle) {
            self.handles.thread_creation_owner_by_id(current_thread_id)
        } else {
            self.handles.thread_creation_owner_for_handle(handle)
        }?;
        Some(birth.process_id())
    }

    pub fn thread_quantum_for_handle(&self, handle: u32, current_thread_id: u32) -> Option<u32> {
        let thread_id = self.guest_thread_id_for_handle(handle, current_thread_id)?;
        Some(
            self.thread_quantums
                .get(&thread_id)
                .copied()
                .unwrap_or(CE_DEFAULT_THREAD_QUANTUM_MS),
        )
    }

    pub fn set_thread_quantum_for_handle(
        &mut self,
        handle: u32,
        current_thread_id: u32,
        quantum: u32,
    ) -> bool {
        let Some(thread_id) = self.guest_thread_id_for_handle(handle, current_thread_id) else {
            return false;
        };
        self.thread_quantums.insert(thread_id, quantum);
        true
    }

    pub fn guest_thread_exit_code(&self, handle: u32) -> Option<u32> {
        self.handles.thread_exit_code(handle)
    }

    pub fn guest_thread_exit_code_for_handle(
        &self,
        handle: u32,
        _current_thread_id: u32,
    ) -> Option<u32> {
        if Self::is_current_thread_pseudo_handle(handle) {
            return Some(STILL_ACTIVE);
        }
        self.handles.thread_exit_code(handle)
    }

    pub fn process_exit_code(&self, handle: u32) -> Option<u32> {
        self.handles.process_exit_code(handle)
    }

    pub fn process_exit_code_for_handle(&self, handle: u32) -> Option<u32> {
        if Self::is_current_process_pseudo_handle(handle) {
            return Some(self.current_process_exit_code);
        }
        self.handles.process_exit_code(handle)
    }

    pub fn process_id(&self, handle: u32) -> Option<u32> {
        self.handles.process_id(handle)
    }

    pub fn process_id_for_handle(&self, handle: u32) -> Option<u32> {
        if Self::is_current_process_pseudo_handle(handle) {
            return Some(self.current_process_id);
        }
        self.handles.process_id(handle)
    }

    fn process_id_from_id_or_handle(&self, process_id_or_handle: u32) -> Option<u32> {
        if process_id_or_handle == self.current_process_id || process_id_or_handle == u32::MAX {
            return Some(self.current_process_id);
        }
        if self.handles.process_id_exists(process_id_or_handle) {
            return Some(process_id_or_handle);
        }
        self.process_id_for_handle(process_id_or_handle)
    }

    pub fn open_process_handle(&mut self, process_id: u32) -> Option<u32> {
        self.handles.open_process(process_id)
    }

    pub fn open_thread_handle(&mut self, thread_id: u32) -> Option<u32> {
        self.handles.open_thread(thread_id)
    }

    pub fn is_remote_debugger_present_for_handle(&self, handle: u32) -> Option<bool> {
        let process_id = self.process_id_for_handle(handle)?;
        Some(
            self.handles
                .process_debugger_thread_id(process_id)
                .is_some(),
        )
    }

    pub fn debug_active_process(
        &mut self,
        process_id_or_handle: u32,
        debugger_thread_id: u32,
    ) -> u32 {
        let Some(process_id) = self.process_id_from_id_or_handle(process_id_or_handle) else {
            return ERROR_INVALID_PARAMETER;
        };
        if process_id == self.current_process_id {
            return ERROR_INVALID_PARAMETER;
        }
        if self
            .handles
            .process_debugger_thread_id(process_id)
            .is_some()
        {
            return ERROR_ACCESS_DENIED;
        }
        if self
            .handles
            .set_process_debugger_thread_id(process_id, Some(debugger_thread_id))
        {
            0
        } else {
            ERROR_INVALID_PARAMETER
        }
    }

    pub fn debug_active_process_stop(
        &mut self,
        process_id_or_handle: u32,
        debugger_thread_id: u32,
    ) -> u32 {
        let Some(process_id) = self.process_id_from_id_or_handle(process_id_or_handle) else {
            return ERROR_INVALID_PARAMETER;
        };
        if self.handles.process_debugger_thread_id(process_id) != Some(debugger_thread_id) {
            return ERROR_INVALID_PARAMETER;
        }
        if self
            .handles
            .set_process_debugger_thread_id(process_id, None)
        {
            0
        } else {
            ERROR_INVALID_PARAMETER
        }
    }

    /// Authenticate the register-supplying guest thread against its frozen
    /// creation process and the currently selected process. A reused public
    /// TID/PID cannot supply authority for a different process incarnation.
    pub fn checked_guest_process_caller_for_birth(
        &self,
        caller: ThreadBirthKey,
    ) -> Option<ProcessBirthKey> {
        if !self.is_live_thread_birth(caller) {
            return None;
        }
        let process = self
            .handles
            .live_thread_creation_owner_by_id(caller.thread_id())?;
        (self.handles.process_execution_live_by_birth(process)
            && self.handles.process_signaled_by_id(process.process_id()) == Some(false)
            && self.current_process_birth_key() == Some(process))
        .then_some(process)
    }

    /// Guest-facing TerminateProcess entry. Real handles retain their request
    /// semantics; the current-process pseudo handle uses the verified caller's
    /// exact process birth for all exit effects.
    pub fn terminate_process_for_caller_birth(
        &mut self,
        caller: ThreadBirthKey,
        handle: u32,
        exit_code: u32,
    ) -> bool {
        let Some(process) = self.checked_guest_process_caller_for_birth(caller) else {
            return false;
        };
        if Self::is_current_process_pseudo_handle(handle) {
            self.terminate_current_process_for_birth(process, exit_code)
        } else {
            self.terminate_process(handle, exit_code)
        }
    }

    pub fn terminate_current_process_for_caller_birth(
        &mut self,
        caller: ThreadBirthKey,
        exit_code: u32,
    ) -> bool {
        self.terminate_process_for_caller_birth(caller, CE_CURRENT_PROCESS_PSEUDO_HANDLE, exit_code)
    }

    fn terminate_current_process_for_birth(
        &mut self,
        process: ProcessBirthKey,
        exit_code: u32,
    ) -> bool {
        // Preflight every fallible identity/state check before GWE teardown.
        // The &mut Kernel lease prevents a selector change before the exact
        // birth is signaled below.
        if self.current_process_birth_key() != Some(process)
            || !self.handles.process_execution_live_by_birth(process)
            || self.handles.process_signaled_by_id(process.process_id()) != Some(false)
        {
            return false;
        }
        self.destroy_process_windows_for_birth(process, 0);
        if self
            .handles
            .signal_pseudo_process_exit_by_birth(process, exit_code)
            .is_none()
        {
            // The exact-birth signal should be infallible after preflight;
            // never report a completed exit if that invariant is broken.
            return false;
        }
        self.invalidate_selected_process_context();
        self.current_process_exit_code = exit_code;
        self.current_process_signaled = true;
        self.queue_object_wake_candidates_for_key(WaitObjectKey::Process(process));
        true
    }

    pub fn terminate_process(&mut self, handle: u32, exit_code: u32) -> bool {
        if Self::is_current_process_pseudo_handle(handle) {
            // Compatibility for direct Kernel/host fixtures. Guest import and
            // decoded paths must use the caller-birth entry points above.
            let Some(process) = self.current_process_birth_key() else {
                return false;
            };
            return self.terminate_current_process_for_birth(process, exit_code);
        }
        let Some(process_id) = self.handles.process_id(handle) else {
            return false;
        };
        if self
            .handles
            .request_process_termination_by_id(process_id, exit_code)
            .is_none()
        {
            return false;
        }
        true
    }

    pub fn suspend_thread(&mut self, handle: u32) -> ThreadSuspendResult {
        self.handles.suspend_thread(handle)
    }

    pub fn suspend_thread_for_handle(
        &mut self,
        handle: u32,
        current_thread_id: u32,
    ) -> ThreadSuspendResult {
        if Self::is_current_thread_pseudo_handle(handle) {
            if let Some(result) = self.handles.suspend_thread_by_id(current_thread_id) {
                return result;
            }
            return self.suspend_thread_by_id_fallback(current_thread_id);
        }
        self.suspend_thread(handle)
    }

    fn suspend_thread_by_id_fallback(&mut self, thread_id: u32) -> ThreadSuspendResult {
        let previous = self
            .thread_suspend_counts
            .get(&thread_id)
            .copied()
            .unwrap_or(0);
        if previous == MAX_SUSPEND_COUNT {
            return ThreadSuspendResult::SignalRefused;
        }
        self.thread_suspend_counts.insert(thread_id, previous + 1);
        ThreadSuspendResult::Previous(previous)
    }

    pub fn resume_thread(&mut self, handle: u32) -> ThreadResumeResult {
        self.handles.resume_thread(handle)
    }

    pub fn resume_thread_for_handle(
        &mut self,
        handle: u32,
        current_thread_id: u32,
    ) -> ThreadResumeResult {
        if Self::is_current_thread_pseudo_handle(handle) {
            if let Some(result) = self.handles.resume_thread_by_id(current_thread_id) {
                return result;
            }
            return self.resume_thread_by_id_fallback(current_thread_id);
        }
        self.resume_thread(handle)
    }

    fn resume_thread_by_id_fallback(&mut self, thread_id: u32) -> ThreadResumeResult {
        let previous = self
            .thread_suspend_counts
            .get(&thread_id)
            .copied()
            .unwrap_or(0);
        if previous > 1 {
            self.thread_suspend_counts.insert(thread_id, previous - 1);
        } else {
            self.thread_suspend_counts.remove(&thread_id);
        }
        ThreadResumeResult::Previous(previous)
    }

    pub fn thread_priority(&self, handle: u32) -> Option<i32> {
        self.handles.thread_priority(handle)
    }

    pub fn thread_priority_for_handle(&self, handle: u32, current_thread_id: u32) -> Option<i32> {
        if Self::is_current_thread_pseudo_handle(handle) {
            return Some(self.thread_priority_by_id(current_thread_id));
        }
        self.handles.thread_priority(handle)
    }

    pub fn thread_win32_priority(&self, handle: u32) -> Option<u32> {
        self.handles
            .thread_priority(handle)
            .and_then(ce_thread_priority_to_win32)
    }

    pub fn thread_win32_priority_for_handle(
        &self,
        handle: u32,
        current_thread_id: u32,
    ) -> Option<u32> {
        self.thread_priority_for_handle(handle, current_thread_id)
            .and_then(ce_thread_priority_to_win32)
    }

    pub fn thread_priority_by_id(&self, thread_id: u32) -> i32 {
        if let Some(priority) = self.thread_priority_overrides.get(&thread_id) {
            return *priority;
        }
        self.handles
            .thread_priority_by_id(thread_id)
            .unwrap_or(CE_THREAD_PRIORITY_NORMAL)
    }

    pub fn set_thread_ce_priority(&mut self, handle: u32, priority: i32) -> bool {
        self.handles.set_thread_priority(handle, priority)
    }

    pub fn set_thread_ce_priority_for_handle(
        &mut self,
        handle: u32,
        priority: i32,
        current_thread_id: u32,
    ) -> bool {
        if Self::is_current_thread_pseudo_handle(handle) {
            return self.set_thread_ce_priority_by_id(current_thread_id, priority);
        }
        self.set_thread_ce_priority(handle, priority)
    }

    fn set_thread_ce_priority_by_id(&mut self, thread_id: u32, priority: i32) -> bool {
        if let Some(success) = self.handles.set_thread_priority_by_id(thread_id, priority) {
            if success {
                self.thread_priority_overrides.remove(&thread_id);
            }
            return success;
        }
        if !(0..crate::ce::object::MAX_CE_PRIORITY_LEVELS).contains(&priority) {
            return false;
        }
        self.thread_priority_overrides.insert(thread_id, priority);
        true
    }

    pub fn set_thread_win32_priority(&mut self, handle: u32, priority: u32) -> bool {
        let Some(priority) = win32_thread_priority_to_ce(priority) else {
            return false;
        };
        self.handles.set_thread_priority(handle, priority)
    }

    pub fn set_thread_win32_priority_for_handle(
        &mut self,
        handle: u32,
        priority: u32,
        current_thread_id: u32,
    ) -> bool {
        let Some(priority) = win32_thread_priority_to_ce(priority) else {
            return false;
        };
        self.set_thread_ce_priority_for_handle(handle, priority, current_thread_id)
    }

    pub fn set_lowest_scheduled_priority(&mut self, max_priority: u32) -> u32 {
        let previous = self.lowest_scheduled_win32_priority;
        self.lowest_scheduled_win32_priority =
            max_priority.min(MAX_WIN32_PRIORITY_LEVELS.saturating_sub(1));
        previous
    }

    pub fn guest_thread_start(&self, handle: u32) -> Option<(u32, u32, u32)> {
        self.handles.thread_start(handle)
    }

    pub fn set_event(&mut self, handle: u32) -> bool {
        self.set_event_with_trace_detail(handle, None)
    }

    pub fn set_event_with_trace_detail(
        &mut self,
        handle: u32,
        trace_detail: Option<String>,
    ) -> bool {
        let success = self.handles.set_event(handle);
        let waiter_ids = self.scheduler.waiter_ids_for_handle(handle);
        let queued = if success {
            self.scheduler
                .queue_pending_wake_ids(waiter_ids.iter().copied())
        } else {
            0
        };
        let detail = append_trace_detail(
            format!(
                "{}/waiters={:?}/queued={}",
                self.describe_handle(handle),
                waiter_ids,
                queued
            ),
            trace_detail,
        );
        self.record_event_trace(EventTraceRecord {
            op: "SetEvent",
            handle: Some(handle),
            name: None,
            manual_reset: None,
            signaled: None,
            result: Some(success),
            detail: Some(detail),
        });
        success
    }

    pub fn reset_event(&mut self, handle: u32) -> bool {
        self.reset_event_with_trace_detail(handle, None)
    }

    pub fn reset_event_with_trace_detail(
        &mut self,
        handle: u32,
        trace_detail: Option<String>,
    ) -> bool {
        let success = self.handles.reset_event(handle);
        let detail = append_trace_detail(self.describe_handle(handle), trace_detail);
        self.record_event_trace(EventTraceRecord {
            op: "ResetEvent",
            handle: Some(handle),
            name: None,
            manual_reset: None,
            signaled: None,
            result: Some(success),
            detail: Some(detail),
        });
        success
    }

    pub fn get_event_data(&mut self, handle: u32) -> Option<u32> {
        let data = self.handles.get_event_data(handle);
        self.record_event_trace(EventTraceRecord {
            op: "GetEventData",
            handle: Some(handle),
            name: None,
            manual_reset: None,
            signaled: None,
            result: Some(data.is_some()),
            detail: Some(match data {
                Some(data) => format!("{}/data=0x{data:08x}", self.describe_handle(handle)),
                None => self.describe_handle(handle),
            }),
        });
        data
    }

    pub fn set_event_data(&mut self, handle: u32, data: u32) -> bool {
        let success = self.handles.set_event_data(handle, data);
        self.record_event_trace(EventTraceRecord {
            op: "SetEventData",
            handle: Some(handle),
            name: None,
            manual_reset: None,
            signaled: None,
            result: Some(success),
            detail: Some(format!(
                "{}/data=0x{data:08x}",
                self.describe_handle(handle)
            )),
        });
        success
    }

    pub fn pulse_event(&mut self, handle: u32) -> bool {
        self.pulse_event_with_trace_detail(handle, None)
    }

    pub fn pulse_event_with_trace_detail(
        &mut self,
        handle: u32,
        trace_detail: Option<String>,
    ) -> bool {
        let (manual_reset, wait_ids) = match self.handles.get(handle) {
            Ok(KernelObject::Event(event)) => {
                let mut wait_ids = self.scheduler.waiter_ids_for_handle(handle);
                if !event.manual_reset {
                    wait_ids.truncate(1);
                }
                (event.manual_reset, wait_ids)
            }
            _ => {
                let detail = append_trace_detail(self.describe_handle(handle), trace_detail);
                self.record_event_trace(EventTraceRecord {
                    op: "PulseEvent",
                    handle: Some(handle),
                    name: None,
                    manual_reset: None,
                    signaled: None,
                    result: Some(false),
                    detail: Some(detail),
                });
                return false;
            }
        };

        let success = self.handles.set_event(handle);
        if success {
            self.scheduler
                .queue_pending_wake_ids(wait_ids.iter().copied());
            for wait_id in wait_ids {
                self.pulsed_wait_handles.insert(wait_id, handle);
            }
            let _ = self.handles.reset_event(handle);
        }
        let detail = append_trace_detail(self.describe_handle(handle), trace_detail);
        self.record_event_trace(EventTraceRecord {
            op: "PulseEvent",
            handle: Some(handle),
            name: None,
            manual_reset: Some(manual_reset),
            signaled: Some(false),
            result: Some(success),
            detail: Some(detail),
        });
        success
    }

    pub fn create_mutex_w(
        &mut self,
        name: Option<String>,
        initial_owner_thread: Option<u32>,
    ) -> u32 {
        self.create_mutex_w_with_status(name, initial_owner_thread)
            .0
    }

    pub fn create_mutex_w_with_status(
        &mut self,
        name: Option<String>,
        initial_owner_thread: Option<u32>,
    ) -> (u32, bool) {
        let (handle, existed) = self
            .handles
            .create_mutex_with_status(name.clone(), initial_owner_thread);
        self.record_event_trace(EventTraceRecord {
            op: if existed {
                "CreateMutexW(existing)"
            } else {
                "CreateMutexW(new)"
            },
            handle: Some(handle),
            name,
            manual_reset: None,
            signaled: None,
            result: Some(!existed),
            detail: Some(format!(
                "pid={}/initial_owner={}/{}",
                self.current_process_id,
                initial_owner_thread
                    .map(|thread| thread.to_string())
                    .unwrap_or_else(|| "none".to_owned()),
                self.describe_handle(handle)
            )),
        });
        (handle, existed)
    }

    pub fn create_semaphore_w(
        &mut self,
        name: Option<String>,
        initial_count: i32,
        maximum_count: i32,
    ) -> Option<u32> {
        self.handles
            .create_semaphore(name, initial_count, maximum_count)
    }

    pub fn release_semaphore(&mut self, handle: u32, release_count: i32) -> Option<i32> {
        let previous = self.handles.release_semaphore(handle, release_count)?;
        self.queue_object_wake_candidates(handle);
        Some(previous)
    }

    pub fn release_mutex(&mut self, handle: u32, thread_id: u32) -> bool {
        let previous_lock_count = self.handles.mutex_lock_count(handle);
        let success = self.handles.release_mutex(handle, thread_id);
        if success && previous_lock_count == Some(1) {
            self.queue_object_wake_candidates(handle);
        }
        self.record_event_trace(EventTraceRecord {
            op: "ReleaseMutex",
            handle: Some(handle),
            name: None,
            manual_reset: None,
            signaled: None,
            result: Some(success),
            detail: Some(format!(
                "pid={}/thread={thread_id}/previous_locks={}/{}",
                self.current_process_id,
                previous_lock_count
                    .map(|count| count.to_string())
                    .unwrap_or_else(|| "invalid".to_owned()),
                self.describe_handle(handle)
            )),
        });
        success
    }

    pub fn is_mutex_handle(&self, handle: u32) -> bool {
        self.handles.is_mutex(handle)
    }

    fn wait_for_single_object_core(
        &mut self,
        handle: u32,
        timeout_ms: u32,
        thread_id: u32,
    ) -> WaitResult {
        if Self::is_current_thread_pseudo_handle(handle) {
            return if timeout_ms == 0 {
                WaitResult::Timeout
            } else {
                WaitResult::Timeout
            };
        }
        if Self::is_current_process_pseudo_handle(handle) {
            return if self.current_process_signaled {
                WaitResult::Object0
            } else {
                WaitResult::Timeout
            };
        }
        if let Ok(KernelObject::FileChangeNotification(notification)) = self.handles.get(handle)
            && self
                .ensure_file_change_notification_owner(notification, handle)
                .is_err()
        {
            return WaitResult::Failed;
        }
        if let Ok(KernelObject::MessageQueue(endpoint)) = self.handles.get(handle) {
            return if self.message_queue_wait_ready(endpoint).unwrap_or(false) {
                WaitResult::Object0
            } else {
                WaitResult::Timeout
            };
        }
        self.handles
            .wait_for_single_object(handle, timeout_ms, thread_id)
    }

    fn wait_for_any_object_core(&mut self, handles: &[u32], thread_id: u32) -> WaitMultipleResult {
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

        match self.wait_for_single_object_core(*handle, 0, thread_id) {
            WaitResult::Object0 => WaitMultipleResult::Object(index as u32),
            WaitResult::Timeout => WaitMultipleResult::Timeout,
            WaitResult::Failed => WaitMultipleResult::Failed,
        }
    }

    pub fn wait_for_single_object(&mut self, handle: u32, timeout_ms: u32, thread_id: u32) -> u32 {
        self.scheduler
            .record_wait_attempt(SchedulerWaitKind::Single, 1, timeout_ms);
        let result = match self.wait_for_single_object_core(handle, timeout_ms, thread_id) {
            WaitResult::Object0 => WAIT_OBJECT_0,
            WaitResult::Timeout => WAIT_TIMEOUT,
            WaitResult::Failed => WAIT_FAILED,
        };
        if result == WAIT_FAILED {
            self.threads
                .set_last_error(thread_id, crate::ce::thread::ERROR_INVALID_HANDLE);
        }
        let wait_trace = match self.handles.get(handle).ok() {
            Some(KernelObject::Process(process)) => Some(ProcessTraceRecord {
                op: "WaitForSingleObjectProcess",
                application: None,
                command_line: None,
                path: None,
                process_handle: Some(handle),
                thread_handle: None,
                process_id: Some(process.process_id()),
                thread_id: None,
                result: Some(result),
                error: self
                    .handles
                    .process_exit_code(handle)
                    .map(|code| format!("exit=0x{code:08x}")),
                detail: None,
            }),
            Some(KernelObject::Thread(_)) => {
                self.handles
                    .thread_snapshot_for_handle(handle)
                    .map(|thread| ProcessTraceRecord {
                        op: "WaitForSingleObjectThread",
                        application: None,
                        command_line: None,
                        path: None,
                        process_handle: None,
                        thread_handle: Some(handle),
                        process_id: None,
                        thread_id: Some(thread.thread_id),
                        result: Some(result),
                        error: Some(format!("exit=0x{:08x}", thread.exit_code)),
                        detail: None,
                    })
            }
            _ => None,
        };
        if let Some(record) = wait_trace {
            self.record_process_trace(record);
        }
        self.scheduler
            .record_wait_result(wait_result_to_wake_reason(result));
        result
    }

    pub fn record_blocked_single_wait(&mut self, timeout_ms: u32) {
        self.scheduler
            .record_wait_attempt(SchedulerWaitKind::Single, 1, timeout_ms);
        self.scheduler.record_blocked_wait();
    }

    pub fn record_blocked_multiple_wait(&mut self, handle_count: u32, timeout_ms: u32) {
        self.scheduler
            .record_wait_attempt(SchedulerWaitKind::Multiple, handle_count, timeout_ms);
        self.scheduler.record_blocked_wait();
    }

    pub fn record_blocked_msg_wait(&mut self, handle_count: u32, timeout_ms: u32) {
        self.scheduler
            .record_wait_attempt(SchedulerWaitKind::MsgWait, handle_count, timeout_ms);
        self.scheduler.record_blocked_wait();
    }

    pub fn record_blocked_thread_sleep(&mut self, timeout_ms: u32) {
        self.scheduler
            .record_wait_attempt(SchedulerWaitKind::Sleep, 0, timeout_ms);
        self.scheduler.record_blocked_wait();
    }

    pub fn record_thread_yield(&mut self) {
        self.scheduler.record_thread_yield();
    }

    pub fn record_resumed_single_wait(&mut self, result: u32) {
        self.scheduler
            .record_wait_wake(wait_result_to_wake_reason(result));
    }

    pub fn record_resumed_wait(&mut self, result: u32) {
        self.scheduler
            .record_wait_wake(wait_result_to_wake_reason(result));
    }

    pub fn record_msg_wait_result(&mut self, handle_count: u32, timeout_ms: u32, result: u32) {
        self.scheduler
            .record_wait_attempt(SchedulerWaitKind::MsgWait, handle_count, timeout_ms);
        self.scheduler
            .record_wait_result(wait_result_to_wake_reason(result));
    }

    pub fn record_msg_wait_input(&mut self, handle_count: u32, timeout_ms: u32) {
        self.scheduler
            .record_wait_attempt(SchedulerWaitKind::MsgWait, handle_count, timeout_ms);
        self.scheduler
            .record_wait_result(SchedulerWakeReason::MessageInput);
    }

    pub fn record_resumed_msg_wait_input(&mut self) {
        self.scheduler
            .record_wait_wake(SchedulerWakeReason::MessageInput);
    }

    pub fn record_resumed_msg_wait_result(&mut self, result: u32) {
        self.scheduler
            .record_wait_wake(wait_result_to_wake_reason(result));
    }

    pub fn record_resumed_thread_sleep(&mut self) {
        self.scheduler
            .record_wait_wake(SchedulerWakeReason::Timeout);
    }

    /// Legacy test/compatibility entry retained for integration-test crates.
    /// Production object waits use `prepare_task_wait`/`try_publish`; opaque
    /// native subscriptions use `try_register_opaque_waiter_for_birth`.
    pub fn register_blocked_waiter(
        &mut self,
        thread_id: u32,
        thread_handle: u32,
        wait_handles: Vec<u32>,
        kind: SchedulerBlockedWaitKind,
        wait_started_ms: u32,
        timeout_ms: u32,
    ) -> u64 {
        // Legacy test/compatibility entry. Production object waits use the
        // fallible prepare/publish transaction; opaque production subscriptions
        // use try_register_opaque_waiter_for_birth. A failed task pin must not
        // panic or publish an unpinned frame. Zero is never a scheduler wait ID.
        self.prepare_wait_with_policy(
            thread_id,
            thread_handle,
            wait_handles,
            kind,
            wait_started_ms,
            timeout_ms,
            false,
        )
        .map(PreparedTaskWait::publish)
        .unwrap_or(0)
    }

    /// Legacy fixture/compatibility subscription whose slots are opaque
    /// tokens, not CE waitable objects. A native producer must use the
    /// fallible exact-birth entry below, not this unknown-owner-capable path.
    pub fn register_opaque_compatibility_waiter(
        &mut self,
        thread_id: u32,
        thread_handle: u32,
        wait_handles: Vec<u32>,
        kind: SchedulerBlockedWaitKind,
        wait_started_ms: u32,
        timeout_ms: u32,
    ) -> u64 {
        let wait_keys = wait_handles
            .iter()
            .copied()
            .map(WaitObjectKey::Handle)
            .collect();
        let pinned_targets = vec![None; wait_handles.len()];
        match self.handles.thread_birth_key_by_id(thread_id) {
            Some(birth) => self
                .scheduler
                .register_blocked_wait_for_birth_with_keys_and_pins(
                    birth,
                    thread_handle,
                    wait_handles,
                    wait_keys,
                    pinned_targets,
                    kind,
                    wait_started_ms,
                    timeout_ms,
                ),
            None => self
                .scheduler
                .register_legacy_blocked_wait_with_keys_and_pins(
                    thread_id,
                    thread_handle,
                    wait_handles,
                    wait_keys,
                    pinned_targets,
                    kind,
                    wait_started_ms,
                    timeout_ms,
                ),
        }
    }

    /// Publish an opaque production subscription only for the exact captured
    /// live owner. Opaque slots are not CE waitable objects and acquire no
    /// target pins; the owner check is nevertheless mandatory at insertion.
    pub fn try_register_opaque_waiter_for_birth(
        &mut self,
        owner_birth: ThreadBirthKey,
        thread_handle: u32,
        wait_handles: Vec<u32>,
        kind: SchedulerBlockedWaitKind,
        wait_started_ms: u32,
        timeout_ms: u32,
    ) -> std::result::Result<u64, WaitPublishError> {
        let thread_id = owner_birth.thread_id();
        self.validate_live_wait_owner_birth(owner_birth, thread_id)?;
        let wait_keys = wait_handles
            .iter()
            .copied()
            .map(WaitObjectKey::Handle)
            .collect();
        let pinned_targets = vec![None; wait_handles.len()];
        Ok(self
            .scheduler
            .register_blocked_wait_for_birth_with_keys_and_pins(
                owner_birth,
                thread_handle,
                wait_handles,
                wait_keys,
                pinned_targets,
                kind,
                wait_started_ms,
                timeout_ms,
            ))
    }

    /// The first Task/Frame migration boundary: publish one finite Sleep for
    /// an exact live owner. The HandleTable owns its mutable phase; the
    /// scheduler record carries only an immutable locator. No target object
    /// pins are acquired by Sleep.
    pub fn try_publish_finite_sleep_wait_for_birth(
        &mut self,
        owner_birth: ThreadBirthKey,
        thread_handle: u32,
        wait_started_ms: u32,
        timeout_ms: u32,
    ) -> std::result::Result<TaskFrameRef, FiniteSleepPublishError> {
        self.validate_live_wait_owner_birth(owner_birth, owner_birth.thread_id())
            .map_err(FiniteSleepPublishError::Owner)?;
        let key = self
            .handles
            .preflight_finite_sleep_frame(owner_birth)
            .map_err(FiniteSleepPublishError::Frame)?;
        let frame = self
            .scheduler
            .try_register_finite_sleep_wait(key, thread_handle, wait_started_ms, timeout_ms)
            .map_err(FiniteSleepPublishError::Scheduler)?;
        if let Err(error) = self.handles.activate_finite_sleep_frame(frame) {
            // No guest-visible result or native payload has been published.
            // This cannot normally fail under the exclusive Kernel borrow,
            // but a failed invariant must not leave a selectable wait.
            let _ = self.remove_blocked_waiter(frame.wait_id());
            return Err(FiniteSleepPublishError::Frame(error));
        }
        Ok(frame)
    }

    pub fn is_blocked_finite_sleep_frame(&self, frame: TaskFrameRef) -> bool {
        self.handles.finite_sleep_frame_is_blocked(frame)
            && self.blocked_waiter(frame.wait_id()).is_some_and(|wait| {
                wait.frame_ref == Some(frame)
                    && wait.owner_birth == Some(frame.key().owner_birth())
                    && matches!(wait.kind, SchedulerBlockedWaitKind::Sleep)
            })
    }

    pub fn begin_finite_sleep_completion(&mut self, frame: TaskFrameRef) -> bool {
        self.is_blocked_finite_sleep_frame(frame)
            && self.handles.begin_finite_sleep_completion(frame).is_ok()
    }

    /// Call only after native register restoration and return PC are checked.
    /// The scheduler wait and its pins stay live until this exact completion.
    pub fn retire_finite_sleep_completion(&mut self, frame: TaskFrameRef) -> bool {
        if !self
            .blocked_waiter(frame.wait_id())
            .is_some_and(|wait| wait.frame_ref == Some(frame))
            || self.handles.retire_finite_sleep_frame(frame).is_err()
        {
            return false;
        }
        self.remove_blocked_waiter(frame.wait_id())
            .is_some_and(|wait| wait.frame_ref == Some(frame))
    }

    fn validate_live_wait_owner_birth(
        &self,
        owner_birth: ThreadBirthKey,
        thread_id: u32,
    ) -> std::result::Result<ThreadBirthKey, WaitPublishError> {
        let current = self.handles.thread_birth_key_by_id(thread_id);
        if owner_birth.thread_id() != thread_id || current != Some(owner_birth) {
            return Err(WaitPublishError::StaleOwnerBirth {
                expected: owner_birth,
                current,
            });
        }
        if !self.handles.thread_execution_live_by_id(thread_id) {
            return Err(WaitPublishError::ExitedOwnerBirth { owner: owner_birth });
        }
        Ok(owner_birth)
    }

    /// Prepares every ordered task reference before publishing the wait.
    /// Invalid object-wait slots, stale births and pin exhaustion leave no
    /// scheduler record and release only pins acquired in this attempt.
    pub fn try_register_blocked_waiter(
        &mut self,
        thread_id: u32,
        thread_handle: u32,
        wait_handles: Vec<u32>,
        kind: SchedulerBlockedWaitKind,
        wait_started_ms: u32,
        timeout_ms: u32,
    ) -> Option<u64> {
        self.prepare_task_wait(
            thread_id,
            thread_handle,
            wait_handles,
            kind,
            wait_started_ms,
            timeout_ms,
        )
        .ok()
        .map(PreparedTaskWait::publish)
    }

    /// Resolves and pins every ordered CE object-wait slot. The returned
    /// guard owns those references until it is published or dropped. A
    /// preparation error sets the waiting thread's CE LastError before the
    /// borrow-bearing result returns to the native caller.
    pub fn prepare_task_wait(
        &mut self,
        thread_id: u32,
        thread_handle: u32,
        wait_handles: Vec<u32>,
        kind: SchedulerBlockedWaitKind,
        wait_started_ms: u32,
        timeout_ms: u32,
    ) -> std::result::Result<PreparedTaskWait<'_>, WaitPrepError> {
        self.prepare_wait_with_policy(
            thread_id,
            thread_handle,
            wait_handles,
            kind,
            wait_started_ms,
            timeout_ms,
            true,
        )
    }

    fn prepare_wait_with_policy(
        &mut self,
        thread_id: u32,
        thread_handle: u32,
        wait_handles: Vec<u32>,
        kind: SchedulerBlockedWaitKind,
        wait_started_ms: u32,
        timeout_ms: u32,
        validate_raw_object_slots: bool,
    ) -> std::result::Result<PreparedTaskWait<'_>, WaitPrepError> {
        let owner_birth = self.handles.thread_birth_key_by_id(thread_id);
        let mut prepared = PreparedTaskWait {
            kernel: self,
            thread_id,
            owner_birth,
            thread_handle,
            wait_keys: Vec::with_capacity(wait_handles.len()),
            pinned_targets: Vec::with_capacity(wait_handles.len()),
            wait_handles,
            kind,
            wait_started_ms,
            timeout_ms,
            published: false,
        };
        for index in 0..prepared.wait_handles.len() {
            let handle = prepared.wait_handles[index];
            let key = prepared
                .kernel
                .wait_object_key_for_handle(handle, thread_id);
            // Pseudo handles retain their existing selector-based semantics;
            // only real public Thread/Process aliases acquire birth pins.
            let pin = if Self::is_current_thread_pseudo_handle(handle)
                || Self::is_current_process_pseudo_handle(handle)
            {
                None
            } else {
                match key {
                    WaitObjectKey::Thread(birth) => Some(TaskWaitPin::Thread(birth)),
                    WaitObjectKey::Process(birth) => Some(TaskWaitPin::Process(birth)),
                    WaitObjectKey::Handle(_) => None,
                }
            };
            if let Some(pin) = pin {
                if !prepared.kernel.handles.try_pin_wait_target(pin) {
                    let error =
                        if prepared.kernel.handles.wait_target_pin_count(pin) == Some(u32::MAX) {
                            WaitPrepError::PinExhausted { index, pin }
                        } else {
                            WaitPrepError::StaleTarget { index, pin }
                        };
                    prepared
                        .kernel
                        .threads
                        .set_last_error(thread_id, error.last_error());
                    return Err(error);
                }
            } else if validate_raw_object_slots
                && prepared.kernel.is_wait_ready(handle, thread_id).is_none()
            {
                let error = WaitPrepError::InvalidHandle { index, handle };
                prepared
                    .kernel
                    .threads
                    .set_last_error(thread_id, error.last_error());
                return Err(error);
            }
            prepared.wait_keys.push(key);
            prepared.pinned_targets.push(pin);
        }
        Ok(prepared)
    }

    pub fn remove_blocked_waiter(&mut self, wait_id: u64) -> Option<SchedulerBlockedWait> {
        let detail = self
            .scheduler
            .blocked_wait(wait_id)
            .map(|wait| {
                let handles = wait
                    .wait_handles
                    .iter()
                    .map(|handle| format!("0x{handle:08x}:{}", self.describe_handle(*handle)))
                    .collect::<Vec<_>>()
                    .join("|");
                format!(
                    "id={}/thread={}/thread_handle=0x{:08x}/kind={:?}/handles=[{}]",
                    wait.id, wait.thread_id, wait.thread_handle, wait.kind, handles
                )
            })
            .unwrap_or_else(|| format!("id={wait_id}/missing"));
        self.pulsed_wait_handles.remove(&wait_id);
        self.comm_event_mask_changed_waits.remove(&wait_id);
        let removed = self.scheduler.remove_blocked_wait(wait_id);
        if let Some(wait) = &removed {
            if let Some(frame) = wait.frame_ref {
                // Cancellation, exact-birth exit and failed publication all
                // retire only the matching canonical frame. A successful
                // completion already retired it before removing the index.
                if self
                    .handles
                    .active_finite_sleep_frame(frame.key().owner_birth())
                    == Some(frame)
                {
                    if let Err(error) = self.handles.cancel_finite_sleep_frame(frame) {
                        tracing::error!(target: "ce.scheduler", ?frame, ?error, "finite Sleep frame cancellation failed");
                    }
                }
            }
            for pin in wait.pinned_targets.iter().flatten() {
                assert!(self.handles.release_wait_target(*pin));
            }
        }
        self.record_wait_event_trace("RemoveBlockedWaiter", None, Some(removed.is_some()), detail);
        removed
    }

    /// Explicit cleanup for legacy unknown-owner compatibility waits only.
    /// Canonical task exit uses the exact birth bucket instead.
    pub fn remove_blocked_waiters_for_thread(&mut self, thread_id: u32) -> usize {
        let wait_ids = self.scheduler.waiter_ids_for_thread(thread_id);
        for wait_id in &wait_ids {
            let _ = self.remove_blocked_waiter(*wait_id);
        }
        wait_ids.len()
    }

    pub(crate) fn take_modal_dialog_result(
        &mut self,
        thread_id: u32,
        dialog_hwnd: u32,
    ) -> Option<u32> {
        self.modal_dialog_results.remove(&(thread_id, dialog_hwnd))
    }

    pub(crate) fn thread_has_modal_dialog_result(&self, thread_id: u32) -> bool {
        self.modal_dialog_results
            .keys()
            .any(|(result_thread_id, _)| *result_thread_id == thread_id)
    }

    pub(crate) fn has_modal_dialog_result(&self, thread_id: u32, dialog_hwnd: u32) -> bool {
        self.modal_dialog_results
            .contains_key(&(thread_id, dialog_hwnd))
    }

    pub fn blocked_waiter(&self, wait_id: u64) -> Option<&SchedulerBlockedWait> {
        self.scheduler.blocked_wait(wait_id)
    }

    pub fn blocked_waiters(&self) -> impl Iterator<Item = &SchedulerBlockedWait> {
        self.scheduler.blocked_waits()
    }

    pub fn describe_handle(&self, handle: u32) -> String {
        if Self::is_current_thread_pseudo_handle(handle) {
            "current_thread_pseudo".to_owned()
        } else if Self::is_current_process_pseudo_handle(handle) {
            format!(
                "current_process_pseudo(signaled={},exit=0x{:08x})",
                self.current_process_signaled, self.current_process_exit_code
            )
        } else {
            self.handles.describe_handle(handle)
        }
    }

    pub fn pulsed_wait_handle(&self, wait_id: u64) -> Option<u32> {
        self.pulsed_wait_handles.get(&wait_id).copied()
    }

    pub fn comm_event_mask_changed_wait(&self, wait_id: u64) -> bool {
        self.comm_event_mask_changed_waits.contains(&wait_id)
    }

    pub fn take_comm_event_mask_changed_wait(&mut self, wait_id: u64) -> bool {
        self.comm_event_mask_changed_waits.remove(&wait_id)
    }

    fn wait_object_key_for_handle(&self, handle: u32, thread_id: u32) -> WaitObjectKey {
        if Self::is_current_thread_pseudo_handle(handle)
            && let Some(key) = self.handles.thread_birth_key_by_id(thread_id)
        {
            return WaitObjectKey::Thread(key);
        }
        if Self::is_current_process_pseudo_handle(handle)
            && let Some(key) = self.handles.process_birth_key(self.current_process_id)
        {
            return WaitObjectKey::Process(key);
        }
        if let Some(key) = self.handles.thread_birth_key_for_handle(handle) {
            return WaitObjectKey::Thread(key);
        }
        if let Some(key) = self.handles.process_birth_key_for_handle(handle) {
            return WaitObjectKey::Process(key);
        }
        WaitObjectKey::Handle(handle)
    }

    fn queue_object_wake_candidates_for_key(&mut self, key: WaitObjectKey) -> usize {
        let wait_ids = self.scheduler.waiter_ids_for_key(key);
        self.scheduler.queue_pending_wake_ids(wait_ids)
    }

    fn queue_object_wake_candidates(&mut self, handle: u32) -> usize {
        let key = self.wait_object_key_for_handle(handle, 0);
        self.queue_object_wake_candidates_for_key(key)
    }

    pub fn queue_serial_read_wake_candidates(&mut self, handle: u32) -> usize {
        self.scheduler.queue_serial_read_wake_candidates(handle)
    }

    pub fn queue_serial_event_wake_candidates(&mut self, handle: u32) -> usize {
        self.scheduler.queue_serial_event_wake_candidates(handle)
    }

    pub fn queue_all_serial_read_wake_candidates(&mut self) -> usize {
        self.scheduler.queue_all_serial_read_wake_candidates()
    }

    pub fn queue_all_serial_event_wake_candidates(&mut self) -> usize {
        self.scheduler.queue_all_serial_event_wake_candidates()
    }

    pub fn queue_message_wake_candidates(&mut self, thread_id: u32) -> usize {
        self.scheduler.queue_message_wake_candidates(thread_id)
    }

    fn queue_paint_wake_candidates(&mut self) -> usize {
        let thread_ids = self
            .gwe
            .windows_snapshot()
            .into_iter()
            .filter(|window| {
                !window.destroyed
                    && window.update_pending
                    && self.gwe.is_window_visible(window.hwnd)
            })
            .map(|window| window.thread_id)
            .collect::<BTreeSet<_>>();
        thread_ids
            .into_iter()
            .map(|thread_id| self.queue_message_wake_candidates(thread_id))
            .sum()
    }

    pub fn queue_send_reply_wake_candidates(&mut self, send_id: u64) -> usize {
        self.scheduler.queue_send_reply_wake_candidates(send_id)
    }

    pub fn queue_winsock_wake_candidates(&mut self, socket: u32) -> usize {
        let wait_ids = self.scheduler.waiter_ids_for_handle(socket);
        self.scheduler.queue_pending_wake_ids(wait_ids)
    }

    pub fn queue_winsock_wake_candidates_for_handles(
        &mut self,
        sockets: impl IntoIterator<Item = u32>,
    ) -> usize {
        let wait_ids = sockets
            .into_iter()
            .flat_map(|socket| self.scheduler.waiter_ids_for_handle(socket))
            .collect::<Vec<_>>();
        self.scheduler.queue_pending_wake_ids(wait_ids)
    }

    pub fn sent_message_result_ready(&self, send_id: u64) -> bool {
        self.gwe.sent_message_result_ready(send_id)
    }

    pub fn thread_has_pending_sent_message(&self, thread_id: u32) -> bool {
        match self.live_thread_birth_key_by_id(thread_id) {
            Some(birth) => self.gwe.has_pending_sent_message_for_birth(birth),
            None => self.gwe.has_pending_sent_message_for_thread(thread_id),
        }
    }

    pub fn select_ready_blocked_waiter(
        &self,
        active_thread_id: u32,
        now_ms: u32,
        is_ready: impl FnMut(&SchedulerBlockedWait, &Self) -> bool,
    ) -> Option<u64> {
        self.select_ready_blocked_waiter_with_eligibility(
            active_thread_id,
            now_ms,
            |_, _| true,
            is_ready,
        )
    }

    /// Apply a caller's ownership gate before either readiness or timeout
    /// selects a wait. A readiness-only filter cannot exclude expired waits.
    pub fn select_ready_blocked_waiter_with_eligibility(
        &self,
        active_thread_id: u32,
        now_ms: u32,
        mut is_eligible: impl FnMut(&SchedulerBlockedWait, &Self) -> bool,
        mut is_ready: impl FnMut(&SchedulerBlockedWait, &Self) -> bool,
    ) -> Option<u64> {
        self.scheduler
            .select_ready_blocked_wait_id_with_eligibility(
                active_thread_id,
                now_ms,
                |wait| {
                    let owner_eligible = if let Some(frame) = wait.frame_ref {
                        // Eligibility precedes readiness and timeout. A stale
                        // generation can never become runnable merely because
                        // its old numeric deadline expired.
                        self.is_blocked_finite_sleep_frame(frame)
                    } else if matches!(
                        wait.kind,
                        SchedulerBlockedWaitKind::GetMessage { .. }
                            | SchedulerBlockedWaitKind::MsgWait { .. }
                            | SchedulerBlockedWaitKind::ModalMessageBox { .. }
                            | SchedulerBlockedWaitKind::PopupMenuModal { .. }
                    ) {
                        match wait.owner_birth {
                            Some(birth) => self.is_live_thread_birth(birth),
                            // An explicitly unknown fixture cannot borrow a
                            // canonical replacement birth via its numeric TID.
                            None => self
                                .handles
                                .thread_birth_key_by_id(wait.thread_id)
                                .is_none(),
                        }
                    } else {
                        true
                    };
                    owner_eligible && is_eligible(wait, self)
                },
                |wait| is_ready(wait, self),
                |thread_id| self.thread_priority_by_id(thread_id),
            )
    }

    pub fn record_multiple_wait_attempt(
        &mut self,
        handle_count: u32,
        timeout_ms: u32,
        result: u32,
    ) {
        self.scheduler
            .record_wait_attempt(SchedulerWaitKind::Multiple, handle_count, timeout_ms);
        self.scheduler
            .record_wait_result(wait_result_to_wake_reason(result));
    }

    pub fn wait_for_single_object_without_scheduler_record(
        &mut self,
        handle: u32,
        timeout_ms: u32,
        thread_id: u32,
    ) -> u32 {
        match self.wait_for_single_object_core(handle, timeout_ms, thread_id) {
            WaitResult::Object0 => WAIT_OBJECT_0,
            WaitResult::Timeout => WAIT_TIMEOUT,
            WaitResult::Failed => {
                self.threads
                    .set_last_error(thread_id, crate::ce::thread::ERROR_INVALID_HANDLE);
                WAIT_FAILED
            }
        }
    }

    pub fn is_wait_ready(&self, handle: u32, thread_id: u32) -> Option<bool> {
        if Self::is_current_thread_pseudo_handle(handle) {
            return Some(false);
        }
        if Self::is_current_process_pseudo_handle(handle) {
            return Some(self.current_process_signaled);
        }
        if let Ok(KernelObject::FileChangeNotification(notification)) = self.handles.get(handle)
            && self
                .ensure_file_change_notification_owner(notification, handle)
                .is_err()
        {
            return None;
        }
        if let Ok(KernelObject::MessageQueue(endpoint)) = self.handles.get(handle) {
            return self.message_queue_wait_ready(endpoint);
        }
        self.handles.is_wait_ready(handle, thread_id)
    }

    pub fn wait_for_multiple_objects(
        &mut self,
        handles: &[u32],
        wait_all: bool,
        timeout_ms: u32,
        thread_id: u32,
    ) -> u32 {
        let result = if handles.is_empty() || wait_all || handles.len() > MAXIMUM_WAIT_OBJECTS {
            self.threads
                .set_last_error(thread_id, crate::ce::thread::ERROR_INVALID_PARAMETER);
            WAIT_FAILED
        } else {
            match self.wait_for_any_object_core(handles, thread_id) {
                WaitMultipleResult::Object(index) => WAIT_OBJECT_0 + index,
                WaitMultipleResult::Timeout => WAIT_TIMEOUT,
                WaitMultipleResult::Failed => {
                    self.threads
                        .set_last_error(thread_id, crate::ce::thread::ERROR_INVALID_HANDLE);
                    WAIT_FAILED
                }
            }
        };
        self.record_multiple_wait_attempt(handles.len() as u32, timeout_ms, result);
        result
    }

    pub fn wait_for_multiple_objects_without_scheduler_record(
        &mut self,
        handles: &[u32],
        wait_all: bool,
        thread_id: u32,
    ) -> u32 {
        if handles.is_empty() || wait_all || handles.len() > MAXIMUM_WAIT_OBJECTS {
            self.threads
                .set_last_error(thread_id, crate::ce::thread::ERROR_INVALID_PARAMETER);
            return WAIT_FAILED;
        }
        match self.wait_for_any_object_core(handles, thread_id) {
            WaitMultipleResult::Object(index) => WAIT_OBJECT_0 + index,
            WaitMultipleResult::Timeout => WAIT_TIMEOUT,
            WaitMultipleResult::Failed => {
                self.threads
                    .set_last_error(thread_id, crate::ce::thread::ERROR_INVALID_HANDLE);
                WAIT_FAILED
            }
        }
    }

    fn registered_wait_slot_ready(
        &self,
        handle: u32,
        pin: Option<TaskWaitPin>,
        thread_id: u32,
    ) -> Option<bool> {
        match pin {
            Some(pin) => self.handles.wait_target_signaled(pin),
            None => self.is_wait_ready(handle, thread_id),
        }
    }

    /// Readiness for an already-published wait uses its pinned task births,
    /// not public aliases which may have closed since registration.
    pub fn registered_wait_has_ready_handle(&self, wait_id: u64) -> bool {
        let Some(wait) = self.scheduler.blocked_wait(wait_id) else {
            return false;
        };
        wait.wait_handles
            .iter()
            .zip(&wait.pinned_targets)
            .any(|(&handle, &pin)| {
                self.registered_wait_slot_ready(handle, pin, wait.thread_id) == Some(true)
            })
    }

    /// Acquires the first ready slot of a registered WaitAny frame. Canonical
    /// Thread/Process targets have no consumable signal; raw compatibility
    /// objects still go through their existing acquisition path.
    pub fn wait_for_registered_wait_without_scheduler_record(
        &mut self,
        wait_id: u64,
        thread_id: u32,
    ) -> u32 {
        let Some(wait) = self.scheduler.blocked_wait(wait_id) else {
            self.threads.set_last_error(thread_id, ERROR_INVALID_HANDLE);
            return WAIT_FAILED;
        };
        if wait.thread_id != thread_id {
            self.threads.set_last_error(thread_id, ERROR_INVALID_HANDLE);
            return WAIT_FAILED;
        }
        if wait.wait_handles.is_empty() || wait.wait_handles.len() > MAXIMUM_WAIT_OBJECTS {
            self.threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            return WAIT_FAILED;
        }
        let handles = wait.wait_handles.clone();
        let pins = wait.pinned_targets.clone();
        let Some(ready) = handles
            .iter()
            .zip(&pins)
            .map(|(&handle, &pin)| self.registered_wait_slot_ready(handle, pin, thread_id))
            .collect::<Option<Vec<_>>>()
        else {
            self.threads.set_last_error(thread_id, ERROR_INVALID_HANDLE);
            return WAIT_FAILED;
        };
        let Some(index) = ready.iter().position(|is_ready| *is_ready) else {
            return WAIT_TIMEOUT;
        };
        if pins[index].is_some() {
            return WAIT_OBJECT_0 + index as u32;
        }
        match self.wait_for_single_object_core(handles[index], 0, thread_id) {
            WaitResult::Object0 => WAIT_OBJECT_0 + index as u32,
            WaitResult::Timeout => WAIT_TIMEOUT,
            WaitResult::Failed => {
                self.threads.set_last_error(thread_id, ERROR_INVALID_HANDLE);
                WAIT_FAILED
            }
        }
    }

    pub fn create_window_ex_w(
        &mut self,
        thread_id: u32,
        class_name: &str,
        title: &str,
        parent: Option<u32>,
        id: u32,
        style: u32,
        ex_style: u32,
    ) -> u32 {
        self.create_window_ex_w_with_rect(
            thread_id,
            class_name,
            title,
            parent,
            id,
            style,
            ex_style,
            Rect::default(),
        )
    }

    pub fn create_window_ex_w_with_rect(
        &mut self,
        thread_id: u32,
        class_name: &str,
        title: &str,
        parent: Option<u32>,
        id: u32,
        style: u32,
        ex_style: u32,
        rect: Rect,
    ) -> u32 {
        self.create_window_ex_w_with_parent_owner_and_rect(
            thread_id, class_name, title, parent, None, id, style, ex_style, rect,
        )
    }

    pub fn create_window_ex_w_with_parent_owner_and_rect(
        &mut self,
        thread_id: u32,
        class_name: &str,
        title: &str,
        parent: Option<u32>,
        owner: Option<u32>,
        id: u32,
        style: u32,
        ex_style: u32,
        rect: Rect,
    ) -> u32 {
        let delivery_owner = WindowDeliveryOwner::LegacyUnknown {
            process: self.current_process_birth_key(),
        };
        self.create_window_ex_w_with_delivery_owner_and_rect(
            thread_id,
            class_name,
            title,
            parent,
            owner,
            id,
            style,
            ex_style,
            rect,
            delivery_owner,
        )
    }

    /// A native caller's captured birth, not a later numeric TID lookup,
    /// authorizes a real guest HWND and freezes its creation process.
    pub fn checked_window_delivery_owner_for_birth(
        &self,
        birth: ThreadBirthKey,
    ) -> Option<WindowDeliveryOwner> {
        if !self.is_live_thread_birth(birth) {
            return None;
        }
        let process = self
            .handles
            .live_thread_creation_owner_by_id(birth.thread_id())?;
        if !self.handles.process_birth_can_create_threads(process) {
            return None;
        }
        (self.current_process_birth_key() == Some(process)).then_some(WindowDeliveryOwner::Birth {
            thread: birth,
            process,
        })
    }

    pub fn create_window_ex_w_with_parent_owner_and_rect_for_birth(
        &mut self,
        birth: ThreadBirthKey,
        class_name: &str,
        title: &str,
        parent: Option<u32>,
        owner: Option<u32>,
        id: u32,
        style: u32,
        ex_style: u32,
        rect: Rect,
    ) -> Option<u32> {
        let delivery_owner = self.checked_window_delivery_owner_for_birth(birth)?;
        Some(self.create_window_ex_w_with_delivery_owner_and_rect(
            birth.thread_id(),
            class_name,
            title,
            parent,
            owner,
            id,
            style,
            ex_style,
            rect,
            delivery_owner,
        ))
    }

    fn create_window_ex_w_with_delivery_owner_and_rect(
        &mut self,
        thread_id: u32,
        class_name: &str,
        title: &str,
        parent: Option<u32>,
        owner: Option<u32>,
        id: u32,
        style: u32,
        ex_style: u32,
        rect: Rect,
        delivery_owner: WindowDeliveryOwner,
    ) -> u32 {
        let hwnd = self.gwe.create_window_ex_with_delivery_owner_and_rect(
            thread_id,
            self.current_process_id,
            class_name,
            title,
            parent,
            owner,
            id,
            style,
            ex_style,
            rect,
            delivery_owner,
        );
        self.handles.insert(KernelObject::Window(hwnd));
        self.record_window_lifecycle_trace(
            "create_window",
            thread_id,
            Some(hwnd),
            Some(hwnd),
            Some(format!(
                "requested_class={class_name}/requested_title={title}/id=0x{id:08x}"
            )),
        );
        hwnd
    }

    pub fn capture_window_backing_store(
        &mut self,
        hwnd: u32,
        framebuffer: &dyn Framebuffer,
    ) -> bool {
        if self.window_backing_stores.contains_key(&hwnd) {
            return true;
        }
        let Some(window) = self.gwe.window(hwnd) else {
            return false;
        };
        if window.destroyed {
            return false;
        }
        let Some(rect) = framebuffer_rect_from_gwe_rect(framebuffer.info(), window.rect) else {
            return false;
        };
        let Some(backing_store) = FramebufferBackingStore::capture(framebuffer, rect) else {
            return false;
        };
        self.window_backing_stores.insert(hwnd, backing_store);
        true
    }

    pub fn restore_window_backing_stores(
        &mut self,
        hwnds: &[u32],
        framebuffer: &mut dyn Framebuffer,
    ) -> usize {
        let mut restored = 0usize;
        for hwnd in hwnds.iter().rev().copied() {
            if self
                .window_backing_stores
                .get(&hwnd)
                .is_some_and(|backing_store| backing_store.restore(framebuffer))
            {
                self.window_backing_stores.remove(&hwnd);
                restored = restored.saturating_add(1);
            }
        }
        restored
    }

    pub fn discard_window_backing_stores(&mut self, hwnds: &[u32]) {
        for hwnd in hwnds {
            self.window_backing_stores.remove(hwnd);
        }
    }

    pub fn window_destroy_targets(&self, hwnd: u32) -> Vec<u32> {
        self.gwe.window_and_descendants(hwnd).unwrap_or_default()
    }

    pub fn show_window(&mut self, hwnd: u32, visible: bool) -> bool {
        self.show_window_with_activation(hwnd, visible, visible)
    }

    pub fn show_window_with_framebuffer(
        &mut self,
        hwnd: u32,
        visible: bool,
        activate: bool,
        framebuffer: Option<&mut dyn Framebuffer>,
    ) -> bool {
        let backing_store_targets = (!visible && self.gwe.is_window_visible(hwnd))
            .then(|| self.window_destroy_targets(hwnd))
            .unwrap_or_default();
        let previous = self.show_window_with_activation(hwnd, visible, activate);
        if previous && !visible && !backing_store_targets.is_empty() {
            if let Some(framebuffer) = framebuffer {
                let _ = self.restore_window_backing_stores(&backing_store_targets, framebuffer);
            } else {
                self.discard_window_backing_stores(&backing_store_targets);
            }
        }
        previous
    }

    pub fn show_window_with_activation(
        &mut self,
        hwnd: u32,
        visible: bool,
        activate: bool,
    ) -> bool {
        if !self.gwe.is_window(hwnd) {
            return false;
        }
        let before = self.gwe.get_window_rect(hwnd);
        let was_direct_visible = self.direct_window_visible(hwnd);
        let was_effective_visible = self.gwe.is_window_visible(hwnd);
        let previous = self.gwe.show_window(hwnd, visible);
        let is_direct_visible = self.direct_window_visible(hwnd);
        let is_effective_visible = self.gwe.is_window_visible(hwnd);
        if was_direct_visible != is_direct_visible || was_effective_visible != is_effective_visible
        {
            let thread_id = self
                .gwe
                .window(hwnd)
                .map(|window| window.thread_id)
                .unwrap_or(0);
            self.record_window_lifecycle_trace(
                "show_window",
                thread_id,
                Some(hwnd),
                Some(u32::from(previous)),
                Some(format!(
                    "requested_visible={visible}/activate={activate}/was_direct={was_direct_visible}/is_direct={is_direct_visible}/was_effective={was_effective_visible}/is_effective={is_effective_visible}"
                )),
            );
        }
        if was_direct_visible != is_direct_visible {
            self.post_window_message(hwnd, WM_SHOWWINDOW, u32::from(is_direct_visible), 0);
            let mut flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER;
            if is_direct_visible {
                flags |= SWP_SHOWWINDOW;
                if !activate {
                    flags |= SWP_NOACTIVATE;
                }
            } else {
                flags |= SWP_HIDEWINDOW | SWP_NOACTIVATE;
            }
            self.post_window_rect_messages(
                hwnd,
                before,
                self.gwe.get_window_rect(hwnd),
                HWND_TOP,
                flags,
                true,
            );
            if is_direct_visible {
                self.post_pending_size_move(hwnd);
            }
        }
        if visible && activate {
            let target = self.top_level_window(hwnd);
            let _ = self.activate_window(Some(target));
        } else if !visible {
            self.clear_focus_and_activation_within(hwnd);
        }
        previous
    }

    pub fn update_window(&mut self, hwnd: u32) -> bool {
        if !self.gwe.is_window(hwnd) {
            return false;
        }
        if let Some(update) = self.gwe.update_rect(hwnd) {
            if !self.gwe.is_window_visible(hwnd) {
                return true;
            }
            if update.erase {
                let hdc = 0x0200_0000 | (hwnd & 0x00ff_ffff);
                if !self.erase_window_background(hwnd, hdc) {
                    return false;
                }
            }
            let _ = self.send_message_w(hwnd, crate::ce::gwe::WM_PAINT, 0, 0);
        }
        true
    }

    pub fn erase_window_background(&mut self, hwnd: u32, hdc: u32) -> bool {
        if !self.gwe.is_window(hwnd) {
            return false;
        }
        let Some(result) = self.send_message_w(hwnd, crate::ce::gwe::WM_ERASEBKGND, hdc, 0) else {
            return false;
        };
        if result != 0 {
            self.gwe.clear_update_erase(hwnd)
        } else {
            true
        }
    }

    pub fn set_window_pos(
        &mut self,
        hwnd: u32,
        insert_after: Option<u32>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        flags: u32,
    ) -> bool {
        let before = self.gwe.get_window_rect(hwnd);
        let was_visible = self.direct_window_visible(hwnd);
        let moved = self
            .gwe
            .set_window_pos(hwnd, insert_after, x, y, width, height, flags);
        if moved {
            let after = self.gwe.get_window_rect(hwnd);
            let is_visible = self.direct_window_visible(hwnd);
            if was_visible != is_visible
                || flags & (SWP_SHOWWINDOW | SWP_HIDEWINDOW) != 0
                || flags & SWP_NOZORDER == 0
            {
                self.record_window_lifecycle_trace(
                    "set_window_pos",
                    self.gwe.window(hwnd).map(|window| window.thread_id).unwrap_or(0),
                    Some(hwnd),
                    Some(u32::from(moved)),
                    Some(format!(
                        "insert_after=0x{:08x}/flags=0x{flags:08x}/was_visible={was_visible}/is_visible={is_visible}/before={}/after={}",
                        insert_after.unwrap_or(HWND_TOP),
                        before
                            .map(|rect| {
                                format!("{},{}-{},{}", rect.left, rect.top, rect.right, rect.bottom)
                            })
                            .unwrap_or_else(|| "<none>".to_owned()),
                        after
                            .map(|rect| format!(
                                "{},{}-{},{}",
                                rect.left, rect.top, rect.right, rect.bottom
                            ))
                            .unwrap_or_else(|| "<none>".to_owned())
                    )),
                );
            }
            self.post_window_visibility_message(hwnd, was_visible, is_visible);
            self.post_window_rect_messages(
                hwnd,
                before,
                after,
                insert_after.unwrap_or(HWND_TOP),
                flags,
                flags & (SWP_SHOWWINDOW | SWP_HIDEWINDOW) != 0 || flags & SWP_NOZORDER == 0,
            );
            if flags & SWP_SHOWWINDOW != 0 && self.direct_window_visible(hwnd) {
                self.post_pending_size_move(hwnd);
            }
            if flags & (SWP_NOACTIVATE | SWP_HIDEWINDOW) == 0 {
                let target = self.top_level_window(hwnd);
                let _ = self.activate_window(Some(target));
            } else if flags & SWP_HIDEWINDOW != 0 {
                self.clear_focus_and_activation_within(hwnd);
            }
        }
        moved
    }

    pub fn enable_window(&mut self, hwnd: u32, enabled: bool) -> Option<bool> {
        let (was_enabled, changed) = self.set_window_enabled_state(hwnd, enabled)?;
        if changed {
            if !enabled {
                self.post_window_message(hwnd, WM_CANCELMODE, 0, 0);
            }
            self.post_window_message(hwnd, WM_ENABLE, u32::from(enabled), 0);
            if !enabled {
                self.clear_focus_and_activation_within(hwnd);
            }
        }
        Some(was_enabled)
    }

    pub fn set_window_enabled_state(&mut self, hwnd: u32, enabled: bool) -> Option<(bool, bool)> {
        if !self.gwe.is_window(hwnd) {
            return None;
        }
        let was_enabled = self.gwe.enable_window(hwnd, enabled);
        let changed = was_enabled != enabled;
        Some((was_enabled, changed))
    }

    pub fn set_parent(&mut self, hwnd: u32, parent: Option<u32>) -> Option<Option<u32>> {
        let previous = self.gwe.set_parent(hwnd, parent)?;
        if !self.gwe.is_window_visible(hwnd) || !self.gwe.is_window_enabled(hwnd) {
            self.clear_focus_and_activation_within(hwnd);
        }
        Some(previous)
    }

    pub fn bring_window_to_top(&mut self, hwnd: u32) -> bool {
        let moved =
            self.gwe
                .set_window_pos(hwnd, Some(HWND_TOP), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        if moved {
            let target = self.top_level_window(hwnd);
            let _ = self.activate_window(Some(target));
        }
        moved
    }

    pub fn set_window_pos_with_framebuffer(
        &mut self,
        hwnd: u32,
        insert_after: Option<u32>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        flags: u32,
        framebuffer: Option<&mut dyn Framebuffer>,
    ) -> bool {
        if flags & SWP_SHOWWINDOW != 0 && !self.gwe.is_window_visible(hwnd) {
            if let Some(framebuffer) = framebuffer.as_deref() {
                let _ = self.capture_window_backing_store(hwnd, framebuffer);
            }
        }
        let backing_store_targets = (flags & SWP_HIDEWINDOW != 0
            && self.gwe.is_window_visible(hwnd))
        .then(|| self.window_destroy_targets(hwnd))
        .unwrap_or_default();
        let moved = self.set_window_pos(hwnd, insert_after, x, y, width, height, flags);
        if moved && flags & SWP_HIDEWINDOW != 0 && !backing_store_targets.is_empty() {
            if let Some(framebuffer) = framebuffer {
                let _ = self.restore_window_backing_stores(&backing_store_targets, framebuffer);
            } else {
                self.discard_window_backing_stores(&backing_store_targets);
            }
        }
        moved
    }

    pub fn move_window(
        &mut self,
        hwnd: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        repaint: bool,
    ) -> bool {
        let before = self.gwe.get_window_rect(hwnd);
        let moved = self.gwe.move_window(hwnd, x, y, width, height, repaint);
        if moved {
            self.post_window_rect_messages(
                hwnd,
                before,
                self.gwe.get_window_rect(hwnd),
                0,
                0,
                true,
            );
        }
        moved
    }

    pub fn set_focus(&mut self, hwnd: Option<u32>) -> Option<u32> {
        if hwnd.is_some_and(|hwnd| !self.gwe.is_window(hwnd) || !self.gwe.is_window_enabled(hwnd)) {
            return None;
        }
        let previous = self.gwe.set_focus(hwnd);
        if previous != hwnd {
            // Focus edge: reset caret blink phase when caret's owner loses/gains focus.
            if let Some(caret) = self.gwe.caret() {
                let caret_hwnd = caret.hwnd;
                if previous == Some(caret_hwnd) {
                    self.gwe.reset_caret_on_focus_lost();
                } else if hwnd == Some(caret_hwnd) {
                    self.gwe.reset_caret_on_focus_gained();
                }
            }
            if let Some(previous_hwnd) = previous {
                self.post_window_message(previous_hwnd, WM_KILLFOCUS, hwnd.unwrap_or(0), 0);
            }
            if let Some(hwnd) = hwnd {
                let _ = self.activate_window(Some(hwnd));
                self.post_window_message(hwnd, WM_SETFOCUS, previous.unwrap_or(0), 0);
            }
        }
        previous
    }

    pub fn activate_window(&mut self, hwnd: Option<u32>) -> Option<u32> {
        if hwnd.is_some_and(|hwnd| !self.gwe.is_window(hwnd) || !self.gwe.is_window_enabled(hwnd)) {
            return None;
        }
        let previous = self.gwe.set_active_window(hwnd);
        if previous != hwnd {
            if let Some(previous_hwnd) = previous {
                self.post_window_message(
                    previous_hwnd,
                    WM_ACTIVATE,
                    WA_INACTIVE,
                    hwnd.unwrap_or(0),
                );
            }
            if let Some(hwnd) = hwnd {
                self.post_window_message(hwnd, WM_ACTIVATE, WA_ACTIVE, previous.unwrap_or(0));
            }
        }
        previous
    }

    pub fn set_capture(&mut self, hwnd: u32) -> Option<u32> {
        let previous = self.gwe.set_capture(hwnd);
        if previous != Some(hwnd) {
            if let Some(old_hwnd) = previous {
                self.post_window_message(old_hwnd, WM_CAPTURECHANGED, 0, hwnd);
            }
        }
        previous
    }

    pub fn release_capture(&mut self) -> bool {
        let previous = self.gwe.get_capture();
        self.gwe.release_capture();
        if let Some(old_hwnd) = previous {
            self.post_window_message(old_hwnd, WM_CAPTURECHANGED, 0, 0);
        }
        true
    }

    pub(crate) fn clear_focus_and_activation_within(&mut self, hwnd: u32) {
        if self.gwe.focus_is_within(hwnd) {
            let _ = self.set_focus(None);
        }
        if self.gwe.active_window_is_within(hwnd) {
            let _ = self.activate_window(None);
        }
        if self.gwe.keyboard_target_is_within(hwnd) {
            self.gwe.clear_keyboard_targets_within(hwnd);
        }
    }

    pub(crate) fn clear_destroyed_window_focus_and_activation(&mut self, hwnd: u32) {
        if self.gwe.focus_is_within(hwnd) {
            let _ = self.set_focus(None);
        }
        if self.gwe.active_window_is_within(hwnd) {
            let _ = self.gwe.set_active_window(None);
        }
        if self.gwe.keyboard_target_is_within(hwnd) {
            self.gwe.clear_keyboard_targets_within(hwnd);
        }
    }

    fn top_level_window(&self, hwnd: u32) -> u32 {
        let mut current = hwnd;
        while let Some(parent) = self.gwe.get_parent(current) {
            current = parent;
        }
        current
    }

    pub fn get_message_w(&mut self, thread_id: u32) -> Option<Message> {
        self.get_message_w_filtered(thread_id, None, 0, 0)
    }

    pub fn has_message_w_filtered_for_thread(
        &self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
    ) -> bool {
        match self.live_thread_birth_key_by_id(thread_id) {
            Some(birth) => self
                .gwe
                .has_message_filtered_for_birth(birth, hwnd, min_msg, max_msg),
            None => self
                .gwe
                .has_message_filtered(thread_id, hwnd, min_msg, max_msg),
        }
    }

    pub fn get_queue_status_for_thread(&mut self, thread_id: u32, flags: u32) -> u32 {
        match self.live_thread_birth_key_by_id(thread_id) {
            Some(birth) => self.gwe.get_queue_status_for_birth(birth, flags),
            None => self.gwe.get_queue_status(thread_id, flags),
        }
    }

    pub fn has_queue_input_for_thread(&self, thread_id: u32, flags: u32) -> bool {
        match self.live_thread_birth_key_by_id(thread_id) {
            Some(birth) => self.gwe.has_queue_input_for_birth(birth, flags),
            None => self.gwe.has_queue_input(thread_id, flags),
        }
    }

    pub fn has_new_queue_input_for_thread(&self, thread_id: u32, flags: u32) -> bool {
        match self.live_thread_birth_key_by_id(thread_id) {
            Some(birth) => self.gwe.has_new_queue_input_for_birth(birth, flags),
            None => self.gwe.has_new_queue_input(thread_id, flags),
        }
    }

    pub fn clear_new_queue_input_for_thread(&mut self, thread_id: u32, flags: u32) {
        match self.live_thread_birth_key_by_id(thread_id) {
            Some(birth) => self.gwe.clear_new_queue_input_for_birth(birth, flags),
            None => self.gwe.clear_new_queue_input(thread_id, flags),
        }
    }

    pub fn get_message_queue_ready_time_stamp_for_thread(&self, thread_id: u32, hwnd: u32) -> u32 {
        match self.live_thread_birth_key_by_id(thread_id) {
            Some(birth) => self
                .gwe
                .get_message_queue_ready_time_stamp_for_birth(birth, hwnd),
            None => self.gwe.get_message_queue_ready_time_stamp(thread_id, hwnd),
        }
    }

    fn get_gwe_message_for_reader(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
    ) -> Option<Message> {
        match self.live_thread_birth_key_by_id(thread_id) {
            Some(birth) => self
                .gwe
                .get_message_filtered_for_birth(birth, hwnd, min_msg, max_msg),
            None => self
                .gwe
                .get_message_filtered(thread_id, hwnd, min_msg, max_msg),
        }
    }

    fn peek_gwe_message_for_reader(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
        flags: PeekFlags,
    ) -> Option<Message> {
        match self.live_thread_birth_key_by_id(thread_id) {
            Some(birth) => self
                .gwe
                .peek_message_filtered_for_birth(birth, hwnd, min_msg, max_msg, flags),
            None => self
                .gwe
                .peek_message_filtered(thread_id, hwnd, min_msg, max_msg, flags),
        }
    }

    pub fn get_message_w_filtered(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
    ) -> Option<Message> {
        self.expire_timed_out_send_messages();
        if self.gwe.advance_caret_blink(self.timers.tick_count()) {
            self.gwe.invalidate_caret_rect();
        }
        // The guest HWND is a retrieval filter, not the recipient of a raw
        // coordinate event. Input resolves by capture/hit-test independently.
        self.drain_remote_input_to_thread_window(thread_id, None);
        if let Some(message) = self.get_gwe_message_for_reader(thread_id, hwnd, min_msg, max_msg) {
            self.clear_timer_message_pending(thread_id, &message);
            self.gwe
                .record_thread_dispatched(thread_id, self.timers.tick_count());
            self.record_message_op(
                "get_message",
                thread_id,
                &message,
                Some(1),
                Some(format_filter_detail(hwnd, min_msg, max_msg)),
            );
            return Some(message);
        }
        self.pump_timers_to_gwe(thread_id);
        let message = self.get_gwe_message_for_reader(thread_id, hwnd, min_msg, max_msg);
        if let Some(message) = message.as_ref() {
            self.clear_timer_message_pending(thread_id, message);
            self.gwe
                .record_thread_dispatched(thread_id, self.timers.tick_count());
            self.record_message_op(
                "get_message",
                thread_id,
                message,
                Some(1),
                Some(format_filter_detail(hwnd, min_msg, max_msg)),
            );
        }
        message
    }

    pub fn take_ready_message_w_filtered(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
    ) -> Option<Message> {
        let message = self.get_gwe_message_for_reader(thread_id, hwnd, min_msg, max_msg)?;
        self.clear_timer_message_pending(thread_id, &message);
        self.gwe
            .record_thread_dispatched(thread_id, self.timers.tick_count());
        self.record_message_op(
            "get_message",
            thread_id,
            &message,
            Some(1),
            Some(format_filter_detail(hwnd, min_msg, max_msg)),
        );
        Some(message)
    }

    /// Read-only in queue semantics: no timer/input pump, delivery trace or
    /// dequeue. Native preflight must inspect the same eligible entry that a
    /// subsequent `take_ready_message_w_filtered` will consume.
    pub fn peek_ready_message_w_filtered(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
    ) -> Option<Message> {
        self.peek_gwe_message_for_reader(thread_id, hwnd, min_msg, max_msg, PeekFlags::empty())
    }

    pub(crate) fn modal_message_w_filtered(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
        flags: PeekFlags,
    ) -> Option<Message> {
        self.peek_gwe_message_for_reader(thread_id, hwnd, min_msg, max_msg, flags)
    }

    pub fn take_ready_visible_message_w_filtered(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
    ) -> Option<Message> {
        let message = match self.live_thread_birth_key_by_id(thread_id) {
            Some(birth) => self
                .gwe
                .get_visible_message_filtered_for_birth(birth, hwnd, min_msg, max_msg),
            None => self
                .gwe
                .get_visible_message_filtered(thread_id, hwnd, min_msg, max_msg),
        }?;
        self.clear_timer_message_pending(thread_id, &message);
        self.gwe
            .record_thread_dispatched(thread_id, self.timers.tick_count());
        self.record_message_op(
            "get_visible_message",
            thread_id,
            &message,
            Some(1),
            Some(format_filter_detail(hwnd, min_msg, max_msg)),
        );
        Some(message)
    }

    pub fn peek_ready_visible_message_w_filtered(
        &self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
    ) -> Option<Message> {
        match self.live_thread_birth_key_by_id(thread_id) {
            Some(birth) => self
                .gwe
                .peek_visible_message_filtered_for_birth(birth, hwnd, min_msg, max_msg),
            None => self
                .gwe
                .peek_visible_message_filtered(thread_id, hwnd, min_msg, max_msg),
        }
    }

    pub fn take_ready_sent_message_w_filtered(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
    ) -> Option<Message> {
        self.peek_ready_sent_message_w_filtered_with_sleep_frame(
            thread_id, hwnd, min_msg, max_msg,
        )?;
        let message = match self.live_thread_birth_key_by_id(thread_id) {
            Some(birth) => self
                .gwe
                .take_sent_message_filtered_for_birth(birth, hwnd, min_msg, max_msg),
            None => self
                .gwe
                .take_sent_message_filtered(thread_id, hwnd, min_msg, max_msg),
        }?;
        self.gwe
            .record_thread_dispatched(thread_id, self.timers.tick_count());
        self.record_message_op(
            "dispatch_sent_message",
            thread_id,
            &message,
            Some(1),
            Some(format_filter_detail(hwnd, min_msg, max_msg)),
        );
        Some(message)
    }

    pub(crate) fn peek_ready_sent_message_w_filtered(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
    ) -> Option<(u64, Message)> {
        self.peek_ready_sent_message_w_filtered_with_sleep_frame(thread_id, hwnd, min_msg, max_msg)
            .map(|(id, message, _frame)| (id, message))
    }

    pub(crate) fn peek_ready_sent_message_w_filtered_with_sleep_frame(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
    ) -> Option<(u64, Message, Option<TaskFrameRef>)> {
        self.expire_timed_out_send_messages();
        let candidate = match self.live_thread_birth_key_by_id(thread_id) {
            Some(birth) => self
                .gwe
                .peek_sent_message_filtered_with_sleep_frame_for_birth(
                    birth, hwnd, min_msg, max_msg,
                ),
            None => self
                .gwe
                .peek_sent_message_filtered_with_sleep_frame(thread_id, hwnd, min_msg, max_msg),
        }?;
        if let Some(frame) = candidate.2
            && self.live_thread_birth_key_by_id(thread_id) != Some(frame.key().owner_birth())
        {
            // A stale receiver incarnation cannot activate a marked send by
            // reusing its numeric TID. Finish only the old birth's deferred
            // transactions so their senders receive a genuine failure.
            for send_id in self
                .gwe
                .terminate_deferred_sent_messages_to_birth(frame.key().owner_birth())
            {
                self.queue_send_reply_wake_candidates(send_id);
            }
            return None;
        }
        Some(candidate)
    }

    pub(crate) fn take_ready_sent_message_w_filtered_if_id(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
        expected_send_id: u64,
    ) -> Option<Message> {
        if self
            .peek_ready_sent_message_w_filtered_with_sleep_frame(thread_id, hwnd, min_msg, max_msg)?
            .0
            != expected_send_id
        {
            return None;
        }
        let message = match self.live_thread_birth_key_by_id(thread_id) {
            Some(birth) => self
                .gwe
                .take_sent_message_filtered_for_birth(birth, hwnd, min_msg, max_msg),
            None => self
                .gwe
                .take_sent_message_filtered(thread_id, hwnd, min_msg, max_msg),
        }?;
        self.gwe
            .record_thread_dispatched(thread_id, self.timers.tick_count());
        self.record_message_op(
            "dispatch_sent_message",
            thread_id,
            &message,
            Some(1),
            Some(format_filter_detail(hwnd, min_msg, max_msg)),
        );
        Some(message)
    }

    pub fn peek_message_w_filtered(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
        flags: PeekFlags,
    ) -> Option<Message> {
        self.expire_timed_out_send_messages();
        // Advance the caret blink timer unconditionally (GWES-internal, not WM_TIMER-priority).
        if self.gwe.advance_caret_blink(self.timers.tick_count()) {
            self.gwe.invalidate_caret_rect();
        }
        // Preserve this HWND only for the queued-message read below. It must
        // not steer host input before the message exists in a recipient FIFO.
        self.drain_remote_input_to_thread_window(thread_id, None);
        if let Some(message) =
            self.peek_gwe_message_for_reader(thread_id, hwnd, min_msg, max_msg, flags)
        {
            if flags.contains(PeekFlags::REMOVE) {
                self.clear_timer_message_pending(thread_id, &message);
                self.gwe
                    .record_thread_dispatched(thread_id, self.timers.tick_count());
            }
            let op = if flags.contains(PeekFlags::REMOVE) {
                "peek_message_remove"
            } else {
                "peek_message"
            };
            self.record_message_op(
                op,
                thread_id,
                &message,
                Some(1),
                Some(format_filter_detail(hwnd, min_msg, max_msg)),
            );
            return Some(message);
        }
        self.pump_timers_to_gwe(thread_id);
        let message = self.peek_gwe_message_for_reader(thread_id, hwnd, min_msg, max_msg, flags);
        if let Some(message) = message.as_ref() {
            if flags.contains(PeekFlags::REMOVE) {
                self.clear_timer_message_pending(thread_id, message);
                self.gwe
                    .record_thread_dispatched(thread_id, self.timers.tick_count());
            }
            let op = if flags.contains(PeekFlags::REMOVE) {
                "peek_message_remove"
            } else {
                "peek_message"
            };
            self.record_message_op(
                op,
                thread_id,
                message,
                Some(1),
                Some(format_filter_detail(hwnd, min_msg, max_msg)),
            );
        }
        message
    }

    /// Consume the already-selected ready message without another timeout,
    /// remote-input, or timer pump. COREDLL uses this after a non-removing
    /// heap-payload preflight so it cannot prepare one LPARAM then remove a
    /// different message during a second selector pass.
    pub(crate) fn take_ready_peek_message_w_filtered(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
    ) -> Option<Message> {
        let message =
            self.peek_gwe_message_for_reader(thread_id, hwnd, min_msg, max_msg, PeekFlags::REMOVE)?;
        self.clear_timer_message_pending(thread_id, &message);
        self.gwe
            .record_thread_dispatched(thread_id, self.timers.tick_count());
        self.record_message_op(
            "peek_message_remove",
            thread_id,
            &message,
            Some(1),
            Some(format_filter_detail(hwnd, min_msg, max_msg)),
        );
        Some(message)
    }

    pub fn post_message_w(&mut self, hwnd: u32, msg: u32, wparam: u32, lparam: u32) -> bool {
        self.post_message_w_for_thread(0, hwnd, msg, wparam, lparam)
    }

    pub fn post_shell_notify_icon_callback(
        &mut self,
        hwnd: u32,
        id: u32,
        event_lparam: u32,
    ) -> bool {
        let Some(icon) = self.shell.notify_icon(hwnd, id).cloned() else {
            return false;
        };
        if icon.callback_message == 0 || !self.gwe.is_window(icon.hwnd) {
            return false;
        }
        self.post_message_w(icon.hwnd, icon.callback_message, icon.id, event_lparam)
    }

    pub fn register_taskbar(&mut self, hwnd: u32) -> bool {
        if hwnd != 0 && !self.gwe.is_window(hwnd) {
            return false;
        }
        self.shell.register_taskbar(hwnd);
        true
    }

    pub fn post_taskbar_notify_icon_message(&mut self, op: u32, data: &NotifyIconData) -> bool {
        const NOTIFYICONDATA_FIXED_SIZE_W: u32 = 152;

        let Some(taskbar_hwnd) = self.shell.taskbar_hwnd() else {
            return false;
        };
        if !self.gwe.is_window(taskbar_hwnd) {
            return false;
        }
        let payload = NotifyIconMessage {
            hwnd: data.hwnd,
            id: data.id,
            flags: data.flags,
            callback_message: data.callback_message,
            icon: data.icon,
            tip: data.tip.clone(),
            state: data.state,
            state_mask: data.state_mask,
        };
        let Some((owner, lparam)) =
            self.queue_notify_icon_payload(taskbar_hwnd, payload, NOTIFYICONDATA_FIXED_SIZE_W)
        else {
            return false;
        };
        if self.post_window_typed_payload_message(
            taskbar_hwnd,
            WM_HANDLESHELLNOTIFYICON,
            op,
            lparam,
            owner,
        ) {
            true
        } else {
            let _ = self.release_message_pointer_payload(owner, lparam);
            false
        }
    }

    pub fn post_shell_notification_callback(
        &mut self,
        clsid: [u8; 16],
        id: u32,
        code: u32,
        data0: u32,
        data1: u32,
    ) -> bool {
        let Some(record) = self.shell.notification(clsid, id).cloned() else {
            return false;
        };
        self.post_shell_notification_record_callback(&record, code, data0, data1, None)
    }

    pub fn post_shell_notification_link_callback(
        &mut self,
        clsid: [u8; 16],
        id: u32,
        link: &str,
    ) -> bool {
        let Some(record) = self.shell.notification(clsid, id).cloned() else {
            return false;
        };
        self.post_shell_notification_record_callback(
            &record,
            crate::ce::shell::SHNN_LINKSEL,
            0,
            0,
            Some(link.to_owned()),
        )
    }

    pub fn post_shell_notification_command_callback(
        &mut self,
        clsid: [u8; 16],
        id: u32,
        command_id: u32,
    ) -> bool {
        let Some(record) = self.shell.notification(clsid, id).cloned() else {
            return false;
        };
        let delivered_com = self.record_shell_notification_com_callback(
            &record,
            ShellNotificationCallbackMethod::OnCommandSelected { command_id },
        );
        let delivered_window = if record.hwnd_sink != 0 && self.gwe.is_window(record.hwnd_sink) {
            self.post_message_w(
                record.hwnd_sink,
                crate::ce::gwe::WM_COMMAND,
                command_id,
                record.id,
            )
        } else {
            false
        };
        delivered_com || delivered_window
    }

    pub fn post_shell_notification_dismiss_callback(
        &mut self,
        clsid: [u8; 16],
        id: u32,
        timed_out: bool,
    ) -> bool {
        self.post_shell_notification_callback(
            clsid,
            id,
            crate::ce::shell::SHNN_DISMISS,
            u32::from(timed_out),
            0,
        )
    }

    pub fn expire_shell_notifications(&mut self) -> usize {
        let expired = self.shell.expire_notifications(self.timers.tick_count());
        let mut posted = 0usize;
        for record in expired {
            if self.post_shell_notification_record_callback(
                &record,
                crate::ce::shell::SHNN_DISMISS,
                1,
                0,
                None,
            ) {
                posted = posted.saturating_add(1);
            }
        }
        posted
    }

    fn post_shell_notification_record_callback(
        &mut self,
        record: &ShellNotificationRecord,
        code: u32,
        data0: u32,
        data1: u32,
        link: Option<String>,
    ) -> bool {
        let method = match code {
            crate::ce::shell::SHNN_SHOW => {
                return self
                    .post_shell_notification_window_callback(record, code, data0, data1, link);
            }
            crate::ce::shell::SHNN_LINKSEL => ShellNotificationCallbackMethod::OnLinkSelected {
                link: link.clone().unwrap_or_default(),
            },
            crate::ce::shell::SHNN_DISMISS => ShellNotificationCallbackMethod::OnDismiss {
                timed_out: data0 != 0,
            },
            _ => {
                return self
                    .post_shell_notification_window_callback(record, code, data0, data1, link);
            }
        };
        let delivered_com = self.record_shell_notification_com_callback(record, method);
        let delivered_window =
            self.post_shell_notification_window_callback(record, code, data0, data1, link);
        delivered_com || delivered_window
    }

    fn record_shell_notification_com_callback(
        &mut self,
        record: &ShellNotificationRecord,
        method: ShellNotificationCallbackMethod,
    ) -> bool {
        if record.clsid == [0; 16] {
            return false;
        }
        let callback_ptr = if record.callback_ptr != 0 {
            record.callback_ptr
        } else {
            match self.com.co_create_instance_guid_values(
                record.clsid,
                crate::ce::shell::IID_ISHELL_NOTIFICATION_CALLBACK,
            ) {
                Ok(callback_ptr) => callback_ptr,
                Err(_) => return false,
            }
        };
        self.shell
            .record_notification_callback(ShellNotificationCallbackRecord {
                clsid: record.clsid,
                id: record.id,
                vtable_offset: method.com_vtable_offset(),
                arguments: method.com_arguments(record.id, record.lparam),
                method,
                lparam: record.lparam,
                callback_ptr,
            });
        true
    }

    fn post_shell_notification_window_callback(
        &mut self,
        record: &ShellNotificationRecord,
        code: u32,
        data0: u32,
        data1: u32,
        link: Option<String>,
    ) -> bool {
        if record.hwnd_sink == 0 || !self.gwe.is_window(record.hwnd_sink) {
            return false;
        }
        let Some((owner, lparam)) = self.queue_shell_notification_payload(
            record.hwnd_sink,
            ShellNotificationMessage {
                hwnd_from: 0,
                id_from: record.id,
                code,
                lparam: record.lparam,
                return_value: 0,
                data0,
                data1,
                link,
            },
        ) else {
            return false;
        };
        if self.post_window_typed_payload_message(
            record.hwnd_sink,
            WM_NOTIFY,
            record.id,
            lparam,
            owner,
        ) {
            true
        } else {
            let _ = self.release_message_pointer_payload(owner, lparam);
            false
        }
    }

    pub fn post_message_w_for_thread(
        &mut self,
        thread_id: u32,
        hwnd: u32,
        msg: u32,
        wparam: u32,
        lparam: u32,
    ) -> bool {
        let time_ms = self.timers.tick_count();
        match hwnd {
            HWND_BROADCAST => {
                let target_threads: Vec<u32> = self
                    .gwe
                    .windows_snapshot()
                    .into_iter()
                    .filter(|window| !window.destroyed && window.parent.is_none())
                    .map(|window| window.thread_id)
                    .collect();
                let target_messages: Vec<(u32, Message)> = self
                    .gwe
                    .windows_snapshot()
                    .into_iter()
                    .filter(|window| !window.destroyed && window.parent.is_none())
                    .map(|window| {
                        (
                            window.thread_id,
                            Message::new(window.hwnd, msg, wparam, lparam, time_ms),
                        )
                    })
                    .collect();
                let posted = self
                    .gwe
                    .post_broadcast_message(msg, wparam, lparam, time_ms);
                if posted {
                    for (target_thread, message) in &target_messages {
                        self.record_message_op(
                            "post_message",
                            *target_thread,
                            message,
                            Some(1),
                            Some("broadcast".to_owned()),
                        );
                    }
                    for target_thread in target_threads {
                        self.queue_message_wake_candidates(target_thread);
                    }
                }
                posted
            }
            0 => {
                let Some(birth) = self.live_thread_birth_key_by_id(thread_id) else {
                    return false;
                };
                let message = Message::new(0, msg, wparam, lparam, time_ms);
                self.gwe
                    .post_thread_message_for_birth(birth, message.clone());
                self.record_message_op(
                    "post_message",
                    thread_id,
                    &message,
                    Some(1),
                    Some("thread".to_owned()),
                );
                self.scheduler
                    .queue_message_wake_candidates_for_birth(birth);
                true
            }
            hwnd => {
                let Some(window) = self.gwe.window(hwnd) else {
                    return false;
                };
                if window.destroyed {
                    return false;
                }
                let target_thread = window.thread_id;
                let delivery_owner = window.delivery_owner();
                if let WindowDeliveryOwner::Birth { thread, process } = delivery_owner
                    && (!self.is_live_thread_birth(thread)
                        || !self.handles.process_execution_live_by_birth(process)
                        || self.handles.live_thread_creation_owner_by_id(target_thread)
                            != Some(process))
                {
                    return false;
                }
                let message = Message::new(hwnd, msg, wparam, lparam, time_ms);
                let posted = match delivery_owner {
                    WindowDeliveryOwner::Birth { thread, .. } => self
                        .gwe
                        .post_message_for_window_birth(hwnd, thread, message.clone()),
                    WindowDeliveryOwner::LegacyUnknown { .. }
                    | WindowDeliveryOwner::SystemDesktop => {
                        self.gwe.post_message_for_window(hwnd, message.clone())
                    }
                };
                if posted {
                    self.record_message_op(
                        "post_message",
                        target_thread,
                        &message,
                        Some(1),
                        Some("window".to_owned()),
                    );
                    match delivery_owner {
                        WindowDeliveryOwner::Birth { thread, .. } => {
                            self.scheduler
                                .queue_message_wake_candidates_for_birth(thread);
                        }
                        WindowDeliveryOwner::LegacyUnknown { .. }
                        | WindowDeliveryOwner::SystemDesktop => {
                            self.queue_message_wake_candidates(target_thread);
                        }
                    }
                }
                posted
            }
        }
    }

    pub fn post_keybd_message_for_thread(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        virtual_key: u32,
        key_down: bool,
        lparam: u32,
        characters: &[u32],
    ) -> bool {
        let time_ms = self.timers.tick_count();
        let target_hwnd = hwnd
            .or_else(|| self.gwe.get_keyboard_target(thread_id))
            .or_else(|| self.gwe.get_focus())
            .or_else(|| self.gwe.get_active_window());
        let target_thread = match target_hwnd {
            Some(hwnd) => match self.gwe.window(hwnd).filter(|window| !window.destroyed) {
                Some(window) => window.thread_id,
                None => return false,
            },
            None => thread_id,
        };
        let hwnd_value = target_hwnd.unwrap_or(0);
        let key_msg = if key_down {
            crate::ce::gwe::WM_KEYDOWN
        } else {
            crate::ce::gwe::WM_KEYUP
        };
        if let Some(hot_key) = self.gwe.matching_hot_key(thread_id, virtual_key, key_down) {
            let hot_key_hwnd = hot_key.hwnd;
            let hot_key_thread = if hot_key_hwnd == 0 {
                hot_key.thread_id
            } else {
                match self
                    .gwe
                    .window(hot_key_hwnd)
                    .filter(|window| !window.destroyed)
                {
                    Some(window) => window.thread_id,
                    None => thread_id,
                }
            };
            let hot_key_message = Message {
                hwnd: hot_key_hwnd,
                msg: crate::ce::gwe::WM_HOTKEY,
                wparam: hot_key.id,
                lparam: (hot_key.modifiers & !0x1000) | (hot_key.vk << 16),
                time_ms,
                source: crate::ce::gwe::MSGSRC_HARDWARE_KEYBOARD,
                mouse_pos_at_post: None,
                typed_payload_owner: None,
                timer_origin: None,
            };
            self.gwe
                .post_message(hot_key_thread, hot_key_message.clone());
            self.record_message_op(
                "post_message",
                hot_key_thread,
                &hot_key_message,
                Some(1),
                Some("hotkey".to_owned()),
            );
            self.queue_message_wake_candidates(hot_key_thread);
        }
        let key_message = Message {
            hwnd: hwnd_value,
            msg: key_msg,
            wparam: virtual_key,
            lparam,
            time_ms,
            source: crate::ce::gwe::MSGSRC_HARDWARE_KEYBOARD,
            mouse_pos_at_post: None,
            typed_payload_owner: None,
            timer_origin: None,
        };
        self.gwe.post_message(target_thread, key_message.clone());
        self.record_message_op(
            "post_message",
            target_thread,
            &key_message,
            Some(1),
            Some("keyboard".to_owned()),
        );
        if key_down {
            for character in characters
                .iter()
                .copied()
                .filter(|character| *character != 0)
            {
                let char_message = Message {
                    hwnd: hwnd_value,
                    msg: crate::ce::gwe::WM_CHAR,
                    wparam: character,
                    lparam,
                    time_ms,
                    source: crate::ce::gwe::MSGSRC_HARDWARE_KEYBOARD,
                    mouse_pos_at_post: None,
                    typed_payload_owner: None,
                    timer_origin: None,
                };
                self.gwe.post_message(target_thread, char_message.clone());
                self.record_message_op(
                    "post_message",
                    target_thread,
                    &char_message,
                    Some(1),
                    Some("keyboard".to_owned()),
                );
            }
        }
        self.queue_message_wake_candidates(target_thread);
        true
    }

    pub fn set_keyboard_target(&mut self, thread_id: u32, hwnd: Option<u32>) -> Option<u32> {
        self.gwe.set_keyboard_target(thread_id, hwnd)
    }

    pub fn post_thread_message_w(
        &mut self,
        target_thread_id: u32,
        msg: u32,
        wparam: u32,
        lparam: u32,
    ) -> bool {
        let Some(birth) = self.live_thread_birth_key_by_id(target_thread_id) else {
            return false;
        };
        self.post_thread_message_w_for_birth(birth, msg, wparam, lparam)
    }

    /// The captured recipient must still be the execution-live task before
    /// any queue, trace, status or wake side effect is published.
    pub fn post_thread_message_w_for_birth(
        &mut self,
        birth: ThreadBirthKey,
        msg: u32,
        wparam: u32,
        lparam: u32,
    ) -> bool {
        if !self.is_live_thread_birth(birth) {
            return false;
        }
        let target_thread_id = birth.thread_id();
        let message = Message::new(0, msg, wparam, lparam, self.timers.tick_count());
        self.gwe
            .post_thread_message_for_birth(birth, message.clone());
        self.record_message_op(
            "post_message",
            target_thread_id,
            &message,
            Some(1),
            Some("thread".to_owned()),
        );
        self.scheduler
            .queue_message_wake_candidates_for_birth(birth);
        true
    }

    pub fn send_message_w(&mut self, hwnd: u32, msg: u32, wparam: u32, lparam: u32) -> Option<u32> {
        let window = self.gwe.window(hwnd).filter(|window| !window.destroyed)?;
        let target_thread = window.thread_id;
        if let WindowDeliveryOwner::Birth { thread, process } = window.delivery_owner()
            && (!self.is_live_thread_birth(thread)
                || !self.handles.process_execution_live_by_birth(process)
                || self.handles.live_thread_creation_owner_by_id(target_thread) != Some(process))
        {
            return None;
        }
        self.gwe.begin_send_message(target_thread);
        let result = self.gwe.send_message(hwnd, msg, wparam, lparam);
        self.gwe.end_direct_send_message(target_thread);
        if msg == crate::ce::gwe::WM_ACTIVATE && wparam != crate::ce::gwe::WA_INACTIVE {
            let _ = self.set_focus(Some(hwnd));
        }
        if msg == crate::ce::gwe::WM_CANCELMODE {
            // CE DefWindowProcW: release mouse capture when the window that holds it gets WM_CANCELMODE.
            if self.gwe.get_capture() == Some(hwnd) {
                self.release_capture();
            }
        }
        if msg == crate::ce::gwe::WM_NEXTDLGCTL {
            // CE DefWindowProcW WM_NEXTDLGCTL: lparam bit 0 selects mode.
            // lparam & 1 → wparam is the HWND to focus directly.
            // lparam & 1 == 0 → navigate: wparam=0 forward, wparam!=0 backward.
            let new_focus = if lparam & 1 != 0 {
                self.gwe.is_window(wparam).then_some(wparam)
            } else {
                let current = self.gwe.get_focus().unwrap_or(hwnd);
                self.gwe.get_next_dlg_tab_item(hwnd, current, wparam != 0)
            };
            if let Some(next) = new_focus {
                let _ = self.set_focus(Some(next));
                if self.gwe.is_push_button(next) {
                    let id = self.gwe.get_dlg_ctrl_id(next).unwrap_or(0);
                    let _ = self.send_message_w(hwnd, crate::ce::gwe::DM_SETDEFID, id, 0);
                }
            }
        }
        result
    }

    pub fn begin_cross_thread_send_message_w(
        &mut self,
        caller_thread_id: u32,
        hwnd: u32,
        msg: u32,
        wparam: u32,
        lparam: u32,
        timeout_ms: Option<u32>,
    ) -> Option<u64> {
        self.begin_cross_thread_send_message_w_with_sender(
            caller_thread_id,
            None,
            hwnd,
            msg,
            wparam,
            lparam,
            timeout_ms,
        )
    }

    /// Publish a synchronous guest send only for the caller incarnation
    /// captured by the native/raw entrypoint. A reused numeric TID cannot
    /// authenticate an earlier caller after its process has exited.
    pub fn begin_cross_thread_send_message_w_for_birth(
        &mut self,
        caller_birth: ThreadBirthKey,
        hwnd: u32,
        msg: u32,
        wparam: u32,
        lparam: u32,
        timeout_ms: Option<u32>,
    ) -> Option<u64> {
        if !self.is_live_thread_birth(caller_birth) {
            return None;
        }
        let process = self
            .handles
            .live_thread_creation_owner_by_id(caller_birth.thread_id())?;
        if !self.handles.process_execution_live_by_birth(process)
            || self.current_process_birth_key() != Some(process)
        {
            return None;
        }
        self.begin_cross_thread_send_message_w_with_sender(
            caller_birth.thread_id(),
            Some((caller_birth, process)),
            hwnd,
            msg,
            wparam,
            lparam,
            timeout_ms,
        )
    }

    fn begin_cross_thread_send_message_w_with_sender(
        &mut self,
        caller_thread_id: u32,
        sender_birth: Option<(ThreadBirthKey, ProcessBirthKey)>,
        hwnd: u32,
        msg: u32,
        wparam: u32,
        lparam: u32,
        timeout_ms: Option<u32>,
    ) -> Option<u64> {
        let window = self.gwe.window(hwnd).filter(|window| !window.destroyed)?;
        let target_thread = window.thread_id;
        if let WindowDeliveryOwner::Birth { thread, process } = window.delivery_owner()
            && (!self.is_live_thread_birth(thread)
                || !self.handles.process_execution_live_by_birth(process)
                || self.handles.live_thread_creation_owner_by_id(target_thread) != Some(process))
        {
            return None;
        }
        if target_thread == caller_thread_id {
            return None;
        }
        let flags = timeout_ms.map_or(SMF_NULL, |_| crate::ce::gwe::SMF_TIMEOUT);
        let deferred_frame = self
            .live_thread_birth_key_by_id(target_thread)
            .and_then(|birth| self.handles.active_finite_sleep_frame(birth))
            .filter(|frame| self.is_blocked_finite_sleep_frame(*frame));
        let message = Message::new(hwnd, msg, wparam, lparam, self.timers.tick_count());
        let send_id = match (sender_birth, deferred_frame) {
            (Some((thread, process)), Some(frame)) => self
                .gwe
                .queue_send_message_for_window_owned_deferred_by_finite_sleep(
                    crate::ce::gwe::SentMessageSenderOwner::Birth { thread, process },
                    hwnd,
                    message,
                    flags,
                    timeout_ms,
                    frame,
                ),
            (Some((thread, process)), None) => self.gwe.queue_send_message_for_window_owned(
                crate::ce::gwe::SentMessageSenderOwner::Birth { thread, process },
                hwnd,
                message,
                flags,
                timeout_ms,
            ),
            (None, Some(frame)) => self
                .gwe
                .queue_send_message_for_window_deferred_by_finite_sleep(
                    Some(caller_thread_id),
                    hwnd,
                    message,
                    flags,
                    timeout_ms,
                    frame,
                ),
            (None, None) => self.gwe.queue_send_message_for_window(
                Some(caller_thread_id),
                hwnd,
                message,
                flags,
                timeout_ms,
            ),
        };
        if let Some(id) = send_id {
            if sender_birth.is_none() {
                // Legacy numeric fixtures retain their old projected PID, but
                // this cannot authorize birth-owned cleanup or native resume.
                self.gwe
                    .set_sent_message_sender_process(id, self.current_process_id);
            }
            self.record_message_trace(MessageTraceRecord {
                op: "queue_send_message",
                thread_id: target_thread,
                hwnd: Some(hwnd),
                msg: Some(msg),
                wparam: Some(wparam),
                lparam: Some(lparam),
                screen_pos: None,
                source: Some(crate::ce::gwe::MSGSRC_SOFTWARE_SEND),
                result: Some(1),
                detail: Some(format!("sender_thread={caller_thread_id}")),
            });
            self.queue_message_wake_candidates(target_thread);
        }
        send_id
    }

    pub fn take_completed_send_message_result(&mut self, send_id: u64) -> Option<u32> {
        self.gwe.take_completed_sent_message_result(send_id)
    }

    pub fn complete_active_sent_message(&mut self, thread_id: u32, result: u32) -> Option<u64> {
        let active_id = self.gwe.active_sent_message_id(thread_id)?;
        if let Some(receiver_birth) = self
            .gwe
            .active_sent_message_receiver_birth(thread_id, active_id)?
            && self.live_thread_birth_key_by_id(thread_id) != Some(receiver_birth)
        {
            return None;
        }
        let was_ready = self.gwe.sent_message_reply_recorded(active_id);
        let send_id = self.gwe.complete_active_sent_message(thread_id, result)?;
        if !was_ready {
            self.queue_send_reply_wake_candidates(send_id);
        }
        Some(send_id)
    }

    /// Complete only the callback belonging to this live receiver birth and
    /// exact send ID; a nested ancestor or reused numeric TID is not a match.
    pub fn complete_sent_message_for_birth(
        &mut self,
        birth: ThreadBirthKey,
        send_id: u64,
        result: u32,
    ) -> Option<u64> {
        if !self.is_live_thread_birth(birth) {
            return None;
        }
        let was_ready = self.gwe.sent_message_reply_recorded(send_id);
        let completed = self
            .gwe
            .complete_active_sent_message_for_birth_and_id(birth, send_id, result)?;
        if !was_ready {
            self.queue_send_reply_wake_candidates(completed);
        }
        Some(completed)
    }

    /// Complete an exact queued callback for an explicitly numeric legacy
    /// HWND. Birth-owned callbacks must use `complete_sent_message_for_birth`.
    pub fn complete_legacy_sent_message_for_id(
        &mut self,
        receiver_thread_id: u32,
        send_id: u64,
        result: u32,
    ) -> Option<u64> {
        let was_ready = self.gwe.sent_message_reply_recorded(send_id);
        let completed = self.gwe.complete_active_legacy_sent_message_for_id(
            receiver_thread_id,
            send_id,
            result,
        )?;
        if !was_ready {
            self.queue_send_reply_wake_candidates(completed);
        }
        Some(completed)
    }

    pub fn reply_message(&mut self, thread_id: u32, result: u32) -> bool {
        let Some(send_id) = self.gwe.reply_message(thread_id, result) else {
            return false;
        };
        self.queue_send_reply_wake_candidates(send_id);
        true
    }

    pub fn destroy_window(&mut self, hwnd: u32) -> bool {
        self.destroy_window_with_reason(hwnd, "DestroyWindow")
    }

    pub fn destroy_window_with_framebuffer(
        &mut self,
        hwnd: u32,
        reason: &'static str,
        framebuffer: Option<&mut dyn Framebuffer>,
    ) -> bool {
        let targets = self.window_destroy_targets(hwnd);
        let destroyed = self.destroy_window_with_reason(hwnd, reason);
        if destroyed {
            if let Some(framebuffer) = framebuffer {
                let _ = self.restore_window_backing_stores(&targets, framebuffer);
            } else {
                self.discard_window_backing_stores(&targets);
            }
        }
        destroyed
    }

    pub fn destroy_window_with_reason(&mut self, hwnd: u32, reason: &'static str) -> bool {
        let Some(targets) = self.gwe.window_and_descendants(hwnd) else {
            self.record_window_lifecycle_trace(
                "destroy_window",
                0,
                Some(hwnd),
                Some(0),
                Some(format!("reason={reason}/invalid_window")),
            );
            return false;
        };
        let thread_id = self
            .gwe
            .window(hwnd)
            .map(|window| window.thread_id)
            .unwrap_or(0);
        self.record_window_lifecycle_trace(
            "destroy_window_begin",
            thread_id,
            Some(hwnd),
            Some(targets.len() as u32),
            Some(format!("reason={reason}/targets={targets:?}")),
        );
        self.clear_destroyed_window_focus_and_activation(hwnd);
        if let Some(owner) = self
            .gwe
            .clipboard_render_all_owner()
            .filter(|owner| targets.contains(owner))
        {
            let target_thread = self
                .gwe
                .window_thread_process_id(owner)
                .map(|(thread, _)| thread)
                .unwrap_or(0);
            let mut message = Message::new(
                owner,
                crate::ce::gwe::WM_RENDERALLFORMATS,
                0,
                0,
                self.timers.tick_count(),
            );
            message.source = crate::ce::gwe::MSGSRC_SOFTWARE_SEND;
            let result = self.send_message_w(owner, message.msg, message.wparam, message.lparam);
            self.record_message_op(
                "send_message",
                target_thread,
                &message,
                result,
                Some("clipboard_render_all".to_owned()),
            );
        }
        let doomed_send_ids = self.gwe.sent_message_ids_for_windows(&targets);
        for target in targets.iter().rev().copied() {
            if self
                .gwe
                .window(target)
                .is_some_and(|window| !window.destroyed && !window.destroy_message_sent)
            {
                let _ = self.send_message_w(target, crate::ce::gwe::WM_DESTROY, 0, 0);
            }
        }
        let destroyed = self.gwe.destroy_window(hwnd, self.timers.tick_count());
        if destroyed {
            self.release_discarded_message_pointer_payloads();
            self.release_message_pointer_payloads_for_windows(&targets);
            self.shell.remove_windows_state(&targets);
            self.timers.remove_window_timers(&targets);
            for send_id in doomed_send_ids {
                self.queue_send_reply_wake_candidates(send_id);
            }
            self.queue_paint_wake_candidates();
        }
        self.record_window_lifecycle_trace(
            "destroy_window_end",
            thread_id,
            Some(hwnd),
            Some(u32::from(destroyed)),
            Some(format!("reason={reason}")),
        );
        destroyed
    }

    pub fn end_dialog(&mut self, thread_id: u32, hwnd: u32, result: u32) -> bool {
        let ok = self.gwe.end_dialog(hwnd, result);
        self.record_window_lifecycle_trace(
            "end_dialog",
            thread_id,
            Some(hwnd),
            Some(u32::from(ok)),
            Some(format!("dialog_result=0x{result:08x}")),
        );
        ok
    }

    pub fn send_notify_message_w(
        &mut self,
        caller_thread_id: u32,
        hwnd: u32,
        msg: u32,
        wparam: u32,
        lparam: u32,
    ) -> bool {
        if hwnd == HWND_BROADCAST {
            // CE broadcasts in top-to-bottom Z-order. Build the target list from the
            // Z-order snapshot so the ordering is deterministic and matches CE behaviour.
            let z_order = self.gwe.z_order_snapshot();
            let targets: Vec<(u32, u32)> = z_order
                .into_iter()
                .filter(|candidate| *candidate != crate::ce::gwe::DESKTOP_HWND)
                .filter_map(|candidate| {
                    self.gwe
                        .window(candidate)
                        .filter(|window| !window.destroyed && window.parent.is_none())
                        .map(|window| (candidate, window.thread_id))
                })
                .collect();
            let mut delivered = false;
            let time_ms = self.timers.tick_count();
            for (target_hwnd, target_thread) in targets {
                if target_thread == caller_thread_id {
                    if self
                        .send_message_w(target_hwnd, msg, wparam, lparam)
                        .is_some()
                    {
                        delivered = true;
                        self.record_message_trace(MessageTraceRecord {
                            op: "send_notify_message",
                            thread_id: target_thread,
                            hwnd: Some(target_hwnd),
                            msg: Some(msg),
                            wparam: Some(wparam),
                            lparam: Some(lparam),
                            screen_pos: None,
                            source: Some(crate::ce::gwe::MSGSRC_SOFTWARE_SEND),
                            result: Some(1),
                            detail: Some(format!("sender_thread={caller_thread_id}/broadcast")),
                        });
                    }
                    continue;
                }
                let queued = self
                    .gwe
                    .queue_send_message_for_window(
                        None,
                        target_hwnd,
                        Message::new(target_hwnd, msg, wparam, lparam, time_ms),
                        crate::ce::gwe::SMF_SENDER_NO_WAIT | crate::ce::gwe::SMF_NOTIFY_MESSAGE,
                        None,
                    )
                    .is_some();
                if queued {
                    delivered = true;
                    self.record_message_trace(MessageTraceRecord {
                        op: "send_notify_message",
                        thread_id: target_thread,
                        hwnd: Some(target_hwnd),
                        msg: Some(msg),
                        wparam: Some(wparam),
                        lparam: Some(lparam),
                        screen_pos: None,
                        source: Some(crate::ce::gwe::MSGSRC_SOFTWARE_POST),
                        result: Some(1),
                        detail: Some(format!("sender_thread={caller_thread_id}/broadcast")),
                    });
                    self.queue_message_wake_candidates(target_thread);
                }
            }
            return delivered;
        }
        let Some(target_thread) = self
            .gwe
            .window(hwnd)
            .filter(|window| !window.destroyed)
            .map(|window| window.thread_id)
        else {
            return false;
        };
        if target_thread == caller_thread_id {
            return self.send_message_w(hwnd, msg, wparam, lparam).is_some();
        }
        let queued = self
            .gwe
            .queue_send_message_for_window(
                None,
                hwnd,
                Message::new(hwnd, msg, wparam, lparam, self.timers.tick_count()),
                crate::ce::gwe::SMF_SENDER_NO_WAIT | crate::ce::gwe::SMF_NOTIFY_MESSAGE,
                None,
            )
            .is_some();
        if queued {
            self.record_message_trace(MessageTraceRecord {
                op: "send_notify_message",
                thread_id: target_thread,
                hwnd: Some(hwnd),
                msg: Some(msg),
                wparam: Some(wparam),
                lparam: Some(lparam),
                screen_pos: None,
                source: Some(crate::ce::gwe::MSGSRC_SOFTWARE_SEND),
                result: Some(1),
                detail: Some(format!("sender_thread={caller_thread_id}")),
            });
            self.queue_message_wake_candidates(target_thread);
        }
        queued
    }

    pub fn post_quit_message(&mut self, thread_id: u32, exit_code: u32) {
        if let Some(birth) = self.live_thread_birth_key_by_id(thread_id) {
            let _ = self.post_quit_message_for_birth(birth, exit_code);
        }
    }

    pub fn post_quit_message_for_birth(&mut self, birth: ThreadBirthKey, exit_code: u32) -> bool {
        if !self.is_live_thread_birth(birth) {
            return false;
        }
        self.gwe
            .post_quit_message_for_birth(birth, exit_code, self.timers.tick_count());
        self.scheduler
            .queue_message_wake_candidates_for_birth(birth);
        true
    }

    pub fn dispatch_message_w(&mut self, message: Message) -> u32 {
        self.dispatch_message_w_for_thread(0, message)
    }

    pub fn dispatch_message_w_for_thread(&mut self, thread_id: u32, message: Message) -> u32 {
        if message.msg == crate::ce::gwe::WM_QUIT {
            return message.wparam;
        }
        if message.typed_payload_owner.is_some()
            && self.message_pointer_payload_for_message(&message).is_none()
        {
            // A tagged host payload cannot become an ordinary guest lParam
            // after its window/birth or registry entry has gone stale.
            return 0;
        }
        let sent_context_thread = if message.source == crate::ce::gwe::MSGSRC_SOFTWARE_SEND {
            self.gwe
                .window(message.hwnd)
                .map(|window| window.thread_id)
                .filter(|thread_id| self.gwe.active_sent_message_id(*thread_id).is_some())
        } else {
            None
        }
        .or_else(|| {
            (thread_id != 0 && self.gwe.active_sent_message_id(thread_id).is_some())
                .then_some(thread_id)
        });
        let result = if sent_context_thread.is_some() {
            self.gwe
                .send_message(message.hwnd, message.msg, message.wparam, message.lparam)
        } else {
            self.send_message_w(message.hwnd, message.msg, message.wparam, message.lparam)
        }
        .unwrap_or(0);
        if let Some(sent_context_thread) = sent_context_thread {
            self.complete_active_sent_message(sent_context_thread, result);
        }
        self.release_dispatch_message_pointer_payload(&message);
        result
    }

    fn recipient_birth_for_window(&self, hwnd: u32) -> Option<ProcessBirthKey> {
        let window = self.gwe.window(hwnd)?;
        if window.destroyed {
            return None;
        }
        match window.delivery_owner() {
            WindowDeliveryOwner::Birth { thread, process } => (self.is_live_thread_birth(thread)
                && self
                    .handles
                    .live_thread_creation_owner_by_id(thread.thread_id())
                    == Some(process)
                && self.handles.process_execution_live_by_birth(process))
            .then_some(process),
            WindowDeliveryOwner::LegacyUnknown { process } => {
                process.filter(|process| self.handles.process_execution_live_by_birth(*process))
            }
            WindowDeliveryOwner::SystemDesktop => None,
        }
    }

    pub fn message_pointer_payload_for_message(
        &self,
        message: &Message,
    ) -> Option<MessagePointerPayload> {
        let owner = message.typed_payload_owner?;
        (self.recipient_birth_for_window(message.hwnd) == Some(owner))
            .then(|| self.gwe.message_pointer_payload(owner, message.lparam))
            .flatten()
    }

    pub fn record_delivered_typed_payload_message(
        &mut self,
        owner: ProcessBirthKey,
        thread_id: u32,
        msg_ptr: u32,
        message: &Message,
    ) {
        let key = (owner, thread_id, msg_ptr);
        if message.typed_payload_owner == Some(owner)
            && self.message_pointer_payload_for_message(message).is_some()
        {
            self.delivered_typed_payload_messages
                .insert(key, message.clone());
        } else {
            // The same guest MSG slot may be reused for an ordinary message.
            self.delivered_typed_payload_messages.remove(&key);
        }
    }

    pub fn reattach_delivered_typed_payload_message(
        &mut self,
        owner: ProcessBirthKey,
        thread_id: u32,
        msg_ptr: u32,
        message: &mut Message,
    ) {
        let provenance =
            self.delivered_typed_payload_owner_for_message(owner, thread_id, msg_ptr, message);
        self.delivered_typed_payload_messages
            .remove(&(owner, thread_id, msg_ptr));
        message.typed_payload_owner = provenance;
    }

    pub fn delivered_typed_payload_owner_for_message(
        &self,
        owner: ProcessBirthKey,
        thread_id: u32,
        msg_ptr: u32,
        message: &Message,
    ) -> Option<ProcessBirthKey> {
        let delivered = self
            .delivered_typed_payload_messages
            .get(&(owner, thread_id, msg_ptr))?;
        (delivered.typed_payload_owner == Some(owner)
            && delivered.hwnd == message.hwnd
            && delivered.msg == message.msg
            && delivered.wparam == message.wparam
            && delivered.lparam == message.lparam
            && self
                .message_pointer_payload_for_message(delivered)
                .is_some())
        .then_some(owner)
    }

    pub fn message_pointer_payload(
        &self,
        owner: ProcessBirthKey,
        ptr: u32,
    ) -> Option<MessagePointerPayload> {
        self.gwe.message_pointer_payload(owner, ptr)
    }

    pub fn release_message_pointer_payload(
        &mut self,
        owner: ProcessBirthKey,
        ptr: u32,
    ) -> Option<MessagePointerPayload> {
        let payload = self.gwe.take_message_pointer_payload(owner, ptr)?;
        self.typed_payload_recipients.remove(&(owner, ptr));
        self.delivered_typed_payload_messages
            .retain(|(birth, _, _), message| *birth != owner || message.lparam != ptr);
        let _ = self.memory.heap_free(owner, PROCESS_HEAP_HANDLE, 0, ptr);
        Some(payload)
    }

    pub fn release_dispatched_typed_payload(
        &mut self,
        owner: ProcessBirthKey,
        ptr: u32,
    ) -> Option<MessagePointerPayload> {
        // A PM_NOREMOVE MSG can be dispatched while an identical tagged
        // message remains in GWE's queue. Its heap bytes stay live for the
        // eventual PM_REMOVE delivery.
        if self.gwe.has_queued_typed_payload_message(owner, ptr) {
            None
        } else {
            self.release_message_pointer_payload(owner, ptr)
        }
    }

    fn release_discarded_message_pointer_payloads(&mut self) {
        for message in self.gwe.take_discarded_host_payload_messages() {
            if let Some(owner) = message.typed_payload_owner {
                let _ = self.release_message_pointer_payload(owner, message.lparam);
            }
        }
    }

    fn release_message_pointer_payloads_for_owner(&mut self, owner: ProcessBirthKey) {
        for ptr in self.gwe.message_pointer_payload_addresses_for_owner(owner) {
            let _ = self.release_message_pointer_payload(owner, ptr);
        }
    }

    fn release_message_pointer_payloads_for_windows(&mut self, targets: &[u32]) {
        // GetMessage removes a message from GWE's queue before the guest can
        // Dispatch it. The registry must remember its original recipient so
        // destroying that window also retires these already-delivered values.
        let doomed: Vec<_> = self
            .typed_payload_recipients
            .iter()
            .filter(|(_, hwnd)| targets.contains(hwnd))
            .map(|(key, _)| *key)
            .collect();
        for (owner, ptr) in doomed {
            let _ = self.release_message_pointer_payload(owner, ptr);
        }
    }

    fn release_dispatch_message_pointer_payload(&mut self, message: &Message) {
        let should_release = matches!(
            (
                message.msg,
                self.message_pointer_payload_for_message(message)
            ),
            (
                WM_WINDOWPOSCHANGED,
                Some(MessagePointerPayload::WindowPos(_))
            ) | (WM_NOTIFY, Some(MessagePointerPayload::ShellNotification(_)))
                | (
                    WM_HANDLESHELLNOTIFYICON,
                    Some(MessagePointerPayload::NotifyIcon(_))
                )
        );
        if should_release {
            if let Some(owner) = message.typed_payload_owner {
                self.release_dispatched_typed_payload(owner, message.lparam);
            }
        }
    }

    pub fn message_pump_step(&mut self, thread_id: u32) -> MessagePumpResult {
        let Some(message) = self.get_message_w(thread_id) else {
            return MessagePumpResult::Idle;
        };
        if message.msg == crate::ce::gwe::WM_QUIT {
            return MessagePumpResult::Quit(message.wparam);
        }
        MessagePumpResult::Dispatched(self.dispatch_message_w_for_thread(thread_id, message))
    }

    fn post_gwe_message(&mut self, thread_id: u32, message: Message) {
        let _ = self.post_gwe_message_with_window_owner(thread_id, None, message);
    }

    /// Remote input already resolved a HWND. Keep its original message
    /// metadata while posting to the immutable window owner, then run the
    /// existing button-command and modal side effects only after publication.
    fn post_gwe_message_with_window_owner(
        &mut self,
        thread_id: u32,
        delivery_owner: Option<WindowDeliveryOwner>,
        message: Message,
    ) -> bool {
        let button_click = if message.msg == crate::ce::gwe::WM_LBUTTONUP
            && self.gwe.is_push_button(message.hwnd)
        {
            self.gwe.get_parent(message.hwnd).and_then(|parent| {
                self.gwe.get_dlg_ctrl_id(message.hwnd).map(|id| {
                    let command = id & 0xffff;
                    let time_ms = message.time_ms;
                    (
                        parent,
                        Message::new(
                            parent,
                            crate::ce::gwe::WM_COMMAND,
                            command,
                            message.hwnd,
                            time_ms,
                        ),
                    )
                })
            })
        } else {
            None
        };
        let trace_message = message.clone();
        let posted = match delivery_owner {
            Some(WindowDeliveryOwner::Birth { thread, .. }) => self
                .gwe
                .post_message_for_window_birth(trace_message.hwnd, thread, message),
            Some(WindowDeliveryOwner::LegacyUnknown { .. })
            | Some(WindowDeliveryOwner::SystemDesktop) => self
                .gwe
                .post_message_for_window(trace_message.hwnd, message),
            None => {
                self.gwe.post_message(thread_id, message);
                true
            }
        };
        if !posted {
            return false;
        }
        self.record_message_op("post_message", thread_id, &trace_message, Some(1), None);
        if let Some((parent, command_message)) = button_click {
            let trace_message = command_message.clone();
            let parent_thread_id = self
                .gwe
                .window_thread_process_id(parent)
                .map(|(thread_id, _)| thread_id);
            let command = command_message.wparam & 0xffff;
            if self
                .gwe
                .queue_sent_message_for_window(parent, command_message.clone())
            {
                self.record_message_op(
                    "button_click",
                    thread_id,
                    &trace_message,
                    Some(1),
                    Some("BN_CLICKED".to_owned()),
                );
                if let Some(parent_thread_id) = parent_thread_id {
                    self.queue_message_wake_candidates(parent_thread_id);
                }
            } else if let Some(parent_thread_id) = parent_thread_id {
                self.gwe.post_message(parent_thread_id, command_message);
                self.record_message_op(
                    "button_click_post",
                    thread_id,
                    &trace_message,
                    Some(1),
                    Some("BN_CLICKED".to_owned()),
                );
                self.queue_message_wake_candidates(parent_thread_id);
            }
            if let Some(parent_thread_id) = parent_thread_id {
                let _ = self.dismiss_modal_dialog_wait_from_button(
                    parent,
                    parent_thread_id,
                    command,
                    trace_message.hwnd,
                );
            }
        }
        if let Some(WindowDeliveryOwner::Birth { thread, .. }) = delivery_owner {
            self.scheduler
                .queue_message_wake_candidates_for_birth(thread);
        } else {
            self.queue_message_wake_candidates(thread_id);
        }
        true
    }

    fn dismiss_modal_dialog_wait_from_button(
        &mut self,
        dialog_hwnd: u32,
        thread_id: u32,
        command: u32,
        button_hwnd: u32,
    ) -> bool {
        let is_dialog = self.gwe.window(dialog_hwnd).is_some_and(|window| {
            window.thread_id == thread_id
                && !window.destroyed
                && window.class_name.eq_ignore_ascii_case("dialog")
        });
        if !is_dialog {
            return false;
        }
        // Numeric message queues are wake hints only. Their candidate IDs
        // cannot authorize retiring a different task's wait after TID reuse.
        // With no canonical live birth, only an explicitly legacy unknown
        // fixture may match.
        let owner = match self.handles.thread_birth_key_by_id(thread_id) {
            Some(birth) if self.handles.thread_execution_live_by_id(thread_id) => {
                ModalDialogWaitOwner::Birth(birth)
            }
            _ => ModalDialogWaitOwner::LegacyUnknownTid(thread_id),
        };
        let wait_id = self
            .scheduler
            .message_waiter_ids_for_thread(thread_id)
            .into_iter()
            .find(|wait_id| {
                self.scheduler.blocked_wait(*wait_id).is_some_and(|wait| {
                    owner.matches(wait)
                        && matches!(
                            wait.kind,
                            SchedulerBlockedWaitKind::ModalMessageBox {
                                dialog_hwnd: wait_dialog_hwnd,
                            } if wait_dialog_hwnd == dialog_hwnd
                        )
                })
            });
        let Some(wait_id) = wait_id else {
            return false;
        };
        self.modal_dialog_results
            .insert((thread_id, dialog_hwnd), command);
        let removed_get_message_waits =
            self.remove_dialog_get_message_waiters(thread_id, dialog_hwnd, owner);
        self.record_message_trace(MessageTraceRecord {
            op: "dialog_modal_button_dismiss",
            thread_id,
            hwnd: Some(dialog_hwnd),
            msg: Some(crate::ce::gwe::WM_COMMAND),
            wparam: Some(command),
            lparam: Some(button_hwnd),
            screen_pos: None,
            source: Some(crate::ce::gwe::MSGSRC_SOFTWARE_POST),
            result: Some(1),
            detail: Some(format!(
                "wait_id={wait_id}/get_message_waits={removed_get_message_waits}/destroy=deferred"
            )),
        });
        self.queue_message_wake_candidates(thread_id);
        true
    }

    fn remove_dialog_get_message_waiters(
        &mut self,
        thread_id: u32,
        dialog_hwnd: u32,
        owner: ModalDialogWaitOwner,
    ) -> usize {
        let wait_ids = self
            .scheduler
            .message_waiter_ids_for_thread(thread_id)
            .into_iter()
            .filter(|wait_id| {
                self.scheduler.blocked_wait(*wait_id).is_some_and(|wait| {
                    owner.matches(wait)
                        && matches!(
                            wait.kind,
                            SchedulerBlockedWaitKind::GetMessage {
                                hwnd: Some(hwnd), ..
                            } if hwnd == dialog_hwnd
                        )
                })
            })
            .collect::<Vec<_>>();
        for wait_id in &wait_ids {
            let _ = self.remove_blocked_waiter(*wait_id);
        }
        wait_ids.len()
    }

    pub fn set_timer(
        &mut self,
        hwnd: Option<u32>,
        requested_id: Option<u32>,
        period_ms: u32,
    ) -> u32 {
        let thread_id = hwnd
            .and_then(|hwnd| self.gwe.window_thread_process_id(hwnd))
            .map(|(thread_id, _)| thread_id)
            .unwrap_or(0);
        self.set_timer_for_thread(thread_id, hwnd, requested_id, period_ms, None)
    }

    pub fn set_timer_for_thread(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        requested_id: Option<u32>,
        period_ms: u32,
        callback: Option<u32>,
    ) -> u32 {
        let id = self.timers.set_timer(
            thread_id,
            hwnd,
            requested_id,
            period_ms,
            crate::ce::gwe::WM_TIMER,
            callback,
        );
        let message = crate::ce::gwe::Message::new(
            hwnd.unwrap_or(0),
            crate::ce::gwe::WM_TIMER,
            id,
            callback.unwrap_or(0),
            self.timers.tick_count(),
        );
        self.record_message_op(
            "set_timer",
            thread_id,
            &message,
            Some(id),
            Some(format!(
                "hwnd={}/requested_id={}/id=0x{id:08x}/period_ms={period_ms}/callback=0x{:08x}",
                format_optional_hwnd(hwnd),
                requested_id
                    .map(|id| format!("0x{id:08x}"))
                    .unwrap_or_else(|| "auto".to_owned()),
                callback.unwrap_or(0)
            )),
        );
        id
    }

    /// Guest HWND-less SetTimer retains its captured caller incarnation.
    /// Numeric-TID and HWND compatibility entry points cannot authorize it.
    pub fn set_timer_for_birth(
        &mut self,
        birth: ThreadBirthKey,
        requested_id: Option<u32>,
        period_ms: u32,
        callback: Option<u32>,
    ) -> u32 {
        if !self.is_live_thread_birth(birth) {
            return 0;
        }
        let thread_id = birth.thread_id();
        let id = self.timers.set_thread_timer_for_birth(
            birth,
            requested_id,
            period_ms,
            crate::ce::gwe::WM_TIMER,
            callback,
        );
        if id == 0 {
            return 0;
        }
        let message = crate::ce::gwe::Message::new(
            0,
            crate::ce::gwe::WM_TIMER,
            id,
            callback.unwrap_or(0),
            self.timers.tick_count(),
        );
        self.record_message_op(
            "set_timer",
            thread_id,
            &message,
            Some(id),
            Some(format!(
                "hwnd=none/requested_id={}/id=0x{id:08x}/period_ms={period_ms}/callback=0x{:08x}",
                requested_id
                    .map(|id| format!("0x{id:08x}"))
                    .unwrap_or_else(|| "auto".to_owned()),
                callback.unwrap_or(0)
            )),
        );
        id
    }

    pub fn kill_timer(&mut self, hwnd: Option<u32>, id: u32) -> bool {
        let thread_id = hwnd
            .and_then(|hwnd| self.gwe.window_thread_process_id(hwnd))
            .map(|(thread_id, _)| thread_id)
            .unwrap_or(0);
        self.kill_timer_for_thread(thread_id, hwnd, id)
    }

    pub fn kill_timer_for_thread(&mut self, thread_id: u32, hwnd: Option<u32>, id: u32) -> bool {
        let ok = self.timers.kill_timer(thread_id, hwnd, id);
        let message =
            crate::ce::gwe::Message::new(hwnd.unwrap_or(0), crate::ce::gwe::WM_TIMER, id, 0, 0);
        self.record_message_op(
            "kill_timer",
            thread_id,
            &message,
            Some(u32::from(ok)),
            Some(format!("hwnd={}/id=0x{id:08x}", format_optional_hwnd(hwnd))),
        );
        ok
    }

    /// A delayed caller A cannot cancel B's same-ID timer after TID reuse.
    pub fn kill_timer_for_birth(&mut self, birth: ThreadBirthKey, id: u32) -> bool {
        if !self.is_live_thread_birth(birth) {
            return false;
        }
        let ok = self.timers.kill_thread_timer_for_birth(birth, id);
        let message = crate::ce::gwe::Message::new(0, crate::ce::gwe::WM_TIMER, id, 0, 0);
        self.record_message_op(
            "kill_timer",
            birth.thread_id(),
            &message,
            Some(u32::from(ok)),
            Some(format!("hwnd=none/id=0x{id:08x}")),
        );
        ok
    }

    pub fn remote_gps_target(&self) -> String {
        self.devices.remote_gps_target().unwrap_or_default()
    }

    pub fn remote_status(&self) -> RemoteStatus {
        self.remote.status(self.remote_gps_target())
    }

    pub fn remote_status_json(&self) -> serde_json::Value {
        self.remote.status_json(self.remote_gps_target())
    }

    pub fn set_remote_server(&mut self, server: RemoteServer) {
        self.remote_server = Some(server);
        self.publish_remote_server_status();
    }

    pub fn publish_remote_server_status(&self) {
        if let Some(server) = self.remote_server.as_ref() {
            server.publish_status(&self.remote_status());
            server.publish_recent_logs(self.remote.recent_log_lines(4096));
        }
    }

    pub fn drain_remote_server_control_messages(&mut self) -> usize {
        self.drain_remote_server_control_messages_with_targets()
            .handled
    }

    pub fn drain_remote_server_control_messages_with_targets(
        &mut self,
    ) -> RemoteServerControlDrain {
        let Some(server) = self.remote_server.clone() else {
            return RemoteServerControlDrain::default();
        };
        let messages = server.drain_control_messages();
        if messages.is_empty() {
            return RemoteServerControlDrain::default();
        }
        let mut applied = 0;
        let touch_before = self.remote.touch_event_count();
        for message in messages {
            self.dispatch_remote_control_message(&message);
            applied += 1;
        }
        let touch_after_dispatch = self.remote.touch_event_count();
        let drain = self.drain_remote_input_to_active_window_detailed();
        if let Some(server) = self.remote_server.as_ref() {
            server.publish_debug_text(
                "remote-input",
                format!(
                    "messages={} route=active touch_before={} touch_after_dispatch={} posted={} touch_after={} targets={}\n",
                    applied,
                    touch_before,
                    touch_after_dispatch,
                    drain.posted,
                    self.remote.touch_event_count(),
                    drain.detail
                ),
            );
        }
        self.publish_remote_server_status();
        RemoteServerControlDrain {
            handled: applied + drain.posted,
            target_thread_ids: drain.target_thread_ids,
        }
    }

    pub fn drain_remote_server_control_messages_to_thread_window(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
    ) -> usize {
        self.drain_remote_server_control_messages_to_thread_window_with_targets(thread_id, hwnd)
            .handled
    }

    pub fn drain_remote_server_control_messages_to_thread_window_with_targets(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
    ) -> RemoteServerControlDrain {
        let Some(server) = self.remote_server.clone() else {
            return RemoteServerControlDrain::default();
        };
        let messages = server.drain_control_messages();
        if messages.is_empty() {
            return RemoteServerControlDrain::default();
        }
        let mut applied = 0;
        let touch_before = self.remote.touch_event_count();
        for message in messages {
            self.dispatch_remote_control_message(&message);
            applied += 1;
        }
        let touch_after_dispatch = self.remote.touch_event_count();
        let drain = self.drain_remote_input_to_thread_window_detailed(thread_id, hwnd);
        if let Some(server) = self.remote_server.as_ref() {
            server.publish_debug_text(
                "remote-input",
                format!(
                    "messages={} route=thread thread_id={} hwnd={} touch_before={} touch_after_dispatch={} posted={} touch_after={} targets={}\n",
                    applied,
                    thread_id,
                    hwnd.map(|hwnd| format!("0x{hwnd:08x}"))
                        .unwrap_or_else(|| "any".to_owned()),
                    touch_before,
                    touch_after_dispatch,
                    drain.posted,
                    self.remote.touch_event_count(),
                    drain.detail
                ),
            );
        }
        self.publish_remote_server_status();
        RemoteServerControlDrain {
            handled: applied + drain.posted,
            target_thread_ids: drain.target_thread_ids,
        }
    }

    pub fn dispatch_remote_control_message(
        &mut self,
        message: &serde_json::Value,
    ) -> serde_json::Value {
        let gps_target = self.remote_gps_target();
        let serial_before = self.remote.serial_byte_count();
        let response = self.remote.dispatch_control_message(message, gps_target);
        if self.remote.serial_byte_count() > serial_before {
            self.drain_remote_serial_to_matching_open_device();
            self.queue_all_serial_read_wake_candidates();
            self.queue_all_serial_event_wake_candidates();
        }
        response
    }

    pub fn read_remote_serial_bytes(&mut self, max_bytes: usize) -> Vec<u8> {
        self.remote.read_serial_bytes(max_bytes)
    }

    /// A host event may use a HWND only while its frozen creation owner is
    /// still the live recipient. Numeric window fields are compatibility
    /// projections and cannot authorize a reused guest TID.
    fn live_remote_window_target(&self, hwnd: u32) -> Option<(u32, WindowDeliveryOwner)> {
        let window = self.gwe.window(hwnd)?;
        if window.destroyed {
            return None;
        }
        let thread_id = window.thread_id;
        let owner = window.delivery_owner();
        match owner {
            WindowDeliveryOwner::Birth { thread, process }
                if thread.thread_id() == thread_id
                    && self.is_live_thread_birth(thread)
                    && self.handles.live_thread_creation_owner_by_id(thread_id)
                        == Some(process)
                    && self.handles.process_execution_live_by_birth(process) =>
            {
                Some((thread_id, owner))
            }
            WindowDeliveryOwner::Birth { .. } => None,
            // Direct fixture windows and the desktop retain their explicit
            // compatibility route; neither is silently assigned a new birth.
            WindowDeliveryOwner::LegacyUnknown { .. } | WindowDeliveryOwner::SystemDesktop => {
                Some((thread_id, owner))
            }
        }
    }

    pub fn drain_remote_input_to_gwe(&mut self, thread_id: u32, hwnd: u32) -> usize {
        // Even a destroyed explicit target consumes and drops its queued host
        // events, so they cannot replay into a later live window.
        self.drain_remote_input_to_target(thread_id, Some(hwnd), RemoteTouchTargeting::Explicit)
            .posted
    }

    fn drain_remote_input_to_target(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        touch_targeting: RemoteTouchTargeting,
    ) -> RemoteInputDrain {
        let input_events = self.remote.drain_input_events();
        let base_time_ms = self.timers.tick_count();
        let first_input_ms = input_events
            .iter()
            .map(|event| event.enqueued_at_ms())
            .min()
            .unwrap_or(0);
        let mut posted = 0;
        let mut details = Vec::new();
        let mut target_thread_ids = Vec::new();

        for input in input_events {
            match input.payload {
                RemoteInputPayload::Touch(event) => {
                    let time_ms =
                        remote_event_time_ms(base_time_ms, first_input_ms, event.enqueued_at_ms);
                    let point = Point {
                        x: event.x,
                        y: event.y,
                    };
                    if touch_targeting == RemoteTouchTargeting::DesktopHitTest
                        && let Some(stale_capture) = self
                            .gwe
                            .get_capture()
                            .filter(|captured| self.live_remote_window_target(*captured).is_none())
                    {
                        if event.message == WM_LBUTTONDOWN {
                            // A fresh gesture may retarget by screen point. Clear only
                            // the same still-stale capture; the kernel release path
                            // would post WM_CAPTURECHANGED to the exited A owner.
                            if self.gwe.get_capture() == Some(stale_capture)
                                && self.live_remote_window_target(stale_capture).is_none()
                            {
                                self.gwe.release_capture();
                            }
                        } else {
                            // An old MOVE/UP has no new DOWN on B. Retain the stale
                            // capture until a fresh DOWN, so later orphan phases also
                            // drop instead of becoming a lone event for B.
                            details.push(format!(
                                "drop:stale-capture=0x{stale_capture:08x}/msg=0x{:04x}",
                                event.message
                            ));
                            self.record_message_trace(MessageTraceRecord {
                                op: "remote_touch_drop",
                                thread_id,
                                hwnd: Some(stale_capture),
                                msg: Some(event.message),
                                wparam: None,
                                lparam: None,
                                screen_pos: Some(make_lparam(point.x, point.y)),
                                source: None,
                                result: Some(0),
                                detail: Some("stale captured owner; orphan touch phase".to_owned()),
                            });
                            continue;
                        }
                    }
                    let target = match touch_targeting {
                        RemoteTouchTargeting::Explicit => hwnd,
                        RemoteTouchTargeting::DesktopHitTest => self
                            .gwe
                            .get_capture()
                            .filter(|captured| self.live_remote_window_target(*captured).is_some())
                            .or_else(|| {
                                self.gwe
                                    .window_from_point(point)
                                    .filter(|hit| self.live_remote_window_target(*hit).is_some())
                            })
                            .or_else(|| {
                                hwnd.filter(|hint| {
                                    matches!(
                                        self.live_remote_window_target(*hint),
                                        Some((_, WindowDeliveryOwner::Birth { .. }))
                                    ) && self.gwe.is_window_visible(*hint)
                                        && self.gwe.is_window_enabled(*hint)
                                        && self
                                            .gwe
                                            .get_window_rect(*hint)
                                            .is_some_and(|rect| rect.contains_point(point))
                                })
                            }),
                    };
                    let Some((target, target_thread_id, delivery_owner)) =
                        target.and_then(|target| {
                            self.live_remote_window_target(target).map(
                                |(target_thread_id, delivery_owner)| {
                                    (target, target_thread_id, delivery_owner)
                                },
                            )
                        })
                    else {
                        details.push(format!(
                            "drop:msg=0x{:04x}/screen={},{}",
                            event.message, point.x, point.y
                        ));
                        self.record_message_trace(MessageTraceRecord {
                            op: "remote_touch_drop",
                            thread_id,
                            hwnd: target.or(hwnd),
                            msg: Some(event.message),
                            wparam: None,
                            lparam: None,
                            screen_pos: Some(make_lparam(point.x, point.y)),
                            source: None,
                            result: Some(0),
                            detail: Some(
                                if touch_targeting == RemoteTouchTargeting::Explicit
                                    && target.is_some()
                                {
                                    "explicit target stale"
                                } else {
                                    "no live target"
                                }
                                .to_owned(),
                            ),
                        });
                        continue;
                    };
                    if event.message == WM_LBUTTONDOWN {
                        let _ = self.set_focus(Some(target));
                    }
                    let client = self.gwe.screen_to_client(target, point).unwrap_or(point);
                    let wparam = if event.message == WM_LBUTTONDOWN || event.message == WM_MOUSEMOVE
                    {
                        1
                    } else {
                        0
                    };
                    let posted_to_owner = self.post_gwe_message_with_window_owner(
                        target_thread_id,
                        Some(delivery_owner),
                        Message::new(
                            target,
                            event.message,
                            wparam,
                            make_lparam(client.x, client.y),
                            time_ms,
                        )
                        .with_mouse_pos(make_lparam(point.x, point.y)),
                    );
                    if !posted_to_owner {
                        details.push(format!(
                            "drop:msg=0x{:04x}/hwnd=0x{target:08x}",
                            event.message
                        ));
                        continue;
                    }
                    if !target_thread_ids.contains(&target_thread_id) {
                        target_thread_ids.push(target_thread_id);
                    }
                    details.push(format!(
                "touch:hwnd=0x{target:08x}/thread={target_thread_id}/msg=0x{:04x}/client={},{} /screen={},{}",
                event.message, client.x, client.y, point.x, point.y
            ));
                    self.record_message_trace(MessageTraceRecord {
                        op: "remote_touch_target",
                        thread_id: target_thread_id,
                        hwnd: Some(target),
                        msg: Some(event.message),
                        wparam: Some(wparam),
                        lparam: Some(make_lparam(client.x, client.y)),
                        screen_pos: Some(make_lparam(point.x, point.y)),
                        source: Some(crate::ce::gwe::MSGSRC_SOFTWARE_POST),
                        result: Some(1),
                        detail: Some(
                            match touch_targeting {
                                RemoteTouchTargeting::Explicit => "explicit-target",
                                RemoteTouchTargeting::DesktopHitTest => "desktop-hit-test",
                            }
                            .to_owned(),
                        ),
                    });
                    posted += 1;
                }
                RemoteInputPayload::Key(event) => {
                    // A host wait hint and mouse capture have no keyboard authority.
                    // Resolve focus first, then the active window, for this event's
                    // current owner birth. The explicit API retains its supplied target.
                    let key_target = if touch_targeting == RemoteTouchTargeting::Explicit {
                        hwnd
                    } else {
                        let live_key_window = |candidate| {
                            self.gwe.is_window_enabled(candidate)
                                && self.live_remote_window_target(candidate).is_some_and(
                                    |(_, owner)| owner != WindowDeliveryOwner::SystemDesktop,
                                )
                        };
                        self.gwe
                            .get_focus()
                            .filter(|candidate| live_key_window(*candidate))
                            .or_else(|| {
                                self.gwe
                                    .get_stored_active_window()
                                    .filter(|candidate| live_key_window(*candidate))
                            })
                    };
                    let time_ms =
                        remote_event_time_ms(base_time_ms, first_input_ms, event.enqueued_at_ms);
                    let Some(key_target) = key_target else {
                        details.push(format!(
                            "drop:key=0x{:02x}/msg=0x{:04x}",
                            event.vk, event.message
                        ));
                        self.record_message_trace(MessageTraceRecord {
                            op: "remote_key_drop",
                            thread_id,
                            hwnd: key_target,
                            msg: Some(event.message),
                            wparam: Some(event.vk),
                            lparam: Some(1),
                            screen_pos: None,
                            source: Some(crate::ce::gwe::MSGSRC_HARDWARE_KEYBOARD),
                            result: Some(0),
                            detail: Some("no live target".to_owned()),
                        });
                        continue;
                    };
                    let Some((target_thread_id, delivery_owner)) =
                        self.live_remote_window_target(key_target)
                    else {
                        details.push(format!(
                            "drop:key=0x{:02x}/msg=0x{:04x}/hwnd=0x{key_target:08x}",
                            event.vk, event.message
                        ));
                        self.record_message_trace(MessageTraceRecord {
                            op: "remote_key_drop",
                            thread_id,
                            hwnd: Some(key_target),
                            msg: Some(event.message),
                            wparam: Some(event.vk),
                            lparam: Some(1),
                            screen_pos: None,
                            source: Some(crate::ce::gwe::MSGSRC_HARDWARE_KEYBOARD),
                            result: Some(0),
                            detail: Some("no live target".to_owned()),
                        });
                        continue;
                    };
                    if !self.post_gwe_message_with_window_owner(
                        target_thread_id,
                        Some(delivery_owner),
                        Message::new(key_target, event.message, event.vk, 1, time_ms)
                            .with_source(crate::ce::gwe::MSGSRC_HARDWARE_KEYBOARD),
                    ) {
                        details.push(format!(
                            "drop:key=0x{:02x}/msg=0x{:04x}/hwnd=0x{key_target:08x}",
                            event.vk, event.message
                        ));
                        continue;
                    }
                    if !target_thread_ids.contains(&target_thread_id) {
                        target_thread_ids.push(target_thread_id);
                    }
                    details.push(format!(
                "key:hwnd=0x{key_target:08x}/thread={target_thread_id}/msg=0x{:04x}/vk=0x{:02x}",
                event.message, event.vk
            ));
                    self.record_message_trace(MessageTraceRecord {
                        op: "remote_key_target",
                        thread_id: target_thread_id,
                        hwnd: Some(key_target),
                        msg: Some(event.message),
                        wparam: Some(event.vk),
                        lparam: Some(1),
                        screen_pos: None,
                        source: Some(crate::ce::gwe::MSGSRC_HARDWARE_KEYBOARD),
                        result: Some(1),
                        detail: (target_thread_id != thread_id)
                            .then(|| format!("drain_thread={thread_id}")),
                    });
                    posted += 1;
                }
            }
        }

        RemoteInputDrain {
            posted,
            target_thread_ids,
            detail: if details.is_empty() {
                "none".to_owned()
            } else {
                details.join(",")
            },
        }
    }

    pub fn drain_remote_input_to_thread_window(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
    ) -> usize {
        self.drain_remote_input_to_thread_window_detailed(thread_id, hwnd)
            .posted
    }

    fn drain_remote_input_to_thread_window_detailed(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
    ) -> RemoteInputDrain {
        // The host's blocked-wait/modal HWND is only a bounded fallback for
        // coordinate input; it is not a queue filter or routing authority.
        self.drain_remote_input_to_target(thread_id, hwnd, RemoteTouchTargeting::DesktopHitTest)
    }

    pub fn drain_remote_input_to_active_window(&mut self) -> usize {
        self.drain_remote_input_to_active_window_detailed().posted
    }

    fn drain_remote_input_to_active_window_detailed(&mut self) -> RemoteInputDrain {
        // Drain even when no window is active: undeliverable input must not
        // remain queued and replay into a window created later.
        self.drain_remote_input_to_target(0, None, RemoteTouchTargeting::DesktopHitTest)
    }

    pub fn wave_out_open(&mut self, format: WaveFormat) -> std::result::Result<u32, MmResult> {
        self.audio.wave_out_open(format)
    }

    pub fn wave_out_write(&mut self, id: u32, buffer: WaveBuffer) -> MmResult {
        self.audio.wave_out_write(id, buffer)
    }

    fn post_window_visibility_message(&mut self, hwnd: u32, before: bool, after: bool) {
        if before != after {
            self.post_window_message(hwnd, WM_SHOWWINDOW, u32::from(after), 0);
        }
    }

    fn direct_window_visible(&self, hwnd: u32) -> bool {
        self.gwe
            .window(hwnd)
            .is_some_and(|window| window.visible && window.style & crate::ce::gwe::WS_VISIBLE != 0)
    }

    fn post_window_rect_messages(
        &mut self,
        hwnd: u32,
        before: Option<Rect>,
        after: Option<Rect>,
        insert_after: u32,
        flags: u32,
        force_window_pos_changed: bool,
    ) {
        let (Some(before), Some(after)) = (before, after) else {
            return;
        };
        if before != after || force_window_pos_changed {
            let payload = self
                .gwe
                .window_pos_for_rect(hwnd, after, insert_after, flags)
                .and_then(|payload| self.queue_window_pos_payload(hwnd, payload));
            if let Some((owner, lparam)) = payload {
                if !self.post_window_typed_payload_message(
                    hwnd,
                    WM_WINDOWPOSCHANGED,
                    0,
                    lparam,
                    owner,
                ) {
                    let _ = self.release_message_pointer_payload(owner, lparam);
                }
            } else {
                self.post_window_message(hwnd, WM_WINDOWPOSCHANGED, 0, 0);
            }
        }
        let moved = before.left != after.left || before.top != after.top;
        let sized = before.width() != after.width() || before.height() != after.height();
        if moved || sized {
            if self.direct_window_visible(hwnd) {
                self.post_size_move(hwnd, after, moved, sized);
            } else {
                let _ = self.gwe.mark_pending_size_move(hwnd, moved, sized);
            }
        }
    }

    fn post_pending_size_move(&mut self, hwnd: u32) {
        let Some((rect, moved, sized)) = self.gwe.take_pending_size_move(hwnd) else {
            return;
        };
        self.post_size_move(hwnd, rect, moved, sized);
    }

    fn post_size_move(&mut self, hwnd: u32, rect: Rect, moved: bool, sized: bool) {
        if moved {
            self.post_window_message(hwnd, WM_MOVE, 0, make_lparam_i16(rect.left, rect.top));
        }
        if sized {
            self.post_window_message(
                hwnd,
                WM_SIZE,
                0,
                make_lparam_i16(rect.width(), rect.height()),
            );
        }
    }

    fn queue_window_pos_payload(
        &mut self,
        hwnd: u32,
        payload: WindowPos,
    ) -> Option<(ProcessBirthKey, u32)> {
        let owner = self.recipient_birth_for_window(hwnd)?;
        let Some(ptr) = self.memory.heap_alloc(owner, PROCESS_HEAP_HANDLE, 0, 28) else {
            return None;
        };
        if self.gwe.insert_message_pointer_payload(
            owner,
            ptr,
            MessagePointerPayload::WindowPos(payload),
        ) {
            self.typed_payload_recipients.insert((owner, ptr), hwnd);
            Some((owner, ptr))
        } else {
            let _ = self.memory.heap_free(owner, PROCESS_HEAP_HANDLE, 0, ptr);
            None
        }
    }

    fn queue_shell_notification_payload(
        &mut self,
        hwnd: u32,
        payload: ShellNotificationMessage,
    ) -> Option<(ProcessBirthKey, u32)> {
        let owner = self.recipient_birth_for_window(hwnd)?;
        let link_bytes = payload
            .link
            .as_ref()
            .map(|link| (link.encode_utf16().count() + 1) * 2)
            .unwrap_or(0);
        let alloc_size = 28usize.saturating_add(link_bytes);
        let Some(ptr) = self
            .memory
            .heap_alloc(owner, PROCESS_HEAP_HANDLE, 0, alloc_size as u32)
        else {
            return None;
        };
        if self.gwe.insert_message_pointer_payload(
            owner,
            ptr,
            MessagePointerPayload::ShellNotification(payload),
        ) {
            self.typed_payload_recipients.insert((owner, ptr), hwnd);
            Some((owner, ptr))
        } else {
            let _ = self.memory.heap_free(owner, PROCESS_HEAP_HANDLE, 0, ptr);
            None
        }
    }

    fn queue_notify_icon_payload(
        &mut self,
        hwnd: u32,
        payload: NotifyIconMessage,
        alloc_size: u32,
    ) -> Option<(ProcessBirthKey, u32)> {
        let owner = self.recipient_birth_for_window(hwnd)?;
        let Some(ptr) = self
            .memory
            .heap_alloc(owner, PROCESS_HEAP_HANDLE, 0, alloc_size)
        else {
            return None;
        };
        if self.gwe.insert_message_pointer_payload(
            owner,
            ptr,
            MessagePointerPayload::NotifyIcon(payload),
        ) {
            self.typed_payload_recipients.insert((owner, ptr), hwnd);
            Some((owner, ptr))
        } else {
            let _ = self.memory.heap_free(owner, PROCESS_HEAP_HANDLE, 0, ptr);
            None
        }
    }

    fn post_shell_file_change_notifications(
        &mut self,
        event_id: u32,
        path1: Option<&str>,
        path2: Option<&str>,
    ) -> usize {
        let registrations: Vec<_> = self.shell.change_notifications().cloned().collect();
        let mut posted = 0usize;
        for registration in registrations {
            if registration.event_mask & event_id == 0
                || !self.gwe.is_window(registration.hwnd)
                || !shell_change_registration_matches(&registration, path1, path2)
            {
                continue;
            }
            let data_path = path2.or(path1);
            let (attributes, file_size) = data_path
                .and_then(|path| self.file_attributes_w(path).ok())
                .map(|data| {
                    (
                        data.attributes,
                        data.file_size.try_into().unwrap_or(u32::MAX),
                    )
                })
                .unwrap_or((0, 0));
            let payload = FileChangeNotificationMessage {
                event_id,
                flags: shell_change_notify_payload_flags(registration.notify_flags),
                path1: path1.map(normalize_shell_change_path),
                path2: path2.map(normalize_shell_change_path),
                pidl1: path1.map(shell_change_path_pidl),
                pidl2: path2.map(shell_change_path_pidl),
                attributes,
                file_size,
            };
            let Some((owner, lparam)) =
                self.queue_shell_file_change_payload(registration.hwnd, payload)
            else {
                continue;
            };
            if self.post_window_typed_payload_message(
                registration.hwnd,
                WM_FILECHANGEINFO,
                event_id,
                lparam,
                owner,
            ) {
                posted = posted.saturating_add(1);
            } else {
                let _ = self.release_message_pointer_payload(owner, lparam);
            }
        }
        posted
    }

    fn signal_file_change_notifications(
        &mut self,
        event_id: u32,
        path1: Option<&str>,
        path2: Option<&str>,
    ) -> usize {
        let Some(required_filter) = file_notify_filter_for_shell_event(event_id) else {
            return 0;
        };
        let path1_volume = path1.and_then(|path| self.files.volume_root_for_guest_path(path));
        let path2_volume = path2.and_then(|path| self.files.volume_root_for_guest_path(path));
        let mut handles = Vec::new();
        for (handle, object) in self.handles.iter_mut() {
            let KernelObject::FileChangeNotification(notification) = object else {
                continue;
            };
            if notification.notify_filter & required_filter == 0
                || !file_change_notification_matches(
                    notification,
                    path1.map(|path| (path, path1_volume.as_deref())),
                    path2.map(|path| (path, path2_volume.as_deref())),
                )
            {
                continue;
            }
            if notification.notify_filter & FILE_NOTIFY_CHANGE_CEGETINFO != 0 {
                let records = file_change_records_for_event(event_id, notification, path1, path2);
                append_file_change_records(&mut notification.pending, records);
                notification.pending_signal_count = notification.pending.len();
                notification.signaled = notification.pending_signal_count > 0;
            } else {
                notification.pending_signal_count =
                    notification.pending_signal_count.saturating_add(1);
                notification.signaled = true;
            }
            if notification.signaled {
                handles.push(handle);
            }
        }
        for handle in &handles {
            self.queue_object_wake_candidates(*handle);
        }
        handles.len()
    }

    fn signal_file_move_notifications(
        &mut self,
        existing_path: &str,
        new_path: &str,
        is_directory: bool,
        same_parent: bool,
    ) -> usize {
        let required_filter = if is_directory {
            FILE_NOTIFY_CHANGE_DIR_NAME
        } else {
            FILE_NOTIFY_CHANGE_FILE_NAME
        };
        let (old_action, new_action) = if same_parent {
            (FILE_ACTION_RENAMED_OLD_NAME, FILE_ACTION_RENAMED_NEW_NAME)
        } else {
            (FILE_ACTION_REMOVED, FILE_ACTION_ADDED)
        };
        self.signal_file_change_notification_path(required_filter, old_action, existing_path)
            + self.signal_file_change_notification_path(required_filter, new_action, new_path)
    }

    fn signal_file_change_notification_path(
        &mut self,
        required_filter: u32,
        action: u32,
        path: &str,
    ) -> usize {
        let path_volume = self.files.volume_root_for_guest_path(path);
        let mut handles = Vec::new();
        for (handle, object) in self.handles.iter_mut() {
            let KernelObject::FileChangeNotification(notification) = object else {
                continue;
            };
            if notification.notify_filter & required_filter == 0
                || !file_change_volume_matches(notification, path_volume.as_deref())
                || !shell_change_path_matches(
                    &notification.watch_path,
                    path,
                    notification.recursive,
                )
            {
                continue;
            }
            if notification.notify_filter & FILE_NOTIFY_CHANGE_CEGETINFO != 0 {
                let mut record = FileChangeRecord {
                    action,
                    path: file_change_relative_path(&notification.watch_path, path),
                };
                apply_ce_current_directory_removal_record(
                    &mut record,
                    required_filter == FILE_NOTIFY_CHANGE_DIR_NAME,
                );
                append_file_change_records(&mut notification.pending, [record]);
                notification.pending_signal_count = notification.pending.len();
                notification.signaled = notification.pending_signal_count > 0;
            } else {
                notification.pending_signal_count =
                    notification.pending_signal_count.saturating_add(1);
                notification.signaled = true;
            }
            if notification.signaled {
                handles.push(handle);
            }
        }
        for handle in &handles {
            self.queue_object_wake_candidates(*handle);
        }
        handles.len()
    }

    fn signal_file_change_notifications_for_removed_mount(&mut self, volume_root: &str) {
        let volume_root = normalize_shell_change_path(volume_root);
        let volume_prefix = format!("{volume_root}\\");
        let prefix_len = volume_prefix.len();
        let mut handles = Vec::new();
        for (handle, object) in self.handles.iter_mut() {
            let KernelObject::FileChangeNotification(notification) = object else {
                continue;
            };
            let watch = normalize_shell_change_path(&notification.watch_path);
            // Signal handles whose watch path is the volume root or any subpath of it.
            // append_file_change_records deduplicates consecutive same records, so handles
            // already signaled by the SHCNE_DRIVEREMOVED call won't accumulate duplicates.
            let (matches, relative_path) = if watch.eq_ignore_ascii_case(&volume_root) {
                (true, String::new())
            } else if watch
                .get(..prefix_len)
                .is_some_and(|head| head.eq_ignore_ascii_case(&volume_prefix))
            {
                (true, watch[prefix_len..].to_owned())
            } else {
                (false, String::new())
            };
            if !matches {
                continue;
            }
            let record = FileChangeRecord {
                action: FILE_ACTION_REMOVED,
                path: relative_path,
            };
            if notification.notify_filter & FILE_NOTIFY_CHANGE_CEGETINFO != 0 {
                append_file_change_records(&mut notification.pending, std::iter::once(record));
                notification.pending_signal_count = notification.pending.len();
                notification.signaled = notification.pending_signal_count > 0;
            } else {
                notification.pending_signal_count =
                    notification.pending_signal_count.saturating_add(1);
                notification.signaled = true;
            }
            if notification.signaled {
                handles.push(handle);
            }
        }
        for handle in &handles {
            self.queue_object_wake_candidates(*handle);
        }
    }

    fn queue_shell_file_change_payload(
        &mut self,
        hwnd: u32,
        payload: FileChangeNotificationMessage,
    ) -> Option<(ProcessBirthKey, u32)> {
        let owner = self.recipient_birth_for_window(hwnd)?;
        let item_bytes = if payload.flags == SHCNF_IDLIST {
            payload
                .pidl1
                .as_ref()
                .into_iter()
                .chain(payload.pidl2.as_ref())
                .map(Vec::len)
                .sum::<usize>()
        } else {
            payload
                .path1
                .as_ref()
                .into_iter()
                .chain(payload.path2.as_ref())
                .map(|path| (path.encode_utf16().count() + 1) * 2)
                .sum::<usize>()
        };
        let alloc_size = FILECHANGENOTIFY_BASE_SIZE.saturating_add(item_bytes);
        let Some(ptr) = self
            .memory
            .heap_alloc(owner, PROCESS_HEAP_HANDLE, 0, alloc_size as u32)
        else {
            return None;
        };
        if self.gwe.insert_message_pointer_payload(
            owner,
            ptr,
            MessagePointerPayload::FileChangeNotification(payload),
        ) {
            self.typed_payload_recipients.insert((owner, ptr), hwnd);
            Some((owner, ptr))
        } else {
            let _ = self.memory.heap_free(owner, PROCESS_HEAP_HANDLE, 0, ptr);
            None
        }
    }

    fn post_window_typed_payload_message(
        &mut self,
        hwnd: u32,
        msg: u32,
        wparam: u32,
        lparam: u32,
        owner: ProcessBirthKey,
    ) -> bool {
        if self.recipient_birth_for_window(hwnd) != Some(owner)
            || self.gwe.message_pointer_payload(owner, lparam).is_none()
        {
            return false;
        }
        let Some(window) = self.gwe.window(hwnd) else {
            return false;
        };
        let thread_id = window.thread_id;
        let delivery_owner = window.delivery_owner();
        let message = Message::new(hwnd, msg, wparam, lparam, self.timers.tick_count())
            .with_typed_payload_owner(owner);
        if let WindowDeliveryOwner::Birth { thread, .. } = delivery_owner {
            if !self
                .gwe
                .post_message_for_window_birth(hwnd, thread, message.clone())
            {
                return false;
            }
            self.record_message_op("post_message", thread_id, &message, Some(1), None);
            self.scheduler
                .queue_message_wake_candidates_for_birth(thread);
            true
        } else {
            self.post_gwe_message(thread_id, message);
            true
        }
    }

    fn post_window_message(&mut self, hwnd: u32, msg: u32, wparam: u32, lparam: u32) {
        let Some(window) = self.gwe.window(hwnd) else {
            return;
        };
        if matches!(window.delivery_owner(), WindowDeliveryOwner::Birth { .. }) {
            let _ = self.post_message_w(hwnd, msg, wparam, lparam);
            return;
        }
        let thread_id = window.thread_id;
        let message = Message::new(hwnd, msg, wparam, lparam, self.timers.tick_count());
        self.post_gwe_message(thread_id, message);
    }

    fn clear_timer_message_pending(&mut self, thread_id: u32, message: &Message) {
        if message.msg != crate::ce::gwe::WM_TIMER {
            return;
        }
        if let Some(origin) = message.timer_origin {
            if self.is_live_thread_birth(origin.birth) {
                let _ = self.timers.clear_pending_for_origin(origin);
            }
            return;
        }
        let hwnd = (message.hwnd != 0).then_some(message.hwnd);
        let _ = self
            .timers
            .clear_pending_message(thread_id, hwnd, message.wparam);
    }
}

fn format_filter_detail(hwnd: Option<u32>, min_msg: u32, max_msg: u32) -> String {
    format!(
        "filter_hwnd={} min=0x{min_msg:08x} max=0x{max_msg:08x}",
        hwnd.map_or_else(|| "any".to_owned(), |hwnd| format!("0x{hwnd:08x}"))
    )
}

fn format_optional_hwnd(hwnd: Option<u32>) -> String {
    hwnd.map_or_else(|| "none".to_owned(), |hwnd| format!("0x{hwnd:08x}"))
}

fn window_lifecycle_detail(window: &crate::ce::gwe::Window) -> String {
    format!(
        "class={}/title={}/tid={}/pid={}/parent={}/owner={}/vis={}/dead={}/style=0x{:08x}/ex=0x{:08x}/rect={},{}-{},{}",
        window.class_name,
        window.title,
        window.thread_id,
        window.process_id,
        window
            .parent
            .map_or_else(|| "none".to_owned(), |hwnd| format!("0x{hwnd:08x}")),
        window
            .owner
            .map_or_else(|| "none".to_owned(), |hwnd| format!("0x{hwnd:08x}")),
        window.visible,
        window.destroyed,
        window.style,
        window.ex_style,
        window.rect.left,
        window.rect.top,
        window.rect.right,
        window.rect.bottom
    )
}

fn file_trace_preview(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }
    let mut preview = String::new();
    for &byte in bytes.iter().take(FILE_TRACE_PREVIEW_LIMIT) {
        match byte {
            b'\r' => preview.push_str("\\r"),
            b'\n' => preview.push_str("\\n"),
            b'\t' => preview.push_str("\\t"),
            0x20..=0x7e => preview.push(byte as char),
            _ => preview.push_str(&format!("\\x{byte:02x}")),
        }
    }
    if bytes.len() > FILE_TRACE_PREVIEW_LIMIT {
        preview.push_str("...");
    }
    Some(preview)
}

fn file_read_trace_preview(
    start_position: Option<u64>,
    end_position: Option<u64>,
    bytes: &[u8],
) -> Option<String> {
    let mut parts = Vec::new();
    if let (Some(start), Some(end)) = (start_position, end_position) {
        parts.push(format!("pos={start}..{end}"));
    }
    if let Some(preview) = file_trace_preview(bytes) {
        parts.push(format!("bytes={preview}"));
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}

fn is_file_open_trace(op: &str) -> bool {
    matches!(op, "CreateFileW" | "CreateFileWArg" | "FindFirstFileW")
}

fn shell_change_registration_matches(
    registration: &ShellChangeNotifyRegistration,
    path1: Option<&str>,
    path2: Option<&str>,
) -> bool {
    let Some(watch_dir) = registration.watch_dir.as_deref() else {
        return true;
    };
    let watch_dir = normalize_shell_change_path(watch_dir);
    [path1, path2]
        .into_iter()
        .flatten()
        .any(|path| shell_change_path_matches(&watch_dir, path, registration.recursive))
}

fn file_change_notification_matches(
    notification: &FileChangeNotificationObject,
    path1: Option<(&str, Option<&str>)>,
    path2: Option<(&str, Option<&str>)>,
) -> bool {
    [path1, path2]
        .into_iter()
        .flatten()
        .any(|(path, volume_root)| {
            file_change_volume_matches(notification, volume_root)
                && shell_change_path_matches(&notification.watch_path, path, notification.recursive)
        })
}

fn file_change_volume_matches(
    notification: &FileChangeNotificationObject,
    event_volume_root: Option<&str>,
) -> bool {
    notification.watch_path == "\\" || notification.volume_root.as_deref() == event_volume_root
}

fn file_notify_filter_for_shell_event(event_id: u32) -> Option<u32> {
    Some(match event_id {
        SHCNE_RENAMEITEM | SHCNE_CREATE | SHCNE_DELETE => {
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CREATION
        }
        SHCNE_MKDIR | SHCNE_RMDIR | SHCNE_DRIVEADD | SHCNE_DRIVEREMOVED => {
            FILE_NOTIFY_CHANGE_DIR_NAME | FILE_NOTIFY_CHANGE_CREATION
        }
        SHCNE_ATTRIBUTES => FILE_NOTIFY_CHANGE_ATTRIBUTES,
        SHCNE_UPDATEITEM => FILE_NOTIFY_CHANGE_LAST_WRITE | FILE_NOTIFY_CHANGE_SIZE,
        _ => return None,
    })
}

fn file_change_records_for_event(
    event_id: u32,
    notification: &FileChangeNotificationObject,
    path1: Option<&str>,
    path2: Option<&str>,
) -> Vec<FileChangeRecord> {
    match event_id {
        SHCNE_RENAMEITEM => [
            path1.map(|path| (FILE_ACTION_RENAMED_OLD_NAME, path)),
            path2.map(|path| (FILE_ACTION_RENAMED_NEW_NAME, path)),
        ]
        .into_iter()
        .flatten()
        .filter(|(_, path)| {
            shell_change_path_matches(&notification.watch_path, path, notification.recursive)
        })
        .map(|(action, path)| FileChangeRecord {
            action,
            path: file_change_relative_path(&notification.watch_path, path),
        })
        .collect(),
        SHCNE_CREATE | SHCNE_MKDIR | SHCNE_DRIVEADD => path1
            .into_iter()
            .filter(|path| {
                shell_change_path_matches(&notification.watch_path, path, notification.recursive)
            })
            .map(|path| FileChangeRecord {
                action: FILE_ACTION_ADDED,
                path: file_change_relative_path(&notification.watch_path, path),
            })
            .collect(),
        SHCNE_DELETE | SHCNE_RMDIR | SHCNE_DRIVEREMOVED => path1
            .into_iter()
            .filter(|path| {
                shell_change_path_matches(&notification.watch_path, path, notification.recursive)
            })
            .map(|path| {
                let mut record = FileChangeRecord {
                    action: FILE_ACTION_REMOVED,
                    path: file_change_relative_path(&notification.watch_path, path),
                };
                apply_ce_current_directory_removal_record(&mut record, event_id == SHCNE_RMDIR);
                record
            })
            .collect(),
        SHCNE_ATTRIBUTES | SHCNE_UPDATEITEM => path1
            .into_iter()
            .filter(|path| {
                shell_change_path_matches(&notification.watch_path, path, notification.recursive)
            })
            .map(|path| FileChangeRecord {
                action: FILE_ACTION_MODIFIED,
                path: file_change_relative_path(&notification.watch_path, path),
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn apply_ce_current_directory_removal_record(record: &mut FileChangeRecord, directory_event: bool) {
    if !directory_event || !record.path.is_empty() {
        return;
    }
    if matches!(
        record.action,
        FILE_ACTION_REMOVED | FILE_ACTION_RENAMED_OLD_NAME
    ) {
        record.action = FILE_ACTION_REMOVED;
        record.path = "\\".to_owned();
    }
}

fn append_file_change_records(
    pending: &mut Vec<FileChangeRecord>,
    records: impl IntoIterator<Item = FileChangeRecord>,
) {
    for record in records {
        if pending.last() == Some(&record) {
            continue;
        }
        if coalesce_file_change_record(pending, &record) {
            continue;
        }
        pending.push(record);
    }
}

fn coalesce_file_change_record(
    pending: &mut Vec<FileChangeRecord>,
    record: &FileChangeRecord,
) -> bool {
    match record.action {
        FILE_ACTION_REMOVED => coalesce_removed_file_change_record(pending, record),
        FILE_ACTION_MODIFIED => pending
            .iter()
            .rev()
            .take_while(|queued| queued.path.eq_ignore_ascii_case(&record.path))
            .any(|queued| queued.action == FILE_ACTION_REMOVED),
        _ => false,
    }
}

fn coalesce_removed_file_change_record(
    pending: &mut Vec<FileChangeRecord>,
    record: &FileChangeRecord,
) -> bool {
    let mut index = pending.len();
    let mut saw_prior_for_path = false;
    while index > 0 {
        index -= 1;
        if !pending[index].path.eq_ignore_ascii_case(&record.path) {
            if saw_prior_for_path {
                break;
            }
            continue;
        }
        saw_prior_for_path = true;
        match pending[index].action {
            FILE_ACTION_ADDED => {
                pending.remove(index);
                return true;
            }
            FILE_ACTION_MODIFIED | FILE_ACTION_CHANGE_COMPLETED => {
                pending.remove(index);
            }
            FILE_ACTION_REMOVED => return true,
            _ => break,
        }
    }
    false
}

fn file_change_relative_path(watch_path: &str, path: &str) -> String {
    let path = normalize_shell_change_path(path);
    if watch_path == "\\" {
        return path.trim_start_matches('\\').to_owned();
    }
    if path.eq_ignore_ascii_case(watch_path) {
        return String::new();
    }
    let prefix = format!("{watch_path}\\");
    path.get(..prefix.len())
        .filter(|head| head.eq_ignore_ascii_case(&prefix))
        .and_then(|_| path.get(prefix.len()..))
        .unwrap_or(path.as_str())
        .to_owned()
}

fn shell_change_notify_payload_flags(flags: u32) -> u32 {
    if flags == SHCNF_IDLIST {
        SHCNF_IDLIST
    } else {
        SHCNF_PATHW
    }
}

fn shell_change_path_pidl(path: &str) -> Vec<u8> {
    let path = normalize_shell_change_path(path);
    let mut payload = Vec::new();
    for unit in path.encode_utf16().chain(std::iter::once(0)) {
        payload.extend_from_slice(&unit.to_le_bytes());
    }
    let item_size = 2usize.saturating_add(payload.len()).min(u16::MAX as usize) as u16;
    let mut pidl = Vec::with_capacity(usize::from(item_size) + 2);
    pidl.extend_from_slice(&item_size.to_le_bytes());
    pidl.extend_from_slice(&payload[..usize::from(item_size).saturating_sub(2)]);
    pidl.extend_from_slice(&0u16.to_le_bytes());
    pidl
}

fn shell_change_path_matches(watch_dir: &str, path: &str, recursive: bool) -> bool {
    let path = normalize_shell_change_path(path);
    if path.eq_ignore_ascii_case(watch_dir) {
        return true;
    }
    if watch_dir == "\\" && recursive {
        return true;
    }
    if recursive {
        let prefix = format!("{watch_dir}\\");
        return path
            .get(..prefix.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(&prefix));
    }
    let parent = path
        .trim_end_matches('\\')
        .rsplit_once('\\')
        .map(|(parent, _name)| if parent.is_empty() { "\\" } else { parent })
        .unwrap_or("");
    parent.eq_ignore_ascii_case(watch_dir)
}

fn shell_change_parent_path(path: &str) -> String {
    let path = normalize_shell_change_path(path);
    if path == "\\" {
        return path;
    }
    path.trim_end_matches('\\')
        .rsplit_once('\\')
        .map(|(parent, _name)| {
            if parent.is_empty() {
                "\\".to_owned()
            } else {
                parent.to_owned()
            }
        })
        .unwrap_or_else(|| "\\".to_owned())
}

fn normalize_shell_change_path(path: &str) -> String {
    canonical_shell_change_path(path)
}

fn canonical_shell_change_path(path: &str) -> String {
    let path = path.trim().replace('/', "\\");
    let mut parts = Vec::new();
    for part in path.split('\\') {
        match part.trim() {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            part => parts.push(part),
        }
    }
    if parts.is_empty() {
        "\\".to_owned()
    } else {
        format!("\\{}", parts.join("\\"))
    }
}

fn remote_event_time_ms(base_time_ms: u32, first_input_ms: u64, event_input_ms: u64) -> u32 {
    let delta = event_input_ms.saturating_sub(first_input_ms);
    base_time_ms.wrapping_add(delta.min(u64::from(u32::MAX)) as u32)
}

fn make_lparam_i16(low: i32, high: i32) -> u32 {
    ((high as u32) & 0xffff) << 16 | ((low as u32) & 0xffff)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ce::{
            devices::{DeviceBackend, DeviceConfig, DeviceConfigFile, DeviceDefaults, DeviceKind},
            gwe::{
                BS_DEFPUSHBUTTON, WM_ACTIVATE, WM_KILLFOCUS, WM_LBUTTONDOWN, WM_LBUTTONUP,
                WM_SETFOCUS, WS_CHILD, WS_VISIBLE,
            },
            memory::{WritableImageRunAccessError, WritableImageRunIdentity},
            registry::RegistryDump,
        },
        config::{ObjectStoreConfig, RootConfig, RuntimeConfig, StorageConfig},
    };
    use std::path::PathBuf;

    fn runtime_config_with_com7() -> RuntimeConfig {
        RuntimeConfig {
            registry: RegistryDump {
                version: 1,
                source: None,
                keys: BTreeMap::new(),
            },
            devices: DeviceConfigFile {
                version: 1,
                defaults: DeviceDefaults::default(),
                devices: vec![DeviceConfig {
                    guest: "COM7:".to_owned(),
                    kind: DeviceKind::Serial,
                    backend: DeviceBackend::Stub,
                    host: None,
                    remote_gps: true,
                    enabled: true,
                    note: None,
                }],
            },
            storage: StorageConfig {
                root: RootConfig {
                    host_root: PathBuf::from("."),
                },
                object_store: ObjectStoreConfig {
                    total_mbytes: 64,
                    free_mbytes: 32,
                },
                mounts: Vec::new(),
            },
            guest_physical_ram_bytes: crate::config::DEFAULT_GUEST_PHYSICAL_RAM_BYTES,
        }
    }

    #[test]
    fn registered_wait_pin_preparation_rolls_back_and_releases_duplicate_slots() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (handle, _) = kernel.create_guest_thread(0x401000, 0, false);
        let birth = kernel
            .handles
            .thread_birth_key_for_handle(handle)
            .expect("live thread birth");
        let pin = TaskWaitPin::Thread(birth);
        let waiters_before = kernel.scheduler.waiter_count();

        assert_eq!(
            kernel.try_register_blocked_waiter(
                77,
                0,
                vec![handle, 0xdead_beef],
                SchedulerBlockedWaitKind::Kernel,
                0,
                crate::ce::timer::INFINITE,
            ),
            None
        );
        assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(0));
        assert_eq!(kernel.scheduler.waiter_count(), waiters_before);

        let wait_id = kernel.register_blocked_waiter(
            77,
            0,
            vec![handle, handle],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        );
        assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(2));
        assert_eq!(
            kernel.blocked_waiter(wait_id).unwrap().pinned_targets.len(),
            2
        );
        assert!(kernel.remove_blocked_waiter(wait_id).is_some());
        assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(0));
        assert!(kernel.remove_blocked_waiter(wait_id).is_none());
        Ok(())
    }

    #[test]
    fn unpublished_prepared_task_wait_releases_every_pin_on_drop() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (handle, _) = kernel.create_guest_thread(0x401000, 0, false);
        let pin = TaskWaitPin::Thread(
            kernel
                .handles
                .thread_birth_key_for_handle(handle)
                .expect("live thread birth"),
        );
        let waiters_before = kernel.scheduler.waiter_count();

        let mut prepared = kernel
            .prepare_task_wait(
                77,
                0,
                vec![handle, handle],
                SchedulerBlockedWaitKind::Kernel,
                0,
                crate::ce::timer::INFINITE,
            )
            .expect("both slots should pin the same live birth");
        assert_eq!(
            prepared.kernel_mut().handles.wait_target_pin_count(pin),
            Some(2)
        );
        assert_eq!(
            prepared.kernel_mut().scheduler.waiter_count(),
            waiters_before
        );
        drop(prepared);
        assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(0));
        assert_eq!(kernel.scheduler.waiter_count(), waiters_before);

        let wait_id = kernel
            .prepare_task_wait(
                77,
                0,
                vec![handle, handle],
                SchedulerBlockedWaitKind::Kernel,
                0,
                crate::ce::timer::INFINITE,
            )
            .expect("valid duplicate task slots")
            .publish();
        assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(2));
        assert_eq!(kernel.scheduler.waiter_count(), waiters_before + 1);
        assert!(kernel.remove_blocked_waiter(wait_id).is_some());
        assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(0));
        Ok(())
    }

    #[test]
    fn prepared_wait_rejects_reused_owner_birth_and_releases_duplicate_pins() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (owner_handle, owner_tid) = kernel.create_guest_thread(0x401000, 0, false);
        let (target_handle, _) = kernel.create_guest_thread(0x402000, 0, false);
        let target_pin = TaskWaitPin::Thread(
            kernel
                .handles
                .thread_birth_key_for_handle(target_handle)
                .expect("target birth"),
        );
        let waiters_before = kernel.scheduler.waiter_count();
        let mut prepared = kernel
            .prepare_task_wait(
                owner_tid,
                owner_handle,
                vec![target_handle, target_handle],
                SchedulerBlockedWaitKind::Kernel,
                0,
                crate::ce::timer::INFINITE,
            )
            .expect("duplicate targets prepare against live owner");
        let old_birth = prepared
            .preflight_owner_birth()
            .expect("live owner before exit");
        assert_eq!(
            prepared
                .kernel_mut()
                .handles
                .wait_target_pin_count(target_pin),
            Some(2)
        );
        assert!(
            prepared
                .kernel_mut()
                .mark_guest_thread_exited(owner_handle, 0x71)
        );
        assert_eq!(
            prepared.preflight_owner_birth(),
            Err(WaitPublishError::ExitedOwnerBirth { owner: old_birth })
        );
        assert!(prepared.kernel_mut().close_handle(owner_handle)?);
        let new_owner = prepared
            .kernel_mut()
            .handles
            .create_thread(owner_tid, 0x403000, 0, false);
        let new_birth = prepared
            .kernel_mut()
            .handles
            .thread_birth_key_for_handle(new_owner)
            .expect("replacement birth");
        assert_ne!(new_birth, old_birth);
        assert_eq!(
            prepared.preflight_owner_birth(),
            Err(WaitPublishError::StaleOwnerBirth {
                expected: old_birth,
                current: Some(new_birth),
            })
        );
        assert_eq!(
            prepared.try_publish(),
            Err(WaitPublishError::StaleOwnerBirth {
                expected: old_birth,
                current: Some(new_birth),
            })
        );
        assert_eq!(kernel.handles.wait_target_pin_count(target_pin), Some(0));
        assert_eq!(kernel.scheduler.waiter_count(), waiters_before);
        Ok(())
    }

    #[test]
    fn production_publish_rejects_unknown_owner_and_releases_pin() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (target_handle, _) = kernel.create_guest_thread(0x402000, 0, false);
        let target_pin = TaskWaitPin::Thread(
            kernel
                .handles
                .thread_birth_key_for_handle(target_handle)
                .expect("target birth"),
        );
        let waiters_before = kernel.scheduler.waiter_count();
        let prepared = kernel
            .prepare_task_wait(
                77,
                0,
                vec![target_handle],
                SchedulerBlockedWaitKind::Kernel,
                0,
                crate::ce::timer::INFINITE,
            )
            .expect("valid pinned target and deliberately unknown owner");
        assert_eq!(
            prepared.preflight_owner_birth(),
            Err(WaitPublishError::MissingOwnerBirth { thread_id: 77 })
        );
        assert_eq!(
            prepared.try_publish(),
            Err(WaitPublishError::MissingOwnerBirth { thread_id: 77 })
        );
        assert_eq!(kernel.handles.wait_target_pin_count(target_pin), Some(0));
        assert_eq!(kernel.scheduler.waiter_count(), waiters_before);
    }

    #[test]
    fn aliasless_boot_thread_publishes_exact_birth_waits() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let boot_birth = kernel
            .handles
            .thread_birth_key_by_id(1)
            .expect("canonical boot birth");
        assert!(kernel.handles.thread_execution_live_by_id(1));
        assert!(kernel.guest_thread_handle_by_id(1).is_none());
        let opaque = kernel
            .try_register_opaque_waiter_for_birth(
                boot_birth,
                CE_CURRENT_THREAD_PSEUDO_HANDLE,
                Vec::new(),
                SchedulerBlockedWaitKind::Sleep,
                0,
                crate::ce::timer::INFINITE,
            )
            .expect("aliasless boot birth is live");
        let event = kernel.create_event_w(None, false, false);
        let prepared = kernel
            .prepare_task_wait(
                1,
                CE_CURRENT_THREAD_PSEUDO_HANDLE,
                vec![event],
                SchedulerBlockedWaitKind::Kernel,
                0,
                crate::ce::timer::INFINITE,
            )
            .expect("boot thread can prepare object wait");
        assert_eq!(prepared.preflight_owner_birth(), Ok(boot_birth));
        let object = prepared.try_publish().expect("boot owner remains live");
        assert_eq!(
            kernel.scheduler.waiter_ids_for_birth(boot_birth),
            [opaque, object]
        );
        assert!(kernel.remove_blocked_waiter(opaque).is_some());
        assert!(kernel.remove_blocked_waiter(object).is_some());
        assert_eq!(kernel.scheduler.waiter_count(), 0);
    }

    #[test]
    fn opaque_wait_rejects_stale_owner_without_publishing() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (old_handle, owner_tid) = kernel.create_guest_thread(0x401000, 0, false);
        let old_birth = kernel
            .handles
            .thread_birth_key_for_handle(old_handle)
            .expect("old birth");
        assert!(kernel.mark_guest_thread_exited(old_handle, 0));
        assert!(kernel.close_handle(old_handle)?);
        let new_handle = kernel.handles.create_thread(owner_tid, 0x402000, 0, false);
        let new_birth = kernel
            .handles
            .thread_birth_key_for_handle(new_handle)
            .expect("new birth");
        let waiters_before = kernel.scheduler.waiter_count();
        assert_eq!(
            kernel.try_register_opaque_waiter_for_birth(
                old_birth,
                old_handle,
                vec![0x200],
                SchedulerBlockedWaitKind::Kernel,
                0,
                crate::ce::timer::INFINITE,
            ),
            Err(WaitPublishError::StaleOwnerBirth {
                expected: old_birth,
                current: Some(new_birth),
            })
        );
        assert_eq!(kernel.scheduler.waiter_count(), waiters_before);
        Ok(())
    }

    #[test]
    fn prepared_task_wait_retains_closed_completed_birth_before_publication() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (handle, thread_id) = kernel.create_guest_thread(0x401000, 0, false);
        let pin = TaskWaitPin::Thread(
            kernel
                .handles
                .thread_birth_key_for_handle(handle)
                .expect("live thread birth"),
        );
        let mut prepared = kernel
            .prepare_task_wait(
                77,
                0,
                vec![handle],
                SchedulerBlockedWaitKind::Kernel,
                0,
                crate::ce::timer::INFINITE,
            )
            .expect("valid target");
        assert!(prepared.kernel_mut().close_handle(handle)?);
        assert!(
            prepared
                .kernel_mut()
                .mark_guest_thread_exited_by_id(thread_id, 0x5678)
        );
        assert_eq!(
            prepared.kernel_mut().handles.wait_target_pin_count(pin),
            Some(1)
        );
        let wait_id = prepared.publish();

        assert_eq!(kernel.is_wait_ready(handle, 77), None);
        assert_eq!(
            kernel.wait_for_registered_wait_without_scheduler_record(wait_id, 77),
            WAIT_OBJECT_0
        );
        assert!(kernel.remove_blocked_waiter(wait_id).is_some());
        assert_eq!(kernel.handles.wait_target_pin_count(pin), None);
        Ok(())
    }

    #[test]
    fn prepared_task_wait_errors_roll_back_prior_pins_without_publication() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (first, _) = kernel.create_guest_thread(0x401000, 0, false);
        let (second, _) = kernel.create_guest_thread(0x402000, 0, false);
        let first_pin = TaskWaitPin::Thread(
            kernel
                .handles
                .thread_birth_key_for_handle(first)
                .expect("first thread birth"),
        );
        let second_pin = TaskWaitPin::Thread(
            kernel
                .handles
                .thread_birth_key_for_handle(second)
                .expect("second thread birth"),
        );
        let waiters_before = kernel.scheduler.waiter_count();

        let invalid = match kernel.prepare_task_wait(
            77,
            0,
            vec![first, 0xdead_beef],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        ) {
            Ok(_) => panic!("invalid second slot must not prepare"),
            Err(error) => error,
        };
        assert_eq!(
            invalid,
            WaitPrepError::InvalidHandle {
                index: 1,
                handle: 0xdead_beef,
            }
        );
        assert_eq!(invalid.last_error(), ERROR_INVALID_HANDLE);
        assert_eq!(kernel.threads.get_last_error(77), ERROR_INVALID_HANDLE);
        assert_eq!(kernel.handles.wait_target_pin_count(first_pin), Some(0));
        assert_eq!(kernel.scheduler.waiter_count(), waiters_before);

        assert!(
            kernel
                .handles
                .set_wait_target_pin_count_for_test(second_pin, u32::MAX)
        );
        let exhausted = match kernel.prepare_task_wait(
            77,
            0,
            vec![first, second],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        ) {
            Ok(_) => panic!("second target's pin counter must not wrap"),
            Err(error) => error,
        };
        assert_eq!(
            exhausted,
            WaitPrepError::PinExhausted {
                index: 1,
                pin: second_pin,
            }
        );
        assert_eq!(exhausted.last_error(), ERROR_NOT_ENOUGH_MEMORY);
        assert_eq!(kernel.threads.get_last_error(77), ERROR_NOT_ENOUGH_MEMORY);
        assert_eq!(kernel.handles.wait_target_pin_count(first_pin), Some(0));
        assert_eq!(
            kernel.handles.wait_target_pin_count(second_pin),
            Some(u32::MAX)
        );
        assert_eq!(kernel.scheduler.waiter_count(), waiters_before);
        assert!(
            kernel
                .handles
                .set_wait_target_pin_count_for_test(second_pin, 0)
        );
    }

    #[test]
    fn legacy_wait_registration_pin_failure_returns_zero_without_publication() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (first, _) = kernel.create_guest_thread(0x401000, 0, false);
        let (second, _) = kernel.create_guest_thread(0x402000, 0, false);
        let first_pin = TaskWaitPin::Thread(
            kernel
                .handles
                .thread_birth_key_for_handle(first)
                .expect("first thread birth"),
        );
        let second_pin = TaskWaitPin::Thread(
            kernel
                .handles
                .thread_birth_key_for_handle(second)
                .expect("second thread birth"),
        );
        assert!(
            kernel
                .handles
                .set_wait_target_pin_count_for_test(second_pin, u32::MAX)
        );
        let waiters_before = kernel.scheduler.waiter_count();
        let wait_id = kernel.register_blocked_waiter(
            77,
            0,
            vec![first, second],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        );
        assert_eq!(wait_id, 0);
        assert!(kernel.blocked_waiter(wait_id).is_none());
        assert_eq!(kernel.scheduler.waiter_count(), waiters_before);
        assert_eq!(kernel.handles.wait_target_pin_count(first_pin), Some(0));
        assert_eq!(
            kernel.handles.wait_target_pin_count(second_pin),
            Some(u32::MAX)
        );
        assert_eq!(kernel.threads.get_last_error(77), ERROR_NOT_ENOUGH_MEMORY);
        assert!(
            kernel
                .handles
                .set_wait_target_pin_count_for_test(second_pin, 0)
        );
    }

    #[test]
    fn registered_wait_uses_closed_task_birth_and_keeps_raw_event_acquisition() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let event = kernel.create_event_w(None, false, false);
        let (source, thread_id) = kernel.create_guest_thread(0x401000, 0, false);
        let alias = kernel.duplicate_handle(source, kernel.current_process_id(), false)?;
        let birth = kernel
            .handles
            .thread_birth_key_for_handle(alias)
            .expect("wait target birth");
        let wait_id = kernel.register_blocked_waiter(
            77,
            0,
            vec![event, alias],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        );
        assert_eq!(
            kernel.blocked_waiter(wait_id).unwrap().pinned_targets,
            vec![None, Some(TaskWaitPin::Thread(birth))]
        );
        assert!(kernel.close_handle(alias)?);
        assert!(kernel.mark_guest_thread_exited_by_id(thread_id, 0x5678));
        assert!(kernel.close_handle(source)?);
        assert_eq!(kernel.is_wait_ready(alias, 77), None);
        assert!(kernel.registered_wait_has_ready_handle(wait_id));
        assert_eq!(
            kernel.wait_for_registered_wait_without_scheduler_record(wait_id, 77),
            WAIT_OBJECT_0 + 1
        );
        assert!(kernel.set_event(event));
        assert_eq!(
            kernel.wait_for_registered_wait_without_scheduler_record(wait_id, 77),
            WAIT_OBJECT_0
        );
        assert_eq!(kernel.is_wait_ready(event, 77), Some(false));
        assert!(kernel.remove_blocked_waiter(wait_id).is_some());
        assert!(
            kernel
                .handles
                .wait_target_pin_count(TaskWaitPin::Thread(birth))
                .is_none()
        );
        Ok(())
    }

    #[test]
    fn exiting_waiter_owner_cancels_its_wait_and_releases_target_pin() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (owner_handle, owner_tid) = kernel.create_guest_thread(0x401000, 0, false);
        let (target_handle, target_tid) = kernel.create_guest_thread(0x402000, 0, false);
        let target_birth = kernel
            .handles
            .thread_birth_key_for_handle(target_handle)
            .expect("target birth");
        let target_pin = TaskWaitPin::Thread(target_birth);
        let wait_id = kernel.register_blocked_waiter(
            owner_tid,
            owner_handle,
            vec![target_handle],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        );
        assert_eq!(kernel.handles.wait_target_pin_count(target_pin), Some(1));
        assert_eq!(
            kernel.blocked_waiter(wait_id).unwrap().owner_birth,
            kernel.handles.thread_birth_key_for_handle(owner_handle)
        );

        assert!(kernel.mark_guest_thread_exited(owner_handle, 0x71));
        assert!(kernel.blocked_waiter(wait_id).is_none());
        assert_eq!(kernel.handles.wait_target_pin_count(target_pin), Some(0));
        assert!(kernel.handles.thread_execution_live_by_id(target_tid));
        assert_eq!(kernel.remove_blocked_waiters_for_thread(owner_tid), 0);
        Ok(())
    }

    #[test]
    fn exact_owner_exit_cleans_duplicate_pins_and_pending_wake() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (owner_handle, owner_tid) = kernel.create_guest_thread(0x401000, 0, false);
        let (target_handle, _) = kernel.create_guest_thread(0x402000, 0, false);
        let target_pin = TaskWaitPin::Thread(
            kernel
                .handles
                .thread_birth_key_for_handle(target_handle)
                .expect("target birth"),
        );
        let wait_id = kernel.register_blocked_waiter(
            owner_tid,
            owner_handle,
            vec![target_handle, target_handle],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        );
        assert_eq!(kernel.handles.wait_target_pin_count(target_pin), Some(2));
        assert_eq!(kernel.scheduler.queue_pending_wake_ids([wait_id]), 1);
        assert!(kernel.mark_guest_thread_exited(owner_handle, 0x71));
        assert!(kernel.blocked_waiter(wait_id).is_none());
        assert_eq!(kernel.handles.wait_target_pin_count(target_pin), Some(0));
        assert_eq!(
            kernel
                .scheduler
                .select_ready_blocked_wait_id(0, 0, |_| true, |_| 20),
            None
        );
        Ok(())
    }

    #[test]
    fn exact_exit_preserves_unknown_legacy_wait_until_explicit_cleanup() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let owner_tid = 77;
        let unknown_wait_id = kernel.register_opaque_compatibility_waiter(
            owner_tid,
            0,
            Vec::new(),
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        );
        assert_eq!(
            kernel.blocked_waiter(unknown_wait_id).unwrap().owner_birth,
            None
        );
        let owner_handle = kernel.handles.create_thread(owner_tid, 0x401000, 0, false);
        let owner_birth = kernel
            .handles
            .thread_birth_key_for_handle(owner_handle)
            .expect("canonical replacement birth");
        let known_wait_id = kernel
            .try_register_opaque_waiter_for_birth(
                owner_birth,
                owner_handle,
                Vec::new(),
                SchedulerBlockedWaitKind::Kernel,
                0,
                crate::ce::timer::INFINITE,
            )
            .expect("live replacement owner");
        assert_eq!(
            kernel.scheduler.waiter_ids_for_birth(owner_birth),
            [known_wait_id]
        );
        assert_eq!(
            kernel.scheduler.waiter_ids_for_thread(owner_tid),
            [unknown_wait_id]
        );
        assert!(kernel.mark_guest_thread_exited_by_birth(owner_birth, 0x71));
        assert!(kernel.blocked_waiter(known_wait_id).is_none());
        assert!(kernel.blocked_waiter(unknown_wait_id).is_some());
        assert_eq!(kernel.remove_blocked_waiters_for_thread(owner_tid), 1);
        assert_eq!(kernel.scheduler.waiter_count(), 0);
        Ok(())
    }

    #[test]
    fn stale_waiter_birth_cannot_cancel_reused_tid_wait() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (old_owner, thread_id) = kernel.create_guest_thread(0x401000, 0, false);
        let old_birth = kernel
            .handles
            .thread_birth_key_for_handle(old_owner)
            .expect("old owner birth");
        assert!(kernel.mark_guest_thread_exited(old_owner, 0));
        assert!(kernel.close_handle(old_owner)?);
        assert!(!kernel.handles.contains_thread_birth(old_birth));

        let new_owner = kernel.handles.create_thread(thread_id, 0x403000, 0, false);
        let new_birth = kernel
            .handles
            .thread_birth_key_for_handle(new_owner)
            .expect("new owner birth");
        assert_ne!(old_birth, new_birth);
        let (target_handle, _) = kernel.create_guest_thread(0x402000, 0, false);
        let target_pin = TaskWaitPin::Thread(
            kernel
                .handles
                .thread_birth_key_for_handle(target_handle)
                .expect("target birth"),
        );
        let wait_id = kernel.register_blocked_waiter(
            thread_id,
            new_owner,
            vec![target_handle],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        );
        assert_eq!(
            kernel.blocked_waiter(wait_id).unwrap().owner_birth,
            Some(new_birth)
        );
        assert!(!kernel.mark_guest_thread_exited_by_birth(old_birth, 0x99));
        assert!(kernel.blocked_waiter(wait_id).is_some());
        assert_eq!(kernel.handles.wait_target_pin_count(target_pin), Some(1));
        assert!(kernel.mark_guest_thread_exited_by_birth(new_birth, 0x77));
        assert!(kernel.blocked_waiter(wait_id).is_none());
        assert_eq!(kernel.handles.wait_target_pin_count(target_pin), Some(0));
        Ok(())
    }

    #[test]
    fn thread_post_and_quit_from_exited_birth_do_not_reach_reused_tid() -> Result<()> {
        use crate::ce::gwe::{PeekFlags, QS_POSTMESSAGE, WM_USER};

        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (old_handle, thread_id) = kernel.create_guest_thread(0x401000, 0, false);
        let old_birth = kernel
            .handles
            .thread_birth_key_for_handle(old_handle)
            .expect("A birth");
        let old_message = WM_USER + 0x71;
        assert!(kernel.post_thread_message_w(thread_id, old_message, 0xa1, 0));
        assert!(kernel.post_message_w_for_thread(thread_id, 0, WM_USER + 0x73, 0xa3, 0));
        kernel.post_quit_message(thread_id, 0xa2);
        assert!(kernel.mark_guest_thread_exited_by_birth(old_birth, 0));
        assert!(kernel.close_handle(old_handle)?);
        assert!(!kernel.handles.contains_thread_birth(old_birth));

        let new_handle = kernel.handles.create_thread(thread_id, 0x402000, 0, false);
        let new_birth = kernel
            .handles
            .thread_birth_key_for_handle(new_handle)
            .expect("B birth");
        assert_ne!(old_birth, new_birth);
        assert!(kernel.handles.thread_execution_live_by_id(thread_id));

        assert_eq!(
            kernel
                .gwe
                .peek_queue_status_for_birth(new_birth, QS_POSTMESSAGE),
            0
        );
        assert_eq!(
            kernel.get_message_queue_ready_time_stamp_for_thread(thread_id, 0),
            0
        );
        assert!(
            kernel
                .peek_message_w_filtered(thread_id, None, 0, 0, PeekFlags::empty())
                .is_none(),
            "B must not peek A's posted message or quit state"
        );
        assert!(
            kernel
                .peek_message_w_filtered(thread_id, None, 0, 0, PeekFlags::REMOVE)
                .is_none(),
            "B must not remove A's posted message or quit state"
        );
        assert!(
            kernel
                .get_message_w_filtered(thread_id, None, 0, 0)
                .is_none(),
            "B must not get A's posted message or quit state"
        );
        let trace_count = kernel.recent_message_ops().len();
        assert!(!kernel.post_thread_message_w_for_birth(old_birth, WM_USER + 0x74, 0xa4, 0,));
        assert!(!kernel.post_quit_message_for_birth(old_birth, 0xa5));
        assert_eq!(kernel.recent_message_ops().len(), trace_count);
        assert_eq!(
            kernel
                .gwe
                .peek_queue_status_for_birth(new_birth, QS_POSTMESSAGE),
            0
        );

        let new_message = WM_USER + 0x72;
        assert!(kernel.post_thread_message_w(thread_id, new_message, 0xb1, 0));
        assert_eq!(
            kernel
                .peek_message_w_filtered(thread_id, None, 0, 0, PeekFlags::empty())
                .map(|message| message.msg),
            Some(new_message)
        );
        assert_eq!(
            kernel
                .get_message_w_filtered(thread_id, None, 0, 0)
                .map(|message| message.msg),
            Some(new_message)
        );
        Ok(())
    }

    #[test]
    fn hwndless_thread_timers_do_not_cross_reused_tid_birth() -> Result<()> {
        use crate::ce::gwe::{QS_TIMER, WM_TIMER};

        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (old_handle, thread_id) = kernel.create_guest_thread(0x401000, 0, false);
        let old_birth = kernel
            .handles
            .thread_birth_key_for_handle(old_handle)
            .expect("A birth");
        let queued_a_id = 0xa1;
        let later_a_id = 0xa2;
        let fresh_b_id = 0xb1;

        assert_eq!(
            kernel.set_timer_for_birth(old_birth, Some(queued_a_id), 0, None),
            queued_a_id
        );
        kernel.pump_timers_to_gwe(thread_id);
        assert!(kernel.timers.pending_timers().iter().any(|timer| {
            timer.owner_birth == Some(old_birth)
                && timer.hwnd.is_none()
                && timer.id == queued_a_id
                && timer.pending_message
        }));
        assert_eq!(
            kernel.set_timer_for_birth(old_birth, Some(later_a_id), 25, None),
            later_a_id
        );

        assert!(kernel.mark_guest_thread_exited_by_birth(old_birth, 0));
        assert!(kernel.close_handle(old_handle)?);
        assert!(!kernel.handles.contains_thread_birth(old_birth));
        let new_handle = kernel.handles.create_thread(thread_id, 0x402000, 0, false);
        let new_birth = kernel
            .handles
            .thread_birth_key_for_handle(new_handle)
            .expect("B birth");
        assert_ne!(old_birth, new_birth);

        let prequeued_for_b = kernel
            .peek_ready_message_w_filtered(thread_id, None, 0, 0)
            .map(|message| (message.msg, message.wparam));
        let timer_status_for_b = kernel.gwe.peek_queue_status_for_birth(new_birth, QS_TIMER);
        let ready_time_for_b = kernel.get_message_queue_ready_time_stamp_for_thread(thread_id, 0);
        let surviving_a_timer_ids: Vec<_> = kernel
            .timers
            .pending_timers()
            .into_iter()
            .filter(|timer| {
                timer.owner_birth == Some(old_birth)
                    && timer.hwnd.is_none()
                    && (timer.id == queued_a_id || timer.id == later_a_id)
            })
            .map(|timer| timer.id)
            .collect();

        let trace_start = kernel.recent_message_ops().len();
        kernel.timers.sleep_ms(25);
        kernel.pump_timers_to_gwe(thread_id);
        let late_a_due: Vec<_> = kernel.recent_message_ops()[trace_start..]
            .iter()
            .filter(|record| {
                record.op == "timer_due"
                    && record.thread_id == thread_id
                    && record.msg == Some(WM_TIMER)
                    && record.wparam == Some(later_a_id)
            })
            .map(|record| record.wparam)
            .collect();
        let queued_a_for_b: Vec<_> = kernel
            .gwe
            .queue_snapshot()
            .into_iter()
            .filter(|(owner, _)| *owner == thread_id)
            .flat_map(|(_, messages)| messages)
            .filter(|message| {
                message.hwnd == 0
                    && message.msg == WM_TIMER
                    && (message.wparam == queued_a_id || message.wparam == later_a_id)
            })
            .map(|message| message.wparam)
            .collect();

        assert_eq!(
            kernel.set_timer_for_birth(new_birth, Some(fresh_b_id), 0, None),
            fresh_b_id
        );
        kernel.pump_timers_to_gwe(thread_id);
        let first_visible_for_b = kernel
            .peek_ready_message_w_filtered(thread_id, None, 0, 0)
            .map(|message| (message.msg, message.wparam));

        assert!(
            prequeued_for_b.is_none()
                && timer_status_for_b == 0
                && ready_time_for_b == 0
                && surviving_a_timer_ids.is_empty()
                && late_a_due.is_empty()
                && queued_a_for_b.is_empty()
                && first_visible_for_b == Some((WM_TIMER, fresh_b_id)),
            "B inherited A's HWND-less timer state: prequeued={prequeued_for_b:?}, status=0x{timer_status_for_b:08x}, ready_time={ready_time_for_b}, timers={surviving_a_timer_ids:?}, late_due={late_a_due:?}, queued={queued_a_for_b:?}, first_visible={first_visible_for_b:?}"
        );
        Ok(())
    }

    #[test]
    fn hwndless_thread_timer_pending_clear_requires_its_live_instance() -> Result<()> {
        use crate::ce::gwe::{PeekFlags, WM_TIMER};

        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (old_handle, thread_id) = kernel.create_guest_thread(0x401000, 0, false);
        let old_birth = kernel
            .handles
            .thread_birth_key_for_handle(old_handle)
            .expect("A birth");
        assert!(kernel.mark_guest_thread_exited_by_birth(old_birth, 0));
        assert!(kernel.close_handle(old_handle)?);
        let new_handle = kernel.handles.create_thread(thread_id, 0x402000, 0, false);
        let new_birth = kernel
            .handles
            .thread_birth_key_for_handle(new_handle)
            .expect("B birth");
        let timer_id = 0xb7;
        assert_eq!(
            kernel.set_timer_for_birth(new_birth, Some(timer_id), 1, Some(0x1234)),
            timer_id
        );
        let trace_count = kernel.recent_message_ops().len();
        assert_eq!(
            kernel.set_timer_for_birth(old_birth, Some(timer_id), 1, Some(0xdead)),
            0
        );
        assert!(!kernel.kill_timer_for_birth(old_birth, timer_id));
        assert_eq!(kernel.recent_message_ops().len(), trace_count);

        assert!(kernel.post_thread_message_w_for_birth(new_birth, WM_TIMER, timer_id, 0));
        kernel.timers.sleep_ms(1);
        kernel.pump_timers_to_gwe(thread_id);
        let pending = |kernel: &CeKernel| {
            kernel
                .timers
                .pending_timers()
                .into_iter()
                .find(|timer| timer.owner_birth == Some(new_birth) && timer.id == timer_id)
                .expect("B timer")
        };
        assert!(pending(&kernel).pending_message);
        let peek = kernel
            .peek_message_w_filtered(thread_id, None, 0, 0, PeekFlags::NO_REMOVE)
            .expect("synthetic timer peek");
        assert_eq!(peek.timer_origin, None);
        assert!(pending(&kernel).pending_message);
        let synthetic = kernel
            .peek_message_w_filtered(thread_id, None, 0, 0, PeekFlags::REMOVE)
            .expect("synthetic timer removal");
        assert_eq!(synthetic.timer_origin, None);
        assert!(pending(&kernel).pending_message);
        let actual = kernel
            .peek_message_w_filtered(thread_id, None, 0, 0, PeekFlags::REMOVE)
            .expect("real B timer removal");
        assert_eq!(
            (actual.msg, actual.wparam, actual.lparam),
            (WM_TIMER, timer_id, 0x1234)
        );
        assert_eq!(
            actual.timer_origin.expect("host timer origin").birth,
            new_birth
        );
        assert!(!pending(&kernel).pending_message);

        // Replacing a same-ID timer gives it a new instance. A queued message
        // from the previous instance cannot clear the replacement's bit.
        assert_eq!(
            kernel.set_timer_for_birth(new_birth, Some(timer_id), 1, Some(0x2222)),
            timer_id
        );
        kernel.timers.sleep_ms(1);
        kernel.pump_timers_to_gwe(thread_id);
        let prior_instance = pending(&kernel).instance;
        assert_eq!(
            kernel.set_timer_for_birth(new_birth, Some(timer_id), 1, Some(0x3333)),
            timer_id
        );
        kernel.timers.sleep_ms(1);
        kernel.pump_timers_to_gwe(thread_id);
        let current_instance = pending(&kernel).instance;
        assert_ne!(prior_instance, current_instance);
        let old_message = kernel
            .peek_message_w_filtered(thread_id, None, 0, 0, PeekFlags::REMOVE)
            .expect("old instance message");
        assert_eq!(
            old_message.timer_origin.expect("old origin").instance,
            prior_instance
        );
        assert!(pending(&kernel).pending_message);
        let new_message = kernel
            .peek_message_w_filtered(thread_id, None, 0, 0, PeekFlags::REMOVE)
            .expect("new instance message");
        assert_eq!(
            new_message.timer_origin.expect("new origin").instance,
            current_instance
        );
        assert!(!pending(&kernel).pending_message);
        Ok(())
    }

    #[test]
    fn checked_window_hwnd_post_does_not_cross_reused_tid_birth() -> Result<()> {
        use crate::ce::gwe::{PeekFlags, QS_POSTMESSAGE, WM_USER};

        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let owner_a = kernel
            .current_process_birth_key()
            .expect("boot process birth");
        let (handle_a, thread_id) = kernel.create_guest_thread(0x401000, 0, false);
        let birth_a = kernel
            .handles
            .thread_birth_key_for_handle(handle_a)
            .expect("A worker birth");
        assert_eq!(
            kernel.handles.thread_creation_owner_by_id(thread_id),
            Some(owner_a)
        );
        let hwnd_a = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth_a,
                "BIRTH_WINDOW_A",
                "A",
                None,
                None,
                0,
                0,
                0,
                Rect::default(),
            )
            .expect("A checked window");
        assert_eq!(kernel.recipient_birth_for_window(hwnd_a), Some(owner_a));
        kernel.timers.sleep_ms(17);
        let message_a = WM_USER + 0x91;
        assert!(kernel.post_message_w(hwnd_a, message_a, 0xa1, 0));
        assert_ne!(
            kernel
                .gwe
                .peek_queue_status_for_birth(birth_a, QS_POSTMESSAGE),
            0
        );
        assert_ne!(
            kernel.get_message_queue_ready_time_stamp_for_thread(thread_id, 0),
            0
        );

        assert!(kernel.mark_guest_thread_exited_by_birth(birth_a, 0));
        assert!(kernel.close_handle(handle_a)?);
        assert!(!kernel.handles.contains_thread_birth(birth_a));

        let launch_b = kernel.queue_process_launch(None, None);
        let owner_b = kernel
            .handles
            .process_birth_key(launch_b.process_id)
            .expect("B process birth");
        assert_ne!(owner_a, owner_b);
        let handle_b = kernel
            .handles
            .try_create_thread_in_birth(owner_b, thread_id, 0x402000, 0, false)?;
        let birth_b = kernel
            .handles
            .thread_birth_key_for_handle(handle_b)
            .expect("B worker birth");
        assert_ne!(birth_a, birth_b);
        assert_eq!(
            kernel.handles.thread_creation_owner_by_id(thread_id),
            Some(owner_b)
        );
        assert_eq!(
            kernel
                .gwe
                .window(hwnd_a)
                .unwrap()
                .delivery_owner()
                .process_birth(),
            Some(owner_a)
        );
        assert_eq!(kernel.recipient_birth_for_window(hwnd_a), None);
        kernel.set_current_process_id(launch_b.process_id);
        let windows_before_stale_create = kernel.gwe.windows_snapshot().len();
        assert!(
            kernel
                .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                    birth_a,
                    "STALE_WINDOW_A",
                    "stale",
                    None,
                    None,
                    0,
                    0,
                    0,
                    Rect::default(),
                )
                .is_none()
        );
        assert_eq!(
            kernel.gwe.windows_snapshot().len(),
            windows_before_stale_create
        );
        assert_eq!(
            kernel
                .gwe
                .peek_queue_status_for_birth(birth_b, QS_POSTMESSAGE),
            0,
            "B must not inherit A's HWND post queue status"
        );
        assert_eq!(
            kernel.get_message_queue_ready_time_stamp_for_thread(thread_id, 0),
            0
        );
        assert!(
            kernel
                .peek_message_w_filtered(thread_id, None, 0, 0, PeekFlags::empty())
                .is_none()
        );
        assert!(
            kernel
                .get_message_w_filtered(thread_id, None, 0, 0)
                .is_none()
        );

        let wait_id = kernel.register_blocked_waiter(
            thread_id,
            handle_b,
            vec![],
            SchedulerBlockedWaitKind::GetMessage {
                hwnd: None,
                min_msg: 0,
                max_msg: 0,
            },
            kernel.timers.tick_count(),
            crate::ce::timer::INFINITE,
        );
        assert_ne!(wait_id, 0);
        let trace_before = kernel.recent_message_ops().len();
        let wakes_before = kernel.scheduler_stats().message_input_wake_candidate_count;
        let signals_before = kernel.scheduler_stats().message_input_signal_count;
        assert!(!kernel.post_message_w(hwnd_a, WM_USER + 0x92, 0xa2, 0));
        assert_eq!(kernel.recent_message_ops().len(), trace_before);
        assert_eq!(
            kernel.scheduler_stats().message_input_wake_candidate_count,
            wakes_before
        );
        assert_eq!(
            kernel.scheduler_stats().message_input_signal_count,
            signals_before
        );
        assert_eq!(
            kernel
                .gwe
                .peek_queue_status_for_birth(birth_b, QS_POSTMESSAGE),
            0
        );
        assert_eq!(
            kernel.get_message_queue_ready_time_stamp_for_thread(thread_id, 0),
            0
        );

        let hwnd_b = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth_b,
                "BIRTH_WINDOW_B",
                "B",
                None,
                None,
                0,
                0,
                0,
                Rect::default(),
            )
            .expect("B checked window");
        assert_eq!(kernel.recipient_birth_for_window(hwnd_b), Some(owner_b));
        assert!(kernel.post_message_w(hwnd_b, WM_USER + 0x93, 0xb1, 0));
        assert!(kernel.scheduler_stats().message_input_wake_candidate_count > wakes_before);
        assert_eq!(
            kernel
                .get_message_w_filtered(thread_id, None, 0, 0)
                .map(|message| message.msg),
            Some(WM_USER + 0x93)
        );
        assert!(kernel.remove_blocked_waiter(wait_id).is_some());
        Ok(())
    }

    #[test]
    fn legacy_window_typed_recipient_keeps_creation_birth_across_pid_reuse() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let process_handle_a = kernel.handles.create_process(77);
        let process_a = kernel
            .handles
            .process_birth_key(77)
            .expect("first PID birth");
        kernel.set_current_process_id(77);
        let hwnd_a = kernel.create_window_ex_w(9, "LEGACY_PID_A", "", None, 0, 0, 0);
        assert_eq!(kernel.recipient_birth_for_window(hwnd_a), Some(process_a));

        kernel.set_current_process_id(1);
        assert!(
            kernel
                .handles
                .finish_process_execution_by_birth(process_a, 0)
                .is_some()
        );
        assert!(kernel.close_handle(process_handle_a)?);
        let process_handle_b = kernel.handles.create_process(77);
        let process_b = kernel
            .handles
            .process_birth_key(77)
            .expect("reused PID birth");
        assert_ne!(process_a, process_b);
        assert_eq!(
            kernel
                .gwe
                .window(hwnd_a)
                .unwrap()
                .delivery_owner()
                .process_birth(),
            Some(process_a)
        );
        assert_eq!(kernel.recipient_birth_for_window(hwnd_a), None);
        assert_ne!(kernel.recipient_birth_for_window(hwnd_a), Some(process_b));
        assert!(kernel.close_handle(process_handle_b)?);
        Ok(())
    }

    #[test]
    fn birth_owned_window_typed_post_retires_without_reaching_reused_tid() -> Result<()> {
        use crate::ce::gwe::{QS_POSTMESSAGE, WM_WINDOWPOSCHANGED};

        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let process_a = kernel.current_process_birth_key().expect("boot birth");
        let (handle_a, thread_id) = kernel.create_guest_thread(0x401000, 0, false);
        let birth_a = kernel
            .handles
            .thread_birth_key_for_handle(handle_a)
            .expect("A thread birth");
        let hwnd_a = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth_a,
                "BIRTH_TYPED_A",
                "A",
                None,
                None,
                0,
                0,
                0,
                Rect::default(),
            )
            .expect("A checked window");
        let position = WindowPos {
            hwnd: hwnd_a,
            insert_after: HWND_TOP,
            x: 1,
            y: 2,
            width: 3,
            height: 4,
            flags: 0,
        };
        let (payload_owner, ptr) = kernel
            .queue_window_pos_payload(hwnd_a, position.clone())
            .expect("A payload");
        assert_eq!(payload_owner, process_a);
        assert!(kernel.post_window_typed_payload_message(
            hwnd_a,
            WM_WINDOWPOSCHANGED,
            0,
            ptr,
            payload_owner,
        ));
        assert_ne!(
            kernel
                .gwe
                .peek_queue_status_for_birth(birth_a, QS_POSTMESSAGE),
            0
        );

        assert!(kernel.mark_guest_thread_exited_by_birth(birth_a, 0));
        assert!(kernel.close_handle(handle_a)?);
        assert!(kernel.gwe.message_pointer_payload(process_a, ptr).is_none());
        assert!(
            kernel
                .memory
                .heap_size(process_a, PROCESS_HEAP_HANDLE, 0, ptr)
                .is_none()
        );
        assert!(kernel.queue_window_pos_payload(hwnd_a, position).is_none());

        let launch_b = kernel.queue_process_launch(None, None);
        let process_b = kernel
            .handles
            .process_birth_key(launch_b.process_id)
            .expect("B process birth");
        let handle_b = kernel
            .handles
            .try_create_thread_in_birth(process_b, thread_id, 0x402000, 0, false)?;
        let birth_b = kernel
            .handles
            .thread_birth_key_for_handle(handle_b)
            .expect("B thread birth");
        assert_ne!(birth_a, birth_b);
        let trace_before = kernel.recent_message_ops().len();
        let wakes_before = kernel.scheduler_stats().message_input_wake_candidate_count;
        let signals_before = kernel.scheduler_stats().message_input_signal_count;
        // Failed typed allocation for the stale HWND follows the ordinary
        // WM_WINDOWPOSCHANGED fallback, which must also reject A's owner.
        kernel.post_window_rect_messages(
            hwnd_a,
            Some(Rect::from_origin_size(0, 0, 10, 10)),
            Some(Rect::from_origin_size(1, 0, 10, 10)),
            HWND_TOP,
            0,
            false,
        );
        assert_eq!(kernel.recent_message_ops().len(), trace_before);
        assert_eq!(
            kernel.scheduler_stats().message_input_wake_candidate_count,
            wakes_before
        );
        assert_eq!(
            kernel.scheduler_stats().message_input_signal_count,
            signals_before
        );
        assert_eq!(
            kernel
                .gwe
                .peek_queue_status_for_birth(birth_b, QS_POSTMESSAGE),
            0
        );
        assert!(!kernel.gwe.has_message_filtered_for_birth(
            birth_b,
            Some(hwnd_a),
            WM_WINDOWPOSCHANGED,
            WM_WINDOWPOSCHANGED,
        ));
        Ok(())
    }

    #[test]
    fn thread_birth_retirement_preserves_unscoped_window_fifo_and_filter() -> Result<()> {
        use crate::ce::gwe::{PeekFlags, QS_POSTMESSAGE, WM_USER};

        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (old_handle, thread_id) = kernel.create_guest_thread(0x401000, 0, false);
        let old_birth = kernel
            .handles
            .thread_birth_key_for_handle(old_handle)
            .expect("A birth");
        let hwnd = kernel.gwe.create_window(thread_id, "BIRTH_FIFO", "fifo");
        let window_message = WM_USER + 0x80;
        let old_thread_message = WM_USER + 0x81;
        let new_thread_message = WM_USER + 0x82;
        assert!(kernel.post_message_w(hwnd, window_message, 1, 0));
        assert!(kernel.post_thread_message_w(thread_id, old_thread_message, 2, 0));
        assert!(kernel.mark_guest_thread_exited_by_birth(old_birth, 0));
        assert!(kernel.close_handle(old_handle)?);

        let new_handle = kernel.handles.create_thread(thread_id, 0x402000, 0, false);
        let new_birth = kernel
            .handles
            .thread_birth_key_for_handle(new_handle)
            .expect("B birth");
        assert_ne!(old_birth, new_birth);
        assert_ne!(
            kernel
                .gwe
                .peek_queue_status_for_birth(new_birth, QS_POSTMESSAGE),
            0,
            "A's exit must not clear the surviving window post"
        );
        assert_eq!(
            kernel
                .peek_message_w_filtered(thread_id, Some(0), 0, 0, PeekFlags::empty())
                .map(|message| message.msg),
            None,
            "A's thread post must not pass B's thread-message filter"
        );
        assert!(kernel.post_thread_message_w(thread_id, new_thread_message, 3, 0));
        assert_eq!(
            kernel
                .peek_message_w_filtered(thread_id, Some(0), 0, 0, PeekFlags::empty())
                .map(|message| message.msg),
            Some(new_thread_message)
        );
        assert_eq!(
            kernel
                .get_message_w_filtered(thread_id, None, 0, 0)
                .map(|message| message.msg),
            Some(window_message),
            "unfiltered delivery keeps the first eligible FIFO entry"
        );
        assert_eq!(
            kernel
                .get_message_w_filtered(thread_id, None, 0, 0)
                .map(|message| message.msg),
            Some(new_thread_message)
        );
        Ok(())
    }

    #[test]
    fn exiting_sender_removes_queued_send_before_receiver_delivery() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (sender_handle, sender_tid) = kernel.create_guest_thread(0x401000, 0, false);
        let sender_birth = kernel
            .handles
            .thread_birth_key_for_handle(sender_handle)
            .expect("sender birth");
        let (_, receiver_tid) = kernel.create_guest_thread(0x402000, 0, false);
        let hwnd = kernel.gwe.create_window(receiver_tid, "target", "");
        let send_id = kernel
            .begin_cross_thread_send_message_w_for_birth(
                sender_birth,
                hwnd,
                crate::ce::gwe::WM_USER + 57,
                1,
                2,
                None,
            )
            .expect("queued sync send");
        let wait_id = kernel.register_opaque_compatibility_waiter(
            sender_tid,
            sender_handle,
            vec![send_id as u32],
            SchedulerBlockedWaitKind::SendMessage { send_id },
            0,
            crate::ce::timer::INFINITE,
        );
        assert!(kernel.gwe.has_pending_sent_message_for_thread(receiver_tid));
        assert!(kernel.blocked_waiter(wait_id).is_some());

        assert!(kernel.mark_guest_thread_exited(sender_handle, 0x71));
        assert!(kernel.blocked_waiter(wait_id).is_none());
        assert!(kernel.gwe.sent_message(send_id).is_none());
        assert!(!kernel.gwe.has_pending_sent_message_for_thread(receiver_tid));
        assert_eq!(kernel.gwe.get_message(receiver_tid), None);
    }

    #[test]
    fn exiting_sender_detaches_active_send_until_receiver_returns() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (sender_handle, _) = kernel.create_guest_thread(0x401000, 0, false);
        let sender_birth = kernel
            .handles
            .thread_birth_key_for_handle(sender_handle)
            .expect("sender birth");
        let (_, receiver_tid) = kernel.create_guest_thread(0x402000, 0, false);
        let hwnd = kernel.gwe.create_window(receiver_tid, "target", "");
        let send_id = kernel
            .begin_cross_thread_send_message_w_for_birth(
                sender_birth,
                hwnd,
                crate::ce::gwe::WM_USER + 58,
                1,
                2,
                None,
            )
            .expect("queued sync send");
        assert!(
            kernel
                .gwe
                .activate_sent_message_for_receiver(receiver_tid, send_id)
        );
        assert_eq!(
            kernel.gwe.active_sent_message_id(receiver_tid),
            Some(send_id)
        );

        assert!(kernel.mark_guest_thread_exited(sender_handle, 0x72));
        let sent = kernel.gwe.sent_message(send_id).expect("active send state");
        assert_eq!(sent.sender_thread_id(), None);
        assert_ne!(sent.flags & crate::ce::gwe::SMF_SENDER_TERMINATED, 0);
        assert_eq!(sent.flags & crate::ce::gwe::SMF_RESULT_READY, 0);
        assert_eq!(
            kernel.gwe.active_sent_message_id(receiver_tid),
            Some(send_id)
        );
        assert_eq!(
            kernel.complete_active_sent_message(receiver_tid, 0x1234),
            Some(send_id)
        );
        assert!(kernel.gwe.sent_message(send_id).is_none());
        assert_eq!(kernel.take_completed_send_message_result(send_id), None);
    }

    #[test]
    fn exiting_sender_preserves_other_senders_queued_send() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (sender_a_handle, _) = kernel.create_guest_thread(0x401000, 0, false);
        let sender_a_birth = kernel
            .handles
            .thread_birth_key_for_handle(sender_a_handle)
            .expect("A sender birth");
        let (sender_b_handle, sender_b_tid) = kernel.create_guest_thread(0x402000, 0, false);
        let sender_b_birth = kernel
            .handles
            .thread_birth_key_for_handle(sender_b_handle)
            .expect("B sender birth");
        let (_, receiver_tid) = kernel.create_guest_thread(0x403000, 0, false);
        let hwnd = kernel.gwe.create_window(receiver_tid, "target", "");
        let active_a = kernel
            .begin_cross_thread_send_message_w_for_birth(
                sender_a_birth,
                hwnd,
                crate::ce::gwe::WM_USER + 60,
                1,
                2,
                None,
            )
            .expect("A's active send");
        assert!(
            kernel
                .gwe
                .activate_sent_message_for_receiver(receiver_tid, active_a)
        );
        let queued_a = kernel
            .begin_cross_thread_send_message_w_for_birth(
                sender_a_birth,
                hwnd,
                crate::ce::gwe::WM_USER + 61,
                1,
                2,
                None,
            )
            .expect("A's queued send");
        let queued_b = kernel
            .begin_cross_thread_send_message_w_for_birth(
                sender_b_birth,
                hwnd,
                crate::ce::gwe::WM_USER + 62,
                1,
                2,
                None,
            )
            .expect("B's queued send");

        assert!(kernel.mark_guest_thread_exited(sender_a_handle, 0x73));
        assert_eq!(
            kernel
                .gwe
                .sent_message(active_a)
                .and_then(|sent| sent.sender_thread_id()),
            None
        );
        assert!(kernel.gwe.sent_message(queued_a).is_none());
        assert_eq!(
            kernel
                .gwe
                .sent_message(queued_b)
                .and_then(|sent| sent.sender_thread_id()),
            Some(sender_b_tid)
        );
        assert!(kernel.gwe.has_pending_sent_message_for_thread(receiver_tid));
        assert_eq!(
            kernel.gwe.active_sent_message_id(receiver_tid),
            Some(active_a)
        );
        assert_eq!(
            kernel.complete_active_sent_message(receiver_tid, 0x1234),
            Some(active_a)
        );
        assert!(kernel.gwe.sent_message(active_a).is_none());
        assert!(kernel.gwe.sent_message(queued_b).is_some());
    }

    #[test]
    fn stale_sender_birth_cannot_cancel_reused_tid_send() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let process_birth = kernel
            .current_process_birth_key()
            .expect("boot process birth");
        let (old_sender, sender_tid) = kernel.create_guest_thread(0x401000, 0, false);
        let old_birth = kernel
            .handles
            .thread_birth_key_for_handle(old_sender)
            .expect("old sender birth");
        assert!(kernel.mark_guest_thread_exited(old_sender, 0));
        assert!(
            kernel
                .handles
                .try_create_thread_in_birth(process_birth, sender_tid, 0x403000, 0, false)
                .is_err()
        );
        assert!(kernel.handles.contains_thread_birth(old_birth));
        assert!(kernel.close_handle(old_sender)?);
        assert!(!kernel.handles.contains_thread_birth(old_birth));

        let new_sender = kernel
            .handles
            .try_create_thread_in_birth(process_birth, sender_tid, 0x403000, 0, false)
            .expect("new sender in boot process birth");
        let new_birth = kernel
            .handles
            .thread_birth_key_for_handle(new_sender)
            .expect("new sender birth");
        assert_ne!(old_birth, new_birth);
        let (_, receiver_tid) = kernel.create_guest_thread(0x402000, 0, false);
        let hwnd = kernel.gwe.create_window(receiver_tid, "target", "");
        let send_id = kernel
            .begin_cross_thread_send_message_w_for_birth(
                new_birth,
                hwnd,
                crate::ce::gwe::WM_USER + 59,
                1,
                2,
                None,
            )
            .expect("new sender's queued send");
        let wait_id = kernel.register_opaque_compatibility_waiter(
            sender_tid,
            new_sender,
            vec![send_id as u32],
            SchedulerBlockedWaitKind::SendMessage { send_id },
            0,
            crate::ce::timer::INFINITE,
        );

        assert!(!kernel.mark_guest_thread_exited(old_sender, 0x99));
        assert!(!kernel.mark_guest_thread_exited_by_birth(old_birth, 0x99));
        assert!(kernel.handles.thread_execution_live_by_id(sender_tid));
        assert_eq!(
            kernel
                .gwe
                .sent_message(send_id)
                .and_then(|sent| sent.sender_thread_id()),
            Some(sender_tid)
        );
        assert!(kernel.gwe.has_pending_sent_message_for_thread(receiver_tid));
        assert!(kernel.blocked_waiter(wait_id).is_some());
        assert!(kernel.mark_guest_thread_exited_by_birth(new_birth, 0x77));
        assert!(kernel.gwe.sent_message(send_id).is_none());
        assert!(kernel.blocked_waiter(wait_id).is_none());
        Ok(())
    }

    #[test]
    fn synthetic_winsock_subscription_keeps_opaque_legacy_kernel_slot() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let socket = 0xcc00_0011;
        let wait_id = kernel.register_opaque_compatibility_waiter(
            77,
            0,
            vec![socket],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        );
        assert_eq!(
            kernel.blocked_waiter(wait_id).unwrap().pinned_targets,
            vec![None]
        );
        assert!(kernel.remove_blocked_waiter(wait_id).is_some());
    }

    #[test]
    fn remote_serial_arrival_signals_pending_overlapped_read_event() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let handle = kernel.create_file_w("COM7:", GENERIC_READ | GENERIC_WRITE, OPEN_EXISTING)?;
        let event_handle = kernel.create_event_w(None, false, false);
        let overlapped_ptr = 0x1000_0100;
        let buffer = 0x1000_0200;
        let transferred_ptr = 0x1000_0300;

        kernel.register_overlapped_serial_read(
            overlapped_ptr,
            PendingOverlappedSerialRead {
                handle,
                buffer,
                requested: 256,
                transferred_ptr,
                event_handle,
            },
        );
        assert_eq!(kernel.is_wait_ready(event_handle, 1), Some(false));

        assert_eq!(
            kernel.remote.inject_nmea_sentences([
                "$GPRMC,170236.000,A,3733.9900,N,12658.6800,E,0.0,0.0,200626,,,A*69"
            ]),
            1
        );
        assert_ne!(kernel.drain_remote_serial_to_matching_open_device(), 0);
        assert_eq!(kernel.is_wait_ready(event_handle, 1), Some(true));

        let mut delivered = Vec::new();
        let completed = kernel
            .complete_overlapped_serial_read_into(handle, overlapped_ptr, |bytes| {
                delivered.extend_from_slice(bytes);
                Ok(())
            })?
            .expect("pending overlapped read completes");

        assert_eq!(completed.0.buffer, buffer);
        assert_eq!(completed.0.transferred_ptr, transferred_ptr);
        assert_eq!(completed.1 as usize, delivered.len());
        assert!(String::from_utf8_lossy(&delivered).starts_with("$GPRMC"));
        assert!(
            kernel
                .pending_overlapped_serial_read(handle, overlapped_ptr)
                .is_none()
        );
        Ok(())
    }

    #[test]
    fn loaded_module_export_snapshots_skip_unload_pending_modules() -> Result<()> {
        let config = RuntimeConfig::load_default()?;
        let mut kernel = CeKernel::boot(config);
        let mut exports_by_name = BTreeMap::new();
        exports_by_name.insert("NamedExport".to_owned(), 0x1000_1234);
        let mut exports_by_ordinal = BTreeMap::new();
        exports_by_ordinal.insert(7, 0x1000_5678);
        kernel.register_loaded_module_with_metadata(
            "dynamic.dll",
            0x1000_0000,
            exports_by_name,
            exports_by_ordinal,
            LoadedModuleMetadata {
                dynamic: true,
                ..Default::default()
            },
        );
        assert_eq!(
            kernel.release_loaded_module(0x1000_0000),
            FreeLibraryResult::UnloadPending
        );

        kernel.register_loaded_module(
            "still_loaded.dll",
            0x2000_0000,
            BTreeMap::from([("Visible".to_owned(), 0x2000_0100)]),
            BTreeMap::from([(3, 0x2000_0200)]),
        );

        let snapshots = kernel.loaded_module_export_snapshots();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].name, "still_loaded.dll");
        assert_eq!(snapshots[0].exports_by_name["visible"], 0x2000_0100);
        assert_eq!(snapshots[0].exports_by_ordinal[&3], 0x2000_0200);
        Ok(())
    }

    #[test]
    fn destroy_cleanup_clears_dead_active_without_posting_inactive_app_window() -> Result<()> {
        let config = RuntimeConfig::load_default()?;
        let mut kernel = CeKernel::boot(config);
        let thread_id = 1;
        let parent = kernel.create_window_ex_w(thread_id, "TOP", "", None, 0, 0, 0);
        let child = kernel.create_window_ex_w(thread_id, "CHILD", "", Some(parent), 0, WS_CHILD, 0);

        assert_eq!(kernel.set_focus(Some(child)), None);
        assert_eq!(kernel.gwe.get_message(thread_id).unwrap().msg, WM_ACTIVATE);
        assert_eq!(kernel.gwe.get_message(thread_id).unwrap().msg, WM_SETFOCUS);
        // active = child after set_focus(child); set_active_window returns the previous active.
        assert_eq!(kernel.gwe.set_active_window(Some(child)), Some(child));

        kernel.clear_destroyed_window_focus_and_activation(child);

        let kill_focus = kernel.gwe.get_message(thread_id).unwrap();
        assert_eq!(kill_focus.hwnd, child);
        assert_eq!(kill_focus.msg, WM_KILLFOCUS);
        assert_eq!(kill_focus.wparam, 0);
        assert!(!kernel.gwe.active_window_is_within(child));
        assert_eq!(kernel.gwe.get_active_window(), Some(parent));
        assert!(kernel.gwe.get_message(thread_id).is_none());

        Ok(())
    }

    #[test]
    fn destroyed_window_restores_captured_framebuffer_backing_store() -> Result<()> {
        let config = RuntimeConfig::load_default()?;
        let mut kernel = CeKernel::boot(config);
        let mut framebuffer = crate::ce::framebuffer::VirtualFramebuffer::new(
            8,
            8,
            crate::ce::framebuffer::PixelFormat::Rgb565,
        )?;
        for (index, byte) in framebuffer.pixels_mut().iter_mut().enumerate() {
            *byte = index as u8;
        }
        let original = framebuffer.snapshot().pixels;
        let hwnd = kernel.create_window_ex_w_with_rect(
            1,
            "DIALOG",
            "",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(1, 1, 4, 4),
        );

        assert!(kernel.capture_window_backing_store(hwnd, &framebuffer));
        let info = framebuffer.info();
        let bytes_per_pixel = info.format.bytes_per_pixel();
        for y in 1usize..5 {
            for x in 1usize..5 {
                let offset = y * info.stride + x * bytes_per_pixel;
                framebuffer.pixels_mut()[offset..offset + bytes_per_pixel].fill(0xff);
            }
        }

        assert!(kernel.destroy_window_with_framebuffer(
            hwnd,
            "DestroyWindow",
            Some(&mut framebuffer)
        ));
        assert_eq!(framebuffer.pixels(), original.as_slice());
        Ok(())
    }

    #[test]
    fn destroy_window_with_framebuffer_restores_captured_backing_store() -> Result<()> {
        let config = RuntimeConfig::load_default()?;
        let mut kernel = CeKernel::boot(config);
        let mut framebuffer = crate::ce::framebuffer::VirtualFramebuffer::new(
            8,
            8,
            crate::ce::framebuffer::PixelFormat::Rgb565,
        )?;
        for (index, byte) in framebuffer.pixels_mut().iter_mut().enumerate() {
            *byte = index as u8;
        }
        let original = framebuffer.snapshot().pixels;
        let hwnd = kernel.create_window_ex_w_with_rect(
            1,
            "DIALOG",
            "",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(1, 1, 4, 4),
        );

        assert!(kernel.capture_window_backing_store(hwnd, &framebuffer));
        let info = framebuffer.info();
        let bytes_per_pixel = info.format.bytes_per_pixel();
        for y in 1usize..5 {
            for x in 1usize..5 {
                let offset = y * info.stride + x * bytes_per_pixel;
                framebuffer.pixels_mut()[offset..offset + bytes_per_pixel].fill(0xff);
            }
        }

        assert!(kernel.destroy_window_with_framebuffer(hwnd, "test", Some(&mut framebuffer)));
        assert_eq!(framebuffer.pixels(), original.as_slice());
        Ok(())
    }

    #[test]
    fn show_window_hide_restores_captured_framebuffer_backing_store() -> Result<()> {
        let config = RuntimeConfig::load_default()?;
        let mut kernel = CeKernel::boot(config);
        let mut framebuffer = crate::ce::framebuffer::VirtualFramebuffer::new(
            8,
            8,
            crate::ce::framebuffer::PixelFormat::Rgb565,
        )?;
        for (index, byte) in framebuffer.pixels_mut().iter_mut().enumerate() {
            *byte = index as u8;
        }
        let original = framebuffer.snapshot().pixels;
        let parent = kernel.create_window_ex_w_with_rect(
            1,
            "DIALOG",
            "",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(1, 1, 5, 5),
        );
        let child = kernel.create_window_ex_w_with_rect(
            1,
            "STATIC",
            "text",
            Some(parent),
            0,
            WS_VISIBLE | crate::ce::gwe::WS_CHILD,
            0,
            Rect::from_origin_size(2, 2, 2, 2),
        );

        assert!(kernel.capture_window_backing_store(parent, &framebuffer));
        assert!(kernel.capture_window_backing_store(child, &framebuffer));
        let info = framebuffer.info();
        let bytes_per_pixel = info.format.bytes_per_pixel();
        for y in 1usize..6 {
            for x in 1usize..6 {
                let offset = y * info.stride + x * bytes_per_pixel;
                framebuffer.pixels_mut()[offset..offset + bytes_per_pixel].fill(0xaa);
            }
        }
        for y in 2usize..4 {
            for x in 2usize..4 {
                let offset = y * info.stride + x * bytes_per_pixel;
                framebuffer.pixels_mut()[offset..offset + bytes_per_pixel].fill(0xbb);
            }
        }

        assert!(kernel.show_window_with_framebuffer(parent, false, false, Some(&mut framebuffer)));
        assert_eq!(framebuffer.pixels(), original.as_slice());
        Ok(())
    }

    #[test]
    fn set_window_pos_hide_restores_captured_framebuffer_backing_store() -> Result<()> {
        let config = RuntimeConfig::load_default()?;
        let mut kernel = CeKernel::boot(config);
        let mut framebuffer = crate::ce::framebuffer::VirtualFramebuffer::new(
            8,
            8,
            crate::ce::framebuffer::PixelFormat::Rgb565,
        )?;
        for (index, byte) in framebuffer.pixels_mut().iter_mut().enumerate() {
            *byte = index as u8;
        }
        let original = framebuffer.snapshot().pixels;
        let hwnd = kernel.create_window_ex_w_with_rect(
            1,
            "DIALOG",
            "",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(1, 1, 4, 4),
        );

        assert!(kernel.capture_window_backing_store(hwnd, &framebuffer));
        let info = framebuffer.info();
        let bytes_per_pixel = info.format.bytes_per_pixel();
        for y in 1usize..5 {
            for x in 1usize..5 {
                let offset = y * info.stride + x * bytes_per_pixel;
                framebuffer.pixels_mut()[offset..offset + bytes_per_pixel].fill(0xff);
            }
        }

        assert!(kernel.set_window_pos_with_framebuffer(
            hwnd,
            Some(HWND_TOP),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_HIDEWINDOW,
            Some(&mut framebuffer)
        ));
        assert_eq!(framebuffer.pixels(), original.as_slice());
        Ok(())
    }

    #[test]
    fn pending_child_birth_survives_last_public_process_handle_close() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        assert!(kernel.handles.process_id_exists(1));
        assert!(kernel.handles.process_alias_handles(1).is_empty());
        let launch = kernel.queue_process_launch(Some("\\Windows\\helper.exe".to_owned()), None);

        assert!(kernel.handles.process_id_exists(launch.process_id));
        assert_eq!(
            kernel.process_id(launch.process_handle),
            Some(launch.process_id)
        );
        assert!(kernel.close_handle(launch.process_handle)?);
        assert_eq!(kernel.process_id(launch.process_handle), None);

        // The queued child and its primary thread still own execution lifetime;
        // closing the last public process alias must not retire its birth.
        assert_eq!(kernel.pending_process_launches.len(), 1);
        assert_eq!(
            kernel.pending_process_launches[0].process_id,
            launch.process_id
        );
        assert_eq!(
            kernel.handles.thread_id(launch.thread_handle),
            Some(launch.thread_id)
        );

        let reopened = kernel.open_process_handle(launch.process_id).expect(
            "a pending child remains addressable by PID after its last process handle closes",
        );
        assert_eq!(kernel.process_id(reopened), Some(launch.process_id));
        Ok(())
    }

    #[test]
    fn vm_owner_key_resolves_birth_not_mutable_pid_selector_or_public_alias() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let boot = kernel.current_process_birth_key().expect("boot birth");
        assert_eq!(boot.process_id(), 1);
        assert!(kernel.handles.contains_process_birth(boot));

        let launch = kernel.queue_process_launch(Some("\\Windows\\helper.exe".to_owned()), None);
        kernel.set_current_process_id(launch.process_id);
        let child = kernel
            .current_process_birth_key()
            .expect("pending child birth");
        assert_eq!(child.process_id(), launch.process_id);
        assert_ne!(child, boot);
        assert!(kernel.close_handle(launch.process_handle)?);
        assert!(kernel.handles.contains_process_birth(child));
        assert_eq!(kernel.current_process_birth_key(), Some(child));

        kernel.set_current_process_id(0xdead);
        assert_eq!(kernel.current_process_birth_key(), None);
        kernel.set_current_process_id(launch.process_id);
        kernel.mark_process_launch_exited(&launch, 0);
        assert_eq!(kernel.current_process_birth_key(), None);
        assert!(!kernel.handles.contains_process_birth(child));
        Ok(())
    }

    #[test]
    fn selected_process_context_stamp_invalidates_on_every_context_writer() {
        fn check_write(kernel: &mut CeKernel, write: impl FnOnce(&mut CeKernel)) {
            let before = kernel
                .selected_process_context_stamp()
                .expect("selected boot birth");
            assert!(kernel.selected_process_context_matches(before));
            write(kernel);
            assert!(!kernel.selected_process_context_matches(before));
            let after = kernel
                .selected_process_context_stamp()
                .expect("same boot birth after context write");
            assert_eq!(after.birth(), before.birth());
            assert_eq!(after.generation(), before.generation() + 1);
            assert!(kernel.selected_process_context_matches(after));
        }

        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        check_write(&mut kernel, |k| k.set_process_module_base(0x1000));
        check_write(&mut kernel, |k| {
            k.set_process_module_path("\\Windows\\boot.exe")
        });
        check_write(&mut kernel, |k| {
            k.set_process_module_host_path("C:\\Windows\\boot.exe")
        });
        check_write(&mut kernel, |k| k.set_process_command_line("boot arg"));
        check_write(&mut kernel, |k| {
            k.set_process_current_directory(Some("\\Windows".to_owned()))
        });
        check_write(&mut kernel, |k| k.set_process_show_cmd(1));
        // Even a same-PID or same-value restore is a distinct selection write.
        check_write(&mut kernel, |k| k.set_current_process_id(1));
        check_write(&mut kernel, |k| {
            let state = k.current_process_state();
            k.set_current_process_state(state);
        });
        check_write(&mut kernel, CeKernel::reset_current_process_exit_state);
    }

    #[test]
    fn selected_process_context_stamp_rejects_aba_and_missing_birth() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let boot = kernel
            .selected_process_context_stamp()
            .expect("boot selection");
        let child = kernel.handles.create_process(77);
        kernel.set_current_process_id(77);
        let selected_child = kernel
            .selected_process_context_stamp()
            .expect("child selection");
        assert_ne!(selected_child.birth(), boot.birth());
        kernel.set_current_process_id(1);
        assert_eq!(kernel.current_process_birth_key(), Some(boot.birth()));
        assert!(!kernel.selected_process_context_matches(boot));
        assert!(!kernel.selected_process_context_matches(selected_child));

        kernel.set_current_process_id(0xdead);
        assert_eq!(kernel.selected_process_context_stamp(), None);
        assert!(!kernel.selected_process_context_matches(boot));
        assert_eq!(kernel.current_process_id(), 0xdead);
        assert!(kernel.close_handle(child).expect("close child alias"));
    }

    #[test]
    fn selected_process_context_stamp_rechecks_reused_pid_without_selector_write() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let handle_a = kernel.handles.create_process(77);
        kernel.set_current_process_id(77);
        let old = kernel
            .selected_process_context_stamp()
            .expect("first PID 77 birth");
        assert!(kernel.selected_process_context_matches(old));

        assert!(
            kernel
                .handles
                .finish_process_execution_by_birth(old.birth(), 0)
                .is_some()
        );
        assert!(kernel.close_handle(handle_a)?);
        assert_eq!(kernel.selected_process_context_stamp(), None);
        let handle_b = kernel.handles.create_process(77);
        let new_birth = kernel
            .current_process_birth_key()
            .expect("reused PID birth");
        assert_ne!(new_birth, old.birth());
        // HandleTable is public: its replacement did not advance the selector
        // generation. The birth check must still reject the stale stamp.
        assert_eq!(
            kernel.selected_process_context_generation,
            Some(old.generation())
        );
        assert!(!kernel.selected_process_context_matches(old));
        assert_eq!(
            kernel
                .selected_process_context_stamp()
                .map(|stamp| stamp.birth()),
            Some(new_birth)
        );
        assert!(kernel.close_handle(handle_b)?);
        Ok(())
    }

    #[test]
    fn selected_process_context_stamp_poisoned_on_generation_overflow() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        kernel.selected_process_context_generation = Some(u64::MAX - 1);
        let penultimate = kernel
            .selected_process_context_stamp()
            .expect("penultimate generation");
        kernel.set_current_process_id(1);
        let last = kernel
            .selected_process_context_stamp()
            .expect("last valid generation");
        assert_eq!(last.generation(), u64::MAX);
        assert!(!kernel.selected_process_context_matches(penultimate));

        kernel.set_current_process_id(1);
        assert_eq!(kernel.current_process_id(), 1);
        assert_eq!(kernel.selected_process_context_generation, None);
        assert_eq!(kernel.selected_process_context_stamp(), None);
        assert!(!kernel.selected_process_context_matches(last));
        kernel.reset_current_process_exit_state();
        assert_eq!(kernel.selected_process_context_stamp(), None);
    }

    #[test]
    fn selected_process_context_stamp_invalidates_on_both_accepted_exit_writes() -> Result<()> {
        let mut pseudo = CeKernel::boot(runtime_config_with_com7());
        let pseudo_stamp = pseudo
            .selected_process_context_stamp()
            .expect("boot pseudo selection");
        let wrong_handle = pseudo.handles.create_process(77);
        let wrong_birth = pseudo.handles.process_birth_key(77).expect("foreign birth");
        assert!(!pseudo.terminate_current_process_for_birth(wrong_birth, 0x40));
        assert!(pseudo.selected_process_context_matches(pseudo_stamp));
        assert!(pseudo.close_handle(wrong_handle)?);
        assert!(pseudo.terminate_current_process_for_birth(pseudo_stamp.birth(), 0x41));
        assert!(!pseudo.selected_process_context_matches(pseudo_stamp));
        assert_eq!(
            pseudo.selected_process_context_generation,
            Some(pseudo_stamp.generation() + 1)
        );

        let mut stopped = CeKernel::boot(runtime_config_with_com7());
        let stopped_stamp = stopped
            .selected_process_context_stamp()
            .expect("boot stopped selection");
        let wrong_handle = stopped.handles.create_process(77);
        let wrong_birth = stopped
            .handles
            .process_birth_key(77)
            .expect("foreign birth");
        assert!(
            !stopped.try_terminate_stopped_current_process_for_birth_with_framebuffer(
                wrong_birth,
                0x40,
                None
            )?
        );
        assert!(stopped.selected_process_context_matches(stopped_stamp));
        assert!(stopped.close_handle(wrong_handle)?);
        assert!(
            stopped.try_terminate_stopped_current_process_for_birth_with_framebuffer(
                stopped_stamp.birth(),
                0x42,
                None
            )?
        );
        assert!(!stopped.selected_process_context_matches(stopped_stamp));
        assert_eq!(
            stopped.selected_process_context_generation,
            Some(stopped_stamp.generation() + 1)
        );
        Ok(())
    }

    #[test]
    fn child_exit_retires_fixed_stack_pages_and_budget_across_fifty_births() -> Result<()> {
        const STACK_BASE: u32 = 0x7100_0000;
        const STACK_SIZE: u32 = 0x2000;
        const PAGE_SIZE: u64 = 0x1000;
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let baseline = kernel.memory.private_commit_used_bytes();

        for cycle in 0..50 {
            let launch =
                kernel.queue_process_launch(Some("\\Windows\\helper.exe".to_owned()), None);
            let owner = kernel
                .handles
                .process_birth_key(launch.process_id)
                .expect("queued child has an execution birth");
            assert_eq!(
                kernel.memory.reserve_fixed_private_range(
                    owner,
                    crate::ce::memory::VirtualAllocationSource::ThreadStack,
                    STACK_BASE,
                    STACK_SIZE,
                    0x04,
                ),
                Some(STACK_BASE),
                "cycle {cycle} should reserve only its own stack"
            );
            let prepared = kernel
                .memory
                .prepare_fixed_private_page(owner, STACK_BASE + 4)
                .expect("first stack touch should prepare one page");
            assert_eq!(
                kernel.memory.commit_prepared_private_vm(prepared),
                Some(STACK_BASE)
            );
            assert_eq!(
                kernel.memory.private_commit_used_bytes(),
                baseline + PAGE_SIZE
            );

            kernel.mark_process_launch_exited(&launch, 0);
            assert!(
                kernel
                    .memory
                    .virtual_allocation(owner, STACK_BASE)
                    .is_none()
            );
            assert!(kernel.memory.private_page_pin(owner, STACK_BASE).is_none());
            assert_eq!(
                kernel.memory.private_commit_used_bytes(),
                baseline,
                "cycle {cycle} must release the completed birth's canonical page"
            );
            assert!(kernel.close_handle(launch.process_handle)?);
            assert!(kernel.close_handle(launch.thread_handle)?);
        }
        Ok(())
    }

    #[test]
    fn child_exit_after_original_process_handle_close_updates_alias_wait_and_retirement()
    -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let launch = kernel.queue_process_launch(Some("\\Windows\\helper.exe".to_owned()), None);
        let first_alias = kernel
            .open_process_handle(launch.process_id)
            .expect("child birth");
        let second_alias = kernel
            .open_process_handle(launch.process_id)
            .expect("child birth");
        assert!(kernel.close_handle(launch.process_handle)?);
        assert_eq!(kernel.process_id(launch.process_handle), None);
        assert_eq!(kernel.is_wait_ready(first_alias, 1), Some(false));

        let wait_id = kernel.register_blocked_waiter(
            77,
            launch.thread_handle,
            vec![second_alias],
            SchedulerBlockedWaitKind::Kernel,
            kernel.timers.tick_count(),
            crate::ce::timer::INFINITE,
        );
        let wakes_before = kernel.scheduler_stats().object_wake_candidate_count;
        kernel.mark_process_launch_exited(&launch, 7);
        assert_eq!(kernel.process_exit_code(first_alias), Some(7));
        assert_eq!(kernel.process_exit_code(second_alias), Some(7));
        assert_eq!(kernel.is_wait_ready(first_alias, 1), Some(true));
        assert_eq!(
            kernel.wait_for_single_object(second_alias, 0, 1),
            WAIT_OBJECT_0
        );
        assert!(kernel.scheduler_stats().object_wake_candidate_count > wakes_before);
        assert_eq!(
            kernel.select_ready_blocked_waiter(0, kernel.timers.tick_count(), |wait, kernel| {
                wait.wait_handles
                    .iter()
                    .any(|handle| kernel.is_wait_ready(*handle, 77) == Some(true))
            }),
            Some(wait_id)
        );
        assert!(kernel.remove_blocked_waiter(wait_id).is_some());

        assert!(kernel.close_handle(first_alias)?);
        assert!(kernel.handles.process_id_exists(launch.process_id));
        assert!(kernel.close_handle(second_alias)?);
        assert!(!kernel.handles.process_id_exists(launch.process_id));
        assert_eq!(kernel.open_process_handle(launch.process_id), None);
        Ok(())
    }

    #[test]
    fn termination_request_keeps_child_openable_until_completion() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let launch = kernel.queue_process_launch(Some("\\Windows\\helper.exe".to_owned()), None);
        let alias = kernel
            .open_process_handle(launch.process_id)
            .expect("live child");
        assert!(kernel.terminate_process(launch.process_handle, 9));
        assert_eq!(kernel.process_exit_code(alias), Some(STILL_ACTIVE));
        assert_eq!(kernel.is_wait_ready(alias, 1), Some(false));
        assert!(kernel.close_handle(launch.process_handle)?);
        assert!(kernel.close_handle(alias)?);
        assert!(kernel.handles.process_id_exists(launch.process_id));
        let reopened = kernel
            .open_process_handle(launch.process_id)
            .expect("requested child remains execution-live");
        assert_eq!(kernel.is_wait_ready(reopened, 1), Some(false));
        assert!(kernel.close_handle(reopened)?);
        assert!(kernel.mark_process_launch_exited(&launch, 9));
        assert_eq!(kernel.open_process_handle(launch.process_id), None);
        Ok(())
    }

    #[test]
    fn selected_child_pseudo_termination_signals_public_alias() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let launch = kernel.queue_process_launch(Some("\\Windows\\helper.exe".to_owned()), None);
        kernel.set_current_process_id(launch.process_id);
        assert!(kernel.terminate_process(CE_CURRENT_PROCESS_PSEUDO_HANDLE, 11));
        assert_eq!(kernel.process_exit_code(launch.process_handle), Some(11));
        assert_eq!(kernel.is_wait_ready(launch.process_handle, 1), Some(true));
        kernel.set_current_process_id(1);
        assert!(kernel.close_handle(launch.process_handle)?);
        assert_eq!(kernel.open_process_handle(launch.process_id), None);
        kernel.mark_process_launch_exited(&launch, 11);
        Ok(())
    }

    #[test]
    fn completed_process_retires_after_final_alias_and_new_birth_is_distinct() -> Result<()> {
        let mut handles = HandleTable::default();
        let original = handles.create_process(77);
        let alias = handles.open_process(77).expect("live process birth");
        let duplicate = handles.insert(handles.get(original)?.clone());
        assert_eq!(handles.process_alias_handles(77).len(), 3);
        assert!(handles.set_process_debugger_thread_id(77, Some(123)));
        assert_eq!(handles.process_debugger_thread_id(77), Some(123));
        assert!(handles.mark_process_exited(original, 5));
        assert_eq!(handles.process_exit_code(alias), Some(5));
        assert_eq!(handles.is_wait_ready(alias, 1), Some(true));
        handles.close(original)?;
        assert!(handles.process_id_exists(77));
        handles.close(alias)?;
        assert!(handles.process_id_exists(77));
        handles.close(duplicate)?;
        assert!(!handles.process_id_exists(77));
        let replacement = handles.create_process(77);
        assert_ne!(replacement, original);
        assert_eq!(handles.process_exit_code(replacement), Some(STILL_ACTIVE));
        assert_eq!(handles.process_debugger_thread_id(77), None);
        Ok(())
    }

    #[test]
    fn process_exit_restores_backing_store_for_already_destroyed_window() -> Result<()> {
        let config = RuntimeConfig::load_default()?;
        let mut kernel = CeKernel::boot(config);
        let launch = kernel.queue_process_launch(Some("\\Windows\\helper.exe".to_owned()), None);
        kernel.set_current_process_id(launch.process_id);
        let mut framebuffer = crate::ce::framebuffer::VirtualFramebuffer::new(
            8,
            8,
            crate::ce::framebuffer::PixelFormat::Rgb565,
        )?;
        for (index, byte) in framebuffer.pixels_mut().iter_mut().enumerate() {
            *byte = index as u8;
        }
        let original = framebuffer.snapshot().pixels;
        let hwnd = kernel.create_window_ex_w_with_rect(
            launch.thread_id,
            "DIALOG",
            "",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(1, 1, 4, 4),
        );

        assert!(kernel.capture_window_backing_store(hwnd, &framebuffer));
        let info = framebuffer.info();
        let bytes_per_pixel = info.format.bytes_per_pixel();
        for y in 1usize..5 {
            for x in 1usize..5 {
                let offset = y * info.stride + x * bytes_per_pixel;
                framebuffer.pixels_mut()[offset..offset + bytes_per_pixel].fill(0xff);
            }
        }

        assert!(kernel.gwe.destroy_window(hwnd, kernel.timers.tick_count()));
        let owner = launch.process_birth.expect("launch birth");
        assert!(
            !kernel
                .process_window_targets_for_birth(owner)
                .contains(&hwnd)
        );
        assert!(
            kernel
                .process_window_backing_store_targets_for_birth(owner)
                .contains(&hwnd)
        );

        kernel.mark_process_launch_exited_with_framebuffer(&launch, 0, Some(&mut framebuffer));

        assert_eq!(framebuffer.pixels(), original.as_slice());
        Ok(())
    }

    #[test]
    fn host_exit_framebuffer_preflight_rejects_destroyed_and_live_backing_mismatch_before_effects()
    -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (sender_handle, _) = kernel.create_guest_thread(0x401000, 0, false);
        let sender = kernel
            .handles
            .thread_birth_key_for_handle(sender_handle)
            .expect("sender birth");
        let launch = kernel.queue_process_launch(None, None);
        let owner = launch.process_birth.expect("exiting process birth");
        let receiver = launch.primary_thread_birth.expect("receiver birth");
        kernel.set_current_process_id(launch.process_id);

        let mut source = crate::ce::framebuffer::VirtualFramebuffer::new(
            8,
            8,
            crate::ce::framebuffer::PixelFormat::Rgb565,
        )?;
        for (index, byte) in source.pixels_mut().iter_mut().enumerate() {
            *byte = index as u8;
        }
        let original = source.snapshot().pixels;
        let destroyed = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                receiver,
                "EXIT_BACKING_DEAD",
                "",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(0, 0, 4, 4),
            )
            .expect("destroyed HWND birth");
        let live = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                receiver,
                "EXIT_BACKING_LIVE",
                "",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(4, 0, 4, 4),
            )
            .expect("live HWND birth");
        assert!(kernel.capture_window_backing_store(destroyed, &source));
        assert!(kernel.capture_window_backing_store(live, &source));
        assert!(
            kernel
                .gwe
                .destroy_window(destroyed, kernel.timers.tick_count())
        );
        assert!(!kernel.gwe.is_window(destroyed));
        assert!(kernel.gwe.is_window(live));
        assert_eq!(
            kernel.process_window_backing_store_targets_for_birth(owner),
            vec![destroyed, live],
            "the dead HWND's backing must still be selected by its frozen birth"
        );

        kernel.set_current_process_id(1);
        let send_id = kernel
            .begin_cross_thread_send_message_w_for_birth(
                sender,
                live,
                crate::ce::gwe::WM_USER + 0xd1,
                0,
                0,
                None,
            )
            .expect("queued A-to-B send");
        let wait_id = kernel
            .try_register_opaque_waiter_for_birth(
                sender,
                sender_handle,
                Vec::new(),
                SchedulerBlockedWaitKind::SendMessage { send_id },
                0,
                crate::ce::timer::INFINITE,
            )
            .expect("exact sender wait");
        assert!(!kernel.sent_message_result_ready(send_id));
        assert!(kernel.blocked_waiter(wait_id).is_some());
        let backings_before = kernel.window_backing_stores.clone();
        let scheduler_before = kernel.scheduler_stats();
        let message_trace_count = kernel.recent_message_ops.len();
        let process_trace_count = kernel.recent_process_ops.len();

        let mut incompatible = crate::ce::framebuffer::VirtualFramebuffer::new(
            8,
            8,
            crate::ce::framebuffer::PixelFormat::Bgra8888,
        )?;
        incompatible.pixels_mut().fill(0x5a);
        let pixels_before = incompatible.snapshot().pixels;
        let dirty_before = incompatible.dirty_rects().to_vec();
        assert!(kernel.handles.process_execution_live_by_birth(owner));
        assert_eq!(
            kernel.handles.process_signaled_by_id(launch.process_id),
            Some(false)
        );

        assert!(
            !kernel.mark_process_launch_exited_with_framebuffer(
                &launch,
                0x42,
                Some(&mut incompatible),
            ),
            "an incompatible saved backing must stop host exit before mutation"
        );
        assert!(matches!(
            kernel.try_mark_process_launch_exited_with_framebuffer(
                &launch,
                0x42,
                Some(&mut incompatible),
            ),
            Err(Error::HostExitRecoveryRequired { process_id, hwnd: Some(_), .. })
                if process_id == launch.process_id
        ));
        assert!(kernel.handles.process_execution_live_by_birth(owner));
        assert_eq!(
            kernel.handles.process_signaled_by_id(launch.process_id),
            Some(false)
        );
        assert!(kernel.handles.contains_thread_birth(receiver));
        assert!(kernel.gwe.is_window(live));
        assert!(!kernel.gwe.is_window(destroyed));
        assert_eq!(kernel.window_backing_stores, backings_before);
        assert!(!kernel.sent_message_result_ready(send_id));
        assert!(kernel.blocked_waiter(wait_id).is_some());
        assert_eq!(kernel.scheduler_stats(), scheduler_before);
        assert_eq!(kernel.recent_message_ops.len(), message_trace_count);
        assert_eq!(kernel.recent_process_ops.len(), process_trace_count);
        assert_eq!(incompatible.pixels(), pixels_before.as_slice());
        assert_eq!(incompatible.dirty_rects(), dirty_before.as_slice());

        // The refused completion leaves the stopped process recoverable. A
        // compatible destination can later restore both saved regions and
        // perform the one authorized exit.
        source.pixels_mut().fill(0xee);
        assert!(kernel.mark_process_launch_exited_with_framebuffer(
            &launch,
            0x42,
            Some(&mut source),
        ));
        let stride = source.info().stride;
        for row in 0..4 {
            let span = row * stride..row * stride + stride;
            assert_eq!(&source.pixels()[span.clone()], &original[span]);
        }
        assert!(!kernel.gwe.is_window(live));
        assert!(!kernel.window_backing_stores.contains_key(&destroyed));
        assert!(!kernel.window_backing_stores.contains_key(&live));
        assert!(kernel.sent_message_result_ready(send_id));
        assert!(!kernel.handles.process_execution_live_by_birth(owner));
        Ok(())
    }

    #[test]
    fn host_exit_framebuffer_preflight_rejects_partial_multiwindow_restore() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let launch = kernel.queue_process_launch(None, None);
        let owner = launch.process_birth.expect("exiting process birth");
        let receiver = launch.primary_thread_birth.expect("receiver birth");
        kernel.set_current_process_id(launch.process_id);
        let valid = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                receiver,
                "EXIT_VALID_BACKING",
                "",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(0, 0, 4, 4),
            )
            .expect("valid backing HWND");
        let invalid = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                receiver,
                "EXIT_INVALID_BACKING",
                "",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(4, 0, 4, 4),
            )
            .expect("invalid backing HWND");
        let mut destination = crate::ce::framebuffer::VirtualFramebuffer::new(
            8,
            8,
            crate::ce::framebuffer::PixelFormat::Rgb565,
        )?;
        destination.pixels_mut().fill(0x11);
        let mut wrong_format = crate::ce::framebuffer::VirtualFramebuffer::new(
            8,
            8,
            crate::ce::framebuffer::PixelFormat::Bgra8888,
        )?;
        wrong_format.pixels_mut().fill(0x22);
        assert!(kernel.capture_window_backing_store(valid, &destination));
        assert!(kernel.capture_window_backing_store(invalid, &wrong_format));
        assert_eq!(
            kernel.process_window_backing_store_targets_for_birth(owner),
            vec![valid, invalid],
            "the restorable store must precede the incompatible store"
        );
        let backings_before = kernel.window_backing_stores.clone();
        destination.pixels_mut().fill(0xa5);
        let pixels_before = destination.snapshot().pixels;
        let dirty_before = destination.dirty_rects().to_vec();
        let message_trace_count = kernel.recent_message_ops.len();
        let process_trace_count = kernel.recent_process_ops.len();

        assert!(
            !kernel.mark_process_launch_exited_with_framebuffer(
                &launch,
                0x43,
                Some(&mut destination),
            ),
            "one incompatible backing must reject the entire multiwindow exit"
        );
        assert_eq!(destination.pixels(), pixels_before.as_slice());
        assert_eq!(destination.dirty_rects(), dirty_before.as_slice());
        assert_eq!(kernel.window_backing_stores, backings_before);
        assert!(kernel.gwe.is_window(valid));
        assert!(kernel.gwe.is_window(invalid));
        assert!(kernel.handles.process_execution_live_by_birth(owner));
        assert_eq!(
            kernel.handles.process_signaled_by_id(launch.process_id),
            Some(false)
        );
        assert_eq!(kernel.recent_message_ops.len(), message_trace_count);
        assert_eq!(kernel.recent_process_ops.len(), process_trace_count);
        Ok(())
    }

    #[test]
    fn host_exit_framebuffer_preflight_requires_destination_for_saved_backing() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let launch = kernel.queue_process_launch(None, None);
        let owner = launch.process_birth.expect("exiting process birth");
        let receiver = launch.primary_thread_birth.expect("receiver birth");
        kernel.set_current_process_id(launch.process_id);
        let hwnd = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                receiver,
                "EXIT_REQUIRES_FRAMEBUFFER",
                "",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(0, 0, 4, 4),
            )
            .expect("owned window");
        let mut framebuffer = crate::ce::framebuffer::VirtualFramebuffer::new(
            8,
            8,
            crate::ce::framebuffer::PixelFormat::Rgb565,
        )?;
        assert!(kernel.capture_window_backing_store(hwnd, &framebuffer));
        let backing_before = kernel.window_backing_stores[&hwnd].clone();

        assert!(!kernel.mark_process_launch_exited(&launch, 0x44));
        assert!(matches!(
            kernel.try_mark_process_launch_exited_with_framebuffer(&launch, 0x44, None),
            Err(Error::HostExitRecoveryRequired { process_id, hwnd: Some(_), .. })
                if process_id == launch.process_id
        ));
        assert_eq!(
            kernel.window_backing_stores.get(&hwnd),
            Some(&backing_before)
        );
        assert!(kernel.gwe.is_window(hwnd));
        assert!(kernel.handles.process_execution_live_by_birth(owner));
        assert_eq!(
            kernel.handles.process_signaled_by_id(launch.process_id),
            Some(false)
        );

        assert!(kernel.try_mark_process_launch_exited_with_framebuffer(
            &launch,
            0x44,
            Some(&mut framebuffer),
        )?);
        assert!(!kernel.gwe.is_window(hwnd));
        assert!(!kernel.handles.process_execution_live_by_birth(owner));
        Ok(())
    }

    #[test]
    fn stopped_root_exit_requires_exact_birth_and_restorable_backing_before_effects() -> Result<()>
    {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let root = kernel
            .current_process_birth_key()
            .expect("root process birth");
        let (thread_handle, _) = kernel.create_guest_thread(0x401000, 0, false);
        let thread = kernel
            .handles
            .thread_birth_key_for_handle(thread_handle)
            .expect("root thread birth");
        let hwnd = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                thread,
                "ROOT_EXIT_BACKING",
                "",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(0, 0, 4, 4),
            )
            .expect("root-owned HWND");
        let mut source = crate::ce::framebuffer::VirtualFramebuffer::new(
            8,
            8,
            crate::ce::framebuffer::PixelFormat::Rgb565,
        )?;
        for (index, byte) in source.pixels_mut().iter_mut().enumerate() {
            *byte = index as u8;
        }
        let original = source.snapshot().pixels;
        assert!(kernel.capture_window_backing_store(hwnd, &source));
        let backing_before = kernel.window_backing_stores[&hwnd].clone();
        let other = kernel.queue_process_launch(None, None);
        kernel.set_current_process_id(other.process_id);
        assert!(
            !kernel.try_terminate_stopped_current_process_for_birth_with_framebuffer(
                root,
                0x51,
                Some(&mut source),
            )?
        );
        assert!(kernel.handles.process_execution_live_by_birth(root));
        assert!(kernel.gwe.is_window(hwnd));
        assert_eq!(
            kernel.window_backing_stores.get(&hwnd),
            Some(&backing_before)
        );
        kernel.set_current_process_id(root.process_id());
        let message_trace_count = kernel.recent_message_ops.len();
        let process_trace_count = kernel.recent_process_ops.len();

        assert!(matches!(
            kernel.try_terminate_stopped_current_process_for_birth_with_framebuffer(
                root, 0x51, None,
            ),
            Err(Error::HostExitRecoveryRequired { process_id, hwnd: Some(rejected), .. })
                if process_id == root.process_id() && rejected == hwnd
        ));
        let mut incompatible = crate::ce::framebuffer::VirtualFramebuffer::new(
            8,
            8,
            crate::ce::framebuffer::PixelFormat::Bgra8888,
        )?;
        incompatible.pixels_mut().fill(0x5a);
        let incompatible_before = incompatible.snapshot().pixels;
        let dirty_before = incompatible.dirty_rects().to_vec();
        assert!(matches!(
            kernel.try_terminate_stopped_current_process_for_birth_with_framebuffer(
                root,
                0x51,
                Some(&mut incompatible),
            ),
            Err(Error::HostExitRecoveryRequired { process_id, hwnd: Some(rejected), .. })
                if process_id == root.process_id() && rejected == hwnd
        ));
        assert!(kernel.handles.process_execution_live_by_birth(root));
        assert_eq!(
            kernel.handles.process_signaled_by_id(root.process_id()),
            Some(false)
        );
        assert!(kernel.gwe.is_window(hwnd));
        assert_eq!(
            kernel.window_backing_stores.get(&hwnd),
            Some(&backing_before)
        );
        assert_eq!(kernel.recent_message_ops.len(), message_trace_count);
        assert_eq!(kernel.recent_process_ops.len(), process_trace_count);
        assert_eq!(incompatible.pixels(), incompatible_before.as_slice());
        assert_eq!(incompatible.dirty_rects(), dirty_before.as_slice());

        source.pixels_mut().fill(0xee);
        assert!(
            kernel.try_terminate_stopped_current_process_for_birth_with_framebuffer(
                root,
                0x51,
                Some(&mut source),
            )?
        );
        for row in 0..4 {
            let start = row * source.info().stride;
            let end = start + 4 * source.info().format.bytes_per_pixel();
            assert_eq!(&source.pixels()[start..end], &original[start..end]);
        }
        assert!(!kernel.gwe.is_window(hwnd));
        assert!(!kernel.window_backing_stores.contains_key(&hwnd));
        assert!(!kernel.handles.process_execution_live_by_birth(root));
        assert_eq!(
            kernel.handles.process_signaled_by_id(root.process_id()),
            None
        );
        assert_eq!(kernel.current_process_state().exit_code, 0x51);
        assert!(kernel.current_process_state().signaled);
        Ok(())
    }

    #[test]
    fn host_exit_writable_image_retirement_waits_for_framebuffer_and_exact_birth() -> Result<()> {
        const MODULE_BASE: u32 = 0x6400_0000;
        const RUN_BASE: u32 = MODULE_BASE + 0x1000;
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let foreign = kernel.current_process_birth_key().expect("boot birth");
        let launch = kernel.queue_process_launch(None, None);
        let owner = launch.process_birth.expect("child birth");
        let primary = launch.primary_thread_birth.expect("child primary birth");
        let identity = WritableImageRunIdentity::new(owner, MODULE_BASE, 1, RUN_BASE)
            .expect("child image identity");
        let foreign_identity = WritableImageRunIdentity::new(foreign, MODULE_BASE, 1, RUN_BASE)
            .expect("foreign image identity at the same VA");
        let bytes = vec![0x5a; 0x1000];
        let pin = kernel
            .memory
            .commit_prepared_writable_image_run(
                kernel
                    .memory
                    .prepare_writable_image_run(identity, &bytes)
                    .expect("prepare child image"),
            )
            .expect("publish child image");
        let foreign_pin = kernel
            .memory
            .commit_prepared_writable_image_run(
                kernel
                    .memory
                    .prepare_writable_image_run(foreign_identity, &[0x7b; 0x1000])
                    .expect("prepare foreign image"),
            )
            .expect("publish foreign image");
        let future_identity =
            WritableImageRunIdentity::new(owner, MODULE_BASE + 0x10000, 1, MODULE_BASE + 0x11000)
                .expect("detached future image identity");
        let stale_prepared = kernel
            .memory
            .prepare_writable_image_run(future_identity, &bytes)
            .expect("prepare before process exit");

        kernel.set_current_process_id(launch.process_id);
        let hwnd = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                primary,
                "IMAGE_EXIT_BACKING",
                "",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(0, 0, 4, 4),
            )
            .expect("child-owned window");
        let mut framebuffer = crate::ce::framebuffer::VirtualFramebuffer::new(
            8,
            8,
            crate::ce::framebuffer::PixelFormat::Rgb565,
        )?;
        assert!(kernel.capture_window_backing_store(hwnd, &framebuffer));
        let mut incompatible = crate::ce::framebuffer::VirtualFramebuffer::new(
            8,
            8,
            crate::ce::framebuffer::PixelFormat::Bgra8888,
        )?;

        let mut no_birth = launch.clone();
        no_birth.process_birth = None;
        assert!(!kernel.try_mark_process_launch_exited_with_framebuffer(
            &no_birth,
            0x51,
            Some(&mut framebuffer),
        )?);
        let mut wrong_birth = launch.clone();
        wrong_birth.process_birth = Some(foreign);
        assert!(!kernel.try_mark_process_launch_exited_with_framebuffer(
            &wrong_birth,
            0x51,
            Some(&mut framebuffer),
        )?);
        assert!(matches!(
            kernel.try_mark_process_launch_exited_with_framebuffer(
                &launch,
                0x51,
                Some(&mut incompatible),
            ),
            Err(Error::HostExitRecoveryRequired { process_id, hwnd: Some(rejected), .. })
                if process_id == launch.process_id && rejected == hwnd
        ));
        assert!(kernel.handles.process_execution_live_by_birth(owner));
        assert!(kernel.memory.contains_writable_image_run_pin(&pin));
        assert!(kernel.memory.contains_writable_image_run_pin(&foreign_pin));
        let mut original = [0; 4];
        pin.read_bytes(0, &mut original)
            .expect("refused exit preserves live image bytes");
        assert_eq!(original, [0x5a; 4]);

        // A stopped native owner may still have a strong backing pin. VM
        // retirement removes publication authority, not that allocation.
        let native_access = pin.begin_native_access().expect("native pin");
        let retained_pin = pin.clone();
        assert!(kernel.try_mark_process_launch_exited_with_framebuffer(
            &launch,
            0x51,
            Some(&mut framebuffer),
        )?);
        assert!(!kernel.handles.process_execution_live_by_birth(owner));
        assert!(!kernel.memory.contains_writable_image_run_pin(&pin));
        assert!(kernel.memory.contains_writable_image_run_pin(&foreign_pin));
        assert_eq!(
            pin.begin_native_access().err(),
            Some(WritableImageRunAccessError::Retired)
        );
        assert_eq!(
            pin.read_bytes(0, &mut original),
            Err(WritableImageRunAccessError::NativeExecutionActive)
        );
        assert!(
            kernel
                .memory
                .commit_prepared_writable_image_run(stale_prepared)
                .is_none()
        );
        assert!(
            kernel
                .memory
                .prepare_writable_image_run(future_identity, &bytes)
                .is_none()
        );
        assert_eq!(
            kernel
                .memory
                .retire_writable_image_module(owner, MODULE_BASE, 1),
            None
        );
        assert!(!kernel.try_mark_process_launch_exited_with_framebuffer(
            &launch,
            0x51,
            Some(&mut framebuffer),
        )?);

        drop(pin);
        assert!(
            kernel
                .memory
                .overlaps_writable_image_guest_range_for_owner(owner, RUN_BASE, 0x1000)
        );
        drop(native_access);
        retained_pin
            .read_bytes(0, &mut original)
            .expect("retained pin keeps retired bytes alive after native close");
        assert_eq!(original, [0x5a; 4]);
        assert!(
            kernel
                .memory
                .overlaps_writable_image_guest_range_for_owner(owner, RUN_BASE, 0x1000)
        );
        drop(retained_pin);
        assert!(
            !kernel
                .memory
                .overlaps_writable_image_guest_range_for_owner(owner, RUN_BASE, 0x1000)
        );
        assert!(
            kernel
                .memory
                .overlaps_writable_image_guest_range_for_owner(foreign, RUN_BASE, 0x1000)
        );
        foreign_pin
            .read_bytes(0, &mut original)
            .expect("foreign same-VA image survives child exit");
        assert_eq!(original, [0x7b; 4]);
        Ok(())
    }

    #[test]
    fn stopped_root_exit_retires_only_root_writable_image_birth() -> Result<()> {
        const MODULE_BASE: u32 = 0x6500_0000;
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let root = kernel.current_process_birth_key().expect("root birth");
        let identity = WritableImageRunIdentity::new(root, MODULE_BASE, 1, MODULE_BASE)
            .expect("root image identity");
        let pin = kernel
            .memory
            .commit_prepared_writable_image_run(
                kernel
                    .memory
                    .prepare_writable_image_run(identity, &[0x5a; 0x1000])
                    .expect("prepare root image"),
            )
            .expect("publish root image");
        let foreign_launch = kernel.queue_process_launch(None, None);
        kernel.set_current_process_id(foreign_launch.process_id);
        assert!(
            !kernel.try_terminate_stopped_current_process_for_birth_with_framebuffer(
                root, 0x52, None,
            )?
        );
        assert!(kernel.memory.contains_writable_image_run_pin(&pin));

        kernel.set_current_process_id(root.process_id());
        assert!(
            kernel.try_terminate_stopped_current_process_for_birth_with_framebuffer(
                root, 0x52, None,
            )?
        );
        assert!(!kernel.memory.contains_writable_image_run_pin(&pin));
        assert_eq!(
            pin.begin_native_access().err(),
            Some(WritableImageRunAccessError::Retired)
        );
        assert!(
            kernel.handles.process_execution_live_by_birth(
                foreign_launch.process_birth.expect("foreign birth")
            )
        );
        Ok(())
    }

    #[test]
    fn reused_pid_host_teardown_preserves_stale_prior_birth_window_and_backing() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let mut framebuffer = crate::ce::framebuffer::VirtualFramebuffer::new(
            64,
            64,
            crate::ce::framebuffer::PixelFormat::Rgb565,
        )?;
        let launch_a = kernel.queue_process_launch(None, None);
        let process_a = launch_a.process_birth.expect("A process birth");
        let thread_a = launch_a.primary_thread_birth.expect("A thread birth");
        kernel.set_current_process_id(launch_a.process_id);
        let hwnd_a = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                thread_a,
                "STALE_PID_A",
                "A",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(0, 0, 16, 16),
            )
            .expect("A exact-birth window");
        assert!(kernel.capture_window_backing_store(hwnd_a, &framebuffer));
        let backing_a = kernel.window_backing_stores[&hwnd_a].clone();

        // Fixture-only: retire A's canonical task/process without the normal
        // host-window teardown, leaving an inert prior-birth HWND/backing.
        // B's later teardown must not claim that artifact by recycled PID.
        assert!(kernel.mark_guest_thread_exited_by_birth(thread_a, 0));
        assert!(kernel.close_handle(launch_a.thread_handle)?);
        assert!(
            kernel
                .handles
                .finish_process_execution_by_birth(process_a, 0)
                .is_some()
        );
        assert!(kernel.close_handle(launch_a.process_handle)?);
        assert_eq!(kernel.handles.process_birth_key(launch_a.process_id), None);
        assert!(kernel.gwe.is_window(hwnd_a));
        assert_eq!(
            kernel
                .gwe
                .window(hwnd_a)
                .map(|window| window.delivery_owner()),
            Some(WindowDeliveryOwner::Birth {
                thread: thread_a,
                process: process_a,
            })
        );

        kernel.next_process_id = launch_a.process_id;
        let launch_b = kernel.queue_process_launch(None, None);
        let process_b = launch_b.process_birth.expect("B process birth");
        let thread_b = launch_b.primary_thread_birth.expect("B thread birth");
        assert_eq!(launch_b.process_id, launch_a.process_id);
        assert_ne!(process_b, process_a);
        kernel.set_current_process_id(launch_b.process_id);
        let hwnd_b = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                thread_b,
                "REUSED_PID_B",
                "B",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(20, 20, 16, 16),
            )
            .expect("B exact-birth window");
        assert!(kernel.capture_window_backing_store(hwnd_b, &framebuffer));
        assert_eq!(
            kernel
                .gwe
                .window(hwnd_b)
                .map(|window| window.delivery_owner()),
            Some(WindowDeliveryOwner::Birth {
                thread: thread_b,
                process: process_b,
            })
        );

        assert!(kernel.mark_process_launch_exited_with_framebuffer(
            &launch_b,
            0,
            Some(&mut framebuffer),
        ));
        assert!(!kernel.gwe.is_window(hwnd_b));
        assert!(!kernel.window_backing_stores.contains_key(&hwnd_b));
        assert!(
            kernel.gwe.is_window(hwnd_a),
            "B teardown must not destroy an HWND owned by prior process birth A"
        );
        assert_eq!(
            kernel
                .gwe
                .window(hwnd_a)
                .map(|window| window.delivery_owner()),
            Some(WindowDeliveryOwner::Birth {
                thread: thread_a,
                process: process_a,
            })
        );
        assert_eq!(kernel.window_backing_stores.get(&hwnd_a), Some(&backing_a));
        Ok(())
    }

    fn host_exit_preserves_foreign_linked_window(owner_link: bool) -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let process_a = kernel
            .current_process_birth_key()
            .expect("boot process birth");
        let (thread_a_handle, _) = kernel.create_guest_thread(0x401000, 0, false);
        let thread_a = kernel
            .handles
            .thread_birth_key_for_handle(thread_a_handle)
            .expect("A thread birth");
        let launch_b = kernel.queue_process_launch(None, None);
        let process_b = launch_b.process_birth.expect("B process birth");
        let thread_b = launch_b.primary_thread_birth.expect("B thread birth");
        assert_ne!(process_a, process_b);

        kernel.set_current_process_id(launch_b.process_id);
        let hwnd_b = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                thread_b,
                "EXIT_PARENT_B",
                "B",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(0, 0, 16, 16),
            )
            .expect("B exact-birth window");
        kernel.set_current_process_id(process_a.process_id());
        let hwnd_a = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                thread_a,
                "FOREIGN_LINK_A",
                "A",
                (!owner_link).then_some(hwnd_b),
                owner_link.then_some(hwnd_b),
                0,
                WS_VISIBLE
                    | if owner_link {
                        crate::ce::gwe::WS_POPUP
                    } else {
                        0
                    },
                0,
                Rect::from_origin_size(20, 20, 16, 16),
            )
            .expect("A exact-birth window with synthetic foreign link");
        assert_eq!(
            kernel
                .gwe
                .window(hwnd_a)
                .map(|window| window.delivery_owner()),
            Some(WindowDeliveryOwner::Birth {
                thread: thread_a,
                process: process_a,
            })
        );
        assert_eq!(
            kernel.gwe.window_and_descendants(hwnd_b),
            Some(vec![hwnd_b, hwnd_a])
        );

        let mut framebuffer = crate::ce::framebuffer::VirtualFramebuffer::new(
            64,
            64,
            crate::ce::framebuffer::PixelFormat::Rgb565,
        )?;
        assert!(kernel.capture_window_backing_store(hwnd_b, &framebuffer));
        assert!(kernel.capture_window_backing_store(hwnd_a, &framebuffer));
        let backing_a = kernel.window_backing_stores[&hwnd_a].clone();
        let position = WindowPos {
            hwnd: hwnd_a,
            insert_after: HWND_TOP,
            x: 20,
            y: 20,
            width: 16,
            height: 16,
            flags: 0,
        };
        let (payload_owner, ptr_a) = kernel
            .queue_window_pos_payload(hwnd_a, position)
            .expect("A typed payload");
        assert_eq!(payload_owner, process_a);
        assert!(kernel.post_window_typed_payload_message(
            hwnd_a,
            WM_WINDOWPOSCHANGED,
            0,
            ptr_a,
            process_a,
        ));
        let timer_a = kernel.set_timer(Some(hwnd_a), Some(0x4455), 60_000);
        assert_eq!(timer_a, 0x4455);
        kernel
            .shell
            .register_change_notification(ShellChangeNotifyRegistration {
                hwnd: hwnd_a,
                event_mask: 1,
                notify_flags: 0,
                watch_dir: None,
                recursive: false,
            });
        assert!(kernel.shell.change_notification(hwnd_a).is_some());
        assert!(
            kernel
                .peek_ready_message_w_filtered(
                    thread_a.thread_id(),
                    Some(hwnd_a),
                    WM_WINDOWPOSCHANGED,
                    WM_WINDOWPOSCHANGED,
                )
                .is_some()
        );

        assert!(kernel.mark_process_launch_exited_with_framebuffer(
            &launch_b,
            0,
            Some(&mut framebuffer),
        ));
        assert!(!kernel.gwe.is_window(hwnd_b));
        assert!(!kernel.window_backing_stores.contains_key(&hwnd_b));
        assert!(
            kernel.gwe.is_window(hwnd_a),
            "B exit must not retire A through a foreign parent/owner link"
        );
        assert_eq!(
            kernel
                .gwe
                .window(hwnd_a)
                .map(|window| window.delivery_owner()),
            Some(WindowDeliveryOwner::Birth {
                thread: thread_a,
                process: process_a,
            })
        );
        assert_eq!(kernel.window_backing_stores.get(&hwnd_a), Some(&backing_a));
        assert!(kernel.shell.change_notification(hwnd_a).is_some());
        assert!(
            kernel
                .peek_ready_message_w_filtered(
                    thread_a.thread_id(),
                    Some(hwnd_a),
                    WM_WINDOWPOSCHANGED,
                    WM_WINDOWPOSCHANGED,
                )
                .is_some()
        );
        assert!(
            kernel
                .gwe
                .message_pointer_payload(process_a, ptr_a)
                .is_some()
        );
        assert!(
            kernel
                .timers
                .pending_timers()
                .iter()
                .any(|timer| { timer.hwnd == Some(hwnd_a) && timer.id == timer_a })
        );
        Ok(())
    }

    #[test]
    fn host_exit_foreign_parent_link_preserves_other_birth_window() -> Result<()> {
        host_exit_preserves_foreign_linked_window(false)
    }

    #[test]
    fn host_exit_foreign_owner_link_preserves_other_birth_window() -> Result<()> {
        host_exit_preserves_foreign_linked_window(true)
    }

    #[test]
    fn host_exit_retires_all_same_birth_child_and_owned_windows() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let launch = kernel.queue_process_launch(None, None);
        let owner = launch.process_birth.expect("process birth");
        let thread = launch.primary_thread_birth.expect("thread birth");
        kernel.set_current_process_id(launch.process_id);
        let root = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                thread,
                "EXIT_ROOT",
                "",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(0, 0, 16, 16),
            )
            .expect("root");
        let child = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                thread,
                "EXIT_CHILD",
                "",
                Some(root),
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(16, 0, 16, 16),
            )
            .expect("child");
        let owned = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                thread,
                "EXIT_OWNED",
                "",
                None,
                Some(root),
                0,
                WS_VISIBLE | crate::ce::gwe::WS_POPUP,
                0,
                Rect::from_origin_size(32, 0, 16, 16),
            )
            .expect("owned popup");
        let mut framebuffer = crate::ce::framebuffer::VirtualFramebuffer::new(
            64,
            64,
            crate::ce::framebuffer::PixelFormat::Rgb565,
        )?;
        for hwnd in [root, child, owned] {
            assert!(kernel.capture_window_backing_store(hwnd, &framebuffer));
        }
        let position = WindowPos {
            hwnd: child,
            insert_after: HWND_TOP,
            x: 16,
            y: 0,
            width: 16,
            height: 16,
            flags: 0,
        };
        let (payload_owner, payload) = kernel
            .queue_window_pos_payload(child, position)
            .expect("child payload");
        assert_eq!(payload_owner, owner);
        assert!(kernel.post_window_typed_payload_message(
            child,
            WM_WINDOWPOSCHANGED,
            0,
            payload,
            owner,
        ));
        let child_timer = kernel.set_timer(Some(child), Some(0x4555), 60_000);
        assert_eq!(child_timer, 0x4555);
        kernel
            .shell
            .register_change_notification(ShellChangeNotifyRegistration {
                hwnd: child,
                event_mask: 1,
                notify_flags: 0,
                watch_dir: None,
                recursive: false,
            });
        assert!(kernel.shell.change_notification(child).is_some());

        assert!(kernel.mark_process_launch_exited_with_framebuffer(
            &launch,
            0,
            Some(&mut framebuffer),
        ));
        for hwnd in [root, child, owned] {
            assert!(!kernel.gwe.is_window(hwnd));
            assert!(!kernel.window_backing_stores.contains_key(&hwnd));
        }
        assert!(kernel.gwe.message_pointer_payload(owner, payload).is_none());
        assert!(kernel.shell.change_notification(child).is_none());
        assert!(
            !kernel
                .timers
                .pending_timers()
                .iter()
                .any(|timer| timer.hwnd == Some(child) && timer.id == child_timer)
        );
        Ok(())
    }

    #[test]
    fn host_exit_wakes_exact_sender_waiting_on_retired_receiver_window() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (sender_handle, _) = kernel.create_guest_thread(0x401000, 0, false);
        let sender_birth = kernel
            .handles
            .thread_birth_key_for_handle(sender_handle)
            .expect("A sender birth");
        let launch_b = kernel.queue_process_launch(None, None);
        let receiver_birth = launch_b.primary_thread_birth.expect("B receiver birth");
        kernel.set_current_process_id(launch_b.process_id);
        let hwnd_b = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                receiver_birth,
                "EXIT_SEND_RECEIVER_B",
                "",
                None,
                None,
                0,
                0,
                0,
                Rect::default(),
            )
            .expect("B receiver window");
        kernel.set_current_process_id(1);
        let send_id = kernel
            .begin_cross_thread_send_message_w_for_birth(
                sender_birth,
                hwnd_b,
                crate::ce::gwe::WM_USER + 0xc1,
                0,
                0,
                None,
            )
            .expect("A queued send to B");
        let wait_id = kernel
            .try_register_opaque_waiter_for_birth(
                sender_birth,
                sender_handle,
                Vec::new(),
                SchedulerBlockedWaitKind::SendMessage { send_id },
                0,
                crate::ce::timer::INFINITE,
            )
            .expect("A exact send waiter");
        assert!(!kernel.sent_message_result_ready(send_id));
        assert!(kernel.blocked_waiter(wait_id).is_some());
        let before = kernel.scheduler_stats();

        assert!(kernel.mark_process_launch_exited(&launch_b, 0));
        assert!(!kernel.gwe.is_window(hwnd_b));
        assert!(kernel.sent_message_result_ready(send_id));
        assert!(kernel.blocked_waiter(wait_id).is_some());
        let after = kernel.scheduler_stats();
        assert_eq!(
            after.send_reply_signal_count,
            before.send_reply_signal_count + 1,
            "receiver teardown must signal A's exact send ID once"
        );
        assert_eq!(
            after.send_reply_wake_candidate_count,
            before.send_reply_wake_candidate_count + 1,
            "A's registered SendReply waiter must be queued"
        );
        assert_eq!(kernel.take_completed_send_message_result(send_id), Some(0));
        Ok(())
    }

    #[test]
    fn pseudo_process_exit_uses_exact_window_birth_set() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let process_a = kernel.current_process_birth_key().expect("boot birth");
        let (thread_a_handle, _) = kernel.create_guest_thread(0x401000, 0, false);
        let thread_a = kernel
            .handles
            .thread_birth_key_for_handle(thread_a_handle)
            .expect("A thread birth");
        let launch_b = kernel.queue_process_launch(None, None);
        let thread_b = launch_b.primary_thread_birth.expect("B thread birth");
        kernel.set_current_process_id(launch_b.process_id);
        let hwnd_b = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                thread_b,
                "PSEUDO_EXIT_B",
                "",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::default(),
            )
            .expect("B window");
        kernel.set_current_process_id(process_a.process_id());
        let hwnd_a = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                thread_a,
                "PSEUDO_FOREIGN_A",
                "",
                None,
                Some(hwnd_b),
                0,
                WS_VISIBLE | crate::ce::gwe::WS_POPUP,
                0,
                Rect::default(),
            )
            .expect("A foreign-owned popup");
        kernel.set_current_process_id(launch_b.process_id);

        assert!(kernel.terminate_process(CE_CURRENT_PROCESS_PSEUDO_HANDLE, 7));
        assert!(!kernel.gwe.is_window(hwnd_b));
        assert!(kernel.gwe.is_window(hwnd_a));
        assert_eq!(kernel.current_process_exit_code, 7);
        Ok(())
    }

    #[test]
    fn reused_pid_host_exit_preserves_stranded_prior_birth_send() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (receiver_handle, receiver_tid) = kernel.create_guest_thread(0x401000, 0, false);
        let receiver_birth = kernel
            .handles
            .thread_birth_key_for_handle(receiver_handle)
            .expect("boot receiver birth");
        let hwnd = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                receiver_birth,
                "REUSED_PID_SEND_RECEIVER",
                "receiver",
                None,
                None,
                0,
                0,
                0,
                Rect::default(),
            )
            .expect("live boot receiver HWND");

        let launch_a = kernel.queue_process_launch(None, None);
        let process_a = launch_a.process_birth.expect("A process birth");
        let thread_a = launch_a.primary_thread_birth.expect("A thread birth");
        kernel.set_current_process_id(launch_a.process_id);
        let send_a = kernel
            .begin_cross_thread_send_message_w_for_birth(
                thread_a,
                hwnd,
                crate::ce::gwe::WM_USER + 0xb1,
                1,
                2,
                None,
            )
            .expect("A queued send");
        assert_eq!(
            kernel.gwe.sent_message(send_a).unwrap().sender_process_id(),
            Some(launch_a.process_id)
        );

        // Fixture-only: retire A's canonical records directly, bypassing
        // Kernel/GWES sender-exit cleanup so its inert send remains observable.
        assert!(kernel.handles.mark_thread_exited_by_birth(thread_a, 0));
        kernel.handles.close(launch_a.thread_handle)?;
        assert!(
            kernel
                .handles
                .finish_process_execution_by_birth(process_a, 0)
                .is_some()
        );
        kernel.handles.close(launch_a.process_handle)?;
        assert_eq!(kernel.handles.process_birth_key(launch_a.process_id), None);
        assert!(kernel.gwe.sent_message(send_a).is_some());

        kernel.next_process_id = launch_a.process_id;
        let launch_b = kernel.queue_process_launch(None, None);
        let process_b = launch_b.process_birth.expect("B process birth");
        assert_eq!(launch_b.process_id, launch_a.process_id);
        assert_ne!(process_a, process_b);
        assert_ne!(launch_a.thread_id, launch_b.thread_id);
        kernel.set_current_process_id(launch_b.process_id);
        let send_b = kernel
            .begin_cross_thread_send_message_w_for_birth(
                launch_b.primary_thread_birth.expect("B thread birth"),
                hwnd,
                crate::ce::gwe::WM_USER + 0xb2,
                3,
                4,
                None,
            )
            .expect("B queued send");
        assert_ne!(send_a, send_b);
        assert_eq!(kernel.gwe.sent_message_ids_for_windows(&[hwnd]).len(), 2);

        assert!(kernel.mark_process_launch_exited(&launch_b, 0));
        assert!(
            kernel.gwe.is_window(hwnd),
            "boot receiver must survive B exit"
        );
        assert!(kernel.gwe.sent_message(send_b).is_none(), "B send retires");
        assert!(
            kernel.gwe.sent_message(send_a).is_some(),
            "B exit must preserve A's stranded prior-birth send"
        );
        assert!(
            kernel
                .gwe
                .has_pending_sent_message_for_birth(receiver_birth)
        );
        assert_eq!(
            kernel.gwe.sent_message_ids_for_windows(&[hwnd]),
            vec![send_a]
        );
        assert_eq!(receiver_tid, receiver_birth.thread_id());
        Ok(())
    }

    #[test]
    fn reused_pid_host_exit_preserves_stranded_prior_birth_active_send() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (receiver_a_handle, receiver_a_tid) = kernel.create_guest_thread(0x401000, 0, false);
        let receiver_a_birth = kernel
            .handles
            .thread_birth_key_for_handle(receiver_a_handle)
            .expect("first boot receiver birth");
        let hwnd_a = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                receiver_a_birth,
                "REUSED_PID_ACTIVE_RECEIVER_A",
                "receiver A",
                None,
                None,
                0,
                0,
                0,
                Rect::default(),
            )
            .expect("first live boot receiver HWND");
        let (receiver_b_handle, receiver_b_tid) = kernel.create_guest_thread(0x402000, 0, false);
        let receiver_b_birth = kernel
            .handles
            .thread_birth_key_for_handle(receiver_b_handle)
            .expect("second boot receiver birth");
        let hwnd_b = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                receiver_b_birth,
                "REUSED_PID_ACTIVE_RECEIVER_B",
                "receiver B",
                None,
                None,
                0,
                0,
                0,
                Rect::default(),
            )
            .expect("second live boot receiver HWND");

        let launch_a = kernel.queue_process_launch(None, None);
        let process_a = launch_a.process_birth.expect("A process birth");
        let thread_a = launch_a.primary_thread_birth.expect("A thread birth");
        kernel.set_current_process_id(launch_a.process_id);
        let send_a = kernel
            .begin_cross_thread_send_message_w_for_birth(
                thread_a,
                hwnd_a,
                crate::ce::gwe::WM_USER + 0xb3,
                1,
                2,
                None,
            )
            .expect("A queued send");
        assert!(
            kernel
                .gwe
                .activate_sent_message_for_receiver_birth(receiver_a_birth, send_a)
        );
        assert_eq!(
            kernel.gwe.active_sent_message_id(receiver_a_tid),
            Some(send_a)
        );

        // Fixture-only retirement strands A's active callback without invoking
        // the normal sender-exit cleanup. A later B exit must not claim it.
        assert!(kernel.handles.mark_thread_exited_by_birth(thread_a, 0));
        kernel.handles.close(launch_a.thread_handle)?;
        assert!(
            kernel
                .handles
                .finish_process_execution_by_birth(process_a, 0)
                .is_some()
        );
        kernel.handles.close(launch_a.process_handle)?;
        assert_eq!(kernel.handles.process_birth_key(launch_a.process_id), None);

        kernel.next_process_id = launch_a.process_id;
        let launch_b = kernel.queue_process_launch(None, None);
        let process_b = launch_b.process_birth.expect("B process birth");
        assert_eq!(launch_b.process_id, launch_a.process_id);
        assert_ne!(process_a, process_b);
        assert_ne!(launch_a.thread_id, launch_b.thread_id);
        kernel.set_current_process_id(launch_b.process_id);
        let send_b = kernel
            .begin_cross_thread_send_message_w_for_birth(
                launch_b.primary_thread_birth.expect("B thread birth"),
                hwnd_b,
                crate::ce::gwe::WM_USER + 0xb4,
                3,
                4,
                Some(500),
            )
            .expect("B queued send");
        assert!(kernel.gwe.set_sent_message_result_ptr(send_b, 0xb2f0));
        assert!(
            kernel
                .gwe
                .activate_sent_message_for_receiver_birth(receiver_b_birth, send_b)
        );
        assert_eq!(
            kernel.gwe.active_sent_message_id(receiver_b_tid),
            Some(send_b)
        );
        assert_eq!(
            kernel.gwe.reply_message(receiver_b_tid, 0x1234),
            Some(send_b)
        );
        assert_eq!(
            kernel.gwe.completed_sent_message_result_writes(),
            vec![(send_b, 0xb2f0, 0x1234)]
        );

        assert!(kernel.mark_process_launch_exited(&launch_b, 0));
        assert!(kernel.gwe.is_window(hwnd_a));
        assert!(kernel.gwe.is_window(hwnd_b));
        let retained_a = kernel
            .gwe
            .sent_message(send_a)
            .expect("prior A active send");
        assert_eq!(retained_a.sender_thread_id(), Some(launch_a.thread_id));
        assert_eq!(retained_a.sender_process_id(), Some(launch_a.process_id));
        assert_eq!(retained_a.flags & crate::ce::gwe::SMF_SENDER_TERMINATED, 0);
        assert_eq!(
            kernel.gwe.active_sent_message_id(receiver_a_tid),
            Some(send_a)
        );

        let detached_b = kernel.gwe.sent_message(send_b).expect("B active send");
        assert_eq!(detached_b.sender_thread_id(), None);
        assert_eq!(detached_b.sender_process_id(), None);
        assert_ne!(detached_b.flags & crate::ce::gwe::SMF_SENDER_TERMINATED, 0);
        assert_eq!(detached_b.result_ptr, None);
        assert!(kernel.gwe.completed_sent_message_result_writes().is_empty());
        assert_eq!(
            kernel.gwe.active_sent_message_id(receiver_b_tid),
            Some(send_b)
        );
        assert_eq!(
            kernel.complete_active_sent_message(receiver_b_tid, 0x1234),
            Some(send_b)
        );
        assert!(kernel.gwe.sent_message(send_b).is_none());
        assert_eq!(
            kernel.gwe.active_sent_message_id(receiver_a_tid),
            Some(send_a)
        );
        Ok(())
    }

    #[test]
    fn reused_pid_pseudo_host_teardown_preserves_stale_prior_birth_window() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let process_handle_a = kernel.handles.create_process(77);
        let process_a = kernel
            .handles
            .process_birth_key(77)
            .expect("A process birth");
        kernel.set_current_process_id(77);
        let hwnd_a = kernel.create_window_ex_w(9, "STALE_PSEUDO_A", "A", None, 0, 0, 0);
        assert_eq!(
            kernel
                .gwe
                .window(hwnd_a)
                .unwrap()
                .delivery_owner()
                .process_birth(),
            Some(process_a)
        );

        // Keep only A's inert host HWND while the public PID becomes reusable.
        kernel.set_current_process_id(1);
        assert!(
            kernel
                .handles
                .finish_process_execution_by_birth(process_a, 0)
                .is_some()
        );
        assert!(kernel.close_handle(process_handle_a)?);
        let process_handle_b = kernel.handles.create_process(77);
        let process_b = kernel
            .handles
            .process_birth_key(77)
            .expect("B process birth");
        assert_ne!(process_a, process_b);
        kernel.set_current_process_id(77);
        let hwnd_b = kernel.create_window_ex_w(10, "PSEUDO_B", "B", None, 0, 0, 0);
        assert_eq!(
            kernel
                .gwe
                .window(hwnd_b)
                .unwrap()
                .delivery_owner()
                .process_birth(),
            Some(process_b)
        );

        assert!(kernel.terminate_process(CE_CURRENT_PROCESS_PSEUDO_HANDLE, 0));
        assert!(kernel.gwe.is_window(hwnd_a));
        assert!(!kernel.gwe.is_window(hwnd_b));
        assert!(kernel.close_handle(process_handle_b)?);
        Ok(())
    }

    #[test]
    fn remote_input_active_window_drain_posts_mouse_messages() -> Result<()> {
        let config = RuntimeConfig::load_default()?;
        let mut kernel = CeKernel::boot(config);
        let thread_id = 1;
        let hwnd = kernel.create_window_ex_w_with_rect(
            thread_id,
            "ACTIVE",
            "",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, 800, 480),
        );
        assert_eq!(kernel.gwe.set_active_window(Some(hwnd)), None);
        kernel.remote.set_framebuffer_size(800, 480);
        kernel.remote.enqueue_touch("tap", 10, 20).unwrap();

        assert_eq!(kernel.drain_remote_input_to_active_window(), 2);

        let down = kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(hwnd),
                WM_LBUTTONDOWN,
                WM_LBUTTONUP,
                PeekFlags::REMOVE,
            )
            .expect("remote tap posts mouse down");
        assert_eq!(down.hwnd, hwnd);
        assert_eq!(down.msg, WM_LBUTTONDOWN);
        let up = kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(hwnd),
                WM_LBUTTONDOWN,
                WM_LBUTTONUP,
                PeekFlags::REMOVE,
            )
            .expect("remote tap posts mouse up");
        assert_eq!(up.hwnd, hwnd);
        assert_eq!(up.msg, WM_LBUTTONUP);
        assert!(up.time_ms > down.time_ms);
        Ok(())
    }

    fn stale_window_remote_input_fixture() -> Result<(CeKernel, u32, ThreadBirthKey, u32, u32)> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (handle_a, thread_id) = kernel.create_guest_thread(0x401000, 0, false);
        let birth_a = kernel
            .handles
            .thread_birth_key_for_handle(handle_a)
            .expect("A thread birth");
        let hwnd_a = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth_a,
                "REMOTE_BIRTH_A",
                "A",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(0, 0, 800, 480),
            )
            .expect("A checked window");
        assert!(kernel.mark_guest_thread_exited_by_birth(birth_a, 0));
        assert!(kernel.close_handle(handle_a)?);

        let launch_b = kernel.queue_process_launch(None, None);
        let process_b = kernel
            .handles
            .process_birth_key(launch_b.process_id)
            .expect("B process birth");
        let handle_b = kernel
            .handles
            .try_create_thread_in_birth(process_b, thread_id, 0x402000, 0, false)?;
        let birth_b = kernel
            .handles
            .thread_birth_key_for_handle(handle_b)
            .expect("B thread birth");
        assert_ne!(birth_a, birth_b);
        kernel.set_current_process_id(launch_b.process_id);
        let hwnd_b = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth_b,
                "REMOTE_BIRTH_B",
                "B",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(0, 0, 800, 480),
            )
            .expect("B checked window");
        assert!(kernel.gwe.is_window(hwnd_a));
        assert_eq!(kernel.recipient_birth_for_window(hwnd_a), None);
        assert!(kernel.gwe.is_window(hwnd_b));
        assert_eq!(kernel.recipient_birth_for_window(hwnd_b), Some(process_b));
        kernel.gwe.set_focus(Some(hwnd_b));
        kernel.gwe.set_active_window(Some(hwnd_b));
        kernel.gwe.set_keyboard_target(thread_id, Some(hwnd_b));
        kernel.remote.set_framebuffer_size(800, 480);
        Ok((kernel, handle_b, birth_b, hwnd_a, hwnd_b))
    }

    #[test]
    fn stale_birth_window_remote_touch_does_not_reach_reused_tid() -> Result<()> {
        use crate::ce::gwe::{QS_MOUSEBUTTON, QS_POSTMESSAGE};

        let (mut kernel, handle_b, birth_b, hwnd_a, hwnd_b) = stale_window_remote_input_fixture()?;
        let thread_id = birth_b.thread_id();
        let wait_id = kernel.register_blocked_waiter(
            thread_id,
            handle_b,
            vec![],
            SchedulerBlockedWaitKind::GetMessage {
                hwnd: None,
                min_msg: 0,
                max_msg: 0,
            },
            kernel.timers.tick_count(),
            crate::ce::timer::INFINITE,
        );
        assert_ne!(wait_id, 0);
        let status_flags = QS_MOUSEBUTTON | QS_POSTMESSAGE;
        let status_before = kernel
            .gwe
            .peek_queue_status_for_birth(birth_b, status_flags);
        let ready_before = kernel.get_message_queue_ready_time_stamp_for_thread(thread_id, 0);
        let wakes_before = kernel.scheduler_stats().message_input_wake_candidate_count;
        let signals_before = kernel.scheduler_stats().message_input_signal_count;
        let focus_before = kernel.gwe.get_focus();
        let active_before = kernel.gwe.get_active_window();
        let keyboard_before = kernel.gwe.get_keyboard_target(thread_id);
        let mouse_pos_before = kernel.gwe.get_message_pos(thread_id);
        let trace_before = kernel.recent_message_ops().len();

        kernel.remote.enqueue_touch("down", 13, 27).unwrap();
        let posted = kernel.drain_remote_input_to_gwe(thread_id, hwnd_a);
        let target_or_post_traces: Vec<_> = kernel.recent_message_ops()[trace_before..]
            .iter()
            .filter(|trace| {
                trace.op == "remote_touch_target"
                    || (trace.op == "post_message"
                        && matches!(trace.msg, Some(WM_LBUTTONDOWN | WM_LBUTTONUP)))
            })
            .cloned()
            .collect();

        assert_eq!(
            posted, 0,
            "stale A input must be dropped, not rerouted to B"
        );
        assert_eq!(kernel.gwe.get_focus(), focus_before);
        assert_eq!(kernel.gwe.get_active_window(), active_before);
        assert_eq!(kernel.gwe.get_keyboard_target(thread_id), keyboard_before);
        assert_eq!(kernel.gwe.get_message_pos(thread_id), mouse_pos_before);
        assert_eq!(
            kernel
                .gwe
                .peek_queue_status_for_birth(birth_b, status_flags),
            status_before
        );
        assert_eq!(
            kernel.get_message_queue_ready_time_stamp_for_thread(thread_id, 0),
            ready_before
        );
        assert_eq!(
            kernel.scheduler_stats().message_input_wake_candidate_count,
            wakes_before
        );
        assert_eq!(
            kernel.scheduler_stats().message_input_signal_count,
            signals_before
        );
        assert!(
            target_or_post_traces.is_empty(),
            "{target_or_post_traces:?}"
        );
        assert!(!kernel.gwe.has_message_filtered_for_birth(
            birth_b,
            Some(hwnd_a),
            WM_LBUTTONDOWN,
            WM_LBUTTONUP,
        ));
        assert!(!kernel.gwe.has_message_filtered_for_birth(
            birth_b,
            Some(hwnd_b),
            WM_LBUTTONDOWN,
            WM_LBUTTONUP,
        ));
        assert!(kernel.remove_blocked_waiter(wait_id).is_some());
        Ok(())
    }

    #[test]
    fn stale_birth_window_remote_key_does_not_reach_reused_tid() -> Result<()> {
        use crate::ce::gwe::{QS_KEY, QS_POSTMESSAGE, WM_KEYDOWN};

        let (mut kernel, handle_b, birth_b, hwnd_a, hwnd_b) = stale_window_remote_input_fixture()?;
        let thread_id = birth_b.thread_id();
        let wait_id = kernel.register_blocked_waiter(
            thread_id,
            handle_b,
            vec![],
            SchedulerBlockedWaitKind::GetMessage {
                hwnd: None,
                min_msg: 0,
                max_msg: 0,
            },
            kernel.timers.tick_count(),
            crate::ce::timer::INFINITE,
        );
        assert_ne!(wait_id, 0);
        let status_flags = QS_KEY | QS_POSTMESSAGE;
        let status_before = kernel
            .gwe
            .peek_queue_status_for_birth(birth_b, status_flags);
        let ready_before = kernel.get_message_queue_ready_time_stamp_for_thread(thread_id, 0);
        let wakes_before = kernel.scheduler_stats().message_input_wake_candidate_count;
        let signals_before = kernel.scheduler_stats().message_input_signal_count;
        let focus_before = kernel.gwe.get_focus();
        let active_before = kernel.gwe.get_active_window();
        let keyboard_before = kernel.gwe.get_keyboard_target(thread_id);
        let mouse_pos_before = kernel.gwe.get_message_pos(thread_id);
        let trace_before = kernel.recent_message_ops().len();

        kernel.remote.enqueue_key("down", 0x0d).unwrap();
        let posted = kernel.drain_remote_input_to_gwe(thread_id, hwnd_a);
        let target_or_post_traces: Vec<_> = kernel.recent_message_ops()[trace_before..]
            .iter()
            .filter(|trace| {
                trace.op == "remote_key_target"
                    || (trace.op == "post_message" && trace.msg == Some(WM_KEYDOWN))
            })
            .cloned()
            .collect();

        assert_eq!(posted, 0, "stale A key must be dropped, not rerouted to B");
        assert_eq!(kernel.gwe.get_focus(), focus_before);
        assert_eq!(kernel.gwe.get_active_window(), active_before);
        assert_eq!(kernel.gwe.get_keyboard_target(thread_id), keyboard_before);
        assert_eq!(kernel.gwe.get_message_pos(thread_id), mouse_pos_before);
        assert_eq!(
            kernel
                .gwe
                .peek_queue_status_for_birth(birth_b, status_flags),
            status_before
        );
        assert_eq!(
            kernel.get_message_queue_ready_time_stamp_for_thread(thread_id, 0),
            ready_before
        );
        assert_eq!(
            kernel.scheduler_stats().message_input_wake_candidate_count,
            wakes_before
        );
        assert_eq!(
            kernel.scheduler_stats().message_input_signal_count,
            signals_before
        );
        assert!(
            target_or_post_traces.is_empty(),
            "{target_or_post_traces:?}"
        );
        assert!(!kernel.gwe.has_message_filtered_for_birth(
            birth_b,
            Some(hwnd_a),
            WM_KEYDOWN,
            WM_KEYDOWN,
        ));
        assert!(!kernel.gwe.has_message_filtered_for_birth(
            birth_b,
            Some(hwnd_b),
            WM_KEYDOWN,
            WM_KEYDOWN,
        ));
        assert!(kernel.remove_blocked_waiter(wait_id).is_some());
        Ok(())
    }

    // Superseded a03–a05: a host wait HWND is only a fallback hint; it does
    // not own a coordinate touch or suppress a live capture/hit-test target.
    // Corrected cases are a15, a16, and a18; original red logs remain saved.

    #[test]
    fn live_capture_touch_uses_captured_window_birth_not_requested_thread() -> Result<()> {
        use crate::ce::gwe::{MSGSRC_SOFTWARE_POST, QS_MOUSEBUTTON};

        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (handle_a, thread_a) = kernel.create_guest_thread(0x401000, 0, false);
        let (handle_b, thread_b) = kernel.create_guest_thread(0x402000, 0, false);
        let birth_a = kernel
            .handles
            .thread_birth_key_for_handle(handle_a)
            .expect("A thread birth");
        let birth_b = kernel
            .handles
            .thread_birth_key_for_handle(handle_b)
            .expect("B thread birth");
        assert_ne!(thread_a, thread_b);
        let hwnd_a = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth_a,
                "CAPTURE_A",
                "A",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(0, 0, 800, 480),
            )
            .expect("A checked window");
        let hwnd_b = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth_b,
                "REQUEST_B",
                "B",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(0, 0, 800, 480),
            )
            .expect("B checked window");
        kernel.gwe.set_focus(None);
        kernel.gwe.set_active_window(None);
        assert_eq!(kernel.set_capture(hwnd_a), None);
        assert_eq!(kernel.gwe.get_capture(), Some(hwnd_a));
        let wait_a = kernel.register_blocked_waiter(
            thread_a,
            handle_a,
            vec![],
            SchedulerBlockedWaitKind::GetMessage {
                hwnd: None,
                min_msg: 0,
                max_msg: 0,
            },
            kernel.timers.tick_count(),
            crate::ce::timer::INFINITE,
        );
        let wait_b = kernel.register_blocked_waiter(
            thread_b,
            handle_b,
            vec![],
            SchedulerBlockedWaitKind::GetMessage {
                hwnd: None,
                min_msg: 0,
                max_msg: 0,
            },
            kernel.timers.tick_count(),
            crate::ce::timer::INFINITE,
        );
        assert_ne!(wait_a, 0);
        assert_ne!(wait_b, 0);
        let status_b_before = kernel
            .gwe
            .peek_queue_status_for_birth(birth_b, QS_MOUSEBUTTON);
        let ready_b_before = kernel.get_message_queue_ready_time_stamp_for_thread(thread_b, 0);
        let wakes_before = kernel.scheduler_stats().message_input_wake_candidate_count;
        let trace_before = kernel.recent_message_ops().len();
        kernel.remote.set_framebuffer_size(800, 480);
        kernel.timers.sleep_ms(17);
        kernel.remote.enqueue_touch("down", 77, 88).unwrap();

        let drain = kernel.drain_remote_input_to_thread_window_detailed(thread_b, Some(hwnd_b));

        assert_eq!(drain.posted, 1);
        assert_eq!(drain.target_thread_ids, vec![thread_a]);
        let mouse = kernel
            .gwe
            .peek_message_filtered_for_birth(
                birth_a,
                Some(hwnd_a),
                WM_LBUTTONDOWN,
                WM_LBUTTONDOWN,
                PeekFlags::empty(),
            )
            .expect("captured A receives the mouse message");
        assert_eq!(mouse.hwnd, hwnd_a);
        assert_eq!(mouse.wparam, 1);
        assert_eq!(mouse.lparam, make_lparam(77, 88));
        assert_eq!(mouse.mouse_pos_at_post, Some(make_lparam(77, 88)));
        assert_eq!(mouse.source, MSGSRC_SOFTWARE_POST);
        assert_ne!(
            kernel
                .gwe
                .peek_queue_status_for_birth(birth_a, QS_MOUSEBUTTON),
            0
        );
        assert_eq!(
            kernel
                .gwe
                .peek_queue_status_for_birth(birth_b, QS_MOUSEBUTTON),
            status_b_before
        );
        assert_eq!(
            kernel.get_message_queue_ready_time_stamp_for_thread(thread_b, 0),
            ready_b_before
        );
        assert!(kernel.scheduler_stats().message_input_wake_candidate_count > wakes_before);
        assert!(!kernel.gwe.has_message_filtered_for_birth(
            birth_b,
            Some(hwnd_a),
            WM_LBUTTONDOWN,
            WM_LBUTTONDOWN,
        ));
        assert!(!kernel.gwe.has_message_filtered_for_birth(
            birth_b,
            Some(hwnd_b),
            WM_LBUTTONDOWN,
            WM_LBUTTONDOWN,
        ));
        let target_traces: Vec<_> = kernel.recent_message_ops()[trace_before..]
            .iter()
            .filter(|trace| trace.op == "remote_touch_target")
            .collect();
        assert_eq!(target_traces.len(), 1);
        assert_eq!(target_traces[0].thread_id, thread_a);
        assert_eq!(target_traces[0].hwnd, Some(hwnd_a));
        assert!(kernel.remove_blocked_waiter(wait_a).is_some());
        assert!(kernel.remove_blocked_waiter(wait_b).is_some());
        Ok(())
    }

    #[test]
    fn destroyed_explicit_remote_target_consumes_input_without_replay() -> Result<()> {
        use crate::ce::gwe::{QS_KEY, QS_MOUSEBUTTON, WM_KEYDOWN};

        let (mut kernel, _handle_b, birth_b, hwnd_a, hwnd_b) = stale_window_remote_input_fixture()?;
        let thread_id = birth_b.thread_id();
        assert!(kernel.destroy_window(hwnd_a));
        assert!(!kernel.gwe.is_window(hwnd_a));
        let status_before = kernel
            .gwe
            .peek_queue_status_for_birth(birth_b, QS_MOUSEBUTTON | QS_KEY);
        let ready_before = kernel.get_message_queue_ready_time_stamp_for_thread(thread_id, 0);
        let wakes_before = kernel.scheduler_stats().message_input_wake_candidate_count;
        let signals_before = kernel.scheduler_stats().message_input_signal_count;
        let trace_before = kernel.recent_message_ops().len();
        kernel.remote.enqueue_touch("down", 19, 23).unwrap();
        kernel.remote.enqueue_key("down", 0x0d).unwrap();

        // This explicit host target was destroyed. Its queued input must be
        // consumed there, not retained for a later unrelated B drain.
        let stale_posted = kernel.drain_remote_input_to_gwe(thread_id, hwnd_a);
        let remaining_after_stale = (
            kernel.remote.touch_event_count(),
            kernel.remote.key_event_count(),
        );
        let replayed_to_b = kernel.drain_remote_input_to_gwe(thread_id, hwnd_b);

        assert_eq!(stale_posted, 0);
        assert_eq!(
            replayed_to_b, 0,
            "destroyed explicit target retained {remaining_after_stale:?} input and replayed to B"
        );
        assert_eq!(remaining_after_stale, (0, 0));
        assert_eq!(kernel.remote.touch_event_count(), 0);
        assert_eq!(kernel.remote.key_event_count(), 0);
        assert_eq!(
            kernel
                .gwe
                .peek_queue_status_for_birth(birth_b, QS_MOUSEBUTTON | QS_KEY),
            status_before
        );
        assert_eq!(
            kernel.get_message_queue_ready_time_stamp_for_thread(thread_id, 0),
            ready_before
        );
        assert_eq!(
            kernel.scheduler_stats().message_input_wake_candidate_count,
            wakes_before
        );
        assert_eq!(
            kernel.scheduler_stats().message_input_signal_count,
            signals_before
        );
        assert!(!kernel.gwe.has_message_filtered_for_birth(
            birth_b,
            Some(hwnd_b),
            WM_LBUTTONDOWN,
            WM_LBUTTONDOWN,
        ));
        assert!(!kernel.gwe.has_message_filtered_for_birth(
            birth_b,
            Some(hwnd_b),
            WM_KEYDOWN,
            WM_KEYDOWN,
        ));
        assert!(
            !kernel.recent_message_ops()[trace_before..]
                .iter()
                .any(|trace| {
                    matches!(trace.op, "remote_touch_target" | "remote_key_target")
                        && trace.hwnd == Some(hwnd_b)
                })
        );
        Ok(())
    }

    // Superseded a08/a09: a GetMessage HWND is a read filter, not a key
    // destination, and stale mouse capture does not consume a fresh B DOWN.
    // Corrected cases are a19–a22; original red logs remain saved.

    #[test]
    fn coordinate_only_remote_tap_reaches_live_b_after_a_exit() -> Result<()> {
        use crate::ce::gwe::MSGSRC_SOFTWARE_POST;

        let (mut kernel, _handle_b, birth_b, hwnd_a, hwnd_b) = stale_window_remote_input_fixture()?;
        let thread_id = birth_b.thread_id();
        let point = Point { x: 211, y: 223 };
        // This is a coordinate-only host event. No stale wait/filter HWND or
        // capture is supplied; the live visible B window is the hit-test hit.
        assert!(kernel.gwe.show_window(hwnd_a, false));
        assert_eq!(kernel.gwe.get_capture(), None);
        assert_eq!(kernel.gwe.get_active_window(), Some(hwnd_b));
        assert_eq!(kernel.gwe.window_from_point(point), Some(hwnd_b));
        kernel
            .remote
            .enqueue_touch("tap", point.x, point.y)
            .unwrap();

        let drain = kernel.drain_remote_input_to_active_window_detailed();

        assert_eq!(drain.posted, 2);
        assert_eq!(drain.target_thread_ids, vec![thread_id]);
        assert_eq!(kernel.remote.touch_event_count(), 0);
        let down = kernel
            .gwe
            .peek_message_filtered_for_birth(
                birth_b,
                Some(hwnd_b),
                WM_LBUTTONDOWN,
                WM_LBUTTONDOWN,
                PeekFlags::REMOVE,
            )
            .expect("coordinate-only down reaches live B");
        let up = kernel
            .gwe
            .peek_message_filtered_for_birth(
                birth_b,
                Some(hwnd_b),
                WM_LBUTTONUP,
                WM_LBUTTONUP,
                PeekFlags::REMOVE,
            )
            .expect("coordinate-only up reaches live B");
        assert_eq!(down.wparam, 1);
        assert_eq!(up.wparam, 0);
        assert_eq!(down.lparam, make_lparam(point.x, point.y));
        assert_eq!(up.lparam, make_lparam(point.x, point.y));
        assert_eq!(down.mouse_pos_at_post, Some(make_lparam(point.x, point.y)));
        assert_eq!(up.mouse_pos_at_post, Some(make_lparam(point.x, point.y)));
        assert_eq!(down.source, MSGSRC_SOFTWARE_POST);
        assert_eq!(up.source, MSGSRC_SOFTWARE_POST);
        assert!(up.time_ms > down.time_ms);
        Ok(())
    }

    #[test]
    fn stale_guest_getmessage_filter_preserves_live_b_hit_test_post() -> Result<()> {
        use crate::ce::gwe::QS_MOUSEBUTTON;

        let (mut kernel, handle_b, birth_b, hwnd_a, hwnd_b) = stale_window_remote_input_fixture()?;
        let thread_id = birth_b.thread_id();
        let point = Point { x: 211, y: 223 };
        assert!(kernel.gwe.show_window(hwnd_a, false));
        assert!(kernel.gwe.is_window(hwnd_a));
        assert_eq!(kernel.gwe.get_capture(), None);
        assert_eq!(kernel.gwe.window_from_point(point), Some(hwnd_b));
        let wait_id = kernel.register_blocked_waiter(
            thread_id,
            handle_b,
            vec![],
            SchedulerBlockedWaitKind::GetMessage {
                hwnd: Some(hwnd_a),
                min_msg: WM_LBUTTONDOWN,
                max_msg: WM_LBUTTONDOWN,
            },
            kernel.timers.tick_count(),
            crate::ce::timer::INFINITE,
        );
        assert_ne!(wait_id, 0);
        kernel.timers.sleep_ms(17);
        let wakes_before = kernel.scheduler_stats().message_input_wake_candidate_count;
        kernel
            .remote
            .enqueue_touch("down", point.x, point.y)
            .unwrap();

        let stale_filtered =
            kernel.get_message_w_filtered(thread_id, Some(hwnd_a), WM_LBUTTONDOWN, WM_LBUTTONDOWN);

        assert!(
            stale_filtered.is_none(),
            "A's filter cannot consume B's post"
        );
        assert_eq!(kernel.remote.touch_event_count(), 0);
        assert!(kernel.gwe.has_message_filtered_for_birth(
            birth_b,
            Some(hwnd_b),
            WM_LBUTTONDOWN,
            WM_LBUTTONDOWN,
        ));
        assert_ne!(
            kernel
                .gwe
                .peek_queue_status_for_birth(birth_b, QS_MOUSEBUTTON),
            0
        );
        assert_ne!(
            kernel.get_message_queue_ready_time_stamp_for_thread(thread_id, 0),
            0
        );
        assert!(kernel.scheduler_stats().message_input_wake_candidate_count > wakes_before);
        let b_message = kernel
            .get_message_w_filtered(thread_id, Some(hwnd_b), WM_LBUTTONDOWN, WM_LBUTTONDOWN)
            .expect("B's own filter consumes its queued touch");
        assert_eq!(b_message.hwnd, hwnd_b);
        assert_eq!(
            b_message.mouse_pos_at_post,
            Some(make_lparam(point.x, point.y))
        );
        assert!(kernel.remove_blocked_waiter(wait_id).is_some());
        Ok(())
    }

    #[test]
    fn destroyed_host_hint_falls_through_to_live_b_hit_test() -> Result<()> {
        let (mut kernel, _handle_b, birth_b, hwnd_a, hwnd_b) = stale_window_remote_input_fixture()?;
        let thread_id = birth_b.thread_id();
        let point = Point { x: 211, y: 223 };
        assert!(kernel.destroy_window(hwnd_a));
        assert!(!kernel.gwe.is_window(hwnd_a));
        assert_eq!(kernel.gwe.get_capture(), None);
        assert_eq!(kernel.gwe.window_from_point(point), Some(hwnd_b));
        kernel
            .remote
            .enqueue_touch("down", point.x, point.y)
            .unwrap();

        let drain = kernel.drain_remote_input_to_thread_window_detailed(thread_id, Some(hwnd_a));

        assert_eq!(drain.posted, 1);
        assert_eq!(drain.target_thread_ids, vec![thread_id]);
        assert_eq!(kernel.remote.touch_event_count(), 0);
        assert!(kernel.gwe.has_message_filtered_for_birth(
            birth_b,
            Some(hwnd_b),
            WM_LBUTTONDOWN,
            WM_LBUTTONDOWN,
        ));
        assert!(!kernel.gwe.has_message_filtered_for_birth(
            birth_b,
            Some(hwnd_a),
            WM_LBUTTONDOWN,
            WM_LBUTTONDOWN,
        ));
        Ok(())
    }

    #[test]
    fn live_x_getmessage_filter_does_not_steal_y_touch() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (handle_x, thread_x) = kernel.create_guest_thread(0x401000, 0, false);
        let (handle_y, thread_y) = kernel.create_guest_thread(0x402000, 0, false);
        let birth_x = kernel
            .handles
            .thread_birth_key_for_handle(handle_x)
            .expect("X thread birth");
        let birth_y = kernel
            .handles
            .thread_birth_key_for_handle(handle_y)
            .expect("Y thread birth");
        let hwnd_x = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth_x,
                "FILTER_X",
                "X",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(0, 0, 100, 100),
            )
            .expect("X checked window");
        let hwnd_y = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth_y,
                "TOUCH_Y",
                "Y",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(200, 200, 100, 100),
            )
            .expect("Y checked window");
        let point = Point { x: 250, y: 250 };
        assert_eq!(kernel.gwe.get_capture(), None);
        assert_eq!(kernel.gwe.window_from_point(point), Some(hwnd_y));
        assert_eq!(
            kernel.gwe.window_from_point_for_thread(thread_x, point),
            None
        );
        kernel.remote.set_framebuffer_size(800, 480);
        kernel
            .remote
            .enqueue_touch("down", point.x, point.y)
            .unwrap();

        let x_filtered =
            kernel.get_message_w_filtered(thread_x, Some(hwnd_x), WM_LBUTTONDOWN, WM_LBUTTONDOWN);

        assert!(x_filtered.is_none(), "X's filter cannot consume Y's touch");
        assert_eq!(kernel.remote.touch_event_count(), 0);
        assert!(kernel.gwe.has_message_filtered_for_birth(
            birth_y,
            Some(hwnd_y),
            WM_LBUTTONDOWN,
            WM_LBUTTONDOWN,
        ));
        assert!(!kernel.gwe.has_message_filtered_for_birth(
            birth_x,
            Some(hwnd_x),
            WM_LBUTTONDOWN,
            WM_LBUTTONDOWN,
        ));
        let y_message = kernel
            .get_message_w_filtered(thread_y, Some(hwnd_y), WM_LBUTTONDOWN, WM_LBUTTONDOWN)
            .expect("Y's reader consumes the touch");
        assert_eq!(y_message.hwnd, hwnd_y);
        assert_eq!(
            y_message.mouse_pos_at_post,
            Some(make_lparam(point.x, point.y))
        );
        Ok(())
    }

    #[test]
    fn stale_hint_with_live_b_capture_routes_touch_to_b() -> Result<()> {
        let (mut kernel, _handle_b, birth_b, hwnd_a, hwnd_b) = stale_window_remote_input_fixture()?;
        let thread_id = birth_b.thread_id();
        assert_eq!(kernel.set_capture(hwnd_b), None);
        assert_eq!(kernel.gwe.get_capture(), Some(hwnd_b));
        kernel.remote.enqueue_touch("down", 31, 37).unwrap();

        let drain = kernel.drain_remote_input_to_thread_window_detailed(thread_id, Some(hwnd_a));

        assert_eq!(drain.posted, 1);
        assert_eq!(drain.target_thread_ids, vec![thread_id]);
        assert!(kernel.gwe.has_message_filtered_for_birth(
            birth_b,
            Some(hwnd_b),
            WM_LBUTTONDOWN,
            WM_LBUTTONDOWN,
        ));
        Ok(())
    }

    #[test]
    fn destroyed_filter_key_follows_live_b_focus() -> Result<()> {
        use crate::ce::gwe::WM_KEYDOWN;

        let (mut kernel, _handle_b, birth_b, hwnd_a, hwnd_b) = stale_window_remote_input_fixture()?;
        let thread_id = birth_b.thread_id();
        assert!(kernel.destroy_window(hwnd_a));
        assert_eq!(kernel.gwe.get_focus(), Some(hwnd_b));
        assert_eq!(kernel.gwe.get_active_window(), Some(hwnd_b));
        kernel.remote.enqueue_key("down", 0x0d).unwrap();

        let stale_filtered =
            kernel.get_message_w_filtered(thread_id, Some(hwnd_a), WM_KEYDOWN, WM_KEYDOWN);

        assert!(stale_filtered.is_none());
        assert_eq!(kernel.remote.key_event_count(), 0);
        assert!(kernel.gwe.has_message_filtered_for_birth(
            birth_b,
            Some(hwnd_b),
            WM_KEYDOWN,
            WM_KEYDOWN,
        ));
        let b_key = kernel
            .get_message_w_filtered(thread_id, Some(hwnd_b), WM_KEYDOWN, WM_KEYDOWN)
            .expect("B focus receives the key");
        assert_eq!(b_key.hwnd, hwnd_b);
        assert_eq!(b_key.wparam, 0x0d);
        Ok(())
    }

    fn captured_a_reused_tid_fixture(
        down_before_a_exit: bool,
    ) -> Result<(CeKernel, ThreadBirthKey, u32, u32)> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (handle_a, thread_id) = kernel.create_guest_thread(0x401000, 0, false);
        let birth_a = kernel
            .handles
            .thread_birth_key_for_handle(handle_a)
            .expect("A thread birth");
        let hwnd_a = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth_a,
                "CAPTURE_BIRTH_A",
                "A",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(0, 0, 800, 480),
            )
            .expect("A checked window");
        kernel.remote.set_framebuffer_size(800, 480);
        assert_eq!(kernel.set_capture(hwnd_a), None);
        if down_before_a_exit {
            kernel.remote.enqueue_touch("down", 51, 61).unwrap();
            let down = kernel.drain_remote_input_to_active_window_detailed();
            assert_eq!(down.posted, 1);
            assert_eq!(down.target_thread_ids, vec![thread_id]);
            assert!(kernel.gwe.has_message_filtered_for_birth(
                birth_a,
                Some(hwnd_a),
                WM_LBUTTONDOWN,
                WM_LBUTTONDOWN,
            ));
        }
        assert!(kernel.mark_guest_thread_exited_by_birth(birth_a, 0));
        assert!(kernel.close_handle(handle_a)?);
        let launch_b = kernel.queue_process_launch(None, None);
        let process_b = kernel
            .handles
            .process_birth_key(launch_b.process_id)
            .expect("B process birth");
        let handle_b = kernel
            .handles
            .try_create_thread_in_birth(process_b, thread_id, 0x402000, 0, false)?;
        let birth_b = kernel
            .handles
            .thread_birth_key_for_handle(handle_b)
            .expect("B thread birth");
        assert_ne!(birth_a, birth_b);
        kernel.set_current_process_id(launch_b.process_id);
        let hwnd_b = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth_b,
                "CAPTURE_BIRTH_B",
                "B",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(0, 0, 800, 480),
            )
            .expect("B checked window");
        kernel.gwe.set_focus(Some(hwnd_b));
        kernel.gwe.set_active_window(Some(hwnd_b));
        assert_eq!(kernel.gwe.get_capture(), Some(hwnd_a));
        assert_eq!(kernel.recipient_birth_for_window(hwnd_a), None);
        assert_eq!(kernel.recipient_birth_for_window(hwnd_b), Some(process_b));
        Ok((kernel, birth_b, hwnd_a, hwnd_b))
    }

    #[test]
    fn stale_capture_new_down_move_up_retargets_live_b() -> Result<()> {
        use crate::ce::gwe::WM_MOUSEMOVE;

        let (mut kernel, birth_b, hwnd_a, hwnd_b) = captured_a_reused_tid_fixture(false)?;
        let thread_id = birth_b.thread_id();
        let point = Point { x: 71, y: 83 };
        assert_eq!(kernel.gwe.window_from_point(point), Some(hwnd_b));
        assert_eq!(kernel.gwe.get_capture(), Some(hwnd_a));
        kernel
            .remote
            .enqueue_touch("down", point.x, point.y)
            .unwrap();
        kernel
            .remote
            .enqueue_touch("move", point.x, point.y)
            .unwrap();
        kernel.remote.enqueue_touch("up", point.x, point.y).unwrap();

        let drain = kernel.drain_remote_input_to_active_window_detailed();

        assert_eq!(drain.posted, 3);
        assert_eq!(drain.target_thread_ids, vec![thread_id]);
        assert_eq!(
            kernel.gwe.get_capture(),
            None,
            "fresh DOWN releases stale A capture"
        );
        assert_eq!(kernel.remote.touch_event_count(), 0);
        for message in [WM_LBUTTONDOWN, WM_MOUSEMOVE, WM_LBUTTONUP] {
            let posted = kernel
                .gwe
                .peek_message_filtered_for_birth(
                    birth_b,
                    Some(hwnd_b),
                    message,
                    message,
                    PeekFlags::REMOVE,
                )
                .expect("B receives each phase of the new gesture");
            assert_eq!(posted.hwnd, hwnd_b);
            assert_eq!(posted.lparam, make_lparam(point.x, point.y));
            assert_eq!(
                posted.mouse_pos_at_post,
                Some(make_lparam(point.x, point.y))
            );
        }
        Ok(())
    }

    #[test]
    fn stale_capture_orphan_up_drops_after_a_exit() -> Result<()> {
        let (mut kernel, birth_b, hwnd_a, hwnd_b) = captured_a_reused_tid_fixture(true)?;
        let thread_id = birth_b.thread_id();
        let status_before = kernel
            .gwe
            .peek_queue_status_for_birth(birth_b, crate::ce::gwe::QS_MOUSEBUTTON);
        let ready_before = kernel.get_message_queue_ready_time_stamp_for_thread(thread_id, 0);
        let wakes_before = kernel.scheduler_stats().message_input_wake_candidate_count;
        assert_eq!(kernel.gwe.get_capture(), Some(hwnd_a));
        kernel.remote.enqueue_touch("up", 51, 61).unwrap();

        let drain = kernel.drain_remote_input_to_active_window_detailed();

        assert_eq!(drain.posted, 0);
        assert!(drain.target_thread_ids.is_empty());
        assert_eq!(kernel.remote.touch_event_count(), 0);
        assert_eq!(
            kernel
                .gwe
                .peek_queue_status_for_birth(birth_b, crate::ce::gwe::QS_MOUSEBUTTON),
            status_before
        );
        assert_eq!(
            kernel.get_message_queue_ready_time_stamp_for_thread(thread_id, 0),
            ready_before
        );
        assert_eq!(
            kernel.scheduler_stats().message_input_wake_candidate_count,
            wakes_before
        );
        assert!(!kernel.gwe.has_message_filtered_for_birth(
            birth_b,
            Some(hwnd_b),
            WM_LBUTTONUP,
            WM_LBUTTONUP,
        ));
        Ok(())
    }

    #[test]
    fn stale_capture_key_uses_live_b_focus() -> Result<()> {
        use crate::ce::gwe::WM_KEYDOWN;

        let (mut kernel, birth_b, hwnd_a, hwnd_b) = captured_a_reused_tid_fixture(false)?;
        assert_eq!(kernel.gwe.get_capture(), Some(hwnd_a));
        assert_eq!(kernel.gwe.get_focus(), Some(hwnd_b));
        kernel.remote.enqueue_key("down", 0x0d).unwrap();

        let drain = kernel.drain_remote_input_to_active_window_detailed();

        assert_eq!(drain.posted, 1);
        assert_eq!(drain.target_thread_ids, vec![birth_b.thread_id()]);
        assert_eq!(kernel.remote.key_event_count(), 0);
        assert_eq!(
            kernel.gwe.get_capture(),
            Some(hwnd_a),
            "key does not change mouse capture"
        );
        let key = kernel
            .gwe
            .peek_message_filtered_for_birth(
                birth_b,
                Some(hwnd_b),
                WM_KEYDOWN,
                WM_KEYDOWN,
                PeekFlags::REMOVE,
            )
            .expect("B focus receives key despite stale A mouse capture");
        assert_eq!(key.wparam, 0x0d);
        Ok(())
    }

    #[test]
    fn stale_focus_key_falls_back_to_live_b_active() -> Result<()> {
        use crate::ce::gwe::WM_KEYDOWN;

        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (handle_a, thread_id) = kernel.create_guest_thread(0x401000, 0, false);
        let birth_a = kernel
            .handles
            .thread_birth_key_for_handle(handle_a)
            .expect("A thread birth");
        let hwnd_a = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth_a,
                "FOCUS_BIRTH_A",
                "A",
                None,
                None,
                0,
                0,
                0,
                Rect::default(),
            )
            .expect("A checked window");
        kernel.gwe.set_focus(Some(hwnd_a));
        assert!(kernel.mark_guest_thread_exited_by_birth(birth_a, 0));
        assert!(kernel.close_handle(handle_a)?);
        let launch_b = kernel.queue_process_launch(None, None);
        let process_b = kernel
            .handles
            .process_birth_key(launch_b.process_id)
            .expect("B process birth");
        let handle_b = kernel
            .handles
            .try_create_thread_in_birth(process_b, thread_id, 0x402000, 0, false)?;
        let birth_b = kernel
            .handles
            .thread_birth_key_for_handle(handle_b)
            .expect("B thread birth");
        kernel.set_current_process_id(launch_b.process_id);
        let hwnd_b = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth_b,
                "ACTIVE_BIRTH_B",
                "B",
                None,
                None,
                0,
                0,
                0,
                Rect::default(),
            )
            .expect("B checked window");
        kernel.gwe.set_active_window(Some(hwnd_b));
        assert_eq!(kernel.gwe.get_focus(), Some(hwnd_a));
        assert_eq!(kernel.gwe.get_active_window(), Some(hwnd_b));
        kernel.remote.enqueue_key("down", 0x0d).unwrap();

        let drain = kernel.drain_remote_input_to_active_window_detailed();

        assert_eq!(drain.posted, 1);
        assert_eq!(drain.target_thread_ids, vec![thread_id]);
        assert!(kernel.gwe.has_message_filtered_for_birth(
            birth_b,
            Some(hwnd_b),
            WM_KEYDOWN,
            WM_KEYDOWN,
        ));
        Ok(())
    }

    #[test]
    fn no_live_active_route_consumes_input_without_later_replay() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        assert_eq!(kernel.gwe.get_capture(), None);
        assert_eq!(kernel.gwe.get_active_window(), None);
        kernel.remote.set_framebuffer_size(800, 480);
        kernel.remote.enqueue_touch("down", 41, 43).unwrap();
        kernel.remote.enqueue_key("down", 0x0d).unwrap();
        let trace_before = kernel.recent_message_ops().len();

        let absent = kernel.drain_remote_input_to_active_window_detailed();
        let no_target_drops = &kernel.recent_message_ops()[trace_before..];
        for op in ["remote_touch_drop", "remote_key_drop"] {
            assert!(
                no_target_drops.iter().any(|record| {
                    record.op == op
                        && record
                            .detail
                            .as_deref()
                            .is_some_and(|detail| detail.contains("no live target"))
                }),
                "missing explicit no-live-target {op} trace"
            );
        }
        let retained = (
            kernel.remote.touch_event_count(),
            kernel.remote.key_event_count(),
        );
        let (handle_b, thread_id) = kernel.create_guest_thread(0x402000, 0, false);
        let birth_b = kernel
            .handles
            .thread_birth_key_for_handle(handle_b)
            .expect("new B thread birth");
        let hwnd_b = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth_b,
                "LATER_ACTIVE_B",
                "B",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(0, 0, 800, 480),
            )
            .expect("new B checked window");
        kernel.gwe.set_focus(Some(hwnd_b));
        kernel.gwe.set_active_window(Some(hwnd_b));
        let replayed = kernel.drain_remote_input_to_active_window_detailed();

        assert_eq!(absent.posted, 0);
        assert_eq!(
            replayed.posted, 0,
            "no-target input retained {retained:?} and replayed"
        );
        assert_eq!(retained, (0, 0));
        assert!(replayed.target_thread_ids.is_empty());
        assert_eq!(thread_id, birth_b.thread_id());
        assert!(!kernel.gwe.has_message_filtered_for_birth(
            birth_b,
            Some(hwnd_b),
            WM_LBUTTONDOWN,
            WM_LBUTTONDOWN,
        ));
        Ok(())
    }

    #[test]
    fn hidden_live_host_hint_does_not_receive_touch() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (handle, thread_id) = kernel.create_guest_thread(0x401000, 0, false);
        let birth = kernel
            .handles
            .thread_birth_key_for_handle(handle)
            .expect("hint thread birth");
        let hwnd = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth,
                "HIDDEN_HINT",
                "hidden",
                None,
                None,
                0,
                0,
                0,
                Rect::from_origin_size(0, 0, 800, 480),
            )
            .expect("checked hidden hint");
        let point = Point { x: 71, y: 83 };
        assert!(!kernel.gwe.is_window_visible(hwnd));
        assert_eq!(kernel.gwe.get_capture(), None);
        assert_eq!(kernel.gwe.window_from_point(point), None);
        kernel.remote.set_framebuffer_size(800, 480);
        kernel
            .remote
            .enqueue_touch("down", point.x, point.y)
            .unwrap();

        let drain = kernel.drain_remote_input_to_thread_window_detailed(thread_id, Some(hwnd));

        assert_eq!(drain.posted, 0);
        assert!(drain.target_thread_ids.is_empty());
        assert_eq!(kernel.remote.touch_event_count(), 0);
        assert!(!kernel.gwe.has_message_filtered_for_birth(
            birth,
            Some(hwnd),
            WM_LBUTTONDOWN,
            WM_LBUTTONDOWN,
        ));
        Ok(())
    }

    #[test]
    fn off_point_live_host_hint_does_not_receive_touch() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (handle, thread_id) = kernel.create_guest_thread(0x401000, 0, false);
        let birth = kernel
            .handles
            .thread_birth_key_for_handle(handle)
            .expect("hint thread birth");
        let hwnd = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth,
                "OFF_POINT_HINT",
                "off point",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(0, 0, 40, 40),
            )
            .expect("checked visible hint");
        let point = Point { x: 71, y: 83 };
        assert!(kernel.gwe.is_window_visible(hwnd));
        assert_eq!(kernel.gwe.get_capture(), None);
        assert_eq!(kernel.gwe.window_from_point(point), None);
        kernel.remote.set_framebuffer_size(800, 480);
        kernel
            .remote
            .enqueue_touch("down", point.x, point.y)
            .unwrap();

        let drain = kernel.drain_remote_input_to_thread_window_detailed(thread_id, Some(hwnd));

        assert_eq!(drain.posted, 0);
        assert!(drain.target_thread_ids.is_empty());
        assert_eq!(kernel.remote.touch_event_count(), 0);
        assert!(!kernel.gwe.has_message_filtered_for_birth(
            birth,
            Some(hwnd),
            WM_LBUTTONDOWN,
            WM_LBUTTONDOWN,
        ));
        Ok(())
    }

    #[test]
    fn disabled_modal_owner_does_not_steal_off_point_dialog_touch() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (handle, thread_id) = kernel.create_guest_thread(0x401000, 0, false);
        let birth = kernel
            .handles
            .thread_birth_key_for_handle(handle)
            .expect("modal thread birth");
        let owner = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth,
                "MODAL_DISABLED_OWNER",
                "owner",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(0, 0, 800, 480),
            )
            .expect("checked owner");
        let dialog = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth,
                "MODAL_OFF_POINT_DIALOG",
                "dialog",
                None,
                Some(owner),
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(400, 300, 100, 100),
            )
            .expect("checked owned dialog");
        let point = Point { x: 71, y: 83 };
        assert_eq!(kernel.enable_window(owner, false), Some(true));
        assert!(!kernel.gwe.is_window_enabled(owner));
        assert!(kernel.gwe.is_window_visible(dialog));
        assert_eq!(kernel.gwe.get_capture(), None);
        assert_eq!(kernel.gwe.window_from_point(point), None);
        kernel.remote.set_framebuffer_size(800, 480);
        kernel
            .remote
            .enqueue_touch("down", point.x, point.y)
            .unwrap();

        let drain = kernel.drain_remote_input_to_thread_window_detailed(thread_id, Some(dialog));

        assert_eq!(drain.posted, 0);
        assert!(drain.target_thread_ids.is_empty());
        assert_eq!(kernel.remote.touch_event_count(), 0);
        for hwnd in [owner, dialog] {
            assert!(!kernel.gwe.has_message_filtered_for_birth(
                birth,
                Some(hwnd),
                WM_LBUTTONDOWN,
                WM_LBUTTONDOWN,
            ));
        }
        Ok(())
    }

    #[test]
    fn stale_topmost_hit_test_falls_through_to_live_b_host_hint() -> Result<()> {
        use crate::ce::gwe::MSGSRC_SOFTWARE_POST;

        let (mut kernel, _handle_b, birth_b, hwnd_a, hwnd_b) = stale_window_remote_input_fixture()?;
        let thread_id = birth_b.thread_id();
        let point = Point { x: 211, y: 223 };
        assert_eq!(kernel.gwe.get_capture(), None);
        // Z-order is independent of A's expired thread birth.
        assert!(kernel.gwe.set_window_pos(
            hwnd_a,
            Some(HWND_TOP),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE,
        ));
        assert_eq!(kernel.gwe.window_from_point(point), Some(hwnd_a));
        assert_eq!(kernel.recipient_birth_for_window(hwnd_a), None);
        assert!(kernel.gwe.is_window_visible(hwnd_b));
        assert!(kernel.gwe.is_window_enabled(hwnd_b));
        assert!(
            kernel
                .gwe
                .get_window_rect(hwnd_b)
                .is_some_and(|rect| rect.contains_point(point))
        );
        kernel
            .remote
            .enqueue_touch("down", point.x, point.y)
            .unwrap();
        let trace_before = kernel.recent_message_ops().len();

        // The topmost coordinate hit is stale A. Reject it, then try the
        // independently valid B host wait/modal hint for this same point.
        let drain = kernel.drain_remote_input_to_thread_window_detailed(thread_id, Some(hwnd_b));

        assert_eq!(drain.posted, 1);
        assert_eq!(drain.target_thread_ids, vec![thread_id]);
        assert_eq!(kernel.remote.touch_event_count(), 0);
        let b_message = kernel
            .gwe
            .peek_message_filtered_for_birth(
                birth_b,
                Some(hwnd_b),
                WM_LBUTTONDOWN,
                WM_LBUTTONDOWN,
                PeekFlags::REMOVE,
            )
            .expect("live exact-Birth B hint receives the touch");
        assert_eq!(b_message.hwnd, hwnd_b);
        assert_eq!(
            b_message.mouse_pos_at_post,
            Some(make_lparam(point.x, point.y))
        );
        assert_eq!(b_message.source, MSGSRC_SOFTWARE_POST);
        let targets: Vec<_> = kernel.recent_message_ops()[trace_before..]
            .iter()
            .filter(|record| record.op == "remote_touch_target")
            .collect();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].hwnd, Some(hwnd_b));
        assert_eq!(targets[0].thread_id, thread_id);
        Ok(())
    }

    #[test]
    fn key_without_explicit_focus_or_active_does_not_use_top_level_fallback() -> Result<()> {
        use crate::ce::gwe::{QS_KEY, WM_KEYDOWN};

        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (handle_b, thread_id) = kernel.create_guest_thread(0x401000, 0, false);
        let birth_b = kernel
            .handles
            .thread_birth_key_for_handle(handle_b)
            .expect("B thread birth");
        let hwnd_b = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth_b,
                "UNFOCUSED_TOP_LEVEL_B",
                "B",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(0, 0, 800, 480),
            )
            .expect("live checked B window");
        kernel.gwe.set_focus(None);
        kernel.gwe.set_active_window(None);
        assert_eq!(kernel.gwe.get_focus(), None);
        assert!(kernel.gwe.is_window(hwnd_b));
        assert!(kernel.gwe.is_window_visible(hwnd_b));
        let status_before = kernel.gwe.peek_queue_status_for_birth(birth_b, QS_KEY);
        let ready_before = kernel.get_message_queue_ready_time_stamp_for_thread(thread_id, 0);
        let wakes_before = kernel.scheduler_stats().message_input_wake_candidate_count;
        let trace_before = kernel.recent_message_ops().len();
        kernel.remote.enqueue_key("down", 0x0d).unwrap();

        // GWE may return this arbitrary top-level HWND as its convenience
        // active fallback. A host key requires explicit live focus or active.
        let drain = kernel.drain_remote_input_to_active_window_detailed();

        assert_eq!(drain.posted, 0);
        assert!(drain.target_thread_ids.is_empty());
        assert_eq!(kernel.remote.key_event_count(), 0);
        assert_eq!(
            kernel.gwe.peek_queue_status_for_birth(birth_b, QS_KEY),
            status_before
        );
        assert_eq!(
            kernel.get_message_queue_ready_time_stamp_for_thread(thread_id, 0),
            ready_before
        );
        assert_eq!(
            kernel.scheduler_stats().message_input_wake_candidate_count,
            wakes_before
        );
        assert!(!kernel.gwe.has_message_filtered_for_birth(
            birth_b,
            Some(hwnd_b),
            WM_KEYDOWN,
            WM_KEYDOWN,
        ));
        assert!(
            kernel.recent_message_ops()[trace_before..]
                .iter()
                .any(|record| {
                    record.op == "remote_key_drop"
                        && record
                            .detail
                            .as_deref()
                            .is_some_and(|detail| detail.contains("no live target"))
                })
        );
        assert!(
            !kernel.recent_message_ops()[trace_before..]
                .iter()
                .any(|record| record.op == "remote_key_target" && record.hwnd == Some(hwnd_b))
        );
        Ok(())
    }

    #[test]
    fn disabled_stored_focus_key_drops_without_arbitrary_owner_fallback() -> Result<()> {
        use crate::ce::gwe::{QS_KEY, WM_KEYDOWN};

        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (handle_a, thread_a) = kernel.create_guest_thread(0x401000, 0, false);
        let (handle_b, thread_b) = kernel.create_guest_thread(0x402000, 0, false);
        let birth_a = kernel
            .handles
            .thread_birth_key_for_handle(handle_a)
            .expect("A thread birth");
        let birth_b = kernel
            .handles
            .thread_birth_key_for_handle(handle_b)
            .expect("B thread birth");
        let hwnd_a = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth_a,
                "DISABLED_FOCUS_A",
                "A",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(0, 0, 400, 480),
            )
            .expect("checked A window");
        let hwnd_b = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth_b,
                "UNFOCUSED_OWNER_B",
                "B",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(400, 0, 400, 480),
            )
            .expect("checked B window");
        kernel.gwe.set_focus(Some(hwnd_a));
        kernel.gwe.set_active_window(None);
        // GWE toggle leaves a disabled HWND stored as focus. The public
        // kernel EnableWindow wrapper would clear focus, so it cannot model
        // this stale stored-focus boundary.
        assert!(kernel.gwe.enable_window(hwnd_a, false));
        assert_eq!(kernel.gwe.get_focus(), Some(hwnd_a));
        assert_eq!(kernel.gwe.get_stored_active_window(), None);
        assert!(!kernel.gwe.is_window_enabled(hwnd_a));
        assert!(kernel.gwe.is_window_enabled(hwnd_b));
        assert_eq!(kernel.gwe.get_active_window(), Some(hwnd_b));
        let status_a = kernel.gwe.peek_queue_status_for_birth(birth_a, QS_KEY);
        let status_b = kernel.gwe.peek_queue_status_for_birth(birth_b, QS_KEY);
        let ready_a = kernel.get_message_queue_ready_time_stamp_for_thread(thread_a, 0);
        let ready_b = kernel.get_message_queue_ready_time_stamp_for_thread(thread_b, 0);
        let wakes_before = kernel.scheduler_stats().message_input_wake_candidate_count;
        let trace_before = kernel.recent_message_ops().len();
        kernel.remote.enqueue_key("down", 0x0d).unwrap();

        let drain = kernel.drain_remote_input_to_active_window_detailed();

        assert_eq!(drain.posted, 0);
        assert!(drain.target_thread_ids.is_empty());
        assert_eq!(kernel.remote.key_event_count(), 0);
        assert_eq!(
            kernel.gwe.peek_queue_status_for_birth(birth_a, QS_KEY),
            status_a
        );
        assert_eq!(
            kernel.gwe.peek_queue_status_for_birth(birth_b, QS_KEY),
            status_b
        );
        assert_eq!(
            kernel.get_message_queue_ready_time_stamp_for_thread(thread_a, 0),
            ready_a
        );
        assert_eq!(
            kernel.get_message_queue_ready_time_stamp_for_thread(thread_b, 0),
            ready_b
        );
        assert_eq!(
            kernel.scheduler_stats().message_input_wake_candidate_count,
            wakes_before
        );
        for (birth, hwnd) in [(birth_a, hwnd_a), (birth_b, hwnd_b)] {
            assert!(!kernel.gwe.has_message_filtered_for_birth(
                birth,
                Some(hwnd),
                WM_KEYDOWN,
                WM_KEYDOWN,
            ));
        }
        assert!(
            kernel.recent_message_ops()[trace_before..]
                .iter()
                .any(|record| {
                    record.op == "remote_key_drop"
                        && record
                            .detail
                            .as_deref()
                            .is_some_and(|detail| detail.contains("no live target"))
                })
        );
        Ok(())
    }

    #[test]
    fn remote_input_blocked_visible_hit_test_overrides_hidden_active_window() -> Result<()> {
        let config = RuntimeConfig::load_default()?;
        let mut kernel = CeKernel::boot(config);
        let active_thread_id = 1;
        let blocked_thread_id = 2;
        let active = kernel.create_window_ex_w(active_thread_id, "ACTIVE", "", None, 0, 0, 0);
        let blocked = kernel.create_window_ex_w_with_rect(
            blocked_thread_id,
            "BLOCKED",
            "",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, 800, 480),
        );
        assert_eq!(kernel.gwe.set_active_window(Some(active)), None);
        // Supersedes the old hidden-hint oracle: the blocked HWND is only a
        // fallback hint. This window earns delivery by visibility and point hit.
        assert_eq!(
            kernel.gwe.window_from_point(Point { x: 640, y: 440 }),
            Some(blocked)
        );
        // A visible unvalidated window can synthesize WM_PAINT indefinitely.
        // Mouse-filtered reads below need no pre-input general-queue drain.

        kernel.remote.set_framebuffer_size(800, 480);
        kernel.remote.enqueue_touch("tap", 640, 440).unwrap();

        assert_eq!(
            kernel.drain_remote_input_to_thread_window(blocked_thread_id, Some(blocked)),
            2
        );

        assert!(!kernel.gwe.has_message_filtered(
            active_thread_id,
            Some(active),
            WM_LBUTTONDOWN,
            WM_LBUTTONUP,
        ));
        let down = kernel
            .gwe
            .peek_message_filtered(
                blocked_thread_id,
                Some(blocked),
                WM_LBUTTONDOWN,
                WM_LBUTTONDOWN,
                PeekFlags::REMOVE,
            )
            .expect("blocked target receives mouse down");
        let up = kernel
            .gwe
            .peek_message_filtered(
                blocked_thread_id,
                Some(blocked),
                WM_LBUTTONUP,
                WM_LBUTTONUP,
                PeekFlags::REMOVE,
            )
            .expect("blocked target receives mouse up");
        assert_eq!(down.hwnd, blocked);
        assert_eq!(up.hwnd, blocked);
        assert!(up.time_ms > down.time_ms);
        Ok(())
    }

    #[test]
    fn remote_key_input_uses_focused_window_owner_thread() -> Result<()> {
        let config = RuntimeConfig::load_default()?;
        let mut kernel = CeKernel::boot(config);
        let owner_thread_id = 1;
        let draining_thread_id = 2;
        let owned = kernel.create_window_ex_w(owner_thread_id, "OWNER", "", None, 0, 0, 0);
        let helper = kernel.create_window_ex_w(draining_thread_id, "HELPER", "", None, 0, 0, 0);
        assert!(kernel.gwe.is_window(helper));
        kernel.gwe.set_focus(Some(owned));
        kernel.gwe.set_active_window(Some(helper));
        assert_eq!(kernel.gwe.get_focus(), Some(owned));
        // Supersedes the old key-hint oracle: the key follows live focus,
        // even when a different window is the host wait hint and active HWND.
        while kernel.gwe.get_message(owner_thread_id).is_some() {}
        while kernel.gwe.get_message(draining_thread_id).is_some() {}

        kernel.remote.enqueue_key("down", 0x0d).unwrap();

        assert_eq!(
            kernel.drain_remote_input_to_thread_window(draining_thread_id, Some(helper)),
            1
        );

        let owner_message = kernel
            .gwe
            .peek_message_filtered(
                owner_thread_id,
                Some(owned),
                crate::ce::gwe::WM_KEYDOWN,
                crate::ce::gwe::WM_KEYDOWN,
                PeekFlags::REMOVE,
            )
            .expect("window owner receives remote key");
        assert_eq!(owner_message.hwnd, owned);
        assert_eq!(owner_message.wparam, 0x0d);
        assert!(
            kernel
                .gwe
                .peek_message_filtered(
                    draining_thread_id,
                    Some(owned),
                    crate::ce::gwe::WM_KEYDOWN,
                    crate::ce::gwe::WM_KEYDOWN,
                    PeekFlags::REMOVE,
                )
                .is_none()
        );
        Ok(())
    }

    #[test]
    fn remote_key_before_focus_changing_down_preserves_cross_type_enqueue_order() -> Result<()> {
        use crate::ce::gwe::{MSGSRC_HARDWARE_KEYBOARD, MSGSRC_SOFTWARE_POST, WM_KEYDOWN};

        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (handle_a, thread_a) = kernel.create_guest_thread(0x401000, 0, false);
        let (handle_b, thread_b) = kernel.create_guest_thread(0x402000, 0, false);
        let birth_a = kernel
            .handles
            .thread_birth_key_for_handle(handle_a)
            .expect("A thread birth");
        let birth_b = kernel
            .handles
            .thread_birth_key_for_handle(handle_b)
            .expect("B thread birth");
        assert_ne!(birth_a, birth_b);
        let hwnd_a = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth_a,
                "INPUT_ORDER_A",
                "A",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(0, 0, 400, 480),
            )
            .expect("A Birth-owned window");
        let hwnd_b = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                birth_b,
                "INPUT_ORDER_B",
                "B",
                None,
                None,
                0,
                WS_VISIBLE,
                0,
                Rect::from_origin_size(400, 0, 400, 480),
            )
            .expect("B Birth-owned window");
        assert!(matches!(
            kernel.gwe.window(hwnd_a).map(|window| window.delivery_owner()),
            Some(WindowDeliveryOwner::Birth { thread, .. }) if thread == birth_a
        ));
        assert!(matches!(
            kernel.gwe.window(hwnd_b).map(|window| window.delivery_owner()),
            Some(WindowDeliveryOwner::Birth { thread, .. }) if thread == birth_b
        ));

        let point = Point { x: 450, y: 30 };
        assert_eq!(kernel.gwe.get_capture(), None);
        assert_eq!(kernel.gwe.window_from_point(point), Some(hwnd_b));
        let _ = kernel.set_focus(Some(hwnd_a));
        assert_eq!(kernel.gwe.get_focus(), Some(hwnd_a));
        kernel.remote.set_framebuffer_size(800, 480);
        let trace_before = kernel.recent_message_ops().len();

        kernel.remote.enqueue_key("down", 0x41).unwrap();
        kernel
            .remote
            .enqueue_touch("down", point.x, point.y)
            .unwrap();
        kernel.remote.enqueue_key("down", 0x42).unwrap();
        assert_eq!(kernel.remote.key_event_count(), 2);
        assert_eq!(kernel.remote.touch_event_count(), 1);

        let drain = kernel.drain_remote_input_to_active_window_detailed();

        assert_eq!(drain.posted, 3);
        assert_eq!(kernel.remote.key_event_count(), 0);
        assert_eq!(kernel.remote.touch_event_count(), 0);
        let key = kernel
            .gwe
            .peek_message_filtered_for_birth(
                birth_a,
                Some(hwnd_a),
                WM_KEYDOWN,
                WM_KEYDOWN,
                PeekFlags::REMOVE,
            )
            .expect("key admitted before B DOWN must reach focused A Birth");
        assert_eq!(key.hwnd, hwnd_a);
        assert_eq!(key.wparam, 0x41);
        assert_eq!(key.source, MSGSRC_HARDWARE_KEYBOARD);
        let down = kernel
            .gwe
            .peek_message_filtered_for_birth(
                birth_b,
                Some(hwnd_b),
                WM_LBUTTONDOWN,
                WM_LBUTTONDOWN,
                PeekFlags::REMOVE,
            )
            .expect("later coordinate DOWN must reach B Birth");
        assert_eq!(down.hwnd, hwnd_b);
        assert_eq!(down.mouse_pos_at_post, Some(make_lparam(point.x, point.y)));
        assert_eq!(down.source, MSGSRC_SOFTWARE_POST);
        let later_key = kernel
            .gwe
            .peek_message_filtered_for_birth(
                birth_b,
                Some(hwnd_b),
                WM_KEYDOWN,
                WM_KEYDOWN,
                PeekFlags::REMOVE,
            )
            .expect("key admitted after B DOWN must reach newly focused B Birth");
        assert_eq!(later_key.hwnd, hwnd_b);
        assert_eq!(later_key.wparam, 0x42);
        assert_eq!(later_key.source, MSGSRC_HARDWARE_KEYBOARD);
        assert_eq!(kernel.gwe.get_focus(), Some(hwnd_b));
        assert_eq!(drain.target_thread_ids, vec![thread_a, thread_b]);
        assert!(!kernel.gwe.has_message_filtered_for_birth(
            birth_a,
            Some(hwnd_a),
            WM_LBUTTONDOWN,
            WM_LBUTTONDOWN,
        ));
        let targets: Vec<_> = kernel.recent_message_ops()[trace_before..]
            .iter()
            .filter(|record| matches!(record.op, "remote_key_target" | "remote_touch_target"))
            .map(|record| (record.op, record.hwnd, record.thread_id))
            .collect();
        assert_eq!(
            targets,
            vec![
                ("remote_key_target", Some(hwnd_a), thread_a),
                ("remote_touch_target", Some(hwnd_b), thread_b),
                ("remote_key_target", Some(hwnd_b), thread_b),
            ]
        );
        Ok(())
    }

    #[test]
    fn modal_button_dismisses_only_requested_dialog_and_never_popup() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let thread_id = 77;
        let dialog_a = kernel.gwe.create_window(thread_id, "dialog", "A");
        let dialog_b = kernel.gwe.create_window(thread_id, "dialog", "B");
        let popup_wait = kernel.register_blocked_waiter(
            thread_id,
            0,
            Vec::new(),
            SchedulerBlockedWaitKind::PopupMenuModal {
                mouse_message_max: crate::ce::gwe::WM_LBUTTONUP,
            },
            0,
            crate::ce::timer::INFINITE,
        );
        let modal_b = kernel.register_blocked_waiter(
            thread_id,
            0,
            Vec::new(),
            SchedulerBlockedWaitKind::ModalMessageBox {
                dialog_hwnd: dialog_b,
            },
            0,
            crate::ce::timer::INFINITE,
        );
        let get_a = kernel.register_blocked_waiter(
            thread_id,
            0,
            Vec::new(),
            SchedulerBlockedWaitKind::GetMessage {
                hwnd: Some(dialog_a),
                min_msg: 0,
                max_msg: 0,
            },
            0,
            crate::ce::timer::INFINITE,
        );
        let get_b = kernel.register_blocked_waiter(
            thread_id,
            0,
            Vec::new(),
            SchedulerBlockedWaitKind::GetMessage {
                hwnd: Some(dialog_b),
                min_msg: 0,
                max_msg: 0,
            },
            0,
            crate::ce::timer::INFINITE,
        );
        assert!(!kernel.dismiss_modal_dialog_wait_from_button(dialog_a, thread_id, 1, 0));
        assert!(!kernel.dismiss_modal_dialog_wait_from_button(dialog_b, thread_id + 1, 1, 0));
        assert!(kernel.blocked_waiter(get_a).is_some());
        assert!(kernel.blocked_waiter(get_b).is_some());
        assert!(!kernel.has_modal_dialog_result(thread_id, dialog_a));

        let modal_a = kernel.register_blocked_waiter(
            thread_id,
            0,
            Vec::new(),
            SchedulerBlockedWaitKind::ModalMessageBox {
                dialog_hwnd: dialog_a,
            },
            0,
            crate::ce::timer::INFINITE,
        );
        assert!(kernel.dismiss_modal_dialog_wait_from_button(dialog_a, thread_id, 1, 0));
        assert!(kernel.blocked_waiter(get_a).is_none());
        assert!(kernel.blocked_waiter(get_b).is_some());
        assert!(kernel.blocked_waiter(modal_a).is_some());
        assert!(kernel.blocked_waiter(modal_b).is_some());
        assert!(kernel.blocked_waiter(popup_wait).is_some());
        assert!(kernel.has_modal_dialog_result(thread_id, dialog_a));
        assert!(!kernel.has_modal_dialog_result(thread_id, dialog_b));
    }

    #[test]
    fn modal_button_preserves_unknown_prior_owner_after_tid_reuse() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let thread_id = 77;
        let old_dialog = kernel.gwe.create_window(thread_id, "dialog", "old");
        let new_dialog = kernel.gwe.create_window(thread_id, "dialog", "new");
        let unknown_modal = kernel.register_blocked_waiter(
            thread_id,
            0,
            Vec::new(),
            SchedulerBlockedWaitKind::ModalMessageBox {
                dialog_hwnd: old_dialog,
            },
            0,
            crate::ce::timer::INFINITE,
        );
        let unknown_get = kernel.register_blocked_waiter(
            thread_id,
            0,
            Vec::new(),
            SchedulerBlockedWaitKind::GetMessage {
                hwnd: Some(old_dialog),
                min_msg: 0,
                max_msg: 0,
            },
            0,
            crate::ce::timer::INFINITE,
        );
        assert_eq!(
            kernel.blocked_waiter(unknown_modal).unwrap().owner_birth,
            None
        );
        let owner_handle = kernel.handles.create_thread(thread_id, 0x401000, 0, false);
        let owner_birth = kernel
            .handles
            .thread_birth_key_for_handle(owner_handle)
            .expect("new live owner birth");
        let new_modal = kernel
            .try_register_opaque_waiter_for_birth(
                owner_birth,
                owner_handle,
                Vec::new(),
                SchedulerBlockedWaitKind::ModalMessageBox {
                    dialog_hwnd: new_dialog,
                },
                0,
                crate::ce::timer::INFINITE,
            )
            .expect("new modal owner");
        let new_get = kernel
            .try_register_opaque_waiter_for_birth(
                owner_birth,
                owner_handle,
                Vec::new(),
                SchedulerBlockedWaitKind::GetMessage {
                    hwnd: Some(new_dialog),
                    min_msg: 0,
                    max_msg: 0,
                },
                0,
                crate::ce::timer::INFINITE,
            )
            .expect("new GetMessage owner");
        assert!(!kernel.dismiss_modal_dialog_wait_from_button(old_dialog, thread_id, 1, 0));
        assert!(kernel.blocked_waiter(unknown_modal).is_some());
        assert!(kernel.blocked_waiter(unknown_get).is_some());
        assert!(kernel.blocked_waiter(new_get).is_some());
        assert!(!kernel.has_modal_dialog_result(thread_id, old_dialog));
        assert!(kernel.dismiss_modal_dialog_wait_from_button(new_dialog, thread_id, 2, 0));
        assert!(kernel.blocked_waiter(new_modal).is_some());
        assert!(kernel.blocked_waiter(new_get).is_none());
        assert!(kernel.blocked_waiter(unknown_get).is_some());
        assert!(kernel.has_modal_dialog_result(thread_id, new_dialog));
    }

    #[test]
    fn remote_button_tap_dismisses_modal_dialog_waiter() -> Result<()> {
        let config = RuntimeConfig::load_default()?;
        let mut kernel = CeKernel::boot(config);
        let background_thread_id = 1;
        let thread_id = 3;
        let background = kernel.create_window_ex_w_with_rect(
            background_thread_id,
            "static",
            "background",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, 800, 480),
        );
        assert!(kernel.gwe.validate_window(background));
        let background_wait_id = kernel.register_blocked_waiter(
            background_thread_id,
            0x101,
            Vec::new(),
            SchedulerBlockedWaitKind::GetMessage {
                hwnd: None,
                min_msg: 0,
                max_msg: 0,
            },
            kernel.timers.tick_count(),
            crate::ce::timer::INFINITE,
        );
        let dialog = kernel.create_window_ex_w_with_rect(
            thread_id,
            "dialog",
            "Button",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(40, 40, 120, 90),
        );
        let button = kernel.create_window_ex_w_with_rect(
            thread_id,
            "button",
            "OK",
            Some(dialog),
            1,
            WS_VISIBLE | WS_CHILD | BS_DEFPUSHBUTTON,
            0,
            Rect::from_origin_size(7, 27, 54, 22),
        );
        let wait_id = kernel.register_blocked_waiter(
            thread_id,
            0x103,
            Vec::new(),
            SchedulerBlockedWaitKind::ModalMessageBox {
                dialog_hwnd: dialog,
            },
            kernel.timers.tick_count(),
            crate::ce::timer::INFINITE,
        );
        let dialog_get_message_wait_id = kernel.register_blocked_waiter(
            thread_id,
            0x104,
            Vec::new(),
            SchedulerBlockedWaitKind::GetMessage {
                hwnd: Some(dialog),
                min_msg: crate::ce::gwe::WM_PAINT,
                max_msg: crate::ce::gwe::WM_LBUTTONUP,
            },
            kernel.timers.tick_count(),
            crate::ce::timer::INFINITE,
        );

        kernel.remote.set_framebuffer_size(800, 480);
        let button_rect = kernel
            .gwe
            .get_window_rect(button)
            .expect("button should have a screen rect");
        let button_point = Point {
            x: button_rect.left + button_rect.width() / 2,
            y: button_rect.top + button_rect.height() / 2,
        };
        assert_eq!(
            kernel
                .gwe
                .window_from_point_for_thread(thread_id, button_point),
            Some(button)
        );
        kernel
            .remote
            .enqueue_touch("tap", button_point.x, button_point.y)
            .unwrap();

        assert_eq!(
            kernel.drain_remote_input_to_thread_window(thread_id, Some(dialog)),
            2
        );
        assert!(kernel.blocked_waiter(wait_id).is_some());
        assert!(kernel.blocked_waiter(dialog_get_message_wait_id).is_none());
        assert!(kernel.thread_has_modal_dialog_result(thread_id));
        assert!(kernel.gwe.is_window(dialog));
        assert!(kernel.gwe.is_window(button));
        assert!(kernel.gwe.update_rect(background).is_none());
        let ready =
            kernel.select_ready_blocked_waiter(0, kernel.timers.tick_count(), |blocked, kernel| {
                match blocked.kind {
                    SchedulerBlockedWaitKind::GetMessage {
                        hwnd,
                        min_msg,
                        max_msg,
                    } => kernel
                        .gwe
                        .has_message_filtered(blocked.thread_id, hwnd, min_msg, max_msg),
                    SchedulerBlockedWaitKind::ModalMessageBox { .. } => {
                        kernel.thread_has_modal_dialog_result(blocked.thread_id)
                    }
                    _ => false,
                }
            });
        assert_ne!(ready, Some(background_wait_id));
        assert_eq!(ready, Some(wait_id));
        assert_eq!(kernel.take_modal_dialog_result(thread_id, dialog), Some(1));
        assert_eq!(kernel.take_modal_dialog_result(thread_id, dialog), None);
        Ok(())
    }

    #[test]
    fn remote_input_any_blocked_thread_uses_desktop_hit_test_owner() -> Result<()> {
        let config = RuntimeConfig::load_default()?;
        let mut kernel = CeKernel::boot(config);
        let visible_thread_id = 1;
        let helper_thread_id = 2;
        let visible = kernel.create_window_ex_w_with_rect(
            visible_thread_id,
            "VISIBLE",
            "",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, 800, 480),
        );
        let hidden_helper = kernel.create_window_ex_w_with_rect(
            helper_thread_id,
            "HELPER",
            "",
            None,
            0,
            0,
            0,
            Rect::from_origin_size(0, 0, 800, 480),
        );
        assert_eq!(kernel.gwe.set_active_window(Some(hidden_helper)), None);

        kernel.remote.set_framebuffer_size(800, 480);
        kernel.remote.enqueue_touch("tap", 410, 454).unwrap();

        assert_eq!(
            kernel.drain_remote_input_to_thread_window(helper_thread_id, None),
            2
        );

        let down = kernel
            .gwe
            .peek_message_filtered(
                visible_thread_id,
                Some(visible),
                WM_LBUTTONDOWN,
                WM_LBUTTONUP,
                PeekFlags::REMOVE,
            )
            .expect("visible owner receives mouse down");
        assert_eq!(down.hwnd, visible);
        assert_eq!(down.msg, WM_LBUTTONDOWN);
        let up = kernel
            .gwe
            .peek_message_filtered(
                visible_thread_id,
                Some(visible),
                WM_LBUTTONDOWN,
                WM_LBUTTONUP,
                PeekFlags::REMOVE,
            )
            .expect("visible owner receives mouse up");
        assert_eq!(up.hwnd, visible);
        assert_eq!(up.msg, WM_LBUTTONUP);
        assert!(
            kernel
                .gwe
                .peek_message_filtered(
                    helper_thread_id,
                    Some(hidden_helper),
                    WM_LBUTTONDOWN,
                    WM_LBUTTONUP,
                    PeekFlags::REMOVE,
                )
                .is_none()
        );
        Ok(())
    }

    #[test]
    fn remote_input_active_window_uses_desktop_hit_test_over_hidden_active_window() -> Result<()> {
        let config = RuntimeConfig::load_default()?;
        let mut kernel = CeKernel::boot(config);
        let visible_thread_id = 1;
        let hidden_thread_id = 2;
        let visible = kernel.create_window_ex_w_with_rect(
            visible_thread_id,
            "VISIBLE",
            "",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, 800, 480),
        );
        let hidden = kernel.create_window_ex_w_with_rect(
            hidden_thread_id,
            "HIDDEN",
            "",
            None,
            0,
            0,
            0,
            Rect::from_origin_size(0, 0, 800, 480),
        );
        assert_eq!(kernel.gwe.set_active_window(Some(hidden)), None);

        kernel.remote.set_framebuffer_size(800, 480);
        kernel.remote.enqueue_touch("tap", 768, 88).unwrap();

        assert_eq!(kernel.drain_remote_input_to_active_window(), 2);

        let down = kernel
            .gwe
            .peek_message_filtered(
                visible_thread_id,
                Some(visible),
                WM_LBUTTONDOWN,
                WM_LBUTTONUP,
                PeekFlags::REMOVE,
            )
            .expect("visible desktop hit-test target receives mouse down");
        assert_eq!(down.hwnd, visible);
        assert_eq!(down.msg, WM_LBUTTONDOWN);
        let up = kernel
            .gwe
            .peek_message_filtered(
                visible_thread_id,
                Some(visible),
                WM_LBUTTONDOWN,
                WM_LBUTTONUP,
                PeekFlags::REMOVE,
            )
            .expect("visible desktop hit-test target receives mouse up");
        assert_eq!(up.hwnd, visible);
        assert_eq!(up.msg, WM_LBUTTONUP);
        assert!(
            kernel
                .gwe
                .peek_message_filtered(
                    hidden_thread_id,
                    Some(hidden),
                    WM_LBUTTONDOWN,
                    WM_LBUTTONUP,
                    PeekFlags::REMOVE,
                )
                .is_none()
        );
        Ok(())
    }

    #[test]
    fn terminate_current_process_destroys_owned_windows() -> Result<()> {
        let config = RuntimeConfig::load_default()?;
        let mut kernel = CeKernel::boot(config);
        let _process_77 = kernel.handles.create_process(77);
        let _process_88 = kernel.handles.create_process(88);
        kernel.set_current_process_id(77);
        let parent = kernel.create_window_ex_w(1, "PARENT", "", None, 0, 0, 0);
        let child = kernel.create_window_ex_w(1, "CHILD", "", Some(parent), 0, WS_CHILD, 0);

        kernel.set_current_process_id(88);
        let survivor = kernel.create_window_ex_w(2, "SURVIVOR", "", None, 0, 0, 0);
        kernel.set_current_process_id(77);

        assert!(kernel.terminate_process(CE_CURRENT_PROCESS_PSEUDO_HANDLE, 0x1234));

        assert!(!kernel.gwe.is_window(parent));
        assert!(!kernel.gwe.is_window(child));
        assert!(kernel.gwe.is_window(survivor));
        assert_eq!(
            kernel.process_exit_code_for_handle(CE_CURRENT_PROCESS_PSEUDO_HANDLE),
            Some(0x1234)
        );
        Ok(())
    }

    #[test]
    fn pseudo_process_exit_without_birth_fails_before_host_mutation() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        kernel.set_current_process_id(77);
        let hwnd = kernel.create_window_ex_w(9, "UNKNOWN_PROCESS", "", None, 0, 0, 0);
        assert_eq!(kernel.current_process_birth_key(), None);

        assert!(!kernel.terminate_process(CE_CURRENT_PROCESS_PSEUDO_HANDLE, 0x1234));
        assert!(kernel.gwe.is_window(hwnd));
        assert_eq!(kernel.current_process_exit_code, STILL_ACTIVE);
        assert!(!kernel.current_process_signaled);
        Ok(())
    }

    #[test]
    fn typed_message_payloads_use_recipient_birth_and_discard_exact_window_owner() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let owner_a = kernel.current_process_birth_key().expect("boot birth");
        let hwnd_a = kernel.create_window_ex_w(11, "PAYLOAD_A", "", None, 0, WS_VISIBLE, 0);
        let launch_b = kernel.queue_process_launch(None, None);
        let owner_b = kernel
            .handles
            .process_birth_key(launch_b.process_id)
            .expect("child birth");
        kernel.set_current_process_id(launch_b.process_id);
        let hwnd_b = kernel.create_window_ex_w(22, "PAYLOAD_B", "", None, 0, WS_VISIBLE, 0);
        kernel.set_current_process_id(1);

        let position = |hwnd| WindowPos {
            hwnd,
            insert_after: HWND_TOP,
            x: 1,
            y: 2,
            width: 3,
            height: 4,
            flags: 0,
        };
        // The mutable current selector is A, but the first producer targets B.
        let (produced_b, ptr_b) = kernel
            .queue_window_pos_payload(hwnd_b, position(hwnd_b))
            .expect("B payload");
        let (produced_a, ptr_a) = kernel
            .queue_window_pos_payload(hwnd_a, position(hwnd_a))
            .expect("A payload");
        assert_eq!((produced_a, produced_b), (owner_a, owner_b));
        assert_eq!(ptr_a, ptr_b, "independent births may reuse one guest VA");
        assert!(kernel.post_window_typed_payload_message(
            hwnd_a,
            WM_WINDOWPOSCHANGED,
            0,
            ptr_a,
            owner_a,
        ));
        assert!(kernel.post_window_typed_payload_message(
            hwnd_b,
            WM_WINDOWPOSCHANGED,
            0,
            ptr_b,
            owner_b,
        ));
        let message_a = kernel
            .take_ready_visible_message_w_filtered(
                11,
                Some(hwnd_a),
                WM_WINDOWPOSCHANGED,
                WM_WINDOWPOSCHANGED,
            )
            .expect("A queued message");
        let message_b = kernel
            .peek_ready_visible_message_w_filtered(
                22,
                Some(hwnd_b),
                WM_WINDOWPOSCHANGED,
                WM_WINDOWPOSCHANGED,
            )
            .expect("B queued message");
        assert_eq!(message_a.typed_payload_owner, Some(owner_a));
        assert_eq!(message_b.typed_payload_owner, Some(owner_b));
        assert_eq!(
            kernel.message_pointer_payload_for_message(&message_a),
            Some(MessagePointerPayload::WindowPos(position(hwnd_a)))
        );
        assert_eq!(
            kernel.message_pointer_payload_for_message(&message_b),
            Some(MessagePointerPayload::WindowPos(position(hwnd_b)))
        );
        assert!(
            kernel
                .message_pointer_payload_for_message(&Message::new(
                    hwnd_a,
                    WM_WINDOWPOSCHANGED,
                    0,
                    ptr_a,
                    0,
                ))
                .is_none(),
            "an unmarked guest pointer must not inherit host provenance"
        );
        assert_eq!(kernel.dispatch_message_w_for_thread(11, message_a), 0);
        assert!(kernel.gwe.message_pointer_payload(owner_a, ptr_a).is_none());
        assert!(kernel.gwe.message_pointer_payload(owner_b, ptr_b).is_some());
        assert!(kernel.destroy_window(hwnd_b));
        assert!(kernel.gwe.message_pointer_payload(owner_b, ptr_b).is_none());
        assert!(
            kernel
                .memory
                .heap_size(owner_b, PROCESS_HEAP_HANDLE, 0, ptr_b)
                .is_none()
        );
    }

    #[test]
    fn process_exit_retires_only_its_typed_payloads_and_guest_pointer_caches() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let owner_a = kernel.current_process_birth_key().expect("boot birth");
        let hwnd_a = kernel.create_window_ex_w(1, "SURVIVING_A", "", None, 0, 0, 0);
        let launch_b = kernel.queue_process_launch(None, None);
        let owner_b = kernel
            .handles
            .process_birth_key(launch_b.process_id)
            .expect("child birth");
        kernel.set_current_process_id(launch_b.process_id);
        let hwnd_b = kernel.create_window_ex_w(launch_b.thread_id, "EXITING_B", "", None, 0, 0, 0);
        kernel.set_current_process_id(1);
        let position = |hwnd| WindowPos {
            hwnd,
            insert_after: HWND_TOP,
            x: 0,
            y: 0,
            width: 10,
            height: 10,
            flags: 0,
        };
        let (_, ptr_a) = kernel
            .queue_window_pos_payload(hwnd_a, position(hwnd_a))
            .unwrap();
        let (_, ptr_b) = kernel
            .queue_window_pos_payload(hwnd_b, position(hwnd_b))
            .unwrap();
        assert_eq!(ptr_a, ptr_b);
        assert!(kernel.post_window_typed_payload_message(
            hwnd_a,
            WM_WINDOWPOSCHANGED,
            0,
            ptr_a,
            owner_a
        ));
        assert!(kernel.post_window_typed_payload_message(
            hwnd_b,
            WM_WINDOWPOSCHANGED,
            0,
            ptr_b,
            owner_b
        ));
        kernel.set_process_name_guest_ptr_for(owner_a, 0x1111);
        kernel.set_process_name_guest_ptr_for(owner_b, 0x2222);
        kernel.set_command_line_guest_ptr_for(owner_a, 0x3333);
        kernel.set_command_line_guest_ptr_for(owner_b, 0x4444);
        kernel.set_module_heap_info_guest_ptr_for(owner_a, 0x5555);
        kernel.set_module_heap_info_guest_ptr_for(owner_b, 0x6666);
        kernel.mark_process_launch_exited(&launch_b, 0);
        assert!(kernel.gwe.message_pointer_payload(owner_b, ptr_b).is_none());
        assert!(kernel.gwe.message_pointer_payload(owner_a, ptr_a).is_some());
        assert_eq!(kernel.process_name_guest_ptr_for(owner_a), Some(0x1111));
        assert_eq!(kernel.process_name_guest_ptr_for(owner_b), None);
        assert_eq!(kernel.command_line_guest_ptr_for(owner_a), Some(0x3333));
        assert_eq!(kernel.command_line_guest_ptr_for(owner_b), None);
        assert_eq!(kernel.module_heap_info_guest_ptr_for(owner_a), Some(0x5555));
        assert_eq!(kernel.module_heap_info_guest_ptr_for(owner_b), None);
    }

    #[test]
    fn delivered_typed_message_destroy_is_exact_birth_even_at_same_guest_va() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let owner_a = kernel.current_process_birth_key().unwrap();
        let hwnd_a = kernel.create_window_ex_w(11, "DELIVERED_A", "", None, 0, WS_VISIBLE, 0);
        let launch_b = kernel.queue_process_launch(None, None);
        let owner_b = kernel
            .handles
            .process_birth_key(launch_b.process_id)
            .unwrap();
        kernel.set_current_process_id(launch_b.process_id);
        let hwnd_b = kernel.create_window_ex_w(22, "DELIVERED_B", "", None, 0, WS_VISIBLE, 0);
        kernel.set_current_process_id(1);
        let position = |hwnd| WindowPos {
            hwnd,
            insert_after: HWND_TOP,
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            flags: 0,
        };
        let (_, ptr_a) = kernel
            .queue_window_pos_payload(hwnd_a, position(hwnd_a))
            .unwrap();
        let (_, ptr_b) = kernel
            .queue_window_pos_payload(hwnd_b, position(hwnd_b))
            .unwrap();
        assert_eq!(ptr_a, ptr_b);
        assert!(kernel.post_window_typed_payload_message(
            hwnd_a,
            WM_WINDOWPOSCHANGED,
            0,
            ptr_a,
            owner_a
        ));
        assert!(kernel.post_window_typed_payload_message(
            hwnd_b,
            WM_WINDOWPOSCHANGED,
            0,
            ptr_b,
            owner_b
        ));
        let message_a = kernel
            .take_ready_visible_message_w_filtered(
                11,
                Some(hwnd_a),
                WM_WINDOWPOSCHANGED,
                WM_WINDOWPOSCHANGED,
            )
            .unwrap();
        let message_b = kernel
            .take_ready_visible_message_w_filtered(
                22,
                Some(hwnd_b),
                WM_WINDOWPOSCHANGED,
                WM_WINDOWPOSCHANGED,
            )
            .unwrap();
        kernel.record_delivered_typed_payload_message(owner_a, 11, 0x2000, &message_a);
        kernel.record_delivered_typed_payload_message(owner_b, 22, 0x2000, &message_b);
        let raw_a = Message::new(hwnd_a, WM_WINDOWPOSCHANGED, 0, ptr_a, 0);
        assert_eq!(
            kernel.delivered_typed_payload_owner_for_message(owner_a, 11, 0x2000, &raw_a),
            Some(owner_a)
        );
        assert_eq!(
            kernel.delivered_typed_payload_owner_for_message(owner_b, 22, 0x2000, &raw_a),
            None
        );

        assert!(kernel.destroy_window(hwnd_a));
        assert!(kernel.message_pointer_payload(owner_a, ptr_a).is_none());
        assert!(kernel.message_pointer_payload(owner_b, ptr_b).is_some());
        assert_eq!(
            kernel.delivered_typed_payload_owner_for_message(owner_a, 11, 0x2000, &raw_a),
            None
        );
        let mut stale_a = raw_a;
        kernel.reattach_delivered_typed_payload_message(owner_a, 11, 0x2000, &mut stale_a);
        assert_eq!(stale_a.typed_payload_owner, None);
        assert_eq!(kernel.dispatch_message_w_for_thread(11, message_a), 0);
        assert!(kernel.message_pointer_payload(owner_b, ptr_b).is_some());

        let mut ready_b = Message::new(hwnd_b, WM_WINDOWPOSCHANGED, 0, ptr_b, 0);
        kernel.reattach_delivered_typed_payload_message(owner_b, 22, 0x2000, &mut ready_b);
        assert_eq!(ready_b.typed_payload_owner, Some(owner_b));
        assert_eq!(kernel.dispatch_message_w_for_thread(22, ready_b), 0);
        assert!(kernel.message_pointer_payload(owner_b, ptr_b).is_none());
    }

    #[test]
    fn process_text_pointer_cache_survives_a_b_a_loader_handoff() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let owner_a = kernel.current_process_birth_key().unwrap();
        let launch_b = kernel.queue_process_launch(None, None);
        let owner_b = kernel
            .handles
            .process_birth_key(launch_b.process_id)
            .unwrap();
        kernel.set_process_module_path("\\A\\parent.exe");
        kernel.set_process_command_line("parent args");
        kernel.set_process_name_guest_ptr_for(owner_a, 0x1000);
        kernel.set_command_line_guest_ptr_for(owner_a, 0x2000);
        kernel.set_process_module_path("\\B\\child.exe");
        kernel.set_process_command_line("child args");
        kernel.set_current_process_id(launch_b.process_id);
        assert_eq!(kernel.process_name_guest_ptr_for(owner_a), None);
        assert_eq!(kernel.command_line_guest_ptr_for(owner_a), None);
        kernel.set_process_name_guest_ptr_for(owner_b, 0x3000);
        kernel.set_command_line_guest_ptr_for(owner_b, 0x4000);
        kernel.set_process_module_path("\\A\\parent.exe");
        kernel.set_process_command_line("parent args");
        kernel.set_current_process_id(1);
        assert_eq!(kernel.process_name_guest_ptr_for(owner_a), Some(0x1000));
        assert_eq!(kernel.command_line_guest_ptr_for(owner_a), Some(0x2000));
        assert_eq!(kernel.process_name_guest_ptr_for(owner_b), None);
        assert_eq!(kernel.command_line_guest_ptr_for(owner_b), None);
    }

    #[test]
    fn overwritten_peek_msg_slot_does_not_free_still_queued_typed_payload() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let owner = kernel.current_process_birth_key().unwrap();
        let hwnd = kernel.create_window_ex_w(11, "PEEK_TYPED", "", None, 0, WS_VISIBLE, 0);
        let (_, ptr) = kernel
            .queue_window_pos_payload(
                hwnd,
                WindowPos {
                    hwnd,
                    insert_after: HWND_TOP,
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 1,
                    flags: 0,
                },
            )
            .unwrap();
        assert!(kernel.post_window_typed_payload_message(hwnd, WM_WINDOWPOSCHANGED, 0, ptr, owner));
        let peeked = kernel
            .peek_ready_visible_message_w_filtered(
                11,
                Some(hwnd),
                WM_WINDOWPOSCHANGED,
                WM_WINDOWPOSCHANGED,
            )
            .unwrap();
        kernel.record_delivered_typed_payload_message(owner, 11, 0x2000, &peeked);
        kernel.record_delivered_typed_payload_message(
            owner,
            11,
            0x2000,
            &Message::new(hwnd, crate::ce::gwe::WM_USER + 1, 0, ptr, 0),
        );
        assert!(kernel.message_pointer_payload(owner, ptr).is_some());
        let taken = kernel
            .take_ready_visible_message_w_filtered(
                11,
                Some(hwnd),
                WM_WINDOWPOSCHANGED,
                WM_WINDOWPOSCHANGED,
            )
            .unwrap();
        kernel.record_delivered_typed_payload_message(owner, 11, 0x2000, &taken);
        let mut dispatched = Message::new(hwnd, WM_WINDOWPOSCHANGED, 0, ptr, 0);
        kernel.reattach_delivered_typed_payload_message(owner, 11, 0x2000, &mut dispatched);
        assert_eq!(dispatched.typed_payload_owner, Some(owner));
        assert_eq!(kernel.dispatch_message_w_for_thread(11, dispatched), 0);
        assert!(kernel.message_pointer_payload(owner, ptr).is_none());
    }

    #[test]
    fn caller_eligibility_excludes_expired_wait_before_timeout_selection() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (excluded_handle, excluded_thread) = kernel.create_guest_thread(0x0040_1000, 0, false);
        let (eligible_handle, eligible_thread) = kernel.create_guest_thread(0x0040_2000, 0, false);
        let now = kernel.timers.tick_count();
        let excluded_id = kernel.register_blocked_waiter(
            excluded_thread,
            excluded_handle,
            Vec::new(),
            SchedulerBlockedWaitKind::Kernel,
            now,
            0,
        );
        let eligible_id = kernel.register_blocked_waiter(
            eligible_thread,
            eligible_handle,
            Vec::new(),
            SchedulerBlockedWaitKind::Kernel,
            now,
            0,
        );
        assert_eq!(
            kernel.select_ready_blocked_waiter(0, now, |_, _| false),
            Some(excluded_id),
            "an expired wait is selectable even when is_ready is false"
        );
        assert_eq!(
            kernel.select_ready_blocked_waiter_with_eligibility(
                0,
                now,
                |wait, _| wait.id != excluded_id,
                |_, _| false,
            ),
            Some(eligible_id),
            "caller eligibility must apply before the independent timeout reason"
        );
        assert!(kernel.blocked_waiter(excluded_id).is_some());
        assert!(kernel.blocked_waiter(eligible_id).is_some());
    }

    #[test]
    fn finite_sleep_kernel_frame_selects_once_and_retires_after_completion() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (handle, thread_id) = kernel.create_guest_thread(0x0040_1000, 0, false);
        let birth = kernel.handles.thread_birth_key_for_handle(handle).unwrap();
        let frame = kernel
            .try_publish_finite_sleep_wait_for_birth(birth, handle, 100, 10)
            .unwrap();
        assert!(kernel.is_blocked_finite_sleep_frame(frame));
        assert!(
            kernel
                .try_publish_finite_sleep_wait_for_birth(birth, handle, 101, 10)
                .is_err(),
            "one birth cannot publish a second active finite-Sleep frame"
        );
        assert!(kernel.close_handle(handle).unwrap());
        assert_eq!(
            kernel.select_ready_blocked_waiter(1, 111, |_, _| false),
            Some(frame.wait_id())
        );
        assert!(kernel.begin_finite_sleep_completion(frame));
        assert!(!kernel.begin_finite_sleep_completion(frame));
        assert_eq!(
            kernel.select_ready_blocked_waiter(1, 111, |_, _| false),
            None
        );
        assert!(kernel.retire_finite_sleep_completion(frame));
        assert!(kernel.blocked_waiter(frame.wait_id()).is_none());
        assert_eq!(kernel.handles.active_finite_sleep_frame(birth), None);
        assert!(kernel.handles.thread_execution_live_by_id(thread_id));
    }

    #[test]
    fn finite_sleep_cancel_and_exit_keep_neighbor_wait_identity() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (handle, _) = kernel.create_guest_thread(0x0040_1000, 0, false);
        let birth = kernel.handles.thread_birth_key_for_handle(handle).unwrap();
        let neighbor = kernel
            .try_register_opaque_waiter_for_birth(
                birth,
                handle,
                Vec::new(),
                SchedulerBlockedWaitKind::GetMessage {
                    hwnd: None,
                    min_msg: 0,
                    max_msg: 0,
                },
                100,
                crate::ce::timer::INFINITE,
            )
            .unwrap();
        let first = kernel
            .try_publish_finite_sleep_wait_for_birth(birth, handle, 100, 10)
            .unwrap();
        assert!(kernel.remove_blocked_waiter(first.wait_id()).is_some());
        assert!(kernel.blocked_waiter(neighbor).is_some());
        assert_eq!(kernel.handles.active_finite_sleep_frame(birth), None);

        let second = kernel
            .try_publish_finite_sleep_wait_for_birth(birth, handle, 200, 10)
            .unwrap();
        assert_ne!(first.key(), second.key());
        assert_ne!(first.wait_id(), second.wait_id());
        assert!(kernel.mark_guest_thread_exited_by_birth(birth, 0));
        assert!(kernel.blocked_waiter(neighbor).is_none());
        assert!(kernel.blocked_waiter(second.wait_id()).is_none());
        assert_eq!(kernel.handles.active_finite_sleep_frame(birth), None);
    }

    #[test]
    fn finite_sleep_send_keeps_frame_provenance_after_wake_until_queue_take() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (receiver_handle, receiver_tid) = kernel.create_guest_thread(0x0040_1000, 0, false);
        let (_, sender_tid) = kernel.create_guest_thread(0x0040_2000, 0, false);
        let receiver_birth = kernel
            .handles
            .thread_birth_key_for_handle(receiver_handle)
            .unwrap();
        let hwnd = kernel.create_window_ex_w(receiver_tid, "SLEEP_SEND_FRAME", "", None, 0, 0, 0);
        let frame = kernel
            .try_publish_finite_sleep_wait_for_birth(receiver_birth, receiver_handle, 100, 10)
            .unwrap();
        let send_id = kernel
            .begin_cross_thread_send_message_w(
                sender_tid,
                hwnd,
                crate::ce::gwe::WM_USER + 1,
                0,
                0,
                None,
            )
            .unwrap();
        assert_eq!(
            kernel
                .gwe
                .sent_message(send_id)
                .unwrap()
                .deferred_from_sleep_frame,
            Some(frame)
        );
        assert!(kernel.thread_has_pending_sent_message(receiver_tid));
        assert!(kernel.begin_finite_sleep_completion(frame));
        assert!(kernel.retire_finite_sleep_completion(frame));
        assert_eq!(
            kernel.handles.active_finite_sleep_frame(receiver_birth),
            None
        );
        assert_eq!(
            kernel
                .gwe
                .sent_message(send_id)
                .unwrap()
                .deferred_from_sleep_frame,
            Some(frame),
            "retiring Sleep must not turn a deferred send into a pre-run callout"
        );
        assert_eq!(kernel.gwe.active_sent_message_id(receiver_tid), None);
        assert_eq!(
            kernel
                .take_ready_sent_message_w_filtered(receiver_tid, None, 0, 0)
                .unwrap()
                .hwnd,
            hwnd
        );
        assert_eq!(
            kernel.gwe.active_sent_message_id(receiver_tid),
            Some(send_id)
        );
        assert_eq!(
            kernel.complete_active_sent_message(receiver_tid, 0x77),
            Some(send_id)
        );
        assert_eq!(
            kernel.take_completed_send_message_result(send_id),
            Some(0x77)
        );
    }

    #[test]
    fn finite_sleep_receiver_exit_replies_to_only_its_deferred_send() {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (receiver_handle, receiver_tid) = kernel.create_guest_thread(0x0040_1000, 0, false);
        let (_, sender_tid) = kernel.create_guest_thread(0x0040_2000, 0, false);
        let (_, neighbor_tid) = kernel.create_guest_thread(0x0040_3000, 0, false);
        let receiver_birth = kernel
            .handles
            .thread_birth_key_for_handle(receiver_handle)
            .unwrap();
        let hwnd = kernel.create_window_ex_w(receiver_tid, "SLEEP_EXIT_SEND", "", None, 0, 0, 0);
        let neighbor_hwnd =
            kernel.create_window_ex_w(neighbor_tid, "NEIGHBOR_SEND", "", None, 0, 0, 0);
        let frame = kernel
            .try_publish_finite_sleep_wait_for_birth(receiver_birth, receiver_handle, 100, 10)
            .unwrap();
        let deferred_send = kernel
            .begin_cross_thread_send_message_w(
                sender_tid,
                hwnd,
                crate::ce::gwe::WM_USER + 1,
                0,
                0,
                None,
            )
            .unwrap();
        let neighbor_send = kernel
            .begin_cross_thread_send_message_w(
                sender_tid,
                neighbor_hwnd,
                crate::ce::gwe::WM_USER + 2,
                0,
                0,
                None,
            )
            .unwrap();
        assert_eq!(
            kernel
                .gwe
                .sent_message(deferred_send)
                .unwrap()
                .deferred_from_sleep_frame,
            Some(frame)
        );
        assert!(kernel.mark_guest_thread_exited_by_birth(receiver_birth, 0));
        assert!(kernel.blocked_waiter(frame.wait_id()).is_none());
        assert!(!kernel.thread_has_pending_sent_message(receiver_tid));
        assert_eq!(
            kernel.take_completed_send_message_result(deferred_send),
            Some(0)
        );
        assert!(kernel.thread_has_pending_sent_message(neighbor_tid));
        assert!(!kernel.sent_message_result_ready(neighbor_send));
        assert!(kernel.gwe.sent_message(neighbor_send).is_some());
    }

    #[test]
    fn finite_sleep_send_rejects_stale_birth_window_after_tid_reuse() -> Result<()> {
        let (mut kernel, receiver_handle, receiver_birth, old_hwnd, live_hwnd) =
            stale_window_remote_input_fixture()?;
        let receiver_tid = receiver_birth.thread_id();
        let (_, sender_tid) = kernel.create_guest_thread(0x0040_3000, 0, false);
        let frame = kernel
            .try_publish_finite_sleep_wait_for_birth(receiver_birth, receiver_handle, 100, 10)
            .expect("new incarnation finite Sleep frame");
        assert!(
            kernel
                .gwe
                .sent_message_ids_for_windows(&[old_hwnd, live_hwnd])
                .is_empty()
        );
        assert_eq!(
            kernel.send_message_w(old_hwnd, crate::ce::gwe::WM_USER, 0, 0),
            None,
            "raw send must also reject the old birth-owned HWND",
        );

        assert_eq!(
            kernel.begin_cross_thread_send_message_w(
                sender_tid,
                old_hwnd,
                crate::ce::gwe::WM_USER + 1,
                0,
                0,
                None,
            ),
            None,
            "an old birth-owned HWND must not borrow the reused TID's Sleep frame",
        );
        assert!(
            kernel
                .gwe
                .sent_message_ids_for_windows(&[old_hwnd])
                .is_empty()
        );
        assert_eq!(
            kernel.handles.active_finite_sleep_frame(receiver_birth),
            Some(frame)
        );
        assert!(kernel.blocked_waiter(frame.wait_id()).is_some());

        let live_send = kernel
            .begin_cross_thread_send_message_w(
                sender_tid,
                live_hwnd,
                crate::ce::gwe::WM_USER + 2,
                0,
                0,
                None,
            )
            .expect("live birth-owned HWND must still accept a send");
        assert_eq!(
            kernel
                .gwe
                .sent_message(live_send)
                .and_then(|send| send.deferred_from_sleep_frame),
            Some(frame),
        );
        assert!(kernel.thread_has_pending_sent_message(receiver_tid));
        Ok(())
    }

    #[test]
    fn ordinary_queued_send_fails_once_on_receiver_exit_before_tid_reuse() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (receiver_handle, receiver_tid) = kernel.create_guest_thread(0x0040_1000, 0, false);
        let receiver_birth = kernel
            .handles
            .thread_birth_key_for_handle(receiver_handle)
            .expect("B's exact thread birth");
        let (_, sender_tid) = kernel.create_guest_thread(0x0040_2000, 0, false);
        let hwnd = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                receiver_birth,
                "QUEUED_SEND_BIRTH_B",
                "B",
                None,
                None,
                0,
                0,
                0,
                Rect::default(),
            )
            .expect("B's exact-birth HWND");
        assert!(matches!(
            kernel.gwe.window(hwnd).unwrap().delivery_owner(),
            WindowDeliveryOwner::Birth { thread, .. } if thread == receiver_birth
        ));

        let message_id = crate::ce::gwe::WM_USER + 0x9b;
        let send_id = kernel
            .begin_cross_thread_send_message_w(sender_tid, hwnd, message_id, 0, 0, None)
            .expect("A's ordinary queued SendMessage to B");
        assert_eq!(
            kernel
                .gwe
                .sent_message(send_id)
                .and_then(|send| send.deferred_from_sleep_frame),
            None,
            "this is not the finite-Sleep deferred-send case"
        );
        assert_eq!(
            kernel.gwe.sent_message(send_id).unwrap().receiver_birth,
            Some(receiver_birth),
            "the ordinary send freezes B's HWND birth when queued"
        );
        assert!(kernel.thread_has_pending_sent_message(receiver_tid));
        assert!(!kernel.sent_message_result_ready(send_id));

        assert!(kernel.mark_guest_thread_exited_by_birth(receiver_birth, 0));
        assert!(!kernel.thread_has_pending_sent_message(receiver_tid));
        assert!(kernel.close_handle(receiver_handle)?);
        assert!(!kernel.handles.contains_thread_birth(receiver_birth));
        let replacement_process = kernel.queue_process_launch(None, None);
        let replacement_owner = kernel
            .handles
            .process_birth_key(replacement_process.process_id)
            .expect("replacement process birth");
        let replacement_handle = kernel.handles.try_create_thread_in_birth(
            replacement_owner,
            receiver_tid,
            0x0040_3000,
            0,
            false,
        )?;
        let replacement_birth = kernel
            .handles
            .thread_birth_key_for_handle(replacement_handle)
            .expect("B-prime's exact thread birth");
        assert_ne!(receiver_birth, replacement_birth);
        assert_eq!(replacement_birth.thread_id(), receiver_tid);
        assert_eq!(
            kernel.live_thread_birth_key_by_id(receiver_tid),
            Some(replacement_birth),
            "B-prime really occupies B's numeric TID"
        );
        assert!(!kernel.thread_has_pending_sent_message(receiver_tid));

        let result_ready_after_exit = kernel.sent_message_result_ready(send_id);
        let replacement_delivery = kernel
            .get_message_w_filtered(receiver_tid, None, 0, 0)
            .map(|message| (message.hwnd, message.msg));
        let sender_result = kernel.take_completed_send_message_result(send_id);
        assert!(
            replacement_delivery.is_none() && result_ready_after_exit && sender_result == Some(0),
            "B-prime GetMessage={replacement_delivery:?}; A result-ready after B exit={result_ready_after_exit}; A one failure result={sender_result:?}"
        );
        assert_eq!(
            kernel.take_completed_send_message_result(send_id),
            None,
            "A must not receive the failure result twice"
        );
        Ok(())
    }

    #[test]
    fn exact_birth_send_completion_rejects_ancestor_id_while_inner_active() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (receiver_handle, receiver_tid) = kernel.create_guest_thread(0x0040_1000, 0, false);
        let receiver_birth = kernel
            .handles
            .thread_birth_key_for_handle(receiver_handle)
            .expect("receiver birth");
        let (_, sender_a) = kernel.create_guest_thread(0x0040_2000, 0, false);
        let (_, sender_b) = kernel.create_guest_thread(0x0040_3000, 0, false);
        let hwnd = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                receiver_birth,
                "EXACT_SEND_COMPLETION",
                "receiver",
                None,
                None,
                0,
                0,
                0,
                Rect::default(),
            )
            .expect("receiver HWND");
        let outer_id = kernel
            .begin_cross_thread_send_message_w(
                sender_a,
                hwnd,
                crate::ce::gwe::WM_USER + 0x9c,
                0,
                0,
                None,
            )
            .expect("outer send");
        let inner_id = kernel
            .begin_cross_thread_send_message_w(
                sender_b,
                hwnd,
                crate::ce::gwe::WM_USER + 0x9d,
                0,
                0,
                None,
            )
            .expect("inner send");
        assert!(
            kernel
                .gwe
                .activate_sent_message_for_receiver_birth(receiver_birth, outer_id)
        );
        assert!(
            kernel
                .gwe
                .activate_sent_message_for_receiver_birth(receiver_birth, inner_id)
        );
        assert_eq!(
            kernel.gwe.active_sent_message_id(receiver_tid),
            Some(inner_id)
        );
        let completed_before = kernel.gwe.stats().send_transaction_completed_count;
        assert_eq!(
            kernel.complete_sent_message_for_birth(receiver_birth, outer_id, 0x1111),
            None,
            "an older send ID cannot complete the active inner callback"
        );
        assert_eq!(
            kernel.gwe.active_sent_message_id(receiver_tid),
            Some(inner_id)
        );
        assert_eq!(
            kernel.gwe.stats().send_transaction_completed_count,
            completed_before
        );
        assert!(!kernel.sent_message_result_ready(outer_id));
        assert!(!kernel.sent_message_result_ready(inner_id));
        assert_eq!(
            kernel.complete_sent_message_for_birth(receiver_birth, inner_id, 0x2222),
            Some(inner_id)
        );
        assert_eq!(
            kernel.gwe.active_sent_message_id(receiver_tid),
            Some(outer_id)
        );
        assert_eq!(
            kernel.complete_sent_message_for_birth(receiver_birth, outer_id, 0x1111),
            Some(outer_id)
        );
        assert_eq!(
            kernel.take_completed_send_message_result(inner_id),
            Some(0x2222)
        );
        assert_eq!(
            kernel.take_completed_send_message_result(outer_id),
            Some(0x1111)
        );
        assert_eq!(kernel.gwe.active_sent_message_id(receiver_tid), None);
        Ok(())
    }

    #[test]
    fn legacy_exact_send_completion_preserves_nested_direct_and_outer_callbacks() -> Result<()> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (_, receiver_tid) = kernel.create_guest_thread(0x0040_1000, 0, false);
        let receiver_birth = kernel
            .live_thread_birth_key_by_id(receiver_tid)
            .expect("receiver birth");
        let (_, sender_a) = kernel.create_guest_thread(0x0040_2000, 0, false);
        let (_, sender_b) = kernel.create_guest_thread(0x0040_3000, 0, false);
        let hwnd = kernel
            .gwe
            .create_window(receiver_tid, "LEGACY_EXACT_SEND", "B");
        let outer_id = kernel
            .begin_cross_thread_send_message_w(
                sender_a,
                hwnd,
                crate::ce::gwe::WM_USER + 0xa1,
                0,
                0,
                None,
            )
            .expect("outer legacy send");
        let inner_id = kernel
            .begin_cross_thread_send_message_w(
                sender_b,
                hwnd,
                crate::ce::gwe::WM_USER + 0xa2,
                0,
                0,
                None,
            )
            .expect("inner legacy send");
        assert_eq!(
            kernel.gwe.sent_message(outer_id).unwrap().receiver_birth,
            None
        );
        assert_eq!(
            kernel.gwe.sent_message(inner_id).unwrap().receiver_birth,
            None
        );
        assert!(
            kernel
                .gwe
                .activate_sent_message_for_receiver(receiver_tid, outer_id)
        );
        assert!(
            kernel
                .gwe
                .activate_sent_message_for_receiver(receiver_tid, inner_id)
        );
        assert!(kernel.reply_message(receiver_tid, 0x2222));
        assert_eq!(
            kernel.take_completed_send_message_result(inner_id),
            Some(0x2222)
        );

        kernel.gwe.begin_send_message(receiver_tid);
        assert_eq!(
            kernel.complete_legacy_sent_message_for_id(receiver_tid, outer_id, 0xbad0),
            None,
            "the active inner callback cannot complete an outer send ID"
        );
        kernel.gwe.end_direct_send_message(receiver_tid);
        assert_eq!(
            kernel.gwe.active_sent_message_id(receiver_tid),
            Some(inner_id)
        );
        assert_eq!(
            kernel.complete_legacy_sent_message_for_id(receiver_tid, inner_id, 0xbad1),
            Some(inner_id)
        );
        assert_eq!(
            kernel.gwe.active_sent_message_id(receiver_tid),
            Some(outer_id)
        );
        assert!(kernel.gwe.in_send_message(receiver_tid));
        assert_eq!(
            kernel.complete_legacy_sent_message_for_id(receiver_tid, inner_id, 0xbad2),
            None,
            "a duplicate inner return cannot pop the outer callback"
        );
        assert_eq!(
            kernel.gwe.active_sent_message_id(receiver_tid),
            Some(outer_id)
        );
        assert_eq!(
            kernel.complete_sent_message_for_birth(receiver_birth, outer_id, 0xbad3),
            None,
            "birth-owned completion cannot claim an explicit legacy HWND"
        );
        assert_eq!(
            kernel.complete_legacy_sent_message_for_id(receiver_tid, outer_id, 0x1111),
            Some(outer_id)
        );
        assert_eq!(kernel.gwe.active_sent_message_id(receiver_tid), None);
        assert_eq!(kernel.take_completed_send_message_result(inner_id), None);
        assert_eq!(
            kernel.take_completed_send_message_result(outer_id),
            Some(0x1111)
        );
        Ok(())
    }

    fn consumed_reply_with_active_birth_callback()
    -> Result<(CeKernel, u32, ThreadBirthKey, u32, u64)> {
        let mut kernel = CeKernel::boot(runtime_config_with_com7());
        let (receiver_handle, receiver_tid) = kernel.create_guest_thread(0x0040_1000, 0, false);
        let receiver_birth = kernel
            .handles
            .thread_birth_key_for_handle(receiver_handle)
            .expect("receiver birth");
        let (_, sender_tid) = kernel.create_guest_thread(0x0040_2000, 0, false);
        let hwnd = kernel
            .create_window_ex_w_with_parent_owner_and_rect_for_birth(
                receiver_birth,
                "EARLY_REPLY_RECEIVER",
                "B",
                None,
                None,
                0,
                0,
                0,
                Rect::default(),
            )
            .expect("birth-owned HWND");
        let send_id = kernel
            .begin_cross_thread_send_message_w(
                sender_tid,
                hwnd,
                crate::ce::gwe::WM_USER + 0x9e,
                0,
                0,
                None,
            )
            .expect("queued send");
        assert!(
            kernel
                .gwe
                .activate_sent_message_for_receiver_birth(receiver_birth, send_id)
        );
        assert!(kernel.reply_message(receiver_tid, 0x55aa));
        assert_eq!(
            kernel.take_completed_send_message_result(send_id),
            Some(0x55aa)
        );
        assert_eq!(kernel.take_completed_send_message_result(send_id), None);
        assert_eq!(
            kernel.gwe.active_sent_message_id(receiver_tid),
            Some(send_id)
        );
        assert!(!kernel.gwe.in_send_message(receiver_tid));
        Ok((
            kernel,
            receiver_handle,
            receiver_birth,
            receiver_tid,
            send_id,
        ))
    }

    #[test]
    fn consumed_reply_keeps_exact_birth_callback_completion() -> Result<()> {
        let (mut kernel, _, receiver_birth, receiver_tid, send_id) =
            consumed_reply_with_active_birth_callback()?;
        let completed_before = kernel.gwe.stats().send_transaction_completed_count;
        assert_eq!(
            kernel.complete_sent_message_for_birth(receiver_birth, send_id, 0x7788),
            Some(send_id),
            "phase=callback-return: consumed ReplyMessage result must not erase active send identity"
        );
        assert_eq!(kernel.gwe.active_sent_message_id(receiver_tid), None);
        assert_eq!(
            kernel.gwe.stats().send_transaction_completed_count,
            completed_before,
            "callback return cannot issue a second reply"
        );
        assert_eq!(kernel.take_completed_send_message_result(send_id), None);
        Ok(())
    }

    #[test]
    fn consumed_reply_receiver_exit_cleans_active_id_before_tid_reuse() -> Result<()> {
        let (mut kernel, receiver_handle, receiver_birth, receiver_tid, send_id) =
            consumed_reply_with_active_birth_callback()?;
        let terminated_before = kernel
            .gwe
            .stats()
            .send_transaction_receiver_terminated_count;
        assert!(kernel.mark_guest_thread_exited_by_birth(receiver_birth, 0));
        assert_eq!(
            kernel.gwe.active_sent_message_id(receiver_tid),
            None,
            "phase=receiver-exit: consumed ReplyMessage result left an orphan active send ID"
        );
        assert!(!kernel.gwe.in_send_message(receiver_tid));
        assert_eq!(
            kernel
                .gwe
                .stats()
                .send_transaction_receiver_terminated_count,
            terminated_before,
            "an already consumed reply needs no failure result"
        );
        assert!(kernel.close_handle(receiver_handle)?);
        let replacement_process = kernel.queue_process_launch(None, None);
        let replacement_owner = kernel
            .handles
            .process_birth_key(replacement_process.process_id)
            .expect("replacement process birth");
        let replacement_handle = kernel.handles.try_create_thread_in_birth(
            replacement_owner,
            receiver_tid,
            0x0040_3000,
            0,
            false,
        )?;
        let replacement_birth = kernel
            .handles
            .thread_birth_key_for_handle(replacement_handle)
            .expect("replacement birth");
        assert_ne!(receiver_birth, replacement_birth);
        assert_eq!(
            kernel.live_thread_birth_key_by_id(receiver_tid),
            Some(replacement_birth)
        );
        assert_eq!(kernel.gwe.active_sent_message_id(receiver_tid), None);
        assert!(!kernel.gwe.in_send_message(receiver_tid));
        assert_eq!(kernel.take_completed_send_message_result(send_id), None);
        Ok(())
    }
}
