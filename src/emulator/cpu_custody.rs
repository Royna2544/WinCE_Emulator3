//! Terminal, exact-birth custody for a CPU whose native attempt failed.
//!
//! This is a deliberately narrow holding directory. It does not schedule a
//! quarantined CPU, reconstruct a continuation, or own Kernel VM/file views.
//! A caller reserves capacity before constructing/running the child, then
//! moves the original CPU into that slot without allocation after a fault.

use std::{
    cell::Cell,
    fmt,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    ce::{kernel::AcceptedProcessExit, object::ProcessBirthKey},
    emulator::{
        cpu::{CpuBackend, UnicornMips},
        types::{NativeSliceOutcome, UnicornMemoryFault},
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
    Quarantined,
}

/// Diagnostic values only: neither an executable lease nor a CPU clone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuCustodySnapshot {
    pub key: CpuCustodyKey,
    pub birth: ProcessBirthKey,
    pub state: CpuCustodyState,
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
        };
        f.write_str(reason)
    }
}

impl std::error::Error for CpuCustodyError {}

#[derive(Debug)]
enum SlotState {
    Vacant,
    Reserved {
        birth: ProcessBirthKey,
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
            Self::Reserved { birth } | Self::Quarantined { birth, .. } => Some(*birth),
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
            slots: Vec::new(),
            retained_trees: Vec::new(),
        })
    }

    pub fn machine_lineage(&self) -> u64 {
        self.machine_lineage
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
                self.slots.push(Slot {
                    generation,
                    state: SlotState::Reserved { birth },
                });
                self.next_generation = generation;
                return Ok(CpuCustodyKey {
                    machine_lineage: self.machine_lineage,
                    index,
                    generation,
                });
            }
        };
        self.slots[index] = Slot {
            generation,
            state: SlotState::Reserved { birth },
        };
        self.next_generation = generation;
        Ok(CpuCustodyKey {
            machine_lineage: self.machine_lineage,
            index,
            generation,
        })
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
            SlotState::Reserved { birth: frozen } if *frozen == birth => {
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
            SlotState::Reserved { birth: frozen } if *frozen == birth => {
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
            _ => Err(CpuCustodyError::NotQuarantined),
        }
    }

    /// Terminal transfer only after Kernel has minted an accepted exact-birth
    /// completion witness. The caller may hoist foreign descendants, then drop
    /// this original CPU; an error leaves the slot and payload untouched.
    pub(crate) fn retire_with_accepted_exit(
        &mut self,
        key: CpuCustodyKey,
        accepted_exit: AcceptedProcessExit,
    ) -> Result<UnicornMips, CpuCustodyError> {
        let slot = self.slot_mut(key)?;
        match &slot.state {
            SlotState::Quarantined { birth, .. } if *birth == accepted_exit.birth() => {}
            SlotState::Quarantined { .. } => return Err(CpuCustodyError::WrongBirth),
            _ => return Err(CpuCustodyError::NotQuarantined),
        }
        match std::mem::replace(&mut slot.state, SlotState::Vacant) {
            SlotState::Quarantined { cpu, .. } => Ok(cpu),
            _ => unreachable!("checked quarantined slot changed before terminal transfer"),
        }
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
        u64::try_from(cpu.retained_tree_blob_bytes()).unwrap_or(u64::MAX)
    }

    fn snapshot_slot(key: CpuCustodyKey, slot: &Slot) -> Option<CpuCustodySnapshot> {
        match &slot.state {
            SlotState::Vacant => None,
            SlotState::Reserved { birth } => Some(CpuCustodySnapshot {
                key,
                birth: *birth,
                state: CpuCustodyState::Reserved,
                mapped_blob_bytes: 0,
                virtual_region_bytes: 0,
                native_outcome: None,
                memory_fault: None,
            }),
            SlotState::Quarantined { birth, cpu } => Some(CpuCustodySnapshot {
                key,
                birth: *birth,
                state: CpuCustodyState::Quarantined,
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ce::object::HandleTable, emulator::memory::MemoryPerms};

    fn births() -> (ProcessBirthKey, ProcessBirthKey) {
        let mut handles = HandleTable::default();
        handles.create_process(2);
        handles.create_process(3);
        (
            handles.process_birth_key(2).unwrap(),
            handles.process_birth_key(3).unwrap(),
        )
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
