# Native-attempt observability

Status: **design only**. No writer, event collector, test result, or live-app
trace is claimed here. The shared contracts are [Architecture A3, A5 and
A6](../ARCHITECTURE.md); implementation follows the gates in the
[plan](../PLAN.md). The [E29 post-120 s design](../evidence/e29-post120-native-attempt-trace-design.md)
is one proposed run recipe, not a second ownership contract.

## Ownership and interface

The CPU backend owns raw facts for one **outgoing primary native attempt**:
attempt ID, captured process birth/PID/TID, start and raw stop PC, native
result, independent count/timeout queries, hook requests, final adapter
outcome, failure operation and possible partial effects. It returns those facts
with the checked `result` as `AttemptExecution`. `NoAttempt` means native entry
did not occur; an uncollected field is not a false or zero observation. The
report's current `LegacyUnbound` frame is diagnostic, not a canonical Task or
Frame identity. The backend may later provide an opt-in, bounded event batch
from actual import and return paths, but it does not own files or host clocks.

The host run loop owns the opt-in diagnostic writer, host-monotonic origin,
host-local call ordinal, window and file budget. It records a flushed
`call_enter` before the CPU wrapper and a `call_end` carrying the returned
report **and separate result** before caller error handling or later loop
maintenance. A child rotation may change the selected CPU before the host
receives the report; the writer must retain the report's outgoing owner rather
than reconstruct it from a later snapshot. Monitor and live-loop origins are
labelled separately. Host elapsed, native-call duration, guest time and CE
quantum are different quantities; no configured timeout proves expiry.

The remote/frame observer owns independently timestamped external samples.
Until one immutable `SnapshotEpoch` spans components, close timestamps or
matching cached PIDs do not establish an atomic join. Observation never
selects a task, changes a stop reason, consumes a wait/input event, repairs a
PC or commits a continuation. Only the machine may commit a validated
Task/Frame transition under A3; a trace is not that authority.

## Staged diagnostic boundary

Stage A is an explicitly enabled, read-only attempt and heartbeat stream.
Disabled mode performs no file work and leaves scheduling and guest behavior
unchanged. A run header identifies source/binary/config, host process and
clock origin. Entered calls within the configured host window get detailed
enter/end records; calls crossing a boundary are marked, not silently counted
as absent. A top-of-loop heartbeat reports that the loop is turning and the
number of native attempts since the last heartbeat. It cannot fire during a
blocked CPU call; a flushed unmatched `call_enter` proves only that no
`call_end` was captured. A zero-attempt interval, host crash and missing writer
output must remain distinguishable. The E29 design specifies one concrete
window and live-run capture recipe.

Stage B, after Stage A's owner/cap controls pass, may collect selected overlay
imports and genuine WNDPROC returns in the backend. Join an event to an
attempt only when it executed inside that native attempt and carries its ID;
host-side callback preparation and older snapshots remain unjoined. A shim
dispatch is not necessarily a completed guest return, and a return-stub hit
does not by itself prove a particular prior entry without continuation
identity. Bound collection before formatting or copying large guest payloads.
Do not add a global code hook, app-specific behavior key or forced callback.
The E29 design owns the detailed overlay event list and run-specific HWNDs.

## Failure, limits and evidence

Set explicit caps for encoded event size, events per attempt, total file bytes
and reserved footer space before enabling the sink; the E29 proposal supplies
initial values to validate. Count dropped events by type, saturate counters,
mark truncated fields and cap exhaustion, and emit a final completeness marker
when normal shutdown permits. Flush entry records before potentially long
native calls. A writer error disables only the diagnostic, reports one host
error and makes trace acceptance fail; it must not transform the CPU's result
or abort guest execution. Neither an incomplete file nor a retained requested
filename is terminal evidence. Preserve logs and hashes outside disposable
`target/`; do not write diagnostic data into guest mounts. Measure overhead,
guest block work and host RSS before comparing boot latency or memory.

## Pre-implementation and acceptance controls

Before source changes, review the producer/consumer and failure paths, stage
expected-red controls, then implement Stage A without changing scheduler
policy. Controls must cover host-window edges and crossings; `NoAttempt`,
native/query/backend errors and their raw provenance; outgoing parent identity
after child handoff; a turning loop with no native entry; an unreturned call;
file cap/footer/drop markers; and writer failure with an unchanged CPU result.
Stage B additionally needs actual-native import/return joins, interrupted
callbacks that are **not** labelled complete, outside-attempt events left
unjoined, and bounded overflow. Review independently, run focused real-MIPS
and full applicable profiles on the final source, and record exact hashes and
raw results. Only a separately bounded, phase-aligned live run can interpret
app behavior. These diagnostics cannot close O-P1, Stage 2, Task/Frame
authority, map readiness or responsive navigation on their own.
