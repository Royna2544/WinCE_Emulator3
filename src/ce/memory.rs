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
    heap_generation: u64,
    virtual_generation: u64,
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
pub struct Reallocation {
    pub ptr: u32,
    pub old_ptr: u32,
    pub old_actual_size: u32,
    pub new_actual_size: u32,
    pub moved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualAllocation {
    pub base: u32,
    pub size: u32,
    pub allocation_type: u32,
    pub protect: u32,
    pub initial_bytes: Vec<u8>,
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
            heap_generation: 0,
            virtual_generation: 0,
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
        self.local_re_alloc_detail(ptr, bytes, flags)
            .map(|result| result.ptr)
    }

    pub fn local_re_alloc_detail(
        &mut self,
        ptr: u32,
        bytes: u32,
        flags: u32,
    ) -> Option<Reallocation> {
        if flags & !(LMEM_VALID_FLAGS | LMEM_MODIFY) != 0 {
            return None;
        }
        let heap_flags = if flags & LMEM_ZEROINIT != 0 {
            HEAP_ZERO_MEMORY
        } else {
            0
        };
        self.heap_re_alloc_detail(self.process_heap, heap_flags, ptr, bytes)
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
        self.bump_heap_generation();
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
        self.bump_heap_generation();
        true
    }

    pub fn heap_alloc(&mut self, heap: u32, flags: u32, bytes: u32) -> Option<u32> {
        if flags & !(HEAP_GENERATE_EXCEPTIONS | HEAP_NO_SERIALIZE | HEAP_ZERO_MEMORY) != 0
            || !self.is_live_heap(heap)
        {
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
        self.bump_heap_generation();
        Some(ptr)
    }

    pub fn heap_re_alloc(&mut self, heap: u32, flags: u32, ptr: u32, bytes: u32) -> Option<u32> {
        self.heap_re_alloc_detail(heap, flags, ptr, bytes)
            .map(|result| result.ptr)
    }

    pub fn heap_re_alloc_detail(
        &mut self,
        heap: u32,
        flags: u32,
        ptr: u32,
        bytes: u32,
    ) -> Option<Reallocation> {
        if flags
            & !(HEAP_GENERATE_EXCEPTIONS
                | HEAP_NO_SERIALIZE
                | HEAP_REALLOC_IN_PLACE_ONLY
                | HEAP_ZERO_MEMORY)
            != 0
            || !self.is_live_heap(heap)
        {
            return None;
        }
        let actual_size = sanitize_size(bytes)?;
        let allocation = self.allocations.get(&ptr)?.clone();
        if allocation.heap != heap || ptr < POINTER_FLOOR {
            return None;
        }
        let old_actual_size = allocation.actual_size;
        if actual_size <= allocation.actual_size {
            let allocation = self.allocations.get_mut(&ptr)?;
            allocation.requested_size = bytes;
            allocation.actual_size = actual_size;
            allocation.zeroed |= flags & HEAP_ZERO_MEMORY != 0;
            self.bump_heap_generation();
            return Some(Reallocation {
                ptr,
                old_ptr: ptr,
                old_actual_size,
                new_actual_size: actual_size,
                moved: false,
            });
        }
        if flags & HEAP_REALLOC_IN_PLACE_ONLY != 0 {
            return None;
        }
        let new_ptr = self.allocate_heap_ptr(actual_size)?;
        self.allocations.remove(&ptr);
        self.allocations.insert(
            new_ptr,
            MemoryAllocation {
                ptr: new_ptr,
                heap,
                requested_size: bytes,
                actual_size,
                zeroed: allocation.zeroed || flags & HEAP_ZERO_MEMORY != 0,
            },
        );
        self.bump_heap_generation();
        Some(Reallocation {
            ptr: new_ptr,
            old_ptr: ptr,
            old_actual_size,
            new_actual_size: actual_size,
            moved: true,
        })
    }

    pub fn heap_free(&mut self, heap: u32, flags: u32, ptr: u32) -> bool {
        if flags & !HEAP_NO_SERIALIZE != 0 || ptr < POINTER_FLOOR || !self.is_live_heap(heap) {
            return false;
        }
        let Some(allocation) = self.allocations.remove(&ptr) else {
            return false;
        };
        if allocation.heap != heap {
            self.allocations.insert(ptr, allocation);
            return false;
        }
        self.bump_heap_generation();
        true
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
                initial_bytes: Vec::new(),
            },
        );
        self.bump_virtual_generation();
        Some(base)
    }

    pub fn set_virtual_initial_bytes(&mut self, base: u32, bytes: Vec<u8>) -> bool {
        let Some(allocation) = self.virtual_allocations.get_mut(&base) else {
            return false;
        };
        allocation.initial_bytes = bytes;
        self.bump_virtual_generation();
        true
    }

    pub fn virtual_free(&mut self, address: u32, size: u32, free_type: u32) -> bool {
        match free_type {
            MEM_RELEASE => {
                if size != 0 || self.virtual_allocations.remove(&address).is_none() {
                    return false;
                }
                self.bump_virtual_generation();
                true
            }
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

    pub fn heap_high_water_mark(&self) -> u32 {
        self.next_heap_ptr
    }

    pub fn heap_generation(&self) -> u64 {
        self.heap_generation
    }

    pub fn virtual_generation(&self) -> u64 {
        self.virtual_generation
    }

    pub fn contains_allocated_range(&self, ptr: u32, size: u32) -> bool {
        if size == 0 {
            return true;
        }
        let Some(end) = ptr.checked_add(size) else {
            return false;
        };
        self.allocations.values().any(|allocation| {
            let Some(allocation_end) = allocation.ptr.checked_add(allocation.actual_size) else {
                return false;
            };
            ptr >= allocation.ptr && end <= allocation_end
        }) || self.virtual_allocations.values().any(|allocation| {
            let Some(allocation_end) = allocation.base.checked_add(allocation.size) else {
                return false;
            };
            ptr >= allocation.base && end <= allocation_end
        })
    }

    pub fn heap_range_status(&self, ptr: u32, size: u32) -> Option<bool> {
        let end = ptr.checked_add(size)?;
        self.allocations.values().find_map(|allocation| {
            let actual_end = allocation.ptr.checked_add(allocation.actual_size)?;
            if ptr < allocation.ptr || ptr >= actual_end {
                return None;
            }
            let requested_end = allocation
                .ptr
                .checked_add(allocation.requested_size.max(1))?;
            Some(end <= requested_end)
        })
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

    fn bump_heap_generation(&mut self) {
        self.heap_generation = self.heap_generation.wrapping_add(1);
    }

    fn bump_virtual_generation(&mut self) {
        self.virtual_generation = self.virtual_generation.wrapping_add(1);
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

    #[test]
    fn realloc_growth_moves_to_avoid_overlapping_later_allocations() {
        let mut memory = MemorySystem::default();
        let first = memory.heap_alloc(PROCESS_HEAP_HANDLE, 0, 16).unwrap();
        let second = memory.heap_alloc(PROCESS_HEAP_HANDLE, 0, 16).unwrap();

        let grown = memory
            .heap_re_alloc_detail(PROCESS_HEAP_HANDLE, 0, first, 64)
            .unwrap();

        assert!(grown.moved);
        assert_ne!(grown.ptr, first);
        assert!(grown.ptr >= second + 16);
        assert_eq!(
            memory.heap_size(PROCESS_HEAP_HANDLE, 0, grown.ptr),
            Some(64)
        );
        assert_eq!(memory.heap_size(PROCESS_HEAP_HANDLE, 0, first), None);
        assert_eq!(memory.heap_size(PROCESS_HEAP_HANDLE, 0, second), Some(16));
    }

    #[test]
    fn contains_allocated_range_checks_heap_and_virtual_bounds() {
        let mut memory = MemorySystem::default();
        let heap_ptr = memory.heap_alloc(PROCESS_HEAP_HANDLE, 0, 16).unwrap();
        let virtual_ptr = memory
            .virtual_alloc(0, 0x1234, MEM_COMMIT | MEM_RESERVE, 4)
            .unwrap();

        assert!(memory.contains_allocated_range(heap_ptr, 16));
        assert!(memory.contains_allocated_range(heap_ptr + 4, 8));
        assert!(!memory.contains_allocated_range(heap_ptr + 8, 16));
        assert!(memory.contains_allocated_range(virtual_ptr, 0x1234));
        assert!(!memory.contains_allocated_range(virtual_ptr.wrapping_sub(1), 2));
    }

    #[test]
    fn virtual_alloc_rejects_zero_size_and_missing_type_flags() {
        let mut memory = MemorySystem::default();

        // size=0 → None regardless of type flags
        assert!(
            memory
                .virtual_alloc(0, 0, MEM_COMMIT | MEM_RESERVE, 4)
                .is_none()
        );

        // allocation_type=0 → None regardless of size
        assert!(memory.virtual_alloc(0, 0x1000, 0, 4).is_none());
    }

    #[test]
    fn virtual_free_release_requires_size_zero_and_valid_base() {
        let mut memory = MemorySystem::default();
        let base = memory
            .virtual_alloc(0, 0x1000, MEM_COMMIT | MEM_RESERVE, 4)
            .unwrap();

        // MEM_RELEASE with size!=0 → false
        assert!(!memory.virtual_free(base, 1, MEM_RELEASE));

        // MEM_RELEASE with correct size=0 on valid base → true
        assert!(memory.virtual_free(base, 0, MEM_RELEASE));

        // repeated release on now-freed base → false
        assert!(!memory.virtual_free(base, 0, MEM_RELEASE));
    }

    #[test]
    fn virtual_free_decommit_succeeds_for_valid_range_and_unknown_type_fails() {
        let mut memory = MemorySystem::default();
        // Allocate exactly one VIRTUAL_ALIGN-sized region (0x1_0000 bytes).
        let base = memory
            .virtual_alloc(0, VIRTUAL_ALIGN, MEM_COMMIT | MEM_RESERVE, 4)
            .unwrap();

        // MEM_DECOMMIT with size within allocation → true
        assert!(memory.virtual_free(base, VIRTUAL_ALIGN / 2, MEM_DECOMMIT));

        // MEM_DECOMMIT with size exceeding aligned allocation size → false
        assert!(!memory.virtual_free(base, VIRTUAL_ALIGN + 1, MEM_DECOMMIT));

        // unknown free_type → false
        assert!(!memory.virtual_free(base, 0, 0xdead));
    }

    #[test]
    fn local_re_alloc_grows_in_place_when_no_later_allocation_blocks() {
        let mut memory = MemorySystem::default();
        let ptr = memory.local_alloc(0, 16).unwrap();

        let grown = memory.local_re_alloc(ptr, 64, 0).unwrap();

        // With no subsequent allocation, the realloc may or may not move,
        // but the new size must be at least 64.
        assert_eq!(memory.local_size(grown), Some(64));
    }

    #[test]
    fn heap_range_status_distinguishes_requested_actual_and_outside() {
        let mut memory = MemorySystem::default();
        // 8-byte allocation: actual_size = align_up(8, 16) = 16, requested_size = 8.
        let ptr = memory.heap_alloc(PROCESS_HEAP_HANDLE, 0, 8).unwrap();

        // Within requested range → Some(true).
        assert_eq!(memory.heap_range_status(ptr, 8), Some(true));
        assert_eq!(memory.heap_range_status(ptr, 1), Some(true));

        // Past requested size but within actual allocation → Some(false).
        assert_eq!(memory.heap_range_status(ptr, 9), Some(false));

        // Completely outside (past actual end, and before ptr) → None.
        assert_eq!(memory.heap_range_status(ptr + 16, 1), None);
        assert_eq!(memory.heap_range_status(ptr - 1, 1), None);
    }
}
