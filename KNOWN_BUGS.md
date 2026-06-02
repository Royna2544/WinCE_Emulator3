# KNOWN_BUGS

## Open

- Main process launch reaches the paint loop without useful GUI output.
  - Symptom: a bounded debug launch gets past the earlier empty-queue
    `GetMessageW` self-stop and dispatches a synthetic `WM_PAINT`, but still
    produces no drawn framebuffer/window output and must be killed by the
    timeout. A generic virtual framebuffer is attached, but guest GDI/surface
    drawing is not connected to it yet. This is not GUI success.
  - Evidence: latest bounded run with `--features unicorn`,
    `--dll-search-dir C:\Program Files (x86)\Windows CE Tools\wce420\STANDARDSDK_420\Mfc\Lib\Mipsii`,
    and `--sdmmc-root D:\INAVI_Emulator\INAVI` timed out after 30 seconds. The
    debug trace shows `PeekMessageW`/`GetMessageW` returning a synthetic
    `WM_PAINT` and `DispatchMessageW` entering the SDK MFC window procedure for
    class `solution_inavi` at `0x6004eba8`. The framebuffer-plumbed run prints
    an attached 800x480 RGB565 virtual framebuffer before CPU execution, but
    the timeout-killed target run does not return far enough to write the
    optional PPM dump. The previous hidden-window, empty-queue `GetMessageW`,
    `pc=0`/reserved-instruction, and decoded `TerminateProcess`
    startup-cleanup states are no longer the current stop.
  - Status: active; `TlsCall` now returns real CE-style slots, but a short debug
    trace still does not reach later drawing imports, and a 30-second non-debug
    run still times out after the startup/framebuffer/PE mapping output. Next
    work is to use bounded instruction snapshots to identify the post-TLS SDK
    MFC path and continue toward CE-referenced GDI/DC/surface drawing and blit
    behavior through the guest path.

- Most COREDLL ordinals are still subsystem stubs.
  - Symptom: every static COREDLL ordinal has subsystem ownership and raw dispatch
    metadata, but only the implemented virtual Win32/CE facade, waveOut,
    `cemath`, the first kernel/thread/time/sync raw ordinal tranche,
    local/heap/virtual memory tranche, raw file buffer/find marshalling, first
    class/HWND/RECT/message GWE tranche, system-info/memory-status helpers, and
    first resource tranche have real semantics.
  - Evidence: `src/ce/coredll.rs` reports implemented-vs-stubbed ordinal plan
    entries and returns subsystem stub policies for remaining exports. Raw
    tests now cover critical sections, interlocked operations, TLS/last-error,
    time, event/wait, close-handle, heap/local/virtual allocation, raw
    file buffers/cursor/size/flush/finds, class registration/window lookup,
    HWND rectangles/points/text/window-long/focus/messages/paint updates,
    unplugged waveOut adapter marshalling, resources, and COM state.
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

- PE resources are not loaded into `ResourceSystem` yet.
  - Symptom: resource API behavior works for registered virtual resources and
    strings, but mapped PE resource directory data is not wired to
    `FindResourceW`, `LoadResource`, `SizeofResource`, or `LoadStringW`.
  - Evidence: `src/ce/resource.rs` has HRSRC/HGLOBAL-like state, but
    `src/pe/mod.rs` does not populate it.
  - Status: next PE/resource integration step.

- Remote API has no Rust socket transport yet.
  - Symptom: remote touch/key/GPS/audio/status behavior exists as emulator API
    state, but there is no HTTP/WebSocket listener serving `/api/v1/...`.
  - Evidence: `src/ce/remote.rs` implements state and control dispatch only;
    websocket audio sink state already tracks per-client host-time cursors and
    flush-marked chunks, and `AudioSinkRegistry` can fan out to host/websocket/
    debug sinks, but no socket writer consumes them yet.
  - Status: expected until host transport work lands.
