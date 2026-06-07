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
    notifications: BTreeMap<([u8; 16], u32), ShellNotificationRecord>,
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

    pub fn add_notification(&mut self, data: ShellNotificationData) -> NotificationResult {
        if !is_notification_priority_valid(data.priority)
            || data.title.is_empty() && data.html.is_empty()
        {
            return NotificationResult::InvalidParameter;
        }
        if data.priority == SHNP_INFORM && data.html.is_empty() {
            return NotificationResult::InvalidParameter;
        }
        let key = (data.clsid, data.id);
        self.notifications
            .insert(key, ShellNotificationRecord::from_data(data));
        NotificationResult::Success
    }

    pub fn update_notification(
        &mut self,
        update_mask: u32,
        data: ShellNotificationData,
    ) -> NotificationResult {
        if update_mask & SHNUM_PRIORITY != 0 && !is_notification_priority_valid(data.priority) {
            return NotificationResult::InvalidParameter;
        }
        let Some(record) = self.notifications.get_mut(&(data.clsid, data.id)) else {
            return NotificationResult::InvalidData;
        };
        record.update(update_mask, data);
        NotificationResult::Success
    }

    pub fn remove_notification(&mut self, clsid: [u8; 16], id: u32) -> NotificationResult {
        if self.notifications.remove(&(clsid, id)).is_some() {
            NotificationResult::Success
        } else {
            NotificationResult::InvalidData
        }
    }

    pub fn notification(&self, clsid: [u8; 16], id: u32) -> Option<&ShellNotificationRecord> {
        self.notifications.get(&(clsid, id))
    }

    pub fn notifications(&self) -> impl Iterator<Item = &ShellNotificationRecord> {
        self.notifications.values()
    }

    pub fn remove_window_state(&mut self, hwnd: u32) -> ShellWindowCleanup {
        let before_icons = self.notify_icons.len();
        self.notify_icons
            .retain(|(_icon_hwnd, _id), record| record.hwnd != hwnd);
        let before_notifications = self.notifications.len();
        self.notifications
            .retain(|(_clsid, _id), record| record.hwnd_sink != hwnd);
        ShellWindowCleanup {
            notify_icons_removed: before_icons.saturating_sub(self.notify_icons.len()),
            notifications_removed: before_notifications.saturating_sub(self.notifications.len()),
        }
    }

    pub fn remove_windows_state<'a>(
        &mut self,
        hwnds: impl IntoIterator<Item = &'a u32>,
    ) -> ShellWindowCleanup {
        let mut cleanup = ShellWindowCleanup::default();
        for hwnd in hwnds {
            cleanup += self.remove_window_state(*hwnd);
        }
        cleanup
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationResult {
    Success,
    InvalidParameter,
    InvalidData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellNotificationData {
    pub id: u32,
    pub priority: u32,
    pub duration_cs: u32,
    pub icon: u32,
    pub flags: u32,
    pub clsid: [u8; 16],
    pub hwnd_sink: u32,
    pub title: String,
    pub html: String,
    pub lparam: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellNotificationRecord {
    pub id: u32,
    pub priority: u32,
    pub duration_cs: u32,
    pub icon: u32,
    pub flags: u32,
    pub clsid: [u8; 16],
    pub hwnd_sink: u32,
    pub title: String,
    pub html: String,
    pub lparam: u32,
}

impl ShellNotificationRecord {
    fn from_data(data: ShellNotificationData) -> Self {
        let duration_cs = defaulted_notification_duration(data.priority, data.duration_cs);
        Self {
            id: data.id,
            priority: data.priority,
            duration_cs,
            icon: data.icon,
            flags: data.flags,
            clsid: data.clsid,
            hwnd_sink: data.hwnd_sink,
            title: data.title,
            html: data.html,
            lparam: data.lparam,
        }
    }

    fn update(&mut self, update_mask: u32, data: ShellNotificationData) {
        let duration_priority = if update_mask & SHNUM_PRIORITY != 0 {
            data.priority
        } else {
            self.priority
        };
        if update_mask & SHNUM_PRIORITY != 0 {
            self.priority = data.priority;
        }
        if update_mask & SHNUM_DURATION != 0 {
            self.duration_cs = defaulted_notification_duration(duration_priority, data.duration_cs);
        }
        if update_mask & SHNUM_ICON != 0 {
            self.icon = data.icon;
        }
        if update_mask & SHNUM_HTML != 0 {
            self.html = data.html;
        }
        if update_mask & SHNUM_TITLE != 0 {
            self.title = data.title;
        }
        self.flags = update_mask;
        self.hwnd_sink = data.hwnd_sink;
        self.lparam = data.lparam;
    }
}

fn is_notification_priority_valid(priority: u32) -> bool {
    priority == SHNP_INFORM || priority == SHNP_ICONIC
}

fn defaulted_notification_duration(priority: u32, duration_cs: u32) -> u32 {
    if duration_cs != 0 {
        duration_cs
    } else if priority == SHNP_ICONIC {
        u32::MAX
    } else {
        5
    }
}

pub const SHNP_INFORM: u32 = 0x0000_01b1;
pub const SHNP_ICONIC: u32 = 0x0000_01b2;
pub const SHNUM_PRIORITY: u32 = 0x0000_0001;
pub const SHNUM_DURATION: u32 = 0x0000_0002;
pub const SHNUM_ICON: u32 = 0x0000_0004;
pub const SHNUM_HTML: u32 = 0x0000_0008;
pub const SHNUM_TITLE: u32 = 0x0000_0010;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ShellWindowCleanup {
    pub notify_icons_removed: usize,
    pub notifications_removed: usize,
}

impl std::ops::AddAssign for ShellWindowCleanup {
    fn add_assign(&mut self, rhs: Self) {
        self.notify_icons_removed += rhs.notify_icons_removed;
        self.notifications_removed += rhs.notifications_removed;
    }
}
