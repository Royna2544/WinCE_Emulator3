use std::collections::{BTreeMap, VecDeque};
#[cfg(windows)]
use std::mem::size_of;

pub type MmResult = u32;

pub const MMSYSERR_NOERROR: MmResult = 0;
pub const MMSYSERR_ERROR: MmResult = 1;
pub const MMSYSERR_BADDEVICEID: MmResult = 2;
pub const MMSYSERR_INVALHANDLE: MmResult = 5;
pub const WAVERR_BADFORMAT: MmResult = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaveFormat {
    pub format_tag: u16,
    pub channels: u16,
    pub samples_per_sec: u32,
    pub avg_bytes_per_sec: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaveBuffer {
    pub guest_ptr: u32,
    pub len: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveOutCallback {
    Event(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioChunk {
    pub payload: Vec<u8>,
    pub sequence: u64,
    pub pts_ms: u64,
    pub duration_ms: u32,
    /// Transport hint: write and flush this chunk without waiting for batching.
    pub flush: bool,
}

impl AudioChunk {
    fn new(payload: Vec<u8>, sequence: u64, pts_ms: u64, duration_ms: u32, flush: bool) -> Self {
        Self {
            payload,
            sequence,
            pts_ms,
            duration_ms: duration_ms.max(1),
            flush,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioSinkSubmit {
    pub sink: String,
    pub sequence: Option<u64>,
    pub accepted: bool,
}

pub trait AudioSink {
    fn name(&self) -> &str;
    fn submit_pcm(
        &mut self,
        payload: &[u8],
        format: &WaveFormat,
        pts_ms: u64,
        duration_ms: u32,
        flush: bool,
    ) -> Option<u64>;
    fn flush(&mut self);
    fn queued_chunk_count(&self) -> usize;
}

pub struct AudioSinkRegistry {
    sinks: BTreeMap<String, Box<dyn AudioSink>>,
}

#[derive(Debug)]
pub struct HostAudioSink {
    name: String,
    backend: HostAudioBackend,
    connected: bool,
    submitted: VecDeque<AudioChunk>,
    max_chunks: usize,
}

#[derive(Debug)]
pub enum HostAudioBackend {
    Unplugged,
    #[cfg(windows)]
    Winmm(WinmmAudioBackend),
}

#[cfg(windows)]
pub struct WinmmAudioBackend {
    device_count: u32,
    output: Option<WinmmWaveOut>,
}

#[cfg(windows)]
struct WinmmWaveOut {
    handle: windows::Win32::Media::Audio::HWAVEOUT,
    format: WaveFormat,
    queued: VecDeque<WinmmQueuedBuffer>,
}

#[cfg(windows)]
struct WinmmQueuedBuffer {
    header: Box<windows::Win32::Media::Audio::WAVEHDR>,
    _payload: Box<[u8]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocketAudioSink {
    name: String,
    chunks: VecDeque<AudioChunk>,
    clients: BTreeMap<u64, WebSocketAudioCursor>,
    next_client_id: u64,
    sequence: u64,
    next_pts_ms: u64,
    max_chunks: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WebSocketAudioCursor {
    next_sequence: u64,
    trim_before_ms: Option<u64>,
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoggingAudioSink {
    name: String,
    events: VecDeque<LoggingAudioEvent>,
    max_events: usize,
    sequence: u64,
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoggingAudioEvent {
    pub sequence: u64,
    pub bytes: usize,
    pub pts_ms: u64,
    pub duration_ms: u32,
    pub flush: bool,
}

#[derive(Debug, Clone)]
pub struct WaveOutDevice {
    pub id: u32,
    pub format: WaveFormat,
    pub callback: Option<WaveOutCallback>,
    pub volume: u32,
    pub pitch: u32,
    pub playback_rate: u32,
    pub submitted_bytes: u32,
    pub state: WaveOutState,
    pending: VecDeque<WaveBuffer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveOutState {
    Open,
    Playing,
    Paused,
    Reset,
    Closed,
}

#[derive(Debug)]
pub struct AudioSystem {
    next_id: u32,
    outputs: BTreeMap<u32, WaveOutDevice>,
    sinks: AudioSinkRegistry,
    next_sink_pts_ms: u64,
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self {
            next_id: 1,
            outputs: BTreeMap::new(),
            sinks: AudioSinkRegistry::default(),
            next_sink_pts_ms: 0,
        }
    }
}

impl Clone for AudioSystem {
    fn clone(&self) -> Self {
        Self {
            next_id: self.next_id,
            outputs: self.outputs.clone(),
            sinks: AudioSinkRegistry::default(),
            next_sink_pts_ms: self.next_sink_pts_ms,
        }
    }
}

impl AudioSystem {
    pub fn open_wave_out(&mut self, format: WaveFormat) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.outputs.insert(
            id,
            WaveOutDevice {
                id,
                format,
                callback: None,
                volume: 0xffff_ffff,
                pitch: 0x0001_0000,
                playback_rate: 0x0001_0000,
                submitted_bytes: 0,
                state: WaveOutState::Open,
                pending: VecDeque::new(),
            },
        );
        id
    }

    pub fn wave_out_get_num_devs(&self) -> u32 {
        1
    }

    pub fn wave_out_open(&mut self, format: WaveFormat) -> Result<u32, MmResult> {
        if !format.is_pcm() || format.block_align == 0 {
            return Err(WAVERR_BADFORMAT);
        }
        Ok(self.open_wave_out(format))
    }

    pub fn wave_out_write(&mut self, id: u32, buffer: WaveBuffer) -> MmResult {
        if self.write(id, buffer) {
            MMSYSERR_NOERROR
        } else {
            MMSYSERR_INVALHANDLE
        }
    }

    pub fn wave_out_write_pcm(&mut self, id: u32, buffer: WaveBuffer, payload: &[u8]) -> MmResult {
        let Some(format) = self.outputs.get(&id).map(|output| output.format.clone()) else {
            return MMSYSERR_INVALHANDLE;
        };
        if !self.write(id, buffer) {
            return MMSYSERR_INVALHANDLE;
        }
        self.submit_pcm_to_sinks(payload, &format, true);
        MMSYSERR_NOERROR
    }

    pub fn write(&mut self, id: u32, buffer: WaveBuffer) -> bool {
        let Some(output) = self.outputs.get_mut(&id) else {
            return false;
        };
        let len = buffer.len;
        output.pending.push_back(buffer);
        output.submitted_bytes = output.submitted_bytes.saturating_add(len);
        output.state = WaveOutState::Playing;
        true
    }

    pub fn complete_next_buffer(&mut self, id: u32) -> Option<WaveBuffer> {
        let output = self.outputs.get_mut(&id)?;
        let buffer = output.pending.pop_front();
        if output.pending.is_empty() && output.state == WaveOutState::Playing {
            output.state = WaveOutState::Open;
        }
        buffer
    }

    pub fn reset(&mut self, id: u32) -> bool {
        let Some(output) = self.outputs.get_mut(&id) else {
            return false;
        };
        output.pending.clear();
        output.submitted_bytes = 0;
        output.state = WaveOutState::Reset;
        true
    }

    pub fn pause(&mut self, id: u32) -> MmResult {
        let Some(output) = self.outputs.get_mut(&id) else {
            return MMSYSERR_INVALHANDLE;
        };
        output.state = WaveOutState::Paused;
        MMSYSERR_NOERROR
    }

    pub fn restart(&mut self, id: u32) -> MmResult {
        let Some(output) = self.outputs.get_mut(&id) else {
            return MMSYSERR_INVALHANDLE;
        };
        if !output.pending.is_empty() {
            output.state = WaveOutState::Playing;
        }
        MMSYSERR_NOERROR
    }

    pub fn wave_out_reset(&mut self, id: u32) -> MmResult {
        if self.reset(id) {
            MMSYSERR_NOERROR
        } else {
            MMSYSERR_INVALHANDLE
        }
    }

    pub fn set_volume(&mut self, id: u32, volume: u32) -> bool {
        let Some(output) = self.outputs.get_mut(&id) else {
            return false;
        };
        output.volume = volume;
        true
    }

    pub fn get_volume(&self, id: u32) -> Result<u32, MmResult> {
        self.outputs
            .get(&id)
            .map(|output| output.volume)
            .ok_or(MMSYSERR_INVALHANDLE)
    }

    pub fn get_position_bytes(&self, id: u32) -> Result<u32, MmResult> {
        self.outputs
            .get(&id)
            .map(|output| output.submitted_bytes)
            .ok_or(MMSYSERR_INVALHANDLE)
    }

    pub fn set_pitch(&mut self, id: u32, pitch: u32) -> MmResult {
        let Some(output) = self.outputs.get_mut(&id) else {
            return MMSYSERR_INVALHANDLE;
        };
        output.pitch = pitch;
        MMSYSERR_NOERROR
    }

    pub fn get_pitch(&self, id: u32) -> Result<u32, MmResult> {
        self.outputs
            .get(&id)
            .map(|output| output.pitch)
            .ok_or(MMSYSERR_INVALHANDLE)
    }

    pub fn set_playback_rate(&mut self, id: u32, rate: u32) -> MmResult {
        let Some(output) = self.outputs.get_mut(&id) else {
            return MMSYSERR_INVALHANDLE;
        };
        output.playback_rate = rate;
        MMSYSERR_NOERROR
    }

    pub fn get_playback_rate(&self, id: u32) -> Result<u32, MmResult> {
        self.outputs
            .get(&id)
            .map(|output| output.playback_rate)
            .ok_or(MMSYSERR_INVALHANDLE)
    }

    pub fn wave_out_set_volume(&mut self, id: u32, volume: u32) -> MmResult {
        if self.set_volume(id, volume) {
            MMSYSERR_NOERROR
        } else {
            MMSYSERR_INVALHANDLE
        }
    }

    pub fn close(&mut self, id: u32) -> bool {
        let Some(output) = self.outputs.get_mut(&id) else {
            return false;
        };
        output.pending.clear();
        output.state = WaveOutState::Closed;
        true
    }

    pub fn wave_out_close(&mut self, id: u32) -> MmResult {
        if self.close(id) {
            MMSYSERR_NOERROR
        } else {
            MMSYSERR_INVALHANDLE
        }
    }

    pub fn output(&self, id: u32) -> Option<&WaveOutDevice> {
        self.outputs.get(&id)
    }

    pub fn set_wave_out_callback(&mut self, id: u32, callback: Option<WaveOutCallback>) -> bool {
        let Some(output) = self.outputs.get_mut(&id) else {
            return false;
        };
        output.callback = callback;
        true
    }

    pub fn register_sink<S>(&mut self, sink: S) -> bool
    where
        S: AudioSink + 'static,
    {
        self.sinks.register(sink)
    }

    pub fn unregister_sink(&mut self, name: &str) -> bool {
        self.sinks.unregister(name)
    }

    pub fn has_sinks(&self) -> bool {
        !self.sinks.is_empty()
    }

    pub fn sink_names(&self) -> Vec<String> {
        self.sinks.sink_names()
    }

    pub fn queued_sink_chunk_count(&self, name: &str) -> Option<usize> {
        self.sinks.queued_chunk_count(name)
    }

    pub fn flush_sinks(&mut self) {
        self.sinks.flush_all();
    }

    fn submit_pcm_to_sinks(
        &mut self,
        payload: &[u8],
        format: &WaveFormat,
        flush: bool,
    ) -> Vec<AudioSinkSubmit> {
        if payload.is_empty() || self.sinks.is_empty() {
            return Vec::new();
        }
        let duration_ms = format.duration_ms_for_bytes(payload.len());
        let pts_ms = self.next_sink_pts_ms;
        self.next_sink_pts_ms = self.next_sink_pts_ms.saturating_add(u64::from(duration_ms));
        self.sinks
            .submit_pcm(payload.to_vec(), format, pts_ms, duration_ms, flush)
    }
}

impl Default for AudioSinkRegistry {
    fn default() -> Self {
        Self {
            sinks: BTreeMap::new(),
        }
    }
}

impl std::fmt::Debug for AudioSinkRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioSinkRegistry")
            .field("sinks", &self.sink_names())
            .finish()
    }
}

impl AudioSinkRegistry {
    pub fn register<S>(&mut self, sink: S) -> bool
    where
        S: AudioSink + 'static,
    {
        let name = sink.name().to_owned();
        if self.sinks.contains_key(&name) {
            return false;
        }
        self.sinks.insert(name, Box::new(sink));
        true
    }

    pub fn unregister(&mut self, name: &str) -> bool {
        self.sinks.remove(name).is_some()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.sinks.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.sinks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sinks.is_empty()
    }

    pub fn sink_names(&self) -> Vec<String> {
        self.sinks.keys().cloned().collect()
    }

    pub fn submit_pcm(
        &mut self,
        payload: Vec<u8>,
        format: &WaveFormat,
        pts_ms: u64,
        duration_ms: u32,
        flush: bool,
    ) -> Vec<AudioSinkSubmit> {
        self.sinks
            .values_mut()
            .map(|sink| {
                let sequence = sink.submit_pcm(&payload, format, pts_ms, duration_ms, flush);
                AudioSinkSubmit {
                    sink: sink.name().to_owned(),
                    accepted: sequence.is_some(),
                    sequence,
                }
            })
            .collect()
    }

    pub fn flush_all(&mut self) {
        for sink in self.sinks.values_mut() {
            sink.flush();
        }
    }

    pub fn queued_chunk_count(&self, name: &str) -> Option<usize> {
        self.sinks.get(name).map(|sink| sink.queued_chunk_count())
    }
}

impl HostAudioSink {
    pub fn unplugged(max_chunks: usize) -> Self {
        Self {
            name: "host".to_owned(),
            backend: HostAudioBackend::Unplugged,
            connected: false,
            submitted: VecDeque::new(),
            max_chunks: max_chunks.max(1),
        }
    }

    pub fn named_unplugged(name: impl Into<String>, max_chunks: usize) -> Self {
        Self {
            name: name.into(),
            backend: HostAudioBackend::Unplugged,
            connected: false,
            submitted: VecDeque::new(),
            max_chunks: max_chunks.max(1),
        }
    }

    #[cfg(windows)]
    pub fn winmm(name: impl Into<String>, max_chunks: usize) -> Self {
        let device_count = unsafe { windows::Win32::Media::Audio::waveOutGetNumDevs() };
        Self {
            name: name.into(),
            backend: HostAudioBackend::Winmm(WinmmAudioBackend::new(device_count)),
            connected: device_count > 0,
            submitted: VecDeque::new(),
            max_chunks: max_chunks.max(1),
        }
    }

    pub fn backend(&self) -> &HostAudioBackend {
        &self.backend
    }

    pub fn connect(&mut self) {
        self.connected = true;
    }

    pub fn unplug(&mut self) {
        self.connected = false;
        self.backend.unplug();
        self.submitted.clear();
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn submit_pcm(&mut self, payload: Vec<u8>, pts_ms: u64, duration_ms: u32) -> Option<u64> {
        self.submit_pcm_with_format(
            payload,
            &WaveFormat::pcm_16bit(2, 44_100),
            pts_ms,
            duration_ms,
        )
    }

    pub fn submit_pcm_with_format(
        &mut self,
        payload: Vec<u8>,
        format: &WaveFormat,
        pts_ms: u64,
        duration_ms: u32,
    ) -> Option<u64> {
        self.submit_pcm_with_flush(payload, format, pts_ms, duration_ms, true)
    }

    fn submit_pcm_with_flush(
        &mut self,
        payload: Vec<u8>,
        format: &WaveFormat,
        pts_ms: u64,
        duration_ms: u32,
        flush: bool,
    ) -> Option<u64> {
        if !self.connected || payload.is_empty() {
            return None;
        }
        if !self.backend.submit_pcm(&payload, format) {
            return None;
        }
        let sequence = self
            .submitted
            .back()
            .map(|chunk| chunk.sequence.saturating_add(1))
            .unwrap_or(1);
        self.submitted.push_back(AudioChunk::new(
            payload,
            sequence,
            pts_ms,
            duration_ms,
            flush,
        ));
        while self.submitted.len() > self.max_chunks {
            self.submitted.pop_front();
        }
        Some(sequence)
    }

    pub fn take_chunks(&mut self, max_chunks: usize) -> Vec<AudioChunk> {
        let count = max_chunks.min(self.submitted.len());
        self.submitted.drain(..count).collect()
    }

    pub fn queued_chunk_count(&self) -> usize {
        self.submitted.len()
    }
}

impl HostAudioBackend {
    #[cfg(windows)]
    pub fn device_count(&self) -> Option<u32> {
        match self {
            HostAudioBackend::Unplugged => None,
            HostAudioBackend::Winmm(backend) => Some(backend.device_count()),
        }
    }

    fn submit_pcm(&mut self, payload: &[u8], format: &WaveFormat) -> bool {
        match self {
            HostAudioBackend::Unplugged => true,
            #[cfg(windows)]
            HostAudioBackend::Winmm(backend) => backend.submit_pcm(payload, format),
        }
    }

    fn unplug(&mut self) {
        match self {
            HostAudioBackend::Unplugged => {}
            #[cfg(windows)]
            HostAudioBackend::Winmm(backend) => backend.close(),
        }
    }

    fn flush(&mut self) {
        match self {
            HostAudioBackend::Unplugged => {}
            #[cfg(windows)]
            HostAudioBackend::Winmm(backend) => backend.reclaim_done(),
        }
    }
}

#[cfg(windows)]
impl std::fmt::Debug for WinmmAudioBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WinmmAudioBackend")
            .field("device_count", &self.device_count)
            .field("open", &self.output.is_some())
            .field(
                "queued",
                &self
                    .output
                    .as_ref()
                    .map(|output| output.queued.len())
                    .unwrap_or(0),
            )
            .finish()
    }
}

#[cfg(windows)]
impl WinmmAudioBackend {
    fn new(device_count: u32) -> Self {
        Self {
            device_count,
            output: None,
        }
    }

    pub fn device_count(&self) -> u32 {
        self.device_count
    }

    fn submit_pcm(&mut self, payload: &[u8], format: &WaveFormat) -> bool {
        if self.device_count == 0 || payload.is_empty() {
            return false;
        }
        let format = format.clone();
        if self
            .output
            .as_ref()
            .is_none_or(|output| output.format != format)
        {
            self.close();
            let Some(output) = WinmmWaveOut::open(format) else {
                return false;
            };
            self.output = Some(output);
        }
        let Some(output) = self.output.as_mut() else {
            return false;
        };
        output.submit_pcm(payload)
    }

    fn reclaim_done(&mut self) {
        if let Some(output) = self.output.as_mut() {
            output.reclaim_done();
        }
    }

    fn close(&mut self) {
        self.output.take();
    }
}

#[cfg(windows)]
impl std::fmt::Debug for WinmmWaveOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WinmmWaveOut")
            .field("format", &self.format)
            .field("queued", &self.queued.len())
            .finish()
    }
}

#[cfg(windows)]
impl WinmmWaveOut {
    fn open(format: WaveFormat) -> Option<Self> {
        use windows::Win32::Media::Audio::{CALLBACK_NULL, HWAVEOUT, WAVE_MAPPER, WAVEFORMATEX};

        let native_format = WAVEFORMATEX {
            wFormatTag: format.format_tag,
            nChannels: format.channels,
            nSamplesPerSec: format.samples_per_sec,
            nAvgBytesPerSec: format.avg_bytes_per_sec,
            nBlockAlign: format.block_align,
            wBitsPerSample: format.bits_per_sample,
            cbSize: 0,
        };
        let mut handle = HWAVEOUT::default();
        let result = unsafe {
            windows::Win32::Media::Audio::waveOutOpen(
                Some(&mut handle),
                WAVE_MAPPER,
                &native_format,
                0,
                0,
                CALLBACK_NULL,
            )
        };
        if result != MMSYSERR_NOERROR || handle.is_invalid() {
            tracing::warn!(
                target: "ce.audio",
                result,
                "failed to open host winmm waveOut"
            );
            return None;
        }
        Some(Self {
            handle,
            format,
            queued: VecDeque::new(),
        })
    }

    fn submit_pcm(&mut self, payload: &[u8]) -> bool {
        use windows::{
            Win32::Media::Audio::{WAVEHDR, waveOutPrepareHeader, waveOutWrite},
            core::PSTR,
        };

        self.reclaim_done();

        let mut payload = payload.to_vec().into_boxed_slice();
        let mut queued = WinmmQueuedBuffer {
            header: Box::new(WAVEHDR {
                lpData: PSTR(payload.as_mut_ptr()),
                dwBufferLength: payload.len().min(u32::MAX as usize) as u32,
                dwBytesRecorded: 0,
                dwUser: 0,
                dwFlags: 0,
                dwLoops: 0,
                lpNext: std::ptr::null_mut(),
                reserved: 0,
            }),
            _payload: payload,
        };
        let header = queued.header.as_mut() as *mut WAVEHDR;
        let header_size = size_of::<WAVEHDR>() as u32;
        let prepare = unsafe { waveOutPrepareHeader(self.handle, header, header_size) };
        if prepare != MMSYSERR_NOERROR {
            tracing::warn!(
                target: "ce.audio",
                result = prepare,
                "failed to prepare host winmm waveOut buffer"
            );
            return false;
        }
        let write = unsafe { waveOutWrite(self.handle, header, header_size) };
        if write != MMSYSERR_NOERROR {
            unsafe {
                let _ = windows::Win32::Media::Audio::waveOutUnprepareHeader(
                    self.handle,
                    header,
                    header_size,
                );
            }
            tracing::warn!(
                target: "ce.audio",
                result = write,
                "failed to write host winmm waveOut buffer"
            );
            return false;
        }
        self.queued.push_back(queued);
        self.trim_backlog();
        true
    }

    fn reclaim_done(&mut self) {
        while self.queued.front().is_some_and(WinmmQueuedBuffer::is_done) {
            if let Some(mut done) = self.queued.pop_front() {
                done.unprepare(self.handle);
            }
        }
    }

    fn trim_backlog(&mut self) {
        const MAX_WINMM_BUFFERS: usize = 16;
        if self.queued.len() <= MAX_WINMM_BUFFERS {
            return;
        }
        unsafe {
            let _ = windows::Win32::Media::Audio::waveOutReset(self.handle);
        }
        while let Some(mut queued) = self.queued.pop_front() {
            queued.unprepare(self.handle);
        }
    }
}

#[cfg(windows)]
impl Drop for WinmmWaveOut {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::Media::Audio::waveOutReset(self.handle);
        }
        while let Some(mut queued) = self.queued.pop_front() {
            queued.unprepare(self.handle);
        }
        unsafe {
            let _ = windows::Win32::Media::Audio::waveOutClose(self.handle);
        }
    }
}

#[cfg(windows)]
impl WinmmQueuedBuffer {
    fn is_done(&self) -> bool {
        let flags = unsafe { std::ptr::addr_of!((*self.header).dwFlags).read_unaligned() };
        flags & windows::Win32::Media::Audio::WHDR_DONE != 0
    }

    fn unprepare(&mut self, handle: windows::Win32::Media::Audio::HWAVEOUT) {
        unsafe {
            let _ = windows::Win32::Media::Audio::waveOutUnprepareHeader(
                handle,
                self.header.as_mut() as *mut windows::Win32::Media::Audio::WAVEHDR,
                size_of::<windows::Win32::Media::Audio::WAVEHDR>() as u32,
            );
        }
    }
}

impl AudioSink for HostAudioSink {
    fn name(&self) -> &str {
        &self.name
    }

    fn submit_pcm(
        &mut self,
        payload: &[u8],
        format: &WaveFormat,
        pts_ms: u64,
        duration_ms: u32,
        flush: bool,
    ) -> Option<u64> {
        self.submit_pcm_with_flush(payload.to_vec(), format, pts_ms, duration_ms, flush)
    }

    fn flush(&mut self) {
        self.backend.flush();
        if let Some(chunk) = self.submitted.back() {
            tracing::debug!(
                target: "ce.audio",
                sink = self.name.as_str(),
                backend = ?self.backend,
                sequence = chunk.sequence,
                bytes = chunk.payload.len(),
                "host audio sink flush"
            );
        }
    }

    fn queued_chunk_count(&self) -> usize {
        HostAudioSink::queued_chunk_count(self)
    }
}

impl WebSocketAudioSink {
    pub fn new(max_chunks: usize) -> Self {
        Self::named("websocket", max_chunks)
    }

    pub fn named(name: impl Into<String>, max_chunks: usize) -> Self {
        Self {
            name: name.into(),
            chunks: VecDeque::new(),
            clients: BTreeMap::new(),
            next_client_id: 1,
            sequence: 0,
            next_pts_ms: 0,
            max_chunks: max_chunks.max(1),
        }
    }

    pub fn register_client(&mut self, now_ms: u64) -> u64 {
        if self.chunks.is_empty() {
            self.next_pts_ms = now_ms;
        }
        let client_id = self.next_client_id;
        self.next_client_id = self.next_client_id.saturating_add(1);
        self.clients
            .insert(client_id, self.cursor_for_host_time(now_ms));
        client_id
    }

    pub fn unregister_client(&mut self, client_id: u64) -> bool {
        let removed = self.clients.remove(&client_id).is_some();
        if self.clients.is_empty() {
            self.clients.clear();
        }
        removed
    }

    pub fn unregister_latest_client(&mut self) -> bool {
        let Some(client_id) = self.clients.keys().next_back().copied() else {
            return false;
        };
        self.unregister_client(client_id)
    }

    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    pub fn publish_pcm(&mut self, payload: Vec<u8>, duration_ms: u32) -> Option<u64> {
        self.publish_pcm_with_flush(payload, duration_ms, true)
    }

    pub fn publish_pcm_with_flush(
        &mut self,
        payload: Vec<u8>,
        duration_ms: u32,
        flush: bool,
    ) -> Option<u64> {
        if payload.is_empty() {
            return None;
        }
        self.sequence += 1;
        let sequence = self.sequence;
        let duration_ms = duration_ms.max(1);
        self.chunks.push_back(AudioChunk::new(
            payload,
            sequence,
            self.next_pts_ms,
            duration_ms,
            flush,
        ));
        self.next_pts_ms = self.next_pts_ms.saturating_add(u64::from(duration_ms));
        while self.chunks.len() > self.max_chunks {
            self.chunks.pop_front();
        }
        Some(sequence)
    }

    pub fn publish_pcm_at(
        &mut self,
        payload: Vec<u8>,
        pts_ms: u64,
        duration_ms: u32,
        flush: bool,
    ) -> Option<u64> {
        if payload.is_empty() {
            return None;
        }
        self.sequence += 1;
        let sequence = self.sequence;
        let duration_ms = duration_ms.max(1);
        self.chunks.push_back(AudioChunk::new(
            payload,
            sequence,
            pts_ms,
            duration_ms,
            flush,
        ));
        self.next_pts_ms = self
            .next_pts_ms
            .max(pts_ms.saturating_add(u64::from(duration_ms)));
        while self.chunks.len() > self.max_chunks {
            self.chunks.pop_front();
        }
        Some(sequence)
    }

    pub fn clear(&mut self, now_ms: u64) {
        self.chunks.clear();
        self.next_pts_ms = now_ms;
    }

    pub fn take_chunks_for_client(&mut self, client_id: u64, max_chunks: usize) -> Vec<AudioChunk> {
        let Some(mut cursor) = self.clients.get(&client_id).copied() else {
            return Vec::new();
        };
        let mut chunks = Vec::new();
        for chunk in self
            .chunks
            .iter()
            .filter(|chunk| chunk.sequence >= cursor.next_sequence)
        {
            if chunks.len() >= max_chunks {
                break;
            }
            let next = if chunk.sequence == cursor.next_sequence {
                match cursor.trim_before_ms.take() {
                    Some(sync_ms) => trim_chunk_to_host_time(chunk, sync_ms),
                    None => Some(chunk.clone()),
                }
            } else {
                cursor.trim_before_ms = None;
                Some(chunk.clone())
            };
            if let Some(next) = next {
                chunks.push(next);
            }
        }
        if let Some(last) = chunks.last() {
            cursor.next_sequence = last.sequence.saturating_add(1);
            cursor.trim_before_ms = None;
        }
        self.clients.insert(client_id, cursor);
        chunks
    }

    pub fn take_chunks(&mut self, max_chunks: usize) -> Vec<AudioChunk> {
        let Some(client_id) = self.clients.keys().next().copied() else {
            return Vec::new();
        };
        self.take_chunks_for_client(client_id, max_chunks)
    }

    pub fn queued_chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn needs_flush_for_client(&self, client_id: u64) -> bool {
        let Some(cursor) = self.clients.get(&client_id) else {
            return false;
        };
        self.chunks
            .iter()
            .any(|chunk| chunk.sequence >= cursor.next_sequence && chunk.flush)
    }

    fn cursor_for_host_time(&self, now_ms: u64) -> WebSocketAudioCursor {
        let Some(chunk) = self
            .chunks
            .iter()
            .find(|chunk| chunk_end_ms(chunk) > now_ms)
        else {
            return WebSocketAudioCursor {
                next_sequence: self.sequence.saturating_add(1),
                trim_before_ms: None,
            };
        };

        WebSocketAudioCursor {
            next_sequence: chunk.sequence,
            trim_before_ms: (now_ms > chunk.pts_ms).then_some(now_ms),
        }
    }
}

impl AudioSink for WebSocketAudioSink {
    fn name(&self) -> &str {
        &self.name
    }

    fn submit_pcm(
        &mut self,
        payload: &[u8],
        _format: &WaveFormat,
        pts_ms: u64,
        duration_ms: u32,
        flush: bool,
    ) -> Option<u64> {
        self.publish_pcm_at(payload.to_vec(), pts_ms, duration_ms, flush)
    }

    fn flush(&mut self) {}

    fn queued_chunk_count(&self) -> usize {
        WebSocketAudioSink::queued_chunk_count(self)
    }
}

#[cfg(debug_assertions)]
impl LoggingAudioSink {
    pub fn new(name: impl Into<String>, max_events: usize) -> Self {
        Self {
            name: name.into(),
            events: VecDeque::new(),
            max_events: max_events.max(1),
            sequence: 0,
        }
    }

    pub fn events(&self) -> impl Iterator<Item = &LoggingAudioEvent> {
        self.events.iter()
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

#[cfg(debug_assertions)]
impl AudioSink for LoggingAudioSink {
    fn name(&self) -> &str {
        &self.name
    }

    fn submit_pcm(
        &mut self,
        payload: &[u8],
        _format: &WaveFormat,
        pts_ms: u64,
        duration_ms: u32,
        flush: bool,
    ) -> Option<u64> {
        if payload.is_empty() {
            return None;
        }
        self.sequence = self.sequence.saturating_add(1);
        let event = LoggingAudioEvent {
            sequence: self.sequence,
            bytes: payload.len(),
            pts_ms,
            duration_ms: duration_ms.max(1),
            flush,
        };
        tracing::debug!(
            target: "ce.audio",
            sink = self.name.as_str(),
            sequence = event.sequence,
            bytes = event.bytes,
            pts_ms = event.pts_ms,
            duration_ms = event.duration_ms,
            flush = event.flush,
            "audio sink submit"
        );
        self.events.push_back(event);
        while self.events.len() > self.max_events {
            self.events.pop_front();
        }
        Some(self.sequence)
    }

    fn flush(&mut self) {
        tracing::debug!(
            target: "ce.audio",
            sink = self.name.as_str(),
            events = self.events.len(),
            "audio sink flush"
        );
    }

    fn queued_chunk_count(&self) -> usize {
        self.events.len()
    }
}

fn chunk_end_ms(chunk: &AudioChunk) -> u64 {
    chunk.pts_ms.saturating_add(u64::from(chunk.duration_ms))
}

fn trim_chunk_to_host_time(chunk: &AudioChunk, sync_ms: u64) -> Option<AudioChunk> {
    let end_ms = chunk_end_ms(chunk);
    if sync_ms <= chunk.pts_ms {
        return Some(chunk.clone());
    }
    if sync_ms >= end_ms || chunk.duration_ms == 0 {
        return None;
    }

    let elapsed_ms = sync_ms.saturating_sub(chunk.pts_ms);
    let offset = ((chunk.payload.len() as u128 * elapsed_ms as u128)
        / u128::from(chunk.duration_ms)) as usize;
    if offset >= chunk.payload.len() {
        return None;
    }

    Some(AudioChunk {
        payload: chunk.payload[offset..].to_vec(),
        sequence: chunk.sequence,
        pts_ms: sync_ms,
        duration_ms: end_ms.saturating_sub(sync_ms).min(u64::from(u32::MAX)) as u32,
        flush: chunk.flush,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wave_write_tracks_pending_buffer() {
        let mut audio = AudioSystem::default();
        let id = audio.open_wave_out(WaveFormat::pcm_16bit(2, 44_100));

        assert!(audio.write(
            id,
            WaveBuffer {
                guest_ptr: 0x1000,
                len: 4096
            }
        ));
        assert_eq!(audio.output(id).unwrap().state, WaveOutState::Playing);
        assert_eq!(audio.complete_next_buffer(id).unwrap().guest_ptr, 0x1000);
    }

    #[test]
    fn host_sink_is_unplugged_until_connected() {
        let mut sink = HostAudioSink::unplugged(2);

        assert_eq!(sink.submit_pcm(vec![1, 2], 10, 0), None);
        sink.connect();
        assert_eq!(sink.submit_pcm(vec![1, 2], 10, 0), Some(1));
        assert_eq!(sink.submit_pcm(vec![3], 11, 1), Some(2));
        assert_eq!(sink.submit_pcm(vec![4], 12, 1), Some(3));
        assert_eq!(sink.queued_chunk_count(), 2);
        assert_eq!(sink.take_chunks(1)[0].sequence, 2);
        sink.unplug();
        assert!(!sink.is_connected());
        assert_eq!(sink.queued_chunk_count(), 0);
    }

    #[test]
    fn websocket_sink_retains_host_timeline_without_clients() {
        let mut sink = WebSocketAudioSink::new(2);

        assert_eq!(sink.publish_pcm(vec![1], 20), Some(1));
        assert_eq!(sink.publish_pcm(vec![2], 0), Some(2));
        assert_eq!(sink.publish_pcm(vec![3], 30), Some(3));
        assert_eq!(sink.queued_chunk_count(), 2);
        let client_id = sink.register_client(21);
        assert!(sink.needs_flush_for_client(client_id));
        let chunks = sink.take_chunks_for_client(client_id, 2);
        assert_eq!(chunks[0].sequence, 3);
        assert_eq!(chunks[0].pts_ms, 21);
        assert_eq!(chunks[0].duration_ms, 30);
        assert!(chunks[0].flush);
        assert!(sink.take_chunks_for_client(client_id, 2).is_empty());
    }

    #[test]
    fn websocket_sink_late_join_gets_partial_chunk_at_host_time() {
        let mut sink = WebSocketAudioSink::new(4);

        let early_client = sink.register_client(1000);
        assert_eq!(sink.publish_pcm(vec![0, 1, 2, 3, 4, 5, 6, 7], 40), Some(1));
        let client_id = sink.register_client(1010);
        assert!(sink.needs_flush_for_client(early_client));
        assert!(sink.needs_flush_for_client(client_id));
        let chunks = sink.take_chunks_for_client(client_id, 1);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].sequence, 1);
        assert_eq!(chunks[0].pts_ms, 1010);
        assert_eq!(chunks[0].duration_ms, 30);
        assert_eq!(chunks[0].payload, vec![2, 3, 4, 5, 6, 7]);
        assert!(chunks[0].flush);
    }

    #[test]
    fn audio_sink_registry_fans_out_registered_sinks() {
        let mut registry = AudioSinkRegistry::default();
        let mut host = HostAudioSink::named_unplugged("host-debug", 2);
        host.connect();

        assert!(registry.register(host));
        assert!(registry.register(WebSocketAudioSink::named("remote", 2)));
        assert!(!registry.register(HostAudioSink::named_unplugged("remote", 2)));

        let format = WaveFormat::pcm_16bit(2, 44_100);
        let results = registry.submit_pcm(vec![1, 2, 3, 4], &format, 100, 20, true);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|result| result.accepted));
        assert_eq!(registry.queued_chunk_count("host-debug"), Some(1));
        assert_eq!(registry.queued_chunk_count("remote"), Some(1));
        registry.flush_all();
    }

    #[test]
    fn wave_write_pcm_fans_out_to_registered_host_sink() {
        let mut audio = AudioSystem::default();
        let mut host = HostAudioSink::named_unplugged("host-test", 2);
        host.connect();
        assert!(audio.register_sink(host));

        let id = audio.open_wave_out(WaveFormat::pcm_16bit(2, 44_100));
        assert_eq!(
            audio.wave_out_write_pcm(
                id,
                WaveBuffer {
                    guest_ptr: 0x4000,
                    len: 8
                },
                &[0, 1, 2, 3, 4, 5, 6, 7]
            ),
            MMSYSERR_NOERROR
        );

        assert_eq!(audio.output(id).unwrap().submitted_bytes, 8);
        assert_eq!(audio.queued_sink_chunk_count("host-test"), Some(1));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn logging_audio_sink_records_debug_events() {
        let mut sink = LoggingAudioSink::new("log", 2);

        let format = WaveFormat::pcm_16bit(2, 44_100);
        assert_eq!(sink.submit_pcm(&[1, 2], &format, 10, 0, true), Some(1));
        assert_eq!(sink.submit_pcm(&[3, 4], &format, 11, 1, false), Some(2));
        assert_eq!(sink.submit_pcm(&[5, 6], &format, 12, 1, true), Some(3));
        let events = sink.events().cloned().collect::<Vec<_>>();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].sequence, 2);
        assert!(!events[0].flush);
        assert!(events[1].flush);
    }
}

impl WaveFormat {
    pub fn pcm_16bit(channels: u16, samples_per_sec: u32) -> Self {
        let block_align = channels * 2;
        Self {
            format_tag: 1,
            channels,
            samples_per_sec,
            avg_bytes_per_sec: samples_per_sec * u32::from(block_align),
            block_align,
            bits_per_sample: 16,
        }
    }

    fn is_pcm(&self) -> bool {
        self.format_tag == 1
    }

    fn duration_ms_for_bytes(&self, bytes: usize) -> u32 {
        if bytes == 0 || self.avg_bytes_per_sec == 0 {
            return 1;
        }
        let duration = ((bytes as u128 * 1000) + u128::from(self.avg_bytes_per_sec) - 1)
            / u128::from(self.avg_bytes_per_sec);
        duration.clamp(1, u128::from(u32::MAX)) as u32
    }
}
