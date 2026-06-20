# Known Bugs And Risks

Refreshed on 2026-06-20. This file lists unresolved behavior only.

## Open Issues

- iNavi route-flow completion remains open beyond process startup, shell
  readiness, real splash rendering, remote input delivery, and initial hidden
  map child-window creation. The latest live sample has no pending WndProc
  returns and remote touch routes to the main iNavi window stack, but the real
  splash artwork still remains visible, so the missing behavior is likely a CE
  wait/event/message/window transition or app readiness signal.
- Scheduler/message semantics are not complete for every reentrant wait,
  cancellation, nested send, abort-if-hung, modal-loop, and cross-thread
  result-delivery combination.
- Shell notification behavior still needs integrated validation for taskbar
  rendering/routing, guest COM callback lifecycle, stale-handle ownership, and
  remove/update races.
- Loader parity is strongest in the Unicorn runtime path. Raw/non-Unicorn
  loader behavior needs explicit audit before all load variants can be treated
  as equivalent.
- Mounted storage remains synthetic for several lower layers beyond covered
  public AFS mount registration: physical block-driver forwarding, external
  cache/filter DLLs, real mounted-volume availability/powerdown, broader ACL
  persistence/enforcement, utility DLL execution, callable FMD callbacks,
  hardware-backed FLS discovery, and lower-FSD `FsIoControl` forwarding.
- GDI/text/IME fidelity is still incomplete for rendered IME UI callbacks,
  caret timing, broader font/style fallback, glyph metrics outside fixture
  ranges, complex clipping, palette/device-color behavior, alpha/mask variants,
  and remaining DIB/bitmap lifetime edges.
- File-change notification reset/error propagation still has deeper edge cases
  beyond the currently covered prefix drains, no-more-items, undersized
  buffers, null-buffer ordering, all-zero no-data reset, and fault-drain cases.
- Popup/menu/modal behavior still needs broader live nested-modal routing,
  unusual cascade cancellation, timeout edges, and user-driven dispatch
  validation.
- Sensor emulation remains partial. GPS/NMEA reaches the configured serial
  receive buffer, but the exact guest `ReadFile`/`WaitCommEvent` cadence,
  parsed-position consumption, and SMB380/G-sensor command contract still need
  real evidence.
- In the latest splash-phase sample, `COM7:` is not opened yet and the only
  device activity is repeated `UID1:` IOCTLs, so sensor data cannot currently
  unblock the visible splash/map transition.
- Startup profiling still reaches the same real guest stop
  (`happyway_win.exe+0x7b56c`) inside traffic/shared-memory initialization; the
  remaining delay is not explained by unreadable MessageBox UI, remote input
  routing, or sensor queuing.
- The stale `happyway_win` escaped visible-message callout no longer remains
  in `pending-wndproc`; keep watching for fresh send-stack or wait-state leaks
  before treating the splash transition as purely app readiness.

## Build And Validation Risks

- The normal validation profile uses `--features unicorn,trace,win32-desktop`.
  No-feature test support needs an explicit decision and cfg audit before it
  can be a required gate.
- `registry.reg` loading accepts REGEDIT text through UTF-8/lossy decoding and
  decodes typed `hex(2)`/`hex(7)` values as UTF-16. Non-UTF-8 ANSI quoted
  strings from future REGEDIT4 exports may need code-page-aware decoding.
- `git diff --check` may report CRLF warnings on existing files. Treat new
  non-CRLF whitespace findings as actionable.

## Recently Closed

- The handle-like `pc=0x00001054` no-hook crash is no longer reproduced in the
  current startup path after blocked scheduler contexts were excluded from
  direct receiver-work eligibility.
- Stale blocked-wait records for the thread that just executed are cleaned up
  at host wall-clock slice stops.
- Stale parked `GetMessage` waiters no longer force short host idle-poll slices
  while active guest CPU work is runnable.
- Remote GPS input drains into the open `COM7:` serial handle in the current
  live path, and remote touch reaches the real iNavi window stack.
- Stale escaped visible-message WndProc callouts no longer block later visible
  input dispatch after the CPU is already parked at their resume PC.
