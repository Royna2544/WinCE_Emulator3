# TODO

Refreshed on 2026-10-15. This file is limited to active, plan-aligned work.

## Immediate engineering queue

History and suite tallies live in `docs/evidence/ledger.md`; open defects are in
`KNOWN_BUGS.md`. Delete an item when it lands; do not annotate it.

0. Migrate writable image sections and DLLs as separate per-birth ownership
   slices, each with focused controls and memory accounting. Finish the direct
   native-write/restore/rebinding inventory,
   and decide/document `PrivatePageBudget` treatment before live acceptance.
   Matched iNavi timing requires a separate run approval and fresh adjacent
   mount backups.
1. Continue the user-authorized Stage 3 scheduler migration. The bounded
   child-exit sweep and owned-child `TerminateProcess` execution-stop handshake
   passed their separate frozen-source broad gates; continue the remaining
   semantics. Implement CE-compatible caller wait/timeout behavior, a safe
   old-MIPS direct-kernel real-handle continuation, and root-process
   real-handle completion. Then complete unpinned intra-slice worker lineage,
   aliasless Sleep-suspend, and generic parked-wait continuation. The bounded
   stale-TID/native-stamp [source gate](evidence/stage3-stale-tid-20261015/validation.receipt.md)
   birth-verifies pending thread returns, suspended thread contexts, native
   slice-end staging, and the GetMessage compatibility bridge. The later bounded
   [Sleep/object parked-wait gate](evidence/stage3-parked-wait-20261015/validation.receipt.md)
   verifies native register ownership before publication and wake consumption,
   then stamps the resumed birth. The separate bounded
   [parked MsgWait gate](evidence/stage3-msgwait-birth-20261015/validation.receipt.md)
   verifies captured birth before timer pumping/publication and the paired
   live owner before input or handle wake consumption. The bounded
   [parked queue/socket gate](evidence/stage3-queue-socket-birth-20261015/validation.receipt.md)
   covers native MessageQueueIo/WinsockRead birth ownership. The bounded
   [parked serial gate](evidence/stage3-serial-birth-20261015/validation.receipt.md)
   covers native SerialRead/SerialCommEvent birth ownership before host RX
   polling, wait publication and paired wake/timeout completion. The bounded
   [parked SendMessage gate](evidence/stage3-send-birth-20261015/validation.receipt.md)
   seals its tested Git HEAD and binary patch and covers cross-process native
   send publication and paired result consumption. The bounded
   [kernel sender-exit gate](evidence/stage3-send-exit-20261015/validation.receipt.md)
   cleans queued sends and detaches active sends after exact live-birth exit.
   The bounded [native exit gate](evidence/stage3-native-exit-birth-20261015/validation.receipt.md)
   checks captured thread birth and process lease before native `ExitThread`
   decisions and the guest return-stub no-exit-candidate fallback. Add an
   end-to-end return-stub hook control and make pending-return KData PID follow
   the captured lease, not mutable process selection. The bounded
   [native SMT gate](evidence/stage3-smt-birth-20261015/validation.receipt.md)
   checks eligible parked producer/re-entry/resume birth and exact wait/send
   pairing, with exact-ID GWE abandon. Close the occupied-slot raw fallback,
   global expiry-as-ready ambiguity, and receiver-side `DestroyWindow`
   completed-result writes before claiming full SMT semantics. Receiver-side
   delivery, broader GWE lifecycle, and other blocked payloads remain
   separate. Raw synchronous/nonblocking
   queue/socket paths and raw synchronous/nonblocking/overlapped serial paths,
   opaque non-task pins, generic parked-wait closure and caller parity remain
   separate. O-K3 execution and registered Thread/Process
   wait-target pins passed a bounded source gate
   ([receipt](evidence/stage3-pins-20261015/validation.receipt.md)); opaque
   non-task wait slots remain compatibility-only and unpinned. The subsequent
   prepared task-wait [source gate](evidence/stage3-waitprep-20261015/validation.receipt.md)
   separates fallible target pinning from native pending/old-wait mutation and
   scheduler publication; keep synchronous finite inline timeout paths and remaining native
   wait payload lifetimes in scope before declaring caller wait parity. The
   41 integration-test calls to the legacy `register_blocked_waiter` helper
   still need migration to prepared or opaque registration before that public
   compatibility entry can become test-support-only. The
   historical E-8/E-11/E-26 reds remain the acceptance baseline, not a reason
   to treat the design as parked. Full Stage 3 and live behavior remain
   unaccepted.

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
