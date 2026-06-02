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
pub const WM_PAINT: u32 = 0x000f;
pub const WM_CLOSE: u32 = 0x0010;
pub const WM_QUIT: u32 = 0x0012;
pub const WM_ERASEBKGND: u32 = 0x0014;
pub const WM_SETTEXT: u32 = 0x000c;
pub const WM_GETTEXT: u32 = 0x000d;
pub const WM_GETTEXTLENGTH: u32 = 0x000e;
pub const WM_TIMER: u32 = 0x0113;
pub const WM_USER: u32 = 0x0400;

pub const HWND_BROADCAST: u32 = 0x0000_ffff;
pub const DESKTOP_HWND: u32 = 0x0001_0000;
pub const WNDCLASSW_SIZE: usize = 40;

pub const GWL_WNDPROC: i32 = -4;
pub const GWL_ID: i32 = -12;
pub const GWL_STYLE: i32 = -16;
pub const GWL_EXSTYLE: i32 = -20;
pub const GWL_USERDATA: i32 = -21;

pub const GW_HWNDFIRST: u32 = 0;
pub const GW_HWNDLAST: u32 = 1;
pub const GW_HWNDNEXT: u32 = 2;
pub const GW_HWNDPREV: u32 = 3;
pub const GW_OWNER: u32 = 4;
pub const GW_CHILD: u32 = 5;

pub const SWP_NOSIZE: u32 = 0x0001;
pub const SWP_NOMOVE: u32 = 0x0002;
pub const SWP_NOZORDER: u32 = 0x0004;
pub const SWP_NOACTIVATE: u32 = 0x0010;
pub const SWP_SHOWWINDOW: u32 = 0x0040;
pub const SWP_HIDEWINDOW: u32 = 0x0080;
pub const WS_VISIBLE: u32 = 0x1000_0000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub hwnd: u32,
    pub msg: u32,
    pub wparam: u32,
    pub lparam: u32,
    pub time_ms: u32,
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
    queues: BTreeMap<u32, VecDeque<Message>>,
    focus: Option<u32>,
    cursor_pos: Point,
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
            queues: BTreeMap::new(),
            focus: None,
            cursor_pos: Point::default(),
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
        let rect = self.parent_to_screen_rect(parent, rect);
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
            .unwrap_or(0);
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
                rect,
                client_rect: rect,
                update_pending: visible,
                erase_pending: visible,
                update_rect,
                destroyed: false,
            },
        );
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

    pub fn destroy_window(&mut self, hwnd: u32, time_ms: u32) -> bool {
        let Some(window) = self.windows.get_mut(&hwnd) else {
            return false;
        };
        if window.destroyed {
            return false;
        }
        window.destroyed = true;
        let thread_id = window.thread_id;
        self.post_message(thread_id, Message::new(hwnd, WM_DESTROY, 0, 0, time_ms));
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
        window.enabled = enabled;
        true
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

    pub fn set_window_pos(
        &mut self,
        hwnd: u32,
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
        let moved = self.set_window_pos(hwnd, x, y, width, height, 0);
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

    pub fn post_message(&mut self, thread_id: u32, message: Message) {
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
        if !self.windows.contains_key(&hwnd) {
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
        self.take_matching_message(thread_id, hwnd, min_msg, max_msg)
            .or_else(|| self.synthetic_paint_message(thread_id, hwnd, min_msg, max_msg))
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
                    queue.remove(index)
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
        window.title = title.to_owned();
        true
    }

    pub fn get_window_text(&self, hwnd: u32, capacity_chars: usize) -> Option<String> {
        let title = &self.windows.get(&hwnd)?.title;
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
        queue.remove(index)
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

    fn first_child(&self, hwnd: u32) -> Option<u32> {
        let parent = if hwnd == DESKTOP_HWND {
            None
        } else {
            Some(hwnd)
        };
        self.sibling_windows(parent).first().copied()
    }

    fn sibling_windows(&self, parent: Option<u32>) -> Vec<u32> {
        self.windows
            .values()
            .filter(|window| {
                !window.destroyed
                    && window.parent == parent
                    && (parent.is_some() || window.hwnd != DESKTOP_HWND)
            })
            .map(|window| window.hwnd)
            .collect()
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
        }
    }
}

fn default_send_message_result(msg: u32, _wparam: u32, _lparam: u32) -> u32 {
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
        _ => None,
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
