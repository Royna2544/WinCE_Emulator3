# E28 Stage 2 run-outcome contract: one sealed native attempt

Status: **partially connected design, not an acceptance claim** (2026-10-14).
The additive native count-stop query and typed `CountLimit` adapter now have
[focused](../../evidence/e28-count-witness-audit-20261014/adapter-slice.receipt.md)
and [broad validation](../../evidence/e28-count-witness-validation-20261014/broad-validation.receipt.md);
the [native C seam](../../evidence/e28-count-witness-audit-20261014/native-c-seam.receipt.md)
is listed in [vendor PATCHES.md](../../vendor/unicorn-engine-sys/PATCHES.md).
The pre-adapter source description below is retained as the design baseline.
An immutable owner/frame-qualified final report, fallible
[finish-before-handoff](../../evidence/e28-run-outcome-20261014/finish-order-test-plan.receipt.md),
nested/reversal/query-error controls and diagnostic-private query-error
provenance remain open; O-P1, Stage 2 and scheduler migration are not claimed.
This narrows [Architecture A3](../ARCHITECTURE.md#a3-execution-and-continuation-boundary),
[PLAN Stages 2-3](../PLAN.md#migration-sequence-and-acceptance-gates) and
[O-P1](../DECISIONS.md) into the outcome prerequisite for the
[E26 S3-A identity slice](e26-scheduler-core-design.md#migration-gates-and-acceptance).
It does not select a task, change the one-million-instruction tick, or make a
host timeout into a CE deadline or quantum.

## Pre-adapter source baseline and the missing authority

`CpuBackend::run_until_import_trap_with_framebuffer_limits` still returns
`Result<()>` ([cpu.rs](../../src/emulator/cpu.rs)); there is no task-facing
`RunOutcome`. `NativeSliceOutcome` in [types.rs](../../src/emulator/types.rs)
is the last attempt's mutable summary. It distinguishes `NoAttempt`,
`UnknownNormalStop`, four typed hook reasons, native deadline, requested PC,
guest process exit and native/backend error. At this baseline it had no proven `CountLimit`,
registered block, voluntary yield, guest-deadline service or structural
boundary variant. `UnknownNormalStop` remains eligible for the *existing*
service-boundary rotation; this is not evidence that a CE quantum expired.

In [unicorn.rs](../../src/emulator/unicorn.rs), `run_with_unicorn` captures
the starting native PID, resets the old outcome, and records `emu_start`'s
result, actual `Query::TIMEOUT`, checked context capture, requested-PC and
hook evidence before deferred launches. This is valuable raw provenance.
`NativeStopEvidence::outcome` then projects those facts to one enum value,
but later encoded-exit, context, mapping, guest-byte sync, PC repair and
native hook-teardown paths can still revise it or fail. A separate getter
lets callers observe the mutable last value, without an immutable attempt
identity, final owner/frame or final-tail success boundary. Some global and
sparse hook reasons are typed already; this note does not reset those gains.
Other `emu_stop` paths and failed PC-write/stop combinations remain to audit.

There is a concrete publication-order trap in the ordinary success tail:
outgoing file views are checked, then `rotate_after_run_slice_handoff` may
switch the active process, and only *after that* does fallible `uc.finish()`
remove hooks ([unicorn.rs](../../src/emulator/unicorn.rs)). A sealed success
cannot be attached to the now-selected `self` after that rotation. The first
connected report slice must check `finish` before rotation, bind the report
to the outgoing owner, and publish it before any handoff; this is an ordering
repair, not a new scheduling policy. The explicit-PC and diagnostic-private
tails already check outgoing sync and `finish` before returning success, but
their error paths still need the same one-attempt report discipline.

For the ordinary tail, the minimal order is: read back the captured outgoing
owner's file views while `uc` is live; check `uc.finish()` and classify any
error on that still-active owner; only then rotate and prune parked processes.
Use a per-engine, one-shot test-only finish failure after a real counted MIPS
run to verify the parent remains selected, receives the backend-error report,
and leaves its runnable child parked. The successful parent/child view-readback
test is the opposite control. This order is necessary but not sufficient for a
fully sealed outcome: `persist_run_state` already drains shared run handles
before the ordinary finish, and `saved_context` is assigned before later
fallible work. Moving persistence after finish without explicit failure
salvage could lose pending/parked state; leaving these pre-finish mutations
requires a separate continuation/error-state review. See the
[finish-order audit](../../evidence/e28-run-outcome-20261014/finish-order-test-plan.receipt.md).

At this baseline, the vendored Unicorn core's count hook had an internal
`emu_counter` and `emu_count` ([uc.c](../../vendor/unicorn-engine-sys/uc.c)),
but public `uc_query` exposed timeout, not a count-stop flag
([unicorn.h](../../vendor/unicorn-engine-sys/include/unicorn/unicorn.h)).
`Ok` plus configured `count > 0`, a matching PC, or a sampled instruction
counter is therefore **not** an independent count-exhaustion witness. The
current `UnknownNormalStop` classification is honest for that residual.

## Proposed contract and publication boundary

The CPU/backend owns native execution and raw stop evidence; the machine owns
the transition and publication. A sealed attempt report, returned or exposed
through one checked API rather than a mutable `last_*` getter, carries:

- A non-reused attempt identifier; configured instruction and host limits;
  captured starting process-birth/TID and native lease identity; and actual
  ending process/task/frame observations. Until Stage 3 supplies a canonical
  `FrameIdentity`, a legacy frame observation is explicitly **unbound** and
  cannot authorize a task-frame transition. Never derive creation owner from
  the Kernel's later selected PID or a debug trace. The current birth lookup
  is mandatory only for diagnostic private slabs; capture it as an optional
  fact for every native attempt without imposing a new legacy preflight
  rejection. The lease's test-only handle address is not a non-reused identity:
  use an epoch/attempt sequence rather than a recyclable pointer alone.
- The raw native result, actual timeout query (including query failure),
  independent count-stop observation, requested-PC match, first typed hook
  request and any competing requests, bounded native fault evidence, and
  checked context/guest-byte/lease-tail results. Configured timeout and an
  uncollected query remain distinct from actual expiry and false.
- One final anchored `RunOutcome`: guest exit, native/backend failure,
  registered block, voluntary yield, proven count limit, actual native host
  budget, registered guest deadline, exact debugger stop, structural
  boundary, or explicit unknown. Ordinary CE API errors stay guest-visible
  return/status/LastError values, not fatal run outcomes. A typed error
  contains the failing operation and whether any guest/native effect may
  already have happened; it does not promise rollback of live writes.

This is one attempted run, not a snapshot of whichever CPU happens to be
selected after a child launch or process rotation. If a run is rejected before
`emu_start`, report `NoAttempt` with its preflight error, not a stale prior
success. Raw evidence is captured immediately after the primary native call,
before deferred work changes owners. The final outcome is published **once**,
only after checked context capture, necessary PC repair, outgoing mappings
and guest-byte reconciliation, hook cleanup/lease finish, and final owner/frame
validation succeed. Debug snapshots and scheduler decisions consume that
same sealed record. No scheduler-visible success is published while any tail
operation is fallible.

| Boundary | Allowed state | Failure behavior |
| --- | --- | --- |
| Prepare/enter | Validate selected owner, context and mapping lease; mark attempt in progress | No native run; leave old report unconsumed, return a distinct preflight failure |
| Native return | Capture result, `Query::TIMEOUT`, count witness, hook/PC/fault facts against the starting owner | Preserve native error/query failure; do not infer deadline or count from configuration |
| Fallible tail | Validate full context, repair only authorized continuation PCs, reconcile bytes/views, finish hooks/lease | Classify backend/native error; retain exact effect evidence; poison an uncertain continuation or mapping lease; no rotation or task/wait transition |
| Seal/commit | Bind actual ending owner/frame and publish one immutable result | Machine may commit only transitions justified by that result and its current canonical keys |

An upstream-wrapper `Drop` does not report `uc_close` status, and the current
`abort_close` checks hook removal but conservatively quarantines pointer
backing when close certainty is unavailable
([native_engine.rs](../../src/emulator/native_engine.rs)). E28 cannot call
that a checked-close proof or clear O-P4. If teardown is uncertain, keep its
pins/quarantine and prevent retry from a guessed continuation; a later exact
owner/frame or checked-close migration must resolve it. Legacy blob/view sync
may have partial effects before a later failure; retain them as evidence, not
as an invented transaction rollback.

## Independent count witness and classification rules

The smallest native change to review is a per-`emu_start` count-stop bit in
the vendored core: reset it at attempt start, set it only at the internal
count hook's successful stop request, and expose it through a checked query
to the Rust adapter. The exact C seam is
[`include/unicorn/unicorn.h`](../../vendor/unicorn-engine-sys/include/unicorn/unicorn.h)
(append a query enum value), [`include/uc_priv.h`](../../vendor/unicorn-engine-sys/include/uc_priv.h)
(per-run field), and [`uc.c`](../../vendor/unicorn-engine-sys/uc.c)
(count hook, start reset, and query case). The vendored [`build.rs`](../../vendor/unicorn-engine-sys/build.rs)
already generates Rust `Query` from the header; the pinned 2.1.5 Rust wrapper
already accepts any `Query` in `Unicorn::query`. Neither the generated binding
file nor the registry wrapper needs a hand edit. Do not read private
`uc_engine` fields from Rust or infer the cause from `emu_counter >= count`.
Read the bit on the same native
owner immediately after `emu_start`, alongside `Query::TIMEOUT`, before
another run can reset it. A query error is an error, never `false`.

The bit must reset on **every** `uc_emu_start` entry, before `UC_INIT`, nesting
validation, hook registration or any other early return; an unsuccessful new
attempt must not expose an older count stop. The current counter already resets
at entry. Same-handle nested starts also reset that counter and can corrupt a
single-bit attempt record, so the first adapter must forbid them or use a
stacked native record. In this core, `hook_count_cb` increments at the code
callback and requests stop only when `emu_counter > emu_count`: a limit of one
is signalled at callback two, before that candidate instruction executes.
The counter is not an independently verified retired-instruction count.

The count hook is inserted first among `UC_HOOK_CODE` hooks. At the threshold,
its stop can suppress a user code hook on that same instruction; a block hook
or exact `until` PC can instead stop before the next count callback. A timeout
thread can race with count stop, leaving both facts true. Preserve these
observations rather than choosing a quantum from the configured count. The
trace helper also has a `UC_HOOK_FLAG_NO_STOP` path that can reverse a stop
request; no current call site passes that flag, but a flag advertised as a
completed count stop must be cleared on such reversal (or be named and used
only as request evidence). The invalid-instruction `size == 0` path can call
the counter too; a primary native error still wins classification.

`CountLimit` requires that witness, a successful primary native result and
no competing explicit PC/hook/deadline/error cause. If independent facts
conflict, preserve them in the report and classify the cause as unknown or
error pending a source-backed precedence rule; do not silently choose count.
Likewise, actual native timeout is a host-budget fact, not guest deadline or
CE quantum expiration. A hook reason is published only after checked safe-PC
write and `emu_stop`; failed control is backend error. Exact debugger stop
requires the final saved PC and frame to match the requested stop. A
source-validated encoded CE exit may override a native error only when no
hook-control, mapping, capture, sync or teardown error competes; unexplained
PC zero remains hard failure. Post-run PC repair consumes typed provenance
and must not turn an unknown/error stop into a safe continuation.

## Red controls, first connected implementation slice and rollback

Before production code, stage ignored expected-red controls on this source
for the new count query and sealed-report boundary. Then implement only the
count-witness adapter and immutable per-attempt report, keeping current
`is_service_boundary()`/rotation policy unchanged. The report may temporarily
record legacy/unbound frame identity; no S3-A production caller consumes it
until the canonical frame shell and outcome review are ready. Do not add a
second task registry or claim that this slice classifies every `emu_stop`.
An ignored test still has to compile: before the header generates a new Rust
`Query` variant, a control can run real MIPS through the existing API and
temporarily compare the successful outcome's debug label with `CountLimit`
(alongside `TIMEOUT == 0`, no requested PC, and no typed hook). Replace that
string assertion with a typed one when the variant exists. Stage a direct
typed native-query control only after the header/bindgen seam compiles; do
not transmute a not-yet-valid Rust enum discriminant.

Focused controls must include real MIPS execution, not just enum-unit tests:

1. A finite count loop stops with the independent witness, `TIMEOUT = 0`,
   exact outgoing context and `CountLimit`; count zero disables the witness.
   Exercise a one-instruction limit and the count-hook's next-instruction
   boundary so the reported count is not off by one.
2. Explicit PC stop, early typed hook stop, actual host timeout and unmapped
   native error each **do not** become `CountLimit` merely because count was
   configured. Competing stop observations stay visible; an unknown successful
   stop remains unknown, not a quantum.
3. Inject query, context-capture, safe-PC write/`emu_stop`, PC-repair,
   outgoing view/byte sync and hook-finish failures. Each suppresses a prior
   provisional success; no stale report, scheduler handoff, wait consumption
   or new Running task is published. A decoded-exit plus injected hook failure
   remains an error. Record any partial guest effects and pin quarantine.
4. Force a deferred child launch/process selection change and assert that
   raw and final report owners still name the actual primary attempt. Assert
   a preflight-only rotation produces `NoAttempt`, not another CPU's last
   result. The no-Unicorn backend must expose an explicit no-attempt/unsupported
   result under the same interface, not fabricated native evidence.

Preserve raw logs, source hashes and the expected-red then corrected outcomes.
After focused tests, run full release features, no-feature and no-trace checks,
then review remaining untyped stops and bounded live evidence separately.
The E26 twelve historical task reds remain red until their connected Stage 3
slice; passing outcome tests does not make them green. If the count query or
report introduces mismatched owner, ambiguous cause, stale publication or
new rotation behavior, revert that connected adapter/report change while
retaining red fixtures and receipts. Do not repair the failure by relabeling
unknown as count/deadline, suppressing native errors or enabling the
persistent-owner policy. S3-A's production identity/lifecycle migration waits
for independent review of this outcome boundary and its own additional
birth, alias, wait-pin, exhaustion and clone controls.
