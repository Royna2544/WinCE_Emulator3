use std::collections::BTreeMap;

pub const ERROR_SUCCESS: u32 = 0;
pub const ERROR_INVALID_HANDLE: u32 = 6;
pub const ERROR_INVALID_PARAMETER: u32 = 87;
pub const ERROR_RESOURCE_NAME_NOT_FOUND: u32 = 1814;
pub const TLS_MINIMUM_AVAILABLE: u32 = 64;

#[derive(Debug, Clone, Default)]
pub struct ThreadSystem {
    last_error_by_thread: BTreeMap<u32, u32>,
    tls_by_thread: BTreeMap<u32, BTreeMap<u32, u32>>,
}

impl ThreadSystem {
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
}
