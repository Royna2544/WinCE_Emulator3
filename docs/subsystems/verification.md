# Verification, migration gates and evidence

Status: design-first verification plan. [Architecture](../ARCHITECTURE.md) owns every shared identity/time/outcome/snapshot definition; [Plan](../PLAN.md) owns implementation approval and ordering. Tests enforce those contracts at genuine consumers; they must not introduce a duplicate test scheduler, guessed application state or passing cosmetic screen.

## Current evidence and boundaries

Current suite/harnesses cover Kernel/API/GWES/memory/native behavior; source
and Cargo output stay in the sole D: checkout, while retained run evidence
stays outside disposable `target/`. Root freezes source hashes, validates
release/features and owns app/input/termination. [types.rs](../../src/emulator/types.rs),
[unicorn.rs](../../src/emulator/unicorn.rs) and the run-loop/interactive-monitor
paths in [main.rs](../../src/main.rs) expose different retained native/debug/loop
contexts. Formatting/source review, test compilation, executed focused/full tests
and real workload are separate acceptance levels.

Executed evidence is recorded once in the [ledger](../evidence/ledger.md); this
page owns only how to read it:

- BED2 (E-1) is the preserved clean 330 s fallback and still decorative
  splash; it is not the current source's release executable. A passing suite
  (the 650 build passed 1,618 tests) and a clean elapsed exit do not prove map
  readiness; the 650 failure (E-2) is positive evidence to preserve, and
  memory-growth correlation does not establish its cause. Later E-29 completed
  a bounded splash-only run, while E-32's E-31-source run host-crashed before
  its tap; neither validates map readiness. The current E-33 native-patched
  source has not completed a live-app run.
- Isolated native controls (E-4 to E-6) establish bounded Stage 8 fixture behavior only.
  Their elapsed time includes native timeout service and observation overhead, so
  it is not application throughput or cache speed, and they did not run the
  production mapping reconciler, Task/FPU transfer, backend stop/error matrix or
  the app.
- Full-backend controls (E-7): the mapping/context control passed; the
  count-stop and native-error controls were an expected-red baseline, then
  turned green in the D: checkout without fabricating native expiry. These
  narrow results did not alone complete O-P1 or Stage 2; E-34 later closed
  classification, not Stage 2.
- Task/handle ownership (E-8/E-11) is a red baseline: 13 controls, 1 pass,
  12 failures.
- The cross-owner same-VA/FPU green control (E-10) uses the experimental
  persistent-owner path; it cannot accept the Stage 2 fresh-engine VM gate.
- E-13's repaired-PC FPU continuation and E-16's owner-qualified injected
  same-VA native-view controls are narrow greens. E-18 was an expected red for
  stale private guest-written bytes after fresh-owner closure; E-25 made it
  green only through a test-only slab opt-in. Public same-VA
  `MapViewOfFile`, public `VirtualAlloc`, and production fresh-owner lifetime
  remain open.
- E-19's global deferred-stop slice types the first hook-observed remote or
  host-budget reason. On that frozen source, three serial focused runs passed;
  the full release suite passed 1,634 tests (0 failed, 4 ignored), both no-trace
  release checks exited 0, and no-feature passed 1,239 tests (0 failed, 1
  ignored). The E-18 red was reconfirmed separately on the same source; it is
  not a suite pass. At that E-19 checkpoint, sparse/hookless reasons, native
  count witness, typed restart-PC repair and Unicorn-control failure injection
  remained open; E-28/E-34 later closed the classification part only.
- Hot-path throughput (E-9) is measured per slice from guest instructions and busy
  MIPS, not from time to a milestone.
- Unexecuted staged fragments are not applied, compiled or passing until
  separately recorded.
- Verify the image RSDS GUID+age and the PDB actually loaded before interpreting a
  crash address; an adjacent generic-name PDB mislabeled the first 650 analysis.

## Gate ladder

| Gate | Required genuine consumer evidence | Explicitly insufficient |
| --- | --- | --- |
| Ownership/lifetime | Canonical handle/task/frame/address-space resolution, independent checkpoint state, termination/alias/wait reference cleanup | New structs or caller-supplied PID labels with old writable authorities still live |
| Fresh-engine CPU boundary (Stage 2) | Checked full native capture/restore including FPU, exact stop/error handling, both branch outcomes and VM private/shared-view coherence | Scalar-PC normalization, hidden state guessed from RA, or green controls run only on persistent owners |
| Persistent-engine swap (Stage 8) | Process engine move/clone, checked hook lifetime/cache/arena boundaries and repeated real workload | Translated-cache reuse or lower memory inferred from handle reuse alone |
| Run classification/service | Actual native result/expiry and real stop markers; quota/yield/block/deadline/debug/error distinguished; progress without fake wall stop | Positive configured timeout or cached HTTP snapshot treated as actual expiry/count completion |
| Task scheduling/time | Real create/suspend/resume/exit, nested frames, priorities/quantum/wait ordering and one guest-time authority | Visibility priority, forced completion event, host sleep substituted for CE timing |
| API/data/render delivery | Prepare-before-publication, exact-ID consumption, persistent guest bytes and executed rendering/input handlers | HTTP ACK, queue empty, writable native bytes without persistent mirror/lifetime |
| Bounded workload stability | Exact frozen binary completes requested horizon with no guest/backend/host failure, safe cleanup, memory accountability | Unit pass, early splash, partial/crashed run or post-exit RSS dip |
| Application map bring-up | Genuine splash dismissal/window transition, navigation map pixels, real GPS effect and ordinary touch response | Decorative splash difference, one finite cache lookup or clean 330 s exit |

## Test design and failure fidelity

Exercise production APIs/ownership rather than parallel models. Native fixtures use real guest instructions, true import/IAT/JALR calls, legitimate created thread handles, actual wait IDs, real in-emu hooks and checked outgoing/incoming context. Verify actual native RAM and persisted address-space bytes independently. Preserve original failing controls; never seed a synthetic successful snapshot or restore test context solely to bypass the bug.

Stage 2 fresh-engine matrices include repeated same-process execution,
same-VA private isolation and shared-view visibility, exact full native
context/FPU across slices, fresh-entry reset, count 1..16 at taken/not-taken/
interior/page-delay boundaries against an uninterrupted reference, nonzero
unmapped access under positive timeout, actual native deadline versus count/
voluntary block, exact diagnostic stop and verified encoded-exit negative
controls. Stage 4 adds genuine blocked wait→resume→worker exit. Stage 8 adds
persistent-owner clone/parked-process isolation and checked hook removal on
every return. Unknown query/provenance must stay unknown. Native host crashes
outside the Result path need separate failure artifacts, not fabricated guest
RunOutcome success.

Kernel/GWES matrices cover canonical aliases and creation owners beyond diagnostic-ring lifetime; suspend while blocked with pending completion; exit/handle/wait target lifetime; nested same-task outer wait; typed payload/output readonly/cross-page failure before consumption; queue replacement/expiry exact ID; temporary allocation/scratch release; selected GDI object/backing lifetime and independent checkpoints. A new policy cannot silently delete existing continuation frames or starve a legitimate helper.

## Reproducible workload protocol

Freeze source/config/data/executable identities and preserve fallback artifacts.
For native crash symbols, verify the image RSDS GUID+age and PDB actually
loaded before interpreting an address; an adjacent generic-name PDB gave the
wrong 650 subpage label. Use the same normal 330 s count 1M / native timeout 1000 ms
configuration, SD/registry/device/mount data and marker-file guard. No
undocumented cache reset or data mutation. Start a single GET-only Boot2
observer before launch and ACK actual config/time; use a fixed horizon long
enough for the real prelaunch gap and terminal run. Any overlapping tail must
be separate, timed and included in overhead. Match memory bracket request
order/count/cadence across comparisons.

Record app start/end, PID/hash, requested/actual duration, exit/watchdog, input server-ready/feed/tap/end clocks and per-request observer clocks. GPS+93/tap+153 are relative to actual input readiness unless explicitly changed; do not call them app-relative. Retain terminal PPM and fault traces only when actually produced; result filenames alone are not artifacts. Record forced observer termination separately from a graceful completion. No new app/control/input action follows from a read-only review request.

Report primary native-start owner separately from helpers and end-of-slice selected owners. Include workload counts, all-slice weighted native-run rates, continuity busy-filter rates, setup/hooks/snapshot/sync time, actual handle reuse/exact-context rates, native sync bytes and complete error/expiry observations. Count estimates/threshold filters cannot prove quota completion or a performance gain. Compare matched intervals and input phases; repeat matched baseline/candidate sequences before causal speed/memory claims.

Track whole-host RSS/private/virtual/CPU peaks and phase samples alongside logical blob length/capacity, allocation live/reserved extents, native regions/context/translation residency and retained queue/snapshot storage. Missing capacity/native-engine metrics remain missing. One active cached PID bracket is not whole-machine memory, an atomic epoch or proof of ownership. The 650 growth recorded in ledger E-2 requires investigation before a memory acceptance claim.

## Readiness and observability oracle

Inspect real frames/windows and genuine executed handlers. First splash is a bounded visual observation, not complete boot. A navigation map must arise from guest startup/paint paths and real splash teardown, then ordinary GPS/touch must affect navigation behavior. Keep app-specific gate addresses/messages only as diagnostic probes, never production heuristics.

HTTP acceptance, serial injection, positive guest ReadFile, parsed location effect, posted touch, guest dispatch/return and visible navigation response are separate milestones. The 650 run (ledger E-2) had 16 location ACKs but its only retained trace showed zero-byte reads, so later consumption was unobserved, and no tap succeeded. Cached debug/fault/window/frame fields need the future shared SnapshotEpoch contract; today report retained-snapshot/age/truncation limits rather than infer whole-run absence.

## Approval and handoff

Before code: each subsystem states current versus target ownership, complete consumer inventory, exact failure cleanup, compatibility removal criteria, dependency and executable acceptance matrix. Root reviews the shared anchor for consistency, then approves one coherent boundary. Staged fragments are not applied/compiled/passing until separately evidenced. After code: repeat source hashes, focused/full/native/feature checks and bounded workload gates; keep rollback available. Full Task/virtual-time and splash-to-map completion remain open until their distinct gates pass.
