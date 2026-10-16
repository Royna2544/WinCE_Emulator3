//! Terminal, exact-birth custody for a CPU whose native attempt failed.
//!
//! This is a deliberately narrow holding directory. It does not schedule a
//! quarantined CPU, reconstruct a continuation, or own Kernel VM/file views.
//! A caller reserves capacity before constructing/running the child, then
//! moves the original CPU into that slot without allocation after a fault.

use std::{
    fmt,
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
    /// Copied/sealed mapped-blob extent, not Kernel physical-page usage.
    pub mapped_blob_bytes: u64,
    /// Guest VA descriptors; these bytes are not necessarily resident.
    pub virtual_region_bytes: u64,
    pub native_outcome: Option<NativeSliceOutcome>,
    pub memory_fault: Option<UnicornMemoryFault>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CpuCustodyCounts {
    pub reserved: usize,
    pub quarantined: usize,
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

/// Sole owner of terminal CPUs. Do not clone or include this in a CPU-only
/// checkpoint; a machine checkpoint must explicitly relineage the directory.
#[derive(Debug)]
pub struct CpuCustody {
    machine_lineage: u64,
    next_generation: u64,
    slots: Vec<Slot>,
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
        })
    }

    pub fn machine_lineage(&self) -> u64 {
        self.machine_lineage
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

    pub fn counts(&self) -> CpuCustodyCounts {
        let mut counts = CpuCustodyCounts::default();
        for slot in &self.slots {
            match &slot.state {
                SlotState::Vacant => {}
                SlotState::Reserved { .. } => counts.reserved += 1,
                SlotState::Quarantined { .. } => counts.quarantined += 1,
            }
        }
        counts
    }

    pub fn has_any_custody(&self) -> bool {
        self.slots
            .iter()
            .any(|slot| !matches!(&slot.state, SlotState::Vacant))
    }

    /// Only CPU-owned mapped blobs; excludes Kernel stack/private/shared pages.
    pub fn retained_cpu_blob_bytes(&self) -> u64 {
        self.slots
            .iter()
            .filter_map(|slot| match &slot.state {
                SlotState::Quarantined { cpu, .. } => Some(Self::blob_bytes(cpu)),
                _ => None,
            })
            .fold(0, u64::saturating_add)
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
        CpuBackend::mapped_blob_ranges(cpu)
            .iter()
            .map(|blob| u64::from(blob.size))
            .fold(0, u64::saturating_add)
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
}
