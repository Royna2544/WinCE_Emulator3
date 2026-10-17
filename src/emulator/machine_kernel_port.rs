//! Preparatory machine-state loan boundary for a future connected native attempt.
//!
//! This module does not yet authorize a Task/frame, run a CPU, or replace the
//! raw Kernel/framebuffer aliases in `unicorn.rs`. The eventual caller must
//! keep this port alive through hook teardown and use it for *all* attempt
//! access before any of those claims become valid.

use std::cell::{OnceCell, RefCell};

use crate::ce::{framebuffer::Framebuffer, kernel::CeKernel};

/// A terminal attempt failure, not an ordinary guest API result or yield.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(crate) enum MachinePortFailure {
    #[error("machine-state loan busy during {operation}")]
    LoanBusy { operation: &'static str },
    #[error("machine-state attempt failed during {operation}: {reason}")]
    Attempt {
        operation: &'static str,
        reason: String,
    },
}

/// The borrowed state is held once, so a Kernel loan conflicts with a
/// framebuffer loan and a reentrant callback cannot borrow either around it.
struct MachineState<'machine> {
    kernel: &'machine mut CeKernel,
    framebuffer: &'machine mut (dyn Framebuffer + 'machine),
}

/// A single-threaded, callback-scoped port. It owns no second Kernel, Task
/// directory, framebuffer or CPU resource; both references remain borrowed
/// from the caller until the port and all callback closures are gone.
pub(crate) struct MachineKernelPort<'machine> {
    state: RefCell<MachineState<'machine>>,
    first_terminal: OnceCell<MachinePortFailure>,
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
            first_terminal: OnceCell::new(),
        }
    }

    /// Latch the first terminal failure. A later native/mapping error cannot
    /// replace an earlier callback conflict (or vice versa).
    pub(crate) fn record_terminal(&self, failure: MachinePortFailure) -> MachinePortFailure {
        self.first_terminal.get_or_init(|| failure).clone()
    }

    pub(crate) fn terminal_failure(&self) -> Option<MachinePortFailure> {
        self.first_terminal.get().cloned()
    }

    /// The higher-ranked closure prevents a Kernel or framebuffer reference
    /// from escaping this loan. No guard may span `emu_start`, a nested child
    /// run, or a reentrant host callback. This is not yet a leaf-authority
    /// check: the connected caller must supply that separately. A closure may
    /// already have mutated state when a nested access latches failure; the
    /// terminal result prevents continuation but does not roll those effects
    /// back. The connected exit/partial-effect fence must handle that case.
    pub(crate) fn try_with_state<R>(
        &self,
        operation: &'static str,
        use_state: impl for<'loan> FnOnce(
            &'loan mut CeKernel,
            &'loan mut (dyn Framebuffer + 'machine),
        ) -> R,
    ) -> std::result::Result<R, MachinePortFailure> {
        if let Some(failure) = self.terminal_failure() {
            return Err(failure);
        }

        let mut borrowed = self
            .state
            .try_borrow_mut()
            .map_err(|_| self.record_terminal(MachinePortFailure::LoanBusy { operation }))?;
        let state = &mut *borrowed;
        let result = use_state(&mut *state.kernel, &mut *state.framebuffer);
        drop(borrowed);

        // A nested callback may have failed while the outer closure held the
        // loan. Never publish that closure's apparent success over the latch.
        if let Some(failure) = self.terminal_failure() {
            return Err(failure);
        }
        Ok(result)
    }

    pub(crate) fn try_with_kernel<R>(
        &self,
        operation: &'static str,
        use_kernel: impl for<'loan> FnOnce(&'loan mut CeKernel) -> R,
    ) -> std::result::Result<R, MachinePortFailure> {
        self.try_with_state(operation, |kernel, _| use_kernel(kernel))
    }

    pub(crate) fn try_with_framebuffer<R>(
        &self,
        operation: &'static str,
        use_framebuffer: impl for<'loan> FnOnce(&'loan mut dyn Framebuffer) -> R,
    ) -> std::result::Result<R, MachinePortFailure> {
        self.try_with_state(operation, |_, framebuffer| use_framebuffer(framebuffer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ce::{
            devices::{DeviceConfigFile, DeviceDefaults},
            framebuffer::{PixelFormat, VirtualFramebuffer},
            registry::RegistryDump,
        },
        config::{
            DEFAULT_GUEST_PHYSICAL_RAM_BYTES, ObjectStoreConfig, RootConfig, RuntimeConfig,
            StorageConfig,
        },
    };
    use std::{collections::BTreeMap, path::PathBuf};

    fn test_kernel() -> CeKernel {
        CeKernel::boot(RuntimeConfig {
            registry: RegistryDump {
                version: 1,
                source: None,
                keys: BTreeMap::new(),
            },
            devices: DeviceConfigFile {
                version: 1,
                defaults: DeviceDefaults::default(),
                devices: Vec::new(),
            },
            storage: StorageConfig {
                root: RootConfig {
                    host_root: PathBuf::from("."),
                },
                object_store: ObjectStoreConfig {
                    total_mbytes: 64,
                    free_mbytes: 32,
                },
                mounts: Vec::new(),
            },
            guest_physical_ram_bytes: DEFAULT_GUEST_PHYSICAL_RAM_BYTES,
        })
    }

    #[test]
    fn machine_port_returns_owned_observation_and_releases_both_loans() {
        let mut kernel = test_kernel();
        let mut framebuffer = VirtualFramebuffer::new(2, 2, PixelFormat::Gray8).unwrap();
        let port = MachineKernelPort::new(&mut kernel, &mut framebuffer);

        // The higher-ranked API returns only owned data here. A returned
        // `&mut CeKernel` or framebuffer slice cannot satisfy its signature.
        let (pid, pixels): (u32, Vec<u8>) = port
            .try_with_state("owned snapshot", |kernel, framebuffer| {
                framebuffer.pixels_mut()[0] = 7;
                (kernel.current_process_id(), framebuffer.pixels().to_vec())
            })
            .unwrap();
        assert_eq!(pid, 1);
        assert_eq!(pixels[0], 7);
        assert!(port.terminal_failure().is_none());

        drop(port);
        assert_eq!(kernel.current_process_id(), pid);
        assert_eq!(framebuffer.pixels()[0], 7);
    }

    #[test]
    fn machine_port_reentrant_cross_resource_loan_fails_without_panic() {
        let mut kernel = test_kernel();
        let mut framebuffer = VirtualFramebuffer::new(2, 2, PixelFormat::Gray8).unwrap();
        let port = MachineKernelPort::new(&mut kernel, &mut framebuffer);
        let busy = MachinePortFailure::LoanBusy {
            operation: "nested framebuffer",
        };

        let outer = port.try_with_kernel("outer kernel", |_| {
            port.try_with_framebuffer("nested framebuffer", |fb| fb.width())
        });
        assert_eq!(outer, Err(busy.clone()));
        assert_eq!(port.terminal_failure(), Some(busy.clone()));
        assert_eq!(port.try_with_kernel("later", |_| 5), Err(busy));
    }

    #[test]
    fn machine_port_keeps_first_terminal_over_later_attempt_errors() {
        let mut kernel = test_kernel();
        let mut framebuffer = VirtualFramebuffer::new(2, 2, PixelFormat::Gray8).unwrap();
        let port = MachineKernelPort::new(&mut kernel, &mut framebuffer);
        let first = MachinePortFailure::Attempt {
            operation: "import hook",
            reason: "checked authority failed".to_owned(),
        };

        assert_eq!(port.record_terminal(first.clone()), first);
        assert_eq!(
            port.record_terminal(MachinePortFailure::LoanBusy {
                operation: "later framebuffer callback"
            }),
            first
        );
        assert_eq!(
            port.try_with_state("later memory hook", |_, _| 1),
            Err(first)
        );
    }
}
