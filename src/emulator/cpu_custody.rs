//! Exact-birth CPU custody and a directory-issued, revocable execution leaf.
//!
//! The runnable-slot API is a resource owner, not a Task or Scheduler. The
//! caller must independently validate the canonical Kernel lineage, thread
//! birth and frame before issuing or using a leaf. The legacy terminal slot
//! and separate unverified whole-tree hold remain available during migration.
//! The process-only permit below is a Cut 1 authority foundation, not a live
//! execution route: `UnicornMips` native entry and `MachineKernelPort` still
//! require a leaf permit until both are migrated to the same process attempt.

use std::{
    cell::{Cell, RefCell},
    fmt,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

#[cfg(test)]
use std::cell::RefMut;

use crate::{
    ce::{
        kernel::{AcceptedProcessExit, CeKernel, CpuMachineLineageError},
        object::{ProcessBirthKey, TaskFrameKey, TaskFrameRef, ThreadBirthKey},
    },
    emulator::{
        cpu::{CpuBackend, UnicornMips},
        types::{AttemptExecution, NativeSliceOutcome, UnicornMemoryFault},
    },
};

static NEXT_MACHINE_LINEAGE: AtomicU64 = AtomicU64::new(1);

/// One slot in one machine lineage. A reused index always has a new generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CpuCustodyKey {
    machine_lineage: u64,
    index: usize,
    generation: u64,
}

/// A separate, generation-checked holding place for a whole returned CPU
/// tree. It is not an executable slot or a verified per-process CPU owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RetainedTreeKey {
    machine_lineage: u64,
    index: usize,
    generation: u64,
}

impl RetainedTreeKey {
    pub fn machine_lineage(self) -> u64 {
        self.machine_lineage
    }

    pub fn index(self) -> usize {
        self.index
    }

    pub fn generation(self) -> u64 {
        self.generation
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetainedTreeState {
    ReservedUnbound,
    UnverifiedStopped,
    /// A reservation token was lost without an explicit cancel or deposit.
    /// This is a terminal custody fault, not evidence that a CPU survived.
    Abandoned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetainedTreeSnapshot {
    pub key: RetainedTreeKey,
    /// The launch that reserved capacity, never a substitute CPU birth.
    pub origin_birth: ProcessBirthKey,
    pub state: RetainedTreeState,
    pub metrics_available: bool,
    pub mapped_blob_bytes: u64,
    /// Only the returned root's virtual map; nested parked maps are not
    /// summed by this diagnostic field.
    pub root_virtual_region_bytes: u64,
}

impl CpuCustodyKey {
    pub fn machine_lineage(self) -> u64 {
        self.machine_lineage
    }

    pub fn index(self) -> usize {
        self.index
    }

    pub fn generation(self) -> u64 {
        self.generation
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuCustodyState {
    Reserved,
    Runnable,
    Leased,
    Parked,
    SuspendedForChild,
    Quarantined,
}

/// A canonical frame key plus its observed version, when a route has one.
/// `None` is permitted only for routes that have not yet acquired a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuFrameStamp {
    key: TaskFrameKey,
    version: u64,
}

impl CpuFrameStamp {
    pub fn new(key: TaskFrameKey, version: u64) -> Self {
        Self { key, version }
    }

    pub fn key(self) -> TaskFrameKey {
        self.key
    }

    pub fn version(self) -> u64 {
        self.version
    }
}

/// The typed, non-runnable disposition of an admitted CPU. It says nothing
/// about canonical process exit and does not authorize automatic retry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuQuarantineReason {
    NativeFault,
    BackendFailure,
    PortFailure,
    RecoveryRequired,
    UnresolvedLeaf,
    UnresolvedProcessAttempt,
    UnresolvedParentSuspension,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuLeaseEnd {
    Runnable,
    Parked(TaskFrameRef),
    Quarantined(CpuQuarantineReason),
}

/// A snapshot of one exact admitted slot's custody phase. Canonical Task and
/// wait state still live in Kernel; this phase is only cached eligibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuAdmittedPhase {
    Runnable,
    Leased,
    Parked(TaskFrameRef),
    SuspendedForChild,
    Quarantined(CpuQuarantineReason),
}

/// An opaque witness of the one currently issued execution leaf. Copies are
/// harmless: every use must revalidate the directory's active nonce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuLeafPermit {
    key: CpuCustodyKey,
    birth: ProcessBirthKey,
    thread_birth: ThreadBirthKey,
    frame: Option<CpuFrameStamp>,
    attempt_generation: u64,
}

/// Cut 1 authority for one process CPU slot and one run attempt. It deliberately
/// carries no thread or frame: those require a separately checked leaf cut.
/// Copies are only witnesses; the directory revokes them on finish or drop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuProcessPermit {
    key: CpuCustodyKey,
    birth: ProcessBirthKey,
    attempt_generation: u64,
}

impl CpuProcessPermit {
    pub fn key(self) -> CpuCustodyKey {
        self.key
    }

    pub fn birth(self) -> ProcessBirthKey {
        self.birth
    }

    pub fn attempt_generation(self) -> u64 {
        self.attempt_generation
    }
}

impl CpuLeafPermit {
    pub fn key(self) -> CpuCustodyKey {
        self.key
    }

    pub fn birth(self) -> ProcessBirthKey {
        self.birth
    }

    pub fn thread_birth(self) -> ThreadBirthKey {
        self.thread_birth
    }

    pub fn frame(self) -> Option<CpuFrameStamp> {
        self.frame
    }

    pub fn attempt_generation(self) -> u64 {
        self.attempt_generation
    }
}

/// Diagnostic values only: neither an executable lease nor a CPU clone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuCustodySnapshot {
    pub key: CpuCustodyKey,
    pub birth: ProcessBirthKey,
    pub state: CpuCustodyState,
    /// False while a scoped payload loan is open; zero metrics then mean
    /// unavailable, not an empty CPU address space.
    pub metrics_available: bool,
    pub quarantine_reason: Option<CpuQuarantineReason>,
    /// Copied/sealed mapped blobs across the retained CPU payload tree, not
    /// Kernel physical-page usage or host RSS.
    pub mapped_blob_bytes: u64,
    /// Returned root's guest VA descriptors; nested parked maps are omitted
    /// and these bytes are not necessarily resident.
    pub virtual_region_bytes: u64,
    pub native_outcome: Option<NativeSliceOutcome>,
    pub memory_fault: Option<UnicornMemoryFault>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CpuCustodyCounts {
    pub reserved: usize,
    pub runnable: usize,
    pub leased: usize,
    pub parked: usize,
    pub suspended_for_child: usize,
    pub quarantined: usize,
    pub reserved_unbound_trees: usize,
    pub unverified_stopped_trees: usize,
    pub abandoned_unbound_trees: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuCustodyError {
    MachineLineageExhausted,
    GenerationExhausted,
    SlotCapacityExhausted,
    DuplicateBirth,
    WrongMachine,
    StaleKey,
    WrongBirth,
    NotReserved,
    NotQuarantined,
    NotRetainedTree,
    NotAdmitted,
    NotRunnable,
    NotLeased,
    NotParked,
    NotSuspendedForChild,
    ActiveLeafBusy,
    StalePermit,
    WrongPermitKind,
    WrongSlot,
    WrongThread,
    WrongFrame,
    WrongAttempt,
    BusyCpuLoan,
    OutstandingCpuHandle,
    HandleCountExhausted,
    MissingCpu,
    CpuAlreadyBound,
    ParkedChildProcessTree,
    UnsealedQuarantine,
    KernelLineage(CpuMachineLineageError),
    StaleSelectedProcess,
    StaleParentProcess,
    StaleParentThread,
    ParentFrameValidationUnavailable,
}

impl fmt::Display for CpuCustodyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = match self {
            Self::MachineLineageExhausted => "CPU custody machine lineage exhausted",
            Self::GenerationExhausted => "CPU custody generation exhausted",
            Self::SlotCapacityExhausted => "CPU custody slot capacity exhausted",
            Self::DuplicateBirth => "process birth already has CPU custody",
            Self::WrongMachine => "CPU custody key belongs to another machine",
            Self::StaleKey => "CPU custody key is stale or vacant",
            Self::WrongBirth => "CPU custody process birth differs from the frozen birth",
            Self::NotReserved => "CPU custody slot is not reserved",
            Self::NotQuarantined => "CPU custody slot is not quarantined",
            Self::NotRetainedTree => "CPU tree holding slot has no retained payload",
            Self::NotAdmitted => "CPU custody slot has no admitted payload",
            Self::NotRunnable => "CPU custody slot is not runnable",
            Self::NotLeased => "CPU custody slot has no active run lease",
            Self::NotParked => "CPU custody slot is not parked at that wait locator",
            Self::NotSuspendedForChild => "CPU parent slot is not suspended for a child",
            Self::ActiveLeafBusy => "another CPU execution leaf is active",
            Self::StalePermit => "CPU execution leaf permit is revoked or stale",
            Self::WrongPermitKind => "CPU execution permit has the wrong authority level",
            Self::WrongSlot => "CPU execution leaf names a different slot",
            Self::WrongThread => "CPU execution leaf names a different thread birth",
            Self::WrongFrame => "CPU execution leaf names a different frame/version",
            Self::WrongAttempt => "CPU execution leaf attempt generation differs",
            Self::BusyCpuLoan => "CPU payload already has a scoped loan",
            Self::OutstandingCpuHandle => "CPU slot still has a payload handle",
            Self::HandleCountExhausted => "CPU slot handle count exhausted",
            Self::MissingCpu => "CPU custody slot has no payload",
            Self::CpuAlreadyBound => "CPU payload already has a custody binding",
            Self::ParkedChildProcessTree => {
                "CPU still owns parked child process CPUs; split the tree before admission"
            }
            Self::UnsealedQuarantine => "CPU quarantine payload has not closed its active loan",
            Self::KernelLineage(error) => return write!(f, "CPU Kernel lineage rejected: {error}"),
            Self::StaleSelectedProcess => {
                "selected process birth is not the exact live CPU custody owner"
            }
            Self::StaleParentProcess => "suspended parent process birth is no longer live/selected",
            Self::StaleParentThread => "suspended parent thread birth is no longer live/owned",
            Self::ParentFrameValidationUnavailable => {
                "suspended parent frame/version validation is not implemented"
            }
        };
        f.write_str(reason)
    }
}

impl std::error::Error for CpuCustodyError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CpuRunPermit {
    Process(CpuProcessPermit),
    Leaf(CpuLeafPermit),
}

impl CpuRunPermit {
    fn key(self) -> CpuCustodyKey {
        match self {
            Self::Process(permit) => permit.key,
            Self::Leaf(permit) => permit.key,
        }
    }
}

#[derive(Debug)]
struct LeafAuthority {
    machine_lineage: u64,
    alive: Cell<bool>,
    // One machine-wide active attempt across both authority levels. A
    // process permit must never run concurrently with a legacy leaf permit.
    active: Cell<Option<CpuRunPermit>>,
}

/// Read-only proof checker for callbacks and the machine-state port. Holding
/// it never borrows the mutable slot topology across a child run.
#[derive(Debug, Clone)]
pub struct CpuLeafVerifier {
    authority: Rc<LeafAuthority>,
}

impl CpuLeafVerifier {
    pub fn machine_lineage(&self) -> u64 {
        self.authority.machine_lineage
    }

    pub(crate) fn is_live(&self) -> bool {
        self.authority.alive.get()
    }

    /// Observe the currently directory-issued token without granting a new
    /// lease. The machine port captures this value at native entry and must
    /// revalidate the same value on every callback.
    pub(crate) fn current_permit(&self) -> Option<CpuLeafPermit> {
        self.authority
            .alive
            .get()
            .then(|| match self.authority.active.get() {
                Some(CpuRunPermit::Leaf(permit)) => Some(permit),
                _ => None,
            })?
    }

    pub fn validate(
        &self,
        permit: CpuLeafPermit,
        key: CpuCustodyKey,
        birth: ProcessBirthKey,
        thread_birth: ThreadBirthKey,
        frame: Option<CpuFrameStamp>,
    ) -> Result<(), CpuCustodyError> {
        if !self.authority.alive.get()
            || permit.key.machine_lineage != self.authority.machine_lineage
            || key.machine_lineage != self.authority.machine_lineage
        {
            return Err(CpuCustodyError::WrongMachine);
        }
        if permit.key != key {
            return Err(CpuCustodyError::WrongSlot);
        }
        if permit.birth != birth {
            return Err(CpuCustodyError::WrongBirth);
        }
        if permit.thread_birth != thread_birth {
            return Err(CpuCustodyError::WrongThread);
        }
        if permit.frame != frame {
            return Err(CpuCustodyError::WrongFrame);
        }
        match self.authority.active.get() {
            Some(CpuRunPermit::Leaf(active)) if active == permit => Ok(()),
            Some(CpuRunPermit::Leaf(active))
                if active.attempt_generation != permit.attempt_generation =>
            {
                Err(CpuCustodyError::WrongAttempt)
            }
            Some(CpuRunPermit::Process(_)) => Err(CpuCustodyError::WrongPermitKind),
            _ => Err(CpuCustodyError::StalePermit),
        }
    }
}

/// Read-only Cut 1 process/slot/attempt validator. It shares the same active
/// authority as leaf validation but cannot produce a thread/frame leaf.
#[derive(Debug, Clone)]
pub struct CpuProcessVerifier {
    authority: Rc<LeafAuthority>,
}

impl CpuProcessVerifier {
    pub fn machine_lineage(&self) -> u64 {
        self.authority.machine_lineage
    }

    pub(crate) fn current_permit(&self) -> Option<CpuProcessPermit> {
        self.authority
            .alive
            .get()
            .then(|| match self.authority.active.get() {
                Some(CpuRunPermit::Process(permit)) => Some(permit),
                _ => None,
            })?
    }

    pub fn validate(
        &self,
        permit: CpuProcessPermit,
        key: CpuCustodyKey,
        birth: ProcessBirthKey,
    ) -> Result<(), CpuCustodyError> {
        if !self.authority.alive.get()
            || permit.key.machine_lineage != self.authority.machine_lineage
            || key.machine_lineage != self.authority.machine_lineage
        {
            return Err(CpuCustodyError::WrongMachine);
        }
        if permit.key != key {
            return Err(CpuCustodyError::WrongSlot);
        }
        if permit.birth != birth {
            return Err(CpuCustodyError::WrongBirth);
        }
        match self.authority.active.get() {
            Some(CpuRunPermit::Process(active)) if active == permit => Ok(()),
            Some(CpuRunPermit::Process(active))
                if active.attempt_generation != permit.attempt_generation =>
            {
                Err(CpuCustodyError::WrongAttempt)
            }
            Some(CpuRunPermit::Leaf(_)) => Err(CpuCustodyError::WrongPermitKind),
            _ => Err(CpuCustodyError::StalePermit),
        }
    }
}

#[derive(Debug)]
struct AdmittedCpuCell {
    key: CpuCustodyKey,
    birth: ProcessBirthKey,
    authority: Rc<LeafAuthority>,
    phase: Cell<CpuAdmittedPhase>,
    cpu: RefCell<Option<UnicornMips>>,
    quarantine_sealed: Cell<bool>,
    open_handles: Cell<usize>,
}

impl AdmittedCpuCell {
    fn new(key: CpuCustodyKey, birth: ProcessBirthKey, authority: Rc<LeafAuthority>) -> Self {
        Self {
            key,
            birth,
            authority,
            phase: Cell::new(CpuAdmittedPhase::Runnable),
            cpu: RefCell::new(None),
            quarantine_sealed: Cell::new(false),
            open_handles: Cell::new(0),
        }
    }

    /// The seal mutates only fields in the already-owned payload. If a
    /// callback still has a scoped mutable loan, its drop performs the same
    /// operation before that borrow is released, including during unwind.
    fn quarantine(&self, reason: CpuQuarantineReason) {
        self.phase.set(CpuAdmittedPhase::Quarantined(reason));
        if self
            .authority
            .active
            .get()
            .is_some_and(|permit| permit.key() == self.key)
        {
            self.authority.active.set(None);
        }
        if let Ok(mut payload) = self.cpu.try_borrow_mut() {
            self.seal_borrowed_if_quarantined(&mut payload);
        }
    }

    fn seal_borrowed_if_quarantined(&self, payload: &mut Option<UnicornMips>) {
        if !matches!(self.phase.get(), CpuAdmittedPhase::Quarantined(_))
            || self.quarantine_sealed.get()
        {
            return;
        }
        if let Some(_cpu) = payload.as_mut() {
            #[cfg(feature = "unicorn")]
            _cpu.seal_for_quarantine();
            self.quarantine_sealed.set(true);
        }
    }
}

/// Test-only mutable loan for custody and unwind controls. Production must
/// expose a restricted run facade: `&mut UnicornMips` allows `mem::replace`
/// to move the admitted original out of its slot.
#[cfg(test)]
struct AdmittedCpuLoan<'a> {
    cell: &'a AdmittedCpuCell,
    payload: RefMut<'a, Option<UnicornMips>>,
}

#[cfg(test)]
impl Drop for AdmittedCpuLoan<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() && self.cell.phase.get() == CpuAdmittedPhase::Leased {
            let reason = match self.cell.authority.active.get() {
                Some(CpuRunPermit::Process(permit)) if permit.key == self.cell.key => {
                    CpuQuarantineReason::UnresolvedProcessAttempt
                }
                _ => CpuQuarantineReason::UnresolvedLeaf,
            };
            self.cell.quarantine(reason);
        }
        self.cell.seal_borrowed_if_quarantined(&mut self.payload);
    }
}

/// A stable payload locator. It owns no CPU copy and cannot itself authorize
/// guest execution; each loan checks the current directory-issued permit.
#[derive(Debug)]
pub struct CpuSlotHandle {
    cell: Rc<AdmittedCpuCell>,
    verifier: CpuLeafVerifier,
    process_verifier: CpuProcessVerifier,
}

impl CpuSlotHandle {
    pub fn key(&self) -> CpuCustodyKey {
        self.cell.key
    }

    pub fn birth(&self) -> ProcessBirthKey {
        self.cell.birth
    }

    #[cfg(test)]
    pub fn with_cpu_mut<R>(
        &self,
        permit: CpuLeafPermit,
        use_cpu: impl for<'loan> FnOnce(&'loan mut UnicornMips) -> R,
    ) -> Result<R, CpuCustodyError> {
        self.verifier.validate(
            permit,
            self.cell.key,
            self.cell.birth,
            permit.thread_birth,
            permit.frame,
        )?;
        if self.cell.phase.get() != CpuAdmittedPhase::Leased {
            return Err(CpuCustodyError::NotLeased);
        }
        let payload = self
            .cell
            .cpu
            .try_borrow_mut()
            .map_err(|_| CpuCustodyError::BusyCpuLoan)?;
        let mut loan = AdmittedCpuLoan {
            cell: &self.cell,
            payload,
        };
        let cpu = loan.payload.as_mut().ok_or(CpuCustodyError::MissingCpu)?;
        Ok(use_cpu(cpu))
    }

    /// Test-only Cut 1 loan probe. A production run must use a restricted,
    /// permit-bound facade that cannot move the original payload out.
    #[cfg(test)]
    pub fn with_process_cpu_mut<R>(
        &self,
        permit: CpuProcessPermit,
        use_cpu: impl for<'loan> FnOnce(&'loan mut UnicornMips) -> R,
    ) -> Result<R, CpuCustodyError> {
        self.process_verifier
            .validate(permit, self.cell.key, self.cell.birth)?;
        if self.cell.phase.get() != CpuAdmittedPhase::Leased {
            return Err(CpuCustodyError::NotLeased);
        }
        let payload = self
            .cell
            .cpu
            .try_borrow_mut()
            .map_err(|_| CpuCustodyError::BusyCpuLoan)?;
        let mut loan = AdmittedCpuLoan {
            cell: &self.cell,
            payload,
        };
        let cpu = loan.payload.as_mut().ok_or(CpuCustodyError::MissingCpu)?;
        Ok(use_cpu(cpu))
    }
}

impl Drop for CpuSlotHandle {
    fn drop(&mut self) {
        let remaining = self
            .cell
            .open_handles
            .get()
            .checked_sub(1)
            .expect("CPU slot handle count must include this handle");
        self.cell.open_handles.set(remaining);
    }
}

/// Non-clone attempt owner. Call `finish_attempt` or `suspend_for_child`; an
/// unresolved drop revokes its permit and quarantines the original CPU.
#[must_use = "finish or suspend the CPU leaf before returning"]
#[derive(Debug)]
pub struct CpuLeafLease {
    permit: CpuLeafPermit,
    cell: Rc<AdmittedCpuCell>,
    authority: Rc<LeafAuthority>,
    finished: bool,
}

impl CpuLeafLease {
    pub fn permit(&self) -> CpuLeafPermit {
        self.permit
    }
}

impl Drop for CpuLeafLease {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        if self.authority.active.get() == Some(CpuRunPermit::Leaf(self.permit)) {
            self.authority.active.set(None);
        }
        if self.cell.phase.get() == CpuAdmittedPhase::Leased {
            self.cell.quarantine(CpuQuarantineReason::UnresolvedLeaf);
        }
    }
}

/// Non-clone process-scoped attempt owner. A dropped unresolved attempt
/// revokes authority and retains the original CPU non-runnable in its slot.
#[must_use = "finish or suspend the process CPU attempt before returning"]
#[derive(Debug)]
pub struct CpuProcessLease {
    permit: CpuProcessPermit,
    cell: Rc<AdmittedCpuCell>,
    authority: Rc<LeafAuthority>,
    finished: bool,
}

impl CpuProcessLease {
    pub fn permit(&self) -> CpuProcessPermit {
        self.permit
    }
}

impl Drop for CpuProcessLease {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        if self.authority.active.get() == Some(CpuRunPermit::Process(self.permit)) {
            self.authority.active.set(None);
        }
        if self.cell.phase.get() == CpuAdmittedPhase::Leased {
            self.cell
                .quarantine(CpuQuarantineReason::UnresolvedProcessAttempt);
        }
    }
}

/// A suspended parent remains in its exact slot while another leaf runs.
/// The guard does not borrow the directory or the parent's CPU payload.
#[must_use = "resume the parent after classifying the child"]
#[derive(Debug)]
pub struct CpuParentSuspension {
    cell: Rc<AdmittedCpuCell>,
    authority: Rc<LeafAuthority>,
    parent_permit: CpuLeafPermit,
    resolved: bool,
}

/// A process-level parent hold. It has no thread/frame execution authority;
/// there is intentionally no success-resume API until the native tail can be
/// typed and checked in the machine-owned host-turn cut.
#[must_use = "an unresolved process parent is quarantined on drop"]
#[derive(Debug)]
pub struct CpuProcessParentSuspension {
    cell: Rc<AdmittedCpuCell>,
    parent_permit: CpuProcessPermit,
}

impl CpuProcessParentSuspension {
    pub fn parent_permit(&self) -> CpuProcessPermit {
        self.parent_permit
    }
}

impl Drop for CpuProcessParentSuspension {
    fn drop(&mut self) {
        if self.cell.phase.get() == CpuAdmittedPhase::SuspendedForChild {
            self.cell
                .quarantine(CpuQuarantineReason::UnresolvedParentSuspension);
        }
    }
}

impl Drop for CpuParentSuspension {
    fn drop(&mut self) {
        if !self.resolved && self.cell.phase.get() == CpuAdmittedPhase::SuspendedForChild {
            self.cell
                .quarantine(CpuQuarantineReason::UnresolvedParentSuspension);
        }
    }
}

#[derive(Debug)]
enum SlotState {
    Vacant,
    Reserved {
        birth: ProcessBirthKey,
        cell: Rc<AdmittedCpuCell>,
    },
    Admitted {
        birth: ProcessBirthKey,
        cell: Rc<AdmittedCpuCell>,
    },
    Quarantined {
        birth: ProcessBirthKey,
        cpu: UnicornMips,
    },
}

impl SlotState {
    fn birth(&self) -> Option<ProcessBirthKey> {
        match self {
            Self::Vacant => None,
            Self::Reserved { birth, .. }
            | Self::Admitted { birth, .. }
            | Self::Quarantined { birth, .. } => Some(*birth),
        }
    }
}

#[derive(Debug)]
struct Slot {
    generation: u64,
    state: SlotState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TreeCellPhase {
    Vacant,
    ReservedUnbound,
    UnverifiedStopped,
    Abandoned,
}

struct RetainedTreeCell {
    key: RetainedTreeKey,
    origin_birth: ProcessBirthKey,
    phase: Cell<TreeCellPhase>,
    // A unique deposit ticket is the only writer while ReservedUnbound. Cell
    // makes that transfer non-borrowing and allocation-free even if a status
    // reader examined the directory during a nested guest run.
    cpu: Cell<Option<UnicornMips>>,
}

impl fmt::Debug for RetainedTreeCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RetainedTreeCell")
            .field("key", &self.key)
            .field("origin_birth", &self.origin_birth)
            .field("phase", &self.phase.get())
            .finish_non_exhaustive()
    }
}

/// Preallocated capacity for exactly one returned CPU tree. This token is
/// deliberately neither Clone nor Copy. The directory retains the same cell,
/// so depositing consumes the ticket without a fallible directory borrow.
/// The owning `CpuCustody` is machine state and must outlive this token and
/// every retained key; dropping the machine intentionally retires its CPUs.
#[must_use = "cancel or deposit the original CPU tree before returning"]
#[derive(Debug)]
pub struct CpuDepositTicket {
    cell: Rc<RetainedTreeCell>,
}

struct RestoredTreeRead<'a> {
    cell: &'a Cell<Option<UnicornMips>>,
    cpu: Option<UnicornMips>,
}

impl Drop for RestoredTreeRead<'_> {
    fn drop(&mut self) {
        // This also restores the original owner if an inspection panics.
        self.cell.set(self.cpu.take());
    }
}

impl CpuDepositTicket {
    pub fn key(&self) -> RetainedTreeKey {
        self.cell.key
    }

    /// Cancel only after the CPU was never admitted or another durable owner
    /// has accepted it. No CPU payload exists in this cell on that path.
    pub fn cancel(self) {
        self.cell.phase.set(TreeCellPhase::Vacant);
    }

    /// Retain a whole tree whose exact, per-process ownership is not proven.
    /// Even a single-birth error path is stopped until a later checked drain;
    /// `origin_birth` is provenance, never an executable CPU label.
    pub fn deposit_unverified(self, cpu: UnicornMips) -> RetainedTreeKey {
        // Only this non-clone ticket can write the preallocated cell, and
        // consuming it prevents a second deposit. The directory has no API
        // that changes a ReservedUnbound cell while its ticket exists.
        self.cell.cpu.set(Some(cpu));
        self.cell.phase.set(TreeCellPhase::UnverifiedStopped);
        self.cell.key
    }
}

impl Drop for CpuDepositTicket {
    fn drop(&mut self) {
        // A `?` or unwind after reservation must not silently look like an
        // active, recoverable ticket. The CPU may already have been admitted,
        // so this marker is terminal and blocks checkpoint/reuse.
        if self.cell.phase.get() == TreeCellPhase::ReservedUnbound {
            self.cell.phase.set(TreeCellPhase::Abandoned);
        }
    }
}

/// Sole owner of terminal CPUs. Do not clone or include this in a CPU-only
/// checkpoint; a machine checkpoint must explicitly relineage the directory.
#[derive(Debug)]
pub struct CpuCustody {
    machine_lineage: u64,
    next_generation: u64,
    next_attempt_generation: u64,
    leaf_authority: Rc<LeafAuthority>,
    slots: Vec<Slot>,
    retained_trees: Vec<Rc<RetainedTreeCell>>,
}

impl CpuCustody {
    pub fn new() -> Result<Self, CpuCustodyError> {
        let machine_lineage = NEXT_MACHINE_LINEAGE
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map_err(|_| CpuCustodyError::MachineLineageExhausted)?;
        Ok(Self {
            machine_lineage,
            next_generation: 0,
            next_attempt_generation: 0,
            leaf_authority: Rc::new(LeafAuthority {
                machine_lineage,
                alive: Cell::new(true),
                active: Cell::new(None),
            }),
            slots: Vec::new(),
            retained_trees: Vec::new(),
        })
    }

    pub fn machine_lineage(&self) -> u64 {
        self.machine_lineage
    }

    /// A read-only validator suitable for a callback-scoped machine port.
    /// Kernel lineage and canonical Task/frame checks remain the port's job.
    pub fn leaf_verifier(&self) -> CpuLeafVerifier {
        CpuLeafVerifier {
            authority: Rc::clone(&self.leaf_authority),
        }
    }

    /// The process-only checker shares the machine-wide active attempt latch
    /// with leaf authority. It never supplies a thread or frame witness.
    pub fn process_verifier(&self) -> CpuProcessVerifier {
        CpuProcessVerifier {
            authority: Rc::clone(&self.leaf_authority),
        }
    }

    /// Reserve a whole-tree holding cell before a child enters guest code.
    /// Its origin is only a launch provenance; the returned root can be a
    /// different process after nested rotation and is not relabeled here.
    pub fn reserve_unbound(
        &mut self,
        origin_birth: ProcessBirthKey,
    ) -> Result<CpuDepositTicket, CpuCustodyError> {
        if self.retained_trees.iter().any(|cell| {
            cell.origin_birth == origin_birth && cell.phase.get() != TreeCellPhase::Vacant
        }) {
            return Err(CpuCustodyError::DuplicateBirth);
        }
        let generation = self
            .next_generation
            .checked_add(1)
            .ok_or(CpuCustodyError::GenerationExhausted)?;
        let vacant = self
            .retained_trees
            .iter()
            .position(|cell| cell.phase.get() == TreeCellPhase::Vacant);
        let index = if let Some(index) = vacant {
            index
        } else {
            self.retained_trees
                .try_reserve(1)
                .map_err(|_| CpuCustodyError::SlotCapacityExhausted)?;
            self.retained_trees.len()
        };
        let cell = Rc::new(RetainedTreeCell {
            key: RetainedTreeKey {
                machine_lineage: self.machine_lineage,
                index,
                generation,
            },
            origin_birth,
            phase: Cell::new(TreeCellPhase::ReservedUnbound),
            cpu: Cell::new(None),
        });
        if index == self.retained_trees.len() {
            self.retained_trees.push(Rc::clone(&cell));
        } else {
            self.retained_trees[index] = Rc::clone(&cell);
        }
        self.next_generation = generation;
        Ok(CpuDepositTicket { cell })
    }

    /// Inspect a retained tree without cloning or granting a run lease.
    /// The payload is returned to the same stable cell even on unwind.
    pub(crate) fn with_retained_tree<R>(
        &self,
        key: RetainedTreeKey,
        inspect: impl for<'a> FnOnce(&'a UnicornMips) -> R,
    ) -> Result<R, CpuCustodyError> {
        let cell = self.retained_tree_cell(key)?;
        if !matches!(cell.phase.get(), TreeCellPhase::UnverifiedStopped) {
            return Err(CpuCustodyError::NotRetainedTree);
        }
        let cpu = cell.cpu.take().ok_or(CpuCustodyError::NotRetainedTree)?;
        let restore = RestoredTreeRead {
            cell: &cell.cpu,
            cpu: Some(cpu),
        };
        Ok(inspect(
            restore.cpu.as_ref().expect("retained tree read owns CPU"),
        ))
    }

    fn retained_tree_cell(
        &self,
        key: RetainedTreeKey,
    ) -> Result<&RetainedTreeCell, CpuCustodyError> {
        if key.machine_lineage != self.machine_lineage {
            return Err(CpuCustodyError::WrongMachine);
        }
        let cell = self
            .retained_trees
            .get(key.index)
            .ok_or(CpuCustodyError::StaleKey)?;
        if cell.key != key || cell.phase.get() == TreeCellPhase::Vacant {
            return Err(CpuCustodyError::StaleKey);
        }
        Ok(cell)
    }

    /// Preallocate the future terminal slot before the child is constructed.
    /// No admission path needs to grow this directory after native execution.
    pub fn reserve(&mut self, birth: ProcessBirthKey) -> Result<CpuCustodyKey, CpuCustodyError> {
        if self
            .slots
            .iter()
            .any(|slot| slot.state.birth() == Some(birth))
        {
            return Err(CpuCustodyError::DuplicateBirth);
        }
        let generation = self
            .next_generation
            .checked_add(1)
            .ok_or(CpuCustodyError::GenerationExhausted)?;
        let index = match self
            .slots
            .iter()
            .position(|slot| matches!(&slot.state, SlotState::Vacant))
        {
            Some(index) => index,
            None => {
                self.slots
                    .try_reserve(1)
                    .map_err(|_| CpuCustodyError::SlotCapacityExhausted)?;
                let index = self.slots.len();
                let key = CpuCustodyKey {
                    machine_lineage: self.machine_lineage,
                    index,
                    generation,
                };
                self.slots.push(Slot {
                    generation,
                    state: SlotState::Reserved {
                        birth,
                        cell: Rc::new(AdmittedCpuCell::new(
                            key,
                            birth,
                            Rc::clone(&self.leaf_authority),
                        )),
                    },
                });
                self.next_generation = generation;
                return Ok(key);
            }
        };
        let key = CpuCustodyKey {
            machine_lineage: self.machine_lineage,
            index,
            generation,
        };
        self.slots[index] = Slot {
            generation,
            state: SlotState::Reserved {
                birth,
                cell: Rc::new(AdmittedCpuCell::new(
                    key,
                    birth,
                    Rc::clone(&self.leaf_authority),
                )),
            },
        };
        self.next_generation = generation;
        Ok(key)
    }

    /// Deposit the original CPU into its preallocated exact-birth cell before
    /// any guest instruction. Every refusal returns that same CPU to the
    /// caller; no allocation is needed at this transfer boundary.
    pub fn admit_reserved(
        &mut self,
        key: CpuCustodyKey,
        birth: ProcessBirthKey,
        cpu: UnicornMips,
    ) -> Result<(), (CpuCustodyError, UnicornMips)> {
        #[cfg(feature = "unicorn")]
        let verifier = self.leaf_verifier();
        let slot = match self.slot_mut(key) {
            Ok(slot) => slot,
            Err(error) => return Err((error, cpu)),
        };
        let cell = match &slot.state {
            SlotState::Reserved {
                birth: frozen,
                cell,
            } if *frozen == birth => Rc::clone(cell),
            SlotState::Reserved { .. } => return Err((CpuCustodyError::WrongBirth, cpu)),
            _ => return Err((CpuCustodyError::NotReserved, cpu)),
        };
        // One admitted slot names one process CPU. A legacy parked subtree
        // still owns other process CPUs, so accepting it here would hide
        // those leaves behind this slot's single-process permit.
        if cpu.has_parked_child_processes() {
            return Err((CpuCustodyError::ParkedChildProcessTree, cpu));
        }
        let Ok(mut payload) = cell.cpu.try_borrow_mut() else {
            return Err((CpuCustodyError::BusyCpuLoan, cpu));
        };
        if payload.is_some() {
            return Err((CpuCustodyError::NotReserved, cpu));
        }
        #[cfg(feature = "unicorn")]
        let mut cpu = cpu;
        #[cfg(feature = "unicorn")]
        if let Err(error) = cpu.bind_cpu_custody(key, birth, verifier) {
            return Err((error, cpu));
        }
        *payload = Some(cpu);
        drop(payload);
        cell.phase.set(CpuAdmittedPhase::Runnable);
        slot.state = SlotState::Admitted { birth, cell };
        Ok(())
    }

    pub fn admitted_key_for_birth(&self, birth: ProcessBirthKey) -> Option<CpuCustodyKey> {
        self.slots.iter().enumerate().find_map(|(index, slot)| {
            matches!(&slot.state, SlotState::Admitted { birth: owner, .. } if *owner == birth)
                .then_some(CpuCustodyKey {
                    machine_lineage: self.machine_lineage,
                    index,
                    generation: slot.generation,
                })
        })
    }

    pub fn admitted_phase(
        &self,
        key: CpuCustodyKey,
        birth: ProcessBirthKey,
    ) -> Result<CpuAdmittedPhase, CpuCustodyError> {
        let cell = self.admitted_cell(key, birth)?;
        Ok(cell.phase.get())
    }

    /// Obtain a stable handle without borrowing directory topology. The
    /// handle cannot execute without a separately issued current leaf.
    pub fn acquire_cpu_handle(
        &self,
        key: CpuCustodyKey,
        birth: ProcessBirthKey,
    ) -> Result<CpuSlotHandle, CpuCustodyError> {
        let cell = self.admitted_cell(key, birth)?;
        let next = cell
            .open_handles
            .get()
            .checked_add(1)
            .ok_or(CpuCustodyError::HandleCountExhausted)?;
        cell.open_handles.set(next);
        Ok(CpuSlotHandle {
            cell: Rc::clone(cell),
            verifier: self.leaf_verifier(),
            process_verifier: self.process_verifier(),
        })
    }

    /// Issue Cut 1 process/slot/attempt authority only for the exact selected,
    /// execution-live birth in the Kernel bound to this directory. A selected
    /// PID alone is never the authority: the caller supplies a checked slot
    /// and the directory mints the non-wrapping attempt generation.
    pub fn issue_process_attempt(
        &mut self,
        kernel: &CeKernel,
        key: CpuCustodyKey,
        birth: ProcessBirthKey,
    ) -> Result<CpuProcessLease, CpuCustodyError> {
        kernel
            .require_cpu_machine_lineage(self.machine_lineage)
            .map_err(CpuCustodyError::KernelLineage)?;
        if kernel.current_process_birth_key() != Some(birth)
            || !kernel.handles.process_execution_live_by_birth(birth)
        {
            return Err(CpuCustodyError::StaleSelectedProcess);
        }
        if self.leaf_authority.active.get().is_some() {
            return Err(CpuCustodyError::ActiveLeafBusy);
        }
        let cell = Rc::clone(self.admitted_cell(key, birth)?);
        if cell.phase.get() != CpuAdmittedPhase::Runnable {
            return Err(CpuCustodyError::NotRunnable);
        }
        let payload = cell
            .cpu
            .try_borrow_mut()
            .map_err(|_| CpuCustodyError::BusyCpuLoan)?;
        if payload.is_none() {
            return Err(CpuCustodyError::MissingCpu);
        }
        drop(payload);
        let generation = self
            .next_attempt_generation
            .checked_add(1)
            .ok_or(CpuCustodyError::GenerationExhausted)?;
        let permit = CpuProcessPermit {
            key,
            birth,
            attempt_generation: generation,
        };
        self.next_attempt_generation = generation;
        cell.phase.set(CpuAdmittedPhase::Leased);
        self.leaf_authority
            .active
            .set(Some(CpuRunPermit::Process(permit)));
        Ok(CpuProcessLease {
            permit,
            cell,
            authority: Rc::clone(&self.leaf_authority),
            finished: false,
        })
    }

    /// Issue the only executable leaf from an exact admitted runnable slot.
    /// The attempt generation is minted here, never supplied by a caller.
    /// Kernel lineage, live thread ownership and frame-version checks must be
    /// performed by the machine port before issue and again before use.
    pub fn issue_leaf(
        &mut self,
        key: CpuCustodyKey,
        birth: ProcessBirthKey,
        thread_birth: ThreadBirthKey,
        frame: Option<CpuFrameStamp>,
    ) -> Result<CpuLeafLease, CpuCustodyError> {
        if self.leaf_authority.active.get().is_some() {
            return Err(CpuCustodyError::ActiveLeafBusy);
        }
        if frame.is_some_and(|stamp| stamp.key.owner_birth() != thread_birth) {
            return Err(CpuCustodyError::WrongFrame);
        }
        let cell = Rc::clone(self.admitted_cell(key, birth)?);
        if cell.phase.get() != CpuAdmittedPhase::Runnable {
            return Err(CpuCustodyError::NotRunnable);
        }
        let payload = cell
            .cpu
            .try_borrow_mut()
            .map_err(|_| CpuCustodyError::BusyCpuLoan)?;
        if payload.is_none() {
            return Err(CpuCustodyError::MissingCpu);
        }
        drop(payload);
        let generation = self
            .next_attempt_generation
            .checked_add(1)
            .ok_or(CpuCustodyError::GenerationExhausted)?;
        let permit = CpuLeafPermit {
            key,
            birth,
            thread_birth,
            frame,
            attempt_generation: generation,
        };
        self.next_attempt_generation = generation;
        cell.phase.set(CpuAdmittedPhase::Leased);
        self.leaf_authority
            .active
            .set(Some(CpuRunPermit::Leaf(permit)));
        Ok(CpuLeafLease {
            permit,
            cell,
            authority: Rc::clone(&self.leaf_authority),
            finished: false,
        })
    }

    /// Revoke the previous thread's token before another instruction or
    /// register transition. The replacement nonce is directory-minted.
    pub fn update_leaf_thread(
        &mut self,
        lease: &mut CpuLeafLease,
        thread_birth: ThreadBirthKey,
        frame: Option<CpuFrameStamp>,
    ) -> Result<CpuLeafPermit, CpuCustodyError> {
        self.verify_lease(lease)?;
        if frame.is_some_and(|stamp| stamp.key.owner_birth() != thread_birth) {
            return Err(CpuCustodyError::WrongFrame);
        }
        let generation = self
            .next_attempt_generation
            .checked_add(1)
            .ok_or(CpuCustodyError::GenerationExhausted)?;
        let next = CpuLeafPermit {
            thread_birth,
            frame,
            attempt_generation: generation,
            ..lease.permit
        };
        self.next_attempt_generation = generation;
        self.leaf_authority
            .active
            .set(Some(CpuRunPermit::Leaf(next)));
        lease.permit = next;
        Ok(next)
    }

    /// End a leased CPU from the actual backend result. The caller's proposed
    /// continuation applies only to a successful, entered, non-exit attempt;
    /// it cannot override an error or a terminal/unverified report. Returning
    /// the original execution preserves its error and raw observations for
    /// the host-turn classifier. An accepted exit must be retired separately
    /// with a Kernel witness after this lease and all payload loans close.
    pub fn finish_attempt(
        &mut self,
        lease: &mut CpuLeafLease,
        execution: AttemptExecution,
        successful_end: CpuLeaseEnd,
    ) -> Result<AttemptExecution, CpuCustodyError> {
        let end = Self::classify_attempt_end(
            &execution,
            successful_end,
            CpuQuarantineReason::UnresolvedLeaf,
        );
        match self.finish_leaf_classified(lease, end) {
            Ok(()) => Ok(execution),
            Err(error @ (CpuCustodyError::WrongThread | CpuCustodyError::WrongFrame)) => {
                // Even a bad caller-supplied parked locator cannot strand
                // the original payload in Leased or make it executable.
                self.finish_leaf_classified(
                    lease,
                    CpuLeaseEnd::Quarantined(CpuQuarantineReason::PortFailure),
                )?;
                Err(error)
            }
            Err(error) => Err(error),
        }
    }

    /// Test-only classifier pending a backend report bound to this issued
    /// process permit. A diagnostic `AttemptExecution` alone is not authority
    /// to resume a production CPU.
    ///
    /// Cut 1 finish uses the actual result and native report, then checks the
    /// live Kernel/directory pair before any successful runnable/parked state.
    /// `Parked` currently accepts only a canonical, still-blocked finite-Sleep
    /// frame; other wait families have no general Task/frame verifier here and
    /// must remain unported. A malformed locator is quarantined, not Leased.
    #[cfg(test)]
    pub fn finish_process_attempt(
        &mut self,
        lease: &mut CpuProcessLease,
        kernel: &CeKernel,
        execution: AttemptExecution,
        successful_end: CpuLeaseEnd,
    ) -> Result<AttemptExecution, CpuCustodyError> {
        self.verify_process_lease(lease)?;
        let end = Self::classify_attempt_end(
            &execution,
            successful_end,
            CpuQuarantineReason::UnresolvedProcessAttempt,
        );
        let pairing = kernel
            .require_cpu_machine_lineage(self.machine_lineage)
            .map_err(CpuCustodyError::KernelLineage);
        let selected = if !matches!(end, CpuLeaseEnd::Quarantined(_))
            && (kernel.current_process_birth_key() != Some(lease.permit.birth)
                || !kernel
                    .handles
                    .process_execution_live_by_birth(lease.permit.birth))
        {
            Err(CpuCustodyError::StaleSelectedProcess)
        } else {
            Ok(())
        };
        if let Err(error) = pairing.and(selected) {
            self.finish_process_classified(
                lease,
                kernel,
                CpuLeaseEnd::Quarantined(CpuQuarantineReason::PortFailure),
            )?;
            return Err(error);
        }
        if !matches!(end, CpuLeaseEnd::Quarantined(_)) {
            let loan_error = match lease.cell.cpu.try_borrow_mut() {
                Ok(payload) if payload.is_none() => Some(CpuCustodyError::MissingCpu),
                Ok(_) => None,
                Err(_) => Some(CpuCustodyError::BusyCpuLoan),
            };
            if let Some(error) = loan_error {
                self.finish_process_classified(
                    lease,
                    kernel,
                    CpuLeaseEnd::Quarantined(CpuQuarantineReason::PortFailure),
                )?;
                return Err(error);
            }
        }
        match self.finish_process_classified(lease, kernel, end) {
            Ok(()) => Ok(execution),
            Err(error @ (CpuCustodyError::WrongThread | CpuCustodyError::WrongFrame)) => {
                self.finish_process_classified(
                    lease,
                    kernel,
                    CpuLeaseEnd::Quarantined(CpuQuarantineReason::PortFailure),
                )?;
                Err(error)
            }
            Err(error) => Err(error),
        }
    }

    fn classify_attempt_end(
        execution: &AttemptExecution,
        successful_end: CpuLeaseEnd,
        unresolved_reason: CpuQuarantineReason,
    ) -> CpuLeaseEnd {
        if execution
            .result
            .as_ref()
            .is_err_and(|error| error.is_host_exit_recovery_required())
        {
            CpuLeaseEnd::Quarantined(CpuQuarantineReason::RecoveryRequired)
        } else if execution.report.final_outcome() == NativeSliceOutcome::NativeError {
            CpuLeaseEnd::Quarantined(CpuQuarantineReason::NativeFault)
        } else if execution.result.is_err()
            || execution.report.final_outcome() == NativeSliceOutcome::BackendError
        {
            CpuLeaseEnd::Quarantined(CpuQuarantineReason::BackendFailure)
        } else if execution.report.attempt_id().is_none()
            || matches!(
                execution.report.final_outcome(),
                NativeSliceOutcome::NoAttempt | NativeSliceOutcome::GuestProcessExit
            )
        {
            // A successful preflight-only action or a reported guest exit is
            // not proof of a resumable permit or accepted exact-birth exit.
            CpuLeaseEnd::Quarantined(unresolved_reason)
        } else {
            successful_end
        }
    }

    #[cfg(test)]
    fn finish_process_classified(
        &mut self,
        lease: &mut CpuProcessLease,
        kernel: &CeKernel,
        end: CpuLeaseEnd,
    ) -> Result<(), CpuCustodyError> {
        self.verify_process_lease(lease)?;
        let phase = match end {
            CpuLeaseEnd::Runnable => CpuAdmittedPhase::Runnable,
            CpuLeaseEnd::Parked(locator) => {
                let owner = locator.key().owner_birth();
                if !kernel.is_live_thread_birth(owner)
                    || kernel
                        .handles
                        .live_thread_creation_owner_by_id(owner.thread_id())
                        != Some(lease.permit.birth)
                {
                    return Err(CpuCustodyError::WrongThread);
                }
                // Only migrated finite Sleep has both an exact scheduler
                // frame reference and a canonical blocked-phase predicate.
                // Do not infer other wait authority from a raw frame value.
                if kernel
                    .scheduler
                    .blocked_wait(locator.wait_id())
                    .is_none_or(|wait| {
                        wait.owner_birth != Some(owner) || wait.frame_ref != Some(locator)
                    })
                    || !kernel.is_blocked_finite_sleep_frame(locator)
                {
                    return Err(CpuCustodyError::WrongFrame);
                }
                CpuAdmittedPhase::Parked(locator)
            }
            CpuLeaseEnd::Quarantined(reason) => CpuAdmittedPhase::Quarantined(reason),
        };
        if let CpuAdmittedPhase::Quarantined(reason) = phase {
            lease.cell.quarantine(reason);
        } else {
            lease.cell.phase.set(phase);
        }
        self.leaf_authority.active.set(None);
        lease.finished = true;
        Ok(())
    }

    /// Test-only primitive for exercising directory phase and permit checks.
    /// Production callers must classify the actual `AttemptExecution` above.
    #[cfg(test)]
    pub fn finish_leaf(
        &mut self,
        lease: &mut CpuLeafLease,
        end: CpuLeaseEnd,
    ) -> Result<(), CpuCustodyError> {
        self.finish_leaf_classified(lease, end)
    }

    fn finish_leaf_classified(
        &mut self,
        lease: &mut CpuLeafLease,
        end: CpuLeaseEnd,
    ) -> Result<(), CpuCustodyError> {
        self.verify_lease(lease)?;
        let phase = match end {
            CpuLeaseEnd::Runnable => CpuAdmittedPhase::Runnable,
            CpuLeaseEnd::Parked(locator) => {
                if locator.key().owner_birth() != lease.permit.thread_birth {
                    return Err(CpuCustodyError::WrongThread);
                }
                if lease
                    .permit
                    .frame
                    .is_some_and(|stamp| stamp.key != locator.key())
                {
                    return Err(CpuCustodyError::WrongFrame);
                }
                CpuAdmittedPhase::Parked(locator)
            }
            CpuLeaseEnd::Quarantined(reason) => CpuAdmittedPhase::Quarantined(reason),
        };
        if let CpuAdmittedPhase::Quarantined(reason) = phase {
            lease.cell.quarantine(reason);
        } else {
            lease.cell.phase.set(phase);
        }
        self.leaf_authority.active.set(None);
        lease.finished = true;
        Ok(())
    }

    /// Revoke A only after its exterior CPU payload loan has closed. B then
    /// receives a distinct permit from `issue_leaf`; a native tail that must
    /// survive B needs a separate typed slot-owned continuation.
    pub fn suspend_for_child(
        &mut self,
        lease: &mut CpuLeafLease,
    ) -> Result<CpuParentSuspension, CpuCustodyError> {
        self.verify_lease(lease)?;
        let payload = lease
            .cell
            .cpu
            .try_borrow_mut()
            .map_err(|_| CpuCustodyError::BusyCpuLoan)?;
        if payload.is_none() {
            return Err(CpuCustodyError::MissingCpu);
        }
        drop(payload);
        lease.cell.phase.set(CpuAdmittedPhase::SuspendedForChild);
        self.leaf_authority.active.set(None);
        lease.finished = true;
        Ok(CpuParentSuspension {
            cell: Rc::clone(&lease.cell),
            authority: Rc::clone(&self.leaf_authority),
            parent_permit: lease.permit,
            resolved: false,
        })
    }

    /// Revoke A's process permit without leaving a scoped exterior payload
    /// loan across B. The still-open native tail is not represented by this
    /// bounded foundation, so dropping the hold fails closed until that
    /// continuation is source-gated in the connected host turn.
    pub fn suspend_process_for_child(
        &mut self,
        lease: &mut CpuProcessLease,
    ) -> Result<CpuProcessParentSuspension, CpuCustodyError> {
        self.verify_process_lease(lease)?;
        let payload = lease
            .cell
            .cpu
            .try_borrow_mut()
            .map_err(|_| CpuCustodyError::BusyCpuLoan)?;
        if payload.is_none() {
            return Err(CpuCustodyError::MissingCpu);
        }
        drop(payload);
        lease.cell.phase.set(CpuAdmittedPhase::SuspendedForChild);
        self.leaf_authority.active.set(None);
        lease.finished = true;
        Ok(CpuProcessParentSuspension {
            cell: Rc::clone(&lease.cell),
            parent_permit: lease.permit,
        })
    }

    /// The caller cannot self-supply a replacement parent tuple. A's original
    /// issued permit freezes its birth/thread/frame, and the live Kernel must
    /// authenticate that original lineage before the slot becomes runnable.
    pub fn resume_parent(
        &mut self,
        suspension: &mut CpuParentSuspension,
        kernel: &CeKernel,
    ) -> Result<(), CpuCustodyError> {
        if !Rc::ptr_eq(&self.leaf_authority, &suspension.authority) {
            return Err(CpuCustodyError::WrongMachine);
        }
        if self.leaf_authority.active.get().is_some() {
            return Err(CpuCustodyError::ActiveLeafBusy);
        }
        let cell = self.admitted_cell(suspension.cell.key, suspension.cell.birth)?;
        if !Rc::ptr_eq(cell, &suspension.cell) {
            return Err(CpuCustodyError::StaleKey);
        }
        if cell.phase.get() != CpuAdmittedPhase::SuspendedForChild || suspension.resolved {
            return Err(CpuCustodyError::NotSuspendedForChild);
        }
        kernel
            .require_cpu_machine_lineage(self.machine_lineage)
            .map_err(CpuCustodyError::KernelLineage)?;
        let parent = suspension.parent_permit;
        if parent.key != cell.key
            || parent.birth != cell.birth
            || kernel.current_process_birth_key() != Some(parent.birth)
            || !kernel.handles.process_execution_live_by_birth(parent.birth)
        {
            return Err(CpuCustodyError::StaleParentProcess);
        }
        if !kernel.is_live_thread_birth(parent.thread_birth)
            || kernel
                .handles
                .live_thread_creation_owner_by_id(parent.thread_birth.thread_id())
                != Some(parent.birth)
        {
            return Err(CpuCustodyError::StaleParentThread);
        }
        if parent.frame.is_some() {
            return Err(CpuCustodyError::ParentFrameValidationUnavailable);
        }
        cell.phase.set(CpuAdmittedPhase::Runnable);
        suspension.resolved = true;
        Ok(())
    }

    /// Test-only cached-phase transition. Production wake requires a typed
    /// canonical completed-wait witness, not only a matching cached locator.
    #[cfg(test)]
    pub fn wake_parked(
        &mut self,
        key: CpuCustodyKey,
        birth: ProcessBirthKey,
        locator: TaskFrameRef,
    ) -> Result<(), CpuCustodyError> {
        let cell = self.admitted_cell(key, birth)?;
        if cell.phase.get() != CpuAdmittedPhase::Parked(locator) {
            return Err(CpuCustodyError::NotParked);
        }
        cell.phase.set(CpuAdmittedPhase::Runnable);
        Ok(())
    }

    /// Move the original failed CPU into its pre-reserved slot. An authentic
    /// reservation has no allocation, borrow, or fallible native operation at
    /// this transfer boundary. A stale caller gets its CPU back, never dropped.
    pub fn quarantine_reserved(
        &mut self,
        key: CpuCustodyKey,
        birth: ProcessBirthKey,
        cpu: UnicornMips,
    ) -> Result<(), (CpuCustodyError, UnicornMips)> {
        let slot = match self.slot_mut(key) {
            Ok(slot) => slot,
            Err(error) => return Err((error, cpu)),
        };
        match &slot.state {
            SlotState::Reserved { birth: frozen, .. } if *frozen == birth => {
                // The directory owns the terminal invariant, not whichever
                // producer happened to transfer this exact admitted CPU.
                #[cfg(feature = "unicorn")]
                let mut cpu = cpu;
                #[cfg(feature = "unicorn")]
                cpu.seal_for_quarantine();
                slot.state = SlotState::Quarantined { birth, cpu };
                Ok(())
            }
            SlotState::Reserved { .. } => Err((CpuCustodyError::WrongBirth, cpu)),
            _ => Err((CpuCustodyError::NotReserved, cpu)),
        }
    }

    /// Cancel a reservation that never admitted a CPU (pre-CPU refusal or a
    /// successfully runnable child that was published elsewhere).
    pub fn cancel_reservation(
        &mut self,
        key: CpuCustodyKey,
        birth: ProcessBirthKey,
    ) -> Result<(), CpuCustodyError> {
        let slot = self.slot_mut(key)?;
        match &slot.state {
            SlotState::Reserved { birth: frozen, .. } if *frozen == birth => {
                slot.state = SlotState::Vacant;
                Ok(())
            }
            SlotState::Reserved { .. } => Err(CpuCustodyError::WrongBirth),
            _ => Err(CpuCustodyError::NotReserved),
        }
    }

    /// Borrow only immutable CPU state for trusted exact-birth exit preflight.
    /// The closure cannot return a borrowed CPU reference; callers must not
    /// clone the wrapper into a new, unowned execution path.
    pub(crate) fn with_quarantined<R>(
        &self,
        key: CpuCustodyKey,
        birth: ProcessBirthKey,
        inspect: impl for<'a> FnOnce(&'a UnicornMips) -> R,
    ) -> Result<R, CpuCustodyError> {
        let slot = self.slot(key)?;
        match &slot.state {
            SlotState::Quarantined { birth: frozen, cpu } if *frozen == birth => Ok(inspect(cpu)),
            SlotState::Quarantined { .. } => Err(CpuCustodyError::WrongBirth),
            SlotState::Admitted {
                birth: frozen,
                cell,
            } if *frozen == birth => {
                if !matches!(cell.phase.get(), CpuAdmittedPhase::Quarantined(_)) {
                    return Err(CpuCustodyError::NotQuarantined);
                }
                if !cell.quarantine_sealed.get() {
                    return Err(CpuCustodyError::UnsealedQuarantine);
                }
                let loan = cell
                    .cpu
                    .try_borrow()
                    .map_err(|_| CpuCustodyError::BusyCpuLoan)?;
                Ok(inspect(loan.as_ref().ok_or(CpuCustodyError::MissingCpu)?))
            }
            SlotState::Admitted { .. } => Err(CpuCustodyError::WrongBirth),
            _ => Err(CpuCustodyError::NotQuarantined),
        }
    }

    /// Legacy terminal transfer after Kernel has minted an accepted exact-birth
    /// completion witness. An explicitly bound witness must name this directory;
    /// an unbound witness is retained only for the pre-directory production
    /// route. Bound callers must use `retire_with_bound_accepted_exit` instead.
    /// An error leaves the slot and payload untouched.
    pub(crate) fn retire_with_accepted_exit(
        &mut self,
        key: CpuCustodyKey,
        accepted_exit: AcceptedProcessExit,
    ) -> Result<UnicornMips, CpuCustodyError> {
        self.retire_quarantined_with_witness(key, accepted_exit, false)
            .map_err(|(error, _witness)| error)
    }

    /// Connected terminal transfer. A live Kernel/directory pair and a witness
    /// issued while that pair was bound are both required. Every refusal returns
    /// the unique, non-Clone witness so the caller can resolve a held resource
    /// conflict without asking Kernel to commit the same exit twice.
    pub(crate) fn retire_with_bound_accepted_exit(
        &mut self,
        kernel: &CeKernel,
        key: CpuCustodyKey,
        accepted_exit: AcceptedProcessExit,
    ) -> Result<UnicornMips, (CpuCustodyError, AcceptedProcessExit)> {
        if let Err(error) = kernel.require_cpu_machine_lineage(self.machine_lineage) {
            return Err((CpuCustodyError::KernelLineage(error), accepted_exit));
        }
        self.retire_quarantined_with_witness(key, accepted_exit, true)
    }

    fn retire_quarantined_with_witness(
        &mut self,
        key: CpuCustodyKey,
        accepted_exit: AcceptedProcessExit,
        require_bound_witness: bool,
    ) -> Result<UnicornMips, (CpuCustodyError, AcceptedProcessExit)> {
        match accepted_exit.cpu_machine_lineage() {
            Some(lineage) if lineage == self.machine_lineage => {}
            Some(lineage) => {
                return Err((
                    CpuCustodyError::KernelLineage(CpuMachineLineageError::WrongMachine {
                        bound: lineage,
                        requested: self.machine_lineage,
                    }),
                    accepted_exit,
                ));
            }
            None if require_bound_witness => {
                return Err((
                    CpuCustodyError::KernelLineage(CpuMachineLineageError::Unbound),
                    accepted_exit,
                ));
            }
            None => {}
        }
        let slot = match self.slot_mut(key) {
            Ok(slot) => slot,
            Err(error) => return Err((error, accepted_exit)),
        };
        match &slot.state {
            SlotState::Quarantined { birth, .. } if *birth == accepted_exit.birth() => {}
            SlotState::Quarantined { .. } => {
                return Err((CpuCustodyError::WrongBirth, accepted_exit));
            }
            _ => return Err((CpuCustodyError::NotQuarantined, accepted_exit)),
        }
        match std::mem::replace(&mut slot.state, SlotState::Vacant) {
            SlotState::Quarantined { cpu, .. } => Ok(cpu),
            _ => unreachable!("checked quarantined slot changed before terminal transfer"),
        }
    }

    /// A separate admitted-slot retirement keeps the unique accepted-exit
    /// witness on every refusal. The caller can release a held loan/handle
    /// and retry without asking Kernel to commit the same exit twice.
    pub(crate) fn retire_admitted_with_accepted_exit(
        &mut self,
        key: CpuCustodyKey,
        accepted_exit: AcceptedProcessExit,
    ) -> Result<UnicornMips, (CpuCustodyError, AcceptedProcessExit)> {
        match accepted_exit.cpu_machine_lineage() {
            Some(lineage) if lineage == self.machine_lineage => {}
            Some(lineage) => {
                return Err((
                    CpuCustodyError::KernelLineage(CpuMachineLineageError::WrongMachine {
                        bound: lineage,
                        requested: self.machine_lineage,
                    }),
                    accepted_exit,
                ));
            }
            None => {
                return Err((
                    CpuCustodyError::KernelLineage(CpuMachineLineageError::Unbound),
                    accepted_exit,
                ));
            }
        }
        let cell = match self.admitted_cell(key, accepted_exit.birth()) {
            Ok(cell) => Rc::clone(cell),
            Err(error) => return Err((error, accepted_exit)),
        };
        if !matches!(
            cell.phase.get(),
            CpuAdmittedPhase::Runnable
                | CpuAdmittedPhase::Parked(_)
                | CpuAdmittedPhase::Quarantined(_)
        ) {
            return Err((CpuCustodyError::NotQuarantined, accepted_exit));
        }
        if self
            .leaf_authority
            .active
            .get()
            .is_some_and(|permit| permit.key() == key)
        {
            return Err((CpuCustodyError::ActiveLeafBusy, accepted_exit));
        }
        let mut payload = match cell.cpu.try_borrow_mut() {
            Ok(payload) => payload,
            Err(_) => return Err((CpuCustodyError::BusyCpuLoan, accepted_exit)),
        };
        if cell.open_handles.get() != 0 {
            return Err((CpuCustodyError::OutstandingCpuHandle, accepted_exit));
        }
        if matches!(cell.phase.get(), CpuAdmittedPhase::Quarantined(_))
            && !cell.quarantine_sealed.get()
        {
            return Err((CpuCustodyError::UnsealedQuarantine, accepted_exit));
        }
        let cpu = match payload.take() {
            Some(cpu) => cpu,
            None => return Err((CpuCustodyError::MissingCpu, accepted_exit)),
        };
        drop(payload);
        self.slot_mut(key)
            .expect("verified admitted CPU slot cannot change before retirement")
            .state = SlotState::Vacant;
        Ok(cpu)
    }

    pub fn snapshot_exact(
        &self,
        key: CpuCustodyKey,
        birth: ProcessBirthKey,
    ) -> Result<CpuCustodySnapshot, CpuCustodyError> {
        let slot = self.slot(key)?;
        if slot.state.birth() != Some(birth) {
            return Err(CpuCustodyError::WrongBirth);
        }
        Ok(Self::snapshot_slot(key, slot).expect("checked occupied custody slot"))
    }

    pub fn snapshots(&self) -> Vec<CpuCustodySnapshot> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                Self::snapshot_slot(
                    CpuCustodyKey {
                        machine_lineage: self.machine_lineage,
                        index,
                        generation: slot.generation,
                    },
                    slot,
                )
            })
            .collect()
    }

    /// The origin is a reservation witness, not necessarily the root CPU's
    /// process birth. Unverified trees never report a verified owner birth.
    pub fn retained_tree_snapshots(&self) -> Vec<RetainedTreeSnapshot> {
        self.retained_trees
            .iter()
            .filter_map(|cell| {
                let state = match cell.phase.get() {
                    TreeCellPhase::Vacant => return None,
                    TreeCellPhase::ReservedUnbound => RetainedTreeState::ReservedUnbound,
                    TreeCellPhase::UnverifiedStopped => RetainedTreeState::UnverifiedStopped,
                    TreeCellPhase::Abandoned => RetainedTreeState::Abandoned,
                };
                let metrics = self.with_retained_tree(cell.key, |cpu| {
                    (
                        Self::blob_bytes(cpu),
                        CpuBackend::memory(cpu)
                            .regions()
                            .map(|region| u64::from(region.size))
                            .fold(0, u64::saturating_add),
                    )
                });
                let (metrics_available, mapped_blob_bytes, root_virtual_region_bytes) =
                    match metrics {
                        Ok((blobs, regions)) => (true, blobs, regions),
                        Err(_) => (false, 0, 0),
                    };
                Some(RetainedTreeSnapshot {
                    key: cell.key,
                    origin_birth: cell.origin_birth,
                    state,
                    metrics_available,
                    mapped_blob_bytes,
                    root_virtual_region_bytes,
                })
            })
            .collect()
    }

    pub fn counts(&self) -> CpuCustodyCounts {
        let mut counts = CpuCustodyCounts::default();
        for slot in &self.slots {
            match &slot.state {
                SlotState::Vacant => {}
                SlotState::Reserved { .. } => counts.reserved += 1,
                SlotState::Quarantined { .. } => counts.quarantined += 1,
                SlotState::Admitted { cell, .. } => match cell.phase.get() {
                    CpuAdmittedPhase::Runnable => counts.runnable += 1,
                    CpuAdmittedPhase::Leased => counts.leased += 1,
                    CpuAdmittedPhase::Parked(_) => counts.parked += 1,
                    CpuAdmittedPhase::SuspendedForChild => counts.suspended_for_child += 1,
                    CpuAdmittedPhase::Quarantined(_) => counts.quarantined += 1,
                },
            }
        }
        for cell in &self.retained_trees {
            match cell.phase.get() {
                TreeCellPhase::Vacant => {}
                TreeCellPhase::ReservedUnbound => counts.reserved_unbound_trees += 1,
                TreeCellPhase::UnverifiedStopped => counts.unverified_stopped_trees += 1,
                TreeCellPhase::Abandoned => counts.abandoned_unbound_trees += 1,
            }
        }
        counts
    }

    pub fn has_any_custody(&self) -> bool {
        self.slots
            .iter()
            .any(|slot| !matches!(&slot.state, SlotState::Vacant))
            || self
                .retained_trees
                .iter()
                .any(|cell| cell.phase.get() != TreeCellPhase::Vacant)
    }

    /// Only CPU-owned mapped blobs; excludes Kernel stack/private/shared pages.
    pub fn retained_cpu_blob_bytes(&self) -> u64 {
        let quarantined = self
            .slots
            .iter()
            .filter_map(|slot| match &slot.state {
                SlotState::Quarantined { cpu, .. } => Some(Self::blob_bytes(cpu)),
                SlotState::Admitted { cell, .. } => cell
                    .cpu
                    .try_borrow()
                    .ok()
                    .and_then(|payload| payload.as_ref().map(Self::blob_bytes)),
                _ => None,
            })
            .fold(0, u64::saturating_add);
        self.retained_tree_snapshots()
            .iter()
            .filter(|snapshot| snapshot.metrics_available)
            .fold(quarantined, |total, snapshot| {
                total.saturating_add(snapshot.mapped_blob_bytes)
            })
    }

    fn slot(&self, key: CpuCustodyKey) -> Result<&Slot, CpuCustodyError> {
        if key.machine_lineage != self.machine_lineage {
            return Err(CpuCustodyError::WrongMachine);
        }
        let slot = self.slots.get(key.index).ok_or(CpuCustodyError::StaleKey)?;
        if slot.generation != key.generation || matches!(&slot.state, SlotState::Vacant) {
            return Err(CpuCustodyError::StaleKey);
        }
        Ok(slot)
    }

    fn admitted_cell(
        &self,
        key: CpuCustodyKey,
        birth: ProcessBirthKey,
    ) -> Result<&Rc<AdmittedCpuCell>, CpuCustodyError> {
        match &self.slot(key)?.state {
            SlotState::Admitted {
                birth: frozen,
                cell,
            } if *frozen == birth => Ok(cell),
            SlotState::Admitted { .. } => Err(CpuCustodyError::WrongBirth),
            _ => Err(CpuCustodyError::NotAdmitted),
        }
    }

    fn verify_lease(&self, lease: &CpuLeafLease) -> Result<(), CpuCustodyError> {
        if !Rc::ptr_eq(&self.leaf_authority, &lease.authority) {
            return Err(CpuCustodyError::WrongMachine);
        }
        if lease.finished {
            return Err(CpuCustodyError::NotLeased);
        }
        let cell = self.admitted_cell(lease.permit.key, lease.permit.birth)?;
        if !Rc::ptr_eq(cell, &lease.cell) {
            return Err(CpuCustodyError::StaleKey);
        }
        if cell.phase.get() != CpuAdmittedPhase::Leased {
            return Err(CpuCustodyError::NotLeased);
        }
        self.leaf_verifier().validate(
            lease.permit,
            lease.permit.key,
            lease.permit.birth,
            lease.permit.thread_birth,
            lease.permit.frame,
        )
    }

    fn verify_process_lease(&self, lease: &CpuProcessLease) -> Result<(), CpuCustodyError> {
        if !Rc::ptr_eq(&self.leaf_authority, &lease.authority) {
            return Err(CpuCustodyError::WrongMachine);
        }
        if lease.finished {
            return Err(CpuCustodyError::NotLeased);
        }
        let cell = self.admitted_cell(lease.permit.key, lease.permit.birth)?;
        if !Rc::ptr_eq(cell, &lease.cell) {
            return Err(CpuCustodyError::StaleKey);
        }
        if cell.phase.get() != CpuAdmittedPhase::Leased {
            return Err(CpuCustodyError::NotLeased);
        }
        self.process_verifier()
            .validate(lease.permit, lease.permit.key, lease.permit.birth)
    }

    fn slot_mut(&mut self, key: CpuCustodyKey) -> Result<&mut Slot, CpuCustodyError> {
        if key.machine_lineage != self.machine_lineage {
            return Err(CpuCustodyError::WrongMachine);
        }
        let slot = self
            .slots
            .get_mut(key.index)
            .ok_or(CpuCustodyError::StaleKey)?;
        if slot.generation != key.generation || matches!(&slot.state, SlotState::Vacant) {
            return Err(CpuCustodyError::StaleKey);
        }
        Ok(slot)
    }

    fn blob_bytes(cpu: &UnicornMips) -> u64 {
        #[cfg(feature = "unicorn")]
        {
            return u64::try_from(cpu.retained_tree_blob_bytes()).unwrap_or(u64::MAX);
        }
        #[cfg(not(feature = "unicorn"))]
        {
            CpuBackend::mapped_blob_ranges(cpu)
                .iter()
                .map(|blob| u64::from(blob.size))
                .fold(0, u64::saturating_add)
        }
    }

    fn snapshot_slot(key: CpuCustodyKey, slot: &Slot) -> Option<CpuCustodySnapshot> {
        match &slot.state {
            SlotState::Vacant => None,
            SlotState::Reserved { birth, .. } => Some(CpuCustodySnapshot {
                key,
                birth: *birth,
                state: CpuCustodyState::Reserved,
                metrics_available: false,
                quarantine_reason: None,
                mapped_blob_bytes: 0,
                virtual_region_bytes: 0,
                native_outcome: None,
                memory_fault: None,
            }),
            SlotState::Quarantined { birth, cpu } => Some(CpuCustodySnapshot {
                key,
                birth: *birth,
                state: CpuCustodyState::Quarantined,
                metrics_available: true,
                quarantine_reason: None,
                mapped_blob_bytes: Self::blob_bytes(cpu),
                virtual_region_bytes: CpuBackend::memory(cpu)
                    .regions()
                    .map(|region| u64::from(region.size))
                    .fold(0, u64::saturating_add),
                native_outcome: Some(cpu.last_native_slice_outcome()),
                memory_fault: cpu
                    .last_debug_snapshot()
                    .and_then(|snapshot| snapshot.memory_fault.clone()),
            }),
            SlotState::Admitted { birth, cell } => {
                let phase = cell.phase.get();
                let (state, quarantine_reason) = match phase {
                    CpuAdmittedPhase::Runnable => (CpuCustodyState::Runnable, None),
                    CpuAdmittedPhase::Leased => (CpuCustodyState::Leased, None),
                    CpuAdmittedPhase::Parked(_) => (CpuCustodyState::Parked, None),
                    CpuAdmittedPhase::SuspendedForChild => {
                        (CpuCustodyState::SuspendedForChild, None)
                    }
                    CpuAdmittedPhase::Quarantined(reason) => {
                        (CpuCustodyState::Quarantined, Some(reason))
                    }
                };
                let payload = cell.cpu.try_borrow().ok();
                let cpu = payload.as_ref().and_then(|payload| payload.as_ref());
                Some(CpuCustodySnapshot {
                    key,
                    birth: *birth,
                    state,
                    metrics_available: cpu.is_some(),
                    quarantine_reason,
                    mapped_blob_bytes: cpu.map(Self::blob_bytes).unwrap_or(0),
                    virtual_region_bytes: cpu
                        .map(|cpu| {
                            CpuBackend::memory(cpu)
                                .regions()
                                .map(|region| u64::from(region.size))
                                .fold(0, u64::saturating_add)
                        })
                        .unwrap_or(0),
                    native_outcome: cpu.map(UnicornMips::last_native_slice_outcome),
                    memory_fault: cpu.and_then(|cpu| {
                        cpu.last_debug_snapshot()
                            .and_then(|snapshot| snapshot.memory_fault.clone())
                    }),
                })
            }
        }
    }
}

impl Drop for CpuCustody {
    fn drop(&mut self) {
        self.leaf_authority.active.set(None);
        self.leaf_authority.alive.set(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ce::object::HandleTable,
        emulator::{
            memory::MemoryPerms,
            types::{DiagnosticAttemptId, NativeAttemptReport, UnicornRunLimits},
        },
        error::Error,
    };

    fn births() -> (ProcessBirthKey, ProcessBirthKey) {
        let mut handles = HandleTable::default();
        handles.create_process(2);
        handles.create_process(3);
        (
            handles.process_birth_key(2).unwrap(),
            handles.process_birth_key(3).unwrap(),
        )
    }

    fn births_with_threads() -> (
        (ProcessBirthKey, ThreadBirthKey),
        (ProcessBirthKey, ThreadBirthKey),
    ) {
        let mut handles = HandleTable::default();
        handles.create_process(20);
        handles.create_process(21);
        let first = handles.process_birth_key(20).unwrap();
        let second = handles.process_birth_key(21).unwrap();
        handles
            .try_create_thread_in_birth(first, 201, 0x1000, 0, false)
            .unwrap();
        handles
            .try_create_thread_in_birth(second, 211, 0x2000, 0, false)
            .unwrap();
        (
            (first, handles.thread_birth_key_by_id(201).unwrap()),
            (second, handles.thread_birth_key_by_id(211).unwrap()),
        )
    }

    fn reported_execution(
        outcome: NativeSliceOutcome,
        result: crate::Result<()>,
    ) -> AttemptExecution {
        let mut report = NativeAttemptReport::no_attempt(UnicornRunLimits::default());
        if outcome != NativeSliceOutcome::NoAttempt {
            report.attempt_id = Some(DiagnosticAttemptId::next().unwrap());
        }
        report.final_outcome = outcome;
        AttemptExecution { report, result }
    }

    #[test]
    fn process_attempt_permit_is_independent_exclusive_and_revocable() {
        use crate::{ce::kernel::CeKernel, config::RuntimeConfig};

        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let launch_a = kernel.queue_process_launch(Some("\\Test\\permit-a.exe".to_owned()), None);
        let launch_b = kernel.queue_process_launch(Some("\\Test\\permit-b.exe".to_owned()), None);
        let birth_a = launch_a.process_birth.unwrap();
        let birth_b = launch_b.process_birth.unwrap();
        let mut custody = CpuCustody::new().unwrap();
        kernel
            .bind_cpu_machine_lineage(custody.machine_lineage())
            .unwrap();
        let key_a = custody.reserve(birth_a).unwrap();
        let key_b = custody.reserve(birth_b).unwrap();
        custody
            .admit_reserved(key_a, birth_a, UnicornMips::new().unwrap())
            .unwrap();
        custody
            .admit_reserved(key_b, birth_b, UnicornMips::new().unwrap())
            .unwrap();
        let handle_a = custody.acquire_cpu_handle(key_a, birth_a).unwrap();
        let verifier = custody.process_verifier();

        kernel.set_current_process_id(launch_a.process_id);
        assert_eq!(
            custody
                .issue_process_attempt(&kernel, key_b, birth_b)
                .unwrap_err(),
            CpuCustodyError::StaleSelectedProcess
        );
        let cloned_kernel = kernel.clone();
        assert_eq!(
            custody
                .issue_process_attempt(&cloned_kernel, key_a, birth_a)
                .unwrap_err(),
            CpuCustodyError::KernelLineage(CpuMachineLineageError::RequiresRelineage)
        );
        let mut a = custody
            .issue_process_attempt(&kernel, key_a, birth_a)
            .unwrap();
        let old_a = a.permit();
        assert_eq!(verifier.current_permit(), Some(old_a));
        assert_eq!(custody.leaf_verifier().current_permit(), None);
        assert_eq!(verifier.validate(old_a, key_a, birth_a), Ok(()));
        assert_eq!(
            verifier.validate(old_a, key_b, birth_a),
            Err(CpuCustodyError::WrongSlot)
        );
        assert_eq!(
            verifier.validate(old_a, key_a, birth_b),
            Err(CpuCustodyError::WrongBirth)
        );
        assert_eq!(
            CpuCustody::new()
                .unwrap()
                .process_verifier()
                .validate(old_a, key_a, birth_a),
            Err(CpuCustodyError::WrongMachine)
        );
        assert_eq!(
            custody
                .issue_leaf(key_b, birth_b, launch_b.primary_thread_birth.unwrap(), None)
                .unwrap_err(),
            CpuCustodyError::ActiveLeafBusy
        );
        assert_eq!(handle_a.with_process_cpu_mut(old_a, |_| 7), Ok(7));
        custody
            .finish_process_attempt(
                &mut a,
                &kernel,
                reported_execution(NativeSliceOutcome::CountLimit, Ok(())),
                CpuLeaseEnd::Runnable,
            )
            .unwrap();
        assert_eq!(
            verifier.validate(old_a, key_a, birth_a),
            Err(CpuCustodyError::StalePermit)
        );
        drop(a);
        kernel.set_current_process_id(launch_b.process_id);
        let mut b = custody
            .issue_process_attempt(&kernel, key_b, birth_b)
            .unwrap();
        assert_ne!(b.permit().attempt_generation(), old_a.attempt_generation());
        assert_eq!(
            verifier.validate(old_a, key_a, birth_a),
            Err(CpuCustodyError::WrongAttempt)
        );
        custody
            .finish_process_attempt(
                &mut b,
                &kernel,
                reported_execution(NativeSliceOutcome::CountLimit, Ok(())),
                CpuLeaseEnd::Runnable,
            )
            .unwrap();
        drop(b);
        let mut legacy_leaf = custody
            .issue_leaf(key_b, birth_b, launch_b.primary_thread_birth.unwrap(), None)
            .unwrap();
        assert_eq!(
            verifier.validate(old_a, key_a, birth_a),
            Err(CpuCustodyError::WrongPermitKind)
        );
        custody
            .finish_leaf(&mut legacy_leaf, CpuLeaseEnd::Runnable)
            .unwrap();
        drop(legacy_leaf);
        drop(handle_a);
    }

    #[test]
    fn process_attempt_actual_errors_and_unverified_exits_never_resume() {
        use crate::{ce::kernel::CeKernel, config::RuntimeConfig};

        let cases = [
            (
                NativeSliceOutcome::NativeError,
                Err(Error::Backend("native fault".to_owned())),
                CpuQuarantineReason::NativeFault,
            ),
            (
                NativeSliceOutcome::CountLimit,
                Err(Error::Backend("tail failed".to_owned())),
                CpuQuarantineReason::BackendFailure,
            ),
            (
                NativeSliceOutcome::NoAttempt,
                Ok(()),
                CpuQuarantineReason::UnresolvedProcessAttempt,
            ),
            (
                NativeSliceOutcome::GuestProcessExit,
                Ok(()),
                CpuQuarantineReason::UnresolvedProcessAttempt,
            ),
        ];
        for (outcome, result, reason) in cases {
            let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
            let launch =
                kernel.queue_process_launch(Some("\\Test\\process-outcome.exe".to_owned()), None);
            let birth = launch.process_birth.unwrap();
            let mut custody = CpuCustody::new().unwrap();
            kernel
                .bind_cpu_machine_lineage(custody.machine_lineage())
                .unwrap();
            kernel.set_current_process_id(launch.process_id);
            let key = custody.reserve(birth).unwrap();
            let mut original = UnicornMips::new().unwrap();
            CpuBackend::map_region(
                &mut original,
                0x3000,
                0x1000,
                MemoryPerms::READ,
                "process-original",
            )
            .unwrap();
            custody.admit_reserved(key, birth, original).unwrap();
            let mut lease = custody.issue_process_attempt(&kernel, key, birth).unwrap();
            let execution = custody
                .finish_process_attempt(
                    &mut lease,
                    &kernel,
                    reported_execution(outcome, result),
                    CpuLeaseEnd::Runnable,
                )
                .unwrap();
            assert_eq!(execution.report.final_outcome(), outcome);
            assert_eq!(
                custody.admitted_phase(key, birth),
                Ok(CpuAdmittedPhase::Quarantined(reason))
            );
            assert_eq!(
                custody
                    .with_quarantined(key, birth, |cpu| CpuBackend::memory(cpu).regions().count()),
                Ok(1)
            );
            assert_eq!(
                custody
                    .issue_process_attempt(&kernel, key, birth)
                    .unwrap_err(),
                CpuCustodyError::NotRunnable
            );
            drop(lease);
        }
    }

    #[test]
    fn process_parent_suspension_refuses_open_payload_loan_then_revokes_a() {
        use crate::{ce::kernel::CeKernel, config::RuntimeConfig};

        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let launch_a = kernel.queue_process_launch(Some("\\Test\\parent-a.exe".to_owned()), None);
        let launch_b = kernel.queue_process_launch(Some("\\Test\\child-b.exe".to_owned()), None);
        let birth_a = launch_a.process_birth.unwrap();
        let birth_b = launch_b.process_birth.unwrap();
        let mut custody = CpuCustody::new().unwrap();
        kernel
            .bind_cpu_machine_lineage(custody.machine_lineage())
            .unwrap();
        let key_a = custody.reserve(birth_a).unwrap();
        let key_b = custody.reserve(birth_b).unwrap();
        custody
            .admit_reserved(key_a, birth_a, UnicornMips::new().unwrap())
            .unwrap();
        custody
            .admit_reserved(key_b, birth_b, UnicornMips::new().unwrap())
            .unwrap();
        let handle_a = custody.acquire_cpu_handle(key_a, birth_a).unwrap();
        kernel.set_current_process_id(launch_a.process_id);
        let mut a = custody
            .issue_process_attempt(&kernel, key_a, birth_a)
            .unwrap();
        let old_a = a.permit();
        handle_a
            .with_process_cpu_mut(old_a, |_| {
                assert_eq!(
                    custody.suspend_process_for_child(&mut a).unwrap_err(),
                    CpuCustodyError::BusyCpuLoan
                );
                assert_eq!(custody.process_verifier().current_permit(), Some(old_a));
            })
            .unwrap();
        let parent = custody.suspend_process_for_child(&mut a).unwrap();
        assert_eq!(parent.parent_permit(), old_a);
        assert_eq!(custody.process_verifier().current_permit(), None);
        assert_eq!(
            custody.admitted_phase(key_a, birth_a),
            Ok(CpuAdmittedPhase::SuspendedForChild)
        );
        drop(a);
        kernel.set_current_process_id(launch_b.process_id);
        let b = custody
            .issue_process_attempt(&kernel, key_b, birth_b)
            .unwrap();
        let permit_b = b.permit();
        assert_eq!(
            custody.process_verifier().validate(old_a, key_a, birth_a),
            Err(CpuCustodyError::WrongAttempt)
        );
        assert_eq!(
            handle_a.with_process_cpu_mut(old_a, |_| ()),
            Err(CpuCustodyError::WrongAttempt)
        );
        drop(parent);
        assert_eq!(
            custody
                .process_verifier()
                .validate(permit_b, key_b, birth_b),
            Ok(())
        );
        drop(b);
        assert_eq!(custody.counts().quarantined, 2);
        assert_eq!(
            custody.admitted_phase(key_a, birth_a),
            Ok(CpuAdmittedPhase::Quarantined(
                CpuQuarantineReason::UnresolvedParentSuspension
            ))
        );
        assert_eq!(
            custody.admitted_phase(key_b, birth_b),
            Ok(CpuAdmittedPhase::Quarantined(
                CpuQuarantineReason::UnresolvedProcessAttempt
            ))
        );
        drop(handle_a);
    }

    #[test]
    fn process_attempt_parks_only_on_current_canonical_finite_sleep_frame() {
        use crate::{ce::kernel::CeKernel, config::RuntimeConfig};

        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let launch =
            kernel.queue_process_launch(Some("\\Test\\parked-process.exe".to_owned()), None);
        let birth = launch.process_birth.unwrap();
        let thread = launch.primary_thread_birth.unwrap();
        let mut custody = CpuCustody::new().unwrap();
        kernel
            .bind_cpu_machine_lineage(custody.machine_lineage())
            .unwrap();
        kernel.set_current_process_id(launch.process_id);
        let key = custody.reserve(birth).unwrap();
        custody
            .admit_reserved(key, birth, UnicornMips::new().unwrap())
            .unwrap();

        let frame = kernel
            .try_publish_finite_sleep_wait_for_birth(thread, launch.thread_handle, 100, 10)
            .unwrap();
        assert!(kernel.is_blocked_finite_sleep_frame(frame));
        let mut first = custody.issue_process_attempt(&kernel, key, birth).unwrap();
        custody
            .finish_process_attempt(
                &mut first,
                &kernel,
                reported_execution(NativeSliceOutcome::CountLimit, Ok(())),
                CpuLeaseEnd::Parked(frame),
            )
            .unwrap();
        assert_eq!(
            custody.admitted_phase(key, birth),
            Ok(CpuAdmittedPhase::Parked(frame))
        );
        drop(first);

        assert!(kernel.begin_finite_sleep_completion(frame));
        assert!(kernel.retire_finite_sleep_completion(frame));
        custody.wake_parked(key, birth, frame).unwrap();
        let mut second = custody.issue_process_attempt(&kernel, key, birth).unwrap();
        assert_eq!(
            custody
                .finish_process_attempt(
                    &mut second,
                    &kernel,
                    reported_execution(NativeSliceOutcome::CountLimit, Ok(())),
                    CpuLeaseEnd::Parked(frame),
                )
                .unwrap_err(),
            CpuCustodyError::WrongFrame
        );
        assert_eq!(
            custody.admitted_phase(key, birth),
            Ok(CpuAdmittedPhase::Quarantined(
                CpuQuarantineReason::PortFailure
            ))
        );
        drop(second);
    }

    #[test]
    fn unresolved_process_attempt_drop_inside_payload_loan_seals_original_cpu() {
        use crate::{ce::kernel::CeKernel, config::RuntimeConfig};

        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let launch =
            kernel.queue_process_launch(Some("\\Test\\unresolved-process.exe".to_owned()), None);
        let birth = launch.process_birth.unwrap();
        let mut custody = CpuCustody::new().unwrap();
        kernel
            .bind_cpu_machine_lineage(custody.machine_lineage())
            .unwrap();
        kernel.set_current_process_id(launch.process_id);
        let key = custody.reserve(birth).unwrap();
        let mut original = UnicornMips::new().unwrap();
        CpuBackend::map_region(
            &mut original,
            0x4000,
            0x1000,
            MemoryPerms::READ,
            "unresolved-process-original",
        )
        .unwrap();
        custody.admit_reserved(key, birth, original).unwrap();
        let handle = custody.acquire_cpu_handle(key, birth).unwrap();
        let lease = custody.issue_process_attempt(&kernel, key, birth).unwrap();
        let permit = lease.permit();
        let cell = Rc::clone(custody.admitted_cell(key, birth).unwrap());
        handle
            .with_process_cpu_mut(permit, |_cpu| {
                drop(lease);
                assert_eq!(
                    cell.phase.get(),
                    CpuAdmittedPhase::Quarantined(CpuQuarantineReason::UnresolvedProcessAttempt)
                );
                assert!(!cell.quarantine_sealed.get());
            })
            .unwrap();
        assert!(cell.quarantine_sealed.get());
        assert_eq!(
            custody.with_quarantined(key, birth, |cpu| CpuBackend::memory(cpu).regions().count()),
            Ok(1)
        );
        assert_eq!(
            custody.process_verifier().validate(permit, key, birth),
            Err(CpuCustodyError::StalePermit)
        );
        drop(handle);
    }

    #[test]
    fn process_finish_cannot_reissue_a_permit_under_an_open_payload_loan() {
        use crate::{ce::kernel::CeKernel, config::RuntimeConfig};

        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let launch = kernel.queue_process_launch(Some("\\Test\\busy-finish.exe".to_owned()), None);
        let birth = launch.process_birth.unwrap();
        let mut custody = CpuCustody::new().unwrap();
        kernel
            .bind_cpu_machine_lineage(custody.machine_lineage())
            .unwrap();
        kernel.set_current_process_id(launch.process_id);
        let key = custody.reserve(birth).unwrap();
        custody
            .admit_reserved(key, birth, UnicornMips::new().unwrap())
            .unwrap();
        let handle = custody.acquire_cpu_handle(key, birth).unwrap();
        let mut lease = custody.issue_process_attempt(&kernel, key, birth).unwrap();
        let permit = lease.permit();
        let cell = Rc::clone(custody.admitted_cell(key, birth).unwrap());
        handle
            .with_process_cpu_mut(permit, |_cpu| {
                assert_eq!(
                    custody
                        .finish_process_attempt(
                            &mut lease,
                            &kernel,
                            reported_execution(NativeSliceOutcome::CountLimit, Ok(())),
                            CpuLeaseEnd::Runnable,
                        )
                        .unwrap_err(),
                    CpuCustodyError::BusyCpuLoan
                );
                assert_eq!(
                    cell.phase.get(),
                    CpuAdmittedPhase::Quarantined(CpuQuarantineReason::PortFailure)
                );
                assert_eq!(custody.process_verifier().current_permit(), None);
                assert_eq!(
                    custody
                        .issue_process_attempt(&kernel, key, birth)
                        .unwrap_err(),
                    CpuCustodyError::NotRunnable
                );
                assert!(!cell.quarantine_sealed.get());
            })
            .unwrap();
        assert!(cell.quarantine_sealed.get());
        assert_eq!(custody.counts().quarantined, 1);
        drop(lease);
        drop(handle);
    }

    #[test]
    fn unwound_process_payload_loan_revokes_its_active_permit_before_lease_drop() {
        use crate::{ce::kernel::CeKernel, config::RuntimeConfig};

        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let launch =
            kernel.queue_process_launch(Some("\\Test\\unwind-process.exe".to_owned()), None);
        let birth = launch.process_birth.unwrap();
        let mut custody = CpuCustody::new().unwrap();
        kernel
            .bind_cpu_machine_lineage(custody.machine_lineage())
            .unwrap();
        kernel.set_current_process_id(launch.process_id);
        let key = custody.reserve(birth).unwrap();
        custody
            .admit_reserved(key, birth, UnicornMips::new().unwrap())
            .unwrap();
        let handle = custody.acquire_cpu_handle(key, birth).unwrap();
        let lease = custody.issue_process_attempt(&kernel, key, birth).unwrap();
        let permit = lease.permit();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ =
                handle.with_process_cpu_mut(permit, |_cpu| panic!("test-only process loan unwind"));
        }));
        assert!(result.is_err());
        assert!(matches!(
            custody.admitted_phase(key, birth),
            Ok(CpuAdmittedPhase::Quarantined(_))
        ));
        assert_eq!(
            custody.process_verifier().validate(permit, key, birth),
            Err(CpuCustodyError::StalePermit)
        );
        assert!(custody.issue_process_attempt(&kernel, key, birth).is_err());
        assert!(
            custody
                .issue_leaf(key, birth, launch.primary_thread_birth.unwrap(), None)
                .is_err()
        );
        drop(lease);
        drop(handle);
    }

    #[test]
    fn accepted_exit_cannot_retire_a_live_process_attempt_or_open_handle() {
        use crate::{ce::kernel::CeKernel, config::RuntimeConfig};

        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let launch =
            kernel.queue_process_launch(Some("\\Test\\process-retire.exe".to_owned()), None);
        let birth = launch.process_birth.unwrap();
        let mut custody = CpuCustody::new().unwrap();
        kernel
            .bind_cpu_machine_lineage(custody.machine_lineage())
            .unwrap();
        kernel.set_current_process_id(launch.process_id);
        let key = custody.reserve(birth).unwrap();
        custody
            .admit_reserved(key, birth, UnicornMips::new().unwrap())
            .unwrap();
        let handle = custody.acquire_cpu_handle(key, birth).unwrap();
        let lease = custody.issue_process_attempt(&kernel, key, birth).unwrap();
        let accepted = kernel
            .try_mark_process_launch_exited_with_witness(&launch, 0x42, None)
            .unwrap()
            .unwrap();
        let (error, accepted) = custody
            .retire_admitted_with_accepted_exit(key, accepted)
            .unwrap_err();
        assert_eq!(error, CpuCustodyError::NotQuarantined);
        assert_eq!(custody.counts().leased, 1);
        drop(lease);
        let (error, accepted) = custody
            .retire_admitted_with_accepted_exit(key, accepted)
            .unwrap_err();
        assert_eq!(error, CpuCustodyError::OutstandingCpuHandle);
        drop(handle);
        let _original = custody
            .retire_admitted_with_accepted_exit(key, accepted)
            .unwrap();
        assert!(!custody.has_any_custody());
    }

    #[test]
    fn finish_attempt_never_resumes_failed_or_unverified_cpu_despite_runnable_request() {
        let cases = [
            (
                NativeSliceOutcome::NativeError,
                Err(Error::Backend("native fault".to_owned())),
                CpuQuarantineReason::NativeFault,
            ),
            (
                NativeSliceOutcome::BackendError,
                Ok(()),
                CpuQuarantineReason::BackendFailure,
            ),
            (
                NativeSliceOutcome::CountLimit,
                Err(Error::Backend("failed tail".to_owned())),
                CpuQuarantineReason::BackendFailure,
            ),
            (
                NativeSliceOutcome::NoAttempt,
                Err(Error::HostExitAuthorityRecoveryRequired {
                    reason: "checked recovery hold",
                }),
                CpuQuarantineReason::RecoveryRequired,
            ),
            (
                NativeSliceOutcome::NoAttempt,
                Ok(()),
                CpuQuarantineReason::UnresolvedLeaf,
            ),
            (
                NativeSliceOutcome::GuestProcessExit,
                Ok(()),
                CpuQuarantineReason::UnresolvedLeaf,
            ),
        ];
        for (outcome, result, reason) in cases {
            let expected_error = result.is_err();
            let ((birth, thread), _) = births_with_threads();
            let mut custody = CpuCustody::new().unwrap();
            let key = custody.reserve(birth).unwrap();
            let mut original = UnicornMips::new().unwrap();
            CpuBackend::map_region(
                &mut original,
                0x3000,
                0x1000,
                MemoryPerms::READ,
                "attempt-original",
            )
            .unwrap();
            custody.admit_reserved(key, birth, original).unwrap();
            let handle = custody.acquire_cpu_handle(key, birth).unwrap();
            let mut lease = custody.issue_leaf(key, birth, thread, None).unwrap();
            let permit = lease.permit();
            let cell = Rc::clone(custody.admitted_cell(key, birth).unwrap());
            let returned = handle
                .with_cpu_mut(permit, |_original| {
                    let returned = custody
                        .finish_attempt(
                            &mut lease,
                            reported_execution(outcome, result),
                            CpuLeaseEnd::Runnable,
                        )
                        .unwrap();
                    assert_eq!(
                        cell.phase.get(),
                        CpuAdmittedPhase::Quarantined(reason),
                        "outcome {outcome:?}"
                    );
                    assert!(!cell.quarantine_sealed.get(), "open loan defers seal");
                    returned
                })
                .unwrap();
            assert_eq!(returned.report.final_outcome(), outcome);
            assert_eq!(returned.result.is_err(), expected_error);
            assert!(
                cell.quarantine_sealed.get(),
                "loan close seals original CPU"
            );
            assert_eq!(custody.counts().quarantined, 1);
            assert_eq!(
                custody.with_quarantined(key, birth, |cpu| {
                    CpuBackend::memory(cpu).regions().count()
                }),
                Ok(1),
                "original CPU stays in its exact slot"
            );
            assert_eq!(
                custody.issue_leaf(key, birth, thread, None).unwrap_err(),
                CpuCustodyError::NotRunnable
            );
            drop(lease);
            drop(handle);
        }
    }

    #[test]
    fn finish_attempt_allows_only_reported_success_to_resume_runnable() {
        let ((birth, thread), _) = births_with_threads();
        let mut custody = CpuCustody::new().unwrap();
        let key = custody.reserve(birth).unwrap();
        custody
            .admit_reserved(key, birth, UnicornMips::new().unwrap())
            .unwrap();
        let mut lease = custody.issue_leaf(key, birth, thread, None).unwrap();
        let old = lease.permit();
        let completed = custody
            .finish_attempt(
                &mut lease,
                reported_execution(NativeSliceOutcome::CountLimit, Ok(())),
                CpuLeaseEnd::Runnable,
            )
            .unwrap();
        assert!(completed.result.is_ok());
        assert_eq!(
            custody.admitted_phase(key, birth),
            Ok(CpuAdmittedPhase::Runnable)
        );
        assert_eq!(
            custody
                .leaf_verifier()
                .validate(old, key, birth, thread, None),
            Err(CpuCustodyError::StalePermit)
        );
        drop(lease);
        let next = custody.issue_leaf(key, birth, thread, None).unwrap();
        assert_ne!(next.permit().attempt_generation(), old.attempt_generation());
        drop(next);
    }

    #[test]
    fn finish_attempt_bad_parked_locator_quarantines_instead_of_stranding_lease() {
        let mut handles = HandleTable::default();
        handles.create_process(20);
        handles.create_process(21);
        let birth = handles.process_birth_key(20).unwrap();
        let other_birth = handles.process_birth_key(21).unwrap();
        handles
            .try_create_thread_in_birth(birth, 201, 0x1000, 0, false)
            .unwrap();
        handles
            .try_create_thread_in_birth(other_birth, 211, 0x2000, 0, false)
            .unwrap();
        let thread = handles.thread_birth_key_by_id(201).unwrap();
        let other_thread = handles.thread_birth_key_by_id(211).unwrap();
        let foreign_wait = TaskFrameRef::new(
            handles.preflight_finite_sleep_frame(other_thread).unwrap(),
            41,
        )
        .unwrap();
        let mut custody = CpuCustody::new().unwrap();
        let key = custody.reserve(birth).unwrap();
        custody
            .admit_reserved(key, birth, UnicornMips::new().unwrap())
            .unwrap();
        let mut lease = custody.issue_leaf(key, birth, thread, None).unwrap();
        assert_eq!(
            custody
                .finish_attempt(
                    &mut lease,
                    reported_execution(NativeSliceOutcome::CountLimit, Ok(())),
                    CpuLeaseEnd::Parked(foreign_wait),
                )
                .unwrap_err(),
            CpuCustodyError::WrongThread
        );
        assert_eq!(
            custody.admitted_phase(key, birth),
            Ok(CpuAdmittedPhase::Quarantined(
                CpuQuarantineReason::PortFailure
            ))
        );
        assert!(
            custody
                .admitted_cell(key, birth)
                .unwrap()
                .quarantine_sealed
                .get()
        );
        assert_eq!(custody.counts().leased, 0);
        drop(lease);
    }

    #[test]
    fn admit_reserved_accepts_clean_cpu_into_exact_runnable_slot() {
        let (birth, _) = births();
        let mut custody = CpuCustody::new().unwrap();
        let key = custody.reserve(birth).unwrap();
        let mut cpu = UnicornMips::new().unwrap();
        CpuBackend::map_region(&mut cpu, 0x1000, 0x1000, MemoryPerms::READ, "clean-cpu").unwrap();
        assert!(!cpu.has_parked_child_processes());

        custody.admit_reserved(key, birth, cpu).unwrap();

        assert_eq!(custody.admitted_key_for_birth(birth), Some(key));
        assert_eq!(
            custody.admitted_phase(key, birth),
            Ok(CpuAdmittedPhase::Runnable)
        );
        assert_eq!(custody.counts().reserved, 0);
        assert_eq!(custody.counts().runnable, 1);
        assert_eq!(
            custody
                .snapshot_exact(key, birth)
                .unwrap()
                .virtual_region_bytes,
            0x1000
        );
    }

    #[test]
    fn admitted_cpu_is_owned_before_first_leaf_and_old_permits_revoke() {
        let ((birth, thread), (other_birth, other_thread)) = births_with_threads();
        let mut custody = CpuCustody::new().unwrap();
        let key = custody.reserve(birth).unwrap();
        let other_key = custody.reserve(other_birth).unwrap();
        let mut cpu = UnicornMips::new().unwrap();
        CpuBackend::map_region(&mut cpu, 0x1000, 0x1000, MemoryPerms::READ, "admitted").unwrap();
        custody.admit_reserved(key, birth, cpu).unwrap();
        assert_eq!(custody.admitted_key_for_birth(birth), Some(key));
        assert_eq!(
            custody.admitted_phase(key, birth),
            Ok(CpuAdmittedPhase::Runnable)
        );
        assert_eq!(custody.counts().runnable, 1);
        assert_eq!(
            custody
                .snapshot_exact(key, birth)
                .unwrap()
                .virtual_region_bytes,
            0x1000
        );
        let handle = custody.acquire_cpu_handle(key, birth).unwrap();
        let verifier = custody.leaf_verifier();
        let mut lease = custody.issue_leaf(key, birth, thread, None).unwrap();
        let old = lease.permit();
        assert_eq!(custody.counts().leased, 1);
        assert_eq!(
            handle.with_cpu_mut(old, |cpu| CpuBackend::memory(cpu).regions().count()),
            Ok(1)
        );
        assert_eq!(
            verifier.validate(old, other_key, birth, thread, None),
            Err(CpuCustodyError::WrongSlot)
        );
        assert_eq!(
            verifier.validate(old, key, other_birth, thread, None),
            Err(CpuCustodyError::WrongBirth)
        );
        assert_eq!(
            verifier.validate(old, key, birth, other_thread, None),
            Err(CpuCustodyError::WrongThread)
        );
        let other_machine = CpuCustody::new().unwrap();
        assert_eq!(
            other_machine
                .leaf_verifier()
                .validate(old, key, birth, thread, None),
            Err(CpuCustodyError::WrongMachine)
        );
        let renewed = custody
            .update_leaf_thread(&mut lease, thread, None)
            .unwrap();
        assert_ne!(renewed.attempt_generation(), old.attempt_generation());
        assert_eq!(
            verifier.validate(old, key, birth, thread, None),
            Err(CpuCustodyError::WrongAttempt)
        );
        assert_eq!(
            handle.with_cpu_mut(old, |_| ()),
            Err(CpuCustodyError::WrongAttempt)
        );
        assert_eq!(verifier.validate(renewed, key, birth, thread, None), Ok(()));
        custody
            .finish_leaf(&mut lease, CpuLeaseEnd::Runnable)
            .unwrap();
        drop(lease);
        assert_eq!(
            custody.admitted_phase(key, birth),
            Ok(CpuAdmittedPhase::Runnable)
        );
        assert_eq!(
            verifier.validate(renewed, key, birth, thread, None),
            Err(CpuCustodyError::StalePermit)
        );
        drop(handle);
    }

    #[test]
    fn suspended_parent_requires_closed_loan_and_rejects_old_parent_reentry() {
        use crate::{ce::kernel::CeKernel, config::RuntimeConfig};

        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let launch_a = kernel.queue_process_launch(Some("\\Test\\parent-a.exe".to_owned()), None);
        let launch_b = kernel.queue_process_launch(Some("\\Test\\child-b.exe".to_owned()), None);
        let birth_a = launch_a.process_birth.unwrap();
        let thread_a = launch_a.primary_thread_birth.unwrap();
        let birth_b = launch_b.process_birth.unwrap();
        let thread_b = launch_b.primary_thread_birth.unwrap();
        let mut custody = CpuCustody::new().unwrap();
        kernel
            .bind_cpu_machine_lineage(custody.machine_lineage())
            .unwrap();
        let key_a = custody.reserve(birth_a).unwrap();
        let key_b = custody.reserve(birth_b).unwrap();
        custody
            .admit_reserved(key_a, birth_a, UnicornMips::new().unwrap())
            .unwrap();
        custody
            .admit_reserved(key_b, birth_b, UnicornMips::new().unwrap())
            .unwrap();
        let handle_a = custody.acquire_cpu_handle(key_a, birth_a).unwrap();
        let handle_b = custody.acquire_cpu_handle(key_b, birth_b).unwrap();
        let mut leaf_a = custody.issue_leaf(key_a, birth_a, thread_a, None).unwrap();
        let old_a = leaf_a.permit();
        handle_a
            .with_cpu_mut(old_a, |_cpu_a| {
                assert_eq!(
                    custody.suspend_for_child(&mut leaf_a).unwrap_err(),
                    CpuCustodyError::BusyCpuLoan
                );
                assert_eq!(custody.leaf_verifier().current_permit(), Some(old_a));
            })
            .unwrap();
        let mut parent = custody.suspend_for_child(&mut leaf_a).unwrap();
        assert_eq!(
            custody.admitted_phase(key_a, birth_a),
            Ok(CpuAdmittedPhase::SuspendedForChild)
        );
        let mut leaf_b = custody.issue_leaf(key_b, birth_b, thread_b, None).unwrap();
        let permit_b = leaf_b.permit();
        assert_eq!(
            handle_a.with_cpu_mut(old_a, |_| ()),
            Err(CpuCustodyError::WrongAttempt)
        );
        kernel.set_current_process_id(launch_b.process_id);
        assert_eq!(handle_b.with_cpu_mut(permit_b, |_| 7), Ok(7));
        custody
            .finish_leaf(&mut leaf_b, CpuLeaseEnd::Runnable)
            .unwrap();
        drop(leaf_b);
        kernel.set_current_process_id(launch_a.process_id);
        custody.resume_parent(&mut parent, &kernel).unwrap();
        drop(leaf_a);
        assert_eq!(
            handle_a.with_cpu_mut(old_a, |_| ()),
            Err(CpuCustodyError::StalePermit)
        );
        let mut resumed = custody.issue_leaf(key_a, birth_a, thread_a, None).unwrap();
        assert_ne!(
            resumed.permit().attempt_generation(),
            old_a.attempt_generation()
        );
        assert_eq!(handle_a.with_cpu_mut(resumed.permit(), |_| 11), Ok(11));
        custody
            .finish_leaf(&mut resumed, CpuLeaseEnd::Runnable)
            .unwrap();
        drop(resumed);
        drop(handle_b);
        drop(handle_a);
    }

    #[test]
    fn suspended_parent_rejects_reused_thread_birth_before_runnable() {
        use crate::{ce::kernel::CeKernel, config::RuntimeConfig};

        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let launch = kernel.queue_process_launch(Some("\\Test\\stale-parent.exe".to_owned()), None);
        let birth = launch.process_birth.unwrap();
        let original_thread = launch.primary_thread_birth.unwrap();
        let mut custody = CpuCustody::new().unwrap();
        kernel
            .bind_cpu_machine_lineage(custody.machine_lineage())
            .unwrap();
        let key = custody.reserve(birth).unwrap();
        custody
            .admit_reserved(key, birth, UnicornMips::new().unwrap())
            .unwrap();
        let mut leaf = custody
            .issue_leaf(key, birth, original_thread, None)
            .unwrap();
        let mut suspension = custody.suspend_for_child(&mut leaf).unwrap();
        drop(leaf);
        kernel.set_current_process_id(launch.process_id);
        assert!(kernel.mark_guest_thread_exited_by_birth(original_thread, 0x21));
        kernel.close_handle(launch.thread_handle).unwrap();
        kernel
            .handles
            .try_create_thread_in_birth(birth, original_thread.thread_id(), 0x4000, 0, false)
            .unwrap();
        assert_ne!(
            kernel
                .handles
                .thread_birth_key_by_id(original_thread.thread_id()),
            Some(original_thread)
        );
        assert_eq!(
            custody.resume_parent(&mut suspension, &kernel),
            Err(CpuCustodyError::StaleParentThread)
        );
        assert_eq!(
            custody.admitted_phase(key, birth),
            Ok(CpuAdmittedPhase::SuspendedForChild)
        );
        drop(suspension);
        assert_eq!(
            custody.admitted_phase(key, birth),
            Ok(CpuAdmittedPhase::Quarantined(
                CpuQuarantineReason::UnresolvedParentSuspension
            ))
        );
    }

    #[test]
    fn unresolved_leaf_and_parent_suspension_retain_nonrunnable_cpu() {
        let ((birth_a, thread_a), (birth_b, thread_b)) = births_with_threads();
        let mut custody = CpuCustody::new().unwrap();
        let key_a = custody.reserve(birth_a).unwrap();
        let key_b = custody.reserve(birth_b).unwrap();
        custody
            .admit_reserved(key_a, birth_a, UnicornMips::new().unwrap())
            .unwrap();
        custody
            .admit_reserved(key_b, birth_b, UnicornMips::new().unwrap())
            .unwrap();
        let mut leaf_a = custody.issue_leaf(key_a, birth_a, thread_a, None).unwrap();
        let parent = custody.suspend_for_child(&mut leaf_a).unwrap();
        drop(leaf_a);
        let leaf_b = custody.issue_leaf(key_b, birth_b, thread_b, None).unwrap();
        drop(leaf_b);
        drop(parent);
        assert_eq!(
            custody.admitted_phase(key_a, birth_a),
            Ok(CpuAdmittedPhase::Quarantined(
                CpuQuarantineReason::UnresolvedParentSuspension
            ))
        );
        assert_eq!(
            custody.admitted_phase(key_b, birth_b),
            Ok(CpuAdmittedPhase::Quarantined(
                CpuQuarantineReason::UnresolvedLeaf
            ))
        );
        assert_eq!(custody.counts().quarantined, 2);
        assert!(
            custody
                .admitted_cell(key_a, birth_a)
                .unwrap()
                .quarantine_sealed
                .get()
        );
        assert!(
            custody
                .admitted_cell(key_b, birth_b)
                .unwrap()
                .quarantine_sealed
                .get()
        );
        assert!(
            custody
                .snapshot_exact(key_a, birth_a)
                .unwrap()
                .metrics_available
        );
        assert_eq!(
            custody
                .issue_leaf(key_a, birth_a, thread_a, None)
                .unwrap_err(),
            CpuCustodyError::NotRunnable
        );
    }

    #[test]
    fn admitted_native_fault_seals_after_open_loan_and_preserves_exit_witness() {
        use crate::{ce::kernel::CeKernel, config::RuntimeConfig};

        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let launch =
            kernel.queue_process_launch(Some("\\Test\\admitted-fault.exe".to_owned()), None);
        let birth = launch.process_birth.unwrap();
        let thread = launch.primary_thread_birth.unwrap();
        let mut custody = CpuCustody::new().unwrap();
        kernel
            .bind_cpu_machine_lineage(custody.machine_lineage())
            .unwrap();
        let key = custody.reserve(birth).unwrap();
        custody
            .admit_reserved(key, birth, UnicornMips::new().unwrap())
            .unwrap();
        let handle = custody.acquire_cpu_handle(key, birth).unwrap();
        let mut lease = custody.issue_leaf(key, birth, thread, None).unwrap();
        let permit = lease.permit();
        let cell = Rc::clone(custody.admitted_cell(key, birth).unwrap());
        let accepted = handle
            .with_cpu_mut(permit, |_cpu| {
                custody
                    .finish_leaf(
                        &mut lease,
                        CpuLeaseEnd::Quarantined(CpuQuarantineReason::NativeFault),
                    )
                    .unwrap();
                assert!(!cell.quarantine_sealed.get(), "open loan defers sealing");
                let accepted = kernel
                    .try_mark_process_launch_exited_with_witness(&launch, 0x31, None)
                    .unwrap()
                    .unwrap();
                let (error, accepted) = custody
                    .retire_admitted_with_accepted_exit(key, accepted)
                    .unwrap_err();
                assert_eq!(error, CpuCustodyError::BusyCpuLoan);
                accepted
            })
            .unwrap();
        assert!(
            cell.quarantine_sealed.get(),
            "loan drop must seal original CPU"
        );
        assert_eq!(
            custody.with_quarantined(key, birth, |cpu| cpu.has_saved_context()),
            Ok(false)
        );
        drop(lease);
        let mut unbound_kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let unbound_launch = unbound_kernel
            .queue_process_launch(Some("\\Test\\unbound-witness.exe".to_owned()), None);
        let unbound_witness = unbound_kernel
            .try_mark_process_launch_exited_with_witness(&unbound_launch, 0x30, None)
            .unwrap()
            .unwrap();
        let (error, _unbound_witness) = custody
            .retire_admitted_with_accepted_exit(key, unbound_witness)
            .unwrap_err();
        assert_eq!(
            error,
            CpuCustodyError::KernelLineage(CpuMachineLineageError::Unbound)
        );
        let (error, accepted) = custody
            .retire_admitted_with_accepted_exit(key, accepted)
            .unwrap_err();
        assert_eq!(error, CpuCustodyError::OutstandingCpuHandle);
        drop(handle);
        let _original_cpu = custody
            .retire_admitted_with_accepted_exit(key, accepted)
            .unwrap();
        assert!(!custody.has_any_custody());
    }

    #[test]
    fn unresolved_leaf_drop_inside_payload_loan_seals_and_can_retire() {
        use crate::{ce::kernel::CeKernel, config::RuntimeConfig};

        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let launch =
            kernel.queue_process_launch(Some("\\Test\\unresolved-leaf.exe".to_owned()), None);
        let birth = launch.process_birth.unwrap();
        let thread = launch.primary_thread_birth.unwrap();
        let mut custody = CpuCustody::new().unwrap();
        kernel
            .bind_cpu_machine_lineage(custody.machine_lineage())
            .unwrap();
        let key = custody.reserve(birth).unwrap();
        custody
            .admit_reserved(key, birth, UnicornMips::new().unwrap())
            .unwrap();
        let handle = custody.acquire_cpu_handle(key, birth).unwrap();
        let lease = custody.issue_leaf(key, birth, thread, None).unwrap();
        let permit = lease.permit();
        let cell = Rc::clone(custody.admitted_cell(key, birth).unwrap());
        handle
            .with_cpu_mut(permit, |_cpu| {
                drop(lease);
                assert!(!cell.quarantine_sealed.get());
                assert_eq!(
                    custody.with_quarantined(key, birth, |_| ()),
                    Err(CpuCustodyError::UnsealedQuarantine)
                );
            })
            .unwrap();
        assert!(cell.quarantine_sealed.get());
        assert_eq!(
            custody.admitted_phase(key, birth),
            Ok(CpuAdmittedPhase::Quarantined(
                CpuQuarantineReason::UnresolvedLeaf
            ))
        );
        drop(handle);
        let accepted = kernel
            .try_mark_process_launch_exited_with_witness(&launch, 0x32, None)
            .unwrap()
            .unwrap();
        let _original_cpu = custody
            .retire_admitted_with_accepted_exit(key, accepted)
            .unwrap();
        assert!(!custody.has_any_custody());
    }

    #[test]
    fn panicking_admitted_cpu_loan_seals_before_unwind_returns() {
        let ((birth, thread), _) = births_with_threads();
        let mut custody = CpuCustody::new().unwrap();
        let key = custody.reserve(birth).unwrap();
        custody
            .admit_reserved(key, birth, UnicornMips::new().unwrap())
            .unwrap();
        let handle = custody.acquire_cpu_handle(key, birth).unwrap();
        let lease = custody.issue_leaf(key, birth, thread, None).unwrap();
        let permit = lease.permit();
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = handle.with_cpu_mut(permit, |_cpu| -> () {
                panic!("intentional admitted CPU loan failure")
            });
        }));
        assert!(panic.is_err());
        let cell = custody.admitted_cell(key, birth).unwrap();
        assert!(cell.quarantine_sealed.get());
        assert_eq!(
            cell.phase.get(),
            CpuAdmittedPhase::Quarantined(CpuQuarantineReason::UnresolvedLeaf)
        );
        drop(lease);
        assert_eq!(
            handle.with_cpu_mut(permit, |_| ()),
            Err(CpuCustodyError::StalePermit)
        );
    }

    #[test]
    fn reservation_and_admission_keep_one_original_cpu() {
        let (birth, other) = births();
        let mut custody = CpuCustody::new().unwrap();
        let key = custody.reserve(birth).unwrap();
        assert_eq!(custody.reserve(birth), Err(CpuCustodyError::DuplicateBirth));
        assert_eq!(custody.counts().reserved, 1);
        let mut cpu = UnicornMips::new().unwrap();
        CpuBackend::map_region(&mut cpu, 0x1000, 0x1000, MemoryPerms::READ, "fixture").unwrap();
        let (error, cpu) = custody.quarantine_reserved(key, other, cpu).unwrap_err();
        assert_eq!(error, CpuCustodyError::WrongBirth);
        custody.quarantine_reserved(key, birth, cpu).unwrap();
        let snapshot = custody.snapshot_exact(key, birth).unwrap();
        assert_eq!(snapshot.state, CpuCustodyState::Quarantined);
        assert_eq!(snapshot.virtual_region_bytes, 0x1000);
        assert_eq!(custody.counts().quarantined, 1);
        assert!(custody.has_any_custody());
        assert_eq!(
            custody.with_quarantined(key, birth, |cpu| CpuBackend::memory(cpu).regions().count()),
            Ok(1)
        );
        assert_eq!(
            custody.cancel_reservation(key, birth),
            Err(CpuCustodyError::NotReserved)
        );
    }

    #[test]
    fn stale_key_machine_birth_and_generation_are_rejected() {
        let (birth, other) = births();
        let mut first = CpuCustody::new().unwrap();
        let second = CpuCustody::new().unwrap();
        let old = first.reserve(birth).unwrap();
        assert_eq!(
            second.snapshot_exact(old, birth),
            Err(CpuCustodyError::WrongMachine)
        );
        assert_eq!(
            first.cancel_reservation(old, other),
            Err(CpuCustodyError::WrongBirth)
        );
        first.cancel_reservation(old, birth).unwrap();
        assert_eq!(
            first.snapshot_exact(old, birth),
            Err(CpuCustodyError::StaleKey)
        );
        let next = first.reserve(other).unwrap();
        assert_eq!(next.index(), old.index());
        assert_ne!(next.generation(), old.generation());
        assert_eq!(
            first.cancel_reservation(old, birth),
            Err(CpuCustodyError::StaleKey)
        );
        first.cancel_reservation(next, other).unwrap();
        first.next_generation = u64::MAX;
        assert_eq!(
            first.reserve(birth),
            Err(CpuCustodyError::GenerationExhausted)
        );
        assert!(!first.has_any_custody());
    }

    #[test]
    fn only_kernel_accepted_exact_exit_releases_quarantined_cpu() {
        use crate::{ce::kernel::CeKernel, config::RuntimeConfig};

        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let launch =
            kernel.queue_process_launch(Some("\\Test\\custody-child.exe".to_owned()), None);
        let other = kernel.queue_process_launch(Some("\\Test\\other-child.exe".to_owned()), None);
        let birth = launch.process_birth.unwrap();
        let mut custody = CpuCustody::new().unwrap();
        let key = custody.reserve(birth).unwrap();
        custody
            .quarantine_reserved(key, birth, UnicornMips::new().unwrap())
            .unwrap();

        let other_exit = kernel
            .try_mark_process_launch_exited_with_witness(&other, 0x22, None)
            .unwrap()
            .expect("other birth completed");
        assert!(matches!(
            custody.retire_with_accepted_exit(key, other_exit),
            Err(CpuCustodyError::WrongBirth)
        ));
        assert_eq!(custody.counts().quarantined, 1);
        assert!(kernel.handles.process_execution_live_by_birth(birth));

        // Closing both aliases before completion can remove the canonical
        // process record as soon as Kernel accepts its exit. The witness still
        // authorizes the exact old CPU's terminal transfer.
        kernel.close_handle(launch.process_handle).unwrap();
        kernel.close_handle(launch.thread_handle).unwrap();
        let accepted = kernel
            .try_mark_process_launch_exited_with_witness(&launch, 0x33, None)
            .unwrap()
            .expect("exact child exit completed");
        assert!(!kernel.handles.process_execution_live_by_birth(birth));
        let _original_cpu = custody.retire_with_accepted_exit(key, accepted).unwrap();
        assert!(!custody.has_any_custody());
        assert_eq!(custody.counts().quarantined, 0);
        assert_eq!(
            custody.snapshot_exact(key, birth),
            Err(CpuCustodyError::StaleKey)
        );
    }

    #[test]
    fn same_birth_wrong_machine_exit_witness_cannot_retire_quarantined_cpu() {
        use crate::{ce::kernel::CeKernel, config::RuntimeConfig};

        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let launch =
            kernel.queue_process_launch(Some("\\Test\\lineage-child.exe".to_owned()), None);
        let birth = launch.process_birth.unwrap();
        // Independent boots can issue the same numeric/incarnation birth.
        // A Kernel clone is deliberately poisoned even before binding.
        let mut wrong_kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let wrong_launch =
            wrong_kernel.queue_process_launch(Some("\\Test\\lineage-child.exe".to_owned()), None);
        let mut legacy_wrong_kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let legacy_wrong_launch = legacy_wrong_kernel
            .queue_process_launch(Some("\\Test\\lineage-child.exe".to_owned()), None);
        assert_eq!(wrong_launch.process_birth, Some(birth));
        assert_eq!(legacy_wrong_launch.process_birth, Some(birth));
        let mut custody = CpuCustody::new().unwrap();
        let other = CpuCustody::new().unwrap();
        kernel
            .bind_cpu_machine_lineage(custody.machine_lineage())
            .unwrap();
        wrong_kernel
            .bind_cpu_machine_lineage(other.machine_lineage())
            .unwrap();
        legacy_wrong_kernel
            .bind_cpu_machine_lineage(other.machine_lineage())
            .unwrap();
        let key = custody.reserve(birth).unwrap();
        custody
            .quarantine_reserved(key, birth, UnicornMips::new().unwrap())
            .unwrap();

        let wrong = wrong_kernel
            .try_mark_process_launch_exited_with_witness(&wrong_launch, 0x41, None)
            .unwrap()
            .unwrap();
        assert_eq!(wrong.birth(), birth);
        let (error, returned_witness) = custody
            .retire_with_bound_accepted_exit(&kernel, key, wrong)
            .unwrap_err();
        assert_eq!(
            error,
            CpuCustodyError::KernelLineage(CpuMachineLineageError::WrongMachine {
                bound: other.machine_lineage(),
                requested: custody.machine_lineage(),
            })
        );
        assert_eq!(returned_witness.birth(), birth);
        assert_eq!(
            returned_witness.cpu_machine_lineage(),
            Some(other.machine_lineage())
        );
        assert_eq!(custody.counts().quarantined, 1);
        assert_eq!(
            custody.snapshot_exact(key, birth).unwrap().state,
            CpuCustodyState::Quarantined
        );

        let legacy_wrong = legacy_wrong_kernel
            .try_mark_process_launch_exited_with_witness(&legacy_wrong_launch, 0x42, None)
            .unwrap()
            .unwrap();
        assert_eq!(legacy_wrong.birth(), birth);
        assert!(matches!(
            custody.retire_with_accepted_exit(key, legacy_wrong),
            Err(CpuCustodyError::KernelLineage(
                CpuMachineLineageError::WrongMachine { .. }
            ))
        ));
        assert_eq!(custody.counts().quarantined, 1);
        assert!(kernel.handles.process_execution_live_by_birth(birth));

        let exact = kernel
            .try_mark_process_launch_exited_with_witness(&launch, 0x43, None)
            .unwrap()
            .unwrap();
        let _original_cpu = custody
            .retire_with_bound_accepted_exit(&kernel, key, exact)
            .unwrap();
        assert!(!custody.has_any_custody());
    }

    #[test]
    fn bound_terminal_retirement_refuses_unbound_same_birth_witness() {
        use crate::{ce::kernel::CeKernel, config::RuntimeConfig};

        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let launch =
            kernel.queue_process_launch(Some("\\Test\\unbound-child.exe".to_owned()), None);
        let birth = launch.process_birth.unwrap();
        let mut unbound_kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let unbound_launch =
            unbound_kernel.queue_process_launch(Some("\\Test\\unbound-child.exe".to_owned()), None);
        assert_eq!(unbound_launch.process_birth, Some(birth));
        let mut custody = CpuCustody::new().unwrap();
        kernel
            .bind_cpu_machine_lineage(custody.machine_lineage())
            .unwrap();
        let key = custody.reserve(birth).unwrap();
        custody
            .quarantine_reserved(key, birth, UnicornMips::new().unwrap())
            .unwrap();

        let unbound = unbound_kernel
            .try_mark_process_launch_exited_with_witness(&unbound_launch, 0x51, None)
            .unwrap()
            .unwrap();
        assert_eq!(unbound.birth(), birth);
        assert_eq!(unbound.cpu_machine_lineage(), None);
        let (error, returned_witness) = custody
            .retire_with_bound_accepted_exit(&kernel, key, unbound)
            .unwrap_err();
        assert_eq!(
            error,
            CpuCustodyError::KernelLineage(CpuMachineLineageError::Unbound)
        );
        assert_eq!(returned_witness.birth(), birth);
        assert_eq!(returned_witness.cpu_machine_lineage(), None);
        assert_eq!(custody.counts().quarantined, 1);
        assert!(kernel.handles.process_execution_live_by_birth(birth));

        let exact = kernel
            .try_mark_process_launch_exited_with_witness(&launch, 0x52, None)
            .unwrap()
            .unwrap();
        assert_eq!(exact.cpu_machine_lineage(), Some(custody.machine_lineage()));
        let _original_cpu = custody
            .retire_with_bound_accepted_exit(&kernel, key, exact)
            .unwrap();
        assert!(!custody.has_any_custody());
    }

    #[test]
    fn unbound_ticket_retains_original_tree_without_origin_relabel() {
        let (origin, actual) = births();
        let mut custody = CpuCustody::new().unwrap();
        let ticket = custody.reserve_unbound(origin).unwrap();
        let key = ticket.key();
        assert_eq!(custody.counts().reserved_unbound_trees, 1);
        assert!(custody.has_any_custody());
        assert!(matches!(
            custody.reserve_unbound(origin),
            Err(CpuCustodyError::DuplicateBirth)
        ));

        let mut cpu = UnicornMips::new().unwrap();
        CpuBackend::map_region(&mut cpu, 0x1000, 0x1000, MemoryPerms::READ, "tree").unwrap();
        assert_eq!(ticket.deposit_unverified(cpu), key);
        let snapshot = custody.retained_tree_snapshots().pop().unwrap();
        assert_eq!(snapshot.key, key);
        assert_eq!(snapshot.origin_birth, origin);
        assert_eq!(snapshot.state, RetainedTreeState::UnverifiedStopped);
        assert!(snapshot.metrics_available);
        assert_eq!(snapshot.root_virtual_region_bytes, 0x1000);
        assert_eq!(custody.counts().unverified_stopped_trees, 1);
        assert_eq!(
            custody.with_retained_tree(key, |original| {
                CpuBackend::memory(original).regions().count()
            }),
            Ok(1)
        );

        let other = custody.reserve_unbound(actual).unwrap();
        let other_key = other.key();
        assert_ne!(other_key.generation(), key.generation());
        other.cancel();
        assert_eq!(custody.counts().reserved_unbound_trees, 0);
        assert_eq!(custody.counts().unverified_stopped_trees, 1);
    }

    #[test]
    fn cancelled_unbound_ticket_reuses_index_with_new_generation() {
        let (origin, actual) = births();
        let mut custody = CpuCustody::new().unwrap();
        let first = custody.reserve_unbound(origin).unwrap();
        let stale = first.key();
        first.cancel();
        assert!(!custody.has_any_custody());
        assert_eq!(
            custody.with_retained_tree(stale, |_| ()),
            Err(CpuCustodyError::StaleKey)
        );
        let next = custody.reserve_unbound(actual).unwrap();
        assert_eq!(next.key().index(), stale.index());
        assert_ne!(next.key().generation(), stale.generation());
        next.cancel();
    }

    #[test]
    fn retained_tree_inspection_panic_restores_original_cpu() {
        let (origin, _) = births();
        let mut custody = CpuCustody::new().unwrap();
        let ticket = custody.reserve_unbound(origin).unwrap();
        let key = ticket.deposit_unverified(UnicornMips::new().unwrap());
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = custody.with_retained_tree(key, |_| -> () {
                panic!("intentional retained-tree inspection failure")
            });
        }));
        assert!(panic.is_err());
        assert_eq!(custody.counts().unverified_stopped_trees, 1);
        assert_eq!(custody.with_retained_tree(key, |_| 42), Ok(42));
    }

    #[test]
    fn implicitly_dropped_ticket_leaves_terminal_custody_fault() {
        let (origin, _) = births();
        let mut custody = CpuCustody::new().unwrap();
        let ticket = custody.reserve_unbound(origin).unwrap();
        let key = ticket.key();
        drop(ticket);
        assert_eq!(custody.counts().reserved_unbound_trees, 0);
        assert_eq!(custody.counts().abandoned_unbound_trees, 1);
        assert!(custody.has_any_custody());
        assert_eq!(
            custody.with_retained_tree(key, |_| ()),
            Err(CpuCustodyError::NotRetainedTree)
        );
        assert!(matches!(
            custody.reserve_unbound(origin),
            Err(CpuCustodyError::DuplicateBirth)
        ));
    }
}
