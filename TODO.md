# TODO

Refreshed on 2026-10-15. This file is limited to active, plan-aligned work.

ACK to Claude (supervisor): the A13 exact-modal-wait selector finding was
accepted and covered in the bounded
[A13 source/test gate](evidence/scheduler-source-cut-a13/validation.final.receipt.md).
The bridge still has an earlier unconfirmed message, so this records the ACK
without bypassing that delivery queue.

ACK to Claude (supervisor), 2026-10-15 19:57 KST steer: A13 seal verification
received. I am fixing the reused-PID host-teardown expected-red in the same
bounded gate, enabling its test before opening another red, and will name every
new validation attempt's raw log in the next receipt. The unused numeric
backing-store selector and legacy no-birth teardown branch will be reviewed
before sealing. The bridge's earlier delivery remains unconfirmed, so this ACK
is recorded here without bypassing its queue.

## Immediate engineering queue

History and suite tallies live in `docs/evidence/ledger.md`; open defects are in
`KNOWN_BUGS.md`. Delete an item when it lands; do not annotate it.

0. Migrate writable image sections and DLLs as separate per-birth ownership
   slices, each with focused controls and memory accounting. Finish the direct
   native-write/restore/rebinding inventory,
   and decide/document `PrivatePageBudget` treatment before live acceptance.
   Matched iNavi timing requires a separate run approval and fresh adjacent
   mount backups.
1. Continue the user-authorized Stage 3 migration. Passed bounded gates are
   listed in `PROGRESS.md` ("2026-10-15 Stage 3 source gates"); open defects
   are in `KNOWN_BUGS.md`. Next, in order:
   - Owner-filtered host teardown, full exit capture retirement and implicit
     gesture ownership need separate gates; so do broadcast, HWND-owned
     timers, hotkeys, sent/paint/modal and other numeric GWE producers.
   - Cross-process parked-sender restore, reciprocal shapes beyond A12's exact
     two-send case, legacy numeric-only HWNDs, post-wake MsgWait/sent
     processing and `SMF_TIMEOUT` last-error semantics remain separate gates,
     not reproduced live-app failures.
   - Native continuation payload ownership, task-owned concurrent SMT frames,
     the SMT global expiry-as-ready ambiguity and receiver-side
     `DestroyWindow` result writes.
   - CE caller wait/timeout behavior, a safe old-MIPS direct-kernel
     real-handle continuation and root-process real-handle completion.
   - Unpinned intra-slice worker lineage, aliasless Sleep-suspend and generic
     parked-wait continuation.
   - An end-to-end return-stub hook control; pending-return KData PID from the
     captured lease, not mutable process selection.
   - Raw synchronous/nonblocking queue/socket/MsgWait and serial (including
     overlapped) paths, opaque non-task pins, synchronous finite inline
     timeouts and native wait payload lifetimes, all before claiming caller
     wait parity.
   - Wait-ID wrap/reuse and live modal behavior.
   - Migrate the remaining fixture calls to the legacy `register_blocked_waiter`
     helper, then make that entry test-support only.
   The historical E-8/E-11/E-26 reds stay the acceptance baseline. Full Stage 3
   and live behavior remain unaccepted.

## Parallel lanes (never gate the queue above)

- E-43's sealed main-image, E-44's fixed initial heap/stack, and O-M2's
  birth-qualified heap **source** gates passed. O-M2 includes an exact stale-
  birth serial/NMEA control; full host-write inventory, memory-mode checkpoint
  relineage, pin-budget treatment and live app behavior remain open
  ([E-43 receipt](evidence/sealed-main-image-20261014/validation.receipt.md),
  [E-44 receipt](evidence/heap-stack-20261014/validation.receipt.md),
  [O-M2 receipt](evidence/om2-heap-20261014/validation.receipt.md)).
- If a later approved live run reaches a direct-import cold-stack bypass,
  consider per-thread/slot 128-KiB committed low-water tracked by slot, with
  contiguous `[page(SP), slot_top)` commit. The current shared sparse
  `ThreadStack` has no per-slot low-water state; do not add this speculatively.
- Splash stall (app evidence lane): run the E-29 post-120 s trace as one
  instrumentation change plus one run using `NativeAttemptReport`
  (`docs/subsystems/native-attempt-observability.md`). Record the
  any/all mode of the TID 10 wait; wait id 833 needs a boot-forward trace. Each
  writable-mount live run needs fresh user approval and a verified backup
  (`scripts/backup_rw_mounts.ps1`).
- Live run: none approved. a02 and a04 are historical and not live-usable;
  the user chose the scheduler cut over a04
  (`call_49d1b609465a4e668da30a31361beb44`). Approval
  `call_9b35e9d31fdf402ea082a3eb6a40ae16` was withdrawn after an SD-only
  backup failure, and that ZIP must not be reused
  ([disposition](evidence/pre-live-mount-backup-20261015T025500Z/aborted-approval-disposition.md)).
  Keep the incomplete archive until the user decides on disposal. History:
  [a04 seal](evidence/stage3-live-evidence-20261015/prelaunch-repair-a04-source-seal.json),
  [offline gate](evidence/stage3-live-evidence-20261015/prelaunch-repair-a04-offline-gate.raw.txt)
  and PROGRESS.md. A future run needs, in order:
  - a fresh, distinct approval whose chat citation is checked;
  - a new SHA-verified adjacent backup triplet, with the writable folders
    quiescent and the inventory validated;
  - a freshly source/build-sealed EXE/PDB (llvm libclang) and the
    `192.168.0.39:8765` endpoint.

  The a04 watchdog (independent worker, 355 s deadline) is the base for that
  rebuild. Report input ACK, drain/post, guest WNDPROC and frame separately,
  plus phase-aligned busy-slice MIPS, per-slice setup cost, host RSS/private
  memory and the narrow guest page budget. Source, backup and terminal
  evidence and any monitor failure are separate acceptance gates.
- Host idle polling must not decide from debug snapshots. Derive it from
  canonical wait state (`docs/evidence/stale-snapshot-host-routing-design-20261014.md`).
- Before claiming exact input-to-visual lag, implement the proposed
  lineage-checked ingress→guest-work→frame-commit gate in
  `docs/subsystems/remote-observability.md`; until then report poll brackets,
  not a causal response time.
- Child `WM_ACTIVATE` stale-return stub (A-6): the controls pass (ledger E-27).
  Any recovery change needs identity-matched live evidence.
- Stage 8 native lane is paused. The E-33 commit-tail clamp landed as the D-009
  exception; do not expand it.

## Older investigation items

- Follow the second startup initializer at `0x31970` (return `0x2be44`) with the
  exact-stop debugger after authentic callback restoration.
- Revalidate the native GetMessage/sent/SetWindowPos boundary.
- Bind the live MDC object and stack chain through method `0x12e15c`, engine
  return `0x12e180` and pool return `0xe6da4`.
- Before any heap address reuse, prove the original reservation footprint,
  checked allocation incarnation and stale-pointer rejection. O-M2 retains
  birth-local monotonic holes rather than reclaiming them.
- Profile boot, frame and input latency plus host memory, phase-aligned. Reduce
  tap-delivery lag using queue and slice evidence.
- Continue the live launch loop past the `resmapi_800x480.bin` map-skin decode
  until the map screen presents.
- Throughput, after Stages 5-6: a time-based quantum replacing the 1M
  instruction count, then virtual time with idle fast-forward. Always report
  per-slice guest MIPS (`ce.slice_profile`).
- Launch wrappers must capture the process exit code and both streams (the
  2026-07-04 13:22:22Z silent desktop-run exit is still unexplained).
- Watch the SD image for re-created zero-byte probe artifacts on long runs.
- Re-measure the map-skin decode: hot PCs `0x00339c00-0x00339da8` are pixel
  conversion, not a readiness wait.
- Broaden coverage where CE source or raw tests justify it: SendMessageTimeout
  and reentrant message waits, Shell_NotifyIcon/SHNotification integration,
  lower mounted-storage layers, and GDI/text/IME parity.

## Next

- Add raw/non-Unicorn loader fixture variants for loader paths where Unicorn
  runtime coverage is currently stronger.
- Expand PE icon/resource tests for uncommon formats, malformed resource tables,
  callback-selected `KernExtractIcons`, non-PE fallback edges, and cleanup
  lifetimes.
- Continue file-change notification reset/error propagation and mounted rename
  edge coverage where current behavior is still only partially proven.
- Prove device-notification consumer ordering and wake cadence in live startup
  scenarios.
- Replace the synthetic `GetCallStackSnapshot` frame with real saved-context
  unwinding when guest diagnostics require deeper stack fidelity.
- Decode the SMB380/G-sensor initialization contract from actual dump code or
  real caller traces before adding accelerometer command behavior.
- Continue live device traces after `COM7:`, `MFS1:`, and `SMB1:` are open.
  GPS now drains and sensor IOCTLs are visible; keep validating parsed-position
  consumption and the remaining presentation gate with real traces.
- Re-run the live startup after the device-information ordinal fix and confirm
  whether guest device probing now reaches `COM7:` earlier or still remains
  blocked behind the splash/readiness path.
- Re-run the live startup after the device-search ordinal fix and confirm
  whether CE Device Manager enumeration advances any GPS/storage probing paths.

## Cleanup Queue

- Clear the 21 release-build warnings in one change, between gates. The
  inventory is in `evidence/stage3-thread-timer-birth-20261015/build-a01.raw.log`.
  Dead code: `ArchHooks`/`ArmArch`/`ArmGuestContext`/`MipsArch` (`arch.rs`,
  `cpu_arm.rs`, `cpu_mips.rs`), `map_kernel_memory_allocations`,
  `fast_start_indirect_transfer_sites`, `ensure_mapped_ram_blob`,
  `reconcile_persistent_native_memory{,_with_private}`, `DPAS_SORTED`
  (`unicorn.rs`), `NativeMappingPins::{pin, release_image_after_unmap,
  private_len}`, `range_contains` and an unread `bytes` field (`pe_loader.rs`),
  `measure_text_units_width` and `PopupMenuModalState::cursor` (`coredll.rs`).
  Unused or misused bindings: the `MEM_DECOMMIT` import, `blobs`,
  `pending_wndproc_returns_live_pump_hook`, `current_thread_id_code_hook`, the
  dead first `fixed_ranges` assignment and one needless `mut`. Delete dead
  items unless a planned production caller is named; do not hide them with
  `allow(dead_code)`.
- Keep the five-file small-intermediate recovery archive/receipt. Larger cache
  cleanup requires fresh ownership/idle checks; D debug/build showed other
  editor/compiler activity. The five inventoried legacy paths were retired
  after separate approval, with selected source, patch and EXE/PDB evidence
  preserved and verified again under D: `evidence/` (post-order retirement
  receipt). Do not infer that other mirrors, hardlinked installed outputs,
  unique binaries/symbols/run evidence, synced references or SD files are
  disposable from that scoped approval.
- Keep `PROGRESS.md`, `TODO.md`, and `KNOWN_BUGS.md` short and current.
- Do not reintroduce historical ledgers into roadmap files.
- Keep `SOURCE_REFERENCES.md` as a compact index of active source evidence, not
  a running implementation diary.
- Keep registry fixtures and launch docs on `registry.reg`; temporary JSON
  parser regressions should use temporary fixtures only.

## Validation Queue

- Stub/raw smoke: `cargo test -j 1`
- Shell/resource/GDI: `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel`
- GWE/message/menu/modal: `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
- File/storage: `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`
- Shared scheduler/runtime: `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems`
- Larger handoff: `cargo check --features unicorn,trace,win32-desktop` and
  `git diff --check`
