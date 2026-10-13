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
| A-3 | Is remote GPS positively consumed by the guest and applied to navigation state? | 650 run: 16 ACKs but only zero-byte reads in the retained trace. June 2026 runs drained NMEA into `COM7:` and returned bytes through guest `ReadFile`; for current builds only acceptance and drain are established. | Count positive `ReadFile` returns and bytes per slice; separate ACK, drain, copied bytes and parsed use. |
| A-4 | Why does the splash not hand over to the map after the progress bar is full? | First splash by about 4.84 s; full map readiness is open; the earlier "map gate" was an emulator defect (fixed). | Frame-hash timeline plus window/message lists from retained snapshots; no hypothesis becomes a code change in this lane. |
| A-5 | Where does the 1.3-1.6 s tap delivery lag arise? | HTTP ACK, queue, guest dispatch and return are separate stages; only some are observed. | Per-event correlation through ingress, drain, WNDPROC entry and return using existing traces. |

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
