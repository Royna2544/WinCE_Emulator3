# FakeCE / iNavi SE G3 Emulator

Rust bootstrap for a Unicorn-backed Windows CE-style emulator targeting the
iNavi SE G3 MIPS R4000 application.

The project favors reusable CE behavior over app-specific shortcuts. Current
coverage includes PE loading, import patching, core kernel handles, memory,
registry, host-backed mounted files, GWE windows/messages, GDI/DIB rendering,
shell/resource APIs, Winsock, serial GPS input, waveOut/audio plumbing, and a
remote-control HTTP/WebSocket surface.

## Current State

- The [architecture anchor](docs/ARCHITECTURE.md), [migration plan](docs/PLAN.md)
  and [subsystem pages](docs/README.md) guide the user-authorized scheduler-core
  rewrite; they do not mark it complete. The sole source checkout is the D:
  repository. Fresh-engine-per-slice is the source-bound default, and public
  private-VM native same-VA A→B→A isolation is validated. D-008 public
  shared-view named A→B→A and same-mapping alias controls pass focused tests
  and both frozen-source broad suites. One sealed task/frame-qualified run
  outcome, remaining writable-image/DLL backing, Stage 2 and
  Stage 3 task migration remain open. After E-42's historical selector-only
  gate, the main EXE loader now seals final patched R/RX bytes and retains
  mutable runs copy-backed in the normal app path. Its E-43 source-bound gate
  passes. E-44 separately source-gated the initial heap/stack fixed slabs;
  O-M2 then made the fixed process heap birth-qualified and removed copied
  spillover. An opt-in synthetic writable-main-image VM/pin path, monitor
  checkpoint refusal and live-native mutable-code reads pass bounded source
  controls. A synthetic second-run native-map failure/checked abort and a
  legacy file-view overlap fence pass. The next synthetic VM-allocation
  collision fence also passes; its complete source suites passed release
  **2,127/0/9 ignored** and no-default **1,520/0/3 ignored**
  ([receipt](evidence/stage2-writable-image-vm-allocation-20261016/validation.receipt.md)).
  The [synthetic exact-process-exit/retained-pin gate](evidence/stage2-writable-image-exit-retire-20261016/validation.receipt.md)
  also passes, but its first full release attempt ended in an unexplained
  access violation; a passing retry does not resolve that failure.
  An intermittent WINDOWPOS test-thread overflow remains unexplained despite
  later clean reruns ([receipt](evidence/stage2-windowpos-stack-20261016/lib-rate.validation.receipt.md)).
  Production SD activation, other direct host-write/restore/rebind and
  module-retirement fences, DLL ownership and O-M3 guest-memory accounting
  remain open; no new iNavi run, performance result or full Stage 2/live
  acceptance follows. Persistent-owner
  architecture is deferred to Stage 8.
- The first [TaskFrame finite-Sleep source cut](evidence/stage3-finite-sleep-frame-20261015/validation.receipt.md)
  is resealed at HEAD `6626e74d`. Real-MIPS controls defer a same-process send
  while its receiver sleeps, then deliver through `GetMessageW`,
  `PeekMessageW(PM_REMOVE)` or `GetMessageWNoWait` and return the WNDPROC result
  to the sender. Focused finite-Sleep **17/17**, native same-process **4/4** and
  stale-HWND **1/1** passed. Its 18-block full suites passed release
  **2,025/0/9 ignored** and no-default **1,472/0/3 ignored**; Unicorn-only
  and desktop/no-trace checks, release build and post-build source hashes also
  passed. The initial **1,032/2/6 ignored** release attempt was diagnostic:
  obsolete hidden-message fixtures were repaired and the source was resealed.
  Reciprocal cross-Send was untested at that D-010 seal; A11b later reproduced
  a next-lease replay and A12 closes only its strict two-send source shape.
  Post-wake MsgWait remains untested. The user skipped the a04 live run; no
  new mount backup or iNavi run follows from these source gates, and full
  Stage 3 remains open.
- The three [real-MIPS cross-thread Send expected reds](evidence/stage3-getmessage-blocked-send-context-20261015/validation.receipt.md)
  are historical pre-repair source controls, not current iNavi failures. The
  connected A→B Send/GetMessage cut now has an exact
  [A10 source seal](evidence/scheduler-source-cut-a10/source-seal.md) and
  [validation receipt](evidence/scheduler-source-cut-a10/validation.receipt.md):
  18-group release-feature **2,045/0/9 ignored** and no-default **1,482/0/3
  ignored**, with unchanged pre/post source hashes, two alternate checks and a
  release build passing; EXE/PDB copies were SHA-preserved, with no app run.
  A09's four
  same-process Send failures remain historical. A10 checks exact blocked-
  GetMessage/native wait and birth before recovering B, then resumes A on the
  next bounded lease. Only the runnable-receiver fixture proves a subsequent
  no-replay slice; the three finite-Sleep fixtures stop at A's continuation.
  This is a bounded source/test gate, not full Stage 3 or live-iNavi proof.
  Reciprocal shapes beyond A12, cross-process parked-sender restore, legacy
  numeric-only HWNDs, post-wake MsgWait and `SMF_TIMEOUT` semantics remain
  separate.
- The strict same-process S→T→S plain-Send path has a bounded
  [A12 source/test gate](evidence/scheduler-source-cut-a12/validation.production.receipt.md):
  the post-format 18-group suites passed release **2,047/0/9 ignored** and
  no-default **1,482/0/3 ignored**. Normal real-MIPS controls cover both
  replies, exact wait retirement, preserved S context and no third send;
  a wrong-parent-birth control fails before guest entry. A11b's next-lease
  replay is historical for this exact shape, not evidence of a live iNavi
  deadlock. Third sender, timed/result-pointer, deeper nesting and
  cross-process variants can fail closed with terminal `Backend` and remain
  live-run risks. No app or writable-mount run was made for A12.
- The bounded [A13 GetMessage orphan source gate](evidence/scheduler-source-cut-a13/validation.final.receipt.md)
  prevents a same-birth blocked import payload from becoming runnable merely
  because its scheduler wait disappeared, and rejects an orphan selected at
  native entry before guest execution. Exact live modal-wait provenance still
  permits the tested button-tap continuation; a missing modal wait fails
  closed. Focused saved-context **7/7**, integrated orphan **1/1** and modal
  **11/11** controls passed. The final 18-group suites passed release-feature
  **2,052/0/9 ignored** and no-default **1,482/0/3 ignored**. This is not
  full modal, Stage 3 or live-iNavi acceptance; no app or mount run was made.
- The bounded [decoded pseudo self-exit source gate](evidence/stage3-raw-decoded-pseudo-exit-20261016/decoded-pseudo-exit.receipt.md)
  requires a witnessed live JALR and delay slot plus exact stopped caller and
  CPU-owner birth before the same D-011-preflight exit finalizer as the native
  path. A decoded PC-zero snapshot or static exit-shaped bytes cannot retire a
  process; raw COREDLL pseudo handling remains fixture-only and fail-closed.
  At source HEAD `5401466b`, the [broad offline suites](evidence/stage3-raw-decoded-pseudo-exit-20261016/broad-validation.receipt.md)
  passed release-feature **2,107/0/9 ignored** and no-default **1,511/0/3
  ignored**. This is neither whole-exit atomicity nor full Stage 3, live-iNavi
  or performance acceptance. No app or writable-mount run was made.
- The bounded [D-011 modal-presence gate](evidence/stage3-modal-exit-failclosed-20261016/validation.receipt.md)
  now refuses affected MessageBox/popup state before stopped-exit effects in
  three red-to-green source controls. The same sealed source passed release
  **2,137/0/10 ignored** and no-default **1,524/0/3 ignored**; modal-pixel
  restoration and live acceptance remain open. The later bounded
  [parked-survivor/saved-CPU-owner gate](evidence/stage3-survivor-resume-green-20261016/validation.receipt.md)
  preserves tested nonmodal foreign CPU trees on natural-parked and
  requested-active stopped exit and refuses invalid saved-thread ownership
  before effects. Its combined sealed source passed release **2,144/0/10
  ignored** and no-default **1,524/0/3 ignored** across 19 blocks each.
  The later bounded [first-slice `Exited` helper gate](evidence/stage3-first-slice-survivor-green-20261016/validation.receipt.md)
  preserves its tested foreign CPU, but does not execute guest MIPS or prove
  the classifier path. A held parent-queue borrow ends in typed terminal
  owner loss, not recovery. Two later ignored, executed
  [guest-launch](evidence/stage3-real-mips-first-slice-20261016/validation.receipt.md)
  and [native-fault](evidence/stage3-post-admission-native-fault-20261016/validation.receipt.md)
  reds show an exact-birth live child losing its only CPU via the
  already-exited-parent skip or ordinary first-slice error. The bounded
  [pre-CPU launch-retention gate](evidence/stage3-pre-cpu-launch-retention-green-20261016/validation.receipt.md)
  keeps an already-published launch available for retry before any CPU
  exists. Those reds are historical: Cut 3a's earlier focused debug/release
  real-MIPS controls passed, and the superseded producer-side a09 debug control kept
  the original faulted B CPU in non-runnable,
  exact-birth custody after A's staged return and native sync/finish. A
  termination request retains the CPU; same-Kernel accepted exit retires it
  and B's fixed-stack pin in that tested fault shape. Stamped a07 reproduced
  a synthetic post-load/pre-native-entry unpoisoned-quarantine failure; the
  producer-side seal passed a08 debug on the same source as a09; intermediate
  producer-only a10 focused release also passed. Current source enforces the
  terminal seal in `CpuCustody::quarantine_reserved` under Unicorn when the
  slot becomes quarantined, while the exit consumer keeps its poison check.
  Natural reachability and general closure remain unproved. The
  [owner-sealed Cut 3a receipt](evidence/stage3-cpu-custody-quarantine-20261017/validation.receipt.md)
  passes a11 focused release **1/0**, 19-block a12 release
  **2,184/0/9 ignored**, a13 no-default **1,543/0/2 ignored**, and three
  feature checks; all six source hashes matched. Pre-seal a03
  **2,183/0/9 ignored** remains development evidence.
  Status/debug report retained mapped-blob bytes, not measured RSS. The
  callback-scoped machine-state port, runnable/parked custody, public-run
  bypass removal and general private VM/file-view exit cleanup remain open.
  Other exit paths, no-context/PC resumability and
  live-app proof remain open; no new app or mount run was made.
- The bounded [exact-primary exit gate](evidence/stage3-primary-exit-birth-v4-20261017/validation.receipt.md)
  authenticates the canonical primary birth before tested stopped child/boot
  exits, and requests recovery when a stopped boot exit loses that birth.
  Its sealed source passed release **2,175/0/10 ignored** and no-feature
  **1,539/0/2 ignored** across 19 blocks each. The earlier 16-failure broad
  attempt is retained as failed evidence; these tests do not resolve general
  CPU custody, Stage 3, or live-app acceptance.
- The [scheduler-block self-reentry fence](evidence/stage3-scheduler-block-fence-20261017/validation.receipt.md)
   passes a real-MIPS preseeded-busy control and sealed 19-block release and
   no-default suites (**2,176/0/10** and **1,539/0/2 ignored**). It refuses a
   preseeded-busy scheduler service with a typed error; natural self-reentry,
   other hook aliases, cross-hook reentry and machine-owned CPU custody remain open.
- A [native-entry precondition](evidence/stage3-native-entry-precondition-20261017/validation.receipt.md)
  now refuses same-host-thread nested guest execution before private CPU
  setup; a preseeded-busy real-MIPS control verifies refusal and same-CPU
  retry. Sealed 19-block release and no-default suites passed **2,178/0/10
  ignored** and **1,539/0/2 ignored**. Natural reentry, a connected
  Kernel/framebuffer port and full child CPU custody remain unproved; the
  later non-runnable Cut 3a quarantine is bounded above.
- A bounded [checkpoint lineage guard](evidence/stage3-checkpoint-orphan-frame-green-20261016/validation.receipt.md)
  refuses monitor save/restore while canonical task records or pending
  launches are retained, or CPU custody remains. This prevents duplicate
  process births but is not live rewind: a D-011 recovery hold currently
  requires repair and restart.
- The E-28 native count-stop query and typed `CountLimit` pass focused real-MIPS
  controls. The count-adapter source passed 1,671 release tests (0 failed,
  16 ignored) and 1,259 no-feature tests (0 failed, 14 ignored), plus no-trace
  and formatting checks. Those counts are **historical** for that exact source.
  Later ARM stop-reversal/reset controls and a checked finish-before-handoff
  repair pass focused tests. That follow-up source also passed **1,674
  release tests, 0 failed, 16 ignored** and **1,259 no-feature tests,
  0 failed, 14 ignored**
  ([release log](evidence/e28-run-outcome-20261014/broad-validation/release-unicorn-trace-desktop.raw.log),
  [no-feature log](evidence/e28-run-outcome-20261014/broad-validation/no-feature.raw.log)).
  The Unicorn+desktop/no-trace release check, format and diff checks also
  passed; exact source/log hashes are in the
  [post-order validation receipt](evidence/e28-run-outcome-20261014/broad-validation/validation.receipt.md).
  A later additive per-call native report retains outgoing-attempt evidence
  across a child handoff and distinguishes preflight `NoAttempt`; its frame is
  still `LegacyUnbound`, and the app scheduler does not consume it. Focused
  report/decoded-exit controls pass. The earlier report-source gates passed
  **1,682 release tests, 0 failed, 16 ignored** and **1,262 no-feature tests,
  0 failed, 14 ignored**, with check/format/diff gates green
  ([validation receipt](evidence/e28-run-outcome-20261014/detached-report-current-source-validation.receipt.md)).
  An executed ordinary finish-failure retry red is now corrected: the
  failed outgoing CPU rejects another native entry. The corrected source
  passed focused 1/0, full release **1,683/0/16 ignored**, no-feature
  **1,262/0/14 ignored**, and applicable check/format/diff gates
  ([red](evidence/e28-run-outcome-20261014/ordinary-finish-retry-expected-red.receipt.md),
  [green](evidence/e28-run-outcome-20261014/ordinary-finish-retry-green.receipt.md)).
  The older 1,674/1,259 and 1,682/1,262 counts above remain historical.
  E-29 below used a pre-fix executable and does not validate this correction.
  Later typed-stop and private entered-attempt wait controls passed focused
  release **4/0**. A pre-entry fence then exposed separate ready-worker and
  orphaned-WNDPROC losses; their combined repair passes focused fence **2/0**
  and executed-wait **5/0** on a later source
  ([receipt](evidence/e28-run-outcome-20261014/private-preentry-ready-and-orphaned-focused-green.receipt.md)).
  A subsequent ready GetMessage/private-slab integration control passes
  focused **1/0**, with the fence and wait filters rerun **2/0** and **5/0**
  on its rebuilt binary
  ([receipt](evidence/e28-run-outcome-20261014/private-ready-get-message-slab-focused-green.receipt.md)).
  The [E-28-source broad gate](evidence/e28-run-outcome-20261014/final-current-source-validation.receipt.md)
  passes release **1,691/0/16 ignored** and no-feature **1,263/0/14 ignored**,
  plus both Unicorn no-trace checks and format/diff checks. No live iNavi
  validation is claimed for that source.
  An exploratory same-handle nested-start hook probe exited
  `0xC0000005` before a query assertion, was removed, and remains unresolved
  ([E-28 ledger](docs/evidence/ledger.md#e-28-native-count-stop-witness-and-typed-adapter-2026-10-14-executed),
  [nested risk audit](evidence/e28-count-witness-audit-20261014/nested-count-risk.receipt.md)).
- The latest bounded iNavi run used the preserved E-37 fresh-default EXE after
  a separate single-run approval and fresh adjacent, SHA-verified ZIP backups.
  It exited normally at 330.392 s with no watchdog trip. Across 160 host
  samples, RSS/private maxima were **430.7/445.0 MiB**; the true continuous
  peaks remain unknown. The real splash persisted, with no map.
  Guest `COM7:` reads transferred 2,262 B of injected raw NMEA and tap down/up
  reached the iNavi WNDPROC. Neither GPS parsing nor navigation was proven.
  Busy iNavi slices measured 26.359 run-only MIPS, within the older E-9
  22–34 MIPS range; overall busy slices across helpers measured 17.120 MIPS.
  The approval was consumed by this one run
  ([E-38 analysis](evidence/e37-live-20261014T051228Z/analysis.receipt.md),
  [historical offline validator](scripts/analysis/e38_validator.py)).
- The E-20 native controls passed real-MIPS A→B→A same-VA bytes with fresh
  owners: public `VirtualAlloc` private memory and a separate injected
  `FileMappingView` fixture. That E-20 view fixture did not validate public
  `MapViewOfFile`, named reopen or shared alias coherence; the later D-008
  source gate tests those separately. On the E-20 combined
  final source, full release passed **1,742/0/14 ignored** and no-feature
  **1,296/0/13 ignored**; the four public GMS tests passed in both. GMS now
  includes boot-reserved RAM in its stable-private-page budget. At that E-39
  gate the initial heap, image and legacy views were not charged; E-44 later
  added committed initial heap/stack slabs to the stable-page budget; O-M2
  extends the sparse heap window without reintroducing copied spillover. Image
  and legacy views remain outside that budget. These are source gates,
  not another app run
  ([E-20 receipt](evidence/e20-native-same-va-20261014/validation.receipt.md),
  [GMS receipt](evidence/gms-device-report-20261014/validation.receipt.md)).
- D-008 first public maps now use one canonical, budget-charged shared page
  authority instead of the old copy-backed view. Focused real-MIPS named
  A→B→A and in-slice alias tests pass, as do DIB-to-public migration, file-
  backed prefix/tail and memory-budget rollback controls. The final same-
  source broad gates passed **1,744/0/14 ignored** with release features and
  **1,297/0/13 ignored** with no features. The no-feature suite caught an
  offset underflow hidden by release wrapping; its guarded fix is included in
  both final gates. This source work did not launch iNavi, measure a new boot
  time, or establish map readiness
  ([D-008 receipt](evidence/d008-shared-view-20261014/validation.receipt.md)).
- E-42 selects whole R/RX image-page candidates by PE section characteristics
  and borrows caller-supplied bytes without a helper copy. Focused
  no-feature/Unicorn controls passed; full no-feature passed
  **1,302/0/13 ignored** across 17 blocks. The synthetic bytes were patched
  manually, not by the actual loader. No sealed storage, native page mapping,
  host-write guard, 56-MiB benchmark or app run is claimed
  ([E-42 receipt](evidence/immutable-image-pages-20261014/validation.receipt.md)).
- At E-43, a read-only production-loader probe sealed **1,379/2,168** iNavi main-
  image pages (**5.386719 MiB**), including **1,224** pages whose final bytes
  were changed by one-time loader patching. All 1,379 non-writable-section
  candidates seal. The initial per-owner mapped-blob inventory is
  **36,679,680 B (34.980469 MiB)**, including 16 MiB of heap backing, 4 MiB
  of stack backing, and 5,648,384 B of sealed main-image bytes. The earlier
  E-42 selector and 155-page equality-filter results are historical. This
  probe did not execute iNavi or touch writable mounts. The final source
  passed release `unicorn,trace,win32-desktop` **1,761/0/17 ignored** and
  paired debug no-feature **1,303/0/13 ignored**, both across 17 blocks;
  focused native-write overlap tests and a no-trace check also passed.
  Exhaustive host-write audit, budget accounting, app speed/memory, and map
  readiness remain open ([E-43 receipt](evidence/sealed-main-image-20261014/validation.receipt.md)).
- E-44 replaced the initial 16-MiB heap and 4-MiB stack owner blobs with
  owner-qualified fixed slabs and passed formatted-source release
  **1,785/0/17 ignored**, paired debug no-feature **1,321/0/13 ignored**,
  and a no-trace check. A synthetic 56-MiB RX control reduced owned blob
  capacity/fresh writes/readback by **20,971,520 B (20 MiB)**; a same-mode
  debug control passed, but single samples and slab reservation outside the
  setup timer do not establish a speed or host-RSS gain. An exact blocked
  serial/NMEA path test passed across two untouched stack pages and a fresh
  owner. Unconverted direct-import cold-stack outputs can still fail closed;
  stale different-birth rejection was not independently tested. There was no
  E-44 iNavi run or map-readiness result
  ([E-44 receipt](evidence/heap-stack-20261014/validation.receipt.md)).
- O-M2 made heap high-water, block metadata, handles and queued typed payloads
  birth-qualified, enlarged the sparse fixed heap reservation to 256 MiB, and
  removed the copied spillover path. Paired all-target source suites and a
  no-trace check pass. Heap address reuse, full clone relineage and live app
  speed, host RSS or map readiness remain unproved
  ([O-M2 receipt](evidence/om2-heap-20261014/validation.receipt.md)).
- An earlier persistent-owner bounded iNavi run exited normally at its 330 s cap
  and remained on the genuine splash, not the map. Guest `COM7:` reads
  transferred injected raw NMEA bytes, and tap down/up WNDPROC returns were
  recorded. GPS fix, map readiness and route interaction were not demonstrated.
  Its sampled working-set and private peaks were 1,527.3/1,540.8 MiB. The
  read/write mounts were backed up as adjacent ZIPs before that run
  ([E-29 analysis](evidence/e22-live-20261013T211230Z/analysis.receipt.md)).
- A later E-31-source diagnostic did **not** complete its 330 s cap: the host
  crashed in Unicorn TCG code emission at 135.282 s, before the tap. It still
  displayed the real splash; 121 native slices completed after input-ready
  +120 s, and positive guest COM7 reads were retained. Its peak sampled RSS
  was 1,578.6 MiB. The exact EXE/PDB/dump, fresh adjacent mount ZIPs, and
  limits are retained; the Windows handler's known commit-tail defect is not
  yet proven to have caused this crash
  ([E-32 analysis](evidence/e22-live-20261014T002535Z/analysis.receipt.md)).
  A narrow vendored Windows fix subsequently passed isolated real-MIPS native
  controls and the broad suites. The later E-38 fresh-default app run survived
  this crash horizon, but its different source/owner mode cannot attribute
  that result to the E-33 fix
  ([E-33 repair](evidence/e33-windows-tcg-commit-tail-20261014/validation.receipt.md)).
- Historical comparison, not current acceptance: the persistent-owner 650
  build passed 1,618 tests before a normal app run crashed at 129.108 s with
  `0xC0000005` and about 1.62 GiB RSS. The installed BED2 fallback's later
  safe-yield guard passed 1,593 tests and a 330.234 s GPS/tap run, still on
  splash; sampled RSS/private were 555.6/568.4 MiB and the tap callback took
  0.397 s. These different-source runs are not a controlled performance
  comparison ([history and caveats](docs/evidence/ledger.md)).
- Hookless trampoline execution remains the default. The live-pump limit is
  one million instructions per native slice; E-28 did not change scheduling
  policy, guest clock or CE quantum handling.

See `PLAN.MD`, `PROGRESS.md`, `TODO.md`, and `KNOWN_BUGS.md` for the current
roadmap and durable project memory. Source evidence is indexed in
`SOURCE_REFERENCES.md`.

## Active Frontiers

- CE-backed GDI parity: bitmap creation, DIB color tables, alpha-preserving
  blits, compatible bitmap behavior, printer abort callbacks, and font
  enumeration validation.
- Message and windowing parity: hotkeys, modal loops, cross-thread sends,
  scheduler waits, and the runtime transition from splash/resource loading to
  map interaction.
- Storage and device parity: mounted file behavior, FSDMGR surfaces, serial GPS
  input, and sensor/device notification timing.

## Basic Commands

Run the base bootstrap:

```bash
cargo run
```

Inspect a PE image without executing it:

```bash
cargo run -- --image D:\INAVI_Emulator\INAVI\INavi\iNavi.exe
```

Run with Unicorn CPU execution:

```bash
cargo run --release --features unicorn,trace,win32-desktop -- --registry registry.reg --devices serial_devices.json --mount-config mounts.toml --image D:\INAVI_Emulator\INAVI\INavi\iNavi.exe --dll-search-dir "D:\INAVI_Emulator\DUMPPLZ\Windows" --desktop virtual --remote-server 192.168.0.39:8765 --framebuffer-dump target\last-framebuffer.ppm --run-cpu
```

Expose the v2-compatible remote API:

```bash
cargo run --release --features unicorn,trace,win32-desktop -- --registry registry.reg --devices serial_devices.json --mount-config mounts.toml --image D:\INAVI_Emulator\INAVI\INavi\iNavi.exe --dll-search-dir "D:\INAVI_Emulator\DUMPPLZ\Windows" --desktop host --remote-server 192.168.0.39:8765 --run-cpu
```

Use `--cpu-instruction-limit N` or `--cpu-wall-clock-limit-ms N` with
`--run-cpu` for bounded diagnostic runs.

Use `RUST_LOG=ce.slice_profile=debug` to compare guest instructions and MIPS.
`WINCE_EMU_GLOBAL_CODE_HOOK=1` restores the global code hook; arbitrary-PC
diagnostics also select it even when legacy sparse-hook flags are present.
`WINCE_EMU_LIVE_PUMP_RUN_SLICE_INSTRUCTIONS=0` is an opt-in scheduling
experiment, not the default.

## Remote API

The HTTP surface includes:

- `GET /api/v1/status`
- `GET /api/v1/frame.jpg`
- `GET /api/v1/debug/screenshot.png`
- `GET /api/v1/video.mjpg`
- `/api/v1/input/*`
- `/api/v1/sensors/*`
- `/api/v1/control/*`
- `GET /api/v1/control/ws`
- `GET /api/v1/audio/ws`

The remote CLI compatibility flags include `--remote-bind`, `--remote-port`,
`--remote-token`, `--remote-video-fps`, `--remote-jpeg-quality`, and
`--remote-audio*`.

## Validation

Use the focused checks that match the area being changed:

```bash
cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel
cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file
cargo test --features unicorn,trace,win32-desktop --test basic_subsystems
cargo check --features unicorn,trace,win32-desktop
```

Run `cargo fmt` or targeted `rustfmt` after Rust edits, and use
`git diff --check` before handoff.
