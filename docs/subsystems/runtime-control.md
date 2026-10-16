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

### Candidate callback-scoped machine-state port (not implemented)

The [anchor](../ARCHITECTURE.md) A1/A3 requires one executable leaf permit and
no mutable Kernel, Task/frame, custody-directory or framebuffer loan across a
nested child run. The [Task/custody plan](tasks-scheduler.md#candidate-o-t7-slot-and-lease-contract-for-review-no-rust-api-exists)
inventories the current raw-pointer hook families. Runtime composition must
provide one checked machine-state port to those consumers, not an additional
Kernel copy or another scheduler. `MachineKernelPort` and its operations below
are candidate responsibilities, not Rust APIs that already exist.

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

The current `run_with_unicorn_attempt` still holds `&mut CeKernel` and a
framebuffer reference across `emu_start` while hooks use raw pointers; the
post-native pending-launch helper also holds a mutable Kernel reference across
the child run. An independent memory-fault first-touch witness reaches that
hook, but does not validate the proposed port. Design and source-gate each
hook family, parent/child handoff and reentry/error case before claiming a
sound machine-owned execution lease.

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
