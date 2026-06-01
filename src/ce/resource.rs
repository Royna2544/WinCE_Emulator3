use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceId {
    Integer(u16),
    NamePtr(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceEntry {
    pub module: u32,
    pub name: ResourceId,
    pub kind: ResourceId,
    pub data_ptr: u32,
    pub size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceString {
    pub module: u32,
    pub id: u32,
    pub text: String,
    pub data_ptr: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ResourceSystem {
    next_handle: u32,
    by_key: BTreeMap<(u32, ResourceId, ResourceId), u32>,
    entries: BTreeMap<u32, ResourceEntry>,
    strings: BTreeMap<(u32, u32), ResourceString>,
}

impl Default for ResourceSystem {
    fn default() -> Self {
        Self {
            next_handle: 0x0009_0000,
            by_key: BTreeMap::new(),
            entries: BTreeMap::new(),
            strings: BTreeMap::new(),
        }
    }
}

impl ResourceSystem {
    pub fn register(
        &mut self,
        module: u32,
        name: ResourceId,
        kind: ResourceId,
        data_ptr: u32,
        size: u32,
    ) -> u32 {
        let handle = self.next_handle;
        self.next_handle += 4;
        self.by_key.insert((module, name, kind), handle);
        self.entries.insert(
            handle,
            ResourceEntry {
                module,
                name,
                kind,
                data_ptr,
                size,
            },
        );
        handle
    }

    pub fn find_resource(&self, module: u32, name: ResourceId, kind: ResourceId) -> Option<u32> {
        self.by_key.get(&(module, name, kind)).copied()
    }

    pub fn load_resource(&self, handle: u32) -> Option<u32> {
        Some(self.entries.get(&handle)?.data_ptr)
    }

    pub fn lock_resource(&self, handle: u32) -> Option<u32> {
        self.load_resource(handle)
    }

    pub fn sizeof_resource(&self, handle: u32) -> Option<u32> {
        Some(self.entries.get(&handle)?.size)
    }

    pub fn register_string(
        &mut self,
        module: u32,
        id: u32,
        text: impl Into<String>,
        data_ptr: Option<u32>,
    ) {
        self.strings.insert(
            (module, id),
            ResourceString {
                module,
                id,
                text: text.into(),
                data_ptr,
            },
        );
    }

    pub fn load_string(&self, module: u32, id: u32) -> Option<&ResourceString> {
        self.strings.get(&(module, id))
    }
}

impl ResourceId {
    pub fn from_guest_arg(value: u32) -> Self {
        if value <= 0xffff {
            Self::Integer(value as u16)
        } else {
            Self::NamePtr(value)
        }
    }
}
