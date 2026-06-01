# FakeCE / iNavi SE G3 Emulator

Rust bootstrap for a Unicorn-backed Windows CE-ish emulator targeting the iNavi
SE G3 MIPS R4000 application.

The current base focuses on reusable CE behavior rather than app-specific
shortcuts:

- registry loading from `regs.json`
- CE-style registry API calls backed by `regs.json`
- Win32/CE-shaped kernel facade for file, device, sync, GWE, timer, and waveOut
  calls
- host-backed file API with contained guest-path translation
- device namespace loading from `serial_devices.json`
- kernel object handles
- GWE-style windows and message queues
- timer bookkeeping
- waveOut-style audio state and buffers
- memory map validation and a Unicorn MIPS adapter boundary
- minimal PE image inspection

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
