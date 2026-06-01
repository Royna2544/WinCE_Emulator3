use std::collections::BTreeMap;

pub const S_OK: u32 = 0x0000_0000;
pub const S_FALSE: u32 = 0x0000_0001;
pub const E_POINTER: u32 = 0x8000_4003;
pub const REGDB_E_CLASSNOTREG: u32 = 0x8004_0154;
pub const RPC_E_CHANGED_MODE: u32 = 0x8001_0106;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComApartment {
    SingleThreaded,
    MultiThreaded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComClass {
    pub clsid_ptr: u32,
    pub factory_token: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComObject {
    pub handle: u32,
    pub clsid_ptr: u32,
    pub iid_ptr: u32,
}

#[derive(Debug, Clone, Default)]
pub struct ComSystem {
    apartments: BTreeMap<u32, ThreadComState>,
    classes: BTreeMap<u32, ComClass>,
    objects: BTreeMap<u32, ComObject>,
    next_object: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ThreadComState {
    apartment: ComApartment,
    depth: u32,
}

impl ComSystem {
    pub fn co_initialize_ex(&mut self, thread_id: u32, coinit: u32) -> u32 {
        let apartment = if coinit & 0x2 != 0 {
            ComApartment::SingleThreaded
        } else {
            ComApartment::MultiThreaded
        };
        match self.apartments.get_mut(&thread_id) {
            Some(state) if state.apartment == apartment => {
                state.depth = state.depth.saturating_add(1);
                S_FALSE
            }
            Some(_) => RPC_E_CHANGED_MODE,
            None => {
                self.apartments.insert(
                    thread_id,
                    ThreadComState {
                        apartment,
                        depth: 1,
                    },
                );
                S_OK
            }
        }
    }

    pub fn co_uninitialize(&mut self, thread_id: u32) {
        let Some(state) = self.apartments.get_mut(&thread_id) else {
            return;
        };
        state.depth = state.depth.saturating_sub(1);
        if state.depth == 0 {
            self.apartments.remove(&thread_id);
        }
    }

    pub fn register_class(&mut self, clsid_ptr: u32, factory_token: u32) {
        self.classes.insert(
            clsid_ptr,
            ComClass {
                clsid_ptr,
                factory_token,
            },
        );
    }

    pub fn co_create_instance(&mut self, clsid_ptr: u32, iid_ptr: u32) -> Result<u32, u32> {
        if clsid_ptr == 0 || iid_ptr == 0 {
            return Err(E_POINTER);
        }
        if !self.classes.contains_key(&clsid_ptr) {
            return Err(REGDB_E_CLASSNOTREG);
        }
        if self.next_object == 0 {
            self.next_object = 0x000a_0000;
        }
        let handle = self.next_object;
        self.next_object += 4;
        self.objects.insert(
            handle,
            ComObject {
                handle,
                clsid_ptr,
                iid_ptr,
            },
        );
        Ok(handle)
    }

    pub fn object(&self, handle: u32) -> Option<ComObject> {
        self.objects.get(&handle).copied()
    }
}
