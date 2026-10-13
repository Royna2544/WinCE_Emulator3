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
