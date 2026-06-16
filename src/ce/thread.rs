use std::collections::{BTreeMap, BTreeSet};

pub const ERROR_SUCCESS: u32 = 0;
pub const ERROR_FILE_NOT_FOUND: u32 = 2;
pub const ERROR_ACCESS_DENIED: u32 = 5;
pub const ERROR_INVALID_HANDLE: u32 = 6;
pub const ERROR_NOT_ENOUGH_MEMORY: u32 = 8;
pub const ERROR_INVALID_ACCESS: u32 = 12;
pub const ERROR_NOT_SAME_DEVICE: u32 = 17;
pub const ERROR_NO_MORE_FILES: u32 = 18;
pub const ERROR_LOCK_VIOLATION: u32 = 33;
pub const ERROR_NOT_SUPPORTED: u32 = 50;
pub const ERROR_OUT_OF_STRUCTURES: u32 = 84;
pub const ERROR_INVALID_PARAMETER: u32 = 87;
pub const ERROR_SIGNAL_REFUSED: u32 = 156;
pub const ERROR_ALREADY_EXISTS: u32 = 183;
pub const ERROR_NOT_OWNER: u32 = 288;
pub const ERROR_INVALID_WINDOW_HANDLE: u32 = 1400;
pub const ERROR_CLASS_DOES_NOT_EXIST: u32 = 1411;
pub const ERROR_DLL_INIT_FAILED: u32 = 1114;
pub const ERROR_RESOURCE_NAME_NOT_FOUND: u32 = 1814;
pub const TLS_MINIMUM_AVAILABLE: u32 = 64;
pub const TLS_OUT_OF_INDEXES: u32 = u32::MAX;

const TLS_FUNC_ALLOC: u32 = 0;
const TLS_FUNC_FREE: u32 = 1;
const TLS_RESERVED_SLOTS: u32 = 4;

#[derive(Debug, Clone, Default)]
pub struct ThreadSystem {
    last_error_by_thread: BTreeMap<u32, u32>,
    tls_by_thread: BTreeMap<u32, BTreeMap<u32, u32>>,
    allocated_tls_slots: BTreeSet<u32>,
    next_guest_thread_id: u32,
}

impl ThreadSystem {
    pub fn allocate_guest_thread_id(&mut self) -> u32 {
        if self.next_guest_thread_id < 2 {
            self.next_guest_thread_id = 2;
        }
        let thread_id = self.next_guest_thread_id;
        self.next_guest_thread_id = self.next_guest_thread_id.saturating_add(1);
        thread_id
    }

    pub fn is_primary_thread(&self, thread_id: u32) -> bool {
        thread_id == 2
    }

    pub fn get_last_error(&self, thread_id: u32) -> u32 {
        self.last_error_by_thread
            .get(&thread_id)
            .copied()
            .unwrap_or(ERROR_SUCCESS)
    }

    pub fn set_last_error(&mut self, thread_id: u32, error: u32) {
        self.last_error_by_thread.insert(thread_id, error);
    }

    pub fn tls_get_value(&mut self, thread_id: u32, slot: u32) -> Option<u32> {
        if slot >= TLS_MINIMUM_AVAILABLE {
            self.set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            return None;
        }

        self.set_last_error(thread_id, ERROR_SUCCESS);
        Some(
            self.tls_by_thread
                .get(&thread_id)
                .and_then(|slots| slots.get(&slot))
                .copied()
                .unwrap_or(0),
        )
    }

    pub fn tls_set_value(&mut self, thread_id: u32, slot: u32, value: u32) -> bool {
        if slot >= TLS_MINIMUM_AVAILABLE {
            self.set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            return false;
        }

        self.tls_by_thread
            .entry(thread_id)
            .or_default()
            .insert(slot, value);
        true
    }

    pub fn tls_call(&mut self, thread_id: u32, kind: u32, slot: u32) -> u32 {
        let result = match kind {
            TLS_FUNC_ALLOC => self.tls_alloc(),
            TLS_FUNC_FREE => u32::from(self.tls_free(slot)),
            _ => 0,
        };
        if result == 0 || result == TLS_OUT_OF_INDEXES {
            self.set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        }
        result
    }

    fn tls_alloc(&mut self) -> u32 {
        for slot in TLS_RESERVED_SLOTS..TLS_MINIMUM_AVAILABLE {
            if self.allocated_tls_slots.insert(slot) {
                return slot;
            }
        }
        TLS_OUT_OF_INDEXES
    }

    fn tls_free(&mut self, slot: u32) -> bool {
        if !(TLS_RESERVED_SLOTS..TLS_MINIMUM_AVAILABLE).contains(&slot) {
            return false;
        }
        if !self.allocated_tls_slots.remove(&slot) {
            return false;
        }
        for thread_slots in self.tls_by_thread.values_mut() {
            thread_slots.remove(&slot);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tls_call_allocates_from_first_unreserved_slot() {
        let mut threads = ThreadSystem::default();

        assert_eq!(threads.tls_call(1, TLS_FUNC_ALLOC, 0), 4);
        assert_eq!(threads.tls_call(1, TLS_FUNC_ALLOC, 0), 5);
        assert_eq!(threads.get_last_error(1), ERROR_SUCCESS);
    }

    #[test]
    fn tls_call_free_clears_values_for_all_threads() {
        let mut threads = ThreadSystem::default();
        let slot = threads.tls_call(1, TLS_FUNC_ALLOC, 0);
        assert!(threads.tls_set_value(1, slot, 0x1234));
        assert!(threads.tls_set_value(2, slot, 0x5678));

        assert_eq!(threads.tls_call(1, TLS_FUNC_FREE, slot), 1);

        assert_eq!(threads.tls_get_value(1, slot), Some(0));
        assert_eq!(threads.tls_get_value(2, slot), Some(0));
    }

    #[test]
    fn tls_call_rejects_freeing_reserved_or_unused_slots() {
        let mut threads = ThreadSystem::default();

        assert_eq!(threads.tls_call(1, TLS_FUNC_FREE, 3), 0);
        assert_eq!(threads.get_last_error(1), ERROR_INVALID_PARAMETER);
        assert_eq!(threads.tls_call(1, TLS_FUNC_FREE, 4), 0);
        assert_eq!(threads.get_last_error(1), ERROR_INVALID_PARAMETER);
    }
}
