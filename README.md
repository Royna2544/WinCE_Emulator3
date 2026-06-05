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
  `--mount-config mounts.toml`
- GWE HWND state with CE-style window/client rectangles, title/class text,
  window-long slots, focus, visibility/enabled state, coordinate mapping, and
  pending paint/update state
- raw CE/MFC-style `MSG`, `RECT`, and `PAINTSTRUCT` marshalling for the basic
  message pump and first paint-update path
- generic virtual framebuffer trait with an in-memory 800x480 RGB565
  implementation, dirty-rectangle tracking, optional PPM dumps, and no
  dependency on HWND/HDC/GDI concepts
- raw `FillRect` drawing for solid brushes on window/screen HDCs into the
  attached framebuffer, including RGB565 conversion and dirty-rectangle marking
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
- remote-control API state and a v2-shaped Rust HTTP transport for touch/key
  input, GPS/NMEA serial injection, IMU state, pause/resume, status JSON, JPEG/
  PNG/MJPEG framebuffer snapshots, logs, and audio-control metadata
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
cargo run --features unicorn -- --mount-config mounts.toml --image D:\INAVI_Emulator\INAVI\INavi\iNavi.exe --dll-search-dir "D:\INAVI_Emulator\DUMPPLZ\Windows" --desktop virtual --framebuffer-dump target\last-framebuffer.ppm --run-cpu
```

Expose the v2-compatible remote API while the emulator runs:

```bash
cargo run --features unicorn,win32-desktop -- --mount-config mounts.toml --image D:\INAVI_Emulator\INAVI\INavi\iNavi.exe --dll-search-dir "D:\INAVI_Emulator\DUMPPLZ\Windows" --desktop host --remote-server 0.0.0.0:8765 --run-cpu
```

The Rust listener serves `GET /api/v1/status`, `GET /api/v1/frame.jpg`,
`GET /api/v1/debug/screenshot.png`, `GET /api/v1/video.mjpg`, and the v2 REST
control routes under `/api/v1/input`, `/api/v1/sensors`, and `/api/v1/control`.
The REST handlers follow v2's validation and response shape for touch/key
input, NMEA/location injection, per-request frame `quality`, and MJPEG
`fps`/`quality`.
The `--remote-bind`, `--remote-port`, `--remote-token`,
`--remote-video-fps`, `--remote-jpeg-quality`, and `--remote-audio*` flags are
accepted for v2 CLI compatibility; WebSocket audio/control upgrade paths are
reserved but not implemented yet.

Use `--cpu-instruction-limit N` or `--cpu-wall-clock-limit-ms N` with
`--run-cpu` to make Unicorn return a bounded diagnostic snapshot instead of
relying on an external timeout.

The current bounded target run creates and shows the main HWND, delivers the
create-time `WM_CREATE` callout, dispatches the initial visible-window
`WM_SHOWWINDOW`/`WM_WINDOWPOSCHANGED`/`WM_SIZE` sequence, synthesizes and
dispatches the first `WM_PAINT` through the SDK MFC window procedure, and then
returns through the emulator's empty-queue `GetMessageW @861` diagnostic after
MFC idle UI update handling. There are now virtual framebuffer, presenter, and
desktop boundaries, and the first solid `FillRect` path can draw into the
attached framebuffer. The mounted run now gets past the earlier
`GetSystemTime @25` trap and can stop through the wall-clock limiter. Current
host-mode runs are no longer blank: by a 60 s wall-clock stop they reach real
paint/DC/DIB/resource work and sparse guest `Polyline` pixels in the
framebuffer. Flamegraph-driven startup fixes now get the profiled run far
enough to hit a later render-map dereference fault at `pc=0x0026f7e4`
(`addr=0x0000005c`) after 61k+ `ReadFile` calls and 317 `CreateDIBSection`
calls. This remains a useful frontier, not a completed GUI launch.
