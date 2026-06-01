use crate::{
    ce::{
        audio::AudioSystem, devices::DeviceNamespace, gwe::Gwe, object::HandleTable,
        registry::Registry, timer::TimerSystem,
    },
    config::RuntimeConfig,
};

#[derive(Debug, Clone)]
pub struct CeKernel {
    pub registry: Registry,
    pub devices: DeviceNamespace,
    pub handles: HandleTable,
    pub gwe: Gwe,
    pub audio: AudioSystem,
    pub timers: TimerSystem,
}

impl CeKernel {
    pub fn boot(config: RuntimeConfig) -> Self {
        Self {
            registry: Registry::from_dump(config.registry),
            devices: DeviceNamespace::from_config(config.devices),
            handles: HandleTable::default(),
            gwe: Gwe::default(),
            audio: AudioSystem::default(),
            timers: TimerSystem::default(),
        }
    }

    pub fn pump_timers_to_gwe(&mut self, thread_id: u32) {
        for timer in self.timers.due_timers() {
            if let Some(hwnd) = timer.hwnd {
                let message = crate::ce::gwe::Message::new(
                    hwnd,
                    timer.message,
                    timer.id,
                    0,
                    self.timers.tick_count(),
                );
                self.gwe.post_message(thread_id, message);
            }
        }
    }
}
