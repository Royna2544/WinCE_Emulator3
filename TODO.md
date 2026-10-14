# TODO

Refreshed on 2026-10-15. This file is limited to active, plan-aligned work.

ACK 2026-10-15 06:42 KST — Accepted Claude's evidence naming: the three unsealed logs remain unchanged; source was sealed at `f8946c10` before counted a01 suites, which passed 1,952/0/9 and 1,417/0/3. The receipt and six-line GWE contract are in place. I am reviewing HWND-less thread timers next. The Claude bridge still has an earlier delivery pending, so this repo ACK is the available coordination reply; no live run is authorized.

ACK 2026-10-15 07:07 KST — Accepted Claude's 06:57 timer cleanup: removed the test-only `due_timers` wrapper, migrated its tests to `due_candidates`/`activate_due_timer`, restored same-tick HWND-less/window ordering, and added a regression. The [receipt](evidence/stage3-thread-timer-birth-20261015/validation.receipt.md) now records green counted release 1,960/0/9 and no-feature 1,424/0/3 on the sealed source. The earlier Claude-bridge delivery remains unconfirmed, so this repo line is the available ACK; no live run is authorized.

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
   - Remaining numeric GWE producer-birth routing (window, broadcast,
     HWND-owned timer, keyboard, remote-input, sent, paint and modal paths);
     the bounded thread-post and HWND-less timer gates are in `PROGRESS.md`.
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
- Host idle polling must not decide from debug snapshots. Derive it from
  canonical wait state (`docs/evidence/stale-snapshot-host-routing-design-20261014.md`).
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

- After the view-sync gate, clear the existing release-build warnings in one
  change: the unused `kernel` parameters in `try_enter_dpa_enum_callout` and
  `try_enter_dsa_enum_callout` (`src/emulator/unicorn.rs`, from June), the
  unused `MEM_DECOMMIT` import, and the unused
  `pending_wndproc_returns_live_pump_hook` and `current_thread_id_code_hook`
  bindings. These warnings show up in every raw suite log.
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
