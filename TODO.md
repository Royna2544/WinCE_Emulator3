# TODO

Refreshed on 2026-10-14. This file is limited to active, plan-aligned work.

## Immediate Engineering Queue

- E-32's bounded iNavi run crashed in host Unicorn TCG code emission at
  input-ready ~+134.8 s, before the tap; it does not support a splash-cause
  inference. The dump/EXE/PDB and fresh original-mount ZIPs are preserved
  ([receipt](evidence/e22-live-20261014T002535Z/analysis.receipt.md)).
  E-33 reproduced the pristine small-arena host AV and applied a narrow
  Windows commit-tail clamp; 1/2/8-MiB and default native MIPS controls plus
  full-feature/no-feature suites now pass
  ([repair receipt](evidence/e33-windows-tcg-commit-tail-20261014/validation.receipt.md)).
  Do not conflate this handler fix with attribution of the app crash or
  default-cache footprint acceptance. A second writable-mount run needs new
  user authorization; no app run followed this repair. The one production-source
  safety exception is recorded as [D-009](docs/DECISIONS.md); stop further
  arena investigation until Stage 8 instead of expanding this fix.

- Begin the user-authorized scheduler-core rewrite within the staged
  plan-first/common-anchor gates. Read reviewed
  `docs/ARCHITECTURE.md`, `docs/PLAN.md` and the relevant subsystem page;
  name ownership/consumers/failure/test gates before code. Phase 1 was
  authorized, but the first persistent-owner 650 trial host-crashed at
  129.108 s and 1.62 GiB RSS despite 1,618 passing tests. BED2 is a
  hash-preserved measured fallback, not the current release EXE; current
  source remains experimental despite the later E-19 suite.
  Per D-003, finish the Stage 2 fresh-engine stop taxonomy/default-owner and
  same-VA/shared-view coherence gates; then migrate one connected
  Task/object authority slice. Do not use the persistent-owner success path as
  a substitute for that gate.
- E-27's test-only child `WM_ACTIVATE` park/resume and healthy/pending-loss
  real-MIPS controls now pass. The controlled stale-stub replay is not tied
  to the 2,518 live warnings or splash stall; any recovery change still needs
  identity-matched live evidence ([ledger E-27](docs/evidence/ledger.md)).
- E-29's pre-retry-fix bounded run still ends on the real splash despite
  positive guest `COM7:` reads and dequeued/returned tap messages. Before
  changing scheduling or window policy, make a bounded +100-to-180 s
  diagnostic joining native
  attempt PID/birth/TID/count/stop/PC/time to TID 1's callback continuation
  and visibility changes on splash window `0x00020008`; record the any/all
  mode and event generation of the apparently signaled TID 10 wait. Static
  service counters, a busy host CPU, and the hidden map-window hierarchy are
  leads, not a proven cause ([E-29 receipt](evidence/e22-live-20261013T211230Z/analysis.receipt.md),
  [post-120 trace design](docs/evidence/e29-post120-native-attempt-trace-design.md)).
  The latter is design only; wait id 833 needs a separate boot-forward trace
  because registration may precede the post-120 capture window. E-30's
  [host-consumer bridge](evidence/e30-native-attempt-observer-20261014/validation.receipt.md)
  now exposes the outgoing report to optional live/monitor observers; no
  writer, native-stage markers, loop heartbeat, or child-report transport
  exists yet. Do not interpret an empty attempt stream as guest idleness.
- **Stage 2, now, in this repo** (source map:
  `docs/subsystems/cpu-engine.md`, "Stage 2 source map"):
  0. D-007's in-repo Unicorn dependency patch is present. The post-order
     serial full release suite in the sole D: target passed 1,623 tests, with
     0 failures and 3 ignored before the E-12 repaired-PC change; this is a
     historical dirty persistent-owner checkpoint, not fresh-engine
     acceptance. The post-E-12 suite passed 1,626 tests (0 failed, 3 ignored);
     Unicorn-only and Unicorn+desktop/no-trace release checks and the
     no-feature suite (1,239 passed, 0 failed, 1 ignored) also passed. These
     remain dirty-source validation, not Stage 2 acceptance (ledger E-13).
     Top-level run
     evidence and release EXE/PDB originals were hash-preserved and removed
     after approval. Four old C: source/build copies and the auxiliary Git
     worktree were separately retired after approval, with selected unique
     source, test patch and binaries preserved under D: `evidence/`. The D:
     source, target and evidence remain in place.
  1. Continue fresh-engine lifetime policy work only in this repo. The E-7
     count-stop and native-error controls passed both focused and post-order
     full release validation, with exact receipts. The separately run ignored
     closed-owner control remains expected red at the final owner-closure
     assertion. Resolve O-P4 closed-owner Context transfer and O-M1
     private/shared guest-byte coherence here.
  1a. The E-14 injected same-VA native-view expected red is now an unignored
      passing default regression after owner-qualified seeding and exact-view
      readback (ledger E-16). Two focused closed-live/ambiguous-lazy controls
      and outgoing-before-rotation passed. Continue O-M1 with per-address-space
      allocation/lazy ownership, public same-VA `MapViewOfFile`, private
      guest-written `VirtualAlloc` persistence across fresh owner closure,
      alias/flush/unmap and remote query controls. The narrow E-16 green does
      not close any of those gates. On the historical legacy-backing E-18
      source, the ignored E-18 control was expected red: guest MIPS wrote
      `0x11223345` to an internal private allocation,
      but after a test-only idle-owner closure the fresh engine loaded stale
      seed `0x11223344` at the correct PC without a fault. D-008 now selects
      AddressSpace-owned committed-page host slabs as the private-page target,
      not outgoing private-page copy-back. E-21 introduced one canonical
      HandleTable process-birth directory independent of public aliases; its
      five focused controls, full release/no-feature suites and alternate
      checks pass. E-22 attaches checked birth keys, non-reused within-lineage
      allocation incarnations and producer kinds to virtual metadata; four
      focused controls and combined release/no-feature suites pass. This
      did not migrate bytes. E-23 now keys private-VA metadata and lookups by
      process birth; two focused controls pass, but the native VA-only map
      retains a global overlap fence and blocks E-20 same-VA allocation.
      Full release 1,649/0/5 ignored and no-feature 1,250/0/2 ignored pass
      on the same source, while explicit ignored E-18/E-20 reruns still fail
      at their intended stale-byte/child-allocation gates (ledger E-23).
      E-24 closed E-23's foreign legacy file-view Free/Flush/Unmap mutation
      escape with a public child-rejection/parent-success regression and
      passed 1,650/0/5 ignored release plus 1,251/0/2 ignored no-feature;
      E-18/E-20 stayed red on E-24 source (ledger E-24). E-25's changed
      internal `cfg(test)` slab-opt-in made the real-MIPS fresh-owner byte
      control green; public `VirtualAlloc` remains `LegacySeed` and E-20 is
      still open. Its first full release run exposed a synthetic-PID birth
      regression (726/1/3 ignored); after the narrow compatibility fix, the
      corrected source passed full release 1,664/0/4 ignored, no-feature
      1,258/0/2 ignored, and a no-trace Unicorn+desktop release check (ledger
      E-25; separate focused and broad receipts). Complete the same-PID DIB
      Unmap path without weakening the owner guard; Flush still works there.
      Selected-process snapshots, worker sweep, wait pins and clone lineage
      remain open. Next connect
      owner-scoped native reservations/mapping, cross-family overlap checks,
      reserve/commit behavior and remote queries to that directory; Stage 3
      extends the same authority. Extend E-25's test-only slab backing to
      public private pages with exact pinned native maps, checked
      failure/publication and no whole-`initial_bytes` reseed for that family.
      A failed unmap must retain its pin and fail the attempt, not recycle a
      stale VA. The changed E-25 control is green; separately prove public
      COREDLL `VirtualAlloc`, two-PID same-VA isolation, free/reallocation,
      protect/decommit and independent clones. Unmigrated
      view/blob families retain an explicit checked copy boundary until their
      connected migration. Add end-to-end immediate CreateProcess sync-error
      fault injection; its error latch is source-reviewed only. Measure host
      RSS/private and guest busy work on matched runs before claiming benefit.
  2. O-P1's E-28 native count-stop query and typed `CountLimit` adapter pass
     focused real-MIPS controls. The previous count-adapter source passed
     **1,671/0/16 ignored** release, **1,259/0/14 ignored** no-feature,
     no-trace release check and format green; those are historical counts for
     a source before the later focused finish-order and edge-control changes
     ([adapter](evidence/e28-count-witness-audit-20261014/adapter-slice.receipt.md),
     [broad](evidence/e28-count-witness-validation-20261014/broad-validation.receipt.md),
     [native C seam](evidence/e28-count-witness-audit-20261014/native-c-seam.receipt.md),
     [vendor patches](vendor/unicorn-engine-sys/PATCHES.md)). The new fact
     witnesses an accepted count-stop request, not an exclusive cause or CE
     quantum; the existing 1M-instruction service-boundary policy is unchanged.
     A direct ARM Thumb stop-reversal/next-start reset control passed (3/0/0
     including the two public MIPS count tests); its same-handle nested-start
     exploratory probe exited `0xC0000005` before a query assertion and was
     removed from default tests
     ([edge receipt](evidence/e28-count-witness-audit-20261014/edge-controls.receipt.md),
     [read-only risk audit](evidence/e28-count-witness-audit-20261014/nested-count-risk.receipt.md)).
     The narrow checked `uc.finish()`-before-handoff fix has an intended red
     and three focused green controls; parent error/view bytes stay with the
     outgoing owner, child stays parked, and the failed native owner is
     discarded ([finish-order receipt](evidence/e28-run-outcome-20261014/finish-order-regression.receipt.md)).
     On this later source, the locked full release suite passed **1,674/0/16
     ignored**
     ([raw log](evidence/e28-run-outcome-20261014/broad-validation/release-unicorn-trace-desktop.raw.log));
     locked no-feature passed **1,259/0/14 ignored**
     ([raw log](evidence/e28-run-outcome-20261014/broad-validation/no-feature.raw.log)).
     The Unicorn+desktop/no-trace release check, format and diff checks also
     exited 0; exact source/log hashes are in the
     [post-order validation receipt](evidence/e28-run-outcome-20261014/broad-validation/validation.receipt.md).
     An additive owned per-call `NativeAttemptReport` now transports primary
     native/query facts and the outgoing owner through preflight, error and
     handoff paths. Focused parent/child, checked-finish, preflight, private
     native-error and decoded-exit finish-error controls pass. This is still
     observational: `AttemptFrameBinding::LegacyUnbound` cannot authorize a
     Task transition; E-30 now offers the outgoing report to optional
     live/monitor observers before handling the separate result, but normal
     production still passes `None` and selected-CPU outcome remains the
     rotation input. The
     ordinary checked-finish failed-retry control is now green: a failed
     entered attempt poisons the outgoing CPU, clears provisional context,
     and a retry returns `NoAttempt` before another native entry
     ([red](evidence/e28-run-outcome-20261014/ordinary-finish-retry-expected-red.receipt.md),
     [green](evidence/e28-run-outcome-20261014/ordinary-finish-retry-green.receipt.md)).
     Two further executed reds narrowed wait ownership: a host-wall debug
     marker could delete the Kernel's wait on a failed native query adapter;
     a private entered-attempt error could retain the Kernel wait while losing
     its CPU record. The typed-success cleanup and handles-only salvage passed
     four focused real-MIPS controls on their source
     ([focused receipt](evidence/e28-run-outcome-20261014/typed-stop-and-private-wait-green.receipt.md)).
     A separate private-page pre-entry fence red returned `NoAttempt` with
     the Kernel wait intact but CPU wait lost
     ([red receipt](evidence/e28-run-outcome-20261014/private-preentry-wait-salvage-expected-red.receipt.md),
     [repair design](docs/evidence/e28-preentry-handle-salvage-design.md)).
     The first handles-only fix passed its non-ready wait control, then
     [ready-worker](evidence/e28-run-outcome-20261014/private-preentry-ready-wait-expected-red.receipt.md)
     and [orphaned-WNDPROC](evidence/e28-run-outcome-20261014/private-preentry-orphaned-wndproc-expected-red.receipt.md)
     reds exposed two more pre-entry losses. The combined repair passes the
     [focused fence **2/0** and wait **5/0** filters](evidence/e28-run-outcome-20261014/private-preentry-ready-and-orphaned-focused-green.receipt.md).
     A [ready GetMessage/private-slab integration control](evidence/e28-run-outcome-20261014/private-ready-get-message-slab-focused-green.receipt.md)
     now passes **1/0** with fence **2/0** and wait **5/0** rerun on its
     rebuilt binary. The [current broad source](evidence/e28-run-outcome-20261014/final-current-source-validation.receipt.md)
     passes release **1,691/0/16 ignored**, no-feature **1,263/0/14 ignored**,
     both no-trace Unicorn checks and format/diff checks. Cover other resume
     families and native-entry failure consistency; do not infer rollback.
     A separate [stale-snapshot remote-target red design](docs/evidence/stale-snapshot-host-routing-design-20261014.md)
     is proposed but unexecuted; keep its routing assertion separate from
     the snapshot-only host idle-slice pacing gate.
     The report remains observational; no production trace sink consumes it.
     Finish the other fallible context/PC/byte-view/lease-tail and
     partial-effect paths, then
     migrate beyond the E-30 live/monitor observational bridge with explicit
     owner/frame validation
     ([report design](evidence/e28-run-outcome-20261014/sealed-report-next-gate.receipt.md)).
     The earlier report source passed **1,682/0/16 ignored** release
     and **1,262/0/14 ignored** no-feature; three check profiles, format and
     diff checks exited 0
     ([validation receipt](evidence/e28-run-outcome-20261014/detached-report-current-source-validation.receipt.md)).
     The retry-fix source passed **1,683/0/16 ignored** release and
     **1,262/0/14 ignored** no-feature, plus focused 1/0, applicable checks,
     format and diff gates
     ([green validation](evidence/e28-run-outcome-20261014/ordinary-finish-retry-green.receipt.md)).
     Do not rerun the recursive Rust hook probe; actual same-handle nested
     execution is unvalidated, not a prerequisite for ordinary count stops.
     A non-recursive private query-state control may check ambiguity policy
     without claiming recursive safety. Three real-MIPS test-only
     post-real-query Rust adapter-error controls pass; they do not establish
     a genuine C `uc_query` failure
     ([focused receipt](evidence/e28-run-outcome-20261014/query-adapter-error-controls.receipt.md)).
     Add genuine C query-error and same-instruction competing-hook controls;
     finish diagnostic-private query provenance validation. Remaining
     sparse/hookless stop reasons, typed restart-PC repair and failed
     PC-write/stop injection also remain. Do not close O-P1, Stage 2 or S3-A
     or infer a scheduler-core rewrite from the count-only adapter. Two
     `host_wall_clock_stop` decision reads still mix a debug marker into
     native-deadline PC repair and retained wall-snapshot promotion. Replace
     those only after focused marker/typed-disagreement and post-query/tail
     failure controls; marker presence is not a typed stop witness, and
     removing both reads alone does not close O-P1. The first JR-repair
     predicate has an [executed expected-red pure control](evidence/e31-native-deadline-jr-repair-20261014/expected-red.receipt.md),
     now ignored in the normal suite; a separate unconditional post-run JR repair also needs a
     real-MIPS mixed timeout/typed-peer-yield PC test before either path's
     policy can be changed safely. E-31's [final-source validation](evidence/e31-native-deadline-jr-repair-20261014/validation.receipt.md)
     remains diagnostic-only.
  2b. The E-12 repaired-PC scalar-fallback loss is fixed for its deterministic
     fresh-owner control: expected red before the change, then focused green
     with F0-F31/FCSR plus HI/LO/PC, guest FPU readback, and checked FCSR
     success/failure controls (ledger E-13). Keep the native snapshot for
     exact raw continuations; writing a synthesized PC into it can retain
     stale branch `hflags`. Closed-owner exact-key Context transfer, guest
     public `VirtualAlloc` coherence (historically red under E-18 and still
     unaccepted after E-25's test-only slab opt-in) and the broader
     Stage 2 gates remain open. E-18's PC/normal-stop/no-fault checks passed;
     its stale byte is a RAM backing gap, not evidence of Context failure.
  3. The later E-17 visible-peer source passed the full release suite
     (1,632/0/3 ignored), both applicable no-trace release checks and the
     no-feature suite. E-16 O-M1 was separately validated on its own source.
     A later E-18 ignored private-byte fixture is expected red when run
     explicitly; its distinct test-only source passed the default full release
     suite (1,632/0/4 ignored), with E-18 the fourth ignored control.
     Both alternate release checks exited 0 and the no-feature suite passed
     1,239/0/1 ignored on E-18 source (ledger E-18; local
     `evidence/stage2-o-m1-private-fresh-20261013/post-e18-validation.receipt.md`).
     This does not close its
     explicitly run red private-byte gate.
     The later E-19 global-hook source passed the full release suite
     (1,634/0/4 ignored), both no-trace release checks and the no-feature
     suite (1,239/0/1 ignored). These
     counts do not turn E-18 green or supersede its expected-red diagnosis.
     E-22's later combined source passed 1,647 release tests (0 failed,
     5 ignored) and 1,248 no-feature tests (0 failed, 2 ignored). Its bounded
     live run reached the configured 330 s limit and exited 0, but remained
     on splash; retained `COM7:` reads prove delivery of injected NMEA bytes,
     but parsed GPS fix and map use do not follow. The new feed was not
     input-matched to BED2. After remaining source fixes, repeat
     applicable profiles and a bounded live run before any Stage 2 claim.
     Record any newly exposed native errors in KNOWN_BUGS; neither the frozen
     1,634-pass E-19 source suite nor the decorative splash closes
     Stage 2.
- Parallel Stage 8 native lane (diagnostic only, never gates Stages 2-6):
  the evidence is in ledger E-3 to E-6. Before any production budget change,
  the remaining open items are concurrent in-flight VEH teardown, VEH
  reentrancy, normal translated-page boundaries, full-backend
  context/FlatView and the real-app budget. Do not spend Stage 2 time here.
- Then follow dependency order: CPU stop/VM coherence on fresh engines,
  canonical task/object identity
  with alias/execution/wait pins, full continuation ownership, explicit outcomes
  with CE6 priorities/quantum, and one guest clock/live event fence. Full Task
  state, virtual time and splash-to-map bring-up remain open. Thirteen
  Task/alias controls ran historically in private copies (ledger E-8/E-11)
  and have now been ported to this D: checkout (ledger E-26). Their focused
  default runs have one active close-source pass and twelve ignored; explicit
  current-source runs fail all twelve at intended assertions, including
  final-public-handle loss, alias-indexed wake and registered-wait target
  loss. Keep the fixtures as the Task migration baseline; add genuine native
  last-public-handle execution, wait acquisition and continuation controls
  before claiming lifecycle acceptance. Native wait/exit FPU controls remain staged;
  the actual-backend count-stop and nonzero-unmapped-error controls were the
  expected-red E-7 mirror baseline; their D-source controls pass the current
  full suite but do not close the broader Stage 2 gate.

- Carry forward the validated entry-only and safe-yield guards (EE18/89F9),
  1,593 tests and the clean330.234s BED2 actual workload, still on splash.
  Native preguard fixtures reproduce
  PC0 with a valid IAT and duplicate-delay replay; do not claim live crash
  attribution or universal native context safety from those synthetic tests.
- Finish per-primary-native-attempt outcome/actual expiry/fault telemetry under
  existing profile logging; summary and terminal retained snapshots cannot
  establish whole-run native behavior. The typed D-source outcome has
  focused E-15/E-17 coverage for two proven hook reasons, but final stop taxonomy,
  full-backend
  ownership/rotation and live controls remain open. Four native counted replay
  tests over64 boundaries already pass.
- Diagnose/fix the actual normal-run PC0 at260s before claiming the new build
  stable. Preserve hard-failure handling; trace signaled pool wait4029/TID10
  return0xe6de4 versus native JALR target and authentic context. Do not infer
  corruption from profile startPC/endTID mismatch or force a guessed resume.
- Follow the second startup initializer at 0x31970/return 0x2be44 with the
  validated exact-stop debugger after authentic callback restoration. Retain
  invocation/thread/stack identity; distinguish setup failure from lost return.
  Global-hook diagnostic timing is not normal boot timing.
- Revalidate the native GetMessage/sent/SetWindowPos boundary after the observed
  late pool-worker CPU failure is diagnosed. FIFO/wait/context, sent identity
  and 28-byte MSG output protection pass regressions; actual GPS/tap worked
  before the failure, but neither tests nor HTTP ACK establish map/stability.
- Bind the live MDC object and stack chain through method 0x12e15c, engine
  return 0x12e180 and pool return 0xe6da4. Binary evidence permits persistent
  repeats; do not equate one finite cache lookup with whole-job completion.
- Before heap reuse, establish authoritative allocation owner, original reserved
  footprint and unique incarnation (avoid prepared-token ABA) with CE evidence
  and measured need. The checked initial byte boundary now passes 1,571 tests;
  it does not justify global same-handle reuse or page reclamation.
- Profile actual boot/frame/input latency and host RSS/private memory from
  before launch, then through splash-to-map bring-up. First-splash timing is
  not full application readiness; keep captures phase-aligned and bounded.
- Bounded read-only windows pass 1,527 tests and lower the 330 s sampled peak
  RSS from 1,299.8 to 562.3 MiB. Measure remaining guest pages/native engine,
  trace capacity and any writable buffers before changing another policy;
  ordinary process rotation moves owners rather than cloning full kernels.
  The new reallocation-prerequisite repeat peaks at RSS 566.0 MiB; it is not
  a late-footprint improvement over the bounded-file baseline.
- Investigate the shared monotonic heap high-water mark mirrored into each
  process's spillover backing. Record actual byte ownership/capacity and read
  CE HeapAlloc/HeapFree evidence before introducing reuse or page reclamation.
  Never unmap a guessed free range or collapse independent process contexts.
- Reduce tap-delivery lag using actual queue/slice
  evidence. DOWN-before-UP and sustained positive GPS payload reads are now
  proven on BED2; its single tap takes0.397s (prior0.835/1.3-1.6s), not a controlled
  gain or proof of map readiness/responsive navigation.
- Continue the live launch loop past `resmapi_800x480.bin` map-skin decode
  with real visible-message delivery (post `2fcc365f`/`dbaf4791`/`265e751a`):
  confirm the map screen presents, then re-validate `COM7:`/`MFS1:`/`SMB1:`
  opens, GPS drain, and touch-driven map navigation on this baseline.
- Guest throughput (Phase 0 landed/pending, see PROGRESS.md 2026-10-12): the
  amortized block hook and hookless trampoline mode (default;
  `WINCE_EMU_GLOBAL_CODE_HOOK=1` restores the global code hook) took iNavi
  from ~1 MIPS to 22-34 MIPS in busy slices. The previous "scope the code hook
  by pages/regions" plan is closed: page-granular scoping was a negative A/B
  and hookless mode superseded it. Remaining throughput levers, in order:
  1. Replace the 1M-instruction `emu_start` count (Unicorn per-instruction
     counting hook, and the de facto round-robin tick) with a real time-based
     scheduler quantum, then set `LIVE_PUMP_RUN_SLICE_INSTRUCTIONS` to 0
     (busy slices reach 30-106 MIPS). CreateProcess replay and block sampling
     are fixed; validate remaining helper starvation and asynchronous native
     timeout branch-state preservation before changing the default.
  2. Persistent per-process engines (per-slice fixed cost is 15-25% at current
     slice lengths, plus a cold translation cache every slice).
  3. Virtual time with idle fast-forward: guest waits of 5-30 s (and 1 s
     import-trap idle slices) now dominate wall time.
  Always report guest MIPS and instructions per slice (`ce.slice_profile`),
  not time to a milestone.
- Investigate the 2026-07-04 silent desktop-run exit at 13:22:22Z (log ended
  mid-operation, no WER record; stderr was not captured for that run — the
  relaunched run captures both streams). The prior October
  `prog_new_default_b.log` also ended silently at 110.47 s; future launch
  wrappers must capture the process exit code as well as both streams.
- Continue tracing the real startup frontier after normal sensor samples and
  GPS reads. The prior callback crash, pre-run livelock, worker-identity loss,
  and sensor -15 causes are corrected and regression-tested. No forced app
  readiness, invented command-flag handshake, or painted map is allowed.
- Watch the SD image for re-created zero-byte probe artifacts during longer
  live runs (route/traffic phases); if any reappear, find and fix the
  emulator path that materializes files on read-only existence probes.
- COM7/MFS1/SMB1 opens and positive GPS ReadFile are now proven on the latest
  build; parsed-position handling and the final map presentation remain open.
- The hot guest PCs around `0x00339c00-0x00339da8` are confirmed pixel-format
  conversion work (RGB565/555 + alpha planes), not a readiness wait; they run
  at hookless speed now, so re-measure the skin decode before assuming it is
  still the boot bottleneck.
- Extend `SendMessageTimeout` and message-wait coverage for broader reentrant
  waits, nested sends, timeout cleanup, `ReplyMessage`, abort-if-hung behavior,
  and live modal-loop interaction.
- Validate `Shell_NotifyIcon` and `SHNotification*` in integrated runtime runs,
  including taskbar-visible state, callback dispatch through real guest COM
  object lifetimes, stale handles, and remove/update races.
- Keep mounted-storage work focused on physical block-driver forwarding,
  cache/filter DLL behavior, FMD callbacks, utility DLL exports, mounted-volume
  availability, file-security ACL persistence, and lower-FSD forwarding.
- Continue GDI/text/IME parity where CE source or raw tests justify it:
  palette/device-color behavior, alpha/mask variants, clipping, font fallback,
  IME UI callbacks, caret timing, and remaining DIB/bitmap lifetime edges.

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
