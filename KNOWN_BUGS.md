# Known Bugs And Risks

Refreshed on 2026-10-15. This file lists unresolved behavior only; resolved entries are removed (history: docs/evidence/ledger.md and git).

## Open Issues

- **Native memory performance remains a live-run gate:** a synthetic
  8,192-page (32-MiB) fixed heap originally took 78,689.550 ms to map and
  another 75.753 s to close its fresh owner. Fixed private heap/stack pages
  now use sparse 1-MiB host blocks and per-page pins/budget; the final
  release control mapped the same 8,192 pages in 32 calls and 5.056 ms,
  closing in 0.530 ms. The same-binary old-shape control omitted the old
  end-of-slice readback copy, so no full-slice speed ordering is established.
  This is not an iNavi speed, lag or RSS result
  ([mapper receipt](evidence/om2-native-map-profile-20261014/validation.receipt.md)).
- Public shared-view slabs and ordinary private `VirtualAlloc` slabs still
  have independent 4-KiB host pages and one `mem_map_ptr` per mapped page.
  They are deliberately outside the fixed heap/stack block migration; the
  saved iNavi evidence does not report a shared-view page count. Their
  real-run cost and any further grouping remain unresolved.
- Stage 8 (persistent owner, paused): E-32 crashed at 135.3 s and the 650
  trial at 129.1 s, both in TCG code emission, cause unattributed. E-33 fixed
  only the small-arena commit-tail arithmetic. Still open: VEH-handler versus
  owner-teardown race, x64 trampoline unwind metadata, the 1 GiB per-engine
  arena, E-12 map/unmap churn, in-hook register helpers that zero failed reads
  and drop write errors, and unaudited o32 float-return restores (ledger E-2
  to E-6, E-12, E-32, E-33).
- VM coherence still unproved: guest `CreateProcessW`/scheduler handoff (the
  fixtures select the child manually), distinct public mappings at one VA, and
  full machine clone relineage. A same-PID DIB view without VA metadata can
  Flush but not Unmap. The immediate CreateProcess view-sync failure path has
  no fault injection. `VirtualAllocEx`/`VirtualQueryEx` name exports are
  absent. History: ledger E-14 to E-41.
- D-008 charges the whole anonymous mapping at first public `MapViewOfFile`,
  not at `CreateFileMapping` as documented for default `SEC_COMMIT` in
  [Microsoft's CE mapping reference](https://learn.microsoft.com/en-us/previous-versions/windows/embedded/ms923951%28v%3Dmsdn.10%29);
  CE6 device-specific accounting remains unmeasured. Raw/non-Unicorn alias
  propagation is also not rollback-atomic after a failed guest write; native
  pointer aliases share canonical bytes directly
  ([D-008 receipt](evidence/d008-shared-view-20261014/validation.receipt.md)).
- GMS's boot-reserved physical RAM and stable private-page commit use one
  budget, with four public controls passing both E-39 broad suites. Only
  stable private pages, D-008 stable shared-view pages, and E-44's committed
  fixed heap/stack slabs (including O-M2's sparse heap growth) enter the
  source-gated accounting path. Persistent native pin budget release is
  deferred until checked owner closure, while image and legacy-view memory
  are not charged by this budget. Copied heap spillover was removed in O-M2.
  It is not a complete guest-memory census
  ([GMS receipt](evidence/gms-device-report-20261014/validation.receipt.md),
  [O-M2 receipt](evidence/om2-heap-20261014/validation.receipt.md)).
- The new main-EXE sealed R/RX path does not cover mutable image pages,
  preloaded/runtime DLLs or trap pages. E-44 source-gated fixed initial
  heap/stack slabs; O-M2 birth-qualified the sparse fixed heap and removed
  copied spillover. Exhaustive host-write inventory, memory-mode checkpoint
  relineage, `PrivatePageBudget` treatment and an approved matched app run
  remain open. Unconverted direct-import outputs can still fail closed on a
  cold, untouched stack page; a per-thread/slot 128-KiB contiguous low-water
  is conditional on an approved live run encountering that bypass. O-M2's
  exact stale different-birth serial/NMEA control passes, but no new iNavi
  memory, speed or map-readiness result has been measured
  ([O-M2 receipt](evidence/om2-heap-20261014/validation.receipt.md),
  [E-44 receipt](evidence/heap-stack-20261014/validation.receipt.md),
  [E-43 receipt](evidence/sealed-main-image-20261014/validation.receipt.md),
  [E-42 receipt](evidence/immutable-image-pages-20261014/validation.receipt.md)).
- Stage 3 task authority is not accepted. The passed bounded gates are listed
  in `PROGRESS.md` ("2026-10-15 Stage 3 source gates"). Still open:
  - Boot-main enumeration and durable child-primary identity controls stay
    ignored (ledger E-8, E-11, E-26).
  - CE caller wait/timeout behavior is not implemented. An old-MIPS
    direct-kernel real-handle continuation fails closed, the root process has
    no owned child-launch handoff for real-handle requests, and the
    current-process pseudo-handle keeps a separate synchronous path.
  - `TaskWaitStore` separates exact births from explicit legacy-unknown TIDs;
    thread-directed post/quit delivery, status and wake, HWND-less thread
    timers, and W-B1a window creation/explicit HWND post/typed payloads now
    use frozen birth ownership
    ([wait receipt](evidence/stage3-wait-owner-20261015/validation.receipt.md),
    [GWE receipt](evidence/stage3-gwe-thread-birth-20261015/validation.receipt.md),
    [timer receipt](evidence/stage3-thread-timer-birth-20261015/validation.receipt.md),
    [window receipt](evidence/stage3-window-owner-birth-20261015/validation.receipt.md)).
    W-B1b separately source-gates bounded remote touch/key target birth
    ([input receipt](evidence/stage3-window-input-birth-20261015/validation.receipt.md));
    W-B1c preserves mixed touch/key admission order within one `CeRemote`
    ([order receipt](evidence/stage3-mixed-input-order-20261015/validation.receipt.md)).
    Broadcast, HWND-owned timers, hotkeys, sent/paint/modal and other input
    producers, full exit capture retirement, implicit gesture ownership and
    host teardown remain outside these gates; their reuse isolation is unproved.
  - W-B1c does not establish physical arrival order across HTTP and desktop
    producers, ordering with NMEA/serial batch dispatch, CE6 focus/activation
    parity, cross-thread guest WNDPROC execution order or O-C5 guest-time
    assignment. Host `GetMessage(NULL)` wake without a HWND is separate.
    W-B1b removed hidden/off-point hint delivery and retention of input with
    no live target. Earlier E-38 and callback-drive live tap-delivery evidence
    predates both W-B1b and W-B1c routing; the next approved live run must
    verify taps still reach the intended iNavi window.
  - Opaque non-task wait slots (queue, socket, serial tokens) and duplicated
    native wait payloads are unpinned despite exact-birth publication. Wait-ID
    wrap/reuse and live modal continuation behavior remain unproved.
    Synchronous finite inline timeouts publish no scheduler wait. Raw
    synchronous/nonblocking MsgWait,
    queue/socket and serial (including overlapped) paths have no birth guard.
  - The guest return-stub hook has no end-to-end control, and pending-return
    KData PID follows mutable process selection, not the captured lease.
  - `SendMessageTimeout` global expiry can read as ready/success; the overloaded
    `SMF_TIMEOUT` flag still needs a separate semantic gate. Receiver-side
    `DestroyWindow` result writes, cross-process callback transfer, legacy
    numeric-only HWND ownership and the broader GWE send lifecycle remain
    ungated. The former ordinary queued-send receiver-TID/exit defect is a
    historical pre-repair source red: after B exited and B-prime reused its
    TID, B-prime took B's send without A's failure result
    ([receipt](evidence/2026-10-15-ordinary-queued-send-receiver-exit-reuse-red.md)).
  - **Reciprocal Send shape remains narrow:** the A11b next-lease replay is a
    [historical source red](evidence/scheduler-source-cut-a11-reciprocal-red/validation.next-lease.receipt.md);
    A12's post-format focused controls cover one birth-owned, same-process,
    plain S→T→S pair and its exact inner/outer return. A third concurrent
    sender, timed/result-pointer Send, early result/reply, second pending
    return or deeper nesting, and cross-process pairs are not supported by
    that handoff. An ineligible
    blocked Send fails closed with terminal `Backend` instead of replaying its
    import; this is a live-run risk, not CE parity or a reproduced iNavi
    deadlock. The checked callback SP is not a complete stack-allocation
    bound proof. A12's two broad source suites passed; live behavior and
    these unsupported shapes remain separate gates.
  - Live-run risk: a second concurrent cross-thread SendMessageTimeout now
    stops with an explicit backend error until task-owned SMT frames exist
    ([receipt](evidence/stage3-smt-occupied-20261015/validation.receipt.md)).
  - Aliasless Sleep-suspend, unpinned-worker lineage (fails closed), other
    parked-wait continuation, clone relineage, complete GMS reclamation and
    live-app behavior.

- Historical: build 354AA0FB failed at 260.174 s with PC 0 on pool worker 10
  (RA `0xe6de4`). Later builds ran 330 s cleanly, so it has not been
  reproduced and its cause is unattributed. PC 0 remains a hard failure.
- O-P1 hardening (outside the classification contract): fallible-tail effect
  salvage, genuine C query-error and competing-hook controls, and state
  consistency for other ready-resume families on a native-entry failure. An
  exploratory same-handle nested-start hook probe crashed with an AV of
  unknown cause and was removed from default tests. Receipts: ledger E-28 to
  E-31.
- `HeapFree` does not reuse addresses. O-M2 prevents foreign-only heap
  free/size/validate and cross-birth cursor charges, but failed prepared
  allocations can leave birth-local monotonic high-water/OOM holes. Original
  reservation footprint and allocation incarnation are still needed before
  safe reuse; no free list, cross-process page reuse or speculative unmapping
  has landed. `ORD_REMOTE_HEAP_*` currently ignores its target argument and
  acts under the selected birth; `GetClipboardDataAlloc` sizes a global
  source handle under the current birth, so those cross-birth paths remain
  unresolved ([O-M2 receipt](evidence/om2-heap-20261014/validation.receipt.md)).
- Host footprint: E-38 (fresh default) peaked at 430.7 MiB RSS, against BED2
  at 555.6 MiB and persistent runs at about 1.5 GiB (ledger E-1, E-22/E-27/E-29,
  E-38). No app run has measured E-41 to O-M2. A cold TCG cache per fresh owner
  is a possible Stage 8 cost. Translation arenas and engine copies are
  unmeasured; virtual reservation is not resident RAM.
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
