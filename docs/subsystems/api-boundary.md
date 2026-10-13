# API boundary and guest effects

Status: existing HLE import/callback paths are not the final machine-owned boundary.
Authority: [shared architecture](../ARCHITECTURE.md). Sequence: [migration plan](../PLAN.md).

## Current implementation

- [imports.rs](../../src/emulator/imports.rs) dispatches traps; [coredll.rs](../../src/ce/coredll.rs)
  contains raw/high-level CE shims and guest-memory contracts. Supporting modules include
  [crt.rs](../../src/ce/crt.rs), [ole.rs](../../src/ce/ole.rs) and [winsock.rs](../../src/winsock.rs).
- [unicorn.rs](../../src/emulator/unicorn.rs) recognizes callouts/waits before raw dispatch,
  mutates native registers, executes callbacks and maintains legacy continuation vectors.
- Native/direct/raw/trace adapters are different entry paths to the same semantics;
  their pointer mapping, error and lifetime behavior must remain equivalent where promised.
- Typed deferred payload and 28-byte MSG preflights now precede known queue consumption.
  Unknown guest pointers remain guest pointers, not guessed host payload allocations.
- Legacy adapters without mapping context defer known typed payloads for a mapped
  successor; they are guarded, not claimed universally functional or fully mapped.

## Target input, effect and continuation contracts

- `CeMachine` is the one effect authority/event sequencer. API code receives the
  anchor's canonical `TaskIdentity` and its authorized `AddressSpace`.
- Blocking/callback work refers to the shared `FrameIdentity`; wait ID and TID alone
  cannot identify a current nested return frame or a new Task incarnation.
- Decode a declared ABI into checked arguments, request typed subsystem effects, then
  return an explicit result/error or machine-owned continuation—not a fake host success.
- Use the anchored `RunOutcome`, preserving unknown observations. A raw shim result
  is not a scheduler stop reason or evidence that a callback actually returned.
- `GuestInstant` defines guest waits/timeouts through machine policy; `HostInstant`
  bounds service/native execution. Host sleeps and configured deadlines cannot forge time.
- Publish effects/results under `SnapshotEpoch`, actual Task/frame and operation identity.
  ACK, queued message, dispatched callback and guest return are separate observations.

## Allowed interfaces and dependency direction

- Shims depend on checked AddressSpace access and standalone kernel/object, storage,
  GWE, device, network, registry/resource or audio services with explicit ownership.
- They cannot select CPU actors through visibility heuristics, allocate canonical Task
  identities independently, or maintain another authoritative ready/wait/frame registry.
- CPU adapter executes validated callback/import frames; API services do not perform
  uncontrolled nested native runs, force application state or paint desired outcomes.
- Ordinary HLE remains the current boundary. Running real CE6 coredll/syscall API sets
  is a separate future spike, not an implied accomplishment of the Phase 1 rewrite.
- Constants, structures and callback behavior require CE source/binary/ABI evidence.
  Do not fix a single program by name, guessed readiness flags or invented return values.

## Memory, queue and frame lifetime

- Prepare known typed payload bytes and writable backing before guest pointer exposure,
  register edits, queue consumption or callback entry. Preserve successful LastError.
- Use exact operation/send/wait/frame identity across timeout/expiry. Identical messages
  may coexist; preflight of one must not consume a replacement with matching parameters.
- Check whole output extents and permissions; host native writes may bypass page access.
  Failed preparation retains FIFO/wakes; already-live partial output has no byte rollback.
- Unknown/mismatched pointer payloads retain documented guest semantics, not opportunistic
  serializer reinterpretation. Host reservations need valid allocation incarnation/extent.
- Frame-owned scratch survives all genuine callback phases and is released at real
  completion/teardown. Kernel lifetime, CPU snapshot and guest pointer lifetime are distinct.
- Same-task nested callbacks retain native parent Context; cross-thread callbacks use
  the receiver's canonical Task. Full GPR restore is not full native CPU restoration.
- A synchronous result changing V0/PC is a validated continuation effect, not permission
  to combine another actor's hidden branch/FPU state with normalized register values.

## Failure and cleanup

- Separate invalid parameter/protection/metadata from identifiable allocation exhaustion;
  do not infer out-of-memory from arbitrary backend text. Preserve CE LastError contracts.
- A handled failure must not fall through and repeat an already-applied API operation.
  Existing SetWindowPos geometry may already change; there is no invented atomic rollback.
- Allocate/map/zero/copy/initialize before publishing a new pointer. Free only unpublished
  candidates on failure; retain old live values where the API guarantees that behavior.
- Preflight fallible CPU transfer before irreversible wake/queue/object effects. Define
  transaction limits explicitly; never re-post guessed messages or synthesize caller registers.
- Native errors, memory faults, checked-restore failures and unexplained PC zero fail hard.
  A configured native timeout cannot erase them or become reported API/guest completion.
- Cleanup follows canonical Task/frame/operation lifetime and evidence newer than the
  frame. A retained debug snapshot is not authority to retire a live callback/wait.

## Evidence, telemetry and migration gates

- The failed 650 persistent-owner run and the native arena evidence are in the
  [ledger](../evidence/ledger.md) (E-2 to E-6). They do not implicate a particular
  shim and are not API, app, throughput or crash-cause evidence.
- Record bounded decoded ABI, operation/frame identity, memory preflight, result/LastError,
  real callback entry/return and cancellation. Preserve native read/query failures explicitly.
- Migrate one complete API family with all raw/native/direct/deferred/debug/teardown
  consumers; routing one path does not establish whole-boundary equivalence.
- Preserve executable raw/native allocation, payload, output-protection, FIFO/expiry,
  nested callback, wait timeout, receiver termination and fault-injection controls.
- Add canonical alias/incarnation/frame rejection and genuine in-slice native FPU/return
  controls. New test designs remain staged until execution is recorded separately.
- Require feature-gated builds and bounded actual delivery evidence. Retained rings,
  HTTP acceptance and clean unit tests do not prove complete startup or map readiness.
