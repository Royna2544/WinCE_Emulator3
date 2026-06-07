use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotifyIconOp {
    Add,
    Modify,
    Delete,
}

impl NotifyIconOp {
    pub fn from_raw(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Add),
            1 => Some(Self::Modify),
            2 => Some(Self::Delete),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotifyIconData {
    pub hwnd: u32,
    pub id: u32,
    pub flags: u32,
    pub callback_message: u32,
    pub icon: u32,
    pub tip: String,
    pub state: u32,
    pub state_mask: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotifyIconRecord {
    pub hwnd: u32,
    pub id: u32,
    pub callback_message: u32,
    pub icon: u32,
    pub tip: String,
    pub state: u32,
}

#[derive(Debug, Clone, Default)]
pub struct ShellSystem {
    notify_icons: BTreeMap<(u32, u32), NotifyIconRecord>,
}

impl ShellSystem {
    pub fn apply_notify_icon(&mut self, op: NotifyIconOp, data: NotifyIconData) -> bool {
        let key = (data.hwnd, data.id);
        match op {
            NotifyIconOp::Add => {
                self.notify_icons
                    .insert(key, NotifyIconRecord::from_data(data));
                true
            }
            NotifyIconOp::Modify => {
                let Some(record) = self.notify_icons.get_mut(&key) else {
                    return false;
                };
                record.update(data);
                true
            }
            NotifyIconOp::Delete => self.notify_icons.remove(&key).is_some(),
        }
    }

    pub fn notify_icon(&self, hwnd: u32, id: u32) -> Option<&NotifyIconRecord> {
        self.notify_icons.get(&(hwnd, id))
    }

    pub fn notify_icons(&self) -> impl Iterator<Item = &NotifyIconRecord> {
        self.notify_icons.values()
    }
}

impl NotifyIconRecord {
    fn from_data(data: NotifyIconData) -> Self {
        Self {
            hwnd: data.hwnd,
            id: data.id,
            callback_message: data.callback_message,
            icon: data.icon,
            tip: data.tip,
            state: data.state,
        }
    }

    fn update(&mut self, data: NotifyIconData) {
        if data.flags & NIF_MESSAGE != 0 {
            self.callback_message = data.callback_message;
        }
        if data.flags & NIF_ICON != 0 {
            self.icon = data.icon;
        }
        if data.flags & NIF_TIP != 0 {
            self.tip = data.tip;
        }
        if data.flags & NIF_STATE != 0 {
            self.state = (self.state & !data.state_mask) | (data.state & data.state_mask);
        }
    }
}

pub const NIF_MESSAGE: u32 = 0x0000_0001;
pub const NIF_ICON: u32 = 0x0000_0002;
pub const NIF_TIP: u32 = 0x0000_0004;
pub const NIF_STATE: u32 = 0x0000_0008;
