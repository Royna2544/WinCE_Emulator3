# TODO

Regenerated on 2026-06-11 from current source and test coverage.

## Immediate Engineering Queue

- Add PE icon extraction edge tests for remaining PE resource variants and non-PE fallback paths; raw `KernExtractIcons` exact integer `RT_GROUP_ICON` lookup and no-output failure behavior are now covered.
- Expand `DrawIconEx`, `ImageList_DrawEx`, and `ImageList_DrawIndirect` tests for remaining mask formats, palette variants, additional overlay rendering variants, stretched draws, framebuffer output, and additional cleanup/error lifetimes; the CE all-white overlay-mask zero-bounds and non-rectangular overlay-mask branches are now covered, and raw `AlphaBlend` now covers negative framebuffer destination clipping.
- Implement and test remaining `SendMessageTimeout` semantics: broader reentrant waits, additional cross-thread `SMTO_BLOCK`/`SMTO_ABORTIFHUNG` edge combinations, and multi-wait interactions beyond the covered below-threshold/hung abort-if-hung split, same-thread abort-if-hung non-abort, early/nested `ReplyMessage`, and active-outer-timeout nested-send paths.
- Extend remaining popup/menu placement coverage beyond the covered `TPMPARAMS.rcExclude` below-placement, right-side fallback, left-side fallback, and all-candidates-intersect fallback cases, especially live nested modal routing and unusual clamped-screen combinations.
- Verify `Shell_NotifyIcon` and `SHNotification*` in an integrated Unicorn run, including Explorer/taskbar `IShellNotificationCallback` dispatch through a real guest COM object lifecycle, broader remove/update race behavior, and runtime invalid-handle behavior beyond the raw stale-taskbar, raw `SHNN_SHOW` window-only, GUID-value OLE `CoCreateInstance` import writeback, local callback-acquisition queue, and mapped vtable callout paths.
- Extend `MessageBoxW` live modal coverage beyond raw default/queued input and the covered `MB_TOPMOST` transient `WS_EX_TOPMOST` style, especially foreground/topmost z-order behavior with live windows and timeout/pump edges.
- Extend file-change notification coverage across internal-vs-external FSDMGR notification handle behavior and remaining volume-handle owner/reset/close edges.
- Audit runtime loader behavior outside Unicorn so raw tests and runtime mapping behavior stay intentionally aligned.
- Continue iNavi route-flow work from the current process/window/shell readiness point into destination search and map interaction.
- Re-run the iNavi UI crawl on a host/remote launch after fixing the detached
  remote-server listener lifetime; validate that the new target-thread wake path
  lets top-right menu/search taps reach and advance the live iNavi UI.
- Continue the remote-driven iNavi startup investigation from the current
  post-dialog state: startup reaches map-data loading, records `end_dialog` for
  the Happyway `MessageBoxW`, and now clears orphaned cross-process
  `SendMessageW` yield snapshots once their live sent-message record is gone.
  Re-run the driven startup to verify it resumes through the parked sender and
  identify the next live blocker.
- Fix exposed-region repaint after destroying modal overlays without inventing
  pixels: the dialog subtree is dead and input routes to iNavi, but the old
  dialog framebuffer pixels remain until the underlying iNavi window processes
  its pending paint.
- Decode the SMB380/G-sensor initialization contract from actual dump code or a
  real caller trace before adding accelerometer commands; do not use the
  unverified `0xb100...` family as SMB380 evidence.

## Cleanup Queue

- Keep registry fixtures and launch docs on `registry.reg`; any explicit
  legacy JSON parser regression should create a temporary JSON fixture instead
  of restoring the removed `regs.json`.
- Decide whether no-feature `cargo test` is a supported build profile. If yes, audit feature-gated Unicorn references and add it to validation.
- Keep `SOURCE_REFERENCES.md` tied to actual source behavior when implementing new slices.
- Avoid reintroducing long historical progress logs into roadmap files; use these files for current state only.

## Validation Queue

- For shell and GDI work: run `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel`.
- For GWE/message work: run `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe`.
- For file/storage work: run `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`.
- For shared scheduler/resource work: run `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems`.
- Before handing off a larger slice: run `cargo check --features unicorn,trace,win32-desktop` and `git diff --check`.
