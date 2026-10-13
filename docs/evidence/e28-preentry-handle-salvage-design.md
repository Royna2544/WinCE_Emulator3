# E-28 pre-entry run-state salvage design (2026-10-14)

## Boundary and invariant

`run_with_unicorn_attempt` transfers CPU scheduler/callback queues into shared
hook handles before installing all hooks and before the native `emu_start`.
A setup error after a transfer must return a `NoAttempt` report with no
diagnostic attempt ID or native observations, while restoring every moved CPU
record. A registered Kernel wait must not outlive its CPU-side wait record
solely because setup failed. A diagnostic private CPU stays poisoned; no
continuation or scheduler rotation may be published. This restoration is not
rollback of earlier Kernel or guest-memory side effects.

There are two transfer phases in the current function. Parked children and
pending WNDPROC returns move before the first hook-install failure. The
remaining queues, including blocked wait threads, move by the construction
of `UnicornRunStateHandles` at the second phase. The current early hook
installations and later hook installations, private-page fence, and attempt-ID
allocation can all fail before `emu_start`.

## Red control

`private_pre_entry_fence_failure_preserves_executed_thread_wait` holds a real
stable-private-page execution guard to force the checked pre-entry fence to
reject the run after the CPU queue transfer. It asserts `NoAttempt`, no ID,
uncollected native result, and the same seeded `Sleep` wait in Kernel and CPU.
The fixture does not inject a C error or execute MIPS. Freeze its expected-red
output and source hash before a production repair.

A second control is needed because an active thread's `Sleep(INFINITE)` is not
eligible for the scheduler's ready-wait selection. Seed a *different* real
same-process worker thread with a wait on an already-signaled event, keep the
main thread selected, and force the same private fence failure. Before the
fence, the current setup resumes that worker into transient Unicorn registers,
consumes the Kernel/CPU wait and changes the selected thread. Abort-closing
the lease then discards the only resumed register state. A `NoAttempt` error
must leave that worker wait and original current-thread selection intact.
This control is distinct from simply restoring a non-ready queue element.

## Narrow repair

Use checked setup-error exits in both transfer phases. Before full run-state
construction, return parked children and pending WNDPROC records from their
shared handles. After construction, call `persist_run_state_handles` once on
each pre-entry error. Abort-close the native lease without publishing it as a
reusable engine; release any active private-page guards only after the native
lease has closed. Do not call `persist_run_state`, which also runs orphan
maintenance/pruning, and do not run successful-stop stale-wait cleanup.

`orphaned_wndproc_returns` is transferred separately from the run-state
bundle and must also be restored on pre-entry error. More importantly, defer
the ready blocked-wait/GetMessage/modal/send/clipboard resume chain until
*after* all fallible hook installations, private-page fencing and diagnostic
attempt-ID allocation. The chain currently has no `Result` return; placing it
immediately before `emu_start` leaves no later setup `?` that could discard
its register continuation. Keep the captured start PC/TID after this resume.

Keep the pre-entry report as `NoAttempt` even when private setup poisons the
CPU. An error after an actual `emu_start` belongs to the separate entered-
attempt finalization path and must retain its attempt ID and raw observations.
Verify the focused red becomes green, rerun the entered-attempt wait controls,
the host-idle live-wait control, and then broader feature/no-feature gates.

The macros/checked exits are source-shape dependent: any future new fallible
exit after a queue transfer must use the corresponding restoration path.
This is a bounded repair, not a canonical Task/Frame scheduler rewrite or
proof that all Kernel-side preflight mutations are reversible.
