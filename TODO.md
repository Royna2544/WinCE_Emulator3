# TODO

## Immediate

- Continue the launch path after the first synthetic `WM_PAINT` dispatch by
  expanding CE-referenced GDI/surface drawing and blit behavior beyond the
  first solid `FillRect` framebuffer path, then verify those pixels through the
  generic presenter boundary. Do not treat the timeout-running paint loop as
  GUI success.
- Continue the post-time iNavi path from the new wall-clock diagnostic frontier.
  The latest mounted run now gets past the
  earlier export-index `GetPaletteEntries` trap via real palette/DC state,
  preserves SDK CRT ordinals such as `memset @1047` and `swprintf @1097` before
  export-index fallback, returns heap-backed `RegisterGesture @2724` state, and
  writes `GetSystemTime @25`. With `--cpu-wall-clock-limit-ms 15000`, the run
  now returns a bounded snapshot and framebuffer dump without external killing;
  the dump is still all zero, and the snapshot stops around `pc=0x0001354c`
  with repeated SDK CRT `memset @1047`/`swprintf @1097` activity. Burn down the
  next real startup bottleneck before expecting guest drawing.
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
  registered with target WNDPROC `0x000135cc`. Continue with a targeted probe of
  `SetWindowLongW`/superclass state and first-message creation ordering before
  adding more lifecycle messages.
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
- Investigate the iNavi startup `FindResourceW(hModule=0x00010000,
  name=0x0e01, type=RT_STRING)` miss as a real MFC/resource-loading path. LLVM
  resource dumping confirms the main EXE has no RT_STRING table, so next
  candidates are language/resource DLL loading, MFC fallback behavior after
  missing `AFX_IDS_APP_TITLE`, or earlier app resource initialization state.
- When GWE/DC behavior is ready, adapt window state to the generic `Desktop`
  trait boundary without replacing CE/MFC message, class, or window semantics
  with host-window shortcuts.

## Next

- Extend bounded run tooling beyond the current snapshot import ring if more
  structured trace context is needed.
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
