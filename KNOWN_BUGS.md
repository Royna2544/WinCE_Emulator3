# Known Bugs And Risks

Refreshed on 2026-10-14. This file lists unresolved behavior only.

## Open Issues

- The first persistent-engine 650 live trial host-crashed at 129.1081087 s with
  `0xC0000005`, no watchdog/empty stderr; peak RSS/private were
  1,662.59 / 1,663.40 MiB. GUID-matched 650 symbols/disassembly place fault
  RVA `0x85e757` at native `tb_gen_code_mipsel` / TranslationBlock jump-reset
  initialization. Earlier `subpage_init` labeling used an adjacent BED2 PDB
  and was wrong.
  No surviving dump or final native Result/profile establishes root cause.
  Terminal PPM/full traces are absent. Failed650/PDB and evidence are preserved;
  BED2 is a hash-preserved measured fallback, not the current `target/release`
  EXE. Current source still builds experimental ownership; do not treat it as
  stable merely because historical suites passed 1,623/0/3 ignored before
  E-12, 1,626/0/3 after it, or frozen E-19 passed 1,634/0/4 ignored.
  Pinned native translation arenas retain touched Windows pages through hook
  invalidation/flush. Six isolated Stage 0 cases queried a 1 GiB initial arena;
  three hook-churn cases retained about 243 MiB sampled RSS until owner close,
  whereas stable/lease/map controls stayed near 7 MiB. Native residency and
  the 650 failure mechanism remain unattributed; bounded fixture success is
  not full-backend or application stability.
- Pre-init native arena requests of1MiB and2MiB both faulted0xC0000005
  while the initializing size query ran; no actual native size was returned.
  Both WER faults point to GUID-matched tcg_out8 at RVA0xA2D31. The pinned
  Windows translation-arena handler requests4MiB at a time and its end branch
  does not clamp to the reserved end. This source mechanism is consistent
  with these tiny-reservation failures, but the pristine failure's exact fault
  address/handler result and any connection to the650 app fault remain
  unproven. An8MiB tiny smoke
  passed; that did not validate later tail behavior or a production cap. A
  copied-source bounds clamp now passes fresh1/2/8MiB/default tiny controls;
  its8MiB distinct-PC probe observes second-half page commitment before any
  TB flush and a later successful translation. The matching pristine-handler
  copy still AVs on1/2MiB and before tail observation. This differential
  supports an isolated repair, not a production cap, full backend safety or
  650 causality. A later isolated 2x2 matrix passed24/24 fresh-owned runs
  and directly recorded a second fault at arena offset4,194,328 with the
  clipped request4,194,280B and a successful commit. The earlier in-VEH
  telemetry build stack-overflowed before its first1MiB query; smaller
  counters did not reproduce it, so its cause remains open. The existing
  x64 call-based trampoline also lacks unwind metadata; a compliant shadow
  stack frame alone does not establish exception/unwind safety. A separate
  leaf-tail-jump candidate passed seven isolated fresh-process controls,
  including the direct clipped tail, but not concurrent callback teardown,
  full-backend or app validation. Two synthetic final-partial-page runs at
  logical4MiB+1 also passed a one-byte clipped commit without observing
  commitment beyond the host-page-rounded reservation; TCG's guard means
  this is not a normal translation-block tail or production-budget proof.
  Sequential VEH owner close/recreate passed twice with handler removal,
  arena/closure release and a surviving owner's successful tail. The
  concurrent in-flight callback versus teardown race remains untested.
- Phase1 experimental migration is underway, not complete. Native per-process engines now
  persist through checked hook-free reuse; exact primary TID/scalar keys can
  restore full native CPU snapshots. Legacy in-slice per-API thread/callback
  continuations still restore scalar contexts; task identity/lifecycle and
  coherent virtual time are not authoritative yet. Global hook removal
  invalidates translated regions, so cache retention is unproven. Retaining
  multiple engines beside host blobs raised whole-host RSS in the failed trial;
  allocation attribution and stable/cache-preserving design remain unproven.
- The pre-repair persistent-owner E-12 map/unmap attempt failed on its first
  mapped phase; 64 cycles were not completed, so no churn acceptance follows.
  Its repaired-PC scalar-fallback FPU loss now has an expected-red/focused-green
  fresh-owner control (ledger E-13), but that result neither attributes the
  650 crash nor closes the fresh-engine Stage 2 VM/context gate. Legacy
  in-hook register helpers still turn some read/write errors into zero or
  ignored results, and callback/o32 floating-return behavior is unverified.
- Fresh-per-slice is not yet safe to select from the current dirty source:
  ordinary guest-written private `VirtualAlloc` bytes lack authoritative
  readback into host backing, and the separately run closed-owner fixture fails at the
  final assertion because the default still retains the owner. Its preceding
  Context/guest-store checks passed in that isolated fixture, but a production
  fresh-owner transition is not proved. Outgoing file-view readback now runs
  before PID rotation. Historical E-14 injected two distinct views at one VA
  under two PIDs and failed after both native MIPS slices: all-view seeding
  gave the parent the child's bytes, and base-only readback credited the
  child's store to the parent mapping. On the E-16 O-M1 source, captured-owner
  seeding and exact-view readback turned this into an unignored passing default
  regression; closed-live exact readback and ambiguous lazy fail-closed
  controls also passed. Exact red/green logs and frozen source hash are in
  local `evidence/stage2-o-m1-same-va-20261013/owner-qualified-native-view.receipt.md`.
  The later ignored E-18 internal `MemorySystem::virtual_alloc` fixture now
  demonstrates the private-byte loss: native RAM held `0x11223345` after a
  guest MIPS store, but a one-instruction load on a fresh owner read stale
  `0x11223344` from unchanged seed bytes. Expected PC, normal stop and no-
  fault checks passed. Test-only helpers simulated idle-owner closure;
  production owner policy is unchanged. Receipt:
  `evidence/stage2-o-m1-private-fresh-20261013/expected-red.receipt.md`.
  The later test-only source passed its default full release suite
  (1,632 passed, 0 failed, 4 ignored), both no-trace release checks and the
  no-feature suite (1,239 passed, 0 failed, 1 ignored); E-18 is the fourth
  ignored test, not a green private-byte result. Exact source/log hashes are
  in local `evidence/stage2-o-m1-private-fresh-20261013/post-e18-validation.receipt.md`.
  E-20 later added a public COREDLL `VirtualAlloc`/`CreateProcessW` two-PID
  same-VA control; it remains ignored and expected red because the child
  allocation returns null at the parent's VA. Its later free/query assertions
  were not reached. Public same-VA `MapViewOfFile`, per-owner lazy fault repair
  and live-app behavior remain untested; immediate CreateProcess error handling
  lacks fault injection. The static COREDLL table also lacks ROM-exported
  `VirtualAllocEx`/`VirtualQueryEx` ordinals, but the target EXE does not
  directly import those Ex functions (dynamic lookup remains possible).
  E-22 now records canonical process-birth keys, per-lineage allocation
  incarnations and explicit private/file-view/locale/Toolhelp/shared-compat
  source kinds. It does not change the global base-keyed allocation directory,
  native copy-seeded bytes, Ex target handling or clone lineage. The combined
  release suite passed 1,647/0/5 ignored and the no-feature suite passed
  1,248/0/2 ignored. Explicit E-18/E-20 reruns on that source still failed
  at the intended stale-byte and child-same-VA-allocation assertions
  (ledger E-22).
  This is not O-M1/O-P4 or Stage 2 acceptance.
- Canonical Task/alias ownership is still absent. In eight actual-Kernel
  copy-mirror controls, seven failed on copied alias priority/suspend/exit,
  inferred creation PID, lost boot/primary identity or alias incoherence in a
  cloned Kernel; only DuplicateHandle(CLOSE_SOURCE) state preservation passed.
  Exit through one alias did not expose the code through another, so the
  later alias wake-index assertion was not reached. These are reproducible
  ownership defects, not a proof that a production Task migration has landed.
  Three further private-copy controls failed for final-public-handle live
  task retention, sibling-alias wake indexing and registered-wait target
  retention after alias closure. Across the thirteen controls, one passes
  and twelve are expected red (ledger E-8/E-11); genuine native exit and
  wait acquisition remain unproved. See chat
  `validation/20261013-task-pin-expected-red-receipt.md`.

- The newly integrated 354AA0FB build passes1,571 tests but its normal GPS/tap
  run fails at260.174s with PC0, RA0xe6de4/SP0x7feff1c8 on pool worker10.
  Event trace records a legitimate signaled0xff4 ready wait (id4029/result0)
  before the failure; worker5 is suspended at0x321c98. The wait continuation,
  native branch/jump target and in-slice context switch need provenance checks.
  Profile startPC26a264/endTID10 alone is not ownership corruption: those fields
  refer to different points in the slice. Cause/regression attribution remains
  open; PC0 stays a hard failure. Artifacts: `target/bringup_1013_boundaries_drive_*`.
  Separate genuine native fixtures now prove unsafe helper-interior handoff
  can produce PC0 with a valid IAT, and original-PC replay after a completed
  guest delay duplicates its effect even when the recipient runs cleanly.
  The entry-only scheduling guard passes1,585 tests and a subsequent normal
  330.268s GPS/tap run (12DF), exit0/no reported CPU error/watchdog. Exact actual-app
  attribution remains unproven; this bounded success is not full map readiness.
  The later BED2 safe-yield build also passes1,593 tests and a clean330.234s
  run. Wall-clock restart and asynchronous native timeout persistence remain
  separate unproven scope; neither bounded success proves full map readiness.
- The historical E-7 actual-backend baseline used `ctl_get_timeout()` as if it
  reported actual expiry. A counted stop with `Query::TIMEOUT = 0` fabricated a
  host-wall record, and a nonzero `READ_UNMAPPED` fault returned Ok under a
  configured positive timeout. The current D: source queries actual expiry,
  records a typed native outcome and passed the post-E-12 full release suite
  (1,626 passed, 0 failed, 3 ignored) on the dirty persistent-owner checkout.
  Unicorn-only and Unicorn+desktop/no-trace release checks and the no-feature
  suite (1,239 passed, 0 failed, 1 ignored) also passed; a bounded app run
  remains pending. A later focused O-P1 slice changed the successful residual
  to `UnknownNormalStop` and proved `HookStop(HostWallBudget)` at one checked
  block-hook site (ledger E-15); six other debug host-wall writers were
  untyped on that E-15 source, and no independent native count-stop witness
  exists. Seven
  focused controls passed on the first O-P1 source; a follow-up guard kept
  hook-control failures from being masked by decoded guest exit and passed
  two host-budget controls. Real hook-control-failure/encoded-exit injection
  was not run. The final follow-up source passed the full release suite
  (1,628 passed, 0 failed, 4 ignored); the four ignored tests, including E-14,
  did not become passes. Both no-trace release checks exited 0, and the
  no-feature suite passed 1,239/0/1 ignored. The 1,626-pass suite above is
  historical. A later E-17 source added
  `HookStop(VisibleParkedPeerYield)` at one checked hookless block-hook site;
  its real-backend control passed three serial runs without a native timeout
  or host-wall marker. Frozen E-17 source passed 1,632 release tests (0 failed,
  3 ignored), both no-trace checks and 1,239 no-feature tests (0 failed,
  1 ignored); exact hashes are in local
  `evidence/op1-visible-peer-20261013/post-visible-peer-validation.receipt.md`.
  The peer was directly constructed, not created through public CreateProcess,
  and failing Unicorn control APIs were not injected. A later E-19 global
  code-hook slice types the first observed `RemoteControlService` or
  `HostWallBudget` deferred stop after checked safe-PC transfer. Its two
  real-backend focused controls passed in three serial runs (2/0 each);
  frozen-source full release passed 1,634/0/4 ignored, both no-trace checks
  exited 0, and no-feature passed 1,239/0/1 ignored. Exact hashes are in
  local `evidence/op1-global-code-hook-20261013/post-global-stop-validation.receipt.md`.
  The queued remote input is test-
  inserted, not a public network-delivery proof. Other sparse/hookless
  reasons, a native count-exhaustion witness and typed
  restart-PC repair remain open. Complete O-P1 classification, fresh-owner
  Stage 2 and live acceptance remain open; the 1M-instruction tick is intact.
- HeapFree does not reuse addresses or release per-process spillover pages.
  Initial/reallocation mapping, ZERO and metadata ordering are now validated,
  but global allocation metadata lacks authoritative private-process ownership,
  immutable reservation footprint and incarnation identity needed for safe reuse.
  No free list, cross-process page reuse or speculative unmapping has landed.
- Host footprint is too large for the intended lean CE environment: a
  300 s profile reached sampled RSS 1,187.1 MiB/private 1,195.9 MiB while
  the app stayed on splash. Eager read-only whole-file retention, writable
  buffer copies, persistent guest blobs and repeated engine/snapshot copies
  are distinct costs to measure; virtual reservation is not resident RAM.
  Bounded read-only windows are implemented and all 1,527 tests pass. The
  new 330 s run peaks at RSS 562.3 MiB/private 573.2 MiB, materially lower
  but not yet a fully lean/interactive CE bring-up. Do not assume ordinary process
  snapshots duplicate full kernel/file buffers: production park/rotation moves
  CPU/blob owners; full kernel clones belong to optional monitor checkpoints.
  The reallocation-prerequisite 330 s repeat peaks at 566.0/568.0 MiB, with
  splash still present; early lower memory samples do not establish a new gain.
  The integrated callback-context run peaks at 492.6/503.2 MiB; it remains too
  large and non-interactive, and has a different helper workload distribution.
- Gesture FIFO is fixed, but a real tap still took about 1.29 s from successful
  HTTP response to first guest wndproc delivery in the prior 330 s input test,
  and 1.62 s in the bounded-cache repeat.
  Guest DOWN/UP handlers each returned in about 23 ms; delivery lag and frame
  readiness, not merely HTTP acceptance, are the remaining response gates.
  BED2's latest one-tap run observes0.397s before DOWN (earlier0.835s), still not responsive
  navigation or a controlled latency improvement.
- Full application boot timing remains unresolved because no map/interactive
  navigation phase has presented. The latest clean 300 s run reaches device
  opens at 31-35 s and positive GPS ReadFile, but splash remains unchanged.
  BED2 first splash is observed by+4.358s; failed650 by+4.800s with per-request timestamps;
  full readiness remains unproven and must be reported separately.
- iNavi route-flow completion remains open beyond process startup, shell
  readiness, real splash rendering, remote input delivery, and initial mapdata
  loading. The former "map presentation gate" was the emulator swallowing all
  host-delivered visible messages via escaped-callout false positives (fixed
  2026-07-04, `2fcc365f`/`dbaf4791`); post-fix runs advance into
  `resmapi_800x480.bin` map-skin streaming with the splash progress bar at
  full width. The remaining frontier is the map screen actually presenting
  after skin decode. COM7/MFS1/SMB1 opens and sustained GPS reads are proven
  on the current build; keep validating the remaining presentation gate.
- `happyway_win.exe` busy-waits inside its WM_ACTIVATE handler
  (`happyway_win.exe+0x28f8..0x2a18`) polling for cross-process state; it
  coexists with iNavi only because quantum rotation is unconditional. If
  startup ever wedges with happyway active and frozen counters, check that
  rotation still reaches iNavi.
- Scheduling is timing-sensitive and bimodal across identical builds
  (2026-10-12): `happyway_win` either gets 170-300 slices (fair round robin) or
  1-7 slices (starved while iNavi runs ahead). Reproduced with and without the
  Phase 0 hook changes, so it is a property of the wall-clock scheduling
  heuristics, not of one build. Metrics that depend on it (milestone times,
  slice splits) need several runs; compare busy-slice MIPS instead.
- Count=0 remains experimental after fixing CreateProcess replay and block-start
  sampling. The fixed 120 s run still gave the helper only six slices while
  iNavi received 844. Native Unicorn timeout stops also bypass the voluntary
  block delay-slot guard; saved contexts contain PC/GPR/HI/LO, not MIPS pending
  branch state. Preservation across an asynchronous stop is not yet proven.
- The former growing 5-30 s waits were sensor initialization retries (Sleep RA
  `0x000e69c0`), corrected by valid packed samples. TimerSystem mixes boot wall time
  with a sleep-driven virtual offset; a coherent central clock and idle
  fast-forward are not implemented. Do not attribute current live waits to
  non-live host-sleep paths without matching execution evidence.
- Watch for re-created zero-byte probe artifacts on the SD image
  (`INavi\traffictest.txt`, `INavi\routetest.txt`, `mapdata\soap.bin`,
  `mapdata\searchdb\oildata.dat`). `happyway_win.exe` embeds
  `traffictest`/`routetest` strings and boots dramatically slower when the
  marker files exist, so a returning zero-byte file means an emulator path is
  still materializing files on read-only existence probes. The 2026-07-04
  no-marker run did not re-create them through the splash and mapdata phases.
  Originals are preserved in `D:\INAVI_Emulator\_zero_byte_probe_artifacts_backup`.
- Scheduler/message semantics are not complete for every reentrant wait,
  cancellation, nested send, abort-if-hung, modal-loop, and cross-thread
  result-delivery combination.
- Shell notification behavior still needs integrated validation for taskbar
  rendering/routing, guest COM callback lifecycle, stale-handle ownership, and
  remove/update races.
- Loader parity is strongest in the Unicorn runtime path. Raw/non-Unicorn
  loader behavior needs explicit audit before all load variants can be treated
  as equivalent.
- Mounted storage remains synthetic for several lower layers beyond covered
  public AFS mount registration: physical block-driver forwarding, external
  cache/filter DLLs, real mounted-volume availability/powerdown, broader ACL
  persistence/enforcement, utility DLL execution, callable FMD callbacks,
  hardware-backed FLS discovery, and lower-FSD `FsIoControl` forwarding.
- GDI/text/IME fidelity is still incomplete for rendered IME UI callbacks,
  caret timing, broader font/style fallback, glyph metrics outside fixture
  ranges, complex clipping, palette/device-color behavior, alpha/mask variants,
  and remaining DIB/bitmap lifetime edges.
- File-change notification reset/error propagation still has deeper edge cases
  beyond the currently covered prefix drains, no-more-items, undersized
  buffers, null-buffer ordering, all-zero no-data reset, and fault-drain cases.
- Popup/menu/modal behavior still needs broader live nested-modal routing,
  unusual cascade cancellation, timeout edges, and user-driven dispatch
  validation.
- Sensor emulation remains partial beyond the basic live paths. GPS/NMEA now
  reaches `COM7:` in the current live run, drains into guest serial RX, and is
  returned by real `ReadFile` calls; `MFS1:`/`SMB1:` also open and receive
  sensor IOCTLs. Parsed-position consumption, broader `WaitCommEvent` cadence,
  and richer accelerometer/magnetometer sample semantics still need trace
  evidence.
- Fresh splash-phase samples still spend time in `UID1:` probing and
  resource/map loading before `COM7:` opens, so sensor data cannot explain the
  earliest visible splash interval. The current post-COM7 path reaches GPS and
  sensor devices plus map-era child windows, but final map presentation remains
  gated while the framebuffer still shows the iNavi splash artwork.
- Startup profiling still reaches real `happyway_win.exe` table/traffic
  preparation loops, but the scheduler no longer leaves an idle visible helper
  process ahead of a runnable visible parked UI process outside the live-pump
  branch. The remaining delay is not explained by unreadable MessageBox UI,
  remote input routing, or pre-COM7 sensor queuing.
- The stale `happyway_win` escaped visible-message callout no longer remains
  in `pending-wndproc`; keep watching for fresh send-stack or wait-state leaks
  before treating the splash transition as purely app readiness.
- Receiver-terminated sends no longer leave active send-stack depth behind in
  live startup; the current completed `happyway_win` send result is retained for
  sender observation without blocking receiver state.
- Remote-server control input no longer remains stuck in `pendingControlMessages`
  while live-pump CPU execution stays inside import-heavy guest work,
  block-hook-visible guest work, or inline wait timeout completions; the current
  live run drains the queue and posts touch messages to the real splash window.

## Build And Validation Risks

- The normal validation profile uses `--features unicorn,trace,win32-desktop`.
  No-feature `cargo test -j 1` is now a supported smoke profile for stub/raw
  runtime paths, but it is not a substitute for the Unicorn feature gate.
- `registry.reg` loading accepts REGEDIT text through UTF-8/lossy decoding and
  decodes typed `hex(2)`/`hex(7)` values as UTF-16. Non-UTF-8 ANSI quoted
  strings from future REGEDIT4 exports may need code-page-aware decoding.
- `git diff --check` may report CRLF warnings on existing files. Treat new
  non-CRLF whitespace findings as actionable.

## Recently Closed

- Unsafe voluntary raw-yield helper interior: genuine native fresh replay
  originally lost a not-taken branch path without PC0. Removal of the unsafe
  fallback preserves the existing safe-yield policy; both outcomes/deferred
  destinations/fresh scalar replay pass. BED2 passes1,593 tests and the actual
  330.234s GPS/tap run; it remains splash, so no startup-causality claim follows.
- Checked initial allocation and remaining scoped native typed-message routes
  pass all 1,571 tests, feature/build/fmt checks in the installed 354AA0FB build.
  Full destination RW/RAM preflight, zero-size physical byte, failed-create
  teardown/scratch lifetime, sent identity/expiry and complete MSG output
  protection are covered. Its actual app run subsequently failed as above; this is not
  a map-frontier, memory-reduction or universal scheduler-rollback claim.
- Authenticated archived WNDPROC recovery now restores SP/GPR/HI/LO (including
  K0/K1) with validated ownership and native readback, instead of PC-only
  history. Native caller-epilogue/refusal tests pass; the normal 330 s app run
  proves original 0x7ffdf578 restoration. This does not close splash-to-map.
- Two visible typed-payload preparation paths and NotifyIcon's actual CE6
  152-byte writer are fixed, with sparse persistent-page coverage and adjacent
  allocation guards. The integrated frozen executable passes 1,552 tests and
  the actual 330 s GPS/tap run; remaining native delivery routes stay open above.
- Nested GPS callback ancestry is preserved without changing the 6144-byte
  grace. MIPS return-chain regressions pass; actual injected NMEA is read in
  the latest clean 300.773 s run, with no PC-zero/guest CPU exception.
- Historical CreateProcess traces no longer reverse pre-run choices, and
  CLI wall limits apply at the loop head. Corrected 600 s run exits cleanly
  at 600.45 s, without the former external-watchdog livelock termination.
- YAS526B rough/normal packed-sample semantics are corrected. Nine focused
  tests pass; no custom -15 modal appeared in the corrected 600 s run.
- Worker identity is not changed merely because another thread has a visible
  window. Real thread 6 continues executing and consumes injected GPS bytes.
- Callback FIFO selection checks receiver eligibility before message removal;
  DOWN/UP and filtered non-head regressions pass. A 330 s actual-input run
  delivers DOWN before UP with clean guest returns; this is not map bring-up
  and does not close the measured delivery lag.
- Live-pump CreateProcess timeout replay is fixed: parent API return state is
  committed before child execution. The former RA `0x00059ba0` replay site ran
  once in the fixed 120 s count=0 / 100 ms run instead of 16 times; PIDs stop at
  68. The actual-runtime parent/child regression passes.
- Scheduler TB starts now use a block-specific predecessor delay-slot guard;
  ordinary targets/fallthrough are eligible, and page-crossing delay slots are
  covered by a real Unicorn count=0 regression.
- Global-hook and diagnostic requests now take precedence over sparse flags;
  both no-trace Unicorn feature profiles compile.
- The handle-like `pc=0x00001054` no-hook crash is no longer reproduced in the
  current startup path after blocked scheduler contexts were excluded from
  direct receiver-work eligibility.
- Stale blocked-wait records for the thread that just executed are cleaned up
  at host wall-clock slice stops.
- Stale parked `GetMessage` waiters no longer force short host idle-poll slices
  while active guest CPU work is runnable.
- Remote touch/key input reaches the real iNavi window stack after the
  scheduler handoff fix. Current COM7 consumption is proven by successful
  ReadFile payload/transferred-byte records, not host queue emptiness alone.
- Stale escaped visible-message WndProc callouts no longer block later visible
  input dispatch after the CPU is already parked at their resume PC.
- Receiver/window teardown clears active send stacks for terminated synchronous
  sends.
- Missing coredll exports for `GetDeviceInformationByDeviceHandle` and
  `GetDeviceInformationByFileHandle` are closed; they now return CE-style
  device information for live device handles and covered failure cases.
- Missing coredll exports for `FindFirstDevice` and `FindNextDevice` are closed;
  they now enumerate activated device handles with CE-style search state and
  no-match/stale-handle behavior.
