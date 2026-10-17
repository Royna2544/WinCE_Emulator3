//! One callback-scoped loan origin for a native attempt's Kernel and framebuffer.
//!
//! This port is borrowed from the caller; it does not duplicate Task or CPU
//! authority. Native callbacks use the current attempt token and release every
//! state loan before another guest engine can enter.

use std::cell::{Cell, OnceCell, RefCell, RefMut};

use super::cpu_custody::{CpuCustodyError, CpuLeafPermit, CpuLeafVerifier};
use crate::ce::{
    framebuffer::Framebuffer,
    kernel::{CeKernel, CpuMachineLineageError},
    object::ProcessBirthKey,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AttemptAuthority {
    pub(crate) process_birth: Option<ProcessBirthKey>,
    pub(crate) slice_generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(crate) enum MachinePortFailure {
    #[error("machine-state loan busy during {operation}")]
    LoanBusy { operation: &'static str },
    #[error("machine-state attempt authority differs during {operation}")]
    WrongAttempt { operation: &'static str },
    #[error("machine-state attempt already active during {operation}")]
    AttemptBusy { operation: &'static str },
    #[error("CPU directory is required during {operation}")]
    MissingCpuDirectory { operation: &'static str },
    #[error("CPU machine binding refused during {operation}: {reason}")]
    CpuMachineBinding {
        operation: &'static str,
        reason: CpuMachineLineageError,
    },
    #[error("CPU execution permit is required during {operation}")]
    MissingCpuLeaf { operation: &'static str },
    #[error("CPU execution permit refused during {operation}: {reason}")]
    CpuLeaf {
        operation: &'static str,
        reason: CpuCustodyError,
    },
}

struct MachineState<'machine> {
    kernel: &'machine mut CeKernel,
    framebuffer: &'machine mut (dyn Framebuffer + 'machine),
}

/// A loan is bounded by its owning callback or host operation. It must never
/// cross `emu_start` or a recursive child run.
pub(crate) struct MachineStateLoan<'port, 'machine> {
    state: RefMut<'port, MachineState<'machine>>,
}

impl MachineStateLoan<'_, '_> {
    pub(crate) fn kernel(&self) -> &CeKernel {
        self.state.kernel
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut CeKernel {
        self.state.kernel
    }

    pub(crate) fn framebuffer_mut(&mut self) -> &mut dyn Framebuffer {
        self.state.framebuffer
    }

    pub(crate) fn split_mut(&mut self) -> (&mut CeKernel, &mut dyn Framebuffer) {
        let state = &mut *self.state;
        (&mut *state.kernel, &mut *state.framebuffer)
    }
}

/// The caller's two mutable references have exactly one location while the
/// port exists. `try_borrow_mut` turns reentry into a typed terminal failure.
pub(crate) struct MachineKernelPort<'machine> {
    state: RefCell<MachineState<'machine>>,
    active: Cell<Option<AttemptAuthority>>,
    leaf_verifier: Option<CpuLeafVerifier>,
    selected_leaf: Cell<Option<CpuLeafPermit>>,
    first_terminal: OnceCell<MachinePortFailure>,
}

pub(crate) struct ActiveAttemptGuard<'port, 'machine> {
    port: &'port MachineKernelPort<'machine>,
    authority: AttemptAuthority,
}

impl Drop for ActiveAttemptGuard<'_, '_> {
    fn drop(&mut self) {
        if self.port.active.get() == Some(self.authority) {
            self.port.active.set(None);
            self.port.selected_leaf.set(None);
        }
    }
}

impl<'machine> MachineKernelPort<'machine> {
    pub(crate) fn new(
        kernel: &'machine mut CeKernel,
        framebuffer: &'machine mut (dyn Framebuffer + 'machine),
    ) -> Self {
        Self::new_with_optional_cpu_directory(kernel, framebuffer, None)
    }

    pub(crate) fn new_with_cpu_directory(
        kernel: &'machine mut CeKernel,
        framebuffer: &'machine mut (dyn Framebuffer + 'machine),
        verifier: CpuLeafVerifier,
    ) -> Self {
        Self::new_with_optional_cpu_directory(kernel, framebuffer, Some(verifier))
    }

    fn new_with_optional_cpu_directory(
        kernel: &'machine mut CeKernel,
        framebuffer: &'machine mut (dyn Framebuffer + 'machine),
        leaf_verifier: Option<CpuLeafVerifier>,
    ) -> Self {
        Self {
            state: RefCell::new(MachineState {
                kernel,
                framebuffer,
            }),
            active: Cell::new(None),
            leaf_verifier,
            selected_leaf: Cell::new(None),
            first_terminal: OnceCell::new(),
        }
    }

    fn check_cpu_pairing(
        &self,
        kernel: &CeKernel,
        operation: &'static str,
    ) -> Result<(), MachinePortFailure> {
        match &self.leaf_verifier {
            Some(verifier) => kernel
                .require_cpu_machine_lineage(verifier.machine_lineage())
                .map_err(|reason| {
                    self.record_terminal(MachinePortFailure::CpuMachineBinding {
                        operation,
                        reason,
                    })
                }),
            None if kernel.requires_cpu_machine_relineage() => {
                Err(self.record_terminal(MachinePortFailure::CpuMachineBinding {
                    operation,
                    reason: CpuMachineLineageError::RequiresRelineage,
                }))
            }
            None if kernel.cpu_machine_lineage().is_some() => {
                Err(self.record_terminal(MachinePortFailure::MissingCpuDirectory { operation }))
            }
            None => Ok(()),
        }
    }

    fn check_cpu_leaf(
        &self,
        authority: AttemptAuthority,
        operation: &'static str,
    ) -> Result<(), MachinePortFailure> {
        let Some(verifier) = &self.leaf_verifier else {
            return Ok(());
        };
        let permit = self.selected_leaf.get().ok_or_else(|| {
            self.record_terminal(MachinePortFailure::MissingCpuLeaf { operation })
        })?;
        if authority.process_birth != Some(permit.birth()) {
            return Err(self.record_terminal(MachinePortFailure::WrongAttempt { operation }));
        }
        verifier
            .validate(
                permit,
                permit.key(),
                permit.birth(),
                permit.thread_birth(),
                permit.frame(),
            )
            .map_err(|reason| {
                self.record_terminal(MachinePortFailure::CpuLeaf { operation, reason })
            })
    }

    pub(crate) fn record_terminal(&self, failure: MachinePortFailure) -> MachinePortFailure {
        self.first_terminal.get_or_init(|| failure).clone()
    }

    pub(crate) fn terminal_failure(&self) -> Option<MachinePortFailure> {
        self.first_terminal.get().cloned()
    }

    pub(crate) fn try_loan(
        &self,
        operation: &'static str,
    ) -> Result<MachineStateLoan<'_, 'machine>, MachinePortFailure> {
        // A stopped native attempt still needs a host-side loan to persist
        // its original CPU/queue state and perform checked cleanup.
        let state = self
            .state
            .try_borrow_mut()
            .map_err(|_| self.record_terminal(MachinePortFailure::LoanBusy { operation }))?;
        self.check_cpu_pairing(state.kernel, operation)?;
        Ok(MachineStateLoan { state })
    }

    pub(crate) fn enter_attempt(
        &self,
        authority: AttemptAuthority,
        operation: &'static str,
    ) -> Result<ActiveAttemptGuard<'_, 'machine>, MachinePortFailure> {
        if let Some(failure) = self.terminal_failure() {
            return Err(failure);
        }
        if self.active.get().is_some() {
            return Err(self.record_terminal(MachinePortFailure::AttemptBusy { operation }));
        }
        // Do not create a run permit while a setup/tail or host callback loan
        // is still open. This check itself does not borrow either resource.
        let state = self
            .state
            .try_borrow_mut()
            .map_err(|_| self.record_terminal(MachinePortFailure::LoanBusy { operation }))?;
        if state.kernel.current_process_birth_key() != authority.process_birth {
            return Err(self.record_terminal(MachinePortFailure::WrongAttempt { operation }));
        }
        self.check_cpu_pairing(state.kernel, operation)?;
        if let Some(verifier) = &self.leaf_verifier {
            let permit = verifier.current_permit().ok_or_else(|| {
                self.record_terminal(MachinePortFailure::MissingCpuLeaf { operation })
            })?;
            if authority.process_birth != Some(permit.birth()) {
                return Err(self.record_terminal(MachinePortFailure::WrongAttempt { operation }));
            }
            verifier
                .validate(
                    permit,
                    permit.key(),
                    permit.birth(),
                    permit.thread_birth(),
                    permit.frame(),
                )
                .map_err(|reason| {
                    self.record_terminal(MachinePortFailure::CpuLeaf { operation, reason })
                })?;
            self.selected_leaf.set(Some(permit));
        }
        drop(state);
        self.active.set(Some(authority));
        Ok(ActiveAttemptGuard {
            port: self,
            authority,
        })
    }

    pub(crate) fn try_loan_for_attempt(
        &self,
        authority: AttemptAuthority,
        operation: &'static str,
    ) -> Result<MachineStateLoan<'_, 'machine>, MachinePortFailure> {
        if let Some(failure) = self.terminal_failure() {
            return Err(failure);
        }
        if self.active.get() != Some(authority) {
            return Err(self.record_terminal(MachinePortFailure::WrongAttempt { operation }));
        }
        let state = self.try_loan(operation)?;
        self.check_cpu_leaf(authority, operation)?;
        Ok(state)
    }

    pub(crate) fn try_with_attempt_state<R>(
        &self,
        authority: AttemptAuthority,
        operation: &'static str,
        use_state: impl for<'loan> FnOnce(
            &'loan mut CeKernel,
            &'loan mut (dyn Framebuffer + 'loan),
        ) -> R,
    ) -> Result<R, MachinePortFailure> {
        let mut loan = self.try_loan_for_attempt(authority, operation)?;
        let (kernel, framebuffer) = loan.split_mut();
        let result = use_state(kernel, framebuffer);
        drop(loan);
        if let Some(failure) = self.terminal_failure() {
            return Err(failure);
        }
        Ok(result)
    }
}
