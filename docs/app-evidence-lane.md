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
| A-3 | Is remote GPS positively consumed by the guest and applied to navigation state? | The bounded E-22 run (ledger E-22) proves positive guest `COM7:` `ReadFile` transfers containing the injected `$GPRMC` bytes, after 16 remote ACKs. It does not prove position parsing or navigation use. The failed 650 run's retained sample had only zero-byte reads. | Correlate the known payload with guest reads, then seek separate position/map-state evidence; never equate ACK or positive read with a GPS fix. |
| A-4 | Why does the splash not hand over to the map after the progress bar is full? | In E-22, genuine splash appeared by +3.52 s; the final frame appeared by +27.45 s and its hash repeated through +329.53 s. The app exited normally at its 330 s cap, still on splash. The earlier "map gate" emulator defect was fixed, but this stall remains unexplained. | Compare frame epochs, scheduler progress and message continuations on matched runs; no hypothesis becomes a code change in this lane. |
| A-5 | Where does tap delivery lag arise, and does it lead to navigation? | E-22's +153 s remote tap reached the main WNDPROC by about +159.4 s; no map or touch-driven transition followed. Earlier 1.3-1.6 s delivery lag is not a stable estimate for this run. | Correlate ingress, drain, WNDPROC entry/return and frame change per event on matched runs. |
| A-6 | Does the repeated child `WM_ACTIVATE` return indicate a scheduler continuation defect? | E-22 retained 2,527 orphaned `DispatchMessageW` return warnings for child PID 67/TID 3 while the parent still processed messages. This is a concrete defect signal, not proof of the splash cause. | Use a bounded, real-MIPS two-process park/resume fixture to check one pending return and the actual `DispatchMessageW` continuation before another live causal claim. |
| A-7 | Is host-memory growth compatible with the CE6 footprint target? | E-22 sampled peak working set was 1,543.9 MiB and private bytes 1,554.4 MiB; bounded survival is not a small-memory result. The source retained its native owner by default. | Compare phase-aligned committed guest bytes, native/engine counts, working set and private bytes on matched runs after the planned backing/lifetime migration. |

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
