pub const KBDI_VKEY_TO_UNICODE_INFO_ID: u32 = 0;
pub const KBDI_AUTOREPEAT_INFO_ID: u32 = 1;
pub const KBDI_AUTOREPEAT_SELECTIONS_INFO_ID: u32 = 2;
pub const KBDI_KEYBOARD_STATUS_ID: u32 = 3;

pub const KBD_AUTO_REPEAT_INITIAL_DELAY_DEFAULT: i32 = 500;
pub const KBD_AUTO_REPEAT_INITIAL_DELAY_MIN: i32 = 250;
pub const KBD_AUTO_REPEAT_INITIAL_DELAY_MAX: i32 = 1000;
pub const KBD_AUTO_REPEAT_KEYS_PER_SEC_DEFAULT: i32 = 20;
pub const KBD_AUTO_REPEAT_KEYS_PER_SEC_MIN: i32 = 2;
pub const KBD_AUTO_REPEAT_KEYS_PER_SEC_MAX: i32 = 30;

pub const KBDI_KEYBOARD_PRESENT: u32 = 0x0001;
pub const KBDI_KEYBOARD_ENABLED: u32 = 0x0002;
pub const KBDI_KEYBOARD_ENTER_ESC: u32 = 0x0004;
pub const KBDI_KEYBOARD_ALPHA_NUM: u32 = 0x0008;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeybdVKeyToUnicodeInfo {
    pub cb_to_unicode_state: u32,
    pub max_to_unicode_characters: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeybdAutoRepeatInfo {
    pub current_initial_delay: i32,
    pub current_repeat_rate: i32,
    pub initial_delays_selectable: i32,
    pub repeat_rates_selectable: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeybdAutoRepeatSelectionsInfo {
    pub min_initial_delay: i32,
    pub max_initial_delay: i32,
    pub min_repeat_rate: i32,
    pub max_repeat_rate: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeybdSystem {
    current_initial_delay: i32,
    current_repeat_rate: i32,
    status: u32,
}

impl Default for KeybdSystem {
    fn default() -> Self {
        Self {
            current_initial_delay: KBD_AUTO_REPEAT_INITIAL_DELAY_DEFAULT,
            current_repeat_rate: KBD_AUTO_REPEAT_KEYS_PER_SEC_DEFAULT,
            status: KBDI_KEYBOARD_PRESENT
                | KBDI_KEYBOARD_ENABLED
                | KBDI_KEYBOARD_ENTER_ESC
                | KBDI_KEYBOARD_ALPHA_NUM,
        }
    }
}

impl KeybdSystem {
    pub fn vkey_to_unicode_info(&self) -> KeybdVKeyToUnicodeInfo {
        KeybdVKeyToUnicodeInfo {
            cb_to_unicode_state: 0,
            max_to_unicode_characters: 1,
        }
    }

    pub fn autorepeat_info(&self) -> KeybdAutoRepeatInfo {
        KeybdAutoRepeatInfo {
            current_initial_delay: self.current_initial_delay,
            current_repeat_rate: self.current_repeat_rate,
            initial_delays_selectable: -1,
            repeat_rates_selectable: -1,
        }
    }

    pub fn autorepeat_selections_info(&self) -> KeybdAutoRepeatSelectionsInfo {
        KeybdAutoRepeatSelectionsInfo {
            min_initial_delay: KBD_AUTO_REPEAT_INITIAL_DELAY_MIN,
            max_initial_delay: KBD_AUTO_REPEAT_INITIAL_DELAY_MAX,
            min_repeat_rate: KBD_AUTO_REPEAT_KEYS_PER_SEC_MIN,
            max_repeat_rate: KBD_AUTO_REPEAT_KEYS_PER_SEC_MAX,
        }
    }

    pub fn status(&self) -> u32 {
        self.status
    }
}
