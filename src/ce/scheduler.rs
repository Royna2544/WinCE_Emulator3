use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::ce::{
    object::{ProcessBirthKey, TaskFrameKey, TaskFrameRef, TaskWaitPin, ThreadBirthKey},
    task_wait::{TaskWaitRecord, TaskWaitStore, TickDeadline},
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerWinsockReadyKind {
    Read,
    Write,
    ReadWrite,
    Except,
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
    pub serial_event_signal_count: u64,
    pub serial_event_wake_candidate_count: u64,
    pub send_reply_signal_count: u64,
    pub send_reply_wake_candidate_count: u64,
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
    SerialCommEvent {
        handle: u32,
    },
    WinsockRead {
        socket: u32,
        readiness: SchedulerWinsockReadyKind,
        read_mask: u64,
        write_mask: u64,
        except_mask: u64,
    },
    SendMessage {
        send_id: u64,
    },
    ModalMessageBox {
        dialog_hwnd: u32,
    },
    PopupMenuModal {
        mouse_message_max: u32,
    },
    ClipboardRender {
        format: u32,
    },
}

/// Identity used only to index object wake candidates. Retaining this key does
/// not pin the object or make a closed public handle valid for acquisition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WaitObjectKey {
    /// Compatibility key for wait kinds that have not migrated to an object key.
    Handle(u32),
    Thread(ThreadBirthKey),
    Process(ProcessBirthKey),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerBlockedWait {
    pub id: u64,
    pub thread_id: u32,
    /// Exact owner identity when Kernel registered this frame for a live task.
    /// Standalone/host compatibility wait records have no known birth.
    pub owner_birth: Option<ThreadBirthKey>,
    /// Immutable locator for a canonical Task/Frame. Only migrated finite
    /// Sleep waits carry one; mutable phase lives in HandleTable.
    pub frame_ref: Option<TaskFrameRef>,
    pub thread_handle: u32,
    pub wait_handles: Vec<u32>,
    pub wait_keys: Vec<WaitObjectKey>,
    /// One acquired canonical lifetime reference per ordered task slot. The
    /// owning Kernel releases these when this record is removed.
    pub pinned_targets: Vec<Option<TaskWaitPin>>,
    pub kind: SchedulerBlockedWaitKind,
    pub wait_started_ms: u32,
    pub timeout_ms: u32,
    sequence: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerPublishError {
    WaitIdExhausted,
    WaitSequenceExhausted,
    DuplicateWaitId { wait_id: u64 },
}

impl std::fmt::Display for SchedulerPublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone, Copy)]
struct ReservedWaitSlot {
    id: u64,
    sequence: u64,
}

impl TaskWaitRecord for SchedulerBlockedWait {
    fn wait_id(&self) -> u64 {
        self.id
    }

    fn owner_thread_id(&self) -> u32 {
        self.thread_id
    }

    fn owner_birth(&self) -> Option<ThreadBirthKey> {
        self.owner_birth
    }
}

#[derive(Debug, Clone, Default)]
pub struct Scheduler {
    stats: SchedulerStats,
    next_wait_id: u64,
    next_wait_sequence: u64,
    blocked_waits: TaskWaitStore<SchedulerBlockedWait>,
    wait_queues: BTreeMap<WaitObjectKey, VecDeque<u64>>,
    message_wait_queues: BTreeMap<u32, VecDeque<u64>>,
    serial_read_queues: BTreeMap<u32, VecDeque<u64>>,
    serial_event_queues: BTreeMap<u32, VecDeque<u64>>,
    send_reply_queues: BTreeMap<u64, VecDeque<u64>>,
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

    /// Legacy/fixture compatibility registration for a wait with no known
    /// canonical birth. Production callers must publish through a verified
    /// birth-bearing Kernel path instead.
    pub fn register_legacy_blocked_wait(
        &mut self,
        thread_id: u32,
        thread_handle: u32,
        wait_handles: Vec<u32>,
        kind: SchedulerBlockedWaitKind,
        wait_started_ms: u32,
        timeout_ms: u32,
    ) -> u64 {
        let wait_keys = wait_handles
            .iter()
            .copied()
            .map(WaitObjectKey::Handle)
            .collect();
        self.register_legacy_blocked_wait_with_keys(
            thread_id,
            thread_handle,
            wait_handles,
            wait_keys,
            kind,
            wait_started_ms,
            timeout_ms,
        )
    }

    /// Legacy/fixture compatibility registration with canonical wake keys.
    /// The keys are an index, not wait-target lifetime references. Production
    /// callers must publish through a verified birth-bearing Kernel path.
    pub(crate) fn register_legacy_blocked_wait_with_keys(
        &mut self,
        thread_id: u32,
        thread_handle: u32,
        wait_handles: Vec<u32>,
        wait_keys: Vec<WaitObjectKey>,
        kind: SchedulerBlockedWaitKind,
        wait_started_ms: u32,
        timeout_ms: u32,
    ) -> u64 {
        let pinned_targets = vec![None; wait_handles.len()];
        self.register_legacy_blocked_wait_with_keys_and_pins(
            thread_id,
            thread_handle,
            wait_handles,
            wait_keys,
            pinned_targets,
            kind,
            wait_started_ms,
            timeout_ms,
        )
    }

    /// Publishes a prepared wait for a canonical owner. Kernel must verify
    /// that this exact birth is execution-live immediately before calling.
    /// The numeric TID is derived from the birth, never caller-selected.
    pub(crate) fn register_blocked_wait_for_birth_with_keys_and_pins(
        &mut self,
        owner_birth: ThreadBirthKey,
        thread_handle: u32,
        wait_handles: Vec<u32>,
        wait_keys: Vec<WaitObjectKey>,
        pinned_targets: Vec<Option<TaskWaitPin>>,
        kind: SchedulerBlockedWaitKind,
        wait_started_ms: u32,
        timeout_ms: u32,
    ) -> u64 {
        self.insert_blocked_wait_with_keys_and_pins(
            owner_birth.thread_id(),
            Some(owner_birth),
            thread_handle,
            wait_handles,
            wait_keys,
            pinned_targets,
            kind,
            wait_started_ms,
            timeout_ms,
        )
    }

    /// Explicit compatibility publication for a waiter whose canonical birth
    /// was never known. Never use this for a production owner that is live.
    pub(crate) fn register_legacy_blocked_wait_with_keys_and_pins(
        &mut self,
        thread_id: u32,
        thread_handle: u32,
        wait_handles: Vec<u32>,
        wait_keys: Vec<WaitObjectKey>,
        pinned_targets: Vec<Option<TaskWaitPin>>,
        kind: SchedulerBlockedWaitKind,
        wait_started_ms: u32,
        timeout_ms: u32,
    ) -> u64 {
        self.insert_blocked_wait_with_keys_and_pins(
            thread_id,
            None,
            thread_handle,
            wait_handles,
            wait_keys,
            pinned_targets,
            kind,
            wait_started_ms,
            timeout_ms,
        )
    }

    /// Publish the first migrated Task/Frame family. The caller preflights
    /// the canonical frame key and activates the returned reference in
    /// HandleTable before exposing it to native scheduling.
    pub(crate) fn try_register_finite_sleep_wait(
        &mut self,
        frame_key: TaskFrameKey,
        thread_handle: u32,
        wait_started_ms: u32,
        timeout_ms: u32,
    ) -> std::result::Result<TaskFrameRef, SchedulerPublishError> {
        let slot = self.try_reserve_wait_slot()?;
        let frame_ref = TaskFrameRef::new(frame_key, slot.id)
            .expect("checked scheduler ID must be a nonzero TaskFrameRef wait ID");
        self.insert_wait_record(SchedulerBlockedWait {
            id: slot.id,
            thread_id: frame_key.owner_birth().thread_id(),
            owner_birth: Some(frame_key.owner_birth()),
            frame_ref: Some(frame_ref),
            thread_handle,
            wait_handles: Vec::new(),
            wait_keys: Vec::new(),
            pinned_targets: Vec::new(),
            kind: SchedulerBlockedWaitKind::Sleep,
            wait_started_ms,
            timeout_ms,
            sequence: slot.sequence,
        });
        Ok(frame_ref)
    }

    /// IDs and tie-breaker sequence values are monotonic within this machine
    /// instance. Exhaustion is a failure, never wrap/replacement of a live ID.
    fn try_reserve_wait_slot(
        &mut self,
    ) -> std::result::Result<ReservedWaitSlot, SchedulerPublishError> {
        let id = self
            .next_wait_id
            .checked_add(1)
            .ok_or(SchedulerPublishError::WaitIdExhausted)?;
        let sequence = self
            .next_wait_sequence
            .checked_add(1)
            .ok_or(SchedulerPublishError::WaitSequenceExhausted)?;
        if self.blocked_waits.contains_key(&id) {
            return Err(SchedulerPublishError::DuplicateWaitId { wait_id: id });
        }
        self.next_wait_id = id;
        self.next_wait_sequence = sequence;
        Ok(ReservedWaitSlot { id, sequence })
    }

    /// One insertion transaction; its optional owner is private to the
    /// scheduler so no external producer can silently publish unknown.
    fn insert_blocked_wait_with_keys_and_pins(
        &mut self,
        thread_id: u32,
        owner_birth: Option<ThreadBirthKey>,
        thread_handle: u32,
        wait_handles: Vec<u32>,
        wait_keys: Vec<WaitObjectKey>,
        pinned_targets: Vec<Option<TaskWaitPin>>,
        kind: SchedulerBlockedWaitKind,
        wait_started_ms: u32,
        timeout_ms: u32,
    ) -> u64 {
        assert_eq!(
            wait_handles.len(),
            wait_keys.len(),
            "each raw wait handle must have one ordered wake key"
        );
        assert_eq!(
            wait_handles.len(),
            pinned_targets.len(),
            "each raw wait handle must have one ordered pin slot"
        );
        if let Some(birth) = owner_birth {
            assert_eq!(
                thread_id,
                birth.thread_id(),
                "wait record TID does not match owner birth"
            );
        }
        // Pre-existing publishers keep their infallible signature in this
        // bounded cut. Exhaustion stops them instead of wrapping onto a live
        // wait; finite Sleep uses the typed path above.
        let slot = self
            .try_reserve_wait_slot()
            .expect("scheduler wait ID or sequence exhausted");
        let wait = SchedulerBlockedWait {
            id: slot.id,
            thread_id,
            owner_birth,
            frame_ref: None,
            thread_handle,
            wait_handles,
            wait_keys,
            pinned_targets,
            kind,
            wait_started_ms,
            timeout_ms,
            sequence: slot.sequence,
        };
        self.insert_wait_record(wait)
    }

    fn insert_wait_record(&mut self, wait: SchedulerBlockedWait) -> u64 {
        let id = wait.id;
        self.blocked_waits
            .insert_new(id, wait)
            .expect("reserved scheduler wait ID was unexpectedly reused");
        let wait = self.blocked_waits.get(&id).expect("inserted wait missing");
        for key in &wait.wait_keys {
            self.wait_queues.entry(*key).or_default().push_back(id);
        }
        if matches!(
            wait.kind,
            SchedulerBlockedWaitKind::GetMessage { .. }
                | SchedulerBlockedWaitKind::MsgWait { .. }
                | SchedulerBlockedWaitKind::ModalMessageBox { .. }
                | SchedulerBlockedWaitKind::PopupMenuModal { .. }
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
        if let SchedulerBlockedWaitKind::SerialCommEvent { handle } = wait.kind {
            self.serial_event_queues
                .entry(handle)
                .or_default()
                .push_back(id);
        }
        if let SchedulerBlockedWaitKind::SendMessage { send_id } = wait.kind {
            self.send_reply_queues
                .entry(send_id)
                .or_default()
                .push_back(id);
        }
        self.stats.waiter_register_count += 1;
        self.stats.max_registered_waits = self
            .stats
            .max_registered_waits
            .max(self.blocked_waits.len() as u32);
        id
    }

    pub fn remove_blocked_wait(&mut self, wait_id: u64) -> Option<SchedulerBlockedWait> {
        let wait = self.blocked_waits.remove(&wait_id)?;
        for key in &wait.wait_keys {
            let remove_queue = if let Some(queue) = self.wait_queues.get_mut(key) {
                queue.retain(|id| *id != wait_id);
                queue.is_empty()
            } else {
                false
            };
            if remove_queue {
                self.wait_queues.remove(key);
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
        if let SchedulerBlockedWaitKind::SerialCommEvent { handle } = wait.kind {
            let remove_serial_event_queue =
                if let Some(queue) = self.serial_event_queues.get_mut(&handle) {
                    queue.retain(|id| *id != wait_id);
                    queue.is_empty()
                } else {
                    false
                };
            if remove_serial_event_queue {
                self.serial_event_queues.remove(&handle);
            }
        }
        if let SchedulerBlockedWaitKind::SendMessage { send_id } = wait.kind {
            let remove_send_queue = if let Some(queue) = self.send_reply_queues.get_mut(&send_id) {
                queue.retain(|id| *id != wait_id);
                queue.is_empty()
            } else {
                false
            };
            if remove_send_queue {
                self.send_reply_queues.remove(&send_id);
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

    pub fn blocked_waits(&self) -> impl Iterator<Item = &SchedulerBlockedWait> {
        self.blocked_waits.values()
    }

    pub fn waiter_count(&self) -> usize {
        self.blocked_waits.len()
    }

    /// IDs for explicit legacy records with unknown birth only. Canonical
    /// task exit must use `waiter_ids_for_birth`; a later birth with this TID
    /// must not inherit unknown records. Remove by ID to retire subscriptions.
    pub fn waiter_ids_for_thread(&self, thread_id: u32) -> Vec<u64> {
        self.blocked_waits.legacy_wait_ids_for_thread(thread_id)
    }

    /// Direct exact-birth owner lookup. The wait store is authoritative;
    /// subscription queues and the wait-ID locator are derived only.
    pub fn waiter_ids_for_birth(&self, owner_birth: ThreadBirthKey) -> Vec<u64> {
        self.blocked_waits.wait_ids_for_birth(owner_birth)
    }

    pub fn waiter_ids_for_handle(&self, handle: u32) -> Vec<u64> {
        let mut ids = BTreeSet::new();
        if let Some(queue) = self.wait_queues.get(&WaitObjectKey::Handle(handle)) {
            ids.extend(queue.iter().copied());
        }
        ids.extend(
            self.blocked_waits
                .values()
                .filter(|wait| wait.wait_handles.contains(&handle))
                .map(|wait| wait.id),
        );
        ids.into_iter().collect()
    }

    /// Looks up wake candidates by object identity, including after a
    /// compatibility index is rebuilt. This does not pin wait targets.
    pub fn waiter_ids_for_key(&self, key: WaitObjectKey) -> Vec<u64> {
        let mut ids = BTreeSet::new();
        if let Some(queue) = self.wait_queues.get(&key) {
            ids.extend(queue.iter().copied());
        }
        ids.extend(
            self.blocked_waits
                .values()
                .filter(|wait| wait.wait_keys.contains(&key))
                .map(|wait| wait.id),
        );
        ids.into_iter().collect()
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

    pub fn serial_event_waiter_ids_for_handle(&self, handle: u32) -> Vec<u64> {
        self.serial_event_queues
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

    pub fn all_serial_event_waiter_ids(&self) -> Vec<u64> {
        self.serial_event_queues
            .values()
            .flat_map(|queue| queue.iter().copied())
            .collect()
    }

    pub fn send_reply_waiter_ids_for_send(&self, send_id: u64) -> Vec<u64> {
        self.send_reply_queues
            .get(&send_id)
            .map(|queue| queue.iter().copied().collect())
            .unwrap_or_default()
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

    /// Thread-directed input is owned by one incarnation. The numeric index
    /// remains a lookup hint; it cannot authorize waking a reused TID's old
    /// wait or an explicit legacy-unknown fixture wait.
    pub fn queue_message_wake_candidates_for_birth(
        &mut self,
        owner_birth: ThreadBirthKey,
    ) -> usize {
        self.stats.message_input_signal_count += 1;
        let wait_ids = self
            .message_waiter_ids_for_thread(owner_birth.thread_id())
            .into_iter()
            .filter(|id| {
                self.blocked_waits
                    .get(id)
                    .is_some_and(|wait| wait.owner_birth == Some(owner_birth))
            })
            .collect::<Vec<_>>();
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

    pub fn queue_serial_event_wake_candidates(&mut self, handle: u32) -> usize {
        self.stats.serial_event_signal_count += 1;
        let wait_ids = self.serial_event_waiter_ids_for_handle(handle);
        let queued = self.enqueue_pending_wake_ids(wait_ids);
        self.stats.serial_event_wake_candidate_count += queued as u64;
        queued
    }

    pub fn queue_all_serial_read_wake_candidates(&mut self) -> usize {
        self.stats.serial_read_signal_count += 1;
        let wait_ids = self.all_serial_read_waiter_ids();
        let queued = self.enqueue_pending_wake_ids(wait_ids);
        self.stats.serial_read_wake_candidate_count += queued as u64;
        queued
    }

    pub fn queue_all_serial_event_wake_candidates(&mut self) -> usize {
        self.stats.serial_event_signal_count += 1;
        let wait_ids = self.all_serial_event_waiter_ids();
        let queued = self.enqueue_pending_wake_ids(wait_ids);
        self.stats.serial_event_wake_candidate_count += queued as u64;
        queued
    }

    pub fn queue_send_reply_wake_candidates(&mut self, send_id: u64) -> usize {
        self.stats.send_reply_signal_count += 1;
        let wait_ids = self.send_reply_waiter_ids_for_send(send_id);
        let queued = self.enqueue_pending_wake_ids(wait_ids);
        self.stats.send_reply_wake_candidate_count += queued as u64;
        queued
    }

    pub fn select_ready_blocked_wait_id(
        &self,
        active_thread_id: u32,
        now_ms: u32,
        is_ready: impl FnMut(&SchedulerBlockedWait) -> bool,
        thread_priority: impl FnMut(u32) -> i32,
    ) -> Option<u64> {
        self.select_ready_blocked_wait_id_with_eligibility(
            active_thread_id,
            now_ms,
            |_| true,
            is_ready,
            thread_priority,
        )
    }

    /// Check task ownership before either input readiness or timeout can
    /// select a wait. A readiness-only guard cannot exclude an expired stale
    /// wait because timeout is an independent selection reason.
    pub fn select_ready_blocked_wait_id_with_eligibility(
        &self,
        active_thread_id: u32,
        now_ms: u32,
        mut is_eligible: impl FnMut(&SchedulerBlockedWait) -> bool,
        mut is_ready: impl FnMut(&SchedulerBlockedWait) -> bool,
        mut thread_priority: impl FnMut(u32) -> i32,
    ) -> Option<u64> {
        let pending = self
            .pending_wake_ids
            .iter()
            .filter_map(|wait_id| self.blocked_waits.get(wait_id))
            .filter(|wait| {
                wait.thread_id != active_thread_id
                    && is_eligible(wait)
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
                    && is_eligible(wait)
                    && (is_ready(wait) || blocked_wait_timed_out(wait, now_ms))
            })
            .min_by_key(|wait| (thread_priority(wait.thread_id), wait.sequence))
            .map(|wait| wait.id)
    }
}

pub fn blocked_wait_timed_out(wait: &SchedulerBlockedWait, now_ms: u32) -> bool {
    TickDeadline::from_legacy(wait.wait_started_ms, wait.timeout_ms).expired(now_ms)
}

#[cfg(test)]
mod tests {
    use super::{
        Scheduler, SchedulerBlockedWaitKind, SchedulerPublishError, blocked_wait_timed_out,
    };
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

    fn register_birth_wait(
        scheduler: &mut Scheduler,
        birth: ThreadBirthKey,
        kind: SchedulerBlockedWaitKind,
    ) -> u64 {
        scheduler.register_blocked_wait_for_birth_with_keys_and_pins(
            birth,
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            kind,
            100,
            25,
        )
    }

    #[test]
    fn scheduler_typed_birth_and_explicit_legacy_publication_keep_owner_kinds_distinct() {
        let (birth, _) = reused_thread_births(7);
        let mut scheduler = Scheduler::default();
        let exact = scheduler.register_blocked_wait_for_birth_with_keys_and_pins(
            birth,
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            SchedulerBlockedWaitKind::Sleep,
            0,
            u32::MAX,
        );
        let legacy = scheduler.register_legacy_blocked_wait_with_keys_and_pins(
            birth.thread_id(),
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            SchedulerBlockedWaitKind::Sleep,
            0,
            u32::MAX,
        );
        assert_eq!(
            scheduler.blocked_wait(exact).unwrap().thread_id,
            birth.thread_id()
        );
        assert_eq!(
            scheduler.blocked_wait(exact).unwrap().owner_birth,
            Some(birth)
        );
        assert_eq!(scheduler.blocked_wait(legacy).unwrap().owner_birth, None);
        assert_eq!(scheduler.waiter_ids_for_birth(birth), [exact]);
        assert_eq!(scheduler.waiter_ids_for_thread(birth.thread_id()), [legacy]);
    }

    #[test]
    fn thread_directed_message_wakes_only_its_exact_birth() {
        let (old, new) = reused_thread_births(7);
        let mut scheduler = Scheduler::default();
        let old_id = register_birth_wait(
            &mut scheduler,
            old,
            SchedulerBlockedWaitKind::GetMessage {
                hwnd: None,
                min_msg: 0,
                max_msg: 0,
            },
        );
        let new_id = register_birth_wait(
            &mut scheduler,
            new,
            SchedulerBlockedWaitKind::GetMessage {
                hwnd: None,
                min_msg: 0,
                max_msg: 0,
            },
        );
        let unknown_id = scheduler.register_legacy_blocked_wait_with_keys_and_pins(
            7,
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            SchedulerBlockedWaitKind::GetMessage {
                hwnd: None,
                min_msg: 0,
                max_msg: 0,
            },
            100,
            25,
        );

        assert_eq!(scheduler.queue_message_wake_candidates_for_birth(new), 1);
        assert_eq!(
            scheduler
                .pending_wake_ids
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            [new_id]
        );
        assert_eq!(
            scheduler.blocked_wait(old_id).unwrap().owner_birth,
            Some(old)
        );
        assert_eq!(
            scheduler.blocked_wait(unknown_id).unwrap().owner_birth,
            None
        );
    }

    #[test]
    fn scheduler_cancels_only_exact_birth_and_keeps_legacy_unknown_separate() {
        let (old, new) = reused_thread_births(7);
        let mut scheduler = Scheduler::default();
        let old_modal = register_birth_wait(
            &mut scheduler,
            old,
            SchedulerBlockedWaitKind::ModalMessageBox {
                dialog_hwnd: 0x20000,
            },
        );
        let unknown = scheduler.register_legacy_blocked_wait(
            7,
            0,
            Vec::new(),
            SchedulerBlockedWaitKind::Sleep,
            100,
            25,
        );
        let new_sleep = register_birth_wait(&mut scheduler, new, SchedulerBlockedWaitKind::Sleep);
        let old_nested = register_birth_wait(&mut scheduler, old, SchedulerBlockedWaitKind::Sleep);
        assert_eq!(scheduler.waiter_ids_for_birth(old), [old_modal, old_nested]);
        assert_eq!(scheduler.waiter_ids_for_birth(new), [new_sleep]);
        assert_eq!(scheduler.waiter_ids_for_thread(7), [unknown]);
        assert_eq!(
            scheduler
                .blocked_waits()
                .map(|wait| wait.id)
                .collect::<Vec<_>>(),
            [old_modal, unknown, new_sleep, old_nested]
        );
        assert_eq!(scheduler.message_waiter_ids_for_thread(7), [old_modal]);

        scheduler.queue_pending_wake_ids([old_modal, new_sleep]);
        let mut clone = scheduler.clone();
        for id in clone.waiter_ids_for_birth(old) {
            clone.remove_blocked_wait(id).unwrap();
        }
        assert!(clone.waiter_ids_for_birth(old).is_empty());
        assert!(clone.message_waiter_ids_for_thread(7).is_empty());
        assert_eq!(clone.waiter_ids_for_birth(new), [new_sleep]);
        assert_eq!(clone.waiter_ids_for_thread(7), [unknown]);
        assert_eq!(scheduler.waiter_ids_for_birth(old), [old_modal, old_nested]);
        assert!(!blocked_wait_timed_out(
            clone.blocked_wait(new_sleep).unwrap(),
            124
        ));
        assert!(blocked_wait_timed_out(
            clone.blocked_wait(new_sleep).unwrap(),
            125
        ));
        clone.remove_blocked_wait(unknown).unwrap();
        clone.remove_blocked_wait(new_sleep).unwrap();
        assert_eq!(clone.waiter_count(), 0);
        assert!(clone.message_waiter_ids_for_thread(7).is_empty());
    }

    #[test]
    fn scheduler_task_owned_cancellation_preserves_other_tasks_and_cleans_subscriptions() {
        let mut scheduler = Scheduler::default();
        let first = scheduler.register_legacy_blocked_wait(
            7,
            0x107,
            vec![0x200],
            SchedulerBlockedWaitKind::Kernel,
            0,
            u32::MAX,
        );
        let nested = scheduler.register_legacy_blocked_wait(
            7,
            0x107,
            Vec::new(),
            SchedulerBlockedWaitKind::GetMessage {
                hwnd: None,
                min_msg: 0,
                max_msg: 0,
            },
            0,
            u32::MAX,
        );
        let serial = scheduler.register_legacy_blocked_wait(
            8,
            0x108,
            Vec::new(),
            SchedulerBlockedWaitKind::SerialRead { handle: 0x300 },
            0,
            u32::MAX,
        );
        let neighbor = scheduler.register_legacy_blocked_wait(
            9,
            0x109,
            vec![0x200],
            SchedulerBlockedWaitKind::Kernel,
            0,
            u32::MAX,
        );
        scheduler.queue_pending_wake_ids([first, nested, serial, neighbor]);
        assert_eq!(scheduler.waiter_ids_for_thread(7), [first, nested]);
        assert_eq!(scheduler.message_waiter_ids_for_thread(7), [nested]);
        // Same cancellation shape as CeKernel: ownership lookup, then the
        // existing removal boundary, never deleting records behind indexes.
        for id in scheduler.waiter_ids_for_thread(7) {
            scheduler.remove_blocked_wait(id).unwrap();
        }
        assert!(scheduler.waiter_ids_for_thread(7).is_empty());
        assert!(scheduler.message_waiter_ids_for_thread(7).is_empty());
        assert_eq!(scheduler.waiter_ids_for_handle(0x200), [neighbor]);
        assert_eq!(scheduler.serial_read_waiter_ids_for_handle(0x300), [serial]);
        assert_eq!(scheduler.waiter_count(), 2);
        assert_eq!(scheduler.stats().waiter_remove_count, 2);
        assert_eq!(
            scheduler.select_ready_blocked_wait_id(0, 0, |_| true, |_| 20),
            Some(serial),
        );
    }

    #[test]
    fn scheduler_task_storage_keeps_global_wait_id_order() {
        let mut scheduler = Scheduler::default();
        let first = scheduler.register_legacy_blocked_wait(
            20,
            0x120,
            Vec::new(),
            SchedulerBlockedWaitKind::Sleep,
            100,
            25,
        );
        let second = scheduler.register_legacy_blocked_wait(
            1,
            0x101,
            Vec::new(),
            SchedulerBlockedWaitKind::Sleep,
            100,
            25,
        );
        let third = scheduler.register_legacy_blocked_wait(
            20,
            0x120,
            Vec::new(),
            SchedulerBlockedWaitKind::Sleep,
            100,
            25,
        );
        assert_eq!(scheduler.waiter_ids_for_thread(20), [first, third]);
        assert_eq!(
            scheduler
                .blocked_waits()
                .map(|wait| wait.id)
                .collect::<Vec<_>>(),
            [first, second, third],
        );
        assert!(!blocked_wait_timed_out(
            scheduler.blocked_wait(first).unwrap(),
            124
        ));
        assert!(blocked_wait_timed_out(
            scheduler.blocked_wait(first).unwrap(),
            125
        ));
    }

    #[test]
    fn scheduler_registers_waits_under_each_handle_and_removes_them() {
        let mut scheduler = Scheduler::default();
        let wait_id = scheduler.register_legacy_blocked_wait(
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
    fn scheduler_waiter_lookup_falls_back_to_live_waits_when_index_is_stale() {
        let mut scheduler = Scheduler::default();
        let wait_id = scheduler.register_legacy_blocked_wait(
            7,
            0x107,
            vec![0x200, 0x204],
            SchedulerBlockedWaitKind::Kernel,
            10,
            50,
        );

        scheduler.wait_queues.clear();

        assert_eq!(scheduler.waiter_ids_for_handle(0x200), vec![wait_id]);
        assert_eq!(scheduler.waiter_ids_for_handle(0x204), vec![wait_id]);
        assert!(scheduler.waiter_ids_for_handle(0x208).is_empty());
        assert_eq!(scheduler.queue_pending_wake_ids([wait_id]), 1);
    }

    #[test]
    fn scheduler_selects_ready_wait_by_ce_priority_then_fifo() {
        let mut scheduler = Scheduler::default();
        let low_priority = scheduler.register_legacy_blocked_wait(
            2,
            0x102,
            vec![0x200],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        );
        let high_priority = scheduler.register_legacy_blocked_wait(
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
        let wait_id = scheduler.register_legacy_blocked_wait(
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
        let wait_id = scheduler.register_legacy_blocked_wait(
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
        let globally_ready = scheduler.register_legacy_blocked_wait(
            2,
            0x102,
            vec![0x200],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        );
        let signaled_waiter = scheduler.register_legacy_blocked_wait(
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
        let global_ready = scheduler.register_legacy_blocked_wait(
            2,
            0x102,
            vec![0x200],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        );
        let message_waiter = scheduler.register_legacy_blocked_wait(
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
        let global_ready = scheduler.register_legacy_blocked_wait(
            2,
            0x102,
            vec![0x200],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        );
        let serial_waiter = scheduler.register_legacy_blocked_wait(
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
    fn scheduler_queues_serial_comm_event_waiters_by_handle() {
        let mut scheduler = Scheduler::default();
        let global_ready = scheduler.register_legacy_blocked_wait(
            2,
            0x102,
            vec![0x200],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        );
        let comm_waiter = scheduler.register_legacy_blocked_wait(
            4,
            0x104,
            Vec::new(),
            SchedulerBlockedWaitKind::SerialCommEvent { handle: 0x300 },
            0,
            crate::ce::timer::INFINITE,
        );

        assert_eq!(
            scheduler.serial_event_waiter_ids_for_handle(0x300),
            vec![comm_waiter]
        );
        assert_eq!(scheduler.queue_serial_event_wake_candidates(0x300), 1);
        let stats = scheduler.stats();
        assert_eq!(stats.serial_event_signal_count, 1);
        assert_eq!(stats.serial_event_wake_candidate_count, 1);
        assert_eq!(
            scheduler.select_ready_blocked_wait_id(
                1,
                0,
                |wait| wait.id == global_ready || wait.id == comm_waiter,
                |_| 20,
            ),
            Some(comm_waiter)
        );

        scheduler.remove_blocked_wait(comm_waiter).unwrap();
        assert!(
            scheduler
                .serial_event_waiter_ids_for_handle(0x300)
                .is_empty()
        );
        scheduler.remove_blocked_wait(global_ready).unwrap();
    }

    #[test]
    fn scheduler_queues_send_reply_waiters_by_send_id() {
        let mut scheduler = Scheduler::default();
        let global_ready = scheduler.register_legacy_blocked_wait(
            2,
            0x102,
            vec![0x200],
            SchedulerBlockedWaitKind::Kernel,
            0,
            crate::ce::timer::INFINITE,
        );
        let send_waiter = scheduler.register_legacy_blocked_wait(
            5,
            0x105,
            Vec::new(),
            SchedulerBlockedWaitKind::SendMessage { send_id: 0x700 },
            0,
            crate::ce::timer::INFINITE,
        );

        assert_eq!(
            scheduler.send_reply_waiter_ids_for_send(0x700),
            vec![send_waiter]
        );
        assert_eq!(scheduler.queue_send_reply_wake_candidates(0x700), 1);
        let stats = scheduler.stats();
        assert_eq!(stats.send_reply_signal_count, 1);
        assert_eq!(stats.send_reply_wake_candidate_count, 1);
        assert_eq!(
            scheduler.select_ready_blocked_wait_id(
                1,
                0,
                |wait| wait.id == global_ready || wait.id == send_waiter,
                |_| 20,
            ),
            Some(send_waiter)
        );

        scheduler.remove_blocked_wait(send_waiter).unwrap();
        assert!(scheduler.send_reply_waiter_ids_for_send(0x700).is_empty());
        scheduler.remove_blocked_wait(global_ready).unwrap();
    }

    #[test]
    fn scheduler_queues_get_message_waiters_by_thread() {
        let mut scheduler = Scheduler::default();
        let get_message_waiter = scheduler.register_legacy_blocked_wait(
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
        let sleep_wait = scheduler.register_legacy_blocked_wait(
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

    #[test]
    fn finite_sleep_selection_rechecks_exact_active_frame_and_phase() {
        let mut handles = HandleTable::default();
        let handle = handles.create_thread(7, 0x1000, 0, false);
        let birth = handles.thread_birth_key_for_handle(handle).unwrap();
        let mut scheduler = Scheduler::default();

        let first = scheduler
            .try_register_finite_sleep_wait(
                handles.preflight_finite_sleep_frame(birth).unwrap(),
                handle,
                100,
                25,
            )
            .unwrap();
        handles.activate_finite_sleep_frame(first).unwrap();
        handles.begin_finite_sleep_completion(first).unwrap();
        handles.retire_finite_sleep_frame(first).unwrap();

        // A stale locator remains in this synthetic index. Its older sequence
        // must not win over the newly active frame for the same exact birth.
        let second = scheduler
            .try_register_finite_sleep_wait(
                handles.preflight_finite_sleep_frame(birth).unwrap(),
                handle,
                100,
                25,
            )
            .unwrap();
        handles.activate_finite_sleep_frame(second).unwrap();
        let select = |handles: &HandleTable, scheduler: &Scheduler| {
            scheduler.select_ready_blocked_wait_id_with_eligibility(
                0,
                125,
                |wait| {
                    wait.frame_ref
                        .is_some_and(|frame| handles.finite_sleep_frame_is_blocked(frame))
                },
                |_| false,
                |_| 20,
            )
        };
        assert_eq!(select(&handles, &scheduler), Some(second.wait_id()));
        handles.begin_finite_sleep_completion(second).unwrap();
        assert_eq!(select(&handles, &scheduler), None, "duplicate selection");
        scheduler.remove_blocked_wait(first.wait_id()).unwrap();
        scheduler.remove_blocked_wait(second.wait_id()).unwrap();
        handles.retire_finite_sleep_frame(second).unwrap();
    }

    #[test]
    fn finite_sleep_wait_id_and_sequence_exhaustion_publish_nothing() {
        let mut handles = HandleTable::default();
        let handle = handles.create_thread(7, 0x1000, 0, false);
        let birth = handles.thread_birth_key_for_handle(handle).unwrap();
        let key = handles.preflight_finite_sleep_frame(birth).unwrap();
        let mut scheduler = Scheduler::default();
        scheduler.next_wait_id = u64::MAX;
        assert_eq!(
            scheduler.try_register_finite_sleep_wait(key, handle, 100, 25),
            Err(SchedulerPublishError::WaitIdExhausted)
        );
        assert_eq!(scheduler.blocked_waits().count(), 0);
        assert_eq!(scheduler.stats().waiter_register_count, 0);

        scheduler.next_wait_id = 0;
        scheduler.next_wait_sequence = u64::MAX;
        assert_eq!(
            scheduler.try_register_finite_sleep_wait(key, handle, 100, 25),
            Err(SchedulerPublishError::WaitSequenceExhausted)
        );
        assert_eq!(scheduler.next_wait_id, 0);
        assert_eq!(scheduler.blocked_waits().count(), 0);
    }

    #[test]
    fn finite_sleep_duplicate_wait_id_does_not_replace_original() {
        let mut handles = HandleTable::default();
        let handle = handles.create_thread(7, 0x1000, 0, false);
        let birth = handles.thread_birth_key_for_handle(handle).unwrap();
        let mut scheduler = Scheduler::default();
        let original = scheduler.register_legacy_blocked_wait(
            8,
            0,
            Vec::new(),
            SchedulerBlockedWaitKind::Kernel,
            0,
            u32::MAX,
        );
        scheduler.next_wait_id = original - 1;
        let count = scheduler.stats().waiter_register_count;
        assert_eq!(
            scheduler.try_register_finite_sleep_wait(
                handles.preflight_finite_sleep_frame(birth).unwrap(),
                handle,
                100,
                25,
            ),
            Err(SchedulerPublishError::DuplicateWaitId { wait_id: original })
        );
        assert_eq!(scheduler.stats().waiter_register_count, count);
        assert_eq!(scheduler.blocked_wait(original).unwrap().thread_id, 8);
        assert_eq!(scheduler.blocked_wait(original).unwrap().frame_ref, None);
    }
}
