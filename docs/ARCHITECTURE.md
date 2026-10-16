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
runs on a fresh-engine-per-slice lifetime. O-P4 now makes a new native owner
the default for each slice under the pinned, statically linked vendored Unicorn
2.1.5 teardown proof; its frozen-source code/test gate passed
([receipt](../evidence/op4-fresh-default-20261014/validation.receipt.md)).
`WINCE_EMU_PERSISTENT_NATIVE_ENGINE=1` explicitly opts into the experimental
process-owned path, which remains a later Stage 8 swap behind the CPU
interface. Neither the O-P4 suite nor this lifetime choice establishes live
iNavi, footprint/map readiness or full Stage 2 acceptance. Native arena work
does not gate later scheduler design. Hot-path throughput facts (amortized
block hook, hookless default, per-slice MIPS logging) are in ledger E-9.

Later E-39 controls proved public private-VM and injected file-view A→B→A
same-VA bytes across fresh native owners. The D008 public named-mapping
A→B→A and in-slice alias control, plus DIB-to-public migration controls,
passed focused and final frozen-source broad gates: release 1,744/0/14 ignored
and no-feature 1,297/0/13 ignored, each across 17 result blocks
([D008 receipt](../evidence/d008-shared-view-20261014/validation.receipt.md)).
This proves isolation at a shared VA selected by the emulator's per-owner
cursor, not CE same-VA placement semantics or guest `CreateProcessW` handoff.
E-44 then source-gated fixed initial heap/stack slabs; O-M2 source-gated a
birth-qualified 256-MiB sparse heap window and removed copied spillover
([receipt](../evidence/om2-heap-20261014/validation.receipt.md)). Distinct
public mappings at one VA, writable image/DLL backing, safe heap address
reuse, machine checkpoint relineage and live-app behavior remain unproved.
A4 below remains the shared ownership contract, not a declaration that all
consumers migrated.

The bounded [D-010 finite-Sleep Task/Frame source gate](../evidence/stage3-finite-sleep-frame-20261015/validation.receipt.md)
passed on resealed source (release 2,025/0/9 ignored; no-feature 1,472/0/3
ignored). It connects one exact-birth frame through wait selection and native
resume, not a general `CeMachine`/TaskDirectory, completed Stage 3 or a live
iNavi result. A1-A3 below remain the target shared contracts.

The follow-on ordinary A→B `SendMessageW` → blocked `GetMessageW` bridge has a
bounded [A10 source/test seal](../evidence/scheduler-source-cut-a10/source-seal.md)
and [validation receipt](../evidence/scheduler-source-cut-a10/validation.receipt.md),
not a general `CeMachine` or Task/Frame migration. Release-feature passed
2,045/0/9 ignored and no-default passed 1,482/0/3 ignored across 18 groups
each, with unchanged pre/post source hashes. A09's four same-process failures
are historical: A10 verifies the exact GetMessage/native wait and saved/live
birth before recovery, then A's next-lease continuation. Only the runnable-
receiver fixture checks a later no-replay slice; the three finite-Sleep
fixtures stop at A's continuation. The follow-on strict same-process
S→T→S plain-Send shape has a bounded
[A12 source/test gate](../evidence/scheduler-source-cut-a12/validation.production.receipt.md):
formatted-source release 2,047/0/9 ignored and no-default 1,482/0/3 ignored.
The historical A11b import replay is repaired for that exact next-lease
handoff and full inner/outer return, including a wrong-parent-birth refusal.
The separate bounded
[A13 source/test gate](../evidence/scheduler-source-cut-a13/validation.final.receipt.md)
prevents saved-context recovery from reviving a same-birth blocked GetMessage
payload after its scheduler wait disappears, and fails a selected orphaned
import before guest entry. The tested modal result path requires its exact live
wait; missing-wait modal continuation fails closed. A13's final source passed
release-feature 2,052/0/9 ignored and no-default 1,482/0/3 ignored across
18 groups each. Neither gate is a general nested-send or machine-owned
Task/Frame migration. Third sender/timed/result-pointer/deeper/cross-process
reciprocal forms, legacy numeric-only HWND ownership, post-wake MsgWait,
`SMF_TIMEOUT` last-error semantics, broader modal behavior, completed Stage 3
and live-iNavi behavior remain open. Ineligible blocked reciprocal sends fail
closed with terminal `Backend`, a live-run risk; no app or mount run was made.

The bounded [host-exit source gate](../evidence/stage3-host-teardown-20261015/validation.receipt.md)
selects direct HWNDs and backing stores by verified process birth. The
subsequent [sender-ownership gate](../evidence/stage3-sender-birth-20261015/validation.receipt.md)
narrows tested queued/active sender cleanup and identified guest no-wait
admission. The later [exact-window-set gate](../evidence/stage3-host-exit-exact-window-20261015/validation.receipt.md)
prevents foreign parent/owner links from expanding host-exit window targets
and wakes an exact sender waiting on a retired receiver HWND. The bounded
[guest pseudo-exit caller gate](../evidence/stage3-pseudo-caller-birth-20261015/validation.receipt.md)
then source-tested captured thread/process-lease admission for native COREDLL
and decoded self-exit; it did not establish immediate CPU stop or Task
retirement. The later bounded [D-011 source/test gate](../evidence/stage3-host-exit-framebuffer-preflight-20261015/validation.receipt.md)
checks every exact-birth saved window backing before stopped-exit mutation and
returns a typed recovery error on refusal. A stopped boot CPU with a changed
selected process also returns typed authority recovery instead of resuming.
The bounded [native COREDLL pseudo self-exit source/test gate](../evidence/stage3-native-pseudo-self-exit-20261016/validation.receipt.md)
stops its guest CPU before another instruction, closes its native owner,
checks the exact caller and child-launch or boot witness, then applies D-011
preflight before canonical Task/VM retirement. A refusal retains the stopped
owner and poisons native re-entry. At that earlier native seal, raw/decoded
pseudo routes remained synchronous compatibility paths. The later bounded
[decoded self-exit gate](../evidence/stage3-raw-decoded-pseudo-exit-20261016/decoded-pseudo-exit.receipt.md)
uses an executed call/delay-slot witness and that stopped finalizer for the
direct-kernel pseudo shape. Raw direct compatibility remains separate. The
bounded [modal-presence refusal gate](../evidence/stage3-modal-exit-failclosed-20261016/validation.receipt.md)
rejects affected MessageBox/popup state before stopped-exit effects in its
tested shapes; checked modal backing restoration and the complete target
exit-effect contract, including failures after successful
preflight, remains open. No live iNavi result follows.

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

The machine must also keep every live CPU execution resource reachable for its
whole admitted lifetime. A machine-owned custody slot is a resource owner,
not a second task registry: it refers to canonical births at context use,
while the CPU backend owns the opaque engine/context implementation. The slot
itself binds the exact process birth and a checked owner generation; active
and displaced context locators separately carry task/frame provenance.
Scheduler decisions revalidate the canonical HandleTable, never infer
lifecycle from custody membership.
Scheduler ready/parked indexes may hold checked slot locators but never become
the sole owners of CPU boxes. A slot has a checked incarnation so an old
locator cannot acquire a replacement; one slot per process is not assumed
while same-process displaced contexts exist. A run lease borrows custody
without moving the only owner into an unprotected local variable. This is a
target contract, not a description of the current parked queues.

Nested native callbacks require a stronger loan boundary than CPU custody
alone. A suspended parent may retain its native call stack while a different
child slot holds the one executable machine permit. A's per-slot CPU payload
loan may remain on that paused stack, but no mutable custody directory or
topology, Kernel, Task/frame or framebuffer loan may span the child run; A's
payload cannot execute while its permit is suspended. Hook access to
machine state must be callback-scoped, fallible and tied to the current leaf
permit; reentrant or busy access fails closed rather than borrowing an
already-loaned object through a raw pointer. A checked parent-suspension token
resumes only after the child is classified and its resource is in custody.
The current raw Kernel/framebuffer hook pointers do not yet meet this target.

A generic `with_cpu` payload loan does not authorize execution. Backend entry
requires an exact active-leaf run permit; a generation-checked slot-loan and
retire latch must refuse slot retirement or reuse while any payload loan or
run permit is outstanding. A private stable per-slot cell alone cannot enforce
that lifetime rule. The proposed callback-scoped machine-state boundary also
does not, by itself, prove every existing pending-return `RefCell` safe.

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

### Exit-effect ownership (target contract)

Process or task exit is a transition of one verified canonical birth, not a
numeric PID/TID command. Before any teardown effect, guest-initiated exit
needs a captured caller lease/frame, while stopped host completion needs its
verified launch/stop witness and canonical exiting birth. A mutable selected
PID or a freshly resolved occupant of the same number cannot authenticate an
old caller. Each consumer identifies its affected windows, backing stores,
sends, waits and payloads from frozen creation/admission ownership, then revalidates
exact identities before mutation. Numeric IDs remain public projections and
diagnostic labels, never the sole selector for process-exit effects. An
unknown-owner compatibility record cannot silently be claimed by a later
birth that reuses its numeric ID.

A parked CPU nested under another CPU is an execution/scheduling container,
not evidence that its process is owned by the container's process. Exiting
one verified process birth may retire only that birth's execution resources;
a separately live foreign-birth CPU must remain reachable by the scheduler.
Before a stopped-exit effect, partition any CPU subtree that would be removed
into exact exiting-birth resources and verified live survivors. Unknown,
stale or conflicting ownership refuses completion before mutation; a
successful exit moves survivors without copying contexts or inventing guest
execution. The current parked-tree implementation does not yet satisfy this
target contract. Local CE6 process/thread source and bounded dumped-COREDLL
wrapper analysis are recorded in the
[exit evidence](../evidence/ce6-rea-exit-20261016/findings.md); the dumped
kernel's recursive child-exit behavior has not been directly recovered.

Cancellation, reply and wake effects are keyed to the selected exact object or
send IDs and occur once. An unavailable owner or failed preparation must stop
before publication; an irreversible partial teardown must be reported as such,
not as a completed exit. Guest-requested `DestroyWindow` recursion and
machine-driven process-exit target selection are different operations; the
policy for foreign parent/owner links remains an open GWES decision. This is
the shared boundary for Stage 3 designs, not a claim about current Kernel/GWES
implementation or CE6 topology.

For stopped host exit with saved window backing, [D-011](DECISIONS.md)
chooses strict framebuffer preflight: if any selected backing cannot restore
to the supplied destination, refuse completion before **any** exit effect and
hold the stopped process for recovery. Do not turn that refusal into an
ordinary CPU continuation or a successful best-effort exit. The bounded
[D-011 gate](../evidence/stage3-host-exit-framebuffer-preflight-20261015/validation.receipt.md)
source-tests this refusal for saved live and already-destroyed HWND backing,
including partial multi-window failure and active/parked/pending scheduler
propagation. The later selected-owner control refuses a boot/root exit whose
saved CPU owner no longer matches the selected process. The monitor blocks a
new checkpoint after refusal. The later
[canonical-map checkpoint guard](../evidence/stage3-checkpoint-orphan-frame-green-20261016/validation.receipt.md)
also refuses save/restore while any canonical Task record/launch or CPU
custody exists, including an exited thread retaining a Task frame; an earlier
live checkpoint cannot currently clear the hold. Until atomic machine
relineage exists, recovery means repairing the cause and restarting rather
than cloning stopped execution. It does not make later teardown effects atomic or complete the
machine-owned Task/VM lifecycle. The later
[modal-presence cut](../evidence/stage3-modal-exit-failclosed-20261016/validation.receipt.md)
refuses affected modal CPU subtrees in three red-to-green stopped-exit shapes;
it does not restore modal backing or preserve nonmodal descendant CPUs on a
successful exit.

### Stage 3 pseudo self-exit convergence (bounded source gate)

The [native source gate](../evidence/stage3-native-pseudo-self-exit-20261016/validation.receipt.md)
and [decoded source gate](../evidence/stage3-raw-decoded-pseudo-exit-20261016/decoded-pseudo-exit.receipt.md)
now reach the same stopped, birth-checked finalizer after CPU-owner closure.
The decoded path requires a same-attempt live JALR/delay-slot witness; static
PC-zero or exit-shaped bytes alone cannot retire a process. D-011 still
requires saved-image restoration preflight before any exit effect and typed
recovery refusal on failure. The raw COREDLL pseudo-exit shim remains
fixture-only and fail-closed for guest callers; there is no raw executor.
These are bounded source/test gates, not proof of complete machine-owned
Task/VM retirement, post-preflight failure atomicity, live iNavi behavior or
a production decoded PC-zero hit. The target remains one machine-owned exit
transaction with those additional effects and failure gates.

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
Prepared waits revalidate their exact live task birth at publication; a stale
candidate releases its prepared pins without becoming a scheduler frame.

Admitting a child CPU, running its first slice and classifying its stop are
one custody lifetime even if the parent launches it synchronously. A failure
before a CPU exists resolves as either an explicit CE creation failure with
no CPU or a retryable error with the exact pending launch retained; neither
may orphan an admitted child. After admission, a refused exit, native error
or publication conflict leaves the exact CPU tree in custody and reports a
stopped/terminal outcome. A synchronous in-hook launch suspends/pins the
parent's execution lease while the child takes the exclusive active lease;
the parent is restored only after the child is classified and safely in
custody, without a mutable frame/queue borrow across the nested run. The
machine cannot leave Kernel execution live while dropping its only CPU owner.
Prepare a survivor's checked destination before irreversible exit effects;
after accepted completion its ownership move must be infallible, or a
partial-effect failure must retain
custody and be reported honestly rather than claiming rollback. An
already-exited launch handle is not proof that the returned CPU belongs to
that launch: rotation may have selected a different still-live process, so
discard requires verification of the CPU's own exact birth. The machine
sequencer must not allow host/framebuffer reentry to mutate a queue during
this transition. The current first-slice helper's terminal owner-loss branch
does not yet satisfy this contract.

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

For external input, a guest `GetMessage`/`PeekMessage` HWND is a retrieval
filter, not authority to choose or reject the input destination. Coordinate
touch resolves against live capture and delivery-time hit-testing; a blocked-
wait or modal HWND can only be a fallback hint after those checks. An explicit
host-targeted HWND is different: its frozen owner must still be live, or that
event is consumed and dropped without rerouting. Keys follow a live focus or
active owner, not mouse capture. Source gates must test these distinct routes
without treating CE-family API documentation as proof of CE6 internals.

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
