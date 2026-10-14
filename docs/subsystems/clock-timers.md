# Clock and timers

Status: proposed clock migration plan; current timing behavior is not replaced by this document. Shared GuestInstant, HostInstant, CeMachine, RunOutcome and SnapshotEpoch contracts are defined only by the [architecture anchor](../ARCHITECTURE.md). The [migration plan](../PLAN.md) gates implementation; [tasks/scheduler](tasks-scheduler.md) and [kernel objects](kernel-objects.md) own runnable and wait lifetimes.

## Responsibility and separation

CeMachine owns one guest monotonic time authority. All guest waits, sleeps, timers, message timestamps and quantum accounting must consume that authority through explicit conversions. HostInstant is used for host elapsed budgets, transport arrival observation, diagnostics and watchdogs; it is not silently added to guest time.

A native configured timeout, an actual host native expiry and a CE guest deadline/quantum are different facts. RunOutcome conveys why execution actually returned; no timer or scheduler consumer may infer actual expiry from a configured positive timeout. SnapshotEpoch identifies observation provenance, not elapsed guest progress.

## Current implementation and gaps

[TimerSystem](../../src/ce/timer.rs) keeps a host boot Instant plus virtual_elapsed_ms. GetTickCount truncates their sum to u32; performance counter uses the sum with a shim frequency 1000 Hz. sleep_ms adds a virtual offset while host elapsed time continues, so the clock is hybrid and not a coherent virtual-time scheduler.

KernelTimer stores numeric TID/HWND/id, u32 due_ms, period and pending_message. Timer keys scope equal IDs by owner/HWND. Current due selection compares due_ms<=now and delay uses saturating subtraction; both differ from bounded wrapping wait arithmetic. Rearm uses now+period with one pending-message flag. These are current source behaviors, not affirmed CE6 timer-overrun/wrap semantics.

[Scheduler](../../src/ce/scheduler.rs) and native blocked records use start/timeout scalars. TaskWaitStore's compatibility TickDeadline preserves the existing unsigned wrapping predicate; it does not silently implement finite CE wait rounding. The raw Sleep converter already distinguishes 0 yield, INFINITE suspension and finite+1ms shape, but raw/high-level/native Sleep paths do not all share one advancement boundary.

The native loop and [main](../../src/main.rs) mix host limits, instruction count servicing, timer polling, idle throttle and deferred work. Current ctl_get_timeout usage is a configured-duration observation, not the actual Query(TIMEOUT) flag. Honest RunOutcome migration is a prerequisite; a one-line query replacement can otherwise remove existing cleanup/rotation without defining its replacement.

### Bounded HWND-less thread-timer birth gate (design before implementation)

This Stage 3 ownership slice consumes the existing exact `ThreadBirthKey` contract (anchor A2) without changing guest-clock policy (A5). `TimerSystem` remains the sole timer store. Guest `SetTimer(NULL,...)` and `KillTimer(NULL,...)` must carry the captured caller birth through raw dispatch into the Kernel mutation and revalidate that exact live birth before timer, trace, queue, status or wake mutation; a fresh numeric-TID lookup is not authorization for a captured stale A to rearm, overwrite or cancel B. Native import preflight checks that captured register birth for only the `HWND == 0` forms before raw dispatch. The existing HWND timer path and explicit host/numeric test compatibility stay separate, never an implicit fallback for guest thread timers.

A thread-timer key includes the exact birth and timer ID; each set/rearm has a checked timer-instance generation. Due selection, pending-flag transition and publication must be one validated operation: revalidate the birth before trace or publication, post `WM_TIMER` to that birth's GWE FIFO with its unchanged ID/callback `lParam`, and wake only eligible waits of that birth. Exact exit retires only A's timers and already-queued A timer messages; B's timer and window timers survive. A stale due candidate is removed rather than left pending forever, without publishing success or work to a replacement TID.

The queued timer message needs host-only origin metadata containing birth and timer instance that survives dequeue into Kernel's pending-clear path, without changing guest MSG bytes. Removing Get/Peek clears `pending_message` only for that exact still-live timer instance; non-removing Peek does not clear it. A synthetic `PostThreadMessage(WM_TIMER)` or an older same-ID timer message cannot clear B's coalescing flag. Checked generation exhaustion or failed publication must fail closed without trace or pending-flag corruption. Preserve requested/auto IDs, callback behavior, current one-pending-message suppression, window teardown semantics and existing timer ordering. No timer minimum, wrap, overrun, pacing or CE LastError parity change is authorized by this slice; those questions remain O-C4 and Stage 6 work.

The expected-red control creates A with a HWND-less timer, exits A, reuses its TID for B and proves that neither a pre-queued nor later-due A timer yields B `QS_TIMER`, `WM_TIMER`, trace, timestamp or wake. B's same-ID timer must still deliver. Additional controls reject late A Set/Kill without altering B, preserve callback and non-removing Peek/coalescing, and keep numeric HWND timers intact. Seal source before counted release/no-feature suites; a source pass does not establish live iNavi or full checkpoint relineage. Copying `TimerSystem` and GWE births independently is not machine-wide checkpoint relineage and remains deferred. Primary evidence is the current `TimerSystem`/Kernel/GWE/native route cited above; CE6 SetTimer delivery details not established by local evidence remain explicitly open.

## Target clock interfaces

Proposed responsibilities, not final Rust signatures:

- Read current guest monotonic instant and project guest API units/wrapping tick views.
- Convert a validated API duration into finite/infinite guest deadlines, with API-specific CE rounding/clamping.
- Register/cancel timer and wait deadlines by canonical owner/frame identity; obtain next established deadline.
- Advance guest time only at a reviewed machine boundary, account execution/quantum under the chosen conversion model, then expire eligible deadlines.
- Accept an external event with observed HostInstant and an explicit guest-time assignment under the selected mode. Preserve actual arrival ordering and any recorded timestamp; do not invent future packets or readiness.
- Emit timer messages/callback eligibility through the real queue/continuation boundary, retain pending-message lifetime and clear it through consumption/teardown.
- Publish guest/host timestamps plus SnapshotEpoch separately. UTC logs are observation coordinates, not GuestInstant.

Use checked/range-bounded conversions. Infinite waits are not huge finite deadlines. u32 GetTickCount wrapping is an API projection; internal guest ordering must remain coherent. Calendar/system-time APIs require a separate reviewed wall-clock epoch mapping, not an accidental monotonic/host-clock cast. Performance-counter frequency/units also require an explicit contract.

## Idle advancement and quantum constraints

Fast-forward is allowed only after canonical Task state proves no eligible task is Ready/Running and the next **established** guest deadline is known. A parked process, a retained wait, no visible window or an empty cached queue is not that proof.

Run mode is decided ([D-002](../DECISIONS.md)): live-first. The default
real-time-coupled mode respects the selected host-to-guest pacing and the external
event bridge; unknown future GPS/input arrivals never authorize jumping past them,
and if no deadline or event is established it waits for external availability under
a host budget rather than fabricating guest progress. An opt-in fast-idle mode (for
bring-up and tests only) may advance to the earliest established deadline when no
task is Ready/Running and no external source is connected. Deterministic replay,
advancing to the earliest timestamped known event/deadline, is a later layer over
the same deadline and ordering semantics.

A count quota can remain an explicit service budget during migration; it is not automatically a CE quantum. OEM-configured quantum, remaining quantum on preemption, equal-priority rotation and quantum 0 must be wired with Task state before replacing count with time-based scheduling. Changing the clock while legacy state/continuations remain split is not an accepted shortcut.

## CE6 source evidence

Primary local source: `C:/WINCE600/PRIVATE/WINCEOS/COREOS/NK/KERNEL`, with `NK/INC/schedule.h`.

- NKSleep 227–251: Sleep(0) invokes ThreadYield; SleepINFINITE suspends; positive finite Sleep adds 1 ms unless at the long bound, and very long sleeps are chunked against MAX_TIMEOUT.
- DoWaitForObjects 563–570: finite nonzero/non-INFINITE timeout adds a millisecond and clamps to MAX_TIMEOUT; deadlines use OEM ticks. MAX_TIMEOUT is 0x7fff0000 (`NK/INC/schedule.h` 29), supporting bounded signed wrapping comparisons rather than unlimited u32 ordinary ordering.
- KCNextThread 1079 processes pending interrupt events before its timeout scan; completion/timeout increments wait generations. Do not assign a fixed universal signal-versus-timeout winner independent of the actual serialized boundary.
- ThreadYield and KCNextThread 865/1198–1298 define yield/quantum placement; InitThreadStruct `thread.c` 279 obtains OEM default quantum.

These kernel sources establish sleep/wait/scheduling contracts, not every GWES SetTimer delivery/minimum/overrun rule. Current UI timer semantics require a separate dumped/source-backed evidence check before parity changes.

## Actual consumers to migrate

- TimerSystem tick/performance/sleep and KernelTimer due/rearm/pending APIs.
- coredll GetTickCount, QueryPerformanceCounter/Frequency, Sleep/SleepTillTick, system-time conversion and SetTimer/KillTimer.
- CeKernel::pump_timers_to_gwe and timer-message consumption; window teardown, caret blink, shell notification expiry, sent-message timeout and message timestamps.
- Scheduler deadlines, native Kernel/Sleep/MsgWait/GetMessage/modal/send/clipboard/serial/socket timeout paths; device/I/O-specific durations remain API-specific.
- Main/monitor native stop handling, instruction service budgets, host limits and process rotation. Host watchdog limits remain HostInstant consumers.
- External remote input/GPS/device arrival observation and delivery timestamps; checkpoint/restore must preserve guest clock state without sharing mutable machine time.

A renamed timer wrapper alone is insufficient while another path directly advances virtual_elapsed_ms or compares raw u32 due values. Each retired compatibility path needs a test and a removal criterion.

## Migration gates and decisions still open

1. Honest RunOutcome, canonical task/frame ownership and pinned wait lifetimes. Preserve configured/actual native timeout observations distinctly; existing exact stop/error handling stays authoritative.
2. Introduce clock-domain interfaces with no advancement-policy change. Test every projection/conversion and retain unavailable observations explicitly.
3. Convert finite/infinite waits and timer deadlines as one reviewed family; verify cancellation/generation, wrap and real queue/pending-message effects.
4. Choose and test host-to-guest execution/pacing model, OEM quantum configuration and external-event bridge.
5. Enable idle advancement only after complete runnable-state proof; validate live-first behavior and the opt-in fast-idle mode independently (replay later).

Open decisions (instruction/cycle/native-elapsed conversion, checkpoint host-anchor rebasing, guest calendar epoch, timer minimum/overrun/coalescing, external-event timestamp assignment, host pacing bounds): [DECISIONS.md](../DECISIONS.md) O-C1 to O-C6. Do not choose a guessed CPU MHz or universally charge native wall time as guest execution.

Required controls:0/INFINITE/finite Sleep, finite wait clamp/rounding, wrap boundaries, equal deadlines with real signal ordering, explicit suspension during wait completion, timer owner/window teardown and pending-message suppression, periodic overrun, checkpoint time isolation, no-ready idle fast-forward, unknown external future arrivals and quantum 0/preemption/round robin. Existing timer unit tests exercise the current hybrid behavior; staged native stop/lifecycle/FPU fragments are unexecuted prerequisites, not clock acceptance.

No clock rewrite, app event invention, forced callback or performance claim is made here.
