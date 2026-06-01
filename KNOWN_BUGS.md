# KNOWN_BUGS

## Open

- CPU execution is not available in the default build.
  - Symptom: `--run-cpu` reports that the binary was built without the `unicorn`
    feature.
  - Evidence: PE image mapping and import traps are not implemented yet.
  - Status: expected initial scaffold limitation.

- Main process launch has not been driven to a stable guest entry yet.
  - Symptom: PE images and import traps can be prepared, but the target main
    EXE has not yet produced a successful bounded Unicorn run.
  - Evidence: `src/emulator/unicorn.rs` maps PE image bytes, installs import
    hooks, and records PC/RA/SP/v0/v1/a0-a3/t9 debug snapshots; next run must
    use the exact target path from `RULES.md` and inspect the first real
    failure.
  - Status: next bounded-run diagnostic.

- Most COREDLL ordinals are still subsystem stubs.
  - Symptom: every static COREDLL ordinal has subsystem ownership and raw dispatch
    metadata, but only the implemented virtual Win32/CE facade, waveOut,
    `cemath`, the first kernel/thread/time/sync raw ordinal tranche,
    local/heap/virtual memory tranche, raw file buffer marshalling, first
    HWND/RECT/message GWE tranche, and first resource tranche have real
    semantics.
  - Evidence: `src/ce/coredll.rs` reports implemented-vs-stubbed ordinal plan
    entries and returns subsystem stub policies for remaining exports. Raw
    tests now cover critical sections, interlocked operations, TLS/last-error,
    time, event/wait, close-handle, heap/local/virtual allocation, raw
    file buffers/cursor/size/flush, HWND rectangles/points/text/window-long/
    focus/messages, unplugged waveOut adapter marshalling, resources, and COM
    state.
  - Status: active ordinal-by-ordinal implementation work.

- External DLL import traps are launch stubs, not final DLL implementations.
  - Symptom: MFC400/mfcce400, commctrl, WINSOCK, and OLE imports can be patched
    to trap addresses so execution can proceed, but most functions return only
    conservative placeholder values.
  - Evidence: `src/emulator/imports.rs` classifies these modules and logs
    external import calls; real behavior still needs MFC/commctrl/WINSOCK/OLE
    subsystem shims.
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
