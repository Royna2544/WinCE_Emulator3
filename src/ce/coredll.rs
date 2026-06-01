use std::{collections::BTreeMap, fs, path::Path};

use crate::{
    ce::{
        audio::{MmResult, WaveBuffer, WaveFormat},
        cemath::{CeMathCall, CeMathValue},
        devices::DeviceIoControlResult,
        file::FileIoResult,
        gwe::{Message, PeekFlags},
        kernel::{CeKernel, MessagePumpResult},
        registry::{HKey, RegOpenResult, RegQueryValueResult},
    },
    error::{Error, Result},
};

pub const DEFAULT_CORE_COMMON_DEF: &str =
    "/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/CORE/DLL/core_common.def";
pub const DEFAULT_CRT_ORDINALS_H: &str =
    "/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/INC/crt_ordinals.h";

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

#[derive(Debug, Clone, Default)]
pub struct CoredllExportTable {
    exports: Vec<CoredllExport>,
    by_name: BTreeMap<String, usize>,
    by_ordinal: BTreeMap<u32, Vec<usize>>,
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
    pub fn from_core_common_def_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let mut table = Self::from_core_common_def(&text);
        if path.ends_with("core_common.def") {
            table.insert_cemath_exports();
        }
        Ok(table)
    }

    pub fn from_core_common_def(text: &str) -> Self {
        let mut table = Self::default();
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

    fn insert_cemath_exports(&mut self) {
        for (name, ordinal) in CEMATH_EXPORTS {
            if self.resolve_name(name).is_none() {
                self.insert(CoredllExport {
                    name: (*name).to_owned(),
                    target: None,
                    ordinal: *ordinal,
                    noname: false,
                    line: 0,
                });
            }
        }
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

const IMPLEMENTED_EXPORTS: &[&str] = &[
    "RegCloseKey",
    "RegOpenKeyExW",
    "RegQueryValueExW",
    "RegSetValueExW",
    "CreateFileW",
    "ReadFile",
    "WriteFile",
    "DeviceIoControl",
    "CloseHandle",
    "CreateEventW",
    "EventModify",
    "WaitForSingleObject",
    "CreateMutexW",
    "ReleaseMutex",
    "CreateWindowExW",
    "DestroyWindow",
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
