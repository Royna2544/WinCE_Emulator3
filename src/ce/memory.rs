use std::{
    cell::{Cell, UnsafeCell},
    collections::BTreeMap,
    ffi::c_void,
    fmt,
    rc::Rc,
};

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

#[derive(Debug)]
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
    // A prepared private reservation is not guest-visible. Keep its VA out of
    // reuse even if its token is dropped after a native mapping side effect.
    pending_private_holes: BTreeMap<(ProcessBirthKey, u32), u32>,
    stable_private_pages: BTreeMap<PrivatePageIdentity, Rc<StablePrivatePage>>,
    // A native pin can outlive a released VM record. Keep the checkpoint
    // unsupported flag set for this MemorySystem lineage even after release.
    stable_private_slabs_seen: bool,
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

/// Backing is explicit: a Private source does not opt public VirtualAlloc into
/// pointer-mapped slabs. LegacySeed remains the production behavior for now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualAllocationBacking {
    LegacySeed,
    StablePrivateSlab,
}

pub const STABLE_PRIVATE_PAGE_SIZE: usize = 0x1000;

/// Exact page identity within a MemorySystem lineage. A native owner must
/// additionally validate the backing pin itself, not just this copied key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrivatePageIdentity {
    owner: ProcessBirthKey,
    allocation_base: u32,
    allocation_incarnation: u64,
    page_base: u32,
}

impl PrivatePageIdentity {
    pub fn owner(self) -> ProcessBirthKey {
        self.owner
    }

    pub fn allocation_base(self) -> u32 {
        self.allocation_base
    }

    pub fn allocation_incarnation(self) -> u64 {
        self.allocation_incarnation
    }

    pub fn page_base(self) -> u32 {
        self.page_base
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivatePageAccessError {
    OutOfRange,
    NativeExecutionActive,
}

/// The only owner of private page bytes. UnsafeCell permits the synchronous
/// native engine to mutate the stable address; host access is copied through
/// bounded methods and forbidden while a native execution guard is held.
// The native pointer refers to `bytes` at offset zero. Rc allocates this
// object at its declared alignment, so Unicorn receives a page-aligned slab.
#[repr(C, align(4096))]
struct StablePrivatePage {
    bytes: UnsafeCell<[u8; STABLE_PRIVATE_PAGE_SIZE]>,
    native_execution_active: Cell<bool>,
}

impl fmt::Debug for StablePrivatePage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StablePrivatePage")
            .field(
                "native_execution_active",
                &self.native_execution_active.get(),
            )
            .finish_non_exhaustive()
    }
}

impl StablePrivatePage {
    fn zeroed() -> Self {
        Self {
            bytes: UnsafeCell::new([0; STABLE_PRIVATE_PAGE_SIZE]),
            native_execution_active: Cell::new(false),
        }
    }

    fn deep_copy(&self) -> Self {
        // MemorySystem::Clone cannot return Result. Refuse to copy while the
        // native engine might be writing; a supported checkpoint operation
        // remains separately gated until machine-wide relineage exists.
        assert!(
            !self.native_execution_active.get(),
            "cannot clone stable private bytes during native execution"
        );
        let mut bytes = [0; STABLE_PRIVATE_PAGE_SIZE];
        // SAFETY: the guard above excludes native execution, all host byte
        // methods finish before returning, and Rc keeps the page alive.
        unsafe {
            std::ptr::copy_nonoverlapping(
                (*self.bytes.get()).as_ptr(),
                bytes.as_mut_ptr(),
                STABLE_PRIVATE_PAGE_SIZE,
            );
        }
        Self {
            bytes: UnsafeCell::new(bytes),
            native_execution_active: Cell::new(false),
        }
    }
}

/// Opaque strong pin. It is !Send/!Sync through Rc and keeps the exact host
/// page alive until native unmap or owner close, even after VM metadata release.
#[derive(Debug, Clone)]
pub struct StablePrivatePagePin {
    identity: PrivatePageIdentity,
    page: Rc<StablePrivatePage>,
}

impl StablePrivatePagePin {
    pub fn identity(&self) -> PrivatePageIdentity {
        self.identity
    }

    /// Obtaining a raw pointer is safe; passing it to native code is not.
    /// The caller must retain this pin through checked unmap/owner close and
    /// fence host access while native execution may write through the pointer.
    pub fn native_ptr(&self) -> *mut c_void {
        self.page.bytes.get().cast::<c_void>()
    }

    pub fn read_bytes(
        &self,
        offset: usize,
        out: &mut [u8],
    ) -> std::result::Result<(), PrivatePageAccessError> {
        self.check_access(offset, out.len())?;
        // SAFETY: no native execution guard is held, the page is pinned, and
        // this bounded copy creates no reference that can outlive the call.
        unsafe {
            std::ptr::copy_nonoverlapping(
                (*self.page.bytes.get()).as_ptr().add(offset),
                out.as_mut_ptr(),
                out.len(),
            );
        }
        Ok(())
    }

    pub fn write_bytes(
        &self,
        offset: usize,
        bytes: &[u8],
    ) -> std::result::Result<(), PrivatePageAccessError> {
        self.check_access(offset, bytes.len())?;
        // SAFETY: same single-threaded access fence as read_bytes. The only
        // mutable access is this bounded copy or the native pointer lease.
        unsafe {
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                (*self.page.bytes.get()).as_mut_ptr().add(offset),
                bytes.len(),
            );
        }
        Ok(())
    }

    pub fn begin_native_execution(
        &self,
    ) -> std::result::Result<StablePrivatePageExecutionGuard, PrivatePageAccessError> {
        if self.page.native_execution_active.replace(true) {
            return Err(PrivatePageAccessError::NativeExecutionActive);
        }
        Ok(StablePrivatePageExecutionGuard {
            page: Rc::clone(&self.page),
        })
    }

    fn check_access(
        &self,
        offset: usize,
        len: usize,
    ) -> std::result::Result<(), PrivatePageAccessError> {
        if self.page.native_execution_active.get() {
            return Err(PrivatePageAccessError::NativeExecutionActive);
        }
        if offset
            .checked_add(len)
            .is_none_or(|end| end > STABLE_PRIVATE_PAGE_SIZE)
        {
            return Err(PrivatePageAccessError::OutOfRange);
        }
        Ok(())
    }
}

/// Hold one guard for every mapped private page for the whole emu_start call.
/// Fault-time pointer mappings must add their guard before resuming the guest.
pub struct StablePrivatePageExecutionGuard {
    page: Rc<StablePrivatePage>,
}

impl Drop for StablePrivatePageExecutionGuard {
    fn drop(&mut self) {
        self.page.native_execution_active.set(false);
    }
}

impl VirtualAllocationSource {
    fn is_private(self) -> bool {
        matches!(self, Self::Private | Self::PrivateExCompatibility)
    }
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
    pub backing: VirtualAllocationBacking,
    /// Exact committed 4-KiB private pages and their individual protection.
    /// Empty for a reserve-only slab or any unmigrated LegacySeed allocation.
    pub private_committed_pages: BTreeMap<u32, u32>,
}

/// A detached private VM proposal. A native owner may map `new_page_pins`
/// before `MemorySystem` publishes this reservation or commit. Dropping the
/// token leaves a new reservation's monotonic VA hole unavailable for reuse.
#[derive(Debug)]
#[must_use = "prepared private VM must be committed or abandoned"]
pub struct PreparedPrivateVm {
    owner: ProcessBirthKey,
    result_ptr: u32,
    proposed_allocation: VirtualAllocation,
    original_allocation: Option<VirtualAllocation>,
    expected_virtual_generation: u64,
    new_page_pins: Vec<StablePrivatePagePin>,
}

impl PreparedPrivateVm {
    pub fn owner(&self) -> ProcessBirthKey {
        self.owner
    }

    pub fn base(&self) -> u32 {
        self.proposed_allocation.base
    }

    pub fn result_ptr(&self) -> u32 {
        self.result_ptr
    }

    pub fn allocation(&self) -> &VirtualAllocation {
        &self.proposed_allocation
    }

    pub fn new_page_pins(&self) -> &[StablePrivatePagePin] {
        &self.new_page_pins
    }

    pub fn protection_for_page(&self, page: u32) -> Option<u32> {
        self.proposed_allocation
            .private_committed_pages
            .get(&page)
            .copied()
    }

    pub fn creates_reservation(&self) -> bool {
        self.original_allocation.is_none()
    }
}

impl Clone for MemorySystem {
    fn clone(&self) -> Self {
        Self {
            process_heap: self.process_heap,
            next_heap: self.next_heap,
            next_heap_ptr: self.next_heap_ptr,
            next_virtual_ptr: self.next_virtual_ptr,
            next_virtual_ptr_by_owner: self.next_virtual_ptr_by_owner.clone(),
            heap_generation: self.heap_generation,
            virtual_generation: self.virtual_generation,
            next_virtual_incarnation: self.next_virtual_incarnation,
            heaps: self.heaps.clone(),
            allocations: self.allocations.clone(),
            virtual_allocations: self.virtual_allocations.clone(),
            pending_private_holes: self.pending_private_holes.clone(),
            stable_private_pages: self
                .stable_private_pages
                .iter()
                .map(|(identity, page)| (*identity, Rc::new(page.deep_copy())))
                .collect(),
            stable_private_slabs_seen: self.stable_private_slabs_seen,
        }
    }
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
            pending_private_holes: BTreeMap::new(),
            stable_private_pages: BTreeMap::new(),
            stable_private_slabs_seen: false,
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

    /// Prepare one private reservation or a commit within an existing private
    /// reservation. No allocation or page becomes visible until the caller has
    /// checked any required native mapping and calls `commit_prepared_private_vm`.
    /// A failed new reservation consumes an owner-qualified VA hole and an
    /// allocation incarnation, because native FFI may have partially mapped it.
    pub fn prepare_private_vm(
        &mut self,
        owner: ProcessBirthKey,
        source: VirtualAllocationSource,
        address: u32,
        size: u32,
        allocation_type: u32,
        protect: u32,
    ) -> Option<PreparedPrivateVm> {
        if !source.is_private()
            || size == 0
            || allocation_type & !(MEM_COMMIT | MEM_RESERVE) != 0
            || !matches!(
                protect & 0xff,
                0x02 | 0x04 | 0x08 | 0x10 | 0x20 | 0x40 | 0x80
            )
            || protect & !0xff != 0
        {
            return None;
        }

        if allocation_type == MEM_COMMIT {
            if address == 0 || address & (STABLE_PRIVATE_PAGE_SIZE as u32 - 1) != 0 {
                return None;
            }
            let end = address.checked_add(size)?;
            let page_end = align_up(end, STABLE_PRIVATE_PAGE_SIZE as u32)?;
            let original = self
                .virtual_allocations_for(owner)
                .find(|allocation| {
                    allocation.backing == VirtualAllocationBacking::StablePrivateSlab
                        && allocation.source == source
                        && address >= allocation.base
                        && allocation
                            .base
                            .checked_add(allocation.size)
                            .is_some_and(|reservation_end| page_end <= reservation_end)
                })?
                .clone();
            let mut proposed = original.clone();
            let mut new_page_pins = Vec::new();
            let mut page = address;
            while page < page_end {
                if let Some(existing_protect) = proposed.private_committed_pages.get(&page) {
                    // A second commit does not silently change protection.
                    if *existing_protect != protect {
                        return None;
                    }
                } else {
                    proposed.private_committed_pages.insert(page, protect);
                    new_page_pins.push(StablePrivatePagePin {
                        identity: PrivatePageIdentity {
                            owner,
                            allocation_base: proposed.base,
                            allocation_incarnation: proposed.incarnation,
                            page_base: page,
                        },
                        page: Rc::new(StablePrivatePage::zeroed()),
                    });
                }
                page = page.checked_add(STABLE_PRIVATE_PAGE_SIZE as u32)?;
            }
            proposed.allocation_type |= MEM_COMMIT;
            return Some(PreparedPrivateVm {
                owner,
                result_ptr: address,
                proposed_allocation: proposed,
                original_allocation: Some(original),
                expected_virtual_generation: self.virtual_generation,
                new_page_pins,
            });
        }

        if allocation_type != MEM_RESERVE && allocation_type != (MEM_RESERVE | MEM_COMMIT) {
            return None;
        }
        let incarnation = self.next_virtual_incarnation;
        let next_incarnation = incarnation.checked_add(1)?;
        let reservation_size = if address == 0 {
            align_up(size, VIRTUAL_ALIGN)?
        } else {
            let end = address.checked_add(size)?;
            align_up(end, VIRTUAL_ALIGN)?.checked_sub(align_down(address, VIRTUAL_ALIGN))?
        };
        let base = if address == 0 {
            let mut candidate = self
                .next_virtual_ptr_by_owner
                .get(&owner)
                .copied()
                .unwrap_or(VIRTUAL_PTR_BASE)
                .max(VIRTUAL_PTR_BASE);
            while self.private_vm_overlaps(owner, candidate, reservation_size) {
                candidate = candidate.checked_add(VIRTUAL_ALIGN)?;
            }
            candidate
        } else {
            let candidate = align_down(address, VIRTUAL_ALIGN);
            if self.private_vm_overlaps(owner, candidate, reservation_size) {
                return None;
            }
            candidate
        };
        let reservation_end = base.checked_add(reservation_size)?;
        let mut committed_pages = BTreeMap::new();
        let mut new_page_pins = Vec::new();
        if allocation_type & MEM_COMMIT != 0 {
            let commit_start = if address == 0 {
                base
            } else {
                align_down(address, STABLE_PRIVATE_PAGE_SIZE as u32)
            };
            let commit_end = align_up(
                (if address == 0 { base } else { address }).checked_add(size)?,
                STABLE_PRIVATE_PAGE_SIZE as u32,
            )?;
            if commit_end > reservation_end {
                return None;
            }
            let mut page = commit_start;
            while page < commit_end {
                committed_pages.insert(page, protect);
                new_page_pins.push(StablePrivatePagePin {
                    identity: PrivatePageIdentity {
                        owner,
                        allocation_base: base,
                        allocation_incarnation: incarnation,
                        page_base: page,
                    },
                    page: Rc::new(StablePrivatePage::zeroed()),
                });
                page = page.checked_add(STABLE_PRIVATE_PAGE_SIZE as u32)?;
            }
        }
        let proposed_allocation = VirtualAllocation {
            base,
            size: reservation_size,
            allocation_type,
            protect,
            initial_bytes: Vec::new(),
            owner,
            incarnation,
            source,
            backing: VirtualAllocationBacking::StablePrivateSlab,
            private_committed_pages: committed_pages,
        };
        // The hole is installed only after every fallible preparation step.
        // Clone copies the hole but never the detached token or its page pins.
        self.pending_private_holes
            .insert((owner, base), reservation_size);
        self.next_virtual_incarnation = next_incarnation;
        if address == 0 {
            self.next_virtual_ptr_by_owner
                .insert(owner, reservation_end);
        }
        Some(PreparedPrivateVm {
            owner,
            result_ptr: base,
            proposed_allocation,
            original_allocation: None,
            expected_virtual_generation: self.virtual_generation,
            new_page_pins,
        })
    }

    /// Publish a fully prepared private VM change. The native caller must
    /// already have checked every required map/protection operation; this
    /// method never calls the engine or accepts a stale metadata generation.
    pub fn commit_prepared_private_vm(&mut self, prepared: PreparedPrivateVm) -> Option<u32> {
        let PreparedPrivateVm {
            owner,
            result_ptr,
            proposed_allocation,
            original_allocation,
            expected_virtual_generation,
            new_page_pins,
        } = prepared;
        if self.virtual_generation != expected_virtual_generation
            || proposed_allocation.owner != owner
            || proposed_allocation.backing != VirtualAllocationBacking::StablePrivateSlab
            || !proposed_allocation.source.is_private()
        {
            return None;
        }
        let key = (owner, proposed_allocation.base);
        match original_allocation.as_ref() {
            Some(original) => {
                if self.virtual_allocations.get(&key) != Some(original)
                    || self.pending_private_holes.contains_key(&key)
                    || proposed_allocation.incarnation != original.incarnation
                    || proposed_allocation.size != original.size
                    || proposed_allocation.source != original.source
                {
                    return None;
                }
            }
            None => {
                if self.pending_private_holes.get(&key) != Some(&proposed_allocation.size)
                    || self.virtual_allocations.contains_key(&key)
                {
                    return None;
                }
            }
        }
        let expected_new_pages = proposed_allocation
            .private_committed_pages
            .keys()
            .filter(|page| {
                original_allocation
                    .as_ref()
                    .is_none_or(|original| !original.private_committed_pages.contains_key(page))
            })
            .copied()
            .collect::<Vec<_>>();
        if expected_new_pages.len() != new_page_pins.len() {
            return None;
        }
        for (page, pin) in expected_new_pages.iter().zip(&new_page_pins) {
            let identity = pin.identity;
            if identity.owner != owner
                || identity.allocation_base != proposed_allocation.base
                || identity.allocation_incarnation != proposed_allocation.incarnation
                || identity.page_base != *page
                || self.stable_private_pages.contains_key(&identity)
            {
                return None;
            }
        }
        for pin in new_page_pins {
            self.stable_private_pages.insert(pin.identity, pin.page);
        }
        self.virtual_allocations.insert(key, proposed_allocation);
        if original_allocation.is_none() {
            self.pending_private_holes.remove(&key);
        }
        self.stable_private_slabs_seen = true;
        self.bump_virtual_generation();
        Some(result_ptr)
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
        self.virtual_alloc_with_backing(
            owner,
            source,
            address,
            size,
            allocation_type,
            protect,
            VirtualAllocationBacking::LegacySeed,
        )
    }

    /// E-18's internal-only backing opt-in. The public VirtualAlloc path above
    /// remains LegacySeed until prepare/map/commit and owner-scoped native
    /// release are connected. This diagnostic accepts one <=64-KiB reservation
    /// beginning at an allocator-chosen base and commits only requested pages.
    #[cfg(test)]
    pub(crate) fn virtual_alloc_private_slab_for_test(
        &mut self,
        owner: ProcessBirthKey,
        address: u32,
        size: u32,
        allocation_type: u32,
        protect: u32,
    ) -> Option<u32> {
        self.virtual_alloc_with_backing(
            owner,
            VirtualAllocationSource::Private,
            address,
            size,
            allocation_type,
            protect,
            VirtualAllocationBacking::StablePrivateSlab,
        )
    }

    fn virtual_alloc_with_backing(
        &mut self,
        owner: ProcessBirthKey,
        source: VirtualAllocationSource,
        address: u32,
        size: u32,
        allocation_type: u32,
        protect: u32,
        backing: VirtualAllocationBacking,
    ) -> Option<u32> {
        if size == 0 || allocation_type & (MEM_COMMIT | MEM_RESERVE) == 0 {
            return None;
        }
        let requested_size = size;
        if backing == VirtualAllocationBacking::StablePrivateSlab
            && (source != VirtualAllocationSource::Private
                || address != 0
                || size > VIRTUAL_ALIGN
                || allocation_type != MEM_COMMIT | MEM_RESERVE)
        {
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
        // Reserve 64 KiB of guest VA, but allocate fixed host storage only
        // for the requested committed 4-KiB pages. Build all page records
        // before publishing the allocation or advancing a cursor.
        let mut new_private_pages = Vec::new();
        let mut committed_private_pages = BTreeMap::new();
        if backing == VirtualAllocationBacking::StablePrivateSlab {
            let committed_size = align_up(requested_size, STABLE_PRIVATE_PAGE_SIZE as u32)?;
            let end = base.checked_add(committed_size)?;
            let mut page_base = base;
            while page_base < end {
                committed_private_pages.insert(page_base, protect);
                new_private_pages.push((
                    PrivatePageIdentity {
                        owner,
                        allocation_base: base,
                        allocation_incarnation: incarnation,
                        page_base,
                    },
                    Rc::new(StablePrivatePage::zeroed()),
                ));
                page_base = page_base.checked_add(STABLE_PRIVATE_PAGE_SIZE as u32)?;
            }
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
                backing,
                private_committed_pages: committed_private_pages,
            },
        );
        if backing == VirtualAllocationBacking::StablePrivateSlab {
            self.stable_private_pages.extend(new_private_pages);
            self.stable_private_slabs_seen = true;
        }
        self.bump_virtual_generation();
        Some(base)
    }

    pub fn set_virtual_initial_bytes(
        &mut self,
        owner: ProcessBirthKey,
        base: u32,
        bytes: Vec<u8>,
    ) -> bool {
        let Some((backing, incarnation, reservation_size)) = self
            .virtual_allocations
            .get(&(owner, base))
            .map(|allocation| (allocation.backing, allocation.incarnation, allocation.size))
        else {
            return false;
        };
        if backing == VirtualAllocationBacking::LegacySeed {
            self.virtual_allocations
                .get_mut(&(owner, base))
                .expect("allocation was just found")
                .initial_bytes = bytes;
            self.bump_virtual_generation();
            return true;
        }
        let Some(end) = u32::try_from(bytes.len())
            .ok()
            .and_then(|len| base.checked_add(len))
        else {
            return false;
        };
        if end > base.saturating_add(reservation_size) {
            return false;
        }
        // A 64-KiB reservation may contain only one committed 4-KiB page.
        // Reject crossing the committed extent before the per-page preflight
        // below, which also detects any future holes in that extent.
        let committed_end = self
            .stable_private_pages
            .keys()
            .filter(|identity| {
                identity.owner == owner
                    && identity.allocation_base == base
                    && identity.allocation_incarnation == incarnation
            })
            .filter_map(|identity| {
                identity
                    .page_base
                    .checked_add(STABLE_PRIVATE_PAGE_SIZE as u32)
            })
            .max()
            .unwrap_or(base);
        if end > committed_end {
            return false;
        }
        let mut writes = Vec::new();
        let mut position = 0usize;
        while position < bytes.len() {
            let Some(address) = base.checked_add(position as u32) else {
                return false;
            };
            let page_base = address & !(STABLE_PRIVATE_PAGE_SIZE as u32 - 1);
            let offset = (address - page_base) as usize;
            let length = (STABLE_PRIVATE_PAGE_SIZE - offset).min(bytes.len() - position);
            let identity = PrivatePageIdentity {
                owner,
                allocation_base: base,
                allocation_incarnation: incarnation,
                page_base,
            };
            let Some(page) = self.stable_private_pages.get(&identity) else {
                return false;
            };
            // Initial seed is not an ongoing copy authority. Reject reseeding
            // a page with an outstanding native or diagnostic pin.
            if Rc::strong_count(page) != 1 || page.native_execution_active.get() {
                return false;
            }
            writes.push((
                StablePrivatePagePin {
                    identity,
                    page: Rc::clone(page),
                },
                offset,
                position,
                length,
            ));
            position += length;
        }
        for (page, offset, position, length) in writes {
            if page
                .write_bytes(offset, &bytes[position..position + length])
                .is_err()
            {
                return false;
            }
        }
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
        // Reads of legacy regions remain globally visible, but releasing or
        // decommitting any region requires its exact birth-key owner.
        let key = (owner, address);
        match free_type {
            MEM_RELEASE => {
                if size != 0 {
                    return false;
                }
                let Some(allocation) = self.virtual_allocations.remove(&key) else {
                    return false;
                };
                if allocation.backing == VirtualAllocationBacking::StablePrivateSlab {
                    self.stable_private_pages.retain(|identity, _| {
                        identity.owner != owner
                            || identity.allocation_incarnation != allocation.incarnation
                    });
                }
                self.bump_virtual_generation();
                true
            }
            MEM_DECOMMIT => self
                .virtual_allocations
                .get(&key)
                .is_some_and(|allocation| {
                    allocation.backing == VirtualAllocationBacking::LegacySeed
                        && size <= allocation.size
                }),
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

    /// Private pages are scoped to `owner`; unmigrated nonprivate producers
    /// retain their previous global visibility for compatibility.
    pub fn virtual_allocations_visible_to(
        &self,
        owner: ProcessBirthKey,
    ) -> impl Iterator<Item = &VirtualAllocation> {
        self.virtual_allocations
            .values()
            .filter(move |allocation| allocation.owner == owner || !allocation.source.is_private())
    }

    /// Conservatively reject independent machine checkpointing after any
    /// diagnostic slab existed, including after release while native pins may
    /// still exist outside this VM directory.
    pub fn has_stable_private_slabs(&self) -> bool {
        self.stable_private_slabs_seen
    }

    pub fn private_page_pin(
        &self,
        owner: ProcessBirthKey,
        page_base: u32,
    ) -> Option<StablePrivatePagePin> {
        if page_base & (STABLE_PRIVATE_PAGE_SIZE as u32 - 1) != 0 {
            return None;
        }
        let allocation = self.virtual_allocations_for(owner).find(|allocation| {
            allocation.backing == VirtualAllocationBacking::StablePrivateSlab
                && page_base >= allocation.base
                && allocation
                    .base
                    .checked_add(allocation.size)
                    .is_some_and(|end| page_base < end)
        })?;
        let identity = PrivatePageIdentity {
            owner,
            allocation_base: allocation.base,
            allocation_incarnation: allocation.incarnation,
            page_base,
        };
        let page = Rc::clone(self.stable_private_pages.get(&identity)?);
        Some(StablePrivatePagePin { identity, page })
    }

    /// Current committed slab pages for exactly one process birth. Reservation
    /// holes and legacy copy-backed allocations are never returned.
    pub fn private_page_pins_for(&self, owner: ProcessBirthKey) -> Vec<StablePrivatePagePin> {
        self.stable_private_pages
            .iter()
            .filter_map(|(identity, page)| {
                if identity.owner != owner {
                    return None;
                }
                let allocation = self.virtual_allocation(owner, identity.allocation_base)?;
                if allocation.backing != VirtualAllocationBacking::StablePrivateSlab
                    || allocation.incarnation != identity.allocation_incarnation
                {
                    return None;
                }
                Some(StablePrivatePagePin {
                    identity: *identity,
                    page: Rc::clone(page),
                })
            })
            .collect()
    }

    /// Revalidate the exact VM record and strong backing identity before a
    /// native owner treats a retained pin as current. Clone deep-copies pages,
    /// so a pin from the source MemorySystem is stale in its clone.
    pub fn contains_private_page_pin(&self, pin: &StablePrivatePagePin) -> bool {
        let identity = pin.identity;
        self.virtual_allocation(identity.owner, identity.allocation_base)
            .is_some_and(|allocation| {
                allocation.backing == VirtualAllocationBacking::StablePrivateSlab
                    && allocation.incarnation == identity.allocation_incarnation
            })
            && self
                .stable_private_pages
                .get(&identity)
                .is_some_and(|page| Rc::ptr_eq(page, &pin.page))
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
        }) || self
            .virtual_allocations_visible_to(owner)
            .any(|allocation| {
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
                && allocation.source.is_private()
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
        self.virtual_allocations
            .values()
            .any(|allocation| ranges_overlap(base, size, allocation.base, allocation.size))
            || self
                .pending_private_holes
                .iter()
                .any(|((_, other_base), other_size)| {
                    ranges_overlap(base, size, *other_base, *other_size)
                })
    }

    fn private_vm_overlaps(&self, owner: ProcessBirthKey, base: u32, size: u32) -> bool {
        // A legacy nonprivate producer may still seed/copy by guest VA alone,
        // so it remains a global collision until that family is migrated.
        self.virtual_allocations.values().any(|allocation| {
            (allocation.owner == owner || !allocation.source.is_private())
                && ranges_overlap(base, size, allocation.base, allocation.size)
        }) || self
            .pending_private_holes
            .iter()
            .any(|((other_owner, other_base), other_size)| {
                *other_owner == owner && ranges_overlap(base, size, *other_base, *other_size)
            })
            || self.allocations.values().any(|allocation| {
                ranges_overlap(base, size, allocation.ptr, allocation.actual_size)
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

fn ranges_overlap(base: u32, size: u32, other_base: u32, other_size: u32) -> bool {
    let start = u64::from(base);
    let end = start + u64::from(size);
    let other_start = u64::from(other_base);
    let other_end = other_start + u64::from(other_size);
    start < other_end && other_start < end
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

        let legacy_base = memory
            .virtual_alloc(
                parent,
                VirtualAllocationSource::FileViewLegacy,
                0x5600_0000,
                0x1000,
                MEM_COMMIT | MEM_RESERVE,
                4,
            )
            .unwrap();
        assert!(memory.contains_allocated_range(child, legacy_base, 4));
        assert!(!memory.virtual_free(child, legacy_base, 0, MEM_RELEASE));
        assert!(!memory.virtual_free(child, legacy_base, 4, MEM_DECOMMIT));
        assert!(memory.virtual_allocation(parent, legacy_base).is_some());
        assert!(memory.virtual_free(parent, legacy_base, 0, MEM_RELEASE));
        assert!(memory.virtual_allocation(parent, legacy_base).is_none());
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
    fn public_private_virtual_alloc_keeps_legacy_seed_backing() {
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
        assert_eq!(
            memory.virtual_allocation(owner, base).unwrap().backing,
            VirtualAllocationBacking::LegacySeed
        );
        assert!(!memory.has_stable_private_slabs());
        assert!(memory.private_page_pin(owner, base).is_none());
        assert!(memory.private_page_pins_for(owner).is_empty());
        assert!(memory.set_virtual_initial_bytes(owner, base, vec![1, 2, 3, 4]));
        assert_eq!(
            memory
                .virtual_allocation(owner, base)
                .unwrap()
                .initial_bytes,
            [1, 2, 3, 4]
        );
    }

    #[test]
    fn prepared_private_reserve_then_commit_publishes_only_exact_pages() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let generation = memory.virtual_generation();
        let reserve = memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::Private,
                0,
                0x3000,
                MEM_RESERVE,
                4,
            )
            .expect("reserve proposal");
        let base = reserve.base();
        assert_eq!(reserve.result_ptr(), base);
        assert!(reserve.creates_reservation());
        assert_eq!(reserve.allocation().size, VIRTUAL_ALIGN);
        assert!(reserve.new_page_pins().is_empty());
        assert!(reserve.allocation().private_committed_pages.is_empty());
        assert_eq!(memory.virtual_generation(), generation);
        assert!(memory.virtual_allocation(owner, base).is_none());
        assert!(!memory.has_stable_private_slabs());
        assert_eq!(memory.commit_prepared_private_vm(reserve), Some(base));
        assert!(memory.has_stable_private_slabs());
        assert!(memory.private_page_pin(owner, base).is_none());

        let commit = memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::Private,
                base + 0x1000,
                0x2000,
                MEM_COMMIT,
                4,
            )
            .expect("commit within reservation");
        assert!(!commit.creates_reservation());
        assert_eq!(commit.result_ptr(), base + 0x1000);
        assert_eq!(commit.new_page_pins().len(), 2);
        assert_eq!(commit.protection_for_page(base + 0x1000), Some(4));
        assert_eq!(commit.protection_for_page(base), None);
        assert!(memory.private_page_pin(owner, base + 0x1000).is_none());
        assert_eq!(
            memory.commit_prepared_private_vm(commit),
            Some(base + 0x1000)
        );
        let allocation = memory.virtual_allocation(owner, base).unwrap();
        assert_eq!(allocation.private_committed_pages.len(), 2);
        assert_eq!(
            allocation.private_committed_pages.get(&(base + 0x2000)),
            Some(&4)
        );
        assert!(memory.private_page_pin(owner, base).is_none());
        let pin = memory.private_page_pin(owner, base + 0x1000).unwrap();
        let mut bytes = [0xff; 4];
        pin.read_bytes(0, &mut bytes).unwrap();
        assert_eq!(bytes, [0; 4]);
    }

    #[test]
    fn prepared_private_abandonment_keeps_owner_hole_but_foreign_private_may_use_va() {
        let mut handles = HandleTable::default();
        handles.register_boot_process(1);
        let parent = handles.process_birth_key(1).unwrap();
        handles.create_process(2);
        let child = handles.process_birth_key(2).unwrap();
        let mut memory = MemorySystem::default();
        let base = 0x5200_0000;
        let abandoned = memory
            .prepare_private_vm(
                parent,
                VirtualAllocationSource::Private,
                base,
                0x1000,
                MEM_RESERVE | MEM_COMMIT,
                4,
            )
            .unwrap();
        assert_eq!(abandoned.base(), base);
        assert!(memory.virtual_allocation(parent, base).is_none());
        let clone = memory.clone();
        drop(abandoned);
        assert!(
            memory
                .prepare_private_vm(
                    parent,
                    VirtualAllocationSource::Private,
                    base,
                    0x1000,
                    MEM_RESERVE,
                    4
                )
                .is_none()
        );
        assert!(clone.pending_private_holes.contains_key(&(parent, base)));
        let child_proposal = memory
            .prepare_private_vm(
                child,
                VirtualAllocationSource::Private,
                base,
                0x1000,
                MEM_RESERVE | MEM_COMMIT,
                4,
            )
            .expect("a foreign private address space may use the same VA");
        assert_eq!(
            memory.commit_prepared_private_vm(child_proposal),
            Some(base)
        );
        assert!(memory.virtual_allocation(parent, base).is_none());
        assert!(memory.private_page_pin(child, base).is_some());
        assert!(memory.private_page_pin(parent, base).is_none());
        // Unmigrated legacy allocations still retain their global fence.
        assert!(
            memory
                .virtual_alloc(
                    parent,
                    VirtualAllocationSource::FileViewLegacy,
                    base,
                    0x1000,
                    MEM_RESERVE | MEM_COMMIT,
                    4,
                )
                .is_none()
        );
    }

    #[test]
    fn prepared_private_commit_rejects_stale_metadata_without_page_publication() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let reserve = memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::Private,
                0x5400_0000,
                0x1000,
                MEM_RESERVE,
                4,
            )
            .unwrap();
        let base = reserve.base();
        assert_eq!(memory.commit_prepared_private_vm(reserve), Some(base));
        let stale_commit = memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::Private,
                base,
                0x1000,
                MEM_COMMIT,
                4,
            )
            .unwrap();
        assert_eq!(stale_commit.new_page_pins().len(), 1);
        memory
            .virtual_alloc(
                owner,
                VirtualAllocationSource::Private,
                0x5500_0000,
                0x1000,
                MEM_RESERVE | MEM_COMMIT,
                4,
            )
            .unwrap();
        assert!(memory.commit_prepared_private_vm(stale_commit).is_none());
        assert!(memory.private_page_pin(owner, base).is_none());
        assert!(
            memory
                .virtual_allocation(owner, base)
                .unwrap()
                .private_committed_pages
                .is_empty()
        );
    }

    #[test]
    fn diagnostic_slab_commits_one_page_not_whole_reservation() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let base = memory
            .virtual_alloc_private_slab_for_test(owner, 0, 0x1000, MEM_COMMIT | MEM_RESERVE, 4)
            .unwrap();
        let incarnation = {
            let allocation = memory.virtual_allocation(owner, base).unwrap();
            assert_eq!(allocation.size, VIRTUAL_ALIGN);
            assert_eq!(
                allocation.backing,
                VirtualAllocationBacking::StablePrivateSlab
            );
            assert!(allocation.initial_bytes.is_empty());
            allocation.incarnation
        };
        assert!(memory.has_stable_private_slabs());
        assert!(memory.private_page_pin(owner, base + 0x1000).is_none());
        assert_eq!(memory.private_page_pins_for(owner).len(), 1);
        // The reservation has another 60 KiB, but the seed may touch only
        // the single committed page. A failed preflight leaves zero bytes.
        assert!(!memory.set_virtual_initial_bytes(
            owner,
            base,
            vec![0xa5; STABLE_PRIVATE_PAGE_SIZE + 1]
        ));
        assert!(memory.set_virtual_initial_bytes(
            owner,
            base,
            0x1122_3344u32.to_le_bytes().to_vec()
        ));
        let pin = memory.private_page_pin(owner, base).unwrap();
        assert_eq!(pin.identity().owner(), owner);
        assert_eq!(pin.identity().allocation_base(), base);
        assert_eq!(pin.identity().allocation_incarnation(), incarnation);
        assert_eq!(pin.identity().page_base(), base);
        assert!(!pin.native_ptr().is_null());
        assert_eq!((pin.native_ptr() as usize) % STABLE_PRIVATE_PAGE_SIZE, 0);
        assert_eq!(pin.native_ptr(), pin.clone().native_ptr());
        assert!(memory.contains_private_page_pin(&pin));
        assert!(pin.read_bytes(STABLE_PRIVATE_PAGE_SIZE, &mut [0]).is_err());
        assert_eq!(
            pin.write_bytes(STABLE_PRIVATE_PAGE_SIZE, &[0]),
            Err(PrivatePageAccessError::OutOfRange)
        );
        let mut bytes = [0; 4];
        pin.read_bytes(0, &mut bytes).unwrap();
        assert_eq!(bytes, 0x1122_3344u32.to_le_bytes());
        // No second copy may become an ongoing seed authority once pinned.
        assert!(!memory.set_virtual_initial_bytes(owner, base, vec![9]));
        assert!(
            memory
                .virtual_allocation(owner, base)
                .unwrap()
                .initial_bytes
                .is_empty()
        );
    }

    #[test]
    fn diagnostic_slab_seed_spans_two_pages_and_execution_fences_host_access() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let base = memory
            .virtual_alloc_private_slab_for_test(owner, 0, 0x2000, MEM_COMMIT | MEM_RESERVE, 4)
            .unwrap();
        let mut seed = vec![0x5a; STABLE_PRIVATE_PAGE_SIZE + 4];
        seed[STABLE_PRIVATE_PAGE_SIZE..].copy_from_slice(&[1, 2, 3, 4]);
        assert!(memory.set_virtual_initial_bytes(owner, base, seed));
        let first = memory.private_page_pin(owner, base).unwrap();
        let second = memory.private_page_pin(owner, base + 0x1000).unwrap();
        let pins = memory.private_page_pins_for(owner);
        assert_eq!(pins.len(), 2);
        assert_eq!(pins[0].identity().page_base(), base);
        assert_eq!(pins[1].identity().page_base(), base + 0x1000);
        let mut bytes = [0; 4];
        first
            .read_bytes(STABLE_PRIVATE_PAGE_SIZE - 4, &mut bytes)
            .unwrap();
        assert_eq!(bytes, [0x5a; 4]);
        second.read_bytes(0, &mut bytes).unwrap();
        assert_eq!(bytes, [1, 2, 3, 4]);
        let guard = second.begin_native_execution().unwrap();
        assert_eq!(
            second.read_bytes(0, &mut bytes),
            Err(PrivatePageAccessError::NativeExecutionActive)
        );
        assert_eq!(
            second.write_bytes(0, &[9]),
            Err(PrivatePageAccessError::NativeExecutionActive)
        );
        assert_eq!(
            second.begin_native_execution().err(),
            Some(PrivatePageAccessError::NativeExecutionActive)
        );
        drop(guard);
        second.write_bytes(0, &[9]).unwrap();
        second.read_bytes(0, &mut bytes).unwrap();
        assert_eq!(bytes[0], 9);
    }

    #[test]
    fn diagnostic_slab_pin_list_is_birth_scoped_and_excludes_legacy() {
        let mut handles = HandleTable::default();
        handles.register_boot_process(1);
        let parent = handles.process_birth_key(1).unwrap();
        let _child_handle = handles.create_process(2);
        let child = handles.process_birth_key(2).unwrap();
        let mut memory = MemorySystem::default();
        let slab_base = memory
            .virtual_alloc_private_slab_for_test(parent, 0, 0x1000, MEM_COMMIT | MEM_RESERVE, 4)
            .unwrap();
        let legacy_base = memory
            .virtual_alloc(
                child,
                VirtualAllocationSource::Private,
                0,
                0x1000,
                MEM_COMMIT | MEM_RESERVE,
                4,
            )
            .unwrap();
        assert_ne!(slab_base, legacy_base);
        assert_eq!(memory.private_page_pins_for(parent).len(), 1);
        assert!(memory.private_page_pins_for(child).is_empty());
        assert!(memory.private_page_pin(child, slab_base).is_none());
        assert!(memory.private_page_pin(child, legacy_base).is_none());
    }

    #[test]
    fn diagnostic_slab_clone_deep_copies_bytes_and_rejects_source_pin() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let base = memory
            .virtual_alloc_private_slab_for_test(owner, 0, 0x1000, MEM_COMMIT | MEM_RESERVE, 4)
            .unwrap();
        assert!(memory.set_virtual_initial_bytes(owner, base, vec![1, 2, 3, 4]));
        let source_pin = memory.private_page_pin(owner, base).unwrap();
        let copy = memory.clone();
        let copy_pin = copy.private_page_pin(owner, base).unwrap();
        assert!(copy.has_stable_private_slabs());
        assert!(memory.contains_private_page_pin(&source_pin));
        assert!(!copy.contains_private_page_pin(&source_pin));
        assert!(!memory.contains_private_page_pin(&copy_pin));
        assert_ne!(source_pin.native_ptr(), copy_pin.native_ptr());
        source_pin.write_bytes(0, &[0xa5]).unwrap();
        let mut source = [0; 4];
        let mut copied = [0; 4];
        source_pin.read_bytes(0, &mut source).unwrap();
        copy_pin.read_bytes(0, &mut copied).unwrap();
        assert_eq!(source, [0xa5, 2, 3, 4]);
        assert_eq!(copied, [1, 2, 3, 4]);
        let guard = source_pin.begin_native_execution().unwrap();
        let rejected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| memory.clone()));
        assert!(
            rejected.is_err(),
            "derived clone must fail closed during native writes"
        );
        drop(guard);
    }

    #[test]
    fn diagnostic_slab_release_invalidates_identity_but_pin_retains_bytes() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let base = memory
            .virtual_alloc_private_slab_for_test(owner, 0, 0x1000, MEM_COMMIT | MEM_RESERVE, 4)
            .unwrap();
        assert!(memory.set_virtual_initial_bytes(owner, base, vec![0x45]));
        let pin = memory.private_page_pin(owner, base).unwrap();
        assert!(!memory.virtual_free(owner, base, 0x1000, MEM_DECOMMIT));
        assert!(memory.contains_private_page_pin(&pin));
        assert!(memory.virtual_free(owner, base, 0, MEM_RELEASE));
        assert!(!memory.contains_private_page_pin(&pin));
        assert!(memory.private_page_pin(owner, base).is_none());
        assert!(memory.private_page_pins_for(owner).is_empty());
        assert!(
            memory.has_stable_private_slabs(),
            "checkpoint gate is sticky"
        );
        let mut byte = [0];
        pin.read_bytes(0, &mut byte).unwrap();
        assert_eq!(byte, [0x45]);
        let replacement = memory
            .virtual_alloc(
                owner,
                VirtualAllocationSource::Private,
                base,
                0x1000,
                MEM_COMMIT | MEM_RESERVE,
                4,
            )
            .unwrap();
        assert_eq!(replacement, base);
        assert_eq!(
            memory.virtual_allocation(owner, base).unwrap().backing,
            VirtualAllocationBacking::LegacySeed
        );
        assert!(!memory.contains_private_page_pin(&pin));
    }

    #[test]
    fn diagnostic_slab_rejects_unsupported_request_without_publication() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        for (address, size, flags) in [
            (0, 0x1000, MEM_RESERVE),
            (0, VIRTUAL_ALIGN + 1, MEM_COMMIT | MEM_RESERVE),
            (0x5200_0000, 0x1000, MEM_COMMIT | MEM_RESERVE),
        ] {
            assert!(
                memory
                    .virtual_alloc_private_slab_for_test(owner, address, size, flags, 4)
                    .is_none()
            );
        }
        assert_eq!(memory.virtual_allocations().count(), 0);
        assert!(!memory.has_stable_private_slabs());
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
