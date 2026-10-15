# Tasks and scheduler

Status: target subsystem plan with bounded birth, alias, wait-pin, parked-continuation and resealed finite-Sleep Task/Frame source gates passed. Only finite Sleep has a canonical active frame; general Task/Frame dispatch, live acceptance and full Stage 3 acceptance remain open. The [architecture anchor](../ARCHITECTURE.md) defines shared identities, ownership, time, outcomes and snapshot contracts. The [migration plan](../PLAN.md) controls sequencing. This document uses those contracts; it does not define competing TaskIdentity, FrameIdentity, RunOutcome or SnapshotEpoch types.
The follow-on connected A→B Send/GetMessage work has a bounded A10 source/test
gate: release-feature 2,045/0/9 ignored and no-default 1,482/0/3 ignored
across 18 groups each, with unchanged pre/post source hashes. A09's four
same-process failures are historical; A10's exact-wait/birth recovery guard
passes the strict next-lease controls. The general Task/Frame and live-app
boundaries above remain open. The strict same-process plain S→T→S reciprocal
shape has a separate bounded
[A12 source/test gate](../../evidence/scheduler-source-cut-a12/validation.production.receipt.md)
with post-format release 2,047/0/9 ignored and no-default 1,482/0/3 ignored;
it is not general nested-send authority. The separate bounded
[A13 orphan source/test gate](../../evidence/scheduler-source-cut-a13/validation.final.receipt.md)
passed release-feature 2,052/0/9 ignored and no-default 1,482/0/3 ignored;
it does not convert GetMessage or modal waits into canonical frames.

## Responsibility and scope

CeMachine owns scheduling across all AddressSpaces. Its machine-owned HandleTable is the target single canonical directory: Stage 2 introduces process-birth records and process-handle aliases there, and Stage 3 extends that same authority for Tasks/objects. The scheduler selects canonical tasks and continuation frames; a process CPU/native engine is an execution resource, not a second Task registry. Kernel shims and native hooks request transitions through that authority.

In scope: authentic task birth/exit, runnable state, explicit suspension, waits, nested continuation ownership, CE priority and quantum accounting. Out of scope for this first plan: heap reuse, app-specific readiness/visibility rules, a new HLE syscall boundary, or assumed cycle timing. Clock conversion is coordinated with [clock and timers](clock-timers.md); wait-target lifetimes with [kernel objects](kernel-objects.md).

## Current implementation, not target guarantees

- [ThreadSystem](../../src/ce/thread.rs) owns TLS, LastError, kernel-mode flags and globally allocated numeric TIDs; it is not a task state manager.
- [HandleTable](../../src/ce/object.rs) stores canonical thread records with checked `ThreadBirthKey`, frozen production creation owner, public aliases, execution-live and registered-wait references. `ThreadRecord` now owns the mutable active finite-Sleep phase (`Blocked` or `Completing`) and checked frame generation. Its mutable `ThreadObject` still carries entry/parameter, priority, suspend count and exit/signaled fields; this is not yet a general Task/Frame dispatch directory.
- Its public-handle allocator advances unchecked; ThreadSystem TID and Kernel child PID allocators saturate. Checked incarnation alone cannot prevent an exhausted public handle number from colliding with an existing alias.
- [CeKernel](../../src/ce/kernel.rs) still has separate selected-PID, priority/suspend fallback and quantum maps. Boot PID1/TID1 has an aliasless canonical birth; child-primary identity is recorded at birth, but some primary/diagnostic fallback paths remain and full boot enumeration is unproved.
- Thread/process `DuplicateHandle` aliases now share canonical mutable records; O-K3 separately keeps execution-live births and exact registered Thread/Process wait pins after public-alias closure. The [E-26](../evidence/ledger.md) one-pass/twelve-ignored result and twelve explicit reds are historical pre-fix baselines. Later bounded [alias](../../evidence/stage3-alias-20261014/validation.receipt.md), [owner](../../evidence/stage3-owner-20261014/validation.receipt.md), [pin](../../evidence/stage3-pins-20261015/validation.receipt.md) and [wait-preparation](../../evidence/stage3-waitprep-20261015/validation.receipt.md) gates turned their named controls green, including the E-11 pin controls. Boot-main enumeration, full primary completion, public-ID exhaustion, machine clone relineage and generic object aliases are not thereby accepted.
- [Scheduler](../../src/ce/scheduler.rs) stores waits through one [TaskWaitStore](../../src/ce/task_wait.rs), keyed by exact `Birth(ThreadBirthKey)` or explicit `LegacyUnknownTid(u32)` compatibility owner. Finite-Sleep waits carry an immutable `TaskFrameRef` and select only the exact active blocked frame; other waits preserve their compatibility selection/subscription behavior. Wait IDs are locators, not a second authority.
- [UnicornMips](../../src/emulator/unicorn.rs) keeps a frame-addressed native register payload for finite Sleep alongside numerous compatibility pending return, blocked wait/GetMessage/modal/send/clipboard, suspended context and parked process records. SuspendedGuestThread slot/queue often contains **ready displaced contexts**, not explicit SuspendThread state.
- Persistent process engines are an experiment (ledger E-2) deferred to Stage 8; Task migration proceeds on the fresh-engine lifetime ([D-003](../DECISIONS.md)). Exact-scalar-key native snapshots are an initial migration, not full per-task native contexts.

### Bounded exact-birth wait-owner gate (source-tested)

One `TaskWaitStore` keys `Birth(ThreadBirthKey) | LegacyUnknownTid(u32)`; unknown is explicit fixture/compatibility ownership and wait IDs are only locators.
Production opaque/prepared publication checks captured live birth and rejects late failure as terminal `Backend`; exact exit and dialog-HWND cleanup preserve waits owned by a reused TID.
Controls include `scheduler_cancels_only_exact_birth_and_keeps_legacy_unknown_separate`, `prepared_wait_rejects_reused_owner_birth_and_releases_duplicate_pins`, and `opaque_wait_late_birth_rejection_is_terminal_without_guest_return_or_b_wait`; [receipt](../../evidence/stage3-wait-owner-20261015/validation.receipt.md): a02 release 1,947/0/9, no-feature 1,414/0/3; failed a01 retained.
No live-app acceptance: numeric GWE wake hints, native continuation payloads/pins, wait-ID wrap/reuse, checkpoint relineage, live modal behavior and full O-T1/O-T3 TaskDirectory/frame decisions remain open.

### Bounded finite-Sleep Task/Frame source gate (D-010)

Only a positive finite `Sleep` now has an active task-owned frame. `TaskFrameRef` is an immutable locator made from the exact `ThreadBirthKey`, a checked per-thread generation and the exact wait ID; it does not copy mutable phase. `ThreadRecord` alone moves that frame through `Blocked` and `Completing`. The scheduler wait and native register payload refer back to the same frame, and exact selection rejects a stale birth, generation, wait ID or already-completing frame. Native capture and publication are checked; native context restoration is checked before the matching frame, scheduler wait and pins retire. Exact cancellation/exit releases that frame without selecting a replacement by numeric TID.

The same-process guest `SendMessageW` path now leaves a send queued while its receiver owns a blocked finite-Sleep frame. The queued send retains that exact frame as immutable provenance even after Sleep retires. The sender uses the existing exact-birth scheduler/native SendMessage wait; it does **not** acquire a new canonical SendMessage frame, activate the sleeping receiver's WNDPROC, publish an early API result or use a cross-process rotation token. If S has no runnable work before T's deadline, the live host idles briefly and reports `NoAttempt` instead of replaying S's blocked import or advancing virtual time twice. Once Sleep returns, the pre-run active-callout selector still leaves this marked send queued; T's real `GetMessageW`, `PeekMessageW(PM_REMOVE)` or `GetMessageWNoWait` retrieval can enter WNDPROC and pass its result to S through the existing compatibility continuation. The two tested nonblocking retrieval paths then use a real finite `Sleep(1)` to yield before S resumes. Exact receiver-birth exit completes only sends marked for that birth with failure and leaves neighboring sends intact. A blocked `GetMessage` receiver retains its legitimate sent-callout bridge and is not classified as finite Sleep.

The formerly expected-red native controls `native_finite_sleep_defers_active_sent_callout_for_same_receiver` and `native_same_process_send_defers_wndproc_during_receiver_sleep` are source-green in the D: checkout. The real-MIPS `native_same_process_send_delivers_after_receiver_sleep_timeout` control also passes: one T `Sleep` and one S `SendMessageW` create one GWE send ID and one S wait; a pre-deadline call makes no native attempt; T resumes at its Sleep return PC without an early callout; T's `GetMessageW` enters the real WNDPROC; and S resumes once with its `0x77` result and no retained send/wait. The separate `native_same_process_peek_remove_delivers_after_receiver_sleep_timeout` and `native_same_process_get_message_no_wait_delivers_after_receiver_sleep_timeout` controls pass the corresponding nonblocking retrieval paths: each enters the deferred WNDPROC once, executes a real finite `Sleep(1)` after retrieval, and gives S `0x77` at its native SendMessage return PC. All four `native_same_process_` controls passed together. Kernel controls separately check that the marker survives Sleep retirement until queue take and that receiver exit replies to only its marked send. Structural controls include exact-frame selection and generation/wait-ID exhaustion. The [resealed source gate](../../evidence/stage3-finite-sleep-frame-20261015/validation.receipt.md) passed full release 2,025/0/9 ignored, no-feature 1,472/0/3 ignored, alternate feature checks and a release build. It does not prove live-app behavior or full Stage 3. Reciprocal cross-Send after wake was untested at this D-010 seal; A11b later exposed a next-lease import replay, and the bounded A12 source/test gate repaired only its strict two-send shape. Neither is a live-app result. Post-wake MsgWait processing, other wait families and nested GetMessage caller-return remain unproved; O-T1, O-T3 and O-K3 remain globally open.

Two native KData failure controls cover suspended-peer activation after finite-Sleep capture/publication, once for a running worker and once for boot main. With KData unmapped, activation stops with typed `Backend` failure, cancels only the newly published frame and wait, preserves a sibling opaque wait, and leaves the original PC/RA/V0 intact; it does not invent a Sleep return. A separate stale-window control rejects an HWND still owned by exited birth A after numeric TID reuse by B: raw and queued Kernel sends do not borrow B's Sleep frame, while native `SendMessageW` and `SendMessageTimeout` stop before WNDPROC, send queue or register changes. B's live HWND remains sendable. These are focused failure and ownership gates, not coverage of every native activation or message route.

## Target ownership and state

Canonical TaskDirectory stores creation identity, lifecycle metadata, base/effective priority, suspend depth, quantum state and frame ownership once. Public thread handles resolve to canonical identity; Kernel cannot retain writable copies as another truth. Boot registration need not publish a new guest-visible handle. Creation identity survives launch-queue drain, trace eviction, process rotation and external-handle closure.

The first cut consumes the durable boot/child ProcessIdentity from Stage 2's canonical birth directory, then adds the primary TaskKey for launch completion; it must not create another process registry. The current completion path signals the original launch process/thread handle numbers; either may already have been closed. Completion and worker cleanup must resolve canonical identities, not recover ownership from a public alias or diagnostic ring. The exact process-worker sweep remains open (O-K5).

The shared execution-state contract must distinguish:

- Running: the task owns the machine's active native execution boundary.
- Ready: a validated entry or resumable active frame exists, with no blocking/suspension barrier.
- Blocked: the active execution frame owns an incomplete wait; retained outer waits alone do not imply Blocked.
- Suspended: genuine suspend depth prevents execution while preserving underlying wait/ready status and pending completion.
- Exited: terminal execution is completed; surviving aliases/wait references can retain an object tombstone.

These are semantics, not an additional mirrored enum. Creation-not-yet-initialized and termination-in-progress need explicit lifecycle representation before projection into the shared states. A queued process without a loaded entry is not Ready. Resume of the final suspend does not universally turn an incomplete wait Ready.

Each nested callback/wait/caller-return frame has its own FrameIdentity and checked continuation version. Task identity is not a wait ID; numeric TID equality is not permission to consume a replacement frame. Frames pin their canonical wait-target identities and context/payload dependencies. No fixed last-snapshot-per-TID slot can replace parent continuations during same-task reentrancy.

## Interfaces and error boundaries

Proposed logical operations, not frozen Rust signatures:

- Register boot/child/worker identity at authentic creation; publish entry readiness only after initialization succeeds; abort an unpublished creation candidate on failed output/setup.
- Allocate task/process generations and public PID/TID/handle numbers with checked failure before publication; no wrap, saturation reuse or overwritten alias can create a phantom task.
- Resolve thread handles/pseudo handles to canonical task identity using the authentic caller. Unknown legacy fixtures remain explicitly unknown, never assigned a guessed PID.
- Prepare, publish, complete and retire waits/frames through one identity-checked boundary. The first cut must settle O-K3: active execution, public aliases and registered wait targets independently retain canonical state; cancel/failed preparation releases each pin exactly once. Subscription indexes and wake hints are derived references, not record owners.
- Select from eligible canonical tasks; activate a validated frame and native context; account its actual RunOutcome; retain exact debugger stops without ordinary scheduler maintenance.
- Exit/terminate and release execution references; retain alias/wait lifetimes according to kernel-object rules. Checkpoint clone duplicates mutable task/index state, sharing only approved immutable snapshots.

Fallible native context capture/restore, destination writes and identity lookup must be preflighted before irreversible queue removal/object acquisition. If later failure cannot be rolled back, fail the attempt honestly; do not invent object re-signaling, queue re-posting or successful API results. Unavailable native reads remain unavailable, not zero. SnapshotEpoch accompanies observed state; cached debug text never creates scheduler authority.

## CE6 evidence contract

Primary evidence is local `C:/WINCE600/PRIVATE/WINCEOS/COREOS/NK/KERNEL`:

- `thread.c` InitThreadStruct 264–282 binds owner/active process and OEM quantum; THRDCreate begins suspended/blocked354–355; NKCreateThread 926–1009 binds pActvProc, creates, duplicates and resumes unless CREATE_SUSPENDED.
- `kcalls.c` MakeRun 62–134 checks suspend depth and compares effective priorities. Lower numeric CE priority wins.
- KCNextThread 1198–1298 rotates an expired equal-priority current task to the tail; higher-priority preemption preserves remaining quantum/head placement unless expired; quantum 0 is run-to-completion. Sleep(0) yields among eligible priorities, not a turn for lower priority while a higher-priority task remains ready.
- `thread.c` THRDGetPrio256 reports base priority; THRDSetPrio256735–752 validates 0..255 and has trusted realtime-band restrictions. Scheduler ordering uses effective priority; inheritance/ceiling paths must not be erased into base priority.
- ThreadSuspend 730–828 includes late/pending suspension in kernel/PSL; ThreadResume 838–857 requires final suspend removal and appropriate wait/proxy conditions. Current shim counters are not full parity.

The default quantum is OEM-configured, not an assumed 10 ms or the current shim's fixed 100 ms. No visibility-based priority, guessed worker readiness or new starvation heuristic is part of the target.

## Actual migration consumers

Canonical lifecycle and handle migration must cover these existing families together:

1. CeKernel::boot, queue_process_launch_with_options (and wrappers), create_guest_thread; coredll create_thread_raw/CreateProcess/shell launch; native try_enter_create_thread_callout.
2. HandleTable thread getters/setters, snapshots, start eligibility, duplicate/close, exit, suspend/resume, priority and all wait-readiness/acquisition methods.
3. Kernel pseudo-handle/ID adapters, fallback maps and quantum APIs; IsPrimaryThread/Toolhelp; mark_guest_thread_exited and process launch/termination cleanup.
4. Native raw/worker ExitThread and CreateThread/ResumeThread return paths; explicit self-suspend; activate/push/pop ready contexts; thread-context get/set.
5. Ordinary Kernel/Sleep/MsgWait register/resume/remove/debug/context-set consumers, followed by GetMessage/send/modal/clipboard/callback families; process parking and checkpoint clone.

The first continuation family should migrate with **all** consumers: attach existing payload to its authoritative task-owned wait/frame, remove duplicate deadline/handle fields only when reads move, and preserve outer same-TID frames. Registration-only wrappers or a new unused model do not meet this gate.

## Migration and test gates

- Gate 1: extend the already source-gated process/thread birth, alias and O-K3 pin records into canonical Task/object lifecycle and primary completion, with checked TID/handle allocation. The D-source E-26 reds are a historical entry baseline; later gates turned their named alias, creation-owner and registered-wait pin assertions green. Boot-main enumeration and durable primary controls remain ignored, and public-ID exhaustion, complete clone relineage and generic object aliases remain open. Add genuine native last-public-handle execution, wait acquisition after alias closure, stale-key, allocator-exhaustion and child-exit-after-original-handle-close controls. This is a partial source gate, not full Stage 3 acceptance.
- Gate 2: ordinary blocked-wait and ready-continuation ownership, with genuine MIPS capture/restore and full caller return. The staged in-slice FPU regression is unexecuted; existing GPR handoff controls do not prove FPU/frame preservation.
- Gate 3: nested callback→outer wait resume, stale incarnation/frame rejection without consumption, explicit suspend while blocked, alias close during execution, process exit with workers and creation failure cleanup.
- Gate 4: effective/base priority, equal-priority quantum rotation, higher-priority preemption, quantum 0 and genuine idle eligibility. Only then enable the clock migration and retire compatibility heuristics.

### Bounded cut status and next gates

Anchor [A1-A3](../ARCHITECTURE.md) supplies the ownership rule: `HandleTable` remains the sole task-birth/lifetime directory, `Scheduler` derives wait and ready indexes from checked references, and the CPU backend stores opaque register payloads for an exact frame. Neither `CeKernel` nor `UnicornMips` may add a second mutable Task state. [D-010](../DECISIONS.md) implements this rule for **finite blocked Sleep publication, timeout and return**, including the one source-tested same-process sent-message path after Sleep. Sleep(0) yield and `Sleep(INFINITE)` self-suspend retain their compatibility paths; GetMessage, object waits, send waits, modal waits and process rotation are not converted to canonical frames. No CE6 clock/priority, broad TaskDirectory or persistent-engine claim follows. Fresh Unicorn owners remain the default.

The first source cut connects `try_block_sleep`, `CeKernel` publication/removal, `Scheduler::TaskWaitStore` selection, native resume and exact cancellation/exit to the task-owned active finite-Sleep frame. A `SchedulerBlockedWait` and `BlockedWaitThread` may still carry compatibility fields, but their finite-Sleep decisions require the same immutable `TaskFrameRef`; the native register snapshot remains a backend payload. A derived wake queue can locate that frame but cannot choose a different one by TID or old vector position. Other wait families have not been erased, and `SuspendedGuestThread` still does not mean CE SuspendThread state. Full consumer and clone-lineage coverage remain later gates.

The two formerly expected-red controls are source-green: one exercises the active sent-callout candidate while T has a real-MIPS finite Sleep; the other runs real-MIPS T `Sleep` followed by same-process S `SendMessageW` and proves T's WNDPROC is not entered early. Three further real-MIPS controls follow that **same** T/S scenario through the pre-deadline `NoAttempt`, T's timed Sleep return, T's `GetMessageW`-, `PeekMessageW(PM_REMOVE)`- or `GetMessageWNoWait`-initiated WNDPROC, and S's single `0x77` API return. The two nonblocking cases use a real finite `Sleep(1)` to yield S after retrieval. They establish eventual delivery for these paths, not a general continuation migration. A blocked `GetMessage` receiver is different: its legitimate sent-callout path may enter WNDPROC, remove the GetMessage wait and retain the parent in `ResumeImportAfterWndProc`. The separate real-MIPS `GetMessage` → sent WNDPROC → finite `Sleep` → WNDPROC return → parent import continuation control remains to be built, with exact birth, SP/RA/S0/HI/LO/FPU and one-return assertions. The continuation may re-enter GetMessage and reblock, so a real message or quit is needed before asserting a caller return. Local CE6 positive `NKSleep` blocks one thread; two concurrent finite Sleeps on that same CE thread are not a valid app-path proof.

The pre-cut a04 inventory found numeric-TID wait removal, scheduler selection without active-frame identity, and native wait/pin retirement before register restoration. The finite-Sleep path now requires exact frame identity, claims completion once, uses checked capture/restore, and retires the exact frame and wait only after its result/context boundary. Its deferred-send marker persists after that retirement so pre-run callout selection cannot bypass real `GetMessageW`, `PeekMessageW(PM_REMOVE)` or `GetMessageWNoWait` retrieval. Exact-birth receiver exit retires only marked sends for that birth and wakes their senders with a failure result. The focused KData activation failure and stale birth-owned HWND controls test exact cancellation and rejection before native side effects. The broader compatibility wait families still need their own checked transition and failure-path coverage; this cut must not be generalized to them by assumption. Keep the existing parked-Sleep birth gates and add or retain wrong-generation, reused-TID birth, alias-close, cancellation/exit and failed capture/restore negatives. Reciprocal cross-Send was outside this finite-Sleep cut; A12 later source-gated only its strict two-send shape, while post-wake MsgWait remains open. The finite-Sleep cut's broad suites and source seal are recorded in the receipt above.

#### Connected A→B bounded gate, then separate cross-Send boundaries

The bounded ordinary same-process A→B cut uses B's birth-verified blocked
`GetMessageW` source rather than the finite-Sleep deferral. Its real-MIPS
controls distinguish A and B by exact birth and disjoint native stacks,
exercise WNDPROC on B's captured stack across a native slice, and require one
result to resume A while retiring only its exact send/wait IDs. The pre-cut
in-hook shortcut instead derived callback SP from A's live register and
changed B's TID/KData label without a native-context transfer. It is not a
general cross-thread dispatch contract; cross-process calls and legacy
numeric-only HWNDs remain outside this bounded cut.

Before the repair, three ignored real-MIPS controls stopped with
`native staged context has stale or unverified thread birth` after verified
`GetMessageW` waits and A's activation
([expected-red receipt](../../evidence/stage3-getmessage-blocked-send-context-20261015/validation.receipt.md)).
Those receipts are historical pre-repair evidence: a03 recorded a native
count stop within B's WNDPROC, while the separate
[Kernel control](../../evidence/2026-10-15-ordinary-queued-send-receiver-exit-reuse-red.md)
showed B-prime taking B's ordinary queued send after B exited. They establish
neither a reciprocal cycle nor a live-app failure. A07 broad suites passed on
the prior connected source. A08's expected-red sender-restore fault exposed
reply/wait/result corruption, and its repair passed focused Send 30/30 and
Kernel 1/1 on the prior A09 source. A09's sealed release-feature run then
failed four existing same-process controls (1,048 passed, 4 failed, 6 ignored);
no-default was not run
([A09 receipt](../../evidence/scheduler-source-cut-a09/validation.receipt.md)).
Strict extra-lease controls showed saved-context recovery reviving B while its
separate blocked-GetMessage slot still held the wait. The exact
[A10 source seal](../../evidence/scheduler-source-cut-a10/source-seal.md) and
[validation receipt](../../evidence/scheduler-source-cut-a10/validation.receipt.md)
cover a guard that checks the matching GetMessage/native wait and saved/live
birth before recovery, and rejects a known stale saved birth before restore.
Both 18-group suites passed: release-feature 2,045/0/9 ignored and no-default
1,482/0/3 ignored. The four formerly failing real-MIPS controls require B's
safe stop with A's reply/wait intact, then A's continuation on exactly the
next bounded lease. Only the runnable-receiver fixture checks a subsequent
no-replay slice; the three finite-Sleep controls stop at A's continuation.
This is a bounded source/test gate, not Stage 3 or live-iNavi acceptance.

The connected handoff contract, validated for the bounded A10 case only:

1. The common authority is A2's exact `ThreadBirthKey` and A3's checked
   native-frame provenance. Use an immutable receiver-source locator, not a
   second mutable task registry. A blocked `GetMessageW` source and a blocked
   outgoing `SendMessage` source have different continuation lifetimes:
   GetMessage's wait may transition into its existing
   `ResumeImportAfterWndProc` parent when the callback is entered; an
   outgoing Send wait remains parked beneath the incoming callback until its
   own reply arrives. Neither path may borrow a numeric TID's replacement
   context.
2. Prepare before mutating GWE or waits: verify the HWND's live birth owner,
   receiver source wait/frame and full saved PC/RA/SP/S0/HI/LO/FPU context;
   checked-capture the sender's complete context. A missing or mismatched
   receiver source is a typed failure, not permission to run its WNDPROC on
   the sender stack. `SentMessage` itself must freeze receiver birth at queue
   time from the HWND owner, including ordinary sends. A queued send must be
   invisible to a later incarnation reusing the numeric TID and must
   fail/reply once when its original receiver exits. Keep sender birth,
   receiver birth, source wait,
   new send and sender wait IDs in the return token. Legacy numeric-only
   windows remain an explicit fixture boundary, not an inferred production
   owner.
3. Commit one send and sender wait, then cross a checked native-context
   transfer boundary before WNDPROC entry. Restore the receiver's saved
   registers and KData; derive call SP from that receiver SP, and mark its
   birth verified only after the transfer. A TID label change or scalar
   register swap alone is insufficient: native stop/hidden branch state must
   have an authenticated source witness. On pre-entry failure, retire only the
   new send/wait/pending callback and restore the original sender context;
   preserve unrelated waits and sends.
4. On callback return, verify receiver birth, expected call SP, active send
   ID and exact sender wait/birth before completing the result and restoring
   the sender's complete context. Sent-message retrieval and callback
   completion must select by receiver birth and exact send ID, not whichever
   send is active for a reused numeric TID. Retire each send/wait once, avoid a second
   active-send-stack pop, and defer timeout result-pointer writes until the
   sender return is validated. A post-execution restore failure is terminal
   with partial-effects evidence, never a fabricated CE success.

For the ordinary blocked-GetMessage source, the bounded A10 path queues A's
send and blocks A, then uses the birth-verified
`try_resume_blocked_get_message_with_active_pc` bridge to restore B at the next
slice/selection boundary, not inside A's import hook. The bridge must select
the *target birth* and exact ready wait ID. The displaced-receiver control is
a regression gate, not proof that the old ready-ID lookup chose a wrong-first
record. Checked receiver register/KData restore must precede consumption of
the send and B's wait; the pre-run active-send callout must not bypass the
designated Get/Peek retrieval. A different-thread guest WNDPROC must not fall
back to A's stack. A receiver not currently blocked keeps the send queued
until its own Get/Peek retrieval. These are bounded source-tested behaviors and
contracts, not general CE6 or live-app acceptance.
This matches the documented CE-family retrieval boundary
([CE 5.0 SendMessage reference](https://learn.microsoft.com/en-us/previous-versions/windows/embedded/ms939980%28v%3Dmsdn.10%29));
the local real-MIPS and later CE6 app gates, not that older reference alone,
decide this implementation's acceptance. A blocked-outgoing-Send receiver is
not served by GetMessage's bridge and retains a separate source/continuation
gate.

The pre-entry native owner can restore A's *hidden* CPU branch/delay state
even with fresh-per-slice lifetime: the old native snapshot survives owner
close and may match A's next exact key. The bridge must not write B's scalar
registers on top of that A state. Select and birth-verify the receiver source
before execution; reset the native owner to blank (or restore B's exact native
snapshot), then checked-restore B's full scalar context and KData before
consuming the send or B's wait. A failed prepare leaves A parked and both
records intact with a typed backend error; a failure after irreversible
guest output must report partial effects rather than a fabricated result.
This is a default-lifetime requirement, not merely a persistent-engine risk.
An in-hook cross-birth bridge without a blank-state witness cannot borrow this
pre-entry proof. For runnable B, queueing A's send is only a first step: its
real-MIPS control must show B scheduled from its own context, reaching Get/Peek
without replaying A's blocked import through a process-rotation heuristic.
B blocked in an outgoing Send or MsgWait has a different receiver source and
remains a separate gate.

The A08 expected-red fault exposed the sender-side legacy return path
consuming its completed reply/removing its wait before a failed native restore.
The A10 source checks A's exact native context and KData before retiring only
its completed send/wait; its broad suites include the sender-restore controls.
On restore failure, A must stay parked, its exact reply
and wait must remain, and its result pointer must remain untouched; no CE
success may be fabricated. Callback return must be keyed to the exact send ID
and pop one active send only, never an ancestor.

The original three native A→B reds, Kernel exit/reuse red and GWE nested
double-completion red are historical controls now promoted into the connected
cut. A07 broad suites passed on earlier source.
A08's new negative was red and its repair passed focused checks; A09's failed
seal is historical. A10 passed the strict extra-lease and two broad suites on
one exact source/test seal. A10 alone did not establish a same-birth orphaned
GetMessage payload after its scheduler wait disappeared; A13 later covered
that narrow boundary. Hidden native state, restore failure, two-sender exact
completion, one-return/parent continuation and runnable-B remain bounded
controls, not general Task/Frame
acceptance. Cross-process parked-sender restore, legacy numeric-only HWNDs,
post-wake MsgWait and overloaded `SMF_TIMEOUT` last-error semantics remain
separate from this cut.

The next distinct gate began with the test-only
[A11b reciprocal expected red](../../evidence/scheduler-source-cut-a11-reciprocal-red/validation.next-lease.receipt.md)
and now has a bounded
[A12 source/test receipt](../../evidence/scheduler-source-cut-a12/validation.production.receipt.md).
The preliminary A11 same-call WNDPROC timing assertion was invalid; A11b
correctly allowed T's inner Send trap to stop safely before checking the next
lease.

Under A2/A3/A5, this fixture first captures S and T's exact
`ThreadBirthKey`s, distinct native stacks, and the original S→T send/wait IDs.
S remains blocked in its outgoing `SendMessage` when T's finite Sleep reaches
its deadline. T enters the original S→T send's `GetMessageW`/SentMessage
WNDPROC and calls `SendMessage` back to S from inside that callback. The
incoming callback must execute on S's own native context and stack while S's
outer send-wait continuation frame remains intact; the reverse result returns
exactly once to T inside its already-active WNDPROC. T then returns from that
WNDPROC to complete outer send ID 1; S's original API returns exactly once.
There is no second Get/Peek retrieval. Record both directions' send/wait IDs,
birth and frame provenance, PC/RA/SP and one-return counts at each step.
An absent native frame or stale sender/receiver birth must fail closed before
borrowing another thread's context or consuming the other send. A11b's first
lease left exact send IDs 1/2 and scheduler/native waits 2/3 intact; its next
bounded lease replayed T's inner Send at the same trap, creating send ID 3 and
duplicate wait 4. That remains historical source-red evidence, not a live-app
deadlock. A12's blank-lease transfer now restores S's captured outer context
before taking only inner ID 2. A birth/frame/parent preflight and exact
return token keep both waits and T's GetMessage parent intact. The unignored
real-MIPS positive control executes S's inner callback on S's stack, returns
`0x88` once to T, then T returns `0x77` once to S; both waits and GWE records
retire, S has the exact import return PC/RA and preserved SP/S0/HI/LO/FPR2/FCSR,
and a further slice creates no
third send. The paired wrong-parent-birth control requires a typed pre-entry
`Backend` with both original sends/waits and parent intact, no child token or
guest start. The formatted-source 18-group release/no-default suites passed
2,047/0/9 ignored and 1,482/0/3 ignored. This only covers one birth-owned,
same-process plain two-send pair. A third sender, timed/result-pointer Send,
second pending return/deeper nesting or cross-process pair makes the strict
handoff ineligible and can fail closed with terminal `Backend`; these are
live-run risks, not import replays or accepted CE behavior. Full Stage 3 and
live iNavi remain open.

The separate bounded
[A13 source/test receipt](../../evidence/scheduler-source-cut-a13/validation.final.receipt.md)
records the historical expected red and the source-only repair. A blocked
GetMessage payload now carries a required exact `ThreadBirthKey`; its presence
prevents saved-context recovery from promoting that same birth even after the
wait disappears. If the selected native import still has that payload but no
matching scheduler wait, a typed `Backend` stops before guest entry, preserving
the tested sender send/wait and blocked receiver payload. A stale payload from
an older birth cannot stop a replacement that reused its numeric TID. The
tested real button-tap modal path may retire its filtered GetMessage wait and
resume through the exact live modal wait; a missing modal wait fails closed
before consuming the result. Focused saved-context 7/7, integrated orphan
1/1 and modal 11/11 passed on the final source; the two 18-group suites passed
release-feature 2,052/0/9 ignored and no-default 1,482/0/3 ignored. This is
not general modal continuation, Task/Frame migration or live-app acceptance;
no mount or iNavi run was made.

### Bounded native pseudo self-exit cut (final source gate pending)

The current native COREDLL pseudo-handle import candidate captures the exact
caller birth and process lease, stages a nonzero return PC, and stops before
the next guest instruction. After the native owner closes, the stopped CPU's
caller and child-launch or boot witness are rechecked; D-011 saved-window
preflight precedes canonical child or boot Task/VM retirement. Completion
seals that CPU against another native attempt. A refused framebuffer or modal
backing retains the stopped owner and poisons re-entry rather than running a
compatibility continuation. This bounded shape is not general Task/Frame
authority, whole-exit atomicity after preflight, a conversion of the raw or
decoded pseudo-exit paths, or live iNavi acceptance.

Post-wake `MsgWaitForMultipleObjectsEx(QS_SENDMESSAGE)` is a separate fixture:
prove the sent edge wakes the exact T birth, then that real Get/Peek retrieval
enters WNDPROC and returns the one blocked sender's result. A wake bit alone
does not prove dispatch or continuation completion.

For finite Sleep, prepare/commit verifies the live `ThreadBirthKey`, checked generation and wait ID, and authentic MIPS context before publishing the frame/wait. A stale frame or birth cannot consume another wait or fabricate a guest return. Timeout selection claims that exact frame and validates/restores its context before retiring its wait and pins; a non-rollbackable native/map failure is a typed backend failure, not a CE success. Exit/cancellation removes only matching finite-Sleep frame state. Wait-ID wrapping, machine/checkpoint lineage and coherent GuestInstant remain separate gates, not presumed solved here.

The independent source audit identified two remaining policy sites: `main.rs` and `UnicornMips` both select/rotate parked processes. They are removed only after all runnable families read canonical Task/Frame state and Stage 5 priority/quantum outcomes are ready; replacing one heuristic alone is not an acceptance gate. The bounded source-green GetMessage/PeekMessage/NoWait lifecycles and focused failure guards do not close O-T1, O-T3 or O-K3, or authorize live-app acceptance.

Open decisions (TaskDirectory API and legacy fixtures, incarnation exhaustion, native-frame payload, termination/pending-suspend scope, priority inheritance, OEM quantum): [DECISIONS.md](../DECISIONS.md) O-T1 to O-T6. None is silently decided by this document.
