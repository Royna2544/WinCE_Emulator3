# FakeCE / iNavi SE G3 Emulator

Rust bootstrap for a Unicorn-backed Windows CE-style emulator targeting the
iNavi SE G3 MIPS R4000 application.

The project favors reusable CE behavior over app-specific shortcuts. Current
coverage includes PE loading, import patching, core kernel handles, memory,
registry, host-backed mounted files, GWE windows/messages, GDI/DIB rendering,
shell/resource APIs, Winsock, serial GPS input, waveOut/audio plumbing, and a
remote-control HTTP/WebSocket surface.

## Current State

- The [architecture anchor](docs/ARCHITECTURE.md), [migration plan](docs/PLAN.md)
  and [subsystem pages](docs/README.md) guide the user-authorized scheduler-core
  rewrite; they do not mark it complete. The sole source checkout is the D:
  repository. Fresh-engine ownership, public same-VA/private-page coherence,
  one sealed task/frame-qualified run outcome, Stage 2 and the Stage 3 task
  migration remain open. The current D: implementation retains native
  process engines by default, but that behavior is experimental and not an
  accepted Stage 2 baseline. The approved target is fresh-engine-per-slice;
  persistent-owner architecture is deferred to Stage 8.
- The E-28 native count-stop query and typed `CountLimit` pass focused real-MIPS
  controls. The count-adapter source passed 1,671 release tests (0 failed,
  16 ignored) and 1,259 no-feature tests (0 failed, 14 ignored), plus no-trace
  and formatting checks. Those counts are **historical** for that exact source.
  Later ARM stop-reversal/reset controls and a checked finish-before-handoff
  repair pass focused tests. That follow-up source also passed **1,674
  release tests, 0 failed, 16 ignored** and **1,259 no-feature tests,
  0 failed, 14 ignored**
  ([release log](evidence/e28-run-outcome-20261014/broad-validation/release-unicorn-trace-desktop.raw.log),
  [no-feature log](evidence/e28-run-outcome-20261014/broad-validation/no-feature.raw.log)).
  The Unicorn+desktop/no-trace release check, format and diff checks also
  passed; exact source/log hashes are in the
  [post-order validation receipt](evidence/e28-run-outcome-20261014/broad-validation/validation.receipt.md).
  A later additive per-call native report retains outgoing-attempt evidence
  across a child handoff and distinguishes preflight `NoAttempt`; its frame is
  still `LegacyUnbound`, and the app scheduler does not consume it. Focused
  report/decoded-exit controls pass. The earlier report-source gates passed
  **1,682 release tests, 0 failed, 16 ignored** and **1,262 no-feature tests,
  0 failed, 14 ignored**, with check/format/diff gates green
  ([validation receipt](evidence/e28-run-outcome-20261014/detached-report-current-source-validation.receipt.md)).
  An executed ordinary finish-failure retry red is now corrected: the
  failed outgoing CPU rejects another native entry. The corrected source
  passed focused 1/0, full release **1,683/0/16 ignored**, no-feature
  **1,262/0/14 ignored**, and applicable check/format/diff gates
  ([red](evidence/e28-run-outcome-20261014/ordinary-finish-retry-expected-red.receipt.md),
  [green](evidence/e28-run-outcome-20261014/ordinary-finish-retry-green.receipt.md)).
  The older 1,674/1,259 and 1,682/1,262 counts above remain historical.
  E-29 below used a pre-fix executable and does not validate this correction.
  An exploratory same-handle nested-start hook probe exited
  `0xC0000005` before a query assertion, was removed, and remains unresolved
  ([E-28 ledger](docs/evidence/ledger.md#e-28-native-count-stop-witness-and-typed-adapter-2026-10-14-executed),
  [nested risk audit](evidence/e28-count-witness-audit-20261014/nested-count-risk.receipt.md)).
- The latest completed bounded iNavi run exited normally at its 330 s cap
  and remained on the genuine splash, not the map. Guest `COM7:` reads
  transferred injected raw NMEA bytes, and tap down/up WNDPROC returns were
  recorded. GPS fix, map readiness and route interaction were not demonstrated.
  Its sampled working-set and private peaks were 1,527.3/1,540.8 MiB. The
  read/write mounts were backed up as adjacent ZIPs before that run
  ([E-29 analysis](evidence/e22-live-20261013T211230Z/analysis.receipt.md)).
- Historical comparison, not current acceptance: the persistent-owner 650
  build passed 1,618 tests before a normal app run crashed at 129.108 s with
  `0xC0000005` and about 1.62 GiB RSS. The installed BED2 fallback's later
  safe-yield guard passed 1,593 tests and a 330.234 s GPS/tap run, still on
  splash; sampled RSS/private were 555.6/568.4 MiB and the tap callback took
  0.397 s. These different-source runs are not a controlled performance
  comparison ([history and caveats](docs/evidence/ledger.md)).
- Hookless trampoline execution remains the default. The live-pump limit is
  one million instructions per native slice; E-28 did not change scheduling
  policy, guest clock or CE quantum handling.

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
