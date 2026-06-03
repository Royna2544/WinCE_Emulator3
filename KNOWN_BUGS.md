# KNOWN_BUGS

## Open

- Main process launch reaches the paint loop without useful GUI output.
  - Symptom: a bounded debug launch gets past the earlier empty-queue
    `GetMessageW` self-stop and the later SD-card validation dialog, creates
    main/child HWNDs, but still produces no drawn framebuffer/window output.
    A generic virtual framebuffer is attached, and raw solid-brush `FillRect`
    can now draw into it when guest code reaches a window/screen HDC, but the
    target launch currently stops before useful drawing/blit output. This is
    not GUI success.
    Newer host-mode runs do produce sparse real pixels through guest GDI
    `Polyline` (`target\startup_flamegraph_after_heap_chunk.ppm` has 401 red
    pixels from `(0,160)..(400,160)` in the latest profiled run), so the
    framebuffer is no longer strictly all-zero.
    It is still not useful GUI output: the app is currently spending bounded
    host/tap time in RSImage/PNG resource loading and DIB creation without
    reaching a screen blit/presentation ordinal.
  - Latest evidence: flamegraph-driven startup fixes removed per-import
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
    401 red pixels from `(0,160)` through `(400,160)`.
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
    PNG headers, and the PNG loop returns at `0x0030f384`. Continuing from that
    point exits through the app singleton/already-running branch:
    `CreateMutexW(L"iNavi")` returns `ERROR_ALREADY_EXISTS`,
    `FindWindowW(title=L"iNavi")` finds hwnd `0x00020000`, then
    `SetForegroundWindow`, `ReleaseMutex`, and encoded `TerminateProcess`
    follow. The framebuffer remains all zero because the app exits before useful
    render output. Current work should identify why the singleton routine is
    reached with an existing `iNavi` mutex/window in this same mounted run, not
    fake success or suppress `ERROR_ALREADY_EXISTS`.

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
  - Symptom: commctrl, WINSOCK, and OLE imports can be patched to trap
    addresses so execution can proceed, but most non-SDK-DLL functions return
    only conservative placeholder values.
  - Evidence: `src/emulator/imports.rs` resolves loaded SDK DLL exports when
    available. MFC imports are deliberately not stubbed anymore; unresolved MFC
    slots are left for the loaded SDK DLL path instead of being patched to an
    emulator `Afx*` return shim.
  - Status: active launch-enabling diagnostic layer for non-MFC external DLLs.

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
