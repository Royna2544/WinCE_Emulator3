use std::collections::BTreeMap;

use crate::ce::object::ProcessBirthKey;

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
    // Owner high-water marks are authoritative metadata. The global floor is
    // retained until native virtual pages are mapped by owner rather than VA.
    next_virtual_ptr: u32,
    next_virtual_ptr_by_owner: BTreeMap<ProcessBirthKey, u32>,
    heap_generation: u64,
    virtual_generation: u64,
    next_virtual_incarnation: u64,
    heaps: BTreeMap<u32, Heap>,
    allocations: BTreeMap<u32, MemoryAllocation>,
    virtual_allocations: BTreeMap<(ProcessBirthKey, u32), VirtualAllocation>,
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

/// A monotonic address reservation, not yet a live guest allocation.
/// Guest-backed callers must map and initialize its bytes before committing.
/// Abort never reuses a destination that a failed memory write may have dirtied.
#[derive(Debug)]
#[must_use = "prepared allocations must be committed or aborted"]
pub struct PreparedAllocation {
    allocation: MemoryAllocation,
}

impl PreparedAllocation {
    pub fn allocation(&self) -> &MemoryAllocation {
        &self.allocation
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reallocation {
    pub ptr: u32,
    pub old_ptr: u32,
    pub old_actual_size: u32,
    pub new_actual_size: u32,
    pub moved: bool,
}

/// A heap reallocation whose destination address is reserved but whose
/// allocation metadata has not been committed yet.
///
/// Guest-memory users must map and populate the destination successfully
/// before calling [`MemorySystem::commit_reallocation`]. Dropping or aborting
/// the token leaves the old allocation live. A moved destination remains an
/// unused address hole: failed guest writes may have changed its bytes, so it
/// is not reused; address reuse is deliberately outside this byte-effect boundary.
#[derive(Debug)]
#[must_use = "prepared reallocations must be committed or aborted"]
pub struct PreparedReallocation {
    result: Reallocation,
    original_allocation: MemoryAllocation,
    requested_size: u32,
    zero_requested: bool,
}

impl PreparedReallocation {
    pub fn result(&self) -> &Reallocation {
        &self.result
    }

    pub fn copy_len(&self) -> u32 {
        if self.result.moved {
            self.result.old_actual_size.min(self.result.new_actual_size)
        } else {
            0
        }
    }

    pub fn zero_range(&self) -> Option<(u32, u32)> {
        let len = self
            .result
            .new_actual_size
            .saturating_sub(self.result.old_actual_size);
        if !self.zero_requested || len == 0 {
            return None;
        }
        self.result
            .ptr
            .checked_add(self.result.old_actual_size)
            .map(|start| (start, len))
    }
}

/// Describes the producer of today's copy-backed virtual allocation. This is
/// metadata only: native mapping remains unchanged until the backing migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualAllocationSource {
    Private,
    PrivateExCompatibility,
    FileViewLegacy,
    LocaleViewLegacy,
    ToolhelpSnapshotLegacy,
    SharedCompatibility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualAllocation {
    pub base: u32,
    pub size: u32,
    pub allocation_type: u32,
    pub protect: u32,
    pub initial_bytes: Vec<u8>,
    pub owner: ProcessBirthKey,
    pub incarnation: u64,
    pub source: VirtualAllocationSource,
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
            next_virtual_ptr_by_owner: BTreeMap::new(),
            heap_generation: 0,
            virtual_generation: 0,
            next_virtual_incarnation: 1,
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

    /// Metadata-only convenience; raw APIs must map/initialize before commit.
    pub fn local_alloc(&mut self, flags: u32, bytes: u32) -> Option<u32> {
        let prepared = self.prepare_local_alloc(flags, bytes)?;
        self.commit_allocation(prepared)
    }

    pub fn prepare_local_alloc(&mut self, flags: u32, bytes: u32) -> Option<PreparedAllocation> {
        if flags & !LMEM_VALID_FLAGS != 0 {
            return None;
        }
        let heap_flags = if flags & LMEM_ZEROINIT != 0 {
            HEAP_ZERO_MEMORY
        } else {
            0
        };
        self.prepare_heap_alloc(self.process_heap, heap_flags, bytes)
    }

    /// Metadata-only local reallocation; it does not copy or zero guest bytes.
    pub fn local_re_alloc(&mut self, ptr: u32, bytes: u32, flags: u32) -> Option<u32> {
        self.local_re_alloc_detail(ptr, bytes, flags)
            .map(|result| result.ptr)
    }

    /// Metadata-only convenience for callers without guest-memory backing.
    /// Raw guest APIs must use `prepare_local_re_alloc` and commit only after
    /// their copy and zero-fill operations succeed.
    pub fn local_re_alloc_detail(
        &mut self,
        ptr: u32,
        bytes: u32,
        flags: u32,
    ) -> Option<Reallocation> {
        let prepared = self.prepare_local_re_alloc(ptr, bytes, flags)?;
        self.commit_reallocation(prepared)
    }

    pub fn prepare_local_re_alloc(
        &mut self,
        ptr: u32,
        bytes: u32,
        flags: u32,
    ) -> Option<PreparedReallocation> {
        if flags & !(LMEM_VALID_FLAGS | LMEM_MODIFY) != 0 {
            return None;
        }
        let heap_flags = if flags & LMEM_ZEROINIT != 0 {
            HEAP_ZERO_MEMORY
        } else {
            0
        };
        self.prepare_heap_re_alloc(self.process_heap, heap_flags, ptr, bytes)
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

    /// Metadata-only convenience; it does not map or zero guest bytes.
    pub fn heap_alloc(&mut self, heap: u32, flags: u32, bytes: u32) -> Option<u32> {
        let prepared = self.prepare_heap_alloc(heap, flags, bytes)?;
        self.commit_allocation(prepared)
    }

    pub fn prepare_heap_alloc(
        &mut self,
        heap: u32,
        flags: u32,
        bytes: u32,
    ) -> Option<PreparedAllocation> {
        if flags & !(HEAP_GENERATE_EXCEPTIONS | HEAP_NO_SERIALIZE | HEAP_ZERO_MEMORY) != 0
            || !self.is_live_heap(heap)
        {
            return None;
        }
        let actual_size = sanitize_size(bytes)?;
        let ptr = self.allocate_heap_ptr(actual_size)?;
        Some(PreparedAllocation {
            allocation: MemoryAllocation {
                ptr,
                heap,
                requested_size: bytes,
                actual_size,
                zeroed: flags & HEAP_ZERO_MEMORY != 0,
            },
        })
    }

    pub fn commit_allocation(&mut self, prepared: PreparedAllocation) -> Option<u32> {
        let allocation = prepared.allocation;
        if !self.is_live_heap(allocation.heap) || self.allocations.contains_key(&allocation.ptr) {
            return None;
        }
        let ptr = allocation.ptr;
        self.allocations.insert(ptr, allocation);
        self.bump_heap_generation();
        Some(ptr)
    }

    /// Discard uncommitted metadata, retaining its possibly dirtied address hole.
    pub fn abort_allocation(&mut self, _prepared: PreparedAllocation) {}

    /// Metadata-only heap reallocation; it does not copy or zero guest bytes.
    pub fn heap_re_alloc(&mut self, heap: u32, flags: u32, ptr: u32, bytes: u32) -> Option<u32> {
        self.heap_re_alloc_detail(heap, flags, ptr, bytes)
            .map(|result| result.ptr)
    }

    /// Metadata-only convenience for callers without guest-memory backing.
    /// Raw guest APIs must use `prepare_heap_re_alloc` and commit only after
    /// their copy and zero-fill operations succeed.
    pub fn heap_re_alloc_detail(
        &mut self,
        heap: u32,
        flags: u32,
        ptr: u32,
        bytes: u32,
    ) -> Option<Reallocation> {
        let prepared = self.prepare_heap_re_alloc(heap, flags, ptr, bytes)?;
        self.commit_reallocation(prepared)
    }

    pub fn prepare_heap_re_alloc(
        &mut self,
        heap: u32,
        flags: u32,
        ptr: u32,
        bytes: u32,
    ) -> Option<PreparedReallocation> {
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
            return Some(PreparedReallocation {
                result: Reallocation {
                    ptr,
                    old_ptr: ptr,
                    old_actual_size,
                    new_actual_size: actual_size,
                    moved: false,
                },
                original_allocation: allocation,
                requested_size: bytes,
                zero_requested: flags & HEAP_ZERO_MEMORY != 0,
            });
        }
        if flags & HEAP_REALLOC_IN_PLACE_ONLY != 0 {
            return None;
        }
        let new_ptr = self.allocate_heap_ptr(actual_size)?;
        Some(PreparedReallocation {
            result: Reallocation {
                ptr: new_ptr,
                old_ptr: ptr,
                old_actual_size,
                new_actual_size: actual_size,
                moved: true,
            },
            original_allocation: allocation,
            requested_size: bytes,
            zero_requested: flags & HEAP_ZERO_MEMORY != 0,
        })
    }

    pub fn commit_reallocation(&mut self, prepared: PreparedReallocation) -> Option<Reallocation> {
        let old_ptr = prepared.result.old_ptr;
        let destination_conflict =
            prepared.result.moved && self.allocations.contains_key(&prepared.result.ptr);
        if self.allocations.get(&old_ptr) != Some(&prepared.original_allocation)
            || destination_conflict
        {
            return None;
        }

        let result = prepared.result.clone();
        if result.moved {
            self.allocations.remove(&old_ptr);
            self.allocations.insert(
                result.ptr,
                MemoryAllocation {
                    ptr: result.ptr,
                    heap: prepared.original_allocation.heap,
                    requested_size: prepared.requested_size,
                    actual_size: result.new_actual_size,
                    zeroed: prepared.original_allocation.zeroed || prepared.zero_requested,
                },
            );
        } else {
            let allocation = self.allocations.get_mut(&old_ptr)?;
            allocation.requested_size = prepared.requested_size;
            allocation.actual_size = result.new_actual_size;
            allocation.zeroed |= prepared.zero_requested;
        }
        self.bump_heap_generation();
        Some(result)
    }

    /// Discard uncommitted metadata without reusing a possibly written address.
    pub fn abort_reallocation(&mut self, _prepared: PreparedReallocation) {}

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
        owner: ProcessBirthKey,
        source: VirtualAllocationSource,
        address: u32,
        size: u32,
        allocation_type: u32,
        protect: u32,
    ) -> Option<u32> {
        if size == 0 || allocation_type & (MEM_COMMIT | MEM_RESERVE) == 0 {
            return None;
        }
        let incarnation = self.next_virtual_incarnation;
        let next_incarnation = incarnation.checked_add(1)?;
        let (base, size, next_cursor) = if address == 0 {
            let size = align_up(size, VIRTUAL_ALIGN)?;
            let base = self
                .next_virtual_ptr_by_owner
                .get(&owner)
                .copied()
                .unwrap_or(VIRTUAL_PTR_BASE)
                .max(self.next_virtual_ptr);
            let next_cursor = base.checked_add(size)?;
            (base, size, Some(next_cursor))
        } else {
            let base = align_down(address, VIRTUAL_ALIGN);
            let end = address.checked_add(size)?;
            let aligned_end = align_up(end, VIRTUAL_ALIGN)?;
            let size = aligned_end.checked_sub(base)?;
            (base, size, None)
        };
        // Native virtual-page ownership is still global/VA-only. Do not
        // publish a second same-VA region merely because its metadata key is
        // distinct: the native mapper would otherwise silently alias it.
        if self.virtual_overlaps(base, size) {
            return None;
        }
        if let Some(next_cursor) = next_cursor {
            self.next_virtual_ptr_by_owner.insert(owner, next_cursor);
            self.next_virtual_ptr = next_cursor;
        }
        self.next_virtual_incarnation = next_incarnation;
        self.virtual_allocations.insert(
            (owner, base),
            VirtualAllocation {
                base,
                size,
                allocation_type,
                protect,
                initial_bytes: Vec::new(),
                owner,
                incarnation,
                source,
            },
        );
        self.bump_virtual_generation();
        Some(base)
    }

    pub fn set_virtual_initial_bytes(
        &mut self,
        owner: ProcessBirthKey,
        base: u32,
        bytes: Vec<u8>,
    ) -> bool {
        let Some(allocation) = self.virtual_allocations.get_mut(&(owner, base)) else {
            return false;
        };
        allocation.initial_bytes = bytes;
        self.bump_virtual_generation();
        true
    }

    pub fn virtual_free(
        &mut self,
        owner: ProcessBirthKey,
        address: u32,
        size: u32,
        free_type: u32,
    ) -> bool {
        match free_type {
            MEM_RELEASE => {
                if size != 0 || self.virtual_allocations.remove(&(owner, address)).is_none() {
                    return false;
                }
                self.bump_virtual_generation();
                true
            }
            MEM_DECOMMIT => self
                .virtual_allocations
                .get(&(owner, address))
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

    pub fn heaps(&self) -> impl Iterator<Item = &Heap> {
        self.heaps.values().filter(|heap| !heap.destroyed)
    }

    pub fn virtual_allocation(
        &self,
        owner: ProcessBirthKey,
        base: u32,
    ) -> Option<&VirtualAllocation> {
        self.virtual_allocations.get(&(owner, base))
    }

    /// VA-only lookup for legacy native mapping/view synchronization. The
    /// global overlap fence above guarantees this is unambiguous for now.
    /// Remove this path when native mappings become owner-qualified.
    pub fn virtual_allocation_by_base_legacy(&self, base: u32) -> Option<&VirtualAllocation> {
        self.virtual_allocations
            .values()
            .find(|allocation| allocation.base == base)
    }

    pub fn virtual_allocations(&self) -> impl Iterator<Item = &VirtualAllocation> {
        self.virtual_allocations.values()
    }

    pub fn virtual_allocations_for(
        &self,
        owner: ProcessBirthKey,
    ) -> impl Iterator<Item = &VirtualAllocation> {
        self.virtual_allocations
            .range((owner, 0)..=(owner, u32::MAX))
            .map(|(_, allocation)| allocation)
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

    pub fn contains_allocated_range(&self, owner: ProcessBirthKey, ptr: u32, size: u32) -> bool {
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
        }) || self.virtual_allocations_for(owner).any(|allocation| {
            let Some(allocation_end) = allocation.base.checked_add(allocation.size) else {
                return false;
            };
            ptr >= allocation.base && end <= allocation_end
        })
    }

    pub fn intersects_foreign_virtual_allocation(
        &self,
        owner: ProcessBirthKey,
        ptr: u32,
        size: u32,
    ) -> bool {
        let Some(end) = ptr.checked_add(size) else {
            return true;
        };
        self.virtual_allocations.values().any(|allocation| {
            allocation.owner != owner
                && matches!(
                    allocation.source,
                    VirtualAllocationSource::Private
                        | VirtualAllocationSource::PrivateExCompatibility
                )
                && allocation
                    .base
                    .checked_add(allocation.size)
                    .is_some_and(|other_end| ptr < other_end && end > allocation.base)
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
    use crate::ce::object::HandleTable;

    fn boot_owner() -> ProcessBirthKey {
        let mut handles = HandleTable::default();
        handles.register_boot_process(1);
        handles.process_birth_key(1).expect("boot birth")
    }

    #[test]
    fn prepared_allocation_is_not_live_until_commit_and_abort_keeps_hole() {
        let mut memory = MemorySystem::default();
        let generation = memory.heap_generation();
        let prepared = memory.prepare_local_alloc(LMEM_ZEROINIT, 0).unwrap();
        let ptr = prepared.allocation().ptr;
        assert_eq!(prepared.allocation().actual_size, 1);
        assert_eq!(prepared.allocation().requested_size, 0);
        assert!(prepared.allocation().zeroed);
        assert!(memory.allocation(ptr).is_none());
        assert_eq!(memory.heap_generation(), generation);
        memory.abort_allocation(prepared);
        assert_eq!(memory.heap_high_water_mark(), ptr + HEAP_ALIGN);
        let prepared = memory
            .prepare_heap_alloc(PROCESS_HEAP_HANDLE, 0, 9)
            .unwrap();
        let next = prepared.allocation().ptr;
        assert_eq!(next, ptr + HEAP_ALIGN);
        assert_eq!(memory.commit_allocation(prepared), Some(next));
        assert_eq!(memory.heap_size(PROCESS_HEAP_HANDLE, 0, next), Some(9));
        assert_eq!(memory.heap_generation(), generation + 1);
    }

    #[test]
    fn prepared_allocation_commit_rejects_destroyed_heap_without_reuse() {
        let mut memory = MemorySystem::default();
        let heap = memory.heap_create(0, 0, 0).unwrap();
        let prepared = memory
            .prepare_heap_alloc(heap, HEAP_ZERO_MEMORY, 32)
            .unwrap();
        let ptr = prepared.allocation().ptr;
        assert!(memory.heap_destroy(heap));
        assert!(memory.commit_allocation(prepared).is_none());
        assert!(memory.allocation(ptr).is_none());
        assert_eq!(memory.heap_alloc(PROCESS_HEAP_HANDLE, 0, 1), Some(ptr + 32));
    }

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
    fn aborted_moved_reallocation_retains_its_reserved_address_hole() {
        let mut memory = MemorySystem::default();
        let source = memory.heap_alloc(PROCESS_HEAP_HANDLE, 0, 5).unwrap();
        let original = memory.allocation(source).unwrap().clone();
        let prepared = memory
            .prepare_heap_re_alloc(PROCESS_HEAP_HANDLE, HEAP_ZERO_MEMORY, source, 32)
            .unwrap();
        let destination = prepared.result().ptr;
        let reserved_end = memory.heap_high_water_mark();

        memory.abort_reallocation(prepared);

        assert_eq!(memory.allocation(source), Some(&original));
        assert!(memory.allocation(destination).is_none());
        assert_eq!(memory.heap_high_water_mark(), reserved_end);
        assert_eq!(
            memory.heap_alloc(PROCESS_HEAP_HANDLE, HEAP_ZERO_MEMORY, 32),
            Some(reserved_end)
        );
    }

    #[test]
    fn conflicting_moved_reallocation_does_not_reuse_reserved_destination() {
        let mut memory = MemorySystem::default();
        let source = memory.heap_alloc(PROCESS_HEAP_HANDLE, 0, 5).unwrap();
        let prepared = memory
            .prepare_heap_re_alloc(PROCESS_HEAP_HANDLE, 0, source, 32)
            .unwrap();
        let destination = prepared.result().ptr;
        let reserved_end = memory.heap_high_water_mark();
        assert!(memory.heap_free(PROCESS_HEAP_HANDLE, 0, source));

        assert!(memory.commit_reallocation(prepared).is_none());

        assert!(memory.allocation(source).is_none());
        assert!(memory.allocation(destination).is_none());
        assert_eq!(memory.heap_high_water_mark(), reserved_end);
        assert_eq!(
            memory.heap_alloc(PROCESS_HEAP_HANDLE, 0, 32),
            Some(reserved_end)
        );
    }

    #[test]
    fn contains_allocated_range_checks_heap_and_virtual_bounds() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let heap_ptr = memory.heap_alloc(PROCESS_HEAP_HANDLE, 0, 16).unwrap();
        let virtual_ptr = memory
            .virtual_alloc(
                owner,
                VirtualAllocationSource::Private,
                0,
                0x1234,
                MEM_COMMIT | MEM_RESERVE,
                4,
            )
            .unwrap();

        assert!(memory.contains_allocated_range(owner, heap_ptr, 16));
        assert!(memory.contains_allocated_range(owner, heap_ptr + 4, 8));
        assert!(!memory.contains_allocated_range(owner, heap_ptr + 8, 16));
        assert!(memory.contains_allocated_range(owner, virtual_ptr, 0x1234));
        assert!(!memory.contains_allocated_range(owner, virtual_ptr.wrapping_sub(1), 2));
    }

    #[test]
    fn virtual_alloc_rejects_zero_size_and_missing_type_flags() {
        let mut memory = MemorySystem::default();

        // size=0 → None regardless of type flags
        assert!(
            memory
                .virtual_alloc(
                    boot_owner(),
                    VirtualAllocationSource::Private,
                    0,
                    0,
                    MEM_COMMIT | MEM_RESERVE,
                    4,
                )
                .is_none()
        );

        // allocation_type=0 → None regardless of size
        assert!(
            memory
                .virtual_alloc(
                    boot_owner(),
                    VirtualAllocationSource::Private,
                    0,
                    0x1000,
                    0,
                    4
                )
                .is_none()
        );
    }

    #[test]
    fn virtual_alloc_with_unaligned_address_covers_requested_end() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let base = memory
            .virtual_alloc(
                owner,
                VirtualAllocationSource::Private,
                0x500a_9d46,
                0x1_0000,
                MEM_COMMIT | MEM_RESERVE,
                4,
            )
            .unwrap();

        assert_eq!(base, 0x500a_0000);
        let allocation = memory.virtual_allocation(owner, base).unwrap();
        assert_eq!(allocation.size, 0x2_0000);
        assert!(memory.contains_allocated_range(owner, 0x500b_9d48, 2));
    }

    #[test]
    fn virtual_free_release_requires_size_zero_and_valid_base() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let base = memory
            .virtual_alloc(
                owner,
                VirtualAllocationSource::Private,
                0,
                0x1000,
                MEM_COMMIT | MEM_RESERVE,
                4,
            )
            .unwrap();

        // MEM_RELEASE with size!=0 → false
        assert!(!memory.virtual_free(owner, base, 1, MEM_RELEASE));

        // MEM_RELEASE with correct size=0 on valid base → true
        assert!(memory.virtual_free(owner, base, 0, MEM_RELEASE));

        // repeated release on now-freed base → false
        assert!(!memory.virtual_free(owner, base, 0, MEM_RELEASE));
    }

    #[test]
    fn virtual_free_decommit_succeeds_for_valid_range_and_unknown_type_fails() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        // Allocate exactly one VIRTUAL_ALIGN-sized region (0x1_0000 bytes).
        let base = memory
            .virtual_alloc(
                owner,
                VirtualAllocationSource::Private,
                0,
                VIRTUAL_ALIGN,
                MEM_COMMIT | MEM_RESERVE,
                4,
            )
            .unwrap();

        // MEM_DECOMMIT with size within allocation → true
        assert!(memory.virtual_free(owner, base, VIRTUAL_ALIGN / 2, MEM_DECOMMIT));

        // MEM_DECOMMIT with size exceeding aligned allocation size → false
        assert!(!memory.virtual_free(owner, base, VIRTUAL_ALIGN + 1, MEM_DECOMMIT));

        // unknown free_type → false
        assert!(!memory.virtual_free(owner, base, 0, 0xdead));
    }

    #[test]
    fn virtual_allocation_metadata_keeps_birth_source_and_nonreused_incarnation() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let base = memory
            .virtual_alloc(
                owner,
                VirtualAllocationSource::Private,
                0x5200_0000,
                0x1000,
                MEM_COMMIT | MEM_RESERVE,
                4,
            )
            .unwrap();
        let original = memory.virtual_allocation(owner, base).unwrap();
        assert_eq!(original.owner, owner);
        assert_eq!(original.source, VirtualAllocationSource::Private);
        let original_incarnation = original.incarnation;
        assert!(memory.virtual_free(owner, base, 0, MEM_RELEASE));
        assert_eq!(
            memory.virtual_alloc(
                owner,
                VirtualAllocationSource::FileViewLegacy,
                base,
                0x1000,
                MEM_COMMIT | MEM_RESERVE,
                4,
            ),
            Some(base)
        );
        let replacement = memory.virtual_allocation(owner, base).unwrap();
        assert_eq!(replacement.source, VirtualAllocationSource::FileViewLegacy);
        assert!(replacement.incarnation > original_incarnation);
    }

    #[test]
    fn virtual_metadata_is_birth_qualified_but_native_same_va_remains_fenced() {
        let mut handles = HandleTable::default();
        handles.register_boot_process(1);
        let parent = handles.process_birth_key(1).unwrap();
        let _child_handle = handles.create_process(2);
        let child = handles.process_birth_key(2).unwrap();
        let mut memory = MemorySystem::default();
        let base = memory
            .virtual_alloc(
                parent,
                VirtualAllocationSource::Private,
                0x5200_0000,
                0x1000,
                MEM_COMMIT | MEM_RESERVE,
                4,
            )
            .unwrap();

        assert!(memory.virtual_allocation(parent, base).is_some());
        assert!(memory.virtual_allocation(child, base).is_none());
        assert_eq!(memory.virtual_allocations_for(parent).count(), 1);
        assert_eq!(memory.virtual_allocations_for(child).count(), 0);
        assert!(memory.contains_allocated_range(parent, base, 4));
        assert!(!memory.contains_allocated_range(child, base, 4));
        assert!(memory.intersects_foreign_virtual_allocation(child, base, 4));
        assert!(!memory.set_virtual_initial_bytes(child, base, vec![1]));
        assert!(!memory.virtual_free(child, base, 0, MEM_RELEASE));
        assert!(memory.virtual_allocation(parent, base).is_some());
        assert_eq!(
            memory.virtual_alloc(
                child,
                VirtualAllocationSource::Private,
                base,
                0x1000,
                MEM_COMMIT | MEM_RESERVE,
                4,
            ),
            None,
            "global native mapping still forbids simultaneous same-VA private regions"
        );
        let child_base = memory
            .virtual_alloc(
                child,
                VirtualAllocationSource::Private,
                0,
                0x1000,
                MEM_COMMIT | MEM_RESERVE,
                4,
            )
            .unwrap();
        assert_ne!(child_base, base);
        assert!(memory.virtual_allocation(child, child_base).is_some());
    }

    #[test]
    fn exhausted_virtual_incarnation_fails_without_publishing_allocation() {
        let mut memory = MemorySystem::default();
        memory.next_virtual_incarnation = u64::MAX;
        assert!(
            memory
                .virtual_alloc(
                    boot_owner(),
                    VirtualAllocationSource::Private,
                    0x5200_0000,
                    0x1000,
                    MEM_COMMIT | MEM_RESERVE,
                    4,
                )
                .is_none()
        );
        assert!(
            memory
                .virtual_allocation(boot_owner(), 0x5200_0000)
                .is_none()
        );
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
