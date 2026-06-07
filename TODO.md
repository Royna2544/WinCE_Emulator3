# TODO

## Artifact Note

- `target\` was cleared on 2026-06-04 to recover disk space. Any existing
  `target\...` names in this ledger are historical evidence labels, not
  guaranteed local files. Use fresh virtual-desktop probe prefixes for new
  validation artifacts.
- Mounted iNavi probes should use dumped runtime DLLs from
  `D:\INAVI_Emulator\DUMPPLZ\Windows`; SDK DLL/library paths are now evidence
  or fallback, not the primary DLL source.

## Current Slice

- CE fidelity catch-up next slice: finish the runtime guest DLL loader at the
  Unicorn import-trap boundary. The kernel-side module manager/refcount shape
  now exists and the shared CE-aware DLL search order is implemented for
  startup preload, child-process preload, and future runtime loads. The
  remaining blocker is the Unicorn-owned runtime mapper itself:
  `LoadLibraryW/LoadLibraryExW` still only reuse already registered modules and
  do not yet synchronously map a new dumped MIPS DLL from
  `D:\INAVI_Emulator\DUMPPLZ\Windows`. Implement PE map/relocate/import
  patching, dynamic trap-page refresh, external guest-DLL import resolution,
  export registration, TLS callbacks, and `DllMain` attach/detach callouts. The
  live import hook now has a mutable/persisted trap table, so the next mapper
  step can merge new DLL import traps and rewrite the live trap page, and
  `ExternalImportTable::add_module_exports` can resolve imports against
  already-loaded runtime modules. Keep `C:\WINCE600` as the behavior reference
  and update `PLAN.MD` after each port.
- COREDLL fallback audit follow-up: stubs now carry audit classification plus
  raw import-trap context with thread id, caller PC, and trap PC. Next, add the
  remaining owning-module context where available and make
  shell/UI/process/loader critical fallbacks fail loudly under an audit mode
  instead of merely warning.
- Shell fidelity follow-up: `ShellExecuteEx` now handles the basic CE launch
  chain through shortcuts, registry associations, and `CreateProcessW`
  queuing. Continue with the remaining shell APIs from the attachment:
  `SHCreateShortcut`, `SHGetShortcutTarget`, `SHGetFileInfo`, notify icons,
  notification data, recent docs, shell change notifications, popup/menu
  behavior, modal `MessageBoxW`, keyboard translation, clipboard, and caret
  state. Keep behavior registry-backed and generic; do not fake route UI.
- Winsock fidelity follow-up: guest-visible local names now use the isolated
  `10.0.0.1`/`10.0.0.2` model while host sockets remain the transport. Blocking
  `connect`, `accept`, `recv`, `recvfrom`, and `select` still need to park
  through the scheduler instead of returning `WSAEWOULDBLOCK`/short host
  timeout behavior.
- Route-search child-startup blocker closed/watch: `target\route_deviceexit1_*`
  confirms the old MIPS CE `TerminateProcess` thunk in `DeviceParser.exe`
  decodes through the interrupt/zero-PC path and the child exits cleanly. The
  same probe reaches `happyway_win.exe`, `iSearch.exe`, and
  `gwe=send:1 done:1`, then returns to the established hidden `afxwnd42u`
  chrome state. Continue by chasing the later guest show/update sequence for
  those existing child controls; do not revisit `DeviceParser.exe` unless a
  fresh trace again records `CreateProcessChildError`.
- Bounded virtual trace caveat: `target\route_deviceexit_long1_*` still does
  not reach the later route chrome reveal after 34 one-second slices. It is
  useful evidence for the clean child-exit path and the hidden-preload state,
  but the practical route-search drive should use the live host path that has
  already reached real map/modal UI, not early taps against this partial
  header/map frame.
- Route-search visibility frontier update: `target\route_showcmd2_messages.txt`
  proves the `afxwnd42u` route/chrome children are hidden by guest
  `ShowWindow(cmd=0/SW_HIDE)` immediately after creation; the bounded run does
  not yet reach a later `ShowWindow(cmd=5)` for those controls. Continue by
  driving or speeding the startup path until the guest should reveal the
  right/bottom chrome, then compare the missing show sequence against the
  successful bottom-strip/menu path. Do not force `WS_VISIBLE`: this trace says
  the preload hide is guest behavior.
- Route-search path next step after the sent-message fix: continue from
  `target\route_after_sendfix1_*`. The old `happyway_win.exe`
  `SendMessageW(hwnd=0x0002000c,msg=0x0401,wParam=14)` deadlock is closed/watch:
  CE-style internal sent-message dispatch during `GetMessageW` now completes
  the transaction and parked sender resume restores the parent context. The
  current route-search frontier is whatever follows that IPC: inspect the
  later `iSearch.exe` activation, `afxwnd42u` child-window creation/show
  sequence, route dialog lifetime, and remaining sleep/timer waits. Do not
  force visibility or change route coordinates unless fresh traces show input
  routing regressed.
- Route-search visibility frontier: continue from `target\route_pos1_*`.
  Evidence now says `iSearch.exe` starts and the route/control children are
  created, but they end hidden before any finished route chrome appears. Use
  the new `set_window_long_style` and `set_window_pos` lifecycle records to
  find the first transition that clears `WS_VISIBLE` on the `afxwnd42u`
  children (`0x00020014` and siblings). Compare against CE/MFC window style
  behavior before changing semantics; do not simply keep the controls visible.
- Route-search live timing follow-up: keep the `live_pump` guards on
  `GetMessageW` / `MsgWaitForMultipleObjectsEx` timer fast-forwarding and the
  real-wall current `Sleep` / finite-wait inline completion. Fresh evidence
  `target\route_search_resume_120ms_*` proves the red search tap reached
  `TGNaviDlg` but the 1500 ms `0x19fe` timer fired before a 120 ms host
  capture. The immediate next route probe should use the normal live run-loop
  or a monitor command that shares the same process-handoff semantics, wait for
  the full right chrome and bottom strip, then tap red search and capture
  `TGNaviDlg` before `WM_TIMER`. Do not use early startup taps against the
  partial header/map frame as route-search evidence.
- Route-search input-routing follow-up: `hwnd=any` blocked helper drains now
  use desktop hit-testing and post to the hit HWND owner's queue; keep the new
  `remote_input_any_blocked_thread_uses_desktop_hit_test_owner` coverage. The
  controlled validation `target\route_search_ownerfix1_*` no longer attempted
  a bottom-strip tap because the bottom/current-location chrome remained
  hidden for the whole wait window. Continue with a host/direct run that first
  reaches visible bottom chrome, then tap the bottom strip and inspect the
  `TGNaviDlg` `WM_TIMER`/`DestroyWindow` trace tail. Do not accept a
  `thread_id=3, hwnd=0x00020004` remote-touch record as valid; window messages
  must be queued to the HWND owner's thread.
- Route-search path next step: continue from
  `target\route_wndproc1_bottom_tap_*`. Input routing is no longer the blocker:
  saved scheduler `GetMessageW` waiters are used for host/remote input, and
  the bottom-strip tap is delivered to thread 1. The tap creates owned
  `TGNaviDlg` HWND `0x00020084`, but the window snapshot already marks it
  `dead=true` and it is not in z-order, so the presenter remains on the map.
  Trace the dialog creation/destruction path next: capture the create/destroy
  WNDPROC return, `EndDialog`/`DestroyWindow` caller, dialog result, resource
  lookup failures, and any dependency on `iSearch.exe`/`happyway_win.exe`
  process IPC. Do not keep changing coordinates or force the dialog visible.
- Route-search path: do not drive taps while the frame is still the partial
  header/map composition. `target\route_path_drive1_montage.png` proves all
  route-path taps were accepted by REST but landed before the right chrome and
  bottom current-location strip were visible. Continue from
  `target\route_chrome_rich1_*`: child windows for the right controls and
  bottom strip exist but remain hidden while `iNavi.exe` is still loading
  `resmapi_800x480.bin` RSImage resources through `CreateDIBSection`. Next
  work should either reduce the live-slice remap/teardown tax safely or chase
  the CE/MFC resource/window-show sequence that makes these existing child
  windows visible. Do not force visibility or hardcode iNavi UI state.
- Live virtual/remote process rotation is fixed/watch. Keep the
  `should_rotate_parked_process(... live_pump && wall_stop ...)` behavior so
  remote virtual runs schedule parked children like host runs. The validation
  artifacts are `target\route_chrome_block_v1_processes.txt` versus
  `target\route_chrome_rotate_v1_processes.txt`: the latter repeatedly cycles
  `iNavi.exe`, `happyway_win.exe`, and `iSearch.exe`. The route chrome still
  does not appear, so the next blocker is not "children never run" but resource
  throughput/hidden-child show sequencing.
- Startup profiling follow-up: the per-page Unicorn heap-spillover map churn is
  fixed/watch. Keep `map_guest_range` span batching only on heap spillover and
  keep virtual allocations page-granular unless `reclaim_stale_virtual_memory_pages`
  is made span-aware. Fresh reference artifacts are
  `target\grouped_map_final_60s_summary.txt`,
  `target\grouped_map_final_60s.png`, and
  `target\grouped_map_final_flame.svg`: release virtual startup completes in
  16.85 s wall time, with `map_kernel_memory_allocations` down from 33.91% to
  1.16% and `map_heap_spillover` down from 33.91% to 0.17%. Next useful speed
  work should chase the remaining CE scheduler/display/app-flow frontier,
  `map_persisted_ram_blob_pages`, hook overhead, and process handoff behavior;
  do not reopen file preload/reopen or per-page heap mapping unless fresh
  traces regress those counters.
- Remote live-pump follow-up: `target\remote_liveslice_fix1_*` closes the
  early remote `FETCH_UNMAPPED pc=0x0000353c` regression caused by using a
  50 ms wall slice as an implicit wait-hook policy. Continue mounted launches
  with `--remote-server 192.168.0.39:8765`; remote/host live runs now use the
  explicit `UnicornRunLimits.live_pump` flag to avoid host sleeps in wait
  hooks, while the remote service slice is 1000 ms. If remote responsiveness
  still feels bad, tune the service cadence separately from wait semantics and
  do not restore tiny arbitrary Unicorn wall stops.
- Route-drive retry after live-pump fix:
  `target\remote_route_after_liveslice1_montage.png` shows the remote endpoint
  stays live and accepts taps, but the app remains on the partial header/map
  frame for the whole drive sequence. Continue route-search work by fixing the
  missing right/bottom chrome and parked child/process/GWE handoff; do not
  spend the next slice on remote touch delivery unless new traces show queued
  touches are not reaching GWE.
- Startup profiling follow-up: persisted-RAM page-by-page remapping is now
  fixed/watch. Keep the contiguous-span remap path validated by
  `persisted_ram_blob_remap_groups_unmapped_spans`; post-fix
  `target\route_search_host_90s_postspan_flame.svg` no longer shows
  `map_persisted_ram_blob_pages` / `uc_mem_map` as a top block and advances
  farther through the mounted host startup slice. Next useful speed work should
  profile the now-visible TCG/code-generation and hook paths
  (`uc_emu_start`, `tcg_cpu_exec`, `tb_find`, `tb_gen_code_mipsel`,
  import/code hooks) and only then consider longer-lived Unicorn instances.
  Do not reopen host-file preload/reopen or per-page persisted-RAM remap unless
  fresh flame evidence regresses those counters.
- Immediate stuck-screen next step: implement the child-process scheduler
  handoff properly instead of reviving the backed-out host-loop rotation
  experiment. Evidence in `target\stuck_process_processes.txt` shows
  `happyway_win.exe` parked as process 67/thread 3, while the parent later
  queues a cross-process `SendMessageW` to HWND `0x0002000c` and keeps working
  through resource/allocator code. The failed experiment proved that simply
  yielding after `CreateProcessW` and round-robining parked child CPUs can
  enter child code but breaks bounded validation. The next implementation
  should make cross-process `SendMessageW` and process waits scheduler-owned:
  register the sender wait, activate the parked receiver process/thread with
  the correct process address space, return through the sender context when
  the receiver replies, and keep explicit wall budgets enforced across nested
  child execution.
- Immediate startup/UI frontier: continue from
  `target\host_after_importgate_30s.png`,
  `target\host_after_importgate_100s.png`, and
  `target\host_after_importgate_200s.png`. The mounted Win32-host run with
  `--remote-server 192.168.0.39:8765` is live/responsive and memory-bounded,
  but remains at the real header/map composition with missing right-side and
  bottom chrome. Progress samples are hot in `HeapAlloc @46` / `malloc @1041`
  and app/resource code, while file I/O remains bounded to a few MB. Next work
  should profile/trace the RSImage/resource allocator loop around return
  addresses near `image:iNavi.exe+0x88a3xx` and `image:iNavi.exe+0x329a68`,
  and determine why the receiver WNDPROC for the outstanding cross-thread
  `SendMessageW(hwnd=0x0002000c,msg=0x0401,wParam=14)` does not return to
  reveal the chrome. Do not treat this as a dead host window, black-screen
  presenter issue, map DB preload, or stale hidden-layer leak unless fresh
  evidence regresses those closed paths.
- Fast-start follow-up: `WINCE_EMU_FAST_START` is still not production-ready.
  It no longer immediately exits via `ce-import-traps+0xfff0` after generated
  trampoline-origin `lui/ori` handling was added, but sparse hooks and
  instruction slices do not yet outperform the normal global-hook path. Keep it
  disabled for mounted host validation until it can reach at least the current
  partial-map frame faster than the normal path and still honor host input,
  remote input, wall stops, import traps, and scheduler wakes.
- Route-search process/GWE handoff is now the active blocker. Continue from
  `target\route_drive_procfix1_*`: `wcstoul @1083` is fixed and
  `CreateProcessW` children that park without an encoded CE exit now remain
  `STILL_ACTIVE` instead of being marked exited. The parent still reaches its
  encoded exit path after launching/parking `happyway_win.exe` and
  `iSearch.exe`; the final scheduler state includes an outstanding
  cross-thread `SendMessageW` from thread 9 to HWND `0x00020004`. Next
  implementation work should keep parked child Unicorn CPU contexts owned by
  the main emulator and hand execution to a live child process when the active
  process exits, while preserving shared CE kernel/GWE/mapping state. Do this
  as generic CE process scheduling; do not special-case `iSearch.exe` or force
  route UI state. Add a focused fixture for a parent that `CreateProcessW`s a
  child, exits, and leaves the child window/message pump alive.
- MultiTBT / companion follow-up: v3 can now launch v2-style diagnostic
  companions explicitly with
  `--companion-image D:\INAVI_Emulator\INAVI\TBT\MultiTBT.exe` while the main
  mounted run continues to use `--remote-server 192.168.0.39:8765`. DUMPPLZ
  has no direct MultiTBT launcher reference, so do not auto-start or hardcode
  `MultiTBT.exe`: `..\wince_emulator_v2` only proves the harness-launched path
  was viable. The old companion startup crash is fixed: loaded DLL images now
  get a second external PE import pass so MFC's late `commctrl.dll` ordinal
  imports resolve, and `AddFontResourceW` now succeeds against mounted font
  files. Next route-search validation should include the companion and compare
  against the current no-companion route probe, but treat any failure to affect
  route search as the real CE-like process/mapping/window IPC gap. v3 external
  companions still do not share v2's window registry/message broker or CE named
  mapping state, so continue with the MappingSystem/process-handle manager work
  rather than app-specific shortcuts.
- Startup profiling follow-up: Windows-sudo flamegraphs
  `target\startup_flame_virtual_sudo1.svg` through
  `target\startup_flame_virtual_sudo3.svg` confirm the mounted virtual startup
  path is no longer dominated by huge file preload, host-file reopen, import
  name lookup, or per-import trap/string/argument cloning. Continue from the
  final frontier at `COREDLL.dll@496` (`Sleep`) with bounded file counters and
  post-map scheduler state. Next useful speed work should measure/attack
  remaining generic costs: Unicorn code-hook/timeslice overhead, raw COREDLL
  dispatch frequency, `combine_rgn_raw`, streamed `read_file_into`, guest
  memcpy, and scheduler/device waits. Do not reintroduce app-specific fast
  paths or fake UI progress.
- Current menu/action frontier: top-right `메뉴` taps are delivered through
  `GetMessageW` to HWND `0x00020004`, and the remote input bridge now gives
  down/up separated `MSG.time` plus correct `VK_LBUTTON` key state. Fresh
  mounted probes `target\menu_popup_touchtime1_*` and
  `target\menu_popup_lbutton1_*` still leave the bottom action controls hidden:
  no post-tap `ShowWindow(SW_SHOW)` appears for children `0x00020060`,
  `0x00020068`, or `0x0002006c`. `target\menu_bottom_compare1_*` proves the
  bottom current-location strip (`0x00020070`) is a separate child shown and
  painted later by the guest, and tapping it legitimately opens the full Route
  Search shell (`0x00020084`). Continue by tracing the top-right parent
  WNDPROC path after `WM_LBUTTONUP` and compare it against the bottom-strip
  success path: inspect the missing app-private post/command/custom messages,
  capture/focus/active/disabled/style state, `GetWindowLong`/`GetDlgCtrlID`
  results, and MFC pretranslation. `ChildWindowFromPoint` is now CE-shaped for
  hidden/disabled immediate children, but the old top-right trace did not call
  it during the failed tap. The sharper evidence is that parent `0x00020004`
  enumerates hidden child rects and sends a child mouse message to
  `0x00020068`; compare the resulting `CallWindowProcW`/same-thread
  `SendMessageW` path against the bottom strip's successful child `WM_LBUTTON`
  path and `0x5734` post. Do not fake the child visibility or force iNavi menu
  state.
- Current live UI frontier after `_hypot`: `target\hypot_route_host1_*`
  supersedes `target\modal_drive_host1_*`. `_hypot @1023` is implemented, and
  the mounted Win32-host run now drives through safety, closes the
  destination/current-location modal, opens the bottom menu, selects
  `목적지`, selects the highlighted row, and taps the red search button before
  the 300 s diagnostic wall-stop. Continue from the valid post-search state:
  final stop is a normal bounded wall-stop at
  `pc=0x00114cc4(image:iNavi.exe+0x104cc4)` with main `GetMessageW`, worker
  sleeps/kernel waits, timer `0x11d5`, and bounded file/RSS counters. Next
  probe should use either no wall or a longer wall budget when manually driving
  beyond search, and should trace exact input targets if a visible control
  fails to react. Do not reopen `_hypot`, safety dismissal, stale framebuffer,
  or route-modal X unless fresh evidence regresses them.
- Safety-dismiss stale-frame follow-up is closed for the generic GWE exposure
  case. `target\gwe_exposure_host2_after_tap.png` proves the safety notice is
  gone after the accepted OK tap; the apparent "fullscreen dismiss not
  happening" was stale pixels because v3 did not invalidate windows exposed by
  destroy/hide paths. Continue from the real modal/ANR/device continuation
  after the route/current-location dialog, using host mode and exact
  `messages` traces for whichever control is unresponsive. Do not revisit
  safety-window hit-testing or presenter framebuffer push unless a fresh run
  again shows a live safety HWND or stale exposed pixels after this fix.
- Immediate live-host follow-up: no-wall Win32-host launches now drain remote
  REST controls and host input from the live Unicorn tick even when no
  `--cpu-wall-clock-limit-ms` is set. `target\live_host_after_drain_fix.png`
  proves a mounted no-wall launch with `--remote-server 192.168.0.39:8765`
  accepts the safety OK tap and reaches the real map UI while staying
  responsive and memory bounded. Continue from the post-map responsiveness and
  device/scheduler frontier: if a specific map control feels dead, run a fresh
  no-wall or bounded host trace with `messages`, deliver the exact tap, and
  inspect the guest handler/timer/device continuation after the delivered
  `WM_LBUTTONDOWN/UP`. Do not revisit black framebuffer, safety-screen REST
  delivery, or multi-GB file/RSS theories unless fresh evidence regresses them.
- Bounded validation follow-up: host `--cpu-wall-clock-limit-ms` now uses a
  total run-loop budget. Use this for short probes without accidentally
  continuing past the requested wall bound into the singleton/exit path.
  `target\bounded_total_45s_*` is the current clean bounded evidence label.
- Remote-server singleton follow-up: safety-screen REST delivery is now closed
  for both bounded and no-wall host runs. The duplicate `iNavi` mutex plus
  self `FindWindowW("iNavi")` singleton/exit trace should be treated as a
  separate startup/teardown frontier only if it reproduces in a fresh no-wall
  run after real map progress. Today’s bounded overrun showed that the old
  validation harness could continue past its requested wall budget and then
  manufacture a misleading second `main_init`/singleton abort. If a fresh
  no-wall run reaches the same branch without a harness wall-stop, investigate
  first mutex/window lifetime, CE visibility/top-level matching, and MFC
  startup re-entry. Do not weaken named mutex semantics or hardcode iNavi
  controls.
- Scheduler bridge follow-up after the MsgWait no-peer fix: over-budget
  `MsgWaitForMultipleObjectsEx` now parks the current thread as a
  scheduler-owned blocked wait and stops Unicorn instead of falling through raw
  dispatch when there is no suspended peer to activate. `WaitCommEvent` now
  follows the same no-peer parking rule, and serial read/comm-event blocking
  purge stale vector-backed waits before registering replacement waits.
  `Sleep(0)` now records a CE-style yield and returns to the same guest thread
  when no runnable peer exists. `WaitForMultipleObjects` and
  `MsgWaitForMultipleObjectsEx` also now hand off to already-ready blocked
  waiters after parking the current thread, instead of stopping behind an empty
  suspended slot. Empty serial `ReadFile` and `WaitCommEvent` now use that same
  ready-waiter handoff before self-timeout/stop. Continue converging the
  remaining wait/send/timer/device handoff paths onto the saved-context FIFO
  run-queue model so signaled waiters are not stranded behind a single
  suspended slot. Fresh short validation
  `target\sched_serial_handoff_virtual_60s_*` reaches a real populated map
  framebuffer with bounded memory/file I/O but stops in guest image code before
  registered scheduler waiters appear, so the next proof run should be a
  longer host/manual or trace-selector probe aimed at the later post-map ANR
  frontier.
- Current host/manual ANR slice after timer coalescing: continue from
  `target\host_timer_pending_300s_*`,
  `target\host_windows_220s_*`, and
  `target\host_modal_lateclick_300s_*`. The CE timer pending-message fix closes
  the repeated timer `4565` post streak: fresh traces show one pending
  `WM_TIMER` `4565`, all synchronous sends completed, responsive host Win32,
  and bounded file/RSS counters. The visible stop is the GPS initialization
  warning modal (`Error Code: -14`) with two top-level `TGNaviDlg` windows:
  underlying full-screen `0x00020080` and top modal `0x00020084`
  (`182,99-618,381`). Injected host clicks at `(400,350)` were delivered
  through GWE as `WM_LBUTTONDOWN/UP`, but landed before `0x00020084` existed
  and hit `0x00020080`; no `WM_COMMAND` followed. Next work should trace the
  exact modal creation/OK-click window timing, then chase the GPS/serial/Deneb
  state that leads to the warning and encoded terminate path. Do not treat this
  as a presenter, framebuffer, timer-flood, hidden-layer, or file-I/O problem
  unless fresh evidence regresses those closed paths.
- Host/manual input follow-up: `target\remote_ok_trace_*` and
  `target\live_host_after_drain_fix.png` supersede the older safety-screen
  ambiguity. Safety OK now advances to the map through real `WM_LBUTTONDOWN/UP`
  delivery. Future manual probes should target the exact post-map control that
  feels unresponsive and then chase the guest MFC handler, timer, serial/Deneb,
  or scheduler continuation after delivery.
- Current host/manual ANR slice: continue from
  `target\host_getmsg_sendwake_300s_*`. The pending synchronous-send deadlock
  from `target\host_handoff_300s_*` is closed: the blocked-current
  `GetMessageW` resume helper now wakes a parked UI thread when
  `current_thread_id` still names that thread but `running_thread` is empty,
  and fresh host evidence reaches the full 300 s wall budget with
  `gwe=send:17 done:17`. The frame is the real populated map UI, and memory/
  file I/O remain bounded. The remaining ANR frontier is now
  `pc=mfcce400.dll+0x4fda8`, `blocked_get_message=thread:1`,
  `threads=current:1/running:1:0x00000041/suspended:6:0x00000f0c:pc=0x0022fa90`,
  finite worker sleeps (`301/334/15001 ms`), active timer `0x11d5`, COM7 empty
  reads, Deneb sensor reads with missing `MS2_CalData`, and `SMB1:`/`MFS1:`
  opens. Next work should determine why the saved worker/runnable waits and
  timer/device/message work do not produce continued host responsiveness:
  inspect timeslice fairness around the suspended worker at
  `image:iNavi.exe+0x21fa90`, timer 4565 delivery cadence, COM7/Deneb/SMB1/
  MFS1 semantics, and exact delivered host-touch continuation. Do not revisit
  the closed hidden-layer leak, file-I/O/RSS growth, or pending-send deadlock
  unless fresh evidence regresses them.
- Host live-input follow-up: after the live Win32 input bridge, use
  `target\host_live_input_300s_*` or a fresh host run to click the exact
  unresponsive UI region while `--tracefile messages` is enabled. Input should
  now be polled during Unicorn execution, not only after an outer run stop. If
  the trace records `remote_touch_target` and matching `get_message`, continue
  into the guest handler/timer/device continuation; if it records
  `remote_touch_drop`, fix generic GWE active/capture/hit-test routing. The
  validation run did not record manual touch/key events, so it proves the code
  path builds and stays bounded, not that the particular UI action succeeds.
- Current host/manual post-map slice: continue from
  `target\host_sleep_getmsg_180s_*`. The fixed-interval timeslice now retries
  a pending scheduler slice after unsafe MIPS branch/trampoline/import samples,
  and current `Sleep` now yields to a ready blocked `GetMessageW` waiter when
  main has queued posted/sent traffic. The previous `target\host_blockctx_180s_*`
  thread-5 image-code frontier moved into worker sleep handoff states. The
  latest wall snapshot is still a bounded-run ANR shape, not a crash:
  `pc=COREDLL.dll@496`, `ra=image:iNavi.exe+0xd69c0`, current thread 8 has a
  long `Sleep(15001)`, main thread 1 is parked in `GetMessageW`, threads 5/6/9
  have shorter sleeps, COM7 reads are empty, and Deneb/device plus map/search
  DB reads are visible. Next slice should trace why the long-sleep/device
  frontier does not produce manual UI responsiveness: check current thread 8's
  caller path, shorter sleep maturation when the host wall budget remains,
  queued send/timer delivery after the last `GetMessageW`, and GPS/Deneb/SMB1/
  MFS1 device semantics. Do not revisit the now-fixed hidden-layer,
  fixed-sample timeslice, stale blocked-current, or file-I/O/RSS theories
  unless fresh evidence regresses them.
- Current host/manual post-map slice: continue from
  `target\host_fullctx_180s_*`. The saved-context dedupe plus full MIPS
  GPR/HI/LO preservation fixes the previous post-map
  `READ_UNMAPPED addr=0x14400018` tree-pointer fault; the Win32-host run now
  reaches the full 180 s wall budget with bounded memory/file I/O and a real
  map frame. The visible frontier is the app's GPS initialization warning modal
  (`Error Code: -14`) and continued device/message behavior behind it. Next
  work should trace the OK-click/modal path and the GPS/Deneb/COM7/SMB1/MFS1
  device inputs that lead to the warning, using host mode for manual feedback
  and message/device traces for evidence. Do not bypass the warning, fake GPS
  state, fabricate Deneb calibration files, or patch iNavi-specific pixels.
- Current host/manual slice: the hidden/pre-rendered UI layer leak is fixed by
  committed CE visible-client-region clipping (`8fa8c9f`). The live host
  presenter now shows the corrected z-order/hide-show behavior, and the Win32
  host window itself stays responsive during the long post-map MFC loop because
  window ownership/pumping moved to a dedicated GUI thread. Continue
  investigating any remaining "ANR" report as guest post-map
  scheduler/device/app responsiveness, not as a rendering leak, host window
  pump freeze, or a basic input-drop bug. The
  immediate duplicate-wait/worker-freeze shape is fixed: fresh
  `target\anr_wait_cleanup_host_*` no longer lists main thread `1` as both
  `sleep` and `get_message`, and `target\anr_worker_resume_virtual_*` proves
  an empty UI `GetMessageW` now lets finite worker waits mature/resume instead
  of stopping the whole run. That probe reaches the full 60 s wall budget in
  guest image code with `send:168 done:167`. Follow-up host evidence
  `target\host_anr_pc0_*` did not reproduce the earlier `pc=0` return, and the
  conservative Unicorn suspended-peer time-slice in `target\host_timeslice_*`
  moved visible host execution to more real GDI/map work without fixing the
  ANR completely. The time-slice can now also preempt the active context into
  the suspended slot for a ready blocked waiter when that does not overwrite an
  existing suspended peer. Next ANR work should validate this in a fresh host
  run, then grow the full CE run queue/ready-waiter model if signaled waiters
  remain parked with more than one saved runnable context. Also inspect the
  remaining in-flight send and any exact host click that still feels dead.
  Fresh `target\host_ready_preempt_*` evidence exposed multiple simultaneous
  thread-1 `GetMessageW` waiters after the ready-preemption slice; the
  per-`GetMessageW` stale waiter is now cleared before a new empty-queue
  registration, and `target\host_getmsg_cleanup_*` confirms thread 1 no longer
  has duplicate `get_message` blocked waits. Continue from the remaining clean
  frontier: one main `GetMessageW`, worker sleeps/kernel waits, active timer
  `0x11d5`, and the app-owned legacy terminate path after the bounded host run.
  The scheduler bridge now has a FIFO saved-context overflow queue for
  time-slice/ready-waiter preemption when the primary suspended slot is already
  occupied. Valid finite current-thread `WaitForMultipleObjects` waits with no
  ready handles now complete as CE `WAIT_TIMEOUT` when the timeout fits the
  host run budget, so the next scheduler work should route the remaining
  over-budget waits and guest-thread handoff paths through the queued
  runnable-context model, then validate whether worker sleeps/kernel waits at
  the clean frontier are still stranded.
  The new
  `messages` trace selector now preserves kernel-level GWE post/target/delivery
  records, including public `PostMessageW`/thread/broadcast posts,
  keyboard-post helpers, `SendNotifyMessageW`, and queued cross-thread sends.
  Fresh Win32-host evidence from
  `target\host_message_trace_{summary,messages,counts}.txt` shows a synthetic
  `400,240` tap hit-tests to HWND `0x00020080`, delivers
  `WM_LBUTTONDOWN`/`WM_LBUTTONUP` through `GetMessageW`, and then reaches the
  app's own current-process terminate path (`api2.2`, process `0x42`, code
  `0`). Next manual-host debugging should use the same trace on the exact
  control/area that feels unresponsive; if messages are delivered, continue
  into guest handler/device/timer behavior instead of adding app-specific hit
  targets. Fresh virtual evidence
  `target\public_message_trace_{summary,messages,counts}.txt` also showed
  queued worker-thread sends to the main HWND and timer `4565`; the newer
  worker-resume probe proves those sends can now churn for the full wall budget,
  so continue from the one pending send instead of the earlier idle stop.
- Current GDI map-fidelity slice: continue from `target\gdi_exttext_virtual.*`.
  The huge black base-layer gap is now fixed generically by honoring
  `ExtTextOutW(ETO_OPAQUE)` as a CE GDI background-rectangle fill with the
  selected DC `bk_color`. The mounted framebuffer now has a real light
  land/background layer (`map_crop` pure black down from `47.2826%` to
  `0.0131%`, center black down to `0.0000%`) while preserving bounded
  memory/file I/O. ROP2 pen drawing is also modeled and tested, and the latest
  clip-region slice `target\gdi_clip_regions_virtual.*` now preserves
  `CombineRgn(RGN_DIFF)` holes for memory/display `FillRect`, `Polygon`,
  `Polyline`, `BitBlt`, `StretchBlt`, and `TransparentImage`; that closes a
  real generic CE clipping bug but the mounted frame still shows road/building
  styling problems. Next GDI work should inspect concrete trace evidence for
  missing road/building primitives: line joins/caps, pen style, polygon fill
  mode, brush/palette/DIB color-table differences, ROP3/non-SRCCOPY blits, or
  any unimplemented CE GDI calls that appear in the mounted render/counts
  traces. Keep tracing generic GDI paths rather than guessing colors or
  special-casing iNavi pixels.
- Continue from `target\wcspbrk_long_virtual_*`. The hardcoded late dialog
  replay and aux alias mutation hooks are gone, and raw `wcspbrk`/COREDLL
  ordinal 68 is now implemented. The longer mounted run proves the previous
  90 s `pc=0x00a7c9e8`/`around.db` frontier was not a permanent stall: DB
  loading completes, guest GDI presents an actual 800x480 map UI to display HDC
  `0x02020004`, and the app parks at scheduler-owned `GetMessageW` with
  periodic `WM_TIMER` id `4565`, custom messages `0x52e8`/`0x5284`, COM7 GPS
  polling, and Deneb sensor reads in the evidence trail. Next step: debug
  post-map progression and the later app-owned encoded terminate path from real
  CE events and devices. Inspect whether timer 4565, repeated custom
  `0x52e8` sends, COM7 empty reads/timeouts, `SMB1:`/`MFS1:` device behavior,
  missing `MS2_CalData`, or a message-wake gap is what drives the rendered map
  to that path. Do not restore the late-init hook, patch guest state, fabricate
  files, or fake pixels.
- Serial control state is now generic and stateful enough for DCB/mask/purge
  callers, and synchronous Unicorn `WaitCommEvent` now parks through the
  scheduler until either `EV_RXCHAR` is ready under the current mask or
  `SetCommMask` wakes the pending wait with event `0`. The rendered-map
  frontier may still need deeper device fidelity. `win32_com` now opens and
  polls configured host COM ports generically, but the current machine reported
  COM4/COM9 as OK and COM21 as not a known-good device while
  `serial_devices.json` maps guest `COM7:` to host `COM21`. Next serial slice,
  if traces point there: validate a working host GPS/serial source or feed
  realistic RX through the existing remote path, then add broader modem/error
  event masks (`EV_ERR`, line status, purge/error wake details), stronger host
  COM failure counters, and CE fixture coverage for an actual blocked
  `WaitCommEvent` thread resuming from injected RX. Use
  `C:\WINCE600\PRIVATE\WINCEOS\DRIVERS\SERDEV\serial.c` as the source
  reference; keep behavior generic and test with CE fixture programs.
- Watch the older mounted destroy/slot crash evidence in
  `target\destroy_lifecycle_current_*`, but do not treat it as the active
  blocker unless a fresh post-dialog run reproduces it. CE/MFC destroy handling
  now has the correct
  fake `WM_NCDESTROY` value (`0x7fff`), records MFC-delivered NC destroy, and
  keeps windows valid while a `DestroyWindow` subtree is in the CE
  `fBeingDestroyed` phase. The remaining startup crash happens after the
  `DestroyWindow/WM_DESTROY` guest callout has returned: iNavi reaches
  `pc=0x0002c264(image:iNavi.exe+0x1c264)` and dereferences a null slot loaded
  from the global object at `+0x10ec`. Stop-PC probes prove that slot is
  initially created/stored as `0x3005bda0`, is still non-null at
  `0x0002bf30`, then is null at `0x0002c260/0x0002c264` while the guest state
  checks have `state[0x8a] == 5` and `state[0x120] == 0`. The final window
  dump now prints `destroying=false dead=true` for the destroyed subtree, so
  chase the guest continuation/slot lifetime rather than in-flight HWND
  validity. After removing the hardcoded late `WM_INITDIALOG` replay hook,
  `target\dialog_init_no_replay_virtual_*` and `target\wcspbrk_long_virtual_*`
  no longer reproduce this crash. If it returns, add a narrow diagnostic for
  writes to that slot and for setters of state index `0x120`, or statically
  find the guest setters around `jal 0x22904` with index `0x120`. Do not patch
  the slot or force the state.
- Continue from the scheduler-clean mounted probe
  `target\unicorn_wait_cleanup_virtual_60s_*`. The previous WNDPROC return
  `user-kdata` execute fault and the immediate `Sleep @496`/wait-storm
  startup blockers are closed by generic Unicorn scheduler/callout fixes:
  WNDPROC stack restoration, deferred resumes while in WNDPROC, pulse-token
  wait release, Sleep-to-ready-waiter handoff, and a tiny host throttle for
  accelerated finite current-thread waits. Stale saved waits for a thread are
  now purged before that thread registers a new Sleep/Wait context; the 60 s
  run has only the COM serial read plus one main-thread sleep active instead
  of duplicate main wait records. It reaches guest image code for the full
  wall budget with a populated framebuffer and front visible `TGNaviDlg`
  HWND `0x00020080`, but still has no iNavi display/controller/render
  milestones. Next work should trace the post-startup GWE/GDI/resource path
  after `TGNaviDlg` creation and visible-window settling, not file I/O or raw
  wait hot loops.
- Continue extending scheduler-backed waits beyond the raw timer-wake slice.
  `MsgWaitForMultipleObjectsEx` raw dispatch now waits through a due CE timer
  inside the requested timeout, and the Unicorn raw-import bridge can complete
  current-thread timer wakes/timeouts that fit the host run budget without
  registering a blocked waiter. The broader scheduler/thread/window goal still
  needs the full Unicorn blocked-wait path to own waits that cannot complete
  inside the active run budget, plus object, send-message, serial/audio/
  process, and message-input resumes consistently. Keep using `C:\WINCE600`
  scheduler/GWE sources as authority.
- Host input while parked on raw `GetMessageW` is no longer a blind spot:
  the host run loop now drains newly polled input into the blocked CE
  thread/window and relies on the scheduler message-wake path to resume the
  syscall. Keep using this for manual `--desktop host` interaction checks, but
  do not treat it as the post-splash progression fix. The mounted release
  virtual probe still parks at `COREDLL.dll@861 blocked_get_message`; the next
  real UI slice remains scheduler/GWE/resource progression after that valid
  idle point.
- Host Win32 presenter is now a usable output boundary for mounted probes:
  live guest framebuffer writes present during long Unicorn runs, `WM_PAINT`
  redraws the last frame, the visible client area is 800x480, and the title
  includes the launched host image path. Closing the host window intentionally
  terminates the emulator process immediately, so close-driven manual runs do
  not guarantee final tracefile/framebuffer artifact writes. Continue using
  `--desktop host` for visual checks when human feedback is useful, and
  `--desktop virtual` for deterministic artifact probes. If NVIDIA Image
  Scaling is added, do it as a real host-presenter scaler mode backed by the
  NIS SDK/shader or a clearly sourced compatible implementation; do not label
  ordinary bilinear/stretch presentation as NIS.
- Continue from the latest region/worker-stack mounted frontier in
  `target\thread_stack_region_virtual_150s_*`. Complex GDI regions now remain
  rect-list backed through `CombineRgn(RGN_DIFF)`, point/rect queries,
  clipping, `SetWindowRgn`, and `GetWindowRgn`; `SetWindowRgn` also respects
  the redraw flag. The first follow-up run moved past the old post-splash
  idle point and exposed a worker-thread stack write below the mapped stack.
  The stack reserve is now 4 MiB with 128 KiB worker slots, and the full
  150 s virtual/tap probe no longer crashes. This is current real progress:
  the app creates/resumes 10 threads, opens 883 host files, performs 79,768
  bounded host reads for only about 5.2 MiB, reaches `BitBlt=103`,
  `Polygon=1023`, `Polyline=415`, `CreateDIBSection=385`, first audio,
  Winsock, and serial/COM imports, and reads many
  `SDMMC Disk\mapdata\point\...` files. The framebuffer is still the real
  iNavi SE splash/art frame, but render traces now show later map/UI drawing
  into memory DCs. The next UI slice should trace the generic presentation
  path from those memory-DC map/UI surfaces to a display HDC or a visibility/
  paint state that should trigger that copy. Do not force pixels, resurrect
  hidden children, or special-case iNavi state.
- Continue from the process-clean mounted frontier in
  `target\process_lifetime_virtual_150s_*`. The current generic child-launch
  path now resolves all three iNavi companion process launches through the CE
  mount table and runs them to exit code `0`; `happyway_win.exe` no longer
  fails DLL layout on `AuthLibrary.dll`, and its top-level HWND is marked
  `dead=true` after child exit instead of remaining a live parent-dispatched
  WNDPROC. The process trace selector is `tracefile processes PATH`. The run
  still ends at `COREDLL.dll@861 blocked_get_message` with stable file/RSS
  counters and a real splash framebuffer. The next UI slice should compare the
  post-child window/message/render state against the previous hidden-strip
  frontier: the active hidden pending-update child is now `0x00020070`
  (`rect=0,426-800,480`, `update=0,0-800,54`) after extra child work, while
  the child-owned `happyway_win` top-level `0x0002000c` is dead and absent from
  z-order. Do not resurrect child windows or force hidden paints; continue
  with CE process/window lifetime and GWE presentation semantics.
- File mapping single-view aliasing is no longer the active current-gap
  suspect. v3 now stores per-mapping `FileMappingView` records, maps distinct
  bases, flushes guest bytes into shared backing, refreshes sibling views on
  flush, and removes/releases views on `UnmapViewOfFile`. The mounted
  `target\mapping_views_virtual_150s_*` probe proves this changed real iNavi
  state (`virtual_live=2/131072B` after the app's `UnmapViewOfFile`) while
  preserving stable file/RSS counters, but it did not advance the visible UI.
  Remaining mapping work is broader CE fidelity: immediate cross-view write
  coherence without `FlushViewOfFile`, page-protection/access validation,
  richer file-backed lifetime/flush semantics, and a dedicated
  `MappingSystem` manager as the catch-up plan grows.
- Continue from the cleaner tap/input frontier in
  `target\touch_focus_virtual_150s_*`. New top-level windows are now placed at
  the front of z-order, so the full-screen popup HWND `0x00020008` receives the
  mounted tap instead of older top-level HWND `0x00020004`; remote mouse-down
  now also produces the normal focus/activation transition before
  `WM_LBUTTONDOWN`. The run still parks at `COREDLL.dll@861
  blocked_get_message`, memory/file-I/O remains stable, and the known hidden
  child `0x0002006c` still owns the later pending `800x54` update while hidden.
  Do not treat the old tap-to-`0x20004` path as progress. The next slice should
  trace the generic GWE/GDI/resource path that should either show that child or
  copy the guest-composed offscreen surface to a display HDC.
- Continue from the new mounted iNavi first-present frontier. The latest
  virtual probes `target\update_erase_virtual_*`,
  `target\timer_cap_startup_tap_virtual_20s_*`,
  `target\unicorn_realtime_timer_virtual_30s_*`, and
  `target\hide_update_clear_virtual_20s_*`, with follow-up timer-scope probe
  `target\timer_scope_virtual_30s_*` and TimerProc bridge probe
  `target\timer_callback_virtual_30s_*`, plus the latest direct-UpdateWindow
  effective-visibility probe
  `target\update_effective_visibility_virtual_150s_*` and the
  show/hide-only `SetWindowPos` payload probe
  `target\setwindowpos_showhide_virtual_150s_*`, prove guest GDI now presents
  a real 800x480 memory surface to a window HDC:
  `BitBlt(dst=0x02020008, dst_memdc=false, dst_hwnd=0x00020008,
  src=0x000a0044, src_memdc=true)`. The framebuffer dump is fully populated
  (`575800` nonzero pixels in the latest run) and
  `target\update_erase_virtual.png` shows the real iNavi SE splash/art frame.
  The raw `GetMessageW` bridge now lets short <=100 ms GUI timers fire, and
  the initial guest thread now registers scheduler-owned `GetMessageW` waits
  instead of a diagnostic-only stop. Long future timers are no longer
  fast-forwarded in a tight loop, and they can now mature inside the same live
  Unicorn run when the host wall-clock budget allows it. The latest
  30 s virtual/tap probe delivered two real no-HWND `WM_TIMER` 1000 messages
  (`time_ms` about `21829` and `29329`) before parking on the next 7.5 s
  period outside the run budget with `sched=wait:3/0/3`, `wake=2`, and
  `msgcand=2`. The latest GWE
  cleanup also clears stale create-time update state when MFC immediately
  hides visible zero-size `AfxWnd42u` children; hidden controls in
  `target\hide_update_clear_virtual_20s_windows.txt` now mostly report
  `upd=false` rather than stale full-screen dirty rectangles. The remaining
  blocker is no longer first pixels, memory-DC-to-screen presentation,
  diagnostic-only timer wait ownership, or hidden-child stale paint state. It
  is now the real post-splash MFC/resource progression after valid timer
  wakes. Next steps: trace what the app does on the first two real
  no-HWND `WM_TIMER` deliveries, correlate that with the `resource_59718` /
  mode-47 table replay evidence, and decide whether the next fidelity slice is
  another GWE message ordering/detail gap, resource/module state behavior, or
  broader scheduler thread-state ownership. Do not force hidden child paints
  or app-specific state. The newer 150 s virtual/tap probe
  `target\writefile_lasterror_virtual_150s_*` confirms this as real UI
  progress: the framebuffer contains the iNavi SE splash art, not a black
  screen or fake host paint. The next presentation frontier is why later
  offscreen DIB/StretchBlt/BitBlt composition into an 800x54 memory surface is
  not copied to a display HDC, and why invalidation is landing on hidden or
  effectively invisible child HWND `0x0002006c`. Direct raw/kernel
  `UpdateWindow` now uses effective `IsWindowVisible` ancestry, so forcing
  `WM_PAINT` into that hidden child is closed as an invalid shortcut. Continue
  by tracing the generic path that should show or present the composed
  offscreen 800x54 surface: richer `WINDOWPOS`/`ShowWindow` state, MFC
  idle/message ordering, resource replay, or screen-HDC blit ownership.
  Basic show/hide/z-order-only `SetWindowPos`
  `WM_WINDOWPOSCHANGED` payload delivery is now covered and did not by itself
  move the frontier. The newest diagnostic trace
  `target\windowpos_trace_decode_virtual_150s_*` decodes queued
  `WM_WINDOWPOSCHANGED` payloads in-place; it confirms HWND `0x0002006c`
  received a normal `WINDOWPOS` notification for `rect=0,0,800,480` with
  `flags=0` before the run later parks with that child hidden and holding the
  pending 800x54 update. Use this decoded message evidence for the next
  show/idle/presentation slice instead of adding opaque pointer-only traces.
  Direct `ShowWindow` visibility changes now queue `WM_SHOWWINDOW` and
  `WM_WINDOWPOSCHANGED` even when the target is already effectively invisible
  through a hidden parent; the mounted
  `target\showwindow_direct_visibility_virtual_150s_*` probe confirms the
  app-visible hide messages now carry flags `0x97`, including for
  `0x0002006c`, but this did not move the final frontier. Continue with the
  remaining generic presentation question: why the guest-composed 800x54
  memory surface under hidden child `0x0002006c` is never shown or copied to a
  display HDC. Changed `QS_PAINT` now follows effective visibility instead of
  raw hidden invalidation. The follow-up
  `target\hidden_paint_qs_virtual_150s_*` probe stayed at the same frontier but
  exposed a stronger CE gap: hidden geometry changes were delivering immediate
  `WM_MOVE`/`WM_SIZE` despite CE `window.hpp` documenting pending size/move
  delivery until `ShowWindow`. v3 now defers those move/size messages for
  direct-hidden windows; `target\hidden_sizemove_virtual_150s_*` and
  `target\filtered_paint_visibility_virtual_150s_*` both keep the same
  `COREDLL.dll@861 blocked_get_message` frontier while reducing message input
  signals from `227` to `174`. Continue from the cleaner state by tracing why
  the guest-composed 800x54 offscreen surface is never shown or copied to a
  display HDC through normal GWE/GDI paths.
- Decide the safe host-write policy for mounted external dumps. The refreshed
  `target\createfile_access_virtual_150s_files.txt` proves iNavi opens
  `SDMMC Disk\iNaviData\config.bin` as `GENERIC_WRITE` + `OPEN_EXISTING`, but
  the run still reports zero write bytes and leaves the source hash unchanged.
  Since focused fixtures prove writable host-backed handles write through, the
  remaining issue is likely host/sandbox permission downgrade for the external
  dump. Prefer an overlay/copy-on-write mount strategy before allowing mounted
  iNavi probes to mutate `D:\INAVI_Emulator\INAVI`.
- Timer identity no longer has the known global-id collapse: v3 now keys
  timers by owner thread/message queue, optional `HWND`, and id, and raw
  `KillTimer(hwnd,id)` only removes the matching scoped timer. Destroyed HWND
  subtrees also clean up their window timers without deleting no-HWND thread
  timers. The first TimerProc bridge now propagates `TIMERPROC` through
  `MSG.lParam` and enters the guest callback from the raw `DispatchMessageW`
  path. Remaining timer work should focus on CE internal callback-timer bypass
  semantics, timer/message ordering, and how timer lifecycle/order interacts
  with the post-splash MFC resource replay, not duplicate numeric id handling
  or destroyed-window timer leaks.
- Continue the mounted iNavi resource-ready investigation from the
  `resource_59718`/mode-47 table frontier. Current evidence says
  `\SDMMC Disk\INavi\res\values.dat` opens and reads correctly, but by the
  time `resource_59718` calls the guest table loader the shared table at
  `0x0079c440` is already populated (`buffer=0x3006d970`,
  `tree_count=215`), so the guest one-shot loader at `0x0006bd18` returns
  `0` and the readiness chain fails. Next steps: preserve or capture the
  earlier mode-47 table load/subcheck history without adding app-specific
  success forcing; trace the `resource_ready` subfunctions before
  `resource_59718` (`59430`, `594f8`, `596b4`) and the mode-source table
  records; then decide whether the real missing piece is a generic message/
  timer lifecycle issue causing a repeated readiness pass, a resource state
  reset/cleanup path not reached because of earlier CE behavior, or a data/
  registry path selection problem.
- Keep the storage root/mount inheritance behavior covered while continuing
  fidelity ports: `[root].host_root` is the default backing root, missing or
  non-directory root values fall back to `"."`, and per-mount `host_root`
  values override the root. Add overlap diagnostics later if real mounted
  traces show ambiguous reverse host-path mapping.
- Treat the latest `explorer.exe` host-presented probe as a launch-fidelity
  checkpoint, not UI success. It now reaches the emulator sentinel instead of
  missing COREDLL ordinals after adding `StringCchCatW`, `wcsncmp`, and
  `DestroyIcon`. The post-send-wait rerun still reaches the same sentinel with
  no render milestones or framebuffer pixels; its optional
  `LoadLibraryW("aygshell.dll")` fails because that DLL is absent from the
  dumped runtime tree, not because the configured search path missed it.
  Validate whether the sentinel is a clean process return or a too-early
  control-flow exit before using explorer as a broader shell fixture.

## CE Fidelity Ledger

- Scheduler/waits/thread contexts:
  - Source refs:
    `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\schedule.c`,
    `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\syncobj.c`,
    `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\thread.c`, CE SDK/OAK wait
    and thread headers, and `SOURCE_REFERENCES.md`.
  - v2 corroboration: v2 proved cross-thread wait/send/audio/serial parking was
    a viable emulator path, but v3 should keep CE source as the behavior
    authority.
  - Current v3 status: scheduler ownership has begun with a `Scheduler`
    subsystem that records single/multiple/msg wait attempts, wait outcomes,
    blocked waits, resumed waits, max handle count, and max timeout in monitor
    summaries. Parked Unicorn `WaitForSingleObject` calls now carry their
    start tick and timeout and can resume with `WAIT_TIMEOUT` when the bounded
    wait expires; object-signaled resumes still acquire/consume the waited
    object first. When multiple parked waits are ready, resume selection now
    follows CE priority ordering from `DoWaitForObjects`: lower numeric thread
    priority wins, with FIFO order for equal priority and no immediate
    self-resume of the active thread. Existing CE6
    `WaitForMultipleObjects(TRUE)` rejection is preserved from
    `NKWaitForMultipleObjects`. Wait validation now follows CE
    `LockWaitableObject` more closely: thread, process, event, mutex, and
    semaphore handles are waitable; file, find-file, device, HWND, waveOut,
    file-mapping, and critical-section handles fail waits instead of being
    treated as signaled. `WaitForMultipleObjects(FALSE)` now validates all
    handles and the `MAXIMUM_WAIT_OBJECTS == 64` limit before acquiring any
    ready object, so invalid later handles do not consume earlier auto-reset
    events. Thread priorities are stored internally as CE absolute priorities
    (`0..255`); Win32 `Get/SetThreadPriority` maps the `0..7` band to/from
    absolute `248..255`, while `CeGet/CeSetThreadPriority` use absolute
    values.
    Suspend/resume count handling now follows CE `MAX_SUSPEND_COUNT == 127`:
    `SuspendThread` returns the previous count, refuses overflow with
    `ERROR_SIGNAL_REFUSED`, and `ResumeThread` decrements only nonzero counts
    while valid zero-count resumes return `0`.
    Unicorn parked waits now also cover the first
    `WaitForMultipleObjects(FALSE)` bridge: the blocked record owns the full
    handle list, wakes when any handle becomes ready, and returns
    `WAIT_OBJECT_0 + index` through the raw import boundary. The first
    `MsgWaitForMultipleObjectsEx` Unicorn bridge also parks raw imports after
    handle validation and GWE queue-input checks, then resumes with either the
    ready handle index, the message-input pseudo-index, or timeout. CE current
    process/thread pseudo handles are now modeled from `kfuncs.h`
    `SYS_HANDLE_BASE`/`SH_CURTHREAD`/`SH_CURPROC`: raw thread/process ID,
    exit-code, thread-times, terminate-process, and wait paths accept the
    pseudo current handles where CE does, and Unicorn initializes/refreshes the
    KData current thread/process ID slots during guest thread, wait, and
    `SendMessageW` context switches. Raw current-thread pseudo handle mutation
    now also covers `SetThreadPriority`, `CeSetThreadPriority`,
    `SuspendThread`, and `ResumeThread`; worker-thread objects are updated by
    current thread id, while main-thread priority/suspend metadata is kept in
    kernel state because v3 does not yet have a normal handle object for the
    initial thread. Mutex ownership now tracks CE recursive lock counts:
    initial-owner mutexes start owned with count `1`, owner waits recurse up to
    `MUTEX_MAXLOCKCNT == 0x7fff`, releases unwind one count at a time, and raw
    `ReleaseMutex` reports `ERROR_NOT_OWNER` for unowned/wrong-owner mutexes
    while still using `ERROR_INVALID_HANDLE` for non-mutex handles.
    The first scheduler-owned blocked-wait registry is also in place: parked
    Unicorn single/multiple/msg waits register a wait id, waited handles, kind,
    timeout, and FIFO sequence in `Scheduler`, with per-handle waiter queues
    and scheduler-side ready selection. The first object-transition wake path
    now feeds those queues: successful `SetEvent`, `ReleaseSemaphore`, and
    final recursive `ReleaseMutex` enqueue registered wait ids as pending wake
    candidates, and resume selection prefers those candidates while still
    rechecking/acquiring the real object state in the wait path. Thread and
    process handle exit transitions now use the same pending-wake path when
    guest threads exit, child process launches complete, or raw
    `TerminateProcess` marks a real process handle or the CE current-process
    pseudo handle signaled. Message input has the first matching queue:
    parked `MsgWaitForMultipleObjectsEx` waits register in a per-thread
    message-wait queue, and posted/thread/broadcast/quit/sent messages, remote
    input, and queued `WM_TIMER` posts enqueue those waits as pending
    candidates while GWE still owns queue status consumption. Unicorn still
    stores the saved MIPS context payload locally beside that wait id. Serial
    reads now have the first matching scheduler hook as a device-wait slice:
    parked raw Unicorn `ReadFile` on an empty serial handle can register a
    `SerialRead` wait under the COM handle, remote NMEA/serial injection queues
    serial-read wake candidates, and the resumed path streams the completed
    read into the original guest buffer through `kernel.read_file_into`.
    Matching remote serial bytes are drained into the target COM session just
    before `ReadFile`/`ReadFileInto`, so the device-file path remains the data
    owner rather than a special app branch. Parked Unicorn `GetMessageW`
    calls now also register in the scheduler's message-wait queue with their
    HWND/min/max filters; GWE message transitions enqueue them as pending wake
    candidates, and resume rechecks immutable filtered queue readiness before
    consuming the message and restoring the guest context. The initial guest
    thread now participates in that path through CE's current-thread
    pseudo-handle when no worker thread handle exists, so an idle main-thread
    `GetMessageW` can advance to a due timer, post it to GWE, remove the
    scheduler waiter, and return the `MSG` through the saved syscall ABI rather
    than stopping with a diagnostic-only blocked snapshot. Raw no-HWND
    `SetTimer` now records the current thread as timer owner, following
    `GWE\INC\cmsgque.h` queue-owned timer entries, and expired no-HWND timers
    post `WM_TIMER` to that owner thread instead of being dropped. CE timer
    sleeps now advance virtual elapsed time rather than blocking host wall
    time. Bounded
    worker-thread `Sleep(ms)` calls now register timeout-only scheduler waits
    and resume with a zero return after timeout expiry, using the CE
    `NKSleep` bounded timeout shape (`ms + 1` below `0xfffffffe`);
    `SleepTillTick` now uses the same bridge with a one-tick timeout.
    `Sleep(0)` now records a scheduler yield and, in the current one-slot
    Unicorn context model, swaps to a saved peer context when one exists
    without pumping messages or waiting for a tick. `Sleep(INFINITE)` now
    records the current-thread suspend count in raw dispatch and self-suspends
    guest worker contexts with a saved CPU context that `ResumeThread` can
    restore once the suspend count reaches zero.
  - Open gaps: full serial semantics beyond the first empty-read wake bridge,
    audio wake ownership, internal CE callback timers that bypass normal
    queued `WM_TIMER`/`DispatchMessageW` delivery, bounded worker-thread sleeps, and main-thread
    timer-expiry `GetMessageW` resumes, bounded idle
    fast-forward policy for long periodic timer loops, full multi-thread run-queue ownership
    beyond the one-slot `Sleep(0)`/`Sleep(INFINITE)` worker-context swaps,
    pending PSL late-suspend, main-thread suspend blocking, long-sleep
    chunking, fuller child-process
    lifecycle scheduling beyond handle signaling, blocked
    thread priority/fairness across all wait kinds beyond
    the current Unicorn bridge, moving saved `GetMessageW`/wait MIPS contexts
    out of the Unicorn bridge into scheduler-owned thread state, richer wake
    reasons across serial/audio/process/GWE waits, priority
    inheritance/boosting around mutex/critical-section ownership, pending
    self-suspend/PSL late-suspend state, resume-to-run-queue wake ownership,
    and fuller Unicorn thread context switching still need the next scheduler
    port slices.
  - Fixture gates: keep existing wait/thread fixtures passing, including
    `tests/test_progs/163_mutex_recursive_ownership`,
    `tests/test_progs/164_object_transition_wake`,
    `tests/test_progs/165_thread_exit_wait_wake`, and
    `tests/test_progs/167_sleep_infinite_resume` when the eVC4 MIPSII fixture
    suite is enabled, then graduate pending scheduler fixtures for multiple
    waiters, `GetMessageW` blocking, `MsgWait*`, fuller serial parking,
    waveOut callback wakeups, child-process waits, and scheduler mini app.
  - Latest iNavi evidence: the old empty `GetMessageW @861`
    `blocked_get_message` frontier is cleared by thread-owned no-HWND timers.
    The latest mounted virtual probe wrote `target\thread_timer_virtual_*`,
    ran to the 120 s wall-clock limit at `pc=0x70028b7c`,
    `ra=0x6002537c`, stayed memory-stable
    (`heap_live=13697/13300954B`, `virtual_live=3/196608B`,
    `host_open=665`, `host_read=80132/4053923B`, `mem_open=3`,
    `max_read=685080`), and repeatedly delivered a thread `WM_TIMER`
    (`hwnd=0`, `wparam=1000`). It still had no useful screen presentation:
    milestones show memory-DC DIB/blit work and the framebuffer remains only
    the 401-pixel red tap marker. Next immediate investigation should identify
    the timer-id 1000 loop and the missing memory-DC-to-screen present path,
    with extra `SetTimer`/`KillTimer` and GDI destination detail if needed.
    The serial-read wake slice is currently covered by focused tests
    (`scheduler_queues_serial_read_waiters_by_handle` and
    `remote_serial_injection_queues_scheduler_serial_read_candidates`) rather
    than by mounted iNavi evidence. This scheduler/loader/thread work is
    foundational and should not be counted as UI success until the mounted run
    advances through guest GDI/render paths.

- Runtime DLL loading / shimmed libraries:
  - Source refs: `D:\INAVI_Emulator\DUMPPLZ\Windows` for target runtime DLL
    bytes, and the mounted executable directory for real app companion DLL
    bytes; CE/MFC/SDK trees only as behavior evidence.
  - Current v3 status: COREDLL remains emulator-provided. OLE remains a
    shimmed launch-surface library. WINSOCK dispatch now goes through
    `src/winsock.rs` and has a first direct-host TCP/UDP implementation for
    the mounted app's classic WINSOCK imports. `commctrl.dll` is no longer
    treated as emulator-provided; startup preloads it from the DLL search paths and
    registered mapped-module exports are available to module APIs. Import
    patching resolves loaded external exports before shim classification, so
    search-path `commctrl.dll` import slots now patch directly to mapped DLL
    exports rather than a common-controls trap. Startup now also preloads real
    sibling DLLs from the main executable directory, skipping emulator-provided
    modules and duplicate normalized module names. This currently covers the
    mounted app's real `AuthLibrary.dll`, `TpSysAuth.dll`, `mMbcAuth.dll`,
    `tpeg_if_dll.dll`, and `tw_tpeg_if_dll.dll` bytes without adding
    file-name-specific behavior. The PE parser now tolerates real CE
    mapped-image zero fill below `SizeOfImage`, so the dumped `commctrl.dll`
    can be inspected and mapped.
  - Open gaps: runtime `LoadLibraryW` is not yet a general on-demand DLL
    mapper for arbitrary non-preloaded DLLs; sibling preload is a launch bridge
    and should graduate to CE-like on-demand module mapping. WINSOCK currently
    uses direct host sockets, shares CE thread last-error storage for
    `WSAGetLastError`, and does not yet provide a virtual NIC, isolated
    10.0.0.x subnet, scheduler-backed blocking waits, or complete `hostent`
    lifetime cleanup. OLE behavior still needs subsystem-backed implementation
    only where fixtures or traces demand it.
  - Fixture gates: keep PE zero-fill tests and module-loader tests passing;
    add focused runtime `LoadLibraryW`/`GetProcAddress` fixtures before
    expanding on-demand DLL mapping.
  - Latest iNavi evidence: `target\inavi_trampoline_virtual_*` confirms the
    sibling DLL path loads the real companion DLLs, reaches `strcat @1063`
    through `AuthLibrary`, clears the old null auth-proc call, and runs to a
    30 s wall-clock stop after the external trampoline pool was moved away from
    CE virtual allocations. It remains memory-stable and reaches repeated
    RSImage `CreateDIBSection` work, but still does not produce render
    milestones or useful framebuffer output.

- Window/GWE subsystem:
  - Source refs:
    `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\cmsgque.h`,
    `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\window.hpp`,
    `C:\WINCE600\PRIVATE\WINCEOS\COREOS\INC\gweapiset1.hpp`,
    `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\dlgmgr.h`, CE SDK
    `winuser.h`, and MFC `wincore.cpp`/`thrdcore.cpp`/`wingdi.cpp`.
  - v2 corroboration: v2 had owner-thread queues, pending message transfers,
    `PendingUpdateWindow`, paint bounds, and host presenter refresh paths. Use
    that only as proof that those emulator paths were viable; CE source remains
    the behavior authority.
  - Current v3 status: raw class/HWND geometry, basic lifecycle messages,
    queue retrieval, guest WNDPROC callouts, subclass `CallWindowProcW`,
    paint/update state, `BeginPaint`/`EndPaint`, and basic `SendMessageW`/
    `DispatchMessageW` are present. `UpdateWindow` now preserves CE's
    erase-before-paint update shape: when `erase_pending` is set, raw guest
    callouts enter the guest WNDPROC with `WM_ERASEBKGND` first, clear only
    the erase bit on nonzero return, then continue to `WM_PAINT`; the
    kernel/default path mirrors the same ordering for non-guest/default
    WNDPROCs. Raw `RedrawWindow`
    now covers the first CE-backed paint slice: rectangle/region invalidation,
    invalidation unioning, `RDW_VALIDATE`, `RDW_ALLCHILDREN`, erase state, and
    `RDW_UPDATENOW` through the same synchronous paint path. Raw `GetUpdateRgn`
    now copies pending paint bounds into an existing HRGN and returns CE-style
    region status. Raw `GetWindowThreadProcessId` now returns stored HWND owner
    thread/process metadata from the GWE window table. Raw `IsChild` now uses
    recursive parent-chain checks over the virtual HWND tree. Raw
    `SendNotifyMessageW` now executes same-thread notifications synchronously
    but routes different-thread notifications through the receiver-side sent
    queue without blocking the sender.
    GWE now has a separate receiver-side sent-message queue; message retrieval
    prefers it over posted messages, marks `InSendMessage`, and reports
    `QS_SENDMESSAGE`/send source state. Sent messages now also carry
    `SendMsgEntry_t`-style transaction state: sender/receiver thread ids,
    flags, timeout metadata, active receiver stack, result-ready completion,
    timeout expiry, and receiver-terminated completion when a target is
    destroyed. Unicorn raw `SendMessageW`/`SendMessageTimeoutW` now enters a
    same-process different-thread guest WNDPROC in the receiver context by
    parking the sender MIPS registers/running-thread metadata in a scheduler
    `SendMessage` blocked wait, activating the GWE sent transaction on the
    receiver, and restoring the sender with the WNDPROC result after the
    callout returns or the scheduler observes a ready send reply.
    `GetQueueStatus` now tracks CE-style current and changed queue bits, and raw
    `MsgWaitForMultipleObjectsEx` now wakes on newly changed queue input unless
    `MWMO_INPUTAVAILABLE` requests wake-on-current queued input.
    `PostQuitMessage` now records queue-owned quit state instead of an ordinary
    posted `WM_QUIT`, so `GetMessageW`/`PeekMessageW` observe quit even through
    nonmatching HWND/message filters. Raw `GetMessageWNoWait` now reaches the
    same GWE queue retrieval path instead of default ordinal handling. GWE
    focus/activation cleanup now also covers disabled or hidden focused/active
    targets: `EnableWindow` disable transitions, `ShowWindow(SW_HIDE)`, and
    `SetWindowPos(SWP_HIDEWINDOW)` clear descendant focus and explicit active
    HWND state through queued `WM_KILLFOCUS`/`WM_ACTIVATE(WA_INACTIVE)`. Raw
    `GetMessagePos` and `GetMessageQueueReadyTimeStamp` now use per-queue and
    per-message metadata from the GWE model: mouse-message screen positions are
    preserved separately from client `lParam`, and ready timestamps update when
    posts, sends, or queue-owned quit state make a thread queue ready. Raw
    `SetDlgItemInt`/`GetDlgItemInt`, `SetDlgItemTextW`/`GetDlgItemTextW`,
    `GetDlgItem`, and `SendDlgItemMessageW` text-message forwarding now reach
    the dialog child-window text model instead of generic ordinal/message
    fallback. Raw `WindowFromPoint` and `ChildWindowFromPoint` now route
    through GWE visible/enabled HWND hit testing instead of generic ordinal
    fallback. Raw `GetDialogBaseUnits` and `MapDialogRect` now cover CE
    dialog-unit mapping, and raw
    `GetNextDlgTabItem`/`GetNextDlgGroupItem` now walk real dialog children
    using visible/enabled state plus `WS_TABSTOP`/`WS_GROUP` boundaries. Raw
    `IsDialogMessageW` now rejects unrelated HWNDs, dispatches dialog-owned
    messages, consults `WM_GETDLGCODE`, moves TAB focus through the same
    tab-order helper, uses `GetKeyState(VK_SHIFT)` for Shift+TAB reverse
    traversal, routes Escape as `IDCANCEL`, and routes Return through a focused
    pushbutton or the dialog's default pushbutton with `IDOK` fallback. GWE now
    reports button `DLGC_DEFPUSHBUTTON`/`DLGC_UNDEFPUSHBUTTON` state, answers
    `DM_GETDEFID`/`DM_SETDEFID` over child pushbutton styles, and has a first
    queued-key `GetKeyState`/`GetAsyncKeyState`/`GetAsyncShiftFlags` model.
    Raw `PostKeybdMessage` now posts hardware-sourced `WM_KEYDOWN`/`WM_KEYUP`
    and optional character-buffer `WM_CHAR` messages through the same queue,
    while raw `keybd_event` targets the explicit per-thread keyboard target
    before falling back to the focused/active keyboard window. Raw
    `SetKeyboardTarget`/`GetKeyboardTarget`/`GetForegroundKeyboardTarget` now
    expose that queue target state. The
    broader CE dialog/input manager remains incomplete: full `DLGC_WANT*` edge
    cases, nested modal loops, default-button repaint/state details, richer
    keyboard-layout/`KeybdVKeyToUnicode` behavior, toggle-key edge cases, and
    full receiver-context guest dialog proc execution still need expansion as
    fixtures/traces demand. Current fixture coverage includes passing
    `052_modeless_dialog_isdialogmessage` and
    `076_dialog_tab_enter_escape` plus `169_post_keybd_message` runs through
    the ignored eVC MIPSII
    `fixture_exes` harness.
    Raw/kernel `DestroyWindow` now records and sends `WM_DESTROY` before final
    GWE cleanup, and the default `WM_CLOSE` shortcut records the same destroy
    observation before deleting HWND state. `WM_NCDESTROY` is now tracked when
    actually delivered through raw `SendMessageW` or a Unicorn guest-WNDPROC
    return, matching CE MFC's source-backed fake-NC-destroy path and CE fake
    value `WM_APP - 1` instead of adding an OS-side synthetic send.
    Raw/kernel parent `DestroyWindow` now sends `WM_DESTROY` to descendants
    before the parent and before final GWE cleanup. Unicorn direct guest-WNDPROC
    `DestroyWindow` now chains guest descendant `WM_DESTROY` callbacks
    child-first before final root cleanup and keeps the subtree valid while it
    is in the CE `fBeingDestroyed` phase so reentrant destroy calls do not
    double-finalize HWND state.
    Unicorn `CreateWindowExW` guest-WNDPROC callouts now honor `WM_CREATE`
    failure returns by returning `NULL` and destroying the just-created virtual
    HWND when guest code returns `-1`. A mounted probe showed that
    unconditionally injecting `WM_NCCREATE` at this import boundary regresses
    CE/MFC startup, so that behavior is not part of the current runtime path.
    GWE now stores explicit active-window state separately from focus, and raw
    `SetFocus`, `SetActiveWindow`, `SetForegroundWindow`, activating
    `ShowWindow` commands, and `SetWindowPos` without `SWP_NOACTIVATE` queue
    CE/MFC-visible `WM_ACTIVATE`, `WM_SETFOCUS`, and `WM_KILLFOCUS`
    lifecycle messages. Raw `EnableWindow` now routes through the same kernel
    lifecycle boundary, keeps the previous-enabled return contract, queues
    `WM_CANCELMODE` before disabling and `WM_ENABLE` for real enabled-state
    transitions, and leaves unchanged-state calls message-free. Raw
    `BringWindowToTop` now reaches the virtual z-order model through the
    kernel lifecycle boundary, moves the target to `HWND_TOP`, and activates
    the top-level target. Raw `IsWindowEnabled`, dialog tab/group traversal,
    and HWND point hit-testing now use effective enabled state through the
    parent chain, while `EnableWindow` still mutates and reports only the
    target HWND's direct enabled state. Raw `IsWindowVisible` and point
    hit-testing likewise walk hidden ancestors, while show/hide APIs keep
    direct visibility synchronized with `WS_VISIBLE`. Raw
    `WM_WINDOWPOSCHANGED` messages now carry a stable guest `WINDOWPOS`
    payload instead of `lParam = 0`; `GetMessageW`/`PeekMessageW` materialize
    the SDK struct and `DispatchMessageW`/guest-WNDPROC return paths release
    the registered heap payload after consumption. Raw `SetParent` now routes
    through the kernel lifecycle boundary, rejects invalid handles and
    descendant-parent cycles, preserves previous-parent returns, relinks the
    HWND into the new parent's sibling z-order, and clears descendant focus/
    explicit activation if the new ancestry makes the subtree effectively
    hidden or disabled. Raw `CreateWindowExW` now distinguishes `WS_CHILD`
    parent semantics from top-level owner semantics: non-child windows keep
    screen-relative top-level geometry and report the supplied owner through
    `GetWindow(GW_OWNER)`, while child windows use parent-relative geometry and
    `GetParent`. Raw `GetUpdateRect`/`GetUpdateRgn` now honor `bErase=TRUE` by
    sending `WM_ERASEBKGND` with the HWND paint HDC through the same window send
    path and clearing only the pending erase bit, leaving the dirty update
    bounds for paint. Raw top-level `CreateWindowExW` now attaches the `hMenu`
    argument as HWND menu state while preserving the same argument as the child
    control id under `WS_CHILD`, and raw `SetMenu`/`GetMenu`/`DrawMenuBar` now
    reach that virtual window menu state without menu painting shortcuts.
    Raw CE menu item APIs now keep ordered virtual `HMENU` entries:
    `CreateMenu`, `CreatePopupMenu`, `AppendMenuW`, `InsertMenuW`,
    `RemoveMenu`/`DeleteMenu`, `GetSubMenu`, `GetMenuItemInfoW`, and
    `SetMenuItemInfoW` preserve command IDs, popup submenu handles, CE
    type/state flags, checkmark bitmap handles, item data, and wide text
    through `MENUITEMINFOW`. Raw `EnableMenuItem` and by-position
    `CheckMenuItem` now update the same ordered state and preserve CE previous
    state return values for MFC command UI updates. Raw
    `SetAssociatedMenu`/`GetAssociatedMenu` now reach the same virtual HWND
    menu association as `SetMenu`/`GetMenu`.
  - Open gaps: update regions are still represented as one bounding rectangle,
    so partial `ValidateRect`/`RedrawWindow(RDW_VALIDATE)` subtracts the
    representable remainder but keeps a conservative bounding rectangle for
    disjoint leftovers. Internal paint requests are represented as normal
    pending update state, and full child clipping/z-order invalidation remains
    for the later GWE/GDI pass. Menu item count/ID exports are not currently
    wired because the parsed runtime ordinal surface has not exposed them yet;
    popup tracking/display, menu command routing, accelerators, and menu
    painting remain open.
  - Port order:
    1. Paint/update correctness: keep `WM_PAINT` synthetic rather than posted,
       finish `UpdateWindow`/`RedrawWindow`/region invalidation semantics, and
       verify `BeginPaint`/`ValidateRect` cancellation behavior.
    2. Window creation/destruction lifecycle: complete create/show/size/move/
       activate/focus/enable/destroy ordering, `WM_NCCREATE`/`WM_CREATE`,
       `WM_DESTROY`/`WM_NCDESTROY`, parent/child invalidation, and z-order
       effects. `WM_DESTROY` is now sent and recorded before raw/kernel
       cleanup, delivered `WM_NCDESTROY` is now recorded, and raw/kernel
       parent destroy sends descendant `WM_DESTROY` before parent cleanup.
       Unicorn guest-WNDPROC destroy callouts now follow the same child-first
       chain before final root cleanup. The first focus/activation slice now
       covers explicit active-window state plus
       `WM_ACTIVATE`/`WM_SETFOCUS`/`WM_KILLFOCUS` queueing, and the first
       enable slice now covers `WM_CANCELMODE`/`WM_ENABLE` queueing plus CE
       previous-state returns. Raw `BringWindowToTop` now covers the first
       dedicated top-of-z-order activation path. Effective disabled-state
       checks now walk ancestors for `IsWindowEnabled`, dialog traversal, and
       hit-testing without sending child `WM_ENABLE`. `WM_WINDOWPOSCHANGED`
       now carries the CE SDK `WINDOWPOS` payload through guest memory.
       `SetParent` now covers previous-parent returns, invalid/cyclic parent
       rejection, new-parent z-order insertion, and focus/activation cleanup
       when reparenting under hidden/disabled ancestry.
       Raw `CreateWindowExW` now splits `hWndParent` into child parent versus
       top-level owner according to `WS_CHILD`, matching CE MFC
       `AfxGetParentOwner` usage. Unicorn create callouts now abort
       creation on guest `WM_CREATE == -1`; unconditional `WM_NCCREATE`
       injection is a rejected false lead for this target/runtime path.
       Top-level `hMenu` creation state and raw `SetMenu`/`GetMenu`/
       `DrawMenuBar` now cover the first CE/MFC frame-menu attachment path
       while child-window `hMenu` still behaves as the control id.
       Remaining lifecycle work includes exact create/z-order side effects
       such as owner/topmost rules, deeper activate/focus/enable edge cases
       such as top-level owner activation, disabled-focus transfer,
       no-activate show variants, richer hidden-window edge cases around
       owner/popups, and destroyed-target behavior under synchronous sends.
    3. Message queues and synchronous sends: replace same-thread-only shortcuts
       with scheduler-owned sent queues, sender blocking, receiver-context
       execution, `InSendMessage`, timeout, destroyed-target, and reentrant
       send behavior. `SendNotifyMessageW` has the first CE-backed no-wait
       split, and receiver-side sent-message retrieval/source/depth state now
       exists; cross-thread `SendNotifyMessageW` now uses that queue with
       `SMF_SENDER_NO_WAIT | SMF_NOTIFY_MESSAGE` metadata and clears receiver
       send depth after dispatch. Raw cross-thread `SendMessageW` now also
       queues a sender/receiver sent transaction instead of executing the
       receiver shortcut in the caller thread, while `DefWindowProcW` remains
       direct default processing. Raw `SendDlgItemMessageW` now follows the CE
       SDK wrapper shape by using the same queueing send path for normal
       messages after `GetDlgItem`. Sender-side transaction bookkeeping
       now exists for blocking sends, and raw receiver `DispatchMessageW`
       stores the WNDPROC result back into that transaction. Timeout expiry now
       marks queued timed sends result-ready and removes them from receiver
       retrieval; raw `SendMessageTimeout(..., timeout=0)` across threads now
       creates and expires that transaction immediately instead of running the
       receiver shortcut, while nonzero cross-thread timeout sends now queue a
       timeout-flagged receiver-side transaction instead of fabricating caller
       thread completion. Scheduler send-reply waiters are now keyed by
       sent-message id and wake when the sent transaction completes, times out,
       is receiver-terminated by target HWND destruction, or the receiver calls
       the internal `ReplyMessage` path before dispatch unwinds. The first Unicorn
       raw-send path now runs same-process cross-thread guest WNDPROCs in the
       receiver context and restores the sender result. `GetQueueStatus`
       changed-bit tracking and
       `MsgWaitForMultipleObjectsEx` `MWMO_INPUTAVAILABLE` semantics now cover
       the first CE queue-status slice, and the raw/Unicorn bridges now ignore
       desktop-only flag bit `0x0001` instead of treating CE msg-waits as
       unsupported wait-all requests. `PostQuitMessage` now uses
       `msgqfGotWMQuitMessage`-style queue state and ignores caller filters
       when delivering `WM_QUIT`. Raw `GetMessageWNoWait` now covers the
       nonblocking get-message API path. Raw `GetMessagePos` and
       `GetMessageQueueReadyTimeStamp` now cover the first CE
       `PostedMsgQueueEntry_t.time`/`MousePosAtPost` and queue
       `m_ReadyTimeStamp` metadata slice. Remaining work: parking/resume
       across longer waits, reentrant cross-thread scheduling, nested modal
       loop unwinding, a public raw `ReplyMessage` boundary if a real target
       import/export path exposes it, richer queue-source/filter precision, and
       complete destroyed-target behavior remain open.
    4. Window data/class/dialog/control surface: class atoms/extra bytes,
       `SetWindowLong`/`GetWindowLong`, owner thread/process queries, dialog
       procs/results, child/descendant relationship queries, child lookup,
       command routing, accelerator/menu state, and MFC attach/subclass paths.
       `SetDlgItemInt`/`GetDlgItemInt` and `SetDlgItemTextW`/
       `GetDlgItemTextW` now cover the first CE dialog item text paths, and
       `SendDlgItemMessageW` now forwards `WM_SETTEXT`, `WM_GETTEXT`, and
       `WM_GETTEXTLENGTH` through the same child-control message boundary.
       HWND menu attachment now covers top-level `CreateWindowExW` menus plus
       `SetMenu`/`GetMenu`/`DrawMenuBar` plus GWE
       `SetAssociatedMenu`/`GetAssociatedMenu`. Ordered menu item state and
       `MENUITEMINFOW` round-tripping now cover create/popup/append/insert/
       remove/submenu/get/set info, plus enable/disable/check command-state
       updates by position. Popup display/tracking, command routing, menu
       painting, and any confirmed exported menu count/ID accessors remain
       open.
       `GetDialogBaseUnits`/`MapDialogRect` and
       `GetNextDlgTabItem`/`GetNextDlgGroupItem` cover the first CE-backed
       dialog layout/navigation slice. Fuller dialog default-proc,
       modal-loop, command-routing, default button, and keyboard traversal
       behavior remain open.
    5. Input/focus/capture/hit testing: keyboard char translation, mouse
       capture, coordinate mapping, modal blockers, active/foreground window
       semantics, and queue-status/source bits. `WindowFromPoint` and
       `ChildWindowFromPoint` now cover the first CE raw HWND hit-test entry
       points; richer disabled-window, transparent/static-control, capture,
       modal, and z-order edge cases remain open.
    6. GDI/DC integration: tie HWND update regions to HDC clipping, memory DCs,
       DIB/palette/text/region drawing, and framebuffer presentation without
       host-window shortcuts.
  - Fixture gates: prioritize existing window fixtures around paint/update,
    create/destroy order, cross-thread sends, dialogs, MFC lifecycle, menus,
    accelerators, hit testing, region clipping, and full UI stress.
  - Latest iNavi evidence: the app now reaches real first-frame UI
    presentation through guest GDI. The fresh
    `target\update_erase_virtual_*` virtual probe records a real
    memory-DC-to-window-HDC transfer
    (`BitBlt(dst=0x02020008, dst_memdc=false, dst_hwnd=0x00020008,
    src=0x000a0044, src_memdc=true, 800x480)`) and a fully populated
    framebuffer (`384000` nonzero pixels). `target\update_erase_virtual.png`
    shows the iNavi SE splash/art frame. The remaining window work should now
    trace post-splash queue/timer/idle progression and hidden/visible update
    semantics rather than faking pixels. The bounded
    destroy-lifecycle probe wrote
    `target\destroy_window_lifecycle_*` artifacts, reached RSImage DIB
    creation by the 10 s wall stop, but still reported no render milestones and
    an all-zero framebuffer body. The bounded `WM_NCDESTROY` lifecycle probe
    wrote `target\nc_destroy_lifecycle_*` artifacts and likewise stopped at the
    10 s resource/DIB frontier with no render milestones and an all-zero
    framebuffer body. The bounded child-destroy probe wrote
    `target\child_destroy_lifecycle_*` artifacts with the same no-render,
    all-zero framebuffer result. The bounded guest-destroy-chain probe wrote
    `target\guest_destroy_chain_*` artifacts, stopped at `pc=0x600c9aec` with
    `host_read=4226/500100B` and `heap_live=5620/2459096B`, and still had no
    render milestones or framebuffer pixels. The bounded sent-queue probe wrote
    `target\sent_queue_*` artifacts, stopped at `pc=0x00b4bc1c` with
    `host_read=4221/495853B` and `heap_live=5948/2767663B`, and likewise had
    no render milestones or framebuffer pixels. The bounded
    send-notify-sent-queue probe wrote `target\send_notify_sent_queue_*`
    artifacts, stopped at `pc=0x00339d8c` with `host_read=4221/499832B` and
    `heap_live=5948/2767663B`, and still had no render milestones or
    framebuffer pixels. The bounded sync-send-transaction probe wrote
    `target\sync_send_transaction_*` artifacts, stopped at `pc=0x00b4bc24`
    with `host_read=4221/495853B` and `heap_live=5948/2767663B`, and likewise
    had no render milestones or framebuffer pixels. The bounded
    send-timeout-expiry probe wrote `target\send_timeout_expiry_*` artifacts,
    stopped at `pc=0x00339c3c` with `host_read=4219/486039B` and
    `heap_live=5948/2767663B`, and still had no render milestones or
    framebuffer pixels. The bounded receiver-context-send probe wrote
    `target\receiver_context_send_*` artifacts, stopped at `pc=0x00b4bc24`
    with `host_read=4221/495853B` and `heap_live=5948/2767663B`, reached
    real resource/DIB work, but still had no render milestones and an all-zero
    framebuffer body. The bounded queue-status/MsgWait probe wrote
    `target\queue_status_msgwait_*` artifacts, stopped at `pc=0x00339d84`
    with `host_read=4221/495853B` and `heap_live=5948/2767663B`, and likewise
    had no render milestones or framebuffer pixels. The bounded
    post-quit-queue-state probe wrote `target\post_quit_queue_state_*`
    artifacts, stopped at `pc=0x00339da0` with `host_read=4221/495853B` and
    `heap_live=5948/2767663B`, and still had no render milestones or
    framebuffer pixels. The bounded GetMessageWNoWait probe wrote
    `target\get_message_nowait_*` artifacts, stopped at `pc=0x00339d88` with
    `host_read=4221/499832B` and `heap_live=5948/2767663B`, and again had no
    render milestones or framebuffer pixels. The bounded message-metadata
    probe wrote `target\message_metadata_*` artifacts, stopped at
    `pc=0x00895bfc` with `host_read=4225/486559B` and
    `heap_live=5621/2459146B`, reached `mapinfo.bin`/`iNaviData` file
    activity, but still had no render milestones and only one nonzero
    framebuffer byte, so it is not UI progress. The bounded dialog-int probe
    wrote `target\dialog_int_*` artifacts, stopped at `pc=0x00b4bc44` with
    `host_read=4221/495853B` and `heap_live=5948/2767663B`, reached
    RSImage/DIB resource work, and still had no render milestones or
    framebuffer pixels. The bounded hit-test probe wrote
    `target\window_from_point_*` artifacts, stopped at `pc=0x6002278c` with
    `host_read=4225/486559B` and `heap_live=5624/2461398B`, reached later
    map/device file activity, but still had no render milestones and only one
    nonzero framebuffer byte. The bounded focus/activation probe wrote
    `target\focus_activation_*`, stopped at `pc=0x0030f7e0` with
    `heap_live=7295/21831892B` and `host_read=24819/1924419B`, and preserved
    the same sparse 301-pixel red framebuffer line with no named render
    milestone. The bounded enable-window probe wrote `target\enable_window_*`,
    stopped at `pc=0x00339d9c` with `heap_live=7294/21830764B` and
    `host_read=24620/1918582B`, and again preserved the 301-pixel red
    framebuffer line with no named render milestone. The bounded
    bring-window-top probe wrote `target\bring_window_top_*`, stopped at
    `pc=0x0030f7c8` with `heap_live=7293/21820764B` and
    `host_read=24620/1922561B`, and likewise preserved the 301-pixel red
    framebuffer line with no named render milestone. A virtual-desktop rerun
    wrote `target\virtual_after_bring_window_top_*`, stopped at
    `pc=0x00343750` with `heap_live=7200/21843325B` and
    `host_read=26122/1952147B`, preserved the same 301-pixel red line, and did
    not show a host presenter window. Prefer `--desktop virtual` for future
    bounded mounted probes unless host presentation/input behavior is the
    thing being tested. The disabled-ancestor enabled-state probe wrote
    `target\disabled_ancestor_virtual_*`, stopped at `pc=0x00339d90` with
    `heap_live=7304/21886404B` and `host_read=25878/1940731B`, preserved the
    same 301-pixel red line, and still had no render milestones. The
    visibility/enabled effective-state probe wrote
    `target\visibility_enabled_virtual_final_*`, stopped at `pc=0x00344780`
    with `heap_live=7305/21887532B` and `host_read=26160/1961105B`, again kept
    the same 301-pixel red line, and still had no render milestones. The
    `WM_WINDOWPOSCHANGED` payload probe wrote `target\windowpos_virtual_*` and
    `target\windowpos_virtual_60s_*`; the 60 s virtual run avoided the host
    presenter window, reached RSImage `CreateDIBSection` work, stopped at
    `pc=0x00073684` with `heap_live=6929/21276879B` and
    `host_read=7839/1759291B`, and produced only a 101-pixel red line with no
    render milestone. The follow-up focus/activation cleanup rerun wrote
    `target\focus_activation_virtual_60s_*`, also in virtual desktop mode,
    stopped at `pc=0x002036fc` with `heap_live=7089/21301763B`,
    `host_read=7852/1765593B`, and the familiar 301-pixel red line from
    `(0,160)` through `(300,160)`, again with no render milestone. The
    `SetParent` lifecycle rerun wrote `target\set_parent_virtual_60s_*`,
    stopped at `pc=0x000be6e4` with `heap_live=6921/21255717B`,
    `host_read=4302/1718377B`, and only a 101-pixel red line from `(0,160)` to
    `(100,160)`, also with no render milestone. The owner/child raw-create
    rerun wrote `target\owner_child_virtual_60s_*`, stopped at
    `pc=0x002a252c` with `heap_live=6940/21278707B`,
    `host_read=7840/1760751B`, and the same 101-pixel red line with no render
    milestone. The fresh `GetUpdateRect`/`GetUpdateRgn` erase-query probe used
    `D:\INAVI_Emulator\DUMPPLZ\Windows` as the DLL source and wrote
    `target\get_update_erase_virtual_60s_*`; it stopped at `pc=0x00a436e0`
    with `heap_live=6930/21294161B`, `virtual_live=2/131072B`,
    `host_open=92`, `host_read=4305/1769298B`, `mem_open=2`,
    `max_read=497178`, no render milestones, and the same 101-pixel red line.
    The dialog/control text-forwarding probe used the same dumped DLL source
    and wrote `target\dialog_text_virtual_60s_*`; it stopped at
    `pc=0x0001362c` with `heap_live=7041/21284917B`,
    `virtual_live=3/196608B`, `host_open=113`,
    `host_read=7843/1763759B`, `mem_open=2`, `max_read=497178`, no render
    milestones, and the same 101-pixel red line. The create-failure contract
    probe wrote `target\create_abort_virtual_60s_*`; it stopped at
    `pc=0x001e5408` with `heap_live=6926/21256719B`, `host_open=91`,
    `host_read=4304/1732170B`, `mem_open=2`, `max_read=497178`, no render
    milestones, and the same 101-pixel red line. An earlier experimental
    `WM_NCCREATE` injection probe wrote `target\nc_create_virtual_60s_*` and
    regressed to an immediate empty-queue stop (`pc=0x7fff0b60`,
    `heap_live=24/12914B`, `host_read=0/0B`), so do not count that path as
    progress. The HWND menu-attachment probe wrote
    `target\menu_attach_virtual_60s_*` using
    `D:\INAVI_Emulator\DUMPPLZ\Windows`; it stopped at `pc=0x004d8ba8` with
    `heap_live=6917/21255371B`, `host_open=91`,
    `host_read=4302/1718377B`, `mem_open=2`, `max_read=497178`, no render
    milestones, and the same 101-pixel red line.
    Treat this as fidelity evidence and a possible performance/lifecycle
    frontier, not useful UI progress.

## Immediate

- GDI/resources/display ledger:
  - Source refs:
    `C:\WINCE600\PUBLIC\COMMON\SDK\INC\wingdi.h`, CE GDI/GPE headers,
    `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\window.hpp`, and MFC
    `wingdi.cpp`/dialog/resource source references in `SOURCE_REFERENCES.md`.
  - v2 corroboration: v2's framebuffer and resource paths prove that generic
    guest GDI output can feed host presentation, but v3 keeps the framebuffer
    as a CE display surface and does not synthesize app-specific pixels.
  - Current v3 status: raw `CreateDIBSection`, compatible DCs, object
    selection, simple blits, DIB source blits, regions, paint DCs, resources,
    and menu/dialog scaffolding are present. The latest slice adds bitmap
    RGBQUAD color-table state plus raw `SetDIBColorTable`/`GetDIBColorTable`;
    8 bpp blits now resolve palette indices before writing RGB565 pixels.
    Direct `BITMAPINFO` sources now parse embedded color tables for 1/4/8 bpp
    `DIB_RGB_COLORS`, covering indexed `StretchDIBits`/`SetDIBitsToDevice`
    source pixels. DCs now seed CE stock/default selected objects from
    `wingdi.h` (`SYSTEM_FONT`, `WHITE_BRUSH`, `BLACK_PEN`,
    `DEFAULT_PALETTE`, plus a restorable default bitmap), so the normal
    `old = SelectObject(...); SelectObject(..., old)` path no longer gets a
    zero previous object on newly created compatible DCs. Raw text/font query
    coverage now stores selected LOGFONTW fields, reports deterministic
    CE-shaped `TEXTMETRICW`, `GetTextExtentExPointW` fit/dx/SIZE data,
    `GetTextFaceW`, `GetTextAlign`, and `GetTextColor` without host-font
    dependencies.
  - Open gaps: complete palette/brush/pen/text/font/menu/dialog/icon/cursor
    behavior as trace evidence demands, including actual text/glyph drawing,
    richer font enumeration/fallback and character-width behavior; broaden
    indexed DIB coverage beyond the currently reached RGBQUAD/RGBTRIPLE table
    shapes; and connect later blit/update transitions to real paint
    invalidation without app-specific shortcuts.
  - Fixture gates: keep raw GWE/GDI tests passing, then add focused fixtures
    for additional indexed palette edge cases, selected-object lifetime edge
    cases, region clipping, glyph/text drawing, menu/accelerator, and MFC
    mini-app paint.
  - Latest iNavi evidence: mounted traces now reach paint, many DIBSections,
    and the first real screen/window-HDC presentation. `target\update_erase_virtual_*`
    records `BitBlt(dst=0x02020008, dst_memdc=false, dst_hwnd=0x00020008,
    src=0x000a0044, src_memdc=true, 800x480)`, and the framebuffer dump is
    fully populated. The display work should therefore target post-present
    clipping/invalid-region behavior, text/font completeness, and sustaining
    later UI updates, not the already-cleared first memory-DC-to-screen
    transfer and not forced presentation. Earlier
    `target\dib_colors_fresh_*` evidence confirms the app's 8 bpp RSImage
    DIBSections now have parsed color tables (`colors=256` on the 800-wide
    surfaces and populated partial tables on later resources), so indexed
    palette ingestion is no longer the leading suspect. The follow-up
    `target\text_metrics_virtual_60s_*` probe after the CE text/font query
    slice preserved the same populated framebuffer and RSImage/DIB loop with
    no named render milestone, so basic text metrics are also no longer the
    leading suspect.

- Continue from the latest stable host-mode UI frontier. Current
  `--desktop host --tap 400,240` evidence no longer has the multi-GB RAM spike
  and produces real but sparse framebuffer pixels through guest `Polyline`
  (`target\startup_flamegraph_after_heap_chunk.ppm`: 401 red pixels from
  `(0,160)..(400,160)` in the profiled run). Flamegraph-driven startup fixes
  removed per-import COREDLL export-table rebuilding, replaced hot trampoline
  scans with lookup maps/page sets, indexed mapped code by page for the global
  Unicorn hook, generation-gated kernel memory mapping, and now map heap
  spillover in 1 MiB chunks instead of one page at a time. A current 60 s
  host/tap run reaches `pc=0x00b55150`, `ra=0x0030f384`, `ReadFile=33759`,
  and `CreateDIBSection=190`; the admin flamegraph runs farther and hits the
  next real guest/UI fault at `pc=0x0026f7e4` (`render_map_pointer_deref`),
  `addr=0x0000005c`, with `ReadFile=61825` and `CreateDIBSection=317`. Next
  File I/O is no longer the bulk-RAM bottleneck: existing host files stay
  host-backed even when opened writable, `ReadFile` streams from the stored
  handle with bounded small-read caching, and existing host files requested
  read/write now fall back to read-only live host handles when Windows denies
  write access. The latest virtual probe,
  `target\file_rw_fallback_virtual_60s_*`, eliminates the previous
  `Access is denied` failures for `SDMMC Disk\mapdata\SearchDB\*.db` and
  advances to `pc=0x003426d0`, `ra=0x002fd5e8`, with
  `host_open=235` and `host_read=38930/2229372B`, but still no render
  milestones and only red tap pixels. Next work should debug the null/invalid
  render-map object path around
  `0x0026f7c0..0x0026f7e4` using real guest state and existing probes. Also
  keep the new trampoline/virtual-allocation layout covered: the external
  Unicorn trampoline pool now starts at `0x70000000` instead of colliding with
  the CE virtual-allocation base `0x50000000`, and
  `target\inavi_trampoline_virtual_*` verifies the previous
  `WRITE_PROT addr=0x50000000` stop no longer reproduces. Do not fake-present
  DIBSections just because their bits are populated.
- Keep the new direct-DIB framebuffer path honest. `StretchDIBits` and
  `SetDIBitsToDevice` now draw `SRCCOPY` `DIB_RGB_COLORS` BITMAPINFO data in
  focused tests, but real iNavi traces have not reached those ordinals yet.
  `TransparentImage` now handles the reached memory-DC composition shape. Extend
  only as real traces demand: likely next GDI work includes palette tables for
  8-bpp DIBs, broader ROPs, screen presentation blits, and text/shape paths if
  they appear in import counts.
- Continue from the post-PNG singleton/already-running exit frontier. Current
  mounted trace evidence gets through RSImage stream reads and PNG decode, then
  returns at `0x0030f384` and exits through the app singleton routine at
  `0x0001199c`: `CreateMutexW(L"iNavi")` returns
  `ERROR_ALREADY_EXISTS`, `FindWindowW(title=L"iNavi")` finds hwnd
  `0x00020000`, `SetForegroundWindow` succeeds, and the app terminates via
  `0x0048fa90`. Do not mask this by weakening CE `CreateMutexW` semantics.
  Trace why `0x0001199c` is reached after an existing `iNavi` window/mutex is
  already present, including caller `0x00011d28`, startup/CRT sequencing around
  `0x0048f8c0`/`0x0048f920`, and whether the second invocation is real app
  re-entry, constructor ordering, or handle/window lifecycle state.
- Use `tracefile milestones PATH` for long monitor runs that need durable
  resource/window/singleton import context. The normal last-import ring can be
  flooded by `memset`/CRT activity during PNG/resource loops; the milestone ring
  now keeps `CreateMutexW`, `ReleaseMutex`, `FindWindowW`,
  `CreateWindowExW`, resource, and string-format milestones with decoded
  argument details.
- Continue the launch path after the first synthetic `WM_PAINT` dispatch by
  expanding CE-referenced GDI/surface drawing and blit behavior beyond the
  first solid `FillRect` framebuffer path, then verify those pixels through the
  generic presenter boundary. Do not treat the timeout-running paint loop as
  GUI success.
- Continue the iNavi render-surface path with targeted diagnostics around the
  resize/surface allocation gate, not app-state forcing. Confirmed host/tap
  evidence: `WM_SIZE` passes `800x480` to render object `0x3006b360`, but it
  calls vtable slot `+0xf0` target `0x0011ce60`; the path never reaches resize
  slot `+0xf4` target `0x001033e4` or
  `render_surface_create_call`/`render_surface_store` at
  `0x00104904`/`0x00104910`. `WM_PAINT` later calls render entry
  `0x0010518c`, which returns immediately because `render_surface=0` and
  `render_enabled=0`. Next evidence should identify which lifecycle branch
  should call `0x001033e4` and which real CE/window/resource input causes it to
  be skipped.
- Continue from the post-resource-read display frontier. The old
  `\res\values.dat` lookup is fixed: real mounted monitor evidence now shows
  successful reads from `\SDMMC Disk\INavi\res\values.dat`, and the previous
  `0x00058a84` readiness failure is no longer hit. The 90 s bounded run still
  produces an all-zero framebuffer and wall-stops in SDK MFC code, so the next
  work is to trace the real render lifecycle/GDI/surface path after
  `values.dat` is loaded.
- Continue from the new real mounted monitor frontier after raw MIPS/CRT math
  dispatch. A `tap 400 240` + `until 0x00058a04 180000 0` run now clears the
  previous `__litodp @2036`, `__dpmul @2027`, and `sqrt @1060` import traps and
  reaches an idle `GetMessageW @861` `blocked_get_message` snapshot instead.
  Capture framebuffer/render evidence from this idle state and keep probing
  the message/paint/GDI path; do not call this GUI success until nonzero app
  pixels are produced.
- Continue the post-time iNavi path from the new wall-clock diagnostic frontier.
  The latest mounted run now gets past the earlier export-index
  `GetPaletteEntries` trap via real palette/DC state, preserves SDK CRT
  ordinals such as `memset @1047` and `swprintf @1097` before export-index
  fallback, returns heap-backed `RegisterGesture @2724` state, and writes
  `GetSystemTime @25`. With sampled Unicorn code tracing and mapped-code
  instruction reads, a 90,000 ms mounted no-tap run now returns in roughly 27 s
  at an idle `GetMessageW @861` `blocked_get_message` snapshot instead of
  timing out in app-side date/geometry logic. The visible top-level
  `wce_solution_inavi` HWND is `800x480`, and the `Afx:10000:b:0:40000006:0`
  child HWND exists, but the framebuffer dump is still all zero. Use the idle
  frontier to keep probing WNDPROC/paint/GDI behavior before expecting guest
  drawing.
- Continue from the new post-jump-table exit frontier. The latest release
  mounted run gets past `__nes @2047`, `__litofp @2032`, `__ll_div @2005`,
  `GetTimeZoneInformation @27`, `SetForegroundWindow @702`,
  `InputDebugCharW @595`, and the previous trampoline corruption of the iNavi
  halfword jump table at `0x000ebbf0`. The `ADBSetAccountProperties @1943`
  frontier now returns `FALSE`/`ERROR_NOT_SUPPORTED` and the app proceeds to an
  encoded `TerminateProcess` exit (`caller=0x0048fa90`, process `0x42`,
  `exit_code=0`). The framebuffer dump `target\inavi-release-adb1943.ppm` is
  still all zero. WNDPROC return trampoline-origin tracing now decodes the
  shutdown path as `0x56d0` entering `0x0004390c`, then an app-side `0x5236`
  send at `0x00043e30`/`0x00043e38`; the main `wce_solution_inavi` WNDPROC maps
  that to `WM_CLOSE`. Disassemble the branch path through `0x0004390c` and
  determine which preceding CE/MFC resource, window, or service result is
  causing the app to shut down before useful drawing.
- Replace launch-stub behavior for WINSOCK and OLE imports with real
  subsystem-backed implementations as import traces demand. WINSOCK already
  routes through `src/winsock.rs` and has a direct-host socket table; add any
  isolated host-network bridge, scheduler-backed blocking behavior, and richer
  lookup/option semantics there rather than growing import-dispatch glue. Keep
  MFC and `commctrl.dll` on the loaded DLL path only; do not add emulator MFC
  or common-controls stubs.
- Continue burning down COREDLL ordinals subsystem by subsystem, replacing
  stubbed ordinal plan entries with CE/MFC/SDK-referenced semantics. Next
  likely tranche: `BitBlt`, `PatBlt`, `StretchDIBits`, `SetDIBitsToDevice`,
  basic shape/text drawing, and memory-DC bitmap surfaces into or through the
  virtual framebuffer; PE-backed resource icon/bitmap loading beyond the
  string-resource path,
  COM/OLE API dispatch when ole32 imports are connected, more GWE menu/dialog/
  control raw pointer marshalling, broader file attributes/directory metadata,
  and timer/system-time structs.
- Continue tracing after CE `CreateWindowExW` now delivers the source-backed
  create-time `WM_CREATE` callout and CE `CallWindowProcW` enters guest
  window-procedure targets. The latest bounded snapshot still reaches SDK MFC
  default/idle handling and then an empty-queue `GetMessageW` diagnostic; the
  former ordinal-1036 `longjmp`/`pc=0` crash is no longer the current stop.
  Raw `GetWindow` sibling/child traversal is now connected for the observed
  MFC `GetWindow @251` calls. Virtual show/move/size lifecycle messages are
  queued for raw `ShowWindow`, `SetWindowPos`, `MoveWindow`, and visible
  top-level `CreateWindowExW`; the mounted bounded rerun confirmed
  `\SDMMC Disk\iNaviData` succeeds and creates `WCE_Solution_iNavi` plus an
  MFC child window. The latest rerun gets past the previous
  `GetPaletteEntries` trap, SDK CRT ordinal normalization bug, and
  `RegisterGesture @2724` pointer-return path, and `GetSystemTime @25`; the
  current wall-clock-bounded post-time run names the next frontier as repeated
  startup CRT/import activity before visible drawing. Continue replacing raw
  COREDLL/GDI/DC
  behavior with CE-referenced semantics that advance the path toward target
  framebuffer drawing.
- Use the new guest-WNDPROC return ring to compare creation-time sequencing
  against CE/MFC expectations. The latest diagnostic shows create/show/size/
  paint/idle messages returning `0`, `WM_PAINT` not reaching `BeginPaint`, MFC
  dispatch through `AfxWndProcBase` (`0x6004eba8`), and `Solution_iNavi`
  registered with target WNDPROC `0x000135cc`. A `--tap 400,240` idle-frontier
  run now confirms queued `WM_LBUTTONDOWN`/`WM_LBUTTONUP` delivery and drain
  through the active HWND, but still produces an all-zero framebuffer. After
  correcting Unicorn paint validation semantics, the app WNDPROC still routes
  top-level `WM_PAINT` to `DefWindowProcW` without `BeginPaint` or drawing
  imports. Continue with a targeted probe of the `0x000135cc` app
  WNDPROC/message-map branch, `SetWindowLongW`/superclass state, and
  first-message creation ordering before adding more lifecycle messages.
- Continue connecting SDK CE 4.2 Mipsii COREDLL CRT ordinals from `coredll.lib`
  as the launch trace demands.
- Add focused import-trap tests for Unicorn `_setjmp`/`longjmp` register/PC
  restoration once the fixture harness is wired to the existing
  `tests/test_progs/006_setjmp_longjmp` program.
- Implement CRT `_msize`/`realloc`/operator delete ordinals from SDK evidence so
  MFC/CRT heap paths do not rely only on Local/Heap reallocation aliases.
- Extend `cemath` as real guest imports demand more CRT/floating-point helpers.
- Extend subsystem smoke tests as each shim is connected to guest import traps.
- Add import-trap argument/result marshalling tests that exercise the new raw
  heap/file/find/message/resource ordinals through decoded guest MIPS
  registers.
- Continue PE resource directory integration beyond string tables so
  `FindResourceW`, `LoadResource`, and `SizeofResource` can consume mapped
  icon/bitmap/dialog/menu data rather than only test-registered virtual
  resources and PE-backed strings.
- Keep resource lookup evidence current. `FindResource(W)` for `RT_STRING` now
  falls back from an individual string id to its containing string block, which
  removed the observed `#3867` string-resource miss in a real host/tap run.
  The older `0x0e01` main-EXE RT_STRING miss remains explained by the EXE
  having no RT_STRING table; continue resource-module/MFC fallback
  investigation only if current traces demand it.
- When GWE/DC behavior is ready, adapt window state to the generic `Desktop`
  trait boundary without replacing CE/MFC message, class, or window semantics
  with host-window shortcuts.

## Next

- Extend `--monitor` from a bounded-run command loop into a persistent
  Unicorn debugger session. The current checkpoint/rewind commands restore the
  Rust-side CPU wrapper, CE kernel, and framebuffer, and current `map`/`x`/
  `disasm` commands inspect mapped static PE/DLL/trap bytes. The current
  `until ADDRESS` command can stop a bounded run at a requested guest PC.
  `step` is deliberately disabled because the current Unicorn wrapper rebuilds
  CPU/RAM state for each run and would otherwise restart from the image entry.
  Remaining pieces are to retain live Unicorn CPU/register/memory state across
  commands and expose live memory examine/write commands.
- Continue the new resource payload frontier after the registry-backed
  `SPI_GETOEMINFO` fix. The latest mounted run selects iNavi resource
  `mode=47`, reads the 1746-byte record payload from
  `\SDMMC Disk\INavi\res\values.dat`, parses the first key, then stops with
  interrupt 20 at `pc=0`, `ra=0x0006bfb4`, `last_pc=0x0006bf8c` on the second
  key. Use `target\monitor_debugger_oeminfo_render.txt` and
  `target\monitor_debugger_oeminfo_code.txt` as the current compact evidence.
- Extend bounded run tooling beyond the current snapshot import ring if more
  structured trace context is needed.
- Trace why the now-consumed `--tap 400,240` messages do not trigger useful
  paint, child-window, or custom-message drawing behavior. The next useful
  evidence is the exact WNDPROC/superclass path and any GDI/DC/resource imports
  following the delivered mouse down/up.
- Continue the scheduler/GWE wake path from the release/no-trace mounted
  frontier. `target\release_virtual_150s.ppm` and the stop summary show preload
  reaches a normal `GetMessageW` block
  (`blocked_get_message=thread:1 hwnd=any`) rather than a file-I/O or PNG
  loader stall. Next fixes should make timers, posted input, host touch/key
  events, serial/audio/process wakes, and message waits resume through the same
  CE scheduler model without forcing paints or app state.
- Continue from the post-delay-slot-fix host Win32 frontier. The scheduler now
  skips MIPS delay-slot PCs during timeslice switching, and
  `target\host_delayfix_180s_*` no longer reproduces the post-map
  `READ_UNMAPPED`/`ra=0` ANR. The visible final frame is populated map UI with
  an app modal warning that GPS initialization detected abnormal behavior and
  recommends restart (`Error Code: -14`). Next work should trace real
  GPS/serial/system-state inputs and the app's reset-warning path through CE
  device/registry semantics, while continuing to validate that map rendering
  and GWE sends stay live under host Win32.
- Continue the route-search/companion-process path from
  `target\route_drive_fast_locale1_*`. `vsprintf @1146` and
  `IsValidLocale @209` are fixed, and the real search modal can be opened
  after dismissing the bottom safety/notice bar. The active bug is now generic
  process/window scheduling: after `CreateProcessChildActivated` for
  `happyway_win.exe`, remote hit-test sends touches to thread 3
  (`hwnd=0x0002000c`) while the host framebuffer still shows the parent iNavi
  modal. Investigate CE process/window lifetime and foreground ownership
  before forcing input to parent windows. Also disassemble/trace the new
  happyway guest-code stop at
  `pc=0x0003eac4(image:happyway_win.exe+0x2eac4)`,
  `ra=0x0003ea0c`.
- Continue from `target\route_drive_heapmap1_*`: the former
  `happyway_win.exe+0x2eac4` heap-tail `WRITE_UNMAPPED` is fixed, and the
  mounted host route-search path now reaches the real child iNavi SE splash.
  The next work is to determine why that child splash does not advance into
  its next route-search UI on the visible host run. Use a settled tap replay or
  monitor-driven run so the trace captures the activated child state rather
  than the parent pre-handoff timing seen in
  `target\route_drive_heapmap_limit1_*`.
- Continue from `target\route_ready_wait2_*`: the patched host/live scheduler
  reaches `iSearch.exe` without the duplicate `happyway_win.exe` regression,
  but the visible route controls are still hidden after the 170s mounted run.
  Remote evidence shows `visibleChildCount=0`, `hiddenChildCount=28`, and the
  last imports are still dominated by `resmapi_800x480.bin`, `CreateDIBSection`,
  `RSImage LoadPNG`, and `MoveWindow` for the hidden chrome. Next work should
  measure and fix the generic startup/resource hot path or the CE
  show/update/paint transition that keeps these HWNDs hidden; do not force
  button visibility or route-search state.
- Extend the new Rust remote server beyond the now v2-aligned REST/WebSocket
  surface: expose recent log lines from `CeRemote`, add mounted iNavi
  validation for `/api/v1/control/ws` and `/api/v1/audio/ws`, and keep proving
  that remote touch/GPS input advances through the same scheduler/GWE paths as
  host input. REST handlers already match v2's touch/key validation, compact
  success responses, NMEA/location errors, frame `quality`, MJPEG
  `fps`/`quality`, and missing-framebuffer error shape. WebSocket control/audio
  upgrades now queue JSON control frames and stream server-backed PCM binary
  frames.
- Add ordinal/decorated-name evidence from the Windows CE 4.2 Mipsii SDK import
  libraries, alongside the source references already recorded.
- Persist host-backed registry writes separately from the source dump.
- Add real serial backend support for `win32_com` devices.
- Bridge selected virtual Win32/CE APIs to host Win32 APIs where that preserves
  real guest semantics.

## Later

- Keep host presentation/streaming of framebuffer snapshots wired through
  `Presenter` implementations as guest drawing paths start writing meaningful
  pixels.
- Add real low-latency host playback draining behind `HostAudioSink`; current
  waveOut work copies guest PCM into registered sinks and `main` registers the
  Windows `winmm` host-sink boundary, but the host backend still retains chunks
  instead of owning a full playback queue.
- Extend WINSOCK beyond the first direct-host implementation: isolate traffic
  behind a CE-like subnet/gateway model if needed, make blocking socket calls
  scheduler-backed instead of short host timeouts, and add focused fixtures for
  `bind`/`listen`/`accept`, UDP, `select`, and lookup edge cases.

## Parked

- App-specific fixes are parked unless backed by guest execution evidence.
