# TODO

## Current Slice

- Attachment fidelity queue merged into `PLAN.MD`: align raw/non-Unicorn
  `LoadLibraryW` with runtime behavior or explicit failure; keep
  emulator-provided core DLL classification audited; make must-implement stub
  hits actionable; finish `SHGetFileInfo`, `TrackPopupMenu*`, modal
  `MessageBoxW`, shell special-folder fallback policy/CSIDL coverage above the
  now-backed `fCreate`/`CSIDL_FLAG_CREATE` path, notification APIs, shell
  namespace/storage presentation, and file-change notifications.
- GWE/input fidelity queue from `PLAN.MD`: raw `TranslateMessage` now covers
  basic ASCII letters/digits with Shift/CapsLock and the `WM_SYSKEYDOWN` to
  `WM_SYSCHAR` path. Implement real keyboard-layout selection, broader
  accelerator/menu shortcut routing above the now-backed
  `TranslateAcceleratorW` modifier/`WM_SYSKEYDOWN` match path, dead-key/IME
  handoff, richer clipboard rendered ownership and delayed-render edge cases
  above the raw clipboard store and `GetClipboardDataAlloc` local-handle copy
  path, rendered caret blink/focus invalidation above the raw caret store, and
  focus/capture integration. Also complete
  cross-thread `SendMessageTimeout` waits until reply or timeout and writes the
  result pointer; cross-process `SendMessageTimeout` to parked children now
  also honors the caller's timeout rather than waiting forever (`INFINITE`).
  Korean 2-bul Hangul IME composition (hangul.rs) is now backed and tested;
  dead-key support for Western layouts (AZERTY 0x040C, QWERTZ 0x0407, Spanish
  0x040A/0x0C0A) is now implemented: `Gwe::dead_key` state machine posts
  `WM_DEADCHAR` on first press and `WM_CHAR(composed)` or
  `WM_CHAR(dead)+WM_CHAR(base)` on the follow-up key; Unicode precomposed-form
  table covers circumflex/grave/acute/tilde/diaeresis; `SMTO_BLOCK` reentrant
  dispatch while waiting remains open.
- Shell fidelity follow-up: `ShellExecuteEx` now handles the basic CE launch
  chain through shortcuts, registry associations, `CreateProcessW` queuing,
  `SEE_MASK_NOCLOSEPROCESS` hProcess output, and `nShow` propagation into child
  `WinMain`. `ShellExecuteEx.lpDirectory` and raw `CreateProcessW.pszCurDir`
  now propagate into pending child launches, relative child-EXE resolution, and
  parked-process identity restore. Registry association launch now covers the
  CE `"%1" %*` command-template shape and appends `lpParameters` only when no
  `%*` placeholder is present. Continue with the remaining shell APIs from the
  attachment:
  richer `SHGetFileInfo` icon extraction/image-list behavior, rich
  notification interaction callbacks above the now-stateful
  `SHNotification*I` data APIs and the now-backed `Shell_NotifyIconW` owner
  callback posting path. `SHNotification*I` can now post `WM_NOTIFY`/`NMSHN`
  payloads for stored notification events and marshal `SHNN_LINKSEL` link
  strings as `pszLink` data inside the receiver allocation, plus
  `SHNN_DISMISS` timeout flags; continue with automatic timeout/dismiss
  ownership, COM notification callbacks, and real taskbar/bubble UI. Also
  continue recent-doc pruning/UI display and PIDL
  input above the now-backed `SHAddToRecentDocs(SHARD_PATH/null)` recent
  shortcut/state path, file-change event delivery/PIDL payloads above the
  now-stateful `SHChangeNotifyRegisterI`/`SHFileNotifyRemoveI`/
  `SHFileNotifyFreeI` registration/free path, rendered popup/menu windows and
  real selection above the now-notifying `TrackPopupMenuEx` owner-message
  path, rendered modal `MessageBoxW` window/nested pump/button input above the
  now-recorded/default-result raw slice, keyboard translation, richer
  clipboard rendered ownership and delayed-render edge cases above the
  now-stateful raw clipboard store and allocation-copy path, and rendered
  caret blink/focus integration above the raw caret state. Basic
  `SHCreateShortcut`/`SHGetShortcutTarget` CE text-link create/read behavior is
  covered and still feeds `ShellExecuteEx`; `SHCreateShortcutEx` now covers
  bounded unique-name and output-buffer behavior. Precise shell errors,
  non-EXE document edge cases, and broader command-template variants remain
  queued.
  Notification records are now pruned during normal HWND/process teardown. Keep
  behavior registry-backed and generic; do not fake route UI.
- Winsock fidelity follow-up: guest-visible local names now use the isolated
  `10.0.0.1`/`10.0.0.2` model while host sockets remain the transport. Blocking
  `accept`, `recv`, `recvfrom`, and `select` park through the scheduler; blocking
  `connect` on TCP streams now parks via `TcpConnecting` background-thread state.
  Blocking UDP/unconnected path edge cases remain if discovered.
- Route-search child-startup blocker closed/watch: `target\route_deviceexit1_*`
  confirms the old MIPS CE `TerminateProcess` thunk in `DeviceParser.exe`
  decodes through the interrupt/zero-PC path and the child exits cleanly. The
  same probe reaches `happyway_win.exe`, `iSearch.exe`, and
  `gwe=send:1 done:1`, then returns to the established hidden `afxwnd42u`
  chrome state. Continue by chasing the later guest show/update sequence for
  those existing child controls; do not revisit `DeviceParser.exe` unless a
  fresh trace again records `CreateProcessChildError`.
- Route-search visibility frontier update: `target\route_showcmd2_messages.txt`
  proves the `afxwnd42u` route/chrome children are hidden by guest
  `ShowWindow(cmd=0/SW_HIDE)` immediately after creation; the bounded run does
  not yet reach a later `ShowWindow(cmd=5)` for those controls. Continue by
  driving or speeding the startup path until the guest should reveal the
  right/bottom chrome, then compare the missing show sequence against the
  successful bottom-strip/menu path. Do not force `WS_VISIBLE`: this trace says
  the preload hide is guest behavior.
- Route-search path next step after the sent-message ordering fix: continue
  from `target\host_presenter_sendorder1_*`. The old `happyway_win.exe`
  `SendMessageW(hwnd=0x0002000c,msg=0x0401,wParam=14)` startup trap is
  closed/watch: CE-style sent/nonqueued-before-posted retrieval now lets
  `PeekMessageW`/`GetMessageW` service synchronous sends before ordinary
  posted messages, and the fresh host run advanced from `gwe=send:1 done:0`
  to `gwe=send:1 done:1`. The current route-search frontier is again the
  later resource/hidden-chrome sequence: inspect `iSearch.exe` activation,
  `afxwnd42u` child-window creation/show, route dialog lifetime, and remaining
  sleep/timer waits. Do not force visibility or change route coordinates
  unless fresh traces show input routing regressed.
- Route-search scheduler follow-up after `target\host_presenter_sendfix1_*`:
  the later frozen-frame send gap is closed/watch. The fresh host run no longer
  sits at `gwe=send:13 done:12`; queued sent messages now make parked receiver
  processes and suspended receiver threads ready, and validation holds
  `gwe=send:1 done:1` while the framebuffer advances. The remaining route
  frontier is not that specific send deadlock; continue with the slow
  guest-side resource/window-show path and visible chrome timing.
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
  remote virtual runs schedule parked children like host runs. Continue with
  resource throughput/hidden-child show sequencing.
- Startup profiling follow-up: the per-page Unicorn heap-spillover map churn is
  fixed/watch. Keep `map_guest_range` span batching only on heap spillover and
  keep virtual allocations page-granular unless `reclaim_stale_virtual_memory_pages`
  is made span-aware. Next useful speed work should chase the remaining CE
  scheduler/display/app-flow frontier, `map_persisted_ram_blob_pages`, hook
  overhead, and process handoff behavior; do not reopen file preload/reopen or
  per-page heap mapping unless fresh flame evidence regresses those counters.
- Remote live-pump follow-up: continue mounted launches with
  `--remote-server 192.168.0.39:8765`; remote/host live runs now use the
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
  fixed/watch. Next useful speed work should profile the now-visible
  TCG/code-generation and hook paths
  (`uc_emu_start`, `tcg_cpu_exec`, `tb_find`, `tb_gen_code_mipsel`,
  import/code hooks) and only then consider longer-lived Unicorn instances.
  Do not reopen host-file preload/reopen or per-page persisted-RAM remap unless
  fresh flame evidence regresses those counters.
- Immediate stuck-screen next step: implement the child-process scheduler
  handoff properly instead of reviving the backed-out host-loop rotation
  experiment. Evidence in `target\stuck_process_processes.txt` shows
  `happyway_win.exe` parked as process 67/thread 3, while the parent later
  queues a cross-process `SendMessageW` to HWND `0x0002000c` and keeps working
  through resource/allocator code. The next implementation
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
  was viable. v3 external companions still do not share v2's window
  registry/message broker or CE named mapping state, so continue with the
  MappingSystem/process-handle manager work rather than app-specific shortcuts.
- Startup profiling follow-up: Windows-sudo flamegraphs confirm the mounted
  virtual startup path is no longer dominated by huge file preload, host-file
  reopen, import name lookup, or per-import trap/string/argument cloning.
  Continue from the final frontier at `COREDLL.dll@496` (`Sleep`) with bounded
  file counters and post-map scheduler state. Next useful speed work should
  measure/attack remaining generic costs: Unicorn code-hook/timeslice overhead,
  raw COREDLL dispatch frequency, `combine_rgn_raw`, streamed `read_file_into`,
  guest memcpy, and scheduler/device waits. Do not reintroduce app-specific fast
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
- Immediate live-host follow-up: no-wall Win32-host launches now drain remote
  REST controls and host input from the live Unicorn tick even when no
  `--cpu-wall-clock-limit-ms` is set. Continue from the post-map
  responsiveness and device/scheduler frontier: if a specific map control feels
  dead, run a fresh no-wall or bounded host trace with `messages`, deliver the
  exact tap, and inspect the guest handler/timer/device continuation after the
  delivered `WM_LBUTTONDOWN/UP`. Do not revisit black framebuffer, safety-screen
  REST delivery, or multi-GB file/RSS theories unless fresh evidence regresses
  them.
- Remote-server singleton follow-up: if a fresh no-wall run reaches the
  `CreateMutexW("iNavi")` → `ERROR_ALREADY_EXISTS` → `FindWindowW` → exit
  branch without a harness wall-stop, investigate mutex/window lifetime, CE
  visibility/top-level matching, and MFC startup re-entry first. Do not weaken
  named mutex semantics or hardcode iNavi controls.
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
  suspended slot. Continue converging the remaining wait/send/timer/device
  handoff paths onto the saved-context FIFO run-queue model so signaled waiters
  are not stranded behind a single suspended slot. The new
  `messages` trace selector now preserves kernel-level GWE post/target/delivery
  records. Next manual-host debugging should use the same trace on the exact
  control/area that feels unresponsive; if messages are delivered, continue
  into guest handler/device/timer behavior instead of adding app-specific hit
  targets.
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
- Current host/manual post-map slice: continue from
  `target\host_sleep_getmsg_180s_*`. The fixed-interval timeslice now retries
  a pending scheduler slice after unsafe MIPS branch/trampoline/import samples,
  and current `Sleep` now yields to a ready blocked `GetMessageW` waiter when
  main has queued posted/sent traffic. The latest wall snapshot is still a
  bounded-run ANR shape, not a crash:
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
  committed CE visible-client-region clipping (`8fa8c9f`). Continue
  investigating any remaining "ANR" report as guest post-map
  scheduler/device/app responsiveness, not as a rendering leak, host window
  pump freeze, or a basic input-drop bug.
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
  targets.
- Current GDI map-fidelity slice: continue from `target\gdi_exttext_virtual.*`.
  The huge black base-layer gap is now fixed generically by honoring
  `ExtTextOutW(ETO_OPAQUE)` as a CE GDI background-rectangle fill with the
  selected DC `bk_color`. The mounted framebuffer now has a real light
  land/background layer while preserving bounded memory/file I/O. ROP2 pen
  drawing is also modeled and tested, and the latest clip-region slice
  `target\gdi_clip_regions_virtual.*` now preserves `CombineRgn(RGN_DIFF)`
  holes for memory/display `FillRect`, `Polygon`, `Polyline`, `BitBlt`,
  `StretchBlt`, and `TransparentImage`; that closes a real generic CE clipping
  bug but the mounted frame still shows road/building styling problems.
  `DcState.poly_fill_mode` now defaults to `ALTERNATE` (even-odd) matching
  the CE `Polygon` default; the old winding-rule default was incorrect.
  Next GDI work should inspect concrete trace evidence for missing road/building
  primitives: line joins/caps, pen style, brush/palette/DIB
  color-table differences, ROP3/non-SRCCOPY blits, or any unimplemented CE GDI
  calls that appear in the mounted render/counts traces. Keep tracing generic
  GDI paths rather than guessing colors or special-casing iNavi pixels.
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
  run has only the COM serial read plus one main-thread sleep active. Next work
  should trace the post-startup GWE/GDI/resource path after `TGNaviDlg`
  creation and visible-window settling, not file I/O or raw wait hot loops.
- Continue from the process-clean mounted frontier in
  `target\process_lifetime_virtual_150s_*`. The current generic child-launch
  path now resolves all three iNavi companion process launches through the CE
  mount table and runs them to exit code `0`; `happyway_win.exe` no longer
  fails DLL layout on `AuthLibrary.dll`, and its top-level HWND is marked
  `dead=true` after child exit instead of remaining a live parent-dispatched
  WNDPROC. The run still ends at `COREDLL.dll@861 blocked_get_message` with
  stable file/RSS counters and a real splash framebuffer. The next UI slice
  should compare the post-child window/message/render state against the
  previous hidden-strip frontier: the active hidden pending-update child is now
  `0x00020070` (`rect=0,426-800,480`, `update=0,0-800,54`) after extra child
  work, while the child-owned `happyway_win` top-level `0x0002000c` is dead and
  absent from z-order. Do not resurrect child windows or force hidden paints;
  continue with CE process/window lifetime and GWE presentation semantics.
- File mapping single-view aliasing is no longer the active current-gap
  suspect. v3 now stores per-mapping `FileMappingView` records, maps distinct
  bases, flushes guest bytes into shared backing, refreshes sibling views on
  flush, and removes/releases views on `UnmapViewOfFile`. Remaining mapping
  work is broader CE fidelity: immediate cross-view write coherence without
  `FlushViewOfFile`, page-protection/access validation, richer file-backed
  lifetime/flush semantics, and a dedicated `MappingSystem` manager.
- Continue from the cleaner tap/input frontier in
  `target\touch_focus_virtual_150s_*`. New top-level windows are now placed at
  the front of z-order, so the full-screen popup HWND `0x00020008` receives the
  mounted tap instead of older top-level HWND `0x00020004`; remote mouse-down
  now also produces the normal focus/activation transition before
  `WM_LBUTTONDOWN`. Do not treat the old tap-to-`0x20004` path as progress. The
  next slice should trace the generic GWE/GDI/resource path that should either
  show that child or copy the guest-composed offscreen surface to a display HDC.
- Continue from the new mounted iNavi first-present frontier. Virtual probes
  `target\update_erase_virtual_*` through `target\setwindowpos_showhide_virtual_150s_*`
  prove guest GDI now presents a real 800x480 memory surface to a window HDC:
  `BitBlt(dst=0x02020008, dst_memdc=false, dst_hwnd=0x00020008,
  src=0x000a0044, src_memdc=true)`. The framebuffer dump is fully populated
  (`575800` nonzero pixels) and shows the real iNavi SE splash/art frame.
  The remaining blocker is the real post-splash MFC/resource progression after
  valid timer wakes. Continue by tracing what the app does on the first two real
  no-HWND `WM_TIMER` deliveries, correlate that with the `resource_59718` /
  mode-47 table replay evidence, and decide whether the next fidelity slice is
  another GWE message ordering/detail gap, resource/module state behavior, or
  broader scheduler thread-state ownership. Do not force hidden child paints
  or app-specific state.
  The next presentation frontier is why later offscreen DIB/StretchBlt/BitBlt
  composition into an 800x54 memory surface is not copied to a display HDC, and
  why invalidation is landing on hidden or effectively invisible child HWND
  `0x0002006c`. Changed `QS_PAINT` now follows effective visibility. Hidden
  geometry changes now defer `WM_MOVE`/`WM_SIZE` until `ShowWindow`. Continue
  from the cleaner state by tracing why the guest-composed 800x54 offscreen
  surface is never shown or copied to a display HDC through normal GWE/GDI paths.
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
    `WaitForMultipleObjects(FALSE)` bridge, the first `MsgWaitForMultipleObjectsEx`
    Unicorn bridge, CE current process/thread pseudo handles from `kfuncs.h`
    `SYS_HANDLE_BASE`/`SH_CURTHREAD`/`SH_CURPROC`, mutex recursive lock counts,
    and scheduler-owned blocked-wait registry with per-handle waiter queues.
    Object-transition wakes, message-input wait queues, and serial-read device
    waits are also covered. Parked Unicorn `GetMessageW` calls register in the
    scheduler's message-wait queue. Bounded worker-thread `Sleep(ms)` calls
    register timeout-only scheduler waits. `Sleep(0)` records a CE-style yield.
    `Sleep(INFINITE)` self-suspends guest worker contexts.
  - Open gaps: full serial semantics beyond the first empty-read wake bridge,
    audio wake ownership, internal CE callback timers that bypass normal
    queued `WM_TIMER`/`DispatchMessageW` delivery, bounded worker-thread sleeps,
    and main-thread timer-expiry `GetMessageW` resumes, bounded idle
    fast-forward policy for long periodic timer loops, full multi-thread
    run-queue ownership beyond the one-slot `Sleep(0)`/`Sleep(INFINITE)`
    worker-context swaps, pending PSL late-suspend, main-thread suspend
    blocking, long-sleep chunking, fuller child-process lifecycle scheduling
    beyond handle signaling, blocked thread priority/fairness across all wait
    kinds beyond the current Unicorn bridge, moving saved `GetMessageW`/wait
    MIPS contexts out of the Unicorn bridge into scheduler-owned thread state,
    richer wake reasons across serial/audio/process/GWE waits, priority
    inheritance/boosting around mutex/critical-section ownership, and fuller
    Unicorn thread context switching.
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
    The latest mounted virtual probe ran to the 120 s wall-clock limit,
    repeatedly delivered a thread `WM_TIMER` (`hwnd=0`, `wparam=1000`), stayed
    memory-stable, but still had no useful screen presentation. Next immediate
    investigation should identify the timer-id 1000 loop and the missing
    memory-DC-to-screen present path.

- Runtime DLL loading / shimmed libraries:
  - Source refs: `D:\INAVI_Emulator\DUMPPLZ\Windows` for target runtime DLL
    bytes, and the mounted executable directory for real app companion DLL
    bytes; CE/MFC/SDK trees only as behavior evidence.
  - Current v3 status: COREDLL remains emulator-provided. OLE remains a
    shimmed launch-surface library. WINSOCK dispatch now goes through
    `src/winsock.rs` and has a first direct-host TCP/UDP implementation.
    `commctrl.dll` is no longer treated as emulator-provided; startup preloads
    it from the DLL search paths. Import patching resolves loaded external
    exports before shim classification. Startup also preloads real sibling DLLs
    from the main executable directory. Runtime `LoadLibraryW/LoadLibraryExW`
    now synchronously maps dumped MIPS DLLs, relocates them, recursively loads
    non-emulator dependencies, patches imports, rewrites the live trap page,
    registers resources/exports, records dynamic module refcounts, parses TLS
    callback addresses, and invokes guest TLS callbacks and
    `DllMain(DLL_PROCESS_ATTACH)` before returning. Final dynamic `FreeLibrary`
    enters guest TLS callbacks and then `DllMain(DLL_PROCESS_DETACH)` before
    marking the module unload-pending. Runtime forwarder resolution and
    `LoadLibraryExW(DONT_RESOLVE_DLL_REFERENCES)` / `LOAD_LIBRARY_AS_DATAFILE`
    modes are also covered.
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
    sibling DLL path loads the real companion DLLs and runs to a 30 s
    wall-clock stop after the external trampoline pool was moved away from CE
    virtual allocations. It reaches repeated RSImage `CreateDIBSection` work but
    still does not produce render milestones or useful framebuffer output.

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
    paint/update state, `BeginPaint`/`EndPaint`, `SendMessageW`/
    `DispatchMessageW`, `UpdateWindow`, `RedrawWindow`, focus/activation, menus,
    dialog APIs, and keyboard input primitives are present. Receiver-side
    sent-message queues, scheduler-owned cross-thread `SendMessageW`, timer
    identity by thread/hwnd/id, `PostQuitMessage` queue state, `GetQueueStatus`
    changed-bit tracking, and `WM_WINDOWPOSCHANGED` SDK payloads are covered.
    CE-shaped `WM_DESTROY`/`WM_NCDESTROY` ordering, `SetParent` lifecycle,
    `CreateWindowExW` WS_CHILD vs owner split, menu item state/MENUITEMINFOW
    round-trip, dialog navigation helpers, and queued `WM_SHOWWINDOW` /
    `WM_WINDOWPOSCHANGED` for direct-hidden windows are covered.
  - Open gaps: update regions now use `Vec<Rect>` with proper subtraction so
    disjoint invalids survive partial `ValidateRect`; `GetUpdateRgn` now
    returns a `COMPLEXREGION` when multiple sub-rects remain.  Menu item
    count/ID exports are not currently wired;
    popup tracking/display, menu command routing, accelerators, and menu
    painting remain open. Fuller `DLGC_WANT*` edge cases, nested modal loops,
    default-button repaint/state details, richer keyboard-layout/
    `KeybdVKeyToUnicode` behavior, toggle-key edge cases, and full
    receiver-context guest dialog proc execution still need expansion.
    Exact create/z-order side effects such as owner/topmost rules, deeper
    activate/focus/enable edge cases, top-level owner activation, disabled-focus
    transfer, no-activate show variants, richer hidden-window edge cases, and
    destroyed-target behavior under synchronous sends remain open.
  - Port order:
    1. Paint/update correctness: keep `WM_PAINT` synthetic rather than posted,
       finish `UpdateWindow`/`RedrawWindow`/region invalidation semantics, and
       verify `BeginPaint`/`ValidateRect` cancellation behavior.
    2. Window creation/destruction lifecycle: complete create/show/size/move/
       activate/focus/enable/destroy ordering, `WM_NCCREATE`/`WM_CREATE`,
       `WM_DESTROY`/`WM_NCDESTROY`, parent/child invalidation, and z-order
       effects.
    3. Message queues and synchronous sends: parking/resume across longer waits,
       reentrant cross-thread scheduling, nested modal loop unwinding, a public
       raw `ReplyMessage` boundary if a real target import/export path exposes
       it, richer queue-source/filter precision, and complete destroyed-target
       behavior.
    4. Window data/class/dialog/control surface: popup display/tracking, command
       routing, accelerators, menu painting, and any confirmed exported menu
       count/ID accessors. Fuller dialog default-proc, modal-loop,
       command-routing, default button, and keyboard traversal behavior.
    5. Input/focus/capture/hit testing: richer disabled-window,
       transparent/static-control, capture, modal, and z-order edge cases for
       `WindowFromPoint`/`ChildWindowFromPoint`.
    6. GDI/DC integration: tie HWND update regions to HDC clipping, memory DCs,
       DIB/palette/text/region drawing, and framebuffer presentation without
       host-window shortcuts.
  - Fixture gates: prioritize existing window fixtures around paint/update,
    create/destroy order, cross-thread sends, dialogs, MFC lifecycle, menus,
    accelerators, hit testing, region clipping, and full UI stress.
  - Latest iNavi evidence: the app now reaches real first-frame UI presentation
    through guest GDI. `target\update_erase_virtual_*` records a real
    memory-DC-to-window-HDC transfer and a fully populated framebuffer showing
    the iNavi SE splash/art frame. The remaining window work should now trace
    post-splash queue/timer/idle progression and hidden/visible update semantics.
    The presentation frontier: why the guest-composed 800x54 offscreen surface
    under hidden child `0x0002006c` is never shown or copied to a display HDC
    through normal GWE/GDI paths.

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
    `DEFAULT_PALETTE`, plus a restorable default bitmap). Raw text/font query
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
    and the first real screen/window-HDC presentation. The display work should
    target post-present clipping/invalid-region behavior, text/font
    completeness, and sustaining later UI updates.

- Continue from the latest stable host-mode UI frontier. Current
  `--desktop host --tap 400,240` evidence produces real but sparse framebuffer
  pixels through guest `Polyline`. Flamegraph-driven startup fixes removed
  per-import COREDLL export-table rebuilding, replaced hot trampoline scans
  with lookup maps/page sets, indexed mapped code by page for the global
  Unicorn hook, generation-gated kernel memory mapping, and now map heap
  spillover in 1 MiB chunks instead of one page at a time. A current 60 s
  host/tap run reaches `pc=0x00b55150`, `ra=0x0030f384`, `ReadFile=33759`,
  and `CreateDIBSection=190`; the admin flamegraph runs farther and hits the
  next real guest/UI fault at `pc=0x0026f7e4` (`render_map_pointer_deref`),
  `addr=0x0000005c`. File I/O is no longer the bulk-RAM bottleneck. The latest
  virtual probe, `target\file_rw_fallback_virtual_60s_*`, advances to
  `pc=0x003426d0`, `ra=0x002fd5e8`, with `host_open=235` and
  `host_read=38930/2229372B`, but still no render milestones. Next work should
  debug the null/invalid render-map object path around
  `0x0026f7c0..0x0026f7e4` using real guest state and existing probes. Also
  keep the new trampoline/virtual-allocation layout covered: the external
  Unicorn trampoline pool now starts at `0x70000000` instead of colliding with
  the CE virtual-allocation base `0x50000000`. Do not fake-present DIBSections
  just because their bits are populated.
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
  at an idle `GetMessageW @861` `blocked_get_message` snapshot. The visible
  top-level `wce_solution_inavi` HWND is `800x480`, and the `Afx:10000:b:0:40000006:0`
  child HWND exists, but the framebuffer dump is still all zero. Use the idle
  frontier to keep probing WNDPROC/paint/GDI behavior before expecting guest
  drawing.
- Continue from the new post-jump-table exit frontier. The latest release
  mounted run gets past `__nes @2047`, `__litofp @2032`, `__ll_div @2005`,
  `GetTimeZoneInformation @27`, `SetForegroundWindow @702`,
  `InputDebugCharW @595`, and the previous trampoline corruption of the iNavi
  halfword jump table at `0x000ebbf0`. The `ADBSetAccountProperties @1943`
  frontier now returns `FALSE`/`ERROR_NOT_SUPPORTED` and the app proceeds to an
  encoded `TerminateProcess` exit. The framebuffer dump is still all zero.
  WNDPROC return trampoline-origin tracing decodes the shutdown path as `0x56d0`
  entering `0x0004390c`, then an app-side `0x5236` send at
  `0x00043e30`/`0x00043e38`; the main `wce_solution_inavi` WNDPROC maps that to
  `WM_CLOSE`. Disassemble the branch path through `0x0004390c` and determine
  which preceding CE/MFC resource, window, or service result is causing the app
  to shut down before useful drawing.
- Replace launch-stub behavior for WINSOCK and OLE imports with real
  subsystem-backed implementations as import traces demand. WINSOCK already
  routes through `src/winsock.rs` and has a direct-host socket table; add any
  isolated host-network bridge, scheduler-backed blocking behavior, and richer
  lookup/option semantics there rather than growing import-dispatch glue. Keep
  MFC and `commctrl.dll` on the loaded DLL path only; do not add emulator MFC
  or common-controls stubs.
- Continue burning down COREDLL ordinals subsystem by subsystem, replacing
  stubbed ordinal plan entries with CE/MFC/SDK-referenced semantics. All
  COREDLL_EXPORTS table entries are now dispatched. Next tranche: `BitBlt`,
  `PatBlt`, `StretchDIBits`, `SetDIBitsToDevice`, basic shape/text drawing, and
  memory-DC bitmap surfaces into or through the virtual framebuffer; PE-backed
  resource icon/bitmap loading beyond the string-resource path, COM/OLE API
  dispatch when ole32 imports are connected, more GWE menu/dialog/control raw
  pointer marshalling, broader file attributes/directory metadata, and
  timer/system-time structs.
- Continue tracing after CE `CreateWindowExW` now delivers the source-backed
  create-time `WM_CREATE` callout and CE `CallWindowProcW` enters guest
  window-procedure targets. The latest bounded snapshot still reaches SDK MFC
  default/idle handling and then an empty-queue `GetMessageW` diagnostic; the
  former ordinal-1036 `longjmp`/`pc=0` crash is no longer the current stop.
  Raw `GetWindow` sibling/child traversal is now connected for the observed
  MFC `GetWindow @251` calls. The latest rerun gets past the previous
  `GetPaletteEntries` trap, SDK CRT ordinal normalization bug, and
  `RegisterGesture @2724` pointer-return path, and `GetSystemTime @25`; the
  current wall-clock-bounded post-time run names the next frontier as repeated
  startup CRT/import activity before visible drawing. Continue replacing raw
  COREDLL/GDI/DC behavior with CE-referenced semantics that advance the path
  toward target framebuffer drawing.
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
  surface with external-client/watch validation as the app frontier advances.
  Keep proving remote touch/GPS input through the same scheduler/GWE paths as
  host input, and capture mounted audio binary PCM once the real app emits wave
  data during a live remote run.
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
