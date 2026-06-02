# FakeCE / iNavi SE G3 Emulator

Rust bootstrap for a Unicorn-backed Windows CE-ish emulator targeting the iNavi
SE G3 MIPS R4000 application.

The current base focuses on reusable CE behavior rather than app-specific
shortcuts:

- registry loading from `regs.json`
- CE-style registry API calls backed by `regs.json`
- Win32/CE-shaped kernel facade for file, device, sync, GWE, timer, and waveOut
  calls
- virtual CE memory model for process/custom heaps, local allocation,
  `VirtualAlloc` ranges, and raw `ReadFile`/`WriteFile` guest buffers
- host-backed file cursor, size, and flush behavior for raw file ordinals
- static CE mount-point enumeration for `\SDMMC Disk` plus host binding with
  `--sdmmc-root`
- GWE HWND state with CE-style window/client rectangles, title/class text,
  window-long slots, focus, visibility/enabled state, coordinate mapping, and
  pending paint/update state
- raw CE/MFC-style `MSG`, `RECT`, and `PAINTSTRUCT` marshalling for the basic
  message pump and first paint-update path
- generic virtual framebuffer trait with an in-memory 800x480 RGB565
  implementation, dirty-rectangle tracking, optional PPM dumps, and no
  dependency on HWND/HDC/GDI concepts
- generic presenter and desktop traits with virtual implementations, kept as
  host-side boundaries for presenting framebuffers and managing virtual windows
  without creating a host window yet
- COREDLL ordinal dispatcher backed by checked-in Rust `ORD_*` constants, a
  static export table, and an ordinal `match`
- COREDLL ordinal plan entries split by subsystem with implemented-vs-stubbed
  status for the full static table
- simple CE CRT/math dispatch for common floating-point math and MIPS helper
  routines
- PE32 parser for DOS/NT/COFF headers, optional headers, sections, imports,
  exports, relocations, RVA mapping, and mapped image bytes
- Unicorn launch prep for mapping PE image bytes, patching supported import
  DLL slots to trap stubs, dispatching COREDLL traps through the raw ordinal
  dispatcher, and reporting PC/RA/SP/v0/v1/a0-a3/t9 debug snapshots on failure
  or bounded self-stop states
- remote-control API state for touch/key input, GPS/NMEA serial injection, IMU
  state, pause/resume, status JSON, logs, and audio chunks
- audio sink registry for host, websocket, and debug logging adapters; `main`
  registers the Windows `winmm` host-sink boundary when running on a Windows
  host, while websocket PCM keeps per-client host-time cursors, partial
  late-join chunks, and immediate flush markers for short sounds
- resource and COM subsystem state for HRSRC-like resource lookup and
  CoInitializeEx/class/object lifecycle modeling
- host-backed file API with contained guest-path translation
- device namespace loading from `serial_devices.json`
- kernel object handles
- GWE-style windows and message queues
- timer bookkeeping
- waveOut adapter state with CE `WAVEFORMATEX`, `WAVEHDR`, `MMTIME`, and raw
  ordinal PCM marshalling into registered audio sinks
- memory map validation and a Unicorn MIPS adapter boundary

Behavior references are tracked in `SOURCE_REFERENCES.md`.
Integration tests are split by subsystem under `tests/` so raw COREDLL
dispatcher, kernel/thread, memory/file, GWE, waveOut, and broad smoke coverage
can grow independently.

Run the base bootstrap:

```bash
cargo run
```

Inspect a PE image without executing it:

```bash
cargo run -- --image /mnt/d/INAVI_Emulator/INAVI/INavi/iNavi.exe
```

CPU execution is behind the `unicorn` feature:

```bash
cargo run --features unicorn -- --image D:\INAVI_Emulator\INAVI\INavi\iNavi.exe --dll-search-dir "C:\Program Files (x86)\Windows CE Tools\wce420\STANDARDSDK_420\Mfc\Lib\Mipsii" --sdmmc-root D:\INAVI_Emulator\INAVI --framebuffer-dump target\last-framebuffer.ppm --run-cpu
```

Use `--cpu-instruction-limit N` with `--run-cpu` to make Unicorn return a
bounded diagnostic snapshot instead of relying on an external timeout.

The current bounded target run creates and shows the main HWND, delivers the
create-time `WM_CREATE` callout, dispatches the initial visible-window
`WM_SHOWWINDOW`/`WM_WINDOWPOSCHANGED`/`WM_SIZE` sequence, synthesizes and
dispatches the first `WM_PAINT` through the SDK MFC window procedure, and then
returns through the emulator's empty-queue `GetMessageW @861` diagnostic after
MFC idle UI update handling. There are now virtual framebuffer, presenter, and
desktop boundaries, but guest drawing is not connected to them yet, so this is
a useful frontier, not a completed GUI launch.
