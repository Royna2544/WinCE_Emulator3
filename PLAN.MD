Reference C:\WINCE600 sources

# CE Guest DLL, Shell, and Winsock Fidelity Plan

## Summary

Use `C:\WINCE600` as the behavior reference and
`D:\INAVI_Emulator\DUMPPLZ\Windows` as the runtime DLL byte source. Implement
this as CE subsystem fidelity, not app-specific behavior: dynamic guest DLL
loading, explicit stub auditing, shell launch semantics, stronger GWE defaults,
and isolated guest networking.

## Implementation Ledger

| Subsystem | CE source reference | v2 corroboration | Implemented APIs | Open gaps | Fixture coverage | Latest iNavi evidence |
| --- | --- | --- | --- | --- | --- | --- |
| Guest DLL loader | `NK\KERNEL\loader.c`, `NK\KERNEL\thread.c`, PE loader and SDK loader headers | startup preload path only | startup import DLL registration, CE-aware DLL search order, live-mutable import trap table, static trap-base allocator with reserved dynamic range, live/persisted import trap-page refresh, runtime normal-code DLL map/relocate/import patching from `LoadLibraryW/LoadLibraryExW(flags=0)`, recursive dependency loading, runtime inline MIPS trampoline patching with live trampoline metadata updates, runtime `LoadLibraryExW(DONT_RESOLVE_DLL_REFERENCES)` mapping without dependency/import/lifecycle resolution, runtime `LoadLibraryExW(LOAD_LIBRARY_AS_DATAFILE)` resource-module mapping with immediate `FindResourceW`/`LoadResource`/`LoadStringW` visibility and no code exports, raw `LoadLibraryExW` supported-flag reuse of already registered modules, runtime loader audit counters in debug/monitor summaries, runtime module export table ingestion, module metadata/refcounts, registered-export lookup, PE forwarded-export metadata retention, runtime forwarder resolution through guest DLL search/load and COREDLL trap addresses, TLS callback metadata parsing, guest TLS and `DllMain(DLL_PROCESS_ATTACH)` callouts, final dynamic `FreeLibrary` guest TLS and `DllMain(DLL_PROCESS_DETACH)` callouts before non-reclaiming unload-pending release | fast-start runtime trampoline hook variants if trace evidence demands them, raw/non-Unicorn real DLL mapping remains explicit failure, broader forwarded-export/no-resolve/resource fixture variants | `dll_search`, import-trap tests, `refresh_import_trap_page_blob_replaces_existing_blob`, `runtime_occupied_ranges_include_memory_regions_and_blobs`, `loaded_module_export_snapshots_skip_unload_pending_modules`, `external_table_accepts_runtime_loaded_module_exports`, `external_table_resolves_forwarded_exports_through_loaded_modules`, `external_table_resolves_multi_level_forwarding_chain`, `external_table_cycle_detection_returns_none`, `parse_forwarder_target_handles_name_and_ordinal_forms`, `065_loadlibrary_getprocaddress_module`, `171_loadlibrary_guest_dll`, `172_loadlibrary_dependent_guest_dll`, `173_loadlibrary_tls_callback`, `174_loadlibrary_forwarded_export`, `175_loadlibrary_noresolve`, `176_loadlibrary_datafile_resource`, raw module refcount tests, `live_trampoline_state_extends_runtime_ranges_and_origin_maps`, `debug_summary_includes_runtime_loader_counters` | dumped DLLs from `D:\INAVI_Emulator\DUMPPLZ\Windows` are authoritative runtime bytes |
| COREDLL stub audit | COREDLL API sets and SDK headers | broad stub policy proved convenient but unsafe | static ordinal table, implemented/stubbed plan, fallback audit classification/logging, raw thread/caller/trap PC context, caller module attribution from mapped image/DLL blobs, must-implement fallback `ERROR_NOT_SUPPORTED` side effects, explicit failure-shaped fallback returns for critical loader/process/shell/UI APIs | replace must-implement stubs with real subsystem behavior as traces hit them | `raw_stub_audit_keeps_import_trap_context`, `mapped_blob_module_for_pc_attributes_image_and_runtime_dlls`, `tests\coredll_dispatch.rs`, `coredll_raw_registry_subkey_create_enum_query_and_delete`, `coredll_raw_get_file_attributes_ex_w_reads_attribute_data` | shell/UI/process calls must not silently fake success |
| Shell launch/settings | `SHELL`, Explorer registry conventions, `PUBLIC\COMMON\SDK\INC\winuser.h` MessageBox/button/menu constants, `PUBLIC\COMMON\SDK\INC\shellsdk.h`, `PUBLIC\COMMON\SDK\INC\shsdkstc.h`, `PUBLIC\COMMON\SDK\INC\shellapi.h`, `PUBLIC\COMMON\SDK\INC\winbase.h`, `PUBLIC\SHELL\OAK\FILES\shell.reg`, `PUBLIC\SHELL\OAK\HPC\CESHELL\API\shortcut.cpp`, `PUBLIC\SHELL\OAK\HPC\CESHELL\API\api.cpp`, `PUBLIC\SHELL\OAK\HPC\CESHELL\UI\filechangemgr.cpp` | v2 shell path shape only | `SHGetSpecialFolderPath` registry/fallback lookup with registry/missing/non-string source tracking, selectable compat-vs-strict fallback policy, plus `fCreate`/`CSIDL_FLAG_CREATE` mounted-folder creation and MAX_PATH rejection, registry/shortcut-backed `ShellExecuteEx` launch chain with `hInstApp`, `SEE_MASK_NOCLOSEPROCESS` hProcess output, `nShow` propagation to child `WinMain`, `lpDirectory` current-directory propagation through pending launches, and CE `%1`/`%L`/`%l`/`%*` association command-template expansion, raw `CreateProcessW` `pszCurDir` propagation, child relative-EXE resolution through queued/current CE directory, parked-process current-directory restore, raw `SHCreateShortcut` CE text-link creation, raw `SHGetShortcutTarget` CE text-link target extraction, raw `SHCreateShortcutEx` unique-name and optional buffer-return behavior, raw `SHAddToRecentDocs(SHARD_PATH/null/path-backed SHARD_PIDL/opaque namespace SHARD_PIDL)` recent-folder shortcut creation/clear plus MRU recent-document state capped to 10 shortcut files with display names, UI-facing display-entry projection, and raw PIDL byte retention, raw `SHChangeNotifyRegisterI`/`SHFileNotifyRemoveI`/`SHFileNotifyFreeI` registration/free tracking with HWND cleanup, matching raw file operations post heap-backed `WM_FILECHANGEINFO`/`FILECHANGENOTIFY` path or PIDL payloads for create/delete/mkdir/rmdir/rename/update events (honoring `SHCNF_PATHW`/`SHCNF_IDLIST`) plus live mount/unmount `SHCNE_DRIVEADD`/`SHCNE_DRIVEREMOVED` storage notifications, registry-backed `SHGetFileInfo` display/type/attribute/sysicon metadata with CE supported-flag validation, explicit `ERROR_INVALID_FLAGS` for unsupported/colliding flags, registry `DefaultIcon` pseudo image-list identity, CE-ordered document/folder/executable/storage-card/network-folder special icon slots including temporary `\\share`, `\release`, and `\network` network-folder detection, raw `ExtractIconExW` count/extract support using real PE icon bitmap extraction (RT_ICON color+AND mask bitmaps) for EXE/DLL paths and synthetic icon fallback for non-PE paths, shell system image-list handle backing, and raw `ImageList_Create`/`LoadImage`/`Add`/`Replace`/`ReplaceIcon`/`Copy`/`CopyDitherImage`/`Merge`/`Remove`/`SetIconSize`/`SetImageCount`/`SetOverlayImage`/`Duplicate`/`Destroy` plus size/count/icon/image-info/background/draw-query overlay capture and drag-image begin/move/show/cursor/query/end behavior, `TrackPopupMenuEx` tracked attempts plus CE menu-loop owner notifications/default `WM_COMMAND` routing honoring `TPM_NONOTIFY`, state-backed `TPM_RETURNCMD` default-command return, rendered popup/submenu surface rows, queued input/cursor row selection for enabled command/submenu rows with framebuffer highlight updates, CE tap-only submenu modal pump where a button-down/up on a submenu parent opens the child pane (stacked `PopupMenuModalMenuState` levels with per-level hit testing and stack truncation) and the command is only returned once the user taps an item inside the child pane rather than immediately picking the submenu default, blocking modal wait with timer pump on re-entry and `ThreadHitTest` live routing via owner `hwnd` in `blocked_get_message` snapshot, `Shell_NotifyIconW` add/modify/delete state, owner callback message posting, and CE `HHTBF_DESTROYICON` owned-icon replacement/delete/teardown bookkeeping over a dedicated `ShellSystem`, `SHNotificationAddI`/`UpdateI`/`RemoveI`/`GetDataI` data storage/query by `CLSID + dwID` preserving notification `grfFlags` separately from update masks, plus `WM_NOTIFY`/`NMSHN` sink callback posting for stored notification events including `SHNN_LINKSEL` link strings and `SHNN_DISMISS` timeout flags, automatic notification timeout expiry/removal with `SHNN_DISMISS` callback delivery, shell notification cleanup on HWND/process teardown, `MessageBoxW` owner validation plus recorded text/caption/style, button IDs, default index, icon class, owner enabled-state, CE default-button return selection, initial keyboard focus on the default button plus CE dialog-manager keyboard navigation in the modal pump (Tab/Shift+Tab and arrow keys move focus between the box's buttons, Space activates the focused button, Enter activates the focused/default button) so a navigated button is the one returned, and blocking modal wait with timer pump on re-entry and `ThreadHitTest` live routing via dialog `hwnd`, `ShellExecuteEx` verb routing: non-`open` verbs use the verb-specific `Shell\{verb}\Command` registry entry when registered and fall back to `Shell\Open\Command` when the requested verb has no command, precise `ShellExecuteEx` launch errors distinguishing `SE_ERR_PNF`/`ERROR_PATH_NOT_FOUND` (containing directory missing) from `SE_ERR_FNF`/`ERROR_FILE_NOT_FOUND` (file missing in an existing directory) and `SE_ERR_NOASSOC` (no registered association) for both executable targets, associated-document viewer paths, and CE shortcut targets, `ImageList_DrawIndirect` `xBitmap`/`yBitmap` source-offset fields from `IMAGELISTDRAWPARAMS` applied to the per-image source coordinates (matching CE `imagelist.cpp` `DrawIndirect` `rcImage.left += xBitmap`), `rgb_bk` background-color pre-fill for non-`ILD_TRANSPARENT` draws via dest-rect fill before blitting so transparent-masked pixels show the background color | richer notification interaction (guest COM callback, bubble/taskbar UI), fuller namespace/policy and mounted CSIDL edge confirmation, richer notification interaction (guest COM callback, bubble/taskbar UI), fuller namespace/policy and mounted CSIDL edge confirmation | raw `ShellExecuteEx` association tests, `shell_execute_ex_appends_parameters_without_template_placeholder`, `shell_execute_ex_expands_long_filename_template_placeholders`, `shell_execute_ex_resolves_hklm_software_classes_association`, `shell_execute_ex_reports_precise_missing_exe_and_no_association_errors`, `coredll_raw_create_process_preserves_current_directory`, `process_launch_resolution_uses_queued_current_directory`, parked-process identity tests, `shell_shortcut_ordinals_create_read_and_launch_ce_lnk_files`, `shell_create_shortcut_ex_returns_unique_name_and_checks_output_capacity`, `shell_add_to_recent_docs_creates_and_clears_recent_shortcuts`, `sh_change_notify_i_tracks_register_remove_and_free_state`, `sh_change_notify_i_posts_filechangeinfo_for_matching_file_operations`, `coredll_raw_shget_special_folder_path_returns_ce_paths`, `coredll_raw_shget_special_folder_path_strict_policy_rejects_fallbacks`, `coredll_raw_shget_special_folder_path_rejects_overlong_registry_path`, `coredll_raw_shget_special_folder_path_honors_create_flags`, `sh_get_file_info_uses_registry_associations_and_attributes`, `sh_get_file_info_system_image_list_supports_icon_queries_and_draw`, `image_list_ordinals_track_created_lists_and_icons`, `sh_get_file_info_rejects_unsupported_and_colliding_flags`, `coredll_raw_track_popup_menu_records_attempt_and_returns_default_command`, `coredll_raw_track_popup_menu_tap_opens_submenu_before_selecting`, `shell_notify_icon_tracks_add_modify_delete_and_posts_callback`, `shnotification_i_tracks_query_update_and_remove_state`, `shnotification_i_posts_timeout_dismiss_and_removes_expired_record`, `shell_window_destroy_removes_notify_icon_and_notification_state`, `message_box_w_records_text_owner_and_returns_default_button`, `message_box_w_uses_queued_modal_key_and_button_input`, `message_box_w_tab_navigation_changes_activated_button`, `message_box_w_rejects_destroyed_owner_without_recording`, `shell_execute_ex_verb_print_falls_back_to_open_when_print_not_registered`, `shell_execute_ex_verb_print_uses_print_command_when_registered`, `image_list_draw_indirect_applies_x_bitmap_offset_and_rgb_bk_fill`, `coredll_raw_track_popup_menu_submenu_flips_left_near_right_edge`, `coredll_raw_track_popup_menu_hover_timer_opens_submenu_on_mouse_device, `coredll_raw_fs_io_control_refresh_and_flush_are_no_ops`` | route UI still needs real shell/process/GWE semantics |
| GWE defaults/input/send | `GWE\INC`, `window.hpp`, CE `winuser.h`, MFC `wincore.cpp`, CE input/clipboard/caret/IME headers, `PUBLIC\COMMON\SDK\INC\winuser.h` clipboard/caret/accelerator/timeout constants/prototypes | window broker behavior evidence | core window/message pump, partial defaults, raw/Unicorn-default `DefWindowProcW` `WM_NCHITTEST`, client `WM_SETCURSOR` class-cursor selection, non-client `WM_SETCURSOR` stock arrow/sizing cursor selection, `WM_MOUSEACTIVATE` activation, `WM_SYSCOMMAND/SC_CLOSE` handling, `WM_NCACTIVATE` returning TRUE, `WM_ACTIVATE(WA_ACTIVE/WA_CLICKACTIVE)` calling `SetFocus(hwnd)` so DefWindowProcW restores focus to the activated window, Unicorn `CreateWindowExW(WS_VISIBLE)` top-level post-WM_CREATE `activate_window` call matching CE's internal `ShowWindow(SW_SHOW)` after WM_CREATE, raw `TranslateMessage` ASCII letter/digit Shift/CapsLock, Ctrl-letter/OEM control codes, `WM_SYSKEYDOWN` to `WM_SYSCHAR` path, dead-key VK→char tables for Western layouts (AZERTY 0x040C, QWERTZ 0x0407, Spanish 0x040A/0x0C0A) with `WM_DEADCHAR`/`WM_CHAR` composition state machine and Unicode precomposed-form table for circumflex/grave/acute/tilde/diaeresis via `Gwe::dead_key` pending state, Western layout Ctrl bypass (pure Ctrl without Alt skips the layout character table and falls through to VK-based control code), AltGr (Ctrl+Alt) character tables for AZERTY/QWERTZ/Spanish layouts in `TranslateMessage` (`translate_vk_for_western_layout_altgr`), raw `TranslateAcceleratorW` modifier and `WM_SYSKEYDOWN` matching, stateful raw keyboard layout probes (`GetKeyboardLayout*`, `LoadKeyboardLayoutW`, `ActivateKeyboardLayout`, `ImmGetKeyboardLayout`) plus minimal HIMC/open/conversion-status IMM context probes including `ImmDisableIME(0)`/`ImmEnableIME(0)`, raw `SendNotifyMessageW` same-thread direct delivery plus cross-thread/broadcast live-top-level no-wait sent-queue delivery, raw `SendMessageTimeout` CE `SMTO_NORMAL`/`SMTO_BLOCK`/`SMTO_ABORTIFHUNG` flag validation with queued flag preservation, timeout-transaction queueing, zero-timeout expiry, cross-thread `lpdwResult` write-back on dispatch completion, receiver-destruction result cleanup, sender-termination cleanup for queued/active sent messages, and `SMTO_BLOCK` cross-thread blocking wait with scheduler parking, sent/nonqueued-before-posted `GetMessageW`/`PeekMessageW` ordering, pending sent-message receiver readiness for parked processes and suspended-thread timeslicing, partial cross-thread send, GWE-owned clipboard open/close/owner/data/format-enumeration/custom-format-name state plus delayed `WM_RENDERFORMAT` owner requests, active render-callback-only fulfillment, stale-render retry cleanup, `WM_RENDERALLFORMATS` owner-destroy request, `WM_DESTROYCLIPBOARD` replacement notification, owner/open-window teardown cleanup, `GetClipboardDataAlloc` local-handle copy through raw COREDLL clipboard ordinals, and cross-thread blocking `GetClipboardData`/`GetClipboardDataAlloc` render fulfillment via `WM_RENDERFORMAT` with scheduler parking, GWE-owned caret create/destroy/show-hide/position/blink-time/system-enable state through raw COREDLL caret ordinals, plus visible-caret invalidation and framebuffer dirty rectangles on show/hide/move/destroy/system-enable changes, caret XOR bitmap rendered directly into the VirtualFramebuffer on `ShowCaret`/`HideCaret`/`SetCaretPos`/`CreateCaret`/`DestroyCaret` and on blink phase transitions in `GetMessageW`/`PeekMessageW` dispatch with `caret_drawn_in_framebuffer` state tracking; `BeginPaint` XOR-erases the caret so the app paints clean pixels, `EndPaint` XOR-restores it matching CE GWES caret paint-cycle coordination, `WM_CAPTURECHANGED` posted to old capture window on `SetCapture`/`ReleaseCapture`, `WM_NCLBUTTONDOWN(HTCLOSE)` and `WM_NCLBUTTONDBLCLK(HTSYSMENU)` routing to `WM_SYSCOMMAND(SC_CLOSE)` in `DefWindowProcW`, `WM_SYSKEYDOWN(VK_F4)` routing to `WM_SYSCOMMAND(SC_CLOSE)` in `DefWindowProcW`, `WM_CONTEXTMENU` forwarding to parent window in `DefWindowProcW`, `TranslateAcceleratorW` `WM_COMMAND` wparam `HIWORD=1` accelerator notification, `WM_RBUTTONUP` routing to `WM_CONTEXTMENU` with client-to-screen coord conversion in `DefWindowProcW`, `WM_CHARTOITEM`/`WM_VKEYTOITEM` returning −1 (list-box default) in `DefWindowProcW`, `VK_NUMLOCK`/`VK_SCROLL` toggle-bit tracking on key-down matching `VK_CAPITAL` behavior, `WM_RBUTTONDBLCLK` routing to `WM_CONTEXTMENU` (same coord-conversion path as `WM_RBUTTONUP`) in `DefWindowProcW`, `WM_MOUSEWHEEL` forwarding to parent window in `DefWindowProcW`, `WM_SETREDRAW` suppress/restore invalidation in `DefWindowProcW` (redraw-suspended windows skip `InvalidateRect`; re-enabling triggers full-client invalidation), `WM_SYSKEYDOWN(VK_F10)` routing to `WM_SYSCOMMAND(SC_KEYMENU, VK_F10)` in `DefWindowProcW`, `WM_SYSKEYUP(VK_MENU)` routing to `WM_SYSCOMMAND(SC_KEYMENU, 0)` in `DefWindowProcW`, `WM_DEVICECHANGE(DBT_DEVICEARRIVAL/DBT_DEVICEREMOVECOMPLETE)` broadcast via `send_notify_message_w(HWND_BROADCAST)` on `mount_guest_root`/`unmount_guest_root`, `WM_SETTINGCHANGE` broadcast via `send_notify_message_w(HWND_BROADCAST)` in `SystemParametersInfoW` when `SPIF_SENDCHANGE` is set, `WM_SYSKEYDOWN(VK_A–VK_Z)` routing to `WM_SYSCOMMAND(SC_KEYMENU, letter)` in `DefWindowProcW` (Alt+letter menu mnemonic activation), `WM_TIMECHANGE` broadcast via `send_notify_message_w(HWND_BROADCAST)` in `SetSystemTime`/`SetLocalTime`, `WM_FONTCHANGE` broadcast via `send_notify_message_w(HWND_BROADCAST)` in `AddFontResourceW` (on success) and `RemoveFontResourceW`, `WM_HSCROLL`/`WM_VSCROLL`/`WM_DRAWITEM`/`WM_MEASUREITEM`/`WM_DELETEITEM` forwarding to parent window in `DefWindowProcW`, `WM_CHAR`/`WM_SYSCHAR` message support in `TranslateAcceleratorW` for non-`FVIRTKEY` character accelerators, `WM_CTLCOLORxxx` returning `GetSysColorBrush` in `DefWindowProcW` (edit/listbox → `COLOR_WINDOW`, btn/dlg/static/msgbox → `COLOR_BTNFACE`, scrollbar → `COLOR_SCROLLBAR`), `WM_INPUTLANGCHANGE` posted to focused window from `ActivateKeyboardLayout`, `WM_GETMINMAXINFO` filling `MINMAXINFO` with screen dimensions in `DefWindowProcW`, `WM_SYSKEYDOWN` Alt+digit (0-9) routing to `WM_SYSCOMMAND(SC_KEYMENU)` in `DefWindowProcW` (extends Alt+letter), `WM_DISPLAYCHANGE` broadcast via `send_notify_message_w(HWND_BROADCAST)` from `ChangeDisplaySettingsEx` (with `CDS_TEST` suppression), caret blink-phase timer driven by `advance_caret_blink` called unconditionally on each `GetMessageW`/`PeekMessageW` entry (GWES-internal, not WM_TIMER priority), toggling `caret_blink_visible` after each `caret_blink_time_ms` interval and invalidating the caret owner window (blink only advances when the caret's owner window has keyboard focus matching CE GWES `GweCaretKillFocus`/`GweCaretSetFocus`), caret focus edge: `set_focus` calls `reset_caret_on_focus_lost`/`reset_caret_on_focus_gained` to hide/restore `caret_blink_visible` and invalidate the caret owner window when focus leaves or returns to its window, `WM_SETFONT`/`WM_GETFONT` storing and retrieving the per-window font handle in `DefWindowProcW`, `WM_SETICON`/`WM_GETICON` (`ICON_SMALL`/`ICON_BIG`) storing and retrieving per-window small/big icon handles in `DefWindowProcW` with class-`hIcon` fallback when no window-level icon is set, `WM_ENABLE` in `DefWindowProcW` calls `InvalidateRect` on the window so it repaints its enabled/disabled visual state, `WM_PARENTNOTIFY(WM_CREATE/WM_DESTROY)` posted to parent window by `CreateWindowExW`/`DestroyWindow` COREDLL handlers unless `WS_EX_NOPARENTNOTIFY` is set on the child, `WM_COMPAREITEM` forwarding to parent window in `DefWindowProcW` (same pattern as `WM_MEASUREITEM`/`WM_DELETEITEM`), `WM_ERASEBKGND` filling the client area with the class `hbrBackground` brush via `FillRect` in `DefWindowProcW` (returns 1 if hbr non-null, 0 if null), `WM_NEXTDLGCTL` in `DefWindowProcW`: `lparam&1` → direct-focus to `wparam` HWND; `lparam&1==0` → forward/backward tab-stop navigation via `get_next_dlg_tab_item`; sets focus via `set_focus` and updates dialog default-button via `DM_SETDEFID` when new focus is a pushbutton, `WM_GETTEXTLENGTH` returning actual UTF-16 window-title length in `DefWindowProcW` (closes dead-zero default in `gwe.rs::send_message`), `WM_SETTEXT` triggering `InvalidateRect` after title update in `DefWindowProcW`, `HWND_BROADCAST` notification delivery in Z-order (top-to-bottom) matching CE broadcast ordering, per-thread last-dispatch-time recording in `GetMessageW`/`PeekMessageW(PM_REMOVE)` and sent-message dispatch paths, `SMTO_ABORTIFHUNG` hung-thread abort in `SendMessageTimeoutW` returning `ERROR_TIMEOUT` when the target has not dispatched for ≥5 seconds, `WM_NCCREATE` returning TRUE in `DefWindowProcW` (window creation handshake), `WM_MENUCHAR` returning 0 (MNC_IGNORE) confirmed by fixture, matching CE DefWindowProcW, `WM_NCLBUTTONDBLCLK(HTCLOSE)` and `WM_NCLBUTTONDOWN(HTSYSMENU)` discriminant coverage completing the NC-button close matrix, `SetWindowLong(GWL_STYLE)` syncing `WS_VISIBLE`→`window.visible` and `WS_DISABLED`→`window.enabled` so `IsWindowVisible`/`IsWindowEnabled` reflect style changes immediately, `WM_CANCELMODE` in `DefWindowProcW` releasing mouse capture when the receiving window holds it (posting `WM_CAPTURECHANGED` to itself), `WM_HELP` forwarding to parent window in `DefWindowProcW` (same pattern as `WM_CONTEXTMENU`/`WM_MOUSEWHEEL`) | — | existing GWE fixtures, `coredll_raw_def_window_proc_handles_hit_test_and_syscommand_close`, `coredll_raw_def_window_proc_setcursor_uses_class_cursor`, `coredll_raw_def_window_proc_ncactivate_returns_true`, `coredll_raw_def_window_proc_activate_sets_focus`, `coredll_raw_set_capture_posts_capture_changed_to_old_owner`, `coredll_raw_def_window_proc_nclbuttondown_htclose_sends_syscommand_close`, `coredll_raw_def_window_proc_syskeydown_f4_closes_window`, `coredll_raw_def_window_proc_context_menu_forwards_to_parent`, `coredll_raw_def_window_proc_rbuttonup_sends_context_menu`, `coredll_raw_def_window_proc_rbuttondblclk_sends_context_menu`, `coredll_raw_def_window_proc_mousewheel_forwards_to_parent`, `coredll_raw_def_window_proc_setredraw_suppresses_and_restores_invalidation`, `coredll_raw_def_window_proc_syskeydown_f10_and_alt_up_send_keymenu`, `coredll_raw_def_window_proc_listbox_routing_returns_minus_one`, `coredll_raw_numlock_scrolllock_toggle_on_keydown`, `default_window_hit_test_reports_client_border_and_nowhere`, `coredll_raw_translate_message_uses_shift_caps_and_syschar`, `coredll_raw_keyboard_layout_and_imm_context_are_stateful`, `coredll_raw_translate_accelerator_honors_modifiers_and_syskey`, `coredll_raw_send_notify_message_is_async_across_threads`, `coredll_raw_send_notify_broadcast_uses_notify_send_for_live_top_level_windows`, `coredll_raw_send_message_timeout_rejects_non_ce_flags_without_queueing`, `coredll_raw_send_message_timeout_nonzero_cross_thread_queues_transaction`, `coredll_raw_send_message_timeout_writes_zero_result_when_target_destroyed`, `terminating_sender_removes_queued_sync_send_before_delivery`, `terminating_sender_marks_active_sync_send_without_result_ready`, `coredll_raw_get_message_services_sent_before_posted`, `coredll_raw_peek_message_services_sent_before_posted`, `rotate_to_ready_parked_process_with_pending_sent_receiver`, `timeslice_prefers_suspended_receiver_with_pending_sent_message`, `clipboard_raw_ordinals_track_lock_owner_formats_and_names`, `clipboard_delayed_render_requests_owner_and_accepts_rendered_handle`, `empty_clipboard_notifies_previous_owner_and_reassigns_owner`, `destroying_clipboard_owner_clears_open_owner_and_unrendered_formats`, `destroying_clipboard_owner_requests_render_all_and_keeps_rendered_data`, `clipboard_raw_ordinals_reject_invalid_open_and_missing_lock`, `clipboard_data_alloc_copies_known_local_handle_contents`, `clipboard_data_alloc_rejects_unknown_source_handle`, `coredll_raw_caret_ordinals_track_position_visibility_and_blink`, `coredll_raw_visible_caret_invalidates_and_marks_framebuffer_dirty`, `coredll_raw_caret_rejects_invalid_state_and_destroys_with_owner`, `mount_unmount_broadcasts_wm_devicechange_to_top_level_windows`, `coredll_raw_system_parameters_info_spif_sendchange_broadcasts_wm_settingchange`, `coredll_raw_def_window_proc_syskeydown_letter_sends_keymenu`, `coredll_raw_set_system_time_broadcasts_wm_timechange`, `coredll_raw_add_remove_font_resource_broadcasts_wm_fontchange`, `coredll_raw_def_window_proc_scroll_and_item_messages_forward_to_parent`, `coredll_raw_translate_accelerator_char_msg_fires_non_fvirtkey_accel`, `coredll_raw_def_window_proc_ctlcolor_returns_sys_color_brush`, `coredll_raw_activate_keyboard_layout_posts_wm_inputlangchange_to_focus`, `coredll_raw_def_window_proc_getminmaxinfo_fills_screen_size`, `coredll_raw_def_window_proc_syskeydown_digit_sends_keymenu`, `coredll_raw_change_display_settings_ex_broadcasts_wm_displaychange`, `coredll_raw_caret_blink_timer_fires_after_blink_time`, `coredll_raw_caret_hides_on_focus_change`, `coredll_raw_show_caret_xors_pixels_into_framebuffer`, `coredll_raw_begin_paint_erases_caret_end_paint_restores`, `coredll_raw_caret_blink_xors_framebuffer_via_peek_message`, `coredll_raw_def_window_proc_setfont_getfont_and_compareitem_forward`, `coredll_raw_def_window_proc_erasebkgnd_respects_hbr_background`, `coredll_raw_def_window_proc_nextdlgctl_moves_focus_and_updates_defid`, `send_message_gettextlength_returns_title_utf16_length`, `coredll_raw_send_notify_broadcast_delivers_in_z_order`, `coredll_raw_send_message_timeout_abortifhung_aborts_when_thread_is_hung`, `send_message_nccreate_returns_true`, `send_message_menuchar_returns_mnc_ignore`, `coredll_raw_def_window_proc_nccreate_returns_true`, `coredll_raw_set_window_long_style_syncs_visible_and_enabled_state`, `coredll_raw_is_dialog_message_tab_skips_disabled_and_invisible_controls`, `coredll_raw_def_window_proc_nextdlgctl_to_non_pushbutton_preserves_defid`, `coredll_raw_def_window_proc_settext_triggers_invalidate`, `coredll_raw_def_window_proc_getdlgcode_for_radio_edit_static`, `coredll_raw_is_dialog_message_vk_return_is_consumed_by_dialog`, `coredll_raw_is_dialog_message_vk_escape_is_consumed_by_dialog`, `coredll_raw_end_dialog_stores_result_and_rejects_invalid_hwnd`, `coredll_raw_check_radio_button_sets_exclusive_check_state`, `coredll_raw_def_window_proc_getdlgcode_for_pushbutton_variants`, `coredll_raw_def_window_proc_paint_validates_update_region`, `coredll_raw_get_version_ex_w_fills_ce_os_info`, `coredll_raw_register_window_message_returns_consistent_atom`, `coredll_raw_get_desktop_window_returns_fixed_handle`, `parent_exits_and_child_window_pump_stays_alive`, `coredll_raw_translate_message_western_layout_dead_keys`, `coredll_raw_translate_message_western_layout_ctrl_and_altgr`, `coredll_raw_ext_text_out_w_draws_glyphs_into_framebuffer`, `coredll_raw_gwe_wm_seticon_and_geticon`, `coredll_raw_gwe_def_window_proc_enable_invalidates_window`, `coredll_raw_gwe_wm_parentnotify_on_create_and_destroy`, `coredll_raw_gwe_wm_parentnotify_suppressed_by_noparentnotify`, `coredll_raw_def_window_proc_cancelmode_releases_capture`, `coredll_raw_def_window_proc_help_forwards_to_parent` | hidden chrome is guest-driven; do not force visibility; `target\host_presenter_sendorder1_*` closed the earlier startup sent-message starvation and `target\host_presenter_sendfix1_*` closes the later frozen-frame send gap |
| Winsock | CE Winsock headers and socket wait behavior | none | direct host socket traps, CE-facing isolated local names/IPs, socket read/write readiness helpers including non-consuming UDP datagram probes, scheduler wake-candidate queuing for socket-handle waiters, `select`/listener readiness probing that buffers pending accepted TCP streams for `accept`, successful `send`/`sendto` wake fan-out to socket wait candidates, stateful `FIONBIO` plus `SO_RCVTIMEO`/`SO_SNDTIMEO` `setsockopt`/`getsockopt` with host TCP/UDP timeout application and `SO_ERROR` `getsockopt`, and Unicorn `accept`/`recv`/`recvfrom` plus read-only infinite/finite-timeout, write-only pending-socket finite-timeout, mixed read/write masked, except-fdset finite-timeout, socket-error-ready, and OOB-ready `select` parking/replay through scheduler `WinsockRead` waiters with read/write/except readiness metadata; `SO_RCVTIMEO` now bounds parked Winsock receive waits, including UDP `recvfrom`, and direct nonblocking `recv`/`recvfrom`/`accept` imports bypass parking so raw Winsock returns `WSAEWOULDBLOCK` without populating `SO_ERROR` | broader Winsock nonblocking/timeout/OOB edge cases as traces demand them | winsock startup/loopback/isolation tests, `winsock::tests::scheduler_waiter_can_use_socket_read_readiness`, `winsock::tests::select_buffers_tcp_listener_readiness_for_accept`, `winsock::tests::udp_select_readiness_waits_for_datagram_before_recvfrom`, `winsock::tests::tcp_send_queues_peer_socket_wait_candidate`, `winsock::tests::socket_timeouts_round_trip_through_setsockopt_getsockopt`, `winsock::tests::socket_timeouts_apply_to_host_tcp_and_udp_sockets`, `winsock::tests::so_error_tracks_failed_connect_and_drives_exception_select`, `winsock::tests::nonblocking_recv_without_data_returns_wouldblock_without_waiting`, `winsock::tests::nonblocking_recvfrom_without_datagram_returns_wouldblock_without_waiting`, `winsock::tests::nonblocking_accept_without_client_returns_wouldblock_without_waiting`, `emulator::unicorn::wait_scheduler_tests::winsock_recv_parks_until_socket_read_ready_and_replays_import`, `emulator::unicorn::wait_scheduler_tests::winsock_recvfrom_parks_until_udp_datagram_ready_and_replays_import`, `emulator::unicorn::wait_scheduler_tests::winsock_recv_uses_so_rcvtimeo_for_blocked_wait_timeout`, `emulator::unicorn::wait_scheduler_tests::winsock_recvfrom_uses_so_rcvtimeo_for_blocked_wait_timeout`, `emulator::unicorn::wait_scheduler_tests::winsock_nonblocking_recv_does_not_park_and_returns_wouldblock`, `emulator::unicorn::wait_scheduler_tests::winsock_nonblocking_recvfrom_does_not_park_and_returns_wouldblock`, `emulator::unicorn::wait_scheduler_tests::winsock_nonblocking_accept_does_not_park_and_returns_wouldblock`, `emulator::unicorn::wait_scheduler_tests::winsock_accept_parks_until_listener_read_ready_and_replays_import`, `emulator::unicorn::wait_scheduler_tests::winsock_select_parks_until_read_fdset_socket_ready_and_replays_import`, `emulator::unicorn::wait_scheduler_tests::winsock_select_finite_timeout_resumes_with_zero_ready_count`, `emulator::unicorn::wait_scheduler_tests::winsock_select_write_fdset_parks_pending_socket_until_timeout`, `emulator::unicorn::wait_scheduler_tests::winsock_select_mixed_read_write_fdsets_use_masked_readiness_until_timeout`, `emulator::unicorn::wait_scheduler_tests::winsock_select_except_fdset_parks_until_timeout_and_clears_fdset`, `emulator::unicorn::wait_scheduler_tests::winsock_select_except_fdset_resumes_when_socket_error_becomes_ready`, `emulator::unicorn::wait_scheduler_tests::winsock_select_except_fdset_resumes_when_oob_data_becomes_ready` | host transport works, subnet identity is now CE-facing |
| Remote server/tooling | `CeRemote` API shape from v2 plus generic v3 framebuffer/audio/input state | `..\WinCE_Emulator_v2\src\remote_server.cpp`, `ce_remote.h` | v2-compatible REST status/frame/debug/input/GPS/NMEA/IMU/pause/resume/logs endpoints, MJPEG, control WebSocket, audio WebSocket, bounded recent-log publishing from `CeRemote`, direct control-WS log/status replies, mounted control/audio WS handshake validation | external-client/watch validation as remote tooling evolves, mounted real-app audio PCM evidence once the app emits wave data, broader live log streaming if external tooling demands it | remote server REST/WS tests, `recent_log_lines_follow_limit_and_clamp`, `remote_server_serves_recent_logs_over_rest_and_control_ws`, `remote_server_control_websocket_status_returns_latest_status_without_queueing`, `target\remote_ws_validation_parallel_probe.json`, `target\host_presenter_once_control_ws.json`, `target\host_presenter_sendorder1_probe_samples.json` | host-presenter validation on `--remote-server 192.168.0.39:8765` is live at PID `48432`; audio WS metadata works, binary late-join evidence remains watch |

## Key Changes

- Add a real guest module manager for dynamically loaded MIPS DLLs. Track
  normalized module name, guest path, host path, base, size, exports by
  name/ordinal, dependencies, refcount, TLS callbacks, entrypoint, flags, and
  unload state.
- Extend PE runtime loading so guest DLLs are mapped, relocated, patched for
  COREDLL traps, resolved against other guest DLLs, and reused by normalized
  module identity.
- Replace broad COREDLL fallback behavior with audited explicit outcomes:
  `safe no-op`, `safe failure`, or `must implement`; every app-hit
  must-implement stub needs actionable callsite diagnostics and shell/UI paths
  should prefer explicit unsupported failure over plausible fake success.
- Implement `ShellExecuteEx` above `CreateProcessW`: normalize path, resolve CE
  shortcuts, query associations, expand command templates, locate executable,
  launch, apply show state, and return correct status. The current raw path
  now preserves `nShow`, `lpDirectory`, `SEE_MASK_NOCLOSEPROCESS`, current CE
  process-directory fallback for relative targets, `.lnk` stored arguments
  during shortcut launch, and the CE `"%1"`, `%L`/`%l`, and `%*`
  command-template shape plus the `urlfile` `-u%1` embedded-target form; HKLM
  `Software\Classes` document associations are honored; existing extensionless
  documents now route through the CE `HKCR\file` generic association while
  missing extensionless names still report `SE_ERR_FNF`; failed direct `.exe`
  launches, missing associated non-EXE documents, and association commands
  with missing absolute executables now report `SE_ERR_FNF` while unassociated
  documents and empty association commands report `SE_ERR_NOASSOC`; finish the
  remaining fields/flags, broader non-EXE document edge cases, and broader
  command-template substitutions as traces demand them.
- Expand registry-backed shell behavior for associations, icons, shell folders,
  startup items, active devices, storage profiles, app paths, rendered
  recent-doc UI, and UI policy/settings. Recent-doc MRU pruning now caps stored
  shortcuts at the CE `MAX_RECENT_FILES` count of 10; path-backed and opaque
  namespace `SHARD_PIDL` input creates recent shortcut/MRU records; and shell
  state now exposes ordered recent-document display entries for later UI/menu
  rendering.
- Implement shell-visible UI APIs that currently still succeed too cheaply:
  `SHGetFileInfo`, `ImageList_*`, `TrackPopupMenu*`, modal `MessageBoxW`,
  notification icons and notification data, shell namespace/storage
  presentation, file type metadata, and shell/file-change notifications.
  Matching raw file operations now post CE-style `WM_FILECHANGEINFO` path
  payloads to registered shell sinks. `SHGetFileInfo(SHGFI_SYSICONINDEX)` now
  backs the returned system image-list handle with queryable pseudo icons,
  file-loaded BMP and RT_BITMAP DIB image-list entries preserve backing pixel
  rows, split wide bitmap strips into per-index source rectangles, masked
  transparent colors, mask-bitmap handles, and registered overlay images for
  framebuffer and selected-memory-DIB `ImageList_Draw*` blits, and user-created image lists track
  file/resource bitmap loading, image/icon entries,
  replacement, copying, merging, icon-size updates, size/count queries,
  background color, duplication, removal, draw attempts, and drag-image
  begin/move/show/cursor/query/end state.
  `GetDiskFreeSpaceExW` now reports object-store capacity for object-store
  paths and mount-specific total/free byte counts for mounted roots such as
  `\SDMMC Disk`, with the lower-level `AFS_GetDiskFreeSpace` sector/cluster
  ordinal backed by the same volume model, giving shell file/storage callers a
  volume-aware size view. `CeFsIoControlW`/`AFS_FsIoControlW`
  `FSCTL_GET_VOLUME_INFO` and `CeGetVolumeInfoW` now report `CE_VOLUME_INFO`
  for mounted roots and the object store, including storage/removable versus
  RAMFS flags, default block size, and fixed CE store/partition names;
  `FSCTL_REFRESH_VOLUME` and `FSCTL_FLUSH_BUFFERS` succeed as no-ops
  matching CE RAMFS/object-store semantics. The
  direct AFS path ordinals for create/remove directory, get/set attributes,
  create file, delete, move/presto-chango rename, and find-first now reuse the
  CE file namespace for full guest paths. Root `FindFirstFileW` namespace
  enumeration now merges visible mount folders with object-store root entries
  while preserving CE mount-folder attributes such as temporary removable
  storage. `FSCTL_REFRESH_VOLUME` and `FSCTL_FLUSH_BUFFERS` are now handled
  as no-ops (CE RAMFS/object-store semantics: no dirty cache to flush or FAT
  to reload). Fuller mount-table volume-handle semantics remain queued. Public
  `FindFirstChangeNotificationW`/`FindNextChangeNotification`/
  `FindCloseChangeNotification` and direct
  `AFS_FindFirstChangeNotificationW` now create waitable directory-change
  handles that signal on matching file create/delete/rename/write/attribute
  events, expose pending `FILE_NOTIFY_INFORMATION` action/name records through
  `CeGetFileNotificationInfo`, coalesce consecutive duplicate action/name
  records plus transient create/delete and modified/delete churn, and rearm via
  `FindNext` or successful info retrieval.
  `MessageBoxW` now validates owner HWNDs, records text/caption/style plus
  button IDs, default-button index, icon class, owner enabled-state, and result
  in `ShellSystem`, returns the CE default button for the requested button
  group, including owner-draw `MB_YESALL` and `MB_CANCEL` modes from
  `OWNERDRAWLIB\appalert.cpp`, and consumes queued modal Enter/Escape key and
  character input, direct button and dialog-client hit-tested button-down/release input, dialog `WM_COMMAND`, dialog
  `WM_CLOSE`, and dialog `WM_SYSCOMMAND/SC_CLOSE` input for the transient
  dialog before falling back to that default result, while dispatching queued
  posted and sent same-thread non-dialog messages ahead of later dialog input;
  the full blocking wait and broader live interaction remain queued.
- Improve GWE defaults for focus, activation, non-client messages, hit testing,
  client `WM_SETCURSOR` class-cursor selection, non-client
  `WM_SETCURSOR` stock sizing/arrow cursor selection, `WM_MOUSEACTIVATE`
  activation, keyboard navigation, mouse capture, close/destroy, command
  routing, dialog and control notifications, and `SendNotifyMessageW`
  ordering. Dialog default-button state now reports CE `DM_GETDEFID`/
  `DM_SETDEFID` values over child pushbuttons, and Escape handling resolves an
  actual `IDCANCEL` pushbutton HWND before falling back to a command id only.
- Expand input and UI state fidelity beyond the current minimal
  `TranslateMessage`: keyboard modifiers, layout/HIMC probes, now-covered
  hardware `PostKeybdMessage` routing through keyboard target/focus fallback
  plus compact/split character-buffer shapes, accelerators and menu shortcuts
  including translated-character ASCII accelerator matching,
  `WM_SYSKEY*`, broader keymap selection beyond the now-covered ASCII
  letters/digits/space/Ctrl control characters/numpad digits and operators/
  common OEM punctuation/VK_F1–F24 no-char guard/VK_SEPARATOR no-char guard,
  dead-key/IME composition
  handoff, remaining clipboard
  guest/nested render fulfillment edges above the now-stateful raw clipboard
  format store, `GetClipboardDataAlloc` local-handle copy path, delayed
  `WM_RENDERFORMAT` owner request/fulfillment path including duplicate-request
  suppression while a delayed render is active, `WM_RENDERALLFORMATS`
  owner-destroy request, and `WM_DESTROYCLIPBOARD` replacement notification,
  visible-caret invalidation/framebuffer dirtying above the now-stateful raw
  caret store, rendered caret XOR bitmap drawn directly into the framebuffer on
  show/hide/move and on blink transitions via `GetMessageW`/`PeekMessageW`
  with `BeginPaint` erase and `EndPaint` restore matching CE GWES paint-cycle
  semantics, and focus/capture integration.
- Complete cross-thread `SendMessageTimeout` semantics: wait for reply or
  timeout, honor remaining `SMTO_*` flags, and perform required reentrant
  dispatch while waiting. Cross-thread dispatch completion now writes
  `lpdwResult` for timeout transactions, raw receiver destruction returns a
  terminated zero result and flushes it to `lpdwResult`, and sender termination
  removes undelivered sends or marks active sends with `SMF_SENDER_TERMINATED`
  before receiver unwind.
- Replace direct-only WINSOCK with a CE-facing isolated NAT model: gateway
  `10.0.0.1`, guest IP `10.0.0.2`, host sockets as the transport boundary, and
  blocking waits owned by the scheduler. Socket-handle waiters can now be
  queued and selected from host read readiness; TCP listener readiness now
  probes nonblocking `accept` and buffers the accepted stream so `select` and
  subsequent `accept` observe the same pending connection; successful
  `send`/`sendto` calls now fan out wake candidates to socket-handle waiters
  so peer read readiness can resume blocked work; UDP read readiness now uses
  non-consuming datagram probes instead of treating every UDP socket as ready,
  so empty `recvfrom` waits can park until a datagram arrives; Unicorn
  `accept`/`recv`/`recvfrom`, read-only infinite or finite-timeout `select`, write-only
  pending-socket finite-timeout `select`, mixed read/write masked `select`,
  and except-fdset finite-timeout `select` can now park as scheduler
  `WinsockRead` waiters with read/write/except readiness metadata and replay
  the saved import once the socket becomes ready or the finite `select`
  deadline expires. `SO_RCVTIMEO`/`SO_SNDTIMEO` now round-trip through
  `setsockopt`/`getsockopt`, apply onto host TCP/UDP read/write timeouts as
  sockets are connected, bound, or updated, and `SO_RCVTIMEO` bounds parked
  Winsock `recv` and UDP `recvfrom` waits before replaying the saved import to
  produce the Winsock timeout error shape. `SO_ERROR` now reports and clears stored socket errors, failed
  connects drive raw `select` exception fdsets, and parked Unicorn
  exception-fdset `select` resumes/replays once socket-error readiness appears.
  `FIONBIO` mode is now retained after `ioctlsocket`, and direct nonblocking
  `recv`/`recvfrom`/`accept` imports bypass scheduler parking so raw Winsock
  returns `WSAEWOULDBLOCK` immediately without filling `SO_ERROR`. `MSG_OOB`
  sends now mark the connected peer's exception readiness independently of
  `SO_ERROR`, raw and parked Unicorn `select` report that peer in `exceptfds`,
  and `recv(..., MSG_OOB)` clears the OOB readiness after consuming the data.
  Broaden socket timeout/nonblocking/OOB edge coverage as traces demand it.

## Validation

- Add eVC fixtures under `tests\test_progs` for guest DLL loading, refcounts,
  exports, shell execution, shortcuts, window defaults, notifications, and
  Winsock waits.
- eVC fixture tooling discovered during implementation:
  `C:\Program Files (x86)\Microsoft eMbedded C++ 4.0\Common\EVC\Bin\RC.EXE`
  is the resource compiler path for `.rc` fixtures.
- Add Rust tests for module normalization/refcounts, export lookup, shell
  registry parsing, stub audit classification, and isolated Winsock address
  translation.
- Recurring checks: `cargo fmt`, `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test --features unicorn,trace,win32-desktop`, and a mounted iNavi run
  with `--remote-server 192.168.0.39:8765`.

## Queued Loader Gaps

These remaining gaps and coverage watchpoints must stay queued until closed by
focused fixtures plus mounted iNavi evidence:

- Extend runtime guest-DLL fixtures as more lifecycle behavior lands.
  `tests\test_progs\171_loadlibrary_guest_dll` now proves the
  direct export-call path with one DLL containing `DllMain`, one named export,
  one ordinal export, and one EXE that calls `LoadLibraryW`, `GetProcAddress`,
  invokes both exports, asserts a single `DllMain(DLL_PROCESS_ATTACH)` call
  across handle reuse, verifies no detach on non-final `FreeLibrary`, and
  proves final `FreeLibrary` runs `DllMain(DLL_PROCESS_DETACH)` by writing a
  DLL detach marker into EXE-owned memory.
- Keep non-Unicorn/raw `LoadLibraryW` / `LoadLibraryExW` behavior aligned where
  practical. Raw `LoadLibraryExW` now accepts supported no-resolve/datafile
  flags when the target is COREDLL or an already registered module and
  refcounts reused modules like `LoadLibraryW`; when the raw path cannot map a
  new DLL, it fails explicitly with `ERROR_FILE_NOT_FOUND` or
  `ERROR_NOT_SUPPORTED` rather than silently pretending that a load occurred.
- Keep the core-library vs guest-DLL boundary audited. Emulator-provided API
  modules must be intentionally classified; non-core libraries should load as
  guest PE DLLs or fail loudly; unresolved/unclassified imports need compact
  diagnostics so user MIPS libraries are not accidentally translated as host
  stubs.
- Extend the dependent-DLL runtime fixture as more lifecycle behavior lands.
  `tests\test_progs\172_loadlibrary_dependent_guest_dll` now proves DLL A
  importing from DLL B through the PE import table, recursive runtime
  dependency loading, guest-DLL-to-guest-DLL import resolution through the
  module manager, and one `DllMain(DLL_PROCESS_ATTACH)` call for both the
  dependency and dependent DLL.
- Keep lifecycle fixtures current as loader behavior expands. Final
  `FreeLibrary` now invokes guest TLS callbacks and then
  `DllMain(..., DLL_PROCESS_DETACH, 0)` before the module is marked
  unload-pending, and mapped address ranges remain reserved.
- Keep forwarded export coverage current. `GetProcAddress` and import patching
  now resolve PE forwarder strings through already loaded modules or trigger
  CE search/load of the forwarded-to DLL, with COREDLL forwarders returning
  dynamic trap addresses. Add more fixture variants as real dumped DLL traces
  demand them; unresolved or unsupported forwarders must continue to fail
  loudly, not return a plausible fake pointer.
- Keep `DONT_RESOLVE_DLL_REFERENCES` coverage current. The Unicorn runtime path
  now maps/reuses the module and exposes ordinary exports without recursive
  dependency loading, import patching, TLS callbacks, or `DllMain`; final
  `FreeLibrary` also skips detach callouts for modules that never received
  attach callouts. Add more fixture variants for forwarded exports or unsafe
  unresolved-IAT execution paths as traces demand them.
- Keep datafile/resource-style `LoadLibraryExW` coverage current. The Unicorn
  runtime path now treats `LOAD_LIBRARY_AS_DATAFILE` as no-resolve, maps the
  module for resource access, immediately registers strings/resource entries
  with `kernel.resources`, and exposes `FindResourceW`/`LoadResource`/
  `LoadStringW` behavior without code exports, dependency loads, TLS callbacks,
  or `DllMain`. Add more resource variants as dumped DLL traces demand them.
- Keep runtime trampoline handling current. Runtime normal/no-resolve DLL loads
  now run the MIPS Unicorn trampoline patcher before mapping, place inline
  stubs inside the runtime DLL image, publish the new ranges/jumps into both
  persistent emulator metadata and the live full-code-hook state, and skip this
  for datafile/resource-only loads. Add fast-start dynamic-hook variants only
  when traces show runtime DLL code executing under `WINCE_EMU_FAST_START`.
- Keep compact loader audit counters current. Debug summaries and monitor
  `loader` output now include runtime load attempts, successful maps,
  dependency loads, export lookups/misses, forwarded exports, TLS callbacks,
  `DllMain` attach/detach calls, and loud loader failures.

## Queued Shell And UI Gaps

- Keep must-implement stub hits fail/actionable: ordinal, name, caller PC,
  owning module where available, thread id, and explicit unsupported result for
  shell/UI/process/loader paths. Caller module attribution is now populated
  for mapped EXE/DLL import-trap callers, and raw fallback records now set
  `ERROR_NOT_SUPPORTED` plus failure-shaped return values. The remaining work
  is replacing those stubs with real subsystem behavior and adding any
  non-mapped callsite attribution discovered by mounted traces.
- Extend `SHGetFileInfo` beyond the current registry-backed slice: actual icon
  bitmap extraction from DLL/icon resources, fuller bitmap-backed image-list
  blitting beyond file-loaded BMP/RT_BITMAP DIB pixel draws with
  transparent-color, mask-bitmap masking, and registered overlay-image
  composition, and mounted-trace edge cases.
  The shell system image-list handle is
  now queryable through raw `ImageList_*` ordinals, and created image lists keep
  state for file/resource bitmap loading, image/icon entries, replace/copy and
  merge operations, icon-size updates, count/size/image-info queries,
  duplication, background color, removal, `GetIcon` overlay identity,
  framebuffer- and selected-memory-DIB-backed pseudo-icon draw attempts,
  file-loaded BMP and RT_BITMAP DIB row-backed `ImageList_Draw*` pixel blits
  with wide-bitmap strip source rectangles into framebuffers and selected
  memory DIBs, `ImageList_AddMasked`/`LoadImage` mask colors,
  `ImageList_Add` mask bitmap handles, and `ImageList_SetOverlayImage`
  bitmap-backed overlay composition, and drag-image lifecycle/query calls.
  Empty-path
  `SHGetFileInfo(SHGFI_SYSICONINDEX)` calls now return the shell system
  image-list handle for CE shell list/taskbar callers that are only probing
  the shared image list.
  `LoadIconW` stock and `RT_GROUP_ICON` resource handles now share stricter
  `DestroyIcon` validation with shell pseudo icon handles, rejecting arbitrary
  nonzero values and non-icon resources, raw `CreateIconIndirect` now
  validates `ICONINFO` mask/color bitmaps and registers state-backed synthetic
  icon handles that `DestroyIcon` releases, and raw `DrawIconEx` validates
  icon/HDC handles while painting deterministic pseudo-icon rectangles into
  attached window framebuffers and selected memory DIBs.
  Raw `ExtractIconExW` now supports `nIconIndex == -1` count probes and extracts
  real PE icon bitmaps (color XOR and AND mask) from RT_ICON/RT_GROUP_ICON
  resources in EXE/DLL files, writing large/small output `HICON`s with real
  color+mask bitmap handles; fallback to synthetic icon for index 0 on
  non-PE paths.
  CE-supported flag validation now rejects unsupported/colliding `SHGFI_*`
  combinations instead of accepting desktop-only icon-location/masked-attribute
  requests, and registry `DefaultIcon` plus CE-ordered document/folder/
  executable/storage-card/network-folder special icon slots now produce stable
  pseudo image-list identities, and `.lnk` files report the CE `Shortcut` type
  while following bounded nested resolved targets for icon selection and
  preserving the CE shortcut overlay in returned pseudo `HICON` handles.
  Missing real paths now follow CE `SHGetFileInfo` by failing with
  `ERROR_MOD_NOT_FOUND` rather than the generic file-not-found status.
  Removable mount roots that surface as `FILE_ATTRIBUTE_DIRECTORY |
  FILE_ATTRIBUTE_TEMPORARY` now use a storage-card pseudo icon/display/type
  slice, while temporary `\\share`, `\release`, and `\network` directories use
  the CE icon-cache network-folder branch with matching type metadata; missing
  UNC paths also follow CE by falling back to that temporary network-folder
  metadata and `SFGAO_FILESYSTEM | SFGAO_FOLDER` shell attributes instead of
  failing unknown-file probing.
- Implement real popup menu behavior for `TrackPopupMenu*`: full nested modal
  menu pump, deeper submenu behavior, and live cancellation edge cases.
  The raw path now records tracking/owner loop notifications, routes the
  state-backed default `WM_COMMAND` when `TPM_RETURNCMD` is not requested, and
  `TPM_RETURNCMD` returns the enabled default command (or first enabled command)
  from the tracked popup menu instead of an unconditional fake cancellation.
  That state-backed command selection now descends through enabled `MF_POPUP`
  submenus and sends/records nested `WM_INITMENUPOPUP` notifications for the
  submenu path that produced the selected command; highlighted submenu panes
  now render beside their parent item and pointer hit-testing can select
  commands inside those rendered child panes; disabled-only menus cover
  the cancellation/no-command edge by returning `0` for `TPM_RETURNCMD` and
  avoiding synthetic `WM_COMMAND`. Framebuffer-backed raw calls now render the
  popup surface and item rows from menu state, including disabled, checked,
  default, separator, and submenu-marker presentation. Current-cursor
  hit-testing now selects enabled command rows and enabled submenu rows before
  falling back to the default/first-command path. Already queued owner-window
  `WM_KEYDOWN` menu navigation now consumes Up/Down/Enter/Escape, skips
  disabled/separator rows, routes Enter to the selected command/submenu command,
  opens highlighted submenu panes on Right-arrow for multi-level keyboard
  navigation and Enter selection, returns keyboard focus one pane upward on
  Left-arrow, and closes a child pane when pointer movement returns to a parent
  row or Escape is pressed inside the child pane,
  accepts queued `WM_CHAR` menu mnemonics from `&`-marked item text,
  keeps the current command/submenu row in menu `MF_HILITE` state with
  framebuffer-highlight rendering and repaints during queued modal input,
  and returns `0` on Escape cancellation; queued owner-window `WM_MOUSEMOVE`
  now updates that highlighted command/submenu row before Enter selection,
  while queued owner-window `WM_CANCELMODE`
  and outside-click messages also cancel, while left-click and `TPM_RIGHTBUTTON`
  right-click messages inside the popup select enabled rows from their
  screen-coordinate `lParam`; queued posted/sent same-thread non-owner messages,
  owner-window non-menu messages, and generated owner `WM_PAINT` are dispatched
  before default or later owner-window menu input resolves.
  The blocking modal wait now pumps timers before each re-entry `poll_once`
  call, and the `blocked_get_message` snapshot now carries the owner `hwnd` so
  that live remote touch is routed via `ThreadHitTest` to the owner window
  instead of the stale active window, closing the remaining live input routing
  gap.
  The modal pump now also implements CE's tap-only submenu UI: a button-down or
  button-up on a submenu parent opens the child pane (tracked as stacked
  `PopupMenuModalMenuState` levels with per-level hit testing and stack
  truncation) and a command is only returned once the user taps an item inside
  the child pane, instead of immediately following the hilite chain to the
  submenu's default command. Cascade geometry (child flips left when it would
  overflow the right/bottom screen edge) is now implemented. Hover-open timers
  for the non-tap `CMenu::s_fUseTapOnlyUI` path are now implemented: on
  mouse-present devices (`SM_MOUSEPRESENT != 0`) the pump sets a
  `POPUP_HOVER_TIMER_ID` timer after the pointer rests on a submenu parent for
  `GetDoubleClickTime()` ms (500 ms); the WM_TIMER fires, kills itself, and
  opens the child pane. Button-down in mouse mode no longer opens submenus
  immediately; button-up does.
- Continue real modal `MessageBoxW`: the raw path now validates owners, records
  text/caption/style, button IDs, CE owner-draw left/center/right button
  labels, default-button index, icon class, result, and owner enabled-state,
  temporarily applies owner disable/reenable, creates a transient visible GWE
  dialog with static text and button child windows, closes it through
  `EndDialog`, paints the dialog/caption/icon/text/button surface into an
  attached framebuffer, and derives the default `ID*` result from CE
  `winuser.h` button/default flags, including the CE owner-draw `MB_YESALL`
  and `MB_CANCEL` modes. It also consumes already queued modal
  Enter/Escape key and character input, direct button and dialog-client
  hit-tested button-down/release input, dialog
  `WM_COMMAND`, dialog `WM_CLOSE`, and dialog `WM_SYSCOMMAND/SC_CLOSE` messages addressed to that transient dialog/buttons
  and routes them to the final `ID*` result, while dispatching queued posted
  and sent same-thread non-dialog messages before later modal dialog input
  resolves; generated non-dialog `WM_PAINT` is dispatched before default
  fallback selection, and generated dialog `WM_PAINT` is also dispatched so
  transient update regions do not starve the modal pump.
  The blocking modal wait now pumps timers on each re-entry and routes live
  remote touch via `ThreadHitTest` targeting the dialog `hwnd` so that
  button-down/release events land on the correct dialog buttons rather than the
  stale active window; the `blocked_get_message` snapshot carries the dialog
  hwnd in both first-entry and re-entry paths.
  The dialog now starts with keyboard focus on the default button and the modal
  pump runs CE dialog-manager keyboard navigation: Tab/Shift+Tab and arrow keys
  move focus between the box's buttons, Space activates the focused button, and
  Enter activates the focused (or default) button, so a navigated button becomes
  the returned command. Remaining edge case is mnemonic accelerators for custom
  (owner-supplied) button captions.
- Extend file-change notification fidelity: shell `WM_FILECHANGEINFO` posting
  and the waitable `FindFirstChangeNotificationW` handle path now share the
  same CE filesystem mutation source, and `CeGetFileNotificationInfo` now
  returns pending action/name records including recursive directory create,
  rename old/new multi-record chains, directory removal, and root namespace
  mount add/remove records. It now also mirrors CE partial-drain behavior for
  capped buffers: too-small probes preserve pending records, successful prefix
  reads return `ERROR_MORE_DATA` with the remaining byte count and keep the
  handle signaled, full drains report zero bytes available, and empty data
  reads fail with `ERROR_NO_MORE_ITEMS`. Consecutive duplicate action/name
  records, transient create/delete pairs, and modified/delete pairs are
  coalesced before pending delivery. Full FSDMGR volume-handle notification
  ownership and broader mounted edge cases remain queued.
- Finish shell special-folder behavior: confirm CSIDL coverage against observed
  calls and mounted edge cases. The raw path now records each resolved
  `SHGetSpecialFolderPath` query with registry/missing/non-string source
  classification, supports a selectable compat-vs-strict fallback policy,
  verifies the CE `shellapi.h` supported fallback set
  (`Desktop`/`DesktopDirectory`, `Programs`, `Personal`, `Favorites`,
  `Startup`, `Recent`, `Start Menu`, `Fonts`, `AppData`, `Windows`,
  `Program Files`, and `Profile`) while preserving CE `CSIDL_SYSTEM`
  compatibility coverage, rejects overlong resolved paths with focused
  coverage, and honors both `fCreate` and `CSIDL_FLAG_CREATE` by creating
  missing root-relative and nested mounted folders, including the CE shell's
  observed `CSIDL_DESKTOPDIRECTORY | CSIDL_FLAG_CREATE` desktop creation path.
- Extend notification APIs: `Shell_NotifyIconW` now tracks add/modify/delete
  records generically and can post the registered owner callback message for a
  shell-icon event; `SHNotificationAddI`/`UpdateI`/`RemoveI`/`GetDataI` now
  preserve/query CE notification data by `CLSID + dwID`, keep notification
  `grfFlags` separate from update masks, and can post a `WM_NOTIFY`/`NMSHN`
  callback payload to the stored sink HWND for notification
  events, including CE-style `SHNN_LINKSEL` `pszLink` strings stored after the
  `NMSHN` payload and `SHNN_DISMISS` `fTimeout` flags. The same stored
  notification events now record the CE `IShellNotificationCallback` method,
  vtable offset, and typed argument payload that would be invoked for non-null
  CLSIDs (`OnShow`, `OnLinkSelected`, `OnDismiss`, and command selection),
  while command selection also posts the CE `WM_COMMAND` sink message. Inform
  notifications now track expiry from their CE centisecond duration, remove the
  stored record on timeout, and post a timeout `SHNN_DISMISS` callback during
  message pumping. `SHNotificationUpdateI` now rejects zero/unknown update masks,
  preserves the previous notification icon for `SHNUM_ICON` updates that
  carry a null icon handle, and stores zero `SHNUM_DURATION` updates literally
  instead of reapplying add-time defaults, while leaving add-time `grfFlags`,
  `hwndSink`, and `lParam` intact. Notify icons now track CE `HHTBF_DESTROYICON`
  ownership and record old/current `HICON` handles that the taskbar would
  destroy during icon replacement, explicit deletion, or owner-window teardown.
  Add/update paths have raw coverage for rejecting nonzero notification sink
  HWNDs that are no longer live. Still implement actual guest COM vtable invocation from that recorded call contract and
  optional shell/taskbar rendering only when backed by real shell state.
  Notification records are pruned during normal HWND/process teardown, with
  raw coverage for both direct window destruction and current-process
  termination cleanup.
- Finish file namespace and file-change notifications: root namespace
  enumeration now surfaces visible mount folders alongside object-store root
  entries and lets mount metadata override duplicate backing directories.
  Continue storage-volume presentation, remaining file type/display/icon
  metadata, fuller recent-doc rendering, namespace policy, and mounted edge
  cases. Raw path-based and IDLIST-based create/delete/mkdir/rmdir/rename/update
  refresh events now deliver heap-backed `FILECHANGENOTIFY` payloads to matching
  `SHChangeNotifyRegisterI` subscribers.

## Completed User Todos

- Added `host_progs\cmd.cpp`: a basic desktop-Windows-like command prompt for
  Windows CE/eVC4 with `/C` and `/K` handling, a small GUI command window, and
  built-ins for `cd`, `dir`, `type`, `echo`, `copy`, `del`, `mkdir`, `rmdir`,
  `cls`, `pwd`, and `exit`, plus `CreateProcessW` fallback for executables.
  Verified it compiles and links with eVC4 MIPSII into
  `target\host_progs\cmd\cmd.exe`.
