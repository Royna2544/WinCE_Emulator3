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
WNDPROC returns occur in those same attempts. This post-120 s trace does **not**
resolve wait id 833: its registration may predate the capture window, and the
current blocked-wait record omits raw wait-any/all mode and object incarnation.
A selected CPU's later `last_debug_snapshot` is **not** an attempt join key:
the ordinary success tail may rotate to a child after the outgoing report is
captured. The throttled
remote debug pages also combine live kernel views and retained CPU snapshots;
matching cached PIDs or close timestamps is only suggestive.
One host wrapper call can also contain a synchronous child run:
`run_pending_process_launches` currently invokes a child through the
`Result<()>` CPU wrapper, losing its detached report. Counting only the
main-loop return would undercount native work. This design keeps that nested
attempt separate, with its own ID and explicit parent call ordinal.

## Gate 1: minimal host-loop and native-attempt record

An explicit opt-in diagnostic path in the run command enables one JSONL writer
outside `target/`. Disabled mode keeps current execution and logging behavior.
Use the `Instant` already created at `run_cpu_loop` entry as the host-clock
origin. In monitor mode, use its own `run_started` and label that distinct
origin. Do not substitute CE ticks, guest time, remote input-ready time or
the configured native timeout. Record a paired host UTC timestamp and
monotonic zero-point at that origin, the host process ID and run/config
identity in a header. Observer launch and input-ready clocks have different
origins; their UTC samples allow only an explicitly skew-bounded comparison
until `SnapshotEpoch` exists. `DiagnosticAttemptId` restarts in a new host
process and is never globally unique by itself. Gate 1 observes execution
and host-loop pacing without installing an overlay collector or changing a
guest operation, task selection, stop reason, wait, callback or frame
publication.

- Entered-at-host `[100_000, 180_000)` ms is the detailed capture window.
  For a call entered in that window, write a small `call_enter` record with a
  host-local call ordinal (distinct from the native attempt ID), host offset
  and **selected** PID/TID/PC, and flush it before invoking the desktop/CPU
  wrapper. The backend then supplies `native_enter` to the host-owned sink for
  emission and flush immediately before the primary `emu_start`, after the
  attempt ID and actual starting TID/PC are known. Emit `native_end`
  immediately after that call returns, with its raw result and elapsed host
  duration; the subsequent `call_end`
  carries the checked queries and final outgoing report, separate `result`,
  end offset and whole-call elapsed time **before the caller's error return
  or next main-loop maintenance**. Join these stages by run identity, call
  ordinal and, where allocated, attempt ID. For an instrumented call, an
  unmatched `call_enter` without `native_enter` localizes the gap to
  desktop/backend pre-entry; `native_enter` without `native_end` localizes it
  to the native call or a host crash during it; `native_end` without
  `call_end` points to the checked tail or a later failure. None identifies a
  root cause without the external watchdog and independent host evidence.
  Writer failure or truncation invalidates these absence inferences. The
  backend must capture outgoing identity and raw facts before any in-call
  child handoff; the host writer cannot promise that its post-return line
  precedes that handoff. Mark calls crossing either window boundary. Emit a
  late summary for a call entered before +100 s but returned inside/after
  the window, with `events_uncollected`, rather than silently implying no
  work. A pre-window call that never returns may leave no trace stage in
  this bounded capture; do not infer that the loop was idle there.
- A scoped diagnostic observer accompanies the host call through synchronous
  `run_pending_process_launches`. Its nested child reports an
  `attempt_end` before that internal `Result<()>` wrapper can discard the
  report. It carries the parent call ordinal plus a distinct attempt ID and
  nesting marker; the parent `call_end` still reports only its own outgoing
  attempt and whole-wrapper result. Native stages from both actors can
  interleave, so neither PID nor line adjacency is a join key. If an internal
  path cannot transport the report, mark its interval
  `nested_attempts_uncollected` and reject any complete attempt-count or
  guest-work total for that interval. The observer is diagnostic-only and
  has no mutable Kernel, Task or Frame authority.
- The per-attempt line retains `result` separately from `report`: `NoAttempt`
  and its preflight operation have no fabricated native facts; a failed run
  still emits its report before the existing error path returns. Include
  attempt ID, start/end process birth/PID/TID (end birth currently copies the
  captured lease birth), start PC, raw native stop PC, final outcome,
  failure operation, partial-effects flag, native result, count-stop and
  timeout observations including `Uncollected` and `Error`, requested-PC
  match, hook reason, and configured limits. Label the frame
  `LegacyUnbound`. Measure primary `emu_start` duration immediately around
  that native call; record it separately from the host wrapper's whole-call
  duration. `native_end` can precede a failed query or tail; it is not an
  independent success result. Neither duration is GuestInstant.
- If guest work volume is needed, carry the existing per-slice basic-block
  code-byte tally as a separately labelled `block_bytes/4` estimate and
  relate it to the same attempt ID. `COUNT_STOP = true` means the native count
  hook requested a stop, not that exactly the configured instruction count
  retired, not an exclusive stop cause, and not a CE quantum. Preserve raw
  competing stop/query facts even when `final_outcome` has one value.
- Add a cheap top-of-live-loop heartbeat, due at most once per second while
  that loop is turning, during the same host window: loop iterations since the
  last heartbeat, native attempts since the last heartbeat,
  selected PID/TID and saved PC labelled **selected, not executed**. Many
  current main-loop branches `continue` before the CPU call. An empty attempt
  stream without this heartbeat cannot distinguish scheduler-only churn from
  a stalled writer or long native call. It cannot fire while one wrapper/native
  call blocks; use the flushed `call_enter` and external observer for that
  interval. The heartbeat does not sample large debug snapshots or alter the
  branch decisions.

Gate 1 can separate observed host-only loop churn, repeated short native
attempts and a long entered call within the capture window. It cannot by
itself say why the splash stayed visible, prove map readiness or resolve a
wait that registered before the window.

## Gate 2: optional overlay-event correlation

For overlay correlation, use only existing import-trap and WNDPROC-return
paths: a bounded optional per-attempt collector in the backend, armed by the
host window before the call, receives the allocated attempt ID immediately
before the primary `emu_start`. Record a monotonic **within-attempt** event
ordinal and host offset, actual executing TID, event phase (import entry,
observed shim-dispatch result, observed WNDPROC return), HWND/message or import
ordinal, arguments, result, and observed window-owner PID/TID and post-call
visibility/parent/rectangle/z-order where available without new guest work.
Include `ShowWindow`, `DestroyWindow`, `SetWindowPos`,
`InvalidateRect`/`ValidateRect`, `BeginPaint`/`EndPaint`, **and**
`SetWindowLongW(GWL_STYLE)`, `SetParent`, `MoveWindow`, `BringWindowToTop`, and
`DeferWindowPos`: the latter APIs can change visibility, geometry or ordering
without a `ShowWindow` call. A shim-dispatch result is not necessarily a
completed guest-visible import return when a callback follows. Scope callback
returns to display lifecycle/paint/position/destroy messages. Capture all
matching HWNDs within
the bounded window, then identify E29's `0x00020008` overlay and `0x00020004`
main window in analysis; those numerical handles are run-specific diagnostic
targets, never behavior keys. The current `is_window_import` diagnostic ring
does not include every API listed above, so Gate 2 needs its own explicit
ordinal filter; silence in that ring is not evidence of no call. Do not
install a new global code hook. The existing GWE `show_window` lifecycle ring
records **changes**, not every guest request, so its silence cannot prove
that a no-op hide/show call was absent. Host-side GWE or callback preparation
outside a native attempt is unbound to that attempt: record it separately
with a host timestamp or leave it explicitly
unjoined. A prepared callout is not proof that its first guest instruction
ran; the existing return-stub path proves a return only. Repeated
HWND/message tuples alone do not prove an entry/return pair; use an actual
continuation identity before making that stronger claim. Gate 2 is a later,
independently reviewable addition to Gate 1, not a prerequisite for publishing
the first attempt/heartbeat evidence.

Both gates share a bounded write path: at most 32 MiB JSONL **including**
header, truncation marker and final counters, with 4 KiB of footer space
reserved before the cap. Gate 2 additionally permits at most 128 compact
events per attempt, with per-type saturating dropped counts and at most 1 KiB
encoded per event (truncate diagnostic text explicitly). Flush `call_enter`
and `native_enter` before their respective calls, then at least once per
second while the loop is turning and at normal exit. These are proposed caps
to verify under focused controls, not evidence of adequate capacity in a real
app run. Overflow may not abort or change the guest. A write failure disables
the diagnostic sink, reports one host-side error, and invalidates trace
acceptance; it must not convert a successful CPU result into an emulator
failure. No absence claim is valid across truncated, dropped, missing, or
uncollected intervals. No guest or mounted file is modified by this
diagnostic, so disabling the opt-in path is its execution rollback; retained
evidence remains in `evidence/`.

## Source boundary and expected-red controls

Gate 1's contemplated edit surface is `src/main.rs::run_cpu_loop`,
`monitor_run_once`, and `DesktopRuntime::run_cpu_until_report` (currently the
live and monitor callers use the Result wrapper); `src/emulator/types.rs`
for compact diagnostic metric transport if necessary; and
`src/emulator/unicorn.rs::run_with_unicorn_attempt` around attempt ID/native
entry/return and its `run_pending_process_launches` child call, which must
preserve the nested detached report. Gate 2 alone adds the existing import
trap/return (`~8813`,
`~9796`) and WNDPROC return stub (`~8245`) observation paths. No
`src/ce/scheduler.rs`, task policy, remote HTTP endpoint, guest API shim,
vendored Unicorn C, or per-instruction hook change is proposed. The
`src/ce/kernel.rs` message/window rings and `src/remote_server.rs` debug
pages remain contextual evidence, not exact joins. A causal test of E29's
blocked wait is a **separate** boot-forward, bounded wait-lifecycle capture:
raw wait-any/all at registration, object incarnation/generation, signal,
eligibility/selection, acquire and removal in ordered records. The post-120 s
attempt trace cannot reconstruct earlier registration from a later snapshot;
until that separate control exists, the wait remains unresolved.

Stage these controls as **expected-red before each gate's implementation**,
then require them green on that gate's implemented source; none is claimed
executed by this note:

Gate 1:

1. `post120_trace_window_uses_host_elapsed_not_guest_ticks`: at 99,999,
   100,000, 179,999 and 180,000 ms, only entered-at-window attempts get
   detailed capture; a boundary-crossing call is labelled, not omitted. The
   run-CPU, observer input-ready and monitor host origins remain distinct.
2. `post120_trace_no_attempt_and_error_keep_raw_provenance`: preflight has no
   ID/native facts; real-MIPS native/query/backend failures preserve their
   raw observations and error precedence, with writer publication before
   the ordinary error return.
3. `post120_trace_child_handoff_keeps_outgoing_owner`: a real-MIPS parent
   attempt that launches a child emits the parent's birth/PID/TID/PC,
   not the selected child's; the nested child receives a different attempt ID
   and its own final record with the parent call ordinal. Failure before that
   report is available marks the interval incomplete, never child-free.
4. `post120_trace_native_stages_locate_unreturned_call`: flushed `call_enter`
   without `native_enter`, `native_enter` without `native_end`, and
   `native_end` without `call_end` are distinguished without changing
   execution. A completed call reports native and whole-call elapsed
   separately. Exercise write failure at each stage and reserve the footer
   within the 32-MiB cap.
5. `post120_trace_heartbeat_exposes_no_attempt_churn`: a turning loop with
   zero native attempts emits bounded heartbeats; a blocked native call does
   not falsely emit them. Capture overhead and truncation are reported.

Gate 2:

6. `post120_trace_overlay_events_join_only_actual_attempt`: a real-MIPS
   fixture's window import and WNDPROC continuation share the attempt ID
   only when executed inside that native call; a host-side preparation or
   retained prior return must be unjoined. An interrupted callback must not
   be reported as a completed return.
7. `post120_trace_overlay_caps_are_observational`: exercise the
   128-event/1-KiB-event limits, per-type drop/truncation counters and shared
   32-MiB cap; no memory growth without bound, scheduler change, or
   transformed CPU result. Include an API outside `is_window_import` and a
   callback with no observed return.

## Acceptance recipe and interpretation limits

After each implemented gate's independent review, run format/diff checks,
focused real-MIPS controls, standard release, Unicorn-only and
Unicorn+desktop/no-trace checks, and no-feature tests on the exact D: source.
Record source/executable/trace hashes and raw commands. After Gate 1, perform
one separately authorized, bounded 330 s live run
with the configured `192.168.0.39:8765` remote endpoint, current adjacent
ZIP/hash-preservation preflight for writable SD/ResidentFlash/emulator-root
mounts, the same raw-NMEA and tap timing, host memory/watchdog sampling,
frame hashes and timestamped endpoint captures. If Gate 2 is subsequently
implemented, its overlay claims require a separate, likewise authorized
source-matched run. An approved writable-mount run is a new experiment, not a
replay of E29's exact input payload.

Analyze attempted versus no-attempt seconds first, including nested child
attempts and their completeness markers. Group attempt records by
outgoing birth/PID/TID, start/raw-stop PC and outcome, with total/median/max
host call/native durations and independent count/timeout facts. After Gate 2,
join overlay imports and WNDPROC events **only** on the same run and attempt
ID; compare host-timestamped remote frame/window state as a looser
observation. Record trace drops, host overhead, hook mode and guest
block-byte/MIPS accounting
before claiming a performance change. A repeated PC, positive count query,
event ACK, or splash frame alone is not a root cause or map readiness.
E29's blocked wait 833/TID 10 also needs its actual wait-any/all mode,
object incarnation/generation and signal/acquire ordering recorded from its
registration in a separate targeted control before claiming a missed wake.
This diagnostic does not implement that wait control or close O-P1, Stage 2,
S3-A or map acceptance.
