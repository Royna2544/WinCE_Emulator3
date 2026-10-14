# O-P4 fresh-default native-owner gate (2026-10-14; executed)

The frozen-source O-P4 gate passed; it is not full Stage 2 acceptance or
permission for a live iNavi run. The exact commands, hashes and limits are in
the [validation receipt](../../evidence/op4-fresh-default-20261014/validation.receipt.md).
It follows [D-003](../DECISIONS.md) and the common
[CPU/VM anchor](../ARCHITECTURE.md): VM owns guest bytes; a native engine only
maps them for one checked run lease. Fresh-per-slice is now the default after
the validated public private-page gate E-35. Persistent
process-owned engines remain an explicit Stage 8 experiment, not a fallback
on fresh failure. The same policy must survive CPU clone and park/unpark.
`WINCE_EMU_PERSISTENT_NATIVE_ENGINE=1` is the explicit experimental runtime
opt-in, sampled once for the host process; absent or other values use fresh.

## Exact source proof and its limit

The proof is specific to the locked `unicorn-engine` / vendored
`unicorn-engine-sys` **2.1.5** source, not an upstream API promise.
`vendor/unicorn-engine-sys/build.rs` must unconditionally compile this
vendored C through CMake for a static link. Its former `pkg-config` branch
could link an unrelated installed Unicorn >= 2, and `dynamic_linkage` could
load an unaudited same-named DLL at runtime, voiding the close proof even
with exact crate versions. That system-library branch is removed and the
optional dynamic feature is rejected at build time. The registry Rust wrapper
  remains pinned by the root manifest and lockfile. The compiled vendor build
  script was directly invoked with the dynamic-link feature set and rejected it
  as intended; this was not a separate full Cargo dynamic-feature build. Source checks:

- `vendor/unicorn-engine-sys/uc.c:497-574` has two `uc_close` exits. An
  uninitialized handle is freed and returns `UC_ERR_OK`; an initialized handle
  tears down translations, CPU, mappings, hooks and the engine, then returns
  `UC_ERR_OK`. It has no recoverable error return on this source. A native
  crash during teardown would terminate the attempt, not publish success.
- `uc.c:1914-1934` runs `UC_INIT` before successful hook registration, and
  `uc.c:304` establishes `init_done`. `uc_hook_del` (`uc.c:2042-2066`) can
  report failure only through `UC_INIT`; successful run-hook registration
  rules that out while this handle remains valid. It marks hooks `to_delete`.
  Dispatch skips deleted hooks; `uc_close` clears deferred and remaining hook
  lists (`uc.c:560-564`). A failed deferred-list append cannot make a deleted
  callback run, and final close still frees the hook list.
- The high-level wrapper's `UnicornInner::Drop` invokes `uc_close` but ignores
  its return (`unicorn-engine-2.1.5/src/lib.rs:151-156`). This gate relies on
  the vendored C proof above; it does **not** claim a general checked-close
  API. `Unicorn::from_handle` is nonowning (`lib.rs:180-234`), while the
  owning `Unicorn::new` must not be cloned or closed through `get_handle()`.
  Doing the latter would double-close when the owner drops.
- The tracked lease calls `finish`/`abort_close` outside native execution
  (`src/emulator/native_engine.rs:546-619`; its callers are in
  `src/emulator/unicorn.rs`). `uc_emu_start` completes the outer run, clears
  deferred hooks and joins its timer before returning (`uc.c:1241-1252`).
  The wrapper's `remove_hook` drops the Rust callback before calling C
  (`lib.rs:1171-1176`); fresh closure must therefore **not** remove individual
  hooks first. Let native close happen while the nonowning wrapper still holds
  every callback box.

Before any dependency, vendor-source, or native-linkage change, re-audit these
exact statements and the wrapper's ownership/drop behavior. If `uc_close`
gains an error path or the proof fails, stop this gate: a consuming
checked-close API and owned callback-referent quarantine would need a
separate design. The broader
[guest-memory O-P4 plan](../subsystems/guest-memory.md) describes that general
case; the current implementation uses this narrower source-bound exception.

## Owner, hook, mapping and snapshot transitions

`ProcessNativeEngine` now creates an owning hook-free `Unicorn` for each fresh
lease; a separate nonowning `from_handle` wrapper registers run-local hooks and
carries the mapping ledger and `NativeMappingPins`
(`src/emulator/native_engine.rs:163-390`). Fresh `finish`, abort and unfinished
lease drop close the owner before dropping callback boxes or pins. Only a
CPU-only exact-key `Context` may transfer; the engine-specific blank context,
native map ledger and pins do not. The old reusable-owner branch remains only
behind the explicit persistent experimental opt-in.

Implemented fresh-default transition:

1. `lease()` creates a new owning engine on every attempt. Reuse is false;
   derive native maps from the selected process's current VM owner and live
   allocation identities. A closed owner's `virtual_pages`, private-page
   records, and blank context never enter the next lease.
2. Before any successful finish, finish guest execution, capture the
   authentic thread/scalar-key CPU context, complete outgoing view/blob
   synchronization, check the mapping ledger, and resolve native/backend
   errors. Do not publish a saved continuation or scheduler handoff yet.
3. For both fresh `finish` and setup/run abort, do not call individual
   `remove_hook`. Drop the unique **owning** wrapper first while the
   nonowning run wrapper, callback boxes, mapping pins and page referents
   remain alive. Under the pinned C proof, this closes the native engine.
   Then drop the nonowning wrapper/callbacks; only then release its pins and
   native page guards. Successful finish retains only the exact-key CPU
   snapshot. Abort retains no snapshot and reports a backend failure.
4. Clear the slot's owner, blank context, native map metadata and pins before
   any next lease. An incomplete ledger, failed native map/protect/unmap,
   mismatched identity, failed sync, or failed CPU capture cannot become a
   successful continuation. A page whose `mem_unmap` fails remains pinned
   and charged until the enclosing native owner has actually closed.

This ordering is explicit in `NativeEngineLease::close` and `Drop`, not
inferred from Rust field-drop order. No raw handle or owning-wrapper clone may
escape the lease. Keep the existing persistent-owner path behind an explicit
opt-in for Stage 8; it may continue to require hook deletion before reuse,
but its experimental safety is not established by this fresh-only gate.

## Executed controls and evidence

After E-35's final source and tests were frozen, this policy switch and its
controls were implemented in the sole D: checkout. The gate criteria were:

1. Turn the ignored E-7
   `native_context_survives_closed_owner_and_resumes_at_instruction_boundary`
   control green without its manual owner-drop block or
   `close_idle_owner_preserving_snapshot_for_test`. Verify source owner is
   absent after ordinary `finish`, a new lease reports `reused == false`, and
   FPU, HI, LO, PC, scalar key and the resumed MIPS store survive the closed
   owner. A native pointer address alone is not an identity witness because
   the allocator may reuse it.
2. Run 200 bounded fresh native slices with a pointer-backed page allocated
   for each slice, a real guest store, checked unmap and owner closure. Require
   stable guest bytes, zero inherited native ledger, no stale callbacks and
   no growing live pin count after each close. Preserve E-35's separate public
   `VirtualAlloc` controls, 64-cycle checked-unmap/budget churn, and failed-
   unmap retention control; no injected map/unmap/sync failure may publish a
   successful continuation.
3. Keep E-18's public fresh-owner guest-write control and E-20 owner isolation
   green. Exercise setup abort and normal post-run completion, clone/park
   policy preservation, and every terminal `finish` call path (normal,
   debugger stop and decoded exit). If a close-status failure seam is added,
   do not present it as a real vendored-C failure without a changed C path.
4. Freeze source hashes, then run focused controls and the full release
   `unicorn,trace,win32-desktop`, no-feature, and no-trace
   `unicorn,win32-desktop` profiles serially. Save raw logs, exact commands,
   exit codes and source/executable hashes outside `target/`; run formatting
   and diff checks. Only frozen-source green evidence can close O-P4.

The formerly ignored closed-owner control, 200-slice pin/map control, focused
failure and private-byte controls passed. On frozen source, full release
passed **1,730/0/14 ignored**, no-feature **1,292/0/13 ignored**; both no-trace
checks and the release build exited 0. The receipt retains raw logs, exact
source hashes and matching preserved EXE/PDB; [ledger E-36](ledger.md) records
the result. A bounded live iNavi run with the original RW mounts requires
**fresh human approval and fresh adjacent ZIP backups** of SD,
ResidentFlash and emulator-root; earlier single-run approvals were consumed.
That run must compare sampled host memory, busy-slice MIPS, fixed per-slice
setup cost and guest/map progress with BED2 and persistent-owner runs. Do not
infer memory, speed, map-readiness, Stage 3 or full Stage 2 acceptance from
these source tests.
