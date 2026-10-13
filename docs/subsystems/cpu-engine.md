# CPU engine

Status: design-first subsystem contract; implementation and evidence are distinguished below.
Authority: [shared architecture](../ARCHITECTURE.md). Sequence: [migration plan](../PLAN.md).
This page cannot independently redefine the shared contracts or mark Phase 1 complete.

## Current implementation

- [unicorn.rs](../../src/emulator/unicorn.rs) combines CPU execution, hooks, imports,
  callback frames, blocked/suspended records, process parking, snapshots and handoff.
- [native_engine.rs](../../src/emulator/native_engine.rs) is the quarantined persistent
  engine experiment; [cpu_mips.rs](../../src/emulator/cpu_mips.rs) supplies MIPS helpers.
- [types.rs](../../src/emulator/types.rs), [stub.rs](../../src/emulator/stub.rs) and
  [main.rs](../../src/main.rs) expose snapshots, feature fallback and orchestration.
- The BED2 live baseline builds a fresh native engine per slice. Experimental
  build 650 retains a process-owned engine with a run-scoped `from_handle` hook
  lease and ledger; its normal live run failed at 129.108 s ([ledger E-2](../evidence/ledger.md))
  and it is deferred to Stage 8 ([D-003](../DECISIONS.md)). It is not a safe baseline.
- Phase 0 hot path (ledger E-9): the scheduler block hook polls only at amortized
  service points, hookless trampoline mode is the default, and `ce.slice_profile`
  logs guest instructions and MIPS per slice. Slice length is still bounded by the
  1M-instruction `emu_start` count; no scheduling policy changed.

## Target owner and interfaces

- `CeMachine` owns execution authority and the event sequencer. An `AddressSpace`
  owns mappings/backing; the CPU backend stores its bound engine and native CPU
  context payloads for machine-owned frames, not an independent Task registry.
- The shared `TaskIdentity` comes from canonical lifecycle registration. Current PID
  plus a passed TID/handle is not an identity constructor or creation-owner proof.
- The shared `FrameIdentity` identifies each parked/import/callback continuation.
  A wait ID or a latest snapshot per TID cannot replace its lifetime identity.
- Accept only a machine-selected Task/frame and an authorized AddressSpace lease.
  Return the anchored `RunOutcome`; preserve unknown instead of guessing a stop reason.
- Native timeout is a host service limit, expressed with `HostInstant`; it must not
  advance `GuestInstant` or masquerade as guest wait expiry/CE quantum expiry.
- Every publication carries `SnapshotEpoch` and its actual owner/frame. Native start,
  native end, post-repair context and later selected process are distinct boundaries.

## CPU state and lifetime rules

- Save checked native CPU Context at the actual transfer boundary, with raw native PC
  and authenticated frame provenance. PC/GPR/HI/LO alone omit FPU and branch metadata.
- A native parked instruction and a synthesized import return are distinct frames:
  restore raw continuation state, then apply only validated ABI return effects.
- Inside active emulation, native Context restore alone does not request TB restart;
  the checked PC-write/restart contract must accompany a genuine actor transfer.
- Preserve entry-only trampoline and ordinary delay guards until their replacements
  have genuine native proof. Never replay an interior's origin as guessed state.
- A new entry with no saved frame resets to initial blank CPU state before SP/RA/A*.
  Do not inherit a prior actor's FPU/HI/LO merely because the native handle is reused.
- CPU-only Context is not a RAM, device, IRQ, translation-cache or machine checkpoint.
  Compatible architecture/mode/configuration and AddressSpace ownership are required.
- Cloned checkpoints have independent native mutable owners and RAM. Only immutable
  CPU snapshot payloads may be shared; never shallow-clone the binding's native `Rc`.
- Run-local callbacks may borrow kernel/framebuffer/blob data only through a bounded
  lease. Track every hook; delete before freeing captures, parking reuse or return.
- Native deletion failure poisons/closes the owner. An incomplete mapping ledger or
  early failure cannot become reusable state through a destructor's implicit success.

For O-P4's proposed fresh-default policy, a successful slice may retain only
validated transferable CPU state after hook removal and owner close. The
engine-specific blank Context and native mapping ledger must be discarded; a
fresh engine starts with `reused == false` and newly derived mappings. The
current native-owner clone control restores a Context while its source owner
is still alive, so cross-engine restore **after source-owner closure** remains
an unexecuted Stage 2 gate. Preserve full FPU/HI/LO/branch state or fail the
attempt; GPR-only fallback does not establish continuation correctness. The
read-only [O-P4 source audit](../evidence/ledger.md#chat-project-validation-notes-not-in-this-repository)
lists the exact paths and bounded controls. Persistent reuse remains Stage 8
opt-in, never a prerequisite for Stages 2–6.

## Dependencies and prohibitions

- Depends on AddressSpace access, validated loader/import metadata, canonical Task/
  frame ownership and the machine-selected outcome/continuation—not window visibility.
- The API boundary supplies real guest effects; the CPU adapter does not duplicate
  scheduler queues, create new identities, infer priorities or force guest callbacks.
- Diagnostics may observe checked values. They cannot repair PC zero, hide native
  errors, consume a different frame, or relabel an executed actor to fit a snapshot.

## Cache/memory evidence and open risk

Facts are in the [ledger](../evidence/ledger.md): arena retention mechanism (E-3),
Stage 0 owner controls (E-4), pre-init arena budget and commit-tail work (E-5,
E-6) and full-backend controls (E-7). Design consequences:

- Hook removal invalidates translations without reclaiming arena commitment, so
  a persistent owner needs an explicit cache/lifetime design before acceptance
  (Stage 8). No cache or performance claim follows from handle reuse alone.
- Mapping/FlatView rebuilds, native RAM, retained blobs and TCG commitment need
  separate measurements. Hook removal also defeats naive translation-cache claims.
- The 650 crash site (TranslationBlock initialization) is distinct from the
  small-arena fault site (code emission); neither has an established cause.
- Native `ctl_get_timeout` returns the configured duration, not actual expiry.
  Two actual-backend controls executed as expected red in the historical E-7
  mirror baseline (a counted stop fabricated a host-wall record; a nonzero
  unmapped fault returned Ok under a positive timeout). The current D: source
  passes focused controls and the post-E-12 full release suite for both
  corrections, but final hardening, stop taxonomy and full Stage 2 gates
  remain open. Do not substitute a
  one-line query swap or change the scheduling tick.

## Stage 2 source map (historical fault and current handoff)

Both E-7 baseline defects originated in `UnicornMips::run_with_unicorn`, in the
post-`emu_start` block of [unicorn.rs](../../src/emulator/unicorn.rs). The
following three bullets describe the **pre-fix** source, not current D: code.
Search by symbol, not by line number; the file moves.

- `unicorn_native_timeout` was computed from `native_timeout_us != 0` and
  `ctl_get_timeout() != 0`, which is the *configured* limit. When no PC stop
  was requested and no hook had already recorded a host-wall stop, that flag
  alone created an `UnicornHostWallClockStop`, whatever actually ended the
  run. That is the fabricated host-wall record.
- `if let Err(err) = result && (native_requested_pc_stop.is_some() || !unicorn_native_timeout)`
  dropped every native `Err` whenever a positive timeout was configured. That
  is the swallowed `READ_UNMAPPED`.
- `Query::TIMEOUT`, the actual expiry, was collected only when
  `ce.slice_profile` debug was on. Current D: source collects it every attempt.

The remaining connected O-P1 work after the narrow E-15 correction is:

1. Obtain an independent native count-stop witness before reporting
   `CountLimit`. `Ok` with `count > 0` and no recorded hook reason remains
   `UnknownNormalStop`: an early native exit or unrecorded hook can also
   return normally.
2. Record each remaining hook reason at its actual request, then make
   post-run restart-PC repair consume typed provenance while preserving safe
   continuations after remote/ready-peer yields and actual budget stops.
   Both rotation consumers already read `NativeSliceOutcome`; keep their
   existing successful-boundary policy. Do not revive
   `unicorn_native_timeout` or change the current scheduling tick. Preserve
   verified encoded-kernel-exit decoding, but never let it mask a failed
   hook-control operation or primary native failure.
3. Repeat the focused controls after hardening, then execute the broader
   feature/full-backend and bounded live gates. Record newly exposed native
   errors in KNOWN_BUGS; do not suppress them again.

These steps would close the remaining O-P1 classification gap. They do not
change the 1M-instruction tick, the quantum or the clock.

Status (working D: tree; E-15 focused controls and final-source full release
suite, not alternate-profile or live-run acceptance):

- Done in source: `unicorn_native_timeout` is gone. `Query::TIMEOUT` is
  always collected, and a post-run *synthetic* host-wall record is created
  only after an actual native deadline. Hook-origin `UnicornHostWallClockStop`
  records can also mark remote-control or ready-peer safe yields, not just
  an elapsed host-wall budget. Every native `Err` now reaches the failure path,
  with encoded-exit decoding kept ahead of it. `NativeSliceOutcome` in
  [types.rs](../../src/emulator/types.rs) is set once per attempt. Both
  rotation sites read it: post-slice handoff in `unicorn.rs` and pre-run
  rotation in `main.rs`. `main.rs` no longer reads
  `host_wall_clock_stop`, so the scheduling decision has one source.
  Rotation still fires on every successful stop, as it did before, so no
  policy changed. In-repo controls cover the count boundary (no fabricated
  deadline) and the native-error case.
- E-15 focused O-P1 progress: the successful `ServiceBoundary` residual is
  now `UnknownNormalStop`; no `CountLimit` is reported without a witness.
  One checked scheduler block-hook path latches
  `HookStop(HostWallBudget)` only after elapsed budget, safe PC, successful
  PC write and successful `emu_stop`. Seven focused release controls passed
  on that initial source, including a real MIPS host-budget stop and actual
  native deadline query. A follow-up guard made both decoded guest-exit
  branches require no pending hook-control error; two host-budget focused
  controls passed on the follow-up source. Real PC-write/stop failure plus
  encoded-exit fault injection has not run. Receipts:
  `evidence/op1-stop-taxonomy-20261013/focused-controls.receipt.md` and
  `evidence/op1-stop-taxonomy-20261013/encoded-exit-mask.receipt.md` (ledger
  E-15). The final follow-up source also passed the complete release suite
  (1,628 passed, 0 failed, 4 ignored; raw log SHA-256
  `8946D073…1E041`); this is later than the E-13 1,626-pass checkpoint and
  does not turn the ignored E-14 VM control green. Keep
  `is_service_boundary()` true for successful Unknown, HookStop and
  NativeDeadline, preserving rotation policy. A debug
  `UnicornHostWallClockStop` alone does not prove a host-budget hook reason.
- Remaining hook-site inventory on the E-15 source:
  `NativeSliceOutcome` now has `UnknownNormalStop` and
  `HookStop(NativeHookStopReason)`, and `HostWallBudget` is the only reason
  so far. Only one of the seven hook sites that write
  `UnicornHostWallClockStop` sets a reason: the timeslice hook. The other
  six fall through to `UnknownNormalStop` while still writing the debug
  record. In this source they are:
  - Global code hook, deferred safe-PC completion (three sites). The reason
    is whatever set `host_wall_clock_stop_pending`, which is only a `bool`
    today.
  - Fast/trampoline block hook, the hookless default path:
    - After `drain_remote_server_control_messages_for_message_target`: a
      remote-control service stop.
    - When a parked peer `can_resume_from_live_pump`: a ready-peer yield.
    - When `elapsed >= limit || ready peer`: an ambiguous stop that needs
      both branches recorded separately.

  Next shape:
  1. Turn the pending `bool` into `Cell<Option<NativeHookStopReason>>`, set
     at the moment a stop is requested.
  2. Add `RemoteControlService` and `ReadyPeerYield`.
  3. Write the reason and the debug record through one helper, so the two
     signals cannot disagree.
  4. Key restart-PC repair off `HookStop(_)` and `NativeDeadline`.
  5. Add one actual-backend control per reason on the hookless path.
  6. Give the other `emu_stop` sites (about 115 non-test calls: import
     traps, probes, PC stops) reasons over time, and count `UnknownNormalStop`
     in `ce.slice_profile` so its rate is visible.
- Still open for O-P4: the in-repo red
  `native_context_survives_closed_owner_and_resumes_at_instruction_boundary`
  (`native_engine.rs`, `#[ignore]`) turns green only when the fresh lifetime
  is the default.
- **E-12 repaired-PC scalar fallback: focused fix, Stage 2 gate still open.**
  The pre-repair native context key used the PC/GPRs captured immediately after
  `emu_start`, while a post-run trampoline/import PC repair changed the scalar
  continuation's PC. `restore_if_exact` therefore missed, and the old blank
  context plus GPR/HI/LO fallback lost F0-F31 and FCSR; HI/LO survived. The
  persistent-owner replay exposed this, and the fresh-owner deterministic
  control reproduced it as an expected red after the repaired-PC and HI/LO
  assertions passed (ledger E-12/E-13).

  `MipsGuestContext` now carries `fprs: [u64; 32]` and `fcsr` as well as GPRs,
  HI and LO. Primary native and post-repair slice captures use the checked
  `capture_mips_context_checked` path; a mismatched native key restores the
  complete guest-visible scalar state via `restore_mips_context_checked` onto
  a blank engine. The primary native error retains precedence over capture
  errors. The native snapshot remains for exact raw continuations, where
  hidden branch/delay state matters. Writing a synthesized PC into a restored
  native snapshot is unsafe: vendored Unicorn's `UC_MIPS_REG_PC` write sets
  `active_tc.PC` and `M16`, but does not clear `hflags` or `btarget`.

  The final focused release control
  `repaired_pc_fresh_owner_preserves_full_mips_fpu_and_hilo` passed after its
  expected-red pre-repair run, including full-width F0, FCSR, HI/LO, repaired
  PC and guest `mfc1`/`cfc1` readback. Two checked-context controls passed on
  the same final executable: FCSR `0x1005` round-tripped and enabled pending
  cause `0x1080` returned `UC_ERR_EXCEPTION`. Exact source/executable hashes,
  commands and limits are in local
  `evidence/e12-repaired-pc-20261013.receipt.md` (ledger E-13). Subsequent
  serial validation passed the full release suite (1,626 passed, 0 failed,
  3 ignored), Unicorn-only and Unicorn+desktop/no-trace release checks, the
  no-feature suite (1,239 passed, 0 failed, 1 ignored), and fmt/diff checks;
  see the E-13 post-E-12 receipt. Neither focused nor suite success proves
  closed-owner exact-key native Context portability, `VirtualAlloc` byte
  coherence, callback/o32 float returns or app readiness.

  Follow-ups, from the source:
  - **Two helpers for one job.** The legacy `capture_mips_gprs` and
    `restore_mips_gprs` (about 57 and 62 non-test call sites) now also move
    FPU state. They still map read failures to zero and drop write errors
    (`let _ =`). Add no new legacy callers. Migrate each continuation family
    to the checked pair as it moves in Stage 4, then delete the legacy pair.
  - **FCSR restore can fail.** In vendored `qemu/target/mips/unicorn.c`,
    writing `UC_MIPS_REG_FCSR` keeps the old value and returns
    `UC_ERR_EXCEPTION` when `Cause & (Enable | 0x20)` is non-zero, that is,
    a pending enabled exception or an unimplemented-operation cause.
    - Checked path: this is a native error. Record the FCSR value; do not
      clear bits to make the restore pass.
    - Legacy path: the error is dropped and FCSR stays stale.
    - The checked-context `0x1005`/`0x1080` controls above establish those
      two focused outcomes, not all live FPU exception behavior.
  - **o32 float returns.** Guest functions return `float`/`double` in
    `$f0`/`$f2`. Any legacy restore site that runs after a host-initiated
    guest call and before the host reads its result would now clobber an FP
    return value. Audit those sites (callbacks, COM method thunks) before
    relying on full-FPU restore there.

## Telemetry, failure and migration gates

- Record native-owner identity/epoch, start/end Task/frame, checked native result,
  actual expiry observation, configured limit, guest instructions and busy MIPS.
- Keep unreadable/uncollected distinct from zero/false; bounded fault summaries are
  not whole-run history. A host AV may precede the final profile publication entirely.
- Account separately for native arena commitment, guest RAM/blob bytes and RSS/private
  memory. No cache or performance gain follows merely from same-handle reuse.
- Stage 2 (fresh-engine lifetime): checked full native context capture/restore
  after source-owner closure, fresh ledger reset and native byte round-trip,
  exact stop/error classification, both branch outcomes, native fault paths,
  all feature profiles and bounded live validation. These fresh-lifetime gates
  are open; the persistent-owner controls do not close them.
- Stage 4: migrate each complete enter/block/resume/return/exit family to real
  Task-owned contexts, with no wake/frame consumption before fallible transfer
  preflight. Genuine in-slice FPU wait→resume→exit remains staged and unexecuted.
- Stage 8 (persistent engines): clean owner/lease teardown; independent clone/
  parked-process same-VA isolation; exact/mismatched context and no-saved-entry
  reset; stale frame rejection; hook-stable rebinding; arena budget; the 650
  workload. The executed owner/hook/mapping controls (E-4) and native tail
  fixture (E-6) cover isolated engines only.
- Require count/yield/deadline/error/debug distinctions before rollout of any
  of the above.
