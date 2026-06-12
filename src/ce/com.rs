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
    pub clsid: Option<[u8; 16]>,
    pub factory_token: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComObject {
    pub handle: u32,
    pub clsid_ptr: u32,
    pub clsid: Option<[u8; 16]>,
    pub iid_ptr: u32,
    pub iid: Option<[u8; 16]>,
}

#[derive(Debug, Clone, Default)]
pub struct ComSystem {
    apartments: BTreeMap<u32, ThreadComState>,
    classes: BTreeMap<u32, ComClass>,
    classes_by_guid: BTreeMap<[u8; 16], ComClass>,
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
                clsid: None,
                factory_token,
            },
        );
    }

    pub fn register_class_guid(&mut self, clsid: [u8; 16], factory_token: u32) {
        self.classes_by_guid.insert(
            clsid,
            ComClass {
                clsid_ptr: 0,
                clsid: Some(clsid),
                factory_token,
            },
        );
    }

    pub fn co_create_instance(&mut self, clsid_ptr: u32, iid_ptr: u32) -> Result<u32, u32> {
        if clsid_ptr == 0 || iid_ptr == 0 {
            return Err(E_POINTER);
        }
        let Some(class) = self.classes.get(&clsid_ptr).copied() else {
            return Err(REGDB_E_CLASSNOTREG);
        };
        let handle = self.object_handle_for_class(class);
        self.objects.insert(
            handle,
            ComObject {
                handle,
                clsid_ptr,
                clsid: None,
                iid_ptr,
                iid: None,
            },
        );
        Ok(handle)
    }

    pub fn co_create_instance_guid(
        &mut self,
        clsid_ptr: u32,
        clsid: [u8; 16],
        iid_ptr: u32,
        iid: [u8; 16],
    ) -> Result<u32, u32> {
        if clsid_ptr == 0 || iid_ptr == 0 {
            return Err(E_POINTER);
        }
        let Some(class) = self.classes_by_guid.get(&clsid).copied() else {
            return Err(REGDB_E_CLASSNOTREG);
        };
        let handle = self.object_handle_for_class(class);
        self.objects.insert(
            handle,
            ComObject {
                handle,
                clsid_ptr,
                clsid: Some(clsid),
                iid_ptr,
                iid: Some(iid),
            },
        );
        Ok(handle)
    }

    pub fn co_create_instance_guid_values(
        &mut self,
        clsid: [u8; 16],
        iid: [u8; 16],
    ) -> Result<u32, u32> {
        let Some(class) = self.classes_by_guid.get(&clsid).copied() else {
            return Err(REGDB_E_CLASSNOTREG);
        };
        let handle = self.object_handle_for_class(class);
        self.objects.insert(
            handle,
            ComObject {
                handle,
                clsid_ptr: 0,
                clsid: Some(clsid),
                iid_ptr: 0,
                iid: Some(iid),
            },
        );
        Ok(handle)
    }

    fn object_handle_for_class(&mut self, class: ComClass) -> u32 {
        if class.factory_token != 0 {
            return class.factory_token;
        }
        if self.next_object == 0 {
            self.next_object = 0x000a_0000;
        }
        let handle = self.next_object;
        self.next_object += 4;
        handle
    }

    pub fn object(&self, handle: u32) -> Option<ComObject> {
        self.objects.get(&handle).copied()
    }
}
