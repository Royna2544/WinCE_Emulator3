//! One callback-scoped loan origin for a native attempt's Kernel and framebuffer.
//!
//! This port is borrowed from the caller; it does not duplicate Task or CPU
//! authority. Native callbacks use the current attempt token and release every
//! state loan before another guest engine can enter.

use std::cell::{Cell, OnceCell, RefCell, RefMut};

use crate::ce::{
    framebuffer::Framebuffer,
    kernel::CeKernel,
    object::{ProcessBirthKey, ThreadBirthKey},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AttemptAuthority {
    pub(crate) process_birth: Option<ProcessBirthKey>,
    pub(crate) thread_birth: Option<ThreadBirthKey>,
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
        }
    }
}

impl<'machine> MachineKernelPort<'machine> {
    pub(crate) fn new(
        kernel: &'machine mut CeKernel,
        framebuffer: &'machine mut (dyn Framebuffer + 'machine),
    ) -> Self {
        Self {
            state: RefCell::new(MachineState {
                kernel,
                framebuffer,
            }),
            active: Cell::new(None),
            first_terminal: OnceCell::new(),
        }
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
        self.try_loan(operation)
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
