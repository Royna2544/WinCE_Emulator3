# PROGRESS

## Confirmed

- Repository started with `RULES.md`, `regs.json`, and `serial_devices.json`.
- `regs.json` contains the registry snapshot used to seed the CE registry model.
- `serial_devices.json` contains enabled guest devices including `COM7:`, `COM3:`,
  `UID1:`, `PIC1:`, `BTN1:`, `LSD1:`, `MFS1:`, `SMB1:`, `CAM1:`, and `TWV1:`.
- Initial Rust crate scaffold added for a CE-ish base:
  - CE-style registry API model backed by `regs.json`
  - device namespace
  - kernel object handle table
  - GWE windows and message queue
  - timer bookkeeping
  - waveOut-style audio state
  - memory map validation
  - Unicorn MIPS adapter boundary
  - PE32 image parsing for headers, sections, imports, exports, relocations, and
    mapped image bytes
  - remote-control API state for touch/key input, GPS/NMEA serial injection, IMU
    state, pause/resume, status JSON, logs, and audio chunks
- Source references are recorded in `SOURCE_REFERENCES.md` for CE registry,
  GWE queue, waveOut exports, and MFC message pump behavior.
- Rust smoke tests cover bootstrapping registry/device JSON backing plus basic
  registry, device, GWE/message, timer, audio, handle, and memory-map behavior.
- Added a virtual Win32/CE API facade on `CeKernel` for:
  - `CreateFileW`-style file/device opens
  - `ReadFile`/`WriteFile`
  - `DeviceIoControl`
  - `CloseHandle`
  - `CreateEventW`/`SetEvent`/`ResetEvent`/`WaitForSingleObject`
  - `CreateMutexW`/`ReleaseMutex`
  - `CreateWindowExW`, `PostMessageW`, `SendMessageW`, `GetMessageW`, and
    single-step message pumping
  - `SetTimer`/`KillTimer`
  - `waveOutOpen`/`waveOutWrite` plus pause/restart/reset/volume helpers
- Host-backed file opens are contained under a configurable file root and reject
  parent-directory escapes.
- Added checked-in Rust COREDLL ordinal definitions in
  `src/ce/coredll_ordinals.rs`. Runtime dispatch now uses Rust `ORD_*`
  constants, a static export table, and an ordinal `match`, with 1,752 export
  entries including CRT/math additions.
- Added a COREDLL export table parser for CE `.def` source evidence and
  validation work; it is not the runtime ordinal source.
- Added a COREDLL dispatcher that routes implemented exports to the virtual
  Win32/CE framework and reports unresolved or unimplemented ordinals explicitly.
- Added a simple `cemath` subsystem for common CE CRT math exports and MIPS
  helper routines, including `sqrt`, `pow`, `fmod`, `div`, `ldiv`, `__ll_div`,
  `__ll_mul`, soft-float add/sub/mul/div, conversion, and compare helpers.
- COREDLL ordinal work is now split by subsystem in code. Every static export
  can produce an ordinal plan entry with subsystem ownership and
  implemented-vs-stubbed status; raw ordinal dispatch preserves raw arguments
  and routes unresolved semantics through subsystem-owned stub policies instead
  of a single generic unimplemented bucket.
- COREDLL raw ordinal dispatch has a stateful guest-memory path for the first
  CE-sourced kernel/thread/time/sync tranche:
  - CE `CRITICAL_SECTION` layout and fast-path state changes for
    `InitializeCriticalSection`, `EnterCriticalSection`,
    `TryEnterCriticalSection`, `LeaveCriticalSection`, and
    `DeleteCriticalSection`
  - `InterlockedTestExchange`, `InterlockedIncrement`,
    `InterlockedDecrement`, `InterlockedExchange`,
    `InterlockedExchangeAdd`, and `InterlockedCompareExchange`
  - per-thread `TlsGetValue`, `TlsSetValue`, `GetLastError`, and
    `SetLastError`
  - `Sleep`, `GetTickCount`, `EventModify`, `WaitForSingleObject`, and
    `CloseHandle`
  - CE/MFC-style HWND geometry ordinals: raw `CreateWindowExW`,
    `SetWindowPos`, `MoveWindow`, `GetWindowRect`, `GetClientRect`,
    `ClientToScreen`, `ScreenToClient`, and `MapWindowPoints`
  - resource ordinals: `FindResourceW`/`FindResource`, `LoadResource`, and
    `SizeofResource`, plus `LoadStringW` buffer copying/null termination for
    registered virtual strings
- GWE window state now tracks CE-style whole-window and client `RECT`s in screen
  coordinates and translates parent/client/screen points for MFC layout paths.
- Added resource and COM subsystem state:
  - resources map `(module, name, type)` to an HRSRC-like handle, data pointer,
    byte size, and string resources keyed by module/id
  - COM tracks per-thread apartment initialization depth, registered class
    factories, and virtual object handles
- PE parsing now validates DOS/NT signatures, reads COFF and PE32 optional
  headers, tracks all 16 standard data directories, maps RVAs through section
  headers, parses import descriptors/thunks by name or ordinal, parses export
  functions, parses base relocation blocks, and can build a zero-filled mapped
  image buffer.
- PE parser smoke tests build a synthetic MIPS R4000 PE32 image with imports,
  exports, relocations, and mapped section data.
- Added a Rust `CeRemote` subsystem based on the prior remote server API shape:
  it queues touch/key events, injects serial/NMEA/GPS data, stores IMU state,
  tracks pause/resume, exposes remote status JSON, holds recent log lines, and
  manages remote audio client/chunk state.
- `CeKernel` owns `CeRemote` and can drain queued remote touch/key events into
  the GWE message queue.

## Current State

- CPU execution is not yet wired to mapped PE images or import traps.
- The default bootstrap uses `regs.json` as backing storage for the fake CE
  registry API and creates base GWE, timer, audio, and memory-map state.
- The virtual Win32/CE framework and COREDLL dispatcher are ready for guest
  import traps to call into. PE import tables can now be parsed, but import trap
  patching and Unicorn memory mapping are not wired yet.
- Many COREDLL ordinals are classified and dispatchable but still stubbed by
  subsystem. Kernel/thread/time/sync, first GWE HWND/RECT, and first resource
  raw ordinals have real CE-referenced semantics; remaining ordinals still need
  to be burned down subsystem by subsystem.
- Remote server socket/WebSocket binding is not implemented in Rust yet; the
  emulator-facing remote API state and dispatch behavior are present.

## False Leads

- None yet.

## Regressions

- None yet.
