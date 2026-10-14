# Kernel objects and handles

Status: proposed subsystem plan; no new object implementation is introduced here. The [architecture anchor](../ARCHITECTURE.md) owns shared CeMachine, TaskIdentity, FrameIdentity, AddressSpace, outcome and SnapshotEpoch contracts; the [migration plan](../PLAN.md) sequences changes. This document describes their kernel-object consumers rather than redefining them.

## Responsibility

The machine-owned HandleTable is the single home of canonical TaskDirectory authority. A handle is a process-visible reference to a canonical object, not the object itself and not a task incarnation. Canonical task state has one owner even when several handles, CPU execution and pending waits reference it.

Object lifetime and wait completion are separate from runnable eligibility. An exited thread object can remain signaled for surviving references. A suspended thread can complete an object wait without becoming eligible to execute. AddressSpace selection does not change creation ownership.

## Current behavior and known gaps

[HandleTable](../../src/ce/object.rs) maps globally allocated handle numbers to KernelObject values. [CeKernel](../../src/ce/kernel.rs) adds pseudo-handle resolution, raw API semantics, owner checks for selected object kinds, wake hints and diagnostic records.

ThreadObject stores TID/start/parameter/priority/suspend/exit/signaled data by value. Kernel duplicate_handle uses clone for ThreadObject, so aliases can disagree; thread_handle_by_id selects a surviving value, not a canonical task owner. Creation PID/incarnation is absent. Separate Kernel main-thread fallback maps can disagree with later real handles. Closing a handle removes its value; existing blocked continuations mostly retain numeric handles.

The same clone-by-value problem affects every object kind that is not a file, find, volume or notification handle. `CeKernel::duplicate_handle` (`src/ce/kernel.rs:6377-6421`) hands the cloned value to its `other => other` arm. A duplicated event, mutex, semaphore or file mapping therefore becomes an independent copy. Signalling the copy, or mapping a view through it, does not reach the source. CE's HNDLDuplicate (`handle.c` 774–829) instead points the new handle at the same PHDATA. This is Stage 3 object authority, so it should be fixed together with the Task controls rather than inside D-008. Until then, a cross-process shared-view test should use a named `CreateFileMappingW` reopen as its public route: `HandleTable::create_file_mapping` (`src/ce/object.rs:654-681`) returns the existing object's handle.

`HandleTable::insert` increments the global public handle number without checked exhaustion. Child launch completion currently signals by the original public process/thread handles, so closure before exit can strand canonical completion even after thread aliases are repaired. A durable child ProcessIdentity and primary TaskKey are required, while process-worker termination policy remains open.

Other object types have individual conventions: process aliases receive some shared-by-ID updates, file duplication uses file-system bookkeeping, and mapped views use canonical mapping data with process ownership. These exceptions do not establish a universal alias/lifetime model. Current global handle numbers and virtual-allocation metadata do not enforce CE6 per-process handle-table/address-space isolation.

Task-owned compatibility wait storage is implemented, but numeric handle arrays plus duplicated native wait payloads are not pinned canonical object references. Toolhelp/diagnostic inference is not authoritative ownership.

## Target object and reference ownership

Keep one canonical TaskDirectory in HandleTable, injected/owned by CeMachine; do not add a mirrored Kernel Task-state enum or a second CPU identity registry. Thread handles carry identity references and resolve mutable state through the directory.

Canonical records explicitly track:

- Execution references while a task/continuation can run or terminate.
- Public alias references while guest-visible handles remain open.
- Wait-frame references while a registered FrameIdentity pins a wait target.
- Any distinct object-specific references (views, callbacks, asynchronous I/O) required by its actual API contract.

These are retirement conditions, not a proposed immortal per-TID cache. The last public alias closing cannot kill a running task or invalidate a target already pinned by a wait. After execution and all dependent references end, retirement releases canonical state. Incarnation/generation checks reject stale references if IDs/storage are reused. Checkpoint clones preserve alias relationships **within** each independent machine clone, without sharing mutable records between clones.

O-K3 pin/retirement representation is a **first-cut Stage 3 gate**: an active or queued task persists with no public alias, and a terminal signaled target persists while an alias or registered wait owns it. A wait must complete through its pinned canonical reference after its public alias closes; re-resolving the raw handle at each poll is insufficient. Cancellation and partial preparation release only the references actually acquired, exactly once. The concrete representation remains open.

Birth is transactional at the API publication boundary. Failed thread-ID/output initialization retires an unpublished creation candidate; ordinary CloseHandle on an established live thread is not that operation. Boot main registration must exist even when no public thread handle is created. Child primary ownership survives pending-launch drain; worker identity records authentic creation ownership.

## Wait and alias interfaces

Proposed logical responsibilities, not final signatures:

- Validate source/target process and pseudo handles; resolve a public handle to a canonical typed object reference.
- Duplicate a reference to the same object; never clone mutable thread state. Source/target process ownership and checked public handle allocation are part of the alias boundary. DUPLICATE_CLOSE_SOURCE semantics and error handling must follow primary CE evidence even on failure.
- Pin all wait targets before publishing a wait frame. Partial validation failure releases only pins acquired during preparation.
- Register ordered wait-set references and deadline under FrameIdentity. Derived object/message/serial/send indexes identify frames, not owners.
- Complete one frame exactly once, consume only the selected object's real acquisition effect and invalidate old generations/subscriptions.
- Release frame pins on completion, cancellation, task/process exit or failed preparation. Canonical thread exit queues wake candidates for every relevant public alias/reference.
- Serve snapshots and API getters from canonical state with SnapshotEpoch/provenance, not arbitrary first alias or current-process guesses.

Unknown/mismatched type, stale incarnation, invalid process ownership, unreadable guest handle arrays and unavailable native state are failures, not permissive defaults. Host memory writes cannot bypass guest permission checks merely because a native library API allows it.

## Primary CE6 evidence

Primary local source: `C:/WINCE600/PRIVATE/WINCEOS/COREOS/NK/KERNEL`.

- `handle.c` HNDLDuplicate 774–829 allocates a destination handle referring to the same PHDATA and increments handle references. NKDuplicateHandle 840–891 validates process handles/options/output and closes the source under `DUPLICATE_CLOSE_SOURCE` when the source process is valid even if validation, duplication or output copy fails. Freeze emulator failure cases before deciding O-K4.
- `thread.c` InitThreadStruct 272 clears reused thread storage while sleeper/synchronization references can remain. Identity/frame generations are required; copied scalar equality alone does not prove incarnation.
- `schedule.c` NKWaitForMultipleObjects 644–656 rejects WaitAll, zero and excessive counts for this CE6 kernel. Do not import desktop-Windows WaitAll semantics.
- Its 660–684 path locks each waitable PHDATA before DoWaitForObjects and unlocks afterward. A wait pins canonical object lifetime, not a mutable lookup of a public handle on every poll.
- Wait proxies store priority, owner/generation and WAIT_OBJECT_0+index (`schedule.c` 519–563; `NK/INC/schedule.h`). Ordered WaitAny acquisition follows WaitOneMore; auto-reset consumes one signal, manual reset remains signaled/wakes all. Pending signal/timeout serialization and generation retirement matter; no invented universal race winner.

Mutex/critical-section ownership, recursion, abandonment and priority inheritance need object-specific evidence/tests; they are not solved by task canonicalization. Event Pulse, Set and Reset remain distinct operations.

## Source consumers and migration scope

Initial task canonicalization touches:

- HandleTable create_thread, thread_id/handle_by_id/start/snapshots, exit and exit-code APIs.
- suspend/resume by handle/ID; base-priority getters/setters; wait_for_single_object/wait_for_any_object/is_wait_ready; diagnostics.
- Kernel boot/create_guest_thread/queue_process_launch_with_options and pseudo-handle adapters; priority/suspend/quantum fallback retirement.
- duplicate_handle/close_handle; IsPrimaryThread/Toolhelp; exit wake hints for aliases and process-worker cleanup.
- coredll create-thread output failure and ExitThread/ThreadBaseFunc/ThreadExceptionExit/TerminateThread; native create/return/resume/context APIs through canonical identity.

Do not migrate only Kernel wrappers while direct HandleTable readiness still reads obsolete mutable copies. Keep typed public snapshots separate from authoritative internal records. The existing raw/general object APIs and direct unit fixtures require a documented compatibility path when creation owner is unknown; do not fabricate a PID.

## Test gates and pending decisions

Eight actual-Kernel controls and two raw-API controls (ledger E-8), plus three
later last-public-handle/alias-wake/wait-pin controls (E-11), form the
expected-red baseline: one close-source preservation pass and twelve failures.
Alias wake-index and clone tail assertions in the earlier controls were not
reached after preceding failures. This is a baseline, not kernel-object
acceptance.

Before implementation acceptance:

1. Alias priority/suspend/exit/wait signals agree bidirectionally; unrelated task stays unchanged.
2. Close-source/last-public-handle cases preserve live execution and pinned wait targets; null/bad duplicate output, failed allocation and invalid source-process cases establish source-close behavior; failed unpublished creation leaves no runnable task.
3. Creation PID and primary identity survive process rotation, diagnostic eviction, launch drain and closure of the original child process/thread handles through canonical child completion; boot main remains registered.
4. Stale handle/incarnation/frame references cannot acquire a replacement object; checked PID/TID/public-handle exhaustion publishes no alias/task; partial wait preparation and cancellation release pins once.
5. Deep clone retains canonical alias graphs within each clone, with independent mutable state/RAM.
6. Event auto/manual/pulse, semaphore/mutex acquisition and signal/timeout interleavings execute through real API/native boundaries; object completion is not automatically CPU dispatch.

Open decisions (per-process handle tables, object-reference representation, retirement bookkeeping, source-close duplication failure, process-termination worker sweep, priority inheritance): [DECISIONS.md](../DECISIONS.md) O-K1 to O-K6. Resolve them against the anchor before changing policy or adding a broad generic object framework.
