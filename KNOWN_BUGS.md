# KNOWN_BUGS

## Artifact Note

- `target\` was cleared on 2026-06-04 after generated build/probe output grew
  to roughly 50 GB. Historical artifact names in this file document observed
  evidence but may no longer exist locally; rerun the relevant probe to
  regenerate fresh files.
- Future mounted validation should use `D:\INAVI_Emulator\DUMPPLZ\Windows` as
  the true runtime DLL source. Older notes that mention SDK MFC `--dll-search-dir`
  are historical evidence labels.

## Open

- Post-region mounted iNavi now runs deeper, but later map/UI composition is
  still not presented to the display surface.
  - Symptom: `target\thread_stack_region_virtual_150s_*` runs the full 150 s
    virtual/tap budget without the old blocked-message stall or the temporary
    worker-stack crash. It still ends with the visible framebuffer on the real
    iNavi SE splash/art frame, while later render trace entries show guest
    map/UI drawing into memory DCs.
  - Evidence: the same run stays memory/file-I/O bounded
    (`heap_live=14200/31768040B`, `virtual_live=2/131072B`,
    `host_open=883`, `host_read=79768/5231945B`, `mem_open=4`,
    `max_read=685080`) and reaches much more real subsystem activity:
    `CreateThread=10`, `ResumeThread=10`, `WaitForMultipleObjects=10`,
    `BitBlt=103`, `Polygon=1023`, `Polyline=415`,
    `CreateDIBSection=385`, plus first audio, Winsock, and serial/COM import
    activity. The files trace confirms many
    `SDMMC Disk\mapdata\point\...` map/icon records are read. The remaining
    bug is therefore a generic presentation/message/visibility/scheduler gap
    after offscreen map/UI composition, not blank framebuffer startup,
    complex-region flattening, worker-thread stack layout, or large-file RSS.
  - Status: open current UI frontier. Trace how the later guest-composed
    memory-DC surfaces should reach a display HDC through normal CE GWE/GDI
    paint/update or visibility transitions.

- Worker thread stack slots previously underflowed the mapped process stack
  reserve once the app reached later worker threads.
  - Symptom: after complex window regions moved iNavi past the prior idle
    frontier, `target\window_region_complex_virtual_150s_*` crashed with
    `WRITE_UNMAPPED` at `pc=0x000e6cd4`, `sp=0x7fedff90`, fault address
    `0x7fedfffc` during a normal MIPS prologue store.
  - Evidence: the eighth 128 KiB guest worker slot landed at the bottom of the
    old 1 MiB process stack reserve, leaving no downward prologue headroom.
    The stack reserve is now 4 MiB and focused
    `guest_thread_stack_tests::eighth_guest_thread_slot_keeps_stack_headroom`
    covers the slot geometry.
  - Status: fixed by the stack-reserve slice; keep watching for true CE
    per-thread stack allocation/protection gaps when the scheduler grows real
    run-queue ownership.

- Dumped `explorer.exe` host-presented launch now exits through the emulator
  sentinel before useful UI.
  - Symptom: running
    `D:\INAVI_Emulator\DUMPPLZ\Windows\explorer.exe` with `--desktop host` and
    `D:\INAVI_Emulator\DUMPPLZ\Windows` as the DLL search path no longer stops
    on missing COREDLL ordinals or the old high-address trampoline failure,
    but the bounded probe reaches `pc=0x7ffffff0`, `ra=0x7ffffff0`, `v0=1`
    without render milestones.
  - Evidence: the latest run wrote
    `target\explorer_win32_host_destroyicon_summary.txt`,
    `target\explorer_win32_host_destroyicon_render.txt`, and
    `target\explorer_win32_host_destroyicon_milestones.txt`. The old
    `0xffff832c` trampoline failure is no longer the active blocker. The
    missing COREDLL frontier moved through `StringCchCatW @1693`,
    `wcsncmp @65`, and `DestroyIcon @725` before reaching the sentinel.
  - Status: launcher fidelity improved; still validate whether the sentinel is
    a clean guest process return or a too-early control-flow exit before
    treating explorer as a complete shell fixture.

- Main process launch presents the first iNavi splash frame but does not yet
  sustain post-splash UI progress.
  - Symptom: a bounded debug launch gets past the earlier empty-queue
    `GetMessageW` self-stop and the later SD-card validation dialog, creates
    main/child HWNDs, and now produces a real guest-presented splash/art frame
    through GDI. This is first UI progress, but not complete GUI success: after
    the presented child-window frame, the run still spends bounded wall time in
    the id-1000 no-HWND thread `WM_TIMER`/MFC idle-update loop and does not yet
    advance into sustained interactive UI.
    The latest resource-focused virtual probes with dumped runtime DLLs confirm
    that `\SDMMC Disk\INavi\res\values.dat` is not missing and is being read
    successfully, but `resource_59718` fails because the guest mode-47 resource
    table object is already populated when the one-shot table loader is called
    again. The relevant final trace state is `table=0x0079c440`,
    `buffer=0x3006d970`, `tree_root=0x3006e830`, `tree_count=215`,
    `check_result=0`, and `ready=1` for path
    `\SDMMC Disk\INavi\res\values.dat`. Disassembly confirms the guest loader
    at `0x0006bd18` returns `0` when the table buffer is non-null, so the next
    investigation should find why the resource-ready chain reaches this replay
    state rather than forcing the load to succeed.
  - Latest evidence: `target\update_erase_virtual_*` is the current frontier.
    The 20 s virtual run uses dumped runtime DLLs from
    `D:\INAVI_Emulator\DUMPPLZ\Windows`, stays memory-stable
    (`heap_live=13697/13300954B`, `virtual_live=3/196608B`,
    `host_open=665`, `host_read=80198/4060882B`, `mem_open=3`), and contains a
    real screen-presenting blit:
    `BitBlt(dst=0x02020008, dst_memdc=false, dst_hwnd=0x00020008,
    src=0x000a0044, src_memdc=true, 800x480)`. The framebuffer dump has
    `384000` nonzero pixels, and `target\update_erase_virtual.png` shows the
    iNavi SE splash/art frame. The run still stops by wall-clock with active
    timer id `0x3e8`/1000 due in virtual time, so this bug remains open as a
    post-splash scheduler/timer/GWE progression problem rather than a blank
    framebuffer problem.
    Current follow-up: `target\unicorn_realtime_timer_virtual_30s_*` keeps the
    same real present and stable memory, and the raw Unicorn `GetMessageW`
    bridge now lets long no-HWND timers mature inside the same live CPU run
    when the wall-clock budget permits. The run delivers two real `WM_TIMER`
    id-1000 messages (`time_ms` about `21829` and `29329`), then stops cleanly
    at `COREDLL.dll@861 blocked_get_message` because the next 7.5 s timer
    period does not fit the 30 s run budget. Counters are bounded
    (`PeekMessageW=200`, `GetMessageW=192`, `DispatchMessageW=191`,
    `sched=wait:3/0/3`, `wake=2`, `reg=3/2`, `msgcand=2`) and the earlier
    unsafe outer-loop continuation that reproduced `pc=0x00000000` is not the
    committed path. The framebuffer remains populated with the real iNavi SE
    splash/art UI, so the remaining failure is post-splash MFC/resource
    progression after valid timer wakes, not runaway timer fast-forward,
    blank framebuffer presentation, or lost blocked-thread context.
    Timer-id scoping follow-up: v3 no longer collapses all timers with the
    same numeric id into one global entry. Timers are now scoped by owner
    thread/message queue, optional `HWND`, and id, and raw `KillTimer(hwnd,id)`
    only removes the matching scoped timer. This closes duplicate timer-id
    aliasing as a likely explanation for the post-splash loop. Destroyed HWND
    subtrees also remove their window timers while preserving no-HWND thread
    timers, closing destroyed-window timer leaks as a likely explanation;
    guest-visible TimerProc delivery through `DispatchMessageW` now has a
    first bridge, while CE internal callback-timer bypass semantics and
    message/timer ordering remain open timer fidelity suspects. The bounded
    mounted follow-up `target\timer_scope_virtual_30s_*` still reaches the real
    800x480 memory-DC-to-window-HDC `BitBlt`, stays memory/file-I/O stable, and
    writes a populated framebuffer (`1151398` nonzero RGB bytes out of
    `1152000`).
    The TimerProc bridge follow-up `target\timer_callback_virtual_30s_*`
    likewise stays in the same real-present band (`pc=0x0030faec`,
    `heap_live=7327/5135247B`, `host_open=159`,
    `host_read=25713/1949108B`) with the same window-HDC 800x480 `BitBlt` and
    populated framebuffer (`1151398/1152000` nonzero RGB bytes), so TimerProc
    lParam/callout support did not regress first UI presentation.
    Current GWE follow-up: `target\hidden_sizemove_virtual_150s_*` keeps the
    same first display present and memory/file-I/O stability, and removes the
    non-CE immediate `WM_MOVE`/`WM_SIZE` traffic for direct-hidden geometry
    changes (`msgsig` dropped from `227` to `174`). It still parks at
    `COREDLL.dll@861 blocked_get_message`, so the open bug is now a cleaner
    post-splash presentation/state progression problem, not hidden move/size
    spam.
    GWE hidden-window follow-up: `target\hide_update_clear_virtual_20s_*`
    proves stale create-time update state is no longer left on immediately
    hidden MFC child controls. `ShowWindow(SW_HIDE)`/`SWP_HIDEWINDOW` now clear
    the hidden window's own update/erase state and `SetWindowPos` clips any
    surviving update rectangle to current client bounds. The bulk hidden
    `AfxWnd42u` controls now show `upd=false`; a later resized/invalidated
    hidden child is clipped to `update=0,0-100,135`. The probe remains
    memory-stable and the framebuffer remains populated (`1151398` nonzero RGB
    bytes), so stale hidden-child paint state is closed as a likely cause of
    the post-splash stall.
    Direct-UpdateWindow visibility follow-up:
    `target\update_effective_visibility_virtual_150s_*` confirms raw/kernel
    `UpdateWindow` now honors effective ancestor visibility. The latest run
    remains stable (`heap_live=13697/13300954B`,
    `virtual_live=3/196608B`, `host_open=665`,
    `host_read=80198/4056903B`, `mem_open=3`,
    `max_read=685080`) and reaches the same frontier: HWND `0x0002006c` is a
    hidden child (`style=0x40000000`, `upd=true`, `erase=true`), app drawing
    composes into offscreen memory DC `0x000a3f38`, and no later display-HDC
    blit or iNavi render milestone appears. Forcing hidden-child paint is no
    longer a valid suspect; investigate why the app never shows or presents
    that composed surface through normal GWE/GDI state.
    `SetWindowPos` metadata follow-up:
    `target\setwindowpos_showhide_virtual_150s_*` confirms show/hide-only and
    z-order-only `SetWindowPos` calls now queue `WM_WINDOWPOSCHANGED` with a
    `WINDOWPOS` payload even when the rectangle is unchanged. The run has more
    window-position message traffic, but it still stops at
    `COREDLL.dll@861 blocked_get_message` with the same stable I/O counters,
    the same hidden `0x0002006c` 800x54 pending update, and no later display
    present. Continue with remaining GWE window-position ordering gaps such as
    MFC show/idle sequencing or the missing screen-HDC blit path.
    Message-trace decode follow-up:
    `target\windowpos_trace_decode_virtual_150s_*` now prints the guest
    `WINDOWPOS` fields for queued `WM_WINDOWPOSCHANGED` records. It confirms
    the current stall is not due to an unreadable/opaque message payload:
    `0x0002006c` received `rect=0,0,800,480/flags=0x00000000`, while the run
    still ends at the same hidden-child/offscreen-composition frontier.
    Direct `ShowWindow` visibility follow-up:
    `target\showwindow_direct_visibility_virtual_150s_*` confirms raw/kernel
    `ShowWindow` now queues direct-state `WM_SHOWWINDOW` plus
    `WM_WINDOWPOSCHANGED` for children that were already effectively hidden by
    their parent. The real app path now shows decoded hide payloads with
    `flags=0x00000097` (`SWP_HIDEWINDOW|SWP_NOMOVE|SWP_NOSIZE|SWP_NOZORDER|SWP_NOACTIVATE`),
    including HWND `0x0002006c`. This still does not produce the later
    display-HDC blit; the child remains hidden with a pending 800x54 update.
    Tap/z-order/focus follow-up:
    `target\touch_focus_virtual_150s_*` confirms a generic input-ordering fix,
    not a UI breakthrough. New top-level HWNDs now become frontmost for
    hit-testing, so the tap reaches the visible full-screen popup
    `0x00020008`; `WM_LBUTTONDOWN` also activates/focuses that hit HWND before
    the mouse message. The guest WNDPROC runs and then deactivates/reactivates
    windows back toward the existing main path, but the run still ends at the
    same hidden `0x0002006c` pending-update frontier with no later display-HDC
    blit. This closes the old tap-to-older-window path as a false lead.
    File-mapping follow-up:
    `target\mapping_views_virtual_150s_*` confirms the new per-view mapping
    model is active in the mounted app: the real `UnmapViewOfFile` path now
    removes/releases one mapped view and the summary ends with
    `virtual_live=2/131072B` instead of the earlier three live virtual
    allocations. This did not advance visible UI. The run still ends at
    `COREDLL.dll@861 blocked_get_message`; the final state has no active
    timers, all the `AfxWnd42u` child controls were explicitly hidden by guest
    `ShowWindow(SW_HIDE)`, later `SetWindowPos` calls only move/size them
    without `SWP_SHOWWINDOW`, and HWND `0x0002006c` remains hidden with
    `update=0,0-800,54`. Continue with the generic guest state/resource/MFC
    progression that should eventually show or present those controls, not
    hidden-child forced painting.
    Child-process launch follow-up:
    `target\process_lifetime_virtual_150s_*` confirms the app now gets past the
    latest companion-process launch blockers. Rooted CE process names resolve
    through the mount table, `DeviceParser.exe`, `happyway_win.exe`, and
    `iSearch.exe` all run and return `0`, and the previous
    `AuthLibrary.dll overlaps pe-image` plus subsequent stale child-WNDPROC
    `pc=0` crash no longer reproduce. The run remains in the same broad
    post-splash class of bug: it parks at `COREDLL.dll@861
    blocked_get_message`, keeps bounded host-file reads
    (`80127/4046053B`), and still has no named render milestone after the
    first splash/art present. The hidden pending-update child is now
    `0x00020070` because the real child process work created/removed extra
    HWNDs before the final state.
    Historical evidence: the mounted virtual run with dumped runtime DLLs
    and real sibling app DLLs wrote `target\inavi_trampoline_virtual_*`. It
    preloaded `AuthLibrary.dll`, `TpSysAuth.dll`, `mMbcAuth.dll`,
    `tpeg_if_dll.dll`, and `tw_tpeg_if_dll.dll` from the main image directory,
    reached real `AuthLibrary` CRT `strcat @1063`, and no longer reproduces the
    old null `GetProcAddressW(TpSysCheckSerial)` crash. The following
    `WRITE_PROT addr=0x50000000` collision between CE virtual allocations and
    Unicorn external trampolines is also gone after moving the external
    trampoline pool to `0x70000000`. The run now stops only on the 30 s wall
    clock at `pc=0x0030f978`, `ra=0x002fd4cc`, with stable counters
    (`heap_live=7340/21892552B`, `virtual_live=3/196608B`,
    `host_open=161`, `host_read=26159/1947356B`, `mem_open=2`,
    `max_read=497178`) and repeated RSImage `CreateDIBSection` work. Render
    milestones are still `none`, and the framebuffer remains only the sparse
    301-byte red line, so this remains the no-useful-UI bug.
    The later thread-owned timer probe wrote `target\thread_timer_virtual_*`
    after v3 learned to deliver no-HWND `SetTimer` expirations to the owning
    GWE message queue and to fast-forward CE virtual sleeps without host
    `std::thread::sleep`. That run clears the earlier
    `COREDLL.dll@861 blocked_get_message` stop and runs to the 120 s wall
    limit at `pc=0x70028b7c`, `ra=0x6002537c`, repeatedly delivering
    `WM_TIMER` with `hwnd=0`, `wparam=1000`. It remains memory-stable
    (`heap_live=13697/13300954B`, `virtual_live=3/196608B`,
    `host_open=665`, `host_read=80132/4053923B`, `mem_open=3`,
    `max_read=685080`) but still has no useful UI: render milestones show
    memory-DC work only and the framebuffer still contains only the 401-pixel
    red tap marker. The active blocker has therefore moved from an empty
    `GetMessageW` self-stop to a periodic thread-timer/no-present frontier.
    The follow-up paint-priority fix wrote
    `target\paint_priority_final_virtual_*`; it preserves send-timeout expiry
    while making raw `GetMessageW`/`PeekMessageW` retrieve pending paint before
    generating due timers. That run reaches real paint (`BeginPaint=6`,
    `EndPaint=6`) and screen-HDC-derived 800x480 DIB work with stable memory
    (`host_open=665`, `host_read=80130/4044109B`, `heap_live=13697/13300954B`).
    It is still not useful GUI output: the final visible-looking blits remain
    memory-DC destinations, the framebuffer is still only the sparse tap line,
    and the run spends 20 s wall time fast-forwarding the id-1000
    thread-queue `WM_TIMER` loop to about `32,189,373 ms` virtual time.
    Earlier flamegraph-driven startup fixes removed per-import
    COREDLL export-table rebuilding, hot linear trampoline scans, linear
    mapped-blob instruction lookup, per-import heap/virtual allocation scans,
    and per-page heap spillover mapping from the hottest paths. A current
    mounted `--desktop host --tap 400,240 --cpu-wall-clock-limit-ms 60000` run
    reaches `pc=0x00b55150`, `ra=0x0030f384`, while heap live remains about
    7,530 allocations / 23.8 MB and the earlier multi-GB spike has not
    returned. The import counts include much deeper resource/DIB work by 60 s:
    `ReadFile=33759`, `CreateDIBSection=190`, `CreateRectRgn=3866`,
    `CombineRgn=3863`, and `DeleteObject=3865`. The final admin flamegraph
    runs farther and hits the next real guest/UI fault at `pc=0x0026f7e4`
    (`render_map_pointer_deref`), `addr=0x0000005c`, after
    `ReadFile=61825` and `CreateDIBSection=317`. The latest dump is sparse
    rather than blank: `target\startup_flamegraph_after_heap_chunk.ppm` has
    401 red pixels from `(0,160)` through `(400,160)`. The file-I/O hot path
    has since been fixed so existing host files remain host-backed instead of
    being preloaded into memory, including read-write opens, and raw COREDLL
    `ReadFile` streams into guest memory. Small host-backed reads use a bounded
    64 KiB per-handle cache. A release host/tap probe to the same `0x0026f7e4`
    stop wrote `target\file_io_hotpath_cached_boot_summary.txt` and
    `target\file_io_hotpath_cached_boot_files.txt`; counters show
    `host_file_open_count=633`, `host_file_read_count=64995`,
    `host_file_read_bytes=3787819`, `memory_backed_open_count=2`, and
    `max_read_request=685080`, so the active blocker is no longer bulk file
    preload or per-read reopen. A follow-up file-open fix now falls back to a
    read-only live host handle when existing files are requested read/write but
    Windows denies write access. The mounted virtual
    `target\file_rw_fallback_virtual_60s_*` probe stops at `pc=0x003426d0`,
    `ra=0x002fd5e8`, stays memory-stable (`heap_live=7482/23071147B`,
    `virtual_live=3/196608B`), reaches much deeper file activity
    (`host_open=235`, `host_read=38930/2229372B`), and has zero remaining
    `Access is denied`/failed `SDMMC Disk\mapdata\SearchDB\*.db` open records,
    but still reports render milestones `none` and only red tap pixels
    (`nonzero=301`). The latest GWE fidelity regression probe for
    dumped-runtime commctrl loading wrote `target\commctrl_virtual_60s_*`:
    verbose loader validation now maps
    `D:\INAVI_Emulator\DUMPPLZ\Windows\commctrl.dll` alongside
    `mfcce400.dll`, after the PE parser learned mapped-image zero-fill below
    `SizeOfImage`. The 60 s virtual probe stopped at `pc=0x00135bd4`,
    `ra=0x00135bc8`, stayed memory-stable (`heap_live=6981/21280227B`,
    `host_open=112`, `host_read=7840/1760751B`, `mem_open=2`), but render
    milestones were still `none` and the framebuffer contained only the
    101-pixel red tap marker. After removing the remaining common-controls
    import trap path, the follow-up `target\commctrl_searchpath_virtual_60s_*`
    probe stopped at `pc=0x6000d9b8`, `ra=0x6004fc6c`, stayed memory-stable
    (`heap_live=6927/21256913B`, `host_open=91`,
    `host_read=4302/1718377B`, `mem_open=2`), and still had render milestones
    `none` with only the same 101-pixel red tap marker. The follow-up
    scheduler/thread priority and
    multi-wait slice wrote `target\scheduler_priority_wait_virtual_60s_*`,
    stopped at `pc=0x00a4a1f4`, `ra=0x002017e0`, stayed memory-stable
    (`heap_live=6930/21296145B`, `host_open=92`,
    `host_read=4305/1765319B`, `mem_open=2`), and likewise had render
    milestones `none` with only the same 101-pixel tap marker. The
    suspend-count follow-up wrote `target\suspend_count_virtual_60s_*`, stopped
    at `pc=0x6000cee4`, `ra=0x6000d06c`, stayed memory-stable
    (`heap_live=6921/21255717B`, `host_open=91`,
    `host_read=4304/1728191B`, `mem_open=2`), and still reported render
    milestones `none` with only the 101-pixel tap marker. The multiple-wait
    parking follow-up wrote `target\multiple_wait_virtual_60s_*`, stopped at
    `pc=0x6000cfd4`, `ra=0x6000d044`, stayed memory-stable
    (`heap_live=6921/21255717B`, `host_open=91`,
    `host_read=4304/1732170B`, `mem_open=2`), and likewise had render
    milestones `none` with only the 101-pixel tap marker; this iNavi path did
    not exercise a multiple-wait block (`sched=wait:1/0/0`). This keeps the
    no-useful-UI bug open. The first Unicorn `MsgWaitForMultipleObjectsEx`
    parking bridge follow-up wrote `target\msgwait_parking_virtual_60s_*`,
    stopped at `pc=0x0006cbd4`, `ra=0x000bdfa0`, stayed memory-stable
    (`heap_live=6927/21273103B`, `host_open=92`,
    `host_read=4305/1769298B`, `mem_open=2`), but also did not exercise a
    parked msg-wait and still reported render milestones `none` with only the
    101-pixel tap marker. The CE pseudo current process/thread and KData
    current-ID slice wrote `target\pseudo_handle_kdata_virtual_60s_*`, stopped
    at `pc=0x6000cee4`, `ra=0x6000d06c`, stayed memory-stable
    (`heap_live=6921/21255717B`, `host_open=91`,
    `host_read=4304/1728191B`, `mem_open=2`), and still reported render
    milestones `none` with only the 101-pixel tap marker. The follow-up raw
    current-thread pseudo mutation slice wrote
    `target\pseudo_thread_mutation_virtual_60s_*`, stopped at
    `pc=0x6000cfd4`, `ra=0x6000d044`, stayed memory-stable
    (`heap_live=6921/21255717B`, `host_open=91`,
    `host_read=4304/1732170B`, `mem_open=2`), and still reported render
    milestones `none` with only the 101-pixel tap marker. The latest GWE
    fidelity regression probe for
    `GetQueueStatus`/`MsgWaitForMultipleObjectsEx` wrote
    `target\queue_status_msgwait_*` artifacts and stayed memory-stable
    (`host_read=4221/495853B`, `heap_live=5948/2767663B`), but still had no
    render milestones and an all-zero framebuffer body. The follow-up
    `PostQuitMessage` queue-state probe wrote
    `target\post_quit_queue_state_*` artifacts and likewise stayed at the
    resource/DIB frontier (`host_read=4221/495853B`,
    `heap_live=5948/2767663B`) with no render milestones and an all-zero
    framebuffer body. The `GetMessageWNoWait` raw-ordinal probe wrote
    `target\get_message_nowait_*` artifacts and stayed in the same frontier
    (`host_read=4221/499832B`, `heap_live=5948/2767663B`) with no render
    milestones and an all-zero framebuffer body. The latest message-metadata
    probe for `GetMessagePos`/`GetMessageQueueReadyTimeStamp` wrote
    `target\message_metadata_*` artifacts, stopped later at `pc=0x00895bfc`,
    reached `mapinfo.bin`/`iNaviData` file activity, and stayed memory-stable
    (`host_read=4225/486559B`, `heap_live=5621/2459146B`), but still had no
    render milestones and only one nonzero framebuffer byte. This remains a
    no-useful-UI bug. The dialog integer item slice wrote
    `target\dialog_int_*` artifacts and likewise stayed memory-stable
    (`host_read=4221/495853B`, `heap_live=5948/2767663B`) while reaching
    RSImage/DIB resource work, but it still had no render milestones and an
    all-zero framebuffer body. The raw HWND hit-test slice wrote
    `target\window_from_point_*` artifacts and stayed memory-stable
    (`host_read=4225/486559B`, `heap_live=5624/2461398B`) while reaching later
    map/device file activity, but it still had no render milestones and only
    one nonzero framebuffer byte. The later dialog-unit/dialog-navigation and
    indexed-DIB color-table slices pass focused raw tests and keep the mounted
    run memory-stable. The latest 30 s host/tap probe after those slices wrote
    `target\long30_*` artifacts and stopped at `pc=0x003446ec` with
    `heap_live=7297/21843020B`, `host_read=25097/1921203B`, and
    `target\long30_probe.ppm` containing 301 nonzero framebuffer bytes, but the
    render trace still reports no named render milestones and no useful UI. The
    embedded-BITMAPINFO indexed-DIB follow-up wrote
    `target\bitmapinfo_palette_*` artifacts and held the same sparse red line
    from `(0,160)` through `(300,160)` with stable memory
    (`heap_live=7192/21798813B`, `host_read=25079/1926075B`); it did not
    unlock a screen blit/presentation milestone. The fresh
    `CreateDIBSection` diagnostic probe (`target\dib_colors_fresh_*`) confirms
    that mounted RSImage indexed DIBSections now carry parsed color tables
    (`colors=256` for the 800-wide 8 bpp surfaces and populated partial tables
    such as `colors=199`, `colors=156`, `colors=223`, `colors=236`, and
    `colors=249` for later resources), but the framebuffer remains the same
    301-pixel red line with no named render milestone. The focus/activation
    lifecycle slice wrote `target\focus_activation_*` and stayed in the same
    band (`heap_live=7295/21831892B`, `host_read=24819/1924419B`,
    framebuffer 301 red pixels from `(0,160)` through `(300,160)`), with no
    named render milestone. The `EnableWindow` lifecycle slice wrote
    `target\enable_window_*` and likewise stayed memory-stable
    (`heap_live=7294/21830764B`, `host_read=24620/1918582B`) with the same
    301-pixel red line and no named render milestone. The `BringWindowToTop`
    z-order/activation slice wrote `target\bring_window_top_*` and stayed in
    the same band (`heap_live=7293/21820764B`,
    `host_read=24620/1922561B`), again with the same 301-pixel red line and
    no named render milestone. A virtual-desktop rerun wrote
    `target\virtual_after_bring_window_top_*`, stopped at `pc=0x00343750`,
    and again had 301 red pixels with no render milestone, but avoided showing
    the black host presenter window. The disabled-ancestor enabled-state slice
    wrote `target\disabled_ancestor_virtual_*` in virtual desktop mode, stopped
    at `pc=0x00339d90`, stayed memory-stable
    (`heap_live=7304/21886404B`, `host_read=25878/1940731B`), and preserved
    the same 301-pixel red line with no render milestone. The matching
    effective-visibility slice wrote
    `target\visibility_enabled_virtual_final_*`, stopped at `pc=0x00344780`,
    stayed memory-stable (`heap_live=7305/21887532B`,
    `host_read=26160/1961105B`), and again kept the same 301-pixel red line
    with no render milestone. The CE `WM_WINDOWPOSCHANGED` payload slice wrote
    `target\windowpos_virtual_*` and `target\windowpos_virtual_60s_*` in
    virtual desktop mode, so it avoided the black host presenter window. The
    60 s probe reached RSImage `CreateDIBSection` work and stayed
    memory-stable (`heap_live=6929/21276879B`,
    `host_read=7839/1759291B`), but the framebuffer only contained 101 red
    pixels from `(0,160)` through `(100,160)` and the render trace still
    reported no named render milestones. The fresh focus/activation cleanup
    rerun wrote `target\focus_activation_virtual_60s_*`, also in virtual
    desktop mode, stopped at `pc=0x002036fc`, and stayed memory-stable
    (`heap_live=7089/21301763B`, `virtual_live=3/196608B`,
    `host_open=115`, `host_read=7852/1765593B`, `mem_open=2`,
    `max_read=497178`). Its framebuffer still contained only 301 red pixels
    from `(0,160)` through `(300,160)` and the render trace still reported no
    named render milestones. The raw `SetParent` lifecycle slice wrote
    `target\set_parent_virtual_60s_*`, stopped at `pc=0x000be6e4`, and stayed
    memory-stable (`heap_live=6921/21255717B`, `host_open=91`,
    `host_read=4302/1718377B`, `mem_open=2`, `max_read=497178`), but still had
    no named render milestone and only 101 red pixels from `(0,160)` through
    `(100,160)`. The owner/child raw-create slice wrote
    `target\owner_child_virtual_60s_*`, stopped at `pc=0x002a252c`, and stayed
    memory-stable (`heap_live=6940/21278707B`,
    `virtual_live=3/196608B`, `host_open=112`,
    `host_read=7840/1760751B`, `mem_open=2`, `max_read=497178`), but still had
    no named render milestone and only the same 101 red pixels from `(0,160)`
    through `(100,160)`. The first post-cleanup probe using
    `D:\INAVI_Emulator\DUMPPLZ\Windows` as the DLL source, after the
    `GetUpdateRect`/`GetUpdateRgn` erase-query slice, wrote
    `target\get_update_erase_virtual_60s_*`; it stopped at `pc=0x00a436e0`
    with stable memory/file counters (`heap_live=6930/21294161B`,
    `virtual_live=2/131072B`, `host_open=92`,
    `host_read=4305/1769298B`, `mem_open=2`, `max_read=497178`), still had no
    render milestones, and still had only 101 red pixels from `(0,160)` through
    `(100,160)`. The follow-up dialog/control text-forwarding slice wrote
    `target\dialog_text_virtual_60s_*` with the same dumped DLL source; it
    stopped at `pc=0x0001362c`, stayed memory/file stable
    (`heap_live=7041/21284917B`, `virtual_live=3/196608B`,
    `host_open=113`, `host_read=7843/1763759B`, `mem_open=2`,
    `max_read=497178`), still had no render milestones, and still had only the
    same 101 red pixels from `(0,160)` through `(100,160)`. An experimental
    unconditional `WM_NCCREATE` create-callout probe wrote
    `target\nc_create_virtual_60s_*` and regressed mounted startup to an
    immediate empty `GetMessageW` stop (`pc=0x7fff0b60`,
    `heap_live=24/12914B`, `host_read=0/0B`); that behavior was removed. The
    retained `WM_CREATE == -1` create-abort slice wrote
    `target\create_abort_virtual_60s_*`, stopped at `pc=0x001e5408`, stayed
    memory/file stable (`heap_live=6926/21256719B`, `host_open=91`,
    `host_read=4304/1732170B`, `mem_open=2`, `max_read=497178`), still had no
    render milestones, and still had only the same 101 red pixels from
    `(0,160)` through `(100,160)`. The HWND menu-attachment slice wrote
    `target\menu_attach_virtual_60s_*` using
    `D:\INAVI_Emulator\DUMPPLZ\Windows`, stopped at `pc=0x004d8ba8`, stayed
    memory/file stable (`heap_live=6917/21255371B`, `host_open=91`,
    `host_read=4302/1718377B`, `mem_open=2`, `max_read=497178`), still had no
    render milestones, and still had only the same 101 red pixels from
    `(0,160)` through `(100,160)`. The ordered CE menu item slice wrote
    `target\menu_items_virtual_60s_*` using the same dumped runtime DLL source;
    it stopped at `pc=0x00496a44`, stayed memory/file stable
    (`heap_live=6930/21302289B`, `virtual_live=2/131072B`,
    `host_open=92`, `host_read=4305/1769298B`, `mem_open=2`,
    `max_read=497178`), still had no render milestones, and still had only the
    same 101 red pixels from `(0,160)` through `(100,160)`. The follow-up
    menu enable/check-state slice wrote `target\menu_enable_virtual_60s_*`,
    stopped at `pc=0x000b9940`, stayed memory/file stable
    (`heap_live=6929/21276863B`, `virtual_live=3/196608B`,
    `host_open=97`, `host_read=5581/1766846B`, `mem_open=2`,
    `max_read=497178`), still had no render milestones, and still had only the
    same 101 red pixels from `(0,160)` through `(100,160)`. The associated
    menu raw-GWE slice wrote `target\associated_menu_virtual_60s_*`, stopped
    at `pc=0x00929804`, stayed memory/file stable
    (`heap_live=6929/21276863B`, `virtual_live=3/196608B`,
    `host_open=97`, `host_read=4332/1769576B`, `mem_open=2`,
    `max_read=497178`), still had no render milestones, and still had only the
    same 101 red pixels from `(0,160)` through `(100,160)`. This remains
    no-useful-UI.
  - Evidence: latest bounded run with `--features unicorn`,
    `--dll-search-dir C:\Program Files (x86)\Windows CE Tools\wce420\STANDARDSDK_420\Mfc\Lib\Mipsii`,
    and `--mount-config mounts.toml` previously timed out after 30
    seconds. A later 1,000,000-instruction bounded run returned through the
    emulator diagnostic path: `CallWindowProcW @285` now enters the guest SDK
    MFC WNDPROC thunk at `0x6000e530`, then the import ring shows
    `DefWindowProcW @264`, `GetWindow @251`, `PeekMessageW @864`, and a final
    empty-queue `GetMessageW @861` `blocked_get_message` snapshot. The
    following bounded trace also logs a source-backed `CreateWindowExW` guest
    `WM_CREATE` callout with a CE SDK `CREATESTRUCTW` lParam for
    `hwnd=0x00020000`, but still reaches the same empty-queue
    `GetMessageW @861` diagnostic without hitting `BeginPaint`, `GetDC`,
    `GetWindowDC`, `SetTimer`, or `KillTimer`. A later 1,000,000-instruction
    bounded run after adding the generic presenter/desktop boundary still
    returned at the same `GetMessageW @861` `blocked_get_message` frontier. The
    framebuffer-plumbed run prints an attached 800x480 RGB565 virtual
    framebuffer before CPU execution. Solid `FillRect` is now connected to that
    attached framebuffer through COREDLL raw ordinal dispatch, but the target
    trace still has not produced visible app pixels. After raw `GetWindow`
    ordinal 251 support was
    added, a 1,000,000-instruction bounded launch still stopped at the same
    empty `GetMessageW @861` diagnostic; the recent import ring shows
    `GetWindow(hwnd=0x00020000, relation=GW_CHILD)` returning `0`, so the
    observed MFC child traversal is no longer just a stubbed ordinal. Raw
    `ShowWindow`, `SetWindowPos`, and `MoveWindow` now queue CE-style
    `WM_SHOWWINDOW`, `WM_WINDOWPOSCHANGED`, `WM_MOVE`, and `WM_SIZE` messages,
    but a corrected bounded run from
    `D:\INAVI_Emulator\INAVI\INavi\iNavi.exe` still reaches the same
    `GetMessageW @861` `blocked_get_message` frontier. Visible top-level
    `CreateWindowExW` now also normalizes a zero/default rect to the 800x480
    desktop and queues `WM_SHOWWINDOW`, `WM_WINDOWPOSCHANGED`, and
    `WM_SIZE(800,480)` before the first synthetic `WM_PAINT`; the latest
    3,000,000-instruction bounded run confirms those messages dispatch through
    SDK MFC, then reaches MFC `WM_IDLEUPDATECMDUI` (`0x0363`) handling and an
    empty `GetMessageW @861` queue without child HWND creation or GDI/DC import
    activity. SDK `coredll.lib` evidence later identified ordinal 1036 as
    `longjmp` and ordinal 2000 as `_setjmp`; the Unicorn import hook now
    restores the saved MIPS `jmp_buf`, so the prior `pc=0` failure after a
    stubbed `longjmp` return at MFC `0x6001f7f8` is no longer the current stop.
    The latest 500,000-instruction launch reaches `WCE_Solution_iNavi`,
    dispatches several main-window messages through `AfxWndProcBase`, restores
    through `longjmp`, and stops at the intentional empty `GetMessageW @861`
    `blocked_get_message` diagnostic. The latest 4,000,000-instruction mounted
    run fixes the earlier `GetModuleFileNameW` host-path leak and CE
    `wcsncpy` byte-count mismatch: `FindFirstFileW("\SDMMC Disk\iNaviData")`
    now maps to `D:\INAVI_Emulator\INAVI\iNaviData` and succeeds instead of
    showing the Korean SD-card-lock `MessageBoxW`. That run creates
    `WCE_Solution_iNavi` plus an `Afx:10000:b:0:40000006:0` child window. After
    adding real first palette/DC state, normalizing the observed COREDLL
    export-index ordinal for `GetPaletteEntries`, preserving checked SDK CRT
    ordinals before export-index fallback, returning heap-backed
    `RegisterGesture @2724` state, and writing basic system time structs, the
    latest mounted run now gets past the prior 1576, 2724, and 25 traps after
    creating the main and MFC child windows. `--cpu-wall-clock-limit-ms 15000`
    now stops this post-time path without external killing and writes
    `target\inavi-wall-clock-stop.ppm`; that dump body is still all zero. The
    captured snapshot stops at `pc=0x0001354c` with repeated SDK CRT
    `memset @1047`/`swprintf @1097` import activity, so the current failure is
    still "no useful GUI pixels," not a missing `GetSystemTime` import. A
    follow-up 8,000 ms run with compact import counting writes
    `target\inavi-import-counts.ppm`; its RGB body is still all zero, and the
    summary shows `memset @1047` 259 times plus
    `WINSOCK.dll!WSAStartup` once before the wall-clock stop. The app is still
    in post-time startup/import churn before useful drawing. With sampled
    Unicorn code tracing, a later 180,000 ms mounted run writes
    `target\inavi-sampled-180s.ppm`; its RGB body is also still all zero, but
    the run gets farther into app code before stopping in a date/geometry loop
    around `0x0024f80c`/`0x0024fa30`. After switching code diagnostics to read
    mapped PE/DLL bytes before Unicorn memory and sampling block traces, a
    90,000 ms no-tap mounted run returns in roughly 27 s at the idle
    `GetMessageW @861` `blocked_get_message` frontier with a visible `800x480`
    `wce_solution_inavi` top-level HWND and an MFC child HWND. A matching
    `--tap 400,240` run confirms `WM_LBUTTONDOWN`/`WM_LBUTTONUP` delivery and
    drains back to the same idle snapshot. Both dumps are still all zero, so
    the current failure is missing paint/GDI/surface output after startup and
    input delivery, not inability to reach the message pump. After correcting
    Unicorn WNDPROC return semantics so only `DefWindowProcW`/default-WNDPROC
    paths validate `WM_PAINT`, a 45,000 ms `--tap 400,240` rerun still drains
    back to idle with an all-zero dump. The trace shows top-level `WM_PAINT`
    entering app WNDPROC `0x000135cc` via `CallWindowProcW @285`, then falling
    through `DefWindowProcW @264` without `BeginPaint`, `GetDC`, or drawing
    imports. The compact import
    summary now includes
    `operator new @1095`, `SetRect @103`, `MultiByteToWideChar @196`, and more
    `GetClassInfoW @878`/class-registration traffic, so this is a deeper
    frontier and still not GUI success. SDK `coredll.lib` evidence then
    identified raw soft-float compare helpers `__lts` through `__ned` at
    ordinals 2042 through 2053; implementing those plus `__litofp @2032` and
    `__ultofp @2033` advances the release mounted run past the previous
    `__nes @2047` and `__litofp @2032` traps. Raw MIPS 64-bit helper dispatch
    and `$v1` import returns then advance the run past `__ll_div @2005`, and
    `GetTimeZoneInformation @27` support advances it past the time-zone query,
    and foreground-window activation support advances it past
    `SetForegroundWindow @702`. `InputDebugCharW @595` now returns CE-style
    no-debug-input (`OEM_DEBUG_READ_NODATA`/`0xffffffff`), advancing the run
    into a guest CPU exception (`interrupt_no=12`, `pc=0x00000000`,
    `ra=0x00035cf4`) after app jump-table code reaches
    `interrupt_last_pc=0x000ef80a`/`interrupt_last_insn=0x007b375a`; that was
    traced to the trampoline scanner rewriting branch-shaped halfword
    jump-table data. The scanner now preserves those table bytes, and the
    latest mounted release run reaches the next clean stop at COREDLL ordinal
    `1943` (`pc=0x7fff0900`, `ra=0x600110e4`). `ADBSetAccountProperties @1943`
    now returns `FALSE`/`ERROR_NOT_SUPPORTED`, moving the launch to an encoded
    guest `TerminateProcess` exit (`caller=0x0048fa90`, process `0x42`,
    `exit_code=0`); the framebuffer is still all zero. A later mounted monitor
    run with a real `tap 400 240` advances past the previous raw math traps
    (`__litodp @2036`, `__dpmul @2027`, `sqrt @1060`) and stops in
    `GetMessageW @861` with `blocked_get_message=thread:1 hwnd=any`, so the
    current active failure remains missing useful paint/GDI/framebuffer output
    after startup/input rather than those math imports.
  - Status: active; `TlsCall` now returns real CE-style slots,
    `CallWindowProcW` now enters guest window-procedure targets, and
    `CreateWindowExW` now delivers the first create-time message. Raw
    `GetWindow` ordinal 251 now handles CE SDK child/sibling/owner traversal,
    virtual HWND lifecycle queueing is connected for show/move/size changes,
    visible top-level create now queues the initial show/size sequence, empty
    class registration is rejected at the raw CE API boundary, and SDK MFC
    `_setjmp`/`longjmp` control flow is emulated in the Unicorn import hook. Raw
    `FindResourceW`/`LoadStringW` now normalize `hModule == 0` to the current
    process module, but the latest shorter iNavi run still shows an EXE-module
    `FindResourceW(..., name=0x0e01, type=RT_STRING)` miss; LLVM resource
    dumping confirms the EXE has no RT_STRING table. The latest bounded launch
    confirms the current frontier is a post-time long-running startup path with
    import-count evidence rather than an unimplemented import trap; sampled
    trace runs now push that frontier into app-side date/geometry work while the
    framebuffer stays blank. The latest launch-demanded stop has moved past the
    generic debug input helper into a MIPS CPU exception after the newly
    connected soft-float helpers, MIPS 64-bit helper returns,
    `GetTimeZoneInformation @27`, foreground-window activation, and
    `InputDebugCharW @595`, the halfword jump-table corruption at
    `0x000ebbf0`, and the `ADBSetAccountProperties @1943` import stop. Next
    work is to follow the decoded shutdown branch through guest function
    `0x0004390c`: it sends `0x5236` at `0x00043e30`/`0x00043e38`, and the main
    WNDPROC converts that custom message to `WM_CLOSE`. Identify which earlier
    CE/MFC result feeds that branch, then continue with CE-referenced raw
    behavior that advances the guest path toward the newly connected
    framebuffer drawing and the remaining GDI/DC/surface drawing and blit
    imports. Newer host/tap diagnostics add a direct render-surface gate:
    durable render milestones show `render_size_entry` receives `800x480`, but
    the app never reaches the surface-allocation probes at
    `0x00104904`/`0x00104910`. The later `WM_PAINT` path reaches
    `paint_render_call` and render entry `0x0010518c`, then returns
    immediately with `render_surface=0` and `render_enabled=0`. The `RT_STRING`
    block fallback fix removed the observed `#3867/type #6` miss, but did not
    change the all-zero framebuffer. The signed `SetFilePointer` fix moved the
    real monitor probe past the prior non-returning `values.dat` parser path:
    `until 0x000587ec 180000 0` now stops at
    `resource_ready_after_589dc` with `v0=0`; the later wide-printf fix below
    moves the run past the subsequent resource-root/readiness failure.
    The latest trace decoder update shows the `WM_SIZE` path itself is not
    missing dimensions: the call at `0x0002d1a0` passes `800x480` to render
    object `0x3006b360`, but dispatches vtable slot `+0xf0`
    (`0x0011ce60`) instead of the resize/allocation slot `+0xf4`
    (`0x001033e4`). The mounted run still idles at `GetMessageW @861` with an
    all-zero framebuffer, so the active display failure is the skipped real
    lifecycle path into `0x001033e4`, not a need to synthesize pixels.
    The `\res\values.dat` resource-root failure is fixed by CE wide printf
    semantics: COREDLL `vswprintf @1099` must treat default `%s` as a wide
    string in the MFC `CString::Format("%s", module_path)` path. A mounted
    trace-enabled monitor run with real `tap 400 240` no longer hits the old
    `0x00058a84` readiness failure and shows successful repeated `ReadFile`
    calls from `\SDMMC Disk\INavi\res\values.dat`. The framebuffer is still
    all zero after the 90 s bounded run, so the active bug remains the missing
    render/GDI/surface path after resources are loaded, not path translation or
    a need to mount app resource data at `\res`.
    The next resource-loading frontier is now after device/OEM classification:
    `SystemParametersInfoW(SPI_GETOEMINFO)` returns `iNavi GN 2010` from
    `regs.json`, the resource selector chooses `mode=47`, and `values.dat`
    record 47 is read into a 1746-byte payload. The parser reads a sane header
    (`field_count=215`) and first key, then stops with interrupt 20 at
    `pc=0`, `ra=0x0006bfb4`, `last_pc=0x0006bf8c` while reading/sign-extending
    the second key. The latest framebuffer dump remains all zero.
    `target\monitor_debugger_oeminfo.log` is compact; detailed evidence lives
    in the sibling `tracefile` artifacts.
    The active mounted frontier has since advanced beyond `values.dat` and PNG
    resource reads. RSImage stream diagnostics show `ReadFile` callbacks from
    `resi_800x480.bin` with full requested-byte transfers and valid embedded
    PNG headers, and the PNG loop returns at `0x0030f384`. Scheduler work now
    lets bounded parked `WaitForSingleObject` waits time out instead of
    depending solely on object signaling. This is a CE scheduler fix, not UI
    success; the active display bug remains missing useful render/GDI/surface
    output. Continuing from that point exits through the app
    singleton/already-running branch:
    `CreateMutexW(L"iNavi")` returns `ERROR_ALREADY_EXISTS`,
    `FindWindowW(title=L"iNavi")` finds hwnd `0x00020000`, then
    `SetForegroundWindow`, `ReleaseMutex`, and encoded `TerminateProcess`
    follow. The framebuffer remains all zero because the app exits before useful
    render output. Current work should identify why the singleton routine is
    reached with an existing `iNavi` mutex/window in this same mounted run, not
    fake success or suppress `ERROR_ALREADY_EXISTS`. Raw `UpdateWindow` is no
    longer a no-op: it now synchronously sends pending `WM_PAINT` through the
    window send path. Raw `RedrawWindow` also handles the first CE-backed
    rectangle/region invalidation, descendant, validate, erase, and update-now
    paths, and raw `ValidateRect` preserves representable remaining update
    bounds. Raw `GetUpdateRgn` now copies pending paint bounds into HRGN
    objects, and `GetWindowThreadProcessId` now reports real HWND owner IDs
    from the virtual GWE table. Raw `IsChild` now reports direct and descendant
    HWND relationships from the parent chain. Raw `SendNotifyMessageW` no
    longer executes different-thread notifications synchronously, so queued
    no-wait sends are closer to CE GWE behavior. Raw/kernel `DestroyWindow`
    now sends and records `WM_DESTROY` before final HWND cleanup, and the
    default `WM_CLOSE` shortcut records the same destroy observation. Delivered
    `WM_NCDESTROY` is now recorded through raw `SendMessageW` and Unicorn
    guest-WNDPROC returns, matching the CE MFC fake-NC-destroy path without
    inventing an OS-side automatic send. Raw/kernel parent `DestroyWindow` now
    records descendant `WM_DESTROY` before parent cleanup, and Unicorn guest
    child-WNDPROC destroy-callout chaining now delivers guest descendant
    `WM_DESTROY` callbacks child-first before final root cleanup. The short
    mounted destroy-lifecycle probes reached RSImage/resource file work by
    10 s but still reported no render milestones and all-zero framebuffer
    bodies. The latest guest-destroy-chain probe wrote
    `target\guest_destroy_chain_*`, stopped at `pc=0x600c9aec`, and likewise
    had no render milestones or framebuffer pixels. Receiver-side sent-message
    retrieval now has its own queue ahead of posted messages and marks
    `InSendMessage`/`QS_SENDMESSAGE`/send source state; the bounded
    `target\sent_queue_*` probe stopped at `pc=0x00b4bc1c` with no render
    milestones or framebuffer pixels. Cross-thread `SendNotifyMessageW` now
    uses that sent queue instead of a normal post and clears receiver send depth
    after dispatch; the bounded `target\send_notify_sent_queue_*` probe stopped
    at `pc=0x00339d8c` with no render milestones or framebuffer pixels.
    Sender-side sent-message transaction bookkeeping now records CE-style
    sender/receiver thread ids, flags, timeout metadata, active receiver send
    stack, WNDPROC result completion, and receiver-terminated completion for
    destroyed targets; raw receiver `DispatchMessageW` stores the dispatch
    result back into that transaction. Timeout expiry now marks queued timed
    sends result-ready and removes them from receiver retrieval; raw
    `SendMessageTimeout(..., timeout=0)` across threads now creates and
    immediately expires the same transaction instead of executing the receiver
    shortcut. The bounded `target\sync_send_transaction_*` probe stopped at
    `pc=0x00b4bc24` with no
    render milestones and an all-zero framebuffer body, and the bounded
    `target\send_timeout_expiry_*` probe stopped at `pc=0x00339c3c` with no
    render milestones and an all-zero framebuffer body. Unicorn raw
    `SendMessageW`/`SendMessageTimeoutW` now has a same-process cross-thread
    receiver-context guest WNDPROC callout that parks/restores the sender MIPS
    context through a scheduler-backed `SendMessage` blocked wait and completes
    the GWE sent transaction with the WNDPROC result; the bounded
    `target\receiver_context_send_*` probe stopped at `pc=0x00b4bc24`, reached
    real resource/DIB activity, but still had no render milestones and an
    all-zero framebuffer body. Scheduler send-reply waiters are now keyed by
    sent-message id and wake when the transaction completes, times out, or is
    receiver-terminated by target HWND destruction, with compact summaries
    exposing send-reply signal/candidate counters. The earlier short mounted
    `SendNotifyMessageW` probe reached later
    `mapinfo.bin`/`UID1:` file activity, another child HWND, and `GetDC`, but
    still produced no render milestones and only one nonzero framebuffer byte.
    Raw `IsDialogMessageW` now handles the first dialog-manager slice:
    unrelated HWNDs are not consumed, dialog-owned messages dispatch through
    the normal path, TAB uses dialog tab traversal, Escape routes as
    `IDCANCEL`, and Return uses a focused pushbutton or the dialog default
    pushbutton with `IDOK` fallback. GWE now covers basic
    `WM_GETDLGCODE` button codes plus `DM_GETDEFID`/`DM_SETDEFID`, so this
    improves modeless/modal-loop fidelity but does not yet cover the full CE
    dialog manager, `DLGC_WANT*` edge cases, Shift+TAB, or nested modal
    scheduling.
    The broader window/GWE subsystem still needs sender parking/resume across
    longer waits, reentrant cross-thread scheduling, destroyed-target behavior,
    input/focus/modal fidelity, and GDI/DC integration before this bug can be
    closed.

- Most COREDLL ordinals are still subsystem stubs.
  - Symptom: every static COREDLL ordinal has subsystem ownership and raw dispatch
    metadata, but only the implemented virtual Win32/CE facade, waveOut,
    `cemath`, the first kernel/thread/time/sync raw ordinal tranche including
    `QueryPerformanceCounter`, `QueryPerformanceFrequency`, and raw
    `CreateEventW`,
    local/heap/virtual memory tranche, raw file buffer/find marshalling, first
    registry create/query/enum/delete tranche, first class/HWND/RECT/message/
    focus/capture/z-order/timer GWE tranche, system-info/memory-status helpers,
    first resource/string tranche, and the Unicorn-only SDK MFC
    `_setjmp`/`longjmp` import control-flow path have real semantics.
  - Evidence: `src/ce/coredll.rs` reports implemented-vs-stubbed ordinal plan
    entries and returns subsystem stub policies for remaining exports. Raw
    tests now cover critical sections, interlocked operations, TLS/last-error,
    time, raw event creation/event modify/wait, close-handle,
    heap/local/virtual allocation, raw
    file buffers/cursor/size/flush/finds, registry create/query/enumeration,
    class registration/window lookup, HWND rectangles/points/text/window-long/
    focus/capture/z-order/timers/messages/paint updates, unplugged waveOut
    adapter marshalling, resources, and COM state.
  - Status: active ordinal-by-ordinal implementation work.

- External DLL import traps are launch stubs, not final DLL implementations.
  - Symptom: WINSOCK and OLE imports can be patched to trap addresses so
    execution can proceed, but most non-SDK-DLL functions return only
    conservative placeholder values. `commctrl.dll` is no longer in this stub
    bucket; when present in the DLL search paths, its imports patch to mapped
    DLL export addresses.
  - Evidence: `src/emulator/imports.rs` resolves loaded SDK DLL exports when
    available and does so before shim classification. MFC and `commctrl`
    imports are deliberately not stubbed anymore; unresolved slots are left for
    the loaded DLL path instead of being patched to emulator return shims.
  - Status: active launch-enabling diagnostic layer for WINSOCK/OLE external
    DLLs.

- PE resources are only partially loaded into `ResourceSystem`.
  - Symptom: resource API behavior works for registered virtual resources and
    PE-backed string tables. Raw PE resource data entries are collected for
    registration, but broader icon/bitmap/dialog/menu parsing/consumption and
    runtime resource-module loading are still incomplete.
  - Evidence: `src/ce/resource.rs` has HRSRC/HGLOBAL-like state and
    `src/pe/mod.rs` parses string-table resources for `LoadStringW` and raw
    resource data entries for registration. The iNavi EXE resource dump has no
    RT_STRING resources, while the latest startup trace still probes
    `FindResourceW(hModule=0x00010000, name=0x0e01, type=6)` and receives 0.
  - Status: next PE/resource integration step beyond strings.

- Remote API has no Rust socket transport yet.
  - Symptom: remote touch/key/GPS/audio/status behavior exists as emulator API
    state, but there is no HTTP/WebSocket listener serving `/api/v1/...`.
  - Evidence: `src/ce/remote.rs` implements state and control dispatch only;
    websocket audio sink state already tracks per-client host-time cursors and
    flush-marked chunks, and `AudioSinkRegistry` can fan out to host/websocket/
    debug sinks, but no socket writer consumes them yet.
  - Status: expected until host transport work lands.

- Scheduler/wait ownership is only partially ported to CE fidelity.
  - Symptom: wait calls now flow through scheduler accounting and the first
    Unicorn blocked/resumed wait bridges report scheduler counters, but
    complete CE waiter queues, timeout expiry, unified timer/serial/audio wake
    ownership, fuller child-process lifecycle scheduling, and full
    blocked-thread context scheduling are still open.
  - Evidence: `SOURCE_REFERENCES.md` records the CE scheduler/sync source
    anchors, and `TODO.md` has the first CE fidelity ledger entry. Mutex
    recursive ownership/release count handling is now source-backed and covered
    by focused Rust tests plus `tests/test_progs/163_mutex_recursive_ownership`.
    Scheduler-owned blocked-wait registration/per-handle queues now exist for
    parked Unicorn waits, and event/semaphore/final-mutex-release transitions
    now enqueue wait ids from those queues as pending wake candidates. Thread
    and process handle exit signaling now does the same for guest-thread exit,
    child-launch completion, raw process-handle `TerminateProcess`, and the CE
    current-process pseudo handle. Posted/thread/broadcast/quit/sent messages,
    remote input, and queued `WM_TIMER` posts now enqueue registered
    per-thread message waiters as pending wake candidates too. Parked
    Unicorn `GetMessageW` calls now also register in that scheduler
    message-wait queue with their original filters and recheck filtered GWE
    message readiness before consuming the message on resume. Bounded
    worker-thread `Sleep(ms)` calls now use the CE bounded timeout shape and
    register timeout-only scheduler waits; `SleepTillTick` uses the same
    bridge with a one-tick timeout. `Sleep(0)` now records a scheduler yield
    and swaps with a saved peer context when the current Unicorn bridge has one
    available. `Sleep(INFINITE)` now updates current-thread suspend counts and
    self-suspends/restores guest worker contexts through `ResumeThread`, but
    full CE run-queue ownership, pending PSL late-suspend, main-thread suspend
    blocking, long sleeps, and main-thread scheduler ownership remain partial. Remote
    serial/NMEA injection now queues registered serial-read waiters by COM
    handle, and parked raw serial `ReadFile` can resume by streaming bytes into
    the original guest buffer. Raw and Unicorn
    `MsgWaitForMultipleObjectsEx` now follow the CE SDK flag set by honoring
    `MWMO_INPUTAVAILABLE` while ignoring desktop-only bit `0x0001` instead of
    converting message waits into unsupported wait-all kernel calls. Full
    serial stack semantics, audio wake model, fuller timer ownership, and full
    scheduler-owned CPU-context/run-queue ownership remain partial. The mounted
    `target\scheduler_msgwait_virtual_60s_*` probe stayed memory-stable,
    exercised seven object signals and 148 message-input transitions without
    registered waiters on those handles/threads
    (`sig:7 cand:0 msgsig:148 msgcand:0 maxpend:0`), and still had no render
    milestones. Existing
    guest-visible wait return behavior is preserved in this slice.
  - Status: active CE-fidelity port; next scheduler work should replace the
    remaining ad hoc blocked-wait vectors, subsystem wake paths, and
    Unicorn-local saved-context storage.

- Post-splash iNavi rendering is still not advancing past the first real
  display present.
  - Symptom: virtual mounted iNavi now shows the real 800x480 iNavi SE splash
    frame, but later GDI work remains offscreen and no named render milestone
    appears by 150 s.
  - Evidence: `target\writefile_lasterror_virtual_150s_render.txt` records the
    splash path as guest memory-DC composition followed by
    `BitBlt(dst=0x02020008,dst_hwnd=0x00020008,dst_memdc=false,src=0x000a0044)`.
    Later trace entries show additional DIBSection creation and memory-DC
    `StretchBlt`/`BitBlt` work into an 800x54 surface, but no later
    display-surface blit. The milestones trace also shows
    `InvalidateRect(hwnd=0x0002006c, rect=NULL, erase=true)` where that child
    is hidden/effectively invisible.
    Current GDI selected-object follow-up: newly created DCs now have
    CE-backed stock/default selected objects and `SelectObject` returns
    restorable previous handles instead of `0`, matching the save/restore
    pattern used by MFC and the fixture programs. This fixes a generic GDI
    fidelity gap visible in mounted traces as `previous=0`. Fresh mounted
    validation in `target\gdi_stock_defaults_virtual_150s_*` confirms the real
    path now returns `previous=0x000b5080` for memory-DC bitmap selects and
    `previous=0x000b5007` for the stock black pen, but the post-splash frontier
    remains open: later work still composes the 800x54 strip into a memory DC,
    invalidates hidden HWND `0x0002006c`, and parks in `GetMessageW` with no
    later display-HDC present.
  - Status: active UI frontier. Investigate generic GWE visibility,
    invalidation propagation, paint/update ordering, and timer/message
    progression; do not force hidden child paints or app-specific pixels.

- Raw `WriteFile` failure on valid non-writable handles previously left
  `LastError` stale.
  - Symptom: a failed `WriteFile` could return `FALSE` and zero bytes written
    without replacing the calling thread's last-error.
  - Evidence: raw COREDLL `write_file_raw` returned `result.success` directly
    after writing the optional byte count. It now sets last-error to
    `ERROR_ACCESS_DENIED` for valid non-writable handles and clears it on
    success; focused raw tests cover host write-through and read-only-handle
    failure.
  - Status: fixed in commit `24edd3f`.

- Writable external SD-card dump opens can still become non-writable in the
  active mounted run.
  - Symptom: iNavi opens `SDMMC Disk\iNaviData\config.bin` for write, seeks to
    EOF minus six bytes, then `WriteFile("E\0O\0F\0")` reports zero bytes.
  - Evidence: `target\createfile_access_virtual_150s_files.txt` prints the
    `CreateFileWArg` fields `req=0x40000000 pos=0x00000003` for
    `SDMMC Disk\iNaviData\config.bin`, proving `GENERIC_WRITE` +
    `OPEN_EXISTING`. The host file SHA-256 remained
    `1F04AE1349063D3A79F74733B233D8872F9A0D808309C33158DCF2EF9A86188A`, and
    focused raw tests prove writable host-backed files write through when the
    host handle is writable.
  - Status: likely environment/permission downgrade for the external mounted
    dump. Prefer overlay/copy-on-write validation before permitting mounted
    probes to mutate the source SD-card tree.

- Host desktop windows may be inaccessible from the current automation session.
  - Symptom: `--desktop host` initializes the Win32 presenter and reports
    `desktop: win32 host presenter`, but an automated user32 `FindWindow`
    script could not discover the visible `WinCE virtual desktop` HWND in this
    session before the scripted timeout.
  - Evidence: `target\inavi-host-touch.out.log` reaches host presenter setup and
    PE import patching; no framebuffer dump was produced because the script
    stopped before it could inject a click or reach a blocked wait.
  - Status: use deterministic `--tap X,Y` runner injection for repeatable touch
    experiments until host-window discovery is reliable in the active session.
