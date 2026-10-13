# Tasks and scheduler

Status: proposed subsystem plan; implementation follows review and test gates. The [architecture anchor](../ARCHITECTURE.md) defines shared identities, ownership, time, outcomes and snapshot contracts. The [migration plan](../PLAN.md) controls sequencing. This document uses those contracts; it does not define competing TaskIdentity, FrameIdentity, RunOutcome or SnapshotEpoch types.

## Responsibility and scope

CeMachine owns scheduling across all AddressSpaces. Its machine-owned HandleTable is the target single canonical directory: Stage 2 introduces process-birth records and process-handle aliases there, and Stage 3 extends that same authority for Tasks/objects. The scheduler selects canonical tasks and continuation frames; a process CPU/native engine is an execution resource, not a second Task registry. Kernel shims and native hooks request transitions through that authority.

In scope: authentic task birth/exit, runnable state, explicit suspension, waits, nested continuation ownership, CE priority and quantum accounting. Out of scope for this first plan: heap reuse, app-specific readiness/visibility rules, a new HLE syscall boundary, or assumed cycle timing. Clock conversion is coordinated with [clock and timers](clock-timers.md); wait-target lifetimes with [kernel objects](kernel-objects.md).

## Current implementation, not target guarantees

- [ThreadSystem](../../src/ce/thread.rs) owns TLS, LastError, kernel-mode flags and globally allocated numeric TIDs; it is not a task state manager.
- [HandleTable](../../src/ce/object.rs) stores ThreadObject values with entry/parameter, priority, suspend count and exit/signaled fields, but no creation PID/incarnation.
- Its public-handle allocator advances unchecked; ThreadSystem TID and Kernel child PID allocators saturate. Checked incarnation alone cannot prevent an exhausted public handle number from colliding with an existing alias.
- [CeKernel](../../src/ce/kernel.rs) has separate priority/suspend fallback and quantum maps. Boot PID1/TID1 has no public ThreadObject. Child primary lookup can depend on pending launches or a bounded diagnostic trace; Toolhelp can attribute existing workers to the selected process.
- DuplicateHandle clones mutable ThreadObject state by value, so aliases can disagree. Thirteen controls cover alias/creator/primary identity, clone coherence, `OpenThread` reusing a public handle, `GetProcessIdOfThread` reporting the selected PID, final-public-handle lifetime, alias-indexed wake and registered-wait target retention. The older private-mirror runs are [E-8/E-11](../evidence/ledger.md); the current D-source focused [E-26](../evidence/ledger.md) default runs report one active pass and twelve ignored, while explicit ignored runs fail all twelve at intended assertions. They are a Stage 3 red baseline, not production Task migration.
- [Scheduler](../../src/ce/scheduler.rs) now stores compatibility waits through [TaskWaitStore](../../src/ce/task_wait.rs). It preserves multiple same-TID frames, global wait-ID order and existing subscriptions/selection/unsigned deadline behavior; TID is still a compatibility key.
- [UnicornMips](../../src/emulator/unicorn.rs) owns numerous pending return, blocked wait/GetMessage/modal/send/clipboard, suspended context and parked process records. SuspendedGuestThread slot/queue often contains **ready displaced contexts**, not explicit SuspendThread state.
- Persistent process engines are an experiment (ledger E-2) deferred to Stage 8; Task migration proceeds on the fresh-engine lifetime ([D-003](../DECISIONS.md)). Exact-scalar-key native snapshots are an initial migration, not full per-task native contexts.

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

- Gate 1: extend Stage 2 process birth/alias authority to canonical Task/object lifecycle, primary completion and checked TID/handle allocation. The ten earlier controls (ledger E-8) plus three Task lifetime/alias-wake reds (E-11) now have a D-source executed-red baseline (E-26) and must turn green; add genuine native last-public-handle execution, wait acquisition after alias closure, stale-key, allocator-exhaustion and child-exit-after-original-handle-close controls. O-K3 pin/retirement is part of this first cut, not a later cleanup. Later assertions in several earlier red cases were not reached. This is not a passed gate.
- Gate 2: ordinary blocked-wait and ready-continuation ownership, with genuine MIPS capture/restore and full caller return. The staged in-slice FPU regression is unexecuted; existing GPR handoff controls do not prove FPU/frame preservation.
- Gate 3: nested callback→outer wait resume, stale incarnation/frame rejection without consumption, explicit suspend while blocked, alias close during execution, process exit with workers and creation failure cleanup.
- Gate 4: effective/base priority, equal-priority quantum rotation, higher-priority preemption, quantum 0 and genuine idle eligibility. Only then enable the clock migration and retire compatibility heuristics.

The first connected Stage 3 production-slice audit and independent contract review (chat-only notes, see the ledger index) identify the HandleTable, Kernel, COREDLL, wait-index, pin/retirement and native-completion edits that must move together after the Stage 1 gate and the Stage 2 outcome contract; persistent-engine acceptance is not a prerequisite. They are design, not an implemented slice.

Open decisions (TaskDirectory API and legacy fixtures, incarnation exhaustion, native-frame payload, termination/pending-suspend scope, priority inheritance, OEM quantum): [DECISIONS.md](../DECISIONS.md) O-T1 to O-T6. None is silently decided by this document.
