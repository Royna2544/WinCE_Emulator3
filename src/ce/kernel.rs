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
            WM_WINDOWPOSCHANGED, WindowPos,
        },
        memory::{MemorySystem, PROCESS_HEAP_HANDLE},
        object::{
            CE_THREAD_PRIORITY_NORMAL, DeviceNotificationObject, FileChangeNotificationObject,
            FileChangeRecord, FileObject, FindFileObject, HandleTable, KernelObject,
            MAX_SUSPEND_COUNT, MessageQueueHandleObject, ThreadResumeResult, ThreadSuspendResult,
            VolumeObject, WaitMultipleResult, WaitResult, ce_thread_priority_to_win32,
            win32_thread_priority_to_ce,
        },
        registry::{Registry, RegistryValue},
        remote::{CeRemote, RemoteStatus, WM_LBUTTONDOWN, WM_MOUSEMOVE, make_lparam},
        resource::ResourceSystem,
        scheduler::{
            Scheduler, SchedulerBlockedWait, SchedulerBlockedWaitKind, SchedulerStats,
            SchedulerWaitKind, SchedulerWakeReason,
        },
        shell::{
            NotifyIconData, ShellChangeNotifyRegistration, ShellNotificationCallbackMethod,
            ShellNotificationCallbackRecord, ShellNotificationRecord, ShellSystem,
        },
        thread::{ERROR_INVALID_PARAMETER, ERROR_NOT_SUPPORTED, ERROR_SUCCESS, ThreadSystem},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessagePumpResult {
    Dispatched(u32),
    Quit(u32),
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RemoteTouchTargeting {
    Explicit,
    ThreadHitTest,
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
    pub remote_server: Option<RemoteServer>,
    process_module_base: u32,
    process_module_path: String,
    process_module_host_path: Option<PathBuf>,
    process_command_line: String,
    command_line_guest_ptr: u32,
    process_current_directory: Option<String>,
    process_show_cmd: u32,
    current_process_id: u32,
    current_process_exit_code: u32,
    current_process_signaled: bool,
    thread_priority_overrides: BTreeMap<u32, i32>,
    thread_suspend_counts: BTreeMap<u32, u32>,
    pending_process_launches: Vec<PendingProcessLaunch>,
    next_process_id: u32,
    loaded_modules: BTreeMap<String, LoadedModule>,
    next_loaded_module_order: u64,
    next_font_mem_resource_handle: u32,
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
    fsdmgr_fmd_sector_sizes: BTreeMap<u32, u32>,
    fsdmgr_fmd_region_tables: BTreeMap<u32, Vec<[u32; 7]>>,
    fsdmgr_fmd_reserved_regions: BTreeMap<(u32, [u8; 8]), Vec<u8>>,
    fsdmgr_fmd_interface_disk: Option<u32>,
    fsdmgr_volume_locks: BTreeMap<u32, FsdmgrVolumeLock>,
    next_fsdmgr_volume_lock: u32,
    modal_dialog_results: BTreeMap<(u32, u32), u32>,
    live_pump_timeout_stop_tick: Option<u32>,
    runtime_loader_stats: RuntimeLoaderStats,
    pulsed_wait_handles: BTreeMap<u64, u32>,
    comm_event_mask_changed_waits: BTreeSet<u64>,
    font_families: Vec<CeFontFamily>,
    fsdmgr_caches: Vec<Option<FsdmgrCacheEntry>>,
    display_gamma_value: u32,
    display_rotation: u32,
    display_lcdcon3_high_nibble: u8,
    display_perf_timings: Vec<DisplayPerfTiming>,
    display_perf_unhandled: u32,
    window_backing_stores: BTreeMap<u32, FramebufferBackingStore>,
}

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentProcessState {
    pub process_id: u32,
    pub exit_code: u32,
    pub signaled: bool,
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
const EVENT_TRACE_LIMIT: usize = 256;
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
    for (index, candidate) in candidates.into_iter().enumerate() {
        registry.set_value(
            &path,
            &format!("__testime_sample_{index:03}"),
            RegistryValue::string(candidate),
        );
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

impl CeKernel {
    const MSGQUEUE_MSGALERT: u32 = 0x0000_0001;
    const MSGQUEUE_NOPRECOMMIT: u32 = 0x0000_0001;
    const MSGQUEUE_ALLOW_BROKEN: u32 = 0x0000_0002;
    const MSGQUEUE_VALID_FLAGS: u32 = Self::MSGQUEUE_NOPRECOMMIT | Self::MSGQUEUE_ALLOW_BROKEN;
    const MSGQUEUE_MAX_NODE_SIZE: u32 = 0x0010_0000;
    const MSGQUEUE_MAX_NUM_NODES: u32 = 0x0010_0000;

    pub fn boot(config: RuntimeConfig) -> Self {
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
            memory: MemorySystem::default(),
            remote_server: None,
            process_module_base: 0,
            process_module_path: "\\FakeCE\\process.exe".to_owned(),
            process_module_host_path: None,
            process_command_line: String::new(),
            command_line_guest_ptr: 0,
            process_current_directory: None,
            process_show_cmd: SW_SHOWNORMAL,
            current_process_id: 1,
            current_process_exit_code: STILL_ACTIVE,
            current_process_signaled: false,
            thread_priority_overrides: BTreeMap::new(),
            thread_suspend_counts: BTreeMap::new(),
            pending_process_launches: Vec::new(),
            next_process_id: 0x42,
            loaded_modules: BTreeMap::new(),
            next_loaded_module_order: 0,
            next_font_mem_resource_handle: 0x5f00_0001,
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
            fsdmgr_fmd_sector_sizes: BTreeMap::new(),
            fsdmgr_fmd_region_tables: BTreeMap::new(),
            fsdmgr_fmd_reserved_regions: BTreeMap::new(),
            fsdmgr_fmd_interface_disk: None,
            fsdmgr_volume_locks: BTreeMap::new(),
            next_fsdmgr_volume_lock: 0x6d00_0001,
            modal_dialog_results: BTreeMap::new(),
            live_pump_timeout_stop_tick: None,
            runtime_loader_stats: RuntimeLoaderStats::default(),
            pulsed_wait_handles: BTreeMap::new(),
            comm_event_mask_changed_waits: BTreeSet::new(),
            font_families: ce_system_font_families(),
            fsdmgr_caches: Vec::new(),
            display_gamma_value: 2330,
            display_rotation: 0,
            display_lcdcon3_high_nibble: 0,
            display_perf_timings: Vec::new(),
            display_perf_unhandled: 0,
            window_backing_stores: BTreeMap::new(),
        };
        kernel.publish_configured_mount_device_interfaces(true);
        kernel
    }

    pub fn runtime_loader_stats(&self) -> RuntimeLoaderStats {
        self.runtime_loader_stats
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

    pub fn font_mem_resource_pseudo_handle(&mut self) -> u32 {
        let handle = self.next_font_mem_resource_handle;
        self.next_font_mem_resource_handle = self.next_font_mem_resource_handle.wrapping_add(1);
        handle
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
        let path = path.into();
        self.files.set_root_relative_guest_path(&path);
        self.process_module_path = path;
    }

    pub fn process_module_path(&self) -> &str {
        &self.process_module_path
    }

    pub fn set_process_module_host_path(&mut self, path: impl Into<PathBuf>) {
        self.process_module_host_path = Some(path.into());
    }

    pub fn process_module_host_path(&self) -> Option<&PathBuf> {
        self.process_module_host_path.as_ref()
    }

    pub fn set_process_command_line(&mut self, command_line: impl Into<String>) {
        self.process_command_line = command_line.into();
        self.command_line_guest_ptr = 0; // invalidate cached guest pointer
    }

    pub fn process_command_line(&self) -> &str {
        &self.process_command_line
    }

    /// Return (or lazily allocate) the guest-side command-line wide-string pointer.
    /// Allocates a LMEM block just large enough to hold the null-terminated UTF-16
    /// command line on first call.  Returns 0 only when the memory system is full.
    pub fn command_line_guest_ptr(&mut self) -> u32 {
        self.command_line_guest_ptr
    }

    pub fn set_command_line_guest_ptr(&mut self, ptr: u32) {
        self.command_line_guest_ptr = ptr;
    }

    pub fn set_process_current_directory(&mut self, directory: Option<String>) {
        self.process_current_directory = directory;
    }

    pub fn process_current_directory(&self) -> Option<&str> {
        self.process_current_directory.as_deref()
    }

    pub fn set_process_show_cmd(&mut self, show_cmd: u32) {
        self.process_show_cmd = show_cmd;
    }

    pub fn process_show_cmd(&self) -> u32 {
        self.process_show_cmd
    }

    pub fn set_current_process_id(&mut self, process_id: u32) {
        self.current_process_id = process_id;
    }

    pub fn current_process_id(&self) -> u32 {
        self.current_process_id
    }

    pub fn current_process_state(&self) -> CurrentProcessState {
        CurrentProcessState {
            process_id: self.current_process_id,
            exit_code: self.current_process_exit_code,
            signaled: self.current_process_signaled,
        }
    }

    pub fn set_current_process_state(&mut self, state: CurrentProcessState) {
        self.current_process_id = state.process_id;
        self.current_process_exit_code = state.exit_code;
        self.current_process_signaled = state.signaled;
    }

    pub fn reset_current_process_exit_state(&mut self) {
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
        self.next_process_id = self.next_process_id.saturating_add(1);
        let thread_id = self.threads.allocate_guest_thread_id();
        let process_handle = self.handles.create_process(process_id);
        let thread_handle = self.handles.create_thread(thread_id, 0, 0, false);
        let launch = PendingProcessLaunch {
            application,
            command_line,
            current_directory,
            show_cmd,
            process_handle,
            thread_handle,
            process_id,
            thread_id,
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

    pub fn mark_process_launch_exited(&mut self, launch: &PendingProcessLaunch, exit_code: u32) {
        self.mark_process_launch_exited_with_framebuffer(launch, exit_code, None);
    }

    pub fn mark_process_launch_exited_with_framebuffer(
        &mut self,
        launch: &PendingProcessLaunch,
        exit_code: u32,
        framebuffer: Option<&mut dyn Framebuffer>,
    ) {
        let backing_store_targets =
            self.process_window_backing_store_targets(launch.process_id, launch.thread_id);
        self.destroy_process_windows(launch.process_id, launch.thread_id);
        if let Some(framebuffer) = framebuffer {
            let _ = self.restore_window_backing_stores(&backing_store_targets, framebuffer);
        } else {
            self.discard_window_backing_stores(&backing_store_targets);
        }
        if self
            .handles
            .mark_process_exited(launch.process_handle, exit_code)
        {
            self.queue_object_wake_candidates(launch.process_handle);
        }
        if self
            .handles
            .mark_thread_exited(launch.thread_handle, exit_code)
        {
            self.queue_object_wake_candidates(launch.thread_handle);
        }
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
    }

    pub fn process_window_targets(&self, process_id: u32, thread_id: u32) -> Vec<u32> {
        self.gwe
            .windows_snapshot()
            .into_iter()
            .filter(|window| {
                !window.destroyed
                    && window.hwnd != crate::ce::gwe::DESKTOP_HWND
                    && (window.process_id == process_id || window.thread_id == thread_id)
            })
            .map(|window| window.hwnd)
            .collect()
    }

    fn process_window_backing_store_targets(&self, process_id: u32, thread_id: u32) -> Vec<u32> {
        self.gwe
            .windows_snapshot()
            .into_iter()
            .filter(|window| {
                window.hwnd != crate::ce::gwe::DESKTOP_HWND
                    && (window.process_id == process_id || window.thread_id == thread_id)
            })
            .map(|window| window.hwnd)
            .collect()
    }

    fn destroy_process_windows(&mut self, process_id: u32, thread_id: u32) {
        self.gwe.terminate_sent_messages_from_process(process_id);
        if thread_id != 0 {
            self.gwe.terminate_sent_messages_from_sender(thread_id);
        }
        let hwnds = self.process_window_targets(process_id, thread_id);
        for hwnd in &hwnds {
            self.record_window_lifecycle_trace(
                "destroy_window_begin",
                thread_id,
                Some(*hwnd),
                Some(1),
                Some(format!("reason=process_exit/process_id={process_id}")),
            );
            let _ = self.gwe.destroy_window(*hwnd, self.timers.tick_count());
            self.record_window_lifecycle_trace(
                "destroy_window_end",
                thread_id,
                Some(*hwnd),
                Some(1),
                Some(format!("reason=process_exit/process_id={process_id}")),
            );
        }
        if !hwnds.is_empty() {
            self.shell.remove_windows_state(&hwnds);
            self.timers.remove_window_timers(&hwnds);
        }
    }

    pub fn pump_timers_to_gwe(&mut self, thread_id: u32) {
        self.expire_timed_out_send_messages();
        self.expire_shell_notifications();
        for timer in self.timers.due_timers() {
            let target_thread_id = if timer.thread_id == 0 {
                thread_id
            } else {
                timer.thread_id
            };
            let message = crate::ce::gwe::Message::new(
                timer.hwnd.unwrap_or(0),
                timer.message,
                timer.id,
                timer.callback.unwrap_or(0),
                self.timers.tick_count(),
            );
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
            self.post_gwe_message(target_thread_id, message);
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
                let _ = writeln!(
                    out,
                    "    0x{handle:08x} {} kind={:?} backend={:?} host={} serial={} rx={} tx={}{}",
                    device.guest_name,
                    device.kind,
                    device.backend,
                    device.host.as_deref().unwrap_or("<none>"),
                    device.is_serial(),
                    rx,
                    tx,
                    remote_serial
                );
            }
        }
        out
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

    fn drain_remote_serial_to_handle(&mut self, handle: u32, max_bytes: usize) -> usize {
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
        count
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
        if let Ok(session) = self.devices.open(path) {
            let handle = self.handles.insert(KernelObject::Device(session));
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
        if notification.pending_signal_count == 0 {
            notification.pending.clear();
        }
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
        self.handles.close(handle)?;
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
        let handle = self
            .handles
            .create_event(name.clone(), manual_reset, initial_state);
        self.record_event_trace(EventTraceRecord {
            op: "CreateEventW",
            handle: Some(handle),
            name,
            manual_reset: Some(manual_reset),
            signaled: Some(initial_state),
            result: Some(handle != 0),
            detail: Some(self.describe_handle(handle)),
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

    pub fn create_guest_thread(
        &mut self,
        start_address: u32,
        parameter: u32,
        suspended: bool,
    ) -> (u32, u32) {
        let thread_id = self.threads.allocate_guest_thread_id();
        let handle = self
            .handles
            .create_thread(thread_id, start_address, parameter, suspended);
        (handle, thread_id)
    }

    pub fn mark_guest_thread_exited(&mut self, handle: u32, exit_code: u32) -> bool {
        let success = self.handles.mark_thread_exited(handle, exit_code);
        if success {
            self.queue_object_wake_candidates(handle);
        }
        success
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

    pub fn terminate_process(&mut self, handle: u32, exit_code: u32) -> bool {
        if Self::is_current_process_pseudo_handle(handle) {
            self.destroy_process_windows(self.current_process_id, 0);
            self.current_process_exit_code = exit_code;
            self.current_process_signaled = true;
            self.queue_object_wake_candidates(handle);
            return true;
        }
        let success = self.handles.mark_process_exited(handle, exit_code);
        if success {
            self.queue_object_wake_candidates(handle);
        }
        success
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

    pub fn guest_thread_start(&self, handle: u32) -> Option<(u32, u32, u32)> {
        self.handles.thread_start(handle)
    }

    pub fn set_event(&mut self, handle: u32) -> bool {
        let success = self.handles.set_event(handle);
        if success {
            self.queue_object_wake_candidates(handle);
        }
        self.record_event_trace(EventTraceRecord {
            op: "SetEvent",
            handle: Some(handle),
            name: None,
            manual_reset: None,
            signaled: None,
            result: Some(success),
            detail: Some(self.describe_handle(handle)),
        });
        success
    }

    pub fn reset_event(&mut self, handle: u32) -> bool {
        let success = self.handles.reset_event(handle);
        self.record_event_trace(EventTraceRecord {
            op: "ResetEvent",
            handle: Some(handle),
            name: None,
            manual_reset: None,
            signaled: None,
            result: Some(success),
            detail: Some(self.describe_handle(handle)),
        });
        success
    }

    pub fn pulse_event(&mut self, handle: u32) -> bool {
        let (manual_reset, wait_ids) = match self.handles.get(handle) {
            Ok(KernelObject::Event(event)) => {
                let mut wait_ids = self.scheduler.waiter_ids_for_handle(handle);
                if !event.manual_reset {
                    wait_ids.truncate(1);
                }
                (event.manual_reset, wait_ids)
            }
            _ => {
                self.record_event_trace(EventTraceRecord {
                    op: "PulseEvent",
                    handle: Some(handle),
                    name: None,
                    manual_reset: None,
                    signaled: None,
                    result: Some(false),
                    detail: Some(self.describe_handle(handle)),
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
        self.record_event_trace(EventTraceRecord {
            op: "PulseEvent",
            handle: Some(handle),
            name: None,
            manual_reset: Some(manual_reset),
            signaled: Some(false),
            result: Some(success),
            detail: Some(self.describe_handle(handle)),
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
                process_id: Some(process.process_id),
                thread_id: None,
                result: Some(result),
                error: Some(format!("exit=0x{:08x}", process.exit_code)),
                detail: None,
            }),
            Some(KernelObject::Thread(thread)) => Some(ProcessTraceRecord {
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
            }),
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

    pub fn register_blocked_waiter(
        &mut self,
        thread_id: u32,
        thread_handle: u32,
        wait_handles: Vec<u32>,
        kind: SchedulerBlockedWaitKind,
        wait_started_ms: u32,
        timeout_ms: u32,
    ) -> u64 {
        self.scheduler.register_blocked_wait(
            thread_id,
            thread_handle,
            wait_handles,
            kind,
            wait_started_ms,
            timeout_ms,
        )
    }

    pub fn remove_blocked_waiter(&mut self, wait_id: u64) -> Option<SchedulerBlockedWait> {
        self.pulsed_wait_handles.remove(&wait_id);
        self.comm_event_mask_changed_waits.remove(&wait_id);
        self.scheduler.remove_blocked_wait(wait_id)
    }

    pub fn remove_blocked_waiters_for_thread(&mut self, thread_id: u32) -> usize {
        let wait_ids = self
            .scheduler
            .blocked_waits()
            .filter(|wait| wait.thread_id == thread_id)
            .map(|wait| wait.id)
            .collect::<Vec<_>>();
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

    fn queue_object_wake_candidates(&mut self, handle: u32) {
        let wait_ids = self.scheduler.waiter_ids_for_handle(handle);
        self.scheduler.queue_pending_wake_ids(wait_ids);
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
        self.gwe.has_pending_sent_message_for_thread(thread_id)
    }

    pub fn select_ready_blocked_waiter(
        &self,
        active_thread_id: u32,
        now_ms: u32,
        mut is_ready: impl FnMut(&SchedulerBlockedWait, &Self) -> bool,
    ) -> Option<u64> {
        self.scheduler.select_ready_blocked_wait_id(
            active_thread_id,
            now_ms,
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
        let hwnd = self
            .gwe
            .create_window_ex_with_process_parent_owner_and_rect(
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
            if let Some(backing_store) = self.window_backing_stores.remove(&hwnd) {
                if backing_store.restore(framebuffer) {
                    restored = restored.saturating_add(1);
                }
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
        self.drain_remote_input_to_thread_window(thread_id, hwnd);
        if let Some(message) = self
            .gwe
            .get_message_filtered(thread_id, hwnd, min_msg, max_msg)
        {
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
        let message = self
            .gwe
            .get_message_filtered(thread_id, hwnd, min_msg, max_msg);
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
        let message = self
            .gwe
            .get_message_filtered(thread_id, hwnd, min_msg, max_msg)?;
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

    pub fn take_ready_visible_message_w_filtered(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
    ) -> Option<Message> {
        let message = self
            .gwe
            .get_visible_message_filtered(thread_id, hwnd, min_msg, max_msg)?;
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
        self.gwe
            .peek_visible_message_filtered(thread_id, hwnd, min_msg, max_msg)
    }

    pub fn take_ready_sent_message_w_filtered(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
    ) -> Option<Message> {
        self.expire_timed_out_send_messages();
        let message = self
            .gwe
            .take_sent_message_filtered(thread_id, hwnd, min_msg, max_msg)?;
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
        self.drain_remote_input_to_thread_window(thread_id, hwnd);
        if let Some(message) = self
            .gwe
            .peek_message_filtered(thread_id, hwnd, min_msg, max_msg, flags)
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
        let message = self
            .gwe
            .peek_message_filtered(thread_id, hwnd, min_msg, max_msg, flags);
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
        let lparam = self.queue_notify_icon_payload(payload, NOTIFYICONDATA_FIXED_SIZE_W);
        if lparam == 0 {
            return false;
        }
        self.post_window_message(taskbar_hwnd, WM_HANDLESHELLNOTIFYICON, op, lparam);
        true
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
        let lparam = self.queue_shell_notification_payload(ShellNotificationMessage {
            hwnd_from: 0,
            id_from: record.id,
            code,
            lparam: record.lparam,
            return_value: 0,
            data0,
            data1,
            link,
        });
        if lparam == 0 {
            return false;
        }
        if self.post_message_w(record.hwnd_sink, WM_NOTIFY, record.id, lparam) {
            true
        } else {
            let _ = self.release_message_pointer_payload(lparam);
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
                let message = Message::new(0, msg, wparam, lparam, time_ms);
                self.gwe.post_message(thread_id, message.clone());
                self.record_message_op(
                    "post_message",
                    thread_id,
                    &message,
                    Some(1),
                    Some("thread".to_owned()),
                );
                self.queue_message_wake_candidates(thread_id);
                true
            }
            hwnd => {
                let target_thread = self.gwe.window(hwnd).map(|window| window.thread_id);
                let message = Message::new(hwnd, msg, wparam, lparam, time_ms);
                let posted = self.gwe.post_message_for_window(hwnd, message.clone());
                if posted {
                    if let Some(target_thread) = target_thread {
                        self.record_message_op(
                            "post_message",
                            target_thread,
                            &message,
                            Some(1),
                            Some("window".to_owned()),
                        );
                        self.queue_message_wake_candidates(target_thread);
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
        let key_message = Message {
            hwnd: hwnd_value,
            msg: key_msg,
            wparam: virtual_key,
            lparam,
            time_ms,
            source: crate::ce::gwe::MSGSRC_HARDWARE_KEYBOARD,
            mouse_pos_at_post: None,
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
        let message = Message::new(0, msg, wparam, lparam, self.timers.tick_count());
        self.gwe.post_message(target_thread_id, message.clone());
        self.record_message_op(
            "post_message",
            target_thread_id,
            &message,
            Some(1),
            Some("thread".to_owned()),
        );
        self.queue_message_wake_candidates(target_thread_id);
        true
    }

    pub fn send_message_w(&mut self, hwnd: u32, msg: u32, wparam: u32, lparam: u32) -> Option<u32> {
        let target_thread = self
            .gwe
            .window(hwnd)
            .filter(|window| !window.destroyed)
            .map(|window| window.thread_id)?;
        self.gwe.begin_send_message(target_thread);
        let result = self.gwe.send_message(hwnd, msg, wparam, lparam);
        self.gwe.end_send_message(target_thread);
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
        let target_thread = self
            .gwe
            .window(hwnd)
            .filter(|window| !window.destroyed)
            .map(|window| window.thread_id)?;
        if target_thread == caller_thread_id {
            return None;
        }
        let flags = timeout_ms.map_or(SMF_NULL, |_| crate::ce::gwe::SMF_TIMEOUT);
        let send_id = self.gwe.queue_send_message_for_window(
            Some(caller_thread_id),
            hwnd,
            Message::new(hwnd, msg, wparam, lparam, self.timers.tick_count()),
            flags,
            timeout_ms,
        );
        if let Some(id) = send_id {
            self.gwe
                .set_sent_message_sender_process(id, self.current_process_id);
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
        let was_ready = self
            .gwe
            .active_sent_message_id(thread_id)
            .is_some_and(|send_id| self.gwe.sent_message_result_ready(send_id));
        let send_id = self.gwe.complete_active_sent_message(thread_id, result)?;
        if !was_ready {
            self.queue_send_reply_wake_candidates(send_id);
        }
        Some(send_id)
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
        self.gwe
            .post_quit_message(thread_id, exit_code, self.timers.tick_count());
        self.queue_message_wake_candidates(thread_id);
    }

    pub fn dispatch_message_w(&mut self, message: Message) -> u32 {
        self.dispatch_message_w_for_thread(0, message)
    }

    pub fn dispatch_message_w_for_thread(&mut self, thread_id: u32, message: Message) -> u32 {
        if message.msg == crate::ce::gwe::WM_QUIT {
            return message.wparam;
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

    pub fn message_pointer_payload(&self, ptr: u32) -> Option<MessagePointerPayload> {
        self.gwe.message_pointer_payload(ptr)
    }

    pub fn release_message_pointer_payload(&mut self, ptr: u32) -> Option<MessagePointerPayload> {
        let payload = self.gwe.take_message_pointer_payload(ptr)?;
        let _ = self.memory.heap_free(PROCESS_HEAP_HANDLE, 0, ptr);
        Some(payload)
    }

    fn release_dispatch_message_pointer_payload(&mut self, message: &Message) {
        let should_release = matches!(
            (
                message.msg,
                self.gwe.message_pointer_payload(message.lparam)
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
            self.release_message_pointer_payload(message.lparam);
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
        self.gwe.post_message(thread_id, message);
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
        self.queue_message_wake_candidates(thread_id);
    }

    fn dismiss_modal_dialog_wait_from_button(
        &mut self,
        dialog_hwnd: u32,
        thread_id: u32,
        command: u32,
        button_hwnd: u32,
    ) -> bool {
        let is_dialog = self
            .gwe
            .window(dialog_hwnd)
            .is_some_and(|window| window.class_name.eq_ignore_ascii_case("dialog"));
        if !is_dialog {
            return false;
        }
        let wait_id = self
            .scheduler
            .message_waiter_ids_for_thread(thread_id)
            .into_iter()
            .find(|wait_id| {
                self.scheduler.blocked_wait(*wait_id).is_some_and(|wait| {
                    matches!(
                        wait.kind,
                        SchedulerBlockedWaitKind::ModalMessageBox
                            | SchedulerBlockedWaitKind::PopupMenuModal { .. }
                    )
                })
            });
        let Some(wait_id) = wait_id else {
            return false;
        };
        self.modal_dialog_results
            .insert((thread_id, dialog_hwnd), command);
        let _ = self.remove_blocked_waiter(wait_id);
        let removed_get_message_waits =
            self.remove_dialog_get_message_waiters(thread_id, dialog_hwnd);
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

    fn remove_dialog_get_message_waiters(&mut self, thread_id: u32, dialog_hwnd: u32) -> usize {
        let wait_ids = self
            .scheduler
            .message_waiter_ids_for_thread(thread_id)
            .into_iter()
            .filter(|wait_id| {
                self.scheduler
                    .blocked_wait(*wait_id)
                    .is_some_and(|wait| match wait.kind {
                        SchedulerBlockedWaitKind::GetMessage {
                            hwnd: Some(hwnd), ..
                        } => hwnd == dialog_hwnd,
                        _ => false,
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
            self.queue_all_serial_read_wake_candidates();
            self.queue_all_serial_event_wake_candidates();
        }
        response
    }

    pub fn read_remote_serial_bytes(&mut self, max_bytes: usize) -> Vec<u8> {
        self.remote.read_serial_bytes(max_bytes)
    }

    pub fn drain_remote_input_to_gwe(&mut self, thread_id: u32, hwnd: u32) -> usize {
        if !self.gwe.is_window(hwnd) {
            return 0;
        }
        self.drain_remote_input_to_target(thread_id, Some(hwnd), RemoteTouchTargeting::Explicit)
            .posted
    }

    fn drain_remote_input_to_target(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        touch_targeting: RemoteTouchTargeting,
    ) -> RemoteInputDrain {
        let touch_events = self.remote.drain_touch_events();
        let key_events = self.remote.drain_key_events();
        let base_time_ms = self.timers.tick_count();
        let first_input_ms = touch_events
            .iter()
            .map(|event| event.enqueued_at_ms)
            .chain(key_events.iter().map(|event| event.enqueued_at_ms))
            .min()
            .unwrap_or(0);
        let mut posted = 0;
        let mut details = Vec::new();
        let mut target_thread_ids = Vec::new();

        for event in touch_events {
            let time_ms = remote_event_time_ms(base_time_ms, first_input_ms, event.enqueued_at_ms);
            let point = Point {
                x: event.x,
                y: event.y,
            };
            let target = match touch_targeting {
                RemoteTouchTargeting::Explicit => hwnd.map(|hwnd| (thread_id, hwnd)),
                RemoteTouchTargeting::ThreadHitTest => self
                    .gwe
                    .get_capture()
                    .or_else(|| self.gwe.window_from_point_for_thread(thread_id, point))
                    .or(hwnd)
                    .map(|hwnd| (thread_id, hwnd)),
                RemoteTouchTargeting::DesktopHitTest => self
                    .gwe
                    .get_capture()
                    .or_else(|| self.gwe.window_from_point(point))
                    .or_else(|| {
                        hwnd.filter(|hwnd| {
                            self.gwe.is_window_visible(*hwnd)
                                && self.gwe.is_window_enabled(*hwnd)
                                && self
                                    .gwe
                                    .get_window_rect(*hwnd)
                                    .is_some_and(|rect| rect.contains_point(point))
                        })
                    })
                    .and_then(|hwnd| {
                        self.gwe
                            .window_thread_process_id(hwnd)
                            .map(|(target_thread_id, _)| (target_thread_id, hwnd))
                    }),
            };
            let Some((target_thread_id, target)) =
                target.filter(|(_, hwnd)| self.gwe.is_window(*hwnd))
            else {
                details.push(format!(
                    "drop:msg=0x{:04x}/screen={},{}",
                    event.message, point.x, point.y
                ));
                self.record_message_trace(MessageTraceRecord {
                    op: "remote_touch_drop",
                    thread_id,
                    hwnd,
                    msg: Some(event.message),
                    wparam: None,
                    lparam: None,
                    screen_pos: Some(make_lparam(point.x, point.y)),
                    source: None,
                    result: Some(0),
                    detail: Some(
                        match touch_targeting {
                            RemoteTouchTargeting::Explicit => "no target window",
                            RemoteTouchTargeting::ThreadHitTest => "no thread hit-test target",
                            RemoteTouchTargeting::DesktopHitTest => "no desktop hit-test target",
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
            let wparam = if event.message == WM_LBUTTONDOWN || event.message == WM_MOUSEMOVE {
                1
            } else {
                0
            };
            self.post_gwe_message(
                target_thread_id,
                Message::new(
                    target,
                    event.message,
                    wparam,
                    make_lparam(client.x, client.y),
                    time_ms,
                )
                .with_mouse_pos(make_lparam(point.x, point.y)),
            );
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
                        RemoteTouchTargeting::ThreadHitTest => "thread-hit-test",
                        RemoteTouchTargeting::DesktopHitTest => "desktop-hit-test",
                    }
                    .to_owned(),
                ),
            });
            posted += 1;
        }

        let key_target = hwnd
            .or_else(|| self.gwe.get_capture())
            .or_else(|| self.gwe.get_active_window());
        for event in key_events {
            let time_ms = remote_event_time_ms(base_time_ms, first_input_ms, event.enqueued_at_ms);
            let Some(key_target) = key_target.filter(|hwnd| self.gwe.is_window(*hwnd)) else {
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
                    detail: Some("no target window".to_owned()),
                });
                continue;
            };
            self.post_gwe_message(
                thread_id,
                Message::new(key_target, event.message, event.vk, 1, time_ms)
                    .with_source(crate::ce::gwe::MSGSRC_HARDWARE_KEYBOARD),
            );
            if !target_thread_ids.contains(&thread_id) {
                target_thread_ids.push(thread_id);
            }
            details.push(format!(
                "key:hwnd=0x{key_target:08x}/thread={thread_id}/msg=0x{:04x}/vk=0x{:02x}",
                event.message, event.vk
            ));
            self.record_message_trace(MessageTraceRecord {
                op: "remote_key_target",
                thread_id,
                hwnd: Some(key_target),
                msg: Some(event.message),
                wparam: Some(event.vk),
                lparam: Some(1),
                screen_pos: None,
                source: Some(crate::ce::gwe::MSGSRC_HARDWARE_KEYBOARD),
                result: Some(1),
                detail: None,
            });
            posted += 1;
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
        let requested_hwnd = hwnd.filter(|hwnd| self.gwe.is_window(*hwnd));
        let targeting = if requested_hwnd.is_some() {
            RemoteTouchTargeting::ThreadHitTest
        } else {
            RemoteTouchTargeting::DesktopHitTest
        };
        let hwnd = hwnd
            .filter(|hwnd| self.gwe.is_window(*hwnd))
            .or_else(|| self.gwe.get_capture())
            .or_else(|| self.gwe.get_active_window());
        self.drain_remote_input_to_target(thread_id, hwnd, targeting)
    }

    pub fn drain_remote_input_to_active_window(&mut self) -> usize {
        self.drain_remote_input_to_active_window_detailed().posted
    }

    fn drain_remote_input_to_active_window_detailed(&mut self) -> RemoteInputDrain {
        let hwnd = self
            .gwe
            .get_capture()
            .or_else(|| self.gwe.get_active_window())
            .filter(|hwnd| self.gwe.is_window(*hwnd));
        let Some(hwnd) = hwnd else {
            return RemoteInputDrain {
                posted: 0,
                target_thread_ids: Vec::new(),
                detail: "no active window".to_owned(),
            };
        };
        let Some((thread_id, _process_id)) = self.gwe.window_thread_process_id(hwnd) else {
            return RemoteInputDrain {
                posted: 0,
                target_thread_ids: Vec::new(),
                detail: format!("no thread for active hwnd=0x{hwnd:08x}"),
            };
        };
        self.drain_remote_input_to_target(
            thread_id,
            Some(hwnd),
            RemoteTouchTargeting::DesktopHitTest,
        )
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
            let lparam = self
                .gwe
                .window_pos_for_rect(hwnd, after, insert_after, flags)
                .map(|payload| self.queue_window_pos_payload(payload))
                .unwrap_or(0);
            self.post_window_message(hwnd, WM_WINDOWPOSCHANGED, 0, lparam);
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

    fn queue_window_pos_payload(&mut self, payload: WindowPos) -> u32 {
        let Some(ptr) = self.memory.heap_alloc(PROCESS_HEAP_HANDLE, 0, 28) else {
            return 0;
        };
        if self
            .gwe
            .insert_message_pointer_payload(ptr, MessagePointerPayload::WindowPos(payload))
        {
            ptr
        } else {
            let _ = self.memory.heap_free(PROCESS_HEAP_HANDLE, 0, ptr);
            0
        }
    }

    fn queue_shell_notification_payload(&mut self, payload: ShellNotificationMessage) -> u32 {
        let link_bytes = payload
            .link
            .as_ref()
            .map(|link| (link.encode_utf16().count() + 1) * 2)
            .unwrap_or(0);
        let alloc_size = 28usize.saturating_add(link_bytes);
        let Some(ptr) = self
            .memory
            .heap_alloc(PROCESS_HEAP_HANDLE, 0, alloc_size as u32)
        else {
            return 0;
        };
        if self
            .gwe
            .insert_message_pointer_payload(ptr, MessagePointerPayload::ShellNotification(payload))
        {
            ptr
        } else {
            let _ = self.memory.heap_free(PROCESS_HEAP_HANDLE, 0, ptr);
            0
        }
    }

    fn queue_notify_icon_payload(&mut self, payload: NotifyIconMessage, alloc_size: u32) -> u32 {
        let Some(ptr) = self.memory.heap_alloc(PROCESS_HEAP_HANDLE, 0, alloc_size) else {
            return 0;
        };
        if self
            .gwe
            .insert_message_pointer_payload(ptr, MessagePointerPayload::NotifyIcon(payload))
        {
            ptr
        } else {
            let _ = self.memory.heap_free(PROCESS_HEAP_HANDLE, 0, ptr);
            0
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
            let lparam = self.queue_shell_file_change_payload(payload);
            if lparam == 0 {
                continue;
            }
            self.post_window_message(registration.hwnd, WM_FILECHANGEINFO, event_id, lparam);
            posted = posted.saturating_add(1);
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

    fn queue_shell_file_change_payload(&mut self, payload: FileChangeNotificationMessage) -> u32 {
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
            .heap_alloc(PROCESS_HEAP_HANDLE, 0, alloc_size as u32)
        else {
            return 0;
        };
        if self.gwe.insert_message_pointer_payload(
            ptr,
            MessagePointerPayload::FileChangeNotification(payload),
        ) {
            ptr
        } else {
            let _ = self.memory.heap_free(PROCESS_HEAP_HANDLE, 0, ptr);
            0
        }
    }

    fn post_window_message(&mut self, hwnd: u32, msg: u32, wparam: u32, lparam: u32) {
        let Some(window) = self.gwe.window(hwnd) else {
            return;
        };
        let message = Message::new(hwnd, msg, wparam, lparam, self.timers.tick_count());
        self.post_gwe_message(window.thread_id, message);
    }

    fn clear_timer_message_pending(&mut self, thread_id: u32, message: &Message) {
        if message.msg != crate::ce::gwe::WM_TIMER {
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
        ce::gwe::{
            BS_DEFPUSHBUTTON, WM_ACTIVATE, WM_KILLFOCUS, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_SETFOCUS,
            WS_CHILD, WS_VISIBLE,
        },
        config::RuntimeConfig,
    };

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
        assert!(
            !kernel
                .process_window_targets(launch.process_id, launch.thread_id)
                .contains(&hwnd)
        );
        assert!(
            kernel
                .process_window_backing_store_targets(launch.process_id, launch.thread_id)
                .contains(&hwnd)
        );

        kernel.mark_process_launch_exited_with_framebuffer(&launch, 0, Some(&mut framebuffer));

        assert_eq!(framebuffer.pixels(), original.as_slice());
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

    #[test]
    fn remote_input_blocked_thread_target_overrides_stale_active_window() -> Result<()> {
        let config = RuntimeConfig::load_default()?;
        let mut kernel = CeKernel::boot(config);
        let active_thread_id = 1;
        let blocked_thread_id = 2;
        let active = kernel.create_window_ex_w(active_thread_id, "ACTIVE", "", None, 0, 0, 0);
        let blocked = kernel.create_window_ex_w(blocked_thread_id, "BLOCKED", "", None, 0, 0, 0);
        assert_eq!(kernel.gwe.set_active_window(Some(active)), None);
        while kernel.gwe.get_message(active_thread_id).is_some() {}
        while kernel.gwe.get_message(blocked_thread_id).is_some() {}

        kernel.remote.set_framebuffer_size(800, 480);
        kernel.remote.enqueue_touch("tap", 640, 440).unwrap();

        assert_eq!(
            kernel.drain_remote_input_to_thread_window(blocked_thread_id, Some(blocked)),
            2
        );

        let mut active_messages = Vec::new();
        while let Some(message) = kernel.gwe.get_message(active_thread_id) {
            active_messages.push(message.msg);
        }
        assert!(!active_messages.contains(&WM_LBUTTONDOWN));
        assert!(!active_messages.contains(&WM_LBUTTONUP));
        let mut blocked_messages = Vec::new();
        while let Some(message) = kernel.gwe.get_message(blocked_thread_id) {
            blocked_messages.push((message.hwnd, message.msg, message.time_ms));
        }
        let down = blocked_messages
            .iter()
            .find(|(message_hwnd, msg, _)| *message_hwnd == blocked && *msg == WM_LBUTTONDOWN)
            .copied()
            .expect("blocked target receives mouse down");
        let up = blocked_messages
            .iter()
            .find(|(message_hwnd, msg, _)| *message_hwnd == blocked && *msg == WM_LBUTTONUP)
            .copied()
            .expect("blocked target receives mouse up");
        assert!(up.2 > down.2);
        Ok(())
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
            SchedulerBlockedWaitKind::ModalMessageBox,
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
        assert!(kernel.blocked_waiter(wait_id).is_none());
        assert!(kernel.blocked_waiter(dialog_get_message_wait_id).is_none());
        assert_eq!(kernel.take_modal_dialog_result(thread_id, dialog), Some(1));
        assert_eq!(kernel.take_modal_dialog_result(thread_id, dialog), None);
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
                    _ => false,
                }
            });
        assert_ne!(ready, Some(background_wait_id));
        assert!(ready.is_none());
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
}
