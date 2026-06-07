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
| Guest DLL loader | `NK\KERNEL\loader.c`, `NK\KERNEL\thread.c`, PE loader and SDK loader headers | startup preload path only | startup import DLL registration, CE-aware DLL search order, live-mutable import trap table, static trap-base allocator with reserved dynamic range, live/persisted import trap-page refresh, runtime normal-code DLL map/relocate/import patching from `LoadLibraryW/LoadLibraryExW(flags=0)`, recursive dependency loading, runtime inline MIPS trampoline patching with live trampoline metadata updates, runtime `LoadLibraryExW(DONT_RESOLVE_DLL_REFERENCES)` mapping without dependency/import/lifecycle resolution, runtime `LoadLibraryExW(LOAD_LIBRARY_AS_DATAFILE)` resource-module mapping with immediate `FindResourceW`/`LoadResource`/`LoadStringW` visibility and no code exports, raw `LoadLibraryExW` supported-flag reuse of already registered modules, runtime loader audit counters in debug/monitor summaries, runtime module export table ingestion, module metadata/refcounts, registered-export lookup, PE forwarded-export metadata retention, runtime forwarder resolution through guest DLL search/load and COREDLL trap addresses, TLS callback metadata parsing, guest TLS and `DllMain(DLL_PROCESS_ATTACH)` callouts, final dynamic `FreeLibrary` guest TLS and `DllMain(DLL_PROCESS_DETACH)` callouts before non-reclaiming unload-pending release | fast-start runtime trampoline hook variants if trace evidence demands them, raw/non-Unicorn real DLL mapping remains explicit failure, broader forwarded-export/no-resolve/resource fixture variants | `dll_search`, import-trap tests, `refresh_import_trap_page_blob_replaces_existing_blob`, `runtime_occupied_ranges_include_memory_regions_and_blobs`, `loaded_module_export_snapshots_skip_unload_pending_modules`, `external_table_accepts_runtime_loaded_module_exports`, `external_table_resolves_forwarded_exports_through_loaded_modules`, `065_loadlibrary_getprocaddress_module`, `171_loadlibrary_guest_dll`, `172_loadlibrary_dependent_guest_dll`, `173_loadlibrary_tls_callback`, `174_loadlibrary_forwarded_export`, `175_loadlibrary_noresolve`, `176_loadlibrary_datafile_resource`, raw module refcount tests, `live_trampoline_state_extends_runtime_ranges_and_origin_maps`, `debug_summary_includes_runtime_loader_counters` | dumped DLLs from `D:\INAVI_Emulator\DUMPPLZ\Windows` are authoritative runtime bytes |
| COREDLL stub audit | COREDLL API sets and SDK headers | broad stub policy proved convenient but unsafe | static ordinal table, implemented/stubbed plan, fallback audit classification/logging, raw thread/caller/trap PC context, caller module attribution from mapped image/DLL blobs, must-implement fallback `ERROR_NOT_SUPPORTED` side effects, explicit failure-shaped fallback returns for critical loader/process/shell/UI APIs | replace must-implement stubs with real subsystem behavior as traces hit them | `raw_stub_audit_keeps_import_trap_context`, `mapped_blob_module_for_pc_attributes_image_and_runtime_dlls`, `tests\coredll_dispatch.rs` | shell/UI/process calls must not silently fake success |
| Shell launch/settings | `SHELL`, Explorer registry conventions, `PUBLIC\COMMON\SDK\INC\winuser.h` MessageBox/button/menu constants, `PUBLIC\COMMON\SDK\INC\shellsdk.h`, `PUBLIC\COMMON\SDK\INC\shsdkstc.h`, `PUBLIC\COMMON\SDK\INC\shellapi.h`, `PUBLIC\COMMON\SDK\INC\winbase.h`, `PUBLIC\SHELL\OAK\FILES\shell.reg`, `PUBLIC\SHELL\OAK\HPC\CESHELL\API\shortcut.cpp`, `PUBLIC\SHELL\OAK\HPC\CESHELL\API\api.cpp`, `PUBLIC\SHELL\OAK\HPC\CESHELL\UI\filechangemgr.cpp` | v2 shell path shape only | `SHGetSpecialFolderPath` registry/fallback lookup plus `fCreate`/`CSIDL_FLAG_CREATE` mounted-folder creation and MAX_PATH rejection, registry/shortcut-backed `ShellExecuteEx` launch chain with `hInstApp`, `SEE_MASK_NOCLOSEPROCESS` hProcess output, `nShow` propagation to child `WinMain`, `lpDirectory` current-directory propagation through pending launches, and CE `"%1" %*` association command-template expansion, raw `CreateProcessW` `pszCurDir` propagation, child relative-EXE resolution through queued/current CE directory, parked-process current-directory restore, raw `SHCreateShortcut` CE text-link creation, raw `SHGetShortcutTarget` CE text-link target extraction, raw `SHCreateShortcutEx` unique-name and optional buffer-return behavior, raw `SHAddToRecentDocs(SHARD_PATH/null)` recent-folder shortcut creation/clear plus `ShellSystem` recent-document state, raw `SHChangeNotifyRegisterI`/`SHFileNotifyRemoveI`/`SHFileNotifyFreeI` registration/free tracking with HWND cleanup, registry-backed `SHGetFileInfo` display/type/attribute/sysicon metadata with CE supported-flag validation and explicit `ERROR_INVALID_FLAGS` for unsupported/colliding flags, `TrackPopupMenuEx` tracked attempts plus CE menu-loop owner notifications honoring `TPM_NONOTIFY`, `Shell_NotifyIconW` add/modify/delete state and owner callback message posting over a dedicated `ShellSystem`, `SHNotificationAddI`/`UpdateI`/`RemoveI`/`GetDataI` data storage/query by `CLSID + dwID` plus `WM_NOTIFY`/`NMSHN` sink callback posting for stored notification events including `SHNN_LINKSEL` link strings and `SHNN_DISMISS` timeout flags, shell notification cleanup on HWND/process teardown, `MessageBoxW` owner validation plus recorded text/caption/style and CE default-button return selection | richer notification interaction (automatic timeout/dismiss policy, COM callback, bubble/taskbar UI), file-change event delivery/PIDL payloads, recent-doc pruning/UI display and PIDL input, fuller namespace/policy, real icon extraction/image lists, rendered popup/menu windows and real selection, rendered modal `MessageBoxW` window/nested pump/button input, precise shell errors and broader non-EXE document edge cases | raw `ShellExecuteEx` association tests, `shell_execute_ex_appends_parameters_without_template_placeholder`, `coredll_raw_create_process_preserves_current_directory`, `process_launch_resolution_uses_queued_current_directory`, parked-process identity tests, `shell_shortcut_ordinals_create_read_and_launch_ce_lnk_files`, `shell_create_shortcut_ex_returns_unique_name_and_checks_output_capacity`, `shell_add_to_recent_docs_creates_and_clears_recent_shortcuts`, `sh_change_notify_i_tracks_register_remove_and_free_state`, `coredll_raw_shget_special_folder_path_honors_create_flags`, `sh_get_file_info_uses_registry_associations_and_attributes`, `sh_get_file_info_rejects_unsupported_and_colliding_flags`, `coredll_raw_track_popup_menu_records_attempt_without_fake_selection`, `shell_notify_icon_tracks_add_modify_delete_and_posts_callback`, `shnotification_i_tracks_query_update_and_remove_state`, `shell_window_destroy_removes_notify_icon_and_notification_state`, `message_box_w_records_text_owner_and_returns_default_button`, `message_box_w_rejects_destroyed_owner_without_recording` | route UI still needs real shell/process/GWE semantics |
| GWE defaults/input/send | `GWE\INC`, `window.hpp`, CE `winuser.h`, MFC `wincore.cpp`, CE input/clipboard/caret/IME headers, `PUBLIC\COMMON\SDK\INC\winuser.h` clipboard/caret/accelerator/timeout constants/prototypes | window broker behavior evidence | core window/message pump, partial defaults, raw/Unicorn-default `DefWindowProcW` `WM_NCHITTEST` and `WM_SYSCOMMAND/SC_CLOSE` handling, raw `TranslateMessage` ASCII letter/digit Shift/CapsLock and `WM_SYSKEYDOWN` to `WM_SYSCHAR` path, raw `TranslateAcceleratorW` modifier and `WM_SYSKEYDOWN` matching, raw `SendNotifyMessageW` same-thread direct delivery plus cross-thread/broadcast live-top-level no-wait sent-queue delivery, raw `SendMessageTimeout` CE `SMTO_NORMAL` flag validation plus timeout-transaction queueing, partial cross-thread send, GWE-owned clipboard open/close/owner/data/format-enumeration/custom-format-name state plus `GetClipboardDataAlloc` local-handle copy through raw COREDLL clipboard ordinals, GWE-owned caret create/destroy/show-hide/position/blink-time/system-enable state through raw COREDLL caret ordinals | fuller `DefWindowProcW` focus/activation/cursor/mouse/default-command behavior, `SendNotifyMessageW` shell/device/storage notification integration and remaining ordering edge cases, real keyboard layout selection, broader accelerator/menu shortcut routing, dead keys/IME handoff, clipboard rendered ownership/delayed-render edge cases, rendered caret blink/focus invalidation integration, full `SendMessageTimeout` wait/result/destruction/reentrancy semantics | existing GWE fixtures, `coredll_raw_def_window_proc_handles_hit_test_and_syscommand_close`, `default_window_hit_test_reports_client_border_and_nowhere`, `coredll_raw_translate_message_uses_shift_caps_and_syschar`, `coredll_raw_translate_accelerator_honors_modifiers_and_syskey`, `coredll_raw_send_notify_message_is_async_across_threads`, `coredll_raw_send_notify_broadcast_uses_notify_send_for_live_top_level_windows`, `coredll_raw_send_message_timeout_rejects_non_ce_flags_without_queueing`, `clipboard_raw_ordinals_track_lock_owner_formats_and_names`, `clipboard_raw_ordinals_reject_invalid_open_and_missing_lock`, `clipboard_data_alloc_copies_known_local_handle_contents`, `clipboard_data_alloc_rejects_unknown_source_handle`, `coredll_raw_caret_ordinals_track_position_visibility_and_blink`, `coredll_raw_caret_rejects_invalid_state_and_destroys_with_owner` | hidden chrome is guest-driven; do not force visibility |
| Winsock | CE Winsock headers and socket wait behavior | none | direct host socket traps, CE-facing isolated local names/IPs | scheduler-backed blocking waits | winsock startup/loopback/isolation tests | host transport works, subnet identity is now CE-facing |

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
  now preserves `nShow`, `lpDirectory`, `SEE_MASK_NOCLOSEPROCESS`, and the CE
  `"%1" %*` command-template shape; finish the remaining fields/flags, precise
  shell errors, non-EXE document edge cases, and broader command-template
  substitutions as traces demand them.
- Expand registry-backed shell behavior for associations, icons, shell folders,
  startup items, active devices, storage profiles, app paths, recent-doc UI/
  pruning above the current `SHAddToRecentDocs` shortcut/state path, and UI
  policy/settings.
- Implement shell-visible UI APIs that currently still succeed too cheaply:
  `SHGetFileInfo`, `TrackPopupMenu*`, modal `MessageBoxW`, notification icons
  and notification data, shell namespace/storage presentation, file type
  metadata, and shell/file-change notifications. `MessageBoxW` now validates
  owner HWNDs, records the requested modal data in `ShellSystem`, and returns
  the CE default button for the requested button group; the rendered modal
  window, nested pump, and real button/key interaction remain queued.
- Improve GWE defaults for focus, activation, non-client messages, hit testing,
  keyboard navigation, mouse capture, close/destroy, command routing, dialog and
  control notifications, and `SendNotifyMessageW` ordering.
- Expand input and UI state fidelity beyond the current minimal
  `TranslateMessage`: keyboard modifiers/layout, accelerators and menu
  shortcuts, `WM_SYSKEY*`, dead-key/IME handoff, clipboard
  rendered ownership/delayed-render edge cases above the now-stateful raw
  clipboard format store and `GetClipboardDataAlloc` local-handle copy path,
  rendered caret blink/focus invalidation above the now-stateful raw caret
  store, and focus/capture integration.
- Complete cross-thread `SendMessageTimeout` semantics: wait for reply or
  timeout, write the result pointer on completion, honor `SMTO_*` flags, handle
  receiver/sender destruction, and perform required reentrant dispatch while
  waiting.
- Replace direct-only WINSOCK with a CE-facing isolated NAT model: gateway
  `10.0.0.1`, guest IP `10.0.0.2`, host sockets as the transport boundary, and
  blocking waits owned by the scheduler.

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
- Extend `SHGetFileInfo` beyond the first registry-backed slice: real icon
  extraction/image-list handles, folder/executable special icon mapping, and
  mounted-trace edge cases. CE-supported flag validation now rejects
  unsupported/colliding `SHGFI_*` combinations instead of accepting
  desktop-only icon-location/masked-attribute requests.
- Implement real popup menu behavior for `TrackPopupMenu*`: menu popup or
  equivalent UI, modal menu loop, pointer/key hit-testing, selection/highlight,
  disabled/checked/default item rendering, submenu behavior, `WM_COMMAND`
  routing, `TPM_RETURNCMD`, and cancellation.
- Continue real modal `MessageBoxW`: the raw path now validates owners, records
  text/caption/style/result, temporarily applies owner disable/reenable, and
  derives the default `ID*` result from CE `winuser.h` button/default flags.
  Still create/show the modal dialog, render button layout/icons/text, run the
  nested message pump, and route Enter/Escape/button clicks to the final result.
- Finish shell special-folder behavior: log or classify fallback use, add a
  strict-vs-compat fallback policy, and confirm CSIDL coverage against
  observed calls. The raw path now rejects overlong resolved paths and honors
  both `fCreate` and `CSIDL_FLAG_CREATE` by creating missing mounted folders.
- Extend notification APIs: `Shell_NotifyIconW` now tracks add/modify/delete
  records generically and can post the registered owner callback message for a
  shell-icon event; `SHNotificationAddI`/`UpdateI`/`RemoveI`/`GetDataI` now
  preserve/query CE notification data by `CLSID + dwID` and can post a
  `WM_NOTIFY`/`NMSHN` callback payload to the stored sink HWND for notification
  events, including CE-style `SHNN_LINKSEL` `pszLink` strings stored after the
  `NMSHN` payload and `SHNN_DISMISS` `fTimeout` flags. Still implement
  automatic timeout/dismiss ownership, COM notification callbacks, richer icon
  lifetime, and optional shell/taskbar rendering only when backed by real shell
  state.
  Notification records are pruned during normal HWND/process teardown.
- Finish file namespace and file-change notifications: shell namespace roots
  over mounts, storage-volume presentation, file type/display/icon metadata,
  change notification handles, shell change notifications, create/delete/
  rename/update refresh events, and storage insertion/removal notifications.

## Completed User Todos

- Added `host_progs\cmd.cpp`: a basic desktop-Windows-like command prompt for
  Windows CE/eVC4 with `/C` and `/K` handling, a small GUI command window, and
  built-ins for `cd`, `dir`, `type`, `echo`, `copy`, `del`, `mkdir`, `rmdir`,
  `cls`, `pwd`, and `exit`, plus `CreateProcessW` fallback for executables.
  Verified it compiles and links with eVC4 MIPSII into
  `target\host_progs\cmd\cmd.exe`.
