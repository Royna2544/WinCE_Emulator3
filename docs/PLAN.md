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
| 2 CPU interface and VM coherence (fresh-engine lifetime) | Primary-native stop contract; D-008 committed-page backing, canonical birth identity, machine-level clone/relineage and native pin/lease design | E35 private backing, O-P4 fresh owner, E-39 native cross-process and D008 public shared-view source gates landed; connect immutable then writable image/heap and remaining identity/failure boundaries | Keep E-7/E-18, E35 public private, O-P4 closed-owner Context/pin, E-39 and D008 controls green; prove full clone relineage and bounded live behavior before Stage 2 acceptance |
| 3 Task/object authority | Extend the Stage 2 canonical process-birth directory with Task/object execution/alias/wait pins | Migrate task, alias and lifecycle consumers to that same authority; do not re-mint boot/child identities | Ledger E-8 reds turn green; alias coherence, durable PID/primary identity, close/exit/wait lifetime, independent checkpoint clones |
| 4 Continuation ownership | Raw-native vs synthesized frames; nested callback/wait state | One complete continuation family, removing old duplicate authority | Native wait→callback→resume/exit, FPU/HI/LO/branch correctness, cancellation and debugger exactness |
| 5 Outcomes and scheduler | Explicit count/yield/deadline/error, ready indexes, CE6 priorities/quantum | Replace false timeout inference and policy heuristics together | Native errors hard-fail, equal-priority RR, higher-priority preemption, blocked/suspended eligibility; no starvation |
| 6 One guest clock | CPU/time conversion; live-first mode plus opt-in fast-idle (D-002) | Replace hybrid offsets; idle advancement only with established events | No double time, finite/infinite/zero waits, wrap projection, timer ordering, external GPS/input pacing |
| 7 Runtime bring-up | Phase-aligned GPS/tap/frame/memory recipe and readiness definition | Integrate validated subsystems, no forced app behavior | Several clean 330 s+ runs; genuine map presentation and navigation response; memory/lag and busy guest work reported |
| 8 Persistent engines (deferred swap) | Engine/cache/hook/FlatView lifetime, stable hook rebinding, translation-arena budget and commit-tail handler proof | Process-owned engines behind the CPU interface, after Stage 5 outcomes | Long stable/churn/map-churn controls; independent contexts; the 650 workload clean over several 330 s+ runs; measured speed or memory gain over fresh engines |

Stages may have parallel design lanes but their implementation dependencies are
not bypassed. Stages 3-6 depend on Stage 1 and the Stage 2 outcome contract, not
on persistent engines. CPU/context work must consume canonical identity rather
than invent another PID/TID registry; scheduler/clock work must consume real
frame and outcome ownership. A fixture's event signal is not live input-delivery
proof. Full task-context transfer waits for Stage 3 canonical identity and
Stage 4 continuation ownership.

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
Unmigrated image/heap bytes, full machine checkpoint relineage, guest
`CreateProcessW` handoff, and live footprint/map readiness retain separate
gates; Stage 2 is not complete. Stage 3 extends, rather than recreates, the
process-birth authority.

### Next Stage 2 slice after the D008 source gate (design only)

Sparse migration was dropped: Microsoft's [CE `CreateFileMapping` reference](https://learn.microsoft.com/en-us/previous-versions/windows/embedded/ms923951%28v%3Dmsdn.10%29)
documents default `SEC_COMMIT` for the whole section. Next are immutable PE
image pages; writable image/heap and Stage 3 Task authority stay later.

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

- Task/handle ownership: 13 controls, 1 pass, 12 expected failures (E-8,
  E-11), the red baseline for Stage 3. The added last-public-handle,
  alias-indexed wake and registered-wait retention controls are red; native
  execution/wait acquisition and stale-incarnation controls remain to add.
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
