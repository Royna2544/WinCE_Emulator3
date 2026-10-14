//! Native engine boundary (module declaration must be Unicorn-gated).
//!
//! Fresh-per-slice is the default. A lease owns one hook-free native engine and
//! creates a lifetime-scoped, non-owning wrapper for run-local callbacks. On
//! fresh finish or abort, the owning wrapper drops while callback boxes and
//! backing pins remain alive; only the exact-key CPU snapshot may survive.
//! The vendored Unicorn 2.1.5 `uc_close` has only successful exits; changing
//! that source or the Rust wrapper invalidates this close proof (see
//! `vendor/unicorn-engine-sys/PATCHES.md`). Do not escape a raw/cloned wrapper.
//! Experimental persistent reuse is an explicit opt-in for Stage 8 only.
//!
//! K is the authentic (thread ID, saved scalar CPU context) resume key. P is the
//! engine's mapping ledger; cloning never copies a live engine or that ledger.

use crate::ce::memory::PrivatePageIdentity;
use std::{
    any::Any,
    cell::RefCell,
    collections::BTreeMap,
    fmt,
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::OnceLock,
};
use unicorn_engine::{
    Context, UcHookId, Unicorn,
    unicorn_const::{Arch, ContextMode, HookType, MemType, Mode, Query, uc_error},
};

struct NativeSnapshot<K> {
    key: K,
    cpu: Context,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativeEngineLifetime {
    FreshPerSlice,
    PersistentExperimental,
}

struct NativeSlot<K, P> {
    engine: Option<Unicorn<'static, ()>>,
    blank: Option<Rc<Context>>,
    saved: Option<Rc<NativeSnapshot<K>>>,
    metadata: Option<P>,
    mapping_pins: Option<NativeMappingPins>,
    leased: bool,
    #[cfg(test)]
    fail_next_finish_before_publication: bool,
    #[cfg(test)]
    fail_next_count_stop_query_after_native_value: Option<usize>,
}

/// Strong backing references tied to one native owner, independently of the
/// mapping metadata transferred to hook-local state during a run. A pending
/// pointer map is pinned before entering Unicorn; even an early error keeps
/// its storage alive until the owner has been discarded.
#[derive(Default)]
struct NativePinSet {
    generic: Vec<Rc<dyn Any>>,
    private_pages: BTreeMap<PrivatePageIdentity, PrivateNativePin>,
    image_runs: BTreeMap<u32, ImageNativePin>,
}

struct PrivateNativePin {
    host_address: usize,
    _backing: Rc<dyn Any>,
}

struct ImageNativePin {
    host_address: usize,
    len: u32,
    _backing: Rc<dyn Any>,
}

#[derive(Clone, Default)]
pub(crate) struct NativeMappingPins(Rc<RefCell<NativePinSet>>);

impl NativeMappingPins {
    pub(crate) fn pin<T: 'static>(&self, backing: T) {
        self.0.borrow_mut().generic.push(Rc::new(backing));
    }

    /// Pin a raw-pointer page before handing it to Unicorn. The exact page
    /// identity is retired only after checked native unmap succeeds.
    pub(crate) fn pin_private<T: 'static>(
        &self,
        identity: PrivatePageIdentity,
        host_address: usize,
        backing: T,
    ) -> bool {
        let mut pins = self.0.borrow_mut();
        if pins.private_pages.contains_key(&identity) {
            return false;
        }
        pins.private_pages.insert(
            identity,
            PrivateNativePin {
                host_address,
                _backing: Rc::new(backing),
            },
        );
        true
    }

    pub(crate) fn release_private_after_unmap(
        &self,
        identity: PrivatePageIdentity,
        expected_host_address: usize,
    ) -> bool {
        let mut pins = self.0.borrow_mut();
        if !pins
            .private_pages
            .get(&identity)
            .is_some_and(|pin| pin.host_address == expected_host_address)
        {
            return false;
        }
        pins.private_pages.remove(&identity).is_some()
    }

    /// Retain one exact sealed image run before its pointer enters Unicorn.
    /// Reused owners validate this same pointer and extent against their
    /// mapping ledger; a pin is retired only after a checked native unmap.
    pub(crate) fn pin_image<T: 'static>(
        &self,
        base: u32,
        host_address: usize,
        len: u32,
        backing: T,
    ) -> bool {
        let mut pins = self.0.borrow_mut();
        if pins.image_runs.contains_key(&base) {
            return false;
        }
        pins.image_runs.insert(
            base,
            ImageNativePin {
                host_address,
                len,
                _backing: Rc::new(backing),
            },
        );
        true
    }

    pub(crate) fn has_image(&self, base: u32, host_address: usize, len: u32) -> bool {
        self.0.borrow().image_runs.get(&base).is_some_and(|pin| {
            pin.host_address == host_address && pin.len == len
        })
    }

    pub(crate) fn image_len(&self) -> usize {
        self.0.borrow().image_runs.len()
    }

    pub(crate) fn release_image_after_unmap(
        &self,
        base: u32,
        expected_host_address: usize,
        expected_len: u32,
    ) -> bool {
        let mut pins = self.0.borrow_mut();
        if !pins.image_runs.get(&base).is_some_and(|pin| {
            pin.host_address == expected_host_address && pin.len == expected_len
        }) {
            return false;
        }
        pins.image_runs.remove(&base).is_some()
    }

    pub(crate) fn len(&self) -> usize {
        let pins = self.0.borrow();
        pins.generic.len() + pins.private_pages.len() + pins.image_runs.len()
    }

    pub(crate) fn private_len(&self) -> usize {
        self.0.borrow().private_pages.len()
    }
}

struct PinQuarantine(RefCell<Vec<NativeMappingPins>>);

impl Drop for PinQuarantine {
    fn drop(&mut self) {
        // Thread exit is not proof that uc_close succeeded. Deliberately leak
        // the remaining strong pins until host-process exit.
        for pins in self.0.get_mut().drain(..) {
            std::mem::forget(pins);
        }
    }
}

thread_local! {
    // unicorn-engine 2.1.5 discards uc_close's result in Drop. A discarded
    // native owner therefore cannot prove its raw pointer mappings are gone.
    // Keep diagnostic backing alive until thread exit (or leak at thread-local
    // teardown) rather than risk releasing a host page still mapped by C.
    static NATIVE_PIN_QUARANTINE: PinQuarantine = const { PinQuarantine(RefCell::new(Vec::new())) };
}

fn quarantine_unchecked_close_pins(pins: NativeMappingPins) {
    if pins.len() == 0 {
        return;
    }
    let stored = NATIVE_PIN_QUARANTINE
        .try_with(|quarantine| {
            quarantine
                .0
                .try_borrow_mut()
                .map(|mut quarantine| quarantine.push(pins.clone()))
                .is_ok()
        })
        .unwrap_or(false);
    if !stored {
        std::mem::forget(pins);
    }
}

impl<K, P> Drop for NativeSlot<K, P> {
    fn drop(&mut self) {
        if let Some(pins) = self.mapping_pins.take() {
            quarantine_unchecked_close_pins(pins);
        }
        // The native owner drops after this method, while the quarantine
        // retains any pointer backing even if uc_close silently fails.
    }
}

pub(crate) struct ProcessNativeEngine<K, P = ()> {
    arch: Arch,
    mode: Mode,
    lifetime: NativeEngineLifetime,
    slot: Rc<RefCell<NativeSlot<K, P>>>,
}

impl<K, P> fmt::Debug for ProcessNativeEngine<K, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slot = self.slot.borrow();
        f.debug_struct("ProcessNativeEngine")
            .field("arch", &self.arch)
            .field("mode", &self.mode)
            .field("lifetime", &self.lifetime)
            .field("leased", &slot.leased)
            .field("initialized", &slot.engine.is_some())
            .field("has_snapshot", &slot.saved.is_some())
            .field("has_metadata", &slot.metadata.is_some())
            .finish()
    }
}

impl<K, P> Clone for ProcessNativeEngine<K, P> {
    fn clone(&self) -> Self {
        let slot = self.slot.borrow();
        // NativeSnapshot is immutable: neither Context::reg_write nor mutable
        // context access is exposed. CPU-only restore does not mutate it.
        // A live lease is not a checkpoint boundary; do not copy its older key.
        let saved = if slot.leased {
            None
        } else {
            slot.saved.clone()
        };
        Self {
            arch: self.arch,
            mode: self.mode,
            lifetime: self.lifetime,
            slot: Rc::new(RefCell::new(NativeSlot {
                engine: None,
                blank: None,
                saved,
                metadata: None,
                mapping_pins: None,
                leased: false,
                #[cfg(test)]
                fail_next_finish_before_publication: false,
                #[cfg(test)]
                fail_next_count_stop_query_after_native_value: None,
            })),
        }
    }
}

impl<K, P> ProcessNativeEngine<K, P> {
    pub(crate) fn new(arch: Arch, mode: Mode) -> Self {
        // Read the Stage 8 experiment switch once for the host process.
        // Clones and parked CPUs retain their originally selected lifetime.
        static DEFAULT_LIFETIME: OnceLock<NativeEngineLifetime> = OnceLock::new();
        let lifetime = *DEFAULT_LIFETIME.get_or_init(|| {
            if matches!(
                std::env::var("WINCE_EMU_PERSISTENT_NATIVE_ENGINE"),
                Ok(value) if value == "1"
            ) {
                NativeEngineLifetime::PersistentExperimental
            } else {
                NativeEngineLifetime::FreshPerSlice
            }
        });
        Self::with_lifetime(arch, mode, lifetime)
    }

    pub(crate) fn with_lifetime(arch: Arch, mode: Mode, lifetime: NativeEngineLifetime) -> Self {
        Self {
            arch,
            mode,
            lifetime,
            slot: Rc::new(RefCell::new(NativeSlot {
                engine: None,
                blank: None,
                saved: None,
                metadata: None,
                mapping_pins: None,
                leased: false,
                #[cfg(test)]
                fail_next_finish_before_publication: false,
                #[cfg(test)]
                fail_next_count_stop_query_after_native_value: None,
            })),
        }
    }

    #[cfg(test)]
    pub(crate) fn new_persistent_for_test(arch: Arch, mode: Mode) -> Self {
        Self::with_lifetime(arch, mode, NativeEngineLifetime::PersistentExperimental)
    }

    /// Owns the slot rather than borrowing a field of the enclosing CPU.
    pub(crate) fn lease<'a>(&self) -> Result<NativeEngineLease<'a, K, P>, uc_error> {
        let mut slot = self.slot.borrow_mut();
        if slot.leased {
            return Err(uc_error::RESOURCE);
        }
        let reused = slot.engine.is_some();
        if self.lifetime == NativeEngineLifetime::FreshPerSlice && reused {
            // A closed owner's ledger/blank context must never be interpreted
            // as a fresh lease, even if an earlier path violated the policy.
            return Err(uc_error::RESOURCE);
        }
        let setup = (|| {
            let (mut owner, blank) = match slot.engine.take() {
                Some(owner) => (owner, slot.blank.take().ok_or(uc_error::HANDLE)?),
                None => {
                    let mut owner: Unicorn<'static, ()> = Unicorn::new(self.arch, self.mode)?;
                    owner.ctl_set_context_mode(ContextMode::CPU)?;
                    let blank = Rc::new(owner.context_init()?);
                    (owner, blank)
                }
            };
            owner.ctl_set_context_mode(ContextMode::CPU)?;
            // SAFETY: owner is alive and moves into this lease. This wrapper is
            // ffi=true, so it cannot close the handle. No owner API is used
            // while the wrapper is live, and all its hooks are removed before
            // reuse (or owner closes the handle before callback Boxes drop).
            let uc: Unicorn<'a, ()> = unsafe { Unicorn::from_handle(owner.get_handle())? };
            Ok::<_, uc_error>((owner, blank, uc))
        })();
        let (owner, blank, uc) = match setup {
            Ok(parts) => parts,
            Err(err) => {
                slot.blank = None;
                slot.saved = None;
                slot.metadata = None;
                if let Some(pins) = slot.mapping_pins.take() {
                    quarantine_unchecked_close_pins(pins);
                }
                return Err(err);
            }
        };
        slot.leased = true;
        let metadata = slot.metadata.take();
        let mapping_pins = slot.mapping_pins.take().unwrap_or_default();
        let saved = slot.saved.clone();
        Ok(NativeEngineLease {
            uc: Some(uc),
            owner: Some(owner),
            blank,
            saved,
            metadata,
            mapping_pins,
            slot: Rc::clone(&self.slot),
            hooks: Vec::new(),
            reused,
            lifetime: self.lifetime,
            closed: false,
        })
    }

    #[cfg(test)]
    pub(crate) fn identity(&self) -> Option<usize> {
        self.slot
            .borrow()
            .engine
            .as_ref()
            .map(|engine| engine.get_handle() as usize)
    }

    /// Exercise a finish failure after policy-specific teardown preflight but
    /// before CPU/ledger publication. This is not a C `uc_close` failure.
    #[cfg(test)]
    pub(crate) fn arm_finish_failure_once_for_test(&self) {
        let mut slot = self.slot.borrow_mut();
        assert!(!slot.leased, "arm only before the native attempt");
        slot.fail_next_finish_before_publication = true;
    }

    /// Test only: force the Rust adapter to observe a COUNT_STOP query error
    /// after a real native query returns the expected value. This does not
    /// claim that the vendored C `uc_query` itself failed.
    #[cfg(test)]
    pub(crate) fn arm_count_stop_query_error_once_for_test(&self, expected_native_value: usize) {
        let mut slot = self.slot.borrow_mut();
        assert!(!slot.leased, "arm only before the native attempt");
        assert!(
            slot.fail_next_count_stop_query_after_native_value.is_none(),
            "query fault is already armed"
        );
        slot.fail_next_count_stop_query_after_native_value = Some(expected_native_value);
    }
}

pub(crate) struct NativeEngineLease<'a, K, P> {
    uc: Option<Unicorn<'a, ()>>,
    owner: Option<Unicorn<'static, ()>>,
    blank: Rc<Context>,
    saved: Option<Rc<NativeSnapshot<K>>>,
    metadata: Option<P>,
    mapping_pins: NativeMappingPins,
    slot: Rc<RefCell<NativeSlot<K, P>>>,
    hooks: Vec<UcHookId>,
    reused: bool,
    lifetime: NativeEngineLifetime,
    closed: bool,
}

impl<'a, K, P> Deref for NativeEngineLease<'a, K, P> {
    type Target = Unicorn<'a, ()>;

    fn deref(&self) -> &Self::Target {
        self.uc.as_ref().expect("native lease is open")
    }
}

impl<K, P> DerefMut for NativeEngineLease<'_, K, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.uc.as_mut().expect("native lease is open")
    }
}

impl<'a, K, P> NativeEngineLease<'a, K, P> {
    pub(crate) fn reused(&self) -> bool {
        self.reused
    }

    /// Read the count-stop witness on the current native lease. The test-only
    /// fault seam verifies the real C query before substituting one error at
    /// this Rust adapter boundary; ordinary builds return the native result.
    pub(crate) fn query_count_stop(&self) -> Result<usize, uc_error> {
        let native = self
            .uc
            .as_ref()
            .expect("native lease is open")
            .query(Query::COUNT_STOP);
        #[cfg(test)]
        if let Some(expected) = self
            .slot
            .borrow_mut()
            .fail_next_count_stop_query_after_native_value
            .take()
        {
            assert!(
                matches!(&native, Ok(actual) if *actual == expected),
                "test query fault expected native COUNT_STOP={expected}, got {native:?}"
            );
            return Err(uc_error::RESOURCE);
        }
        native
    }

    #[cfg(test)]
    pub(crate) fn identity(&self) -> usize {
        self.get_handle() as usize
    }

    pub(crate) fn take_metadata(&mut self) -> Option<P> {
        self.metadata.take()
    }

    /// A handle to the lease-resident backing pins. Hook-local clones may
    /// append pending pins, but cannot make the lease release them early.
    pub(crate) fn mapping_pins(&self) -> NativeMappingPins {
        self.mapping_pins.clone()
    }

    pub(crate) fn set_metadata(&mut self, metadata: P) {
        self.metadata = Some(metadata);
    }

    pub(crate) fn add_code_hook<F>(
        &mut self,
        begin: u64,
        end: u64,
        callback: F,
    ) -> Result<UcHookId, uc_error>
    where
        F: FnMut(&mut Unicorn<'_, ()>, u64, u32) + 'a,
    {
        let id = self
            .uc
            .as_mut()
            .unwrap()
            .add_code_hook(begin, end, callback)?;
        self.hooks.push(id);
        Ok(id)
    }

    pub(crate) fn add_block_hook<F>(
        &mut self,
        begin: u64,
        end: u64,
        callback: F,
    ) -> Result<UcHookId, uc_error>
    where
        F: FnMut(&mut Unicorn<'_, ()>, u64, u32) + 'a,
    {
        let id = self
            .uc
            .as_mut()
            .unwrap()
            .add_block_hook(begin, end, callback)?;
        self.hooks.push(id);
        Ok(id)
    }

    pub(crate) fn add_mem_hook<F>(
        &mut self,
        kind: HookType,
        begin: u64,
        end: u64,
        callback: F,
    ) -> Result<UcHookId, uc_error>
    where
        F: FnMut(&mut Unicorn<'_, ()>, MemType, u64, usize, i64) -> bool + 'a,
    {
        let id = self
            .uc
            .as_mut()
            .unwrap()
            .add_mem_hook(kind, begin, end, callback)?;
        self.hooks.push(id);
        Ok(id)
    }

    pub(crate) fn add_intr_hook<F>(&mut self, callback: F) -> Result<UcHookId, uc_error>
    where
        F: FnMut(&mut Unicorn<'_, ()>, u32) + 'a,
    {
        let id = self.uc.as_mut().unwrap().add_intr_hook(callback)?;
        self.hooks.push(id);
        Ok(id)
    }

    pub(crate) fn add_insn_invalid_hook<F>(&mut self, callback: F) -> Result<UcHookId, uc_error>
    where
        F: FnMut(&mut Unicorn<'_, ()>) -> bool + 'a,
    {
        let id = self.uc.as_mut().unwrap().add_insn_invalid_hook(callback)?;
        self.hooks.push(id);
        Ok(id)
    }

    /// Call only at the primary native return boundary, before host repairs.
    /// K must describe that actual native CPU and its authentic thread owner.
    pub(crate) fn capture(&mut self, key: K) -> Result<(), uc_error> {
        self.ctl_set_context_mode(ContextMode::CPU)?;
        let cpu = self.context_init()?;
        self.saved = Some(Rc::new(NativeSnapshot { key, cpu }));
        Ok(())
    }

    /// Retain hidden CPU state only for the same authentic scalar/TID key.
    /// Required before an entry with no saved continuation, even on reuse.
    pub(crate) fn reset_to_blank(&mut self) -> Result<(), uc_error> {
        self.saved = None;
        self.ctl_set_context_mode(ContextMode::CPU)?;
        self.context_restore(&self.blank)
    }

    /// Retain hidden CPU state only for the same authentic scalar/TID key.
    /// False means the caller must establish scalar entry/callout registers
    /// after a checked reset to this engine's initial blank CPU context.
    pub(crate) fn restore_if_exact(&mut self, key: &K) -> Result<bool, uc_error>
    where
        K: PartialEq,
    {
        self.ctl_set_context_mode(ContextMode::CPU)?;
        if let Some(saved) = self.saved.as_ref().filter(|saved| saved.key == *key) {
            self.context_restore(&saved.cpu)?;
            return Ok(true);
        }
        self.reset_to_blank()?;
        Ok(false)
    }

    /// Commit the exact CPU snapshot only after complete native teardown.
    /// Persistent experimental owners instead commit reuse and their ledger.
    pub(crate) fn finish(mut self) -> Result<(), uc_error> {
        self.close(true)
    }

    /// Failure boundary: never publish a reusable owner or CPU snapshot.
    /// Fresh owners close before releasing callback boxes and backing pins.
    pub(crate) fn abort_close(mut self) -> Result<(), uc_error> {
        self.close(false)
    }

    fn close(&mut self, commit: bool) -> Result<(), uc_error> {
        if self.closed {
            return Ok(());
        }
        if self.lifetime == NativeEngineLifetime::FreshPerSlice {
            // `uc_close` in the pinned vendored 2.1.5 C source has only
            // UC_ERR_OK exits (uc.c:497-574). A fresh owner is unique and
            // hook-free itself; its nonowning run wrapper retains all Rust
            // callback Boxes. Do not call remove_hook: that Rust wrapper drops
            // a Box before its C call, and deletion is unnecessary when the
            // complete native owner is closing outside emulation.
            let failure = if commit && self.metadata.is_none() {
                Some(uc_error::ARG)
            } else {
                None
            };
            #[cfg(test)]
            let failure = if commit
                && failure.is_none()
                && std::mem::take(&mut self.slot.borrow_mut().fail_next_finish_before_publication)
            {
                Some(uc_error::RESOURCE)
            } else {
                failure
            };
            let owner = self
                .owner
                .take()
                .expect("fresh native lease lost its owner");
            let callbacks = self
                .uc
                .take()
                .expect("fresh native lease lost its run wrapper");
            drop(owner); // native hooks/maps are gone before Rust captures or pins
            drop(callbacks);
            let mut slot = self.slot.borrow_mut();
            slot.engine = None;
            slot.blank = None;
            slot.saved = if commit && failure.is_none() {
                self.saved.take()
            } else {
                None
            };
            slot.metadata = None;
            slot.mapping_pins = None;
            slot.leased = false;
            #[cfg(test)]
            {
                slot.fail_next_finish_before_publication = false;
                slot.fail_next_count_stop_query_after_native_value = None;
            }
            self.closed = true;
            return match failure {
                Some(err) => Err(err),
                None => Ok(()),
            };
        }

        // Stage 8 persistent opt-in: hooks must be removed before this owner
        // can be reused. The fresh-default close proof does not accept this
        // experimental branch as production-safe.
        let mut failure = None;
        while let Some(id) = self.hooks.pop() {
            if let Err(err) = self.uc.as_mut().unwrap().remove_hook(id) {
                failure = Some(err);
                break;
            }
        }
        if commit && self.metadata.is_none() && failure.is_none() {
            failure = Some(uc_error::ARG);
        }
        #[cfg(test)]
        if commit && failure.is_none() {
            // Remove real hooks and check the ledger first; then make this
            // owner fail exactly once through the ordinary discard path.
            if std::mem::take(&mut self.slot.borrow_mut().fail_next_finish_before_publication) {
                failure = Some(uc_error::RESOURCE);
            }
        }
        if commit && failure.is_none() {
            // Native hooks are deleted before freeing the borrowed wrapper.
            // Only then may the owning handle and matching ledger be reused.
            self.uc.take();
            let mut slot = self.slot.borrow_mut();
            slot.engine = self.owner.take();
            slot.blank = Some(Rc::clone(&self.blank));
            slot.saved = self.saved.take();
            slot.metadata = self.metadata.take();
            slot.mapping_pins = Some(self.mapping_pins.clone());
            slot.leased = false;
        } else {
            // remove_hook drops its Box before its FFI delete. On any failure
            // there must be no future native entry: close the native handle
            // before dropping the remaining borrowed callback Boxes.
            quarantine_unchecked_close_pins(self.mapping_pins.clone());
            self.owner.take();
            self.uc.take();
            let mut slot = self.slot.borrow_mut();
            slot.engine = None;
            slot.blank = None;
            slot.saved = None;
            slot.metadata = None;
            slot.mapping_pins = None;
            slot.leased = false;
            #[cfg(test)]
            {
                slot.fail_next_finish_before_publication = false;
                slot.fail_next_count_stop_query_after_native_value = None;
            }
        }
        self.closed = true;
        match failure {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}

impl<K, P> Drop for NativeEngineLease<'_, K, P> {
    fn drop(&mut self) {
        // Unfinished leases are deliberately not a reusable state boundary.
        let _ = self.close(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use unicorn_engine::{RegisterMIPS, unicorn_const::Prot};

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Key {
        tid: u32,
        pc: u32,
        regs: [u32; 32],
        hi: u32,
        lo: u32,
    }

    fn key(uc: &Unicorn<'_, ()>, tid: u32) -> Key {
        let mut regs = [0; 32];
        for (index, value) in regs.iter_mut().enumerate() {
            *value = uc.reg_read(RegisterMIPS::R0 as i32 + index as i32).unwrap() as u32;
        }
        Key {
            tid,
            pc: uc.reg_read(RegisterMIPS::PC).unwrap() as u32,
            regs,
            hi: uc.reg_read(RegisterMIPS::HI).unwrap() as u32,
            lo: uc.reg_read(RegisterMIPS::LO).unwrap() as u32,
        }
    }

    fn slot() -> ProcessNativeEngine<Key, Vec<(u64, u64)>> {
        // Retain the old reuse/teardown diagnostics under explicit Stage 8
        // policy. Production `new` now chooses fresh-per-slice by default.
        ProcessNativeEngine::new_persistent_for_test(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN)
    }

    fn mapped(lease: &mut NativeEngineLease<'_, Key, Vec<(u64, u64)>>) {
        lease.mem_map(0x1000, 0x1000, Prot::ALL).unwrap();
        let words = [0x2408_0007u32, 0x3c09_0000, 0x3529_1800, 0xad28_0000];
        let bytes: Vec<u8> = words.into_iter().flat_map(u32::to_le_bytes).collect();
        lease.mem_write(0x1000, &bytes).unwrap();
        lease.set_metadata(vec![(0x1000, 0x1000)]);
    }

    #[test]
    fn native_engine_checked_finish_reuses_handle_and_metadata() {
        let slot = slot();
        let mut lease = slot.lease().unwrap();
        assert!(!lease.reused());
        assert!(lease.take_metadata().is_none());
        mapped(&mut lease);
        let identity = lease.identity();
        lease.emu_start(0x1000, 0x1010, 100_000, 16).unwrap();
        lease.finish().unwrap();
        assert_eq!(slot.identity(), Some(identity));
        let mut next = slot.lease().unwrap();
        assert!(next.reused());
        assert_eq!(next.identity(), identity);
        let metadata = next.take_metadata().unwrap();
        assert_eq!(metadata, vec![(0x1000, 0x1000)]);
        assert_eq!(next.mem_read_as_vec(0x1800, 4).unwrap(), 7u32.to_le_bytes());
        next.set_metadata(metadata);
        next.finish().unwrap();
    }

    #[test]
    fn injected_checked_finish_failure_discards_owner_and_is_one_shot() {
        let slot = slot();
        slot.arm_finish_failure_once_for_test();
        let mut first = slot.lease().unwrap();
        mapped(&mut first);
        first.add_code_hook(0x1000, 0x1010, |_, _, _| {}).unwrap();
        first.emu_start(0x1000, 0x1010, 100_000, 16).unwrap();
        assert_eq!(first.finish(), Err(uc_error::RESOURCE));
        assert!(
            slot.identity().is_none(),
            "failed finish cannot publish owner"
        );

        let mut second = slot.lease().unwrap();
        assert!(!second.reused(), "replacement owner must be fresh");
        assert!(
            second.take_metadata().is_none(),
            "failed ledger was discarded"
        );
        mapped(&mut second);
        second.finish().unwrap();
        assert!(
            slot.identity().is_some(),
            "one-shot failure cannot fire twice"
        );
    }

    #[test]
    fn fresh_finish_failure_closes_owner_before_releasing_pins() {
        struct LivePinnedPage {
            bytes: Box<[u8; 0x1000]>,
            drops: Rc<Cell<u32>>,
        }

        impl Drop for LivePinnedPage {
            fn drop(&mut self) {
                self.drops.set(self.drops.get() + 1);
            }
        }

        let slot: ProcessNativeEngine<Key, Vec<(u64, u64)>> = ProcessNativeEngine::with_lifetime(
            Arch::MIPS,
            Mode::MIPS32 | Mode::LITTLE_ENDIAN,
            NativeEngineLifetime::FreshPerSlice,
        );
        let drops = Rc::new(Cell::new(0));
        slot.arm_finish_failure_once_for_test();
        let mut lease = slot.lease().unwrap();
        mapped(&mut lease);
        let mut page = LivePinnedPage {
            bytes: Box::new([0; 0x1000]),
            drops: Rc::clone(&drops),
        };
        let ptr = page.bytes.as_mut_ptr();
        lease.mapping_pins().pin(page);
        // SAFETY: the page is fixed-size and pinned until this owner closes.
        unsafe { lease.mem_map_ptr(0x3000, 0x1000, Prot::ALL, ptr.cast()) }.unwrap();
        lease.set_metadata(vec![(0x1000, 0x1000), (0x3000, 0x1000)]);
        let words = [0x2408_0007u32, 0x3c09_0000, 0x3529_3000, 0xad28_0000];
        let program: Vec<u8> = words.into_iter().flat_map(u32::to_le_bytes).collect();
        lease.mem_write(0x1000, &program).unwrap();
        lease.add_code_hook(0x1000, 0x1010, |_, _, _| {}).unwrap();
        lease.emu_start(0x1000, 0x1010, 100_000, 16).unwrap();
        assert_eq!(
            lease.mem_read_as_vec(0x3000, 4).unwrap(),
            7u32.to_le_bytes()
        );
        assert_eq!(drops.get(), 0, "live mapped backing must outlive execution");
        assert_eq!(lease.finish(), Err(uc_error::RESOURCE));
        assert_eq!(drops.get(), 1, "closed fresh owner retained a pin");
        assert!(slot.identity().is_none());
        let mut replacement = slot.lease().unwrap();
        assert!(!replacement.reused());
        assert!(replacement.take_metadata().is_none());
        mapped(&mut replacement);
        replacement.finish().unwrap();
        assert!(slot.identity().is_none());
    }

    #[test]
    fn native_engine_unfinished_or_ledgerless_lease_is_discarded() {
        let slot = slot();
        {
            let mut lease = slot.lease().unwrap();
            mapped(&mut lease);
            assert!(
                slot.lease().is_err(),
                "no concurrent wrapper over one owner"
            );
        }
        assert!(slot.identity().is_none());
        let mut lease = slot.lease().unwrap();
        assert!(!lease.reused());
        assert!(lease.take_metadata().is_none());
        assert_eq!(lease.finish(), Err(uc_error::ARG));
        assert!(slot.identity().is_none());
    }

    #[test]
    fn native_mapping_pin_outlives_metadata_transfer_and_unchecked_close() {
        let slot = slot();
        let dropped = Rc::new(Cell::new(0));
        let mut first = slot.lease().unwrap();
        mapped(&mut first);
        first.mapping_pins().pin(Dropped(Rc::clone(&dropped)));
        assert_eq!(first.mapping_pins().len(), 1);
        first.finish().unwrap();
        assert_eq!(dropped.get(), 0, "reused owner still holds its pin");

        let mut second = slot.lease().unwrap();
        assert!(second.reused());
        let _metadata = second.take_metadata().unwrap();
        assert_eq!(second.mapping_pins().len(), 1);
        second.abort_close().unwrap();
        assert!(slot.identity().is_none());
        assert_eq!(
            dropped.get(),
            0,
            "unchecked upstream uc_close leaves diagnostic backing quarantined"
        );
    }

    #[test]
    fn quarantined_native_pin_survives_host_thread_exit() {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };

        struct ThreadDrop(Arc<AtomicUsize>);
        impl Drop for ThreadDrop {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }

        let drops = Arc::new(AtomicUsize::new(0));
        let observer = Arc::clone(&drops);
        std::thread::spawn(move || {
            let pins = NativeMappingPins::default();
            pins.pin(ThreadDrop(observer));
            quarantine_unchecked_close_pins(pins);
        })
        .join()
        .unwrap();
        assert_eq!(
            drops.load(Ordering::SeqCst),
            0,
            "thread exit cannot retire unchecked native pointer backing"
        );
    }

    struct Dropped(Rc<Cell<u32>>);
    impl Drop for Dropped {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    #[test]
    fn native_engine_all_five_hook_kinds_are_removed_before_reuse() {
        let slot = slot();
        let dropped = Rc::new(Cell::new(0));
        let code_hits = Rc::new(Cell::new(0));
        let mut lease = slot.lease().unwrap();
        mapped(&mut lease);
        let code_guard = Dropped(Rc::clone(&dropped));
        let hits = Rc::clone(&code_hits);
        lease
            .add_code_hook(1, 0, move |_, _, _| {
                let _ = &code_guard;
                hits.set(hits.get() + 1);
            })
            .unwrap();
        let block_guard = Dropped(Rc::clone(&dropped));
        lease
            .add_block_hook(1, 0, move |_, _, _| {
                let _ = &block_guard;
            })
            .unwrap();
        let mem_guard = Dropped(Rc::clone(&dropped));
        lease
            .add_mem_hook(HookType::MEM_WRITE, 1, 0, move |_, _, _, _, _| {
                let _ = &mem_guard;
                true
            })
            .unwrap();
        let intr_guard = Dropped(Rc::clone(&dropped));
        lease
            .add_intr_hook(move |_, _| {
                let _ = &intr_guard;
            })
            .unwrap();
        let invalid_guard = Dropped(Rc::clone(&dropped));
        lease
            .add_insn_invalid_hook(move |_| {
                let _ = &invalid_guard;
                false
            })
            .unwrap();
        lease.emu_start(0x1000, 0x1010, 100_000, 16).unwrap();
        assert!(code_hits.get() != 0);
        let old_hits = code_hits.get();
        lease.finish().unwrap();
        assert_eq!(dropped.get(), 5);
        let mut next = slot.lease().unwrap();
        next.emu_start(0x1000, 0x1010, 100_000, 16).unwrap();
        assert_eq!(
            code_hits.get(),
            old_hits,
            "old callbacks cannot execute again"
        );
        let metadata = next.take_metadata().unwrap();
        next.set_metadata(metadata);
        next.finish().unwrap();
    }

    #[test]
    fn native_engine_exact_context_restores_fpu_and_scalar_key() {
        let slot = slot();
        let mut lease = slot.lease().unwrap();
        mapped(&mut lease);
        lease.reg_write(RegisterMIPS::F0, 0x3f80_0000).unwrap();
        lease.reg_write(RegisterMIPS::HI, 0x1234_5678).unwrap();
        lease.reg_write(RegisterMIPS::LO, 0x9abc_def0).unwrap();
        lease.emu_start(0x1000, 0x1010, 100_000, 16).unwrap();
        let saved = key(&lease, 7);
        lease.capture(saved.clone()).unwrap();
        lease.finish().unwrap();
        let mut next = slot.lease().unwrap();
        next.reg_write(RegisterMIPS::F0, 0x4000_0000).unwrap();
        next.reg_write(RegisterMIPS::T0, 99).unwrap();
        assert!(next.restore_if_exact(&saved).unwrap());
        assert_eq!(key(&next, 7), saved);
        assert_eq!(next.reg_read(RegisterMIPS::F0).unwrap(), 0x3f80_0000);
        let metadata = next.take_metadata().unwrap();
        next.set_metadata(metadata);
        next.finish().unwrap();
    }

    #[test]
    fn native_engine_mismatched_tid_or_scalar_key_resets_hidden_state() {
        for mismatch_tid in [false, true] {
            let slot = slot();
            let mut lease = slot.lease().unwrap();
            mapped(&mut lease);
            let blank_f0 = lease.reg_read(RegisterMIPS::F0).unwrap();
            lease.reg_write(RegisterMIPS::F0, 0x3f80_0000).unwrap();
            lease.emu_start(0x1000, 0x1010, 100_000, 16).unwrap();
            let saved = key(&lease, 7);
            lease.capture(saved.clone()).unwrap();
            lease.finish().unwrap();
            let mut requested = saved.clone();
            if mismatch_tid {
                requested.tid += 1;
            } else {
                requested.pc += 4;
            }
            let mut next = slot.lease().unwrap();
            assert!(!next.restore_if_exact(&requested).unwrap());
            assert_eq!(next.reg_read(RegisterMIPS::F0).unwrap(), blank_f0);
            assert_eq!(next.reg_read(RegisterMIPS::PC).unwrap(), 0);
            assert_eq!(next.reg_read(RegisterMIPS::T0).unwrap(), 0);
            assert!(
                !next.restore_if_exact(&saved).unwrap(),
                "rejected snapshot stays cleared"
            );
            let metadata = next.take_metadata().unwrap();
            next.set_metadata(metadata);
            next.finish().unwrap();
        }
    }

    #[test]
    fn native_engine_no_saved_entry_resets_reused_cpu_before_entry_setup() {
        let slot = slot();
        let mut lease = slot.lease().unwrap();
        mapped(&mut lease);
        let blank_f0 = lease.reg_read(RegisterMIPS::F0).unwrap();
        lease.reg_write(RegisterMIPS::F0, 0x3f80_0000).unwrap();
        lease.reg_write(RegisterMIPS::S0, 0x1234).unwrap();
        lease.reg_write(RegisterMIPS::HI, 0x1122).unwrap();
        lease.reg_write(RegisterMIPS::LO, 0x3344).unwrap();
        lease.emu_start(0x1000, 0x1010, 100_000, 16).unwrap();
        let saved = key(&lease, 7);
        lease.capture(saved.clone()).unwrap();
        lease.finish().unwrap();
        let mut next = slot.lease().unwrap();
        assert!(next.reused());
        assert_eq!(next.reg_read(RegisterMIPS::F0).unwrap(), 0x3f80_0000);
        next.reset_to_blank().unwrap();
        assert_eq!(next.reg_read(RegisterMIPS::F0).unwrap(), blank_f0);
        for register in [
            RegisterMIPS::PC,
            RegisterMIPS::S0,
            RegisterMIPS::HI,
            RegisterMIPS::LO,
        ] {
            assert_eq!(next.reg_read(register).unwrap(), 0);
        }
        assert!(!next.restore_if_exact(&saved).unwrap());
        next.reg_write(RegisterMIPS::SP, 0x1f00).unwrap();
        next.reg_write(RegisterMIPS::RA, 0x1010).unwrap();
        next.emu_start(0x1000, 0x1010, 100_000, 16).unwrap();
        assert_eq!(next.reg_read(RegisterMIPS::SP).unwrap(), 0x1f00);
        assert_eq!(next.reg_read(RegisterMIPS::RA).unwrap(), 0x1010);
        assert_eq!(next.reg_read(RegisterMIPS::F0).unwrap(), blank_f0);
        let metadata = next.take_metadata().unwrap();
        next.set_metadata(metadata);
        next.finish().unwrap();
    }

    #[test]
    fn native_engine_clone_has_independent_handle_and_ram_with_immutable_cpu_snapshot() {
        let original = slot();
        let mut lease = original.lease().unwrap();
        mapped(&mut lease);
        lease.reg_write(RegisterMIPS::F0, 0x3f80_0000).unwrap();
        lease.emu_start(0x1000, 0x1010, 100_000, 16).unwrap();
        let saved = key(&lease, 7);
        lease.capture(saved.clone()).unwrap();
        let old_identity = lease.identity();
        lease.finish().unwrap();
        let cloned = original.clone();
        assert!(cloned.identity().is_none());
        let mut clone_lease = cloned.lease().unwrap();
        assert!(!clone_lease.reused());
        assert_ne!(clone_lease.identity(), old_identity);
        assert!(clone_lease.take_metadata().is_none());
        mapped(&mut clone_lease); // Host integration must copy its independent RAM separately.
        assert!(clone_lease.restore_if_exact(&saved).unwrap());
        assert_eq!(key(&clone_lease, 7), saved);
        assert_eq!(clone_lease.reg_read(RegisterMIPS::F0).unwrap(), 0x3f80_0000);
        clone_lease.mem_write(0x1800, &44u32.to_le_bytes()).unwrap();
        clone_lease.finish().unwrap();
        let mut original_lease = original.lease().unwrap();
        assert_eq!(original_lease.identity(), old_identity);
        assert_eq!(
            original_lease.mem_read_as_vec(0x1800, 4).unwrap(),
            7u32.to_le_bytes()
        );
        let metadata = original_lease.take_metadata().unwrap();
        original_lease.set_metadata(metadata);
        original_lease.finish().unwrap();
        let mut clone_lease = cloned.lease().unwrap();
        assert_eq!(
            clone_lease.mem_read_as_vec(0x1800, 4).unwrap(),
            44u32.to_le_bytes()
        );
        let metadata = clone_lease.take_metadata().unwrap();
        clone_lease.set_metadata(metadata);
        clone_lease.finish().unwrap();
    }

    #[test]
    fn native_context_survives_closed_owner_and_resumes_at_instruction_boundary() {
        let slot: ProcessNativeEngine<Key, Vec<(u64, u64)>> = ProcessNativeEngine::with_lifetime(
            Arch::MIPS,
            Mode::MIPS32 | Mode::LITTLE_ENDIAN,
            NativeEngineLifetime::FreshPerSlice,
        );
        let mut first = slot.lease().unwrap();
        mapped(&mut first);
        first.reg_write(RegisterMIPS::F0, 0x3f80_0000).unwrap();
        first.reg_write(RegisterMIPS::HI, 0x1122_3344).unwrap();
        first.reg_write(RegisterMIPS::LO, 0x5566_7788).unwrap();

        // Stop after the first two real MIPS instructions. The remaining ORI
        // and SW must run on the newly created owner, not the old handle.
        first.emu_start(0x1000, 0x1010, 100_000, 2).unwrap();
        let saved = key(&first, 7);
        assert_eq!(saved.pc, 0x1008);
        assert_eq!(saved.regs[8], 7);
        first.capture(saved.clone()).unwrap();
        first.finish().unwrap();

        // Ordinary finish closed the source owner while retaining the exact
        // CPU snapshot. No test-only owner disposal is allowed here.
        assert!(slot.identity().is_none());

        let mut second = slot.lease().unwrap();
        assert!(!second.reused());
        mapped(&mut second);
        assert!(second.restore_if_exact(&saved).unwrap());
        assert_eq!(second.reg_read(RegisterMIPS::F0).unwrap(), 0x3f80_0000);
        assert_eq!(second.reg_read(RegisterMIPS::HI).unwrap(), 0x1122_3344);
        assert_eq!(second.reg_read(RegisterMIPS::LO).unwrap(), 0x5566_7788);
        assert_eq!(second.reg_read(RegisterMIPS::PC).unwrap(), 0x1008);
        assert_eq!(second.reg_read(RegisterMIPS::T0).unwrap(), 7);

        // The count hook is a service bound, not proof that the last PC has
        // reached the instruction after SW. Resume to the known linear end
        // address instead; the native 100 ms limit still bounds this run.
        second.emu_start(0x1008, 0x1010, 100_000, 0).unwrap();
        // MIPS `until` may stop its translator without publishing the
        // terminal address back to the readable PC. The guest store, not
        // that host-facing PC register, proves both remaining instructions
        // executed after restoring the closed owner's CPU context.
        assert_eq!(
            second.mem_read_as_vec(0x1800, 4).unwrap(),
            7u32.to_le_bytes()
        );
        assert_eq!(
            second
                .query(unicorn_engine::unicorn_const::Query::TIMEOUT)
                .unwrap(),
            0
        );
        assert_eq!(second.reg_read(RegisterMIPS::F0).unwrap(), 0x3f80_0000);
        assert_eq!(second.reg_read(RegisterMIPS::HI).unwrap(), 0x1122_3344);
        assert_eq!(second.reg_read(RegisterMIPS::LO).unwrap(), 0x5566_7788);
        second.finish().unwrap();
        assert!(
            slot.identity().is_none(),
            "Stage 2 default finish must close the fresh native owner"
        );
    }

    #[test]
    fn fresh_owner_two_hundred_slices_reclaim_native_maps_and_pins() {
        struct PinnedPage {
            bytes: Box<[u8; 0x1000]>,
            drops: Rc<Cell<usize>>,
        }

        impl Drop for PinnedPage {
            fn drop(&mut self) {
                self.drops.set(self.drops.get() + 1);
            }
        }

        let slot: ProcessNativeEngine<Key, Vec<(u64, u64)>> = ProcessNativeEngine::with_lifetime(
            Arch::MIPS,
            Mode::MIPS32 | Mode::LITTLE_ENDIAN,
            NativeEngineLifetime::FreshPerSlice,
        );
        let drops = Rc::new(Cell::new(0));
        for cycle in 0..200usize {
            let mut lease = slot.lease().unwrap();
            assert!(!lease.reused(), "cycle {cycle} inherited a native owner");
            assert!(
                lease.take_metadata().is_none(),
                "cycle {cycle} inherited a closed owner's map ledger"
            );
            lease.mem_map(0x1000, 0x1000, Prot::ALL).unwrap();
            let mut page = PinnedPage {
                bytes: Box::new([0; 0x1000]),
                drops: Rc::clone(&drops),
            };
            let ptr = page.bytes.as_mut_ptr();
            lease.mapping_pins().pin(page);
            assert_eq!(lease.mapping_pins().len(), 1);
            // SAFETY: the pointer belongs to a fixed-size Box pinned by this
            // lease. It remains live through native unmap, if performed, and
            // otherwise through owner close.
            unsafe { lease.mem_map_ptr(0x3000, 0x1000, Prot::ALL, ptr.cast()) }.unwrap();
            let value = cycle as u32 + 1;
            let words = [
                0x2408_0000 | value, // addiu t0, zero, value
                0x3c09_0000,         // lui t1, 0
                0x3529_3000,         // ori t1, t1, 0x3000
                0xad28_0000,         // sw t0, 0(t1)
            ];
            let program: Vec<u8> = words.into_iter().flat_map(u32::to_le_bytes).collect();
            lease.mem_write(0x1000, &program).unwrap();
            lease.emu_start(0x1000, 0x1010, 100_000, 16).unwrap();
            assert_eq!(
                lease.mem_read_as_vec(0x3000, 4).unwrap(),
                value.to_le_bytes()
            );
            let metadata = if cycle % 2 == 0 {
                lease.mem_unmap(0x3000, 0x1000).unwrap();
                vec![(0x1000, 0x1000)]
            } else {
                // Exercise owner close with a still-live native pointer map.
                vec![(0x1000, 0x1000), (0x3000, 0x1000)]
            };
            assert_eq!(
                drops.get(),
                cycle,
                "native backing was dropped before fresh owner closure"
            );
            lease.set_metadata(metadata);
            lease.finish().unwrap();
            assert!(slot.identity().is_none(), "cycle {cycle} retained an owner");
            assert_eq!(drops.get(), cycle + 1, "cycle {cycle} retained a pin");
        }
        assert_eq!(slot.clone().lifetime, NativeEngineLifetime::FreshPerSlice);
    }

    #[test]
    #[ignore = "diagnostic: O-M1 Stage 2 native pointer backing"]
    fn native_mem_map_ptr_boxed_slab_survives_splits_and_fresh_owner() {
        const DATA_BASE: u64 = 0x3000;
        const SLAB_SIZE: usize = 0x4000;

        fn write_program(uc: &mut Unicorn<'_, ()>, address: u64, words: &[u32]) {
            let bytes: Vec<u8> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
            uc.mem_write(address, &bytes).unwrap();
        }

        // The host address is fixed for both native owners. The slab is never
        // resized and is dropped only after the final Unicorn owner closes.
        let mut slab = Box::new([0u8; SLAB_SIZE]);
        let slab_ptr = slab.as_mut_ptr();
        assert!(!slab_ptr.is_null());
        assert_eq!(slab.len(), SLAB_SIZE);
        assert_eq!(DATA_BASE & 0xfff, 0);
        assert_eq!(SLAB_SIZE & 0xfff, 0);

        let mut first: Unicorn<'static, ()> =
            Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();
        first.mem_map(0x1000, 0x1000, Prot::ALL).unwrap();
        // SAFETY: slab_ptr points to a writable, fixed-size Box of exactly
        // SLAB_SIZE bytes. The guest range and size are 4-KiB aligned. The
        // slab is retained until after both native owners have been dropped;
        // no host reference to it is used concurrently with emu_start.
        unsafe {
            first
                .mem_map_ptr(DATA_BASE, SLAB_SIZE as u64, Prot::ALL, slab_ptr.cast())
                .unwrap();
        }
        write_program(
            &mut first,
            0x1000,
            &[
                0x2408_3000, // addiu t0, zero, 0x3000
                0x3c09_1122, // lui t1, 0x1122
                0x3529_3344, // ori t1, t1, 0x3344
                0xad09_0000, // sw t1, 0(t0)
                0xad09_1000, // sw t1, 0x1000(t0)
                0xad09_2000, // sw t1, 0x2000(t0)
                0xad09_3000, // sw t1, 0x3000(t0)
            ],
        );
        first.emu_start(0x1000, 0x101c, 100_000, 7).unwrap();
        for offset in [0, 0x1000, 0x2000, 0x3000] {
            assert_eq!(
                &slab[offset..offset + 4],
                &0x1122_3344u32.to_le_bytes(),
                "MIPS store must reach the stable host slab"
            );
        }
        assert_eq!(
            slab.as_mut_ptr(),
            slab_ptr,
            "native mapping must not move the Box"
        );

        // Protecting a middle page splits the native mapping. It must retain
        // the original backing on both sides and enforce guest permissions.
        first
            .mem_protect(DATA_BASE + 0x1000, 0x1000, Prot::READ)
            .unwrap();
        let regions = first.mem_regions().unwrap();
        assert!(regions.iter().any(|region| {
            region.begin <= DATA_BASE + 0x1000
                && region.end >= DATA_BASE + 0x1fff
                && region.perms == Prot::READ.0 as u32
        }));
        assert_eq!(
            first.mem_read_as_vec(DATA_BASE + 0x1000, 4).unwrap(),
            0x1122_3344u32.to_le_bytes()
        );
        write_program(
            &mut first,
            0x1100,
            &[
                0x2408_6000, // addiu t0, zero, 0x6000
                0x2409_0077, // addiu t1, zero, 0x77
                0xad09_0000, // sw t1, 0(t0)
            ],
        );
        first.emu_start(0x1100, 0x110c, 100_000, 3).unwrap();
        assert_eq!(&slab[0x3000..0x3004], &0x77u32.to_le_bytes());

        write_program(
            &mut first,
            0x1200,
            &[
                0x2408_4000, // addiu t0, zero, 0x4000
                0x2409_0055, // addiu t1, zero, 0x55
                0xad09_0000, // sw t1, 0(t0), protected page
            ],
        );
        assert_eq!(
            first.emu_start(0x1200, 0x120c, 100_000, 3),
            Err(uc_error::WRITE_PROT)
        );
        assert_eq!(&slab[0x1000..0x1004], &0x1122_3344u32.to_le_bytes());

        // Unmapping another middle page must not free the slab or detach the
        // remaining suffix from it. The previously written bytes remain in
        // host backing even though that page is absent in this owner.
        first.mem_unmap(DATA_BASE + 0x2000, 0x1000).unwrap();
        assert_eq!(
            first.mem_read_as_vec(DATA_BASE + 0x2000, 4),
            Err(uc_error::READ_UNMAPPED)
        );
        assert_eq!(
            first.mem_read_as_vec(DATA_BASE + 0x3000, 4).unwrap(),
            0x77u32.to_le_bytes()
        );
        assert_eq!(slab.as_mut_ptr(), slab_ptr);
        drop(first);

        // Remap exactly the same stable backing only after source closure.
        // This is a fresh native owner, not a reused ProcessNativeEngine.
        let mut second: Unicorn<'static, ()> =
            Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();
        second.mem_map(0x1000, 0x1000, Prot::ALL).unwrap();
        // SAFETY: the original Box is still alive, unchanged in size and
        // writable; the first owner was explicitly closed before this map.
        unsafe {
            second
                .mem_map_ptr(DATA_BASE, SLAB_SIZE as u64, Prot::ALL, slab_ptr.cast())
                .unwrap();
        }
        write_program(
            &mut second,
            0x1000,
            &[
                0x2408_3000, // addiu t0, zero, 0x3000
                0x8d0a_0000, // lw t2, 0(t0)
            ],
        );
        second.emu_start(0x1000, 0x1008, 100_000, 2).unwrap();
        assert_eq!(second.reg_read(RegisterMIPS::T2).unwrap(), 0x1122_3344);
        assert_eq!(
            second.mem_read_as_vec(DATA_BASE + 0x2000, 4).unwrap(),
            0x1122_3344u32.to_le_bytes(),
            "the page unmapped only in the first owner is still in host backing"
        );
        assert_eq!(
            second.mem_read_as_vec(DATA_BASE + 0x3000, 4).unwrap(),
            0x77u32.to_le_bytes()
        );
        drop(second);
        assert_eq!(slab.as_mut_ptr(), slab_ptr);
        drop(slab);
    }
}
