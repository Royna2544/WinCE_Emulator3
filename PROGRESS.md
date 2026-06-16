# Progress

Regenerated on 2026-06-11 from the current implementation and test surface.

## Current Snapshot

- Runtime loader work has reached dynamic Unicorn DLL mapping with dependency loading, dependency-ref release on unload, current-image cleanup on failed maps, transactional current-image resource/trap/trampoline commit ordering, import patching, forwarders, malformed-forwarder rejection, failed-load and failed-attach rollback for load-attempt refs, trampoline tracking, datafile/no-resolve flags, raw `LOAD_WITH_ALTERED_SEARCH_PATH` loaded-module reuse, datafile export suppression, CE-style process/thread lifecycle calls, process-detach refcount draining, and CE `DisableThreadLibraryCalls` filtering.
- Shell icon work now includes `ExtractIconExW`, real PE resource icon extraction, PE group-icon count reporting for `nIconIndex == -1`, raw `KernExtractIcons` integer group-resource extraction, shell fallback icons, `CreateIconIndirect`, `DrawIconEx`, image lists, CE-valid image-list creation/copy flag validation, CE-style image-list bitmap copy/lifetime handling, bitmap-backed image-list drawing, `xBitmap` offsets, `rgbBk` fill handling, CE color-image vs mask-only `rgbFg == CLR_NONE` blend behavior, and exact CE `IMAGELISTDRAWPARAMS` size plus normalized-field write-back validation.
- `Shell_NotifyIcon` now tracks add/modify/delete state, rejects duplicate `(hwnd,uID)` adds, honors member `NIF_*` flags on add, requires the fixed CE `NOTIFYICONDATAW` footprint and readable 64-WCHAR `szTip` buffer, keeps the existing icon on `NIM_MODIFY | NIF_ICON` with null `hIcon` per the CE taskbar path, posts callback messages, records destroy-icon cleanup, tracks registered taskbar HWND state, posts CE `WM_HANDLESHELLNOTIFYICON` taskbar messages with copied `NOTIFYICONDATAW` payloads, releases those copied payloads after dispatch, treats stale registered taskbar windows as a non-posting shell-state update success, and allows `NIM_DELETE` to remove an existing tray record after the app owner HWND has gone stale while preserving stale add/modify failures.
- `SHNotificationUpdateI` now covers CE update-mask behavior for null icon preservation, non-null icon replacement cleanup, stale incoming `hwndSink` values while keeping the original registered sink, inform/iconic priority-list movement, overlong taskbar-title clearing through the CE `CCHMAXTBLABEL` storage rule, and second-based `csDuration` expiration; iconic notification expiration, explicit remove, and sink cleanup now record copied icon destruction like the CE taskbar cleanup paths, notification remove and sink cleanup now purge pending callback records for removed notifications, and `SHNotificationGetDataI` accepts the CE fixed-title-buffer path when `cbTitle == 0`.
- File-change notifications now canonicalize public watch paths, preserve caller notification filter bits while CE-style known-bit matching decides whether changes signal, honor root and non-root `WatchSubtree` boundaries, map same-parent vs cross-parent move notifications through CE `NotifyMoveFileEx` action semantics, coalesce exact duplicates, transient create/delete pairs, and modified/delete sequences, track CE-style outstanding notification signals across `FindNextChangeNotification`, and gate detailed notification records by the CE `FILE_NOTIFY_CHANGE_CEGETINFO` flag while signal-only watches still wake normally. `CeGetFileNotificationInfo` record sizing now follows CE `NotifyReset`, including the copied trailing NUL WCHAR and DWORD padding while preserving non-NUL `FileNameLength`, and its no-pending path now preserves CE's guarded output-pointer write order while returning `ERROR_NO_MORE_ITEMS`. Mounted file operations now enforce CE volume boundaries and read-only root access checks for mutating calls, with access-denied read-only mutations leaving watchers unsignaled. Mounted change notifications now retain the resolved owning mount root, so non-root watches are scoped to their CE-style volume while recursive root watchers still report mounted-volume-prefixed child paths. Raw `DuplicateHandle` now creates independent local handles for notification/file/find objects and supports the CE `DUPLICATE_CLOSE_SOURCE` ownership-transfer shape used by notification close paths. Public file-change notification handles now track their creating process and reject foreign-process wait/reset/info/duplicate/close attempts, direct `AFS_FindFirstChangeNotificationW` now honors its nonzero `hProc` owner, and raw AFS volume handles now enforce owner-checked unmount/close plus `FSCTL_GET_VOLUME_INFO` metadata while signaling mounted-root removal.
- GWE message work includes cross-thread send setup, timeout marking, CE-public `SMTO_NORMAL` timeout-send completion, destroyed-window completion, and zero-result writes for destroyed `SendMessageTimeout` targets.
- Winsock has CE-facing dispatch for core socket operations with isolated NAT addressing, `select` fd-set validation, readiness checks, and scheduler wake candidate integration.
- Core CE subsystems remain broad and test-backed: handles, waits, events, TLS, critical sections, registry, files, memory, DPA/DSA containers, GDI resources, DIBs, windows, menus, clipboard, and scheduler selection.

## Recent Source-Visible Slices

- `src/ce/coredll.rs` and `src/emulator/imports.rs`: direct
  `FSDMGR_DiskIoControl @12` now recognizes CE `fmd.h`
  `IOCTL_FMD_GET_RESERVED_TABLE`, `IOCTL_FMD_GET_RAW_BLOCK_SIZE`, and
  `IOCTL_FMD_GET_INFO` controls. The synthetic disk reports an empty reserved
  table, returns the current synthetic block size, validates FMD metadata
  buffers, and fills deterministic NOR-style `FMDInfo` metadata while leaving
  hardware flash lock/write/reserved-region behavior queued.
- Validation after the FMD metadata IOCTL slice: focused
  `cargo test -j 1 --features unicorn,trace,win32-desktop
  emulator::imports::tests::fsdmgr_disk_support_imports_round_trip_sparse_sectors_and_info`,
  `cargo check --features unicorn,trace,win32-desktop`, and full
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed. Logs were
  kept under `target/`.
- `src/emulator/unicorn.rs`: mapped-code indexing now keeps lightweight
  references to mapped blob byte buffers instead of cloning every blob into each
  index. The index still keeps immutable static-code snapshots authoritative
  for image/DLL/trampoline reads while using live Unicorn memory for mutable
  trap and heap-spillover code.
- Validation after the mapped-code index storage slice: focused
  `cargo test -j 1 --features unicorn,trace,win32-desktop mapped_code_index`,
  `cargo check --features unicorn,trace,win32-desktop`, and full
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed. Logs were
  kept under `target/`.
- `src/ce/file.rs`, `src/ce/kernel.rs`, `src/ce/coredll.rs`, and
  `tests/coredll_raw_memory_file.rs`: raw coredll `LockFileEx @1968` and
  `UnlockFileEx @1969` now follow the CE `LOCKMGR` range model for host-backed
  files. The emulator validates non-null/readable 20-byte `OVERLAPPED`
  payloads, rejects zero or wrapping ranges, allows overlapping shared locks,
  rejects overlapping exclusive/shared conflicts with `ERROR_LOCK_VIOLATION`,
  requires exact owner/range unlock, and drops a handle's owned locks on close.
  Remaining file-lock parity is CE-style blocking wait queues for non-immediate
  conflicts plus lower-FSD/filter forwarding.
- Validation after the CE file-lock range slice: `cargo fmt --check`, `cargo
  check --features unicorn,trace,win32-desktop`, `git diff --check`, focused
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_memory_file
  coredll_raw_lock_file_ex_validates_file_handle_and_overlapped`, and full
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_memory_file`, followed by full
  `cargo test -j 1 --features unicorn,trace,win32-desktop`. Logs were kept
  under `target/`.
- `src/emulator/unicorn.rs`: Unicorn mapped-code fallback now indexes every
  mapped blob on a page instead of the first one only, prefers immutable
  static-code snapshots for image/DLL/trampoline instruction reads, and keeps
  live Unicorn memory authoritative for mutable trap and heap-spillover code.
- `tests/coredll_raw_kernel.rs`, `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and
  `SOURCE_REFERENCES.md`: raw `ImageList_GetIcon` lifetime coverage now
  verifies that the returned icon-owned mask/color bitmaps survive source
  `ImageList_Destroy`, remain visible through `GetIconInfo`, and are released
  by `DestroyIcon`, matching CE `imagelist.cpp::GetIcon` temporary-bitmap
  cleanup around `CreateIconIndirect_I`.
- Full validation after the mapped-code and `ImageList_GetIcon` lifetime
  slices: `cargo fmt --check`, `git diff --check`, `cargo test -j 1
  --features unicorn,trace,win32-desktop mapped_code_index`, `cargo test -j 1
  --features unicorn,trace,win32-desktop --test coredll_raw_kernel
  image_list_ordinals_track_created_lists_and_icons`, `cargo check --features
  unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_kernel`, and `cargo test -j
  1 --features unicorn,trace,win32-desktop` passed. Logs were written under
  `target/`.
- `src/ce/coredll.rs`, `tests/coredll_raw_kernel.rs`, `PLAN.MD`,
  `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: bitmap-backed
  `ImageList_GetIcon` now mirrors CE `imagelist.cpp::GetIcon` by initializing
  its temporary mono mask bitmap to white before drawing, so unmasked image
  lists produce an all-white returned icon mask instead of deriving mask bits
  from the color image.
- Focused validation after the unmasked `ImageList_GetIcon` mask slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons --
  --nocapture` passed.
- Full validation after the unmasked `ImageList_GetIcon` mask slice:
  `cargo fmt --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_kernel`, and `cargo test -j
  1 --features unicorn,trace,win32-desktop` passed.
- `tests/coredll_raw_kernel.rs`, `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`,
  and `SOURCE_REFERENCES.md`: raw bitmap-backed `ImageList_GetIcon` now has a
  `GetIconInfo` consumer regression. The returned `ICONINFO` reports `fIcon`,
  exposes the rendered icon-owned mask/color bitmap handles, and does not leak
  the source image-list backing handles.
- Focused validation after the `ImageList_GetIcon`/`GetIconInfo` bridge slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons --
  --nocapture` passed.
- Full validation after the `ImageList_GetIcon`/`GetIconInfo` bridge slice:
  `cargo fmt --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_kernel`, and `cargo test -j
  1 --features unicorn,trace,win32-desktop` passed.
- `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`,
  and `SOURCE_REFERENCES.md`: raw `GetIconInfo` lifetime coverage now verifies
  that `CreateIconIndirect` returns cloned mask/color bitmap handles through
  `ICONINFO`, caller-owned source bitmaps can be deleted, and the icon still
  draws from its cloned backing afterward.
- Focused validation after the raw `GetIconInfo` caller-bitmap lifetime slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_destroy_icon_accepts_loaded_icon_handles --
  --nocapture` passed.
- Full validation after the raw `GetIconInfo` caller-bitmap lifetime slice:
  `cargo fmt --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and `cargo test -j
  1 --features unicorn,trace,win32-desktop` passed.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`,
  `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: raw
  `GetIconInfo` now reports CE `ICONINFO` cursor/icon state from real tracked
  icon objects, including hotspot and mask/color bitmap handles, treats stock
  cursor pseudo handles as `fIcon == FALSE`, preserves stock icon pseudo
  handles as `fIcon == TRUE`, and rejects invalid non-icon handles with
  `ERROR_INVALID_HANDLE`.
- Focused validation after the raw `GetIconInfo` cursor/icon slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_def_window_proc_setcursor_uses_class_cursor --
  --nocapture` passed.
- Full validation after the raw `GetIconInfo` cursor/icon slice:
  `cargo fmt --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and `cargo test -j
  1 --features unicorn,trace,win32-desktop` passed.
- `src/ce/coredll.rs`, `tests/coredll_raw_kernel.rs`, `PLAN.MD`, and
  `SOURCE_REFERENCES.md`: raw `SHGetFileInfoW` now follows the CE
  `CESHELL\API\api.cpp` field-write contract. Requests with `SHGFI_ICON`
  populate both `hIcon` and `iIcon` even without `SHGFI_SYSICONINDEX`, while
  calls that omit `SHGFI_ICON`, `SHGFI_SYSICONINDEX`, or `SHGFI_ATTRIBUTES`
  leave the corresponding `SHFILEINFO` fields untouched instead of clearing
  them first.
- Validation after the raw `SHGetFileInfoW` field-write slice: focused
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_kernel sh_get_file_info_uses_registry_associations_and_attributes
  -- --nocapture` passed.
- `src/ce/coredll.rs`, `tests/coredll_raw_kernel.rs`, `PLAN.MD`,
  `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: raw
  `KernExtractIcons` now mirrors the CE `resource.cpp` distinction between
  absent integer `RT_GROUP_ICON` resources and missing selected `RT_ICON`
  payloads. The raw path resolves the group first without touching outputs on
  absent-group failure, then extracts requested large/small slots
  independently so a missing peer icon does not prevent a requested success;
  failed requested slots are assigned NULL and leave
  `ERROR_RESOURCE_NAME_NOT_FOUND` visible when another slot succeeds.
- Validation after the raw `KernExtractIcons` partial-output slice: focused
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_kernel
  coredll_raw_kern_extract_icons_copies_group_rt_icon_payloads --
  --nocapture` passed.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: raw `MapVirtualKeyW` now
  follows the CE PC keyboard driver split between public scan-code modes:
  `MAPVK_VK_TO_VSC` masks side-specific right Ctrl/Alt scans to the low byte,
  `MAPVK_VSC_TO_VK` collapses LR modifier VKs back to common
  `VK_SHIFT`/`VK_CONTROL`/`VK_MENU`, `MAPVK_VSC_TO_VK_EX` preserves
  side-specific VKs, and unsupported map types set `ERROR_INVALID_PARAMETER`.
- Validation after the CE `MapVirtualKeyW` scan-code slice: `cargo fmt`,
  focused default-feature `cargo test --test coredll_raw_gwe
  coredll_raw_map_virtual_key_scan_code_modes_follow_ce_driver --
  --nocapture`, and focused `cargo test --features unicorn --test
  coredll_raw_gwe coredll_raw_map_virtual_key_scan_code_modes_follow_ce_driver
  -- --nocapture` passed. Full-feature `cargo check --features
  unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full `cargo test
  -j 1 --features unicorn,trace,win32-desktop` also passed. Logs are under
  `target/map_virtual_key_scan_modes_default.*.log`,
  `target/map_virtual_key_scan_modes_unicorn.*.log`,
  `target/cargo-check-features-mapvirtualkey-20260616-154631.*.log`,
  `target/cargo-test-coredll-raw-gwe-mapvirtualkey-20260616-154648.*.log`,
  and `target/cargo-test-full-features-mapvirtualkey-20260616-154701.*.log`.
- `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and
  `SOURCE_REFERENCES.md`: raw `MsgWaitForMultipleObjectsEx` coverage now locks
  CE invalid handle-array shapes for `nCount > MAXIMUM_WAIT_OBJECTS` and
  nonzero count with null `pHandles`, preserving `WAIT_FAILED`,
  `ERROR_INVALID_PARAMETER`, message-wait failure telemetry, and no hidden
  `WaitForMultipleObjects` probe.
- Validation after the CE message-wait invalid-array slice: `cargo fmt`,
  focused `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_msgwait_rejects_invalid_ce_handle_array_shapes
  -- --nocapture`, `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, and full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. Logs are under
  `target/cargo-test-msgwait-invalid-arrays-20260616-153257.*.log`,
  `target/cargo-check-features-msgwait-20260616-153314.*.log`,
  `target/cargo-test-coredll-raw-gwe-msgwait-20260616-153321.*.log`, and
  `target/cargo-test-full-features-msgwait-20260616-153329.*.log`.
- `src/ce/coredll.rs`, `tests/coredll_raw_kernel.rs`, `PLAN.MD`,
  `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: raw
  `GetFileVersionInfoSizeW`/`GetFileVersionInfoW` now follow CE
  `resource.cpp` by extracting integer `RT_VERSION/VS_VERSION_INFO` resources
  from PE files, validating `VS_FFI_SIGNATURE`, clearing the size API handle
  out-param, returning `ERROR_INVALID_DATA` for malformed version blocks, and
  rewriting copied `VERHEAD.wTotLen` to the bounded caller copy length.
- Validation after the CE version-resource slice: `cargo fmt --check`,
  focused `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_kernel
  coredll_raw_get_file_version_info_reads_ce_version_resource -- --nocapture`,
  `cargo check --features unicorn,trace,win32-desktop`, `cargo test -j 1
  --features unicorn,trace,win32-desktop --test coredll_raw_kernel`, full
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed. Logs are under
  `target/cargo-test-file-version-info-20260616-152613.*.log`,
  `target/cargo-check-features-version-info-20260616-152734.*.log`,
  `target/cargo-test-coredll-raw-kernel-version-info-20260616-152743.*.log`,
  and `target/cargo-test-full-features-version-info-20260616-152751.*.log`;
  `git diff --check` output was limited to CRLF normalization warnings.
- `src/ce/coredll.rs`, `src/emulator/unicorn.rs`, `tests/support/mod.rs`, and
  `tests/coredll_raw_gwe.rs`: DeviceEmulator `GETRAWFRAMEBUFFER` now maps the
  returned `IMAGE_FRAMEBUFFER_UA_BASE` range when needed and writes a
  guest-readable RGB565 snapshot behind `pFramePointer`. The raw regression
  checks the CE `RawFrameBufferInfo` metadata and then reads the live framebuffer
  bytes back from the returned pointer.
- Validation after the raw framebuffer pointer slice: `cargo fmt`, focused
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe
  coredll_raw_ext_escape_matches_ce_query_and_protected_escape_edges -- --nocapture`,
  `cargo check --features unicorn,trace,win32-desktop`, `cargo test -j 1
  --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed. Logs are
  under `target/cargo-test-extescape-rawfb-20260616-151318.*.log`,
  `target/cargo-check-features-20260616-151359.*.log`,
  `target/cargo-test-coredll-raw-gwe-20260616-151407.*.log`, and
  `target/cargo-test-full-features-20260616-151416.*.log`.
- `src/emulator/unicorn.rs`: the direct-send WNDPROC orphan cleanup now avoids
  mapped-blob module lookups unless the current frame is already a plausible
  nested WNDPROC frame, and it skips those lookups entirely for the existing
  direct `SendMessageW` no-restore acceptance path. A sudo cargo flamegraph of
  iNavi startup showed the slowdown concentrated in Unicorn code hooks and this
  cleanup path, not host SD-card file I/O.
- `src/emulator/unicorn.rs` and `src/main.rs`: remote input draining can now
  rotate from the currently active guest thread to a suspended target thread
  with visible receiver work, preserving the active saved CPU context as a
  persisted suspended thread for later resume. This covers the active-thread
  variant of the iNavi visible-message handoff instead of only waking already
  parked send/wait threads.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `TransparentImage` now follows CE `wingdi.h`/`draw.cpp::TransparentBltBitmapTest`
  by accepting a direct bitmap `HANDLE` as `hSrc` in addition to HDC sources.
  The bitmap-handle route reuses the color-key blit path and keeps DISPPERF
  source-video-memory accounting clear for system-memory bitmap sources.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: selecting a pointerless
  `CreateBitmap(..., NULL)` bitmap into a compatible memory DC now allocates
  owned zeroed backing, so CE `draw.cpp::TransparentBltTransparencyTest`-style
  `FillRect`/`SetPixel` drawing into the offscreen bitmap becomes visible to a
  later `TransparentImage` color-key copy.
- `tests/coredll_raw_gwe.rs` and `tests/support/mod.rs`: the same
  `TransparentBltTransparencyTest` regression now sweeps CE's raw
  `CreateBitmap` bit depths (`1`, `2`, `4`, `8`, `16`, `24`, and `32` bpp).
  The test memory backend now keeps repeated halfword writes visible through
  byte snapshots, matching the `TransparentImage` source-bitmap read path.
- `tests/coredll_raw_gwe.rs`: raw same-framebuffer `TransparentImage` now covers
  CE `draw.cpp::TransparentBltErrorTest` near-miss transparent keys. A solid
  nonzero RGB source remains opaque when the transparent key matches only one
  or two color channels, preserving the CE screen-half comparison behavior.
- `tests/coredll_raw_gwe.rs`: raw selected-DIB `TransparentImage` now has CE
  `draw.cpp::ClipBitBlt(ETransparentImage)` off-left clipping coverage. The
  regression proves that destination clipping advances the source sample and
  that a clipped-in transparent source pixel still preserves the destination.
- `tests/coredll_raw_gwe.rs`: the same CE
  `draw.cpp::ClipBitBlt(ETransparentImage)` off-left source-alignment case now
  covers same-framebuffer HDC copies, proving the framebuffer fallback snapshots
  source pixels before writing and preserves non-overlapped source pixels.
- `tests/coredll_raw_gwe.rs`: raw selected-DIB and same-framebuffer
  `TransparentImage` now cover CE `draw.cpp::ClipBitBlt(ETransparentImage)`
  off-top source alignment, proving top-edge destination clipping advances
  source rows and transparent sampled rows preserve the destination.
- `tests/coredll_raw_gwe.rs`: raw selected-DIB and same-framebuffer
  `TransparentImage` now cover the remaining CE
  `draw.cpp::ClipBitBlt(ETransparentImage)` right and bottom destination
  clipping edges, proving the visible tail is trimmed while transparent samples
  inside the clipped area still preserve the destination.
- `tests/coredll_raw_gwe.rs`: raw selected-DIB and framebuffer
  `TransparentImage` now cover the CE
  `draw.cpp::ClipBitBlt(ETransparentImage)` simultaneous top-left corner
  clipping case. The regressions prove destination clipping advances both the
  source column and row together while preserving transparent pixels in the
  clipped-in corner.
- Focused validation after the `TransparentImage` top-left corner clipping
  slice: `cargo test --features unicorn,trace,win32-desktop
  coredll_raw_transparent_image_clips_off_top_left -- --nocapture` passed,
  followed by `cargo fmt --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full `cargo test -j
  1 --features unicorn,trace,win32-desktop`. Test logs are under
  `target/cargo-test-coredll-raw-gwe-transparent-corner-clip.20260616-141232.*.log`
  and `target/cargo-test-full-transparent-corner-clip.20260616-141239.*.log`.
- `tests/coredll_raw_gwe.rs`: raw selected-DIB and framebuffer
  `TransparentImage` now cover the CE
  `draw.cpp::ClipBitBlt(ETransparentImage)` simultaneous bottom-right corner
  clipping case. The regressions prove trailing-edge clipping trims both axes
  together while preserving a transparent source pixel at the clipped-in corner.
- Focused validation after the `TransparentImage` bottom-right corner clipping
  slice: `cargo test --features unicorn,trace,win32-desktop
  coredll_raw_transparent_image_clips_off_bottom_right -- --nocapture` passed,
  followed by `cargo fmt --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full `cargo test -j
  1 --features unicorn,trace,win32-desktop`. Test logs are under
  `target/cargo-test-coredll-raw-gwe-transparent-bottom-right-clip.20260616-141627.*.log`
  and `target/cargo-test-full-transparent-bottom-right-clip.20260616-141635.*.log`.
- `tests/coredll_raw_gwe.rs`: raw selected-DIB and framebuffer
  `TransparentImage` now cover the remaining CE
  `draw.cpp::ClipBitBlt(ETransparentImage)` two-edge corner combinations:
  top-right and bottom-left. Together with the existing top-left and
  bottom-right regressions, the corner matrix proves leading-edge source
  advancement and trailing-edge trimming compose correctly through the color-key
  path.
- Focused validation after the `TransparentImage` remaining corner clipping
  slice: `cargo test --features unicorn,trace,win32-desktop
  coredll_raw_transparent_image_clips_off_top_right --
  coredll_raw_transparent_image_clips_off_bottom_left --nocapture` passed,
  followed by `cargo fmt --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full `cargo test -j
  1 --features unicorn,trace,win32-desktop`. Test logs are under
  `target/cargo-test-coredll-raw-gwe-transparent-mixed-corners.20260616-142029.*.log`
  and `target/cargo-test-full-transparent-mixed-corners.20260616-142037.*.log`.
- `tests/coredll_raw_gwe.rs`: raw selected-DIB and framebuffer
  `TransparentImage` now cover CE `draw.cpp::ClipBitBlt(ETransparentImage)`
  fully clipped top-left and bottom-right no-op cases. The regressions prove
  fully eliminated destination rectangles still return success without changing
  selected-DIB pixels or marking framebuffer dirty.
- Focused validation after the `TransparentImage` fully clipped no-op slice:
  `cargo test --features unicorn,trace,win32-desktop
  coredll_raw_transparent_image_fully_clipped -- --nocapture` passed, followed
  by `cargo fmt --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full `cargo test -j
  1 --features unicorn,trace,win32-desktop`. Test logs are under
  `target/cargo-test-coredll-raw-gwe-transparent-full-clip.20260616-142511.*.log`
  and `target/cargo-test-full-transparent-full-clip.20260616-142535.*.log`.
- `tests/coredll_raw_gwe.rs`: raw selected-DIB and framebuffer
  `TransparentImage` now cover CE `draw.cpp::ClipBitBlt(ETransparentImage)`
  one-pixel-visible top-left and bottom-right near-full clips. The regressions
  prove the single surviving destination pixel samples the expected source
  corner and framebuffer dirtiness is restricted to that one pixel.
- Focused validation after the `TransparentImage` almost fully clipped slice:
  `cargo test --features unicorn,trace,win32-desktop
  coredll_raw_transparent_image_almost_fully_clipped -- --nocapture`, `cargo
  fmt --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full `cargo test -j
  1 --features unicorn,trace,win32-desktop`. Test logs are under
  `target/cargo-test-coredll-raw-gwe-transparent-almost-full-clip.20260616-143045.*.log`
  and
  `target/cargo-test-full-transparent-almost-full-clip.20260616-143106.*.log`.
- `tests/coredll_raw_gwe.rs`: raw selected-DIB and framebuffer
  `TransparentImage` now cover CE `draw.cpp::ClipBitBlt(ETransparentImage)`
  individual left/right/top/bottom fully clipped no-op rows plus their
  one-pixel-visible near-full edge rows. The helpers verify successful no-op
  returns without framebuffer dirtiness and one-pixel survivors sampling the
  expected source edge pixel.
- Validation after the `TransparentImage` individual-edge clipping slice:
  `cargo test --features unicorn,trace,win32-desktop individual_edges --
  --nocapture`, `cargo fmt --check`, `git diff --check`, `cargo check
  --features unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full `cargo test
  -j 1 --features unicorn,trace,win32-desktop`. Test logs are under
  `target/cargo-test-coredll-raw-gwe-transparent-individual-edge-clip.20260616-143621.*.log`
  and `target/cargo-test-full-transparent-individual-edge-clip.20260616-143632.*.log`.
- `tests/coredll_raw_gwe.rs`: raw selected-DIB and framebuffer
  `TransparentImage` now cover CE `draw.cpp::SimpleTransparentImageTest`'s
  translated source rectangle: a green transparent source quadrant preserves a
  black destination while a nested red source pixel copies from the offset
  source rectangle.
- Validation after the `TransparentImage` simple source-offset slice: `cargo
  test --features unicorn,trace,win32-desktop simple_source_offset --
  --nocapture`, `cargo fmt --check`, `git diff --check`, `cargo check
  --features unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full `cargo test
  -j 1 --features unicorn,trace,win32-desktop`. Test logs are under
  `target/cargo-test-coredll-raw-gwe-transparent-simple-source-offset.20260616-144131.*.log`
  and `target/cargo-test-full-transparent-simple-source-offset.20260616-144139.*.log`.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw selected-DIB
  `TransparentImage` now covers CE `draw.cpp::BltAlphaDIBTest(ETransparentImage)`
  for top-down 32 bpp DIB sections. Plain color-key copies now preserve the
  source alpha byte in the destination DWORD when no ROP/blend/brush transform
  is active instead of forcing opaque alpha.
- `tests/coredll_raw_gwe.rs`, `PLAN.MD`, and `SOURCE_REFERENCES.md`: raw
  framebuffer `TransparentImage` coverage now includes the CE
  `draw.cpp::TransparentImagePalTest(ETransparentImage)` duplicate palette-RGB
  branch. A 4 bpp selected-DIB source mutates multiple color-table indexes to
  the transparent RGB and verifies the CE-accepted hardware-driver behavior
  where all matching realized RGB entries preserve the destination while a
  nonmatching red entry still copies.
- Focused validation after the `TransparentImage` paletted-DIB slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_transparent_image_paletted_dib_duplicate_rgb_keys
  -- --nocapture` passed. Test logs are under
  `target/cargo-test-transparent-pal-dib.20260616-150044.*.log`.
- Broader validation after the `TransparentImage` paletted-DIB slice:
  `cargo fmt`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and `cargo test -j 1
  --features unicorn,trace,win32-desktop` passed. Test logs are under
  `target/cargo-test-coredll-raw-gwe-transparent-pal-dib.20260616-150425.*.log`
  and `target/cargo-test-full-transparent-pal-dib.20260616-150448.*.log`.
- `src/ce/coredll.rs`, `src/emulator/imports.rs`, `src/emulator/unicorn.rs`,
  and `tests/coredll_dispatch.rs`: raw import-trap context plumbing now
  compiles with the restored raw caller PC, stack pointer, and stack-word audit
  fields; the Unicorn trap path precomputes stack words before passing mutable
  memory into the coredll dispatcher.
- Validation after the `TransparentImage` alpha-DIB and raw import stack-context
  slices: `cargo test --features unicorn,trace,win32-desktop
  coredll_raw_transparent_image_preserves_alpha_dib_pixel_dword -- --nocapture`,
  `cargo test --features unicorn,trace,win32-desktop --test coredll_dispatch
  raw_stub_audit_keeps_import_trap_context -- --nocapture`, `cargo fmt
  --check`, `cargo check --features unicorn,trace,win32-desktop`, and `cargo
  build --release --features unicorn,trace,win32-desktop` passed with existing
  warnings.
- Live iNavi trace after `trace: include raw import stack context`: the owned
  splash `ShowWindow(SW_SHOW)` still enters through the MFC wrapper
  `caller_pc=0x6002ab5c`, but the stack snapshot exposes app-side candidates
  `0x0005e864`, `0x0005e810`, and `0x0005895c`. Disassembly shows the app
  routine around `iNavi.exe+0x4d7a0` creates/configures the full-screen splash
  and calls wrapper `0x48e998` with `cmd=5`; no matching hide/destroy/demote
  for `0x00020008` was observed yet.
- Validation after the transparent all-edge clipping and active visible-receiver
  rotation slice: focused
  `remote_input_rotates_to_active_visible_receiver_thread`, focused
  `coredll_raw_transparent_image_clips_off_right`, focused
  `coredll_raw_transparent_image_clips_off_bottom`, `cargo fmt --check`, `git
  diff --check`, `cargo check --features unicorn,trace,win32-desktop`, `cargo
  test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  and full `cargo test -j 1 --features unicorn,trace,win32-desktop` passed.
  Full-test logs are under
  `target/cargo-test-full-transparent-all-edges.20260616-134146.*.log`.
- `tests/coredll_raw_gwe.rs`: raw selected-DIB and framebuffer `MaskBlt` now
  cover CE `draw.cpp::ClipBitBlt(EMaskBlt)` top, right, and bottom destination
  clipping in addition to the existing off-left cases. The regressions use
  `MAKEROP4(SRCCOPY, WHITENESS)` and 1 bpp masks to prove source and mask
  coordinates stay aligned as the leading edge advances or the trailing edge is
  trimmed.
- Validation after the `MaskBlt` all-edge clipping slice: focused
  `coredll_raw_mask_blt_clips_off_top`, focused
  `coredll_raw_mask_blt_clips_off_right`, focused
  `coredll_raw_mask_blt_clips_off_bottom`, `cargo fmt --check`, `git diff
  --check`, `cargo check --features unicorn,trace,win32-desktop`, `cargo test
  -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed. Full-test
  logs are under
  `target/cargo-test-full-maskblt-all-edges.20260616-134932.*.log`.
- `tests/coredll_raw_gwe.rs`: raw selected-DIB and framebuffer `MaskBlt` now
  cover CE `draw.cpp::NegativeSize(EMaskBlt)` vertical destination mirroring
  with a real 1 bpp mask and `MAKEROP4(SRCCOPY, WHITENESS)`, proving source
  rows and foreground/background mask rows stay aligned through the CE
  bottom/right-plus-one signed-extent convention.
- Focused validation after the `MaskBlt` negative-height slice:
  `cargo test --features unicorn,trace,win32-desktop
  coredll_raw_mask_blt_mirrors_negative_destination_height -- --nocapture`
  passed, followed by `cargo fmt --check`, `git diff --check`, `cargo check
  --features unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full `cargo test -j
  1 --features unicorn,trace,win32-desktop`. Test logs are under
  `target/cargo-test-coredll-raw-gwe-maskblt-negheight.20260616-135717.*.log`
  and `target/cargo-test-full-maskblt-negheight.20260616-135728.*.log`.
- `tests/coredll_raw_gwe.rs`: the same CE `NegativeSize(EMaskBlt)` coverage
  now includes combined negative destination width and height for selected-DIB
  and framebuffer destinations. A checker 1 bpp mask proves the source samples
  and foreground/background ROP4 mask bits mirror together across both axes.
- Focused validation after the `MaskBlt` both-axes mirroring slice:
  `cargo test --features unicorn,trace,win32-desktop
  coredll_raw_mask_blt_mirrors_negative_destination_width_and_height --
  --nocapture` passed, followed by `cargo fmt --check`, `git diff --check`,
  `cargo check --features unicorn,trace,win32-desktop`, `cargo test -j 1
  --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full
  `cargo test -j 1 --features unicorn,trace,win32-desktop`. Test logs are
  under `target/cargo-test-coredll-raw-gwe-maskblt-negwh.20260616-140100.*.log`
  and `target/cargo-test-full-maskblt-negwh.20260616-140111.*.log`.
- `tests/coredll_raw_gwe.rs`: raw `TransparentImage` now covers CE
  `draw.cpp::StretchBltFlipMirrorTest(ETransparentImage)` isolated negative
  source-height mirroring for selected-DIB and framebuffer destinations. The
  regressions prove vertical source rows flip through the transparent color-key
  path without requiring a simultaneous destination flip.
- Focused validation after the `TransparentImage` negative source-height slice:
  `cargo test --features unicorn,trace,win32-desktop
  coredll_raw_transparent_image_mirrors_negative_source_height -- --nocapture`
  passed, followed by `cargo fmt --check`, `git diff --check`, `cargo check
  --features unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full `cargo test -j
  1 --features unicorn,trace,win32-desktop`. Test logs are under
  `target/cargo-test-coredll-raw-gwe-transparent-src-height.20260616-140450.*.log`
  and `target/cargo-test-full-transparent-src-height.20260616-140501.*.log`.
- `tests/coredll_raw_gwe.rs`: raw `TransparentImage` now also covers CE
  `draw.cpp::StretchBltFlipMirrorTest(ETransparentImage)` isolated negative
  source-width mirroring for selected-DIB and framebuffer destinations. The
  regressions prove horizontal source columns flip through the transparent
  color-key path without requiring a simultaneous destination flip.
- Focused validation after the `TransparentImage` negative source-width slice:
  `cargo test --features unicorn,trace,win32-desktop
  coredll_raw_transparent_image_mirrors_negative_source_width -- --nocapture`
  passed, followed by `cargo fmt --check`, `git diff --check`, `cargo check
  --features unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full `cargo test -j
  1 --features unicorn,trace,win32-desktop`. Test logs are under
  `target/cargo-test-coredll-raw-gwe-transparent-src-width.20260616-140838.*.log`
  and `target/cargo-test-full-transparent-src-width.20260616-140845.*.log`.
- Validation after the transparent off-top clipping slice: focused
  `coredll_raw_transparent_image_clips_off_top`, `cargo fmt --check`, `git
  diff --check`, `cargo check --features unicorn,trace,win32-desktop`, `cargo
  test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  and full `cargo test -j 1 --features unicorn,trace,win32-desktop` passed.
  Full-test logs are under
  `target/cargo-test-full-transparent-offtop-clip.20260616-133448.*.log`.
- Validation after the framebuffer transparent off-left clipping slice: focused
  `coredll_raw_transparent_image_clips_off_left_framebuffer_source_alignment`,
  `cargo fmt --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full `cargo test -j
  1 --features unicorn,trace,win32-desktop` passed. Full-test logs are under
  `target/cargo-test-full-transparent-framebuffer-clip.20260616-132914.*.log`.
- Validation after the transparent off-left clipping slice: focused
  `coredll_raw_transparent_image_clips_off_left_selected_dib_source_alignment`,
  `cargo fmt --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full `cargo test -j
  1 --features unicorn,trace,win32-desktop` passed. Full-test logs are under
  `target/cargo-test-full-transparent-clip.20260616-132423.*.log`.
- Validation after the transparent near-miss slice: focused
  `coredll_raw_transparent_image_near_miss_keys_remain_opaque`, `cargo fmt
  --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full `cargo test -j
  1 --features unicorn,trace,win32-desktop` passed. Full-test logs are under
  `target/cargo-test-full-transparent-nearmiss.20260616-131840.*.log`.
- `src/emulator/unicorn.rs`: escaped cross-thread visible-message WNDPROC
  callouts that already reached their saved import PC now restore the captured
  `ResumeImportAfterWndProc` thread/register context instead of being archived
  as compact orphan return records. This prevents the later lossy
  `OrphanedVisibleMessage` `WM_PAINT` recovery path from resuming with only
  `PC`/`RA` and losing the original active-thread state.
- Live iNavi drive after the escaped visible-message resume fix: the rebuilt
  release process stayed alive past the prior heap-execute/`FETCH_PROT`
  failure after repeated touch sweeps. The run continued through
  `iNavi.exe+0x2f8100`, `+0x2ffe8c`, `+0x3342d4`, `+0x33432c`,
  `+0x329da8`, and `+0x33292c` while file reads advanced through
  `resmapi_800x480.bin`. The owned full-screen splash popup remains visible
  and the captured framebuffer hash did not change, so the map reveal is still
  unresolved.
- `src/emulator/unicorn.rs`: Unicorn `SendMessageTimeout` direct import
  re-entry now clears an expired parked send without tripping the blocked-state
  `RefCell` borrow, restores the sender MIPS context to the saved return PC,
  reports `ERROR_TIMEOUT`, leaves `lpdwResult` untouched, consumes the timeout
  completion record, and removes the receiver's stale queued delivery.
- `src/emulator/unicorn.rs`: Unicorn `SendMessageTimeout` direct import
  re-entry now also covers the CE `smfResultReady` success path: an already
  completed receiver result writes `lpdwResult`, consumes the completed send
  state, restores the saved sender context, returns `TRUE`, and leaves last
  error clear without incrementing timeout statistics.
- `src/emulator/unicorn.rs`: the scheduler-driven blocked
  `SendMessageTimeout` resume path now has the same borrow-safe timeout cleanup
  and a regression that proves the interrupted active thread is suspended while
  execution returns to the timed-out sender without stale receiver delivery.
- `src/emulator/unicorn.rs`: scheduler-driven `SendMessageTimeout`
  result-ready replay now has focused coverage for the CE `smfResultReady`
  branch. A completed receiver result writes `lpdwResult`, consumes the send
  completion, suspends the interrupted active thread, resumes the sender with
  `TRUE`, and leaves last error clear.
- `src/ce/coredll.rs` and `tests/coredll_raw_kernel.rs`: modal
  `MessageBoxW` teardown now restores the caller's previous focus/activation
  target when it is still valid and enabled, falling back to the owner only if
  the owner was enabled before the modal box. A raw regression covers the
  default-result fallback path restoring focus to an owner child after the
  transient dialog is destroyed.
- `tests/coredll_raw_kernel.rs`: registered-taskbar `Shell_NotifyIcon`
  coverage now proves duplicate `NIM_ADD` and missing-record `NIM_MODIFY`
  failures do not post copied `NOTIFYICONDATAW` payloads, while successful live
  `NIM_MODIFY` and `NIM_DELETE` calls still post CE-style taskbar copies and
  release private payloads.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `EqualRgn` now
  follows CE `region.cpp::passNull2Region(EEqualRgn)` by splitting null-region
  operands (`ERROR_INVALID_PARAMETER`) from bad or wrong-type GDI handles
  (`ERROR_INVALID_HANDLE`) while preserving clear-last-error equal/unequal
  comparisons for valid regions.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `PtInRegion` and
  `RectInRegion` now follow CE
  `region.cpp::passNull2Region(EPtInRegion/ERectInRegion)`, including
  invalid-handle reporting for null, bad, and wrong-type region handles,
  `RectInRegion(validRegion, NULL)` invalid-parameter reporting, and
  clear-last-error valid inside/outside tests.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `GetRgnBox` and
  `CombineRgn` now follow the remaining CE
  `region.cpp::passNull2Region(EGetRgnBox/ECombineRgn)` parameter edges for
  null output rectangles on valid regions and invalid combine modes.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `GetRegionData`
  now covers CE `region.cpp::GetRegionDataCheckReturn` and
  `passNull2Region(EGetRegionData)` size-query, `RGNDATAHEADER`/RECT payload,
  invalid-handle, and caller-buffer write-fault behavior; `CreateRectRgnIndirect(NULL)`
  is also covered for CE invalid-parameter reporting.
- `tests/coredll_raw_gwe.rs`: raw region coverage now includes CE
  `region.cpp::CreateNullRegionTest` and `SetRectRgnNULLRgnTest`, proving
  zero-width/zero-height create/set inputs keep a valid region handle while
  `GetRgnBox` reports `NULLREGION` with canonical zero bounds.
- Focused validation after the Unicorn parked `SendMessageTimeout` timeout
  re-entry slice: `cargo fmt` and `$env:CARGO_INCREMENTAL='0'; cargo test
  -j 1 --features unicorn,trace,win32-desktop send_message_timeout_ --lib`
  passed after the timeout and result-ready scheduler slices; full validation
  then passed with `cargo fmt --check`, `git diff --check`,
  `$env:CARGO_INCREMENTAL='0'; cargo check --features
  unicorn,trace,win32-desktop`, and `$env:CARGO_INCREMENTAL='0'; cargo test -j
  1 --features unicorn,trace,win32-desktop`. Cargo still emits the existing
  unused-code warnings, plus an unused `CpuBackend` import warning in filtered
  tests.
- `tests/coredll_raw_gwe.rs`: raw `SendMessageTimeout` coverage now verifies
  `SMTO_ABORTIFHUNG` by itself queues normally while the receiver is just below
  the CE 5-second hung threshold, leaves the caller result pointer untouched,
  clears last error, and preserves the exact accepted timeout flag on the
  queued cross-thread transaction.
- Validation after the `SMTO_ABORTIFHUNG`-only below-threshold slice:
  `cargo fmt --check`, `git diff --check`,
  `$env:CARGO_INCREMENTAL='0'; cargo check --features
  unicorn,trace,win32-desktop`, `$env:CARGO_INCREMENTAL='0'; cargo test -j 1
  --features unicorn,trace,win32-desktop
  coredll_raw_send_message_timeout_abortifhung_only_queues_below_hung_threshold`,
  and `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. Cargo still emits the existing
  unused-code warnings, plus an unused `CpuBackend` import warning in the
  heavily filtered focused test.
- `src/emulator/unicorn.rs`: Unicorn blocked `MsgWaitForMultipleObjectsEx`
  resume coverage now verifies CE's message-wake slot after all supplied wait
  handles. A blocked message wait with two unsignaled event handles resumes the
  guest with `WAIT_OBJECT_0 + 2`, restores the parked MIPS context, removes the
  scheduler waiter, and clears the changed queue-input bit for the default
  non-`MWMO_INPUTAVAILABLE` path.
- Validation after the Unicorn `MsgWaitForMultipleObjectsEx` resume-index
  slice: `cargo fmt --check`, `git diff --check`,
  `$env:CARGO_INCREMENTAL='0'; cargo check --features
  unicorn,trace,win32-desktop`, `$env:CARGO_INCREMENTAL='0'; cargo test -j 1
  --features unicorn,trace,win32-desktop
  blocked_msg_wait_resumes_with_message_index_after_all_handles`, and
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. Cargo still emits the existing
  unused-code warnings, plus an unused `CpuBackend` import warning in the
  heavily filtered focused test.
- `src/config.rs`, `src/ce/file.rs`, and `tests/basic_subsystems.rs`:
  configured mounted-storage `IClass` entries now support CE devcore-style
  `%b` bus-name advertisements when the mount supplies `bus_name`. `%b` entries
  remain skipped without bus metadata, matching the CE `devpnp.c` guard, while
  real device-manager `fsdev_t` interface handles remain queued.
- `src/emulator/unicorn.rs`: orphaned visible `WM_PAINT` return stubs now share
  the caller-frame-free recovery path used by `CallWindowProcW`, with a focused
  regression proving the stub restores `PC`/`RA` to the archived return address.
- `src/emulator/unicorn.rs`: orphaned visible `WM_PAINT` WNDPROC returns now
  also validate the pending update region after the guest returns, so the
  recovered `OrphanedVisibleMessage` paint path clears the same dirty state as
  the normal paint dispatch path.
- `src/ce/coredll.rs` and `tests/coredll_raw_memory_file.rs`: raw
  `CreateFileW` now maps missing files to `ERROR_FILE_NOT_FOUND`, empty or
  root-escaping paths to `ERROR_PATH_NOT_FOUND`, access denial to
  `ERROR_ACCESS_DENIED`, and other invalid arguments to
  `ERROR_INVALID_PARAMETER` instead of collapsing all non-access failures to an
  invalid handle status.
- Validation after the configured `%b` IClass and orphaned visible-paint slice:
  `cargo fmt --check`, `git diff --check`,
  `$env:CARGO_INCREMENTAL='0'; cargo check --features
  unicorn,trace,win32-desktop`, `$env:CARGO_INCREMENTAL='0'; cargo test -j 1
  --features unicorn,trace,win32-desktop mount_iclass`,
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop
  orphaned_visible_paint_return_stub_recovers_without_caller_frame`, and
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. Cargo still emits the existing
  unused-code warnings, plus an unused `CpuBackend` import warning in the
  heavily filtered focused test.
- `src/ce/coredll.rs` and `src/emulator/imports.rs`: direct
  `FSDMGR_FormatVolume @15` and `FSDMGR_ScanVolume @31` now use the configured
  logical-disk registry profile to probe the CE `Util` value before reporting
  status. Missing utility metadata still returns `ERROR_FILE_NOT_FOUND`; a
  registered disk with a configured utility DLL name now advances to the current
  `ERROR_MOD_NOT_FOUND` load/execution boundary. Real guest utility DLL export
  execution remains queued.
- `src/emulator/cpu_mips.rs` and `src/emulator/unicorn.rs`: startup
  profiling with `sudo --inline cargo flamegraph` showed the iNavi run spending
  scheduler-hook time in a linear MIPS trampoline-origin scan. The scheduler
  and wall-clock stop paths now use a sorted trampoline jump index, and live
  trampoline state reuses the already-built origin-to-stub map. A follow-up
  Windows-sudo flamegraph on 2026-06-16 found the remaining raw-import and
  WNDPROC annotation paths still using the old linear helper; commit `4559d704`
  switches those paths to the existing trampoline jump index and removes the
  dead linear helper. The post-fix profile
  `profiles/startup-windows-sudo-bounded-indexed-20260616-142504.svg` no longer
  contains `mips_trampoline_origin_for_pc` and reaches the later
  `iNavi.exe+0x329da4` resource/map-loading slice with about 3.1 MB read. The
  remaining visible startup overhead is Unicorn TCG execution/translation and
  code-hook callbacks, not host SD-card file I/O or the old linear trampoline
  scan.
- `src/ce/file.rs`, `src/ce/kernel.rs`, and `tests/basic_subsystems.rs`:
  mount-published device-interface advertisements now track owner roots, so two
  mounted volumes advertising the same class/name keep one visible advertisement
  until the final owning mount is removed. The regression proves unmounting one
  duplicate owner does not emit a detach notification or hide the shared
  `\StoreMgr\<device>` interface while another mount still owns it.
- `src/ce/kernel.rs` and `src/emulator/unicorn.rs`: Unicorn now parks
  blocking `ReadMsgQueue` and `WriteMsgQueue` calls on the queue endpoint
  handle when CE would wait, then replays the original coredll import when
  the queue becomes read/write-ready or the timeout expires. Read waits resume
  with the posted message data and flags, write waits resume after a reader
  frees normal queue capacity, and current-thread finite read timeouts replay
  to `FALSE` with `ERROR_TIMEOUT` instead of returning a wait code.
- `src/ce/coredll.rs`, `src/ce/coredll_ordinals.rs`,
  `src/ce/kernel.rs`, and `tests/coredll_raw_kernel.rs`: coredll now exports
  CE `EnumDeviceInterfaces @1874` and enumerates the kernel's tracked
  advertised interface table. The raw path writes the advertised GUID, reports
  required UTF-16 byte counts for names, returns `ERROR_INSUFFICIENT_BUFFER`
  for undersized buffers, supports class-only enumeration, reports
  `ERROR_NO_MORE_ITEMS` at the end, and rejects invalid class pointers,
  name-buffer/size-pointer combinations, and unknown nonzero handles. Coredll
  `RequestDeviceNotifications @1504` now records typed notification
  subscriptions with the requested class GUID, message-queue handle, and
  all-devices flag, while `StopDeviceNotifications @1505` closes only those
  handles and rejects stale/wrong handles with `ERROR_INVALID_HANDLE`. The CE
  message-queue ordinals now create/open/read/write/report/close shared queue
  state, reject wrong read/write endpoints like CE, preserve explicit
  `dwMaxMessages` limits, report full queues as `ERROR_TIMEOUT`, reject
  oversized writes with `ERROR_INSUFFICIENT_BUFFER`, consume truncated reads
  with CE's success-plus-error surface, report non-`MSGQUEUE_ALLOW_BROKEN`
  counterpart closure as `ERROR_PIPE_NOT_CONNECTED`, preserve allow-broken
  drain/write behavior, make read endpoints wait-ready for data or broken
  writers, make write endpoints wait-ready for normal queue room or broken
  readers, wake counterpart waiters as queue state changes, and device
  advertisement changes enqueue CE-shaped `DEVDETAIL` attach/detach records
  that the storage-manager-style `ReadMsgQueue` path can consume, and
  `MSGQUEUE_MSGALERT` messages use CE's priority alert slot without increasing
  normal queue depth. Full CE `fsdev_t` per-device interface scoping remains
  queued.
- `src/config.rs`, `src/ce/file.rs`, `src/ce/kernel.rs`, `mounts.toml`, and
  `tests/basic_subsystems.rs`: mounted storage config now carries optional
  CE-style block `device_name` plus `interface_classes` GUID strings. Kernel
  boot parses those `IClass` strings into Windows GUID memory layout, publishes
  `\StoreMgr\<device>` advertisements through the existing device-interface
  table, ignores malformed GUID strings, and removes the configured
  advertisements when a removable mounted root is unmounted. The default SDMMC
  mount now advertises the block-driver and power-manageable block-device
  classes visible in the local registry/CE platform sources.
- `src/ce/timer.rs`, `src/main.rs`, and `src/remote_server.rs`: live remote
  diagnostics now publish `/api/v1/debug/timers.txt` with generic pending
  `SetTimer` state (`HWND`, id, message, callback, due/period, and pending flag)
  so the iNavi splash-to-map transition can be checked for stuck or undrained
  timer messages without adding app-specific behavior.
- Live iNavi drive after the reset-recovery FSDMGR work: the current
  `192.168.0.39:8765` process remains on the real splash framebuffer, but file
  tracing has advanced into map-layer reads (`mapdata\bgdata`,
  `mapdata\landuse`, `mapdata\point`, `displayreshigh.bin`, and `soap.bin`).
  The visible tree now contains the main `0x20004` map child plus hidden map
  subwindows, while the top owned `0x20008` splash popup remains visible and no
  later hide/demote trace for that popup has been observed.
- `src/ce/coredll.rs` and `src/emulator/imports.rs`: direct `fsdmgr.dll`
  `FSDMGR_AdvertiseInterface @2` imports now patch through the normal import
  trap table and share the coredll `AdvertiseInterface` implementation,
  validating the GUID/name inputs, mapping empty FSDMGR names to the CE root
  backslash name, and recording add/remove device-interface advertisements in
  kernel state.
- `src/ce/coredll.rs` and `src/emulator/imports.rs`: direct `fsdmgr.dll`
  `FSDMGR_GetRegistryFlag @18`, `FSDMGR_GetRegistryString @19`, and
  `FSDMGR_GetRegistryValue @20` imports now patch through the normal import
  trap table, fail closed with CE-style missing-registry output clearing
  instead of leaving stale caller buffers behind, and read configured
  logical-disk root/subkey profile values for DWORD, string, and flag queries.
- `src/ce/coredll.rs`, `tests/coredll_raw_memory_file.rs`, and
  `SOURCE_REFERENCES.md`: raw `DPA_EnumCallback`/`DSA_EnumCallback` now treat a
  null callback as a successful no-op instead of a raw unsupported callback
  failure, while raw `DPA_DestroyCallback`/`DSA_DestroyCallback` with a null
  callback now frees the container through the existing destroy helpers.
  Non-null callbacks remain routed to the Unicorn callout frontier.
- `src/ce/coredll.rs`, `tests/coredll_raw_kernel.rs`, `PLAN.MD`, and
  `SOURCE_REFERENCES.md`: raw `Shell_NotifyIcon(NIM_DELETE)` now removes an
  existing stored tray record even after the owner HWND has gone stale, matching
  the CE sample shell's copied `NOTIFYICONDATA` taskbar-routing shape and
  allowing destroy-icon cleanup during teardown, while stale-owner add/modify
  requests still fail with `ERROR_INVALID_WINDOW_HANDLE`.
- `src/ce/shell.rs`, `tests/coredll_raw_kernel.rs`, `PLAN.MD`, and
  `SOURCE_REFERENCES.md`: `SHNotification*` expiration now treats
  `csDuration` as seconds like the CE taskbar `m_csDuration * 1000` timer path
  instead of centiseconds; the raw timeout regression now proves a one-second
  iconic notification survives the old sub-second fast-expiry edge and expires on
  the CE second boundary.
- `src/ce/coredll.rs`, `tests/coredll_raw_kernel.rs`, `PLAN.MD`, and
  `SOURCE_REFERENCES.md`: raw `LoadLibraryExW` now accepts CE's documented
  `LOAD_WITH_ALTERED_SEARCH_PATH` low-word flag for already registered modules,
  retaining the loaded module and preserving the unsupported-flag failure for
  non-CE flag bits.
- `src/ce/coredll.rs`, `tests/coredll_raw_memory_file.rs`, and
  `SOURCE_REFERENCES.md`: raw `DPA_Grow` and `DSA_Grow` now preallocate their
  guest backing arrays through the same heap-backed capacity helpers used by
  insert/clone paths, including grow-increment rounding, no-shrink behavior,
  negative-count rejection, and checked allocation-size overflow handling.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, and
  `SOURCE_REFERENCES.md`: raw `ImageList_ReplaceIcon` now follows the CE
  `imagelist.cpp` bitmap-backed path for real icons by snapshotting rendered
  color plus optional `DI_MASK` output into owned image-list bitmap/mask
  storage. Synthetic shell pseudo-icons still store their handle metadata. The
  GWE regression destroys the source icon before drawing the image-list entry,
  proving the list owns a durable rendered snapshot.
- `src/emulator/unicorn.rs` and `src/main.rs`: post-run wall-clock stop
  snapshots now also canonicalize a parked MIPS `jr` import thunk PC to the
  actual import-trap page target before restart/debug bookkeeping, and the CLI
  rotation gate now treats a captured wall-clock stop as rotatable even when the
  snapshot carries a trap address. This covers the case where the run stops
  after the thunk branch but before the trap PC is reflected in the saved
  snapshot.
- Live verification for the import-thunk wall-stop slice: iNavi no longer
  remains parked at `iNavi.exe+0x3554` before `COREDLL.dll@1047` (`memset`).
  With progress tracing enabled, a bounded host run advanced through the
  Happyway/modal path, cleared/recovered stale WNDPROC state, and reached
  `iNavi.exe+0x329da8` with later file/resource activity.
- Validation after the ImageList_ReplaceIcon snapshot and wall-clock rotation
  slice: `cargo fmt --check`, `git diff --check`, `CARGO_INCREMENTAL=0 cargo
  check --features unicorn,trace,win32-desktop`, focused `coredll_raw_gwe`,
  `coredll_raw_kernel`, `wall_clock_import_jr_thunk_resumes_at_import_trap`,
  and `idle_poll_detects_saved_get_message_waiter` tests, plus full
  `CARGO_INCREMENTAL=0 cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The eVC4 MIPSII fixture remains ignored
  because that toolchain is not configured, and Cargo still emits the existing
  unused-code warnings.
- `tests/coredll_raw_kernel.rs`, `PLAN.MD`, and `SOURCE_REFERENCES.md`: raw
  `ExtractIconExW` PE coverage now includes a dedicated 32bpp `BI_RGB`
  `RT_ICON` fixture, verifying extracted bitmap metadata, trailing AND-mask
  backing, and `DrawIconEx(DI_NORMAL)` BGRA-to-RGB565 rendering.
- `src/emulator/unicorn.rs`: the saved-register diagnostic string now includes
  `t0` alongside `pc`, `ra`, stack/frame, return-value, and saved-register
  fields, making parked process dumps a little more useful when tracing MIPS
  handoff state.
- Validation after the 32bpp PE icon and Unicorn diagnostic slice: `cargo
  fmt`, focused `CARGO_INCREMENTAL=0 cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_kernel
  sh_get_file_info_uses_registry_associations_and_attributes -- --nocapture`,
  `cargo fmt --check`, `git diff --check`, and full `CARGO_INCREMENTAL=0 cargo
  test -j 1 --features unicorn,trace,win32-desktop` passed. The eVC4 MIPSII
  fixture remains ignored because that toolchain is not configured, and Cargo
  still emits the existing unused-code warnings.
- `src/emulator/unicorn.rs`: host wall-clock timeout capture now recognizes
  MIPS `jr` import-thunk stubs with a `nop` delay slot and records the actual
  import-trap target PC, so resume/restart bookkeeping lands at the imported
  coredll trap instead of the thunk instruction.
- Validation after the wall-clock import-thunk resume slice so far: `cargo
  test -j 1 --features unicorn,trace,win32-desktop
  wall_clock_import_jr_thunk_resumes_at_import_trap`, `cargo fmt --check`,
  `git diff --check`, and a full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed.
- `src/ce/coredll.rs` and `tests/coredll_raw_kernel.rs`: bitmap-backed
  image-list rendering now applies CE's non-DDB first-palette latch while
  decoding indexed image pixels, so later 8bpp entries draw through the list's
  first color table instead of their own source bitmap palette; `ILC_COLORDDB`
  lists still draw each indexed bitmap through its own palette.
- Validation after the image-list indexed draw palette slice so far: `cargo
  fmt`, `cargo check --features unicorn,trace,win32-desktop`, and `cargo test
  -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel
  image_list_ordinals_track_created_lists_and_icons` passed. `cargo fmt
  --check`, `git diff --check`, and a full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` also passed. Cargo still emits the existing
  unused-code warnings plus Windows profile/incremental-cache cleanup noise,
  and the eVC4 MIPSII fixture remains ignored because that toolchain is not
  configured.
- `src/ce/resource.rs`, `tests/basic_subsystems.rs`, and
  `tests/coredll_raw_kernel.rs`: image-list objects now record CE's first-add
  indexed palette latch for non-`ILC_COLORDDB` lists, preserving the first
  source bitmap color table while leaving DDB lists and later additions
  untouched; size changes clear the latched palette with the images/overlays.
- Validation after the image-list palette latch slice: `cargo fmt`, `cargo
  test -j 1 --features unicorn,trace,win32-desktop --test basic_subsystems
  resource_system_image_list_create_add_count_info_bk_color_and_destroy`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed.
  `cargo fmt --check`, `git diff --check`, and a full `cargo test -j 1
  --features unicorn,trace,win32-desktop` also passed. Cargo still emits the
  existing unused-code warnings plus Windows profile/incremental-cache cleanup
  noise, and the eVC4 MIPSII fixture remains ignored because that toolchain is
  not configured.
- `tests/coredll_raw_gwe.rs`: `DrawIconEx` now covers shell/system
  `ImageList_GetIcon` pseudo handles with valid overlay flags and verifies
  invalid overlay slots above CE's four registered overlay entries stay ignored
  when those pseudo handles are rendered into framebuffers.
- `src/emulator/unicorn.rs`: direct-send orphaned wndproc-callout cleanup now
  threads mapped-blob module context into the stack-grace check, so a nearby
  saved frame only preserves a pending direct-send callout when it is unmapped,
  image-backed, or belongs to the same mapped module as the pending wndproc when
  that module is known.
- Validation after the pseudo-icon `DrawIconEx` overlay slice: `cargo fmt
  --check`, `git diff --check`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_destroy_icon_accepts_loaded_icon_handles`, `cargo test -j 1
  --features unicorn,trace,win32-desktop
  direct_send_wndproc_cleanup_keeps_live_callout_until_return_pc`, `cargo test
  -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, and a
  rerun of `cargo test -j 1 --features unicorn,trace,win32-desktop` passed.
  The eVC4 MIPSII fixture remains ignored without toolchain configuration, and
  Cargo still emits the existing unused-code warnings.
- `tests/coredll_raw_gwe.rs`: raw framebuffer-HDC `TransparentImage` now has
  CE `draw.cpp::StretchBltFlipMirrorTest(ETransparentImage)` coverage for
  negative source and destination extents through the transparent color key,
  proving the screen-HDC path uses the same bottom/right +1 mirror convention
  as selected-DIB copies.
- `src/emulator/unicorn.rs`: orphaned `CallWindowProcW` `WM_PAINT` return-stub
  recovery now accepts the live iNavi shape where the archived return has no
  caller frame but the guest `PC`/`RA` are both still parked at the WNDPROC
  return stub, while preserving mismatched-frame rejection for other cases.
- Validation after the framebuffer signed-extent `TransparentImage` and
  `CallWindowProcW` return-stub recovery slice: `cargo fmt --check`, `git diff
  --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe
  coredll_raw_transparent_image_mirrors_negative_extents_on_framebuffer_hdc`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop
  orphaned_call_window_proc_paint_return_stub_recovers_without_caller_frame`,
  and full `cargo test -j 1 --features unicorn,trace,win32-desktop` passed.
  Cargo still emits the existing unused-code warnings plus Windows
  profile/incremental-cache cleanup noise, and the eVC4 MIPSII fixture remains
  ignored because that toolchain is not configured.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `TransparentImage` now accepts CE `draw.cpp::NegativeSize` and
  `StretchBltFlipMirrorTest` nonzero signed destination/source extents for
  selected-DIB and framebuffer HDC copies, normalizing the destination clip
  rectangle and using the shared signed source-coordinate mapping while still
  honoring the transparent color key.
- Validation after the signed-extent `TransparentImage` slice: `cargo fmt
  --check`, `git diff --check`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_transparent_image_mirrors_negative_extents_between_selected_dibs`,
  and full `cargo test -j 1 --features unicorn,trace,win32-desktop` passed.
  Cargo still emits the existing unused-code warnings plus Windows
  profile/incremental-cache cleanup noise, and the eVC4 MIPSII fixture remains
  ignored because that toolchain is not configured.
- `src/ce/kernel.rs`, `src/ce/coredll.rs`, and `tests/coredll_raw_gwe.rs`:
  raw `PatBlt` display-performance timing now mirrors CE
  `dispperf.h::DispPerfParam` for display-target `PARAM_DESTINVIDMEM` plus
  `PARAM_COLORBLACK` and `PARAM_COLORWHITE`, so black and white solid-brush
  `PATCOPY` rows report the expected blit parameter counters through
  `ExtEscape(DISPPERF_EXTESC_GETTIMING)`. Same-framebuffer raw
  `TransparentImage` rows now also report source-video-memory,
  destination-video-memory, and transparent blit counters. Framebuffer
  destination `AlphaBlend` rows now report the CE destination-video-memory
  blit counter while preserving stretch accounting.
- `src/winsock.rs` and `src/emulator/unicorn.rs`: blocking guest TCP
  `connect` waits now use `SO_SNDTIMEO` when present, falling back to the local
  30-second default, so scheduler wait timeouts and the host
  `TcpStream::connect_timeout` path agree.
- `tests/coredll_raw_gwe.rs`: direct caller-DIB coverage now exercises CE
  GDIPRINT-style `BI_ALPHABITFIELDS` rows for `StretchDIBits` and
  `SetDIBitsToDevice`, including 16 bpp BGR4444 and 32 bpp BGR8888 four-mask
  DIBs rendered into selected RGB565 memory DIBs.
- `src/ce/coredll.rs`, `src/ce/resource.rs`, and `tests/coredll_raw_gwe.rs`:
  raw `CreateDIBSection` now stores the fourth `BI_ALPHABITFIELDS` mask on
  bitmap objects, preserves it across icon/image-list bitmap clones, reports
  `BI_ALPHABITFIELDS` in `GetObjectW(DIBSECTION)`, and uses the stored mask for
  32 bpp selected-DIB/framebuffer `AlphaBlend(AC_SRC_ALPHA)` source alpha
  instead of assuming byte-three alpha.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `CreateDIBSection` now accepts the CE alpha stress module's 16 bpp
  `BI_ALPHABITFIELDS` 4-4-4-4 layout, skips the fourth alpha mask before pixel
  storage, preserves the RGB masks, and renders selected 16 bpp
  alpha-bitfield sources through ordinary `BitBlt`.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `CreateDIBSection` now accepts the CE alpha stress module's 32 bpp
  `BI_ALPHABITFIELDS` layout with four masks, skips the alpha mask before pixel
  storage, and preserves high-byte per-pixel alpha for selected-DIB
  `AlphaBlend(AC_SRC_ALPHA)`.
- `src/ce/coredll.rs` and `tests/coredll_raw_kernel.rs`: synthetic shell
  system image-list handles returned from `SHGetFileInfoW` now accept
  `ImageList_Destroy` as a successful CE shell teardown path for both large and
  small lists, while preserving the fixed shared fallback handles for later
  icon-size/query use.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `GetPixel` now
  reads selected memory-DIB and framebuffer pixels instead of returning
  unconditional black for valid HDCs. Raw `DrawEdge` coverage now verifies CE
  `BF_MIDDLE | BF_RECT` center fill and `BF_FLAT | BF_RECT` center preservation
  through real selected-DIB pixel reads. Focused validation passed for
  `coredll_raw_draw_edge_matches_ce_middle_and_flat_center_pixels`.
- `src/emulator/unicorn.rs`: orphaned WNDPROC return stubs now scan recent
  archived WNDPROC returns for a real guest return PC whose saved frame matches
  the current frame exactly or by one WNDPROC call-frame. This covers the live
  iNavi return-stub stall where the archived direct-send frame was 0x20 bytes
  above the current FP, while still rejecting larger mismatches. The recovery
  preserves the current guest `v0` result instead of replaying stale metadata.
  Focused validation passed for matching-frame recovery, nearby-frame recovery,
  missing-return-PC rejection, mismatched-frame rejection, and archived
  direct-send cleanup recovery.
- `src/ce/coredll.rs`, `src/ce/resource.rs`, `tests/basic_subsystems.rs`, and
  `tests/coredll_raw_gwe.rs`: CE clip-region state now copies selected region
  geometry into the DC instead of retaining a live `HRGN` handle. Raw
  `GetClipRgn`, drawing clip helpers, intersect/exclude clip updates, and
  resource-system tests now use copied region objects so callers can mutate or
  delete the source region after `SelectClipRgn`.
- `src/ce/gwe.rs`, `src/ce/kernel.rs`, `src/emulator/unicorn.rs`, and
  `src/main.rs`: restored cross-thread visible-window message handoff after the
  reset. The scheduler now peeks visible-only queues before taking a message,
  refuses cross-process guest WNDPROC callouts until the WNDPROC address belongs
  to a mapped runtime image, preserves live direct-send WNDPROC callouts while
  the receiver context is still active, and rotates idle runnable parked
  processes only when the active process has no visible UI work. Focused
  validation passed for cross-thread mapped-WNDPROC handoff, live direct-send
  cleanup preservation, idle visible-work rotation, and the release Unicorn
  build.
- `src/ce/devices.rs` and `serial_devices.json`: remote GPS serial routing is
  now config-selectable with an optional `remote_gps` device flag. The live
  `drive29` iNavi run starts with `gps_target=COM7:`, and when the guest opens
  `COM7:` the queued NMEA bytes drain from 2784 to 0 and the handle is marked
  `remote-gps-target`; `MFS1:` and `SMB1:` also open and receive dump-derived
  IOCTL traffic. This replaces the previous static preference for the first
  Win32-backed serial device, which incorrectly targeted `COM1:` for this live
  path even though the guest was using `COM7:`.
- Validation for the remote GPS target update: `cargo fmt`,
  `cargo check --features unicorn,trace,win32-desktop`,
  `cargo build --release --features unicorn,trace,win32-desktop`, and
  `cargo test remote_gps_target --features unicorn,trace,win32-desktop` passed
  with existing warnings.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: synthetic IME HKLs now
  report TESTIME resource-backed identity strings through raw
  `ImmGetIMEFileNameW` (`TESTIME.IME`) and `ImmGetDescriptionW`
  (`TESTIME 4.0`), while non-IME HKLs still return empty strings.
- `CARGO_INCREMENTAL=0 cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  and `CARGO_INCREMENTAL=0 cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed after the IMM identity-string update.
- `cargo fmt --check` and
  `CARGO_INCREMENTAL=0 cargo test -j 1 --features unicorn,trace,win32-desktop`
  passed after the IMM identity-string update; the eVC4 MIPSII fixture remains
  ignored because the toolchain is not configured.
- `src/ce/gwe.rs`, `src/ce/coredll.rs`, and `tests/coredll_raw_gwe.rs`: CE
  IMM status-window position now lives in HIMC state and round-trips through
  raw `ImmGet/SetStatusWindowPos`, including active `HIMC == NULL` resolution.
  `ImmNotifyIME(NI_CONTEXTUPDATED, ..., IMC_SETSTATUSWINDOWPOS)`,
  `IMC_OPENSTATUSWINDOW`, and `IMC_CLOSESTATUSWINDOW` now post the matching
  `WM_IME_NOTIFY` status-window messages from `imm.h`/TESTIME.
- `CARGO_INCREMENTAL=0 cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  and `CARGO_INCREMENTAL=0 cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed after the IMM status-window update.
- `cargo fmt --check` and
  `CARGO_INCREMENTAL=0 cargo test -j 1 --features unicorn,trace,win32-desktop`
  passed after the IMM status-window update; the eVC4 MIPSII fixture remains
  ignored because the toolchain is not configured.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `ImmGetProperty`
  now returns CE `imm.h` and TESTIME `ImeInquire`-backed capability values for
  synthetic IME HKLs, including `IMEVER_0400`, Unicode/caret/KBD-first
  properties, conversion caps, `UI_CAP_2700`, `SELECT_CAP_CONVERSION`, zero
  set-composition-string/sentence caps, and `sizeof(DWORD)` private-data size;
  non-IME HKLs still return zero.
- `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  and `CARGO_INCREMENTAL=0 cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed after the `ImmGetProperty` update.
- `cargo fmt --check` and
  `CARGO_INCREMENTAL=0 cargo test -j 1 --features unicorn,trace,win32-desktop`
  passed after the `ImmGetProperty` update; the eVC4 MIPSII fixture remains
  ignored because the toolchain is not configured.
- `src/ce/gwe.rs`, `src/ce/coredll.rs`, and `tests/coredll_raw_gwe.rs`: CE
  IMM placement forms now round-trip through stored HIMC state. Raw
  `ImmGet/SetCompositionWindow` marshal the CE `COMPOSITIONFORM` layout, raw
  `ImmGet/SetCandidateWindow` marshal the CE `CANDIDATEFORM` layout with
  candidate-index validation, and the active `HIMC == NULL` path resolves
  through the current foreground/focused keyboard target where CE samples use
  the active context.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `ImmNotifyIME` now
  maps CE candidate notification actions to posted `WM_IME_NOTIFY` messages.
  `NI_OPENCANDIDATE`, `NI_SELECTCANDIDATESTR`, and `NI_CLOSECANDIDATE` are
  covered with CE candidate-list bit-mask `lParam` values, `HIMC == NULL`
  resolves through the active keyboard target, and out-of-range candidate-list
  indexes fail with `ERROR_INVALID_PARAMETER`.
- `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed after the IMM notification update.
- `cargo fmt --check`, `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `CARGO_INCREMENTAL=0 cargo test -j 1 --features unicorn,trace,win32-desktop`,
  and `git diff --check` passed after the IMM composition/candidate window
  update; the eVC4 MIPSII fixture remains ignored because the toolchain is not
  configured, and `git diff --check` output was limited to existing LF-to-CRLF
  normalization warnings.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: CE input-method sample
  parity now resolves `ImmGetOpenStatus(NULL)`, `ImmSetOpenStatus(NULL, ...)`,
  `ImmGetConversionStatus(NULL, ...)`, and `ImmSetConversionStatus(NULL, ...)`
  through the current foreground/focused keyboard target's HIMC, while nonzero
  invalid HIMC values still fail as invalid handles.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: the same active-HIMC
  resolver now covers CE sample composition probes, including
  `ImmGetCompositionStringW(NULL, GCS_COMPSTR, NULL, 0)`, buffer copies through
  `HIMC == NULL`, and `ImmSetCompositionStringW(NULL, SCS_SETSTR, ...)`.
- `cargo test coredll_raw_translate_message_hangul_ime_composes_syllables -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  and `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed for the NULL-HIMC composition-string update.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  NULL-HIMC composition-string update; the eVC4 MIPSII fixture remains ignored
  because the toolchain is not configured.
- `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful -j 1 --features unicorn,trace,win32-desktop`
  passed for the NULL-HIMC open/conversion status update.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  NULL-HIMC status update; the eVC4 MIPSII fixture remains ignored because the
  toolchain is not configured.
- `tests/coredll_raw_gwe.rs`: added `MsgWaitForMultipleObjectsEx` coverage for
  two valid unsignaled handles plus queued thread input, proving CE-style
  message wake results return at `WAIT_OBJECT_0 + nCount` and leave the
  message retrievable.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_msgwait_returns_message_index_after_all_unsignaled_handles`
  passed for the mixed MsgWait handle/input coverage update.
- `tests/coredll_raw_gwe.rs`: added `SendMessageTimeout` coverage for the
  cross-thread `SMTO_BLOCK | SMTO_ABORTIFHUNG` case after the receiver crosses
  the CE hung threshold, proving it aborts without queueing or writing
  `lpdwResult`.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_send_message_timeout_block_abortifhung_aborts_when_thread_is_hung`
  passed for the combined abort-if-hung coverage update.
- `cargo fmt --check` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed after adding the combined abort-if-hung coverage update.
- `cargo check --features unicorn,trace,win32-desktop` and `cargo test -j 1 --features unicorn,trace,win32-desktop`
  passed after reconciling the current full-feature validation state; the eVC4
  MIPSII fixture remains ignored because the toolchain is not configured.
- `git diff --check` passed with only existing LF-to-CRLF working-copy
  warnings.
- `tests/coredll_raw_gwe.rs`: added raw `AlphaBlend` selected-memory-DIB
  clipping coverage for a negative destination origin, proving the visible
  destination pixel maps to the clipped source pixel instead of restarting at
  source zero.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_alpha_blend_clips_negative_destination_between_selected_dibs`
  passed for the alpha selected-DIB clipping coverage update.
- `tests/coredll_raw_gwe.rs`: added raw `AlphaBlend` framebuffer clipping
  coverage for a negative destination origin, proving the visible pixel maps to
  the correct clipped source pixel and only the clipped framebuffer rectangle is
  dirtied.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_alpha_blend_clips_negative_destination_to_framebuffer`
  passed for the alpha framebuffer clipping coverage update.
- `src/ce/coredll.rs`, `src/ce/gwe.rs`, and `src/ce/shell.rs`: raw
  `MessageBoxW` now creates `MB_TOPMOST` dialogs with the CE `WS_EX_TOPMOST`
  extended style and records whether the transient dialog was active while the
  modal box was live.
- `tests/coredll_raw_kernel.rs`: expanded `MessageBoxW` coverage to assert the
  ordinary dialog has no topmost extended style while `MB_TOPMOST` sets
  `WS_EX_TOPMOST`; the focused
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel message_box_w_records_text_owner_and_returns_default_button`
  passed after the update.
- `tests/coredll_raw_gwe.rs`: expanded raw `TrackPopupMenuEx`
  `TPMPARAMS.rcExclude` coverage from CE `winuser.h`, proving the top-level
  popup can move below an excluded screen rectangle, fall back to the right side
  when below/above candidates still intersect the excluded region after screen
  clamping, and fall back to the left side when the right candidate clamps back
  into the excluded region. The same test now also covers the final
  first-clamped-candidate fallback when every candidate still intersects a
  full-screen exclusion rectangle.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_track_popup_menu_ex_uses_tpmp_params_exclude_rect_for_initial_position`
  passed for the `TPMPARAMS.rcExclude` all-candidates fallback coverage update.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  `TPMPARAMS.rcExclude` all-candidates fallback coverage update; the eVC4 MIPSII
  fixture test remains ignored because the toolchain is not configured.
- `tests/coredll_raw_gwe.rs`: expanded cross-thread
  `SendMessageTimeout` coverage so `SMTO_BLOCK | SMTO_ABORTIFHUNG` queues a
  timeout transaction when the receiver is just below the CE hung threshold,
  complementing the existing hung-receiver abort-without-queue coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_send_message_timeout_nonzero_cross_thread_queues_transaction`
  passed for the cross-thread `SMTO_ABORTIFHUNG` below-threshold queueing
  update.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  cross-thread `SMTO_ABORTIFHUNG` below-threshold queueing update; the eVC4
  MIPSII fixture test remains ignored because the toolchain is not configured.
- `tests/coredll_raw_gwe.rs`: expanded same-thread
  `SendMessageTimeout` coverage so `SMTO_BLOCK | SMTO_ABORTIFHUNG` still
  dispatches synchronously and writes `lpdwResult` even when the target thread's
  last-dispatch timestamp satisfies the CE hung threshold. This pins the local
  split between same-thread direct send behavior and cross-thread hung-abort
  behavior.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_send_message_timeout_same_thread_dispatches_synchronously`
  passed for the same-thread `SMTO_ABORTIFHUNG` non-abort coverage update.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  same-thread `SMTO_ABORTIFHUNG` non-abort coverage update; the eVC4 MIPSII
  fixture test remains ignored because the toolchain is not configured.
- `tests/coredll_raw_kernel.rs`: added raw `ImageList_SetOverlayImage`
  coverage from CE `imagelist.cpp`, proving an all-white overlay mask records
  the CE zero-sized `(0, 0)` overlay bounds with no `ILD_IMAGE` flag, leaves
  base framebuffer pixels untouched when drawn through `ImageList_DrawEx`, and
  keeps sparse non-rectangular overlay masks in the mask-driven path instead of
  promoting the full black-pixel bounding box to `ILD_IMAGE`.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons`
  passed for the CE all-white and non-rectangular overlay-mask coverage update.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  CE all-white and non-rectangular overlay-mask coverage update; the eVC4
  MIPSII fixture test remains ignored because the toolchain is not configured.
- `git diff --check` passed after the CE all-white and non-rectangular overlay-mask
  coverage update; output was limited to existing LF-to-CRLF normalization
  warnings.
- `tests/coredll_raw_kernel.rs`: expanded raw `KernExtractIcons` coverage from
  CE `resource.cpp`, proving the ordinal resolves exact integer
  `RT_GROUP_ICON` resource IDs rather than zero-based group enumeration, copies
  the selected large/small `RT_ICON` payloads for sparse group IDs, and reports
  `ERROR_RESOURCE_NAME_NOT_FOUND` when a valid group is found but neither
  output pointer is supplied.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel coredll_raw_kern_extract_icons_copies_group_rt_icon_payloads`
  passed for the raw `KernExtractIcons` exact-ID/no-output coverage update.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  raw `KernExtractIcons` exact-ID/no-output coverage update; the eVC4 MIPSII
  fixture test remains ignored because the toolchain is not configured.
- `tests/coredll_raw_memory_file.rs`: added mounted cross-parent rename
  notification coverage from CE `fsnotify.cpp::NotifyMoveFileEx`, proving
  recursive mounted-volume watches receive `FILE_ACTION_REMOVED` plus
  `FILE_ACTION_ADDED` relative paths, sibling mounted-volume watches stay
  quiet for the same child names, and recursive root watchers receive the
  mounted-volume-prefixed remove/add paths.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_mounted_cross_parent_rename_notifications_are_volume_scoped`
  passed for the mounted cross-parent rename notification coverage update.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  mounted cross-parent rename notification coverage update; the eVC4 MIPSII
  fixture test remains ignored because the toolchain is not configured.
- `src/ce/com.rs`, `src/ce/shell.rs`, and `src/ce/kernel.rs`: shell
  notification link/dismiss/command callback records now mirror the CE taskbar
  `GetCallbackInterface` acquisition path when a notification carries only a
  CLSID. The shell stores the CE
  `IID_IShellNotificationCallback` GUID from `shellsdkguids.h`, asks the local
  COM registry for that interface by GUID value, records the acquired local
  interface token as `callback_ptr`, and now skips COM callback records for
  unregistered classes so those events fall back to the sink-window path like
  CE `CHtmlBubble::GetCallbackInterface`.
- `tests/basic_subsystems.rs`: direct COM coverage now includes value-only
  CLSID/IID object creation metadata, and shell notification coverage asserts a
  registered notification CLSID queues the acquired
  `IShellNotificationCallback` token plus IID metadata.
- `src/emulator/unicorn.rs`: added a positive Unicorn regression for a mapped
  guest `IShellNotificationCallback` interface pointer/vtable. The dispatcher
  now has coverage showing it redirects PC/T9 to the selected method, supplies
  `this`, ID, timeout, and lParam in the CE MIPS calling convention, installs
  the COM return stub, adjusts the stack, and records pending return metadata.
- `cargo test -j 1 --features unicorn,trace,win32-desktop com_system_initialize_uninitialize_register_class_create_instance_and_object`
  `cargo test -j 1 --features unicorn,trace,win32-desktop kernel_post_shell_notify_icon_and_notification_callbacks_expire`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop shell_notification_com_dispatch_enters_mapped_guest_callback`
  passed after binding registered shell-notification CLSIDs to the COM-acquired
  callback interface token and proving the mapped guest vtable callout path.
- `src/ce/com.rs`, `src/ce/ole.rs`, and `src/emulator/imports.rs`: OLE
  `CoCreateInstance` now prefers CE-style CLSID/IID GUID bytes read from guest
  memory and resolves classes registered by GUID value, so two different guest
  pointers containing the same CLSID can acquire the same registered class. The
  returned pointer is the registered local interface token when one exists,
  which lets the COM registry stand in for the guest `IShellNotificationCallback`
  pointer that CE taskbar `GetCallbackInterface` acquires. The previous
  pointer-token class path remains as a fallback for existing local callers.
- `tests/basic_subsystems.rs`: direct COM coverage now asserts
  `CoCreateInstance` returns the registered interface token instead of ignoring
  it.
- `cargo test -j 1 --features unicorn,trace,win32-desktop ole_cocreateinstance_import_uses_com_registry_and_writes_ppv`
  and `cargo test -j 1 --features unicorn,trace,win32-desktop com_system_initialize_uninitialize_register_class_create_instance_and_object`
  passed after adding GUID-value `CoCreateInstance` import coverage and returned
  interface-token semantics.
- `cargo test -j 1 --features unicorn,trace,win32-desktop imports::tests`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test basic_subsystems`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  GUID-value OLE registry and returned-interface-token changes; the eVC4 MIPSII
  fixture remains ignored because that toolchain is not configured.
- `src/ce/gwe.rs`: sent-message queue readiness and retrieval now apply the
  caller's HWND and message-range filters for `has`, peek, and remove paths, so
  a pending cross-thread `SendNotifyMessage` for a top-level broadcast target no
  longer satisfies an unrelated child-window-filtered receive.
- `cargo test -j 1 --features unicorn,trace,win32-desktop ce::gwe::tests::sent_message_dispatch_honors_hwnd_and_message_filters`
  passed after tightening the direct sent-message filter unit.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_send_notify_broadcast_uses_notify_send_for_live_top_level_windows`
  and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed after fixing sent-message queue filtering.
- `cargo check --features unicorn,trace,win32-desktop` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the OLE
  `CoCreateInstance` import writeback slice and sent-message filter repair; the
  eVC4 MIPSII fixture remains ignored because that toolchain is not configured.
- `src/ce/ole.rs` and `src/emulator/imports.rs`: OLE import dispatch now routes
  `CoInitialize`, `CoInitializeEx`, and `CoUninitialize` through the local COM
  apartment tracker, and `CoCreateInstance` through the local COM class registry
  with `ppv` zeroing/writeback and `CLASS_E_NOAGGREGATION` rejection for
  non-null aggregation.
- `src/emulator/imports.rs`: added raw import coverage for `CoCreateInstance`
  null `ppv`, unregistered class failure, successful registry-backed object
  creation, and returned interface pointer writeback.
- Roadmap docs now narrow the remaining shell-notification COM gap to integrated
  Explorer taskbar bubble `GetCallbackInterface` acquisition/dispatch using
  real guest `IShellNotificationCallback` interface pointers, rather than the
  now-covered OLE import `CoCreateInstance` writeback path.
- `src/ce/kernel.rs` and `tests/coredll_raw_kernel.rs`: `SHNN_SHOW` now follows
  the CE taskbar `CHtmlBubble::PopUp` shape by posting the window-based
  `WM_NOTIFY`/`NMSHN` sink notification without queuing an
  `IShellNotificationCallback::OnShow` COM dispatch. Link, dismiss, and command
  events still record COM callback candidates and also notify the sink window,
  matching the taskbar `bubble.cpp` paths that call `GetCallbackInterface` only
  for those interactions.
- `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel shnotification_i_tracks_query_update_and_remove_state`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after aligning
  `SHNN_SHOW` with the CE taskbar window-only notification path; the eVC4 MIPSII
  fixture test remains ignored because the toolchain is not configured. `git
  diff --check` also passed with output limited to existing CRLF normalization
  warnings.
- `src/emulator/unicorn.rs`: added Unicorn-level coverage for
  `IShellNotificationCallback` dispatch failure when a queued non-null callback
  interface pointer is unmapped. The runtime leaves the COM return stack empty
  and restores the callback record to the front of the queue for a later retry,
  matching CE's "try COM, then keep sink notification independent" shape without
  silently dropping bubble callbacks.
- `src/ce/kernel.rs` and `tests/coredll_raw_kernel.rs`: shell notification COM
  callbacks now require either a stored non-null interface pointer or a
  successful local `CoCreateInstance(... IID_IShellNotificationCallback ...)`
  acquisition. Unregistered notification CLSIDs no longer queue zero-pointer
  COM callback records; link/dismiss/command events still notify a live sink
  window when present, matching CE taskbar `CHtmlBubble::GetCallbackInterface`.
- `cargo test shnotification_i_ --test coredll_raw_kernel --features unicorn,trace,win32-desktop -j 1 -- --nocapture`
  passed after the unregistered callback-CLSID fallback regression.
- `cargo fmt --check`,
  `cargo check --features unicorn,trace,win32-desktop -j 1`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed after the unregistered callback-CLSID sink-window
  fallback slice; `git diff --check` output was limited to existing CRLF
  normalization warnings and the eVC4 MIPSII fixture test remains ignored
  because the toolchain is not configured.
- `src/ce/shell.rs` and `tests/coredll_raw_kernel.rs`: shell notification
  title storage now mirrors CE `StringCbCopy` into `CCHMAXTBLABEL`/`MAX_PATH`:
  add/update titles with 260 or more UTF-16 code units clear the stored
  taskbar label instead of retaining an overlong string.
- `cargo fmt --check`,
  `cargo test shnotification_i_clears_overlong_taskbar_titles_like_ce --test coredll_raw_kernel --features unicorn,trace,win32-desktop -j 1 -- --nocapture`,
  and `cargo test --test coredll_raw_kernel --features unicorn,trace,win32-desktop -j 1`
  passed after the CE taskbar title-overflow regression.
- `cargo check --features unicorn,trace,win32-desktop -j 1`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` also passed after the taskbar title-overflow slice;
  `git diff --check` output was limited to existing CRLF normalization
  warnings and the eVC4 MIPSII fixture test remains ignored because the
  toolchain is not configured.
- `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop shell_notification_com_dispatch_restores_unmapped_callback_pointer`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding
  the unmapped callback-interface pointer regression; the eVC4 MIPSII fixture
  test remains ignored because the toolchain is not configured. `git diff
  --check` also passed with output limited to the repository's existing CRLF
  normalization warnings.
- `src/ce/kernel.rs` and `tests/coredll_raw_kernel.rs`: `DispatchMessageW`
  now releases heap-backed private pointer payloads only when the dispatched
  message ID matches the stored payload type. A spoofed
  `WM_HANDLESHELLNOTIFYICON` with a `WINDOWPOS` payload no longer frees the
  wrong allocation; the matching `WM_WINDOWPOSCHANGED` dispatch still releases
  it.
- `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel dispatch_message_releases_only_matching_private_pointer_payload_type`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel`
  passed after the private pointer payload release guard update; the full
  `cargo check --features unicorn,trace,win32-desktop` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` suite also passed,
  with the eVC4 MIPSII fixture test still ignored because the toolchain is not
  configured.
- `src/ce/shell.rs` and `tests/coredll_raw_kernel.rs`: explicit
  `SHNotificationRemoveI` and sink window/process cleanup now record copied
  icon destruction for iconic notification records, matching the CE
  `NIM_BUBBLE_DELETE` path through `DeleteItem(..., TRUE)` and the existing
  expiration cleanup rule.
- `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel shnotification_i_tracks_query_update_and_remove_state`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel shell_window_destroy_removes_notify_icon_and_notification_state`
  passed after the iconic `SHNotificationRemoveI` and sink-cleanup icon
  lifetime update; the full `coredll_raw_kernel` target,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` also passed, with
  the eVC4 MIPSII fixture test still ignored because the toolchain is not
  configured.
- `tests/coredll_raw_kernel.rs`: `Shell_NotifyIcon` registered-taskbar coverage
  now destroys the registered taskbar window after a copied
  `WM_HANDLESHELLNOTIFYICON` payload is dispatched, then verifies a later
  `NIM_MODIFY` still succeeds and updates shell state without queuing a stale
  private taskbar message. This matches the CE sample callback's copy/forward
  shape while keeping stale taskbar HWNDs from leaking queued payloads.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel shell_notify_icon_posts_registered_taskbar_message_with_copied_data`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  stale registered-taskbar `Shell_NotifyIcon` coverage update; the eVC4 MIPSII
  fixture test remains ignored because the toolchain is not configured.
- `src/ce/registry.rs`, `src/config.rs`, and `src/main.rs`: runtime registry
  loading now accepts Windows REGEDIT `.reg` dumps, keeps JSON dump support for
  explicit legacy files, and defaults the emulator to the checked-in
  `registry.reg` copied from the current CE dump export.
- `src/ce/coredll.rs`: `SystemParametersInfoW(SPI_GETOEMINFO)` now keeps the
  emulator-specific override path but falls back to `HKLM\Ident\Name` from the
  imported CE registry dump, matching the CE core DLL device-info routing and
  keeping full-feature tests aligned with `registry.reg`.
- Test fixture loads in `src` unit tests and integration tests now use
  `registry.reg` instead of the removed `regs.json`; the full
  `cargo test -j 1 --features unicorn,trace,win32-desktop` suite passes with
  that fixture.
- `src/ce/kernel.rs`: loaded-module export resolution now suppresses name and ordinal exports for modules flagged with `LOAD_LIBRARY_AS_DATAFILE`, matching CE `loader.c` resource-only load behavior and keeping raw and Unicorn `GetProcAddress` paths aligned.
- `src/ce/shell.rs`: removing a shell notification by `(CLSID, id)` or by sink-window/process cleanup now removes queued `IShellNotificationCallback` records for that notification so stale callbacks cannot survive after the CE taskbar/bubble record is gone.
- `src/ce/coredll.rs`: `SHNotificationGetDataI` now ignores `cbTitle` for non-null title output and writes through the fixed CE taskbar title capacity (`MAX_PATH`), matching `notification.cpp`'s `CCHMAXTBLABEL` assumption.
- `src/ce/shell.rs`: `SHNotificationUpdateI` with `SHNUM_ICON` and a non-null replacement icon now records the previous owned notification icon for destruction before replacing it, matching `notification.cpp::UpdateBubble`.
- `src/ce/shell.rs`: expired iconic `SHNotification` records now record destruction of their copied notification icon, matching CE `UpdateTimedNotificationIcons` setting `HHTBF_DESTROYICON` before `DeleteItem(..., TRUE)`.
- `src/ce/kernel.rs` and `src/ce/coredll.rs`: public `FindFirstChangeNotificationW` setup now canonicalizes the CE watch path before directory validation and handle registration, so unrooted watch paths and `.`/`..` components resolve like the CE FSDMGR `SafeGetCanonicalPathW` path.
- `src/ce/file.rs`, `src/ce/object.rs`, and `src/ce/kernel.rs`: file-change notification handles now retain the resolved mounted-volume root from registration and require later filesystem changes to come from the same volume unless the watch is the root namespace. This follows CE `fsnotify.cpp` storing events under a `NOTVOLENTRY` while preserving root recursive watches over visible mounted folders.
- `tests/coredll_raw_memory_file.rs`: added a mounted-volume notification test where `\ResidentFlash\watch` ignores a same-child-path mutation under `\SDMMC Disk\watch`, while a recursive root watcher receives `SDMMC Disk\...` and `ResidentFlash\...` paths with their mount prefixes.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file mounted_volume_change_notifications_are_volume_scoped`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  CE mounted-volume notification ownership update; the eVC4 MIPSII fixture test
  remains ignored because the toolchain is not configured.
- `src/ce/coredll.rs` and `src/ce/kernel.rs`: raw `DuplicateHandle` now carries
  the requested target-process identity into duplicated file-change notification
  handles instead of restamping them with the source/current process, matching
  CE's per-process duplicated notification/event handle ownership model.
- `tests/coredll_raw_memory_file.rs`: added coverage for duplicating a
  notification handle into another process, rejecting source-process reset on
  the duplicate, and allowing the target process to wait and drain the detailed
  file-change record.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file duplicate_handle_retargets_notification_owner`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  target-process `DuplicateHandle` notification ownership update; the eVC4
  MIPSII fixture test remains ignored because the toolchain is not configured.
- `src/ce/file.rs` and `src/ce/kernel.rs`: cross-volume `MoveFileW` now follows
  CE `FS_MoveFileW` copy/delete semantics more closely by allowing a copy from
  read-only source media to a writable destination, treating source deletion as
  best-effort, and only reporting a source deletion notification if the source
  path is actually gone after the move.
- `tests/coredll_raw_memory_file.rs`: added coverage for moving a file from a
  read-only mounted volume to `\ResidentFlash`, verifying the destination copy,
  preserved source file, destination add record, and quiet source watch.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file cross_volume_move_from_readonly_source_copies_without_delete`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`
  passed after the CE cross-volume copy/delete `MoveFileW` update.
- `cargo check --features unicorn,trace,win32-desktop` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` also passed after
  the CE cross-volume copy/delete `MoveFileW` update; the eVC4 MIPSII fixture
  test remains ignored because the toolchain is not configured.
- `src/ce/file.rs`, `src/ce/kernel.rs`, and `src/ce/coredll.rs`:
  `DeleteAndRenameFile` and direct `AFS_PrestoChangoFileName` now use a
  dedicated CE-style delete-and-rename path: validate same-volume/mount-root
  edges before deleting, delete the old path, move the new path into the old
  name, and emit the FSDMGR delete-plus-move notification sequence.
- `tests/coredll_raw_memory_file.rs`: added public `DeleteAndRenameFile`
  coverage for replacement direction, failure without deleting the destination
  when the replacement source is missing, and the CE detailed notification
  sequence; the AFS path ordinal test now verifies PrestoChango's same
  delete-old/move-new direction.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file delete_and_rename_file_replaces_destination_atomically`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file afs_path_ordinals_use_ce_file_namespace`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`
  passed after the CE DeleteAndRename/PrestoChango update.
- `cargo check --features unicorn,trace,win32-desktop` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` also passed after
  the CE DeleteAndRename/PrestoChango update; the eVC4 MIPSII fixture test
  remains ignored because the toolchain is not configured.
- `src/ce/object.rs` and `src/ce/kernel.rs`: file handles now track whether a
  successful write changed the handle, and `CloseHandle` emits CE's
  `FILE_ACTION_CHANGE_COMPLETED` detailed file-change record using the
  FSDMGR `NotifyCloseHandle` attribute/size/write/access/creation filter mask.
  Remove coalescing now also drops prior close-completed records for the same
  path so write/delete churn still collapses to the final removal.
- `tests/coredll_raw_memory_file.rs`: added close-completion coverage showing a
  write reports `FILE_ACTION_MODIFIED`, while closing that changed handle
  reports `FILE_ACTION_CHANGE_COMPLETED`; existing detailed notification
  coverage now expects the CE close-completed record after write+close.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file changed_file_close_reports_change_completed`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file change_notification_coalesces_transient_name_churn`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`
  passed after the CE close-completed file-notification update.
- `cargo fmt --check`, `cargo check --features unicorn,trace,win32-desktop`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after
  the CE close-completed file-notification update; the eVC4 MIPSII fixture test
  remains ignored because the toolchain is not configured.
- `src/ce/file.rs`: host-backed files and directories below a configured system
  mount now inherit `FILE_ATTRIBUTE_SYSTEM`, matching CE `pathapi.cpp` for
  `AFS_FLAG_SYSTEM` volumes instead of only marking the mount-root entry.
- `tests/coredll_raw_memory_file.rs`: added raw coverage for system-volume
  nested file attributes, hidden mount suppression from root enumeration, exact
  hidden mount-root attributes, system/hidden `CeGetVolumeInfoW` attribute
  reporting from the FSDMGR mount flags, and read-only/removable volume
  attributes through `CeGetVolumeInfoW` plus
  `CeFsIoControlW(FSCTL_GET_VOLUME_INFO)`.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_system_and_hidden_mounts_follow_fsdmgr_attributes`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop mount_without_host_root_inherits_default_root_backing`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`
  passed after the CE system/hidden mount attribute update.
- `cargo fmt --check`, `cargo check --features unicorn,trace,win32-desktop`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop` also passed
  after the CE system/hidden mount attribute update; the eVC4 MIPSII fixture
  test remains ignored because the toolchain is not configured.
- `tests/coredll_raw_memory_file.rs`: added mounted same-parent rename
  notification coverage showing the owning mounted-volume watch receives CE
  old/new rename records, a same-child path under a different mounted volume
  stays quiet, and a recursive root watch receives mount-prefixed old/new
  paths.
- `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_mounted_same_parent_rename_notifications_are_volume_scoped`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  mounted same-parent rename notification coverage update; the eVC4 MIPSII
  fixture test remains ignored because the toolchain is not configured.
- `src/ce/kernel.rs`: `MoveFileW` file-change notification records now follow CE `NotifyMoveFileEx`, preserving rename old/new actions only for same-parent moves and reporting cross-parent moves as remove/add with the correct file-vs-directory notify filter. Watches on the directory being renamed or removed now follow CE `NotifyPathChangeEx` by reporting `FILE_ACTION_REMOVED` for `"\\"`.
- `src/emulator/imports.rs`: forwarded-export parsing now treats PE forwarder strings as literal module/symbol targets, rejecting whitespace-padded and missing-ordinal malformed strings instead of trimming them into resolvable imports.
- `src/emulator/unicorn.rs`: runtime `LoadLibraryW` failure cleanup now tracks modules loaded by the current attempt and releases them in reverse order when dependency mapping/import patching or lifecycle callout setup fails, matching CE `loader.cpp` import-block unwind behavior.
- `src/emulator/unicorn.rs`: runtime dependency loading now retains already-loaded direct dependency modules, and final dynamic `FreeLibrary` plans target-plus-dependency release after detach callouts so imported dependency refs are unwound like CE `UnDoDepends`.
- `src/emulator/memory.rs` and `src/emulator/unicorn.rs`: runtime DLL mapping now has exact current-image unmap cleanup, so late map/write/metadata/forwarder/trap/resource failures remove the image's `MemoryMap` and Unicorn reservation before the loader failure propagates.
- `src/ce/resource.rs` and `src/emulator/unicorn.rs`: runtime resource parsing now completes before current-image trap/trampoline/module/resource commits, and resource state has a module-scoped cleanup path for failed loader attempts.
- `src/ce/thread.rs` and `src/emulator/unicorn.rs`: runtime `DllMain(DLL_PROCESS_ATTACH)` false returns now follow CE `CallDllMain` failure shape by setting `ERROR_DLL_INIT_FAILED`, recording a loud loader failure, and releasing newly loaded plus retained dependency refs owned by that load attempt.
- `src/ce/kernel.rs`, `src/ce/coredll.rs`, and `src/emulator/unicorn.rs`: `LoadLibraryExW` now rejects non-null `hFile` handles with `ERROR_INVALID_PARAMETER`, and already loaded `DONT_RESOLVE_DLL_REFERENCES` modules are promoted to normal loaded-module flags when a later normal load requests them without CE no-import flags.
- `src/ce/kernel.rs`, `src/ce/coredll.rs`, `src/emulator/unicorn.rs`, and `src/main.rs`: loaded modules now retain CE load-order and no-thread-calls metadata; raw `DisableThreadLibraryCalls` sets the module flag; Unicorn `ThreadAttachAllDLLs` and `ThreadDetachAllDLLs` now enter guest DLL entrypoints in load order while skipping no-import/datafile and disabled modules.
- `src/ce/kernel.rs` and `src/emulator/unicorn.rs`: runtime `ProcessDetachAllDLLs` now plans `DLL_PROCESS_DETACH` callouts with CE `ProcessDetach` min-refcount ordering, does not let `DisableThreadLibraryCalls` suppress process detach, skips no-import/datafile modules, and releases the included modules' per-process refs after lifecycle callbacks complete.
- `src/ce/kernel.rs` and `src/ce/coredll.rs`: raw `ProcessDetachAllDLLs` now drains eligible loaded-module refs for raw/non-Unicorn parity with the runtime process-detach state transition, while preserving no-resolve/datafile modules because they never received process attach.
- `src/ce/kernel.rs`: file-change notifications now track an outstanding signal count separately from detailed pending records, matching the CE `NotifyReset` path where `FindNextChangeNotification` consumes one pending signal without discarding every queued notification record.
- `src/ce/file.rs` and `src/ce/coredll.rs`: `MoveFileW` now classifies source and destination guest volumes before host translation, disallows mount-point renames, emulates cross-volume file moves by copy/delete, rejects cross-volume directory moves with `ERROR_NOT_SAME_DEVICE`, and preserves CE-specific raw last-error values.
- `src/ce/file.rs`, `src/ce/kernel.rs`, and `src/ce/coredll.rs`: mounted roots now follow CE `pathapi.cpp` guards for directory create (`ERROR_ALREADY_EXISTS`), directory removal (`ERROR_ACCESS_DENIED`), delete-as-file (`ERROR_FILE_NOT_FOUND`), and attribute changes (`ERROR_ACCESS_DENIED`). Read-only mounted roots also surface `ERROR_ACCESS_DENIED` for raw mutating paths including write `CreateFileW`, copy destinations, file deletion, child directory creation/removal, rename, and attribute changes instead of collapsing to generic invalid-argument or file-not-found failures; the failed mutations do not queue CE file-change notifications.
- `src/ce/kernel.rs`: root file-change watches now honor the CE `WatchSubtree` flag instead of treating `\` as a universal match for non-recursive handles; non-recursive root watches match immediate root children only, while recursive root watches still match deeper descendants.
- `src/ce/kernel.rs`: `FindCloseChangeNotification` now closes a valid non-notification handle before returning `ERROR_INVALID_HANDLE`, matching CE's public `NotifyCloseChangeHandle` path that duplicates the caller handle with `DUPLICATE_CLOSE_SOURCE` before checking notification event data.
- `src/ce/file.rs`, `src/ce/kernel.rs`, and `src/ce/coredll.rs`: raw `DuplicateHandle` now validates source handles, target output pointers, and CE-supported option bits, creates independent handle-table entries instead of aliasing the source handle value, duplicates file and find backing IDs, and honors `DUPLICATE_CLOSE_SOURCE` so duplicated notification handles remain usable after the source is consumed.
- `src/ce/object.rs`, `src/ce/kernel.rs`, and `src/ce/coredll.rs`: file-change notification handles now store the creating process ID from `FindFirstChangeNotificationW` and enforce it on wait polling, `FindNextChangeNotification`, `CeGetFileNotificationInfo`, `DuplicateHandle`, and `FindCloseChangeNotification`, matching CE `NotifyCreateEvent`/`NotifyGetNextChange` ownership through caller-process handle duplication.
- `src/ce/kernel.rs` and `src/ce/coredll.rs`: direct `AFS_FindFirstChangeNotificationW` now passes its nonzero `hProc` argument into notification-handle ownership, matching the CE `pathapi.cpp` path that forwards caller/current process identity to `NotifyCreateEvent`.
- `src/ce/shell.rs`, `src/ce/kernel.rs`, `src/ce/coredll.rs`, and `src/ce/gwe.rs`: `RegisterTaskBar` now tracks or clears the CE taskbar window, successful `Shell_NotifyIcon` calls post `WM_HANDLESHELLNOTIFYICON` to that taskbar with a heap-backed copied `NOTIFYICONDATAW` message payload, message materialization writes the copied payload back to guest memory for taskbar consumers, and `DispatchMessageW` releases that private taskbar payload after delivery like `mintask.cpp`.
- `src/main.rs`: the saved remote-input target unit fixture now registers its saved `GetMessage` waiter against a live visible window, matching the current remote-input selection rule that ignores stale or invisible HWND filters.
- `src/emulator/unicorn.rs`, `src/emulator/cpu.rs`, `src/emulator/stub.rs`,
  and `src/main.rs`: cross-process `SendMessageW`/`SendMessageTimeout`
  runtime handoff now clears stale send-yield debug snapshots once the matching
  GWE sent-message record is gone, so remote and monitor process rotation use
  current CE sent-message queue state instead of an orphaned stop snapshot.
- `src/ce/coredll.rs`: raw `SHNotificationAddI` now treats the second marshalled argument as CE `cbData` only and no longer records it as an `IShellNotificationCallback*`; callback metadata is derived from the notification CLSID path like CE `bubble.cpp`.
- `src/ce/shell.rs` and `src/emulator/unicorn.rs`: runtime `IShellNotificationCallback` dispatch now restores non-null pending callback records to the front of the shell queue if the guest COM vtable or method pointer cannot be entered yet, while still consuming null/no-interface callback records as CE no-ops.
- `src/ce/shell.rs` and `src/ce/coredll.rs`: raw `SHNotificationAddI` now validates notification title/HTML content using CE marshalled pointer presence, so `SHNP_INFORM` rejects a null HTML pointer but accepts a non-null empty HTML string.
- `src/ce/shell.rs`: shell notifications now maintain CE-style inform and iconic priority lists, moving records between lists on `SHNUM_PRIORITY` updates and removing list entries when notifications are removed, expired, or cleaned up with their sink windows.
- `src/ce/coredll.rs`: `ExtractIconExW` reads guest paths, validates files, extracts PE icon resources when available, enumerates `RT_GROUP_ICON` data through a shared helper for count/extract behavior, falls back to CE-style integer `RT_GROUP_ICON` resource ID lookup for sparse indexes, selects distinct large/small icons from multi-size PE icon groups, fills successive large/small icon output-array slots, reports `ERROR_RESOURCE_NAME_NOT_FOUND` for malformed present PE group/icon resources, falls back to shell icons for non-PE index zero, and supports bitmap-backed icon rendering through `DrawIconEx`.
- `src/ce/coredll.rs`: raw `KernExtractIcons` now follows CE `resource.cpp` by resolving integer `RT_GROUP_ICON` resource IDs from datafile-loaded PE resources and copying the selected `RT_ICON` payload bytes into guest heap outputs instead of returning the old unsupported stub.
- `src/ce/coredll.rs`: `DrawIconEx` now scales bitmap-backed icons from their native bitmap dimensions into caller-requested destination rectangles for both framebuffer and selected-memory-DIB targets instead of treating the requested destination size as the source extent, and honors bitmap-backed `DI_MASK` by selecting the icon mask bitmap as the draw source.
- `src/ce/resource.rs`: `ImageList_Create` now applies the CE `ILC_VALID` flag mask, rejecting unsupported creation flags such as `ILC_LARGESMALL`, `ILC_UNIQUE`, and unknown high bits before allocating image-list state.
- `src/ce/resource.rs`: `ImageList_Create` now follows CE `imagelist.cpp` by normalizing a missing color-depth mask (`ILC_COLOR`) to `ILC_COLORDDB` before storing image-list flags.
- `src/ce/resource.rs`: `ImageList_Add` and `ImageList_AddMasked` now follow CE `imagelist.cpp` by failing when the source bitmap is invalid or narrower than the image-list slot width instead of returning a success index with no appended images.
- `src/ce/resource.rs`: `ImageList_ReplaceIcon` now follows CE `imagelist.cpp` by accepting only `-1` as the append sentinel and rejecting indexes below `-1`.
- `src/ce/coredll.rs`: raw `ImageList_GetIconSize` now follows CE `imagelist.cpp` by validating both output pointers before writing either dimension, avoiding partial `cx` writes when `cy` is null.
- `src/ce/coredll.rs`: raw `ImageList_LoadImage` now follows CE `imagelist.cpp` by using the loaded bitmap height as the image width when callers pass `cx == 0`, while letting negative `cx` fail through image-list creation instead of widening to the full bitmap width.
- `src/ce/coredll.rs`: raw `ImageList_LoadImage` now follows CE `imagelist.cpp` by creating image lists with `ILC_MASK` when `crMask != CLR_NONE` and with the loaded bitmap bit depth in the `ILC_COLORMASK` bits.
- `src/ce/coredll.rs`: raw `ImageList_LoadImage` now follows CE `imagelist.cpp` by using the unmasked `ImageList_Add` path when `crMask == CLR_NONE`, leaving image-list entries with mask handle `0` and no transparent color.
- `src/ce/resource.rs`: `ImageList_Merge` metadata now follows CE `imagelist.cpp` by sizing the merged list from the union of both source image rectangles and combining mask/color flags from both lists.
- `src/ce/resource.rs`: `ImageList_Copy` now applies CE `ILCF_VALID` flag validation, rejects CE-unsupported cross-list copies, copies only between valid same-list slots for `ILCF_MOVE`, and swaps same-list slots for `ILCF_SWAP`.
- `src/ce/coredll.rs`: bitmap-backed `ImageList_Draw*` blending now models CE `rgbFg == CLR_NONE` destination blending by combining source pixels with the existing destination pixels instead of treating `CLR_NONE` as no blend.
- `src/ce/coredll.rs`: bitmap-backed `ImageList_Draw*` mask-only blending now follows CE `imagelist.cpp` by forcing `ILD_TRANSPARENT` for `rgbFg == CLR_NONE`, ORing mask pixels with the 50% mono-dither pattern, and drawing through the `SRCAND` mask path instead of tinting mask pixels against the destination.
- `src/ce/coredll.rs`: bitmap-backed `ImageList_Draw*` blending now uses CE's private `ILD_BLENDMASK == 0x000E` shape, so `ILD_BLEND75` enters the blend path and follows CE's non-`ILD_BLEND50` 25% branch.
- `src/ce/coredll.rs`: `ImageList_Draw*` setup now follows CE `imagelist.cpp` by resolving `rgbBk == CLR_DEFAULT` to the image-list background color and defaulting zero draw `cx`/`cy` after applying `xBitmap`/`yBitmap`.
- `src/ce/coredll.rs`: raw `ImageList_Draw` now follows CE `imagelist.cpp` by using the wrapper default `rgbBk = CLR_DEFAULT`, so masked pixels fill from the image-list background color unless callers explicitly pass `ILD_TRANSPARENT`.
- `src/ce/coredll.rs`: raw `ImageList_DrawIndirect` now mirrors CE's in-place `IMAGELISTDRAWPARAMS` normalization by writing defaulted `cx`/`cy`, resolved `rgbBk`, normalized `fStyle`, and final overlay-pass `i`/`x`/`y`/`cx`/`cy`/`fStyle` values back to the guest struct before rendering.
- `src/ce/coredll.rs` and `src/ce/resource.rs`: bitmap-backed `ImageList_DrawIndirect` now carries `IMAGELISTDRAWPARAMS.dwRop` and applies CE `ILD_ROP` raster operations for `ILD_MASK`/`ILD_IMAGE` selected-DIB and framebuffer draws, including CE's default `ILD_MASK | ILD_TRANSPARENT` `SRCAND` branch.
- `src/ce/coredll.rs` and `src/ce/resource.rs`: raw `ImageList_CopyDitherImage` now follows CE by preserving destination image metadata, masking `fStyle` to `ILD_OVERLAYMASK`, copying source pixels into bitmap-backed destination image storage, and applying CE's 50% mono-dither/SRCAND update to destination mask storage when backing bits are available.
- `src/ce/coredll.rs` and `src/ce/resource.rs`: raw `ImageList_Duplicate` now follows CE `imagelist.cpp` by deep-copying bitmap-backed image and mask storage into fresh owned bitmaps while preserving list metadata, overlays, and pseudo-handle entries.
- `src/ce/coredll.rs`: raw `ImageList_Destroy` now follows CE `ImageList::Cleanup` by releasing image-list-owned bitmap/mask backing handles and freeing their heap-backed bits while leaving caller-owned bitmap handles alone.
- `src/ce/coredll.rs` and `src/ce/resource.rs`: raw `ImageList_Replace`, `ImageList_ReplaceIcon`, `ImageList_Remove`, `ImageList_SetIconSize`, and `ImageList_SetImageCount` now clean up cloned image-list-owned bitmap backing handles only after the successful mutation leaves those handles unreferenced, preserving shared strip storage that still has live entries; replacing a slot with an icon now clears the old bitmap/mask metadata instead of leaving bitmap rendering to win over the icon.
- `src/ce/coredll.rs`: raw `ImageList_SetBkColor` now follows CE `ImageList::ResetBkColor` for bitmap-backed masked entries by rewriting mask-on backing pixels to black/white/the selected background color while preserving mask-off source pixels; `ILD_IMAGE` draws now observe the reset backing pixels.
- `src/ce/coredll.rs`: raw `ImageList_GetIcon` now follows CE `ImageList::GetIcon` for bitmap-backed entries with real backing storage by rendering a mono mask pass and transparent color pass into new icon bitmaps instead of returning a bitmap-derived pseudo handle.
- `src/ce/coredll.rs` and `src/ce/resource.rs`: icon objects now track owned color/mask bitmap backing; raw `CreateIconIndirect` copies readable caller bitmaps into owned icon storage like CE, and `DestroyIcon` frees only icon-owned backing so `ImageList_GetIcon`/PE icons clean up without deleting caller-owned bitmaps.
- `src/ce/coredll.rs`: synthetic shell/system image-list pseudo icons now ignore invalid overlay-mask slots above CE's four overlay entries, matching `OVERLAYMASKTOINDEX`/`NUM_OVERLAY_IMAGES` behavior for both `ImageList_GetIcon` handles and pseudo-rendered `ImageList_DrawEx` output.
- `src/ce/gwe.rs`: filtered GWE sent-message retrieval now applies the requested HWND/min/max range before reporting, peeking, or removing receiver-side sends, so queued `SendNotifyMessage` traffic does not satisfy unrelated modal or shell notification filters.
- `src/ce/resource.rs`: `ImageList_Remove` now follows CE `imagelist.cpp` by treating only `-1` as remove-all and rejecting other negative indexes without changing the image count.
- `src/ce/resource.rs`: `ImageList_Remove` and `ImageList_SetImageCount` now follow CE `imagelist.cpp` overlay-slot lifetime semantics by leaving overlay indexes intact for single-image removal and truncation, while `ImageList_Remove(-1)` still clears all overlay slots.
- `src/ce/resource.rs` and `src/ce/coredll.rs`: `ImageList_SetImageCount` now models CE's `ReAllocBitmaps` success gate by returning failure and preserving the existing count when the requested backing allocation cannot be satisfied; the raw ordinal now reports `ERROR_INVALID_PARAMETER` for that failure path.
- `src/emulator/unicorn.rs`: parked-child `GetMessageW` sent-message callout preparation now honors the existing synthetic `import_pc == 0` sentinel, so unit-test blocked GetMessage states keep the queued notify send for the synthetic resume path while real guest import contexts still enter WndProc callouts.
- `src/ce/resource.rs`: `ImageList_SetOverlayImage` now follows CE `imagelist.cpp` by rejecting lists created without `ILC_MASK` and overlay slot values outside `1..=4`, matching the `m_hdcMask == NULL` and `NUM_OVERLAY_IMAGES` failure paths before an overlay slot is recorded.
- `src/ce/resource.rs` and `src/ce/coredll.rs`: `ImageList_SetOverlayImage` now stores CE overlay metadata beyond the image index, including mask-derived x/y/dx/dy bounds and the rectangular-overlay `ILD_IMAGE` fast path used when drawing overlays.
- `src/ce/coredll.rs`: bitmap-backed `ImageList_Draw*` overlay rendering now follows CE by preserving overlay drawing when `ILD_MASK` is combined with `ILD_OVERLAYMASK`, rendering the overlay mask instead of skipping the overlay pass.
- `src/ce/resource.rs`: `ImageList_SetIconSize` now follows CE `imagelist.cpp` by returning false for unchanged dimensions and clearing all images/overlays through the remove-all path whenever the icon size changes.
- `src/ce/resource.rs`: `ImageList_DragMove` now follows CE `imagelist.cpp` by returning true even when there is no active/visible drag image, while only advancing the stored drag point when an active drag image is visible.
- `src/ce/resource.rs`: `ImageList_SetDragCursorImage` now follows CE `MergeDragImages` by returning true when no drag/dither image exists yet, treating that state as a no-op instead of an error.
- `src/ce/coredll.rs`: `ImageList_GetDragImage` now follows CE `imagelist.cpp` by returning a null drag-image handle with zeroed default points when no drag image is active, instead of reporting an invalid-handle error.
- `src/ce/resource.rs` and `src/ce/coredll.rs`: no-active `ImageList_DragEnter`/`ImageList_DragLeave` now follow CE's static `s_DragContext` lock and point state, so `DragEnter` can succeed before `ImageList_BeginDrag` and later no-active `ImageList_GetDragImage` reports that stored point.
- `src/ce/coredll.rs`: `ImageList_DrawIndirect` now rejects undersized or oversized `IMAGELISTDRAWPARAMS` records before reading optional fields, recording draw state, or rendering, matching CE `imagelist.cpp`'s exact-struct-size gate.
- `src/ce/coredll.rs`: raw `GetTextFaceW` now follows CE GDIAPI parameter edges by returning the selected face-name length including the trailing NUL for null output buffers and rejecting negative character counts with `ERROR_INVALID_PARAMETER`.
- `src/ce/coredll.rs`: raw `GetTextExtentExPointW` now follows CE GDIAPI invalid-parameter behavior for null output `SIZE`, null input text with a positive count, and negative character counts.
- `src/ce/coredll.rs`: raw `SetBkMode`/`GetBkMode` now follow CE GDIAPI `PassNull2da` invalid-HDC behavior by returning `0` and setting `ERROR_INVALID_HANDLE` for null and bad DC handles before touching DC state.
- `src/ce/coredll.rs`: raw `GetBkColor` now follows CE GDIAPI device-attribute invalid-HDC behavior by returning `CLR_INVALID` and setting `ERROR_INVALID_HANDLE`.
- `src/ce/coredll.rs`: raw `SetBkColor`/`GetBkColor` and `SetTextColor`/`GetTextColor` now follow CE GDIAPI `PassNull2da` invalid-HDC behavior and `AlphaCheckGetSetColor` behavior for `CLR_INVALID`, preserving the special color value while reporting `ERROR_INVALID_PARAMETER` only on valid DCs that actually hold `CLR_INVALID`.
- `src/ce/coredll.rs`: raw `SetTextAlign`/`GetTextAlign` now follow CE GDIAPI `passNull2Text` invalid-HDC behavior by returning `GDI_ERROR` and setting `ERROR_INVALID_HANDLE` for null and bad DC handles before touching DC state.
- `src/ce/coredll.rs`: raw `ExtTextOutW` now follows CE GDIAPI text background-mode expectations by filling selected-memory-DIB and framebuffer text cells with the DC background color when `SetBkMode(..., OPAQUE)` is active, while explicit `TRANSPARENT` mode keeps glyph-only rendering.
- `src/ce/coredll.rs`: raw `ExtTextOutW` now follows CE GDIAPI `passNull2Text` invalid-HDC behavior by returning `FALSE` and setting `ERROR_INVALID_HANDLE` for null and bad DC handles before preserving `ERROR_INVALID_PARAMETER` for null text with a positive count on valid DCs.
- `src/ce/coredll.rs`: raw `BitBlt` and `StretchBlt` now follow CE GDIAPI `draw.cpp` invalid-HDC behavior by rejecting null/bad destination and source DCs with `ERROR_INVALID_HANDLE`, and reject CE-invalid `MAKEROP4(PATCOPY, PATINVERT)`-style ROP4 values with `ERROR_INVALID_PARAMETER`.
- `src/ce/coredll.rs`: raw `BitBlt` and `StretchBlt` now render CE's common source/destination ROP3 operations (`SRCPAINT`, `SRCAND`, `SRCINVERT`, `SRCERASE`, `MERGEPAINT`, `NOTSRCCOPY`, `NOTSRCERASE`) through the shared selected-DIB/framebuffer bitmap renderer, apply `DSTINVERT` directly to selected-DIB and framebuffer destinations, sample the selected brush for CE pattern ROPs (`MERGECOPY`, `PATCOPY`, `PATINVERT`, `PATPAINT`), and evaluate the remaining CE ROP3 byte through the generic truth table for literal `draw.cpp::gnvRop3Array` values.
- `src/ce/coredll.rs`: raw `MaskBlt` now follows CE GDIAPI `draw.cpp` parameter behavior for null/bad destination DCs, null/bad source DCs, invalid or color mask bitmaps, negative mask origins, and masks that cannot cover the requested rectangle, while selected-memory-DIB and framebuffer calls render the common 1bpp `MAKEROP4(DSTCOPY, SRCCOPY)` mask-copy path and generic CE `TestAllRops`-style foreground/background ROP3 bytes from ROP4 values instead of reporting a no-op success.
- `src/ce/coredll.rs`: raw `AlphaBlend` now follows CE GDIAPI `draw.cpp` invalid-HDC and blend-function validation by rejecting null/bad destination and source DCs with `ERROR_INVALID_HANDLE`, rejecting invalid `BlendOp`, nonzero `BlendFlags`, unsupported `AlphaFormat`, and non-32bpp per-pixel alpha with `ERROR_INVALID_PARAMETER`, preserving selected-memory-DIB source-constant-alpha blending, applying CE premultiplied `AC_SRC_ALPHA` and non-premultiplied `AC_SRC_ALPHA_NONPREMULT` math, applying top-down and bottom-up 32bpp per-pixel alpha between selected-memory DIBs, and applying source-constant plus top-down and bottom-up 32bpp per-pixel alpha when the destination is a framebuffer-backed window DC.
- `src/ce/coredll.rs`: raw `Get/SetStretchBltMode`, `Get/SetTextCharacterExtra`, and `Get/SetLayout` now follow CE GDIAPI device-attribute sentinel behavior for null/bad HDCs and invalid parameters; invalid stretch modes and `SetTextCharacterExtra(INT_MIN)` fail without changing state, and layout state is constrained to CE layout flags so `GDI_ERROR` remains an error sentinel.
- `src/ce/coredll.rs`: raw exported `SetViewportOrgEx` now follows CE GDIAPI `PassNull2da` invalid-HDC behavior by returning `FALSE` and setting `ERROR_INVALID_HANDLE`, while valid DCs keep the current CE-compatible no-op origin semantics and optional previous-origin output.
- `src/ce/coredll.rs`: raw `SetBrushOrgEx` now follows CE GDIAPI `passBrushNULL` invalid-HDC behavior by returning `FALSE` with `ERROR_INVALID_HANDLE` for null and bad DC handles, while valid calls continue to update brush origin and return the previous origin when requested.
- `src/ce/coredll.rs` and `src/ce/resource.rs`: raw `SaveDC`/`RestoreDC` now follow CE GDIAPI DC-stack behavior for invalid HDCs (`ERROR_INVALID_HANDLE`), `RestoreDC(hdc, 0)` (`ERROR_INVALID_PARAMETER`), `RestoreDC(hdc, -1)` restoring only the top saved level, and absolute positive restore levels removing newer saved states.
- `src/emulator/unicorn.rs`: full-feature Unicorn tests now compile with the newer blocked-GetMessage `import_pc` and pending-WndProc-return plumbing, while synthetic zero-`import_pc` GetMessage resumes still write queued sent messages directly into the guest MSG buffer.
- `src/emulator/unicorn.rs`: escaped `GetMessageW` sent-message WndProc callouts now complete the active send and restore the saved import registers, current thread, and running-thread state before the original `GetMessageW` import trap is processed.
- `src/ce/coredll.rs`: `MessageBoxW` now validates the CE `winuser.h` style surface, accepting CE high flags such as `MB_SETFOREGROUND`, `MB_TOPMOST`, and `MB_RTLREADING` while rejecting unsupported desktop-only bits and undefined icon nibbles before recording dialog state.
- `src/ce/coredll.rs`: `TrackPopupMenuEx` now applies CE horizontal/vertical `TPM_*ALIGN` flags before recording tracking state, painting, hit-testing, and any `TPMPARAMS.rcExclude` adjustment.
- `src/ce/coredll.rs`: `SHGetFileInfoW` now writes CE shell `SFGAO_*` attributes for `SHGFI_ATTRIBUTES` instead of raw `FILE_ATTRIBUTE_*` values, covering filesystem, folder, shortcut, and read-only outputs.
- `src/ce/coredll.rs`: `SHGetFileInfoW` now follows CE `api.cpp` return semantics by returning the system image-list handle for `SHGFI_ICON` queries as well as `SHGFI_SYSICONINDEX`, while still writing the extracted/synthetic `hIcon` into `SHFILEINFO`.
- `src/ce/coredll.rs`: `Shell_NotifyIconW` now follows the CE fixed `NOTIFYICONDATAW` contract from `shellapi.h`/`minserver.cpp` by rejecting short `cbSize` values and unreadable `szTip[64]` buffers before updating shell state.
- `src/ce/kernel.rs`: file-change record append now coalesces pending records and signals only when pending notification data remains.
- `src/ce/kernel.rs`: CE file-notification detail records are only queued for watches created with `FILE_NOTIFY_CHANGE_CEGETINFO`; watches without that flag still signal on matching changes and report no detailed records to `CeGetFileNotificationInfo`.
- `src/ce/kernel.rs` and `src/ce/coredll.rs`: `FindFirstChangeNotificationW` now preserves unknown notification filter bits like CE `pathapi.cpp`/`volumeapi.cpp` passing `NotifyFilter` into `fsnotify.cpp::NotifyCreateEvent`, instead of rejecting those bits before watch creation.
- `src/ce/coredll.rs`: `CeGetFileNotificationInfo` now matches CE `NotifyReset` record byte sizing by including the copied trailing NUL WCHAR and DWORD padding in fit checks, returned bytes, and available bytes while leaving `FileNameLength` as the non-NUL byte count.
- `src/ce/coredll.rs`: `CeGetFileNotificationInfo` now follows CE `fsnotify.cpp::NotifyReset` for null output buffers with enough length to fit the first record: the call reaches the guarded guest write path, fails with `ERROR_INVALID_PARAMETER`, and leaves pending notification records intact instead of reporting `ERROR_INSUFFICIENT_BUFFER`.
- `src/ce/coredll.rs`: `CeGetFileNotificationInfo` now follows CE `fsnotify.cpp::NotifyReset` for no-pending data fetches by writing `lpBytesReturned` before `lpBytesAvailable` inside guarded output writes and reporting `ERROR_NO_MORE_ITEMS` even when a nonzero output pointer is invalid.
- `src/ce/gwe.rs` and `src/ce/coredll.rs`: destroyed-window handling exposes completed send-message result writes and flushes them to guest memory.
- `tests/coredll_raw_kernel.rs`: icon extraction, PE group-icon count, multi-slot PE `ExtractIconExW` output, multi-size large/small PE icon selection, string-named `RT_GROUP_ICON` extraction, sparse integer `RT_GROUP_ICON` ID extraction, 4bpp and 8bpp indexed PE icon extraction, missing-AND-mask color-only PE icon extraction, malformed PE group/icon failure, missing primary and secondary `RT_ICON` ordinal failure, shell icon, and image-list drawing coverage is present.
- `tests/coredll_raw_kernel.rs`: `LoadLibraryExW(LOAD_LIBRARY_AS_DATAFILE | DONT_RESOLVE_DLL_REFERENCES)` now covers already-loaded module reuse plus blocked raw `GetProcAddressA/W` name and ordinal lookups for datafile-flagged modules with real exports.
- `tests/coredll_raw_kernel.rs`: `ImageList_Create` now covers rejection of non-CE creation flags and acceptance/preservation of CE-valid private/shared flags.
- `tests/basic_subsystems.rs` and `tests/coredll_raw_kernel.rs`: image-list creation coverage now verifies CE's default `ILC_COLOR` to `ILC_COLORDDB` flag normalization for direct resource-system and raw ordinal paths.
- `tests/basic_subsystems.rs` and `tests/coredll_raw_kernel.rs`: image-list add coverage now verifies CE rejects bitmaps narrower than the image-list slot width for both unmasked and masked add paths without changing the image count.
- `tests/basic_subsystems.rs` and `tests/coredll_raw_kernel.rs`: image-list icon replacement coverage now verifies CE rejects `ImageList_ReplaceIcon` indexes below `-1` while preserving `-1` append behavior.
- `tests/coredll_raw_kernel.rs`: raw image-list size coverage now verifies null `cy` rejects `ImageList_GetIconSize` before `cx` is written.
- `tests/coredll_raw_kernel.rs`: `ImageList_LoadImage` coverage now verifies CE `cx == 0` behavior by splitting a 2x1 bitmap strip into two 1x1 image-list entries and verifies negative `cx` fails with `ERROR_INVALID_PARAMETER`.
- `tests/coredll_raw_kernel.rs`: `ImageList_LoadImage` coverage now verifies masked 24bpp loads create `ILC_MASK | ILC_COLOR24` image lists and accept `ImageList_SetOverlayImage`.
- `tests/coredll_raw_kernel.rs`: `ImageList_LoadImage` coverage now verifies `CLR_NONE` loads create unmasked entries with mask handle `0` and no transparent color.
- `tests/basic_subsystems.rs` and `tests/coredll_raw_kernel.rs`: image-list merge coverage now verifies CE union-rectangle sizing and combined mask/color flags for mixed-size and offset source lists.
- `tests/coredll_raw_kernel.rs`: `ImageList_Copy` now covers invalid copy flags, CE-unsupported cross-list copies, same-list `ILCF_SWAP`, and same-list copy behavior without source removal.
- `tests/coredll_raw_kernel.rs`: `ImageList_DrawIndirect` now covers exact CE `cbSize == sizeof(IMAGELISTDRAWPARAMS)` validation, including no draw-state mutation for short or oversized records.
- `tests/coredll_raw_kernel.rs`: `ImageList_DrawIndirect` now covers CE zero-`cx`/`cy` defaulting after `xBitmap` and `rgbBk == CLR_DEFAULT` resolution through the image-list background color.
- `tests/coredll_raw_kernel.rs`: `ImageList_DrawIndirect` now verifies CE-visible guest-struct mutation for defaulted `cx`/`cy`, forced `ILD_TRANSPARENT`, and resolved `rgbBk`.
- `tests/coredll_raw_kernel.rs`: `ImageList_DrawIndirect` now verifies CE-visible overlay-pass guest-struct mutation for mask-bound overlays, including stripped overlay-mask flags and `ILD_TRANSPARENT | ILD_IMAGE` final style.
- `tests/coredll_raw_kernel.rs`: raw `ImageList_Draw` now verifies CE's wrapper background default by filling a masked pixel from the image-list background color while preserving explicit `ILD_TRANSPARENT` behavior.
- `tests/coredll_raw_kernel.rs`: `ImageList_DrawIndirect` now covers CE `rgbFg == CLR_NONE` destination blending against an existing selected-memory-DIB pixel.
- `tests/coredll_raw_kernel.rs`: `ImageList_DrawIndirect` now covers CE's separate mask-only `rgbFg == CLR_NONE` blend path, including guest-visible `ILD_TRANSPARENT` mutation and the dither/SRCAND selected-memory-DIB result.
- `tests/coredll_raw_kernel.rs`: `ImageList_DrawIndirect` now covers CE `ILD_BLEND75` recognition through the private `ILD_BLENDMASK` path for bitmap-backed selected-memory-DIB drawing.
- `tests/coredll_raw_kernel.rs`: `ImageList_CopyDitherImage` now covers CE's 50% mono-dither mask update by copying from a black source mask into a white destination mask and verifying the resulting mask draw.
- `src/emulator/unicorn.rs` unit coverage now verifies escaped `GetMessageW` sent-message WndProc completion clears the active send, preserves the completed result, and restores current/running-thread state.
- `tests/basic_subsystems.rs` and `tests/coredll_raw_kernel.rs`: image-list remove coverage now verifies CE rejects negative indexes below `-1` without clearing the list, while preserving `-1` remove-all behavior.
- `tests/basic_subsystems.rs`: direct image-list coverage now verifies CE overlay slot retention after single-image removal and count truncation, plus overlay clearing for `ImageList_Remove(-1)`.
- `tests/basic_subsystems.rs` and `tests/coredll_raw_kernel.rs`: image-list count coverage now verifies failed `ImageList_SetImageCount` allocation requests preserve the current count and surface a guest-visible raw failure.
- `tests/basic_subsystems.rs` and `tests/coredll_raw_kernel.rs`: image-list overlay coverage now verifies unmasked lists and slot `5` reject `ImageList_SetOverlayImage`, while `ILC_MASK` lists still record and draw valid overlays.
- `tests/coredll_raw_kernel.rs`: image-list overlay coverage now verifies CE mask-bound metadata for a right-edge rectangular overlay and confirms the stored `ILD_IMAGE` flag.
- `tests/coredll_raw_kernel.rs`: image-list overlay coverage now verifies `ILD_MASK | ILD_OVERLAYMASK` still draws the registered overlay mask into framebuffer output.
- `tests/basic_subsystems.rs` and `tests/coredll_raw_kernel.rs`: image-list size coverage now verifies CE `ImageList_SetIconSize` no-op failure and image-count clearing after a real size change.
- `tests/basic_subsystems.rs` and `tests/coredll_raw_kernel.rs`: image-list drag coverage now verifies CE `ImageList_DragMove` succeeds even before `ImageList_BeginDrag` has created an active drag image.
- `tests/basic_subsystems.rs` and `tests/coredll_raw_kernel.rs`: image-list drag coverage now verifies hidden active `ImageList_DragMove` calls succeed without advancing the stored drag point.
- `tests/basic_subsystems.rs` and `tests/coredll_raw_kernel.rs`: image-list drag coverage now verifies CE `ImageList_SetDragCursorImage` succeeds as a no-op when no active drag image exists.
- `tests/coredll_raw_kernel.rs`: raw image-list drag coverage now verifies CE `ImageList_GetDragImage` returns null plus zeroed default points before `ImageList_BeginDrag`, then returns the no-active point stored by `ImageList_DragEnter` until a matching `ImageList_DragLeave`.
- `tests/test_progs/174_loadlibrary_forwarded_export`: forwarded-export fixture coverage now verifies a named export forwarding to a target DLL ordinal (`a_forward_target.#7`).
- `src/emulator/unicorn.rs` unit coverage now verifies failed runtime loader rollback releases only modules from the failed attempt, `DllMain(DLL_PROCESS_ATTACH)` false-return cleanup sets `ERROR_DLL_INIT_FAILED` while releasing attempt refs, and final `FreeLibrary` dependency release plans detach/release dependency refs without unloading still-referenced modules.
- `tests/basic_subsystems.rs` and `tests/coredll_raw_kernel.rs` now verify CE loaded-module no-resolve promotion and raw `LoadLibraryExW` non-null `hFile` rejection.
- `tests/coredll_raw_kernel.rs` and `src/emulator/unicorn.rs` unit coverage now verify raw `DisableThreadLibraryCalls` persists the module flag and runtime thread lifecycle plans preserve CE load order while skipping disabled and no-import modules.
- `src/emulator/unicorn.rs` unit coverage now verifies CE process-detach refcount-drain ordering, post-lifecycle ref release, no-resolve exclusion, and process-detach inclusion of modules marked with `DisableThreadLibraryCalls`.
- `tests/coredll_raw_kernel.rs` now verifies raw `ProcessDetachAllDLLs` drains normal and `DisableThreadLibraryCalls` dynamic modules while preserving no-resolve/datafile loaded modules.
- `src/emulator/memory.rs` unit coverage now verifies exact unmap removes only a matching mapped region while preserving adjacent mappings.
- `src/ce/resource.rs` unit coverage now verifies module-scoped resource cleanup removes only that module's resource entries and strings.
- `tests/coredll_raw_gwe.rs`: `DrawIconEx` now verifies scaled framebuffer and selected-memory-DIB output from a 2x2 bitmap-backed icon into a 4x4 destination rectangle, verifies `DI_MASK` draws/scales the mask bitmap rather than the color bitmap, and covers 1bpp mask-only icon draws into framebuffers and selected-memory DIBs.
- `tests/coredll_raw_kernel.rs`: `MessageBoxW` now verifies CE-supported high style bits are preserved and unsupported style/icon bits fail without creating a new shell message-box record.
- `tests/coredll_raw_gwe.rs`: `TrackPopupMenuEx` now verifies that `TPMPARAMS.rcExclude` moves the top-level popup before pointer hit-testing and that CE center/right/bottom alignment flags reposition the popup before pointer selection.
- `tests/basic_subsystems.rs`: direct `ResourceSystem::copy_image_list_image` callers now assert CE same-list-only `ImageList_Copy` behavior instead of the removed cross-list move approximation.
- `tests/coredll_raw_kernel.rs`: `SHGetFileInfo` now verifies CE `SFGAO_*` attribute output for regular files, shortcuts, read-only files, storage-card folders, and inaccessible network folders.
- `tests/coredll_raw_kernel.rs`: `SHGetFileInfo` system-image-list coverage now verifies an icon-only request returns the system image-list handle and separately populates `hIcon`.
- `tests/coredll_raw_kernel.rs`: `Shell_NotifyIcon` duplicate-add rejection, `NIF_*` member flag handling, fixed CE `NOTIFYICONDATAW` size/readability, null-icon modify preservation, registered-taskbar `WM_USER+0xBAD` posts with copied `NOTIFYICONDATAW` payloads, and dispatch-time payload release are covered.
- `tests/basic_subsystems.rs`: direct `ShellSystem` coverage now verifies taskbar HWND register/clear state.
- `tests/coredll_raw_kernel.rs`: `SHNotificationAddI` sink-window validation, `SHNotificationUpdateI` stale-sink update behavior, `SHNotificationRemoveI` pending-callback purge, sink-window destruction callback cleanup, and `SHNotificationGetDataI` fixed-title-buffer output with `cbTitle == 0` are covered.
- `tests/coredll_raw_kernel.rs`: `SHNotificationUpdateI` now also verifies non-null `SHNUM_ICON` replacement records the previous notification icon for destruction, while null-icon updates preserve the existing icon and do not add another destruction record.
- `tests/coredll_raw_kernel.rs`: `SHNotificationAddI`/timeout coverage now uses an iconic notification and verifies timeout dismissal removes the record and records copied icon destruction.
- `tests/coredll_raw_kernel.rs`: the `MessageBoxW` render-state test no longer samples hard-coded black/foreground pixels; it asserts the stable rendered record and dirty-rectangle signal instead.
- `tests/basic_subsystems.rs`: direct `ShellSystem` cleanup now verifies window-state removal clears pending notification callbacks along with notify icons, shell notifications, and change-notification registrations.
- `tests/coredll_raw_memory_file.rs`: transient file-change notification churn coverage is present.
- `tests/coredll_raw_memory_file.rs`: signal-only change notifications without `FILE_NOTIFY_CHANGE_CEGETINFO` are covered separately from detailed `CeGetFileNotificationInfo` drains.
- `tests/coredll_raw_memory_file.rs`: `FindFirstChangeNotificationW` now covers an unrooted watch path containing `.` and `..` components against a mounted `\ResidentFlash` directory and verifies the canonical watch receives detailed create records.
- `tests/coredll_raw_memory_file.rs`: `FindNextChangeNotification` now covers the CE reset case where two pending detailed creates remain signaled after one reset and can still be fetched through `CeGetFileNotificationInfo`.
- `tests/coredll_raw_memory_file.rs`: `MoveFileW` now covers cross-volume file success, cross-volume directory rejection, source mount-point rename denial, and destination mount-point collision errors across mounted `\ResidentFlash` and `\Storage Card` roots.
- `tests/coredll_raw_memory_file.rs`: mounted-root coverage now verifies `CreateDirectoryW` on the root returns `ERROR_ALREADY_EXISTS`, `DeleteFileW` on the root returns `ERROR_FILE_NOT_FOUND`, and `RemoveDirectoryW`/`SetFileAttributesW` return `ERROR_ACCESS_DENIED`. Read-only mounted root coverage verifies raw `CreateFileW`, `CopyFileW`, child `CreateDirectoryW`/`RemoveDirectoryW`, `MoveFileW`, file `SetFileAttributesW`, and `DeleteFileW` fail with `ERROR_ACCESS_DENIED` without mutating the host backing directory, signaling a matching change-notification watch, or returning detailed `CeGetFileNotificationInfo` records.
- `tests/coredll_raw_memory_file.rs`: root file-change notification coverage now verifies a nested create under `\ResidentFlash\watch` does not signal a non-recursive `\` watch, does signal a recursive `\` watch, and returns the expected root-relative detailed record.
- `tests/coredll_raw_memory_file.rs`: non-root file-change notification coverage now verifies nested creates and cross-parent moves under `\ResidentFlash\watch` are ignored by non-recursive watches, reported by recursive watches, and returned as CE remove/add move records.
- `tests/coredll_raw_memory_file.rs`: directory self-watch coverage now verifies same-parent rename and removal of the watched directory report CE current-directory removal as `(FILE_ACTION_REMOVED, "\\")` instead of an empty relative name or rename-old action.
- `src/emulator/imports.rs`: import-table tests now verify malformed forwarded-export strings fail closed, including whitespace-padded module/symbol halves and missing ordinal digits.
- `tests/coredll_raw_memory_file.rs`: `FindCloseChangeNotification` coverage now verifies a wrong-type file handle fails with `ERROR_INVALID_HANDLE` and is no longer closeable afterward, preserving the CE caller-handle ownership side effect.
- `tests/coredll_raw_memory_file.rs`: `DuplicateHandle` coverage now verifies `DUPLICATE_CLOSE_SOURCE` invalidates the source change-notification handle while preserving an independent duplicate that still receives and drains detailed file-change records.
- `src/ce/kernel.rs`, `src/ce/coredll.rs`, and `tests/coredll_raw_memory_file.rs`: process-owned notification coverage now verifies a different current process cannot wait on, reset, query, duplicate, directly `CloseHandle`, or find-close a file-change notification handle created by another process, and that the failed direct close leaves the handle available for the owner to close afterward.
- `tests/coredll_raw_memory_file.rs`: direct `AFS_FindFirstChangeNotificationW` coverage now verifies a nonzero `hProc` creates a notification handle owned by that target process rather than by the raw ordinal caller.
- `tests/coredll_raw_memory_file.rs`: `CeGetFileNotificationInfo` coverage now verifies a two-character filename record requires CE's 20-byte NUL-padded record size instead of the old 16-byte non-NUL-only size, and confirms the copied NUL/padding bytes while preserving `FileNameLength`.
- `tests/coredll_raw_memory_file.rs`: `CeGetFileNotificationInfo` coverage now verifies the no-pending CE output-pointer fault order: a bad `lpBytesReturned` leaves `lpBytesAvailable` untouched, while a bad `lpBytesAvailable` still leaves `lpBytesReturned` zeroed and returns `ERROR_NO_MORE_ITEMS`.
- `tests/coredll_raw_memory_file.rs`: file-change notification filter coverage now verifies CE-style unknown filter preservation: unknown-only watches stay inert, while unknown bits combined with `FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO` still signal and return detailed records.
- `tests/coredll_raw_kernel.rs`: `SHNotificationAddI` coverage now asserts the stored notification and queued COM callback record do not inherit the marshalled `cbData` argument as a callback pointer.
- `tests/basic_subsystems.rs`: direct shell notification callback queue coverage now verifies a failed non-null runtime COM dispatch can restore its callback record ahead of newer pending callbacks.
- `tests/coredll_raw_kernel.rs`: `SHNotificationAddI` title/HTML coverage now verifies the CE pointer-presence rule for `SHNP_INFORM`: title-only with null HTML fails, while non-null empty HTML succeeds and receives the default inform duration.
- `tests/coredll_raw_kernel.rs`: `SHNotificationUpdateI` coverage now asserts `SHNUM_PRIORITY` moves a notification from the inform list to the iconic list and `SHNotificationRemoveI` clears the priority-list entry.
- `src/ce/coredll.rs`: raw `ImageList_Add`, `ImageList_AddMasked`, and `ImageList_Replace` now snapshot real source and mask bitmap pixels into owned bitmap storage like CE `imagelist.cpp` image/mask DCs, while preserving metadata-only pseudo handles.
- `src/ce/resource.rs` and `tests/coredll_raw_kernel.rs`: `ImageList_SetIconSize` now follows CE `imagelist.cpp` by accepting changed zero/negative dimensions, storing them, and clearing images/overlays while still failing unchanged no-op size requests.
- `src/ce/resource.rs`, `src/ce/coredll.rs`, `tests/basic_subsystems.rs`, and `tests/coredll_raw_kernel.rs`: `ImageList_SetOverlayImage` now follows CE `imagelist.cpp` by treating the same overlay slot/image pair as a successful no-op that preserves the previously computed overlay bounds.
- `tests/coredll_raw_kernel.rs`: image-list lifetime coverage now verifies deleting the caller-owned source bitmap after `ImageList_Add` does not invalidate later bitmap-backed image-list draws.
- `tests/coredll_raw_kernel.rs`: `ImageList_Duplicate` coverage now verifies bitmap-backed image and mask entries receive distinct duplicated handles and backing memory, matching CE `CopyDIBBitmap`/`CopyBitmap` ownership instead of aliasing the source list.
- `tests/coredll_raw_gwe.rs`: destroyed-target `SendMessageTimeout` result write coverage is present.
- `tests/coredll_raw_gwe.rs`: `AlphaBlend` coverage now verifies CE premultiplied `AC_SRC_ALPHA` source pixels separately from `AC_SRC_ALPHA_NONPREMULT` non-premultiplied source pixels, while preserving selected-DIB and framebuffer top-down/bottom-up per-pixel alpha coverage.
- `tests/coredll_raw_gwe.rs`: same-thread `SendMessageTimeout` coverage now verifies synchronous dispatch, direct result-pointer writes, clean last-error state, and no cross-thread sent-message transaction.
- `tests/coredll_raw_gwe.rs`: CE-public `SMTO_NORMAL` cross-thread timeout coverage now verifies nonzero timeout queueing, timeout metadata preservation, dispatch completion, and result-pointer writes.
- `tests/coredll_raw_gwe.rs`: cross-thread `SendMessageTimeout` coverage now verifies an early receiver `ReplyMessage` result wins over the later wndproc return and writes through the timeout result pointer.
- `tests/coredll_raw_gwe.rs`: nested `SendMessageTimeout` coverage now verifies an inner `ReplyMessage` writes the inner result without clearing or overwriting the still-active outer send.
- `tests/coredll_raw_gwe.rs`: nested timeout lifetime coverage now verifies an
  active outer `SendMessageTimeout` can time out while the receiver is still in
  that send, a nested sent message can still dispatch and complete
  independently, and the outer dispatch unwind preserves the timeout result
  instead of overwriting it with the later wndproc return.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_send_message_timeout_active_outer_timeout_preserves_nested_send`
  passed after adding the CE `MessageTimeout::cReference`-style nested timeout
  lifetime regression.
- `src/ce/coredll.rs`: Unicorn `SendMessageTimeoutW` block preparation now declines zero-timeout cross-thread sends, leaving them to the raw CE expiry path instead of registering a blocked wait that could leave a stale receiver send queued.
- `src/ce/coredll.rs`: unit coverage now verifies zero-timeout block preparation returns `None` without creating a queued sent-message record.
- `src/ce/gwe.rs`, `src/ce/kernel.rs`, and `src/emulator/unicorn.rs`:
  timed-out sent-message transactions can now be marked complete by id, removed
  from receiver queues, and consumed by the Unicorn blocked
  `SendMessageTimeout` timeout path before it returns `ERROR_TIMEOUT`, avoiding
  stale receiver delivery after the sender resumes.
- `tests/coredll_raw_gwe.rs` and `src/ce/gwe.rs`: nonzero
  `SendMessageTimeout` expiry coverage now verifies the queued send is removed
  before receiver retrieval and the completed zero result remains available to
  the sender-side transaction.
- `tests/coredll_raw_gwe.rs`: `MsgWaitForMultipleObjectsEx` coverage now verifies `QS_SENDMESSAGE`, CE `QS_ALLINPUT` over posted messages, paint, and timers, `MWMO_INPUTAVAILABLE` new-vs-existing input behavior, and signaled handle precedence over queued sent-message input.
- `src/ce/coredll.rs`: raw `MsgWaitForMultipleObjectsEx` now uses the no-record multiple-wait helper for its internal handle readiness probe, so a CE message wait with handles records one message-wait attempt instead of a hidden `WaitForMultipleObjects` attempt plus the public message wait.
- `tests/coredll_raw_gwe.rs`: mixed handle `MsgWaitForMultipleObjectsEx` coverage now verifies scheduler telemetry records only the CE message-wait attempt while preserving signaled-handle return precedence.
- `src/winsock.rs`: `select` now ignores `nfds` like CE callers expect while validating non-null fd sets, `FD_SETSIZE`, invalid socket handles, and fd-set memory faults before filtering readiness.
- `src/winsock.rs`: Winsock unit coverage now exercises `select` with `nfds` values `0`, `-1`, and active counts, mixed read/write/except fd sets, null fd-set triads, oversized fd sets, invalid socket handles, and `WSAEFAULT` memory failures.
- `src/winsock.rs`: TCP peer close is now read-ready for `select`, allowing the follow-up `recv` to return zero; repeated zero-ready `select` polling is covered by a recovery test that becomes readable after a later datagram.
- `src/winsock.rs`: UDP `recvfrom` coverage now verifies host loopback datagram sources are exposed to CE callers as the isolated gateway address with the original sender port.
- `src/winsock.rs`: TCP half-close coverage now verifies peer write shutdown wakes `select` for a zero-length `recv` while the guest socket can still `send` on its write half.
- `src/winsock.rs`: TCP reset coverage now treats reset sockets as read-ready, caches `WSAECONNRESET` for `SO_ERROR`, and verifies `recv` reports the reset after a host `SO_LINGER(0)` close.
- `src/winsock.rs`: Listener coverage now runs repeated `select`/`accept` cycles, verifies re-arming after each accepted client, and checks accepted loopback peer addresses are exposed as CE gateway addresses.
- `src/winsock.rs`: host TCP send/recv unit coverage now waits for socket read readiness before asserting the follow-up `recv`, avoiding the emulator's intentional short host read-timeout race during full parallel test runs.
- `tests/basic_subsystems.rs`: shell notify-icon and file-notification expectations now use explicit CE member/detail flags after the flag-gated notify and `FILE_NOTIFY_CHANGE_CEGETINFO` behavior changes.

## Last Known Validation

- `cargo fmt --check` passed after aligning `SHNN_SHOW` with the CE taskbar
  window-only notification path; PowerShell emitted the existing non-fatal
  PSReadLine profile warning.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel shnotification_i_tracks_query_update_and_remove_state`
  passed after the `SHNN_SHOW` callback-routing correction.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel`
  passed after the `SHNN_SHOW` callback-routing correction.
- `cargo check --features unicorn,trace,win32-desktop` passed with
  `CARGO_INCREMENTAL=0` after the `SHNN_SHOW` callback-routing correction.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  `SHNN_SHOW` callback-routing correction; the eVC4 MIPSII fixture test remains
  ignored because the toolchain is not configured.
- `git diff --check` passed after the `SHNN_SHOW` callback-routing correction;
  output was limited to existing CRLF normalization warnings.
- `cargo fmt --check` passed after adding Unicorn-level unmapped
  `IShellNotificationCallback` pointer retry coverage; PowerShell emitted the
  existing non-fatal PSReadLine profile warning.
- `cargo test -j 1 --features unicorn,trace,win32-desktop shell_notification_com_dispatch_restores_unmapped_callback_pointer`
  passed after adding the runtime unmapped callback-interface pointer
  regression.
- `cargo check --features unicorn,trace,win32-desktop` passed with
  `CARGO_INCREMENTAL=0` after adding the runtime unmapped callback-interface
  pointer regression.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding
  the runtime unmapped callback-interface pointer regression; the eVC4 MIPSII
  fixture test remains ignored because the toolchain is not configured.
- `git diff --check` passed after adding the runtime unmapped
  callback-interface pointer regression; output was limited to existing CRLF
  normalization warnings.
- `cargo fmt --check` passed after preserving non-null pending `IShellNotificationCallback` records on transient runtime vtable dispatch failure; PowerShell emitted the existing non-fatal PSReadLine profile warning.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test basic_subsystems shell_notification_callback_queue_restores_failed_runtime_dispatch_to_front` passed after adding direct shell callback queue restoration coverage.
- `cargo check --features unicorn,trace,win32-desktop` passed with `CARGO_INCREMENTAL=0` after preserving non-null pending shell notification COM callbacks on failed runtime dispatch.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after preserving non-null pending shell notification COM callbacks on failed runtime dispatch; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `git diff --check` passed after preserving non-null pending shell notification COM callbacks on failed runtime dispatch; output was limited to existing CRLF normalization warnings.
- `cargo fmt --check` passed after aligning bitmap-backed `ImageList_Draw*` mask-only `rgbFg == CLR_NONE` blend behavior with CE; PowerShell emitted the existing non-fatal PSReadLine profile warning.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_draw_indirect_applies_x_bitmap_offset_and_rgb_bk_fill` passed after adding the CE mask-only blend regression.
- `cargo check --features unicorn,trace,win32-desktop` passed with `CARGO_INCREMENTAL=0` after adding the CE mask-only image-list blend path.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding the CE mask-only image-list blend path; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `git diff --check` passed after adding the CE mask-only image-list blend path; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after adding CE `DisableThreadLibraryCalls` state and thread attach/detach lifecycle callout planning; PowerShell emitted the existing non-fatal PSReadLine profile warning.
- `cargo test -j 1 --features unicorn,trace,win32-desktop dll_process_detach_plan_drains_refcounts_in_ce_order` passed through the x64 VS developer environment after adding CE process-detach refcount-drain planning.
- `cargo check --features unicorn,trace,win32-desktop` passed through the x64 VS developer environment after adding CE process-detach refcount-drain planning.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed through the x64 VS developer environment after adding CE process-detach refcount-drain planning; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop coredll_raw_process_detach_all_dlls_drains_imported_module_refs` passed through the x64 VS developer environment after adding raw `ProcessDetachAllDLLs` module-state draining.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel` passed through the x64 VS developer environment after adding raw `ProcessDetachAllDLLs` module-state draining.
- `cargo check --features unicorn,trace,win32-desktop` passed through the x64 VS developer environment after adding raw `ProcessDetachAllDLLs` module-state draining.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed through the x64 VS developer environment after adding raw `ProcessDetachAllDLLs` module-state draining; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop coredll_raw_disable_thread_library_calls_validates_module_handles` passed through the x64 VS developer environment after verifying the raw no-thread-calls module flag.
- `cargo test -j 1 --features unicorn,trace,win32-desktop dll_thread_lifecycle_calls_follow_load_order_and_skip_disabled_noimport_modules` passed through the x64 VS developer environment after adding runtime thread-notification planning coverage.
- `cargo check --features unicorn,trace,win32-desktop` passed through the x64 VS developer environment after the loader thread-notification changes.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed through the x64 VS developer environment after the loader thread-notification changes; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo fmt` passed after aligning file-change move records with CE `NotifyMoveFileEx`; PowerShell emitted the existing non-fatal PSReadLine profile warning.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_nonroot_change_notification_honors_subtree_and_move_boundaries` passed with `CARGO_INCREMENTAL=0` after adding non-root subtree and cross-parent move notification coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_change_notification_handles_signal_and_rearm` passed with `CARGO_INCREMENTAL=0` after updating cross-parent move expectations from rename old/new to CE remove/add records.
- `cargo test --features unicorn,trace,win32-desktop` passed with `CARGO_INCREMENTAL=0` after aligning file-change move records with CE `NotifyMoveFileEx`; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo fmt` passed after tightening malformed forwarded-export parsing; PowerShell emitted the existing non-fatal PSReadLine profile warning.
- `cargo test --features unicorn,trace,win32-desktop emulator::imports::tests` passed with `CARGO_INCREMENTAL=0` after adding malformed forwarded-export parser/resolver coverage.
- `cargo check --features unicorn,trace,win32-desktop` passed with `CARGO_INCREMENTAL=0` after tightening malformed forwarded-export parsing.
- `cargo test --features unicorn,trace,win32-desktop` passed with `CARGO_INCREMENTAL=0` after tightening malformed forwarded-export parsing; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.

- `cargo fmt` passed after aligning raw `ImageList_CopyDitherImage` with CE destination-mutation semantics; PowerShell emitted the existing non-fatal PSReadLine profile warning.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed with `CARGO_INCREMENTAL=0` after adding raw bitmap-backed `ImageList_CopyDitherImage` metadata-preservation and pixel-copy coverage.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems image_list` passed with `CARGO_INCREMENTAL=0` after updating direct image-list dither-copy expectations.
- `cargo test --features unicorn,trace,win32-desktop` passed with `CARGO_INCREMENTAL=0` after aligning raw `ImageList_CopyDitherImage` with CE destination-mutation semantics; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.

- `cargo fmt` passed after adding CE `ImageList_DrawIndirect` `ILD_ROP`/`dwRop` raster-operation handling; PowerShell emitted the existing non-fatal PSReadLine profile warning.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_draw_indirect_applies_x_bitmap_offset_and_rgb_bk_fill` passed with `CARGO_INCREMENTAL=0` after adding `ILD_IMAGE | ILD_ROP` `SRCINVERT` selected-DIB coverage.
- `cargo test --features unicorn,trace,win32-desktop` passed with `CARGO_INCREMENTAL=0` after adding CE `ImageList_DrawIndirect` `ILD_ROP`/`dwRop` raster-operation handling; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.

- `cargo fmt` passed after adding CE `ImageList_DrawIndirect` overlay-pass `IMAGELISTDRAWPARAMS` mutation; PowerShell emitted the existing non-fatal PSReadLine profile warning.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_draw_indirect_applies_x_bitmap_offset_and_rgb_bk_fill` passed with `CARGO_INCREMENTAL=0` after adding mask-bound overlay-pass struct-mutation coverage.
- `cargo check --features unicorn,trace,win32-desktop` passed with `CARGO_INCREMENTAL=0` after adding CE `ImageList_DrawIndirect` overlay-pass `IMAGELISTDRAWPARAMS` mutation.
- `cargo test --features unicorn,trace,win32-desktop` passed with `CARGO_INCREMENTAL=0` after adding CE `ImageList_DrawIndirect` overlay-pass `IMAGELISTDRAWPARAMS` mutation; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `git diff --check` passed after adding CE `ImageList_DrawIndirect` overlay-pass `IMAGELISTDRAWPARAMS` mutation; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after aligning raw `ImageList_Draw` with CE's `CLR_DEFAULT` wrapper background default and repairing escaped `GetMessageW` sent-message callout completion; PowerShell emitted the existing non-fatal PSReadLine profile warning.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after adding raw `ImageList_Draw` default-background coverage.
- `cargo check --features unicorn,trace,win32-desktop` passed with `CARGO_INCREMENTAL=0` after the raw `ImageList_Draw` CE default and escaped `GetMessageW` callout repair.
- `cargo test --features unicorn,trace,win32-desktop` passed with `CARGO_INCREMENTAL=0` after the raw `ImageList_Draw` CE default and escaped `GetMessageW` callout repair; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `git diff --check` passed after the raw `ImageList_Draw` CE default and escaped `GetMessageW` callout repair; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after adding CE `ImageList_DrawIndirect` in-place `IMAGELISTDRAWPARAMS` normalization; PowerShell emitted the existing non-fatal PSReadLine profile warning.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_draw_indirect_applies_x_bitmap_offset_and_rgb_bk_fill` passed after verifying CE-visible `cx`/`cy`, `rgbBk`, and `fStyle` guest-struct mutation. The first sandboxed attempt failed before tests because CMake could not run Ninja for `unicorn-engine-sys`; rerunning outside the sandbox passed.
- `cargo check --features unicorn,trace,win32-desktop` passed after adding CE `ImageList_DrawIndirect` in-place `IMAGELISTDRAWPARAMS` normalization.
- `git diff --check` passed after adding CE `ImageList_DrawIndirect` in-place `IMAGELISTDRAWPARAMS` normalization; output was limited to existing CRLF normalization warnings.
- `cargo test --features unicorn,trace,win32-desktop` passed after adding CE `ImageList_DrawIndirect` in-place `IMAGELISTDRAWPARAMS` normalization; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo fmt` passed after the CE `ImageList_SetImageCount` allocation-failure slice and parked-child GetMessage sentinel repair; PowerShell emitted the existing non-fatal PSReadLine profile warning.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems resource_system_image_list_duplicate_replace_remove_copy_count_overlay_and_drag` passed after adding direct `ImageList_SetImageCount` failed-allocation count-preservation coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after adding raw `ImageList_SetImageCount` failed-allocation coverage.
- `cargo test --features unicorn,trace,win32-desktop emulator::unicorn::guest_thread_stack_tests::parent_exits_and_child_window_pump_stays_alive --lib` passed after making parked-child GetMessage callout preparation honor synthetic zero-`import_pc` blocked states.
- `cargo test --features unicorn,trace,win32-desktop` passed after the CE `ImageList_SetImageCount` allocation-failure slice and parked-child GetMessage sentinel repair; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo check --features unicorn,trace,win32-desktop` passed after the CE `ImageList_SetImageCount` allocation-failure slice and parked-child GetMessage sentinel repair.
- `git diff --check` passed after the CE `ImageList_SetImageCount` allocation-failure slice and parked-child GetMessage sentinel repair; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after the CE `ImageList_Merge` metadata slice and Winsock host TCP test stabilization; PowerShell emitted the existing non-fatal PSReadLine profile warning.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems resource_system_owned_bitmap_bitmap_mut_region_rects_palette_mut_shell_image_list_merge_and_dither` passed after adding direct `ImageList_Merge` union sizing coverage.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems resource_image_list_duplicate_merge_add_masked_replace_remove_copy_overlay_drag` passed after adding mixed-size/negative-offset `ImageList_Merge` flag and size coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after aligning raw `ImageList_Merge` metadata expectations with CE.
- `cargo test --features unicorn,trace,win32-desktop winsock::tests::host_tcp_socket_connect_send_and_recv_use_loopback -- --nocapture` passed after waiting for read readiness before `recv`.
- `cargo check --features unicorn,trace,win32-desktop` passed after the CE `ImageList_Merge` metadata slice and Winsock test stabilization.
- `cargo test --features unicorn,trace,win32-desktop` passed after the CE `ImageList_Merge` metadata slice and Winsock test stabilization; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `git diff --check` passed after the CE `ImageList_Merge` metadata slice and Winsock test stabilization; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after aligning `ImageList_Remove(i < -1)` rejection with CE and repairing full-feature Unicorn test plumbing; PowerShell emitted the existing non-fatal PSReadLine profile warning.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems resource_system_image_list_duplicate_replace_remove_copy_count_overlay_and_drag` passed after adding direct `ImageList_Remove(i < -1)` coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after adding raw `ImageList_Remove(i < -1)` coverage.
- `cargo test --features unicorn,trace,win32-desktop` passed after aligning `ImageList_Remove(i < -1)` rejection with CE and repairing full-feature Unicorn test plumbing; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo check --features unicorn,trace,win32-desktop` passed after aligning `ImageList_Remove(i < -1)` rejection with CE and repairing full-feature Unicorn test plumbing.
- `git diff --check` passed after aligning `ImageList_Remove(i < -1)` rejection with CE and repairing full-feature Unicorn test plumbing; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after aligning `ImageList_GetIconSize` null-output validation with CE; PowerShell emitted the existing non-fatal PSReadLine profile warning.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after adding raw `ImageList_GetIconSize` null-output no-partial-write coverage.
- `cargo check --features unicorn,trace,win32-desktop` passed after aligning `ImageList_GetIconSize` null-output validation with CE.
- `cargo test --features unicorn,trace,win32-desktop` passed after aligning `ImageList_GetIconSize` null-output validation with CE; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `git diff --check` passed after aligning `ImageList_GetIconSize` null-output validation with CE; output was limited to existing CRLF normalization warnings.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after adding bitmap-backed `ImageList_GetIcon` real-icon coverage.
- `CARGO_INCREMENTAL=0 cargo check --features unicorn,trace,win32-desktop` passed after implementing bitmap-backed `ImageList_GetIcon` real-icon creation; warnings remain the known unused-variable/dead-code set.
- `CARGO_INCREMENTAL=0 cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel` passed after implementing bitmap-backed `ImageList_GetIcon` real-icon creation.
- `CARGO_INCREMENTAL=0 cargo test -j 1 --features unicorn,trace,win32-desktop` passed after implementing bitmap-backed `ImageList_GetIcon` real-icon creation; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo fmt --check` passed after implementing bitmap-backed `ImageList_GetIcon` real-icon creation; PowerShell emitted the existing non-fatal PSReadLine profile warning.
- `git diff --check` passed after implementing bitmap-backed `ImageList_GetIcon` real-icon creation; output was limited to existing CRLF normalization warnings.
- `CARGO_INCREMENTAL=0 cargo check --features unicorn,trace,win32-desktop` passed after adding icon-owned bitmap cleanup and filtered sent-message retrieval; warnings remain the known unused-variable/dead-code set.
- `CARGO_INCREMENTAL=0 cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed after filtered sent-message retrieval stopped nonmatching broadcast/notify sends from being returned.
- `CARGO_INCREMENTAL=0 cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel` passed after the mount shell-change notification path no longer consumed `WM_DEVICECHANGE` for a `WM_FILECHANGEINFO` peek.
- `CARGO_INCREMENTAL=0 cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the icon-owned bitmap cleanup and GWE sent-message filter slices; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `CARGO_INCREMENTAL=0 cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel sh_get_file_info_system_image_list_supports_icon_queries_and_draw` passed after clamping synthetic shell image-list overlay masks to CE's four valid overlay slots.
- `CARGO_INCREMENTAL=0 cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel` passed after the synthetic overlay clamp and sent-message filter fix.
- `CARGO_INCREMENTAL=0 cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed after the same GWE sent-message filtering change.
- `CARGO_INCREMENTAL=0 cargo check --features unicorn,trace,win32-desktop` passed after the synthetic overlay clamp and sent-message filter fix; warnings remain the known unused-variable/dead-code set.
- `cargo fmt` passed after aligning `ImageList_ReplaceIcon(i < -1)` rejection with CE; PowerShell emitted the existing non-fatal PSReadLine profile warning.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems resource_system_image_list_duplicate_replace_remove_copy_count_overlay_and_drag` passed after adding direct `ImageList_ReplaceIcon(i < -1)` coverage.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems resource_image_list_duplicate_merge_add_masked_replace_remove_copy_overlay_drag` passed after adding direct `ImageList_ReplaceIcon(i < -1)` coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after adding raw `ImageList_ReplaceIcon(i < -1)` coverage.
- `cargo check --features unicorn,trace,win32-desktop` passed after aligning `ImageList_ReplaceIcon(i < -1)` rejection with CE.
- `cargo test --features unicorn,trace,win32-desktop` passed after aligning `ImageList_ReplaceIcon(i < -1)` rejection with CE; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `git diff --check` passed after aligning `ImageList_ReplaceIcon(i < -1)` rejection with CE; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after aligning `ImageList_Add`/`ImageList_AddMasked` undersized-bitmap rejection with CE; PowerShell emitted the existing non-fatal PSReadLine profile warning.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems resource_system_owned_bitmap_bitmap_mut_region_rects_palette_mut_shell_image_list_merge_and_dither` passed after adding direct undersized-bitmap add coverage.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems resource_image_list_duplicate_merge_add_masked_replace_remove_copy_overlay_drag` passed after adding direct undersized-bitmap add coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after adding raw `ImageList_Add`/`ImageList_AddMasked`/`ImageList_LoadImage` undersized-bitmap coverage.
- `cargo check --features unicorn,trace,win32-desktop` passed after aligning undersized image-list bitmap handling with CE.
- `cargo test --features unicorn,trace,win32-desktop` passed after aligning undersized image-list bitmap handling with CE; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `git diff --check` passed after aligning undersized image-list bitmap handling with CE; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after adding CE `ILC_COLOR` default-to-`ILC_COLORDDB` image-list flag normalization and the ordinal-forwarder fixture update; PowerShell emitted the existing non-fatal PSReadLine profile warning.
- `cargo fmt` passed after aligning raw `ImageList_LoadImage(cx=0)` with CE; PowerShell emitted the existing non-fatal PSReadLine profile warning.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after adding CE `ImageList_LoadImage` zero/negative `cx` coverage.
- `cargo fmt` passed after aligning raw `ImageList_LoadImage` mask/color flag construction with CE; PowerShell emitted the existing non-fatal PSReadLine profile warning.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after adding CE `ImageList_LoadImage` mask/color flag coverage.
- `cargo fmt` passed after aligning raw `ImageList_LoadImage(CLR_NONE)` with CE's unmasked add path; PowerShell emitted the existing non-fatal PSReadLine profile warning.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after adding CE `ImageList_LoadImage(CLR_NONE)` unmasked-entry coverage.
- `cargo test --features unicorn,trace,win32-desktop parse_forwarder_target_handles_name_and_ordinal_forms` passed after fixing stale full-feature Unicorn sleep-test helper call sites.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems resource_system_image_list_create_add_count_info_bk_color_and_destroy` passed after adding CE `ILC_COLOR` default-to-`ILC_COLORDDB` direct image-list coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after adding CE `ILC_COLOR` default-to-`ILC_COLORDDB` raw ordinal coverage.
- `cargo check --features unicorn,trace,win32-desktop` passed after adding CE `ILC_COLOR` default-to-`ILC_COLORDDB` image-list flag normalization and the ordinal-forwarder fixture update.
- `cargo test --features unicorn,trace,win32-desktop --test host_progs` passed after the ordinal-forwarder fixture update.
- `cargo test --features unicorn,trace,win32-desktop --test fixture_exes` passed with the eVC4 MIPSII fixture test ignored because the toolchain is not configured.
- `cargo test --features unicorn,trace,win32-desktop` passed after adding CE `ILC_COLOR` default-to-`ILC_COLORDDB` image-list flag normalization and the ordinal-forwarder fixture update.
- `git diff --check` passed after adding CE `ILC_COLOR` default-to-`ILC_COLORDDB` image-list flag normalization and the ordinal-forwarder fixture update; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after aligning bitmap-backed `ImageList_Draw*` overlay-mask rendering with CE.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after aligning bitmap-backed `ImageList_Draw*` overlay-mask rendering with CE.
- `cargo check --features unicorn,trace,win32-desktop` passed after aligning bitmap-backed `ImageList_Draw*` overlay-mask rendering with CE.
- `cargo test --features unicorn,trace,win32-desktop` passed after aligning bitmap-backed `ImageList_Draw*` overlay-mask rendering with CE.
- `git diff --check` passed after aligning bitmap-backed `ImageList_Draw*` overlay-mask rendering with CE; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after adding CE `rgbFg == CLR_NONE` destination blending to bitmap-backed `ImageList_Draw*`.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_draw_indirect_applies_x_bitmap_offset_and_rgb_bk_fill` passed after adding CE `rgbFg == CLR_NONE` destination blending to bitmap-backed `ImageList_Draw*`.
- `cargo check --features unicorn,trace,win32-desktop` passed after adding CE `rgbFg == CLR_NONE` destination blending to bitmap-backed `ImageList_Draw*`.
- `cargo test --features unicorn,trace,win32-desktop` passed after adding CE `rgbFg == CLR_NONE` destination blending to bitmap-backed `ImageList_Draw*`.
- `git diff --check` passed after adding CE `rgbFg == CLR_NONE` destination blending to bitmap-backed `ImageList_Draw*`; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after aligning bitmap-backed `ImageList_Draw*` `ILD_BLEND75` handling with CE's private blend mask.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_draw_indirect_applies_x_bitmap_offset_and_rgb_bk_fill` passed after aligning bitmap-backed `ImageList_Draw*` `ILD_BLEND75` handling with CE's private blend mask.
- `cargo check --features unicorn,trace,win32-desktop` passed after aligning bitmap-backed `ImageList_Draw*` `ILD_BLEND75` handling with CE's private blend mask.
- `cargo test --features unicorn,trace,win32-desktop` passed after aligning bitmap-backed `ImageList_Draw*` `ILD_BLEND75` handling with CE's private blend mask.
- `git diff --check` passed after aligning bitmap-backed `ImageList_Draw*` `ILD_BLEND75` handling with CE's private blend mask; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after normalizing `ImageList_Draw*` zero-size and `CLR_DEFAULT` draw parameters with CE.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_draw_indirect_applies_x_bitmap_offset_and_rgb_bk_fill` passed after normalizing `ImageList_Draw*` zero-size and `CLR_DEFAULT` draw parameters with CE.
- `cargo check --features unicorn,trace,win32-desktop` passed after normalizing `ImageList_Draw*` zero-size and `CLR_DEFAULT` draw parameters with CE.
- `cargo test --features unicorn,trace,win32-desktop` passed after normalizing `ImageList_Draw*` zero-size and `CLR_DEFAULT` draw parameters with CE.
- `git diff --check` passed after normalizing `ImageList_Draw*` zero-size and `CLR_DEFAULT` draw parameters with CE; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after enforcing CE's four-slot `ImageList_SetOverlayImage` range.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems resource_image_list_duplicate_merge_add_masked_replace_remove_copy_overlay_drag` passed after enforcing CE's four-slot `ImageList_SetOverlayImage` range.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after enforcing CE's four-slot `ImageList_SetOverlayImage` range.
- `cargo check --features unicorn,trace,win32-desktop` passed after enforcing CE's four-slot `ImageList_SetOverlayImage` range.
- `cargo test --features unicorn,trace,win32-desktop` passed after enforcing CE's four-slot `ImageList_SetOverlayImage` range.
- `git diff --check` passed after enforcing CE's four-slot `ImageList_SetOverlayImage` range; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after aligning `ImageList_SetOverlayImage` overlay mask-bound metadata and overlay drawing with CE.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after aligning `ImageList_SetOverlayImage` overlay mask-bound metadata and overlay drawing with CE.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems resource_image_list_duplicate_merge_add_masked_replace_remove_copy_overlay_drag` passed after changing image-list overlay records to CE metadata.
- `cargo check --features unicorn,trace,win32-desktop` passed after aligning `ImageList_SetOverlayImage` overlay mask-bound metadata and overlay drawing with CE.
- `cargo test --features unicorn,trace,win32-desktop` passed after aligning `ImageList_SetOverlayImage` overlay mask-bound metadata and overlay drawing with CE.
- `git diff --check` passed after aligning `ImageList_SetOverlayImage` overlay mask-bound metadata and overlay drawing with CE; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after aligning no-active `ImageList_DragEnter`/`ImageList_DragLeave` static context and visible-only `ImageList_DragMove` behavior with CE.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems resource_image_list_duplicate_merge_add_masked_replace_remove_copy_overlay_drag` passed after aligning no-active `ImageList_DragEnter`/`ImageList_DragLeave` static context and visible-only `ImageList_DragMove` behavior with CE.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after aligning no-active `ImageList_DragEnter`/`ImageList_DragLeave` static context and visible-only `ImageList_DragMove` behavior with CE.
- `cargo check --features unicorn,trace,win32-desktop` passed after aligning no-active `ImageList_DragEnter`/`ImageList_DragLeave` static context and visible-only `ImageList_DragMove` behavior with CE.
- `cargo test --features unicorn,trace,win32-desktop` passed after aligning no-active `ImageList_DragEnter`/`ImageList_DragLeave` static context and visible-only `ImageList_DragMove` behavior with CE.
- `git diff --check` passed after aligning no-active `ImageList_DragEnter`/`ImageList_DragLeave` static context and visible-only `ImageList_DragMove` behavior with CE; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after aligning no-active `ImageList_GetDragImage` behavior with CE.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after aligning no-active `ImageList_GetDragImage` behavior with CE.
- `cargo check --features unicorn,trace,win32-desktop` passed after aligning no-active `ImageList_GetDragImage` behavior with CE.
- `cargo test --features unicorn,trace,win32-desktop` passed after aligning no-active `ImageList_GetDragImage` behavior with CE.
- `git diff --check` passed after aligning no-active `ImageList_GetDragImage` behavior with CE; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after aligning `ImageList_SetDragCursorImage` no-active-drag behavior with CE.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems resource_system_owned_bitmap_bitmap_mut_region_rects_palette_mut_shell_image_list_merge_and_dither` passed after aligning `ImageList_SetDragCursorImage` no-active-drag behavior with CE.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems resource_image_list_duplicate_merge_add_masked_replace_remove_copy_overlay_drag` passed after aligning `ImageList_SetDragCursorImage` no-active-drag behavior with CE.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after aligning `ImageList_SetDragCursorImage` no-active-drag behavior with CE.
- `cargo check --features unicorn,trace,win32-desktop` passed after aligning `ImageList_SetDragCursorImage` no-active-drag behavior with CE.
- `cargo test --features unicorn,trace,win32-desktop` passed after aligning `ImageList_SetDragCursorImage` no-active-drag behavior with CE.
- `git diff --check` passed after aligning `ImageList_SetDragCursorImage` no-active-drag behavior with CE; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after aligning `ImageList_DragMove` with CE's unconditional success return.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems resource_image_list_duplicate_merge_add_masked_replace_remove_copy_overlay_drag` passed after aligning `ImageList_DragMove` with CE's unconditional success return.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after aligning `ImageList_DragMove` with CE's unconditional success return.
- `cargo check --features unicorn,trace,win32-desktop` passed after aligning `ImageList_DragMove` with CE's unconditional success return.
- `cargo test --features unicorn,trace,win32-desktop` passed after aligning `ImageList_DragMove` with CE's unconditional success return.
- `git diff --check` passed after aligning `ImageList_DragMove` with CE's unconditional success return; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after aligning `ImageList_Copy` with CE same-list-only copy/swap semantics.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems resource_system_image_list_duplicate_replace_remove_copy_count_overlay_and_drag` passed after aligning `ImageList_Copy` with CE same-list-only copy/swap semantics.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems resource_image_list_duplicate_merge_add_masked_replace_remove_copy_overlay_drag` passed after aligning `ImageList_Copy` with CE same-list-only copy/swap semantics.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_copy_honors_ce_move_swap_flags` passed after aligning `ImageList_Copy` with CE same-list-only copy/swap semantics.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after aligning `ImageList_Copy` with CE same-list-only copy/swap semantics.
- `cargo check --features unicorn,trace,win32-desktop` passed after aligning `ImageList_Copy` with CE same-list-only copy/swap semantics.
- `cargo test --features unicorn,trace,win32-desktop` passed after aligning `ImageList_Copy` with CE same-list-only copy/swap semantics.
- `git diff --check` passed after aligning `ImageList_Copy` with CE same-list-only copy/swap semantics; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after aligning `ImageList_SetIconSize` with CE no-op failure and remove-all resize semantics.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems resource_system_image_list_create_add_count_info_bk_color_and_destroy` passed after aligning `ImageList_SetIconSize` with CE no-op failure and remove-all resize semantics.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after aligning `ImageList_SetIconSize` with CE no-op failure and remove-all resize semantics.
- `cargo check --features unicorn,trace,win32-desktop` passed after aligning `ImageList_SetIconSize` with CE no-op failure and remove-all resize semantics.
- `cargo test --features unicorn,trace,win32-desktop` passed after aligning `ImageList_SetIconSize` with CE no-op failure and remove-all resize semantics.
- `git diff --check` passed after aligning `ImageList_SetIconSize` with CE no-op failure and remove-all resize semantics; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after enforcing CE `ILC_MASK` requirements for `ImageList_SetOverlayImage`.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems resource_system_image_list_duplicate_replace_remove_copy_count_overlay_and_drag` passed after enforcing CE `ILC_MASK` requirements for `ImageList_SetOverlayImage`.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems resource_image_list_duplicate_merge_add_masked_replace_remove_copy_overlay_drag` passed after enforcing CE `ILC_MASK` requirements for `ImageList_SetOverlayImage`.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after enforcing CE `ILC_MASK` requirements for `ImageList_SetOverlayImage`.
- `cargo check --features unicorn,trace,win32-desktop` passed after enforcing CE `ILC_MASK` requirements for `ImageList_SetOverlayImage`.
- `cargo test --features unicorn,trace,win32-desktop` passed after enforcing CE `ILC_MASK` requirements for `ImageList_SetOverlayImage`.
- `git diff --check` passed after enforcing CE `ILC_MASK` requirements for `ImageList_SetOverlayImage`; output was limited to existing CRLF normalization warnings.
- `cargo fmt` passed after aligning image-list overlay slot retention with CE.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems resource_system_image_list_duplicate_replace_remove_copy_count_overlay_and_drag` passed after aligning image-list overlay slot retention with CE.
- `cargo test --features unicorn,trace,win32-desktop` passed after aligning image-list overlay slot retention with CE.
- `git diff --check` passed after aligning image-list overlay slot retention with CE; output was limited to existing CRLF normalization warnings.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel sh_get_file_info_system_image_list_supports_icon_queries_and_draw` passed after aligning icon-only `SHGetFileInfoW` return semantics with CE.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel` passed after aligning icon-only `SHGetFileInfoW` return semantics with CE.
- `cargo check --features unicorn,trace,win32-desktop` passed after aligning icon-only `SHGetFileInfoW` return semantics with CE.
- `cargo test --features unicorn,trace,win32-desktop` passed after aligning icon-only `SHGetFileInfoW` return semantics with CE.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel shnotification_i_tracks_query_update_and_remove_state` passed after adding CE-style inform/iconic notification priority-list movement.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel` passed after adding CE-style inform/iconic notification priority-list movement.
- `cargo check --features unicorn,trace,win32-desktop` passed after adding CE-style inform/iconic notification priority-list movement.
- `cargo test --features unicorn,trace,win32-desktop` passed after adding CE-style inform/iconic notification priority-list movement.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel shnotification_i_add_uses_marshalled_html_pointer_presence` passed after aligning raw `SHNotificationAddI` title/HTML validation with CE marshalled pointer-presence semantics.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel` passed after aligning raw `SHNotificationAddI` title/HTML validation with CE marshalled pointer-presence semantics.
- `cargo check --features unicorn,trace,win32-desktop` passed after aligning raw `SHNotificationAddI` title/HTML validation with CE marshalled pointer-presence semantics.
- `cargo test --features unicorn,trace,win32-desktop` passed after aligning raw `SHNotificationAddI` title/HTML validation with CE marshalled pointer-presence semantics.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel shnotification_i_tracks_query_update_and_remove_state` passed after aligning raw `SHNotificationAddI` callback metadata with CE `cbData` semantics.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel` passed after aligning raw `SHNotificationAddI` callback metadata with CE `cbData` semantics.
- `cargo check --features unicorn,trace,win32-desktop` passed after aligning raw `SHNotificationAddI` callback metadata with CE `cbData` semantics.
- `cargo test --features unicorn,trace,win32-desktop` passed after aligning raw `SHNotificationAddI` callback metadata with CE `cbData` semantics.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_find_close_change_notification_consumes_wrong_handle_type` passed after matching CE wrong-handle close behavior for `FindCloseChangeNotification`.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file` passed after matching CE wrong-handle close behavior for `FindCloseChangeNotification`.
- `cargo check --features unicorn,trace,win32-desktop` passed after matching CE wrong-handle close behavior for `FindCloseChangeNotification`.
- `cargo test --features unicorn,trace,win32-desktop` passed after matching CE wrong-handle close behavior for `FindCloseChangeNotification`.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_root_change_notification_honors_subtree_flag` passed after fixing root `WatchSubtree` matching.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file` passed after fixing root `WatchSubtree` matching.
- `cargo check --features unicorn,trace,win32-desktop` passed after fixing root `WatchSubtree` matching.
- `cargo test --features unicorn,trace,win32-desktop` passed after fixing root `WatchSubtree` matching.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_readonly_mount_reports_access_denied_for_mutations` passed after adding read-only mounted-root mutation coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file` passed after adding read-only mounted-root mutation coverage.
- `cargo check --features unicorn,trace,win32-desktop` passed after adding read-only mounted-root mutation coverage.
- `cargo test --features unicorn,trace,win32-desktop` passed after adding read-only mounted-root mutation coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_move_file_w_enforces_ce_volume_boundaries` passed after adding CE mounted-volume `MoveFileW` boundary behavior.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file` passed after adding CE mounted-volume `MoveFileW` boundary behavior and fixing repeat setup for the copy-file raw test.
- `cargo check --features unicorn,trace,win32-desktop` passed after adding CE mounted-volume `MoveFileW` boundary behavior.
- `cargo test --features unicorn,trace,win32-desktop` passed after adding CE mounted-volume `MoveFileW` boundary behavior.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_find_next_change_notification_consumes_one_pending_signal` passed after adding CE outstanding notification signal-count behavior.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file` passed after adding CE outstanding notification signal-count behavior.
- `cargo check --features unicorn,trace,win32-desktop` passed after adding CE outstanding notification signal-count behavior.
- `cargo test --features unicorn,trace,win32-desktop` passed after adding CE outstanding notification signal-count behavior.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_change_notification_canonicalizes_watch_path` passed after adding CE canonical watch-path coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file` passed after adding CE canonical watch-path coverage.
- `cargo check --features unicorn,trace,win32-desktop` passed after adding CE canonical watch-path coverage.
- `cargo test --features unicorn,trace,win32-desktop` passed after adding CE canonical watch-path coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel shnotification_i_tracks_query_update_and_remove_state` passed after adding CE fixed-title-buffer `SHNotificationGetDataI` coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel` passed after adding CE fixed-title-buffer `SHNotificationGetDataI` coverage.
- `cargo check --features unicorn,trace,win32-desktop` passed after adding CE fixed-title-buffer `SHNotificationGetDataI` coverage.
- `cargo test --features unicorn,trace,win32-desktop` passed after adding CE fixed-title-buffer `SHNotificationGetDataI` coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel shnotification_i_tracks_query_update_and_remove_state` passed after adding notification callback cleanup.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel shell_window_destroy_removes_notify_icon_and_notification_state` passed after adding notification callback cleanup.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems shell_system_remove_window_state_and_remove_windows_state` passed after adding notification callback cleanup.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel` passed after adding notification callback cleanup.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems` passed after adding notification callback cleanup.
- `cargo check --features unicorn,trace,win32-desktop` passed after adding notification callback cleanup.
- `cargo test --features unicorn,trace,win32-desktop` passed after adding notification callback cleanup.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel coredll_raw_loadlibrary_refcounts_dynamic_modules_and_ex_flags_reuse_loaded_modules` passed after adding datafile export-suppression coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel` passed after adding datafile export-suppression coverage.
- `cargo check --features unicorn,trace,win32-desktop` passed after adding datafile export-suppression coverage.
- `cargo test --features unicorn,trace,win32-desktop` passed after adding datafile export-suppression coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed after the same-thread, early-`ReplyMessage`, and nested `SendMessageTimeout` slices.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_track_popup_menu_ex_applies_ce_alignment_flags` passed after adding CE `TPM_*ALIGN` popup positioning.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed after adding CE `TPM_*ALIGN` popup positioning.
- `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems image_list` passed after updating direct image-list copy tests to CE `ILCF_MOVE` semantics.
- `cargo test --features unicorn,trace,win32-desktop` passed after adding CE `TPM_*ALIGN` popup positioning and updating direct image-list move tests.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel sh_get_file_info` passed after adding secondary `RT_ICON` ordinal failure coverage.
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
- `cargo test -j 1 --features unicorn,trace,win32-desktop coredll_raw_kern_extract_icons_copies_group_rt_icon_payloads` passed after implementing raw CE `KernExtractIcons` resource-byte extraction.
- `cargo check --features unicorn,trace,win32-desktop`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel`, and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after implementing raw CE `KernExtractIcons` resource-byte extraction.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_draw_indirect` passed after enforcing exact `IMAGELISTDRAWPARAMS` size validation.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after enforcing CE `ILC_VALID` image-list creation flags.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_copy_honors_ce_move_swap_flags` passed after enforcing CE `ILCF_VALID` image-list copy flags.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_destroy_icon_accepts_loaded_icon_handles` passed after adding bitmap-backed `DrawIconEx` stretched selected-memory-DIB coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_destroy_icon_accepts_loaded_icon_handles` passed after adding bitmap-backed `DrawIconEx` `DI_MASK` source selection coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_destroy_icon_accepts_loaded_icon_handles` passed after adding bitmap-backed `DrawIconEx` framebuffer stretched-output coverage.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_destroy_icon_accepts_loaded_icon_handles` passed after adding 1bpp mask-only framebuffer `DrawIconEx` coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_destroy_icon_accepts_loaded_icon_handles` and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed after adding 1bpp mask-only selected-memory-DIB `DrawIconEx` coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_text_metrics_use_selected_logfont` passed after adding CE `GetTextFaceW` null-output and negative-count coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed after adding CE `GetTextFaceW` null-output and negative-count coverage.
- `cargo check --features unicorn,trace,win32-desktop` passed after adding CE `GetTextFaceW` null-output and negative-count coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding CE `GetTextFaceW` null-output and negative-count coverage; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_get_text_extent_ex_point_fills_fit_dx_and_size` passed after adding CE `GetTextExtentExPointW` invalid-parameter coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe` and `cargo check --features unicorn,trace,win32-desktop` passed after adding CE `GetTextExtentExPointW` invalid-parameter coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding CE `GetTextExtentExPointW` invalid-parameter coverage; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_gdi_set_bk_mode_returns_previous_mode` passed after adding CE `GetBkMode` invalid-HDC last-error coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe` and `cargo check --features unicorn,trace,win32-desktop` passed after adding CE `GetBkMode` invalid-HDC last-error coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding CE `GetBkMode` invalid-HDC last-error coverage; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_gdi_set_bk_mode_returns_previous_mode` passed after adding CE `GetBkColor` invalid-HDC last-error coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe` and `cargo check --features unicorn,trace,win32-desktop` passed after adding CE `GetBkColor` invalid-HDC last-error coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding CE `GetBkColor` invalid-HDC last-error coverage; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_gdi_set_bk_mode_returns_previous_mode`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, and `cargo check --features unicorn,trace,win32-desktop` passed after adding CE `CLR_INVALID` background/text color state coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding CE `CLR_INVALID` background/text color state coverage; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_gdi_device_attribute_modes_follow_ce_sentinels` and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed after adding CE stretch-mode, text-character-extra, and layout sentinel coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding CE stretch-mode, text-character-extra, and layout sentinel coverage; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_gdi_device_attribute_modes_follow_ce_sentinels` and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed after adding CE `SetViewportOrgEx` invalid-HDC coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding CE `SetViewportOrgEx` invalid-HDC coverage; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_gdi_save_restore_dc_follows_ce_levels` and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed after adding CE `SaveDC`/`RestoreDC` invalid-HDC and restore-level coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding CE `SaveDC`/`RestoreDC` invalid-HDC and restore-level coverage; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_gdi_device_attribute_modes_follow_ce_sentinels` and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed after adding CE `SetBrushOrgEx` invalid-HDC and previous-origin coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding CE `SetBrushOrgEx` invalid-HDC and previous-origin coverage; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_gdi_set_bk_mode_returns_previous_mode` and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed after adding CE color API invalid-HDC coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding CE color API invalid-HDC coverage; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_text_metrics_use_selected_logfont` and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed after adding CE `SetTextAlign`/`GetTextAlign` invalid-HDC coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding CE `SetTextAlign`/`GetTextAlign` invalid-HDC coverage; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_gdi_set_bk_mode_returns_previous_mode` and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed after adding CE `SetBkMode`/`GetBkMode` bad-HDC validation.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding CE `SetBkMode`/`GetBkMode` bad-HDC validation; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe ext_text_out` and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed after adding CE `ExtTextOutW` `OPAQUE` background-mode text-cell fill coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding CE `ExtTextOutW` `OPAQUE` background-mode text-cell fill coverage; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe ext_text_out` and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed after adding CE `ExtTextOutW` invalid-HDC/null-text parameter coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding CE `ExtTextOutW` invalid-HDC/null-text parameter coverage; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe mask_blt` and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed after adding CE `MaskBlt` validation plus selected-memory-DIB and framebuffer 1bpp mask-copy coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding CE `MaskBlt` validation plus selected-memory-DIB and framebuffer 1bpp mask-copy coverage; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe alpha_blend` passed after adding framebuffer destination source-constant-alpha and top-down/bottom-up 32 bpp per-pixel-alpha `AlphaBlend` coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe` and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding framebuffer destination source-constant-alpha and top-down/bottom-up 32 bpp per-pixel-alpha `AlphaBlend` coverage; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe send_message_timeout`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding CE-public `SMTO_NORMAL` nonzero cross-thread timeout completion coverage; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel`, and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding CE-style image-list source/mask bitmap copy and caller-bitmap deletion lifetime coverage; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel`, `cargo check --features unicorn,trace,win32-desktop`, and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after aligning `ImageList_Duplicate` with CE bitmap/mask deep-copy ownership; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe alpha_blend`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding selected-memory-DIB top-down/bottom-up 32 bpp per-pixel-alpha `AlphaBlend` coverage and fixing the modal remote-button test tap to use the actual button screen rect; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file file_notification_info_partially_drains_pending_records` passed after aligning the `CeGetFileNotificationInfo` null-buffer/nonzero-length path with CE `NotifyReset`.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file` and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after aligning the `CeGetFileNotificationInfo` null-buffer/nonzero-length path with CE `NotifyReset`; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel` passed after aligning `ImageList_CopyDitherImage` mask updates with CE's 50% mono-dither/SRCAND path.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after aligning `ImageList_CopyDitherImage` mask updates with CE's 50% mono-dither/SRCAND path; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `git diff --check` passed after aligning `ImageList_CopyDitherImage` mask updates with CE's 50% mono-dither/SRCAND path; output was limited to existing CRLF normalization warnings.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_readonly_mount_reports_access_denied_for_mutations`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`, `cargo check --features unicorn,trace,win32-desktop`, and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding CE mount-root mutation guards and read-only mount access-denied notification non-signaling coverage; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_readonly_mount_reports_access_denied_for_mutations`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`, `cargo check --features unicorn,trace,win32-desktop`, and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after aligning `DeleteFileW` on mount roots with CE `FS_DeleteFileW`'s `ERROR_FILE_NOT_FOUND` guard; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_directory_watch_reports_self_rename_and_remove_as_current_removed` passed after matching CE `NotifyPathChangeEx` current-directory removal records for watched directory rename/removal.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`, `cargo check --features unicorn,trace,win32-desktop`, and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after matching CE `NotifyPathChangeEx` current-directory removal records for watched directory rename/removal; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_duplicate_handle_close_source_preserves_notification_duplicate`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`, `cargo check --features unicorn,trace,win32-desktop`, and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after replacing the raw `DuplicateHandle` alias stub with CE-style source validation, independent local duplicates, and `DUPLICATE_CLOSE_SOURCE` ownership transfer; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_change_notification_handles_are_process_owned`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`, `cargo check --features unicorn,trace,win32-desktop`, and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding CE-style process ownership checks for public file-change notification handles; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_file_notification_info_uses_ce_nul_padded_record_lengths`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`, `cargo check --features unicorn,trace,win32-desktop`, and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after matching CE `NotifyReset` trailing-NUL record sizing for `CeGetFileNotificationInfo`; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_afs_change_notification_uses_hproc_owner`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`, `cargo check --features unicorn,trace,win32-desktop`, and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after honoring direct AFS notification `hProc` ownership; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo fmt`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_file_notification_info_partially_drains_pending_records`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`, `cargo check --features unicorn,trace,win32-desktop`, and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after matching CE `NotifyReset` no-pending guarded output-pointer writes for `CeGetFileNotificationInfo`; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo fmt`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_change_notification_preserves_unknown_filter_bits`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test basic_subsystems kernel_file_pointer_size_position_flush_find_first_next_close_and_change_notification`, `cargo check --features unicorn,trace,win32-desktop`, and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after matching CE unknown notification filter preservation for `FindFirstChangeNotificationW`; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel shell_notify_icon_posts_registered_taskbar_message_with_copied_data`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test basic_subsystems shell_system_message_box_recent_docs_notify_icons_and_notifications`, `cargo check --features unicorn,trace,win32-desktop`, and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after routing `Shell_NotifyIcon` through registered CE taskbar posts with copied `NOTIFYICONDATAW` payloads; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel shell_notify_icon_posts_registered_taskbar_message_with_copied_data`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel`, and `cargo check --features unicorn,trace,win32-desktop` passed after adding CE taskbar copied-payload release on `DispatchMessageW`.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --bin wince_emulation_v3 saved_remote_input_target_uses_saved_get_message_waiter` passed after fixing the stale saved-GetMessage remote-input unit fixture, and the full `cargo test -j 1 --features unicorn,trace,win32-desktop` suite passed afterward; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel shnotification_i_tracks_query_update_and_remove_state`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel`, and `cargo check --features unicorn,trace,win32-desktop` passed after adding CE `SHNotificationUpdateI` non-null icon replacement cleanup.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe alpha_blend` and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed after adding CE `AlphaBlend` invalid-HDC/blend-function validation and selected-memory-DIB source-constant-alpha coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe alpha_blend`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, `cargo check --features unicorn,trace,win32-desktop`, and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after aligning CE premultiplied `AC_SRC_ALPHA` and non-premultiplied `AC_SRC_ALPHA_NONPREMULT` `AlphaBlend` math; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_blt_validates_ce_hdc_and_rop_edges`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_mask_blt_validates_ce_mask_parameters`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding CE `BitBlt`/`StretchBlt` invalid-HDC and invalid-ROP4 validation and correcting `MaskBlt` source-HDC last-error behavior; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
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
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_msgwait_returns_message_index_after_all_unsignaled_handles`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, `cargo check --features unicorn,trace,win32-desktop`, `cargo test -j 1 --features unicorn,trace,win32-desktop`, and `git diff --check` passed after adding CE `MsgWaitForMultipleObjectsEx` message-wake return-index coverage after multiple unsignaled handles; `git diff --check` output was limited to existing CRLF normalization warnings and the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop fsdmgr_notification_import`, `cargo check --features unicorn,trace,win32-desktop`, and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after adding `fsdmgr.dll` notification import trapping for CE `FSINT_*`/`FSEXT_*` names and ordinals 68-75; the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_change_notification_handles_are_process_owned`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`, `cargo check --features unicorn,trace,win32-desktop`, `cargo test -j 1 --features unicorn,trace,win32-desktop`, and `git diff --check` passed after making raw `CloseHandle` reject foreign-owned public file-change notification handles with `ERROR_ACCESS_DENIED` without consuming the handle; `git diff --check` output was limited to existing CRLF normalization warnings and the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test basic_subsystems resource_system_image_list_create_add_count_info_bk_color_and_destroy`, `cargo check --features unicorn,trace,win32-desktop`, `cargo test -j 1 --features unicorn,trace,win32-desktop`, and `git diff --check` passed after aligning `ImageList_SetIconSize` changed zero/negative dimensions with CE `imagelist.cpp`; `git diff --check` output was limited to existing CRLF normalization warnings and the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_draw_indirect_applies_x_bitmap_offset_and_rgb_bk_fill`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test basic_subsystems resource_system_image_list_duplicate_replace_remove_copy_count_overlay_and_drag`, `cargo check --features unicorn,trace,win32-desktop`, and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after aligning same-slot `ImageList_SetOverlayImage` bounds preservation with CE `imagelist.cpp`; the Cargo runs required unsandboxed execution after sandboxed CMake/Ninja probing for `unicorn-engine-sys` failed with `operation not permitted`, and the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test basic_subsystems resource_image_list_duplicate_merge_add_masked_replace_remove_copy_overlay_drag`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons`, `cargo check --features unicorn,trace,win32-desktop`, `cargo test -j 1 --features unicorn,trace,win32-desktop`, and `git diff --check` passed after aligning hidden-after-`ImageList_BeginDrag` visibility and pre-`DragEnter` `DragMove` point preservation with CE `imagelist.cpp`; the first sandboxed Cargo probe hit the known `unicorn-engine-sys` CMake/Ninja `operation not permitted` restriction, so the feature Cargo runs were rerun unsandboxed, `git diff --check` output was limited to existing CRLF normalization warnings, and the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- Raw `ImageList_AddMasked(CLR_DEFAULT)` now follows CE `imagelist.cpp::AddMasked` by sampling the source bitmap's upper-left pixel before storing the transparent color; the raw image-list regression confirms a magenta/green bitmap masks the magenta pixel and still draws the green pixel.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons`, `cargo check --features unicorn,trace,win32-desktop`, `cargo test -j 1 --features unicorn,trace,win32-desktop`, and `git diff --check` passed after aligning raw `ImageList_AddMasked(CLR_DEFAULT)` with CE upper-left color sampling; `git diff --check` output was limited to existing CRLF normalization warnings and the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- Raw `ImageList_LoadImage(CLR_DEFAULT)` now follows CE `LoadImageW_I` by routing the mask through the same `ImageList::AddMasked` upper-left color sampling as direct `ImageList_AddMasked`; the raw image-list regression confirms the loaded magenta/green bitmap masks the sampled magenta pixel and still draws the green pixel.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons`, `cargo check --features unicorn,trace,win32-desktop`, `cargo test -j 1 --features unicorn,trace,win32-desktop`, and `git diff --check` passed after aligning raw `ImageList_LoadImage(CLR_DEFAULT)` with CE upper-left color sampling; `git diff --check` output was limited to existing CRLF normalization warnings and the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- Raw bitmap-backed `ImageList_AddMasked` and masked `ImageList_LoadImage` now create CE-style owned mono mask bitmaps, mark mask-color pixels white, punch those source pixels to black, and keep bitmap rendering mask-driven even when transparent color metadata is also present; the raw regression now verifies real 1bpp mask handles and post-`SetBkColor` transparent drawing.
- `cargo fmt --check` and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel image_list_ordinals_track_created_lists_and_icons` passed after aligning bitmap-backed `ImageList_AddMasked` mono-mask creation/rendering with CE `imagelist.cpp`.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel`, `cargo check --features unicorn,trace,win32-desktop`, `cargo test -j 1 --features unicorn,trace,win32-desktop`, and `git diff --check` passed after the CE mono-mask AddMasked slice; `git diff --check` output was limited to existing CRLF normalization warnings and the eVC4 MIPSII fixture test remains ignored because the toolchain is not configured.
- Raw `AlphaBlend` now matches CE `AlphaBlendGoodRectTest`/`AlphaBlendBadRectTest`
  rectangle handling for selected-DIB sources: zero source or destination
  dimensions succeed without painting, while negative dimensions and
  out-of-bounds nonempty selected-DIB source rectangles fail with
  `ERROR_INVALID_PARAMETER`.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe alpha_blend`
  passed after the CE `AlphaBlend` rectangle-parity update.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  same `AlphaBlend` rectangle-parity update; the eVC4 MIPSII fixture test
  remains ignored because the toolchain is not configured.
- `git diff --check` passed after the CE `AlphaBlend` rectangle-parity update;
  output was limited to existing CRLF normalization warnings.
- Raw `AlphaBlend` selected-DIB and framebuffer stretch paths now use CE GPE
  `swblt.cpp` Bresenham-style source sampling for uneven stretches instead of
  floor-division sampling; focused tests cover 2-to-3 selected-DIB and 3-to-5
  framebuffer alpha stretches where the source-pixel repetition pattern differs.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe alpha_blend`
  passed after the CE GPE alpha-stretch sampling update.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  same CE GPE alpha-stretch sampling update; the eVC4 MIPSII fixture test
  remains ignored because the toolchain is not configured.
- `git diff --check` passed after the CE GPE alpha-stretch sampling update;
  output was limited to existing CRLF normalization warnings.

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

## Runtime Diagnostics (2026-06-11)

- Verified the initial `happyway_win` "Quit Happyway" dialog is real helper UI,
  not emulator-invented UI. The unreadable/undismissable behavior came from the
  modal loop and scheduler not servicing cross-thread sent messages before
  posted input. `MessageBoxW` modal pumping now drains ready sent messages, and
  modal waits wake for pending sends.
- Verified CE dump IOCTL contracts directly with LLVM tools from
  `D:\GitHub\llvm-proj\build-mips-objdump\bin`: `YAS526B.dll` exposes the
  `0xb0000000..0xb0000010` magnetometer family, `light_sensor_drv.dll` exposes
  the `0xd2000004/8/14/18` light-sensor family, and the I2C DLLs differ by bus
  (`I2C3:` accepts `0x80002001`, `I2C2:`/`I2C4:` do not). The I2C emulation now
  derives per-bus behavior from the configured guest device name.
- The old `SMB380.dll` `0xb100...` assumption was not verified in the CE DLL
  dump; the observed SMB380 handler uses a dense `0x01012ee0..0x01012fcc`
  command family. Do not implement `0xb100...` SMB behavior without a real
  caller/source reference.
- The SD card mount is configured writable (`mounts.toml`, `\SDMMC Disk` to
  `D:\INAVI_Emulator\INAVI`) and a prior route-test file was created there, so
  the current sensor/dialog failure is not explained by read-only SD storage.
- Remote touch posting now records and wakes the actual hit-test target threads.
  This fixes the observed case where taps hit the iNavi window on thread 1 while
  the host loop kept resuming the unrelated iSearch thread 4.
- Added live device diagnostics at `/api/v1/debug/devices.txt`; current driven
  iNavi runs only show `UID1:`/`NandUuid` IOCTLs (`0xa0000010` and
  `0xa000000c`) before the later startup stall, so the G-sensor/SMB/light/I2C
  paths have not been reached in this run.
- The `happyway_win` `MessageBoxW` dialog is now generically dismissible through
  its real button window (`OK`, rect `47,67-101,89`): the button click queues
  `WM_COMMAND`, destroys the modal dialog subtree, removes the live modal waiter,
  and subsequent remote taps hit the iNavi window (`0x00020008`) rather than the
  dead dialog. The old modal wait remains visible only in stale stop snapshots.
- Active modal `MessageBoxW` waits are now checked from the main remote loop
  before idling, using the same generic modal teardown path as parked-process
  modal waits. A fresh driven run records `button_click`, `dialog_modal_button_dismiss`,
  and `end_dialog` after tapping the real Happyway `OK` button.
- Live-pump current-timeout handling now coalesces inline `Sleep`/single-wait
  timeout completions for a bounded guest-time slice instead of stopping Unicorn
  after every `Sleep(500)`. This lets remote-driven startup advance past the
  earlier splash-only state into map-data loading while still yielding
  periodically for snapshots and process rotation.
- Current remote-driven frontier: after the Happyway modal returns, the runtime
  now clears orphaned cross-process `SendMessageW` yield snapshots when their
  live GWE sent-message record is gone, matching the current CE send queue as
  the source of truth. The next checkpoint is a fresh driven startup run to
  prove the parked sender resumes past the prior `0x008cc9b0` stop and to find
  the next live blocker. No sensor IOCTLs are observed yet.
- Fresh remote-driven runs now dismiss the real Happyway modal, remove the live
  modal waiter, and keep taps routed to the visible iNavi window. The scheduler
  no longer rotates on stale blocked-wait snapshots owned by another process,
  and live-pump rotation ignores dead/stale modal `GetMessage` snapshots. After
  a post-dialog tap, iNavi advances into resource/map loading (`SetFilePointer`
  on Coredll ordinal 173 at `iNavi.exe+0x642f8`, handle `0x440`) and opens
  `mapdata`, `iNaviData`, font, and resource files. The app still stays on the
  animated splash during this run, and device traffic remains limited to
  `UID1:` NAND UUID IOCTLs; GPS/IMU/light/I2C/SMB traffic has not been reached.
- `drive50` validation after the visible-work scheduler handoff patch:
  Happyway `OK` dismisses through the real dialog button, live waits clear,
  active state returns to iNavi (`pid=1`) after dismissal and after main-surface
  taps, and remote touches are recorded against iNavi's visible `0x00020008`
  window. The splash continues to animate and resource reads continue, but the
  UI still does not transition to the map. Sensor REST injection succeeds, yet
  the guest still only opens/issues IOCTLs to `UID1:`.
- `tests/coredll_raw_memory_file.rs`: direct `AFS_FindFirstChangeNotificationW`
  hProc owner coverage now verifies a foreign raw ordinal caller cannot
  directly `CloseHandle` or `DuplicateHandle` the target-process-owned
  notification, reports `ERROR_ACCESS_DENIED`, preserves the live owner handle,
  and leaves the duplicate output pointer unchanged.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after
  extending direct AFS `hProc` notification-owner coverage; the eVC4 MIPSII
  fixture test remains ignored because the toolchain is not configured.
- `src/emulator/imports.rs`: FSDMGR notification import coverage now exercises
  `FSEXT_FindFirstChangeNotificationW` creation, validates the created handle
  remains owner-scoped when reset through the FSDMGR import from a foreign
  process, and closes it through the paired FSDMGR close import as the owner.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop emulator::imports::tests::fsdmgr_notification_import_first_change_preserves_owner`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after
  adding the FSDMGR first-change import owner regression; the eVC4 MIPSII
  fixture test remains ignored because the toolchain is not configured.
- `src/ce/kernel.rs`, `src/ce/coredll.rs`, and `src/emulator/imports.rs`:
  FSDMGR `FSINT_FindCloseChangeNotification` now follows CE
  `INT_NotifyCloseChangeHandle` by rejecting valid non-notification handles
  without consuming them, while the public/`FSEXT` path keeps the existing
  consume-then-error behavior for valid-but-wrong caller handles.
- `cargo fmt`, `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop emulator::imports::tests::fsdmgr_internal_close_rejects_wrong_handle_without_consuming_it`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after
  splitting internal and external FSDMGR notification close behavior; the eVC4
  MIPSII fixture test remains ignored because the toolchain is not configured.
- `src/ce/coredll.rs` and `tests/coredll_raw_memory_file.rs`: pending
  `CeGetFileNotificationInfo` no-fit buffers now follow CE `NotifyReset` by
  writing `lpBytesAvailable` before `lpBytesReturned` and returning
  `ERROR_INVALID_PARAMETER` when a bad non-null returned-count pointer faults
  after the available byte count is stored.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_file_notification_info_partially_drains_pending_records`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`,
  `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed after aligning pending
  `CeGetFileNotificationInfo` no-fit output-count write order with CE
  `NotifyReset`; `git diff --check` output was limited to existing CRLF
  normalization warnings and the eVC4 MIPSII fixture test remains ignored
  because the toolchain is not configured.
- `src/ce/coredll.rs` and `tests/coredll_raw_memory_file.rs`: fitted
  `CeGetFileNotificationInfo` records now follow CE `NotifyReset` by draining
  records after a successful caller-buffer copy but before
  `lpBytesAvailable`/`lpBytesReturned` writes, so a bad returned-count pointer
  reports `ERROR_INVALID_PARAMETER` while the copied record is consumed.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_file_notification_info_count_fault_drains_copied_records`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`,
  `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed after matching CE `NotifyReset` fitted-record
  count-pointer fault drain behavior; `git diff --check` output was limited to
  existing CRLF normalization warnings and the eVC4 MIPSII fixture test remains
  ignored because the toolchain is not configured.
- `src/ce/coredll.rs`, `src/ce/kernel.rs`, and
  `tests/coredll_raw_memory_file.rs`: fitted `CeGetFileNotificationInfo`
  records now copy and drain one record at a time like CE `NotifyReset`, so a
  later caller-buffer fault consumes the already copied prefix, leaves the
  failed reset un-signaled, and lets a later direct info query retrieve the
  remaining records.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_file_notification_info_partial_buffer_fault_drains_copied_prefix`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`,
  `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed after matching CE `NotifyReset` partial
  caller-buffer fault copied-prefix drain behavior; `git diff --check` output
  was limited to existing CRLF normalization warnings and the eVC4 MIPSII
  fixture test remains ignored because the toolchain is not configured.
- `src/ce/coredll.rs` and `tests/coredll_raw_memory_file.rs`: no-pending
  `CeGetFileNotificationInfo` now follows CE `NotifyReset` by treating
  `lpBytesReturned` as the first mandatory guarded write, so a null returned
  pointer leaves `lpBytesAvailable` untouched while the call still reports
  `ERROR_NO_MORE_ITEMS`.
- `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_file_notification_info_partially_drains_pending_records`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`,
  `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed after matching CE `NotifyReset` no-pending
  null-returned pointer ordering; `git diff --check` output was limited to
  existing CRLF normalization warnings and the eVC4 MIPSII fixture test remains
  ignored because the toolchain is not configured.
- `src/ce/coredll.rs` and `tests/coredll_raw_memory_file.rs`: all-zero
  `CeGetFileNotificationInfo` calls now follow CE
  `INT_NotifyGetNextChange`/`NotifyReset` by passing null reset data, returning
  success, and consuming exactly one outstanding notification signal without
  forcing a detail-buffer fetch.
- `cargo fmt` and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_file_notification_info_all_zero_args_resets_one_pending_signal`
  passed after matching the CE all-zero no-data reset branch for
  `CeGetFileNotificationInfo`.
- `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`,
  `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed after matching the CE all-zero no-data reset branch
  for `CeGetFileNotificationInfo`; `git diff --check` output was limited to
  existing CRLF normalization warnings and the eVC4 MIPSII fixture test remains
  ignored because the toolchain is not configured.
- Remote iNavi drive update: the Happyway popup is a real guest
  `happyway_win.exe` `MessageBoxW` (`Quit Happyway`, title `Button`), not a
  synthetic iNavi UI element. The real OK button is at roughly `47,67-101,89`;
  tapping `(74,78)` dismisses it.
- `src/emulator/unicorn.rs`: parked modal `MessageBoxW` resume paths now pass
  the live framebuffer into modal teardown, so the existing backing-store
  restore removes the dialog pixels after remote-button dismissal.
- `src/emulator/unicorn.rs`: persisted parked process state now filters out
  entries whose process id matches the active kernel process, preventing the
  active process from being queued as its own parked sibling.
- `src/ce/kernel.rs`, `src/ce/object.rs`, and `src/main.rs`: added generic
  device debug text to `/api/v1/debug/devices.txt`, including configured
  device names, remote GPS/IMU queue state, and open device handles.
- Dump-derived IOCTL verification: SMB380, YAS526B, light sensor, and the four
  GIO I2C DLLs match the implemented command families; the suspected command
  offset error was not supported by the DLL disassembly.
- `drive64` is the current clean running process (`wince_emulation_v3.exe` PID
  31812 at launch verification). It uses `registry.reg`, `mounts.toml`, the
  iNavi image from `D:\INAVI_Emulator\INAVI\INavi\iNavi.exe`, and the DLL dump
  from `D:\INAVI_Emulator\DUMPPLZ\Windows`.
- Sensor REST injection succeeds and queues GPS NMEA/IMU state, but the guest
  still has not opened `COM*`, `SMB1:`, `MFS1:`, or `LSD1:` during the observed
  splash/resource-loading phase. Device traffic remains limited to `UID1:`
  NAND UUID IOCTLs.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `AlphaBlend` now
  accepts the CE GPE `BLT_ALPHASRCNEG`/`BLT_ALPHADESTNEG` blend-flag bits from
  `winddi.h`; the source-negation path mirrors `swblt.cpp` by inverting both
  source constant alpha and per-pixel source alpha before blending selected-DIB
  and framebuffer destinations.
- `cargo fmt --check` and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe alpha_blend_honors_ce_source`
  passed after adding CE `BLT_ALPHASRCNEG` AlphaBlend coverage.
- `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after
  the CE `BLT_ALPHASRCNEG` AlphaBlend slice; the eVC4 MIPSII fixture remains
  ignored because the toolchain is not configured.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `GetTextExtentExPointW`, `DrawTextW`, and `ExtTextOutW` now apply CE
  `SetTextCharacterExtra` advance math from `text.cpp`, including positive
  widening, first-character-preserving negative spacing, and selected character
  extra added to `ExtTextOutW` caller `lpDx` advances.
- `cargo fmt --check` and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_get_text_extent_ex_point_applies_text_character_extra`
  passed after adding CE text-character-extra extent coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed after the CE text-character-extra slice; the eVC4
  MIPSII fixture remains ignored because the toolchain is not configured, and
  `git diff --check` output was limited to existing CRLF normalization
  warnings.
- `src/ce/resource.rs`, `src/ce/coredll.rs`,
  `tests/coredll_raw_gwe.rs`, and `tests/basic_subsystems.rs`: raw
  `DeleteObject` now fails without destroying a GDI object that remains
  selected into any live DC, preserving selected custom-font metrics until the
  caller selects a different font before deletion like the CE GDI text tests.
- `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_text_metrics_use_selected_logfont`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test basic_subsystems resource_system_dc_palette_bk_mode_color_text_align_rop2_is_memory_and_delete_gdi`
  passed after adding selected GDI object deletion coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  and `cargo test -j 1 --features unicorn,trace,win32-desktop --test basic_subsystems`
  passed after the selected GDI object deletion slice.
- `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed after the selected GDI object deletion slice; the
  eVC4 MIPSII fixture remains ignored because the toolchain is not configured,
  and `git diff --check` output was limited to existing CRLF normalization
  warnings.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `GetCharABCWidthsI` now follows the CE `wingdi.h` signature by accepting
  `giFirst`, `cgi`, optional WORD glyph indices, and the fifth-argument ABC
  output buffer, instead of sharing the non-`I` `GetCharABCWidths` first/last
  range parser.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_get_char_abc_widths_i_uses_glyph_count_signature`
  passed after splitting the CE `GetCharABCWidthsI` ABI path.
- `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed after splitting the CE `GetCharABCWidthsI` ABI path;
  `git diff --check` output was limited to existing CRLF normalization warnings
  and the eVC4 MIPSII fixture test remains ignored because the toolchain is not
  configured.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `ExtTextOutW` now
  intersects glyph output and `OPAQUE` text-cell background fills with the
  selected DC clip region, in addition to any `ETO_CLIPPED` rectangle, for
  selected-DIB and framebuffer rendering paths.
- `cargo fmt --check` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_ext_text_out_honors_dc_clip_region_on_selected_dib`
  passed after adding CE text DC clip-region coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed after the CE text DC clip-region slice.
- `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed after the CE text DC clip-region slice; the eVC4
  MIPSII fixture remains ignored because the toolchain is not configured, and
  `git diff --check` output was limited to existing CRLF normalization
  warnings.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `AlphaBlend` now
  threads CE GPE destination alpha through the selected-DIB and framebuffer
  blend paths, applies `BLT_ALPHADESTNEG` before output-alpha blending for
  32bpp selected-DIB and alpha-capable framebuffer destinations, and preserves
  the resulting alpha byte instead of forcing opaque 32bpp output.
- `cargo fmt --check` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_alpha_blend_honors_ce_destination_alpha_negation_between_32bpp_dibs`
  passed after adding CE `BLT_ALPHADESTNEG` destination-alpha coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  CE `BLT_ALPHADESTNEG` destination-alpha slice; the eVC4 MIPSII fixture
  remains ignored because the toolchain is not configured.
- `git diff --check` passed after the CE `BLT_ALPHADESTNEG` destination-alpha
  slice; output was limited to existing CRLF normalization warnings.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `MaskBlt` now
  normalizes negative destination extents like CE GPE `swblt.cpp` for masked
  selected-DIB and framebuffer draws, mirrors output through the ordered
  destination rectangle, and keeps source/mask sampling aligned with the
  original positive source/mask direction.
- `cargo fmt --check` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_mask_blt_mirrors_negative_destination_width_between_selected_dibs`
  passed after adding CE `MaskBlt` negative destination extent coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  CE `MaskBlt` negative destination extent slice; the eVC4 MIPSII fixture
  remains ignored because the toolchain is not configured.
- `git diff --check` passed after the CE `MaskBlt` negative destination extent
  slice; output was limited to existing CRLF normalization warnings.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: the shared selected-DIB
  and framebuffer bitmap draw helper now normalizes signed destination
  rectangles and maps destination pixels back to source pixels with CE
  bottom/right-plus-one mirroring semantics. Raw `BitBlt` now accepts negative
  source-copy destination extents, and raw `MaskBlt`'s null-mask `SRCCOPY`
  shortcut uses that path instead of failing through the old invalid-size
  `BitBlt` guard.
- `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe negative_destination_width_between_selected_dibs`
  passed after adding CE `draw.cpp::NegativeSize` source-copy coverage for
  direct `BitBlt`, masked `MaskBlt`, and null-mask `MaskBlt`.
- `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  CE signed-destination source-copy blit slice; the eVC4 MIPSII fixture
  remains ignored because the toolchain is not configured.
- `git diff --check` passed after the CE signed-destination source-copy blit
  slice; output was limited to existing CRLF normalization warnings.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `StretchBlt` now
  uses the CE signed destination rectangle helper for selected-DIB and
  framebuffer clipping, so CE `draw.cpp::NegativeSize(EStretchBlt)` and
  `StretchBltFlipMirrorTest`-style signed destination/source extents reach the
  shared mirrored source-coordinate mapping instead of clipping away negative
  destination rectangles.
- `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_stretchblt_mirrors_negative_extents_between_selected_dibs`
  passed after adding CE `StretchBlt` signed-extent selected-DIB coverage.
- `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  CE `StretchBlt` signed-extent slice; the eVC4 MIPSII fixture remains ignored
  because the toolchain is not configured.
- `git diff --check` passed after the CE `StretchBlt` signed-extent slice;
  output was limited to existing CRLF normalization warnings.
- `src/emulator/unicorn.rs`: live-pump handoff now gives visible/priority
  parked processes first chance before generic runnable-process rotation, so a
  hidden runnable child cannot beat a visible iNavi UI process when both are
  eligible after a run slice.
- `cargo fmt`, `cargo test live_pump_handoff_prefers_visible_parked_process_before_generic_runnable --features unicorn,trace,win32-desktop`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo build --release --features unicorn,trace,win32-desktop` passed after
  the live-pump handoff change; warnings were existing unused/dead-code warnings.
- Added `WINCE_EMU_FAST_START_LIVE` as an explicit override that allows
  `WINCE_EMU_FAST_START=1` to remain active for live-pump runs. In the current
  iNavi run, non-fast-start bounded CPU execution exits through the CE current
  process pseudo-handle (`0x42`) with exit code `3` from `mfcce400.dll+0xd674`,
  while fast-start mode keeps the process executing past that point.
- Launch diagnostics on `drive78` through `drive93` found a separate remote
  driving blocker: detached/hosted live launches can print
  `remote server: http://192.168.0.39:8765` and keep `wince_emulation_v3.exe`
  alive for several seconds, but no TCP listener appears on port `8765`
  (`Get-NetTCPConnection` and HTTP both show no listener/refused). Stable UI
  driving is blocked until the remote accept thread/lifetime issue is fixed.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw direct-DIB
  rendering now parses caller `BITMAPINFO` once and can render
  `StretchDIBits`/`SetDIBitsToDevice` into selected memory DIB destinations
  through the shared bitmap draw path, including CE-style `StretchDIBits`
  bad-HDC and ROP4-shaped ROP rejection before success. `SetDIBitsToDevice`
  now converts bottom-up DIB source scanline arguments into the renderer's
  normalized top-left coordinate space, matching CE
  `draw.cpp::SimpleSetDIBitsToDeviceTest`.
- `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe dibits`
  passed after adding the selected-memory-DIB direct-DIB coverage.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed with 276 raw GWE tests after the direct-DIB selected-memory-DIB slice.
- `cargo fmt --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  direct-DIB selected-memory-DIB slice; the eVC4 MIPSII fixture remains ignored
  because the toolchain is not configured.
- `git diff --check` passed after the direct-DIB selected-memory-DIB slice;
  output was limited to existing CRLF normalization warnings.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw direct-DIB source
  parsing now receives the caller's color-usage value, so
  `StretchDIBits`/`SetDIBitsToDevice` accept `DIB_PAL_COLORS` for direct
  indexed DIB sources and resolve WORD palette-index tables through the shared
  indexed bitmap renderer for selected-memory-DIB destinations.
- `cargo fmt`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe direct_dib`,
  and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed with 277 raw GWE tests after the direct-DIB `DIB_PAL_COLORS` slice.
- `cargo fmt --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  direct-DIB `DIB_PAL_COLORS` slice; the eVC4 MIPSII fixture remains ignored
  because the toolchain is not configured.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: indexed bitmap helpers
  now decode, read, and write 2 bpp packed pixels, so CE GDIPRINT-style direct
  `StretchDIBits`/`SetDIBitsToDevice` caller DIBs with 2 bpp
  `DIB_RGB_COLORS` color tables render through the shared selected-DIB path.
- `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_direct_2bpp_dib_uses_bitmapinfo_color_table`
  passed after the direct-DIB 2 bpp indexed source slice.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed with 278 raw GWE tests after the direct-DIB 2 bpp indexed source
  slice.
- `cargo fmt --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  direct-DIB 2 bpp indexed source slice; the eVC4 MIPSII fixture remains
  ignored because the toolchain is not configured.
- `src/ce/coredll.rs`, `src/ce/resource.rs`, and `tests/coredll_raw_gwe.rs`:
  raw `CreateDIBPatternBrushPt` now validates CE null/unsupported color-use
  inputs, copies packed guest DIB data into private owned bitmap backing, uses
  that backing for pattern-brush tiling, and releases the private bitmap when
  the brush is deleted.
- `cargo fmt`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_create_dib_pattern_brush_pt_uses_packed_dib`,
  and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed with 279 raw GWE tests after the packed-DIB pattern-brush slice.
- `cargo fmt --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  packed-DIB pattern-brush slice; the eVC4 MIPSII fixture remains ignored
  because the toolchain is not configured.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, and
  `SOURCE_REFERENCES.md`: the raw stock/fallback `Tahoma` `TEXTMETRICW` path
  now uses the CE GDIAPI `fontdata.h` Tahoma `tmPitchAndFamily == 39` and
  `tmCharSet == 0` bytes, with regression coverage before a custom font is
  selected into the DC.
- `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_text_metrics_use_selected_logfont`
  passed after adding the CE default Tahoma `TEXTMETRICW` metadata slice.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed with 280 raw GWE tests after the CE default Tahoma `TEXTMETRICW`
  metadata slice.
- `cargo fmt --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  CE default Tahoma `TEXTMETRICW` metadata slice; the eVC4 MIPSII fixture
  remains ignored because the toolchain is not configured.
- `git diff --check` passed after the CE default Tahoma `TEXTMETRICW` metadata
  slice; output was limited to existing LF-to-CRLF normalization warnings.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, and
  `SOURCE_REFERENCES.md`: plain 20px selected `Tahoma` fonts now use the CE
  GDIAPI `fontdata.h` `NTFontMetrics` row for the comparable `TEXTMETRICW`
  fields, while arbitrary custom fonts keep the existing deterministic metrics
  model.
- `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_text_metrics_use`
  passed after adding the CE plain 20px Tahoma metrics profile.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed with 281 raw GWE tests after the CE plain 20px Tahoma metrics profile.
- `cargo fmt --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the CE
  plain 20px Tahoma metrics profile; the eVC4 MIPSII fixture remains ignored
  because the toolchain is not configured.
- `git diff --check` passed after the CE plain 20px Tahoma metrics profile;
  output was limited to existing LF-to-CRLF normalization warnings.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, and
  `SOURCE_REFERENCES.md`: the CE 20px known-font metrics profile now covers
  the full `fontdata.h` `NTFontMetrics` row set for Tahoma, Courier New,
  Symbol, Times New Roman, Wingdings, and Verdana instead of only Tahoma.
- `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_text_metrics_use_ce_known_font_profiles`
  passed after extending the CE 20px known-font metrics profile.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed with 282 raw GWE tests after extending the CE 20px known-font metrics
  profile.
- `cargo fmt --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after
  extending the CE 20px known-font metrics profile; the eVC4 MIPSII fixture
  remains ignored because the toolchain is not configured.
- `git diff --check` passed after extending the CE 20px known-font metrics
  profile; output was limited to existing LF-to-CRLF normalization warnings.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, and
  `SOURCE_REFERENCES.md`: plain 16px known-font text extents now use the CE
  GDIAPI `fontdata.h` `NTExtentResults` rows for Tahoma, Courier New,
  Times New Roman, Wingdings, and Verdana in `GetTextExtentExPointW`,
  `GetCharWidth32`, and shared text-run measurement.
- `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_text_extents_use_ce_known_font_widths`
  passed after adding the CE 16px known-font extent-width profile.
- `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed after adding the CE 16px known-font extent-width
  profile; the full raw GWE suite passed with 283 tests, `git diff --check`
  output was limited to existing LF-to-CRLF normalization warnings, and the
  eVC4 MIPSII fixture remains ignored because the toolchain is not configured.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: the CE 16px known-font
  extent-width profile now includes the `fontdata.h` Arial raster row selected
  through the `"Arial"` face name, matching `text.cpp::GetTextExtentPointTest`.
- `src/emulator/unicorn.rs`, `tests/basic_subsystems.rs`, and
  `tests/coredll_raw_kernel.rs`: serial-read/comm-event tests now use the
  configured remote GPS target where they exercise runtime routing, while the
  raw comm-state/mask/purge test owns a one-device stub `COM7:` fixture so it
  stays stable when local `serial_devices.json` prefers a Win32-backed serial
  port.
- `cargo fmt --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  Arial raster extent row and serial test stabilization; the eVC4 MIPSII
  fixture remains ignored because the toolchain is not configured.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: raw `GetCharABCWidths` and
  `GetCharABCWidthsI` now use the CE GDIAPI `fontdata.h` Tahoma-only
  `NT_ABCWidths` table for plain selected 16px Tahoma glyphs from `!` through
  `z`, while generic fonts keep the existing average-width fallback.
- `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe coredll_raw_get_char_abc_widths -- --nocapture`
  passed after adding the CE Tahoma ABC table path.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed with 284 raw GWE tests after adding the CE Tahoma ABC table path.
- `cargo fmt --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  CE Tahoma ABC table path; the eVC4 MIPSII fixture remains ignored because
  the toolchain is not configured.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: raw `SetTextCharacterExtra`
  now rejects selected CE Arial raster fonts with the GDI error sentinel and
  preserves the previous DC spacing value, matching the CE GDIAPI text/font
  tests that skip character-extra handling for non-TrueType raster fonts.
- `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe text_character_extra -- --nocapture`
  passed after adding the CE Arial raster `SetTextCharacterExtra` rejection.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed with 285 raw GWE tests after adding the CE Arial raster
  `SetTextCharacterExtra` rejection.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  CE Arial raster `SetTextCharacterExtra` rejection; the eVC4 MIPSII fixture
  remains ignored because the toolchain is not configured.
- `cargo fmt --check` and
  `cargo check --features unicorn,trace,win32-desktop` passed after the CE
  Arial raster `SetTextCharacterExtra` rejection.
- `git diff --check` passed after the CE Arial raster
  `SetTextCharacterExtra` rejection; Git still reports LF-to-CRLF conversion
  warnings for the dirty worktree files.
- `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and
  `SOURCE_REFERENCES.md`: raw `CreateFontIndirectW` now has a CE
  `font.cpp::TestCreateFontIndirectZero` regression proving a selected zeroed
  `LOGFONTW` round-trips through `GetCurrentObject(OBJ_FONT)` and `GetObjectW`
  without being normalized to fallback font fields.
- `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe create_font_indirect_zero_logfont -- --nocapture`
  passed after adding the zeroed `LOGFONTW` regression.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed with 286 raw GWE tests after adding the zeroed `LOGFONTW` regression.
- `cargo fmt --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  zeroed `LOGFONTW` regression; the eVC4 MIPSII fixture remains ignored
  because the toolchain is not configured.
- `git diff --check` passed after the zeroed `LOGFONTW` regression; Git still
  reports LF-to-CRLF conversion warnings for the dirty worktree files.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: raw `GetTextMetricsW` now uses
  the CE `font.cpp::passOddSize` known-font realized `tmHeight` rows for
  `CreateFontIndirectW` selections with `lfHeight` 0 and -24 across Tahoma,
  Courier New, Symbol, Times New Roman, and Wingdings.
- `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe createfont_pass_odd_heights -- --nocapture`
  passed after adding the CE `passOddSize` known-font height rows.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed with 287 raw GWE tests after adding the CE `passOddSize` known-font
  height rows.
- `cargo fmt --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  CE `passOddSize` known-font height rows; the eVC4 MIPSII fixture remains
  ignored because the toolchain is not configured.
- `src/ce/resource.rs`, `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: raw font
  objects now preserve nonzero `LOGFONTW` escapement, orientation, precision,
  quality, pitch/family, style, charset, and face-name fields across
  `CreateFontIndirectW` and `GetObjectW`.
- `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe create_font_indirect_preserves_logfont_fields -- --nocapture`
  passed after adding the nonzero `LOGFONTW` round-trip regression.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed with 288 raw GWE tests after adding the nonzero `LOGFONTW`
  round-trip regression.
- `cargo fmt --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  nonzero `LOGFONTW` round-trip regression and test-constructor call-site
  updates; the eVC4 MIPSII fixture remains ignored because the toolchain is
  not configured.
- `git diff --check` passed after the CE `passOddSize` and nonzero
  `LOGFONTW` work; Git still reports LF-to-CRLF conversion warnings for the
  dirty worktree files.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: raw `GetCharABCWidths` now
  follows CE `font.cpp::abcEscapementTest` by rejecting selected fonts with
  nonzero escapement and setting `ERROR_INVALID_PARAMETER`.
- `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe rejects_nonzero_escapement -- --nocapture`
  passed after adding the CE nonzero-escapement ABC-width regression.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed with 289 raw GWE tests after adding the CE nonzero-escapement
  ABC-width regression.
- `cargo fmt --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  CE nonzero-escapement ABC-width regression; the eVC4 MIPSII fixture remains
  ignored because the toolchain is not configured.
- `git diff --check` passed after the CE nonzero-escapement ABC-width
  regression; Git still reports LF-to-CRLF conversion warnings for the dirty
  worktree files.
- `src/ce/gwe.rs`, `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: raw IMM
  contexts now preserve CE-sized `LOGFONTW` composition-font state across
  `ImmSetCompositionFontW`/`ImmGetCompositionFontW`, resolve NULL-HIMC calls
  through the active keyboard target, and post `IMN_SETCOMPOSITIONFONT` for
  `ImmNotifyIME(NI_CONTEXTUPDATED, ..., IMC_SETCOMPOSITIONFONT)`.
- `cargo fmt` and
  `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`
  passed after adding the CE IMM composition-font state regression.
- `cargo fmt --check` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the CE
  IMM composition-font work; the eVC4 MIPSII fixture remains ignored because
  the toolchain is not configured.
- `git diff --check` passed after the CE IMM composition-font work; Git still
  reports LF-to-CRLF conversion warnings for the dirty worktree files.
- `src/ce/gwe.rs`, `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: raw IMM
  contexts now carry stored CE candidate-list state, report candidate-list
  count/required bytes through `ImmGetCandidateListCountW`, marshal UTF-16
  `CANDIDATELIST` payloads through `ImmGetCandidateListW`, and mutate
  selection/page-start/page-size for TESTIME-shaped `ImmNotifyIME` candidate
  selection/page actions when a stored list exists.
- `cargo fmt` and
  `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`
  passed after adding the CE IMM candidate-list payload/state regression.
- `cargo fmt --check` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the CE
  IMM candidate-list payload/state work; the eVC4 MIPSII fixture remains
  ignored because the toolchain is not configured.
- `git diff --check` passed after the CE IMM candidate-list payload/state
  work; Git still reports LF-to-CRLF conversion warnings for the dirty
  worktree files.
- `src/ce/gwe.rs`, `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: SIP panel
  state now follows the CE `shellapi.h`/`imm.h`/shell PSL surface for
  `RegisterSIPanel`, `SHSipPreferenceI`, and
  `ImmSIPanelState(SIP_QUERY_LOCATION/SIP_SET_LOCATION/SIP_INPUT_ATTRIBUTES)`,
  including null/out-of-range preference validation, up/down visibility flags,
  remembered input-dialog child rejection, SIP rectangle round-trips, and
  input-attribute storage.
- `cargo fmt` and
  `cargo test coredll_raw_sip_panel_state_and_shell_preference_are_stateful --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`
  passed after adding the CE SIP panel state regression.
- `src/emulator/unicorn.rs`: parked-process switching now reconciles a bare
  default parked CPU thread id with the `ParkedProcess` wrapper thread id before
  deciding whether to requeue the outgoing process. This keeps sender/current
  parked entries from being pruned as duplicate thread-id owners when rotating
  to a synthetic parked receiver.
- `cargo test rotate_to_parked_process_skips_send_blocked_process_until_ready --lib --features unicorn,trace,win32-desktop -j 1`
  passed after the parked-process thread-id reconciliation.
- `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed after the CE SIP panel state and parked-process
  scheduler fixes; the eVC4 MIPSII fixture remains ignored because the
  toolchain is not configured, and Git still reports LF-to-CRLF conversion
  warnings for the dirty worktree files.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: `ImmGetDefaultIMEWnd` and the
  internal `DefaultImeWndGet` ordinal now share a CE default-IME-window proxy
  backed by the focused window, with valid caller HWND fallback until hidden IME
  control windows are emulated.
- `cargo fmt` and
  `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`
  passed after adding the CE default IME window regression.
- `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed after the CE default IME window proxy work; the
  eVC4 MIPSII fixture remains ignored because the toolchain is not configured,
  and Git still reports LF-to-CRLF conversion warnings for the dirty worktree
  files.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: `ImmIsUIMessageW` now recognizes
  the CE `WM_IME_*` UI message family from `imm.h`, forwards recognized messages
  to the supplied IME HWND or focused default-IME proxy, returns false for
  non-IME messages, and reports invalid-window failure when no target exists.
- `cargo fmt` and
  `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`
  passed after adding the CE `ImmIsUIMessageW` forwarding regression.
- `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed after the CE `ImmIsUIMessageW` forwarding work; the
  eVC4 MIPSII fixture remains ignored because the toolchain is not configured,
  and Git still reports LF-to-CRLF conversion warnings for the dirty worktree
  files.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: `ImmGetHotKey` now returns a
  BOOL false no-hotkey result while clearing optional modifier, virtual-key, and
  HKL outputs, matching the CE `imm.h` signature used by CHTIM startup without
  pretending an emulator IME-switch hotkey is registered.
- `cargo fmt` and
  `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`
  passed after adding the CE `ImmGetHotKey` no-hotkey regression.
- `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed after the CE `ImmGetHotKey` no-hotkey work; the
  eVC4 MIPSII fixture remains ignored because the toolchain is not configured,
  and Git still reports LF-to-CRLF conversion warnings for the dirty worktree
  files.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: `ImmGetGuideLineW` now follows
  the CE no-guideline query shape from `imm.h` and TESTIME, returning zero for
  `GGL_LEVEL`/`GGL_INDEX`, clearing caller string/private buffers for empty
  `GGL_STRING`/`GGL_PRIVATE` results, and reporting invalid index/handle errors
  without claiming HIMCC-backed guideline payload support yet.
- `cargo fmt` and
  `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`
  passed after adding the CE `ImmGetGuideLineW` no-guideline regression.
- `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed after the CE `ImmGetGuideLineW` no-guideline work;
  the eVC4 MIPSII fixture remains ignored because the toolchain is not
  configured, and Git still reports LF-to-CRLF conversion warnings for the dirty
  worktree files.
- `src/ce/gwe.rs`, `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`:
  `ImmLockIMC`/`ImmUnlockIMC` now expose a CE-shaped guest `INPUTCONTEXT`, and
  `ImmLockIMCC`/`ImmUnlockIMCC` expose lockable `COMPOSITIONSTRING` and
  `CANDIDATEINFO` buffers for the HIMC state already tracked by GWE, including
  `ImmGetIMCLockCount`, `ImmGetIMCCLockCount`, `ImmGetIMCCSize`,
  `ImmCreateIMCC`, and `ImmReSizeIMCC` behavior needed by TESTIME-style IME
  code.
- `cargo fmt` and
  `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`
  passed after adding the CE HIMC/HIMCC lock-buffer regression.
- `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed after the CE HIMC/HIMCC lock-buffer work; the eVC4
  MIPSII fixture remains ignored because the toolchain is not configured, and
  Git still reports LF-to-CRLF conversion warnings for the dirty worktree files.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: `ImmUnlockIMC` now retags
  assigned `INPUTCONTEXT` component handles by their CE field role, and
  `ImmGetGuideLineW` reads HIMCC-backed `GUIDELINE` headers through
  `hGuideLine`, returning `GGL_LEVEL`/`GGL_INDEX` and querying/copying
  `GGL_STRING`/`GGL_PRIVATE` payloads with CE byte-count return semantics.
- `cargo fmt` and
  `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`
  passed after adding the CE HIMCC-backed guideline payload regression.
- `cargo fmt --check` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the CE
  HIMCC-backed guideline payload work; the eVC4 MIPSII fixture remains ignored
  because the toolchain is not configured.
- `git diff --check` passed after the CE HIMCC-backed guideline payload work;
  Git still reports LF-to-CRLF conversion warnings for the dirty worktree files.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: `ImmUnlockIMC` now parses
  resized CE `COMPOSITIONSTRING` and `CANDIDATEINFO` HIMCC buffers assigned
  through `INPUTCONTEXT::hCompStr`/`hCandInfo`, updating the stored composition
  string and candidate lists so later `ImmGetCompositionStringW`,
  `ImmGetCandidateListCountW`, `ImmGetCandidateListW`, and `ImmNotifyIME`
  selection/page mutations operate on the guest-written state.
- `cargo fmt` and
  `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`
  passed after adding the CE resized composition/candidate IMCC mutation
  regression.
- `cargo fmt --check` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the CE
  resized composition/candidate IMCC mutation work; the eVC4 MIPSII fixture
  remains ignored because the toolchain is not configured.
- `git diff --check` passed after the CE resized composition/candidate IMCC
  mutation work; Git still reports LF-to-CRLF conversion warnings for the dirty
  worktree files.
- `src/ce/gwe.rs`, `src/ce/coredll.rs`, `tests/basic_subsystems.rs`,
  `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and
  `SOURCE_REFERENCES.md`: keyboard layout state now tracks the loaded HKL list
  instead of only the active HKL, `LoadKeyboardLayoutW` honors CE
  `KLF_ACTIVATE`, and `ActivateKeyboardLayout` supports `HKL_NEXT`/`HKL_PREV`
  cycling while continuing to post `WM_INPUTLANGCHANGE` to the focused window.
- `cargo fmt`,
  `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`, and
  `cargo test gwe_keyboard_layout_ime_and_activate_layout --test basic_subsystems --features unicorn,trace,win32-desktop -j 1`
  passed after adding the CE keyboard-layout transition regression.
- `cargo fmt --check` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the CE
  keyboard-layout transition work; the eVC4 MIPSII fixture remains ignored
  because the toolchain is not configured.
- `git diff --check` passed after the CE keyboard-layout transition work; Git
  still reports LF-to-CRLF conversion warnings for the dirty worktree files.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: synthetic IME HKLs now mirror
  TESTIME `dic.c`'s built-in single-letter candidate table when
  `ImmSetCompositionStringW(..., SCS_SETSTR, ...)` receives an ASCII letter,
  generating a slot-0 `IME_CAND_READ` list with the same/toggled-case pair and
  clearing that generated list when the composition is not in the table.
- `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`
  passed after adding the TESTIME dictionary-backed candidate regression.
- `cargo fmt --check` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  TESTIME dictionary-backed candidate work; the eVC4 MIPSII fixture remains
  ignored because the toolchain is not configured.
- `git diff --check` passed after the TESTIME dictionary-backed candidate work;
  Git still reports LF-to-CRLF conversion warnings for the dirty worktree files.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: raw `ImmGenerateMessage` now
  mirrors TESTIME `GenerateMessage`/`hMsgBuf` delivery by reading
  `INPUTCONTEXT::dwNumMsgBuf` three-DWORD message records, posting them to the
  HIMC owner window, clearing the count, and preserving the message-buffer
  HIMCC handle for later locks.
- `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`
  passed after adding the TESTIME `ImmGenerateMessage` message-buffer
  regression.
- `cargo fmt --check` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  TESTIME `ImmGenerateMessage` work; the eVC4 MIPSII fixture remains ignored
  because the toolchain is not configured.
- `git diff --check` passed after the TESTIME `ImmGenerateMessage` work; Git
  still reports LF-to-CRLF conversion warnings for the dirty worktree files.
- `src/ce/gwe.rs`, `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: TESTIME
  registered-word support now exposes `NOUN`/`VERB` style descriptors for IME
  HKLs, accepts `ImmRegisterWordW`/`ImmUnregisterWordW` for those fake styles,
  stores the registered reading/string pairs, and appends them to generated
  `ImmSetCompositionStringW(..., SCS_SETSTR, ...)` candidate lists before
  clearing them again after unregistering.
- `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`
  passed after adding the TESTIME registered-word/private-profile candidate
  regression.
- `cargo fmt --check` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  TESTIME registered-word/private-profile candidate work; the eVC4 MIPSII
  fixture remains ignored because the toolchain is not configured.
- `git diff --check` passed after the TESTIME registered-word/private-profile
  candidate work; Git still reports LF-to-CRLF conversion warnings for the
  dirty worktree files.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: raw `ImmGetConversionListW`
  now mirrors TESTIME `ImeConversionList` by returning a clean zero-byte result
  for conversion, reverse-conversion, and reverse-length requests instead of
  reporting the ordinal as unsupported.
- `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`
  passed after adding the TESTIME `ImmGetConversionListW` zero-result
  regression.
- `cargo fmt --check` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  TESTIME `ImmGetConversionListW` work; the eVC4 MIPSII fixture remains ignored
  because the toolchain is not configured.
- `git diff --check` passed after the TESTIME `ImmGetConversionListW` work; Git
  still reports LF-to-CRLF conversion warnings for the dirty worktree files.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: raw `ImmEscapeW` now mirrors
  TESTIME `ImeEscape` for `IME_ESC_QUERY_SUPPORT`, returning false for non-IME
  HKLs, null `lpData`, or unsupported requested escapes, and true only for
  IME HKL self-query support requests.
- `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`
  passed after adding the TESTIME `ImmEscapeW(IME_ESC_QUERY_SUPPORT)`
  regression.
- `cargo fmt --check` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  TESTIME `ImmEscapeW(IME_ESC_QUERY_SUPPORT)` work; the eVC4 MIPSII fixture
  remains ignored because the toolchain is not configured.
- `git diff --check` passed after the TESTIME
  `ImmEscapeW(IME_ESC_QUERY_SUPPORT)` work; Git still reports LF-to-CRLF
  conversion warnings for the dirty worktree files.
- `src/ce/gwe.rs`, `src/ce/coredll.rs`, `src/emulator/unicorn.rs`,
  `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and
  `SOURCE_REFERENCES.md`: `ImmEnumRegisterWordW` now mirrors TESTIME
  `ImeEnumRegisterWord` for Unicorn guest callbacks by issuing the initial
  filter probe callback, then enumerating registered private-profile words for
  non-null readings when the requested style is zero or `FAKEWORD_NOUN`; raw
  dispatch now returns the clean zero fallback instead of unsupported when no
  guest callback can be invoked.
- `cargo check --features unicorn,trace,win32-desktop -j 1` and
  `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`
  passed after the TESTIME `ImmEnumRegisterWordW` callback-enumeration work.
- `cargo fmt --check` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  TESTIME `ImmEnumRegisterWordW` callback-enumeration work; the eVC4 MIPSII
  fixture remains ignored because the toolchain is not configured.
- `git diff --check` passed after the TESTIME `ImmEnumRegisterWordW`
  callback-enumeration work; Git still reports LF-to-CRLF conversion warnings
  for the dirty worktree files.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: TESTIME candidate generation now
  mirrors `dic.c::ConvKanji` by rejecting combined built-in/registered-word
  candidate lists above the CE `MAXCANDSTRNUM` ceiling of 32 instead of
  returning an oversized `CANDIDATELIST`.
- `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`
  passed after adding the TESTIME oversized-candidate rejection regression.
- `cargo fmt --check` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  TESTIME oversized-candidate rejection work; the eVC4 MIPSII fixture remains
  ignored because the toolchain is not configured.
- `git diff --check` passed after the TESTIME oversized-candidate rejection
  work; Git still reports LF-to-CRLF conversion warnings for the dirty worktree
  files.
- `src/ce/gwe.rs`, `src/emulator/unicorn.rs`, `tests/coredll_raw_gwe.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: TESTIME
  registered-word candidate lookup and Unicorn `ImmEnumRegisterWordW`
  enumeration now mirror `stub.c::GetPrivateProfileString` by hiding
  private-profile entries for readings with lowercase characters while keeping
  uppercase readings visible.
- `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`
  passed after adding the TESTIME private-profile uppercase-section visibility
  regression.
- `tests/coredll_raw_gwe.rs`: the nonzero cross-thread
  `SendMessageTimeout` regression now verifies the actual queued send ID for
  the target window instead of assuming the incidental transaction ID is always
  `1`, removing an order-sensitive full-suite failure.
- `cargo fmt --check`, `cargo check --features unicorn,trace,win32-desktop -j 1`,
  `cargo test coredll_raw_send_message_timeout_nonzero_cross_thread_queues_transaction --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after
  the TESTIME private-profile visibility and SendMessageTimeout test-hardening
  work; the eVC4 MIPSII fixture remains ignored because the toolchain is not
  configured.
- `git diff --check` passed after the TESTIME private-profile visibility and
  SendMessageTimeout test-hardening work; Git still reports LF-to-CRLF
  conversion warnings for the dirty worktree files.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: TESTIME candidate generation now
  mirrors the active `stub.c::GetPrivateProfileString` dictionary read by
  appending registry-value private-profile entries under
  `HKLM\SOFTWARE\Microsoft\testime\Windows\testime.DIC\<reading>` after the
  built-in candidate table, while still applying the lowercase-section guard.
- `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`
  passed after adding the TESTIME registry-backed private-profile candidate
  regression.
- `cargo fmt --check`, `cargo check --features unicorn,trace,win32-desktop -j 1`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after
  the TESTIME registry-backed private-profile candidate work; the eVC4 MIPSII
  fixture remains ignored because the toolchain is not configured.
- `git diff --check` passed after the TESTIME registry-backed private-profile
  candidate work; Git still reports LF-to-CRLF conversion warnings for the
  dirty worktree files.
- `src/ce/kernel.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: kernel boot now reconciles the
  bundled TESTIME `testime.reg` sample dictionary with the active
  `stub.c::GetPrivateProfileString` registry-value enumeration path by seeding
  stable registry values whose string data preserves the sample candidates,
  including the numeric rolling candidate entries.
- `cargo test coredll_raw_keyboard_layout_and_imm_context_are_stateful --test coredll_raw_gwe --features unicorn,trace,win32-desktop -j 1`
  passed after adding default TESTIME sample-dictionary candidate coverage.
- `cargo fmt --check`, `cargo check --features unicorn,trace,win32-desktop -j 1`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after
  the TESTIME sample-dictionary seed work; the eVC4 MIPSII fixture remains
  ignored because the toolchain is not configured.
- `git diff --check` passed after the TESTIME sample-dictionary seed work; Git
  still reports LF-to-CRLF conversion warnings for the dirty worktree files.
- `src/ce/kernel.rs`, `src/ce/coredll.rs`, `src/emulator/imports.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: FSDMGR
  notification imports now distinguish CE `FSINT_*` internal notification
  handles from caller-owned `FSEXT_*` handles. `FSINT_FindFirstChangeNotificationW`
  creates an internal FSDMGR-owned handle, public/`FSEXT` reset-info rejects it
  without touching caller output pointers, and paired `FSINT` reset/info/close
  continues to operate on it, matching the `pathapi.cpp` and `fsnotify.cpp`
  internal-vs-external split.
- `cargo test emulator::imports::tests::fsdmgr --lib --features unicorn,trace,win32-desktop -j 1`
  and
  `cargo test --test coredll_raw_memory_file --features unicorn,trace,win32-desktop -j 1`
  passed after the FSDMGR internal notification-owner work.
- `cargo fmt --check`, `cargo check --features unicorn,trace,win32-desktop -j 1`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after
  the FSDMGR internal notification-owner work; the eVC4 MIPSII fixture remains
  ignored because the toolchain is not configured.
- `git diff --check` passed after the FSDMGR internal notification-owner work;
  Git still reports LF-to-CRLF conversion warnings for the dirty worktree
  files.
- `tests/coredll_raw_kernel.rs`, `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and
  `SOURCE_REFERENCES.md`: raw `ExtractIconExW` coverage now includes a 16bpp
  `BI_BITFIELDS`/RGB555 `RT_ICON` fixture, verifying that extracted PE icon
  color bitmaps preserve the DIB masks and render red correctly into the RGB565
  framebuffer.
- `cargo fmt --check` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel sh_get_file_info_uses_registry_associations_and_attributes -- --nocapture`
  passed after adding the `BI_BITFIELDS` PE icon coverage.
- `cargo check --features unicorn,trace,win32-desktop -j 1` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` also passed after
  the `BI_BITFIELDS` PE icon coverage; the eVC4 MIPSII fixture remains ignored
  because the toolchain is not configured.
- `tests/coredll_raw_kernel.rs`, `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and
  `SOURCE_REFERENCES.md`: the same PE `BI_BITFIELDS` extraction path now asserts
  that `DestroyIcon` releases PE-extracted owned color/mask bitmap handles and
  their heap storage.
- `cargo fmt --check` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel sh_get_file_info_uses_registry_associations_and_attributes -- --nocapture`
  passed after adding the PE-extracted `DestroyIcon` cleanup assertion.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel`
  passed after adding the PE-extracted `DestroyIcon` cleanup assertion.
- `cargo check --features unicorn,trace,win32-desktop -j 1`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and `git diff --check`
  passed after the PE-extracted `DestroyIcon` cleanup assertion; `git diff --check`
  output remains limited to existing LF-to-CRLF warnings, and the eVC4 MIPSII
  fixture remains ignored because that toolchain is not configured.
- `tests/coredll_raw_kernel.rs`, `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and
  `SOURCE_REFERENCES.md`: raw `ExtractIconExW` non-PE fallback coverage now
  proves the CE shell-icon fallback only fills the index-zero synthetic icon,
  leaves additional requested output-array slots untouched, and leaves
  large/small output pointers untouched for nonzero-index misses.
- `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel sh_get_file_info_uses_registry_associations_and_attributes -- --nocapture`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel`,
  `cargo check --features unicorn,trace,win32-desktop -j 1`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  non-PE `ExtractIconExW` fallback preservation regression; the eVC4 MIPSII
  fixture remains ignored because that toolchain is not configured.
- `tests/coredll_raw_kernel.rs`, `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and
  `SOURCE_REFERENCES.md`: raw `ExtractIconExW` PE coverage now includes a
  stride-padded 24bpp `BI_RGB` `RT_ICON` fixture, verifying extracted bitmap
  metadata, trailing AND-mask creation, and RGB565 framebuffer rendering of both
  padded BGR source pixels.
- `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel sh_get_file_info_uses_registry_associations_and_attributes -- --nocapture`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel`,
  `cargo check --features unicorn,trace,win32-desktop -j 1`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  24bpp PE icon coverage; the eVC4 MIPSII fixture remains ignored because that
  toolchain is not configured.
- `tests/coredll_raw_kernel.rs`, `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and
  `SOURCE_REFERENCES.md`: raw `ExtractIconExW` PE coverage now includes a 1bpp
  indexed `RT_ICON` fixture, verifying palette preservation, high-bit pixel
  decoding, and RGB565 framebuffer rendering for both palette indexes.
- `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel sh_get_file_info_uses_registry_associations_and_attributes -- --nocapture`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel`,
  `cargo check --features unicorn,trace,win32-desktop -j 1`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  1bpp PE icon coverage; the eVC4 MIPSII fixture remains ignored because that
  toolchain is not configured.
- `tests/coredll_raw_kernel.rs`, `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and
  `SOURCE_REFERENCES.md`: raw `ExtractIconExW` PE coverage now includes a
  masked stride-padded 24bpp `BI_RGB` `RT_ICON` fixture, verifying the trailing
  AND-mask byte becomes a 1bpp mask bitmap and `DrawIconEx(DI_NORMAL)` leaves
  the masked framebuffer destination pixel untouched while painting the
  unmasked color pixel.
- `src/emulator/unicorn.rs` and `src/main.rs`: immutable publish/snapshot paths
  now call the read-only parked-process duplicate pruning helper instead of the
  mutating trace-recording helper, restoring full-feature compilation while
  keeping the mutating helper available for callers that can pass `&mut
  CeKernel`.
- `cargo fmt`, `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel sh_get_file_info_uses_registry_associations_and_attributes -- --nocapture`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel`,
  `cargo check --features unicorn,trace,win32-desktop -j 1`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  masked 24bpp PE icon coverage and read-only prune call-site fix; the eVC4
  MIPSII fixture remains ignored because that toolchain is not configured.
- `tests/coredll_raw_kernel.rs`, `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and
  `SOURCE_REFERENCES.md`: raw shell system image-list pseudo-icon coverage now
  verifies selected-memory-DIB rendering, including untouched outside pixels,
  a painted pseudo body, and a visibly distinct valid overlay marker. This
  complements the existing framebuffer and invalid-overlay-slot coverage for
  synthetic shell/system image-list icons.
- `cargo fmt`, `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel sh_get_file_info_system_image_list_supports_icon_queries_and_draw -- --nocapture`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel`,
  `cargo check --features unicorn,trace,win32-desktop -j 1`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  system image-list pseudo-icon selected-DIB coverage; the eVC4 MIPSII fixture
  remains ignored because that toolchain is not configured.
- `src/ce/object.rs`, `src/ce/kernel.rs`, `src/ce/coredll.rs`,
  `tests/coredll_raw_memory_file.rs`, `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`,
  and `SOURCE_REFERENCES.md`: raw AFS volume handles now model the CE
  `HT_AFSVOLUME` shape enough for mounted roots. `AFS_Unmount(hAFS)` rejects
  foreign-process volume handles, successful unmount consumes the handle,
  direct `CloseHandle(hAFS)` also unmounts once, and both paths signal root and
  subpath file-change watchers through the mounted-root removal notifier.
- `cargo fmt`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_afs_unmount_volume_handle_signals_and_enforces_owner -- --nocapture`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_close_handle_on_volume_handle_unmounts_volume_once -- --nocapture`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`
  passed after the AFS volume-handle unmount/close work.
- `src/ce/kernel.rs`, `src/ce/coredll.rs`,
  `tests/coredll_raw_memory_file.rs`, `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`,
  and `SOURCE_REFERENCES.md`: raw `AFS_FsIoControlW` now resolves nonzero AFS
  volume handles through the owner-checked mounted root. `FSCTL_GET_VOLUME_INFO`
  reports the mounted volume's `CE_VOLUME_INFO` instead of falling back to the
  object store, while foreign-process volume-handle FSCTL attempts fail without
  touching caller output buffers.
- `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_memory_file coredll_raw_memory_and_file_ordinals_use_virtual_ce_heap_and_guest_buffers -- --nocapture`
  passed after the volume-handle `AFS_FsIoControlW(FSCTL_GET_VOLUME_INFO)`
  update.
- `src/ce/coredll.rs`, `src/emulator/imports.rs`, `PLAN.MD`,
  `PROGRESS.md`, and `SOURCE_REFERENCES.md`: `FSDMGR_DiskIoControl @12`
  now handles CE `IOCTL_DISK_COPY_EXTERNAL_COMPLETE` with the same
  `DISK_COPY_EXTERNAL` fixed-header and trailing `SECTOR_LIST_ENTRY`
  validation as `IOCTL_DISK_COPY_EXTERNAL_START`, then returns
  `ERROR_NOT_SUPPORTED` without mutating caller buffers for the host-backed
  synthetic disk. Real external-copy accelerator/filter behavior remains
  queued with the physical block-driver backing gaps.
- `src/ce/coredll.rs`, `src/emulator/imports.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: FSDMGR imports now recognize
  CE `STOREMGR_FsIoControlW @44` by ordinal/name and dispatch its
  `hProcess, path, Fsctl, ...` signature through the same mounted-volume
  FSCTL path used by `CeFsIoControlW`. The raw import regression now reaches
  mounted `FSCTL_GET_VOLUME_INFO` metadata through an actual `fsdmgr.dll`
  import trap, while the local non-volume-info fallback documents CE's
  `FsIoControl` forwarding/`FSStub_Bool` unsupported behavior.
- `cargo fmt`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop emulator::imports::tests::patches_supported_fsdmgr_imports_only -- --nocapture`,
  and
  `cargo test -j 1 --features unicorn,trace,win32-desktop emulator::imports::tests::fsdmgr_storemgr_fs_io_control_import_dispatches_to_mounted_volume_info -- --nocapture`
  passed after the FSDMGR `STOREMGR_FsIoControlW` import work.
- `cargo fmt --check`,
  `cargo check --features unicorn,trace,win32-desktop -j 1`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed after the FSDMGR `STOREMGR_FsIoControlW` import
  work; `git diff --check` reported only the existing LF-to-CRLF warnings.
- `src/ce/file.rs`, `src/ce/coredll.rs`, `src/emulator/imports.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: FSDMGR
  imports now recognize CE `FSDMGR_GetVolumeName @22` and
  `FSDMGR_GetMountFlags @37` by ordinal/name. Existing mounted HVOL handles now
  expose their mounted folder name through `GetVolumeName` with CE
  NUL-inclusive buffer sizing and expose CE AFS hidden/system/permanent mount
  flags through `GetMountFlags`; the remaining gap is still full
  `FSDMGR_RegisterVolume`/disk-handle mapping and broader non-volume-info
  FSCTL forwarding.
- `src/emulator/unicorn.rs`: fixed the old-MIPS encoded-exit regression test so
  its non-process-handle case uses the current-thread pseudo-handle instead of
  `0x42`, which is this repo's current-process pseudo-handle.
- `cargo fmt`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop emulator::imports::tests::patches_supported_fsdmgr_imports_only -- --nocapture`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop emulator::imports::tests::fsdmgr_mount_table_imports_query_volume_name_and_flags -- --nocapture`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop emulator::unicorn::unicorn_tests::old_mips_api_thunk_with_non_process_handle_is_not_process_exit -- --nocapture`,
  `cargo fmt --check`,
  `cargo check --features unicorn,trace,win32-desktop -j 1`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed after the FSDMGR mount-table query import and
  old-MIPS test-harness correction; the eVC4 MIPSII fixture remains ignored
  because that toolchain is not configured, and `git diff --check` reported
  only the existing LF-to-CRLF warnings.
- `src/ce/object.rs`, `src/ce/kernel.rs`, `src/ce/file.rs`,
  `src/ce/coredll.rs`, `src/emulator/imports.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: FSDMGR imports now recognize
  CE `FSDMGR_DeregisterVolume @10`, `FSDMGR_GetVolumeHandle @21`, and
  `FSDMGR_RegisterVolume @27`. The host-backed model registers/resolves a CE
  mount folder name, creates an HVOL carrying the FSD `PDSK` token and optional
  FSD volume context, maps that same disk token back through
  `FSDMGR_GetVolumeHandle`, and keeps `FSDMGR_DeregisterVolume` as the CE 6
  no-op. Remaining storage gaps are real block-driver/cache/filter support
  functions and broader non-volume-info FSCTL forwarding.
- `cargo fmt`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop emulator::imports::tests::fsdmgr_register_volume_maps_disk_pointer_to_volume_handle -- --nocapture`,
  and
  `cargo test -j 1 --features unicorn,trace,win32-desktop emulator::imports::tests::patches_supported_fsdmgr_imports_only -- --nocapture`
  passed after the FSDMGR register/get-volume-handle support-function work.
- `cargo fmt --check`,
  `cargo check --features unicorn,trace,win32-desktop -j 1`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  FSDMGR register/get-volume-handle support-function work; the eVC4 MIPSII
  fixture remains ignored because that toolchain is not configured.
- `src/ce/coredll.rs`, `src/emulator/imports.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: FSDMGR imports now also
  recognize CE `FSDMGR_CreateFileHandle @7` and
  `FSDMGR_CreateSearchHandle @8`. The raw traps follow
  `fsdmgrapi.cpp` by returning the FSD-supplied file/search context pointer as
  the handle while ignoring the HVOL and originating process handle. Remaining
  storage gaps are still real block-driver/cache/filter support functions and
  broader non-volume-info FSCTL forwarding.
- `cargo fmt`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop emulator::imports::tests::fsdmgr_create_file_and_search_handle_return_fsd_context_pointer -- --nocapture`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop emulator::imports::tests::patches_supported_fsdmgr_imports_only -- --nocapture`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop emulator::unicorn::unicorn_tests::wall_clock_restart_pc_restarts_mips_trampoline_before_delay_slot -- --nocapture`,
  `cargo fmt --check`,
  `cargo check --features unicorn,trace,win32-desktop -j 1`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  FSDMGR create-file/search-handle import work; the eVC4 MIPSII fixture remains
  ignored because that toolchain is not configured.
- `git diff --check` passed after the FSDMGR create-file/search-handle import
  work, reporting only the existing LF-to-CRLF working-copy warnings.
- `src/ce/kernel.rs`, `src/ce/coredll.rs`, `src/emulator/imports.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: FSDMGR
  imports now recognize the CE null-cache surface:
  `FSDMGR_CacheIoControl @3`, `FSDMGR_CachedRead @4`,
  `FSDMGR_CachedWrite @5`, `FSDMGR_CreateCache @6`,
  `FSDMGR_DeleteCache @9`, `FSDMGR_FlushCache @14`,
  `FSDMGR_InvalidateCache @24`, `FSDMGR_ResizeCache @30`, and
  `FSDMGR_SyncCache @32`. The host-backed model now tracks CE null-cache IDs,
  reuses deleted slots, returns `ERROR_INVALID_PARAMETER` for invalid
  delete/read/write/flush/cache-IOCTL ids, treats resize/sync/invalidate as CE
  null-cache no-ops, reports cached read/write as `ERROR_NOT_SUPPORTED` until
  real block-driver forwarding exists, and handles
  `IOCTL_DISK_DELETE_SECTORS` as null-cache success after unsupported device
  forwarding.
- `cargo fmt`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop emulator::imports::tests::fsdmgr_null_cache_imports_track_ids_and_nullcache_results -- --nocapture`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop emulator::imports::tests::patches_supported_fsdmgr_imports_only -- --nocapture`,
  `cargo fmt --check`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  FSDMGR null-cache import work; the eVC4 MIPSII fixture remains ignored
  because that toolchain is not configured.
- `src/ce/kernel.rs`, `src/ce/coredll.rs`, `src/emulator/imports.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: FSDMGR
  imports now also recognize CE `FSDMGR_DiskIoControl @12`,
  `FSDMGR_ReadDisk @25`, and `FSDMGR_WriteDisk @35`. The host-backed model
  persists direct and cache-sector writes in sparse synthetic 512-byte sectors,
  returns zero-filled bytes for unwritten sectors, writes a CE-shaped
  six-DWORD `DISK_INFO`, handles SG read/write requests through the direct
  disk IOCTL path, and treats delete-sector/flush-cache as successful basic
  disk IOCTLs. Cache write buffer sizing now uses the created cache block size
  instead of assuming 512 bytes at the guest boundary.
- `cargo fmt`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop fsdmgr_disk_support_imports_round_trip_sparse_sectors_and_info -- --nocapture`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop fsdmgr_null_cache_imports_track_ids_and_nullcache_results -- --nocapture`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop patches_supported_fsdmgr_imports_only -- --nocapture`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`,
  `cargo fmt --check`, and `git diff --check` passed after the direct FSDMGR
  disk read/write and disk-IOCTL work; the eVC4 MIPSII fixture remains ignored
  because that toolchain is not configured, and `git diff --check` reported
  only the existing LF-to-CRLF warnings.
- `src/ce/coredll.rs`, `src/emulator/imports.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: FSDMGR imports now recognize
  CE `FSDMGR_ReadDiskEx @26` and `FSDMGR_WriteDiskEx @36` by ordinal/name.
  The raw traps parse guest `FSD_SCATTER_GATHER_INFO`, scatter/gather the
  requested sector payload across caller `FSD_BUFFER_INFO` arrays, persist the
  transfer in the same sparse synthetic disk sectors as direct/cache I/O, and
  write `FSD_SCATTER_GATHER_RESULTS` flags plus transferred-sector counts.
  Remaining storage gaps are physical block-driver backing, external cache
  DLL/filter behavior, richer disk IOCTLs, and broader non-volume-info FSCTL
  forwarding.
- `cargo fmt`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop fsdmgr_disk_ex_imports_scatter_gather_sparse_sectors -- --nocapture`,
  and
  `cargo test -j 1 --features unicorn,trace,win32-desktop patches_supported_fsdmgr_imports_only -- --nocapture`
  passed after the FSDMGR `ReadDiskEx`/`WriteDiskEx` import work.
- `src/ce/kernel.rs`, `src/ce/coredll.rs`, `src/emulator/imports.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`:
  `FSDMGR_DiskIoControl @12` now treats CE
  `IOCTL_DISK_DELETE_SECTORS` as a real sparse-disk mutation. The raw shim
  validates the 12-byte `DELETE_SECTOR_INFO` payload, reads
  `startsector`/`numsectors`, clears matching synthetic sectors, and later
  reads return zero-filled data for the deleted range. Remaining storage gaps
  are physical block-driver backing, external cache DLL/filter behavior,
  remaining disk IOCTLs, and broader non-volume-info FSCTL forwarding.
- `cargo fmt`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop fsdmgr_disk_support_imports_round_trip_sparse_sectors_and_info -- --nocapture`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`,
  `cargo fmt --check`, and `git diff --check` passed after the
  `IOCTL_DISK_DELETE_SECTORS` sparse-sector clearing work; the eVC4 MIPSII
  fixture remains ignored because that toolchain is not configured, and
  `git diff --check` reported only the existing LF-to-CRLF warnings.
- `src/ce/kernel.rs`, `src/ce/coredll.rs`, `src/emulator/imports.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`:
  `FSDMGR_DiskIoControl @12` now covers more CE `diskio.h` metadata/status
  paths against the synthetic disk backing. The raw trap returns a CE-shaped
  disk name, `STORAGEDEVICEINFO`, and invalid-manufacturer/serial
  `STORAGE_IDENTIFICATION`, accepts `DISK_IOCTL_SETINFO`/`IOCTL_DISK_SETINFO`
  buffers as validated no-ops, treats `DISK_IOCTL_INITIALIZED`/
  `IOCTL_DISK_INITIALIZED` as success, and clears all synthetic sectors for
  `DISK_IOCTL_FORMAT_MEDIA`/`IOCTL_DISK_FORMAT_MEDIA`. Remaining storage gaps
  are physical block-driver backing, external cache DLL/filter behavior,
  remaining specialized disk IOCTLs, and broader non-volume-info FSCTL
  forwarding.
- `cargo fmt`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop fsdmgr_disk_support_imports_round_trip_sparse_sectors_and_info -- --nocapture`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after
  the FSDMGR disk metadata/status IOCTL work; the eVC4 MIPSII fixture remains
  ignored because that toolchain is not configured.
- `src/emulator/imports.rs`, `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and
  `SOURCE_REFERENCES.md`: the FSDMGR `STOREMGR_FsIoControlW @44` import
  regression now covers the non-volume-info edges that our host-backed FSD can
  model today. `FSCTL_REFRESH_VOLUME` and `FSCTL_FLUSH_BUFFERS` are successful
  no-ops with zero returned bytes, while an unknown FSCTL follows the CE
  `FSStub_Bool` unsupported shape by returning false, setting
  `ERROR_NOT_SUPPORTED`, and leaving caller buffers/counts untouched. Remaining
  FSCTL fidelity is real mounted-FSD `FsIoControl` hook forwarding beyond the
  host-backed unsupported stub.
- `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop fsdmgr_storemgr_fs_io_control_import_dispatches_to_mounted_volume_info -- --nocapture`
  passed after the FSDMGR `STOREMGR_FsIoControlW` refresh/flush/unsupported
  FSCTL import coverage work.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  FSDMGR `STOREMGR_FsIoControlW` refresh/flush/unsupported FSCTL import
  coverage work; the eVC4 MIPSII fixture remains ignored because that
  toolchain is not configured.
- `src/ce/coredll.rs`, `src/emulator/imports.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: `FSDMGR_DiskIoControl @12`
  now has an explicit CE `IOCTL_DISK_GET_SECTOR_ADDR` path. The raw shim
  validates the CE/DOSPART buffer contract (`in`/`out` pointers present,
  matching nonzero DWORD-array byte counts, readable sector list) and then
  reports `ERROR_NOT_SUPPORTED` without touching the address list because the
  host-backed synthetic disk has no static XIP sector-address mapping.
- `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop fsdmgr_disk_support_imports_round_trip_sparse_sectors_and_info -- --nocapture`
  passed after the FSDMGR `GET_SECTOR_ADDR` validation/no-XIP unsupported
  import work.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  FSDMGR `GET_SECTOR_ADDR` validation/no-XIP unsupported import work; the eVC4
  MIPSII fixture remains ignored because that toolchain is not configured.
- `src/ce/coredll.rs`, `src/emulator/imports.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: `FSDMGR_DiskIoControl @12`
  now handles CE `IOCTL_DISK_GETPMTIMINGS` for the synthetic disk. The raw
  shim follows the ATAPI power path's in/out `PowerTimings` contract by
  requiring a writable 68-byte buffer whose `dwSize` covers the full structure,
  then returns a correctly sized all-zero timing snapshot rather than inventing
  hardware power-transition counters.
- `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop fsdmgr_disk_support_imports_round_trip_sparse_sectors_and_info -- --nocapture`
  passed after the FSDMGR `GETPMTIMINGS` synthetic timing work.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  FSDMGR `GETPMTIMINGS` synthetic timing work; the eVC4 MIPSII fixture remains
  ignored because that toolchain is not configured.
- `src/ce/coredll.rs`, `src/emulator/imports.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: `FSDMGR_DiskIoControl @12`
  now models CE secure-wipe disk IOCTLs against the sparse synthetic disk.
  `IOCTL_DISK_SET_SECURE_WIPE_FLAG` validates the 12-byte
  `DELETE_SECTOR_INFO` payload and succeeds without erasing data, while
  `IOCTL_DISK_SECURE_WIPE` validates the same payload and clears the requested
  sparse-sector range. This follows DOSPART's partition-to-delete-info wrapping
  and MSFLASH's payload validation while leaving hardware flash resume behavior
  as an open fidelity gap.
- `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop fsdmgr_disk_support_imports_round_trip_sparse_sectors_and_info -- --nocapture`
  passed after the FSDMGR secure-wipe disk IOCTL work.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  FSDMGR secure-wipe disk IOCTL work; the eVC4 MIPSII fixture remains ignored
  because that toolchain is not configured.
- `src/ce/coredll.rs`, `src/emulator/imports.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: `FSDMGR_DiskIoControl @12`
  now explicitly handles CE `IOCTL_DISK_COPY_EXTERNAL_START` enough for the
  host-backed synthetic disk model. The raw shim validates the CE
  `DISK_COPY_EXTERNAL` fixed header and trailing `SECTOR_LIST_ENTRY` array,
  then reports `ERROR_NOT_SUPPORTED` without mutating caller input or output
  buffers because there is no external copy accelerator behind the sparse disk.
  Real accelerator/filter behavior remains queued with the physical block-driver
  backing gaps.
- `cargo fmt`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop fsdmgr_disk_support_imports_round_trip_sparse_sectors_and_info -- --nocapture`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`,
  `cargo fmt --check`, and `git diff --check` passed after the FSDMGR
  copy-external-start validation work. `git diff --check` reported only the
  existing LF-to-CRLF warnings.
- `src/ce/coredll.rs`, `src/emulator/imports.rs`,
  `tests/coredll_raw_memory_file.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: `CeFsIoControlW`,
  `AFS_FsIoControlW`, and `STOREMGR_FsIoControlW @44` now explicitly handle
  CE `FSCTL_COPY_EXTERNAL_START` and `FSCTL_COPY_EXTERNAL_COMPLETE` for the
  host-backed FSD model. The raw helper validates the fixed 536-byte
  `FILE_COPY_EXTERNAL` payload, requires `cbSize` to cover the structure, and
  then returns `ERROR_NOT_SUPPORTED` without mutating caller buffers, matching
  the reviewed CE shape while leaving real external-copy acceleration as an
  open physical/FSD hook gap.
- `cargo fmt`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop coredll_raw_fs_io_control_refresh_and_flush_are_no_ops -- --nocapture`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop fsdmgr_storemgr_fs_io_control_import_dispatches_to_mounted_volume_info -- --nocapture`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after
  the filesystem copy-external FSCTL validation work.
- `src/ce/kernel.rs`, `src/ce/coredll.rs`,
  `tests/coredll_raw_memory_file.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: file-handle
  `DeviceIoControl(FSCTL_SET_FILE_CACHE)` now follows the reviewed CE
  cache-filter shape. `FileCacheDisableStandard` succeeds as a zero-byte
  no-op for host-backed file handles, enable/discard levels return
  `ERROR_NOT_SUPPORTED`, malformed input returns `ERROR_INVALID_PARAMETER`,
  closed handles return `ERROR_INVALID_HANDLE`, and volume-level
  `CeFsIoControlW(FSCTL_SET_FILE_CACHE)` remains unsupported. Broader external
  cache DLL/filter behavior stays queued with the remaining storage fidelity
  gaps.
- `cargo fmt`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop coredll_raw_file_handle_set_file_cache_follows_cache_filter_shape -- --nocapture`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  file-handle `FSCTL_SET_FILE_CACHE` work; the eVC4 MIPSII fixture remains
  ignored because that toolchain is not configured.
- `src/emulator/unicorn.rs`, `src/emulator/types.rs`, `SOURCE_REFERENCES.md`,
  `TODO.md`, and `KNOWN_BUGS.md`: direct SendMessage-owned WNDPROC cleanup now
  preserves pending live callouts until the saved/debug PC reaches the callout's
  return PC, avoiding loss of a live guest WNDPROC return when the GWE
  sent-message record has temporarily disappeared during iNavi/Happyway modal
  handoff diagnostics. WNDPROC return traces now include live/caller frame
  pointers, and the optional `WINCE_EMU_INAVI_GLOBAL_TRACE` hooks capture the
  object/thunk globals used while narrowing the animated-splash frontier.
- `cargo test -j 1 --features unicorn,trace,win32-desktop direct_send_wndproc_cleanup_keeps_live_callout_until_return_pc -- --nocapture`
  passed after the direct-send WNDPROC cleanup work.
- `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test basic_subsystems`
  passed after the direct-send WNDPROC cleanup work.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  direct-send WNDPROC cleanup work; the eVC4 MIPSII fixture remains ignored
  because that toolchain is not configured.
- `src/remote_server.rs`, `PLAN.MD`, `TODO.md`, and `KNOWN_BUGS.md`: the remote
  REST server now keeps a cloned listener guard in shared server state and waits
  for the accept thread to enter `serve` before `RemoteServer::start` prints and
  returns. This hardens the detached-launch listener lifetime regression that
  blocked unattended iNavi UI driving; a full detached host/remote launch recheck
  remains queued before treating REST screenshot/input control as reliable in
  the live app flow.
- `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop remote_server_start_keeps_listener_guard_and_accepts_immediately -- --nocapture`
  passed after the remote-server listener hardening.
- `cargo test -j 1 --features unicorn,trace,win32-desktop remote_server::tests -- --nocapture`
  passed after the remote-server listener hardening.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  remote-server listener hardening; the eVC4 MIPSII fixture remains ignored
  because that toolchain is not configured.
- Detached debug `drive94` iNavi launch on `127.0.0.1:8766` rechecked the
  remote-server listener hardening in the host process path:
  `cargo build --features unicorn` refreshed the debug binary, then
  `target\debug\wince_emulation_v3.exe --image D:\INAVI_Emulator\INAVI\INavi\iNavi.exe --dll-search-dir D:\INAVI_Emulator\DUMPPLZ\Windows --mount-config mounts.toml --desktop virtual --remote-server 127.0.0.1:8766 --run-cpu --remote-video-fps 5 --remote-jpeg-quality 80 --cpu-instruction-limit 8000000 --verbose`
  printed `remote server: http://127.0.0.1:8766`. Unlike prior `drive93`
  listener failures, `/api/v1/status` answered with `running: true`,
  `/api/v1/frame.jpg` wrote `target\drive94_frame.jpg`, and
  `/api/v1/input/touch` accepted a top-right tap. The remote input debug route
  showed the tap drained through the active route into `hwnd=0x00020008` as
  `WM_LBUTTONDOWN`/move-style mouse messages for iNavi's thread. The remaining
  frontier is not REST listener lifetime: `target\drive94_screenshot.png` was
  still black while iNavi had dirty visible windows and active thread work, so
  map/splash rendering remains open.
- `src/remote_server.rs`: `drive96` reproduced the optimized detached false-URL
  failure after the cloned-listener guard work: the release process printed
  `remote server: http://127.0.0.1:8768` but `/api/v1/status` timed out/refused
  while the process remained alive. The remote server now owns the actual
  `TcpListener` in shared `RemoteServerState`, the accept thread calls
  `accept()` on that state-owned listener, and startup self-probes
  `/api/v1/status` before printing the URL. This makes listener lifetime match
  the `RemoteServer` handle stored by the kernel instead of a moved listener
  clone.
- `cargo test -j 1 --features unicorn,trace,win32-desktop remote_server::tests -- --nocapture`
  passed after the state-owned remote listener and startup probe fix. The same
  test build also caught and fixed a test-only MIPS `reg_write` type mismatch in
  the direct-send WNDPROC regression.
- `cargo build --release --features unicorn,trace,win32-desktop` passed after
  stopping the stale detached release process that had locked
  `target\release\wince_emulation_v3.exe`.
- Detached release `drive97` on `127.0.0.1:8769` validated the fixed optimized
  host path: `/api/v1/status` returned `running: true`, `/api/v1/frame.jpg`
  wrote `target\drive97_frame.jpg` with the real iNavi SE splash art, and
  `/api/v1/input/touch` accepted a top-right tap. The remote input debug route
  showed the tap drained through the active iNavi window (`hwnd=0x00020008`) as
  posted mouse messages, and file traces had advanced through map/config/search
  data (`mapinfo.bin`, `config.bin`, `GpsPosition.bin`, search DB/MRData files).
  The remaining route-flow frontier is now post-splash map UI transition, not
  listener lifetime or black framebuffer output.
- `cargo test -j 1 --features unicorn,trace,win32-desktop` passed after the
  state-owned remote listener/startup-probe fix and the direct-send WNDPROC
  regression update. The eVC4 MIPSII fixture remains intentionally ignored.
- Live trace-mode iNavi drive on `192.168.0.39:8765` after commit `5d020dc`
  confirmed the current splash stall is still making guest progress: valid
  remote touch JSON is `{"type":"tap","x":N,"y":N}` and drains into the visible
  iNavi window, the real `happyway_win.exe` dialog is created by the guest and
  destroyed through the existing modal path, `iSearch.exe` remains parked in a
  normal `GetMessage` wait, and iNavi continues reading/decompressing
  `\SDMMC Disk\INavi\res\resmapi_800x480.bin`. Current PCs around
  `iNavi.exe+0x2ff8xx`/`+0x2ff9xx` disassemble as the byte-copy/decompression
  loop reached from `iNavi.exe+0x2ed4cc`, while render/controller milestones
  remain empty. In this run no `COM7:`, `MFS1:`, or `SMB1:` handles are open;
  posted GPS NMEA bytes stay queued, so the sensor symptom is lack of guest
  polling/opening during this startup phase rather than a failed REST enqueue.
- `src/ce/coredll.rs`: implemented the CE `compr2.c` raw/all-zero packet
  branches for `StringCompress`/`StringDecompress`. The new raw kernel
  regression covers UTF-16-style low-byte raw packet round-trip, all-zero
  packets, size-only compression queries, the CE non-shrinking raw
  `CECOMPRESS_FAILED` rule, and fail-closed behavior for still-unsupported
  opaque compressed stream payloads.
- `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel coredll_raw_string_compress_decompress_matches_ce_raw_packet_edges -- --nocapture`
  and `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_kernel`
  passed after the `StringCompress`/`StringDecompress` slice.
- `cargo check --features unicorn,trace,win32-desktop` and full
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed outside the
  sandbox after the CE string-compression wrapper slice; the sandboxed
  `cargo check` attempt had failed only because Ninja execution was blocked in
  the sandbox cwd.
- `src/ce/coredll.rs`, `src/ce/thread.rs`, and `tests/coredll_raw_gwe.rs`: raw
  `ExtEscape` now follows the CE display-driver `QUERYESCSUPPORT` and
  protected-escape surface from `pwingdi.h`, `dispperf.h`, `dc.cpp`,
  `ddi_if.cpp`, and `gpeflat.cpp`. The new regression covers supported gamma
  and display-performance support queries, unsupported raw-framebuffer queries,
  direct privileged screen-rotation escape denial with `ERROR_INVALID_ACCESS`,
  invalid HDCs, and invalid query buffers.
- `cargo fmt --check` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passed after the raw `ExtEscape` slice. The run also fixed the full-feature
  test compile break by retaining the single shared
  `blocked_get_message_has_visible_receiver_work` scheduler helper instead of a
  duplicate definition.
- The full-feature test suite initially exposed stale scheduler regression
  fixtures in `direct_send_wndproc_cleanup_keeps_live_callout_until_return_pc`
  and `escaped_get_message_sent_callout_completes_active_send`; the fixtures now
  preserve the live pending callout and restore the guarded `GetMessageW`
  import arguments. Both focused tests pass, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` now passes again
  with the eVC4 MIPSII fixture still intentionally ignored.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: the initial raw
  `ExtEscape` DISPPERF slice implemented the CE `DispPerfDrvEscape` payload
  contract for
  `DISPPERF_EXTESC_GETSIZE`, `GETTIMING`, `CLEARTIMING`, and `GETUNHANDLED`.
  `GETSIZE` reports the CE 32-entry, 64-byte `DISPPERF_TIMING` table size
  (2048 bytes), `GETTIMING` initially returned a zeroed table, `CLEARTIMING`
  succeeded as a no-op, and `GETUNHANDLED` reported zero unhandled operations
  while preserving CE invalid-output-buffer failures.
- `src/ce/linux_x11_desktop.rs` now parses and formats as the Linux X11 host
  desktop backend behind the existing `linux-x11-desktop` feature wiring; the
  key-symbol range patterns were normalized to valid Rust literal ranges so
  rustfmt can resolve the module.
- `Cargo.toml`: `win32-desktop` now enables `win32-audio`, matching the host
  desktop code path that registers the WinMM sink on Windows. This keeps the
  requested `unicorn,trace,win32-desktop` feature set buildable without asking
  callers to remember a separate audio feature.
- Validation after the DISPPERF payload slice: `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` all pass; the eVC4
  MIPSII fixture remains intentionally ignored.
- `tests/coredll_raw_kernel.rs`: expanded the CE `compr2.c`
  `StringCompress`/`StringDecompress` regression to cover high-byte-only raw
  packets (`STRPART2RAW` without `STRPART1RAW`), odd-length raw packets that
  decompress to CE's padded even byte count, and fail-closed behavior when only
  one stream is raw and the other would require the unavailable
  `CEDecompress` engine. A source search found declarations/callers for
  `CECompress`/`CEDecompress`, but not the engine body, so the opaque branch is
  still intentionally unsupported rather than guessed.
- `tests/coredll_raw_gwe.rs`: the raw `ExtEscape` regression now covers the
  CE GDIAPI `ExtEscapeInvalidAccess` list for direct get/set rotation, get/set
  video protection, save/restore video memory, and get/set gamma escapes, plus
  the CE query-supported get/set gamma, get/set rotation, and
  `DISPPERF_EXTESC_*` escape set. At that stage, the remaining display-driver
  gap was real payload execution and nonzero display-performance
  instrumentation.
- Validation after the expanded `ExtEscape` coverage:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_ext_escape_matches_ce_query_and_protected_escape_edges`,
  `cargo fmt --check`, `git diff --check`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe` all pass. The cargo runs still emit the existing unused-code
  warnings and a Windows incremental-cache cleanup warning.
- `src/ce/coredll.rs`, `src/ce/resource.rs`, and `tests/coredll_raw_gwe.rs`:
  raw palette APIs now follow the CE GDIAPI `pal.cpp` edges for invalid
  palette/HDC handles, `SetPaletteEntries` null-entry and zero-count
  `ERROR_INVALID_PARAMETER`, non-paletted `GetSystemPaletteEntries` returning
  zero entries on the RGB565 display, and `SelectPalette` accepting the stock
  `DEFAULT_PALETTE` handle when restoring a DC. Raw `GetNearestColor` now
  rejects invalid/null HDCs with `CLR_INVALID`/`ERROR_INVALID_HANDLE` while
  preserving direct true-color return behavior for valid display DCs. Raw
  `GetCurrentObject(OBJ_PAL)` now reports selected and restored stock palettes,
  rejects invalid/null HDCs with `ERROR_INVALID_HANDLE`, and rejects unknown
  object types on valid HDCs with `ERROR_INVALID_PARAMETER`. Raw
  `GetObjectType` now follows the CE `wingdi.h` object constants for screen DC,
  memory DC, and palettes while rejecting invalid handles with
  `ERROR_INVALID_HANDLE`. Raw `GetStockObject` now reports
  `ERROR_INVALID_PARAMETER` for unsupported stock indexes, covering the CE
  `GetStockObject(-1)` edge. Raw `SelectObject` now rejects invalid/null HDCs
  plus null or unknown GDI objects with `ERROR_INVALID_HANDLE` before mutating
  DC state. The existing user-palette entry round-trip and nearest-index
  coverage remains intact.
- Validation after the CE palette slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_palette_entries_round_trip_and_select`,
  `cargo fmt --check`, `git diff --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe` all pass. The cargo runs still emit the existing unused-code
  warnings and a Windows incremental-cache cleanup warning.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: stock
  `DEFAULT_PALETTE` query paths now match the CE GDIAPI helper flow in
  `global.cpp::myCreatePal`: `GetObjectW` reports the stock palette entry
  count, `GetPaletteEntries` can copy those entries, and
  `GetNearestPaletteIndex` treats the stock palette as a readable palette while
  palette mutation remains restricted to created palettes. Raw palette coverage
  also now mirrors `pal.cpp::GetPaletteEntriesTest`,
  `SetGetPaletteEntriesTest`, and `SimpleGetNearestPaletteIndexTest` with a
  256-entry `CreatePalette`/`SetPaletteEntries`/`GetPaletteEntries`/
  nearest-index flow.
- Validation after the stock/default and 256-entry palette slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe palette`, `cargo fmt --check`, `git diff --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe` all pass. The full raw GWE suite now has 292 passing tests;
  the cargo runs still emit the existing unused-code warnings and a Windows
  incremental-cache cleanup warning.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `GetDeviceCaps`
  now follows CE GDIAPI `dc.cpp`/`dcdata.h` edges for display-capability
  queries. Bad HDCs fail with `ERROR_INVALID_HANDLE`, invalid index `-1` fails
  with `ERROR_INVALID_PARAMETER`, a null HDC uses the primary display like CE,
  `DRIVERVERSION` returns a nonzero display-driver version, and
  `SIZEPALETTE`/`NUMRESERVED` report zero for the emulator's RGB565
  non-paletted display.
- Validation after the `GetDeviceCaps` parity slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_get_device_caps_returns_ce_display_capabilities`,
  `cargo fmt --check`, `git diff --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe` all pass. The cargo runs still emit the existing unused-code
  warnings and a Windows incremental-cache cleanup warning.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: the raw
  `GetDeviceCaps` regression now covers the rest of the CE GDIAPI
  `dcdata.h` display-capability sweep that is meaningful for the local primary
  display: unlimited brush/pen/marker/font counts, rectangular `CLIPCAPS`, and
  square-pixel `ASPECTX`/`ASPECTY`/`ASPECTXY` values. This keeps the primary
  display aligned with the CE `DeviceCapsGPERegTest` aspect-ratio indices while
  leaving real secondary-display registry reload behavior out of scope for the
  raw path.
- Validation after the expanded `GetDeviceCaps` cap-sweep slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_get_device_caps_returns_ce_display_capabilities`,
  `cargo fmt --check`, `git diff --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe` all pass. The cargo runs still emit the existing unused-code
  warnings and a Windows incremental-cache cleanup warning.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `SelectObject`
  now follows the CE GDIAPI `do.cpp::SelectObjectTest` region return path for
  simple regions. Selecting a region into a valid DC stores it as the tracked
  clip region and returns the existing CE region complexity value, so selecting
  the same `CreateRectRgn` handle twice reports `SIMPLEREGION` instead of the
  previous pen/brush/bitmap object selection.
- Validation after the CE region `SelectObject` slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_select_object_returns_restorable_dc_defaults`
  passed. `cargo fmt --check`, `git diff --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe` also pass. The cargo runs still emit the existing
  unused-code warnings and Windows profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `DeleteObject`
  now treats stock GDI handles as CE no-op successes. This matches
  `do.cpp::DeleteGetStockObjectTest`: deleting stock brush, pen, font, or
  palette handles must not destroy the stable stock object, and subsequent
  `GetStockObject`/`SelectObject` use of the brush and pen handles remains
  valid. The `DEFAULT_PALETTE` path is covered as a stock handle that survives
  deletion without using `SelectObject`.
- Validation after the CE stock `DeleteObject` slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_select_object_returns_restorable_dc_defaults`
  passed. `cargo fmt --check`, `git diff --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe` also pass. The cargo runs still emit the existing
  unused-code warnings and Windows profile/incremental-cache cleanup noise.
- `src/ce/resource.rs`, `src/ce/coredll.rs`, and `tests/coredll_raw_gwe.rs`:
  raw stock-object handling now includes the CE public `wingdi.h`
  `BORDERX_PEN` and `BORDERY_PEN` indexes. Both resolve to stable stock pen
  handles, report `OBJ_PEN` through `GetObjectType`, participate in
  `SelectObject` previous-pen restoration, and survive the stock
  `DeleteObject` no-op path.
- Validation after the CE border stock-pen slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_select_object_returns_restorable_dc_defaults`
  passed. `cargo fmt --check`, `git diff --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe` also pass. The cargo runs still emit the existing
  unused-code warnings and Windows profile/incremental-cache cleanup noise.
- `src/ce/resource.rs`, `src/ce/coredll.rs`, and `tests/coredll_raw_gwe.rs`:
  raw bitmap objects now distinguish `CreateDIBSection` handles from ordinary
  bitmaps. `GetObjectW` preserves the 24-byte `BITMAP` fallback for normal
  bitmaps and BITMAP-sized DIB-section queries, while DIBSECTION-sized
  DIB-section queries now return the CE `wingdi.h` 84-byte layout with
  `BITMAPINFOHEADER` width, signed top-down height, bit depth, compression,
  image size, color-table count, RGB masks, and null section/offset metadata.
  This follows CE GDIAPI `do.cpp::GetObjectDIBTest`.
- Validation after the CE DIBSECTION `GetObjectW` slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_gdi_create_bitmap_and_get_object_w` passed.
  `cargo fmt --check`, `git diff --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe` also pass. The full raw GWE suite remains at 292 passing
  tests; the cargo runs still emit the existing unused-code warnings and
  Windows profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `CreateDIBSection` now follows the CE-only 24 bpp `BI_BITFIELDS` behavior
  documented in GDIAPI `verify.cpp::myCreateRGBDIBSection` and exercised by
  `draw.cpp::CreateDIBSection24bppDIBTest`. For 24 bpp headers the parser now
  accepts `BI_BITFIELDS` without storing masks, and selected-DIB `SetPixel`
  writes plain BGR 24 bpp bytes for red, green, and blue.
- Validation after the CE 24 bpp `BI_BITFIELDS` DIB-section slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_create_dibsection_accepts_ce_24bpp_bitfields_as_bgr`
  passed. `cargo fmt --check`, `git diff --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe` also pass. The full raw GWE suite now has 293 passing tests;
  the cargo runs still emit the existing unused-code warnings and Windows
  profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `CreateDIBSection` now follows CE GDIAPI
  `draw.cpp::SimpleCreateDIBSectionTest2` by accepting a null `ppvBits`
  output pointer. The implementation still allocates owned DIB backing storage
  and still frees it if a non-null caller output pointer faults, while callers
  that pass `NULL` simply receive the bitmap handle.
- Validation after the CE null-`ppvBits` DIB-section slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_create_dibsection_accepts_null_bits_output`
  passed. `cargo fmt --check`, `git diff --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe` also pass. The full raw GWE suite now has 294 passing tests;
  the cargo runs still emit the existing unused-code warnings and Windows
  profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `CreateDIBSection` now follows CE GDIAPI
  `draw.cpp::CreateDIBSectionPalBadUsage` for `DIB_PAL_COLORS` usage. The
  implementation rejects non-indexed/high-bpp palette-index DIB-section
  requests with `ERROR_INVALID_PARAMETER` before allocating backing storage or
  touching `ppvBits`, while preserving the already-covered indexed
  `DIB_PAL_COLORS` path.
- Validation after the CE high-bpp `DIB_PAL_COLORS` DIB-section slice:
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_create_dibsection_rejects_high_bpp_pal_colors` passed. `cargo
  fmt --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe` also pass. The full raw
  GWE suite now has 295 passing tests; the cargo runs still emit the existing
  unused-code warnings and Windows profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `CreateDIBSection` now also follows CE GDIAPI
  `verify.cpp::myCreateDIBSection` for the paletted-HDC requirement. An 8 bpp
  `DIB_PAL_COLORS` DIB-section request with a null or invalid HDC now fails
  with `ERROR_INVALID_HANDLE` before writing `ppvBits`, while the same header
  still succeeds with a real DC and `DIB_RGB_COLORS` null-HDC callers remain
  unaffected.
- Validation after the CE null-HDC `DIB_PAL_COLORS` DIB-section slice:
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_create_dibsection_rejects_pal_colors_without_hdc` passed.
  `cargo fmt --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe` also pass. The full raw
  GWE suite now has 296 passing tests; the cargo runs still emit the existing
  unused-code warnings and Windows profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `CreateDIBSection` now enforces another CE
  `draw.cpp::CreateDIBSectionPalBadUsage` matrix edge by rejecting indexed
  `DIB_RGB_COLORS` sections when `biClrUsed > 256` before allocating backing
  storage or writing `ppvBits`. The CE indexed `BI_RGB`/`DIB_PAL_COLORS`
  exception remains accepted with a valid HDC.
- Validation after the CE oversized indexed RGB `biClrUsed` DIB-section slice:
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_create_dibsection_rejects_oversized_indexed_rgb_color_count`
  passed.
  `cargo fmt --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe` also pass. The full raw
  GWE suite now has 297 passing tests; the cargo runs still emit the existing
  unused-code warnings and Windows profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `CreateDIBSection` now follows the CE
  `draw.cpp::CreateDIBSectionPalBadUsage` bit-depth matrix by accepting only
  `1`, `2`, `4`, `8`, `16`, `24`, and `32` bpp section requests. Unsupported
  depths such as `3`, `5`, `15`, and `33` now fail with
  `ERROR_INVALID_PARAMETER` before allocating storage or writing `ppvBits`.
- Validation after the CE unsupported-bit-depth DIB-section slice:
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_create_dibsection_rejects_unsupported_bit_depths` passed.
  `cargo fmt --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe` also pass. The full raw
  GWE suite now has 298 passing tests; the cargo runs still emit the existing
  unused-code warnings and Windows profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: the same supported DIB
  bit-depth predicate is now used for direct caller-DIB parsing. Raw
  `StretchDIBits` and `SetDIBitsToDevice` reject unsupported source DIB depths
  before rendering, preserving selected-DIB destination pixels on failure.
- Validation after the direct-DIB unsupported-bit-depth slice:
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_direct_dib_calls_reject_unsupported_bit_depths` passed.
  `cargo fmt --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe` also pass. The full raw
  GWE suite now has 299 passing tests; the cargo runs still emit the existing
  unused-code warnings and Windows profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs`, `src/ce/resource.rs`, and `tests/coredll_raw_gwe.rs`:
  raw `CreateDIBSection` now covers the first CE `wingdi.h` section-backed DIB
  path. A non-null file-mapping `hSection` is validated, the DIB bits are
  registered as a mapping view at `dwOffset`, `FlushViewOfFile` synchronizes
  written pixels back to the mapping, `DeleteObject` removes the implicit view,
  and `GetObjectW(DIBSECTION)` reports `dshSection` plus `dsOffset`.
- Validation after the section-backed DIB-section slice:
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_create_dibsection_backs_bits_with_file_mapping_section` passed.
  `cargo fmt --check`, `cargo check --features unicorn,trace,win32-desktop`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe` also pass. The full raw GWE suite now has 300 passing tests;
  the cargo runs still emit the existing unused-code warnings and Windows
  profile/incremental-cache cleanup noise.
- `tests/coredll_raw_gwe.rs`: section-backed DIB-section coverage now verifies
  two `CreateDIBSection` handles over the same file-mapping section/offset stay
  coherent through `FlushViewOfFile`: the second DIB is seeded from mapping
  bytes written by the first, later flushes from the second update the first
  view, and `DeleteObject` removes both implicit views.
- Validation after the shared section-backed DIB-section view slice:
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_section_backed_dibsections_share_mapping_bytes` passed. `cargo
  fmt --check`, `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe` also pass. The full raw GWE suite now has 301 passing tests;
  cargo runs still emit the existing unused-code warnings and Windows
  profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `CreateDIBSection` now follows CE GDIAPI `draw.cpp::passNull2Draw` by
  rejecting bad nonzero HDC handles with `ERROR_INVALID_HANDLE`, even for
  `DIB_RGB_COLORS`, while preserving the null-HDC RGB DIB-section creation path
  covered by the existing null-`ppvBits` regression.
- Validation after the CE bad-HDC DIB-section slice:
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_create_dibsection_rejects_bad_nonzero_hdc` passed. `cargo
  fmt --check`, `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe` also pass. The full raw GWE suite now has 302 passing tests;
  cargo runs still emit the existing unused-code warnings and Windows
  profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `CreateDIBSection` now accepts CE public `wingdi.h` 12-byte
  `BITMAPCOREHEADER`/`BITMAPCOREINFO` inputs for indexed `DIB_RGB_COLORS`
  sections. The path treats core headers as uncompressed `BI_RGB`, reads
  RGBTRIPLE color-table entries through the existing DIB table parser, and
  still allocates ordinary DIB-section backing plus an optional `ppvBits`
  output.
- Validation after the CE `BITMAPCOREHEADER` DIB-section slice:
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_create_dibsection_accepts_bitmapcoreheader` passed. `cargo
  fmt --check`, `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe` also pass. The full raw GWE suite now has 303 passing tests;
  cargo runs still emit the existing unused-code warnings and Windows
  profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: section-backed DIB-section
  teardown now commits dirty DIB bits through the shared file-mapping backing
  before `DeleteObject` removes the implicit mapping view. The shared commit
  helper is reused by `FlushViewOfFile` and `UnmapViewOfFile`, so a sibling
  DIB-section view over the same section/offset observes the final bytes even
  when the caller never explicitly flushed the deleted DIB.
- Validation after the section-backed DIB `DeleteObject` writeback slice:
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_deleteobject_flushes_section_backed_dib_bits` passed. `cargo
  fmt --check`, `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe` also pass. The full raw GWE suite now has 304 passing tests;
  cargo runs still emit the existing unused-code warnings and Windows
  profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_kernel.rs`: raw
  `StringCompress`/`StringDecompress` now cover the CE `compr2.c` non-raw
  packet branch with an emulator-owned deterministic opaque stream encoding for
  shrinkable nonzero half-streams. Unknown external non-raw payloads still fail
  closed because the reviewed CE source tree exposes `CECompress`/
  `CEDecompress` declarations/callers, but not the private engine body.
- `src/emulator/unicorn.rs`: full-feature scheduler validation is green again.
  Parked-process selection now preserves active-matching parked entries while
  skipping them as candidates, and the affected scheduler fixtures now create
  resumable parked CPUs with nonzero saved PCs.
- Validation after the `StringCompress` opaque-payload and scheduler test-fix
  slice: `cargo test --features unicorn,trace,win32-desktop --test
  coredll_raw_kernel coredll_raw_string_compress_decompress_matches_ce_raw_packet_edges`,
  `cargo test --features unicorn,trace,win32-desktop guest_thread_stack_tests`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed. The
  full run still emits the existing unused-code warnings, PowerShell profile
  noise, and Windows incremental-cache cleanup note.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw direct-DIB source
  parsing now accepts CE 8 bpp `BI_RLE8` compressed caller bits for
  `StretchDIBits` and `SetDIBitsToDevice` when `biSizeImage` supplies the
  compressed payload size. The decoder handles encoded runs, EOL/EOB, delta,
  and absolute-mode records before feeding ordinary 8 bpp indexed rows through
  the existing selected-DIB/framebuffer renderer.
- Validation after the direct-DIB `BI_RLE8` slice: `cargo test --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_direct_rle8_dib_decodes_for_dib_to_device_calls`,
  `cargo check --features unicorn,trace,win32-desktop`, `cargo test -j 1
  --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed. The full
  raw GWE suite now has 305 passing tests; cargo runs still emit the existing
  unused-code warnings and Windows profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw direct-DIB source
  parsing now also accepts CE 4 bpp `BI_RLE4` compressed caller bits when
  `biSizeImage` supplies the payload size. The decoder covers encoded runs,
  EOL/EOB, delta, and absolute-mode packed nibbles before rendering through
  ordinary 4 bpp indexed bitmap rows.
- Validation after the direct-DIB `BI_RLE4` slice: `cargo test --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_direct_rle4_dib_decodes_for_dib_to_device_calls`, `cargo check
  --features unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and `cargo test -j 1
  --features unicorn,trace,win32-desktop` passed. The full raw GWE suite now
  has 306 passing tests; cargo runs still emit the existing unused-code
  warnings and Windows profile/incremental-cache cleanup noise.
- `src/ce/object.rs`, `src/ce/kernel.rs`, and `tests/coredll_raw_gwe.rs`:
  unnamed file-mapping objects with live views now distinguish a closed public
  handle from the still-retained mapping backing. Section-backed DIB views keep
  flushing and synchronizing with sibling DIB views after the caller closes the
  original mapping handle, while the closed handle is rejected for new
  `MapViewOfFile` calls. The retained mapping object is removed once the last
  live view/DIB section is deleted.
- Validation after the closed mapping-handle DIB-section lifetime slice:
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_section_backed_dib_survives_closed_mapping_handle`, `cargo check
  --features unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, `cargo test -j 1
  --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed. The full
  raw GWE suite now has 307 passing tests; cargo runs still emit the existing
  unused-code warnings and Windows profile/incremental-cache cleanup noise.
- `src/emulator/unicorn.rs`: while validating the DIB-section slice, the
  scheduler parked-process pop path was corrected to preserve active-matching
  parked entries that still carry blocked send/wait state, instead of treating
  them like stale self-duplicates. This keeps a blocked sender queued until its
  send result is ready while still dropping ordinary active duplicate entries.
- `src/ce/object.rs`, `src/ce/coredll.rs`, and `tests/coredll_raw_gwe.rs`:
  file-backed mappings now start with no synthetic zero-filled mapping buffer,
  so `CreateDIBSection` over a mapped file seeds DIB bits from the backing file
  while still overlaying any already-populated dirty mapping bytes. The DIB
  section path now caches the materialized range into the mapping and
  `FlushViewOfFile` writes dirty DIB bytes back to the host-backed CE file.
- Validation after the file-backed DIB-section mapping slice: `cargo test
  --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_file_backed_dibsection_seeds_and_flushes_file_bytes`, `cargo test
  -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_memory_file`, `cargo check --features unicorn,trace,win32-desktop`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed. The full
  raw GWE suite now has 308 passing tests; cargo runs still emit the existing
  unused-code warnings and Windows profile/incremental-cache cleanup noise.
- `src/ce/kernel.rs`, `src/ce/coredll.rs`, and `tests/coredll_raw_gwe.rs`:
  raw `ExtEscape` now implements the CE GPE direct gamma payload path from
  `ddi_if.cpp`: `DRVESC_GETGAMMAVALUE` writes the current `ULONG` gamma through
  `pvOut`, and `DRVESC_SETGAMMAVALUE` takes the gamma value from `cjIn` while
  validating the `pvIn` BOOL buffer. The display gamma state starts at the CE
  AA text default from `aablt.cpp` (`2330`) and clamps to the CE 1000..3000
  range, while the existing CE GDIAPI null-buffer invalid-access coverage stays
  intact.
- Validation after the direct gamma `ExtEscape` slice: `cargo fmt`, `cargo
  test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_ext_escape_matches_ce_query_and_protected_escape_edges`, `cargo
  test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `cargo fmt --check`, `cargo check --features unicorn,trace,win32-desktop`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed. The
  full raw GWE suite remains at 308 passing tests; cargo runs still emit the
  existing unused-code warnings and Windows profile/incremental-cache cleanup
  noise.
- `src/ce/kernel.rs`, `src/ce/coredll.rs`, and `tests/coredll_raw_gwe.rs`:
  raw `ExtEscape` now also implements the CE VGAFLAT/SMI3DR direct
  screen-rotation payload path. `DRVESC_GETSCREENROTATION` writes the
  supported `DMDO_0|90|180|270` mask in the high byte plus the current mode in
  the low byte, and `DRVESC_SETSCREENROTATION` accepts the `cjIn` mode value,
  updates local display-driver rotation state, and returns
  `DISP_CHANGE_BADMODE` for unsupported modes while preserving the CE GDIAPI
  all-zero/null invalid-access edge.
- Validation after the direct rotation `ExtEscape` slice: `cargo fmt`, `cargo
  test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_ext_escape_matches_ce_query_and_protected_escape_edges`, `cargo
  test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `cargo fmt --check`, `cargo check --features unicorn,trace,win32-desktop`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed. The
  full raw GWE suite remains at 308 passing tests; cargo runs still emit the
  existing unused-code warnings and Windows profile/incremental-cache cleanup
  noise.
- `src/ce/kernel.rs`, `src/ce/coredll.rs`, and `tests/coredll_raw_gwe.rs`:
  raw `ExtEscape` now returns a CE-shaped `DISPPERF_TIMING` table backed by
  local display-performance state instead of a permanently zeroed table. Raw
  `BitBlt`, `StretchBlt`, `PatBlt`, non-copy `MaskBlt`, and
  `TransparentImage` record nonzero GPE counts/timing rows keyed by ROP/ROP4,
  raw `LineTo` and `Polyline` record the CE VGAFLAT `ROP_LINE` row,
  `StretchBlt` marks the CE stretch parameter slot, and
  `DISPPERF_EXTESC_CLEARTIMING` clears the table/unhandled counter.
- Validation after the DISPPERF timing slice: `cargo fmt`, `cargo test
  --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_ext_escape_matches_ce_query_and_protected_escape_edges`, `cargo
  test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `cargo fmt --check`, `cargo check --features unicorn,trace,win32-desktop`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed. The
  full raw GWE suite remains at 308 passing tests; cargo runs still emit the
  existing unused-code warnings plus Windows profile/incremental-cache cleanup
  noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `TransparentImage` now records a CE `DrvTransparentBlt`-style DISPPERF GPE
  row with ROP4 `0xCCCC` after routing a readable selected-bitmap source
  through the transparent blit path; the framebuffer color-key regression now
  asserts the timing row.
- Validation after the `TransparentImage` DISPPERF slice: `cargo fmt`, `cargo
  test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_transparent_image_copies_selected_bitmap_with_color_key`, `cargo
  test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `cargo fmt --check`, `cargo check --features unicorn,trace,win32-desktop`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed. The
  full raw GWE suite remains at 308 passing tests; cargo runs still emit the
  existing unused-code warnings plus Windows profile/incremental-cache cleanup
  noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_kernel.rs`: raw
  `ImageList_Draw`, `ImageList_DrawEx`, and `ImageList_DrawIndirect` now record
  CE `StretchBlt_I`-style DISPPERF GPE rows when the explicit `ILD_MASK` or
  `ILD_IMAGE` branches run, including caller `ILD_ROP`/`dwRop` values. The
  `ImageList_DrawIndirect` regression now asserts the `SRCINVERT` timing row
  for an `ILD_IMAGE | ILD_ROP` draw.
- Validation after the image-list DISPPERF draw-pass slice: `cargo fmt`,
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel
  image_list_draw_indirect_applies_x_bitmap_offset_and_rgb_bk_fill`, `cargo
  fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_kernel`, `cargo check --features unicorn,trace,win32-desktop`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop` passed. Cargo
  runs still emit the existing unused-code warnings plus Windows
  profile/incremental-cache cleanup noise, and the eVC4 MIPSII fixture remains
  ignored because the toolchain is not configured.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw direct-DIB
  `StretchDIBits` and `SetDIBitsToDevice` now contribute CE display-blit-style
  DISPPERF GPE rows after accepting a caller DIB source. `StretchDIBits`
  records the caller ROP and marks the stretch parameter when the source and
  destination extents differ; `SetDIBitsToDevice` records the copy-style
  `SRCCOPY` row. The existing direct-DIB selected-memory-DIB regressions now
  assert those timing rows.
- Validation after the direct-DIB DISPPERF slice: `cargo fmt`, `cargo test
  --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_stretch_dibits_paints_selected_memory_dib_and_validates_ce_edges`,
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_set_dibits_to_device_paints_selected_memory_dib_from_bottom_up_source`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, `cargo fmt --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. Cargo runs still emit the existing
  unused-code warnings plus Windows profile/incremental-cache cleanup noise,
  and the eVC4 MIPSII fixture remains ignored because the toolchain is not
  configured.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `AlphaBlend` now
  contributes a CE `DrvAlphaBlend`/`AnyBlt`/`BltPrepare` DISPPERF GPE timing
  row keyed by ROP4 `0xCCCC` after a readable selected-bitmap source reaches
  the alpha blit path. The selected-DIB source-constant, selected-DIB stretch,
  and framebuffer stretch alpha regressions now assert the timing row, GPE
  count/time, no emulated count, and the CE stretch parameter slot for
  non-matching source/destination extents.
- Validation after the `AlphaBlend` DISPPERF slice: `cargo fmt`, `cargo test
  --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_alpha_blend_applies_source_constant_alpha_between_selected_dibs`,
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_alpha_blend_uses_ce_gpe_stretch_sampling_between_selected_dibs`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe alpha_blend` passed. Cargo runs still emit the existing
  unused-code warnings plus Windows profile/incremental-cache cleanup noise.
- Broader validation after the `AlphaBlend` DISPPERF slice: `cargo fmt
  --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, `cargo check --features unicorn,trace,win32-desktop`,
  `git diff --check`, and `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. `git diff --check` output was limited
  to existing CRLF normalization warnings, Cargo still emits the existing
  unused-code warnings plus Windows incremental-cache cleanup noise, and the
  eVC4 MIPSII fixture remains ignored because the toolchain is not configured.
- After adding the framebuffer `AlphaBlend` DISPPERF assertion, `cargo fmt`
  and `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe alpha_blend` passed.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `GradientFill`
  now follows the CE rectangle-gradient contract from `draw.cpp`,
  `wingdi.h`, `winddi.h`, and GPE `ddi_if.cpp`: null/bad HDCs fail with
  `ERROR_INVALID_HANDLE`, null vertex/mesh pointers, zero counts, invalid mesh
  indices, and `GRADIENT_FILL_TRIANGLE` fail with `ERROR_INVALID_PARAMETER`,
  horizontal/vertical rectangle rendering remains covered, and
  `GetDeviceCaps(SHADEBLENDCAPS)` now returns `0x17`
  (`SB_CONST_ALPHA | SB_PIXEL_ALPHA | SB_PREMULT_ALPHA | SB_GRAD_RECT`) instead
  of advertising unsupported triangle gradients.
- Validation after the `GradientFill` capability/validation slice: `cargo
  fmt`, `cargo test --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe gradient_fill`, and `cargo test --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_get_device_caps_returns_ce_display_capabilities` passed. Cargo
  runs still emit the existing unused-code warnings plus Windows
  profile/incremental-cache cleanup noise.
- Broader validation after the `GradientFill` slice: `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and `git diff
  --check` passed. `git diff --check` output was limited to the existing
  LF-to-CRLF normalization warnings, Cargo still emits the existing unused-code
  warnings plus Windows incremental-cache cleanup noise, and the eVC4 MIPSII
  fixture remains ignored because the toolchain is not configured.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `Polygon` and
  `Polyline` now follow CE `draw.cpp::passNull2Draw`/`SimplePolyTest`
  validation. Null/bad HDCs fail with `ERROR_INVALID_HANDLE`, null point
  arrays fail with `ERROR_INVALID_PARAMETER`, `Polyline` rejects negative and
  single-point counts but accepts count zero as a successful no-op, and
  `Polygon` rejects counts below two while preserving success last-error state
  for those small-count failures.
- Validation after the polygon/polyline validation slice: `cargo fmt`, `cargo
  test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  polyline`, and `cargo test --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe polygon` passed. Cargo runs still emit the existing
  unused-code warnings plus Windows profile/incremental-cache cleanup noise.
- Broader validation after the polygon/polyline slice: `cargo fmt --check`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and `git diff
  --check` passed. The raw GWE suite is now at 311 passing tests. `git diff
  --check` output was limited to the existing LF-to-CRLF normalization warnings,
  Cargo still emits the existing unused-code warnings plus Windows
  incremental-cache cleanup noise, and the eVC4 MIPSII fixture remains ignored
  because the toolchain is not configured.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `Rectangle`,
  `Ellipse`, and `RoundRect` now follow CE `draw.cpp::passNull2Draw` invalid
  HDC behavior. Null and bad HDC handles fail with `ERROR_INVALID_HANDLE`
  before any degenerate/no-op shape path, and the raw shape validation
  regression now covers all three APIs plus valid-HDC success.
- Validation after the shape-HDC validation slice: `cargo fmt` and `cargo test
  --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_gdi_shapes_validate_ce_hdc_edges` passed. Cargo runs still emit
  the existing unused-code warnings plus Windows profile/incremental-cache
  cleanup noise.
- The full feature test build also exposed a test-profile compile break in
  `src/emulator/unicorn.rs`: `persist_run_state` restored state and then called
  escaped-send completion helpers that require `&mut CeKernel` while accepting
  only `&CeKernel`. The helper now accepts the mutable kernel already supplied
  by every call site, keeping persisted escaped callout cleanup available in
  full `cargo test`.
- Broader validation after the shape-HDC/test-profile fix slice: `cargo fmt
  --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and `git diff
  --check` passed. The raw GWE suite remains at 311 passing tests. `git diff
  --check` output was limited to the existing LF-to-CRLF normalization warnings,
  Cargo still emits the existing unused-code warnings plus Windows
  incremental-cache cleanup noise, and the eVC4 MIPSII fixture remains ignored
  because the toolchain is not configured.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw miscellaneous draw
  APIs now follow CE `draw.cpp::passNull2Draw` validation ordering for
  `RectVisible`, `FillRect`, `DrawFocusRect`, `DrawEdge`, `MoveToEx`, `LineTo`,
  `SetPixel`, `GetROP2`, and `SetROP2`. Null and bad HDC handles now fail with
  `ERROR_INVALID_HANDLE`, `RectVisible` returns CE's `-1` error value on HDC
  failures, valid-HDC null rectangles fail with `ERROR_INVALID_PARAMETER` for
  the checked APIs, invalid `FillRect` brushes fail with `ERROR_INVALID_HANDLE`,
  and `SetROP2(NULL, 0)` now reports the invalid handle before the invalid ROP2
  value.
- Validation after the miscellaneous draw-validation slice: `cargo fmt`,
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_gdi_misc_draw_apis_validate_ce_pass_null_edges`, `cargo test -j
  1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, `cargo fmt
  --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 312 passing
  tests. `git diff --check` output was limited to the existing LF-to-CRLF
  normalization warnings, and Cargo runs still emit the existing unused-code
  warnings plus Windows profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: the same CE
  `draw.cpp::passNull2Draw` validation work now covers `GetPixel`,
  `GetDIBColorTable`, `SetDIBColorTable`, and `SetBitmapBits`. `GetPixel`
  returns `CLR_INVALID` with `ERROR_INVALID_HANDLE` for null/bad HDCs, DIB
  color-table calls check HDC validity before null table pointers, selected-DIB
  color-table starts past the palette fail with `ERROR_INVALID_HANDLE`, and
  `SetBitmapBits` now distinguishes null/bad bitmap handles from null or zero
  source payloads before copying valid bitmap bytes.
- Validation after the DIB color-table/bitmap-bits validation slice: `cargo
  fmt`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, `cargo fmt --check`, `git diff --check`, `cargo check
  --features unicorn,trace,win32-desktop`, and `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 313 passing
  tests. `git diff --check` output was limited to the existing LF-to-CRLF
  normalization warnings, and Cargo runs still emit the existing unused-code
  warnings plus Windows profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `PatBlt`
  validation now joins the CE `draw.cpp::passNull2Draw` slice. Null and bad
  destination HDCs fail with `ERROR_INVALID_HANDLE`, and CE-invalid ROP4 values
  now fail with `ERROR_INVALID_PARAMETER` instead of reporting success.
- Validation after the `PatBlt` validation slice: `cargo fmt`, `cargo fmt
  --check`, `cargo check --features unicorn,trace,win32-desktop`, `cargo test
  -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed. The raw GWE
  suite remains at 313 passing tests, and `git diff --check` output was limited
  to the existing LF-to-CRLF normalization warnings. Cargo runs still emit the
  existing unused-code warnings plus Windows profile/incremental-cache cleanup
  noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `TransparentImage` now follows the CE `draw.cpp::passNull2Draw` validation
  order for null/bad destination and source HDCs. Those calls now fail with
  `ERROR_INVALID_HANDLE` before dimension validation, while valid-HDC zero
  extents still fail with `ERROR_INVALID_PARAMETER`.
- Validation after the `TransparentImage` validation slice: `cargo fmt`,
  `cargo fmt --check`, `git diff --check`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_gdi_misc_draw_apis_validate_ce_pass_null_edges`, `cargo test -j
  1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, `cargo
  check --features unicorn,trace,win32-desktop`, and `cargo test -j 1
  --features unicorn,trace,win32-desktop` passed. The raw GWE suite remains at
  313 passing tests, and the full suite keeps the eVC4 MIPSII fixture ignored
  because that toolchain is not configured. `git diff --check` output was
  limited to the existing LF-to-CRLF normalization warnings. Cargo runs still
  emit the existing unused-code warnings plus Windows profile/incremental-cache
  cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `SetDIBitsToDevice` now follows CE `draw.cpp::passNull2Draw` for null DIB
  payloads. Calls with null `lpvBits`/`lpbmi`, including the CE
  null-HDC/null-payload case, now fail with `ERROR_INVALID_PARAMETER` before
  HDC validation, while valid payloads still reach the existing HDC
  validation/rendering path.
- Validation after the `SetDIBitsToDevice` validation-order slice: `cargo
  fmt`, `cargo fmt --check`, `git diff --check`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_set_dibits_to_device_paints_selected_memory_dib_from_bottom_up_source`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, `cargo check --features unicorn,trace,win32-desktop`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed. The raw GWE
  suite remains at 313 passing tests, and the full suite keeps the eVC4 MIPSII
  fixture ignored because that toolchain is not configured. `git diff --check`
  output was limited to the existing LF-to-CRLF normalization warnings. Cargo
  runs still emit the existing unused-code warnings plus Windows
  profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `TransparentImage` now follows CE `draw.cpp::TransparentBltPatBltTest` and
  `TransparentBltSetPixelTest` for same-framebuffer HDC copies. The framebuffer
  fallback snapshots the source pixels before writing, skips source pixels that
  match the transparent `COLORREF`, copies nonmatching source pixels, and
  preserves the black/white skip/copy behavior CE verifies after `PatBlt` or
  `SetPixel` setup.
- Validation after the framebuffer `TransparentImage` color-key slice: `cargo
  fmt`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe
  coredll_raw_transparent_image_copies_between_framebuffer_hdc_with_color_key`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, `cargo fmt --check`, `git diff --check`, `cargo check
  --features unicorn,trace,win32-desktop`, and `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 314 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. `git diff --check` output was limited to the
  existing LF-to-CRLF normalization warnings. Cargo runs still emit the
  existing unused-code warnings plus Windows profile/incremental-cache cleanup
  noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `DrawEdge` now
  follows CE `draw.cpp::DrawEdgeTest1` for invalid edge-type bits on color
  display targets. `DrawEdge(hdc, &rc, 0xFFFFFFFF, BF_RECT)` now fails with
  `ERROR_INVALID_PARAMETER` after the HDC and rectangle pointer are valid,
  while retaining the existing monochrome selected-DIB exception path.
- Validation after the `DrawEdge` invalid-edge slice: `cargo fmt`, `cargo test
  -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_gdi_misc_draw_apis_validate_ce_pass_null_edges`, `cargo test -j
  1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, `cargo fmt
  --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite remains at 314 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. `git diff --check` output was limited to the
  existing LF-to-CRLF normalization warnings. Cargo runs still emit the
  existing unused-code warnings plus Windows profile/incremental-cache cleanup
  noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `DrawFocusRect` now follows CE `draw.cpp::DrawFocusRectTest` for
  framebuffer-backed HDCs. Screen/window HDC calls now XOR the dotted perimeter
  through the visible framebuffer clips, mark the affected rectangle dirty, and
  a second draw toggles the same pixels back to their original values instead
  of returning success without drawing.
- Validation after the framebuffer `DrawFocusRect` slice: `cargo fmt`, `cargo
  test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_draw_focus_rect_xors_framebuffer_and_toggles`, `cargo test -j 1
  --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, `cargo fmt
  --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 315 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. `git diff --check` output was limited to the
  existing LF-to-CRLF normalization warnings. Cargo runs still emit the
  existing unused-code warnings plus Windows profile/incremental-cache cleanup
  noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `PatBlt` now
  follows CE `draw.cpp::PatBltBadRopTest` for source-dependent ROP3 values.
  After HDC, dimension, and ROP4 validation, `PatBlt` now applies the CE
  source-dependency test to the ROP3 byte and fails source-dependent ROPs such
  as `SRCCOPY`, `SRCINVERT`, and `PATPAINT` with `ERROR_INVALID_HANDLE`
  because `PatBlt` has no source DC, while source-free ROPs like `BLACKNESS`,
  `WHITENESS`, `DSTINVERT`, `PATCOPY`, and `NOP` remain valid.
- Validation after the `PatBltBadRopTest` source-dependent ROP3 slice: `cargo
  fmt`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe
  coredll_raw_pat_blt_rejects_source_dependent_rop3_without_source`, `cargo
  test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `cargo fmt --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 316 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. `git diff --check` output was limited to the
  existing LF-to-CRLF normalization warnings. Cargo runs still emit the
  existing unused-code warnings plus Windows profile/incremental-cache cleanup
  noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `PatBlt` now also
  renders CE source-free pattern/destination ROP3 values through the shared
  ROP3 truth-table evaluator instead of treating unrecognized successful ROPs
  as no-op success. The regression uses CE `draw.cpp::gnvRop3Array` `DPa`
  (`0x00A00000`) with a red brush over a white destination and verifies red
  output for both selected memory DIB and framebuffer-backed HDC targets.
- Validation after the source-free `PatBlt` ROP3 rendering slice: `cargo fmt`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe
  coredll_raw_pat_blt_applies_source_free_pattern_destination_rop3`, `cargo
  test -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `cargo fmt --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 317 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. `git diff --check` output was limited to the
  existing LF-to-CRLF normalization warnings. Cargo runs still emit the
  existing unused-code warnings plus Windows profile/incremental-cache cleanup
  noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `PatBlt` now
  accepts CE `draw.cpp::TryShapes`/`PatBltSimple` signed extents. Zero width or
  height now succeeds as a no-op, and negative width/height normalizes into the
  mirrored destination rectangle for both selected memory DIBs and
  framebuffer-backed HDCs.
- Validation after the `PatBlt` signed-extent slice: `cargo fmt`, `cargo test
  -j 1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_pat_blt_accepts_zero_and_negative_extents`, `cargo test -j 1
  --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, `cargo fmt
  --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 318 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. `git diff --check` output was limited to the
  existing LF-to-CRLF normalization warnings. Cargo runs still emit the
  existing unused-code warnings plus Windows profile/incremental-cache cleanup
  noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `SetBitmapBits` now
  allocates owned backing for pointerless `CreateBitmap(..., NULL)` handles
  before copying, so CE `draw.cpp::CreateBitmapSquares*` and
  `SetBitmapBitsOnePixel`-style writes are visible to later blits. Copies are
  bounded to bitmap storage and return the copied byte count.
- Validation after the `SetBitmapBits` pointerless `CreateBitmap` backing
  slice: `cargo fmt`, `cargo test -j 1 --features unicorn,trace,win32-desktop
  --test coredll_raw_gwe
  coredll_raw_set_bitmap_bits_allocates_createbitmap_backing`, `cargo test -j
  1 --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, `cargo fmt
  --check`, `git diff --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 319 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. `git diff --check` output was limited to the
  existing LF-to-CRLF normalization warnings. Cargo runs still emit the
  existing unused-code warnings plus Windows profile/incremental-cache cleanup
  noise.
- `src/ce/resource.rs`, `src/ce/coredll.rs`, and `tests/coredll_raw_gwe.rs`:
  bitmap objects now track whether their backing bits are writable. BMP/DIB
  loader paths create read-only owned backing, while caller-created bitmaps and
  DIB sections remain writable. Raw `SetBitmapBits` now fails before copying
  when the target bitmap is read-only, preserving the original backing bytes as
  CE `draw.cpp::WritableBitmapTest(ESetBitmapBits)` expects for loaded
  bitmaps.
- Validation after the read-only loaded-bitmap `SetBitmapBits` slice: `cargo
  fmt`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_set_bitmap_bits_rejects_readonly_loaded_bitmap`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, `cargo fmt --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 320 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. Cargo runs still emit the existing unused-code
  warnings plus Windows profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `PatBlt` now rejects
  read-only selected bitmap destinations before rendering, preserving the
  backing bytes for CE `draw.cpp::WritableBitmapTest(EPatBlt)`-style loaded
  bitmap targets while leaving writable selected-DIB and framebuffer paths
  intact.
- Validation after the read-only selected-bitmap `PatBlt` slice: `cargo fmt`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_pat_blt_rejects_readonly_selected_bitmap`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe` passed. The raw GWE suite is now at 321 passing tests. Cargo
  runs still emit the existing unused-code warnings plus Windows
  profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `FillRect` now
  rejects read-only selected bitmap destinations before filling, preserving the
  backing bytes for CE `draw.cpp::WritableBitmapTest(EFillRect)`-style loaded
  bitmap targets while leaving writable selected-DIB and framebuffer paths
  intact.
- Validation after the read-only selected-bitmap `FillRect` slice: `cargo fmt`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_fill_rect_rejects_readonly_selected_bitmap`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, `cargo fmt --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 322 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. Cargo runs still emit the existing unused-code
  warnings plus Windows profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `InvertRect` now
  rejects read-only selected bitmap destinations before inversion, preserving
  backing bytes for CE `draw.cpp::WritableBitmapTest(EInvertRect)`-style loaded
  bitmap targets while leaving writable selected-DIB behavior intact.
- `src/emulator/unicorn.rs`: `rotate_to_ready_parked_threads` now treats a
  requested receiver thread with pending sent-message work as ready for the
  blocked-send handoff, fixing
  `blocked_send_handoff_prefers_parked_receiver_over_visible_sender_work`
  without changing unrelated parked-process filters.
- Validation after the read-only selected-bitmap `InvertRect` slice and parked
  receiver handoff test fix: `cargo fmt`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_invert_rect_rejects_readonly_selected_bitmap`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, `cargo fmt --check`, `cargo check --features
  unicorn,trace,win32-desktop`, targeted lib tests for
  `blocked_send_handoff_prefers_parked_receiver_over_visible_sender_work` and
  `ready_parked`, and full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 323 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. Cargo runs still emit the existing unused-code
  warnings plus Windows profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `SetPixel` now
  rejects read-only selected bitmap destinations before filling the target
  pixel, returning `CLR_INVALID` and preserving the four-pixel backing block for
  CE `draw.cpp::WritableBitmapTest(ESetPixel)`-style loaded bitmap targets.
- Validation after the read-only selected-bitmap `SetPixel` slice: `cargo fmt`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_set_pixel_rejects_readonly_selected_bitmap`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, `cargo fmt --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 324 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. Cargo runs still emit the existing unused-code
  warnings plus Windows profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `LineTo` now rejects
  read-only selected bitmap destinations after the CE-compatible `MoveToEx`
  setup but before drawing or advancing the current point, preserving backing
  bytes for CE `draw.cpp::WritableBitmapTest(ELineTo)`-style loaded bitmap
  targets.
- Focused validation after the read-only selected-bitmap `LineTo` slice:
  `cargo fmt` and `cargo test -j 1 --features unicorn,trace,win32-desktop
  --test coredll_raw_gwe
  coredll_raw_line_to_rejects_readonly_selected_bitmap` passed. Cargo still
  emits the existing unused-code warnings plus Windows profile/incremental-cache
  cleanup noise.
- Broader validation after the read-only selected-bitmap `LineTo` slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, `cargo fmt --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 325 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. Cargo runs still emit the existing unused-code
  warnings plus Windows profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `Polyline` now
  rejects read-only selected bitmap destinations after reading the caller's
  point array but before drawing or advancing the current point, preserving
  backing bytes for CE `draw.cpp::WritableBitmapTest(EPolyline)`-style loaded
  bitmap targets.
- Focused validation after the read-only selected-bitmap `Polyline` slice:
  `cargo fmt` and `cargo test -j 1 --features unicorn,trace,win32-desktop
  --test coredll_raw_gwe
  coredll_raw_polyline_rejects_readonly_selected_bitmap` passed. Cargo still
  emits the existing unused-code warnings plus Windows profile/incremental-cache
  cleanup noise.
- Broader validation after the read-only selected-bitmap `Polyline` slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, `cargo fmt --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 326 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. Cargo runs still emit the existing unused-code
  warnings plus Windows profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `Polygon` now
  rejects read-only selected bitmap destinations after reading the caller's
  point array but before fill/stroke rendering, preserving backing bytes for CE
  `draw.cpp::WritableBitmapTest(EPolygon)`-style loaded bitmap targets.
- Focused validation after the read-only selected-bitmap `Polygon` slice:
  `cargo fmt` and `cargo test -j 1 --features unicorn,trace,win32-desktop
  --test coredll_raw_gwe
  coredll_raw_polygon_rejects_readonly_selected_bitmap` passed. Cargo still
  emits the existing unused-code warnings plus Windows profile noise.
- Broader validation after the read-only selected-bitmap `Polygon` slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, `cargo fmt --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 327 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. Cargo runs still emit the existing unused-code
  warnings plus Windows profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `Rectangle` now
  rejects read-only selected bitmap destinations before brush fill or pen stroke
  rendering, preserving backing bytes for CE
  `draw.cpp::WritableBitmapTest(ERectangle)`-style loaded bitmap targets.
- Focused validation after the read-only selected-bitmap `Rectangle` slice:
  `cargo fmt` and `cargo test -j 1 --features unicorn,trace,win32-desktop
  --test coredll_raw_gwe
  coredll_raw_rectangle_rejects_readonly_selected_bitmap` passed. Cargo still
  emits the existing unused-code warnings plus Windows profile/incremental-cache
  cleanup noise.
- Broader validation after the read-only selected-bitmap `Rectangle` slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, `cargo fmt --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 328 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. Cargo runs still emit the existing unused-code
  warnings plus Windows profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `Ellipse` now
  rejects read-only selected bitmap destinations before brush fill or pen
  outline rendering, preserving backing bytes for CE
  `draw.cpp::WritableBitmapTest(EEllipse)`-style loaded bitmap targets.
- Focused validation after the read-only selected-bitmap `Ellipse` slice:
  `cargo fmt` and `cargo test -j 1 --features unicorn,trace,win32-desktop
  --test coredll_raw_gwe
  coredll_raw_ellipse_rejects_readonly_selected_bitmap` passed. Cargo still
  emits the existing unused-code warnings plus Windows profile/incremental-cache
  cleanup noise.
- Broader validation after the read-only selected-bitmap `Ellipse` slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, `cargo fmt --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 329 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. Cargo runs still emit the existing unused-code
  warnings plus Windows profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `RoundRect` now
  rejects read-only selected bitmap destinations before rounded fill or pen
  outline rendering, preserving backing bytes for CE
  `draw.cpp::WritableBitmapTest(ERoundRect)`-style loaded bitmap targets.
- Focused validation after the read-only selected-bitmap `RoundRect` slice:
  `cargo fmt` and `cargo test -j 1 --features unicorn,trace,win32-desktop
  --test coredll_raw_gwe
  coredll_raw_round_rect_rejects_readonly_selected_bitmap` passed. Cargo still
  emits the existing unused-code warnings plus Windows profile/incremental-cache
  cleanup noise.
- Broader validation after the read-only selected-bitmap `RoundRect` slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, `cargo fmt --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 330 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. Cargo runs still emit the existing unused-code
  warnings plus Windows profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `DrawFocusRect`
  now rejects read-only selected bitmap destinations before XOR perimeter
  rendering, and raw `DrawEdge` rejects read-only selected bitmap destinations
  after CE parameter validation and before border/fill rendering or `BF_ADJUST`
  rectangle mutation, preserving backing bytes for CE
  `draw.cpp::WritableBitmapTest(EDrawFocusRect/EDrawEdge)`-style loaded bitmap
  targets.
- Focused validation after the read-only selected-bitmap `DrawFocusRect` and
  `DrawEdge` slice: `cargo fmt`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_draw_focus_rect_rejects_readonly_selected_bitmap`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_draw_edge_rejects_readonly_selected_bitmap`
  passed. Cargo still emits the existing unused-code warnings plus Windows
  profile/incremental-cache cleanup noise.
- Broader validation after the read-only selected-bitmap `DrawFocusRect` and
  `DrawEdge` slice: `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, `cargo fmt --check`,
  `cargo check --features unicorn,trace,win32-desktop`, and full `cargo test
  -j 1 --features unicorn,trace,win32-desktop` passed. The raw GWE suite is
  now at 332 passing tests, and the full suite keeps the eVC4 MIPSII fixture
  ignored because that toolchain is not configured. Cargo runs still emit the
  existing unused-code warnings plus Windows profile/incremental-cache cleanup
  noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `BitBlt`,
  `StretchBlt`, and `MaskBlt` now reject read-only selected bitmap destinations
  after their CE HDC/extent/ROP or mask validation and before destination pixel
  writes, preserving backing bytes for CE
  `draw.cpp::WritableBitmapTest(EBitBlt/EStretchBlt/EMaskBlt)`-style loaded
  bitmap targets.
- Focused validation after the read-only selected-bitmap blit slice:
  `cargo fmt`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_bit_blt_rejects_readonly_selected_bitmap`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_stretch_blt_rejects_readonly_selected_bitmap`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_mask_blt_rejects_readonly_selected_bitmap`
  passed. Cargo still emits the existing unused-code warnings plus Windows
  profile/incremental-cache cleanup noise.
- Broader validation after the read-only selected-bitmap blit slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, `cargo fmt --check`, `cargo check --features
  unicorn,trace,win32-desktop`, and full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 335 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. Cargo runs still emit the existing unused-code
  warnings plus Windows profile/incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `TransparentImage` now rejects read-only selected bitmap destinations after
  CE HDC and nonzero-extent validation and before source snapshotting or
  color-key rendering, preserving backing bytes for CE
  `draw.cpp::WritableBitmapTest(ETransparentImage)`-style loaded bitmap
  targets.
- Focused validation after the read-only selected-bitmap `TransparentImage`
  slice: `cargo fmt` and `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_transparent_image_rejects_readonly_selected_bitmap` passed. This
  also repaired the Unicorn modal handoff compile break by consuming the
  optional framebuffer only once during parked-modal preparation. Cargo still
  emits the existing unused-code warnings plus Windows profile/import noise.
- Broader validation after the read-only selected-bitmap `TransparentImage`
  slice and Unicorn modal-handoff compile repair: `cargo fmt --check`,
  `cargo check --features unicorn,trace,win32-desktop`, `cargo test -j 1
  --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed. The raw GWE
  suite is now at 336 passing tests, and the full suite keeps the eVC4 MIPSII
  fixture ignored because that toolchain is not configured. Cargo runs still
  emit the existing unused-code warnings plus Windows profile/import and
  incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `GradientFill` now
  rejects read-only selected bitmap destinations after CE HDC, pointer/count,
  and mode validation and before reading caller vertex/mesh payloads or
  rendering rectangle gradients, preserving backing bytes for CE
  `draw.cpp::WritableBitmapTest(EGradientFill)`-style loaded bitmap targets.
- Focused validation after the read-only selected-bitmap `GradientFill` slice:
  `cargo fmt` and `cargo test -j 1 --features unicorn,trace,win32-desktop
  --test coredll_raw_gwe
  coredll_raw_gradient_fill_rejects_readonly_selected_bitmap` passed. Cargo
  still emits the existing unused-code warnings plus Windows profile/import and
  incremental-cache cleanup noise.
- Broader validation after the read-only selected-bitmap `GradientFill` slice:
  `cargo fmt --check`, `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, and full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 337 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. Cargo runs still emit the existing unused-code
  warnings plus Windows profile/import and incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `AlphaBlend` now
  rejects read-only selected bitmap destinations after CE HDC, blend-function,
  and nonzero-extent validation and before source bitmap validation or
  blending, preserving backing bytes for CE
  `draw.cpp::WritableBitmapTest(EAlphaBlend)`-style loaded bitmap targets.
- Focused validation after the read-only selected-bitmap `AlphaBlend` slice:
  `cargo fmt` and `cargo test -j 1 --features unicorn,trace,win32-desktop
  --test coredll_raw_gwe
  coredll_raw_alpha_blend_rejects_readonly_selected_bitmap` passed. Cargo still
  emits the existing unused-code warnings plus Windows profile/import and
  incremental-cache cleanup noise.
- Broader validation after the read-only selected-bitmap `AlphaBlend` slice:
  `cargo fmt --check`, `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, and full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 338 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. Cargo runs still emit the existing unused-code
  warnings plus Windows profile/import and incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw direct-DIB
  `StretchDIBits` and `SetDIBitsToDevice` now reject read-only selected bitmap
  destinations like CE `draw.cpp::WritableBitmapTest(EStretchDIBits/
  ESetDIBitsToDevice)`, preserving selected-bitmap backing bytes before caller
  DIB payload reads or rendering.
- Focused validation after the read-only selected-bitmap direct-DIB slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_direct_dib_calls_reject_readonly_selected_bitmap`
  passed. Cargo still emits the existing unused-code warnings plus Windows
  profile/import noise.
- Broader validation after the read-only selected-bitmap direct-DIB slice:
  `cargo fmt --check`, `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, and full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 339 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. Cargo runs still emit the existing unused-code
  warnings plus Windows profile/import and incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `DrawTextW` and
  `ExtTextOutW` now reject read-only selected bitmap destinations like CE
  `text.cpp` importing `draw.cpp::WritableBitmapTest(EDrawTextW/EExtTextOut)`,
  preserving selected-bitmap backing bytes before glyph rendering or
  `ETO_OPAQUE` rectangle filling.
- Focused validation after the read-only selected-bitmap text draw slice:
  `cargo fmt` and `cargo test -j 1 --features unicorn,trace,win32-desktop
  --test coredll_raw_gwe
  coredll_raw_text_draw_calls_reject_readonly_selected_bitmap` passed. Cargo
  still emits the existing unused-code warnings plus Windows profile/import
  noise.
- Broader validation after the read-only selected-bitmap text draw slice:
  `cargo fmt --check`, `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, and full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE suite is now at 340 passing
  tests, and the full suite keeps the eVC4 MIPSII fixture ignored because that
  toolchain is not configured. Cargo runs still emit the existing unused-code
  warnings plus Windows profile/import and incremental-cache cleanup noise.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `TrackPopupMenuEx` now clamps normal top-level popup placement to CE screen
  metrics after alignment, matching the `menu.h` screen-sized menu placement
  model. The alignment regression now verifies a near-bottom-right request
  records `(710, 440)` on the default 800x480 screen and still hit-tests the
  selected row.
- Focused validation after the popup screen-clamp slice: `cargo fmt` and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_track_popup_menu_ex_applies_ce_alignment_flags`
  passed. Cargo still emits the existing unused-code warnings plus Windows
  profile/import noise.
- Broader validation after the popup screen-clamp slice: `cargo fmt --check`,
  `cargo check --features unicorn,trace,win32-desktop`, `cargo test -j 1
  --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, full
  `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed. The raw GWE suite remains at 340 passing tests,
  the full suite keeps the eVC4 MIPSII fixture ignored because that toolchain
  is not configured, and `git diff --check` output was limited to existing
  LF-to-CRLF warnings.
- `src/ce/coredll.rs`: popup child-submenu placement now uses one CE-style
  cascade helper for both live modal stack state and framebuffer rendering,
  matching the `menu.h` screen-sized candidate-placement model. Left-flipped
  child panes and bottom-edge child panes are clamped back inside the active
  screen/framebuffer before the submenu is pushed or rendered.
- Focused validation after the popup child-submenu clamp slice:
  `cargo test -j 1 popup_menu_cascading_child_position --features
  unicorn,trace,win32-desktop` passed. Cargo still emits the existing
  unused-code warnings plus Windows profile/import noise.
- Broader validation after the popup child-submenu clamp slice:
  `cargo fmt --check`, `cargo check --features unicorn,trace,win32-desktop`,
  full `cargo test -j 1 --features unicorn,trace,win32-desktop`, and
  `git diff --check` passed. The full suite keeps the eVC4 MIPSII fixture
  ignored because that toolchain is not configured, and `git diff --check`
  output was limited to existing LF-to-CRLF warnings.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw selected-DIB
  `AlphaBlend(AC_SRC_ALPHA)` now matches selected CE
  `alphablend.h::g_stcPPAlpha` 32bpp rows from
  `draw.cpp::AlphaBlendPerPixelAlphaTest`. The implementation now uses CE
  GPE-style rounded divide-by-255 source scaling, destination attenuation, and
  saturating channel adds for premultiplied pixels, and byte-reads 32bpp DIB
  destination pixels so byte-backed test DIBs do not fall back to black.
- Focused validation after the CE per-pixel AlphaBlend table slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_alpha_blend_matches_ce_per_pixel_alpha_32bpp_rows`
  passed. Cargo still emits the existing unused-code warnings.
- Broader validation after the CE per-pixel AlphaBlend table slice:
  `cargo fmt`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe alpha_blend`, `cargo check --features
  unicorn,trace,win32-desktop`, and full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The AlphaBlend filter is at 19 passing
  tests, and the full suite still ignores the eVC4 MIPSII fixture because that
  toolchain is not configured.
- `src/ce/resource.rs`, `src/ce/coredll.rs`, `src/emulator/unicorn.rs`,
  `tests/basic_subsystems.rs`, and `tests/coredll_raw_gwe.rs`: selected DC
  clip regions now store copied `RegionObject` geometry instead of retaining a
  live source `HRGN` handle. Raw `SelectClipRgn`/`GetClipRgn` now matches CE
  `clip.cpp::TestNormalClipRgn` and `TestNoClipRgn`: callers can mutate/delete
  the source region after selection without changing the DC clip, `GetClipRgn`
  copies complex rect lists into the caller region, and null output handles are
  tolerated only when no clip exists. Unicorn DC diagnostics now report copied
  clip rect counts/bounds instead of a stale source handle.
- `src/emulator/unicorn.rs`: orphaned WNDPROC return stubs can recover through
  the latest saved return record when it carries a real guest return PC,
  preserving the guest caller return path when a pending return record was
  pruned too early.
- Focused validation after the clip-region copy/lifetime slice:
  `cargo fmt`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe
  coredll_raw_select_clip_rgn_copies_region_lifetime_like_ce`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe clip`, and `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test basic_subsystems resource_system` passed.
- Broader validation after the clip-region copy/lifetime slice:
  `cargo check --features unicorn,trace,win32-desktop` and full
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed. The first
  full run caught one stale direct resource-system assertion, which was updated
  before the successful rerun. Cargo still emits the existing unused-code
  warnings.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `IntersectClipRect`, `ExcludeClipRect`, and `GetClipBox` now follow CE
  `clip.cpp::passNull2ClipRegion` invalid-parameter ordering. Null or bad HDCs
  fail with `ERROR_INVALID_HANDLE`; `GetClipBox` on a valid HDC with a null
  output `RECT*` fails with `ERROR_INVALID_PARAMETER`; successful `GetClipBox`
  clears last error after writing the output rectangle.
- Focused validation after the clip entrypoint validation slice:
  `cargo fmt`, `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_clip_rect_entrypoints_validate_hdc_like_ce`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe clip` passed. Cargo still emits the existing unused-code
  warnings.
- Broader validation after the clip entrypoint validation slice and WNDPROC
  return-stub recovery adjustment: `cargo check --features
  unicorn,trace,win32-desktop`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop orphaned_wndproc_return_stub`, full
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, and full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The full suite still ignores the eVC4
  MIPSII fixture because that toolchain is not configured.
- `src/ce/resource.rs`, `src/ce/coredll.rs`, `src/ce/kernel.rs`, and
  `tests/coredll_raw_gwe.rs`: raw `SetViewportOrgEx` now stores real viewport
  origin state, returns the previous origin through `lpPoint`, and applies the
  selected-DIB viewport offset to `LineTo`, `Polyline`, and `Polygon` pixels
  like CE `draw.cpp::ViewPort`. Process exit cleanup also snapshots backing
  store restoration targets before window destruction so already-destroyed
  windows can still restore their captured framebuffer pixels.
- `src/emulator/unicorn.rs`: orphaned direct-send WNDPROC recovery now has a
  wider nested-stack grace window and a regression for a deeper live WNDPROC
  frame, so legitimate nested callouts are not pruned as stale recovery records.
- Focused validation after the viewport/backing-store/WNDPROC slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe
  coredll_raw_polyline_and_polygon_apply_viewport_origin_on_selected_dib`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_gdi_device_attribute_modes_follow_ce_sentinels`,
  and `cargo test -j 1 --features unicorn,trace,win32-desktop
  process_exit_restores_backing_store_for_already_destroyed_window` passed.
- Broader validation after the viewport/backing-store/WNDPROC slice:
  `cargo fmt --check`, `cargo check --features unicorn,trace,win32-desktop`,
  full `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`, full `cargo test -j 1 --features
  unicorn,trace,win32-desktop`, and `git diff --check` passed. Cargo still
  emits the existing unused-code warnings; one run also reported a transient
  `target\debug\incremental` finalize access-denied note, but all validation
  commands returned success. The full suite still ignores the eVC4 MIPSII
  fixture because that toolchain is not configured.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `OffsetRgn` now
  returns the region's post-offset CE status instead of treating every
  non-empty region as `SIMPLEREGION`. Complex difference regions retain their
  multi-rect hole geometry, move their bounds by the requested offset, continue
  to reject points in the shifted hole, and report `COMPLEXREGION` through both
  `OffsetRgn` and `GetRgnBox`.
- Focused validation after the `OffsetRgn` complex-status slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_combine_rgn_diff_preserves_holes` passed. Cargo
  still emits the existing unused-code warnings and the recurring
  `target\debug\incremental` finalize access-denied note, but the command
  returned success.
- Broader validation after the `OffsetRgn` complex-status slice and overlapping
  WNDPROC grace regression: `cargo fmt --check`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop direct_send_wndproc_cleanup_keeps_live_callout_until_return_pc`,
  `cargo check --features unicorn,trace,win32-desktop`, and full
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed. The full
  suite still ignores the eVC4 MIPSII fixture because that toolchain is not
  configured, and Cargo still emits the existing unused-code warnings plus the
  recurring `target\debug\incremental` finalize access-denied note.
- `src/ce/kernel.rs`, `src/ce/coredll.rs`, and `src/emulator/unicorn.rs`:
  window destruction now uses a shared framebuffer-aware helper that snapshots
  the target subtree before `WM_DESTROY`/destroy cleanup and then either
  restores captured backing-store pixels to the live framebuffer or discards
  the saved stores when no framebuffer is available. Raw `DestroyWindow`,
  Unicorn `DestroyWindow`, `DefWindowProcW(WM_CLOSE)`, default
  `CallWindowProcW(WM_CLOSE)`, and pending destroy finalization now share that
  path instead of duplicating or skipping backing-store cleanup.
- Focused validation after the framebuffer-aware destroy slice:
  `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop
  destroyed_window_restores_captured_framebuffer_backing_store`, `cargo test
  -j 1 --features unicorn,trace,win32-desktop
  destroy_window_with_framebuffer_restores_captured_backing_store`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe
  coredll_raw_send_message_timeout_writes_zero_result_when_target_destroyed`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe
  coredll_raw_destroy_parent_invalidates_children_and_purges_messages`, and
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_gwe_wm_parentnotify_on_create_and_destroy`
  passed.
- Broader validation after the framebuffer-aware destroy slice:
  `cargo check --features unicorn,trace,win32-desktop` and full
  `cargo test -j 1 --features unicorn,trace,win32-desktop` passed. The full
  suite still ignores the eVC4 MIPSII fixture because that toolchain is not
  configured, and Cargo still emits the existing unused-code warnings plus the
  recurring `target\debug\incremental` finalize access-denied note.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw selected-DIB
  `Rectangle`, `Ellipse`, and `RoundRect` now apply the DC viewport origin like
  CE `draw.cpp::ViewPort`, sharing the existing viewport-origin helper with
  line/polyline/polygon drawing and preserving the zero-rounding `RoundRect`
  fallback through `Rectangle`.
- Focused validation after the shape viewport-origin slice:
  `cargo fmt --check` and `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_shapes_apply_viewport_origin_on_selected_dib` passed. Cargo still
  emits the existing unused-code warnings and the recurring
  `target\debug\incremental` finalize access-denied note, but the test command
  returned success.
- Broader validation after the shape viewport-origin slice:
  `cargo check --features unicorn,trace,win32-desktop`, `git diff --check`,
  and full `cargo test -j 1 --features unicorn,trace,win32-desktop` passed.
  The full suite still ignores the eVC4 MIPSII fixture because that toolchain
  is not configured, and Cargo still emits the existing unused-code warnings
  plus the recurring `target\debug\incremental` finalize access-denied note.
- `src/ce/coredll_ordinals.rs`, `src/ce/resource.rs`, `src/ce/coredll.rs`,
  and `tests/coredll_raw_gwe.rs`: CE origin/ext ordinals from
  `core_common.def`/`coredll.def` are now exported for `SetWindowOrgEx @1984`,
  `GetWindowOrgEx @1985`, `GetWindowExtEx @1986`,
  `OffsetViewportOrgEx @1987`, `GetViewportOrgEx @1988`, and
  `GetViewportExtEx @1989`. Raw DC state now stores window origin separately,
  `OffsetViewportOrgEx` reports the previous viewport origin, `Get*ExtEx`
  reports selected-DIB extents, and selected-DIB drawing applies the combined
  viewport plus CE window-origin translation.
- Focused validation after the origin/ext ordinal slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_origin_apis_follow_ce_viewport_contract` passed.
  Cargo still emits the existing unused-code warnings and the recurring
  `target\debug\incremental` finalize access-denied note.
- Broader validation after the origin/ext ordinal slice:
  `cargo fmt --check`, `cargo test -j 1 --features unicorn,trace,win32-desktop
  --test coredll_dispatch`, `cargo check --features unicorn,trace,win32-desktop`,
  `git diff --check`, and full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The full suite still ignores the eVC4
  MIPSII fixture because that toolchain is not configured, and Cargo still
  emits the existing unused-code warnings plus the recurring
  `target\debug\incremental` finalize access-denied note.
- `src/emulator/unicorn.rs`: an overlapping WNDPROC guard now declines
  `CallWindowProcW` guest callouts that would re-enter through an orphaned
  WNDPROC return stub without an enclosing pending frame, with focused
  validation from `cargo test -j 1 --features unicorn,trace,win32-desktop
  call_window_proc_declines_orphaned_wndproc_return_stub`.
- `tests/coredll_raw_gwe.rs`: selected-DIB `Polygon` and `Polyline` now have
  CE `draw.cpp::SimpleClipRgnTest0`-style clip-region containment coverage:
  drawing spans outside a selected rectangular clip, and pixels immediately
  outside the clip remain unchanged while in-clip pixels render.
- Focused validation after the selected-DIB polygon/polyline clip-region slice:
  `cargo fmt --check` and `cargo test -j 1 --features
  unicorn,trace,win32-desktop
  coredll_raw_polygon_and_polyline_respect_selected_clip_region_on_memory_dib`
  passed. Cargo still emits the existing unused-code warnings and recurring
  `target\debug\incremental` finalize access-denied note, but the test command
  returned success.
- Broader validation after the selected-DIB polygon/polyline clip-region slice:
  `cargo check --features unicorn,trace,win32-desktop`, `git diff --check`,
  and full `cargo test -j 1 --features unicorn,trace,win32-desktop` passed.
  The full suite still ignores the eVC4 MIPSII fixture because that toolchain
  is not configured, and Cargo still emits the existing unused-code warnings
  plus the recurring `target\debug\incremental` finalize access-denied note.
- `src/ce/kernel.rs` and `src/ce/coredll.rs`: framebuffer-backed
  `ShowWindow` and `SetWindowPos(SWP_HIDEWINDOW)` now use the captured
  backing-store restoration path when hiding visible windows, with no-framebuffer
  calls discarding stale saved backing instead of leaving it live.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw two-point
  selected-DIB `Polygon` now follows CE `draw.cpp::ShapeColorTest(EPolygon)`
  by stroking the single segment once. This fixes `R2_XORPEN` cancellation from
  drawing the reverse closing segment over the same pixels.
- Focused validation after the recovered hide/backing-store and two-point
  polygon ROP2 slice: `cargo fmt --check`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop framebuffer_backing_store`, and `cargo test -j 1
  --features unicorn,trace,win32-desktop
  coredll_raw_two_point_polygon_applies_rop2_xorpen_once_on_selected_memory_dib`
  passed. Cargo still emits the existing unused-code warnings and recurring
  `target\debug\incremental` finalize access-denied note, but both test
  commands returned success.
- Broader validation after the recovered hide/backing-store and two-point
  polygon ROP2 slice: `cargo check --features unicorn,trace,win32-desktop`,
  `git diff --check`, and full `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The full suite still ignores the eVC4
  MIPSII fixture because that toolchain is not configured, and Cargo still
  emits the existing unused-code warnings plus the recurring
  `target\debug\incremental` finalize access-denied note.
- `tests/coredll_raw_gwe.rs`: selected-DIB `Rectangle` and `RoundRect` now
  have CE `draw.cpp::ShapeColorTest`-style `R2_XORPEN` outline coverage with a
  `NULL_BRUSH`, proving outline pixels combine the selected pen with existing
  destination pixels while interiors remain untouched.
- Focused validation after the selected-DIB `Rectangle`/`RoundRect` ROP2 slice:
  `cargo fmt --check` and `cargo test -j 1 --features
  unicorn,trace,win32-desktop
  coredll_raw_rectangle_and_roundrect_apply_rop2_xorpen_on_selected_memory_dib`
  passed. Cargo still emits the existing unused-code warnings and recurring
  `target\debug\incremental` finalize access-denied note, but the test command
  returned success.
- Follow-up validation while recovering the UI-drive run: `cargo fmt`, `cargo
  fmt --check`, `git diff --check`, the focused selected-DIB
  `Rectangle`/`RoundRect` ROP2 test, two focused orphaned WNDPROC recovery
  tests, and `cargo build --release --features unicorn` passed. Cargo still
  emits the existing unused-code warnings.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: bitmap-backed
  `DrawIconEx` now preserves the icon bitmap's native dimensions when callers
  pass zero width/height, matching the CE `winuser.h` `DrawIcon` macro and
  CE `imagelist.cpp` `DrawIconEx_I(..., 0, 0, DI_NORMAL)` image-storage path.
  The selected-DIB regression proves a 2x2 icon stays 2x2 instead of expanding
  to the synthetic 32-pixel shell-icon fallback.
- Focused validation after the bitmap-backed `DrawIconEx` native-size slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_destroy_icon_accepts_loaded_icon_handles` passed.
  Cargo still emits the existing unused-code warnings and recurring
  `target\debug\incremental` finalize access-denied note, but the test command
  returned success.
- Broader validation after the bitmap-backed `DrawIconEx` native-size slice:
  `cargo fmt`, `cargo fmt --check`, `git diff --check`, `CARGO_INCREMENTAL=0
  cargo check --features unicorn,trace,win32-desktop`, and `CARGO_INCREMENTAL=0
  cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe` passed. The full raw GWE test binary reported 349 passing
  tests, and Cargo still emits the existing unused-code warnings.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: `GetClipBox` now
  reports `COMPLEXREGION` for selected complex clip regions using the stored
  clip rect list instead of collapsing to the bounding rectangle's simple
  status. The CE `clip.cpp::SelectComplexTest`-style regression selects a
  `CombineRgn(RGN_DIFF)` complex region, mutates and deletes the source region,
  then verifies `GetClipRgn`, `GetRgnBox`, and `GetClipBox` still expose the
  original complex clip.
- Focused validation after the complex `SelectClipRgn`/`GetClipBox` slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe
  coredll_raw_select_clip_rgn_copies_complex_region_lifetime_like_ce` passed.
  Cargo still emits the existing unused-code warnings and recurring
  `target\debug\incremental` finalize access-denied note, but the test command
  returned success.
- Broader validation after the complex `SelectClipRgn`/`GetClipBox` slice:
  `cargo fmt`, `cargo fmt --check`, `git diff --check`, `CARGO_INCREMENTAL=0
  cargo check --features unicorn,trace,win32-desktop`, and `CARGO_INCREMENTAL=0
  cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe` passed. The full raw GWE test binary reported 350 passing
  tests, and Cargo still emits the existing unused-code warnings.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw
  `IntersectClipRect`/`ExcludeClipRect` now use the implicit HDC surface when
  no clip region has been selected, matching the CE `clip.cpp` fixture shape
  where clip-rect operations are bounded by the drawable DC extent. The
  regression covers an oversized intersect clipping back to the default
  800x480 surface and an exclude-from-implicit-surface call producing a complex
  stored clip region.
- Focused validation after the implicit clip-surface slice: `cargo test -j 1
  --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_intersect_clip_rect_and_exclude_clip_rect_update_dc_clip`
  passed. Cargo still emits the existing unused-code warnings and recurring
  `target\debug\incremental` finalize access-denied note, but the test command
  returned success.
- `src/remote_server.rs`: the remote location endpoint now accepts
  `latitude`/`longitude` aliases in addition to `lat`/`lon`, normalizing the
  queued control payload back to the canonical `lat`/`lon` fields used by the
  CE remote sensor path.
- Validation after the implicit clip-surface and remote location-alias slices:
  `cargo fmt`, `cargo fmt --check`, `git diff --check`,
  `CARGO_INCREMENTAL=0 cargo check --features unicorn,trace,win32-desktop`,
  `CARGO_INCREMENTAL=0 cargo test -j 1 --features unicorn,trace,win32-desktop
  --test coredll_raw_gwe`, focused `remote_server_normalizes_location_coordinate_aliases`,
  and full `CARGO_INCREMENTAL=0 cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The eVC4 MIPSII fixture remains ignored
  because that toolchain is not configured, and Cargo still emits the existing
  unused-code warnings.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `GetPixel` now reads
  selected memory-DIB and framebuffer pixels instead of returning unconditional
  black, and the CE `draw.cpp::DrawEdgeTest2/3` regression verifies
  `BF_MIDDLE | BF_RECT` fills the center with `COLOR_BTNFACE` while
  `BF_FLAT | BF_RECT` preserves a white center.
- Focused validation after the `DrawEdgeTest2/3` and `GetPixel` slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_draw_edge_matches_ce_middle_and_flat_center_pixels`
  passed. Cargo still emits the existing unused-code warnings.
- Broader validation after the `DrawEdgeTest2/3` source-reference refresh:
  `cargo fmt`, `cargo fmt --check`, `git diff --check`,
  `CARGO_INCREMENTAL=0 cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full
  `CARGO_INCREMENTAL=0 cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The eVC4 MIPSII fixture remains ignored
  because that toolchain is not configured, and Cargo still emits the existing
  unused-code warnings.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `DrawEdge`
  `BF_ADJUST` now shrinks only the sides requested by the edge mask, matching
  CE public callers such as `updownview.cpp::SetSunkenBorder` and
  `atlctrlx.h::DrawPaneTitle` that use top/bottom or three-sided adjusted
  borders. The regression verifies `BF_TOP | BF_BOTTOM | BF_ADJUST` mutates
  only the rectangle's top and bottom while leaving an unrequested left-side
  pixel untouched.
- Validation after the partial-edge `BF_ADJUST` refresh: `cargo fmt`,
  `cargo fmt --check`, `git diff --check`, focused `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_draw_edge_adjusts_only_requested_edges -- --nocapture`,
  `CARGO_INCREMENTAL=0 cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full
  `CARGO_INCREMENTAL=0 cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE binary reported 352 passing
  tests, the eVC4 MIPSII fixture remains ignored because that toolchain is not
  configured, and Cargo still emits the existing unused-code warnings.
- `src/ce/coredll.rs` and `tests/coredll_raw_gwe.rs`: raw `DrawEdge`
  `BF_DIAGONAL_END*` flags now paint a clipped diagonal line instead of
  returning success as a no-op. The slice is backed by CE `draw.cpp`
  `IterateDrawEdge` bounded-draw coverage and public tab/trackbar callers that
  use diagonal endpoint flags for visible slanted edges. The regression verifies
  `BF_DIAGONAL | BF_TOP | BF_RIGHT` changes an in-rect endpoint pixel while an
  adjacent outside-rect pixel remains untouched.
- Validation after the `BF_DIAGONAL_END*` DrawEdge slice: `cargo fmt`,
  focused `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_draw_edge_diagonal_paints_within_requested_rect
  -- --nocapture`, `cargo fmt --check`, `git diff --check`,
  `CARGO_INCREMENTAL=0 cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full
  `CARGO_INCREMENTAL=0 cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE binary reported 353 passing
  tests, the eVC4 MIPSII fixture remains ignored because that toolchain is not
  configured, and Cargo still emits the existing unused-code warnings.
- `src/ce/gwe.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: CE topmost windows now remain in
  a front z-order group, normal `HWND_TOP` promotion stays below existing
  topmost siblings, and raw `SetWindowPos(HWND_TOPMOST/HWND_NOTOPMOST)` mutates
  the stored `WS_EX_TOPMOST` extended style while moving the window between the
  topmost and normal groups. This supports `MessageBoxW(MB_TOPMOST)` live modal
  ordering and other CE callers that use topmost status windows or taskbars.
- Validation after the GWE topmost z-order slice: `cargo fmt` and focused
  `CARGO_INCREMENTAL=0 cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_set_window_pos_respects_topmost_z_order_group`, `cargo fmt
  --check`, `git diff --check`, full `CARGO_INCREMENTAL=0 cargo test -j 1
  --features unicorn,trace,win32-desktop --test coredll_raw_gwe`, and full
  `CARGO_INCREMENTAL=0 cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw GWE binary reported 354 passing
  tests, the eVC4 MIPSII fixture remains ignored because that toolchain is not
  configured, and Cargo still emits the existing unused-code warnings.
- `src/ce/coredll.rs` and `tests/coredll_raw_kernel.rs`: raw
  `GetCommModemStatus` now reports asserted CTS, DSR, and RLSD modem-status
  bits for valid serial device handles instead of returning an all-zero status.
  This keeps the remote GPS serial path generic while matching CE callers that
  wait for carrier/control-line readiness before trusting `ReadFile` or
  `WaitCommEvent`.
- Focused validation after the serial modem-status slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_kernel coredll_raw_comm_state_mask_wait_and_purge_are_stateful
  -- --nocapture` passed. Cargo still emits the existing unused-code warnings.
- `src/emulator/unicorn.rs`: escaped cross-thread visible-message WNDPROC
  callouts are now cleared when execution has already reached the saved
  `resume_import` return PC, covering the live iNavi touch path where
  `WM_LBUTTONDOWN/UP` reached the AFX window but the pending
  `OrphanedVisibleMessage` record stayed live after the guest bypassed the
  synthetic WNDPROC return stub.
- Focused validation after the escaped visible-message WNDPROC slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop
  cross_thread_visible_message_requires_mapped_wndproc_before_handoff --
  --nocapture` passed. Cargo still emits the existing unused-code warnings.
- `src/ce/coredll.rs` and `tests/coredll_raw_kernel.rs`: `SHGetFileInfoW`
  now distinguishes CE small and large system image lists. Calls with
  `SHGFI_SMALLICON` keep the 16x16 pseudo-list handle, while calls without it
  return a separate 32x32 large-list handle that participates in image-list
  count, icon-size, icon, image-info, background-color, and destroy semantics.
- Focused validation after the shell system image-list slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_kernel sh_get_file_info_system_image_list_supports_icon_queries_and_draw
  -- --nocapture` passed. Cargo still emits the existing unused-code warnings.
- Broader validation after the escaped visible-message WNDPROC and shell
  system image-list slices: `cargo fmt`, focused `CARGO_INCREMENTAL=0 cargo
  test -j 1 --features unicorn,trace,win32-desktop
  cross_thread_visible_message_requires_mapped_wndproc_before_handoff --
  --nocapture`, full `CARGO_INCREMENTAL=0 cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_kernel`, and full
  `CARGO_INCREMENTAL=0 cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The raw kernel binary reported 75
  passing tests, the eVC4 MIPSII fixture remains ignored because that toolchain
  is not configured, and Cargo still emits the existing unused-code warnings.
- `tests/coredll_raw_memory_file.rs`: aligned the older raw serial control
  ordinal fixture with the CE-backed `GetCommModemStatus` behavior so it now
  expects asserted CTS, DSR, and RLSD bits rather than the former zero-status
  placeholder.
- Validation after aligning the memory/file modem-status fixture: `cargo fmt`,
  focused `CARGO_INCREMENTAL=0 cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_memory_file
  coredll_raw_serial_comm_control_ordinals_accept_valid_device_handle --
  --nocapture`, and full `CARGO_INCREMENTAL=0 cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_memory_file` passed. The
  raw memory/file binary reported 108 passing tests, and Cargo still emits the
  existing unused-code warnings.
- Full validation after the memory/file modem-status expectation cleanup:
  `cargo fmt --check`, `git diff --check`, and `CARGO_INCREMENTAL=0 cargo test
  -j 1 --features unicorn,trace,win32-desktop` passed. The eVC4 MIPSII fixture
  remains ignored because that toolchain is not configured, and Cargo still
  emits the existing unused-code warnings.
- `tests/coredll_raw_gwe.rs` and `SOURCE_REFERENCES.md`: extended
  the CE `draw.cpp::NegativeSize(EMaskBlt)`/`swblt.cpp` signed-extent coverage
  with a framebuffer HDC regression that uses a 1 bpp mask, a negative
  destination width, and verifies mirrored RGB565 pixels plus the resulting
  dirty rectangle.
- Focused validation after the framebuffer `MaskBlt` signed-extent slice:
  `cargo fmt --check` and `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_mask_blt_mirrors_negative_destination_width_to_framebuffer`
  passed. Cargo still emits the existing unused-code warnings.
- Full validation after the framebuffer `MaskBlt` signed-extent slice:
  `git diff --check`, `cargo fmt --check`, and `cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The eVC4 MIPSII fixture remains ignored
  because that toolchain is not configured, and Cargo still emits the existing
  unused-code warnings plus the intermittent incremental cleanup access-denied
  note under `target/debug/incremental`.
- `tests/coredll_raw_gwe.rs`, `PLAN.MD`, and `SOURCE_REFERENCES.md`: extended
  CE `draw.cpp::ClipBitBlt(EMaskBlt)` coverage with off-left clipping
  regressions for selected-memory-DIB and framebuffer HDC destinations. Both
  use a non-copy ROP4 plus a 1 bpp mask to prove clipped-away destination pixels
  still advance the source and mask coordinates before the visible pixels are
  rendered.
- Focused validation after the `MaskBlt` clip-alignment slice: `cargo fmt
  --check` and `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_mask_blt_clips_off_left` passed. Cargo still
  emits the existing unused-code warnings.
- Full validation after the `MaskBlt` clip-alignment slice: `git diff
  --check`, `cargo fmt --check`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe`, and `cargo test -j 1
  --features unicorn,trace,win32-desktop` passed. The raw GWE binary reported
  364 passing tests, the eVC4 MIPSII fixture remains ignored because that
  toolchain is not configured, and Cargo still emits the existing unused-code
  warnings.
- `src/ce/coredll.rs`, `src/emulator/imports.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: `FSDMGR_DiskIoControl`
  delete-sector, secure-wipe, and set-secure-wipe-flag payload validation now
  requires the exact CE `sizeof(DELETE_SECTOR_INFO)` byte count before reading
  or mutating sparse synthetic disk sectors, matching `diskio.h` plus the
  MSFLASH `falmain.cpp` parameter gate.
- Validation after the exact `DELETE_SECTOR_INFO` size slice: focused
  `CARGO_INCREMENTAL=0 cargo test -j 1 --features
  unicorn,trace,win32-desktop
  fsdmgr_disk_support_imports_round_trip_sparse_sectors_and_info -- --nocapture`,
  `cargo fmt --check`, `git diff --check`, `CARGO_INCREMENTAL=0 cargo check
  --features unicorn,trace,win32-desktop`, and full `CARGO_INCREMENTAL=0 cargo
  test -j 1 --features unicorn,trace,win32-desktop` passed. The eVC4 MIPSII
  fixture remains ignored because that toolchain is not configured, and Cargo
  still emits the existing unused-code warnings.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: raw
  `MsgWaitForMultipleObjectsEx` now records unreadable handle-array failures
  as message-wait failures after `read_guest_u32` sets
  `ERROR_INVALID_PARAMETER`, preserving the CE NETUI thunk/wrapper shape where
  caller handle arrays are copied and the forwarded return code/last-error pair
  remains visible.
- Focused validation after the message-wait bad handle-array diagnostics slice:
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_msgwait_bad_handle_pointer_records_failed_msgwait -- --nocapture`
  passed. Cargo still emits the existing unused-code warnings.
- Full validation after the message-wait bad handle-array diagnostics slice:
  `git diff --check`, `cargo fmt --check`,
  `$env:CARGO_INCREMENTAL='0'; cargo check --features
  unicorn,trace,win32-desktop`, and `$env:CARGO_INCREMENTAL='0'; cargo test
  -j 1 --features unicorn,trace,win32-desktop` passed. The eVC4 MIPSII
  fixture remains ignored because that toolchain is not configured, and Cargo
  still emits the existing unused-code warnings.
- `src/emulator/unicorn.rs`: direct same-thread `SendMessageW` WNDPROC cleanup
  now keeps bounded nested callouts even when the saved frame has dispatched
  from MFC into a helper DLL. The live iNavi trace matched this shape with
  pending `0x5237` callouts and a stack delta inside
  `WNDPROC_NESTED_STACK_GRACE_BYTES`, but the previous module check archived the
  callout and forced orphaned return-stub recovery.
- Validation after the direct-send WNDPROC cleanup slice: `cargo fmt`, focused
  `cargo test -j 1 --features unicorn,trace,win32-desktop
  direct_send_wndproc_cleanup_keeps_live_callout_until_return_pc --
  --nocapture`, `cargo test -j 1 --features unicorn,trace,win32-desktop
  fsdmgr_disk_support_imports_round_trip_sparse_sectors_and_info --
  --nocapture`, and `cargo test -j 1 --features
  unicorn,trace,win32-desktop
  coredll_raw_msgwait_bad_handle_pointer_records_failed_msgwait --
  --nocapture` passed. Cargo still emits the existing unused-code warnings and
  the intermittent Windows incremental-cache cleanup note.
- `src/ce/coredll.rs`, `src/ce/kernel.rs`, `src/emulator/imports.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: direct
  `FSDMGR_DiskIoControl(IOCTL_DISK_FLUSH_CACHE)` is now named from CE
  `diskio.h` and covered as a successful no-op on synthetic sparse disks,
  matching the optional flush-forwarding behavior in CE `fsdcache.cpp` and
  `nullcache.cpp`.
- `src/emulator/unicorn.rs`: fixed the full feature test build by cloning the
  reused synthetic `MipsGuestContext` in the direct-send WNDPROC cleanup test
  fixture before the first `SavedCpuContext` consumes it.
- Focused validation after the FSDMGR disk flush-cache and test-fixture slice:
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop
  fsdmgr_disk_support_imports_round_trip_sparse_sectors_and_info --
  --nocapture` passed. Cargo still emits the existing unused-code warnings.
- Full validation after the FSDMGR disk flush-cache and test-fixture slice:
  `git diff --check`, `cargo fmt --check`,
  `$env:CARGO_INCREMENTAL='0'; cargo check --features
  unicorn,trace,win32-desktop`, and `$env:CARGO_INCREMENTAL='0'; cargo test
  -j 1 --features unicorn,trace,win32-desktop` passed. The eVC4 MIPSII
  fixture remains ignored because that toolchain is not configured, and Cargo
  still emits the existing unused-code warnings.
- `src/ce/coredll.rs`, `src/ce/kernel.rs`, `tests/coredll_raw_gwe.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`:
  raw `ExtEscape` now covers the DeviceEmulator display
  `SETBACKLIGHT`/`GETBACKLIGHT` escapes from CE `display_escapes.h` and
  `s3c2410x_lcd.cpp`, with local BOOL state, fixed four-byte input/output
  buffer validation, and invalid-size no-touch coverage.
- Focused validation after the DeviceEmulator backlight `ExtEscape` slice:
  `cargo fmt --check` and `$env:CARGO_INCREMENTAL='0'; cargo test -j 1
  --features unicorn,trace,win32-desktop
  coredll_raw_ext_escape_matches_ce_query_and_protected_escape_edges --
  --nocapture` passed. Cargo still emits the existing unused-code warnings.
- `src/emulator/types.rs` and `src/emulator/unicorn.rs`: preserved the sender
  thread id on archived `UnicornWndProcReturn` records so orphaned
  SendMessage-owned WNDPROC returns can keep their send-depth owner protected
  during cleanup, and fixed the newly required initializers/predicate parsing
  so the full feature build stays green.
- Full validation after the DeviceEmulator backlight `ExtEscape` slice and
  overlapping WNDPROC send-depth fix: `cargo fmt --check`, `git diff --check`,
  `$env:CARGO_INCREMENTAL='0'; cargo check --features
  unicorn,trace,win32-desktop`, and `$env:CARGO_INCREMENTAL='0'; cargo test
  -j 1 --features unicorn,trace,win32-desktop` passed. The eVC4 MIPSII
  fixture remains ignored because that toolchain is not configured, and Cargo
  still emits the existing unused-code warnings.
- `src/ce/coredll.rs`, `src/ce/kernel.rs`, `tests/coredll_raw_gwe.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`:
  raw `ExtEscape` now covers the DeviceEmulator display `CONTRASTCOMMAND`
  escape from CE `pwingdi.h` and
  `PLATFORM\DEVICEEMULATOR\SRC\DRIVERS\DISPLAY\LCD\s3c2410x_lcd.cpp`.
  The local display model now tracks the LCDCON3 high nibble shared by
  contrast and backlight, supports the `ContrastCmdInputParm` get/set/increase/
  decrease/default/max commands, clamps signed set values to 0..15, accepts
  null output for state-changing calls, preserves CE's default-command return
  quirk, and tests the resulting backlight-bit coupling.
- Focused validation after the DeviceEmulator contrast `ExtEscape` slice:
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop
  coredll_raw_ext_escape_matches_ce_query_and_protected_escape_edges --
  --nocapture` passed. Cargo still emits the existing unused-code warnings.
- Full validation after the DeviceEmulator contrast `ExtEscape` slice:
  `cargo fmt --check`, `git diff --check`,
  `$env:CARGO_INCREMENTAL='0'; cargo check --features
  unicorn,trace,win32-desktop`, and `$env:CARGO_INCREMENTAL='0'; cargo test
  -j 1 --features unicorn,trace,win32-desktop` passed. The eVC4 MIPSII
  fixture remains ignored because that toolchain is not configured, and Cargo
  still emits the existing unused-code warnings.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: raw `ExtEscape` now covers
  DeviceEmulator `GETRAWFRAMEBUFFER` metadata from CE `gxdma.h` and
  `image_cfg.h`, including the `QUERYESCSUPPORT` response, 28-byte
  `RawFrameBufferInfo` output, RGB565/16 bpp fields, uncached framebuffer
  base, x/y strides, dimensions, and invalid-output no-touch behavior. The
  remaining framebuffer-access gap is guest-readable direct memory behind the
  returned pointer.
- Focused validation after the DeviceEmulator raw-framebuffer `ExtEscape`
  slice: `cargo fmt --check`, `git diff --check`, `cargo test -j 1
  --features unicorn,trace,win32-desktop
  coredll_raw_ext_escape_matches_ce_query_and_protected_escape_edges --test
  coredll_raw_gwe -- --nocapture`, and `cargo build --release --features
  unicorn,win32-desktop` passed. Cargo still emits the existing unused-code
  warnings.
- `src/ce/coredll.rs`, `src/emulator/imports.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: fsdmgr import traps now cover
  the CE `FSDMGR_GetDiskInfo @16` and `FSDMGR_GetDiskName @17` exports from
  `fsdmgr.def`/`fsdmgrapi.cpp`, routing them through the synthetic disk
  metadata/name contract already used by `FSDMGR_DiskIoControl` while keeping
  the broader physical block-driver forwarding gap open.
- Focused validation after the direct fsdmgr disk metadata/name import slice:
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop
  fsdmgr_direct_disk_info_and_name_imports_use_ce_metadata_contract --
  --nocapture` and `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop patches_supported_fsdmgr_imports_only --
  --nocapture` passed. Cargo still emits the existing unused-code warnings.
- Full validation after the direct fsdmgr disk metadata/name import slice:
  `cargo fmt --check`, `git diff --check`,
  `$env:CARGO_INCREMENTAL='0'; cargo check --features
  unicorn,trace,win32-desktop`, and `$env:CARGO_INCREMENTAL='0'; cargo test
  -j 1 --features unicorn,trace,win32-desktop` passed. The eVC4 MIPSII
  fixture remains ignored because that toolchain is not configured, and Cargo
  still emits the existing unused-code warnings.
- `src/ce/coredll.rs`, `src/emulator/imports.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: fsdmgr import traps now cover
  CE `FSDMGR_DeviceHandleToHDSK @11` from `storemain.cpp` as an identity
  handle-to-disk cast, plus CE `FSDMGR_FormatVolume @15` and
  `FSDMGR_ScanVolume @31` from `fsdmgrapi.cpp` as the no-configured-`Util`
  utility DLL failure status for synthetic disks. Real utility DLL lookup and
  execution remains queued with the broader physical block-driver fidelity
  work.
- `src/ce/coredll.rs`, `src/emulator/imports.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: fsdmgr import traps now cover
  CE `FSDMGR_AsyncEnterVolume @80` and `FSDMGR_AsyncExitVolume @81` from
  `fsdmgr.def`/`fsdmgrapi.cpp`. The host-backed model validates registered
  HVOLs, returns `ERROR_DEVICE_REMOVED` for missing volumes, writes a synthetic
  HVOL lock token to the caller outputs, rejects bad output/mismatched lock
  state with `ERROR_INVALID_PARAMETER`, and keeps the deeper CE
  `MountedVolume_t::Enter/Exit` availability, powerdown, and thread-exit wait
  reference behavior queued.
- Focused validation after the direct fsdmgr async-volume import slice:
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop
  fsdmgr_async_volume_imports_lock_registered_hvol_shape -- --nocapture` and
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop patches_supported_fsdmgr_imports_only --
  --nocapture` passed. Cargo still emits the existing unused-code warnings.
- Full validation after the direct fsdmgr async-volume import slice:
  `cargo fmt --check`, `git diff --check`,
  `$env:CARGO_INCREMENTAL='0'; cargo check --features
  unicorn,trace,win32-desktop`, and `$env:CARGO_INCREMENTAL='0'; cargo test
  -j 1 --features unicorn,trace,win32-desktop` passed. The eVC4 MIPSII
  fixture remains ignored because that toolchain is not configured, and Cargo
  still emits the existing unused-code warnings.
- `src/ce/coredll.rs`, `src/emulator/imports.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: fsdmgr import traps now cover
  CE `FSDMGR_ParseSecurityDescriptor @82` from
  `fsdmgr.def`/`fsdmgrapi.cpp` with the private `aclpriv.h::SDSize`
  `SECDESHDR.cbSize` rule. The raw trap writes null/zero for null
  `SECURITY_ATTRIBUTES`, validates 12-byte `SECURITY_ATTRIBUTES`,
  `bInheritHandle == FALSE`, and kernel-mode descriptor pointers, returns
  `ERROR_INVALID_SECURITY_DESCR` for malformed descriptors, and leaves broader
  file-security ACL storage/enforcement queued.
- Reset recovery validation for the fsdmgr security-descriptor parser slice:
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop
  fsdmgr_parse_security_descriptor_import_uses_ce_secdeschdr_size --
  --nocapture`, `cargo fmt --check`, `git diff --check`,
  `$env:CARGO_INCREMENTAL='0'; cargo check --features
  unicorn,trace,win32-desktop`, and `$env:CARGO_INCREMENTAL='0'; cargo test
  -j 1 --features unicorn,trace,win32-desktop` passed. Cargo still emits the
  existing unused-code warnings.
- Live iNavi drive checkpoint after the reset recovery: the host process stayed
  alive, touch injection posted `WM_LBUTTONDOWN/WM_LBUTTONUP` to the full-screen
  `afx:10000:3:0:b5000:0` iNavi window, and no message boxes were live. GPS
  injection was accepted by the remote endpoint, but `COM7:` was not open in
  the guest and the queued serial buffer grew, so the sensor path is waiting on
  the app to reach its GPS consumer. The app remained on the SE splash while
  active guest code continued reading `\SDMMC Disk\INavi\res\resmapi_800x480.bin`.
- `src/ce/coredll.rs`, `src/emulator/imports.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: fsdmgr import traps now cover
  CE `FSDMGR_GetRegistryFlag @18`, `FSDMGR_GetRegistryString @19`, and
  `FSDMGR_GetRegistryValue @20` from `fsdmgr.def`, `fsdmgrapi.cpp`,
  `logicaldisk.cpp`, and `fsdhelper.cpp`. The synthetic-disk baseline follows
  the CE missing-value surface by returning false, setting `ERROR_FILE_NOT_FOUND`,
  clearing missing DWORD/string outputs, and leaving flags unchanged; null disk
  pointers return `ERROR_GEN_FAILURE`. Real logical-disk registry-root lookup
  and cache DLL/filter behavior remain queued.
- Focused validation after the FSDMGR registry-helper baseline:
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop
  fsdmgr_registry_imports_clear_missing_outputs_like_ce -- --nocapture` and
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop patches_supported_fsdmgr_imports_only --
  --nocapture` passed. Cargo still emits the existing unused-code warnings.
- Full validation after the FSDMGR registry-helper baseline:
  `cargo fmt --check`, `git diff --check`, `$env:CARGO_INCREMENTAL='0';
  cargo check --features unicorn,trace,win32-desktop`, and
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The eVC4 MIPSII fixture remains ignored,
  and Cargo still emits the existing unused-code warnings.
- `src/ce/coredll.rs`, `src/emulator/imports.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: fsdmgr import traps now cover
  CE `FSDMGR_AdvertiseInterface @2` from `fsdmgr.def`/`fsdmgrapi.cpp`. The
  direct FSDMGR import resolves by name and ordinal and now records/removes
  device-interface advertisements through the shared coredll
  `AdvertiseInterface` path.
- Validation after the FSDMGR advertise-interface import slice:
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop
  fsdmgr_advertise_interface_import_publishes_and_removes_device_interface --
  --nocapture`,
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop patches_supported_fsdmgr_imports_only --
  --nocapture`, `cargo fmt --check`, `git diff --check`,
  `$env:CARGO_INCREMENTAL='0'; cargo check --features
  unicorn,trace,win32-desktop`, and `$env:CARGO_INCREMENTAL='0'; cargo test
  -j 1 --features unicorn,trace,win32-desktop` passed. The eVC4 MIPSII fixture
  remains ignored, and Cargo still emits the existing unused-code warnings.
- `src/ce/file.rs`, `src/ce/kernel.rs`, `tests/basic_subsystems.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`:
  configured mounted-storage `IClass` advertisements now carry an owner token
  derived from the mount root. The public GUID/name advertisement table remains
  unique for `EnumDeviceInterfaces`, but duplicate owners keep a shared
  advertisement alive until the last owner withdraws, so unmounting one mount no
  longer sends a premature `DEVDETAIL` detach while another matching
  `\StoreMgr\<device>` advertisement remains active. Real device-manager
  `fsdev_t` handles plus `IClass` `GUID=name` and `%d`/`%b`/`%l` substitution
  parsing remain queued.
- Focused validation after the mounted `IClass` owner-scope slice:
  `cargo fmt` and `cargo test -j 1 --features unicorn,trace,win32-desktop
  mount_iclass_duplicate_advertisements_are_owner_scoped -- --nocapture`
  passed. Cargo still emits the existing unused-code warnings; one incremental
  cache directory under `target/` reported an access-denied finalization warning
  after the test completed.
- Full validation after the mounted `IClass` owner-scope slice:
  `cargo fmt --check`, `git diff --check`, `cargo test -j 1 --features
  unicorn,trace,win32-desktop mount_iclass -- --nocapture`, and
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The eVC4 MIPSII fixture remains ignored,
  and Cargo still emits the existing unused-code warnings.
- `src/ce/file.rs`, `src/ce/kernel.rs`, `tests/basic_subsystems.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`:
  configured mounted-storage `IClass` parsing now accepts CE devcore-style
  `GUID=name` entries in addition to the FSDMGR plain-GUID form. Plain GUIDs
  still publish `\StoreMgr\<device>`, explicit names publish the supplied
  address, `%d` publishes `$device\<device>`, `%l` publishes the legacy device
  name, empty explicit names are skipped, and a later `bus_name` follow-up covers
  configured `%b` advertisements while real `fsdev_t` bus/open-callback scoping
  remains queued.
- Focused validation after the mounted `IClass` name-parsing slice:
  `cargo fmt` and `cargo test -j 1 --features unicorn,trace,win32-desktop
  mount_iclass -- --nocapture` passed. Cargo still emits the existing
  unused-code warnings.
- Full validation after the mounted `IClass` name-parsing slice:
  `cargo fmt --check`, `git diff --check`, and
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The eVC4 MIPSII fixture remains ignored,
  and Cargo still emits the existing unused-code warnings.
- `src/ce/kernel.rs`, `src/ce/coredll.rs`, `src/emulator/imports.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: direct
  FSDMGR disk IOCTL handling now persists validated CE `DISK_INFO` payloads
  from `IOCTL_DISK_SETINFO`/`DISK_IOCTL_SETINFO` and reports the updated
  synthetic disk metadata through later GETINFO calls. Real physical
  block-driver SETINFO forwarding remains queued.
- Focused validation after the FSDMGR SETINFO persistence slice:
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop
  fsdmgr_disk_support_imports_round_trip_sparse_sectors_and_info --
  --nocapture` passed. Cargo still emits the existing unused-code warnings.
- Full validation after the FSDMGR SETINFO persistence slice:
  `cargo fmt --check`, `git diff --check`, `$env:CARGO_INCREMENTAL='0';
  cargo check --features unicorn,trace,win32-desktop`, and
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The eVC4 MIPSII fixture remains ignored,
  and Cargo still emits the existing unused-code warnings.
- `src/ce/kernel.rs`, `src/ce/coredll.rs`, `src/emulator/imports.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: direct
  FSDMGR disk IOCTL handling now recognizes CE `IOCTL_DISK_FORMAT_VOLUME` and
  `IOCTL_DISK_SCAN_VOLUME` from `diskio.h`. Synthetic format-volume clears
  sparse sectors like the existing format-media path, scan-volume succeeds as a
  no-op, and real FATFS utility execution remains queued.
- Validation after the FSDMGR format/scan volume IOCTL slice:
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop
  fsdmgr_disk_support_imports_round_trip_sparse_sectors_and_info --
  --nocapture`, `cargo fmt --check`, and `git diff --check` passed. Cargo still
  emits the existing unused-code warnings.
- `src/ce/kernel.rs`, `src/ce/coredll.rs`, `src/emulator/imports.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: direct
  FSDMGR disk IOCTL handling now names CE `IOCTL_DISK_SET_STANDBY_TIMER`,
  `IOCTL_DISK_STANDBY_NOW`, obsolete `IOCTL_DISK_DELETE_CLUSTER`, and disk-level
  `IOCTL_DISK_READ_CDROM`/`IOCTL_DISK_WRITE_CDROM` from `diskio.h`. The
  synthetic disk reports `ERROR_NOT_SUPPORTED` without touching input/output
  buffers or sector state; real power, cluster, and CD-ROM driver behavior
  remains queued.
- Validation after the FSDMGR unsupported specialized disk IOCTL slice:
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop
  fsdmgr_disk_support_imports_round_trip_sparse_sectors_and_info --
  --nocapture`, `cargo fmt --check`, `git diff --check`,
  `$env:CARGO_INCREMENTAL='0'; cargo check --features
  unicorn,trace,win32-desktop`, and `$env:CARGO_INCREMENTAL='0'; cargo test
  -j 1 --features unicorn,trace,win32-desktop` passed. The eVC4 MIPSII fixture
  remains ignored, and Cargo still emits the existing unused-code warnings.
- `src/ce/kernel.rs`, `src/ce/coredll.rs`, `src/emulator/imports.rs`,
  `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: direct
  CE `FSDMGR_AsyncEnterVolume @80` now allocates a distinct one-shot synthetic
  lock token for registered HVOLs, returns the HVOL as lock data, and cleans the
  token if output copying fails. `FSDMGR_AsyncExitVolume @81` now rejects null,
  mismatched, duplicate, and stale lock/data pairs, consumes successful exits,
  and drops outstanding async volume locks when their mounted volume is
  unmounted or closed. Real CE mounted-volume availability, powerdown, and
  thread-exit wait reference behavior remains queued.
- Focused validation after the FSDMGR async volume lock-lifetime slice:
  `cargo test -j 1 --features unicorn,trace,win32-desktop
  fsdmgr_async_volume_imports_lock_registered_hvol_shape -- --nocapture`
  passed. Cargo still emits the existing unused-code warnings.
- Full validation after the FSDMGR async volume lock-lifetime slice:
  `cargo fmt --check`, `git diff --check`, and
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The eVC4 MIPSII fixture remains ignored,
  and Cargo still emits the existing unused-code warnings.
- `src/ce/file.rs`, `src/ce/kernel.rs`, `src/ce/coredll.rs`,
  `src/ce/thread.rs`, `src/error.rs`, `src/emulator/imports.rs`, `PLAN.MD`,
  `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: direct
  `FSDMGR_RegisterVolume @27` mount-name allocation now follows the CE
  `fsdmgrapi.cpp` suffix loop for conflicting AFS folder names. Registered
  volumes get `Name`, then `Name2` through `Name9`, and exhaustion now reports
  `ERROR_OUT_OF_STRUCTURES` instead of collapsing into invalid-parameter.
- Focused validation after the FSDMGR register-volume suffix slice:
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop
  fsdmgr_register_volume_maps_disk_pointer_to_volume_handle -- --nocapture`
  passed. Cargo still emits the existing unused-code warnings.
- Full validation after the FSDMGR register-volume suffix slice:
  `cargo fmt --check`, `git diff --check`,
  `$env:CARGO_INCREMENTAL='0'; cargo check --features
  unicorn,trace,win32-desktop`, and `$env:CARGO_INCREMENTAL='0'; cargo test
  -j 1 --features unicorn,trace,win32-desktop` passed. The eVC4 MIPSII
  fixture remains ignored, and Cargo still emits the existing unused-code
  warnings.
- `src/ce/coredll.rs`, `src/ce/coredll_ordinals.rs`, `src/ce/file.rs`,
  `tests/coredll_raw_memory_file.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: coredll and AFS
  `GetFileSecurityW`/`SetFileSecurityW` now follow the reviewed CE
  `pathapi.cpp`/`fsdacl.h` no-security-manager shape. The raw wrappers validate
  guest path and buffer arguments, route mounted paths before reporting the
  current no-ACL-DLL `ERROR_NOT_SUPPORTED` result, and copy a zero
  length-needed value for `GetFileSecurityW` failures like the FSEXT wrapper.
  Real ACL descriptor storage and enforcement remain queued.
- Focused validation after the coredll file-security no-security-manager slice:
  `$env:CARGO_INCREMENTAL='0'; cargo test --features
  unicorn,trace,win32-desktop --test coredll_raw_memory_file file_security --
  --nocapture` passed. Cargo still emits the existing unused-code warnings.
- Full validation after the coredll file-security no-security-manager slice:
  `cargo fmt --check`, `git diff --check`,
  `$env:CARGO_INCREMENTAL='0'; cargo check --features
  unicorn,trace,win32-desktop`, and `$env:CARGO_INCREMENTAL='0'; cargo test
  -j 1 --features unicorn,trace,win32-desktop` passed. The eVC4 MIPSII
  fixture remains ignored, and Cargo still emits the existing unused-code
  warnings.
- `src/ce/coredll.rs`, `src/emulator/imports.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: `FSDMGR_DiskIoControl @12`
  now names the CE `IOCTL_DISK_GETINFO` alias from `diskio.h` alongside legacy
  `DISK_IOCTL_GETINFO`. The import regression covers the new output-buffer
  `DISK_INFO` call shape after `IOCTL_DISK_SETINFO` persistence.
- Validation after the FSDMGR `IOCTL_DISK_GETINFO` alias slice:
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop
  fsdmgr_disk_support_imports_round_trip_sparse_sectors_and_info --
  --nocapture`, `cargo fmt --check`, `git diff --check`,
  `$env:CARGO_INCREMENTAL='0'; cargo check --features
  unicorn,trace,win32-desktop`, and `$env:CARGO_INCREMENTAL='0'; cargo test
  -j 1 --features unicorn,trace,win32-desktop` passed. The eVC4 MIPSII
  fixture remains ignored, and Cargo still emits the existing unused-code
  warnings.
- `src/ce/coredll.rs`, `tests/coredll_raw_memory_file.rs`, `PLAN.MD`,
  `TODO.md`, `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: file-handle
  `DeviceIoControl(IOCTL_FILE_READ_SCATTER/IOCTL_FILE_WRITE_GATHER)` now
  follows the CE `fileapi.cpp` scatter/gather dispatch enough for synchronous
  page-multiple `FILE_SEGMENT_ELEMENT` arrays at the current file position.
  The raw regression writes two pages with gather and reads them back with
  scatter through host-backed mounted storage; offset-array and overlapped
  variants remain queued.
- `src/ce/coredll.rs`, `src/ce/kernel.rs`,
  `tests/coredll_raw_memory_file.rs`, `PLAN.MD`, `TODO.md`, `KNOWN_BUGS.md`,
  and `SOURCE_REFERENCES.md`: file-handle
  `DeviceIoControl(IOCTL_FILE_READ_SCATTER/IOCTL_FILE_WRITE_GATHER)` now also
  supports CE reserved offset arrays from `privatefilehandle.cpp`, reading and
  writing each page at its caller-supplied absolute offset without advancing the
  current file pointer. The raw regression writes two pages out-of-cursor with
  `WriteFileGather`, proves a later normal `WriteFile` still lands at the old
  cursor, then reads the pages back in reversed offset order with
  `ReadFileScatter`; overlapped scatter/gather and real FSD filter forwarding
  remain queued.
- Validation after the reserved-offset scatter/gather slice:
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_memory_file
  coredll_raw_device_io_control_file_scatter_gather_reserved_offsets --
  --nocapture`, `cargo fmt --check`, `git diff --check`,
  `$env:CARGO_INCREMENTAL='0'; cargo check --features
  unicorn,trace,win32-desktop`, and `$env:CARGO_INCREMENTAL='0'; cargo test
  -j 1 --features unicorn,trace,win32-desktop` passed. The eVC4 MIPSII
  fixture remains ignored, and Cargo still emits the existing unused-code
  warnings.
- `src/ce/coredll.rs`, `tests/coredll_raw_memory_file.rs`, `PLAN.MD`,
  `TODO.md`, `KNOWN_BUGS.md`, `SOURCE_REFERENCES.md`, and `PROGRESS.md`:
  file-handle scatter/gather `DeviceIoControl` now accepts non-null
  `OVERLAPPED*` arguments and ignores them like CE cachefilt's
  `FCFILT_ReadFileScatter`/`FCFILT_WriteFileGather` path. The raw regression
  passes a deliberately bogus non-null overlapped pointer through write-gather
  and read-scatter to prove the pointer is neither dereferenced nor rejected;
  real lower-FSD/filter forwarding remains queued.
- Validation after the scatter/gather overlapped-pointer slice:
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_memory_file
  coredll_raw_device_io_control_file_scatter_gather_ignores_overlapped --
  --nocapture`, `cargo fmt --check`, `git diff --check`,
  `$env:CARGO_INCREMENTAL='0'; cargo check --features
  unicorn,trace,win32-desktop`, `$env:CARGO_INCREMENTAL='0'; cargo test -j 1
  --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`, and
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The eVC4 MIPSII fixture remains ignored,
  and Cargo still emits the existing unused-code warnings.
- Validation after the file scatter/gather IOCTL slice:
  `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_memory_file
  coredll_raw_device_io_control_file_scatter_gather_transfers_pages --
  --nocapture`, `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop --test coredll_raw_memory_file`,
  `cargo fmt --check`, `git diff --check`,
  `$env:CARGO_INCREMENTAL='0'; cargo check --features
  unicorn,trace,win32-desktop`, and `$env:CARGO_INCREMENTAL='0'; cargo test
  -j 1 --features unicorn,trace,win32-desktop` passed. The eVC4 MIPSII
  fixture remains ignored, and Cargo still emits the existing unused-code
  warnings.
- `src/ce/coredll.rs`, `tests/coredll_raw_gwe.rs`, `PLAN.MD`, `TODO.md`,
  `KNOWN_BUGS.md`, and `SOURCE_REFERENCES.md`: raw `GetCharABCWidthsI` now
  shares the CE nonzero-escapement rejection already covered for
  `GetCharABCWidths`. The regression selects a nonzero-escapement Tahoma font,
  calls the glyph-index/count `I` variant, expects `ERROR_INVALID_PARAMETER`,
  and verifies the caller ABC buffer remains untouched.
- Validation after the `GetCharABCWidthsI` nonzero-escapement slice:
  `cargo fmt --check`, `git diff --check`,
  `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test -j 1 --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe coredll_raw_get_char_abc_widths_rejects_nonzero_escapement`,
  and `$env:CARGO_INCREMENTAL='0'; cargo test -j 1 --features
  unicorn,trace,win32-desktop` passed. The eVC4 MIPSII fixture remains
  ignored, and Cargo still emits the existing unused-code warnings.
