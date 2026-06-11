# Progress

Regenerated on 2026-06-11 from the current implementation and test surface.

## Current Snapshot

- Runtime loader work has reached dynamic Unicorn DLL mapping with dependency loading, import patching, forwarders, trampoline tracking, datafile/no-resolve flags, and lifecycle calls.
- Shell icon work now includes `ExtractIconExW`, real PE resource icon extraction, PE group-icon count reporting for `nIconIndex == -1`, shell fallback icons, `CreateIconIndirect`, `DrawIconEx`, image lists, bitmap-backed image-list drawing, `xBitmap` offsets, and `rgbBk` fill handling.
- `Shell_NotifyIcon` now tracks add/modify/delete state, rejects duplicate `(hwnd,uID)` adds, honors member `NIF_*` flags on add, keeps the existing icon on `NIM_MODIFY | NIF_ICON` with null `hIcon` per the CE taskbar path, posts callback messages, and records destroy-icon cleanup.
- `SHNotificationUpdateI` now covers CE update-mask behavior for null icon preservation and stale incoming `hwndSink` values while keeping the original registered sink.
- File-change notifications now coalesce exact duplicates, transient create/delete pairs, and modified/delete sequences, and detailed notification records are gated by the CE `FILE_NOTIFY_CHANGE_CEGETINFO` flag while signal-only watches still wake normally.
- GWE message work includes cross-thread send setup, timeout marking, destroyed-window completion, and zero-result writes for destroyed `SendMessageTimeout` targets.
- Winsock has CE-facing dispatch for core socket operations with isolated NAT addressing, `select` fd-set validation, readiness checks, and scheduler wake candidate integration.
- Core CE subsystems remain broad and test-backed: handles, waits, events, TLS, critical sections, registry, files, memory, GDI resources, DIBs, windows, menus, clipboard, and scheduler selection.

## Recent Source-Visible Slices

- `src/ce/coredll.rs`: `ExtractIconExW` reads guest paths, validates files, extracts PE icon resources when available, falls back to shell icons for index zero, writes large/small icon outputs, and supports bitmap-backed icon rendering through `DrawIconEx`.
- `src/ce/kernel.rs`: file-change record append now coalesces pending records and signals only when pending notification data remains.
- `src/ce/kernel.rs`: CE file-notification detail records are only queued for watches created with `FILE_NOTIFY_CHANGE_CEGETINFO`; watches without that flag still signal on matching changes and report no detailed records to `CeGetFileNotificationInfo`.
- `src/ce/gwe.rs` and `src/ce/coredll.rs`: destroyed-window handling exposes completed send-message result writes and flushes them to guest memory.
- `tests/coredll_raw_kernel.rs`: icon extraction, PE group-icon count, shell icon, and image-list drawing coverage is present.
- `tests/coredll_raw_kernel.rs`: `Shell_NotifyIcon` duplicate-add rejection, `NIF_*` member flag handling, and null-icon modify preservation are covered.
- `tests/coredll_raw_kernel.rs`: `SHNotificationAddI` sink-window validation and `SHNotificationUpdateI` stale-sink update behavior are covered.
- `tests/coredll_raw_memory_file.rs`: transient file-change notification churn coverage is present.
- `tests/coredll_raw_memory_file.rs`: signal-only change notifications without `FILE_NOTIFY_CHANGE_CEGETINFO` are covered separately from detailed `CeGetFileNotificationInfo` drains.
- `tests/coredll_raw_gwe.rs`: destroyed-target `SendMessageTimeout` result write coverage is present.
- `src/winsock.rs`: `select` now ignores `nfds` like CE callers expect while validating non-null fd sets, `FD_SETSIZE`, invalid socket handles, and fd-set memory faults before filtering readiness.
- `src/winsock.rs`: Winsock unit coverage now exercises `select` with `nfds` values `0`, `-1`, and active counts, mixed read/write/except fd sets, null fd-set triads, oversized fd sets, invalid socket handles, and `WSAEFAULT` memory failures.
- `src/winsock.rs`: TCP peer close is now read-ready for `select`, allowing the follow-up `recv` to return zero; repeated zero-ready `select` polling is covered by a recovery test that becomes readable after a later datagram.
- `src/winsock.rs`: UDP `recvfrom` coverage now verifies host loopback datagram sources are exposed to CE callers as the isolated gateway address with the original sender port.
- `src/winsock.rs`: TCP half-close coverage now verifies peer write shutdown wakes `select` for a zero-length `recv` while the guest socket can still `send` on its write half.
- `tests/basic_subsystems.rs`: shell notify-icon and file-notification expectations now use explicit CE member/detail flags after the flag-gated notify and `FILE_NOTIFY_CHANGE_CEGETINFO` behavior changes.

## Last Known Validation

- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed for the GWE slice.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file` passed for the file-change slice.
- `cargo test --features unicorn,trace,win32-desktop winsock::tests::select_` passed for the Winsock select validation slice.
- `cargo test --features unicorn,trace,win32-desktop winsock::tests` passed after the TCP peer-close and zero-ready recovery slice.
- `cargo test --features unicorn,trace,win32-desktop winsock::tests` passed after adding UDP source-address validation.
- `cargo test --features unicorn,trace,win32-desktop winsock::tests` passed after adding TCP half-close validation.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems` passed after updating stale test expectations.
- `cargo test --features unicorn,trace,win32-desktop` passed after the Winsock select, test-fix, and TCP peer-close slices.
- `cargo check --features unicorn,trace,win32-desktop` passed after the recent code slices.
- `git diff --check` was clean except for expected CRLF warnings on existing files.

## iNavi Launch Breakthrough (2026-06-11)

- Root cause of the long-standing mounted-iNavi black frame: `CreateWindowExW`'s
  Unicorn callout delivered WM_CREATE to the class wndproc cached before
  WM_NCCREATE. MFC CE subclasses during WM_NCCREATE, so the stale delivery made
  `AfxWndProc` unsubclass the window (`SetWindowLong(GWL_WNDPROC, 0)`), leaving
  `window.wndproc == 0` and every later `DispatchMessageW` a no-op. Fixed by
  re-reading the current wndproc between the two creation messages
  (commit `0ab2e80d`).
- With the fix, mounted iNavi fully launches: renders the 아이나비 SE splash,
  grows to 200+ MB heap, opens 260+ files, loads its runtime DLLs, spawns
  worker threads, and `CreateProcessW`-launches multiple `happyway_win.exe`
  map-engine helpers with command line
  `iNavi|SDMMC Disk\mapdata|SDMMC Disk\inavidata|11|7|0|1` that communicate
  via `CreateFileMappingW`/`MapViewOfFile` shared memory.
- Second blocker fixed (commit `c02998db`): cross-process file-mapping views
  were per-process snapshot copies — the run-end sync let stale copies of
  other processes' views clobber the canonical mapping data, and resuming
  processes never saw peer writes. Views are now process-tagged (owner-only
  capture), every view's lazy-map seed refreshes from canonical data, and
  each run start writes canonical data into the resuming process's mapped
  view pages. With this, happyway helpers complete and exit
  (`CreateProcessExited` appears), the splash artwork renders fully with a
  progress bar advancing, and iNavi keeps dispatching helper workers.
- Current frontier: drive past the splash into the main map UI and first
  map-tile render; then GPS serial (COM21) NMEA feed and touch-driven menus.

## Next Checkpoint

The next useful checkpoint is targeted validation after expanding shell icon/image-list edge coverage or completing the next `SendMessageTimeout` semantics slice.
