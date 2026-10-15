# Tasks and scheduler

Status: target subsystem plan with bounded birth, alias, wait-pin, parked-continuation and finite-Sleep Task/Frame source gates passed. Only finite Sleep has a canonical active frame; general Task/Frame dispatch, broad validation, source seal, live acceptance and full Stage 3 acceptance remain open. The [architecture anchor](../ARCHITECTURE.md) defines shared identities, ownership, time, outcomes and snapshot contracts. The [migration plan](../PLAN.md) controls sequencing. This document uses those contracts; it does not define competing TaskIdentity, FrameIdentity, RunOutcome or SnapshotEpoch types.

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

The same-process guest `SendMessageW` path now leaves a send queued while its receiver owns a blocked finite-Sleep frame. The queued send retains that exact frame as immutable provenance even after Sleep retires. The sender uses the existing exact-birth scheduler/native SendMessage wait; it does **not** acquire a new canonical SendMessage frame, activate the sleeping receiver's WNDPROC, publish an early API result or use a cross-process rotation token. If S has no runnable work before T's deadline, the live host idles briefly and reports `NoAttempt` instead of replaying S's blocked import or advancing virtual time twice. Once Sleep returns, the pre-run active-callout selector still leaves this marked send queued; T's real `GetMessageW` retrieval can then enter WNDPROC and return its result to S through the existing compatibility continuation. Exact receiver-birth exit completes only sends marked for that birth with failure and leaves neighboring sends intact. A blocked `GetMessage` receiver retains its legitimate sent-callout bridge and is not classified as finite Sleep.

The formerly expected-red native controls `native_finite_sleep_defers_active_sent_callout_for_same_receiver` and `native_same_process_send_defers_wndproc_during_receiver_sleep` are source-green in the D: checkout. The real-MIPS `native_same_process_send_delivers_after_receiver_sleep_timeout` control also passes: one T `Sleep` and one S `SendMessageW` create one GWE send ID and one S wait; a pre-deadline call makes no native attempt; T resumes at its Sleep return PC without an early callout; T's `GetMessageW` enters the real WNDPROC; and S resumes once with its `0x77` result and no retained send/wait. Kernel controls separately check that the marker survives Sleep retirement until queue take and that receiver exit replies to only its marked send. Structural controls include exact-frame selection and generation/wait-ID exhaustion. This proves the bounded source path, not broad feature/no-feature suites, a frozen source seal or live-app acceptance. `PeekMessage(PM_REMOVE)`, post-wake cross-Send/MsgWait processing, other wait families and nested GetMessage caller-return remain unproved; O-T1, O-T3 and O-K3 remain globally open.

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

The two formerly expected-red controls are source-green: one exercises the active sent-callout candidate while T has a real-MIPS finite Sleep; the other runs real-MIPS T `Sleep` followed by same-process S `SendMessageW` and proves T's WNDPROC is not entered early. A third real-MIPS control follows that **same** T/S scenario through the pre-deadline `NoAttempt`, T's timed Sleep return, T's `GetMessageW`-initiated WNDPROC, and S's single `0x77` API return; it establishes eventual delivery for this path, not a general continuation migration. A blocked `GetMessage` receiver is different: its legitimate sent-callout path may enter WNDPROC, remove the GetMessage wait and retain the parent in `ResumeImportAfterWndProc`. The separate real-MIPS `GetMessage` → sent WNDPROC → finite `Sleep` → WNDPROC return → parent import continuation control remains to be built, with exact birth, SP/RA/S0/HI/LO/FPU and one-return assertions. The continuation may re-enter GetMessage and reblock, so a real message or quit is needed before asserting a caller return. Local CE6 positive `NKSleep` blocks one thread; two concurrent finite Sleeps on that same CE thread are not a valid app-path proof.

The pre-cut a04 inventory found numeric-TID wait removal, scheduler selection without active-frame identity, and native wait/pin retirement before register restoration. The finite-Sleep path now requires exact frame identity, claims completion once, uses checked capture/restore, and retires the exact frame and wait only after its result/context boundary. Its deferred-send marker persists after that retirement so pre-run callout selection cannot bypass real `GetMessageW` retrieval. Exact-birth receiver exit retires only marked sends for that birth and wakes their senders with a failure result. The broader compatibility wait families still need their own checked transition and failure-path coverage; this cut must not be generalized to them by assumption. Keep the existing parked-Sleep birth gates and add or retain wrong-generation, reused-TID birth, alias-close, cancellation/exit and failed capture/restore negatives. `PeekMessage(PM_REMOVE)` and post-wake cross-Send/MsgWait delivery remain open, as do broad feature/no-feature suites and a frozen-source seal for this cut.

For finite Sleep, prepare/commit verifies the live `ThreadBirthKey`, checked generation and wait ID, and authentic MIPS context before publishing the frame/wait. A stale frame or birth cannot consume another wait or fabricate a guest return. Timeout selection claims that exact frame and validates/restores its context before retiring its wait and pins; a non-rollbackable native/map failure is a typed backend failure, not a CE success. Exit/cancellation removes only matching finite-Sleep frame state. Wait-ID wrapping, machine/checkpoint lineage and coherent GuestInstant remain separate gates, not presumed solved here.

The independent source audit identified two remaining policy sites: `main.rs` and `UnicornMips` both select/rotate parked processes. They are removed only after all runnable families read canonical Task/Frame state and Stage 5 priority/quantum outcomes are ready; replacing one heuristic alone is not an acceptance gate. The bounded source-green GetMessage lifecycle does not close O-T1, O-T3 or O-K3, or authorize live-app acceptance.

Open decisions (TaskDirectory API and legacy fixtures, incarnation exhaustion, native-frame payload, termination/pending-suspend scope, priority inheritance, OEM quantum): [DECISIONS.md](../DECISIONS.md) O-T1 to O-T6. None is silently decided by this document.
