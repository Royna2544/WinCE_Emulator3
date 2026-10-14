# Common architecture anchor

Status: target design for review, not a claim that these Rust types exist.
This file is the single normative anchor for subsystem ownership and shared
contracts. [The migration plan](PLAN.md) controls implementation order;
[subsystem pages](README.md) refine their own boundary, never redefine these
contracts. A change to shared ownership, identity, time or outcome semantics
must update this anchor and its consuming tests before implementation.

## Current evidence and safe baseline

The hash-preserved measured fallback is BED2: a clean, bounded 330 s run that
is still on the decorative splash. The current `target/release` executable is
not BED2. This is not full Windows CE boot or interactive map readiness
([ledger E-1](evidence/ledger.md)).

The first persistent-engine migration passed 1,618 tests but crashed after
129.108 s with host access violation `0xC0000005` (ledger E-2). Its source is
preserved in the D: checkout and backup commits; rebuilding current source
does not reproduce BED2, and no runtime acceptance is implied. The crash site is
native TranslationBlock initialization; the cause is not established.

A source-confirmed candidate mechanism is the 1 GiB per-engine translation
arena that hook churn does not release (ledger E-3). Isolated Stage 0 controls
(E-4 to E-6) measured hook-associated retention and a commit-tail defect
candidate in the pinned native allocator. E-33 repaired that one Windows
handler arithmetic defect under [D-009](DECISIONS.md), with a red-to-green
native control. This does not explain the 650/E-32 crashes, validate a cache
budget or accept any stage.

Per [DECISIONS D-003](DECISIONS.md), the Task, object, outcome and clock work
runs on the fresh-engine-per-slice target lifetime after the Stage 2 O-P4 gate;
production currently retains an owner, so this native-lifetime investigation
does not gate later scheduler design. Persistent engines are a later swap behind the
CPU interface (plan Stage 8). Hot-path throughput facts (amortized block hook,
hookless default, per-slice MIPS logging) are in ledger E-9.

## A1. Machine authority

`CeMachine` is the proposed common execution anchor: one sequenced owner of
kernel/task state, address spaces, guest time and accepted external events.
There is at most one active guest execution lease. CPU engines, API shims,
GWE and host adapters cannot independently select a task or advance time.
It is a composition root and event sequencer, not a God implementation of
every subsystem or a second stateful scheduler. It validates and commits the
Scheduler's selection through the owning task/frame/backend interfaces.

| Authority | Owns | Consumers receive |
| --- | --- | --- |
| Machine event sequencer | Commit validated Scheduler decisions, accepted event order, publication | Validated commands/results and immutable snapshots |
| Canonical task/object directory in machine-owned HandleTable | Birth identity, aliases, lifecycle, priorities, execution phase/frame references | Read-only identity/object views and checked transitions |
| Scheduler | Derived ready/wait indexes and selection policy | Canonical tasks/frames; no copied mutable thread state |
| AddressSpace/VM | Mapping/permissions, guest byte coherence and allocation ownership | Scoped GuestMemory access and checked mapping operations |
| CPU backend | Process engine, bounded translations, native CPU contexts | Explicit RunOutcome and opaque context handles |
| Guest clock | One monotonic GuestInstant domain | Deadlines and time observations, never direct sleep-driven offsets |
| Presentation/observation | Frame/snapshot publication | Immutable epoch-labelled evidence, not execution authority |

The directory is one authority, not an additional Kernel TaskState mirror.
Native context payloads may live in the backend, but they are reachable only
through canonical task/frame references; an old container's membership does
not decide whether the task is runnable.

## A2. Shared identity and lifetime

Names below are proposed contracts, not newly implemented APIs.

- `ProcessIdentity`: machine/checkpoint lineage, public PID and checked
  creation incarnation. Reusing a numeric PID cannot reuse a birth key.
- `TaskIdentity`: immutable creation ProcessIdentity, public TID and checked
  task incarnation. Creation owner is not whichever process is selected later.
- `FrameIdentity`: TaskIdentity plus monotonically checked frame generation.
  Reentrant callbacks have nested frames; wait ID is not a task identity.
- `ObjectIdentity`: canonical object plus incarnation. Public handles are
  aliases, not copies of mutable ThreadObject or the wait target's lifetime.
- `AddressSpace` is a stable VM owner tied to ProcessIdentity. Mapping/access
  views carry a checked mapping generation in addition to that owner identity.
  A mapping change does not create a different owner; stale views must refresh
  and revalidate the relevant mappings, permissions and operation lifetime.
  An address alone does not establish ownership or distinguish reused backing.
- `SnapshotEpoch`: machine-instance/publication-lineage identity plus a checked
  monotonic sequence, with owner, frame/run-attempt and separate guest/host
  capture times. Independent clones and checkpoint restores establish a fresh
  publication lineage; copying a counter cannot reuse an observation identity.

Register boot PID1/TID1 without inventing a public guest handle. Register
workers and child primary tasks at actual creation; retain a durable primary
task index, not a bounded diagnostic-ring lookup. Handle aliases, active
execution and pending wait frames pin object/task lifetime independently.
Closing the last public handle must not kill a running task or invalidate a
wait's locked object. Exit and final retirement are distinct transitions.
Checkpoint clones must have independent mutable task/object/memory/engine
state; immutable CPU snapshots may be shared only where the native contract
permits it. Incarnation/frame mismatch fails closed, never guessed recovery.
Lineage changes are one machine/checkpoint operation that remaps canonical
birth, pending launch, current selection, VM/view and CPU/parked owner keys
together. Independently deriving `Clone` for those owners cannot relineage a
partial machine; a key from the original must fail in its clone.

## A3. Execution and continuation boundary

`RunOutcome` distinguishes guest exit, unrecoverable native/backend or
API-boundary execution failure, registered blocking, voluntary yield, proven
count exhaustion, actual native host-budget expiry, registered guest-deadline
service, explicit debugger stop and structural boundary/reconfiguration.
Ordinary CE API failures return their authentic result/status/LastError and
continue the guest; they are not automatically fatal run failures or yields.
An ambiguous stop is recorded as unknown and investigated; a configured
timeout is not actual expiry. Native host-budget expiry alone establishes
neither a GuestInstant wait deadline nor CE quantum expiration.

Every primary attempt records starting owner, actual ending owner/frame,
native result, actual deadline query and bounded fault evidence before
deferred work or repairs. Native errors are not suppressed because a positive
timeout was configured. Source-validated encoded CE exit is explicitly decoded
and reported as exit, not used as general permission to ignore PC zero.
Unexplained guest PC zero remains a hard failure.

Raw native instruction continuations and synthesized import-return/callback
continuations are different frame forms. A synthesized PC cannot carry an
unrelated pending native branch. Preserve full CPU state for authentic raw
continuations; establish a validated frame for synthesized entry/return.
In-slice restoration must also request native control-transfer exit through
the checked backend mechanism. Delay-slot/helper-interior safety guards stay
until genuine native tests prove the replacement.

Only the machine commits a task transition after context, memory and wait
ownership validate. A nested callback can run while an outer wait remains;
explicit suspension overlays ready/blocked state rather than erasing it.
Do not infer CE suspension from the legacy SuspendedGuestThread queue name.

## A4. Guest memory and publication

VM is the single coherence authority. Retained host blobs and native RAM must
not both be independently writable truths. O-M1 selects address-stable host
backing for migrated committed pages: an AddressSpace owns private slabs and
view records, while a canonical VM/file-mapping object owns shared slabs.
Each native engine maps the exact backing under a checked run lease, not a
second writable copy. Existing
unmigrated blob/view copy boundaries remain explicit and checked until their
own connected migration. The isolated pointer-mapping diagnostic is not VM
integration or Stage 2 acceptance. Debug/checkpoint copies are snapshots,
not authorization to retain a third full RAM image indefinitely.

`GuestMemory` scopes access to the stable AddressSpace owner, validated
mapping/permissions and a checked operation lifetime. Retained guest API and
callback pointers require their real live frame. Bootstrap/loader transactions
and diagnostic/checkpoint operations instead use an explicitly authorized
machine operation scope; they must not fabricate a Task/Frame to gain access.
Prepared allocations reserve without publication, establish writable backing,
initialize/copy bytes, then commit metadata and publish the pointer. Failure
does not publish partial allocation metadata or reuse an uncertain hole.
Preflight known message/output payloads before consuming a queue or exposing
a callback. Already-live partial output writes are not promised byte rollback.
Scratch lifetime belongs to its actual frame/operation; private heap ownership
and original reservation footprint must be authoritative before reuse/unmap.

Shared file views have canonical shared backing and explicit per-process views.
An inactive process's cached view must not overwrite the active owner's bytes.
Unknown pointers, absent data and failed reads remain explicit failures; no
zero-filled evidence or guessed ownership is substituted.
Native pointer mappings pin their exact private or shared backing until checked
unmap or engine closure; a closed public mapping handle with a live view does
not release shared bytes. A reservation does not itself allocate committed host pages. Private
checkpoint clones copy bytes into independent backing; shared aliases within a
clone retain one clone-local backing. Native map/protect/unmap or identity
failure cannot publish a successful continuation.

## A5. Time, scheduling and external events

`GuestInstant` and `HostInstant` are distinct. GetTickCount is the CE-width
projection of one monotonic guest timeline; wait/timer/quantum deadlines use
that same domain. Host elapsed time is evidence/watchdog time, not a second
sleep offset. CPU-to-guest-time conversion is an explicit pending decision,
not a guessed MHz value or hidden count budget.

Run mode is decided ([D-002](DECISIONS.md)): **live-first**. The default mode is
real-time coupled to live I/O (remote GPS, touch, serial) and never
fast-forwards past input that has not been observed or presents a fabricated
completion. An opt-in **fast-idle** mode, for bring-up and tests only, may
advance GuestInstant to the next established deadline when no task is
Ready/Running and no external source is connected. Deterministic replay is a
later layer over the same deadline and ordering semantics, not a prerequisite
of the clock stage. The progression policy of each mode must be specified and
tested before implementation.

Use CE6 effective-priority ordering, base/effective distinction, equal-priority
quantum rotation and correct remaining quantum on higher-priority preemption.
Quantum zero means run to completion; OEM/configured quantum is not a guessed
10-ms constant. Sleep(0), finite rounding, suspend/resume, event acquisition
and WaitAny semantics must follow local CE6 evidence. Current compatibility
wait storage intentionally does not claim these migrations are finished.

Remote controls and host I/O enqueue typed, sequenced events. HTTP ACK means
acceptance only; guest delivery/consumption requires separate evidence. Host
adapters cannot force a WNDPROC, signal an app-specific completion event or
raise priority because a window is visible.

## A6. Failure, resource and evidence policy

Every owner has explicit release/cancellation rules and bounded transient
buffers. Translation cache budgets are host-backend policy, not fake guest
heap limits. Report actual configured/native cache size, engine count, guest
mapped/live/reserved bytes, host RSS/private and trace capacity separately.
No cache/performance claim follows from retaining a native handle alone.

Snapshots name epoch, owner, attempt/frame and freshness. Cross-endpoint
atomicity requires the same immutable SnapshotEpoch; matching cached PIDs
alone cannot establish it. An unfinished run's requested artifact filenames
do not establish terminal capture. Splash timing, map readiness, input ACK,
actual payload transfer and user-visible response are separate gates.

Disposable build artifacts have an exact scoped manifest and preserved
reproduction references. Preserve installed/fallback/failed binaries and
needed symbols/evidence; do not delete active caches, dirty sources, synced
project references or user SD data. Recoverable archival precedes removal
when unique evidence may matter.

## Change rule and unresolved decisions

Before code: name the anchor contract being changed, its authority/lifetime,
all consumers, failure/rollback rules, primary CE/native evidence and tests.
Review that plan and then implement one connected migration. Do not add an
unused Task enum, another pending-vector variant or hidden policy heuristic.

Open decisions are tracked in [DECISIONS.md](DECISIONS.md), not on individual
pages. Gating design items: canonical task/object reference retirement; VM page
coherence; guest-time progression and live-event fencing. Explicit stop
classification is closed by E-34; report/failure hardening remains.
Deferred to the persistent-engine swap (not gating): native
translation-arena handler proof and budget; stable hook rebinding.
Real-coredll/syscall migration remains a separate later go/no-go, not implied
by scheduler-core authorization.
