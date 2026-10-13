# E29 follow-up: bounded post-120 s native-attempt trace

Status: **design only; no instrumentation or app rerun has been executed**
(2026-10-14). The [E29 bounded run](../../evidence/e22-live-20261013T211230Z/analysis.receipt.md)
remained on the genuine splash at its 330 s host cap. Its service counters
plateaued by about +120 s while host CPU and sampled PCs changed. That is a
question about where host and guest time went, not proof of a scheduler spin,
missed wake, failed WNDPROC, or native count exhaustion. This diagnostic
narrows the [E28 detached report](e28-run-outcome-contract-design.md) and
[Architecture A3/A5/A6](../ARCHITECTURE.md); it does not advance Stage 2 or
give a legacy frame Task authority.

## Owner, boundary, and question

The CPU backend owns the raw, **outgoing** primary native-attempt facts.
`NativeAttemptReport` already carries a host-process-local checked attempt ID,
captured process birth/PID/TID, start PC, raw pre-repair stop PC, native result,
`COUNT_STOP`/timeout query observations, hook/PC-stop facts and final adapter
outcome. Its `LegacyUnbound` frame cannot authorize a Task transition.
`main.rs` owns the host-run clock and a diagnostic writer. The remote observer
owns separately timestamped frame/window/input samples. No diagnostic sink
selects a task, changes a stop reason, drains an event, repairs a PC, or
publishes a scheduler-visible outcome.

The first question is whether +100 to +180 s contains primary native attempts,
which outgoing PID/birth/TID and PCs they actually execute, and how much host
time each takes. The second is whether actual overlay-window imports and
WNDPROC continuations occur in those same attempts. A selected CPU's later
`last_debug_snapshot` is **not** an attempt join key: the ordinary success tail
may rotate to a child after the outgoing report is captured. The throttled
remote debug pages also combine live kernel views and retained CPU snapshots;
matching cached PIDs or close timestamps is only suggestive.

## Proposed read-only interface and record

An explicit opt-in diagnostic path in the run command enables one JSONL writer
outside `target/`. Disabled mode keeps current execution and logging behavior.
Use the `Instant` already created at `run_cpu_loop` entry as the host-clock
origin. In monitor mode, use its own `run_started` and label that distinct
origin. Do not substitute CE ticks, guest time, remote input-ready time or
the configured native timeout. Record the wall-clock origin and host process
ID in a run header; `DiagnosticAttemptId` restarts in a new host process and
is never globally unique by itself.

- Entered-at-host `[100_000, 180_000)` ms is the detailed capture window.
  Bracket the CPU call with host start/end offsets and whole-call elapsed time;
  mark an attempt crossing either boundary. Emit a late summary for a call
  entered before +100 s but returned inside/after the window, with
  `events_uncollected`, rather than silently implying no work. A host-crashed
  or never-returning call still needs the external watchdog and memory log;
  the in-process writer cannot promise a terminal record.
- The per-attempt line retains `result` separately from `report`: `NoAttempt`
  and its preflight operation have no fabricated native facts; a failed run
  still emits its report before the existing error path returns. Include
  attempt ID, start/end process birth/PID/TID (end birth currently copies the
  captured lease birth), start PC, raw native stop PC, final outcome,
  failure operation, partial-effects flag, native result, count-stop and
  timeout observations including `Uncollected` and `Error`, requested-PC
  match, hook reason, and configured limits. Label the frame
  `LegacyUnbound`. Record whole-call and primary `emu_start` host durations
  separately; neither is GuestInstant.
- If guest work volume is needed, carry the existing per-slice basic-block
  code-byte tally as a separately labelled `block_bytes/4` estimate and
  relate it to the same attempt ID. `COUNT_STOP = true` means the native count
  hook requested a stop, not that exactly the configured instruction count
  retired, not an exclusive stop cause, and not a CE quantum. Preserve raw
  competing stop/query facts even when `final_outcome` has one value.
- Add a 1 Hz, cheap top-of-live-loop heartbeat during the same host window:
  loop iterations since last heartbeat, native attempts since last heartbeat,
  selected PID/TID and saved PC labelled **selected, not executed**. Many
  current main-loop branches `continue` before the CPU call. An empty attempt
  stream without this heartbeat cannot distinguish scheduler-only churn from
  a stalled writer or long native call. The heartbeat does not sample large
  debug snapshots or alter the branch decisions.

For overlay correlation, use only existing import-trap and WNDPROC paths: a
bounded optional per-attempt collector in the backend, armed by the host window
before the call, receives the allocated attempt ID immediately before the
primary `emu_start`. Record a monotonic **within-attempt** event ordinal and
host offset, actual executing TID, event phase (import entry/return, WNDPROC
entry/return where observable), HWND/message or import ordinal, arguments,
result, and the observed window-owner PID/TID if looked up at event time.
Scope import families to `ShowWindow`, `DestroyWindow`, `SetWindowPos`,
`InvalidateRect`/`ValidateRect`, `BeginPaint`/`EndPaint`; scope callbacks to
display lifecycle/paint/position/destroy messages. Capture all matching
HWNDs within the bounded window, then identify E29's `0x00020008` overlay
and `0x00020004` main window in analysis; those numerical handles are
run-specific diagnostic targets, never behavior keys. Do not install a new
global code hook. Host-side GWE or callback preparation outside a native
attempt is unbound to that attempt: record it separately with a host timestamp
or leave it explicitly unjoined. Repeated HWND/message tuples alone do not
prove an entry/return pair; use an actual continuation identity before making
that stronger claim.

The write path is bounded: at most 128 compact events per attempt, with a
per-type dropped count; at most 32 MiB JSONL total, with a truncation marker
and final counters; a small buffered writer flushed at least once per second
and at normal exit. These are proposed caps to verify under the focused
controls, not evidence of adequate capacity in a real app run. Overflow may
not abort or change the guest. A write failure disables the diagnostic sink,
reports one host-side error, and invalidates trace acceptance; it must not
convert a successful CPU result into an emulator failure. No absence claim is
valid across truncated, dropped, missing, or uncollected intervals. No guest
or mounted file is modified by this diagnostic, so disabling the opt-in path
is its execution rollback; retained evidence remains in `evidence/`.

## Source boundary and expected-red controls

The contemplated edit surface is `src/main.rs::run_cpu_loop`,
`monitor_run_once`, and `DesktopRuntime::run_cpu_until_report` (currently the
live and monitor callers use the Result wrapper); `src/emulator/types.rs`
for compact diagnostic event/metric transport if necessary; and
`src/emulator/unicorn.rs::run_with_unicorn_attempt` around attempt ID/native
return, the existing import trap/return and WNDPROC return paths. No
`src/ce/scheduler.rs`, task policy, remote HTTP endpoint, guest API shim,
vendored Unicorn C, or per-instruction hook change is proposed. The
`src/ce/kernel.rs` message/window rings and `src/remote_server.rs` debug
pages remain contextual evidence, not exact joins.

Stage these controls as **expected-red before implementation**, then require
them green on the implemented source; none is claimed executed by this note:

1. `post120_trace_window_uses_host_elapsed_not_guest_ticks`: at 99,999,
   100,000, 179,999 and 180,000 ms, only entered-at-window attempts get
   detailed capture; a boundary-crossing call is labelled, not omitted.
2. `post120_trace_no_attempt_and_error_keep_raw_provenance`: preflight has no
   ID/native facts; real-MIPS native/query/backend failures preserve their
   raw observations and error precedence, with writer publication before
   the ordinary error return.
3. `post120_trace_child_handoff_keeps_outgoing_owner`: a real-MIPS parent
   attempt that launches a child emits the parent's birth/PID/TID/PC/events,
   not the selected child's; the child receives a different attempt ID.
4. `post120_trace_overlay_events_join_only_actual_attempt`: a real-MIPS
   fixture's window import and WNDPROC continuation share the attempt ID
   only when executed inside that native call; a host-side preparation or
   retained prior return must be unjoined. An interrupted callback must not
   be reported as a completed return.
5. `post120_trace_caps_and_writer_failure_are_observational`: exercise
   per-attempt and file caps, drop/truncation markers and write failure; no
   memory growth without bound, scheduler change, or transformed CPU result.
   A 1 Hz heartbeat still exposes a loop with zero native attempts.

## Acceptance recipe and interpretation limits

After review, run format/diff checks, focused real-MIPS controls, standard
release, Unicorn-only and Unicorn+desktop/no-trace checks, and no-feature
tests on the exact D: source. Record source/executable/trace hashes and raw
commands. Then perform one separately authorized, bounded 330 s live run
with the configured `192.168.0.39:8765` remote endpoint, current adjacent
ZIP/hash-preservation preflight for writable SD/ResidentFlash/emulator-root
mounts, the same raw-NMEA and tap timing, host memory/watchdog sampling,
frame hashes and timestamped endpoint captures. The approved writable-mount
run is a new experiment, not a replay of E29's exact input payload.

Analyze attempted versus no-attempt seconds first. Group attempt records by
outgoing birth/PID/TID, start/raw-stop PC and outcome, with total/median/max
host call/native durations and independent count/timeout facts. Join overlay
imports and WNDPROC events **only** on the same run and attempt ID; compare
host-timestamped remote frame/window state as a looser observation. Record
trace drops, host overhead, hook mode and guest block-byte/MIPS accounting
before claiming a performance change. A repeated PC, positive count query,
event ACK, or splash frame alone is not a root cause or map readiness.
E29's blocked wait 833/TID 10 also needs its actual wait-any/all mode,
object incarnation/generation and signal/acquire ordering recorded in a
separate targeted control before claiming a missed wake. This diagnostic
does not implement that wait control or close O-P1, Stage 2, S3-A or map
acceptance.
