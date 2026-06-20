# Source References

Refreshed on 2026-06-20. Keep this file as a compact index of active evidence.
Do not expand it into a historical progress log.

## Local Reference Roots

- Windows CE 6.0 source/header tree: `C:\WINCE600`
- Prior emulator notes, when needed for comparison only:
  `..\WinCE_Emulator_v2`
- Checked-in registry/device fixtures:
  - `registry.reg`
  - `serial_devices.json`
  - `mounts.toml`

## Active Evidence Map

- Loader and PE behavior:
  - CE loader/import/resource headers and implementation references under
    `C:\WINCE600`
  - Local implementation under `src\emulator\`, `src\pe.rs`, and loader-facing
    tests under `tests\`
- Shell, image-list, icon, and notification behavior:
  - CE shell/taskbar/resource sources under `C:\WINCE600\PUBLIC`
  - Local implementation under `src\ce\shell.rs`, `src\ce\coredll.rs`, and
    raw kernel/GWE tests
- GWE, menu, modal, message, and scheduler behavior:
  - CE GWE, window, menu, and message-loop references under `C:\WINCE600`
  - Local implementation under `src\ce\gwe.rs`, `src\ce\thread.rs`, and
    `src\emulator\unicorn.rs`
- Storage and FSDMGR behavior:
  - CE storage, AFS, FSDMGR, disk, FMD, and file-notification references under
    `C:\WINCE600`
  - Local implementation under `src\ce\kernel.rs`, `src\ce\coredll.rs`, and
    memory/file tests
- GDI, DIB, palette, text, and IME behavior:
  - CE GDIAPI, draw, font, palette, IMM, and TESTIME references under
    `C:\WINCE600`
  - Local implementation under `src\ce\gdi*`, `src\ce\coredll.rs`, and raw
    GWE/kernel tests
- Runtime iNavi investigation:
  - Current evidence should come from reproducible live runs, debug endpoints,
    target logs, and committed code changes.
  - Keep emulator policy generic; source references should describe CE/runtime
    behavior rather than app-specific shortcuts.

## Maintenance Rules

- Add a reference only when it supports active implementation or validation.
- Prefer precise file paths, function names, structs, constants, and observed
  call shapes.
- Remove references once they become stale, speculative, or disconnected from
  the active roadmap.
