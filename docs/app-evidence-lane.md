# App evidence lane

Status: parallel lane ([D-005](DECISIONS.md)). It is read-only observation of the
real iNavi workload. It never gates a stage, never changes runtime behavior and
never adds an app-specific shortcut. Its output is evidence: new
[ledger](evidence/ledger.md) entries and confirmed facts in `PROGRESS.md`, so that
Stage 7 bring-up starts from measured behavior rather than inference.

## Rules

- Follow `RULES.md`: no forced callbacks, no guessed readiness flags, no behavior
  keyed on application names or strings. A probe may name an address; a
  production path may not.
- Use only existing instruments: `RUST_LOG` targets (`ce.slice_profile`,
  `ce.quantum`, `ce.gwe`, `ce.imports`, `ce.devices`, `ce.file`), `--tracefile
  counts|milestones`, the remote debug endpoints and the dumped binaries
  (`llvm-objdump`). Debug endpoint text can be minutes stale; record each
  capture's clock, body and frame hash and trust executed slices over cached
  snapshots (see [remote observability](subsystems/remote-observability.md)).
- Follow the workload protocol in [verification](subsystems/verification.md):
  frozen binary, SD marker-file guard, explicit clocks, at least three matched
  runs before any causal statement. Single runs of scheduling-sensitive metrics
  are weak evidence.

## Questions

| ID | Question | What is known | Method |
| --- | --- | --- | --- |
| A-1 | What does iNavi wait on in the 1 s idle slices that begin at an import trap with only 2-6k guest instructions? | 10-40% of 1 s slices in 2026-10-12 runs (ledger E-9). The growing 5/10/15/20 s sleeps were attributed to sensor-init retries and fixed; these 1 s slices were not re-attributed. | Sample the active import and return address during a gap, disassemble the caller in `iNavi.exe`, correlate with `ce.slice_profile`. |
| A-2 | What does the helper-scheduling bimodality depend on (`happyway_win` 170-300 slices versus 1-7 for the same build)? | Observed with and without Phase 0 hook changes; one run after the `CreateProcessW` replay fix still gave the helper a single slice. | At least five matched runs per configuration; record slice splits and the first 30 s of event ordering. Propose no scheduling heuristic from this lane. |
| A-3 | Is remote GPS positively consumed by the guest and applied to navigation state? | E-22 and the second bounded run (E-27) each retain positive guest `COM7:` `ReadFile` previews matching injected `$GPRMC` bytes after 16 raw-NMEA ACKs. E-27 retains three positive reads totaling 696 B; neither run proves position parsing or navigation use. The failed 650 run's retained sample had only zero-byte reads. | Correlate the known payload with guest reads, then seek separate position/map-state evidence; never equate ACK or positive read with a GPS fix. |
| A-4 | Why does the splash not hand over to the map after the progress bar is full? | E-22's final JPEG hash repeated through +329.53 s. In E-27 the final splash JPEG hash repeated in 151 successful polls from +28.99 through +328.97 s; both runs exited normally at their 330 s cap without an observed map transition. The earlier "map gate" emulator defect was fixed, but this stall remains unexplained. | Compare frame epochs, scheduler progress and message continuations on matched runs; no hypothesis becomes a code change in this lane. |
| A-5 | Where does tap delivery lag arise, and does it lead to navigation? | Both runs accepted a +153 s remote tap and retained down/up message delivery with WNDPROC returns, but no touch-driven map transition. E-27 first captured the delivery at +159.00 s, a poll-time upper bound rather than exact handler latency. | Correlate ingress, drain, WNDPROC entry/return and frame change per event on matched runs. |
| A-6 | Does the repeated child `WM_ACTIVATE` return indicate a scheduler continuation defect? | E-22 retained 2,527 and E-27 retained 2,518 orphaned `DispatchMessageW` return-stub warnings for child TID 3 while the parent still processed messages. The archived `last_return` is not identity-matched to each encounter. A later [test-only real-MIPS discriminator](../evidence/e27-orphan-discriminator-20261014/receipt.md) passed three focused controls and a no-feature check: after one injected pending loss it observed two fresh entries at the same stub without a second guest callback entry. That synthetic signature does not classify the live warnings or explain the persistent splash. | Trace a live pending return by process birth, thread, callback identity and native slice through the actual continuation; distinguish stale-stub replay from fresh callback entries before a causal fix. |
| A-7 | Is host-memory growth compatible with the CE6 footprint target? | E-22 sampled peaks were 1,543.9 MiB working set and 1,554.4 MiB private. E-27 sampled 1,489.2/1,503.5 MiB; unmatched single runs cannot establish improvement, and bounded survival is not a small-memory result. | Compare phase-aligned committed guest bytes, native/engine counts, working set and private bytes on matched runs after the planned backing/lifetime migration. |

Closed by other sessions (recorded in `PROGRESS.md`): growing sensor-init retries
(packed-sample fix), the repeated `CreateProcessW(happyway_win.exe)` storm under
count 0 plus a 100 ms native timeout (live-pump replay fix), whole-file read
retention (bounded 64 KiB windows). Re-verify rather than assume if a regression
is suspected.

## Closing a question

A question closes when a ledger entry records the evidence and either a fix is
routed through the plan's stages or the behavior is documented as expected. A
finding that suggests a design change goes to [DECISIONS.md](DECISIONS.md) as an
open question; it does not change code from this lane.
