use std::{
    collections::VecDeque,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const WM_KEYDOWN: u32 = 0x0100;
pub const WM_KEYUP: u32 = 0x0101;
pub const WM_MOUSEMOVE: u32 = 0x0200;
pub const WM_LBUTTONDOWN: u32 = 0x0201;
pub const WM_LBUTTONUP: u32 = 0x0202;

const DEFAULT_MAX_SERIAL_BYTES: usize = 64 * 1024;
const DEFAULT_MAX_AUDIO_CHUNKS: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteConfig {
    pub video_enabled: bool,
    pub video_fps: u32,
    pub audio_enabled: bool,
    pub audio_sample_rate: u32,
    pub audio_channels: u16,
    pub audio_format: String,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub max_serial_bytes: usize,
    pub max_audio_chunks: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TouchEvent {
    pub message: u32,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyEvent {
    pub message: u32,
    pub vk: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioChunk {
    pub payload: Vec<u8>,
    pub sequence: u64,
    pub pts_ms: u64,
    pub duration_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RemoteStatus {
    pub running: bool,
    pub guest_width: u32,
    pub guest_height: u32,
    pub guest_fps: u32,
    pub video_enabled: bool,
    pub video_codec: &'static str,
    pub audio_enabled: bool,
    pub audio_codec: &'static str,
    pub audio_sample_rate: u32,
    pub audio_channels: u16,
    pub audio_format: String,
    pub gps_enabled: bool,
    pub gps_target: String,
    pub paused: bool,
    pub queued_touch_events: usize,
    pub queued_key_events: usize,
    pub queued_serial_bytes: usize,
    pub audio_clients: usize,
    pub queued_audio_chunks: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocationFix {
    pub lat: f64,
    pub lon: f64,
    pub altitude_m: f64,
    pub speed_mps: f64,
    pub bearing_deg: f64,
    pub timestamp_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteError {
    InvalidTouchBody,
    InvalidKeyBody,
    InvalidLocationBody,
    InvalidNmeaBody,
    UnsupportedTouchType(String),
    UnsupportedKeyType(String),
    TouchOutsideFramebuffer {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    },
    FramebufferUnavailable,
    InvalidVirtualKey(u32),
}

#[derive(Debug, Clone)]
pub struct CeRemote {
    config: RemoteConfig,
    touch_events: VecDeque<TouchEvent>,
    key_events: VecDeque<KeyEvent>,
    serial_bytes: VecDeque<u8>,
    audio_chunks: VecDeque<AudioChunk>,
    audio_client_count: usize,
    audio_sequence: u64,
    audio_next_pts_ms: u64,
    imu_state: Value,
    paused: bool,
    recent_logs: VecDeque<String>,
}

impl Default for RemoteConfig {
    fn default() -> Self {
        Self {
            video_enabled: true,
            video_fps: 15,
            audio_enabled: true,
            audio_sample_rate: 44_100,
            audio_channels: 2,
            audio_format: "s16le".to_owned(),
            framebuffer_width: 0,
            framebuffer_height: 0,
            max_serial_bytes: DEFAULT_MAX_SERIAL_BYTES,
            max_audio_chunks: DEFAULT_MAX_AUDIO_CHUNKS,
        }
    }
}

impl Default for LocationFix {
    fn default() -> Self {
        Self {
            lat: 0.0,
            lon: 0.0,
            altitude_m: 50.0,
            speed_mps: 0.0,
            bearing_deg: 0.0,
            timestamp_ms: None,
        }
    }
}

impl std::fmt::Display for RemoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTouchBody => write!(f, "invalid touch body"),
            Self::InvalidKeyBody => write!(f, "invalid key body"),
            Self::InvalidLocationBody => write!(f, "invalid location body"),
            Self::InvalidNmeaBody => write!(f, "invalid nmea body"),
            Self::UnsupportedTouchType(phase) => write!(f, "unsupported touch type: {phase}"),
            Self::UnsupportedKeyType(phase) => write!(f, "unsupported key type: {phase}"),
            Self::TouchOutsideFramebuffer {
                x,
                y,
                width,
                height,
            } => write!(
                f,
                "touch point ({x},{y}) outside guest framebuffer {width}x{height}"
            ),
            Self::FramebufferUnavailable => write!(f, "guest framebuffer is unavailable"),
            Self::InvalidVirtualKey(vk) => write!(f, "vk must be between 1 and 255, got {vk}"),
        }
    }
}

impl std::error::Error for RemoteError {}

impl Default for CeRemote {
    fn default() -> Self {
        Self::new(RemoteConfig::default())
    }
}

impl CeRemote {
    pub fn new(config: RemoteConfig) -> Self {
        Self {
            config,
            touch_events: VecDeque::new(),
            key_events: VecDeque::new(),
            serial_bytes: VecDeque::new(),
            audio_chunks: VecDeque::new(),
            audio_client_count: 0,
            audio_sequence: 0,
            audio_next_pts_ms: 0,
            imu_state: Value::Object(Default::default()),
            paused: false,
            recent_logs: VecDeque::new(),
        }
    }

    pub fn config(&self) -> &RemoteConfig {
        &self.config
    }

    pub fn set_framebuffer_size(&mut self, width: u32, height: u32) {
        self.config.framebuffer_width = width;
        self.config.framebuffer_height = height;
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn enqueue_touch(&mut self, phase: &str, x: i32, y: i32) -> Result<(), RemoteError> {
        let messages = touch_messages(phase)?;
        let width = self.config.framebuffer_width;
        let height = self.config.framebuffer_height;
        if width == 0 || height == 0 {
            return Err(RemoteError::FramebufferUnavailable);
        }
        if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
            return Err(RemoteError::TouchOutsideFramebuffer {
                x,
                y,
                width,
                height,
            });
        }

        for message in messages {
            self.touch_events.push_back(TouchEvent { message, x, y });
        }
        Ok(())
    }

    pub fn drain_touch_events(&mut self) -> Vec<TouchEvent> {
        self.touch_events.drain(..).collect()
    }

    pub fn touch_event_count(&self) -> usize {
        self.touch_events.len()
    }

    pub fn enqueue_key(&mut self, phase: &str, vk: u32) -> Result<(), RemoteError> {
        let message = match phase {
            "down" => WM_KEYDOWN,
            "up" => WM_KEYUP,
            other => return Err(RemoteError::UnsupportedKeyType(other.to_owned())),
        };
        if !(1..=0xff).contains(&vk) {
            return Err(RemoteError::InvalidVirtualKey(vk));
        }
        self.key_events.push_back(KeyEvent { message, vk });
        Ok(())
    }

    pub fn drain_key_events(&mut self) -> Vec<KeyEvent> {
        self.key_events.drain(..).collect()
    }

    pub fn key_event_count(&self) -> usize {
        self.key_events.len()
    }

    pub fn inject_serial_bytes(&mut self, bytes: impl AsRef<[u8]>) {
        for byte in bytes.as_ref() {
            self.serial_bytes.push_back(*byte);
        }
        while self.serial_bytes.len() > self.config.max_serial_bytes {
            self.serial_bytes.pop_front();
        }
    }

    pub fn read_serial_bytes(&mut self, max_bytes: usize) -> Vec<u8> {
        let count = max_bytes.min(self.serial_bytes.len());
        self.serial_bytes.drain(..count).collect()
    }

    pub fn serial_byte_count(&self) -> usize {
        self.serial_bytes.len()
    }

    pub fn inject_nmea_sentences<I, S>(&mut self, sentences: I) -> usize
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut accepted = 0;
        let mut bytes = Vec::new();
        for sentence in sentences {
            bytes.extend_from_slice(normalize_nmea_sentence(sentence.as_ref()).as_bytes());
            accepted += 1;
        }
        self.inject_serial_bytes(bytes);
        accepted
    }

    pub fn inject_location_nmea(&mut self, fix: LocationFix) -> Vec<String> {
        let sentences = make_location_nmea(&fix);
        self.inject_nmea_sentences(sentences.iter().map(String::as_str));
        sentences
    }

    pub fn update_imu_state(&mut self, state: Value) {
        self.imu_state = state;
    }

    pub fn imu_state(&self) -> &Value {
        &self.imu_state
    }

    pub fn register_audio_client(&mut self, now_ms: u64) {
        if self.audio_client_count == 0 {
            self.audio_chunks.clear();
            self.audio_next_pts_ms = now_ms;
        }
        self.audio_client_count += 1;
    }

    pub fn unregister_audio_client(&mut self) {
        self.audio_client_count = self.audio_client_count.saturating_sub(1);
        if self.audio_client_count == 0 {
            self.audio_chunks.clear();
            self.audio_next_pts_ms = 0;
        }
    }

    pub fn audio_client_count(&self) -> usize {
        self.audio_client_count
    }

    pub fn publish_audio_chunk(&mut self, payload: Vec<u8>, duration_ms: u32) -> Option<u64> {
        if payload.is_empty() || !self.config.audio_enabled || self.audio_client_count == 0 {
            return None;
        }
        self.audio_sequence += 1;
        let sequence = self.audio_sequence;
        let duration_ms = duration_ms.max(1);
        let chunk = AudioChunk {
            payload,
            sequence,
            pts_ms: self.audio_next_pts_ms,
            duration_ms,
        };
        self.audio_next_pts_ms = self
            .audio_next_pts_ms
            .saturating_add(u64::from(duration_ms));
        self.audio_chunks.push_back(chunk);
        while self.audio_chunks.len() > self.config.max_audio_chunks {
            self.audio_chunks.pop_front();
        }
        Some(sequence)
    }

    pub fn clear_audio_chunks(&mut self, now_ms: u64) {
        self.audio_chunks.clear();
        self.audio_next_pts_ms = now_ms;
    }

    pub fn take_audio_chunks(&mut self, max_chunks: usize) -> Vec<AudioChunk> {
        let count = max_chunks.min(self.audio_chunks.len());
        self.audio_chunks.drain(..count).collect()
    }

    pub fn audio_chunk_count(&self) -> usize {
        self.audio_chunks.len()
    }

    pub fn push_log_line(&mut self, line: impl Into<String>) {
        self.recent_logs.push_back(clamp_text(line.into(), 4096));
        while self.recent_logs.len() > 4096 {
            self.recent_logs.pop_front();
        }
    }

    pub fn recent_log_lines(&self, max_lines: usize) -> Vec<String> {
        let count = max_lines.clamp(1, 4096).min(self.recent_logs.len());
        self.recent_logs
            .iter()
            .skip(self.recent_logs.len() - count)
            .cloned()
            .collect()
    }

    pub fn status(&self, gps_target: impl Into<String>) -> RemoteStatus {
        RemoteStatus {
            running: true,
            guest_width: self.config.framebuffer_width,
            guest_height: self.config.framebuffer_height,
            guest_fps: self.config.video_fps,
            video_enabled: self.config.video_enabled,
            video_codec: "mjpeg",
            audio_enabled: self.config.audio_enabled,
            audio_codec: "pcm",
            audio_sample_rate: self.config.audio_sample_rate,
            audio_channels: self.config.audio_channels,
            audio_format: self.config.audio_format.clone(),
            gps_enabled: true,
            gps_target: gps_target.into(),
            paused: self.paused,
            queued_touch_events: self.touch_events.len(),
            queued_key_events: self.key_events.len(),
            queued_serial_bytes: self.serial_bytes.len(),
            audio_clients: self.audio_client_count,
            queued_audio_chunks: self.audio_chunks.len(),
        }
    }

    pub fn status_json(&self, gps_target: impl Into<String>) -> Value {
        serde_json::to_value(self.status(gps_target)).expect("RemoteStatus serializes")
    }

    pub fn dispatch_control_message(
        &mut self,
        message: &Value,
        gps_target: impl Into<String>,
    ) -> Value {
        let Some(kind) = message.get("type").and_then(Value::as_str) else {
            return json!({"type": "error", "ok": false, "error": "invalid control message"});
        };
        let gps_target = gps_target.into();
        match kind {
            "touch" => {
                let phase = message
                    .get("phase")
                    .and_then(Value::as_str)
                    .unwrap_or("tap");
                let Some(x) = message.get("x").and_then(Value::as_i64) else {
                    return json!({"type": "error", "ok": false, "error": RemoteError::InvalidTouchBody.to_string()});
                };
                let Some(y) = message.get("y").and_then(Value::as_i64) else {
                    return json!({"type": "error", "ok": false, "error": RemoteError::InvalidTouchBody.to_string()});
                };
                self.enqueue_touch(phase, x as i32, y as i32)
                    .map(|_| json!({"type": "status", "ok": true, "status": self.status_json(gps_target)}))
                    .unwrap_or_else(error_response)
            }
            "key" => {
                let phase = message
                    .get("phase")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let Some(vk) = message.get("vk").and_then(Value::as_u64) else {
                    return json!({"type": "error", "ok": false, "error": RemoteError::InvalidKeyBody.to_string()});
                };
                self.enqueue_key(phase, vk as u32)
                    .map(|_| json!({"type": "status", "ok": true, "status": self.status_json(gps_target)}))
                    .unwrap_or_else(error_response)
            }
            "location" => parse_location_fix(message)
                .map(|fix| {
                    let sentences = self.inject_location_nmea(fix);
                    json!({"type": "status", "ok": true, "sentencesGenerated": sentences.len()})
                })
                .unwrap_or_else(error_response),
            "nmea" => parse_nmea_sentences(message)
                .map(|sentences| {
                    let accepted = self.inject_nmea_sentences(sentences.iter().map(String::as_str));
                    json!({"type": "status", "ok": true, "accepted": accepted})
                })
                .unwrap_or_else(error_response),
            "imu" => {
                self.update_imu_state(message.clone());
                json!({"type": "status", "ok": true})
            }
            "pause" => {
                self.set_paused(true);
                json!({"type": "status", "ok": true, "paused": true})
            }
            "resume" => {
                self.set_paused(false);
                json!({"type": "status", "ok": true, "paused": false})
            }
            "status" => {
                json!({"type": "status", "ok": true, "status": self.status_json(gps_target)})
            }
            "logs" => {
                let lines = message
                    .get("lines")
                    .and_then(Value::as_u64)
                    .unwrap_or(200)
                    .clamp(1, 4096) as usize;
                json!({"type": "log", "ok": true, "lines": self.recent_log_lines(lines)})
            }
            _ => json!({"type": "error", "ok": false, "error": "unsupported control message type"}),
        }
    }
}

pub fn normalize_nmea_sentence(sentence: &str) -> String {
    let mut normalized = sentence.trim_end_matches(['\r', '\n']).to_owned();
    normalized.push_str("\r\n");
    normalized
}

pub fn nmea_checksum_line(body: &str) -> String {
    let checksum = body.bytes().fold(0u8, |checksum, byte| checksum ^ byte);
    format!("${body}*{checksum:02X}\r\n")
}

pub fn make_location_nmea(fix: &LocationFix) -> Vec<String> {
    let timestamp_ms = fix.timestamp_ms.unwrap_or_else(system_time_ms);
    let utc = utc_from_unix_ms(timestamp_ms);
    let time_text = format!("{:02}{:02}{:02}.000", utc.hour, utc.minute, utc.second);
    let date_text = format!("{:02}{:02}{:02}", utc.day, utc.month, utc.year % 100);
    let (lat_text, ns) = format_nmea_coord(fix.lat, true);
    let (lon_text, ew) = format_nmea_coord(fix.lon, false);
    let speed_knots = fix.speed_mps * 1.943_844_492_440_6;
    let speed_kmh = fix.speed_mps * 3.6;

    let rmc = format!(
        "GPRMC,{time_text},A,{lat_text},{ns},{lon_text},{ew},{speed_knots:.1},{:.1},{date_text},,,A",
        fix.bearing_deg
    );
    let gga = format!(
        "GPGGA,{time_text},{lat_text},{ns},{lon_text},{ew},1,08,0.9,{:.1},M,19.5,M,,",
        fix.altitude_m
    );
    let vtg = format!(
        "GPVTG,{:.1},T,,M,{speed_knots:.1},N,{speed_kmh:.1},K,A",
        fix.bearing_deg
    );

    vec![
        nmea_checksum_line(&rmc),
        nmea_checksum_line(&gga),
        nmea_checksum_line(&vtg),
    ]
}

pub fn make_lparam(x: i32, y: i32) -> u32 {
    ((y as u32) & 0xffff) << 16 | ((x as u32) & 0xffff)
}

fn touch_messages(phase: &str) -> Result<Vec<u32>, RemoteError> {
    match phase {
        "down" => Ok(vec![WM_LBUTTONDOWN]),
        "move" => Ok(vec![WM_MOUSEMOVE]),
        "up" | "cancel" => Ok(vec![WM_LBUTTONUP]),
        "" | "tap" | "click" | "single" | "single-touch" => Ok(vec![WM_LBUTTONDOWN, WM_LBUTTONUP]),
        other => Err(RemoteError::UnsupportedTouchType(other.to_owned())),
    }
}

fn parse_location_fix(message: &Value) -> Result<LocationFix, RemoteError> {
    let Some(lat) = message.get("lat").and_then(Value::as_f64) else {
        return Err(RemoteError::InvalidLocationBody);
    };
    let Some(lon) = message.get("lon").and_then(Value::as_f64) else {
        return Err(RemoteError::InvalidLocationBody);
    };
    Ok(LocationFix {
        lat,
        lon,
        altitude_m: message
            .get("altitudeM")
            .and_then(Value::as_f64)
            .unwrap_or(50.0),
        speed_mps: message
            .get("speedMps")
            .and_then(Value::as_f64)
            .unwrap_or(0.0),
        bearing_deg: message
            .get("bearingDeg")
            .and_then(Value::as_f64)
            .unwrap_or(0.0),
        timestamp_ms: message.get("timestampMs").and_then(Value::as_u64),
    })
}

fn parse_nmea_sentences(message: &Value) -> Result<Vec<String>, RemoteError> {
    let Some(sentences) = message.get("sentences").and_then(Value::as_array) else {
        return Err(RemoteError::InvalidNmeaBody);
    };
    Ok(sentences
        .iter()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect())
}

fn error_response(error: RemoteError) -> Value {
    json!({"type": "error", "ok": false, "error": error.to_string()})
}

fn clamp_text(mut value: String, max_len: usize) -> String {
    if value.len() <= max_len {
        return value;
    }
    let mut end = max_len;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    value.truncate(end);
    value
}

fn format_nmea_coord(value: f64, latitude: bool) -> (String, char) {
    let abs_value = value.abs();
    let degrees = abs_value.floor() as u32;
    let minutes = (abs_value - f64::from(degrees)) * 60.0;
    let hemisphere = if latitude {
        if value >= 0.0 { 'N' } else { 'S' }
    } else if value >= 0.0 {
        'E'
    } else {
        'W'
    };
    let degree_width = if latitude { 2 } else { 3 };
    (
        format!("{degrees:0degree_width$}{minutes:07.4}"),
        hemisphere,
    )
}

fn system_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Copy)]
struct UtcFields {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
}

fn utc_from_unix_ms(timestamp_ms: u64) -> UtcFields {
    let total_seconds = (timestamp_ms / 1000) as i64;
    let days = total_seconds.div_euclid(86_400);
    let seconds_of_day = total_seconds.rem_euclid(86_400) as u32;
    let (year, month, day) = civil_from_days(days);
    UtcFields {
        year,
        month,
        day,
        hour: seconds_of_day / 3600,
        minute: (seconds_of_day % 3600) / 60,
        second: seconds_of_day % 60,
    }
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };
    (year as i32, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queues_touch_key_serial_and_status() {
        let mut remote = CeRemote::default();
        remote.set_framebuffer_size(800, 480);

        remote.enqueue_touch("tap", 10, 20).unwrap();
        remote.enqueue_key("down", 0x26).unwrap();
        remote.inject_serial_bytes(b"abc");

        assert_eq!(remote.touch_event_count(), 2);
        assert_eq!(remote.key_event_count(), 1);
        assert_eq!(remote.serial_byte_count(), 3);
        assert_eq!(remote.drain_touch_events()[0].message, WM_LBUTTONDOWN);
        assert_eq!(remote.read_serial_bytes(2), b"ab");
        assert_eq!(remote.status("COM7:").gps_target, "COM7:");
    }

    #[test]
    fn generates_nmea_location_sentences() {
        let fix = LocationFix {
            lat: 37.5,
            lon: 127.25,
            altitude_m: 42.0,
            speed_mps: 10.0,
            bearing_deg: 90.0,
            timestamp_ms: Some(0),
        };
        let sentences = make_location_nmea(&fix);

        assert_eq!(sentences.len(), 3);
        assert!(sentences[0].starts_with("$GPRMC,000000.000,A,3730.0000,N,12715.0000,E"));
        assert!(sentences[0].ends_with("\r\n"));
    }

    #[test]
    fn dispatches_control_messages() {
        let mut remote = CeRemote::default();
        remote.set_framebuffer_size(320, 240);

        let response =
            remote.dispatch_control_message(&json!({"type": "touch", "x": 5, "y": 6}), "COM7:");
        assert_eq!(response["ok"], true);
        assert_eq!(remote.touch_event_count(), 2);

        let response = remote.dispatch_control_message(&json!({"type": "pause"}), "COM7:");
        assert_eq!(response["paused"], true);
        assert!(remote.paused());
    }
}
