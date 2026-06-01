use std::{collections::BTreeMap, fs, path::Path};

use crate::{
    ce::{
        audio::{MmResult, WaveBuffer, WaveFormat},
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoredllExport {
    pub name: String,
    pub target: Option<String>,
    pub ordinal: u32,
    pub noname: bool,
    pub line: usize,
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
    OrdinalMismatch {
        ordinal: u32,
        export: CoredllExport,
        call_name: &'static str,
    },
}

impl CoredllExportTable {
    pub fn from_core_common_def_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(Self::from_core_common_def(&text))
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
        match self.resolve_ordinal(ordinal).cloned() {
            Some(export) => CoredllDispatch::Unimplemented { export },
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

impl CoredllExport {
    fn matches_name(&self, name: &str) -> bool {
        normalize_name(&self.name) == normalize_name(name)
            || self
                .target
                .as_deref()
                .is_some_and(|target| normalize_name(target) == normalize_name(name))
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
