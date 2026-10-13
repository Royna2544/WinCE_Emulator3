# FakeCE / iNavi SE G3 Emulator

Rust bootstrap for a Unicorn-backed Windows CE-style emulator targeting the
iNavi SE G3 MIPS R4000 application.

The project favors reusable CE behavior over app-specific shortcuts. Current
coverage includes PE loading, import patching, core kernel handles, memory,
registry, host-backed mounted files, GWE windows/messages, GDI/DIB rendering,
shell/resource APIs, Winsock, serial GPS input, waveOut/audio plumbing, and a
remote-control HTTP/WebSocket surface.

## Current State

- The reviewed [common architecture anchor](docs/ARCHITECTURE.md),
  [migration plan](docs/PLAN.md) and [19 subsystem pages](docs/README.md)
  implement the latest plan-first documentation direction, not finished runtime
  architecture. The earlier persistent-owner freeze passed 1,618 tests but its
  first normal trial host-crashed at 129.108 s/0xC0000005 with 1.62 GiB RSS.
  The later dirty-source post-order full release suite passed 1,623 tests
  (0 failed, 3 ignored); the isolated closed-owner test remains expected red
  at its final closure assertion. This does not accept fresh-owner execution
  or the real application.
  Exact failed executable/symbols are preserved; installed BED2 is restored.
  Rebuilding the dirty source does not reproduce that fallback. Crash/cache
  diagnostics continue in parallel; the fresh-engine Stage 2 gates precede
  further production scheduler/clock migration. Six isolated native stress
  cases pass and measure hook-associated retention;
  they do not clear the failed app run or the remaining Stage0/Stage2 gates.
- Runtime runs can reach real iNavi splash/resource-loading state with hidden
  map child windows created.
- Remote status/frame capture, touch input, and GPS/NMEA injection are wired.
- Current mounted runs open `COM7:`, `MFS1:`, and `SMB1:`; injected GPS bytes
  are positively returned by guest `ReadFile`, not just accepted by HTTP.
- Earlier BED2 bounded 330 s runs exited cleanly but remained on splash.
  Bounded read-only file windows reduce sampled peak host RSS from about
  1,300 to 562 MiB;
  the latest validated guard build passes 1,593 tests (one fixture ignored).
  Reallocation preserves old bytes on failure; authenticated callback recovery
  restores the original caller stack in the real app. The earlier BC 330 s run
  peaks at about 493 MiB and delivers the test tap after 0.835 s, but remains
  on splash. Full map readiness, residual memory and input lag remain open;
  these combined-run observations are not controlled performance claims.
  Checked initial allocation and native typed-message delivery are now
  validated/integrated; their new app run failed at260s with a pool-worker
  zero-PC CPU exception. That continuation bug is under investigation, so
  that354 build was not live-stable. The new12DF entry-only handoff guard
  passes1,585 tests and a clean330.268s actual GPS/tap run, still on splash;
  peakRSS/private556.0/571.3MiB, tap lag0.777s. Full map bring-up remains open.
  Bounded native diagnostics pass1,580 tests but are not installed/live-run.
  Two further genuine native tests reproduce unsafe branch-helper handoff
  and duplicate caller replay; the validated guard defers those interiors.
  The subsequent BED2 safe-yield guard passes1,593 tests and another clean
  330.234s run: splash by4.358s, tap callback0.397s, peakRSS/private555.6/568.4MiB.
  All observed frames remain genuine splash. These single observations are
  not controlled gains. The later E-7 correction queries actual native expiry
  and hard-fails native errors except source-validated guest exits, but complete
  stop-reason taxonomy and
  fresh-owner acceptance stay open; a retained terminal snapshot does not
  prove every native run succeeded.
- Real visible-message delivery advances startup into map-skin decoding. The
  remaining runtime frontier is map presentation and route interaction.
- Hookless trampoline execution is the default. The live-pump instruction
  budget remains one million per slice while time-based scheduling is validated.

See `PLAN.MD`, `PROGRESS.md`, `TODO.md`, and `KNOWN_BUGS.md` for the current
roadmap and durable project memory. Source evidence is indexed in
`SOURCE_REFERENCES.md`.

## Active Frontiers

- CE-backed GDI parity: bitmap creation, DIB color tables, alpha-preserving
  blits, compatible bitmap behavior, printer abort callbacks, and font
  enumeration validation.
- Message and windowing parity: hotkeys, modal loops, cross-thread sends,
  scheduler waits, and the runtime transition from splash/resource loading to
  map interaction.
- Storage and device parity: mounted file behavior, FSDMGR surfaces, serial GPS
  input, and sensor/device notification timing.

## Basic Commands

Run the base bootstrap:

```bash
cargo run
```

Inspect a PE image without executing it:

```bash
cargo run -- --image D:\INAVI_Emulator\INAVI\INavi\iNavi.exe
```

Run with Unicorn CPU execution:

```bash
cargo run --release --features unicorn,trace,win32-desktop -- --registry registry.reg --devices serial_devices.json --mount-config mounts.toml --image D:\INAVI_Emulator\INAVI\INavi\iNavi.exe --dll-search-dir "D:\INAVI_Emulator\DUMPPLZ\Windows" --desktop virtual --remote-server 192.168.0.39:8765 --framebuffer-dump target\last-framebuffer.ppm --run-cpu
```

Expose the v2-compatible remote API:

```bash
cargo run --release --features unicorn,trace,win32-desktop -- --registry registry.reg --devices serial_devices.json --mount-config mounts.toml --image D:\INAVI_Emulator\INAVI\INavi\iNavi.exe --dll-search-dir "D:\INAVI_Emulator\DUMPPLZ\Windows" --desktop host --remote-server 192.168.0.39:8765 --run-cpu
```

Use `--cpu-instruction-limit N` or `--cpu-wall-clock-limit-ms N` with
`--run-cpu` for bounded diagnostic runs.

Use `RUST_LOG=ce.slice_profile=debug` to compare guest instructions and MIPS.
`WINCE_EMU_GLOBAL_CODE_HOOK=1` restores the global code hook; arbitrary-PC
diagnostics also select it even when legacy sparse-hook flags are present.
`WINCE_EMU_LIVE_PUMP_RUN_SLICE_INSTRUCTIONS=0` is an opt-in scheduling
experiment, not the default.

## Remote API

The HTTP surface includes:

- `GET /api/v1/status`
- `GET /api/v1/frame.jpg`
- `GET /api/v1/debug/screenshot.png`
- `GET /api/v1/video.mjpg`
- `/api/v1/input/*`
- `/api/v1/sensors/*`
- `/api/v1/control/*`
- `GET /api/v1/control/ws`
- `GET /api/v1/audio/ws`

The remote CLI compatibility flags include `--remote-bind`, `--remote-port`,
`--remote-token`, `--remote-video-fps`, `--remote-jpeg-quality`, and
`--remote-audio*`.

## Validation

Use the focused checks that match the area being changed:

```bash
cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel
cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file
cargo test --features unicorn,trace,win32-desktop --test basic_subsystems
cargo check --features unicorn,trace,win32-desktop
```

Run `cargo fmt` or targeted `rustfmt` after Rust edits, and use
`git diff --check` before handoff.
