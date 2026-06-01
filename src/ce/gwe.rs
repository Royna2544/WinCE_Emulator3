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
pub const WM_CLOSE: u32 = 0x0010;
pub const WM_QUIT: u32 = 0x0012;
pub const WM_SETTEXT: u32 = 0x000c;
pub const WM_GETTEXT: u32 = 0x000d;
pub const WM_GETTEXTLENGTH: u32 = 0x000e;
pub const WM_TIMER: u32 = 0x0113;
pub const WM_USER: u32 = 0x0400;

pub const GWL_WNDPROC: i32 = -4;
pub const GWL_ID: i32 = -12;
pub const GWL_STYLE: i32 = -16;
pub const GWL_EXSTYLE: i32 = -20;
pub const GWL_USERDATA: i32 = -21;

pub const SWP_NOSIZE: u32 = 0x0001;
pub const SWP_NOMOVE: u32 = 0x0002;
pub const SWP_NOZORDER: u32 = 0x0004;
pub const SWP_NOACTIVATE: u32 = 0x0010;
pub const SWP_SHOWWINDOW: u32 = 0x0040;
pub const SWP_HIDEWINDOW: u32 = 0x0080;

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
    pub parent: Option<u32>,
    pub id: u32,
    pub style: u32,
    pub ex_style: u32,
    pub wndproc: u32,
    pub user_data: u32,
    pub rect: Rect,
    pub client_rect: Rect,
    pub destroyed: bool,
}

#[derive(Debug, Clone)]
pub struct Gwe {
    next_hwnd: u32,
    windows: BTreeMap<u32, Window>,
    queues: BTreeMap<u32, VecDeque<Message>>,
}

impl Default for Gwe {
    fn default() -> Self {
        Self {
            next_hwnd: 0x0002_0000,
            windows: BTreeMap::new(),
            queues: BTreeMap::new(),
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
        let hwnd = self.next_hwnd;
        self.next_hwnd += 4;
        self.windows.insert(
            hwnd,
            Window {
                hwnd,
                thread_id,
                class_name: class_name.to_owned(),
                title: title.to_owned(),
                visible: false,
                parent,
                id,
                style,
                ex_style,
                wndproc: 0,
                user_data: 0,
                rect,
                client_rect: rect,
                destroyed: false,
            },
        );
        self.post_message(thread_id, Message::new(hwnd, WM_CREATE, 0, 0, 0));
        hwnd
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
        window.visible = visible;
        true
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
        self.set_window_pos(hwnd, x, y, width, height, 0)
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
        self.post_message(window.thread_id, message);
        true
    }

    pub fn send_message(&mut self, hwnd: u32, msg: u32, wparam: u32, lparam: u32) -> Option<u32> {
        if !self.windows.contains_key(&hwnd) {
            return None;
        }
        if msg == WM_CLOSE {
            self.destroy_window(hwnd, 0);
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
    }

    pub fn peek_message_filtered(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
        flags: PeekFlags,
    ) -> Option<Message> {
        let queue = self.queues.get_mut(&thread_id)?;
        let index = queue
            .iter()
            .position(|message| message_matches(message, hwnd, min_msg, max_msg))?;
        if flags.contains(PeekFlags::REMOVE) {
            queue.remove(index)
        } else {
            queue.get(index).cloned()
        }
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
    (min_msg..=max_msg).contains(&message.msg)
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
    fn create_window_posts_create_message() {
        let mut gwe = Gwe::default();
        let hwnd = gwe.create_window(1, "STATIC", "ready");
        let message = gwe.get_message(1).unwrap();

        assert_eq!(message.hwnd, hwnd);
        assert_eq!(message.msg, WM_CREATE);
    }
}
