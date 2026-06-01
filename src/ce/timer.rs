use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub struct TimerSystem {
    boot: Instant,
    next_timer: u32,
    timers: BTreeMap<u32, KernelTimer>,
}

pub const INFINITE: u32 = 0xffff_ffff;
pub const WAIT_OBJECT_0: u32 = 0;
pub const WAIT_TIMEOUT: u32 = 258;
pub const WAIT_FAILED: u32 = 0xffff_ffff;

#[derive(Debug, Clone)]
pub struct KernelTimer {
    pub id: u32,
    pub hwnd: Option<u32>,
    pub message: u32,
    pub due_ms: u32,
    pub period_ms: Option<u32>,
}

impl Default for TimerSystem {
    fn default() -> Self {
        Self {
            boot: Instant::now(),
            next_timer: 1,
            timers: BTreeMap::new(),
        }
    }
}

impl TimerSystem {
    pub fn tick_count(&self) -> u32 {
        self.boot.elapsed().as_millis() as u32
    }

    pub fn sleep_ms(&self, ms: u32) {
        std::thread::sleep(Duration::from_millis(u64::from(ms)));
    }

    pub fn set_timer(
        &mut self,
        hwnd: Option<u32>,
        requested_id: Option<u32>,
        period_ms: u32,
        message: u32,
    ) -> u32 {
        let id = requested_id.unwrap_or_else(|| {
            let id = self.next_timer;
            self.next_timer += 1;
            id
        });
        let due_ms = self.tick_count().wrapping_add(period_ms);
        self.timers.insert(
            id,
            KernelTimer {
                id,
                hwnd,
                message,
                due_ms,
                period_ms: Some(period_ms),
            },
        );
        id
    }

    pub fn kill_timer(&mut self, id: u32) -> bool {
        self.timers.remove(&id).is_some()
    }

    pub fn timer_count(&self) -> usize {
        self.timers.len()
    }

    pub fn due_timers(&mut self) -> Vec<KernelTimer> {
        let now = self.tick_count();
        let due_ids: Vec<u32> = self
            .timers
            .iter()
            .filter_map(|(id, timer)| (timer.due_ms <= now).then_some(*id))
            .collect();

        let mut due = Vec::new();
        for id in due_ids {
            if let Some(mut timer) = self.timers.remove(&id) {
                if let Some(period) = timer.period_ms {
                    timer.due_ms = now.wrapping_add(period);
                    self.timers.insert(id, timer.clone());
                }
                due.push(timer);
            }
        }
        due
    }
}
