use std::{
    collections::VecDeque,
    ffi::c_void,
    fmt,
    sync::{Mutex, OnceLock},
};

use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{
            BI_RGB, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, GetDC, RGBQUAD, ReleaseDC,
            SetDIBitsToDevice, UpdateWindow,
        },
        UI::WindowsAndMessaging::{
            CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DestroyWindow,
            DispatchMessageW, HMENU, MSG, PM_REMOVE, PeekMessageW, RegisterClassW, SW_SHOW,
            ShowWindow, TranslateMessage, WINDOW_EX_STYLE, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN,
            WM_LBUTTONUP, WM_MOUSEMOVE, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        },
    },
    core::{PCWSTR, w},
};

use crate::{
    Error, Result,
    ce::{
        desktop::{Input, Presentation, Presenter, VirtualInputEvent},
        framebuffer::{Framebuffer, FramebufferSnapshot, PixelFormat},
    },
};

const CLASS_NAME: PCWSTR = w!("WinceEmulationV3VirtualDesktop");

static INPUT_EVENTS: OnceLock<Mutex<VecDeque<VirtualInputEvent>>> = OnceLock::new();
static TOUCH_DOWN: OnceLock<Mutex<bool>> = OnceLock::new();

pub type InputCallback = Box<dyn FnMut(&VirtualInputEvent) + Send>;

pub struct Win32Input {
    callback: Option<InputCallback>,
}

impl Win32Input {
    pub fn new() -> Self {
        Self { callback: None }
    }

    pub fn with_callback(callback: InputCallback) -> Self {
        Self {
            callback: Some(callback),
        }
    }

    pub fn set_callback(&mut self, callback: Option<InputCallback>) {
        self.callback = callback;
    }
}

impl Default for Win32Input {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Win32Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Win32Input")
            .field("callback", &self.callback.as_ref().map(|_| "<callback>"))
            .finish()
    }
}

impl Input for Win32Input {
    fn poll_events(&mut self) -> Result<Vec<VirtualInputEvent>> {
        pump_messages();
        let mut events = input_events()
            .lock()
            .map_err(|_| Error::Backend("win32 input queue lock is poisoned".to_owned()))?;
        let drained: Vec<_> = events.drain(..).collect();
        drop(events);
        if let Some(callback) = self.callback.as_mut() {
            for event in &drained {
                callback(event);
            }
        }
        Ok(drained)
    }
}

#[derive(Debug)]
pub struct Win32Presenter {
    hwnd: HWND,
    title: String,
    last_presentation: Option<Presentation>,
    presentation_count: u64,
    scratch_bgra: Vec<u8>,
}

impl Win32Presenter {
    pub fn new(width: u32, height: u32, title: impl Into<String>) -> Result<Self> {
        let title = title.into();
        register_window_class();
        let title_wide = wide_null(&title);
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                CLASS_NAME,
                PCWSTR(title_wide.as_ptr()),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                width as i32,
                height as i32,
                HWND::default(),
                HMENU::default(),
                HINSTANCE::default(),
                None,
            )
        }
        .map_err(|err| Error::Backend(format!("create Win32 desktop window: {err}")))?;
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = UpdateWindow(hwnd);
        }
        Ok(Self {
            hwnd,
            title,
            last_presentation: None,
            presentation_count: 0,
            scratch_bgra: Vec::new(),
        })
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
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
}

impl Presenter for Win32Presenter {
    fn present(&mut self, framebuffer: &dyn Framebuffer) -> Result<Presentation> {
        pump_messages();
        let snapshot = FramebufferSnapshot {
            info: framebuffer.info(),
            pixels: framebuffer.pixels().to_vec(),
        };
        let presentation = Presentation {
            framebuffer: snapshot,
            dirty_rects: framebuffer.dirty_rects().to_vec(),
        };
        blit_framebuffer(self.hwnd, framebuffer, &mut self.scratch_bgra)?;
        self.last_presentation = Some(presentation.clone());
        self.presentation_count = self.presentation_count.saturating_add(1);
        Ok(presentation)
    }
}

impl Drop for Win32Presenter {
    fn drop(&mut self) {
        if !self.hwnd.is_invalid() {
            unsafe {
                let _ = DestroyWindow(self.hwnd);
            }
        }
    }
}

fn register_window_class() {
    let class = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wnd_proc),
        lpszClassName: CLASS_NAME,
        ..Default::default()
    };
    unsafe {
        let _ = RegisterClassW(&class);
    }
}

fn pump_messages() {
    let mut message = MSG::default();
    unsafe {
        while PeekMessageW(&mut message, HWND::default(), 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&message);
            let _ = DispatchMessageW(&message);
        }
    }
}

fn blit_framebuffer(
    hwnd: HWND,
    framebuffer: &dyn Framebuffer,
    scratch_bgra: &mut Vec<u8>,
) -> Result<()> {
    let info = framebuffer.info();
    copy_to_bgra_top_down(framebuffer, scratch_bgra)?;
    let mut bitmap_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: info.width as i32,
            biHeight: -(info.height as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            biSizeImage: scratch_bgra.len() as u32,
            ..Default::default()
        },
        bmiColors: [RGBQUAD::default()],
    };
    let hdc = unsafe { GetDC(hwnd) };
    if hdc.is_invalid() {
        return Err(Error::Backend("get Win32 desktop DC failed".to_owned()));
    }
    let written = unsafe {
        SetDIBitsToDevice(
            hdc,
            0,
            0,
            info.width,
            info.height,
            0,
            0,
            0,
            info.height,
            scratch_bgra.as_ptr() as *const c_void,
            &mut bitmap_info,
            DIB_RGB_COLORS,
        )
    };
    unsafe {
        let _ = ReleaseDC(hwnd, hdc);
    }
    if written == 0 {
        return Err(Error::Backend(
            "blit Win32 desktop framebuffer failed".to_owned(),
        ));
    }
    Ok(())
}

fn copy_to_bgra_top_down(framebuffer: &dyn Framebuffer, out: &mut Vec<u8>) -> Result<()> {
    let info = framebuffer.info();
    let required_len = (info.width as usize)
        .checked_mul(info.height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| Error::InvalidArgument("desktop blit buffer overflow".to_owned()))?;
    out.resize(required_len, 0);
    for y in 0..info.height as usize {
        let src_row = &framebuffer.pixels()[y * info.stride..];
        for x in 0..info.width as usize {
            let src_offset = x * info.format.bytes_per_pixel();
            let dst_offset = (y * info.width as usize + x) * 4;
            let [r, g, b] = pixel_to_rgb(info.format, &src_row[src_offset..]);
            out[dst_offset] = b;
            out[dst_offset + 1] = g;
            out[dst_offset + 2] = r;
            out[dst_offset + 3] = 0xff;
        }
    }
    Ok(())
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

fn push_input_event(event: VirtualInputEvent) {
    if let Ok(mut events) = input_events().lock() {
        events.push_back(event);
    }
}

fn wide_null(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn mouse_x(lparam: LPARAM) -> i32 {
    (lparam.0 as u16 as i16) as i32
}

fn mouse_y(lparam: LPARAM) -> i32 {
    ((lparam.0 >> 16) as u16 as i16) as i32
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_LBUTTONDOWN => {
            if let Ok(mut down) = touch_down().lock() {
                *down = true;
            }
            push_input_event(VirtualInputEvent::TouchDown {
                x: mouse_x(lparam),
                y: mouse_y(lparam),
            });
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            let down = touch_down().lock().map(|down| *down).unwrap_or(false);
            if down || (wparam.0 & 1) != 0 {
                push_input_event(VirtualInputEvent::TouchMove {
                    x: mouse_x(lparam),
                    y: mouse_y(lparam),
                });
            }
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            if let Ok(mut down) = touch_down().lock() {
                *down = false;
            }
            push_input_event(VirtualInputEvent::TouchUp {
                x: mouse_x(lparam),
                y: mouse_y(lparam),
            });
            LRESULT(0)
        }
        WM_KEYDOWN => {
            push_input_event(VirtualInputEvent::Key {
                virtual_key: wparam.0 as u32,
                pressed: true,
            });
            LRESULT(0)
        }
        WM_KEYUP => {
            push_input_event(VirtualInputEvent::Key {
                virtual_key: wparam.0 as u32,
                pressed: false,
            });
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
