# Kernel objects and handles

Status: first thread/process alias entry slice partially implemented, with final source gate pending; full Stage 3 remains proposed. The [architecture anchor](../ARCHITECTURE.md) owns shared CeMachine, TaskIdentity, FrameIdentity, AddressSpace, outcome and SnapshotEpoch contracts; the [migration plan](../PLAN.md) sequences changes. This document describes their kernel-object consumers rather than redefining them.

## Responsibility

The machine-owned HandleTable is the single home of canonical TaskDirectory authority. A handle is a process-visible reference to a canonical object, not the object itself and not a task incarnation. Canonical task state has one owner even when several handles, CPU execution and pending waits reference it.

Object lifetime and wait completion are separate from runnable eligibility. An exited thread object can remain signaled for surviving references. A suspended thread can complete an object wait without becoming eligible to execute. AddressSpace selection does not change creation ownership.

## Current behavior and known gaps

[HandleTable](../../src/ce/object.rs) maps globally allocated handle numbers to KernelObject values. [CeKernel](../../src/ce/kernel.rs) adds pseudo-handle resolution, raw API semantics, owner checks for selected object kinds, wake hints and diagnostic records.

The entry slice stores thread mutable state once in `HandleTable` and gives public thread handles incarnation-qualified aliases. `DuplicateHandle` and `OpenThread` share that record; wait registration also records a typed canonical wake key. An execution-live thread survives closure of its final public alias internally, but post-close wait acquisition and target pinning are not implemented. Creation ProcessBirthKey is absent. Separate Kernel main-thread fallback maps can disagree with later real handles, and existing blocked continuations mostly retain numeric handles.

The same clone-by-value problem affects every object kind that is not a file, find, volume or notification handle. `CeKernel::duplicate_handle` (`src/ce/kernel.rs:6410-6453`) hands the cloned value to its `other => other` arm. A duplicated event, mutex, semaphore or file mapping therefore becomes an independent copy. Signalling the copy, or mapping a view through it, does not reach the source. CE's HNDLDuplicate (`handle.c` 775–829) instead points the new handle at the same PHDATA. The reviewed first alias slice below covers thread/process handles only; generic Event/Mutex/Semaphore/FileMapping aliasing is an explicitly open following slice, not a D-008 memory shortcut. Until then, a cross-process shared-view test should use a named `CreateFileMappingW` reopen as its public route: `HandleTable::create_file_mapping` (`src/ce/object.rs:696-725`) returns the existing object's handle.

`HandleTable::try_insert` checks public handle-number exhaustion for typed alias issuance, but infallible birth creation and generic objects still call `insert`; those paths are not a complete failure transaction. Child launch completion now marks its known primary TID by ID and emits a canonical process wake once, but its launch record carries bare PID/TID, not durable birth keys. Other workers are not swept on process exit, and a request-only parked child can be dropped before checked completion. A durable child ProcessIdentity and primary TaskKey are required.

Other object types have individual conventions: `OpenProcess(current PID)` now returns a distinct closable alias, while `GetCurrentProcess` remains pseudo. File duplication uses file-system bookkeeping and mapped views use canonical mapping data with process ownership. Named Event/Mapping CloseHandle can return success without removing a public handle, duplicated file/find contexts have separate cursors, and shared mapping pages are still keyed by a public handle. These exceptions do not establish a universal alias/lifetime model. Current global handle numbers and virtual-allocation metadata do not enforce CE6 per-process handle-table/address-space isolation.

Task-owned compatibility wait storage is implemented, but numeric handle arrays plus duplicated native wait payloads are not pinned canonical object references. Toolhelp/diagnostic inference is not authoritative ownership.

## Target object and reference ownership

Keep one canonical TaskDirectory in HandleTable, injected/owned by CeMachine; do not add a mirrored Kernel Task-state enum or a second CPU identity registry. Thread handles carry identity references and resolve mutable state through the directory.

Canonical records explicitly track:

- Execution references while a task/continuation can run or terminate.
- Public alias references while guest-visible handles remain open.
- Wait-frame references while a registered FrameIdentity pins a wait target.
- Any distinct object-specific references (views, callbacks, asynchronous I/O) required by its actual API contract.

These are retirement conditions, not a proposed immortal per-TID cache. The last public alias closing cannot kill a running task or invalidate a target already pinned by a wait. After execution and all dependent references end, retirement releases canonical state. Incarnation/generation checks reject stale references if IDs/storage are reused. Checkpoint clones must preserve alias relationships **within** each independent machine clone, without sharing mutable records between clones. Passing the first slice's `CeKernel::clone` alias-coherence test does not establish machine/checkpoint relineage or A2 clone acceptance; that remains open.

O-K3 pin/retirement representation is a **first accepted Stage 3 gate**: an active or queued task persists with no public alias, and a terminal signaled target persists while an alias or registered wait owns it. A wait must complete through its pinned canonical reference after its public alias closes; re-resolving the raw handle at each poll is insufficient. Cancellation and partial preparation release only the references actually acquired, exactly once. The proposed thread/process alias slice is an entry milestone, not that accepted gate: its E-11 last-public-handle and wait-pin controls remain expected red. No last-public-alias retirement policy can be inferred from the alias-only implementation.

Birth is transactional at the API publication boundary. Failed thread-ID/output initialization retires an unpublished creation candidate; ordinary CloseHandle on an established live thread is not that operation. Boot main registration must exist even when no public thread handle is created. Child primary ownership survives pending-launch drain; worker identity records authentic creation ownership.

## Wait and alias interfaces

Proposed logical responsibilities, not final signatures:

- Validate source/target process and pseudo handles; resolve a public handle to a canonical typed object reference.
- Duplicate a reference to the same object; never clone mutable thread state. Source/target process ownership and checked public handle allocation are part of the alias boundary. DUPLICATE_CLOSE_SOURCE semantics and error handling must follow primary CE evidence even on failure.
- Pin all wait targets before publishing a wait frame. Partial validation failure releases only pins acquired during preparation.
- Register ordered wait-set references and deadline under FrameIdentity. Derive object-wake indexes from checked canonical object keys, never alias-number fan-out; message/serial/send indexes retain their own typed compatibility keys. Indexes identify frames, not owners.
- Complete one frame exactly once, consume only the selected object's real acquisition effect and invalidate old generations/subscriptions.
- Release frame pins on completion, cancellation, task/process exit or failed preparation. Canonical thread exit queues wake candidates for every relevant public alias/reference.
- Serve snapshots and API getters from canonical state with SnapshotEpoch/provenance, not arbitrary first alias or current-process guesses.

Unknown/mismatched type, stale incarnation, invalid process ownership, unreadable guest handle arrays and unavailable native state are failures, not permissive defaults. Host memory writes cannot bypass guest permission checks merely because a native library API allows it.

## Primary CE6 evidence

Primary local source: `C:/WINCE600/PRIVATE/WINCEOS/COREOS/NK/KERNEL`.

- `handle.c` HNDLDuplicate 774–829 allocates a destination handle referring to the same PHDATA and increments handle references. NKDuplicateHandle 840–891 validates process handles/options/output and closes the source under `DUPLICATE_CLOSE_SOURCE` when the source process is valid even if validation, duplication or output copy fails. Freeze emulator failure cases before deciding O-K4.
- `thread.c` NKOpenThread 1018–1033 and `process.c` NKOpenProcess 1515–1533 duplicate the kernel thread/process object into the active process's handle table; the latter explicitly permits an exited but still referenced process. `OpenProcess(current PID)` is a new closable alias, not GetCurrentProcess's pseudo-handle.
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

## First handle-alias entry slice (partial, final gate pending)

This is the reviewed **thread/process** source slice, not generic object authority or full Stage 3. Keep the existing canonical process-birth directory (`src/ce/object.rs:728-842`) as the one process owner. In HandleTable, give each new thread record a checked, non-reused identity; public thread handles reference that record rather than copying mutable priority, suspend, start, exit and signaled fields. The existing direct HandleTable getters, by-ID methods, snapshots, wait-readiness methods and Kernel/native completion consumers must resolve the same record (`src/ce/object.rs:1050-1174`; `src/ce/kernel.rs:3116-3135,6529-6543,7200-7212`). No second Kernel Task state is allowed. Legacy fixtures with unknown creation owner remain explicitly unknown.

`DuplicateHandle`, `OpenThread` and `OpenProcess` issue checked new public numbers referring to the existing typed identity. Preflight source type, source/target process birth and handle capacity before publication; a failed allocation publishes neither an alias nor cloned state. Keep GetCurrentProcess pseudo semantics separate from a real `OpenProcess(current PID)` alias. Migrate the old raw assertion in `tests/coredll_raw_kernel.rs:6704-6717` deliberately and preserve the child process alias/exit controls in `src/ce/kernel.rs:11310-11412`. Closing one public reference must not mutate a sibling. The successful `DUPLICATE_CLOSE_SOURCE` control remains active, but CE's source-close-on-failure rule (`handle.c:880-883`) needs null/bad-output, failed-allocation and invalid-source-process controls before O-K4 can be accepted. A raw output-copy failure must remove a newly created destination alias; it must not leave a phantom reference.

For object waits, derive a typed canonical Thread/Process wake key at the Kernel-to-Scheduler registration boundary (`src/ce/kernel.rs:7290-7330,7388-7391`; `src/ce/scheduler.rs:97-130,179-305,327-339`). Preserve ordered public handles for API reporting and the existing GetMessage/Sleep/Serial/Winsock/SendMessage/modal compatibility queues; do not invent an object identity for them. Wake by canonical key, not by scanning sibling alias numbers. This **index** change alone neither pins a target nor authorizes post-close acquisition. O-K3 must later preflight and pin ordered wait targets, release each acquired pin on completion/cancellation/failure, and make exit/retirement independent of public aliases. The first slice must not claim last-public-handle survival or a completed wait after alias closure.

Deep-copy mutable thread records on `CeKernel::clone` while preserving sibling aliases inside that clone; never share mutable `Rc`/`Arc` state across clones. This proves only the local test's alias graph, not machine/checkpoint birth-key relineage. Leave generic `other => other` duplication and named-object/file/find/mapping/view lifetime behavior explicitly open for a following connected object slice. A source-gate failure calls for a reviewed code rollback with logs and source hashes preserved; this plan gives no permission for a live app run.

## Test gates and pending decisions

Eight actual-Kernel controls and two raw-API controls (ledger E-8), plus three
later last-public-handle/alias-wake/wait-pin controls (E-11), form the
expected-red baseline: one close-source preservation pass and twelve failures.
Alias wake-index and clone tail assertions in the earlier controls were not
reached after preceding failures. This is a baseline, not kernel-object
acceptance. The first alias slice must activate and turn green these six in
the same source change:

| Control | First-slice evidence |
| --- | --- |
| `open_thread_returns_independent_alias_to_same_live_task` | A distinct public `OpenThread` alias |
| `task_ownership_thread_aliases_share_ce_priority` | One mutable priority |
| `task_ownership_thread_aliases_share_suspend_count` | One suspend/start state |
| `task_ownership_thread_aliases_share_exit_and_wait_signal` | One exit state and canonical wake key |
| `thread_exit_through_one_alias_indexes_waiter_on_another_alias` | Canonical-key wake index |
| `task_ownership_cloned_kernel_does_not_share_mutable_thread_state` | Independent clone-local alias graph |

Leave these six ignored with the stated missing contract, rather than
unignoring them as an incidental consequence of alias coherence:

| Control | Still missing |
| --- | --- |
| `get_process_id_of_thread_reports_immutable_creation_owner` | Raw API creation ProcessBirthKey |
| `task_ownership_pid_is_creation_owner_not_selected_process` | Worker/Toolhelp creation ownership |
| `task_ownership_boot_main_is_retained_without_a_public_thread_handle` | Boot execution record without alias |
| `task_ownership_primary_identity_outlives_launch_queue_and_trace_ring` | Durable child primary identity and completion |
| `live_task_survives_closing_its_final_public_handle` | Execution pin and retirement |
| `closed_thread_alias_wait_keeps_an_exited_target_pinned` | Wait pin and post-close acquisition |

Preserve the active `task_ownership_duplicate_close_source_keeps_thread_alive`
success-path control. Add process current-PID alias/pseudo separation,
sibling-close, checked exhaustion/no-publication and stale-key controls, then
run focused active controls, raw Kernel/subsystem suites, release/no-default
all-targets and desktop no-trace checks on the same hashed source. Record
ignored counts, failures and immutable logs in a receipt. Even a green first
slice does not establish O-K3, generic object aliasing, full Stage 3, CE
scheduling, checkpoint lineage, native last-public-handle execution or live
app responsiveness.

Before implementation acceptance:

1. Alias priority/suspend/exit/wait signals agree bidirectionally; unrelated task stays unchanged.
2. Close-source/last-public-handle cases preserve live execution and pinned wait targets; null/bad duplicate output, failed allocation and invalid source-process cases establish source-close behavior; failed unpublished creation leaves no runnable task.
3. Creation PID and primary identity survive process rotation, diagnostic eviction, launch drain and closure of the original child process/thread handles through canonical child completion; boot main remains registered.
4. Stale handle/incarnation/frame references cannot acquire a replacement object; checked PID/TID/public-handle exhaustion publishes no alias/task; partial wait preparation and cancellation release pins once.
5. Deep clone retains canonical alias graphs within each clone, with independent mutable state/RAM.
6. Event auto/manual/pulse, semaphore/mutex acquisition and signal/timeout interleavings execute through real API/native boundaries; object completion is not automatically CPU dispatch.

Open decisions (per-process handle tables, object-reference representation, retirement bookkeeping, source-close duplication failure, process-termination worker sweep, priority inheritance): [DECISIONS.md](../DECISIONS.md) O-K1 to O-K6. Resolve them against the anchor before changing policy or adding a broad generic object framework.
