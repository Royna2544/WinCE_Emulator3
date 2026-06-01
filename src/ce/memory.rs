use std::collections::BTreeMap;

pub const HEAP_MAX_ALLOC: u32 = 0x4000_0000;
pub const PROCESS_HEAP_HANDLE: u32 = 0x0001_0000;

pub const LMEM_ZEROINIT: u32 = 0x0040;
pub const LMEM_MODIFY: u32 = 0x0080;
pub const LMEM_VALID_FLAGS: u32 = 0x0f72;

pub const HEAP_NO_SERIALIZE: u32 = 0x0000_0001;
pub const HEAP_GENERATE_EXCEPTIONS: u32 = 0x0000_0004;
pub const HEAP_ZERO_MEMORY: u32 = 0x0000_0008;
pub const HEAP_REALLOC_IN_PLACE_ONLY: u32 = 0x0000_0010;
pub const HEAP_SHARED_READONLY: u32 = 0x0000_8000;

pub const MEM_COMMIT: u32 = 0x0000_1000;
pub const MEM_RESERVE: u32 = 0x0000_2000;
pub const MEM_DECOMMIT: u32 = 0x0000_4000;
pub const MEM_RELEASE: u32 = 0x0000_8000;

const POINTER_FLOOR: u32 = 0x0001_0000;
const HEAP_PTR_BASE: u32 = 0x3000_0000;
const VIRTUAL_PTR_BASE: u32 = 0x5000_0000;
const HEAP_ALIGN: u32 = 16;
const VIRTUAL_ALIGN: u32 = 0x0001_0000;

#[derive(Debug, Clone)]
pub struct MemorySystem {
    process_heap: u32,
    next_heap: u32,
    next_heap_ptr: u32,
    next_virtual_ptr: u32,
    heaps: BTreeMap<u32, Heap>,
    allocations: BTreeMap<u32, MemoryAllocation>,
    virtual_allocations: BTreeMap<u32, VirtualAllocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heap {
    pub handle: u32,
    pub options: u32,
    pub initial_size: u32,
    pub maximum_size: u32,
    pub destroyed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryAllocation {
    pub ptr: u32,
    pub heap: u32,
    pub requested_size: u32,
    pub actual_size: u32,
    pub zeroed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualAllocation {
    pub base: u32,
    pub size: u32,
    pub allocation_type: u32,
    pub protect: u32,
}

impl Default for MemorySystem {
    fn default() -> Self {
        let mut heaps = BTreeMap::new();
        heaps.insert(
            PROCESS_HEAP_HANDLE,
            Heap {
                handle: PROCESS_HEAP_HANDLE,
                options: 0,
                initial_size: 0,
                maximum_size: 0,
                destroyed: false,
            },
        );
        Self {
            process_heap: PROCESS_HEAP_HANDLE,
            next_heap: PROCESS_HEAP_HANDLE + 0x1000,
            next_heap_ptr: HEAP_PTR_BASE,
            next_virtual_ptr: VIRTUAL_PTR_BASE,
            heaps,
            allocations: BTreeMap::new(),
            virtual_allocations: BTreeMap::new(),
        }
    }
}

impl MemorySystem {
    pub fn get_process_heap(&self) -> u32 {
        self.process_heap
    }

    pub fn local_alloc(&mut self, flags: u32, bytes: u32) -> Option<u32> {
        if flags & !LMEM_VALID_FLAGS != 0 {
            return None;
        }
        let heap_flags = if flags & LMEM_ZEROINIT != 0 {
            HEAP_ZERO_MEMORY
        } else {
            0
        };
        self.heap_alloc(self.process_heap, heap_flags, bytes)
    }

    pub fn local_re_alloc(&mut self, ptr: u32, bytes: u32, flags: u32) -> Option<u32> {
        if flags & !(LMEM_VALID_FLAGS | LMEM_MODIFY) != 0 {
            return None;
        }
        let heap_flags = if flags & LMEM_ZEROINIT != 0 {
            HEAP_ZERO_MEMORY
        } else {
            0
        };
        self.heap_re_alloc(self.process_heap, heap_flags, ptr, bytes)
    }

    pub fn local_size(&self, ptr: u32) -> Option<u32> {
        self.heap_size(self.process_heap, 0, ptr)
    }

    pub fn local_free(&mut self, ptr: u32) -> bool {
        ptr == 0 || self.heap_free(self.process_heap, 0, ptr)
    }

    pub fn heap_create(
        &mut self,
        options: u32,
        initial_size: u32,
        maximum_size: u32,
    ) -> Option<u32> {
        if options & !(HEAP_NO_SERIALIZE | HEAP_SHARED_READONLY) != 0 {
            return None;
        }
        let handle = self.next_heap;
        self.next_heap = self.next_heap.saturating_add(0x1000);
        self.heaps.insert(
            handle,
            Heap {
                handle,
                options,
                initial_size,
                maximum_size,
                destroyed: false,
            },
        );
        Some(handle)
    }

    pub fn heap_destroy(&mut self, heap: u32) -> bool {
        if heap == self.process_heap {
            return false;
        }
        let Some(item) = self.heaps.get_mut(&heap) else {
            return false;
        };
        if item.destroyed {
            return false;
        }
        item.destroyed = true;
        self.allocations
            .retain(|_, allocation| allocation.heap != heap);
        true
    }

    pub fn heap_alloc(&mut self, heap: u32, flags: u32, bytes: u32) -> Option<u32> {
        if flags & !(HEAP_NO_SERIALIZE | HEAP_ZERO_MEMORY) != 0 || !self.is_live_heap(heap) {
            return None;
        }
        let actual_size = sanitize_size(bytes)?;
        let ptr = self.allocate_heap_ptr(actual_size)?;
        self.allocations.insert(
            ptr,
            MemoryAllocation {
                ptr,
                heap,
                requested_size: bytes,
                actual_size,
                zeroed: flags & HEAP_ZERO_MEMORY != 0,
            },
        );
        Some(ptr)
    }

    pub fn heap_re_alloc(&mut self, heap: u32, flags: u32, ptr: u32, bytes: u32) -> Option<u32> {
        if flags
            & !(HEAP_GENERATE_EXCEPTIONS
                | HEAP_NO_SERIALIZE
                | HEAP_REALLOC_IN_PLACE_ONLY
                | HEAP_ZERO_MEMORY)
            != 0
            || flags & HEAP_GENERATE_EXCEPTIONS != 0
            || !self.is_live_heap(heap)
        {
            return None;
        }
        let actual_size = sanitize_size(bytes)?;
        let allocation = self.allocations.get_mut(&ptr)?;
        if allocation.heap != heap || ptr < POINTER_FLOOR {
            return None;
        }
        allocation.requested_size = bytes;
        allocation.actual_size = actual_size;
        allocation.zeroed |= flags & HEAP_ZERO_MEMORY != 0;
        Some(ptr)
    }

    pub fn heap_free(&mut self, heap: u32, flags: u32, ptr: u32) -> bool {
        if flags & !HEAP_NO_SERIALIZE != 0 || ptr < POINTER_FLOOR || !self.is_live_heap(heap) {
            return false;
        }
        matches!(self.allocations.remove(&ptr), Some(allocation) if allocation.heap == heap)
    }

    pub fn heap_size(&self, heap: u32, flags: u32, ptr: u32) -> Option<u32> {
        if flags & !HEAP_NO_SERIALIZE != 0 || ptr < POINTER_FLOOR || !self.is_live_heap(heap) {
            return None;
        }
        let allocation = self.allocations.get(&ptr)?;
        (allocation.heap == heap).then_some(allocation.actual_size)
    }

    pub fn heap_validate(&self, heap: u32, flags: u32, ptr: u32) -> bool {
        if flags & !HEAP_NO_SERIALIZE != 0 || !self.is_live_heap(heap) {
            return false;
        }
        ptr == 0 || (ptr >= POINTER_FLOOR && self.allocations.contains_key(&ptr))
    }

    pub fn virtual_alloc(
        &mut self,
        address: u32,
        size: u32,
        allocation_type: u32,
        protect: u32,
    ) -> Option<u32> {
        if size == 0 || allocation_type & (MEM_COMMIT | MEM_RESERVE) == 0 {
            return None;
        }
        let size = align_up(size, VIRTUAL_ALIGN)?;
        let base = if address == 0 {
            let base = self.next_virtual_ptr;
            self.next_virtual_ptr = self.next_virtual_ptr.checked_add(size)?;
            base
        } else {
            align_down(address, VIRTUAL_ALIGN)
        };
        if self.virtual_overlaps(base, size) {
            return None;
        }
        self.virtual_allocations.insert(
            base,
            VirtualAllocation {
                base,
                size,
                allocation_type,
                protect,
            },
        );
        Some(base)
    }

    pub fn virtual_free(&mut self, address: u32, size: u32, free_type: u32) -> bool {
        match free_type {
            MEM_RELEASE => size == 0 && self.virtual_allocations.remove(&address).is_some(),
            MEM_DECOMMIT => self
                .virtual_allocations
                .get(&address)
                .is_some_and(|allocation| size <= allocation.size),
            _ => false,
        }
    }

    pub fn allocation(&self, ptr: u32) -> Option<&MemoryAllocation> {
        self.allocations.get(&ptr)
    }

    pub fn allocations(&self) -> impl Iterator<Item = &MemoryAllocation> {
        self.allocations.values()
    }

    pub fn virtual_allocation(&self, base: u32) -> Option<&VirtualAllocation> {
        self.virtual_allocations.get(&base)
    }

    pub fn virtual_allocations(&self) -> impl Iterator<Item = &VirtualAllocation> {
        self.virtual_allocations.values()
    }

    fn is_live_heap(&self, heap: u32) -> bool {
        self.heaps.get(&heap).is_some_and(|item| !item.destroyed)
    }

    fn allocate_heap_ptr(&mut self, size: u32) -> Option<u32> {
        let ptr = self.next_heap_ptr;
        let step = align_up(size.max(1), HEAP_ALIGN)?;
        self.next_heap_ptr = self.next_heap_ptr.checked_add(step)?;
        Some(ptr)
    }

    fn virtual_overlaps(&self, base: u32, size: u32) -> bool {
        let end = base.saturating_add(size);
        self.virtual_allocations.values().any(|allocation| {
            let other_end = allocation.base.saturating_add(allocation.size);
            base < other_end && end > allocation.base
        })
    }
}

fn sanitize_size(size: u32) -> Option<u32> {
    if size > HEAP_MAX_ALLOC {
        None
    } else {
        Some(size.max(1))
    }
}

fn align_up(value: u32, alignment: u32) -> Option<u32> {
    let mask = alignment.checked_sub(1)?;
    value.checked_add(mask).map(|value| value & !mask)
}

fn align_down(value: u32, alignment: u32) -> u32 {
    value & !(alignment - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_alloc_size_and_free_follow_ce_return_shape() {
        let mut memory = MemorySystem::default();
        let ptr = memory.local_alloc(LMEM_ZEROINIT, 0).unwrap();

        assert!(ptr >= POINTER_FLOOR);
        assert_eq!(memory.local_size(ptr), Some(1));
        assert!(memory.allocation(ptr).unwrap().zeroed);
        assert!(memory.local_free(ptr));
        assert_eq!(memory.local_size(ptr), None);
        assert!(memory.local_free(0));
    }

    #[test]
    fn heap_alloc_requires_live_heap_and_valid_flags() {
        let mut memory = MemorySystem::default();
        let heap = memory.heap_create(HEAP_NO_SERIALIZE, 0x1000, 0).unwrap();
        let ptr = memory.heap_alloc(heap, HEAP_ZERO_MEMORY, 32).unwrap();

        assert_eq!(memory.heap_size(heap, 0, ptr), Some(32));
        assert!(memory.heap_validate(heap, 0, ptr));
        assert!(!memory.heap_free(heap, HEAP_GENERATE_EXCEPTIONS, ptr));
        assert!(memory.heap_free(heap, 0, ptr));
        assert!(memory.heap_destroy(heap));
        assert!(memory.heap_alloc(heap, 0, 8).is_none());
    }
}
