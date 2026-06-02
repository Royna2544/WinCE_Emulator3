use std::{collections::BTreeMap, fs, path::Path};

use crate::{
    ce::{
        audio::{
            MMSYSERR_BADDEVICEID, MMSYSERR_INVALHANDLE, MMSYSERR_NOERROR, MmResult,
            WAVERR_BADFORMAT, WaveBuffer, WaveFormat,
        },
        cemath::{CeMathCall, CeMathValue},
        coredll_ordinals::{self, *},
        crt,
        devices::DeviceIoControlResult,
        file::FileIoResult,
        file::FindData,
        gwe::{Message, PeekFlags, Point, Rect, WNDCLASSW_SIZE},
        kernel::{CeKernel, MessagePumpResult},
        memory::HEAP_GENERATE_EXCEPTIONS,
        object::{CriticalSectionObject, KernelObject},
        registry::{HKey, RegOpenResult, RegQueryValueResult},
        resource::ResourceId,
        thread::{
            ERROR_CLASS_DOES_NOT_EXIST, ERROR_FILE_NOT_FOUND, ERROR_INVALID_HANDLE,
            ERROR_INVALID_PARAMETER, ERROR_INVALID_WINDOW_HANDLE, ERROR_NOT_ENOUGH_MEMORY,
            ERROR_NOT_SUPPORTED, ERROR_RESOURCE_NAME_NOT_FOUND,
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
        for ordinal in CE42_MIPSII_SDK_ORDINALS {
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
        match self.resolve_ordinal(ordinal).cloned() {
            Some(export) => {
                let args = args.into_iter().collect::<Vec<_>>();
                if let Some(value) =
                    dispatch_real_raw_ordinal(kernel, memory, thread_id, &export, &args)
                {
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
        ORD_GET_LAST_ERROR => Some(CoredllValue::U32(kernel.threads.get_last_error(thread_id))),
        ORD_SET_LAST_ERROR => {
            kernel.threads.set_last_error(thread_id, raw_arg(args, 0));
            Some(CoredllValue::U32(0))
        }
        ORD_GET_TICK_COUNT => Some(CoredllValue::U32(kernel.timers.tick_count())),
        ORD_SLEEP => {
            kernel.timers.sleep_ms(raw_arg(args, 0));
            Some(CoredllValue::U32(0))
        }
        ORD_GLOBAL_MEMORY_STATUS => {
            write_global_memory_status(kernel, memory, thread_id, raw_arg(args, 0));
            Some(CoredllValue::U32(0))
        }
        ORD_GET_SYSTEM_INFO => {
            write_system_info(kernel, memory, thread_id, raw_arg(args, 0));
            Some(CoredllValue::U32(0))
        }
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
        ORD_CREATE_MUTEX_W => Some(CoredllValue::Handle(create_mutex_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_RELEASE_MUTEX => Some(CoredllValue::Bool(release_mutex_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_WAIT_FOR_SINGLE_OBJECT => Some(CoredllValue::U32(kernel.wait_for_single_object(
            raw_arg(args, 0),
            raw_arg(args, 1),
            thread_id,
        ))),
        ORD_CREATE_FILE_W => Some(CoredllValue::Handle(create_file_w_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_FIND_FIRST_FILE_W => Some(CoredllValue::Handle(find_first_file_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
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
        ORD_SET_FILE_POINTER => Some(CoredllValue::U32(set_file_pointer_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_GET_FILE_SIZE => Some(CoredllValue::U32(get_file_size_raw(
            kernel, memory, thread_id, args,
        ))),
        ORD_FLUSH_FILE_BUFFERS => Some(CoredllValue::Bool(flush_file_buffers_raw(
            kernel,
            thread_id,
            raw_arg(args, 0),
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
        ORD_CE42_MIPSII_WCSRCHR => Some(CoredllValue::Handle(crt::wcsrchr_raw(
            memory,
            raw_arg(args, 0),
            raw_arg(args, 1),
        ))),
        ORD_CE42_MIPSII_WCSDUP => Some(CoredllValue::Handle(crt::wcsdup_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_CE42_MIPSII_MALLOC | ORD_CE42_MIPSII_OPERATOR_NEW => Some(CoredllValue::Handle(
            crt::malloc_raw(kernel, thread_id, raw_arg(args, 0)),
        )),
        ORD_CE42_MIPSII_MEMCPY => Some(CoredllValue::Handle(crt::memcpy_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_CE42_MIPSII_MEMSET => Some(CoredllValue::Handle(crt::memset_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
            raw_arg(args, 1),
            raw_arg(args, 2),
        ))),
        ORD_CE42_MIPSII_SWPRINTF | ORD_CE42_MIPSII_PRINTF => {
            Some(CoredllValue::U32(crt::printf_family_raw(kernel, thread_id)))
        }
        ORD_CE42_MIPSII_FREE => {
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
            kernel
                .gwe
                .show_window(raw_arg(args, 0), raw_arg(args, 1) != 0),
        )),
        ORD_UPDATE_WINDOW => Some(CoredllValue::Bool(
            kernel.gwe.update_window(raw_arg(args, 0)),
        )),
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
        ORD_GET_DESKTOP_WINDOW => Some(CoredllValue::Handle(kernel.gwe.get_desktop_window())),
        ORD_GET_ACTIVE_WINDOW => Some(CoredllValue::Handle(
            kernel.gwe.get_active_window().unwrap_or(0),
        )),
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
        ORD_SET_WINDOW_LONG_W => Some(CoredllValue::U32(
            kernel
                .gwe
                .set_window_long(raw_arg(args, 0), raw_i32_arg(args, 1), raw_arg(args, 2))
                .unwrap_or(0),
        )),
        ORD_GET_WINDOW_LONG_W => Some(CoredllValue::U32(
            kernel
                .gwe
                .get_window_long(raw_arg(args, 0), raw_i32_arg(args, 1))
                .unwrap_or(0),
        )),
        ORD_SET_WINDOW_POS => Some(CoredllValue::Bool(kernel.gwe.set_window_pos(
            raw_arg(args, 0),
            raw_i32_arg(args, 2),
            raw_i32_arg(args, 3),
            raw_i32_arg(args, 4),
            raw_i32_arg(args, 5),
            raw_arg(args, 6),
        ))),
        ORD_MOVE_WINDOW => Some(CoredllValue::Bool(kernel.gwe.move_window(
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
        ORD_DISPATCH_MESSAGE_W => Some(CoredllValue::U32(dispatch_message_w_raw(
            kernel,
            memory,
            thread_id,
            raw_arg(args, 0),
        ))),
        ORD_TRANSLATE_MESSAGE => Some(CoredllValue::Bool(raw_arg(args, 0) != 0)),
        ORD_MESSAGE_BOX_W => Some(CoredllValue::U32(1)),
        ORD_FIND_RESOURCE | ORD_FIND_RESOURCE_W => Some(CoredllValue::Handle(find_resource(
            kernel,
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
    let (handle, find_data) = match kernel.find_first_file_w(&pattern) {
        Ok(result) => result,
        Err(_) => {
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
        (40, 0),
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
    if flags & HEAP_GENERATE_EXCEPTIONS != 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_NOT_SUPPORTED);
        return 0;
    }
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
    if flags & HEAP_GENERATE_EXCEPTIONS != 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_NOT_SUPPORTED);
        return 0;
    }
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
    let handle = kernel.create_mutex_w(name, owner);
    kernel.threads.set_last_error(thread_id, 0);
    handle
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
    let class_name = read_guest_wide_arg(memory, raw_arg(args, 1)).unwrap_or_default();
    let title = read_guest_wide_arg(memory, raw_arg(args, 2)).unwrap_or_default();
    let rect = Rect::from_origin_size(
        raw_i32_arg(args, 4),
        raw_i32_arg(args, 5),
        raw_i32_arg(args, 6),
        raw_i32_arg(args, 7),
    );
    let parent = (raw_arg(args, 8) != 0).then_some(raw_arg(args, 8));
    kernel.gwe.create_window_ex_with_rect(
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
    handle: u32,
    header_ptr: u32,
) -> MmResult {
    let Some(data_ptr) = read_guest_u32(kernel, memory, 0, header_ptr) else {
        return MMSYSERR_INVALHANDLE;
    };
    let Some(len) = read_guest_u32(kernel, memory, 0, header_ptr.wrapping_add(4)) else {
        return MMSYSERR_INVALHANDLE;
    };
    let result = kernel.audio.wave_out_write(
        handle,
        WaveBuffer {
            guest_ptr: data_ptr,
            len,
        },
    );
    if result != MMSYSERR_NOERROR {
        return result;
    }
    if let Some(flags) = read_guest_u32(kernel, memory, 0, header_ptr.wrapping_add(16)) {
        let _ = write_guest_u32(
            kernel,
            memory,
            0,
            header_ptr.wrapping_add(16),
            (flags | WHDR_INQUEUE) & !WHDR_DONE,
        );
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

fn find_resource(kernel: &mut CeKernel, thread_id: u32, module: u32, name: u32, kind: u32) -> u32 {
    let Some(handle) = kernel.resources.find_resource(
        module,
        ResourceId::from_guest_arg(name),
        ResourceId::from_guest_arg(kind),
    ) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_RESOURCE_NAME_NOT_FOUND);
        return 0;
    };
    handle
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
    let from = (hwnd_from != 0).then_some(hwnd_from);
    let to = (hwnd_to != 0).then_some(hwnd_to);
    if !kernel.gwe.map_window_points(from, to, &mut points) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        return 0;
    }
    for (index, point) in points.into_iter().enumerate() {
        let point_ptr = points_ptr.wrapping_add((index as u32).wrapping_mul(8));
        if !write_guest_point(kernel, memory, thread_id, point_ptr, point) {
            return 0;
        }
    }
    1
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
const WIN32_FIND_DATAW_FILE_NAME: u32 = 44;
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
    "HeapCreate",
    "HeapDestroy",
    "HeapAlloc",
    "HeapAllocTrace",
    "HeapReAlloc",
    "HeapSize",
    "HeapFree",
    "HeapValidate",
    "malloc",
    "wcsrchr",
    "_wcsdup",
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
    "RegCloseKey",
    "RegOpenKeyExW",
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
    "DeviceIoControl",
    "CloseHandle",
    "CreateEventW",
    "EventModify",
    "WaitForSingleObject",
    "CreateMutexW",
    "ReleaseMutex",
    "CreateWindowExW",
    "DestroyWindow",
    "ShowWindow",
    "UpdateWindow",
    "GetParent",
    "IsWindow",
    "GetWindowTextLengthW",
    "GetClassNameW",
    "EnableWindow",
    "IsWindowEnabled",
    "GetDesktopWindow",
    "GetWindowTextWDirect",
    "SetFocus",
    "GetFocus",
    "IsWindowVisible",
    "SetWindowPos",
    "MoveWindow",
    "GetWindowRect",
    "GetClientRect",
    "ClientToScreen",
    "ScreenToClient",
    "MapWindowPoints",
    "SetWindowTextW",
    "GetWindowTextW",
    "GetWindowLongW",
    "SetWindowLongW",
    "GetMessageW",
    "PeekMessageW",
    "PostMessageW",
    "SendMessageW",
    "DispatchMessageW",
    "TranslateMessage",
    "DefWindowProcW",
    "SetTimer",
    "KillTimer",
    "GetTickCount",
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
