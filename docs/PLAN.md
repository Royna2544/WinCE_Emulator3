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
The subsequent bounded [guest pseudo-exit caller gate](../evidence/stage3-pseudo-caller-birth-20261015/validation.receipt.md)
source-tested captured thread and native process-lease admission plus decoded
self-PID collision safety; release **2,081/0/9 ignored** and no-default
**1,502/0/3 ignored** passed. It does not prove immediate native CPU stop,
canonical Task retirement or whole-exit atomicity. The later bounded
[D-011 framebuffer gate](../evidence/stage3-host-exit-framebuffer-preflight-20261015/validation.receipt.md)
rejects an un-restorable saved backing before stopped-exit effects and
propagates recovery through tested scheduler paths. It also fails closed when
the stopped boot CPU's saved owner differs from the selected process. The
monitor refuses a new checkpoint after recovery is required. The later
live-lineage [canonical-map guard](../evidence/stage3-checkpoint-orphan-frame-green-20261016/validation.receipt.md)
refuses save/restore while Task records/launches or CPU custody are present,
including an exited thread retaining a frame; no earlier live checkpoint can
currently clear the hold. Repair and
restart are the safe recovery route until atomic machine relineage lands.
The bounded [native COREDLL pseudo self-exit gate](../evidence/stage3-native-pseudo-self-exit-20261016/validation.receipt.md)
source-tested guest CPU stop, native owner closure, exact caller and
launch/boot witnesses, and D-011 preflight before child or boot Task/VM
retirement. At that earlier seal, raw/decoded pseudo paths remained
synchronous compatibility behavior. The later bounded
[decoded self-exit gate](../evidence/stage3-raw-decoded-pseudo-exit-20261016/decoded-pseudo-exit.receipt.md)
requires an attempt-local executed JALR/delay-slot witness and exact stopped
caller/owner before the same finalizer; its
[broad offline suites](../evidence/stage3-raw-decoded-pseudo-exit-20261016/broad-validation.receipt.md)
passed release 2,107/0/9 ignored and no-default 1,511/0/3 ignored. Raw direct
pseudo compatibility remains separate. The bounded
[modal-presence refusal gate](../evidence/stage3-modal-exit-failclosed-20261016/validation.receipt.md)
now fails closed before effects in its tested exact-birth stopped-exit shapes;
checked modal backing restoration remains separate.
The bounded [parked-survivor/saved-CPU-owner source gate](../evidence/stage3-survivor-resume-green-20261016/validation.receipt.md)
applies the [Task/scheduler design](subsystems/tasks-scheduler.md#exact-birth-parked-cpu-survivors-on-process-exit-design-and-bounded-cuts)
to tested natural-parked and requested-active stopped exits: exact-birth
preflight precedes effects, and original nonmodal foreign CPUs are moved
after accepted completion. A foreign saved-context CPU lacking a matching
execution-live current thread birth refuses exit before effects. The
[natural-parked/active duplicate expected reds](../evidence/stage3-parked-survivor-red-20261016/validation.receipt.md)
and earlier passing a09 source are historical; the combined sealed source
passed release 2,144/0/10 ignored and no-default 1,524/0/3 ignored across
19 blocks each. At that seal, first-slice `Exited` still needed a separate
test seam or real-MIPS fixture; other active exits, requested parked
siblings, already-exited skips, generic duplicate pruning, foreign CPU nodes
without saved context and PC resumability remain unproved. The
[CE6/REA evidence](../evidence/ce6-rea-exit-20261016/findings.md) supports
per-process ownership without locating the dumped kernel exit body.

The subsequent bounded [first-slice `Exited` source gate](../evidence/stage3-first-slice-survivor-green-20261016/validation.receipt.md)
uses a production-helper seam: synthetic A→B controls preserve the original
nonmodal B CPU after accepted A exit and refuse mismatched saved ownership
before effects. A held parent queue borrow yields a typed terminal error
without Kernel exit or host panic, but drops the local CPU tree; it is not a
recovery hold. The older test-seam requirement is thus met for this one shape,
not for real-MIPS classifier reachability. Final-source suites passed release
2,147/0/10 ignored and no-default 1,524/0/3 ignored over 19 blocks each.
At that seal, exact live primary birth, nonempty `Completed` descendant
publication, custom framebuffer reentry, other survivor shapes and live iNavi
remained open. The later bounded
[v4 stopped-exit gate](../evidence/stage3-primary-exit-birth-v4-20261017/validation.receipt.md)
checks the canonical exact primary for tested child/boot exits and passed
19-block release and no-default suites. Other exit consumers, descendant CPU
custody, Task/Frame authority and live-app acceptance remain open.

The next preparatory Stage 3/4 cross-boundary source cut requires a
[machine-owned CPU-custody design](subsystems/tasks-scheduler.md#next-custody-and-reachability-design-gate-proposed-not-implemented)
before production custody code: inventory every boot/child/displaced/parked/Completed/switch/
failure/exit owner, settle [O-T7](DECISIONS.md), and stage expected reds for
held-queue publication and ordinary first-slice failure losing a live CPU.
The read-only owner audit found pre-CPU drained-launch errors, a then-
unconditional `Completed` queue borrow, numeric prune/drop sites and
independently cloned checkpoints. A later direct-MIPS source cut skips the
empty-descendant borrow, but nonempty publication conflict still loses CPU
custody after committed exit effects. Add controls for the remaining
boundaries and a late post-effect failure
before admitting a machine-owned slot cut. A live-custody checkpoint fails
closed until CPU and Kernel/VM identities can be relineaged atomically.
The current Kernel exit path has no safe ordinary failure injection after
GWE/window/framebuffer effects and before canonical completion: its dirty
callback is infallible, and a panic or unsafe reentry would test a different
contract. Add a fallible, test-injectable commit seam before claiming a typed
partial-effects quarantine gate; neither the seam nor that gate exists yet.
At its earlier source seal, the [ignored native-fault expected-red](../evidence/stage3-post-admission-native-fault-20261016/validation.receipt.md)
executed an admitted synthetic child through `READ_UNMAPPED`: Kernel
kept its exact birth live/unsignaled, but the ordinary error discarded its
only CPU. This is offline non-live-pump evidence, not a source fix or app run.
The [real-MIPS non-live control](../evidence/stage3-real-mips-first-slice-20261016/validation.receipt.md)
added an executed third red at that seal: A launched B and exited, then the
already-exited-A park skip dropped B's CPU while B stayed Kernel-live. The
slot must be a resource
owner behind the canonical Task directory,
not a new lifecycle registry or another pending vector. Review exact-birth,
clone and post-effect failures before connecting all consumers. This is not
full Stage 4 acceptance or a bypass of unfinished Stage 3 Task authority. A
controlled fixture for the `Exited` classifier remains separate: the real-MIPS
non-live A→B shape rotated and pruned A first, then returned B as `Parked` to
the outer launch handler. At that seal only live-pump deferred the child;
the current source defers successful `CreateProcessW` in both modes after
staging the parent's return PC. Its bounded
[source/test gate](../evidence/stage3-createprocess-deferred-20261017/validation.receipt.md)
passed focused and complete release/no-default suites. Live-pump still needs
its own later-slice survivor fixture; this gate does not provide O-T7 CPU
custody, live-app acceptance or a writable-mount run.

The [2026-10-17 pre-deferral rerun](../evidence/stage3-completed-reentry-red-20261017/validation.receipt.md)
confirmed three then-ignored offline reds on one earlier seal: `Completed`
panicked on a held parent-queue borrow and stranded B; the already-exited-A
skip dropped B; and a post-admission native error dropped the child's CPU.
Two old fixture implementations were subsequently retired because they miss
the post-native route; their historical receipts remain. A new direct-MIPS
first-slice `Completed` fixture reaches the empty-descendant borrow hazard,
and its [bounded source/test gate](../evidence/stage3-completed-borrow-20261017/validation.receipt.md)
passed focused **1/0**, 19-block release **2,162/0/11 ignored**, and
no-default **1,532/0/3 ignored**. Neither nonempty descendant preservation
nor later-slice A→B survival is proved; an earlier sealed-source ignored
native-fault rerun exited 101 at the child-CPU-custody assertion. A later
narrow Cut 3a quarantine passes its [owner-sealed release/no-default gate](../evidence/stage3-cpu-custody-quarantine-20261017/validation.receipt.md)
for the bounded fault and synthetic pre-entry shapes.
The checkpoint refusal's seven former ignores are now
[ordinary green tests](../evidence/stage3-checkpoint-unignored-20261017/validation.receipt.md)
in complete release and no-default suites. For runnable CPU slots, the
staging order remains callback-scoped Kernel/framebuffer port (step 2) before
the full custody/leaf-permit migration (step 3); a non-runnable Cut 3a
quarantine is the bounded exception.

O-T7 is now a [reviewable candidate contract](subsystems/tasks-scheduler.md#candidate-o-t7-slot-and-lease-contract-for-review-no-rust-api-exists),
not an implemented owner. Stage its connected source cut in this order:

1. Add focused pre-CPU pending-claim/return, same-process multi-slot,
   stale-generation/PID-reuse and live-custody checkpoint-refusal controls.
   Distinguish real CE zero-slice creation completion from retryable failure;
   neither may lose an already-published exact-birth launch.
   An exact-birth claim/return in Kernel's existing pending-launch queue may
   be implemented as a separate pre-CPU ownership cut. It does not confer a
   child run permit or make an admitted CPU safe on error.
   The bounded [P0 in-place queue ownership gate](subsystems/tasks-scheduler.md#preparatory-p0-in-place-pending-launch-ownership-cut-bounded-source-test-gate)
   now replaces the aggregate take/local-drain loop with exact-birth claims in
   Kernel's original queue. Its executed controls cover claim-time sibling
   visibility, nested batch order, stale/duplicate claims and same-slot pre-CPU
   retry. At P0's seal the admission set and order still included older event E
   and new B under A's tail; that was an ownership repair, not creator
   attribution or a new service trigger. During-run queue visibility and an
   active-claim checkpoint save were not independently executed in this cut.
   The [bounded P1 Event-service gate](subsystems/tasks-scheduler.md#p1-independent-event-service-trigger-bounded-source-test-gate)
   now tags each event-published entry with checked publisher provenance and
   offers one oldest Event in place at a neutral bounded host-loop turn,
   independent of `CreateProcessW`. Focused Kernel, real-MIPS, path,
   raw-kernel, retry and host-loop controls pass; the [P1 receipt](../evidence/stage3-event-trigger-p1-20261017/validation.receipt.md)
   seals 19-block no-default **1,558/0/2 ignored**, release-feature
   **2,210/0/9 ignored**, and three feature checks at exit 0. At that P1
   seal, nested P0 E+B batches could still drain siblings under A; it proved
   neither `NFY0:` return timing nor monitor/live behavior. The later
   [bounded combined monitor/P2 producer gate](subsystems/tasks-scheduler.md#exact-birth-pending-launch-producer-and-claim-gate-bounded-source-test-gate)
   carries an attempt-local exact publication witness through the import
   boundary and claims only B after selected-context preflight. Its real-MIPS
   `CreateProcessW` and original direct-`.exe` Shell controls leave older E
   queued; Kernel B1/B2 and same-birth bad-PE retry controls pass. An ordinary
   monitor `continue` offers Event work, while `stop_pc` deliberately leaves
   it queued. The evidenced Shell boundary is B loader/admission before A's
   next guest instruction, not required B user-code execution. None of these
   bounded cuts settles O-T7, the callback-scoped port, checked post-B
   restoration, runnable CPU custody or live iNavi acceptance.
2. Complete callback-scoped machine-state authority before claiming a sound
   lease. Successful `CreateProcessW` stops A's native attempt at its staged
   return PC and runs B post-native. The later
   [connected port/custody cut](../evidence/stage3-connected-port-20261017/validation.receipt.md)
   removes the named raw `kernel_ptr`/`framebuffer_ptr` aliases and holds no
   mutable Kernel/framebuffer loan across `emu_start` or B's nested run in
   the reviewed production path. Its process-scoped entry check and
   callback-scoped short loans are not an independently issued exact
   executing-thread/leaf permit. Add that authority, checked selected-process
   restoration and an exit-transaction event fence. A second guest-executable
   permit or a panicking `RefCell` conflict is a failed control, not a yield.
   A [test-only real-MIPS fixed-stack witness](../evidence/stage3-live-pump-fixed-stack-hook-20261017/validation.receipt.md)
   covers the live-pump memory-fault route in both explicit-PC/global-hook
   and default hookless/count-limited modes. It identifies the callback path
   by source exclusivity rather than a callback counter; no production raw
   pointer, reentry or custody boundary has moved. Use it as a narrow
   fixture for the port, not as Step 2 acceptance.
   The [bounded scheduler-block fence](../evidence/stage3-scheduler-block-fence-20261017/validation.receipt.md)
   routes that callback's raw Kernel/framebuffer uses through a guarded
   adapter. Its real-MIPS **preseeded** busy control stops with a typed error;
   full release/no-default suites pass on the sealed source. Production's
   flag only excludes scheduler-block self-reentry, not another hook family,
   and no natural nested call was exercised. The current single native entry
   runs the import and block hooks sequentially, so do not manufacture a
   shared-flag import/block gate. This does not promote Step 2; audit reachable
   raw-alias and host-callback loans. The connected cut later removes the
   named aliases, but modal partial-effects and exact leaf authority remain.

   The historical pre-port Step 2 inventory below is for `unicorn.rs` SHA-256
   `FB855EEB177A42335A7EAC4BC9CDB56FE987E016EDFBA4E6D8149E837D92E381`.
   Counting whole-identifier use-lines after declarations at `:10204–10205`
   and before tests gives **112 `kernel_ptr`** and **9 `framebuffer_ptr`**
   lines, not 123 independent loans or runtime calls. Global code hooks
   (`:10321–10760`) account for 4/1, sparse code hooks (`:10803–10980`)
   5/1, scheduler block (`:11375–11842`) 1/0, import
   (`:11888–14327`) 101/7, and memory fault (`:14490–14533`) 1/0.
   The scheduler line only passes the pointer into its guarded adapter;
   that adapter still dereferences its raw Kernel/framebuffer fields at
   `:251–268`. Auxiliary trampoline/trace hooks have no named-alias uses.

   | Reentry boundary | State lifetime and scope at the historical pre-port seal |
   | --- | --- |
   | Native callbacks | The public/private wrapper `unicorn.rs:9346–9692` retains mutable Kernel/framebuffer references through `emu_start` at `:15051–15057`; code, import and memory callbacks still use raw aliases. The scheduler-block adapter is callback-local only. |
   | Post-native child and Event service | The ordinary child helper `unicorn.rs:20925–21424` holds real mutable state through B's guest run at `:21147–21160`, after A's native return but before its owner finishes. The neutral host-loop and monitor Event paths (`main.rs:393–441`, `:2783–2825`) enter that helper from closed native turns and also pass mutable state through the child run. |
   | Live host loop and monitor | Ordinary slice entry `main.rs:918–930` and monitor slice entry `:2852` reach the same long-lived native boundary. Rotation, input, status, monitor handoff and checkpoint calls are separate host-turn operations, not loans spanning `emu_start`. |
   | Exit/finalizer | Host-turn quarantine exit (`main.rs:377–384`; `unicorn.rs:6284–6389`) and stopped pseudo-exit after native finish (`unicorn.rs:6973–7042`) do not enter a guest engine, but Kernel/window cleanup and partial-effect ordering still need the exit fence. |
   | Framebuffer present | `main.rs:3704–3719` ordinary present is a separate closed-turn call. Live remote/Win32 framebuffer wrappers (`:3738–3815`, `:3861–4019`) retain framebuffer/presenter state for the guest attempt; `mark_dirty`/`emulator_tick` may publish, pump or blit during it. No guest reentry from Win32 pump is source-proven. |

   This inventory distinguishes short closed-turn accesses from state held
   across guest execution; it is not a port, alias-safety proof or runtime
   frequency measurement. The real-MIPS A→B→C census shows that the
   post-native child path is reachable, while a synthetic busy-flag test
   alone cannot prove natural cross-hook overlap.

   The later [native-entry precondition](../evidence/stage3-native-entry-precondition-20261017/validation.receipt.md)
   refuses a second same-host-thread guest-native entry at the start of the
   private attempt, before saved-CPU setup. Its preseeded-busy real-MIPS
   control verifies typed refusal and same-CPU retry; complete sealed release
   and no-default suites pass. At that seal it did not remove the outer
   mutable Kernel/framebuffer references across `emu_start`, protect another
   host thread, or give the machine an exact leaf permit. The later connected
   port removes the named raw aliases and spans a child's post-native
   first-touch fault, but still does not confer exact leaf authority or full
   Step 3. Cut 3a's narrow CPU quarantine alone never passed that port.
   A later [test-only nested A→B→C observation](../evidence/stage3-post-child-selection-observation-20261017/validation.receipt.md)
   found C selected and occupying the returned child CPU slot at B's
   post-child boundary. A B-only restore preflight is therefore not a safe
   preparatory shortcut; checked restoration and rotated-subtree custody
   belong in O-T7's connected slot/leaf cut.
   The historical [follow-on census](../evidence/stage3-nested-cpu-census-20261017/validation.receipt.md)
   preserved A/B/C payloads but reached typed `NoAttempt` on the next call
   because B was selected with C's CPU. The connected cut stops earlier and
   retains the returned C-root/B-parked tree as unverified; it still cannot
   execute B's next slice. O-T7 acceptance must turn that into an executed
   exact-owner guest slice, not suppress the preflight.
   The [proposed port contract](subsystems/tasks-scheduler.md#callback-scoped-machine-state-port-cut-proposed-not-implemented)
   specifies one borrowed Kernel/framebuffer port for the native attempt,
   fallible short loans tied to exact attempt/leaf authority, and a checked
   selected-process/parent-suspension token. The bounded connected cut now
   wires a process-scoped port through the public run wrapper, private native
   attempt, **all** code/block/import/memory hooks and post-native child
   helper, with typed busy refusal and no Kernel/framebuffer guard across
   `emu_start` or B's nested run. This is not the full proposed contract:
   entry verifies selected process birth, not the final executing thread;
   callback thread/slice token values are self-compared rather than issued by
   an independent leaf owner. Prove framebuffer callback conflicts where
   reachable, exact selected-birth/PID reuse, checked post-B restoration,
   exit-event fencing and hook finish/abort lifetime. Any conflict must stop
   with a typed terminal outcome, never guest API success or a scheduling
   yield. Independent review and executed feature-appropriate suites support
   this bounded cut; O-T7 runnable custody remains open.
3. Admit boot and child CPU payloads into one machine-owned slot directory;
   stage this rather than converting every parked path at once. A narrow
   [Cut 3a quarantine](subsystems/tasks-scheduler.md#cut-3a-quarantine-resource-lifetime-bounded-source-test-gate)
   now precedes complete Step 2 because it never runs a custody-held CPU:
   the main run loop owns exact-birth, generation-checked slots only for
   faulted, non-runnable admitted children. Earlier focused real-MIPS
   debug/release controls passed; superseded producer-side a09 debug confirmed A's staged
   return survives native sync/finish, B executes
   fixed-stack first touch then faults, and its original CPU is retained in
   the slot rather than the legacy parked tree. Pre-CPU launch errors retry;
   native-error quarantine never fabricates a resumable context. A termination
   request alone retains B; same-Kernel accepted exit retires its slot and
   fixed-stack pin in that focused fault shape. Stamped a07 expected red
   reproduced a synthetic post-load/pre-native-entry unpoisoned quarantine;
   producer-side sealing passed a08 debug pre-entry and a09 debug real-MIPS
   fault on superseded `unicorn.rs` SHA-256 `1AAE8A66…33090`. Intermediate
   producer-only a10 focused release passed but is superseded. Current source
   moves the seal to `CpuCustody::quarantine_reserved` under Unicorn on the
   `Reserved`→`Quarantined` transition, retaining the exit consumer's poison
   backstop. The [owner-sealed receipt](../evidence/stage3-cpu-custody-quarantine-20261017/validation.receipt.md)
   passes a11 focused release **1/0**, 19-block a12 release
   **2,184/0/9 ignored**, a13 no-default **1,543/0/2 ignored**, and three
   feature checks, with six matching source hashes. Natural reachability
   and general error-path closure remain unproved.
   A later [connected source/test gate](../evidence/stage3-connected-port-20261017/validation.receipt.md)
   preallocates an unbound deposit ticket and private whole-tree holding cell
   before a `Some(custody)` child runs. Its ordinary B executes a second
   real-MIPS slice; a nested C-root/B-parked mismatch and tested early/late
   queue conflicts retain the original CPU tree as `UnverifiedStopped`, with
   zero abandoned unbound trees in those controls. A fixed-stack native fault
   also retains B. This is preservation, not a runnable B locator or checked
   parent-suspension transfer; at that connected seal, public `custody=None`
   and other classifier/exit paths remained outside the gate. The bounded
   release/no-default suites pass on the exact five-file seal, not the full
   Step 3 contract. The later bounded
   [no-custody admission fence](../evidence/stage3-no-custody-admission-20261017/validation.receipt.md)
   fails the tested public/direct pending-child path after local setup but
   before claim consumption, CPU admission or guest child execution. It
   retains the exact launch for machine-owned recovery while poisoning the
   public parent's attempt; no same-session retry, runnable slot, checked
   leaf authority or broader classifier/exit closure follows.
   Status/debug expose retained mapped-blob bytes, not
   measured RSS; checkpoint save/restore refuses custody. Pre-seal a03 broad
   **2,183/0/9 ignored** is development-only. Cut 3a alone could not claim a
   sound machine lease, raw-alias safety or general private VM/view cleanup.
   The later port removes named raw aliases, but not the leaf permit or
   runnable/parked slot ownership. Move those CPU paths one connected family
   at a time; add the leaf permit when the first runnable slot lands.
   For those later connected cuts, review private stable per-slot cells with short directory borrowing and
   closure-scoped fallible payload loans. Gate backend entry on the exact
   active leaf permit, and forbid slot retirement/reuse while any loan or
   permit exists; a private `Rc` clone is not that gate. A checked
   `ParentSuspension` transfers the one executable leaf permit A→B while A's
   staged return context is pinned; a different nested path may still pause
   A's native callback. B is classified into custody before A may resume.
   Keep a claimed launch in Kernel's original queue until admission/checked
   completion, and reserve
   slot capacity before moving a constructed payload. Convert recursive
   parked CPU boxes and active/displaced/ready references to checked locators
   together. Migrate the post-native parent/child lease, any remaining nested
   callback path, `Completed`, run-state drain/return, process switch
   and both main-loop/CPU-loop selector call sites before claiming custody.
   Keep existing scheduling policy pending Stage 5; do not add a parallel
   pending-owner vector or use a queue borrow as the resource owner.
4. Make every exit/prune path preflight exact-birth survivors and reserve the
   destination before effects, then move original locators infallibly after
   accepted exit. Native/load/publication failures retain non-runnable CPU
   custody; a late post-effect failure quarantines it and reports partial
   effects. The historical native-error red has a bounded green Cut 3a
   replacement under its owner-sealed release/no-default gate. The two stale
   held-borrow and real-MIPS A-exits/B-live fixture implementations were
   retired after they missed post-native preconditions; their historical
   evidence remains. A direct empty-descendant `Completed` control is now
   staged, while nonempty descendant custody, numeric duplicate pruning,
   later-slice framebuffer reentry and post-effect injection need separate
   controls.
5. Separately prove controlled first-slice `Exited` classification, B's later
   PC resumability and the live-pump later-slice deferred-launch/exit route. Keep
   live iNavi/frame/footprint acceptance behind a newly sealed build and its
   own writable-mount backup/run approval.

Direct WndProc dispatch, receiver-side cross-process result writes,
conditional system-origin device broadcasts, post-preflight whole-exit
atomicity, CE6 foreign-link/cancellation parity, full Stage 3 and
live iNavi remain separate gates. No live run is approved by these source gates.

Stage 7 budgets are a design gate, not values inferred from the historical
splash-only E-38 observation. Before calling memory or responsiveness "small"
or "fast," record the target device's configured CE RAM and real navigation
interaction expectations, define which guest/host memory quantities and which
input-response interval each budget covers, and review the limits with the
user. The hash-pinned, read-only [historical E-38 validator](../scripts/analysis/e38_validator.py)
([receipt](../evidence/stage7-historical-validator-20261016/A01.receipt.md))
checks 166 successful frame polls/four hash segments, 160 host memory samples
and 1,714 completed busy slices. Its RSS maximum is sampled, not the true
continuous peak; frame hashes do not prove image/map content, and current-build
live readiness remains unknown. A future exact lag claim needs lineage-linked
event IDs and stamps on one host-monotonic clock at named boundaries, such as
actual guest input dispatch and publication of a subsequent changed frame.
Until those links exist, the current two-second frame polling yields only a
conservative observation bound, which must be labelled as such. Use repeated,
phase- and input-matched runs for comparisons. The
[verification recipe](subsystems/verification.md)
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
Unmigrated production writable-image/DLL bytes, safe heap address reuse, full machine
checkpoint relineage, guest `CreateProcessW` handoff, and live footprint/map
readiness retain separate
gates; Stage 2 is not complete. Stage 3 extends, rather than recreates, the
process-birth authority.

### Remaining Stage 2 slices after the O-M2 and opt-in image source gates

Sparse migration was dropped: Microsoft's [CE `CreateFileMapping` reference](https://learn.microsoft.com/en-us/previous-versions/windows/embedded/ms923951%28v%3Dmsdn.10%29)
documents default `SEC_COMMIT` for the whole section. E-43 sealed immutable
main-EXE pages, E-44 migrated the initial heap/stack slabs and O-M2 replaced
copied heap spillover with sparse per-birth fixed pages. Production writable
image/DLL backing, full clone lineage, safe heap reuse and Stage 3 Task authority remain
separate gates.
The bounded [VM backing](../evidence/stage2-writable-image-vm-20261016/validation.receipt.md),
[opt-in main-image adoption](../evidence/stage2-writable-image-green-20261016/validation.receipt.md),
and [mutable-code fallback](../evidence/stage2-mutable-code-fallback-20261016/broad-validation.receipt.md)
source/test gates pass. The later
[second-run native-map failure](../evidence/stage2-writable-image-native-failure-20261016/validation.receipt.md)
and [legacy file-view overlap](../evidence/stage2-writable-image-file-view-20261016/broad-validation.receipt.md)
synthetic gates pass. The later [VM-allocation overlap gate](../evidence/stage2-writable-image-vm-allocation-20261016/validation.receipt.md)
also fences LegacySeed, same-birth stable/private/shared and heap allocation,
and native seed/reclaim/fault/map collisions, but normal SD-app loading
remains copy-backed. The later [synthetic exact-process-exit/retained-pin gate](../evidence/stage2-writable-image-exit-retire-20261016/validation.receipt.md)
retires only a verified birth's image runs after D-011 framebuffer preflight
and refuses writes through retained retired pins; its native control covers
the after-owner-close case, not exit while an engine remains mapped. The
next connected sequence is synthetic host-write/pin ownership and O-M3
physical-page accounting. Only after accounting is established may production
writable main-image integration proceed, followed by preloaded DLLs and
dynamic DLL load/unload/reload; do not treat byte persistence through the old
copy path as the ownership proof. Each cut needs
real-MIPS bytes, exact birth/module-generation pins, native copy/readback
exclusion and failure controls. The tested second-run `mem_map_ptr` collision
retains pins through checked abort-close; other direct host-write/restore/rebind
paths and exact module-retirement effects still need synthetic fences
before production activation. A native owner-close failure itself was not
injected. The [memory](subsystems/guest-memory.md) and
[loader](subsystems/pe-loader.md) plans detail those gates. [O-M3](DECISIONS.md)
keeps ROM/XIP versus RAM page provenance and `GlobalMemoryStatus` charge timing
open. The user chose to keep production images copy-backed until that
accounting is known; synthetic pinned controls are not production adoption.
A writable-mount run still needs a distinct approval and fresh verified
adjacent backups.

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
  `DestroyWindow` recursion. The later
  [pseudo-caller gate](../evidence/stage3-pseudo-caller-birth-20261015/validation.receipt.md)
  source-tests native COREDLL and decoded self-exit caller/owner preflight,
  but not immediate CPU stop. The later bounded
  [D-011 gate](../evidence/stage3-host-exit-framebuffer-preflight-20261015/validation.receipt.md)
  checks saved-backing restore feasibility before stopped-exit effects and
  propagates a typed recovery refusal. A selected-owner mismatch on the
  stopped boot CPU likewise requires typed authority recovery. Post-preflight
  whole-exit atomicity remains open. The bounded native pseudo self-exit gate
  above source-tests stop-before-next-instruction and exact child/boot Task/VM
  retirement after owner closure and preflight. The later decoded gate above
  covers the witnessed direct-kernel pseudo shape; raw direct pseudo
  compatibility remains separate. The bounded
  [modal-presence refusal gate](../evidence/stage3-modal-exit-failclosed-20261016/validation.receipt.md)
  protects its tested stopped-exit shapes; checked modal restoration remains
  a separate gate. Full exit-effect
  authority, live-app behavior and CE6 parity
  remain open.
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
