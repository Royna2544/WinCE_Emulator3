# KNOWN_BUGS

## Open

- Main process launch reaches the paint loop without useful GUI output.
  - Symptom: a bounded debug launch gets past the earlier empty-queue
    `GetMessageW` self-stop and dispatches a synthetic `WM_PAINT`, but still
    produces no drawn framebuffer/window output and must be killed by the
    timeout. A generic virtual framebuffer is attached, and raw solid-brush
    `FillRect` can now draw into it when guest code reaches a window/screen
    HDC, but the target launch has not yet reached useful drawing/blit imports.
    This is not GUI success.
  - Evidence: latest bounded run with `--features unicorn`,
    `--dll-search-dir C:\Program Files (x86)\Windows CE Tools\wce420\STANDARDSDK_420\Mfc\Lib\Mipsii`,
    and `--sdmmc-root D:\INAVI_Emulator\INAVI` previously timed out after 30
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
    `blocked_get_message` diagnostic.
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
    confirms the current `GW_CHILD` query returns no child HWNDs; next work is
    to identify which remaining CE/MFC-sourced queue, timer, paint,
    posted-message, window-child creation, resource-module loading, or GDI
    behavior should advance the guest path toward the newly connected
    framebuffer drawing and the remaining GDI/DC/surface drawing and blit
    imports.

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
