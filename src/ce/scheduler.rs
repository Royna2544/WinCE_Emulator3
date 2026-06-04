#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerWaitKind {
    Single,
    Multiple,
    MsgWait,
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
    pub wait_acquired_count: u64,
    pub wait_timeout_count: u64,
    pub wait_failed_count: u64,
    pub wait_block_count: u64,
    pub wait_wake_count: u64,
    pub max_wait_handles: u32,
    pub max_timeout_ms: u32,
}

#[derive(Debug, Clone, Default)]
pub struct Scheduler {
    stats: SchedulerStats,
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

    pub fn record_wait_wake(&mut self, reason: SchedulerWakeReason) {
        self.stats.wait_wake_count += 1;
        self.record_wait_result(reason);
    }
}
