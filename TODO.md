# TODO

## Immediate

- Run a bounded Unicorn launch of
  `/mnt/d/INAVI_Emulator/INAVI/INavi/INavi.exe` with the SDK MFC DLL search dir
  and trace the null function-pointer call at `0x0048f9d4`/RA `0x0048f9dc`.
- Add a targeted write diagnostic for the heap-backed destructor/function-pointer
  table slot currently seen at `0x30002390` to identify which guest instruction
  registered the low pointer value `0x00010000` before `jalr`.
- Replace launch-stub behavior for MFC400/mfcce400, commctrl, WINSOCK, and OLE
  imports with real subsystem-backed implementations as import traces demand.
- Continue burning down COREDLL ordinals subsystem by subsystem, replacing
  stubbed ordinal plan entries with CE/MFC/SDK-referenced semantics. Next
  likely tranche: PE-backed resource string/icon/bitmap loading, COM/OLE API
  dispatch when ole32 imports are connected, more GWE class/menu/dialog/control
  raw pointer marshalling, file attributes/find-directory APIs, and
  time/system-info structs.
- Continue connecting SDK CE 4.2 Mipsii COREDLL CRT ordinals from `coredll.lib`
  as the launch trace demands.
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
  host runtime is ready for remote UI/audio streaming; audio transport should
  honor the sink's per-client cursors and flush-marked chunks immediately.
- Add ordinal/decorated-name evidence from the Windows CE 4.2 Mipsii SDK import
  libraries, alongside the source references already recorded.
- Persist host-backed registry writes separately from the source dump.
- Add real serial backend support for `win32_com` devices.
- Bridge selected virtual Win32/CE APIs to host Win32 APIs where that preserves
  real guest semantics.

## Later

- Implement drawing surfaces and blit paths.
- Keep actual host audio playback unplugged until guest callback/import trap
  semantics are traced; current waveOut work is a virtual adapter only, with an
  `AudioSinkRegistry`, a Windows `winmm` host-sink boundary, websocket sink, and
  debug logging sink ready for later binding.
- Implement socket behavior for WINSOCK imports.

## Parked

- App-specific fixes are parked unless backed by guest execution evidence.
