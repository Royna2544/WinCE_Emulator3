# Progress

Regenerated on 2026-06-11 from the current implementation and test surface.

## Current Snapshot

- Runtime loader work has reached dynamic Unicorn DLL mapping with dependency loading, import patching, forwarders, trampoline tracking, datafile/no-resolve flags, and lifecycle calls.
- Shell icon work now includes `ExtractIconExW`, real PE resource icon extraction, PE group-icon count reporting for `nIconIndex == -1`, shell fallback icons, `CreateIconIndirect`, `DrawIconEx`, image lists, CE-valid image-list creation/copy flag validation, bitmap-backed image-list drawing, `xBitmap` offsets, `rgbBk` fill handling, and exact CE `IMAGELISTDRAWPARAMS` size validation.
- `Shell_NotifyIcon` now tracks add/modify/delete state, rejects duplicate `(hwnd,uID)` adds, honors member `NIF_*` flags on add, requires the fixed CE `NOTIFYICONDATAW` footprint and readable 64-WCHAR `szTip` buffer, keeps the existing icon on `NIM_MODIFY | NIF_ICON` with null `hIcon` per the CE taskbar path, posts callback messages, and records destroy-icon cleanup.
- `SHNotificationUpdateI` now covers CE update-mask behavior for null icon preservation and stale incoming `hwndSink` values while keeping the original registered sink.
- File-change notifications now coalesce exact duplicates, transient create/delete pairs, and modified/delete sequences, and detailed notification records are gated by the CE `FILE_NOTIFY_CHANGE_CEGETINFO` flag while signal-only watches still wake normally.
- GWE message work includes cross-thread send setup, timeout marking, destroyed-window completion, and zero-result writes for destroyed `SendMessageTimeout` targets.
- Winsock has CE-facing dispatch for core socket operations with isolated NAT addressing, `select` fd-set validation, readiness checks, and scheduler wake candidate integration.
- Core CE subsystems remain broad and test-backed: handles, waits, events, TLS, critical sections, registry, files, memory, GDI resources, DIBs, windows, menus, clipboard, and scheduler selection.

## Recent Source-Visible Slices

- `src/ce/coredll.rs`: `ExtractIconExW` reads guest paths, validates files, extracts PE icon resources when available, enumerates `RT_GROUP_ICON` data through a shared helper for count/extract behavior, falls back to CE-style integer `RT_GROUP_ICON` resource ID lookup for sparse indexes, selects distinct large/small icons from multi-size PE icon groups, fills successive large/small icon output-array slots, reports `ERROR_RESOURCE_NAME_NOT_FOUND` for malformed present PE group/icon resources, falls back to shell icons for non-PE index zero, and supports bitmap-backed icon rendering through `DrawIconEx`.
- `src/ce/coredll.rs`: `DrawIconEx` now scales bitmap-backed icons from their native bitmap dimensions into caller-requested destination rectangles for both framebuffer and selected-memory-DIB targets instead of treating the requested destination size as the source extent, and honors bitmap-backed `DI_MASK` by selecting the icon mask bitmap as the draw source.
- `src/ce/resource.rs`: `ImageList_Create` now applies the CE `ILC_VALID` flag mask, rejecting unsupported creation flags such as `ILC_LARGESMALL`, `ILC_UNIQUE`, and unknown high bits before allocating image-list state.
- `src/ce/resource.rs`: `ImageList_Copy` now applies CE `ILCF_VALID` flag validation, implements `ILCF_SWAP` as an actual image exchange, preserves overlay index mappings during same-list swaps, and treats move-to-self as a no-op instead of deleting the image.
- `src/ce/coredll.rs`: `ImageList_DrawIndirect` now rejects undersized or oversized `IMAGELISTDRAWPARAMS` records before reading optional fields, recording draw state, or rendering, matching CE `imagelist.cpp`'s exact-struct-size gate.
- `src/ce/coredll.rs`: `MessageBoxW` now validates the CE `winuser.h` style surface, accepting CE high flags such as `MB_SETFOREGROUND`, `MB_TOPMOST`, and `MB_RTLREADING` while rejecting unsupported desktop-only bits and undefined icon nibbles before recording dialog state.
- `src/ce/coredll.rs`: `TrackPopupMenuEx` now applies the CE `TPMPARAMS.rcExclude` screen rectangle to top-level popup placement before recording tracking state, painting, and hit-testing.
- `src/ce/coredll.rs`: `SHGetFileInfoW` now writes CE shell `SFGAO_*` attributes for `SHGFI_ATTRIBUTES` instead of raw `FILE_ATTRIBUTE_*` values, covering filesystem, folder, shortcut, and read-only outputs.
- `src/ce/coredll.rs`: `Shell_NotifyIconW` now follows the CE fixed `NOTIFYICONDATAW` contract from `shellapi.h`/`minserver.cpp` by rejecting short `cbSize` values and unreadable `szTip[64]` buffers before updating shell state.
- `src/ce/kernel.rs`: file-change record append now coalesces pending records and signals only when pending notification data remains.
- `src/ce/kernel.rs`: CE file-notification detail records are only queued for watches created with `FILE_NOTIFY_CHANGE_CEGETINFO`; watches without that flag still signal on matching changes and report no detailed records to `CeGetFileNotificationInfo`.
- `src/ce/gwe.rs` and `src/ce/coredll.rs`: destroyed-window handling exposes completed send-message result writes and flushes them to guest memory.
- `tests/coredll_raw_kernel.rs`: icon extraction, PE group-icon count, multi-slot PE `ExtractIconExW` output, multi-size large/small PE icon selection, string-named `RT_GROUP_ICON` extraction, sparse integer `RT_GROUP_ICON` ID extraction, 4bpp and 8bpp indexed PE icon extraction, missing-AND-mask color-only PE icon extraction, malformed PE group/icon failure, shell icon, and image-list drawing coverage is present.
- `tests/coredll_raw_kernel.rs`: `ImageList_Create` now covers rejection of non-CE creation flags and acceptance/preservation of CE-valid private/shared flags.
- `tests/coredll_raw_kernel.rs`: `ImageList_Copy` now covers invalid copy flags, cross-list `ILCF_SWAP`, and move-to-self behavior.
- `tests/coredll_raw_kernel.rs`: `ImageList_DrawIndirect` now covers exact CE `cbSize == sizeof(IMAGELISTDRAWPARAMS)` validation, including no draw-state mutation for short or oversized records.
- `tests/coredll_raw_gwe.rs`: `DrawIconEx` now verifies scaled framebuffer and selected-memory-DIB output from a 2x2 bitmap-backed icon into a 4x4 destination rectangle, verifies `DI_MASK` draws/scales the mask bitmap rather than the color bitmap, and covers a 1bpp mask-only icon draw into a framebuffer.
- `tests/coredll_raw_kernel.rs`: `MessageBoxW` now verifies CE-supported high style bits are preserved and unsupported style/icon bits fail without creating a new shell message-box record.
- `tests/coredll_raw_gwe.rs`: `TrackPopupMenuEx` now verifies that `TPMPARAMS.rcExclude` moves the top-level popup before pointer hit-testing.
- `tests/coredll_raw_kernel.rs`: `SHGetFileInfo` now verifies CE `SFGAO_*` attribute output for regular files, shortcuts, read-only files, storage-card folders, and inaccessible network folders.
- `tests/coredll_raw_kernel.rs`: `Shell_NotifyIcon` duplicate-add rejection, `NIF_*` member flag handling, fixed CE `NOTIFYICONDATAW` size/readability, and null-icon modify preservation are covered.
- `tests/coredll_raw_kernel.rs`: `SHNotificationAddI` sink-window validation and `SHNotificationUpdateI` stale-sink update behavior are covered.
- `tests/coredll_raw_memory_file.rs`: transient file-change notification churn coverage is present.
- `tests/coredll_raw_memory_file.rs`: signal-only change notifications without `FILE_NOTIFY_CHANGE_CEGETINFO` are covered separately from detailed `CeGetFileNotificationInfo` drains.
- `tests/coredll_raw_gwe.rs`: destroyed-target `SendMessageTimeout` result write coverage is present.
- `tests/coredll_raw_gwe.rs`: same-thread `SendMessageTimeout` coverage now verifies synchronous dispatch, direct result-pointer writes, clean last-error state, and no cross-thread sent-message transaction.
- `tests/coredll_raw_gwe.rs`: cross-thread `SendMessageTimeout` coverage now verifies an early receiver `ReplyMessage` result wins over the later wndproc return and writes through the timeout result pointer.
- `tests/coredll_raw_gwe.rs`: nested `SendMessageTimeout` coverage now verifies an inner `ReplyMessage` writes the inner result without clearing or overwriting the still-active outer send.
- `tests/coredll_raw_gwe.rs`: `MsgWaitForMultipleObjectsEx` coverage now verifies `QS_SENDMESSAGE`, CE `QS_ALLINPUT` over posted messages, paint, and timers, `MWMO_INPUTAVAILABLE` new-vs-existing input behavior, and signaled handle precedence over queued sent-message input.
- `src/winsock.rs`: `select` now ignores `nfds` like CE callers expect while validating non-null fd sets, `FD_SETSIZE`, invalid socket handles, and fd-set memory faults before filtering readiness.
- `src/winsock.rs`: Winsock unit coverage now exercises `select` with `nfds` values `0`, `-1`, and active counts, mixed read/write/except fd sets, null fd-set triads, oversized fd sets, invalid socket handles, and `WSAEFAULT` memory failures.
- `src/winsock.rs`: TCP peer close is now read-ready for `select`, allowing the follow-up `recv` to return zero; repeated zero-ready `select` polling is covered by a recovery test that becomes readable after a later datagram.
- `src/winsock.rs`: UDP `recvfrom` coverage now verifies host loopback datagram sources are exposed to CE callers as the isolated gateway address with the original sender port.
- `src/winsock.rs`: TCP half-close coverage now verifies peer write shutdown wakes `select` for a zero-length `recv` while the guest socket can still `send` on its write half.
- `src/winsock.rs`: TCP reset coverage now treats reset sockets as read-ready, caches `WSAECONNRESET` for `SO_ERROR`, and verifies `recv` reports the reset after a host `SO_LINGER(0)` close.
- `src/winsock.rs`: Listener coverage now runs repeated `select`/`accept` cycles, verifies re-arming after each accepted client, and checks accepted loopback peer addresses are exposed as CE gateway addresses.
- `tests/basic_subsystems.rs`: shell notify-icon and file-notification expectations now use explicit CE member/detail flags after the flag-gated notify and `FILE_NOTIFY_CHANGE_CEGETINFO` behavior changes.

## Last Known Validation

- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed after the same-thread, early-`ReplyMessage`, and nested `SendMessageTimeout` slices.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_send_message_timeout` passed after adding same-thread, early-`ReplyMessage`, and nested `SendMessageTimeout` coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_msgwait` passed after adding `QS_SENDMESSAGE`/`MWMO_INPUTAVAILABLE` message-wait coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_track_popup_menu` passed after applying `TPMPARAMS.rcExclude` to popup placement.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed after applying `TPMPARAMS.rcExclude` to popup placement.
- `cargo test --features unicorn,trace,win32-desktop` passed after applying `TPMPARAMS.rcExclude` to popup placement.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel message_box_w` passed after the CE `MessageBoxW` style-mask validation slice.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel` passed after the CE `MessageBoxW` style-mask validation slice.
- `cargo check --features unicorn,trace,win32-desktop` passed after the CE `MessageBoxW` style-mask validation slice.
- `cargo test --features unicorn,trace,win32-desktop` passed after the CE `MessageBoxW` style-mask validation slice.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel shell_notify_icon` passed after the CE fixed-`NOTIFYICONDATAW` `Shell_NotifyIconW` slice.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel` passed after the CE fixed-`NOTIFYICONDATAW` `Shell_NotifyIconW` slice.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel sh_get_file_info` passed after the CE `SFGAO_*` `SHGetFileInfoW` attribute slice.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel sh_get_file_info` passed after adding multi-slot PE `ExtractIconExW` extraction coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel sh_get_file_info` passed after adding multi-size large/small PE `ExtractIconExW` selection coverage.
- `cargo check --features unicorn,trace,win32-desktop` passed after unblocking the feature build around displaced blocked-`GetMessage` state.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel sh_get_file_info` passed after adding malformed present PE group/icon `ExtractIconExW` failure coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel sh_get_file_info` passed after adding string-named `RT_GROUP_ICON` `ExtractIconExW` coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel sh_get_file_info` passed after adding missing-AND-mask `RT_ICON` `ExtractIconExW` coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel sh_get_file_info` passed after adding 8bpp indexed `RT_ICON` `ExtractIconExW` coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel sh_get_file_info` passed after adding 4bpp indexed `RT_ICON` extraction and render coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel sh_get_file_info` passed after adding sparse integer `RT_GROUP_ICON` ID lookup coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_draw_indirect` passed after enforcing exact `IMAGELISTDRAWPARAMS` size validation.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after enforcing CE `ILC_VALID` image-list creation flags.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_copy_honors_ce_move_swap_flags` passed after enforcing CE `ILCF_VALID` image-list copy flags.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_destroy_icon_accepts_loaded_icon_handles` passed after adding bitmap-backed `DrawIconEx` stretched selected-memory-DIB coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_destroy_icon_accepts_loaded_icon_handles` passed after adding bitmap-backed `DrawIconEx` `DI_MASK` source selection coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_destroy_icon_accepts_loaded_icon_handles` passed after adding bitmap-backed `DrawIconEx` framebuffer stretched-output coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_destroy_icon_accepts_loaded_icon_handles` passed after adding 1bpp mask-only framebuffer `DrawIconEx` coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel` passed after adding malformed present PE group/icon `ExtractIconExW` failure coverage.
- `cargo test --features unicorn,trace,win32-desktop` passed after fixing the feature-enabled Unicorn test helper call sites and adding malformed present PE group/icon `ExtractIconExW` failure coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel` passed after adding multi-size large/small PE `ExtractIconExW` selection coverage.
- `cargo check --features unicorn,trace,win32-desktop` passed after adding multi-size large/small PE `ExtractIconExW` selection coverage.
- `cargo test --features unicorn,trace,win32-desktop` passed after adding multi-size large/small PE `ExtractIconExW` selection coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel` passed after adding multi-slot PE `ExtractIconExW` extraction coverage.
- `cargo check --features unicorn,trace,win32-desktop` passed after adding multi-slot PE `ExtractIconExW` extraction coverage.
- `cargo test --features unicorn,trace,win32-desktop` passed after adding multi-slot PE `ExtractIconExW` extraction coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel` passed after the CE `SFGAO_*` `SHGetFileInfoW` attribute slice.
- `cargo test --features unicorn,trace,win32-desktop` passed after the CE `SFGAO_*` `SHGetFileInfoW` attribute slice.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed with the `QS_ALLINPUT` post/paint/timer and multi-handle message-wait additions.
- `cargo test --features unicorn,trace,win32-desktop` passed after the CE fixed-`NOTIFYICONDATAW` shell notify slice and the message-wait coverage additions.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file` passed for the file-change slice.
- `cargo test --features unicorn,trace,win32-desktop winsock::tests::select_` passed for the Winsock select validation slice.
- `cargo test --features unicorn,trace,win32-desktop winsock::tests` passed after the TCP peer-close and zero-ready recovery slice.
- `cargo test --features unicorn,trace,win32-desktop winsock::tests` passed after adding UDP source-address validation.
- `cargo test --features unicorn,trace,win32-desktop winsock::tests` passed after adding TCP half-close validation.
- `cargo test --features unicorn,trace,win32-desktop winsock::tests` passed after adding TCP reset and repeated listener accept-loop coverage.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems` passed after updating stale test expectations.
- `cargo test --features unicorn,trace,win32-desktop` passed after the Winsock select, test-fix, TCP peer-close, TCP reset, and repeated accept-loop slices.
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
- Third blocker fixed (commit `4a9865d0`): live-pump remote-drain stops and
  wall-clock slice stops called `emu_stop` at arbitrary instructions; landing
  on a MIPS branch delay slot lost the branch on resume and the guest jumped
  wild (pc=0x3548), killing 2 of 3 remote-server runs in the first minute.
  Stops now defer to the next safe pc (not a delay slot, not in a trampoline,
  below the import-trap region) like the scheduler timeslice hook already did;
  3 of 3 post-fix runs reach the healthy startup profile.
- Repeated-run regression evidence: single-shot runs advance much further than
  remote-server (live-pump) runs in the same wall time — a 150 s single shot
  reached 868 file opens and a 10-thread event-pair worker pool past the old
  264-open plateau, while live-pump runs sit in the helper spawn phase. Each
  blocked-GetMessage exit in live-pump mode re-enters a fresh Unicorn (~400 MB
  blob copy round trip plus full translation rebuild, ~1/s), so live-pump is
  an order of magnitude slower, not stuck. Host-desktop mode uses 120 s slices
  (`HOST_LIVE_RUN_SLICE_MS`) and is the better interactive driving path until
  live-pump keeps its Unicorn/TB state across slices.
- MAP UI MILESTONE: a 10-minute single-shot run
  (`--cpu-wall-clock-limit-ms 600000`, framebuffer dump
  `target/inavi_fb_long.ppm`) reached the full iNavi navigation screen —
  street map tiles with Korean street/POI labels, POI icons, compass, zoom
  controls, clock, and the bottom info bar all render. The app fully boots
  from splash through map-engine startup to the live map with no
  must-implement stub hits.
- BOOT TIME: the 10-minute figure was a dev-profile build. The release build
  boots cold start to the fully rendered map UI in about 30 seconds. Two
  flamegraph-driven fixes (cargo flamegraph under Windows sudo, ETW): the
  undocumented full TB-cache flush every 0x40000 instructions made QEMU
  re-translation (`tb_gen_code_mipsel`, 5.2% of system samples vs <1% for the
  hook callback itself) dominate guest execution — removed (commit
  `66a2f151`, identical boot trajectory and framebuffer, suite green); and
  the trampoline patcher's per-word linear range scan (2.5% of samples) now
  binary-searches merged sorted ranges (commit `4f4917d7`). Release profile
  keeps debug symbols for future profiling.
- Current frontier: interactive driving — touch input needs the remote
  server (live-pump), which is an order of magnitude slower per guest second
  than single-shot (see above). Either keep the Unicorn/TB state alive across
  live-pump slices or use host-desktop mode (120 s slices) for interactive
  sessions; then GPS serial (COM21) NMEA feed for live positioning.

## Next Checkpoint

The next useful checkpoint is targeted validation after expanding shell icon/image-list edge coverage or completing the next `SendMessageTimeout` semantics slice.

## UI Crawl Findings (2026-06-11, 50-step tap crawl)

- A 50-step automated tap crawl over the live map UI discovered 23 unique UI
  states with the emulator healthy throughout: map-browse mode with selection
  cursor and register/nearby/origin/destination action bar, menu screens,
  destination pages, pan/zoom states; identical taps reproduce identical
  states across rounds.
- Real app-level finding: opening 주변검색 (nearby search) raises iNavi's
  G-sensor error modal ("G센서 초기화 ... GPS로 자동 전환 [Error Code: -15]")
  — the emulator's IMU/G-sensor device does not answer initialization like
  real hardware. The modal then never dismisses on its OK button even though
  touches deliver to the view window (down/up posted at the button), which
  suggests the dialog's owner thread is blocked on the sensor device and
  never pumps the tap. Next root cause: DenebSensor/IMU device emulation
  (serial_devices.json device behavior), then re-crawl the nearby-search and
  remaining menu paths.
