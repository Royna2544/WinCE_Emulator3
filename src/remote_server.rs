// Remote REST API v1
//
// This server intentionally tracks the v2 host tooling shape while sourcing all
// state from v3's generic CeRemote/RemoteStatus/framebuffer paths.
//
// Auth:
// - If RemoteServerConfig::token is set, every request must include
//   `Authorization: Bearer <token>`.
// - OPTIONS always returns 204 after auth is accepted.
//
// Discovery and status:
// - GET / returns JSON with the public endpoint list.
// - GET /api/v1/status returns v2-style camelCase JSON status:
//   running, guestWidth, guestHeight, guestFps, videoEnabled, videoCodec,
//   audioEnabled, audioCodec, audioSampleRate, audioChannels, audioFormat,
//   gpsEnabled, gpsTarget, paused, queuedTouchEvents, queuedKeyEvents,
//   queuedSerialBytes, audioClients, queuedAudioChunks, pendingControlMessages.
// - GET /status is a legacy alias for /api/v1/status.
//
// Frame/video:
// - GET /api/v1/frame.jpg[?quality=1..100] returns the latest framebuffer as
//   JPEG, or 503 {"ok":false,"error":"no framebuffer"} before the first frame.
// - GET /api/v1/debug/screenshot.png returns the latest framebuffer as PNG.
// - GET /api/v1/debug/summary.txt returns the latest emulator debug summary.
// - GET /api/v1/debug/windows.txt returns the latest GWE window snapshot text.
// - GET /api/v1/debug/messages.txt returns the latest GWE/message trace text.
// - GET /api/v1/debug/message-boxes.txt returns live MessageBoxW records.
// - GET /api/v1/debug/processes.txt returns recent process/thread ops.
// - GET /api/v1/debug/processes-live.txt returns live process/thread ops.
// - GET /api/v1/debug/sends.txt returns synchronous SendMessage state.
// - GET /api/v1/debug/pending-wndproc.txt returns pending WNDPROC callouts.
// - GET /api/v1/debug/wndproc.txt returns recent WNDPROC call/return traces.
// - GET /api/v1/debug/imports.txt returns recent import calls.
// - GET /api/v1/debug/milestones.txt returns recent milestone import calls.
// - GET /api/v1/debug/counts.txt returns import call counts.
// - GET /api/v1/debug/calls.txt returns recent guest call targets.
// - GET /api/v1/debug/code.txt returns recent guest code samples.
// - GET /api/v1/debug/blocks.txt returns recent guest basic blocks.
// - GET /api/v1/debug/blobs.txt returns mapped guest blob ranges.
// - GET /api/v1/debug/trampolines.txt returns MIPS trampoline stub origins.
// - GET /api/v1/debug/files.txt returns recent file I/O summary.
// - GET /api/v1/debug/files-full.txt returns recent file I/O records.
// - GET /api/v1/debug/events.txt returns recent event operations.
// - GET /api/v1/debug/devices.txt returns recent device IOCTL operations.
// - GET /api/v1/debug/timers.txt returns live SetTimer/KillTimer state.
// - GET /api/v1/debug/render.txt returns display/controller render traces.
// - GET /api/v1/debug/controller.txt returns iNavi controller traces.
// - GET /api/v1/debug/resource.txt returns iNavi resource traces.
// - GET /api/v1/debug/active.txt returns live active process/thread state.
// - GET /api/v1/debug/parked.txt returns parked-process snapshots.
// - GET /api/v1/debug/remote-input.txt returns the latest remote input drain line.
// - GET /api/v1/video.mjpg[?fps=1..60][&quality=1..100] streams multipart
//   MJPEG frames until the client disconnects.
// - GET /framebuffer.ppm is a legacy/debug PPM framebuffer endpoint.
//
// Input:
// - POST /api/v1/input/touch accepts JSON
//   {"type":"touch","phase":"down|move|up|cancel|tap|click|single|single-touch","x":N,"y":N}
//   or the v2 shorthand {"type":"down|move|up|cancel|tap|click|single|single-touch","x":N,"y":N}.
//   It queues {"type":"touch","phase":phase,"x":N,"y":N} and returns {"ok":true}.
// - POST /api/v1/input/key accepts JSON {"type":"down|up","vk":1..255}.
//   It queues {"type":"key","phase":type,"vk":vk} and returns {"ok":true}.
//
// Sensors:
// - POST /api/v1/sensors/location accepts JSON containing numeric lat/lon
//   or latitude/longitude, queues it with type "location", and returns
//   {"ok":true,"sentencesGenerated":3}.
// - POST /api/v1/sensors/nmea accepts JSON {"sentences":[...]} and queues it
//   with type "nmea"; non-string entries are ignored in the accepted count.
// - POST /api/v1/sensors/imu accepts any JSON object, queues it with type
//   "imu" when parse succeeds, and still returns {"ok":true}.
//
// Logs/control:
// - GET /api/v1/logs/recent[?lines=1..4096] returns recent CeRemote log lines
//   as {"ok":true,"lines":[...]}.
// - POST /api/v1/control/pause queues {"type":"pause"} and returns paused true.
// - POST /api/v1/control/resume queues {"type":"resume"} and returns paused false.
// - GET /api/v1/control/ws upgrades to WebSocket. Text frames must be JSON
//   control messages. "status" and "logs" are answered directly; other valid
//   frames are queued unchanged and acknowledged with
//   {"ok":true,"queued":true,"pending":N}. Ping/pong/close are supported.
// - GET /api/v1/audio/ws upgrades to WebSocket. The server first sends a JSON
//   text metadata frame, then streams guest PCM chunks as binary frames when a
//   RemoteServer audio sink is registered with the CE audio system.
//
// Legacy control:
// - POST /control and POST /remote accept arbitrary JSON, queue it unchanged,
//   and return 202 {"ok":true,"queued":true,"pending":N}.

use std::{
    collections::{BTreeMap, VecDeque},
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex, mpsc},
    thread,
    time::{Duration, Instant},
};

use image::{ColorType, ImageEncoder, codecs::jpeg::JpegEncoder, codecs::png::PngEncoder};
use serde_json::{Value, json};

use crate::{
    ce::{
        audio::{AudioChunk, AudioSink, WaveFormat},
        framebuffer::{Framebuffer, PixelFormat},
        remote::RemoteStatus,
    },
    error::{Error, Result},
};

const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const MAX_PENDING_CONTROL_MESSAGES: usize = 1024;
const MAX_RECENT_LOG_LINES: usize = 4096;
const MAX_RECENT_LOG_LINE_BYTES: usize = 4096;
const DEFAULT_VIDEO_FPS: u32 = 30;
const DEFAULT_JPEG_QUALITY: u8 = 80;
const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

#[derive(Debug, Clone)]
pub struct RemoteServerConfig {
    pub addr: SocketAddr,
    pub token: Option<String>,
    pub video_fps: u32,
    pub jpeg_quality: u8,
    pub audio_enabled: bool,
    pub audio_sample_rate: u32,
    pub audio_channels: u16,
    pub audio_format: String,
}

impl Default for RemoteServerConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:8765".parse().expect("valid default remote addr"),
            token: None,
            video_fps: DEFAULT_VIDEO_FPS,
            jpeg_quality: DEFAULT_JPEG_QUALITY,
            audio_enabled: false,
            audio_sample_rate: 44_100,
            audio_channels: 2,
            audio_format: "s16le".to_owned(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RemoteServer {
    state: Arc<RemoteServerState>,
    local_addr: SocketAddr,
}

#[derive(Debug)]
struct RemoteServerState {
    config: RemoteServerConfig,
    listener: TcpListener,
    pending_control: Mutex<VecDeque<Value>>,
    latest_status: Mutex<Value>,
    latest_framebuffer: Mutex<Option<RemoteFramebufferImage>>,
    latest_debug: Mutex<BTreeMap<String, String>>,
    latest_debug_publish: Mutex<Option<Instant>>,
    recent_logs: Mutex<VecDeque<String>>,
    audio: Mutex<RemoteAudioState>,
}

#[derive(Debug, Clone)]
struct RemoteFramebufferImage {
    width: u32,
    height: u32,
    rgb: Vec<u8>,
}

#[derive(Debug)]
struct RemoteAudioState {
    chunks: VecDeque<AudioChunk>,
    clients: BTreeMap<u64, RemoteAudioCursor>,
    next_client_id: u64,
    sequence: u64,
    max_chunks: usize,
}

#[derive(Debug, Clone, Copy)]
struct RemoteAudioCursor {
    next_sequence: u64,
}

#[derive(Debug, Clone)]
pub struct RemoteAudioSink {
    state: Arc<RemoteServerState>,
}

impl RemoteServer {
    pub fn start(config: RemoteServerConfig) -> Result<Self> {
        let listener = TcpListener::bind(config.addr)
            .map_err(|err| Error::Backend(format!("bind remote server {}: {err}", config.addr)))?;
        listener.set_nonblocking(false).map_err(|err| {
            Error::Backend(format!("configure remote server {}: {err}", config.addr))
        })?;
        let local_addr = listener
            .local_addr()
            .map_err(|err| Error::Backend(format!("remote server local addr: {err}")))?;
        let server = Self {
            state: Arc::new(RemoteServerState {
                config,
                listener,
                pending_control: Mutex::new(VecDeque::new()),
                latest_status: Mutex::new(v2_status_json(&RemoteStatus {
                    running: true,
                    guest_width: 0,
                    guest_height: 0,
                    guest_fps: DEFAULT_VIDEO_FPS,
                    video_enabled: true,
                    video_codec: "mjpeg",
                    audio_enabled: false,
                    audio_codec: "pcm",
                    audio_sample_rate: 44_100,
                    audio_channels: 2,
                    audio_format: "s16le".to_owned(),
                    gps_enabled: true,
                    gps_target: String::new(),
                    paused: false,
                    queued_touch_events: 0,
                    queued_key_events: 0,
                    queued_serial_bytes: 0,
                    audio_clients: 0,
                    queued_audio_chunks: 0,
                })),
                latest_framebuffer: Mutex::new(None),
                latest_debug: Mutex::new(BTreeMap::new()),
                latest_debug_publish: Mutex::new(None),
                recent_logs: Mutex::new(VecDeque::new()),
                audio: Mutex::new(RemoteAudioState {
                    chunks: VecDeque::new(),
                    clients: BTreeMap::new(),
                    next_client_id: 1,
                    sequence: 0,
                    max_chunks: 16,
                }),
            }),
            local_addr,
        };
        let worker = server.clone();
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        thread::Builder::new()
            .name("wince-remote-server".to_owned())
            .spawn(move || {
                let _ = ready_tx.send(());
                worker.serve();
            })
            .map_err(|err| Error::Backend(format!("start remote server thread: {err}")))?;
        ready_rx
            .recv_timeout(Duration::from_secs(2))
            .map_err(|err| {
                Error::Backend(format!(
                    "remote server accept thread did not start for {local_addr}: {err}"
                ))
            })?;
        wait_until_http_ready(local_addr).map_err(|err| {
            Error::Backend(format!(
                "remote server did not answer startup probe for {local_addr}: {err}"
            ))
        })?;
        println!("  remote server: http://{local_addr}");
        Ok(server)
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    pub fn video_frame_interval(&self) -> Duration {
        let fps = self.state.config.video_fps.max(1) as u64;
        Duration::from_millis((1_000 / fps).max(1))
    }

    #[cfg(test)]
    fn listener_guard_addr(&self) -> SocketAddr {
        self.state
            .listener
            .local_addr()
            .expect("remote listener guard local addr")
    }

    pub fn audio_sink(&self) -> RemoteAudioSink {
        RemoteAudioSink {
            state: self.state.clone(),
        }
    }

    pub fn publish_status(&self, status: &RemoteStatus) {
        let mut value = v2_status_json(status);
        if let Some(object) = value.as_object_mut() {
            let audio = self.state.audio.lock().expect("remote audio mutex");
            object.insert(
                "audioEnabled".to_owned(),
                Value::Bool(self.state.config.audio_enabled),
            );
            object.insert(
                "audioSampleRate".to_owned(),
                json!(self.state.config.audio_sample_rate),
            );
            object.insert(
                "audioChannels".to_owned(),
                json!(self.state.config.audio_channels),
            );
            object.insert(
                "audioFormat".to_owned(),
                Value::String(self.state.config.audio_format.clone()),
            );
            object.insert("guestFps".to_owned(), json!(self.state.config.video_fps));
            object.insert("audioClients".to_owned(), json!(audio.clients.len()));
            object.insert("queuedAudioChunks".to_owned(), json!(audio.chunks.len()));
            object.insert(
                "pendingControlMessages".to_owned(),
                json!(self.pending_control_count()),
            );
        }
        *self
            .state
            .latest_status
            .lock()
            .expect("remote status mutex") = value;
    }

    pub fn publish_framebuffer(&self, framebuffer: &dyn Framebuffer) {
        self.publish_framebuffer_with_caret(framebuffer, None);
    }

    /// Publish the framebuffer and overlay the caret XOR rectangle on top.
    /// `caret` is (screen_x, screen_y, width, height) in framebuffer coordinates.
    /// The caret is drawn by inverting (XOR with 0xFF) all three RGB channels of
    /// each pixel in the rectangle, matching CE GWES caret rendering semantics.
    pub fn publish_framebuffer_with_caret(
        &self,
        framebuffer: &dyn Framebuffer,
        caret: Option<(u32, u32, u32, u32)>,
    ) {
        let mut image = framebuffer_to_rgb(framebuffer);
        if let Some((cx, cy, cw, ch)) = caret {
            overlay_caret_xor(&mut image, cx, cy, cw, ch);
        }
        *self
            .state
            .latest_framebuffer
            .lock()
            .expect("remote framebuffer mutex") = Some(image);
    }

    pub fn publish_debug_text(&self, key: impl Into<String>, text: impl Into<String>) {
        self.state
            .latest_debug
            .lock()
            .expect("remote debug mutex")
            .insert(key.into(), text.into());
    }

    pub fn claim_debug_publish_slot(&self, interval: Duration) -> bool {
        if interval.is_zero() {
            return true;
        }
        let now = Instant::now();
        let mut latest = self
            .state
            .latest_debug_publish
            .lock()
            .expect("remote debug publish mutex");
        if latest.is_some_and(|last| now.duration_since(last) < interval) {
            return false;
        }
        *latest = Some(now);
        true
    }

    pub fn publish_recent_logs(&self, lines: impl IntoIterator<Item = String>) {
        let mut recent_logs = self
            .state
            .recent_logs
            .lock()
            .expect("remote recent-log mutex");
        recent_logs.clear();
        for line in lines {
            recent_logs.push_back(clamp_log_line(line));
            while recent_logs.len() > MAX_RECENT_LOG_LINES {
                recent_logs.pop_front();
            }
        }
    }

    pub fn publish_log_line(&self, line: impl Into<String>) {
        let mut recent_logs = self
            .state
            .recent_logs
            .lock()
            .expect("remote recent-log mutex");
        recent_logs.push_back(clamp_log_line(line.into()));
        while recent_logs.len() > MAX_RECENT_LOG_LINES {
            recent_logs.pop_front();
        }
    }

    pub fn recent_log_lines(&self, max_lines: usize) -> Vec<String> {
        recent_log_lines(&self.state, max_lines)
    }

    pub fn drain_control_messages(&self) -> Vec<Value> {
        self.state
            .pending_control
            .lock()
            .expect("remote control mutex")
            .drain(..)
            .collect()
    }

    pub fn pending_control_count(&self) -> usize {
        self.state
            .pending_control
            .lock()
            .expect("remote control mutex")
            .len()
    }

    fn serve(self) {
        loop {
            match self.state.listener.accept() {
                Ok((stream, _)) => {
                    let server = self.clone();
                    let _ = thread::Builder::new()
                        .name("wince-remote-client".to_owned())
                        .spawn(move || server.handle_stream(stream));
                }
                Err(err) => {
                    tracing::warn!(target: "ce.remote", "remote server accept failed: {err}");
                    thread::sleep(Duration::from_millis(50));
                }
            }
        }
    }

    fn handle_stream(&self, mut stream: TcpStream) {
        let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
        let response = match read_http_request(&mut stream) {
            Ok(request) => self.dispatch_http_request(request),
            Err(err) => HttpResponse::json(
                400,
                json!({"ok": false, "error": format!("bad request: {err}")}),
            )
            .into(),
        };
        match response {
            RemoteHttpResponse::One(response) => {
                let _ = stream.write_all(&response.to_bytes());
            }
            RemoteHttpResponse::Mjpeg { request } => self.write_mjpeg_stream(stream, request),
            RemoteHttpResponse::WebSocket { request, kind } => match kind {
                WebSocketKind::Control => self.handle_control_websocket(stream, request),
                WebSocketKind::Audio => self.handle_audio_websocket(stream, request),
            },
        }
    }

    fn dispatch_http_request(&self, request: HttpRequest) -> RemoteHttpResponse {
        if !self.authorized(&request) {
            return HttpResponse::json(401, json!({"ok": false, "error": "unauthorized"})).into();
        }
        if request.method == "OPTIONS" {
            return HttpResponse::empty(204).into();
        }
        match (request.method.as_str(), request.path.as_str()) {
            ("GET", "/") => self.endpoint_index().into(),
            ("GET", "/api/v1/status") | ("GET", "/status") => {
                let status = self
                    .state
                    .latest_status
                    .lock()
                    .expect("remote status mutex")
                    .clone();
                HttpResponse::json(200, status).into()
            }
            ("GET", "/api/v1/frame.jpg") => self.latest_jpeg_response(&request).into(),
            ("GET", "/api/v1/debug/screenshot.png") => self.latest_png_response().into(),
            ("GET", "/api/v1/debug/summary.txt") => self.latest_debug_text("summary").into(),
            ("GET", "/api/v1/debug/windows.txt") => self.latest_debug_text("windows").into(),
            ("GET", "/api/v1/debug/messages.txt") => self.latest_debug_text("messages").into(),
            ("GET", "/api/v1/debug/message-boxes.txt") => {
                self.latest_debug_text("message-boxes").into()
            }
            ("GET", "/api/v1/debug/processes.txt") => self.latest_debug_text("processes").into(),
            ("GET", "/api/v1/debug/processes-live.txt") => {
                self.latest_debug_text("processes-live").into()
            }
            ("GET", "/api/v1/debug/sends.txt") => self.latest_debug_text("sends").into(),
            ("GET", "/api/v1/debug/pending-wndproc.txt") => {
                self.latest_debug_text("pending-wndproc").into()
            }
            ("GET", "/api/v1/debug/wndproc.txt") => self.latest_debug_text("wndproc").into(),
            ("GET", "/api/v1/debug/imports.txt") => self.latest_debug_text("imports").into(),
            ("GET", "/api/v1/debug/milestones.txt") => self.latest_debug_text("milestones").into(),
            ("GET", "/api/v1/debug/counts.txt") => self.latest_debug_text("counts").into(),
            ("GET", "/api/v1/debug/calls.txt") => self.latest_debug_text("calls").into(),
            ("GET", "/api/v1/debug/code.txt") => self.latest_debug_text("code").into(),
            ("GET", "/api/v1/debug/blocks.txt") => self.latest_debug_text("blocks").into(),
            ("GET", "/api/v1/debug/blobs.txt") => self.latest_debug_text("blobs").into(),
            ("GET", "/api/v1/debug/trampolines.txt") => {
                self.latest_debug_text("trampolines").into()
            }
            ("GET", "/api/v1/debug/files.txt") => self.latest_debug_text("files").into(),
            ("GET", "/api/v1/debug/files-full.txt") => self.latest_debug_text("files-full").into(),
            ("GET", "/api/v1/debug/events.txt") => self.latest_debug_text("events").into(),
            ("GET", "/api/v1/debug/devices.txt") => self.latest_debug_text("devices").into(),
            ("GET", "/api/v1/debug/timers.txt") => self.latest_debug_text("timers").into(),
            ("GET", "/api/v1/debug/render.txt") => self.latest_debug_text("render").into(),
            ("GET", "/api/v1/debug/controller.txt") => self.latest_debug_text("controller").into(),
            ("GET", "/api/v1/debug/resource.txt") => self.latest_debug_text("resource").into(),
            ("GET", "/api/v1/debug/active.txt") => self.latest_debug_text("active").into(),
            ("GET", "/api/v1/debug/parked.txt") => self.latest_debug_text("parked").into(),
            ("GET", "/api/v1/debug/remote-input.txt") => {
                self.latest_debug_text("remote-input").into()
            }
            ("GET", "/api/v1/video.mjpg") => RemoteHttpResponse::Mjpeg { request },
            ("GET", "/framebuffer.ppm") => self.latest_ppm_response().into(),
            ("POST", "/api/v1/input/touch") => self.post_touch(request).into(),
            ("POST", "/api/v1/input/key") => self.post_key(request).into(),
            ("POST", "/api/v1/sensors/location") => self.post_location(request).into(),
            ("POST", "/api/v1/sensors/nmea") => self.post_nmea(request).into(),
            ("POST", "/api/v1/sensors/imu") => self.post_imu(request).into(),
            ("GET", "/api/v1/logs/recent") => {
                let lines = request.query_u64("lines", 200, 1, 4096) as usize;
                HttpResponse::json(
                    200,
                    json!({"ok": true, "lines": self.recent_log_lines(lines)}),
                )
                .into()
            }
            ("POST", "/api/v1/control/pause") => {
                self.queue_control(json!({"type": "pause"}));
                HttpResponse::json(200, json!({"ok": true, "paused": true})).into()
            }
            ("POST", "/api/v1/control/resume") => {
                self.queue_control(json!({"type": "resume"}));
                HttpResponse::json(200, json!({"ok": true, "paused": false})).into()
            }
            ("GET", "/api/v1/audio/ws") => RemoteHttpResponse::WebSocket {
                request,
                kind: WebSocketKind::Audio,
            },
            ("GET", "/api/v1/control/ws") => RemoteHttpResponse::WebSocket {
                request,
                kind: WebSocketKind::Control,
            },
            ("POST", "/control") | ("POST", "/remote") => self.post_legacy_control(request).into(),
            _ => HttpResponse::json(404, json!({"ok": false, "error": "not found"})).into(),
        }
    }

    fn authorized(&self, request: &HttpRequest) -> bool {
        let Some(token) = self.state.config.token.as_deref() else {
            return true;
        };
        let Some(value) = request.header("authorization") else {
            return false;
        };
        value.trim() == format!("Bearer {token}")
    }

    fn endpoint_index(&self) -> HttpResponse {
        HttpResponse::json(
            200,
            json!({
                "ok": true,
                "endpoints": {
                    "status": "GET /api/v1/status",
                    "frame": "GET /api/v1/frame.jpg",
                    "screenshot": "GET /api/v1/debug/screenshot.png",
                    "debugSummary": "GET /api/v1/debug/summary.txt",
                    "debugWindows": "GET /api/v1/debug/windows.txt",
                    "debugMessages": "GET /api/v1/debug/messages.txt",
                    "debugMessageBoxes": "GET /api/v1/debug/message-boxes.txt",
                    "debugProcesses": "GET /api/v1/debug/processes.txt",
                    "debugProcessesLive": "GET /api/v1/debug/processes-live.txt",
                    "debugSends": "GET /api/v1/debug/sends.txt",
                    "debugPendingWndProc": "GET /api/v1/debug/pending-wndproc.txt",
                    "debugWndProc": "GET /api/v1/debug/wndproc.txt",
                    "debugImports": "GET /api/v1/debug/imports.txt",
                    "debugMilestones": "GET /api/v1/debug/milestones.txt",
                    "debugCounts": "GET /api/v1/debug/counts.txt",
                    "debugCalls": "GET /api/v1/debug/calls.txt",
                    "debugCode": "GET /api/v1/debug/code.txt",
                    "debugBlocks": "GET /api/v1/debug/blocks.txt",
                    "debugBlobs": "GET /api/v1/debug/blobs.txt",
                    "debugTrampolines": "GET /api/v1/debug/trampolines.txt",
                    "debugFiles": "GET /api/v1/debug/files.txt",
                    "debugFilesFull": "GET /api/v1/debug/files-full.txt",
                    "debugEvents": "GET /api/v1/debug/events.txt",
                    "debugDevices": "GET /api/v1/debug/devices.txt",
                    "debugRender": "GET /api/v1/debug/render.txt",
                    "debugController": "GET /api/v1/debug/controller.txt",
                    "debugResource": "GET /api/v1/debug/resource.txt",
                    "debugActive": "GET /api/v1/debug/active.txt",
                    "debugParked": "GET /api/v1/debug/parked.txt",
                    "debugRemoteInput": "GET /api/v1/debug/remote-input.txt",
                    "video": "GET /api/v1/video.mjpg",
                    "touch": "POST /api/v1/input/touch",
                    "key": "POST /api/v1/input/key",
                    "location": "POST /api/v1/sensors/location",
                    "nmea": "POST /api/v1/sensors/nmea",
                    "imu": "POST /api/v1/sensors/imu",
                    "logs": "GET /api/v1/logs/recent",
                    "pause": "POST /api/v1/control/pause",
                    "resume": "POST /api/v1/control/resume"
                }
            }),
        )
    }

    fn post_touch(&self, request: HttpRequest) -> HttpResponse {
        let body = match parse_json_body(&request, "invalid touch body") {
            Ok(body) => body,
            Err(response) => return response,
        };
        let Some(kind) = body.get("type").and_then(Value::as_str) else {
            return HttpResponse::json(400, json!({"ok": false, "error": "invalid touch body"}));
        };
        let Some(x) = body.get("x").and_then(Value::as_i64) else {
            return HttpResponse::json(400, json!({"ok": false, "error": "invalid touch body"}));
        };
        let Some(y) = body.get("y").and_then(Value::as_i64) else {
            return HttpResponse::json(400, json!({"ok": false, "error": "invalid touch body"}));
        };
        let phase = if kind == "touch" {
            body.get("phase").and_then(Value::as_str).unwrap_or("tap")
        } else {
            kind
        };
        if !is_supported_touch_phase(phase) {
            return HttpResponse::json(
                400,
                json!({"ok": false, "error": "unsupported touch type"}),
            );
        }
        let pending = self.queue_control(json!({
            "type": "touch",
            "phase": phase,
            "x": x,
            "y": y
        }));
        HttpResponse::json(200, json!({"ok": true, "pending": pending}))
    }

    fn post_key(&self, request: HttpRequest) -> HttpResponse {
        let body = match parse_json_body(&request, "invalid key body") {
            Ok(body) => body,
            Err(response) => return response,
        };
        let Some(kind) = body.get("type").and_then(Value::as_str) else {
            return HttpResponse::json(400, json!({"ok": false, "error": "invalid key body"}));
        };
        let Some(vk) = body.get("vk").and_then(Value::as_u64) else {
            return HttpResponse::json(400, json!({"ok": false, "error": "invalid key body"}));
        };
        if kind != "down" && kind != "up" {
            return HttpResponse::json(400, json!({"ok": false, "error": "unsupported key type"}));
        }
        if !(1..=0xff).contains(&vk) {
            return HttpResponse::json(
                400,
                json!({"ok": false, "error": "vk must be between 1 and 255"}),
            );
        }
        let pending = self.queue_control(json!({
            "type": "key",
            "phase": kind,
            "vk": vk
        }));
        HttpResponse::json(200, json!({"ok": true, "pending": pending}))
    }

    fn post_location(&self, request: HttpRequest) -> HttpResponse {
        let mut body = match parse_json_body(&request, "invalid location body") {
            Ok(body) => body,
            Err(response) => return response,
        };
        let lat = body
            .get("lat")
            .or_else(|| body.get("latitude"))
            .and_then(Value::as_f64);
        let lon = body
            .get("lon")
            .or_else(|| body.get("longitude"))
            .and_then(Value::as_f64);
        let (Some(lat), Some(lon)) = (lat, lon) else {
            return HttpResponse::json(400, json!({"ok": false, "error": "invalid location body"}));
        };
        body["lat"] = json!(lat);
        body["lon"] = json!(lon);
        body["type"] = Value::String("location".to_owned());
        self.queue_control(body);
        HttpResponse::json(200, json!({"ok": true, "sentencesGenerated": 3}))
    }

    fn post_nmea(&self, request: HttpRequest) -> HttpResponse {
        let mut body = match parse_json_body(&request, "invalid nmea body") {
            Ok(body) => body,
            Err(response) => return response,
        };
        let Some(sentences) = body.get("sentences").and_then(Value::as_array) else {
            return HttpResponse::json(400, json!({"ok": false, "error": "invalid nmea body"}));
        };
        let accepted = sentences
            .iter()
            .filter(|sentence| sentence.is_string())
            .count();
        body["type"] = Value::String("nmea".to_owned());
        self.queue_control(body);
        HttpResponse::json(200, json!({"ok": true, "accepted": accepted}))
    }

    fn post_imu(&self, request: HttpRequest) -> HttpResponse {
        if let Ok(mut body) = serde_json::from_slice::<Value>(&request.body) {
            body["type"] = Value::String("imu".to_owned());
            self.queue_control(body);
        }
        HttpResponse::json(200, json!({"ok": true}))
    }

    fn post_legacy_control(&self, request: HttpRequest) -> HttpResponse {
        match serde_json::from_slice(&request.body) {
            Ok(message) => {
                let pending = self.queue_control(message);
                HttpResponse::json(202, json!({"ok": true, "queued": true, "pending": pending}))
            }
            Err(err) => HttpResponse::json(
                400,
                json!({"ok": false, "error": format!("invalid JSON: {err}")}),
            ),
        }
    }

    fn latest_jpeg_response(&self, request: &HttpRequest) -> HttpResponse {
        let quality =
            request.query_u64("quality", self.state.config.jpeg_quality as u64, 1, 100) as u8;
        match self.latest_framebuffer_image() {
            Some(image) => match encode_jpeg(&image, quality) {
                Ok(bytes) => HttpResponse::bytes(200, "image/jpeg", bytes),
                Err(err) => HttpResponse::json(503, json!({"ok": false, "error": err})),
            },
            None => framebuffer_unavailable(),
        }
    }

    fn latest_png_response(&self) -> HttpResponse {
        match self.latest_framebuffer_image() {
            Some(image) => match encode_png(&image) {
                Ok(bytes) => HttpResponse::bytes(200, "image/png", bytes),
                Err(err) => HttpResponse::json(503, json!({"ok": false, "error": err})),
            },
            None => framebuffer_unavailable(),
        }
    }

    fn latest_ppm_response(&self) -> HttpResponse {
        match self.latest_framebuffer_image() {
            Some(image) => HttpResponse::bytes(200, "image/x-portable-pixmap", encode_ppm(&image)),
            None => framebuffer_unavailable(),
        }
    }

    fn latest_debug_text(&self, key: &str) -> HttpResponse {
        let debug = self.state.latest_debug.lock().expect("remote debug mutex");
        match debug.get(key) {
            Some(text) => {
                HttpResponse::bytes(200, "text/plain; charset=utf-8", text.as_bytes().to_vec())
            }
            None => HttpResponse::json(503, json!({"ok": false, "error": "no debug snapshot"})),
        }
    }

    fn latest_framebuffer_image(&self) -> Option<RemoteFramebufferImage> {
        self.state
            .latest_framebuffer
            .lock()
            .expect("remote framebuffer mutex")
            .clone()
    }

    fn queue_control(&self, message: Value) -> usize {
        queue_control_message(&self.state, message)
    }

    fn write_mjpeg_stream(&self, mut stream: TcpStream, request: HttpRequest) {
        let fps = request.query_u64("fps", self.state.config.video_fps as u64, 1, 60) as u32;
        let quality =
            request.query_u64("quality", self.state.config.jpeg_quality as u64, 1, 100) as u8;
        let frame_delay = Duration::from_millis((1000 / fps).max(1) as u64);
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace; boundary=frame\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n"
        );
        if stream.write_all(header.as_bytes()).is_err() {
            return;
        }
        loop {
            let Some(image) = self.latest_framebuffer_image() else {
                thread::sleep(frame_delay);
                continue;
            };
            let Ok(jpeg) = encode_jpeg(&image, quality) else {
                thread::sleep(frame_delay);
                continue;
            };
            let part = format!(
                "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                jpeg.len()
            );
            if stream.write_all(part.as_bytes()).is_err()
                || stream.write_all(&jpeg).is_err()
                || stream.write_all(b"\r\n").is_err()
            {
                return;
            }
            thread::sleep(frame_delay);
        }
    }

    fn handle_control_websocket(&self, mut stream: TcpStream, request: HttpRequest) {
        if write_websocket_handshake(&mut stream, &request).is_err() {
            return;
        }
        let _ = stream.set_read_timeout(None);
        let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
        loop {
            let frame = match read_websocket_frame(&mut stream) {
                Ok(Some(frame)) => frame,
                Ok(None) | Err(_) => return,
            };
            match frame.opcode {
                WebSocketOpcode::Text => {
                    let response = match serde_json::from_slice::<Value>(&frame.payload) {
                        Ok(message) => match message.get("type").and_then(Value::as_str) {
                            Some("logs") => {
                                let lines = message
                                    .get("lines")
                                    .and_then(Value::as_u64)
                                    .unwrap_or(200)
                                    .clamp(1, 4096)
                                    as usize;
                                json!({"type": "log", "ok": true, "lines": self.recent_log_lines(lines)})
                            }
                            Some("status") => {
                                let status = self
                                    .state
                                    .latest_status
                                    .lock()
                                    .expect("remote status mutex")
                                    .clone();
                                json!({"type": "status", "ok": true, "status": status})
                            }
                            _ => {
                                let pending = self.queue_control(message);
                                json!({"ok": true, "queued": true, "pending": pending})
                            }
                        },
                        Err(err) => json!({"ok": false, "error": format!("invalid JSON: {err}")}),
                    };
                    if write_websocket_text(&mut stream, &response.to_string()).is_err() {
                        return;
                    }
                }
                WebSocketOpcode::Binary => {
                    if write_websocket_text(
                        &mut stream,
                        r#"{"ok":false,"error":"control websocket expects JSON text frames"}"#,
                    )
                    .is_err()
                    {
                        return;
                    }
                }
                WebSocketOpcode::Ping => {
                    if write_websocket_frame(&mut stream, WebSocketOpcode::Pong, &frame.payload)
                        .is_err()
                    {
                        return;
                    }
                }
                WebSocketOpcode::Close => {
                    let _ = write_websocket_frame(&mut stream, WebSocketOpcode::Close, &[]);
                    return;
                }
                WebSocketOpcode::Pong => {}
            }
        }
    }

    fn handle_audio_websocket(&self, mut stream: TcpStream, request: HttpRequest) {
        if write_websocket_handshake(&mut stream, &request).is_err() {
            return;
        }
        let _ = stream.set_read_timeout(None);
        let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
        let client_id = self.register_audio_client();
        let metadata = json!({
            "type": "audio",
            "codec": "pcm",
            "sampleRate": self.state.config.audio_sample_rate,
            "channels": self.state.config.audio_channels,
            "format": self.state.config.audio_format,
            "enabled": self.state.config.audio_enabled
        });
        if write_websocket_text(&mut stream, &metadata.to_string()).is_err() {
            self.unregister_audio_client(client_id);
            return;
        }
        let mut ticks = 0u32;
        loop {
            let chunks = self.take_audio_chunks_for_client(client_id, 8);
            for chunk in chunks {
                if write_websocket_frame(&mut stream, WebSocketOpcode::Binary, &chunk.payload)
                    .is_err()
                {
                    self.unregister_audio_client(client_id);
                    return;
                }
            }
            ticks = ticks.wrapping_add(1);
            if ticks >= 50 {
                ticks = 0;
                if write_websocket_frame(&mut stream, WebSocketOpcode::Ping, &[]).is_err() {
                    self.unregister_audio_client(client_id);
                    return;
                }
            }
            thread::sleep(Duration::from_millis(20));
        }
    }

    fn register_audio_client(&self) -> u64 {
        let mut audio = self.state.audio.lock().expect("remote audio mutex");
        let client_id = audio.next_client_id;
        audio.next_client_id = audio.next_client_id.saturating_add(1);
        let next_sequence = audio
            .chunks
            .back()
            .map(|chunk| chunk.sequence.saturating_add(1))
            .unwrap_or(audio.sequence.saturating_add(1));
        audio
            .clients
            .insert(client_id, RemoteAudioCursor { next_sequence });
        client_id
    }

    fn unregister_audio_client(&self, client_id: u64) {
        let mut audio = self.state.audio.lock().expect("remote audio mutex");
        audio.clients.remove(&client_id);
    }

    fn take_audio_chunks_for_client(&self, client_id: u64, max_chunks: usize) -> Vec<AudioChunk> {
        let mut audio = self.state.audio.lock().expect("remote audio mutex");
        let Some(cursor) = audio.clients.get(&client_id).copied() else {
            return Vec::new();
        };
        let mut chunks = Vec::new();
        for chunk in audio
            .chunks
            .iter()
            .filter(|chunk| chunk.sequence >= cursor.next_sequence)
        {
            if chunks.len() >= max_chunks {
                break;
            }
            chunks.push(chunk.clone());
        }
        if let Some(last) = chunks.last() {
            audio.clients.insert(
                client_id,
                RemoteAudioCursor {
                    next_sequence: last.sequence.saturating_add(1),
                },
            );
        }
        chunks
    }
}

impl AudioSink for RemoteAudioSink {
    fn name(&self) -> &str {
        "remote-websocket"
    }

    fn submit_pcm(
        &mut self,
        payload: &[u8],
        _format: &WaveFormat,
        pts_ms: u64,
        duration_ms: u32,
        flush: bool,
    ) -> Option<u64> {
        if !self.state.config.audio_enabled || payload.is_empty() {
            return None;
        }
        let mut audio = self.state.audio.lock().expect("remote audio mutex");
        audio.sequence = audio.sequence.saturating_add(1);
        let sequence = audio.sequence;
        audio.chunks.push_back(AudioChunk {
            payload: payload.to_vec(),
            sequence,
            pts_ms,
            duration_ms: duration_ms.max(1),
            flush,
        });
        while audio.chunks.len() > audio.max_chunks {
            audio.chunks.pop_front();
        }
        Some(sequence)
    }

    fn flush(&mut self) {}

    fn queued_chunk_count(&self) -> usize {
        self.state
            .audio
            .lock()
            .expect("remote audio mutex")
            .chunks
            .len()
    }
}

pub fn v2_status_json(status: &RemoteStatus) -> Value {
    json!({
        "running": status.running,
        "guestWidth": status.guest_width,
        "guestHeight": status.guest_height,
        "guestFps": status.guest_fps,
        "videoEnabled": status.video_enabled,
        "videoCodec": status.video_codec,
        "audioEnabled": status.audio_enabled,
        "audioCodec": status.audio_codec,
        "audioSampleRate": status.audio_sample_rate,
        "audioChannels": status.audio_channels,
        "audioFormat": status.audio_format,
        "gpsEnabled": status.gps_enabled,
        "gpsTarget": status.gps_target,
        "paused": status.paused,
        "queuedTouchEvents": status.queued_touch_events,
        "queuedKeyEvents": status.queued_key_events,
        "queuedSerialBytes": status.queued_serial_bytes,
        "audioClients": status.audio_clients,
        "queuedAudioChunks": status.queued_audio_chunks
    })
}

fn wait_until_http_ready(addr: SocketAddr) -> std::result::Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut last_error = String::from("not attempted");
    while Instant::now() < deadline {
        match probe_status_endpoint(addr) {
            Ok(()) => return Ok(()),
            Err(err) => last_error = err,
        }
        thread::sleep(Duration::from_millis(25));
    }
    Err(last_error)
}

fn probe_status_endpoint(addr: SocketAddr) -> std::result::Result<(), String> {
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(250))
        .map_err(|err| format!("connect failed: {err}"))?;
    stream
        .set_read_timeout(Some(Duration::from_millis(500)))
        .map_err(|err| format!("set read timeout failed: {err}"))?;
    stream
        .set_write_timeout(Some(Duration::from_millis(500)))
        .map_err(|err| format!("set write timeout failed: {err}"))?;
    stream
        .write_all(
            b"GET /api/v1/status HTTP/1.1\r\nHost: startup-probe\r\nConnection: close\r\n\r\n",
        )
        .map_err(|err| format!("write probe failed: {err}"))?;
    let mut response = [0u8; 64];
    let read = stream
        .read(&mut response)
        .map_err(|err| format!("read probe failed: {err}"))?;
    if response[..read].starts_with(b"HTTP/1.1 200 ") {
        Ok(())
    } else {
        Err(format!(
            "unexpected probe response: {}",
            String::from_utf8_lossy(&response[..read])
        ))
    }
}

fn queue_control_message(state: &RemoteServerState, message: Value) -> usize {
    let mut pending = state.pending_control.lock().expect("remote control mutex");
    pending.push_back(message);
    while pending.len() > MAX_PENDING_CONTROL_MESSAGES {
        pending.pop_front();
    }
    pending.len()
}

fn recent_log_lines(state: &RemoteServerState, max_lines: usize) -> Vec<String> {
    let recent_logs = state.recent_logs.lock().expect("remote recent-log mutex");
    let count = max_lines
        .clamp(1, MAX_RECENT_LOG_LINES)
        .min(recent_logs.len());
    recent_logs
        .iter()
        .skip(recent_logs.len() - count)
        .cloned()
        .collect()
}

fn clamp_log_line(mut line: String) -> String {
    if line.len() <= MAX_RECENT_LOG_LINE_BYTES {
        return line;
    }
    let mut end = MAX_RECENT_LOG_LINE_BYTES;
    while end > 0 && !line.is_char_boundary(end) {
        end -= 1;
    }
    line.truncate(end);
    line
}

fn parse_json_body(
    request: &HttpRequest,
    invalid_body_error: &'static str,
) -> std::result::Result<Value, HttpResponse> {
    serde_json::from_slice(&request.body)
        .map_err(|_| HttpResponse::json(400, json!({"ok": false, "error": invalid_body_error})))
}

fn framebuffer_unavailable() -> HttpResponse {
    HttpResponse::json(503, json!({"ok": false, "error": "no framebuffer"}))
}

fn is_supported_touch_phase(phase: &str) -> bool {
    matches!(
        phase,
        "down" | "move" | "up" | "cancel" | "" | "tap" | "click" | "single" | "single-touch"
    )
}

#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    query: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl HttpRequest {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(header, _)| header.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    fn query_u64(&self, name: &str, default: u64, min: u64, max: u64) -> u64 {
        for part in self.query.split('&') {
            let (key, value) = part.split_once('=').unwrap_or((part, ""));
            if key == name
                && let Ok(parsed) = value.parse::<u64>()
            {
                return parsed.clamp(min, max);
            }
        }
        default.clamp(min, max)
    }
}

enum RemoteHttpResponse {
    One(HttpResponse),
    Mjpeg {
        request: HttpRequest,
    },
    WebSocket {
        request: HttpRequest,
        kind: WebSocketKind,
    },
}

enum WebSocketKind {
    Control,
    Audio,
}

impl From<HttpResponse> for RemoteHttpResponse {
    fn from(value: HttpResponse) -> Self {
        Self::One(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WebSocketOpcode {
    Text,
    Binary,
    Close,
    Ping,
    Pong,
}

#[derive(Debug)]
struct WebSocketFrame {
    opcode: WebSocketOpcode,
    payload: Vec<u8>,
}

#[derive(Debug)]
struct HttpResponse {
    status: u16,
    reason: &'static str,
    content_type: &'static str,
    body: Vec<u8>,
}

impl HttpResponse {
    fn empty(status: u16) -> Self {
        Self {
            status,
            reason: reason_phrase(status),
            content_type: "text/plain",
            body: Vec::new(),
        }
    }

    fn json(status: u16, body: Value) -> Self {
        let reason = reason_phrase(status);
        Self {
            status,
            reason,
            content_type: "application/json; charset=utf-8",
            body: serde_json::to_vec(&body).expect("JSON response serializes"),
        }
    }

    fn bytes(status: u16, content_type: &'static str, body: Vec<u8>) -> Self {
        Self {
            status,
            reason: reason_phrase(status),
            content_type,
            body,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: Authorization, Content-Type\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\n\r\n",
            self.status,
            self.reason,
            self.content_type,
            self.body.len()
        )
        .into_bytes();
        bytes.extend_from_slice(&self.body);
        bytes
    }
}

fn write_websocket_handshake(stream: &mut TcpStream, request: &HttpRequest) -> std::io::Result<()> {
    let key = request.header("sec-websocket-key").ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "missing sec-websocket-key")
    })?;
    let accept = websocket_accept_key(key);
    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {accept}\r\nAccess-Control-Allow-Origin: *\r\n\r\n"
    );
    stream.write_all(response.as_bytes())
}

fn websocket_accept_key(key: &str) -> String {
    let mut bytes = Vec::with_capacity(key.trim().len() + WEBSOCKET_GUID.len());
    bytes.extend_from_slice(key.trim().as_bytes());
    bytes.extend_from_slice(WEBSOCKET_GUID.as_bytes());
    base64_encode(&sha1_digest(&bytes))
}

fn read_websocket_frame(stream: &mut TcpStream) -> std::io::Result<Option<WebSocketFrame>> {
    let mut header = [0u8; 2];
    if let Err(err) = stream.read_exact(&mut header) {
        return if err.kind() == std::io::ErrorKind::UnexpectedEof {
            Ok(None)
        } else {
            Err(err)
        };
    }
    let fin = (header[0] & 0x80) != 0;
    let opcode = match header[0] & 0x0f {
        0x1 => WebSocketOpcode::Text,
        0x2 => WebSocketOpcode::Binary,
        0x8 => WebSocketOpcode::Close,
        0x9 => WebSocketOpcode::Ping,
        0xA => WebSocketOpcode::Pong,
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unsupported websocket opcode",
            ));
        }
    };
    if !fin {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "fragmented websocket frames are not supported",
        ));
    }
    let masked = (header[1] & 0x80) != 0;
    if !masked {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "client websocket frames must be masked",
        ));
    }
    let mut len = u64::from(header[1] & 0x7f);
    if len == 126 {
        let mut extended = [0u8; 2];
        stream.read_exact(&mut extended)?;
        len = u64::from(u16::from_be_bytes(extended));
    } else if len == 127 {
        let mut extended = [0u8; 8];
        stream.read_exact(&mut extended)?;
        len = u64::from_be_bytes(extended);
    }
    if len as usize > MAX_REQUEST_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "websocket frame too large",
        ));
    }
    let mut mask = [0u8; 4];
    stream.read_exact(&mut mask)?;
    let mut payload = vec![0; len as usize];
    stream.read_exact(&mut payload)?;
    for (index, byte) in payload.iter_mut().enumerate() {
        *byte ^= mask[index % 4];
    }
    Ok(Some(WebSocketFrame { opcode, payload }))
}

fn write_websocket_text(stream: &mut TcpStream, text: &str) -> std::io::Result<()> {
    write_websocket_frame(stream, WebSocketOpcode::Text, text.as_bytes())
}

fn write_websocket_frame(
    stream: &mut TcpStream,
    opcode: WebSocketOpcode,
    payload: &[u8],
) -> std::io::Result<()> {
    let opcode_byte = match opcode {
        WebSocketOpcode::Text => 0x1,
        WebSocketOpcode::Binary => 0x2,
        WebSocketOpcode::Close => 0x8,
        WebSocketOpcode::Ping => 0x9,
        WebSocketOpcode::Pong => 0xA,
    };
    let mut header = Vec::with_capacity(14);
    header.push(0x80 | opcode_byte);
    if payload.len() <= 125 {
        header.push(payload.len() as u8);
    } else if payload.len() <= u16::MAX as usize {
        header.push(126);
        header.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    } else {
        header.push(127);
        header.extend_from_slice(&(payload.len() as u64).to_be_bytes());
    }
    stream.write_all(&header)?;
    stream.write_all(payload)
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(b2 & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn sha1_digest(bytes: &[u8]) -> [u8; 20] {
    let mut h0 = 0x67452301u32;
    let mut h1 = 0xEFCDAB89u32;
    let mut h2 = 0x98BADCFEu32;
    let mut h3 = 0x10325476u32;
    let mut h4 = 0xC3D2E1F0u32;

    let bit_len = (bytes.len() as u64) * 8;
    let mut message = bytes.to_vec();
    message.push(0x80);
    while (message.len() % 64) != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in message.chunks_exact(64) {
        let mut w = [0u32; 80];
        for (index, word) in w.iter_mut().take(16).enumerate() {
            let offset = index * 4;
            *word = u32::from_be_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }
        for index in 16..80 {
            w[index] = (w[index - 3] ^ w[index - 8] ^ w[index - 14] ^ w[index - 16]).rotate_left(1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;
        for (index, word) in w.iter().copied().enumerate() {
            let (f, k) = match index {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }
        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    let mut out = [0u8; 20];
    out[..4].copy_from_slice(&h0.to_be_bytes());
    out[4..8].copy_from_slice(&h1.to_be_bytes());
    out[8..12].copy_from_slice(&h2.to_be_bytes());
    out[12..16].copy_from_slice(&h3.to_be_bytes());
    out[16..20].copy_from_slice(&h4.to_be_bytes());
    out
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        202 => "Accepted",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        501 => "Not Implemented",
        503 => "Service Unavailable",
        _ => "OK",
    }
}

fn read_http_request(stream: &mut TcpStream) -> std::io::Result<HttpRequest> {
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 4096];
    let header_end = loop {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            break find_header_end(&bytes).unwrap_or(bytes.len());
        }
        bytes.extend_from_slice(&buffer[..read]);
        if bytes.len() > MAX_REQUEST_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "request too large",
            ));
        }
        if let Some(header_end) = find_header_end(&bytes) {
            break header_end;
        }
    };
    let header_text = String::from_utf8_lossy(&bytes[..header_end]);
    let mut lines = header_text.lines();
    let request_line = lines.next().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "missing request line")
    })?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    let target = request_parts.next().unwrap_or("/");
    let (path, query) = split_target(target);
    let mut content_length = 0usize;
    let mut headers = Vec::new();
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let name = name.trim().to_ascii_lowercase();
        let value = value.trim().to_owned();
        if name.eq_ignore_ascii_case("content-length") {
            content_length = value.parse().map_err(|err| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("invalid content-length: {err}"),
                )
            })?;
        }
        headers.push((name, value));
    }
    if content_length > MAX_REQUEST_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "request body too large",
        ));
    }
    let body_start = header_end + 4;
    while bytes.len().saturating_sub(body_start) < content_length {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..read]);
    }
    let body_end = body_start.saturating_add(content_length).min(bytes.len());
    Ok(HttpRequest {
        method,
        path,
        query,
        headers,
        body: bytes[body_start..body_end].to_vec(),
    })
}

fn split_target(target: &str) -> (String, String) {
    match target.split_once('?') {
        Some((path, query)) => (path.to_owned(), query.to_owned()),
        None => (target.to_owned(), String::new()),
    }
}

fn find_header_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

fn framebuffer_to_rgb(framebuffer: &dyn Framebuffer) -> RemoteFramebufferImage {
    let info = framebuffer.info();
    let mut rgb = Vec::with_capacity((info.width as usize) * (info.height as usize) * 3);
    for y in 0..info.height as usize {
        let row = &framebuffer.pixels()[y * info.stride..];
        for x in 0..info.width as usize {
            let offset = x * info.format.bytes_per_pixel();
            rgb.extend_from_slice(&pixel_to_rgb(info.format, &row[offset..]));
        }
    }
    RemoteFramebufferImage {
        width: info.width,
        height: info.height,
        rgb,
    }
}

/// XOR-invert the RGB pixels inside the caret rectangle on a composed frame.
/// CE GWES draws the caret by XOR-ing a solid rectangle against the surface,
/// which is equivalent to inverting all three channels of each covered pixel.
fn overlay_caret_xor(image: &mut RemoteFramebufferImage, cx: u32, cy: u32, cw: u32, ch: u32) {
    let x0 = cx.min(image.width) as usize;
    let y0 = cy.min(image.height) as usize;
    let x1 = (cx.saturating_add(cw)).min(image.width) as usize;
    let y1 = (cy.saturating_add(ch)).min(image.height) as usize;
    let stride = image.width as usize * 3;
    for y in y0..y1 {
        for x in x0..x1 {
            let base = y * stride + x * 3;
            image.rgb[base] ^= 0xff;
            image.rgb[base + 1] ^= 0xff;
            image.rgb[base + 2] ^= 0xff;
        }
    }
}

fn encode_jpeg(
    image: &RemoteFramebufferImage,
    quality: u8,
) -> std::result::Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let encoder = JpegEncoder::new_with_quality(&mut bytes, quality.clamp(1, 100));
    encoder
        .write_image(
            &image.rgb,
            image.width,
            image.height,
            ColorType::Rgb8.into(),
        )
        .map_err(|err| format!("JPEG encode failed: {err}"))?;
    Ok(bytes)
}

fn encode_png(image: &RemoteFramebufferImage) -> std::result::Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let encoder = PngEncoder::new(&mut bytes);
    encoder
        .write_image(
            &image.rgb,
            image.width,
            image.height,
            ColorType::Rgb8.into(),
        )
        .map_err(|err| format!("PNG encode failed: {err}"))?;
    Ok(bytes)
}

fn encode_ppm(image: &RemoteFramebufferImage) -> Vec<u8> {
    let mut bytes = format!("P6\n{} {}\n255\n", image.width, image.height).into_bytes();
    bytes.extend_from_slice(&image.rgb);
    bytes
}

fn pixel_to_rgb(format: PixelFormat, bytes: &[u8]) -> [u8; 3] {
    match format {
        PixelFormat::Rgb565 => {
            let raw = u16::from_le_bytes([bytes[0], bytes[1]]);
            let r5 = ((raw >> 11) & 0x1f) as u8;
            let g6 = ((raw >> 5) & 0x3f) as u8;
            let b5 = (raw & 0x1f) as u8;
            [
                (u16::from(r5) * 255 / 31) as u8,
                (u16::from(g6) * 255 / 63) as u8,
                (u16::from(b5) * 255 / 31) as u8,
            ]
        }
        PixelFormat::Bgra8888 => [bytes[2], bytes[1], bytes[0]],
        PixelFormat::Rgba8888 => [bytes[0], bytes[1], bytes[2]],
        PixelFormat::Gray8 => [bytes[0], bytes[0], bytes[0]],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ce::framebuffer::{PixelFormat, VirtualFramebuffer};
    use std::io::{Read, Write};

    #[test]
    fn remote_server_accepts_v2_touch_route() {
        let server = RemoteServer::start(RemoteServerConfig {
            addr: "127.0.0.1:0".parse().unwrap(),
            ..RemoteServerConfig::default()
        })
        .unwrap();

        let body = r#"{"type":"tap","x":12,"y":34}"#;
        let response = http_request(
            server.local_addr(),
            &format!(
                "POST /api/v1/input/touch HTTP/1.1\r\nHost: local\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            ),
        );
        assert!(response.contains("200 OK"));
        assert!(response.contains(r#""ok":true"#));
        let queued = server.drain_control_messages();
        assert_eq!(queued.len(), 1);
        assert_eq!(queued[0]["type"], "touch");
        assert_eq!(queued[0]["phase"], "tap");
        assert_eq!(queued[0]["x"], 12);
        assert_eq!(queued[0]["y"], 34);
    }

    #[test]
    fn remote_server_start_keeps_listener_guard_and_accepts_immediately() {
        let server = RemoteServer::start(RemoteServerConfig {
            addr: "127.0.0.1:0".parse().unwrap(),
            ..RemoteServerConfig::default()
        })
        .unwrap();

        assert_eq!(server.listener_guard_addr(), server.local_addr());
        let response = http_request(
            server.local_addr(),
            "GET /api/v1/status HTTP/1.1\r\nHost: local\r\n\r\n",
        );
        assert!(response.contains("200 OK"));
        assert!(response.contains(r#""running":true"#));
    }

    #[test]
    fn remote_server_rejects_invalid_v2_input_bodies() {
        let server = RemoteServer::start(RemoteServerConfig {
            addr: "127.0.0.1:0".parse().unwrap(),
            ..RemoteServerConfig::default()
        })
        .unwrap();

        let missing_touch_type = r#"{"x":12,"y":34}"#;
        let response = http_request(
            server.local_addr(),
            &format!(
                "POST /api/v1/input/touch HTTP/1.1\r\nHost: local\r\nContent-Length: {}\r\n\r\n{}",
                missing_touch_type.len(),
                missing_touch_type
            ),
        );
        assert!(response.contains("400 Bad Request"));
        assert!(response.contains(r#""error":"invalid touch body""#));

        let unsupported_key = r#"{"type":"tap","vk":38}"#;
        let response = http_request(
            server.local_addr(),
            &format!(
                "POST /api/v1/input/key HTTP/1.1\r\nHost: local\r\nContent-Length: {}\r\n\r\n{}",
                unsupported_key.len(),
                unsupported_key
            ),
        );
        assert!(response.contains("400 Bad Request"));
        assert!(response.contains(r#""error":"unsupported key type""#));
        assert!(server.drain_control_messages().is_empty());
    }

    #[test]
    fn remote_server_accepts_v2_single_touch_alias() {
        let server = RemoteServer::start(RemoteServerConfig {
            addr: "127.0.0.1:0".parse().unwrap(),
            ..RemoteServerConfig::default()
        })
        .unwrap();

        let body = r#"{"type":"single-touch","x":40,"y":50}"#;
        let response = http_request(
            server.local_addr(),
            &format!(
                "POST /api/v1/input/touch HTTP/1.1\r\nHost: local\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            ),
        );
        assert!(response.contains("200 OK"));
        assert!(response.contains(r#""ok":true"#));
        let queued = server.drain_control_messages();
        assert_eq!(queued.len(), 1);
        assert_eq!(queued[0]["phase"], "single-touch");
    }

    #[test]
    fn remote_server_normalizes_location_coordinate_aliases() {
        let server = RemoteServer::start(RemoteServerConfig {
            addr: "127.0.0.1:0".parse().unwrap(),
            ..RemoteServerConfig::default()
        })
        .unwrap();

        let body = r#"{"latitude":37.5665,"longitude":126.9780}"#;
        let response = http_request(
            server.local_addr(),
            &format!(
                "POST /api/v1/sensors/location HTTP/1.1\r\nHost: local\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            ),
        );

        assert!(response.contains("200 OK"));
        assert!(response.contains(r#""ok":true"#));
        let queued = server.drain_control_messages();
        assert_eq!(queued.len(), 1);
        assert_eq!(queued[0]["type"], "location");
        assert_eq!(queued[0]["lat"], 37.5665);
        assert_eq!(queued[0]["lon"], 126.9780);
    }

    #[test]
    fn remote_server_serves_frame_with_v2_quality_query_and_error() {
        let server = RemoteServer::start(RemoteServerConfig {
            addr: "127.0.0.1:0".parse().unwrap(),
            ..RemoteServerConfig::default()
        })
        .unwrap();

        let response = http_request(
            server.local_addr(),
            "GET /api/v1/frame.jpg HTTP/1.1\r\nHost: local\r\n\r\n",
        );
        assert!(response.contains("503 Service Unavailable"));
        assert!(response.contains(r#""error":"no framebuffer""#));

        let mut framebuffer = VirtualFramebuffer::new(2, 2, PixelFormat::Rgb565).unwrap();
        framebuffer.clear(0xff);
        server.publish_framebuffer(&framebuffer);
        let response = http_request_bytes(
            server.local_addr(),
            "GET /api/v1/frame.jpg?quality=1 HTTP/1.1\r\nHost: local\r\n\r\n",
        );
        let header = String::from_utf8_lossy(&response[..response.len().min(256)]);
        assert!(header.contains("200 OK"));
        assert!(header.contains("Content-Type: image/jpeg"));
    }

    #[test]
    fn remote_server_serves_v2_status_shape() {
        let server = RemoteServer::start(RemoteServerConfig {
            addr: "127.0.0.1:0".parse().unwrap(),
            ..RemoteServerConfig::default()
        })
        .unwrap();
        server.publish_status(&RemoteStatus {
            running: true,
            guest_width: 800,
            guest_height: 480,
            guest_fps: 15,
            video_enabled: true,
            video_codec: "mjpeg",
            audio_enabled: false,
            audio_codec: "pcm",
            audio_sample_rate: 44_100,
            audio_channels: 2,
            audio_format: "s16le".to_owned(),
            gps_enabled: true,
            gps_target: "COM7:".to_owned(),
            paused: false,
            queued_touch_events: 0,
            queued_key_events: 0,
            queued_serial_bytes: 0,
            audio_clients: 0,
            queued_audio_chunks: 0,
        });

        let response = http_request(
            server.local_addr(),
            "GET /api/v1/status HTTP/1.1\r\nHost: local\r\n\r\n",
        );
        assert!(response.contains("200 OK"));
        assert!(response.contains(r#""guestWidth":800"#));
        assert!(response.contains(r#""guestHeight":480"#));
    }

    #[test]
    fn remote_server_serves_recent_logs_over_rest_and_control_ws() {
        let server = RemoteServer::start(RemoteServerConfig {
            addr: "127.0.0.1:0".parse().unwrap(),
            ..RemoteServerConfig::default()
        })
        .unwrap();
        server.publish_recent_logs(vec!["first".to_owned(), "second".to_owned()]);
        server.publish_log_line("third");

        let response = http_request(
            server.local_addr(),
            "GET /api/v1/logs/recent?lines=2 HTTP/1.1\r\nHost: local\r\n\r\n",
        );
        assert!(response.contains("200 OK"));
        assert!(response.contains(r#""lines":["second","third"]"#));

        let mut stream = websocket_connect(server.local_addr(), "/api/v1/control/ws");
        stream
            .write_all(&websocket_client_frame(
                WebSocketOpcode::Text,
                br#"{"type":"logs","lines":1}"#,
            ))
            .unwrap();
        let ack = read_unmasked_server_frame(&mut stream);
        let payload = String::from_utf8_lossy(&ack.payload);
        assert_eq!(ack.opcode, WebSocketOpcode::Text);
        assert!(payload.contains(r#""type":"log""#));
        assert!(payload.contains(r#""lines":["third"]"#));
        assert!(server.drain_control_messages().is_empty());
    }

    #[test]
    fn remote_server_control_websocket_queues_json_frames() {
        let server = RemoteServer::start(RemoteServerConfig {
            addr: "127.0.0.1:0".parse().unwrap(),
            ..RemoteServerConfig::default()
        })
        .unwrap();
        let mut stream = websocket_connect(server.local_addr(), "/api/v1/control/ws");
        stream
            .write_all(&websocket_client_frame(
                WebSocketOpcode::Text,
                br#"{"type":"tap","x":12,"y":34}"#,
            ))
            .unwrap();

        let ack = read_unmasked_server_frame(&mut stream);
        assert_eq!(ack.opcode, WebSocketOpcode::Text);
        assert!(String::from_utf8_lossy(&ack.payload).contains(r#""queued":true"#));
        let queued = server.drain_control_messages();
        assert_eq!(queued.len(), 1);
        assert_eq!(queued[0]["type"], "tap");
    }

    #[test]
    fn remote_server_control_websocket_status_returns_latest_status_without_queueing() {
        let server = RemoteServer::start(RemoteServerConfig {
            addr: "127.0.0.1:0".parse().unwrap(),
            ..RemoteServerConfig::default()
        })
        .unwrap();
        server.publish_status(&RemoteStatus {
            running: true,
            guest_width: 800,
            guest_height: 480,
            guest_fps: 20,
            video_enabled: true,
            video_codec: "mjpeg",
            audio_enabled: false,
            audio_codec: "pcm",
            audio_sample_rate: 44_100,
            audio_channels: 2,
            audio_format: "s16le".to_owned(),
            gps_enabled: true,
            gps_target: "COM7:".to_owned(),
            paused: false,
            queued_touch_events: 0,
            queued_key_events: 0,
            queued_serial_bytes: 0,
            audio_clients: 0,
            queued_audio_chunks: 0,
        });

        let mut stream = websocket_connect(server.local_addr(), "/api/v1/control/ws");
        stream
            .write_all(&websocket_client_frame(
                WebSocketOpcode::Text,
                br#"{"type":"status"}"#,
            ))
            .unwrap();
        let ack = read_unmasked_server_frame(&mut stream);
        let payload = String::from_utf8_lossy(&ack.payload);
        assert_eq!(ack.opcode, WebSocketOpcode::Text);
        assert!(payload.contains(r#""type":"status""#));
        assert!(payload.contains(r#""guestWidth":800"#));
        assert!(payload.contains(r#""guestHeight":480"#));
        assert!(server.drain_control_messages().is_empty());
    }

    #[test]
    fn remote_server_audio_websocket_streams_registered_sink_pcm() {
        let server = RemoteServer::start(RemoteServerConfig {
            addr: "127.0.0.1:0".parse().unwrap(),
            audio_enabled: true,
            ..RemoteServerConfig::default()
        })
        .unwrap();
        let mut stream = websocket_connect(server.local_addr(), "/api/v1/audio/ws");
        let metadata = read_unmasked_server_frame(&mut stream);
        assert_eq!(metadata.opcode, WebSocketOpcode::Text);
        assert!(String::from_utf8_lossy(&metadata.payload).contains(r#""codec":"pcm""#));

        let format = WaveFormat::pcm_16bit(2, 44_100);
        let mut sink = server.audio_sink();
        assert_eq!(
            sink.submit_pcm(&[1, 2, 3, 4], &format, 100, 20, true),
            Some(1)
        );
        let audio = read_unmasked_server_frame(&mut stream);
        assert_eq!(audio.opcode, WebSocketOpcode::Binary);
        assert_eq!(audio.payload, vec![1, 2, 3, 4]);
    }

    fn http_request(addr: SocketAddr, request: &str) -> String {
        let mut stream = TcpStream::connect(addr).unwrap();
        stream.write_all(request.as_bytes()).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    }

    fn http_request_bytes(addr: SocketAddr, request: &str) -> Vec<u8> {
        let mut stream = TcpStream::connect(addr).unwrap();
        stream.write_all(request.as_bytes()).unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();
        response
    }

    fn websocket_connect(addr: SocketAddr, path: &str) -> TcpStream {
        let mut stream = TcpStream::connect(addr).unwrap();
        stream
            .write_all(
                format!(
                    "GET {path} HTTP/1.1\r\nHost: local\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n"
                )
                .as_bytes(),
            )
            .unwrap();
        let mut response = Vec::new();
        let mut byte = [0u8; 1];
        while !response.ends_with(b"\r\n\r\n") {
            stream.read_exact(&mut byte).unwrap();
            response.push(byte[0]);
        }
        let header = String::from_utf8_lossy(&response);
        assert!(header.contains("101 Switching Protocols"));
        assert!(header.contains("s3pPLMBiTxaQ9kYGzzhZRbK+xOo="));
        stream
    }

    fn websocket_client_frame(opcode: WebSocketOpcode, payload: &[u8]) -> Vec<u8> {
        let opcode_byte = match opcode {
            WebSocketOpcode::Text => 0x1,
            WebSocketOpcode::Binary => 0x2,
            WebSocketOpcode::Close => 0x8,
            WebSocketOpcode::Ping => 0x9,
            WebSocketOpcode::Pong => 0xA,
        };
        let mask = [0x11, 0x22, 0x33, 0x44];
        let mut frame = Vec::new();
        frame.push(0x80 | opcode_byte);
        assert!(payload.len() <= 125);
        frame.push(0x80 | payload.len() as u8);
        frame.extend_from_slice(&mask);
        for (index, byte) in payload.iter().copied().enumerate() {
            frame.push(byte ^ mask[index % 4]);
        }
        frame
    }

    fn read_unmasked_server_frame(stream: &mut TcpStream) -> WebSocketFrame {
        let mut header = [0u8; 2];
        stream.read_exact(&mut header).unwrap();
        assert_eq!(header[0] & 0x80, 0x80);
        assert_eq!(header[1] & 0x80, 0);
        let opcode = match header[0] & 0x0f {
            0x1 => WebSocketOpcode::Text,
            0x2 => WebSocketOpcode::Binary,
            0x8 => WebSocketOpcode::Close,
            0x9 => WebSocketOpcode::Ping,
            0xA => WebSocketOpcode::Pong,
            other => panic!("unexpected opcode {other}"),
        };
        let mut len = usize::from(header[1] & 0x7f);
        if len == 126 {
            let mut extended = [0u8; 2];
            stream.read_exact(&mut extended).unwrap();
            len = usize::from(u16::from_be_bytes(extended));
        } else if len == 127 {
            let mut extended = [0u8; 8];
            stream.read_exact(&mut extended).unwrap();
            len = u64::from_be_bytes(extended) as usize;
        }
        let mut payload = vec![0; len];
        stream.read_exact(&mut payload).unwrap();
        WebSocketFrame { opcode, payload }
    }
}
