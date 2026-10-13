# E26 scheduler-core design record: one task, wait and continuation authority

Status: **plan for review only** (2026-10-14). No scheduler migration,
persistent-engine acceptance or splash diagnosis is claimed. The user
authorized beginning the rewrite, not bypassing the dependency gates in
[Architecture A1-A5](../ARCHITECTURE.md), [PLAN Stages 2-6 and 8](../PLAN.md)
and [D-003](../DECISIONS.md). This record names the connected production
slice to review **before** editing scheduler policy.

## Current-source pre-code control status

The historical E-8/E-11 fixtures are now ported to the D-007 sole source
checkout. Focused no-feature default runs compiled them and passed the one
active duplicate-close control with twelve expected-red controls ignored.
Three explicit `--ignored --nocapture` runs then failed all twelve at their
intended ownership, raw-API identity and execution-pin assertions; none
unexpectedly passed. The [port/compile receipt](../../evidence/e26-stage3-red-20261014/port-and-compile.receipt.md)
and [executed-red receipt](../../evidence/e26-stage3-red-20261014/executed-red.receipt.md)
retain the test/support hashes, full outputs and exit codes. Earlier
[E-8/E-11](ledger.md) results were private-mirror runs, not this D-source
baseline. This satisfies only the historical expected-red execution part of
the pre-code review gate: the additional controls below, Stage 2 outcome and
VM-owner contracts, independent review and rollback plan still matter. No
production scheduler migration or Stage 3 acceptance follows.
The separate [E28 outcome-contract design](e28-run-outcome-contract-design.md)
defines the pre-S3-A sealed-attempt and count-witness work; it is not yet
implemented or accepted.

## Existing authorities and the conflict to remove

| Current owner/consumer | Observed source boundary |
| --- | --- |
| Process and public handles | `HandleTable` owns a birth-qualified `ProcessBirthKey` and process aliases (`object.rs:239-253,686-799`), but `ThreadObject` is still a value behind public handles and contains no creator birth (`object.rs:216-225,636-650`). Boot TID 1 has no public thread record. `ThreadSystem` supplies TLS/LastError and a saturating numeric TID counter (`thread.rs:31-60`); it is not a task directory. |
| Kernel task projections | `CeKernel` separately owns selected bare PID, priority overrides, suspend counts, quantum maps and pending launches (`kernel.rs:312-377,2823-2878`). Child primary lookup can fall back to a recent trace after the launch queue drains. Launch creation has a checked-but-panicking PID increment, then separately allocates TID and handles (`kernel.rs:3014-3071`); worker creation and exit are handle-based (`kernel.rs:6481-6508`). These cannot be durable task identity or atomic creation. |
| Wait storage and hints | `Scheduler::blocked_waits` uses `TaskWaitStore` keyed by global TID plus wait ID (`scheduler.rs:98-133`, `task_wait.rs:1-99`); per-handle/message/serial/send queues and pending wake IDs are derived indexes (`scheduler.rs:179-483`). `next_wait_id` wraps (`scheduler.rs:188-190`). `CeKernel` registers, removes and traces waits, and separately owns pulse/comm-event bookkeeping (`kernel.rs:7255-7356`). Wait records do not yet pin canonical target objects or a specific continuation frame. |
| CPU execution and continuations | `UnicornMips` keeps a `ProcessNativeEngine`, a scalar `saved_context`, current/running TID, many pending-return vectors and distinct blocked/suspended containers (`unicorn.rs`, `UnicornMips` fields and run path). A `ParkedProcess` owns a boxed CPU and module/process selection metadata. `rotate_to_parked_process_id` removes/requeues entire CPU objects and rewrites Kernel selection. `ProcessNativeEngine::lease`/`finish` keep one owner and a CPU-only snapshot keyed by TID plus saved scalar context, but not a complete birth/task/frame key (`native_engine.rs` and the `UnicornMips` run path). |
| Selection and message delivery | `main.rs:354-1119` repeatedly asks CPU parked-process helpers and Kernel wake queries, then applies visibility, remote-input and service-boundary rotations. `should_rotate_parked_process` is another policy site (`main.rs:1195-1223`); `UnicornMips` parked-process runnable/visible/priority scans are another (`unicorn.rs`, near `rotate_to_parked_process_id`). GWE owns queue contents, while Kernel get/peek consumes messages and queues wake hints (`kernel.rs:8045-8282,8521-8733`). Cross-thread send/reply has a send ID and separate wait hints (`kernel.rs:8735-8842`). |

`TaskWaitStore` is useful transitional **record** ownership; it is not the
target TaskDirectory. Neither the parked deque nor the `SuspendedGuestThread`
queue name proves CE suspension. Today a selected PID, a public TID/handle,
a wait ID and a callback stack slot can all be mistaken for one another's
authority. No new parallel Task enum or second PID registry should be added.

## Proposed connected authority and identity

The machine event sequencer validates one action at a time. Extend the
**existing** machine-owned `HandleTable` process-birth directory with canonical
task and object records. An object has a canonical identity plus checked
incarnation; a public handle is only an alias. A task has immutable creation
process birth, public TID, checked task incarnation, durable primary/worker
relationship, base and
effective priority, suspend depth, lifecycle, quantum state and a stack of
frame identities. A frame is `(TaskIdentity, checked frame generation)` and
owns exactly one current continuation form: raw-native instruction state or a
validated synthesized import/callback return. Its active wait, if any, is
owned there. Nested callback frames may be runnable while an outer frame's
wait remains pending. Public handles are aliases; alias closure, active
execution and wait-target references hold independent lifetime pins. Exit
signals a tombstone; retirement waits for all pins.

Use the current `ProcessBirthKey` as the process component during migration,
never `kernel.current_process_id` or TID alone. Its current `(PID,
incarnation)` still lacks machine/checkpoint lineage; checked machine-level
clone/relineage must remap process, task, frame, wait, VM/view, pending-launch
and backend snapshot keys together. Until that operation exists, slab-bearing
checkpoint save **and** restore remain Unsupported; generic independent
`Clone` is not a checkpoint. Numeric PID/TID/handle/wait/frame generation
exhaustion must fail before publication, not wrap, saturate or overwrite an
active alias. A legacy fixture with unknown creator birth remains explicitly
unknown, never assigned the selected PID by convenience.

The Scheduler owns only *derived* ready, deadline and wake/subscription
indexes over canonical task/frame/wait references. The GWE retains message
bytes and queue order; the canonical frame owns the pending `GetMessage`,
`MsgWait`, synchronous-send or callback continuation. An event (object
signal, message arrival, send reply, timer, remote input) queues a typed wake
candidate; it does not itself remove a waiter, acquire an object, consume a
message or select a task. The machine revalidates birth/frame/target state,
readiness and effective priority at commit. Selection must not depend on an
old container's membership, a window being visible, or a guessed return PC.

Stage 3 must give each active task at least a canonical **frame identity shell**
before Stage 4 transfers full CPU/return payloads. Its first compatibility
adapter maps each existing `TaskWaitStore` wait ID to exactly one live frame and
birth-qualified target object pins. During migration, that store may carry the
old wait payload and derived subscription indexes, but it cannot own a second
wait lifetime or choose a replacement frame by TID. Every registration,
timeout, acquisition, cancellation, task exit and alias-close path for the
migrated process/thread wait targets must pass through this adapter; release
each pin once. A stale frame or target key rejects the wake without consuming
the waiter or acquiring its target. Stage 4 then moves the continuation payload
and removes the compatibility authority. Other object families are not claimed
as migrated until their corresponding readers and wait paths move together.

## Native lease and continuation boundary

The backend owns one process execution resource behind an opaque interface;
the scheduler holds a process key and frame/context handle, **not** a Unicorn
object. At most one task holds the active native lease. Before entry, validate
the task's creator birth and active frame, map the exact AddressSpace backing,
restore a raw-native context or construct a checked synthesized frame, then
write current PID/TID KData. After `emu_start`, capture full MIPS state and an
explicit outcome before changing task, wait or process state. A successful
count witness, voluntary yield, host-budget expiry, guest deadline, block,
debug stop, exit and backend error are distinct; `UnknownNormalStop` is not
a CE quantum-expiry proof (`types.rs:28-67`, `ARCHITECTURE.md` A3).

The Stage 3-6 implementation target remains **fresh engine per slice** under
D-003 and Stage 2 O-P4. The current source may retain an owner through
`ProcessNativeEngine`, and a boxed parked CPU carries that owner across
process swaps. An isolated Stage 3 identity slice may be exercised on that
current owner only as provisional authority evidence; it cannot claim the
fresh-lifetime acceptance gate or build policy around retention. At
Stage 8, a persistent per-process lease can replace the backend policy *only*
after native arena/hook lifetime, mapping pins, close errors and churn are
accepted; it must use the same birth/frame/run-outcome contract. A persistent
owner does not make a blocked task runnable or own wait state. An engine may
serve multiple tasks in one process but cannot be simultaneously leased by
two frames or restored under a foreign birth.

`DispatchMessageW` illustrates the frame boundary: `try_enter_dispatch_message_callout`
reads the guest MSG, saves the real MIPS `RA` and return SP, pushes a
`PendingWndProcReturn`, and enters guest WNDPROC with a return stub; the
`WNDPROC_RETURN_STUB_ADDR` handler later pops/matches and publishes the return
(`unicorn.rs`; these symbol anchors avoid moving line-number citations). The real-MIPS
`child_dispatch_message_wm_activate_returns_once_after_process_park_resume`
fixture (`unicorn.rs`, test by that name) parks the child mid-callback,
executes a parent instruction and resumes the child. In that first resumed
slice it asserts one `WM_ACTIVATE` return, the real `jalr` continuation,
`s1=0x1234` and no leftover pending return. A later ordinary scheduling
opportunity checks only owner-qualified child pending state, continuation PC
and `s1`; it does not prove another child instruction ran or that later
deliveries cannot repeat a callback. Three compiled earlier revisions failed
on superseded secondary assertions; the final test-only edit passed in the
E25 corrected-source full release suite (1,664 passed, 0 failed, 4 ignored;
[receipt](../../evidence/e25-private-slab-20261014/broad-validation.receipt.md)).
This is a bounded fixture result, not Stage 4 or live-app acceptance. Migration
must replace the pending vector and parked copy as authorities only when every
import/stub, escaped callout, send, GetMessage, modal and debugger consumer
reads the same frame.
The return-stub commit must match creator birth, task, frame generation and
validated return address/stack; an SP-only match or orphan repair is not
authority. A mismatch leaves the unrelated frame/pending return untouched and
reports a typed failure rather than borrowing another task's continuation.

## Prepare, commit, abort

1. Prepare task creation with checked IDs, creator birth, canonical object and
   entry/context candidates. A synchronous candidate can abort before any
   public alias or guest output is committed. A queued launch instead has two
   publication boundaries: atomically publish the canonical process/primary
   task in `Created/Initializing`, its public aliases/valid guest outputs and
   launch record; publish `Ready` only after image, stack and initial frame
   initialization succeed. A queued or half-loaded child is never Ready.
   If a *published* launch later fails, transition its canonical process/task
   to Failed/Exited, signal surviving aliases and release execution/launch
   pins by identity; do not erase already observed handles or pretend the
   earlier CreateProcess call failed retroactively. Completion must work even
   if the original handle numbers were closed. Preflight output pointers
   before publication; report any irreversible live partial write under A4.
   Closing an alias does not erase its executing or waited-on target.
2. Prepare a wait on the active frame: resolve and pin exact object identities,
   validate `WaitAny`/message parameters and guest output addresses, then
   register one canonical wait with a non-reused ID. Derived subscriptions may
   be rebuilt. If preparation fails, release pins and leave the caller frame
   unchanged. Timeout uses a registered guest deadline once Stage 6 owns the
   clock; the current wrapping `TickDeadline` is compatibility only
   (`task_wait.rs:9-37`).
3. On a wake, revalidate frame generation, target incarnation, suspension and
   readiness before acquisition. Preflight known output/message payloads
   before consuming a queue or object. A failed write or stale key must not
   publish a successful return, silently repost a consumed message or
   re-signal an object. If a live guest write was partial, report that effect
   and poison/abort the attempt rather than promise byte rollback. Cancellation
   or exit removes exactly that frame's wait and all derived subscriptions;
   outer same-task waits remain.
4. Select only Ready, unsuspended tasks at effective CE priority. Preserve
   remaining quantum on higher-priority preemption; rotate equal-priority
   tasks when an *established* quantum expires; quantum zero runs to
   completion. Sleep(0), finite-wait rounding and suspend/termination edge
   cases follow primary CE6 evidence summarized in
   [`tasks-scheduler.md`](../subsystems/tasks-scheduler.md), not the current
   fixed 100-ms shim or a visibility heuristic. OEM default quantum,
   inheritance and pending-suspend scope remain open O-T4/O-T6.
5. Commit outgoing context, wait acquisition/completion, guest byte sync and
   native lease result **before** marking a new Running task or publishing
   process exit, debugger stop, frame or screenshot. Native/map/hook/close
   failure is a typed backend error; do not rotate on an unknown or failed
   stop. If an irreversible effect already occurred, retain a poisoned frame
   with evidence instead of reconstructing a guessed continuation.

## Migration gates and acceptance

1. Freeze source-level consumers and expected reds: E-8/E-11 alias, primary,
   final-handle and registered-wait controls; add boot/worker/child birth,
   stale PID/TID/incarnation, checked exhaustion, exit after original-handle
   closure, wait-target acquisition after alias closure and independent-clone
   negatives. Review O-T1/O-T4 before changing production task state.
2. The smallest connected Stage 3 slice, S3-A, owns process/thread task
   identity, aliases, lifecycle and process/thread wait-target pins (not
   scheduler selection). Move boot, queued child primary and worker creation
   through the same directory; change `CeKernel::boot`,
   `queue_process_launch_with_options`, `create_guest_thread`,
   `mark_process_launch_exited`/`mark_guest_thread_exited`, HandleTable
   create/open/duplicate/close/signal/retire and the corresponding COREDLL and
   native create/return/exit consumers together. Route every process/thread
   waiter registration, removal and acquisition through the Stage 3
   compatibility-frame adapter, including completion after the original
   handle closes. Primary identity, priority and suspend reads for these tasks
   must resolve the canonical record; remove their fallback maps/trace reads
   as each connected consumer moves, never silently re-mint a task. Other
   object families follow as separately named connected slices, not as an
   inferred S3-A success. S3-A red controls cover alias coherence, late load
   failure, checked exhaustion, alias-close execution/wait pins, stale
   task/frame/target rejection and a **positive** independent machine clone
   with foreign-key rejection; without clone/relineage, Stage 3 acceptance
   remains open. Roll back S3-A production callers together if any mixed
   authority or publication failure appears, retaining the red fixtures.
3. Stage 4 moves one complete ordinary wait/frame family, then GetMessage,
   send/reply and WNDPROC callback families with nested frames. Run genuine
   MIPS wait→callback→resume/exit with FPU/HI/LO/branch state, cancellation,
   repeated same-TID waits, wrong-frame rejection and debugger exactness.
   Keep the E25-passing child `DispatchMessageW` park/resume fixture's
   first-resumed-slice exact-one return and owner-qualified later-state
   assertions green as migration proceeds. No orphan-repair
   fallback is accepted as proof of correct ownership.
4. Stage 5 replaces **both** `main.rs` rotation heuristics and the
   `UnicornMips` parked-deque runnable scans with one derived ready policy.
   Test effective/base priority, equal-priority RR, higher-priority
   preemption, blocked/suspended exclusion, quantum zero, Sleep(0), no
   starvation and distinct native outcomes. Stage 6 then unifies GuestInstant
   deadlines and live-first external event ordering. Stage 8 persistent
   engines are a later independent backend swap, not a prerequisite.

Before production scheduler changes: independent contract/source review,
executed red controls on the same source, the Stage 2 outcome contract and
canonical process-birth/VM-owner interface (not all Stage 2 public VM
acceptance), and an explicit rollback plan for the first connected Stage 3
consumer slice.
Before runtime acceptance: focused plus full release/no-feature checks,
several matched bounded iNavi runs with actual task/frame/outcome and input
consumption evidence, frame epochs, busy MIPS and host memory measurements.
The E-22 splash remained unchanged through the 330-s bound and logged 2,527
orphaned child `DispatchMessageW` returns ([app-evidence A-4/A-6](../app-evidence-lane.md));
that is a defect signal, **not proof that scheduler behavior causes the
splash stall**. Neither this fixture nor a green test suite establishes map
presentation or navigation readiness.
