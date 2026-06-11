# Known Bugs And Risks

Regenerated on 2026-06-11 from current source and test evidence. Items here are unresolved issues, unverified behavior, or risk areas that should not be presented as complete.

## Open Issues

- `SendMessageTimeout` is not yet proven complete for nested sends, reentrant waits, `ReplyMessage`, abort-if-hung edge cases, and all scheduler wait combinations.
- Shell notification behavior needs integrated validation for COM callback dispatch, taskbar-visible state, handle ownership, stale updates, and remove/update races.
- Mounted storage behavior still needs coverage for volume-handle ownership, cross-mount file-change delivery, external SD-card writeability, and rename churn.
- Loader parity is strongest in the Unicorn runtime path. Raw/non-Unicorn loader behavior should be explicitly audited before treating all `LoadLibraryEx` variants as equivalent.
- PE icon extraction now exists, but malformed resource tables, uncommon icon formats, ordinal/name edge cases, and mask/alpha fidelity need more coverage.
- GDI/text/input fidelity remains incomplete around IME behavior, caret timing, font fallback, glyph metrics, clipping, palette state, and alpha/mask parity.
- iNavi route-flow completion remains open beyond process startup, shell readiness, and initial UI/window behavior.
- Detached host launches can keep `wince_emulation_v3.exe` alive while the
  remote HTTP listener is already gone/refusing connections. Foreground runs
  print and hold the remote server long enough to boot, so this needs a launcher
  or remote-server lifetime fix before unattended UI driving is reliable.
- The iNavi nearby-search G-sensor modal still indicates incomplete sensor
  initialization semantics. `YAS526B`, light-sensor, and per-I2C-bus command
  contracts have been verified, but the SMB380 command family is still
  unresolved and must come from real dump/caller evidence.

## Build And Validation Risks

- The normal validation profile uses `--features unicorn,trace,win32-desktop`. No-feature test support needs an explicit decision and cfg audit before it can be treated as a required gate.
- `registry.reg` loading accepts REGEDIT text through UTF-8/lossy decoding and
  decodes typed `hex(2)`/`hex(7)` values as UTF-16. If a future REGEDIT4 export
  depends on a non-UTF-8 ANSI code page for quoted string values, those strings
  may need code-page-aware decoding.
- `IOCTL_NANDUUID_MICOM_RESET_STAGE` is currently reported as unused by the validated build profile; remove it or document the intended future use.
- `git diff --check` may report CRLF warnings on existing files. Treat non-CRLF whitespace findings as actionable.

## Recently Closed From Source State

- `ExtractIconExW` no longer appears to be synthetic-only: current source reads PE resources, chooses an icon group, builds color/mask bitmaps, creates icon handles, and falls back to shell icons for index zero.
- File-change notification coalescing now handles duplicate records, transient create/delete churn, and modified/delete collapse.
- Destroying a cross-thread `SendMessageTimeout` target now writes a zero result to `lpdwResult` for the completed destroyed-target case.
