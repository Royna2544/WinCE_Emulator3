# KNOWN_BUGS

## Open

- Main process launch parks in an empty `GetMessageW` wait before useful GUI
  output.
  - Symptom: bounded launch exits the emulator process with status 0 only
    because the Unicorn runner stops itself at a CE/MFC-correct empty-queue
    `GetMessageW` block; this is not GUI success.
  - Evidence: latest bounded run with `--features unicorn`,
    `--dll-search-dir C:\Program Files (x86)\Windows CE Tools\wce420\STANDARDSDK_420\Mfc\Lib\Mipsii`,
    and `--sdmmc-root D:\INAVI_Emulator\INAVI` stops at trap
    `0x7fff0b60` / COREDLL ordinal 861 with
    `blocked_get_message thread_id=1 hwnd=<any> min_msg=0 max_msg=0`.
    The previous `pc=0`/reserved-instruction and decoded
    `TerminateProcess` startup-cleanup states are no longer the current stop.
  - Status: active; next work is real CE/MFC message, timer, paint,
    invalidation, and input behavior that wakes the pump through the guest path.

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
    HWND rectangles/points/text/window-long/focus/messages, unplugged waveOut
    adapter marshalling, resources, and COM state.
  - Status: active ordinal-by-ordinal implementation work.

- External DLL import traps are launch stubs, not final DLL implementations.
  - Symptom: MFC400/mfcce400, commctrl, WINSOCK, and OLE imports can be patched
    to trap addresses so execution can proceed, but most non-SDK-DLL functions
    return only conservative placeholder values.
  - Evidence: `src/emulator/imports.rs` classifies these modules, resolves
    loaded SDK DLL exports when available, and logs external import calls; real
    behavior still needs MFC/commctrl/WINSOCK/OLE subsystem shims.
  - Status: active launch-enabling diagnostic layer.

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
