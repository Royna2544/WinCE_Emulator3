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
- The host-backed file namespace now has a static CE mount table containing
  `SDMMC Disk`. `FindFirstFileW("\\")` enumerates that mount prefix, exact
  `\SDMMC Disk` returns directory metadata, and `--sdmmc-root DIR` binds the
  mount contents to host storage for file opens/finds beneath the mount.
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
    `SetLastError`, plus process-style `TlsCall` allocation/free for CE TLS
    slots `4..63`
  - `Sleep`, `GetTickCount`, `EventModify`, `WaitForSingleObject`, and
    `CloseHandle`
  - CE heap/local/virtual-memory ordinals: `GetProcessHeap`, `LocalAlloc`,
    `LocalReAlloc`, `LocalSize`, `LocalFree`, `HeapCreate`, `HeapDestroy`,
    `HeapAlloc`, `HeapReAlloc`, `HeapSize`, `HeapFree`, `HeapValidate`,
    `VirtualAlloc`, and `VirtualFree`, plus the remote/in-process local and
    remote heap variants routed through the virtual heap model
  - raw file ordinals: `CreateFileW`, `ReadFile`, and `WriteFile` now marshal
    UTF-16 paths, guest byte buffers, and transferred-byte output pointers;
    `SetFilePointer`, `GetFileSize`, and `FlushFileBuffers` track cursor,
    size, high-word output, and flush behavior for host-backed files;
    `FindFirstFileW`/`FindClose` marshal `WIN32_FIND_DATAW` and enumerate the
    static CE mount table at `\`
  - CE/MFC-style HWND geometry/state ordinals: raw `CreateWindowExW`,
    `DestroyWindow`, `ShowWindow`, `UpdateWindow`, `EnableWindow`,
    `IsWindow`, `IsWindowEnabled`, `IsWindowVisible`, `GetParent`,
    `GetDesktopWindow`, `SetFocus`, `GetFocus`, `SetWindowTextW`,
    `GetWindowTextW`, `GetWindowTextLengthW`, `GetClassNameW`,
    `SetWindowLongW`, `GetWindowLongW`, `SetWindowPos`, `MoveWindow`,
    `GetWindowRect`, `GetClientRect`, `ClientToScreen`, `ScreenToClient`,
    `MapWindowPoints`, `RegisterClassW`, `GetClassInfoW`, `FindWindowW`,
    `GetCursorPos`, `GetActiveWindow`, and `MessageBoxW`
  - CE/MFC-style message-pump ordinals: raw `GetMessageW`, `PeekMessageW`,
    `PostMessageW`, `SendMessageW`, `DispatchMessageW`, `TranslateMessage`,
    and `DefWindowProcW` marshal `MSG` structs and queue state
  - CE/MFC-style paint/update ordinals: `InvalidateRect`, `ValidateRect`,
    `GetUpdateRect`, `BeginPaint`, and `EndPaint` track pending window update
    state, marshal CE `RECT`/`PAINTSTRUCT` data, and synthesize `WM_PAINT`
    through the message pump for visible invalidated windows
  - unplugged multimedia adapter ordinals: raw `waveOutGetNumDevs`,
    `waveOutOpen`, `waveOutPrepareHeader`, `waveOutUnprepareHeader`,
    `waveOutWrite`, `waveOutPause`, `waveOutRestart`, `waveOutReset`,
    `waveOutClose`, `waveOutGetVolume`, `waveOutSetVolume`,
    `waveOutGetPosition`, `waveOutGetPitch`, `waveOutSetPitch`,
    `waveOutGetPlaybackRate`, `waveOutSetPlaybackRate`, `waveOutGetID`,
    `waveOutGetDevCaps`, and `waveOutGetErrorText` marshal CE
    `WAVEFORMATEX`, `WAVEHDR`, `MMTIME`, and output pointers without binding to
    host playback
  - resource ordinals: `FindResourceW`/`FindResource`, `LoadResource`, and
    `SizeofResource`, plus `LoadStringW` buffer copying/null termination for
    registered virtual strings
- GWE window state now tracks CE-style whole-window and client `RECT`s in screen
  coordinates, parent HWNDs, title/class text, enabled/visible/focus state, and
  window-long slots for MFC layout and subclassing paths.
- Added a virtual CE memory subsystem for process/custom heaps, local
  allocations, allocation sizes, frees, and page-granular virtual allocations.
- Added resource and COM subsystem state:
  - resources map `(module, name, type)` to an HRSRC-like handle, data pointer,
    byte size, and string resources keyed by module/id
  - COM tracks per-thread apartment initialization depth, registered class
    factories, and virtual object handles
- PE parsing now validates DOS/NT signatures, reads COFF and PE32 optional
  headers, tracks all 16 standard data directories, maps RVAs through section
  headers, parses import descriptors/thunks by name or ordinal, parses export
  functions, parses base relocation blocks, and can build a zero-filled mapped
  image buffer. Relocation application now refuses to move relocation-stripped
  images and applies supported CE/MIPS base relocations for SDK DLL images.
- PE parser smoke tests build a synthetic MIPS R4000 PE32 image with imports,
  exports, relocations, and mapped section data.
- Added a Rust `CeRemote` subsystem based on the prior remote server API shape:
  it queues touch/key events, injects serial/NMEA/GPS data, stores IMU state,
  tracks pause/resume, exposes remote status JSON, holds recent log lines, and
  manages remote audio client/chunk state.
- `CeKernel` owns `CeRemote` and can drain queued remote touch/key events into
  the GWE message queue.
- Audio output has two sink classes:
  - `AudioSinkRegistry` lets host, websocket, and debug sinks register behind a
    shared PCM submission/flush contract.
  - `HostAudioSink` is an explicit host adapter boundary with an unplugged
    default and a Windows `winmm` constructor gated through the `windows` crate.
  - `WebSocketAudioSink` owns per-client cursors, PCM chunk sequencing, PTS,
    queue limits, and flush-marked chunks for the remote audio path;
    middle-joined websocket devices attach at the host audio timeline and
    receive a trimmed partial chunk when the host is already inside a retained
    chunk.
- Debug builds include `LoggingAudioSink`, which records/logs PCM submissions
  and flush hints for short-audio debugging.
- Added a generic virtual framebuffer boundary:
  - `Framebuffer` describes a byte-addressable surface with dimensions, stride,
    pixel format, dirty rectangles, and mutable pixel storage without depending
    on Windows names or handles
  - `VirtualFramebuffer` provides an in-memory implementation, defaults to an
    800x480 RGB565 primary surface, and can write a temporary PPM dump
  - `main` owns the virtual framebuffer, updates the remote/input framebuffer
    size from it, and passes it into the Unicorn execution boundary
- Unicorn launch prep is wired:
  - parsed PE images can be mapped into the Unicorn memory plan
  - `--dll-search-dir` can load SDK DLL images such as `mfcce400.dll`; the main
    relocation-stripped EXE remains at its preferred base while relocatable DLLs
    are moved when their preferred base overlaps
  - COREDLL, commctrl, winsock, and OLE import slots are patched to shim trap
    addresses when no loaded DLL export resolves them
  - MFC imports are not emulated by external stubs; they must resolve to loaded
    SDK DLL exports such as `mfcce400.dll`
  - external imports can resolve to loaded DLL exports before falling back to
    module-owned traps
  - COREDLL traps decode MIPS `a0`-`a3`, dispatch through the raw ordinal
    dispatcher, write `v0`, and retain a debug snapshot with PC/RA/SP/v0/v1/
    a0-a3/t9 plus memory-fault details on run failure
  - guest heap pages are mapped as a CE heap arena for APIs that allocate and
    populate memory during the same import call
  - non-COREDLL supported DLLs other than MFC currently use module-owned launch
    stubs with debug logs, not final API semantics
- SDK CE 4.2 Mipsii COREDLL ordinal evidence from `coredll.lib` is now captured
  for the launch-demanded CRT ordinals: `_wcsdup`, `wcsrchr`, `malloc`,
  `memcpy`, `memset`, operator `new`, `swprintf`, `printf`, and `free`.
- Launch-demanded CE 4.2 CRT raw helper bodies now live in `src/ce/crt.rs`,
  with COREDLL keeping ordinal dispatch ownership and delegating the actual
  CRT memory/string routines to that module.
- The bounded Unicorn launch with SDK `mfcce400.dll` now progresses past the
  previous unmapped-write failures and stops at a null function-pointer call from
  the main image destructor/function-pointer table around `0x0048f9d4`.
- A targeted Unicorn probe shows the failing destructor/function-pointer call is
  currently `jalr` from `0x0048f9d4` through slot `0x30002390` with value
  `0x00010000`, so the immediate launch failure is a low/invalid registered
  function pointer rather than a normal guest exit.
- A follow-up write probe showed the exit table slot `0x30002390` was populated
  by guest code at `0x0048f864` with callback `0x00019d7c`; that callback is
  valid app code. The remaining `pc=0` symptom is therefore the direct Unicorn
  entry lacking a CE loader/thread-exit return address after cleanup completes.
- Heap and local reallocation growth now move allocations and the raw COREDLL
  reallocation shims copy the old guest bytes to the new block. This fixed the
  launch-path overlap where a later guest `memcpy` corrupted the CRT/MFC exit
  callback table after `_onexit` table growth.
- Unicorn now decodes the old MIPS CE directly encoded `TerminateProcess`
  kernel thunk (`API set 2`, method `2`) from the caller instructions when the
  guest exits through that path.
- The Unicorn MIPS backend now rewrites direct `jal`, ordinary conditional
  branch, and branch-likely sites in executable PE sections into same-image
  trampolines. This works around the observed Unicorn control-flow fault where
  returning into MFC branch/call sites could fall into `pc=0`/reserved
  instruction state. Branch-likely delay-slot annulment, normal branch delay
  slots, and `jal` link/delay-slot behavior are covered by feature-gated tests.
- Raw `GetMessageW` now models CE/MFC blocking semantics for an empty queue in
  the Unicorn import path. It stops the bounded run with a
  `blocked_get_message` debug snapshot instead of returning `FALSE` to MFC and
  causing normal thread/application cleanup.
- GWE now tracks pending update regions for visible windows. `ShowWindow`,
  `SetWindowPos`, `MoveWindow`, and `InvalidateRect` can mark a window dirty,
  `PeekMessageW`/`GetMessageW` can synthesize `WM_PAINT`, and `BeginPaint` or
  `ValidateRect` clears the pending update state using the CE SDK
  `PAINTSTRUCT` layout.
- Unicorn now initializes the main PE entry context with CE/MFC-style WinMain
  arguments: `A0=hInstance`, `A1=0`, `A2` pointing at a real empty UTF-16
  command-line string, and `A3=1` (`SW_SHOWNORMAL`). The kernel also tracks the
  main process module base so `GetModuleFileNameW(hInstance, ...)` returns the
  configured CE module path instead of failing for nonzero `hModule`.
- The bounded Unicorn launch of `INavi.exe` with SDK `mfcce400.dll`,
  `--sdmmc-root D:\INAVI_Emulator\INAVI`, and the current debug binary now
  progresses past the previous empty-queue `GetMessageW` frontier. The latest
  debug trace shows `PeekMessageW` and `GetMessageW` returning a synthetic
  `WM_PAINT`, followed by `DispatchMessageW` entering the SDK MFC window
  procedure for class `solution_inavi` at `0x6004eba8`. A 30-second bounded
  run still had to be killed by the timeout and produced no host-visible GUI;
  this is not launch success.
- The framebuffer-plumbed bounded launch prints an attached 800x480 RGB565
  virtual framebuffer (`stride=1600`, `bytes=768000`) before entering CPU
  execution. The same 30-second target run still times out and has to be
  killed, so the optional framebuffer dump is only produced for runs that
  return normally or error through the emulator path. A non-CPU smoke run wrote
  `target\framebuffer-smoke.ppm` from the virtual framebuffer.
- `TlsCall` ordinal 520 now returns a real CE-style TLS allocation result. The
  short debug trace changed the first `TlsCall(TLS_FUNCALLOC, 0)` result from
  `0` to slot `4`; a 10-second debug run still does not reach later GDI/DC
  imports, and a 30-second non-debug run still times out after the normal
  startup/framebuffer/PE mapping output. This is progress in startup TLS setup
  rather than GUI success.
- Added bounded Unicorn instruction-count tooling via
  `--cpu-instruction-limit N`. A 10,000-instruction run now returns an emulator
  snapshot instead of needing an external kill, and a 100,000-instruction run
  reaches SDK MFC code around `0x6004f6a0..0x6004f8dc` with PC near
  `0x600dd98c`. This is diagnostic tooling only; default `0` keeps the previous
  unbounded CPU behavior.
- Unicorn debug snapshots now include a compact recent-import ring with module
  kind, ordinal/name, the first four arguments, stack pointer, and return value.
  This is diagnostic tooling only and is used to continue launch tracing without
  enabling high-volume import logs.
- CE `CallWindowProcW` ordinal 285 now enters nonzero guest window-procedure
  targets directly from the Unicorn import hook. This follows the SDK MFC
  `CWnd::DefWindowProc`/superclass path rather than adding emulator-side MFC
  stubs. The latest 1,000,000-instruction bounded launch shows the prior
  `CallWindowProcW(0x6000e530, hwnd=0x00020000, msg=0x363, ...)` call pending
  inside guest MFC code, followed by `DefWindowProcW`, `GetWindow`,
  `PeekMessageW`, and an intentional `blocked_get_message` snapshot on an empty
  queue.

## Current State

- CPU execution is wired far enough to load mapped PE images, dispatch import
  traps, run the target entry path, execute SDK MFC code through the current
  MIPS trampoline workaround, create/show the main HWND, synthesize and dispatch
  the first `WM_PAINT`, enter guest `CallWindowProcW` targets, and then reach an
  empty-queue `GetMessageW` diagnostic snapshot. A generic virtual framebuffer
  is now attached to the emulator boundary, but guest drawing/blit behavior is
  not connected yet and this must not be treated as GUI success.
- Instruction-limited snapshots show the post-`WM_PAINT` path entering SDK MFC
  thread-local state and message pre-translation (`CThreadLocalObject::GetData`
  and later `CWnd::WalkPreTranslateTree`) rather than reaching guest drawing
  imports yet.
- The default bootstrap uses `regs.json` as backing storage for the fake CE
  registry API and creates base GWE, timer, audio, and memory-map state.
- The virtual Win32/CE framework and COREDLL dispatcher are connected to Unicorn
  import traps. SDK `mfcce400.dll` can execute from a relocated image through
  the current target startup and message-pump entry path. MFC imports are now
  SDK-DLL-only; commctrl, WINSOCK, OLE, and additional CE 4.2 ordinal behavior
  still need real subsystem-backed implementation as traces demand.
- Many COREDLL ordinals are classified and dispatchable but still stubbed by
  subsystem. Kernel/thread/time/sync, memory/local/heap/virtual allocation,
  raw file buffer/find marshalling, first GWE class/HWND/RECT/text/window-long/
  focus/message pump/paint-update behavior, unplugged waveOut adapter ordinals,
  system-info/memory status, and first resource raw ordinals have real
  CE-referenced semantics; remaining ordinals still need to be burned down
  subsystem by subsystem.
- Remote server socket/WebSocket binding is not implemented in Rust yet; the
  emulator-facing remote API state and dispatch behavior are present.

## False Leads

- None yet.

## Regressions

- None yet.
