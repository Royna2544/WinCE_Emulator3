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
pub const WM_TIMER: u32 = 0x0113;
pub const WM_USER: u32 = 0x0400;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub hwnd: u32,
    pub msg: u32,
    pub wparam: u32,
    pub lparam: u32,
    pub time_ms: u32,
}

#[derive(Debug, Clone)]
pub struct Window {
    pub hwnd: u32,
    pub thread_id: u32,
    pub class_name: String,
    pub title: String,
    pub visible: bool,
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
            },
        );
        self.post_message(thread_id, Message::new(hwnd, WM_CREATE, 0, 0, 0));
        hwnd
    }

    pub fn destroy_window(&mut self, hwnd: u32, time_ms: u32) -> bool {
        let Some(window) = self.windows.remove(&hwnd) else {
            return false;
        };
        self.post_message(
            window.thread_id,
            Message::new(hwnd, WM_DESTROY, 0, 0, time_ms),
        );
        true
    }

    pub fn show_window(&mut self, hwnd: u32, visible: bool) -> bool {
        let Some(window) = self.windows.get_mut(&hwnd) else {
            return false;
        };
        window.visible = visible;
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
        self.windows
            .contains_key(&hwnd)
            .then_some(default_send_message_result(msg, wparam, lparam))
    }

    pub fn get_message(&mut self, thread_id: u32) -> Option<Message> {
        self.queues
            .get_mut(&thread_id)
            .and_then(VecDeque::pop_front)
    }

    pub fn peek_message(&mut self, thread_id: u32, flags: PeekFlags) -> Option<Message> {
        let queue = self.queues.get_mut(&thread_id)?;
        if flags.contains(PeekFlags::REMOVE) {
            queue.pop_front()
        } else {
            queue.front().cloned()
        }
    }

    pub fn window(&self, hwnd: u32) -> Option<&Window> {
        self.windows.get(&hwnd)
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
        _ => 0,
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
