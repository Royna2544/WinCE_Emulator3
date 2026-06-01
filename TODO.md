# TODO

## Immediate

- Wire the Unicorn backend to map guest memory regions into the engine.
- Map parsed MIPS PE32 images into Unicorn memory.
- Build import thunk/trap handling that decodes guest MIPS arguments and calls
  the COREDLL dispatcher.
- Connect guest MFC, CRT-like exports, and WINSOCK imports.
- Extend `cemath` as real guest imports demand more CRT/floating-point helpers.
- Extend subsystem smoke tests as each shim is connected to guest import traps.

## Next

- Add bounded run tooling and structured logs.
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
