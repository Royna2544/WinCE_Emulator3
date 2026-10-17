# Runtime control and host composition

Common anchor: [ARCHITECTURE.md](../ARCHITECTURE.md), especially A1–A3/A6.
Implementation order: [PLAN.md](../PLAN.md). Target interfaces are proposals.

## Current boundary

[main.rs](../../src/main.rs) loads configuration/PE images, composes desktop,
kernel and CPU, runs the live loop, publishes remote state and hosts monitor
commands/checkpoints. The CPU facade owns both execution glue and much of the
current scheduling/parked-process logic. Pre-run maintenance and slice-end
rotation can change owners; a debug snapshot is not necessarily live state.
The installed BED2 fallback differs from the failed 650 experimental source.
`DesktopRuntime` (its enum, methods and desktop selection) lives in
`src/main.rs`; the Windows host input and presenter it composes live in
`src/ce/win32_desktop.rs`, not under `src/emulator/`.

## Target owner and interfaces

The composition root constructs one CeMachine and explicit host adapters.
The machine, not CLI/monitor/desktop, commits execution selection and state
transitions. Configuration supplies CE/native policies once; environment
switches must not become alternate hidden state authorities.

- `MachineCommand` requests run, bounded diagnostics or validated checkpoint
  operations. It cannot force an app callback or readiness event.
- `MachineStep` produces the anchor's RunOutcome plus a SnapshotEpoch.
- Desktop/remote sinks receive immutable presentation/observation views.
- Host adapters submit sequenced external completions through the machine.
- Monitor exact stops retain owner/frame/context and cannot rotate or repair
  a different continuation before the requested stop is handed back.

No frontend directly edits task priority, guest time or backend internals.
Checkpoint restore is an explicit machine boundary with independent mutable
kernel/VM/engine state; rewind is not valid performance/run evidence.

The bounded [D-011 stopped-exit gate](../../evidence/stage3-host-exit-framebuffer-preflight-20261015/validation.receipt.md)
propagates a typed recovery refusal from an un-restorable saved window image
through tested active, parked and pending launch paths; a stopped boot CPU with
a mismatched selected owner is refused too. The monitor holds
further execution, tap input and new checkpoints after this refusal. The later
[live-checkpoint lineage guard](../../evidence/stage3-checkpoint-orphan-frame-green-20261016/validation.receipt.md)
also refuses restore while canonical Task/launch authority or CPU custody is
retained. An earlier live checkpoint cannot clear this hold today: repair the
cause, quit and restart. These source-tested boundaries do not establish
whole-exit atomicity after preflight or live-app recovery.

### Callback-scoped machine-state port (bounded foundation; full lease open)

The [anchor](../ARCHITECTURE.md) A1/A3 requires one executable leaf permit and
no mutable Kernel, Task/frame, custody-directory or framebuffer loan across a
nested child run. The [Task/custody plan](tasks-scheduler.md#o-t7-slot-and-lease-contract-partial-foundation-connected-cuts-open)
inventories the hook families. The bounded `MachineKernelPort` source cut
already removes the named raw Kernel/framebuffer hook aliases and uses
fallible short loans across the reviewed production path. Its process-scoped
attempt check is not an independently directory-issued exact execution leaf.
Runtime composition still needs one checked machine-state port for all
remaining consumers, not an additional Kernel copy or another scheduler.
The following full-leaf and restoration responsibilities remain targets.

- A hook presents its exact active-leaf permit, Task/frame and attempt to a
  fallible, callback-scoped Kernel or framebuffer operation. The operation
  resolves current authority, lends only the required state, and ends its loan
  before returning to native execution, a nested guest run or a reentrant host
  callback. A busy,
  stale or wrong-leaf request returns a typed failure; it never uses a
  panicking borrow or an unchecked raw-pointer alias.
- Multi-step work carries an exact-birth claim and checked generation/version
  across short loans. Revalidate at commit; a previously selected numeric PID
  is compatibility state, not proof of the running CPU's ownership. A checked
  restoration token returns that selection after a child run without
  overwriting a newer owner or fabricating a parent continuation.
- An exit-effect transaction fences host/framebuffer reentry from mutating
  Kernel, custody or topology under its loan. Incoming observations may be
  queued for later machine-sequenced service. Refusal before effects preserves
  the stopped owner; failure after irreversible effects retains non-runnable
  custody and reports partial effects, not a successful exit or rollback.

The outer `run_with_unicorn_attempt` still receives Kernel/framebuffer
references, but the reviewed native and post-native path passes them through
the connected port; its named raw aliases have been removed. That bounded
gate does not connect boot to directory-owned runnable CPU slots or issue a
full leaf permit. Reentry/error cases and remaining consumers still need
their own evidence before claiming a sound machine-owned execution lease.

### O-T7a post-native host-turn protocol (candidate; not connected)

The host, not an active `UnicornMips` wrapper, must commit the next runnable
owner. One typed host-turn request covers ordinary execution, neutral Event
service and monitor entry; pre/post rotation may retain today's policy but
must submit selection through the same directory gate. A request freezes its
producer's exact process birth, slot key/generation and attempt, native stop
report and staged return PC. A deferred `CreateProcessW` request additionally
carries the attempt-local publication/Kernel claim and checked parent
selection witness. Neutral Event is not a synthetic CreateProcess child: it
has its own reason and checked previous-selection restoration token.

On native return, A's callback and `emu_start` have ended, but A's native
owner and post-child view-sync/finish tail may remain. Before B enters, seal
that tail as an A-owned suspended continuation and revoke A's executable
process permit. A's slot may retain a scoped payload loan to pin native
resources; no Kernel, framebuffer, queue, Task/frame or custody-topology loan
may span B. The host reserves/constructs/admits B into its exact-birth slot
before its first guest instruction, issues B's distinct process permit and
classifies its stop back into custody. A witnessed B→C repeats this sequence;
a C-root/B-parked return creates distinct C/B locators from each original
CPU, not from the launch label or selected PID. Only then does A's exact tail
resume for view sync/native close and checked selected-context restoration.
Reissue A only after that tail and every child classification pass. This is
a process-scoped Cut 1 protocol; Cut 2 adds intra-slice thread/frame leaves.

Pre-CPU setup failure returns the exact claim for same-birth retry. A stale
claim or selection, failed admission, exited parent, native child fault,
reentry conflict or post-effect exit failure reports a typed terminal or
partial-effects outcome with every admitted CPU still reachable and
non-runnable; no fallback may execute an unported route. The connected gate
requires boot and ordinary/Event/monitor route tests, pre/post rotation,
A→B and witnessed A→B→C later slices, process/slot/attempt mismatch refusals,
and from-boot checkpoint refusal. A separate exact thread/frame gate follows.

## Failure, lifetime and observation

Startup publishes a runnable task only after actual image/entry preparation.
An owned run has explicit host watchdog bounds even if no CPU slice executes.
Guest/native failure and host process crash are not normal wall-limit exit.
Flush evidence where possible, report missing terminal artifacts honestly,
and preserve the prior installed binary before replacing a candidate.

Only the exact process created by a test wrapper may be stopped automatically.
An existing emulator is never silently replaced. Input/capture helpers have
their own clocks/horizons; end them or report post-target failures when the
owned target ends. User SD files and synced project references are not cleanup.

## Plan and gates

Inventory main/monitor/CPU-loop policy callers before moving selection into
CeMachine. Replace one connected path and remove the previous writable
authority; do not layer another orchestration loop over it. Keep count/policy
unchanged while validating composition/lifetime, then follow the outcome and
clock gates. Verify loop-head bounds, exact debugger stops, process entry/exit,
clone isolation, remote responsiveness and explicit source/binary identity.
Several real bounded runs are required; unit success does not establish map.
