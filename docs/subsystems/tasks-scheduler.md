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

The bounded [deferred CreateProcessW source/test gate](../../evidence/stage3-createprocess-deferred-20261017/validation.receipt.md)
passed focused non-live **5/0**, two-mode file-view reseed **1/0**, and
19-block release-feature **2,161/0/13 ignored** and no-default **1,532/0/3
ignored** suites on `unicorn.rs` SHA-256 `4BE7F3F2…EAB69D`. This removes the
successful non-live immediate in-hook child run and stages the parent's
nonzero return PC before post-native launch in both modes. The file-view
control checks canonical host bytes, not a guest-visible post-lease read.
That deferred-launch gate alone did not provide CPU custody. A later narrow
Cut 3a non-runnable fault quarantine passes focused real-MIPS controls, while
callback-scoped machine-state access, runnable/parked custody, live-pump
later-slice survival, Stage 5 scheduling and live-app acceptance remain open.
The subsequent direct-MIPS first-slice `Completed` empty-descendant cut has a
[stamped red](../../evidence/stage3-completed-borrow-20261017/attempt01-direct-completed-expected-red.raw.log):
window restoration held a parent-queue read borrow and the old unconditional
borrow panicked. Current source skips an empty descendant publication and
uses fallible borrow/reservation for nonempty descendants; the latter can
still lose CPU custody after committed exit effects. The
[bounded receipt](../../evidence/stage3-completed-borrow-20261017/validation.receipt.md)
seals focused **1/0**, 19-block release **2,162/0/11 ignored**, and
no-default **1,532/0/3 ignored** on `unicorn.rs` SHA-256
`0DCF5C5D…136D06`. A sealed-source ignored native-fault rerun was red
(exit 101) at that earlier seal; the later Cut 3a focused replacement is
green. The old two ignored held-borrow/A→B fixtures were retired as
stale after missing post-deferral preconditions; their receipts remain.
The subsequent [test-only live-pump fixed-stack witness](../../evidence/stage3-live-pump-fixed-stack-hook-20261017/validation.receipt.md)
executes real-MIPS first-touch `SW`/`LW` in both explicit-PC/global-hook and
default hookless/count-limited modes. Its focused **2/0**, 19-block release
**2,164/0/11 ignored**, and no-default **1,532/0/3 ignored** gates confirm
the checked page state and guest progress on the production run path; source
exclusivity attributes the memory-fault hook, not a callback counter. It does
not replace the raw Kernel pointer, prove callback reentry safety or admit
machine-owned CPU slots.

## Responsibility and scope

CeMachine owns scheduling across all AddressSpaces. Its machine-owned HandleTable is the target single canonical directory: Stage 2 introduces process-birth records and process-handle aliases there, and Stage 3 extends that same authority for Tasks/objects. The scheduler selects canonical tasks and continuation frames; a process CPU/native engine is an execution resource, not a second Task registry. Kernel shims and native hooks request transitions through that authority.

In scope: authentic task birth/exit, runnable state, explicit suspension, waits, nested continuation ownership, CE priority and quantum accounting. Out of scope for this first plan: heap reuse, app-specific readiness/visibility rules, a new HLE syscall boundary, or assumed cycle timing. Clock conversion is coordinated with [clock and timers](clock-timers.md); wait-target lifetimes with [kernel objects](kernel-objects.md).

## Current implementation, not target guarantees

- [ThreadSystem](../../src/ce/thread.rs) owns TLS, LastError, kernel-mode flags and globally allocated numeric TIDs; it is not a task state manager.
- [HandleTable](../../src/ce/object.rs) stores canonical thread records with checked `ThreadBirthKey`, frozen production creation owner, public aliases, execution-live and registered-wait references. `ThreadRecord` now owns the mutable active finite-Sleep phase (`Blocked` or `Completing`) and checked frame generation. Its mutable `ThreadObject` still carries entry/parameter, priority, suspend count and exit/signaled fields; this is not yet a general Task/Frame dispatch directory.
- Its public-handle allocator advances unchecked; ThreadSystem TID and Kernel child PID allocators saturate. Checked incarnation alone cannot prevent an exhausted public handle number from colliding with an existing alias.
- [CeKernel](../../src/ce/kernel.rs) still has separate selected-PID, priority/suspend fallback and quantum maps. Boot PID1/TID1 has an aliasless canonical birth. The bounded [canonical-primary query gate](../../evidence/stage3-primary-canonical-green-20261017/validation.receipt.md) stores an exact child-primary birth on the canonical process, independent of launch/trace records. The later [v4 stopped-exit gate](../../evidence/stage3-primary-exit-birth-v4-20261017/validation.receipt.md) checks that exact primary and the stopped CPU witness for its tested child/boot paths; other exit reconstruction and full boot enumeration remain unproved.
- Thread/process `DuplicateHandle` aliases now share canonical mutable records; O-K3 separately keeps execution-live births and exact registered Thread/Process wait pins after public-alias closure. The [E-26](../evidence/ledger.md) one-pass/twelve-ignored result and twelve explicit reds are historical pre-fix baselines. Later bounded [alias](../../evidence/stage3-alias-20261014/validation.receipt.md), [owner](../../evidence/stage3-owner-20261014/validation.receipt.md), [pin](../../evidence/stage3-pins-20261015/validation.receipt.md) and [wait-preparation](../../evidence/stage3-waitprep-20261015/validation.receipt.md) gates turned their named controls green, including the E-11 pin controls. Boot-main enumeration, full primary completion, public-ID exhaustion, machine clone relineage and generic object aliases are not thereby accepted.
- [Scheduler](../../src/ce/scheduler.rs) stores waits through one [TaskWaitStore](../../src/ce/task_wait.rs), keyed by exact `Birth(ThreadBirthKey)` or explicit `LegacyUnknownTid(u32)` compatibility owner. Finite-Sleep waits carry an immutable `TaskFrameRef` and select only the exact active blocked frame; other waits preserve their compatibility selection/subscription behavior. Wait IDs are locators, not a second authority.
- [UnicornMips](../../src/emulator/unicorn.rs) keeps a frame-addressed native register payload for finite Sleep alongside numerous compatibility pending return, blocked wait/GetMessage/modal/send/clipboard, suspended context and parked process records. SuspendedGuestThread slot/queue often contains **ready displaced contexts**, not explicit SuspendThread state.
- In the current D: source, a successful guest `CreateProcessW` stages the parent's nonzero import-return PC and stops that native attempt in both non-live and live-pump modes. Child loading/running uses the post-native pending-launch path, not an immediate child run through the parent import hook's raw Kernel/framebuffer pointers. Normal non-live rotation may select the child before the parent executes another instruction. That [bounded source-path gate](../../evidence/stage3-createprocess-deferred-20261017/validation.receipt.md) alone did not provide machine-owned CPU custody. Cut 3a now retains an admitted, faulted child CPU non-runnable after native sync/finish; it does not provide a callback-scoped Kernel/Task port, runnable custody, Stage 5 scheduling or live-app acceptance.
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

- Gate 1: extend the already source-gated process/thread birth, alias and O-K3 pin records into canonical Task/object lifecycle and primary completion, with checked TID/handle allocation. The D-source E-26 reds are a historical entry baseline; later gates turned their named alias, creation-owner and registered-wait pin assertions green. Bounded canonical-primary query and exact-primary stopped-exit controls now pass, but boot-main Toolhelp enumeration, public-ID exhaustion, complete clone relineage, other exit consumers and generic object aliases remain open. Add genuine native last-public-handle execution, wait acquisition after alias closure, stale-key, allocator-exhaustion and child-exit-after-original-handle-close controls. This is a partial source gate, not full Stage 3 acceptance.
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

### Bounded native pseudo self-exit cut (source-tested)

The bounded [native COREDLL pseudo-handle import gate](../../evidence/stage3-native-pseudo-self-exit-20261016/validation.receipt.md)
captures the exact caller birth and process lease, stages a nonzero return PC,
and stops before
the next guest instruction. After the native owner closes, the stopped CPU's
caller and child-launch or boot witness are rechecked; D-011 saved-window
preflight precedes canonical child or boot Task/VM retirement. Completion
seals that CPU against another native attempt. A refused framebuffer or modal
backing retains the stopped owner and poisons re-entry rather than running a
compatibility continuation. At that earlier native seal, this bounded shape
was not general Task/Frame authority, whole-exit atomicity after preflight,
a conversion of the raw or decoded pseudo-exit paths, or live iNavi acceptance.

### Bounded decoded pseudo self-exit gate (source-tested)

The witnessed direct-kernel pseudo shape now uses the same stopped post-owner
finalizer as native self-exit. An attempt-local live JALR/delay-slot witness,
frozen caller/owner birth and exact PC-zero stop precede D-011 preflight and
Task/VM effects; a static decoded snapshot alone cannot complete a deferred
child. Positive child exit and stale/refusal controls passed the
[focused gate](../../evidence/stage3-raw-decoded-pseudo-exit-20261016/decoded-pseudo-exit.receipt.md)
and [broad offline seal](../../evidence/stage3-raw-decoded-pseudo-exit-20261016/broad-validation.receipt.md).
In Hookless/Sparse, new same-slice callsites without an installed exact hook
fail closed. Live modal restoration, raw direct COREDLL pseudo compatibility, whole-exit
atomicity and live iNavi remain separate gates.

### Exact-birth parked-CPU survivors on process exit (design and bounded cuts)

The parked CPU tree is a scheduler container, not a CE process-lifetime tree.
Removing an exiting process must not dispose an independently live foreign
process nested below it. The bounded natural-parked and requested-active
source cuts and the later synthetic first-slice `Exited` helper cut hoist
their tested nonmodal foreign CPUs; real-MIPS first-slice classification,
other active exits, requested parked siblings, already-exited skips and
generic duplicate pruning remain open. The requested `Completed` hoist is a
control, not proof that every exit path preserves descendants.

Before any Kernel, GWE, framebuffer, Task or VM exit effect, one read-only
preflight must partition the stopped CPU tree by exact process birth. Resolve
the exiting birth from its frozen launch witness or boot saved-primary-thread
witness, and verify each live CPU against its Kernel owner. An exiting node is
disposable; a foreign live node is a survivor with its original CPU context,
wait and nested children. Numeric PID, a mutable selected process, a closed
handle or nesting alone cannot authorize disposal. Unknown/stale ownership or
modal backing without a checked restoration path requires typed recovery
before effects. No CPU may be resumed to fabricate this proof.

On accepted completion, move (do not clone) surviving CPU nodes into the
nearest surviving queue at the removed node's position, preserving relative
order and exact birth. Publication after successful Kernel retirement must be
infallible, or the whole exit needs a separately proven rollback boundary;
the present D-011 preflight does not establish post-preflight atomicity.

The [preserved expected-red natural parked and active same-birth-duplicate
A→B controls](../../evidence/stage3-parked-survivor-red-20261016/validation.receipt.md)
gave B a distinct PC/SP, exact birth and legitimate blocked wait. The bounded
[combined source gate](../../evidence/stage3-survivor-resume-green-20261016/validation.receipt.md)
now hoists those tested nonmodal foreign CPU trees after exact-birth exit and
refuses stale/unknown ownership before effects. Its saved-context follow-up
requires the foreign CPU's current TID to be nonzero, match the saved
`ThreadBirthKey`, and still be execution-live. A retained handle or two
different live same-process TIDs do not suffice. Current-zero,
current/saved mismatch and exited-but-retained worker negatives, plus a live
representative-worker positive, passed **4/0** focused release controls.
Refusals retain the parent exit and original CPU boxes, contexts and waits.
The parked record's representative TID is a scheduler hint, not a captured
CPU birth; other numeric consumers need separate review. Sealed complete
suites passed release **2,144/0/10 ignored** and no-default **1,524/0/3
ignored** across 19 blocks each. The earlier passing a09 source and its a06
regression are superseded, not credited to this seal. Foreign CPUs without
saved context and PC resumability remain unproved; none of these source
tests establishes live-app behavior.

The bounded [first-slice `Exited` source gate](../../evidence/stage3-first-slice-survivor-green-20261016/validation.receipt.md)
uses a production-helper seam rather than guest MIPS. A→B success moves the
original nonmodal B CPU, context and wait after accepted A completion;
mismatched saved B ownership refuses before effects. When a deliberately
held parent queue borrow blocks publication, preflight refuses A's exit and
the fallback returns a typed terminal error without host panic, but loses the
local A+B CPU tree. This is not the infallible publication/recovery target
above: Stage 4 needs machine-owned child custody. Current framebuffer
implementations do not reenter that queue while its mutable borrow spans
Kernel completion, but the open trait does not prevent a custom one.
The later direct-MIPS empty-descendant `Completed` source cut skips that
borrow; nonempty descendant publication still has a terminal post-effect
ownership-loss path and no preservation control. First-slice primary-thread
exact live-birth authority remains unproved. Already-exited
skips, requested parked siblings and generic duplicate pruning need separate
gates, then a real-MIPS CreateProcess→parent-exit fixture. No one of these
tests is iNavi acceptance. The [CE6 source and REA finding](../../evidence/ce6-rea-exit-20261016/findings.md)
supports per-process exit ownership; the dumped ROM kernel's exit body is
still unidentified.
The shipped iNavi PE [REA callsite check](../../evidence/ce6-rea-inavi-20261016/findings.md)
independently identifies `FindWindowW`-guarded `happyway_win.exe` and
`iSearch.exe` creation branches that retain process/thread return words. It
is static evidence of real child-launch dependencies, not a new live run.
The [bounded CE6 wait/child-lifetime follow-up](../../evidence/ce6-rea-exit-20261016/findings.md#2026-10-16-bounded-waitchild-lifetime-follow-up)
corroborates the COREDLL wait/exit-code syscall wrappers. The local CE6
source additionally shows that process creation waits for loader completion,
not child lifetime; wait pins a process event, and termination request is not
accepted execution exit. An emulator public handle or parent queue therefore
cannot substitute for exact-birth CPU custody. The dumped kernel exit body
is still unidentified.

#### Next custody and reachability design gate (proposed, not implemented)

The [common anchor](../ARCHITECTURE.md) A1/A3 requires machine-owned custody
of every admitted live CPU resource. This is not a new Task authority or a
second pending-CPU vector. The canonical HandleTable remains lifecycle truth;
an execution-resource slot owns the opaque CPU box/context, binds it to exact
process birth and a checked owner generation, and gives Scheduler queues only
locators. Active and displaced context locators bind task/frame provenance at
lease/use; each readiness and exit decision revalidates canonical HandleTable
births rather than deriving Task life from custody membership. Do not assume
one slot per process: current same-process displaced contexts and nested parked
records need an ownership inventory before choosing slot cardinality. The CPU
backend still owns engine
internals. A run lease returns its resource to that same custody on each
non-retiring stop, including ordinary native errors, so a queue borrow cannot
drop the only child owner. Verified exit retires custody after preflight;
unexpected post-effect failure retains/quarantines it non-runnable.

The connected migration inventory is boot CPU, pending-launch admission,
first-slice child, nested parked descendants, `Completed`, process rotation,
same-process displaced contexts, native failure, stopped exit and checkpoint
clone. Before CPU admission, distinguish terminal CE creation failure (no CPU)
from retryable error (exact pending launch retained), and never orphan a child.
A source audit found that the current pending-launch drain has fallible steps
before CPU construction; because an already-returned guest `CreateProcessW`
cannot be undone as a failed parent call, such an error needs an explicit
retained-launch or child-completion outcome. It also found a then-
unconditional `Completed` parent-queue borrow; the later empty-descendant
source cut removes only that unnecessary borrow. Nonempty publication
conflict, post-construction load/native errors and numeric duplicate pruning
can still drop local or nested CPU boxes.
A post-admission failure retains the exact stopped CPU tree and Kernel live/wait
state. A successful A exit must reserve a checked survivor destination before
Kernel effects, then move original B custody without a fallible publication.
An already-exited A launch handle cannot authorize discarding a returned B
CPU; the resource's own exact birth is checked before any skip/retirement.
Unexpected post-effect failure is a terminal partial-effect report with CPU
custody retained, not an invented rollback. The event sequencer must prevent
framebuffer reentry from borrowing/mutating the same queue while the exit
transaction holds it. The read-only owner inventory has been independently
reviewed; [O-T7](../DECISIONS.md) keeps slot/lease and clone semantics open
until their exact failure policies and source gates are settled.
The current Kernel exit helper can retire window/host metadata before a later
completion check fails; this requires an effects-started, non-runnable
quarantine result rather than treating that case as a cleanly refused exit.
The bounded read-only exit audit found no safe ordinary `Result` failure seam
there today: `try_destroy_process_windows_with_backing_restore_for_birth`
destroys GWE windows/sends and restores framebuffer pixels before canonical
Task/VM completion, but `Framebuffer::mark_dirty` is infallible and the later
completion checks cannot be injected under the exclusive Kernel borrow.
A panic-only callback or unsafe reentry is not a valid expected-red control.
An explicit fallible, test-injectable commit boundary after a real effect is
needed to prove typed partial-effects quarantine; it is not implemented.
Until a checkpoint atomically relineages Kernel births, CPU slots/locators,
frames, waits and VM, live-custody checkpoint must fail closed instead of
duplicating an executable owner with the old birth.
For the current successful `CreateProcessW` path, the parent native attempt
stops at its staged import-return PC before the child runs post-native. The
target lease must still pin the parent's return context and CPU custody while
the child exclusively runs, then allow the parent to resume only after the
child is classified and in custody. Any other nested in-hook child path must
also release mutable parent queue/frame and machine-state loans before B runs.

##### Candidate O-T7 slot and lease contract (for review; no Rust API exists)

The full proposed machine-owned directory would hold each runnable/parked CPU
payload exactly once. Cut 3a implements only a `CpuCustody` holding directory
for faulted non-runnable children, not this slot/lease contract. A proposed
`CpuSlotKey` contains machine/checkpoint lineage, a slot
index and a checked generation; generation exhaustion fails closed rather than
wrapping. Each slot freezes its `ProcessBirthKey` at admission and retains the
CPU payload plus the process-image/launch metadata now carried by
`ParkedProcess`. A process may own several slots: rotation can displace a
same-PID CPU with a different thread context, and a parked CPU may itself
contain displaced contexts. The slot is resource custody, not a second
Process/Task record. Canonical HandleTable remains the only lifecycle authority.

One implementable *candidate*, not a selected Rust API, is a private directory
of stable `Rc<CpuSlot>` cells. A slot has immutable key/process-birth metadata,
a separately inspectable checked state header, and a `RefCell<CpuPayload>`;
only the directory publishes its copyable `CpuSlotKey`, while scoped internal
loans may hold a temporary private `Rc`. A non-executing
`with_cpu(key, closure)` operation resolves and clones the private cell under
a short, fallible directory borrow, releases that borrow, then uses
`try_borrow_mut` on **that slot only** for the closure. It does not
build a self-referential guard containing both an `Rc` and a `RefMut`, return an
unguarded `&mut CpuPayload`, or use a panicking `borrow_mut` for reentry. A
parent's payload loan may stay on the paused import-hook stack while a
different child slot is admitted and run; metadata/readiness queries must use
the separate header, not try to reborrow the paused parent payload. Attempted
same-slot reentry stops or defers with a typed conflict, never creates a
second executable loan. A separate slot-loan latch, checked with the key's
generation, must make `retire_exact` refuse removal/reuse while any payload
loan or run permit exists; cloning the private `Rc` alone does not ensure
that. The directory remains the durable registration/CPU owner after each
scoped loan ends.

Parked-tree edges and Scheduler ready/wait indexes would hold checked slot
locators, not CPU boxes. A slot has at most one structural parent/location;
distinct task/frame continuations within it may have separate Scheduler
references, but one `(slot, frame)` cannot acquire duplicate executable
membership. An active or displaced context locator carries the exact
`ThreadBirthKey`, optional `TaskFrameKey` and attempt generation in addition
to its slot key. Selection checks the slot generation and machine lineage,
the slot's exact live process birth, and the selected task/frame against the
canonical directory. A numeric PID/TID, process handle, representative parked
TID or queue position is never sufficient. Exactly one machine-wide run lease
may execute guest instructions; every admitted slot is reachable from custody
even when it is not runnable or its queue locator is being rewritten.

Executable access needs a distinct `with_run_cpu(&RunPermit, closure)` gate
that checks the permit is the active leaf for the exact slot/task/frame. A
general `with_cpu` loan cannot expose an unmediated backend `run` method;
the current public `UnicornMips::run_until_*` surface must be hidden behind
the machine gate in a connected implementation. A private `Rc` clone or
mutable payload reference never grants execution authority by itself.

The candidate machine run permit is a checked *leaf* token, distinct from a
backend native-engine lease. `suspend_parent_for_child` changes A's permit to
`SuspendedForChild` and issues a scoped `ParentSuspension`; B may then become
the one executable leaf. For current `CreateProcessW`, A's native attempt has
already stopped with its import-return PC staged; its CPU and return context
remain pinned until B is classified and the parent token resumes after exact
birth/frame revalidation. If another nested path pauses A's `emu_start` in a
callback, A still cannot execute another guest instruction while B owns the
leaf. Nested B→C repeats the transfer; no directory, parent queue or canonical
Task borrow spans the child run. A typed
execution/custody error or abandoned guard leaves the admitted slot
stopped/non-runnable in custody and
stops the parent instead of silently resuming either actor. Infallible
drop-time state fencing needs a non-borrowing slot/leaf latch; diagnostic
details and checked recovery remain explicit operations, not `Drop` success.

The proposed operations are `reserve_launch` (exact-birth pending claim and
capacity), `admit_cpu` (put the newly constructed payload in its slot before
fallible DLL load or native entry), `begin_run`/`finish_run` (validated
task/frame lease and typed stop), `suspend_parent_for_child` (exclusive token
transfer at the deferred `CreateProcessW` handoff or another nested run),
`prepare_exit_exact` and
`commit_exit` (preflight/reserve before Kernel effects, then infallible
original-locator splice), and `quarantine_exact`/`retire_exact` (never infer
retirement from queue absence). These names describe responsibilities, not
implemented methods or a promise about their Rust signatures. The pending
claim should stay represented in Kernel's existing launch queue with a checked
claim phase/token, not move into a second owner vector. Reserve slot capacity
before CPU construction, then make payload admission into that reservation
non-fallible; a failed pre-CPU claim releases the exact original launch for
retry without leaving a live but unreachable child. A real, evidenced CE
creation/zero-slice failure completes only that exact birth. Once admitted,
even a DLL-load, native, `Completed` publication or queue-borrow failure
leaves the original CPU/context and Kernel live/wait
state reachable, stopped and non-runnable until a checked recovery. D-011
pre-effect refusal remains a recovery hold. A failure after an irreversible
Kernel/GWE/framebuffer effect becomes an explicit partial-effects quarantine,
not a claimed rollback or successful process exit.

Accepted exit targets every CPU slot of the verified exiting birth and hoists
foreign-birth survivor locators at the removed structural position without
copying their contexts or waits. Duplicate pruning may remove redundant
locators for the **same** slot/frame; distinct slot keys are not disposable
because their public PID/TID sets overlap. In particular, an already-exited
launch A cannot dispose the returned CPU until that CPU's **own** slot birth
is verified as A: a first-slice handoff may have selected still-live B.

This is a connected ownership cut, not a first-slice-only queue patch.
`run_pending_process_launches`, import-return stop/staging and post-native
launch,
`Completed` and first-slice exit, `persist_run_state*`, all park/activate/
switch/rotate selectors, duplicate/exit pruning and survivor hoists, the
`main.rs` CPU loop and monitor checkpoint boundary must use the same slot
contract. The current `UnicornMips` recursively contains boxed
`ParkedProcess.cpu` owners, so that recursive owner field must become checked
locators as part of this cut. The two existing rotation-policy sites may
retain their present policy until Stage 5, but cannot bypass custody or the
canonical Task/Frame checks. A conventional long-lived `&mut CpuCustody`
borrow cannot span parent A's native hook and also admit/run B; whichever
stable per-slot interior or scoped guard representation is chosen must release
directory and parent queue/frame borrows before the nested run, use fallible
reentry checks rather than panicking `RefCell` borrows, and return every lease
to custody on all typed errors. An exposed cloneable pointer to a slot would
not itself confer execution authority.

The slot representation alone does **not** make current child execution a
sound whole-machine lease. `run_with_unicorn_attempt` still holds
`&mut CeKernel` across native entry and gives hooks a raw `kernel_ptr`; after
native return, `run_pending_process_launches` still passes a mutable Kernel
reference into the child run. The connected cut needs a single-thread,
callback-scoped
`MachineKernelPort`: each fallible `try_with_kernel` loan is short, carries the
current leaf/task authority, and ends before `emu_start`, a nested child run,
or a reentrant host callback. Multi-step operations retain exact claims and
checked versions rather than an open Kernel borrow. Parent/child selected
process compatibility state is restored by a checked scoped token, never used
as the CPU's birth witness. Framebuffer access needs the same scoped or
non-reentrant event-publication rule; callbacks during an exit transaction may
enqueue observations but cannot mutate Kernel/custody/topology under its
borrow. Reentry into these proposed machine-state/custody ports is typed and
fail-closed, not a `RefCell` panic or an unchecked raw-pointer alias; this
does not yet prove every existing pending-return `RefCell` safe. Until those
hook/Kernel/framebuffer loans are removed or explicitly proven at one
narrowly audited unsafe boundary, O-T7 cannot claim a sound machine lease
even if CPU owner-loss reds turn green. In any remaining nested in-hook path,
A's native owner may remain open while its callback is paused, but only the
checked leaf permit grants guest execution to B.

##### Cut 3a quarantine resource lifetime (bounded source-test gate)

The first CPU-custody production cut is narrower than wholesale parked-tree
conversion: the main run loop owns a directory that accepts
only exact-birth, stopped, **non-runnable** child payloads after a
post-admission non-recovery error. A native fault is the real-MIPS tested
subset; the post-load/pre-native-entry refusal is a synthetic control. This
is not a second runnable queue. A pre-CPU
DLL/PE/setup error still returns the original pending launch for retry;
[`run_pending_process_launches`](../../src/emulator/unicorn.rs) now reserves
exact-birth slot capacity before constructing B and separates pre-CPU retry
from post-admission non-recovery ownership. The original faulted B CPU moves
to quarantine, never the legacy parked tree. Every quarantined slot is
reported in status/debug, counted by checkpoint save/restore refusal, and
retired only after checked accepted exact-birth exit in the tested fault path;
a `TerminateProcess`
request alone is not completion. The accepted-exit witness carries birth only,
so production trusts the same Kernel instance used for preflight/completion;
it is not an independent machine-lineage proof. The leaf permit, runnable
locators and public-run
bypass removal belong to subsequent connected cuts. The callback-scoped
Kernel/framebuffer port remains separate Step 2 work; this quarantine does
not make the current raw aliases sound.

Under D-003's fresh-engine default, the quarantined payload and its release
obligations are:

| Resource after B's ordinary native fault | Lifetime / exact-birth exit path |
| --- | --- |
| B's original `UnicornMips` loader `MemoryMap`, mapped blobs, module/import metadata, `owned_process_launch`, pending/wait records and fault snapshot | The directory retains the same payload, poisoned and non-runnable. After [checked Kernel completion](../../src/ce/kernel.rs), the consumer retires B's exact slot and drops its CPU-local data; it preflights and hoists foreign-birth parked descendants rather than dropping their original CPUs. On preflight refusal the slot remains. Mapped-blob byte counts are observable but are not measured resident host memory. |
| Native Unicorn handle, hooks, mapping pins and resumable native snapshot | The [private-error abort](../../src/emulator/unicorn.rs#L15265) poisons and clears the saved continuation; [fresh-owner close](../../src/emulator/native_engine.rs#L680) destroys native maps/hooks before releasing callback boxes and pins, publishing no native snapshot. The quarantine therefore must not claim a live native engine or a recoverable PC. Opt-in persistent-owner semantics are not established by this proof. |
| Canonical B-birth fixed heap/stack pages, writable-image runs and heap state | These live in `CeKernel::memory`, not merely in B's CPU. [Accepted exact-birth completion](../../src/ce/kernel.rs#L3555) retires those three families after the execution pin is released. Dropping a CPU without that transition is not VM cleanup. |
| General private VM and file-mapping view registrations | No complete process-exit release path exists yet: [completion](../../src/ce/kernel.rs#L3555) does not retire these families, while explicit [UnmapViewOfFile](../../src/ce/coredll.rs#L31147) removes exact-birth public view records and `CloseHandle` [preserves live views](../../src/ce/kernel.rs#L6865). Normal public views are birth-keyed pointer-backed slabs; legacy injected/DIB views can have no birth. A first-slice fixed-stack fault with no such views may gate the narrow 3a ownership result, but full resource-release parity needs separate exact-birth cleanup and tests. The private-error abort also does not prove legacy file-view writeback immediately before a fault. |

The ordinary real-MIPS replacement witness now runs guest A's real
`CreateProcessW` with a staged return after native sync/finish, executes B's
fixed-stack `SW`/`LW` first touch followed by `READ_UNMAPPED`, and checks one
exact B-birth quarantined slot, zero B in the legacy parked tree, closed
native owner, live unsignaled B and no runnable continuation. A termination
request leaves that state intact; accepted exit retires the slot and fixed
stack in that tested path. Stamped a07 expected red reproduced a synthetic
post-load/pre-native-entry unpoisoned quarantine; a producer-side seal made
a08 debug pre-entry and a09 debug real-MIPS fault controls green on superseded
`unicorn.rs` SHA-256 `1AAE8A66…33090`. Intermediate producer-only a10 focused
release passed but is superseded. Current source seals the CPU inside
`CpuCustody::quarantine_reserved` under Unicorn on `Reserved`→`Quarantined`,
with the exit consumer's poison backstop retained. This forced precondition
does not prove natural reachability or general error-path closure. Earlier
focused debug/release fault controls passed. The
[owner-sealed receipt](../../evidence/stage3-cpu-custody-quarantine-20261017/validation.receipt.md)
passes a11 focused release **1/0**, 19-block a12 release
**2,184/0/9 ignored**, a13 no-default **1,543/0/2 ignored**, and three
feature checks, with six source hashes matching the seal. The final-source
real-MIPS control is counted in a12, not a separate focused final-source run.
Pre-seal a03 release broad **2,183/0/9 ignored** remains development-only.
Neither this witness nor the directory proves
the Step 2 port, complete view/VM cleanup, full O-T7 leaf authority, measured
RSS retention or live-app acceptance. No app or writable-mount run was made.

##### Parked-publication custody expected red (test-only)

The [bounded expected-red receipt](../../evidence/stage3-parked-publication-red-20261017/validation.receipt.md)
records real-MIPS A `CreateProcessW` followed by B's post-native first slice.
B executes fixed-stack `SW`/`LW`, with the stored value verified, then
classifies `Parked`. A test-only
framebuffer gate holds the parent parked-queue read borrow only during B's
publication. A02 returns typed `Backend`; B stays exact-birth Kernel-live,
unsignaled and wait-unready, but the parent queue is empty and the final
assertion finds zero B machine-custody slots rather than one. A01 missed the
fixture's intended expectation; a02 is the executed red. The artificial
borrow does not prove natural callback reentry, live-pump exposure or an
iNavi failure. Cut 3a quarantines non-runnable admitted-child errors only.
This witness supports the connected O-T7 contract above: after the Step 2
callback-scoped port, an admitted runnable/parked CPU must remain in machine
custody through fallible publication. Its execution requires the checked
leaf permit.
It does not justify adding a second queue owner or claiming a queue-only fix.
The same-source loopback-enabled release suite passed **2,184/0/10 ignored**
and no-default passed **1,543/0/2 ignored** across 19 blocks each; three
feature checks exited 0. The first sandboxed release attempt had 45 failures
concentrated in socket/remote tests; multiple traces show Windows `10013`.
Its identical-source loopback-enabled rerun passed.

##### Callback-scoped machine-state port cut (proposed, not implemented)

The first connected port cut changes the *whole native attempt's* access
boundary, not only the missing-page hook. A candidate `MachineKernelPort<'m>`
borrows the one Kernel and framebuffer together for the duration of an
attempt, with a fallible short-loan operation such as
`try_with_kernel(&AttemptAuthority, closure)` or
`try_with_state(&AttemptAuthority, closure)`. The closure receives a temporary
`&mut CeKernel` (and, for the joint form, `&mut dyn Framebuffer`); its
higher-ranked loan lifetime prevents either reference escaping. A shared
single-thread borrow cell can enforce this without creating another Kernel or
Task registry. The authority argument is a checked view of the current exact
process/thread/frame and attempt, not a replacement lifecycle record. A busy
loan, stale authority or unexpected reentry returns a typed conflict, stops
the native attempt and records the failure; it is never a CE API success or a
scheduler yield. All direct setup, callback and tail Kernel/framebuffer use
must go through the same port while it exists. Merely wrapping the current
long-lived `&mut CeKernel` while continuing to dereference `kernel_ptr` would
retain the alias defect.

An unconnected crate-private foundation module historically held the caller's
Kernel and framebuffer in one `RefCell` state, offered fallible higher-ranked
short loans, and latched the first typed terminal failure. It was removed
from active source after review and remains recoverable through Git and its
[development receipt](../../evidence/stage3-machine-port-foundation-20261017/validation.receipt.md).
[A01 focused no-default](../../evidence/stage3-machine-port-foundation-20261017/a01-focused-no-default.raw.log)
and [a02 focused release](../../evidence/stage3-machine-port-foundation-20261017/a02-focused-release.raw.log)
each passed **3/0** on unchanged source. [A03 broad release](../../evidence/stage3-machine-port-foundation-20261017/a03-broad-release.raw.log)
passed 19 blocks **2,192/0/10 ignored**; stamped source hashes stayed
stable while backup commits moved HEAD from `fa7de51a` to `a105546f`.
This is development validation, not Step 2 acceptance. The retired foundation
had no production consumer, attempt/leaf authority, callback teardown proof,
selected-process restoration or event fence; the a03 foundation seal did not
include broad no-default or feature-matrix checks. The target API
above includes authority that foundation never took. It did not move any
normal native attempt or hook across the boundary. The selected-context
stamp now has a narrower production pre-admission consumer below, not the
connected handoff/restoration consumer required here. The a03
suite predates retirement of the parked-publication red's test-only seam and
ignored fixture. Its preserved receipt remains evidence, but current source
has no active ignored fixture and the runnable/parked custody bug remains
open.

The connected signature changes, shown as responsibilities rather than
compiled Rust APIs, are:

| Current entry | Required private boundary |
| --- | --- |
| `run_until_import_trap_with_framebuffer_limits_report(&mut self, &mut CeKernel, &mut dyn Framebuffer, limits)` | Keep a compatibility outer entry temporarily, but construct one port and pass it to a private `run_until_import_trap_with_port(&mut self, &MachineKernelPort, &AttemptAuthority, limits)`. The caller cannot use its two mutable references again until native hooks have closed. |
| `run_with_unicorn` and `run_with_unicorn_attempt` with long-lived mutable Kernel/framebuffer arguments | Accept that same port and captured attempt authority. All code, scoped-code, block, import, pre/post-dispatch and memory-fault callbacks borrow through it, including the scheduler-block wait-resume path. One callback adapter converts a loan failure into `emu_stop` plus the attempt's typed terminal error; callbacks whose native signature returns `bool` must not silently turn conflict into `false` without the error record. |
| `run_pending_process_launches(parent_uc, &mut CeKernel, ..., &mut dyn Framebuffer, ...)` | Accept the port and checked parent authority, perform each preparation/selection/completion under separate short loans, release them before the child runs through the private port entry, then classify and restore through checked transitions. Do not call the public outer entry from a Kernel loan. |
| `repair_missing_virtual_page_for_fault(&mut CeKernel, ...)` and existing `&mut CeKernel` wait/import helpers | They may keep their local signatures only when called *inside* one short port closure. Preserve the fault's live exact-birth VM lookup and native map-before-canonical-pin commit; neither an eager snapshot nor a presumed PC restart is an equivalent replacement. |

`NativeEngineLease<'a>` currently accepts hook closures with lifetime `'a`
and destroys their callback boxes after native hook teardown/owner close. This
makes a borrowed port plausible, but the implementation must prove the port
outlives every installed callback on every finish/abort/error path; the
fresh-owner close proof does not automatically accept the opt-in persistent
mode. No port guard may span `emu_start`, nested child execution or a host
callback allowed to mutate machine state. A native mapping or framebuffer
operation performed during a short loan needs an explicit reentry audit: an
unexpected reentrant machine access fails closed, while permitted host
observations are queued outside Kernel/custody/topology mutation. An exit
transaction fences those events until the exact preflight/commit result is
known; an effect already applied before refusal requires partial-effects
quarantine, not invented rollback.

The deferred `CreateProcessW` handoff needs a checked selected-process token,
not the current blind save/set/restore of module fields and
`CurrentProcessState`. It captures the parent's exact birth, selected-state
version and staged nonzero return context, then selects B under a short loan
and relinquishes that loan before B's native entry. The parent's executable
leaf must be suspended while B owns the sole leaf; once O-T7 slots land, B
must be classified into durable custody before the token may resume A.
Restoration checks the original parent birth and selection lineage rather
than accepting a reused PID or whichever process was selected last. Token
drop may latch a stopped failure, but cannot claim successful restoration.
This token and the O-T7 `ParentSuspension` are one target handoff contract,
not two independent selectors. A port-only cut can validate selection and
non-overlapping native entry but cannot claim the slot/leaf/custody portion.

A preparatory crate-private `SelectedProcessContextStamp` now captures the
selected canonical `ProcessBirthKey` and a checked generation; capture returns
`None` without a registered birth. The nine selected-process metadata/state
setters and the two accepted exit compatibility writes advance the generation
before changing selected state. A failed exit preflight does not advance it.
Comparison re-resolves birth each time because the public `HandleTable` can
replace a PID without a setter. Overflow poisons capture permanently instead
of wrapping. Focused [no-default a01](../../evidence/stage3-selector-context-stamp-20261017/a01-focused-no-default.raw.log)
and [release a02](../../evidence/stage3-selector-context-stamp-20261017/a02-focused-release.raw.log)
each passed **5/0**; same-source [broad release a03](../../evidence/stage3-selector-context-stamp-20261017/a03-broad-release.raw.log)
passed 19 blocks **2,189/0/10 ignored**; broad no-default a04 passed 19
blocks **1,548/0/2 ignored**, and a05–a07 feature checks exited 0
([receipt](../../evidence/stage3-selector-context-stamp-20261017/validation.receipt.md)).
The bounded pre-admission cut below consumes this stamp before queue
drain, but the stamp is not a post-B restoration token and does not bind a
cloned Kernel to machine/checkpoint lineage. The candidate checks the live
owner/caller separately; the stamp itself does not prove execution-live
ownership. Direct mutation through public backing objects is not fully
stamped. The connected whole-attempt port, exact parent selection
restoration, leaf authority and exit event fence above remain open.

##### Deferred-launch selected-context pre-admission (bounded source/test gate)

Immediately before A's native `emu_start`, the current source captures its
selected-context stamp. If A stages deferred `CreateProcessW`, the post-native
tail checks that stamp against the selected exact parent birth, verifies the
stopped caller remains execution-live under that birth, and requires a
nonzero staged return PC. A mismatch returns typed `Backend` before the
then-aggregate pending-launch queue was drained or a child CPU was
constructed. In the tested canonical/private A path, the stopped refusal
poisons its saved continuation and requires recovery rather than rollback or
resume; aliasless synthetic nonprivate attempts do not establish that poison
or an exact `failure_operation` record. A test-only real-MIPS
control makes A call `CreateProcessW` after a preexisting raw
`CeEventHasOccurred` launch; it is red-to-green on a same-birth
selected metadata rewrite: both original births remain queued in FIFO order,
live and unresolved. An unchanged-context mixed control confirms that both
launches are admitted in FIFO order; it does not attribute them
to the `CreateProcessW` caller. Existing non-live, both-mode and final
parent-resume controls also pass. The final `unicorn.rs` SHA-256 is
`ED2A2AB693735794DB86DCC97E672DCB16AB5071C13EA0010D0A877E8393E2B8`.
The [sealed receipt](../../evidence/stage3-deferred-context-preflight-20261017/validation.receipt.md)
records a13 focused `deferred_` **17/0**, a14 no-default 19 blocks
**1,548/0/2 ignored**, a15 release-feature 19 blocks **2,192/0/9 ignored**,
a16 parent-resume **1/0**, and a17–a19 feature checks exiting 0. Source
hashes remained stable despite backup-only HEAD movement during a15. Raw
`CreateProcessW` queues B before the post-native caller-birth verification;
there is no independent event/Shell trigger. The later P0 gate adds in-place
claims but deliberately keeps both births eligible; a fair trigger and
producer witness must later change the mixed control. This earlier cut did
not supply a post-B restoration token, `ParentSuspension`, a
connected port, Step 2/Stage 3 acceptance or live proof.

The separate raw `CreateProcessW` output-Boolean cut treats
`PROCESS_INFORMATION==NULL` as optional and returns `TRUE` after an exact child
birth is published even if a later output word cannot be copied. The bounded
[A03 raw-kernel](../../evidence/ce6-createprocess-output-contract-20261017/a03-raw-kernel-green.raw.log)
passed **135/0** and [A04 no-default](../../evidence/ce6-createprocess-output-contract-20261017/a04-broad-no-default.raw.log)
passed 19 blocks **1,550/0/2 ignored**. The [sealed receipt](../../evidence/ce6-createprocess-output-contract-20261017/validation.receipt.md)
also records A05 release-feature **2,194/0/9 ignored** over 19 blocks and
a06–a08 feature checks exiting 0. Earlier output words may remain written; no
guest-byte rollback is promised. The shim's forced success-path `LastError=0`
is a known CE6 mismatch, not a result of this gate. Null/unwritable output can leave
process/thread aliases undisclosed to the guest, and their lifetime after
accepted exit is unproved. `TRUE` still precedes loader/CPU admission, and the
then-aggregate drain had no creator attribution or independent
event/Shell trigger. This does not establish Step 2, O-T7 or live acceptance.

The separate [bounded Shell output gate](../../evidence/stage3-shell-createprocess-handoff-20261017/validation.receipt.md)
keeps a published exact-birth direct-EXE launch from returning `FALSE` solely
because an `hInstApp` or requested `hProcess` copy failed. A01 was expected
red **0/1**; final focused **2/0**, raw-kernel **137/0**, 19-block no-default
**1,552/0/2 ignored**, release-feature **2,196/0/9 ignored**, and three
feature checks passed with stable source hashes. Invalid-pointer CE parity,
success-path `LastError=0`, undisclosed alias retirement, loader admission
and a post-native Shell trigger remain open. It does not implement the
later producer-witness gate below or confer O-T7/live acceptance.

##### Preparatory P0 in-place pending-launch ownership cut (bounded source-test gate)

The production post-native runner now reserves non-owning entry-time FIFO
locators in `CeKernel` instead of using aggregate
`take_pending_process_launches` and a local `VecDeque`. It claims each original
entry by exact process and primary-thread births in Kernel's existing queue.
Unattempted siblings remain queryable in their original slots during another
child's preparation or native run; no queue borrow spans that run. A retryable
pre-CPU failure releases the same birth in place, ahead of later publications.
Stale or duplicate claims refuse without consuming a sibling; checked
zero-slice completion or CPU admission consumes only the claimed entry.
Nested batches see new publications but cannot take or reorder an outer
batch's reserved siblings.

The [bounded receipt](../../evidence/stage3-pending-queue-p0-20261017/validation.receipt.md)
records Kernel retry/tail-order and stale-token controls, runner observers
that see unattempted siblings after claim and post-admission immediately
before native execution, a real-MIPS nested E→D before outer
B/C order control, the unchanged-context mixed E+B control, 19-block
no-default **1,555/0/2 ignored** and release-feature **2,202/0/9 ignored**
suites, and three feature checks exiting 0 on stable source hashes. The
observers do **not** independently inspect the queue during guest execution,
and no active-claim checkpoint save was executed; the existing checkpoint
refusal checks pending-launch and canonical Task presence. These are bounded
source/test facts, not broader custody or runtime acceptance.

P0 deliberately preserves the previous admission set and order: older event
E and new B may both be admitted under A's post-native tail. It adds no creator
attribution, attempt-local producer witness or independent Shell/notification
trigger. E-under-A remains a documented compatibility limit, not CE timing
evidence or a policy to extend. P0 does not establish Stage 3, O-T7, a
callback-scoped port, runnable CPU custody or live-app acceptance. No app or
writable mount was run.

The focused P1 cut follows P0 and precedes the witnessed producer gate below.
The [dumped `notify.dll`/hive binding](../../evidence/ce6-rea-nfy0-20261017/findings.md)
identifies a concrete `NFY0:` driver candidate but does not settle
before/after-IOCTL-return timing.

##### P1 independent event-service trigger (focused source gate; broad pending)

The source now stores Event origin on each published descriptor: event ID and
the checked publishing thread/process births are provenance, not the child
owner. A missing or stale native caller refuses before event launch
publication; raw compatibility is explicit. The origin and captured path base
survive queue take/restore and later selected-process changes. At a neutral
host-loop turn after the wall-clock cap check, the machine offers only the
oldest unreserved Event by its original exact birth before completed-active
handling. No native slice is open and no dummy owner is created.
Ordinary B and unselected Event siblings remain in Kernel's queue. One Event
is selected per neutral offer, but a nested P0 `CreateProcessW` batch can
still drain older E, siblings and B under A's existing admission policy.
Retryable pre-CPU failure releases the original Event in its queue slot.

Stamped focused logs passed [a04 Kernel **2/0**](../../evidence/stage3-event-trigger-p1-20261017/a04-pending-event-focused.raw.log),
[a05 real-MIPS Event **2/0**](../../evidence/stage3-event-trigger-p1-20261017/a05-guest-event-focused.raw.log),
[a06 publisher-path **1/0**](../../evidence/stage3-event-trigger-p1-20261017/a06-event-path-provenance.raw.log),
[a07 raw-kernel **138/0**](../../evidence/stage3-event-trigger-p1-20261017/a07-raw-kernel.raw.log),
[a08 same-birth pre-CPU retry **1/0**](../../evidence/stage3-event-trigger-p1-20261017/a08-event-retry.raw.log)
and [a10 real host-loop wiring **1/0**](../../evidence/stage3-event-trigger-p1-20261017/a10-host-loop-wiring-green.raw.log).
A09 failed because its harness propagated an intentionally expected no-PE
error; a10 corrected the assertion without a production fix. Broad suites,
feature checks and the final source seal are pending. The controls do not
establish requested debugger-stop/monitor paths, full selected-context or
parked-CPU ownership on every error, independent Shell service, a witnessed
B-only handoff, the callback-scoped port, full Stage 3/O-T7 or live iNavi.
The timing claim is only eventual host-loop service; `NFY0:` driver behavior
and CE before/after-IOCTL-return timing remain unknown.

##### Exact-birth pending-launch producer and claim gate (proposed, not implemented)

The owning state stays in `CeKernel`'s existing pending-launch queue: each
original entry retains its exact process/primary-thread births and a checked,
in-place claim phase/token. For this single-child `CreateProcessW`/direct-Shell
cut, the producer must pass an attempt-local exact publication witness through
dispatch to the post-native consumer;
`TRUE`, an ordinal, a mutable selected PID or the last queue entry cannot
identify B after nested publication. Do not add a second pending-owner vector.
Claim/release/consume only the witnessed entry, after the selected-context
preflight; a stale or duplicate claim refuses without touching siblings.
For the witnessed path, the consumer claims B in place and must not call
`take_pending_process_launches`; no unattempted entry leaves Kernel's queue.

Write these controls test-first within the same sealed source cut; none lands
as an ignored red:

1. For an original non-shortcut direct `.exe` `ShellExecuteEx` branch, the
   resolver must preserve that route distinction (it currently merges a
   `.lnk` target with direct `.exe`). Service only its exact B after the native
   attempt closes. The [dumped-COREDLL branch and local CE6 source](../../evidence/ce6-rea-shell-trigger-20261017/findings.md)
   establish one reachable Shell-to-`CreateProcessW` branch and support a
   loader-completion boundary, not exhaustive route coverage. Require the
   emulator's B loader/admission to resolve before A's next guest instruction;
   do not require B to execute user code or exit first.
2. With an older notification E queued before B, A's `CreateProcessW` or
   eligible Shell handoff claims only B. E retains its exact birth and FIFO
   position and stays visible to Kernel while B is claimed and running: an
   exact-birth queue query still finds E, `has_pending_process_launches`
   remains true, and a monitor checkpoint save refuses. A changed
   parent-context stamp leaves both unclaimed and live.
3. At the queue level, a claimed B1 must not hide a newly published B2 from
   B2's own claim or consume an unrelated C; after both exact claims resolve,
   remaining E/C keep their order. Do not treat this as executed nested
   `emu_start` or a leaf-permit/reentry proof.
4. A retryable pre-CPU path/PE/setup error releases B **in place**, keeping
   its original position among E/C and any subsequently published tail, with
   its original live, unsignaled birth. Once CPU-admitted, B is not replayed
   as a pending launch; that error boundary remains a custody concern.

Raw `CeEventHasOccurred` had no independent service trigger at P0's seal; the
focused P1 source now offers one Event at a neutral host turn, with broad
validation pending. The same [REA finding](../../evidence/ce6-rea-shell-trigger-20261017/findings.md)
shows a COREDLL `DeviceIoControl` call and local CE6 `NFY0:` routing; the
[dumped driver binding](../../evidence/ce6-rea-nfy0-20261017/findings.md)
identifies `notify.dll` but not its scheduling/return timing. The P1 eventual-
service cut claims exact Event entries but must not assert before- or
after-IOCTL-return parity without handler/runtime evidence. This proposal
does not implement O-T7, a connected callback-scoped Kernel/framebuffer port,
checked post-B parent restoration, runnable/parked CPU custody, Stage 3/5 or
live acceptance.

Test-first acceptance for this port is narrower than CPU-custody or Stage 4
acceptance:

1. Preserve the two ordinary real-MIPS live-pump fixed-stack first-touch
   controls as positive fault-path witnesses. Add a deliberate busy-port
   fault control: typed stop, nonzero guest PC, no canonical page publication
   or pin, no host panic, and no replay as an ordinary unmapped guest error.
2. Exercise loan conflicts and exact-authority mismatch in code/block,
   import/HLE and framebuffer paths at reachable callback boundaries. A
   nested host callback needs an actual source path; do not manufacture an
   import/block overlap with a nested `uc_emu_start`. Each real conflict must
   stop through the same terminal attempt record without consuming a
   different birth's wait or returning a fabricated API value. Check native
   hook finish/abort teardown before the outer mutable references are reused.
3. Execute a post-native real-MIPS A-to-B launch with no open parent Kernel or
   framebuffer loan. Check B's exact birth, A's staged return PC,
   non-overlapping A/B native execution and checked parent selection restoration;
   inject PID reuse or changed selection lineage and require a stopped
   refusal, not blind restoration. Test queued host observations during an
   exit transaction and a reentrant mutation refusal without using undefined
   raw-pointer aliasing. The exact single-leaf permit and retained B custody
   are later O-T7 controls, not port-only acceptance.
4. Review every hook family and require no remaining raw Kernel/framebuffer
   dereference in the normal attempt. A source inventory supports, but does
   not replace, the executed controls. Keep `unicorn`-only, desktop/no-trace,
   release and no-default gates separate where feature boundaries differ.

A pre-CPU exact-birth claim/return cut in Kernel's existing pending-launch
queue can be independently correct before this port, and its retry/tail-order
controls should precede nested execution. The bounded pre-admission check
above does not provide that claim: its unchanged-context mixed control still
drains older event and new `CreateProcessW` launches together. A fair trigger
must retain older work without silently admitting it as A's child. A
memory-fault-only port cut cannot
make the normal attempt sound while other raw hooks and the caller's mutable
references remain. Conversely, a complete port can establish machine-state
loan safety before runnable CPU slots, but by itself does not preserve an
admitted child on ordinary native error or nonempty `Completed` publication
conflict. Cut 3a now preserves the first bounded fault shape only.
The nonempty publication conflict still requires the connected O-T7 custody
cut and active-leaf run gate. The earlier ignored native-fault red has an
ordinary real-MIPS focused green Cut 3a replacement with non-runnable
quarantine, but that narrow directory is not the proposed runnable slot/lease
contract. The unconnected loan-cell foundation was retired from active
source. The selected-context pre-admission check is a narrow production
consumer, but no connected callback-scoped port, checked post-B selector
token or exit event fence described here is used by production native
attempts today.

The historical 2026-10-17 pre-deferral read-only callback inventory is for
`unicorn.rs` SHA-256
`F102058948DCA07E745BA98FF8E482B32E72995B6924D76A8FDCD3911859857B`;
it counts source-line references, not runtime calls. `kernel_ptr` is created
at `unicorn.rs:9754`; its 133 hook
uses and `framebuffer_ptr`'s 10 hook uses are grouped below. The latter is
created at `:9755` (11 total source lines including its declaration).

| Hook family in `unicorn.rs` | `kernel_ptr` uses | `framebuffer_ptr` uses | Reentry surface |
|---|---:|---:|---|
| Global code hook (`9871–10352`) | 4 | 1 | GetMessage/remote/input service and tick |
| Scoped code hooks (`10353–10916`) | 5 | 1 | Same service and parked-peer readiness |
| Scheduler block hook (`10917–11397`) | 19 | 0 | Wait/modal/suspended return and host input |
| Import entry/return (`11398–12335`) | 50 | 1 | WNDPROC, sent reply, GUI completion/destruction |
| Import pre-dispatch/callout (`12336–13279`) | 38 | 5 | Birth/wait checks, serial/socket, window/thread callouts |
| Import loader/HLE/launch (`13280–13679`) | 10 | 2 | HLE dispatch and then-immediate child run |
| Import post-dispatch (`13680–13807`) | 6 | 0 | Callout and wait/GetMessage resume |
| Memory-fault hook (`13970–14013`) | 1 | 0 | Missing-page repair |

The later test-only fixed-stack witness reaches the production live-pump
missing-page repair path in both explicit-stop/global-hook and default
hookless/count-limited configurations. This narrows a Step 2 fixture, but
does not make the raw `kernel_ptr` loan sound or cover reentry into that hook.
The read-only follow-up found no safe hook-only port: the caller keeps
`&mut CeKernel`/framebuffer through `emu_start`, and the other raw-pointer hook
families can access the same machine state. Repair consults live exact-birth
VM state and publishes a prepared native page mapping before committing the
canonical pin; an eager snapshot would change that ordering. Deferring the
fault until after native stop is not established either: the current error
tail poisons the continuation, and a MIPS delay-slot fault cannot be assumed
restartable from the scalar PC. Port all callback families under one checked
machine-state loan/leaf contract; use this first-touch witness as its positive
fault control, not as proof of the port.

The [current-source pointer delta audit](../../evidence/stage3-machine-port-inventory-20261017/findings.md)
is separate from the historical 133/10 inventory above. On the later
`unicorn.rs` source, the [scheduler-block fence](../../evidence/stage3-scheduler-block-fence-20261017/validation.receipt.md)
replaced that callback's 19 direct Kernel and two separate framebuffer raw
dereferences with one `SchedulerBlockStateLease` unsafe adapter. Its guard
spans this callback only; the executed real-MIPS test **preseeded** busy and
did not trigger natural nested reentry. Code/import/memory hooks still hold
raw aliases, the outer mutable references span `emu_start`, and post-native
A-to-B child execution still lacks the checked shared leaf. Source order
permits a modal restore to publish pixels after consuming a wait before a
hypothetical nested fault is observed; this gate did not execute that case.
This is not a sound machine-state loan or Step 2 acceptance. A later
[source call-chain finding](../../evidence/stage3-machine-port-inventory-20261017/findings.md#current-importscheduler-call-chain-finding)
found no natural import↔scheduler-block callback overlap in the current
attempt: the block hook and import hook run sequentially under the sole
production native `emu_start` (now at `unicorn.rs:14796`), neither hook starts a
nested guest run, and successful `CreateProcessW` defers its child until the
parent native run returns. The vendored Unicorn
[`uc_emu_start` implementation](../../vendor/unicorn-engine-sys/uc.c#L1117)
marks nested count/stop state ambiguous and warns about incorrect JIT state
at `uc.c:1117–1139`. Do not add a synthetic shared import/block gate as a
surrogate for natural overlap. The outer long-lived mutable Kernel and
framebuffer references, other raw hook aliases and post-native child loan
remain open; sequential hooks alone do not prove a sound whole-attempt port.

The subsequent [bounded native-entry precondition](../../evidence/stage3-native-entry-precondition-20261017/validation.receipt.md)
now refuses a second guest-native entry on the same host thread at the start
of the private attempt, before saved-CPU setup. Its preseeded-busy real-MIPS
control verifies typed refusal and same-CPU retry, and the sealed broad
suites pass. It is not a machine-owned leaf permit, does not exclude another
host thread, and does not remove the outer Kernel/framebuffer loans or the
post-native child helper's alias boundary. The proposed port and O-T7 slot
contract remain unimplemented.

At that historical source seal, non-live-pump `CreateProcessW` ran the child
immediately from the import hook at `:13569–13579`; its HLE dispatch loan had
ended, but `run_pending_process_launches(&mut CeKernel)` retained a mutable
Kernel loan across the child run. The current cut removed that immediate raw
hook call. After a successful import, both modes now write the API result,
stage the parent's nonzero return PC, stop the native attempt and run pending
launches from the post-native path. Non-live normal rotation can then select
the child before the parent resumes. This closes that one in-hook alias route,
but the post-native child helper still holds `&mut CeKernel` across B's run and
the other hook raw aliases remain. Wrapping the entire child call in a short-
loan API would still retain the loan, not implement the proposed port. The
callback-scoped machine-state port, checked CPU slots and live-pump later-slice
custody gate remain open; no sound machine lease is claimed.

`ProcessBirthKey` currently has no machine lineage, and the monitor's clone
path copies `UnicornMips` and `CeKernel` independently. Until one checkpoint
transaction can relineage Kernel process/thread births, frames/waits, VM,
slots and locators together, save **and** restore must refuse any live or
retained canonical Task/launch authority and unresolved CPU custody. An empty
CPU wrapper is insufficient: boot PID 1/TID 1 already have live births, and a
queued child has its own birth before a CPU exists. A cloned slot key from the
original machine must be rejected in its clone. The bounded fail-closed
source guard does not provide live in-session rewind; after D-011 refusal the
operator must repair the cause and restart until atomic relineage exists.
O-T7 remains open on the exact representation and recovery interface; this
candidate fixes the invariants to review first.

At its historical 2026-10-16 seal, the preserved
[real-MIPS non-live expected-red](../../evidence/stage3-real-mips-first-slice-20261016/validation.receipt.md)
executed A's actual COREDLL `CreateProcessW` IAT call to launch B and
returns through A's saved startup RA. A exits `0x63`, B stays exact-birth
Kernel-live and unsignaled, but slice-end rotation/pruning makes B the local
CPU before the outer launch handler sees A's exited handle. Its
`CreateProcessChildParkSkipped(already_exited)` drops B instead of publishing
it to the parent (queue length zero, expected one). This is a CPU-custody red,
not merely a missing `initial_child_exit` trace label; it predates the current
post-native launch cut and is not its validation result.

At the final deferred-source seal, ignored reruns
[16](../../evidence/stage3-createprocess-deferred-20261017/attempt16-current-completed-red.raw.log)
and [17](../../evidence/stage3-createprocess-deferred-20261017/attempt17-current-first-slice-red.raw.log)
exit 101 at stale fixture preconditions, before their old failure sites;
neither proves those failures fixed or reproduced on the new route. Their
ignored fixture implementations were retired. A new direct-MIPS fixture
reaches first-slice `Completed` with an empty descendant queue and a held
parent borrow; it does not prove a nonempty descendant splice or live-pump
later-slice behavior.
[Attempt 18](../../evidence/stage3-createprocess-deferred-20261017/attempt18-current-native-fault-red.raw.log)
exited 101 at the admitted child's sole-CPU-owner assertion after an ordinary
native fault at that earlier seal. Cut 3a now has an ordinary green
replacement in the owner-sealed a12 release suite.

Full O-T7 acceptance still needs the held-borrow and later-slice shapes. The
historical [injected first-slice native-error red](../../evidence/stage3-post-admission-native-fault-20261016/validation.receipt.md)
executed the child's first native slice to `READ_UNMAPPED`, then found the
exact child Kernel-live/unsignaled but the parent custody queue empty. It
covered the non-live-pump helper only; the later ordinary Cut 3a real-MIPS
replacement is focused green, not full ownership acceptance. Turn remaining
paths green without drops; also require exact-birth/PID-reuse and
cloned-lineage negatives, and A→B success preserving original B CPU, context
and wait. A controlled fixture
must separately exercise the `Exited` classifier: this natural real-MIPS
shape rotates and prunes A first and returns B as `Parked`. B's later PC
resumability is another gate. Current successful `CreateProcessW` defers the
child to the post-native path in **both** modes; the non-live path may then
rotate to the child before the parent's staged return PC executes. Live-pump
still needs its own later-slice A exit/survivor fixture plus eventual approved
app observation. Deferral alone did not implement CPU custody or replace the
two existing rotation-policy sites ahead of Stage 5.
Neither fixture substitutes for a matched iNavi map, GPS, frame, latency or
footprint run.

Post-wake `MsgWaitForMultipleObjectsEx(QS_SENDMESSAGE)` is a separate fixture:
prove the sent edge wakes the exact T birth, then that real Get/Peek retrieval
enters WNDPROC and returns the one blocked sender's result. A wake bit alone
does not prove dispatch or continuation completion.

For finite Sleep, prepare/commit verifies the live `ThreadBirthKey`, checked generation and wait ID, and authentic MIPS context before publishing the frame/wait. A stale frame or birth cannot consume another wait or fabricate a guest return. Timeout selection claims that exact frame and validates/restores its context before retiring its wait and pins; a non-rollbackable native/map failure is a typed backend failure, not a CE success. Exit/cancellation removes only matching finite-Sleep frame state. Wait-ID wrapping, machine/checkpoint lineage and coherent GuestInstant remain separate gates, not presumed solved here.

The independent source audit identified two remaining policy sites: `main.rs` and `UnicornMips` both select/rotate parked processes. They are removed only after all runnable families read canonical Task/Frame state and Stage 5 priority/quantum outcomes are ready; replacing one heuristic alone is not an acceptance gate. The bounded source-green GetMessage/PeekMessage/NoWait lifecycles and focused failure guards do not close O-T1, O-T3 or O-K3, or authorize live-app acceptance.

Open decisions (TaskDirectory API and legacy fixtures, incarnation exhaustion, native-frame payload, termination/pending-suspend scope, priority inheritance, OEM quantum): [DECISIONS.md](../DECISIONS.md) O-T1 to O-T6. None is silently decided by this document.
