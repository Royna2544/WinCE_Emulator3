use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerWaitKind {
    Single,
    Multiple,
    MsgWait,
    Sleep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerWakeReason {
    ObjectSignaled,
    Timeout,
    MessageInput,
    Failed,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SchedulerStats {
    pub wait_single_count: u64,
    pub wait_multiple_count: u64,
    pub msg_wait_count: u64,
    pub sleep_count: u64,
    pub yield_count: u64,
    pub wait_acquired_count: u64,
    pub wait_timeout_count: u64,
    pub wait_failed_count: u64,
    pub wait_block_count: u64,
    pub wait_wake_count: u64,
    pub waiter_register_count: u64,
    pub waiter_remove_count: u64,
    pub object_signal_count: u64,
    pub object_wake_candidate_count: u64,
    pub message_input_signal_count: u64,
    pub message_input_wake_candidate_count: u64,
    pub serial_read_signal_count: u64,
    pub serial_read_wake_candidate_count: u64,
    pub max_wait_handles: u32,
    pub max_timeout_ms: u32,
    pub max_registered_waits: u32,
    pub max_pending_wakes: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerBlockedWaitKind {
    Kernel,
    GetMessage {
        hwnd: Option<u32>,
        min_msg: u32,
        max_msg: u32,
    },
    Sleep,
    MsgWait {
        wake_mask: u32,
        input_available: bool,
    },
    SerialRead {
        handle: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerBlockedWait {
    pub id: u64,
    pub thread_id: u32,
    pub thread_handle: u32,
    pub wait_handles: Vec<u32>,
    pub kind: SchedulerBlockedWaitKind,
    pub wait_started_ms: u32,
    pub timeout_ms: u32,
    sequence: u64,
}

#[derive(Debug, Clone, Default)]
pub struct Scheduler {
    stats: SchedulerStats,
    next_wait_id: u64,
    next_wait_sequence: u64,
    blocked_waits: BTreeMap<u64, SchedulerBlockedWait>,
    wait_queues: BTreeMap<u32, VecDeque<u64>>,
    message_wait_queues: BTreeMap<u32, VecDeque<u64>>,
    serial_read_queues: BTreeMap<u32, VecDeque<u64>>,
    pending_wake_ids: VecDeque<u64>,
    pending_wake_set: BTreeSet<u64>,
}

impl Scheduler {
    pub fn stats(&self) -> SchedulerStats {
        self.stats
    }

    pub fn record_wait_attempt(
        &mut self,
        kind: SchedulerWaitKind,
        handle_count: u32,
        timeout_ms: u32,
    ) {
        match kind {
            SchedulerWaitKind::Single => self.stats.wait_single_count += 1,
            SchedulerWaitKind::Multiple => self.stats.wait_multiple_count += 1,
            SchedulerWaitKind::MsgWait => self.stats.msg_wait_count += 1,
            SchedulerWaitKind::Sleep => self.stats.sleep_count += 1,
        }
        self.stats.max_wait_handles = self.stats.max_wait_handles.max(handle_count);
        self.stats.max_timeout_ms = self.stats.max_timeout_ms.max(timeout_ms);
    }

    pub fn record_wait_result(&mut self, reason: SchedulerWakeReason) {
        match reason {
            SchedulerWakeReason::ObjectSignaled | SchedulerWakeReason::MessageInput => {
                self.stats.wait_acquired_count += 1;
            }
            SchedulerWakeReason::Timeout => self.stats.wait_timeout_count += 1,
            SchedulerWakeReason::Failed => self.stats.wait_failed_count += 1,
        }
    }

    pub fn record_blocked_wait(&mut self) {
        self.stats.wait_block_count += 1;
    }

    pub fn record_thread_yield(&mut self) {
        self.record_wait_attempt(SchedulerWaitKind::Sleep, 0, 0);
        self.stats.yield_count += 1;
    }

    pub fn record_wait_wake(&mut self, reason: SchedulerWakeReason) {
        self.stats.wait_wake_count += 1;
        self.record_wait_result(reason);
    }

    pub fn register_blocked_wait(
        &mut self,
        thread_id: u32,
        thread_handle: u32,
        wait_handles: Vec<u32>,
        kind: SchedulerBlockedWaitKind,
        wait_started_ms: u32,
        timeout_ms: u32,
    ) -> u64 {
        self.next_wait_id = self.next_wait_id.wrapping_add(1).max(1);
        self.next_wait_sequence = self.next_wait_sequence.wrapping_add(1);
        let id = self.next_wait_id;
        let wait = SchedulerBlockedWait {
            id,
            thread_id,
            thread_handle,
            wait_handles,
            kind,
            wait_started_ms,
            timeout_ms,
            sequence: self.next_wait_sequence,
        };
        for handle in &wait.wait_handles {
            self.wait_queues.entry(*handle).or_default().push_back(id);
        }
        if matches!(
            wait.kind,
            SchedulerBlockedWaitKind::GetMessage { .. } | SchedulerBlockedWaitKind::MsgWait { .. }
        ) {
            self.message_wait_queues
                .entry(wait.thread_id)
                .or_default()
                .push_back(id);
        }
        if let SchedulerBlockedWaitKind::SerialRead { handle } = wait.kind {
            self.serial_read_queues
                .entry(handle)
                .or_default()
                .push_back(id);
        }
        self.blocked_waits.insert(id, wait);
        self.stats.waiter_register_count += 1;
        self.stats.max_registered_waits = self
            .stats
            .max_registered_waits
            .max(self.blocked_waits.len() as u32);
        id
    }

    pub fn remove_blocked_wait(&mut self, wait_id: u64) -> Option<SchedulerBlockedWait> {
        let wait = self.blocked_waits.remove(&wait_id)?;
        for handle in &wait.wait_handles {
            let remove_queue = if let Some(queue) = self.wait_queues.get_mut(handle) {
                queue.retain(|id| *id != wait_id);
                queue.is_empty()
            } else {
                false
            };
            if remove_queue {
                self.wait_queues.remove(handle);
            }
        }
        let remove_message_queue =
            if let Some(queue) = self.message_wait_queues.get_mut(&wait.thread_id) {
                queue.retain(|id| *id != wait_id);
                queue.is_empty()
            } else {
                false
            };
        if remove_message_queue {
            self.message_wait_queues.remove(&wait.thread_id);
        }
        if let SchedulerBlockedWaitKind::SerialRead { handle } = wait.kind {
            let remove_serial_queue = if let Some(queue) = self.serial_read_queues.get_mut(&handle)
            {
                queue.retain(|id| *id != wait_id);
                queue.is_empty()
            } else {
                false
            };
            if remove_serial_queue {
                self.serial_read_queues.remove(&handle);
            }
        }
        if self.pending_wake_set.remove(&wait_id) {
            self.pending_wake_ids.retain(|id| *id != wait_id);
        }
        self.stats.waiter_remove_count += 1;
        Some(wait)
    }

    pub fn blocked_wait(&self, wait_id: u64) -> Option<&SchedulerBlockedWait> {
        self.blocked_waits.get(&wait_id)
    }

    pub fn waiter_count(&self) -> usize {
        self.blocked_waits.len()
    }

    pub fn waiter_ids_for_handle(&self, handle: u32) -> Vec<u64> {
        self.wait_queues
            .get(&handle)
            .map(|queue| queue.iter().copied().collect())
            .unwrap_or_default()
    }

    pub fn message_waiter_ids_for_thread(&self, thread_id: u32) -> Vec<u64> {
        self.message_wait_queues
            .get(&thread_id)
            .map(|queue| queue.iter().copied().collect())
            .unwrap_or_default()
    }

    pub fn serial_read_waiter_ids_for_handle(&self, handle: u32) -> Vec<u64> {
        self.serial_read_queues
            .get(&handle)
            .map(|queue| queue.iter().copied().collect())
            .unwrap_or_default()
    }

    pub fn all_serial_read_waiter_ids(&self) -> Vec<u64> {
        self.serial_read_queues
            .values()
            .flat_map(|queue| queue.iter().copied())
            .collect()
    }

    fn enqueue_pending_wake_ids(&mut self, wait_ids: impl IntoIterator<Item = u64>) -> usize {
        let mut queued = 0;
        for wait_id in wait_ids {
            if !self.blocked_waits.contains_key(&wait_id) || !self.pending_wake_set.insert(wait_id)
            {
                continue;
            }
            self.pending_wake_ids.push_back(wait_id);
            queued += 1;
        }
        self.stats.max_pending_wakes = self
            .stats
            .max_pending_wakes
            .max(self.pending_wake_ids.len() as u32);
        queued
    }

    pub fn queue_pending_wake_ids(&mut self, wait_ids: impl IntoIterator<Item = u64>) -> usize {
        self.stats.object_signal_count += 1;
        let queued = self.enqueue_pending_wake_ids(wait_ids);
        self.stats.object_wake_candidate_count += queued as u64;
        queued
    }

    pub fn queue_message_wake_candidates(&mut self, thread_id: u32) -> usize {
        self.stats.message_input_signal_count += 1;
        let wait_ids = self.message_waiter_ids_for_thread(thread_id);
        let queued = self.enqueue_pending_wake_ids(wait_ids);
        self.stats.message_input_wake_candidate_count += queued as u64;
        queued
    }

    pub fn queue_serial_read_wake_candidates(&mut self, handle: u32) -> usize {
        self.stats.serial_read_signal_count += 1;
        let wait_ids = self.serial_read_waiter_ids_for_handle(handle);
        let queued = self.enqueue_pending_wake_ids(wait_ids);
        self.stats.serial_read_wake_candidate_count += queued as u64;
        queued
    }

    pub fn queue_all_serial_read_wake_candidates(&mut self) -> usize {
        self.stats.serial_read_signal_count += 1;
        let wait_ids = self.all_serial_read_waiter_ids();
        let queued = self.enqueue_pending_wake_ids(wait_ids);
        self.stats.serial_read_wake_candidate_count += queued as u64;
        queued
    }

    pub fn select_ready_blocked_wait_id(
        &self,
        active_thread_id: u32,
        now_ms: u32,
        mut is_ready: impl FnMut(&SchedulerBlockedWait) -> bool,
        mut thread_priority: impl FnMut(u32) -> i32,
    ) -> Option<u64> {
        let pending = self
            .pending_wake_ids
            .iter()
            .filter_map(|wait_id| self.blocked_waits.get(wait_id))
            .filter(|wait| {
                wait.thread_id != active_thread_id
                    && (is_ready(wait) || blocked_wait_timed_out(wait, now_ms))
            })
            .min_by_key(|wait| (thread_priority(wait.thread_id), wait.sequence))
            .map(|wait| wait.id);
        if pending.is_some() {
            return pending;
        }

        self.blocked_waits
            .values()
            .filter(|wait| {
                wait.thread_id != active_thread_id
                    && (is_ready(wait) || blocked_wait_timed_out(wait, now_ms))
            })
            .min_by_key(|wait| (thread_priority(wait.thread_id), wait.sequence))
            .map(|wait| wait.id)
    }
}

pub fn blocked_wait_timed_out(wait: &SchedulerBlockedWait, now_ms: u32) -> bool {
    wait.timeout_ms != crate::ce::timer::INFINITE
        && now_ms.wrapping_sub(wait.wait_started_ms) >= wait.timeout_ms
}

#[cfg(test)]
mod tests {
    use super::{Scheduler, SchedulerBlockedWaitKind, blocked_wait_timed_out};

    #[test]
    fn scheduler_registers_waits_under_each_handle_and_removes_them() {
        let mut scheduler = Scheduler::default();
        let wait_id = scheduler.register_blocked_wait(
            7,
            0x107,
            vec![0x200, 0x204],
            SchedulerBlockedWaitKind::Kernel,
            10,
            50,
        );

        assert_eq!(scheduler.waiter_count(), 1);
        assert_eq!(scheduler.waiter_ids_for_handle(0x200), vec![wait_id]);
        assert_eq!(scheduler.waiter_ids_for_handle(0x204), vec![wait_id]);
        assert!(scheduler.blocked_wait(wait_id).is_some());

        let removed = scheduler.remove_blocked_wait(wait_id).unwrap();
        assert_eq!(removed.thread_id, 7);
        assert_eq!(scheduler.waiter_count(), 0);
        assert!(scheduler.waiter_ids_for_handle(0x200).is_empty());
        assert!(scheduler.waiter_ids_for_handle(0x204).is_empty());
    }

    #[test]
    fn scheduler_selects_ready_wait_by_ce_priority_then_fifo() {
        let mut scheduler = Scheduler::default();
        let low_priority = scheduler.register_blocked_wait(
            2,
            0x102,
            vec![0x200],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        );
        let high_priority = scheduler.register_blocked_wait(
            3,
            0x103,
            vec![0x204],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        );

        let selected = scheduler.select_ready_blocked_wait_id(
            1,
            0,
            |_| true,
            |thread_id| if thread_id == 3 { 10 } else { 200 },
        );
        assert_eq!(selected, Some(high_priority));

        scheduler.remove_blocked_wait(high_priority).unwrap();
        let selected = scheduler.select_ready_blocked_wait_id(1, 0, |_| true, |_| 20);
        assert_eq!(selected, Some(low_priority));
    }

    #[test]
    fn scheduler_timeout_uses_wrapping_tick_elapsed_time() {
        let mut scheduler = Scheduler::default();
        let wait_id = scheduler.register_blocked_wait(
            2,
            0x102,
            vec![0x200],
            SchedulerBlockedWaitKind::Kernel,
            u32::MAX - 5,
            10,
        );
        let wait = scheduler.blocked_wait(wait_id).unwrap();
        assert!(!blocked_wait_timed_out(wait, 3));
        assert!(blocked_wait_timed_out(wait, 4));
    }

    #[test]
    fn scheduler_queues_pending_wakes_and_cleans_them_on_remove() {
        let mut scheduler = Scheduler::default();
        let wait_id = scheduler.register_blocked_wait(
            2,
            0x102,
            vec![0x200],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        );

        assert_eq!(
            scheduler.queue_pending_wake_ids([wait_id, wait_id, 0xdead]),
            1
        );
        let stats = scheduler.stats();
        assert_eq!(stats.object_signal_count, 1);
        assert_eq!(stats.object_wake_candidate_count, 1);
        assert_eq!(stats.max_pending_wakes, 1);
        assert_eq!(
            scheduler.select_ready_blocked_wait_id(1, 0, |_| true, |_| 20),
            Some(wait_id)
        );

        scheduler.remove_blocked_wait(wait_id).unwrap();
        assert_eq!(
            scheduler.select_ready_blocked_wait_id(1, 0, |_| true, |_| 20),
            None
        );
    }

    #[test]
    fn scheduler_prefers_pending_wakes_before_global_ready_scan() {
        let mut scheduler = Scheduler::default();
        let globally_ready = scheduler.register_blocked_wait(
            2,
            0x102,
            vec![0x200],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        );
        let signaled_waiter = scheduler.register_blocked_wait(
            3,
            0x103,
            vec![0x204],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        );

        scheduler.queue_pending_wake_ids([signaled_waiter]);
        let selected = scheduler.select_ready_blocked_wait_id(
            1,
            0,
            |wait| wait.id == globally_ready || wait.id == signaled_waiter,
            |_| 20,
        );
        assert_eq!(selected, Some(signaled_waiter));
    }

    #[test]
    fn scheduler_queues_message_waiters_by_thread() {
        let mut scheduler = Scheduler::default();
        let global_ready = scheduler.register_blocked_wait(
            2,
            0x102,
            vec![0x200],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        );
        let message_waiter = scheduler.register_blocked_wait(
            3,
            0x103,
            Vec::new(),
            SchedulerBlockedWaitKind::MsgWait {
                wake_mask: 0x0008,
                input_available: false,
            },
            0,
            crate::ce::timer::INFINITE,
        );

        assert_eq!(
            scheduler.message_waiter_ids_for_thread(3),
            vec![message_waiter]
        );
        assert_eq!(scheduler.queue_message_wake_candidates(3), 1);
        let stats = scheduler.stats();
        assert_eq!(stats.message_input_signal_count, 1);
        assert_eq!(stats.message_input_wake_candidate_count, 1);
        assert_eq!(
            scheduler.select_ready_blocked_wait_id(
                1,
                0,
                |wait| wait.id == global_ready || wait.id == message_waiter,
                |_| 20,
            ),
            Some(message_waiter)
        );

        scheduler.remove_blocked_wait(message_waiter).unwrap();
        assert!(scheduler.message_waiter_ids_for_thread(3).is_empty());
        scheduler.remove_blocked_wait(global_ready).unwrap();
    }

    #[test]
    fn scheduler_queues_serial_read_waiters_by_handle() {
        let mut scheduler = Scheduler::default();
        let global_ready = scheduler.register_blocked_wait(
            2,
            0x102,
            vec![0x200],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        );
        let serial_waiter = scheduler.register_blocked_wait(
            4,
            0x104,
            Vec::new(),
            SchedulerBlockedWaitKind::SerialRead { handle: 0x300 },
            0,
            crate::ce::timer::INFINITE,
        );

        assert_eq!(
            scheduler.serial_read_waiter_ids_for_handle(0x300),
            vec![serial_waiter]
        );
        assert_eq!(scheduler.queue_serial_read_wake_candidates(0x300), 1);
        let stats = scheduler.stats();
        assert_eq!(stats.serial_read_signal_count, 1);
        assert_eq!(stats.serial_read_wake_candidate_count, 1);
        assert_eq!(
            scheduler.select_ready_blocked_wait_id(
                1,
                0,
                |wait| wait.id == global_ready || wait.id == serial_waiter,
                |_| 20,
            ),
            Some(serial_waiter)
        );

        scheduler.remove_blocked_wait(serial_waiter).unwrap();
        assert!(
            scheduler
                .serial_read_waiter_ids_for_handle(0x300)
                .is_empty()
        );
        scheduler.remove_blocked_wait(global_ready).unwrap();
    }

    #[test]
    fn scheduler_queues_get_message_waiters_by_thread() {
        let mut scheduler = Scheduler::default();
        let get_message_waiter = scheduler.register_blocked_wait(
            7,
            0x107,
            Vec::new(),
            SchedulerBlockedWaitKind::GetMessage {
                hwnd: Some(0x20000),
                min_msg: 0,
                max_msg: 0,
            },
            0,
            crate::ce::timer::INFINITE,
        );

        assert_eq!(
            scheduler.message_waiter_ids_for_thread(7),
            vec![get_message_waiter]
        );
        assert_eq!(scheduler.queue_message_wake_candidates(7), 1);
        let stats = scheduler.stats();
        assert_eq!(stats.message_input_signal_count, 1);
        assert_eq!(stats.message_input_wake_candidate_count, 1);
        assert_eq!(
            scheduler.select_ready_blocked_wait_id(
                1,
                0,
                |wait| wait.id == get_message_waiter,
                |_| 20,
            ),
            Some(get_message_waiter)
        );

        scheduler.remove_blocked_wait(get_message_waiter).unwrap();
        assert!(scheduler.message_waiter_ids_for_thread(7).is_empty());
    }

    #[test]
    fn scheduler_selects_timeout_only_sleep_wait_after_timeout() {
        let mut scheduler = Scheduler::default();
        let sleep_wait = scheduler.register_blocked_wait(
            8,
            0x108,
            Vec::new(),
            SchedulerBlockedWaitKind::Sleep,
            100,
            25,
        );

        assert_eq!(
            scheduler.select_ready_blocked_wait_id(1, 124, |_| false, |_| 20),
            None
        );
        assert_eq!(
            scheduler.select_ready_blocked_wait_id(1, 125, |_| false, |_| 20),
            Some(sleep_wait)
        );
        scheduler.remove_blocked_wait(sleep_wait).unwrap();
    }

    #[test]
    fn scheduler_records_thread_yield_as_sleep_attempt_without_blocking() {
        let mut scheduler = Scheduler::default();

        scheduler.record_thread_yield();

        let stats = scheduler.stats();
        assert_eq!(stats.sleep_count, 1);
        assert_eq!(stats.yield_count, 1);
        assert_eq!(stats.wait_block_count, 0);
        assert_eq!(stats.wait_wake_count, 0);
    }
}
