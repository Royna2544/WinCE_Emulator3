# Kernel objects and handles

Status: thread/process alias, immutable creation-owner, child completion, bounded `TerminateProcess`, O-K3 execution/task-wait pins, stale-TID/native-stamp continuation, and parked Sleep/object/MsgWait/queue/socket birth slices passed source gates; full Stage 3 and live-app behavior remain unaccepted. The [architecture anchor](../ARCHITECTURE.md) owns shared CeMachine, TaskIdentity, FrameIdentity, AddressSpace, outcome and SnapshotEpoch contracts; the [migration plan](../PLAN.md) sequences changes. This document describes their kernel-object consumers rather than redefining them.

## Responsibility

The machine-owned HandleTable is the single home of canonical TaskDirectory authority. A handle is a process-visible reference to a canonical object, not the object itself and not a task incarnation. Canonical task state has one owner even when several handles, CPU execution and pending waits reference it.

Object lifetime and wait completion are separate from runnable eligibility. An exited thread object can remain signaled for surviving references. A suspended thread can complete an object wait without becoming eligible to execute. AddressSpace selection does not change creation ownership.

## Current behavior and known gaps

[HandleTable](../../src/ce/object.rs) maps globally allocated handle numbers to KernelObject values. [CeKernel](../../src/ce/kernel.rs) adds pseudo-handle resolution, raw API semantics, owner checks for selected object kinds, wake hints and diagnostic records.

The entry slice stores thread mutable state once in `HandleTable` and gives public thread handles incarnation-qualified aliases. `DuplicateHandle` and `OpenThread` share that record; wait registration also records a typed canonical wake key. The later O-K3 slice retains an execution-live thread after its final public alias closes and pins registered Thread/Process wait targets by exact birth. A new wait cannot acquire an already closed public handle, but a previously registered wait can complete through its pin. The creation-owner follow-on stores a checked ProcessBirthKey for production thread births; direct legacy HandleTable fixtures may still have an explicit unknown owner. Boot TID 1 has an execution-live, aliasless record, and the child primary is born under the child process birth. Separate Kernel primary-thread fallback maps can disagree with durable identity. Existing blocked continuations mostly retain numeric handles; the bounded GetMessage compatibility bridge checks the stored wait-owner birth rather than treating its handle number as task identity.

The same clone-by-value problem affects every object kind that is not a file, find, volume or notification handle. `CeKernel::duplicate_handle` (`src/ce/kernel.rs:6410-6453`) hands the cloned value to its `other => other` arm. A duplicated event, mutex, semaphore or file mapping therefore becomes an independent copy. Signalling the copy, or mapping a view through it, does not reach the source. CE's HNDLDuplicate (`handle.c` 775–829) instead points the new handle at the same PHDATA. The reviewed first alias slice below covers thread/process handles only; generic Event/Mutex/Semaphore/FileMapping aliasing is an explicitly open following slice, not a D-008 memory shortcut. Until then, a cross-process shared-view test should use a named `CreateFileMappingW` reopen as its public route: `HandleTable::create_file_mapping` (`src/ce/object.rs:696-725`) returns the existing object's handle.

`HandleTable::try_insert` checks public handle-number exhaustion for typed alias issuance, but infallible birth creation and generic objects still call `insert`; those paths are not a complete failure transaction. The child launch record carries checked process and primary-thread births. Verified, exact-birth child completion signals the process wait and sweeps owned workers; a request alone never signals that wait. The bounded source-tested `TerminateProcess` handshake stops an owned active child at preflight or a self-call import boundary, and completes requested parked or pending children without another guest instruction. Saved worker contexts, exit stubs, pending thread returns and suspended thread contexts reject same-process TID reuse by birth; verified native slice-end staging rejects stale or unverified identity. Bounded Sleep/object, parked MsgWait and native queue/socket continuations now verify native register ownership before publication or wake consumption and restore a verified birth. An unpinned intra-slice worker switch still fails closed. This is not full TaskDirectory retirement: CE caller wait/timeout, a safe old-MIPS direct-kernel real-handle continuation, root-process real-handle completion, aliasless Sleep-suspend and other parked-wait families remain open.

Other object types have individual conventions: `OpenProcess(current PID)` now returns a distinct closable alias, while `GetCurrentProcess` remains pseudo. File duplication uses file-system bookkeeping and mapped views use canonical mapping data with process ownership. Named Event/Mapping CloseHandle can return success without removing a public handle, duplicated file/find contexts have separate cursors, and shared mapping pages are still keyed by a public handle. These exceptions do not establish a universal alias/lifetime model. Current global handle numbers and virtual-allocation metadata do not enforce CE6 per-process handle-table/address-space isolation.

Task-owned compatibility wait storage is implemented. Registered Thread/Process object-wait slots now hold checked canonical birth pins, while opaque non-task wait slots and duplicated native wait payloads remain unpinned compatibility state. Toolhelp reads stored creation ownership for registered threads; a synthetic host-only caller TID retains a selector-based compatibility fallback. Other diagnostic/primary fallback maps are not durable Task authority.

## Target object and reference ownership

Keep one canonical TaskDirectory in HandleTable, injected/owned by CeMachine; do not add a mirrored Kernel Task-state enum or a second CPU identity registry. Thread handles carry identity references and resolve mutable state through the directory.

Canonical records explicitly track:

- Execution references while a task/continuation can run or terminate.
- Public alias references while guest-visible handles remain open.
- Wait-frame references while a registered FrameIdentity pins a wait target.
- Any distinct object-specific references (views, callbacks, asynchronous I/O) required by its actual API contract.

These are retirement conditions, not a proposed immortal per-TID cache. The last public alias closing cannot kill a running task or invalidate a target already pinned by a wait. After execution and all dependent references end, retirement releases canonical state. Incarnation/generation checks reject stale references if IDs/storage are reused. Checkpoint clones must preserve alias relationships **within** each independent machine clone, without sharing mutable records between clones. Passing the first slice's `CeKernel::clone` alias-coherence test does not establish machine/checkpoint relineage or A2 clone acceptance; that remains open.

O-K3 pin/retirement representation is a bounded **Stage 3 source gate**: an active or queued task persists with no public alias, and a terminal signaled target persists while an alias or registered task wait owns it. A registered wait completes through its pinned canonical reference after its public alias closes; re-resolving the raw handle at each poll is insufficient. Cancellation and partial preparation release only the references actually acquired, exactly once. The E-11 last-public-handle and wait-pin controls were expected red at the earlier alias-only checkpoint; they are now active and green under the O-K3 source gate. This is not full Stage 3 or live-app acceptance.

The next native wait boundary is a prepare → mutate → publish transaction.
`CeKernel::prepare_task_wait` resolves ordered object-wait slots and holds
Thread/Process births in an unpublished guard. Failed preparation releases
earlier pins, including duplicate slots, and sets the waiting thread's CE
LastError (`ERROR_INVALID_HANDLE` for invalid/stale targets,
`ERROR_NOT_ENOUGH_MEMORY` for pin exhaustion). The native
`WaitForSingleObject`, `WaitForMultipleObjects`, and
`MsgWaitForMultipleObjectsEx` callers return `WAIT_FAILED` without consuming
their pending callout or replacing an old blocked wait; successful handoff
publishes once. Production
non-object subscriptions use `register_opaque_compatibility_waiter` instead:
their message, socket, serial and other tokens are not interpreted as task
handles or pinned merely because a numeric value coincides. The legacy
`register_blocked_waiter` remains a test/compatibility entry and returns
reserved wait ID zero on preparation failure, without publication. Focused
controls cover rollback, failure and retry; the bounded
[source gate](../../evidence/stage3-waitprep-20261015/validation.receipt.md)
passed, but the live gate remains separate. Synchronous finite inline timeout paths do not publish a
wait, and duplicated native wait payloads are not covered by these pins.

Birth is transactional at the API publication boundary. Failed thread-ID/output initialization retires an unpublished creation candidate; ordinary CloseHandle on an established live thread is not that operation. Boot main registration must exist even when no public thread handle is created. Child primary ownership survives pending-launch drain; worker identity records authentic creation ownership.

## Wait and alias interfaces

Target logical responsibilities, not final signatures; registered task-object pin acquisition and release are source-tested, while other object families remain proposed:

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

## First handle-alias entry slice (source-gated)

This is the reviewed **thread/process** source slice, not generic object authority or full Stage 3. Keep the existing canonical process-birth directory (`src/ce/object.rs:728-842`) as the one process owner. In HandleTable, give each new thread record a checked, non-reused identity; public thread handles reference that record rather than copying mutable priority, suspend, start, exit and signaled fields. The existing direct HandleTable getters, by-ID methods, snapshots, wait-readiness methods and Kernel/native completion consumers must resolve the same record (`src/ce/object.rs:1050-1174`; `src/ce/kernel.rs:3116-3135,6529-6543,7200-7212`). No second Kernel Task state is allowed. Legacy fixtures with unknown creation owner remain explicitly unknown.

`DuplicateHandle`, `OpenThread` and `OpenProcess` issue checked new public numbers referring to the existing typed identity. Preflight source type, source/target process birth and handle capacity before publication; a failed allocation publishes neither an alias nor cloned state. Keep GetCurrentProcess pseudo semantics separate from a real `OpenProcess(current PID)` alias. Migrate the old raw assertion in `tests/coredll_raw_kernel.rs:6704-6717` deliberately and preserve the child process alias/exit controls in `src/ce/kernel.rs:11310-11412`. Closing one public reference must not mutate a sibling. The successful `DUPLICATE_CLOSE_SOURCE` control remains active, but CE's source-close-on-failure rule (`handle.c:880-883`) needs null/bad-output, failed-allocation and invalid-source-process controls before O-K4 can be accepted. A raw output-copy failure must remove a newly created destination alias; it must not leave a phantom reference.

For object waits, derive a typed canonical Thread/Process wake key at the Kernel-to-Scheduler registration boundary (`src/ce/kernel.rs:7290-7330,7388-7391`; `src/ce/scheduler.rs:97-130,179-305,327-339`). Preserve ordered public handles for API reporting and the existing GetMessage/Sleep/Serial/Winsock/SendMessage/modal compatibility queues; do not invent an object identity for them. Wake by canonical key, not by scanning sibling alias numbers. At the first alias-only checkpoint, this **index** change did not pin a target or authorize post-close acquisition. O-K3 subsequently added preflight pins for registered ordered Thread/Process wait slots and exact release on completion, cancellation or failed preparation; opaque non-task slots remain compatibility-only and unpinned. A new wait still cannot acquire an already closed handle.

Deep-copy mutable thread records on `CeKernel::clone` while preserving sibling aliases inside that clone; never share mutable `Rc`/`Arc` state across clones. This proves only the local test's alias graph, not machine/checkpoint birth-key relineage. Leave generic `other => other` duplication and named-object/file/find/mapping/view lifetime behavior explicitly open for a following connected object slice. A source-gate failure calls for a reviewed code rollback with logs and source hashes preserved; this plan gives no permission for a live app run.

## Bounded immutable creation-owner slice

The source-gated follow-on stores a checked `ProcessBirthKey` in each production
`ThreadRecord` when a thread is born. CE6's `NKCreateThread` selects the active
process at creation (`thread.c` 935) and `InitThreadStruct` stores it as the
thread owner (`thread.c` 276); `GetProcessIdOfThread` reads that frozen owner
(`thread.c` 810–812). The emulator's normal guest path now resolves the
creator TID's canonical owner before creation, not the mutable selected PID
afterward. Future cross-process/PSL execution still needs a checked active
process frame at that boundary; this slice did not fabricate one.

Boot TID 1 is registered as an execution-live, aliasless canonical thread owned
by the boot process birth, so raw and native CreateThread can resolve its
creator. The child primary receives the child ProcessBirthKey at birth.
Production raw and native CreateThread paths pass their creator TID through
one checked owner route and fail closed for an absent/stale owner. The existing
selector-based `create_guest_thread` remains an explicitly labelled host/test
compatibility entry; direct legacy HandleTable fixtures may retain an explicit
unknown owner. Neither path infers ownership later from the currently selected
process.

`GetProcessIdOfThread` and Toolhelp now read the stored owner for a valid
registered thread birth. The active
`get_process_id_of_thread_reports_immutable_creation_owner` and
`task_ownership_pid_is_creation_owner_not_selected_process` controls pass
after launch-queue drain, process rotation and adversarial selector changes.
Boot-main enumeration and durable child-primary identity controls remain
ignored; an aliasless boot record alone does not satisfy their full contracts.
The historical owner key was a prerequisite, not proof, of process cleanup.
A subsequent child-exit slice sweeps workers and releases fixed-stack backing
only after exact-birth verified completion; a request alone does not signal a
real process wait. A later bounded `TerminateProcess` source slice stops an
owned active child at preflight or a self-call import boundary, and completes
requested parked or pending children without another guest instruction. The
frozen-source broad gates for that latest slice passed: no-feature 1,375/0/5
ignored and release Unicorn/trace/desktop 1,858/0/11 ignored
([receipt](../../evidence/stage3-terminate-20261015/validation.receipt.md)).
The later O-K3 execution/wait-pin source gate is recorded in its
[receipt](../../evidence/stage3-pins-20261015/validation.receipt.md).
The bounded stale-TID/native-stamp [source gate](../../evidence/stage3-stale-tid-20261015/validation.receipt.md)
birth-verifies pending thread returns, suspended thread contexts, native
slice-end staging, and the GetMessage compatibility bridge.
The bounded [Sleep/object parked-wait gate](../../evidence/stage3-parked-wait-20261015/validation.receipt.md)
verifies captured register birth before publication, checks the exact live
wait owner before acquiring a wake, and stamps verified birth on resume.
The bounded [parked MsgWait gate](../../evidence/stage3-msgwait-birth-20261015/validation.receipt.md)
also rejects a stale frame before timer pumping or input-edge acknowledgement;
raw synchronous MsgWait paths remain outside it.
The bounded [parked queue/socket gate](../../evidence/stage3-queue-socket-birth-20261015/validation.receipt.md)
verifies native MessageQueueIo/WinsockRead register birth before publication and
paired live birth before wake consumption or register restoration. Raw
synchronous/nonblocking fallbacks and opaque non-task pins remain outside it.
CE caller wait/timeout, old-MIPS direct-kernel real-handle continuation,
root-process real-handle completion, aliasless Sleep-suspend, unpinned-worker
lineage and other parked-wait families remain open. The owner-slice
[receipt](../../evidence/stage3-owner-20261014/validation.receipt.md) records
its own source-bound gates; no live app behavior was measured for these slices.

## Test gates and pending decisions

Eight actual-Kernel controls and two raw-API controls (ledger E-8), plus three
later last-public-handle/alias-wake/wait-pin controls (E-11), formed the
historical expected-red baseline: one close-source preservation pass and
twelve failures.
Alias wake-index and clone tail assertions in the earlier controls were not
reached after preceding failures. This is a baseline, not kernel-object
acceptance. The first alias slice activated and turned green these six in
the same source change:

| Control | First-slice evidence |
| --- | --- |
| `open_thread_returns_independent_alias_to_same_live_task` | A distinct public `OpenThread` alias |
| `task_ownership_thread_aliases_share_ce_priority` | One mutable priority |
| `task_ownership_thread_aliases_share_suspend_count` | One suspend/start state |
| `task_ownership_thread_aliases_share_exit_and_wait_signal` | One exit state and canonical wake key |
| `thread_exit_through_one_alias_indexes_waiter_on_another_alias` | Canonical-key wake index |
| `task_ownership_cloned_kernel_does_not_share_mutable_thread_state` | Independent clone-local alias graph |

The bounded creation-owner slice also activated and turned green these two:

| Control | Owner-slice evidence |
| --- | --- |
| `get_process_id_of_thread_reports_immutable_creation_owner` | Raw API reads frozen creation ProcessBirthKey |
| `task_ownership_pid_is_creation_owner_not_selected_process` | Worker/Toolhelp owner is stable across process selection and launch drain |

Two controls remain ignored with their stated missing contracts:

| Control | Still missing |
| --- | --- |
| `task_ownership_boot_main_is_retained_without_a_public_thread_handle` | Aliasless boot-main Toolhelp enumeration and full identity contract |
| `task_ownership_primary_identity_outlives_launch_queue_and_trace_ring` | Durable child primary identity and completion |

The O-K3 source slice activated and turned green the other two historical
E-11 controls, with checked wait-reference release and retirement:

| Control | O-K3 source-gate evidence |
| --- | --- |
| `live_task_survives_closing_its_final_public_handle` | Execution-live record survives final alias closure and retires after exit |
| `closed_thread_alias_wait_keeps_an_exited_target_pinned` | Registered wait pins the exact exited birth through alias closure, then releases it |

Preserve the active `task_ownership_duplicate_close_source_keeps_thread_alive`
success-path control. The alias slice also gated process current-PID
alias/pseudo separation, sibling-close, checked exhaustion/no-publication and
stale-key behavior. The owner follow-on passed its focused controls, debug
no-default all-targets (1,364/0/5), release unicorn/trace/desktop all-targets
(1,829/0/11) and desktop no-trace check on the same owner source. Its receipt
records hashes and logs. Those earlier owner gates did not establish O-K3;
the later O-K3 [receipt](../../evidence/stage3-pins-20261015/validation.receipt.md)
records its own bounded source gate. Generic object aliasing, full Stage 3,
CE scheduling, checkpoint lineage, native last-public-handle execution and
live-app responsiveness remain unaccepted.

Before implementation acceptance:

1. Alias priority/suspend/exit/wait signals agree bidirectionally; unrelated task stays unchanged.
2. Close-source/last-public-handle cases preserve live execution and pinned wait targets; null/bad duplicate output, failed allocation and invalid source-process cases establish source-close behavior; failed unpublished creation leaves no runnable task.
3. Creation PID and primary identity survive process rotation, diagnostic eviction, launch drain and closure of the original child process/thread handles through canonical child completion; boot main remains registered.
4. Stale handle/incarnation/frame references cannot acquire a replacement object; checked PID/TID/public-handle exhaustion publishes no alias/task; partial wait preparation and cancellation release pins once.
5. Deep clone retains canonical alias graphs within each clone, with independent mutable state/RAM.
6. Event auto/manual/pulse, semaphore/mutex acquisition and signal/timeout interleavings execute through real API/native boundaries; object completion is not automatically CPU dispatch.

Open decisions (per-process handle tables, object-reference representation, retirement bookkeeping, source-close duplication failure, process-termination worker sweep, priority inheritance): [DECISIONS.md](../DECISIONS.md) O-K1 to O-K6. Resolve them against the anchor before changing policy or adding a broad generic object framework.
