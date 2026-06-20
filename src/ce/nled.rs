#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NledSupportsInfo {
    pub led_num: u32,
    pub cycle_adjust: i32,
    pub adjust_total_cycle_time: bool,
    pub adjust_on_time: bool,
    pub adjust_off_time: bool,
    pub meta_cycle_on: bool,
    pub meta_cycle_off: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NledSettingsInfo {
    pub led_num: u32,
    pub off_on_blink: i32,
    pub total_cycle_time: i32,
    pub on_time: i32,
    pub off_time: i32,
    pub meta_cycle_on: i32,
    pub meta_cycle_off: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NledSystem {
    settings: [NledSettingsInfo; 2],
}

impl Default for NledSystem {
    fn default() -> Self {
        Self {
            settings: [
                NledSettingsInfo {
                    led_num: 0,
                    ..Default::default()
                },
                NledSettingsInfo {
                    led_num: 1,
                    ..Default::default()
                },
            ],
        }
    }
}

impl NledSystem {
    pub const COUNT_INFO_ID: u32 = 0;
    pub const SUPPORTS_INFO_ID: u32 = 1;
    pub const SETTINGS_INFO_ID: u32 = 2;

    pub fn led_count(&self) -> u32 {
        self.settings.len() as u32
    }

    pub fn supports_info(&self, led_num: u32) -> Option<NledSupportsInfo> {
        match led_num {
            0 => Some(NledSupportsInfo {
                led_num,
                cycle_adjust: 100,
                adjust_total_cycle_time: false,
                adjust_on_time: true,
                adjust_off_time: true,
                meta_cycle_on: false,
                meta_cycle_off: false,
            }),
            1 => Some(NledSupportsInfo {
                led_num,
                cycle_adjust: -1,
                adjust_total_cycle_time: false,
                adjust_on_time: false,
                adjust_off_time: false,
                meta_cycle_on: false,
                meta_cycle_off: false,
            }),
            _ => None,
        }
    }

    pub fn settings_info(&self, led_num: u32) -> Option<NledSettingsInfo> {
        self.settings.get(led_num as usize).copied()
    }

    pub fn set_device(&mut self, mut info: NledSettingsInfo) -> bool {
        if info.led_num >= self.led_count()
            || info.on_time < 0
            || info.off_time < 0
            || info.off_on_blink < 0
            || info.off_on_blink > 2
            || (info.meta_cycle_on != 0 && info.on_time == 0)
            || info.meta_cycle_off != 0
        {
            return false;
        }

        info.total_cycle_time = info.on_time.saturating_add(info.off_time);
        self.settings[info.led_num as usize] = info;
        true
    }
}
