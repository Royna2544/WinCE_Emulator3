# TODO

## CE Fidelity Ledger

- Scheduler/waits/thread contexts:
  - Source refs:
    `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\schedule.c`,
    `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\syncobj.c`, CE SDK/OAK wait
    headers, and `SOURCE_REFERENCES.md`.
  - v2 corroboration: v2 proved cross-thread wait/send/audio/serial parking was
    a viable emulator path, but v3 should keep CE source as the behavior
    authority.
  - Current v3 status: scheduler ownership has begun with a `Scheduler`
    subsystem that records single/multiple/msg wait attempts, wait outcomes,
    blocked waits, resumed waits, max handle count, and max timeout in monitor
    summaries. Parked Unicorn `WaitForSingleObject` calls now carry their
    start tick and timeout and can resume with `WAIT_TIMEOUT` when the bounded
    wait expires; object-signaled resumes still acquire/consume the waited
    object first. Existing CE6 `WaitForMultipleObjects(TRUE)` rejection is
    preserved from `NKWaitForMultipleObjects`.
  - Open gaps: real scheduler-owned waiter queues, unified timer/serial/audio/
    process wake ownership, blocked thread priority/fairness across all wait
    kinds, multiple-object parking, message-wait parking, and fuller Unicorn
    thread context switching still need the next scheduler port slices.
  - Fixture gates: keep existing wait/thread fixtures passing, then graduate
    pending scheduler fixtures for multiple waiters, `MsgWait*`, serial
    parking, waveOut callback wakeups, child-process waits, and scheduler mini
    app.
  - Latest iNavi evidence: active long-run frontier remains the render-map/
    surface path around `0x0026f7e4`. The bounded timeout-slice host/tap probe
    wrote `target\scheduler_timeout_*` artifacts and stopped at the familiar
    10 s resource-loading frontier with no render milestones. This scheduler
    slice is foundational and should not be counted as UI success until the
    mounted host/tap run advances. The bounded `SendNotifyMessageW` slice probe
    wrote `target\send_notify_*` artifacts and reached later file/window
    activity (`mapinfo.bin`, `UID1:`, child HWND, `GetDC`), but still had no
    render milestones and no useful framebuffer output.

- Window/GWE subsystem:
  - Source refs:
    `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\cmsgque.h`,
    `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\window.hpp`,
    `C:\WINCE600\PRIVATE\WINCEOS\COREOS\INC\gweapiset1.hpp`, CE SDK
    `winuser.h`, and MFC `wincore.cpp`/`thrdcore.cpp`/`wingdi.cpp`.
  - v2 corroboration: v2 had owner-thread queues, pending message transfers,
    `PendingUpdateWindow`, paint bounds, and host presenter refresh paths. Use
    that only as proof that those emulator paths were viable; CE source remains
    the behavior authority.
  - Current v3 status: raw class/HWND geometry, basic lifecycle messages,
    queue retrieval, guest WNDPROC callouts, subclass `CallWindowProcW`,
    paint/update state, `BeginPaint`/`EndPaint`, and basic `SendMessageW`/
    `DispatchMessageW` are present. `UpdateWindow` now forces pending
    `WM_PAINT` synchronously instead of acting as a no-op. Raw `RedrawWindow`
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
    saving the sender MIPS registers/running-thread metadata, activating the
    GWE sent transaction on the receiver, and restoring the sender with the
    WNDPROC result after the callout returns. `GetQueueStatus` now tracks
    CE-style current and changed queue bits, and raw
    `MsgWaitForMultipleObjectsEx` now wakes on newly changed queue input unless
    `MWMO_INPUTAVAILABLE` requests wake-on-current queued input.
    `PostQuitMessage` now records queue-owned quit state instead of an ordinary
    posted `WM_QUIT`, so `GetMessageW`/`PeekMessageW` observe quit even through
    nonmatching HWND/message filters. Raw `GetMessageWNoWait` now reaches the
    same GWE queue retrieval path instead of default ordinal handling. Raw
    `GetMessagePos` and `GetMessageQueueReadyTimeStamp` now use per-queue and
    per-message metadata from the GWE model: mouse-message screen positions are
    preserved separately from client `lParam`, and ready timestamps update when
    posts, sends, or queue-owned quit state make a thread queue ready. Raw
    `SetDlgItemInt`/`GetDlgItemInt` now reach the dialog child-window text
    model instead of generic ordinal fallback. Raw `WindowFromPoint` and
    `ChildWindowFromPoint` now route through GWE visible/enabled HWND hit
    testing instead of generic ordinal fallback. Raw `GetDialogBaseUnits` and
    `MapDialogRect` now cover CE dialog-unit mapping, and raw
    `GetNextDlgTabItem`/`GetNextDlgGroupItem` now walk real dialog children
    using visible/enabled state plus `WS_TABSTOP`/`WS_GROUP` boundaries.
    Raw/kernel `DestroyWindow` now records and sends `WM_DESTROY` before final
    GWE cleanup, and the default `WM_CLOSE` shortcut records the same destroy
    observation before deleting HWND state. `WM_NCDESTROY` is now tracked when
    actually delivered through raw `SendMessageW` or a Unicorn guest-WNDPROC
    return, matching CE MFC's source-backed fake-NC-destroy path instead of
    adding an OS-side synthetic send. Raw/kernel parent `DestroyWindow` now
    sends `WM_DESTROY` to descendants before the parent and before final GWE
    cleanup. Unicorn direct guest-WNDPROC `DestroyWindow` now chains guest
    descendant `WM_DESTROY` callbacks child-first before final root cleanup.
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
    the top-level target.
  - Open gaps: update regions are still represented as one bounding rectangle,
    so partial `ValidateRect`/`RedrawWindow(RDW_VALIDATE)` subtracts the
    representable remainder but keeps a conservative bounding rectangle for
    disjoint leftovers. Internal paint requests are represented as normal
    pending update state, `GetUpdateRect`/`GetUpdateRgn` do not yet send
    background erase when `bErase` is true, and full child clipping/z-order
    invalidation remains for the later GWE/GDI pass.
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
       dedicated top-of-z-order activation path. Remaining lifecycle work
       includes exact create/z-order side effects such as
       `WM_WINDOWPOSCHANGING`/`WINDOWPOS` marshalling and owner/topmost rules,
       deeper activate/focus/enable edge cases such as top-level owner
       activation, disabled-focus transfer, child implicit disabled state,
       no-activate show variants, and destroyed-target behavior under
       synchronous sends.
    3. Message queues and synchronous sends: replace same-thread-only shortcuts
       with scheduler-owned sent queues, sender blocking, receiver-context
       execution, `InSendMessage`, timeout, destroyed-target, and reentrant
       send behavior. `SendNotifyMessageW` has the first CE-backed no-wait
       split, and receiver-side sent-message retrieval/source/depth state now
       exists; cross-thread `SendNotifyMessageW` now uses that queue and clears
       receiver send depth after dispatch. Sender-side transaction bookkeeping
       now exists for blocking sends, and raw receiver `DispatchMessageW`
       stores the WNDPROC result back into that transaction. Timeout expiry now
       marks queued timed sends result-ready and removes them from receiver
       retrieval. The first Unicorn raw-send path now runs same-process
       cross-thread guest WNDPROCs in the receiver context and restores the
       sender result. `GetQueueStatus` changed-bit tracking and
       `MsgWaitForMultipleObjectsEx` `MWMO_INPUTAVAILABLE` semantics now cover
       the first CE queue-status slice. `PostQuitMessage` now uses
       `msgqfGotWMQuitMessage`-style queue state and ignores caller filters
       when delivering `WM_QUIT`. Raw `GetMessageWNoWait` now covers the
       nonblocking get-message API path. Raw `GetMessagePos` and
       `GetMessageQueueReadyTimeStamp` now cover the first CE
       `PostedMsgQueueEntry_t.time`/`MousePosAtPost` and queue
       `m_ReadyTimeStamp` metadata slice. Full scheduler-owned sender
       parking/resume across longer waits, reentrant cross-thread scheduling,
       nested modal loop unwinding, `ReplyMessage` wake semantics if a real
       export is confirmed, richer queue-source/filter precision, and complete
       destroyed-target behavior remain open.
    4. Window data/class/dialog/control surface: class atoms/extra bytes,
       `SetWindowLong`/`GetWindowLong`, owner thread/process queries, dialog
       procs/results, child/descendant relationship queries, child lookup,
       command routing, accelerator/menu state, and MFC attach/subclass paths.
       `SetDlgItemInt`/`GetDlgItemInt` now cover the first CE dialog integer
       item text path; `GetDialogBaseUnits`/`MapDialogRect` and
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
  - Latest iNavi evidence: the app reaches real windows and some sparse GDI
    pixels but still misses useful UI. Current render evidence says `WM_SIZE`
    reaches dimensions while the app skips its resize/surface allocation path;
    window work should keep tracing real lifecycle/message causes rather than
    faking pixels. The bounded destroy-lifecycle probe wrote
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
    thing being tested.

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
    source pixels.
  - Open gaps: complete palette/brush/pen/text/font/menu/dialog/icon/cursor
    behavior as trace evidence demands, broaden indexed DIB coverage beyond
    the currently reached RGBQUAD/RGBTRIPLE table shapes, and connect later
    blit/update transitions to real paint invalidation without app-specific
    shortcuts.
  - Fixture gates: keep raw GWE/GDI tests passing, then add focused fixtures
    for additional indexed palette edge cases, region clipping, text/font
    metrics, menu/accelerator, and MFC mini-app paint.
  - Latest iNavi evidence: mounted traces create many DIBSections and windows
    before useful rendering. The newest 30 s probe has sparse nonzero
    framebuffer bytes (`target\long30_probe.ppm`: 301 nonzero bytes) and
    memory-stable file I/O (`heap_live=7297/21843020B`,
    `host_read=25097/1921203B`) but no named render milestone, so the next
    display work should target generic paint/blit/surface fidelity, not forced
    presentation. The fresh `target\dib_colors_fresh_*` probe confirms the
    app's 8 bpp RSImage DIBSections now have parsed color tables (`colors=256`
    on the 800-wide surfaces and populated partial tables on later resources),
    so indexed palette ingestion is no longer the leading suspect.

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
  handle with bounded small-read caching, and the latest release host/tap run
  reports only 3,787,819 host-file bytes read before this fault. Next work
  should debug the null/invalid
  render-map object path around
  `0x0026f7c0..0x0026f7e4` using real guest state and existing probes. Do not
  fake-present DIBSections just because their bits are populated.
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
- Replace launch-stub behavior for commctrl, WINSOCK, and OLE imports with
  real subsystem-backed implementations as import traces demand. Keep MFC on
  the loaded SDK DLL path only; do not add emulator MFC stubs.
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
- Add an HTTP/WebSocket transport over the Rust `CeRemote` API state when the
  host runtime is ready for remote UI/audio streaming; audio transport should
  honor the sink's per-client cursors and flush-marked chunks immediately.
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
- Implement socket behavior for WINSOCK imports.

## Parked

- App-specific fixes are parked unless backed by guest execution evidence.
