# Progress

Regenerated on 2026-06-11 from the current implementation and test surface.

## Current Snapshot

- Runtime loader work has reached dynamic Unicorn DLL mapping with dependency loading, dependency-ref release on unload, current-image cleanup on failed maps, transactional current-image resource/trap/trampoline commit ordering, import patching, forwarders, malformed-forwarder rejection, failed-load and failed-attach rollback for load-attempt refs, trampoline tracking, datafile/no-resolve flags, datafile export suppression, CE-style process/thread lifecycle calls, process-detach refcount draining, and CE `DisableThreadLibraryCalls` filtering.
- Shell icon work now includes `ExtractIconExW`, real PE resource icon extraction, PE group-icon count reporting for `nIconIndex == -1`, raw `KernExtractIcons` integer group-resource extraction, shell fallback icons, `CreateIconIndirect`, `DrawIconEx`, image lists, CE-valid image-list creation/copy flag validation, CE-style image-list bitmap copy/lifetime handling, bitmap-backed image-list drawing, `xBitmap` offsets, `rgbBk` fill handling, CE color-image vs mask-only `rgbFg == CLR_NONE` blend behavior, and exact CE `IMAGELISTDRAWPARAMS` size plus normalized-field write-back validation.
- `Shell_NotifyIcon` now tracks add/modify/delete state, rejects duplicate `(hwnd,uID)` adds, honors member `NIF_*` flags on add, requires the fixed CE `NOTIFYICONDATAW` footprint and readable 64-WCHAR `szTip` buffer, keeps the existing icon on `NIM_MODIFY | NIF_ICON` with null `hIcon` per the CE taskbar path, posts callback messages, records destroy-icon cleanup, tracks registered taskbar HWND state, posts CE `WM_HANDLESHELLNOTIFYICON` taskbar messages with copied `NOTIFYICONDATAW` payloads, releases those copied payloads after dispatch, and treats stale registered taskbar windows as a non-posting shell-state update success.
- `SHNotificationUpdateI` now covers CE update-mask behavior for null icon preservation, non-null icon replacement cleanup, stale incoming `hwndSink` values while keeping the original registered sink, and inform/iconic priority-list movement; iconic notification expiration, explicit remove, and sink cleanup now record copied icon destruction like the CE taskbar cleanup paths, notification remove and sink cleanup now purge pending callback records for removed notifications, and `SHNotificationGetDataI` accepts the CE fixed-title-buffer path when `cbTitle == 0`.
- File-change notifications now canonicalize public watch paths, preserve caller notification filter bits while CE-style known-bit matching decides whether changes signal, honor root and non-root `WatchSubtree` boundaries, map same-parent vs cross-parent move notifications through CE `NotifyMoveFileEx` action semantics, coalesce exact duplicates, transient create/delete pairs, and modified/delete sequences, track CE-style outstanding notification signals across `FindNextChangeNotification`, and gate detailed notification records by the CE `FILE_NOTIFY_CHANGE_CEGETINFO` flag while signal-only watches still wake normally. `CeGetFileNotificationInfo` record sizing now follows CE `NotifyReset`, including the copied trailing NUL WCHAR and DWORD padding while preserving non-NUL `FileNameLength`, and its no-pending path now preserves CE's guarded output-pointer write order while returning `ERROR_NO_MORE_ITEMS`. Mounted file operations now enforce CE volume boundaries and read-only root access checks for mutating calls, with access-denied read-only mutations leaving watchers unsignaled. Mounted change notifications now retain the resolved owning mount root, so non-root watches are scoped to their CE-style volume while recursive root watches still report mounted-volume-prefixed child paths. Raw `DuplicateHandle` now creates independent local handles for notification/file/find objects and supports the CE `DUPLICATE_CLOSE_SOURCE` ownership-transfer shape used by notification close paths. Public file-change notification handles now track their creating process and reject foreign-process wait/reset/info/duplicate/close attempts, and direct `AFS_FindFirstChangeNotificationW` now honors its nonzero `hProc` owner.
- GWE message work includes cross-thread send setup, timeout marking, CE-public `SMTO_NORMAL` timeout-send completion, destroyed-window completion, and zero-result writes for destroyed `SendMessageTimeout` targets.
- Winsock has CE-facing dispatch for core socket operations with isolated NAT addressing, `select` fd-set validation, readiness checks, and scheduler wake candidate integration.
- Core CE subsystems remain broad and test-backed: handles, waits, events, TLS, critical sections, registry, files, memory, GDI resources, DIBs, windows, menus, clipboard, and scheduler selection.

## Recent Source-Visible Slices

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
  COM registry for that interface by GUID value, and records the acquired local
  interface token as `callback_ptr` while preserving the previous null fallback
  for unregistered classes.
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
- `tests/coredll_raw_memory_file.rs`: process-owned notification coverage now verifies a different current process cannot wait on, reset, query, duplicate, or close a file-change notification handle created by another process, and that the owner can still close it afterward.
- `tests/coredll_raw_memory_file.rs`: direct `AFS_FindFirstChangeNotificationW` coverage now verifies a nonzero `hProc` creates a notification handle owned by that target process rather than by the raw ordinal caller.
- `tests/coredll_raw_memory_file.rs`: `CeGetFileNotificationInfo` coverage now verifies a two-character filename record requires CE's 20-byte NUL-padded record size instead of the old 16-byte non-NUL-only size, and confirms the copied NUL/padding bytes while preserving `FileNameLength`.
- `tests/coredll_raw_memory_file.rs`: `CeGetFileNotificationInfo` coverage now verifies the no-pending CE output-pointer fault order: a bad `lpBytesReturned` leaves `lpBytesAvailable` untouched, while a bad `lpBytesAvailable` still leaves `lpBytesReturned` zeroed and returns `ERROR_NO_MORE_ITEMS`.
- `tests/coredll_raw_memory_file.rs`: file-change notification filter coverage now verifies CE-style unknown filter preservation: unknown-only watches stay inert, while unknown bits combined with `FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO` still signal and return detailed records.
- `tests/coredll_raw_kernel.rs`: `SHNotificationAddI` coverage now asserts the stored notification and queued COM callback record do not inherit the marshalled `cbData` argument as a callback pointer.
- `tests/basic_subsystems.rs`: direct shell notification callback queue coverage now verifies a failed non-null runtime COM dispatch can restore its callback record ahead of newer pending callbacks.
- `tests/coredll_raw_kernel.rs`: `SHNotificationAddI` title/HTML coverage now verifies the CE pointer-presence rule for `SHNP_INFORM`: title-only with null HTML fails, while non-null empty HTML succeeds and receives the default inform duration.
- `tests/coredll_raw_kernel.rs`: `SHNotificationUpdateI` coverage now asserts `SHNUM_PRIORITY` moves a notification from the inform list to the iconic list and `SHNotificationRemoveI` clears the priority-list entry.
- `src/ce/coredll.rs`: raw `ImageList_Add`, `ImageList_AddMasked`, and `ImageList_Replace` now snapshot real source and mask bitmap pixels into owned bitmap storage like CE `imagelist.cpp` image/mask DCs, while preserving metadata-only pseudo handles.
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
