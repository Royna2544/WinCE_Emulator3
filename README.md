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
- GWE HWND state with CE-style window/client rectangles, title/class text,
  window-long slots, focus, visibility/enabled state, and coordinate mapping
- raw CE/MFC-style `MSG` marshalling for the basic message pump
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
- remote-control API state for touch/key input, GPS/NMEA serial injection, IMU
  state, pause/resume, status JSON, logs, and audio chunks
- audio sink registry for host, websocket, and debug logging adapters; the host
  boundary has a Windows `winmm` constructor through the `windows` crate, while
  websocket PCM keeps per-client host-time cursors, partial late-join chunks,
  and immediate flush markers for short sounds
- resource and COM subsystem state for HRSRC-like resource lookup and
  CoInitializeEx/class/object lifecycle modeling
- host-backed file API with contained guest-path translation
- device namespace loading from `serial_devices.json`
- kernel object handles
- GWE-style windows and message queues
- timer bookkeeping
- unplugged waveOut adapter state with CE `WAVEFORMATEX`, `WAVEHDR`, and
  `MMTIME` raw ordinal marshalling
- memory map validation and a Unicorn MIPS adapter boundary

Behavior references are tracked in `SOURCE_REFERENCES.md`.

Run the base bootstrap:

```bash
cargo run
```

Inspect a PE image without executing it:

```bash
cargo run -- --image /mnt/d/INAVI_Emulator/INAVI/INavi/INavi.exe
```

CPU execution is behind the `unicorn` feature:

```bash
cargo run --features unicorn -- --image /mnt/d/INAVI_Emulator/INAVI/INavi/INavi.exe --run-cpu
```
