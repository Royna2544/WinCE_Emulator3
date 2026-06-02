use std::collections::{BTreeMap, VecDeque};

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
pub const WM_WINDOWPOSCHANGED: u32 = 0x0047;
pub const WM_SETTEXT: u32 = 0x000c;
pub const WM_GETTEXT: u32 = 0x000d;
pub const WM_GETTEXTLENGTH: u32 = 0x000e;
pub const WM_KEYDOWN: u32 = 0x0100;
pub const WM_CHAR: u32 = 0x0102;
pub const WM_COMMAND: u32 = 0x0111;
pub const WM_TIMER: u32 = 0x0113;
pub const WM_USER: u32 = 0x0400;
pub const MSGSRC_UNKNOWN: u32 = 0;
pub const MSGSRC_SOFTWARE_POST: u32 = 1;
pub const MSGSRC_HARDWARE_KEYBOARD: u32 = 2;
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
pub const SWP_NOACTIVATE: u32 = 0x0010;
pub const SWP_SHOWWINDOW: u32 = 0x0040;
pub const SWP_HIDEWINDOW: u32 = 0x0080;
pub const HWND_TOP: u32 = 0;
pub const HWND_BOTTOM: u32 = 1;
pub const HWND_TOPMOST: u32 = u32::MAX;
pub const HWND_NOTOPMOST: u32 = u32::MAX - 1;
pub const WS_CHILD: u32 = 0x4000_0000;
pub const WS_VISIBLE: u32 = 0x1000_0000;

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
    pub class_name: String,
    pub title: String,
    pub visible: bool,
    pub enabled: bool,
    pub parent: Option<u32>,
    pub id: u32,
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
    pub destroyed: bool,
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

#[derive(Debug, Clone)]
pub struct Gwe {
    next_hwnd: u32,
    next_class_atom: u16,
    classes_by_name: BTreeMap<String, WindowClass>,
    class_names_by_atom: BTreeMap<u16, String>,
    windows: BTreeMap<u32, Window>,
    z_order: Vec<u32>,
    queues: BTreeMap<u32, VecDeque<Message>>,
    focus: Option<u32>,
    capture: Option<u32>,
    cursor: Option<u32>,
    cursor_pos: Point,
    next_registered_message: u32,
    registered_messages: BTreeMap<String, u32>,
    gesture_registrations: BTreeMap<u32, GestureRegistration>,
    dialog_results: BTreeMap<u32, u32>,
    dialog_checks: BTreeMap<(u32, u32), u32>,
    window_regions: BTreeMap<u32, Rect>,
    send_depth_by_thread: BTreeMap<u32, u32>,
    last_message_source_by_thread: BTreeMap<u32, u32>,
    replied_send_depth_by_thread: BTreeMap<u32, u32>,
}

impl Default for Gwe {
    fn default() -> Self {
        let mut windows = BTreeMap::new();
        windows.insert(
            DESKTOP_HWND,
            Window {
                hwnd: DESKTOP_HWND,
                thread_id: 0,
                class_name: "Desktop".to_owned(),
                title: String::new(),
                visible: true,
                enabled: true,
                parent: None,
                id: 0,
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
                destroyed: false,
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
            focus: None,
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
            replied_send_depth_by_thread: BTreeMap::new(),
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
        let update_rect = rect.zero_origin();
        self.windows.insert(
            hwnd,
            Window {
                hwnd,
                thread_id,
                class_name,
                title: title.to_owned(),
                visible,
                enabled: true,
                parent,
                id,
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
                destroyed: false,
            },
        );
        self.z_order.push(hwnd);
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
        if !self.is_window(hwnd) {
            return false;
        }
        let mut targets = vec![hwnd];
        let mut index = 0;
        while let Some(parent) = targets.get(index).copied() {
            index += 1;
            let children: Vec<u32> = self
                .windows
                .values()
                .filter(|window| !window.destroyed && window.parent == Some(parent))
                .map(|window| window.hwnd)
                .collect();
            targets.extend(children);
        }
        for target in targets.iter().rev().copied() {
            if let Some(window) = self.windows.get_mut(&target) {
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
        for queue in self.queues.values_mut() {
            queue.retain(|message| message.hwnd == 0 || !targets.contains(&message.hwnd));
        }
        true
    }

    pub fn show_window(&mut self, hwnd: u32, visible: bool) -> bool {
        let Some(window) = self.windows.get_mut(&hwnd) else {
            return false;
        };
        let previous = window.visible;
        window.visible = visible;
        if visible && !previous {
            window.update_pending = true;
            window.erase_pending = true;
            window.update_rect = window.client_rect.zero_origin();
        }
        previous
    }

    pub fn update_window(&self, hwnd: u32) -> bool {
        self.is_window(hwnd)
    }

    pub fn invalidate_window(&mut self, hwnd: u32, rect: Option<Rect>, erase: bool) -> bool {
        let Some(window) = self.windows.get_mut(&hwnd) else {
            return false;
        };
        if window.destroyed {
            return false;
        }
        window.update_pending = true;
        window.erase_pending |= erase;
        window.update_rect = rect.unwrap_or_else(|| window.client_rect.zero_origin());
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

    pub fn update_rect(&self, hwnd: u32) -> Option<PaintUpdate> {
        let window = self.windows.get(&hwnd)?;
        (!window.destroyed && window.update_pending).then_some(PaintUpdate {
            rect: window.update_rect,
            erase: window.erase_pending,
        })
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
        let previous = window.enabled;
        window.enabled = enabled;
        previous
    }

    pub fn is_window_enabled(&self, hwnd: u32) -> bool {
        self.windows
            .get(&hwnd)
            .is_some_and(|window| !window.destroyed && window.enabled)
    }

    pub fn is_window_visible(&self, hwnd: u32) -> bool {
        self.windows
            .get(&hwnd)
            .is_some_and(|window| !window.destroyed && window.visible)
    }

    pub fn get_parent(&self, hwnd: u32) -> Option<u32> {
        self.windows.get(&hwnd).and_then(|window| window.parent)
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
        if !self.is_window(hwnd) {
            return false;
        }
        if let Some(rect) = rect {
            self.window_regions.insert(hwnd, rect);
        } else {
            self.window_regions.remove(&hwnd);
        }
        true
    }

    pub fn window_region(&self, hwnd: u32) -> Option<Rect> {
        self.is_window(hwnd)
            .then(|| self.window_regions.get(&hwnd).copied())
            .flatten()
    }

    pub fn set_parent(&mut self, hwnd: u32, parent: Option<u32>) -> Option<Option<u32>> {
        if parent.is_some_and(|parent| !self.is_window(parent) || parent == hwnd) {
            return None;
        }
        let window = self.windows.get_mut(&hwnd)?;
        if window.destroyed {
            return None;
        }
        let previous = window.parent;
        window.parent = parent;
        Some(previous)
    }

    pub fn get_window(&self, hwnd: u32, cmd: u32) -> Option<u32> {
        let window = self.windows.get(&hwnd)?;
        if window.destroyed {
            return None;
        }

        match cmd {
            GW_OWNER => None,
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
        if let Some(hwnd) = self.focus.filter(|hwnd| self.is_window(*hwnd)) {
            return Some(hwnd);
        }
        self.windows
            .values()
            .find(|window| {
                !window.destroyed && window.parent.is_none() && window.hwnd != DESKTOP_HWND
            })
            .map(|window| window.hwnd)
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
        if flags & SWP_SHOWWINDOW != 0 {
            window.visible = true;
            window.update_pending = true;
            window.erase_pending = true;
            window.update_rect = window.client_rect.zero_origin();
        }
        if flags & SWP_HIDEWINDOW != 0 {
            window.visible = false;
        }
        if flags & SWP_NOZORDER == 0 {
            self.apply_z_order(hwnd, parent, insert_after.unwrap_or(HWND_TOP));
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
        let moved = self.set_window_pos(hwnd, None, x, y, width, height, 0);
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
        self.queues.entry(thread_id).or_default().push_back(message);
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
                self.destroy_window(hwnd, 0);
            }
            WM_PAINT => {
                self.validate_window(hwnd);
            }
            _ => {}
        }
        Some(default_send_message_result(msg, wparam, lparam))
    }

    pub fn begin_send_message(&mut self, thread_id: u32) {
        *self.send_depth_by_thread.entry(thread_id).or_default() += 1;
    }

    pub fn end_send_message(&mut self, thread_id: u32) {
        let Some(depth) = self.send_depth_by_thread.get_mut(&thread_id) else {
            return;
        };
        *depth = depth.saturating_sub(1);
        if *depth == 0 {
            self.send_depth_by_thread.remove(&thread_id);
            self.replied_send_depth_by_thread.remove(&thread_id);
        }
    }

    pub fn in_send_message(&self, thread_id: u32) -> bool {
        self.send_depth_by_thread
            .get(&thread_id)
            .copied()
            .unwrap_or(0)
            > 0
    }

    pub fn reply_message(&mut self, thread_id: u32, _result: u32) -> bool {
        let depth = self
            .send_depth_by_thread
            .get(&thread_id)
            .copied()
            .unwrap_or(0);
        if depth == 0 {
            return false;
        }
        self.replied_send_depth_by_thread.insert(thread_id, depth);
        true
    }

    pub fn get_message_source(&self, thread_id: u32) -> u32 {
        self.last_message_source_by_thread
            .get(&thread_id)
            .copied()
            .unwrap_or(MSGSRC_UNKNOWN)
    }

    pub fn get_queue_status(&self, thread_id: u32, flags: u32) -> u32 {
        let current = self.queue_status_bits(thread_id) & flags;
        (current << 16) | current
    }

    pub fn has_queue_input(&self, thread_id: u32, flags: u32) -> bool {
        self.queue_status_bits(thread_id) & flags != 0
    }

    pub fn get_message(&mut self, thread_id: u32) -> Option<Message> {
        self.get_message_filtered(thread_id, None, 0, 0)
    }

    pub fn peek_message(&mut self, thread_id: u32, flags: PeekFlags) -> Option<Message> {
        self.peek_message_filtered(thread_id, None, 0, 0, flags)
    }

    pub fn post_quit_message(&mut self, thread_id: u32, exit_code: u32, time_ms: u32) {
        self.post_message(thread_id, Message::new(0, WM_QUIT, exit_code, 0, time_ms));
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
        let message = self.synthetic_paint_message(thread_id, hwnd, min_msg, max_msg)?;
        self.last_message_source_by_thread
            .insert(thread_id, message.source);
        Some(message)
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
                        self.last_message_source_by_thread
                            .insert(thread_id, message.source);
                    }
                    message
                } else {
                    queue.get(index).cloned()
                };
            }
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
        Some(message)
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
        self.windows
            .values()
            .find(|window| {
                !window.destroyed
                    && window.thread_id == thread_id
                    && window.visible
                    && window.update_pending
                    && hwnd.is_none_or(|wanted| window.hwnd == wanted)
            })
            .map(|window| Message::new(window.hwnd, WM_PAINT, 0, 0, 0))
    }

    fn queue_status_bits(&self, thread_id: u32) -> u32 {
        let mut status = 0;
        if self.in_send_message(thread_id) {
            status |= QS_SENDMESSAGE;
        }
        if let Some(queue) = self.queues.get(&thread_id) {
            for message in queue {
                status |= match message.msg {
                    WM_TIMER => QS_TIMER,
                    WM_PAINT => QS_PAINT,
                    WM_KEYDOWN | WM_CHAR => QS_KEY,
                    0x0201 | 0x0202 => QS_MOUSEBUTTON,
                    0x0200 => QS_MOUSEMOVE,
                    _ => QS_POSTMESSAGE,
                };
            }
        }
        if self.windows.values().any(|window| {
            !window.destroyed
                && window.thread_id == thread_id
                && window.visible
                && window.update_pending
        }) {
            status |= QS_PAINT;
        }
        status
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
        }
    }

    pub fn with_source(mut self, source: u32) -> Self {
        self.source = source;
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
    fn hidden_and_child_zero_size_windows_keep_requested_rect() {
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
            Rect::from_origin_size(10, 20, 0, 0)
        );
    }

    #[test]
    fn create_and_show_window_track_visibility() {
        let mut gwe = Gwe::default();
        let hwnd = gwe.create_window_ex(1, "STATIC", "ready", None, 0, WS_VISIBLE, 0);

        assert!(gwe.is_window_visible(hwnd));
        assert!(gwe.show_window(hwnd, false));
        assert!(!gwe.is_window_visible(hwnd));
        assert!(!gwe.show_window(hwnd, true));
        assert!(gwe.is_window_visible(hwnd));
    }
}
