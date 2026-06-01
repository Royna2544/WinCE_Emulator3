# TODO

## Immediate

- Wire the Unicorn backend to map guest memory regions into the engine.
- Map parsed MIPS PE32 images into Unicorn memory.
- Build import thunk/trap handling that decodes guest MIPS arguments and calls
  the COREDLL dispatcher.
- Continue burning down COREDLL ordinals subsystem by subsystem, replacing
  stubbed ordinal plan entries with CE/MFC/SDK-referenced semantics. Next
  likely tranche: PE-backed resource string/icon/bitmap loading, COM/OLE API
  dispatch when ole32 imports are connected, more GWE class/menu/dialog/control
  raw pointer marshalling, file attributes/find-directory APIs, and
  time/system-info structs.
- Connect guest MFC, CRT-like exports, and WINSOCK imports.
- Extend `cemath` as real guest imports demand more CRT/floating-point helpers.
- Extend subsystem smoke tests as each shim is connected to guest import traps.
- Add import-trap argument/result marshalling tests that exercise the new raw
  heap/file/message/resource ordinals through decoded guest MIPS registers.
- Parse PE resource directories into `ResourceSystem` so `FindResourceW` and
  `LoadStringW` use the mapped guest image data rather than test-registered
  virtual resources.

## Next

- Add bounded run tooling and structured logs.
- Add an HTTP/WebSocket transport over the Rust `CeRemote` API state when the
  host runtime is ready for remote UI/audio streaming.
- Add ordinal/decorated-name evidence from the Windows CE 4.2 Mipsii SDK import
  libraries, alongside the source references already recorded.
- Persist host-backed registry writes separately from the source dump.
- Add real serial backend support for `win32_com` devices.
- Bridge selected virtual Win32/CE APIs to host Win32 APIs where that preserves
  real guest semantics.

## Later

- Implement drawing surfaces and blit paths.
- Implement audio playback backend after waveOut callback semantics are traced.
- Implement socket behavior for WINSOCK imports.

## Parked

- App-specific fixes are parked unless backed by guest execution evidence.
