use std::{collections::BTreeMap, sync::OnceLock};

use crate::{
    ce::{
        audio::{
            MMSYSERR_BADDEVICEID, MMSYSERR_INVALHANDLE, MMSYSERR_NOERROR, MmResult,
            WAVERR_BADFORMAT, WaveBuffer, WaveFormat, WaveOutCallback,
        },
        cemath::{CeMathBinaryF32, CeMathBinaryF64, CeMathCall, CeMathUnaryF64, CeMathValue},
        coredll_ordinals::{self, *},
        crt,
        devices::DeviceIoControlResult,
        file::FileIoResult,
        file::FindData,
        framebuffer::{Framebuffer, FramebufferRect, PixelFormat},
        gwe::{
            Message, MessagePointerPayload, PeekFlags, Point, Rect, WNDCLASSW_SIZE, WS_CHILD,
            WindowPos,
        },
        kernel::{CeKernel, MessagePumpResult},
        memory::{HEAP_ZERO_MEMORY, PROCESS_HEAP_HANDLE},
        object::{CriticalSectionObject, KernelObject, ThreadResumeResult, ThreadSuspendResult},
        registry::{HKey, RegOpenResult, RegQueryValueResult},
        resource::{
            AcceleratorEntry, MenuItem, PopupMenuTracking, ResourceId, stock_object_handle,
        },
        thread::{
            ERROR_ALREADY_EXISTS, ERROR_CLASS_DOES_NOT_EXIST, ERROR_FILE_NOT_FOUND,
            ERROR_INVALID_HANDLE, ERROR_INVALID_PARAMETER, ERROR_INVALID_WINDOW_HANDLE,
            ERROR_NO_MORE_FILES, ERROR_NOT_ENOUGH_MEMORY, ERROR_NOT_OWNER, ERROR_NOT_SUPPORTED,
            ERROR_RESOURCE_NAME_NOT_FOUND, ERROR_SIGNAL_REFUSED,
        },
    },
    error::{Error, Result},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoredllExport {
    pub name: String,
    pub ordinal: u32,
}

const CE_ACP_CODE_PAGE: u32 = 949;
const CTYPE_UPPER: u32 = 0x0001;
const CTYPE_LOWER: u32 = 0x0002;
const CTYPE_DIGIT: u32 = 0x0004;
const CTYPE_SPACE: u32 = 0x0008;
const CTYPE_PUNCT: u32 = 0x0010;
const CTYPE_CONTROL: u32 = 0x0020;
const CTYPE_BLANK: u32 = 0x0040;
const CTYPE_HEX: u32 = 0x0080;
const CTYPE_ALPHA: u32 = 0x0100;
const SPI_GETWORKAREA: u32 = 0x0030;
const SPI_GETPLATFORMTYPE: u32 = 0x0101;
const SPI_GETOEMINFO: u32 = 0x0102;
const SYSTEM_PARAMETERS_INFO_REGISTRY_PATH: &str = r"HKLM\System\Emulator\SystemParametersInfo";
const SHELL_FOLDERS_REGISTRY_PATH: &str = r"HKLM\System\Explorer\Shell Folders";
const IOCTL_HAL_GET_DEVICEID: u32 = 0x0101_207c;
const EVENT_ALL_ACCESS: u32 = 0x001f_0003;
const RDW_INVALIDATE: u32 = 0x0001;
const RDW_INTERNALPAINT: u32 = 0x0002;
const RDW_ERASE: u32 = 0x0004;
const RDW_VALIDATE: u32 = 0x0008;
const RDW_NOINTERNALPAINT: u32 = 0x0010;
const RDW_NOERASE: u32 = 0x0020;
const RDW_NOCHILDREN: u32 = 0x0040;
const RDW_ALLCHILDREN: u32 = 0x0080;
const RDW_UPDATENOW: u32 = 0x0100;
const RDW_ERASENOW: u32 = 0x0200;
const TPM_RETURNCMD: u32 = 0x0100;
const TPMPARAMS_SIZE: u32 = 20;
const STRSAFE_E_INSUFFICIENT_BUFFER: u32 = 0x8007_007a;
const STRSAFE_E_INVALID_PARAMETER: u32 = 0x8007_0057;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CoredllSubsystem {
    KernelSync,
    ThreadProcess,
    Memory,
    FileSystem,
    DeviceIo,
    Registry,
    GweWindow,
    GweMessage,
    GdiGraphics,
    Multimedia,
    LocaleString,
    Time,
    Crypto,
    Comm,
    Storage,
    MsgQueue,
    Power,
    Services,
    Telephony,
    Security,
    Debug,
    InputIme,
    ShellUi,
    Bluetooth,
    EventLog,
    Credential,
    MathCrt,
    KernelPrivate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoredllImplementationStatus {
    Implemented,
    Stubbed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoredllOrdinalPlan {
    pub export: CoredllExport,
    pub subsystem: CoredllSubsystem,
    pub status: CoredllImplementationStatus,
}

#[derive(Debug, Clone)]
pub struct CoredllExportTable {
    exports: Vec<CoredllExport>,
    by_name: BTreeMap<String, usize>,
    by_ordinal: BTreeMap<u32, Vec<usize>>,
}

impl Default for CoredllExportTable {
    fn default() -> Self {
        Self::from_static_ordinals()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventModifyAction {
    Set,
    Reset,
    Pulse,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CoredllCall {
    RegCloseKey {
        hkey: HKey,
    },
    RegOpenKeyExW {
        hkey: HKey,
        subkey: Option<String>,
        options: u32,
        sam_desired: u32,
    },
    RegQueryValueExW {
        hkey: HKey,
        value_name: Option<String>,
        data_capacity: Option<usize>,
    },
    RegSetValueExW {
        hkey: HKey,
        value_name: Option<String>,
        value_type: u32,
        data: Vec<u8>,
    },
    CreateFileW {
        path: String,
        desired_access: u32,
        creation_disposition: u32,
    },
    ReadFile {
        handle: u32,
        requested: u32,
    },
    WriteFile {
        handle: u32,
        data: Vec<u8>,
    },
    DeviceIoControl {
        handle: u32,
        ioctl_code: u32,
        input: Vec<u8>,
        output_capacity: u32,
    },
    CloseHandle {
        handle: u32,
    },
    CreateEventW {
        name: Option<String>,
        manual_reset: bool,
        initial_state: bool,
    },
    EventModify {
        handle: u32,
        action: EventModifyAction,
    },
    WaitForSingleObject {
        handle: u32,
        timeout_ms: u32,
        thread_id: u32,
    },
    CreateMutexW {
        name: Option<String>,
        initial_owner_thread: Option<u32>,
    },
    ReleaseMutex {
        handle: u32,
        thread_id: u32,
    },
    CreateWindowExW {
        thread_id: u32,
        class_name: String,
        title: String,
        parent: Option<u32>,
        id: u32,
        style: u32,
        ex_style: u32,
    },
    DestroyWindow {
        hwnd: u32,
    },
    SetWindowTextW {
        hwnd: u32,
        title: String,
    },
    GetWindowTextW {
        hwnd: u32,
        capacity_chars: usize,
    },
    GetWindowLongW {
        hwnd: u32,
        index: i32,
    },
    SetWindowLongW {
        hwnd: u32,
        index: i32,
        value: u32,
    },
    GetMessageW {
        thread_id: u32,
    },
    PeekMessageW {
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
        flags: PeekFlags,
    },
    PostMessageW {
        hwnd: u32,
        msg: u32,
        wparam: u32,
        lparam: u32,
    },
    SendMessageW {
        hwnd: u32,
        msg: u32,
        wparam: u32,
        lparam: u32,
    },
    DispatchMessageW {
        message: Message,
    },
    TranslateMessage {
        message: Message,
    },
    DefWindowProcW {
        hwnd: u32,
        msg: u32,
        wparam: u32,
        lparam: u32,
    },
    SetTimer {
        hwnd: Option<u32>,
        requested_id: Option<u32>,
        period_ms: u32,
    },
    KillTimer {
        id: u32,
    },
    GetTickCount,
    Sleep {
        ms: u32,
    },
    WaveOutGetNumDevs,
    WaveOutOpen {
        format: WaveFormat,
    },
    WaveOutWrite {
        id: u32,
        buffer: WaveBuffer,
    },
    WaveOutPause {
        id: u32,
    },
    WaveOutRestart {
        id: u32,
    },
    WaveOutReset {
        id: u32,
    },
    WaveOutClose {
        id: u32,
    },
    WaveOutGetVolume {
        id: u32,
    },
    WaveOutSetVolume {
        id: u32,
        volume: u32,
    },
    CeMath(CeMathCall),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CoredllValue {
    Bool(bool),
    U32(u32),
    Handle(u32),
    Bytes(Vec<u8>),
    String(String),
    OptionalMessage(Option<Message>),
    MessagePump(MessagePumpResult),
    FileIo(FileIoResult),
    DeviceIoControl(DeviceIoControlResult),
    RegOpen(RegOpenResult),
    RegQuery(RegQueryValueResult),
    MmResult(MmResult),
    MmOpen {
        status: MmResult,
        handle: Option<u32>,
    },
    CeMath(CeMathValue),
}

pub trait CoredllGuestMemory {
    fn read_u8(&self, addr: u32) -> Result<u8>;
    fn write_u8(&mut self, addr: u32, value: u8) -> Result<()>;
    fn read_u32(&self, addr: u32) -> Result<u32>;
    fn write_u32(&mut self, addr: u32, value: u32) -> Result<()>;
    fn read_u16(&self, addr: u32) -> Result<u16>;
    fn write_u16(&mut self, addr: u32, value: u16) -> Result<()>;

    fn read_bytes(&self, addr: u32, out: &mut [u8]) -> Result<()> {
        for (offset, byte) in out.iter_mut().enumerate() {
            *byte = self.read_u8(addr.wrapping_add(offset as u32))?;
        }
        Ok(())
    }

    fn write_bytes(&mut self, addr: u32, bytes: &[u8]) -> Result<()> {
        for (offset, byte) in bytes.iter().copied().enumerate() {
            self.write_u8(addr.wrapping_add(offset as u32), byte)?;
        }
        Ok(())
    }

    fn fill_bytes(&mut self, addr: u32, value: u8, len: u32) -> Result<()> {
        for offset in 0..len {
            self.write_u8(addr.wrapping_add(offset), value)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CoredllDispatch {
    Returned {
        export: CoredllExport,
        value: CoredllValue,
    },
    UnresolvedOrdinal(u32),
    UnresolvedName(String),
    Unimplemented {
        export: CoredllExport,
    },
    Stubbed {
        export: CoredllExport,
        stub: CoredllStubResult,
    },
    OrdinalMismatch {
        ordinal: u32,
        export: CoredllExport,
        call_name: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoredllStubResult {
    pub subsystem: CoredllSubsystem,
    pub policy: CoredllStubPolicy,
    pub return_value: u32,
    pub args: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoredllStubPolicy {
    VoidNoOp,
    BoolSuccess,
    BoolFailure,
    NullPointer,
    InvalidHandle,
    ZeroValue,
}

impl CoredllExportTable {
    fn empty() -> Self {
        Self {
            exports: Vec::new(),
            by_name: BTreeMap::new(),
            by_ordinal: BTreeMap::new(),
        }
    }

    pub fn static_ordinals() -> &'static Self {
        static TABLE: OnceLock<CoredllExportTable> = OnceLock::new();
        TABLE.get_or_init(Self::from_static_ordinals)
    }

    pub fn from_static_ordinals() -> Self {
        let mut table = Self::empty();
        for ordinal in COREDLL_EXPORTS {
            table.insert(CoredllExport::from_static_ordinal(ordinal));
        }
        for ordinal in SDK_ORDINALS {
            table.insert(CoredllExport::from_static_ordinal(ordinal));
        }
        for ordinal in SUPPLEMENTAL_ORDINALS {
            table.insert(CoredllExport::from_static_ordinal(ordinal));
        }
        table
    }

    pub fn resolve_static_ordinal(ordinal: u32) -> Option<CoredllExport> {
        coredll_ordinals::lookup(ordinal).map(CoredllExport::from_static_ordinal)
    }

    pub fn export_count(&self) -> usize {
        self.exports.len()
    }

    pub fn resolve_name(&self, name: &str) -> Option<&CoredllExport> {
        self.by_name
            .get(&normalize_name(name))
            .and_then(|index| self.exports.get(*index))
    }

    pub fn resolve_ordinal(&self, ordinal: u32) -> Option<&CoredllExport> {
        self.by_ordinal
            .get(&ordinal)
            .and_then(|indices| indices.first())
            .and_then(|index| self.exports.get(*index))
    }

    pub fn exports_by_ordinal(&self, ordinal: u32) -> Vec<&CoredllExport> {
        self.by_ordinal
            .get(&ordinal)
            .into_iter()
            .flat_map(|indices| indices.iter())
            .filter_map(|index| self.exports.get(*index))
            .collect()
    }

    pub fn ordinals(&self) -> impl Iterator<Item = u32> + '_ {
        self.by_ordinal.keys().copied()
    }

    pub fn ordinal_plan(&self) -> Vec<CoredllOrdinalPlan> {
        self.exports
            .iter()
            .cloned()
            .map(|export| {
                let subsystem = export.subsystem();
                let status = if export.has_real_shim() {
                    CoredllImplementationStatus::Implemented
                } else {
                    CoredllImplementationStatus::Stubbed
                };
                CoredllOrdinalPlan {
                    export,
                    subsystem,
                    status,
                }
            })
            .collect()
    }

    pub fn dispatch_by_name(
        &self,
        kernel: &mut CeKernel,
        name: &str,
        call: CoredllCall,
    ) -> CoredllDispatch {
        let Some(export) = self.resolve_name(name).cloned() else {
            return CoredllDispatch::UnresolvedName(name.to_owned());
        };
        dispatch_resolved(kernel, export, call)
    }

    pub fn dispatch_by_ordinal(
        &self,
        kernel: &mut CeKernel,
        ordinal: u32,
        call: CoredllCall,
    ) -> CoredllDispatch {
        let Some(export) = self.resolve_ordinal(ordinal).cloned() else {
            return CoredllDispatch::UnresolvedOrdinal(ordinal);
        };
        if !export.matches_name(call.export_name()) {
            return CoredllDispatch::OrdinalMismatch {
                ordinal,
                export,
                call_name: call.export_name(),
            };
        }
        dispatch_resolved(kernel, export, call)
    }

    pub fn dispatch_untyped_ordinal(&self, ordinal: u32) -> CoredllDispatch {
        self.dispatch_raw_ordinal(ordinal, [])
    }

    pub fn dispatch_raw_ordinal<I>(&self, ordinal: u32, args: I) -> CoredllDispatch
    where
        I: IntoIterator<Item = u32>,
    {
        match self.resolve_ordinal(ordinal).cloned() {
            Some(export) => {
                let args = args.into_iter().collect();
                let stub = CoredllStubResult::for_export(&export, args);
                CoredllDispatch::Stubbed { export, stub }
            }
            None => CoredllDispatch::UnresolvedOrdinal(ordinal),
        }
    }

    pub fn dispatch_raw_ordinal_with_memory<M, I>(
        &self,
        kernel: &mut CeKernel,
        memory: &mut M,
        thread_id: u32,
        ordinal: u32,
        args: I,
    ) -> CoredllDispatch
    where
        M: CoredllGuestMemory,
        I: IntoIterator<Item = u32>,
    {
        self.dispatch_raw_ordinal_with_framebuffer(kernel, memory, None, thread_id, ordinal, args)
    }

    pub fn dispatch_raw_ordinal_with_framebuffer<M, I>(
        &self,
        kernel: &mut CeKernel,
        memory: &mut M,
        framebuffer: Option<&mut dyn Framebuffer>,
        thread_id: u32,
        ordinal: u32,
        args: I,
    ) -> CoredllDispatch
    where
        M: CoredllGuestMemory,
        I: IntoIterator<Item = u32>,
    {
        match self.resolve_ordinal(ordinal).cloned() {
            Some(export) => {
                let args = args.into_iter().collect::<Vec<_>>();
                if let Some(value) = dispatch_real_raw_ordinal(
                    kernel,
                    memory,
                    framebuffer,
                    thread_id,
                    &export,
                    &args,
                ) {
                    CoredllDispatch::Returned { export, value }
                } else {
                    let stub = CoredllStubResult::for_export(&export, args);
                    CoredllDispatch::Stubbed { export, stub }
                }
            }
            None => CoredllDispatch::UnresolvedOrdinal(ordinal),
        }
    }

    fn insert(&mut self, export: CoredllExport) {
        if self
            .exports_by_ordinal(export.ordinal)
            .iter()
            .any(|existing| existing.matches_name(&export.name))
        {
            return;
        }
        let index = self.exports.len();
        self.by_name
            .entry(normalize_name(&export.name))
            .or_insert(index);
        self.by_ordinal
            .entry(export.ordinal)
            .or_default()
            .push(index);
        self.exports.push(export);
    }
}

impl CoredllStubResult {
    fn for_export(export: &CoredllExport, args: Vec<u32>) -> Self {
        let subsystem = export.subsystem();
        let policy = CoredllStubPolicy::for_export(export, subsystem);
        Self {
            subsystem,
            policy,
            return_value: policy.return_value(),
            args,
        }
    }
}

impl CoredllStubPolicy {
    fn for_export(export: &CoredllExport, subsystem: CoredllSubsystem) -> Self {
        let name = normalize_name(&export.name);
        if VOID_NOOP_STUBS
            .iter()
            .any(|stub_name| normalize_name(stub_name) == name)
        {
            return Self::VoidNoOp;
        }
        if BOOL_SUCCESS_STUBS
            .iter()
            .any(|stub_name| normalize_name(stub_name) == name)
        {
            return Self::BoolSuccess;
        }
        if INVALID_HANDLE_STUBS
            .iter()
            .any(|stub_name| normalize_name(stub_name) == name)
        {
            return Self::InvalidHandle;
        }
        if NULL_POINTER_STUB_PREFIXES
            .iter()
            .any(|prefix| name.starts_with(&normalize_name(prefix)))
        {
            return Self::NullPointer;
        }
        if BOOL_FAILURE_STUB_PREFIXES
            .iter()
            .any(|prefix| name.starts_with(&normalize_name(prefix)))
        {
            return Self::BoolFailure;
        }
        match subsystem {
            CoredllSubsystem::Memory => Self::NullPointer,
            CoredllSubsystem::FileSystem
            | CoredllSubsystem::DeviceIo
            | CoredllSubsystem::Registry
            | CoredllSubsystem::GweWindow
            | CoredllSubsystem::GweMessage
            | CoredllSubsystem::GdiGraphics
            | CoredllSubsystem::Multimedia
            | CoredllSubsystem::Comm
            | CoredllSubsystem::Storage
            | CoredllSubsystem::MsgQueue
            | CoredllSubsystem::Power
            | CoredllSubsystem::Services
            | CoredllSubsystem::Telephony
            | CoredllSubsystem::Security
            | CoredllSubsystem::InputIme
            | CoredllSubsystem::ShellUi
            | CoredllSubsystem::Bluetooth
            | CoredllSubsystem::EventLog
            | CoredllSubsystem::Credential => Self::BoolFailure,
            CoredllSubsystem::KernelSync
            | CoredllSubsystem::ThreadProcess
            | CoredllSubsystem::LocaleString
            | CoredllSubsystem::Time
            | CoredllSubsystem::Crypto
            | CoredllSubsystem::Debug
            | CoredllSubsystem::MathCrt
            | CoredllSubsystem::KernelPrivate => Self::ZeroValue,
        }
    }

    fn return_value(self) -> u32 {
        match self {
            Self::VoidNoOp | Self::ZeroValue => 0,
            Self::BoolSuccess => 1,
            Self::BoolFailure | Self::NullPointer => 0,
            Self::InvalidHandle => u32::MAX,
        }
    }
}

impl CoredllExport {
    fn from_static_ordinal(ordinal: &CoredllOrdinalDef) -> Self {
        Self {
            name: ordinal.name.to_owned(),
            ordinal: ordinal.ordinal,
        }
    }

    fn matches_name(&self, name: &str) -> bool {
        normalize_name(&self.name) == normalize_name(name)
    }

    pub fn subsystem(&self) -> CoredllSubsystem {
        CoredllSubsystem::for_export(self)
    }

    pub fn has_real_shim(&self) -> bool {
        IMPLEMENTED_EXPORTS
            .iter()
            .any(|name| self.matches_name(name))
            || CEMATH_EXPORTS
                .iter()
                .any(|(name, _ordinal)| self.matches_name(name))
    }
}

impl CoredllSubsystem {
    fn for_export(export: &CoredllExport) -> Self {
        let name = normalize_name(&export.name);
        if is_sdk_crt_export(export) || has_any_prefix(&name, MATH_PREFIXES) {
            return Self::MathCrt;
        }
        if has_any_prefix(&name, REGISTRY_PREFIXES) {
            return Self::Registry;
        }
        if has_any_prefix(&name, MEMORY_PREFIXES) {
            return Self::Memory;
        }
        if has_any_prefix(&name, SYNC_PREFIXES) {
            return Self::KernelSync;
        }
        if has_any_prefix(&name, THREAD_PROCESS_PREFIXES) {
            return Self::ThreadProcess;
        }
        if has_any_prefix(&name, TIME_PREFIXES) {
            return Self::Time;
        }
        if has_any_prefix(&name, LOCALE_STRING_PREFIXES) {
            return Self::LocaleString;
        }
        if has_any_prefix(&name, CRYPTO_PREFIXES) {
            return Self::Crypto;
        }
        if has_any_prefix(&name, COMM_PREFIXES) {
            return Self::Comm;
        }
        if has_any_prefix(&name, DEVICE_IO_PREFIXES) {
            return Self::DeviceIo;
        }
        if has_any_prefix(&name, STORAGE_PREFIXES) {
            return Self::Storage;
        }
        if has_any_prefix(&name, MSG_QUEUE_PREFIXES) {
            return Self::MsgQueue;
        }
        if has_any_prefix(&name, POWER_PREFIXES) {
            return Self::Power;
        }
        if has_any_prefix(&name, SERVICE_PREFIXES) {
            return Self::Services;
        }
        if has_any_prefix(&name, TELEPHONY_PREFIXES) {
            return Self::Telephony;
        }
        if has_any_prefix(&name, SECURITY_PREFIXES) {
            return Self::Security;
        }
        if has_any_prefix(&name, DEBUG_PREFIXES) {
            return Self::Debug;
        }
        if has_any_prefix(&name, INPUT_IME_PREFIXES) {
            return Self::InputIme;
        }
        if has_any_prefix(&name, SHELL_UI_PREFIXES) {
            return Self::ShellUi;
        }
        if has_any_prefix(&name, BLUETOOTH_PREFIXES) {
            return Self::Bluetooth;
        }
        if has_any_prefix(&name, EVENT_LOG_PREFIXES) {
            return Self::EventLog;
        }
        if has_any_prefix(&name, CREDENTIAL_PREFIXES) {
            return Self::Credential;
        }
        if has_any_prefix(&name, GWE_MESSAGE_PREFIXES) {
            return Self::GweMessage;
        }
        if has_any_prefix(&name, GWE_WINDOW_PREFIXES) || (246..=293).contains(&export.ordinal) {
            return Self::GweWindow;
        }
        if has_any_prefix(&name, GDI_PREFIXES) || (873..=987).contains(&export.ordinal) {
            return Self::GdiGraphics;
        }
        if has_any_prefix(&name, MULTIMEDIA_PREFIXES) || (379..=454).contains(&export.ordinal) {
            return Self::Multimedia;
        }
        if has_any_prefix(&name, FILESYSTEM_PREFIXES) {
            return Self::FileSystem;
        }
        Self::KernelPrivate
    }
}

impl CoredllCall {
    pub fn export_name(&self) -> &'static str {
        match self {
            Self::RegCloseKey { .. } => "RegCloseKey",
            Self::RegOpenKeyExW { .. } => "RegOpenKeyExW",
            Self::RegQueryValueExW { .. } => "RegQueryValueExW",
            Self::RegSetValueExW { .. } => "RegSetValueExW",
            Self::CreateFileW { .. } => "CreateFileW",
            Self::ReadFile { .. } => "ReadFile",
            Self::WriteFile { .. } => "WriteFile",
            Self::DeviceIoControl { .. } => "DeviceIoControl",
            Self::CloseHandle { .. } => "CloseHandle",
            Self::CreateEventW { .. } => "CreateEventW",
            Self::EventModify { .. } => "EventModify",
            Self::WaitForSingleObject { .. } => "WaitForSingleObject",
            Self::CreateMutexW { .. } => "CreateMutexW",
            Self::ReleaseMutex { .. } => "ReleaseMutex",
            Self::CreateWindowExW { .. } => "CreateWindowExW",
            Self::DestroyWindow { .. } => "DestroyWindow",
            Self::SetWindowTextW { .. } => "SetWindowTextW",
            Self::GetWindowTextW { .. } => "GetWindowTextW",
            Self::GetWindowLongW { .. } => "GetWindowLongW",
            Self::SetWindowLongW { .. } => "SetWindowLongW",
            Self::GetMessageW { .. } => "GetMessageW",
            Self::PeekMessageW { .. } => "PeekMessageW",
            Self::PostMessageW { .. } => "PostMessageW",
            Self::SendMessageW { .. } => "SendMessageW",
            Self::DispatchMessageW { .. } => "DispatchMessageW",
            Self::TranslateMessage { .. } => "TranslateMessage",
            Self::DefWindowProcW { .. } => "DefWindowProcW",
            Self::SetTimer { .. } => "SetTimer",
            Self::KillTimer { .. } => "KillTimer",
            Self::GetTickCount => "GetTickCount",
            Self::Sleep { .. } => "Sleep",
            Self::WaveOutGetNumDevs => "waveOutGetNumDevs",
            Self::WaveOutOpen { .. } => "waveOutOpen",
            Self::WaveOutWrite { .. } => "waveOutWrite",
            Self::WaveOutPause { .. } => "waveOutPause",
            Self::WaveOutRestart { .. } => "waveOutRestart",
            Self::WaveOutReset { .. } => "waveOutReset",
            Self::WaveOutClose { .. } => "waveOutClose",
            Self::WaveOutGetVolume { .. } => "waveOutGetVolume",
            Self::WaveOutSetVolume { .. } => "waveOutSetVolume",
            Self::CeMath(call) => call.export_name(),
        }
    }
}

fn dispatch_resolved(
    kernel: &mut CeKernel,
    export: CoredllExport,
    call: CoredllCall,
) -> CoredllDispatch {
    if !export.matches_name(call.export_name()) {
        return CoredllDispatch::Unimplemented { export };
    }

    let value = match call {
        CoredllCall::RegCloseKey { hkey } => CoredllValue::U32(kernel.registry.reg_close_key(hkey)),
        CoredllCall::RegOpenKeyExW {
            hkey,
            subkey,
            options,
            sam_desired,
        } => CoredllValue::RegOpen(kernel.registry.reg_open_key_exw(
            hkey,
            subkey.as_deref(),
            options,
            sam_desired,
        )),
        CoredllCall::RegQueryValueExW {
            hkey,
            value_name,
            data_capacity,
        } => CoredllValue::RegQuery(kernel.registry.reg_query_value_exw(
            hkey,
            value_name.as_deref(),
            data_capacity,
        )),
        CoredllCall::RegSetValueExW {
            hkey,
            value_name,
            value_type,
            data,
        } => CoredllValue::U32(kernel.registry.reg_set_value_exw(
            hkey,
            value_name.as_deref(),
            value_type,
            &data,
        )),
        CoredllCall::CreateFileW {
            path,
            desired_access,
            creation_disposition,
        } => match kernel.create_file_w(&path, desired_access, creation_disposition) {
            Ok(handle) => CoredllValue::Handle(handle),
            Err(_) => CoredllValue::Handle(u32::MAX),
        },
        CoredllCall::ReadFile { handle, requested } => {
            CoredllValue::Bytes(kernel.read_file(handle, requested).unwrap_or_default())
        }
        CoredllCall::WriteFile { handle, data } => {
            CoredllValue::FileIo(kernel.write_file(handle, &data).unwrap_or(FileIoResult {
                success: false,
                bytes_transferred: 0,
            }))
        }
        CoredllCall::DeviceIoControl {
            handle,
            ioctl_code,
            input,
            output_capacity,
        } => CoredllValue::DeviceIoControl(
            kernel
                .device_io_control(handle, ioctl_code, &input, output_capacity)
                .unwrap_or(DeviceIoControlResult {
                    success: false,
                    bytes_returned: 0,
                    output: Vec::new(),
                }),
        ),
        CoredllCall::CloseHandle { handle } => {
            CoredllValue::Bool(kernel.close_handle(handle).unwrap_or(false))
        }
        CoredllCall::CreateEventW {
            name,
            manual_reset,
            initial_state,
        } => CoredllValue::Handle(kernel.create_event_w(name, manual_reset, initial_state)),
        CoredllCall::EventModify { handle, action } => {
            let ok = match action {
                EventModifyAction::Set => kernel.set_event(handle),
                EventModifyAction::Reset => kernel.reset_event(handle),
                EventModifyAction::Pulse => kernel.set_event(handle) && kernel.reset_event(handle),
            };
            CoredllValue::Bool(ok)
        }
        CoredllCall::WaitForSingleObject {
            handle,
            timeout_ms,
            thread_id,
        } => CoredllValue::U32(kernel.wait_for_single_object(handle, timeout_ms, thread_id)),
        CoredllCall::CreateMutexW {
            name,
            initial_owner_thread,
        } => CoredllValue::Handle(kernel.create_mutex_w(name, initial_owner_thread)),
        CoredllCall::ReleaseMutex { handle, thread_id } => {
            CoredllValue::Bool(kernel.release_mutex(handle, thread_id))
        }
        CoredllCall::CreateWindowExW {
            thread_id,
            class_name,
            title,
            parent,
            id,
            style,
            ex_style,
        } => CoredllValue::Handle(kernel.create_window_ex_w(
            thread_id,
            &class_name,
            &title,
            parent,
            id,
            style,
            ex_style,
        )),
        CoredllCall::DestroyWindow { hwnd } => CoredllValue::Bool(kernel.destroy_window(hwnd)),
        CoredllCall::SetWindowTextW { hwnd, title } => {
            CoredllValue::Bool(kernel.gwe.set_window_text(hwnd, &title))
        }
        CoredllCall::GetWindowTextW {
            hwnd,
            capacity_chars,
        } => CoredllValue::String(
            kernel
                .gwe
                .get_window_text(hwnd, capacity_chars)
                .unwrap_or_default(),
        ),
        CoredllCall::GetWindowLongW { hwnd, index } => {
            CoredllValue::U32(kernel.gwe.get_window_long(hwnd, index).unwrap_or(0))
        }
        CoredllCall::SetWindowLongW { hwnd, index, value } => {
            CoredllValue::U32(kernel.gwe.set_window_long(hwnd, index, value).unwrap_or(0))
        }
        CoredllCall::GetMessageW { thread_id } => {
            CoredllValue::OptionalMessage(kernel.get_message_w(thread_id))
        }
        CoredllCall::PeekMessageW {
            thread_id,
            hwnd,
            min_msg,
            max_msg,
            flags,
        } => {
            kernel.pump_timers_to_gwe(thread_id);
            kernel.drain_remote_input_to_thread_window(thread_id, hwnd);
            CoredllValue::OptionalMessage(
                kernel
                    .gwe
                    .peek_message_filtered(thread_id, hwnd, min_msg, max_msg, flags),
            )
        }
        CoredllCall::PostMessageW {
            hwnd,
            msg,
            wparam,
            lparam,
        } => CoredllValue::Bool(kernel.post_message_w(hwnd, msg, wparam, lparam)),
        CoredllCall::SendMessageW {
            hwnd,
            msg,
            wparam,
            lparam,
        }
        | CoredllCall::DefWindowProcW {
            hwnd,
            msg,
            wparam,
            lparam,
        } => CoredllValue::U32(
            kernel
                .send_message_w(hwnd, msg, wparam, lparam)
                .unwrap_or(0),
        ),
        CoredllCall::DispatchMessageW { message } => {
            CoredllValue::U32(kernel.dispatch_message_w(message))
        }
        CoredllCall::TranslateMessage { message: _ } => CoredllValue::Bool(true),
        CoredllCall::SetTimer {
            hwnd,
            requested_id,
            period_ms,
        } => CoredllValue::U32(kernel.set_timer(hwnd, requested_id, period_ms)),
        CoredllCall::KillTimer { id } => CoredllValue::Bool(kernel.kill_timer(id)),
        CoredllCall::GetTickCount => CoredllValue::U32(kernel.timers.tick_count()),
        CoredllCall::Sleep { ms } => {
            kernel.timers.sleep_ms(ms);
            CoredllValue::U32(0)
        }
        CoredllCall::WaveOutGetNumDevs => CoredllValue::U32(kernel.audio.wave_out_get_num_devs()),
        CoredllCall::WaveOutOpen { format } => match kernel.wave_out_open(format) {
            Ok(handle) => CoredllValue::MmOpen {
                status: 0,
                handle: Some(handle),
            },
            Err(status) => CoredllValue::MmOpen {
                status,
                handle: None,
            },
        },
        CoredllCall::WaveOutWrite { id, buffer } => {
            CoredllValue::MmResult(kernel.wave_out_write(id, buffer))
        }
        CoredllCall::WaveOutPause { id } => CoredllValue::MmResult(kernel.audio.pause(id)),
        CoredllCall::WaveOutRestart { id } => CoredllValue::MmResult(kernel.audio.restart(id)),
        CoredllCall::WaveOutReset { id } => CoredllValue::MmResult(kernel.audio.wave_out_reset(id)),
        CoredllCall::WaveOutClose { id } => CoredllValue::MmResult(kernel.audio.wave_out_close(id)),
        CoredllCall::WaveOutGetVolume { id } => {
            CoredllValue::U32(kernel.audio.get_volume(id).unwrap_or(0))
        }
        CoredllCall::WaveOutSetVolume { id, volume } => {
            CoredllValue::MmResult(kernel.audio.wave_out_set_volume(id, volume))
        }
        CoredllCall::CeMath(call) => CoredllValue::CeMath(kernel.math.eval(call)),
    };

    CoredllDispatch::Returned { export, value }
}

fn dispatch_real_raw_ordinal<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    framebuffer: Option<&mut dyn Framebuffer>,
    thread_id: u32,
    export: &CoredllExport,
    args: &[u32],
) -> Option<CoredllValue> {
    match export.ordinal {
        ORD_INITIALIZE_CRITICAL_SECTION => {
            initialize_critical_section(kernel, memory, thread_id, raw_arg(args, 0));
            Some(CoredllValue::U32(0))
        }
        ORD_DELETE_CRITICAL_SECTION => {
            delete_critical_section(kernel, memory, thread_id, raw_arg(args, 0));
            Some(CoredllValue::U32(0))
        }
        ORD_ENTER_CRITICAL_SECTION => {
            enter_critical_section(kernel, memory, thread_id, raw_arg(args, 0), false);
            Some(CoredllValue::U32(0))
        }
        ORD_LEAVE_CRITICAL_SECTION => {
            leave_critical_section(kernel, memory, thread_id, raw_arg(args, 0));
            Some(CoredllValue::U32(0))
        }
        ORD_TRY_ENTER_CRITICAL_SECTION => Some(CoredllValue::Bool(enter_critical_section(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            true,
        ))),
        ORD_INTERLOCKED_TEST_EXCHANGE => Some(CoredllValue::U32(interlocked_compare_store(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 2),
            raw_arg(args, 1),
        ))),
        ORD_INTERLOCKED_INCREMENT => Some(CoredllValue::U32(interlocked_update(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            |value| value.wrapping_add(1),
            InterlockedReturn::NewValue,
        ))),
        ORD_INTERLOCKED_DECREMENT => Some(CoredllValue::U32(interlocked_update(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            |value| value.wrapping_sub(1),
            InterlockedReturn::NewValue,
        ))),
        ORD_INTERLOCKED_EXCHANGE => Some(CoredllValue::U32(interlocked_update(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            |_| raw_arg(args, 1),
            InterlockedReturn::OldValue,
        ))),
        ORD_INTERLOCKED_EXCHANGE_ADD => Some(CoredllValue::U32(interlocked_update(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            |value| value.wrapping_add(raw_arg(args, 1)),
            InterlockedReturn::OldValue,
        ))),
        ORD_INTERLOCKED_COMPARE_EXCHANGE => Some(CoredllValue::U32(interlocked_compare_store(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_TLS_GET_VALUE => Some(CoredllValue::U32(
            kernel
                .threads
                .tls_get_value(thread_id, raw_arg(args, 0))
                .unwrap_or(0),
        )),
        ORD_TLS_SET_VALUE => Some(CoredllValue::Bool(kernel.threads.tls_set_value(
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_TLS_CALL => Some(CoredllValue::U32(kernel.threads.tls_call(
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_GET_LAST_ERROR => Some(CoredllValue::U32(kernel.threads.get_last_error(thread_id))),
        ORD_SET_LAST_ERROR => {
            kernel.threads.set_last_error(thread_id, raw_arg(args, 0));
            Some(CoredllValue::U32(0))
        }
        ORD_GET_TICK_COUNT => Some(CoredllValue::U32(kernel.timers.tick_count())),
        ORD_GET_SYSTEM_TIME | ORD_GET_LOCAL_TIME => {
            write_current_system_time(kernel, memory, thread_id, raw_arg(args, 0));
            Some(CoredllValue::U32(0))
        }
        ORD_GET_SYSTEM_TIME_AS_FILE_TIME => {
            write_current_file_time(kernel, memory, thread_id, raw_arg(args, 0));
            Some(CoredllValue::U32(0))
        }
        ORD_GET_TIME_ZONE_INFORMATION => Some(CoredllValue::U32(write_time_zone_information(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_INPUT_DEBUG_CHAR_W => Some(CoredllValue::U32(input_debug_char_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_QUERY_PERFORMANCE_FREQUENCY => {
            Some(CoredllValue::Bool(write_performance_counter_value(
                kernel,
                memory,
                thread_id,
                raw_arg(args, 0),
                kernel.timers.performance_frequency(),
            )))
        }
        ORD_QUERY_PERFORMANCE_COUNTER => Some(CoredllValue::Bool(write_performance_counter_value(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            kernel.timers.performance_counter(),
        ))),
        ORD_SLEEP => {
            match crate::ce::timer::ce_sleep_request(raw_arg(args, 0)) {
                crate::ce::timer::CeSleepRequest::Yield => kernel.record_thread_yield(),
                crate::ce::timer::CeSleepRequest::Bounded(ms) => kernel.timers.sleep_ms(ms),
                crate::ce::timer::CeSleepRequest::Suspend => {
                    let _ = kernel.suspend_thread_for_handle(
                        crate::ce::kernel::CE_CURRENT_THREAD_PSEUDO_HANDLE,
                        thread_id,
                    );
                }
            }
            Some(CoredllValue::U32(0))
        }
        ORD_SLEEP_TILL_TICK => {
            kernel.timers.sleep_ms(1);
            Some(CoredllValue::U32(0))
        }
        ORD_SET_TIMER => Some(CoredllValue::U32(kernel.set_timer(
            (raw_arg(args, 0) != 0).then_some(raw_arg(args, 0)),
            (raw_arg(args, 1) != 0).then_some(raw_arg(args, 1)),
            raw_arg(args, 2),
        ))),
        ORD_KILL_TIMER => Some(CoredllValue::Bool(kernel.kill_timer(raw_arg(args, 1)))),
        ORD_CREATE_THREAD => Some(CoredllValue::Handle(create_thread_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_SET_THREAD_PRIORITY => Some(CoredllValue::Bool(set_thread_priority_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            false,
        ))),
        ORD_CE_SET_THREAD_PRIORITY => Some(CoredllValue::Bool(set_thread_priority_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            true,
        ))),
        ORD_GET_THREAD_PRIORITY => Some(CoredllValue::U32(get_thread_priority_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            false,
        ))),
        ORD_CE_GET_THREAD_PRIORITY => Some(CoredllValue::U32(get_thread_priority_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            true,
        ))),
        ORD_GLOBAL_MEMORY_STATUS => {
            write_global_memory_status(kernel, memory, thread_id, raw_arg(args, 0));
            Some(CoredllValue::U32(0))
        }
        ORD_SYSTEM_PARAMETERS_INFO_W => Some(CoredllValue::Bool(system_parameters_info_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_GET_STORE_INFORMATION => Some(CoredllValue::Bool(write_store_information(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_GET_SYSTEM_INFO => {
            write_system_info(kernel, memory, thread_id, raw_arg(args, 0));
            Some(CoredllValue::U32(0))
        }
        ORD_GET_VERSION_EX | ORD_GET_VERSION_EX_W => Some(CoredllValue::Bool(
            write_os_version_info_w(kernel, memory, thread_id, raw_arg(args, 0)),
        )),
        ORD_GET_SYSTEM_METRICS => Some(CoredllValue::U32(
            kernel.gwe.system_metric(raw_arg(args, 0)) as u32,
        )),
        ORD_GET_SYS_COLOR => Some(CoredllValue::U32(get_sys_color(raw_arg(args, 0)))),
        ORD_GET_SYS_COLOR_BRUSH => {
            Some(CoredllValue::Handle(get_sys_color_brush(raw_arg(args, 0))))
        }
        ORD_SET_SYS_COLORS => Some(CoredllValue::Bool(true)),
        ORD_ADJUST_WINDOW_RECT_EX => Some(CoredllValue::Bool(adjust_window_rect_ex_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_COPY_RECT => Some(CoredllValue::Bool(copy_rect_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_EQUAL_RECT => Some(CoredllValue::Bool(equal_rect_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_INFLATE_RECT => Some(CoredllValue::Bool(inflate_rect_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1) as i32,
            raw_arg(args, 2) as i32,
        ))),
        ORD_INTERSECT_RECT => Some(CoredllValue::Bool(intersect_rect_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_IS_RECT_EMPTY => Some(CoredllValue::Bool(is_rect_empty_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_OFFSET_RECT => Some(CoredllValue::Bool(offset_rect_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1) as i32,
            raw_arg(args, 2) as i32,
        ))),
        ORD_PT_IN_RECT => Some(CoredllValue::Bool(pt_in_rect_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            Point {
                x: raw_arg(args, 1) as i32,
                y: raw_arg(args, 2) as i32,
            },
        ))),
        ORD_SET_RECT => Some(CoredllValue::Bool(set_rect_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            Rect {
                left: raw_arg(args, 1) as i32,
                top: raw_arg(args, 2) as i32,
                right: raw_arg(args, 3) as i32,
                bottom: raw_arg(args, 4) as i32,
            },
        ))),
        ORD_SET_RECT_EMPTY => Some(CoredllValue::Bool(set_rect_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            Rect::default(),
        ))),
        ORD_UNION_RECT => Some(CoredllValue::Bool(union_rect_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_EVENT_MODIFY => {
            let ok = match raw_arg(args, 1) {
                EVENT_PULSE => {
                    kernel.set_event(raw_arg(args, 0)) && kernel.reset_event(raw_arg(args, 0))
                }
                EVENT_RESET => kernel.reset_event(raw_arg(args, 0)),
                EVENT_SET => kernel.set_event(raw_arg(args, 0)),
                _ => {
                    kernel
                        .threads
                        .set_last_error(thread_id, ERROR_INVALID_HANDLE);
                    false
                }
            };
            if !ok {
                kernel
                    .threads
                    .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            }
            Some(CoredllValue::Bool(ok))
        }
        ORD_CREATE_EVENT_W => Some(CoredllValue::Handle(create_event_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_OPEN_EVENT_W => Some(CoredllValue::Handle(open_event_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_SUSPEND_THREAD => Some(CoredllValue::U32(suspend_thread_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_RESUME_THREAD => Some(CoredllValue::U32(resume_thread_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_GET_THREAD_ID => Some(CoredllValue::U32(get_thread_id_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_GET_EXIT_CODE_THREAD => Some(CoredllValue::Bool(get_thread_exit_code_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_GET_EXIT_CODE_PROCESS => Some(CoredllValue::Bool(get_process_exit_code_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_GET_PROCESS_VERSION => Some(CoredllValue::U32(get_process_version_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_GET_PROCESS_ID => Some(CoredllValue::U32(get_process_id_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_GET_THREAD_TIMES => Some(CoredllValue::Bool(get_thread_times_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_TERMINATE_PROCESS => Some(CoredllValue::Bool(terminate_process_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_CREATE_MUTEX_W => Some(CoredllValue::Handle(create_mutex_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_RELEASE_MUTEX => Some(CoredllValue::Bool(release_mutex_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_CREATE_SEMAPHORE_W => Some(CoredllValue::Handle(create_semaphore_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_RELEASE_SEMAPHORE => Some(CoredllValue::Bool(release_semaphore_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_WAIT_FOR_SINGLE_OBJECT => Some(CoredllValue::U32(kernel.wait_for_single_object(
            raw_arg(args, 0),
            raw_arg(args, 1),
            thread_id,
        ))),
        ORD_WAIT_FOR_MULTIPLE_OBJECTS => Some(CoredllValue::U32(wait_for_multiple_objects_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_MSG_WAIT_FOR_MULTIPLE_OBJECTS_EX => Some(CoredllValue::U32(
            msg_wait_for_multiple_objects_ex_raw(kernel, memory, thread_id, args),
        )),
        ORD_ACOS => Some(raw_unary_f64(kernel, args, CeMathUnaryF64::Acos)),
        ORD_ASIN => Some(raw_unary_f64(kernel, args, CeMathUnaryF64::Asin)),
        ORD_ATAN => Some(raw_unary_f64(kernel, args, CeMathUnaryF64::Atan)),
        ORD_ATAN2 => Some(raw_binary_f64(kernel, args, CeMathBinaryF64::Atan2)),
        ORD_CEIL => Some(raw_unary_f64(kernel, args, CeMathUnaryF64::Ceil)),
        ORD_COS => Some(raw_unary_f64(kernel, args, CeMathUnaryF64::Cos)),
        ORD_COSH => Some(raw_unary_f64(kernel, args, CeMathUnaryF64::Cosh)),
        ORD_EXP => Some(raw_unary_f64(kernel, args, CeMathUnaryF64::Exp)),
        ORD_FABS => Some(raw_unary_f64(kernel, args, CeMathUnaryF64::Fabs)),
        ORD_FLOOR => Some(raw_unary_f64(kernel, args, CeMathUnaryF64::Floor)),
        ORD_FMOD => Some(raw_binary_f64(kernel, args, CeMathBinaryF64::Fmod)),
        ORD_FMODF => Some(raw_binary_f32(kernel, args, CeMathBinaryF32::Fmod)),
        ORD_LOG => Some(raw_unary_f64(kernel, args, CeMathUnaryF64::Log)),
        ORD_LOG10 => Some(raw_unary_f64(kernel, args, CeMathUnaryF64::Log10)),
        ORD_POW => Some(raw_binary_f64(kernel, args, CeMathBinaryF64::Pow)),
        ORD_SIN => Some(raw_unary_f64(kernel, args, CeMathUnaryF64::Sin)),
        ORD_SINH => Some(raw_unary_f64(kernel, args, CeMathUnaryF64::Sinh)),
        ORD_SQRT => Some(raw_unary_f64(kernel, args, CeMathUnaryF64::Sqrt)),
        ORD_TAN => Some(raw_unary_f64(kernel, args, CeMathUnaryF64::Tan)),
        ORD_TANH => Some(raw_unary_f64(kernel, args, CeMathUnaryF64::Tanh)),
        ORD_CREATE_FILE_W => Some(CoredllValue::Handle(create_file_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_GET_COMM_STATE => Some(CoredllValue::Bool(get_comm_state_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_SET_COMM_STATE | ORD_SET_COMM_TIMEOUTS => {
            Some(CoredllValue::Bool(comm_handle_and_ptr_raw(
                kernel,
                memory,
                thread_id,
                raw_arg(args, 0),
                raw_arg(args, 1),
            )))
        }
        ORD_GET_COMM_TIMEOUTS => Some(CoredllValue::Bool(get_comm_timeouts_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_SETUP_COMM
        | ORD_PURGE_COMM
        | ORD_CLEAR_COMM_BREAK
        | ORD_SET_COMM_BREAK
        | ORD_ESCAPE_COMM_FUNCTION
        | ORD_SET_COMM_MASK => Some(CoredllValue::Bool(comm_handle_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_CLEAR_COMM_ERROR => Some(CoredllValue::Bool(clear_comm_error_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_GET_COMM_MASK | ORD_GET_COMM_MODEM_STATUS | ORD_WAIT_COMM_EVENT => {
            Some(CoredllValue::Bool(comm_out_u32_zero_raw(
                kernel,
                memory,
                thread_id,
                raw_arg(args, 0),
                raw_arg(args, 1),
            )))
        }
        ORD_FIND_FIRST_FILE_W => Some(CoredllValue::Handle(find_first_file_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_FIND_NEXT_FILE_W => Some(CoredllValue::Bool(find_next_file_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_CREATE_DIRECTORY_W => Some(CoredllValue::Bool(path_bool_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            CeKernel::create_directory_w,
        ))),
        ORD_REMOVE_DIRECTORY_W => Some(CoredllValue::Bool(path_bool_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            CeKernel::remove_directory_w,
        ))),
        ORD_DELETE_FILE_W => Some(CoredllValue::Bool(path_bool_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            CeKernel::delete_file_w,
        ))),
        ORD_MOVE_FILE_W => Some(CoredllValue::Bool(move_file_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_COPY_FILE_W => Some(CoredllValue::Bool(copy_file_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_SET_FILE_ATTRIBUTES_W => Some(CoredllValue::Bool(set_file_attributes_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_GET_FILE_ATTRIBUTES_W => Some(CoredllValue::U32(get_file_attributes_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_GET_FILE_ATTRIBUTES_EX_W => Some(CoredllValue::Bool(get_file_attributes_ex_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_FIND_CLOSE => Some(CoredllValue::Bool(find_close_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_READ_FILE => Some(CoredllValue::Bool(read_file_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_WRITE_FILE => Some(CoredllValue::Bool(write_file_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_DEVICE_IO_CONTROL => Some(CoredllValue::Bool(device_io_control_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_KERNEL_IO_CONTROL => Some(CoredllValue::Bool(kernel_io_control_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_SET_FILE_POINTER => Some(CoredllValue::U32(set_file_pointer_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_GET_FILE_SIZE => Some(CoredllValue::U32(get_file_size_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_SET_FILE_TIME => Some(CoredllValue::Bool(file_handle_bool_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_SYSTEM_TIME_TO_FILE_TIME => Some(CoredllValue::Bool(system_time_to_file_time_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_FILE_TIME_TO_SYSTEM_TIME => Some(CoredllValue::Bool(file_time_to_system_time_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_FLUSH_FILE_BUFFERS => Some(CoredllValue::Bool(flush_file_buffers_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_CREATE_PROCESS_W => Some(CoredllValue::Bool(create_process_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_REG_CLOSE_KEY => Some(CoredllValue::U32(
            kernel.registry.reg_close_key(raw_arg(args, 0)),
        )),
        ORD_REG_CREATE_KEY_EX_W => Some(CoredllValue::U32(reg_create_key_ex_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_REG_DELETE_KEY_W => Some(CoredllValue::U32(reg_delete_key_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_REG_DELETE_VALUE_W => Some(CoredllValue::U32(reg_delete_value_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_REG_ENUM_KEY_EX_W => Some(CoredllValue::U32(reg_enum_key_ex_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_REG_ENUM_VALUE_W => Some(CoredllValue::U32(reg_enum_value_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_REG_OPEN_KEY_EX_W => Some(CoredllValue::U32(reg_open_key_ex_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_REG_QUERY_INFO_KEY_W => Some(CoredllValue::U32(reg_query_info_key_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_REG_QUERY_VALUE_EX_W => Some(CoredllValue::U32(reg_query_value_ex_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_REG_SET_VALUE_EX_W => Some(CoredllValue::U32(reg_set_value_ex_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_CLOSE_HANDLE => Some(CoredllValue::Bool(
            kernel.close_handle(raw_arg(args, 0)).unwrap_or(false),
        )),
        ORD_GET_PROCESS_HEAP => Some(CoredllValue::Handle(kernel.memory.get_process_heap())),
        ORD_GET_MODULE_FILE_NAME_W => Some(CoredllValue::U32(get_module_file_name_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_GET_MODULE_HANDLE_W => Some(CoredllValue::Handle(get_module_handle_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_LOAD_LIBRARY_W | ORD_LOAD_LIBRARY_EX_W => Some(CoredllValue::Handle(
            load_library_w_raw(kernel, memory, thread_id, raw_arg(args, 0)),
        )),
        ORD_FREE_LIBRARY => Some(CoredllValue::Bool(free_library_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_GET_PROC_ADDRESS_W => Some(CoredllValue::Handle(get_proc_address_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_GET_PROC_ADDRESS_A => Some(CoredllValue::Handle(get_proc_address_a_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_WCSRCHR => Some(CoredllValue::Handle(crt::wcsrchr_raw(
            memory,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_WCSSTR => Some(CoredllValue::Handle(crt::wcsstr_raw(
            memory,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_WCSCHR => Some(CoredllValue::Handle(crt::wcschr_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_ISWCTYPE => Some(CoredllValue::U32(iswctype_raw(
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_WCSDUP => Some(CoredllValue::Handle(crt::wcsdup_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_WTOL => Some(CoredllValue::U32(
            crt::wtol_raw(memory, raw_arg(args, 0)) as u32
        )),
        ORD_WCSICMP => {
            Some(CoredllValue::U32(
                crt::wcsicmp_raw(memory, raw_arg(args, 0), raw_arg(args, 1)) as u32,
            ))
        }
        ORD_WCSNICMP => Some(CoredllValue::U32(crt::wcsnicmp_raw(
            memory,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ) as u32)),
        ORD_WCSNCMP => Some(CoredllValue::U32(crt::wcsncmp_raw(
            memory,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ) as u32)),
        ORD_STRING_CCH_CAT_W => Some(CoredllValue::U32(string_cch_cat_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_STRING_CB_CAT_W => Some(CoredllValue::U32(string_cb_cat_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_STRING_CCH_LENGTH_W => Some(CoredllValue::U32(string_cch_length_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_WCSCPY => Some(CoredllValue::Handle(crt::wcscpy_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_WCSNCPY => Some(CoredllValue::Handle(crt::wcsncpy_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_MULTI_BYTE_TO_WIDE_CHAR => Some(CoredllValue::U32(multi_byte_to_wide_char_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_WIDE_CHAR_TO_MULTI_BYTE => Some(CoredllValue::U32(wide_char_to_multi_byte_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_CHAR_UPPER_W => Some(CoredllValue::Handle(char_case_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            WideCaseMode::Upper,
        ))),
        ORD_CHAR_LOWER_W => Some(CoredllValue::Handle(char_case_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            WideCaseMode::Lower,
        ))),
        ORD_CHAR_UPPER_BUFF_W => Some(CoredllValue::U32(char_case_buff_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            WideCaseMode::Upper,
        ))),
        ORD_CHAR_LOWER_BUFF_W => Some(CoredllValue::U32(char_case_buff_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            WideCaseMode::Lower,
        ))),
        ORD_MALLOC | ORD_OPERATOR_NEW | ORD_OPERATOR_NEW_ARRAY | ORD_OPERATOR_NEW_ARRAY_NOTHROW => {
            Some(CoredllValue::Handle(crt::malloc_raw(
                kernel,
                thread_id,
                raw_arg(args, 0),
            )))
        }
        ORD_MEMCPY | ORD_MEMMOVE => Some(CoredllValue::Handle(crt::memcpy_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_MEMSET => Some(CoredllValue::Handle(crt::memset_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_MEMCMP => Some(CoredllValue::U32(crt::memcmp_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ) as u32)),
        ORD_STRCPY => Some(CoredllValue::Handle(crt::strcpy_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_STRTOK => Some(CoredllValue::Handle(crt::strtok_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_STRTOUL => Some(CoredllValue::U32(crt::strtoul_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_WSPRINTF_W => Some(CoredllValue::U32(crt::wsprintf_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            if args.len() > 2 { &args[2..] } else { &[] },
        ))),
        ORD_SWPRINTF => Some(CoredllValue::U32(crt::swprintf_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            if args.len() > 2 { &args[2..] } else { &[] },
        ))),
        ORD_WVSPRINTF_W => Some(CoredllValue::U32(crt::wvsprintf_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_VSWPRINTF => Some(CoredllValue::U32(crt::vswprintf_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_SPRINTF => Some(CoredllValue::U32(crt::sprintf_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            if args.len() > 2 { &args[2..] } else { &[] },
        ))),
        ORD_PRINTF => Some(CoredllValue::U32(crt::printf_family_raw(kernel, thread_id))),
        ORD_ATOI => Some(CoredllValue::U32(
            crt::atoi_raw(memory, raw_arg(args, 0)) as u32
        )),
        ORD_FOPEN => Some(CoredllValue::Handle(crt::fopen_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_WFOPEN => Some(CoredllValue::Handle(crt::wfopen_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_FREAD => Some(CoredllValue::U32(crt::fread_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
            raw_arg(args, 3),
        ))),
        ORD_FGETS => Some(CoredllValue::Handle(crt::fgets_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_FWRITE => Some(CoredllValue::U32(crt::fwrite_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
            raw_arg(args, 3),
        ))),
        ORD_FSEEK => Some(CoredllValue::U32(crt::fseek_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_FTELL => Some(CoredllValue::U32(crt::ftell_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_FCLOSE => Some(CoredllValue::U32(crt::fclose_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_FFLUSH => Some(CoredllValue::U32(crt::fflush_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_FEOF => Some(CoredllValue::U32(crt::feof_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_FERROR => Some(CoredllValue::U32(crt::ferror_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_SRAND => Some(CoredllValue::U32(crt::srand_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_RAND => Some(CoredllValue::U32(crt::rand_raw(kernel, thread_id))),
        ORD_SECURITY_GEN_COOKIE2 => Some(CoredllValue::U32(security_gen_cookie2_raw(
            kernel, thread_id,
        ))),
        ORD_FREE
        | ORD_OPERATOR_DELETE
        | ORD_OPERATOR_DELETE_ARRAY
        | ORD_OPERATOR_DELETE_ARRAY_NOTHROW => {
            crt::free_raw(kernel, raw_arg(args, 0));
            Some(CoredllValue::U32(0))
        }
        ORD_LOCAL_ALLOC | ORD_LOCAL_ALLOC_TRACE => Some(CoredllValue::Handle(local_alloc_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_LOCAL_RE_ALLOC => Some(CoredllValue::Handle(local_re_alloc_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_LOCAL_SIZE => Some(CoredllValue::U32(local_size_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_LOCAL_FREE => Some(CoredllValue::Handle(local_free_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_REMOTE_LOCAL_ALLOC => Some(CoredllValue::Handle(local_alloc_raw(
            kernel,
            thread_id,
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_LOCAL_ALLOC_IN_PROCESS => Some(CoredllValue::Handle(local_alloc_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_REMOTE_LOCAL_RE_ALLOC => Some(CoredllValue::Handle(local_re_alloc_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 1),
            raw_arg(args, 2),
            raw_arg(args, 3),
        ))),
        ORD_REMOTE_LOCAL_SIZE => Some(CoredllValue::U32(local_size_raw(
            kernel,
            thread_id,
            raw_arg(args, 1),
        ))),
        ORD_LOCAL_SIZE_IN_PROCESS => Some(CoredllValue::U32(local_size_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_REMOTE_LOCAL_FREE => Some(CoredllValue::Handle(local_free_raw(
            kernel,
            thread_id,
            raw_arg(args, 1),
        ))),
        ORD_LOCAL_FREE_IN_PROCESS => Some(CoredllValue::Handle(local_free_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_HEAP_CREATE => Some(CoredllValue::Handle(heap_create_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_HEAP_DESTROY => Some(CoredllValue::Bool(heap_destroy_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_HEAP_ALLOC | ORD_HEAP_ALLOC_TRACE => Some(CoredllValue::Handle(heap_alloc_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_HEAP_RE_ALLOC => Some(CoredllValue::Handle(heap_re_alloc_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
            raw_arg(args, 3),
        ))),
        ORD_HEAP_SIZE => Some(CoredllValue::U32(heap_size_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_HEAP_FREE => Some(CoredllValue::Bool(heap_free_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_REMOTE_HEAP_ALLOC => Some(CoredllValue::Handle(heap_alloc_raw(
            kernel,
            thread_id,
            raw_arg(args, 1),
            raw_arg(args, 2),
            raw_arg(args, 3),
        ))),
        ORD_REMOTE_HEAP_RE_ALLOC => Some(CoredllValue::Handle(heap_re_alloc_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 1),
            raw_arg(args, 2),
            raw_arg(args, 3),
            raw_arg(args, 4),
        ))),
        ORD_REMOTE_HEAP_SIZE => Some(CoredllValue::U32(heap_size_raw(
            kernel,
            thread_id,
            raw_arg(args, 1),
            raw_arg(args, 2),
            raw_arg(args, 3),
        ))),
        ORD_REMOTE_HEAP_FREE => Some(CoredllValue::Bool(heap_free_raw(
            kernel,
            thread_id,
            raw_arg(args, 1),
            raw_arg(args, 2),
            raw_arg(args, 3),
        ))),
        ORD_HEAP_VALIDATE => Some(CoredllValue::Bool(kernel.memory.heap_validate(
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_IS_BAD_READ_PTR => Some(CoredllValue::Bool(is_bad_ptr_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            PointerProbe::Read,
        ))),
        ORD_IS_BAD_WRITE_PTR => Some(CoredllValue::Bool(is_bad_ptr_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            PointerProbe::Write,
        ))),
        ORD_VIRTUAL_ALLOC => Some(CoredllValue::Handle(virtual_alloc_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
            raw_arg(args, 3),
        ))),
        ORD_VIRTUAL_FREE => Some(CoredllValue::Bool(virtual_free_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_FLUSH_INSTRUCTION_CACHE => Some(CoredllValue::Bool(true)),
        ORD_CREATE_FILE_MAPPING_W => Some(CoredllValue::Handle(create_file_mapping_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_MAP_VIEW_OF_FILE => Some(CoredllValue::Handle(map_view_of_file_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 2),
            raw_arg(args, 3),
            raw_arg(args, 4),
        ))),
        ORD_FLUSH_VIEW_OF_FILE | ORD_FLUSH_VIEW_OF_FILE_MAYBE => {
            Some(CoredllValue::Bool(flush_view_of_file_raw(
                kernel,
                memory,
                thread_id,
                raw_arg(args, 0),
                raw_arg(args, 1),
            )))
        }
        ORD_UNMAP_VIEW_OF_FILE => Some(CoredllValue::Bool(unmap_view_of_file_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_REGISTER_CLASS_W => Some(CoredllValue::U32(register_class_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_GET_CLASS_INFO_W => Some(CoredllValue::Bool(get_class_info_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_REGISTER_WINDOW_MESSAGE_W => Some(CoredllValue::U32(register_window_message_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_REGISTER_GESTURE => Some(CoredllValue::Handle(register_gesture_raw(
            kernel, thread_id, args,
        ))),
        ORD_FIND_WINDOW_W => Some(CoredllValue::Handle(find_window_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_CREATE_WINDOW_EX_W => Some(CoredllValue::Handle(create_window_ex_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_DESTROY_WINDOW => Some(CoredllValue::Bool(kernel.destroy_window(raw_arg(args, 0)))),
        ORD_SHOW_WINDOW => Some(CoredllValue::Bool(show_window_raw(kernel, args))),
        ORD_UPDATE_WINDOW => Some(CoredllValue::Bool(kernel.update_window(raw_arg(args, 0)))),
        ORD_INVALIDATE_RECT => Some(CoredllValue::Bool(invalidate_rect_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_REDRAW_WINDOW => Some(CoredllValue::Bool(redraw_window_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_VALIDATE_RECT => Some(CoredllValue::Bool(validate_rect_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_GET_UPDATE_RECT => Some(CoredllValue::Bool(get_update_rect_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2) != 0,
        ))),
        ORD_GET_UPDATE_RGN => Some(CoredllValue::U32(get_update_rgn_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2) != 0,
        ))),
        ORD_BEGIN_PAINT => Some(CoredllValue::Handle(begin_paint_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_END_PAINT => Some(CoredllValue::Bool(end_paint_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_GET_DC => Some(CoredllValue::Handle(get_dc_raw(kernel, raw_arg(args, 0)))),
        ORD_RELEASE_DC => Some(CoredllValue::U32(release_dc_raw(
            kernel,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_GET_DEVICE_CAPS => Some(CoredllValue::U32(get_device_caps_raw(
            kernel,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_CREATE_COMPATIBLE_DC => Some(CoredllValue::Handle(create_compatible_dc_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_DELETE_DC => Some(CoredllValue::Bool(delete_dc_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_CREATE_COMPATIBLE_BITMAP => Some(CoredllValue::Handle(create_compatible_bitmap_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_i32_arg(args, 1),
            raw_i32_arg(args, 2),
        ))),
        ORD_CREATE_BITMAP => Some(CoredllValue::Handle(create_bitmap_raw(
            kernel,
            thread_id,
            raw_i32_arg(args, 0),
            raw_i32_arg(args, 1),
            raw_arg(args, 2) as u16,
            raw_arg(args, 3) as u16,
            raw_arg(args, 4),
        ))),
        ORD_CREATE_DIBSECTION => Some(CoredllValue::Handle(create_dib_section_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_CREATE_FONT_INDIRECT_W => Some(CoredllValue::Handle(create_font_indirect_w_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_CREATE_SOLID_BRUSH => Some(CoredllValue::Handle(create_solid_brush_raw(
            kernel,
            raw_arg(args, 0),
        ))),
        ORD_CREATE_PATTERN_BRUSH => Some(CoredllValue::Handle(create_pattern_brush_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_CREATE_PEN => Some(CoredllValue::Handle(create_pen_raw(
            kernel,
            raw_arg(args, 0),
            raw_i32_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_CREATE_PEN_INDIRECT => Some(CoredllValue::Handle(create_pen_indirect_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_CREATE_PALETTE => Some(CoredllValue::Handle(create_palette_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_GET_NEAREST_PALETTE_INDEX => Some(CoredllValue::U32(get_nearest_palette_index_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_GET_PALETTE_ENTRIES => Some(CoredllValue::U32(get_palette_entries_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
            raw_arg(args, 3),
        ))),
        ORD_GET_SYSTEM_PALETTE_ENTRIES => Some(CoredllValue::U32(get_system_palette_entries_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
            raw_arg(args, 3),
        ))),
        ORD_SET_PALETTE_ENTRIES => Some(CoredllValue::U32(set_palette_entries_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
            raw_arg(args, 3),
        ))),
        ORD_REALIZE_PALETTE => Some(CoredllValue::U32(realize_palette_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_SELECT_PALETTE => Some(CoredllValue::Handle(select_palette_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_GET_STOCK_OBJECT => Some(CoredllValue::Handle(get_stock_object_raw(raw_arg(args, 0)))),
        ORD_SELECT_OBJECT => Some(CoredllValue::Handle(select_object_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_SET_BK_MODE => Some(CoredllValue::U32(set_bk_mode_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_i32_arg(args, 1),
        ))),
        ORD_SET_BK_COLOR => Some(CoredllValue::U32(set_bk_color_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_SET_TEXT_COLOR => Some(CoredllValue::U32(set_text_color_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_SET_TEXT_ALIGN => Some(CoredllValue::U32(set_text_align_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_FILL_RECT => Some(CoredllValue::U32(fill_rect_raw(
            kernel,
            memory,
            framebuffer,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_BIT_BLT => Some(CoredllValue::Bool(bit_blt_raw(
            kernel,
            memory,
            framebuffer,
            thread_id,
            args,
        ))),
        ORD_STRETCH_BLT => Some(CoredllValue::Bool(bit_blt_raw(
            kernel,
            memory,
            framebuffer,
            thread_id,
            args,
        ))),
        ORD_STRETCH_DIBITS => Some(CoredllValue::U32(stretch_dibits_raw(
            kernel,
            memory,
            framebuffer,
            thread_id,
            args,
        ))),
        ORD_SET_DIBITS_TO_DEVICE => Some(CoredllValue::U32(set_dibits_to_device_raw(
            kernel,
            memory,
            framebuffer,
            thread_id,
            args,
        ))),
        ORD_GET_DIBCOLOR_TABLE => Some(CoredllValue::U32(get_dib_color_table_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
            raw_arg(args, 3),
        ))),
        ORD_SET_DIBCOLOR_TABLE => Some(CoredllValue::U32(set_dib_color_table_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
            raw_arg(args, 3),
        ))),
        ORD_SET_BITMAP_BITS => Some(CoredllValue::U32(set_bitmap_bits_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_GET_PIXEL => Some(CoredllValue::U32(get_pixel_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_PAT_BLT => Some(CoredllValue::Bool(pat_blt_raw(kernel, thread_id, args))),
        ORD_TRANSPARENT_IMAGE => Some(CoredllValue::Bool(transparent_image_raw(
            kernel,
            memory,
            framebuffer,
            thread_id,
            args,
        ))),
        ORD_SET_BRUSH_ORG_EX => Some(CoredllValue::Bool(set_brush_org_ex_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_POLYLINE => Some(CoredllValue::Bool(polyline_raw(
            kernel,
            memory,
            framebuffer,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_i32_arg(args, 2),
        ))),
        ORD_RECTANGLE | ORD_ROUND_RECT | ORD_ELLIPSE | ORD_POLYGON => {
            Some(CoredllValue::Bool(gdi_shape_raw(kernel, thread_id, args)))
        }
        ORD_MOVE_TO_EX => Some(CoredllValue::Bool(move_to_ex_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_LINE_TO => Some(CoredllValue::Bool(line_to_raw(
            kernel,
            framebuffer,
            thread_id,
            args,
        ))),
        ORD_DRAW_TEXT_W => Some(CoredllValue::U32(draw_text_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_i32_arg(args, 2),
            raw_arg(args, 3),
        ))),
        ORD_EXT_TEXT_OUT_W => Some(CoredllValue::Bool(ext_text_out_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_CREATE_RECT_RGN => Some(CoredllValue::Handle(create_rect_rgn_raw(
            kernel,
            raw_i32_arg(args, 0),
            raw_i32_arg(args, 1),
            raw_i32_arg(args, 2),
            raw_i32_arg(args, 3),
        ))),
        ORD_CREATE_RECT_RGN_INDIRECT => Some(CoredllValue::Handle(create_rect_rgn_indirect_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_SET_RECT_RGN => Some(CoredllValue::Bool(set_rect_rgn_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_i32_arg(args, 1),
            raw_i32_arg(args, 2),
            raw_i32_arg(args, 3),
            raw_i32_arg(args, 4),
        ))),
        ORD_COMBINE_RGN => Some(CoredllValue::U32(combine_rgn_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
            raw_arg(args, 3),
        ))),
        ORD_SELECT_CLIP_RGN => Some(CoredllValue::U32(select_clip_rgn_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_GET_CLIP_BOX => Some(CoredllValue::U32(get_clip_box_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_GET_RGN_BOX => Some(CoredllValue::U32(get_rgn_box_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_PT_IN_REGION => Some(CoredllValue::Bool(pt_in_region_raw(
            kernel,
            raw_arg(args, 0),
            raw_i32_arg(args, 1),
            raw_i32_arg(args, 2),
        ))),
        ORD_RECT_IN_REGION => Some(CoredllValue::Bool(rect_in_region_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_SET_WINDOW_RGN => Some(CoredllValue::U32(set_window_rgn_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_GET_WINDOW_RGN => Some(CoredllValue::U32(get_window_rgn_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_ENABLE_WINDOW => Some(CoredllValue::Bool(
            kernel
                .enable_window(raw_arg(args, 0), raw_arg(args, 1) != 0)
                .unwrap_or(false),
        )),
        ORD_IS_WINDOW => Some(CoredllValue::Bool(kernel.gwe.is_window(raw_arg(args, 0)))),
        ORD_IS_WINDOW_ENABLED => Some(CoredllValue::Bool(
            kernel.gwe.is_window_enabled(raw_arg(args, 0)),
        )),
        ORD_IS_WINDOW_VISIBLE => Some(CoredllValue::Bool(
            kernel.gwe.is_window_visible(raw_arg(args, 0)),
        )),
        ORD_GET_PARENT => Some(CoredllValue::Handle(
            kernel.gwe.get_parent(raw_arg(args, 0)).unwrap_or(0),
        )),
        ORD_IS_CHILD => Some(CoredllValue::Bool(
            kernel.gwe.is_child(raw_arg(args, 0), raw_arg(args, 1)),
        )),
        ORD_GET_WINDOW_THREAD_PROCESS_ID => Some(CoredllValue::U32(
            get_window_thread_process_id_raw(kernel, memory, thread_id, args),
        )),
        ORD_GET_DLG_ITEM => Some(CoredllValue::Handle(
            kernel
                .gwe
                .get_dlg_item(raw_arg(args, 0), raw_arg(args, 1))
                .unwrap_or(0),
        )),
        ORD_GET_DLG_CTRL_ID => Some(CoredllValue::U32(
            kernel.gwe.get_dlg_ctrl_id(raw_arg(args, 0)).unwrap_or(0),
        )),
        ORD_SET_MENU => Some(CoredllValue::Bool(set_menu_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_GET_MENU => Some(CoredllValue::Handle(get_menu_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_SET_ASSOCIATED_MENU => Some(CoredllValue::U32(set_associated_menu_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_GET_ASSOCIATED_MENU => Some(CoredllValue::Handle(get_menu_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_DRAW_MENU_BAR => Some(CoredllValue::Bool(draw_menu_bar_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_GET_NEXT_DLG_TAB_ITEM => Some(CoredllValue::Handle(get_next_dlg_tab_item_raw(
            kernel, thread_id, args,
        ))),
        ORD_GET_NEXT_DLG_GROUP_ITEM => Some(CoredllValue::Handle(get_next_dlg_group_item_raw(
            kernel, thread_id, args,
        ))),
        ORD_GET_DIALOG_BASE_UNITS => Some(CoredllValue::U32(get_dialog_base_units_raw())),
        ORD_MAP_DIALOG_RECT => Some(CoredllValue::Bool(map_dialog_rect_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_CREATE_DIALOG_INDIRECT_PARAM_W | ORD_DIALOG_BOX_INDIRECT_PARAM_W => {
            Some(CoredllValue::Handle(create_dialog_indirect_param_w_raw(
                kernel, memory, thread_id, args,
            )))
        }
        ORD_END_DIALOG => Some(CoredllValue::Bool(end_dialog_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_DEF_DLG_PROC_W => Some(CoredllValue::U32(0)),
        ORD_SET_DLG_ITEM_TEXT_W => Some(CoredllValue::Bool(set_dlg_item_text_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_GET_DLG_ITEM_TEXT_W => Some(CoredllValue::U32(get_dlg_item_text_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_SET_DLG_ITEM_INT => Some(CoredllValue::Bool(set_dlg_item_int_raw(
            kernel, thread_id, args,
        ))),
        ORD_GET_DLG_ITEM_INT => Some(CoredllValue::U32(get_dlg_item_int_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_SEND_DLG_ITEM_MESSAGE_W => Some(CoredllValue::U32(send_dlg_item_message_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_CHECK_RADIO_BUTTON => Some(CoredllValue::Bool(check_radio_button_raw(
            kernel, thread_id, args,
        ))),
        ORD_IS_DIALOG_MESSAGE_W => Some(CoredllValue::Bool(is_dialog_message_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_SET_PARENT => Some(CoredllValue::Handle(set_parent_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_GET_WINDOW => Some(CoredllValue::Handle(get_window_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_GET_DESKTOP_WINDOW => Some(CoredllValue::Handle(kernel.gwe.get_desktop_window())),
        ORD_GET_FOREGROUND_WINDOW => Some(CoredllValue::Handle(
            kernel.gwe.get_active_window().unwrap_or(0),
        )),
        ORD_SET_FOREGROUND_WINDOW => Some(CoredllValue::Bool(set_foreground_window_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_SET_ACTIVE_WINDOW => Some(CoredllValue::Handle(set_active_window_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_GET_ACTIVE_WINDOW => Some(CoredllValue::Handle(
            kernel.gwe.get_active_window().unwrap_or(0),
        )),
        ORD_LOAD_CURSOR_W => Some(CoredllValue::Handle(load_cursor_w_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_LOAD_ICON_W => Some(CoredllValue::Handle(load_icon_w_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_DESTROY_ICON => Some(CoredllValue::Bool(destroy_icon_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_SET_CURSOR => Some(CoredllValue::Handle(
            kernel.gwe.set_cursor(raw_arg(args, 0)).unwrap_or(0),
        )),
        ORD_GET_CURSOR => Some(CoredllValue::Handle(kernel.gwe.get_cursor().unwrap_or(0))),
        ORD_GET_CURSOR_POS => {
            let cursor_pos = kernel.gwe.get_cursor_pos();
            Some(CoredllValue::Bool(write_guest_point(
                kernel,
                memory,
                thread_id,
                raw_arg(args, 0),
                cursor_pos,
            )))
        }
        ORD_SET_FOCUS => Some(CoredllValue::Handle(
            set_focus_raw(kernel, thread_id, raw_arg(args, 0)).unwrap_or(0),
        )),
        ORD_GET_FOCUS => Some(CoredllValue::Handle(kernel.gwe.get_focus().unwrap_or(0))),
        ORD_SET_CAPTURE => Some(CoredllValue::Handle(
            kernel.gwe.set_capture(raw_arg(args, 0)).unwrap_or(0),
        )),
        ORD_GET_CAPTURE => Some(CoredllValue::Handle(kernel.gwe.get_capture().unwrap_or(0))),
        ORD_RELEASE_CAPTURE => Some(CoredllValue::Bool(kernel.gwe.release_capture())),
        ORD_SET_WINDOW_TEXT_W => Some(CoredllValue::Bool(set_window_text_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_GET_WINDOW_TEXT_W | ORD_GET_WINDOW_TEXT_WDIRECT => Some(CoredllValue::U32(
            get_window_text_w_raw(kernel, memory, thread_id, args),
        )),
        ORD_GET_WINDOW_TEXT_LENGTH_W => Some(CoredllValue::U32(
            kernel
                .gwe
                .get_window_text_length(raw_arg(args, 0))
                .unwrap_or(0) as u32,
        )),
        ORD_GET_CLASS_NAME_W => Some(CoredllValue::U32(get_class_name_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_SET_WINDOW_LONG_W => Some(CoredllValue::U32(set_window_long_w_raw(kernel, args))),
        ORD_GET_WINDOW_LONG_W => Some(CoredllValue::U32(get_window_long_w_raw(kernel, args))),
        ORD_BRING_WINDOW_TO_TOP => Some(CoredllValue::Bool(
            kernel.bring_window_to_top(raw_arg(args, 0)),
        )),
        ORD_SET_WINDOW_POS => Some(CoredllValue::Bool(kernel.set_window_pos(
            raw_arg(args, 0),
            Some(raw_arg(args, 1)),
            raw_i32_arg(args, 2),
            raw_i32_arg(args, 3),
            raw_i32_arg(args, 4),
            raw_i32_arg(args, 5),
            raw_arg(args, 6),
        ))),
        ORD_MOVE_WINDOW => Some(CoredllValue::Bool(kernel.move_window(
            raw_arg(args, 0),
            raw_i32_arg(args, 1),
            raw_i32_arg(args, 2),
            raw_i32_arg(args, 3),
            raw_i32_arg(args, 4),
            raw_arg(args, 5) != 0,
        ))),
        ORD_GET_WINDOW_RECT => Some(CoredllValue::Bool(write_window_rect(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            WindowRectKind::Window,
        ))),
        ORD_GET_CLIENT_RECT => Some(CoredllValue::Bool(write_window_rect(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            WindowRectKind::Client,
        ))),
        ORD_CLIENT_TO_SCREEN => Some(CoredllValue::Bool(map_single_point(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            PointMapKind::ClientToScreen,
        ))),
        ORD_SCREEN_TO_CLIENT => Some(CoredllValue::Bool(map_single_point(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            PointMapKind::ScreenToClient,
        ))),
        ORD_MAP_WINDOW_POINTS => Some(CoredllValue::U32(map_window_points(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
            raw_arg(args, 3),
        ))),
        ORD_WINDOW_FROM_POINT => Some(CoredllValue::Handle(window_from_point_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_CHILD_WINDOW_FROM_POINT => Some(CoredllValue::Handle(child_window_from_point_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_GET_MESSAGE_W => Some(CoredllValue::Bool(get_message_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_GET_MESSAGE_WNO_WAIT => Some(CoredllValue::Bool(get_message_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_GET_MESSAGE_POS => Some(CoredllValue::U32(kernel.gwe.get_message_pos(thread_id))),
        ORD_PEEK_MESSAGE_W => Some(CoredllValue::Bool(peek_message_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_POST_MESSAGE_W => Some(CoredllValue::Bool(kernel.post_message_w_for_thread(
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
            raw_arg(args, 3),
        ))),
        ORD_POST_THREAD_MESSAGE_W => Some(CoredllValue::Bool(kernel.post_thread_message_w(
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
            raw_arg(args, 3),
        ))),
        ORD_POST_QUIT_MESSAGE => {
            kernel.post_quit_message(thread_id, raw_arg(args, 0));
            Some(CoredllValue::U32(0))
        }
        ORD_SEND_MESSAGE_W | ORD_DEF_WINDOW_PROC_W => Some(CoredllValue::U32(send_message_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_SEND_NOTIFY_MESSAGE_W => {
            let ok = kernel.send_notify_message_w(
                thread_id,
                raw_arg(args, 0),
                raw_arg(args, 1),
                raw_arg(args, 2),
                raw_arg(args, 3),
            );
            if ok {
                kernel.threads.set_last_error(thread_id, 0);
            } else {
                kernel
                    .threads
                    .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
            }
            Some(CoredllValue::Bool(ok))
        }
        ORD_SEND_MESSAGE_TIMEOUT => Some(CoredllValue::U32(send_message_timeout_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_IN_SEND_MESSAGE => Some(CoredllValue::Bool(kernel.gwe.in_send_message(thread_id))),
        ORD_GET_MESSAGE_SOURCE => Some(CoredllValue::U32(kernel.gwe.get_message_source(thread_id))),
        ORD_GET_QUEUE_STATUS => Some(CoredllValue::U32(
            kernel.gwe.get_queue_status(thread_id, raw_arg(args, 0)),
        )),
        ORD_GET_MESSAGE_QUEUE_READY_TIME_STAMP => Some(CoredllValue::U32(
            kernel
                .gwe
                .get_message_queue_ready_time_stamp(thread_id, raw_arg(args, 0)),
        )),
        ORD_DISPATCH_MESSAGE_W => Some(CoredllValue::U32(dispatch_message_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_TRANSLATE_MESSAGE => Some(CoredllValue::Bool(translate_message_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_MESSAGE_BOX_W => Some(CoredllValue::U32(message_box_w_raw(memory, args))),
        ORD_SHGET_SPECIAL_FOLDER_PATH => Some(CoredllValue::Bool(sh_get_special_folder_path_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_FIND_RESOURCE | ORD_FIND_RESOURCE_W => Some(CoredllValue::Handle(find_resource(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_LOAD_RESOURCE => Some(CoredllValue::Handle(load_resource(
            kernel,
            thread_id,
            raw_arg(args, 1),
        ))),
        ORD_SIZEOF_RESOURCE => Some(CoredllValue::U32(sizeof_resource(
            kernel,
            thread_id,
            raw_arg(args, 1),
        ))),
        ORD_LOAD_STRING_W => Some(CoredllValue::U32(load_string_w(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
            raw_i32_arg(args, 3),
        ))),
        ORD_LOAD_MENU_W => Some(CoredllValue::Handle(load_menu_w_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_CREATE_MENU => Some(CoredllValue::Handle(create_menu_raw(kernel, thread_id))),
        ORD_CREATE_POPUP_MENU => Some(CoredllValue::Handle(create_popup_menu_raw(
            kernel, thread_id,
        ))),
        ORD_INSERT_MENU_W => Some(CoredllValue::Bool(insert_menu_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_APPEND_MENU_W => Some(CoredllValue::Bool(append_menu_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_TRACK_POPUP_MENU_EX => Some(CoredllValue::U32(track_popup_menu_ex_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_GET_SUB_MENU => Some(CoredllValue::Handle(get_sub_menu_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_GET_MENU_ITEM_INFO_W => Some(CoredllValue::Bool(get_menu_item_info_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_SET_MENU_ITEM_INFO_W => Some(CoredllValue::Bool(set_menu_item_info_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_ENABLE_MENU_ITEM => Some(CoredllValue::U32(enable_menu_item_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_CHECK_MENU_ITEM => Some(CoredllValue::U32(check_menu_item_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_CHECK_MENU_RADIO_ITEM => Some(CoredllValue::Bool(check_menu_radio_item_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
            raw_arg(args, 3),
        ))),
        ORD_REMOVE_MENU => Some(CoredllValue::Bool(remove_menu_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_DELETE_MENU => Some(CoredllValue::Bool(remove_menu_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_DESTROY_MENU => Some(CoredllValue::Bool(destroy_menu_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_LOAD_ACCELERATORS_W => Some(CoredllValue::Handle(load_accelerators_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_DESTROY_ACCELERATOR_TABLE => Some(CoredllValue::Bool(destroy_accelerator_table_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_TRANSLATE_ACCELERATOR_W => Some(CoredllValue::U32(translate_accelerator_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_LOAD_IMAGE_W => Some(CoredllValue::Handle(load_image_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_LOAD_BITMAP_W => Some(CoredllValue::Handle(load_bitmap_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_GET_OBJECT_W => Some(CoredllValue::U32(get_object_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_i32_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_DELETE_OBJECT => Some(CoredllValue::Bool(delete_object_raw(
            kernel,
            raw_arg(args, 0),
        ))),
        ORD_WAVE_OUT_GET_NUM_DEVS => Some(CoredllValue::U32(kernel.audio.wave_out_get_num_devs())),
        ORD_WAVE_OUT_OPEN => Some(CoredllValue::MmResult(wave_out_open_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_WAVE_OUT_PREPARE_HEADER => Some(CoredllValue::MmResult(wave_out_prepare_header_raw(
            kernel,
            memory,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_WAVE_OUT_UNPREPARE_HEADER => Some(CoredllValue::MmResult(
            wave_out_unprepare_header_raw(kernel, memory, raw_arg(args, 0), raw_arg(args, 1)),
        )),
        ORD_WAVE_OUT_WRITE => Some(CoredllValue::MmResult(wave_out_write_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_WAVE_OUT_PAUSE => Some(CoredllValue::MmResult(kernel.audio.pause(raw_arg(args, 0)))),
        ORD_WAVE_OUT_RESTART => Some(CoredllValue::MmResult(
            kernel.audio.restart(raw_arg(args, 0)),
        )),
        ORD_WAVE_OUT_RESET => Some(CoredllValue::MmResult(
            kernel.audio.wave_out_reset(raw_arg(args, 0)),
        )),
        ORD_WAVE_OUT_CLOSE => Some(CoredllValue::MmResult(
            kernel.audio.wave_out_close(raw_arg(args, 0)),
        )),
        ORD_WAVE_OUT_GET_VOLUME => Some(CoredllValue::MmResult(wave_out_get_u32_raw(
            kernel,
            memory,
            raw_arg(args, 0),
            raw_arg(args, 1),
            WaveOutU32Kind::Volume,
        ))),
        ORD_WAVE_OUT_SET_VOLUME => Some(CoredllValue::MmResult(
            kernel
                .audio
                .wave_out_set_volume(raw_arg(args, 0), raw_arg(args, 1)),
        )),
        ORD_WAVE_OUT_GET_POSITION => Some(CoredllValue::MmResult(wave_out_get_position_raw(
            kernel,
            memory,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_WAVE_OUT_GET_PITCH => Some(CoredllValue::MmResult(wave_out_get_u32_raw(
            kernel,
            memory,
            raw_arg(args, 0),
            raw_arg(args, 1),
            WaveOutU32Kind::Pitch,
        ))),
        ORD_WAVE_OUT_SET_PITCH => Some(CoredllValue::MmResult(
            kernel.audio.set_pitch(raw_arg(args, 0), raw_arg(args, 1)),
        )),
        ORD_WAVE_OUT_GET_PLAYBACK_RATE => Some(CoredllValue::MmResult(wave_out_get_u32_raw(
            kernel,
            memory,
            raw_arg(args, 0),
            raw_arg(args, 1),
            WaveOutU32Kind::PlaybackRate,
        ))),
        ORD_WAVE_OUT_SET_PLAYBACK_RATE => Some(CoredllValue::MmResult(
            kernel
                .audio
                .set_playback_rate(raw_arg(args, 0), raw_arg(args, 1)),
        )),
        ORD_WAVE_OUT_GET_ID => Some(CoredllValue::MmResult(wave_out_get_id_raw(
            kernel,
            memory,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_WAVE_OUT_BREAK_LOOP => Some(CoredllValue::MmResult(MMSYSERR_NOERROR)),
        ORD_WAVE_OUT_MESSAGE => Some(CoredllValue::U32(0)),
        ORD_WAVE_OUT_GET_DEV_CAPS => Some(CoredllValue::MmResult(wave_out_get_dev_caps_raw(
            kernel,
            memory,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_WAVE_OUT_GET_ERROR_TEXT => Some(CoredllValue::MmResult(wave_out_get_error_text_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_LL_MUL => Some(CoredllValue::CeMath(kernel.math.eval(CeMathCall::LlMul {
            lhs: raw_i64_pair(args, 0, 1),
            rhs: raw_i64_pair(args, 2, 3),
        }))),
        ORD_LL_DIV => Some(CoredllValue::CeMath(kernel.math.eval(CeMathCall::LlDiv {
            lhs: raw_i64_pair(args, 0, 1),
            rhs: raw_i64_pair(args, 2, 3),
        }))),
        ORD_LL_REM => Some(CoredllValue::CeMath(kernel.math.eval(CeMathCall::LlRem {
            lhs: raw_i64_pair(args, 0, 1),
            rhs: raw_i64_pair(args, 2, 3),
        }))),
        ORD_ULL_DIV => Some(CoredllValue::CeMath(kernel.math.eval(CeMathCall::UllDiv {
            lhs: raw_u64_pair(args, 0, 1),
            rhs: raw_u64_pair(args, 2, 3),
        }))),
        ORD_ULL_REM => Some(CoredllValue::CeMath(kernel.math.eval(CeMathCall::UllRem {
            lhs: raw_u64_pair(args, 0, 1),
            rhs: raw_u64_pair(args, 2, 3),
        }))),
        ORD_LL_LSHIFT => Some(CoredllValue::CeMath(kernel.math.eval(
            CeMathCall::LlLShift {
                value: raw_i64_pair(args, 0, 1),
                shift: raw_arg(args, 2),
            },
        ))),
        ORD_LL_RSHIFT => Some(CoredllValue::CeMath(kernel.math.eval(
            CeMathCall::LlRShift {
                value: raw_i64_pair(args, 0, 1),
                shift: raw_arg(args, 2),
            },
        ))),
        ORD_ULL_RSHIFT => Some(CoredllValue::CeMath(kernel.math.eval(
            CeMathCall::UllRShift {
                value: raw_u64_pair(args, 0, 1),
                shift: raw_arg(args, 2),
            },
        ))),
        ORD_FPADD => Some(CoredllValue::CeMath(kernel.math.eval(
            CeMathCall::FloatAdd {
                lhs: raw_f32_arg(args, 0),
                rhs: raw_f32_arg(args, 1),
            },
        ))),
        ORD_FPSUB => Some(CoredllValue::CeMath(kernel.math.eval(
            CeMathCall::FloatSub {
                lhs: raw_f32_arg(args, 0),
                rhs: raw_f32_arg(args, 1),
            },
        ))),
        ORD_FPMUL => Some(CoredllValue::CeMath(kernel.math.eval(
            CeMathCall::FloatMul {
                lhs: raw_f32_arg(args, 0),
                rhs: raw_f32_arg(args, 1),
            },
        ))),
        ORD_FPDIV => Some(CoredllValue::CeMath(kernel.math.eval(
            CeMathCall::FloatDiv {
                lhs: raw_f32_arg(args, 0),
                rhs: raw_f32_arg(args, 1),
            },
        ))),
        ORD_DPADD => Some(CoredllValue::CeMath(kernel.math.eval(
            CeMathCall::DoubleAdd {
                lhs: raw_f64_pair(args, 0, 1),
                rhs: raw_f64_pair(args, 2, 3),
            },
        ))),
        ORD_DPSUB => Some(CoredllValue::CeMath(kernel.math.eval(
            CeMathCall::DoubleSub {
                lhs: raw_f64_pair(args, 0, 1),
                rhs: raw_f64_pair(args, 2, 3),
            },
        ))),
        ORD_DPMUL => Some(CoredllValue::CeMath(kernel.math.eval(
            CeMathCall::DoubleMul {
                lhs: raw_f64_pair(args, 0, 1),
                rhs: raw_f64_pair(args, 2, 3),
            },
        ))),
        ORD_DPDIV => Some(CoredllValue::CeMath(kernel.math.eval(
            CeMathCall::DoubleDiv {
                lhs: raw_f64_pair(args, 0, 1),
                rhs: raw_f64_pair(args, 2, 3),
            },
        ))),
        ORD_FPTOLI => Some(CoredllValue::CeMath(
            kernel
                .math
                .eval(CeMathCall::FloatToLong(raw_f32_arg(args, 0))),
        )),
        ORD_FPTOUL => Some(CoredllValue::CeMath(
            kernel
                .math
                .eval(CeMathCall::FloatToUnsignedLong(raw_f32_arg(args, 0))),
        )),
        ORD_DPTOLI => Some(CoredllValue::CeMath(
            kernel
                .math
                .eval(CeMathCall::DoubleToLong(raw_f64_pair(args, 0, 1))),
        )),
        ORD_DPTOUL => {
            Some(CoredllValue::CeMath(kernel.math.eval(
                CeMathCall::DoubleToUnsignedLong(raw_f64_pair(args, 0, 1)),
            )))
        }
        ORD_LTS | ORD_LES | ORD_EQS | ORD_GES | ORD_GTS | ORD_NES => {
            Some(CoredllValue::Bool(compare_guest_f32_raw(
                kernel,
                memory,
                thread_id,
                raw_arg(args, 0),
                raw_arg(args, 1),
                export.ordinal,
            )))
        }
        ORD_LTD | ORD_LED | ORD_EQD | ORD_GED | ORD_GTD | ORD_NED => {
            Some(CoredllValue::Bool(compare_guest_f64_raw(
                kernel,
                memory,
                thread_id,
                raw_arg(args, 0),
                raw_arg(args, 1),
                export.ordinal,
            )))
        }
        ORD_LITOFP => Some(CoredllValue::CeMath(
            kernel
                .math
                .eval(CeMathCall::LongToFloat(raw_i32_arg(args, 0))),
        )),
        ORD_ULTOFP => Some(CoredllValue::CeMath(
            kernel
                .math
                .eval(CeMathCall::UnsignedLongToFloat(raw_arg(args, 0))),
        )),
        ORD_LITODP => Some(CoredllValue::CeMath(
            kernel
                .math
                .eval(CeMathCall::LongToDouble(raw_i32_arg(args, 0))),
        )),
        ORD_ULTODP => Some(CoredllValue::CeMath(
            kernel
                .math
                .eval(CeMathCall::UnsignedLongToDouble(raw_arg(args, 0))),
        )),
        ORD_FPTODP => Some(CoredllValue::CeMath(
            kernel
                .math
                .eval(CeMathCall::FloatToDouble(raw_f32_arg(args, 0))),
        )),
        ORD_DPTOFP => Some(CoredllValue::CeMath(
            kernel
                .math
                .eval(CeMathCall::DoubleToFloat(raw_f64_pair(args, 0, 1))),
        )),
        _ => None,
    }
}

fn compare_guest_f32_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    lhs_ptr: u32,
    rhs_ptr: u32,
    ordinal: u32,
) -> bool {
    let Some(lhs_bits) = read_guest_u32(kernel, memory, thread_id, lhs_ptr) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    };
    let Some(rhs_bits) = read_guest_u32(kernel, memory, thread_id, rhs_ptr) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    };
    kernel.threads.set_last_error(thread_id, 0);
    compare_f32(f32::from_bits(lhs_bits), f32::from_bits(rhs_bits), ordinal)
}

fn compare_guest_f64_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    lhs_ptr: u32,
    rhs_ptr: u32,
    ordinal: u32,
) -> bool {
    let Some(lhs) = read_guest_f64(kernel, memory, thread_id, lhs_ptr) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    };
    let Some(rhs) = read_guest_f64(kernel, memory, thread_id, rhs_ptr) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    };
    kernel.threads.set_last_error(thread_id, 0);
    compare_f64(lhs, rhs, ordinal)
}

fn compare_f32(lhs: f32, rhs: f32, ordinal: u32) -> bool {
    match ordinal {
        ORD_LTS => lhs < rhs,
        ORD_LES => lhs <= rhs,
        ORD_EQS => lhs == rhs,
        ORD_GES => lhs >= rhs,
        ORD_GTS => lhs > rhs,
        ORD_NES => lhs != rhs,
        _ => false,
    }
}

fn compare_f64(lhs: f64, rhs: f64, ordinal: u32) -> bool {
    match ordinal {
        ORD_LTD => lhs < rhs,
        ORD_LED => lhs <= rhs,
        ORD_EQD => lhs == rhs,
        ORD_GED => lhs >= rhs,
        ORD_GTD => lhs > rhs,
        ORD_NED => lhs != rhs,
        _ => false,
    }
}

fn read_guest_f64<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
) -> Option<f64> {
    let low = read_guest_u32(kernel, memory, thread_id, addr)?;
    let high = read_guest_u32(kernel, memory, thread_id, addr.wrapping_add(4))?;
    Some(f64::from_bits((u64::from(high) << 32) | u64::from(low)))
}

fn write_global_memory_status<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    status_ptr: u32,
) -> bool {
    if status_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    let total = 64 * 1024 * 1024;
    let available = 48 * 1024 * 1024;
    let writes = [
        (0, 32),
        (4, 25),
        (8, total),
        (12, available),
        (16, total),
        (20, available),
        (24, 0x7fff_0000),
        (28, 0x4000_0000),
    ];
    for (offset, value) in writes {
        if !write_guest_u32(
            kernel,
            memory,
            thread_id,
            status_ptr.wrapping_add(offset),
            value,
        ) {
            return false;
        }
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn write_store_information<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    info_ptr: u32,
) -> bool {
    if info_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    let object_store = kernel.files.object_store();
    let store_size = object_store.total_bytes.min(u32::MAX as u64) as u32;
    let free_size = object_store
        .free_bytes
        .min(object_store.total_bytes)
        .min(u32::MAX as u64) as u32;
    if !write_guest_u32(kernel, memory, thread_id, info_ptr, store_size)
        || !write_guest_u32(
            kernel,
            memory,
            thread_id,
            info_ptr.wrapping_add(4),
            free_size,
        )
    {
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn write_system_info<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    info_ptr: u32,
) -> bool {
    if info_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    let word_writes = [(0, 0x0004), (2, 0), (32, 4), (34, 0)];
    for (offset, value) in word_writes {
        if !write_guest_u16(
            kernel,
            memory,
            thread_id,
            info_ptr.wrapping_add(offset),
            value,
        ) {
            return false;
        }
    }
    let dword_writes = [
        (4, 4096),
        (8, 0x0001_0000),
        (12, 0x7ffe_ffff),
        (16, 1),
        (20, 1),
        (24, 4000),
        (28, 64 * 1024),
    ];
    for (offset, value) in dword_writes {
        if !write_guest_u32(
            kernel,
            memory,
            thread_id,
            info_ptr.wrapping_add(offset),
            value,
        ) {
            return false;
        }
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn write_os_version_info_w<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    info_ptr: u32,
) -> bool {
    if info_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    let Some(size) = read_guest_u32(kernel, memory, thread_id, info_ptr) else {
        return false;
    };
    if size < 20 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    let dword_writes = [(4, 4), (8, 20), (12, 0), (16, VER_PLATFORM_WIN32_CE)];
    for (offset, value) in dword_writes {
        if !write_guest_u32(
            kernel,
            memory,
            thread_id,
            info_ptr.wrapping_add(offset),
            value,
        ) {
            return false;
        }
    }
    let csd_units = size.saturating_sub(20).min(OSVERSIONINFO_CSD_WCHARS * 2) / 2;
    for index in 0..csd_units {
        if !write_guest_u16(
            kernel,
            memory,
            thread_id,
            info_ptr.wrapping_add(20 + index * 2),
            0,
        ) {
            return false;
        }
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn multi_byte_to_wide_char_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let code_page = active_conversion_code_page(raw_arg(args, 0));
    let flags = raw_arg(args, 1);
    let input_ptr = raw_arg(args, 2);
    let input_len = raw_arg(args, 3) as i32;
    let output_ptr = raw_arg(args, 4);
    let output_capacity = raw_arg(args, 5);
    let Some(bytes) = read_conversion_bytes(kernel, memory, thread_id, input_ptr, input_len) else {
        return 0;
    };
    let Some(units) = decode_multibyte_to_wide(code_page, flags, &bytes) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    write_conversion_wide_result(
        kernel,
        memory,
        thread_id,
        output_ptr,
        output_capacity,
        &units,
    )
}

fn active_conversion_code_page(code_page: u32) -> u32 {
    if code_page == 0 {
        CE_ACP_CODE_PAGE
    } else {
        code_page
    }
}

#[cfg(windows)]
fn decode_multibyte_to_wide(code_page: u32, flags: u32, bytes: &[u8]) -> Option<Vec<u16>> {
    use windows::Win32::Globalization::{MULTI_BYTE_TO_WIDE_CHAR_FLAGS, MultiByteToWideChar};

    let flags = MULTI_BYTE_TO_WIDE_CHAR_FLAGS(flags);
    let needed = unsafe { MultiByteToWideChar(code_page, flags, bytes, None) };
    if needed <= 0 {
        return None;
    }
    let mut units = vec![0; needed as usize];
    let written = unsafe { MultiByteToWideChar(code_page, flags, bytes, Some(&mut units)) };
    if written <= 0 {
        return None;
    }
    units.truncate(written as usize);
    Some(units)
}

#[cfg(not(windows))]
fn decode_multibyte_to_wide(_code_page: u32, _flags: u32, bytes: &[u8]) -> Option<Vec<u16>> {
    Some(bytes.iter().copied().map(u16::from).collect())
}

fn wide_char_to_multi_byte_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let input_ptr = raw_arg(args, 2);
    let input_len = raw_arg(args, 3) as i32;
    let output_ptr = raw_arg(args, 4);
    let output_capacity = raw_arg(args, 5);
    let Some(units) = read_conversion_wide(kernel, memory, thread_id, input_ptr, input_len) else {
        return 0;
    };
    let bytes: Vec<u8> = units
        .into_iter()
        .map(|unit| if unit <= 0x7f { unit as u8 } else { b'?' })
        .collect();
    write_conversion_byte_result(
        kernel,
        memory,
        thread_id,
        output_ptr,
        output_capacity,
        &bytes,
    )
}

fn read_conversion_bytes<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    ptr: u32,
    len: i32,
) -> Option<Vec<u8>> {
    if ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return None;
    }
    if len < 0 {
        let mut bytes = Vec::new();
        for offset in 0..MAX_CONVERSION_CHARS {
            let byte = match memory.read_u8(ptr.wrapping_add(offset)) {
                Ok(byte) => byte,
                Err(_) => {
                    kernel
                        .threads
                        .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
                    return None;
                }
            };
            bytes.push(byte);
            if byte == 0 {
                return Some(bytes);
            }
        }
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        None
    } else {
        read_guest_bytes(kernel, memory, thread_id, ptr, len as u32)
    }
}

fn read_conversion_wide<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    ptr: u32,
    len: i32,
) -> Option<Vec<u16>> {
    if ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return None;
    }
    let max_units = if len < 0 {
        MAX_CONVERSION_CHARS
    } else {
        len as u32
    };
    let mut units = Vec::new();
    for index in 0..max_units {
        let unit = read_guest_u16(
            kernel,
            memory,
            thread_id,
            ptr.wrapping_add(index.wrapping_mul(2)),
        )?;
        units.push(unit);
        if len < 0 && unit == 0 {
            return Some(units);
        }
    }
    if len < 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        None
    } else {
        Some(units)
    }
}

fn write_conversion_wide_result<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    output_ptr: u32,
    output_capacity: u32,
    units: &[u16],
) -> u32 {
    if output_ptr == 0 || output_capacity == 0 {
        return units.len() as u32;
    }
    if output_capacity < units.len() as u32 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INSUFFICIENT_BUFFER);
        return 0;
    }
    for (index, unit) in units.iter().copied().enumerate() {
        if !write_guest_u16(
            kernel,
            memory,
            thread_id,
            output_ptr.wrapping_add((index as u32) * 2),
            unit,
        ) {
            return 0;
        }
    }
    kernel.threads.set_last_error(thread_id, 0);
    units.len() as u32
}

fn write_conversion_byte_result<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    output_ptr: u32,
    output_capacity: u32,
    bytes: &[u8],
) -> u32 {
    if output_ptr == 0 || output_capacity == 0 {
        return bytes.len() as u32;
    }
    if output_capacity < bytes.len() as u32 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INSUFFICIENT_BUFFER);
        return 0;
    }
    if !write_guest_bytes(kernel, memory, thread_id, output_ptr, bytes) {
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    bytes.len() as u32
}

fn string_cb_cat_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    cb_dest: u32,
    src: u32,
) -> u32 {
    string_cat_w_raw(kernel, memory, thread_id, dest, (cb_dest / 2) as usize, src)
}

fn string_cch_cat_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    cch_dest: u32,
    src: u32,
) -> u32 {
    string_cat_w_raw(kernel, memory, thread_id, dest, cch_dest as usize, src)
}

fn string_cch_length_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    src: u32,
    cch_max: u32,
    out_len: u32,
) -> u32 {
    if src == 0 || cch_max == 0 {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let Some(len) = guest_wide_len_bounded(kernel, memory, thread_id, src, cch_max as usize) else {
        return STRSAFE_E_INVALID_PARAMETER;
    };
    if out_len != 0 && !write_guest_u32(kernel, memory, thread_id, out_len, len as u32) {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    0
}

fn string_cat_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    cch_dest: usize,
    src: u32,
) -> u32 {
    if dest == 0 || src == 0 || cch_dest == 0 {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let Some(dest_len) = guest_wide_len_bounded(kernel, memory, thread_id, dest, cch_dest) else {
        return STRSAFE_E_INVALID_PARAMETER;
    };
    let mut remaining = cch_dest.saturating_sub(dest_len);
    if remaining == 0 {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let mut write_addr = dest.wrapping_add((dest_len as u32) * 2);
    let mut read_addr = src;
    loop {
        let Some(unit) = read_guest_u16(kernel, memory, thread_id, read_addr) else {
            return STRSAFE_E_INVALID_PARAMETER;
        };
        if unit == 0 {
            return if write_guest_u16(kernel, memory, thread_id, write_addr, 0) {
                0
            } else {
                STRSAFE_E_INVALID_PARAMETER
            };
        }
        if remaining == 1 {
            return if write_guest_u16(kernel, memory, thread_id, write_addr, 0) {
                STRSAFE_E_INSUFFICIENT_BUFFER
            } else {
                STRSAFE_E_INVALID_PARAMETER
            };
        }
        if !write_guest_u16(kernel, memory, thread_id, write_addr, unit) {
            return STRSAFE_E_INVALID_PARAMETER;
        }
        write_addr = write_addr.wrapping_add(2);
        read_addr = read_addr.wrapping_add(2);
        remaining -= 1;
    }
}

fn guest_wide_len_bounded<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    ptr: u32,
    max_chars: usize,
) -> Option<usize> {
    for index in 0..max_chars {
        let addr = ptr.wrapping_add((index as u32) * 2);
        if read_guest_u16(kernel, memory, thread_id, addr)? == 0 {
            return Some(index);
        }
    }
    None
}

#[derive(Debug, Clone, Copy)]
enum WideCaseMode {
    Lower,
    Upper,
}

fn char_case_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    ptr_or_char: u32,
    mode: WideCaseMode,
) -> u32 {
    if ptr_or_char <= 0xffff {
        return u32::from(convert_ascii_wide_case(ptr_or_char as u16, mode));
    }
    for index in 0..MAX_CONVERSION_CHARS {
        let addr = ptr_or_char.wrapping_add(index.wrapping_mul(2));
        let Some(unit) = read_guest_u16(kernel, memory, thread_id, addr) else {
            return 0;
        };
        if unit == 0 {
            kernel.threads.set_last_error(thread_id, 0);
            return ptr_or_char;
        }
        if !write_guest_u16(
            kernel,
            memory,
            thread_id,
            addr,
            convert_ascii_wide_case(unit, mode),
        ) {
            return 0;
        }
    }
    kernel
        .threads
        .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
    0
}

fn char_case_buff_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    ptr: u32,
    len: u32,
    mode: WideCaseMode,
) -> u32 {
    if ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    for index in 0..len {
        let addr = ptr.wrapping_add(index.wrapping_mul(2));
        let Some(unit) = read_guest_u16(kernel, memory, thread_id, addr) else {
            return 0;
        };
        if !write_guest_u16(
            kernel,
            memory,
            thread_id,
            addr,
            convert_ascii_wide_case(unit, mode),
        ) {
            return 0;
        }
    }
    kernel.threads.set_last_error(thread_id, 0);
    len
}

fn convert_ascii_wide_case(unit: u16, mode: WideCaseMode) -> u16 {
    match mode {
        WideCaseMode::Lower if (b'A' as u16..=b'Z' as u16).contains(&unit) => unit + 0x20,
        WideCaseMode::Upper if (b'a' as u16..=b'z' as u16).contains(&unit) => unit - 0x20,
        _ => unit,
    }
}

fn system_parameters_info_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let action = raw_arg(args, 0);
    let ui_param = raw_arg(args, 1);
    let pv_param = raw_arg(args, 2);
    let ok = match action {
        SPI_GETWORKAREA => {
            if pv_param == 0 {
                false
            } else {
                let rect = Rect {
                    left: 0,
                    top: 0,
                    right: kernel.gwe.system_metric(crate::ce::gwe::SM_CXSCREEN),
                    bottom: kernel.gwe.system_metric(crate::ce::gwe::SM_CYSCREEN),
                };
                write_guest_rect(kernel, memory, thread_id, pv_param, rect)
            }
        }
        SPI_GETPLATFORMTYPE => {
            let text = system_parameter_info_config_string(
                kernel,
                action,
                &["platformtype", "platform_type"],
            )
            .unwrap_or_else(|| "Windows CE".to_owned());
            write_system_parameter_info_string(kernel, memory, thread_id, pv_param, ui_param, &text)
        }
        SPI_GETOEMINFO => {
            let text =
                system_parameter_info_config_string(kernel, action, &["oeminfo", "oem_info"])
                    .unwrap_or_else(|| "WinCE Emulator".to_owned());
            write_system_parameter_info_string(kernel, memory, thread_id, pv_param, ui_param, &text)
        }
        _ => true,
    };
    kernel
        .threads
        .set_last_error(thread_id, if ok { 0 } else { ERROR_INVALID_PARAMETER });
    ok
}

fn write_system_parameter_info_string<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    capacity_chars: u32,
    text: &str,
) -> bool {
    if dest == 0 || capacity_chars == 0 {
        return false;
    }
    write_guest_wide_fixed(
        kernel,
        memory,
        thread_id,
        dest,
        text,
        capacity_chars as usize,
    )
}

fn system_parameter_info_config_string(
    kernel: &CeKernel,
    action: u32,
    aliases: &[&str],
) -> Option<String> {
    let action_name = format!("{action:08x}");
    std::iter::once(action_name.as_str())
        .chain(aliases.iter().copied())
        .find_map(|name| {
            kernel
                .registry
                .query_value(SYSTEM_PARAMETERS_INFO_REGISTRY_PATH, name)
                .ok()
                .and_then(|value| value.as_str())
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        })
}

fn adjust_window_rect_ex_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    rect_ptr: u32,
) -> bool {
    let Some(rect) = read_guest_rect(kernel, memory, thread_id, rect_ptr) else {
        return false;
    };
    set_rect_raw(kernel, memory, thread_id, rect_ptr, rect)
}

fn copy_rect_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest_ptr: u32,
    src_ptr: u32,
) -> bool {
    let Some(rect) = read_guest_rect(kernel, memory, thread_id, src_ptr) else {
        return false;
    };
    set_rect_raw(kernel, memory, thread_id, dest_ptr, rect)
}

fn equal_rect_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    left_ptr: u32,
    right_ptr: u32,
) -> bool {
    let Some(left) = read_guest_rect(kernel, memory, thread_id, left_ptr) else {
        return false;
    };
    let Some(right) = read_guest_rect(kernel, memory, thread_id, right_ptr) else {
        return false;
    };
    left == right
}

fn inflate_rect_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    rect_ptr: u32,
    dx: i32,
    dy: i32,
) -> bool {
    let Some(rect) = read_guest_rect(kernel, memory, thread_id, rect_ptr) else {
        return false;
    };
    set_rect_raw(
        kernel,
        memory,
        thread_id,
        rect_ptr,
        Rect {
            left: rect.left.saturating_sub(dx),
            top: rect.top.saturating_sub(dy),
            right: rect.right.saturating_add(dx),
            bottom: rect.bottom.saturating_add(dy),
        },
    )
}

fn intersect_rect_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest_ptr: u32,
    left_ptr: u32,
    right_ptr: u32,
) -> bool {
    let Some(left) = read_guest_rect(kernel, memory, thread_id, left_ptr) else {
        return false;
    };
    let Some(right) = read_guest_rect(kernel, memory, thread_id, right_ptr) else {
        return false;
    };
    let intersection = Rect {
        left: left.left.max(right.left),
        top: left.top.max(right.top),
        right: left.right.min(right.right),
        bottom: left.bottom.min(right.bottom),
    };
    let intersects = !is_rect_empty_value(intersection);
    let rect_to_write = if intersects {
        intersection
    } else {
        Rect::default()
    };
    set_rect_raw(kernel, memory, thread_id, dest_ptr, rect_to_write) && intersects
}

fn is_rect_empty_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    rect_ptr: u32,
) -> bool {
    read_guest_rect(kernel, memory, thread_id, rect_ptr)
        .map(is_rect_empty_value)
        .unwrap_or(false)
}

fn offset_rect_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    rect_ptr: u32,
    dx: i32,
    dy: i32,
) -> bool {
    let Some(rect) = read_guest_rect(kernel, memory, thread_id, rect_ptr) else {
        return false;
    };
    set_rect_raw(kernel, memory, thread_id, rect_ptr, rect.offset(dx, dy))
}

fn pt_in_rect_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    rect_ptr: u32,
    point: Point,
) -> bool {
    let Some(rect) = read_guest_rect(kernel, memory, thread_id, rect_ptr) else {
        return false;
    };
    point.x >= rect.left && point.x < rect.right && point.y >= rect.top && point.y < rect.bottom
}

fn set_rect_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    rect_ptr: u32,
    rect: Rect,
) -> bool {
    let ok = write_guest_rect(kernel, memory, thread_id, rect_ptr, rect);
    if ok {
        kernel.threads.set_last_error(thread_id, 0);
    }
    ok
}

fn union_rect_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest_ptr: u32,
    left_ptr: u32,
    right_ptr: u32,
) -> bool {
    let Some(left) = read_guest_rect(kernel, memory, thread_id, left_ptr) else {
        return false;
    };
    let Some(right) = read_guest_rect(kernel, memory, thread_id, right_ptr) else {
        return false;
    };
    let left_empty = is_rect_empty_value(left);
    let right_empty = is_rect_empty_value(right);
    let union = match (left_empty, right_empty) {
        (true, true) => Rect::default(),
        (true, false) => right,
        (false, true) => left,
        (false, false) => Rect {
            left: left.left.min(right.left),
            top: left.top.min(right.top),
            right: left.right.max(right.right),
            bottom: left.bottom.max(right.bottom),
        },
    };
    set_rect_raw(kernel, memory, thread_id, dest_ptr, union) && !is_rect_empty_value(union)
}

fn is_rect_empty_value(rect: Rect) -> bool {
    rect.right <= rect.left || rect.bottom <= rect.top
}

fn write_performance_counter_value<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
    value: u64,
) -> bool {
    if addr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    let low = value as u32;
    let high = (value >> 32) as u32;
    write_guest_u32(kernel, memory, thread_id, addr, low)
        && write_guest_u32(kernel, memory, thread_id, addr.wrapping_add(4), high)
}

fn create_file_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let path_ptr = raw_arg(args, 0);
    let path = read_guest_wide_arg(memory, path_ptr);
    let preview = create_file_w_arg_preview(memory, path_ptr);
    kernel.record_file_trace(crate::ce::kernel::FileTraceRecord {
        op: "CreateFileWArg",
        handle: None,
        path: path.clone(),
        preview,
        requested: Some(raw_arg(args, 1)),
        transferred: None,
        position: Some(u64::from(raw_arg(args, 4))),
        result: None,
        error: path.is_none().then(|| "invalid wide path".to_owned()),
    });
    let Some(path) = path else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return u32::MAX;
    };
    match kernel.create_file_w(&path, raw_arg(args, 1), raw_arg(args, 4)) {
        Ok(handle) => handle,
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            u32::MAX
        }
    }
}

fn iswctype_raw(wch: u32, ctype: u32) -> u32 {
    let ch = char::from_u32(wch & 0xffff).unwrap_or('\0');
    let mut mask = 0_u32;
    if ch.is_uppercase() {
        mask |= CTYPE_UPPER;
    }
    if ch.is_lowercase() {
        mask |= CTYPE_LOWER;
    }
    if ch.is_ascii_digit() {
        mask |= CTYPE_DIGIT;
    }
    if ch.is_whitespace() {
        mask |= CTYPE_SPACE;
    }
    if ch.is_ascii_punctuation() {
        mask |= CTYPE_PUNCT;
    }
    if ch.is_control() {
        mask |= CTYPE_CONTROL;
    }
    if matches!(ch, ' ' | '\t') {
        mask |= CTYPE_BLANK;
    }
    if ch.is_ascii_hexdigit() {
        mask |= CTYPE_HEX;
    }
    if ch.is_alphabetic() {
        mask |= CTYPE_ALPHA;
    }
    mask & ctype
}

fn create_file_w_arg_preview<M: CoredllGuestMemory>(memory: &M, path_ptr: u32) -> Option<String> {
    let ansi = read_guest_narrow_preview(memory, path_ptr, 96)?;
    let mut parts = vec![format!("ptr=0x{path_ptr:08x}"), format!("ansi={ansi}")];
    if let Ok(first) = memory.read_u32(path_ptr) {
        parts.push(format!("w0=0x{first:08x}"));
        if let Ok(second) = memory.read_u32(path_ptr.wrapping_add(4)) {
            parts.push(format!("w1=0x{second:08x}"));
        }
        if let Ok(third) = memory.read_u32(path_ptr.wrapping_add(8)) {
            parts.push(format!("w2=0x{third:08x}"));
        }
        if first > 0xffff {
            if let Some(wide) = read_guest_wide_arg(memory, first) {
                parts.push(format!("deref_wide={wide:?}"));
            }
            if let Some(ansi) = read_guest_narrow_preview(memory, first, 96) {
                parts.push(format!("deref_ansi={ansi}"));
            }
        }
    }
    Some(parts.join("/"))
}

fn is_comm_handle(kernel: &CeKernel, handle: u32) -> bool {
    matches!(kernel.handles.get(handle), Ok(KernelObject::Device(_)))
}

fn comm_handle_raw(kernel: &mut CeKernel, thread_id: u32, handle: u32) -> bool {
    if !is_comm_handle(kernel, handle) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn comm_handle_and_ptr_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    handle: u32,
    ptr: u32,
) -> bool {
    if ptr == 0 || !is_comm_handle(kernel, handle) || memory.read_u8(ptr).is_err() {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn get_comm_state_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    handle: u32,
    dcb_ptr: u32,
) -> bool {
    if dcb_ptr == 0 || !is_comm_handle(kernel, handle) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    let length = read_guest_u32(kernel, memory, thread_id, dcb_ptr).unwrap_or(0);
    let ok = write_guest_u32(kernel, memory, thread_id, dcb_ptr, length.max(24))
        && write_guest_u32(kernel, memory, thread_id, dcb_ptr.wrapping_add(4), 9600)
        && write_guest_u8(kernel, memory, thread_id, dcb_ptr.wrapping_add(18), 8)
        && write_guest_u8(kernel, memory, thread_id, dcb_ptr.wrapping_add(19), 0)
        && write_guest_u8(kernel, memory, thread_id, dcb_ptr.wrapping_add(20), 0);
    if ok {
        kernel.threads.set_last_error(thread_id, 0);
    }
    ok
}

fn get_comm_timeouts_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    handle: u32,
    timeouts_ptr: u32,
) -> bool {
    if timeouts_ptr == 0 || !is_comm_handle(kernel, handle) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    let zeros = [0u8; 20];
    if !write_guest_bytes(kernel, memory, thread_id, timeouts_ptr, &zeros) {
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn clear_comm_error_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let handle = raw_arg(args, 0);
    if !is_comm_handle(kernel, handle) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return false;
    }
    let errors_ptr = raw_arg(args, 1);
    if errors_ptr != 0 && !write_guest_u32(kernel, memory, thread_id, errors_ptr, 0) {
        return false;
    }
    let stat_ptr = raw_arg(args, 2);
    if stat_ptr != 0 {
        let zeros = [0u8; 12];
        if !write_guest_bytes(kernel, memory, thread_id, stat_ptr, &zeros) {
            return false;
        }
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn comm_out_u32_zero_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    handle: u32,
    out_ptr: u32,
) -> bool {
    if out_ptr == 0 || !is_comm_handle(kernel, handle) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    if !write_guest_u32(kernel, memory, thread_id, out_ptr, 0) {
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn find_first_file_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    pattern_ptr: u32,
    find_data_ptr: u32,
) -> u32 {
    let Some(pattern) = read_guest_wide_arg(memory, pattern_ptr) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return u32::MAX;
    };
    tracing::debug!(
        target: "ce.file",
        pattern = %pattern,
        find_data_ptr = format_args!("0x{find_data_ptr:08x}"),
        "FindFirstFileW"
    );
    let (handle, find_data) = match kernel.find_first_file_w(&pattern) {
        Ok(result) => {
            tracing::debug!(
                target: "ce.file",
                pattern = %pattern,
                handle = format_args!("0x{:08x}", result.0),
                file_name = %result.1.file_name,
                attributes = format_args!("0x{:08x}", result.1.attributes),
                "FindFirstFileW matched"
            );
            result
        }
        Err(_) => {
            tracing::debug!(
                target: "ce.file",
                pattern = %pattern,
                "FindFirstFileW failed"
            );
            kernel
                .threads
                .set_last_error(thread_id, ERROR_FILE_NOT_FOUND);
            return u32::MAX;
        }
    };
    if find_data_ptr == 0
        || !write_win32_find_data_w(kernel, memory, thread_id, find_data_ptr, &find_data)
    {
        let _ = kernel.find_close(handle);
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return u32::MAX;
    }
    kernel.threads.set_last_error(thread_id, 0);
    handle
}

fn message_box_w_raw<M: CoredllGuestMemory>(memory: &M, args: &[u32]) -> u32 {
    let text = read_guest_wide_arg(memory, raw_arg(args, 1)).unwrap_or_default();
    let caption = read_guest_wide_arg(memory, raw_arg(args, 2)).unwrap_or_default();
    tracing::debug!(
        target: "ce.gwe",
        hwnd = format_args!("0x{:08x}", raw_arg(args, 0)),
        text = %text,
        caption = %caption,
        style = format_args!("0x{:08x}", raw_arg(args, 3)),
        "MessageBoxW"
    );
    1
}

fn find_close_raw(kernel: &mut CeKernel, thread_id: u32, handle: u32) -> bool {
    match kernel.find_close(handle) {
        Ok(ok) => {
            kernel.threads.set_last_error(thread_id, 0);
            ok
        }
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            false
        }
    }
}

fn find_next_file_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    handle: u32,
    find_data_ptr: u32,
) -> bool {
    if find_data_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    match kernel.find_next_file_w(handle) {
        Ok(Some(find_data)) => {
            if !write_win32_find_data_w(kernel, memory, thread_id, find_data_ptr, &find_data) {
                kernel
                    .threads
                    .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
                return false;
            }
            kernel.threads.set_last_error(thread_id, 0);
            true
        }
        Ok(None) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_NO_MORE_FILES);
            false
        }
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            false
        }
    }
}

fn path_bool_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    path_ptr: u32,
    op: fn(&CeKernel, &str) -> Result<()>,
) -> bool {
    let Some(path) = read_guest_wide_arg(memory, path_ptr) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    };
    match op(kernel, &path) {
        Ok(()) => {
            kernel.threads.set_last_error(thread_id, 0);
            true
        }
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_FILE_NOT_FOUND);
            false
        }
    }
}

fn move_file_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let Some(existing_path) = read_guest_wide_arg(memory, raw_arg(args, 0)) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    };
    let Some(new_path) = read_guest_wide_arg(memory, raw_arg(args, 1)) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    };
    match kernel.move_file_w(&existing_path, &new_path) {
        Ok(()) => {
            kernel.threads.set_last_error(thread_id, 0);
            true
        }
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_FILE_NOT_FOUND);
            false
        }
    }
}

fn copy_file_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let Some(existing_path) = read_guest_wide_arg(memory, raw_arg(args, 0)) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    };
    let Some(new_path) = read_guest_wide_arg(memory, raw_arg(args, 1)) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    };
    match kernel.copy_file_w(&existing_path, &new_path, raw_arg(args, 2) != 0) {
        Ok(()) => {
            kernel.threads.set_last_error(thread_id, 0);
            true
        }
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_FILE_NOT_FOUND);
            false
        }
    }
}

fn set_file_attributes_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let Some(path) = read_guest_wide_arg(memory, raw_arg(args, 0)) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    };
    match kernel.set_file_attributes_w(&path, raw_arg(args, 1)) {
        Ok(()) => {
            kernel.threads.set_last_error(thread_id, 0);
            true
        }
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_FILE_NOT_FOUND);
            false
        }
    }
}

fn get_file_attributes_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    path_ptr: u32,
) -> u32 {
    let Some(path) = read_guest_wide_arg(memory, path_ptr) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return u32::MAX;
    };
    match kernel.file_attributes_w(&path) {
        Ok(data) => {
            kernel.threads.set_last_error(thread_id, 0);
            data.attributes
        }
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_FILE_NOT_FOUND);
            u32::MAX
        }
    }
}

fn get_file_attributes_ex_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let Some(path) = read_guest_wide_arg(memory, raw_arg(args, 0)) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    };
    if raw_arg(args, 1) != 0 || raw_arg(args, 2) == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    let Ok(data) = kernel.file_attributes_w(&path) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_FILE_NOT_FOUND);
        return false;
    };
    if !write_win32_file_attribute_data_w(kernel, memory, thread_id, raw_arg(args, 2), &data) {
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn file_handle_bool_raw(kernel: &mut CeKernel, thread_id: u32, handle: u32) -> bool {
    match kernel.handles.get(handle) {
        Ok(KernelObject::File(_)) => {
            kernel.threads.set_last_error(thread_id, 0);
            true
        }
        _ => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            false
        }
    }
}

const FILETIME_TICKS_PER_SECOND: u64 = 10_000_000;
const FILETIME_TICKS_PER_MILLISECOND: u64 = 10_000;
const SYSTEM_TIME_BASE_YEAR: i32 = 2024;
const SYSTEM_TIME_BASE_MONTH: i32 = 1;
const SYSTEM_TIME_BASE_DAY: i32 = 1;
const TIME_ZONE_INFORMATION_SIZE: usize = 172;
const TIME_ZONE_ID_UNKNOWN: u32 = 0;
const TIME_ZONE_ID_INVALID: u32 = u32::MAX;
const OEM_DEBUG_READ_NODATA: u32 = u32::MAX;

fn input_debug_char_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let dest = raw_arg(args, 0);
    let format = raw_arg(args, 1);
    if args.len() >= 2
        && dest != 0
        && format != 0
        && read_guest_wide_z(memory, format, 256).is_ok_and(|format| format.contains('%'))
    {
        return crt::wsprintf_w_raw(
            kernel,
            memory,
            thread_id,
            dest,
            format,
            if args.len() > 2 { &args[2..] } else { &[] },
        );
    }
    OEM_DEBUG_READ_NODATA
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SystemTimeFields {
    year: u16,
    month: u16,
    day_of_week: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
    milliseconds: u16,
}

fn write_current_system_time<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    system_time_ptr: u32,
) {
    if system_time_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return;
    }
    let fields = system_time_fields_from_ticks(current_filetime_ticks(kernel));
    let ok = write_guest_u16(kernel, memory, thread_id, system_time_ptr, fields.year)
        && write_guest_u16(
            kernel,
            memory,
            thread_id,
            system_time_ptr.wrapping_add(2),
            fields.month,
        )
        && write_guest_u16(
            kernel,
            memory,
            thread_id,
            system_time_ptr.wrapping_add(4),
            fields.day_of_week,
        )
        && write_guest_u16(
            kernel,
            memory,
            thread_id,
            system_time_ptr.wrapping_add(6),
            fields.day,
        )
        && write_guest_u16(
            kernel,
            memory,
            thread_id,
            system_time_ptr.wrapping_add(8),
            fields.hour,
        )
        && write_guest_u16(
            kernel,
            memory,
            thread_id,
            system_time_ptr.wrapping_add(10),
            fields.minute,
        )
        && write_guest_u16(
            kernel,
            memory,
            thread_id,
            system_time_ptr.wrapping_add(12),
            fields.second,
        )
        && write_guest_u16(
            kernel,
            memory,
            thread_id,
            system_time_ptr.wrapping_add(14),
            fields.milliseconds,
        );
    if ok {
        kernel.threads.set_last_error(thread_id, 0);
    }
}

fn write_current_file_time<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    file_time_ptr: u32,
) {
    if file_time_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return;
    }
    let ticks = current_filetime_ticks(kernel);
    if write_guest_u32(kernel, memory, thread_id, file_time_ptr, ticks as u32)
        && write_guest_u32(
            kernel,
            memory,
            thread_id,
            file_time_ptr.wrapping_add(4),
            (ticks >> 32) as u32,
        )
    {
        kernel.threads.set_last_error(thread_id, 0);
    }
}

fn write_time_zone_information<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    time_zone_information_ptr: u32,
) -> u32 {
    if time_zone_information_ptr == 0
        || !write_guest_bytes(
            kernel,
            memory,
            thread_id,
            time_zone_information_ptr,
            &[0; TIME_ZONE_INFORMATION_SIZE],
        )
        || !write_guest_i32(kernel, memory, thread_id, time_zone_information_ptr, 0)
        || !write_guest_wide_fixed(
            kernel,
            memory,
            thread_id,
            time_zone_information_ptr.wrapping_add(4),
            "UTC",
            32,
        )
        || !write_guest_i32(
            kernel,
            memory,
            thread_id,
            time_zone_information_ptr.wrapping_add(84),
            0,
        )
        || !write_guest_i32(
            kernel,
            memory,
            thread_id,
            time_zone_information_ptr.wrapping_add(168),
            0,
        )
    {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return TIME_ZONE_ID_INVALID;
    }
    kernel.threads.set_last_error(thread_id, 0);
    TIME_ZONE_ID_UNKNOWN
}

fn current_filetime_ticks(kernel: &CeKernel) -> u64 {
    let base_days = days_before_year(SYSTEM_TIME_BASE_YEAR)
        + days_before_month(SYSTEM_TIME_BASE_YEAR, SYSTEM_TIME_BASE_MONTH)
        + i64::from(SYSTEM_TIME_BASE_DAY - 1);
    (base_days as u64)
        .saturating_mul(86_400)
        .saturating_mul(FILETIME_TICKS_PER_SECOND)
        .saturating_add(
            u64::from(kernel.timers.tick_count()).saturating_mul(FILETIME_TICKS_PER_MILLISECOND),
        )
}

fn system_time_fields_from_ticks(ticks: u64) -> SystemTimeFields {
    let total_ms = ticks / FILETIME_TICKS_PER_MILLISECOND;
    let total_seconds = total_ms / 1_000;
    let mut days = (total_seconds / 86_400) as i64;
    let seconds_of_day = total_seconds % 86_400;
    let milliseconds = (total_ms % 1_000) as u16;

    let mut year = 1601;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    let mut month = 1;
    loop {
        let days_in_month = days_in_month(year, month);
        if days < i64::from(days_in_month) {
            break;
        }
        days -= i64::from(days_in_month);
        month += 1;
    }

    let days_since_epoch = total_seconds / 86_400;
    SystemTimeFields {
        year: year as u16,
        month: month as u16,
        day_of_week: ((days_since_epoch + 1) % 7) as u16,
        day: (days + 1) as u16,
        hour: (seconds_of_day / 3_600) as u16,
        minute: ((seconds_of_day % 3_600) / 60) as u16,
        second: (seconds_of_day % 60) as u16,
        milliseconds,
    }
}

fn system_time_to_file_time_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    system_time_ptr: u32,
    file_time_ptr: u32,
) -> bool {
    if system_time_ptr == 0 || file_time_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    let Some(year) = read_guest_u16(kernel, memory, thread_id, system_time_ptr) else {
        return false;
    };
    let Some(month) = read_guest_u16(kernel, memory, thread_id, system_time_ptr.wrapping_add(2))
    else {
        return false;
    };
    let Some(day) = read_guest_u16(kernel, memory, thread_id, system_time_ptr.wrapping_add(6))
    else {
        return false;
    };
    let Some(hour) = read_guest_u16(kernel, memory, thread_id, system_time_ptr.wrapping_add(8))
    else {
        return false;
    };
    let Some(minute) = read_guest_u16(kernel, memory, thread_id, system_time_ptr.wrapping_add(10))
    else {
        return false;
    };
    let Some(second) = read_guest_u16(kernel, memory, thread_id, system_time_ptr.wrapping_add(12))
    else {
        return false;
    };
    let Some(milliseconds) =
        read_guest_u16(kernel, memory, thread_id, system_time_ptr.wrapping_add(14))
    else {
        return false;
    };
    if year < 1601
        || month == 0
        || month > 12
        || day == 0
        || day > days_in_month(year as i32, month as i32) as u16
        || hour > 23
        || minute > 59
        || second > 59
        || milliseconds > 999
    {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    let days = days_before_year(year as i32)
        + days_before_month(year as i32, month as i32)
        + i64::from(day - 1);
    let seconds =
        days * 86_400 + i64::from(hour) * 3_600 + i64::from(minute) * 60 + i64::from(second);
    let ticks = (seconds as u64)
        .saturating_mul(FILETIME_TICKS_PER_SECOND)
        .saturating_add(u64::from(milliseconds).saturating_mul(FILETIME_TICKS_PER_MILLISECOND));
    if !write_guest_u32(kernel, memory, thread_id, file_time_ptr, ticks as u32)
        || !write_guest_u32(
            kernel,
            memory,
            thread_id,
            file_time_ptr.wrapping_add(4),
            (ticks >> 32) as u32,
        )
    {
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn file_time_to_system_time_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    file_time_ptr: u32,
    system_time_ptr: u32,
) -> bool {
    if file_time_ptr == 0 || system_time_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    let Some(low) = read_guest_u32(kernel, memory, thread_id, file_time_ptr) else {
        return false;
    };
    let Some(high) = read_guest_u32(kernel, memory, thread_id, file_time_ptr.wrapping_add(4))
    else {
        return false;
    };
    let fields = system_time_fields_from_ticks((u64::from(high) << 32) | u64::from(low));
    let ok = write_guest_u16(kernel, memory, thread_id, system_time_ptr, fields.year)
        && write_guest_u16(
            kernel,
            memory,
            thread_id,
            system_time_ptr.wrapping_add(2),
            fields.month,
        )
        && write_guest_u16(
            kernel,
            memory,
            thread_id,
            system_time_ptr.wrapping_add(4),
            fields.day_of_week,
        )
        && write_guest_u16(
            kernel,
            memory,
            thread_id,
            system_time_ptr.wrapping_add(6),
            fields.day,
        )
        && write_guest_u16(
            kernel,
            memory,
            thread_id,
            system_time_ptr.wrapping_add(8),
            fields.hour,
        )
        && write_guest_u16(
            kernel,
            memory,
            thread_id,
            system_time_ptr.wrapping_add(10),
            fields.minute,
        )
        && write_guest_u16(
            kernel,
            memory,
            thread_id,
            system_time_ptr.wrapping_add(12),
            fields.second,
        )
        && write_guest_u16(
            kernel,
            memory,
            thread_id,
            system_time_ptr.wrapping_add(14),
            fields.milliseconds,
        );
    if ok {
        kernel.threads.set_last_error(thread_id, 0);
    }
    ok
}

fn days_before_year(year: i32) -> i64 {
    let y = year - 1601;
    let leap_days_before = |year: i32| year / 4 - year / 100 + year / 400;
    i64::from(y * 365 + leap_days_before(year - 1) - leap_days_before(1600))
}

fn days_before_month(year: i32, month: i32) -> i64 {
    const MONTH_DAYS: [i32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut days = 0;
    for m in 1..month {
        days += MONTH_DAYS[(m - 1) as usize];
        if m == 2 && is_leap_year(year) {
            days += 1;
        }
    }
    i64::from(days)
}

fn days_in_month(year: i32, month: i32) -> i32 {
    const MONTH_DAYS: [i32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut days = MONTH_DAYS[(month - 1) as usize];
    if month == 2 && is_leap_year(year) {
        days += 1;
    }
    days
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn reg_create_key_ex_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let Some(subkey) = read_optional_guest_wide_arg(memory, raw_arg(args, 1)) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return crate::ce::registry::ERROR_INVALID_PARAMETER;
    };
    let result = kernel
        .registry
        .reg_create_key_exw(raw_arg(args, 0), subkey.as_deref());
    if result.status == crate::ce::registry::ERROR_SUCCESS {
        if let Some(hkey) = result.hkey {
            if !write_optional_u32(kernel, memory, thread_id, raw_arg(args, 7), hkey) {
                return crate::ce::registry::ERROR_INVALID_PARAMETER;
            }
        }
        if !write_optional_u32(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 8),
            result.disposition,
        ) {
            return crate::ce::registry::ERROR_INVALID_PARAMETER;
        }
    }
    result.status
}

fn reg_open_key_ex_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let Some(subkey) = read_optional_guest_wide_arg(memory, raw_arg(args, 1)) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return crate::ce::registry::ERROR_INVALID_PARAMETER;
    };
    let result = kernel.registry.reg_open_key_exw(
        raw_arg(args, 0),
        subkey.as_deref(),
        raw_arg(args, 2),
        raw_arg(args, 3),
    );
    if result.status == crate::ce::registry::ERROR_SUCCESS {
        if let Some(hkey) = result.hkey {
            if !write_optional_u32(kernel, memory, thread_id, raw_arg(args, 4), hkey) {
                return crate::ce::registry::ERROR_INVALID_PARAMETER;
            }
        }
    }
    result.status
}

fn reg_delete_key_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let Some(subkey) = read_optional_guest_wide_arg(memory, raw_arg(args, 1)) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return crate::ce::registry::ERROR_INVALID_PARAMETER;
    };
    kernel
        .registry
        .reg_delete_key_w(raw_arg(args, 0), subkey.as_deref())
}

fn reg_set_value_ex_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let Some(value_name) = read_optional_guest_wide_arg(memory, raw_arg(args, 1)) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return crate::ce::registry::ERROR_INVALID_PARAMETER;
    };
    let Some(data) = read_guest_bytes(
        kernel,
        memory,
        thread_id,
        raw_arg(args, 4),
        raw_arg(args, 5),
    ) else {
        return crate::ce::registry::ERROR_INVALID_PARAMETER;
    };
    kernel.registry.reg_set_value_exw(
        raw_arg(args, 0),
        value_name.as_deref(),
        raw_arg(args, 3),
        &data,
    )
}

fn reg_query_value_ex_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let Some(value_name) = read_optional_guest_wide_arg(memory, raw_arg(args, 1)) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return crate::ce::registry::ERROR_INVALID_PARAMETER;
    };
    let data_len_ptr = raw_arg(args, 5);
    let data_capacity = if data_len_ptr == 0 {
        None
    } else {
        let Some(capacity) = read_guest_u32(kernel, memory, thread_id, data_len_ptr) else {
            return crate::ce::registry::ERROR_INVALID_PARAMETER;
        };
        Some(capacity as usize)
    };
    let result =
        kernel
            .registry
            .reg_query_value_exw(raw_arg(args, 0), value_name.as_deref(), data_capacity);
    if let Some(value_type) = result.value_type {
        if !write_optional_u32(kernel, memory, thread_id, raw_arg(args, 3), value_type) {
            return crate::ce::registry::ERROR_INVALID_PARAMETER;
        }
    }
    if !write_optional_u32(kernel, memory, thread_id, data_len_ptr, result.required_len) {
        return crate::ce::registry::ERROR_INVALID_PARAMETER;
    }
    if result.status == crate::ce::registry::ERROR_SUCCESS {
        if let Some(data) = result.data {
            if raw_arg(args, 4) != 0
                && !write_guest_bytes(kernel, memory, thread_id, raw_arg(args, 4), &data)
            {
                return crate::ce::registry::ERROR_INVALID_PARAMETER;
            }
        }
    }
    result.status
}

fn reg_query_info_key_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let result = kernel.registry.reg_query_info_key_w(raw_arg(args, 0));
    if result.status != crate::ce::registry::ERROR_SUCCESS {
        return result.status;
    }
    for (arg_index, value) in [
        (4, result.subkeys),
        (5, result.max_subkey_chars),
        (7, result.values),
        (8, result.max_value_name_chars),
        (9, result.max_value_data_len),
    ] {
        if !write_optional_u32(kernel, memory, thread_id, raw_arg(args, arg_index), value) {
            return crate::ce::registry::ERROR_INVALID_PARAMETER;
        }
    }
    result.status
}

fn reg_enum_value_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let name_len_ptr = raw_arg(args, 3);
    let name_capacity = if name_len_ptr == 0 {
        None
    } else {
        let Some(capacity) = read_guest_u32(kernel, memory, thread_id, name_len_ptr) else {
            return crate::ce::registry::ERROR_INVALID_PARAMETER;
        };
        Some(capacity as usize)
    };
    let data_len_ptr = raw_arg(args, 7);
    let data_capacity = if data_len_ptr == 0 {
        None
    } else {
        let Some(capacity) = read_guest_u32(kernel, memory, thread_id, data_len_ptr) else {
            return crate::ce::registry::ERROR_INVALID_PARAMETER;
        };
        Some(capacity as usize)
    };
    let result = kernel.registry.reg_enum_value_w(
        raw_arg(args, 0),
        raw_arg(args, 1),
        name_capacity,
        data_capacity,
    );
    if let Some(value_type) = result.value_type {
        if !write_optional_u32(kernel, memory, thread_id, raw_arg(args, 5), value_type) {
            return crate::ce::registry::ERROR_INVALID_PARAMETER;
        }
    }
    if !write_optional_u32(
        kernel,
        memory,
        thread_id,
        name_len_ptr,
        result.required_name_chars,
    ) || !write_optional_u32(
        kernel,
        memory,
        thread_id,
        data_len_ptr,
        result.required_data_len,
    ) {
        return crate::ce::registry::ERROR_INVALID_PARAMETER;
    }
    if result.status == crate::ce::registry::ERROR_SUCCESS {
        if let Some(name) = result.name {
            if raw_arg(args, 2) != 0
                && write_wide_result(
                    kernel,
                    memory,
                    thread_id,
                    raw_arg(args, 2),
                    name_capacity.unwrap_or(0),
                    &name,
                ) == 0
            {
                return crate::ce::registry::ERROR_INVALID_PARAMETER;
            }
        }
        if let Some(data) = result.data {
            if raw_arg(args, 6) != 0
                && !write_guest_bytes(kernel, memory, thread_id, raw_arg(args, 6), &data)
            {
                return crate::ce::registry::ERROR_INVALID_PARAMETER;
            }
        }
    }
    result.status
}

fn reg_enum_key_ex_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let name_len_ptr = raw_arg(args, 3);
    let name_capacity = if name_len_ptr == 0 {
        None
    } else {
        let Some(capacity) = read_guest_u32(kernel, memory, thread_id, name_len_ptr) else {
            return crate::ce::registry::ERROR_INVALID_PARAMETER;
        };
        Some(capacity as usize)
    };
    let result =
        kernel
            .registry
            .reg_enum_key_ex_w(raw_arg(args, 0), raw_arg(args, 1), name_capacity);
    if !write_optional_u32(
        kernel,
        memory,
        thread_id,
        name_len_ptr,
        result.required_name_chars,
    ) {
        return crate::ce::registry::ERROR_INVALID_PARAMETER;
    }
    if result.status == crate::ce::registry::ERROR_SUCCESS {
        if let Some(name) = result.name {
            if raw_arg(args, 2) != 0
                && write_wide_result(
                    kernel,
                    memory,
                    thread_id,
                    raw_arg(args, 2),
                    name_capacity.unwrap_or(0),
                    &name,
                ) == 0
            {
                return crate::ce::registry::ERROR_INVALID_PARAMETER;
            }
        }
    }
    result.status
}

fn reg_delete_value_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let Some(value_name) = read_optional_guest_wide_arg(memory, raw_arg(args, 1)) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return crate::ce::registry::ERROR_INVALID_PARAMETER;
    };
    kernel
        .registry
        .reg_delete_value_w(raw_arg(args, 0), value_name.as_deref())
}

fn write_win32_find_data_w<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
    find_data: &FindData,
) -> bool {
    let file_size_high = (find_data.file_size >> 32) as u32;
    let file_size_low = find_data.file_size as u32;
    let dwords = [
        (0, find_data.attributes),
        (4, 0),
        (8, 0),
        (12, 0),
        (16, 0),
        (20, 0),
        (24, 0),
        (28, file_size_high),
        (32, file_size_low),
        (36, 0),
    ];
    for (offset, value) in dwords {
        if !write_guest_u32(kernel, memory, thread_id, addr.wrapping_add(offset), value) {
            return false;
        }
    }
    write_guest_wide_fixed(
        kernel,
        memory,
        thread_id,
        addr.wrapping_add(WIN32_FIND_DATAW_FILE_NAME),
        &find_data.file_name,
        WIN32_FIND_DATAW_FILE_NAME_CHARS,
    )
}

fn write_win32_file_attribute_data_w<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
    find_data: &FindData,
) -> bool {
    let file_size_high = (find_data.file_size >> 32) as u32;
    let file_size_low = find_data.file_size as u32;
    let dwords = [
        (0, find_data.attributes),
        (4, 0),
        (8, 0),
        (12, 0),
        (16, 0),
        (20, 0),
        (24, 0),
        (28, file_size_high),
        (32, file_size_low),
    ];
    for (offset, value) in dwords {
        if !write_guest_u32(kernel, memory, thread_id, addr.wrapping_add(offset), value) {
            return false;
        }
    }
    true
}

fn get_module_file_name_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    module: u32,
    buffer: u32,
    max_chars: u32,
) -> u32 {
    if buffer == 0 || max_chars == 0 || (module != 0 && module != kernel.process_module_base()) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let units = kernel
        .process_module_path()
        .encode_utf16()
        .collect::<Vec<_>>();
    let copied = units.len().min(max_chars.saturating_sub(1) as usize);
    for (index, unit) in units.iter().copied().take(copied).enumerate() {
        if !write_guest_u16(
            kernel,
            memory,
            thread_id,
            buffer.wrapping_add(index as u32 * 2),
            unit,
        ) {
            return 0;
        }
    }
    if !write_guest_u16(
        kernel,
        memory,
        thread_id,
        buffer.wrapping_add(copied as u32 * 2),
        0,
    ) {
        return 0;
    }
    copied as u32
}

const COREDLL_MODULE_HANDLE: u32 = 0x7000_0001;

fn get_module_handle_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    name_ptr: u32,
) -> u32 {
    if name_ptr == 0 {
        kernel.threads.set_last_error(thread_id, 0);
        return kernel.process_module_base();
    }
    let Some(name) = read_guest_wide_arg(memory, name_ptr) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    if is_coredll_module_name(&name) {
        kernel.threads.set_last_error(thread_id, 0);
        return COREDLL_MODULE_HANDLE;
    }
    if let Some(handle) = kernel.loaded_module_handle(&name) {
        kernel.threads.set_last_error(thread_id, 0);
        return handle;
    }
    kernel
        .threads
        .set_last_error(thread_id, ERROR_FILE_NOT_FOUND);
    0
}

fn load_library_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    name_ptr: u32,
) -> u32 {
    let Some(name) = read_guest_wide_arg(memory, name_ptr) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    if is_coredll_module_name(&name) {
        kernel.threads.set_last_error(thread_id, 0);
        return COREDLL_MODULE_HANDLE;
    }
    if let Some(handle) = kernel.loaded_module_handle(&name) {
        kernel.threads.set_last_error(thread_id, 0);
        return handle;
    }
    kernel
        .threads
        .set_last_error(thread_id, ERROR_FILE_NOT_FOUND);
    0
}

fn free_library_raw(kernel: &mut CeKernel, thread_id: u32, module: u32) -> bool {
    if module == COREDLL_MODULE_HANDLE
        || module == kernel.process_module_base()
        || kernel.is_loaded_module_handle(module)
    {
        kernel.threads.set_last_error(thread_id, 0);
        return true;
    }
    kernel
        .threads
        .set_last_error(thread_id, ERROR_INVALID_HANDLE);
    false
}

fn get_proc_address_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let module = raw_arg(args, 0);
    let name_ptr = raw_arg(args, 1);
    if name_ptr <= 0xffff {
        return get_proc_address_ordinal_raw(kernel, thread_id, module, name_ptr);
    }
    let Some(name) = read_guest_wide_arg(memory, name_ptr) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    get_proc_address_name_raw(kernel, thread_id, module, &name)
}

fn get_proc_address_a_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let module = raw_arg(args, 0);
    let name_ptr = raw_arg(args, 1);
    if name_ptr <= 0xffff {
        return get_proc_address_ordinal_raw(kernel, thread_id, module, name_ptr);
    }
    let Some(name) = read_guest_ascii_z(memory, name_ptr, 256) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    get_proc_address_name_raw(kernel, thread_id, module, &name)
}

fn get_proc_address_name_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    module: u32,
    name: &str,
) -> u32 {
    if module == COREDLL_MODULE_HANDLE {
        return get_coredll_proc_address_name_raw(kernel, thread_id, name);
    }
    if kernel.is_loaded_module_handle(module) {
        if let Some(address) = kernel.resolve_loaded_module_proc_by_name(module, name) {
            kernel.threads.set_last_error(thread_id, 0);
            return address;
        }
        kernel
            .threads
            .set_last_error(thread_id, ERROR_FILE_NOT_FOUND);
        return 0;
    }
    kernel
        .threads
        .set_last_error(thread_id, ERROR_INVALID_HANDLE);
    0
}

fn get_proc_address_ordinal_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    module: u32,
    ordinal: u32,
) -> u32 {
    if module == COREDLL_MODULE_HANDLE {
        return get_coredll_proc_address_ordinal_raw(kernel, thread_id, ordinal);
    }
    if kernel.is_loaded_module_handle(module) {
        if let Some(address) = kernel.resolve_loaded_module_proc_by_ordinal(module, ordinal) {
            kernel.threads.set_last_error(thread_id, 0);
            return address;
        }
        kernel
            .threads
            .set_last_error(thread_id, ERROR_FILE_NOT_FOUND);
        return 0;
    }
    kernel
        .threads
        .set_last_error(thread_id, ERROR_INVALID_HANDLE);
    0
}

fn get_coredll_proc_address_name_raw(kernel: &mut CeKernel, thread_id: u32, name: &str) -> u32 {
    let table = CoredllExportTable::static_ordinals();
    let Some(export) = table.resolve_name(name) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_FILE_NOT_FOUND);
        return 0;
    };
    get_coredll_proc_address_ordinal_raw(kernel, thread_id, export.ordinal)
}

fn get_coredll_proc_address_ordinal_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    ordinal: u32,
) -> u32 {
    if CoredllExportTable::resolve_static_ordinal(ordinal).is_none() {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_FILE_NOT_FOUND);
        return 0;
    }
    let Some(address) = crate::emulator::imports::dynamic_coredll_proc_address(ordinal) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    kernel.threads.set_last_error(thread_id, 0);
    address
}

fn is_coredll_module_name(name: &str) -> bool {
    let normalized = name
        .trim()
        .trim_end_matches('\0')
        .trim_end_matches(".dll")
        .trim_end_matches(".DLL")
        .to_ascii_lowercase();
    normalized == "coredll"
}

fn read_file_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let handle = raw_arg(args, 0);
    let buffer = raw_arg(args, 1);
    let requested = raw_arg(args, 2);
    let transferred_ptr = raw_arg(args, 3);
    if requested != 0 && buffer == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        write_optional_count(kernel, memory, thread_id, transferred_ptr, 0);
        return false;
    }
    let mut write_failed = false;
    let mut cursor = buffer;
    let transferred = match kernel.read_file_into(handle, requested, |bytes| {
        if memory.write_bytes(cursor, bytes).is_err() {
            write_failed = true;
            return Err(Error::InvalidArgument(
                "ReadFile guest buffer is not writable".to_owned(),
            ));
        }
        cursor = cursor.wrapping_add(bytes.len() as u32);
        Ok(())
    }) {
        Ok(transferred) => transferred,
        Err(_) => {
            let error = if write_failed {
                ERROR_INVALID_PARAMETER
            } else {
                ERROR_INVALID_HANDLE
            };
            kernel.threads.set_last_error(thread_id, error);
            write_optional_count(kernel, memory, thread_id, transferred_ptr, 0);
            return false;
        }
    };
    if !write_optional_count(kernel, memory, thread_id, transferred_ptr, transferred) {
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn write_file_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let handle = raw_arg(args, 0);
    let buffer = raw_arg(args, 1);
    let requested = raw_arg(args, 2);
    let transferred_ptr = raw_arg(args, 3);
    let Some(bytes) = read_guest_bytes(kernel, memory, thread_id, buffer, requested) else {
        write_optional_count(kernel, memory, thread_id, transferred_ptr, 0);
        return false;
    };
    let result = match kernel.write_file(handle, &bytes) {
        Ok(result) => result,
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            write_optional_count(kernel, memory, thread_id, transferred_ptr, 0);
            return false;
        }
    };
    write_optional_count(
        kernel,
        memory,
        thread_id,
        transferred_ptr,
        result.bytes_transferred,
    ) && result.success
}

fn device_io_control_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let handle = raw_arg(args, 0);
    let ioctl_code = raw_arg(args, 1);
    let input_ptr = raw_arg(args, 2);
    let input_len = raw_arg(args, 3);
    let output_ptr = raw_arg(args, 4);
    let output_capacity = raw_arg(args, 5);
    let returned_ptr = raw_arg(args, 6);
    let input = if input_len == 0 {
        Vec::new()
    } else if input_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        write_optional_count(kernel, memory, thread_id, returned_ptr, 0);
        return false;
    } else {
        let Some(bytes) = read_guest_bytes(kernel, memory, thread_id, input_ptr, input_len) else {
            write_optional_count(kernel, memory, thread_id, returned_ptr, 0);
            return false;
        };
        bytes
    };
    let result = match kernel.device_io_control(handle, ioctl_code, &input, output_capacity) {
        Ok(result) => result,
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            write_optional_count(kernel, memory, thread_id, returned_ptr, 0);
            return false;
        }
    };
    if !result.output.is_empty()
        && (output_ptr == 0
            || !write_guest_bytes(kernel, memory, thread_id, output_ptr, &result.output))
    {
        write_optional_count(kernel, memory, thread_id, returned_ptr, 0);
        return false;
    }
    if !write_optional_count(
        kernel,
        memory,
        thread_id,
        returned_ptr,
        result.bytes_returned,
    ) {
        return false;
    }
    kernel.threads.set_last_error(
        thread_id,
        if result.success {
            0
        } else {
            ERROR_NOT_SUPPORTED
        },
    );
    result.success
}

fn kernel_io_control_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    match raw_arg(args, 0) {
        IOCTL_HAL_GET_DEVICEID => write_hal_device_id(kernel, memory, thread_id, args),
        _ => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_NOT_SUPPORTED);
            write_optional_count(kernel, memory, thread_id, raw_arg(args, 5), 0);
            false
        }
    }
}

fn write_hal_device_id<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    const DEVICE_ID_HEADER_SIZE: u32 = 20;
    const PRESET_ID: &[u8] = b"WINCE_EMU\0";
    const PLATFORM_ID: &[u8] = b"INAVI_HOST\0";

    let out_ptr = raw_arg(args, 3);
    let out_size = raw_arg(args, 4);
    let returned_ptr = raw_arg(args, 5);
    let preset_offset = DEVICE_ID_HEADER_SIZE;
    let platform_offset = preset_offset + PRESET_ID.len() as u32;
    let required_size = platform_offset + PLATFORM_ID.len() as u32;

    if out_ptr == 0 || out_size < required_size {
        if out_ptr != 0 && out_size >= 4 {
            let _ = write_guest_u32(kernel, memory, thread_id, out_ptr, required_size);
        }
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INSUFFICIENT_BUFFER);
        return false;
    }

    let ok = write_guest_u32(kernel, memory, thread_id, out_ptr, required_size)
        && write_guest_u32(
            kernel,
            memory,
            thread_id,
            out_ptr.wrapping_add(4),
            preset_offset,
        )
        && write_guest_u32(
            kernel,
            memory,
            thread_id,
            out_ptr.wrapping_add(8),
            PRESET_ID.len() as u32,
        )
        && write_guest_u32(
            kernel,
            memory,
            thread_id,
            out_ptr.wrapping_add(12),
            platform_offset,
        )
        && write_guest_u32(
            kernel,
            memory,
            thread_id,
            out_ptr.wrapping_add(16),
            PLATFORM_ID.len() as u32,
        )
        && write_guest_bytes(
            kernel,
            memory,
            thread_id,
            out_ptr.wrapping_add(preset_offset),
            PRESET_ID,
        )
        && write_guest_bytes(
            kernel,
            memory,
            thread_id,
            out_ptr.wrapping_add(platform_offset),
            PLATFORM_ID,
        )
        && write_optional_count(kernel, memory, thread_id, returned_ptr, required_size);
    if ok {
        kernel.threads.set_last_error(thread_id, 0);
    }
    ok
}

fn set_file_pointer_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let handle = raw_arg(args, 0);
    let low = raw_arg(args, 1);
    let high_ptr = raw_arg(args, 2);
    let method = raw_arg(args, 3);
    let distance = if high_ptr == 0 {
        low as i32 as i64
    } else {
        let high = match read_guest_u32(kernel, memory, thread_id, high_ptr) {
            Some(high) => high,
            None => return u32::MAX,
        };
        (((high as u64) << 32) | low as u64) as i64
    };
    let position = match kernel.set_file_pointer(handle, distance, method) {
        Ok(position) => position as u64,
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            return u32::MAX;
        }
    };
    if high_ptr != 0
        && !write_guest_u32(kernel, memory, thread_id, high_ptr, (position >> 32) as u32)
    {
        return u32::MAX;
    }
    position as u32
}

fn get_file_size_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let size = match kernel.get_file_size(raw_arg(args, 0)) {
        Ok(size) => size as u64,
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            return u32::MAX;
        }
    };
    let high_ptr = raw_arg(args, 1);
    if high_ptr != 0 && !write_guest_u32(kernel, memory, thread_id, high_ptr, (size >> 32) as u32) {
        return u32::MAX;
    }
    size as u32
}

fn flush_file_buffers_raw(kernel: &mut CeKernel, thread_id: u32, handle: u32) -> bool {
    match kernel.flush_file_buffers(handle) {
        Ok(ok) => ok,
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            false
        }
    }
}

fn create_process_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let application = match read_optional_guest_wide_arg(memory, raw_arg(args, 0)) {
        Some(value) => value,
        None => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            return false;
        }
    };
    let command_line = match read_optional_guest_wide_arg(memory, raw_arg(args, 1)) {
        Some(value) => value,
        None => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            return false;
        }
    };
    let process_information = raw_arg(args, 9);
    if process_information == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }

    let launch = kernel.queue_process_launch(application, command_line);
    for (offset, value) in [
        (0, launch.process_handle),
        (4, launch.thread_handle),
        (8, launch.process_id),
        (12, launch.thread_id),
    ] {
        if !write_guest_u32(
            kernel,
            memory,
            thread_id,
            process_information + offset,
            value,
        ) {
            return false;
        }
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn wait_for_multiple_objects_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let count = raw_arg(args, 0);
    let handles_ptr = raw_arg(args, 1);
    let wait_all = raw_arg(args, 2) != 0;
    let timeout_ms = raw_arg(args, 3);
    if count == 0 || handles_ptr == 0 || count > 64 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return crate::ce::timer::WAIT_FAILED;
    }
    if wait_all {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return crate::ce::timer::WAIT_FAILED;
    }

    let mut handles = Vec::with_capacity(count as usize);
    for index in 0..count {
        let Some(handle) = read_guest_u32(
            kernel,
            memory,
            thread_id,
            handles_ptr.wrapping_add(index * 4),
        ) else {
            return crate::ce::timer::WAIT_FAILED;
        };
        handles.push(handle);
    }

    let result = kernel.wait_for_multiple_objects(&handles, wait_all, timeout_ms, thread_id);
    if result == crate::ce::timer::WAIT_FAILED {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
    } else {
        kernel.threads.set_last_error(thread_id, 0);
    }
    result
}

fn msg_wait_for_multiple_objects_ex_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let count = raw_arg(args, 0);
    let handles_ptr = raw_arg(args, 1);
    let timeout_ms = raw_arg(args, 2);
    let wake_mask = raw_arg(args, 3);
    let flags = raw_arg(args, 4);
    const MWMO_WAITALL: u32 = 0x0001;
    const MWMO_INPUTAVAILABLE: u32 = 0x0004;
    const MAXIMUM_WAIT_OBJECTS: u32 = 64;

    if count > MAXIMUM_WAIT_OBJECTS || (count != 0 && handles_ptr == 0) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        kernel.record_msg_wait_result(count, timeout_ms, crate::ce::timer::WAIT_FAILED);
        return crate::ce::timer::WAIT_FAILED;
    }

    let mut handles = Vec::with_capacity(count as usize);
    for index in 0..count {
        let Some(handle) = read_guest_u32(
            kernel,
            memory,
            thread_id,
            handles_ptr.wrapping_add(index * 4),
        ) else {
            return crate::ce::timer::WAIT_FAILED;
        };
        handles.push(handle);
    }

    if !handles.is_empty() {
        let result =
            kernel.wait_for_multiple_objects(&handles, flags & MWMO_WAITALL != 0, 0, thread_id);
        if result != crate::ce::timer::WAIT_TIMEOUT {
            if result == crate::ce::timer::WAIT_FAILED {
                kernel
                    .threads
                    .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            } else {
                kernel.threads.set_last_error(thread_id, 0);
            }
            kernel.record_msg_wait_result(count, timeout_ms, result);
            return result;
        }
    }

    kernel.pump_timers_to_gwe(thread_id);
    let has_input = if flags & MWMO_INPUTAVAILABLE != 0 {
        kernel.gwe.has_queue_input(thread_id, wake_mask)
    } else {
        kernel.gwe.has_new_queue_input(thread_id, wake_mask)
    };
    if has_input {
        if flags & MWMO_INPUTAVAILABLE == 0 {
            kernel.gwe.clear_new_queue_input(thread_id, wake_mask);
        }
        kernel.threads.set_last_error(thread_id, 0);
        kernel.record_msg_wait_input(count, timeout_ms);
        return crate::ce::timer::WAIT_OBJECT_0 + count;
    }

    let result = if timeout_ms == 0 {
        crate::ce::timer::WAIT_TIMEOUT
    } else {
        crate::ce::timer::WAIT_TIMEOUT
    };
    kernel.threads.set_last_error(thread_id, 0);
    kernel.record_msg_wait_result(count, timeout_ms, result);
    result
}

fn local_alloc_raw(kernel: &mut CeKernel, thread_id: u32, flags: u32, bytes: u32) -> u32 {
    match kernel.memory.local_alloc(flags, bytes) {
        Some(ptr) => ptr,
        None => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            0
        }
    }
}

fn local_re_alloc_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    ptr: u32,
    bytes: u32,
    flags: u32,
) -> u32 {
    match kernel.memory.local_re_alloc_detail(ptr, bytes, flags) {
        Some(result) => copy_reallocated_bytes(kernel, memory, thread_id, &result),
        None => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            0
        }
    }
}

fn local_size_raw(kernel: &mut CeKernel, thread_id: u32, ptr: u32) -> u32 {
    match kernel.memory.local_size(ptr) {
        Some(size) => size,
        None => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            0
        }
    }
}

fn local_free_raw(kernel: &mut CeKernel, thread_id: u32, ptr: u32) -> u32 {
    if kernel.memory.local_free(ptr) {
        0
    } else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        ptr
    }
}

fn heap_create_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    options: u32,
    initial_size: u32,
    maximum_size: u32,
) -> u32 {
    match kernel
        .memory
        .heap_create(options, initial_size, maximum_size)
    {
        Some(heap) => heap,
        None => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            0
        }
    }
}

fn heap_destroy_raw(kernel: &mut CeKernel, thread_id: u32, heap: u32) -> bool {
    if kernel.memory.heap_destroy(heap) {
        true
    } else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        false
    }
}

fn heap_alloc_raw(kernel: &mut CeKernel, thread_id: u32, heap: u32, flags: u32, bytes: u32) -> u32 {
    match kernel.memory.heap_alloc(heap, flags, bytes) {
        Some(ptr) => ptr,
        None => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            0
        }
    }
}

fn heap_re_alloc_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    heap: u32,
    flags: u32,
    ptr: u32,
    bytes: u32,
) -> u32 {
    match kernel.memory.heap_re_alloc_detail(heap, flags, ptr, bytes) {
        Some(result) => copy_reallocated_bytes(kernel, memory, thread_id, &result),
        None => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            0
        }
    }
}

fn copy_reallocated_bytes<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    result: &crate::ce::memory::Reallocation,
) -> u32 {
    if !result.moved {
        return result.ptr;
    }
    let copy_len = result.old_actual_size.min(result.new_actual_size);
    if copy_len == 0 {
        return result.ptr;
    }
    if let Some(bytes) = read_guest_bytes(kernel, memory, thread_id, result.old_ptr, copy_len) {
        if !write_guest_bytes(kernel, memory, thread_id, result.ptr, &bytes) {
            return 0;
        }
    }
    result.ptr
}

fn heap_size_raw(kernel: &mut CeKernel, thread_id: u32, heap: u32, flags: u32, ptr: u32) -> u32 {
    match kernel.memory.heap_size(heap, flags, ptr) {
        Some(size) => size,
        None => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            u32::MAX
        }
    }
}

fn heap_free_raw(kernel: &mut CeKernel, thread_id: u32, heap: u32, flags: u32, ptr: u32) -> bool {
    if kernel.memory.heap_free(heap, flags, ptr) {
        true
    } else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        false
    }
}

fn virtual_alloc_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    address: u32,
    size: u32,
    allocation_type: u32,
    protect: u32,
) -> u32 {
    match kernel
        .memory
        .virtual_alloc(address, size, allocation_type, protect)
    {
        Some(ptr) => ptr,
        None => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_NOT_ENOUGH_MEMORY);
            0
        }
    }
}

fn virtual_free_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    address: u32,
    size: u32,
    free_type: u32,
) -> bool {
    if kernel.memory.virtual_free(address, size, free_type) {
        true
    } else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PointerProbe {
    Read,
    Write,
}

fn is_bad_ptr_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    ptr: u32,
    bytes: u32,
    probe: PointerProbe,
) -> bool {
    if bytes == 0 {
        kernel.threads.set_last_error(thread_id, 0);
        return false;
    }
    if ptr == 0 || ptr.checked_add(bytes).is_none() {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return true;
    }
    if kernel.memory.contains_allocated_range(ptr, bytes) {
        kernel.threads.set_last_error(thread_id, 0);
        return false;
    }
    if probe_guest_range(memory, ptr, bytes, probe) {
        kernel.threads.set_last_error(thread_id, 0);
        false
    } else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        true
    }
}

fn probe_guest_range<M: CoredllGuestMemory>(
    memory: &mut M,
    ptr: u32,
    bytes: u32,
    probe: PointerProbe,
) -> bool {
    let end = ptr + bytes - 1;
    let mut addr = ptr;
    loop {
        if !probe_guest_byte(memory, addr, probe) {
            return false;
        }
        if addr == end {
            return true;
        }
        let next_page = (addr & !0xfff).saturating_add(0x1000);
        let next = next_page.min(end);
        if next == addr {
            return true;
        }
        addr = next;
    }
}

fn probe_guest_byte<M: CoredllGuestMemory>(memory: &mut M, addr: u32, probe: PointerProbe) -> bool {
    let Ok(value) = memory.read_u8(addr) else {
        return false;
    };
    match probe {
        PointerProbe::Read => true,
        PointerProbe::Write => memory.write_u8(addr, value).is_ok(),
    }
}

fn create_file_mapping_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    const INVALID_HANDLE_VALUE: u32 = 0xffff_ffff;

    let file_handle = raw_arg(args, 0);
    let protect = raw_arg(args, 2);
    let size_high = raw_arg(args, 3);
    let size_low = raw_arg(args, 4);
    if size_high != 0 || size_low == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let name = read_optional_guest_wide_arg(memory, raw_arg(args, 5)).and_then(|name| name);
    let file_id = if file_handle == 0 || file_handle == INVALID_HANDLE_VALUE {
        None
    } else {
        let Ok(KernelObject::File(file)) = kernel.handles.get(file_handle) else {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            return 0;
        };
        Some(file.file_id)
    };
    let handle = kernel
        .handles
        .create_file_mapping(name, size_low.max(1), protect, file_id);
    kernel.threads.set_last_error(thread_id, 0);
    handle
}

fn map_view_of_file_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    _memory: &mut M,
    thread_id: u32,
    mapping_handle: u32,
    offset_high: u32,
    offset_low: u32,
    bytes_to_map: u32,
) -> u32 {
    const MEM_COMMIT: u32 = 0x0000_1000;
    const MEM_RESERVE: u32 = 0x0000_2000;
    const PAGE_READWRITE: u32 = 0x04;

    if offset_high != 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let Ok(mapping) = kernel.handles.file_mapping(mapping_handle) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    if let Some(base) = mapping.view_base {
        kernel.threads.set_last_error(thread_id, 0);
        return base;
    }

    let size = if bytes_to_map == 0 {
        mapping.size
    } else {
        bytes_to_map.min(mapping.size)
    };
    let file_id = mapping.file_id;
    let mut initial_bytes = if let Some(file_id) = file_id {
        match kernel.read_file_at(file_id, offset_low as usize, size as usize) {
            Ok(mut bytes) => {
                bytes.resize(size as usize, 0);
                bytes
            }
            Err(_) => {
                kernel
                    .threads
                    .set_last_error(thread_id, ERROR_INVALID_HANDLE);
                return 0;
            }
        }
    } else {
        let start = offset_low as usize;
        let end = start.saturating_add(size as usize).min(mapping.data.len());
        let mut bytes = if start < mapping.data.len() {
            mapping.data[start..end].to_vec()
        } else {
            Vec::new()
        };
        bytes.resize(size as usize, 0);
        bytes
    };
    let mapping_bytes = initial_bytes.clone();
    let Some(base) = kernel
        .memory
        .virtual_alloc(0, size, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE)
    else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    };
    kernel
        .memory
        .set_virtual_initial_bytes(base, std::mem::take(&mut initial_bytes));
    let Ok(mapping) = kernel.handles.file_mapping_mut(mapping_handle) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    let start = offset_low as usize;
    let end = start.saturating_add(mapping_bytes.len());
    if end > mapping.data.len() {
        mapping.data.resize(end, 0);
    }
    mapping.data[start..end].copy_from_slice(&mapping_bytes);
    mapping.view_base = Some(base);
    mapping.view_size = size;
    mapping.view_offset = offset_low;
    kernel.threads.set_last_error(thread_id, 0);
    base
}

fn flush_view_of_file_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    base: u32,
    bytes_to_flush: u32,
) -> bool {
    let Some(mapping) = kernel.handles.file_mapping_by_view(base).cloned() else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    };
    let count = if bytes_to_flush == 0 {
        mapping.view_size
    } else {
        bytes_to_flush.min(mapping.view_size)
    };
    let Some(bytes) = read_guest_bytes(kernel, memory, thread_id, base, count) else {
        return false;
    };
    if let Some(mapping) = kernel.handles.file_mapping_by_view_mut(base) {
        let start = mapping.view_offset as usize;
        let end = start.saturating_add(bytes.len());
        if end > mapping.data.len() {
            mapping.data.resize(end, 0);
        }
        mapping.data[start..end].copy_from_slice(&bytes);
    }
    let Some(file_id) = mapping.file_id else {
        kernel.threads.set_last_error(thread_id, 0);
        return true;
    };
    match kernel.write_file_at(file_id, mapping.view_offset as usize, &bytes) {
        Ok(result) if result.success && result.bytes_transferred == count => {
            kernel.threads.set_last_error(thread_id, 0);
            true
        }
        _ => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            false
        }
    }
}

fn unmap_view_of_file_raw(kernel: &mut CeKernel, thread_id: u32, base: u32) -> bool {
    if kernel.handles.has_file_mapping_view(base) {
        kernel.threads.set_last_error(thread_id, 0);
        true
    } else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        false
    }
}

fn create_mutex_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let name_ptr = raw_arg(args, 2);
    let name = if name_ptr == 0 {
        None
    } else {
        let Some(name) = read_guest_wide_arg(memory, name_ptr) else {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            return 0;
        };
        Some(name)
    };
    let owner = (raw_arg(args, 1) != 0).then_some(thread_id);
    let (handle, existed) = kernel.create_mutex_w_with_status(name, owner);
    kernel
        .threads
        .set_last_error(thread_id, if existed { ERROR_ALREADY_EXISTS } else { 0 });
    handle
}

fn create_event_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let name_ptr = raw_arg(args, 3);
    let name = if name_ptr == 0 {
        None
    } else {
        let Some(name) = read_guest_wide_arg(memory, name_ptr) else {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            return 0;
        };
        Some(name)
    };
    let handle = kernel.create_event_w(name, raw_arg(args, 1) != 0, raw_arg(args, 2) != 0);
    kernel.threads.set_last_error(thread_id, 0);
    handle
}

fn open_event_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    if raw_arg(args, 0) != EVENT_ALL_ACCESS || raw_arg(args, 1) != 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let name_ptr = raw_arg(args, 2);
    if name_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let Some(name) = read_guest_wide_arg(memory, name_ptr) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    if let Some(handle) = kernel.open_event_w(&name) {
        kernel.threads.set_last_error(thread_id, 0);
        handle
    } else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_FILE_NOT_FOUND);
        0
    }
}

fn create_semaphore_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let initial_count = raw_i32_arg(args, 1);
    let maximum_count = raw_i32_arg(args, 2);
    let name_ptr = raw_arg(args, 3);
    let name = if name_ptr == 0 {
        None
    } else {
        let Some(name) = read_guest_wide_arg(memory, name_ptr) else {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            return 0;
        };
        Some(name)
    };
    let Some(handle) = kernel.create_semaphore_w(name, initial_count, maximum_count) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    kernel.threads.set_last_error(thread_id, 0);
    handle
}

fn create_thread_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let start_address = raw_arg(args, 2);
    if start_address == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let parameter = raw_arg(args, 3);
    let suspended = raw_arg(args, 4) & 0x0000_0004 != 0;
    let thread_id_ptr = raw_arg(args, 5);
    let (handle, created_thread_id) =
        kernel.create_guest_thread(start_address, parameter, suspended);
    if !write_optional_count(kernel, memory, thread_id, thread_id_ptr, created_thread_id) {
        let _ = kernel.close_handle(handle);
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    handle
}

fn suspend_thread_raw(kernel: &mut CeKernel, thread_id: u32, handle: u32) -> u32 {
    match kernel.suspend_thread_for_handle(handle, thread_id) {
        ThreadSuspendResult::Previous(previous) => {
            kernel.threads.set_last_error(thread_id, 0);
            previous
        }
        ThreadSuspendResult::InvalidHandle => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            u32::MAX
        }
        ThreadSuspendResult::SignalRefused => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_SIGNAL_REFUSED);
            u32::MAX
        }
    }
}

fn resume_thread_raw(kernel: &mut CeKernel, thread_id: u32, handle: u32) -> u32 {
    match kernel.resume_thread_for_handle(handle, thread_id) {
        ThreadResumeResult::Previous(previous) => {
            kernel.threads.set_last_error(thread_id, 0);
            previous
        }
        ThreadResumeResult::InvalidHandle => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            u32::MAX
        }
    }
}

fn get_thread_id_raw(kernel: &mut CeKernel, thread_id: u32, handle: u32) -> u32 {
    match kernel.guest_thread_id_for_handle(handle, thread_id) {
        Some(target_thread_id) => {
            kernel.threads.set_last_error(thread_id, 0);
            target_thread_id
        }
        None => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            0
        }
    }
}

fn get_thread_exit_code_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let handle = raw_arg(args, 0);
    let exit_code_ptr = raw_arg(args, 1);
    if exit_code_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    let Some(exit_code) = kernel.guest_thread_exit_code_for_handle(handle, thread_id) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return false;
    };
    if !write_guest_u32(kernel, memory, thread_id, exit_code_ptr, exit_code) {
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn get_process_exit_code_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let handle = raw_arg(args, 0);
    let exit_code_ptr = raw_arg(args, 1);
    if exit_code_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    let Some(exit_code) = kernel.process_exit_code_for_handle(handle) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return false;
    };
    if !write_guest_u32(kernel, memory, thread_id, exit_code_ptr, exit_code) {
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn get_process_version_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    _process_id_or_module: u32,
) -> u32 {
    kernel.threads.set_last_error(thread_id, 0);
    0x0004_0014
}

fn get_process_id_raw(kernel: &mut CeKernel, thread_id: u32, handle: u32) -> u32 {
    match kernel.process_id_for_handle(handle) {
        Some(process_id) => {
            kernel.threads.set_last_error(thread_id, 0);
            process_id
        }
        None => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            0
        }
    }
}

fn get_thread_times_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let handle = raw_arg(args, 0);
    if kernel
        .guest_thread_id_for_handle(handle, thread_id)
        .is_none()
    {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return false;
    }

    for filetime_ptr in [
        raw_arg(args, 1),
        raw_arg(args, 2),
        raw_arg(args, 3),
        raw_arg(args, 4),
    ] {
        if filetime_ptr != 0 {
            if !write_guest_u32(kernel, memory, thread_id, filetime_ptr, 0)
                || !write_guest_u32(kernel, memory, thread_id, filetime_ptr + 4, 0)
            {
                return false;
            }
        }
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn terminate_process_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    handle: u32,
    exit_code: u32,
) -> bool {
    if kernel.terminate_process(handle, exit_code) {
        kernel.threads.set_last_error(thread_id, 0);
        true
    } else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        false
    }
}

fn get_thread_priority_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    handle: u32,
    ce_priority: bool,
) -> u32 {
    const THREAD_PRIORITY_ERROR_RETURN: u32 = 0x7fff_ffff;

    let priority = if ce_priority {
        kernel
            .thread_priority_for_handle(handle, thread_id)
            .map(|priority| priority as u32)
    } else {
        kernel.thread_win32_priority_for_handle(handle, thread_id)
    };

    match priority {
        Some(priority) => {
            kernel.threads.set_last_error(thread_id, 0);
            priority
        }
        None => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            THREAD_PRIORITY_ERROR_RETURN
        }
    }
}

fn set_thread_priority_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    handle: u32,
    priority: u32,
    ce_priority: bool,
) -> bool {
    let priority_is_valid = if ce_priority {
        priority < crate::ce::object::MAX_CE_PRIORITY_LEVELS as u32
    } else {
        crate::ce::object::win32_thread_priority_to_ce(priority).is_some()
    };
    if !priority_is_valid {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }

    let success = if ce_priority {
        kernel.set_thread_ce_priority_for_handle(handle, priority as i32, thread_id)
    } else {
        kernel.set_thread_win32_priority_for_handle(handle, priority, thread_id)
    };

    if success {
        kernel.threads.set_last_error(thread_id, 0);
        true
    } else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        false
    }
}

fn release_mutex_raw(kernel: &mut CeKernel, thread_id: u32, handle: u32) -> bool {
    if kernel.release_mutex(handle, thread_id) {
        kernel.threads.set_last_error(thread_id, 0);
        true
    } else if kernel.is_mutex_handle(handle) {
        kernel.threads.set_last_error(thread_id, ERROR_NOT_OWNER);
        false
    } else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        false
    }
}

fn release_semaphore_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let handle = raw_arg(args, 0);
    let release_count = raw_i32_arg(args, 1);
    let previous_ptr = raw_arg(args, 2);
    let Some(previous) = kernel.release_semaphore(handle, release_count) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    };
    if !write_optional_count(kernel, memory, thread_id, previous_ptr, previous as u32) {
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn register_class_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    class_ptr: u32,
) -> u32 {
    if class_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let Some(bytes) = read_guest_bytes(kernel, memory, thread_id, class_ptr, WNDCLASSW_SIZE as u32)
    else {
        return 0;
    };
    let mut wndclass = [0; WNDCLASSW_SIZE];
    wndclass.copy_from_slice(&bytes);
    let class_name_ptr =
        u32::from_le_bytes([wndclass[36], wndclass[37], wndclass[38], wndclass[39]]);
    let class_name = read_guest_wide_arg(memory, class_name_ptr).unwrap_or_default();
    if class_name.is_empty() {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let wndproc = u32::from_le_bytes([wndclass[4], wndclass[5], wndclass[6], wndclass[7]]);
    tracing::debug!(
        target: "ce.gwe",
        class_name = class_name.as_str(),
        class_name_ptr = format_args!("0x{class_name_ptr:08x}"),
        wndproc = format_args!("0x{wndproc:08x}"),
        "RegisterClassW"
    );
    let atom = kernel.gwe.register_class(&class_name, wndclass);
    kernel.threads.set_last_error(thread_id, 0);
    u32::from(atom)
}

fn get_class_info_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    class_name_ptr: u32,
    out_ptr: u32,
) -> bool {
    let Some(class_name) = read_guest_wide_arg(memory, class_name_ptr) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_CLASS_DOES_NOT_EXIST);
        return false;
    };
    if class_name.is_empty() {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_CLASS_DOES_NOT_EXIST);
        return false;
    }
    tracing::debug!(
        target: "ce.gwe",
        class_name = class_name.as_str(),
        class_name_ptr = format_args!("0x{class_name_ptr:08x}"),
        out_ptr = format_args!("0x{out_ptr:08x}"),
        "GetClassInfoW"
    );
    let Some(bytes) = kernel.gwe.class_info(&class_name).map(|class| class.bytes) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_CLASS_DOES_NOT_EXIST);
        return false;
    };
    if out_ptr == 0 || !write_guest_bytes(kernel, memory, thread_id, out_ptr, &bytes) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_CLASS_DOES_NOT_EXIST);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn register_window_message_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    name_ptr: u32,
) -> u32 {
    let Some(name) = read_guest_wide_arg(memory, name_ptr) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let Some(message) = kernel.gwe.register_window_message(&name) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    kernel.threads.set_last_error(thread_id, 0);
    message
}

const REGISTER_GESTURE_BYTES: u32 = 0x400;

fn register_gesture_raw(kernel: &mut CeKernel, thread_id: u32, args: &[u32]) -> u32 {
    let id = raw_arg(args, 0);
    let arg1 = raw_arg(args, 1);
    let arg2 = raw_arg(args, 2);
    let arg3 = raw_arg(args, 3);
    tracing::debug!(
        target: "ce.gwe",
        id = format_args!("0x{id:08x}"),
        arg1 = format_args!("0x{arg1:08x}"),
        arg2 = format_args!("0x{arg2:08x}"),
        arg3 = format_args!("0x{arg3:08x}"),
        "RegisterGesture"
    );
    if id == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let Some(handle) = kernel.memory.heap_alloc(
        PROCESS_HEAP_HANDLE,
        HEAP_ZERO_MEMORY,
        REGISTER_GESTURE_BYTES,
    ) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    };
    if !kernel.gwe.register_gesture(id, handle, arg1, arg2, arg3) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    handle
}

fn find_window_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    class_name_ptr: u32,
    title_ptr: u32,
) -> u32 {
    let class_name = read_guest_wide_arg(memory, class_name_ptr);
    let title = read_guest_wide_arg(memory, title_ptr);
    let hwnd = kernel
        .gwe
        .find_window(class_name.as_deref(), title.as_deref())
        .unwrap_or(0);
    if hwnd == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
    } else {
        kernel.threads.set_last_error(thread_id, 0);
    }
    hwnd
}

fn create_window_ex_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let Some(class_name) = read_guest_wide_arg(memory, raw_arg(args, 1)) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    if class_name.is_empty() {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_CLASS_DOES_NOT_EXIST);
        return 0;
    }
    let title = read_guest_wide_arg(memory, raw_arg(args, 2)).unwrap_or_default();
    let rect = Rect::from_origin_size(
        raw_i32_arg(args, 4),
        raw_i32_arg(args, 5),
        raw_i32_arg(args, 6),
        raw_i32_arg(args, 7),
    );
    let parent_or_owner = (raw_arg(args, 8) != 0).then_some(raw_arg(args, 8));
    let style = raw_arg(args, 3);
    let parent = parent_or_owner.filter(|_| style & WS_CHILD != 0);
    let owner = parent_or_owner.filter(|_| style & WS_CHILD == 0);
    let id = if style & WS_CHILD != 0 {
        raw_arg(args, 9)
    } else {
        0
    };
    let menu = if style & WS_CHILD == 0 {
        raw_arg(args, 9)
    } else {
        0
    };
    tracing::debug!(
        target: "ce.gwe",
        class_name = class_name.as_str(),
        class_ptr = format_args!("0x{:08x}", raw_arg(args, 1)),
        title = title.as_str(),
        style = format_args!("0x{:08x}", style),
        ex_style = format_args!("0x{:08x}", raw_arg(args, 0)),
        x = raw_i32_arg(args, 4),
        y = raw_i32_arg(args, 5),
        width = raw_i32_arg(args, 6),
        height = raw_i32_arg(args, 7),
        parent = format_args!("0x{:08x}", raw_arg(args, 8)),
        id = format_args!("0x{:08x}", raw_arg(args, 9)),
        create_params = format_args!("0x{:08x}", raw_arg(args, 11)),
        "CreateWindowExW"
    );
    let hwnd = kernel.create_window_ex_w_with_parent_owner_and_rect(
        thread_id,
        &class_name,
        &title,
        parent,
        owner,
        id,
        style,
        raw_arg(args, 0),
        rect,
    );
    if hwnd != 0 && menu != 0 {
        let _ = kernel.gwe.set_menu(hwnd, menu);
    }
    hwnd
}

fn read_guest_wide_arg<M: CoredllGuestMemory>(memory: &M, ptr: u32) -> Option<String> {
    if ptr == 0 {
        return None;
    }
    if ptr <= 0xffff {
        return Some(format!("#{ptr}"));
    }
    read_guest_wide_z(memory, ptr, 512).ok()
}

fn read_optional_guest_wide_arg<M: CoredllGuestMemory>(
    memory: &M,
    ptr: u32,
) -> Option<Option<String>> {
    if ptr == 0 {
        Some(None)
    } else {
        read_guest_wide_arg(memory, ptr).map(Some)
    }
}

fn read_guest_ascii_z<M: CoredllGuestMemory>(
    memory: &M,
    addr: u32,
    max_bytes: usize,
) -> Option<String> {
    if addr == 0 {
        return None;
    }
    let mut bytes = Vec::new();
    for index in 0..max_bytes {
        let byte = memory.read_u8(addr.wrapping_add(index as u32)).ok()?;
        if byte == 0 {
            break;
        }
        bytes.push(byte);
    }
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

fn set_window_text_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let Some(title) = read_guest_wide_arg(memory, raw_arg(args, 1)) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    };
    if kernel.gwe.set_window_text(raw_arg(args, 0), &title) {
        true
    } else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        false
    }
}

fn get_window_text_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let hwnd = raw_arg(args, 0);
    let buffer = raw_arg(args, 1);
    let capacity = raw_arg(args, 2) as usize;
    let Some(text) = kernel.gwe.get_window_text(hwnd, capacity) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    write_wide_result(kernel, memory, thread_id, buffer, capacity, &text)
}

fn get_class_name_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let hwnd = raw_arg(args, 0);
    let buffer = raw_arg(args, 1);
    let capacity = raw_arg(args, 2) as usize;
    let Some(class_name) = kernel.gwe.get_class_name(hwnd, capacity) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    write_wide_result(kernel, memory, thread_id, buffer, capacity, &class_name)
}

fn set_window_long_w_raw(kernel: &mut CeKernel, args: &[u32]) -> u32 {
    let hwnd = raw_arg(args, 0);
    let index = raw_i32_arg(args, 1);
    let value = raw_arg(args, 2);
    let class_name = kernel
        .gwe
        .window(hwnd)
        .map(|window| window.class_name.clone());
    let previous = kernel.gwe.set_window_long(hwnd, index, value).unwrap_or(0);
    tracing::debug!(
        target: "ce.gwe",
        hwnd = format_args!("0x{hwnd:08x}"),
        index,
        index_name = window_long_index_name(index),
        value = format_args!("0x{value:08x}"),
        previous = format_args!("0x{previous:08x}"),
        class = class_name.as_deref().unwrap_or("<invalid>"),
        "SetWindowLongW"
    );
    previous
}

fn get_window_long_w_raw(kernel: &mut CeKernel, args: &[u32]) -> u32 {
    let hwnd = raw_arg(args, 0);
    let index = raw_i32_arg(args, 1);
    let class_name = kernel
        .gwe
        .window(hwnd)
        .map(|window| window.class_name.clone());
    let value = kernel.gwe.get_window_long(hwnd, index).unwrap_or(0);
    tracing::debug!(
        target: "ce.gwe",
        hwnd = format_args!("0x{hwnd:08x}"),
        index,
        index_name = window_long_index_name(index),
        value = format_args!("0x{value:08x}"),
        class = class_name.as_deref().unwrap_or("<invalid>"),
        "GetWindowLongW"
    );
    value
}

fn window_long_index_name(index: i32) -> &'static str {
    match index {
        crate::ce::gwe::GWL_WNDPROC => "GWL_WNDPROC",
        crate::ce::gwe::GWL_ID => "GWL_ID",
        crate::ce::gwe::GWL_STYLE => "GWL_STYLE",
        crate::ce::gwe::GWL_EXSTYLE => "GWL_EXSTYLE",
        crate::ce::gwe::GWL_USERDATA => "GWL_USERDATA",
        crate::ce::gwe::DWL_MSGRESULT => "DWL_MSGRESULT",
        crate::ce::gwe::DWL_DLGPROC => "DWL_DLGPROC",
        crate::ce::gwe::DWL_USER => "DWL_USER",
        _ => "<unknown>",
    }
}

fn create_dialog_indirect_param_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let template_ptr = raw_arg(args, 1);
    let parent = (raw_arg(args, 2) != 0).then_some(raw_arg(args, 2));
    let dlgproc = raw_arg(args, 3);
    if template_ptr == 0 || dlgproc == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let Some(template) = read_dialog_template(kernel, memory, thread_id, template_ptr) else {
        return 0;
    };
    let hwnd = kernel.gwe.create_window_ex_with_process_and_rect(
        thread_id,
        kernel.current_process_id(),
        "Dialog",
        &template.title,
        parent,
        0,
        template.style,
        template.ex_style,
        Rect::from_origin_size(template.x, template.y, template.cx, template.cy),
    );
    kernel
        .gwe
        .set_window_long(hwnd, crate::ce::gwe::GWL_WNDPROC, dlgproc);
    for control in template.controls {
        let control_hwnd = kernel.gwe.create_window_ex_with_process_and_rect(
            thread_id,
            kernel.current_process_id(),
            &control.class_name,
            &control.title,
            Some(hwnd),
            control.id,
            control.style,
            control.ex_style,
            Rect::from_origin_size(control.x, control.y, control.cx, control.cy),
        );
        if control.class_name.eq_ignore_ascii_case("button") && control.title.is_empty() {
            kernel.gwe.set_window_text(control_hwnd, "Button");
        }
    }
    kernel.threads.set_last_error(thread_id, 0);
    hwnd
}

fn end_dialog_raw(kernel: &mut CeKernel, thread_id: u32, hwnd: u32, result: u32) -> bool {
    if !kernel.gwe.end_dialog(hwnd, result) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn set_dlg_item_text_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let hwnd = raw_arg(args, 0);
    let id = raw_arg(args, 1);
    let text_ptr = raw_arg(args, 2);
    let Some(text) = read_guest_wide_arg(memory, text_ptr) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    };
    let Some(child) = kernel.gwe.get_dlg_item(hwnd, id) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return false;
    };
    if !kernel.gwe.set_window_text(child, &text) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn get_dlg_item_text_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let hwnd = raw_arg(args, 0);
    let id = raw_arg(args, 1);
    let buffer = raw_arg(args, 2);
    let capacity = raw_arg(args, 3) as usize;
    let Some(child) = kernel.gwe.get_dlg_item(hwnd, id) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    };
    let Some(text) = kernel.gwe.get_window_text(child, capacity) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    };
    write_wide_result(kernel, memory, thread_id, buffer, capacity, &text)
}

fn set_dlg_item_int_raw(kernel: &mut CeKernel, thread_id: u32, args: &[u32]) -> bool {
    let hwnd = raw_arg(args, 0);
    let id = raw_arg(args, 1);
    let value = raw_arg(args, 2);
    let signed = raw_arg(args, 3) != 0;
    let text = if signed {
        (value as i32).to_string()
    } else {
        value.to_string()
    };
    let Some(child) = kernel.gwe.get_dlg_item(hwnd, id) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return false;
    };
    if !kernel.gwe.set_window_text(child, &text) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn get_dlg_item_int_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let hwnd = raw_arg(args, 0);
    let id = raw_arg(args, 1);
    let ok_ptr = raw_arg(args, 2);
    let signed = raw_arg(args, 3) != 0;
    let Some(child) = kernel.gwe.get_dlg_item(hwnd, id) else {
        let _ = write_optional_u32(kernel, memory, thread_id, ok_ptr, 0);
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    };
    let Some(text) = kernel.gwe.get_window_text(child, 64) else {
        let _ = write_optional_u32(kernel, memory, thread_id, ok_ptr, 0);
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    };
    let Some(value) = parse_dlg_item_int(&text, signed) else {
        let _ = write_optional_u32(kernel, memory, thread_id, ok_ptr, 0);
        kernel.threads.set_last_error(thread_id, 0);
        return 0;
    };
    if !write_optional_u32(kernel, memory, thread_id, ok_ptr, 1) {
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    value
}

fn parse_dlg_item_int(text: &str, signed: bool) -> Option<u32> {
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return None;
    }
    if signed {
        let (negative, digits) = if let Some(rest) = trimmed.strip_prefix('-') {
            (true, rest)
        } else if let Some(rest) = trimmed.strip_prefix('+') {
            (false, rest)
        } else {
            (false, trimmed)
        };
        let digit_count = digits
            .as_bytes()
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count();
        if digit_count == 0 {
            return None;
        }
        let number = digits[..digit_count].parse::<i64>().ok()?;
        let signed_value = if negative { -number } else { number };
        i32::try_from(signed_value).ok().map(|value| value as u32)
    } else {
        let digits = trimmed.strip_prefix('+').unwrap_or(trimmed);
        let digit_count = digits
            .as_bytes()
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count();
        if digit_count == 0 {
            return None;
        }
        digits[..digit_count].parse::<u32>().ok()
    }
}

fn check_radio_button_raw(kernel: &mut CeKernel, thread_id: u32, args: &[u32]) -> bool {
    if !kernel.gwe.check_radio_button(
        raw_arg(args, 0),
        raw_arg(args, 1),
        raw_arg(args, 2),
        raw_arg(args, 3),
    ) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

const BM_GETCHECK: u32 = 0x00f0;
const BM_SETCHECK: u32 = 0x00f1;

fn send_message_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let hwnd = raw_arg(args, 0);
    let msg = raw_arg(args, 1);
    let wparam = raw_arg(args, 2);
    let lparam = raw_arg(args, 3);
    match msg {
        crate::ce::gwe::WM_SETTEXT => {
            let Some(text) = read_guest_wide_arg(memory, lparam) else {
                kernel
                    .threads
                    .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
                return 0;
            };
            if !kernel.gwe.set_window_text(hwnd, &text) {
                kernel
                    .threads
                    .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
                return 0;
            }
            kernel.threads.set_last_error(thread_id, 0);
            1
        }
        crate::ce::gwe::WM_GETTEXT => {
            let Some(text) = kernel.gwe.get_window_text(hwnd, wparam as usize) else {
                kernel
                    .threads
                    .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
                return 0;
            };
            write_wide_result(kernel, memory, thread_id, lparam, wparam as usize, &text)
        }
        crate::ce::gwe::WM_GETTEXTLENGTH => {
            let Some(length) = kernel.gwe.get_window_text_length(hwnd) else {
                kernel
                    .threads
                    .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
                return 0;
            };
            kernel.threads.set_last_error(thread_id, 0);
            length as u32
        }
        _ => {
            if let Some(result) = kernel.send_message_w(hwnd, msg, wparam, lparam) {
                kernel.threads.set_last_error(thread_id, 0);
                result
            } else {
                kernel
                    .threads
                    .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
                0
            }
        }
    }
}

fn send_dlg_item_message_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let hwnd = raw_arg(args, 0);
    let id = raw_arg(args, 1);
    let msg = raw_arg(args, 2);
    let wparam = raw_arg(args, 3);
    let lparam = raw_arg(args, 4);
    let Some(child) = kernel.gwe.get_dlg_item(hwnd, id) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    };

    let result = match msg {
        BM_GETCHECK => kernel.gwe.is_dlg_button_checked(hwnd, id).unwrap_or(0),
        BM_SETCHECK => {
            if !kernel.gwe.check_dlg_button(hwnd, id, wparam) {
                kernel
                    .threads
                    .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
                return 0;
            }
            0
        }
        _ => return send_message_w_raw(kernel, memory, thread_id, &[child, msg, wparam, lparam]),
    };
    kernel.threads.set_last_error(thread_id, 0);
    result
}

fn is_dialog_message_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let hwnd = raw_arg(args, 0);
    let msg_ptr = raw_arg(args, 1);
    if !kernel.gwe.is_window(hwnd) || msg_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    if read_guest_message(kernel, memory, thread_id, msg_ptr).is_none() {
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

#[derive(Debug, Clone)]
struct DialogTemplate {
    style: u32,
    ex_style: u32,
    x: i32,
    y: i32,
    cx: i32,
    cy: i32,
    title: String,
    controls: Vec<DialogControlTemplate>,
}

#[derive(Debug, Clone)]
struct DialogControlTemplate {
    style: u32,
    ex_style: u32,
    x: i32,
    y: i32,
    cx: i32,
    cy: i32,
    id: u32,
    class_name: String,
    title: String,
}

fn read_dialog_template<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    template_ptr: u32,
) -> Option<DialogTemplate> {
    let style = read_guest_u32(kernel, memory, thread_id, template_ptr)?;
    let ex_style = read_guest_u32(kernel, memory, thread_id, template_ptr.wrapping_add(4))?;
    let control_count = read_guest_u16(kernel, memory, thread_id, template_ptr.wrapping_add(8))?;
    let x = read_guest_i16(kernel, memory, thread_id, template_ptr.wrapping_add(10))? as i32;
    let y = read_guest_i16(kernel, memory, thread_id, template_ptr.wrapping_add(12))? as i32;
    let cx = read_guest_i16(kernel, memory, thread_id, template_ptr.wrapping_add(14))? as i32;
    let cy = read_guest_i16(kernel, memory, thread_id, template_ptr.wrapping_add(16))? as i32;
    let mut cursor = template_ptr.wrapping_add(18);
    skip_dialog_field(kernel, memory, thread_id, &mut cursor)?;
    skip_dialog_field(kernel, memory, thread_id, &mut cursor)?;
    let title = read_dialog_field_text(kernel, memory, thread_id, &mut cursor)?.unwrap_or_default();
    if style & DS_SETFONT != 0 {
        cursor = cursor.wrapping_add(2);
        skip_dialog_string(kernel, memory, thread_id, &mut cursor)?;
    }
    cursor = align_u32(cursor, 4);

    let mut controls = Vec::with_capacity(control_count as usize);
    for _ in 0..control_count {
        cursor = align_u32(cursor, 4);
        let control_style = read_guest_u32(kernel, memory, thread_id, cursor)?;
        let control_ex_style = read_guest_u32(kernel, memory, thread_id, cursor.wrapping_add(4))?;
        let control_x = read_guest_i16(kernel, memory, thread_id, cursor.wrapping_add(8))? as i32;
        let control_y = read_guest_i16(kernel, memory, thread_id, cursor.wrapping_add(10))? as i32;
        let control_cx = read_guest_i16(kernel, memory, thread_id, cursor.wrapping_add(12))? as i32;
        let control_cy = read_guest_i16(kernel, memory, thread_id, cursor.wrapping_add(14))? as i32;
        let id = u32::from(read_guest_u16(
            kernel,
            memory,
            thread_id,
            cursor.wrapping_add(16),
        )?);
        cursor = cursor.wrapping_add(18);
        let class_name =
            read_dialog_class_name(kernel, memory, thread_id, &mut cursor)?.unwrap_or_default();
        let title =
            read_dialog_field_text(kernel, memory, thread_id, &mut cursor)?.unwrap_or_default();
        let extra_size = read_guest_u16(kernel, memory, thread_id, cursor)?;
        cursor = cursor.wrapping_add(2 + u32::from(extra_size));
        controls.push(DialogControlTemplate {
            style: control_style,
            ex_style: control_ex_style,
            x: control_x,
            y: control_y,
            cx: control_cx,
            cy: control_cy,
            id,
            class_name,
            title,
        });
    }

    Some(DialogTemplate {
        style,
        ex_style,
        x,
        y,
        cx,
        cy,
        title,
        controls,
    })
}

fn skip_dialog_field<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    cursor: &mut u32,
) -> Option<()> {
    read_dialog_field_text(kernel, memory, thread_id, cursor).map(|_| ())
}

fn read_dialog_class_name<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    cursor: &mut u32,
) -> Option<Option<String>> {
    let value = read_guest_u16(kernel, memory, thread_id, *cursor)?;
    if value == 0xffff {
        let atom = read_guest_u16(kernel, memory, thread_id, cursor.wrapping_add(2))?;
        *cursor = cursor.wrapping_add(4);
        return Some(Some(dialog_class_atom_name(atom).to_owned()));
    }
    read_dialog_field_text(kernel, memory, thread_id, cursor)
}

fn read_dialog_field_text<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    cursor: &mut u32,
) -> Option<Option<String>> {
    let first = read_guest_u16(kernel, memory, thread_id, *cursor)?;
    if first == 0 {
        *cursor = cursor.wrapping_add(2);
        return Some(None);
    }
    if first == 0xffff {
        *cursor = cursor.wrapping_add(4);
        return Some(None);
    }
    let mut units = Vec::new();
    let mut scan = *cursor;
    loop {
        let unit = read_guest_u16(kernel, memory, thread_id, scan)?;
        scan = scan.wrapping_add(2);
        if unit == 0 {
            break;
        }
        units.push(unit);
        if units.len() > 512 {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            return None;
        }
    }
    *cursor = scan;
    Some(Some(String::from_utf16_lossy(&units)))
}

fn skip_dialog_string<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    cursor: &mut u32,
) -> Option<()> {
    read_dialog_field_text(kernel, memory, thread_id, cursor).map(|_| ())
}

fn dialog_class_atom_name(atom: u16) -> &'static str {
    match atom {
        0x80 => "button",
        0x81 => "edit",
        0x82 => "static",
        0x83 => "listbox",
        0x84 => "scrollbar",
        0x85 => "combobox",
        _ => "control",
    }
}

fn align_u32(value: u32, align: u32) -> u32 {
    if align <= 1 {
        value
    } else {
        value.wrapping_add(align - 1) & !(align - 1)
    }
}

fn write_wide_result<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    buffer: u32,
    capacity_chars: usize,
    text: &str,
) -> u32 {
    if buffer == 0 || capacity_chars == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let units: Vec<u16> = text
        .encode_utf16()
        .take(capacity_chars.saturating_sub(1))
        .collect();
    for (index, unit) in units.iter().copied().enumerate() {
        if !write_guest_u16(
            kernel,
            memory,
            thread_id,
            buffer.wrapping_add((index as u32) * 2),
            unit,
        ) {
            return 0;
        }
    }
    if !write_guest_u16(
        kernel,
        memory,
        thread_id,
        buffer.wrapping_add((units.len() as u32) * 2),
        0,
    ) {
        return 0;
    }
    units.len() as u32
}

const WAVE_MAPPER: u32 = u32::MAX;
const WAVE_FORMAT_QUERY: u32 = 0x0001;
const WHDR_DONE: u32 = 0x0000_0001;
const WHDR_PREPARED: u32 = 0x0000_0002;
const WHDR_INQUEUE: u32 = 0x0000_0010;
const CALLBACK_EVENT: u32 = 0x0005_0000;
const TIME_MS: u32 = 0x0001;
const TIME_SAMPLES: u32 = 0x0002;
const TIME_BYTES: u32 = 0x0004;
const WAVE_FORMAT_1M08: u32 = 0x0000_0001;
const WAVE_FORMAT_1S08: u32 = 0x0000_0002;
const WAVE_FORMAT_1M16: u32 = 0x0000_0004;
const WAVE_FORMAT_1S16: u32 = 0x0000_0008;
const WAVE_FORMAT_2M08: u32 = 0x0000_0010;
const WAVE_FORMAT_2S08: u32 = 0x0000_0020;
const WAVE_FORMAT_2M16: u32 = 0x0000_0040;
const WAVE_FORMAT_2S16: u32 = 0x0000_0080;
const WAVE_FORMAT_4M08: u32 = 0x0000_0100;
const WAVE_FORMAT_4S08: u32 = 0x0000_0200;
const WAVE_FORMAT_4M16: u32 = 0x0000_0400;
const WAVE_FORMAT_4S16: u32 = 0x0000_0800;
const WAVECAPS_VOLUME: u32 = 0x0004;
const WAVECAPS_LRVOLUME: u32 = 0x0008;
const WAVECAPS_SAMPLEACCURATE: u32 = 0x0020;

fn wave_out_open_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> MmResult {
    let handle_ptr = raw_arg(args, 0);
    let device_id = raw_arg(args, 1);
    let format_ptr = raw_arg(args, 2);
    let callback = raw_arg(args, 3);
    let open_flags = raw_arg(args, 5);
    if device_id != 0 && device_id != WAVE_MAPPER {
        return MMSYSERR_BADDEVICEID;
    }
    let Some(format) = read_wave_format(kernel, memory, thread_id, format_ptr) else {
        return WAVERR_BADFORMAT;
    };
    if open_flags & WAVE_FORMAT_QUERY != 0 {
        return if format.format_tag == 1 && format.block_align != 0 {
            MMSYSERR_NOERROR
        } else {
            WAVERR_BADFORMAT
        };
    }
    let handle = match kernel.wave_out_open(format) {
        Ok(handle) => handle,
        Err(status) => return status,
    };
    if open_flags & CALLBACK_EVENT != 0 {
        kernel
            .audio
            .set_wave_out_callback(handle, Some(WaveOutCallback::Event(callback)));
    }
    if !write_guest_u32(kernel, memory, thread_id, handle_ptr, handle) {
        return MMSYSERR_INVALHANDLE;
    }
    MMSYSERR_NOERROR
}

fn wave_out_prepare_header_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    handle: u32,
    header_ptr: u32,
) -> MmResult {
    if kernel.audio.output(handle).is_none() {
        return MMSYSERR_INVALHANDLE;
    }
    let Some(flags) = read_guest_u32(kernel, memory, 0, header_ptr.wrapping_add(16)) else {
        return MMSYSERR_INVALHANDLE;
    };
    if !write_guest_u32(
        kernel,
        memory,
        0,
        header_ptr.wrapping_add(16),
        (flags | WHDR_PREPARED) & !WHDR_DONE,
    ) {
        return MMSYSERR_INVALHANDLE;
    }
    MMSYSERR_NOERROR
}

fn wave_out_unprepare_header_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    handle: u32,
    header_ptr: u32,
) -> MmResult {
    if kernel.audio.output(handle).is_none() {
        return MMSYSERR_INVALHANDLE;
    }
    let Some(flags) = read_guest_u32(kernel, memory, 0, header_ptr.wrapping_add(16)) else {
        return MMSYSERR_INVALHANDLE;
    };
    if !write_guest_u32(
        kernel,
        memory,
        0,
        header_ptr.wrapping_add(16),
        flags & !WHDR_PREPARED,
    ) {
        return MMSYSERR_INVALHANDLE;
    }
    MMSYSERR_NOERROR
}

fn wave_out_write_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    handle: u32,
    header_ptr: u32,
) -> MmResult {
    let Some(data_ptr) = read_guest_u32(kernel, memory, 0, header_ptr) else {
        return MMSYSERR_INVALHANDLE;
    };
    let Some(len) = read_guest_u32(kernel, memory, 0, header_ptr.wrapping_add(4)) else {
        return MMSYSERR_INVALHANDLE;
    };
    if kernel.audio.output(handle).is_none() {
        return MMSYSERR_INVALHANDLE;
    }
    let buffer = WaveBuffer {
        guest_ptr: data_ptr,
        len,
    };
    let payload = if kernel.audio.has_sinks() && len > 0 {
        let Some(payload) = read_guest_bytes(kernel, memory, thread_id, data_ptr, len) else {
            return MMSYSERR_INVALHANDLE;
        };
        Some(payload)
    } else {
        None
    };
    let result = if let Some(payload) = payload.as_deref() {
        kernel.audio.wave_out_write_pcm(handle, buffer, payload)
    } else {
        kernel.audio.wave_out_write(handle, buffer)
    };
    if result != MMSYSERR_NOERROR {
        return result;
    }
    if let Some(flags) = read_guest_u32(kernel, memory, 0, header_ptr.wrapping_add(16)) {
        let _ = write_guest_u32(
            kernel,
            memory,
            0,
            header_ptr.wrapping_add(16),
            flags | WHDR_INQUEUE | WHDR_DONE,
        );
    }
    if let Some(WaveOutCallback::Event(event_handle)) = kernel
        .audio
        .output(handle)
        .and_then(|output| output.callback)
    {
        let _ = kernel.set_event(event_handle);
    }
    MMSYSERR_NOERROR
}

#[derive(Debug, Clone, Copy)]
enum WaveOutU32Kind {
    Volume,
    Pitch,
    PlaybackRate,
}

fn wave_out_get_u32_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    handle: u32,
    value_ptr: u32,
    kind: WaveOutU32Kind,
) -> MmResult {
    let value = match kind {
        WaveOutU32Kind::Volume => kernel.audio.get_volume(handle),
        WaveOutU32Kind::Pitch => kernel.audio.get_pitch(handle),
        WaveOutU32Kind::PlaybackRate => kernel.audio.get_playback_rate(handle),
    };
    let Ok(value) = value else {
        return MMSYSERR_INVALHANDLE;
    };
    if write_guest_u32(kernel, memory, 0, value_ptr, value) {
        MMSYSERR_NOERROR
    } else {
        MMSYSERR_INVALHANDLE
    }
}

fn wave_out_get_position_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    handle: u32,
    mmtime_ptr: u32,
) -> MmResult {
    let Ok(bytes) = kernel.audio.get_position_bytes(handle) else {
        return MMSYSERR_INVALHANDLE;
    };
    let Some(time_type) = read_guest_u32(kernel, memory, 0, mmtime_ptr) else {
        return MMSYSERR_INVALHANDLE;
    };
    let value = match time_type {
        TIME_SAMPLES => {
            let Some(output) = kernel.audio.output(handle) else {
                return MMSYSERR_INVALHANDLE;
            };
            if output.format.block_align == 0 {
                0
            } else {
                bytes / u32::from(output.format.block_align)
            }
        }
        TIME_MS => {
            let Some(output) = kernel.audio.output(handle) else {
                return MMSYSERR_INVALHANDLE;
            };
            if output.format.avg_bytes_per_sec == 0 {
                0
            } else {
                bytes.saturating_mul(1000) / output.format.avg_bytes_per_sec
            }
        }
        _ => bytes,
    };
    if !write_guest_u32(kernel, memory, 0, mmtime_ptr, time_type.max(TIME_BYTES)) {
        return MMSYSERR_INVALHANDLE;
    }
    if write_guest_u32(kernel, memory, 0, mmtime_ptr.wrapping_add(4), value) {
        MMSYSERR_NOERROR
    } else {
        MMSYSERR_INVALHANDLE
    }
}

fn wave_out_get_id_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    handle: u32,
    id_ptr: u32,
) -> MmResult {
    if kernel.audio.output(handle).is_none() {
        return MMSYSERR_INVALHANDLE;
    }
    if write_guest_u32(kernel, memory, 0, id_ptr, 0) {
        MMSYSERR_NOERROR
    } else {
        MMSYSERR_INVALHANDLE
    }
}

fn wave_out_get_dev_caps_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    device_id: u32,
    caps_ptr: u32,
    caps_size: u32,
) -> MmResult {
    if device_id != 0 && device_id != WAVE_MAPPER {
        return MMSYSERR_BADDEVICEID;
    }
    let writes = [
        (0, 1),
        (2, 1),
        (4, 0x0004_0000),
        (72, wave_formats_mask()),
        (76, 2),
        (78, 0),
        (
            80,
            WAVECAPS_VOLUME | WAVECAPS_LRVOLUME | WAVECAPS_SAMPLEACCURATE,
        ),
    ];
    for (offset, value) in writes {
        if offset >= caps_size {
            continue;
        }
        let addr = caps_ptr.wrapping_add(offset);
        let ok = if offset == 0 || offset == 2 || offset == 76 || offset == 78 {
            write_guest_u16(kernel, memory, 0, addr, value as u16)
        } else {
            write_guest_u32(kernel, memory, 0, addr, value)
        };
        if !ok {
            return MMSYSERR_INVALHANDLE;
        }
    }
    write_guest_wide_fixed(
        kernel,
        memory,
        0,
        caps_ptr.wrapping_add(8),
        "Virtual WaveOut",
        32,
    );
    MMSYSERR_NOERROR
}

fn wave_out_get_error_text_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    error: u32,
    buffer: u32,
    capacity: u32,
) -> MmResult {
    let text = match error {
        MMSYSERR_NOERROR => "No error",
        MMSYSERR_BADDEVICEID => "Bad device id",
        MMSYSERR_INVALHANDLE => "Invalid handle",
        WAVERR_BADFORMAT => "Bad wave format",
        _ => "Multimedia error",
    };
    if write_wide_result(kernel, memory, thread_id, buffer, capacity as usize, text) > 0 {
        MMSYSERR_NOERROR
    } else {
        MMSYSERR_INVALHANDLE
    }
}

fn read_wave_format<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    addr: u32,
) -> Option<WaveFormat> {
    Some(WaveFormat {
        format_tag: read_guest_u16(kernel, memory, thread_id, addr)?,
        channels: read_guest_u16(kernel, memory, thread_id, addr.wrapping_add(2))?,
        samples_per_sec: read_guest_u32(kernel, memory, thread_id, addr.wrapping_add(4))?,
        avg_bytes_per_sec: read_guest_u32(kernel, memory, thread_id, addr.wrapping_add(8))?,
        block_align: read_guest_u16(kernel, memory, thread_id, addr.wrapping_add(12))?,
        bits_per_sample: read_guest_u16(kernel, memory, thread_id, addr.wrapping_add(14))?,
    })
}

fn wave_formats_mask() -> u32 {
    WAVE_FORMAT_1M08
        | WAVE_FORMAT_1S08
        | WAVE_FORMAT_1M16
        | WAVE_FORMAT_1S16
        | WAVE_FORMAT_2M08
        | WAVE_FORMAT_2S08
        | WAVE_FORMAT_2M16
        | WAVE_FORMAT_2S16
        | WAVE_FORMAT_4M08
        | WAVE_FORMAT_4S08
        | WAVE_FORMAT_4M16
        | WAVE_FORMAT_4S16
}

fn write_guest_wide_fixed<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
    text: &str,
    capacity_chars: usize,
) -> bool {
    let mut units = text.encode_utf16().take(capacity_chars.saturating_sub(1));
    for (index, unit) in units.by_ref().enumerate() {
        if !write_guest_u16(
            kernel,
            memory,
            thread_id,
            addr.wrapping_add((index as u32) * 2),
            unit,
        ) {
            return false;
        }
    }
    let len = text
        .encode_utf16()
        .take(capacity_chars.saturating_sub(1))
        .count();
    write_guest_u16(
        kernel,
        memory,
        thread_id,
        addr.wrapping_add((len as u32) * 2),
        0,
    )
}

fn sh_get_special_folder_path_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let path_ptr = raw_arg(args, 1);
    if path_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    let Some(path) = special_folder_path(kernel, raw_arg(args, 2)) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    };
    if write_guest_wide_fixed(
        kernel,
        memory,
        thread_id,
        path_ptr,
        &path,
        WIN32_FIND_DATAW_FILE_NAME_CHARS,
    ) {
        kernel.threads.set_last_error(thread_id, 0);
        true
    } else {
        false
    }
}

fn special_folder_path(kernel: &CeKernel, folder: u32) -> Option<String> {
    let (value_name, fallback) = match folder {
        0x0000 | 0x0010 => ("Desktop", r"\Windows\Desktop"),
        0x0002 => ("Programs", r"\Windows\Programs"),
        0x0005 => ("Personal", r"\My Documents"),
        0x0006 => ("Favorites", r"\Windows\Favorites"),
        0x0007 => ("Startup", r"\Windows\Startup"),
        0x0008 => ("Recent", r"\Windows\Recent"),
        0x000b => ("Start Menu", r"\Windows\Start Menu"),
        0x0014 => ("Fonts", r"\Windows\Fonts"),
        0x001a => ("AppData", r"\Application Data"),
        0x0024 => ("Windows", r"\Windows"),
        0x0026 => ("Program Files", r"\Program Files"),
        0x0028 => ("Profile", r"\Windows\Profiles"),
        _ => return None,
    };
    Some(
        kernel
            .registry
            .query_value(SHELL_FOLDERS_REGISTRY_PATH, value_name)
            .ok()
            .and_then(|value| value.as_str())
            .unwrap_or(fallback)
            .to_owned(),
    )
}

fn find_resource<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    module: u32,
    name: u32,
    kind: u32,
) -> u32 {
    let Some(name) = resource_id_from_guest_arg(memory, name) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_RESOURCE_NAME_NOT_FOUND);
        return 0;
    };
    let Some(kind) = resource_id_from_guest_arg(memory, kind) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_RESOURCE_NAME_NOT_FOUND);
        return 0;
    };
    let module = normalized_module(kernel, module);
    let Some(handle) = kernel.resources.find_resource(module, name, kind) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_RESOURCE_NAME_NOT_FOUND);
        return 0;
    };
    handle
}

fn resource_id_from_guest_arg<M: CoredllGuestMemory>(memory: &M, value: u32) -> Option<ResourceId> {
    if value <= u16::MAX as u32 {
        Some(ResourceId::Integer(value as u16))
    } else {
        read_guest_wide_arg(memory, value).map(ResourceId::Name)
    }
}

fn load_resource(kernel: &mut CeKernel, thread_id: u32, resource_handle: u32) -> u32 {
    let Some(data_ptr) = kernel.resources.load_resource(resource_handle) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    data_ptr
}

fn sizeof_resource(kernel: &mut CeKernel, thread_id: u32, resource_handle: u32) -> u32 {
    let Some(size) = kernel.resources.sizeof_resource(resource_handle) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    size
}

fn load_string_w<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    module: u32,
    id: u32,
    buffer: u32,
    capacity_chars: i32,
) -> u32 {
    if buffer != 0 && capacity_chars <= 1 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let module = normalized_module(kernel, module);
    let Some(resource) = kernel.resources.load_string(module, id) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_RESOURCE_NAME_NOT_FOUND);
        return 0;
    };
    if buffer == 0 {
        return resource.data_ptr.unwrap_or(0);
    }

    let copy_limit = capacity_chars.saturating_sub(1) as usize;
    let utf16: Vec<u16> = resource.text.encode_utf16().take(copy_limit).collect();
    for (index, unit) in utf16.iter().copied().enumerate() {
        if !write_guest_u16(
            kernel,
            memory,
            thread_id,
            buffer.wrapping_add((index as u32) * 2),
            unit,
        ) {
            return 0;
        }
    }
    if !write_guest_u16(
        kernel,
        memory,
        thread_id,
        buffer.wrapping_add((utf16.len() as u32) * 2),
        0,
    ) {
        return 0;
    }
    utf16.len() as u32
}

fn load_menu_w_raw(kernel: &mut CeKernel, thread_id: u32, module: u32, name: u32) -> u32 {
    let module = normalized_module(kernel, module);
    let name = ResourceId::from_guest_arg(name);
    let Some(resource_handle) =
        kernel
            .resources
            .find_resource(module, name.clone(), ResourceId::Integer(RT_MENU as u16))
    else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_RESOURCE_NAME_NOT_FOUND);
        return 0;
    };
    kernel.threads.set_last_error(thread_id, 0);
    kernel
        .resources
        .create_menu(module, name, Some(resource_handle))
}

fn create_menu_raw(kernel: &mut CeKernel, thread_id: u32) -> u32 {
    let handle = kernel
        .resources
        .create_menu(0, ResourceId::Integer(0), None);
    kernel.threads.set_last_error(thread_id, 0);
    handle
}

fn create_popup_menu_raw(kernel: &mut CeKernel, thread_id: u32) -> u32 {
    let handle = kernel.resources.create_popup_menu();
    kernel.threads.set_last_error(thread_id, 0);
    handle
}

fn set_menu_raw(kernel: &mut CeKernel, thread_id: u32, hwnd: u32, menu: u32) -> bool {
    if !kernel.gwe.set_menu(hwnd, menu) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn get_menu_raw(kernel: &mut CeKernel, thread_id: u32, hwnd: u32) -> u32 {
    let Some(menu) = kernel.gwe.get_menu(hwnd) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    };
    kernel.threads.set_last_error(thread_id, 0);
    menu
}

fn set_associated_menu_raw(kernel: &mut CeKernel, thread_id: u32, hwnd: u32, menu: u32) -> u32 {
    if !kernel.gwe.set_menu(hwnd, menu) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    0
}

fn draw_menu_bar_raw(kernel: &mut CeKernel, thread_id: u32, hwnd: u32) -> bool {
    if !kernel.gwe.draw_menu_bar(hwnd) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn insert_menu_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let menu = raw_arg(args, 0);
    let position = raw_arg(args, 1);
    let flags = raw_arg(args, 2);
    let id_or_submenu = raw_arg(args, 3);
    let item = menu_item_from_insert_args(memory, flags, id_or_submenu, raw_arg(args, 4));
    if !kernel
        .resources
        .insert_menu_item(menu, position, flags, item)
    {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn append_menu_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let menu = raw_arg(args, 0);
    let flags = raw_arg(args, 1);
    let id_or_submenu = raw_arg(args, 2);
    let item = menu_item_from_insert_args(memory, flags, id_or_submenu, raw_arg(args, 3));
    if !kernel.resources.append_menu_item(menu, item) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn track_popup_menu_ex_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let menu = raw_arg(args, 0);
    if kernel.resources.menu(menu).is_none() {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    }
    let hwnd = raw_arg(args, 4);
    if hwnd != 0 && !kernel.gwe.is_window(hwnd) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    }
    let lptpm = raw_arg(args, 5);
    let exclude_rect = if lptpm == 0 {
        None
    } else {
        let Some(cb_size) = read_guest_u32(kernel, memory, thread_id, lptpm) else {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            return 0;
        };
        if cb_size < TPMPARAMS_SIZE {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            return 0;
        }
        let Some(rect) = read_guest_rect(kernel, memory, thread_id, lptpm.wrapping_add(4)) else {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            return 0;
        };
        Some([rect.left, rect.top, rect.right, rect.bottom])
    };
    let flags = raw_arg(args, 1);
    let tracking = PopupMenuTracking {
        menu,
        flags,
        x: raw_i32_arg(args, 2),
        y: raw_i32_arg(args, 3),
        hwnd,
        exclude_rect,
    };
    if !kernel.resources.track_popup_menu(tracking) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    if flags & TPM_RETURNCMD != 0 { 0 } else { 1 }
}

fn get_sub_menu_raw(kernel: &mut CeKernel, thread_id: u32, menu: u32, position: u32) -> u32 {
    let submenu = kernel.resources.get_sub_menu(menu, position).unwrap_or(0);
    kernel.threads.set_last_error(
        thread_id,
        if submenu == 0 {
            ERROR_INVALID_PARAMETER
        } else {
            0
        },
    );
    submenu
}

fn get_menu_item_info_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let menu = raw_arg(args, 0);
    let item_or_pos = raw_arg(args, 1);
    let by_position = raw_arg(args, 2) != 0;
    let info_ptr = raw_arg(args, 3);
    let Some(cb_size) = read_guest_u32(kernel, memory, thread_id, info_ptr) else {
        return false;
    };
    if cb_size < MENUITEMINFOW_SIZE {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    let Some(mask) = read_guest_u32(kernel, memory, thread_id, info_ptr + 4) else {
        return false;
    };
    let Some(item) = kernel
        .resources
        .get_menu_item(menu, item_or_pos, by_position)
        .cloned()
    else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    };
    if !write_menu_item_info(kernel, memory, thread_id, info_ptr, mask, &item) {
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn set_menu_item_info_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let menu = raw_arg(args, 0);
    let item_or_pos = raw_arg(args, 1);
    let by_position = raw_arg(args, 2) != 0;
    let info_ptr = raw_arg(args, 3);
    let Some((mask, patch)) = read_menu_item_info(kernel, memory, thread_id, info_ptr) else {
        return false;
    };
    let Some(mut item) = kernel
        .resources
        .get_menu_item(menu, item_or_pos, by_position)
        .cloned()
    else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    };
    apply_menu_item_info_mask(&mut item, mask, patch);
    if !kernel
        .resources
        .set_menu_item(menu, item_or_pos, by_position, item)
    {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn menu_item_from_insert_args<M: CoredllGuestMemory>(
    memory: &M,
    flags: u32,
    id_or_submenu: u32,
    text_ptr: u32,
) -> MenuItem {
    let text = if flags & crate::ce::resource::MF_SEPARATOR != 0 || text_ptr == 0 {
        None
    } else {
        read_guest_wide_arg(memory, text_ptr)
    };
    MenuItem::from_insert_flags(flags, id_or_submenu, text)
}

fn read_menu_item_info<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    info_ptr: u32,
) -> Option<(u32, MenuItem)> {
    let cb_size = read_guest_u32(kernel, memory, thread_id, info_ptr)?;
    if cb_size < MENUITEMINFOW_SIZE {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return None;
    }
    let mask = read_guest_u32(kernel, memory, thread_id, info_ptr + 4)?;
    let text_ptr = read_guest_u32(kernel, memory, thread_id, info_ptr + 36).unwrap_or(0);
    let text = if menu_info_wants_text(mask) && text_ptr != 0 {
        read_guest_wide_arg(memory, text_ptr)
    } else {
        None
    };
    Some((
        mask,
        MenuItem {
            id: read_guest_u32(kernel, memory, thread_id, info_ptr + 16).unwrap_or(0),
            item_type: read_guest_u32(kernel, memory, thread_id, info_ptr + 8).unwrap_or(0),
            state: read_guest_u32(kernel, memory, thread_id, info_ptr + 12).unwrap_or(0),
            submenu: read_guest_u32(kernel, memory, thread_id, info_ptr + 20).unwrap_or(0),
            checked_bitmap: read_guest_u32(kernel, memory, thread_id, info_ptr + 24).unwrap_or(0),
            unchecked_bitmap: read_guest_u32(kernel, memory, thread_id, info_ptr + 28).unwrap_or(0),
            data: read_guest_u32(kernel, memory, thread_id, info_ptr + 32).unwrap_or(0),
            text,
        },
    ))
}

fn apply_menu_item_info_mask(item: &mut MenuItem, mask: u32, patch: MenuItem) {
    if mask & MIIM_TYPE != 0 {
        item.item_type = patch.item_type;
    }
    if mask & MIIM_STATE != 0 {
        item.state = patch.state;
    }
    if mask & MIIM_ID != 0 {
        item.id = patch.id;
    }
    if mask & MIIM_SUBMENU != 0 {
        item.submenu = patch.submenu;
        if patch.submenu != 0 && item.id == 0 {
            item.id = u32::MAX;
        }
    }
    if mask & MIIM_CHECKMARKS != 0 {
        item.checked_bitmap = patch.checked_bitmap;
        item.unchecked_bitmap = patch.unchecked_bitmap;
    }
    if mask & MIIM_DATA != 0 {
        item.data = patch.data;
    }
    if menu_info_wants_text(mask) {
        item.text = patch.text;
    }
}

fn write_menu_item_info<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    info_ptr: u32,
    mask: u32,
    item: &MenuItem,
) -> bool {
    if mask & MIIM_TYPE != 0
        && !write_guest_u32(kernel, memory, thread_id, info_ptr + 8, item.item_type)
    {
        return false;
    }
    if mask & MIIM_STATE != 0
        && !write_guest_u32(kernel, memory, thread_id, info_ptr + 12, item.state)
    {
        return false;
    }
    if mask & MIIM_ID != 0 && !write_guest_u32(kernel, memory, thread_id, info_ptr + 16, item.id) {
        return false;
    }
    if mask & MIIM_SUBMENU != 0
        && !write_guest_u32(kernel, memory, thread_id, info_ptr + 20, item.submenu)
    {
        return false;
    }
    if mask & MIIM_CHECKMARKS != 0 {
        if !write_guest_u32(
            kernel,
            memory,
            thread_id,
            info_ptr + 24,
            item.checked_bitmap,
        ) || !write_guest_u32(
            kernel,
            memory,
            thread_id,
            info_ptr + 28,
            item.unchecked_bitmap,
        ) {
            return false;
        }
    }
    if mask & MIIM_DATA != 0
        && !write_guest_u32(kernel, memory, thread_id, info_ptr + 32, item.data)
    {
        return false;
    }
    if menu_info_wants_text(mask) {
        let text = item.text.as_deref().unwrap_or("");
        let full_len = text.encode_utf16().count() as u32;
        let buffer = read_guest_u32(kernel, memory, thread_id, info_ptr + 36).unwrap_or(0);
        let capacity = read_guest_u32(kernel, memory, thread_id, info_ptr + 40).unwrap_or(0);
        if buffer != 0
            && capacity != 0
            && write_wide_result(kernel, memory, thread_id, buffer, capacity as usize, text) == 0
            && !text.is_empty()
        {
            return false;
        }
        if !write_guest_u32(kernel, memory, thread_id, info_ptr + 40, full_len) {
            return false;
        }
    }
    true
}

fn menu_info_wants_text(mask: u32) -> bool {
    mask & (MIIM_TYPE | MIIM_STRING) != 0
}

fn check_menu_item_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    menu: u32,
    item: u32,
    flags: u32,
) -> u32 {
    let Some(previous) = kernel.resources.check_menu_item(menu, item, flags) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return u32::MAX;
    };
    kernel.threads.set_last_error(thread_id, 0);
    previous
}

fn enable_menu_item_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    menu: u32,
    item: u32,
    flags: u32,
) -> u32 {
    let Some(previous) = kernel.resources.enable_menu_item(menu, item, flags) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return u32::MAX;
    };
    kernel.threads.set_last_error(thread_id, 0);
    previous
}

fn check_menu_radio_item_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    menu: u32,
    first: u32,
    last: u32,
    checked: u32,
) -> bool {
    if !kernel
        .resources
        .check_menu_radio_item(menu, first, last, checked)
    {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn remove_menu_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    menu: u32,
    item: u32,
    flags: u32,
) -> bool {
    if !kernel.resources.remove_menu_item(menu, item, flags) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn destroy_menu_raw(kernel: &mut CeKernel, thread_id: u32, menu: u32) -> bool {
    if !kernel.resources.delete_menu(menu) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn load_accelerators_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    module: u32,
    name: u32,
) -> u32 {
    let module = normalized_module(kernel, module);
    let name = ResourceId::from_guest_arg(name);
    let Some(resource_handle) = kernel.resources.find_resource(
        module,
        name.clone(),
        ResourceId::Integer(RT_ACCELERATOR as u16),
    ) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_RESOURCE_NAME_NOT_FOUND);
        return 0;
    };
    let data = kernel
        .resources
        .load_resource(resource_handle)
        .zip(kernel.resources.sizeof_resource(resource_handle))
        .and_then(|(data_ptr, size)| read_guest_bytes(kernel, memory, thread_id, data_ptr, size))
        .unwrap_or_default();
    let entries = parse_accelerator_entries(&data);
    kernel.threads.set_last_error(thread_id, 0);
    kernel
        .resources
        .create_accelerator(module, name, Some(resource_handle), entries)
}

fn destroy_accelerator_table_raw(kernel: &mut CeKernel, thread_id: u32, accel: u32) -> bool {
    if !kernel.resources.delete_accelerator(accel) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn translate_accelerator_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    hwnd: u32,
    accel: u32,
    msg_ptr: u32,
) -> u32 {
    let Some(accel_table) = kernel.resources.accelerator(accel).cloned() else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    let Some(message) = read_guest_message(kernel, memory, thread_id, msg_ptr) else {
        return 0;
    };
    if message.msg != crate::ce::gwe::WM_KEYDOWN {
        kernel.threads.set_last_error(thread_id, 0);
        return 0;
    }
    let key = message.wparam as u16;
    let Some(entry) = accel_table.entries.iter().find(|entry| entry.key == key) else {
        kernel.threads.set_last_error(thread_id, 0);
        return 0;
    };
    let target = if hwnd != 0 { hwnd } else { message.hwnd };
    if target == 0 {
        kernel.threads.set_last_error(thread_id, 0);
        return 0;
    }
    kernel.post_message_w_for_thread(
        thread_id,
        target,
        crate::ce::gwe::WM_COMMAND,
        u32::from(entry.command),
        0,
    );
    kernel.threads.set_last_error(thread_id, 0);
    1
}

fn parse_accelerator_entries(bytes: &[u8]) -> Vec<AcceleratorEntry> {
    let mut entries = Vec::new();
    for chunk in bytes.chunks_exact(8) {
        let flags_word = u16::from_le_bytes([chunk[0], chunk[1]]);
        let key = u16::from_le_bytes([chunk[2], chunk[3]]);
        let command = u16::from_le_bytes([chunk[4], chunk[5]]);
        let flags = flags_word as u8;
        entries.push(AcceleratorEntry {
            flags,
            key,
            command,
        });
        if flags_word & 0x0080 != 0 {
            break;
        }
    }
    entries
}

fn normalized_module(kernel: &CeKernel, module: u32) -> u32 {
    if module == 0 {
        kernel.process_module_base()
    } else {
        module
    }
}

fn load_bitmap_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    module: u32,
    name: u32,
) -> u32 {
    load_bitmap_resource(kernel, memory, thread_id, module, name)
}

fn load_image_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let module = raw_arg(args, 0);
    let name = raw_arg(args, 1);
    let image_type = raw_arg(args, 2);
    let flags = raw_arg(args, 5);

    if image_type == IMAGE_CURSOR {
        return load_cursor_w_raw(kernel, thread_id, module, name);
    }
    if image_type == IMAGE_ICON {
        return load_icon_w_raw(kernel, thread_id, module, name);
    }
    if image_type != IMAGE_BITMAP {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }

    if flags & LR_LOADFROMFILE != 0 {
        return load_bitmap_file(kernel, memory, thread_id, name);
    }

    load_bitmap_resource(kernel, memory, thread_id, module, name)
}

fn load_cursor_w_raw(kernel: &mut CeKernel, thread_id: u32, module: u32, name: u32) -> u32 {
    load_stock_or_resource_image(
        kernel,
        thread_id,
        module,
        name,
        RT_GROUP_CURSOR as u16,
        0x000b_0000,
    )
}

fn load_icon_w_raw(kernel: &mut CeKernel, thread_id: u32, module: u32, name: u32) -> u32 {
    load_stock_or_resource_image(
        kernel,
        thread_id,
        module,
        name,
        RT_GROUP_ICON as u16,
        0x000b_8000,
    )
}

fn destroy_icon_raw(kernel: &mut CeKernel, thread_id: u32, icon: u32) -> bool {
    if icon == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn load_stock_or_resource_image(
    kernel: &mut CeKernel,
    thread_id: u32,
    module: u32,
    name: u32,
    resource_kind: u16,
    stock_base: u32,
) -> u32 {
    if module == 0 && name <= u16::MAX as u32 {
        kernel.threads.set_last_error(thread_id, 0);
        return stock_base | name;
    }

    let module = normalized_module(kernel, module);
    let name = ResourceId::from_guest_arg(name);
    let Some(resource_handle) =
        kernel
            .resources
            .find_resource(module, name, ResourceId::Integer(resource_kind))
    else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_RESOURCE_NAME_NOT_FOUND);
        return 0;
    };
    kernel.threads.set_last_error(thread_id, 0);
    resource_handle
}

fn load_bitmap_resource<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    module: u32,
    name: u32,
) -> u32 {
    let module = if module == 0 {
        kernel.process_module_base()
    } else {
        module
    };
    let Some(resource_handle) = kernel.resources.find_resource(
        module,
        ResourceId::from_guest_arg(name),
        ResourceId::Integer(RT_BITMAP as u16),
    ) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_RESOURCE_NAME_NOT_FOUND);
        return 0;
    };
    let Some(data_ptr) = kernel.resources.load_resource(resource_handle) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    let Some(size) = kernel.resources.sizeof_resource(resource_handle) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    let Some(bytes) = read_guest_bytes(kernel, memory, thread_id, data_ptr, size) else {
        return 0;
    };
    create_bitmap_from_dib_bytes(kernel, thread_id, &bytes, data_ptr)
}

fn load_bitmap_file<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    name_ptr: u32,
) -> u32 {
    let Some(path) = read_guest_wide_arg(memory, name_ptr) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let Ok(bytes) = kernel.read_guest_file(&path) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_FILE_NOT_FOUND);
        return 0;
    };
    create_bitmap_from_file_bytes(kernel, thread_id, &bytes)
}

fn create_bitmap_from_file_bytes(kernel: &mut CeKernel, thread_id: u32, bytes: &[u8]) -> u32 {
    if bytes.len() < 14 || &bytes[0..2] != b"BM" {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    if bytes.len() <= 14 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    create_bitmap_from_dib_bytes(kernel, thread_id, &bytes[14..], 0)
}

fn create_bitmap_from_dib_bytes(
    kernel: &mut CeKernel,
    thread_id: u32,
    bytes: &[u8],
    bits_base: u32,
) -> u32 {
    let Some(header) = parse_dib_header(bytes) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    kernel.threads.set_last_error(thread_id, 0);
    kernel.resources.create_bitmap(
        header.width,
        header.height,
        header.planes,
        header.bits_pixel,
        bits_base,
    )
}

fn create_bitmap_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    width: i32,
    height: i32,
    planes: u16,
    bits_pixel: u16,
    bits_ptr: u32,
) -> u32 {
    if width <= 0 || height == 0 || planes == 0 || bits_pixel == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    kernel
        .resources
        .create_bitmap(width, height, planes, bits_pixel, bits_ptr)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DibHeader {
    width: i32,
    height: i32,
    planes: u16,
    bits_pixel: u16,
}

fn parse_dib_header(bytes: &[u8]) -> Option<DibHeader> {
    let header_size = read_le_u32(bytes, 0)?;
    match header_size {
        12 => Some(DibHeader {
            width: read_le_u16(bytes, 4)? as i32,
            height: read_le_u16(bytes, 6)? as i32,
            planes: read_le_u16(bytes, 8)?,
            bits_pixel: read_le_u16(bytes, 10)?,
        }),
        40..=124 => Some(DibHeader {
            width: read_le_i32(bytes, 4)?,
            height: read_le_i32(bytes, 8)?,
            planes: read_le_u16(bytes, 12)?,
            bits_pixel: read_le_u16(bytes, 14)?,
        }),
        _ => None,
    }
}

fn bitmap_byte_count(width: i32, height: i32, bits_pixel: u16) -> Option<u32> {
    if width == 0 || height == 0 || bits_pixel == 0 {
        return None;
    }
    let width = u64::from(width.unsigned_abs());
    let height = u64::from(height.unsigned_abs());
    let bits_per_row = width.checked_mul(u64::from(bits_pixel))?;
    let row_bytes = bits_per_row
        .checked_add(31)?
        .checked_div(32)?
        .checked_mul(4)?;
    let bytes = row_bytes.checked_mul(height)?;
    (bytes <= u64::from(u32::MAX)).then_some(bytes as u32)
}

fn dib_rgb_masks<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    info_ptr: u32,
    header_size: u32,
    compression: u32,
    bits_pixel: u16,
) -> Option<Option<[u32; 3]>> {
    match compression {
        BI_RGB => Some(None),
        BI_BITFIELDS if matches!(bits_pixel, 16 | 32) => {
            let mask_bytes = if header_size >= 52 {
                read_guest_bytes(kernel, memory, thread_id, info_ptr.wrapping_add(40), 12)?
            } else {
                read_guest_bytes(
                    kernel,
                    memory,
                    thread_id,
                    info_ptr.wrapping_add(header_size),
                    12,
                )?
            };
            Some(Some([
                read_le_u32(&mask_bytes, 0)?,
                read_le_u32(&mask_bytes, 4)?,
                read_le_u32(&mask_bytes, 8)?,
            ]))
        }
        _ => None,
    }
}

fn read_dib_color_table<M: CoredllGuestMemory>(
    memory: &M,
    info_ptr: u32,
    header_size: u32,
    compression: u32,
    bits_pixel: u16,
) -> Vec<[u8; 4]> {
    if !matches!(bits_pixel, 1 | 4 | 8) {
        return Vec::new();
    }
    let max_entries = 1usize << bits_pixel;
    let used_entries = if header_size >= 40 {
        memory
            .read_u32(info_ptr.wrapping_add(32))
            .ok()
            .and_then(|count| (count != 0).then_some(count as usize))
            .unwrap_or(max_entries)
            .min(max_entries)
    } else {
        max_entries
    };
    if used_entries == 0 {
        return Vec::new();
    }
    let table_ptr = if header_size == 12 {
        info_ptr.wrapping_add(12)
    } else if compression == BI_BITFIELDS && header_size == 40 {
        info_ptr.wrapping_add(52)
    } else {
        info_ptr.wrapping_add(header_size)
    };
    let bytes_per_entry = if header_size == 12 { 3 } else { 4 };
    let mut table = Vec::with_capacity(used_entries);
    for index in 0..used_entries {
        let entry_ptr = table_ptr.wrapping_add((index * bytes_per_entry) as u32);
        let blue = match memory.read_u8(entry_ptr) {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
        let green = match memory.read_u8(entry_ptr.wrapping_add(1)) {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
        let red = match memory.read_u8(entry_ptr.wrapping_add(2)) {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
        let reserved = if bytes_per_entry == 4 {
            match memory.read_u8(entry_ptr.wrapping_add(3)) {
                Ok(value) => value,
                Err(_) => return Vec::new(),
            }
        } else {
            0
        };
        table.push([blue, green, red, reserved]);
    }
    table
}

fn get_object_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    handle: u32,
    byte_count: i32,
    out_ptr: u32,
) -> u32 {
    const BITMAP_SIZE: u32 = 24;
    if out_ptr == 0 || byte_count < BITMAP_SIZE as i32 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INSUFFICIENT_BUFFER);
        return 0;
    }
    let Some(bitmap) = kernel.resources.bitmap(handle).cloned() else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    let ok = write_guest_i32(kernel, memory, thread_id, out_ptr, 0)
        && write_guest_i32(
            kernel,
            memory,
            thread_id,
            out_ptr.wrapping_add(4),
            bitmap.width,
        )
        && write_guest_i32(
            kernel,
            memory,
            thread_id,
            out_ptr.wrapping_add(8),
            bitmap.height,
        )
        && write_guest_i32(
            kernel,
            memory,
            thread_id,
            out_ptr.wrapping_add(12),
            bitmap.width_bytes,
        )
        && write_guest_u16(
            kernel,
            memory,
            thread_id,
            out_ptr.wrapping_add(16),
            bitmap.planes,
        )
        && write_guest_u16(
            kernel,
            memory,
            thread_id,
            out_ptr.wrapping_add(18),
            bitmap.bits_pixel,
        )
        && write_guest_u32(
            kernel,
            memory,
            thread_id,
            out_ptr.wrapping_add(20),
            bitmap.bits_ptr,
        );
    if ok {
        kernel.threads.set_last_error(thread_id, 0);
        BITMAP_SIZE
    } else {
        0
    }
}

fn delete_object_raw(kernel: &mut CeKernel, handle: u32) -> bool {
    if handle == 0 {
        return false;
    }
    let bitmap = kernel.resources.bitmap(handle).cloned();
    let removed = kernel.resources.delete_gdi_object(handle);
    if removed
        && let Some(bitmap) = bitmap
        && bitmap.bits_owned
        && bitmap.bits_ptr != 0
    {
        let _ = kernel
            .memory
            .heap_free(PROCESS_HEAP_HANDLE, 0, bitmap.bits_ptr);
    }
    true
}

fn read_le_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes([
        *bytes.get(offset)?,
        *bytes.get(offset + 1)?,
    ]))
}

fn read_le_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes([
        *bytes.get(offset)?,
        *bytes.get(offset + 1)?,
        *bytes.get(offset + 2)?,
        *bytes.get(offset + 3)?,
    ]))
}

fn read_le_i32(bytes: &[u8], offset: usize) -> Option<i32> {
    read_le_u32(bytes, offset).map(|value| value as i32)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowRectKind {
    Window,
    Client,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PointMapKind {
    ClientToScreen,
    ScreenToClient,
}

fn write_window_rect<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    hwnd: u32,
    rect_ptr: u32,
    kind: WindowRectKind,
) -> bool {
    let rect = match kind {
        WindowRectKind::Window => kernel.gwe.get_window_rect(hwnd),
        WindowRectKind::Client => kernel.gwe.get_client_rect(hwnd),
    };
    let Some(rect) = rect else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return false;
    };
    write_guest_rect(kernel, memory, thread_id, rect_ptr, rect)
}

fn get_window_raw(kernel: &mut CeKernel, thread_id: u32, hwnd: u32, cmd: u32) -> u32 {
    if !kernel.gwe.is_window(hwnd) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    }
    if cmd > crate::ce::gwe::GW_CHILD {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    kernel.gwe.get_window(hwnd, cmd).unwrap_or(0)
}

fn set_parent_raw(kernel: &mut CeKernel, thread_id: u32, hwnd: u32, parent: u32) -> u32 {
    let parent = (parent != 0).then_some(parent);
    if !kernel.gwe.is_window(hwnd) || parent.is_some_and(|parent| !kernel.gwe.is_window(parent)) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    }
    if parent.is_some_and(|parent| parent == hwnd || kernel.gwe.is_child(hwnd, parent)) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let previous = kernel.set_parent(hwnd, parent).flatten().unwrap_or(0);
    kernel.threads.set_last_error(thread_id, 0);
    previous
}

fn show_window_raw(kernel: &mut CeKernel, args: &[u32]) -> bool {
    const SW_HIDE: u32 = 0;
    const SW_SHOWNOACTIVATE: u32 = 4;
    const SW_SHOWMINNOACTIVE: u32 = 7;
    const SW_SHOWNA: u32 = 8;
    let hwnd = raw_arg(args, 0);
    let cmd = raw_arg(args, 1);
    let visible = cmd != SW_HIDE;
    let activate = visible && !matches!(cmd, SW_SHOWNOACTIVATE | SW_SHOWMINNOACTIVE | SW_SHOWNA);
    kernel.show_window_with_activation(hwnd, visible, activate)
}

fn set_foreground_window_raw(kernel: &mut CeKernel, thread_id: u32, hwnd: u32) -> bool {
    if hwnd == 0 || !kernel.gwe.is_window(hwnd) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return false;
    }
    let _ = kernel.activate_window(Some(hwnd));
    let _ = kernel.set_focus(Some(hwnd));
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn set_active_window_raw(kernel: &mut CeKernel, thread_id: u32, hwnd: u32) -> u32 {
    if hwnd != 0 && !kernel.gwe.is_window(hwnd) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    }
    let previous = kernel.gwe.get_active_window().unwrap_or(0);
    let _ = kernel.activate_window((hwnd != 0).then_some(hwnd));
    kernel.threads.set_last_error(thread_id, 0);
    previous
}

fn set_focus_raw(kernel: &mut CeKernel, thread_id: u32, hwnd: u32) -> Option<u32> {
    if hwnd != 0 && !kernel.gwe.is_window(hwnd) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return None;
    }
    let previous = kernel.set_focus((hwnd != 0).then_some(hwnd));
    kernel.threads.set_last_error(thread_id, 0);
    previous
}

fn map_single_point<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    hwnd: u32,
    point_ptr: u32,
    kind: PointMapKind,
) -> bool {
    let Some(point) = read_guest_point(kernel, memory, thread_id, point_ptr) else {
        return false;
    };
    let mapped = match kind {
        PointMapKind::ClientToScreen => kernel.gwe.client_to_screen(hwnd, point),
        PointMapKind::ScreenToClient => kernel.gwe.screen_to_client(hwnd, point),
    };
    let Some(mapped) = mapped else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return false;
    };
    write_guest_point(kernel, memory, thread_id, point_ptr, mapped)
}

fn window_from_point_raw(kernel: &mut CeKernel, thread_id: u32, x: u32, y: u32) -> u32 {
    let point = Point {
        x: x as i32,
        y: y as i32,
    };
    let hwnd = kernel
        .gwe
        .window_from_point_for_thread(thread_id, point)
        .unwrap_or(0);
    kernel.threads.set_last_error(thread_id, 0);
    hwnd
}

fn child_window_from_point_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    parent: u32,
    x: u32,
    y: u32,
) -> u32 {
    let point = Point {
        x: x as i32,
        y: y as i32,
    };
    let hwnd = kernel
        .gwe
        .child_window_from_point_for_thread(thread_id, parent, point)
        .unwrap_or(0);
    if hwnd == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
    } else {
        kernel.threads.set_last_error(thread_id, 0);
    }
    hwnd
}

const DIALOG_BASE_UNIT_X: i32 = 8;
const DIALOG_BASE_UNIT_Y: i32 = 16;

fn get_dialog_base_units_raw() -> u32 {
    (DIALOG_BASE_UNIT_X as u32 & 0xffff) | ((DIALOG_BASE_UNIT_Y as u32 & 0xffff) << 16)
}

fn map_dialog_rect_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let hwnd = raw_arg(args, 0);
    let rect_ptr = raw_arg(args, 1);
    if hwnd == 0 || !kernel.gwe.is_window(hwnd) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return false;
    }
    let Some(rect) = read_guest_rect(kernel, memory, thread_id, rect_ptr) else {
        return false;
    };
    let mapped = Rect {
        left: mul_div_i32(rect.left, DIALOG_BASE_UNIT_X, 4),
        top: mul_div_i32(rect.top, DIALOG_BASE_UNIT_Y, 8),
        right: mul_div_i32(rect.right, DIALOG_BASE_UNIT_X, 4),
        bottom: mul_div_i32(rect.bottom, DIALOG_BASE_UNIT_Y, 8),
    };
    if !write_guest_rect(kernel, memory, thread_id, rect_ptr, mapped) {
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn mul_div_i32(value: i32, numerator: i32, denominator: i32) -> i32 {
    (((value as i64) * (numerator as i64)) / (denominator as i64)) as i32
}

fn get_next_dlg_tab_item_raw(kernel: &mut CeKernel, thread_id: u32, args: &[u32]) -> u32 {
    let dialog = raw_arg(args, 0);
    let control = raw_arg(args, 1);
    let previous = raw_arg(args, 2) != 0;
    let hwnd = kernel
        .gwe
        .get_next_dlg_tab_item(dialog, control, previous)
        .unwrap_or(0);
    if hwnd == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
    } else {
        kernel.threads.set_last_error(thread_id, 0);
    }
    hwnd
}

fn get_next_dlg_group_item_raw(kernel: &mut CeKernel, thread_id: u32, args: &[u32]) -> u32 {
    let dialog = raw_arg(args, 0);
    let control = raw_arg(args, 1);
    let previous = raw_arg(args, 2) != 0;
    let hwnd = kernel
        .gwe
        .get_next_dlg_group_item(dialog, control, previous)
        .unwrap_or(0);
    if hwnd == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
    } else {
        kernel.threads.set_last_error(thread_id, 0);
    }
    hwnd
}

fn map_window_points<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    hwnd_from: u32,
    hwnd_to: u32,
    points_ptr: u32,
    point_count: u32,
) -> u32 {
    let mut points = Vec::new();
    for index in 0..point_count {
        let point_ptr = points_ptr.wrapping_add(index.wrapping_mul(8));
        let Some(point) = read_guest_point(kernel, memory, thread_id, point_ptr) else {
            return 0;
        };
        points.push(point);
    }
    let first_before = points.first().copied();
    let from = (hwnd_from != 0).then_some(hwnd_from);
    let to = (hwnd_to != 0).then_some(hwnd_to);
    if !kernel.gwe.map_window_points(from, to, &mut points) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    }
    let packed_delta = first_before
        .zip(points.first().copied())
        .map(|(before, after)| {
            let dx = after.x.saturating_sub(before.x) as i16 as u16;
            let dy = after.y.saturating_sub(before.y) as i16 as u16;
            u32::from(dx) | (u32::from(dy) << 16)
        })
        .unwrap_or(0);
    for (index, point) in points.into_iter().enumerate() {
        let point_ptr = points_ptr.wrapping_add((index as u32).wrapping_mul(8));
        if !write_guest_point(kernel, memory, thread_id, point_ptr, point) {
            return 0;
        }
    }
    packed_delta
}

fn invalidate_rect_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let hwnd = raw_arg(args, 0);
    let rect_ptr = raw_arg(args, 1);
    let erase = raw_arg(args, 2) != 0;
    let rect = if rect_ptr == 0 {
        None
    } else {
        match read_guest_rect(kernel, memory, thread_id, rect_ptr) {
            Some(rect) => Some(rect),
            None => return false,
        }
    };
    if !kernel.gwe.invalidate_window(hwnd, rect, erase) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn redraw_window_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let hwnd = raw_arg(args, 0);
    let rect_ptr = raw_arg(args, 1);
    let region_handle = raw_arg(args, 2);
    let flags = raw_arg(args, 3);

    if !kernel.gwe.is_window(hwnd) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return false;
    }

    let rect = if region_handle != 0 {
        match kernel.resources.region(region_handle) {
            Some(region) => Some(region.rect),
            None => {
                kernel
                    .threads
                    .set_last_error(thread_id, ERROR_INVALID_HANDLE);
                return false;
            }
        }
    } else if rect_ptr != 0 {
        match read_guest_rect(kernel, memory, thread_id, rect_ptr) {
            Some(rect) => Some(rect),
            None => return false,
        }
    } else {
        None
    };

    let include_children = flags & RDW_ALLCHILDREN != 0 && flags & RDW_NOCHILDREN == 0;
    let targets = if include_children {
        match kernel.gwe.window_and_descendants(hwnd) {
            Some(targets) => targets,
            None => {
                kernel
                    .threads
                    .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
                return false;
            }
        }
    } else {
        vec![hwnd]
    };

    let should_validate = flags & (RDW_VALIDATE | RDW_NOINTERNALPAINT) != 0;
    let should_invalidate = flags & (RDW_INVALIDATE | RDW_INTERNALPAINT) != 0;
    let erase = flags & RDW_NOERASE == 0 && flags & (RDW_ERASE | RDW_ERASENOW) != 0;

    for target in targets.iter().copied() {
        if should_validate && !kernel.gwe.validate_window_rect(target, rect) {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
            return false;
        }
        if should_invalidate && !kernel.gwe.invalidate_window(target, rect, erase) {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
            return false;
        }
    }

    if flags & (RDW_UPDATENOW | RDW_ERASENOW) != 0 {
        for target in targets {
            if !kernel.update_window(target) {
                kernel
                    .threads
                    .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
                return false;
            }
        }
    }

    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn validate_rect_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let hwnd = raw_arg(args, 0);
    let rect_ptr = raw_arg(args, 1);
    let rect = if rect_ptr == 0 {
        None
    } else {
        match read_guest_rect(kernel, memory, thread_id, rect_ptr) {
            Some(rect) => Some(rect),
            None => return false,
        }
    };
    if !kernel.gwe.validate_window_rect(hwnd, rect) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn get_update_rect_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    hwnd: u32,
    rect_ptr: u32,
    erase: bool,
) -> bool {
    if !kernel.gwe.is_window(hwnd) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return false;
    }
    let Some(update) = kernel.gwe.update_rect(hwnd) else {
        kernel.threads.set_last_error(thread_id, 0);
        return false;
    };
    if rect_ptr != 0 && !write_guest_rect(kernel, memory, thread_id, rect_ptr, update.rect) {
        return false;
    }
    if erase && update.erase {
        let hdc = paint_hdc_for_hwnd(hwnd);
        if !kernel.erase_window_background(hwnd, hdc) {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
            return false;
        }
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn get_update_rgn_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    hwnd: u32,
    region: u32,
    erase: bool,
) -> u32 {
    if region == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return ERROR_REGION;
    }
    if !kernel.gwe.is_window(hwnd) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return ERROR_REGION;
    }
    let update = kernel.gwe.update_rect(hwnd);
    let rect = update.map(|update| update.rect).unwrap_or_default();
    if !kernel.resources.set_region(region, rect) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return ERROR_REGION;
    }
    if erase && update.is_some_and(|update| update.erase) {
        let hdc = paint_hdc_for_hwnd(hwnd);
        if !kernel.erase_window_background(hwnd, hdc) {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
            return ERROR_REGION;
        }
    }
    kernel.threads.set_last_error(thread_id, 0);
    region_status(rect)
}

fn get_window_thread_process_id_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let hwnd = raw_arg(args, 0);
    let process_id_ptr = raw_arg(args, 1);
    let Some((owner_thread_id, owner_process_id)) = kernel.gwe.window_thread_process_id(hwnd)
    else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    };
    if process_id_ptr != 0
        && !write_guest_u32(kernel, memory, thread_id, process_id_ptr, owner_process_id)
    {
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    owner_thread_id
}

fn begin_paint_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    hwnd: u32,
    paint_ptr: u32,
) -> u32 {
    let Some(update) = kernel.gwe.begin_paint(hwnd) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    };
    let hdc = paint_hdc_for_hwnd(hwnd);
    if paint_ptr != 0 && !write_paint_struct(kernel, memory, thread_id, paint_ptr, hdc, update) {
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    hdc
}

fn end_paint_raw(kernel: &mut CeKernel, thread_id: u32, hwnd: u32) -> bool {
    if !kernel.gwe.validate_window(hwnd) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn get_dc_raw(kernel: &CeKernel, hwnd: u32) -> u32 {
    let target = if hwnd == 0 {
        kernel.gwe.get_desktop_window()
    } else {
        hwnd
    };
    if kernel.gwe.is_window(target) {
        paint_hdc_for_hwnd(target)
    } else {
        0
    }
}

fn release_dc_raw(kernel: &CeKernel, hwnd: u32, hdc: u32) -> u32 {
    if hdc == 0 {
        return 0;
    }
    if hwnd == 0 || kernel.gwe.is_window(hwnd) {
        1
    } else {
        0
    }
}

fn get_device_caps_raw(kernel: &CeKernel, hdc: u32, index: u32) -> u32 {
    if hdc == 0 {
        return 0;
    }
    match index {
        HORZRES => kernel.gwe.system_metric(crate::ce::gwe::SM_CXSCREEN) as u32,
        VERTRES => kernel.gwe.system_metric(crate::ce::gwe::SM_CYSCREEN) as u32,
        BITSPIXEL => 16,
        PLANES => 1,
        LOGPIXELSX | LOGPIXELSY => 96,
        _ => 0,
    }
}

fn create_compatible_dc_raw(kernel: &mut CeKernel, thread_id: u32, hdc: u32) -> u32 {
    let _ = hdc;
    kernel.threads.set_last_error(thread_id, 0);
    kernel.resources.create_compatible_dc()
}

fn delete_dc_raw(kernel: &mut CeKernel, thread_id: u32, hdc: u32) -> bool {
    if hdc == 0 || !kernel.resources.delete_dc(hdc) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn create_compatible_bitmap_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    hdc: u32,
    width: i32,
    height: i32,
) -> u32 {
    if hdc == 0 || width <= 0 || height <= 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let Some(byte_count) = bitmap_byte_count(width, height, 32) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let Some(bits_ptr) =
        kernel
            .memory
            .heap_alloc(PROCESS_HEAP_HANDLE, HEAP_ZERO_MEMORY, byte_count)
    else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    };
    kernel.threads.set_last_error(thread_id, 0);
    kernel
        .resources
        .create_owned_bitmap(width, height, 1, 32, bits_ptr)
}

fn create_dib_section_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let _hdc = raw_arg(args, 0);
    let info_ptr = raw_arg(args, 1);
    let color_usage = raw_arg(args, 2);
    let bits_out = raw_arg(args, 3);
    let section = raw_arg(args, 4);
    let offset = raw_arg(args, 5);
    if info_ptr == 0 || bits_out == 0 || color_usage != DIB_RGB_COLORS || offset != 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    if section != 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_NOT_SUPPORTED);
        return 0;
    }
    let Some(header_size) = read_guest_u32(kernel, memory, thread_id, info_ptr) else {
        return 0;
    };
    if !(40..=124).contains(&header_size) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let Some(header_bytes) = read_guest_bytes(kernel, memory, thread_id, info_ptr, header_size)
    else {
        return 0;
    };
    let Some(header) = parse_dib_header(&header_bytes) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let Some(compression) = read_le_u32(&header_bytes, 16) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let rgb_masks = match dib_rgb_masks(
        kernel,
        memory,
        thread_id,
        info_ptr,
        header_size,
        compression,
        header.bits_pixel,
    ) {
        Some(masks) => masks,
        None => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            return 0;
        }
    };
    let color_table = read_dib_color_table(
        memory,
        info_ptr,
        header_size,
        compression,
        header.bits_pixel,
    );
    if header.width == 0 || header.height == 0 || header.planes != 1 || header.bits_pixel == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let Some(byte_count) = bitmap_byte_count(header.width, header.height, header.bits_pixel) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let Some(bits_ptr) =
        kernel
            .memory
            .heap_alloc(PROCESS_HEAP_HANDLE, HEAP_ZERO_MEMORY, byte_count)
    else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    };
    if !write_guest_u32(kernel, memory, thread_id, bits_out, bits_ptr) {
        let _ = kernel.memory.heap_free(PROCESS_HEAP_HANDLE, 0, bits_ptr);
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    let bitmap = kernel.resources.create_owned_bitmap_with_masks(
        header.width,
        header.height,
        header.planes,
        header.bits_pixel,
        bits_ptr,
        rgb_masks,
    );
    if !color_table.is_empty() {
        if let Some(object) = kernel.resources.bitmap_mut(bitmap) {
            object.color_table = color_table;
        }
    }
    bitmap
}

fn create_font_indirect_w_raw(kernel: &mut CeKernel, thread_id: u32, logfont_ptr: u32) -> u32 {
    if logfont_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    kernel.resources.create_font(logfont_ptr)
}

fn create_solid_brush_raw(kernel: &mut CeKernel, color: u32) -> u32 {
    kernel.resources.create_brush(color)
}

fn create_pattern_brush_raw(kernel: &mut CeKernel, thread_id: u32, bitmap: u32) -> u32 {
    let Some(handle) = kernel.resources.create_pattern_brush(bitmap) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    kernel.threads.set_last_error(thread_id, 0);
    handle
}

fn create_pen_raw(kernel: &mut CeKernel, style: u32, width: i32, color: u32) -> u32 {
    kernel.resources.create_pen(style, width, color)
}

fn create_pen_indirect_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    logpen_ptr: u32,
) -> u32 {
    if logpen_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let Some(bytes) = read_guest_bytes(kernel, memory, thread_id, logpen_ptr, 16) else {
        return 0;
    };
    let Some(style) = read_le_u32(&bytes, 0) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let Some(width) = read_le_i32(&bytes, 4) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let Some(color) = read_le_u32(&bytes, 12) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    kernel.threads.set_last_error(thread_id, 0);
    kernel.resources.create_pen(style, width, color)
}

fn create_palette_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    log_palette_ptr: u32,
) -> u32 {
    if log_palette_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let Some(header) = read_guest_bytes(kernel, memory, thread_id, log_palette_ptr, 4) else {
        return 0;
    };
    let Some(entry_count) = read_le_u16(&header, 2).map(u32::from) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    if entry_count > MAX_LOG_PALETTE_ENTRIES {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let Some(byte_count) = entry_count.checked_mul(PALETTE_ENTRY_SIZE_U32) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let Some(bytes) = read_guest_bytes(
        kernel,
        memory,
        thread_id,
        log_palette_ptr.wrapping_add(4),
        byte_count,
    ) else {
        return 0;
    };
    let entries = bytes
        .chunks_exact(PALETTE_ENTRY_SIZE)
        .map(|entry| [entry[0], entry[1], entry[2], entry[3]])
        .collect();
    kernel.threads.set_last_error(thread_id, 0);
    kernel.resources.create_palette(entries)
}

fn get_palette_entries_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    palette: u32,
    start: u32,
    count: u32,
    entries_ptr: u32,
) -> u32 {
    let Some(entries) = kernel
        .resources
        .palette(palette)
        .map(|palette| &palette.entries)
    else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    let Some(bytes) = palette_entry_bytes(entries, start, count) else {
        kernel.threads.set_last_error(thread_id, 0);
        return 0;
    };
    if entries_ptr == 0 || bytes.is_empty() {
        kernel.threads.set_last_error(thread_id, 0);
        return (bytes.len() / PALETTE_ENTRY_SIZE) as u32;
    }
    if !write_guest_bytes(kernel, memory, thread_id, entries_ptr, &bytes) {
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    (bytes.len() / PALETTE_ENTRY_SIZE) as u32
}

fn set_palette_entries_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    palette: u32,
    start: u32,
    count: u32,
    entries_ptr: u32,
) -> u32 {
    let Some(entries_len) = kernel
        .resources
        .palette(palette)
        .map(|palette| palette.entries.len())
    else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    let Ok(start_index) = usize::try_from(start) else {
        kernel.threads.set_last_error(thread_id, 0);
        return 0;
    };
    if start_index >= entries_len || count == 0 {
        kernel.threads.set_last_error(thread_id, 0);
        return 0;
    }
    if entries_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let copy_count = usize::try_from(count)
        .ok()
        .unwrap_or(usize::MAX)
        .min(entries_len - start_index);
    let Some(byte_count) = (copy_count as u32).checked_mul(PALETTE_ENTRY_SIZE_U32) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let Some(bytes) = read_guest_bytes(kernel, memory, thread_id, entries_ptr, byte_count) else {
        return 0;
    };
    if let Some(palette) = kernel.resources.palette_mut(palette) {
        for (index, entry) in bytes.chunks_exact(PALETTE_ENTRY_SIZE).enumerate() {
            palette.entries[start_index + index] = [entry[0], entry[1], entry[2], entry[3]];
        }
    }
    kernel.threads.set_last_error(thread_id, 0);
    copy_count as u32
}

fn get_system_palette_entries_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    hdc: u32,
    start: u32,
    count: u32,
    entries_ptr: u32,
) -> u32 {
    if hdc == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    }
    let entries = system_palette_entries();
    let Some(bytes) = palette_entry_bytes(&entries, start, count) else {
        kernel.threads.set_last_error(thread_id, 0);
        return 0;
    };
    if entries_ptr == 0 || bytes.is_empty() {
        kernel.threads.set_last_error(thread_id, 0);
        return (bytes.len() / PALETTE_ENTRY_SIZE) as u32;
    }
    if !write_guest_bytes(kernel, memory, thread_id, entries_ptr, &bytes) {
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    (bytes.len() / PALETTE_ENTRY_SIZE) as u32
}

fn get_nearest_palette_index_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    palette: u32,
    colorref: u32,
) -> u32 {
    let Some(entries) = kernel
        .resources
        .palette(palette)
        .map(|palette| &palette.entries)
    else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return u32::MAX;
    };
    let red = (colorref & 0xff) as i32;
    let green = ((colorref >> 8) & 0xff) as i32;
    let blue = ((colorref >> 16) & 0xff) as i32;
    let Some((index, _distance)) = entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let dr = i32::from(entry[0]) - red;
            let dg = i32::from(entry[1]) - green;
            let db = i32::from(entry[2]) - blue;
            (index, dr * dr + dg * dg + db * db)
        })
        .min_by_key(|(_, distance)| *distance)
    else {
        kernel.threads.set_last_error(thread_id, 0);
        return u32::MAX;
    };
    kernel.threads.set_last_error(thread_id, 0);
    index as u32
}

fn select_palette_raw(kernel: &mut CeKernel, thread_id: u32, hdc: u32, palette: u32) -> u32 {
    let Some(previous) = kernel.resources.select_palette(hdc, palette) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    kernel.threads.set_last_error(thread_id, 0);
    previous
}

fn realize_palette_raw(kernel: &mut CeKernel, thread_id: u32, hdc: u32) -> u32 {
    let Some(mapped) = kernel.resources.realize_palette(hdc) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    kernel.threads.set_last_error(thread_id, 0);
    mapped
}

fn palette_entry_bytes(entries: &[[u8; 4]], start: u32, count: u32) -> Option<Vec<u8>> {
    let start = usize::try_from(start).ok()?;
    if start >= entries.len() || count == 0 {
        return Some(Vec::new());
    }
    let count = usize::try_from(count).ok().unwrap_or(usize::MAX);
    let end = start.saturating_add(count).min(entries.len());
    Some(entries[start..end].iter().flatten().copied().collect())
}

fn system_palette_entries() -> Vec<[u8; 4]> {
    (0..=255).map(|value| [value, value, value, 0]).collect()
}

fn get_stock_object_raw(index: u32) -> u32 {
    stock_object_handle(index).unwrap_or(0)
}

fn select_object_raw(kernel: &mut CeKernel, thread_id: u32, hdc: u32, object: u32) -> u32 {
    let Some(previous) = kernel.resources.select_object(hdc, object) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    kernel.threads.set_last_error(thread_id, 0);
    previous
}

fn set_bk_mode_raw(kernel: &mut CeKernel, thread_id: u32, hdc: u32, mode: i32) -> u32 {
    let Some(previous) = kernel.resources.set_dc_bk_mode(hdc, mode) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    kernel.threads.set_last_error(thread_id, 0);
    previous as u32
}

fn set_bk_color_raw(kernel: &mut CeKernel, thread_id: u32, hdc: u32, color: u32) -> u32 {
    let Some(previous) = kernel.resources.set_dc_bk_color(hdc, color) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return CLR_INVALID;
    };
    kernel.threads.set_last_error(thread_id, 0);
    previous
}

fn set_text_color_raw(kernel: &mut CeKernel, thread_id: u32, hdc: u32, color: u32) -> u32 {
    let Some(previous) = kernel.resources.set_dc_text_color(hdc, color) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return u32::MAX;
    };
    kernel.threads.set_last_error(thread_id, 0);
    previous
}

fn set_text_align_raw(kernel: &mut CeKernel, thread_id: u32, hdc: u32, align: u32) -> u32 {
    let Some(previous) = kernel.resources.set_dc_text_align(hdc, align) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return u32::MAX;
    };
    kernel.threads.set_last_error(thread_id, 0);
    previous
}

fn fill_rect_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    framebuffer: Option<&mut dyn Framebuffer>,
    thread_id: u32,
    hdc: u32,
    rect_ptr: u32,
    brush: u32,
) -> u32 {
    if hdc == 0 || rect_ptr == 0 || brush == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let Some(rect) = read_guest_rect(kernel, memory, thread_id, rect_ptr) else {
        return 0;
    };
    let Some(color) = brush_colorref(kernel, brush) else {
        kernel.threads.set_last_error(thread_id, 0);
        return 1;
    };
    if let Some(framebuffer) = framebuffer {
        fill_framebuffer_rect_for_hdc(kernel, framebuffer, hdc, rect, color);
    }
    kernel.threads.set_last_error(thread_id, 0);
    1
}

fn fill_framebuffer_rect_for_hdc(
    kernel: &CeKernel,
    framebuffer: &mut dyn Framebuffer,
    hdc: u32,
    rect: Rect,
    colorref: u32,
) {
    let Some(hwnd) = hdc_to_hwnd(hdc) else {
        return;
    };
    let Some(client_origin) = kernel.gwe.client_to_screen(hwnd, Point { x: 0, y: 0 }) else {
        return;
    };
    let Some(client_rect) = kernel.gwe.get_client_rect(hwnd) else {
        return;
    };
    let mut rect = intersect_rect_value(normalize_rect(rect), client_rect).unwrap_or_default();
    if let Some(clip) = kernel
        .resources
        .clip_region(hdc)
        .and_then(|region| kernel.resources.region(region))
        .map(|region| region.rect)
    {
        rect = intersect_rect_value(rect, clip).unwrap_or_default();
    }
    if is_rect_empty_value(rect) {
        return;
    }
    let screen_rect = rect.offset(client_origin.x, client_origin.y);
    fill_framebuffer_screen_rect(framebuffer, screen_rect, colorref);
}

fn fill_framebuffer_screen_rect(framebuffer: &mut dyn Framebuffer, rect: Rect, colorref: u32) {
    let info = framebuffer.info();
    let left = rect.left.max(0).min(info.width as i32) as u32;
    let top = rect.top.max(0).min(info.height as i32) as u32;
    let right = rect.right.max(0).min(info.width as i32) as u32;
    let bottom = rect.bottom.max(0).min(info.height as i32) as u32;
    if right <= left || bottom <= top {
        return;
    }
    let pixel = pixel_bytes_for_colorref(info.format, colorref);
    let bytes_per_pixel = info.format.bytes_per_pixel();
    let pixels = framebuffer.pixels_mut();
    for y in top as usize..bottom as usize {
        let row_start = y * info.stride;
        for x in left as usize..right as usize {
            let offset = row_start + x * bytes_per_pixel;
            pixels[offset..offset + bytes_per_pixel].copy_from_slice(&pixel[..bytes_per_pixel]);
        }
    }
    framebuffer.mark_dirty(FramebufferRect::new(left, top, right - left, bottom - top));
}

fn pixel_bytes_for_colorref(format: PixelFormat, colorref: u32) -> [u8; 4] {
    let [red, green, blue] = colorref_rgb(colorref);
    pixel_bytes_for_rgb(format, [red, green, blue])
}

fn colorref_rgb(colorref: u32) -> [u8; 3] {
    [
        (colorref & 0xff) as u8,
        ((colorref >> 8) & 0xff) as u8,
        ((colorref >> 16) & 0xff) as u8,
    ]
}

fn pixel_bytes_for_rgb(format: PixelFormat, rgb: [u8; 3]) -> [u8; 4] {
    let [red, green, blue] = rgb;
    match format {
        PixelFormat::Rgb565 => {
            let raw = colorref_to_rgb565(red, green, blue);
            [raw as u8, (raw >> 8) as u8, 0, 0]
        }
        PixelFormat::Bgra8888 => [blue, green, red, 0xff],
        PixelFormat::Rgba8888 => [red, green, blue, 0xff],
        PixelFormat::Gray8 => {
            let gray =
                ((u16::from(red) * 30 + u16::from(green) * 59 + u16::from(blue) * 11) / 100) as u8;
            [gray, 0, 0, 0]
        }
    }
}

fn colorref_to_rgb565(red: u8, green: u8, blue: u8) -> u16 {
    ((u16::from(red) & 0xf8) << 8) | ((u16::from(green) & 0xfc) << 3) | (u16::from(blue) >> 3)
}

fn brush_colorref(kernel: &CeKernel, brush: u32) -> Option<u32> {
    if brush & 0xffff_ff00 == 0x000b_4000 {
        return Some(get_sys_color(brush & 0xff));
    }
    if Some(brush) == stock_object_handle(0) {
        return Some(rgb(0xff, 0xff, 0xff));
    }
    if Some(brush) == stock_object_handle(1) {
        return Some(rgb(0xc0, 0xc0, 0xc0));
    }
    if Some(brush) == stock_object_handle(2) {
        return Some(rgb(0x80, 0x80, 0x80));
    }
    if Some(brush) == stock_object_handle(3) {
        return Some(rgb(0x40, 0x40, 0x40));
    }
    if Some(brush) == stock_object_handle(4) {
        return Some(rgb(0x00, 0x00, 0x00));
    }
    kernel
        .resources
        .brush(brush)
        .and_then(|brush| brush.pattern_bitmap.is_none().then_some(brush.color))
}

fn pen_colorref(kernel: &CeKernel, pen: u32) -> Option<u32> {
    if Some(pen) == stock_object_handle(6) {
        return Some(rgb(0xff, 0xff, 0xff));
    }
    if Some(pen) == stock_object_handle(7) {
        return Some(rgb(0x00, 0x00, 0x00));
    }
    if Some(pen) == stock_object_handle(8) {
        return None;
    }
    if Some(pen) == stock_object_handle(19) {
        return Some(rgb(0x00, 0x00, 0x00));
    }
    kernel
        .resources
        .pen(pen)
        .and_then(|pen| (pen.style != 5).then_some(pen.color))
}

fn selected_pen_colorref(kernel: &CeKernel, hdc: u32) -> Option<u32> {
    match kernel.resources.selected_pen(hdc) {
        Some(pen) => pen_colorref(kernel, pen),
        None => Some(rgb(0x00, 0x00, 0x00)),
    }
}

fn bit_blt_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    framebuffer: Option<&mut dyn Framebuffer>,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let dst = raw_arg(args, 0);
    let dst_x = raw_i32_arg(args, 1);
    let dst_y = raw_i32_arg(args, 2);
    let width = raw_i32_arg(args, 3);
    let height = raw_i32_arg(args, 4);
    let src = raw_arg(args, 5);
    let src_x = raw_i32_arg(args, 6);
    let src_y = raw_i32_arg(args, 7);
    let rop = raw_arg(args, 8);
    if dst == 0 || src == 0 || width <= 0 || height <= 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    if rop == SRCCOPY
        && let Some(framebuffer) = framebuffer
        && kernel.resources.is_memory_dc(src)
        && let Some(bitmap) = kernel.resources.selected_bitmap(src)
    {
        blit_selected_bitmap_to_framebuffer(
            kernel,
            memory,
            thread_id,
            framebuffer,
            dst,
            dst_x,
            dst_y,
            width,
            height,
            bitmap,
            src_x,
            src_y,
        );
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn blit_selected_bitmap_to_framebuffer<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    framebuffer: &mut dyn Framebuffer,
    hdc: u32,
    dst_x: i32,
    dst_y: i32,
    width: i32,
    height: i32,
    bitmap_handle: u32,
    src_x: i32,
    src_y: i32,
) {
    let Some(bitmap) = kernel.resources.bitmap(bitmap_handle).cloned() else {
        return;
    };
    if bitmap.bits_ptr == 0 || bitmap.width <= 0 || bitmap.height <= 0 || bitmap.width_bytes <= 0 {
        return;
    }
    let byte_count = bitmap.width_bytes.saturating_mul(bitmap.height) as u32;
    let Some(bitmap_bytes) =
        read_guest_bytes(kernel, memory, thread_id, bitmap.bits_ptr, byte_count)
    else {
        return;
    };
    let Some(hwnd) = hdc_to_hwnd(hdc) else {
        return;
    };
    let Some(client_origin) = kernel.gwe.client_to_screen(hwnd, Point { x: 0, y: 0 }) else {
        return;
    };
    let Some(client_rect) = kernel.gwe.get_client_rect(hwnd) else {
        return;
    };
    let mut dst_rect = Rect {
        left: dst_x,
        top: dst_y,
        right: dst_x.saturating_add(width),
        bottom: dst_y.saturating_add(height),
    };
    dst_rect = intersect_rect_value(normalize_rect(dst_rect), client_rect).unwrap_or_default();
    if let Some(clip) = kernel
        .resources
        .clip_region(hdc)
        .and_then(|region| kernel.resources.region(region))
        .map(|region| region.rect)
    {
        dst_rect = intersect_rect_value(dst_rect, clip).unwrap_or_default();
    }
    if is_rect_empty_value(dst_rect) {
        return;
    }

    let info = framebuffer.info();
    let screen_rect = dst_rect.offset(client_origin.x, client_origin.y);
    let left = screen_rect.left.max(0).min(info.width as i32);
    let top = screen_rect.top.max(0).min(info.height as i32);
    let right = screen_rect.right.max(0).min(info.width as i32);
    let bottom = screen_rect.bottom.max(0).min(info.height as i32);
    if right <= left || bottom <= top {
        return;
    }

    let bytes_per_pixel = info.format.bytes_per_pixel();
    let dst_stride = info.stride;
    let pixels = framebuffer.pixels_mut();
    for screen_y in top..bottom {
        let client_y = screen_y - client_origin.y;
        let source_y = src_y + (client_y - dst_y);
        for screen_x in left..right {
            let client_x = screen_x - client_origin.x;
            let source_x = src_x + (client_x - dst_x);
            let Some(rgb) = bitmap_pixel_rgb(&bitmap, &bitmap_bytes, source_x, source_y) else {
                continue;
            };
            let pixel = pixel_bytes_for_rgb(info.format, rgb);
            let offset = (screen_y as usize * dst_stride) + (screen_x as usize * bytes_per_pixel);
            pixels[offset..offset + bytes_per_pixel].copy_from_slice(&pixel[..bytes_per_pixel]);
        }
    }
    framebuffer.mark_dirty(FramebufferRect::new(
        left as u32,
        top as u32,
        (right - left) as u32,
        (bottom - top) as u32,
    ));
}

fn draw_dib_to_framebuffer<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    framebuffer: &mut dyn Framebuffer,
    hdc: u32,
    dst_x: i32,
    dst_y: i32,
    dst_width: i32,
    dst_height: i32,
    src_x: i32,
    src_y: i32,
    src_width: i32,
    src_height: i32,
    bits_ptr: u32,
    info_ptr: u32,
) {
    let Some((bitmap, bitmap_bytes)) =
        read_dib_source(kernel, memory, thread_id, bits_ptr, info_ptr)
    else {
        return;
    };
    draw_bitmap_bytes_to_framebuffer(
        kernel,
        framebuffer,
        hdc,
        dst_x,
        dst_y,
        dst_width,
        dst_height,
        src_x,
        src_y,
        src_width,
        src_height,
        &bitmap,
        &bitmap_bytes,
        None,
    );
}

fn read_dib_source<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    bits_ptr: u32,
    info_ptr: u32,
) -> Option<(crate::ce::resource::BitmapObject, Vec<u8>)> {
    let header_size = read_guest_u32(kernel, memory, thread_id, info_ptr)?;
    if !(12..=124).contains(&header_size) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return None;
    }
    let header_bytes = read_guest_bytes(kernel, memory, thread_id, info_ptr, header_size)?;
    let Some(header) = parse_dib_header(&header_bytes) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return None;
    };
    if header.width <= 0 || header.height == 0 || header.planes != 1 || header.bits_pixel == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return None;
    }
    let compression = if header_size >= 40 {
        read_le_u32(&header_bytes, 16)?
    } else {
        BI_RGB
    };
    let rgb_masks = dib_rgb_masks(
        kernel,
        memory,
        thread_id,
        info_ptr,
        header_size,
        compression,
        header.bits_pixel,
    )?;
    let color_table = read_dib_color_table(
        memory,
        info_ptr,
        header_size,
        compression,
        header.bits_pixel,
    );
    let height = header.height.checked_abs()?;
    let byte_count = bitmap_byte_count(header.width, header.height, header.bits_pixel)?;
    let width_bytes = (byte_count / height as u32) as i32;
    let bytes = read_guest_bytes(kernel, memory, thread_id, bits_ptr, byte_count)?;
    Some((
        crate::ce::resource::BitmapObject {
            handle: 0,
            width: header.width,
            height,
            top_down: header.height < 0,
            width_bytes,
            planes: header.planes,
            bits_pixel: header.bits_pixel,
            rgb_masks,
            color_table,
            bits_ptr,
            bits_owned: false,
        },
        bytes,
    ))
}

fn draw_bitmap_bytes_to_framebuffer(
    kernel: &CeKernel,
    framebuffer: &mut dyn Framebuffer,
    hdc: u32,
    dst_x: i32,
    dst_y: i32,
    dst_width: i32,
    dst_height: i32,
    src_x: i32,
    src_y: i32,
    src_width: i32,
    src_height: i32,
    bitmap: &crate::ce::resource::BitmapObject,
    bitmap_bytes: &[u8],
    transparent_rgb: Option<[u8; 3]>,
) {
    if dst_width == 0 || dst_height == 0 || src_width == 0 || src_height == 0 {
        return;
    }
    let Some(hwnd) = hdc_to_hwnd(hdc) else {
        return;
    };
    let Some(client_origin) = kernel.gwe.client_to_screen(hwnd, Point { x: 0, y: 0 }) else {
        return;
    };
    let Some(client_rect) = kernel.gwe.get_client_rect(hwnd) else {
        return;
    };
    let mut dst_rect = Rect {
        left: dst_x,
        top: dst_y,
        right: dst_x.saturating_add(dst_width),
        bottom: dst_y.saturating_add(dst_height),
    };
    dst_rect = intersect_rect_value(normalize_rect(dst_rect), client_rect).unwrap_or_default();
    if let Some(clip) = kernel
        .resources
        .clip_region(hdc)
        .and_then(|region| kernel.resources.region(region))
        .map(|region| region.rect)
    {
        dst_rect = intersect_rect_value(dst_rect, clip).unwrap_or_default();
    }
    if is_rect_empty_value(dst_rect) {
        return;
    }

    let info = framebuffer.info();
    let screen_rect = dst_rect.offset(client_origin.x, client_origin.y);
    let left = screen_rect.left.max(0).min(info.width as i32);
    let top = screen_rect.top.max(0).min(info.height as i32);
    let right = screen_rect.right.max(0).min(info.width as i32);
    let bottom = screen_rect.bottom.max(0).min(info.height as i32);
    if right <= left || bottom <= top {
        return;
    }

    let dst_width_abs = dst_width.abs().max(1);
    let dst_height_abs = dst_height.abs().max(1);
    let src_width_abs = src_width.abs();
    let src_height_abs = src_height.abs();
    let bytes_per_pixel = info.format.bytes_per_pixel();
    let dst_stride = info.stride;
    let pixels = framebuffer.pixels_mut();
    for screen_y in top..bottom {
        let client_y = screen_y - client_origin.y;
        let dst_rel_y = client_y - dst_y;
        let source_y = src_y + (dst_rel_y * src_height_abs / dst_height_abs);
        for screen_x in left..right {
            let client_x = screen_x - client_origin.x;
            let dst_rel_x = client_x - dst_x;
            let source_x = src_x + (dst_rel_x * src_width_abs / dst_width_abs);
            let Some(rgb) = bitmap_pixel_rgb(bitmap, bitmap_bytes, source_x, source_y) else {
                continue;
            };
            if let Some(transparent) = transparent_rgb
                && rgb == transparent
            {
                continue;
            }
            let pixel = pixel_bytes_for_rgb(info.format, rgb);
            let offset = (screen_y as usize * dst_stride) + (screen_x as usize * bytes_per_pixel);
            pixels[offset..offset + bytes_per_pixel].copy_from_slice(&pixel[..bytes_per_pixel]);
        }
    }
    framebuffer.mark_dirty(FramebufferRect::new(
        left as u32,
        top as u32,
        (right - left) as u32,
        (bottom - top) as u32,
    ));
}

fn draw_bitmap_bytes_to_bitmap<M: CoredllGuestMemory>(
    memory: &mut M,
    dst: &crate::ce::resource::BitmapObject,
    dst_x: i32,
    dst_y: i32,
    dst_width: i32,
    dst_height: i32,
    src_x: i32,
    src_y: i32,
    src_width: i32,
    src_height: i32,
    src: &crate::ce::resource::BitmapObject,
    src_bytes: &[u8],
    transparent_rgb: Option<[u8; 3]>,
) {
    if dst.bits_ptr == 0
        || dst.width <= 0
        || dst.height <= 0
        || dst.width_bytes <= 0
        || dst_width == 0
        || dst_height == 0
        || src_width == 0
        || src_height == 0
    {
        return;
    }
    let left = dst_x.max(0).min(dst.width);
    let top = dst_y.max(0).min(dst.height);
    let right = dst_x.saturating_add(dst_width).max(0).min(dst.width);
    let bottom = dst_y.saturating_add(dst_height).max(0).min(dst.height);
    if right <= left || bottom <= top {
        return;
    }
    let dst_width_abs = dst_width.abs().max(1);
    let dst_height_abs = dst_height.abs().max(1);
    let src_width_abs = src_width.abs();
    let src_height_abs = src_height.abs();
    for y in top..bottom {
        let dst_rel_y = y - dst_y;
        let source_y = src_y + (dst_rel_y * src_height_abs / dst_height_abs);
        for x in left..right {
            let dst_rel_x = x - dst_x;
            let source_x = src_x + (dst_rel_x * src_width_abs / dst_width_abs);
            let Some(rgb) = bitmap_pixel_rgb(src, src_bytes, source_x, source_y) else {
                continue;
            };
            if let Some(transparent) = transparent_rgb
                && rgb == transparent
            {
                continue;
            }
            let _ = write_bitmap_pixel_rgb(memory, dst, x, y, rgb);
        }
    }
}

fn write_bitmap_pixel_rgb<M: CoredllGuestMemory>(
    memory: &mut M,
    bitmap: &crate::ce::resource::BitmapObject,
    x: i32,
    y: i32,
    rgb: [u8; 3],
) -> Result<()> {
    if x < 0 || y < 0 || x >= bitmap.width || y >= bitmap.height {
        return Ok(());
    }
    let row = if bitmap.top_down {
        y
    } else {
        bitmap.height - 1 - y
    } as u32;
    let x = x as u32;
    let row_start = row.saturating_mul(bitmap.width_bytes as u32);
    let addr = match bitmap.bits_pixel {
        32 => bitmap.bits_ptr.wrapping_add(row_start).wrapping_add(x * 4),
        24 => bitmap.bits_ptr.wrapping_add(row_start).wrapping_add(x * 3),
        16 => bitmap.bits_ptr.wrapping_add(row_start).wrapping_add(x * 2),
        8 => bitmap.bits_ptr.wrapping_add(row_start).wrapping_add(x),
        _ => return Ok(()),
    };
    let [red, green, blue] = rgb;
    match bitmap.bits_pixel {
        32 => {
            if let Some([red_mask, green_mask, blue_mask]) = bitmap.rgb_masks {
                let Some(raw) = rgb_to_masks(red, green, blue, red_mask, green_mask, blue_mask)
                else {
                    return Ok(());
                };
                memory.write_bytes(addr, &raw.to_le_bytes())
            } else {
                memory.write_bytes(addr, &[blue, green, red, 0xff])
            }
        }
        24 => memory.write_bytes(addr, &[blue, green, red]),
        16 => {
            let raw = if let Some([red_mask, green_mask, blue_mask]) = bitmap.rgb_masks {
                let Some(raw) = rgb_to_masks(red, green, blue, red_mask, green_mask, blue_mask)
                else {
                    return Ok(());
                };
                raw
            } else {
                u32::from(colorref_to_rgb565(red, green, blue))
            };
            memory.write_u16(addr, raw as u16)
        }
        8 => memory.write_u8(addr, nearest_color_table_index(bitmap, [red, green, blue])),
        _ => Ok(()),
    }
}

fn bitmap_pixel_rgb(
    bitmap: &crate::ce::resource::BitmapObject,
    bytes: &[u8],
    x: i32,
    y: i32,
) -> Option<[u8; 3]> {
    if x < 0 || y < 0 || x >= bitmap.width || y >= bitmap.height {
        return None;
    }
    let row = if bitmap.top_down {
        y
    } else {
        bitmap.height - 1 - y
    } as usize;
    let x = x as usize;
    let row_start = row.checked_mul(bitmap.width_bytes as usize)?;
    match bitmap.bits_pixel {
        32 => {
            let offset = row_start.checked_add(x.checked_mul(4)?)?;
            let pixel = bytes.get(offset..offset + 4)?;
            let raw = u32::from_le_bytes([pixel[0], pixel[1], pixel[2], pixel[3]]);
            if let Some([red_mask, green_mask, blue_mask]) = bitmap.rgb_masks {
                Some([
                    component_from_mask(raw, red_mask)?,
                    component_from_mask(raw, green_mask)?,
                    component_from_mask(raw, blue_mask)?,
                ])
            } else {
                Some([pixel[2], pixel[1], pixel[0]])
            }
        }
        24 => {
            let offset = row_start.checked_add(x.checked_mul(3)?)?;
            let pixel = bytes.get(offset..offset + 3)?;
            Some([pixel[2], pixel[1], pixel[0]])
        }
        16 => {
            let offset = row_start.checked_add(x.checked_mul(2)?)?;
            let pixel = bytes.get(offset..offset + 2)?;
            let raw = u16::from_le_bytes([pixel[0], pixel[1]]);
            let raw = u32::from(raw);
            if let Some([red_mask, green_mask, blue_mask]) = bitmap.rgb_masks {
                Some([
                    component_from_mask(raw, red_mask)?,
                    component_from_mask(raw, green_mask)?,
                    component_from_mask(raw, blue_mask)?,
                ])
            } else {
                Some([
                    component_from_mask(raw, 0x0000_f800)?,
                    component_from_mask(raw, 0x0000_07e0)?,
                    component_from_mask(raw, 0x0000_001f)?,
                ])
            }
        }
        8 => bytes
            .get(row_start + x)
            .copied()
            .map(|index| rgb_from_color_table(bitmap, index)),
        _ => None,
    }
}

fn rgb_from_color_table(bitmap: &crate::ce::resource::BitmapObject, index: u8) -> [u8; 3] {
    bitmap
        .color_table
        .get(index as usize)
        .map(|entry| [entry[2], entry[1], entry[0]])
        .unwrap_or([index, index, index])
}

fn nearest_color_table_index(bitmap: &crate::ce::resource::BitmapObject, rgb: [u8; 3]) -> u8 {
    if bitmap.color_table.is_empty() {
        return ((u16::from(rgb[0]) * 30 + u16::from(rgb[1]) * 59 + u16::from(rgb[2]) * 11) / 100)
            as u8;
    }
    bitmap
        .color_table
        .iter()
        .enumerate()
        .min_by_key(|(_, entry)| {
            let red = i32::from(entry[2]) - i32::from(rgb[0]);
            let green = i32::from(entry[1]) - i32::from(rgb[1]);
            let blue = i32::from(entry[0]) - i32::from(rgb[2]);
            red * red + green * green + blue * blue
        })
        .map(|(index, _)| index.min(u8::MAX as usize) as u8)
        .unwrap_or(0)
}

fn component_from_mask(raw: u32, mask: u32) -> Option<u8> {
    if mask == 0 {
        return None;
    }
    let shift = mask.trailing_zeros();
    let bits = mask.count_ones();
    let max = (1u32.checked_shl(bits)?).checked_sub(1)?;
    let value = (raw & mask) >> shift;
    Some(((value * 255 + (max / 2)) / max) as u8)
}

fn component_to_mask(value: u8, mask: u32) -> Option<u32> {
    if mask == 0 {
        return None;
    }
    let shift = mask.trailing_zeros();
    let bits = mask.count_ones();
    let max = (1u32.checked_shl(bits)?).checked_sub(1)?;
    let scaled = (u32::from(value) * max + 127) / 255;
    Some((scaled << shift) & mask)
}

fn rgb_to_masks(
    red: u8,
    green: u8,
    blue: u8,
    red_mask: u32,
    green_mask: u32,
    blue_mask: u32,
) -> Option<u32> {
    Some(
        component_to_mask(red, red_mask)?
            | component_to_mask(green, green_mask)?
            | component_to_mask(blue, blue_mask)?,
    )
}

fn pat_blt_raw(kernel: &mut CeKernel, thread_id: u32, args: &[u32]) -> bool {
    let dst = raw_arg(args, 0);
    let width = raw_i32_arg(args, 3);
    let height = raw_i32_arg(args, 4);
    if dst == 0 || width <= 0 || height <= 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn transparent_image_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    framebuffer: Option<&mut dyn Framebuffer>,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let dst = raw_arg(args, 0);
    let dst_x = raw_i32_arg(args, 1);
    let dst_y = raw_i32_arg(args, 2);
    let dst_width = raw_i32_arg(args, 3);
    let dst_height = raw_i32_arg(args, 4);
    let src = raw_arg(args, 5);
    let src_x = raw_i32_arg(args, 6);
    let src_y = raw_i32_arg(args, 7);
    let src_width = raw_i32_arg(args, 8);
    let src_height = raw_i32_arg(args, 9);
    let transparent_color = raw_arg(args, 10);
    if dst == 0
        || src == 0
        || dst_width <= 0
        || dst_height <= 0
        || src_width <= 0
        || src_height <= 0
    {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    if kernel.resources.is_memory_dc(src)
        && let Some(src_bitmap_handle) = kernel.resources.selected_bitmap(src)
        && let Some(src_bitmap) = kernel.resources.bitmap(src_bitmap_handle).cloned()
        && src_bitmap.bits_ptr != 0
        && src_bitmap.width > 0
        && src_bitmap.height > 0
        && src_bitmap.width_bytes > 0
    {
        let byte_count = src_bitmap.width_bytes.saturating_mul(src_bitmap.height) as u32;
        if let Some(bitmap_bytes) =
            read_guest_bytes(kernel, memory, thread_id, src_bitmap.bits_ptr, byte_count)
        {
            if kernel.resources.is_memory_dc(dst)
                && let Some(dst_bitmap_handle) = kernel.resources.selected_bitmap(dst)
                && let Some(dst_bitmap) = kernel.resources.bitmap(dst_bitmap_handle).cloned()
            {
                draw_bitmap_bytes_to_bitmap(
                    memory,
                    &dst_bitmap,
                    dst_x,
                    dst_y,
                    dst_width,
                    dst_height,
                    src_x,
                    src_y,
                    src_width,
                    src_height,
                    &src_bitmap,
                    &bitmap_bytes,
                    Some(colorref_rgb(transparent_color)),
                );
            } else if let Some(framebuffer) = framebuffer {
                draw_bitmap_bytes_to_framebuffer(
                    kernel,
                    framebuffer,
                    dst,
                    dst_x,
                    dst_y,
                    dst_width,
                    dst_height,
                    src_x,
                    src_y,
                    src_width,
                    src_height,
                    &src_bitmap,
                    &bitmap_bytes,
                    Some(colorref_rgb(transparent_color)),
                );
            }
        }
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn stretch_dibits_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    framebuffer: Option<&mut dyn Framebuffer>,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let hdc = raw_arg(args, 0);
    let dst_x = raw_i32_arg(args, 1);
    let dst_y = raw_i32_arg(args, 2);
    let dst_width = raw_i32_arg(args, 3);
    let dst_height = raw_i32_arg(args, 4);
    let src_x = raw_i32_arg(args, 5);
    let src_y = raw_i32_arg(args, 6);
    let src_width = raw_i32_arg(args, 7);
    let src_height = raw_i32_arg(args, 8);
    let bits = raw_arg(args, 9);
    let info = raw_arg(args, 10);
    let usage = raw_arg(args, 11);
    let rop = raw_arg(args, 12);
    if hdc == 0
        || dst_width == 0
        || dst_height == 0
        || src_width == 0
        || src_height == 0
        || bits == 0
        || info == 0
        || usage != DIB_RGB_COLORS
    {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    if read_guest_u32(kernel, memory, thread_id, info).is_none() {
        return 0;
    }
    if rop == SRCCOPY
        && let Some(framebuffer) = framebuffer
    {
        draw_dib_to_framebuffer(
            kernel,
            memory,
            thread_id,
            framebuffer,
            hdc,
            dst_x,
            dst_y,
            dst_width,
            dst_height,
            src_x,
            src_y,
            src_width,
            src_height,
            bits,
            info,
        );
    }
    kernel.threads.set_last_error(thread_id, 0);
    src_height.unsigned_abs()
}

fn set_dibits_to_device_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    framebuffer: Option<&mut dyn Framebuffer>,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let hdc = raw_arg(args, 0);
    let dst_x = raw_i32_arg(args, 1);
    let dst_y = raw_i32_arg(args, 2);
    let dst_width = raw_i32_arg(args, 3);
    let dst_height = raw_i32_arg(args, 4);
    let src_x = raw_i32_arg(args, 5);
    let src_y = raw_i32_arg(args, 6);
    let start_scan = raw_i32_arg(args, 7);
    let lines = raw_arg(args, 8);
    let bits = raw_arg(args, 9);
    let info = raw_arg(args, 10);
    let usage = raw_arg(args, 11);
    if hdc == 0
        || dst_width == 0
        || dst_height == 0
        || lines == 0
        || bits == 0
        || info == 0
        || usage != DIB_RGB_COLORS
    {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    if read_guest_u32(kernel, memory, thread_id, info).is_none() {
        return 0;
    }
    if let Some(framebuffer) = framebuffer {
        draw_dib_to_framebuffer(
            kernel,
            memory,
            thread_id,
            framebuffer,
            hdc,
            dst_x,
            dst_y,
            dst_width,
            dst_height,
            src_x,
            src_y.saturating_add(start_scan),
            dst_width,
            lines as i32,
            bits,
            info,
        );
    }
    kernel.threads.set_last_error(thread_id, 0);
    lines
}

fn get_dib_color_table_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    hdc: u32,
    start: u32,
    entries: u32,
    colors_ptr: u32,
) -> u32 {
    if hdc == 0 || colors_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let Some(bitmap_handle) = kernel.resources.selected_bitmap(hdc) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    let Some(bitmap) = kernel.resources.bitmap(bitmap_handle) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    let Ok(start_index) = usize::try_from(start) else {
        kernel.threads.set_last_error(thread_id, 0);
        return 0;
    };
    if entries == 0 || start_index >= bitmap.color_table.len() {
        kernel.threads.set_last_error(thread_id, 0);
        return 0;
    }
    let copy_count = usize::try_from(entries)
        .ok()
        .unwrap_or(usize::MAX)
        .min(bitmap.color_table.len() - start_index);
    let bytes: Vec<u8> = bitmap.color_table[start_index..start_index + copy_count]
        .iter()
        .flat_map(|entry| entry.iter().copied())
        .collect();
    if !write_guest_bytes(kernel, memory, thread_id, colors_ptr, &bytes) {
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    copy_count as u32
}

fn set_dib_color_table_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    hdc: u32,
    start: u32,
    entries: u32,
    colors_ptr: u32,
) -> u32 {
    if hdc == 0 || colors_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let Some(bitmap_handle) = kernel.resources.selected_bitmap(hdc) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    if entries == 0 {
        kernel.threads.set_last_error(thread_id, 0);
        return 0;
    }
    let Some(byte_count) = entries.checked_mul(4) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let Some(bytes) = read_guest_bytes(kernel, memory, thread_id, colors_ptr, byte_count) else {
        return 0;
    };
    let Ok(start_index) = usize::try_from(start) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let copy_count = bytes.len() / 4;
    let Some(bitmap) = kernel.resources.bitmap_mut(bitmap_handle) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    let required_len = start_index.saturating_add(copy_count);
    if bitmap.color_table.len() < required_len {
        bitmap.color_table.resize(required_len, [0, 0, 0, 0]);
    }
    for (index, entry) in bytes.chunks_exact(4).enumerate() {
        bitmap.color_table[start_index + index] = [entry[0], entry[1], entry[2], entry[3]];
    }
    kernel.threads.set_last_error(thread_id, 0);
    copy_count as u32
}

fn set_bitmap_bits_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    bitmap: u32,
    byte_count: u32,
    bits_ptr: u32,
) -> u32 {
    if bitmap == 0 || bits_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let Some(dest_ptr) = kernel
        .resources
        .bitmap(bitmap)
        .map(|bitmap| bitmap.bits_ptr)
    else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    if dest_ptr != 0 {
        let Some(bytes) = read_guest_bytes(kernel, memory, thread_id, bits_ptr, byte_count) else {
            return 0;
        };
        if !write_guest_bytes(kernel, memory, thread_id, dest_ptr, &bytes) {
            return 0;
        }
    }
    kernel.threads.set_last_error(thread_id, 0);
    byte_count
}

fn get_pixel_raw(kernel: &mut CeKernel, thread_id: u32, hdc: u32) -> u32 {
    if hdc == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return CLR_INVALID;
    }
    kernel.threads.set_last_error(thread_id, 0);
    0
}

fn set_brush_org_ex_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let hdc = raw_arg(args, 0);
    let out_ptr = raw_arg(args, 3);
    if hdc == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    if out_ptr != 0 && !write_guest_point(kernel, memory, thread_id, out_ptr, Point { x: 0, y: 0 })
    {
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn polyline_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    framebuffer: Option<&mut dyn Framebuffer>,
    thread_id: u32,
    hdc: u32,
    points_ptr: u32,
    point_count: i32,
) -> bool {
    if hdc == 0 || points_ptr == 0 || point_count <= 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    let mut points = Vec::new();
    for index in 0..point_count as u32 {
        let Some(point) = read_guest_point(
            kernel,
            memory,
            thread_id,
            points_ptr.wrapping_add(index * 8),
        ) else {
            return false;
        };
        points.push(point);
    }
    if let Some(framebuffer) = framebuffer
        && let Some(color) = selected_pen_colorref(kernel, hdc)
    {
        draw_polyline_for_hdc(kernel, framebuffer, hdc, &points, color);
    }
    if let Some(last) = points.last().copied() {
        let _ = kernel.resources.move_to(hdc, last);
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn draw_polyline_for_hdc(
    kernel: &CeKernel,
    framebuffer: &mut dyn Framebuffer,
    hdc: u32,
    points: &[Point],
    colorref: u32,
) {
    let Some(hwnd) = hdc_to_hwnd(hdc) else {
        return;
    };
    let Some(client_origin) = kernel.gwe.client_to_screen(hwnd, Point { x: 0, y: 0 }) else {
        return;
    };
    let Some(client_rect) = kernel.gwe.get_client_rect(hwnd) else {
        return;
    };
    let clip = kernel
        .resources
        .clip_region(hdc)
        .and_then(|region| kernel.resources.region(region))
        .and_then(|region| intersect_rect_value(client_rect, region.rect))
        .unwrap_or(client_rect)
        .offset(client_origin.x, client_origin.y);
    let screen_points: Vec<Point> = points
        .iter()
        .map(|point| Point {
            x: point.x + client_origin.x,
            y: point.y + client_origin.y,
        })
        .collect();
    for segment in screen_points.windows(2) {
        draw_framebuffer_line(framebuffer, segment[0], segment[1], clip, colorref);
    }
}

fn draw_framebuffer_line(
    framebuffer: &mut dyn Framebuffer,
    start: Point,
    end: Point,
    clip: Rect,
    colorref: u32,
) {
    let info = framebuffer.info();
    let pixel = pixel_bytes_for_colorref(info.format, colorref);
    let bytes_per_pixel = info.format.bytes_per_pixel();
    let mut x0 = start.x;
    let mut y0 = start.y;
    let x1 = end.x;
    let y1 = end.y;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut dirty: Option<Rect> = None;

    loop {
        if x0 >= clip.left
            && x0 < clip.right
            && y0 >= clip.top
            && y0 < clip.bottom
            && x0 >= 0
            && y0 >= 0
            && x0 < info.width as i32
            && y0 < info.height as i32
        {
            let offset = y0 as usize * info.stride + x0 as usize * bytes_per_pixel;
            framebuffer.pixels_mut()[offset..offset + bytes_per_pixel]
                .copy_from_slice(&pixel[..bytes_per_pixel]);
            let pixel_rect = Rect {
                left: x0,
                top: y0,
                right: x0 + 1,
                bottom: y0 + 1,
            };
            dirty = Some(
                dirty
                    .map(|rect| union_rect_value(rect, pixel_rect))
                    .unwrap_or(pixel_rect),
            );
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }

    if let Some(rect) = dirty {
        framebuffer.mark_dirty(FramebufferRect::new(
            rect.left as u32,
            rect.top as u32,
            (rect.right - rect.left) as u32,
            (rect.bottom - rect.top) as u32,
        ));
    }
}

fn gdi_shape_raw(kernel: &mut CeKernel, thread_id: u32, args: &[u32]) -> bool {
    if raw_arg(args, 0) == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn move_to_ex_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let hdc = raw_arg(args, 0);
    let point = Point {
        x: raw_i32_arg(args, 1),
        y: raw_i32_arg(args, 2),
    };
    let out_ptr = raw_arg(args, 3);
    if hdc == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    let Some(previous) = kernel.resources.move_to(hdc, point) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return false;
    };
    if out_ptr != 0 && !write_guest_point(kernel, memory, thread_id, out_ptr, previous) {
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn line_to_raw(
    kernel: &mut CeKernel,
    framebuffer: Option<&mut dyn Framebuffer>,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let hdc = raw_arg(args, 0);
    let end = Point {
        x: raw_i32_arg(args, 1),
        y: raw_i32_arg(args, 2),
    };
    if hdc == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    let Some(start) = kernel.resources.current_pos(hdc) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return false;
    };
    if let Some(framebuffer) = framebuffer
        && let Some(color) = selected_pen_colorref(kernel, hdc)
    {
        draw_polyline_for_hdc(kernel, framebuffer, hdc, &[start, end], color);
    }
    let _ = kernel.resources.move_to(hdc, end);
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn draw_text_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    hdc: u32,
    text_ptr: u32,
    count: i32,
    rect_ptr: u32,
) -> u32 {
    if hdc == 0 || text_ptr == 0 || rect_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    if read_guest_rect(kernel, memory, thread_id, rect_ptr).is_none() {
        return 0;
    }
    let text_len = if count < 0 {
        read_guest_wide_arg(memory, text_ptr).map(|text| text.encode_utf16().count())
    } else {
        Some(count as usize)
    };
    let Some(text_len) = text_len else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    kernel.threads.set_last_error(thread_id, 0);
    text_len.max(1).min(i32::MAX as usize) as u32
}

fn ext_text_out_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let hdc = raw_arg(args, 0);
    let rect_ptr = raw_arg(args, 4);
    let text_ptr = raw_arg(args, 5);
    let count = raw_arg(args, 6);
    if hdc == 0 || (count != 0 && text_ptr == 0) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    if rect_ptr != 0 && read_guest_rect(kernel, memory, thread_id, rect_ptr).is_none() {
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn create_rect_rgn_raw(kernel: &mut CeKernel, left: i32, top: i32, right: i32, bottom: i32) -> u32 {
    kernel.resources.create_region(normalize_rect(Rect {
        left,
        top,
        right,
        bottom,
    }))
}

fn create_rect_rgn_indirect_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    rect_ptr: u32,
) -> u32 {
    let Some(rect) = read_guest_rect(kernel, memory, thread_id, rect_ptr) else {
        return 0;
    };
    kernel.resources.create_region(normalize_rect(rect))
}

fn set_rect_rgn_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    region: u32,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
) -> bool {
    if !kernel.resources.set_region(
        region,
        normalize_rect(Rect {
            left,
            top,
            right,
            bottom,
        }),
    ) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn combine_rgn_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    dest: u32,
    src1: u32,
    src2: u32,
    mode: u32,
) -> u32 {
    let Some(lhs) = kernel.resources.region(src1).map(|region| region.rect) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    };
    let rhs = if mode == RGN_COPY {
        Rect::default()
    } else {
        let Some(rhs) = kernel.resources.region(src2).map(|region| region.rect) else {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            return 0;
        };
        rhs
    };
    let combined = match mode {
        RGN_AND => intersect_rect_value(lhs, rhs).unwrap_or_default(),
        RGN_OR | RGN_XOR => union_rect_value(lhs, rhs),
        RGN_DIFF => lhs,
        RGN_COPY => lhs,
        _ => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_NOT_SUPPORTED);
            return ERROR_REGION;
        }
    };
    if !kernel.resources.set_region(dest, combined) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    region_status(combined)
}

fn get_rgn_box_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    region: u32,
    rect_ptr: u32,
) -> u32 {
    let Some(rect) = kernel.resources.region(region).map(|region| region.rect) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return ERROR_REGION;
    };
    if rect_ptr != 0 && !write_guest_rect(kernel, memory, thread_id, rect_ptr, rect) {
        return ERROR_REGION;
    }
    kernel.threads.set_last_error(thread_id, 0);
    region_status(rect)
}

fn select_clip_rgn_raw(kernel: &mut CeKernel, thread_id: u32, hdc: u32, region: u32) -> u32 {
    if hdc == 0 || (region != 0 && kernel.resources.region(region).is_none()) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    }
    kernel
        .resources
        .select_clip_region(hdc, (region != 0).then_some(region));
    kernel.threads.set_last_error(thread_id, 0);
    if region == 0 {
        SIMPLEREGION
    } else {
        region_status(kernel.resources.region(region).unwrap().rect)
    }
}

fn get_clip_box_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    hdc: u32,
    rect_ptr: u32,
) -> u32 {
    let rect = if let Some(region) = kernel
        .resources
        .clip_region(hdc)
        .and_then(|region| kernel.resources.region(region))
    {
        region.rect
    } else {
        hdc_to_hwnd(hdc)
            .and_then(|hwnd| kernel.gwe.get_client_rect(hwnd))
            .unwrap_or_else(|| Rect::from_origin_size(0, 0, 800, 480))
    };
    if rect_ptr != 0 && !write_guest_rect(kernel, memory, thread_id, rect_ptr, rect) {
        return 0;
    }
    region_status(rect)
}

fn pt_in_region_raw(kernel: &CeKernel, region: u32, x: i32, y: i32) -> bool {
    kernel
        .resources
        .region(region)
        .is_some_and(|region| point_in_rect(region.rect, x, y))
}

fn rect_in_region_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    region: u32,
    rect_ptr: u32,
) -> bool {
    let Some(rect) = read_guest_rect(kernel, memory, thread_id, rect_ptr) else {
        return false;
    };
    kernel
        .resources
        .region(region)
        .is_some_and(|region| rects_intersect(region.rect, rect))
}

fn set_window_rgn_raw(kernel: &mut CeKernel, thread_id: u32, hwnd: u32, region: u32) -> u32 {
    let rect = if region == 0 {
        None
    } else {
        let Some(region) = kernel.resources.region(region) else {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            return 0;
        };
        Some(region.rect)
    };
    if !kernel.gwe.set_window_region(hwnd, rect) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    1
}

fn get_window_rgn_raw(kernel: &mut CeKernel, thread_id: u32, hwnd: u32, region: u32) -> u32 {
    if region == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return ERROR_REGION;
    }
    let Some(rect) = kernel.gwe.window_region(hwnd) else {
        if kernel.gwe.is_window(hwnd) {
            kernel.threads.set_last_error(thread_id, 0);
            return NULLREGION;
        }
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return ERROR_REGION;
    };
    if !kernel.resources.set_region(region, rect) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return ERROR_REGION;
    }
    kernel.threads.set_last_error(thread_id, 0);
    region_status(rect)
}

fn paint_hdc_for_hwnd(hwnd: u32) -> u32 {
    0x0200_0000 | (hwnd & 0x00ff_ffff)
}

fn hdc_to_hwnd(hdc: u32) -> Option<u32> {
    (hdc & 0xff00_0000 == 0x0200_0000).then_some(hdc & 0x00ff_ffff)
}

fn normalize_rect(rect: Rect) -> Rect {
    Rect {
        left: rect.left.min(rect.right),
        top: rect.top.min(rect.bottom),
        right: rect.left.max(rect.right),
        bottom: rect.top.max(rect.bottom),
    }
}

fn union_rect_value(lhs: Rect, rhs: Rect) -> Rect {
    match (is_rect_empty_value(lhs), is_rect_empty_value(rhs)) {
        (true, true) => Rect::default(),
        (true, false) => rhs,
        (false, true) => lhs,
        (false, false) => Rect {
            left: lhs.left.min(rhs.left),
            top: lhs.top.min(rhs.top),
            right: lhs.right.max(rhs.right),
            bottom: lhs.bottom.max(rhs.bottom),
        },
    }
}

fn intersect_rect_value(lhs: Rect, rhs: Rect) -> Option<Rect> {
    let rect = Rect {
        left: lhs.left.max(rhs.left),
        top: lhs.top.max(rhs.top),
        right: lhs.right.min(rhs.right),
        bottom: lhs.bottom.min(rhs.bottom),
    };
    (!is_rect_empty_value(rect)).then_some(rect)
}

fn region_status(rect: Rect) -> u32 {
    if is_rect_empty_value(rect) {
        NULLREGION
    } else {
        SIMPLEREGION
    }
}

fn point_in_rect(rect: Rect, x: i32, y: i32) -> bool {
    x >= rect.left && x < rect.right && y >= rect.top && y < rect.bottom
}

fn rects_intersect(lhs: Rect, rhs: Rect) -> bool {
    !is_rect_empty_value(lhs)
        && !is_rect_empty_value(rhs)
        && lhs.left < rhs.right
        && rhs.left < lhs.right
        && lhs.top < rhs.bottom
        && rhs.top < lhs.bottom
}

fn write_paint_struct<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
    hdc: u32,
    update: crate::ce::gwe::PaintUpdate,
) -> bool {
    write_guest_u32(kernel, memory, thread_id, addr, hdc)
        && write_guest_u32(
            kernel,
            memory,
            thread_id,
            addr.wrapping_add(4),
            u32::from(update.erase),
        )
        && write_guest_rect(kernel, memory, thread_id, addr.wrapping_add(8), update.rect)
        && write_guest_u32(kernel, memory, thread_id, addr.wrapping_add(24), 0)
        && write_guest_u32(kernel, memory, thread_id, addr.wrapping_add(28), 0)
        && (0..32).all(|offset| {
            write_guest_u8(kernel, memory, thread_id, addr.wrapping_add(32 + offset), 0)
        })
}

fn get_message_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let msg_ptr = raw_arg(args, 0);
    let hwnd = (raw_arg(args, 1) != 0).then_some(raw_arg(args, 1));
    let min_msg = raw_arg(args, 2);
    let max_msg = raw_arg(args, 3);
    kernel.pump_timers_to_gwe(thread_id);
    kernel.drain_remote_input_to_thread_window(thread_id, hwnd);
    let Some(message) = kernel
        .gwe
        .get_message_filtered(thread_id, hwnd, min_msg, max_msg)
    else {
        return false;
    };
    if !write_guest_message(kernel, memory, thread_id, msg_ptr, &message) {
        return false;
    }
    message.msg != crate::ce::gwe::WM_QUIT
}

fn peek_message_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> bool {
    let msg_ptr = raw_arg(args, 0);
    let hwnd = (raw_arg(args, 1) != 0).then_some(raw_arg(args, 1));
    let min_msg = raw_arg(args, 2);
    let max_msg = raw_arg(args, 3);
    let flags = PeekFlags::from_bits_truncate(raw_arg(args, 4));
    kernel.pump_timers_to_gwe(thread_id);
    kernel.drain_remote_input_to_thread_window(thread_id, hwnd);
    let Some(message) = kernel
        .gwe
        .peek_message_filtered(thread_id, hwnd, min_msg, max_msg, flags)
    else {
        return false;
    };
    write_guest_message(kernel, memory, thread_id, msg_ptr, &message)
}

fn dispatch_message_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    msg_ptr: u32,
) -> u32 {
    let Some(message) = read_guest_message(kernel, memory, thread_id, msg_ptr) else {
        return 0;
    };
    let (class_name, wndproc) = kernel
        .gwe
        .window(message.hwnd)
        .map(|window| (window.class_name.clone(), window.wndproc))
        .unwrap_or_default();
    tracing::debug!(
        target: "ce.gwe",
        thread_id,
        msg_ptr = format_args!("0x{msg_ptr:08x}"),
        hwnd = format_args!("0x{:08x}", message.hwnd),
        msg = format_args!("0x{:08x}", message.msg),
        wparam = format_args!("0x{:08x}", message.wparam),
        lparam = format_args!("0x{:08x}", message.lparam),
        class = class_name.as_str(),
        wndproc = format_args!("0x{wndproc:08x}"),
        "DispatchMessageW"
    );
    kernel.dispatch_message_w_for_thread(thread_id, message)
}

fn send_message_timeout_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let hwnd = raw_arg(args, 0);
    let msg = raw_arg(args, 1);
    let wparam = raw_arg(args, 2);
    let lparam = raw_arg(args, 3);
    let timeout_ms = raw_arg(args, 5);
    let result_ptr = raw_arg(args, 6);
    let Some(target_thread) = kernel
        .gwe
        .window(hwnd)
        .filter(|window| !window.destroyed)
        .map(|window| window.thread_id)
    else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    };
    if target_thread != thread_id && timeout_ms == 0 {
        let Some(send_id) =
            kernel.begin_cross_thread_send_message_w(thread_id, hwnd, msg, wparam, lparam, Some(0))
        else {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
            return 0;
        };
        let expired = kernel.expire_timed_out_send_messages();
        if expired.contains(&send_id) {
            let _ = kernel.take_completed_send_message_result(send_id);
        }
        kernel.threads.set_last_error(thread_id, 0);
        return 0;
    }
    let Some(result) = kernel.send_message_w(hwnd, msg, wparam, lparam) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    };
    if result_ptr != 0 && !write_guest_u32(kernel, memory, thread_id, result_ptr, result) {
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    result
}

fn translate_message_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    msg_ptr: u32,
) -> bool {
    let Some(message) = read_guest_message(kernel, memory, thread_id, msg_ptr) else {
        return false;
    };
    if message.msg != crate::ce::gwe::WM_KEYDOWN || message.hwnd == 0 {
        kernel.threads.set_last_error(thread_id, 0);
        return false;
    }
    let char_code = translate_virtual_key_to_char(message.wparam);
    if char_code == 0 {
        kernel.threads.set_last_error(thread_id, 0);
        return false;
    }
    let posted = kernel.post_message_w_for_thread(
        thread_id,
        message.hwnd,
        crate::ce::gwe::WM_CHAR,
        char_code,
        message.lparam,
    );
    kernel.threads.set_last_error(thread_id, 0);
    posted
}

fn translate_virtual_key_to_char(vkey: u32) -> u32 {
    match vkey {
        0x30..=0x39 | 0x41..=0x5a | 0x61..=0x7a => vkey,
        0x20 => 0x20,
        _ => 0,
    }
}

const CS_LOCK_COUNT: u32 = 0;
const CS_OWNER_THREAD: u32 = 4;
const CS_HANDLE: u32 = 8;
const CS_NEED_TRAP: u32 = 12;
const CS_CONTENTIONS: u32 = 16;
const CS_SIZE_WORDS: u32 = 5;

fn initialize_critical_section<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
) {
    if addr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return;
    }
    for offset in [
        CS_LOCK_COUNT,
        CS_OWNER_THREAD,
        CS_HANDLE,
        CS_NEED_TRAP,
        CS_CONTENTIONS,
    ] {
        if !write_guest_u32(kernel, memory, thread_id, addr.wrapping_add(offset), 0) {
            return;
        }
    }
    let handle = kernel
        .handles
        .insert(KernelObject::CriticalSection(CriticalSectionObject {
            guest_ptr: addr,
        }));
    write_guest_u32(
        kernel,
        memory,
        thread_id,
        addr.wrapping_add(CS_HANDLE),
        handle,
    );
}

fn delete_critical_section<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
) {
    if let Some(handle) = read_guest_u32(kernel, memory, thread_id, addr.wrapping_add(CS_HANDLE)) {
        let _ = kernel.close_handle(handle);
    }
    for word in 0..CS_SIZE_WORDS {
        write_guest_u32(kernel, memory, thread_id, addr.wrapping_add(word * 4), 0);
    }
}

fn enter_critical_section<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
    try_only: bool,
) -> bool {
    let owner_addr = addr.wrapping_add(CS_OWNER_THREAD);
    let lock_addr = addr.wrapping_add(CS_LOCK_COUNT);
    let contentions_addr = addr.wrapping_add(CS_CONTENTIONS);
    let Some(owner) = read_guest_u32(kernel, memory, thread_id, owner_addr) else {
        return false;
    };

    if owner == thread_id {
        let Some(lock_count) = read_guest_u32(kernel, memory, thread_id, lock_addr) else {
            return false;
        };
        return write_guest_u32(
            kernel,
            memory,
            thread_id,
            lock_addr,
            lock_count.wrapping_add(1),
        );
    }

    if owner == 0 {
        return write_guest_u32(kernel, memory, thread_id, owner_addr, thread_id)
            && write_guest_u32(kernel, memory, thread_id, lock_addr, 1);
    }

    if !try_only {
        interlocked_update(
            kernel,
            memory,
            thread_id,
            contentions_addr,
            |value| value.wrapping_add(1),
            InterlockedReturn::NewValue,
        );
    }
    false
}

fn leave_critical_section<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
) {
    let owner_addr = addr.wrapping_add(CS_OWNER_THREAD);
    let lock_addr = addr.wrapping_add(CS_LOCK_COUNT);
    let Some(owner) = read_guest_u32(kernel, memory, thread_id, owner_addr) else {
        return;
    };
    if owner != thread_id {
        return;
    }
    let Some(lock_count) = read_guest_u32(kernel, memory, thread_id, lock_addr) else {
        return;
    };
    if lock_count > 1 {
        write_guest_u32(
            kernel,
            memory,
            thread_id,
            lock_addr,
            lock_count.wrapping_sub(1),
        );
    } else {
        write_guest_u32(kernel, memory, thread_id, owner_addr, thread_id | 1);
        interlocked_compare_store(kernel, memory, thread_id, owner_addr, 0, thread_id | 1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InterlockedReturn {
    OldValue,
    NewValue,
}

fn interlocked_update<M, F>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
    update: F,
    return_mode: InterlockedReturn,
) -> u32
where
    M: CoredllGuestMemory,
    F: FnOnce(u32) -> u32,
{
    let Some(old_value) = read_guest_u32(kernel, memory, thread_id, addr) else {
        return 0;
    };
    let new_value = update(old_value);
    if !write_guest_u32(kernel, memory, thread_id, addr, new_value) {
        return 0;
    }
    match return_mode {
        InterlockedReturn::OldValue => old_value,
        InterlockedReturn::NewValue => new_value,
    }
}

fn interlocked_compare_store<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
    exchange: u32,
    comparand: u32,
) -> u32 {
    let Some(old_value) = read_guest_u32(kernel, memory, thread_id, addr) else {
        return 0;
    };
    if old_value == comparand {
        write_guest_u32(kernel, memory, thread_id, addr, exchange);
    }
    old_value
}

pub(crate) fn read_guest_bytes<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    addr: u32,
    len: u32,
) -> Option<Vec<u8>> {
    let mut bytes = vec![0; len as usize];
    if memory.read_bytes(addr, &mut bytes).is_err() {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return None;
    }
    Some(bytes)
}

fn read_guest_narrow_preview<M: CoredllGuestMemory>(
    memory: &M,
    ptr: u32,
    max_len: usize,
) -> Option<String> {
    if ptr == 0 {
        return None;
    }
    let mut preview = String::new();
    for offset in 0..max_len {
        let byte = memory.read_u8(ptr.wrapping_add(offset as u32)).ok()?;
        if byte == 0 {
            break;
        }
        match byte {
            b'\r' => preview.push_str("\\r"),
            b'\n' => preview.push_str("\\n"),
            b'\t' => preview.push_str("\\t"),
            0x20..=0x7e => preview.push(byte as char),
            _ => preview.push_str(&format!("\\x{byte:02x}")),
        }
    }
    Some(preview)
}

pub(crate) fn write_guest_bytes<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
    bytes: &[u8],
) -> bool {
    if memory.write_bytes(addr, bytes).is_err() {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    true
}

fn write_optional_count<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
    value: u32,
) -> bool {
    addr == 0 || write_guest_u32(kernel, memory, thread_id, addr, value)
}

fn write_optional_u32<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
    value: u32,
) -> bool {
    write_optional_count(kernel, memory, thread_id, addr, value)
}

fn read_guest_u32<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    addr: u32,
) -> Option<u32> {
    match memory.read_u32(addr) {
        Ok(value) => Some(value),
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            None
        }
    }
}

fn read_guest_u16<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    addr: u32,
) -> Option<u16> {
    match memory.read_u16(addr) {
        Ok(value) => Some(value),
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            None
        }
    }
}

fn read_guest_i16<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    addr: u32,
) -> Option<i16> {
    read_guest_u16(kernel, memory, thread_id, addr).map(|value| value as i16)
}

fn write_guest_u32<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
    value: u32,
) -> bool {
    match memory.write_u32(addr, value) {
        Ok(()) => true,
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            false
        }
    }
}

fn write_guest_u8<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
    value: u8,
) -> bool {
    match memory.write_u8(addr, value) {
        Ok(()) => true,
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            false
        }
    }
}

pub(crate) fn write_guest_u16<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
    value: u16,
) -> bool {
    match memory.write_u16(addr, value) {
        Ok(()) => true,
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            false
        }
    }
}

fn read_guest_wide_z<M: CoredllGuestMemory>(
    memory: &M,
    addr: u32,
    max_chars: usize,
) -> Result<String> {
    let mut units = Vec::new();
    for index in 0..max_chars {
        let unit = memory.read_u16(addr.wrapping_add((index as u32) * 2))?;
        if unit == 0 {
            break;
        }
        units.push(unit);
    }
    Ok(String::from_utf16_lossy(&units))
}

fn read_guest_i32<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    addr: u32,
) -> Option<i32> {
    read_guest_u32(kernel, memory, thread_id, addr).map(|value| value as i32)
}

fn write_guest_i32<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
    value: i32,
) -> bool {
    write_guest_u32(kernel, memory, thread_id, addr, value as u32)
}

fn read_guest_point<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    addr: u32,
) -> Option<Point> {
    Some(Point {
        x: read_guest_i32(kernel, memory, thread_id, addr)?,
        y: read_guest_i32(kernel, memory, thread_id, addr.wrapping_add(4))?,
    })
}

fn write_guest_point<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
    point: Point,
) -> bool {
    write_guest_i32(kernel, memory, thread_id, addr, point.x)
        && write_guest_i32(kernel, memory, thread_id, addr.wrapping_add(4), point.y)
}

fn read_guest_rect<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    addr: u32,
) -> Option<Rect> {
    Some(Rect {
        left: read_guest_i32(kernel, memory, thread_id, addr)?,
        top: read_guest_i32(kernel, memory, thread_id, addr.wrapping_add(4))?,
        right: read_guest_i32(kernel, memory, thread_id, addr.wrapping_add(8))?,
        bottom: read_guest_i32(kernel, memory, thread_id, addr.wrapping_add(12))?,
    })
}

fn write_guest_rect<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
    rect: Rect,
) -> bool {
    write_guest_i32(kernel, memory, thread_id, addr, rect.left)
        && write_guest_i32(kernel, memory, thread_id, addr.wrapping_add(4), rect.top)
        && write_guest_i32(kernel, memory, thread_id, addr.wrapping_add(8), rect.right)
        && write_guest_i32(
            kernel,
            memory,
            thread_id,
            addr.wrapping_add(12),
            rect.bottom,
        )
}

fn read_guest_message<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    addr: u32,
) -> Option<Message> {
    Some(Message {
        hwnd: read_guest_u32(kernel, memory, thread_id, addr)?,
        msg: read_guest_u32(kernel, memory, thread_id, addr.wrapping_add(4))?,
        wparam: read_guest_u32(kernel, memory, thread_id, addr.wrapping_add(8))?,
        lparam: read_guest_u32(kernel, memory, thread_id, addr.wrapping_add(12))?,
        time_ms: read_guest_u32(kernel, memory, thread_id, addr.wrapping_add(16))?,
        source: crate::ce::gwe::MSGSRC_UNKNOWN,
        mouse_pos_at_post: None,
    })
}

fn write_guest_message<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
    message: &Message,
) -> bool {
    if !write_message_pointer_payload(kernel, memory, thread_id, message) {
        return false;
    }
    write_guest_u32(kernel, memory, thread_id, addr, message.hwnd)
        && write_guest_u32(kernel, memory, thread_id, addr.wrapping_add(4), message.msg)
        && write_guest_u32(
            kernel,
            memory,
            thread_id,
            addr.wrapping_add(8),
            message.wparam,
        )
        && write_guest_u32(
            kernel,
            memory,
            thread_id,
            addr.wrapping_add(12),
            message.lparam,
        )
        && write_guest_u32(
            kernel,
            memory,
            thread_id,
            addr.wrapping_add(16),
            message.time_ms,
        )
        && write_guest_i32(kernel, memory, thread_id, addr.wrapping_add(20), 0)
        && write_guest_i32(kernel, memory, thread_id, addr.wrapping_add(24), 0)
}

fn write_message_pointer_payload<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    message: &Message,
) -> bool {
    if message.lparam == 0 {
        return true;
    }
    match kernel.message_pointer_payload(message.lparam) {
        Some(MessagePointerPayload::WindowPos(window_pos))
            if message.msg == crate::ce::gwe::WM_WINDOWPOSCHANGED =>
        {
            write_guest_window_pos(kernel, memory, thread_id, message.lparam, window_pos)
        }
        Some(_) | None => true,
    }
}

fn write_guest_window_pos<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
    window_pos: WindowPos,
) -> bool {
    write_guest_u32(kernel, memory, thread_id, addr, window_pos.hwnd)
        && write_guest_u32(
            kernel,
            memory,
            thread_id,
            addr.wrapping_add(4),
            window_pos.insert_after,
        )
        && write_guest_i32(
            kernel,
            memory,
            thread_id,
            addr.wrapping_add(8),
            window_pos.x,
        )
        && write_guest_i32(
            kernel,
            memory,
            thread_id,
            addr.wrapping_add(12),
            window_pos.y,
        )
        && write_guest_i32(
            kernel,
            memory,
            thread_id,
            addr.wrapping_add(16),
            window_pos.width,
        )
        && write_guest_i32(
            kernel,
            memory,
            thread_id,
            addr.wrapping_add(20),
            window_pos.height,
        )
        && write_guest_u32(
            kernel,
            memory,
            thread_id,
            addr.wrapping_add(24),
            window_pos.flags,
        )
}

fn get_sys_color(index: u32) -> u32 {
    match sys_color_index(index) {
        0 => rgb(0xc0, 0xc0, 0xc0),
        1 => rgb(0x00, 0x80, 0x80),
        2 | 27 => rgb(0x00, 0x00, 0x80),
        3 | 28 => rgb(0x80, 0x80, 0x80),
        4 | 15 | 25 => rgb(0xc0, 0xc0, 0xc0),
        5 => rgb(0xff, 0xff, 0xff),
        6 => rgb(0x00, 0x00, 0x00),
        7 | 8 | 9 | 18 | 23 | 26 => rgb(0x00, 0x00, 0x00),
        10 | 11 | 12 | 16 | 17 | 19 | 21 => rgb(0x80, 0x80, 0x80),
        13 => rgb(0x00, 0x00, 0x80),
        14 => rgb(0xff, 0xff, 0xff),
        20 | 22 => rgb(0xff, 0xff, 0xff),
        24 => rgb(0xff, 0xff, 0xe1),
        _ => rgb(0x00, 0x00, 0x00),
    }
}

fn get_sys_color_brush(index: u32) -> u32 {
    0x000b_4000 | sys_color_index(index)
}

fn sys_color_index(index: u32) -> u32 {
    index & !SYS_COLOR_INDEX_FLAG
}

const fn rgb(red: u32, green: u32, blue: u32) -> u32 {
    red | (green << 8) | (blue << 16)
}

fn raw_arg(args: &[u32], index: usize) -> u32 {
    args.get(index).copied().unwrap_or(0)
}

fn raw_i32_arg(args: &[u32], index: usize) -> i32 {
    raw_arg(args, index) as i32
}

fn raw_f32_arg(args: &[u32], index: usize) -> f32 {
    f32::from_bits(raw_arg(args, index))
}

fn raw_u64_pair(args: &[u32], low_index: usize, high_index: usize) -> u64 {
    u64::from(raw_arg(args, low_index)) | (u64::from(raw_arg(args, high_index)) << 32)
}

fn raw_i64_pair(args: &[u32], low_index: usize, high_index: usize) -> i64 {
    raw_u64_pair(args, low_index, high_index) as i64
}

fn raw_f64_pair(args: &[u32], low_index: usize, high_index: usize) -> f64 {
    f64::from_bits(raw_u64_pair(args, low_index, high_index))
}

fn security_gen_cookie2_raw(kernel: &CeKernel, thread_id: u32) -> u32 {
    let mut cookie = 0xa5a5_5a5a_u32
        ^ kernel.current_process_id().rotate_left(5)
        ^ thread_id.rotate_left(13)
        ^ kernel.timers.tick_count().rotate_left(17);
    cookie ^= cookie >> 16;
    cookie = cookie.wrapping_mul(0x045d_9f3b);
    if cookie == 0 || cookie == 0xbb40_e64e {
        cookie ^= 0x4711_2010;
    }
    if cookie == 0 { 0x4711_2010 } else { cookie }
}

fn raw_unary_f64(kernel: &CeKernel, args: &[u32], op: CeMathUnaryF64) -> CoredllValue {
    CoredllValue::CeMath(kernel.math.eval(CeMathCall::UnaryF64 {
        op,
        value: raw_f64_pair(args, 0, 1),
    }))
}

fn raw_binary_f64(kernel: &CeKernel, args: &[u32], op: CeMathBinaryF64) -> CoredllValue {
    CoredllValue::CeMath(kernel.math.eval(CeMathCall::BinaryF64 {
        op,
        lhs: raw_f64_pair(args, 0, 1),
        rhs: raw_f64_pair(args, 2, 3),
    }))
}

fn raw_binary_f32(kernel: &CeKernel, args: &[u32], op: CeMathBinaryF32) -> CoredllValue {
    CoredllValue::CeMath(kernel.math.eval(CeMathCall::BinaryF32 {
        op,
        lhs: raw_f32_arg(args, 0),
        rhs: raw_f32_arg(args, 1),
    }))
}

fn normalize_name(name: &str) -> String {
    name.to_ascii_lowercase()
}

fn has_any_prefix(name: &str, prefixes: &[&str]) -> bool {
    prefixes
        .iter()
        .any(|prefix| name.starts_with(&normalize_name(prefix)))
}

fn is_sdk_crt_export(export: &CoredllExport) -> bool {
    SDK_ORDINALS
        .iter()
        .any(|sdk| sdk.name == export.name && sdk.ordinal == export.ordinal)
}

const EVENT_PULSE: u32 = 1;
const EVENT_RESET: u32 = 2;
const EVENT_SET: u32 = 3;
const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
const HORZRES: u32 = 8;
const VERTRES: u32 = 10;
const BITSPIXEL: u32 = 12;
const PLANES: u32 = 14;
const LOGPIXELSX: u32 = 88;
const LOGPIXELSY: u32 = 90;
const IMAGE_BITMAP: u32 = 0;
const IMAGE_ICON: u32 = 1;
const IMAGE_CURSOR: u32 = 2;
const BI_RGB: u32 = 0;
const BI_BITFIELDS: u32 = 3;
const CLR_INVALID: u32 = 0xffff_ffff;
const DIB_RGB_COLORS: u32 = 0;
const SRCCOPY: u32 = 0x00cc_0020;
const PALETTE_ENTRY_SIZE: usize = 4;
const PALETTE_ENTRY_SIZE_U32: u32 = 4;
const MAX_LOG_PALETTE_ENTRIES: u32 = 4096;
const DS_SETFONT: u32 = 0x0000_0040;
const LR_LOADFROMFILE: u32 = 0x0000_0010;
const MIIM_STATE: u32 = 0x0000_0001;
const MIIM_ID: u32 = 0x0000_0002;
const MIIM_SUBMENU: u32 = 0x0000_0004;
const MIIM_CHECKMARKS: u32 = 0x0000_0008;
const MIIM_TYPE: u32 = 0x0000_0010;
const MIIM_DATA: u32 = 0x0000_0020;
const MIIM_STRING: u32 = 0x0000_0040;
const MENUITEMINFOW_SIZE: u32 = 44;
const MAX_CONVERSION_CHARS: u32 = 0x1_0000;
const OSVERSIONINFO_CSD_WCHARS: u32 = 128;
const ERROR_REGION: u32 = 0;
const NULLREGION: u32 = 1;
const SIMPLEREGION: u32 = 2;
const RGN_AND: u32 = 1;
const RGN_OR: u32 = 2;
const RGN_XOR: u32 = 3;
const RGN_DIFF: u32 = 4;
const RGN_COPY: u32 = 5;
const RT_ACCELERATOR: u32 = 9;
const RT_BITMAP: u32 = 2;
const RT_GROUP_CURSOR: u32 = 12;
const RT_GROUP_ICON: u32 = 14;
const RT_MENU: u32 = 4;
const SYS_COLOR_INDEX_FLAG: u32 = 0x4000_0000;
const VER_PLATFORM_WIN32_CE: u32 = 3;
const WIN32_FIND_DATAW_FILE_NAME: u32 = 40;
const WIN32_FIND_DATAW_FILE_NAME_CHARS: usize = 260;

const IMPLEMENTED_EXPORTS: &[&str] = &[
    "InitializeCriticalSection",
    "DeleteCriticalSection",
    "EnterCriticalSection",
    "LeaveCriticalSection",
    "TryEnterCriticalSection",
    "InterlockedTestExchange",
    "InterlockedIncrement",
    "InterlockedDecrement",
    "InterlockedExchange",
    "InterlockedExchangeAdd",
    "InterlockedCompareExchange",
    "TlsGetValue",
    "TlsSetValue",
    "GetLastError",
    "SetLastError",
    "FindResource",
    "FindResourceW",
    "LoadResource",
    "LoadStringW",
    "SizeofResource",
    "DestroyIcon",
    "LocalAlloc",
    "LocalAllocTrace",
    "LocalReAlloc",
    "LocalSize",
    "LocalFree",
    "RemoteLocalAlloc",
    "RemoteLocalReAlloc",
    "RemoteLocalSize",
    "RemoteLocalFree",
    "LocalAllocInProcess",
    "LocalFreeInProcess",
    "LocalSizeInProcess",
    "GetProcessHeap",
    "GetModuleFileNameW",
    "GetVersionEx",
    "GetVersionExW",
    "GetSystemMetrics",
    "SHGetSpecialFolderPath",
    "SystemParametersInfoW",
    "CopyRect",
    "EqualRect",
    "InflateRect",
    "IntersectRect",
    "IsRectEmpty",
    "OffsetRect",
    "PtInRect",
    "SetRect",
    "SetRectEmpty",
    "UnionRect",
    "HeapCreate",
    "HeapDestroy",
    "HeapAlloc",
    "HeapAllocTrace",
    "HeapReAlloc",
    "HeapSize",
    "HeapFree",
    "HeapValidate",
    "IsBadReadPtr",
    "IsBadWritePtr",
    "malloc",
    "wcsrchr",
    "_wcsdup",
    "iswctype",
    "wcsncmp",
    "MultiByteToWideChar",
    "WideCharToMultiByte",
    "StringCchCatW",
    "StringCbCatW",
    "CharLowerW",
    "CharLowerBuffW",
    "CharUpperBuffW",
    "CharUpperW",
    "memcpy",
    "memset",
    "??2@YAPAXI@Z",
    "wsprintfW",
    "wvsprintfW",
    "swprintf",
    "vswprintf",
    "printf",
    "__security_gen_cookie2",
    "free",
    "RemoteHeapAlloc",
    "RemoteHeapReAlloc",
    "RemoteHeapSize",
    "RemoteHeapFree",
    "VirtualAlloc",
    "VirtualFree",
    "FlushInstructionCache",
    "RegCloseKey",
    "RegCreateKeyExW",
    "RegDeleteKeyW",
    "RegDeleteValueW",
    "RegEnumValueW",
    "RegOpenKeyExW",
    "RegQueryInfoKeyW",
    "RegQueryValueExW",
    "RegSetValueExW",
    "CreateFileW",
    "CopyFileW",
    "FindFirstFileW",
    "FindClose",
    "ReadFile",
    "WriteFile",
    "SetFilePointer",
    "GetFileSize",
    "FlushFileBuffers",
    "GetStoreInformation",
    "DeviceIoControl",
    "KernelIoControl",
    "ClearCommBreak",
    "ClearCommError",
    "EscapeCommFunction",
    "GetCommMask",
    "GetCommModemStatus",
    "GetCommState",
    "GetCommTimeouts",
    "PurgeComm",
    "SetCommBreak",
    "SetCommMask",
    "SetCommState",
    "SetCommTimeouts",
    "SetupComm",
    "WaitCommEvent",
    "CloseHandle",
    "CreateEventW",
    "OpenEventW",
    "EventModify",
    "WaitForSingleObject",
    "WaitForMultipleObjects",
    "CreateSemaphoreW",
    "ReleaseSemaphore",
    "CreateThread",
    "SuspendThread",
    "ResumeThread",
    "GetThreadId",
    "GetExitCodeThread",
    "GetExitCodeProcess",
    "GetProcessVersion",
    "GetProcessId",
    "GetThreadTimes",
    "TerminateProcess",
    "CreateMutexW",
    "ReleaseMutex",
    "CreateProcessW",
    "CreateWindowExW",
    "DestroyWindow",
    "ShowWindow",
    "UpdateWindow",
    "GetParent",
    "SetParent",
    "IsWindow",
    "GetWindowTextLengthW",
    "GetClassNameW",
    "EnableWindow",
    "IsWindowEnabled",
    "GetDesktopWindow",
    "GetWindowTextWDirect",
    "SetFocus",
    "GetFocus",
    "SetCapture",
    "GetCapture",
    "ReleaseCapture",
    "IsWindowVisible",
    "SetWindowPos",
    "MoveWindow",
    "GetDlgItem",
    "GetDlgCtrlID",
    "CreateDialogIndirectParamW",
    "DialogBoxIndirectParamW",
    "EndDialog",
    "DefDlgProcW",
    "SetDlgItemTextW",
    "GetDlgItemTextW",
    "CheckRadioButton",
    "IsDialogMessageW",
    "GetWindowRect",
    "GetClientRect",
    "ClientToScreen",
    "ScreenToClient",
    "MapWindowPoints",
    "SetWindowTextW",
    "GetWindowTextW",
    "GetWindowLongW",
    "SetWindowLongW",
    "GetDC",
    "ReleaseDC",
    "GetDeviceCaps",
    "GetMessageW",
    "PeekMessageW",
    "PostMessageW",
    "PostThreadMessageW",
    "MsgWaitForMultipleObjectsEx",
    "PostQuitMessage",
    "SendMessageW",
    "SendNotifyMessageW",
    "SendMessageTimeout",
    "InSendMessage",
    "GetMessageSource",
    "GetQueueStatus",
    "DispatchMessageW",
    "TranslateMessage",
    "DefWindowProcW",
    "SetTimer",
    "KillTimer",
    "RegisterGesture",
    "GetSystemTime",
    "GetLocalTime",
    "GetSystemTimeAsFileTime",
    "FileTimeToSystemTime",
    "GetTickCount",
    "QueryPerformanceCounter",
    "QueryPerformanceFrequency",
    "Sleep",
    "waveOutGetNumDevs",
    "waveOutOpen",
    "waveOutWrite",
    "waveOutPause",
    "waveOutRestart",
    "waveOutReset",
    "waveOutClose",
    "waveOutGetVolume",
    "waveOutSetVolume",
    "LoadImageW",
    "LoadBitmapW",
    "GetObjectW",
    "DeleteObject",
    "CreateRectRgn",
    "CreateRectRgnIndirect",
    "SetRectRgn",
    "CombineRgn",
    "SelectClipRgn",
    "GetClipBox",
    "GetRgnBox",
    "PtInRegion",
    "RectInRegion",
    "SetWindowRgn",
    "GetWindowRgn",
    "CreateCompatibleDC",
    "DeleteDC",
    "CreateCompatibleBitmap",
    "CreateDIBSection",
    "CreateFontIndirectW",
    "CreateSolidBrush",
    "CreatePatternBrush",
    "CreatePen",
    "CreatePalette",
    "GetNearestPaletteIndex",
    "GetPaletteEntries",
    "GetSystemPaletteEntries",
    "SetPaletteEntries",
    "RealizePalette",
    "SelectPalette",
    "SelectObject",
    "SetBkMode",
    "SetTextColor",
    "SetTextAlign",
    "FillRect",
    "BitBlt",
    "StretchBlt",
    "StretchDIBits",
    "SetDIBitsToDevice",
    "SetDIBColorTable",
    "SetBitmapBits",
    "GetPixel",
    "PatBlt",
    "TransparentImage",
    "SetBrushOrgEx",
    "Rectangle",
    "Ellipse",
    "MoveToEx",
    "LineTo",
    "DrawTextW",
    "ExtTextOutW",
    "LoadMenuW",
    "CheckMenuItem",
    "RemoveMenu",
    "DestroyMenu",
    "LoadAcceleratorsW",
    "DestroyAcceleratorTable",
    "TranslateAcceleratorW",
];

const REGISTRY_PREFIXES: &[&str] = &[
    "RegClose",
    "RegCreate",
    "RegDelete",
    "RegEnum",
    "RegOpen",
    "RegQuery",
    "RegSet",
    "RegFlush",
    "RegCopy",
    "RegRestore",
    "RegSave",
    "RegReplace",
    "CeReg",
    "Registry",
];
const FILESYSTEM_PREFIXES: &[&str] = &[
    "CreateFile",
    "ReadFile",
    "WriteFile",
    "CopyFile",
    "DeleteFile",
    "MoveFile",
    "FindFirstFile",
    "FindNextFile",
    "FindClose",
    "FindFirstChange",
    "FindNextChange",
    "GetFile",
    "SetFile",
    "FlushFile",
    "LockFile",
    "UnlockFile",
    "CreateDirectory",
    "RemoveDirectory",
    "GetDisk",
    "GetStore",
    "CeFs",
    "CeGetVolume",
    "AFS_",
    "LoadFSD",
    "RegisterAFS",
    "DeregisterAFS",
];
const DEVICE_IO_PREFIXES: &[&str] = &[
    "DeviceIoControl",
    "RegisterDevice",
    "DeregisterDevice",
    "ActivateDevice",
    "DeactivateDevice",
    "GetDevice",
    "EnumDevice",
    "OpenDevice",
    "DDKReg",
    "Resource",
    "ClearComm",
    "EscapeComm",
    "GetComm",
    "PurgeComm",
    "SetComm",
    "SetupComm",
    "TransmitComm",
    "WaitComm",
];
const GWE_MESSAGE_PREFIXES: &[&str] = &[
    "DispatchMessage",
    "GetMessage",
    "PeekMessage",
    "PostMessage",
    "PostQuitMessage",
    "PostThreadMessage",
    "SendMessage",
    "SendNotifyMessage",
    "TranslateMessage",
    "MsgWait",
    "InSendMessage",
    "GetQueue",
];
const GWE_WINDOW_PREFIXES: &[&str] = &[
    "CreateWindow",
    "DestroyWindow",
    "DefWindow",
    "SetWindow",
    "GetWindow",
    "IsWindow",
    "MoveWindow",
    "ShowWindow",
    "UpdateWindow",
    "Invalidate",
    "Validate",
    "BeginPaint",
    "EndPaint",
    "GetDC",
    "ReleaseDC",
    "SetParent",
    "GetParent",
    "FindWindow",
    "EnableWindow",
    "CallWindow",
    "MapWindow",
    "RegisterClass",
    "RegisterGesture",
    "UnregisterClass",
    "GetClass",
    "SetClass",
    "SetTimer",
    "KillTimer",
    "CreateDialog",
    "DialogBox",
    "DefDlg",
    "GetDlg",
    "SetDlg",
    "EndDialog",
    "IsDialog",
    "CheckRadio",
];
const GDI_PREFIXES: &[&str] = &[
    "CreateDIB",
    "CreateBitmap",
    "CreateCompatible",
    "CreateFont",
    "CreatePen",
    "CreateSolidBrush",
    "CreatePatternBrush",
    "CreatePalette",
    "CreateRectRgn",
    "BitBlt",
    "Stretch",
    "SetDIB",
    "GetDIB",
    "Draw",
    "ExtText",
    "GetText",
    "SetText",
    "DeleteObject",
    "DeleteDC",
    "GetObject",
    "GetStock",
    "SelectObject",
    "Fill",
    "PatBlt",
    "Rectangle",
    "RoundRect",
    "Ellipse",
    "Polygon",
    "Polyline",
    "LineTo",
    "MoveTo",
    "AlphaBlend",
    "GradientFill",
    "CombineRgn",
    "IntersectClip",
    "SelectClip",
    "OffsetRgn",
    "PtInRegion",
    "RectInRegion",
];
const MULTIMEDIA_PREFIXES: &[&str] = &[
    "wave", "mixer", "acm", "midi", "line", "phone", "Audio", "snd",
];
const MEMORY_PREFIXES: &[&str] = &[
    "LocalAlloc",
    "LocalReAlloc",
    "LocalSize",
    "LocalFree",
    "RemoteLocal",
    "RemoteHeap",
    "Heap",
    "Virtual",
    "MapView",
    "UnmapView",
    "CreateFileMapping",
    "CreateFileForMapping",
    "CeVirtual",
    "CeSafeCopyMemory",
    "MapPtr",
];
const SYNC_PREFIXES: &[&str] = &[
    "InitializeCriticalSection",
    "DeleteCriticalSection",
    "EnterCriticalSection",
    "LeaveCriticalSection",
    "TryEnterCriticalSection",
    "Interlocked",
    "CreateEvent",
    "EventModify",
    "WaitFor",
    "CreateMutex",
    "ReleaseMutex",
    "SignalStarted",
];
const THREAD_PROCESS_PREFIXES: &[&str] = &[
    "Thread",
    "CreateThread",
    "ExitThread",
    "TerminateThread",
    "SuspendThread",
    "ResumeThread",
    "GetThread",
    "SetThread",
    "CreateProcess",
    "TerminateProcess",
    "OpenProcess",
    "OpenThread",
    "GetProcess",
    "SetProc",
    "GetProc",
    "IsProcess",
    "Tls",
    "ConvertThreadToFiber",
    "CreateFiber",
    "DeleteFiber",
    "SwitchToFiber",
    "GetCurrentFiber",
    "GetFiberData",
    "CloseHandle",
];
const TIME_PREFIXES: &[&str] = &[
    "GetTick",
    "Sleep",
    "GetSystemTime",
    "SetSystemTime",
    "GetLocalTime",
    "SetLocalTime",
    "FileTime",
    "CompareFileTime",
    "GetTimeZone",
    "SetTimeZone",
    "CeGetRawTime",
];
const LOCALE_STRING_PREFIXES: &[&str] = &[
    "String",
    "lstr",
    "wsprintf",
    "wvsprintf",
    "MultiByte",
    "WideChar",
    "CompareString",
    "LCMap",
    "GetLocale",
    "SetLocale",
    "GetACP",
    "GetOEMCP",
    "SetACP",
    "SetOEMCP",
    "GetCPInfo",
    "IsDBCS",
    "Char",
    "FoldString",
    "FormatMessage",
    "GetDateFormat",
    "GetTimeFormat",
    "GetNumberFormat",
    "GetCurrencyFormat",
    "EnumCalendar",
    "EnumTime",
    "EnumDate",
    "EnumSystem",
    "GetStringType",
    "GetSystemDefault",
    "GetUserDefault",
    "SetUserDefault",
    "SetSystemDefault",
    "ConvertDefaultLocale",
    "IsValidLocale",
    "IsValidCodePage",
];
const CRYPTO_PREFIXES: &[&str] = &["Crypt", "A_SHA", "MD5", "IsEncryption"];
const COMM_PREFIXES: &[&str] = &[
    "ClearComm",
    "EscapeComm",
    "GetComm",
    "PurgeComm",
    "SetComm",
    "SetupComm",
    "TransmitComm",
    "WaitComm",
];
const STORAGE_PREFIXES: &[&str] = &[
    "OpenStore",
    "DismountStore",
    "FormatStore",
    "FindFirstStore",
    "FindNextStore",
    "FindCloseStore",
    "CreatePartition",
    "DeletePartition",
    "OpenPartition",
    "MountPartition",
    "DismountPartition",
    "RenamePartition",
    "SetPartition",
    "GetPartition",
    "FormatPartition",
    "FindFirstPartition",
    "FindNextPartition",
    "FindClosePartition",
    "GetStoreInfo",
];
const MSG_QUEUE_PREFIXES: &[&str] = &[
    "CreateMsgQueue",
    "ReadMsgQueue",
    "WriteMsgQueue",
    "GetMsgQueue",
    "CloseMsgQueue",
    "OpenMsgQueue",
];
const POWER_PREFIXES: &[&str] = &[
    "Power",
    "Battery",
    "SetSystemPower",
    "GetSystemPower",
    "SetDevicePower",
    "GetDevicePower",
    "RequestPower",
    "StopPower",
    "DevicePower",
    "ReleasePower",
];
const SERVICE_PREFIXES: &[&str] = &[
    "Service",
    "ActivateService",
    "RegisterService",
    "DeregisterService",
    "CloseAllService",
];
const TELEPHONY_PREFIXES: &[&str] = &["line", "phone"];
const SECURITY_PREFIXES: &[&str] = &[
    "CeAccess",
    "CePrivilege",
    "CeCreateToken",
    "CeImpersonate",
    "CeGetOwner",
    "CeGetGroup",
    "CeConvert",
    "CheckPassword",
    "SetPassword",
    "GetPassword",
    "VerifyUser",
    "LASS",
    "CePolicy",
    "CeCert",
];
const DEBUG_PREFIXES: &[&str] = &[
    "Debug",
    "Attach",
    "Connect",
    "CaptureDump",
    "ReportFault",
    "CeLog",
    "Profile",
    "SetDbg",
    "GetLastError",
    "SetLastError",
    "OutputDebug",
    "NKDbg",
    "Rtl",
];
const INPUT_IME_PREFIXES: &[&str] = &[
    "Imm",
    "Ime",
    "DefaultIm",
    "SendInput",
    "mouse",
    "keybd",
    "Keybd",
    "GetAsync",
    "GetKey",
    "MapVirtualKey",
    "PostKeybd",
    "EnableHardwareKeyboard",
    "RegisterHotKey",
    "UnregisterHotKey",
    "SetWindowsHook",
    "UnhookWindowsHook",
    "CallNextHook",
    "AllKeys",
    "Touch",
    "Gesture",
    "Sip",
];
const SHELL_UI_PREFIXES: &[&str] = &[
    "CreateCaret",
    "DestroyCaret",
    "HideCaret",
    "ShowCaret",
    "SetCaret",
    "GetCaret",
    "OpenClipboard",
    "CloseClipboard",
    "GetClipboard",
    "SetClipboard",
    "RegisterClipboard",
    "CountClipboard",
    "EnumClipboard",
    "EmptyClipboard",
    "IsClipboard",
    "GetPriorityClipboard",
    "InsertMenu",
    "AppendMenu",
    "RemoveMenu",
    "DestroyMenu",
    "TrackPopupMenu",
    "LoadMenu",
    "EnableMenu",
    "CheckMenu",
    "DeleteMenu",
    "CreateMenu",
    "CreatePopupMenu",
    "SetMenu",
    "GetMenu",
    "DrawMenuBar",
    "MessageBox",
    "MessageBeep",
    "Shell",
    "NotifyWinUser",
    "ExtractIcon",
    "CreateIcon",
    "DestroyIcon",
    "DrawIcon",
    "LoadIcon",
    "GetIcon",
    "DestroyCursor",
    "CreateCursor",
    "SetCursor",
    "LoadCursor",
    "ClipCursor",
    "GetCursor",
    "ShowCursor",
    "LoadImage",
    "ImageList",
];
const BLUETOOTH_PREFIXES: &[&str] = &["Bluetooth"];
const EVENT_LOG_PREFIXES: &[&str] = &[
    "ClearEventLog",
    "ReportEvent",
    "RegisterEventSource",
    "DeregisterEventSource",
    "OpenEventLog",
    "CloseEventLog",
    "BackupEventLog",
    "LockEventLog",
    "UnLockEventLog",
    "ReadEventLog",
];
const CREDENTIAL_PREFIXES: &[&str] = &["Cred"];
const MATH_PREFIXES: &[&str] = &[
    "__", "abs", "acos", "asin", "atan", "ceil", "cos", "div", "exp", "fabs", "floor", "fmod",
    "frexp", "labs", "ldexp", "ldiv", "log", "modf", "pow", "sin", "sqrt", "tan", "MulDiv",
    "Random",
];

const VOID_NOOP_STUBS: &[&str] = &[
    "InitializeCriticalSection",
    "DeleteCriticalSection",
    "EnterCriticalSection",
    "LeaveCriticalSection",
    "SetLastError",
    "OutputDebugStringW",
    "OutputDebugStringA",
];

const BOOL_SUCCESS_STUBS: &[&str] = &[
    "TryEnterCriticalSection",
    "IsValidLocale",
    "IsValidCodePage",
];

const INVALID_HANDLE_STUBS: &[&str] = &[
    "CreateFileForMappingW",
    "CreateFileMappingW",
    "CreateProcessW",
    "CreateThread",
    "LoadLibraryW",
    "LoadLibraryExW",
];

const NULL_POINTER_STUB_PREFIXES: &[&str] = &[
    "LocalAlloc",
    "VirtualAlloc",
    "MapViewOfFile",
    "HeapAlloc",
    "TlsGetValue",
    "GetProcAddress",
    "GetModuleHandle",
];

const BOOL_FAILURE_STUB_PREFIXES: &[&str] = &[
    "LocalFree",
    "VirtualFree",
    "UnmapViewOfFile",
    "HeapFree",
    "TlsFree",
    "TlsSetValue",
    "FreeLibrary",
];

const CEMATH_EXPORTS: &[(&str, u32)] = &[
    ("abs", 988),
    ("acos", 989),
    ("asin", 990),
    ("atan", 991),
    ("atan2", 992),
    ("ceil", 999),
    ("cos", 1004),
    ("cosh", 1005),
    ("div", 1007),
    ("exp", 1009),
    ("fabs", 1010),
    ("floor", 1013),
    ("fmod", 1014),
    ("frexp", 1019),
    ("labs", 1030),
    ("ldexp", 1031),
    ("ldiv", 1032),
    ("log", 1033),
    ("log10", 1034),
    ("modf", 1048),
    ("pow", 1051),
    ("sin", 1058),
    ("sinh", 1059),
    ("sqrt", 1060),
    ("tan", 1075),
    ("tanh", 1076),
    ("__ll_rshift", 2002),
    ("__ll_lshift", 2003),
    ("__ll_mul", 2004),
    ("__ll_div", 2005),
    ("__ll_rem", 2006),
    ("__ull_rshift", 2011),
    ("__ull_div", 2012),
    ("__ull_rem", 2013),
    ("__fpadd", 2022),
    ("__dpadd", 2023),
    ("__fpsub", 2024),
    ("__dpsub", 2025),
    ("__fpmul", 2026),
    ("__dpmul", 2027),
    ("__fpdiv", 2028),
    ("__dpdiv", 2029),
    ("__fptoli", 2030),
    ("__fptoul", 2031),
    ("__litofp", 2032),
    ("__ultofp", 2033),
    ("__dptoli", 2034),
    ("__dptoul", 2035),
    ("__litodp", 2036),
    ("__ultodp", 2037),
    ("__fptodp", 2038),
    ("__dptofp", 2039),
    ("__fpcmp", 2040),
    ("__dpcmp", 2041),
    ("__lts", 2042),
    ("__les", 2043),
    ("__eqs", 2044),
    ("__ges", 2045),
    ("__gts", 2046),
    ("__nes", 2047),
    ("__ltd", 2048),
    ("__led", 2049),
    ("__eqd", 2050),
    ("__ged", 2051),
    ("__gtd", 2052),
    ("__ned", 2053),
];
