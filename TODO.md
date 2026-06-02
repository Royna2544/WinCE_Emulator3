# TODO

## Immediate

- Continue the launch path after the first synthetic `WM_PAINT` dispatch by
  connecting CE-referenced GDI/surface drawing and blit behavior to the generic
  virtual framebuffer and then to the generic presenter boundary. Do not treat
  the timeout-running paint loop as GUI success.
- Replace launch-stub behavior for commctrl, WINSOCK, and OLE imports with
  real subsystem-backed implementations as import traces demand. Keep MFC on
  the loaded SDK DLL path only; do not add emulator MFC stubs.
- Continue burning down COREDLL ordinals subsystem by subsystem, replacing
  stubbed ordinal plan entries with CE/MFC/SDK-referenced semantics. Next
  likely tranche: GDI/DC/surface drawing into the virtual framebuffer,
  PE-backed resource string/icon/bitmap loading, COM/OLE API dispatch when
  ole32 imports are connected, more GWE menu/dialog/control raw pointer
  marshalling, file attributes/directory metadata beyond the first
  `FindFirstFileW` tranche, and timer/system-time structs.
- Continue tracing after CE `CreateWindowExW` now delivers the source-backed
  create-time `WM_CREATE` callout and CE `CallWindowProcW` enters guest
  window-procedure targets. The latest bounded snapshot still reaches SDK MFC
  default/idle handling and then an empty-queue `GetMessageW` diagnostic.
  Raw `GetWindow` sibling/child traversal is now connected for the observed
  MFC `GetWindow @251` calls, and a bounded rerun shows the main window's
  `GW_CHILD` query legitimately returning `0`; next work is to identify the
  next CE/MFC-sourced queue, timer, paint, posted-message, window-child
  creation, or GDI behavior that should advance the path toward real
  framebuffer drawing.
- Continue connecting SDK CE 4.2 Mipsii COREDLL CRT ordinals from `coredll.lib`
  as the launch trace demands.
- Implement CRT `_msize`/`realloc`/operator delete ordinals from SDK evidence so
  MFC/CRT heap paths do not rely only on Local/Heap reallocation aliases.
- Extend `cemath` as real guest imports demand more CRT/floating-point helpers.
- Extend subsystem smoke tests as each shim is connected to guest import traps.
- Add import-trap argument/result marshalling tests that exercise the new raw
  heap/file/find/message/resource ordinals through decoded guest MIPS
  registers.
- Parse PE resource directories into `ResourceSystem` so `FindResourceW` and
  `LoadStringW` use the mapped guest image data rather than test-registered
  virtual resources.
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

- Add host presentation/streaming of framebuffer snapshots through `Presenter`
  implementations after guest drawing writes meaningful pixels.
- Keep actual host audio playback unplugged until guest callback/import trap
  semantics are traced; current waveOut work is a virtual adapter only, with an
  `AudioSinkRegistry`, a Windows `winmm` host-sink boundary, websocket sink, and
  debug logging sink ready for later binding.
- Implement socket behavior for WINSOCK imports.

## Parked

- App-specific fixes are parked unless backed by guest execution evidence.
