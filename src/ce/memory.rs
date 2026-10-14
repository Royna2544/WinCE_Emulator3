use std::{
    alloc::{Layout, alloc_zeroed, dealloc},
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet},
    ffi::c_void,
    fmt,
    ptr::NonNull,
    rc::{Rc, Weak},
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
const HEAP_PTR_LIMIT: u32 = 0x4000_0000;
const VIRTUAL_PTR_BASE: u32 = 0x5000_0000;
const HEAP_ALIGN: u32 = 16;
const VIRTUAL_ALIGN: u32 = 0x0001_0000;

#[derive(Debug)]
pub struct MemorySystem {
    // Detached VM proposals belong to exactly this in-memory lineage. Clone
    // deep-copies pages and receives a fresh identity, even when generations
    // and reservation holes happen to match the source.
    instance: Rc<()>,
    process_heap: u32,
    next_heap: u32,
    next_heap_ptr_by_owner: BTreeMap<ProcessBirthKey, u32>,
    retired_heap_owners: BTreeSet<ProcessBirthKey>,
    // Owner high-water marks are authoritative metadata. The global floor is
    // retained until native virtual pages are mapped by owner rather than VA.
    next_virtual_ptr: u32,
    next_virtual_ptr_by_owner: BTreeMap<ProcessBirthKey, u32>,
    heap_generation: u64,
    virtual_generation: u64,
    next_virtual_incarnation: u64,
    heaps: BTreeMap<u32, Heap>,
    allocations: BTreeMap<(ProcessBirthKey, u32), MemoryAllocation>,
    virtual_allocations: BTreeMap<(ProcessBirthKey, u32), VirtualAllocation>,
    // A prepared private reservation is not guest-visible. Keep its VA out of
    // reuse even if its token is dropped after a native mapping side effect.
    pending_private_holes: BTreeMap<(ProcessBirthKey, u32), u32>,
    stable_private_pages: BTreeMap<PrivatePageIdentity, Rc<StablePrivatePage>>,
    // A weak conflict fence for still-detached fixed-page proposals. It is
    // not a second byte owner: only page pins carry bytes and budget. A new
    // proposal for an occupied offset fails instead of aliasing that offset
    // or inventing a second host block before either token is published.
    pending_fixed_blocks:
        RefCell<BTreeMap<(ProcessBirthKey, u32, u64, u32), Weak<HostPrivateBlock>>>,
    // Public file views alias pages by mapping object and file offset, not by
    // guest VA. A page is charged once even when several views pin it.
    stable_shared_pages: BTreeMap<(u32, u32), Rc<StablePrivatePage>>,
    shared_view_bindings: BTreeMap<(ProcessBirthKey, u32), SharedViewBinding>,
    private_page_budget: Rc<PrivatePageBudget>,
    // A native pin can outlive a released VM record. Keep the checkpoint
    // unsupported flag set for this MemorySystem lineage even after release.
    stable_private_slabs_seen: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heap {
    pub handle: u32,
    /// `None` is the shared numeric process-heap handle; its blocks remain
    /// separately owned by each process birth.
    pub owner: Option<ProcessBirthKey>,
    pub options: u32,
    pub initial_size: u32,
    pub maximum_size: u32,
    pub destroyed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryAllocation {
    pub owner: ProcessBirthKey,
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
    instance: Rc<()>,
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
    instance: Rc<()>,
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

/// Describes the API or subsystem that produced a virtual allocation. Source
/// and backing are separate: compatibility views still use legacy seeds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualAllocationSource {
    Private,
    PrivateExCompatibility,
    ProcessHeapArena,
    ThreadStack,
    FileViewLegacy,
    FileViewSlab,
    LocaleViewLegacy,
    ToolhelpSnapshotLegacy,
    SharedCompatibility,
}

/// Backing is explicit. COREDLL's checked private VM transaction publishes
/// StablePrivateSlab; unmigrated file views and compatibility producers retain
/// LegacySeed until their own native mapping paths are converted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualAllocationBacking {
    LegacySeed,
    StablePrivateSlab,
    StableSharedViewSlab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SharedViewBinding {
    mapping_handle: u32,
    offset: u32,
    size: u32,
}

pub const STABLE_PRIVATE_PAGE_SIZE: usize = 0x1000;
const FIXED_PRIVATE_BLOCK_SIZE: usize = 0x10_0000;
/// Physical RAM reported by the target device's `sources/SYSTEM.TXT`.
/// One runtime setting may override this; the same budget drives private
/// committed pages and the GlobalMemoryStatus physical-memory fields.
pub const DEFAULT_GUEST_PHYSICAL_RAM_BYTES: u64 = 195_350_528;

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

#[derive(Debug)]
struct PrivatePageBudget {
    limit_bytes: Cell<u64>,
    // Guest RAM occupied by the OS and other boot-time consumers in the
    // device dump. This is not a private slab, but it reduces the amount a
    // private MEM_COMMIT may charge.
    boot_reserved_bytes: Cell<u64>,
    // Includes detached proposals and pages retained by native/quarantine
    // pins even after canonical VM metadata has been released.
    used_bytes: Cell<u64>,
}

impl PrivatePageBudget {
    fn new(limit_bytes: u64, boot_reserved_bytes: u64) -> Self {
        Self {
            limit_bytes: Cell::new(limit_bytes),
            boot_reserved_bytes: Cell::new(boot_reserved_bytes),
            used_bytes: Cell::new(0),
        }
    }

    fn available_bytes(&self) -> u64 {
        self.limit_bytes.get().saturating_sub(
            self.boot_reserved_bytes
                .get()
                .saturating_add(self.used_bytes.get()),
        )
    }

    fn can_fit(&self, new_pages: usize) -> bool {
        u64::try_from(new_pages)
            .ok()
            .and_then(|count| count.checked_mul(STABLE_PRIVATE_PAGE_SIZE as u64))
            .is_some_and(|bytes| bytes <= self.available_bytes())
    }
}

/// One 1-MiB host VA reservation for fixed heap/stack pages. Only committed
/// 4-KiB guest pages acquire host RAM; the reserved gaps remain inaccessible.
/// Every live page and native pin keeps its exact block alive.
struct HostPrivateBlock {
    base: NonNull<u8>,
    occupied: RefCell<[bool; FIXED_PRIVATE_BLOCK_SIZE / STABLE_PRIVATE_PAGE_SIZE]>,
}

impl HostPrivateBlock {
    #[cfg(windows)]
    fn reserve() -> Option<Rc<Self>> {
        // SAFETY: null asks Windows to choose a fresh 1-MiB reservation.
        let base = unsafe {
            host_virtual_alloc(std::ptr::null_mut(), FIXED_PRIVATE_BLOCK_SIZE, 0x2000, 0x01)
        };
        Some(Rc::new(Self {
            base: NonNull::new(base.cast())?,
            occupied: RefCell::new([false; FIXED_PRIVATE_BLOCK_SIZE / STABLE_PRIVATE_PAGE_SIZE]),
        }))
    }

    #[cfg(not(windows))]
    fn reserve() -> Option<Rc<Self>> {
        None
    }

    #[cfg(windows)]
    fn commit_page(&self, offset: usize) -> Option<NonNull<u8>> {
        if offset >= FIXED_PRIVATE_BLOCK_SIZE || offset % STABLE_PRIVATE_PAGE_SIZE != 0 {
            return None;
        }
        let index = offset / STABLE_PRIVATE_PAGE_SIZE;
        if self.occupied.borrow()[index] {
            // A concurrent detached proposal must not become a second byte
            // authority for an already committed offset in this block.
            return None;
        }
        // SAFETY: the checked offset is wholly inside this live reservation.
        let wanted = unsafe { self.base.as_ptr().add(offset) };
        // SAFETY: MEM_COMMIT of one previously reserved page returns its exact
        // address or null. The block is retained by the resulting page.
        let actual =
            unsafe { host_virtual_alloc(wanted.cast(), STABLE_PRIVATE_PAGE_SIZE, 0x1000, 0x04) };
        let bytes = NonNull::new(actual.cast::<u8>())?;
        if bytes.as_ptr() != wanted {
            // Explicit-address VirtualAlloc should never choose another VA.
            // Do not publish an unexpected host pointer as a page.
            return None;
        }
        self.occupied.borrow_mut()[index] = true;
        Some(bytes)
    }

    #[cfg(not(windows))]
    fn commit_page(&self, _offset: usize) -> Option<NonNull<u8>> {
        None
    }

    #[cfg(windows)]
    fn decommit_page(&self, offset: usize) -> bool {
        let index = offset / STABLE_PRIVATE_PAGE_SIZE;
        if offset >= FIXED_PRIVATE_BLOCK_SIZE || !self.occupied.borrow()[index] {
            return false;
        }
        // SAFETY: every stored offset came from commit_page, and this block
        // remains reserved until its last strong reference drops.
        let address = unsafe { self.base.as_ptr().add(offset) };
        if unsafe { host_virtual_free(address.cast(), STABLE_PRIVATE_PAGE_SIZE, 0x4000) } == 0 {
            return false;
        }
        self.occupied.borrow_mut()[index] = false;
        true
    }

    #[cfg(not(windows))]
    fn decommit_page(&self, _offset: usize) -> bool {
        false
    }
}

impl Drop for HostPrivateBlock {
    fn drop(&mut self) {
        #[cfg(windows)]
        {
            // SAFETY: MEM_RELEASE takes the original reservation base and a
            // zero size. No page or native pin remains at the last Rc drop.
            let _ = unsafe { host_virtual_free(self.base.as_ptr().cast(), 0, 0x8000) };
        }
    }
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    #[link_name = "VirtualAlloc"]
    fn host_virtual_alloc(
        address: *mut c_void,
        size: usize,
        allocation_type: u32,
        protect: u32,
    ) -> *mut c_void;
    #[link_name = "VirtualFree"]
    fn host_virtual_free(address: *mut c_void, size: usize, free_type: u32) -> i32;
}

enum PrivatePageStorage {
    Independent,
    HostBlock {
        block: Rc<HostPrivateBlock>,
        offset: usize,
    },
}

/// The only owner of private page bytes. Ordinary private and shared-view
/// pages keep separate 4-KiB allocations; fixed heap/stack pages may occupy
/// a committed offset in a reserved 1-MiB block. The Rc wrapper never moves.
/// Raw native and bounded host access are fenced by execution guards.
struct StablePrivatePage {
    bytes: NonNull<u8>,
    storage: PrivatePageStorage,
    native_execution_guards: Cell<u32>,
    allow_nested_native: bool,
    budget: Rc<PrivatePageBudget>,
}

impl fmt::Debug for StablePrivatePage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StablePrivatePage")
            .field(
                "native_execution_guards",
                &self.native_execution_guards.get(),
            )
            .finish_non_exhaustive()
    }
}

impl StablePrivatePage {
    fn zeroed(budget: &Rc<PrivatePageBudget>) -> Option<Self> {
        let layout =
            Layout::from_size_align(STABLE_PRIVATE_PAGE_SIZE, STABLE_PRIVATE_PAGE_SIZE).ok()?;
        let charged = budget
            .used_bytes
            .get()
            .checked_add(STABLE_PRIVATE_PAGE_SIZE as u64)?;
        if budget.available_bytes() < STABLE_PRIVATE_PAGE_SIZE as u64 {
            return None;
        }
        budget.used_bytes.set(charged);
        // SAFETY: layout is nonzero and power-of-two aligned. A null result
        // is an ordinary allocation failure, never a published guest page.
        let Some(bytes) = NonNull::new(unsafe { alloc_zeroed(layout) }) else {
            budget
                .used_bytes
                .set(charged - STABLE_PRIVATE_PAGE_SIZE as u64);
            return None;
        };
        Some(Self {
            bytes,
            storage: PrivatePageStorage::Independent,
            native_execution_guards: Cell::new(0),
            allow_nested_native: false,
            budget: Rc::clone(budget),
        })
    }

    fn zeroed_shared(budget: &Rc<PrivatePageBudget>) -> Option<Self> {
        let mut page = Self::zeroed(budget)?;
        page.allow_nested_native = true;
        Some(page)
    }

    fn zeroed_in_block(
        budget: &Rc<PrivatePageBudget>,
        block: &Rc<HostPrivateBlock>,
        offset: usize,
    ) -> Option<Self> {
        let charged = budget
            .used_bytes
            .get()
            .checked_add(STABLE_PRIVATE_PAGE_SIZE as u64)?;
        if budget.available_bytes() < STABLE_PRIVATE_PAGE_SIZE as u64 {
            return None;
        }
        let bytes = block.commit_page(offset)?;
        budget.used_bytes.set(charged);
        Some(Self {
            bytes,
            storage: PrivatePageStorage::HostBlock {
                block: Rc::clone(block),
                offset,
            },
            native_execution_guards: Cell::new(0),
            allow_nested_native: false,
            budget: Rc::clone(budget),
        })
    }

    fn deep_copy(
        &self,
        budget: &Rc<PrivatePageBudget>,
        cloned_blocks: &mut BTreeMap<usize, Rc<HostPrivateBlock>>,
    ) -> Self {
        // MemorySystem::Clone cannot return Result. Refuse to copy while the
        // native engine might be writing; a supported checkpoint operation
        // remains separately gated until machine-wide relineage exists.
        assert!(
            self.native_execution_guards.get() == 0,
            "cannot clone stable private bytes during native execution"
        );
        let copy = match &self.storage {
            PrivatePageStorage::Independent if self.allow_nested_native => {
                Self::zeroed_shared(budget)
            }
            PrivatePageStorage::Independent => Self::zeroed(budget),
            PrivatePageStorage::HostBlock { block, offset } => {
                let key = block.base.as_ptr() as usize;
                let new_block = match cloned_blocks.get(&key) {
                    Some(block) => Rc::clone(block),
                    None => {
                        let block = HostPrivateBlock::reserve()
                            .expect("cannot reserve cloned fixed private block");
                        cloned_blocks.insert(key, Rc::clone(&block));
                        block
                    }
                };
                Self::zeroed_in_block(budget, &new_block, *offset)
            }
        }
        .expect("cannot clone stable page after host allocation failure");
        // SAFETY: the guard above excludes native execution, all host byte
        // methods finish before returning, and Rc keeps the page alive.
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.bytes.as_ptr(),
                copy.bytes.as_ptr(),
                STABLE_PRIVATE_PAGE_SIZE,
            );
        }
        copy
    }
}

impl Drop for StablePrivatePage {
    fn drop(&mut self) {
        match &self.storage {
            PrivatePageStorage::Independent => {
                let layout =
                    Layout::from_size_align(STABLE_PRIVATE_PAGE_SIZE, STABLE_PRIVATE_PAGE_SIZE)
                        .expect("fixed private page layout");
                // SAFETY: independent bytes came from alloc_zeroed exactly
                // once and native users retain a pin through checked unmap.
                unsafe { dealloc(self.bytes.as_ptr(), layout) };
            }
            PrivatePageStorage::HostBlock { block, offset } => {
                if !block.decommit_page(*offset) {
                    // Host decommit did not establish release. Keep the whole
                    // reservation and this physical-page charge quarantined.
                    std::mem::forget(Rc::clone(block));
                    return;
                }
            }
        }
        self.budget.used_bytes.set(
            self.budget
                .used_bytes
                .get()
                .checked_sub(STABLE_PRIVATE_PAGE_SIZE as u64)
                .expect("private page budget underflow"),
        );
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
        self.page.bytes.as_ptr().cast::<c_void>()
    }

    /// Only fixed heap/stack pages use block storage. Shared-view aliases
    /// deliberately retain their independent canonical page ownership.
    pub fn is_host_block_backed(&self) -> bool {
        matches!(self.page.storage, PrivatePageStorage::HostBlock { .. })
    }

    pub fn contiguous_in_same_host_block(&self, next: &Self) -> bool {
        match (&self.page.storage, &next.page.storage) {
            (
                PrivatePageStorage::HostBlock {
                    block: left,
                    offset: left_offset,
                },
                PrivatePageStorage::HostBlock {
                    block: right,
                    offset: right_offset,
                },
            ) => {
                Rc::ptr_eq(left, right)
                    && left_offset.checked_add(STABLE_PRIVATE_PAGE_SIZE) == Some(*right_offset)
                    && self.identity.owner == next.identity.owner
                    && self.identity.allocation_base == next.identity.allocation_base
                    && self.identity.allocation_incarnation == next.identity.allocation_incarnation
                    && self
                        .identity
                        .page_base
                        .checked_add(STABLE_PRIVATE_PAGE_SIZE as u32)
                        == Some(next.identity.page_base)
            }
            _ => false,
        }
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
                self.page.bytes.as_ptr().add(offset),
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
                self.page.bytes.as_ptr().add(offset),
                bytes.len(),
            );
        }
        Ok(())
    }

    pub fn begin_native_execution(
        &self,
    ) -> std::result::Result<StablePrivatePageExecutionGuard, PrivatePageAccessError> {
        if self.page.native_execution_guards.get() != 0 && !self.page.allow_nested_native {
            return Err(PrivatePageAccessError::NativeExecutionActive);
        }
        let next = self
            .page
            .native_execution_guards
            .get()
            .checked_add(1)
            .ok_or(PrivatePageAccessError::NativeExecutionActive)?;
        // Nested process launches run serially on this Rc-bound thread. The
        // parent lease remains fenced while a child maps the same shared page.
        self.page.native_execution_guards.set(next);
        Ok(StablePrivatePageExecutionGuard {
            page: Rc::clone(&self.page),
        })
    }

    fn check_access(
        &self,
        offset: usize,
        len: usize,
    ) -> std::result::Result<(), PrivatePageAccessError> {
        if self.page.native_execution_guards.get() != 0 {
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
        self.page.native_execution_guards.set(
            self.page
                .native_execution_guards
                .get()
                .checked_sub(1)
                .expect("stable page execution guard underflow"),
        );
    }
}

impl VirtualAllocationSource {
    fn is_private(self) -> bool {
        matches!(self, Self::Private | Self::PrivateExCompatibility)
    }

    fn is_native_fixed(self) -> bool {
        matches!(self, Self::ProcessHeapArena | Self::ThreadStack)
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
    instance: Rc<()>,
    // Some(None) means a null-base proposal created this owner's cursor;
    // Some(Some(cursor)) remembers its prior value for safe pre-native abort.
    previous_owner_cursor: Option<Option<u32>>,
    owner: ProcessBirthKey,
    result_ptr: u32,
    proposed_allocation: VirtualAllocation,
    original_allocation: Option<VirtualAllocation>,
    expected_virtual_generation: u64,
    new_page_pins: Vec<StablePrivatePagePin>,
}

/// A native side effect to validate before publishing a private VM change.
/// The strong pin remains in the detached token until commit or abort.
#[derive(Debug, Clone)]
pub enum PrivateVmNativePageChange {
    Unmap(StablePrivatePagePin),
    Protect {
        pin: StablePrivatePagePin,
        new_protect: u32,
    },
}

impl PrivateVmNativePageChange {
    pub fn pin(&self) -> &StablePrivatePagePin {
        match self {
            Self::Unmap(pin) | Self::Protect { pin, .. } => pin,
        }
    }
}

/// Release, decommit, or protect is detached until checked native changes
/// finish. Its original allocation is retained for a generation/identity
/// recheck; dropping the token leaves the published metadata unchanged.
#[derive(Debug)]
#[must_use = "prepared private VM changes must be committed or abandoned"]
pub struct PreparedPrivateVmChange {
    instance: Rc<()>,
    owner: ProcessBirthKey,
    original_allocation: VirtualAllocation,
    proposed_allocation: Option<VirtualAllocation>,
    expected_virtual_generation: u64,
    native_page_changes: Vec<PrivateVmNativePageChange>,
    old_protect: Option<u32>,
}

impl PreparedPrivateVmChange {
    pub fn owner(&self) -> ProcessBirthKey {
        self.owner
    }

    pub fn allocation_base(&self) -> u32 {
        self.original_allocation.base
    }

    pub fn native_page_changes(&self) -> &[PrivateVmNativePageChange] {
        &self.native_page_changes
    }

    pub fn is_release(&self) -> bool {
        self.proposed_allocation.is_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivateVmChangeResult {
    Free,
    Protect { old_protect: u32 },
}

/// The contiguous run, starting at the queried page, with one private VM
/// state/protection. `page_protect == None` means reserved but uncommitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrivateVmPageQuery {
    pub allocation_base: u32,
    pub allocation_protect: u32,
    pub region_base: u32,
    pub region_size: u32,
    pub page_protect: Option<u32>,
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
        let private_page_budget = Rc::new(PrivatePageBudget::new(
            self.private_page_budget.limit_bytes.get(),
            self.private_page_budget.boot_reserved_bytes.get(),
        ));
        let mut cloned_blocks = BTreeMap::new();
        Self {
            instance: Rc::new(()),
            process_heap: self.process_heap,
            next_heap: self.next_heap,
            next_heap_ptr_by_owner: self.next_heap_ptr_by_owner.clone(),
            retired_heap_owners: self.retired_heap_owners.clone(),
            next_virtual_ptr: self.next_virtual_ptr,
            next_virtual_ptr_by_owner: self.next_virtual_ptr_by_owner.clone(),
            heap_generation: self.heap_generation,
            virtual_generation: self.virtual_generation,
            next_virtual_incarnation: self.next_virtual_incarnation,
            heaps: self.heaps.clone(),
            allocations: self.allocations.clone(),
            virtual_allocations: self.virtual_allocations.clone(),
            // Detached tokens cannot cross this new lineage, and their
            // possible native side effects remain in the source owner only.
            pending_private_holes: BTreeMap::new(),
            stable_private_pages: self
                .stable_private_pages
                .iter()
                .map(|(identity, page)| {
                    (
                        *identity,
                        Rc::new(page.deep_copy(&private_page_budget, &mut cloned_blocks)),
                    )
                })
                .collect(),
            pending_fixed_blocks: RefCell::new(BTreeMap::new()),
            stable_shared_pages: self
                .stable_shared_pages
                .iter()
                .map(|(identity, page)| {
                    (
                        *identity,
                        Rc::new(page.deep_copy(&private_page_budget, &mut cloned_blocks)),
                    )
                })
                .collect(),
            shared_view_bindings: self.shared_view_bindings.clone(),
            private_page_budget,
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
                owner: None,
                options: 0,
                initial_size: 0,
                maximum_size: 0,
                destroyed: false,
            },
        );
        Self {
            instance: Rc::new(()),
            process_heap: PROCESS_HEAP_HANDLE,
            next_heap: PROCESS_HEAP_HANDLE + 0x1000,
            next_heap_ptr_by_owner: BTreeMap::new(),
            retired_heap_owners: BTreeSet::new(),
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
            pending_fixed_blocks: RefCell::new(BTreeMap::new()),
            stable_shared_pages: BTreeMap::new(),
            shared_view_bindings: BTreeMap::new(),
            private_page_budget: Rc::new(PrivatePageBudget::new(
                DEFAULT_GUEST_PHYSICAL_RAM_BYTES,
                0,
            )),
            stable_private_slabs_seen: false,
        }
    }
}

impl MemorySystem {
    pub fn get_process_heap(&self) -> u32 {
        self.process_heap
    }

    /// Metadata-only convenience; raw APIs must map/initialize before commit.
    pub fn local_alloc(&mut self, owner: ProcessBirthKey, flags: u32, bytes: u32) -> Option<u32> {
        let prepared = self.prepare_local_alloc(owner, flags, bytes)?;
        self.commit_allocation(prepared)
    }

    pub fn prepare_local_alloc(
        &mut self,
        owner: ProcessBirthKey,
        flags: u32,
        bytes: u32,
    ) -> Option<PreparedAllocation> {
        if flags & !LMEM_VALID_FLAGS != 0 {
            return None;
        }
        let heap_flags = if flags & LMEM_ZEROINIT != 0 {
            HEAP_ZERO_MEMORY
        } else {
            0
        };
        self.prepare_heap_alloc(owner, self.process_heap, heap_flags, bytes)
    }

    /// Metadata-only local reallocation; it does not copy or zero guest bytes.
    pub fn local_re_alloc(
        &mut self,
        owner: ProcessBirthKey,
        ptr: u32,
        bytes: u32,
        flags: u32,
    ) -> Option<u32> {
        self.local_re_alloc_detail(owner, ptr, bytes, flags)
            .map(|result| result.ptr)
    }

    /// Metadata-only convenience for callers without guest-memory backing.
    /// Raw guest APIs must use `prepare_local_re_alloc` and commit only after
    /// their copy and zero-fill operations succeed.
    pub fn local_re_alloc_detail(
        &mut self,
        owner: ProcessBirthKey,
        ptr: u32,
        bytes: u32,
        flags: u32,
    ) -> Option<Reallocation> {
        let prepared = self.prepare_local_re_alloc(owner, ptr, bytes, flags)?;
        self.commit_reallocation(prepared)
    }

    pub fn prepare_local_re_alloc(
        &mut self,
        owner: ProcessBirthKey,
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
        self.prepare_heap_re_alloc(owner, self.process_heap, heap_flags, ptr, bytes)
    }

    pub fn local_size(&self, owner: ProcessBirthKey, ptr: u32) -> Option<u32> {
        self.heap_size(owner, self.process_heap, 0, ptr)
    }

    pub fn local_free(&mut self, owner: ProcessBirthKey, ptr: u32) -> bool {
        ptr == 0 || self.heap_free(owner, self.process_heap, 0, ptr)
    }

    pub fn heap_create(
        &mut self,
        owner: ProcessBirthKey,
        options: u32,
        initial_size: u32,
        maximum_size: u32,
    ) -> Option<u32> {
        if self.retired_heap_owners.contains(&owner)
            || options & !(HEAP_NO_SERIALIZE | HEAP_SHARED_READONLY) != 0
        {
            return None;
        }
        let handle = self.next_heap;
        self.next_heap = self.next_heap.saturating_add(0x1000);
        self.heaps.insert(
            handle,
            Heap {
                handle,
                owner: Some(owner),
                options,
                initial_size,
                maximum_size,
                destroyed: false,
            },
        );
        self.bump_heap_generation();
        Some(handle)
    }

    pub fn heap_destroy(&mut self, owner: ProcessBirthKey, heap: u32) -> bool {
        if heap == self.process_heap {
            return false;
        }
        let Some(item) = self.heaps.get_mut(&heap) else {
            return false;
        };
        if item.destroyed || item.owner != Some(owner) || self.retired_heap_owners.contains(&owner)
        {
            return false;
        }
        item.destroyed = true;
        self.allocations
            .retain(|(allocation_owner, _), allocation| {
                *allocation_owner != owner || allocation.heap != heap
            });
        self.bump_heap_generation();
        true
    }

    /// Metadata-only convenience; it does not map or zero guest bytes.
    pub fn heap_alloc(
        &mut self,
        owner: ProcessBirthKey,
        heap: u32,
        flags: u32,
        bytes: u32,
    ) -> Option<u32> {
        let prepared = self.prepare_heap_alloc(owner, heap, flags, bytes)?;
        self.commit_allocation(prepared)
    }

    pub fn prepare_heap_alloc(
        &mut self,
        owner: ProcessBirthKey,
        heap: u32,
        flags: u32,
        bytes: u32,
    ) -> Option<PreparedAllocation> {
        if flags & !(HEAP_GENERATE_EXCEPTIONS | HEAP_NO_SERIALIZE | HEAP_ZERO_MEMORY) != 0
            || !self.is_live_heap(owner, heap)
        {
            return None;
        }
        let actual_size = sanitize_size(bytes)?;
        let ptr = self.allocate_heap_ptr(owner, actual_size)?;
        Some(PreparedAllocation {
            instance: Rc::clone(&self.instance),
            allocation: MemoryAllocation {
                owner,
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
        let key = (allocation.owner, allocation.ptr);
        if !Rc::ptr_eq(&self.instance, &prepared.instance)
            || !self.is_live_heap(allocation.owner, allocation.heap)
            || self.allocations.contains_key(&key)
        {
            return None;
        }
        let ptr = allocation.ptr;
        self.allocations.insert(key, allocation);
        self.bump_heap_generation();
        Some(ptr)
    }

    /// Discard uncommitted metadata, retaining its possibly dirtied address hole.
    pub fn abort_allocation(&mut self, _prepared: PreparedAllocation) {}

    /// Metadata-only heap reallocation; it does not copy or zero guest bytes.
    pub fn heap_re_alloc(
        &mut self,
        owner: ProcessBirthKey,
        heap: u32,
        flags: u32,
        ptr: u32,
        bytes: u32,
    ) -> Option<u32> {
        self.heap_re_alloc_detail(owner, heap, flags, ptr, bytes)
            .map(|result| result.ptr)
    }

    /// Metadata-only convenience for callers without guest-memory backing.
    /// Raw guest APIs must use `prepare_heap_re_alloc` and commit only after
    /// their copy and zero-fill operations succeed.
    pub fn heap_re_alloc_detail(
        &mut self,
        owner: ProcessBirthKey,
        heap: u32,
        flags: u32,
        ptr: u32,
        bytes: u32,
    ) -> Option<Reallocation> {
        let prepared = self.prepare_heap_re_alloc(owner, heap, flags, ptr, bytes)?;
        self.commit_reallocation(prepared)
    }

    pub fn prepare_heap_re_alloc(
        &mut self,
        owner: ProcessBirthKey,
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
            || !self.is_live_heap(owner, heap)
        {
            return None;
        }
        let actual_size = sanitize_size(bytes)?;
        let allocation = self.allocations.get(&(owner, ptr))?.clone();
        if allocation.heap != heap || ptr < POINTER_FLOOR {
            return None;
        }
        let old_actual_size = allocation.actual_size;
        if actual_size <= allocation.actual_size {
            return Some(PreparedReallocation {
                instance: Rc::clone(&self.instance),
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
        let new_ptr = self.allocate_heap_ptr(owner, actual_size)?;
        Some(PreparedReallocation {
            instance: Rc::clone(&self.instance),
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
        let owner = prepared.original_allocation.owner;
        let old_ptr = prepared.result.old_ptr;
        let destination_conflict =
            prepared.result.moved && self.allocations.contains_key(&(owner, prepared.result.ptr));
        if !Rc::ptr_eq(&self.instance, &prepared.instance)
            || !self.is_live_heap(owner, prepared.original_allocation.heap)
            || self.allocations.get(&(owner, old_ptr)) != Some(&prepared.original_allocation)
            || destination_conflict
        {
            return None;
        }

        let result = prepared.result.clone();
        if result.moved {
            self.allocations.remove(&(owner, old_ptr));
            self.allocations.insert(
                (owner, result.ptr),
                MemoryAllocation {
                    owner,
                    ptr: result.ptr,
                    heap: prepared.original_allocation.heap,
                    requested_size: prepared.requested_size,
                    actual_size: result.new_actual_size,
                    zeroed: prepared.original_allocation.zeroed || prepared.zero_requested,
                },
            );
        } else {
            let allocation = self.allocations.get_mut(&(owner, old_ptr))?;
            allocation.requested_size = prepared.requested_size;
            allocation.actual_size = result.new_actual_size;
            allocation.zeroed |= prepared.zero_requested;
        }
        self.bump_heap_generation();
        Some(result)
    }

    /// Discard uncommitted metadata without reusing a possibly written address.
    pub fn abort_reallocation(&mut self, _prepared: PreparedReallocation) {}

    pub fn heap_free(&mut self, owner: ProcessBirthKey, heap: u32, flags: u32, ptr: u32) -> bool {
        if flags & !HEAP_NO_SERIALIZE != 0 || ptr < POINTER_FLOOR || !self.is_live_heap(owner, heap)
        {
            return false;
        }
        let key = (owner, ptr);
        let Some(allocation) = self.allocations.get(&key) else {
            return false;
        };
        if allocation.heap != heap {
            return false;
        }
        self.allocations.remove(&key);
        self.bump_heap_generation();
        true
    }

    pub fn heap_size(
        &self,
        owner: ProcessBirthKey,
        heap: u32,
        flags: u32,
        ptr: u32,
    ) -> Option<u32> {
        if flags & !HEAP_NO_SERIALIZE != 0 || ptr < POINTER_FLOOR || !self.is_live_heap(owner, heap)
        {
            return None;
        }
        let allocation = self.allocations.get(&(owner, ptr))?;
        (allocation.heap == heap).then_some(allocation.actual_size)
    }

    pub fn heap_validate(&self, owner: ProcessBirthKey, heap: u32, flags: u32, ptr: u32) -> bool {
        if flags & !HEAP_NO_SERIALIZE != 0 || !self.is_live_heap(owner, heap) {
            return false;
        }
        ptr == 0
            || (ptr >= POINTER_FLOOR
                && self
                    .allocations
                    .get(&(owner, ptr))
                    .is_some_and(|allocation| allocation.heap == heap))
    }

    /// Publish a fixed process heap or stack reservation in the same
    /// owner-qualified VM directory as private VirtualAlloc. Reserving costs
    /// no guest RAM. Stack pages are fault-prepared; heap growth must be
    /// committed by its allocation path before a guest pointer is returned.
    pub fn reserve_fixed_private_range(
        &mut self,
        owner: ProcessBirthKey,
        source: VirtualAllocationSource,
        base: u32,
        size: u32,
        protect: u32,
    ) -> Option<u32> {
        self.reserve_fixed_private_ranges(owner, &[(source, base, size, protect)])?;
        Some(base)
    }

    /// Publish a set of fixed reservations together. Validation is complete
    /// before any record, incarnation, or generation changes, so a bad later
    /// stack cannot strand a newly reserved process heap. Exact existing
    /// reservations are accepted without modifying their committed pages.
    pub fn reserve_fixed_private_ranges(
        &mut self,
        owner: ProcessBirthKey,
        ranges: &[(VirtualAllocationSource, u32, u32, u32)],
    ) -> Option<Vec<(u32, u32)>> {
        let page_size = STABLE_PRIVATE_PAGE_SIZE as u32;
        let mut new_ranges = Vec::new();
        new_ranges.try_reserve_exact(ranges.len()).ok()?;
        let mut result = Vec::new();
        result.try_reserve_exact(ranges.len()).ok()?;
        for (index, &(source, base, size, protect)) in ranges.iter().enumerate() {
            if !source.is_native_fixed()
                || base < POINTER_FLOOR
                || base & (page_size - 1) != 0
                || size == 0
                || size & (page_size - 1) != 0
                || base.checked_add(size).is_none()
                || protect != 0x04
                || (source == VirtualAllocationSource::ProcessHeapArena && base != HEAP_PTR_BASE)
                || ranges[..index]
                    .iter()
                    .any(|&(_, earlier_base, earlier_size, _)| {
                        ranges_overlap(base, size, earlier_base, earlier_size)
                    })
            {
                return None;
            }
            result.push((base, size));
            if let Some(existing) = self.virtual_allocation(owner, base) {
                if existing.size != size
                    || existing.source != source
                    || existing.backing != VirtualAllocationBacking::StablePrivateSlab
                    || existing.protect != protect
                {
                    return None;
                }
                continue;
            }
            // Heap allocations intentionally occupy the heap reservation;
            // stacks retain the ordinary heap collision fence.
            let overlaps = if source == VirtualAllocationSource::ProcessHeapArena {
                self.virtual_allocations.values().any(|allocation| {
                    (allocation.owner == owner
                        || allocation.backing == VirtualAllocationBacking::LegacySeed)
                        && ranges_overlap(base, size, allocation.base, allocation.size)
                }) || self.pending_private_holes.iter().any(
                    |((hole_owner, hole_base), hole_size)| {
                        *hole_owner == owner && ranges_overlap(base, size, *hole_base, *hole_size)
                    },
                )
            } else {
                self.private_vm_overlaps(owner, base, size)
            };
            if overlaps {
                return None;
            }
            new_ranges.push((source, base, size, protect));
        }
        let next_incarnation = self
            .next_virtual_incarnation
            .checked_add(u64::try_from(new_ranges.len()).ok()?)?;
        let inserted = !new_ranges.is_empty();
        let mut incarnation = self.next_virtual_incarnation;
        for (source, base, size, protect) in new_ranges {
            self.virtual_allocations.insert(
                (owner, base),
                VirtualAllocation {
                    base,
                    size,
                    allocation_type: MEM_RESERVE,
                    protect,
                    initial_bytes: Vec::new(),
                    owner,
                    incarnation,
                    source,
                    backing: VirtualAllocationBacking::StablePrivateSlab,
                    private_committed_pages: BTreeMap::new(),
                },
            );
            incarnation += 1;
        }
        self.next_virtual_incarnation = next_incarnation;
        if inserted {
            self.stable_private_slabs_seen = true;
            self.bump_virtual_generation();
        }
        Some(result)
    }

    fn new_fixed_private_page(
        &self,
        identity: PrivatePageIdentity,
        local_blocks: &mut BTreeMap<u32, Rc<HostPrivateBlock>>,
    ) -> Option<Rc<StablePrivatePage>> {
        #[cfg(not(windows))]
        {
            let _ = (identity, local_blocks);
            return Some(Rc::new(StablePrivatePage::zeroed(
                &self.private_page_budget,
            )?));
        }
        #[cfg(windows)]
        {
            let block_base = identity.page_base & !(FIXED_PRIVATE_BLOCK_SIZE as u32 - 1);
            let block = if let Some(block) = local_blocks.get(&block_base) {
                Rc::clone(block)
            } else {
                let first = PrivatePageIdentity {
                    page_base: block_base,
                    ..identity
                };
                let last = PrivatePageIdentity {
                    page_base: block_base.checked_add(FIXED_PRIVATE_BLOCK_SIZE as u32)?,
                    ..identity
                };
                let existing =
                    self.stable_private_pages
                        .range(first..last)
                        .find_map(|(_, page)| match &page.storage {
                            PrivatePageStorage::HostBlock { block, .. } => Some(Rc::clone(block)),
                            PrivatePageStorage::Independent => None,
                        });
                let key = (
                    identity.owner,
                    identity.allocation_base,
                    identity.allocation_incarnation,
                    block_base,
                );
                let block = match existing {
                    Some(block) => block,
                    None => {
                        let mut pending = self.pending_fixed_blocks.borrow_mut();
                        pending.retain(|_, block| block.strong_count() != 0);
                        match pending.get(&key).and_then(Weak::upgrade) {
                            Some(block) => block,
                            None => {
                                let block = HostPrivateBlock::reserve()?;
                                pending.insert(key, Rc::downgrade(&block));
                                block
                            }
                        }
                    }
                };
                local_blocks.insert(block_base, Rc::clone(&block));
                block
            };
            let offset = (identity.page_base - block_base) as usize;
            Some(Rc::new(StablePrivatePage::zeroed_in_block(
                &self.private_page_budget,
                &block,
                offset,
            )?))
        }
    }

    /// Prepare every page of one host-accessed span in a fixed stack
    /// reservation. The entire span must belong to this process birth and
    /// reservation; neither a bad tail nor a short budget publishes any new
    /// page. The caller must check every new native pointer map before
    /// publishing the token with `commit_prepared_private_vm`. Already
    /// committed pages retain their pins without another budget charge.
    pub fn prepare_fixed_private_span(
        &self,
        owner: ProcessBirthKey,
        address: u32,
        len: u32,
    ) -> Option<PreparedPrivateVm> {
        if len == 0 {
            return None;
        }
        let span_end = address.checked_add(len)?;
        let page_size = STABLE_PRIVATE_PAGE_SIZE as u32;
        let first_page = align_down(address, page_size);
        let last_page = align_down(span_end.checked_sub(1)?, page_size);
        let original = self
            .virtual_allocations_for(owner)
            .find(|allocation| {
                allocation.backing == VirtualAllocationBacking::StablePrivateSlab
                    && allocation.source == VirtualAllocationSource::ThreadStack
                    && address >= allocation.base
                    && allocation
                        .base
                        .checked_add(allocation.size)
                        .is_some_and(|end| span_end <= end)
            })?
            .clone();
        let mut missing_count = 0usize;
        let mut page = first_page;
        loop {
            if original.private_committed_pages.contains_key(&page) {
                // A committed page without its canonical pin cannot be
                // silently replaced by a second byte authority.
                self.private_page_pin(owner, page)?;
            } else {
                missing_count = missing_count.checked_add(1)?;
            }
            if page == last_page {
                break;
            }
            page = page.checked_add(page_size)?;
        }
        if !self.private_page_budget.can_fit(missing_count) {
            return None;
        }
        let mut new_page_pins = Vec::new();
        new_page_pins.try_reserve_exact(missing_count).ok()?;
        let mut local_blocks = BTreeMap::new();
        let mut proposed = original.clone();
        let mut page = first_page;
        loop {
            if !original.private_committed_pages.contains_key(&page) {
                let identity = PrivatePageIdentity {
                    owner,
                    allocation_base: original.base,
                    allocation_incarnation: original.incarnation,
                    page_base: page,
                };
                new_page_pins.push(StablePrivatePagePin {
                    identity,
                    page: self.new_fixed_private_page(identity, &mut local_blocks)?,
                });
                proposed
                    .private_committed_pages
                    .insert(page, original.protect);
            }
            if page == last_page {
                break;
            }
            page = page.checked_add(page_size)?;
        }
        if !proposed.private_committed_pages.is_empty() {
            proposed.allocation_type |= MEM_COMMIT;
        }
        Some(PreparedPrivateVm {
            instance: Rc::clone(&self.instance),
            previous_owner_cursor: None,
            owner,
            result_ptr: first_page,
            proposed_allocation: proposed,
            original_allocation: Some(original),
            expected_virtual_generation: self.virtual_generation,
            new_page_pins,
        })
    }

    /// Prepare only the faulting 4-KiB page of a fixed stack reservation.
    /// A previously committed page produces no new pin or budget charge.
    pub fn prepare_fixed_private_page(
        &self,
        owner: ProcessBirthKey,
        fault_address: u32,
    ) -> Option<PreparedPrivateVm> {
        self.prepare_fixed_private_span(owner, fault_address, 1)
    }

    /// Locate the owner-qualified process heap reservation. Callers can use
    /// this to distinguish a compatibility owner without a fixed heap from
    /// a fixed heap whose growth cannot fit the guest RAM budget.
    pub fn fixed_heap_reservation(&self, owner: ProcessBirthKey) -> Option<(u32, u32)> {
        self.virtual_allocations_for(owner)
            .find(|allocation| {
                allocation.source == VirtualAllocationSource::ProcessHeapArena
                    && allocation.backing == VirtualAllocationBacking::StablePrivateSlab
            })
            .map(|allocation| (allocation.base, allocation.size))
    }

    /// End of the already committed contiguous heap prefix. This read-only
    /// check lets ordinary sub-page HeapAlloc calls avoid cloning thousands
    /// of committed-page entries just to discover there is no new page.
    pub fn fixed_heap_committed_prefix_end(&self, owner: ProcessBirthKey) -> Option<u32> {
        let (base, size) = self.fixed_heap_reservation(owner)?;
        let reservation_end = base.checked_add(size)?;
        let allocation = self.virtual_allocation(owner, base)?;
        match allocation.private_committed_pages.last_key_value() {
            Some((&last, _)) => {
                let offset = last.checked_sub(base)?;
                if offset % STABLE_PRIVATE_PAGE_SIZE as u32 != 0
                    || last >= reservation_end
                    || allocation.private_committed_pages.len()
                        != usize::try_from(offset / STABLE_PRIVATE_PAGE_SIZE as u32 + 1).ok()?
                {
                    return None;
                }
                last.checked_add(STABLE_PRIVATE_PAGE_SIZE as u32)
            }
            None => Some(base),
        }
    }

    /// Prepare every newly crossed process-heap page through `high_water_end`
    /// in one budget-checked token. HeapAlloc must map and commit this token
    /// before publishing its pointer; heap faults never allocate pages.
    pub fn prepare_fixed_heap_growth(
        &self,
        owner: ProcessBirthKey,
        high_water_end: u32,
    ) -> Option<PreparedPrivateVm> {
        let (base, size) = self.fixed_heap_reservation(owner)?;
        let original = self.virtual_allocation(owner, base)?;
        let reservation_end = base.checked_add(size)?;
        if high_water_end <= base {
            return None;
        }
        let commit_end = align_up(
            high_water_end.min(reservation_end),
            STABLE_PRIVATE_PAGE_SIZE as u32,
        )?;
        let first_missing = self.fixed_heap_committed_prefix_end(owner)?;
        let new_count = usize::try_from(
            commit_end.saturating_sub(first_missing) / STABLE_PRIVATE_PAGE_SIZE as u32,
        )
        .ok()?;
        if !self.private_page_budget.can_fit(new_count) {
            return None;
        }
        let mut new_page_pins = Vec::new();
        new_page_pins.try_reserve_exact(new_count).ok()?;
        let mut local_blocks = BTreeMap::new();
        let original = original.clone();
        let mut proposed = original.clone();
        let mut page_base = first_missing;
        while page_base < commit_end {
            let identity = PrivatePageIdentity {
                owner,
                allocation_base: original.base,
                allocation_incarnation: original.incarnation,
                page_base,
            };
            new_page_pins.push(StablePrivatePagePin {
                identity,
                page: self.new_fixed_private_page(identity, &mut local_blocks)?,
            });
            proposed
                .private_committed_pages
                .insert(page_base, original.protect);
            page_base = page_base.checked_add(STABLE_PRIVATE_PAGE_SIZE as u32)?;
        }
        if !proposed.private_committed_pages.is_empty() {
            proposed.allocation_type |= MEM_COMMIT;
        }
        Some(PreparedPrivateVm {
            instance: Rc::clone(&self.instance),
            previous_owner_cursor: None,
            owner,
            result_ptr: commit_end,
            proposed_allocation: proposed,
            original_allocation: Some(original),
            expected_virtual_generation: self.virtual_generation,
            new_page_pins,
        })
    }

    /// Retire this birth's fixed heap/stack VM records at process exit. A
    /// still-live native or quarantine pin keeps its page bytes and budget
    /// charge alive until checked native owner close releases that pin;
    /// unrelated private VM and other process births are left untouched.
    pub fn retire_fixed_private_ranges_for_owner(&mut self, owner: ProcessBirthKey) -> usize {
        let records = self
            .virtual_allocations_for(owner)
            .filter(|allocation| allocation.source.is_native_fixed())
            .map(|allocation| (allocation.base, allocation.incarnation))
            .collect::<Vec<_>>();
        for (base, incarnation) in &records {
            self.virtual_allocations.remove(&(owner, *base));
            self.stable_private_pages.retain(|identity, _| {
                identity.owner != owner
                    || identity.allocation_base != *base
                    || identity.allocation_incarnation != *incarnation
            });
        }
        if !records.is_empty() {
            self.bump_virtual_generation();
        }
        records.len()
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
            || !valid_private_protect(protect)
        {
            return None;
        }

        if allocation_type == MEM_COMMIT {
            if address == 0 {
                return None;
            }
            let page_start = align_down(address, STABLE_PRIVATE_PAGE_SIZE as u32);
            let end = address.checked_add(size)?;
            let page_end = align_up(end, STABLE_PRIVATE_PAGE_SIZE as u32)?;
            let original = self
                .virtual_allocations_for(owner)
                .find(|allocation| {
                    allocation.backing == VirtualAllocationBacking::StablePrivateSlab
                        && allocation.source.is_private()
                        && page_start >= allocation.base
                        && allocation
                            .base
                            .checked_add(allocation.size)
                            .is_some_and(|reservation_end| page_end <= reservation_end)
                })?
                .clone();
            let mut proposed = original.clone();
            let mut page = page_start;
            let mut new_count = 0usize;
            while page < page_end {
                if !proposed.private_committed_pages.contains_key(&page) {
                    new_count = new_count.checked_add(1)?;
                }
                page = page.checked_add(STABLE_PRIVATE_PAGE_SIZE as u32)?;
            }
            if !self.private_page_budget.can_fit(new_count) {
                return None;
            }
            let mut new_page_pins = Vec::new();
            new_page_pins.try_reserve_exact(new_count).ok()?;
            let mut page = page_start;
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
                        page: Rc::new(StablePrivatePage::zeroed(&self.private_page_budget)?),
                    });
                }
                page = page.checked_add(STABLE_PRIVATE_PAGE_SIZE as u32)?;
            }
            proposed.allocation_type |= MEM_COMMIT;
            return Some(PreparedPrivateVm {
                instance: Rc::clone(&self.instance),
                previous_owner_cursor: None,
                owner,
                result_ptr: page_start,
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
        // CE6 VMAlloc counts pages from the caller's page-aligned address,
        // while DoVMReserve rounds a specified reservation base down to 64
        // KiB and DoVMAlloc commits from that returned base (vm.c:2433-2459,
        // 676-690, 883-889). Do not return an uncommitted rounded base.
        let committed_len = if address == 0 {
            align_up(size, STABLE_PRIVATE_PAGE_SIZE as u32)?
        } else {
            let request_end = address.checked_add(size)?;
            let requested_page_base = align_down(address, STABLE_PRIVATE_PAGE_SIZE as u32);
            align_up(
                request_end.checked_sub(requested_page_base)?,
                STABLE_PRIVATE_PAGE_SIZE as u32,
            )?
        };
        let reservation_size = align_up(committed_len, VIRTUAL_ALIGN)?;
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
        let (commit_start, commit_end) = if allocation_type & MEM_COMMIT != 0 {
            let start = base;
            let end = base.checked_add(committed_len)?;
            if end > reservation_end {
                return None;
            }
            (start, end)
        } else {
            (base, base)
        };
        let new_count = usize::try_from(
            commit_end.checked_sub(commit_start)? / STABLE_PRIVATE_PAGE_SIZE as u32,
        )
        .ok()?;
        if !self.private_page_budget.can_fit(new_count) {
            return None;
        }
        new_page_pins.try_reserve_exact(new_count).ok()?;
        if allocation_type & MEM_COMMIT != 0 {
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
                    page: Rc::new(StablePrivatePage::zeroed(&self.private_page_budget)?),
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
        // Clone carries neither this pending hole nor the detached token.
        let previous_owner_cursor =
            (address == 0).then(|| self.next_virtual_ptr_by_owner.get(&owner).copied());
        self.pending_private_holes
            .insert((owner, base), reservation_size);
        self.next_virtual_incarnation = next_incarnation;
        if address == 0 {
            self.next_virtual_ptr_by_owner
                .insert(owner, reservation_end);
        }
        Some(PreparedPrivateVm {
            instance: Rc::clone(&self.instance),
            previous_owner_cursor,
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
            instance,
            previous_owner_cursor: _,
            owner,
            result_ptr,
            proposed_allocation,
            original_allocation,
            expected_virtual_generation,
            new_page_pins,
        } = prepared;
        if !Rc::ptr_eq(&self.instance, &instance)
            || self.virtual_generation != expected_virtual_generation
            || proposed_allocation.owner != owner
            || proposed_allocation.backing != VirtualAllocationBacking::StablePrivateSlab
            || !(proposed_allocation.source.is_private()
                || proposed_allocation.source.is_native_fixed())
            || (proposed_allocation.source.is_native_fixed() && original_allocation.is_none())
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
        if proposed_allocation.source == VirtualAllocationSource::ThreadStack {
            let reservation_end = proposed_allocation
                .base
                .checked_add(proposed_allocation.size)?;
            if proposed_allocation
                .private_committed_pages
                .iter()
                .any(|(&page, &protect)| {
                    page < proposed_allocation.base
                        || page >= reservation_end
                        || page & (STABLE_PRIVATE_PAGE_SIZE as u32 - 1) != 0
                        || protect != proposed_allocation.protect
                })
            {
                return None;
            }
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

    /// Cancel a proposal that is proven never to have been offered to native
    /// mapping or protection. An ambiguous FFI attempt must *not* call this:
    /// dropping its token retains the owner-qualified VA hole and the native
    /// lease must quarantine any mapped pin until checked unmap/close.
    pub fn abort_prepared_private_vm(&mut self, prepared: PreparedPrivateVm) -> bool {
        let PreparedPrivateVm {
            instance,
            previous_owner_cursor,
            owner,
            proposed_allocation,
            original_allocation,
            expected_virtual_generation,
            new_page_pins: _,
            result_ptr: _,
        } = prepared;
        if !Rc::ptr_eq(&self.instance, &instance)
            || self.virtual_generation != expected_virtual_generation
            || proposed_allocation.owner != owner
            || proposed_allocation.backing != VirtualAllocationBacking::StablePrivateSlab
        {
            return false;
        }
        let key = (owner, proposed_allocation.base);
        if let Some(original) = original_allocation {
            return previous_owner_cursor.is_none()
                && self.virtual_allocations.get(&key) == Some(&original);
        }
        if self.pending_private_holes.get(&key) != Some(&proposed_allocation.size)
            || self.virtual_allocations.contains_key(&key)
        {
            return false;
        }
        self.pending_private_holes.remove(&key);
        if let Some(previous_cursor) = previous_owner_cursor
            && proposed_allocation
                .base
                .checked_add(proposed_allocation.size)
                == self.next_virtual_ptr_by_owner.get(&owner).copied()
        {
            match previous_cursor {
                Some(cursor) => {
                    self.next_virtual_ptr_by_owner.insert(owner, cursor);
                }
                None => {
                    self.next_virtual_ptr_by_owner.remove(&owner);
                }
            }
        }
        true
    }

    /// Prepare release or decommit of a slab-backed private reservation.
    /// Existing metadata and pins remain live until checked native unmaps
    /// finish and `commit_prepared_private_vm_change` accepts this token.
    pub fn prepare_private_vm_free(
        &self,
        owner: ProcessBirthKey,
        address: u32,
        size: u32,
        free_type: u32,
    ) -> Option<PreparedPrivateVmChange> {
        match free_type {
            MEM_RELEASE if size == 0 => {
                let original = self.virtual_allocation(owner, address)?.clone();
                if original.backing != VirtualAllocationBacking::StablePrivateSlab
                    || !original.source.is_private()
                {
                    return None;
                }
                let mut native_page_changes = Vec::new();
                native_page_changes
                    .try_reserve_exact(original.private_committed_pages.len())
                    .ok()?;
                for page in original.private_committed_pages.keys() {
                    native_page_changes.push(PrivateVmNativePageChange::Unmap(
                        self.private_page_pin(owner, *page)?,
                    ));
                }
                Some(PreparedPrivateVmChange {
                    instance: Rc::clone(&self.instance),
                    owner,
                    original_allocation: original,
                    proposed_allocation: None,
                    expected_virtual_generation: self.virtual_generation,
                    native_page_changes,
                    old_protect: None,
                })
            }
            MEM_DECOMMIT if size != 0 => {
                let range_start = align_down(address, STABLE_PRIVATE_PAGE_SIZE as u32);
                let range_end =
                    align_up(address.checked_add(size)?, STABLE_PRIVATE_PAGE_SIZE as u32)?;
                let original =
                    self.private_vm_allocation_covering(owner, range_start, range_end)?;
                let mut proposed = original.clone();
                let pages = original
                    .private_committed_pages
                    .range(range_start..range_end)
                    .map(|(page, _)| *page)
                    .collect::<Vec<_>>();
                let mut native_page_changes = Vec::new();
                native_page_changes.try_reserve_exact(pages.len()).ok()?;
                for page in pages {
                    native_page_changes.push(PrivateVmNativePageChange::Unmap(
                        self.private_page_pin(owner, page)?,
                    ));
                    proposed.private_committed_pages.remove(&page);
                }
                if proposed.private_committed_pages.is_empty() {
                    proposed.allocation_type &= !MEM_COMMIT;
                }
                Some(PreparedPrivateVmChange {
                    instance: Rc::clone(&self.instance),
                    owner,
                    original_allocation: original,
                    proposed_allocation: Some(proposed),
                    expected_virtual_generation: self.virtual_generation,
                    native_page_changes,
                    old_protect: None,
                })
            }
            _ => None,
        }
    }

    /// Prepare protection of only committed pages in one exact reservation.
    /// The old protection is the first queried page's value, matching CE6's
    /// VMProtect return convention even when later pages previously differed.
    pub fn prepare_private_vm_protect(
        &self,
        owner: ProcessBirthKey,
        address: u32,
        size: u32,
        new_protect: u32,
    ) -> Option<PreparedPrivateVmChange> {
        if size == 0 || !valid_private_protect(new_protect) {
            return None;
        }
        let range_start = align_down(address, STABLE_PRIVATE_PAGE_SIZE as u32);
        let range_end = align_up(address.checked_add(size)?, STABLE_PRIVATE_PAGE_SIZE as u32)?;
        let original = self.private_vm_allocation_covering(owner, range_start, range_end)?;
        let old_protect = original
            .private_committed_pages
            .get(&range_start)
            .copied()?;
        let mut proposed = original.clone();
        let page_count =
            usize::try_from(range_end.checked_sub(range_start)? / STABLE_PRIVATE_PAGE_SIZE as u32)
                .ok()?;
        let mut native_page_changes = Vec::new();
        native_page_changes.try_reserve_exact(page_count).ok()?;
        let mut page = range_start;
        while page < range_end {
            let current = proposed.private_committed_pages.get_mut(&page)?;
            *current = new_protect;
            native_page_changes.push(PrivateVmNativePageChange::Protect {
                pin: self.private_page_pin(owner, page)?,
                new_protect,
            });
            page = page.checked_add(STABLE_PRIVATE_PAGE_SIZE as u32)?;
        }
        Some(PreparedPrivateVmChange {
            instance: Rc::clone(&self.instance),
            owner,
            original_allocation: original,
            proposed_allocation: Some(proposed),
            expected_virtual_generation: self.virtual_generation,
            native_page_changes,
            old_protect: Some(old_protect),
        })
    }

    /// Publish a private change only after its native page effects were
    /// checked. A stale or cross-clone token cannot retire canonical pins.
    pub fn commit_prepared_private_vm_change(
        &mut self,
        prepared: PreparedPrivateVmChange,
    ) -> Option<PrivateVmChangeResult> {
        let PreparedPrivateVmChange {
            instance,
            owner,
            original_allocation,
            proposed_allocation,
            expected_virtual_generation,
            native_page_changes,
            old_protect,
        } = prepared;
        let key = (owner, original_allocation.base);
        if !Rc::ptr_eq(&self.instance, &instance)
            || self.virtual_generation != expected_virtual_generation
            || self.virtual_allocations.get(&key) != Some(&original_allocation)
            || original_allocation.owner != owner
            || original_allocation.backing != VirtualAllocationBacking::StablePrivateSlab
            || !original_allocation.source.is_private()
            || native_page_changes
                .iter()
                .any(|change| !self.contains_private_page_pin(change.pin()))
        {
            return None;
        }
        if let Some(proposed) = &proposed_allocation
            && (proposed.owner != owner
                || proposed.base != original_allocation.base
                || proposed.size != original_allocation.size
                || proposed.incarnation != original_allocation.incarnation
                || proposed.source != original_allocation.source
                || proposed.backing != original_allocation.backing)
        {
            return None;
        }
        let result = match old_protect {
            Some(old_protect) => PrivateVmChangeResult::Protect { old_protect },
            None => PrivateVmChangeResult::Free,
        };
        match proposed_allocation {
            Some(proposed) => {
                for page in original_allocation.private_committed_pages.keys() {
                    if !proposed.private_committed_pages.contains_key(page) {
                        self.stable_private_pages.remove(&PrivatePageIdentity {
                            owner,
                            allocation_base: original_allocation.base,
                            allocation_incarnation: original_allocation.incarnation,
                            page_base: *page,
                        });
                    }
                }
                self.virtual_allocations.insert(key, proposed);
            }
            None => {
                self.virtual_allocations.remove(&key);
                self.stable_private_pages.retain(|identity, _| {
                    identity.owner != owner
                        || identity.allocation_base != original_allocation.base
                        || identity.allocation_incarnation != original_allocation.incarnation
                });
            }
        }
        self.bump_virtual_generation();
        Some(result)
    }

    fn private_vm_allocation_covering(
        &self,
        owner: ProcessBirthKey,
        range_start: u32,
        range_end: u32,
    ) -> Option<VirtualAllocation> {
        if range_end <= range_start {
            return None;
        }
        self.virtual_allocations_for(owner)
            .find(|allocation| {
                allocation.backing == VirtualAllocationBacking::StablePrivateSlab
                    && allocation.source.is_private()
                    && range_start >= allocation.base
                    && allocation
                        .base
                        .checked_add(allocation.size)
                        .is_some_and(|end| range_end <= end)
            })
            .cloned()
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
        if source.is_native_fixed() {
            return None;
        }
        self.virtual_alloc_legacy_seed(owner, source, address, size, allocation_type, protect)
    }

    fn virtual_alloc_legacy_seed(
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
            let mut base = self
                .next_virtual_ptr_by_owner
                .get(&owner)
                .copied()
                .unwrap_or(VIRTUAL_PTR_BASE)
                .max(self.next_virtual_ptr);
            // Compatibility views still use a VA-only native mapper. A
            // private slab may occupy this cursor without advancing the old
            // global high-water mark, so search subsequent aligned slots.
            let next_cursor = loop {
                let end = base.checked_add(size)?;
                if !self.virtual_overlaps(base, size) {
                    break end;
                }
                base = base.checked_add(VIRTUAL_ALIGN)?;
            };
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
                backing: VirtualAllocationBacking::LegacySeed,
                private_committed_pages: BTreeMap::new(),
            },
        );
        self.bump_virtual_generation();
        Some(base)
    }

    /// Publish a public file view backed by one canonical page per mapping
    /// offset. Aliases in this MemorySystem lineage share the same page Rc;
    /// independent process births may choose the same guest VA.
    pub fn allocate_shared_view(
        &mut self,
        owner: ProcessBirthKey,
        mapping_handle: u32,
        offset: u32,
        size: u32,
        protect: u32,
        initial_bytes: &[u8],
        legacy_bytes: Option<&[u8]>,
        legacy_underlay: Option<&[u8]>,
    ) -> Option<u32> {
        if size == 0 || offset & (STABLE_PRIVATE_PAGE_SIZE as u32 - 1) != 0 {
            return None;
        }
        let page_size = STABLE_PRIVATE_PAGE_SIZE as u32;
        let mapped_size = align_up(size, page_size)?;
        let reservation_size = align_up(mapped_size, VIRTUAL_ALIGN)?;
        let mut base = self
            .next_virtual_ptr_by_owner
            .get(&owner)
            .copied()
            .unwrap_or(VIRTUAL_PTR_BASE)
            .max(VIRTUAL_PTR_BASE);
        while self.owner_scoped_slab_overlaps(owner, base, reservation_size) {
            base = base.checked_add(VIRTUAL_ALIGN)?;
        }
        let next_cursor = base.checked_add(reservation_size)?;
        let incarnation = self.next_virtual_incarnation;
        let next_incarnation = incarnation.checked_add(1)?;
        // The first public view retires the complete legacy byte store, not
        // merely the range selected by this view. Keep this preflight local so
        // an OOM leaves the Legacy object and all VM metadata untouched.
        let legacy_size = match legacy_bytes {
            Some(bytes) => Some(align_up(u32::try_from(bytes.len()).ok()?, page_size)?),
            None => None,
        };
        let mut requested_pages = BTreeSet::new();
        if let Some(legacy_size) = legacy_size {
            let mut page_offset = 0;
            while page_offset < legacy_size {
                requested_pages.insert(page_offset);
                page_offset = page_offset.checked_add(page_size)?;
            }
        }
        let mut delta = 0;
        while delta < mapped_size {
            let page_offset = offset.checked_add(delta)?;
            requested_pages.insert(page_offset);
            delta = delta.checked_add(page_size)?;
        }
        let missing = requested_pages
            .into_iter()
            .filter(|page_offset| {
                !self
                    .stable_shared_pages
                    .contains_key(&(mapping_handle, *page_offset))
            })
            .collect::<Vec<_>>();
        if !self.private_page_budget.can_fit(missing.len()) {
            return None;
        }
        let mut new_pages = Vec::new();
        new_pages.try_reserve_exact(missing.len()).ok()?;
        for page_offset in missing {
            let page = Rc::new(StablePrivatePage::zeroed_shared(&self.private_page_budget)?);
            let source = legacy_underlay
                .filter(|bytes| (page_offset as usize) < bytes.len())
                .map(|bytes| (bytes, page_offset as usize))
                .or_else(|| {
                    (page_offset >= offset && page_offset < offset.saturating_add(mapped_size))
                        .then(|| (initial_bytes, (page_offset - offset) as usize))
                });
            let (seed, start) = source.unwrap_or((&[][..], 0));
            let end = start
                .saturating_add(STABLE_PRIVATE_PAGE_SIZE)
                .min(seed.len());
            let pin = StablePrivatePagePin {
                identity: PrivatePageIdentity {
                    owner,
                    allocation_base: base,
                    allocation_incarnation: incarnation,
                    page_base: base,
                },
                page: Rc::clone(&page),
            };
            if start < end {
                pin.write_bytes(0, &seed[start..end]).ok()?;
            }
            if let Some(legacy) = legacy_bytes {
                let start = page_offset as usize;
                let end = start
                    .saturating_add(STABLE_PRIVATE_PAGE_SIZE)
                    .min(legacy.len());
                if start < end {
                    pin.write_bytes(0, &legacy[start..end]).ok()?;
                }
            }
            new_pages.push(((mapping_handle, page_offset), page));
        }
        let mut committed_pages = BTreeMap::new();
        let mut delta = 0;
        while delta < mapped_size {
            committed_pages.insert(base.checked_add(delta)?, protect);
            delta = delta.checked_add(page_size)?;
        }
        for (key, page) in new_pages {
            self.stable_shared_pages.insert(key, page);
        }
        self.virtual_allocations.insert(
            (owner, base),
            VirtualAllocation {
                base,
                size: reservation_size,
                allocation_type: MEM_RESERVE | MEM_COMMIT,
                protect,
                initial_bytes: Vec::new(),
                owner,
                incarnation,
                source: VirtualAllocationSource::FileViewSlab,
                backing: VirtualAllocationBacking::StableSharedViewSlab,
                private_committed_pages: committed_pages,
            },
        );
        self.shared_view_bindings.insert(
            (owner, base),
            SharedViewBinding {
                mapping_handle,
                offset,
                size,
            },
        );
        self.next_virtual_ptr_by_owner.insert(owner, next_cursor);
        self.next_virtual_incarnation = next_incarnation;
        self.stable_private_slabs_seen = true;
        self.bump_virtual_generation();
        Some(base)
    }

    pub fn release_shared_view(&mut self, owner: ProcessBirthKey, base: u32) -> bool {
        if !self
            .virtual_allocations
            .get(&(owner, base))
            .is_some_and(|allocation| {
                allocation.backing == VirtualAllocationBacking::StableSharedViewSlab
            })
        {
            return false;
        }
        self.virtual_allocations.remove(&(owner, base));
        self.shared_view_bindings.remove(&(owner, base));
        self.bump_virtual_generation();
        true
    }

    pub fn forget_shared_mapping(&mut self, mapping_handle: u32) {
        if self
            .shared_view_bindings
            .values()
            .any(|binding| binding.mapping_handle == mapping_handle)
        {
            return;
        }
        self.stable_shared_pages
            .retain(|(handle, _), _| *handle != mapping_handle);
    }

    pub fn shared_mapping_read(&self, handle: u32, offset: u32, out: &mut [u8]) -> bool {
        self.shared_mapping_transfer(handle, offset, out, false)
    }

    pub fn has_shared_mapping(&self, handle: u32) -> bool {
        self.stable_shared_pages
            .keys()
            .any(|(mapped, _)| *mapped == handle)
    }

    pub fn shared_mapping_write(&self, handle: u32, offset: u32, bytes: &[u8]) -> bool {
        let mut bytes = bytes.to_vec();
        self.shared_mapping_transfer(handle, offset, &mut bytes, true)
    }

    fn shared_mapping_transfer(
        &self,
        handle: u32,
        offset: u32,
        bytes: &mut [u8],
        write: bool,
    ) -> bool {
        let Some(end) = (offset as usize).checked_add(bytes.len()) else {
            return false;
        };
        if end > u32::MAX as usize + 1 {
            return false;
        }
        let mut check = offset as usize;
        while check < end {
            let page_offset = (check & !(STABLE_PRIVATE_PAGE_SIZE - 1)) as u32;
            if !self
                .stable_shared_pages
                .get(&(handle, page_offset))
                .is_some_and(|page| page.native_execution_guards.get() == 0)
            {
                return false;
            }
            check =
                (check & !(STABLE_PRIVATE_PAGE_SIZE - 1)).saturating_add(STABLE_PRIVATE_PAGE_SIZE);
        }
        let mut copied = 0;
        while copied < bytes.len() {
            let Some(current) = usize::try_from(offset)
                .ok()
                .and_then(|offset| offset.checked_add(copied))
            else {
                return false;
            };
            let page_offset = (current & !(STABLE_PRIVATE_PAGE_SIZE - 1)) as u32;
            let in_page = current & (STABLE_PRIVATE_PAGE_SIZE - 1);
            let count = (bytes.len() - copied).min(STABLE_PRIVATE_PAGE_SIZE - in_page);
            let Some(page) = self.stable_shared_pages.get(&(handle, page_offset)) else {
                return false;
            };
            if page.native_execution_guards.get() != 0 {
                return false;
            }
            // SAFETY: a guard above excludes native execution; this bounded
            // copy ends before returning and the Rc holds the fixed page.
            unsafe {
                if write {
                    std::ptr::copy_nonoverlapping(
                        bytes.as_ptr().add(copied),
                        page.bytes.as_ptr().add(in_page),
                        count,
                    );
                } else {
                    std::ptr::copy_nonoverlapping(
                        page.bytes.as_ptr().add(in_page),
                        bytes.as_mut_ptr().add(copied),
                        count,
                    );
                }
            }
            copied += count;
        }
        true
    }

    pub fn shared_mapping_native_active(&self, handle: u32, offset: u32, len: usize) -> bool {
        let Some(end) = (offset as usize).checked_add(len) else {
            return false;
        };
        let mut current = offset as usize;
        while current < end {
            let page_offset = (current & !(STABLE_PRIVATE_PAGE_SIZE - 1)) as u32;
            if !self
                .stable_shared_pages
                .get(&(handle, page_offset))
                .is_some_and(|page| page.native_execution_guards.get() != 0)
            {
                return false;
            }
            current = (current & !(STABLE_PRIVATE_PAGE_SIZE - 1))
                .saturating_add(STABLE_PRIVATE_PAGE_SIZE);
        }
        true
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
        if allocation.backing != VirtualAllocationBacking::LegacySeed {
            return false;
        }
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
        // Reads of legacy regions remain globally visible, but releasing or
        // decommitting any region requires its exact birth-key owner.
        let key = (owner, address);
        match free_type {
            MEM_RELEASE => {
                if size != 0 {
                    return false;
                }
                // Slabs must retire native mappings before their metadata or
                // canonical pins can be removed. Only the detached private
                // transaction above may release them.
                if !self
                    .virtual_allocations
                    .get(&key)
                    .is_some_and(|allocation| {
                        allocation.backing == VirtualAllocationBacking::LegacySeed
                    })
                {
                    return false;
                }
                self.virtual_allocations.remove(&key);
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

    pub fn allocation(&self, owner: ProcessBirthKey, ptr: u32) -> Option<&MemoryAllocation> {
        self.allocations.get(&(owner, ptr))
    }

    pub fn allocations(&self) -> impl Iterator<Item = &MemoryAllocation> {
        self.allocations.values()
    }

    pub fn allocations_for(
        &self,
        owner: ProcessBirthKey,
    ) -> impl Iterator<Item = &MemoryAllocation> {
        self.allocations
            .range((owner, u32::MIN)..=(owner, u32::MAX))
            .map(|(_, allocation)| allocation)
    }

    pub fn heaps(&self) -> impl Iterator<Item = &Heap> {
        self.heaps.values().filter(|heap| !heap.destroyed)
    }

    pub fn heaps_for(&self, owner: ProcessBirthKey) -> impl Iterator<Item = &Heap> {
        self.heaps().filter(move |heap| {
            !self.retired_heap_owners.contains(&owner)
                && (heap.handle == self.process_heap || heap.owner == Some(owner))
        })
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
        self.virtual_allocations.values().filter(move |allocation| {
            allocation.owner == owner
                || (allocation.backing == VirtualAllocationBacking::LegacySeed
                    && !allocation.source.is_private())
        })
    }

    /// Query the current owner's slab at one address. A reservation can
    /// contain alternating reserved and committed pages with distinct native
    /// permissions; never infer the whole region from `allocation.protect`.
    pub fn private_vm_query(
        &self,
        owner: ProcessBirthKey,
        address: u32,
    ) -> Option<PrivateVmPageQuery> {
        let allocation = self.virtual_allocations_for(owner).find(|allocation| {
            allocation.backing == VirtualAllocationBacking::StablePrivateSlab
                && address >= allocation.base
                && allocation
                    .base
                    .checked_add(allocation.size)
                    .is_some_and(|end| address < end)
        })?;
        let region_base = align_down(address, STABLE_PRIVATE_PAGE_SIZE as u32);
        let page_protect = allocation
            .private_committed_pages
            .get(&region_base)
            .copied();
        let allocation_end = allocation.base.checked_add(allocation.size)?;
        let mut region_end = region_base;
        while region_end < allocation_end
            && allocation.private_committed_pages.get(&region_end).copied() == page_protect
        {
            region_end = region_end.checked_add(STABLE_PRIVATE_PAGE_SIZE as u32)?;
        }
        Some(PrivateVmPageQuery {
            allocation_base: allocation.base,
            allocation_protect: allocation.protect,
            region_base,
            region_size: region_end.checked_sub(region_base)?,
            page_protect,
        })
    }

    pub fn shared_view_query(
        &self,
        owner: ProcessBirthKey,
        address: u32,
    ) -> Option<PrivateVmPageQuery> {
        let allocation = self.virtual_allocations_for(owner).find(|allocation| {
            allocation.backing == VirtualAllocationBacking::StableSharedViewSlab
                && address >= allocation.base
                && allocation
                    .base
                    .checked_add(allocation.size)
                    .is_some_and(|end| address < end)
        })?;
        let region_base = align_down(address, STABLE_PRIVATE_PAGE_SIZE as u32);
        let page_protect = allocation
            .private_committed_pages
            .get(&region_base)
            .copied();
        let allocation_end = allocation.base.checked_add(allocation.size)?;
        let mut region_end = region_base;
        while region_end < allocation_end
            && allocation.private_committed_pages.get(&region_end).copied() == page_protect
        {
            region_end = region_end.checked_add(STABLE_PRIVATE_PAGE_SIZE as u32)?;
        }
        Some(PrivateVmPageQuery {
            allocation_base: allocation.base,
            allocation_protect: allocation.protect,
            region_base,
            region_size: region_end.checked_sub(region_base)?,
            page_protect,
        })
    }

    /// Conservatively reject independent machine checkpointing after any
    /// diagnostic slab existed, including after release while native pins may
    /// still exist outside this VM directory.
    pub fn has_stable_private_slabs(&self) -> bool {
        self.stable_private_slabs_seen
    }

    pub fn stable_page_pin(
        &self,
        owner: ProcessBirthKey,
        page_base: u32,
    ) -> Option<StablePrivatePagePin> {
        self.private_page_pin(owner, page_base).or_else(|| {
            let allocation = self.virtual_allocations_for(owner).find(|allocation| {
                allocation.backing == VirtualAllocationBacking::StableSharedViewSlab
                    && allocation.private_committed_pages.contains_key(&page_base)
            })?;
            let binding = self.shared_view_bindings.get(&(owner, allocation.base))?;
            let delta = page_base.checked_sub(allocation.base)?;
            let page_offset = binding.offset.checked_add(delta)?;
            let page = Rc::clone(
                self.stable_shared_pages
                    .get(&(binding.mapping_handle, page_offset))?,
            );
            Some(StablePrivatePagePin {
                identity: PrivatePageIdentity {
                    owner,
                    allocation_base: allocation.base,
                    allocation_incarnation: allocation.incarnation,
                    page_base,
                },
                page,
            })
        })
    }

    pub fn stable_page_pins_for(&self, owner: ProcessBirthKey) -> Vec<StablePrivatePagePin> {
        let mut pins = self.private_page_pins_for(owner);
        for ((binding_owner, base), binding) in &self.shared_view_bindings {
            if *binding_owner != owner {
                continue;
            }
            let Some(allocation) = self.virtual_allocation(owner, *base) else {
                continue;
            };
            for page_base in allocation.private_committed_pages.keys().copied() {
                let Some(delta) = page_base.checked_sub(*base) else {
                    continue;
                };
                let Some(page_offset) = binding.offset.checked_add(delta) else {
                    continue;
                };
                let Some(page) = self
                    .stable_shared_pages
                    .get(&(binding.mapping_handle, page_offset))
                else {
                    continue;
                };
                pins.push(StablePrivatePagePin {
                    identity: PrivatePageIdentity {
                        owner,
                        allocation_base: *base,
                        allocation_incarnation: allocation.incarnation,
                        page_base,
                    },
                    page: Rc::clone(page),
                });
            }
        }
        pins
    }

    pub fn contains_stable_page_pin(&self, pin: &StablePrivatePagePin) -> bool {
        if self.contains_private_page_pin(pin) {
            return true;
        }
        let identity = pin.identity;
        let Some(allocation) = self.virtual_allocation(identity.owner, identity.allocation_base)
        else {
            return false;
        };
        if allocation.backing != VirtualAllocationBacking::StableSharedViewSlab
            || allocation.incarnation != identity.allocation_incarnation
            || !allocation
                .private_committed_pages
                .contains_key(&identity.page_base)
        {
            return false;
        }
        let Some(binding) = self
            .shared_view_bindings
            .get(&(identity.owner, identity.allocation_base))
        else {
            return false;
        };
        let Some(offset) = identity
            .page_base
            .checked_sub(identity.allocation_base)
            .and_then(|delta| binding.offset.checked_add(delta))
        else {
            return false;
        };
        self.stable_shared_pages
            .get(&(binding.mapping_handle, offset))
            .is_some_and(|page| Rc::ptr_eq(page, &pin.page))
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

    pub fn heap_high_water_mark(&self, owner: ProcessBirthKey) -> u32 {
        self.next_heap_ptr_by_owner
            .get(&owner)
            .copied()
            .unwrap_or(HEAP_PTR_BASE)
    }

    /// Retire only this process birth's heap metadata after its execution
    /// lifetime ends. A stale prepared token cannot republish into this birth.
    pub fn retire_heap_state_for_owner(&mut self, owner: ProcessBirthKey) -> usize {
        let before = self.allocations.len();
        self.allocations
            .retain(|(allocation_owner, _), _| *allocation_owner != owner);
        let removed = before - self.allocations.len();
        let cursor_removed = self.next_heap_ptr_by_owner.remove(&owner).is_some();
        let before_heaps = self.heaps.len();
        self.heaps.retain(|_, heap| heap.owner != Some(owner));
        let heap_removed = before_heaps != self.heaps.len();
        let newly_retired = self.retired_heap_owners.insert(owner);
        if removed != 0 || cursor_removed || heap_removed || newly_retired {
            self.bump_heap_generation();
        }
        removed
    }

    pub fn heap_generation(&self) -> u64 {
        self.heap_generation
    }

    pub fn virtual_generation(&self) -> u64 {
        self.virtual_generation
    }

    /// Set a raw private-page budget with no boot reservation. Low-level
    /// controls use this to exercise exact page counts independently of the
    /// device's reported boot-time memory occupancy.
    #[cfg(test)]
    pub fn set_private_commit_limit_bytes(&mut self, bytes: u64) {
        self.private_page_budget.limit_bytes.set(bytes);
        self.private_page_budget.boot_reserved_bytes.set(0);
    }

    /// Configure guest physical RAM at boot. The device dump reports
    /// 164_368_384 available of 195_350_528 total bytes before private pages;
    /// preserve that ratio for overrides so allocation and status agree.
    pub fn configure_guest_physical_ram_bytes(&mut self, bytes: u64) {
        const DEVICE_BOOT_AVAIL_BYTES: u64 = 164_368_384;
        let available = u64::try_from(
            (u128::from(bytes) * u128::from(DEVICE_BOOT_AVAIL_BYTES))
                / u128::from(DEFAULT_GUEST_PHYSICAL_RAM_BYTES),
        )
        .unwrap_or(u64::MAX);
        self.private_page_budget.limit_bytes.set(bytes);
        self.private_page_budget
            .boot_reserved_bytes
            .set(bytes.saturating_sub(available));
    }

    pub fn private_commit_limit_bytes(&self) -> u64 {
        self.private_page_budget.limit_bytes.get()
    }

    pub fn private_commit_used_bytes(&self) -> u64 {
        self.private_page_budget.used_bytes.get()
    }

    pub fn private_commit_available_bytes(&self) -> u64 {
        self.private_page_budget.available_bytes()
    }

    pub fn contains_allocated_range(&self, owner: ProcessBirthKey, ptr: u32, size: u32) -> bool {
        if size == 0 {
            return true;
        }
        let Some(end) = ptr.checked_add(size) else {
            return false;
        };
        self.allocations_for(owner).any(|allocation| {
            let Some(allocation_end) = allocation.ptr.checked_add(allocation.actual_size) else {
                return false;
            };
            ptr >= allocation.ptr && end <= allocation_end
        }) || self
            .virtual_allocations_visible_to(owner)
            .any(|allocation| allocation_contains_committed_range(allocation, ptr, end))
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
        // A local committed owner-qualified range remains valid even if a
        // foreign process independently owns the same guest VA.
        if self.virtual_allocations_for(owner).any(|allocation| {
            ((allocation.source.is_private() || allocation.source.is_native_fixed())
                || allocation.backing == VirtualAllocationBacking::StableSharedViewSlab)
                && allocation_contains_committed_range(allocation, ptr, end)
        }) {
            return false;
        }
        self.virtual_allocations.values().any(|allocation| {
            allocation.owner != owner
                && (allocation.source.is_private() || allocation.source.is_native_fixed())
                && allocation
                    .base
                    .checked_add(allocation.size)
                    .is_some_and(|other_end| ptr < other_end && end > allocation.base)
        })
    }

    pub fn heap_range_status(&self, owner: ProcessBirthKey, ptr: u32, size: u32) -> Option<bool> {
        let end = ptr.checked_add(size)?;
        self.allocations_for(owner).find_map(|allocation| {
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

    fn is_live_heap(&self, owner: ProcessBirthKey, heap: u32) -> bool {
        !self.retired_heap_owners.contains(&owner)
            && self.heaps.get(&heap).is_some_and(|item| {
                !item.destroyed && (heap == self.process_heap || item.owner == Some(owner))
            })
    }

    fn allocate_heap_ptr(&mut self, owner: ProcessBirthKey, size: u32) -> Option<u32> {
        let step = align_up(size.max(1), HEAP_ALIGN)?;
        let mut ptr = self.heap_high_water_mark(owner);
        loop {
            let next = ptr.checked_add(step)?;
            if next > HEAP_PTR_LIMIT {
                return None;
            }
            // Only this birth's private VM reservations and globally mapped
            // legacy bytes can conflict with its fixed heap window.
            let collision_end = self
                .virtual_allocations
                .values()
                .filter(|allocation| {
                    allocation.source != VirtualAllocationSource::ProcessHeapArena
                        && (allocation.owner == owner
                            || allocation.backing == VirtualAllocationBacking::LegacySeed)
                        && ranges_overlap(ptr, step, allocation.base, allocation.size)
                })
                .filter_map(|allocation| allocation.base.checked_add(allocation.size))
                .chain(self.pending_private_holes.iter().filter_map(
                    |((hole_owner, base), size)| {
                        (*hole_owner == owner && ranges_overlap(ptr, step, *base, *size))
                            .then(|| base.checked_add(*size))
                            .flatten()
                    },
                ))
                .max();
            if let Some(end) = collision_end {
                ptr = align_up(end, HEAP_ALIGN)?;
                continue;
            }
            self.next_heap_ptr_by_owner.insert(owner, next);
            return Some(ptr);
        }
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
        // Only a foreign owner-qualified slab can safely reuse a VA. Every
        // LegacySeed allocation (including a Private source) still uses the
        // global/VA-only native mapper and remains a collision.
        self.virtual_allocations.values().any(|allocation| {
            (allocation.owner == owner
                || allocation.backing == VirtualAllocationBacking::LegacySeed)
                && ranges_overlap(base, size, allocation.base, allocation.size)
        }) || self
            .pending_private_holes
            .iter()
            .any(|((other_owner, other_base), other_size)| {
                *other_owner == owner && ranges_overlap(base, size, *other_base, *other_size)
            })
            || self
                .heap_high_water_mark(owner)
                .checked_sub(HEAP_PTR_BASE)
                .is_some_and(|consumed| ranges_overlap(base, size, HEAP_PTR_BASE, consumed))
            || self.allocations_for(owner).any(|allocation| {
                ranges_overlap(base, size, allocation.ptr, allocation.actual_size)
            })
    }

    fn owner_scoped_slab_overlaps(&self, owner: ProcessBirthKey, base: u32, size: u32) -> bool {
        self.private_vm_overlaps(owner, base, size)
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

fn valid_private_protect(protect: u32) -> bool {
    matches!(
        protect,
        0x01 | 0x02 | 0x04 | 0x08 | 0x10 | 0x20 | 0x40 | 0x80
    )
}

fn allocation_contains_committed_range(allocation: &VirtualAllocation, ptr: u32, end: u32) -> bool {
    if end <= ptr
        || ptr < allocation.base
        || !allocation
            .base
            .checked_add(allocation.size)
            .is_some_and(|allocation_end| end <= allocation_end)
    {
        return false;
    }
    if allocation.backing == VirtualAllocationBacking::LegacySeed {
        return true;
    }
    let mut page = align_down(ptr, STABLE_PRIVATE_PAGE_SIZE as u32);
    let last_page = align_down(end - 1, STABLE_PRIVATE_PAGE_SIZE as u32);
    loop {
        if !allocation
            .private_committed_pages
            .get(&page)
            .is_some_and(|protect| *protect != 0x01)
        {
            return false;
        }
        if page == last_page {
            return true;
        }
        let Some(next) = page.checked_add(STABLE_PRIVATE_PAGE_SIZE as u32) else {
            return false;
        };
        page = next;
    }
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

    fn two_owners() -> (ProcessBirthKey, ProcessBirthKey) {
        let mut handles = HandleTable::default();
        handles.register_boot_process(1);
        let parent = handles.process_birth_key(1).expect("boot birth");
        let _child_handle = handles.create_process(2);
        let child = handles.process_birth_key(2).expect("child birth");
        (parent, child)
    }

    fn committed_private_slab(
        memory: &mut MemorySystem,
        owner: ProcessBirthKey,
        address: u32,
        size: u32,
    ) -> u32 {
        let prepared = memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::Private,
                address,
                size,
                MEM_RESERVE | MEM_COMMIT,
                4,
            )
            .expect("prepare private slab");
        memory
            .commit_prepared_private_vm(prepared)
            .expect("publish private slab")
    }

    #[test]
    fn prepared_allocation_is_not_live_until_commit_and_abort_keeps_hole() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let generation = memory.heap_generation();
        let prepared = memory.prepare_local_alloc(owner, LMEM_ZEROINIT, 0).unwrap();
        let ptr = prepared.allocation().ptr;
        assert_eq!(prepared.allocation().actual_size, 1);
        assert_eq!(prepared.allocation().requested_size, 0);
        assert!(prepared.allocation().zeroed);
        assert!(memory.allocation(owner, ptr).is_none());
        assert_eq!(memory.heap_generation(), generation);
        memory.abort_allocation(prepared);
        assert_eq!(memory.heap_high_water_mark(owner), ptr + HEAP_ALIGN);
        let prepared = memory
            .prepare_heap_alloc(owner, PROCESS_HEAP_HANDLE, 0, 9)
            .unwrap();
        let next = prepared.allocation().ptr;
        assert_eq!(next, ptr + HEAP_ALIGN);
        assert_eq!(memory.commit_allocation(prepared), Some(next));
        assert_eq!(
            memory.heap_size(owner, PROCESS_HEAP_HANDLE, 0, next),
            Some(9)
        );
        assert_eq!(memory.heap_generation(), generation + 1);
    }

    #[test]
    fn prepared_allocation_commit_rejects_destroyed_heap_without_reuse() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let heap = memory.heap_create(owner, 0, 0, 0).unwrap();
        let prepared = memory
            .prepare_heap_alloc(owner, heap, HEAP_ZERO_MEMORY, 32)
            .unwrap();
        let ptr = prepared.allocation().ptr;
        assert!(memory.heap_destroy(owner, heap));
        assert!(memory.commit_allocation(prepared).is_none());
        assert!(memory.allocation(owner, ptr).is_none());
        assert_eq!(
            memory.heap_alloc(owner, PROCESS_HEAP_HANDLE, 0, 1),
            Some(ptr + 32)
        );
    }

    #[test]
    fn local_alloc_size_and_free_follow_ce_return_shape() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let ptr = memory.local_alloc(owner, LMEM_ZEROINIT, 0).unwrap();

        assert!(ptr >= POINTER_FLOOR);
        assert_eq!(memory.local_size(owner, ptr), Some(1));
        assert!(memory.allocation(owner, ptr).unwrap().zeroed);
        assert!(memory.local_free(owner, ptr));
        assert_eq!(memory.local_size(owner, ptr), None);
        assert!(memory.local_free(owner, 0));
    }

    #[test]
    fn heap_alloc_requires_live_heap_and_valid_flags() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let heap = memory
            .heap_create(owner, HEAP_NO_SERIALIZE, 0x1000, 0)
            .unwrap();
        let ptr = memory
            .heap_alloc(owner, heap, HEAP_ZERO_MEMORY, 32)
            .unwrap();

        assert_eq!(memory.heap_size(owner, heap, 0, ptr), Some(32));
        assert!(memory.heap_validate(owner, heap, 0, ptr));
        assert!(!memory.heap_free(owner, heap, HEAP_GENERATE_EXCEPTIONS, ptr));
        assert!(memory.heap_free(owner, heap, 0, ptr));
        assert!(memory.heap_destroy(owner, heap));
        assert!(memory.heap_alloc(owner, heap, 0, 8).is_none());
    }

    #[test]
    fn realloc_growth_moves_to_avoid_overlapping_later_allocations() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let first = memory
            .heap_alloc(owner, PROCESS_HEAP_HANDLE, 0, 16)
            .unwrap();
        let second = memory
            .heap_alloc(owner, PROCESS_HEAP_HANDLE, 0, 16)
            .unwrap();

        let grown = memory
            .heap_re_alloc_detail(owner, PROCESS_HEAP_HANDLE, 0, first, 64)
            .unwrap();

        assert!(grown.moved);
        assert_ne!(grown.ptr, first);
        assert!(grown.ptr >= second + 16);
        assert_eq!(
            memory.heap_size(owner, PROCESS_HEAP_HANDLE, 0, grown.ptr),
            Some(64)
        );
        assert_eq!(memory.heap_size(owner, PROCESS_HEAP_HANDLE, 0, first), None);
        assert_eq!(
            memory.heap_size(owner, PROCESS_HEAP_HANDLE, 0, second),
            Some(16)
        );
    }

    #[test]
    fn aborted_moved_reallocation_retains_its_reserved_address_hole() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let source = memory.heap_alloc(owner, PROCESS_HEAP_HANDLE, 0, 5).unwrap();
        let original = memory.allocation(owner, source).unwrap().clone();
        let prepared = memory
            .prepare_heap_re_alloc(owner, PROCESS_HEAP_HANDLE, HEAP_ZERO_MEMORY, source, 32)
            .unwrap();
        let destination = prepared.result().ptr;
        let reserved_end = memory.heap_high_water_mark(owner);

        memory.abort_reallocation(prepared);

        assert_eq!(memory.allocation(owner, source), Some(&original));
        assert!(memory.allocation(owner, destination).is_none());
        assert_eq!(memory.heap_high_water_mark(owner), reserved_end);
        assert_eq!(
            memory.heap_alloc(owner, PROCESS_HEAP_HANDLE, HEAP_ZERO_MEMORY, 32),
            Some(reserved_end)
        );
    }

    #[test]
    fn conflicting_moved_reallocation_does_not_reuse_reserved_destination() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let source = memory.heap_alloc(owner, PROCESS_HEAP_HANDLE, 0, 5).unwrap();
        let prepared = memory
            .prepare_heap_re_alloc(owner, PROCESS_HEAP_HANDLE, 0, source, 32)
            .unwrap();
        let destination = prepared.result().ptr;
        let reserved_end = memory.heap_high_water_mark(owner);
        assert!(memory.heap_free(owner, PROCESS_HEAP_HANDLE, 0, source));

        assert!(memory.commit_reallocation(prepared).is_none());

        assert!(memory.allocation(owner, source).is_none());
        assert!(memory.allocation(owner, destination).is_none());
        assert_eq!(memory.heap_high_water_mark(owner), reserved_end);
        assert_eq!(
            memory.heap_alloc(owner, PROCESS_HEAP_HANDLE, 0, 32),
            Some(reserved_end)
        );
    }

    #[test]
    fn heap_allocations_at_same_va_are_birth_qualified_and_charge_only_own_growth() {
        let (parent, child) = two_owners();
        let mut memory = MemorySystem::default();
        let arena_size = HEAP_PTR_LIMIT - HEAP_PTR_BASE;
        memory.set_private_commit_limit_bytes(3 * STABLE_PRIVATE_PAGE_SIZE as u64);
        for owner in [parent, child] {
            assert_eq!(
                memory.reserve_fixed_private_range(
                    owner,
                    VirtualAllocationSource::ProcessHeapArena,
                    HEAP_PTR_BASE,
                    arena_size,
                    4,
                ),
                Some(HEAP_PTR_BASE)
            );
        }

        let parent_ptr = memory
            .heap_alloc(parent, PROCESS_HEAP_HANDLE, 0, 0x2000)
            .expect("parent allocation");
        assert_eq!(parent_ptr, HEAP_PTR_BASE);
        let parent_growth = memory
            .prepare_fixed_heap_growth(parent, memory.heap_high_water_mark(parent))
            .expect("parent's two pages fit");
        assert_eq!(parent_growth.new_page_pins().len(), 2);
        assert!(memory.commit_prepared_private_vm(parent_growth).is_some());

        let child_ptr = memory
            .heap_alloc(child, PROCESS_HEAP_HANDLE, 0, 16)
            .expect("child allocation at its own base");
        assert_eq!(child_ptr, parent_ptr);
        let child_growth = memory
            .prepare_fixed_heap_growth(child, memory.heap_high_water_mark(child))
            .expect("child only charges its first page");
        assert_eq!(child_growth.new_page_pins().len(), 1);
        assert!(memory.commit_prepared_private_vm(child_growth).is_some());
        assert_eq!(
            memory.private_commit_used_bytes(),
            3 * STABLE_PRIVATE_PAGE_SIZE as u64
        );
        assert_eq!(memory.heap_high_water_mark(parent), HEAP_PTR_BASE + 0x2000);
        assert_eq!(memory.heap_high_water_mark(child), HEAP_PTR_BASE + 16);
        assert_eq!(
            memory.allocation(parent, parent_ptr).unwrap().actual_size,
            0x2000
        );
        assert_eq!(memory.allocation(child, child_ptr).unwrap().actual_size, 16);
    }

    #[test]
    fn foreign_heap_free_size_validate_and_dynamic_handle_access_fail_closed() {
        let (parent, child) = two_owners();
        let mut memory = MemorySystem::default();
        let ptr = memory
            .heap_alloc(parent, PROCESS_HEAP_HANDLE, 0, 32)
            .expect("parent allocation");
        assert_eq!(ptr, HEAP_PTR_BASE);
        assert_eq!(memory.heap_size(child, PROCESS_HEAP_HANDLE, 0, ptr), None);
        assert!(!memory.heap_validate(child, PROCESS_HEAP_HANDLE, 0, ptr));
        assert!(!memory.heap_free(child, PROCESS_HEAP_HANDLE, 0, ptr));
        assert_eq!(memory.heap_range_status(child, ptr, 1), None);
        assert!(!memory.contains_allocated_range(child, ptr, 1));
        assert!(memory.allocation(parent, ptr).is_some());

        let dynamic = memory.heap_create(parent, 0, 0, 0).expect("parent heap");
        assert!(memory.heap_alloc(child, dynamic, 0, 16).is_none());
        assert!(!memory.heap_destroy(child, dynamic));
        assert!(memory.heaps_for(parent).any(|heap| heap.handle == dynamic));
        assert!(!memory.heaps_for(child).any(|heap| heap.handle == dynamic));

        let child_ptr = memory
            .heap_alloc(child, PROCESS_HEAP_HANDLE, 0, 16)
            .expect("child's independent allocation");
        assert_eq!(child_ptr, ptr);
        assert!(memory.heap_free(child, PROCESS_HEAP_HANDLE, 0, child_ptr));
        assert!(memory.allocation(parent, ptr).is_some());
    }

    #[test]
    fn aborted_heap_holes_and_retirement_are_exact_to_one_birth() {
        let (parent, child) = two_owners();
        let mut memory = MemorySystem::default();
        let prepared = memory
            .prepare_heap_alloc(parent, PROCESS_HEAP_HANDLE, 0, 16)
            .expect("parent reservation");
        assert_eq!(prepared.allocation().ptr, HEAP_PTR_BASE);
        memory.abort_allocation(prepared);
        let child_ptr = memory
            .heap_alloc(child, PROCESS_HEAP_HANDLE, 0, 16)
            .expect("child may use the same base");
        assert_eq!(child_ptr, HEAP_PTR_BASE);
        let parent_ptr = memory
            .heap_alloc(parent, PROCESS_HEAP_HANDLE, 0, 16)
            .expect("parent preserves its hole");
        assert_eq!(parent_ptr, HEAP_PTR_BASE + 16);
        let stale = memory
            .prepare_heap_alloc(parent, PROCESS_HEAP_HANDLE, 0, 16)
            .expect("unpublished parent token");
        assert_eq!(memory.retire_heap_state_for_owner(parent), 1);
        assert_eq!(memory.commit_allocation(stale), None);
        assert!(
            memory
                .heap_alloc(parent, PROCESS_HEAP_HANDLE, 0, 16)
                .is_none()
        );
        assert_eq!(memory.allocation(child, child_ptr).unwrap().actual_size, 16);
        assert_eq!(memory.heap_high_water_mark(child), HEAP_PTR_BASE + 16);
    }

    #[test]
    fn reused_pid_receives_new_heap_authority_and_rejects_old_birth_token() {
        let mut handles = HandleTable::default();
        handles.register_boot_process(1);
        let old_handle = handles.create_process(2);
        let old_birth = handles.process_birth_key(2).expect("first PID birth");
        let mut memory = MemorySystem::default();
        let pending = memory
            .prepare_heap_alloc(old_birth, PROCESS_HEAP_HANDLE, 0, 16)
            .expect("old-birth token");
        assert!(handles.close(old_handle).is_ok());
        assert!(handles.finish_process_execution_by_id(2, 0).is_some());
        assert!(handles.process_birth_key(2).is_none());
        assert_eq!(memory.retire_heap_state_for_owner(old_birth), 0);
        let _new_handle = handles.create_process(2);
        let new_birth = handles.process_birth_key(2).expect("reused PID birth");
        assert_ne!(old_birth, new_birth);
        assert_eq!(memory.commit_allocation(pending), None);
        assert_eq!(
            memory.heap_alloc(new_birth, PROCESS_HEAP_HANDLE, 0, 16),
            Some(HEAP_PTR_BASE)
        );
        assert_eq!(memory.allocation(old_birth, HEAP_PTR_BASE), None);
        assert!(memory.allocation(new_birth, HEAP_PTR_BASE).is_some());
    }

    #[test]
    fn cloned_heap_tokens_and_allocations_do_not_cross_lineage() {
        let owner = boot_owner();
        let mut memory = MemorySystem::default();
        let prepared = memory
            .prepare_heap_alloc(owner, PROCESS_HEAP_HANDLE, 0, 16)
            .expect("source reservation");
        let mut clone = memory.clone();
        assert_eq!(clone.commit_allocation(prepared), None);
        assert_eq!(clone.allocations_for(owner).count(), 0);
        assert_eq!(memory.allocations_for(owner).count(), 0);
    }

    #[test]
    fn heap_window_crosses_old_sixteen_mib_boundary_and_fails_at_top_without_charge() {
        let (parent, child) = two_owners();
        let mut memory = MemorySystem::default();
        memory.set_private_commit_limit_bytes(0);
        let crossing = memory
            .prepare_heap_alloc(parent, PROCESS_HEAP_HANDLE, 0, 0x0200_0010)
            .expect("the fixed window extends beyond thirty-two MiB");
        assert_eq!(crossing.allocation().ptr, HEAP_PTR_BASE);
        assert_eq!(
            memory.heap_high_water_mark(parent),
            HEAP_PTR_BASE + 0x0200_0010
        );
        memory.abort_allocation(crossing);
        let rest = HEAP_PTR_LIMIT - memory.heap_high_water_mark(parent);
        let final_token = memory
            .prepare_heap_alloc(parent, PROCESS_HEAP_HANDLE, 0, rest)
            .expect("the final reservation fits exactly");
        assert_eq!(memory.heap_high_water_mark(parent), HEAP_PTR_LIMIT);
        memory.abort_allocation(final_token);
        assert!(
            memory
                .prepare_heap_alloc(parent, PROCESS_HEAP_HANDLE, 0, 1)
                .is_none()
        );
        assert_eq!(memory.private_commit_used_bytes(), 0);
        assert_eq!(
            memory.heap_alloc(child, PROCESS_HEAP_HANDLE, 0, 16),
            Some(HEAP_PTR_BASE),
            "one birth's top-boundary OOM does not advance another birth"
        );
    }

    #[test]
    fn a_foreign_birth_private_reservation_does_not_skip_own_heap_cursor() {
        let (parent, child) = two_owners();
        let mut memory = MemorySystem::default();
        assert_eq!(
            memory.heap_alloc(parent, PROCESS_HEAP_HANDLE, 0, 16),
            Some(HEAP_PTR_BASE)
        );
        assert_eq!(
            committed_private_slab(&mut memory, child, HEAP_PTR_BASE, 0x1000),
            HEAP_PTR_BASE
        );
        assert_eq!(memory.heap_high_water_mark(parent), HEAP_PTR_BASE + 16);
        assert_eq!(memory.heap_high_water_mark(child), HEAP_PTR_BASE);
    }

    #[test]
    fn contains_allocated_range_checks_heap_and_virtual_bounds() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let heap_ptr = memory
            .heap_alloc(owner, PROCESS_HEAP_HANDLE, 0, 16)
            .unwrap();
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
    fn null_base_legacy_view_scans_past_private_slab_and_occupied_legacy_slot() {
        let mut handles = HandleTable::default();
        handles.register_boot_process(1);
        let slab_owner = handles.process_birth_key(1).unwrap();
        let _view_handle = handles.create_process(2);
        let view_owner = handles.process_birth_key(2).unwrap();
        let mut memory = MemorySystem::default();
        let slab_base = committed_private_slab(&mut memory, slab_owner, 0, 0x1000);
        assert_eq!(slab_base, VIRTUAL_PTR_BASE);
        let occupied = memory
            .virtual_alloc(
                view_owner,
                VirtualAllocationSource::FileViewLegacy,
                slab_base + VIRTUAL_ALIGN,
                0x1000,
                MEM_RESERVE | MEM_COMMIT,
                4,
            )
            .unwrap();
        assert_eq!(occupied, slab_base + VIRTUAL_ALIGN);

        let view_base = memory
            .virtual_alloc(
                view_owner,
                VirtualAllocationSource::FileViewLegacy,
                0,
                0x1000,
                MEM_RESERVE | MEM_COMMIT,
                4,
            )
            .unwrap();
        assert_eq!(view_base, slab_base + 2 * VIRTUAL_ALIGN);
        assert_eq!(
            memory
                .virtual_allocation(view_owner, view_base)
                .unwrap()
                .backing,
            VirtualAllocationBacking::LegacySeed
        );
        assert!(memory.private_page_pin(slab_owner, slab_base).is_some());
        assert!(
            memory
                .virtual_alloc(
                    view_owner,
                    VirtualAllocationSource::FileViewLegacy,
                    slab_base,
                    0x1000,
                    MEM_RESERVE | MEM_COMMIT,
                    4,
                )
                .is_none(),
            "explicit LegacySeed placement still cannot alias a private slab"
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
    fn explicit_private_reserve_commit_returns_a_committed_rounded_base() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let token = memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::Private,
                0x5800_1000,
                0x1000,
                MEM_RESERVE | MEM_COMMIT,
                4,
            )
            .unwrap();
        let base = 0x5800_0000;
        assert_eq!(token.result_ptr(), base);
        assert_eq!(token.allocation().size, VIRTUAL_ALIGN);
        assert_eq!(token.new_page_pins().len(), 1);
        assert_eq!(token.new_page_pins()[0].identity().page_base(), base);
        assert_eq!(memory.commit_prepared_private_vm(token), Some(base));
        assert!(memory.private_page_pin(owner, base).is_some());
        assert!(memory.private_page_pin(owner, base + 0x1000).is_none());

        let commit = memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::Private,
                base + 0x1001,
                1,
                MEM_COMMIT,
                2,
            )
            .unwrap();
        assert_eq!(commit.result_ptr(), base + 0x1000);
        assert_eq!(
            memory.commit_prepared_private_vm(commit),
            Some(base + 0x1000)
        );
        assert_eq!(
            memory
                .private_vm_query(owner, base + 0x1000)
                .unwrap()
                .page_protect,
            Some(2)
        );
    }

    #[test]
    fn proven_pre_native_abort_reclaims_hole_and_cursor_but_not_incarnation() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let token = memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::Private,
                0,
                0x1000,
                MEM_RESERVE | MEM_COMMIT,
                4,
            )
            .unwrap();
        let base = token.base();
        let incarnation = token.allocation().incarnation;
        assert!(memory.pending_private_holes.contains_key(&(owner, base)));
        assert!(memory.next_virtual_ptr_by_owner.contains_key(&owner));
        assert!(memory.abort_prepared_private_vm(token));
        assert!(!memory.pending_private_holes.contains_key(&(owner, base)));
        assert!(!memory.next_virtual_ptr_by_owner.contains_key(&owner));
        assert_eq!(memory.private_commit_used_bytes(), 0);
        let retry = memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::Private,
                0,
                0x1000,
                MEM_RESERVE | MEM_COMMIT,
                4,
            )
            .unwrap();
        assert_eq!(retry.base(), base);
        assert!(retry.allocation().incarnation > incarnation);
        assert!(memory.abort_prepared_private_vm(retry));

        let fixed = memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::Private,
                0x5900_0000,
                0x1000,
                MEM_RESERVE,
                4,
            )
            .unwrap();
        assert!(memory.abort_prepared_private_vm(fixed));
        assert!(
            memory
                .prepare_private_vm(
                    owner,
                    VirtualAllocationSource::Private,
                    0x5900_0000,
                    0x1000,
                    MEM_RESERVE,
                    4,
                )
                .is_some()
        );
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
        assert!(!clone.pending_private_holes.contains_key(&(parent, base)));
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
    fn prepared_private_tokens_cannot_cross_a_deep_copied_memory_lineage() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let token = memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::Private,
                0x5400_0000,
                0x1000,
                MEM_RESERVE | MEM_COMMIT,
                4,
            )
            .unwrap();
        let mut clone = memory.clone();
        assert!(clone.commit_prepared_private_vm(token).is_none());
        assert!(clone.virtual_allocation(owner, 0x5400_0000).is_none());
        assert!(clone.private_page_pin(owner, 0x5400_0000).is_none());
        assert!(memory.virtual_allocation(owner, 0x5400_0000).is_none());
    }

    #[test]
    fn foreign_legacy_private_stays_globally_fenced_while_foreign_slabs_do_not() {
        let mut handles = HandleTable::default();
        handles.register_boot_process(1);
        let parent = handles.process_birth_key(1).unwrap();
        handles.create_process(2);
        let child = handles.process_birth_key(2).unwrap();
        let mut memory = MemorySystem::default();
        let base = 0x5200_0000;
        assert_eq!(
            memory.virtual_alloc(
                parent,
                VirtualAllocationSource::Private,
                base,
                0x1000,
                MEM_RESERVE | MEM_COMMIT,
                4,
            ),
            Some(base)
        );
        assert!(
            memory
                .prepare_private_vm(
                    child,
                    VirtualAllocationSource::Private,
                    base,
                    0x1000,
                    MEM_RESERVE | MEM_COMMIT,
                    4,
                )
                .is_none()
        );
        assert!(memory.virtual_free(parent, base, 0, MEM_RELEASE));
        let parent_slab = memory
            .prepare_private_vm(
                parent,
                VirtualAllocationSource::Private,
                base,
                0x1000,
                MEM_RESERVE | MEM_COMMIT,
                4,
            )
            .unwrap();
        assert_eq!(memory.commit_prepared_private_vm(parent_slab), Some(base));
        let child_slab = memory
            .prepare_private_vm(
                child,
                VirtualAllocationSource::Private,
                base,
                0x1000,
                MEM_RESERVE | MEM_COMMIT,
                4,
            )
            .unwrap();
        assert_eq!(memory.commit_prepared_private_vm(child_slab), Some(base));
        assert!(memory.contains_allocated_range(parent, base, 4));
        assert!(memory.contains_allocated_range(child, base, 4));
        assert!(!memory.intersects_foreign_virtual_allocation(parent, base, 4));
        assert!(!memory.intersects_foreign_virtual_allocation(child, base, 4));
    }

    #[test]
    fn private_reservation_does_not_validate_uncommitted_pointer_or_lose_page_query() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let token = memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::Private,
                0,
                0x3000,
                MEM_RESERVE,
                1,
            )
            .unwrap();
        let base = token.base();
        assert_eq!(memory.commit_prepared_private_vm(token), Some(base));
        assert!(!memory.contains_allocated_range(owner, base, 1));
        let query = memory.private_vm_query(owner, base).unwrap();
        assert_eq!(query.page_protect, None);
        assert_eq!(query.region_size, VIRTUAL_ALIGN);
        let commit = memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::PrivateExCompatibility,
                base + 0x1000,
                0x1000,
                MEM_COMMIT,
                0x01,
            )
            .unwrap();
        assert_eq!(
            memory.commit_prepared_private_vm(commit),
            Some(base + 0x1000)
        );
        assert!(!memory.contains_allocated_range(owner, base + 0x1000, 1));
        assert_eq!(
            memory
                .private_vm_query(owner, base + 0x1000)
                .unwrap()
                .page_protect,
            Some(0x01)
        );
        assert!(!memory.contains_allocated_range(owner, base, 0x2000));
    }

    #[test]
    fn private_commit_budget_rejects_over_limit_without_losing_earlier_bytes() {
        let mut memory = MemorySystem::default();
        memory.set_private_commit_limit_bytes((2 * STABLE_PRIVATE_PAGE_SIZE) as u64);
        let owner = boot_owner();
        let first = memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::Private,
                0,
                0x1000,
                MEM_RESERVE | MEM_COMMIT,
                4,
            )
            .unwrap();
        let first_base = first.base();
        assert_eq!(memory.commit_prepared_private_vm(first), Some(first_base));
        memory
            .private_page_pin(owner, first_base)
            .unwrap()
            .write_bytes(0, &[0x45])
            .unwrap();
        assert!(
            memory
                .prepare_private_vm(
                    owner,
                    VirtualAllocationSource::Private,
                    0,
                    0x2000,
                    MEM_RESERVE | MEM_COMMIT,
                    4,
                )
                .is_none()
        );
        let mut byte = [0];
        memory
            .private_page_pin(owner, first_base)
            .unwrap()
            .read_bytes(0, &mut byte)
            .unwrap();
        assert_eq!(byte, [0x45]);
        let second = memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::Private,
                0,
                0x1000,
                MEM_RESERVE | MEM_COMMIT,
                4,
            )
            .unwrap();
        assert_eq!(second.new_page_pins().len(), 1);
        drop(second);
        assert_eq!(
            memory.private_commit_used_bytes(),
            STABLE_PRIVATE_PAGE_SIZE as u64
        );
    }

    #[test]
    fn released_but_retained_private_pin_remains_charged_until_final_drop() {
        let mut memory = MemorySystem::default();
        memory.set_private_commit_limit_bytes(STABLE_PRIVATE_PAGE_SIZE as u64);
        let owner = boot_owner();
        let base = 0x5a00_0000;
        let first = memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::Private,
                base,
                0x1000,
                MEM_RESERVE | MEM_COMMIT,
                4,
            )
            .unwrap();
        memory.commit_prepared_private_vm(first).unwrap();
        let retained_pin = memory.private_page_pin(owner, base).unwrap();
        let release = memory
            .prepare_private_vm_free(owner, base, 0, MEM_RELEASE)
            .unwrap();
        assert_eq!(
            memory.commit_prepared_private_vm_change(release),
            Some(PrivateVmChangeResult::Free)
        );
        assert!(memory.virtual_allocation(owner, base).is_none());
        assert_eq!(
            memory.private_commit_used_bytes(),
            STABLE_PRIVATE_PAGE_SIZE as u64
        );
        assert!(
            memory
                .prepare_private_vm(
                    owner,
                    VirtualAllocationSource::Private,
                    base,
                    0x1000,
                    MEM_RESERVE | MEM_COMMIT,
                    4,
                )
                .is_none()
        );
        drop(retained_pin);
        assert_eq!(memory.private_commit_used_bytes(), 0);
        assert!(
            memory
                .prepare_private_vm(
                    owner,
                    VirtualAllocationSource::Private,
                    base,
                    0x1000,
                    MEM_RESERVE | MEM_COMMIT,
                    4,
                )
                .is_some()
        );
    }

    #[test]
    fn sixty_four_private_release_cycles_reclaim_checked_page_budget() {
        let mut memory = MemorySystem::default();
        memory.set_private_commit_limit_bytes(STABLE_PRIVATE_PAGE_SIZE as u64);
        let owner = boot_owner();
        let base = 0x5b00_0000;
        for cycle in 0..64u8 {
            let prepared = memory
                .prepare_private_vm(
                    owner,
                    VirtualAllocationSource::Private,
                    base,
                    0x1000,
                    MEM_RESERVE | MEM_COMMIT,
                    4,
                )
                .unwrap();
            memory.commit_prepared_private_vm(prepared).unwrap();
            memory
                .private_page_pin(owner, base)
                .unwrap()
                .write_bytes(0, &[cycle])
                .unwrap();
            assert_eq!(
                memory.private_commit_used_bytes(),
                STABLE_PRIVATE_PAGE_SIZE as u64
            );
            let release = memory
                .prepare_private_vm_free(owner, base, 0, MEM_RELEASE)
                .unwrap();
            memory.commit_prepared_private_vm_change(release).unwrap();
            assert_eq!(memory.private_commit_used_bytes(), 0);
        }
    }

    #[test]
    fn cloned_memory_has_independent_private_page_bytes_and_budget() {
        let mut source = MemorySystem::default();
        let owner = boot_owner();
        let prepared = source
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::Private,
                0x5c00_0000,
                0x1000,
                MEM_RESERVE | MEM_COMMIT,
                4,
            )
            .unwrap();
        let base = prepared.base();
        source.commit_prepared_private_vm(prepared).unwrap();
        let source_pin = source.private_page_pin(owner, base).unwrap();
        source_pin.write_bytes(0, &[0x45]).unwrap();
        let clone = source.clone();
        let clone_pin = clone.private_page_pin(owner, base).unwrap();
        assert_ne!(source_pin.native_ptr(), clone_pin.native_ptr());
        assert!(source.contains_private_page_pin(&source_pin));
        assert!(clone.contains_private_page_pin(&clone_pin));
        assert!(!source.contains_private_page_pin(&clone_pin));
        assert!(!clone.contains_private_page_pin(&source_pin));
        clone_pin.write_bytes(0, &[0x99]).unwrap();
        let mut byte = [0];
        source_pin.read_bytes(0, &mut byte).unwrap();
        assert_eq!(byte, [0x45]);
        clone_pin.read_bytes(0, &mut byte).unwrap();
        assert_eq!(byte, [0x99]);
        assert_eq!(
            source.private_commit_used_bytes(),
            STABLE_PRIVATE_PAGE_SIZE as u64
        );
        assert_eq!(
            clone.private_commit_used_bytes(),
            STABLE_PRIVATE_PAGE_SIZE as u64
        );
    }

    #[test]
    fn future_heap_pointer_skips_private_reservation_and_pending_heap_hole_blocks_vm() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let reserved_heap_va = HEAP_PTR_BASE;
        let token = memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::Private,
                reserved_heap_va,
                0x1000,
                MEM_RESERVE,
                4,
            )
            .unwrap();
        assert_eq!(
            memory.commit_prepared_private_vm(token),
            Some(reserved_heap_va)
        );
        let heap_ptr = memory
            .heap_alloc(owner, PROCESS_HEAP_HANDLE, 0, 16)
            .unwrap();
        assert_eq!(heap_ptr, reserved_heap_va + VIRTUAL_ALIGN);
        let pending_heap = memory
            .prepare_heap_alloc(owner, PROCESS_HEAP_HANDLE, 0, 16)
            .unwrap();
        let pending_ptr = pending_heap.allocation().ptr;
        assert!(
            memory
                .prepare_private_vm(
                    owner,
                    VirtualAllocationSource::Private,
                    pending_ptr,
                    0x1000,
                    MEM_RESERVE,
                    4,
                )
                .is_none()
        );
    }

    #[test]
    fn private_change_tokens_publish_only_after_commit_and_reject_stale_or_foreign() {
        let mut handles = HandleTable::default();
        handles.register_boot_process(1);
        let owner = handles.process_birth_key(1).unwrap();
        handles.create_process(2);
        let foreign = handles.process_birth_key(2).unwrap();
        let mut memory = MemorySystem::default();
        let first = memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::Private,
                0x5600_0000,
                0x2000,
                MEM_RESERVE | MEM_COMMIT,
                4,
            )
            .unwrap();
        let base = first.base();
        assert_eq!(memory.commit_prepared_private_vm(first), Some(base));
        let original_incarnation = memory.virtual_allocation(owner, base).unwrap().incarnation;
        assert!(
            memory
                .prepare_private_vm_free(foreign, base, 0, MEM_RELEASE)
                .is_none()
        );
        assert!(
            memory
                .prepare_private_vm_protect(foreign, base, 0x1000, 2)
                .is_none()
        );
        let protect = memory
            .prepare_private_vm_protect(owner, base + 0x1000, 0x1000, 2)
            .unwrap();
        assert_eq!(protect.native_page_changes().len(), 1);
        assert_eq!(
            memory
                .private_vm_query(owner, base + 0x1000)
                .unwrap()
                .page_protect,
            Some(4)
        );
        assert_eq!(
            memory.commit_prepared_private_vm_change(protect),
            Some(PrivateVmChangeResult::Protect { old_protect: 4 })
        );
        assert_eq!(
            memory
                .private_vm_query(owner, base + 0x1000)
                .unwrap()
                .page_protect,
            Some(2)
        );
        let stale = memory
            .prepare_private_vm_free(owner, base, 0x1000, MEM_DECOMMIT)
            .unwrap();
        let unrelated = memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::Private,
                0x5700_0000,
                0x1000,
                MEM_RESERVE,
                4,
            )
            .unwrap();
        memory.commit_prepared_private_vm(unrelated).unwrap();
        assert!(memory.commit_prepared_private_vm_change(stale).is_none());
        assert!(memory.private_page_pin(owner, base).is_some());
        let decommit = memory
            .prepare_private_vm_free(owner, base, 0x1000, MEM_DECOMMIT)
            .unwrap();
        assert!(memory.private_page_pin(owner, base).is_some());
        assert_eq!(
            memory.commit_prepared_private_vm_change(decommit),
            Some(PrivateVmChangeResult::Free)
        );
        assert!(memory.private_page_pin(owner, base).is_none());
        assert_eq!(
            memory.private_vm_query(owner, base).unwrap().page_protect,
            None
        );
        let recommit = memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::PrivateExCompatibility,
                base,
                0x1000,
                MEM_COMMIT,
                4,
            )
            .unwrap();
        memory.commit_prepared_private_vm(recommit).unwrap();
        assert_eq!(
            memory.virtual_allocation(owner, base).unwrap().incarnation,
            original_incarnation
        );
        let release = memory
            .prepare_private_vm_free(owner, base, 0, MEM_RELEASE)
            .unwrap();
        assert!(release.is_release());
        assert_eq!(release.native_page_changes().len(), 2);
        assert_eq!(
            memory.commit_prepared_private_vm_change(release),
            Some(PrivateVmChangeResult::Free)
        );
        assert!(memory.virtual_allocation(owner, base).is_none());
        let reuse = memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::Private,
                base,
                0x1000,
                MEM_RESERVE | MEM_COMMIT,
                4,
            )
            .unwrap();
        assert!(reuse.allocation().incarnation > original_incarnation);
    }

    #[test]
    fn prepared_slab_commits_one_page_not_whole_reservation() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let base = committed_private_slab(&mut memory, owner, 0, 0x1000);
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
        // The reservation has another 60 KiB, but only the committed page
        // has a writable pin. The legacy seed API must reject this slab.
        assert!(!memory.set_virtual_initial_bytes(owner, base, vec![0xa5]));
        let pin = memory.private_page_pin(owner, base).unwrap();
        pin.write_bytes(0, &0x1122_3344u32.to_le_bytes()).unwrap();
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
    fn prepared_slab_spans_two_pages_and_execution_fences_host_access() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let base = committed_private_slab(&mut memory, owner, 0, 0x2000);
        let first = memory.private_page_pin(owner, base).unwrap();
        let second = memory.private_page_pin(owner, base + 0x1000).unwrap();
        first
            .write_bytes(STABLE_PRIVATE_PAGE_SIZE - 4, &[0x5a; 4])
            .unwrap();
        second.write_bytes(0, &[1, 2, 3, 4]).unwrap();
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
    fn prepared_slab_pin_list_is_birth_scoped_and_excludes_legacy() {
        let mut handles = HandleTable::default();
        handles.register_boot_process(1);
        let parent = handles.process_birth_key(1).unwrap();
        let _child_handle = handles.create_process(2);
        let child = handles.process_birth_key(2).unwrap();
        let mut memory = MemorySystem::default();
        let slab_base = committed_private_slab(&mut memory, parent, 0, 0x1000);
        let legacy_base = memory
            .virtual_alloc(
                child,
                VirtualAllocationSource::Private,
                slab_base + VIRTUAL_ALIGN,
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
    fn prepared_slab_clone_deep_copies_bytes_and_rejects_source_pin() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let base = committed_private_slab(&mut memory, owner, 0, 0x1000);
        let source_pin = memory.private_page_pin(owner, base).unwrap();
        source_pin.write_bytes(0, &[1, 2, 3, 4]).unwrap();
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
    fn prepared_slab_release_invalidates_identity_but_pin_retains_bytes() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let base = committed_private_slab(&mut memory, owner, 0, 0x1000);
        let pin = memory.private_page_pin(owner, base).unwrap();
        pin.write_bytes(0, &[0x45]).unwrap();
        assert!(!memory.virtual_free(owner, base, 0x1000, MEM_DECOMMIT));
        assert!(!memory.virtual_free(owner, base, 0, MEM_RELEASE));
        assert!(memory.contains_private_page_pin(&pin));
        let release = memory
            .prepare_private_vm_free(owner, base, 0, MEM_RELEASE)
            .unwrap();
        assert_eq!(release.native_page_changes().len(), 1);
        assert_eq!(
            memory.commit_prepared_private_vm_change(release),
            Some(PrivateVmChangeResult::Free)
        );
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
    fn prepared_slab_rejects_invalid_request_without_publication() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        for (address, size, flags, protect) in [
            (0, 0, MEM_COMMIT | MEM_RESERVE, 4),
            (0, 0x1000, MEM_COMMIT, 4),
            (0, 0x1000, MEM_COMMIT | MEM_RESERVE | 0x10, 4),
            (0, 0x1000, MEM_COMMIT | MEM_RESERVE, 3),
        ] {
            assert!(
                memory
                    .prepare_private_vm(
                        owner,
                        VirtualAllocationSource::Private,
                        address,
                        size,
                        flags,
                        protect,
                    )
                    .is_none()
            );
        }
        assert_eq!(memory.virtual_allocations().count(), 0);
        assert!(!memory.has_stable_private_slabs());
    }

    #[test]
    fn local_re_alloc_grows_in_place_when_no_later_allocation_blocks() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        let ptr = memory.local_alloc(owner, 0, 16).unwrap();

        let grown = memory.local_re_alloc(owner, ptr, 64, 0).unwrap();

        // With no subsequent allocation, the realloc may or may not move,
        // but the new size must be at least 64.
        assert_eq!(memory.local_size(owner, grown), Some(64));
    }

    #[test]
    fn heap_range_status_distinguishes_requested_actual_and_outside() {
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        // 8-byte allocation: actual_size = align_up(8, 16) = 16, requested_size = 8.
        let ptr = memory.heap_alloc(owner, PROCESS_HEAP_HANDLE, 0, 8).unwrap();

        // Within requested range → Some(true).
        assert_eq!(memory.heap_range_status(owner, ptr, 8), Some(true));
        assert_eq!(memory.heap_range_status(owner, ptr, 1), Some(true));

        // Past requested size but within actual allocation → Some(false).
        assert_eq!(memory.heap_range_status(owner, ptr, 9), Some(false));

        // Completely outside (past actual end, and before ptr) → None.
        assert_eq!(memory.heap_range_status(owner, ptr + 16, 1), None);
        assert_eq!(memory.heap_range_status(owner, ptr - 1, 1), None);
    }

    #[test]
    fn fixed_heap_growth_charges_all_crossed_pages_before_allocation_commit() {
        const ARENA_SIZE: u32 = 0x0100_0000;
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        memory.set_private_commit_limit_bytes(0);
        let first_heap_ptr = memory
            .heap_alloc(owner, PROCESS_HEAP_HANDLE, 0, 16)
            .unwrap();
        assert_eq!(first_heap_ptr, HEAP_PTR_BASE);
        assert_eq!(
            memory.reserve_fixed_private_range(
                owner,
                VirtualAllocationSource::ProcessHeapArena,
                HEAP_PTR_BASE,
                ARENA_SIZE,
                4,
            ),
            Some(HEAP_PTR_BASE)
        );
        assert_eq!(memory.private_commit_used_bytes(), 0);
        assert!(memory.private_page_pins_for(owner).is_empty());
        assert_eq!(
            memory.fixed_heap_reservation(owner),
            Some((HEAP_PTR_BASE, ARENA_SIZE))
        );
        assert_eq!(
            memory
                .private_vm_query(owner, HEAP_PTR_BASE)
                .unwrap()
                .page_protect,
            None
        );
        assert!(
            memory
                .prepare_fixed_heap_growth(owner, first_heap_ptr + 16)
                .is_none()
        );
        assert_eq!(
            memory.reserve_fixed_private_range(
                owner,
                VirtualAllocationSource::ProcessHeapArena,
                HEAP_PTR_BASE,
                ARENA_SIZE,
                4,
            ),
            Some(HEAP_PTR_BASE),
            "an exact repeated reservation retains its incarnation and pages"
        );
        let pending_allocation = memory
            .prepare_heap_alloc(owner, PROCESS_HEAP_HANDLE, 0, 0x2000)
            .unwrap();
        assert_eq!(pending_allocation.allocation().ptr, HEAP_PTR_BASE + 16);
        let high_water = pending_allocation.allocation().ptr + 0x2000;
        memory.set_private_commit_limit_bytes(2 * STABLE_PRIVATE_PAGE_SIZE as u64);
        assert!(
            memory
                .prepare_fixed_heap_growth(owner, high_water)
                .is_none()
        );
        assert_eq!(memory.private_commit_used_bytes(), 0);
        memory.set_private_commit_limit_bytes(3 * STABLE_PRIVATE_PAGE_SIZE as u64);
        // GlobalMemoryStatus.dwAvailPhys reports this same available-byte
        // field, so detached preparation must debit the full high-water rise.
        let available_before = memory.private_commit_available_bytes();
        let prepared = memory.prepare_fixed_heap_growth(owner, high_water).unwrap();
        assert_eq!(prepared.result_ptr(), HEAP_PTR_BASE + 0x3000);
        assert_eq!(prepared.new_page_pins().len(), 3);
        let pointers = prepared
            .new_page_pins()
            .iter()
            .map(StablePrivatePagePin::native_ptr)
            .collect::<Vec<_>>();
        assert_eq!(
            memory.private_commit_used_bytes(),
            3 * STABLE_PRIVATE_PAGE_SIZE as u64
        );
        assert_eq!(
            available_before - memory.private_commit_available_bytes(),
            3 * STABLE_PRIVATE_PAGE_SIZE as u64
        );
        assert!(memory.private_page_pin(owner, HEAP_PTR_BASE).is_none());
        assert_eq!(
            memory.commit_prepared_private_vm(prepared),
            Some(HEAP_PTR_BASE + 0x3000)
        );
        assert_eq!(
            memory.commit_allocation(pending_allocation),
            Some(HEAP_PTR_BASE + 16)
        );
        for (index, pointer) in pointers.into_iter().enumerate() {
            let page = HEAP_PTR_BASE + index as u32 * STABLE_PRIVATE_PAGE_SIZE as u32;
            assert_eq!(
                memory.private_page_pin(owner, page).unwrap().native_ptr(),
                pointer
            );
        }
        assert_eq!(memory.private_page_pins_for(owner).len(), 3);
        assert_eq!(
            memory
                .private_vm_query(owner, HEAP_PTR_BASE)
                .unwrap()
                .page_protect,
            Some(4)
        );
        let repeat = memory.prepare_fixed_heap_growth(owner, high_water).unwrap();
        assert!(repeat.new_page_pins().is_empty());
        assert_eq!(
            memory.commit_prepared_private_vm(repeat),
            Some(HEAP_PTR_BASE + 0x3000)
        );
        assert_eq!(
            memory.private_commit_used_bytes(),
            3 * STABLE_PRIVATE_PAGE_SIZE as u64
        );
        assert!(
            memory
                .prepare_fixed_private_page(owner, HEAP_PTR_BASE + 0x3000)
                .is_none()
        );
        assert!(
            memory
                .prepare_fixed_heap_growth(owner, HEAP_PTR_BASE + 0x4000)
                .is_none()
        );
        assert_eq!(
            memory.private_commit_used_bytes(),
            3 * STABLE_PRIVATE_PAGE_SIZE as u64
        );
    }

    #[test]
    fn fixed_stack_span_commits_all_untouched_pages_with_one_charge() {
        const STACK_BASE: u32 = 0x7000_0000;
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        assert_eq!(
            memory.reserve_fixed_private_range(
                owner,
                VirtualAllocationSource::ThreadStack,
                STACK_BASE,
                0x4000,
                4,
            ),
            Some(STACK_BASE)
        );
        let available_before = memory.private_commit_available_bytes();
        let generation_before = memory.virtual_generation();
        let prepared = memory
            .prepare_fixed_private_span(owner, STACK_BASE + 0xff0, 0x1011)
            .unwrap();
        assert_eq!(prepared.result_ptr(), STACK_BASE);
        assert_eq!(prepared.new_page_pins().len(), 3);
        assert_eq!(
            prepared
                .new_page_pins()
                .iter()
                .map(|pin| pin.identity().page_base())
                .collect::<Vec<_>>(),
            [STACK_BASE, STACK_BASE + 0x1000, STACK_BASE + 0x2000]
        );
        assert_eq!(memory.virtual_generation(), generation_before);
        assert!(memory.private_page_pins_for(owner).is_empty());
        assert_eq!(
            available_before - memory.private_commit_available_bytes(),
            3 * STABLE_PRIVATE_PAGE_SIZE as u64
        );
        assert_eq!(
            memory.commit_prepared_private_vm(prepared),
            Some(STACK_BASE)
        );
        assert_eq!(memory.private_page_pins_for(owner).len(), 3);
        for offset in [0, 0x1000, 0x2000] {
            assert_eq!(
                memory
                    .private_vm_query(owner, STACK_BASE + offset)
                    .unwrap()
                    .page_protect,
                Some(4)
            );
        }
        assert_eq!(
            memory
                .private_vm_query(owner, STACK_BASE + 0x3000)
                .unwrap()
                .page_protect,
            None
        );
        let repeat = memory
            .prepare_fixed_private_span(owner, STACK_BASE + 0xff0, 0x1011)
            .unwrap();
        assert!(repeat.new_page_pins().is_empty());
        assert_eq!(memory.commit_prepared_private_vm(repeat), Some(STACK_BASE));
        assert_eq!(
            memory.private_commit_used_bytes(),
            3 * STABLE_PRIVATE_PAGE_SIZE as u64
        );
    }

    #[cfg(windows)]
    #[test]
    fn fixed_block_sparse_pages_clone_and_detached_slot_do_not_alias() {
        const STACK_BASE: u32 = 0x7000_0000;
        let mut source = MemorySystem::default();
        let owner = boot_owner();
        assert_eq!(
            source.reserve_fixed_private_range(
                owner,
                VirtualAllocationSource::ThreadStack,
                STACK_BASE,
                0x20_0000,
                4,
            ),
            Some(STACK_BASE)
        );
        for page in [STACK_BASE, STACK_BASE + 0x2000] {
            let proposal = source.prepare_fixed_private_page(owner, page).unwrap();
            assert_eq!(source.commit_prepared_private_vm(proposal), Some(page));
        }
        let first = source.private_page_pin(owner, STACK_BASE).unwrap();
        let third = source.private_page_pin(owner, STACK_BASE + 0x2000).unwrap();
        assert!(first.is_host_block_backed() && third.is_host_block_backed());
        assert_eq!(
            third.native_ptr() as usize - first.native_ptr() as usize,
            0x2000
        );
        assert!(
            source
                .private_page_pin(owner, STACK_BASE + 0x1000)
                .is_none()
        );
        assert_eq!(source.private_commit_used_bytes(), 0x2000);
        first.write_bytes(0, &[0x41]).unwrap();
        third.write_bytes(0, &[0x43]).unwrap();

        // A detached proposal occupies its block offset before publication.
        // A competing proposal must not receive that same host pointer.
        let pending = source
            .prepare_fixed_private_page(owner, STACK_BASE + 0x1000)
            .unwrap();
        assert!(
            source
                .prepare_fixed_private_page(owner, STACK_BASE + 0x1000)
                .is_none()
        );
        assert_eq!(source.private_commit_used_bytes(), 0x3000);
        drop(pending);
        assert_eq!(source.private_commit_used_bytes(), 0x2000);
        let replacement = source
            .prepare_fixed_private_page(owner, STACK_BASE + 0x1000)
            .unwrap();
        assert_eq!(
            source.commit_prepared_private_vm(replacement),
            Some(STACK_BASE + 0x1000)
        );

        let mut clone = source.clone();
        let clone_first = clone.private_page_pin(owner, STACK_BASE).unwrap();
        let clone_third = clone.private_page_pin(owner, STACK_BASE + 0x2000).unwrap();
        assert!(clone_first.is_host_block_backed() && clone_third.is_host_block_backed());
        assert_ne!(clone_first.native_ptr(), first.native_ptr());
        assert_eq!(
            clone_third.native_ptr() as usize - clone_first.native_ptr() as usize,
            0x2000
        );
        clone_first.write_bytes(0, &[0x99]).unwrap();
        let mut byte = [0];
        first.read_bytes(0, &mut byte).unwrap();
        assert_eq!(byte, [0x41]);
        clone_first.read_bytes(0, &mut byte).unwrap();
        assert_eq!(byte, [0x99]);
        assert_eq!(source.private_commit_used_bytes(), 0x3000);
        assert_eq!(clone.private_commit_used_bytes(), 0x3000);
        source.retire_fixed_private_ranges_for_owner(owner);
        clone.retire_fixed_private_ranges_for_owner(owner);
        assert_eq!(source.private_commit_used_bytes(), 0x2000);
        assert_eq!(clone.private_commit_used_bytes(), 0x2000);
        drop((first, third, clone_first, clone_third));
        assert_eq!(source.private_commit_used_bytes(), 0);
        assert_eq!(clone.private_commit_used_bytes(), 0);
    }

    #[cfg(windows)]
    #[test]
    fn fixed_block_first_cold_page_competing_proposals_have_one_pending_slot() {
        const STACK_BASE: u32 = 0x7100_0000;
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        memory
            .reserve_fixed_private_range(
                owner,
                VirtualAllocationSource::ThreadStack,
                STACK_BASE,
                0x10_0000,
                4,
            )
            .unwrap();
        let first = memory
            .prepare_fixed_private_page(owner, STACK_BASE)
            .unwrap();
        assert!(first.new_page_pins()[0].is_host_block_backed());
        assert!(
            memory
                .prepare_fixed_private_page(owner, STACK_BASE)
                .is_none(),
            "a second detached token may not allocate competing bytes for the same cold page"
        );
        assert_eq!(memory.private_commit_used_bytes(), 0x1000);
        assert!(memory.private_page_pin(owner, STACK_BASE).is_none());
        drop(first);
        assert_eq!(memory.private_commit_used_bytes(), 0);
        let replacement = memory
            .prepare_fixed_private_page(owner, STACK_BASE)
            .unwrap();
        assert!(replacement.new_page_pins()[0].is_host_block_backed());
        assert_eq!(
            memory.commit_prepared_private_vm(replacement),
            Some(STACK_BASE)
        );
        assert_eq!(memory.private_commit_used_bytes(), 0x1000);
    }

    #[test]
    fn fixed_stack_span_short_budget_or_bad_tail_publishes_no_pages() {
        const STACK_BASE: u32 = 0x7000_0000;
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        memory.set_private_commit_limit_bytes(2 * STABLE_PRIVATE_PAGE_SIZE as u64);
        memory
            .reserve_fixed_private_range(
                owner,
                VirtualAllocationSource::ThreadStack,
                STACK_BASE,
                0x3000,
                4,
            )
            .unwrap();
        let first = memory
            .prepare_fixed_private_page(owner, STACK_BASE)
            .unwrap();
        assert_eq!(memory.commit_prepared_private_vm(first), Some(STACK_BASE));
        let published = memory
            .virtual_allocation(owner, STACK_BASE)
            .unwrap()
            .clone();
        let generation = memory.virtual_generation();
        let available = memory.private_commit_available_bytes();
        assert!(
            memory
                .prepare_fixed_private_span(owner, STACK_BASE + 0xff0, 0x1011)
                .is_none(),
            "two missing pages must not partially charge a one-page budget"
        );
        assert!(
            memory
                .prepare_fixed_private_span(owner, STACK_BASE + 0x2ff0, 0x20)
                .is_none(),
            "a tail beyond the reservation must reject before mutation"
        );
        assert!(
            memory
                .prepare_fixed_private_span(owner, u32::MAX - 1, 3)
                .is_none(),
            "a wrapping span must reject before mutation"
        );
        assert_eq!(memory.virtual_generation(), generation);
        assert_eq!(memory.private_commit_available_bytes(), available);
        assert_eq!(
            memory.virtual_allocation(owner, STACK_BASE),
            Some(&published)
        );
        assert_eq!(memory.private_page_pins_for(owner).len(), 1);
        assert!(
            memory
                .private_page_pin(owner, STACK_BASE + 0x1000)
                .is_none()
        );
        assert!(
            memory
                .private_page_pin(owner, STACK_BASE + 0x2000)
                .is_none()
        );
    }

    #[test]
    fn fixed_stack_span_same_va_keeps_process_births_independent() {
        const STACK_BASE: u32 = 0x7000_0000;
        let mut handles = HandleTable::default();
        handles.register_boot_process(1);
        let first_owner = handles.process_birth_key(1).unwrap();
        let _child_handle = handles.create_process(2);
        let second_owner = handles.process_birth_key(2).unwrap();
        let mut memory = MemorySystem::default();
        for owner in [first_owner, second_owner] {
            memory
                .reserve_fixed_private_range(
                    owner,
                    VirtualAllocationSource::ThreadStack,
                    STACK_BASE,
                    0x3000,
                    4,
                )
                .unwrap();
            let prepared = memory
                .prepare_fixed_private_span(owner, STACK_BASE + 0xff0, 0x1011)
                .unwrap();
            assert_eq!(prepared.new_page_pins().len(), 3);
            assert_eq!(
                memory.commit_prepared_private_vm(prepared),
                Some(STACK_BASE)
            );
        }
        let first = memory
            .private_page_pin(first_owner, STACK_BASE + 0x2000)
            .unwrap();
        let second = memory
            .private_page_pin(second_owner, STACK_BASE + 0x2000)
            .unwrap();
        assert_ne!(first.native_ptr(), second.native_ptr());
        first.write_bytes(0, &[0x31]).unwrap();
        second.write_bytes(0, &[0x72]).unwrap();
        let mut byte = [0];
        first.read_bytes(0, &mut byte).unwrap();
        assert_eq!(byte, [0x31]);
        second.read_bytes(0, &mut byte).unwrap();
        assert_eq!(byte, [0x72]);
        assert_eq!(memory.private_page_pins_for(first_owner).len(), 3);
        assert_eq!(memory.private_page_pins_for(second_owner).len(), 3);
        assert_eq!(
            memory.private_commit_used_bytes(),
            6 * STABLE_PRIVATE_PAGE_SIZE as u64
        );
    }

    #[test]
    fn fixed_stack_pages_at_same_va_are_isolated_by_process_birth() {
        const STACK_BASE: u32 = 0x7000_0000;
        let mut handles = HandleTable::default();
        handles.register_boot_process(1);
        let first_owner = handles.process_birth_key(1).unwrap();
        let _child_handle = handles.create_process(2);
        let second_owner = handles.process_birth_key(2).unwrap();
        let mut memory = MemorySystem::default();
        let mut available_before = memory.private_commit_available_bytes();
        for owner in [first_owner, second_owner] {
            assert_eq!(
                memory.reserve_fixed_private_range(
                    owner,
                    VirtualAllocationSource::ThreadStack,
                    STACK_BASE,
                    0x2000,
                    4,
                ),
                Some(STACK_BASE)
            );
            let prepared = memory
                .prepare_fixed_private_page(owner, STACK_BASE + 3)
                .unwrap();
            assert_eq!(prepared.new_page_pins().len(), 1);
            assert_eq!(
                memory.commit_prepared_private_vm(prepared),
                Some(STACK_BASE)
            );
            // This is the source for GlobalMemoryStatus.dwAvailPhys: one
            // newly touched stack page subtracts exactly 4 KiB.
            assert_eq!(
                available_before - memory.private_commit_available_bytes(),
                STABLE_PRIVATE_PAGE_SIZE as u64
            );
            available_before = memory.private_commit_available_bytes();
        }
        let first = memory.private_page_pin(first_owner, STACK_BASE).unwrap();
        let second = memory.private_page_pin(second_owner, STACK_BASE).unwrap();
        assert_ne!(first.native_ptr(), second.native_ptr());
        first.write_bytes(0, &[0x31]).unwrap();
        second.write_bytes(0, &[0x72]).unwrap();
        let mut byte = [0];
        first.read_bytes(0, &mut byte).unwrap();
        assert_eq!(byte, [0x31]);
        second.read_bytes(0, &mut byte).unwrap();
        assert_eq!(byte, [0x72]);
        assert_eq!(memory.private_page_pins_for(first_owner).len(), 1);
        assert_eq!(memory.private_page_pins_for(second_owner).len(), 1);
        assert_eq!(
            memory.private_commit_used_bytes(),
            2 * STABLE_PRIVATE_PAGE_SIZE as u64
        );
        assert!(!memory.intersects_foreign_virtual_allocation(first_owner, STACK_BASE, 1));
        assert!(!memory.intersects_foreign_virtual_allocation(second_owner, STACK_BASE, 1));
    }

    #[test]
    fn fixed_reservation_batch_rejects_later_stack_without_stranding_heap() {
        const OLD_STACK: u32 = 0x7100_0000;
        const NEW_STACK: u32 = 0x7200_0000;
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        assert_eq!(
            memory.reserve_fixed_private_range(
                owner,
                VirtualAllocationSource::ThreadStack,
                OLD_STACK,
                0x2000,
                4,
            ),
            Some(OLD_STACK)
        );
        let page = memory.prepare_fixed_private_page(owner, OLD_STACK).unwrap();
        memory.commit_prepared_private_vm(page).unwrap();
        let existing_pin = memory.private_page_pin(owner, OLD_STACK).unwrap();
        let existing_incarnation = memory
            .virtual_allocation(owner, OLD_STACK)
            .unwrap()
            .incarnation;
        let generation = memory.virtual_generation();
        let next_incarnation = memory.next_virtual_incarnation;
        let available = memory.private_commit_available_bytes();

        let bad_later_stack = [
            (
                VirtualAllocationSource::ProcessHeapArena,
                HEAP_PTR_BASE,
                0x10000,
                4,
            ),
            (VirtualAllocationSource::ThreadStack, OLD_STACK, 0x3000, 4),
        ];
        assert!(
            memory
                .reserve_fixed_private_ranges(owner, &bad_later_stack)
                .is_none()
        );
        assert!(memory.fixed_heap_reservation(owner).is_none());
        assert_eq!(memory.virtual_generation(), generation);
        assert_eq!(memory.next_virtual_incarnation, next_incarnation);
        assert_eq!(memory.private_commit_available_bytes(), available);
        assert_eq!(
            memory
                .private_page_pin(owner, OLD_STACK)
                .unwrap()
                .native_ptr(),
            existing_pin.native_ptr()
        );
        assert_eq!(
            memory
                .virtual_allocation(owner, OLD_STACK)
                .unwrap()
                .incarnation,
            existing_incarnation
        );

        let valid = [
            (
                VirtualAllocationSource::ProcessHeapArena,
                HEAP_PTR_BASE,
                0x10000,
                4,
            ),
            (VirtualAllocationSource::ThreadStack, NEW_STACK, 0x2000, 4),
            (VirtualAllocationSource::ThreadStack, OLD_STACK, 0x2000, 4),
        ];
        assert_eq!(
            memory.reserve_fixed_private_ranges(owner, &valid),
            Some(vec![
                (HEAP_PTR_BASE, 0x10000),
                (NEW_STACK, 0x2000),
                (OLD_STACK, 0x2000)
            ])
        );
        assert_eq!(memory.virtual_generation(), generation + 1);
        assert_eq!(memory.next_virtual_incarnation, next_incarnation + 2);
        assert_eq!(memory.private_commit_available_bytes(), available);
        let published_generation = memory.virtual_generation();
        assert_eq!(
            memory.reserve_fixed_private_ranges(owner, &valid),
            Some(vec![
                (HEAP_PTR_BASE, 0x10000),
                (NEW_STACK, 0x2000),
                (OLD_STACK, 0x2000)
            ])
        );
        assert_eq!(memory.virtual_generation(), published_generation);
        assert_eq!(memory.next_virtual_incarnation, next_incarnation + 2);
        assert_eq!(
            memory
                .private_page_pin(owner, OLD_STACK)
                .unwrap()
                .native_ptr(),
            existing_pin.native_ptr()
        );

        let duplicate = [valid[0], valid[0]];
        assert!(
            memory
                .reserve_fixed_private_ranges(owner, &duplicate)
                .is_none()
        );
        let cross_source_overlap = [
            valid[0],
            (
                VirtualAllocationSource::ThreadStack,
                HEAP_PTR_BASE + 0x1000,
                0x2000,
                4,
            ),
        ];
        assert!(
            memory
                .reserve_fixed_private_ranges(owner, &cross_source_overlap)
                .is_none()
        );
        assert_eq!(memory.virtual_generation(), published_generation);
        assert_eq!(memory.private_commit_available_bytes(), available);
    }

    #[test]
    fn fixed_stack_clone_deep_copies_pages_and_guest_vm_mutations_are_rejected() {
        const STACK_BASE: u32 = 0x7100_0000;
        let mut memory = MemorySystem::default();
        let owner = boot_owner();
        assert_eq!(
            memory.reserve_fixed_private_range(
                owner,
                VirtualAllocationSource::ThreadStack,
                STACK_BASE,
                0x2000,
                4,
            ),
            Some(STACK_BASE)
        );
        let prepared = memory
            .prepare_fixed_private_page(owner, STACK_BASE + 1)
            .unwrap();
        assert_eq!(
            memory.commit_prepared_private_vm(prepared),
            Some(STACK_BASE)
        );
        let source_pin = memory.private_page_pin(owner, STACK_BASE).unwrap();
        source_pin.write_bytes(0, &[0x45]).unwrap();
        let clone = memory.clone();
        let clone_pin = clone.private_page_pin(owner, STACK_BASE).unwrap();
        assert_ne!(source_pin.native_ptr(), clone_pin.native_ptr());
        assert!(!clone.contains_private_page_pin(&source_pin));
        assert!(!memory.contains_private_page_pin(&clone_pin));
        clone_pin.write_bytes(0, &[0x99]).unwrap();
        let mut byte = [0];
        source_pin.read_bytes(0, &mut byte).unwrap();
        assert_eq!(byte, [0x45]);
        clone_pin.read_bytes(0, &mut byte).unwrap();
        assert_eq!(byte, [0x99]);
        assert_eq!(
            memory.private_commit_used_bytes(),
            STABLE_PRIVATE_PAGE_SIZE as u64
        );
        assert_eq!(
            clone.private_commit_used_bytes(),
            STABLE_PRIVATE_PAGE_SIZE as u64
        );

        assert!(
            memory
                .virtual_alloc(
                    owner,
                    VirtualAllocationSource::ThreadStack,
                    STACK_BASE,
                    0x1000,
                    MEM_COMMIT,
                    4,
                )
                .is_none()
        );
        assert!(
            memory
                .prepare_private_vm(
                    owner,
                    VirtualAllocationSource::Private,
                    STACK_BASE,
                    0x1000,
                    MEM_COMMIT,
                    4,
                )
                .is_none()
        );
        assert!(!memory.virtual_free(owner, STACK_BASE, 0, MEM_RELEASE));
        assert!(!memory.virtual_free(owner, STACK_BASE, 0x1000, MEM_DECOMMIT));
        assert!(
            memory
                .prepare_private_vm_free(owner, STACK_BASE, 0, MEM_RELEASE)
                .is_none()
        );
        assert!(
            memory
                .prepare_private_vm_free(owner, STACK_BASE, 0x1000, MEM_DECOMMIT)
                .is_none()
        );
        assert!(
            memory
                .prepare_private_vm_protect(owner, STACK_BASE, 0x1000, 0x02)
                .is_none()
        );
        assert_eq!(
            memory
                .private_page_pin(owner, STACK_BASE)
                .unwrap()
                .native_ptr(),
            source_pin.native_ptr()
        );
        assert_eq!(
            memory.private_commit_used_bytes(),
            STABLE_PRIVATE_PAGE_SIZE as u64
        );
    }

    #[test]
    fn fixed_owner_retirement_preserves_native_pin_until_close_and_other_births() {
        const STACK_BASE: u32 = 0x7200_0000;
        let mut handles = HandleTable::default();
        handles.register_boot_process(1);
        let first_owner = handles.process_birth_key(1).unwrap();
        let _child_handle = handles.create_process(2);
        let second_owner = handles.process_birth_key(2).unwrap();
        let mut memory = MemorySystem::default();
        assert_eq!(
            memory.reserve_fixed_private_range(
                first_owner,
                VirtualAllocationSource::ProcessHeapArena,
                HEAP_PTR_BASE,
                0x10000,
                4,
            ),
            Some(HEAP_PTR_BASE)
        );
        for owner in [first_owner, second_owner] {
            assert_eq!(
                memory.reserve_fixed_private_range(
                    owner,
                    VirtualAllocationSource::ThreadStack,
                    STACK_BASE,
                    0x2000,
                    4,
                ),
                Some(STACK_BASE)
            );
            let page = memory
                .prepare_fixed_private_page(owner, STACK_BASE)
                .unwrap();
            assert_eq!(memory.commit_prepared_private_vm(page), Some(STACK_BASE));
        }
        let unrelated_private_base =
            committed_private_slab(&mut memory, first_owner, 0x6000_0000, 0x1000);
        let retained_native_pin = memory.private_page_pin(first_owner, STACK_BASE).unwrap();
        assert_eq!(
            memory.private_commit_used_bytes(),
            3 * STABLE_PRIVATE_PAGE_SIZE as u64
        );

        assert_eq!(memory.retire_fixed_private_ranges_for_owner(first_owner), 2);
        assert_eq!(memory.retire_fixed_private_ranges_for_owner(first_owner), 0);
        assert!(memory.fixed_heap_reservation(first_owner).is_none());
        assert!(memory.private_page_pin(first_owner, STACK_BASE).is_none());
        assert!(memory.private_page_pin(second_owner, STACK_BASE).is_some());
        assert!(
            memory
                .private_page_pin(first_owner, unrelated_private_base)
                .is_some()
        );
        assert_eq!(
            memory.private_commit_used_bytes(),
            3 * STABLE_PRIVATE_PAGE_SIZE as u64,
            "a retained native pin delays release of its page budget"
        );
        drop(retained_native_pin);
        assert_eq!(
            memory.private_commit_used_bytes(),
            2 * STABLE_PRIVATE_PAGE_SIZE as u64
        );
        assert_eq!(
            memory.retire_fixed_private_ranges_for_owner(second_owner),
            1
        );
        assert_eq!(
            memory.private_commit_used_bytes(),
            STABLE_PRIVATE_PAGE_SIZE as u64
        );
        assert!(
            memory
                .private_page_pin(first_owner, unrelated_private_base)
                .is_some()
        );
    }
}
