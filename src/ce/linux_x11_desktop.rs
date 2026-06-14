use std::{
    collections::{BTreeMap, VecDeque},
    path::Path,
    sync::{Mutex, OnceLock, mpsc},
    thread,
    time::Duration,
};

use x11rb::{
    COPY_FROM_PARENT,
    connection::Connection,
    protocol::{
        Event,
        xproto::{
            AtomEnum, ConnectionExt, CreateGCAux, CreateWindowAux, EventMask, Gcontext,
            ImageFormat, ImageOrder, PropMode, Screen, Visualtype, Window, WindowClass,
        },
    },
    rust_connection::RustConnection,
    wrapper::ConnectionExt as _,
};

use crate::{
    Error, Result,
    ce::{
        desktop::{Input, Presentation, Presenter, VirtualInputEvent},
        framebuffer::{Framebuffer, FramebufferInfo, FramebufferSnapshot, PixelFormat},
    },
};

const POINTER_BUTTON_LEFT: u8 = 1;

static INPUT_EVENTS: OnceLock<Mutex<VecDeque<VirtualInputEvent>>> = OnceLock::new();
static TOUCH_DOWN: OnceLock<Mutex<bool>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct X11VisualMasks {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum X11ByteOrder {
    LsbFirst,
    MsbFirst,
}

#[derive(Debug)]
enum LinuxWindowCommand {
    Frame(FramebufferSnapshot),
    Stopped,
    Quit,
}

#[derive(Debug)]
pub struct LinuxX11Input;

#[derive(Debug)]
pub struct LinuxX11Presenter {
    tx: mpsc::Sender<LinuxWindowCommand>,
    title: String,
    last_presentation: Option<Presentation>,
    presentation_count: u64,
}

#[derive(Debug)]
struct X11Runtime {
    conn: RustConnection,
    window: Window,
    gc: Gcontext,
    depth: u8,
    byte_order: X11ByteOrder,
    visual_masks: X11VisualMasks,
    keyboard: BTreeMap<u8, u32>,
    last_frame: Option<FramebufferSnapshot>,
    stopped: bool,
}

impl LinuxX11Input {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LinuxX11Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Input for LinuxX11Input {
    fn poll_events(&mut self) -> Result<Vec<VirtualInputEvent>> {
        let mut events = input_events()
            .lock()
            .map_err(|_| Error::Backend("X11 input queue lock is poisoned".to_owned()))?;
        Ok(events.drain(..).collect())
    }
}

impl LinuxX11Presenter {
    pub fn new(
        width: u32,
        height: u32,
        title: impl Into<String>,
        _icon_path: Option<&Path>,
    ) -> Result<Self> {
        let title = title.into();
        let (tx, rx) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::channel();
        let thread_title = title.clone();
        thread::Builder::new()
            .name("wince-linux-x11-presenter".to_owned())
            .spawn(move || {
                let result = X11Runtime::new(width, height, &thread_title)
                    .map_err(|err| format!("create Linux X11 desktop window: {err}"));
                let mut runtime = match result {
                    Ok(runtime) => {
                        let _ = ready_tx.send(Ok(()));
                        runtime
                    }
                    Err(err) => {
                        let _ = ready_tx.send(Err(err));
                        return;
                    }
                };
                runtime.run(rx);
            })
            .map_err(|err| Error::Backend(format!("spawn Linux X11 desktop thread: {err}")))?;
        ready_rx
            .recv()
            .map_err(|err| Error::Backend(format!("receive Linux X11 desktop startup: {err}")))?
            .map_err(Error::Backend)?;
        Ok(Self {
            tx,
            title,
            last_presentation: None,
            presentation_count: 0,
        })
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn last_presentation(&self) -> Option<&Presentation> {
        self.last_presentation.as_ref()
    }

    pub fn presentation_count(&self) -> u64 {
        self.presentation_count
    }

    pub fn pump_messages(&mut self) {}

    pub fn show_stopped_message(&mut self, _message: impl Into<String>) -> Result<()> {
        if let Ok(mut events) = input_events().lock() {
            events.clear();
        }
        if let Ok(mut down) = touch_down().lock() {
            *down = false;
        }
        self.tx
            .send(LinuxWindowCommand::Stopped)
            .map_err(|err| Error::Backend(format!("send Linux X11 stopped frame: {err}")))
    }

    pub fn blit(&mut self, framebuffer: &dyn Framebuffer) -> Result<()> {
        let snapshot = FramebufferSnapshot {
            info: framebuffer.info(),
            pixels: framebuffer.pixels().to_vec(),
        };
        self.tx
            .send(LinuxWindowCommand::Frame(snapshot))
            .map_err(|err| Error::Backend(format!("send Linux X11 framebuffer: {err}")))
    }
}

impl Presenter for LinuxX11Presenter {
    fn present(&mut self, framebuffer: &dyn Framebuffer) -> Result<Presentation> {
        let snapshot = FramebufferSnapshot {
            info: framebuffer.info(),
            pixels: framebuffer.pixels().to_vec(),
        };
        let presentation = Presentation {
            framebuffer: snapshot.clone(),
            dirty_rects: framebuffer.dirty_rects().to_vec(),
        };
        self.tx
            .send(LinuxWindowCommand::Frame(snapshot))
            .map_err(|err| Error::Backend(format!("send Linux X11 presentation: {err}")))?;
        self.last_presentation = Some(presentation.clone());
        self.presentation_count = self.presentation_count.saturating_add(1);
        Ok(presentation)
    }
}

impl Drop for LinuxX11Presenter {
    fn drop(&mut self) {
        let _ = self.tx.send(LinuxWindowCommand::Quit);
    }
}

impl X11Runtime {
    fn new(width: u32, height: u32, title: &str) -> Result<Self> {
        let (conn, screen_num) =
            x11rb::connect(None).map_err(|err| Error::Backend(format!("connect to X11: {err}")))?;
        let screen = conn.setup().roots[screen_num].clone();
        let visual = root_visual(&screen)?;
        let depth = screen.root_depth;
        let byte_order = x11_byte_order(conn.setup().image_byte_order);
        let visual_masks = X11VisualMasks {
            red: visual.red_mask,
            green: visual.green_mask,
            blue: visual.blue_mask,
        };
        let window = conn
            .generate_id()
            .map_err(|err| Error::Backend(format!("generate X11 window id: {err}")))?;
        let event_mask = EventMask::EXPOSURE
            | EventMask::KEY_PRESS
            | EventMask::KEY_RELEASE
            | EventMask::BUTTON_PRESS
            | EventMask::BUTTON_RELEASE
            | EventMask::POINTER_MOTION
            | EventMask::STRUCTURE_NOTIFY;
        conn.create_window(
            COPY_FROM_PARENT as u8,
            window,
            screen.root,
            0,
            0,
            width.min(u32::from(u16::MAX)) as u16,
            height.min(u32::from(u16::MAX)) as u16,
            0,
            WindowClass::INPUT_OUTPUT,
            screen.root_visual,
            &CreateWindowAux::new()
                .background_pixel(screen.black_pixel)
                .event_mask(event_mask),
        )
        .map_err(|err| Error::Backend(format!("create X11 window: {err}")))?;
        conn.change_property8(
            PropMode::REPLACE,
            window,
            AtomEnum::WM_NAME,
            AtomEnum::STRING,
            title.as_bytes(),
        )
        .map_err(|err| Error::Backend(format!("set X11 window title: {err}")))?;
        let gc = conn
            .generate_id()
            .map_err(|err| Error::Backend(format!("generate X11 GC id: {err}")))?;
        conn.create_gc(gc, window, &CreateGCAux::new())
            .map_err(|err| Error::Backend(format!("create X11 GC: {err}")))?;
        conn.map_window(window)
            .map_err(|err| Error::Backend(format!("map X11 window: {err}")))?;
        conn.flush()
            .map_err(|err| Error::Backend(format!("flush X11 startup: {err}")))?;
        let keyboard = keyboard_map(&conn);
        Ok(Self {
            conn,
            window,
            gc,
            depth,
            byte_order,
            visual_masks,
            keyboard,
            last_frame: None,
            stopped: false,
        })
    }

    fn run(&mut self, rx: mpsc::Receiver<LinuxWindowCommand>) {
        loop {
            while let Ok(command) = rx.try_recv() {
                if self.handle_command(command) {
                    return;
                }
            }
            while let Ok(Some(event)) = self.conn.poll_for_event() {
                self.handle_event(event);
            }
            if let Ok(command) = rx.recv_timeout(Duration::from_millis(16))
                && self.handle_command(command)
            {
                return;
            }
        }
    }

    fn handle_command(&mut self, command: LinuxWindowCommand) -> bool {
        match command {
            LinuxWindowCommand::Frame(frame) => {
                self.stopped = false;
                self.draw_frame(&frame);
                self.last_frame = Some(frame);
                false
            }
            LinuxWindowCommand::Stopped => {
                self.stopped = true;
                if let Ok(mut down) = touch_down().lock() {
                    *down = false;
                }
                let _ = self.conn.clear_area(false, self.window, 0, 0, 0, 0);
                let _ = self.conn.flush();
                false
            }
            LinuxWindowCommand::Quit => true,
        }
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Expose(_) => {
                if !self.stopped
                    && let Some(frame) = self.last_frame.clone()
                {
                    self.draw_frame(&frame);
                }
            }
            Event::ButtonPress(event) => {
                if self.stopped || event.detail != POINTER_BUTTON_LEFT {
                    return;
                }
                set_touch_down(true);
                push_input_event(VirtualInputEvent::TouchDown {
                    x: i32::from(event.event_x),
                    y: i32::from(event.event_y),
                });
            }
            Event::ButtonRelease(event) => {
                if self.stopped || event.detail != POINTER_BUTTON_LEFT {
                    return;
                }
                set_touch_down(false);
                push_input_event(VirtualInputEvent::TouchUp {
                    x: i32::from(event.event_x),
                    y: i32::from(event.event_y),
                });
            }
            Event::MotionNotify(event) => {
                if self.stopped || !is_touch_down() {
                    return;
                }
                push_input_event(VirtualInputEvent::TouchMove {
                    x: i32::from(event.event_x),
                    y: i32::from(event.event_y),
                });
            }
            Event::KeyPress(event) => {
                if self.stopped {
                    return;
                }
                if let Some(virtual_key) = self.keyboard.get(&event.detail).copied() {
                    push_input_event(VirtualInputEvent::Key {
                        virtual_key,
                        pressed: true,
                    });
                }
            }
            Event::KeyRelease(event) => {
                if self.stopped {
                    return;
                }
                if let Some(virtual_key) = self.keyboard.get(&event.detail).copied() {
                    push_input_event(VirtualInputEvent::Key {
                        virtual_key,
                        pressed: false,
                    });
                }
            }
            Event::MappingNotify(_) => {
                self.keyboard = keyboard_map(&self.conn);
            }
            _ => {}
        }
    }

    fn draw_frame(&mut self, frame: &FramebufferSnapshot) {
        let data = framebuffer_to_ximage_data(
            frame.info,
            &frame.pixels,
            self.visual_masks,
            self.byte_order,
        );
        let width = frame.info.width.min(u32::from(u16::MAX)) as u16;
        let height = frame.info.height.min(u32::from(u16::MAX)) as u16;
        let result = self.conn.put_image(
            ImageFormat::Z_PIXMAP,
            self.window,
            self.gc,
            width,
            height,
            0,
            0,
            0,
            self.depth,
            &data,
        );
        if result.is_ok() {
            let _ = self.conn.flush();
        }
    }
}

fn root_visual(screen: &Screen) -> Result<Visualtype> {
    screen
        .allowed_depths
        .iter()
        .flat_map(|depth| depth.visuals.iter())
        .find(|visual| visual.visual_id == screen.root_visual)
        .copied()
        .ok_or_else(|| {
            Error::Backend(format!(
                "X11 root visual {} was not found",
                screen.root_visual
            ))
        })
}

fn keyboard_map(conn: &RustConnection) -> BTreeMap<u8, u32> {
    let setup = conn.setup();
    let min = setup.min_keycode;
    let count = setup.max_keycode.saturating_sub(min).saturating_add(1);
    let Ok(cookie) = conn.get_keyboard_mapping(min, count) else {
        return BTreeMap::new();
    };
    let Ok(reply) = cookie.reply() else {
        return BTreeMap::new();
    };
    let syms_per_code = usize::from(reply.keysyms_per_keycode).max(1);
    let mut map = BTreeMap::new();
    for key_index in 0..usize::from(count) {
        let keycode = min.saturating_add(key_index as u8);
        let base = key_index.saturating_mul(syms_per_code);
        if let Some(virtual_key) = reply.keysyms[base..base + syms_per_code]
            .iter()
            .copied()
            .find_map(keysym_to_virtual_key)
        {
            map.insert(keycode, virtual_key);
        }
    }
    map
}

fn keysym_to_virtual_key(keysym: u32) -> Option<u32> {
    match keysym {
        0xff08 => Some(0x08),
        0xff09 => Some(0x09),
        0xff0d => Some(0x0d),
        0xff1b => Some(0x1b),
        0xff51 => Some(0x25),
        0xff52 => Some(0x26),
        0xff53 => Some(0x27),
        0xff54 => Some(0x28),
        0x20 => Some(0x20),
        0x30..=0x39 => Some(keysym),
        0x61..=0x7a => Some(keysym - 0x20),
        0x41..=0x5a => Some(keysym),
        _ => None,
    }
}

fn x11_byte_order(order: ImageOrder) -> X11ByteOrder {
    if order == ImageOrder::MSB_FIRST {
        X11ByteOrder::MsbFirst
    } else {
        X11ByteOrder::LsbFirst
    }
}

pub fn framebuffer_to_ximage_data(
    info: FramebufferInfo,
    pixels: &[u8],
    masks: X11VisualMasks,
    byte_order: X11ByteOrder,
) -> Vec<u8> {
    let bytes_per_pixel = if masks.red | masks.green | masks.blue <= 0xffff {
        2
    } else {
        4
    };
    let mut out = Vec::with_capacity(info.width as usize * info.height as usize * bytes_per_pixel);
    for y in 0..info.height as usize {
        let row_start = y.saturating_mul(info.stride);
        if row_start >= pixels.len() {
            break;
        }
        let row = &pixels[row_start..];
        for x in 0..info.width as usize {
            let src_offset = x.saturating_mul(info.format.bytes_per_pixel());
            if src_offset + info.format.bytes_per_pixel() > row.len() {
                out.extend(std::iter::repeat_n(0, bytes_per_pixel));
                continue;
            }
            let [r, g, b] = pixel_to_rgb(info.format, &row[src_offset..]);
            let pixel = channel_to_mask(r, masks.red)
                | channel_to_mask(g, masks.green)
                | channel_to_mask(b, masks.blue);
            match (bytes_per_pixel, byte_order) {
                (2, X11ByteOrder::LsbFirst) => out.extend_from_slice(&(pixel as u16).to_le_bytes()),
                (2, X11ByteOrder::MsbFirst) => out.extend_from_slice(&(pixel as u16).to_be_bytes()),
                (_, X11ByteOrder::LsbFirst) => out.extend_from_slice(&pixel.to_le_bytes()),
                (_, X11ByteOrder::MsbFirst) => out.extend_from_slice(&pixel.to_be_bytes()),
            }
        }
    }
    out
}

fn channel_to_mask(value: u8, mask: u32) -> u32 {
    if mask == 0 {
        return 0;
    }
    let shift = mask.trailing_zeros();
    let bits = mask.count_ones();
    let max_value = (1u32 << bits) - 1;
    ((u32::from(value) * max_value + 127) / 255) << shift
}

fn pixel_to_rgb(format: PixelFormat, pixel: &[u8]) -> [u8; 3] {
    match format {
        PixelFormat::Rgb565 => {
            let raw = u16::from_le_bytes([pixel[0], pixel[1]]);
            let r = ((raw >> 11) & 0x1f) as u8;
            let g = ((raw >> 5) & 0x3f) as u8;
            let b = (raw & 0x1f) as u8;
            [
                (u16::from(r) * 255 / 31) as u8,
                (u16::from(g) * 255 / 63) as u8,
                (u16::from(b) * 255 / 31) as u8,
            ]
        }
        PixelFormat::Bgra8888 => [pixel[2], pixel[1], pixel[0]],
        PixelFormat::Rgba8888 => [pixel[0], pixel[1], pixel[2]],
        PixelFormat::Gray8 => [pixel[0], pixel[0], pixel[0]],
    }
}

fn input_events() -> &'static Mutex<VecDeque<VirtualInputEvent>> {
    INPUT_EVENTS.get_or_init(|| Mutex::new(VecDeque::new()))
}

fn touch_down() -> &'static Mutex<bool> {
    TOUCH_DOWN.get_or_init(|| Mutex::new(false))
}

fn set_touch_down(value: bool) {
    if let Ok(mut down) = touch_down().lock() {
        *down = value;
    }
}

fn is_touch_down() -> bool {
    touch_down().lock().map(|down| *down).unwrap_or(false)
}

fn push_input_event(event: VirtualInputEvent) {
    if let Ok(mut events) = input_events().lock() {
        events.push_back(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ce::framebuffer::Framebuffer;
    use crate::ce::framebuffer::VirtualFramebuffer;

    #[test]
    fn converts_rgb565_to_32_bit_x11_truecolor() {
        let mut framebuffer = VirtualFramebuffer::new(2, 1, PixelFormat::Rgb565).unwrap();
        framebuffer.pixels_mut()[0..2].copy_from_slice(&0xf800u16.to_le_bytes());
        framebuffer.pixels_mut()[2..4].copy_from_slice(&0x07e0u16.to_le_bytes());

        let bytes = framebuffer_to_ximage_data(
            framebuffer.info(),
            framebuffer.pixels(),
            X11VisualMasks {
                red: 0x00ff_0000,
                green: 0x0000_ff00,
                blue: 0x0000_00ff,
            },
            X11ByteOrder::LsbFirst,
        );

        assert_eq!(&bytes[0..4], &0x00ff_0000u32.to_le_bytes());
        assert_eq!(&bytes[4..8], &0x0000_ff00u32.to_le_bytes());
    }

    #[test]
    fn maps_common_x11_keysyms_to_win32_virtual_keys() {
        assert_eq!(keysym_to_virtual_key(0xff1b), Some(0x1b));
        assert_eq!(keysym_to_virtual_key(0xff51), Some(0x25));
        assert_eq!(keysym_to_virtual_key(u32::from(b'a')), Some(0x41));
        assert_eq!(keysym_to_virtual_key(u32::from(b'7')), Some(0x37));
    }
}
