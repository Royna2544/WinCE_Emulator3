# E-34 O-P1 mixed native-deadline/JR continuation fixture design

Status: proposed test gate; no new runtime behavior or acceptance is implied.
This follows the E-31 ignored pure-predicate red and remains in Stage 2's CPU
outcome boundary. The [common anchor](../ARCHITECTURE.md) owns the outcome and
identity contracts; the [plan](../PLAN.md) keeps Task/scheduler policy changes
behind the Stage 2 gate. The source under test is the sole D: checkout.

## Owner, consumer, and actual state

The `UnicornMips` CPU backend owns the native engine, its saved continuation,
and the per-attempt stop facts. Its block hook may request a checked safe-PC
yield to a visible parked peer; Unicorn may independently report expiry of the
real native deadline. `NativeAttemptReport` transports both facts for the
outgoing PID/birth/TID even if post-slice selection rotates to the peer. It is
diagnostic, not Task authority. The host live/monitor loops currently discard
the report through their `Result<()>` wrapper, so this fixture does not change
their scheduling decisions or the one-million-instruction tick.

The two JR repair sites are distinct: the first checks a registered import
target when the classified outcome is `NativeDeadline` and no historical
debug wall-stop marker exists; the second may rewrite any aligned target in
the import-trap page while preparing the debug/saved continuation. A typed
`VisibleParkedPeerYield` can coexist with an actual native deadline in the
same attempt. The typed reason must protect its checked JR restart PC even if
the aggregate outcome retains `NativeDeadline` precedence.

## Test construction and evidence gate

1. Build the existing tiny real-MIPS PE shape with `lui/ori t9; jr t9; nop`.
   Register one valid `GetTickCount` import trap and refresh its import-trap
   page blob. Execute the first two guest
   instructions under an explicit PC stop at the JR, then verify `t9`, the
   saved/native PC, and absence of import dispatch. This setup stop is a
   separate attempt and cannot be mistaken for the mixed-stop observation.
2. Create a real child process identity and visible child window using the
   existing parked-peer fixture. Keep the parent selected for one live-pump
   attempt, using the existing slice-end rotation gate. A per-CPU, `cfg(test)`
   one-shot probe may force the existing block-hook service at this exact JR
   address; it must not fabricate a stop reason, native query, saved context,
   or guest instruction. After the hook's checked PC write and `emu_stop`
   succeed and it latches `VisibleParkedPeerYield`, the probe may pause long
   enough for the real one-second Unicorn deadline to expire. The probe is
   consumed by this attempt and cannot affect another CPU or production build.
3. Require the returned outgoing report to show native success,
   `COUNT_STOP=false`, `TIMEOUT=true`, typed `VisibleParkedPeerYield`, raw stop
   PC equal to the JR, and no requested PC stop. Require the parent, not the
   selected child, as the report's native owner. Assert saved, native, and
   debug-snapshot parent PCs still equal the JR, and assert the snapshot has
   no synthetic `host_wall_clock_stop` marker; no import dispatch occurred
   before resumption. Since the handoff selects the child, explicitly rotate
   back to the parked parent, neutralize the peer-yield condition and prevent
   an immediate preflight re-rotation before resuming the real trap exactly
   once. Do not require `final_outcome=HookStop`: mixed evidence
   currently gives the actual deadline aggregate precedence.
4. Repeat the stop with an aligned *unregistered* import-page target. The
   first registered-target repair cannot rewrite its PC, so a changed PC
   isolates the second repair. The first site may still create its historical
   marker for this unregistered target; assert that separately. This variant
   must not resume into a fabricated import.

The fixture is expected red at the protected-PC assertion on current source;
keep it ignored in the normal suites until a connected repair turns it green.
The test must first establish all mixed native facts. If Unicorn does not
return both typed yield and `TIMEOUT=true`, the fixture is inconclusive and
must not be reported as a JR-policy red. Run each native case in a fresh
subprocess if one case's native state or crash could contaminate another.

## Failure, cancellation, and acceptance boundaries

All native setup, PC writes, stop requests, timeout/count queries, context
capture and checked lease finish retain their existing error behavior. A
failed test probe must not turn a native error into a successful continuation;
it must fail the fixture. A future production repair may consume typed stop
provenance in both JR sites, but only after this real-MIPS red, source review,
and a no-regression control for true deadline-only JR completion. The first
green is not O-P1 closure: report sealing, nested-count ambiguity, native
query-error competition, broader feature suites, and the real-app gate remain
separate. No extra writable-mount app run is authorized by this design.
