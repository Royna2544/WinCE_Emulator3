# FakeCE / iNavi SE G3 Emulator

Rust bootstrap for a Unicorn-backed Windows CE-ish emulator targeting the iNavi
SE G3 MIPS R4000 application.

The current base focuses on reusable CE behavior rather than app-specific
shortcuts:

- registry loading from `regs.json`
- CE-style registry API calls backed by `regs.json`
- Win32/CE-shaped kernel facade for file, device, sync, GWE, timer, and waveOut
  calls
- GWE HWND state with CE-style window/client rectangles and coordinate mapping
- COREDLL ordinal dispatcher backed by checked-in Rust `ORD_*` constants, a
  static export table, and an ordinal `match`
- COREDLL ordinal plan entries split by subsystem with implemented-vs-stubbed
  status for the full static table
- simple CE CRT/math dispatch for common floating-point math and MIPS helper
  routines
- PE32 parser for DOS/NT/COFF headers, optional headers, sections, imports,
  exports, relocations, RVA mapping, and mapped image bytes
- remote-control API state for touch/key input, GPS/NMEA serial injection, IMU
  state, pause/resume, status JSON, logs, and audio chunks
- resource and COM subsystem state for HRSRC-like resource lookup and
  CoInitializeEx/class/object lifecycle modeling
- host-backed file API with contained guest-path translation
- device namespace loading from `serial_devices.json`
- kernel object handles
- GWE-style windows and message queues
- timer bookkeeping
- waveOut-style audio state and buffers
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

CPU execution is intentionally behind the `unicorn` feature until PE mapping and
import traps are wired:

```bash
cargo run --features unicorn -- --run-cpu
```
