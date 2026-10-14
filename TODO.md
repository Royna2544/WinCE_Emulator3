# TODO

Refreshed on 2026-10-14. This file is limited to active, plan-aligned work.

## Immediate Engineering Queue (Stage 2, in order)

History and suite tallies live in `docs/evidence/ledger.md`; open defects are in
`KNOWN_BUGS.md`. Delete an item when it lands; do not annotate it.

0. Fix the fresh-default file-view readback regression (KNOWN_BUGS, top
   entry). Read back only the parts of each view that are natively mapped in
   this owner, using `mem_regions` intersected per page. A view range that is
   not mapped received no native writes in this lease, so the kernel copy stays
   authoritative. Hard-fail only on real read errors on mapped ranges.
   Same change: a real-native public `MapViewOfFile` test on the fresh default
   that is red first, then green. It covers three slices: one writes page 0, the
   next touches nothing, the third writes page 1. Both pages must keep the right
   bytes.
1. Only after fresh human approval and fresh adjacent ZIP backups of the
   original SD, ResidentFlash and emulator-root writable mounts, re-run the
   bounded iNavi comparison on a build with item 0 fixed. The O-P4 EXE run
   failed at 2.9 s (`evidence/e22-live-20261014T041105Z`). Compare sampled
   host memory, busy-slice guest MIPS, fixed per-slice setup cost and guest/map
   progress with the persistent-owner runs (about 1.5 GiB; E-22, E-27, E-29)
   and BED2 (E-1); retain exact binary, input and run evidence. The
   source-bound O-P4 suite alone is not live acceptance.
2. Then finish the remaining D-008 families: real native E-20 two-PID same-VA
   A→B→A byte isolation, shared file views on slabs, and image/heap blobs.
3. Stage 3 entry: the 12 expected-red Task controls (ledger E-26). The E-26
   scheduler design draft stays parked until this step.

## Parallel lanes (never gate the queue above)

- Splash stall (app evidence lane): run the E-29 post-120 s trace as one
  instrumentation change plus one run using `NativeAttemptReport`
  (`docs/evidence/e29-post120-native-attempt-trace-design.md`). Record the
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
- Before any heap reuse, establish the allocation owner, original reservation
  footprint and incarnation. Investigate the shared monotonic heap high-water
  mark mirrored into each process's spillover backing.
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

- Make `GlobalMemoryStatus` report from the configured guest physical RAM
  setting rather than its current 64 MiB total / 48 MiB available constants;
  give that change its own public API test after the E-35 private-slab gate.
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
