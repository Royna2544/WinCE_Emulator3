# Native-attempt observability

Status: **design only**. No writer, event collector, test result, or live-app
trace is claimed here. The shared contracts are [Architecture A3, A5 and
A6](../ARCHITECTURE.md); implementation follows the gates in the
[plan](../PLAN.md). The [E29 bounded run](../../evidence/e22-live-20261013T211230Z/analysis.receipt.md)
remained on the genuine splash at its 330 s cap: service counters plateaued
around +120 s while host CPU and sampled PCs changed. That does not prove a
scheduler spin, missed wake, WNDPROC failure or count exhaustion. The E29
recipe below asks where host and guest time went during +100 to +180 s; it is
not a second ownership contract or a claim of map readiness.

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
`DiagnosticAttemptId` is checked but only host-process-local: pair it with
run identity, never assume global uniqueness. `COUNT_STOP = true` records a
count-hook request, not an exact retired-instruction count, exclusive cause or
CE quantum. If work volume is needed, attach the existing per-slice
`block_bytes/4` estimate to the same attempt, labelled as an estimate.

The host run loop owns the opt-in diagnostic writer, host-monotonic origin,
host-local call ordinal, window and file budget. It records a flushed
`call_enter` before the CPU wrapper and a `call_end` carrying the returned
report **and separate result** before caller error handling or later loop
maintenance. A child rotation may change the selected CPU before the host
receives the report; the writer must retain the report's outgoing owner rather
than reconstruct it from a later snapshot. Monitor and live-loop origins are
labelled separately. Host elapsed, native-call duration, guest time and CE
quantum are different quantities; no configured timeout proves expiry. The
header pairs UTC with the `Instant` created at live `run_cpu_loop` entry (or
the monitor's distinct `run_started`), plus host PID and run/config identity.
Observer launch/input-ready clocks have separate origins; their UTC samples
permit only an explicitly skew-bounded comparison until `SnapshotEpoch` exists.

The remote/frame observer owns independently timestamped external samples.
Until one immutable `SnapshotEpoch` spans components, close timestamps or
matching cached PIDs do not establish an atomic join. Observation never
selects a task, changes a stop reason, consumes a wait/input event, repairs a
PC or commits a continuation. Only the machine may commit a validated
Task/Frame transition under A3; a trace is not that authority.

## Staged diagnostic boundary

Stage A is an explicitly enabled, read-only attempt and heartbeat stream.
Disabled mode performs no file work and leaves scheduling and guest behavior
unchanged. E29's detailed window is **host elapsed [100,000, 180,000) ms by
call entry**, not guest ticks, native timeout or remote input-ready time. For
each in-window call, flush `call_enter` before the desktop/CPU wrapper with a
distinct host call ordinal, selected PID/TID/PC and host offset. The backend
flushes `native_enter` after assigning the attempt ID and capturing the
*actual* starting owner/PC, immediately before primary `emu_start`; it emits
`native_end` immediately after native return with raw result and native-call
duration. Then `call_end` reports checked queries, raw outgoing report,
separate checked result, end offset and whole-wrapper duration before error
return or next loop maintenance. A failed run still publishes its report;
`NoAttempt` carries its failed preflight operation but no fabricated native
facts. `native_end` may precede a failed query/tail and is not an independent
success result. Label the report's end birth as copied from its captured
lease, not newly observed after child selection. Keep count/timeout
`Uncollected` and `Error`, requested-PC match, hook reason, limits, failure
operation and partial-effects flag, even if one final outcome is chosen.

Join by run identity, call ordinal and attempt ID where assigned, never by
line adjacency, PID or a selected CPU's later `last_debug_snapshot`. The
success tail may rotate to a child. `call_enter` without `native_enter`
localizes a gap before backend entry; `native_enter` without `native_end`
localizes it to native execution or a host crash there; `native_end` without
`call_end` points to the checked tail or later failure. These are locations,
not root causes; writer failure/truncation and the external watchdog govern
absence claims. The outgoing report must capture identity/raw facts before
an in-call child handoff, although the host's post-return line may follow that
handoff. Mark calls crossing either window edge. A call entered before +100 s
but returning inside/after the window gets a late summary marked
`events_uncollected`; if it never returns, the bounded capture alone cannot
prove the loop was idle.

Synchronous `run_pending_process_launches` may execute a child inside one
wrapper call and discard its detached report through a `Result<()>` wrapper.
The diagnostic observer must emit that nested `attempt_end` before discard,
with its **own** attempt ID, parent call ordinal and nesting marker; parent
`call_end` reports only the parent's outgoing attempt and the whole-wrapper
result. Native stages may interleave. If a path cannot transport the child
report, mark `nested_attempts_uncollected` and reject complete attempt/work
totals for that interval. The observer has no Kernel/Task/Frame authority.

A cheap heartbeat at the top of the turning live loop is due at most once per
second inside the same window: loop iterations and native attempts since the
previous heartbeat, selected PID/TID and saved PC labelled **selected, not
executed**. It does not sample large debug snapshots or alter branch choices.
It cannot fire during a blocked wrapper/native call; a missing heartbeat is
not proof of a scheduler stall. The flushed stages plus independent host
evidence distinguish no-attempt churn from an entered long call.

Stage B, after Stage A's owner/cap controls pass, may collect selected overlay
imports and genuine WNDPROC returns in the backend. Join an event to an
attempt only when it executed inside that native attempt and carries its ID;
host-side callback preparation and older snapshots remain unjoined. A shim
dispatch is not necessarily a completed guest return, and a return-stub hit
does not by itself prove a particular prior entry without continuation
identity. Bound collection before formatting or copying large guest payloads.
Do not add a global code hook, app-specific behavior key or forced callback.
Collect an event ordinal within the attempt, host offset, executing TID,
phase (import entry, observed dispatch result or observed WNDPROC return),
ordinal/HWND/message, compact arguments/result, observed window-owner PID/TID
and post-call visibility/parent/rectangle/z-order where available without new
guest work.
The explicit import filter includes `ShowWindow`, `DestroyWindow`,
`SetWindowPos`, `InvalidateRect`/`ValidateRect`, `BeginPaint`/`EndPaint`,
`SetWindowLongW(GWL_STYLE)`, `SetParent`, `MoveWindow`, `BringWindowToTop` and
`DeferWindowPos`. Scope callback returns to display lifecycle, paint,
position and destroy messages. The existing `is_window_import` ring omits
some of these APIs, and GWE `show_window` records changes rather than every
request; silence in either ring cannot prove no call. Capture all matching
HWNDs, then identify E29's `0x00020008` overlay and `0x00020004` main window
only during analysis; these numbers are run-specific diagnostics, never
behavior keys. Host-side GWE/callback preparation outside an attempt is
host-timestamped separately or explicitly unjoined. A prepared callback is
not evidence its guest instruction ran; repeated HWND/message tuples alone
do not establish an entry/return pair.

## Failure, limits and evidence

The proposed cap is **32 MiB JSONL total**, including header, truncation
marker and final counters, reserving **4 KiB** for the footer before the cap.
Stage B additionally allows at most **128 compact events per attempt** and
**1 KiB encoded per event** (truncate diagnostic text explicitly), with
per-type saturating drop counts. Verify these caps under focused controls;
they are not evidence that a real app run fits. Flush `call_enter` and
`native_enter` before their respective calls, then at least once per second
while the loop turns and at normal exit. A writer error disables only the
diagnostic, reports one host error and fails trace acceptance; it must not
transform the CPU result or abort guest execution. No absence claim is valid
across truncated, dropped, missing or uncollected intervals. Preserve logs
and hashes outside disposable `target/`, never in guest mounts. Disabling
the opt-in path is its execution rollback; retained evidence stays in
`evidence/`. Measure overhead, guest block work and host RSS before comparing
boot latency or memory.

## Pre-implementation and acceptance controls

Contemplated Stage A edit surface is `src/main.rs::run_cpu_loop`,
`monitor_run_once`, `DesktopRuntime::run_cpu_until_report`, compact metric
transport in `src/emulator/types.rs` if needed, and
`src/emulator/unicorn.rs::run_with_unicorn_attempt` plus its nested child
handoff. Stage B alone adds existing import/return and WNDPROC-return paths.
Do not change scheduler policy, guest shim behavior, remote endpoints,
vendored Unicorn C or install a per-instruction/global code hook. Stage the
following **named expected-red controls before each gate's implementation**;
none is claimed executed by this design:

1. Stage A `post120_trace_window_uses_host_elapsed_not_guest_ticks`:
   99,999/100,000/179,999/180,000 ms edges, crossings and distinct live,
   monitor and observer origins.
2. Stage A `post120_trace_no_attempt_and_error_keep_raw_provenance`:
   preflight without native facts; real-MIPS native/query/backend errors keep
   raw observations and publish before ordinary error return.
3. Stage A `post120_trace_child_handoff_keeps_outgoing_owner`: real-MIPS
   outgoing parent birth/PID/TID/PC and separately ID'd nested child with
   parent call ordinal; unavailable nested report marks an incomplete interval.
4. Stage A `post120_trace_native_stages_locate_unreturned_call`: distinguish
   each incomplete stage, native versus whole-call elapsed, writer failure at
   every stage and footer reservation within 32 MiB.
5. Stage A `post120_trace_heartbeat_exposes_no_attempt_churn`: bounded
   turning-loop heartbeats, none fabricated during a blocked native call;
   overhead and truncation reported.
6. Stage B `post120_trace_overlay_events_join_only_actual_attempt`:
   real-MIPS import and WNDPROC continuation join only when executed in that
   attempt; preparation/prior return unjoined, interrupted callback not
   labelled complete.
7. Stage B `post120_trace_overlay_caps_are_observational`: 128-event,
   1-KiB-event and shared 32-MiB limits, drop/truncation accounting,
   unchanged CPU result; include an API absent from `is_window_import` and a
   callback with no observed return.

After independent review of each implemented gate, run format/diff checks,
focused real-MIPS controls, release all-targets, no-feature all-targets,
Unicorn-only and Unicorn+desktop/no-trace checks on the exact D: source;
record source, executable, trace and raw-log hashes. Gate A's first app
interpretation needs a **newly authorized**, bounded 330 s run with the
configured `192.168.0.39:8765` remote endpoint, fresh adjacent ZIP and hash
verification for writable SD/ResidentFlash/emulator-root mounts, the same
raw-NMEA/tap timing, watchdog/host-memory sampling and timestamped frame,
window and input captures. Stage B needs its own approval and source-matched
run. Analyze attempted versus no-attempt seconds including incomplete nested
intervals, group actual outgoing owner/PC/outcome and native versus wrapper
durations, and join Stage B events only on run plus attempt ID. Remote samples
are looser observations. Record drops, host overhead, hook mode and guest
block-byte/MIPS accounting before a performance claim.

The E29 wait id 833/TID 10 question requires a **separate boot-forward**
wait-lifecycle control recording raw wait-any/all mode, object incarnation,
signal, eligibility/selection, acquisition and removal from registration.
The post-120 s trace cannot reconstruct that earlier registration. A repeated
PC, positive count query, event ACK or splash frame alone is not a root cause
or map readiness. These diagnostics do not close O-P1, Stage 2, S3-A,
Task/Frame authority or responsive navigation.
