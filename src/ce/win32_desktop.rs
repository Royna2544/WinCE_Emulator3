use std::{
    collections::{BTreeMap, VecDeque},
    ffi::c_void,
    fmt,
    os::windows::ffi::OsStrExt,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock, mpsc},
    thread,
};

use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BeginPaint, DIB_RGB_COLORS, EndPaint, GetDC, HDC,
            PAINTSTRUCT, RGBQUAD, ReleaseDC, SRCCOPY, StretchDIBits, UpdateWindow,
        },
        UI::WindowsAndMessaging::{
            AdjustWindowRectEx, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW,
            DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW, HICON, HMENU, ICON_BIG,
            ICON_SMALL, MSG, PM_REMOVE, PeekMessageW, PostMessageW, PostQuitMessage,
            PrivateExtractIconsW, RegisterClassW, SW_SHOW, SendMessageW, ShowWindow,
            TranslateMessage, WINDOW_EX_STYLE, WM_APP, WM_CLOSE, WM_DESTROY, WM_ERASEBKGND,
            WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_SETICON,
            WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
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
const WM_INTERNAL_QUIT: u32 = WM_APP + 0x3e9;

static INPUT_EVENTS: OnceLock<Mutex<VecDeque<VirtualInputEvent>>> = OnceLock::new();
static TOUCH_DOWN: OnceLock<Mutex<bool>> = OnceLock::new();
static PRESENTED_FRAMES: OnceLock<Mutex<BTreeMap<usize, PresentedFrame>>> = OnceLock::new();

#[derive(Debug, Clone)]
struct PresentedFrame {
    width: u32,
    height: u32,
    bgra: Vec<u8>,
}

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
        let drained = poll_global_input_events()?;
        if let Some(callback) = self.callback.as_mut() {
            for event in &drained {
                callback(event);
            }
        }
        Ok(drained)
    }
}

pub fn poll_global_input_events() -> Result<Vec<VirtualInputEvent>> {
    pump_messages();
    let mut events = input_events()
        .lock()
        .map_err(|_| Error::Backend("win32 input queue lock is poisoned".to_owned()))?;
    Ok(events.drain(..).collect())
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
    pub fn new(
        width: u32,
        height: u32,
        title: impl Into<String>,
        icon_path: Option<&Path>,
    ) -> Result<Self> {
        let title = title.into();
        let hwnd =
            spawn_window_thread(width, height, title.clone(), icon_path.map(Path::to_owned))?;
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

    pub fn pump_messages(&mut self) {
        pump_messages();
    }

    pub fn blit(&mut self, framebuffer: &dyn Framebuffer) -> Result<()> {
        pump_messages();
        blit_framebuffer(self.hwnd, framebuffer, &mut self.scratch_bgra)
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
        self.blit(framebuffer)?;
        self.last_presentation = Some(presentation.clone());
        self.presentation_count = self.presentation_count.saturating_add(1);
        Ok(presentation)
    }
}

impl Drop for Win32Presenter {
    fn drop(&mut self) {
        if !self.hwnd.is_invalid() {
            if let Ok(mut frames) = presented_frames().lock() {
                frames.remove(&hwnd_key(self.hwnd));
            }
            unsafe {
                let _ = PostMessageW(self.hwnd, WM_INTERNAL_QUIT, WPARAM(0), LPARAM(0));
            }
        }
    }
}

fn spawn_window_thread(
    width: u32,
    height: u32,
    title: String,
    icon_path: Option<PathBuf>,
) -> Result<HWND> {
    let (tx, rx) = mpsc::channel::<std::result::Result<isize, String>>();
    thread::Builder::new()
        .name("wince-win32-presenter".to_owned())
        .spawn(move || {
            let result =
                create_window_on_current_thread(width, height, &title, icon_path.as_deref())
                    .map(|hwnd| hwnd.0 as isize)
                    .map_err(|err| err.to_string());
            let hwnd = match result {
                Ok(hwnd) => {
                    let _ = tx.send(Ok(hwnd));
                    HWND(hwnd as *mut c_void)
                }
                Err(err) => {
                    let _ = tx.send(Err(err));
                    return;
                }
            };
            run_window_message_loop(hwnd);
        })
        .map_err(|err| Error::Backend(format!("spawn Win32 desktop thread: {err}")))?;
    let hwnd = rx
        .recv()
        .map_err(|err| Error::Backend(format!("receive Win32 desktop window handle: {err}")))?
        .map_err(|err| Error::Backend(format!("create Win32 desktop window: {err}")))?;
    Ok(HWND(hwnd as *mut c_void))
}

fn create_window_on_current_thread(
    width: u32,
    height: u32,
    title: &str,
    icon_path: Option<&Path>,
) -> Result<HWND> {
    register_window_class();
    let title_wide = wide_null(&title);
    let ex_style = WINDOW_EX_STYLE::default();
    let style = WS_OVERLAPPEDWINDOW | WS_VISIBLE;
    let mut window_rect = RECT {
        left: 0,
        top: 0,
        right: width as i32,
        bottom: height as i32,
    };
    unsafe {
        AdjustWindowRectEx(&mut window_rect, style, false, ex_style)
            .map_err(|err| Error::Backend(format!("adjust Win32 desktop window rect: {err}")))?;
    }
    let window_width = window_rect.right - window_rect.left;
    let window_height = window_rect.bottom - window_rect.top;
    let hwnd = unsafe {
        CreateWindowExW(
            ex_style,
            CLASS_NAME,
            PCWSTR(title_wide.as_ptr()),
            style,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            window_width,
            window_height,
            HWND::default(),
            HMENU::default(),
            HINSTANCE::default(),
            None,
        )
    }
    .map_err(|err| Error::Backend(format!("create Win32 desktop window: {err}")))?;
    let _icons = install_window_icons(hwnd, icon_path);
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = UpdateWindow(hwnd);
    }
    Ok(hwnd)
}

fn run_window_message_loop(hwnd: HWND) {
    let mut message = MSG::default();
    unsafe {
        while GetMessageW(&mut message, HWND::default(), 0, 0).as_bool() {
            let _ = TranslateMessage(&message);
            let _ = DispatchMessageW(&message);
        }
    }
    if let Ok(mut frames) = presented_frames().lock() {
        frames.remove(&hwnd_key(hwnd));
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

fn install_window_icons(hwnd: HWND, icon_path: Option<&Path>) -> Vec<HICON> {
    let Some(path) = icon_path else {
        return Vec::new();
    };
    let mut icons = Vec::new();
    if let Some(icon) = extract_icon(path, 32, 32) {
        unsafe {
            let _ = SendMessageW(
                hwnd,
                WM_SETICON,
                WPARAM(ICON_BIG as usize),
                LPARAM(icon.0 as isize),
            );
        }
        icons.push(icon);
    }
    if let Some(icon) = extract_icon(path, 16, 16) {
        unsafe {
            let _ = SendMessageW(
                hwnd,
                WM_SETICON,
                WPARAM(ICON_SMALL as usize),
                LPARAM(icon.0 as isize),
            );
        }
        icons.push(icon);
    }
    icons
}

fn extract_icon(path: &Path, width: i32, height: i32) -> Option<HICON> {
    let mut wide = [0u16; 260];
    for (index, unit) in path
        .as_os_str()
        .encode_wide()
        .take(wide.len().saturating_sub(1))
        .enumerate()
    {
        wide[index] = unit;
    }
    let mut icons = [HICON::default()];
    let extracted =
        unsafe { PrivateExtractIconsW(&wide, 0, width, height, Some(&mut icons), None, 0) };
    (extracted != 0 && !icons[0].is_invalid()).then_some(icons[0])
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
    if let Ok(mut frames) = presented_frames().lock() {
        frames.insert(
            hwnd_key(hwnd),
            PresentedFrame {
                width: info.width,
                height: info.height,
                bgra: scratch_bgra.clone(),
            },
        );
    }
    let hdc = unsafe { GetDC(hwnd) };
    if hdc.is_invalid() {
        return Err(Error::Backend("get Win32 desktop DC failed".to_owned()));
    }
    let written = blit_bgra_to_hwnd_hdc(hwnd, hdc, info.width, info.height, scratch_bgra);
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

fn blit_bgra_to_hwnd_hdc(hwnd: HWND, hdc: HDC, width: u32, height: u32, bgra: &[u8]) -> i32 {
    let mut client = RECT::default();
    let (dst_width, dst_height) = unsafe {
        if GetClientRect(hwnd, &mut client).is_ok() {
            (
                (client.right - client.left).max(1),
                (client.bottom - client.top).max(1),
            )
        } else {
            (width as i32, height as i32)
        }
    };
    let mut bitmap_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: -(height as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            biSizeImage: bgra.len() as u32,
            ..Default::default()
        },
        bmiColors: [RGBQUAD::default()],
    };
    unsafe {
        StretchDIBits(
            hdc,
            0,
            0,
            dst_width,
            dst_height,
            0,
            0,
            width as i32,
            height as i32,
            Some(bgra.as_ptr() as *const c_void),
            &mut bitmap_info,
            DIB_RGB_COLORS,
            SRCCOPY,
        )
    }
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

fn presented_frames() -> &'static Mutex<BTreeMap<usize, PresentedFrame>> {
    PRESENTED_FRAMES.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn hwnd_key(hwnd: HWND) -> usize {
    hwnd.0 as usize
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
        WM_INTERNAL_QUIT => {
            unsafe {
                PostQuitMessage(0);
            }
            LRESULT(0)
        }
        WM_CLOSE | WM_DESTROY => {
            std::process::exit(0);
        }
        windows::Win32::UI::WindowsAndMessaging::WM_PAINT => {
            let mut paint = PAINTSTRUCT::default();
            let hdc = unsafe { BeginPaint(hwnd, &mut paint) };
            if !hdc.is_invalid() {
                if let Ok(frames) = presented_frames().lock()
                    && let Some(frame) = frames.get(&hwnd_key(hwnd))
                {
                    let _ =
                        blit_bgra_to_hwnd_hdc(hwnd, hdc, frame.width, frame.height, &frame.bgra);
                }
            }
            unsafe {
                let _ = EndPaint(hwnd, &paint);
            }
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
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
