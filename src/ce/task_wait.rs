//! Transitional authoritative wait storage for the existing Scheduler.
//!
//! This migrates wait-record ownership, not the CPU runnable state machine.
//! The current globally allocated thread ID is the compatibility task key.
//! Process/incarnation, continuation frames, ready queues and virtual time must
//! be wired through real thread/engine lifecycle before replacing that key.

use std::collections::BTreeMap;

/// Compatibility deadline in the existing wrapping GetTickCount domain.
/// This deliberately preserves current behavior; it is not a virtual clock
/// and does not silently add CE6's finite-wait tick rounding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TickDeadline {
    Infinite,
    Finite { started_ms: u32, timeout_ms: u32 },
}

impl TickDeadline {
    pub(crate) fn from_legacy(started_ms: u32, timeout_ms: u32) -> Self {
        if timeout_ms == u32::MAX {
            Self::Infinite
        } else {
            Self::Finite {
                started_ms,
                timeout_ms,
            }
        }
    }

    pub(crate) fn expired(self, now_ms: u32) -> bool {
        match self {
            Self::Infinite => false,
            Self::Finite {
                started_ms,
                timeout_ms,
            } => now_ms.wrapping_sub(started_ms) >= timeout_ms,
        }
    }
}

/// Stored records expose only immutable identity; callers cannot mutate
/// ownership behind the registry's indexes.
pub(crate) trait TaskWaitRecord {
    fn wait_id(&self) -> u64;
    fn owner_thread_id(&self) -> u32;
}

/// Each record is owned exactly once by a compatibility task. The wait-ID map
/// is a derived locator, not a second record store. Multiple records for one
/// TID are preserved until legacy reentrant continuation frames are migrated.
#[derive(Debug, Clone)]
pub(crate) struct TaskWaitStore<W> {
    tasks: BTreeMap<u32, BTreeMap<u64, W>>,
    owners: BTreeMap<u64, u32>,
}

impl<W> Default for TaskWaitStore<W> {
    fn default() -> Self {
        Self {
            tasks: BTreeMap::new(),
            owners: BTreeMap::new(),
        }
    }
}

impl<W: TaskWaitRecord> TaskWaitStore<W> {
    /// Match the old BTreeMap replacement contract while keeping task indexes
    /// coherent. This does not authorize wait-ID reuse or add ABA protection.
    pub(crate) fn insert(&mut self, wait_id: u64, record: W) -> Option<W> {
        assert_eq!(wait_id, record.wait_id(), "wait record identity mismatch");
        let previous = self.remove(&wait_id);
        let owner = record.owner_thread_id();
        self.tasks.entry(owner).or_default().insert(wait_id, record);
        self.owners.insert(wait_id, owner);
        previous
    }

    pub(crate) fn remove(&mut self, wait_id: &u64) -> Option<W> {
        let owner = self.owners.remove(wait_id)?;
        let records = self.tasks.get_mut(&owner).expect("wait owner missing");
        let removed = records.remove(wait_id).expect("owned wait missing");
        if records.is_empty() {
            self.tasks.remove(&owner);
        }
        Some(removed)
    }

    pub(crate) fn get(&self, wait_id: &u64) -> Option<&W> {
        let owner = self.owners.get(wait_id)?;
        self.tasks.get(owner)?.get(wait_id)
    }

    pub(crate) fn contains_key(&self, wait_id: &u64) -> bool {
        self.owners.contains_key(wait_id)
    }

    pub(crate) fn len(&self) -> usize {
        self.owners.len()
    }

    /// Preserve the former global BTreeMap's wait-ID iteration order, even
    /// when thread-ID order differs. Missing derived links are invariant
    /// failures, never silently omitted waiters.
    pub(crate) fn values(&self) -> impl Iterator<Item = &W> {
        self.owners.iter().map(|(id, owner)| {
            self.tasks
                .get(owner)
                .and_then(|records| records.get(id))
                .expect("wait owner index inconsistent")
        })
    }

    pub(crate) fn wait_ids_for_thread(&self, thread_id: u32) -> Vec<u64> {
        self.tasks
            .get(&thread_id)
            .map(|records| records.keys().copied().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Wait {
        id: u64,
        thread: u32,
        payload: u32,
    }
    impl TaskWaitRecord for Wait {
        fn wait_id(&self) -> u64 {
            self.id
        }
        fn owner_thread_id(&self) -> u32 {
            self.thread
        }
    }
    fn wait(id: u64, thread: u32) -> Wait {
        Wait {
            id,
            thread,
            payload: id as u32,
        }
    }

    #[test]
    fn task_wait_store_preserves_global_order_and_multiple_legacy_frames() {
        let mut store = TaskWaitStore::default();
        store.insert(3, wait(3, 1));
        store.insert(1, wait(1, 20));
        store.insert(2, wait(2, 20));
        assert_eq!(
            store.values().map(|wait| wait.id).collect::<Vec<_>>(),
            [1, 2, 3]
        );
        assert_eq!(store.wait_ids_for_thread(20), [1, 2]);
        assert_eq!(store.wait_ids_for_thread(1), [3]);
        assert!(store.wait_ids_for_thread(99).is_empty());
        assert_eq!(store.len(), 3);
        assert!(store.contains_key(&2));
        assert_eq!(store.get(&2), Some(&wait(2, 20)));
    }

    #[test]
    fn task_wait_store_removal_drops_only_selected_owner_frame() {
        let mut store = TaskWaitStore::default();
        store.insert(1, wait(1, 7));
        store.insert(2, wait(2, 7));
        store.insert(3, wait(3, 8));
        assert_eq!(store.remove(&1), Some(wait(1, 7)));
        assert_eq!(store.wait_ids_for_thread(7), [2]);
        assert_eq!(store.remove(&2), Some(wait(2, 7)));
        assert!(!store.tasks.contains_key(&7));
        assert_eq!(store.wait_ids_for_thread(8), [3]);
        assert_eq!(store.remove(&2), None);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn task_wait_store_replacement_and_clone_keep_owner_indexes_independent() {
        let mut store = TaskWaitStore::default();
        store.insert(1, wait(1, 7));
        let mut cloned = store.clone();
        assert_eq!(cloned.insert(1, wait(1, 8)), Some(wait(1, 7)));
        assert!(cloned.wait_ids_for_thread(7).is_empty());
        assert_eq!(cloned.wait_ids_for_thread(8), [1]);
        assert_eq!(store.wait_ids_for_thread(7), [1]);
        assert!(store.wait_ids_for_thread(8).is_empty());
        assert_eq!(cloned.remove(&1), Some(wait(1, 8)));
        assert!(cloned.tasks.is_empty());
        assert!(cloned.owners.is_empty());
    }

    #[test]
    #[should_panic(expected = "wait record identity mismatch")]
    fn task_wait_store_rejects_record_key_mismatch() {
        TaskWaitStore::default().insert(2, wait(1, 7));
    }

    #[test]
    fn task_wait_deadline_preserves_existing_wrap_infinite_and_zero_contract() {
        let deadline = TickDeadline::from_legacy(u32::MAX - 5, 10);
        assert!(!deadline.expired(3));
        assert!(deadline.expired(4));
        assert!(TickDeadline::from_legacy(123, 0).expired(123));
        assert!(!TickDeadline::from_legacy(123, u32::MAX).expired(122));
        assert!(!TickDeadline::from_legacy(123, u32::MAX).expired(u32::MAX));
        assert!(!TickDeadline::from_legacy(100, 25).expired(124));
        assert!(TickDeadline::from_legacy(100, 25).expired(125));
    }
}
