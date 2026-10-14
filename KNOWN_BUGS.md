# Known Bugs And Risks

Refreshed on 2026-10-14. This file lists unresolved behavior only; resolved entries are removed (history: docs/evidence/ledger.md and git).

## Open Issues

- The E-32 source-matched, read/write bounded iNavi run exited with host
  `0xC0000005` after 135.282 s, before its planned tap. The matching
  minidump/PDB locate a 4-byte write crossing a host page boundary in
  Unicorn's `tcg_out32`/`tcg_out_movi` during `uc_emu_start`; there is no
  terminal PPM, guest CPU exception, or evidence of map readiness. The dump
  omits code-arena bounds and VEH return, so the known un-clipped Windows
  commit-tail size bug cannot yet be assigned as this crash's cause.
  Retained 121 post-120 completed slices disprove total native idleness, not
  the reason for the later no-completion gap or persistent splash
  ([E-32 receipt](evidence/e22-live-20261014T002535Z/analysis.receipt.md)).
  E-33 repaired that isolated handler arithmetic defect and passed native and
  broad tests, but no post-repair app run has tested this failure.
- Persistent-owner 650 trial: host AV at 129.1 s with 1.66 GiB RSS in TCG
  TranslationBlock initialization; cause unattributed. EXE/PDB/minidump are
  preserved under `evidence/legacy-target-release-20261013` (ledger E-2 to E-4).
- TCG code-arena commit tail: the small-arena arithmetic defect is fixed in the
  vendored source (E-33). Still open: concurrent in-flight VEH-handler versus
  owner-teardown race, x64 trampoline unwind metadata, and the default 1 GiB
  per-engine footprint (Stage 8 lane, paused; ledger E-5, E-6, E-33).
- The persistent-owner E-12 map/unmap churn control failed at its first mapped
  phase (Stage 8 diagnostic, ledger E-12). Legacy in-hook register helpers
  still zero failed reads and drop write errors; o32 float-return restore sites
  are unaudited (CPU page, E-12 follow-ups).
- E-35 public private slabs and E-36 source-bound fresh-per-slice default are
  validated. The first post-switch iNavi run failed at 2.9 s on mapped-view
  readback; E-37 repaired that path and the separately approved E-38 run
  survived 330 s at 430.676 MiB sampled RSS, but remained on the splash with
  no map. E-39 proves public private-VM native A→B→A same-VA isolation and
  injected file-view native A→B→A isolation, but public `MapViewOfFile` still
  uses a global view cursor; public same-VA A/B views, named reopen and
  same-mapping alias coherence remain unproved. A same-PID DIB view without
  VA metadata can Flush but not Unmap; the immediate CreateProcess view-sync
  failure path has no fault injection. `VirtualAllocEx`/`VirtualQueryEx` name
  exports remain absent. History: ledger E-14, E-16, E-18, E-20, E-22 to
  E-25, E-35 to E-39.
- GMS's boot-reserved physical RAM and stable private-page commit use one
  budget, with four public controls passing both E-39 broad suites. Only
  stable private pages charge this budget: heap, image and legacy-view memory
  do not reduce reported available physical memory. It is not a complete
  guest-memory census
  ([GMS receipt](evidence/gms-device-report-20261014/validation.receipt.md)).
- Canonical Task/alias ownership is absent: 12 expected-red Task controls live
  in `tests/` (ledger E-8, E-11, E-26). Process birth and alias lifetime are
  fixed (E-21); thread/task aliases, wait pins and clone lineage are not.

- Historical: build 354AA0FB failed at 260.174 s with PC 0 on pool worker 10
  (RA `0xe6de4`). Later builds ran 330 s cleanly, so it has not been
  reproduced and its cause is unattributed. PC 0 remains a hard failure.
- O-P1 hardening (outside the classification contract): fallible-tail effect
  salvage, genuine C query-error and competing-hook controls, and state
  consistency for other ready-resume families on a native-entry failure. An
  exploratory same-handle nested-start hook probe crashed with an AV of
  unknown cause and was removed from default tests. Receipts: ledger E-28 to
  E-31.
- HeapFree does not reuse addresses or release per-process spillover pages.
  Initial/reallocation mapping, ZERO and metadata ordering are now validated,
  but global allocation metadata lacks authoritative private-process ownership,
  immutable reservation footprint and incarnation identity needed for safe reuse.
  No free list, cross-process page reuse or speculative unmapping has landed.
- Host footprint and phase behavior still need work. Older persistent-owner
  bounded 330 s runs peaked at about 1.49-1.55 GiB working set (E-22, E-27,
  E-29), and E-32 crashed at 1.65 GiB. The preserved fresh-default E-37
  executable peaked at 430.676 MiB in E-38, below BED2 E-1's 555.63 MiB
  sampled peak, but both remained on the splash and the runs are not fully
  input/host-matched. Fresh-default E-38 had 17.1 run-only MIPS across busy
  slices of all guest processes, but **26.4 MIPS for iNavi**, inside E-9's
  same-field hookless iNavi 22–34 range. A cold TCG cache per fresh owner
  remains a possible, unisolated Stage 8 cost. Translation arenas, guest
  blobs, file buffers and engine
  copies are separate costs; virtual reservation is not resident RAM.
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
- In E-22, `happyway_win` process 67/thread 3 revisited a completed WNDPROC
  return stub 2,527 times with no pending callout. Its recorded last return
  lacks caller context, so fabricating a recovery would be unsafe. Parent
  message delivery continued, and this is not proved to cause the splash
  stall. Preserve a child `WM_ACTIVATE` dispatch/park/resume regression that
  checks the next PC is the `DispatchMessageW` continuation (local
  `evidence/e22-live-20261013T154435Z/app.stdout.raw.txt`).
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
- E-29's pre-retry-fix bounded 330 s run again stayed on the splash despite
  10 retained positive guest `COM7:` reads (2,784 B) and down/up touch return
  records.
  The full-screen splash window `0x00020008` remained visible over 23 hidden,
  update-pending controls. Service counters stopped changing by about +120 s
  while sampled host CPU kept increasing; the previous E-27 run also reached
  a service-counter plateau. A blocked TID 10 wait has one signaled component,
  but its any/all mode is absent from the trace. Neither CPU spin, missed wake,
  splash callback failure, nor loader failure is established; trace these
  identities/causes before altering policy (ledger E-29).
- Startup profiling still reaches real `happyway_win.exe` table/traffic
  preparation loops, but the scheduler no longer leaves an idle visible helper
  process ahead of a runnable visible parked UI process outside the live-pump
  branch. The remaining delay is not explained by unreadable MessageBox UI,
  remote input routing, or pre-COM7 sensor queuing.

## Build And Validation Risks

- The normal validation profile uses `--features unicorn,trace,win32-desktop`.
  No-feature `cargo test -j 1` is now a supported smoke profile for stub/raw
  runtime paths, but it is not a substitute for the Unicorn feature gate.
- `registry.reg` loading accepts REGEDIT text through UTF-8/lossy decoding and
  decodes typed `hex(2)`/`hex(7)` values as UTF-16. Non-UTF-8 ANSI quoted
  strings from future REGEDIT4 exports may need code-page-aware decoding.
- `git diff --check` may report CRLF warnings on existing files. Treat new
  non-CRLF whitespace findings as actionable.
