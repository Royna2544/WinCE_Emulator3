# TODO

## Immediate

- Wire the Unicorn backend to map guest memory regions into the engine.
- Implement MIPS PE loading for the target executable.
- Build import thunk/trap handling for COREDLL, MFC, CRT-like exports, and WINSOCK.
- Connect guest COREDLL registry imports to the existing CE-style registry API.
- Add CE API shims for file, device, GWE, timer, and waveOut calls.
- Extend subsystem smoke tests as each shim is connected to guest import traps.

## Next

- Add bounded run tooling and structured logs.
- Add ordinal/decorated-name evidence from the Windows CE 4.2 Mipsii SDK import
  libraries, alongside the source references already recorded.
- Persist host-backed registry writes separately from the source dump.
- Add real serial backend support for `win32_com` devices.

## Later

- Implement drawing surfaces and blit paths.
- Implement audio playback backend after waveOut callback semantics are traced.
- Implement socket behavior for WINSOCK imports.

## Parked

- App-specific fixes are parked unless backed by guest execution evidence.
