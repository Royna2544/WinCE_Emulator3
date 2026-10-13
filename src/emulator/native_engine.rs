//! Process-owned native engine boundary (module declaration must be Unicorn-gated).
//!
//! The stored engine is hook-free. A lease creates a lifetime-scoped, non-owning
//! wrapper over its handle. Every run-scoped hook must use the lease's tracked
//! registration methods; do not escape a raw/cloned wrapper or register through
//! `DerefMut`. Normal reuse requires checked `finish` and a complete mapping ledger.
//! Dropping a lease on an early return discards its native engine and metadata.
//!
//! K is the authentic (thread ID, saved scalar CPU context) resume key. P is the
//! engine's mapping ledger; cloning never copies a live engine or that ledger.

use std::{
    cell::RefCell,
    fmt,
    ops::{Deref, DerefMut},
    rc::Rc,
};
use unicorn_engine::{
    Context, UcHookId, Unicorn,
    unicorn_const::{Arch, ContextMode, HookType, MemType, Mode, uc_error},
};

struct NativeSnapshot<K> {
    key: K,
    cpu: Context,
}

struct NativeSlot<K, P> {
    engine: Option<Unicorn<'static, ()>>,
    blank: Option<Rc<Context>>,
    saved: Option<Rc<NativeSnapshot<K>>>,
    metadata: Option<P>,
    leased: bool,
}

pub(crate) struct ProcessNativeEngine<K, P = ()> {
    arch: Arch,
    mode: Mode,
    slot: Rc<RefCell<NativeSlot<K, P>>>,
}

impl<K, P> fmt::Debug for ProcessNativeEngine<K, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slot = self.slot.borrow();
        f.debug_struct("ProcessNativeEngine")
            .field("arch", &self.arch)
            .field("mode", &self.mode)
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
            slot: Rc::new(RefCell::new(NativeSlot {
                engine: None,
                blank: None,
                saved,
                metadata: None,
                leased: false,
            })),
        }
    }
}

impl<K, P> ProcessNativeEngine<K, P> {
    pub(crate) fn new(arch: Arch, mode: Mode) -> Self {
        Self {
            arch,
            mode,
            slot: Rc::new(RefCell::new(NativeSlot {
                engine: None,
                blank: None,
                saved: None,
                metadata: None,
                leased: false,
            })),
        }
    }

    /// Owns the slot rather than borrowing a field of the enclosing CPU.
    pub(crate) fn lease<'a>(&self) -> Result<NativeEngineLease<'a, K, P>, uc_error> {
        let mut slot = self.slot.borrow_mut();
        if slot.leased {
            return Err(uc_error::RESOURCE);
        }
        let reused = slot.engine.is_some();
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
                return Err(err);
            }
        };
        slot.leased = true;
        let metadata = slot.metadata.take();
        let saved = slot.saved.clone();
        Ok(NativeEngineLease {
            uc: Some(uc),
            owner: Some(owner),
            blank,
            saved,
            metadata,
            slot: Rc::clone(&self.slot),
            hooks: Vec::new(),
            reused,
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
}

pub(crate) struct NativeEngineLease<'a, K, P> {
    uc: Option<Unicorn<'a, ()>>,
    owner: Option<Unicorn<'static, ()>>,
    blank: Rc<Context>,
    saved: Option<Rc<NativeSnapshot<K>>>,
    metadata: Option<P>,
    slot: Rc<RefCell<NativeSlot<K, P>>>,
    hooks: Vec<UcHookId>,
    reused: bool,
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

    #[cfg(test)]
    pub(crate) fn identity(&self) -> usize {
        self.get_handle() as usize
    }

    pub(crate) fn take_metadata(&mut self) -> Option<P> {
        self.metadata.take()
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

    /// Commit reuse only with a complete ledger and successful hook cleanup.
    pub(crate) fn finish(mut self) -> Result<(), uc_error> {
        self.close(true)
    }

    fn close(&mut self, commit: bool) -> Result<(), uc_error> {
        if self.closed {
            return Ok(());
        }
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
        if commit && failure.is_none() {
            // Native hooks are deleted before freeing the borrowed wrapper.
            // Only then may the owning handle and matching ledger be reused.
            self.uc.take();
            let mut slot = self.slot.borrow_mut();
            slot.engine = self.owner.take();
            slot.blank = Some(Rc::clone(&self.blank));
            slot.saved = self.saved.take();
            slot.metadata = self.metadata.take();
            slot.leased = false;
        } else {
            // remove_hook drops its Box before its FFI delete. On any failure
            // there must be no future native entry: close the native handle
            // before dropping the remaining borrowed callback Boxes.
            self.owner.take();
            self.uc.take();
            let mut slot = self.slot.borrow_mut();
            slot.engine = None;
            slot.blank = None;
            slot.saved = None;
            slot.metadata = None;
            slot.leased = false;
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
        ProcessNativeEngine::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN)
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
    #[ignore = "expected red: ledger E-7, Stage 2 O-P4 fresh owner closure"]
    fn native_context_survives_closed_owner_and_resumes_at_instruction_boundary() {
        let slot = slot();
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

        // Until fresh-per-slice is the default, force only this test's idle
        // owner closed. Keep its immutable CPU snapshot to isolate whether
        // Unicorn permits Context restore after the source owner is gone.
        {
            let mut state = slot.slot.borrow_mut();
            state.blank = None;
            state.metadata = None;
            drop(state.engine.take());
            assert!(state.saved.is_some());
        }
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
