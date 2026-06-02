use std::{collections::BTreeMap, fs, path::Path};

use crate::{
    ce::{
        audio::{
            MMSYSERR_BADDEVICEID, MMSYSERR_INVALHANDLE, MMSYSERR_NOERROR, MmResult,
            WAVERR_BADFORMAT, WaveBuffer, WaveFormat, WaveOutCallback,
        },
        cemath::{CeMathCall, CeMathValue},
        coredll_ordinals::{self, *},
        crt,
        devices::DeviceIoControlResult,
        file::FileIoResult,
        file::FindData,
        framebuffer::{Framebuffer, FramebufferRect, PixelFormat},
        gwe::{Message, PeekFlags, Point, Rect, WNDCLASSW_SIZE},
        kernel::{CeKernel, MessagePumpResult},
        memory::{HEAP_ZERO_MEMORY, PROCESS_HEAP_HANDLE},
        object::{CriticalSectionObject, KernelObject},
        registry::{HKey, RegOpenResult, RegQueryValueResult},
        resource::{AcceleratorEntry, ResourceId, stock_object_handle},
        thread::{
            ERROR_ALREADY_EXISTS, ERROR_CLASS_DOES_NOT_EXIST, ERROR_FILE_NOT_FOUND,
            ERROR_INVALID_HANDLE, ERROR_INVALID_PARAMETER, ERROR_INVALID_WINDOW_HANDLE,
            ERROR_NOT_ENOUGH_MEMORY, ERROR_NOT_SUPPORTED, ERROR_RESOURCE_NAME_NOT_FOUND,
        },
    },
    error::{Error, Result},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoredllExport {
    pub name: String,
    pub target: Option<String>,
    pub ordinal: u32,
    pub noname: bool,
    pub line: usize,
}

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

    pub fn from_static_ordinals() -> Self {
        let mut table = Self::empty();
        for ordinal in COREDLL_EXPORTS {
            table.insert(CoredllExport::from_static_ordinal(ordinal));
        }
        for ordinal in SDK_ORDINALS {
            table.insert(CoredllExport::from_static_ordinal(ordinal));
        }
        table
    }

    pub fn resolve_static_ordinal(ordinal: u32) -> Option<CoredllExport> {
        coredll_ordinals::lookup(ordinal).map(CoredllExport::from_static_ordinal)
    }

    pub fn from_core_common_def_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(Self::from_core_common_def(&text))
    }

    pub fn from_core_common_def(text: &str) -> Self {
        let mut table = Self::empty();
        for (line_index, line) in text.lines().enumerate() {
            if let Some(export) = parse_export_line(line, line_index + 1) {
                table.insert(export);
            }
        }
        table
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
            target: ordinal.target.map(str::to_owned),
            ordinal: ordinal.ordinal,
            noname: ordinal.noname,
            line: ordinal.line,
        }
    }

    fn matches_name(&self, name: &str) -> bool {
        normalize_name(&self.name) == normalize_name(name)
            || self
                .target
                .as_deref()
                .is_some_and(|target| normalize_name(target) == normalize_name(name))
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
        let target = export.target.as_deref().map(normalize_name);
        if export.line == 0 || has_any_prefix(&name, MATH_PREFIXES) {
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
        if target
            .as_deref()
            .is_some_and(|target| has_any_prefix(target, FILESYSTEM_PREFIXES))
        {
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
        CoredllCall::DestroyWindow { hwnd } => {
            CoredllValue::Bool(kernel.gwe.destroy_window(hwnd, kernel.timers.tick_count()))
        }
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
        } => CoredllValue::OptionalMessage(
            kernel
                .gwe
                .peek_message_filtered(thread_id, hwnd, min_msg, max_msg, flags),
        ),
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
            kernel.timers.sleep_ms(raw_arg(args, 0));
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
        ORD_SET_THREAD_PRIORITY | ORD_CE_SET_THREAD_PRIORITY => Some(CoredllValue::Bool(
            set_thread_priority_raw(kernel, thread_id, raw_arg(args, 0), raw_arg(args, 1)),
        )),
        ORD_GET_THREAD_PRIORITY | ORD_CE_GET_THREAD_PRIORITY => Some(CoredllValue::U32(
            get_thread_priority_raw(kernel, thread_id, raw_arg(args, 0)),
        )),
        ORD_GLOBAL_MEMORY_STATUS => {
            write_global_memory_status(kernel, memory, thread_id, raw_arg(args, 0));
            Some(CoredllValue::U32(0))
        }
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
        ORD_WCSDUP => Some(CoredllValue::Handle(crt::wcsdup_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_WCSNICMP => Some(CoredllValue::U32(crt::wcsnicmp_raw(
            memory,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ) as u32)),
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
        ORD_MALLOC | ORD_OPERATOR_NEW => Some(CoredllValue::Handle(crt::malloc_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_MEMCPY => Some(CoredllValue::Handle(crt::memcpy_raw(
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
        ORD_WSPRINTF_W | ORD_SWPRINTF => Some(CoredllValue::U32(crt::wsprintf_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            if args.len() > 2 { &args[2..] } else { &[] },
        ))),
        ORD_PRINTF => Some(CoredllValue::U32(crt::printf_family_raw(kernel, thread_id))),
        ORD_FREE | ORD_OPERATOR_DELETE => {
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
        ORD_DESTROY_WINDOW => Some(CoredllValue::Bool(
            kernel
                .gwe
                .destroy_window(raw_arg(args, 0), kernel.timers.tick_count()),
        )),
        ORD_SHOW_WINDOW => Some(CoredllValue::Bool(
            kernel.show_window(raw_arg(args, 0), raw_arg(args, 1) != 0),
        )),
        ORD_UPDATE_WINDOW => Some(CoredllValue::Bool(
            kernel.gwe.update_window(raw_arg(args, 0)),
        )),
        ORD_INVALIDATE_RECT => Some(CoredllValue::Bool(invalidate_rect_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_VALIDATE_RECT => Some(CoredllValue::Bool(validate_rect_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_GET_UPDATE_RECT => Some(CoredllValue::Bool(get_update_rect_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
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
        ORD_BIT_BLT => Some(CoredllValue::Bool(bit_blt_raw(kernel, thread_id, args))),
        ORD_STRETCH_BLT => Some(CoredllValue::Bool(bit_blt_raw(kernel, thread_id, args))),
        ORD_STRETCH_DIBITS => Some(CoredllValue::U32(stretch_dibits_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_SET_DIBITS_TO_DEVICE => Some(CoredllValue::U32(set_dibits_to_device_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_SET_DIBCOLOR_TABLE => Some(CoredllValue::U32(set_dib_color_table_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
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
            kernel, thread_id, args,
        ))),
        ORD_SET_BRUSH_ORG_EX => Some(CoredllValue::Bool(set_brush_org_ex_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_RECTANGLE | ORD_ROUND_RECT | ORD_ELLIPSE | ORD_POLYGON | ORD_POLYLINE => {
            Some(CoredllValue::Bool(gdi_shape_raw(kernel, thread_id, args)))
        }
        ORD_MOVE_TO_EX => Some(CoredllValue::Bool(move_to_ex_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_LINE_TO => Some(CoredllValue::Bool(line_to_raw(kernel, thread_id, args))),
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
                .gwe
                .enable_window(raw_arg(args, 0), raw_arg(args, 1) != 0),
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
        ORD_GET_DLG_ITEM => Some(CoredllValue::Handle(
            kernel
                .gwe
                .get_dlg_item(raw_arg(args, 0), raw_arg(args, 1))
                .unwrap_or(0),
        )),
        ORD_GET_DLG_CTRL_ID => Some(CoredllValue::U32(
            kernel.gwe.get_dlg_ctrl_id(raw_arg(args, 0)).unwrap_or(0),
        )),
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
        ORD_SEND_DLG_ITEM_MESSAGE_W => Some(CoredllValue::U32(send_dlg_item_message_w_raw(
            kernel, thread_id, args,
        ))),
        ORD_CHECK_RADIO_BUTTON => Some(CoredllValue::Bool(check_radio_button_raw(
            kernel, thread_id, args,
        ))),
        ORD_IS_DIALOG_MESSAGE_W => Some(CoredllValue::Bool(is_dialog_message_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_SET_PARENT => Some(CoredllValue::Handle(
            kernel
                .gwe
                .set_parent(
                    raw_arg(args, 0),
                    (raw_arg(args, 1) != 0).then_some(raw_arg(args, 1)),
                )
                .flatten()
                .unwrap_or(0),
        )),
        ORD_GET_WINDOW => Some(CoredllValue::Handle(get_window_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_GET_DESKTOP_WINDOW => Some(CoredllValue::Handle(kernel.gwe.get_desktop_window())),
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
            kernel
                .gwe
                .set_focus((raw_arg(args, 0) != 0).then_some(raw_arg(args, 0)))
                .unwrap_or(0),
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
        ORD_GET_MESSAGE_W => Some(CoredllValue::Bool(get_message_w_raw(
            kernel, memory, thread_id, args,
        ))),
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
            kernel
                .gwe
                .post_quit_message(thread_id, raw_arg(args, 0), kernel.timers.tick_count());
            Some(CoredllValue::U32(0))
        }
        ORD_SEND_MESSAGE_W | ORD_DEF_WINDOW_PROC_W => Some(CoredllValue::U32(
            kernel
                .send_message_w(
                    raw_arg(args, 0),
                    raw_arg(args, 1),
                    raw_arg(args, 2),
                    raw_arg(args, 3),
                )
                .unwrap_or(0),
        )),
        ORD_SEND_NOTIFY_MESSAGE_W => {
            let ok = kernel.gwe.is_window(raw_arg(args, 0))
                && kernel
                    .send_message_w(
                        raw_arg(args, 0),
                        raw_arg(args, 1),
                        raw_arg(args, 2),
                        raw_arg(args, 3),
                    )
                    .is_some();
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
        ORD_GET_MESSAGE_QUEUE_READY_TIME_STAMP => {
            Some(CoredllValue::U32(kernel.timers.tick_count()))
        }
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
        _ => None,
    }
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
    let input_ptr = raw_arg(args, 2);
    let input_len = raw_arg(args, 3) as i32;
    let output_ptr = raw_arg(args, 4);
    let output_capacity = raw_arg(args, 5);
    let Some(bytes) = read_conversion_bytes(kernel, memory, thread_id, input_ptr, input_len) else {
        return 0;
    };
    let units: Vec<u16> = bytes.into_iter().map(u16::from).collect();
    write_conversion_wide_result(
        kernel,
        memory,
        thread_id,
        output_ptr,
        output_capacity,
        &units,
    )
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
    let Some(path) = read_guest_wide_arg(memory, raw_arg(args, 0)) else {
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
    if year < 1601
        || month == 0
        || month > 12
        || day == 0
        || day > 31
        || hour > 23
        || minute > 59
        || second > 59
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
    let ticks = (seconds as u64).saturating_mul(10_000_000);
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
    kernel
        .threads
        .set_last_error(thread_id, ERROR_FILE_NOT_FOUND);
    0
}

fn free_library_raw(kernel: &mut CeKernel, thread_id: u32, module: u32) -> bool {
    if module == COREDLL_MODULE_HANDLE || module == kernel.process_module_base() {
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
    let Some(name) = read_guest_wide_arg(memory, raw_arg(args, 1)) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    get_coredll_proc_address_raw(kernel, thread_id, raw_arg(args, 0), &name)
}

fn get_proc_address_a_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let Some(name) = read_guest_ascii_z(memory, raw_arg(args, 1), 256) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    get_coredll_proc_address_raw(kernel, thread_id, raw_arg(args, 0), &name)
}

fn get_coredll_proc_address_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    module: u32,
    name: &str,
) -> u32 {
    if module != COREDLL_MODULE_HANDLE {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    }
    let table = CoredllExportTable::default();
    let Some(export) = table.resolve_name(name) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_FILE_NOT_FOUND);
        return 0;
    };
    let Some(address) = crate::emulator::imports::dynamic_coredll_proc_address(export.ordinal)
    else {
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
    let bytes = match kernel.read_file(handle, requested) {
        Ok(bytes) => bytes,
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            write_optional_count(kernel, memory, thread_id, transferred_ptr, 0);
            return false;
        }
    };
    if !write_guest_bytes(kernel, memory, thread_id, buffer, &bytes) {
        write_optional_count(kernel, memory, thread_id, transferred_ptr, 0);
        return false;
    }
    write_optional_count(
        kernel,
        memory,
        thread_id,
        transferred_ptr,
        bytes.len() as u32,
    )
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
    let high = if high_ptr == 0 {
        0
    } else {
        match read_guest_u32(kernel, memory, thread_id, high_ptr) {
            Some(high) => high,
            None => return u32::MAX,
        }
    };
    let distance = (((high as u64) << 32) | low as u64) as i64;
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
    const MAXIMUM_WAIT_OBJECTS: u32 = 64;

    if count > MAXIMUM_WAIT_OBJECTS || (count != 0 && handles_ptr == 0) {
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
            return result;
        }
    }

    kernel.pump_timers_to_gwe(thread_id);
    if kernel.gwe.has_queue_input(thread_id, wake_mask) {
        kernel.threads.set_last_error(thread_id, 0);
        return crate::ce::timer::WAIT_OBJECT_0 + count;
    }

    let result = if timeout_ms == 0 {
        crate::ce::timer::WAIT_TIMEOUT
    } else {
        crate::ce::timer::WAIT_TIMEOUT
    };
    kernel.threads.set_last_error(thread_id, 0);
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
    match kernel.suspend_thread(handle) {
        Some(previous) => {
            kernel.threads.set_last_error(thread_id, 0);
            previous
        }
        None => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            u32::MAX
        }
    }
}

fn resume_thread_raw(kernel: &mut CeKernel, thread_id: u32, handle: u32) -> u32 {
    match kernel.resume_thread(handle) {
        Some(previous) => {
            kernel.threads.set_last_error(thread_id, 0);
            previous
        }
        None => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            u32::MAX
        }
    }
}

fn get_thread_id_raw(kernel: &mut CeKernel, thread_id: u32, handle: u32) -> u32 {
    match kernel.guest_thread_id(handle) {
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
    let Some(exit_code) = kernel.guest_thread_exit_code(handle) else {
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
    let Some(exit_code) = kernel.process_exit_code(handle) else {
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
    match kernel.process_id(handle) {
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
    if kernel.guest_thread_id(handle).is_none() {
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

fn get_thread_priority_raw(kernel: &mut CeKernel, thread_id: u32, handle: u32) -> u32 {
    const THREAD_PRIORITY_ERROR_RETURN: u32 = 0x7fff_ffff;

    match kernel.thread_priority(handle) {
        Some(priority) => {
            kernel.threads.set_last_error(thread_id, 0);
            priority as u32
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
) -> bool {
    if kernel.set_thread_priority(handle, priority as i32) {
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
    let parent = (raw_arg(args, 8) != 0).then_some(raw_arg(args, 8));
    tracing::debug!(
        target: "ce.gwe",
        class_name = class_name.as_str(),
        class_ptr = format_args!("0x{:08x}", raw_arg(args, 1)),
        title = title.as_str(),
        style = format_args!("0x{:08x}", raw_arg(args, 3)),
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
    kernel.create_window_ex_w_with_rect(
        thread_id,
        &class_name,
        &title,
        parent,
        raw_arg(args, 9),
        raw_arg(args, 3),
        raw_arg(args, 0),
        rect,
    )
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
    let hwnd = kernel.gwe.create_window_ex_with_rect(
        thread_id,
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
        let control_hwnd = kernel.gwe.create_window_ex_with_rect(
            thread_id,
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

fn send_dlg_item_message_w_raw(kernel: &mut CeKernel, thread_id: u32, args: &[u32]) -> u32 {
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
        _ => kernel
            .send_message_w(child, msg, wparam, lparam)
            .unwrap_or(0),
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

fn check_menu_item_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    menu: u32,
    item: u32,
    flags: u32,
) -> u32 {
    let checked = flags & MF_CHECKED != 0;
    let Some(previous) = kernel.resources.check_menu_item(menu, item, checked) else {
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

fn remove_menu_raw(kernel: &mut CeKernel, thread_id: u32, menu: u32, item: u32) -> bool {
    if !kernel.resources.remove_menu_item(menu, item) {
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
    let _ = kernel.resources.delete_gdi_object(handle);
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

fn validate_rect_raw(kernel: &mut CeKernel, thread_id: u32, hwnd: u32) -> bool {
    if !kernel.gwe.validate_window(hwnd) {
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
    kernel.threads.set_last_error(thread_id, 0);
    true
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
    if hdc == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    }
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
        .create_bitmap(width, height, 1, 32, bits_ptr)
}

fn create_dib_section_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let hdc = raw_arg(args, 0);
    let info_ptr = raw_arg(args, 1);
    let color_usage = raw_arg(args, 2);
    let bits_out = raw_arg(args, 3);
    let section = raw_arg(args, 4);
    let offset = raw_arg(args, 5);
    if hdc == 0 || info_ptr == 0 || bits_out == 0 || color_usage != DIB_RGB_COLORS || offset != 0 {
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
    if header.width == 0
        || header.height == 0
        || header.planes != 1
        || header.bits_pixel == 0
        || compression != BI_RGB
    {
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
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    kernel.resources.create_bitmap(
        header.width,
        header.height,
        header.planes,
        header.bits_pixel,
        bits_ptr,
    )
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
    let red = (colorref & 0xff) as u8;
    let green = ((colorref >> 8) & 0xff) as u8;
    let blue = ((colorref >> 16) & 0xff) as u8;
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

fn bit_blt_raw(kernel: &mut CeKernel, thread_id: u32, args: &[u32]) -> bool {
    let dst = raw_arg(args, 0);
    let width = raw_i32_arg(args, 3);
    let height = raw_i32_arg(args, 4);
    let src = raw_arg(args, 5);
    if dst == 0 || src == 0 || width <= 0 || height <= 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
    kernel.threads.set_last_error(thread_id, 0);
    true
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

fn transparent_image_raw(kernel: &mut CeKernel, thread_id: u32, args: &[u32]) -> bool {
    let dst = raw_arg(args, 0);
    let dst_width = raw_i32_arg(args, 3);
    let dst_height = raw_i32_arg(args, 4);
    let src = raw_arg(args, 5);
    let src_width = raw_i32_arg(args, 8);
    let src_height = raw_i32_arg(args, 9);
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
    kernel.threads.set_last_error(thread_id, 0);
    true
}

fn stretch_dibits_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let hdc = raw_arg(args, 0);
    let dst_height = raw_i32_arg(args, 4);
    let src_height = raw_i32_arg(args, 8);
    let bits = raw_arg(args, 9);
    let info = raw_arg(args, 10);
    if hdc == 0 || dst_height == 0 || src_height == 0 || bits == 0 || info == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    if read_guest_u32(kernel, memory, thread_id, info).is_none() {
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    src_height.unsigned_abs()
}

fn set_dibits_to_device_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    args: &[u32],
) -> u32 {
    let hdc = raw_arg(args, 0);
    let lines = raw_arg(args, 8);
    let bits = raw_arg(args, 9);
    let info = raw_arg(args, 10);
    if hdc == 0 || lines == 0 || bits == 0 || info == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    if read_guest_u32(kernel, memory, thread_id, info).is_none() {
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    lines
}

fn set_dib_color_table_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    hdc: u32,
    entries: u32,
    colors_ptr: u32,
) -> u32 {
    if hdc == 0 || entries == 0 || colors_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    entries
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

fn line_to_raw(kernel: &mut CeKernel, thread_id: u32, args: &[u32]) -> bool {
    if raw_arg(args, 0) == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return false;
    }
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
    kernel.dispatch_message_w(message)
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
    let result_ptr = raw_arg(args, 6);
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
    let mut bytes = Vec::with_capacity(len as usize);
    for offset in 0..len {
        match memory.read_u8(addr.wrapping_add(offset)) {
            Ok(byte) => bytes.push(byte),
            Err(_) => {
                kernel
                    .threads
                    .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
                return None;
            }
        }
    }
    Some(bytes)
}

pub(crate) fn write_guest_bytes<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
    bytes: &[u8],
) -> bool {
    for (offset, byte) in bytes.iter().copied().enumerate() {
        if memory
            .write_u8(addr.wrapping_add(offset as u32), byte)
            .is_err()
        {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            return false;
        }
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
    })
}

fn write_guest_message<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    addr: u32,
    message: &Message,
) -> bool {
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

fn parse_export_line(line: &str, line_number: usize) -> Option<CoredllExport> {
    let trimmed = line.trim();
    if trimmed.is_empty()
        || trimmed.starts_with(';')
        || trimmed.starts_with("//")
        || trimmed.starts_with('#')
    {
        return None;
    }

    let at = trimmed.rfind('@')?;
    let ordinal = parse_ordinal(&trimmed[at + 1..])?;
    let before = trimmed[..at].trim();
    let name = extract_export_name(before)?;
    let target = extract_export_target(before);
    let noname = trimmed[at + 1..]
        .split_ascii_whitespace()
        .any(|part| part.eq_ignore_ascii_case("NONAME"));

    Some(CoredllExport {
        name,
        target,
        ordinal,
        noname,
        line: line_number,
    })
}

fn parse_ordinal(text: &str) -> Option<u32> {
    let digits: String = text
        .trim_start()
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    (!digits.is_empty()).then(|| digits.parse().ok()).flatten()
}

fn extract_export_name(before_ordinal: &str) -> Option<String> {
    let mut text = before_ordinal.trim().trim_end_matches(')').trim();
    if let Some(eq) = text.find('=') {
        text = text[..eq].trim();
    }
    if let Some(paren) = text.rfind('(') {
        text = text[paren + 1..].trim();
    }
    if let Some(comma) = text.find(',') {
        text = text[..comma].trim();
    }
    if let Some(space) = text.find(char::is_whitespace) {
        text = text[..space].trim();
    }
    (!text.is_empty()).then(|| text.to_owned())
}

fn extract_export_target(before_ordinal: &str) -> Option<String> {
    let eq = before_ordinal.find('=')?;
    let mut text = before_ordinal[eq + 1..].trim().trim_end_matches(')').trim();
    if let Some(paren) = text.rfind('(') {
        text = text[paren + 1..].trim();
    }
    if let Some(comma) = text.find(',') {
        text = text[..comma].trim();
    }
    if let Some(space) = text.find(char::is_whitespace) {
        text = text[..space].trim();
    }
    (!text.is_empty()).then(|| text.to_owned())
}

fn normalize_name(name: &str) -> String {
    name.to_ascii_lowercase()
}

fn has_any_prefix(name: &str, prefixes: &[&str]) -> bool {
    prefixes
        .iter()
        .any(|prefix| name.starts_with(&normalize_name(prefix)))
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
const CLR_INVALID: u32 = 0xffff_ffff;
const DIB_RGB_COLORS: u32 = 0;
const PALETTE_ENTRY_SIZE: usize = 4;
const PALETTE_ENTRY_SIZE_U32: u32 = 4;
const MAX_LOG_PALETTE_ENTRIES: u32 = 4096;
const DS_SETFONT: u32 = 0x0000_0040;
const LR_LOADFROMFILE: u32 = 0x0000_0010;
const MF_CHECKED: u32 = 0x0000_0008;
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
    "MultiByteToWideChar",
    "WideCharToMultiByte",
    "CharLowerW",
    "CharLowerBuffW",
    "CharUpperBuffW",
    "CharUpperW",
    "memcpy",
    "memset",
    "??2@YAPAXI@Z",
    "swprintf",
    "printf",
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
    "FindFirstFileW",
    "FindClose",
    "ReadFile",
    "WriteFile",
    "SetFilePointer",
    "GetFileSize",
    "FlushFileBuffers",
    "GetStoreInformation",
    "DeviceIoControl",
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
    "ADB",
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
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_def_line_forms() {
        let text = "\
   STDAPI(InitializeCriticalSection,4) @2
   IsProcessDying @ 1213
   RegOpenKeyExW=xxx_RegOpenKeyExW @461
   DPA_CreateEx @1838 NONAME
   KCOREDLL_ONLY(DirectHandleCall @2552)
";
        let table = CoredllExportTable::from_core_common_def(text);

        assert_eq!(table.export_count(), 5);
        assert_eq!(
            table.resolve_ordinal(2).unwrap().name,
            "InitializeCriticalSection"
        );
        assert_eq!(table.resolve_name("RegOpenKeyExW").unwrap().ordinal, 461);
        assert!(table.resolve_name("DPA_CreateEx").unwrap().noname);
        assert_eq!(
            table.resolve_ordinal(2552).unwrap().name,
            "DirectHandleCall"
        );
    }
}
