use std::{
    collections::VecDeque,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use image::{ColorType, ImageEncoder, codecs::jpeg::JpegEncoder, codecs::png::PngEncoder};
use serde_json::{Value, json};

use crate::{
    ce::{
        framebuffer::{Framebuffer, PixelFormat},
        remote::RemoteStatus,
    },
    error::{Error, Result},
};

const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const MAX_PENDING_CONTROL_MESSAGES: usize = 1024;
const DEFAULT_VIDEO_FPS: u32 = 30;
const DEFAULT_JPEG_QUALITY: u8 = 80;

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
    pending_control: Mutex<VecDeque<Value>>,
    latest_status: Mutex<Value>,
    latest_framebuffer: Mutex<Option<RemoteFramebufferImage>>,
}

#[derive(Debug, Clone)]
struct RemoteFramebufferImage {
    width: u32,
    height: u32,
    rgb: Vec<u8>,
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
            }),
            local_addr,
        };
        let worker = server.clone();
        thread::Builder::new()
            .name("wince-remote-server".to_owned())
            .spawn(move || worker.serve(listener))
            .map_err(|err| Error::Backend(format!("start remote server thread: {err}")))?;
        println!("  remote server: http://{local_addr}");
        Ok(server)
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    pub fn publish_status(&self, status: &RemoteStatus) {
        let mut value = v2_status_json(status);
        if let Some(object) = value.as_object_mut() {
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
        }
        *self
            .state
            .latest_status
            .lock()
            .expect("remote status mutex") = value;
    }

    pub fn publish_framebuffer(&self, framebuffer: &dyn Framebuffer) {
        let image = framebuffer_to_rgb(framebuffer);
        *self
            .state
            .latest_framebuffer
            .lock()
            .expect("remote framebuffer mutex") = Some(image);
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

    fn serve(self, listener: TcpListener) {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
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
            ("GET", "/api/v1/video.mjpg") => RemoteHttpResponse::Mjpeg { request },
            ("GET", "/framebuffer.ppm") => self.latest_ppm_response().into(),
            ("POST", "/api/v1/input/touch") => self.post_touch(request).into(),
            ("POST", "/api/v1/input/key") => self.post_key(request).into(),
            ("POST", "/api/v1/sensors/location") => self.post_location(request).into(),
            ("POST", "/api/v1/sensors/nmea") => self.post_nmea(request).into(),
            ("POST", "/api/v1/sensors/imu") => self.post_imu(request).into(),
            ("GET", "/api/v1/logs/recent") => {
                let _lines = request.query_u64("lines", 200, 1, 4096);
                HttpResponse::json(200, json!({"ok": true, "lines": Vec::<String>::new()})).into()
            }
            ("POST", "/api/v1/control/pause") => {
                self.queue_control(json!({"type": "pause"}));
                HttpResponse::json(200, json!({"ok": true, "paused": true})).into()
            }
            ("POST", "/api/v1/control/resume") => {
                self.queue_control(json!({"type": "resume"}));
                HttpResponse::json(200, json!({"ok": true, "paused": false})).into()
            }
            ("GET", "/api/v1/audio/ws") | ("GET", "/api/v1/control/ws") => HttpResponse::json(
                501,
                json!({"ok": false, "error": "websocket transport is not implemented in Rust server yet"}),
            )
            .into(),
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
        self.queue_control(json!({
            "type": "touch",
            "phase": phase,
            "x": x,
            "y": y
        }));
        HttpResponse::json(200, json!({"ok": true}))
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
        self.queue_control(json!({
            "type": "key",
            "phase": kind,
            "vk": vk
        }));
        HttpResponse::json(200, json!({"ok": true}))
    }

    fn post_location(&self, request: HttpRequest) -> HttpResponse {
        let mut body = match parse_json_body(&request, "invalid location body") {
            Ok(body) => body,
            Err(response) => return response,
        };
        if body.get("lat").and_then(Value::as_f64).is_none()
            || body.get("lon").and_then(Value::as_f64).is_none()
        {
            return HttpResponse::json(400, json!({"ok": false, "error": "invalid location body"}));
        }
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

fn queue_control_message(state: &RemoteServerState, message: Value) -> usize {
    let mut pending = state.pending_control.lock().expect("remote control mutex");
    pending.push_back(message);
    while pending.len() > MAX_PENDING_CONTROL_MESSAGES {
        pending.pop_front();
    }
    pending.len()
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
    Mjpeg { request: HttpRequest },
}

impl From<HttpResponse> for RemoteHttpResponse {
    fn from(value: HttpResponse) -> Self {
        Self::One(value)
    }
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
}
