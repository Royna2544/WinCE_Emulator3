use std::{collections::BTreeMap, time::Instant};

#[derive(Debug, Clone)]
pub struct TimerSystem {
    boot: Instant,
    virtual_elapsed_ms: u64,
    next_timer: u32,
    timers: BTreeMap<TimerKey, KernelTimer>,
}

pub const INFINITE: u32 = 0xffff_ffff;
pub const WAIT_OBJECT_0: u32 = 0;
pub const WAIT_TIMEOUT: u32 = 258;
pub const WAIT_FAILED: u32 = 0xffff_ffff;
const CE_SLEEP_LONG_BOUND: u32 = 0xffff_fffe;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CeSleepRequest {
    Yield,
    Suspend,
    Bounded(u32),
}

pub fn ce_sleep_request(ms: u32) -> CeSleepRequest {
    match ms {
        0 => CeSleepRequest::Yield,
        INFINITE => CeSleepRequest::Suspend,
        1..CE_SLEEP_LONG_BOUND => CeSleepRequest::Bounded(ms + 1),
        CE_SLEEP_LONG_BOUND => CeSleepRequest::Bounded(CE_SLEEP_LONG_BOUND),
    }
}

#[derive(Debug, Clone)]
pub struct KernelTimer {
    pub id: u32,
    pub thread_id: u32,
    pub hwnd: Option<u32>,
    pub message: u32,
    pub callback: Option<u32>,
    pub due_ms: u32,
    pub period_ms: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct TimerKey {
    thread_id: u32,
    hwnd: Option<u32>,
    id: u32,
}

#[cfg(test)]
mod tests {
    use super::{CeSleepRequest, INFINITE, TimerSystem, ce_sleep_request};

    #[test]
    fn ce_sleep_request_matches_nksleep_timeout_shape() {
        assert_eq!(ce_sleep_request(0), CeSleepRequest::Yield);
        assert_eq!(ce_sleep_request(1), CeSleepRequest::Bounded(2));
        assert_eq!(
            ce_sleep_request(0xffff_fffd),
            CeSleepRequest::Bounded(0xffff_fffe)
        );
        assert_eq!(
            ce_sleep_request(0xffff_fffe),
            CeSleepRequest::Bounded(0xffff_fffe)
        );
        assert_eq!(ce_sleep_request(INFINITE), CeSleepRequest::Suspend);
    }

    #[test]
    fn sleep_advances_virtual_timer_clock() {
        let mut timers = TimerSystem::default();
        timers.set_timer(1, Some(0x20004), Some(1000), 7500, 0x0113, None);

        let delay = timers.next_due_delay_ms().unwrap();
        assert!(delay <= 7500);
        timers.sleep_ms(delay);

        assert_eq!(timers.next_due_delay_ms(), Some(0));
        let due = timers.due_timers();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].id, 1000);
        assert_eq!(due[0].thread_id, 1);
    }

    #[test]
    fn timers_with_same_id_are_scoped_by_owner_and_hwnd() {
        let mut timers = TimerSystem::default();
        assert_eq!(
            timers.set_timer(1, Some(0x10001), Some(7), 0, 0x0113, None),
            7
        );
        assert_eq!(
            timers.set_timer(2, Some(0x20001), Some(7), 0, 0x0113, None),
            7
        );
        assert_eq!(timers.set_timer(1, None, Some(7), 0, 0x0113, None), 7);
        assert_eq!(timers.timer_count(), 3);

        assert!(timers.kill_timer(1, Some(0x10001), 7));
        assert_eq!(timers.timer_count(), 2);
        assert!(!timers.kill_timer(1, Some(0x10001), 7));
        assert!(timers.kill_timer(1, None, 7));
        assert!(timers.kill_timer(2, Some(0x20001), 7));
        assert_eq!(timers.timer_count(), 0);
    }

    #[test]
    fn removing_window_timers_keeps_thread_timers_alive() {
        let mut timers = TimerSystem::default();
        timers.set_timer(1, Some(0x10001), Some(7), 0, 0x0113, None);
        timers.set_timer(1, Some(0x10005), Some(8), 0, 0x0113, None);
        timers.set_timer(1, None, Some(7), 0, 0x0113, None);

        assert_eq!(timers.remove_window_timers(&[0x10001, 0x10005]), 2);
        let remaining = timers.pending_timers();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].hwnd, None);
        assert_eq!(remaining[0].id, 7);
    }
}

impl Default for TimerSystem {
    fn default() -> Self {
        Self {
            boot: Instant::now(),
            virtual_elapsed_ms: 0,
            next_timer: 1,
            timers: BTreeMap::new(),
        }
    }
}

impl TimerSystem {
    pub fn performance_frequency(&self) -> u64 {
        1_000
    }

    pub fn performance_counter(&self) -> u64 {
        self.elapsed_ms()
    }

    pub fn tick_count(&self) -> u32 {
        self.elapsed_ms() as u32
    }

    pub fn sleep_ms(&mut self, ms: u32) {
        self.virtual_elapsed_ms = self.virtual_elapsed_ms.saturating_add(u64::from(ms));
    }

    fn elapsed_ms(&self) -> u64 {
        self.boot
            .elapsed()
            .as_millis()
            .try_into()
            .unwrap_or(u64::MAX)
            .saturating_add(self.virtual_elapsed_ms)
    }

    pub fn set_timer(
        &mut self,
        thread_id: u32,
        hwnd: Option<u32>,
        requested_id: Option<u32>,
        period_ms: u32,
        message: u32,
        callback: Option<u32>,
    ) -> u32 {
        let id = requested_id.unwrap_or_else(|| {
            let mut id = self.next_timer;
            while self.timers.contains_key(&TimerKey {
                thread_id,
                hwnd,
                id,
            }) {
                id = id.wrapping_add(1).max(1);
            }
            self.next_timer = id.wrapping_add(1).max(1);
            id
        });
        let due_ms = self.tick_count().wrapping_add(period_ms);
        let key = TimerKey {
            thread_id,
            hwnd,
            id,
        };
        self.timers.insert(
            key,
            KernelTimer {
                id,
                thread_id,
                hwnd,
                message,
                callback,
                due_ms,
                period_ms: Some(period_ms),
            },
        );
        id
    }

    pub fn kill_timer(&mut self, thread_id: u32, hwnd: Option<u32>, id: u32) -> bool {
        self.timers
            .remove(&TimerKey {
                thread_id,
                hwnd,
                id,
            })
            .is_some()
    }

    pub fn remove_window_timers(&mut self, hwnds: &[u32]) -> usize {
        let doomed: Vec<TimerKey> = self
            .timers
            .iter()
            .filter_map(|(key, timer)| {
                timer
                    .hwnd
                    .is_some_and(|hwnd| hwnds.contains(&hwnd))
                    .then_some(*key)
            })
            .collect();
        let removed = doomed.len();
        for key in doomed {
            self.timers.remove(&key);
        }
        removed
    }

    pub fn timer_count(&self) -> usize {
        self.timers.len()
    }

    pub fn pending_timers(&self) -> Vec<KernelTimer> {
        self.timers.values().cloned().collect()
    }

    pub fn next_due_delay_ms(&self) -> Option<u32> {
        let now = self.tick_count();
        self.timers
            .values()
            .map(|timer| timer.due_ms.saturating_sub(now))
            .min()
    }

    pub fn due_timers(&mut self) -> Vec<KernelTimer> {
        let now = self.tick_count();
        let due_keys: Vec<TimerKey> = self
            .timers
            .iter()
            .filter_map(|(key, timer)| (timer.due_ms <= now).then_some(*key))
            .collect();

        let mut due = Vec::new();
        for key in due_keys {
            if let Some(mut timer) = self.timers.remove(&key) {
                if let Some(period) = timer.period_ms {
                    timer.due_ms = now.wrapping_add(period);
                    self.timers.insert(key, timer.clone());
                }
                due.push(timer);
            }
        }
        due
    }
}
