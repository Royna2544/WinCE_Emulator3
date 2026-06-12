# Known Bugs And Risks

Regenerated on 2026-06-11 from current source and test evidence. Items here are unresolved issues, unverified behavior, or risk areas that should not be presented as complete.

## Open Issues

- `SendMessageTimeout` is not yet proven complete for broader reentrant waits, additional cross-thread `SMTO_BLOCK`/`SMTO_ABORTIFHUNG` edge combinations, and all scheduler wait combinations; zero-timeout cross-thread raw expiry, nonzero timeout stale-delivery cleanup, Unicorn block-prep refusal, below-threshold vs hung `SMTO_ABORTIFHUNG`, same-thread abort-if-hung non-abort, early/nested `ReplyMessage`, and active-outer-timeout/nested-send lifetime coverage are covered.
- Shell notification behavior needs integrated validation for Explorer/taskbar `IShellNotificationCallback` dispatch through a real guest COM object lifecycle, taskbar-visible state, broader handle ownership, and remove/update races; the raw stale registered-taskbar, `SHNN_SHOW` window-only taskbar path, GUID-value OLE `CoCreateInstance` import writeback, local callback-acquisition queue path, mapped guest vtable callout, unmapped callback-pointer retry, and iconic remove/sink-cleanup icon lifetime paths are now covered.
- Mounted storage behavior still needs coverage for internal-vs-external notification handle behavior and remaining volume-handle owner/reset/close edges; cross-mounted file-change isolation, same-parent and cross-parent mounted rename scoping, target-process duplicate ownership, write-close completion records, read-only-source cross-volume move copy/delete semantics, DeleteAndRename/PrestoChango direction, read-only/removable volume attributes, and system/hidden mount attributes are now covered for mounted roots.
- Loader parity is strongest in the Unicorn runtime path. Raw/non-Unicorn loader behavior should be explicitly audited before treating all `LoadLibraryEx` variants as equivalent.
- PE icon extraction now exists, but malformed resource tables, uncommon icon formats, live callback-selected `KernExtractIcons` behavior, non-PE fallback edges, and mask/alpha fidelity need more coverage; raw exact integer `RT_GROUP_ICON` lookup and no-output `KernExtractIcons` failure behavior are now covered.
- GDI/text/input fidelity remains incomplete around IME behavior, caret timing, font fallback, glyph metrics, clipping, palette state, remaining alpha orientation/clipping edges, additional overlay rendering variants, and mask parity; the CE all-white image-list overlay-mask zero-bounds, non-rectangular overlay-mask paths, and raw `AlphaBlend` negative framebuffer destination clipping are now covered.
- iNavi route-flow completion remains open beyond process startup, shell readiness, and initial UI/window behavior.
- Detached host launches can keep `wince_emulation_v3.exe` alive while the
  remote HTTP listener is already gone/refusing connections. Foreground runs
  print and hold the remote server long enough to boot, so this needs a launcher
  or remote-server lifetime fix before unattended UI driving is reliable.
- The iNavi nearby-search G-sensor modal still indicates incomplete sensor
  initialization semantics. `YAS526B`, light-sensor, and per-I2C-bus command
  contracts have been verified, but the SMB380 command family is still
  unresolved and must come from real dump/caller evidence.
- Remote-driven iNavi startup now advances past the real `happyway_win` modal
  far enough to record `end_dialog`; runtime cleanup now drops stale
  cross-process `SendMessageW` yield snapshots whose live sent-message record is
  gone. A fresh driven run is still needed to prove the parked sender resumes
  through that handoff and to identify the next blocker before sensor
  initialization traffic is observed. Current device trace only shows `UID1:`
  NAND UUID IOCTLs.
- Dead modal dialog pixels can remain in the framebuffer after the modal window
  tree is destroyed. The window model and hit testing are correct, but the
  exposed underlying iNavi region still depends on the app servicing pending
  paint.

## Build And Validation Risks

- The normal validation profile uses `--features unicorn,trace,win32-desktop`. No-feature test support needs an explicit decision and cfg audit before it can be treated as a required gate.
- `registry.reg` loading accepts REGEDIT text through UTF-8/lossy decoding and
  decodes typed `hex(2)`/`hex(7)` values as UTF-16. If a future REGEDIT4 export
  depends on a non-UTF-8 ANSI code page for quoted string values, those strings
  may need code-page-aware decoding.
- `git diff --check` may report CRLF warnings on existing files. Treat non-CRLF whitespace findings as actionable.

## Recently Closed From Source State

- `ExtractIconExW` no longer appears to be synthetic-only: current source reads PE resources, chooses an icon group, builds color/mask bitmaps, creates icon handles, and falls back to shell icons for index zero.
- File-change notification coalescing now handles duplicate records, transient create/delete churn, and modified/delete collapse.
- Destroying a cross-thread `SendMessageTimeout` target now writes a zero result to `lpdwResult` for the completed destroyed-target case.
- System and hidden mounted-volume attributes are now source-backed: nested
  system-volume files inherit `FILE_ATTRIBUTE_SYSTEM`, hidden mounts are skipped
  from root enumeration, exact hidden mount probes keep `FILE_ATTRIBUTE_HIDDEN`,
  and `CeGetVolumeInfoW` exposes system/hidden volume attributes.
