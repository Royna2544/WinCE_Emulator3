# Progress

Refreshed on 2026-10-14 (client KST date). This file records confirmed facts only.
Older investigation ledgers were purged; use commits, logs, and targeted source
references for historical reconstruction.

## 2026-10-13 Parallel Continuation

- Decision D-003 now keeps the Stage 2 CPU/VM contract and Stages 3-6
  Task/continuation/scheduler/clock migration on fresh-engine-per-slice
  lifetime. Persistent-owner/native-arena work is a parallel Stage 8 swap;
  its diagnostic successes cannot close fresh-engine acceptance. A bounded
  two-process persistent-owner control preserved distinct same-VA page values
  and F0 markers over six counted slices and four selected/parked swaps, with
  actual native timeout 0 (docs/evidence/ledger.md E-10). It used synthetic
  pages and explicit selection; Kernel VirtualAlloc, native wait/exit, long
  churn and app readiness were not proved.
- A separate persistent-owner full-backend diagnostic made progress through
  128 counted slices but lost guest F0; a 64-cycle map/unmap attempt failed in
  its first mapped phase. A short instrumented replay proved that the native
  context was captured with a pre-repair PC while the saved continuation used
  a post-repair PC, so the next exact restore missed and GPR-only fallback
  left F0 at zero (ledger E-12). Those pre-fix runs are Stage 8 diagnostic
  evidence, not a fresh-engine or app acceptance result. A subsequent
  deterministic fresh-owner repaired-PC control reproduced the loss as an
  expected red after its PC and HI/LO checks passed, then passed after the
  scalar context gained full 64-bit F0-F31 and FCSR with checked boundary
  capture/restore. Two checked-context FCSR controls also passed: unenabled
  sticky Cause/Flags round-tripped, while enabled pending Cause returned
  `UC_ERR_EXCEPTION`. Exact source/executable hashes and outputs are in local
  `evidence/e12-repaired-pc-20261013.receipt.md` (ledger E-13). This fixes the
  demonstrated repaired-PC scalar-fallback loss, not exact-key native Context
  transfer after owner closure, guest `VirtualAlloc` coherence, scheduler
  behavior or app readiness.
- Post-E-12 serial validation on the same dirty persistent-owner checkout
  passed the full release suite (**1,626 passed, 0 failed, 3 ignored**),
  Unicorn-only and Unicorn+desktop/no-trace release checks, and the no-feature
  suite (**1,239 passed, 0 failed, 1 ignored**). Format and diff checks passed.
  Exact commands, source/executable/log hashes and limits are in local
  `evidence/e12-repaired-pc-green-20261013/post-e12-validation.receipt.md`
  (ledger E-13). This is not fresh-engine default ownership, O-P1/O-P4/O-M1,
  Stage 2, bounded live-app or map/input acceptance.
- A separate ignored O-M1 control ran two real Unicorn MIPS slices with
  distinct file-mapping views injected at the same guest VA under two PIDs.
  It failed at the intended ownership assertion: all-view seeding gave the
  parent the child's `0x55667788` word (saved `T1=0x55667789` instead of
  `0x11223345`), and base-only readback applied the child's store to the
  parent mapping while leaving the child mapping unchanged. The exact
  observed/required tuple, source/binary/log hashes and raw output are in
  local `evidence/stage2-o-m1-same-va-20261013/expected-red.receipt.md`
  (ledger E-14). This is an injected-view native failure, not a public
  same-VA `MapViewOfFile`, private `VirtualAlloc`, fresh-owner or app result;
  it does not change the post-E-12 suite totals above.
- A later narrow O-P1 source slice replaced the successful
  `ServiceBoundary` residual with `UnknownNormalStop`, and one scheduler
  block-hook path now reports `HookStop(HostWallBudget)` only after a safe,
  checked PC write and successful stop request. Seven focused release controls
  passed on the first O-P1 source: actual MIPS host-budget stop, failed hook-control
  precedence, counted unknown boundary, native error, explicit PC stop,
  actual native deadline and unchanged service-handoff policy. The host-budget
  control observed `Query::TIMEOUT = 0`; the deadline control observed 1.
  Review then closed a source-level error-precedence hole: both decoded
  guest-exit branches now require no pending hook-control error. Two
  host-budget focused controls passed on that follow-up source; no real
  PC-write/stop failure plus encoded-exit injection was run. Source/binary
  hashes and raw logs are in local
  `evidence/op1-stop-taxonomy-20261013/focused-controls.receipt.md`
  and `evidence/op1-stop-taxonomy-20261013/encoded-exit-mask.receipt.md`
  (ledger E-15). Six other debug host-wall writers remain untyped, and native
  count exhaustion has no independent witness. The final follow-up source
  passed the full release suite (**1,628 passed, 0 failed, 4 ignored**); the
  fourth ignored test is the expected-red E-14 same-VA fixture. Exact raw log
  SHA-256 `8946D073…1E041` is in local
  `evidence/op1-stop-taxonomy-20261013/full-release-suite-final.log`.
  Unicorn-only and Unicorn+desktop/no-trace release checks also exited 0;
  the no-feature suite passed **1,239 passed, 0 failed, 1 ignored**. Their
  exact source/log hashes and paths are in local
  `evidence/op1-stop-taxonomy-20261013/post-op1-validation.receipt.md`
  (ledger E-15). This is the O-P1 source identity before later O-M1 edits;
  those edits require separate validation. Typed restart-PC repair,
  complete O-P1 classification, fresh-owner/VM Stage 2 and live acceptance
  remain open. The 1,626-pass suite above predates this slice.
- On the later O-M1 source, the E-14 injected two-PID same-VA native-view
  control was unignored and passed after the owner-qualified seeding and
  exact-view readback change. The final source also passed two closed-live
  readback/ambiguous-lazy controls and the outgoing-before-rotation control.
  Its full release suite passed **1,631 passed, 0 failed, 3 ignored**;
  Unicorn-only and Unicorn+desktop/no-trace release checks exited 0, and the
  no-feature suite passed **1,239 passed, 0 failed, 1 ignored**. Frozen
  `unicorn.rs` SHA-256 `0467E73E…F8D89`, exact logs and limits are in local
  `evidence/stage2-o-m1-same-va-20261013/owner-qualified-native-view.receipt.md`
  (ledger E-16). This narrows the historical E-14 failure; it does not prove
  public same-VA `MapViewOfFile`, lazy native fault repair, private guest
  `VirtualAlloc` bytes across fresh owner closure, O-P4, Stage 2 or live
  readiness. Immediate CreateProcess sync-error handling is source-reviewed
  but lacks end-to-end fault injection. Later source edits need new validation.
- A later narrow O-P1 slice added `HookStop(VisibleParkedPeerYield)` at one
  hookless live-pump block-hook path after a safe PC and successful checked
  PC write/stop request. The real-backend fixture passed three serial runs:
  a directly constructed visible parked peer triggered a native parent slice,
  typed yield, safe saved/native PC and handoff with `Query::TIMEOUT = 0` and
  no host-wall marker. Seven nearby focused controls also passed. Frozen
  `types.rs`/`unicorn.rs` SHA-256 `6EA882B5…D14DE` / `F1D45245…FD38` passed
  the full release suite (**1,632 passed, 0 failed, 3 ignored**), both
  no-trace release checks and the no-feature suite (**1,239 passed, 0 failed,
  1 ignored**). Exact commands, source/log hashes and limits are in local
  `evidence/op1-visible-peer-20261013/post-visible-peer-validation.receipt.md`
  (ledger E-17). This source follows E-16; the O-M1 green remains bound to
  its earlier frozen source. Other hook causes, actual count witness, typed
  restart-PC repair, API-failure injection, O-P4/O-M1 completion, Stage 2 and
  live acceptance remain open. A later source edit needs its own validation.
- An ignored E-18 private-page control on later test-only source ran five real
  MIPS instructions against an internal `MemorySystem::virtual_alloc` page:
  the first native owner held the guest-written `0x11223345`. A `cfg(test)`
  helper closed that idle owner while retaining the CPU snapshot; a one-
  instruction load on a genuinely fresh owner completed at the expected PC
  with normal stop/no fault but saw stale `0x11223344`. The focused release
  run exited 101 at the intended final byte assertion, not in setup or
  Context restore. Exact source/log hashes and raw output are in local
  `evidence/stage2-o-m1-private-fresh-20261013/expected-red.receipt.md`
  (ledger E-18). This demonstrates missing private-byte readback across the
  simulated fresh-owner boundary; it does not change production owner policy
  or exercise public COREDLL `VirtualAlloc`. The default release suite on
  this later test-only source passed **1,632 passed, 0 failed, 4 ignored**;
  E-18 is the fourth ignored control. Raw log SHA-256 `50310507…D622` is in
  local `evidence/stage2-o-m1-private-fresh-20261013/default-release-suite-final.log`.
  Unicorn-only and Unicorn+desktop/no-trace release checks exited 0, and the
  no-feature suite passed **1,239 passed, 0 failed, 1 ignored**; exact raw
  log hashes and source identity are in local
  `evidence/stage2-o-m1-private-fresh-20261013/post-e18-validation.receipt.md`
  (ledger E-18). E-17's separate 1,632/0/3 count is
  historical. None of these default/profile passes turns the explicitly run
  ignored private-byte failure green.
- A later narrow O-P1 global code-hook change records the first observed
  deferred stop reason: test-queued `RemoteControlService` or elapsed
  `HostWallBudget`. At each of three safe-PC completion paths it publishes
  `HookStop(reason)` only after checked PC write/stop; control failures take
  the backend-error route. Two real-backend focused controls passed together
  in three serial runs (2/0 each), with saved/native safe PC and actual
  `Query::TIMEOUT = 0`. Frozen `types.rs`/`unicorn.rs` SHA-256
  `00856084…80521` / `6560E1E3…A0F2` passed the full release suite
  (**1,634 passed, 0 failed, 4 ignored**); raw log SHA-256 `B56F3E54…CE30`
  is local `evidence/op1-global-code-hook-20261013/full-release-final.log`
  (ledger E-19). Both no-trace release checks exited 0 and the no-feature
  suite passed **1,239 passed, 0 failed, 1 ignored**. An explicit E-18 rerun
  on this source still failed at the intended stale-byte assertion, not as a
  normal-suite failure. Exact source/log hashes are in local
  `evidence/op1-global-code-hook-20261013/post-global-stop-validation.receipt.md`.
  The queued-input/straight-line controls do not prove public
  network delivery, every safe-PC shape or failing Unicorn API injection.
  Sparse/hookless remaining causes, native count witness, typed restart-PC
  repair, E-18 private red, O-P4/O-M1, Stage 2 and live acceptance stay open.
- Design decision D-008 selected one stable host-byte authority for each
  migrated committed page: AddressSpace-owned private slabs and canonical
  mapping-owned shared slabs, pinned through exact native owner leases. Local
  CE6 `NK/KERNEL/vm.c` confirms reserve/commit, decommit/release, protection
  and shared-heap operations are distinct; dumped-app use remains to measure.
  Independent source review at that design gate found no canonical birth record, per-handle
  copied process objects, global VA-only virtual allocations, startup native
  remapping that could bypass slabs, and CPU snapshot publication before some
  outgoing byte sync. The revised anchor/Stage 2 design requires one birth
  directory that Stage 3 extends, owner-qualified region/overlap and public Ex
  controls, pin-before-map/close-before-unpin, clone isolation, and an all-
  family checked slice commit. This is design/source review only: no O-M1
  implementation, fresh-default acceptance, new live run or performance gain.
- E-20 added two ignored ownership controls: last-public-handle child birth
  and public parent/child same-VA private `VirtualAlloc`; both initially failed
  at the intended boundary. E-21 introduced the canonical HandleTable birth
  record and process alias lifetime. The first control is now unignored and
  green with four added completion/termination/retirement controls (five
  focused 1/0 results). Final source passed **1,639 release tests, 0 failed,
  5 ignored**, **1,244 no-feature tests, 0 failed, 2 ignored**, and both
  no-trace release checks (ledger E-21; local
  `evidence/e21-process-birth-20261013.receipt.md`). This is a narrow public
  process-handle milestone; selected-process snapshots, wait pins, clone
  lineage, public same-VA VM, private bytes across fresh owner, Stage 2 and
  app acceptance remain open. No app run was made on E-21 source.
- E-22 added a metadata-only VM identity slice: the canonical process-birth
  directory supplies checked owner keys; successful virtual allocations carry
  a non-reused incarnation within one `MemorySystem` lineage and an explicit
  private/Ex-compat/file-view/locale/Toolhelp/shared-compat source kind. Four
  focused release controls passed (2/2, 1/1, 1/1). The later combined O-P1
  sparse-stop plus VM-metadata source passed the full release suite (**1,647
  passed, 0 failed, 5 ignored**) and no-feature suite (**1,248 passed,
  0 failed, 2 ignored**), with raw logs under
  `evidence/op1-sparse-stop-20261014/`. Explicit E-18 and E-20 reruns still
  failed at their intended boundaries: the fresh owner read stale
  `0x11223344` after a guest store of `0x11223345`, and the public child
  same-VA allocation returned `Handle(0)`. This metadata does not change
  global base-keyed VA, native bytes, reserve-only behavior, Ex targeting,
  clone lineage or O-P4 fresh-default lifetime. No Stage 2 or app acceptance
  is claimed (ledger E-22).
- The E-22 bounded live run reached its configured 330 s limit and the
  emulator exited 0 after 330.502 s; its monitor did not fire and no
  CPU-exception/PC-zero stderr text was reported. The 420 s observer saved
  four changed frames; its last observed and terminal frames remain iNavi
  splash rather than map. Sixteen newly constructed raw-NMEA posts and one
  tap were accepted by the remote API. Beyond those ACKs, retained traces
  prove positive guest `COM7:` reads of injected `$GPRMC` bytes and tap
  down/up window-procedure returns; parsed GPS fix and touch-driven map
  navigation remain unproven. The three original read/write mounts were
  ZIP-backed up beside their folders first.
  Four mounted entries changed only timestamps, and both affected files
  retained their length and SHA-256. See local
  `evidence/e22-pre-live-mounted-data-20261013.receipt.md` and
  `evidence/e22-live-20261013T154435Z/result.json` (ledger E-22). This is
  bounded survival, not Stage 2, memory-footprint or map acceptance.
- D-007 now requires this D: checkout as the only active source and one Cargo
  target. The validated Unicorn 2.1.5 dependency was copied in-repo (567/567
  matching file hashes) and selected via Cargo's path patch. The initial
  focused native count, nonzero unmapped-error and live pre-run rotation
  controls passed here. After outgoing file-view readback was ordered before
  PID rotation and stale typed outcomes were cleared on preflight swaps, the
  serial full release suite passed **1,623 tests, 0 failures, 3 ignored** in
  this dirty persistent-owner checkout before the E-12 repair above. The
  separately run ignored closed-owner fixture reached its intended final
  assertion and failed because the
  default still retains the native owner; its preceding guest store and native
  no-timeout checks passed. O-P1/O-P4/O-M1 and a bounded app run remain open;
  this intermediate checkpoint is not fresh-engine Stage 2 acceptance.
  Exact log/binary hashes and the expected-red receipt are in local
  `evidence/stage2-e7-stop-20261013/post-order-and-retirement.receipt.md`.
  The 243 immediate target-root run-evidence files and 28 release EXE/PDB
  files were separately copied and hash-verified under local `evidence/`
  receipts. After explicit approval, those 271 originals were
  reverified and removed by exact paths; nested target directories remain intact.
  The root-only `/evidence/` ignore rule keeps large local artifacts ignored
  while `docs/evidence/ledger.md` is Git-visible.
- After separate explicit approval, five inventoried old source/build copies
  were retired by exact path; the listed auxiliary Git worktree's branch was
  preserved. The selected unique source ZIP, exact fresh-HEAD red-test patch
  and selected EXE/PDB evidence were verified again after retirement under
  D: `evidence/`. Unselected compiler outputs can be rebuilt where source was
  retained, but are not recoverable from those archives. No other mirror,
  D: source/target/evidence or user SD data was removed by that operation.
- The user authorized Phase 1 after discussion, directed plan-first,
  implement-late work around a common anchor, and has now asked to begin the
  scheduler-core rewrite under its measured gates. `docs/ARCHITECTURE.md`, PLAN,
  index and 19 subsystem pages are published: 22 reviewed pages / 136 valid local
  links. Shared contracts are proposed; canonical task/frame/VM/clock migration
  remains gated. No production rewrite follows merely from creating docs.
- The earlier exact 64-file source freeze FCD9663B/native37B7724C passed
  1,618 tests, zero failures/one unavailable eVC4 fixture ignored, release
  build, both
  no-trace/stub checks, fmt/diff. Persistent engine/checked leases, primary
  CPU snapshots, mapping reconciliation and task-owned compatibility waits
  are in experimental source. Exact-scalar matching is not full in-slice
  task continuation authority; global hook removal invalidates translations.
- Its executable 65054E88...877F5 normal 330 s trial FAILED after 129.1081087 s,
  exit `0xC0000005` / no watchdog / empty stderr. Peak RSS/private were
  1,662.59 / 1,663.40 MiB. GUID-matched 650 symbols/disassembly locate fault
  RVA `0x85e757` at `tb_gen_code_mipsel` / `translate-all.c:1764`, a
  TranslationBlock jump-reset metadata write. Earlier `subpage_init` labeling
  was a false lead: renamed 650
  used adjacent generic BED2 PDB. The corrected site does not establish cause.
  Terminal PPM/full traces were not produced. Failed executable/PDB remain
  `scheduler_core_failed_1013.{exe,pdb}`; BED2 was hash-verified and restored
  at 02:30:01Z. Dirty source remains experimental; rebuilding is not BED2.
- Failed650 first splash is observed by+4.800s; last real frame+128.892s is
  still splash. Original5237 callerSPf578 restoration remains valid.16 GPS
  ACKs start atapp+109.108s (server readiness+16.075s, then93s delay); the
  only retained files-full snapshot has56 zero-byte COM7 reads, so positive
  consumption in that run is unproven. Tap failed after process death; no
  successful tap/lag evidence. Exact owned observers were stopped afterward.
  Artifacts:`target/bringup_1013_scheduler_core_drive_*`; detailed timed
  report is chat `validation/bringup-1013-scheduler-core-drive-evidence.md`.
- 1,791 completed primary native profiles record1,787 reused engines,
  1,472 exact contexts; actual expiry0/1 counts1,732/59. The host crash has
  no final native Result/profile. One early child error/IRQ26/PC0 requires
  explicit encoded-exit decoder provenance, not automatic suppression or
  guessed attribution. Diagnostic observations did not change runtime policy.
- Pinned64-bit Unicorn defaults to a1GiB translation arena per engine;
  Windows commits touched portions, and hook invalidation does not rewind
  its cursor. Full TB flush resets cursor but does not decommit. This is a
  source-confirmed retention mechanism, not measured allocation attribution
  or crash causality. A separately compiled Stage0 harness queried the actual
  initial arena as1,073,741,824B in six fresh processes. All six bounded
  stable/repeated-lease/hook/mapping/combined/in-block cases passed with
  checked native PC, counter and actual timeout result. At retained-owner
  phase, stable/lease/mapping sampled about7MiB RSS while the three hook
  churn cases sampled about243MiB; after owner close the latter fell to
  about5.7-5.8MiB RSS. This establishes bounded hook-associated retention
  in that isolated workload, not the allocation breakdown or cause of650,
  app throughput, full backend/FlatView safety or Stage2 acceptance. Native
  wait/exit FPU controls remain staged; later full-backend mapping and
  expected-red stop/error controls are reported below. Evidence:
  chat `validation/native-stress-1013/run-matrix-1` and
  `validation/20261013-stage0-native-stress-independent-analysis.md`.
- A separate pre-init arena differential used four fresh native processes.
  Actual queried sizes were8,388,608B for the8MiB request and
  1,073,741,824B for default; both passed a corrected four-instruction MIPS
  smoke with actual timeout0, RAM7 and owner close. Requested1MiB and2MiB
  controls exited0xC0000005 during the initializing query, before actual-size
  results. GUID-matched WER faults share RVA0xA2D31 in tcg_out8 code emission.
  The Windows4MiB commit handler has a source tail-clamp error consistent
  with this separate small-reservation failure; no fault address/dump proves
  the exact handler return or the650 cause. First controls had an incorrect
  next-PC assertion, corrected to the observed last-instruction PC0x100c
  before the clean rerun. Evidence: chat
  `validation/20261013-stage0-arena-initialization-differential.md`.
- A copied native dependency with a bounds-checked Windows commit request
  passed four fresh-process 1/2/8MiB/default pre-init MIPS controls; the
  otherwise identical pristine-handler copy still host-AVed on1/2MiB while
  its8MiB/default controls passed. In an8MiB distinct-PC translation probe,
  the repaired copy generated4,210,928B without TB flush, observed both
  sampled second-half arena pages change reserved to committed and translated
  another TB. The pristine handler host-AVed before that observation. This is
  isolated operational evidence, not direct handler fault/request tracing,
  production-cap safety, a650 cause or full-backend/app acceptance. A separate
  in-handler telemetry build stack-overflowed before its first1MiB query
  returned and is excluded from acceptance. Frozen sources, binaries, WER,
  receipts and limits: chat
  `validation/20261013-stage0-native-commit-tail-differential.md`.
- A follow-up isolated Win64 2x2 native-handler matrix held the bounded
  commit repair constant and varied post-return in-VEH counters and x64
  trampoline stack reservation (0x10/0x28). All four cells passed fresh
  1/2/8MiB/default smokes and the8MiB distinct-PC tail; four extra1MiB
  telemetry repetitions also passed (24/24 owned runs, no watchdog).
  Telemetry-on cells directly recorded a second in-range fault at arena
  offset4,194,328, remaining/request4,194,280B, successful VirtualAlloc,
  zero TB flush and later successful translation. This proves the isolated
  clipped tail path executed; it does not explain the earlier candidate2
  stack overflow, validate native unwind safety, justify a production cap,
  or establish full-backend/app stability. Frozen hashes/logs/receipts:
  chat `validation/native-tail-veh-abi-matrix-1013/REPORT.md`.
- A separate fifth isolated variant replaced the generated call-based Win64
  VEH trampoline with a leaf tail jump that leaves RSP untouched and sets
  only volatile RDX/RAX. Seven fresh-owned bounded runs passed: 1/2/8MiB,
  default, tail, plus1MiB/tail repeats. The8MiB tail reproduced the direct
  clipped-success counters and later TB without flush. The frozen source/
  executable/symbols and receipts are in chat
  `validation/native-tail-leaf-thunk-1013/README.md`. This is not a production
  dependency patch or a concurrency/teardown/full-backend/app acceptance.
- A separate synthetic final-partial-page control requested a logical
  4,194,305-byte arena (4MiB+1), reserved through the 4,198,400-byte host
  page boundary, and deliberately touched its final logical byte. Two
  fresh-owned bounded runs passed. Direct VEH counters showed the second
  in-range fault at offset4,194,304 with one byte remaining/requested and
  successful commitment; VirtualQuery showed the final and padding bytes
  committed, the first page beyond the rounded reservation still free.
  This is not a normal translator TB at the arena end or a concurrency,
  full-backend or production-cap test. Frozen evidence: chat
  `validation/native-tail-page-rounding-1013/README.md`.
- A separate leaf-thunk sequential VEH lifecycle control passed twice in
  fresh owned processes: each closed/recreated eight owner pairs, drove 16
  clipped second-half commits, verified successful handler removal and former
  closure/arena `MEM_FREE`, and continued translating in the surviving owner.
  Its final counters were 16/16 successful add/remove/free operations and
  32 in-range commits without failure. This is isolated single-threaded
  teardown evidence, not an in-flight callback race, full-backend or app
  acceptance. Chat `validation/native-tail-veh-lifecycle-1013/README.md`.
- A separate actual `UnicornMips` full-backend Stage2 control compiled in an
  isolated mirror and passed two owned release unit-test processes (first
  PID40728, 0.805s, exit0; both without watchdog). Across three
  counted20-instruction slices in each run,
  the same native engine reconciled a newly mapped 0x20000000 page, guest LW
  read value0x4f219a7b, guest MFC1 read preserved F0=0x6200, and a later
  removal left native page unmapped. Actual Query::TIMEOUT was0 at each
  boundary. This proves only this one-owner mapping/context route, not
  real Kernel VirtualAlloc, cross-task FPU, full stop/error classification,
  FlatView fragmentation or app acceptance. Fixture/hash/receipts: chat
  `validation/20261013-stage2-full-backend-mapping-stop-context-controls.md`.
- Two separate actual-backend stop/error controls compiled and executed in
  the isolated mirror as expected red. A positive configured timeout with
  count20 stopped at the correct PC/stores and Query::TIMEOUT=0, yet the
  backend fabricated a host-wall deadline. A nonzero unmapped guest LW
  retained a READ_UNMAPPED fault/nonzero PC, yet returned Ok under a positive
  configured timeout. Both one-test processes exited101 without watchdog;
  the mapping/context control passed again in the same combined test binary.
  This characterized the then-existing false timeout/native-error gate, not
  the later D: outcome correction recorded above. Receipts: chat
  `validation/stage2-full-backend-1013`.
- The eight staged real-Kernel Task/alias controls were independently reviewed,
  corrected for active-TID exclusion and alias-wake index coverage, then
  compiled/run in the source-hash-matching copy mirror. The exact release
  fixture exited test-code101 with one close-source preservation pass and
  seven expected ownership failures: copied alias priority/suspend/exit,
  creator PID, boot retention, primary identity after trace eviction and
  clone alias coherence. Some later assertions remain unreached after earlier
  failures; this is a baseline, not Task migration or live scheduler proof.
  Frozen fixture hash and logs: chat
  `validation/20261013-task-ownership-expected-red-baseline.md`.
- Two additional raw-COREDLL Task API controls compiled/run in the disposable
  mirror under the same Kernel/object/coredll source hashes. Both are expected
  red: OpenThread returned the existing handle256 rather than a new alias;
  GetProcessIdOfThread returned selected PID1 rather than child creation PID66
  after launch-queue drain. Their owned release test exited101 in0.583s
  without a watchdog. Together the separate eight- and two-test baselines
  cover one passing and nine failing controls; they do not implement or accept
  Task authority. Chat evidence:
  `validation/20261013-task-identity-api-expected-red-baseline.md`.
- Three further bounded no-Unicorn release controls (ledger E-11) failed at
  their intended Task lifetime/wake assertions: last public close lost a
  live task snapshot; exit via one alias missed a sibling-alias-indexed
  waiter; and an exited target vanished despite a registered wait after both
  public aliases closed. Together with the earlier ten controls, the Stage 3
  baseline is thirteen distinct controls, one pass and twelve expected red.
  No production Task code changed; chat receipt and fixture are
  `validation/20261013-task-pin-expected-red-receipt.md` and
  `validation/task_pin_expected_red_1013.rs`.
- Scoped cleanup archived/hash-verified and removed only two obsolete
  standalone test executable/PDB pairs and one duplicate cache-entry trace.
  Five files total9,522,161 logical bytes; recovery ZIP2,794,447bytes plus
  receipt are in chat `validation/archives/cleanup-20261013-small-intermediates.*`.
  Sources, active caches, unique evidence, synced references and SD data were
  untouched. This is recoverable cleanup, not physical-disk reclaim measurement.
- The diagnostic-only baseline4E0A/types89F9 passes1,596 tests, one eVC4
  fixture ignored, release/three feature/fmt/diff checks. Configured positive
  count and voluntary stops report actual expiry0; a genuine deadline reports1.
  DD2A886E...AD6A22 executable/symbols are preserved as
  `native_outcome_baseline_1013.{exe,pdb}`, not installed or live-run. Installed
  BED2 was restored after the failed scheduler ownership trial above.

- HeapReAlloc/LocalReAlloc/CRT realloc now map, copy the retained prefix and
  zero only the grown tail before committing allocation metadata. IME resize
  bypasses use the same boundary. Failure retains the old allocation; partly
  written destinations remain unused monotonic holes, not silently reused.
  Raw mapping/read/copy/zero failures and actual MIPS fresh-engine spillover
  reads pass. Checked initial allocation is now validated below; heap reuse
  and page reclamation remain separate unfinished work.
- The normal A3BA00D6...B3F5A6 bundle passes 1,536 tests, one eVC4 fixture
  ignored, release build and three feature checks. Its actual 330 s GPS/tap
  run ends at 330.239 s, exit 0/no watchdog/empty stderr. Sampled peak RSS is
  566.0 MiB/private 568.0 MiB: essentially the preceding 562.3/573.2 MiB,
  not a new late-memory improvement. First splash is observed by +4.631 s
  (2 s polling; not full boot). 206 location responses and retained positive
  COM7 ReadFile payloads prove input consumption. Tap delivery is FIFO but
  takes 1.578 s; both handlers return in about 22 ms. All observed frames and
  the terminal PPM are splash, not map. Four SD probe markers remain absent.
  Artifacts: `target/bringup_1013_heap_drive_*`; detailed report is in the
  chat workspace `validation/bringup-1013-heap-drive-evidence.md`.
- That run executes 1,037,557,247 guest instructions across 3,406 slices;
  weighted busy-slice MIPS is 12.846 versus preceding 12.314. Guest work and
  late phase rates are comparable; no controlled speed improvement is claimed.
- Explicit nonzero PC stops now preserve native registers, thread/process,
  pending callback state and ownership before ordinary thunk/callback repairs
  or handoff. Import traps independently honor the stop before dispatch;
  PC zero remains an error. CLI/monitor continuation and trace selection require
  the fresh matching stop and retained context, not a historical snapshot.
  Real MIPS thunk/trap/nested-callback/ready-peer regressions pass. The isolated
  debugger build DE5AEDB1...858A98 passes all 1,540 tests, one fixture ignored,
  release build and both no-trace plus stub checks. Logs:
  `target/bringup_1013_pcstop_trace_{tests,build,check_ci,check_unicorn,check_stub}.log`.
  Earlier installed executable/symbols remain in `pre_exact_pc_1013.{exe,pdb}`.
- Read-only binary evidence identifies the real splash manager show/destroy
  methods at 0x5e7a0/0x5e880 and WM_INITDIALOG startup at 0x2bc34. The first
  large initializer is 0x30e1c, return 0x2bde4. Pool worker 5 starts at
  0xe6cd0, not the unproven engine wrapper. The first probed cache miss returns
  a valid node; that does not establish whole-job completion. Current exact
  entry/return investigation must retain actual PID/TID, SP and controller
  identity. No completion event or app state is forced.
- Source audit finds direct visible callbacks bypass the deferred typed
  message-payload serializer. It also finds NotifyIcon output writes beyond
  its 152-byte allocation. The target CE6 coredll ordinal 481 explicitly
  marshals 0x98 bytes, so enlarging to 160 is not a CE6 fix. Scoped shared
  payload/writer corrections now pass seven focused MIPS/raw-equivalence tests,
  a notification neighbor-sentinel test, all 1,547 tests (one fixture ignored),
  release build and three feature checks. Payload-only executable is
  26FD49C5...D7ABFD in the mirror; it is not installed or live-run evidence.
  Logs: `target/bringup_1013_payload_fixed_*`. Its sparse persisted heap pages
  now use a contiguous mapping frontier so lower gaps are not skipped.
  The first fixture run hung on unvalidated synthetic paint; only its owned
  test process was stopped, and bounded validating drains fix that test setup.
- The exact debugger subsequently proves MapInit enters and returns normally
  on UI thread 1 with V0=1. Its actual stack is 0x7ffdf558, 0x20 below the
  original 0x5237 caller's 0x7ffdf578. Historical WNDPROC fallback discarded
  saved SP/GPR/HI/LO and restored only PC/RA/V0. The authentic startup saved
  RA is 0x2bad0 at 0x7ffdf664; the shifted epilogue slot instead contains 5.
  Authentic archived SP/GPR/HI/LO restoration now passes native callback and
  real caller-epilogue regressions, including K0/K1. Missing/ambiguous ownership
  fails closed; native write/readback must succeed before consuming history.
  The diagnostic is not a timing benchmark (manual pauses/global hooks and
  simultaneous fixture spin), and ended via explicit monitor quit, exit 0.
- The integrated installed BC2E13E0...954555 executable passes 1,552 tests,
  one eVC4 fixture ignored, release build and three feature checks. Logs:
  `target/bringup_1013_archive_fixed_*`; previous debugger outputs are retained
  in `pre_callback_archive_1013.{exe,pdb}`. Later allocation/message source
  edits are in progress and are NOT covered by that frozen validation.
- Its normal 330 s GPS/tap run exits 0 at 330.236 s, without watchdog or
  stderr. Actual 0x5237 callback recovery restores UI thread 1's original
  0x7ffdf578 stack at +2.796 s, with result 1. No old PC-only fallback or
  CPU-error marker is observed. Splash appears by +4.683 s, COM7 by +27.293 s,
  and every later sampled/terminal frame remains genuine splash, not map.
  199 GPS responses plus retained positive COM7 payloads prove delivery;
  they do not prove parsed navigation position. Tap DOWN then UP return
  cleanly, with 0.835 s initial delivery lag and about 22 ms per handler.
  Peak RSS/private are 492.6/503.2 MiB, versus prior A3's 566.0/568.0 MiB.
  This combined, non-cache-controlled run is not an isolated latency/memory
  benchmark. Four SD probe markers remain absent. Artifacts:
  `target/bringup_1013_callback_drive_*`; full timed evidence is in the chat
  workspace `validation/bringup-1013-callback-drive-evidence.md`.
- That run executes 1,434,175,567 guest instructions in 3,211 slices at
  weighted busy MIPS 14.071. iNavi contributes 1,002,008,727 at 13.140 MIPS;
  most extra aggregate work is happyway (432,073,151 versus prior 2,072,966),
  not a pure speed improvement or proof of startup completion.
- Binary audit identifies the second startup initializer at 0x31970, return
  0x2be44. It creates the pool and dispatches jobs, without a top-level wait.
  Owner+0x39dd4 is the concrete MDC object (vtable 0x4e57dc, method 0x12e15c),
  which invokes engine virtual +0x14c then can sleep/wait and repeat. A finite
  cache lookup is not whole-job completion. Actual matching invocation/return
  and current object reads are the next diagnostic boundary, not forced events.
- Historical copy-mirror validation also needed changed source mtimes updated:
  Copy-Item could preserve a timestamp older than an existing library artifact.
  Exact hashes plus a fresh build were required. This is an archived workflow,
  superseded by D-007's single D: source checkout; do not create or stamp new
  copy mirrors.
- Checked initial guest allocation now reserves without live metadata, maps
  writable persistent RAM, explicitly zeros requested physical extent, runs
  checked initializers, then commits/publishes. All 36 immediate raw/native
  callers route through it; four host-queued typed reservations materialize
  at delivery. Logical-zero allocations still have one physical byte. Failure
  retains monotonic holes; no address reuse/page reclamation is introduced.
  CREATESTRUCT failure now tears down only the failed new window and returns
  zero, not raw HWND success; scratch lives through both create phases only.
  Clipboard/IME/array/cache publication and cleanup are checked without an
  invented rollback guarantee for already-live partial outputs.
- Native current/blocked GetMessage and SetWindowPos delivery now prepares
  known typed payloads before queue consumption/callback exposure. Sent
  messages use exact send identity across expiry; MSG output requires all
  28 bytes mapped RW. Unknown guest pointers stay untouched. Legacy wrappers
  lacking mapping context defer known payloads for a mapped successor; they
  are not claimed fully mapped. Failure preserves wait/FIFO/context; SetPos
  returns handled-zero rather than repeating an already-applied operation.
- Exact frozen source passes 17 focused tests and all 1,571 tests (one eVC4
  fixture ignored), release build, two no-trace checks and stub check, fmt and
  diff checks. Installed SHA256 354AA0FB...2E4327; prior BC executable/symbols
  are preserved as `pre_boundaries_1013.{exe,pdb}`. Logs:
  `target/bringup_1013_boundaries_fixed_*`. The initial compile-only Prot API
  mismatch log is preserved separately; no failing build was installed.
  Its requested 330 s GPS/tap/frame run `bringup_1013_boundaries_drive` FAILED
  early at 260.174 s, exit 1/no watchdog. PC becomes zero on pool worker 10,
  RA0x000e6de4/SP0x7feff1c8 after recorded ready-event wait resume (id4029,
  handle0xff4, result0). Suspended worker5 retains PC0x00321c98. Do not suppress
  this CPU error or call it a normal timeout; causality of the combined changes
  is not yet established. Unit/source acceptance does not establish live stability.
  Native restored continuation versus JALR target must be distinguished.
- That failed run still observes genuine splash by +3.411 s, COM7 by +27.877 s,
  actual positive GPS ReadFile, and FIFO tap DOWN/UP with clean returns. DOWN
  delay is 1.489 s, handlers24.373/22.080 ms; no map presentation. Peak sampled
  RSS/private509.3/520.6 MiB; the late cleanup dip is not a memory improvement.
  Workload differs (iSearch rather than happyway executes much more work), and
  duration is shorter than BC; no controlled performance claim is made.
  Full outcome/evidence report is in chat validation; executable354 remains
  identifiable and prior BC backup is preserved for explicit comparisons.
- Frozen BC exact diagnostics independently reach second-init 0x31970 on
  TID1/SP0x7ffdf578/RA0x2be44 and MDC engine return 0x12e180 on TID5, V0=1.
  Live object/vtable and saved-frame chain match the binary job binding.
  Current MDC body SP is 0x7ff9f1b0, not the older candidate 0x7ff9f190.
  Missed later stops in bounded monitor chunks do not prove lost continuation;
  remote debug publication is stale during interactive continuation. The
  owned monitor quit normally; terminal image is still genuine splash.

- Bounded exception diagnostics and six genuine native handoff controls pass
  1,580 tests (one unavailable eVC4 fixture ignored), release and three feature
  checks, fmt/diff. Freeze A206/89F9 builds executable528F776B...EC705B;
  it was not installed or live-run. Checked current GPR/IAT values are not
  claimed historical cause. Logs: `target/bringup_1013_fault_diag_*`.
- Two additional native caller-helper fixtures then reproduce generic defects:
  native delay block0xb000 normalized to source0x1100 causes interrupt20/PC0
  despite a valid IAT; completed-delay block0xc000 resumes its recipient cleanly
  but source replay increments callerV0 twice (0x105 rather than0x103).
  Separate failure logs are preserved. These do not yet prove the actual
  wait4029 app failure traversed that boundary.
- A minimal entry-only helper guard now defers interior scheduling instead of
  treating the original source PC as a saved continuation. Frozen Unicorn
  5152C897...A17CE5/types89F9 adds true/false native branch, entry, deferred
  replay and direct predicate controls; independent source review accepts it.
  Exact freeze passes 1,585 tests, one unavailable fixture ignored, release,
  three feature checks and fmt/diff. Installed12DFB7B4...0CAFF5 after all62
  Rust files matched; prior354 executable/symbols are preserved as
  `pre_safe_handoff_1013.{exe,pdb}`. Normal330s GPS/tap run completes330.268s,
  exit0/no watchdog/empty stderr, beyond the prior failure window. Authentic
  original5237 stack restoration and actual GPS payload transfer continue.
  This is bounded stability, not exact attribution of every prior live crash.
- Latest12DF run accepts208 GPS updates; retained ring proves148 positive
  COM7 reads/34,800B. Tap DOWN/UP are FIFO with0.777s first-delivery lag and
  22.628/22.575ms handlers. All observed/terminal frames remain splash, exactly
  matching BC. Capture started19.706s late, so first-splash timing is unavailable.
  PeakRSS/private556.031/571.316MiB; helper workload differs sharply from BC
  and354, and an extra GET-only memory observer was added. No controlled gain
  is claimed. Fullguest1,015,592,266/iNavi1,013,426,929 instructions;
  busy-weightedMIPS13.250/13.231. Logs:`target/bringup_1013_safe_handoff_drive_*`.
- Saved helper backing observations are77/78MiB RAM, while separately retained
  global live heap is30.085MiB. Ages and cached non-atomic PID brackets prevent
  exact simultaneous ownership or RSS attribution; capacity/native bytes and
  per-PID live heap are unavailable. No speculative reuse/reclamation landed.
- A subsequent native raw-yield/fresh-engine audit reproduces silent wrong-path
  execution for a not-taken branch at page-delay0xb000:3 controls pass,1 fails,
  sourceA676. Separately, native ctl_get_timeout returns configured duration,
  not expiry; production at that point used it as an expiry flag. A query-only
  correction was held because count-tick rotation/cleanup/restart depended on
  the false wall flag; the later connected D: correction is recorded above.
  Four counted replay tests over64 native boundaries
  pass; they do not cover full backend cleanup/process rotation.
  No new states/priorities/hooks or CPU-error suppression were introduced.
- The unsafe raw-yield fallback is now removed: its existing non-import and
  no-pending-return gates imply contextNone, so it never served a safe boundary.
  The earlier safe eligible-peer policy is preserved in a shared decision.
  Four genuine native both-outcome tests now defer actual helper interiors,
  execute the selected branch unaided, and freshly replay a safe destination
  with exactly-once effects. ExactEE18/89F9 freeze passes1,593 tests, one fixture
  ignored, release/three feature/fmt/diff checks; installedBED2F559...276E4F
  after all62 Rust files match. Prior12DF executable/symbols are preserved in
  `pre_safe_visible_yield_1013.{exe,pdb}`. No commit/push.
- BED2 normal330s run completes330.234s, exit0/no watchdog/empty stderr.
  Prelaunch frames observe genuine splash by4.358s, not full boot; a separate
  overlapping tail covers the final frame without extending the first watcher.
  Original5237SPf578 restoration remains intact.208GPS responses and a retained
  146-read/35,148B COM7 ring prove payload transfer; tap FIFO first callback
  takes0.397s, handlers22.746/22.898ms. All frames remain splash, not map.
  PeakRSS/private555.633/568.383MiB is essentially12DF556.031/571.316, not a
  memory gain. Profileguest1,009,828,063/iNavi1,007,664,687; busyMIPS13.126/13.107.
  Extra observers and variable scheduling prevent controlled performance claims.
  Artifacts:`target/bringup_1013_safe_visible_yield_drive_*`; detailed chat report
  `validation/bringup-1013-safe-visible-yield-drive-evidence.md`.
- Full terminal trace-all exposes no fault/probe marker only in its selected
  retained helper67 GetMessage snapshot, not all native attempts. Summary emits
  optional interrupt/invalid markers but omits memory faults; raw nativeResult
  and actual expiry were absent from that BED2 profile. The later diagnostic
  freeze logs both without changing timeout/error/rotation decisions. Full map and
  authoritative heap PID/reserved-footprint/incarnation remain unfinished.

## 2026-10-12 Parallel Continuation (baseline)

The bounded-file baseline: 1,527 tests passed, one eVC4 fixture ignored; both
no-trace Unicorn release checks pass. Exact mirror validation logs are
`target/bringup_1012_bounded_files_{tests,build,check_ci,check_unicorn}.log`.
That executable SHA256 is `C274BEE2...ADD1D1`; no commit/push was made.
The prior executable/symbols remain in `pre_bounded_files_1012.{exe,pdb}`.

- Corrected callback lifetime without expanding the stack grace: retain an
  outer direct send while a later same-thread, lower-stack callback is live.
  Real MIPS regressions verify nested return order and saved SP/S0/FP.
- Removed historical child-lifetime traces from pending-CreateProcess
  scheduling and preserve the selected slice-end rotation through run entry.
  The loop-head wall-limit check also bounds paths that execute no slice.
  A 600 s mounted run exited 0 at 600.45 s without an external watchdog.
- Decoded the YAS526B command/selector protocol and Yamaha B/C packed samples.
  The deterministic ideal sensor models calibration, rough/normal commands,
  bank ordering, packed 11-bit values, and coordinate signs. Nine focused
  sample/contract tests pass. The former invalid rough payload caused -15;
  no such modal appeared throughout the corrected 600 s run.
- Empty synchronous serial reads no longer add virtual time after spending
  the timeout in real host sleep. The focused tick/host-time regression
  passes. The broader hybrid clock remains an architectural gap.
- Bare visible-window thread relabeling is now restricted to the actual
  terminal thread stub, preserving worker contexts. The 300 s profile shows
  thread 6 continuing real slices and a successful COM7 ReadFile of 174
  injected NMEA bytes (not merely an empty host queue). It exited 0 at
  300.773 s with empty stderr and no PC-zero/guest-exception marker.
- Inactive/unmapped callback receivers are checked before removing a queued
  message. DOWN/UP FIFO and filtered non-head regressions pass. The subsequent
  330 s actual-input run delivered DOWN then UP through the real MFC wndproc,
  with clean returns: tap response 13:53:05.483680Z, DOWN push 06.771140Z,
  DOWN return 06.793756Z, UP push 06.861065Z, UP return 06.883623Z.
  About 1.29 s to first guest delivery is still a responsiveness issue.
- That run accepted 184 stationary location updates; the final retained file
  ring proves 124 positive COM7 reads totaling 28,536 B. Rings are incomplete
  history, not total injected/consumed bytes. Exit was 0 at 330.221 s, no
  watchdog or stderr, but no map transition. Sampled peak RSS was
  1,362,944,000 B/private 1,377,103,872 B. Boot2 frame timestamps were batch
  stamps, so its ~2.5-4.7 s splash bracket is approximate, not exact timing.
- Read-only ordinary files now use the existing bounded 64 KiB HostFile
  window rather than eager whole-file retention, with no path special cases
  or arbitrary RAM cap. Four new regressions plus converted cache tests cover
  byte/seek/EOF, duplicates, streaming, flush/truncation/reopen and closed-file
  lifetime; 22 file unit tests and all 136 raw memory/file tests pass. Writable
  private-memory promotion is intentionally unchanged.
- The bounded-window executable completed the matched 330 s live run in
  330.259 s, exit 0/no watchdog/empty stderr/no CPU-error markers. Sampled
  peak RSS is 589,615,104 B (562.3 MiB), private 601,038,848 B (573.2 MiB),
  versus the prior 330 s run's RSS 1,362,944,000 B (1,299.8 MiB): about 57%
  lower peak RSS. Phase-aligned samples are also lower. This is a bounded
  observational comparison, not an OS-cache-controlled benchmark or full
  allocation attribution; SD user data and host file cache were not reset.
- New per-request clocks observe black at +2.605 s and splash at +4.837 s
  (2 s polling cadence). COM7 opens at +27.897 s, MFS1 at +30.672 s, and
  actual GPS payload reads continue after sustained input. HTTP accepted 204
  location updates. Tap delivery again preserves DOWN then UP, but response
  to first guest callback is 1.617 s; both guest handlers return in ~22 ms.
  Pixels remain splash, with no map transition at the 330 s limit.
  Logs/result/memory/frame: `target/bringup_1012_bounded_files_drive_*`.
  The four SD zero-byte probe markers remain absent.
- Full bring-up is NOT complete: these runs remain on the genuine splash,
  without a map/interactive navigation transition. The 300 s run first
  opened COM7 at 30.936 s, MFS1 at 34.338 s, and SMB1 at 34.578 s. Its frame
  capture began late (+23.171 s), so that run cannot establish first-splash
  latency. Report: chat-workspace `validation/bringup-1012-boot-profile-evidence.md`.
- That profile sampled host working set up to 1,244,803,072 B (1,187.1 MiB)
  and private bytes up to 1,253,953,536 B (1,195.9 MiB). This is host memory,
  not CE guest heap or virtual reservation, and is not yet proof of a leak.
  Eager whole-file storage caching is an evidenced contributor candidate:
  31 reached read-only files total 263.30 MiB; all 44 mid-size candidates
  total 428.62 MiB, with incomplete access/ownership history. The bounded
  windows now materially lower live RSS as measured above. Remaining guest
  pages/native engine/trace capacity and any writable duplication need
  separate ownership measurements; do not infer a kernel-clone leak.
- Read-only residual-memory audit finds that ordinary process switches move
  CPU/blob owners; full cpu/kernel clones are optional monitor checkpoints,
  not this run. The allocator has a global monotonic heap cursor, and each
  process's `map_heap_spillover` persists pages up to that same shared high
  water mark. In the prior run, persisted blobs grew to roughly 93/81/81 MiB
  for PIDs 1/67/68 while live heap was about 31.6 MB. HeapFree removes live
  metadata but neither reuses addresses nor shrinks spillover backing.
  Measure per-process blob length/capacity and CE heap-reuse semantics before
  reclaiming pages or changing allocator ownership; no speculative fix landed.

Prior reproductions (fixed causes, retained for comparison):

- Normal remote GPS input exposed another control-flow failure in the prior
  validated executable (SHA256 `AF4CA1A7...E687DC2`): the 300 s input run
  exited 1 at 86.90 s, not by watchdog. Location `37.5,127.25` was accepted
  as three NMEA sentences; no touch was sent. Final PC/RA were zero and the
  emulator correctly reported a guest CPU exception. Before it, an outer
  SendMessageW callback was discarded while a nested visible callback still
  ran, then recovered from a historical FP-only record. Fixed by preserving
  the live callback ancestry above; the new actual-GPS run survives.
  Logs/frame/result: `target/bringup_1012_modal_drive_*`.
- A 600 s default-count live run reached a genuine Korean G-sensor error
  dialog (`Error Code: -15`) over the splash. The dialog says navigation can
  continue with GPS and offers OK. `COM7:`, `MFS1:`, and `SMB1:` opened;
  actual serial reads and sensor IOCTLs are present. No input was consumed
  before the run ended. Frame and timestamp/hash snapshots are under the
  chat workspace's `validation/captures/remote-20261012-121948Z`.
- That run stopped executing slices at 12:18:35Z but kept switching cached
  helper contexts and consuming a host core. PID67 was genuinely runnable
  (`can_run=true`, no ready wait/blocked send, PC `0x0001b4a0`). Historical
  `CreateProcessChildParked` trace records were still treated as unreturned
  parent API calls until child exit, reversing scheduler choices. Pre-run
  `continue` paths also bypassed the CLI wall limit. The external watchdog
  stopped PID6268 at 625.78 s, exit -1; this was not a clean guest completion.
  Log: `target/bringup_1012_default_stdout.log` (stderr empty).
- Disproved the old "command-flag readiness loop" lead: the flags
  0x4000/0x2000/0x100/0x80 belong to libpng allocation cleanup, not a
  splash-to-map handshake. `png_free_data` is VA `0x00308298`, image RVA
  `+0x2f8298`; the old `+0x2f7298` was a .text-relative disassembly offset.
  `png_sig_cmp` at VA `0x00307e74` and the PNG signature confirm the region.
- The growing 5/10/15/20 s sleeps are sensor initialization retries, not an
  unidentified helper/network wait. Sleep caller VA `0x000e69b8`, return
  `0x000e69c0`, multiplies object retry count by 5000 after failure of
  `0x000e6754`. The sensor read validation returns -15 after invalid
  rough-offset status; the command/packed-sample correction above closes
  this demonstrated failure. Do not interpret normal 50 ms sensor pacing
  arithmetic as a fresh SensorError merely because V0 was briefly negative.
- Fixed live-pump `CreateProcessW` replay: the child previously executed inside
  the parent import hook before its return registers/PC were committed. A child
  consuming the parent's native timeout left the parent on the completed import.
  Successful live launches now commit the return, stop the parent, and drain
  children after the outer `emu_start` and timeout check. Non-live execution
  retains its synchronous launch path.
- Fresh count=0 / native-timeout=100 ms reproduction dispatched the same helper
  launch at RA `0x00059ba0` 16 times, producing PIDs 67-82. The fixed 120 s run
  dispatched that site once, launched three distinct children (66-68), executed
  735,491,203 guest instructions, and exited normally at the requested limit.
  Logs: `target/parallel_1012_{baseline,fixed}_stdout.log`. The framebuffer is
  still the real splash; this does not establish map/route completion.
  A lower-noise 180 s repeat also exited 0 with only PIDs 1/66/67/68, executed
  1,280,494,534 instructions, and measured 84.56 busy-slice MIPS for iNavi.
  The helper received only one slice; fairness remains open. No explicit device
  open evidence was captured. The four zero-byte probe markers remain absent.
  Repeat artifacts: `target/parallel_1012_repeat_{stdout.log,frame.png}`.
- Corrected scheduler block sampling: prior TB starts were mistaken for prior
  instructions, rejecting ordinary branch targets/fallthrough blocks. The block
  path now checks the word at PC-4 for a possible delay slot. Separate block and
  instruction stop helpers preserve the sparse/global hook guards. A real
  Unicorn count=0 test crosses a page-end branch, executes its delay slot once,
  and stops before the target instruction.
- Explicit global-hook requests and arbitrary-PC diagnostics now override
  legacy sparse flags. Non-trace selector bindings are covered by feature checks.
- Exact source copies in `C:\Temp\wce-1012-parallel` passed 1,511 tests: lib 582,
  bin 28, basic 130, dispatch 10, GWE 495, kernel 125, memory/file 136, waveOut 3,
  host helpers 2. The eVC4 fixture runner is ignored because the MIPS compiler is
  unavailable. `cargo check --release` passes for `unicorn,win32-desktop` and
  `unicorn` without trace. Validation log: `target/parallel_1012_tests.log`.
- At the time, direct repository builds hit Unicorn's Windows symlink privilege
  error. The temporary mirror changed only its local dependency build to copy
  the archive and watch the correct source paths; global registry/system settings
  were not changed. The later in-repo vendor patch and focused D: build are
  recorded above. The 1M-instruction default remains while scheduling fairness
  and native-timeout branch-state preservation are validated. The then-validated
  executable/symbols were installed under `target/release/`; their later
  preservation and exact-path cleanup are recorded above.

## 2026-10-12 Session Facts (guest throughput)

Measured with `ce.slice_profile` (now logs `blocks`, `guest_insns`, `mips`;
`fc8c8384`) on 2-minute live runs, `--desktop virtual`, clean SD image, same
host. A bare Unicorn MIPS loop on this host does ~3200 MIPS unhooked, ~1900 with
a trivial block hook and ~400 with a trivial per-instruction hook, so hooks are
only slow when their body is.

- Root cause of the ~1 MIPS iNavi speed: the scheduler timeslice BLOCK hook in
  `run_with_unicorn` ran kernel/GWE/parked-process queries (visible receiver
  work, remote-control mutex, `Instant::elapsed`) on every translated block,
  about 15 us per block for iNavi (hundreds of hidden MFC child windows;
  `happyway_win` with none ran ~10x faster per block). Polling now only runs
  at amortized service points (every 1024 blocks, at most once per ms).
- The July "hookless A/B shows no gain" conclusion was a false lead: both arms
  kept that block hook. iNavi guest MIPS by variant: HEAD 1.05 average (4.6 in
  30-300 ms slices); amortized hook with the global code hook 7.3 average
  (12.9 busy); amortized hook plus hookless trampoline mode (now the default,
  `WINCE_EMU_GLOBAL_CODE_HOOK=1` restores the global hook) 22-34 busy versus
  13 busy with the global hook, ~6.5x more guest instructions per wall second
  than HEAD.
- The July "per-slice engine rebuild is 1.5%" figure only held for ~300 ms
  slices. Fixed per-slice cost (setup + hooks + snapshot + sync_out, 8-15 ms)
  was 3.6% / 14.9% / 25.5% of slice time at 302 / 143 / 65 ms average slices,
  and the cold translation cache each slice is not counted in it.
- Live-pump slices pass `instruction_limit = 1_000_000` to `emu_start`, which
  makes Unicorn install its own per-instruction counting hook and is the de
  facto round-robin tick (95% of slices in the fast runs ended at the count).
  `WINCE_EMU_LIVE_PUMP_RUN_SLICE_INSTRUCTIONS=0` removes it: iNavi busy slices
  reach 30-106 MIPS, but scheduling dynamics change (see KNOWN_BUGS.md), so
  the default stays 1M until the scheduler has a real time-based quantum.
- After the compute speedup wall time is dominated by guest-level waits, not
  compute: with unbounded slices iNavi runs ~600M instructions in ~20 s, then
  sits in waits of 15, 20, 25, 30 s (5 s growing steps) with no slices at all
  (104 of 127 s wall idle in one run); with the 1M count, 1 s slices starting
  at an import trap with ~2-6k guest instructions fill the same role. There is
  no centralized virtual scheduler clock. `TimerSystem` combines boot wall
  time with a sleep-driven virtual offset; coherent clock semantics and idle
  fast-forward remain unproven.
- Run-to-run scheduling is bimodal for the same build: `happyway_win` either
  gets 170-300 slices (fair round robin) or 1-7 slices (starved while iNavi
  runs). Single A/B runs of scheduling-sensitive metrics are therefore weak
  evidence; compare busy-slice MIPS and total guest instructions instead.
- Remote status/live snapshots are serviced in the main loop, including
  between rotations; framebuffer publication normally follows slices or
  scheduler changes. Import/file counters can be retained per-process CPU
  snapshots. Alternating frozen snapshots are not progress. Record capture
  timestamps and body/frame hashes alongside slice logs.
- The Jul-2026 release binary could not be rebuilt in an agent shell after the
  rustc 1.98.1 update: `unicorn-engine-sys` needs `LIBCLANG_PATH` and cmake
  symlink privilege (see AGENTS.md). Validation was run from a mirror build
  with the symlink step replaced by a copy: lib 576, bin 28, basic_subsystems
  130, coredll_dispatch 10, coredll_raw_gwe 495, coredll_raw_kernel 125,
  coredll_raw_memory_file 136, waveout 3, host_progs 2 all pass (the mirror
  needs `registry.reg`, `mounts.toml`, `serial_devices.json`, `host_progs/` in
  its root).

## 2026-07-04 Session Facts

- Map-gate root cause: escaped-callout recovery false-positived on the
  pre-push stop snapshot (a cross-thread visible-message callout copies its
  return pc from that same stop), so every host-delivered WM_SHOWWINDOW /
  WM_WINDOWPOSCHANGED / WM_ACTIVATE / WM_PAINT was consumed host-side ~1 ms
  after push without running the guest wndproc (`ce.gwe=debug` showed
  back-to-back push/escape pairs; 115+ per run). Fixed by gating recovery on
  a run slice completing after the newest push (`2fcc365f`); the same commit
  keeps staged callouts ahead of the run-entry ready-parked-wait rotation,
  which otherwise ping-pongs processes every ~2.6 ms without executing the
  staged slice.
- Quantum rotation is now unconditional at wall-clock quantum expiry
  (`dbaf4791`): the previous empty-receiver-queue gate let `happyway_win`
  busy-wait inside its WM_ACTIVATE handler (pc cycling `+0x28f8..0x2a18`,
  heap/message counters frozen) with one queued message it could not consume,
  pinning the CPU while iNavi's 43-message layout cascade starved.
- With real visible-message delivery, startup takes a different shape: the
  afxwnd42u children are created through actual dispatched messages, the
  splash progress bar advances to full width, and the run proceeds into
  `res\resmapi_800x480.bin` map-skin streaming (past the old
  InaviMainConfig.bin plateau). Post-fix runs show 0 escapes and 90+ genuine
  WNDPROC stub completions.
- Rotation churn could still starve a freshly staged visible callout (one
  hot core of silent main-loop rotation, no slices); all parked-process
  rotations now refuse to switch while a staged callout has not yet received
  a run slice (`265e751a`). With that veto, iNavi's 43-message child cascade
  drains completely and the skin decode advances past the prior stop
  (`resmapi` last_pos 1343058 -> 1415371+).
- The previously labeled command-flag region is libpng cleanup (corrected
  by the 2026-10-12 disassembly above), not a readiness condition.
  `happyway_win` runs a designed ~1 s retry cycle inside its wndproc: it
  polls `FindWindowW(NULL, "utis_parser")` (no such binary exists on the
  device image, so the poll fails forever on real hardware too), then
  self-posts msg 0x405 and Sleeps 1000 ms. Its permanently queued
  WM_ACTIVATE plus that cycle made every "pending message = priority"
  scheduler heuristic hand it the CPU.
- Scheduler fairness completed in three further fixes: priority parked
  handoff now requires a genuine wake event — completed send or signaled/
  timed-out wait — instead of any pending posted message (`814a6605`);
  the slice-end handoff records its rotation and every pre-run rotation
  branch defers to that decision until the chosen process actually starts a
  run attempt (`eb7065e1`, `c6f0552d`). Live verification: process slices
  went from 4700:37 (happyway:iNavi) to an exact 150:150 round-robin, and
  `resmapi_800x480.bin` decode advanced past 3.0 MB (previous best 1.4 MB)
  while the splash stays fully painted.

- Raw `GetOwnerProcess` now returns the current CE process id, and
  `GetCallerVMProcessId`/`GetDirectCallerProcessId` resolve through coredll
  ordinals with the same value (`c59d08ad`).
- Boot-time regression root cause: zero-byte marker files created on the SD
  image between 2026-06-11 and 2026-06-17 (`traffictest.txt`, `routetest.txt`,
  `soap.bin`, `oildata.dat`) enabled `happyway_win.exe` test modes
  (`traffictest`/`routetest` strings are embedded in that binary) and slowed
  boot from ~30 s to 20+ minutes in both single-shot and live-pump modes.
  Removing them restored full splash paint in ~90 s of a live run. Current
  builds do not re-create the files through splash/mapdata phases, so the
  probe-materialization bug that produced them appears already fixed.
- Slice profiling (`RUST_LOG=ce.slice_profile=debug`) shows the per-slice
  Unicorn engine rebuild is NOT the live-pump bottleneck: setup ~4 ms,
  hook install ~6 ms, blob read-back ~1 ms against ~1010 ms of guest
  execution per slice, with 25 blobs totalling ~43 MB. The dominant boot cost
  is genuine guest work: the hot PCs around `0x00339c00-0x00339da8` are a
  pixel-format conversion loop (RGB565/555 packing with `0xf81f`/`0x7c1f`
  transparency keys plus an alpha-plane pass), not a readiness spin.
- Live-pump handoff now applies CE quantum rotation: an active process that
  consumed its full host wall-clock slice rotates to a CPU-runnable parked
  process even while it keeps dirty visible windows, matching NK same-priority
  round-robin (`73bff641`). Previously a visible process with a dirty window
  and a runnable hidden helper starved the helper indefinitely (measured
  175:2 slice split over three minutes).
- The per-instruction code hook no longer clones the trampoline range vector
  on every executed instruction (`58ccb04d`).
- Debug snapshots published to the remote server can be minutes stale while
  the same process keeps the CPU, because publication happens on scheduler
  changes; use the run log or `ce.slice_profile`/`ce.quantum` debug targets
  for live-state truth.

## Current Snapshot

- Loader/runtime mapping reaches dynamic Unicorn DLL mapping with dependency
  loading, import patching, forwarders, rollback on failed loads, trampoline
  tracking, datafile/no-resolve handling, and CE-style attach/detach lifecycle
  behavior.
- Shell/resource behavior includes PE icon extraction, shell fallback icons,
  image lists, `DrawIconEx`, `SHGetFileInfo`, version-resource extraction, and
  several CE-compatible image-list validation and lifetime paths.
- File and storage behavior includes mounted host paths, volume-boundary checks,
  file attributes, find handles, change notifications, `CeGetFileNotificationInfo`,
  FSDMGR import shims, public AFS mount registration, synthetic disk metadata,
  sparse-sector backing, StoreMgr partition format no-ops, and selected
  FMD/file-lock/security descriptor surfaces.
- GWE/message behavior includes HWND/message-queue state, menu and message-box
  foundations, cross-thread send setup, timeout completion, nested reply paths,
  and scheduler wake integration for multiple wait classes.
- GDI/text/IME behavior covers many raw CE edge cases for DIBs, bitmaps,
  palettes, draw APIs, alpha/mask paths, selected font metrics, IME state, and
  TESTIME-backed dictionary behavior.
- CE `CreateDIBSection(DIB_PAL_COLORS)` now follows the GDIAPI helper's 8 bpp
  rule: 8 bpp DIBSections keep mutable color tables, while 1/2/4 bpp and
  truecolor requests fail during creation.
- Remote runtime behavior exposes status, frame capture, touch/key input,
  GPS/NMEA serial injection, debug handle snapshots, and audio/control WebSocket
  plumbing.
- Process-handle behavior includes opening launched process IDs as distinct
  handles plus raw `ReadProcessMemory`/`WriteProcessMemory` byte copies with
  CE-style handle validation and transferred-byte counts.
- Raw stack diagnostic behavior includes a minimal `GetCallStackSnapshot`
  surface for normal and extended frame buffers, skip handling, flag validation,
  and CE-style last-error reporting.
- Stub/raw no-feature validation now keeps visible receiver work aligned with
  the Unicorn runtime, including sent messages, queued visible messages, and
  dirty visible windows.
- Toolhelp process snapshots now expose the current process and launched child
  process entries through raw `THCreateSnapshot` memory blocks.
- Toolhelp thread snapshots now expose current and launched thread entries,
  owner process IDs, priorities, and state flags through raw `THCreateSnapshot`
  memory blocks.
- Toolhelp module snapshots now expose live loaded-module entries, image bases,
  sizes, refcounts, names, and guest paths through raw `THCreateSnapshot`
  memory blocks.
- Toolhelp heap snapshots now append live process heap lists and fixed heap
  allocation entries through raw `GetHeapSnapshot` memory blocks.
- Public `toolhelp.dll` imports now create/close snapshots, walk process,
  thread, module, heap-list, and heap-entry records, and read current-process
  memory through CE-shaped snapshot buffers.

## Earlier Runtime And Subsystem Facts

- The former no-hook crash at handle-like `pc=0x00001054` no longer reproduces
  after blocked scheduler contexts were removed from direct receiver-work
  eligibility. Debug handle output identifies `0x1054` as an event handle.
- Host wall-clock slice stops now purge stale blocked-wait and parked
  `GetMessage` records for the thread that just executed, avoiding impossible
  active-and-blocked thread snapshots at `THREAD_EXIT_STUB_ADDR`.
- Host live-pump idle polling no longer forces short 100 ms CPU slices while an
  active guest context is runnable.
- Current live runs reach real iNavi splash/resource state, hidden map child
  windows, map/resource file loading, `UID1:` probing, and eventually `COM7:`
  GPS startup. The 2026-06-21 post-drive sample opened `COM7:`, drained queued
  remote NMEA into the serial RX buffer, and accepted the guest `$PUBX`
  configuration writes.
- Remote taps are accepted by the live server and now route through the main
  iNavi splash/window stack after stale visible-message cleanup.
- Remote live-pump servicing now drains queued remote-server controls from
  import traps, the scheduler block hook, and inline wait timeout paths. The
  2026-06-21 live run cleared `pendingControlMessages` to zero and posted a tap
  to splash window `0x00020008` as real `WM_LBUTTONDOWN`/`WM_MOUSEMOVE`
  messages.
- Escaped visible-message WndProc callouts are now retired before the
  visible-dispatch gate decides whether pending callbacks block new input. The
  live `happyway_win` stale `OrphanedVisibleMessage` case now clears, leaving
  `pending wndproc returns: 0`.
- Current live taps route to the main iNavi splash/window stack with accepted
  remote key/touch posts; the framebuffer still shows the real iNavi splash
  artwork, so the remaining issue is after message dispatch has resumed.
- Receiver/window teardown now removes terminated send ids from active send
  stacks. The latest live `happyway_win` receiver-terminated send still leaves
  a completed result for the sender, but `active_stacks=0` and
  `depth_threads=0`.
- Current 2026-06-21 live samples show the splash phase persists after
  touch/key input is accepted and routed; the app continues resource/window
  setup (`resmapi_800x480.bin` memory-backed opens and hidden `afxwnd42u` child
  creation).
- Remote/live scheduling now rotates away from an active process with no
  running guest thread and no queued visible receiver work when another parked
  process is runnable. The latest sample advances iNavi through additional
  map/resource loads and accepts touch/key WndProc callouts on the real splash
  window instead of sitting only in the hidden `happyway_win` helper slice.
- Live-pump run entry now also prioritizes ready parked waits before starting a
  hidden active slice. This matches the after-slice policy and covers the
  observed state where hidden `happyway_win` had no receiver work while a
  visible iNavi thread was parked on a ready sleep wait.
- Live-pump generic parked-process rotation now keeps a visible active process
  in front at both after-slice handoff and run entry. The 2026-06-21 fresh
  release run advanced beyond the earlier splash-only sample into real
  mapdata/SearchDB/mapinfo loading while keeping remote input accepted; the
  debug snapshot can still land on hidden `happyway_win` between slices.
- `YAS526B.dll` dump validation confirmed `MFS_IOControl` dispatches exact
  IOCTLs `0xb0000000`, `0xb0000004`, `0xb0000008`, `0xb000000c`, and
  `0xb0000010`. The virtual `MFS1:` shim now follows the dumped contract:
  writes validate/send selector-command input but do not mutate the next sample,
  and reads use input byte 0 as device selector plus byte 2 as count.
- Elevated `cargo flamegraph` startup samples still stop in the real
  `happyway_win.exe+0x7b56c` traffic/shared-memory fill loop; the live-pump
  visible-window readiness check no longer clones the full GWE window list on
  that hot path.
- `MultiByteToWideChar`/`WideCharToMultiByte` now skip host Windows conversion
  calls for zero-flag ASCII input under Korean ACP/UTF-8, preserving explicit
  length and null-terminated return counts.
- Raw `GetDeviceInformationByDeviceHandle` and
  `GetDeviceInformationByFileHandle` now resolve through coredll ordinals and
  write the CE `DEVMGR_DEVICE_INFORMATION` layout for live device handles,
  including legacy name, registry driver key, and `$device\...` name.
- Raw `FindFirstDevice` and `FindNextDevice` now resolve through coredll
  ordinals and enumerate live activated device handles with CE-style device
  search handles, including legacy-name, device-name, bus-name, parent, no
  match, and stale-handle behavior.
- Activated configured devices now capture registry `BusName`, expose it in
  CE `DEVMGR_DEVICE_INFORMATION` as `$bus\...`, and use that activation-time
  value for `DeviceSearchByBusName` wildcard matching.
- Current post-fix live device sample still has no open `COM7:` handle after
  queued remote GPS input; `queued_serial_bytes` grows because the guest has
  not opened the serial device in this run. `MFS1:` and `SMB1:` also remain
  unopened in the same sample.
- Fresh 2026-06-21 release run no longer logs mounted resource file opens as
  `ce.devices` "not configured" misses before falling back to host storage.
  `CreateFileW` now probes the device namespace only for names present in the
  configured device table; configured CE device names still open normally.
- Remote GPS is now explicitly routed to configured `COM7:` and live startup
  reaches that handle again. The 2026-06-21 release sample drained queued NMEA
  into `COM7:`, accepted `$PUBX` configuration writes, and returned GPS bytes
  through real guest `ReadFile` calls. The same run opened `MFS1:` and `SMB1:`
  and issued the dumped sensor IOCTL command values through the virtual devices.
- Serial `ReadFile` now models pending overlapped reads for empty serial RX:
  it records the caller buffer, resets/signals the supplied event, reports
  `ERROR_IO_PENDING`/`ERROR_IO_INCOMPLETE`, and completes through
  `GetOverlappedResult` once remote serial bytes arrive. Focused kernel and raw
  coredll tests cover the event wake and byte delivery path.
- Idle visible helper-process slices no longer keep CPU ownership ahead of a
  runnable visible parked UI process. The scheduler now applies the visible
  parked-process handoff outside live-pump-only paths; the 2026-06-21 release
  validation showed active execution returning to `iNavi.exe+0x317ea0` with
  `happyway_win` parked after the earlier `happyway_win.exe+0x47490` sample.
- The live code-hook stop path now uses the same larger visible parked-process
  predicate as the pending-timeslice decision. This covers helpers with a small
  visible window, such as `happyway_win`, instead of only stopping helpers with
  no visible surface.

## Recent Validation

- Targeted formatting and tests passed for the blocked-thread receiver-work
  slice, stale active-wait cleanup, host idle-slice guard, and related focused
  scheduler regressions.
- Release/live validation survived the prior crash window, served framebuffer
  snapshots, consumed remote touch/key input on the iNavi splash window, and
  confirmed remote GPS drains once the guest opens `COM7:`.
- Focused raw kernel ordinal validation covers duplicated process handles,
  shared process exit state, and process memory read/write copies.
- Focused raw kernel ordinal validation covers normal and extended
  `GetCallStackSnapshot` buffers plus skip and invalid-flag behavior.
- Focused raw kernel ordinal validation covers `THCreateSnapshot` success and
  invalid requested-process failure, thread-entry serialization, module-entry
  serialization, and `GetHeapSnapshot` heap-list and entry serialization.
- Focused raw kernel conversion validation covers Korean ACP conversion plus
  ASCII ACP fast-path sizing and explicit-length no-NUL behavior.
- Focused raw coredll/kernel validation covers device-information success for
  activated and file-open device handles plus null, short-buffer, and stale
  handle failures.
- Focused raw coredll/kernel validation covers `FindFirstDevice`/`FindNextDevice`
  device search output and cleanup semantics.
- Focused Unicorn validation covers the ready parked wait handoff and the
  hidden-helper live-pump handoff. Release build passed with existing warnings.
- Focused raw coredll/kernel validation covers activation-time `BusName`
  capture, `$bus\...` device-info output, and bus-name wildcard search.
- Focused device-name validation covers the configured-device precheck used by
  `CreateFileW`, including CE colon suffixes, `\\.\` prefixes, and ordinary
  mounted file paths. Release build with `--features unicorn` passed with
  existing warnings after cleaning the stale running executable.
- Focused serial validation covers remote GPS target selection, pending
  overlapped serial event signaling, and raw coredll
  `ReadFile`/`GetOverlappedResult` completion after remote NMEA arrives.
- Release/live driving with `--remote-server 192.168.0.39:8765` confirmed
  `queuedSerialBytes` dropped to zero after `COM7:` opened, followed by repeated
  `SerialRemoteDrain` and `ReadFile` records with NMEA payloads.
- Focused Unicorn scheduler validation covers the idle visible helper handoff
  to a visible parked process even when the handoff is not entered through the
  live-pump-only branch. Release build with `--features unicorn` passed with
  existing warnings.
- Full `cargo fmt --check` may still report unrelated pre-existing formatting
  drift in older files; treat new non-formatting whitespace findings as
  actionable.

## Working Rules

- Keep future entries concise and source-visible.
- Record completed behavior here only after code and validation support it.
- Move active work to `TODO.md`; move reproducible unresolved failures to
  `KNOWN_BUGS.md`.
