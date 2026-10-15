# Architecture-first implementation plan

Common anchor: [ARCHITECTURE.md](ARCHITECTURE.md). All subsystem plans use its
contracts. Open decisions: [DECISIONS.md](DECISIONS.md). Evidence:
[evidence/ledger.md](evidence/ledger.md). The user has authorized beginning the
scheduler-core rewrite while retaining plan-first, implement-late dependency
gates. This document is a reviewable sequence, not a claim that the target
architecture is implemented.

## Entry state

- The clean bounded BED2 binary is hash-preserved under `evidence/` and remains
  the measured fallback, still on the splash (ledger E-1). The current
  `target/release` executable has a different hash and is not that fallback.
- The persistent-owner experiment passed 1,618 tests but crashed at 129.108 s
  with `0xC0000005` and 1.62 GiB RSS (E-2). Its source, binary and evidence
  are preserved; a unit suite is not permission for runtime acceptance.
- Decision [D-003](DECISIONS.md): Stages 3-6 run on the fresh-engine-per-slice
  lifetime. Persistent engines move to Stage 8, a later swap behind the CPU
  interface, so the native-arena investigation no longer gates the scheduler.
- O-P4 now makes that fresh lifetime the source-bound default. Its frozen
  release/no-feature suites, native closed-owner Context and 200-slice pin
  controls passed ([receipt](../evidence/op4-fresh-default-20261014/validation.receipt.md)).
  This is a code/test gate, not a bounded iNavi run or Stage 2 acceptance.
- E-39 then proved public private-VM and injected file-view A→B→A same-VA
  bytes across fresh native owners. D008's public named mapping A→B→A plus
  same-slice alias and legacy-DIB migration controls passed focused and final
  frozen-source broad gates: release 1,744/0/14 ignored and no-feature
  1,297/0/13 ignored, each across 17 result blocks
  ([receipt](../evidence/d008-shared-view-20261014/validation.receipt.md)).
  These source controls do not prove guest
  `CreateProcessW` handoff, image/heap migration or full clone relineage.
- E-43 later sealed immutable main-EXE pages, E-44 source-gated fixed initial
  heap/stack slabs, and O-M2 source-gated per-birth heap authority with a
  256-MiB sparse fixed window and no copied spillover
  ([receipt](../evidence/om2-heap-20261014/validation.receipt.md)). This did
  not run iNavi or prove a speed, host-RSS or map-readiness gain.
- D-010's finite-Sleep Task/Frame cut passed its resealed source gate: release
  2,025/0/9 ignored and no-feature 1,472/0/3 ignored
  ([receipt](../evidence/stage3-finite-sleep-frame-20261015/validation.receipt.md)).
  Only finite Sleep has this canonical active frame; broader Task/continuation
  authority, Stage 3 and live iNavi acceptance remain open.
- Phase 0 hot-path hygiene is done (E-9): amortized scheduler block hook,
  hookless trampoline default, per-slice guest-MIPS logging. It changed no
  scheduling policy; the 1M-instruction slice count is still the round-robin tick.
- Documentation and diagnostics are kept separate from feature migrations.

## Required design record before each implementation

1. Identify the common-anchor contract and single owning subsystem.
2. Inventory real state, callers, aliases, lifetime pins and clone behavior.
3. Specify interfaces, legal dependency direction and state/event transitions.
4. Define failure, cancellation, publication and rollback boundaries.
5. Cite local CE6/native/device evidence; mark unknowns and proposals explicitly.
6. Stage genuine expected-red tests and an exact acceptance/measurement recipe.
7. Obtain independent source review, resolve contract conflicts, then implement
   the smallest connected change. Review is not a substitute for execution.

No implementation starts merely because a design file exists. Record gate
results and remaining decisions in the subsystem page, [DECISIONS.md](DECISIONS.md)
and the durable status files.

## Migration sequence and acceptance gates

| Stage | Design/output first | Implementation allowed after gate | Acceptance |
| --- | --- | --- | --- |
| 0 Recovery/evidence | Crash ledger, preserved 650/BED2, scoped cleanup manifest | Diagnostic-only probes/fixtures; no architecture policy change | Exact crash symbol/source recorded in the ledger; clean fallback intact |
| 1 Common boundaries | Anchor + subsystem ownership/interface/error pages, dependency review | Named interfaces only when they replace actual consumers | No duplicate state authority; no unresolved identity/time semantics hidden in defaults |
| 2 CPU interface and VM coherence (fresh-engine lifetime) | Primary-native stop contract; D-008 committed-page backing, canonical birth identity, machine-level clone/relineage and native pin/lease design | E35 private backing, O-P4 fresh owner, E-39 native cross-process, D008 public shared-view, E-43 sealed main image, E-44 fixed heap/stack and O-M2 per-birth heap source gates landed; connect writable image/DLL and remaining identity/failure boundaries | Keep E-7/E-18, E35 public private, O-P4 closed-owner Context/pin, E-39, D008 and O-M2 controls green; prove full clone relineage and bounded live behavior before Stage 2 acceptance |
| 3 Task/object authority | Extend the Stage 2 canonical process-birth directory with Task/object execution/alias/wait pins | Migrate task, alias and lifecycle consumers to that same authority; do not re-mint boot/child identities | Ledger E-8 reds turn green; alias coherence, durable PID/primary identity, close/exit/wait lifetime, independent checkpoint clones |
| 4 Continuation ownership | Raw-native vs synthesized frames; nested callback/wait state | One complete continuation family, removing old duplicate authority | Native wait→callback→resume/exit, FPU/HI/LO/branch correctness, cancellation and debugger exactness |
| 5 Outcomes and scheduler | Explicit count/yield/deadline/error, ready indexes, CE6 priorities/quantum | Replace false timeout inference and policy heuristics together | Native errors hard-fail, equal-priority RR, higher-priority preemption, blocked/suspended eligibility; no starvation |
| 6 One guest clock | CPU/time conversion; live-first mode plus opt-in fast-idle (D-002) | Replace hybrid offsets; idle advancement only with established events | No double time, finite/infinite/zero waits, wrap projection, timer ordering, external GPS/input pacing |
| 7 Runtime bring-up | Phase-aligned GPS/tap/frame/memory recipe, readiness definition and reviewed footprint/responsiveness budgets | Integrate validated subsystems, no forced app behavior | Several clean 330 s+ runs; genuine map presentation and navigation response; phase-matched memory, correlated input-to-frame responsiveness and busy guest work measured against the reviewed budgets |
| 8 Persistent engines (deferred swap) | Engine/cache/hook/FlatView lifetime, stable hook rebinding, translation-arena budget and commit-tail handler proof | Process-owned engines behind the CPU interface, after Stage 5 outcomes | Long stable/churn/map-churn controls; independent contexts; the 650 workload clean over several 330 s+ runs; measured speed or memory gain over fresh engines |

Stages may have parallel design lanes but their implementation dependencies are
not bypassed. Stages 3-6 depend on Stage 1 and the Stage 2 outcome contract, not
on persistent engines. CPU/context work must consume canonical identity rather
than invent another PID/TID registry; scheduler/clock work must consume real
frame and outcome ownership. A fixture's event signal is not live input-delivery
proof. Full task-context transfer waits for Stage 3 canonical identity and
Stage 4 continuation ownership.

The bounded [host-exit source gate](../evidence/stage3-host-teardown-20261015/validation.receipt.md)
selects direct windows/backing by process birth and fences a no-birth pseudo-exit.
The subsequent [sender-ownership source gate](../evidence/stage3-sender-birth-20261015/validation.receipt.md)
passed captured-birth Send/Timeout admission, atomic birth-owned publication,
exact sender process/thread exit selection, native wait/result/re-entry checks,
and pre-entry guards for the identified guest no-wait message producers. The
three preserved sender expected-red controls are now enabled; release
**2,067/0/9 ignored** and no-default **1,495/0/3 ignored** passed across 18
groups each. `LegacyUnknown` remains explicit fixture compatibility, not
production birth authority. The later
[exact-window-set source gate](../evidence/stage3-host-exit-exact-window-20261015/validation.receipt.md)
retired only frozen-birth host-exit HWNDs despite synthetic foreign links and
woke an exact sender waiting on a retired receiver; its release
**2,072/0/9 ignored** and no-default **1,500/0/3 ignored** suites passed.
Direct WndProc dispatch, receiver-side cross-process result writes,
conditional system-origin device broadcasts, pseudo-exit caller authority,
whole-exit atomicity, CE6 foreign-link/cancellation parity, full Stage 3 and
live iNavi remain separate gates. No live run is approved by these source gates.

Stage 7 budgets are a design gate, not values inferred from the current
splash-only E-38 observation. Before calling memory or responsiveness "small"
or "fast," record the target device's configured CE RAM and real navigation
interaction expectations, define which guest/host memory quantities and which
input-response interval each budget covers, and review the limits with the
user. A future exact lag claim needs lineage-linked event IDs and stamps on
one host-monotonic clock at named boundaries, such as actual guest input
dispatch and publication of a subsequent changed frame. Until those links
exist, the current two-second frame polling yields only a conservative
observation bound, which must be labelled as such. Use repeated, phase- and
input-matched runs for comparisons. The [verification recipe](subsystems/verification.md)
keeps incomplete telemetry and unknown native/backing capacity explicit.

D-008 permitted a narrow private-slab E-18 diagnostic before public owner/clone
acceptance. E25 reached that historical diagnostic milestone on the corrected
source under a test-only opt-in
([receipt](../evidence/e25-private-slab-20261014/broad-validation.receipt.md)).
E35 subsequently validated the public private `VirtualAlloc` family with
owner-qualified backing and negative checkpoint guards
([receipt](../evidence/e35-public-virtualalloc-validation-20261014/validation.receipt.md)).
O-P4 then made a fresh native owner the default and passed its pinned-source
close/pin/context code and test gate
([receipt](../evidence/op4-fresh-default-20261014/validation.receipt.md)).
The pinned vendored C `uc_close` proof is not a general checked-close API.
E-39 separately validated real native public-private and injected-file-view
cross-process same-VA isolation. D008's public shared-view, same-slice alias
and DIB migration controls passed focused and final frozen-source broad gates
([receipt](../evidence/d008-shared-view-20261014/validation.receipt.md)).
Unmigrated writable image/DLL bytes, safe heap address reuse, full machine
checkpoint relineage, guest `CreateProcessW` handoff, and live footprint/map
readiness retain separate
gates; Stage 2 is not complete. Stage 3 extends, rather than recreates, the
process-birth authority.

### Remaining Stage 2 slices after the O-M2 source gate

Sparse migration was dropped: Microsoft's [CE `CreateFileMapping` reference](https://learn.microsoft.com/en-us/previous-versions/windows/embedded/ms923951%28v%3Dmsdn.10%29)
documents default `SEC_COMMIT` for the whole section. E-43 sealed immutable
main-EXE pages, E-44 migrated the initial heap/stack slabs and O-M2 replaced
copied heap spillover with sparse per-birth fixed pages. Writable image/DLL
backing, full clone lineage, safe heap reuse and Stage 3 Task authority remain
separate gates.

### Parallel lanes (never gate a stage)

- Native arena lane: E-33's one vendored Windows commit-tail arithmetic repair
  is the [D-009](DECISIONS.md) safety exception after a pristine-red native
  control. Stop further arena investigation here. No arena-size or
  persistent-owner/cache production changes precede Stage 8's design gate. O-P4's
  fresh-default Stage 2 migration is a separate executed gate, not Stage 8 or
  app acceptance.
- App evidence lane: bounded, read-only investigation of what iNavi waits on and
  what the helper-scheduling bimodality depends on. See
  [app-evidence-lane.md](app-evidence-lane.md).

## Current controls, not yet full-stage acceptance

Executed controls are recorded in the ledger and source-bound receipts; they
close only their named gates, not Stage 2 or live acceptance.

- Task/handle ownership: the thread/process alias entry slice passed its
  frozen-source gate ([receipt](../evidence/stage3-alias-20261014/validation.receipt.md)).
  The bounded creation-owner follow-on now freezes a checked ProcessBirthKey
  for production thread births, registers aliasless boot TID 1, binds the
  child primary to its birth and routes production CreateThread through a
  checked creator TID. Two E-26 owner controls are active; its source-bound
  debug no-default gate passed 1,364/0/5 and release unicorn/trace/desktop
  passed 1,829/0/11, with desktop no-trace check also green
  ([receipt](../evidence/stage3-owner-20261014/validation.receipt.md)).
  Four ownership/pin controls were deliberately ignored at that owner-slice
  checkpoint; the later O-K3 pin gate activated two of them. A bounded
  child-exit slice adds exact-birth completion, worker retirement at verified
  exit, request-only parked-child retention and deferred real-process signaling.
  A subsequent bounded, source-tested `TerminateProcess` handshake stops an
  owned active child before another native attempt or at its self-call import
  boundary. It also completes exact-birth requested parked and pending children
  without running them. The current-process pseudo-handle remains a separate
  synchronous compatibility lane. Its frozen-source broad gates passed:
  no-feature 1,375/0/5 ignored and release Unicorn/trace/desktop
  1,858/0/11 ignored, with the no-trace desktop check green
  ([receipt](../evidence/stage3-terminate-20261015/validation.receipt.md));
  no live-app behavior has been measured. A subsequent O-K3 source gate
  retains execution-live task births after final public-alias closure and
  pins registered Thread/Process wait targets by exact birth before publishing
  a wait. Completion, timeout, cancellation and failed preparation release
  only acquired pins ([receipt](../evidence/stage3-pins-20261015/validation.receipt.md)).
  The subsequent native prepared-wait slice makes target preparation fallible
  before pending-callout/old-wait mutation, then publishes the prepared pins;
  invalid/stale targets and pin exhaustion return `WAIT_FAILED` with a CE last
  error, leaving the old state intact. Focused failure/retry controls and its
  bounded [source gate](../evidence/stage3-waitprep-20261015/validation.receipt.md)
  passed; runtime behavior is a separate acceptance step. The later bounded
  stale-TID/native-stamp [source gate](../evidence/stage3-stale-tid-20261015/validation.receipt.md)
  carries exact thread birth with pending returns and suspended contexts,
  verifies the current `(TID, birth)` at native slice-end staging, and
  birth-verifies the GetMessage compatibility bridge before resuming. Stale
  same-TID A→B reuse fails closed without restoring A's state into B.
  The later bounded [Sleep/object parked-wait source gate](../evidence/stage3-parked-wait-20261015/validation.receipt.md)
  verifies captured register ownership before publication, pairs the native
  payload with its live scheduler wait before wake consumption, retires only
  an invalid exact wait ID, and carries verified birth into a normal resumed
  slice.
  The separate bounded [parked MsgWait source gate](../evidence/stage3-msgwait-birth-20261015/validation.receipt.md)
  validates the captured native birth before timer pumping or publication,
  quarantines a stale exact wait before preselection timer delivery or input
  acknowledgement, and stamps a verified birth on normal resume. Its raw
  synchronous fallback paths are not covered.
  The bounded [parked queue/socket source gate](../evidence/stage3-queue-socket-birth-20261015/validation.receipt.md)
  verifies native MessageQueueIo/WinsockRead register birth before publication
  or host socket work, rejects stale exact waits before wake consumption, and
  stamps verified birth on normal resume. Raw synchronous/nonblocking fallbacks
  and opaque non-task target pins remain separate.
  The bounded [parked serial source gate](../evidence/stage3-serial-birth-20261015/validation.receipt.md)
  verifies native SerialRead/SerialCommEvent register birth before host serial
  polling or publication, checks the exact live scheduler/native pair before
  wake or finite-timeout completion, and stamps verified birth on normal
  resume. Raw synchronous/nonblocking/overlapped serial paths and opaque
  non-task serial target pins remain separate.
  The bounded [parked SendMessage source gate](../evidence/stage3-send-birth-20261015/validation.receipt.md)
  verifies cross-process native sender birth before queueing, checks the exact
  scheduler/native pair before result consumption, and stamps a verified birth
  on normal resume. The gate seals the tested Git HEAD plus binary patch and
  passed full release 1,916/0/9 ignored and no-feature 1,395/0/3 ignored
  tests. The separate bounded
  [kernel sender-exit source gate](../evidence/stage3-send-exit-20261015/validation.receipt.md)
  removes exact-birth queued sends and detaches active sends on the first
  successful live thread exit, preserving other senders' queued work. Its
  tested HEAD plus patch passed full release 1,920/0/9 ignored and no-feature
  1,399/0/3 ignored tests. The bounded
  [native exit source gate](../evidence/stage3-native-exit-birth-20261015/validation.receipt.md)
  verifies captured thread birth and process lease before native `ExitThread`
  decisions and the guest return-stub no-exit-candidate fallback; its sealed
  HEAD plus patch passed full release 1,923/0/9 ignored and no-feature
  1,399/0/3 ignored tests. The return-stub hook lacks an end-to-end control,
  and pending-return KData PID still uses mutable process selection rather
  than the captured lease.
  The bounded [native SMT source gate](../evidence/stage3-smt-birth-20261015/validation.receipt.md)
  verifies eligible parked producer/re-entry/resume against captured birth and
  exact scheduler/native/GWE transaction, abandoning only an exact stale GWE
  send ID. Captured HEAD `d5738222` plus the sealed patch was incorporated as
  the same source at final HEAD `7533b528`; clean full release a02 passed
  1,929/0/9 ignored and no-feature a01 passed 1,402/0/3 ignored tests. The
  subsequent bounded [occupied-slot gate](../evidence/stage3-smt-occupied-20261015/validation.receipt.md)
  fails a second eligible sender with a backend error before raw dispatch;
  release passed 1,932/0/9 and no-feature 1,402/0/3 on the sealed source.
  This is not concurrent A+B support. Global expiry may be misreported as
  ready/success, and receiver-side `DestroyWindow` result writes and broader
  GWE lifecycle remain separate.
  The bounded exact-birth wait-owner
  [source gate](../evidence/stage3-wait-owner-20261015/validation.receipt.md)
  passed release 1,947/0/9 and no-feature 1,414/0/3 across 18 blocks each.
  One wait store now distinguishes exact birth from explicit legacy unknown;
  production publication is fallible, late failure is terminal, and exact
  exit/modal-HWND cleanup does not sweep a reused TID. A failed a01 attempt
  remains preserved. The separate bounded thread-directed GWE birth
  [source gate](../evidence/stage3-gwe-thread-birth-20261015/validation.receipt.md)
  passed release 1,952/0/9 and no-feature 1,417/0/3. The separate HWND-less thread-timer
  [source gate](../evidence/stage3-thread-timer-birth-20261015/validation.receipt.md)
  passed release 1,960/0/9 and no-feature 1,424/0/3 while preserving the
  HWND-owned/legacy path. The bounded W-B1a window-owner
  [source gate](../evidence/stage3-window-owner-birth-20261015/validation.receipt.md)
  freezes creation ownership, preflights native ordinals 246/688/690, and
  birth-gates explicit HWND post and typed-payload retirement. Five pre-fix
  reds and the failed first broad attempt remain preserved; corrected release
  passed 1,967/0/9 and no-feature 1,427/0/3. The separate bounded W-B1b
  remote touch/key [source gate](../evidence/stage3-window-input-birth-20261015/validation.receipt.md)
  passed 27/27 focused controls, a38 release 1,988/0/9 and a39 no-feature
  1,448/0/3 across 18 blocks each, plus a40/a41 feature checks on the fresh
  a38 seal. Earlier a33/a34 broad attempts failed on an obsolete hidden-window
  fixture, repaired at a35 and focused a36/a37; they are not acceptance runs.
  The input-only gate distinguishes explicit host HWNDs from guest Get/Peek
  retrieval filters; coordinate touch retains live capture/hit-test, blocked-
  wait/modal HWNDs are fallback hints only, and keys use live focus/active
  ownership. Full exit capture retirement and implicit gesture ownership
  remain open. Independent-window/backing selection on verified process exit
  and the no-birth pseudo-exit fence are source-tested under PID reuse (focused
  controls; release **2,055/0/9 ignored** and no-default **1,485/0/3 ignored**
  across 18 groups each;
  [receipt](../evidence/stage3-host-teardown-20261015/validation.receipt.md)).
  At that earlier host-exit seal, recursive foreign parent/owner links and
  numeric sent-message sender cleanup lacked birth proof. The later bounded
  [sender gate](../evidence/stage3-sender-birth-20261015/validation.receipt.md)
  covers tested queued and active sends, including identified native and
  no-wait producers. The later
  [exact-window-set gate](../evidence/stage3-host-exit-exact-window-20261015/validation.receipt.md)
  source-tests foreign-link isolation without changing guest
  `DestroyWindow` recursion; full exit-effect authority, live-app behavior
  and CE6 parity remain open.
  Broadcast, HWND timers, hotkeys,
  sent/paint/modal producers, native continuation
  payloads, wait-ID wrap/reuse and live modal behavior remain later gates
  ([tasks-scheduler](subsystems/tasks-scheduler.md)). Opaque non-task targets
  remain unpinned, while synchronous finite inline timeout paths are outside
  the published-wait pin transaction. CE
  caller wait/timeout semantics, old-MIPS direct-kernel real-handle continuation,
  root-process real-handle completion, intra-slice unpinned-worker lineage,
  aliasless Sleep-suspend and generic parked-wait continuation remain open.
  Other blocked payload families have not been migrated by these parked-wait
  gates.
  These bounded source slices are not full Stage 3 or live-app acceptance.
- W-B1c mixed touch/key admission order passed its bounded
  [source gate](../evidence/stage3-mixed-input-order-20261015/validation.receipt.md)
  after a preserved expected red and independent plan review. One typed,
  per-`CeRemote`-instance sequenced FIFO replaces the separate touch/key
  drains; tap admission is atomic, and each key resolves live focus at its
  event position. Explicit HWND routing for both types, W-B1b exact-Birth
  guards and stale-target drops, and the existing `Message.time_ms` projection
  remain. The key(A) → DOWN(B) → key(B) release control passed; the full release
  suite passed 1,992/0/9, no-feature 1,452/0/3 across 18 blocks each, and two
  alternate feature checks passed. The release seal was captured after its run
  began, with matching committed source hashes and no observed intervening
  source write; it is not a pre-run seal. This is local host-input admission,
  not CE6 focus parity, machine-wide cross-adapter arrival order, cross-thread
  WNDPROC execution order, NMEA/serial ordering, or live tap delivery.
  [O-C5](DECISIONS.md) guest-time assignment and host `GetMessage(NULL)` wake
  without an HWND remain open. See [remote-observability](subsystems/remote-observability.md)
  for scope and remaining remote-boundary work. No app or writable-mount run
  was performed for this gate.
- D-010 finite-Sleep Task/Frame publication, exact wake/restore and deferred
  same-process sent-message delivery passed the resealed source gate
  ([receipt](../evidence/stage3-finite-sleep-frame-20261015/validation.receipt.md)).
  Real-MIPS GetMessageW, PeekMessageW(PM_REMOVE) and GetMessageWNoWait paths
  return one WNDPROC result to the blocked sender. Reciprocal cross-Send was
  outside this D-010 gate; post-wake MsgWait, other waits, full Stage 3 and
  live behavior remain open.
- A10 ordinary A→B Send/GetMessage and the subsequent strict A12 S→T→S
  plain-Send handoff passed separate bounded source/test gates
  ([A10](../evidence/scheduler-source-cut-a10/validation.receipt.md),
  [A12](../evidence/scheduler-source-cut-a12/validation.production.receipt.md)).
  A12's post-format release/no-default suites passed 2,047/0/9 ignored and
  1,482/0/3 ignored. The unignored real-MIPS control executes both replies
  once; a wrong-parent-birth control fails before guest entry. Third sender,
  timed/result-pointer, deeper pending-callback and cross-process forms are
  ineligible for this handoff and fail closed with terminal `Backend`. This
  is not general Task/continuation authority or a live-app result.
- The separate bounded A13 blocked-GetMessage orphan
  [source/test gate](../evidence/scheduler-source-cut-a13/validation.final.receipt.md)
  retains exact birth on native blocked payloads, prevents saved-context
  resurrection after the scheduler wait is lost, and fails an orphan selected
  for native entry before guest execution. The tested modal result path
  requires its exact live wait; a missing modal wait fails closed before
  result consumption. Focused saved-context 7/7, integrated orphan 1/1 and
  modal 11/11 passed; the final 18-group suites passed release-feature
  2,052/0/9 ignored and no-default 1,482/0/3 ignored. Broader modal and
  continuation shapes, full Stage 3 and live-app behavior remain open; no
  writable-mount or iNavi run was performed.
- Native outcome: the E-7 full-backend count-stop and nonzero-unmapped-error
  controls were red originally and later turned green on the D: source. E-19
  added focused typed global-hook stop controls and passed the frozen full
  release suite (1,634/0/4 ignored). E-28 added the native count-stop witness;
  E-34 closes stop classification and checked mixed-stop JR preservation, while
  report/failure hardening and in-slice native FPU wait→resume→exit remain open.
- VM coherence: E-16's injected same-VA view control is green on its frozen
  source. E-18's private fresh-owner guest-write control was an expected red
  on the frozen E-19 source; its updated test-only slab control is green on
  the corrected E25 source
  ([receipt](../evidence/e25-private-slab-20261014/broad-validation.receipt.md)).
  E35's normal public-ordinal control failed at its pre-switch backing
  assertion, then passed on frozen source with the public private-slab
  transaction, negative checkpoint boundary and checked pin budget
  ([receipt](../evidence/e35-public-virtualalloc-validation-20261014/validation.receipt.md)).
  E-39's native public-private and injected-view A→B→A controls passed on
  later source. D008's public named-reopen A→B→A and in-slice alias controls
  passed focused runs, as did DIB-to-public whole-object migration/low-budget
  rejection; its final frozen-source release/no-feature broad gates passed
  ([receipt](../evidence/d008-shared-view-20261014/validation.receipt.md)).
  A raw adapter still copies guest
  bytes, and alias propagation failure can leave already-committed effects.
  None of these controls is Stage 2 acceptance or a demonstrated footprint/
  timing improvement.
- Native owner/arena: six Stage 0 owner controls, the pre-init budget
  differential and the commit-tail repair controls (E-4 to E-6) are isolated and
  diagnostic; they feed Stage 8 only.
- Fresh native owner (O-P4): the default now closes after each slice while
  `WINCE_EMU_PERSISTENT_NATIVE_ENGINE=1` preserves the Stage 8 experiment.
  Closed-owner exact-key FPU/HI/LO/MIPS continuation, a 200-slice pointer-map
  and pin churn, finish-failure, typed stops and broad feature suites passed
  on frozen source ([receipt](../evidence/op4-fresh-default-20261014/validation.receipt.md)).
  The source-specific vendored `uc_close` proof does not establish a generic
  checked-close API, cross-task context or application readiness. Later E-39
  and D008's source gate address native same-VA and shared views separately.

## Integration discipline

- **One source dir, one disposable build dir ([D-007](DECISIONS.md)).** All code
  lives in this repo, with no copy mirrors and no numbered builds. A control
  that matters, including an expected-red control, lands here as a normal
  test marked `#[ignore = "expected red: ledger E-n, Stage k"]`. The fix that
  turns it green removes the marker in the same change. Run reds with
  `cargo test ... -- --ignored`. The single Cargo target dir can be deleted at
  any time, so evidence never lives in it.
- Engine lifetime is one policy behind the CPU interface, not a second source
  tree (see [DECISIONS](DECISIONS.md) O-P4). Stage 2-6 code is written once,
  against the fresh lifetime.
- Record the source hashes and configuration used for each evidence run.
- Compile focused tests first, then the complete release suite and normal/
  no-trace/stub variants. Format and check diffs; keep unrelated user changes.
- Keep BED2 and candidate binary/symbol hashes separate. Never replace a live
  process or reset source history to get a benchmark.
- Observe before launch with explicit capture clocks; keep terminal coverage
  after approval/setup gaps. Root owns app/input/watchdog; analysis lanes read
  saved evidence. Stop only exact owned helpers after the target ends.
- No writable-mount live run is authorized by O-P4's green suite. Obtain fresh
  human approval and fresh adjacent ZIP backups of the original SD,
  ResidentFlash and emulator-root mounts before one bounded run. Compare its
  host memory peak, busy-slice MIPS and fixed per-slice setup cost with the
  phase-matched BED2 E-1 and persistent E-22/E-27/E-29 evidence; do not infer
  a win from a single splash or different boot phase.
- Report completed attempts, actual native outcomes, guest instructions/busy
  MIPS, input consumption and epoch-labelled frames. Compare several matched
  phases; no single milestone or truncated crash run proves a speed gain.
- Maintain docs/PROGRESS/TODO/KNOWN_BUGS and scoped artifact receipts. No commit
  or push unless requested. Synced sources and SD data remain untouched.

## Completion definition

Architecture completion requires connected single-authority consumers, not a
folder of unused types. Bring-up completion requires actual map/navigation
presentation, positive GPS consumption and responsive FIFO input under a
stable bounded runtime. Splash appearance, API ACK, test count and native
handle reuse alone cannot close the active goal.
