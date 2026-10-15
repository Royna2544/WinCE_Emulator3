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
| A-3 | Is remote GPS positively consumed by the guest and applied to navigation state? | E-22 and E-27 retain positive guest `COM7:` `ReadFile` previews matching injected `$GPRMC` bytes after 16 raw-NMEA ACKs; E-27 retained three positive reads/696 B. The later source-matched E-32 run received 16 raw-NMEA ACKs and retained 11 positive reads/2,784 B; E-38 retained nine positive reads/2,262 B after 16 ACKs. These are bounded rings, not lifetime totals; none proves position parsing or navigation use. The failed 650 run's retained sample had only zero-byte reads. | Correlate the known payload with guest reads, then seek separate position/map-state evidence; never equate ACK or positive read with a GPS fix. |
| A-4 | Why does the splash not hand over to the map after the progress bar is full? | E-22's final JPEG hash repeated through +329.53 s. In E-27 the final splash JPEG hash repeated in 151 successful polls from +28.99 through +328.97 s; both runs exited normally at their 330 s cap without a map transition. E-32 showed the splash through +130.994 s but crashed at +135.282 s; its 121 post-120 native slices disprove total idleness, not the splash cause. The separately approved E-38 fresh-default run also reached its 330 s cap with the splash still displayed. Map-readiness time is therefore unmeasured beyond this observation window, not a successful boot-time result. | Compare frame epochs, scheduler progress and message continuations on matched runs; no hypothesis becomes a code change in this lane. |
| A-5 | Where does tap delivery lag arise, and does it lead to navigation? | E-22/E-27 accepted a +153 s remote tap and retained down/up message delivery with WNDPROC returns, but no touch-driven map transition. E-27 first captured delivery at +159.00 s, a poll-time upper bound rather than exact handler latency. E-38 also received one tap ACK and retained down/up WNDPROC returns without a map transition. | Correlate ingress, drain, WNDPROC entry/return and frame change per event on matched runs. |
| A-6 | Does the repeated child `WM_ACTIVATE` return indicate a scheduler continuation defect? | E-22 retained 2,527 and E-27 retained 2,518 orphaned `DispatchMessageW` return-stub warnings for child TID 3 while the parent still processed messages. The archived `last_return` is not identity-matched to each encounter. A later [test-only real-MIPS discriminator](../evidence/e27-orphan-discriminator-20261014/receipt.md) passed three focused controls and a no-feature check: after one injected pending loss it observed two fresh entries at the same stub without a second guest callback entry. That synthetic signature does not classify the live warnings or explain the persistent splash. | Trace a live pending return by process birth, thread, callback identity and native slice through the actual continuation; distinguish stale-stub replay from fresh callback entries before a causal fix. |
| A-7 | Is host-memory growth compatible with the CE6 footprint target? | E-22 sampled peaks were 1,543.9 MiB working set and 1,554.4 MiB private; E-27 sampled 1,489.2/1,503.5 MiB. E-32 peaked at 1,655,271,424 B RSS and 1,652,563,968 B private before its early crash. The later E-38 fresh-default 330 s run peaked at 430.676 MiB RSS and 444.953 MiB private. These are unmatched single runs with differing source, logging and possibly phase/input, so neither the drop nor a small-memory CE6 target is causally established. | Compare phase-aligned committed guest bytes, native/engine counts, working set and private bytes on matched runs after the planned backing/lifetime migration. |

The [E-32 run receipt](../evidence/e22-live-20261014T002535Z/analysis.receipt.md)
retains exact source, input, frame, memory and crash boundaries behind these
newer observations; its post-120 slice count is not proof of map progress.
The [E-38 receipt](../evidence/e37-live-20261014T051228Z/analysis.receipt.md)
is the latest approved live observation; its one-run approval is spent. New
writable-mount observations require a separate human approval and fresh
adjacent, SHA-verified ZIP backups.

### Next live-run preparation (design gate; not authorized or executed)

Instrumentation and runner work below require their own source gate; this
read-only app-evidence lane does not authorize a launch or gate a stage.

**Narrow memory telemetry.** In `src/main.rs::publish_remote_endpoint`, plan a
`live_guest_stable_page_budget_used_bytes` field in the debug `summary`, read
from `kernel.memory.private_commit_used_bytes()`. The debug-publisher owner
must assign a per-run monotonic publication sequence and host-monotonic
nanoseconds at its accepted publication slot, with UTC for human alignment,
*in that same body*. A repeated sequence is stale even if the HTTP receipt
time differs; UTC alone can tie or jump. The observer should fetch the summary
every 2 s, retaining each response's receive UTC/monotonic clock, HTTP result,
body SHA-256, embedded publication identity and parsed field;
retain at most 256 summary bodies of at most 64 KiB each, independently of the
raw-log limit. Reject an oversized summary while retaining the HTTP failure
metadata; do not truncate it into an apparently valid sample. The current
generic capture path accepts much larger responses, so this needs a dedicated
summary bound and a real 2-s fetch rather than its current every-fifth-slot
(10-s) cadence. A source control must show reserve-only leaves the field
unchanged and committing one new stable page through the *public*
`VirtualAlloc` path adds exactly 4 KiB; also check that shared-view aliases
charge once, retained pins keep their charge until final release, and O-M2
heap/stack page commits charge per guest page rather than per 1-MiB native
mapping block. This is the guest **stable-page budget**: `used_bytes` includes
charged shared-view pages, detached/native-retained pins and quarantined
charges,
but excludes `boot_reserved_bytes` (a separately tracked budget component)
and other guest backings. It is not total guest commit, guest physical RAM or
host RSS. Existing heap-live,
virtual-live and `blob_mb` values remain partial proxies; report any missing
quantity rather than infer a small CE6 footprint.

**Fresh-run manifest and preflight.** Do not reuse
`scripts/inavi_bounded_e22.ps1` unchanged: it hardcodes the E-37 EXE/PDB,
2026-10-14 adjacent ZIP hashes, a spent approval and `e37-live-` paths. A
future parameterized runner must require a unique evidence tag and per-run
manifest naming and SHA-256-verifying the fresh release EXE and matching PDB
against a build receipt chained from the W-B1c sealed HEAD `0e06e2c5` and its
patch/file hashes to the new telemetry/runner source seal. Immediately before
launch, the recorded source hashes must still match that new seal. The
EXE↔PDB RSDS GUID+age must match, not merely their separate hashes; any crash
analysis must verify the actually loaded PDB. The new source must pass its own
focused controls and sealed full release/no-feature/alternate-feature gate
before that EXE can be used. The manifest also records current
`registry.reg`, `serial_devices.json`, `mounts.toml` and target-app hashes,
the exact fresh backup manifest and all three adjacent
SD/ResidentFlash/EmulatorRoot ZIP paths and hashes, plus the
human one-run approval reference. Validate these relationships and hashes
before launch, including that `mounts.toml`'s three writable roots are exactly
the three backup source roots. Reject E-37 paths, missing fields or an
existing evidence tag. The monitor worker must independently revalidate the
sealed manifest hash and executable provenance immediately before its actual
`Start-Process`, not rely only on the parent process's earlier check.
A manifest records approval but cannot grant it. Keep a durable append-only
approval/backup use ledger under `evidence/`, outside disposable build output.
Mirror every entry (approval reference, ZIP hashes, state and UTC) to a tracked
append-only ledger under `docs/evidence/` for off-machine backup commits; the
runner must refuse launch if either copy is missing or their histories differ.
If a crash interrupts the two-file update, that disagreement fails closed
until a human-recorded reconciliation, never an automatic timeout.
The monitor worker that actually launches the process must, under an
exclusive ledger lock, reject any approval reference or ZIP hash already
reserved/spent and append and flush an attempt reservation immediately before
`Start-Process` in both ledgers; a parent-only check is insufficient. Record the exact spawned
PID/start identity and mark the
approval spent once spawn succeeds. An unresolved reservation fails closed and
cannot be reused automatically; even a proved no-spawn attempt needs fresh
human direction before retry. Clearing a stuck reservation requires an
explicit human action recorded as another ledger entry, never an automatic
timeout. Test the ledger's duplicate and interrupted
reservation behavior without launching the emulator.
After a new human approval, `scripts/backup_rw_mounts.ps1` must produce and
verify the fresh ZIPs. Its source inventory and sequential archive are not an
atomic snapshot if a mount changes mid-backup: require the three mount trees
to stay quiescent through inventory/verification, compare a post-backup
content-hash inventory to the source manifests, and fail closed on any detected
mismatch.
Just before spawn, compare a cheap size/mtime inventory against that
post-backup inventory and reject another detected emulator process using the
mounts; if process identity cannot be checked, fail closed. These checks
narrow but cannot close the later-write window or prove the absence of every
external writer.
Preserve the four slow-boot marker guard without deleting markers, the
no-reparse checks, pre/post mount inventories, 330 s app cap, independent
390 s watchdog, 2 GiB RSS cap and 512 MiB raw-log cap.

The expected-red controls are an absent/repeated-sequence summary publication
and reserve-only versus public one-page accounting, plus stale E-37/approval/ZIP,
already-spent approval, interrupted ledger reservation, bad-hash and
duplicate-tag manifests that must reject before process spawn.
Review the metric semantics, manifest schema and failure boundaries
independently **before implementation**; acceptance then needs those focused
source/parser controls green and a source-bound build/test receipt. Tests of
the parser and negative manifests must not launch the app or touch mounts.
This design gate itself neither backs up mounts nor spends a run approval.

For a separately approved live run, keep the comparable raw-NMEA feed and one
input-ready-relative tap. Record HTTP ACK, remote drain/post, actual guest
WNDPROC entry/return and subsequent frame hashes as different milestones.
E-38 tap delivery predates W-B1b/W-B1c, so establish that new routing still
reaches the intended iNavi window; a poll interval only bounds lag unless
events carry correlated monotonic timestamps. Capture launch-relative and
input-ready-relative splash, progress and *actual map* frames separately,
along with phase-aligned PID1/helper guest instructions, busy-slice MIPS and
fixed setup cost, host RSS/private/virtual peaks, stable-page budget and
separately reported VM reservation. Separate native cache/engine residency
from guest RAM and host RSS. One clean run can test delivery and stability;
causal speed/footprint claims still require matched repeated runs under
[verification](subsystems/verification.md).

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
