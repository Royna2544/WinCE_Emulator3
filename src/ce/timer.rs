use std::{collections::BTreeMap, time::Instant};

use super::object::ThreadBirthKey;

#[derive(Debug, Clone)]
pub struct TimerSystem {
    boot: Instant,
    virtual_elapsed_ms: u64,
    next_timer: u32,
    last_timer_instance: u64,
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
    pub owner_birth: Option<ThreadBirthKey>,
    /// Changes on each explicit SetTimer, including replacement of the same ID.
    pub instance: u64,
    pub hwnd: Option<u32>,
    pub message: u32,
    pub callback: Option<u32>,
    pub due_ms: u32,
    pub period_ms: Option<u32>,
    pub pending_message: bool,
}

/// Host-only provenance for a WM_TIMER posted by an exact thread timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimerMessageOrigin {
    pub birth: ThreadBirthKey,
    pub id: u32,
    pub instance: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct TimerKey {
    thread_id: u32,
    owner_birth: Option<ThreadBirthKey>,
    hwnd: Option<u32>,
    id: u32,
}

impl TimerKey {
    fn for_timer(timer: &KernelTimer) -> Self {
        Self {
            thread_id: timer.thread_id,
            owner_birth: timer.owner_birth,
            hwnd: timer.hwnd,
            id: timer.id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CeSleepRequest, INFINITE, TimerMessageOrigin, TimerSystem, ce_sleep_request};
    use crate::ce::object::{HandleTable, ThreadBirthKey};

    fn reused_thread_births(thread_id: u32) -> (ThreadBirthKey, ThreadBirthKey) {
        let mut handles = HandleTable::default();
        let old_handle = handles.try_create_thread(thread_id, 0, 0, false).unwrap();
        let old = handles.thread_birth_key_for_handle(old_handle).unwrap();
        assert!(handles.mark_thread_exited_by_birth(old, 0));
        handles.close(old_handle).unwrap();
        let new_handle = handles.try_create_thread(thread_id, 0, 0, false).unwrap();
        let new = handles.thread_birth_key_for_handle(new_handle).unwrap();
        assert_ne!(old, new);
        (old, new)
    }

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

    #[test]
    fn debug_text_lists_pending_timer_state() {
        let mut timers = TimerSystem::default();
        timers.set_timer(1, Some(0x20008), Some(42), 1000, 0x0113, Some(0x1234));

        let debug = timers.debug_text();

        assert!(debug.contains("timers: now_ms="));
        assert!(debug.contains("count=1"));
        assert!(debug.contains("hwnd=0x00020008"));
        assert!(debug.contains("id=42"));
        assert!(debug.contains("msg=0x0113"));
        assert!(debug.contains("callback=0x00001234"));
        assert!(debug.contains("pending_message=false"));
    }

    #[test]
    fn exact_thread_timers_survive_only_their_own_birth() {
        let (old, new) = reused_thread_births(45);
        let mut timers = TimerSystem::default();
        assert_eq!(
            timers.set_thread_timer_for_birth(old, Some(7), 1, 0x0113, None),
            7
        );
        assert_eq!(
            timers.set_thread_timer_for_birth(new, Some(7), 1, 0x0113, None),
            7
        );
        assert_eq!(timers.set_timer(45, None, Some(7), 1, 0x0113, None), 7);

        assert!(!timers.kill_thread_timer_for_birth(old, 8));
        assert_eq!(timers.remove_thread_timers_for_birth(old), 1);
        assert!(!timers.kill_thread_timer_for_birth(old, 7));
        assert_eq!(timers.timer_count(), 2);
        assert!(timers.kill_timer(45, None, 7));
        assert!(timers.kill_thread_timer_for_birth(new, 7));
        assert_eq!(timers.timer_count(), 0);
    }

    #[test]
    fn replaced_timer_rejects_stale_due_and_message_origins() {
        let (birth, _) = reused_thread_births(46);
        let mut timers = TimerSystem::default();
        assert_eq!(
            timers.set_thread_timer_for_birth(birth, Some(9), 1, 0x0113, None),
            9
        );
        timers.sleep_ms(1);
        let stale_candidate = timers.due_candidates().pop().unwrap();
        assert_eq!(
            timers.set_thread_timer_for_birth(birth, Some(9), 1, 0x0113, None),
            9
        );
        assert!(!timers.activate_due_timer(&stale_candidate));
        assert!(!timers.cancel_timer_instance(&stale_candidate));
        timers.sleep_ms(1);
        let current = timers.due_candidates().pop().unwrap();
        assert_ne!(stale_candidate.instance, current.instance);
        assert!(timers.activate_due_timer(&current));
        assert!(!timers.clear_pending_message(46, None, 9));
        assert!(!timers.clear_pending_for_origin(TimerMessageOrigin {
            birth,
            id: 9,
            instance: stale_candidate.instance,
        }));
        assert!(timers.pending_timers()[0].pending_message);
        assert!(timers.clear_pending_for_origin(TimerMessageOrigin {
            birth,
            id: 9,
            instance: current.instance,
        }));
        assert!(!timers.pending_timers()[0].pending_message);
    }

    #[test]
    fn timer_instance_overflow_refuses_set_without_mutation() {
        let (birth, _) = reused_thread_births(47);
        let mut timers = TimerSystem::default();
        assert_eq!(
            timers.set_thread_timer_for_birth(birth, Some(3), 1, 0x0113, None),
            3
        );
        let before = timers.pending_timers();
        let next_id = timers.next_timer;
        timers.last_timer_instance = u64::MAX;
        assert_eq!(
            timers.set_thread_timer_for_birth(birth, Some(3), 2, 0x0113, None),
            0
        );
        assert_eq!(
            timers.set_thread_timer_for_birth(birth, None, 2, 0x0113, None),
            0
        );
        assert_eq!(timers.next_timer, next_id);
        assert_eq!(timers.pending_timers().len(), before.len());
        assert_eq!(timers.pending_timers()[0].instance, before[0].instance);
        assert_eq!(timers.pending_timers()[0].period_ms, before[0].period_ms);
    }
}

impl Default for TimerSystem {
    fn default() -> Self {
        Self {
            boot: Instant::now(),
            virtual_elapsed_ms: 0,
            next_timer: 1,
            last_timer_instance: 0,
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
        self.set_timer_with_owner(
            thread_id,
            None,
            hwnd,
            requested_id,
            period_ms,
            message,
            callback,
        )
    }

    pub fn set_thread_timer_for_birth(
        &mut self,
        birth: ThreadBirthKey,
        requested_id: Option<u32>,
        period_ms: u32,
        message: u32,
        callback: Option<u32>,
    ) -> u32 {
        self.set_timer_with_owner(
            birth.thread_id(),
            Some(birth),
            None,
            requested_id,
            period_ms,
            message,
            callback,
        )
    }

    fn set_timer_with_owner(
        &mut self,
        thread_id: u32,
        owner_birth: Option<ThreadBirthKey>,
        hwnd: Option<u32>,
        requested_id: Option<u32>,
        period_ms: u32,
        message: u32,
        callback: Option<u32>,
    ) -> u32 {
        // Refuse overflow before changing either the timer ID cursor or its map.
        let Some(instance) = self.last_timer_instance.checked_add(1) else {
            return 0;
        };
        let id = requested_id.unwrap_or_else(|| {
            let mut id = self.next_timer;
            while self.timers.contains_key(&TimerKey {
                thread_id,
                owner_birth,
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
            owner_birth,
            hwnd,
            id,
        };
        self.last_timer_instance = instance;
        self.timers.insert(
            key,
            KernelTimer {
                id,
                thread_id,
                owner_birth,
                instance,
                hwnd,
                message,
                callback,
                due_ms,
                period_ms: Some(period_ms),
                pending_message: false,
            },
        );
        id
    }

    pub fn kill_timer(&mut self, thread_id: u32, hwnd: Option<u32>, id: u32) -> bool {
        self.timers
            .remove(&TimerKey {
                thread_id,
                owner_birth: None,
                hwnd,
                id,
            })
            .is_some()
    }

    pub fn kill_thread_timer_for_birth(&mut self, birth: ThreadBirthKey, id: u32) -> bool {
        self.timers
            .remove(&TimerKey {
                thread_id: birth.thread_id(),
                owner_birth: Some(birth),
                hwnd: None,
                id,
            })
            .is_some()
    }

    pub fn remove_thread_timers_for_birth(&mut self, birth: ThreadBirthKey) -> usize {
        let doomed: Vec<TimerKey> = self
            .timers
            .keys()
            .filter(|key| key.owner_birth == Some(birth) && key.hwnd.is_none())
            .copied()
            .collect();
        let removed = doomed.len();
        for key in doomed {
            self.timers.remove(&key);
        }
        removed
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

    pub fn debug_text(&self) -> String {
        use std::fmt::Write as _;

        let now = self.tick_count();
        let pending = self.pending_timers();
        let next_due = self.next_due_delay_ms();
        let mut out = String::new();
        let _ = writeln!(
            &mut out,
            "  timers: now_ms={now} count={} next_due_ms={}",
            pending.len(),
            next_due
                .map(|delay| delay.to_string())
                .unwrap_or_else(|| "none".to_owned())
        );
        if pending.is_empty() {
            out.push_str("    none\n");
            return out;
        }
        for timer in pending {
            let delay_ms = timer.due_ms.saturating_sub(now);
            let _ = writeln!(
                &mut out,
                "    tid={} hwnd={} id={} msg=0x{:04x} callback={} due_ms={} delay_ms={} period_ms={} pending_message={}",
                timer.thread_id,
                timer
                    .hwnd
                    .map(|hwnd| format!("0x{hwnd:08x}"))
                    .unwrap_or_else(|| "none".to_owned()),
                timer.id,
                timer.message,
                timer
                    .callback
                    .map(|callback| format!("0x{callback:08x}"))
                    .unwrap_or_else(|| "none".to_owned()),
                timer.due_ms,
                delay_ms,
                timer
                    .period_ms
                    .map(|period| period.to_string())
                    .unwrap_or_else(|| "none".to_owned()),
                timer.pending_message
            );
        }
        out
    }

    pub fn next_due_delay_ms(&self) -> Option<u32> {
        let now = self.tick_count();
        self.timers
            .values()
            .filter(|timer| !timer.pending_message)
            .map(|timer| timer.due_ms.saturating_sub(now))
            .min()
    }

    pub fn due_candidates(&self) -> Vec<KernelTimer> {
        let now = self.tick_count();
        self.timers
            .values()
            .filter(|timer| !timer.pending_message && timer.due_ms <= now)
            .cloned()
            .collect()
    }

    /// Commit one previously observed due candidate only if that timer has not
    /// been replaced, cancelled, or consumed in the meantime.
    pub fn activate_due_timer(&mut self, candidate: &KernelTimer) -> bool {
        let key = TimerKey::for_timer(candidate);
        let now = self.tick_count();
        let Some(current) = self.timers.get(&key) else {
            return false;
        };
        if current.instance != candidate.instance
            || current.due_ms != candidate.due_ms
            || current.pending_message
            || current.due_ms > now
        {
            return false;
        }
        if let Some(period) = current.period_ms {
            let current = self.timers.get_mut(&key).expect("validated timer key");
            current.due_ms = now.wrapping_add(period);
            current.pending_message = true;
        } else {
            self.timers.remove(&key);
        }
        true
    }

    pub fn cancel_timer_instance(&mut self, candidate: &KernelTimer) -> bool {
        let key = TimerKey::for_timer(candidate);
        if self
            .timers
            .get(&key)
            .is_none_or(|current| current.instance != candidate.instance)
        {
            return false;
        }
        self.timers.remove(&key).is_some()
    }

    pub fn due_timers(&mut self) -> Vec<KernelTimer> {
        self.due_candidates()
            .into_iter()
            .filter_map(|candidate| {
                self.activate_due_timer(&candidate).then(|| {
                    self.timers
                        .get(&TimerKey::for_timer(&candidate))
                        .cloned()
                        .unwrap_or(candidate)
                })
            })
            .collect()
    }

    pub fn clear_pending_message(&mut self, thread_id: u32, hwnd: Option<u32>, id: u32) -> bool {
        let Some(timer) = self.timers.get_mut(&TimerKey {
            thread_id,
            owner_birth: None,
            hwnd,
            id,
        }) else {
            return false;
        };
        let was_pending = timer.pending_message;
        timer.pending_message = false;
        was_pending
    }

    pub fn clear_pending_for_origin(&mut self, origin: TimerMessageOrigin) -> bool {
        let Some(timer) = self.timers.get_mut(&TimerKey {
            thread_id: origin.birth.thread_id(),
            owner_birth: Some(origin.birth),
            hwnd: None,
            id: origin.id,
        }) else {
            return false;
        };
        if timer.instance != origin.instance {
            return false;
        }
        let was_pending = timer.pending_message;
        timer.pending_message = false;
        was_pending
    }
}
