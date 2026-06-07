use std::collections::{BTreeMap, BTreeSet, VecDeque};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PeekFlags: u32 {
        const REMOVE = 0x0001;
        const NO_REMOVE = 0x0000;
    }
}

pub const WM_NULL: u32 = 0x0000;
pub const WM_CREATE: u32 = 0x0001;
pub const WM_DESTROY: u32 = 0x0002;
pub const WM_MOVE: u32 = 0x0003;
pub const WM_SIZE: u32 = 0x0005;
pub const WM_PAINT: u32 = 0x000f;
pub const WM_CLOSE: u32 = 0x0010;
pub const WM_QUIT: u32 = 0x0012;
pub const WM_ERASEBKGND: u32 = 0x0014;
pub const WM_SHOWWINDOW: u32 = 0x0018;
pub const WM_CANCELMODE: u32 = 0x001f;
pub const WM_WINDOWPOSCHANGED: u32 = 0x0047;
pub const WM_ACTIVATE: u32 = 0x0006;
pub const WM_SETFOCUS: u32 = 0x0007;
pub const WM_KILLFOCUS: u32 = 0x0008;
pub const WM_ENABLE: u32 = 0x000a;
pub const WM_NCCREATE: u32 = 0x0081;
pub const WM_GETDLGCODE: u32 = 0x0087;
pub const WM_SETTEXT: u32 = 0x000c;
pub const WM_GETTEXT: u32 = 0x000d;
pub const WM_GETTEXTLENGTH: u32 = 0x000e;
pub const WM_KEYDOWN: u32 = 0x0100;
pub const WM_KEYUP: u32 = 0x0101;
pub const WM_CHAR: u32 = 0x0102;
pub const WM_COMMAND: u32 = 0x0111;
pub const WM_TIMER: u32 = 0x0113;
pub const WM_MOUSEMOVE: u32 = 0x0200;
pub const WM_LBUTTONDOWN: u32 = 0x0201;
pub const WM_LBUTTONUP: u32 = 0x0202;
pub const WM_USER: u32 = 0x0400;
pub const WM_APP: u32 = 0x8000;
pub const WM_NCDESTROY: u32 = WM_APP - 1;
pub const DM_GETDEFID: u32 = WM_USER;
pub const DM_SETDEFID: u32 = WM_USER + 1;
pub const DC_HASDEFID: u32 = 0x534b;
pub const BS_PUSHBUTTON: u32 = 0x0000;
pub const BS_DEFPUSHBUTTON: u32 = 0x0001;
pub const BS_CHECKBOX: u32 = 0x0002;
pub const BS_RADIOBUTTON: u32 = 0x0004;
pub const BS_AUTORADIOBUTTON: u32 = 0x0009;
pub const BS_TYPEMASK: u32 = 0x000f;
pub const DLGC_WANTTAB: u32 = 0x0002;
pub const DLGC_WANTALLKEYS: u32 = 0x0004;
pub const DLGC_HASSETSEL: u32 = 0x0008;
pub const DLGC_DEFPUSHBUTTON: u32 = 0x0010;
pub const DLGC_UNDEFPUSHBUTTON: u32 = 0x0020;
pub const DLGC_RADIOBUTTON: u32 = 0x0040;
pub const DLGC_WANTCHARS: u32 = 0x0080;
pub const DLGC_STATIC: u32 = 0x0100;
pub const DLGC_BUTTON: u32 = 0x2000;
pub const VK_LBUTTON: u32 = 0x01;
pub const VK_SHIFT: u32 = 0x10;
pub const VK_CONTROL: u32 = 0x11;
pub const VK_MENU: u32 = 0x12;
pub const VK_CAPITAL: u32 = 0x14;
pub const VK_LSHIFT: u32 = 0xa0;
pub const VK_RSHIFT: u32 = 0xa1;
pub const VK_LCONTROL: u32 = 0xa2;
pub const VK_RCONTROL: u32 = 0xa3;
pub const VK_LMENU: u32 = 0xa4;
pub const VK_RMENU: u32 = 0xa5;
pub const KEY_STATE_TOGGLED_FLAG: u32 = 0x0001;
pub const KEY_STATE_GET_ASYNC_DOWN_FLAG: u32 = 0x0002;
pub const KEY_STATE_PREV_DOWN_FLAG: u32 = 0x0040;
pub const KEY_STATE_DOWN_FLAG: u32 = 0x0080;
pub const KEY_SHIFT_ANY_CTRL_FLAG: u32 = 0x4000_0000;
pub const KEY_SHIFT_ANY_SHIFT_FLAG: u32 = 0x2000_0000;
pub const KEY_SHIFT_ANY_ALT_FLAG: u32 = 0x1000_0000;
pub const KEY_SHIFT_CAPITAL_FLAG: u32 = 0x0800_0000;
pub const KEY_SHIFT_LEFT_CTRL_FLAG: u32 = 0x0400_0000;
pub const KEY_SHIFT_LEFT_SHIFT_FLAG: u32 = 0x0200_0000;
pub const KEY_SHIFT_LEFT_ALT_FLAG: u32 = 0x0100_0000;
pub const KEY_SHIFT_RIGHT_CTRL_FLAG: u32 = 0x0040_0000;
pub const KEY_SHIFT_RIGHT_SHIFT_FLAG: u32 = 0x0020_0000;
pub const KEY_SHIFT_RIGHT_ALT_FLAG: u32 = 0x0010_0000;
pub const MSGSRC_UNKNOWN: u32 = 0;
pub const MSGSRC_SOFTWARE_POST: u32 = 1;
pub const MSGSRC_HARDWARE_KEYBOARD: u32 = 2;
pub const MSGSRC_SOFTWARE_SEND: u32 = 4;
pub const SMF_NULL: u32 = 0x0000_0000;
pub const SMF_SENDER_NO_WAIT: u32 = 0x0000_0001;
pub const SMF_SENDER_NO_WAIT_IF_DIFFERENT_THREAD: u32 = 0x0000_0002;
pub const SMF_RESULT_READY: u32 = 0x8000_0000;
pub const SMF_SENDER_TERMINATED: u32 = 0x4000_0000;
pub const SMF_RECEIVER_TERMINATED: u32 = 0x2000_0000;
pub const SMF_TIMEOUT: u32 = 0x1000_0000;
pub const SMF_NOTIFY_MESSAGE: u32 = 0x0800_0000;
pub const QS_KEY: u32 = 0x0001;
pub const QS_MOUSEMOVE: u32 = 0x0002;
pub const QS_MOUSEBUTTON: u32 = 0x0004;
pub const QS_POSTMESSAGE: u32 = 0x0008;
pub const QS_TIMER: u32 = 0x0010;
pub const QS_PAINT: u32 = 0x0020;
pub const QS_SENDMESSAGE: u32 = 0x0040;

pub const HWND_BROADCAST: u32 = 0x0000_ffff;
pub const DESKTOP_HWND: u32 = 0x0001_0000;
pub const WNDCLASSW_SIZE: usize = 40;
pub const DEFAULT_WNDPROC: u32 = 0xffff_fffc;

pub const GWL_WNDPROC: i32 = -4;
pub const GWL_ID: i32 = -12;
pub const GWL_STYLE: i32 = -16;
pub const GWL_EXSTYLE: i32 = -20;
pub const GWL_USERDATA: i32 = -21;
pub const DWL_MSGRESULT: i32 = 0;
pub const DWL_DLGPROC: i32 = 4;
pub const DWL_USER: i32 = 8;

pub const GW_HWNDFIRST: u32 = 0;
pub const GW_HWNDLAST: u32 = 1;
pub const GW_HWNDNEXT: u32 = 2;
pub const GW_HWNDPREV: u32 = 3;
pub const GW_OWNER: u32 = 4;
pub const GW_CHILD: u32 = 5;

pub const CW_USEDEFAULT: i32 = i32::MIN;

pub const SWP_NOSIZE: u32 = 0x0001;
pub const SWP_NOMOVE: u32 = 0x0002;
pub const SWP_NOZORDER: u32 = 0x0004;
pub const SWP_NOREDRAW: u32 = 0x0008;
pub const SWP_NOACTIVATE: u32 = 0x0010;
pub const SWP_SHOWWINDOW: u32 = 0x0040;
pub const SWP_HIDEWINDOW: u32 = 0x0080;
pub const HWND_TOP: u32 = 0;
pub const HWND_BOTTOM: u32 = 1;
pub const HWND_TOPMOST: u32 = u32::MAX;
pub const HWND_NOTOPMOST: u32 = u32::MAX - 1;
pub const WS_POPUP: u32 = 0x8000_0000;
pub const WS_CHILD: u32 = 0x4000_0000;
pub const WS_VISIBLE: u32 = 0x1000_0000;
pub const WS_DISABLED: u32 = 0x0800_0000;
pub const WS_CLIPSIBLINGS: u32 = 0x0400_0000;
pub const WS_CLIPCHILDREN: u32 = 0x0200_0000;
pub const WS_GROUP: u32 = 0x0002_0000;
pub const WS_TABSTOP: u32 = 0x0001_0000;
pub const WA_INACTIVE: u32 = 0;
pub const WA_ACTIVE: u32 = 1;

pub const SM_CXSCREEN: u32 = 0;
pub const SM_CYSCREEN: u32 = 1;
pub const SM_CXVSCROLL: u32 = 2;
pub const SM_CYHSCROLL: u32 = 3;
pub const SM_CYCAPTION: u32 = 4;
pub const SM_CXBORDER: u32 = 5;
pub const SM_CYBORDER: u32 = 6;
pub const SM_CXDLGFRAME: u32 = 7;
pub const SM_CYDLGFRAME: u32 = 8;
pub const SM_CXICON: u32 = 11;
pub const SM_CYICON: u32 = 12;
pub const SM_CXCURSOR: u32 = 13;
pub const SM_CYCURSOR: u32 = 14;
pub const SM_CYMENU: u32 = 15;
pub const SM_CXFULLSCREEN: u32 = 16;
pub const SM_CYFULLSCREEN: u32 = 17;
pub const SM_MOUSEPRESENT: u32 = 19;
pub const SM_CYVSCROLL: u32 = 20;
pub const SM_CXHSCROLL: u32 = 21;
pub const SM_DEBUG: u32 = 22;
pub const SM_CXDOUBLECLK: u32 = 36;
pub const SM_CYDOUBLECLK: u32 = 37;
pub const SM_CXICONSPACING: u32 = 38;
pub const SM_CYICONSPACING: u32 = 39;
pub const SM_CXEDGE: u32 = 45;
pub const SM_CYEDGE: u32 = 46;
pub const SM_CXSMICON: u32 = 49;
pub const SM_CYSMICON: u32 = 50;
pub const SM_XVIRTUALSCREEN: u32 = 76;
pub const SM_YVIRTUALSCREEN: u32 = 77;
pub const SM_CXVIRTUALSCREEN: u32 = 78;
pub const SM_CYVIRTUALSCREEN: u32 = 79;
pub const SM_CMONITORS: u32 = 80;
pub const SM_SAMEDISPLAYFORMAT: u32 = 81;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub hwnd: u32,
    pub msg: u32,
    pub wparam: u32,
    pub lparam: u32,
    pub time_ms: u32,
    pub source: u32,
    pub mouse_pos_at_post: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowPos {
    pub hwnd: u32,
    pub insert_after: u32,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub flags: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessagePointerPayload {
    WindowPos(WindowPos),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentMessage {
    pub id: u64,
    pub sender_thread_id: Option<u32>,
    pub receiver_thread_id: u32,
    pub message: Message,
    pub flags: u32,
    pub timeout_ms: Option<u32>,
    pub result: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct QuitState {
    exit_code: u32,
    time_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn from_origin_size(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            left: x,
            top: y,
            right: x.saturating_add(width),
            bottom: y.saturating_add(height),
        }
    }

    pub fn width(self) -> i32 {
        self.right.saturating_sub(self.left)
    }

    pub fn height(self) -> i32 {
        self.bottom.saturating_sub(self.top)
    }

    pub fn offset(self, dx: i32, dy: i32) -> Self {
        Self {
            left: self.left.saturating_add(dx),
            top: self.top.saturating_add(dy),
            right: self.right.saturating_add(dx),
            bottom: self.bottom.saturating_add(dy),
        }
    }

    pub fn zero_origin(self) -> Self {
        Self::from_origin_size(0, 0, self.width(), self.height())
    }

    pub fn contains_point(self, point: Point) -> bool {
        point.x >= self.left && point.x < self.right && point.y >= self.top && point.y < self.bottom
    }

    pub fn normalized(self) -> Self {
        Self {
            left: self.left.min(self.right),
            top: self.top.min(self.bottom),
            right: self.left.max(self.right),
            bottom: self.top.max(self.bottom),
        }
    }

    pub fn is_empty(self) -> bool {
        self.width() <= 0 || self.height() <= 0
    }

    pub fn union(self, other: Self) -> Self {
        let lhs = self.normalized();
        let rhs = other.normalized();
        if lhs.is_empty() {
            return rhs;
        }
        if rhs.is_empty() {
            return lhs;
        }
        Self {
            left: lhs.left.min(rhs.left),
            top: lhs.top.min(rhs.top),
            right: lhs.right.max(rhs.right),
            bottom: lhs.bottom.max(rhs.bottom),
        }
    }

    pub fn intersect(self, other: Self) -> Option<Self> {
        let lhs = self.normalized();
        let rhs = other.normalized();
        let rect = Self {
            left: lhs.left.max(rhs.left),
            top: lhs.top.max(rhs.top),
            right: lhs.right.min(rhs.right),
            bottom: lhs.bottom.min(rhs.bottom),
        };
        (!rect.is_empty()).then_some(rect)
    }

    pub fn subtract_bounding(self, other: Self) -> Option<Self> {
        let lhs = self.normalized();
        let Some(intersection) = lhs.intersect(other) else {
            return Some(lhs);
        };
        if intersection == lhs {
            return None;
        }

        let candidates = [
            Self {
                left: lhs.left,
                top: lhs.top,
                right: lhs.right,
                bottom: intersection.top,
            },
            Self {
                left: lhs.left,
                top: intersection.bottom,
                right: lhs.right,
                bottom: lhs.bottom,
            },
            Self {
                left: lhs.left,
                top: intersection.top,
                right: intersection.left,
                bottom: intersection.bottom,
            },
            Self {
                left: intersection.right,
                top: intersection.top,
                right: lhs.right,
                bottom: intersection.bottom,
            },
        ];

        candidates
            .into_iter()
            .filter(|rect| !rect.is_empty())
            .reduce(|acc, rect| acc.union(rect))
    }

    pub fn subtract(self, other: Self) -> Vec<Self> {
        let lhs = self.normalized();
        let Some(intersection) = lhs.intersect(other) else {
            return vec![lhs];
        };
        let candidates = [
            Self {
                left: lhs.left,
                top: lhs.top,
                right: lhs.right,
                bottom: intersection.top,
            },
            Self {
                left: lhs.left,
                top: intersection.bottom,
                right: lhs.right,
                bottom: lhs.bottom,
            },
            Self {
                left: lhs.left,
                top: intersection.top,
                right: intersection.left,
                bottom: intersection.bottom,
            },
            Self {
                left: intersection.right,
                top: intersection.top,
                right: lhs.right,
                bottom: intersection.bottom,
            },
        ];

        candidates
            .into_iter()
            .filter(|rect| !rect.is_empty())
            .collect()
    }
}

pub(crate) fn canonicalize_region_rects(rects: Vec<Rect>) -> Vec<Rect> {
    let rects: Vec<Rect> = rects
        .into_iter()
        .map(Rect::normalized)
        .filter(|rect| !rect.is_empty())
        .collect();
    if rects.len() <= 1 {
        return rects;
    }

    let mut y_edges: Vec<i32> = rects
        .iter()
        .flat_map(|rect| [rect.top, rect.bottom])
        .collect();
    y_edges.sort_unstable();
    y_edges.dedup();

    let mut canonical: Vec<Rect> = Vec::new();
    for band in y_edges.windows(2) {
        let top = band[0];
        let bottom = band[1];
        if top >= bottom {
            continue;
        }

        let mut spans: Vec<(i32, i32)> = rects
            .iter()
            .filter(|rect| rect.top <= top && rect.bottom >= bottom)
            .map(|rect| (rect.left, rect.right))
            .collect();
        if spans.is_empty() {
            continue;
        }
        spans.sort_unstable();

        let mut merged_spans: Vec<(i32, i32)> = Vec::new();
        for (left, right) in spans {
            if left >= right {
                continue;
            }
            if let Some((_, last_right)) = merged_spans.last_mut()
                && left <= *last_right
            {
                *last_right = (*last_right).max(right);
                continue;
            }
            merged_spans.push((left, right));
        }

        for (left, right) in merged_spans {
            if let Some(previous) = canonical.last_mut()
                && previous.left == left
                && previous.right == right
                && previous.bottom == top
            {
                previous.bottom = bottom;
                continue;
            }
            canonical.push(Rect {
                left,
                top,
                right,
                bottom,
            });
        }
    }

    canonical
}

fn bounding_region_rect(rects: &[Rect]) -> Rect {
    rects
        .iter()
        .copied()
        .reduce(Rect::union)
        .unwrap_or_default()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone)]
pub struct Window {
    pub hwnd: u32,
    pub thread_id: u32,
    pub process_id: u32,
    pub class_name: String,
    pub title: String,
    pub visible: bool,
    pub enabled: bool,
    pub parent: Option<u32>,
    pub owner: Option<u32>,
    pub id: u32,
    pub menu: Option<u32>,
    pub style: u32,
    pub ex_style: u32,
    pub wndproc: u32,
    pub user_data: u32,
    pub extra_longs: Vec<u32>,
    pub rect: Rect,
    pub client_rect: Rect,
    pub update_pending: bool,
    pub erase_pending: bool,
    pub update_rect: Rect,
    pub pending_move: bool,
    pub pending_size: bool,
    pub being_destroyed: bool,
    pub destroyed: bool,
    pub destroy_message_sent: bool,
    pub nc_destroy_message_sent: bool,
    pub destroy_message_order: Option<u64>,
    pub nc_destroy_message_order: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowClass {
    pub atom: u16,
    pub name: String,
    pub bytes: [u8; WNDCLASSW_SIZE],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GestureRegistration {
    pub id: u32,
    pub handle: u32,
    pub arg1: u32,
    pub arg2: u32,
    pub arg3: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaintUpdate {
    pub rect: Rect,
    pub erase: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WindowRegion {
    pub rect: Rect,
    pub rects: Vec<Rect>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GweStats {
    pub send_transaction_count: u64,
    pub send_transaction_completed_count: u64,
    pub send_transaction_timeout_count: u64,
    pub send_transaction_receiver_terminated_count: u64,
    pub max_sent_queue_depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardState {
    open_window: Option<u32>,
    owner: Option<u32>,
    data_by_format: BTreeMap<u32, u32>,
    registered_formats_by_name: BTreeMap<String, u32>,
    registered_format_names: BTreeMap<u32, String>,
    next_registered_format: u32,
}

impl Default for ClipboardState {
    fn default() -> Self {
        Self {
            open_window: None,
            owner: None,
            data_by_format: BTreeMap::new(),
            registered_formats_by_name: BTreeMap::new(),
            registered_format_names: BTreeMap::new(),
            next_registered_format: 0xc000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaretState {
    pub hwnd: u32,
    pub bitmap: u32,
    pub width: i32,
    pub height: i32,
    pub position: Point,
    pub show_count: i32,
}

#[derive(Debug, Clone)]
pub struct Gwe {
    next_hwnd: u32,
    next_class_atom: u16,
    classes_by_name: BTreeMap<String, WindowClass>,
    class_names_by_atom: BTreeMap<u16, String>,
    windows: BTreeMap<u32, Window>,
    z_order: Vec<u32>,
    queues: BTreeMap<u32, VecDeque<Message>>,
    sent_queues: BTreeMap<u32, VecDeque<u64>>,
    sent_messages: BTreeMap<u64, SentMessage>,
    active_sent_stack_by_thread: BTreeMap<u32, Vec<u64>>,
    next_sent_message_id: u64,
    active_window: Option<u32>,
    focus: Option<u32>,
    keyboard_target_by_thread: BTreeMap<u32, u32>,
    capture: Option<u32>,
    cursor: Option<u32>,
    cursor_pos: Point,
    next_registered_message: u32,
    registered_messages: BTreeMap<String, u32>,
    gesture_registrations: BTreeMap<u32, GestureRegistration>,
    dialog_results: BTreeMap<u32, u32>,
    dialog_checks: BTreeMap<(u32, u32), u32>,
    window_regions: BTreeMap<u32, WindowRegion>,
    send_depth_by_thread: BTreeMap<u32, u32>,
    last_message_source_by_thread: BTreeMap<u32, u32>,
    last_message_pos_by_thread: BTreeMap<u32, u32>,
    ready_timestamp_by_thread: BTreeMap<u32, u32>,
    changed_queue_status_by_thread: BTreeMap<u32, u32>,
    quit_by_thread: BTreeMap<u32, QuitState>,
    key_state: [u16; 256],
    async_key_down: [bool; 256],
    message_pointer_payloads: BTreeMap<u32, MessagePointerPayload>,
    replied_send_depth_by_thread: BTreeMap<u32, BTreeSet<u32>>,
    clipboard: ClipboardState,
    caret: Option<CaretState>,
    caret_blink_time_ms: u32,
    caret_system_enabled: bool,
    next_lifecycle_message_order: u64,
    stats: GweStats,
}

impl Default for Gwe {
    fn default() -> Self {
        let mut windows = BTreeMap::new();
        windows.insert(
            DESKTOP_HWND,
            Window {
                hwnd: DESKTOP_HWND,
                thread_id: 0,
                process_id: 0,
                class_name: "Desktop".to_owned(),
                title: String::new(),
                visible: true,
                enabled: true,
                parent: None,
                owner: None,
                id: 0,
                menu: None,
                style: 0,
                ex_style: 0,
                wndproc: 0,
                user_data: 0,
                extra_longs: Vec::new(),
                rect: Rect::from_origin_size(0, 0, 800, 480),
                client_rect: Rect::from_origin_size(0, 0, 800, 480),
                update_pending: false,
                erase_pending: false,
                update_rect: Rect::default(),
                pending_move: false,
                pending_size: false,
                being_destroyed: false,
                destroyed: false,
                destroy_message_sent: false,
                nc_destroy_message_sent: false,
                destroy_message_order: None,
                nc_destroy_message_order: None,
            },
        );
        Self {
            next_hwnd: 0x0002_0000,
            next_class_atom: 0xc000,
            classes_by_name: BTreeMap::new(),
            class_names_by_atom: BTreeMap::new(),
            windows,
            z_order: vec![DESKTOP_HWND],
            queues: BTreeMap::new(),
            sent_queues: BTreeMap::new(),
            sent_messages: BTreeMap::new(),
            active_sent_stack_by_thread: BTreeMap::new(),
            next_sent_message_id: 1,
            active_window: None,
            focus: None,
            keyboard_target_by_thread: BTreeMap::new(),
            capture: None,
            cursor: None,
            cursor_pos: Point::default(),
            next_registered_message: 0xc000,
            registered_messages: BTreeMap::new(),
            gesture_registrations: BTreeMap::new(),
            dialog_results: BTreeMap::new(),
            dialog_checks: BTreeMap::new(),
            window_regions: BTreeMap::new(),
            send_depth_by_thread: BTreeMap::new(),
            last_message_source_by_thread: BTreeMap::new(),
            last_message_pos_by_thread: BTreeMap::new(),
            ready_timestamp_by_thread: BTreeMap::new(),
            changed_queue_status_by_thread: BTreeMap::new(),
            quit_by_thread: BTreeMap::new(),
            key_state: [0; 256],
            async_key_down: [false; 256],
            message_pointer_payloads: BTreeMap::new(),
            replied_send_depth_by_thread: BTreeMap::new(),
            clipboard: ClipboardState::default(),
            caret: None,
            caret_blink_time_ms: 500,
            caret_system_enabled: true,
            next_lifecycle_message_order: 1,
            stats: GweStats::default(),
        }
    }
}

impl Gwe {
    pub fn create_window(&mut self, thread_id: u32, class_name: &str, title: &str) -> u32 {
        self.create_window_ex(thread_id, class_name, title, None, 0, 0, 0)
    }

    pub fn create_window_ex(
        &mut self,
        thread_id: u32,
        class_name: &str,
        title: &str,
        parent: Option<u32>,
        id: u32,
        style: u32,
        ex_style: u32,
    ) -> u32 {
        self.create_window_ex_with_rect(
            thread_id,
            class_name,
            title,
            parent,
            id,
            style,
            ex_style,
            Rect::default(),
        )
    }

    pub fn create_window_ex_with_rect(
        &mut self,
        thread_id: u32,
        class_name: &str,
        title: &str,
        parent: Option<u32>,
        id: u32,
        style: u32,
        ex_style: u32,
        rect: Rect,
    ) -> u32 {
        self.create_window_ex_with_process_and_rect(
            thread_id, 0, class_name, title, parent, id, style, ex_style, rect,
        )
    }

    pub fn create_window_ex_with_process_and_rect(
        &mut self,
        thread_id: u32,
        process_id: u32,
        class_name: &str,
        title: &str,
        parent: Option<u32>,
        id: u32,
        style: u32,
        ex_style: u32,
        rect: Rect,
    ) -> u32 {
        self.create_window_ex_with_process_parent_owner_and_rect(
            thread_id, process_id, class_name, title, parent, None, id, style, ex_style, rect,
        )
    }

    pub fn create_window_ex_with_process_parent_owner_and_rect(
        &mut self,
        thread_id: u32,
        process_id: u32,
        class_name: &str,
        title: &str,
        parent: Option<u32>,
        owner: Option<u32>,
        id: u32,
        style: u32,
        ex_style: u32,
        rect: Rect,
    ) -> u32 {
        let rect = self.normalize_create_rect(parent, style, rect);
        let class_name = self.resolve_class_name(class_name);
        let wndproc = self
            .class_info(&class_name)
            .map(|window_class| {
                u32::from_le_bytes([
                    window_class.bytes[4],
                    window_class.bytes[5],
                    window_class.bytes[6],
                    window_class.bytes[7],
                ])
            })
            .filter(|wndproc| *wndproc != 0)
            .unwrap_or(DEFAULT_WNDPROC);
        let is_dialog = class_name.eq_ignore_ascii_case("dialog");
        let extra_longs = self
            .class_info(&class_name)
            .map(window_extra_long_count)
            .unwrap_or(0)
            .max(if is_dialog { 3 } else { 0 });
        let hwnd = self.next_hwnd;
        self.next_hwnd += 4;
        let visible = style & WS_VISIBLE != 0;
        let enabled = style & WS_DISABLED == 0;
        let update_rect = rect.zero_origin();
        self.windows.insert(
            hwnd,
            Window {
                hwnd,
                thread_id,
                process_id,
                class_name,
                title: title.to_owned(),
                visible,
                enabled,
                parent,
                owner,
                id,
                menu: None,
                style,
                ex_style,
                wndproc,
                user_data: 0,
                extra_longs: vec![0; extra_longs],
                rect,
                client_rect: rect,
                update_pending: visible,
                erase_pending: visible,
                update_rect,
                pending_move: false,
                pending_size: false,
                being_destroyed: false,
                destroyed: false,
                destroy_message_sent: false,
                nc_destroy_message_sent: false,
                destroy_message_order: None,
                nc_destroy_message_order: None,
            },
        );
        if parent.is_none() {
            self.apply_z_order(hwnd, parent, HWND_TOP);
        } else {
            self.z_order.push(hwnd);
        }
        if self.is_window_visible(hwnd) {
            self.mark_queue_status_changed(thread_id, QS_PAINT);
        }
        hwnd
    }

    pub fn register_class(&mut self, name_or_atom: &str, bytes: [u8; WNDCLASSW_SIZE]) -> u16 {
        let name = normalize_class_name(name_or_atom);
        if let Some(window_class) = self.classes_by_name.get_mut(&name) {
            window_class.bytes = bytes;
            return window_class.atom;
        }

        let atom = self.next_class_atom;
        self.next_class_atom = self.next_class_atom.wrapping_add(1).max(0xc000);
        self.classes_by_name.insert(
            name.clone(),
            WindowClass {
                atom,
                name: name.clone(),
                bytes,
            },
        );
        self.class_names_by_atom.insert(atom, name);
        atom
    }

    pub fn register_window_message(&mut self, name: &str) -> Option<u32> {
        let name = normalize_class_name(name);
        if name.is_empty() {
            return None;
        }
        if let Some(message) = self.registered_messages.get(&name) {
            return Some(*message);
        }
        let message = self.next_registered_message;
        if message > 0xffff {
            return None;
        }
        self.next_registered_message = self.next_registered_message.saturating_add(1);
        self.registered_messages.insert(name, message);
        Some(message)
    }

    pub fn open_clipboard(&mut self, hwnd: u32) -> bool {
        if hwnd != 0 && !self.is_window(hwnd) {
            return false;
        }
        if self.clipboard.open_window.is_some() {
            return false;
        }
        self.clipboard.open_window = Some(hwnd);
        true
    }

    pub fn close_clipboard(&mut self) -> bool {
        if self.clipboard.open_window.is_none() {
            return false;
        }
        self.clipboard.open_window = None;
        true
    }

    pub fn get_open_clipboard_window(&self) -> u32 {
        self.clipboard.open_window.unwrap_or(0)
    }

    pub fn clipboard_is_open(&self) -> bool {
        self.clipboard.open_window.is_some()
    }

    pub fn get_clipboard_owner(&self) -> u32 {
        self.clipboard.owner.unwrap_or(0)
    }

    pub fn empty_clipboard(&mut self) -> bool {
        let Some(open_window) = self.clipboard.open_window else {
            return false;
        };
        self.clipboard.data_by_format.clear();
        self.clipboard.owner = Some(open_window);
        true
    }

    pub fn set_clipboard_data(&mut self, format: u32, handle: u32) -> Option<u32> {
        if self.clipboard.open_window.is_none() || format == 0 {
            return None;
        }
        self.clipboard.data_by_format.insert(format, handle);
        Some(handle)
    }

    pub fn get_clipboard_data(&self, format: u32) -> Option<u32> {
        if self.clipboard.open_window.is_none() {
            return None;
        }
        self.clipboard.data_by_format.get(&format).copied()
    }

    pub fn is_clipboard_format_available(&self, format: u32) -> bool {
        self.clipboard.data_by_format.contains_key(&format)
    }

    pub fn count_clipboard_formats(&self) -> u32 {
        self.clipboard
            .data_by_format
            .len()
            .try_into()
            .unwrap_or(u32::MAX)
    }

    pub fn enum_clipboard_formats(&self, previous: u32) -> u32 {
        self.clipboard
            .data_by_format
            .keys()
            .copied()
            .find(|format| *format > previous)
            .unwrap_or(0)
    }

    pub fn get_priority_clipboard_format(&self, formats: &[u32]) -> i32 {
        if self.clipboard.data_by_format.is_empty() {
            return 0;
        }
        formats
            .iter()
            .copied()
            .find(|format| self.is_clipboard_format_available(*format))
            .map(|format| format as i32)
            .unwrap_or(-1)
    }

    pub fn register_clipboard_format(&mut self, name: &str) -> Option<u32> {
        let normalized = normalize_class_name(name);
        if normalized.is_empty() {
            return None;
        }
        if let Some(format) = self.clipboard.registered_formats_by_name.get(&normalized) {
            return Some(*format);
        }
        let format = self.clipboard.next_registered_format;
        if format > 0xffff {
            return None;
        }
        self.clipboard.next_registered_format =
            self.clipboard.next_registered_format.saturating_add(1);
        self.clipboard
            .registered_formats_by_name
            .insert(normalized, format);
        self.clipboard
            .registered_format_names
            .insert(format, name.to_owned());
        Some(format)
    }

    pub fn clipboard_format_name(&self, format: u32) -> Option<&str> {
        self.clipboard
            .registered_format_names
            .get(&format)
            .map(String::as_str)
    }

    pub fn create_caret(&mut self, hwnd: u32, bitmap: u32, width: i32, height: i32) -> bool {
        if !self.is_window(hwnd) || width < 0 || height < 0 {
            return false;
        }
        self.caret = Some(CaretState {
            hwnd,
            bitmap,
            width,
            height,
            position: Point { x: 0, y: 0 },
            show_count: -1,
        });
        true
    }

    pub fn destroy_caret(&mut self) -> bool {
        if self.caret.is_none() {
            return false;
        }
        self.caret = None;
        true
    }

    pub fn hide_caret(&mut self, hwnd: u32) -> bool {
        let Some(caret) = self.caret.as_mut() else {
            return false;
        };
        if hwnd != 0 && caret.hwnd != hwnd {
            return false;
        }
        caret.show_count = caret.show_count.saturating_sub(1);
        true
    }

    pub fn show_caret(&mut self, hwnd: u32) -> bool {
        let Some(caret) = self.caret.as_mut() else {
            return false;
        };
        if hwnd != 0 && caret.hwnd != hwnd {
            return false;
        }
        caret.show_count = caret.show_count.saturating_add(1);
        true
    }

    pub fn set_caret_pos(&mut self, x: i32, y: i32) -> bool {
        let Some(caret) = self.caret.as_mut() else {
            return false;
        };
        caret.position = Point { x, y };
        true
    }

    pub fn get_caret_pos(&self) -> Option<Point> {
        self.caret.map(|caret| caret.position)
    }

    pub fn caret(&self) -> Option<CaretState> {
        self.caret
    }

    pub fn set_caret_blink_time(&mut self, milliseconds: u32) -> bool {
        if milliseconds == 0 {
            return false;
        }
        self.caret_blink_time_ms = milliseconds;
        true
    }

    pub fn get_caret_blink_time(&self) -> u32 {
        self.caret_blink_time_ms
    }

    pub fn disable_caret_system_wide(&mut self) {
        self.caret_system_enabled = false;
    }

    pub fn enable_caret_system_wide(&mut self) {
        self.caret_system_enabled = true;
    }

    pub fn caret_system_enabled(&self) -> bool {
        self.caret_system_enabled
    }

    pub fn register_gesture(
        &mut self,
        id: u32,
        handle: u32,
        arg1: u32,
        arg2: u32,
        arg3: u32,
    ) -> bool {
        if id == 0 || handle == 0 {
            return false;
        }
        self.gesture_registrations.insert(
            id,
            GestureRegistration {
                id,
                handle,
                arg1,
                arg2,
                arg3,
            },
        );
        true
    }

    pub fn gesture_registration(&self, id: u32) -> Option<GestureRegistration> {
        self.gesture_registrations.get(&id).copied()
    }

    pub fn class_info(&self, name_or_atom: &str) -> Option<&WindowClass> {
        let name = if let Some(atom) = parse_atom_class_name(name_or_atom) {
            self.class_names_by_atom.get(&atom)?.clone()
        } else {
            normalize_class_name(name_or_atom)
        };
        self.classes_by_name.get(&name)
    }

    pub fn resolve_class_name(&self, name_or_atom: &str) -> String {
        if let Some(window_class) = self.class_info(name_or_atom) {
            return window_class.name.clone();
        }
        normalize_class_name(name_or_atom)
    }

    pub fn find_window(&self, class_name: Option<&str>, title: Option<&str>) -> Option<u32> {
        let class_name = class_name.map(|class_name| self.resolve_class_name(class_name));
        self.windows
            .iter()
            .find(|(_, window)| {
                !window.destroyed
                    && class_name
                        .as_ref()
                        .is_none_or(|class_name| &window.class_name == class_name)
                    && title.is_none_or(|title| window.title == title)
            })
            .map(|(hwnd, _)| *hwnd)
    }

    pub fn destroy_window(&mut self, hwnd: u32, _time_ms: u32) -> bool {
        let Some(targets) = self.window_and_descendants(hwnd) else {
            return false;
        };
        let exposed_rects: Vec<Rect> = targets
            .iter()
            .filter_map(|target| {
                let window = self.windows.get(target)?;
                (!window.destroyed && self.is_window_visible(*target))
                    .then_some(window.client_rect.normalized())
            })
            .collect();
        for target in targets.iter().rev().copied() {
            if let Some(window) = self.windows.get_mut(&target) {
                window.being_destroyed = false;
                window.destroyed = true;
            }
            self.window_regions.remove(&target);
            self.z_order.retain(|candidate| *candidate != target);
        }
        if targets.contains(&self.capture.unwrap_or(0)) {
            self.capture = None;
        }
        if targets.contains(&self.focus.unwrap_or(0)) {
            self.focus = None;
        }
        if targets.contains(&self.active_window.unwrap_or(0)) {
            self.active_window = None;
        }
        if self
            .caret
            .is_some_and(|caret| targets.contains(&caret.hwnd))
        {
            self.caret = None;
        }
        self.keyboard_target_by_thread
            .retain(|_, hwnd| !targets.contains(hwnd));
        for queue in self.queues.values_mut() {
            queue.retain(|message| message.hwnd == 0 || !targets.contains(&message.hwnd));
        }
        let doomed_sent: Vec<u64> = self
            .sent_messages
            .iter()
            .filter(|(_, sent)| sent.message.hwnd != 0 && targets.contains(&sent.message.hwnd))
            .map(|(id, _)| *id)
            .collect();
        for id in &doomed_sent {
            if let Some(sent) = self.sent_messages.get_mut(id) {
                sent.flags |= SMF_RECEIVER_TERMINATED | SMF_RESULT_READY;
                sent.result = Some(0);
                self.stats.send_transaction_receiver_terminated_count = self
                    .stats
                    .send_transaction_receiver_terminated_count
                    .saturating_add(1);
            }
        }
        for queue in self.sent_queues.values_mut() {
            queue.retain(|id| !doomed_sent.contains(id));
        }
        for rect in exposed_rects {
            self.invalidate_visible_windows_in_screen_rect(rect, true, &targets);
        }
        true
    }

    pub fn mark_window_subtree_being_destroyed(&mut self, hwnd: u32) -> bool {
        let Some(targets) = self.window_and_descendants(hwnd) else {
            return false;
        };
        for target in targets {
            if let Some(window) = self.windows.get_mut(&target) {
                window.being_destroyed = true;
            }
        }
        true
    }

    pub fn is_window_being_destroyed(&self, hwnd: u32) -> bool {
        self.windows
            .get(&hwnd)
            .is_some_and(|window| !window.destroyed && window.being_destroyed)
    }

    pub fn show_window(&mut self, hwnd: u32, visible: bool) -> bool {
        let Some(window) = self.windows.get(&hwnd) else {
            return false;
        };
        let exposed_rect = (!visible && !window.destroyed && self.is_window_visible(hwnd))
            .then_some(window.client_rect.normalized());
        let Some(window) = self.windows.get_mut(&hwnd) else {
            return false;
        };
        let previous = window.visible;
        window.visible = visible;
        if visible {
            window.style |= WS_VISIBLE;
        } else {
            window.style &= !WS_VISIBLE;
        }
        if visible && !previous {
            window.update_pending = true;
            window.erase_pending = true;
            window.update_rect = window.client_rect.zero_origin();
            let thread_id = window.thread_id;
            let _ = window;
            if self.is_window_visible(hwnd) {
                self.mark_queue_status_changed(thread_id, QS_PAINT);
            }
        } else if !visible {
            Self::clear_window_update(window);
            if let Some(rect) = exposed_rect {
                self.invalidate_visible_windows_in_screen_rect(rect, true, &[hwnd]);
            }
        }
        previous
    }

    pub fn invalidate_window(&mut self, hwnd: u32, rect: Option<Rect>, erase: bool) -> bool {
        let Some(window) = self.windows.get_mut(&hwnd) else {
            return false;
        };
        if window.destroyed {
            return false;
        }
        let full_client = window.client_rect.zero_origin();
        let Some(rect) = rect
            .unwrap_or(full_client)
            .normalized()
            .intersect(full_client)
        else {
            return true;
        };
        window.update_rect = if window.update_pending {
            window.update_rect.union(rect)
        } else {
            rect
        };
        window.update_pending = true;
        window.erase_pending |= erase;
        let thread_id = window.thread_id;
        let _ = window;
        if self.is_window_visible(hwnd) {
            self.mark_queue_status_changed(thread_id, QS_PAINT);
        }
        true
    }

    pub fn validate_window(&mut self, hwnd: u32) -> bool {
        let Some(window) = self.windows.get_mut(&hwnd) else {
            return false;
        };
        if window.destroyed {
            return false;
        }
        window.update_pending = false;
        window.erase_pending = false;
        window.update_rect = Rect::default();
        true
    }

    pub fn validate_window_rect(&mut self, hwnd: u32, rect: Option<Rect>) -> bool {
        let Some(window) = self.windows.get_mut(&hwnd) else {
            return false;
        };
        if window.destroyed {
            return false;
        }
        let Some(rect) = rect else {
            window.update_pending = false;
            window.erase_pending = false;
            window.update_rect = Rect::default();
            return true;
        };
        if !window.update_pending {
            return true;
        }
        let Some(rect) = rect
            .normalized()
            .intersect(window.client_rect.zero_origin())
        else {
            return true;
        };
        match window.update_rect.subtract_bounding(rect) {
            Some(remaining) => {
                window.update_rect = remaining;
            }
            None => {
                window.update_pending = false;
                window.erase_pending = false;
                window.update_rect = Rect::default();
            }
        }
        true
    }

    pub fn update_rect(&self, hwnd: u32) -> Option<PaintUpdate> {
        let window = self.windows.get(&hwnd)?;
        (!window.destroyed && window.update_pending).then_some(PaintUpdate {
            rect: window.update_rect,
            erase: window.erase_pending,
        })
    }

    pub fn mark_pending_size_move(&mut self, hwnd: u32, moved: bool, sized: bool) -> bool {
        let Some(window) = self.windows.get_mut(&hwnd) else {
            return false;
        };
        if window.destroyed {
            return false;
        }
        window.pending_move |= moved;
        window.pending_size |= sized;
        true
    }

    pub fn take_pending_size_move(&mut self, hwnd: u32) -> Option<(Rect, bool, bool)> {
        let window = self.windows.get_mut(&hwnd)?;
        if window.destroyed || (!window.pending_move && !window.pending_size) {
            return None;
        }
        let rect = window.rect;
        let pending_move = window.pending_move;
        let pending_size = window.pending_size;
        window.pending_move = false;
        window.pending_size = false;
        Some((rect, pending_move, pending_size))
    }

    pub fn clear_update_erase(&mut self, hwnd: u32) -> bool {
        let Some(window) = self.windows.get_mut(&hwnd) else {
            return false;
        };
        if window.destroyed {
            return false;
        }
        window.erase_pending = false;
        true
    }

    fn clamp_window_update_to_client(window: &mut Window) {
        if !window.update_pending {
            return;
        }
        let Some(rect) = window
            .update_rect
            .intersect(window.client_rect.zero_origin())
        else {
            window.update_pending = false;
            window.erase_pending = false;
            window.update_rect = Rect::default();
            return;
        };
        window.update_rect = rect;
    }

    fn clear_window_update(window: &mut Window) {
        window.update_pending = false;
        window.erase_pending = false;
        window.update_rect = Rect::default();
    }

    fn invalidate_visible_windows_in_screen_rect(
        &mut self,
        screen_rect: Rect,
        erase: bool,
        excluded: &[u32],
    ) {
        let screen_rect = screen_rect.normalized();
        if screen_rect.is_empty() {
            return;
        }
        let targets: Vec<(u32, Rect)> = self
            .z_order
            .iter()
            .copied()
            .filter(|hwnd| !excluded.contains(hwnd))
            .filter_map(|hwnd| {
                let window = self.windows.get(&hwnd)?;
                if window.destroyed || !self.is_window_visible(hwnd) {
                    return None;
                }
                let intersection = window.client_rect.normalized().intersect(screen_rect)?;
                Some((
                    hwnd,
                    intersection.offset(-window.client_rect.left, -window.client_rect.top),
                ))
            })
            .collect();
        for (hwnd, rect) in targets {
            self.invalidate_window(hwnd, Some(rect), erase);
        }
    }

    pub fn begin_paint(&mut self, hwnd: u32) -> Option<PaintUpdate> {
        let window = self.windows.get(&hwnd)?;
        if window.destroyed {
            return None;
        }
        let update = PaintUpdate {
            rect: if window.update_pending {
                window.update_rect
            } else {
                window.client_rect.zero_origin()
            },
            erase: window.erase_pending,
        };
        self.validate_window(hwnd);
        Some(update)
    }

    pub fn enable_window(&mut self, hwnd: u32, enabled: bool) -> bool {
        let Some(window) = self.windows.get_mut(&hwnd) else {
            return false;
        };
        if window.destroyed {
            return false;
        }
        let previous = window.enabled;
        window.enabled = enabled;
        if enabled {
            window.style &= !WS_DISABLED;
        } else {
            window.style |= WS_DISABLED;
        }
        previous
    }

    pub fn is_window_enabled(&self, hwnd: u32) -> bool {
        let mut current = Some(hwnd);
        while let Some(hwnd) = current {
            let Some(window) = self.windows.get(&hwnd) else {
                return false;
            };
            if window.destroyed || !window.enabled || window.style & WS_DISABLED != 0 {
                return false;
            }
            current = window.parent;
        }
        true
    }

    pub fn is_window_visible(&self, hwnd: u32) -> bool {
        let mut current = Some(hwnd);
        while let Some(hwnd) = current {
            let Some(window) = self.windows.get(&hwnd) else {
                return false;
            };
            if window.destroyed || !window.visible || window.style & WS_VISIBLE == 0 {
                return false;
            }
            current = window.parent;
        }
        true
    }

    pub fn get_parent(&self, hwnd: u32) -> Option<u32> {
        let window = self.windows.get(&hwnd)?;
        if window.destroyed {
            return None;
        }
        if let Some(parent) = window.parent.filter(|parent| self.is_window(*parent)) {
            return Some(parent);
        }
        if window.style & WS_POPUP != 0 {
            return window.owner.filter(|owner| self.is_window(*owner));
        }
        None
    }

    pub fn is_child(&self, parent: u32, child: u32) -> bool {
        if parent == child || !self.is_window(parent) || !self.is_window(child) {
            return false;
        }
        let mut current = self.windows.get(&child).and_then(|window| window.parent);
        while let Some(hwnd) = current {
            if hwnd == parent {
                return true;
            }
            current = self.windows.get(&hwnd).and_then(|window| window.parent);
        }
        false
    }

    pub fn get_dlg_item(&self, parent: u32, id: u32) -> Option<u32> {
        self.windows
            .values()
            .find(|window| !window.destroyed && window.parent == Some(parent) && window.id == id)
            .map(|window| window.hwnd)
    }

    pub fn get_dlg_ctrl_id(&self, hwnd: u32) -> Option<u32> {
        let window = self.windows.get(&hwnd)?;
        (!window.destroyed).then_some(window.id)
    }

    pub fn get_menu(&self, hwnd: u32) -> Option<u32> {
        let window = self.windows.get(&hwnd)?;
        (!window.destroyed).then_some(window.menu.unwrap_or(0))
    }

    pub fn set_menu(&mut self, hwnd: u32, menu: u32) -> bool {
        let Some(window) = self.windows.get_mut(&hwnd) else {
            return false;
        };
        if window.destroyed {
            return false;
        }
        window.menu = (menu != 0).then_some(menu);
        true
    }

    pub fn draw_menu_bar(&self, hwnd: u32) -> bool {
        self.is_window(hwnd)
    }

    pub fn end_dialog(&mut self, hwnd: u32, result: u32) -> bool {
        if !self.is_window(hwnd) {
            return false;
        }
        self.dialog_results.insert(hwnd, result);
        true
    }

    pub fn dialog_result(&self, hwnd: u32) -> Option<u32> {
        self.dialog_results.get(&hwnd).copied()
    }

    pub fn check_dlg_button(&mut self, hwnd: u32, id: u32, check: u32) -> bool {
        if !self.is_window(hwnd) {
            return false;
        }
        self.dialog_checks.insert((hwnd, id), check);
        true
    }

    pub fn check_radio_button(&mut self, hwnd: u32, first: u32, last: u32, check: u32) -> bool {
        if !self.is_window(hwnd) || first > last || check < first || check > last {
            return false;
        }
        for id in first..=last {
            self.dialog_checks
                .insert((hwnd, id), u32::from(id == check));
        }
        true
    }

    pub fn is_dlg_button_checked(&self, hwnd: u32, id: u32) -> Option<u32> {
        self.is_window(hwnd)
            .then(|| self.dialog_checks.get(&(hwnd, id)).copied().unwrap_or(0))
    }

    pub fn set_window_region(&mut self, hwnd: u32, rect: Option<Rect>) -> bool {
        self.set_window_region_rects(hwnd, rect.map(|rect| vec![rect]))
    }

    pub fn set_window_region_rects(&mut self, hwnd: u32, rects: Option<Vec<Rect>>) -> bool {
        if !self.is_window(hwnd) {
            return false;
        }
        if let Some(rects) = rects {
            let rects = canonicalize_region_rects(rects);
            let rect = bounding_region_rect(&rects);
            self.window_regions
                .insert(hwnd, WindowRegion { rect, rects });
        } else {
            self.window_regions.remove(&hwnd);
        }
        true
    }

    pub fn window_region(&self, hwnd: u32) -> Option<Rect> {
        self.is_window(hwnd)
            .then(|| self.window_regions.get(&hwnd).map(|region| region.rect))
            .flatten()
    }

    pub fn window_region_rects(&self, hwnd: u32) -> Option<&[Rect]> {
        self.is_window(hwnd)
            .then(|| {
                self.window_regions
                    .get(&hwnd)
                    .map(|region| region.rects.as_slice())
            })
            .flatten()
    }

    pub fn visible_client_rects(&self, hwnd: u32) -> Vec<Rect> {
        let Some(window) = self.windows.get(&hwnd) else {
            return Vec::new();
        };
        let origin = window.client_rect;
        self.visible_client_screen_rects_for_paint(hwnd)
            .into_iter()
            .map(|rect| rect.offset(-origin.left, -origin.top))
            .collect()
    }

    pub fn set_parent(&mut self, hwnd: u32, parent: Option<u32>) -> Option<Option<u32>> {
        if parent.is_some_and(|parent| {
            !self.is_window(parent) || parent == hwnd || self.is_child(hwnd, parent)
        }) {
            return None;
        }
        if !self.is_window(hwnd) {
            return None;
        }
        let window = self.windows.get_mut(&hwnd)?;
        let previous = window.parent;
        window.parent = parent;
        self.apply_z_order(hwnd, parent, HWND_TOP);
        Some(previous)
    }

    pub fn get_window(&self, hwnd: u32, cmd: u32) -> Option<u32> {
        let window = self.windows.get(&hwnd)?;
        if window.destroyed {
            return None;
        }

        match cmd {
            GW_OWNER => window.owner.filter(|owner| self.is_window(*owner)),
            GW_CHILD => self.first_child(hwnd),
            GW_HWNDFIRST | GW_HWNDLAST | GW_HWNDNEXT | GW_HWNDPREV => {
                let siblings = self.sibling_windows(window.parent);
                let index = siblings.iter().position(|candidate| *candidate == hwnd)?;
                match cmd {
                    GW_HWNDFIRST => siblings.first().copied(),
                    GW_HWNDLAST => siblings.last().copied(),
                    GW_HWNDNEXT => siblings.get(index + 1).copied(),
                    GW_HWNDPREV => index
                        .checked_sub(1)
                        .and_then(|index| siblings.get(index))
                        .copied(),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn get_next_dlg_tab_item(&self, dialog: u32, control: u32, previous: bool) -> Option<u32> {
        let candidates = self.dialog_child_candidates(dialog, WS_TABSTOP);
        self.next_dialog_candidate(&candidates, control, previous)
    }

    pub fn get_next_dlg_group_item(
        &self,
        dialog: u32,
        control: u32,
        previous: bool,
    ) -> Option<u32> {
        let children = self.dialog_child_candidates(dialog, 0);
        if children.is_empty() {
            return None;
        }
        let current_index = children
            .iter()
            .position(|hwnd| *hwnd == control)
            .unwrap_or(if previous { children.len() - 1 } else { 0 });
        let mut group_start = 0;
        for index in (0..=current_index).rev() {
            if self.window_has_style(children[index], WS_GROUP) {
                group_start = index;
                break;
            }
        }
        let mut group_end = children.len();
        for (index, hwnd) in children.iter().enumerate().skip(current_index + 1) {
            if self.window_has_style(*hwnd, WS_GROUP) {
                group_end = index;
                break;
            }
        }
        self.next_dialog_candidate(&children[group_start..group_end], control, previous)
    }

    pub fn get_desktop_window(&self) -> u32 {
        DESKTOP_HWND
    }

    pub fn set_focus(&mut self, hwnd: Option<u32>) -> Option<u32> {
        if hwnd.is_some_and(|hwnd| !self.is_window(hwnd)) {
            return None;
        }
        let previous = self.focus;
        self.focus = hwnd;
        previous
    }

    pub fn get_focus(&self) -> Option<u32> {
        self.focus
    }

    pub fn focus_is_within(&self, hwnd: u32) -> bool {
        self.focus
            .is_some_and(|focus| focus == hwnd || self.is_child(hwnd, focus))
    }

    pub fn set_keyboard_target(&mut self, thread_id: u32, hwnd: Option<u32>) -> Option<u32> {
        if hwnd.is_some_and(|hwnd| !self.is_window(hwnd)) {
            return None;
        }
        let previous = self.get_keyboard_target(thread_id);
        match hwnd {
            Some(hwnd) => {
                self.keyboard_target_by_thread.insert(thread_id, hwnd);
            }
            None => {
                self.keyboard_target_by_thread.remove(&thread_id);
            }
        }
        previous
    }

    pub fn get_keyboard_target(&self, thread_id: u32) -> Option<u32> {
        self.keyboard_target_by_thread
            .get(&thread_id)
            .copied()
            .filter(|hwnd| self.is_window(*hwnd))
    }

    pub fn get_foreground_keyboard_target(&self) -> Option<u32> {
        self.get_active_window()
            .and_then(|active| self.windows.get(&active).map(|window| window.thread_id))
            .and_then(|thread_id| self.get_keyboard_target(thread_id))
            .or_else(|| self.get_focus())
            .or_else(|| self.get_active_window())
    }

    pub fn keyboard_target_is_within(&self, hwnd: u32) -> bool {
        self.keyboard_target_by_thread
            .values()
            .copied()
            .any(|target| target == hwnd || self.is_child(hwnd, target))
    }

    pub fn clear_keyboard_targets_within(&mut self, hwnd: u32) {
        let targets: Vec<u32> = self
            .keyboard_target_by_thread
            .iter()
            .filter(|(_, target)| **target == hwnd || self.is_child(hwnd, **target))
            .map(|(thread_id, _)| *thread_id)
            .collect();
        for thread_id in targets {
            self.keyboard_target_by_thread.remove(&thread_id);
        }
    }

    pub fn set_active_window(&mut self, hwnd: Option<u32>) -> Option<u32> {
        if hwnd.is_some_and(|hwnd| !self.is_window(hwnd)) {
            return None;
        }
        let previous = self.active_window;
        self.active_window = hwnd;
        previous
    }

    pub fn set_capture(&mut self, hwnd: u32) -> Option<u32> {
        if !self.is_window(hwnd) {
            return None;
        }
        let previous = self.capture;
        self.capture = Some(hwnd);
        previous
    }

    pub fn get_capture(&self) -> Option<u32> {
        self.capture.filter(|hwnd| self.is_window(*hwnd))
    }

    pub fn release_capture(&mut self) -> bool {
        self.capture = None;
        true
    }

    pub fn set_cursor(&mut self, cursor: u32) -> Option<u32> {
        let previous = self.cursor;
        self.cursor = (cursor != 0).then_some(cursor);
        previous
    }

    pub fn get_cursor(&self) -> Option<u32> {
        self.cursor
    }

    pub fn get_active_window(&self) -> Option<u32> {
        if let Some(hwnd) = self
            .active_window
            .filter(|hwnd| self.is_window(*hwnd) && self.is_window_enabled(*hwnd))
        {
            return Some(hwnd);
        }
        if let Some(hwnd) = self
            .focus
            .filter(|hwnd| self.is_window(*hwnd) && self.is_window_enabled(*hwnd))
        {
            return Some(hwnd);
        }
        self.windows
            .values()
            .find(|window| {
                !window.destroyed
                    && window.parent.is_none()
                    && window.hwnd != DESKTOP_HWND
                    && self.is_window_enabled(window.hwnd)
            })
            .map(|window| window.hwnd)
    }

    pub fn active_window_is_within(&self, hwnd: u32) -> bool {
        self.active_window
            .is_some_and(|active| active == hwnd || self.is_child(hwnd, active))
    }

    pub fn set_cursor_pos(&mut self, point: Point) {
        self.cursor_pos = point;
    }

    pub fn get_cursor_pos(&self) -> Point {
        self.cursor_pos
    }

    pub fn system_metric(&self, index: u32) -> i32 {
        let desktop = self.desktop_rect();
        match index {
            SM_CXSCREEN | SM_CXFULLSCREEN | SM_CXVIRTUALSCREEN => desktop.width(),
            SM_CYSCREEN | SM_CYFULLSCREEN | SM_CYVIRTUALSCREEN => desktop.height(),
            SM_XVIRTUALSCREEN => desktop.left,
            SM_YVIRTUALSCREEN => desktop.top,
            SM_CXVSCROLL | SM_CXHSCROLL => 13,
            SM_CYVSCROLL | SM_CYHSCROLL => 13,
            SM_CYCAPTION | SM_CYMENU => 24,
            SM_CXBORDER | SM_CYBORDER => 1,
            SM_CXDLGFRAME | SM_CYDLGFRAME => 3,
            SM_CXICON | SM_CYICON | SM_CXCURSOR | SM_CYCURSOR => 32,
            SM_CXSMICON | SM_CYSMICON => 16,
            SM_CXDOUBLECLK | SM_CYDOUBLECLK => 4,
            SM_CXICONSPACING | SM_CYICONSPACING => 75,
            SM_CXEDGE | SM_CYEDGE => 2,
            SM_MOUSEPRESENT => 1,
            SM_CMONITORS | SM_SAMEDISPLAYFORMAT => 1,
            SM_DEBUG => 0,
            _ => 0,
        }
    }

    pub fn set_window_pos(
        &mut self,
        hwnd: u32,
        insert_after: Option<u32>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        flags: u32,
    ) -> bool {
        let Some(window) = self.windows.get(&hwnd) else {
            return false;
        };
        let old = window.rect;
        let old_visible = self.is_window_visible(hwnd);
        let parent = window.parent;
        let mut left = old.left;
        let mut top = old.top;
        let mut right = old.right;
        let mut bottom = old.bottom;

        if flags & SWP_NOMOVE == 0 {
            let origin = self.parent_client_origin(parent);
            left = origin.x.saturating_add(x);
            top = origin.y.saturating_add(y);
            if flags & SWP_NOSIZE != 0 {
                right = left.saturating_add(old.width());
                bottom = top.saturating_add(old.height());
            }
        }

        if flags & SWP_NOSIZE == 0 {
            right = left.saturating_add(width);
            bottom = top.saturating_add(height);
        }

        let Some(window) = self.windows.get_mut(&hwnd) else {
            return false;
        };
        window.rect = Rect {
            left,
            top,
            right,
            bottom,
        };
        window.client_rect = window.rect;
        Self::clamp_window_update_to_client(window);
        let mut paint_changed_thread = None;
        if flags & SWP_SHOWWINDOW != 0 {
            window.visible = true;
            window.style |= WS_VISIBLE;
            window.update_pending = true;
            window.erase_pending = true;
            window.update_rect = window.client_rect.zero_origin();
            paint_changed_thread = Some(window.thread_id);
        }
        if flags & SWP_HIDEWINDOW != 0 {
            window.visible = false;
            window.style &= !WS_VISIBLE;
            Self::clear_window_update(window);
        }
        if flags & SWP_NOZORDER == 0 {
            self.apply_z_order(hwnd, parent, insert_after.unwrap_or(HWND_TOP));
        }
        if let Some(thread_id) = paint_changed_thread.filter(|_| self.is_window_visible(hwnd)) {
            self.mark_queue_status_changed(thread_id, QS_PAINT);
        }
        let moved_or_sized = flags & SWP_NOMOVE == 0 || flags & SWP_NOSIZE == 0;
        let z_order_changed = flags & SWP_NOZORDER == 0;
        if flags & SWP_HIDEWINDOW == 0
            && flags & SWP_NOREDRAW == 0
            && old_visible
            && self.is_window_visible(hwnd)
            && flags & SWP_SHOWWINDOW == 0
            && (moved_or_sized || z_order_changed)
        {
            self.invalidate_window(hwnd, None, true);
        }
        if old_visible && (moved_or_sized || flags & SWP_HIDEWINDOW != 0) {
            self.invalidate_visible_windows_in_screen_rect(old.normalized(), true, &[hwnd]);
        }
        true
    }

    pub fn move_window(
        &mut self,
        hwnd: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        _repaint: bool,
    ) -> bool {
        let flags = if _repaint { 0 } else { SWP_NOREDRAW };
        let moved = self.set_window_pos(hwnd, None, x, y, width, height, flags);
        if moved && _repaint {
            self.invalidate_window(hwnd, None, true);
        }
        moved
    }

    pub fn get_window_rect(&self, hwnd: u32) -> Option<Rect> {
        Some(self.windows.get(&hwnd)?.rect)
    }

    pub fn get_client_rect(&self, hwnd: u32) -> Option<Rect> {
        Some(self.windows.get(&hwnd)?.client_rect.zero_origin())
    }

    pub fn window_pos_for_rect(
        &self,
        hwnd: u32,
        rect: Rect,
        insert_after: u32,
        flags: u32,
    ) -> Option<WindowPos> {
        let window = self.windows.get(&hwnd)?;
        let origin = self.parent_client_origin(window.parent);
        Some(WindowPos {
            hwnd,
            insert_after,
            x: rect.left.saturating_sub(origin.x),
            y: rect.top.saturating_sub(origin.y),
            width: rect.width(),
            height: rect.height(),
            flags,
        })
    }

    pub fn client_to_screen(&self, hwnd: u32, point: Point) -> Option<Point> {
        let origin = self.windows.get(&hwnd)?.client_rect;
        Some(Point {
            x: point.x.saturating_add(origin.left),
            y: point.y.saturating_add(origin.top),
        })
    }

    pub fn screen_to_client(&self, hwnd: u32, point: Point) -> Option<Point> {
        let origin = self.windows.get(&hwnd)?.client_rect;
        Some(Point {
            x: point.x.saturating_sub(origin.left),
            y: point.y.saturating_sub(origin.top),
        })
    }

    pub fn map_window_points(
        &self,
        from: Option<u32>,
        to: Option<u32>,
        points: &mut [Point],
    ) -> bool {
        let from_origin = match from {
            Some(hwnd) => match self.windows.get(&hwnd) {
                Some(window) => window.client_rect,
                None => return false,
            },
            None => Rect::default(),
        };
        let to_origin = match to {
            Some(hwnd) => match self.windows.get(&hwnd) {
                Some(window) => window.client_rect,
                None => return false,
            },
            None => Rect::default(),
        };
        let dx = from_origin.left.saturating_sub(to_origin.left);
        let dy = from_origin.top.saturating_sub(to_origin.top);
        for point in points {
            point.x = point.x.saturating_add(dx);
            point.y = point.y.saturating_add(dy);
        }
        true
    }

    pub fn post_message(&mut self, thread_id: u32, mut message: Message) {
        if message.source == MSGSRC_UNKNOWN {
            message.source = MSGSRC_SOFTWARE_POST;
        }
        if message.mouse_pos_at_post.is_none() && is_mouse_message(message.msg) {
            message.mouse_pos_at_post = Some(message.lparam);
        }
        self.update_key_state_for_message(message.msg, message.wparam);
        let status_bit = queue_status_bit_for_message(message.msg);
        let ready_time = message.time_ms;
        self.queues.entry(thread_id).or_default().push_back(message);
        self.ready_timestamp_by_thread.insert(thread_id, ready_time);
        self.mark_queue_status_changed(thread_id, status_bit);
    }

    pub fn post_message_for_window(&mut self, hwnd: u32, message: Message) -> bool {
        let Some(window) = self.windows.get(&hwnd) else {
            return false;
        };
        if window.destroyed {
            return false;
        }
        self.post_message(window.thread_id, message);
        true
    }

    pub fn insert_message_pointer_payload(
        &mut self,
        ptr: u32,
        payload: MessagePointerPayload,
    ) -> bool {
        if ptr == 0 {
            return false;
        }
        self.message_pointer_payloads.insert(ptr, payload);
        true
    }

    pub fn message_pointer_payload(&self, ptr: u32) -> Option<MessagePointerPayload> {
        self.message_pointer_payloads.get(&ptr).copied()
    }

    pub fn take_message_pointer_payload(&mut self, ptr: u32) -> Option<MessagePointerPayload> {
        self.message_pointer_payloads.remove(&ptr)
    }

    pub fn queue_sent_message_for_window(&mut self, hwnd: u32, message: Message) -> bool {
        self.queue_send_message_for_window(None, hwnd, message, SMF_SENDER_NO_WAIT, None)
            .is_some()
    }

    pub fn queue_send_message_for_window(
        &mut self,
        sender_thread_id: Option<u32>,
        hwnd: u32,
        mut message: Message,
        flags: u32,
        timeout_ms: Option<u32>,
    ) -> Option<u64> {
        let Some(window) = self.windows.get(&hwnd) else {
            return None;
        };
        if window.destroyed {
            return None;
        }
        if message.source == MSGSRC_UNKNOWN {
            message.source = MSGSRC_SOFTWARE_SEND;
        }
        let ready_time = message.time_ms;
        let id = self.next_sent_message_id;
        self.next_sent_message_id = self.next_sent_message_id.saturating_add(1).max(1);
        self.sent_messages.insert(
            id,
            SentMessage {
                id,
                sender_thread_id,
                receiver_thread_id: window.thread_id,
                message,
                flags,
                timeout_ms,
                result: None,
            },
        );
        let receiver_thread_id = window.thread_id;
        let queue = self.sent_queues.entry(receiver_thread_id).or_default();
        queue.push_back(id);
        let queue_depth = queue.len();
        self.ready_timestamp_by_thread
            .insert(receiver_thread_id, ready_time);
        self.mark_queue_status_changed(receiver_thread_id, QS_SENDMESSAGE);
        self.stats.send_transaction_count = self.stats.send_transaction_count.saturating_add(1);
        self.stats.max_sent_queue_depth = self.stats.max_sent_queue_depth.max(queue_depth);
        Some(id)
    }

    pub fn post_broadcast_message(
        &mut self,
        msg: u32,
        wparam: u32,
        lparam: u32,
        time_ms: u32,
    ) -> bool {
        let targets: Vec<(u32, u32)> = self
            .windows
            .values()
            .filter(|window| !window.destroyed && window.parent.is_none())
            .map(|window| (window.hwnd, window.thread_id))
            .collect();
        for (hwnd, thread_id) in &targets {
            self.post_message(
                *thread_id,
                Message::new(*hwnd, msg, wparam, lparam, time_ms),
            );
        }
        !targets.is_empty()
    }

    pub fn post_thread_message(
        &mut self,
        thread_id: u32,
        msg: u32,
        wparam: u32,
        lparam: u32,
        time_ms: u32,
    ) {
        self.post_message(thread_id, Message::new(0, msg, wparam, lparam, time_ms));
    }

    pub fn send_message(&mut self, hwnd: u32, msg: u32, wparam: u32, lparam: u32) -> Option<u32> {
        if !self.is_window(hwnd) {
            return None;
        }
        match msg {
            WM_CLOSE => {
                let _ = self.send_message(hwnd, WM_DESTROY, 0, 0);
                self.destroy_window(hwnd, 0);
            }
            WM_DESTROY | WM_NCDESTROY => {
                self.record_destroy_lifecycle_message(hwnd, msg);
            }
            WM_PAINT => {
                self.validate_window(hwnd);
            }
            WM_GETDLGCODE => return Some(self.window_dialog_code(hwnd)),
            DM_GETDEFID => return Some(self.dialog_default_id_result(hwnd)),
            DM_SETDEFID => {
                self.set_dialog_default_id(hwnd, wparam);
                return Some(1);
            }
            _ => {}
        }
        Some(default_send_message_result(msg, wparam, lparam))
    }

    pub fn dialog_return_command(&self, dialog: u32, source: u32, fallback: u32) -> (u32, u32) {
        if self.is_push_button(source) {
            let id = self
                .get_dlg_ctrl_id(source)
                .filter(|id| *id != 0)
                .unwrap_or(fallback);
            return (id, source);
        }
        if let Some(default_hwnd) = self.default_push_button(dialog) {
            let id = self
                .get_dlg_ctrl_id(default_hwnd)
                .filter(|id| *id != 0)
                .unwrap_or(fallback);
            return (id, default_hwnd);
        }
        (fallback, 0)
    }

    pub fn default_push_button(&self, dialog: u32) -> Option<u32> {
        self.windows
            .values()
            .find(|window| {
                !window.destroyed
                    && window.parent == Some(dialog)
                    && self.window_is_push_button(window)
                    && window.style & BS_TYPEMASK == BS_DEFPUSHBUTTON
            })
            .map(|window| window.hwnd)
    }

    pub fn is_push_button(&self, hwnd: u32) -> bool {
        self.windows
            .get(&hwnd)
            .is_some_and(|window| self.window_is_push_button(window))
    }

    pub fn get_key_state(&self, virtual_key: u32) -> u32 {
        let state = self.effective_key_state(virtual_key);
        (state as i16 as i32) as u32
    }

    pub fn get_async_key_state(&mut self, virtual_key: u32) -> u32 {
        let state = self.effective_key_state(virtual_key);
        let async_down = self.consume_async_key_down(virtual_key);
        let result = (state & 0x8000) | u16::from(async_down);
        (result as i16 as i32) as u32
    }

    pub fn get_async_shift_flags(&mut self, virtual_key: u32) -> u32 {
        let state = self.effective_key_state(virtual_key);
        let mut flags = 0;
        if state & 0x0001 != 0 {
            flags |= KEY_STATE_TOGGLED_FLAG;
        }
        if self.consume_async_key_down(virtual_key) {
            flags |= KEY_STATE_GET_ASYNC_DOWN_FLAG;
        }
        if state & 0x8000 != 0 {
            flags |= KEY_STATE_DOWN_FLAG;
        }
        flags | self.current_shift_flags()
    }

    fn window_dialog_code(&self, hwnd: u32) -> u32 {
        let Some(window) = self.windows.get(&hwnd).filter(|window| !window.destroyed) else {
            return 0;
        };
        if window.class_name.eq_ignore_ascii_case("button") {
            let mut code = DLGC_BUTTON;
            match window.style & BS_TYPEMASK {
                BS_DEFPUSHBUTTON => code |= DLGC_DEFPUSHBUTTON,
                BS_PUSHBUTTON => code |= DLGC_UNDEFPUSHBUTTON,
                BS_RADIOBUTTON | BS_AUTORADIOBUTTON => code |= DLGC_RADIOBUTTON,
                _ => {}
            }
            return code;
        }
        if window.class_name.eq_ignore_ascii_case("static") {
            return DLGC_STATIC;
        }
        if window.class_name.eq_ignore_ascii_case("edit") {
            return DLGC_HASSETSEL | DLGC_WANTCHARS;
        }
        default_send_message_result(WM_GETDLGCODE, 0, 0)
    }

    fn dialog_default_id_result(&self, dialog: u32) -> u32 {
        self.default_push_button(dialog)
            .and_then(|hwnd| self.get_dlg_ctrl_id(hwnd))
            .filter(|id| *id != 0)
            .map(|id| id | (DC_HASDEFID << 16))
            .unwrap_or(0)
    }

    fn set_dialog_default_id(&mut self, dialog: u32, id: u32) {
        let children: Vec<u32> = self
            .windows
            .values()
            .filter(|window| {
                !window.destroyed
                    && window.parent == Some(dialog)
                    && self.window_is_push_button(window)
            })
            .map(|window| window.hwnd)
            .collect();
        for hwnd in children {
            if let Some(window) = self.windows.get_mut(&hwnd) {
                let button_type = if window.id == id {
                    BS_DEFPUSHBUTTON
                } else if window.style & BS_TYPEMASK == BS_DEFPUSHBUTTON {
                    BS_PUSHBUTTON
                } else {
                    window.style & BS_TYPEMASK
                };
                window.style = (window.style & !BS_TYPEMASK) | button_type;
            }
        }
    }

    fn window_is_push_button(&self, window: &Window) -> bool {
        window.class_name.eq_ignore_ascii_case("button")
            && matches!(window.style & BS_TYPEMASK, BS_PUSHBUTTON | BS_DEFPUSHBUTTON)
    }

    fn update_key_state_for_message(&mut self, msg: u32, virtual_key: u32) {
        match msg {
            WM_KEYDOWN => self.set_key_down(virtual_key),
            WM_KEYUP => self.set_key_up(virtual_key),
            WM_LBUTTONDOWN => self.set_key_down(VK_LBUTTON),
            WM_LBUTTONUP => self.set_key_up(VK_LBUTTON),
            _ => {}
        }
    }

    fn set_key_down(&mut self, virtual_key: u32) {
        let Some(state) = self.key_state.get_mut(virtual_key as usize) else {
            return;
        };
        let was_down = *state & 0x8000 != 0;
        if virtual_key == VK_CAPITAL && !was_down {
            *state ^= 0x0001;
        }
        *state |= 0x8000;
        if let Some(async_down) = self.async_key_down.get_mut(virtual_key as usize) {
            *async_down = true;
        }
        if let Some(alias) = modifier_alias_for(virtual_key) {
            if let Some(state) = self.key_state.get_mut(alias as usize) {
                *state |= 0x8000;
            }
            if let Some(async_down) = self.async_key_down.get_mut(alias as usize) {
                *async_down = true;
            }
        }
    }

    fn set_key_up(&mut self, virtual_key: u32) {
        let Some(state) = self.key_state.get_mut(virtual_key as usize) else {
            return;
        };
        *state &= !0x8000;
        if let Some(alias) = modifier_alias_for(virtual_key) {
            let still_down = match alias {
                VK_SHIFT => {
                    self.effective_key_state(VK_LSHIFT) & 0x8000 != 0
                        || self.effective_key_state(VK_RSHIFT) & 0x8000 != 0
                }
                VK_CONTROL => {
                    self.effective_key_state(VK_LCONTROL) & 0x8000 != 0
                        || self.effective_key_state(VK_RCONTROL) & 0x8000 != 0
                }
                VK_MENU => {
                    self.effective_key_state(VK_LMENU) & 0x8000 != 0
                        || self.effective_key_state(VK_RMENU) & 0x8000 != 0
                }
                _ => false,
            };
            if !still_down {
                if let Some(state) = self.key_state.get_mut(alias as usize) {
                    *state &= !0x8000;
                }
            }
        }
    }

    fn effective_key_state(&self, virtual_key: u32) -> u16 {
        let mut state = self
            .key_state
            .get(virtual_key as usize)
            .copied()
            .unwrap_or(0);
        match virtual_key {
            VK_SHIFT => {
                state |= self.key_state.get(VK_LSHIFT as usize).copied().unwrap_or(0) & 0x8000;
                state |= self.key_state.get(VK_RSHIFT as usize).copied().unwrap_or(0) & 0x8000;
            }
            VK_CONTROL => {
                state |= self
                    .key_state
                    .get(VK_LCONTROL as usize)
                    .copied()
                    .unwrap_or(0)
                    & 0x8000;
                state |= self
                    .key_state
                    .get(VK_RCONTROL as usize)
                    .copied()
                    .unwrap_or(0)
                    & 0x8000;
            }
            VK_MENU => {
                state |= self.key_state.get(VK_LMENU as usize).copied().unwrap_or(0) & 0x8000;
                state |= self.key_state.get(VK_RMENU as usize).copied().unwrap_or(0) & 0x8000;
            }
            _ => {}
        }
        state
    }

    fn consume_async_key_down(&mut self, virtual_key: u32) -> bool {
        let mut async_down = self
            .async_key_down
            .get_mut(virtual_key as usize)
            .map(|down| {
                let was_down = *down;
                *down = false;
                was_down
            })
            .unwrap_or(false);
        let aliases: &[u32] = match virtual_key {
            VK_SHIFT => &[VK_LSHIFT, VK_RSHIFT],
            VK_CONTROL => &[VK_LCONTROL, VK_RCONTROL],
            VK_MENU => &[VK_LMENU, VK_RMENU],
            _ => &[],
        };
        for alias in aliases {
            if let Some(down) = self.async_key_down.get_mut(*alias as usize) {
                async_down |= *down;
                *down = false;
            }
        }
        async_down
    }

    fn current_shift_flags(&self) -> u32 {
        let left_shift = self.effective_key_state(VK_LSHIFT) & 0x8000 != 0;
        let right_shift = self.effective_key_state(VK_RSHIFT) & 0x8000 != 0;
        let any_shift = self.effective_key_state(VK_SHIFT) & 0x8000 != 0;
        let left_ctrl = self.effective_key_state(VK_LCONTROL) & 0x8000 != 0;
        let right_ctrl = self.effective_key_state(VK_RCONTROL) & 0x8000 != 0;
        let any_ctrl = self.effective_key_state(VK_CONTROL) & 0x8000 != 0;
        let left_alt = self.effective_key_state(VK_LMENU) & 0x8000 != 0;
        let right_alt = self.effective_key_state(VK_RMENU) & 0x8000 != 0;
        let any_alt = self.effective_key_state(VK_MENU) & 0x8000 != 0;
        let capital = self.effective_key_state(VK_CAPITAL) & 0x0001 != 0;
        let mut flags = 0;
        if any_shift {
            flags |= KEY_SHIFT_ANY_SHIFT_FLAG;
        }
        if left_shift {
            flags |= KEY_SHIFT_LEFT_SHIFT_FLAG;
        }
        if right_shift {
            flags |= KEY_SHIFT_RIGHT_SHIFT_FLAG;
        }
        if any_ctrl {
            flags |= KEY_SHIFT_ANY_CTRL_FLAG;
        }
        if left_ctrl {
            flags |= KEY_SHIFT_LEFT_CTRL_FLAG;
        }
        if right_ctrl {
            flags |= KEY_SHIFT_RIGHT_CTRL_FLAG;
        }
        if any_alt {
            flags |= KEY_SHIFT_ANY_ALT_FLAG;
        }
        if left_alt {
            flags |= KEY_SHIFT_LEFT_ALT_FLAG;
        }
        if right_alt {
            flags |= KEY_SHIFT_RIGHT_ALT_FLAG;
        }
        if capital {
            flags |= KEY_SHIFT_CAPITAL_FLAG;
        }
        flags
    }

    pub fn record_destroy_lifecycle_message(&mut self, hwnd: u32, msg: u32) -> bool {
        let Some(window) = self.windows.get_mut(&hwnd) else {
            return false;
        };
        if window.destroyed {
            return false;
        }
        match msg {
            WM_DESTROY => {
                if !window.destroy_message_sent {
                    window.destroy_message_sent = true;
                    window.destroy_message_order = Some(self.next_lifecycle_message_order);
                    self.next_lifecycle_message_order =
                        self.next_lifecycle_message_order.saturating_add(1);
                }
                true
            }
            WM_NCDESTROY => {
                if !window.nc_destroy_message_sent {
                    window.nc_destroy_message_sent = true;
                    window.nc_destroy_message_order = Some(self.next_lifecycle_message_order);
                    self.next_lifecycle_message_order =
                        self.next_lifecycle_message_order.saturating_add(1);
                }
                true
            }
            _ => false,
        }
    }

    pub fn begin_send_message(&mut self, thread_id: u32) {
        *self.send_depth_by_thread.entry(thread_id).or_default() += 1;
    }

    pub fn end_send_message(&mut self, thread_id: u32) {
        let popped_send = self
            .active_sent_stack_by_thread
            .get_mut(&thread_id)
            .and_then(|stack| stack.pop());
        if self
            .active_sent_stack_by_thread
            .get(&thread_id)
            .is_some_and(|stack| stack.is_empty())
        {
            self.active_sent_stack_by_thread.remove(&thread_id);
        }
        if let Some(id) = popped_send {
            if self
                .sent_messages
                .get(&id)
                .is_some_and(|sent| sent.sender_thread_id.is_none())
            {
                self.sent_messages.remove(&id);
            }
        }
        let Some(depth) = self.send_depth_by_thread.get_mut(&thread_id) else {
            return;
        };
        *depth = depth.saturating_sub(1);
        if let Some(replied_depths) = self.replied_send_depth_by_thread.get_mut(&thread_id) {
            replied_depths.remove(&depth.saturating_add(1));
            if replied_depths.is_empty() {
                self.replied_send_depth_by_thread.remove(&thread_id);
            }
        }
        if *depth == 0 {
            self.send_depth_by_thread.remove(&thread_id);
        }
    }

    pub fn in_send_message(&self, thread_id: u32) -> bool {
        let depth = self
            .send_depth_by_thread
            .get(&thread_id)
            .copied()
            .unwrap_or(0);
        depth > 0
            && !self
                .replied_send_depth_by_thread
                .get(&thread_id)
                .is_some_and(|replied_depths| replied_depths.contains(&depth))
    }

    pub fn reply_message(&mut self, thread_id: u32, result: u32) -> Option<u64> {
        let depth = self
            .send_depth_by_thread
            .get(&thread_id)
            .copied()
            .unwrap_or(0);
        if depth == 0 {
            return None;
        }
        let id = self.active_sent_message_id(thread_id)?;
        let sent = self.sent_messages.get_mut(&id)?;
        if sent.sender_thread_id.is_none() || sent.flags & SMF_RESULT_READY != 0 {
            return None;
        }
        sent.flags |= SMF_RESULT_READY;
        sent.result = Some(result);
        self.replied_send_depth_by_thread
            .entry(thread_id)
            .or_default()
            .insert(depth);
        self.stats.send_transaction_completed_count = self
            .stats
            .send_transaction_completed_count
            .saturating_add(1);
        Some(id)
    }

    pub fn active_sent_message_id(&self, thread_id: u32) -> Option<u64> {
        self.active_sent_stack_by_thread
            .get(&thread_id)
            .and_then(|stack| stack.last())
            .copied()
    }

    pub fn complete_active_sent_message(&mut self, thread_id: u32, result: u32) -> Option<u64> {
        let id = self.active_sent_message_id(thread_id)?;
        if let Some(sent) = self.sent_messages.get_mut(&id) {
            if sent.flags & SMF_RESULT_READY == 0 {
                sent.flags |= SMF_RESULT_READY;
                sent.result = Some(result);
                self.stats.send_transaction_completed_count = self
                    .stats
                    .send_transaction_completed_count
                    .saturating_add(1);
            }
        }
        self.end_send_message(thread_id);
        Some(id)
    }

    pub fn activate_sent_message_for_receiver(&mut self, thread_id: u32, id: u64) -> bool {
        let Some(sent) = self.sent_messages.get(&id) else {
            return false;
        };
        if sent.receiver_thread_id != thread_id || sent.flags & SMF_RESULT_READY != 0 {
            return false;
        }
        let Some(queue) = self.sent_queues.get_mut(&thread_id) else {
            return false;
        };
        let Some(index) = queue.iter().position(|candidate| *candidate == id) else {
            return false;
        };
        queue.remove(index);
        self.last_message_source_by_thread
            .insert(thread_id, MSGSRC_SOFTWARE_SEND);
        self.active_sent_stack_by_thread
            .entry(thread_id)
            .or_default()
            .push(id);
        self.begin_send_message(thread_id);
        true
    }

    pub fn take_completed_sent_message_result(&mut self, id: u64) -> Option<u32> {
        if !self
            .sent_messages
            .get(&id)
            .is_some_and(|sent| sent.flags & SMF_RESULT_READY != 0)
        {
            return None;
        }
        self.sent_messages.remove(&id)?.result
    }

    pub fn sent_message(&self, id: u64) -> Option<&SentMessage> {
        self.sent_messages.get(&id)
    }

    pub fn sent_message_result_ready(&self, id: u64) -> bool {
        self.sent_messages
            .get(&id)
            .is_some_and(|sent| sent.flags & SMF_RESULT_READY != 0)
    }

    pub fn sent_message_ids_for_windows(&self, hwnds: &[u32]) -> Vec<u64> {
        self.sent_messages
            .iter()
            .filter(|(_, sent)| sent.message.hwnd != 0 && hwnds.contains(&sent.message.hwnd))
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn expire_timed_out_sent_messages(&mut self, now_ms: u32) -> Vec<u64> {
        let expired: Vec<u64> = self
            .sent_messages
            .iter()
            .filter(|(_, sent)| sent.flags & SMF_RESULT_READY == 0)
            .filter(|(_, sent)| {
                sent.timeout_ms
                    .is_some_and(|timeout| now_ms.wrapping_sub(sent.message.time_ms) >= timeout)
            })
            .map(|(id, _)| *id)
            .collect();
        if expired.is_empty() {
            return expired;
        }

        for id in &expired {
            if let Some(sent) = self.sent_messages.get_mut(id) {
                sent.flags |= SMF_TIMEOUT | SMF_RESULT_READY;
                sent.result = Some(0);
                self.stats.send_transaction_timeout_count =
                    self.stats.send_transaction_timeout_count.saturating_add(1);
            }
        }
        for queue in self.sent_queues.values_mut() {
            queue.retain(|id| !expired.contains(id));
        }
        expired
    }

    pub fn stats(&self) -> GweStats {
        self.stats
    }

    pub fn get_message_source(&self, thread_id: u32) -> u32 {
        self.last_message_source_by_thread
            .get(&thread_id)
            .copied()
            .unwrap_or(MSGSRC_UNKNOWN)
    }

    pub fn get_message_pos(&self, thread_id: u32) -> u32 {
        self.last_message_pos_by_thread
            .get(&thread_id)
            .copied()
            .unwrap_or(0)
    }

    pub fn get_message_queue_ready_time_stamp(&self, thread_id: u32, hwnd: u32) -> u32 {
        let queue_thread = if hwnd != 0 {
            self.window(hwnd)
                .filter(|window| !window.destroyed)
                .map(|window| window.thread_id)
                .unwrap_or(thread_id)
        } else {
            thread_id
        };
        self.ready_timestamp_by_thread
            .get(&queue_thread)
            .copied()
            .unwrap_or(0)
    }

    pub fn get_queue_status(&mut self, thread_id: u32, flags: u32) -> u32 {
        let current = self.queue_status_bits(thread_id) & flags;
        let changed_for_flags = self
            .changed_queue_status_by_thread
            .get(&thread_id)
            .copied()
            .unwrap_or(0)
            & flags;
        if let Some(changed) = self.changed_queue_status_by_thread.get_mut(&thread_id) {
            *changed &= !flags;
            if *changed == 0 {
                self.changed_queue_status_by_thread.remove(&thread_id);
            }
        }
        (current << 16) | changed_for_flags
    }

    pub fn peek_queue_status(&self, thread_id: u32, flags: u32) -> u32 {
        let current = self.queue_status_bits(thread_id) & flags;
        let changed = self
            .changed_queue_status_by_thread
            .get(&thread_id)
            .copied()
            .unwrap_or(0)
            & flags;
        (current << 16) | changed
    }

    pub fn windows_snapshot(&self) -> Vec<Window> {
        self.windows.values().cloned().collect()
    }

    pub fn queue_snapshot(&self) -> Vec<(u32, Vec<Message>)> {
        self.queues
            .iter()
            .map(|(thread_id, queue)| (*thread_id, queue.iter().cloned().collect()))
            .collect()
    }

    pub fn sent_queue_snapshot(&self) -> Vec<(u32, Vec<Message>)> {
        self.sent_queues
            .iter()
            .map(|(thread_id, queue)| {
                (
                    *thread_id,
                    queue
                        .iter()
                        .filter_map(|id| self.sent_messages.get(id))
                        .map(|sent| sent.message.clone())
                        .collect(),
                )
            })
            .collect()
    }

    pub fn z_order_snapshot(&self) -> Vec<u32> {
        self.z_order.clone()
    }

    pub fn window_from_point(&self, point: Point) -> Option<u32> {
        self.window_from_point_in_parent(None, None, point)
    }

    pub fn window_from_point_for_thread(&self, thread_id: u32, point: Point) -> Option<u32> {
        self.window_from_point_in_parent(Some(thread_id), None, point)
    }

    pub fn child_window_from_point_for_thread(
        &self,
        _thread_id: u32,
        parent: u32,
        point: Point,
    ) -> Option<u32> {
        let screen_point = self.client_to_screen(parent, point)?;
        let parent_window = self.windows.get(&parent)?;
        if !parent_window.client_rect.contains_point(screen_point) {
            return None;
        }
        for hwnd in self.sibling_windows(Some(parent)) {
            let Some(window) = self.windows.get(&hwnd) else {
                continue;
            };
            if window.client_rect.contains_point(screen_point) {
                return Some(hwnd);
            }
        }
        Some(parent)
    }

    pub fn has_queue_input(&self, thread_id: u32, flags: u32) -> bool {
        self.queue_status_bits(thread_id) & flags != 0
    }

    pub fn has_new_queue_input(&self, thread_id: u32, flags: u32) -> bool {
        self.changed_queue_status_by_thread
            .get(&thread_id)
            .copied()
            .unwrap_or(0)
            & flags
            != 0
    }

    pub fn clear_new_queue_input(&mut self, thread_id: u32, flags: u32) {
        let Some(changed) = self.changed_queue_status_by_thread.get_mut(&thread_id) else {
            return;
        };
        *changed &= !flags;
        if *changed == 0 {
            self.changed_queue_status_by_thread.remove(&thread_id);
        }
    }

    pub fn get_message(&mut self, thread_id: u32) -> Option<Message> {
        self.get_message_filtered(thread_id, None, 0, 0)
    }

    pub fn peek_message(&mut self, thread_id: u32, flags: PeekFlags) -> Option<Message> {
        self.peek_message_filtered(thread_id, None, 0, 0, flags)
    }

    pub fn has_message_filtered(
        &self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
    ) -> bool {
        self.sent_queues.get(&thread_id).is_some_and(|queue| {
            queue.iter().any(|id| {
                self.sent_messages
                    .get(id)
                    .is_some_and(|sent| message_matches(&sent.message, hwnd, min_msg, max_msg))
            })
        }) || self.queues.get(&thread_id).is_some_and(|queue| {
            queue
                .iter()
                .any(|message| message_matches(message, hwnd, min_msg, max_msg))
        }) || self.quit_message(thread_id).is_some()
            || self
                .synthetic_paint_message(thread_id, hwnd, min_msg, max_msg)
                .is_some()
    }

    pub fn post_quit_message(&mut self, thread_id: u32, exit_code: u32, time_ms: u32) {
        self.quit_by_thread
            .insert(thread_id, QuitState { exit_code, time_ms });
        self.ready_timestamp_by_thread.insert(thread_id, time_ms);
        self.mark_queue_status_changed(thread_id, QS_POSTMESSAGE);
    }

    pub fn get_message_filtered(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
    ) -> Option<Message> {
        if let Some(message) = self.take_matching_message(thread_id, hwnd, min_msg, max_msg) {
            return Some(message);
        }
        if let Some(message) = self.take_matching_sent_message(thread_id, hwnd, min_msg, max_msg) {
            return Some(message);
        }
        if let Some(message) = self.take_quit_message(thread_id) {
            return Some(message);
        }
        let message = self.synthetic_paint_message(thread_id, hwnd, min_msg, max_msg)?;
        self.last_message_source_by_thread
            .insert(thread_id, message.source);
        self.record_last_message_pos(thread_id, &message);
        Some(message)
    }

    pub fn take_sent_message_filtered(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
    ) -> Option<Message> {
        self.take_matching_sent_message(thread_id, hwnd, min_msg, max_msg)
    }

    pub fn peek_message_filtered(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
        flags: PeekFlags,
    ) -> Option<Message> {
        if let Some(queue) = self.queues.get_mut(&thread_id) {
            if let Some(index) = queue
                .iter()
                .position(|message| message_matches(message, hwnd, min_msg, max_msg))
            {
                return if flags.contains(PeekFlags::REMOVE) {
                    let message = queue.remove(index);
                    if let Some(message) = message.as_ref() {
                        let source = message.source;
                        self.last_message_source_by_thread.insert(thread_id, source);
                        self.record_last_message_pos(thread_id, message);
                    }
                    message
                } else {
                    queue.get(index).cloned()
                };
            }
        }
        if let Some(message) =
            self.peek_matching_sent_message(thread_id, hwnd, min_msg, max_msg, flags)
        {
            return Some(message);
        }
        if flags.contains(PeekFlags::REMOVE) {
            if let Some(message) = self.take_quit_message(thread_id) {
                return Some(message);
            }
        } else if let Some(message) = self.quit_message(thread_id) {
            return Some(message);
        }
        self.synthetic_paint_message(thread_id, hwnd, min_msg, max_msg)
    }

    pub fn set_window_text(&mut self, hwnd: u32, title: &str) -> bool {
        let Some(window) = self.windows.get_mut(&hwnd) else {
            return false;
        };
        if window.destroyed {
            return false;
        }
        window.title = title.to_owned();
        true
    }

    pub fn get_window_text(&self, hwnd: u32, capacity_chars: usize) -> Option<String> {
        let window = self.windows.get(&hwnd)?;
        if window.destroyed {
            return None;
        }
        let title = &window.title;
        Some(
            title
                .chars()
                .take(capacity_chars.saturating_sub(1))
                .collect(),
        )
    }

    pub fn get_window_text_length(&self, hwnd: u32) -> Option<usize> {
        Some(self.windows.get(&hwnd)?.title.encode_utf16().count())
    }

    pub fn get_class_name(&self, hwnd: u32, capacity_chars: usize) -> Option<String> {
        let class_name = &self.windows.get(&hwnd)?.class_name;
        Some(
            class_name
                .chars()
                .take(capacity_chars.saturating_sub(1))
                .collect(),
        )
    }

    pub fn set_window_long(&mut self, hwnd: u32, index: i32, value: u32) -> Option<u32> {
        let window = self.windows.get_mut(&hwnd)?;
        let slot = window_long_slot_mut(window, index)?;
        let previous = *slot;
        *slot = value;
        if index == GWL_STYLE {
            window.enabled = value & WS_DISABLED == 0;
            window.visible = value & WS_VISIBLE != 0;
        }
        Some(previous)
    }

    pub fn get_window_long(&self, hwnd: u32, index: i32) -> Option<u32> {
        let window = self.windows.get(&hwnd)?;
        window_long_slot(window, index).copied()
    }

    pub fn is_window(&self, hwnd: u32) -> bool {
        self.windows
            .get(&hwnd)
            .is_some_and(|window| !window.destroyed)
    }

    pub fn window(&self, hwnd: u32) -> Option<&Window> {
        self.windows.get(&hwnd)
    }

    pub fn window_thread_process_id(&self, hwnd: u32) -> Option<(u32, u32)> {
        let window = self.windows.get(&hwnd)?;
        (!window.destroyed).then_some((window.thread_id, window.process_id))
    }

    pub fn window_and_descendants(&self, hwnd: u32) -> Option<Vec<u32>> {
        if !self.is_window(hwnd) {
            return None;
        }
        let mut targets = vec![hwnd];
        let mut index = 0;
        while let Some(ancestor) = targets.get(index).copied() {
            index += 1;
            let descendants: Vec<u32> = self
                .windows
                .values()
                .filter(|window| {
                    !window.destroyed
                        && (window.parent == Some(ancestor) || window.owner == Some(ancestor))
                        && !targets.contains(&window.hwnd)
                })
                .map(|window| window.hwnd)
                .collect();
            targets.extend(descendants);
        }
        Some(targets)
    }

    fn take_matching_message(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
    ) -> Option<Message> {
        let queue = self.queues.get_mut(&thread_id)?;
        let index = queue
            .iter()
            .position(|message| message_matches(message, hwnd, min_msg, max_msg))?;
        let message = queue.remove(index)?;
        self.last_message_source_by_thread
            .insert(thread_id, message.source);
        self.record_last_message_pos(thread_id, &message);
        Some(message)
    }

    fn take_matching_sent_message(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
    ) -> Option<Message> {
        let queue = self.sent_queues.get_mut(&thread_id)?;
        let index = queue.iter().position(|id| {
            self.sent_messages
                .get(id)
                .is_some_and(|sent| message_matches(&sent.message, hwnd, min_msg, max_msg))
        })?;
        let id = queue.remove(index)?;
        let message = self.sent_messages.get(&id)?.message.clone();
        self.last_message_source_by_thread
            .insert(thread_id, message.source);
        self.record_last_message_pos(thread_id, &message);
        self.active_sent_stack_by_thread
            .entry(thread_id)
            .or_default()
            .push(id);
        self.begin_send_message(thread_id);
        Some(message)
    }

    fn peek_matching_sent_message(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
        flags: PeekFlags,
    ) -> Option<Message> {
        let queue = self.sent_queues.get_mut(&thread_id)?;
        let index = queue.iter().position(|id| {
            self.sent_messages
                .get(id)
                .is_some_and(|sent| message_matches(&sent.message, hwnd, min_msg, max_msg))
        })?;
        if flags.contains(PeekFlags::REMOVE) {
            let id = queue.remove(index)?;
            let message = self.sent_messages.get(&id)?.message.clone();
            self.last_message_source_by_thread
                .insert(thread_id, message.source);
            self.record_last_message_pos(thread_id, &message);
            self.active_sent_stack_by_thread
                .entry(thread_id)
                .or_default()
                .push(id);
            self.begin_send_message(thread_id);
            Some(message)
        } else {
            queue
                .get(index)
                .and_then(|id| self.sent_messages.get(id))
                .map(|sent| sent.message.clone())
        }
    }

    fn synthetic_paint_message(
        &self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
    ) -> Option<Message> {
        if !message_id_matches(WM_PAINT, min_msg, max_msg) {
            return None;
        }
        self.z_order.iter().find_map(|candidate| {
            let window = self.windows.get(candidate)?;
            (!window.destroyed
                && window.thread_id == thread_id
                && self.is_window_visible(window.hwnd)
                && window.update_pending
                && hwnd.is_none_or(|wanted| window.hwnd == wanted))
            .then(|| Message::new(window.hwnd, WM_PAINT, 0, 0, 0))
        })
    }

    fn queue_status_bits(&self, thread_id: u32) -> u32 {
        let mut status = 0;
        if self.in_send_message(thread_id)
            || self
                .sent_queues
                .get(&thread_id)
                .is_some_and(|queue| !queue.is_empty())
        {
            status |= QS_SENDMESSAGE;
        }
        if let Some(queue) = self.queues.get(&thread_id) {
            for message in queue {
                status |= queue_status_bit_for_message(message.msg);
            }
        }
        if self.quit_by_thread.contains_key(&thread_id) {
            status |= QS_POSTMESSAGE;
        }
        if self.windows.values().any(|window| {
            !window.destroyed
                && window.thread_id == thread_id
                && self.is_window_visible(window.hwnd)
                && window.update_pending
        }) {
            status |= QS_PAINT;
        }
        status
    }

    fn take_quit_message(&mut self, thread_id: u32) -> Option<Message> {
        let message = self.quit_message(thread_id)?;
        self.quit_by_thread.remove(&thread_id);
        self.last_message_source_by_thread
            .insert(thread_id, message.source);
        self.record_last_message_pos(thread_id, &message);
        Some(message)
    }

    fn quit_message(&self, thread_id: u32) -> Option<Message> {
        let state = self.quit_by_thread.get(&thread_id)?;
        Some(
            Message::new(0, WM_QUIT, state.exit_code, 0, state.time_ms)
                .with_source(MSGSRC_SOFTWARE_POST),
        )
    }

    fn record_last_message_pos(&mut self, thread_id: u32, message: &Message) {
        let pos = message
            .mouse_pos_at_post
            .unwrap_or_else(|| point_to_lparam(self.cursor_pos));
        self.last_message_pos_by_thread.insert(thread_id, pos);
    }

    fn mark_queue_status_changed(&mut self, thread_id: u32, bits: u32) {
        if bits == 0 {
            return;
        }
        *self
            .changed_queue_status_by_thread
            .entry(thread_id)
            .or_default() |= bits;
    }

    fn parent_to_screen_rect(&self, parent: Option<u32>, rect: Rect) -> Rect {
        let origin = self.parent_client_origin(parent);
        rect.offset(origin.x, origin.y)
    }

    fn parent_client_origin(&self, parent: Option<u32>) -> Point {
        parent
            .and_then(|hwnd| self.windows.get(&hwnd))
            .map(|window| Point {
                x: window.client_rect.left,
                y: window.client_rect.top,
            })
            .unwrap_or_default()
    }

    fn desktop_rect(&self) -> Rect {
        self.windows
            .get(&DESKTOP_HWND)
            .map(|window| window.client_rect)
            .unwrap_or_else(|| Rect::from_origin_size(0, 0, 800, 480))
    }

    fn normalize_create_rect(&self, parent: Option<u32>, style: u32, rect: Rect) -> Rect {
        let mut rect = self.parent_to_screen_rect(parent, rect);
        let visible_child =
            parent.is_some() && style & (WS_VISIBLE | WS_CHILD) == (WS_VISIBLE | WS_CHILD);
        if visible_child && (rect.width() <= 0 || rect.height() <= 0) {
            if let Some(parent_rect) = parent.and_then(|hwnd| self.windows.get(&hwnd)) {
                if rect.width() <= 0 {
                    rect.right = rect.left.saturating_add(parent_rect.client_rect.width());
                }
                if rect.height() <= 0 {
                    rect.bottom = rect.top.saturating_add(parent_rect.client_rect.height());
                }
            }
        }
        let visible_top_level = parent.is_none() && style & (WS_VISIBLE | WS_CHILD) == WS_VISIBLE;
        if !visible_top_level {
            return rect;
        }

        let desktop = self.desktop_rect();
        if rect.left == CW_USEDEFAULT || rect.top == CW_USEDEFAULT {
            let width = rect.width();
            let height = rect.height();
            rect.left = desktop.left;
            rect.top = desktop.top;
            rect.right = rect.left.saturating_add(width);
            rect.bottom = rect.top.saturating_add(height);
        }
        if rect.width() <= 0 || rect.right == CW_USEDEFAULT {
            rect.right = rect.left.saturating_add(desktop.width());
        }
        if rect.height() <= 0 || rect.bottom == CW_USEDEFAULT {
            rect.bottom = rect.top.saturating_add(desktop.height());
        }
        rect
    }

    fn first_child(&self, hwnd: u32) -> Option<u32> {
        let parent = if hwnd == DESKTOP_HWND {
            None
        } else {
            Some(hwnd)
        };
        self.sibling_windows(parent).first().copied()
    }

    fn dialog_child_candidates(&self, dialog: u32, required_style: u32) -> Vec<u32> {
        if !self.is_window(dialog) {
            return Vec::new();
        }
        self.sibling_windows(Some(dialog))
            .into_iter()
            .filter(|hwnd| {
                self.windows.get(hwnd).is_some_and(|window| {
                    self.is_window_visible(*hwnd)
                        && self.is_window_enabled(*hwnd)
                        && (required_style == 0 || window.style & required_style != 0)
                })
            })
            .collect()
    }

    fn next_dialog_candidate(
        &self,
        candidates: &[u32],
        control: u32,
        previous: bool,
    ) -> Option<u32> {
        if candidates.is_empty() {
            return None;
        }
        if control == 0 {
            return if previous {
                candidates.last().copied()
            } else {
                candidates.first().copied()
            };
        }
        let index = candidates
            .iter()
            .position(|candidate| *candidate == control)
            .unwrap_or(if previous { 0 } else { candidates.len() - 1 });
        let next_index = if previous {
            index.checked_sub(1).unwrap_or(candidates.len() - 1)
        } else {
            (index + 1) % candidates.len()
        };
        candidates.get(next_index).copied()
    }

    fn window_has_style(&self, hwnd: u32, style: u32) -> bool {
        self.windows
            .get(&hwnd)
            .is_some_and(|window| !window.destroyed && window.style & style != 0)
    }

    fn visible_client_screen_rects_for_paint(&self, hwnd: u32) -> Vec<Rect> {
        self.visible_client_screen_rects(hwnd, true)
    }

    fn visible_client_screen_rects_for_ancestor_clip(&self, hwnd: u32) -> Vec<Rect> {
        self.visible_client_screen_rects(hwnd, false)
    }

    fn visible_client_screen_rects(&self, hwnd: u32, clip_own_children: bool) -> Vec<Rect> {
        let Some(window) = self.windows.get(&hwnd) else {
            return Vec::new();
        };
        if window.destroyed || (hwnd != DESKTOP_HWND && !self.is_window_visible(hwnd)) {
            return Vec::new();
        }

        let mut visible = vec![window.client_rect.normalized()];
        if hwnd == DESKTOP_HWND {
            return visible;
        }
        if let Some(parent) = window.parent {
            visible = intersect_rect_lists(
                &visible,
                &self.visible_client_screen_rects_for_ancestor_clip(parent),
            );
        }
        if let Some(region) = self.window_regions.get(&hwnd) {
            let region_rects: Vec<Rect> = region
                .rects
                .iter()
                .map(|rect| rect.offset(window.client_rect.left, window.client_rect.top))
                .collect();
            visible = intersect_rect_lists(&visible, &region_rects);
        }

        let clip_siblings = window.parent.is_none() || window.style & WS_CLIPSIBLINGS != 0;
        if clip_siblings {
            let siblings = self.sibling_windows(window.parent);
            for sibling in siblings {
                if sibling == hwnd {
                    break;
                }
                if !self.is_window_visible(sibling) {
                    continue;
                }
                let sibling_rects = self.visible_client_screen_rects_for_paint(sibling);
                visible = subtract_rect_lists(&visible, &sibling_rects);
                if visible.is_empty() {
                    break;
                }
            }
        }

        if clip_own_children && window.style & WS_CLIPCHILDREN != 0 {
            for child in self.sibling_windows(Some(hwnd)) {
                if !self.is_window_visible(child) {
                    continue;
                }
                let child_rects = self.visible_client_screen_rects_for_paint(child);
                visible = subtract_rect_lists(&visible, &child_rects);
                if visible.is_empty() {
                    break;
                }
            }
        }
        visible
    }

    fn sibling_windows(&self, parent: Option<u32>) -> Vec<u32> {
        self.z_order
            .iter()
            .filter_map(|hwnd| self.windows.get(hwnd))
            .filter(|window| {
                !window.destroyed
                    && window.parent == parent
                    && (parent.is_some() || window.hwnd != DESKTOP_HWND)
            })
            .map(|window| window.hwnd)
            .collect()
    }

    fn window_from_point_in_parent(
        &self,
        thread_id: Option<u32>,
        parent: Option<u32>,
        point: Point,
    ) -> Option<u32> {
        for hwnd in self.sibling_windows(parent) {
            let Some(window) = self.windows.get(&hwnd) else {
                continue;
            };
            if !self.is_window_visible(hwnd)
                || !self.is_window_enabled(hwnd)
                || !window.client_rect.contains_point(point)
            {
                continue;
            }
            if let Some(child) = self.window_from_point_in_parent(thread_id, Some(hwnd), point) {
                return Some(child);
            }
            if thread_id.is_none_or(|thread_id| window.thread_id == thread_id) {
                return Some(hwnd);
            }
        }
        None
    }

    fn apply_z_order(&mut self, hwnd: u32, parent: Option<u32>, insert_after: u32) {
        self.z_order.retain(|candidate| *candidate != hwnd);
        let siblings = self.sibling_windows(parent);
        let index = match insert_after {
            HWND_TOP | HWND_TOPMOST | HWND_NOTOPMOST => siblings
                .first()
                .and_then(|sibling| {
                    self.z_order
                        .iter()
                        .position(|candidate| candidate == sibling)
                })
                .unwrap_or(self.z_order.len()),
            HWND_BOTTOM => siblings
                .last()
                .and_then(|sibling| {
                    self.z_order
                        .iter()
                        .position(|candidate| candidate == sibling)
                })
                .map(|index| index + 1)
                .unwrap_or(self.z_order.len()),
            sibling if siblings.contains(&sibling) => self
                .z_order
                .iter()
                .position(|candidate| *candidate == sibling)
                .map(|index| index + 1)
                .unwrap_or(self.z_order.len()),
            _ => self.z_order.len(),
        };
        self.z_order.insert(index.min(self.z_order.len()), hwnd);
    }
}

fn intersect_rect_lists(lhs: &[Rect], rhs: &[Rect]) -> Vec<Rect> {
    canonicalize_region_rects(
        lhs.iter()
            .flat_map(|left| rhs.iter().filter_map(|right| left.intersect(*right)))
            .filter(|rect| !rect.is_empty())
            .collect(),
    )
}

fn subtract_rect_lists(lhs: &[Rect], rhs: &[Rect]) -> Vec<Rect> {
    let mut remaining: Vec<Rect> = lhs
        .iter()
        .copied()
        .filter(|rect| !rect.is_empty())
        .collect();
    for cut in rhs {
        remaining = remaining
            .into_iter()
            .flat_map(|rect| rect.subtract(*cut))
            .filter(|rect| !rect.is_empty())
            .collect();
        if remaining.is_empty() {
            break;
        }
    }
    canonicalize_region_rects(remaining)
}

fn normalize_class_name(name_or_atom: &str) -> String {
    if name_or_atom.is_empty() {
        return "#anonymous".to_owned();
    }
    if parse_atom_class_name(name_or_atom).is_some() {
        return name_or_atom.to_owned();
    }
    name_or_atom.to_ascii_lowercase()
}

fn parse_atom_class_name(name_or_atom: &str) -> Option<u16> {
    name_or_atom
        .strip_prefix('#')
        .and_then(|atom| atom.parse::<u16>().ok())
}

impl Message {
    pub fn new(hwnd: u32, msg: u32, wparam: u32, lparam: u32, time_ms: u32) -> Self {
        Self {
            hwnd,
            msg,
            wparam,
            lparam,
            time_ms,
            source: MSGSRC_UNKNOWN,
            mouse_pos_at_post: None,
        }
    }

    pub fn with_source(mut self, source: u32) -> Self {
        self.source = source;
        self
    }

    pub fn with_mouse_pos(mut self, mouse_pos: u32) -> Self {
        self.mouse_pos_at_post = Some(mouse_pos);
        self
    }
}

pub fn default_send_message_result(msg: u32, _wparam: u32, _lparam: u32) -> u32 {
    match msg {
        WM_NULL => 0,
        WM_GETTEXTLENGTH => 0,
        WM_ERASEBKGND => 1,
        _ => 0,
    }
}

fn queue_status_bit_for_message(msg: u32) -> u32 {
    match msg {
        WM_TIMER => QS_TIMER,
        WM_PAINT => QS_PAINT,
        WM_KEYDOWN | WM_KEYUP | WM_CHAR => QS_KEY,
        WM_LBUTTONDOWN | WM_LBUTTONUP => QS_MOUSEBUTTON,
        WM_MOUSEMOVE => QS_MOUSEMOVE,
        _ => QS_POSTMESSAGE,
    }
}

fn modifier_alias_for(virtual_key: u32) -> Option<u32> {
    match virtual_key {
        VK_LSHIFT | VK_RSHIFT => Some(VK_SHIFT),
        VK_LCONTROL | VK_RCONTROL => Some(VK_CONTROL),
        VK_LMENU | VK_RMENU => Some(VK_MENU),
        _ => None,
    }
}

fn is_mouse_message(msg: u32) -> bool {
    matches!(msg, WM_MOUSEMOVE | WM_LBUTTONDOWN | WM_LBUTTONUP)
}

fn point_to_lparam(point: Point) -> u32 {
    ((point.y as u16 as u32) << 16) | (point.x as u16 as u32)
}

fn message_matches(message: &Message, hwnd: Option<u32>, min_msg: u32, max_msg: u32) -> bool {
    if hwnd.is_some_and(|wanted| message.hwnd != wanted) {
        return false;
    }
    if min_msg == 0 && max_msg == 0 {
        return true;
    }
    message_id_matches(message.msg, min_msg, max_msg)
}

fn message_id_matches(msg: u32, min_msg: u32, max_msg: u32) -> bool {
    if min_msg == 0 && max_msg == 0 {
        return true;
    }
    (min_msg..=max_msg).contains(&msg)
}

fn window_long_slot(window: &Window, index: i32) -> Option<&u32> {
    match index {
        GWL_ID => Some(&window.id),
        GWL_STYLE => Some(&window.style),
        GWL_EXSTYLE => Some(&window.ex_style),
        GWL_WNDPROC => Some(&window.wndproc),
        GWL_USERDATA => Some(&window.user_data),
        _ if index >= 0 && index % 4 == 0 => window.extra_longs.get((index / 4) as usize),
        _ => None,
    }
}

fn window_long_slot_mut(window: &mut Window, index: i32) -> Option<&mut u32> {
    match index {
        GWL_ID => Some(&mut window.id),
        GWL_STYLE => Some(&mut window.style),
        GWL_EXSTYLE => Some(&mut window.ex_style),
        GWL_WNDPROC => Some(&mut window.wndproc),
        GWL_USERDATA => Some(&mut window.user_data),
        _ if index >= 0 && index % 4 == 0 => window.extra_longs.get_mut((index / 4) as usize),
        _ => None,
    }
}

fn window_extra_long_count(class: &WindowClass) -> usize {
    let bytes = i32::from_le_bytes([
        class.bytes[12],
        class.bytes[13],
        class.bytes[14],
        class.bytes[15],
    ]);
    if bytes <= 0 {
        0
    } else {
        ((bytes as usize) + 3) / 4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_window_records_window_state() {
        let mut gwe = Gwe::default();
        let hwnd = gwe.create_window(1, "STATIC", "ready");
        let window = gwe.window(hwnd).unwrap();

        assert_eq!(window.hwnd, hwnd);
        assert_eq!(window.thread_id, 1);
        assert_eq!(window.class_name, "static");
        assert_eq!(window.title, "ready");
        assert!(gwe.get_message(1).is_none());
    }

    #[test]
    fn visible_top_level_default_size_uses_desktop_rect() {
        let mut gwe = Gwe::default();
        let zero = gwe.create_window_ex_with_rect(
            1,
            "STATIC",
            "zero",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::default(),
        );
        assert_eq!(
            gwe.get_window_rect(zero).unwrap(),
            Rect::from_origin_size(0, 0, 800, 480)
        );

        let default = gwe.create_window_ex_with_rect(
            1,
            "STATIC",
            "default",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, CW_USEDEFAULT, CW_USEDEFAULT),
        );
        assert_eq!(
            gwe.get_window_rect(default).unwrap(),
            Rect::from_origin_size(0, 0, 800, 480)
        );
    }

    #[test]
    fn hidden_zero_size_windows_keep_requested_rect_and_visible_children_fill_parent() {
        let mut gwe = Gwe::default();
        let hidden =
            gwe.create_window_ex_with_rect(1, "STATIC", "hidden", None, 0, 0, 0, Rect::default());
        assert_eq!(gwe.get_window_rect(hidden).unwrap(), Rect::default());

        let parent = gwe.create_window_ex_with_rect(
            1,
            "PARENT",
            "parent",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(10, 20, 300, 200),
        );
        let child = gwe.create_window_ex_with_rect(
            1,
            "CHILD",
            "child",
            Some(parent),
            0,
            WS_VISIBLE | WS_CHILD,
            0,
            Rect::default(),
        );
        assert_eq!(
            gwe.get_window_rect(child).unwrap(),
            Rect::from_origin_size(10, 20, 300, 200)
        );
    }

    #[test]
    fn window_from_point_prefers_visible_child_over_parent() {
        let mut gwe = Gwe::default();
        let parent = gwe.create_window_ex_with_rect(
            1,
            "PARENT",
            "parent",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, 800, 480),
        );
        let child = gwe.create_window_ex_with_rect(
            1,
            "CHILD",
            "child",
            Some(parent),
            0,
            WS_VISIBLE | WS_CHILD,
            0,
            Rect::from_origin_size(0, 0, 800, 480),
        );

        assert_eq!(
            gwe.window_from_point_for_thread(1, Point { x: 400, y: 240 }),
            Some(child)
        );
        assert_eq!(
            gwe.window_from_point_for_thread(2, Point { x: 400, y: 240 }),
            None
        );
    }

    #[test]
    fn window_from_point_prefers_newer_overlapping_sibling() {
        let mut gwe = Gwe::default();
        let first = gwe.create_window_ex_with_rect(
            1,
            "FIRST",
            "first",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, 800, 480),
        );
        let second = gwe.create_window_ex_with_rect(
            1,
            "SECOND",
            "second",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, 800, 480),
        );

        assert_eq!(
            gwe.window_from_point_for_thread(1, Point { x: 400, y: 240 }),
            Some(second)
        );
        assert_eq!(gwe.get_window(first, GW_HWNDFIRST), Some(second));
        assert_eq!(gwe.get_window(second, GW_HWNDNEXT), Some(first));
    }

    #[test]
    fn window_from_point_skips_hidden_top_level_above_visible_window() {
        let mut gwe = Gwe::default();
        let visible = gwe.create_window_ex_with_rect(
            1,
            "VISIBLE",
            "visible",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, 800, 480),
        );
        let hidden = gwe.create_window_ex_with_rect(
            2,
            "HIDDEN",
            "hidden",
            None,
            0,
            0,
            0,
            Rect::from_origin_size(0, 0, 800, 480),
        );

        assert_eq!(gwe.get_window(visible, GW_HWNDFIRST), Some(hidden));
        assert_eq!(
            gwe.window_from_point(Point { x: 768, y: 88 }),
            Some(visible)
        );
        assert_eq!(
            gwe.window_from_point_for_thread(2, Point { x: 768, y: 88 }),
            None
        );
    }

    #[test]
    fn clip_children_excludes_visible_children_from_parent_paint() {
        let mut gwe = Gwe::default();
        let parent = gwe.create_window_ex_with_rect(
            1,
            "PARENT",
            "parent",
            None,
            0,
            WS_VISIBLE | WS_CLIPCHILDREN,
            0,
            Rect::from_origin_size(0, 0, 800, 480),
        );
        let _bottom_bar = gwe.create_window_ex_with_rect(
            1,
            "BAR",
            "bar",
            Some(parent),
            0,
            WS_VISIBLE | WS_CHILD,
            0,
            Rect::from_origin_size(0, 426, 800, 54),
        );

        assert_eq!(
            gwe.visible_client_rects(parent),
            vec![Rect::from_origin_size(0, 0, 800, 426)]
        );
    }

    #[test]
    fn parent_without_clip_children_can_paint_across_child_rects() {
        let mut gwe = Gwe::default();
        let parent = gwe.create_window_ex_with_rect(
            1,
            "PARENT",
            "parent",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, 800, 480),
        );
        let _bottom_bar = gwe.create_window_ex_with_rect(
            1,
            "BAR",
            "bar",
            Some(parent),
            0,
            WS_VISIBLE | WS_CHILD,
            0,
            Rect::from_origin_size(0, 426, 800, 54),
        );

        assert_eq!(
            gwe.visible_client_rects(parent),
            vec![Rect::from_origin_size(0, 0, 800, 480)]
        );
    }

    #[test]
    fn clip_siblings_excludes_higher_z_order_sibling_from_child_paint() {
        let mut gwe = Gwe::default();
        let parent = gwe.create_window_ex_with_rect(
            1,
            "PARENT",
            "parent",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, 200, 100),
        );
        let back = gwe.create_window_ex_with_rect(
            1,
            "BACK",
            "back",
            Some(parent),
            0,
            WS_VISIBLE | WS_CHILD | WS_CLIPSIBLINGS,
            0,
            Rect::from_origin_size(0, 0, 100, 100),
        );
        let _front = gwe.create_window_ex_with_rect(
            1,
            "FRONT",
            "front",
            Some(parent),
            0,
            WS_VISIBLE | WS_CHILD,
            0,
            Rect::from_origin_size(50, 0, 50, 100),
        );
        assert!(gwe.set_window_pos(_front, None, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE));

        assert_eq!(
            gwe.visible_client_rects(back),
            vec![Rect::from_origin_size(0, 0, 50, 100)]
        );
    }

    #[test]
    fn child_without_clip_siblings_can_paint_across_higher_siblings() {
        let mut gwe = Gwe::default();
        let parent = gwe.create_window_ex_with_rect(
            1,
            "PARENT",
            "parent",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, 200, 100),
        );
        let back = gwe.create_window_ex_with_rect(
            1,
            "BACK",
            "back",
            Some(parent),
            0,
            WS_VISIBLE | WS_CHILD,
            0,
            Rect::from_origin_size(0, 0, 100, 100),
        );
        let _front = gwe.create_window_ex_with_rect(
            1,
            "FRONT",
            "front",
            Some(parent),
            0,
            WS_VISIBLE | WS_CHILD,
            0,
            Rect::from_origin_size(50, 0, 50, 100),
        );

        assert_eq!(
            gwe.visible_client_rects(back),
            vec![Rect::from_origin_size(0, 0, 100, 100)]
        );
    }

    #[test]
    fn posted_messages_are_retrieved_before_received_sends() {
        let mut gwe = Gwe::default();
        let thread_id = 7;
        let hwnd = gwe.create_window(thread_id, "target", "");
        gwe.post_message(thread_id, Message::new(hwnd, WM_USER + 1, 0x11, 0x12, 10));
        assert!(
            gwe.queue_sent_message_for_window(
                hwnd,
                Message::new(hwnd, WM_USER + 2, 0x21, 0x22, 11)
            )
        );

        assert_eq!(
            gwe.get_queue_status(thread_id, QS_SENDMESSAGE),
            (QS_SENDMESSAGE << 16) | QS_SENDMESSAGE
        );
        assert!(!gwe.in_send_message(thread_id));

        let posted = gwe.get_message(thread_id).unwrap();
        assert_eq!(posted.msg, WM_USER + 1);
        assert_eq!(posted.source, MSGSRC_SOFTWARE_POST);
        assert_eq!(gwe.get_message_source(thread_id), MSGSRC_SOFTWARE_POST);
        assert!(!gwe.in_send_message(thread_id));

        let sent = gwe.get_message(thread_id).unwrap();
        assert_eq!(sent.msg, WM_USER + 2);
        assert_eq!(sent.source, MSGSRC_SOFTWARE_SEND);
        assert!(gwe.in_send_message(thread_id));
        assert_eq!(gwe.get_message_source(thread_id), MSGSRC_SOFTWARE_SEND);

        gwe.end_send_message(thread_id);
        assert!(!gwe.in_send_message(thread_id));
    }

    #[test]
    fn mouse_button_messages_update_lbutton_key_state() {
        let mut gwe = Gwe::default();
        let thread_id = 7;
        let hwnd = gwe.create_window(thread_id, "target", "");

        gwe.post_message(thread_id, Message::new(hwnd, WM_LBUTTONDOWN, 1, 0, 10));
        assert_ne!(gwe.get_key_state(VK_LBUTTON) & 0x8000_0000, 0);

        gwe.post_message(thread_id, Message::new(hwnd, WM_LBUTTONUP, 0, 0, 90));
        assert_eq!(gwe.get_key_state(VK_LBUTTON) & 0x8000_0000, 0);
    }

    #[test]
    fn synchronous_sent_message_records_result_for_sender() {
        let mut gwe = Gwe::default();
        let sender_thread = 11;
        let receiver_thread = 12;
        let hwnd = gwe.create_window(receiver_thread, "target", "");

        let send_id = gwe
            .queue_send_message_for_window(
                Some(sender_thread),
                hwnd,
                Message::new(hwnd, WM_ERASEBKGND, 0xaa, 0xbb, 0),
                SMF_NULL,
                Some(250),
            )
            .expect("queued sync send");
        assert_eq!(
            gwe.sent_message(send_id)
                .and_then(|sent| sent.sender_thread_id),
            Some(sender_thread)
        );

        let message = gwe.get_message(receiver_thread).expect("receiver message");
        assert_eq!(message.msg, WM_ERASEBKGND);
        assert_eq!(gwe.active_sent_message_id(receiver_thread), Some(send_id));
        assert!(gwe.in_send_message(receiver_thread));

        assert_eq!(
            gwe.complete_active_sent_message(receiver_thread, 0x1234),
            Some(send_id)
        );
        assert!(!gwe.in_send_message(receiver_thread));
        assert_eq!(
            gwe.take_completed_sent_message_result(send_id),
            Some(0x1234)
        );
        assert!(gwe.sent_message(send_id).is_none());
    }

    #[test]
    fn destroying_target_marks_queued_sync_send_receiver_terminated() {
        let mut gwe = Gwe::default();
        let sender_thread = 21;
        let receiver_thread = 22;
        let hwnd = gwe.create_window(receiver_thread, "target", "");

        let send_id = gwe
            .queue_send_message_for_window(
                Some(sender_thread),
                hwnd,
                Message::new(hwnd, WM_USER + 55, 1, 2, 0),
                SMF_NULL,
                None,
            )
            .expect("queued sync send");

        assert!(gwe.destroy_window(hwnd, 0));
        let sent = gwe.sent_message(send_id).expect("terminated send state");
        assert_ne!(sent.flags & SMF_RECEIVER_TERMINATED, 0);
        assert_ne!(sent.flags & SMF_RESULT_READY, 0);
        assert_eq!(gwe.get_message(receiver_thread), None);
        assert_eq!(gwe.take_completed_sent_message_result(send_id), Some(0));
    }

    #[test]
    fn timed_out_sent_message_is_removed_from_receiver_queue() {
        let mut gwe = Gwe::default();
        let sender_thread = 31;
        let receiver_thread = 32;
        let hwnd = gwe.create_window(receiver_thread, "target", "");

        let send_id = gwe
            .queue_send_message_for_window(
                Some(sender_thread),
                hwnd,
                Message::new(hwnd, WM_USER + 56, 1, 2, 100),
                SMF_TIMEOUT,
                Some(25),
            )
            .expect("queued timeout send");

        assert!(gwe.expire_timed_out_sent_messages(124).is_empty());
        assert!(gwe.get_queue_status(receiver_thread, QS_SENDMESSAGE) != 0);
        assert_eq!(gwe.expire_timed_out_sent_messages(125), vec![send_id]);
        let sent = gwe.sent_message(send_id).expect("timed out send state");
        assert_ne!(sent.flags & SMF_TIMEOUT, 0);
        assert_ne!(sent.flags & SMF_RESULT_READY, 0);
        assert_eq!(gwe.get_message(receiver_thread), None);
        assert_eq!(gwe.take_completed_sent_message_result(send_id), Some(0));
        assert_eq!(gwe.stats().send_transaction_timeout_count, 1);
    }

    #[test]
    fn reply_message_marks_result_ready_without_leaving_send_state() {
        let mut gwe = Gwe::default();
        let sender_thread = 33;
        let receiver_thread = 34;
        let hwnd = gwe.create_window(receiver_thread, "reply", "");

        let send_id = gwe
            .queue_send_message_for_window(
                Some(sender_thread),
                hwnd,
                Message::new(hwnd, WM_USER + 70, 1, 2, 0),
                SMF_NULL,
                None,
            )
            .expect("queued sync send");
        let message = gwe.get_message(receiver_thread).expect("receiver message");
        assert_eq!(message.msg, WM_USER + 70);
        assert!(gwe.in_send_message(receiver_thread));

        assert_eq!(gwe.reply_message(receiver_thread, 0xbeef), Some(send_id));
        assert!(!gwe.in_send_message(receiver_thread));
        assert_eq!(gwe.reply_message(receiver_thread, 0xcafe), None);
        assert_eq!(
            gwe.complete_active_sent_message(receiver_thread, 0x1234),
            Some(send_id)
        );
        assert_eq!(
            gwe.take_completed_sent_message_result(send_id),
            Some(0xbeef)
        );
        assert_eq!(gwe.stats().send_transaction_completed_count, 1);
    }

    #[test]
    fn nested_reply_message_only_clears_the_replied_send_depth() {
        let mut gwe = Gwe::default();
        let receiver_thread = 36;
        let hwnd = gwe.create_window(receiver_thread, "nested-reply", "");
        let outer_id = gwe
            .queue_send_message_for_window(
                Some(35),
                hwnd,
                Message::new(hwnd, WM_USER + 71, 1, 2, 0),
                SMF_NULL,
                None,
            )
            .expect("outer send");
        let inner_id = gwe
            .queue_send_message_for_window(
                Some(37),
                hwnd,
                Message::new(hwnd, WM_USER + 72, 3, 4, 0),
                SMF_NULL,
                None,
            )
            .expect("inner send");

        assert_eq!(
            gwe.get_message(receiver_thread).map(|message| message.msg),
            Some(WM_USER + 71)
        );
        assert!(gwe.in_send_message(receiver_thread));
        assert_eq!(
            gwe.get_message(receiver_thread).map(|message| message.msg),
            Some(WM_USER + 72)
        );
        assert!(gwe.in_send_message(receiver_thread));

        assert_eq!(gwe.reply_message(receiver_thread, 0x2222), Some(inner_id));
        assert!(!gwe.in_send_message(receiver_thread));
        assert_eq!(
            gwe.complete_active_sent_message(receiver_thread, 0x3333),
            Some(inner_id)
        );
        assert!(gwe.in_send_message(receiver_thread));
        assert_eq!(
            gwe.complete_active_sent_message(receiver_thread, 0x1111),
            Some(outer_id)
        );
        assert!(!gwe.in_send_message(receiver_thread));
        assert_eq!(
            gwe.take_completed_sent_message_result(inner_id),
            Some(0x2222)
        );
        assert_eq!(
            gwe.take_completed_sent_message_result(outer_id),
            Some(0x1111)
        );
    }

    #[test]
    fn being_destroyed_window_remains_valid_until_final_destroy() {
        let mut gwe = Gwe::default();
        let hwnd = gwe.create_window(1, "destroying", "");
        let child = gwe.create_window_ex(1, "child", "", Some(hwnd), 10, 0, 0);

        assert!(gwe.mark_window_subtree_being_destroyed(hwnd));
        assert!(gwe.is_window(hwnd));
        assert!(gwe.is_window(child));
        assert!(gwe.is_window_being_destroyed(hwnd));
        assert!(gwe.is_window_being_destroyed(child));
        assert_eq!(gwe.send_message(child, WM_USER + 0x32, 1, 2), Some(0));

        assert!(gwe.destroy_window(hwnd, 0));
        assert!(!gwe.is_window(hwnd));
        assert!(!gwe.is_window(child));
        assert!(!gwe.is_window_being_destroyed(hwnd));
    }

    #[test]
    fn owned_popup_is_part_of_owner_destroy_subtree() {
        let mut gwe = Gwe::default();
        let owner = gwe.create_window_ex_with_process_parent_owner_and_rect(
            1,
            1,
            "owner",
            "",
            None,
            None,
            0,
            0,
            0,
            Rect::from_origin_size(0, 0, 100, 100),
        );
        let owned = gwe.create_window_ex_with_process_parent_owner_and_rect(
            1,
            1,
            "owned",
            "",
            None,
            Some(owner),
            WS_POPUP,
            0,
            0,
            Rect::from_origin_size(0, 0, 100, 100),
        );
        let child = gwe.create_window_ex(1, "child", "", Some(owned), 10, 0, 0);

        assert_eq!(
            gwe.window_and_descendants(owner).unwrap(),
            vec![owner, owned, child]
        );
        assert!(gwe.mark_window_subtree_being_destroyed(owner));
        assert!(gwe.is_window_being_destroyed(owner));
        assert!(gwe.is_window_being_destroyed(owned));
        assert!(gwe.is_window_being_destroyed(child));

        assert!(gwe.destroy_window(owner, 0));
        assert!(!gwe.is_window(owner));
        assert!(!gwe.is_window(owned));
        assert!(!gwe.is_window(child));
    }

    #[test]
    fn queue_status_low_word_tracks_changed_bits_until_observed() {
        let mut gwe = Gwe::default();
        let thread_id = 41;

        gwe.post_message(thread_id, Message::new(0, WM_USER + 1, 0, 0, 0));
        assert_eq!(
            gwe.get_queue_status(thread_id, QS_POSTMESSAGE),
            (QS_POSTMESSAGE << 16) | QS_POSTMESSAGE
        );
        assert_eq!(
            gwe.get_queue_status(thread_id, QS_POSTMESSAGE),
            QS_POSTMESSAGE << 16
        );

        gwe.post_message(thread_id, Message::new(0, WM_USER + 2, 0, 0, 0));
        assert_eq!(
            gwe.get_queue_status(thread_id, QS_POSTMESSAGE),
            (QS_POSTMESSAGE << 16) | QS_POSTMESSAGE
        );
    }

    #[test]
    fn post_quit_state_ignores_window_and_message_filters() {
        let mut gwe = Gwe::default();
        let thread_id = 42;
        let hwnd = gwe.create_window(thread_id, "target", "");

        gwe.post_quit_message(thread_id, 0x4d, 123);
        assert_eq!(
            gwe.get_queue_status(thread_id, QS_POSTMESSAGE),
            (QS_POSTMESSAGE << 16) | QS_POSTMESSAGE
        );

        let peeked = gwe
            .peek_message_filtered(
                thread_id,
                Some(hwnd),
                WM_USER + 1,
                WM_USER + 1,
                PeekFlags::NO_REMOVE,
            )
            .expect("peeked quit");
        assert_eq!(peeked.msg, WM_QUIT);
        assert_eq!(peeked.wparam, 0x4d);

        let quit = gwe
            .get_message_filtered(thread_id, Some(hwnd), WM_USER + 2, WM_USER + 2)
            .expect("got quit");
        assert_eq!(quit.msg, WM_QUIT);
        assert_eq!(quit.wparam, 0x4d);
        assert_eq!(gwe.get_message(thread_id), None);
        assert_eq!(gwe.get_queue_status(thread_id, QS_POSTMESSAGE), 0);
    }

    #[test]
    fn create_and_show_window_track_visibility() {
        let mut gwe = Gwe::default();
        let hwnd = gwe.create_window_ex(1, "STATIC", "ready", None, 0, WS_VISIBLE, 0);

        assert!(gwe.is_window_visible(hwnd));
        assert!(gwe.update_rect(hwnd).is_some());
        assert!(gwe.show_window(hwnd, false));
        assert!(!gwe.is_window_visible(hwnd));
        assert!(gwe.update_rect(hwnd).is_none());
        assert!(!gwe.show_window(hwnd, true));
        assert!(gwe.is_window_visible(hwnd));
        assert_eq!(
            gwe.update_rect(hwnd).unwrap().rect,
            Rect::from_origin_size(0, 0, 800, 480)
        );
    }

    #[test]
    fn destroy_window_invalidates_newly_exposed_windows() {
        let mut gwe = Gwe::default();
        let background = gwe.create_window_ex_with_rect(
            1,
            "STATIC",
            "background",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, 800, 480),
        );
        let foreground = gwe.create_window_ex_with_rect(
            1,
            "STATIC",
            "foreground",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(50, 60, 100, 80),
        );
        assert!(gwe.validate_window(background));
        assert!(gwe.validate_window(foreground));

        assert!(gwe.destroy_window(foreground, 0));

        assert_eq!(
            gwe.update_rect(background).unwrap().rect,
            Rect::from_origin_size(50, 60, 100, 80)
        );
    }

    #[test]
    fn hiding_window_invalidates_newly_exposed_windows() {
        let mut gwe = Gwe::default();
        let background = gwe.create_window_ex_with_rect(
            1,
            "STATIC",
            "background",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(10, 20, 200, 120),
        );
        let foreground = gwe.create_window_ex_with_rect(
            1,
            "STATIC",
            "foreground",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(40, 50, 80, 60),
        );
        assert!(gwe.validate_window(background));
        assert!(gwe.validate_window(foreground));

        assert!(gwe.show_window(foreground, false));

        assert!(gwe.update_rect(foreground).is_none());
        assert_eq!(
            gwe.update_rect(background).unwrap().rect,
            Rect::from_origin_size(30, 30, 80, 60)
        );
    }

    #[test]
    fn validate_window_rect_subtracts_representable_update_bounds() {
        let mut gwe = Gwe::default();
        let hwnd = gwe.create_window_ex_with_rect(
            1,
            "STATIC",
            "ready",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, 100, 80),
        );
        assert!(gwe.validate_window(hwnd));
        assert!(gwe.invalidate_window(hwnd, None, true));

        assert!(gwe.validate_window_rect(hwnd, Some(Rect::from_origin_size(0, 0, 100, 20))));
        assert_eq!(
            gwe.update_rect(hwnd).unwrap().rect,
            Rect::from_origin_size(0, 20, 100, 60)
        );

        assert!(gwe.validate_window_rect(hwnd, Some(Rect::from_origin_size(200, 200, 20, 20))));
        assert_eq!(
            gwe.update_rect(hwnd).unwrap().rect,
            Rect::from_origin_size(0, 20, 100, 60)
        );

        assert!(gwe.validate_window(hwnd));
        assert!(gwe.invalidate_window(hwnd, None, true));
        assert!(gwe.validate_window_rect(hwnd, Some(Rect::from_origin_size(25, 20, 50, 40))));
        assert_eq!(
            gwe.update_rect(hwnd).unwrap().rect,
            Rect::from_origin_size(0, 0, 100, 80)
        );

        assert!(gwe.validate_window_rect(hwnd, None));
        assert!(gwe.update_rect(hwnd).is_none());
    }

    #[test]
    fn set_window_pos_clips_pending_update_to_new_client_rect() {
        let mut gwe = Gwe::default();
        let hwnd = gwe.create_window_ex_with_rect(
            1,
            "STATIC",
            "ready",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, 800, 480),
        );
        assert_eq!(
            gwe.update_rect(hwnd).unwrap().rect,
            Rect::from_origin_size(0, 0, 800, 480)
        );

        assert!(gwe.set_window_pos(hwnd, None, 10, 20, 64, 62, 0));

        let update = gwe
            .update_rect(hwnd)
            .expect("pending update remains clipped");
        assert_eq!(update.rect, Rect::from_origin_size(0, 0, 64, 62));
        assert!(update.erase);
    }

    #[test]
    fn set_window_pos_invalidates_clean_visible_window_without_no_redraw() {
        let mut gwe = Gwe::default();
        let hwnd = gwe.create_window_ex_with_rect(
            1,
            "STATIC",
            "ready",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, 100, 80),
        );
        assert!(gwe.validate_window(hwnd));
        assert!(gwe.update_rect(hwnd).is_none());

        assert!(gwe.set_window_pos(hwnd, None, 10, 20, 100, 80, SWP_NOZORDER));

        let update = gwe
            .update_rect(hwnd)
            .expect("visible move schedules repaint");
        assert_eq!(update.rect, Rect::from_origin_size(0, 0, 100, 80));
        assert!(update.erase);
    }

    #[test]
    fn set_window_pos_no_redraw_keeps_clean_visible_window_clean() {
        let mut gwe = Gwe::default();
        let hwnd = gwe.create_window_ex_with_rect(
            1,
            "STATIC",
            "ready",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, 100, 80),
        );
        assert!(gwe.validate_window(hwnd));
        assert!(gwe.update_rect(hwnd).is_none());

        assert!(gwe.set_window_pos(hwnd, None, 10, 20, 100, 80, SWP_NOZORDER | SWP_NOREDRAW));

        assert!(gwe.update_rect(hwnd).is_none());
    }

    #[test]
    fn set_window_pos_z_order_change_invalidates_promoted_visible_window() {
        let mut gwe = Gwe::default();
        let top = gwe.create_window_ex_with_rect(
            1,
            "STATIC",
            "top",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, 100, 80),
        );
        let bottom = gwe.create_window_ex_with_rect(
            1,
            "STATIC",
            "bottom",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, 100, 80),
        );
        assert!(gwe.validate_window(top));
        assert!(gwe.validate_window(bottom));

        assert!(gwe.set_window_pos(bottom, Some(HWND_TOP), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE));

        let update = gwe
            .update_rect(bottom)
            .expect("z-order promotion schedules repaint");
        assert_eq!(update.rect, Rect::from_origin_size(0, 0, 100, 80));
        assert!(update.erase);
    }

    #[test]
    fn set_window_pos_hide_clears_pending_update() {
        let mut gwe = Gwe::default();
        let hwnd = gwe.create_window_ex_with_rect(
            1,
            "STATIC",
            "ready",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, 100, 80),
        );
        assert!(gwe.update_rect(hwnd).is_some());

        assert!(gwe.set_window_pos(hwnd, None, 0, 0, 100, 80, SWP_HIDEWINDOW));

        assert!(!gwe.is_window_visible(hwnd));
        assert!(gwe.update_rect(hwnd).is_none());
    }

    #[test]
    fn set_window_pos_hide_invalidates_newly_exposed_windows() {
        let mut gwe = Gwe::default();
        let background = gwe.create_window_ex_with_rect(
            1,
            "STATIC",
            "background",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(0, 0, 320, 240),
        );
        let foreground = gwe.create_window_ex_with_rect(
            1,
            "STATIC",
            "foreground",
            None,
            0,
            WS_VISIBLE,
            0,
            Rect::from_origin_size(20, 30, 64, 48),
        );
        assert!(gwe.validate_window(background));
        assert!(gwe.validate_window(foreground));

        assert!(gwe.set_window_pos(
            foreground,
            None,
            20,
            30,
            64,
            48,
            SWP_NOMOVE | SWP_NOSIZE | SWP_HIDEWINDOW
        ));

        assert!(!gwe.is_window_visible(foreground));
        assert_eq!(
            gwe.update_rect(background).unwrap().rect,
            Rect::from_origin_size(20, 30, 64, 48)
        );
    }
}
