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
    pub destroy_icon: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ShellSystem {
    notify_icons: BTreeMap<(u32, u32), NotifyIconRecord>,
    destroyed_notify_icons: Vec<u32>,
    notifications: BTreeMap<([u8; 16], u32), ShellNotificationRecord>,
    notification_callbacks: Vec<ShellNotificationCallbackRecord>,
    change_notifications: BTreeMap<u32, ShellChangeNotifyRegistration>,
    freed_file_notifications: Vec<u32>,
    message_boxes: Vec<MessageBoxRecord>,
    recent_documents: Vec<RecentDocumentRecord>,
    special_folder_queries: Vec<ShellSpecialFolderRecord>,
    special_folder_fallback_policy: ShellSpecialFolderFallbackPolicy,
}

pub const MAX_RECENT_DOCUMENTS: usize = 10;

impl ShellSystem {
    pub fn record_message_box(&mut self, record: MessageBoxRecord) {
        self.message_boxes.push(record);
    }

    pub fn last_message_box(&self) -> Option<&MessageBoxRecord> {
        self.message_boxes.last()
    }

    pub fn message_boxes(&self) -> impl Iterator<Item = &MessageBoxRecord> {
        self.message_boxes.iter()
    }

    pub fn record_recent_document(
        &mut self,
        record: RecentDocumentRecord,
    ) -> Vec<RecentDocumentRecord> {
        self.recent_documents
            .retain(|existing| existing.shortcut_path != record.shortcut_path);
        self.recent_documents.insert(0, record);
        if self.recent_documents.len() > MAX_RECENT_DOCUMENTS {
            self.recent_documents.split_off(MAX_RECENT_DOCUMENTS)
        } else {
            Vec::new()
        }
    }

    pub fn clear_recent_documents(&mut self) {
        self.recent_documents.clear();
    }

    pub fn recent_documents(&self) -> impl Iterator<Item = &RecentDocumentRecord> {
        self.recent_documents.iter()
    }

    pub fn recent_document_display_entries(
        &self,
    ) -> impl Iterator<Item = RecentDocumentDisplayEntry<'_>> {
        self.recent_documents.iter().map(|record| {
            let label = if record.display_name.is_empty() {
                &record.target_path
            } else {
                &record.display_name
            };
            RecentDocumentDisplayEntry {
                label,
                shortcut_path: &record.shortcut_path,
                target_path: &record.target_path,
                flags: record.flags,
                has_namespace_pidl: record.pidl_bytes.is_some(),
            }
        })
    }

    pub fn record_special_folder_query(&mut self, record: ShellSpecialFolderRecord) {
        self.special_folder_queries.push(record);
    }

    pub fn special_folder_queries(&self) -> impl Iterator<Item = &ShellSpecialFolderRecord> {
        self.special_folder_queries.iter()
    }

    pub fn set_special_folder_fallback_policy(&mut self, policy: ShellSpecialFolderFallbackPolicy) {
        self.special_folder_fallback_policy = policy;
    }

    pub fn special_folder_fallback_policy(&self) -> ShellSpecialFolderFallbackPolicy {
        self.special_folder_fallback_policy
    }

    pub fn register_change_notification(&mut self, registration: ShellChangeNotifyRegistration) {
        self.change_notifications
            .insert(registration.hwnd, registration);
    }

    pub fn remove_change_notification(
        &mut self,
        hwnd: u32,
    ) -> Option<ShellChangeNotifyRegistration> {
        self.change_notifications.remove(&hwnd)
    }

    pub fn change_notification(&self, hwnd: u32) -> Option<&ShellChangeNotifyRegistration> {
        self.change_notifications.get(&hwnd)
    }

    pub fn change_notifications(&self) -> impl Iterator<Item = &ShellChangeNotifyRegistration> {
        self.change_notifications.values()
    }

    pub fn record_freed_file_notification(&mut self, ptr: u32) {
        if ptr != 0 {
            self.freed_file_notifications.push(ptr);
        }
    }

    pub fn freed_file_notifications(&self) -> impl Iterator<Item = &u32> {
        self.freed_file_notifications.iter()
    }

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
                if let Some(icon) = record.update(data) {
                    self.destroyed_notify_icons.push(icon);
                }
                true
            }
            NotifyIconOp::Delete => {
                let Some(record) = self.notify_icons.remove(&key) else {
                    return false;
                };
                self.record_notify_icon_destroy(record);
                true
            }
        }
    }

    pub fn notify_icon(&self, hwnd: u32, id: u32) -> Option<&NotifyIconRecord> {
        self.notify_icons.get(&(hwnd, id))
    }

    pub fn notify_icons(&self) -> impl Iterator<Item = &NotifyIconRecord> {
        self.notify_icons.values()
    }

    pub fn destroyed_notify_icons(&self) -> impl Iterator<Item = &u32> {
        self.destroyed_notify_icons.iter()
    }

    pub fn add_notification(
        &mut self,
        data: ShellNotificationData,
        now_ms: u32,
    ) -> NotificationResult {
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
            .insert(key, ShellNotificationRecord::from_data(data, now_ms));
        NotificationResult::Success
    }

    pub fn update_notification(
        &mut self,
        update_mask: u32,
        data: ShellNotificationData,
        now_ms: u32,
    ) -> NotificationResult {
        if update_mask == 0 || update_mask & !SHNUM_VALID_MASK != 0 {
            return NotificationResult::InvalidData;
        }
        if update_mask & SHNUM_PRIORITY != 0 && !is_notification_priority_valid(data.priority) {
            return NotificationResult::InvalidParameter;
        }
        let Some(record) = self.notifications.get_mut(&(data.clsid, data.id)) else {
            return NotificationResult::InvalidData;
        };
        record.update(update_mask, data, now_ms);
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

    pub fn record_notification_callback(&mut self, record: ShellNotificationCallbackRecord) {
        self.notification_callbacks.push(record);
    }

    pub fn notification_callbacks(&self) -> impl Iterator<Item = &ShellNotificationCallbackRecord> {
        self.notification_callbacks.iter()
    }

    pub fn expire_notifications(&mut self, now_ms: u32) -> Vec<ShellNotificationRecord> {
        let expired = self
            .notifications
            .iter()
            .filter_map(|(key, record)| {
                record
                    .expires_at_ms
                    .is_some_and(|expires_at| expires_at <= now_ms)
                    .then_some(*key)
            })
            .collect::<Vec<_>>();
        expired
            .into_iter()
            .filter_map(|key| self.notifications.remove(&key))
            .collect()
    }

    pub fn remove_window_state(&mut self, hwnd: u32) -> ShellWindowCleanup {
        let notify_icon_keys = self
            .notify_icons
            .iter()
            .filter_map(|(key, record)| (record.hwnd == hwnd).then_some(*key))
            .collect::<Vec<_>>();
        let notify_icons_removed = notify_icon_keys.len();
        for key in notify_icon_keys {
            if let Some(record) = self.notify_icons.remove(&key) {
                self.record_notify_icon_destroy(record);
            }
        }
        let before_notifications = self.notifications.len();
        self.notifications
            .retain(|(_clsid, _id), record| record.hwnd_sink != hwnd);
        let before_change_notifications = self.change_notifications.len();
        self.change_notifications
            .retain(|_registered_hwnd, record| record.hwnd != hwnd);
        ShellWindowCleanup {
            notify_icons_removed,
            notifications_removed: before_notifications.saturating_sub(self.notifications.len()),
            change_notifications_removed: before_change_notifications
                .saturating_sub(self.change_notifications.len()),
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

    fn record_notify_icon_destroy(&mut self, record: NotifyIconRecord) {
        if record.destroy_icon && record.icon != 0 {
            self.destroyed_notify_icons.push(record.icon);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentDocumentRecord {
    pub flags: u32,
    pub target_path: String,
    pub display_name: String,
    pub shortcut_path: String,
    pub pidl_bytes: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecentDocumentDisplayEntry<'a> {
    pub label: &'a str,
    pub shortcut_path: &'a str,
    pub target_path: &'a str,
    pub flags: u32,
    pub has_namespace_pidl: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellSpecialFolderSource {
    Registry,
    FallbackMissing,
    FallbackNonString,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ShellSpecialFolderFallbackPolicy {
    #[default]
    Compat,
    Strict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellSpecialFolderRecord {
    pub csidl: u32,
    pub value_name: String,
    pub path: String,
    pub source: ShellSpecialFolderSource,
    pub create_requested: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellChangeNotifyRegistration {
    pub hwnd: u32,
    pub event_mask: u32,
    pub notify_flags: u32,
    pub watch_dir: Option<String>,
    pub recursive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageBoxRecord {
    pub thread_id: u32,
    pub owner_hwnd: u32,
    pub dialog_hwnd: u32,
    pub text_hwnd: u32,
    pub text: String,
    pub caption: String,
    pub style: u32,
    pub buttons: Vec<u32>,
    pub button_hwnds: Vec<u32>,
    pub button_layout: Vec<MessageBoxButton>,
    pub default_button_index: usize,
    pub icon: Option<MessageBoxIcon>,
    pub result: u32,
    pub owner_was_enabled: Option<bool>,
    pub rendered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageBoxButton {
    pub id: u32,
    pub label: MessageBoxButtonLabel,
    pub slot: MessageBoxButtonSlot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageBoxButtonLabel {
    Ok,
    Cancel,
    Abort,
    Retry,
    Ignore,
    Yes,
    No,
    YesAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageBoxButtonSlot {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageBoxIcon {
    Hand,
    Question,
    Exclamation,
    Asterisk,
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
            destroy_icon: data.flags & HHTBF_DESTROYICON != 0,
        }
    }

    fn update(&mut self, data: NotifyIconData) -> Option<u32> {
        let mut destroyed_icon = None;
        if data.flags & NIF_MESSAGE != 0 {
            self.callback_message = data.callback_message;
        }
        if data.flags & NIF_ICON != 0 {
            if self.destroy_icon && self.icon != 0 && data.icon != 0 && self.icon != data.icon {
                destroyed_icon = Some(self.icon);
            }
            self.icon = data.icon;
        }
        if data.flags & NIF_TIP != 0 {
            self.tip = data.tip;
        }
        if data.flags & NIF_STATE != 0 {
            self.state = (self.state & !data.state_mask) | (data.state & data.state_mask);
        }
        if data.flags & HHTBF_DESTROYICON != 0 {
            self.destroy_icon = true;
        }
        destroyed_icon
    }
}

pub const NIF_MESSAGE: u32 = 0x0000_0001;
pub const NIF_ICON: u32 = 0x0000_0002;
pub const NIF_TIP: u32 = 0x0000_0004;
pub const NIF_STATE: u32 = 0x0000_0008;
pub const HHTBF_DESTROYICON: u32 = 0x1000_0000;
pub const SHNN_SHOW: u32 = 0xffff_fc16;
pub const SHNN_DISMISS: u32 = 0xffff_fc17;
pub const SHNN_LINKSEL: u32 = 0xffff_fc18;
pub const ISHELL_NOTIFICATION_CALLBACK_ON_SHOW_VTABLE_OFFSET: u32 = 0x0c;
pub const ISHELL_NOTIFICATION_CALLBACK_ON_COMMAND_SELECTED_VTABLE_OFFSET: u32 = 0x10;
pub const ISHELL_NOTIFICATION_CALLBACK_ON_LINK_SELECTED_VTABLE_OFFSET: u32 = 0x14;
pub const ISHELL_NOTIFICATION_CALLBACK_ON_DISMISS_VTABLE_OFFSET: u32 = 0x18;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellNotificationCallbackRecord {
    pub clsid: [u8; 16],
    pub id: u32,
    pub method: ShellNotificationCallbackMethod,
    pub vtable_offset: u32,
    pub arguments: ShellNotificationCallbackArguments,
    pub lparam: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellNotificationCallbackMethod {
    OnShow { x: u32, y: u32 },
    OnCommandSelected { command_id: u32 },
    OnLinkSelected { link: String },
    OnDismiss { timed_out: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellNotificationCallbackArguments {
    OnShow {
        id: u32,
        x: u32,
        y: u32,
        lparam: u32,
    },
    OnCommandSelected {
        id: u32,
        command_id: u32,
    },
    OnLinkSelected {
        id: u32,
        link: String,
        lparam: u32,
    },
    OnDismiss {
        id: u32,
        timed_out: bool,
        lparam: u32,
    },
}

impl ShellNotificationCallbackMethod {
    pub fn com_vtable_offset(&self) -> u32 {
        match self {
            Self::OnShow { .. } => ISHELL_NOTIFICATION_CALLBACK_ON_SHOW_VTABLE_OFFSET,
            Self::OnCommandSelected { .. } => {
                ISHELL_NOTIFICATION_CALLBACK_ON_COMMAND_SELECTED_VTABLE_OFFSET
            }
            Self::OnLinkSelected { .. } => {
                ISHELL_NOTIFICATION_CALLBACK_ON_LINK_SELECTED_VTABLE_OFFSET
            }
            Self::OnDismiss { .. } => ISHELL_NOTIFICATION_CALLBACK_ON_DISMISS_VTABLE_OFFSET,
        }
    }

    pub fn com_arguments(&self, id: u32, lparam: u32) -> ShellNotificationCallbackArguments {
        match self {
            Self::OnShow { x, y } => ShellNotificationCallbackArguments::OnShow {
                id,
                x: *x,
                y: *y,
                lparam,
            },
            Self::OnCommandSelected { command_id } => {
                ShellNotificationCallbackArguments::OnCommandSelected {
                    id,
                    command_id: *command_id,
                }
            }
            Self::OnLinkSelected { link } => ShellNotificationCallbackArguments::OnLinkSelected {
                id,
                link: link.clone(),
                lparam,
            },
            Self::OnDismiss { timed_out } => ShellNotificationCallbackArguments::OnDismiss {
                id,
                timed_out: *timed_out,
                lparam,
            },
        }
    }
}

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
    pub added_at_ms: u32,
    pub expires_at_ms: Option<u32>,
    pub icon: u32,
    pub flags: u32,
    pub clsid: [u8; 16],
    pub hwnd_sink: u32,
    pub title: String,
    pub html: String,
    pub lparam: u32,
}

impl ShellNotificationRecord {
    fn from_data(data: ShellNotificationData, now_ms: u32) -> Self {
        let duration_cs = defaulted_notification_duration(data.priority, data.duration_cs);
        let expires_at_ms = notification_expires_at_ms(now_ms, duration_cs);
        Self {
            id: data.id,
            priority: data.priority,
            duration_cs,
            added_at_ms: now_ms,
            expires_at_ms,
            icon: data.icon,
            flags: data.flags,
            clsid: data.clsid,
            hwnd_sink: data.hwnd_sink,
            title: data.title,
            html: data.html,
            lparam: data.lparam,
        }
    }

    fn update(&mut self, update_mask: u32, data: ShellNotificationData, now_ms: u32) {
        if update_mask & SHNUM_PRIORITY != 0 {
            self.priority = data.priority;
        }
        if update_mask & SHNUM_DURATION != 0 {
            self.duration_cs = data.duration_cs;
        }
        self.added_at_ms = now_ms;
        self.expires_at_ms = notification_expires_at_ms(now_ms, self.duration_cs);
        if update_mask & SHNUM_ICON != 0 && data.icon != 0 {
            self.icon = data.icon;
        }
        if update_mask & SHNUM_HTML != 0 {
            self.html = data.html;
        }
        if update_mask & SHNUM_TITLE != 0 {
            self.title = data.title;
        }
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

fn notification_expires_at_ms(now_ms: u32, duration_cs: u32) -> Option<u32> {
    (duration_cs != u32::MAX).then(|| now_ms.saturating_add(duration_cs.saturating_mul(10)))
}

pub const SHNP_INFORM: u32 = 0x0000_01b1;
pub const SHNP_ICONIC: u32 = 0x0000_01b2;
pub const SHNUM_PRIORITY: u32 = 0x0000_0001;
pub const SHNUM_DURATION: u32 = 0x0000_0002;
pub const SHNUM_ICON: u32 = 0x0000_0004;
pub const SHNUM_HTML: u32 = 0x0000_0008;
pub const SHNUM_TITLE: u32 = 0x0000_0010;
const SHNUM_VALID_MASK: u32 =
    SHNUM_PRIORITY | SHNUM_DURATION | SHNUM_ICON | SHNUM_HTML | SHNUM_TITLE;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ShellWindowCleanup {
    pub notify_icons_removed: usize,
    pub notifications_removed: usize,
    pub change_notifications_removed: usize,
}

impl std::ops::AddAssign for ShellWindowCleanup {
    fn add_assign(&mut self, rhs: Self) {
        self.notify_icons_removed += rhs.notify_icons_removed;
        self.notifications_removed += rhs.notifications_removed;
        self.change_notifications_removed += rhs.change_notifications_removed;
    }
}
