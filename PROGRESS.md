# PROGRESS

## Artifact Note

- `target\` was cleared on 2026-06-04 to recover disk space after accumulated
  build directories and probe artifacts reached roughly 50 GB. Historical
  `target\...` paths below remain evidence labels only; regenerate fresh
  artifacts under a new prefix before relying on local files.

## Confirmed

- `SHGetSpecialFolderPath` now honors CE creation intent. The raw COREDLL path
  masks `CSIDL_FLAG_CREATE`, also respects the explicit `fCreate` parameter,
  creates missing mounted shell folders only when requested, and fails
  overlong resolved paths instead of truncating the fixed MAX_PATH output.
  Focused coverage:
  `coredll_raw_shget_special_folder_path_honors_create_flags`.
- Shell file-change notification ordinals now have stateful raw coverage.
  `SHChangeNotifyRegisterI` accepts both the direct `(HWND,
  SHCHANGENOTIFYENTRY*)` shape and the one-pointer PSL-style packed shape,
  validates sink HWNDs, records event mask/watch directory/recursive state in
  `ShellSystem`, and cleans registrations up when the HWND is destroyed.
  `SHFileNotifyRemoveI` deregisters by HWND and `SHFileNotifyFreeI` records
  freed notification payload pointers without fabricating event delivery.
  Focused coverage:
  `sh_change_notify_i_tracks_register_remove_and_free_state`.
- `SHAddToRecentDocs` now has a CE-backed raw path for `SHARD_PATH` and null
  clear calls. It resolves `CSIDL_RECENT` through
  `HKLM\System\Explorer\Shell Folders`, creates the Recent folder, writes CE
  text `.lnk` shortcuts with file-name stems that preserve spaces, records
  recent-document state in `ShellSystem`, and clears both the mounted recent
  files and shell state when the caller passes a null item. Focused coverage:
  `shell_add_to_recent_docs_creates_and_clears_recent_shortcuts`.
- Shell shortcut raw APIs now cover the basic CE text-link path used by
  `ShellExecuteEx`. `SHCreateShortcut` creates a mounted `.lnk` using CE's
  UTF-8 BOM plus `count#"<target>" args` format with `CREATE_NEW` semantics,
  while `SHGetShortcutTarget` reads the same file format, validates caller
  buffers, and returns the stored target text. The focused raw shell fixture
  proves create, readback, launch through `ShellExecuteEx`, existing-file
  failure, insufficient-buffer failure, `SEE_MASK_NOCLOSEPROCESS` hProcess
  output, and `SHELLEXECUTEINFO.nShow` propagation into pending launch
  metadata. Child MIPS startup now passes the current process show command as
  `WinMain`'s `nCmdShow` argument instead of forcing `SW_SHOWNORMAL` for shell
  launches. `ShellExecuteEx` and raw `CreateProcessW` also preserve explicit
  current-directory fields in pending process launches; relative child EXE
  resolution now checks that CE current directory before the existing parent
  executable fallback, and parked process switching restores the effective
  current directory with module path and command line. Registry association
  command templates now handle the CE `"%1" %*` shape from `shell.reg` by
  replacing `%*` with `lpParameters`, and only append `lpParameters` when the
  template lacks an explicit `%*` placeholder. `SHCreateShortcutEx` now
  shares the CE text-link writer, chooses a
  bounded unique sibling name with ` (2)` style suffixes when the requested
  link already exists, writes the actual name to the optional output buffer
  when it fits, and reports the required character count without creating a
  file when the caller buffer is too small.
- Raw clipboard fidelity now includes `GetClipboardDataAlloc` for clipboard
  data stored as emulator-tracked local heap handles. The COREDLL raw path
  looks up the format through GWE clipboard state, derives the source size from
  `LocalSize`, allocates a fresh local handle, copies the guest bytes, and
  rejects unknown source handles with `ERROR_INVALID_HANDLE` rather than
  guessing a buffer length. Focused raw clipboard fixtures cover successful
  byte-copy independence and the unknown-handle failure path.
- Runtime guest-DLL normal loads now enter guest lifecycle code before
  returning from `LoadLibraryW` / `LoadLibraryExW(flags=0)`. The PE parser reads
  the TLS directory callback table into loaded-module metadata, and the Unicorn
  import-trap path queues dependency-order lifecycle callouts through a
  reserved return stub. The implemented lifecycle slice invokes guest TLS
  callbacks and then `DllMain(hinst, DLL_PROCESS_ATTACH, 0)` for newly mapped
  normal-code DLLs, preserves the importing thread registers around the
  callout, and returns the module handle only after attach completes. Focused
  eVC fixtures now assert
  exactly one attach for `171_loadlibrary_guest_dll` across handle reuse and
  exactly one attach for both dependency and dependent DLLs in
  `172_loadlibrary_dependent_guest_dll`. The new
  `173_loadlibrary_tls_callback` fixture uses an eVC-built MIPS DLL with a
  real PE TLS directory (`TLSTableRVA=0x2000`, size `0x18`) and proves the TLS
  callback executes before `DllMain` for attach and final detach.
  Focused PE/TLS parser coverage and the full
  `cargo test --features unicorn,trace,win32-desktop` suite pass. Final dynamic
  `FreeLibrary` now uses the same runtime-loader interception path to enter
  guest TLS callbacks and then `DllMain(hinst, DLL_PROCESS_DETACH, 0)` before
  marking the module unload-pending and preserving the mapped range. The
  updated `171_loadlibrary_guest_dll` fixture arms an EXE-owned detach marker,
  proves a non-final `FreeLibrary` only decrements the refcount, and proves
  final release runs one guest detach. PE forwarded-export metadata is now
  retained and resolved in the runtime loader: `GetProcAddress` and guest-DLL
  IAT patching follow forwarder strings through already-loaded modules or CE
  search/load of the target guest DLL, and COREDLL forwarders return dynamic
  trap addresses. The new `174_loadlibrary_forwarded_export` fixture proves a
  forwarded export by name, ordinal lookup on the forwarder export, and import
  patching through a forwarder DLL. Runtime
  `LoadLibraryExW(DONT_RESOLVE_DLL_REFERENCES)` now maps and registers ordinary
  exports without recursive dependency loading, import patching, TLS callbacks,
  `DllMain`, or final detach callouts; the new
  `175_loadlibrary_noresolve` fixture proves a staged imported dependency is
  not loaded in no-resolve mode, the target DLL attach count remains zero, and
  a later normal `LoadLibraryW` resolves imports and runs attach. Runtime
  `LoadLibraryExW(LOAD_LIBRARY_AS_DATAFILE)` now follows CE's datafile-implies-
  no-resolve rule, maps the image for resource access, registers strings and
  raw resource entries immediately with `kernel.resources`, and hides code
  exports from `GetProcAddress`; the new
  `176_loadlibrary_datafile_resource` fixture proves `LoadStringW`,
  `FindResourceW`, `SizeofResource`, and `LoadResource` work from the loaded
  resource module without dependency loading or `DllMain`, then a later normal
  load resolves imports and runs attach. The raw/non-Unicorn
  `LoadLibraryExW` helper now accepts supported no-resolve/datafile flags when
  reusing COREDLL or an already registered module, increments the reused module
  refcount, reports `ERROR_FILE_NOT_FOUND` for missing modules, and still
  returns `ERROR_NOT_SUPPORTED` for unsupported flags instead of pretending to
  map bytes without a Unicorn memory context. Runtime normal/no-resolve DLL
  loads now also run the same MIPS Unicorn trampoline patcher used by startup
  PE mapping before the DLL image is mapped, place generated stubs inline in
  the mapped DLL image, and publish new trampoline ranges/jumps into both the
  persisted emulator metadata and the live full-code-hook state; datafile loads
  stay non-executable and skip trampoline patching. Runtime loader audit
  counters now live on `CeKernel` and appear in `UnicornDebugSnapshot`
  summaries, full display text, and monitor `loader` output: load attempts,
  successful maps, dependency loads, export lookups/misses, forwarded exports,
  TLS callbacks, `DllMain` attach/detach calls, and loud loader failures are
  compactly visible during mounted runs. The updated
  `173_loadlibrary_tls_callback` fixture arms an EXE-owned detach order marker
  and proves the complete lifecycle order word `0x01020304`: TLS attach,
  `DllMain` attach, TLS detach, `DllMain` detach.
- eVC fixture infrastructure now supports fixture-local runtime DLLs under
  `tests\test_progs\<fixture>\dlls\<dll-name>\`. The runner discovers `.cpp`,
  `.rc`, and optional `.def` files, links each DLL with an import library,
  stages the EXE/DLL artifacts under the per-fixture `\SDMMC Disk` mount, and
  treats DLL outputs as rebuild dependencies. The new
  `171_loadlibrary_guest_dll` fixture source exercises `LoadLibraryW`,
  same-module reuse, named and ordinal `GetProcAddress`, guest export calls,
  and paired `FreeLibrary` calls against a generated MIPS DLL. The fixture now
  asserts one `DllMain(DLL_PROCESS_ATTACH)` call across handle reuse and one
  `DllMain(DLL_PROCESS_DETACH)` call on final release.
  `cargo check --features
  unicorn,evc4-fixtures --test fixture_exes` passes, and the focused eVC run
  `WINCE_FIXTURE_FILTER=171_loadlibrary_guest_dll cargo test --features
  unicorn,evc4-fixtures --test fixture_exes -- --ignored --nocapture` passes.
- The runtime guest-DLL fixture suite now includes
  `172_loadlibrary_dependent_guest_dll`. The harness links later fixture DLLs
  against import libraries produced by earlier fixture DLLs, allowing real PE
  imports between sibling DLLs. The fixture builds `dependency_user.dll` with a
  normal import from `dependency_base.dll`, then the EXE loads only
  `dependency_user.dll`, resolves `DependentUserExport`, and calls through the
  patched guest IAT into the base DLL. It now also asserts one attach for the
  recursively loaded base DLL and one attach for the user DLL. The focused eVC run
  `WINCE_FIXTURE_FILTER=172_loadlibrary_dependent_guest_dll cargo test
  --features unicorn,evc4-fixtures --test fixture_exes -- --ignored
  --nocapture` passes, proving recursive runtime dependency loading and
  guest-DLL-to-guest-DLL import resolution plus attach callouts for eVC-built
  MIPS DLLs.
- The latest attachment audit has been merged into `PLAN.MD`/`TODO.md` as
  explicit queued fidelity work: raw/non-Unicorn loader alignment, core-vs-guest
  DLL boundary diagnostics, actionable must-implement stub hits, richer
  `SHGetFileInfo`, popup menus, modal `MessageBoxW`, shell special-folder
  fallback policy, notification APIs, shell namespace/storage presentation,
  file-change notifications, input/IME/caret/clipboard, and full
  `SendMessageTimeout` semantics.
- `SHGetFileInfo` now has a registry-backed raw COREDLL implementation instead
  of a must-implement stub fallback. The first slice handles Unicode paths,
  display names, type names from HKCR class descriptions, real mounted-file
  attributes, stable system icon indexes, and generic icon handles. The raw
  boundary now mirrors CE's supported flag subset: unsupported or colliding
  `SHGFI_ICONLOCATION`, `SHGFI_ATTR_SPECIFIED`, `SHGFI_PIDL`, `SHGFI_SMALLICON`
  without icon/index, and `SHGFI_ATTRIBUTES | SHGFI_USEFILEATTRIBUTES` calls
  fail with `ERROR_INVALID_FLAGS` instead of returning plausible desktop-style
  metadata. Focused raw shell tests
  `sh_get_file_info_uses_registry_associations_and_attributes` and
  `sh_get_file_info_rejects_unsupported_and_colliding_flags` pass; real icon
  extraction/image-list ownership remains queued.
- `TrackPopupMenuEx` now mirrors the CE menu-loop notification boundary above
  the existing popup tracking state. For valid owner HWNDs it sends and records
  `WM_ENTERMENULOOP`, `WM_INITMENUPOPUP`, and `WM_EXITMENULOOP` unless
  `TPM_NONOTIFY` is set, while `TPM_RETURNCMD` still returns no command rather
  than fabricating a user selection. Focused coverage:
  `coredll_raw_track_popup_menu_records_attempt_without_fake_selection`.
  Rendered popup windows, hit-testing, keyboard navigation, and real command
  selection remain queued.
- Raw `TranslateAcceleratorW` now honors CE accelerator modifier bits instead
  of matching only the virtual key. It accepts `WM_KEYDOWN` and
  `WM_SYSKEYDOWN`, checks `FVIRTKEY`, `FSHIFT`, `FCONTROL`, and `FALT` against
  GWE key state, treats `WM_SYSKEYDOWN` as the Alt path, and preserves the
  existing `WM_COMMAND` queue behavior only for a matching entry. Focused
  coverage:
  `coredll_raw_translate_accelerator_honors_modifiers_and_syskey`.
- Raw `TranslateMessage` now uses GWE key state for the basic CE keyboard
  translation path instead of echoing virtual keys. It translates ASCII
  letters with Shift/CapsLock XOR casing, maps the shifted digit row to
  punctuation, preserves the original lParam, posts `WM_CHAR` for `WM_KEYDOWN`,
  and posts `WM_SYSCHAR` for `WM_SYSKEYDOWN`. Focused coverage:
  `coredll_raw_translate_message_uses_shift_caps_and_syschar`.
- Raw `SendNotifyMessageW(HWND_BROADCAST, ...)` now follows the same CE
  no-wait notification-send model as single-HWND notify sends instead of
  falling back to posted broadcast messages. It filters to live application
  top-level windows, skips the synthetic desktop, child windows, and destroyed
  targets, runs same-thread targets synchronously, and queues different-thread
  targets as `SMF_SENDER_NO_WAIT | SMF_NOTIFY_MESSAGE` sent transactions.
  Focused coverage:
  `coredll_raw_send_notify_broadcast_uses_notify_send_for_live_top_level_windows`.
- Raw and Unicorn-default `DefWindowProcW` now cover the first CE non-client
  default slice. `WM_NCHITTEST` decodes screen coordinates from lParam and
  returns CE `HT*` codes for client, border bands, and outside-window points,
  while `WM_SYSCOMMAND/SC_CLOSE` routes through the same default `WM_CLOSE`
  destroy path rather than returning a no-op. Focused coverage:
  `coredll_raw_def_window_proc_handles_hit_test_and_syscommand_close` and
  `default_window_hit_test_reports_client_border_and_nowhere`.
- Shell notification state now has a dedicated `ShellSystem` instead of a
  COREDLL-local placeholder. Raw `Shell_NotifyIconW @481` decodes the
  `NOTIFYICONDATAW` prefix, validates owner HWNDs, tracks add/modify/delete
  records keyed by `(HWND,uID)`, preserves callback message/icon/tip/state,
  posts the registered callback message to the owner HWND with `wParam=uID` and
  `lParam` carrying the shell event, and reports CE-shaped
  `ERROR_INVALID_PARAMETER`, `ERROR_INVALID_WINDOW_HANDLE`, or
  `ERROR_INVALID_HANDLE` failures. Focused coverage
  `shell_notify_icon_tracks_add_modify_delete_and_posts_callback` passes. Rich
  `SHNotification*` interaction callbacks, timeout/dismiss behavior, and
  taskbar rendering remain queued.
- Raw `MessageBoxW` no longer returns a fixed success value. Using CE
  `PUBLIC\COMMON\SDK\INC\winuser.h` constants as the source reference, it now
  validates non-null owner HWNDs, rejects invalid string pointers, records
  owner/text/caption/style/result in `ShellSystem`, briefly applies owner
  disable/reenable bookkeeping, and returns the CE default `ID*` for the
  requested button group/default-button flags. Focused coverage
  `message_box_w_records_text_owner_and_returns_default_button` and
  `message_box_w_rejects_destroyed_owner_without_recording` passes. The real
  rendered modal dialog, nested message pump, and button/key interaction remain
  queued.
- Raw clipboard COREDLL ordinals now have GWE-owned state instead of broad
  fallback behavior. Using CE `PUBLIC\COMMON\SDK\INC\winuser.h` clipboard
  constants/prototypes as the source reference, `OpenClipboard`,
  `CloseClipboard`, `EmptyClipboard`, `SetClipboardData`, `GetClipboardData`,
  `GetClipboardOwner`, `GetOpenClipboardWindow`,
  `IsClipboardFormatAvailable`, `CountClipboardFormats`,
  `EnumClipboardFormats`, `GetPriorityClipboardFormat`,
  `RegisterClipboardFormatW`, and `GetClipboardFormatNameW` track the open
  lock, owner, format handles, standard/custom formats, enumeration, duplicate
  custom format registration, and null-owner opens. Focused coverage
  `clipboard_raw_ordinals_track_lock_owner_formats_and_names` and
  `clipboard_raw_ordinals_reject_invalid_open_and_missing_lock` passes. Richer
  allocation/copy ownership behavior such as `GetClipboardDataAlloc` remains
  queued.
- Raw caret COREDLL ordinals now have GWE-owned state instead of broad fallback
  behavior. Using CE `PUBLIC\COMMON\SDK\INC\winuser.h` caret prototypes as the
  source reference, `CreateCaret`, `DestroyCaret`, `ShowCaret`, `HideCaret`,
  `SetCaretPos`, `GetCaretPos`, `SetCaretBlinkTime`, `GetCaretBlinkTime`, and
  the CE ordinal-only system-wide enable/disable calls track owner HWND,
  bitmap, size, signed position, show count, blink time, system enable state,
  and cleanup when the owner window is destroyed. Focused coverage
  `coredll_raw_caret_ordinals_track_position_visibility_and_blink` and
  `coredll_raw_caret_rejects_invalid_state_and_destroys_with_owner` passes.
  Rendered caret blinking and focus/update invalidation remain queued.
- Runtime guest-DLL loading now has a shared CE-aware search helper in
  `emulator::dll_search`, replacing duplicate private startup/child-process
  search logic. It resolves exact mounted CE paths first, then the active
  process directory, configured DLL search dirs, and mounted `\Windows`, with
  case variants and optional `.dll` suffixes. Startup preload, `commctrl.dll`
  preload, and child-process import preload all use this helper, so the future
  Unicorn-owned `LoadLibraryExW` mapper can share the same search order instead
  of inventing a third path. Focused `dll_search` tests pass.
- The live Unicorn import-trap hook now owns a mutable shared
  `ImportTrapTable` and persists it back to `self.import_traps` before debug
  capture. Existing trap dispatch still behaves the same, but the runtime DLL
  mapper can now add traps for newly mapped guest DLL imports and have both live
  dispatch and later diagnostics see them. Focused import-trap coverage passes.
- Runtime guest-DLL import patching now has the next trap-table groundwork:
  `ImportTrapTable::next_static_trap_base` finds the next static import-trap
  slot while preserving the dynamic `GetProcAddress` trap range, and Unicorn
  image setup now refreshes the persisted `ce-import-traps` mapped blob through
  a shared helper instead of pushing a one-shot page. This does not yet rewrite
  the live Unicorn trap page after a runtime load, but it gives the mapper the
  allocation and persisted-state hooks it needs. Focused
  `next_static_trap_base_*` and
  `refresh_import_trap_page_blob_replaces_existing_blob` coverage passes.
- Runtime normal-code guest DLL loading has its first Unicorn import-trap path.
  `LoadLibraryW` and `LoadLibraryExW(flags=0)` now search through the shared
  CE-aware DLL search order, parse dumped MIPS PE DLLs, recursively load
  non-emulator dependencies, relocate/map the image into guest memory, patch
  COREDLL/WINSOCK/OLE imports to trap slots, patch external guest-DLL imports
  against the kernel's loaded-module export snapshots, rewrite the live import
  trap page, refresh the persisted trap blob, register PE resources/exports,
  and record the module as dynamic with CE-style refcounts. The runtime
  `DONT_RESOLVE_DLL_REFERENCES` path maps/reuses modules and exposes ordinary
  exports while deliberately skipping dependency loads, import patching,
  forwarder resolution, TLS callbacks, and `DllMain`; datafile/resource-style
  loads now expose resource APIs without code exports or lifecycle calls. Guest
  `DllMain(DLL_PROCESS_DETACH)` on final dynamic `FreeLibrary` plus TLS detach
  ordering are now covered by direct/TLS runtime fixtures; forwarded exports
  are covered by runtime `GetProcAddress` and import-patching fixtures, and
  no-resolve mode is covered by fixture 175, and datafile/resource mode is
  covered by fixture 176. Fuller runtime trampoline handling remains open.
  Focused coverage passes
  for loaded-module export snapshots, runtime occupied-range calculation, and
  the existing raw `LoadLibraryExW` flag/refcount behavior.
- `ExternalImportTable` now has a public `add_module_exports` path for
  already-loaded guest DLL metadata, not only `PeImage` startup inputs. This is
  the import-resolution surface the runtime loader will use after mapping a
  DLL or when resolving a dynamically loaded DLL against previously loaded
  guest modules. Focused runtime-module export lookup coverage passes.
- COREDLL fallback audit now carries real raw-dispatch context from the import
  trap boundary. `CoredllRawContext` records thread id, caller PC, and trap PC;
  the Unicorn import hook fills it from the active CE thread and guest RA before
  dispatching COREDLL ordinals, and fallback logs include both caller/trap PCs.
  `SHGetFileInfo` is now explicitly classified as `must implement` instead of
  slipping through the broad kernel-private safe-failure bucket. Focused
  coverage `raw_stub_audit_keeps_import_trap_context` passes. The same context
  now also includes caller-module attribution when the guest RA falls inside a
  mapped process image or runtime DLL blob, giving must-implement stub hits the
  responsible `image:`/`dll:` owner in fallback logs. Focused coverage
  `mapped_blob_module_for_pc_attributes_image_and_runtime_dlls` verifies that
  ordinary RAM blobs are not mislabeled as module callers. Must-implement raw
  fallbacks now also record `ERROR_NOT_SUPPORTED`, set that last-error value on
  the caller thread when raw dispatch has context, and force explicit
  failure-shaped return values for critical loader/process/shell/UI APIs such
  as `LoadLibraryW` if they ever miss the real implementation path.
- CE fidelity catch-up now has a durable implementation ledger in `PLAN.MD`;
  its first line is `Reference C:\WINCE600 sources` as requested. The first
  structural slice is implemented without app-specific behavior: loaded guest
  module metadata now tracks guest/host path, image size, entrypoint,
  dependencies, TLS callback slots, refcount, load flags, dynamic/pinned state,
  and unload-pending state. Startup-preloaded DLLs are registered through this
  richer shape, `LoadLibraryW` increments refcounts for already loaded modules,
  `LoadLibraryExW` rejects unsupported/datafile runtime flags explicitly
  instead of ignoring them, runtime no-resolve loads now map modules without
  dependency/import/lifecycle resolution, and `FreeLibrary` decrements dynamic
  modules while keeping mapped ranges reserved. Raw `ShellExecuteEx` now reads
  `SHELLEXECUTEINFO`, resolves CE `.lnk` text shortcuts and registry
  association command templates, queues the existing `CreateProcessW` launch
  path, fills `hInstApp`, and returns `hProcess` for
  `SEE_MASK_NOCLOSEPROCESS`. COREDLL fallback stubs now carry an audit
  classification (`safe no-op`, `safe failure`, or `must implement`) and log
  must-implement fallbacks. WINSOCK now exposes an isolated CE-facing NAT model
  with guest IP `10.0.0.2` and gateway `10.0.0.1`; local name resolution uses
  those CE addresses while host sockets remain the transport boundary. Focused
  tests pass for raw module refcounts/`LoadLibraryExW` failure, registry-backed
  `ShellExecuteEx`, stub audit classification, and isolated Winsock local name
  resolution. Full `cargo check --features unicorn,trace,win32-desktop` passes
  with only the existing unused IOCTL warning.
- WINSOCK import traps now route through the top-level `src/winsock.rs`
  subsystem boundary instead of living inline in `src/emulator/imports.rs`.
  The first host-backed slice is implemented there: `socket`, `connect`,
  `bind`, `listen`, `accept`, `send`, `recv`, `sendto`, `recvfrom`,
  `closesocket`, `ioctlsocket(FIONBIO)`, `shutdown`, `select`/`__WSAFDIsSet`,
  byte-order helpers, `inet_addr`, `inet_ntoa`, `gethostname`,
  `gethostbyname`, `WSAGetLastError`, and `WSASetLastError` now dispatch
  through a WINSOCK-owned socket table and host TCP/UDP sockets. Focused tests
  cover `WSAStartup`, byte-order/address conversion, and real loopback
  `connect`/`send`/`recv`. This is direct host networking, not the future
  isolated CE subnet/gateway model.
- Windows-sudo `cargo flamegraph` profiling of the mounted Win32-host route
  startup path now identifies and closes another generic startup-speed tax.
  Pre-fix `target\route_search_host_90s_20260607_debugforced2_flame.svg`
  spent about 27.96% of samples in
  `map_persisted_ram_blob_pages` / `uc_mem_map` and about 31.91% in
  `uc_close` / `memory_free_mipsel`, while the live summary still showed
  bounded file I/O (`host_read=29778/2490133B`). Persisted heap-spillover RAM
  blobs are now remapped into each fresh Unicorn instance as contiguous
  unmapped spans instead of one 4 KiB map per page. Post-fix
  `target\route_search_host_90s_postspan_flame.svg` removes the persisted-RAM
  remap/teardown blocks from the top profile; the same bounded host profile
  advances farther (`host_read=78968/4003851B`, PC in `mfcce400.dll`) and shows
  actual `uc_emu_start`/TCG execution as the visible cost. Focused coverage
  `persisted_ram_blob_remap_groups_unmapped_spans` passes, and the full
  `cargo test --features unicorn,trace,win32-desktop` suite is green after
  aligning two stale visibility/input fixtures with the current CE-ish
  hit-test contract.
- Fresh route-search visibility probes `target\route_showcmd1_*` and
  `target\route_showcmd2_*` show that the post-IPC `afxwnd42u` route/chrome
  children are not being hidden by an untraced z-order side effect. They are
  created visible and then the guest explicitly calls raw
  `ShowWindow(cmd=0/SW_HIDE)` for each child; no matching `cmd=5`/show appears
  before the bounded stop. The only early positive show in this slice is the
  owner/popup class `Afx:10000:3:0:b5000:0` with `cmd=5`. The framebuffer
  remains the partial header/map, `gwe=send:1 done:1`, and render milestones
  are still `none`, so this remains a generic startup/resource/show-sequencing
  frontier rather than stale hidden-layer leakage or a stranded sent message.
- Route-search monitor handoff now gets through the `happyway_win.exe`
  cross-process `SendMessageW` frontier instead of stranding the sender. CE
  sent messages are dispatched internally during `GetMessageW` rather than
  returned to the guest as ordinary `MSG`s, and parked sender CPU snapshots now
  complete ready scheduler-backed send waits when execution rotates back to
  the parent. Fresh evidence `target\route_after_sendfix1_*` records
  `dispatch_sent_message` for HWND `0x0002000c`, guest WNDPROC return from
  `GetMessageW/SentMessage`, `gwe=send:1 done:1`, parent reactivation, and
  later `iSearch.exe` activation plus many `afxwnd42u` child-window creations.
  Focused coverage `rotate_to_parked_process_skips_send_blocked_process_until_ready`
  and the send-message test set pass. The route path is not complete yet: the
  next frontier is the post-IPC route chrome/dialog sequence, not the old
  stuck synchronous send.
- Post-IPC route probes now show the next concrete blocker. Fresh captures
  `target\route_postipc3_*` and `target\route_pos1_*` still render only the
  header/map framebuffer, but traces prove the app advances past helper IPC:
  `iSearch.exe` is created/activated, and iNavi creates `afxwnd42u` child
  windows for IDs `0x2710` through `0x2743` under parent `0x00020004`. The
  final window snapshot has those children present but hidden (`WS_VISIBLE`
  cleared; e.g. `0x00020014` style `0x44000000`, rect `3,102-103,237`) and no
  iNavi display/render milestones. New diagnostic trace records now capture
  `SetWindowLongW(GWL_STYLE)` and visibility-affecting `SetWindowPos` calls so
  the next route probe can identify the exact guest/API transition that hides
  these controls. This is trace-only; it does not force child visibility.
- Route-search retry on the fresh host/remote path confirmed two separate
  issues. In the live remote run, tapping the red search button on the settled
  map created owned top-level `TGNaviDlg` HWND `0x00020084`, delivered
  `WM_PAINT`, set `WM_TIMER id=0x19fe` for 1500 ms, then destroyed the dialog
  through the guest `WM_TIMER` path; the 120 ms capture
  `target\route_search_resume_120ms_*` had already recorded `timer_due`, so
  v3 was still compressing live UI time in some wait paths. Live
  `GetMessageW` and `MsgWaitForMultipleObjectsEx` short-timer fast-forwarding
  are now guarded by `live_pump`, and current-thread finite `Sleep` /
  `WaitForSingleObject` inline completion now sleeps real wall time in live
  mode instead of throttling to 1 ms while advancing the full CE timeout.
  Focused coverage `live_pump_getmessage_does_not_fast_forward_short_ui_timer`,
  `live_pump_msg_wait_does_not_fast_forward_short_ui_timer`, and the current
  wait/sleep scheduler tests pass. The route UI is not fixed yet: scripted
  monitor probes `target\route_mon_live1_*` and
  `target\route_mon_progress2_*` show taps before the right chrome/bottom strip
  are still too early, and monitor mode still needs better parity with the
  normal live run-loop to drive the parked child/process handoff.
- Remote input for `hwnd=any` blocked helper threads now routes mouse messages
  through the visible desktop hit-test target's owning thread instead of
  posting a foreign HWND to the active helper's queue. Fresh route evidence
  `target\route_search_tail4_messages.txt` showed the bad shape:
  `happyway_win.exe` thread 3 was blocked in `GetMessageW`, but a tap over
  visible parent HWND `0x00020004` was posted as
  `thread_id=3, hwnd=0x00020004`. `drain_remote_input_to_thread_window` now
  uses desktop hit-testing when the blocked waiter has `hwnd=any`, while
  explicit HWND drains keep the existing blocked-thread behavior. Focused
  coverage `remote_input_any_blocked_thread_uses_desktop_hit_test_owner`,
  `remote_input_blocked_thread_target_overrides_stale_active_window`,
  `remote_input_active_window_uses_desktop_hit_test_over_hidden_active_window`,
  and `saved_remote_input_target_uses_saved_get_message_waiter` passes. The
  follow-up controlled probe `target\route_search_ownerfix1_*` did not reach
  visible bottom chrome within the wait window, so it did not re-enter the
  `TGNaviDlg` timer/destroy frontier.
- Current route-search host run now routes REST/host taps to the saved main
  `GetMessageW` waiter when the active CPU context is a hidden helper process.
  The previous tap path could post input while `happyway_win.exe` was current;
  `target\route_savedwait1_bottom_tap_remote_input.txt` now shows
  `route=thread thread_id=1 hwnd=any`, and focused coverage
  `saved_remote_input_target_uses_saved_get_message_waiter`,
  `remote_input_active_window_uses_desktop_hit_test_over_hidden_active_window`,
  and `window_from_point_skips_hidden_top_level_above_visible_window` passes.
  Host pointer input also uses a desktop-wide visible hit-test before falling
  back to the active window, so a hidden top-level helper above the map no
  longer steals taps. The route path is not finished: after tapping the bottom
  current-location strip, iNavi creates top-level owned `TGNaviDlg`
  `0x00020084`, but it is already `dead=true` and removed from z-order in
  `target\route_wndproc1_bottom_tap_windows.txt`; even an 80 ms capture
  (`target\route_wndproc1_bottom_retap_80ms.png`) never shows the route dialog.
  The current frontier is dialog/window lifetime or guest early-close behavior,
  not remote input delivery.
- Current route-search retry returned to the real startup/chrome blocker, not
  to a touch-coordinate problem. The in-shell Win32-host drive harness
  `target\route_path_drive1_montage.png` kept the remote endpoint alive and
  accepted taps for safety/bottom/current-location/route-search coordinates,
  but every frame stayed on the partial header/map composition with no
  right-side chrome or bottom current-location strip. Rich virtual evidence
  `target\route_chrome_rich1_*` shows the chrome child HWNDs already exist
  under parent `0x00020004`, but are still hidden (`0x00020020`..`0x00020030`
  for right-side buttons, `0x00020070` for the bottom strip) while iNavi is
  still loading `resmapi_800x480.bin` RSImage/PNG resources through repeated
  `CreateDIBSection` calls. Route driving must wait for, or speed up/fix, this
  generic startup/chrome path; tapping before those children are visible is not
  meaningful route UI progress.
- Live virtual/remote runs now rotate parked child processes on live-pump wall
  stops, matching the host-presenter scheduling policy instead of making CE
  process scheduling depend on the presenter. Pre-fix
  `target\route_chrome_block_v1_processes.txt` activated
  `happyway_win.exe` and `iSearch.exe` once, then returned to `iNavi.exe` only.
  Post-fix `target\route_chrome_rotate_v1_processes.txt` shows repeated
  `iNavi.exe -> happyway_win.exe -> iSearch.exe` activation cycles across the
  120 s virtual/remote probe. Focused coverage
  `live_wall_stop_rotates_parked_processes` and
  `host_no_wall_run_uses_implicit_live_slice` passes, and
  `cargo check --features unicorn,trace,win32-desktop` passes with only the
  known unused IOCTL/incremental-cleanup warnings. This does not yet reveal the
  route chrome; the active bottleneck has moved to the resource/GDI throughput
  and hidden-child show/update sequencing after the process handoff.
- Startup speed regression from Unicorn heap-spillover mapping is closed for
  the current mounted virtual profile. Windows-sudo flame
  `target\startup_debugsym2_flame.svg` showed
  `map_kernel_memory_allocations` / `map_heap_spillover` consuming 33.91% of
  samples because `map_guest_range` called `uc.mem_map` once per 4 KiB
  spillover page. Heap spillover now maps contiguous unmapped spans while
  virtual allocations remain page-granular for stale-page reclaim semantics.
  Focused coverage `unmapped_guest_spans_group_contiguous_pages` passes.
  Final release bench
  `target\grouped_map_final_60s_summary.txt` completed the bounded mounted
  virtual startup slice in 16.85 s wall time with bounded file I/O
  (`host_open=691`, `host_read=79199/4284623B`, `max_read=685080`) and a real
  header/map framebuffer in `target\grouped_map_final_60s.png`. Fresh
  Windows-sudo flame `target\grouped_map_final_flame.svg` drops
  `map_kernel_memory_allocations` to 1.16% and `map_heap_spillover` to 0.17%;
  the debuginfo flame run reaches the safety screen with later scheduler state
  and companion activation evidence (`happyway_win.exe` and `iSearch.exe`
  parked/activated). Remaining startup work is now CE scheduling/display/app
  flow, not multi-GB file preload, host-file reopen, or per-page Unicorn map
  churn.
- Fresh Windows-sudo debuginfo flame
  `target\startup_flame_virtual_250m_20260607_sudo_debugsym.svg` gives the
  current 250M mounted virtual baseline after the heap-spillover fix. A plain
  release run of the same slice completes in 17.783 s and stops at
  `image:iNavi.exe+0x329da8` with bounded file I/O (`host_open=445`,
  `host_read=78849/3286831B`, `max_read=497178`) and ~24.6 MB live heap.
  The remaining profile is dominated by Unicorn state churn rather than file
  I/O: `map_persisted_ram_blob_pages` / `uc_mem_map` is ~13.5% and Unicorn
  teardown (`uc_close` / `memory_free_mipsel`) is ~15.8%, while
  `map_kernel_memory_allocations` is ~1.7% and code-hook execution is ~1.8%.
  A first remote flame exposed a separate live-slice bug:
  `target\startup_flame_remote_250m_20260607_sudo.svg` stopped after a 52 ms
  live slice and faulted at `pc=0x0000353c` before any host-file reads.
- Remote live-pump runs no longer depend on tiny 50 ms CPU wall slices to
  prevent host-side wait sleeps. `UnicornRunLimits` now carries an explicit
  `live_pump` flag, live host/remote runs refuse wait-helper `thread::sleep`
  inside Unicorn hooks, and the remote service slice is 1000 ms. The direct
  mounted virtual remote probe
  `target\remote_liveslice_fix1_summary.txt` with
  `--remote-server 192.168.0.39:8765` and a 30 s wall budget exits cleanly
  instead of `FETCH_UNMAPPED`, reaching `image:iNavi.exe+0xb3b9b0` with real
  file activity (`host_open=189`, `host_read=25997/2429816B`) and framebuffer
  evidence in `target\remote_liveslice_fix1.png`.
- After the remote live-pump fix, a bounded remote-drive attempt
  `target\remote_route_after_liveslice1_montage.png` kept the REST endpoint
  live (`/api/v1/status` reported `running=true`, 800x480, `gpsTarget=COM21`)
  and accepted tap posts, but every captured frame stayed at the same partial
  header/map composition with no right-side chrome, bottom current-location
  strip, safety OK button, or route-search controls. That confirms this probe
  is blocked by the known startup/chrome scheduler/process frontier, not by
  dead remote input or the previous remote `FETCH_UNMAPPED`.
- Current stuck-screen investigation narrowed the partial startup to the CE
  multi-process handoff, not to presenter blackness or map file I/O. Fresh
  mounted virtual traces `target\stuck_process_*` show
  `happyway_win.exe` is created and parked as process 67/thread 3, then the
  parent queues `SendMessageW(hwnd=0x0002000c,msg=0x0401,wParam=14)` to that
  helper window while thread-1 chrome `WM_SHOWWINDOW` /
  `WM_WINDOWPOSCHANGED` messages accumulate. A naive experiment that yielded
  immediately after a parked child was created did rotate into child code, but
  longer bounded runs became non-returning around hot `HeapAlloc @46`; that
  experiment was backed out. The safe conclusion is that v3 needs a real
  scheduler-owned child-process run queue/cross-process SendMessage handoff,
  not a host-loop shortcut.
- Current mounted host startup frontier is a real, bounded CPU/resource path,
  not a dead process or file-I/O/RSS regression. Fresh Win32-host validation
  with dumped DLLs from `D:\INAVI_Emulator\DUMPPLZ\Windows` and
  `--remote-server 192.168.0.39:8765` is live as PID `33428` during this
  session. Remote `/api/v1/status` reports `running=true`, the host window
  responds, and memory stays bounded around 260-306 MB. The visible frame in
  `target\host_after_importgate_30s.png` and
  `target\host_after_importgate_200s.png` is a real iNavi header plus map
  composition, but the right-side and bottom chrome are still missing after
  several minutes. Progress samples stay in app/resource work with hot
  `COREDLL.dll!HeapAlloc @46` / `malloc @1041`, not in map DB file reads.
- Two startup-speed experiments are now characterized. Heap spillover pages are
  persisted as restorable RAM blobs, which removes the older post-route
  unmapped heap fault shape without reintroducing large file preloads. The
  qsort callout now uses a partitioning quicksort state machine instead of the
  previous bubble-style comparator loop, but that did not materially move the
  partial-map startup frontier. `WINCE_EMU_FAST_START` no longer immediately
  returns from the PE entry to the thread-exit sentinel after adding generated
  trampoline-origin handling, but it is still not a usable speed path: bounded
  probes are slower/less advanced than the normal path, so do not use it for
  host validation yet.
- Default import bookkeeping now skips very hot CRT/heap/file/DIB imports
  unless full/import trace is explicitly enabled, while keeping milestone,
  window, and presentation imports visible by default. This reduces ordinary
  host-run bookkeeping noise for imports such as `memcpy`, `memset`,
  `ReadFile`, `HeapAlloc`, `malloc`, and `CreateDIBSection`; diagnostic runs
  that need complete import counts should set `WINCE_EMU_IMPORT_TRACE=1` or
  `WINCE_EMU_FULL_TRACE=1`.
- Mounted route-search driving now gets past the `wcstoul @1083` import trap
  and exposes the next process/GWE fidelity frontier. `wcstoul` is implemented
  from the CE ordinal evidence in `coredll.def`/`crt_ordinals.h`, with focused
  raw COREDLL coverage for decimal, prefix, end-pointer, and negative-wrap
  cases. Fresh Win32-host runs with dumped runtime DLLs and
  `--remote-server 192.168.0.39:8765` reached the bottom action/modal route
  path without multi-GB RSS growth. `target\route_drive_procfix1_*` confirms
  `happyway_win.exe` and `iSearch.exe` are no longer falsely marked exited
  when their bounded child runner returns without an encoded CE exit; they are
  recorded as `CreateProcessChildParked` with `STILL_ACTIVE`. The remaining
  blocker is the larger CE process scheduler handoff: the parent `iNavi.exe`
  exits while parked child/worker contexts and a thread-9 synchronous
  `SendMessageW` transaction are still outstanding, so the host process stops
  before route/search can continue in the child context.
- v3 now has a generic v2-style diagnostic companion launcher for mounted
  runs. `--companion-image PATH` and the v2-named alias
  `--companion-target PATH` start an additional v3 process after a short delay,
  using the same registry, devices, mount config, and DLL search paths as the
  parent, but with `--desktop virtual`, `--cpu-instruction-limit 250000000`,
  and no nested remote server. Companion stdout/stderr are written to
  `target\companion_*.stdout.log` and `target\companion_*.stderr.log`, and the
  parent cleans up still-running companions on exit. This mirrors
  `..\wince_emulator_v2\tools\autodrive_inavi.ps1` evidence where
  `TBT\MultiTBT.exe` is a harness-launched companion, not an observed guest
  `CreateProcessW` target. Focused coverage
  `companion_command_uses_shared_config_without_remote_or_nested_companions`
  passes. This is only diagnostic parity: v3 still lacks v2's shared
  cross-process window registry/message broker, so real CE process/mapping IPC
  remains the fidelity target.
- MultiTBT startup now passes the old dumped-MFC/commctrl crash frontier.
  DUMPPLZ search found no `MultiTBT.exe` or direct `TBT\...` launcher evidence
  under `D:\INAVI_Emulator\DUMPPLZ`; only CE `TBTCORE` feature macros appear in
  `Windows\ceconfig.h`, so `..\wince_emulator_v2` remains harness evidence only.
  The crash at `pc=0x80000002`, `ra=mfcce400.dll+0x29780` was an unresolved PE
  ordinal import marker (`IMAGE_ORDINAL_FLAG32 | 2`) left in
  `mfcce400.dll`'s `commctrl.dll` IAT because v3 patched each DLL before later
  loaded DLL exports were known. The loader now does a second pass over loaded
  DLL images to resolve external PE exports by name/ordinal after the full
  dumped DLL export table is built. Focused coverage
  `second_external_pass_patches_late_loaded_ordinal_imports` and
  `patches_loaded_commctrl_exports_from_external_table_without_stub_trap`
  passes. MultiTBT then reached `AddFontResourceW @893` for
  `\SDMMC Disk\TBT\ygo550.ttf`; v3 now implements count-style
  `AddFontResourceW` over guest file mounts, with focused coverage
  `coredll_raw_add_font_resource_uses_guest_file_mounts`. Standalone
  `target\multitbt_font_wall1_*` runs for a 60 s wall slice, creates a visible
  zero-sized `MultiTBT` top window plus hidden child windows, receives timer
  `WM_TIMER id=0xa`, and parks in `GetMessageW` without the old crash. Mounted
  host validation `target\mounted_multitbt_fix1_*` launched the companion with
  empty stdout/stderr and no crash before parent cleanup; the main iNavi process
  still hit the known singleton/encoded-exit path after the wall slice.
- Remote/host touch delivery now preserves event timing and mouse-button key
  state across the CE/GWE boundary. The remote queue records enqueue time on
  touch/key events, synthesized tap aliases give `WM_LBUTTONDOWN` and
  `WM_LBUTTONUP` an 80 ms gap, and `drain_remote_input_to_target` maps those
  deltas into guest `MSG.time` instead of stamping a whole drained batch with
  one tick. GWE also updates `VK_LBUTTON` state for `WM_LBUTTONDOWN/UP`, so
  `GetKeyState(VK_LBUTTON)` reflects mouse-button state like keyboard posts.
  Focused coverage `remote_input_active_window_drain_posts_mouse_messages`,
  `remote_input_blocked_thread_target_overrides_stale_active_window`,
  `queues_touch_key_serial_and_status`, and
  `mouse_button_messages_update_lbutton_key_state` passes. Fresh mounted
  Win32-host probes using dumped runtime DLLs and
  `--remote-server 192.168.0.39:8765` confirm the old same-tick failure is
  gone: `target\menu_popup_touchtime1_*` and
  `target\menu_popup_lbutton1_*` deliver top-right menu tap messages to HWND
  `0x00020004` as `WM_LBUTTONDOWN`/`WM_LBUTTONUP` at `136700`/`136780` ms
  (or equivalent separated times) with bounded RSS/file I/O. The menu/action
  transition itself is still not fixed: after the tap, the guest does not issue
  post-tap `ShowWindow(SW_SHOW)` for hidden child controls `0x00020060`,
  `0x00020068`, or `0x0002006c`.
- The bottom current-location strip is a real, later-shown child window, not a
  host presenter delay. In `target\menu_bottom_compare1_*`, child HWND
  `0x00020070` is created hidden, later shown with `ShowWindow(SW_SHOW)`,
  receives `WM_WINDOWPOSCHANGING/CHANGED`, `WM_SIZE`, and its own `WM_PAINT`
  after the parent/map window work, then responds to a remote tap at the bottom
  strip by posting the app-private `0x5734` message to parent `0x00020004`.
  That path creates and shows top-level/owned `WCE_TGNaviWindow` HWND
  `0x00020084`, and the converted final framebuffer
  `target\menu_bottom_compare1_final.png` shows the real Route Search shell.
  Therefore the visible delay of the bottom strip is currently consistent with
  CE/MFC child-window sequencing; the open bug is the separate top-right
  `메뉴` path not issuing the expected transition.
- Raw `ChildWindowFromPoint` now follows CE/Win32 child-hit-test semantics
  instead of reusing the normal visible/enabled input hit-test. CE docs state
  that `ChildWindowFromPoint` returns a containing child even when it is hidden
  or disabled, and the child search is restricted to immediate children; v3 now
  implements that for raw ordinal `253` while leaving `WindowFromPoint` and
  remote touch targeting on the visible/enabled recursive path. Focused
  coverage in
  `coredll_raw_window_from_point_hits_visible_thread_windows` now checks hidden
  child, disabled child, parent fallback, outside-parent NULL, and strict
  `WindowFromPoint` behavior. The current top-right `메뉴` trace did not show a
  `ChildWindowFromPoint` call around the failed tap, so this is a confirmed CE
  fidelity fix but not yet proven as the visible menu transition fix.
- GWE message retrieval order now follows the CE message-queue order from
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\cmsgque.h`: posted messages are
  returned before received synchronous sends, then quit/paint. Focused coverage
  `posted_messages_are_retrieved_before_received_sends` and existing raw
  queued cross-thread send coverage pass. This is a CE-correct queue-order fix,
  but mounted `target\menu_popup_orderfix1_*` evidence showed it is not by
  itself sufficient to make the top-right menu tap reveal the expected action
  controls.
- Windows-sudo `cargo flamegraph` profiling is now in use for mounted iNavi
  startup. Fresh virtual runs
  `target\startup_flame_virtual_sudo1.svg` and
  `target\startup_flame_virtual_sudo3.svg` reached the same post-map
  scheduler/file frontier at `COREDLL.dll@496` (`Sleep`) with bounded RSS/file
  counters (`host_open=912`, `host_read=83985/~6.46 MB`, `mem_open=4`,
  `max_read=685080`). The first profile showed repeated import bookkeeping
  cost in `trace_import_name`, `coredll_ordinals::lookup`, string clones, and
  `Vec` argument clones. v3 now caches COREDLL ordinal lookup with the same
  static/SDK/supplemental precedence, borrows normal IAT `ImportTrap`s instead
  of cloning them on every trap, and routes live import dispatch through a
  borrowed argument slice. The final profile drops `trace_import_name`,
  linear ordinal lookup, and the `Vec` clone frames from the filtered hot list;
  remaining visible cost is spread across Unicorn guest execution, generic raw
  COREDLL dispatch, `combine_rgn_raw`, streamed `read_file_into`, and guest
  memcpy.
- Win32 host presenter now has an explicit stopped state. When a CPU run
  segment ends or errors while the host window stays alive for monitor/debug
  use, v3 replaces the last guest frame with a black status surface reading
  `Emulator process stopped`, clears pending host input, and handles later
  mouse/key messages without queuing guest input. Entering another host CPU run
  blits the current framebuffer first, returning the presenter to live input.
  This is host-tooling behavior only; it does not alter guest framebuffer
  contents or CE input semantics.
- GWE `SetWindowPos` now schedules repaint for clean, already-visible windows
  when they are moved, resized, or z-order promoted unless the caller passed
  `SWP_NOREDRAW`; `MoveWindow(FALSE)` now maps to the same no-redraw rule.
  Hide/exposure invalidation remains intact. Focused coverage
  `set_window_pos_invalidates_clean_visible_window_without_no_redraw`,
  `set_window_pos_no_redraw_keeps_clean_visible_window_clean`, and
  `set_window_pos_z_order_change_invalidates_promoted_visible_window` passes,
  as does the full `ce::gwe::tests` slice. This is a generic CE/MFC window
  update fix aimed at visible child bands that react logically but do not get a
  paint opportunity after layout/z-order changes.
- COREDLL CRT `_hypot @1023` is now implemented through the generic `cemath`
  binary-f64 path. The mounted Route Search/menu probe
  `target\modal_drive_host1_*` previously stopped at
  `trap=COREDLL.dll@1023` (`_hypot` in
  `C:\WINCE600\PUBLIC\COMMON\OAK\LIB\MIPSII\RETAIL\coredll.def`) after the
  route/current-location dialog path. v3 now exposes `ORD_HYPOT`, evaluates
  `lhs.hypot(rhs)`, and routes both typed and raw dispatch through
  `CeMathBinaryF64::Hypot`. Focused coverage in `tests\coredll_dispatch.rs`
  and `src\ce\cemath.rs` passes; the full `coredll_dispatch` test target and
  `cargo check --features unicorn,trace,win32-desktop` pass with only the known
  Windows incremental cleanup warning. Fresh Win32-host validation
  `target\hypot_route_host1_*` used dumped DLLs from
  `D:\INAVI_Emulator\DUMPPLZ\Windows` and
  `--remote-server 192.168.0.39:8765`; it drove safety OK, dismissed the
  destination/current-location modal via the real X, opened the bottom menu,
  selected `목적지`, selected the highlighted row, tapped the red search button,
  and then reached the 300 s wall-stop in guest image code
  `pc=0x00114cc4(image:iNavi.exe+0x104cc4)` instead of a missing import trap.
  The final visible frame `target\hypot_route_host1_after_search.png` shows a
  live shifted map/search-focus state with the host process responsive and
  bounded around 249 MB before the wall-stop.
- GWE now invalidates newly exposed screen areas when a visible window is
  destroyed, hidden through `ShowWindow(FALSE)`, or hidden/moved away through
  `SetWindowPos`. The fix stays at the generic CE window/update boundary:
  surviving visible windows whose client rect intersects the old screen rect
  receive clipped update rectangles and repaint through the normal `WM_PAINT`
  path. Focused coverage
  `destroy_window_invalidates_newly_exposed_windows`,
  `hiding_window_invalidates_newly_exposed_windows`, and
  `set_window_pos_hide_invalidates_newly_exposed_windows` passes, as does the
  full `ce::gwe::tests` filter and `cargo check --features
  unicorn,trace,win32-desktop`. Mounted Win32-host validation
  `target\gwe_exposure_host2_after_tap.png` used dumped runtime DLLs from
  `D:\INAVI_Emulator\DUMPPLZ\Windows` plus
  `--remote-server 192.168.0.39:8765`; after a remote tap at `(645,443)`, the
  old fullscreen safety pixels are gone and the real route/current-location
  modal is shown over the map/current UI. The remaining visible blocker is the
  app modal/continuation path, not safety-window dismissal or stale framebuffer
  presentation.
- No-wall Win32-host remote input now drains during long Unicorn execution,
  not only when a CPU wall-clock limit is configured. The Unicorn live code hook
  now services REST controls and Win32 host input on the normal framebuffer tick
  path before the optional wall-stop check, so mounted launches using
  `--remote-server 192.168.0.39:8765` remain touchable while guest code is
  running. Fresh validation `target\live_host_after_drain_fix.png` shows a
  no-wall host process (PID `25908` during the probe) accepting a remote OK tap
  and advancing from the safety notice to the real map UI; the process stayed
  responding at roughly 247 MB RSS. A traced bounded repro
  `target\remote_ok_trace_*` records the tap as `WM_LBUTTONDOWN/UP` targeted to
  HWND `0x00020080` and consumed by `GetMessageW`, then reaches the populated
  map with all synchronous sends completed (`gwe=send:68 done:68`).
- Host-mode `--cpu-wall-clock-limit-ms` is now treated as a total run-loop
  budget instead of being reset for every inner Unicorn burst. This fixes a
  misleading diagnostic shape where bounded host validations would hit a
  healthy wall-stop, continue because a `GetMessageW` waiter existed, and then
  later re-enter the app singleton/exit path. Focused coverage
  `host_loop_wall_clock_budget_is_total_not_per_burst` passes; fresh
  `target\bounded_total_45s_*` stops once at the requested total budget and no
  longer prints a fabricated post-limit CE process exit.
- Child process execution now saves and restores the full current-process
  pseudo-handle state (`process_id`, pseudo exit code, and signaled flag) around
  `run_pending_process_launches`. A child that terminates
  `CE_CURRENT_PROCESS_PSEUDO_HANDLE` no longer leaves the parent pseudo process
  handle signaled. Focused coverage
  `child_current_process_exit_state_does_not_signal_parent_pseudo_handle`
  passes. The attempted deferred child-launch ordering was reverted; children
  are again executed immediately after successful `CreateProcessW`, preserving
  the previously fast mounted startup path.
- Remote-server touch input during a host `GetMessageW` idle wait now targets
  the parked message thread/window instead of draining only through stale
  active-window fallback. The host run loop passes the last blocked
  `GetMessageW` snapshot into remote-server control draining, and
  `CeKernel::drain_remote_server_control_messages_to_thread_window` dispatches
  REST controls then drains touch/key events through the same blocked
  thread/window path used for host mouse input. The Unicorn live-tick drain
  now uses that parked `GetMessageW` target too, closing the race where REST
  controls could be consumed through active-window fallback during long host
  runs before the outer loop saw the blocked snapshot. Focused coverage
  `remote_input_blocked_thread_target_overrides_stale_active_window` passes.
  A short optimized trace repro `target\safety_remote_trace_*` confirms a
  remote tap at the safety-notice OK button posts and delivers
  `WM_LBUTTONDOWN/WM_LBUTTONUP` to HWND `0x00020080` through `GetMessageW`.
  The app then leaves the safety screen and reaches its existing encoded
  current-process terminate path (`api2.2`, process `0x42`, code `0`), so the
  remaining "ded after OK" symptom is now guest continuation/device/state
  fidelity, not remote HTTP input loss.
- Fresh host message evidence `target\remote_ok_tap_preidle_*` confirms the
  accepted REST tap is delivered as `WM_LBUTTONDOWN/WM_LBUTTONUP`; when sent
  too early it legitimately hits the then-front HWND `0x00020008`. The later
  fatal path remains the existing singleton/front-window branch:
  `CreateMutexW(L"iNavi")` reports `ERROR_ALREADY_EXISTS`, `FindWindowW`
  finds HWND `0x00020000` by title `iNavi`, `ReleaseMutex` fails with
  `ERROR_NOT_OWNER`, and the app reaches the encoded current-process exit
  thunk at `0x0048fa90`. The active blocker is therefore the old
  singleton/window identity or lifecycle frontier, not remote transport,
  Win32 presenter responsiveness, or file-I/O startup speed.
- The Unicorn serial wait bridges now yield to already-ready scheduler waiters
  after parking the current serial operation. Empty serial `ReadFile` and
  `WaitCommEvent` previously registered the active thread's serial wait, then
  either completed the serial timeout itself or stopped when no suspended peer
  context existed, even if another blocked waiter was already signaled. Both
  paths now try the common ready-waiter resume helper before self-timeout/stop,
  preserving CE scheduler ownership and avoiding serial/device waits pinning
  ready worker progress behind the current thread. Focused coverage
  `serial_read_yields_to_ready_blocked_waiter_before_timeout` and
  `wait_comm_event_yields_to_ready_blocked_waiter` passes. Mounted validation
  `target\sched_serial_handoff_virtual_60s_*` used dumped runtime DLLs from
  `D:\INAVI_Emulator\DUMPPLZ\Windows`, ran the full 60 s wall budget, stayed
  bounded (`heap_live=7591/23053925B`, `virtual_live=2/131072B`,
  `host_open=228`, `host_read=37983/2184009B`, `mem_open=2`,
  `max_read=497178`), and produced a real populated 800x480 map framebuffer.
  The short probe stopped in guest image code
  `pc=0x0034285c(image:iNavi.exe+0x33285c)` with no registered waiters
  (`reg:0/0`), so it validates no regression and real UI output rather than
  proving the later post-map ANR frontier has moved.
- The Unicorn `WaitForMultipleObjects` and `MsgWaitForMultipleObjectsEx`
  block bridges now hand off to already-ready scheduler waiters after parking
  the current thread, even when there is no saved suspended peer context.
  Previously the multiple-wait path could register the current thread's
  kernel wait and then only look for a `GetMessageW` waiter before stopping,
  while the msg-wait path stopped immediately after checking the suspended
  slot. Both paths now use the same ready-waiter and bounded-timeout helper
  shape as single waits and sleeps, preserving scheduler-owned state instead
  of stranding signaled worker waits. Focused coverage
  `current_multiple_wait_yields_to_ready_blocked_waiter` and
  `current_msg_wait_yields_to_ready_blocked_waiter` passes; the filtered
  `cargo test wait_scheduler --features unicorn,trace,win32-desktop` suite now
  has 34 passing tests, and `cargo check --features
  unicorn,trace,win32-desktop` passes with only the known Windows incremental
  finalization warning.
- The Rust remote server transport now exists and is aligned with the
  `..\wince_emulator_v2` REST API shape while staying wired through v3's generic
  `CeRemote` state. CLI supports the requested compact
  `--remote-server IP:PORT` form plus v2-compatible `--remote-server`,
  `--remote-bind`, `--remote-port`, `--remote-token`, video, JPEG-quality, and
  audio metadata flags. The server exposes `/api/v1/status`,
  `/api/v1/frame.jpg`, `/api/v1/debug/screenshot.png`,
  `/api/v1/video.mjpg`, `/api/v1/input/touch`, `/api/v1/input/key`,
  `/api/v1/sensors/location`, `/api/v1/sensors/nmea`,
  `/api/v1/sensors/imu`, `/api/v1/logs/recent`,
  `/api/v1/control/pause`, and `/api/v1/control/resume`; status uses the v2
  camelCase JSON names and frame endpoints encode the real guest framebuffer as
  JPEG/PNG/MJPEG. Posted REST controls are drained through
  `CeKernel::dispatch_remote_control_message` and live Unicorn ticks call the
  same drain path so remote touch/key/GPS input can wake CE scheduler/GWE paths
  during long guest execution. Focused coverage
  `remote_server_accepts_v2_touch_route`,
  `remote_server_accepts_v2_single_touch_alias`,
  `remote_server_rejects_invalid_v2_input_bodies`,
  `remote_server_serves_frame_with_v2_quality_query_and_error`,
  `remote_server_serves_v2_status_shape`, and the existing
  `remote_server_api_state_queues_input_serial_audio_and_status` pass; `cargo
  check --features unicorn,trace,win32-desktop` also passes with only the known
  Windows incremental finalization warning. Follow-up WebSocket support now
  upgrades `/api/v1/control/ws` and `/api/v1/audio/ws`: control text frames are
  queued as JSON control messages with JSON acknowledgements, and audio sockets
  receive a metadata text frame followed by binary PCM frames from the
  server-backed audio sink when `--remote-audio` is enabled. Focused coverage
  `remote_server_control_websocket_queues_json_frames` and
  `remote_server_audio_websocket_streams_registered_sink_pcm` passes.
- Follow-up v2 remote API alignment tightened the REST handler contract against
  `..\wince_emulator_v2\src\remote_server.cpp`: touch/key/location/NMEA bodies
  now return the same invalid-body errors, touch aliases include `tap`, `click`,
  `single`, and `single-touch`, key phases are limited to `down`/`up`, frame and
  MJPEG endpoints honor per-request `quality`/`fps`, missing framebuffer now
  reports `{"error":"no framebuffer"}`, and successful REST input/control
  responses use the compact v2 `{"ok":true}` shape while still queueing through
  generic `CeRemote`/`CeKernel` dispatch.
- The Unicorn `MsgWaitForMultipleObjectsEx` parking bridge now preserves
  scheduler-owned blocked state when the current guest thread has no runnable
  suspended peer. Previously the raw ordinal bridge could register the current
  thread as `BlockedWaitKind::MsgWait`, clear `running_thread`, and then return
  `false`, allowing raw dispatch fallthrough with inconsistent scheduler
  ownership. It now stops Unicorn and returns handled after registering the
  waiter. This follows the CE scheduler/message-queue reference already
  recorded from `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\schedule.c` and
  `GWE\INC\cmsgque.h`: over-budget message waits remain owned by the
  queue/scheduler until a message, object, or timeout wake. Focused coverage
  `current_msg_wait_without_peer_parks_and_stops_instead_of_falling_through`
  passes, and `cargo check --features unicorn,trace,win32-desktop` passes with
  only the known Windows incremental finalization warning.
- The Unicorn serial wait bridge now applies the same scheduler-owned blocking
  rule to `WaitCommEvent`: when a running guest thread parks on an empty serial
  comm event and no suspended peer exists, the bridge registers a
  `SerialCommEvent` waiter, clears the running slot, stops Unicorn, and returns
  handled instead of falling through raw dispatch. Serial read and comm-event
  blocking also purge stale vector-backed waits for the same thread before
  registering a new wait, matching the CE one-blocked-state invariant recorded
  from `schedule.c`/`cmsgque.h`. Focused coverage
  `current_wait_comm_event_without_peer_parks_and_stops_instead_of_falling_through`
  proves the no-peer path parks, removes stale waiter metadata, and preserves
  guest registers; `cargo test wait_scheduler --features
  unicorn,trace,win32-desktop` passes all 31 filtered tests and `cargo check
  --features unicorn,trace,win32-desktop` passes.
- The Unicorn `Sleep(0)` bridge now follows CE `NKSleep` yield semantics from
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\schedule.c`: `cMilliseconds ==
  0` calls `ThreadYield`, and if no runnable peer exists the current guest
  thread returns immediately with `V0=0` rather than falling through raw
  dispatch. Focused coverage
  `current_yield_sleep_without_peer_returns_to_same_thread` proves the no-peer
  yield records the scheduler yield, preserves the current running thread, and
  returns to the guest RA. `cargo test wait_scheduler --features
  unicorn,trace,win32-desktop` now passes all 32 filtered tests, and `cargo
  check --features unicorn,trace,win32-desktop` passes with only the known
  incremental finalization warning.
- CE-style pending timer-message coalescing is now implemented for scheduler
  generated `WM_TIMER`. `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\cmsgque.h`
  models timer entries with separate message-queue and timer-queue linkage plus
  a fired state, so v3 now marks a periodic timer as having a pending message
  when `pump_timers_to_gwe` posts it and only clears that state when
  `GetMessageW`/removing `PeekMessageW` actually consumes the timer message.
  Focused coverage `kernel_timer_messages_coalesce_while_pending` proves
  repeated timer pumps do not queue duplicate messages and that consuming the
  message permits the next period to post. Full
  `cargo test --features unicorn,trace,win32-desktop`, `cargo check`, and
  release build pass. Fresh visible host validations
  `target\host_timer_pending_300s_*`, `target\host_modal_click_260s_*`,
  `target\host_windows_220s_*`, and `target\host_modal_lateclick_300s_*` used
  dumped runtime DLLs from `D:\INAVI_Emulator\DUMPPLZ\Windows`, stayed
  host-window responsive and memory/file-I/O bounded, and reduced timer `4565`
  from the previous repeated post streak to one pending post in the message
  trace. The final frame is the real GPS initialization warning modal
  (`Error Code: -14`). Host input injection reaches GWE and delivers
  `WM_LBUTTONDOWN/UP`, but the tested clicks landed before the top modal HWND
  `0x00020084` was created and hit the underlying full-screen `TGNaviDlg`
  `0x00020080`; the remaining ANR/frontier is therefore GPS/modal/device
  continuation, not timer flooding, host input loss, hidden-layer leakage, or
  file-I/O/RSS growth.
- Follow-up visible host validation `target\host_modal_clickburst_300s_*`
  clicked the OK area repeatedly across the GPS-warning transition. Two
  delivered `WM_LBUTTONDOWN/UP` events hit the top warning modal HWND
  `0x00020084`; that modal is `dead=true` in the final window snapshot and the
  final framebuffer returns to the earlier safety notice with the bottom OK
  button. This confirms the GPS warning is dismissible through real host input
  and GWE hit-testing. The run stayed host-window responsive and bounded, ran
  almost to the 300 s wall budget, and then reached the same app-owned encoded
  terminate path while waiting at the safety notice, so the next manual-host
  probe should click the safety notice OK after the GPS warning is dismissed
  before treating the state as a new scheduler ANR.
- The post-map Win32-host pending-send deadlock is fixed. A 300 s host probe
  before this slice (`target\host_handoff_300s_*`) ended with thread 9 blocked
  in a synchronous send while thread 1 was parked in `GetMessageW`
  (`gwe=send:19 done:18`, `blocked_waits` containing
  `thr=9/kind=send_message` plus `blocked_get_message=thread:1`) because the
  Unicorn bridge refused to resume the blocked current UI thread when
  `current_thread_id == blocked_get_message.thread_id` even though no guest
  thread was actually running. The `GetMessageW` resume helper now
  distinguishes that parked-current state from a truly running active context
  and does not save a stale active snapshot during blocking handoff. Focused
  coverage includes
  `blocked_current_get_message_resumes_pending_send_when_no_thread_running`,
  `current_sleep_yields_to_ready_blocked_get_message`, the full
  `wait_scheduler` suite, and full
  `cargo test --features unicorn,trace,win32-desktop --lib` with 177 passing
  tests. Fresh visible host validation
  `target\host_getmsg_sendwake_300s_*` used dumped runtime DLLs from
  `D:\INAVI_Emulator\DUMPPLZ\Windows`, stayed memory/file-I/O bounded
  (`host_open=910`, `host_read=82824/6410280B`, `mem_open=4`), rendered the
  real map UI, and no longer has a pending synchronous send at the wall stop
  (`gwe=send:17 done:17`). The remaining ANR shape moved to main-thread MFC
  message-pump execution with `blocked_get_message=thread:1`, finite worker
  sleeps, active timer `0x11d5`, Deneb/COM7/SMB1/MFS1 device activity, and one
  saved worker context (`suspended:6:0x00000f0c:pc=0x0022fa90`), so the next
  slice should focus on runnable-context fairness and device/timer/message
  progression, not the now-closed pending-send deadlock.
- Win32 host input is no longer limited to outer run-loop stop points. The host
  presenter already pumped and blitted frames during long Unicorn execution,
  but host mouse/key events were only polled before/after `run_until` or when a
  previous run had stopped at a blocked `GetMessageW`. The Unicorn live tick
  now drains the existing Win32 input queue, enqueues those events through the
  normal `CeRemote` touch/key path, and lets GWE route them to the active or
  captured CE window with ordinary hit-testing and message-wake behavior.
  Focused coverage `remote_input_active_window_drain_posts_mouse_messages`
  proves queued touch input becomes normal `WM_LBUTTONDOWN/WM_LBUTTONUP`
  messages for the active HWND, and the full
  `cargo test --features unicorn,trace,win32-desktop --lib` suite now passes
  178 tests. Fresh visible host validation `target\host_live_input_300s_*`
  stayed bounded and reached the same real map UI with all sync sends completed
  (`gwe=send:24 done:24`); no manual `remote_touch`/`remote_key` was recorded
  in that run's trace, so the deeper post-map ANR remains open, but a real
  host-input latency blind spot is closed.
- The post-map Win32-host scheduler bridge now avoids two more generic
  starvation traps. The Unicorn code-hook scheduler timeslice no longer drops
  a due slice just because the fixed sample lands on an import trap,
  trampoline page, pending WNDPROC return, MIPS control-transfer target, or
  delay-slot PC; it keeps the slice pending and consumes it at the next safe
  instruction. Current-thread `Sleep` inline completion also now sees ready
  blocked `GetMessageW` waiters, so a worker sleep cannot keep running itself
  while the UI thread has queued posted/sent traffic. Focused coverage includes
  `timeslice_pending_survives_unsafe_sample_until_safe_pc` and
  `current_sleep_yields_to_ready_blocked_get_message`; full
  `cargo test --features unicorn,trace,win32-desktop --lib` now passes 176
  tests, with `cargo check` passing except the known Windows incremental
  finalization warning. Fresh Win32-host validations
  `target\host_timeslice_pending_180s_*` and
  `target\host_sleep_getmsg_180s_*` used dumped runtime DLLs from
  `D:\INAVI_Emulator\DUMPPLZ\Windows`, stayed memory/file-I/O bounded
  (`host_open=910`, `host_read=82822/6400466B`, `mem_open=4`), and ran the full
  180 s wall budget without backend faults. The frontier moved from valid
  running thread 5 in image code to worker sleep handoff states; latest wall
  evidence ends at `COREDLL.dll@496` from thread 8's long sleep
  (`ra=image:iNavi.exe+0xd69c0`) with main `GetMessageW`, short worker sleeps,
  COM7 empty reads, Deneb device reads, and `around.db` map/search reads still
  in the evidence trail.
- The post-map Win32-host scheduler frontier moved forward again with generic
  CE thread ownership and bounded timeout handoff fixes. Cross-thread
  `SendMessageW` receiver-context WNDPROC callouts now keep real receiver
  thread ownership when the target window belongs to a created guest thread,
  and send completion recovers the sender's running-thread tuple from the
  blocked waiter/thread handle if older metadata is missing. Current-thread
  `Sleep` no longer fast-forwards itself ahead of an already-blocked waiter
  whose finite timeout matures first; the same bounded wait-to-next-ready path
  is also available after object-wait blocking. Focused coverage includes
  `send_message_callout_enters_cross_thread_receiver_context`,
  `send_message_blocked_wait_resume_recovers_missing_running_metadata`, and
  `current_sleep_waits_to_next_blocked_sleep_timeout`; full
  `cargo test --features unicorn,trace,win32-desktop --lib` now passes 174
  tests. Fresh Win32-host validation `target\host_sleepctx_180s_*` used dumped
  runtime DLLs from `D:\INAVI_Emulator\DUMPPLZ\Windows`, stayed bounded
  (`heap_live=14652/31406594B`,
  `file_io=host_open:924 host_read:83673/6454222B mem_open:4`), and moved the
  wall-stop from `image:iNavi.exe+0x48684c` in `target\host_sendctx_180s_*` to
  `image:iNavi.exe+0xa5d7e0` with more scheduler/message activity
  (`wake:1167`, `timeout:951`, `gwe=send:506 done:506`). The remaining ANR
  shape is still open: the 180 s wall snapshot can end with
  `threads=current:5/running:none` while thread 5 is also a finite sleep
  waiter, so the next slice should continue with run-queue ownership for
  blocked current contexts instead of treating this as a rendering or host
  presenter failure.
- A follow-up scheduler ownership fix removes the stale runnable copy created
  when the active thread blocks and immediately hands off to another ready
  waiter. `try_resume_blocked_wait_with_active_pc` now distinguishes true
  preemption, which must save the active context, from blocking handoff, where
  the active registers already live in the blocked-wait record. Focused tests
  updated `current_sleep_yields_to_ready_blocked_waiter` and
  `get_message_block_yields_to_timed_out_worker_sleep` to assert no runnable
  suspended clone remains for the just-blocked thread, while timeslice tests
  still prove preemption saves the active context. Full
  `cargo test --features unicorn,trace,win32-desktop --lib` passes 174 tests.
  Fresh Win32-host validation `target\host_blockctx_180s_*` used the dumped
  runtime DLL path and no longer reports the contradictory
  `current:5/running:none` plus thread-5 sleep waiter shape: the wall snapshot
  now ends with `threads=current:5/running:5:0x00000f00/suspended:none/queue:0`
  and blocked waits only for other worker/kernel/serial/main-message waits.
  This confirms the stale blocked-current ownership bug is closed; remaining
  post-map ANR work should now continue from valid running thread 5 at
  `image:iNavi.exe+0x13e5c4` with main thread parked in `GetMessageW` and
  worker/device waits still pending.
- The post-map Win32-host scheduler fault moved past the bad tree-pointer
  crash after the Unicorn saved-context fix. Guest thread contexts now preserve
  MIPS HI/LO along with the 32 GPRs across blocked-wait, `GetMessageW`,
  send-message, sleep/yield, and ready-waiter scheduler resumes; stale saved
  snapshots for a thread are also removed from the primary suspended slot and
  FIFO overflow queue before that same thread is resumed. Focused coverage
  extends the ready-waiter preemption test to prove active HI/LO is saved and
  the resumed waiter HI/LO is restored, and the stale `GetMessageW` resume
  regression still passes. Full `cargo test --features unicorn,trace,win32-desktop --lib`
  passes. Fresh Win32-host mounted validation
  `target\host_fullctx_180s_*` used dumped runtime DLLs from
  `D:\INAVI_Emulator\DUMPPLZ\Windows`, ran the full 180 s wall budget without
  the previous `READ_UNMAPPED addr=0x14400018` fault, stayed bounded
  (`heap_live=14649/31405978B`, `virtual_live=2/131072B`,
  `host_open=924`, `host_read=83673/6450243B`, `mem_open=4`,
  `max_read=685080`), and ended the wall-stop in guest image code
  (`pc=0x0014e7dc(image:iNavi.exe+0x13e7dc)`). The final frame is the real map
  plus the app's GPS initialization warning modal (`Error Code: -14`), so the
  active frontier is now GPS/serial/Deneb/system-state fidelity and modal/UI
  continuation, not the prior scheduler context-corruption fault.
- Host ANR diagnostics now preserve the actual wall-clock stop snapshot even
  when later cleanup reaches the app's legacy terminate path. Commit `7866f07`
  adds `preferred_trace_snapshot()` plus opt-in
  `WINCE_EMU_GUEST_ENTRY_TRACE` sampling around the guest helper
  `0x000e8ce4..0x000e9d10`. Fresh host evidence
  `target\host_anr_plain_*` used dumped runtime DLLs from
  `D:\INAVI_Emulator\DUMPPLZ\Windows`, ran the full 180 s host budget after the
  visible map, and preserved the real stop at
  `pc=0x0022f9ec(image:iNavi.exe+0x21f9ec)`,
  `ra=0x0022f944(image:iNavi.exe+0x21f944)`. Guest-entry samples showed
  healthy return addresses at `entry_e9a40`/`return_e9d0c`, so the earlier
  `ra=0` suspicion did not reproduce under the new sampler. The same run stayed
  bounded (`heap_live=15047/32541135B`,
  `file_io=host_open:907 host_read:83092/5905148B mem_open:4`) while render
  traces showed real GDI display blits and file traces showed a later COM7
  open/read-zero storm with `$PUBX,40,GSV,...` GPS setup writes.
- `win32_com` serial devices now have a first generic host bridge. Enabled
  serial devices configured with backend `win32_com` and a host name are opened
  as Windows COM handles, configured for nonblocking reads and the current DCB
  line state, polled before CE `ReadFile` drains/parks, and receive guest
  `WriteFile` bytes. Focused coverage includes
  `win32_com_without_host_still_opens_as_serial_session`,
  `win32_com_path_uses_device_prefix`, the existing serial timeout fixtures,
  and `cargo check --features unicorn,trace,win32-desktop`. The current local
  Windows port evidence only listed COM4/COM9 as OK while `serial_devices.json`
  maps guest `COM7:` to host `COM21`, so the mounted ANR frontier still needs a
  verified host GPS/serial source instead of treating empty COM21 reads as app
  evidence.
- The Win32 host-presenter ANR symptom is fixed at the host boundary. The host
  window is now created and pumped on a dedicated GUI thread, while the emulator
  thread continues to blit real guest framebuffer pixels and poll the shared
  input queue. This does not change CE GWE/message semantics. Before the change,
  `target\host_anr_current_*` reached
  `pc=0x60024834(dll:mfcce400.dll+0x24834)` around 168 s with
  `Responding=False`; after the change, `target\host_gui_thread_*` reached the
  same PC around 180 s with `Responding=True`, similar RSS
  (`~224 MB`), and then followed the same app-owned encoded terminate path at
  `image:iNavi.exe+0x47fa94`. The remaining post-map issue is therefore guest
  message/scheduler/device behavior, not a frozen host Win32 window.
- The post-map ANR-shaped scheduler stop was narrowed and moved forward by a
  generic Unicorn wait-bridge fix. Fresh host evidence
  `target\anr_wait_cleanup_host_*` first confirmed the stale duplicate
  main-thread wait from `target\public_message_trace_*` is gone: thread `1`
  no longer appears as both `sleep` and `get_message`. A follow-up no-tap
  control initially showed the remaining ANR shape: main thread parked in
  `GetMessageW` while worker threads still had short finite waits
  (`201/301/334/501 ms`) and a serial read timeout pending. The bridge now
  removes stale saved waits before registering new `GetMessageW`,
  `MsgWaitForMultipleObjectsEx`, and blocking object waits, and an empty
  main-thread `GetMessageW` block waits to the next finite blocked-worker
  timeout when it fits the host budget, then resumes that worker through the
  normal scheduler path. Focused Unicorn regressions
  `get_message_block_registration_clears_stale_sleep_wait` and
  `get_message_block_yields_to_timed_out_worker_sleep` pass, and the full
  `cargo test --features unicorn,trace,win32-desktop` suite passes. Mounted
  validation `target\anr_worker_resume_virtual_*` now runs the full 60 s wall
  budget inside guest iNavi image code
  (`pc=0x00984c64(image:iNavi.exe+0x974c64)`) instead of stopping at idle
  `GetMessageW`; scheduler activity rises to `block:1307/wake:441`, timers
  advance to due `140613`, and GWE records `send:168 done:167`. Current
  frontier is the final in-flight thread-9 send or subsequent map/image code,
  not lost host input, duplicate main-thread waits, or UI idle freezing all
  worker timeouts. Follow-up host evidence added invalid-indirect stack-word
  diagnostics and narrowed the next ANR shape further: `target\host_anr_pc0_*`
  did not reproduce the earlier `pc=0` return, but spent the full 90 s in
  guest image code with several signaled blocked waits still parked. A
  conservative Unicorn time-slice now swaps the active guest context with the
  already-suspended peer context at bounded code-hook intervals, skipping
  import traps, trampoline pages, and pending WNDPROC returns. Visible host
  validation `target\host_timeslice_*` ran for 120 s with no `pc=0`, reached
  MFC trampoline code instead of the earlier image-code wall stop, kept bounded
  file/RSS counters (`host_open=898`, `host_read=81537/5294249B`), and showed
  more real GDI/map work (`BitBlt=177`, `CreateDIBSection=412`) with active
  timers. Remaining evidence still shows signaled waiters parked, so this is
  a scheduler movement, not the final CE run-queue fix. The next generic slice
  lets the time-slice preempt the active running context into the suspended slot
  and resume a ready blocked waiter when no existing suspended peer would be
  overwritten; focused regression
  `timeslice_preempts_active_thread_to_ready_blocked_waiter` covers preserving
  the active PC and completing the ready wait. Fresh visible host validation
  `target\host_ready_preempt_*` moved into a normal CE wait at
  `COREDLL.dll@497` and then the app's own legacy terminate path near the wall
  budget, but exposed multiple simultaneous thread-1 `GetMessageW` scheduler
  waiters. The empty-queue `GetMessageW` bridge now clears the separate
  `blocked_guest_thread` waiter for that thread before registering a new one;
  focused coverage
  `get_message_block_registration_clears_stale_get_message_wait` prevents that
  duplicate scheduler state. Follow-up visible host validation
  `target\host_getmsg_cleanup_*` confirms the duplicate is gone: only
  `id=46/thr=1/kind=get_message` remains for the UI thread at the stop, with
  bounded counters (`host_open=902`, `host_read=81850/5311346B`) and the known
  app-owned legacy terminate path afterward. The remaining scheduler frontier
  is now a clean single UI `GetMessageW` plus worker sleeps/kernel waits, not
  duplicate main-thread message wait bookkeeping. The Unicorn bridge now has
  the next ready-run slice: a FIFO saved-context overflow queue behind the
  primary suspended slot for scheduler time-slice/ready-waiter preemption. This
  lets the bridge preserve an active running context even when another runnable
  context is already suspended. Focused coverage
  `timeslice_ready_waiter_queues_active_when_suspended_slot_is_occupied` passes
  alongside `timeslice_preempts_active_thread_to_ready_blocked_waiter`, and the
  full `cargo test --features unicorn,trace,win32-desktop` suite passes with
  160 lib tests.
- Host/manual post-map input now has a durable kernel-level GWE/message trace
  under the existing `messages` monitor selector. The trace records generic
  message posts, host/remote touch target/drop decisions, keyboard target/drop
  decisions, and `GetMessageW`/`PeekMessageW` delivery with thread id, HWND,
  message id, params, source, and screen coordinates. Focused coverage extends
  the remote touch and blocked-`GetMessage` fixtures, and the full
  `cargo test --features unicorn,trace,win32-desktop` suite passes. A bounded
  Win32-host mounted probe
  `target\host_message_trace_{summary,messages,counts}.txt` used dumped DLLs
  from `D:\INAVI_Emulator\DUMPPLZ\Windows`; the synthetic `400,240` tap was
  hit-tested to HWND `0x00020080`, delivered as `WM_LBUTTONDOWN`/`WM_LBUTTONUP`
  through `GetMessageW`, and the guest then ran its own legacy current-process
  terminate path (`api2.2`, process `0x42`, code `0`). This proves that tap is
  not being dropped before the CE queue.
- The same durable GWE/message trace now covers public message entrypoints,
  not only internal `post_gwe_message` calls: `PostMessageW` window/thread/
  broadcast posts, keyboard-post helpers, `SendNotifyMessageW`, and queued
  cross-thread sends all record the target thread/HWND, message id, params,
  message source, and sender detail before waking scheduler waiters. Focused
  coverage `public_message_entrypoints_record_durable_gwe_trace` passes, the
  full feature test suite passes, and mounted virtual validation
  `target\public_message_trace_{summary,messages,counts}.txt` shows real iNavi
  public posts, broadcasts, queued sends from worker thread `9`, delivered
  `WM_LBUTTONDOWN`/`WM_LBUTTONUP`, and timer `4565` in one trace.
- Window-HDC drawing now uses CE-style visible client regions instead of only
  a raw client rectangle. GWE computes multi-rect visible client areas from
  the window's own visibility, parent visibility, window region, and front
  siblings in z-order; framebuffer HDC drawing intersects those rects before
  `FillRect`, `BitBlt`, `StretchBlt`, `TransparentImage`, and line/polygon
  primitives can touch the host surface. This matches CE GWE's
  `m_hrgnVisible`/`m_hrgnClientVisible` shape from
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\window.hpp` and DC clipping
  ownership from `...\GWE\MGDI\INC\dc.hpp`. Focused coverage includes
  `coredll_raw_bitblt_skips_effectively_hidden_window_hdc` and
  `coredll_raw_bitblt_clips_window_hdc_behind_visible_sibling`; the full
  `cargo test --features unicorn,trace,win32-desktop` suite passes. Host
  validation after commit `8fa8c9f` confirmed the pre-rendered hidden button
  layer leak is fixed in the live Win32 presenter.
- Old direct CE MIPS process-terminate calls are no longer treated only as a
  diagnostic `pc=0` snapshot. When Unicorn decodes the legacy process API
  method 2 terminate target (`api2.2`, e.g. `target=0xfffff3fa`) it now applies
  `kernel.terminate_process(process, exit_code)` before returning `Ok(())`.
  This keeps waitable current-process state and exit code consistent with the
  raw `TerminateProcess` path. The decoder is now also checked before
  interrupt-probe stops are reported as fatal, matching the `DeviceParser.exe`
  zero-PC shape from v2. Fresh mounted validation
  `target\route_deviceexit1_*` records `DeviceParser.exe` as
  `CreateProcessChildReturned`/`CreateProcessExited` instead of
  `CreateProcessChildError`, then continues through `happyway_win.exe`,
  `iSearch.exe`, and the completed `SendMessageW` helper transaction
  (`gwe=send:1 done:1`). The active route frontier is again the hidden
  `afxwnd42u` chrome show sequence, not the old encoded-exit crash.
- Longer bounded virtual validation `target\route_deviceexit_long1_*` keeps
  the same shape after the decoded `DeviceParser.exe` exit: file I/O plateaus
  at about 2.35 MB (`host_open=178`, `host_read=23172/2350816B`), scheduler
  state remains compact with one finite sleep waiter, and the CPU repeatedly
  stops in iNavi image/resource/geometry code around `image:iNavi.exe+0x2ff*`
  and `+0x332*` with hot `memcpy @1044`. No later `ShowWindow(cmd=5)` is
  reached for the hidden `afxwnd42u` route/chrome controls, and render/display
  milestones are still `none`.
- Complex GDI clip regions now paint through their full CE region rect list
  instead of the old bounding-box shortcut. `FillRect`, memory/display
  `Polygon`, `Polyline`, `BitBlt`, `StretchBlt`, and `TransparentImage` now
  intersect drawing with every selected `HRGN` rectangle, preserving holes from
  `CombineRgn(RGN_DIFF)` on selected memory DIBs and framebuffer HDCs. Source
  anchors remain CE `wingdi.h` region status/clipping semantics plus the
  existing GWE region ownership refs in `SOURCE_REFERENCES.md`. Focused
  coverage `coredll_raw_fill_rect_respects_complex_clip_holes_on_memory_dib`
  passes, and the full `cargo test --features unicorn,trace,win32-desktop`
  suite passes. Mounted validation `target\gdi_clip_regions_virtual.*` used
  dumped runtime DLLs from `D:\INAVI_Emulator\DUMPPLZ\Windows`, reached the
  same post-map `GetMessageW` frontier, stayed bounded
  (`heap_live=14628/31597954B`, `virtual_live=2/131072B`,
  `host_open=910`, `host_read=83967/6451484B`, `mem_open=4`,
  `max_read=685080`), and wrote a populated 800x480 map PNG. This closes a
  real generic clipping bug but does not fully solve the remaining visual
  styling complaint: roads/building blocks still need more CE GDI evidence.
- The black iNavi map base-layer gap was traced to a generic CE GDI omission:
  `ExtTextOutW` with `ETO_OPAQUE` validated arguments but did not fill the
  supplied opaque rectangle with the DC background color. CE defines
  `ETO_OPAQUE` in
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\wingdi.h`, and old GDI/MFC code uses it
  as a rectangle fill primitive. Raw `ExtTextOutW` now fills selected memory
  DIBs or display HDCs through the existing CE GDI fill path before returning
  success. Focused coverage
  `coredll_raw_ext_text_out_opaque_fills_selected_memory_dib_with_bk_color`
  passes with the full raw GWE test binary. Mounted validation
  `target\gdi_exttext_virtual.*` uses dumped runtime DLLs from
  `D:\INAVI_Emulator\DUMPPLZ\Windows` and replaces the black map void with a
  real light land/background layer: pure-black pixels in the map crop dropped
  from `47.2826%` in `target\gdi_rop2_virtual.png` to `0.0131%`, and center
  crop black dropped from `51.6434%` to `0.0000%`. Memory/file counters remain
  bounded (`heap_live=14630/31597994B`, `virtual_live=2/131072B`,
  `host_open=910`, `host_read=83967/6451484B`, `mem_open=4`,
  `max_read=685080`). The run used the full 150 s wall budget inside
  `iNavi.exe+0x13e164` while continuing real map GDI work, so the remaining
  visible fidelity work is road/building styling and longer post-map
  scheduler/device progression, not a black framebuffer/base-layer failure.
- GDI pen ROP2 state is now modeled for the raw COREDLL boundary. `SetROP2`
  and `GetROP2` preserve DC state with CE `R2_*` values from
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\wingdi.h`, `GetROP2` is present in the
  COREDLL export table, and framebuffer/memory-DIB line drawing applies the
  selected ROP2 operation to destination pixels. CE GPE corroboration came
  from `C:\WINCE600\PUBLIC\COMMON\OAK\DRIVERS\DISPLAY\GPE\ddi_if.cpp`,
  `swline.cpp`, and `blthelpers.cpp`. Focused coverage includes
  `coredll_raw_set_get_rop2_round_trips_dc_state` and
  `coredll_raw_polyline_applies_rop2_xorpen_on_selected_memory_dib`. Mounted
  `target\gdi_rop2_virtual.*` was pixel-identical to the prior winding run
  because this iNavi map path did not call `SetROP2`, but the primitive is now
  CE-correct for callers that do.
- Offscreen GDI map drawing now covers the next generic fidelity slice used by
  the mounted iNavi map UI. Pattern brushes created with `CreatePatternBrush`
  tile into selected memory DIBs through `PatBlt`/`Polygon`, `SetBrushOrgEx`
  stores and returns the DC brush origin, and polygon scan conversion now uses
  CE GPE-style winding edge accumulation as corroborated by
  `C:\WINCE600\PUBLIC\COMMON\OAK\DRIVERS\DISPLAY\GPE\ddi_if.cpp` and
  `swfill.cpp`. Focused coverage includes
  `coredll_raw_pat_blt_tiles_pattern_brush_on_selected_memory_dib` and
  `coredll_raw_polygon_uses_winding_fill_for_repeated_edges`; the full
  `coredll_raw_gwe` test binary passes. Mounted validation
  `target\gdi_winding_virtual.*` reaches the same stable post-map
  `GetMessageW` idle frontier with bounded counters
  (`heap_live=14628/31597954B`, `virtual_live=2/131072B`,
  `host_open=910`, `host_read=83969/6465277B`, `mem_open=4`,
  `max_read=685080`). The framebuffer is a real 800x480 iNavi map with
  roads, labels, icons, patterned buildings, and controls; the remaining map
  fidelity issue is not blank startup or file I/O, but the dark/black base map
  layer and road/background styling.
- Removed the remaining hardcoded mounted-iNavi startup code-hook behavior that
  posted a late `WM_INITDIALOG` and wrote the aux touch alias from fixed app PCs.
  Raw dialog creation now has regression coverage proving it does not leave a
  queued `WM_INITDIALOG`; synchronous dialog init through the normal Unicorn/MFC
  path remains the guest-visible behavior. The mounted
  `target\dialog_init_no_replay_virtual_*` probe no longer reproduces the
  old `0x0002c264` null-slot crash: the resource-ready check returns success,
  `resource_59718_lookup_return` returns `v0=1`, and real paints reach
  `TGNaviDlg`, the main child, and visible bottom child `0x00020070`.
- COREDLL ordinal 68 / `wcspbrk` is now implemented in the raw CRT dispatch and
  covered by `coredll_raw_wcspbrk_finds_first_accepted_wide_char`. This removed
  the immediate `COREDLL.dll@68` stop from the post-dialog mounted run. Release
  validation `target\wcspbrk_virtual_*` ran to the 90 s wall limit without an
  unmapped crash, with bounded memory/file I/O
  (`heap_live=15101/32492068B`, `virtual_live=2/131072B`,
  `host_open=908`, `host_read=83967/6465215B`, `mem_open=4`,
  `max_read=685080`). The app progressed into later resource messages,
  visible child `0x00020070` paint, repeated `WM_TIMER`, and real map/search DB
  activity, including `around.db` reads at ~51-52 MB.
- Longer release validation `target\wcspbrk_long_virtual_*` supersedes the
  short `wcspbrk_virtual` wall-stop suspicion. With lighter trace output, the
  map/search DB phase completes naturally and the run parks at scheduler-owned
  `GetMessageW` (`pc=0x7fff0b60(ce-import-traps+0xb60)`,
  `ra=0x60024834(dll:mfcce400.dll+0x24834)`,
  `blocked_get_message=thread:1 hwnd=any`) rather than crashing or spinning in
  the DB loop. Memory remains bounded (`heap_live=14628/31597954B`,
  `virtual_live=2/131072B`, `host_open=912`,
  `host_read=83971/6465339B`, `mem_open=4`, `max_read=685080`). The final
  framebuffer dump is a real iNavi map UI, and the render trace shows guest GDI
  presenting `BitBlt` to display HDC `0x02020004` for HWND `0x00020004`
  (`800x480`) from composed memory DC `0x000a7be8`, plus a later
  `64x62` overlay blit at `732,114`. Current frontier is now post-map idle:
  periodic `WM_TIMER` id `4565`, custom messages such as `0x52e8`/`0x5284`,
  COM7 GPS polling, and scheduler/device/message wakes. The render milestone
  detector still reports `inavi display/controller: none`, so that detector is
  stale for this real GDI present path.
- Added stateful CE serial control data beneath device handles, using
  `C:\WINCE600\PRIVATE\WINCEOS\DRIVERS\SERDEV\serial.c` and the CE 4.2 Mipsii
  `winbase.h` DCB/COMSTAT layout as source evidence. `GetCommState` and
  `SetCommState` now round-trip the 28-byte DCB, `SetCommMask` and
  `GetCommMask` preserve the event mask, `ClearCommError` reports modeled RX
  and TX queue depths, and `PurgeComm(PURGE_RXCLEAR|PURGE_TXCLEAR)` clears both
  device-session and remote-injected serial bytes. `WaitCommEvent` now reports
  `EV_RXCHAR` immediately when the mask includes it and RX is already ready.
  Focused coverage:
  `serial_comm_state_mask_and_purge_are_handle_state` and
  `coredll_raw_comm_state_mask_wait_and_purge_are_stateful`.
- `WaitCommEvent` now has a scheduler-owned blocking path for real Unicorn
  imports instead of only the raw immediate `EV_RXCHAR` helper behavior. CE
  source evidence is
  `C:\WINCE600\PRIVATE\WINCEOS\DRIVERS\SERDEV\serial.c`, where
  `WaitCommEvent` routes to `IOCTL_SERIAL_WAIT_ON_MASK` and the documentation
  notes that changing the event mask wakes a pending wait with event `0`.
  v3 now registers `SerialCommEvent` waiters separately from serial
  `ReadFile` waiters, wakes them on remote/injected RX when the current comm mask
  includes `EV_RXCHAR`, and marks pending waits so `SetCommMask` resumes them
  with a zero event. Compact and trace summaries now include serial
  comm-event counters. Focused coverage includes
  `scheduler_queues_serial_comm_event_waiters_by_handle`,
  `remote_serial_injection_queues_scheduler_comm_event_candidates`,
  `set_comm_mask_wakes_pending_comm_event_with_zero_event`, and Unicorn resume
  tests for writing both `EV_RXCHAR` and zero into guest memory. Mounted
  no-regression probe `target\comm_event_virtual.*` used dumped runtime DLLs
  from `D:\INAVI_Emulator\DUMPPLZ\Windows`, reached the known
  `COREDLL.dll@861 blocked_get_message` frontier with bounded counters
  (`heap_live=14631/31599400B`, `virtual_live=2/131072B`,
  `host_open=906`, `host_read=82688/5859512B`, `mem_open=4`,
  `max_read=685080`), and showed the 60 s mounted path called `SetCommMask`
  once but did not enter a pending `WaitCommEvent` before stopping
  (`serevsig=0`, `serevcand=0` in the new summary fields).
- Mounted validation after the serial-control state slice wrote
  `target\comm_state_virtual_*`. The run still reaches the real rendered map
  screen and parks at scheduler-owned `GetMessageW` with bounded memory/file
  counters (`heap_live=14628/31597954B`, `virtual_live=2/131072B`,
  `host_open=912`, `host_read=83971/6465339B`, `mem_open=4`,
  `max_read=685080`). This confirms the comm-state change does not regress UI
  progress; current frontier remains the rendered-map idle path.
- Unicorn startup scheduling now avoids two recent mounted blockers without
  changing guest-visible API results. Guest WNDPROC callouts reserve/restore a
  small stack frame and defer blocked-wait/get-message resumes until the
  WNDPROC return stub completes, fixing the release/no-trace `FETCH_PROT` at
  `0x000052e8(user-kdata+0x2e8)`. `PulseEvent` now releases only waiters that
  were registered at pulse time and records a pulse token so the resumed waiter
  still gets `WAIT_OBJECT_0` after the event has been reset. `Sleep` now uses
  the same scheduler handoff as `WaitForSingleObject` when another blocked
  waiter is ready, and accelerated finite current-thread waits have a tiny
  host throttle so guest timeout polling no longer burns millions of import
  traps per minute. The Unicorn bridge also removes stale saved waits for a
  thread before that same thread registers a new `Sleep`/`WaitForSingleObject`
  wait, matching CE's one-active-wait-per-running-thread shape and preventing
  duplicate saved main-thread contexts. Focused coverage includes
  `pulse_event_releases_registered_waiter_after_reset` and
  `current_sleep_yields_to_ready_blocked_waiter` plus
  `stale_blocked_wait_cleanup_removes_prior_context_for_thread`; the full
  Unicorn scheduler test module passes. Mounted validation with dumped runtime
  DLLs wrote `target\unicorn_wait_cleanup_virtual_60s.*`: it ran to the 60 s
  wall limit inside guest image code
  (`pc=0x00953bec(image:iNavi.exe+0x943bec)`), stayed memory/file-I/O bounded
  (`heap_live=14677/32567956B`, `virtual_live=2/131072B`,
  `host_open=906`, `host_read=82796/6057032B`), and active blocked waits are
  now only the COM serial read plus one main-thread `Sleep(101)`. Scheduler
  counters are throttled (`block:43895`, `wake:21947`) instead of the prior
  multi-million wait/sleep storm, and the framebuffer dump is populated. The
  visible window set now includes front top-level `TGNaviDlg` (`0x00020080`)
  with updates clear, but the render trace still has no iNavi
  display/controller/milestone entries, so the active UI frontier remains
  generic GWE/GDI/resource presentation after startup scheduling is sane.
- Unicorn raw-import blocking for `MsgWaitForMultipleObjectsEx` now has a
  current-thread timer/timeout bridge matching the conservative `GetMessageW`
  wait shape. After validating handles and finding no immediate queue input, a
  running guest thread can complete the syscall immediately when the next CE
  timer is already due or due within the short timer fast-forward cap, can
  sleep to a longer timer if it fits the requested timeout and host wall-clock
  budget, and can return `WAIT_TIMEOUT` for bounded waits that expire before
  any timer. The helper advances the virtual timer clock, pumps GWE, clears
  changed queue input when `MWMO_INPUTAVAILABLE` is not set, writes either
  `WAIT_OBJECT_0 + nCount` or `WAIT_TIMEOUT` back to `v0`, and resumes at the
  import return PC without registering a blocked waiter. Waits that cannot fit
  the host run budget still become scheduler-owned blocked waits. Focused
  coverage now includes `current_msg_wait_long_timer_writes_message_wait_result`
  and `current_msg_wait_timeout_writes_wait_timeout_result`, which assert the
  MIPS `v0`/`pc`/`ra` return registers against a real Unicorn instance. The
  raw timer-wake regression still passes after the Unicorn bridge change.
  Mounted release sanity probe `target\unicorn_msgwait_timer_virtual_45s.ppm`
  remains at the same stable `COREDLL.dll@861 blocked_get_message` frontier
  with bounded counters, so the change is a fidelity slice and not a visible
  UI breakthrough by itself.
- Raw `MsgWaitForMultipleObjectsEx` now wakes for CE timers that become due
  inside the requested timeout instead of immediately returning
  `WAIT_TIMEOUT` whenever no handle/input is ready at call entry. The raw
  COREDLL path checks immediate input, pumps already-due timers, and then, for
  nonzero waits, advances the virtual timer clock to the next timer only when
  that due time fits within the requested timeout (or `INFINITE`) before
  pumping GWE and returning the message wait slot. Timers beyond the timeout
  remain pending and the wait returns `WAIT_TIMEOUT`. Focused coverage
  `coredll_raw_msgwait_wakes_for_timer_due_inside_timeout` covers both the
  inside-timeout wake and beyond-timeout guard. Reference constants/result
  shape were checked against `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winuser.h`
  (`MsgWaitForMultipleObjectsEx`, `MWMO_INPUTAVAILABLE`, `QS_TIMER`) and
  `winbase.h` (`WAIT_OBJECT_0`, `INFINITE`). Verified with
  `cargo test --features unicorn,trace,win32-desktop --test
  coredll_raw_kernel`, the CE flag regression in `coredll_raw_gwe`, and
  `cargo check --features unicorn,trace,win32-desktop`; the full feature test
  suite also passes. Mounted release validation
  `target\msgwait_timer_virtual_45s.ppm` stays at the same stable
  `COREDLL.dll@861 blocked_get_message` frontier with bounded counters, so
  this is scheduler wait fidelity, not the post-splash UI breakthrough.
- Host-mode parked `GetMessageW` now has an input wake bridge instead of only
  polling host input outside the blocked CE wait. When the Win32 presenter
  run loop observes that Unicorn is stopped on a blocked raw
  `COREDLL!GetMessageW`, newly polled host input is enqueued through the
  existing remote-input path and immediately drained into the blocked
  thread/window's GWE queue, which queues the normal scheduler message-wake
  candidate. The raw syscall still resumes through Unicorn/COREDLL rather
  than `main.rs` faking a return. Focused coverage
  `blocked_get_message_wait_wakes_when_remote_input_is_drained` proves a
  registered `GetMessage` waiter becomes selectable after remote touch input
  is drained. `cargo check --features unicorn,trace,win32-desktop` and the
  full `cargo test --features unicorn,trace,win32-desktop --test
  basic_subsystems` pass with only the known Windows incremental-finalize
  warning. Mounted release validation
  `target\blocked_input_bridge_virtual_45s.ppm` still reaches the same stable
  `COREDLL.dll@861 blocked_get_message` frontier quickly, with bounded
  counters (`heap_live=13705/30071199B`, `virtual_live=2/131072B`,
  `host_open=665`, `host_read=80129/4055867B`, `mem_open=3`,
  `max_read=685080`), so this is a host interactivity/wake fix rather than
  the post-splash UI breakthrough.
- Win32 host presentation now updates while Unicorn is still running and uses
  an 800x480 client area instead of treating 800x480 as the outer window
  rectangle. The host desktop runtime wraps the framebuffer only for
  `--desktop host`: guest `mark_dirty` calls schedule throttled live blits,
  the Unicorn code hook pumps host window messages during long CPU runs, and
  `WM_PAINT` redraws the last presented BGRA frame instead of letting Windows
  erase the client area back to black. The Win32 presenter now sizes the outer
  HWND with `AdjustWindowRectEx`, paints with `StretchDIBits` into the actual
  client rectangle, sets the window title to the actual host image path, and
  best-effort extracts big/small taskbar icons from the launched host PE path
  with `PrivateExtractIconsW`. Mounted validation wrote
  `target\host_title_icon_client_30s_*`; it stayed in the normal 30 s startup
  band (`host_open=157`, `host_read=25196/1936212B`) and still reached the
  real guest display present
  `BitBlt(dst=0x02020008,dst_hwnd=0x00020008,dst_memdc=false,800x480)`.
  A follow-up `target\host_interactive_close_300s_*` launch stayed responsive
  long enough for manual touch/close testing, and closing the host window now
  terminates the emulator process immediately via the Win32 window proc.
- Preserving complex GDI/window regions moved the mounted iNavi path past the
  previous post-splash idle frontier, and widening guest worker-stack reserve
  removed the follow-up thread prologue crash. `CombineRgn(RGN_DIFF)` now
  keeps multi-rect region holes instead of collapsing to a bounding box;
  `PtInRegion`, `RectInRegion`, `GetRgnBox`, `SelectClipRgn`,
  `SetWindowRgn`, and `GetWindowRgn` consume the rect-list-backed region
  state. `SetWindowRgn(hwnd, hrgn, redraw)` also honors the redraw flag by
  invalidating the target only when requested. Focused coverage
  `coredll_raw_set_window_rgn_honors_redraw_flag` and
  `coredll_raw_combine_rgn_diff_preserves_holes` passes in the raw GWE suite.
  Mounted validation with dumped runtime DLLs wrote
  `target\window_region_complex_virtual_150s_*`; it no longer stopped at the
  old `GetMessageW` idle frontier and instead exposed a worker stack
  `WRITE_UNMAPPED` at `pc=0x000e6cd4`. The follow-up stack fix grows the
  guest stack reserve to 4 MiB, keeps 128 KiB per worker-thread slot, and adds
  focused `guest_thread_stack_tests` coverage for the eighth worker slot. The
  mounted `target\thread_stack_region_virtual_150s_*` probe now runs the full
  150 s wall-clock budget without crashing, stays memory/file-I/O bounded
  (`heap_live=14200/31768040B`, `virtual_live=2/131072B`,
  `host_open=883`, `host_read=79768/5231945B`, `mem_open=4`,
  `max_read=685080`), and reaches substantially more real guest work:
  `CreateThread=10`, `ResumeThread=10`, `WaitForMultipleObjects=10`,
  `BitBlt=103`, `Polygon=1023`, `Polyline=415`, `CreateDIBSection=385`,
  plus first audio, Winsock, and serial/COM import activity. The framebuffer
  still shows the real iNavi SE splash/art frame, while the render trace tail
  shows later map/UI composition into memory DCs and map point/icon files
  being read from `SDMMC Disk\mapdata\point\...`. The active frontier is now
  why that later offscreen map/UI composition is not presented to a display
  HDC, not the old hidden-child stale-paint path, region flattening, worker
  stack fault, or file/RSS bottleneck.
- Child process launch fidelity advanced on the mounted iNavi path. Rooted CE
  `CreateProcessW` application names now resolve through the mount table before
  falling back to parent-relative lookup, so
  `\SDMMC Disk\INavi\res\DeviceParser.exe` resolves to
  `D:\INAVI_Emulator\INAVI\INavi\res\DeviceParser.exe` instead of failing as
  not found. Child PE/DLL loading now reserves the trampoline-expanded main
  image range before placing imported DLLs, fixing the
  `AuthLibrary.dll overlaps pe-image` failure seen when launching
  `happyway_win.exe`. Child runs also start under their own CE thread id, and
  windows owned by that child process/thread are destroyed at child exit so the
  parent no longer dispatches into a stale child WNDPROC after the child
  returns. Mounted validation wrote `target\process_lifetime_virtual_150s_*`:
  `DeviceParser.exe`, `happyway_win.exe`, and `iSearch.exe` all resolved and
  returned exit code `0`; the previous `pc=0x00000000` stale-WNDPROC crash did
  not reproduce. Memory/file I/O stayed bounded
  (`heap_live=13705/30071199B`, `virtual_live=2/131072B`,
  `host_open=665`, `host_read=80127/4046053B`, `mem_open=3`,
  `max_read=685080`). This is real launch/process progress, not complete UI:
  the run still parks at `COREDLL.dll@861 blocked_get_message`, the
  framebuffer remains the real iNavi splash/art frame, and the later hidden
  child update frontier shifted to HWND `0x00020070` after the child process
  work created and cleaned up additional windows.
- File mappings now track explicit mapped views instead of one reusable
  `view_base` per mapping object. `MapViewOfFile` allocates a distinct virtual
  allocation for each view, honors the requested offset/remaining size, seeds
  the view from pagefile or file backing, and records `FileMappingView`
  lifetime. `FlushViewOfFile` copies guest bytes into the shared mapping
  backing, writes through to file-backed mappings at the view offset, and
  best-effort refreshes sibling views after a flush. `UnmapViewOfFile` now
  removes the view and releases the CE virtual allocation instead of only
  returning success. Unicorn child-process mapping sync now loops all live
  views. Focused coverage:
  `cargo test --features unicorn,trace,win32-desktop --test
  coredll_raw_memory_file
  coredll_raw_file_mapping_multiple_views_share_flushed_backing`; the full raw
  memory/file test binary and `cargo check --features
  unicorn,trace,win32-desktop` pass with the known non-fatal Windows
  incremental-finalize warning. Mounted validation wrote
  `target\mapping_views_virtual_150s_*`; it remains memory/file-I/O stable
  (`heap_live=13694/13292926B`, `host_open=665`,
  `host_read=80129/4055867B`, `mem_open=3`) and confirms the real
  `UnmapViewOfFile` call now drops `virtual_live` to `2/131072B`. This is a CE
  fidelity fix, not the post-splash UI breakthrough: the run still parks at
  `COREDLL.dll@861 blocked_get_message` with hidden child HWND `0x0002006c`
  holding the later `800x54` update.
- GWE top-level creation now puts newly-created top-level windows at the front
  of the z-order instead of behind older overlapping top-levels. Child window
  append order is unchanged. This makes `WindowFromPoint`/remote touch choose
  the visible newer popup in the same way CE/MFC expects from top-level
  activation order, and raw `GW_HWNDFIRST` now reports that newest top-level
  before older siblings. Remote `WM_LBUTTONDOWN` delivery now also activates
  and focuses the hit window before queuing the mouse message, which gives MFC
  the normal `WM_KILLFOCUS`/`WM_ACTIVATE`/`WM_SETFOCUS` transition instead of
  leaving focus on an older overlapped window. Focused coverage:
  `cargo test --features unicorn,trace,win32-desktop ce::gwe::tests`,
  `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems`,
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe`,
  `cargo check --features unicorn,trace,win32-desktop`, and full
  `cargo test --features unicorn,trace,win32-desktop` pass. Mounted validation
  wrote `target\touch_focus_virtual_150s_*`; it confirms the real tap at
  `(400,240)` is delivered to HWND `0x00020008`, queues the focus/activation
  transition, runs the guest WNDPROC through Unicorn, then returns to the known
  post-splash hidden-strip frontier (`0x0002006c` hidden with a pending
  `800x54` update). Memory/file-I/O remains stable
  (`heap_live=13694/13292926B`, `virtual_live=3/196608B`,
  `host_open=665`, `host_read=80127/4046053B`, `mem_open=3`,
  `max_read=685080`). This is not new visible UI yet, but it closes the older
  accidental tap-to-main-window path as an invalid explanation.
- GDI DCs now start with CE stock/default selections instead of all-zero
  selected-object slots. `DcState` seeds a default bitmap handle, `SYSTEM_FONT`,
  `WHITE_BRUSH`, `BLACK_PEN`, and `DEFAULT_PALETTE`; `SelectObject` returns
  those previous handles for the common save/restore drawing pattern, and
  selecting the default bitmap back restores the no-user-bitmap memory-DC
  state. Stock-object classification now treats CE `GetStockObject(15)` as
  `DEFAULT_PALETTE` instead of a desktop-style font index. This follows
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\wingdi.h`. Focused coverage:
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_select_object_returns_restorable_dc_defaults`, and the full raw
  GWE/GDI test binary now pass with the known non-fatal Windows
  incremental-finalize warning. Full `cargo check --features
  unicorn,trace,win32-desktop` and full
  `cargo test --features unicorn,trace,win32-desktop` pass. Mounted
  validation wrote `target\gdi_stock_defaults_virtual_150s_*`; it confirms the
  real iNavi path now returns stock/default previous objects in trace
  (`previous=0x000b5080` for memory-DC bitmaps and `previous=0x000b5007` for
  the stock black pen) and restores selected bitmaps back to the default state.
  The run remains memory/file-I/O stable
  (`heap_live=13697/13300954B`, `virtual_live=3/196608B`,
  `host_open=665`, `host_read=80196/4047089B`, `mem_open=3`) and keeps the
  real splash framebuffer populated (`1151198` nonzero RGB bytes). This did
  not advance the post-splash frontier: later work still composes the 800x54
  strip into a memory DC, invalidates hidden HWND `0x0002006c`, and parks at
  `COREDLL.dll@861 blocked_get_message` with no later display-HDC blit.
- GWE invalidation/show/create/set-window-position now only marks changed
  `QS_PAINT` when the target HWND is effectively visible. Hidden or
  ancestor-hidden windows can still keep a simplified pending update rectangle
  for later presentation, but they no longer wake `MsgWaitForMultipleObjectsEx`
  or `GetQueueStatus` as new paint input when `GetMessageW` cannot synthesize a
  `WM_PAINT`. This follows CE's `m_hrgnUpdate == Invalid /\ Visible` model in
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\window.hpp` and paint-request
  queueing shape in `cmsgque.h`. Focused coverage:
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_hidden_invalidated_child_does_not_signal_paint_until_visible`
  passes with the known non-fatal Windows incremental-finalize warning.
- Filtered synthetic paint selection now observes effective ancestor
  visibility too. `PeekMessageW(hwnd, WM_PAINT, WM_PAINT, ...)` no longer
  exposes `WM_PAINT` for a child hidden by its parent while unfiltered
  `GetMessageW` correctly suppresses it. Focused coverage:
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_update_window_observes_hidden_ancestors`. The mounted
  `target\filtered_paint_visibility_virtual_150s_*` follow-up remains at the
  same `COREDLL.dll@861 blocked_get_message` frontier with stable counters and
  the reduced `msgsig=174` hidden-size/move state, confirming this paint
  selection fix did not regress the first real splash present or file/RSS
  behavior.
- GWE/kernel window-position delivery now models CE's pending size/move
  behavior for direct-hidden windows. Hidden `MoveWindow`/`SetWindowPos`
  changes still queue `WM_WINDOWPOSCHANGED` with the CE `WINDOWPOS` payload,
  but `WM_MOVE` and `WM_SIZE` are deferred in per-HWND pending bits and flushed
  on the next direct `ShowWindow`. Focused coverage:
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_hidden_move_defers_size_move_until_show_window` and
  `coredll_raw_window_state_changes_queue_lifecycle_messages` pass with the
  known non-fatal Windows incremental-finalize warning. The mounted
  `target\hidden_sizemove_virtual_150s_*` probe remains at the same
  `COREDLL.dll@861 blocked_get_message` frontier, with stable RSS/file-I/O
  (`heap_live=13697/13300954B`, `virtual_live=3/196608B`, `host_open=665`,
  `host_read=80196/4047089B`), but message input signals dropped from the
  previous `227` to `174`. The `0x0002006c` strip child now receives the later
  hidden geometry update as `WM_WINDOWPOSCHANGED` only; its old hidden
  `WM_MOVE`/`WM_SIZE` traffic is gone. Window snapshots now expose
  `pending_move`/`pending_size` for future mounted trace comparison.
- Raw/kernel `ShowWindow` now treats `WM_SHOWWINDOW` as a direct HWND
  visibility transition instead of comparing effective ancestor visibility to
  the requested state. Hiding a child that is already effectively invisible
  because its parent is hidden still queues the child's own `WM_SHOWWINDOW`,
  and direct show/hide transitions now also queue `WM_WINDOWPOSCHANGED` with a
  CE `WINDOWPOS` payload using no-move/no-size/no-zorder show/hide flags.
  Focused coverage:
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_show_window_queues_direct_visibility_windowpos_under_hidden_parent`,
  `coredll_raw_window_state_changes_queue_lifecycle_messages`,
  `coredll_raw_focus_and_activation_queue_ce_messages`, and the full raw GWE
  binary pass. `cargo fmt`, `cargo check --features unicorn,trace,win32-desktop`,
  `cargo build --features unicorn,trace,win32-desktop`, and full
  `cargo test --features unicorn,trace,win32-desktop` pass with the known
  non-fatal Windows incremental-finalize warning. A mounted 150 s virtual/tap
  probe wrote `target\showwindow_direct_visibility_virtual_150s_*`; it confirms
  the real app path now emits decoded hide `WINDOWPOS` records such as
  `0x0002006c/flags=0x00000097`, but the run still stops at the same
  `COREDLL.dll@861 blocked_get_message` frontier with stable memory/file-I/O
  counters and no later display-HDC present.
- Message trace records now decode queued `WM_WINDOWPOSCHANGED` `WINDOWPOS`
  payloads from guest memory. This is diagnostic only: it exposes `hwnd`,
  `hwndInsertAfter`, `x/y/cx/cy`, and `flags` in monitor message snapshots
  without changing GWE behavior. Verification:
  `cargo fmt`, `cargo check --features unicorn,trace,win32-desktop`,
  focused
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_windowposchanged_carries_guest_windowpos_payload`,
  `cargo build --features unicorn,trace,win32-desktop`, and full
  `cargo test --features unicorn,trace,win32-desktop` pass with the known
  non-fatal Windows incremental-finalize warning. A mounted 150 s virtual
  probe wrote `target\windowpos_trace_decode_virtual_150s_*`; it reaches the
  same stable `COREDLL.dll@861 blocked_get_message` frontier
  (`heap_live=13697/13300954B`, `virtual_live=3/196608B`,
  `host_open=665`, `host_read=80196/4047089B`, `mem_open=3`,
  `max_read=685080`) but the message log now shows decoded `WINDOWPOS`
  details, including HWND `0x0002006c` receiving
  `rect=0,0,800,480/flags=0x00000000`.
- Raw/kernel `SetWindowPos` now queues `WM_WINDOWPOSCHANGED` with a
  `WINDOWPOS` payload for CE-visible metadata changes even when the rectangle
  is unchanged. Show-only, hide-only, and z-order-only calls no longer vanish
  just because `x/y/cx/cy` stayed the same; `WM_MOVE`/`WM_SIZE` are still only
  queued for real geometry deltas. Focused coverage:
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_set_window_pos_show_hide_queues_windowpos_without_rect_change`,
  the full raw GWE binary, `cargo check --features unicorn,trace,win32-desktop`,
  `cargo build --features unicorn,trace,win32-desktop`, and full
  `cargo test --features unicorn,trace,win32-desktop` pass with the known
  non-fatal Windows incremental-finalize warning. A mounted 150 s virtual/tap
  probe wrote `target\setwindowpos_showhide_virtual_150s_*`; it now shows the
  extra `WM_WINDOWPOSCHANGED` traffic in the window/message frontier, but the
  app still stops at `COREDLL.dll@861 blocked_get_message`, remains
  memory/file-I/O stable (`heap_live=13697/13300954B`,
  `virtual_live=3/196608B`, `host_open=665`,
  `host_read=80198/4056903B`, `mem_open=3`, `max_read=685080`), leaves HWND
  `0x0002006c` hidden with a pending 800x54 update, and reports no iNavi render
  milestones or later display-HDC blit.
- Direct `UpdateWindow` now observes effective ancestor visibility instead of
  only the target HWND's direct visible bit. This keeps CE/MFC paint forcing
  generic: a child under a hidden parent keeps its pending update region for a
  later visible state, and v3 does not manufacture a paint into an effectively
  invisible subtree. Focused coverage:
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_update_window_observes_hidden_ancestors`, the full raw GWE test
  binary, `cargo test --features unicorn,trace,win32-desktop ce::gwe::tests`,
  full `cargo test --features unicorn,trace,win32-desktop`, and
  `cargo build --features unicorn,trace,win32-desktop` pass with the known
  non-fatal Windows incremental-finalize warning. A mounted 150 s virtual/tap
  probe wrote `target\update_effective_visibility_virtual_150s_*`; it remains
  memory/file-I/O stable (`heap_live=13697/13300954B`,
  `virtual_live=3/196608B`, `host_open=665`,
  `host_read=80198/4056903B`, `mem_open=3`, `max_read=685080`) and confirms
  the same real frontier: child HWND `0x0002006c` is still effectively hidden,
  app composition continues into offscreen memory DC `0x000a3f38`, and no later
  display-HDC blit or iNavi render milestone appears. This closes forced
  hidden-child painting as a valid path; it does not yet move post-splash UI.
- CE/GWE timers are now scoped by owner thread/message queue plus optional
  `HWND` plus timer id instead of by id globally. This follows
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\cmsgque.h`, where
  `TimerEntry_t` carries `m_pmsgqOwner`, `m_hwnd`, and `m_idTimer`, and
  `TimerQueuesRemoveSingleEvent(HWND,id,MsgQueue*)` removes one scoped timer.
  v3 can keep duplicate numeric timer ids alive for different windows or a
  window timer plus a no-HWND thread timer; raw `KillTimer(hwnd,id)` now routes
  through the real COREDLL ordinal path and only removes the matching scoped
  timer. Destroying a window now also removes timers for that HWND subtree
  while preserving no-HWND owner-thread timers, matching the same header's
  `TimerQueuesRemoveAllMsgQueueOrHwnd` /
  `TimerQueueWindowDestroyedNotification` cleanup shape. Callback timer
  entries now propagate the stored `TIMERPROC` through `MSG.lParam`, and the
  Unicorn `DispatchMessageW` bridge enters that guest callback with CE/Win32
  timer-proc arguments instead of dispatching through the window WNDPROC. The
  CE-internal callback-timer path that bypasses the normal message queue is
  still future work if traces demand it.
  Focused coverage:
  `cargo test --features unicorn,trace,win32-desktop ce::timer`,
  `cargo test --features unicorn,trace,win32-desktop
  window_timers_with_same_id_keep_independent_owners`,
  `cargo test --features unicorn,trace,win32-desktop
  destroy_window_removes_hwnd_timers_but_keeps_thread_timers`, and
  `cargo test --features unicorn,trace,win32-desktop
  coredll_raw_gwe_ordinals_manage_hwnd_rects_points_and_resources`,
  `cargo test --features unicorn,trace,win32-desktop
  coredll_raw_destroy_parent_invalidates_children_and_purges_messages`,
  `cargo test --features unicorn,trace,win32-desktop
  send_message_callout_enters_cross_thread_receiver_context`,
  `cargo test --features unicorn,trace,win32-desktop
  dispatch_message_callout_enters_timerproc_callback`, and
  `cargo test --features unicorn,trace,win32-desktop
  create_window_callout_returns_hwnd_or_null_after_wm_create` pass with
  the known non-fatal Windows incremental-finalize warning. A mounted virtual
  iNavi probe wrote `target\timer_scope_virtual_30s_*`; it still reaches the
  real guest GDI present (`BitBlt` from memory DC `0x000a0044` to window HDC
  `0x02020008`, 800x480), keeps memory/file I/O stable
  (`heap_live=7333/5168631B`, `virtual_live=3/196608B`,
  `host_open=162`, `host_read=26479/1958374B`, `mem_open=2`), and the
  framebuffer dump has `1151398` nonzero RGB bytes out of `1152000`. The
  follow-up mounted probe after destroyed-window timer cleanup wrote
  `target\timer_destroy_virtual_30s_*`; it remains in the same stable real
  present band (`heap_live=7329/5146375B`, `host_open=160`,
  `host_read=25977/1955740B`, framebuffer nonzero `1151398/1152000`) and
  continues RSImage PNG/DIB activity without regressing to a blank screen. The
  TimerProc bridge follow-up wrote `target\timer_callback_virtual_30s_*` and
  likewise stopped only on the 30 s wall limit at `pc=0x0030faec`, stayed
  memory/file-I/O stable (`heap_live=7327/5135247B`, `virtual_live=3/196608B`,
  `host_open=159`, `host_read=25713/1949108B`, `mem_open=2`), retained the
  real screen present
  `BitBlt(dst=0x02020008,dst_memdc=false,dst_hwnd=0x00020008,src=0x000a0044,800x480)`,
  and wrote a populated framebuffer (`1151398` nonzero RGB bytes out of
  `1152000`).
- Long `GetMessageW` timer waits can now mature inside a single live Unicorn
  invocation without rebuilding CPU state. When the current thread parks in an
  empty message queue and the next timer is beyond the <=100 ms fast-forward
  window, the raw COREDLL bridge checks the run's host wall-clock budget,
  waits only if the timer can become due inside that budget, then reuses the
  existing scheduler-owned `GetMessageW` resume path while the saved MIPS
  registers/RAM are still live. This fixes the unsafe outer-runner re-entry
  experiment that reproduced `pc=0x00000000` after a blocked wait. Focused
  coverage: `cargo test --features unicorn,trace,win32-desktop
  long_getmessage_timer_wait_respects_host_wall_budget` and
  `cargo test --features unicorn,trace,win32-desktop
  empty_queue_getmessage_only_fast_forwards_near_due_timers` pass. A mounted
  virtual/tap probe wrote `target\unicorn_realtime_timer_virtual_30s_*`: it
  keeps the real iNavi SE splash/art framebuffer, delivers two real no-HWND
  timer messages (`WM_TIMER` id 1000 at about `21829 ms` and `29329 ms`),
  then parks cleanly at `COREDLL.dll@861 blocked_get_message` because the next
  7.5 s period does not fit the 30 s run budget. Scheduler counters now show
  bounded live timer wake/resume instead of spin or control-flow loss
  (`sched=wait:3/0/3`, `wake=2`, `reg=3/2`, `msgcand=2`), with stable
  memory/file I/O (`heap_live=13697/13300954B`,
  `virtual_live=3/196608B`, `host_open=665`,
  `host_read=80198/4060882B`, `mem_open=3`, `max_read=685080`).
- Raw Unicorn `GetMessageW` empty-queue handling no longer fast-forwards long
  future timers in the current blocked-wait completion path. Both the
  pre-block and just-registered blocked-wait paths now share the same
  <=100 ms fast-forward guard: near-due GUI settling timers can still fire,
  while a 7.5 s CE timer parks as a real blocked message wait instead of
  spinning thousands of synthetic `WM_TIMER` deliveries. Focused coverage:
  `cargo test --features unicorn,trace,win32-desktop
  empty_queue_getmessage_only_fast_forwards_near_due_timers`,
  `cargo test --features unicorn,trace,win32-desktop ce::gwe::tests`, and
  `cargo check --features unicorn,trace,win32-desktop` pass. A mounted virtual
  tap probe wrote `target\timer_cap_startup_tap_virtual_20s_*`: it still
  reaches the real iNavi SE splash/art UI through guest GDI, then stops at
  `COREDLL.dll@861 blocked_get_message` with one registered wait
  (`sched=wait:3/0/1`, `reg:1/0`) and the long no-HWND timer preserved as
  pending (`id=0x3e8`, `hwnd=0`, `msg=0x113`, `due=22086`,
  `period=7500`). The previous startup-tap run without this cap consumed the
  full 60 s wall budget while delivering about 22,924 timer wakes; the same
  path is now bounded (`PeekMessageW=194`, `GetMessageW=190`) and remains
  memory/file-I/O stable (`heap_live=13697/13300954B`,
  `virtual_live=3/196608B`, `host_open=665`,
  `host_read=80198/4056903B`, `mem_open=3`, `max_read=685080`).
- GWE now keeps hidden-window paint/update state coherent during MFC child
  control creation. `ShowWindow(SW_HIDE)` and `SWP_HIDEWINDOW` clear the
  window's own pending update/erase state, while `SetWindowPos` clips any
  surviving pending update rectangle to the current zero-origin client bounds.
  This matches the existing `InvalidateRect` clipping model and CE GWE's
  paint-request-as-window-state shape from
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\cmsgque.h`/
  `window.hpp`, without painting hidden children or special-casing iNavi.
  Focused coverage: `cargo test --features unicorn,trace,win32-desktop
  ce::gwe::tests`, raw GWE coverage:
  `cargo test --features unicorn,trace,win32-desktop coredll_raw_gwe`, and the
  full `cargo test --features unicorn,trace,win32-desktop` all pass. A mounted
  virtual probe wrote `target\hide_update_clear_virtual_20s_*`; the bulk
  hidden `AfxWnd42u` controls now show `upd=false`/`erase=false` instead of
  carrying stale full-screen dirty rectangles from visible zero-size creation,
  while a later legitimately resized/invalidated hidden child is clipped to
  `update=0,0-100,135`. The run remains memory/file-I/O stable
  (`heap_live=7286/4855918B`, `virtual_live=3/196608B`,
  `host_open=142`, `host_read=22508/1829377B`, `mem_open=2`,
  `max_read=497178`) and keeps the real populated framebuffer
  (`1151398` nonzero RGB bytes). This is GWE state fidelity progress; it does
  not yet solve the post-splash resource/UI progression loop.
- Main-thread `GetMessageW` waits can now resume from scheduler-owned timer
  expiry instead of stopping as diagnostic-only waits. The Unicorn bridge
  gives the initial guest thread a CE current-thread pseudo-handle wait
  identity, records blocked `GetMessageW` as a scheduler message wait, pumps
  already-due or near-due timers into the owning GWE queue, selects the ready
  waiter, writes the `MSG`, removes the waiter, and returns through the saved
  MIPS syscall return site. Long future timers now remain parked rather than
  being advanced immediately. Focused coverage:
  `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_get_message_prioritizes_paint_over_generated_timer`, and
  `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems
  get_message_waiter_uses_filtered_scheduler_message_readiness` pass with the
  known non-fatal Windows incremental-finalize warning. A mounted virtual probe
  wrote `target\main_getmessage_timer_resume_virtual_*` and now shows real
  scheduler wait ownership for the long id-1000 timer loop
  (`block=4221`, `wake=4214`, `reg=4214/4214`, `msgcand=4214`) instead of the
  previous `reg=0` blocked diagnostic stop. The run still reaches the 20 s
  wall-clock limit in the MFC idle-update/resource loop, with stable memory and
  file I/O (`heap_live=13697/13300954B`, `virtual_live=3/196608B`,
  `host_open=665`, `host_read=80198/4060882B`, `mem_open=3`,
  `max_read=685080`) and the same populated framebuffer (`575800` nonzero
  pixels). This is scheduler fidelity progress, not new post-splash UI.
- Mounted iNavi now reaches real guest-driven UI presentation in virtual
  desktop mode. The `UpdateWindow` guest trampoline now honors the CE paint
  update shape by entering the guest WNDPROC with `WM_ERASEBKGND` when
  `erase_pending` is set, preserving the update region for the follow-up
  `WM_PAINT`, and only clearing the erase bit when the erase handler returns
  nonzero. The default/kernel `UpdateWindow` path uses the same
  erase-before-paint ordering for non-guest/default WNDPROCs. Focused
  coverage: `cargo test --features unicorn,trace,win32-desktop --test
  coredll_raw_gwe`; `cargo check --features unicorn,trace,win32-desktop`
  passes with the known non-fatal Windows incremental-finalize warning. A
  fresh mounted virtual probe wrote `target\update_erase_virtual_*` and
  stopped only on the 20 s wall-clock limit, memory-stable
  (`heap_live=13697/13300954B`, `virtual_live=3/196608B`,
  `host_open=665`, `host_read=80198/4060882B`, `mem_open=3`,
  `max_read=685080`). Its presentation trace includes the first confirmed
  memory-DC-to-window-HDC present:
  `BitBlt(dst=0x02020008, dst_memdc=false, dst_hwnd=0x00020008,
  src=0x000a0044, src_memdc=true, 800x480)`, and the framebuffer dump is no
  longer black/sparse (`384000` nonzero pixels). The generated preview
  `target\update_erase_virtual.png` shows the real iNavi SE splash/art frame.
  This is real UI progress from guest GDI paths, not a fabricated host paint.
  The process still runs into the fast id-1000 thread `WM_TIMER`/idle loop
  after presentation, so the next frontier is sustaining post-splash UI and
  scheduler/timer fidelity rather than first pixels.
- Raw `GetMessageW`/`PeekMessageW` now preserves CE/MFC generated-message
  ordering by checking sent, posted, quit, and synthetic paint work before
  generating due timers. Timed-out cross-thread sends still expire before
  retrieval, so the existing send-timeout behavior is preserved while pending
  `WM_PAINT` is no longer starved by a due `WM_TIMER`. Focused coverage:
  `coredll_raw_get_message_prioritizes_paint_over_generated_timer`, and the
  full `coredll_raw_gwe` test binary passes. A fresh mounted virtual probe
  wrote `target\paint_priority_final_virtual_*`; it stayed memory-stable
  (`heap_live=13697/13300954B`, `virtual_live=3/196608B`,
  `host_open=665`, `host_read=80130/4044109B`, `mem_open=3`,
  `max_read=685080`) and now reaches real paint processing
  (`BeginPaint=6`, `EndPaint=6`) plus 800x480 DIB/surface work from a
  screen/window HDC. Useful UI still does not present: final blits remain
  memory-DC-to-memory-DC, the framebuffer remains the sparse tap marker, and
  virtual time still races through the 7.5 s thread timer loop
  (`WM_TIMER`, id `1000`, due around `32189373 ms`). This moved the frontier
  from missing paint to missing memory-DC-to-screen presentation plus timer/
  idle scheduling fidelity.
- Raw `StretchBlt` now uses the correct 11-argument CE ABI instead of the
  `BitBlt` ABI in the real COREDLL syscall path. The trace decoder now reports
  destination and source rectangles separately, and focused coverage
  `coredll_raw_stretchblt_uses_stretch_abi_and_scales_between_memory_dcs`
  passes. A mounted virtual run with dumped runtime DLLs reached a real
  `StretchBlt` call with `dst_rect=20,0,760,54`,
  `src_rect=20,0,10,54`, `rop=0x00cc0020`, and `ok=1`; useful UI still did
  not advance.
- CRT `feof` for CE file handles now follows the stream EOF-indicator shape
  instead of reporting EOF merely when the cursor equals file size. Exact-size
  reads leave `feof` clear, a subsequent failed/short read sets it, and
  `SetFilePointer`/`fseek` clears it. Focused coverage:
  `coredll_raw_stdio_reads_host_backed_files`; the
  `coredll_raw_memory_file` test binary passes. A mounted virtual rerun stayed
  memory-stable but did not move the UI frontier, so this was a correctness fix
  rather than the active render blocker.
- The current iNavi resource/render frontier is now narrowed to the app's
  resource-ready chain, not bulk file I/O. Fresh virtual probes
  `target\resource_table_diag_*`, `target\resource_watch_*`, and
  `target\resource_watch_big_*` show `\SDMMC Disk\INavi\res\values.dat`
  opens and reads successfully (`host_read` remains about 80k reads / 4 MB,
  RSS-stable), but `resource_59718` fails after calling the guest
  resource-table loader for mode `47` while the shared table object is already
  populated (`buffer=0x3006d970`, `tree_root=0x3006e830`,
  `tree_count=215`). Disassembly of `iNavi.exe` around `0x0006bd18` confirms
  that this guest loader returns `0` immediately when the table buffer is
  non-null, and `0x0001ad94` treats that return as failure. The latest
  framebuffer remains only the red tap line (`401` nonzero pixels from
  `(0,160)` to `(400,160)`), and a 120 s diagnostic run stopped by wall clock
  in dumped-runtime code rather than producing UI progress.
- Mounted iNavi startup now preloads real sibling DLLs from the main image
  directory in addition to dumped-runtime `mfcce400.dll`/`commctrl.dll`.
  This is a generic CE loader-search bridge, not a hardcoded auth shortcut:
  COREDLL remains emulator-provided, emulator-provided modules are skipped, and
  sibling DLLs are inspected from the executable directory. A focused test
  covers case-insensitive sibling `.dll` discovery. The first mounted virtual
  probe after this change loaded `AuthLibrary.dll`, `TpSysAuth.dll`,
  `mMbcAuth.dll`, `tpeg_if_dll.dll`, and `tw_tpeg_if_dll.dll`, then reached
  real `AuthLibrary` code and exposed COREDLL `strcat @1063` instead of the old
  null `GetProcAddressW(TpSysCheckSerial)` call. `strcat @1063` is now in the
  checked-in CRT ordinal tranche with focused raw-dispatch coverage. Moving the
  external Unicorn trampoline pool from `0x50000000` to `0x70000000` also
  removed a generic collision with CE virtual allocations starting at
  `0x50000000`; the same mounted virtual run no longer stops with
  `WRITE_PROT` at the first mapped-view page. Latest evidence:
  `target\inavi_trampoline_virtual_*` loaded 7 DLLs, stopped only at the
  30 s wall-clock limit (`pc=0x0030f978`, `ra=0x002fd4cc`), stayed
  memory-stable (`heap_live=7340/21892552B`,
  `virtual_live=3/196608B`, `host_open=161`,
  `host_read=26159/1947356B`, `mem_open=2`, `max_read=497178`), and reached
  repeated RSImage `CreateDIBSection` work. Render milestones remain `none`;
  the framebuffer still contains only the 301-byte red line, so this is loader,
  CRT, and trampoline-memory progress rather than useful UI output.
- Dumped-runtime `explorer.exe` now gets past the earlier host-presenter
  trampoline and missing-ordinal startup blockers when run with
  `D:\INAVI_Emulator\DUMPPLZ\Windows` as the DLL search path. The old
  high-address MIPS trampoline failure (`0xffff832c` from `0x00057108`) no
  longer reproduces, and the follow-up COREDLL frontier ordinals have been
  filled with source-backed shims: `__security_gen_cookie2 @2696`,
  `OpenEventW @1496`, `SHGetSpecialFolderPath @295` with
  `HKLM\System\Explorer\Shell Folders` lookup/fallbacks, `StringCbCatW @1694`,
  `CopyFileW @164`, `StringCchCatW @1693`, `wcsncmp @65`, and
  `DestroyIcon @725`. The latest bounded host-presented probe wrote
  `target\explorer_win32_host_destroyicon_summary.txt`,
  `target\explorer_win32_host_destroyicon_render.txt`, and
  `target\explorer_win32_host_destroyicon_milestones.txt`; it reaches the
  emulator sentinel (`pc=0x7ffffff0`, `ra=0x7ffffff0`, `v0=1`) instead of a
  COREDLL trap. Render milestones remain `none`, so this is explorer launch
  fidelity rather than UI progress. A fresh rerun after the scheduler
  `SendMessage` sender-parking slice wrote `target\explorer_send_wait_*` and
  reproduced the same sentinel result with `sched=wait:0/0/0`, no render
  milestones, and an all-zero framebuffer body. The trace shows
  `LoadLibraryW("aygshell.dll")` returning `ERROR_FILE_NOT_FOUND`; a recursive
  search under `D:\INAVI_Emulator\DUMPPLZ` found no `aygshell.dll`, so this is
  missing dumped runtime content rather than a loader search regression.
- Storage mount configuration now supports a `[root].host_root` backing root.
  If the root value is absent or not an existing directory, v3 falls back to
  `"."`. Mount entries without their own `host_root` inherit
  `<root>\<guest-root-components>`, while explicit mount `host_root` values
  still override the root. The current `mounts.toml` can therefore back
  `\Windows` from the configured root while keeping explicit `\SDMMC Disk` and
  `\ResidentFlash` paths authoritative.
- Dumped-runtime `commctrl.dll` now loads from the configured DLL search path
  instead of failing PE inspection. The loader already preloads
  `commctrl.dll` from `--dll-search-dir`; the missing piece was PE mapped-RVA
  parsing for real CE DLLs whose directory terminators/relocation directory
  point into zero-filled image memory rather than raw file bytes. `PeImage`
  typed/string RVA readers now use mapped-image semantics: raw bytes when
  present, zeroes for any RVA below `SizeOfImage` that is not backed by file
  data, and an error only outside the mapped image. Focused coverage:
  `mapped_rva_reads_zero_filled_section_tail` and
  `mapped_rva_reads_zero_filled_image_gaps`. Verbose mounted loader validation
  using `D:\INAVI_Emulator\DUMPPLZ\Windows` now reports both
  `mfcce400.dll` and `commctrl.dll` loaded (`loader: 2 DLL(s), 1001 import
  trap(s)`). A bounded virtual iNavi probe wrote
  `target\commctrl_virtual_60s_summary.txt`,
  `target\commctrl_virtual_60s_render.txt`,
  `target\commctrl_virtual_60s_milestones.txt`,
  `target\commctrl_virtual_60s_files.txt`, and
  `target\commctrl_virtual_60s.ppm`; it stopped at the 60 s wall limit
  (`pc=0x00135bd4`, `ra=0x00135bc8`) with stable memory
  (`heap_live=6981/21280227B`, `virtual_live=3/196608B`,
  `host_open=112`, `host_read=7840/1760751B`, `mem_open=2`,
  `max_read=497178`). Render milestones are still `none`; the framebuffer is
  effectively black except the 101-pixel red tap marker, so this is runtime
  DLL/loader fidelity rather than UI success. The import patcher now resolves
  loaded external DLL exports before considering emulator shim traps and no
  longer classifies `commctrl.dll`/`commctrlce.dll` as a common-controls shim,
  so search-path `commctrl` calls patch directly to the mapped DLL export
  addresses. Focused coverage:
  `patches_loaded_commctrl_exports_from_external_table_without_stub_trap`,
  `patches_winsock_and_ole_imports_as_supported_traps_without_mfc_or_commctrl_stub`,
  and `coredll_raw_module_apis_resolve_preloaded_search_dll_exports`. A
  follow-up bounded virtual probe with the dumped DLL search path wrote
  `target\commctrl_searchpath_virtual_60s_summary.txt`,
  `target\commctrl_searchpath_virtual_60s_render.txt`,
  `target\commctrl_searchpath_virtual_60s_milestones.txt`,
  `target\commctrl_searchpath_virtual_60s_files.txt`, and
  `target\commctrl_searchpath_virtual_60s.ppm`; it stopped at the 60 s wall
  limit (`pc=0x6000d9b8`, `ra=0x6004fc6c`) with stable memory
  (`heap_live=6927/21256913B`, `host_open=91`,
  `host_read=4302/1718377B`, `mem_open=2`, `max_read=497178`). Render
  milestones remain `none`, and framebuffer stats are still only the 101-pixel
  red tap marker (`nonzero=101`, colors `0,0,0;255,0,0`).
- Existing host-file opens requested as `GENERIC_READ | GENERIC_WRITE` now keep
  the file host-backed even when the host denies write access. v3 first tries a
  read/write host handle for write-through, then falls back to a read-only live
  host handle instead of failing `CreateFileW` or preloading the file into a
  `Vec`; writes on that fallback handle return an unsupported write result.
  Focused coverage:
  `readwrite_existing_readonly_host_files_fall_back_to_read_handle`, plus the
  existing host-backed file streaming tests. Full `cargo check --features
  unicorn,trace,win32-desktop` and
  `cargo test --features unicorn,trace,win32-desktop` pass, with the existing
  non-fatal Windows incremental-finalize warning. A mounted virtual probe using
  `D:\INAVI_Emulator\DUMPPLZ\Windows` wrote
  `target\file_rw_fallback_virtual_60s_summary.txt`,
  `target\file_rw_fallback_virtual_60s_render.txt`,
  `target\file_rw_fallback_virtual_60s_milestones.txt`,
  `target\file_rw_fallback_virtual_60s_files.txt`, and
  `target\file_rw_fallback_virtual_60s.ppm`; it stopped at the 60 s wall limit
  at `pc=0x003426d0`, `ra=0x002fd5e8` with stable memory
  (`heap_live=7482/23071147B`, `virtual_live=3/196608B`) and much deeper file
  activity (`host_open=235`, `host_read=38930/2229372B`, `mem_open=2`,
  `max_read=497178`). The previous `Access is denied` churn for
  `SDMMC Disk\mapdata\SearchDB\*.db` is gone (`0` matching failures), but
  render milestones remain `none` and the framebuffer still contains only red
  tap pixels (`nonzero=301`, color `255,0,0`), so this is startup/file-I/O
  progress rather than useful UI output.
- Scheduler/wait fidelity now has a source-backed priority and waitable-handle
  slice. Parked Unicorn `WaitForSingleObject` resumes now choose the ready
  blocked waiter by CE priority ordering, where lower numeric priorities win,
  and preserve FIFO order within the same priority while skipping the active
  thread. Kernel waits now reject nonwaitable handles such as file, device,
  window, waveOut, file-mapping, find-file, and critical-section handles with
  `WAIT_FAILED`/`ERROR_INVALID_HANDLE` instead of treating them as immediately
  signaled. `WaitForMultipleObjects` sets `ERROR_INVALID_PARAMETER` for empty
  or CE6-unsupported wait-all calls and `ERROR_INVALID_HANDLE` when any handle
  is not waitable. Source anchors are
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\schedule.c`
  `LockWaitableObject`/`DoWaitForObjects` and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\syncobj.c` waiter wake paths.
  Focused coverage: `wait_scheduler_tests`,
  `virtual_win32_api_smoke_covers_file_device_sync_gwe_and_audio`, and
  `coredll_raw_ordinals_execute_kernel_thread_time_and_sync_semantics`.
- Scheduler/thread fidelity now also matches two CE wait/thread contracts from
  `schedule.c` and `thread.c`. `WaitForMultipleObjects(FALSE)` validates every
  handle before acquiring any ready object, so a later invalid handle returns
  `WAIT_FAILED`/`ERROR_INVALID_HANDLE` without consuming an earlier auto-reset
  event; the kernel also enforces CE's `MAXIMUM_WAIT_OBJECTS == 64` limit.
  Raw COREDLL coverage exercises the same path through
  `ORD_WAIT_FOR_MULTIPLE_OBJECTS`. Thread objects now store CE absolute
  priorities (`0..255`) with default Win32-normal priority `251`;
  `SetThreadPriority`/`GetThreadPriority` map the CE Win32 priority band
  `0..7` to/from absolute `248..255`, while `CeSetThreadPriority` and
  `CeGetThreadPriority` use the absolute value. Focused coverage:
  `wait_for_multiple_validates_all_handles_before_acquiring`,
  `coredll_raw_ordinals_execute_kernel_thread_time_and_sync_semantics`, and
  `wait_scheduler_tests`. A fresh mounted virtual probe wrote
  `target\scheduler_priority_wait_virtual_60s_summary.txt`,
  `target\scheduler_priority_wait_virtual_60s_render.txt`,
  `target\scheduler_priority_wait_virtual_60s_milestones.txt`,
  `target\scheduler_priority_wait_virtual_60s_files.txt`, and
  `target\scheduler_priority_wait_virtual_60s.ppm`; it stopped at the 60 s
  wall limit (`pc=0x00a4a1f4`, `ra=0x002017e0`) with stable counters
  (`heap_live=6930/21296145B`, `virtual_live=2/131072B`,
  `host_open=92`, `host_read=4305/1765319B`, `mem_open=2`,
  `max_read=497178`, and
  `sched=wait:1/0/0 ok:1 timeout:0 fail:0 block:0 wake:0`). Render
  milestones are still `none`; framebuffer output remains
  black except the 101-pixel red tap marker, so this is scheduler/thread
  fidelity rather than UI success.
- Scheduler/thread suspend-count handling now follows the CE KCall count
  contract from `schedule.h`, `kcalls.c`, and `thread.c`. Thread suspend
  counts are capped at `MAX_SUSPEND_COUNT == 127`; `SuspendThread` returns the
  previous count, leaves the count unchanged at the cap, and reports
  `ERROR_SIGNAL_REFUSED` with return `0xffffffff` on overflow. `ResumeThread`
  returns the previous count, decrements only when nonzero, and valid
  zero-count resumes return `0` without treating the handle as failed. Focused
  coverage: `suspend_resume_thread_counts_follow_ce_cap` and the raw
  `coredll_raw_ordinals_execute_kernel_thread_time_and_sync_semantics`
  `SuspendThread`/`ResumeThread` path. Full
  `cargo check --features unicorn,trace,win32-desktop` and
  `cargo test --features unicorn,trace,win32-desktop` pass, with the existing
  non-fatal Windows incremental-finalize warning. A fresh mounted virtual
  probe wrote `target\suspend_count_virtual_60s_summary.txt`,
  `target\suspend_count_virtual_60s_render.txt`,
  `target\suspend_count_virtual_60s_milestones.txt`,
  `target\suspend_count_virtual_60s_files.txt`, and
  `target\suspend_count_virtual_60s.ppm`; it stopped at the 60 s wall limit
  (`pc=0x6000cee4`, `ra=0x6000d06c`) with stable counters
  (`heap_live=6921/21255717B`, `host_open=91`,
  `host_read=4304/1728191B`, `mem_open=2`, `max_read=497178`, and
  `sched=wait:1/0/0 ok:1 timeout:0 fail:0 block:0 wake:0`). Render
  milestones are still `none`; framebuffer output remains black except the
  101-pixel red tap marker, so this does not count as UI progress.
- Scheduler wait parking now covers the first `WaitForMultipleObjects(FALSE)`
  Unicorn bridge instead of only `WaitForSingleObject`. A blocked
  `WaitForMultipleObjects` import with a nonzero timeout now records the full
  handle list, parks only after all handles validate and none are ready, wakes
  by CE priority/FIFO selection when any handle becomes ready, returns
  `WAIT_OBJECT_0 + index` through the raw import boundary, and preserves
  object-signaled acquisition precedence over timeout expiry. The existing
  `WaitForSingleObject` bridge is now the one-handle case of the same parked
  wait record. Source anchor: `schedule.c::DoWaitForObjects`, whose proxies
  carry `WAIT_OBJECT_0 + idx` and current thread priority. Focused coverage:
  `wait_scheduler_tests`, including
  `ready_blocked_wait_selection_checks_all_multiple_wait_handles`. Full
  `cargo check --features unicorn,trace,win32-desktop` and
  `cargo test --features unicorn,trace,win32-desktop` pass, with the existing
  non-fatal Windows incremental-finalize warning. A fresh mounted virtual
  probe wrote `target\multiple_wait_virtual_60s_summary.txt`,
  `target\multiple_wait_virtual_60s_render.txt`,
  `target\multiple_wait_virtual_60s_milestones.txt`,
  `target\multiple_wait_virtual_60s_files.txt`, and
  `target\multiple_wait_virtual_60s.ppm`; it stopped at the 60 s wall limit
  (`pc=0x6000cfd4`, `ra=0x6000d044`) with stable counters
  (`heap_live=6921/21255717B`, `host_open=91`,
  `host_read=4304/1732170B`, `mem_open=2`, `max_read=497178`, and
  `sched=wait:1/0/0 ok:1 timeout:0 fail:0 block:0 wake:0`). This iNavi path
  did not exercise a multiple wait block in 60 s. Render milestones are still
  `none`; framebuffer output remains black except the 101-pixel red tap
  marker, so this is scheduler fidelity rather than UI progress.
- Scheduler/GWE bridging now also has the first Unicorn
  `MsgWaitForMultipleObjectsEx` parking path. The import-boundary bridge reads
  and validates the handle array, rejects wait-all parking, honors the existing
  GWE queue changed/input-available bits, parks only when no waited handle and
  no requested queue input is ready, wakes by CE priority/FIFO selection, and
  returns either `WAIT_OBJECT_0 + handle_index`, `WAIT_OBJECT_0 + handle_count`
  for message input, or `WAIT_TIMEOUT`. Source anchors are CE GWE
  `cmsgque.h` `MsgWaitForMultipleObjectsEx_*` wrappers, CE SDK `winuser.h`
  `MWMO_INPUTAVAILABLE`/queue flags, and NK
  `schedule.c::DoWaitForObjects` for the shared handle-wait proxy shape.
  Focused coverage extends `wait_scheduler_tests` with
  `blocked_msg_wait_wakes_on_queue_input`; full
  `cargo check --features unicorn,trace,win32-desktop` and
  `cargo test --features unicorn,trace,win32-desktop` pass with the existing
  non-fatal Windows incremental-finalize warning. A fresh mounted virtual
  probe wrote `target\msgwait_parking_virtual_60s_summary.txt`,
  `target\msgwait_parking_virtual_60s_render.txt`,
  `target\msgwait_parking_virtual_60s_milestones.txt`,
  `target\msgwait_parking_virtual_60s_files.txt`, and
  `target\msgwait_parking_virtual_60s.ppm`; it stopped at the 60 s wall limit
  (`pc=0x0006cbd4`, `ra=0x000bdfa0`) with stable counters
  (`heap_live=6927/21273103B`, `virtual_live=3/196608B`,
  `host_open=92`, `host_read=4305/1769298B`, `mem_open=2`,
  `max_read=497178`, and
  `sched=wait:1/0/0 ok:1 timeout:0 fail:0 block:0 wake:0`). This run did not
  exercise a parked msg-wait; render milestones are still `none`, and the
  framebuffer contains only the 101-pixel red tap marker.
- Scheduler/thread pseudo-handle fidelity now follows CE's current
  process/thread handle and KData contracts. `kfuncs.h` defines
  `SYS_HANDLE_BASE == 64`, `SH_CURTHREAD == 1`, and `SH_CURPROC == 2`, so
  `GetCurrentThread()`/`GetCurrentProcess()` return pseudo handles `65`/`66`
  while `GetCurrentThreadId()`/`GetCurrentProcessId()` read KData system-handle
  slots. `handle.c::LockHandleParam`, `schedule.c::LockWaitableObject`, and
  `thread.c::THRDGetCode`/`THRDGetTimes` map those pseudo handles to the active
  process/thread where appropriate. v3 now exposes pseudo-aware raw COREDLL
  `GetThreadId`, `GetExitCodeThread`, `GetThreadTimes`, `GetProcessId`,
  `GetExitCodeProcess`, `TerminateProcess`, and wait behavior, and Unicorn
  refreshes the KData current thread/process ID slots during guest thread,
  wait, and `SendMessageW` context switches. The same pseudo current-thread
  mapping now covers raw `SetThreadPriority`, `CeSetThreadPriority`,
  `SuspendThread`, and `ResumeThread`; real guest-thread objects are updated
  by thread id, while the main thread keeps CE priority/suspend metadata
  without inventing an app-specific handle. Focused coverage:
  `current_process_pseudo_handle_is_waitable_after_terminate`,
  `current_thread_pseudo_handle_updates_priority_and_suspend_state`,
  `user_kdata_page_exposes_current_thread_and_process_ids`, and
  `coredll_raw_ordinals_execute_kernel_thread_time_and_sync_semantics`. Full
  `cargo check --features unicorn,trace,win32-desktop` and
  `cargo test --features unicorn,trace,win32-desktop` pass, with the existing
  non-fatal Windows incremental-finalize warning. Mounted virtual probes wrote
  `target\pseudo_handle_kdata_virtual_60s_summary.txt`,
  `target\pseudo_handle_kdata_virtual_60s_render.txt`,
  `target\pseudo_handle_kdata_virtual_60s_milestones.txt`,
  `target\pseudo_handle_kdata_virtual_60s_files.txt`, and
  `target\pseudo_handle_kdata_virtual_60s.ppm`; it stopped at the 60 s wall
  limit (`pc=0x6000cee4`, `ra=0x6000d06c`) with stable counters
  (`heap_live=6921/21255717B`, `host_open=91`,
  `host_read=4304/1728191B`, `mem_open=2`, `max_read=497178`, and
  `sched=wait:1/0/0 ok:1 timeout:0 fail:0 block:0 wake:0`), and
  `target\pseudo_thread_mutation_virtual_60s_summary.txt`,
  `target\pseudo_thread_mutation_virtual_60s_render.txt`,
  `target\pseudo_thread_mutation_virtual_60s_milestones.txt`,
  `target\pseudo_thread_mutation_virtual_60s_files.txt`, and
  `target\pseudo_thread_mutation_virtual_60s.ppm`; the follow-up stopped at
  `pc=0x6000cfd4`, `ra=0x6000d044` with stable counters
  (`heap_live=6921/21255717B`, `host_open=91`,
  `host_read=4304/1732170B`, `mem_open=2`, `max_read=497178`, and
  `sched=wait:1/0/0 ok:1 timeout:0 fail:0 block:0 wake:0`). Render milestones
  remain `none`; the framebuffer still contains only the 101-pixel red tap
  marker, so this is CE scheduler/thread fidelity rather than UI progress.
- Mutex wait/release fidelity now follows the CE recursive ownership contract
  from `syncobj.c`, `kcalls.c`, `syncobj.h`, and `winerror.h`. `CreateMutexW`
  with initial ownership seeds owner/current-thread state with lock count `1`,
  recursive owner waits increment the count up to `MUTEX_MAXLOCKCNT == 0x7fff`,
  `ReleaseMutex` unwinds one count at a time, and wrong-owner/unowned release
  now fails with `ERROR_NOT_OWNER` while invalid handles still report
  `ERROR_INVALID_HANDLE`. Focused Rust coverage:
  `mutex_waits_track_recursive_owner_lock_count`,
  `mutex_recursive_wait_fails_at_ce_max_lock_count`, and the raw
  `ReleaseMutex` path in
  `coredll_raw_ordinals_execute_kernel_thread_time_and_sync_semantics`. Added
  `tests/test_progs/163_mutex_recursive_ownership` so the eVC4 MIPSII fixture
  suite can pin the same guest-visible behavior at the Win32 API boundary.
  Full `cargo check --features unicorn,trace,win32-desktop` and
  `cargo test --features unicorn,trace,win32-desktop` pass, with the existing
  non-fatal Windows incremental-finalize warning.
- Scheduler wait ownership now has the first real registry for blocked waits.
  Parked Unicorn `WaitForSingleObject`, `WaitForMultipleObjects(FALSE)`, and
  `MsgWaitForMultipleObjectsEx` still keep their saved MIPS register payload in
  Unicorn, but each parked wait is now registered in `Scheduler` with a wait id,
  thread id/handle, waited handles, kind, start tick, timeout, FIFO sequence,
  and per-handle waiter queues. Resume selection now asks the scheduler for the
  ready wait id using CE lower-numeric priority ordering and FIFO tie-breaks,
  then removes the scheduler registration when Unicorn restores the saved CPU
  context. Monitor/debug summaries now include waiter register/remove/max
  counters. Focused coverage: scheduler registry unit tests plus
  `wait_scheduler_tests`; `cargo check --features unicorn,trace,win32-desktop`
  passes with the existing non-fatal Windows incremental-finalize warning, and
  full `cargo test --features unicorn,trace,win32-desktop` passes. A mounted
  virtual probe using `D:\INAVI_Emulator\DUMPPLZ\Windows` wrote
  `target\scheduler_wait_registry_virtual_60s_summary.txt`,
  `target\scheduler_wait_registry_virtual_60s_render.txt`,
  `target\scheduler_wait_registry_virtual_60s_milestones.txt`,
  `target\scheduler_wait_registry_virtual_60s_files.txt`, and
  `target\scheduler_wait_registry_virtual_60s.ppm`; it stopped at the 60 s wall
  limit at `pc=0x00339ca4`, `ra=0x00b4bb1c` with stable memory
  (`heap_live=7471/23006007B`, `virtual_live=3/196608B`) and file activity
  (`host_open=231`, `host_read=38664/2226093B`, `mem_open=2`,
  `max_read=497178`). This path did not park a wait in 60 s
  (`reg:0/0 maxreg:0`), had no actual `SearchDB` `CreateFileW` failures and no
  `Access is denied` records, and still produced no render milestones; the
  framebuffer remains only the 301-pixel red tap line.
- Scheduler object-transition wake ownership now uses that blocked-wait
  registry for the first signal/release paths. Successful `SetEvent`,
  `ReleaseSemaphore`, and only the final recursive `ReleaseMutex` collect
  wait ids from the scheduler per-handle queues and enqueue them as pending
  wake candidates; live resume selection prefers those candidates before the
  global ready scan, then rechecks/consumes the underlying CE object state
  through the existing wait path. This keeps auto-reset events, semaphores, and
  recursive mutexes from being consumed by scheduler bookkeeping. Monitor and
  debug summaries now include object signal/candidate/max-pending counters.
  Focused coverage: scheduler pending-wake unit tests plus
  `object_transitions_queue_scheduler_wait_candidates`; full
  `cargo check --features unicorn,trace,win32-desktop` and
  `cargo test --features unicorn,trace,win32-desktop` pass, with the existing
  non-fatal Windows incremental-finalize warning. Added
  `tests/test_progs/164_object_transition_wake` so the eVC4 MIPSII fixture
  suite can pin event/semaphore/mutex wake behavior at the guest Win32 API
  boundary when enabled. A mounted virtual probe using
  `D:\INAVI_Emulator\DUMPPLZ\Windows` wrote
  `target\scheduler_object_wake_virtual_60s_summary.txt`,
  `target\scheduler_object_wake_virtual_60s_render.txt`,
  `target\scheduler_object_wake_virtual_60s_milestones.txt`,
  `target\scheduler_object_wake_virtual_60s_files.txt`, and
  `target\scheduler_object_wake_virtual_60s.ppm`; it stopped at the 60 s wall
  limit at `pc=0x00b54128`, `ra=0x002fd5e8` with stable memory
  (`heap_live=7488/23106403B`, `virtual_live=3/196608B`) and file activity
  (`host_open=237`, `host_read=38970/2222793B`, `mem_open=2`,
  `max_read=497178`). The real app path exercised one object signal but no
  registered waiter on that handle (`sig:1 cand:0 maxpend:0`) and still
  produced no render milestones; the framebuffer remains only the 301-pixel
  red tap line. `Access is denied` and actual failed `SearchDB` opens both
  remained at zero.
- Scheduler waitable-handle wake ownership now also covers thread and process
  exit transitions. `mark_guest_thread_exited`, child process completion via
  `mark_process_launch_exited`, child initial-thread completion, and raw
  `TerminateProcess` now enqueue scheduler wait ids registered under the
  corresponding thread/process handles after the handle is marked signaled,
  including waiters registered on the CE current-process pseudo handle.
  Existing `GetExitCodeThread`/`GetExitCodeProcess` state remains unchanged;
  the scheduler only owns wake-candidate routing. Focused coverage:
  `thread_and_process_exit_queue_scheduler_wait_candidates` and the raw
  `TerminateProcess` path inside
  `coredll_raw_ordinals_execute_kernel_thread_time_and_sync_semantics`; full
  `cargo check --features unicorn,trace,win32-desktop` and
  `cargo test --features unicorn,trace,win32-desktop` pass, with the existing
  non-fatal Windows incremental-finalize warning. Added
  `tests/test_progs/165_thread_exit_wait_wake` so the eVC4 MIPSII fixture
  suite can pin thread-handle exit signaling through `WaitForSingleObject`,
  `GetExitCodeThread`, and wait-any. A mounted virtual probe using
  `D:\INAVI_Emulator\DUMPPLZ\Windows` wrote
  `target\scheduler_exit_wake_virtual_60s_summary.txt`,
  `target\scheduler_exit_wake_virtual_60s_render.txt`,
  `target\scheduler_exit_wake_virtual_60s_milestones.txt`,
  `target\scheduler_exit_wake_virtual_60s_files.txt`, and
  `target\scheduler_exit_wake_virtual_60s.ppm`; it stopped at the 60 s wall
  limit at `pc=0x00b54128`, `ra=0x002fd5e8` with stable memory
  (`heap_live=7586/23006573B`, `virtual_live=3/196608B`) and file activity
  (`host_open=227`, `host_read=37888/2191219B`, `mem_open=2`,
  `max_read=497178`). The real app path exercised seven object signals but no
  registered waiters on those handles (`sig:7 cand:0 maxpend:0`) and still
  produced no render milestones; the framebuffer remains only the 301-pixel
  red tap line. `Access is denied` and actual failed `SearchDB` opens both
  remained at zero.
- Scheduler message-input wake ownership now covers the first GWE/timer input
  transitions. `Scheduler` keeps a per-thread message-wait queue for parked
  `MsgWaitForMultipleObjectsEx` waits, and kernel/GWE transitions for posted
  messages, thread messages, broadcast messages, quit messages, queued sent
  messages, remote input, and `WM_TIMER` posts enqueue pending message-wake
  candidates. Resume selection still rechecks real GWE queue status and wake
  masks before returning, so queue status consumption remains GWE-owned.
  Monitor/debug summaries now include `msgsig`/`msgcand` counters. Focused
  coverage: `scheduler_queues_message_waiters_by_thread` and
  `message_and_timer_transitions_queue_scheduler_msg_wait_candidates`; full
  `cargo check --features unicorn,trace,win32-desktop` and
  `cargo test --features unicorn,trace,win32-desktop` pass, with the existing
  non-fatal Windows incremental-finalize warning. Added
  `tests/test_progs/166_msgwait_message_timer_wake` so the eVC4 MIPSII fixture
  suite can pin posted-message and timer wake behavior through
  `MsgWaitForMultipleObjectsEx`. A mounted virtual probe using
  `D:\INAVI_Emulator\DUMPPLZ\Windows` wrote
  `target\scheduler_msgwait_virtual_60s_summary.txt`,
  `target\scheduler_msgwait_virtual_60s_render.txt`,
  `target\scheduler_msgwait_virtual_60s_milestones.txt`,
  `target\scheduler_msgwait_virtual_60s_files.txt`, and
  `target\scheduler_msgwait_virtual_60s.ppm`; it stopped at the 60 s wall
  limit at `pc=0x00339c44`, `ra=0x00b4bb1c` with stable memory
  (`heap_live=7588/23317613B`, `virtual_live=3/196608B`) and file activity
  (`host_open=227`, `host_read=37890/2211452B`, `mem_open=2`,
  `max_read=497178`). The real app path exercised message-input transitions
  (`msgsig:148`) but no registered message waiters in this window
  (`msgcand:0`), still produced no render milestones, and the framebuffer
  remains only the 301-pixel red tap line. `Access is denied` and actual
  failed `SearchDB` opens both remained at zero.
- Scheduler/GWE ownership now covers the first parked `GetMessageW` bridge.
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\cmsgque.h` declares blocking
  `GetMessageW_I` separately from `GetMessageWNoWait_I` and documents paint
  requests as queue state observed by later `GetMessage` calls. v3 now
  registers Unicorn-blocked `GetMessageW` calls in the scheduler
  message-wait queue with the original HWND/min/max filters, lets normal GWE
  post/quit/sent/timer/input transitions enqueue those waits as pending
  candidates, and rechecks immutable filtered GWE readiness before restoring
  the saved guest CPU context and consuming the message. Focused coverage:
  `scheduler_queues_get_message_waiters_by_thread`,
  `get_message_waiter_uses_filtered_scheduler_message_readiness`, and the full
  `basic_subsystems` suite. `cargo check --features
  unicorn,trace,win32-desktop` passes with the existing non-fatal Windows
  incremental-finalize warning. This moves plain `GetMessageW` blocking onto
  the same scheduler registry as `MsgWaitForMultipleObjectsEx`; full run-queue
  ownership and moving the saved MIPS context out of the Unicorn bridge remain
  open.
- CE timer delivery now preserves thread-owned no-HWND timers and avoids host
  sleeping on CE virtual time. `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\cmsgque.h`
  `TimerEntry_t` stores the owning `MsgQueue` as well as optional `HWND`, so
  v3 timers now carry an owner thread id; raw `SetTimer` registers timers for
  the current thread when `hwnd == NULL`, and `pump_timers_to_gwe` posts
  due `WM_TIMER` messages to that owner instead of silently dropping no-HWND
  timers. `TimerSystem::sleep_ms` now advances an emulator virtual elapsed
  offset instead of calling `std::thread::sleep`, so CE sleeps/timer fast
  forwards do not consume host wall time. Focused coverage:
  `sleep_advances_virtual_timer_clock`, the no-HWND thread-timer assertions in
  `message_and_timer_transitions_queue_scheduler_msg_wait_candidates`, the full
  `basic_subsystems` suite, and the full `coredll_raw_gwe` suite. Full
  `cargo check --features unicorn,trace,win32-desktop` and release build pass.
  A mounted virtual run using dumped runtime DLLs wrote
  `target\thread_timer_virtual_*`; it cleared the previous
  `COREDLL.dll@861 blocked_get_message` frontier and instead ran until the
  120 s wall-clock limit at `pc=0x70028b7c`, `ra=0x6002537c`. The run stayed
  memory-stable (`heap_live=13697/13300954B`,
  `virtual_live=3/196608B`, `host_open=665`,
  `host_read=80132/4053923B`, `mem_open=3`, `max_read=685080`) and delivered
  repeated no-HWND `WM_TIMER` messages (`hwnd=0`, `wparam=1000`), but render
  milestones still show memory-DC DIB/blit work only and the framebuffer still
  contains only the red tap marker (`401` nonzero pixels). This is a startup
  speed/scheduler correctness fix, not UI success.
- Scheduler/timer ownership now has the first parked worker-thread
  `Sleep(ms)` bridge. CE `schedule.c` implements `NKSleep` via
  `ThreadSleep`/`PutThreadToSleep`, with bounded sleeps entering the kernel
  sleep list, `Sleep(0)` yielding, `Sleep(INFINITE)` suspending the current
  thread, and `NKSleepTillTick` sleeping one tick. v3 now centralizes that CE
  sleep request shape: bounded `Sleep(ms)` below `0xfffffffe` uses the CE
  `ms + 1` timeout, `SleepTillTick` uses a one-tick timeout, and raw
  host-side dispatch avoids blocking the host for `Sleep(INFINITE)`. The
  Unicorn bridge registers bounded raw `Sleep(ms)` and `SleepTillTick` calls
  from guest worker-thread contexts as timeout-only scheduler waits, switches
  back to an available saved context, and resumes the sleeping worker with
  return value `0` after the scheduler timeout expires. `Sleep(0)` now records
  a scheduler yield and swaps to a saved peer context when the current
  one-slot Unicorn bridge has one available; the no-peer path still returns
  immediately through raw dispatch. `Sleep(INFINITE)` now increments the CE
  current-thread suspend count in raw dispatch and, for guest worker contexts
  with a real thread handle, saves the worker CPU context until `ResumeThread`
  drops that suspend count from `1` to `0`; the resume hook then restores the
  saved context instead of restarting the thread entry point. Run summaries now include
  `sched_sleep_count` and `sched_yield_count`. Focused coverage:
  `ce_sleep_request_matches_nksleep_timeout_shape`,
  `scheduler_selects_timeout_only_sleep_wait_after_timeout`,
  `scheduler_records_thread_yield_as_sleep_attempt_without_blocking`, and the raw
  `coredll_raw_ordinals_execute_kernel_thread_time_and_sync_semantics`
  `Sleep`/`SleepTillTick`/current-thread suspend path. Added
  `tests/test_progs/167_sleep_infinite_resume` to pin the guest-visible worker
  `Sleep(INFINITE)`/`ResumeThread` contract. `cargo check --features
  unicorn,trace,win32-desktop`, the focused raw/suspend tests, and
  `cargo test --features unicorn,trace,win32-desktop fixture_exes -- --ignored`
  pass with the existing non-fatal Windows incremental-finalize warning. Full
  scheduler-owned run queues beyond the current one-slot yield/suspend swaps,
  pending PSL late-suspend, long-sleep chunking, and scheduler-owned
  main-thread run-queue state remain open.
- Scheduler/device wake ownership now has a first serial-read slice. The
  `Scheduler` can register blocked `SerialRead` waits by COM handle and queue
  pending wake candidates when remote serial/NMEA input arrives; serial device
  sessions now expose normalized target matching so `COM7:`, `COM7`, and
  `\\.\COM7` style names line up with the remote GPS target. `CeKernel`
  drains matching `CeRemote` serial bytes into the target device session just
  before `ReadFile`/`ReadFileInto`, and the Unicorn raw `ReadFile` bridge can
  park an empty serial read, then resume by streaming bytes directly into the
  original guest buffer via `kernel.read_file_into`. This is a generic
  scheduler/device path, not an iNavi-specific serial shortcut. Focused
  coverage: `scheduler_queues_serial_read_waiters_by_handle` and
  `remote_serial_injection_queues_scheduler_serial_read_candidates`; full
  `cargo check --features unicorn,trace,win32-desktop` and
  `cargo test --features unicorn,trace,win32-desktop` pass. Remaining serial
  gaps include COMMTIMEOUTS, `WaitCommEvent`, masks, purge/error state, active
  remote-server wake/resume integration, the optional `win32_com` host backend,
  and the no-alternate-thread read-blocking edge.
- Virtual-desktop runs no longer register the WinMM host audio sink. Audio
  registration now follows the presenter mode: `--desktop host` keeps the
  existing host sink behavior, while `--desktop virtual` registers a debug
  `LoggingAudioSink` in debug builds and a release `NullAudioSink` that accepts
  and discards PCM without queuing playback. Focused coverage:
  `null_audio_sink_accepts_and_discards_pcm` and
  `virtual_desktop_uses_headless_audio_sink`; `cargo check --features
  unicorn,trace,win32-desktop` passes. A verbose virtual boot now reports
  `host audio: virtual desktop logging sink registered`, confirming headless
  probes do not attempt host playback.
- A requested one-shot host-presented probe of dumped
  `D:\INAVI_Emulator\DUMPPLZ\Windows\explorer.exe` did not reach CE startup or
  emit trace summaries. The run used `--desktop host` and the dumped Windows
  directory as `--dll-search-dir`, but failed while building a MIPS trampoline:
  `MIPS jump target 0xffff832c is outside direct jump region from 0x00057108`.
  A repeat run on 2026-06-04 using the Win32 host presenter and fresh
  `target\explorer_win32_host_once_*` trace paths reproduced the same failure
  before any summaries were written. This is a loader/trampoline reachability
  gap for high-address targets, not evidence that the host presenter or
  virtual/null-audio path is blocking.
- Raw `SetAssociatedMenu @299` and `GetAssociatedMenu @300` now reach the same
  CE HWND-associated menu state as `SetMenu`/`GetMenu`. The raw dispatcher
  validates live HWNDs through the GWE window table and keeps the CE
  `SetAssociatedMenu_I` void-style return at the trap boundary while still
  setting last error on invalid HWNDs. Source anchors are CE GWE
  `window.hpp::SetAssociatedMenu_I`/`GetAssociatedMenu_I` and
  `gweapiset1.hpp` entries for those functions. Focused coverage extends
  `coredll_raw_window_menu_state_preserves_child_control_ids`; the raw GWE
  suite passes (`54 passed`), `cargo check --features
  unicorn,trace,win32-desktop` passes, and
  `cargo test --features unicorn,trace,win32-desktop` passes (`94` unit tests
  plus integration suites). A mounted virtual-desktop iNavi probe using
  `D:\INAVI_Emulator\DUMPPLZ\Windows` wrote
  `target\associated_menu_virtual_60s_summary.txt`,
  `target\associated_menu_virtual_60s_render.txt`,
  `target\associated_menu_virtual_60s_milestones.txt`,
  `target\associated_menu_virtual_60s_files.txt`, and
  `target\associated_menu_virtual_60s.ppm`; it stopped at the 60 s wall limit
  (`pc=0x00929804`, `ra=0x000b9980`) with stable counters
  (`heap_live=6929/21276863B`, `virtual_live=3/196608B`,
  `host_open=97`, `host_read=4332/1769576B`, `mem_open=2`,
  `max_read=497178`) and no render milestones. The framebuffer still contains
  only 101 red pixels from `(0,160)` through `(100,160)`, color `255,0,0`, so
  this is associated-menu API fidelity rather than useful UI output.
- Raw menu enable/check state now honors CE by-position command UI updates.
  `EnableMenuItem @847` mutates ordered virtual menu items and returns the
  previous enabled/disabled/grayed state, and raw `CheckMenuItem` now respects
  `MF_BYPOSITION` instead of treating the item argument only as a command ID.
  Source anchors are CE SDK `winuser.h` menu flags/`EnableMenuItem` and CE MFC
  `cmdtarg.cpp::CCmdUI::Enable`/`SetCheck`, which update menu items by
  position with `MF_DISABLED | MF_GRAYED` and `MF_CHECKED`. Focused coverage is
  folded into `coredll_raw_menu_items_round_trip_through_ce_menuiteminfo`; the
  raw GWE suite passes (`54 passed`), `cargo check --features
  unicorn,trace,win32-desktop` passes, and
  `cargo test --features unicorn,trace,win32-desktop` passes (`94` unit tests
  plus integration suites). A mounted virtual-desktop iNavi probe using
  `D:\INAVI_Emulator\DUMPPLZ\Windows` wrote
  `target\menu_enable_virtual_60s_summary.txt`,
  `target\menu_enable_virtual_60s_render.txt`,
  `target\menu_enable_virtual_60s_milestones.txt`,
  `target\menu_enable_virtual_60s_files.txt`, and
  `target\menu_enable_virtual_60s.ppm`; it stopped at the 60 s wall limit
  (`pc=0x000b9940`, `ra=0x000b993c`) with stable counters
  (`heap_live=6929/21276863B`, `virtual_live=3/196608B`,
  `host_open=97`, `host_read=5581/1766846B`, `mem_open=2`,
  `max_read=497178`) and no render milestones. The framebuffer still contains
  only 101 red pixels from `(0,160)` through `(100,160)`, color `255,0,0`, so
  this is CE menu command-state progress rather than useful UI output.
- Raw CE menu item state now covers ordered virtual menu entries and the
  common `MENUITEMINFOW` boundary. `CreateMenu`, `CreatePopupMenu`,
  `AppendMenuW`, `InsertMenuW`, `RemoveMenu`/`DeleteMenu`, `GetSubMenu`,
  `GetMenuItemInfoW`, and `SetMenuItemInfoW` now dispatch through raw COREDLL
  into `ResourceSystem` menu objects instead of acting as generic success
  stubs. Menu items preserve command IDs, popup submenu handles, type/state
  flags, checkmark bitmap handles, item data, and wide text; `CheckMenuItem`
  and `CheckMenuRadioItem` also update ordered item state where present. Source
  anchors are CE SDK `winuser.h` menu flags/`MENUITEMINFOW`/`MIIM_*` and CE MFC
  `winfrm.cpp` menu traversal/item-info use. Focused coverage:
  `coredll_raw_menu_items_round_trip_through_ce_menuiteminfo`; the raw GWE
  suite passes (`54 passed`), `cargo check --features
  unicorn,trace,win32-desktop` passes, and
  `cargo test --features unicorn,trace,win32-desktop` passes (`94` unit tests
  plus integration suites). A mounted virtual-desktop iNavi probe using
  `D:\INAVI_Emulator\DUMPPLZ\Windows` wrote
  `target\menu_items_virtual_60s_summary.txt`,
  `target\menu_items_virtual_60s_render.txt`,
  `target\menu_items_virtual_60s_milestones.txt`,
  `target\menu_items_virtual_60s_files.txt`, and
  `target\menu_items_virtual_60s.ppm`; it stopped at the 60 s wall limit
  (`pc=0x00496a44`, `ra=0x002017d0`) with stable counters
  (`heap_live=6930/21302289B`, `virtual_live=2/131072B`,
  `host_open=92`, `host_read=4305/1769298B`, `mem_open=2`,
  `max_read=497178`) and no render milestones. The framebuffer still contains
  only 101 red pixels from `(0,160)` through `(100,160)`, color `255,0,0`, so
  this is menu-resource fidelity progress rather than useful UI output.
- Raw HWND menu association now covers the first CE/MFC menu-state slice.
  Virtual GWE windows store an optional `HMENU`; raw `CreateWindowExW` treats
  the `hMenu` argument as a top-level window menu when `WS_CHILD` is absent and
  as the child/control id when `WS_CHILD` is present. Raw `SetMenu`,
  `GetMenu`, and `DrawMenuBar` now dispatch through COREDLL to that HWND state
  with invalid-HWND last-error handling; `DrawMenuBar` validates only for now
  and does not fake menu painting. Source anchors are CE SDK `winuser.h`
  `CREATESTRUCTW.hMenu`/`CreateWindowExW`/`DrawMenuBar` and CE MFC
  `wincore.cpp::PreCreateWindowEx`/`PostCreateWindowEx`, which strips
  standalone menus during create and reattaches them with `SetMenu`. Focused
  coverage: `coredll_raw_window_menu_state_preserves_child_control_ids`; the
  raw GWE suite passes (`53 passed`), `cargo check --features
  unicorn,trace,win32-desktop` passes, and
  `cargo test --features unicorn,trace,win32-desktop` passes (`94` unit tests
  plus integration suites). A mounted virtual-desktop iNavi probe using
  `D:\INAVI_Emulator\DUMPPLZ\Windows` wrote
  `target\menu_attach_virtual_60s_summary.txt`,
  `target\menu_attach_virtual_60s_render.txt`,
  `target\menu_attach_virtual_60s_milestones.txt`,
  `target\menu_attach_virtual_60s_files.txt`, and
  `target\menu_attach_virtual_60s.ppm`; it stopped at the 60 s wall limit
  (`pc=0x004d8ba8`, `ra=0x0006b8b0`) with stable counters
  (`heap_live=6917/21255371B`, `host_open=91`,
  `host_read=4302/1718377B`, `mem_open=2`, `max_read=497178`) and no render
  milestones. The framebuffer still contains only 101 red pixels from
  `(0,160)` through `(100,160)`, color `255,0,0`, so this is menu/GWE fidelity
  progress rather than useful UI output.
- The Unicorn `CreateWindowExW` guest-WNDPROC callout now preserves the CE/MFC
  `WM_CREATE` failure contract: a guest `WM_CREATE` return of `-1` makes the
  raw API return `NULL` and destroys the just-created virtual HWND, while a
  normal return still returns the created HWND. Focused coverage:
  `emulator::unicorn::unicorn_tests::create_window_callout_returns_hwnd_or_null_after_wm_create`;
  the raw GWE suite still passes (`52 passed`), `cargo check --features
  unicorn,trace,win32-desktop` passes, and
  `cargo test --features unicorn,trace,win32-desktop` passes (`94` unit tests
  plus integration suites). A source/probe correction happened in the same
  slice: an experimental unconditional `WM_NCCREATE` callout at
  `CreateWindowExW` regressed mounted startup to an immediate empty
  `GetMessageW` frontier (`target\nc_create_virtual_60s_*`,
  `pc=0x7fff0b60`, `heap_live=24/12914B`, `host_read=0/0B`), so that runtime
  behavior was removed. The corrected mounted virtual-desktop iNavi probe using
  `D:\INAVI_Emulator\DUMPPLZ\Windows` wrote
  `target\create_abort_virtual_60s_summary.txt`,
  `target\create_abort_virtual_60s_render.txt`,
  `target\create_abort_virtual_60s_milestones.txt`,
  `target\create_abort_virtual_60s_files.txt`, and
  `target\create_abort_virtual_60s.ppm`; it stopped at the 60 s wall limit
  (`pc=0x001e5408`, `ra=0x000c1944`) with stable counters
  (`heap_live=6926/21256719B`, `host_open=91`,
  `host_read=4304/1732170B`, `mem_open=2`, `max_read=497178`) and no render
  milestones. The framebuffer still contains only 101 red pixels from
  `(0,160)` through `(100,160)`, color `255,0,0`, so this is create-lifecycle
  fidelity progress rather than useful UI output.
- Raw dialog/control text APIs now share the virtual child-window text model
  with raw message forwarding. `GetDlgItem`, `SetDlgItemTextW`,
  `GetDlgItemTextW`, `SendDlgItemMessageW(WM_SETTEXT/WM_GETTEXT/
  WM_GETTEXTLENGTH)`, and direct `SendMessageW(WM_GETTEXT*)` now move text
  through the raw COREDLL guest-memory boundary instead of returning generic
  message defaults or allocating helper-only values. `SendDlgItemMessageW`
  still preserves the existing `BM_GETCHECK`/`BM_SETCHECK` button-state path.
  Focused coverage:
  `coredll_raw_dialog_controls_support_text_and_message_forwarding`; the full
  raw GWE suite passes (`52 passed`), `cargo check --features
  unicorn,trace,win32-desktop` passes, and
  `cargo test --features unicorn,trace,win32-desktop` passes (`93` unit tests
  plus integration suites). A mounted virtual-desktop iNavi probe using
  `D:\INAVI_Emulator\DUMPPLZ\Windows` as the DLL source wrote
  `target\dialog_text_virtual_60s_summary.txt`,
  `target\dialog_text_virtual_60s_render.txt`,
  `target\dialog_text_virtual_60s_milestones.txt`,
  `target\dialog_text_virtual_60s_files.txt`, and
  `target\dialog_text_virtual_60s.ppm`; it stopped at the 60 s wall limit
  (`pc=0x0001362c`, `ra=0x0013dfc0`) with stable counters
  (`heap_live=7041/21284917B`, `virtual_live=3/196608B`,
  `host_open=113`, `host_read=7843/1763759B`, `mem_open=2`,
  `max_read=497178`) and no render milestones. The framebuffer still contains
  only 101 red pixels from `(0,160)` through `(100,160)`, color `255,0,0`, so
  this is dialog/control fidelity progress rather than useful UI output.
- Raw `GetUpdateRect` and `GetUpdateRgn` now honor CE's `bErase` path for
  pending paint without validating the update region. When a HWND has
  `erase_pending`, the raw query writes/copies the pending bounds, sends
  `WM_ERASEBKGND` with the HWND paint HDC through the normal window send path,
  and clears only the erase bit so subsequent `BeginPaint` reports
  `PAINTSTRUCT.fErase = FALSE` while the dirty bounds remain pending. Focused
  coverage: `coredll_raw_get_update_queries_consume_pending_erase_only`; the
  full raw GWE suite passes (`51 passed`), `cargo check --features
  unicorn,trace,win32-desktop` passes, and
  `cargo test --features unicorn,trace,win32-desktop` passes (`93` unit tests
  plus integration suites). A mounted virtual-desktop iNavi probe using
  `D:\INAVI_Emulator\DUMPPLZ\Windows` as the DLL source wrote
  `target\get_update_erase_virtual_60s_summary.txt`,
  `target\get_update_erase_virtual_60s_render.txt`,
  `target\get_update_erase_virtual_60s_milestones.txt`,
  `target\get_update_erase_virtual_60s_files.txt`, and
  `target\get_update_erase_virtual_60s.ppm`; it stopped at the 60 s wall limit
  (`pc=0x00a436e0`, `ra=0x002017e0`) with stable counters
  (`heap_live=6930/21294161B`, `virtual_live=2/131072B`,
  `host_open=92`, `host_read=4305/1769298B`, `mem_open=2`,
  `max_read=497178`) and no render milestones. The framebuffer still contains
  only 101 red pixels from `(0,160)` through `(100,160)`, color `255,0,0`, so
  this is paint/update fidelity progress rather than useful UI output.
- Raw `CreateWindowExW` now distinguishes CE child parenting from top-level
  ownership at the syscall boundary. When `WS_CHILD` is present, the
  `hWndParent` argument becomes the virtual parent and child coordinates are
  parent-client-relative; when `WS_CHILD` is absent, the argument becomes the
  owner reported by `GetWindow(GW_OWNER)` and the window remains a top-level
  desktop child with screen-relative coordinates. Existing direct kernel/GWE
  test helpers keep their explicit parent semantics. Focused coverage:
  `coredll_raw_create_window_distinguishes_owner_from_child_parent`, plus the
  raw GWE integration suite (`50 passed`) and
  `cargo test --features unicorn,trace,win32-desktop` (`93` unit tests plus
  integration suites passed). A mounted virtual-desktop iNavi probe wrote
  `target\owner_child_virtual_60s_summary.txt`,
  `target\owner_child_virtual_60s_render.txt`,
  `target\owner_child_virtual_60s_milestones.txt`,
  `target\owner_child_virtual_60s_files.txt`, and
  `target\owner_child_virtual_60s.ppm`; it stopped at the 60 s wall limit
  (`pc=0x002a252c`, `ra=0x00135468`) with stable counters
  (`heap_live=6940/21278707B`, `virtual_live=3/196608B`,
  `host_open=112`, `host_read=7840/1760751B`, `mem_open=2`,
  `max_read=497178`) and no render milestones. The framebuffer still contains
  only 101 red pixels from `(0,160)` through `(100,160)`, color `255,0,0`, so
  this remains fidelity progress rather than UI success.
- Raw `SetParent` now goes through the kernel window lifecycle boundary instead
  of mutating GWE state directly. The raw ordinal preserves the previous-parent
  return, reports invalid HWNDs and parent-cycle attempts distinctly, rejects
  descendant-parent cycles, relinks the HWND into the new parent's z-order
  sibling set, and clears descendant focus/explicit activation through normal
  `WM_KILLFOCUS`/`WM_ACTIVATE(WA_INACTIVE)` messages when the new ancestry
  makes the subtree effectively hidden or disabled. Focused coverage:
  `coredll_raw_set_parent_relinks_tree_and_clears_invalid_focus`, plus the raw
  GWE integration suite (`49 passed`) and
  `cargo test --features unicorn,trace,win32-desktop` (`93` unit tests plus
  integration suites passed). A mounted virtual-desktop iNavi probe wrote
  `target\set_parent_virtual_60s_summary.txt`,
  `target\set_parent_virtual_60s_render.txt`,
  `target\set_parent_virtual_60s_milestones.txt`,
  `target\set_parent_virtual_60s_files.txt`, and
  `target\set_parent_virtual_60s.ppm`; it stopped at the 60 s wall limit
  (`pc=0x000be6e4`, `ra=0x000be6e0`) with stable memory/file counters
  (`heap_live=6921/21255717B`, `host_open=91`,
  `host_read=4302/1718377B`, `mem_open=2`, `max_read=497178`) and no render
  milestones. The framebuffer still contains only 101 red pixels from
  `(0,160)` through `(100,160)`, color `255,0,0`, so this is fidelity progress
  rather than UI success.
- GWE focus/activation lifecycle is now cleared through normal CE-style
  messages when disabling or hiding the focused/active HWND or one of its
  ancestors. `EnableWindow(FALSE)` queues `WM_CANCELMODE`, `WM_ENABLE(FALSE)`,
  then clears descendant focus/explicit activation through `WM_KILLFOCUS` and
  `WM_ACTIVATE(WA_INACTIVE)`. `ShowWindow(SW_HIDE)` and
  `SetWindowPos(SWP_HIDEWINDOW)` use the same focus/activation cleanup path.
  Focused coverage:
  `coredll_raw_disable_or_hide_clears_focus_and_activation`, plus the full raw
  GWE integration suite (`48 passed`) and
  `cargo test --features unicorn,trace,win32-desktop` (`93` unit tests plus
  integration suites passed). A mounted virtual-desktop iNavi probe wrote
  `target\focus_activation_virtual_60s_summary.txt`,
  `target\focus_activation_virtual_60s_render.txt`,
  `target\focus_activation_virtual_60s_milestones.txt`,
  `target\focus_activation_virtual_60s_files.txt`, and
  `target\focus_activation_virtual_60s.ppm`; it stopped at the 60 s wall limit
  (`pc=0x002036fc`, `ra=0x000c47f8`) with stable memory/file counters
  (`heap_live=7089/21301763B`, `virtual_live=3/196608B`,
  `host_open=115`, `host_read=7852/1765593B`, `mem_open=2`,
  `max_read=497178`) and no render milestones. The framebuffer still contains
  only the known sparse red line: 301 pixels from `(0,160)` through
  `(300,160)`, color `255,0,0`. This is lifecycle fidelity progress, not UI
  success.
- CE fidelity catch-up has started with a dedicated scheduler/wait owner rather
  than another local stub layer. `src/ce/scheduler.rs` now records
  `WaitForSingleObject`, `WaitForMultipleObjects`, and
  `MsgWaitForMultipleObjectsEx` attempts/outcomes, blocked Unicorn waits,
  resumed waits, max wait handle count, and max timeout. `CeKernel` exposes the
  scheduler stats, Unicorn debug snapshots include compact `sched=...`
  summaries, and `SOURCE_REFERENCES.md`/`TODO.md` carry the CE scheduler ledger
  entry. Parked Unicorn `WaitForSingleObject` waits now keep start tick/timeout
  metadata and resume with `WAIT_TIMEOUT` when the bounded wait expires, while
  signaled object resumes still acquire the object first and `INFINITE` waits
  remain untimed. CE6 `WaitForMultipleObjects(TRUE)` remains rejected at the raw
  API boundary per `NKWaitForMultipleObjects`. Full scheduler-owned waiter
  queues/context switching remain open. A short mounted host/tap probe after
  the timeout slice wrote `target\scheduler_timeout_summary.txt`,
  `target\scheduler_timeout_files.txt`,
  `target\scheduler_timeout_render.txt`,
  `target\scheduler_timeout_milestones.txt`, and
  `target\scheduler_timeout_probe.ppm`; it stopped at the normal 10 s
  wall-clock frontier (`pc=0x00339da8`, `ra=0x0033a624`) with compact file/RSS
  counters (`host_read=4219/486039B`, `heap_live=5948/2767663B`) and no render
  milestones, so it is a regression check rather than UI progress.
- Window/GWE fidelity planning now has a durable ledger in `TODO.md` with CE
  source anchors, v2 corroboration, port order, fixture gates, and latest iNavi
  evidence. The first concrete window fix changes raw `UpdateWindow` from a
  valid-HWND no-op into a synchronous pending-paint delivery: if an update
  region exists, `CeKernel::update_window` sends `WM_PAINT` through the normal
  window send path, which validates the synthetic paint request. This matches
  the CE paint-request model from `cmsgque.h`: paint is not an ordinary posted
  message and is canceled by paint processing. A short mounted host/tap
  regression probe wrote `target\window_update_summary.txt`,
  `target\window_update_files.txt`, `target\window_update_render.txt`,
  `target\window_update_milestones.txt`, and `target\window_update_probe.ppm`;
  it stayed on the expected early resource-loading wall-clock frontier and is
  not UI success.
- The same paint/update slice now includes raw `RedrawWindow`: rectangle and
  HRGN invalidation feed the pending update state, repeated invalidations union
  their rectangles, `RDW_ALLCHILDREN` reaches descendants, `RDW_VALIDATE`
  validates pending paint, erase state reaches `BeginPaint`, and
  `RDW_UPDATENOW` forces synchronous `WM_PAINT` through
  `CeKernel::update_window`. Focused coverage:
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe
  coredll_raw_redraw_window_invalidates_regions_children_and_updates_now`, plus
  the full raw GWE integration test. A short mounted host/tap regression probe
  wrote `target\redraw_window_summary.txt`, `target\redraw_window_files.txt`,
  `target\redraw_window_render.txt`, `target\redraw_window_milestones.txt`, and
  `target\redraw_window_probe.ppm`; it stopped at the expected 10 s wall limit
  with small file/RSS counters (`host_read=4221/499832B`,
  `heap_live=5948/2767663B`) and no render milestones, so it is not UI success.
- Paint/update validation now preserves remaining dirty bounds instead of
  clearing the whole window for every rect validation. `ValidateRect(hwnd,
  rect)` and `RedrawWindow(..., hrgn, RDW_VALIDATE)` subtract representable
  rectangular update bounds, while `ValidateRect(hwnd, NULL)` still clears the
  full update state. Focused coverage:
  `ce::gwe::tests::validate_window_rect_subtracts_representable_update_bounds`
  and `coredll_raw_validate_rect_preserves_remaining_update_bounds`, plus the
  full raw GWE integration test. A short mounted host/tap regression probe
  wrote `target\validate_rect_summary.txt`, `target\validate_rect_files.txt`,
  `target\validate_rect_render.txt`, `target\validate_rect_milestones.txt`, and
  `target\validate_rect_probe.ppm`; it stopped at the expected 10 s wall limit
  with small file/RSS counters (`host_read=4221/495853B`,
  `heap_live=5948/2767663B`) and no render milestones, so it is not UI success.
- Raw `GetUpdateRgn` now bridges GWE pending-paint state into existing GDI
  region objects: no pending update returns `NULLREGION` and clears the output
  HRGN to an empty rectangle, pending paint copies the update bounds and returns
  `SIMPLEREGION`, and invalid HWND/HRGN cases return `ERROR_REGION`. Focused
  coverage: `coredll_raw_get_update_rgn_copies_pending_paint_bounds`, plus the
  full raw GWE integration test. A short mounted host/tap regression probe
  wrote `target\get_update_rgn_summary.txt`, `target\get_update_rgn_files.txt`,
  `target\get_update_rgn_render.txt`, `target\get_update_rgn_milestones.txt`,
  and `target\get_update_rgn_probe.ppm`; it stopped at the expected 10 s wall
  limit with small file/RSS counters (`host_read=4221/499832B`,
  `heap_live=5948/2767663B`) and no render milestones, so it is not UI success.
- Raw `GetWindowThreadProcessId` now reports stored HWND owner metadata from
  GWE: it returns the creating thread ID, writes the owner process ID when the
  caller supplies an output pointer, accepts a null process-output pointer, and
  rejects destroyed/invalid HWNDs. Focused coverage:
  `coredll_raw_get_window_thread_process_id_reports_owner_ids`, plus the full
  raw GWE integration test. A short mounted host/tap regression probe wrote
  `target\window_owner_ids_summary.txt`, `target\window_owner_ids_files.txt`,
  `target\window_owner_ids_render.txt`,
  `target\window_owner_ids_milestones.txt`, and
  `target\window_owner_ids_probe.ppm`; it stopped at the expected 10 s wall
  limit with small file/RSS counters (`host_read=4219/486039B`,
  `heap_live=5948/2767663B`) and no render milestones, so it is not UI success.
- Raw `IsChild` now uses the virtual HWND tree instead of a generic stub:
  direct children and descendants return true, self/sibling/invalid/destroyed
  relationships return false. Focused coverage:
  `coredll_raw_is_child_checks_descendant_relationships`, plus the full raw GWE
  integration test. A short mounted host/tap regression probe wrote
  `target\is_child_summary.txt`, `target\is_child_files.txt`,
  `target\is_child_render.txt`, `target\is_child_milestones.txt`, and
  `target\is_child_probe.ppm`; it stopped at the expected 10 s wall limit with
  small file/RSS counters (`host_read=4235/491369B`,
  `heap_live=5649/19244201B`) and no render milestones. The short probe did
  reach later map/device file activity (`mapinfo.bin`, `UID1:`), but this is
  not UI success.
- Raw `SendNotifyMessageW` now follows the CE GWE no-wait notification split:
  sends to windows owned by the caller thread still execute synchronously
  through `SendMessageW`, while sends to windows on a different thread are
  queued to the receiver instead of immediately running or destroying the
  target. Focused coverage:
  `coredll_raw_send_notify_message_is_async_across_threads`, plus the full raw
  GWE integration test. A short mounted host/tap regression probe wrote
  `target\send_notify_summary.txt`, `target\send_notify_files.txt`,
  `target\send_notify_render.txt`, `target\send_notify_milestones.txt`, and
  `target\send_notify_probe.ppm`; it stopped at the 10 s wall limit with small
  file/RSS counters (`host_read=4225/486559B`,
  `heap_live=5624/2461398B`). The run reached later map/device and window/DC
  activity (`mapinfo.bin`, repeated `UID1:`, an additional child window, and
  `GetDC`), but `target\send_notify_render.txt` still reports no render
  milestones and the framebuffer body has only one nonzero byte, so this is not
  UI success. Full scheduler-owned cross-thread
  `SendMessageW`/`SendMessageTimeout` blocking and receiver-context execution
  remain open.
- GWE now has a separate receiver-side sent-message queue in front of the
  posted-message and synthetic-paint paths. `GetMessageW`/`PeekMessageW(PM_REMOVE)`
  retrieval can now take a sent message first, mark the receiver as
  `InSendMessage`, expose `QS_SENDMESSAGE`, and preserve `GetMessageSource`
  as a send source. Focused coverage:
  `ce::gwe::tests::sent_messages_are_retrieved_before_posts_and_mark_receiver_send_state`,
  `coredll_raw_message_ipc_state_tracks_source_send_and_timeout`, the full raw
  GWE suite, and full `cargo test --features unicorn,trace,win32-desktop`. A
  bounded mounted host/tap probe wrote `target\sent_queue_summary.txt`,
  `target\sent_queue_files.txt`, `target\sent_queue_render.txt`,
  `target\sent_queue_milestones.txt`, and `target\sent_queue_probe.ppm`; it
  stopped at the familiar 10 s resource frontier (`pc=0x00b4bc1c`) with
  `host_read=4221/495853B`, `heap_live=5948/2767663B`, no render milestones,
  and an all-zero framebuffer body. Sender blocking/resume and cross-thread
  guest WNDPROC receiver-context execution are still open.
- Cross-thread raw `SendNotifyMessageW` now uses the receiver-side sent-message
  queue instead of the posted-message queue, matching the CE no-wait send
  split more closely. The receiver observes `QS_SENDMESSAGE`, retrieves the
  notify as a send source, enters `InSendMessage`, and raw `DispatchMessageW`
  now clears that receiver send context after dispatch; the Unicorn
  `DispatchMessageW` guest-WNDPROC callout path also clears receiver send depth
  on return. Focused coverage extends
  `coredll_raw_send_notify_message_is_async_across_threads` and the full raw
  GWE suite; full `cargo test --features unicorn,trace,win32-desktop` passes.
  A bounded mounted host/tap probe wrote
  `target\send_notify_sent_queue_summary.txt`,
  `target\send_notify_sent_queue_files.txt`,
  `target\send_notify_sent_queue_render.txt`,
  `target\send_notify_sent_queue_milestones.txt`, and
  `target\send_notify_sent_queue_probe.ppm`; it stopped at the familiar 10 s
  resource frontier (`pc=0x00339d8c`) with `host_read=4221/499832B`,
  `heap_live=5948/2767663B`, no render milestones, and an all-zero framebuffer
  body. Scheduler-owned sender blocking/resume and true cross-thread guest
  receiver-context execution for `SendMessageW`/`SendMessageTimeout` remain
  open.
- GWE sent messages now carry CE `SendMsgEntry_t`-style transaction state:
  sender thread id, receiver thread id, flags, timeout metadata, WNDPROC
  result, active receiver send stack, and result-ready/receiver-terminated
  completion. Receiver retrieval still exposes the normal `Message` shape, but
  raw `DispatchMessageW` now completes the active sent transaction with the
  receiver result, and destroying a target marks queued synchronous sends as
  receiver-terminated/result-ready instead of losing the sender-side state.
  Compact Unicorn summaries now include `gwe=send:...` counters when send
  transactions occur. Focused coverage:
  `ce::gwe::tests::synchronous_sent_message_records_result_for_sender`,
  `ce::gwe::tests::destroying_target_marks_queued_sync_send_receiver_terminated`,
  `coredll_raw_dispatch_completes_queued_cross_thread_send`, the full raw GWE
  suite, and full `cargo test --features unicorn,trace,win32-desktop`. A
  bounded mounted host/tap probe wrote
  `target\sync_send_transaction_summary.txt`,
  `target\sync_send_transaction_files.txt`,
  `target\sync_send_transaction_render.txt`,
  `target\sync_send_transaction_milestones.txt`, and
  `target\sync_send_transaction_probe.ppm`; it stopped at the familiar 10 s
  resource/DIB frontier (`pc=0x00b4bc24`) with
  `host_read=4221/495853B`, `heap_live=5948/2767663B`, no render milestones,
  and an all-zero framebuffer body. This is sender/receiver bookkeeping for
  the blocking-send port, not live cross-thread guest scheduling yet.
- Sent-message timeout metadata is now active instead of just stored. GWE
  expires non-result-ready sent transactions using wrapping tick arithmetic
  from the message timestamp plus timeout, marks `SMF_TIMEOUT|SMF_RESULT_READY`,
  removes the entry from the receiver queue, and leaves a zero result for the
  sender side to consume. `CeKernel::pump_timers_to_gwe` runs the expiry pass
  before delivering timers/messages, so raw `GetMessageW` observes expired
  sends as gone instead of dispatchable. Compact Unicorn summaries now include
  a send-timeout counter when the path is exercised. Focused coverage:
  `ce::gwe::tests::timed_out_sent_message_is_removed_from_receiver_queue`,
  `coredll_raw_get_message_expires_timed_out_cross_thread_send`, the full raw
  GWE suite, and full `cargo test --features unicorn,trace,win32-desktop`.
  A bounded mounted host/tap probe wrote
  `target\send_timeout_expiry_summary.txt`,
  `target\send_timeout_expiry_files.txt`,
  `target\send_timeout_expiry_render.txt`,
  `target\send_timeout_expiry_milestones.txt`, and
  `target\send_timeout_expiry_probe.ppm`; it stopped at the familiar 10 s
  resource frontier (`pc=0x00339c3c`) with `host_read=4219/486039B`,
  `heap_live=5948/2767663B`, no render milestones, and an all-zero framebuffer
  body. Live sender parking/resume and receiver-context guest execution remain
  open.
- Raw `SendMessageTimeout(..., timeout=0)` across threads now uses the same
  CE-style sent-message transaction path, then immediately expires through the
  GWE timeout logic instead of running the receiver shortcut. The caller gets a
  zero return, the optional result pointer is left untouched, and the receiver
  queue is not left with a stale sent message. Focused coverage:
  `coredll_raw_send_message_timeout_zero_cross_thread_expires_transaction` and
  eVC fixture `168_sendmessage_timeout_zero_cross_thread`.
- Scheduler/GWE ownership now includes send-reply waiters keyed by sent-message
  transaction id. `SchedulerBlockedWaitKind::SendMessage` tracks blocked
  senders separately from object/message/serial waits, compact monitor
  summaries include send-reply signal/candidate counters, and kernel/GWE
  transitions enqueue those waiters when a sent message completes normally,
  times out, or is receiver-terminated by target HWND destruction. Focused
  coverage: `scheduler_queues_send_reply_waiters_by_send_id` and
  `send_message_transitions_queue_scheduler_reply_wait_candidates`. Unicorn
  same-process cross-thread sends now also park the saved sender MIPS context
  in the shared `BlockedWaitThread` list with `BlockedWaitKind::SendMessage`;
  WNDPROC returns and generic scheduler wake/resume both restore the sender
  from that blocked record and remove the scheduler waiter. Focused coverage:
  `emulator::unicorn::unicorn_tests::send_message_callout_enters_cross_thread_receiver_context`.
  Reentrant cross-thread scheduling and longer parked-send edge cases remain
  open.
- Unicorn raw `SendMessageW`/`SendMessageTimeoutW` now has the first real
  cross-thread receiver-context guest WNDPROC path. Same-process sends to a
  guest WNDPROC create a GWE sent-message transaction, activate it on the
  receiver thread, park the sender MIPS registers/running-thread metadata as a
  scheduler-backed blocked wait, run the target WNDPROC with the receiver
  thread as the active CE thread, complete the sent transaction with the
  WNDPROC result on return, write the `SendMessageTimeoutW` result pointer when
  supplied, and restore the sender context/result from the blocked wait instead
  of falling back to host-side default behavior. Focused coverage:
  `emulator::unicorn::unicorn_tests::send_message_callout_enters_cross_thread_receiver_context`,
  `emulator::unicorn::unicorn_tests::send_message_receiver_context_requires_same_process_guest_wndproc`,
  the full raw GWE suite, and full
  `cargo test --features unicorn,trace,win32-desktop`. A bounded mounted
  host/tap probe wrote `target\receiver_context_send_summary.txt`,
  `target\receiver_context_send_files.txt`,
  `target\receiver_context_send_render.txt`,
  `target\receiver_context_send_milestones.txt`, and
  `target\receiver_context_send_probe.ppm`; the final rerun after the return-
  path fix stopped at `pc=0x00b4bc24` after 10 s with small file/RSS counters
  (`host_open=7`, `host_read=4221/495853B`,
  `heap_live=5948/2767663B`). The milestones show real window/resource/DIB
  activity including `CreateDIBSection` for 800x160 and 800x320 resources, but
  the render trace still reports no iNavi render milestones and the framebuffer
  body has zero nonzero bytes, so this is not visible UI success.
- GWE queue-status tracking now distinguishes current input from newly changed
  input. `GetQueueStatus` reports CE-style high-word current bits and low-word
  changed bits, clearing only the requested changed bits after observation.
  Window creation/show/invalidation, posts, timers, key/mouse messages, and
  sent-message queueing now mark the appropriate `QS_*` changed bits. Raw
  `MsgWaitForMultipleObjectsEx` now uses those changed bits by default and only
  treats already-queued messages as immediately wakeable when
  `MWMO_INPUTAVAILABLE` is supplied. The raw and Unicorn
  `MsgWaitForMultipleObjectsEx` paths also no longer interpret desktop flag
  bit `0x0001` as `MWMO_WAITALL`: CE `winuser.h` only defines
  `MWMO_INPUTAVAILABLE`, and CE MFC emulates wait-all outside the OS call.
  Focused coverage:
  `ce::gwe::tests::queue_status_low_word_tracks_changed_bits_until_observed`,
  `coredll_raw_msgwait_requires_new_input_unless_inputavailable`,
  `coredll_raw_msgwait_ignores_desktop_waitall_flag_bit_on_ce`, the full raw
  GWE/raw-kernel suites, and full
  `cargo test --features unicorn,trace,win32-desktop`. A bounded mounted
  host/tap probe wrote `target\queue_status_msgwait_summary.txt`,
  `target\queue_status_msgwait_files.txt`,
  `target\queue_status_msgwait_render.txt`,
  `target\queue_status_msgwait_milestones.txt`, and
  `target\queue_status_msgwait_probe.ppm`; it stopped at `pc=0x00339d84`
  after 10 s with small file/RSS counters (`host_open=7`,
  `host_read=4221/495853B`, `heap_live=5948/2767663B`), no render milestones,
  and an all-zero framebuffer body. This is scheduler/GWE fidelity, not visible
  UI progress.
- `PostQuitMessage` now uses queue-owned quit state instead of posting
  `WM_QUIT` as an ordinary queued message. This follows CE `cmsgque.h`
  evidence (`msgqfGotWMQuitMessage`, `m_nExitCode`, and `mgefQuitMsg`):
  `GetMessageW`/`PeekMessageW` synthesize `WM_QUIT` from the thread queue state
  even when the caller supplies a nonmatching HWND or message filter, and the
  quit state still participates in `QS_POSTMESSAGE` current/changed status.
  Focused coverage:
  `ce::gwe::tests::post_quit_state_ignores_window_and_message_filters`,
  `coredll_raw_post_quit_uses_queue_state_not_filtered_post`, the full raw GWE
  suite, and full `cargo test --features unicorn,trace,win32-desktop`. A
  bounded mounted host/tap probe wrote
  `target\post_quit_queue_state_summary.txt`,
  `target\post_quit_queue_state_files.txt`,
  `target\post_quit_queue_state_render.txt`,
  `target\post_quit_queue_state_milestones.txt`, and
  `target\post_quit_queue_state_probe.ppm`; it stopped at `pc=0x00339da0`
  after 10 s with stable file/RSS counters (`host_open=7`,
  `host_read=4221/495853B`, `heap_live=5948/2767663B`), no render milestones,
  and an all-zero framebuffer body. This is modal-loop/message-queue fidelity,
  not visible UI progress.
- Raw `GetMessageWNoWait` (ordinal 863) is now wired to the GWE queue instead
  of falling through generic ordinal handling. The CE GWE API set exposes it
  beside `GetMessageW` with the same `MSG*, HWND, min, max` signature but no
  blocking wait, so v3 now uses the same nonblocking filtered retrieval,
  including posted-message removal and queue-owned `WM_QUIT` synthesis. Focused
  coverage: `coredll_raw_get_message_no_wait_uses_gwe_queue_without_blocking`,
  the full raw GWE suite, and full
  `cargo test --features unicorn,trace,win32-desktop`. A bounded mounted
  host/tap probe wrote `target\get_message_nowait_summary.txt`,
  `target\get_message_nowait_files.txt`,
  `target\get_message_nowait_render.txt`,
  `target\get_message_nowait_milestones.txt`, and
  `target\get_message_nowait_probe.ppm`; it stopped at `pc=0x00339d88` after
  10 s with stable file/RSS counters (`host_open=7`,
  `host_read=4221/499832B`, `heap_live=5948/2767663B`), no render milestones,
  and an all-zero framebuffer body. This is another message-pump fidelity
  slice, not visible UI progress.
- Raw `GetMessagePos` and `GetMessageQueueReadyTimeStamp` are now backed by
  GWE message metadata instead of timer-only or zero-ish syscall stubs. This
  follows CE `cmsgque.h` fields on `PostedMsgQueueEntry_t` (`time` and
  `MousePosAtPost`) plus the queue `m_ReadyTimeStamp`: posted mouse messages
  preserve their screen mouse position separately from client-coordinate
  `lParam`, remote tap injection supplies that screen position, message
  retrieval records the last message position for the receiving thread, and
  ready timestamps update when posted, sent, or quit-state work makes a queue
  ready. Focused coverage:
  `coredll_raw_message_pos_and_ready_timestamp_follow_pulled_queue_entry`, the
  full raw GWE suite, and full
  `cargo test --features unicorn,trace,win32-desktop`. A bounded mounted
  host/tap probe wrote `target\message_metadata_summary.txt`,
  `target\message_metadata_files.txt`,
  `target\message_metadata_render.txt`,
  `target\message_metadata_milestones.txt`, and
  `target\message_metadata_probe.ppm`; it stopped at `pc=0x00895bfc` after
  10 s with stable counters (`host_open=9`,
  `host_read=4225/486559B`, `heap_live=5621/2459146B`) and reached
  `mapinfo.bin`/`iNaviData` file activity. Render milestones were still none
  and the framebuffer body had only one nonzero byte, so this is queue
  metadata fidelity, not visible UI progress.
- Raw `SetDlgItemInt` and `GetDlgItemInt` are now routed through the dialog
  child-window text model instead of generic ordinal fallback. This follows the
  CE dialog API surface in `GWE\INC\dlgmgr.h` and `gweapiset1.hpp`: dialog item
  lookup resolves the child by control ID, `SetDlgItemInt` stores signed or
  unsigned decimal text on that child, and `GetDlgItemInt` parses the child
  text with the caller's signed/unsigned mode and writes the optional success
  flag. Focused coverage:
  `coredll_raw_dialog_item_int_uses_child_window_text_and_ok_flag`, the full
  raw GWE suite, and full
  `cargo test --features unicorn,trace,win32-desktop`. A bounded mounted
  host/tap probe wrote `target\dialog_int_summary.txt`,
  `target\dialog_int_files.txt`, `target\dialog_int_render.txt`,
  `target\dialog_int_milestones.txt`, and `target\dialog_int_probe.ppm`; it
  stopped at `pc=0x00b4bc44` after 10 s with stable counters
  (`host_open=7`, `host_read=4221/495853B`,
  `heap_live=5948/2767663B`), reached RSImage/DIB resource work, but had no
  render milestones and an all-zero framebuffer body. This is dialog/control
  surface fidelity, not visible UI progress.
- Raw `WindowFromPoint` and `ChildWindowFromPoint` now reach the GWE hit-test
  model instead of generic ordinal fallback. This follows the CE window-manager
  API surface in `winuser.h`, `gweapiset1.hpp`, and `window.hpp`: by-value
  `POINT` arguments are decoded at the raw syscall boundary, top-level
  hit-testing walks visible/enabled windows for the caller's thread, and
  `ChildWindowFromPoint` converts parent-client coordinates through the
  existing client/screen mapping before returning the deepest visible child or
  the parent when no child contains the point. Focused coverage:
  `coredll_raw_window_from_point_hits_visible_thread_windows`, the full raw
  GWE suite, and full
  `cargo test --features unicorn,trace,win32-desktop`. A bounded mounted
  host/tap probe wrote `target\window_from_point_summary.txt`,
  `target\window_from_point_files.txt`,
  `target\window_from_point_render.txt`,
  `target\window_from_point_milestones.txt`, and
  `target\window_from_point_probe.ppm`; it stopped at `pc=0x6002278c` after
  10 s with stable counters (`host_open=9`,
  `host_read=4225/486559B`, `heap_live=5624/2461398B`), reached later
  map/device file activity, but had no render milestones and only one nonzero
  framebuffer byte. This is input/hit-test fidelity, not visible UI progress.
- Raw/kernel `DestroyWindow` now routes through the kernel window lifecycle
  instead of deleting GWE HWND state directly. Virtual windows carry a CE
  `CWindow::fSentWmDestroy`-style bit, raw/kernel destroy sends `WM_DESTROY`
  through `SendMessageW` before final cleanup, and the default `WM_CLOSE`
  shortcut records the same destroy-message observation before deleting the
  target. Focused coverage extends raw parent/child destroy cleanup and
  `SendNotifyMessageW(..., WM_CLOSE, ...)` delivery. A bounded mounted host/tap
  probe wrote `target\destroy_window_lifecycle_summary.txt`,
  `target\destroy_window_lifecycle_files.txt`,
  `target\destroy_window_lifecycle_render.txt`,
  `target\destroy_window_lifecycle_milestones.txt`, and
  `target\destroy_window_lifecycle_probe.ppm`; it stopped at 10 s with
  `pc=0x00b2171c`, small file/RSS counters
  (`host_read=4223/486055B`, `heap_live=5868/2705123B`), reached RSImage DIB
  creation, but had no render milestones and an all-zero framebuffer body.
  automatic OS-side `WM_NCDESTROY` synthesis, exact child destroy-message
  ordering, and guest-WNDPROC receiver-context scheduling remain open, so this
  is another lifecycle fidelity slice rather than UI success.
- `WM_NCDESTROY` is now represented as an explicit window lifecycle message
  when it is actually delivered, matching the CE MFC source where
  `AfxWndProc` post-processes `WM_DESTROY` by sending `WM_NCDESTROY` itself.
  GWE windows now track `nc_destroy_message_sent`; raw `SendMessageW` records
  both `WM_DESTROY` and `WM_NCDESTROY`, and Unicorn direct guest-WNDPROC
  returns record those lifecycle messages before final destroy cleanup. Focused
  coverage: `coredll_raw_send_message_records_nc_destroy_lifecycle`, plus the
  full raw GWE suite. A bounded mounted host/tap probe wrote
  `target\nc_destroy_lifecycle_summary.txt`,
  `target\nc_destroy_lifecycle_files.txt`,
  `target\nc_destroy_lifecycle_render.txt`,
  `target\nc_destroy_lifecycle_milestones.txt`, and
  `target\nc_destroy_lifecycle_probe.ppm`; it stopped at 10 s with
  `pc=0x00b4bc24`, small file/RSS counters
  (`host_read=4221/495853B`, `heap_live=5948/2767663B`), no render
  milestones, and an all-zero framebuffer body. Automatic synthesis is not
  added at the GWE boundary because the CE MFC source explicitly fakes the
  message in MFC; full child destroy ordering remains open.
- CE/MFC destroy lifecycle now uses the CE fake `WM_NCDESTROY` value
  `WM_APP - 1` (`0x7fff`) instead of the desktop value `0x0082`, matching
  `atlosapice.h`, and virtual windows carry a CE `fBeingDestroyed`-style
  transient bit during `DestroyWindow` guest-WNDPROC callouts. A reentrant
  `DestroyWindow(hwnd)` seen while the subtree is already in this state now
  returns through the import bridge without finalizing the HWND twice, and the
  empty-callout path restores `PC`/`RA` as well as `v0`. Window trace snapshots
  include the `destroying` state so mounted probes can distinguish in-flight
  destroy from final dead HWNDs. Focused coverage:
  `being_destroyed_window_remains_valid_until_final_destroy` and the raw
  `coredll_raw_send_message_records_nc_destroy_lifecycle` fixture, now also
  asserting `WM_NCDESTROY == 0x7fff`. Mounted validation
  `target\destroy_lifecycle_current_*` still crashes after `DestroyWindow`
  returns:
  the app reaches `pc=0x0002c264(image:iNavi.exe+0x1c264)` with `a0=0` and
  faults on `lw $10,0($4)`. Stop-PC probes show the global slot at
  `+0x10ec` is initialized to `0x3005bda0` and remains valid at
  `0x0002bf30`, then becomes null before the unguarded path at `0x0002c264`
  while state getter `0x22998` returns `state[0x8a] == 5` and
  `state[0x120] == 0`. The fresh final window dump shows the relevant
  destroyed subtree as `destroying=false dead=true`, so finalization completed
  before the guest continuation dereference. This is not fixed by the
  destroy-lifecycle slice; the next work is to trace the generic state/slot
  clear path, not to force guest state.
- Raw/kernel parent `DestroyWindow` now delivers `WM_DESTROY` to live
  descendants before the parent and before final GWE cleanup, with a lightweight
  lifecycle order counter on virtual windows to prove the child-first sequence
  in fixtures. Focused coverage extends
  `coredll_raw_destroy_parent_invalidates_children_and_purges_messages` to a
  parent/child/grandchild tree and asserts grandchild-before-child-before-parent
  `WM_DESTROY` observation plus queued-message purge. A bounded mounted
  host/tap probe wrote `target\child_destroy_lifecycle_summary.txt`,
  `target\child_destroy_lifecycle_files.txt`,
  `target\child_destroy_lifecycle_render.txt`,
  `target\child_destroy_lifecycle_milestones.txt`, and
  `target\child_destroy_lifecycle_probe.ppm`; it stopped at 10 s with
  `pc=0x00339c3c`, small file/RSS counters
  (`host_read=4221/499832B`, `heap_live=5948/2767663B`), no render milestones,
  and an all-zero framebuffer body. The Unicorn direct guest-WNDPROC
  `DestroyWindow` callout still sends only the target window's guest
  `WM_DESTROY` before final cleanup; chaining guest child-WNDPROC destroy
  callouts remains open.
- Unicorn direct guest-WNDPROC `DestroyWindow` now follows the same child-first
  lifecycle model before final root cleanup. The callout planner walks the
  virtual descendant tree, records default/non-guest `WM_DESTROY` observations,
  chains guest WNDPROC `WM_DESTROY` callbacks in descendant-before-parent order
  through the existing return stub, and destroys the root subtree only after the
  final guest callback returns. Focused coverage:
  `emulator::unicorn::unicorn_tests::destroy_wndproc_callouts_are_guest_child_first`,
  plus the full raw GWE suite and full
  `cargo test --features unicorn,trace,win32-desktop`. A bounded mounted
  host/tap probe wrote `target\guest_destroy_chain_summary.txt`,
  `target\guest_destroy_chain_files.txt`,
  `target\guest_destroy_chain_render.txt`,
  `target\guest_destroy_chain_milestones.txt`, and
  `target\guest_destroy_chain_probe.ppm`; it stopped at the 10 s wall limit
  with `pc=0x600c9aec`, small file/RSS counters
  (`host_read=4226/500100B`, `heap_live=5620/2459096B`), no render milestones,
  and an all-zero framebuffer body. This closes the specific guest child
  destroy-callout gap, not the broader UI/render frontier.
- Repository started with `RULES.md`, `regs.json`, and `serial_devices.json`.
- `regs.json` contains the registry snapshot used to seed the CE registry model.
- `serial_devices.json` contains enabled guest devices including `COM7:`, `COM3:`,
  `UID1:`, `PIC1:`, `BTN1:`, `LSD1:`, `MFS1:`, `SMB1:`, `CAM1:`, and `TWV1:`.
- Initial Rust crate scaffold added for a CE-ish base:
  - CE-style registry API model backed by `regs.json`
  - device namespace
  - kernel object handle table
  - GWE windows and message queue
  - timer bookkeeping
  - waveOut-style audio state
  - memory map validation
  - Unicorn MIPS adapter boundary
  - PE32 image parsing for headers, sections, imports, exports, relocations, and
    mapped image bytes
  - remote-control API state for touch/key input, GPS/NMEA serial injection, IMU
    state, pause/resume, status JSON, logs, and audio chunks
- Source references are recorded in `SOURCE_REFERENCES.md` for CE registry,
  GWE queue, waveOut exports, and MFC message pump behavior.
- Rust smoke tests cover bootstrapping registry/device JSON backing plus basic
  registry, device, GWE/message, timer, audio, handle, and memory-map behavior.
- Added a virtual Win32/CE API facade on `CeKernel` for:
  - `CreateFileW`-style file/device opens
  - `ReadFile`/`WriteFile`
  - `DeviceIoControl`
  - `CloseHandle`
  - `CreateEventW`/`SetEvent`/`ResetEvent`/`WaitForSingleObject`
  - `CreateMutexW`/`ReleaseMutex`
  - `CreateWindowExW`, `PostMessageW`, `SendMessageW`, `GetMessageW`, and
    single-step message pumping
  - `SetTimer`/`KillTimer`
  - `waveOutOpen`/`waveOutWrite` plus pause/restart/reset/volume helpers
- Host-backed file opens are contained under a configurable file root and reject
  parent-directory escapes.
- The host-backed file namespace now has a static CE mount table containing
  `SDMMC Disk`. `FindFirstFileW("\\")` enumerates that mount prefix, exact
  `\SDMMC Disk` returns directory metadata, and `--mount-config mounts.toml`
  binds mount contents such as `D:\INAVI_Emulator\INAVI` to host storage for
  file opens/finds beneath the mount.
- Added checked-in Rust COREDLL ordinal definitions in
  `src/ce/coredll_ordinals.rs`. Runtime dispatch now uses Rust `ORD_*`
  constants, a static export table, and an ordinal `match`, with 1,752 export
  entries including CRT/math additions.
- Added a COREDLL export table parser for CE `.def` source evidence and
  validation work; it is not the runtime ordinal source.
- Added a COREDLL dispatcher that routes implemented exports to the virtual
  Win32/CE framework and reports unresolved or unimplemented ordinals explicitly.
- Added a simple `cemath` subsystem for common CE CRT math exports and MIPS
  helper routines, including `sqrt`, `pow`, `fmod`, `div`, `ldiv`, `__ll_div`,
  `__ll_mul`, soft-float add/sub/mul/div, conversion, and compare helpers.
- Raw COREDLL import-trap dispatch now routes the observed MIPS soft-float
  conversion/arithmetic helpers (`__litodp`, `__ultodp`, `__fp*`, `__dp*`,
  `__fptodp`, `__dptofp`) and CRT double math exports (`sqrt`, `pow`, `fmod`,
  trigonometric/log/rounding unary helpers) through the `cemath` backend using
  the verified low-word/high-word MIPS double register ABI.
- COREDLL ordinal work is now split by subsystem in code. Every static export
  can produce an ordinal plan entry with subsystem ownership and
  implemented-vs-stubbed status; raw ordinal dispatch preserves raw arguments
  and routes unresolved semantics through subsystem-owned stub policies instead
  of a single generic unimplemented bucket.
- COREDLL raw ordinal dispatch has a stateful guest-memory path for the first
  CE-sourced kernel/thread/time/sync tranche:
  - CE `CRITICAL_SECTION` layout and fast-path state changes for
    `InitializeCriticalSection`, `EnterCriticalSection`,
    `TryEnterCriticalSection`, `LeaveCriticalSection`, and
    `DeleteCriticalSection`
  - `InterlockedTestExchange`, `InterlockedIncrement`,
    `InterlockedDecrement`, `InterlockedExchange`,
    `InterlockedExchangeAdd`, and `InterlockedCompareExchange`
  - per-thread `TlsGetValue`, `TlsSetValue`, `GetLastError`, and
    `SetLastError`, plus process-style `TlsCall` allocation/free for CE TLS
    slots `4..63`
  - `Sleep`, `GetTickCount`, `EventModify`, `WaitForSingleObject`, and
    `CloseHandle`
  - CE heap/local/virtual-memory ordinals: `GetProcessHeap`, `LocalAlloc`,
    `LocalReAlloc`, `LocalSize`, `LocalFree`, `HeapCreate`, `HeapDestroy`,
    `HeapAlloc`, `HeapReAlloc`, `HeapSize`, `HeapFree`, `HeapValidate`,
    `VirtualAlloc`, and `VirtualFree`, plus the remote/in-process local and
    remote heap variants routed through the virtual heap model
  - raw file ordinals: `CreateFileW`, `ReadFile`, and `WriteFile` now marshal
    UTF-16 paths, guest byte buffers, and transferred-byte output pointers;
    `SetFilePointer`, `GetFileSize`, and `FlushFileBuffers` track cursor,
    size, high-word output, and flush behavior for host-backed files;
    `FindFirstFileW`/`FindClose` marshal `WIN32_FIND_DATAW` and enumerate the
    static CE mount table at `\`
  - CE/MFC-style HWND geometry/state ordinals: raw `CreateWindowExW`,
    `DestroyWindow`, `ShowWindow`, `UpdateWindow`, `EnableWindow`,
    `IsWindow`, `IsWindowEnabled`, `IsWindowVisible`, `GetParent`,
    `GetWindow`, `GetDesktopWindow`, `SetFocus`, `GetFocus`, `SetWindowTextW`,
    `GetWindowTextW`, `GetWindowTextLengthW`, `GetClassNameW`,
    `SetWindowLongW`, `GetWindowLongW`, `SetWindowPos`, `MoveWindow`,
    `GetWindowRect`, `GetClientRect`, `ClientToScreen`, `ScreenToClient`,
    `MapWindowPoints`, `RegisterClassW`, `GetClassInfoW`, `FindWindowW`,
    `GetCursorPos`, `GetActiveWindow`, and `MessageBoxW`
  - CE/MFC-style message-pump ordinals: raw `GetMessageW`, `PeekMessageW`,
    `PostMessageW`, `SendMessageW`, `DispatchMessageW`, `TranslateMessage`,
    and `DefWindowProcW` marshal `MSG` structs and queue state
  - CE/MFC-style paint/update ordinals: `InvalidateRect`, `ValidateRect`,
    `GetUpdateRect`, `BeginPaint`, and `EndPaint` track pending window update
    state, marshal CE `RECT`/`PAINTSTRUCT` data, and synthesize `WM_PAINT`
    through the message pump for visible invalidated windows
  - unplugged multimedia adapter ordinals: raw `waveOutGetNumDevs`,
    `waveOutOpen`, `waveOutPrepareHeader`, `waveOutUnprepareHeader`,
    `waveOutWrite`, `waveOutPause`, `waveOutRestart`, `waveOutReset`,
    `waveOutClose`, `waveOutGetVolume`, `waveOutSetVolume`,
    `waveOutGetPosition`, `waveOutGetPitch`, `waveOutSetPitch`,
    `waveOutGetPlaybackRate`, `waveOutSetPlaybackRate`, `waveOutGetID`,
    `waveOutGetDevCaps`, and `waveOutGetErrorText` marshal CE
    `WAVEFORMATEX`, `WAVEHDR`, `MMTIME`, and output pointers without binding to
    host playback
  - resource ordinals: `FindResourceW`/`FindResource`, `LoadResource`, and
    `SizeofResource`, plus `LoadStringW` buffer copying/null termination for
    registered virtual strings
- GWE window state now tracks CE-style whole-window and client `RECT`s in screen
  coordinates, parent HWNDs, title/class text, enabled/visible/focus state, and
  window-long slots for MFC layout and subclassing paths.
- Added a virtual CE memory subsystem for process/custom heaps, local
  allocations, allocation sizes, frees, and page-granular virtual allocations.
- Added resource and COM subsystem state:
  - resources map `(module, name, type)` to an HRSRC-like handle, data pointer,
    byte size, and string resources keyed by module/id
  - COM tracks per-thread apartment initialization depth, registered class
    factories, and virtual object handles
- PE parsing now validates DOS/NT signatures, reads COFF and PE32 optional
  headers, tracks all 16 standard data directories, maps RVAs through section
  headers, parses import descriptors/thunks by name or ordinal, parses export
  functions, parses base relocation blocks, and can build a zero-filled mapped
  image buffer. Relocation application now refuses to move relocation-stripped
  images and applies supported CE/MIPS base relocations for SDK DLL images.
- PE parser smoke tests build a synthetic MIPS R4000 PE32 image with imports,
  exports, relocations, and mapped section data.
- Added a Rust `CeRemote` subsystem based on the prior remote server API shape:
  it queues touch/key events, injects serial/NMEA/GPS data, stores IMU state,
  tracks pause/resume, exposes remote status JSON, holds recent log lines, and
  manages remote audio client/chunk state.
- `CeKernel` owns `CeRemote` and can drain queued remote touch/key events into
  the GWE message queue.
- Audio output has two sink classes:
  - `AudioSinkRegistry` lets host, websocket, and debug sinks register behind a
    shared PCM submission/flush contract.
  - `HostAudioSink` is an explicit host adapter boundary with an unplugged
    default and a Windows `winmm` constructor gated through the `windows` crate.
  - `AudioSystem` owns the sink registry; raw `waveOutWrite` now copies guest
    `WAVEHDR` PCM into registered sinks, and `main` registers the Windows host
    sink on Windows hosts.
  - `WebSocketAudioSink` owns per-client cursors, PCM chunk sequencing, PTS,
    queue limits, and flush-marked chunks for the remote audio path;
    middle-joined websocket devices attach at the host audio timeline and
    receive a trimmed partial chunk when the host is already inside a retained
    chunk.
- Debug builds include `LoggingAudioSink`, which records/logs PCM submissions
  and flush hints for short-audio debugging.
- Added a generic virtual framebuffer boundary:
  - `Framebuffer` describes a byte-addressable surface with dimensions, stride,
    pixel format, dirty rectangles, and mutable pixel storage without depending
    on Windows names or handles
  - `VirtualFramebuffer` provides an in-memory implementation, defaults to an
    800x480 RGB565 primary surface, and can write a temporary PPM dump
  - `main` owns the virtual framebuffer, updates the remote/input framebuffer
    size from it, and passes it into the Unicorn execution boundary
- Added generic virtual presentation and desktop boundaries:
  - `Presenter` describes how to present any `Framebuffer` trait object, and
    `VirtualPresenter` snapshots framebuffer pixels plus dirty rectangles
  - `Desktop` describes create/move/remove/window-enumeration operations, and
    `VirtualDesktop` provides an in-memory virtual implementation
  - These interfaces are host-side architecture boundaries only; they are not
    CE/MFC behavior and do not create a host window yet
- Unicorn launch prep is wired:
  - parsed PE images can be mapped into the Unicorn memory plan
  - `--dll-search-dir` can load SDK DLL images such as `mfcce400.dll`; the main
    relocation-stripped EXE remains at its preferred base while relocatable DLLs
    are moved when their preferred base overlaps
  - COREDLL, winsock, and OLE import slots are patched to shim trap addresses
    when no loaded DLL export resolves them; `commctrl.dll` is expected to come
    from `--dll-search-dir` and patch to the mapped DLL exports instead of an
    emulator common-controls trap
  - MFC imports are not emulated by external stubs; they must resolve to loaded
    SDK DLL exports such as `mfcce400.dll`
  - external imports can resolve to loaded DLL exports before falling back to
    module-owned traps
  - COREDLL traps decode MIPS `a0`-`a3`, dispatch through the raw ordinal
    dispatcher, write `v0`, and retain a debug snapshot with PC/RA/SP/v0/v1/
    a0-a3/t9 plus memory-fault details on run failure
  - guest heap pages are mapped as a CE heap arena for APIs that allocate and
    populate memory during the same import call
  - non-COREDLL supported DLLs other than MFC/loaded `commctrl.dll` currently
    use module-owned launch stubs with debug logs, not final API semantics;
    WINSOCK dispatch is now owned by `src/winsock.rs`
- SDK CE 4.2 Mipsii COREDLL ordinal evidence from `coredll.lib` is now captured
  for the launch-demanded CRT ordinals: `_wcsdup`, `wcsrchr`, `_wcsnicmp`,
  `malloc`, `memcpy`, `memset`, operator `new`, `swprintf`, `printf`, `free`,
  `longjmp`, and `_setjmp`.
- Launch-demanded CE 4.2 CRT raw helper bodies now live in `src/ce/crt.rs`,
  with COREDLL keeping ordinal dispatch ownership and delegating the actual
  CRT memory/string routines to that module.
- CE wide printf handling now matches the observed CE/MFC wide-format path:
  `wsprintfW`, `wvsprintfW`, CRT `swprintf`, and CRT `vswprintf` all treat
  default `%s` as a wide string, while `%hs` remains the explicit narrow-string
  form and `%ls` forces wide. Focused raw ordinal tests cover the wide default
  and narrow override. In the real mounted iNavi run, this fixed the
  `CString::Format("%s", module_path)` truncation that produced `\res`; the
  app now opens and repeatedly reads `\SDMMC Disk\INavi\res\values.dat`.
- The bounded Unicorn launch with SDK `mfcce400.dll` now progresses past the
  previous unmapped-write failures and stops at a null function-pointer call from
  the main image destructor/function-pointer table around `0x0048f9d4`.
- A targeted Unicorn probe shows the failing destructor/function-pointer call is
  currently `jalr` from `0x0048f9d4` through slot `0x30002390` with value
  `0x00010000`, so the immediate launch failure is a low/invalid registered
  function pointer rather than a normal guest exit.
- A follow-up write probe showed the exit table slot `0x30002390` was populated
  by guest code at `0x0048f864` with callback `0x00019d7c`; that callback is
  valid app code. The remaining `pc=0` symptom is therefore the direct Unicorn
  entry lacking a CE loader/thread-exit return address after cleanup completes.
- Heap and local reallocation growth now move allocations and the raw COREDLL
  reallocation shims copy the old guest bytes to the new block. This fixed the
  launch-path overlap where a later guest `memcpy` corrupted the CRT/MFC exit
  callback table after `_onexit` table growth.
- Unicorn now decodes the old MIPS CE directly encoded `TerminateProcess`
  kernel thunk (`API set 2`, method `2`) from the caller instructions when the
  guest exits through that path.
- The Unicorn MIPS backend now rewrites direct `jal`, ordinary conditional
  branch, and branch-likely sites in executable PE sections into same-image
  trampolines. This works around the observed Unicorn control-flow fault where
  returning into MFC branch/call sites could fall into `pc=0`/reserved
  instruction state. Branch-likely delay-slot annulment, normal branch delay
  slots, and `jal` link/delay-slot behavior are covered by feature-gated tests.
- Raw `GetMessageW` now models CE/MFC blocking semantics for an empty queue in
  the Unicorn import path. It stops the bounded run with a
  `blocked_get_message` debug snapshot instead of returning `FALSE` to MFC and
  causing normal thread/application cleanup.
- GWE now tracks pending update regions for visible windows. `ShowWindow`,
  `SetWindowPos`, `MoveWindow`, and `InvalidateRect` can mark a window dirty,
  `PeekMessageW`/`GetMessageW` can synthesize `WM_PAINT`, and `BeginPaint` or
  `ValidateRect` clears the pending update state using the CE SDK
  `PAINTSTRUCT` layout.
- Unicorn now initializes the main PE entry context with CE/MFC-style WinMain
  arguments: `A0=hInstance`, `A1=0`, `A2` pointing at a real empty UTF-16
  command-line string, and `A3=1` (`SW_SHOWNORMAL`). The kernel also tracks the
  main process module base so `GetModuleFileNameW(hInstance, ...)` returns the
  configured CE module path instead of failing for nonzero `hModule`.
- The bounded Unicorn launch of `INavi.exe` with SDK `mfcce400.dll`,
  `--mount-config mounts.toml`, and the current debug binary now
  progresses past the previous empty-queue `GetMessageW` frontier. The latest
  debug trace shows `PeekMessageW` and `GetMessageW` returning a synthetic
  `WM_PAINT`, followed by `DispatchMessageW` entering the SDK MFC window
  procedure for class `solution_inavi` at `0x6004eba8`. A 30-second bounded
  run still had to be killed by the timeout and produced no host-visible GUI;
  this is not launch success.
- The framebuffer-plumbed bounded launch prints an attached 800x480 RGB565
  virtual framebuffer (`stride=1600`, `bytes=768000`) before entering CPU
  execution. The same 30-second target run still times out and has to be
  killed, so the optional framebuffer dump is only produced for runs that
  return normally or error through the emulator path. A non-CPU smoke run wrote
  `target\framebuffer-smoke.ppm` from the virtual framebuffer.
- `TlsCall` ordinal 520 now returns a real CE-style TLS allocation result. The
  short debug trace changed the first `TlsCall(TLS_FUNCALLOC, 0)` result from
  `0` to slot `4`; a 10-second debug run still does not reach later GDI/DC
  imports, and a 30-second non-debug run still times out after the normal
  startup/framebuffer/PE mapping output. This is progress in startup TLS setup
  rather than GUI success.
- Added bounded Unicorn instruction-count tooling via
  `--cpu-instruction-limit N`. A 10,000-instruction run now returns an emulator
  snapshot instead of needing an external kill, and a 100,000-instruction run
  reaches SDK MFC code around `0x6004f6a0..0x6004f8dc` with PC near
  `0x600dd98c`. This is diagnostic tooling only; default `0` keeps the previous
  unbounded CPU behavior.
- Unicorn debug snapshots now include a compact recent-import ring with module
  kind, ordinal/name, the first four arguments, stack pointer, and return value.
  This is diagnostic tooling only and is used to continue launch tracing without
  enabling high-volume import logs.
- CE `CallWindowProcW` ordinal 285 now enters nonzero guest window-procedure
  targets directly from the Unicorn import hook. This follows the SDK MFC
  `CWnd::DefWindowProc`/superclass path rather than adding emulator-side MFC
  stubs. The latest 1,000,000-instruction bounded launch shows the prior
  `CallWindowProcW(0x6000e530, hwnd=0x00020000, msg=0x363, ...)` call pending
  inside guest MFC code, followed by `DefWindowProcW`, `GetWindow`,
  `PeekMessageW`, and an intentional `blocked_get_message` snapshot on an empty
  queue.
- Unicorn raw `CreateWindowExW` now performs a source-backed create-time guest
  WNDPROC callout for windows with a registered class procedure. The hook
  marshals the CE SDK `CREATESTRUCTW` layout into guest heap memory, enters the
  created window's proc with `WM_CREATE`, and returns through a reserved
  import-page shim that restores the API return value to the HWND. This follows
  the MFC CE `PreCreateWindowEx`/`DefWindowProcEx` first-message path rather
  than adding any emulator-side MFC stub. A feature-gated test covers the
  `CREATESTRUCTW` field offsets.
- The latest 1,000,000-instruction bounded launch with SDK `mfcce400.dll` and
  `--mount-config mounts.toml` logs
  `CreateWindowExW guest WM_CREATE callout` for `hwnd=0x00020000`,
  `class="solution_inavi"`, `wndproc=0x000135cc`, and lParam pointing at the
  marshalled `CREATESTRUCTW`. It still reaches the first synthetic `WM_PAINT`,
  dispatches through SDK MFC, and ends at the intentional empty-queue
  `GetMessageW @861` `blocked_get_message` diagnostic. The trace still does not
  reach `BeginPaint`, `GetDC`, `GetWindowDC`, `SetTimer`, or `KillTimer`, so
  this is not GUI success.
- Re-running the 1,000,000-instruction bounded launch after the virtual
  presenter/desktop boundary addition still returns at the same SDK MFC
  message-pump frontier: `CallWindowProcW @285`, `DefWindowProcW @264`,
  `GetWindow @251`, `PeekMessageW @864`, and final `GetMessageW @861`
  `blocked_get_message`. This interface work did not change launch behavior or
  produce a visible GUI.
- The large `tests/basic_subsystems.rs` integration suite is now split into
  subsystem-focused files for broad smoke coverage, COREDLL dispatch, raw
  kernel/thread/sync, raw memory/file/find, raw GWE/resource/window behavior,
  and raw waveOut marshalling. Shared guest-memory helpers live under
  `tests/support/`.
- Raw `GetWindow` ordinal 251 now follows the CE SDK `GW_HWNDFIRST`,
  `GW_HWNDLAST`, `GW_HWNDNEXT`, `GW_HWNDPREV`, `GW_OWNER`, and `GW_CHILD`
  command values over the virtual HWND tree. It can enumerate top-level
  desktop children, child windows, and sibling windows for the MFC idle/modal
  traversal paths without adding host windows or app-specific behavior.
- A fresh 1,000,000-instruction bounded launch after raw `GetWindow` support
  still returns at the empty `GetMessageW @861` `blocked_get_message`
  diagnostic. The recent import ring now shows `GetWindow @251` called as
  `GetWindow(hwnd=0x00020000, relation=GW_CHILD)` and returning `0`, meaning
  the main window currently has no virtual child HWNDs for MFC idle/update
  traversal. This is still not GUI success.
- Virtual HWND show, move, and resize state changes now queue CE-style
  lifecycle messages through the kernel boundary: `WM_SHOWWINDOW`,
  `WM_WINDOWPOSCHANGED`, `WM_MOVE`, and `WM_SIZE`. Raw `ShowWindow`,
  `SetWindowPos`, and `MoveWindow` ordinals use that path so Unicorn import
  execution and subsystem tests see the same queue behavior.
- A corrected 1,000,000-instruction bounded launch using
  `D:\INAVI_Emulator\INAVI\INavi\iNavi.exe`, SDK `mfcce400.dll`, and
  `--mount-config mounts.toml` still reaches the same
  `GetMessageW @861` `blocked_get_message` frontier after SDK MFC dispatch.
  The run writes `target\framebuffer-launch.ppm`, but this remains diagnostic
  output only because no guest drawing/blit imports have produced GUI pixels.
- Unicorn debug snapshots now include a compact recent-message ring for
  `PeekMessageW`/`GetMessageW` results. The diagnostic confirmed that before
  create-time visible-window lifecycle queueing, the target pump only observed
  synthetic `WM_PAINT` and then an empty queue.
- GWE now normalizes visible top-level `CreateWindowExW` windows with
  default/zero dimensions to the virtual desktop client size and exposes first
  CE SDK `GetSystemMetrics` values from the same desktop model. Raw
  `CreateWindowExW` goes through the kernel boundary so visible creates queue
  `WM_SHOWWINDOW`, `WM_WINDOWPOSCHANGED`, and `WM_SIZE`; tests cover the raw
  visible zero-rect create case as `WM_SIZE(800,480)`.
- A 3,000,000-instruction bounded launch after visible-create lifecycle
  queueing now dispatches `WM_SHOWWINDOW`, `WM_WINDOWPOSCHANGED`,
  `WM_SIZE(lParam=0x01e00320)`, and then synthetic `WM_PAINT` for
  `hwnd=0x00020000`. It still reaches the intentional empty-queue
  `GetMessageW @861` `blocked_get_message` diagnostic after MFC
  `WM_IDLEUPDATECMDUI` (`0x0363`) handling, without reaching child HWND
  creation or GDI/DC drawing imports. This is progress in CE/GWE message
  semantics, not GUI success.
- The Unicorn import hook can now enter a registered guest WNDPROC for raw
  `SendMessageW` using the same guest-callout pattern as `DispatchMessageW` and
  `CallWindowProcW`. A follow-up 3,000,000-instruction bounded launch compiled
  this path but did not show `SendMessageW` as the current main-pump frontier;
  the run still stops at empty `GetMessageW @861` after the show/size/paint and
  MFC idle-update sequence.
- Unicorn debug snapshots now include a compact recent guest-WNDPROC return
  ring for `CreateWindowExW` create-time callouts, `DispatchMessageW`,
  `SendMessageW`, and `CallWindowProcW`. The 3,000,000-instruction iNavi launch
  shows `WM_CREATE`, `WM_SHOWWINDOW`, `WM_WINDOWPOSCHANGED`,
  `WM_SIZE(800,480)`, `WM_PAINT`, and MFC `WM_IDLEUPDATECMDUI` all returning
  `0` through the guest path. `WM_PAINT` still falls through the MFC superclass
  path without reaching `BeginPaint`, `GetDC`, or GDI imports.
- SDK MFC export evidence labels the dispatch WNDPROC at `0x6004eba8` as
  `AfxWndProcBase` and the registered `AfxFrameOrView42u` first window proc at
  `0x60005488` as `wce_FirstDefWindowProc`. The target also registers its own
  `Solution_iNavi` class with WNDPROC `0x000135cc`; the create-time `WM_CREATE`
  diagnostic currently enters that target proc and returns `0`.
- SDK `coredll.lib` evidence identifies COREDLL ordinal 1036 as `longjmp` and
  ordinal 2000 as `_setjmp`; `_purecall` is ordinal 1092, so the earlier MFC
  `pc=0` suspicion was not a purecall. The Unicorn import hook now saves and
  restores a CE MIPS `jmp_buf` for `_setjmp`/`longjmp`, including return PC,
  SP/FP/RA/GP, and callee-saved `s0..s7`. The bounded iNavi launch now logs
  `restored MIPS longjmp buffer` and continues through SDK MFC instead of
  returning from `longjmp` into the stale `jalr $v0` site at `0x6001f7f8`.
- Raw CE/GWE class registration now rejects empty class names at the API
  boundary, which removes the bogus `WCE_` recursive class path observed after
  `_wcsnicmp` first enabled the MFC CE superclass flow.
- The latest 500,000-instruction Unicorn launch reaches the real
  `WCE_Solution_iNavi` class, enters create-time `wce_FirstDefWindowProc`,
  restores through `longjmp`, switches the window proc to `AfxWndProcBase`,
  dispatches `WM_SHOWWINDOW`, `WM_WINDOWPOSCHANGED`, `WM_SIZE`, `WM_PAINT`,
  `WM_IDLEUPDATECMDUI`, and then stops at the intentional empty
  `GetMessageW @861` `blocked_get_message` snapshot. This is the current
  frontier; the previous ordinal-1036 `pc=0` crash is retired.
- The ignored eVC4 fixture harness now rebuilds the committed MIPSII fixture
  source tree under `target/wince-fixtures/mipsii/` when local SDK env vars are
  configured. The fixture sources were adjusted for this CE SDK by defining the
  standard `TLS_OUT_OF_INDEXES` sentinel when headers omit it and by using
  explicit `CreateEventW` calls. The ignored eVC4 integration command now
  builds and runs fixtures `001_exit` through `021_rect_math` through the
  emulator successfully. Normal `cargo test --test fixture_exes` still leaves
  the fixture test ignored and does not require eVC4.
- The fixture source ladder now includes focused CE API fixtures for window
  geometry, parent/child relationships, Z-order, message queue behavior,
  synchronous `SendMessageW`, timers, focus/enable state, coordinate mapping,
  RECT helpers, and a system/memory/heap/local/virtual allocation plus registry
  smoke test. The existing `011_api_storm` fixture was made eVC4/MIPSII-valid
  by using the wide version/type of `GetVersionEx`, adding the TLS sentinel
  fallback, and avoiding the MIPS `small` identifier trap. A manual eVC4
  compile/link pass succeeded for `011_api_storm` and the new `012` through
  `021` focused fixtures.
- Core support added for the expanded eVC4 fixtures includes cooperative guest
  `CreateThread` execution/handle signaling in the Unicorn path, executable
  `VirtualAlloc` permissions, PE-backed string resource registration,
  `PostQuitMessage`, CE `GetVersionExW`, RECT helper ordinals, ASCII ACP
  conversion/case APIs, raw registry create/set/query/enum/delete/close,
  CE `WIN32_FIND_DATAW` layout, DC/device-caps/capture APIs, `SetParent`,
  mutable z-order for `SetWindowPos`, raw `SetTimer`/`KillTimer`, correct
  `EnableWindow` previous-state returns, and packed `MapWindowPoints` deltas.
- Raw `FillRect` now paints solid brushes into an attached framebuffer for
  window/screen HDCs. The implementation resolves solid and stock/system brush
  colors, clips to the client/update surface, converts CE `COLORREF` values to
  the framebuffer pixel format including RGB565, and marks dirty rectangles.
  The Unicorn import path now passes the active framebuffer into COREDLL raw
  ordinal dispatch, while memory-only tests keep the existing dispatch path.
  Focused coverage:
  `cargo test --test coredll_raw_gwe coredll_raw_fill_rect_paints_attached_framebuffer`.
- Raw `FindResourceW` and `LoadStringW` now normalize a null `hModule` to the
  current process module, matching the module fallback already used by raw
  menu/bitmap/icon resource helpers. Focused coverage:
  `cargo test --test coredll_raw_gwe coredll_raw_gwe_ordinals_manage_hwnd_rects_points_and_resources`.
- The latest iNavi resource probe shows the main EXE resource tree has CEUX,
  icon, menu, dialog, group-icon, and version resources but no RT_STRING table;
  the observed `FindResourceW(hModule=0x00010000, name=0x0e01, type=6)` miss is
  therefore not a parser miss for a present main-image string resource.
- Host image paths now map through configured storage mounts even when earlier
  virtual mounts have no host root or do not match the path. With
  `mounts.toml`, `GetModuleFileNameW` for
  `D:\INAVI_Emulator\INAVI\INavi\iNavi.exe` now returns
  `\SDMMC Disk\INavi\iNavi.exe` instead of leaking the host path.
- Raw CE `wcsncpy` now follows the aligned byte-count behavior observed from
  the CE 4.2 Mipsii target path. This lets the app derive
  `\SDMMC Disk\iNaviData` from its module path; the latest bounded run confirms
  `FindFirstFileW("\SDMMC Disk\iNaviData")` maps to
  `D:\INAVI_Emulator\INAVI\iNaviData` and succeeds instead of showing the
  Korean SD-card-lock message.
- GDI palette handles and entry APIs now have first real raw semantics:
  `CreatePalette`, `GetPaletteEntries`, `SetPaletteEntries`,
  `GetNearestPaletteIndex`, `GetSystemPaletteEntries`, `SelectPalette`, and
  `RealizePalette` are backed by the generic resource/DC state instead of
  launch stubs. COREDLL import-by-ordinal patching also now normalizes the
  observed SDK export-table index form when it does not collide with a real
  static ordinal; the iNavi import previously trapped at export index 1576,
  which maps to real `GetPaletteEntries`.
- COREDLL import ordinal normalization now preserves checked SDK CRT ordinals
  before attempting export-table-index fallback. The iNavi import slots for raw
  ordinals 1047 and 1097 are SDK CRT `memset` and `swprintf`, not export-index
  aliases for `AddEventAccess` or `BinaryDecompress`; preserving those ordinals
  lets MFC startup continue through the real CRT helpers.
- `RegisterGesture @2724` now records the guest registration arguments and
  returns a zeroed process-heap registration block, matching the observed guest
  behavior where the return value is treated as writable state rather than a
  BOOL. The latest 9,000,000-instruction bounded launch with SDK
  `mfcce400.dll` and `--mount-config mounts.toml` gets past the previous
  `GetPaletteEntries`, SDK CRT ordinal, and `RegisterGesture` frontiers,
  creates `WCE_Solution_iNavi` plus the MFC child HWND
  `Afx:10000:b:0:40000006:0`, and now stops at unimplemented COREDLL ordinal
  25 (`GetSystemTime`). The framebuffer dump
  `target\inavi-register-gesture-handle.ppm` is still diagnostic rather than
  visible app output, so this is progress into the next raw COREDLL tranche,
  not GUI success.
- `GetSystemTime @25`, `GetLocalTime @23`, and
  `GetSystemTimeAsFileTime @2536` now write guest `SYSTEMTIME`/`FILETIME`
  values from a fixed emulator epoch plus the timer tick counter. A reduced-log
  9,050,000-instruction mounted run no longer reaches the previous
  `GetSystemTime` trap, but it did not return a bounded snapshot before the
  shell timeout and had to be stopped manually; it produced no framebuffer
  dump. The next task is to instrument or bound the post-time path so the
  emulator can report whether it is spinning in guest code, spending excessive
  time in translated blocks, or waiting in a message/timer path.
- `--cpu-wall-clock-limit-ms N` now lets Unicorn stop from inside the generic
  code hook after real CPU execution exceeds a host wall-clock budget, captures
  the same register/import/block rings, and still writes `--framebuffer-dump`.
  A 15,000 ms mounted iNavi run now returns without external killing and writes
  `target\inavi-wall-clock-stop.ppm`, but the dump body is still all zero. The
  snapshot stops at `pc=0x0001354c` with repeated SDK CRT `memset @1047`/
  `swprintf @1097` activity in the import ring, so the current frontier is
  startup initialization past system time, not an unimplemented raw import.
- Guest-memory byte helpers now have bulk read/write/fill methods, and the
  Unicorn-backed implementation maps them to `mem_read`/`mem_write` so raw CRT
  `memcpy`/`memset` no longer have to cross the memory trait one byte at a
  time. The focused memory/file/CRT test still passes. A follow-up 15,000 ms
  mounted iNavi wall-clock run stopped at the same `pc=0x0001354c`/blank
  framebuffer frontier, so this is a generic startup-cost cleanup, not the
  visible-GUI breakthrough.
- Unicorn debug snapshots now include a compact top-import count summary in
  addition to the recent import ring. An 8,000 ms mounted iNavi wall-clock run
  writes `target\inavi-import-counts.ppm`, whose 800x480 RGB bytes are still
  all zero, and reports the hottest imports as `memset @1047` 259 times,
  `LocalAlloc @33`/TLS-ish `TlsGetValue @15` 7 times each, and
  `WINSOCK.dll!WSAStartup` once. This confirms the current post-time frontier
  is still legitimate startup/import churn before visible drawing, not a new
  unimplemented import trap.
- The verbose Unicorn `last_code` diagnostic ring now samples ordinary
  per-instruction records while still recording trampoline-sensitive code
  points. A comparable 60,000 ms mounted iNavi run reaches the same MFC
  create-window frontier as before, while a 180,000 ms run gets much farther
  into app code: import counts now include `operator new @1095`, `SetRect @103`,
  `MultiByteToWideChar @196`, and more `GetClassInfoW @878`/class-registration
  traffic before stopping in an app-side date/geometry loop around
  `0x0024f80c`/`0x0024fa30`. The framebuffer dump remains all zero, so this is
  a run-depth/frontier improvement, not visible GUI success.
- SDK CE 4.2 Mipsii `coredll.lib` evidence identified raw soft-float compare
  helpers `__lts` through `__ned` at ordinals 2042 through 2053. COREDLL raw
  dispatch now maps those helpers, reads guest float/double operands from their
  pointer arguments, and routes `__litofp @2032`/`__ultofp @2033` through the
  existing `cemath` conversion path. Focused dispatch coverage passes. A
  release mounted run now gets past the previous `__nes @2047` and
  `__litofp @2032` frontiers and stops at `__ll_div @2005` from SDK MFC
  (`pc=0x7fff06b0`, `ra=0x6000cd80`, `a0:a1=0x00000000_09896800`,
  `a2:a3=0x00000000_00989680`). The framebuffer dump remains blank, so this
  is an ABI/helper frontier, not GUI success.
- Raw MIPS 64-bit helper dispatch now routes signed/unsigned div/rem/mul and
  shift helpers through `cemath`, and the Unicorn import trap writes high-word
  `CeMathValue::I64`/`U64`/`F64` returns to `$v1` while preserving the existing
  `$v0` path. A release mounted run gets past the previous `__ll_div @2005`
  trap. `GetTimeZoneInformation @27` now writes a CE
  `TIME_ZONE_INFORMATION`-layout UTC/no-DST struct and returns
  `TIME_ZONE_ID_UNKNOWN`; the next release mounted run gets past ordinal 27 and
  now stops at `SetForegroundWindow @702` (`pc=0x7fff1410`,
  `ra=0x0089ecec`, `a0=0x00020000`). The framebuffer dump
  `target\inavi-release-timezone.ppm` is still all zero.
- Raw `GetForegroundWindow @701`, `SetForegroundWindow @702`, and
  `SetActiveWindow @703` now use the existing GWE focus/active-window model.
  The mounted release run gets past the previous `SetForegroundWindow @702`
  trap and now stops at `InputDebugCharW @595` (`pc=0x7fff0a90`,
  `ra=0x600119c4`). `target\inavi-release-foreground.ppm` remains all zero.
- Raw `InputDebugCharW @595` now follows the CE debug-port no-data path and
  returns `OEM_DEBUG_READ_NODATA` (`0xffffffff`) when no host debug character is
  available. The focused raw kernel/time/sync dispatch test passes and
  `cargo check --features unicorn` is clean. The mounted release run gets past
  the previous `InputDebugCharW @595` trap and now stops on a guest CPU
  exception (`interrupt_no=12`, `pc=0x00000000`, `ra=0x00035cf4`) after app
  code near `0x000ef80a`; `target\inavi-release-debugchar.ppm` remains all
  zero.
- Unicorn interrupt snapshots now retain the last code-hook PC and instruction
  seen before the interrupt. The post-debug-input release run confirms the CPU
  exception follows the app jump table at `0x000ebb84`: for selector
  `a1=0x5835`, the table base `0x000ebbf0` plus halfword offset `0x3c1a`
  lands at `interrupt_last_pc=0x000ef80a`
  (`interrupt_last_insn=0x007b375a`). This is a halfword-aligned
  MIPS/control-flow frontier, not another unresolved COREDLL import; the
  framebuffer dump remains blank.
- The Unicorn trampoline scanner now detects MIPS halfword jump-table data that
  immediately follows the `lui/addiu/sll/addu/lh/addu/jr` dispatch pattern and
  skips branch/JAL rewrites that overlap those table bytes. This preserves the
  iNavi selector-3 table entry `0x16b0` at `0x000ebbf6`, avoiding the previous
  corrupted jump to `0x000ef80a`. The latest mounted release run now gets past
  that CPU exception and stops cleanly at a COREDLL import trap for ordinal
  `1943` (`pc=0x7fff0900`, `ra=0x600110e4`); the framebuffer dump is still all
  zero.
- Unicorn stop snapshots now print the current trap module kind and module name
  in addition to the trap address/ordinal, confirming the `0x7fff0900` stop as
  `COREDLL.dll` ordinal `1943`. That launch-demanded
  `ADBSetAccountProperties` path now returns `FALSE` with
  `ERROR_NOT_SUPPORTED`, modeling an absent CE account database rather than
  reporting an emulator import stop. The mounted release run gets past both
  observed ordinal-1943 calls and now exits through the guest encoded
  `TerminateProcess` path (`caller=0x0048fa90`, process `0x42`,
  `exit_code=0`); `target\inavi-release-adb1943.ppm` is still all zero.
- Unicorn WNDPROC return traces now annotate return PCs that land inside
  generated branch/JAL trampolines with their original guest instruction. The
  latest mounted release run keeps the same encoded `TerminateProcess` exit and
  blank framebuffer, but the shutdown path is now decoded: the app handles
  `0x56d0`, enters the guest function at `0x0004390c`, reaches the shutdown
  epilogue at `0x00043e30`, and sends `0x5236` from trampoline return
  `0x008b7b70` back to origin `0x00043e38`; the `wce_solution_inavi` WNDPROC
  maps `0x5236` to `WM_CLOSE`.
- `scripts/generate_coredll_ordinals.ps1` now rustfmt-formats its own generated
  Rust output. A temp-output regeneration from `coredll.map` produced a
  byte-identical `coredll_ordinals.rs`, confirming the script is the complete
  map-to-Rust workflow without a separate manual `cargo fmt` step
  (`1b6bc23`).
- Unicorn debug snapshots now retain a bounded `inavi_render_milestones` ring
  for `render_*`, `paint_*`, and `init_dialog_*` app probes, separate from the
  rolling controller tail (`3d908f1`). Real mounted `--desktop host`
  `--tap 400,240 --tap 400,240` runs with framebuffer dumps confirm the app
  reaches `render_size_entry` with `800x480`, then later enters paint and calls
  the render object at `0x0010518c`, but the renderer returns immediately with
  `render_surface=0` and `render_enabled=0`; no
  `render_surface_create_call`/`render_surface_store` milestone is observed,
  and the framebuffer remains all zero.
- `FindResource(W)` for `RT_STRING` now mirrors CE/MFC string-table lookup by
  falling back from an individual string id to its containing string block
  `((id >> 4) + 1)` (`80a88e4`). Focused regression coverage:
  `cargo test rt_string_resource_lookup_falls_back_to_string_block --features unicorn,win32-desktop`.
  A real mounted host/tap run after the fix no longer shows the previous
  `FindResourceW(name="#3867", type="#6")` miss, but it still reaches paint
  with `render_surface=0`, `render_enabled=0`, no useful GDI imports beyond
  `BeginPaint`/`EndPaint`, and an all-zero framebuffer.
- Trace-only resource-root diagnostics around `0x129524`, `0x596b4`,
  `0x59718`, and `0x1ad94` confirmed the bad `\res\values.dat` path came from
  COREDLL `vswprintf @1099` formatting a wide module path with `%s` as narrow.
  After `e52e402`, a scripted mounted monitor run with real `tap 400 240`,
  `until 0x00058a84 90000 0`, `tracefile imports`, `tracefile render`, and
  `tracefile files-full` no longer hits the old `0x00058a84` failure. It
  wall-stops at `pc=0x600972e0` after 90 s with successful `ReadFile` records
  on `\SDMMC Disk\INavi\res\values.dat`; `target\monitor_resource_fixed.ppm`
  remains all zero.
- Host-backed read-only files are now streamed instead of being fully
  preloaded into `OpenFile` data buffers. The RAM spike was traced to normal
  app opens of huge map/search files such as `searchdb\united.db` (~1.7 GB)
  and `image_800x480.db` (~1.09 GB); after the streaming change, bounded
  mounted host runs stay around 100-150 MB private memory instead of climbing
  into the multi-GB range.
- GDI pen state and raw `Polyline`/`LineTo` now write real pixels to the
  attached framebuffer. The latest host/tap evidence is still sparse, but
  nonzero: `target\host_directdib_tap_300s.ppm` has 301 red pixels spanning
  `(0,160)..(300,160)`. This is guest-driven drawing through COREDLL/GDI, not
  manual app painting, and it is not full GUI success.
- Raw `StretchDIBits` and `SetDIBitsToDevice` now have a framebuffer-backed
  `SRCCOPY` path for `DIB_RGB_COLORS` BITMAPINFO data, sharing the existing DIB
  pixel decoder used by memory-DC `BitBlt`. Focused coverage:
  `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  passes 20 tests, including direct DIB framebuffer drawing. Real iNavi host
  runs have not reached these blit ordinals yet.
- Latest bounded `--desktop host` runs built with
  `--features unicorn,trace,win32-desktop` stay memory-stable:
  120 s no-tap produced 101 red Polyline pixels; 180 s tap produced 301 red
  Polyline pixels and reached RSImage/PNG resource loading with 11
  `CreateDIBSection` calls; 300 s tap stayed around 147 MB private memory with
  77 `CreateDIBSection` calls and 9 `CreateCompatibleDC` calls while loading
  RSImage PNG chunks. At that point the app still had not reached screen
  `BitBlt`, `StretchDIBits`, `SetDIBitsToDevice`, `TransparentImage`,
  `PatBlt`, or full UI framebuffer output.
- A 420-500 s tapped host-mode sequence moved the current GDI frontier past the
  first reached `TransparentImage @906` call. `TransparentImage` now supports
  both source memory-DC to screen/window-HDC drawing and memory-DC to memory-DC
  composition with COLORREF keying, using selected DIBSection bitmap bits
  rather than app-specific painting. Focused GDI coverage now passes 23 tests,
  including `NULL_PEN`, direct DIB framebuffer drawing, screen-HDC
  `TransparentImage`, and memory-DC `TransparentImage` composition. Real iNavi
  evidence: `target\host_memdc_transparent_tap_500s_milestones.txt` shows
  `TransparentImage(dst=0x000a00e0, src=0x000a00f8, 90x64,
  transparent=0x00ff00ff)` compositing a 90x64 8-bpp top-down source bitmap
  into a 90x64 16-bpp top-down destination bitmap. The app still has not
  reached a later screen `BitBlt`, `StretchDIBits`, `SetDIBitsToDevice`,
  `PatBlt`, or full UI framebuffer output; the framebuffer dump remains the
  same 301 red Polyline pixels.
- A longer 900 s tapped host-mode run confirms the RAM spike remains fixed and
  resource loading is still advancing. `target\host_more_tap_900s.out.log`
  stops by wall time at `pc=0x60096458`, `ra=0x60010000` with private memory
  samples rising only to about 155 MB. Import counts reached 317
  `CreateDIBSection`, 30 `CreateCompatibleDC`, 40 `SelectObject`, 5
  `Polyline`, and the same single `TransparentImage`; there is still no screen
  `BitBlt`, `StretchDIBits`, `SetDIBitsToDevice`, `PatBlt`, `AlphaBlend`,
  `GradientFill`, `DrawText`, or palette-setting presentation path. The latest
  dump `target\host_more_tap_900s.ppm` is nonblank but still sparse: 401 red
  pixels from `(0,160)` through `(400,160)`.
- A 1200 s tapped host-mode rerun with file-summary tracing advanced beyond
  the prior SDK stop into app/resource code (`pc=0x00b23a5c`,
  `ra=0x00309cb4`) and still stayed memory-stable at roughly 155 MB private
  memory. GDI/import counts and the framebuffer remained unchanged from the
  900 s run: 317 `CreateDIBSection`, 30 `CreateCompatibleDC`, 40
  `SelectObject`, 5 `Polyline`, 1 `TransparentImage`, and 401 red framebuffer
  pixels only. File evidence moved forward: the app opened
  `SDMMC Disk\inavidata\locuspos.bin`, failed to open absent
  `SDMMC Disk\inavidata\goallocuspos.bin`, then opened
  `SDMMC Disk\mapdata\resource\displayreshigh.bin` and read 685090 bytes from
  that 23067871-byte host file before the wall stop.

## Current State

- CPU execution is wired far enough to load mapped PE images, dispatch import
  traps, run the target entry path, execute SDK MFC code through the current
  MIPS trampoline workaround, create/show the main HWND, normalize visible
  top-level default sizing to the virtual desktop, deliver the create-time
  `WM_CREATE` callout, queue and dispatch visible-create show/size lifecycle
  messages, synthesize and dispatch the first `WM_PAINT`, enter guest
  `CallWindowProcW` targets, enter registered guest WNDPROCs for raw
  `SendMessageW` when that import path is used, emulate the SDK MFC
  `_setjmp`/`longjmp` exception path, pass iNavi's `iNaviData` SD-card
  directory validation, implement first palette/DC state behavior, preserve SDK
  CRT import ordinals ahead of export-index aliases, return heap-backed
  `RegisterGesture @2724` state, write basic system/local time structs, and
  stop long post-time runs through `--cpu-wall-clock-limit-ms` with a diagnostic
  snapshot plus framebuffer dump. The current mounted run progresses past the
  previous `GetSystemTime @25` trap, the previous soft-float `__nes @2047`/
  `__litofp @2032` traps, the MIPS `__ll_div @2005` helper frontier,
  `GetTimeZoneInformation @27`, `SetForegroundWindow @702`, and
  `InputDebugCharW @595`, the trampoline scanner's halfword jump-table
  corruption bug, and the launch-demanded `ADBSetAccountProperties @1943`
  import. The current concrete stop is an encoded guest `TerminateProcess`
  path (`caller=0x0048fa90`, process `0x42`, `exit_code=0`); the framebuffer
  remains blank. The current decoded shutdown chain is app message `0x56d0`
  into guest function `0x0004390c`, then a `0x5236` send at `0x00043e30`/
  `0x00043e38` that the main `wce_solution_inavi` WNDPROC converts to
  `WM_CLOSE`. Recent host-backed tap runs also show the app's internal render
  object remains uninitialized at paint time: `render_size_entry` sees
  `800x480`, but the surface allocation path around `0x00104878..0x00104954`
  is skipped and the paint render call at `0x0010518c` returns with
  `render_surface=0` and `render_enabled=0`. A generic virtual framebuffer is
  now attached to the emulator boundary, generic virtual presenter/desktop
  interfaces exist for host
  presentation/window management, and solid `FillRect` on a window/screen HDC
  can write pixels into that framebuffer. Broader guest drawing/blit behavior
  and the target app's own useful drawing path are still incomplete, so this
  must not be treated as GUI success.
- The latest launch diagnostics show the main window's delivered create/show/
  size/paint/idle messages all return through guest code, but no handler creates
  child HWNDs or enters GDI/DC drawing. The next launch-path question is whether
  the create-time sequencing and superclass WNDPROC chain are still incomplete,
  or whether a later CE resource/menu/file/device/event path must seed the UI.
  A later shorter run exited through the guest's encoded `TerminateProcess`
  path after an MFC RT_STRING lookup miss for `0x0e01`; the EXE has no such
  string table, so companion resource-module loading or MFC fallback behavior
  remains under investigation.
- Instruction-limited snapshots show the post-`WM_PAINT` path entering SDK MFC
  thread-local state and message pre-translation (`CThreadLocalObject::GetData`
  and later `CWnd::WalkPreTranslateTree`) rather than reaching guest drawing
  imports yet.
- Remote touch/key input is now connected to guest message retrieval instead of
  only being stored in `CeRemote`: `GetMessageW`/`PeekMessageW` and the Unicorn
  empty-queue block check drain queued remote input into the active, captured,
  or explicitly filtered HWND before checking GWE queues. The runner also has
  repeatable `--tap X,Y` startup injection, and host desktop mode keeps pumping
  while blocked in `GetMessageW` and refreshes `--framebuffer-dump` at each new
  blocked wait. A focused test verifies a queued tap becomes
  `WM_LBUTTONDOWN`/`WM_LBUTTONUP` through `CeKernel::get_message_w`.
- Unicorn code tracing now reads static instructions from mapped PE/DLL bytes
  before falling back to emulator memory and samples block traces. With that
  overhead removed, a real mounted iNavi no-tap run with a 90,000 ms wall-clock
  limit returns in roughly 27 s at the idle `GetMessageW @861`
  `blocked_get_message` frontier instead of timing out in app-side
  date/geometry code. The snapshot has a visible `800x480`
  `wce_solution_inavi` top-level HWND plus an MFC child HWND, but the
  framebuffer dump remains all zero.
- A real mounted iNavi run with `--tap 400,240` and the same 90,000 ms
  wall-clock limit now confirms startup-injected input is actually consumed:
  the import/message trace reaches the `WM_LBUTTONDOWN`/`WM_LBUTTONUP` path and
  drains back to the idle `GetMessageW @861` snapshot. The framebuffer is still
  all zero, so the current target frontier is no longer startup input delivery;
  it is the missing app paint/GDI/surface path after the visible window and real
  tap are present.
- The command-line runner now has an interactive `--monitor` mode for
  repeatable emulator control. The first monitor command set supports
  `continue [wall_ms] [insns]` bounded CPU slices, `step [insns]` bounded
  instruction slices, `tap X Y`, `dump [path]`, `present`, `regs`,
  `checkpoint [name]`, `checkpoints`, `rewind [name|index]`, and `quit`.
  A scripted smoke test with `help`/`quit` passed, and a scripted mounted iNavi
  session verified `tap`, bounded `continue`, `dump`, and `regs`, writing
  `target\monitor_slice.ppm` and `target\monitor_default.ppm`. A follow-up
  scripted mounted session verified `checkpoint before`, real `tap 400 240`,
  bounded `continue`, `checkpoint after`, `rewind before`, and `dump`, writing
  `target\monitor_rewind.ppm` and `target\monitor_rewind_default.ppm`.
  Monitor checkpoints clone and restore the CPU wrapper, CE kernel, and
  framebuffer state. Live in-core Unicorn register/memory rewind still requires
  persistent Unicorn CPU snapshots.
- Monitor diagnostics now replace the previous always-explosive default stop
  output with compact summaries. Detailed trace rings are still captured, but
  are pulled explicitly through `trace all`, `trace imports`, `trace counts`,
  `trace calls`, `trace code`, `trace blocks`, `trace messages`,
  `trace wndproc`, `trace render`, or `trace files`. The monitor also exposes
  `map`, `x ADDRESS [LEN]`, and `disasm ADDRESS [WORDS]` for mapped static
  PE/DLL/trap bytes. Scripted verification wrote
  `target\monitor_mapped_inspect.log` for `map`/`x`/`disasm` and
  `target\monitor_quiet_default.log` plus `target\monitor_quiet_default.ppm`
  for compact stop output with explicit trace selectors.
- Monitor `tracefile KIND PATH` writes selected detailed trace rings to disk
  instead of stdout. A scripted mounted session kept
  `target\monitor_tracefile.log` compact while writing detailed imports to
  `target\monitor_trace_imports.txt` and import counts to
  `target\monitor_trace_counts.txt`.
- Monitor `until ADDRESS [wall_ms] [insns]` now uses the Unicorn code hook to
  stop on a requested guest PC and records `pc_stop` in the debug snapshot. A
  scripted mounted iNavi session verified `until 0x0048f6d8 1000 100000`
  stopped at the main EXE entry PC and wrote
  `target\monitor_until_summary.txt` through `tracefile summary`.
- Default logging now keeps diagnostics opt-in: the tracing subscriber defaults
  to `warn` unless `RUST_LOG` is set, compact stop summaries no longer append
  import-count diagnostics, and startup output is concise by default. The old
  detailed registry/device/PE/DLL boot context remains available with
  `--verbose`. Scripted monitor startup checks wrote
  `target\monitor_default_startup.log` (compact) and
  `target\monitor_verbose_startup.log` (detailed).
- Default startup output is now quiet unless `--verbose` is requested. Normal
  run/monitor output keeps stop summaries, framebuffer dump paths, explicit
  monitor command responses, and error diagnostics; PE/layout/DLL/import-count
  boot context is opt-in through `--verbose`.
- Raw `SetFilePointer` now treats `lDistanceToMove` as a signed 32-bit `LONG`
  when `lpDistanceToMoveHigh == NULL`, matching the Win32/CE API shape instead
  of converting negative low-word seeks into large positive offsets. Explicit
  monitor file traces now record read cursor ranges and `trace files` prints a
  compact activity summary by default; the old raw 512-record dump remains
  available as `trace files-full`. Focused raw file regression coverage passes,
  and the real mounted iNavi monitor probe advanced: `until 0x000587ec
  180000 0` now hits `pc_stop=0x000587ec` with `v0=0` instead of wall-stopping
  inside the `values.dat` parser. This proves the `0x589dc` readiness subcall
  now returns; the next frontier is why that first readiness check returns
  false.
- Unicorn WNDPROC return handling no longer validates every `WM_PAINT`
  unconditionally. Plain guest WNDPROC returns leave the update region pending;
  `DefWindowProcW` and `CallWindowProcW(DEFAULT)` consume paint through the
  default-proc helper instead. A focused Unicorn-feature regression covers this
  distinction. A real mounted `--tap 400,240` rerun still writes an all-zero
  framebuffer, but the trace now clearly shows the top-level `WM_PAINT` entering
  app WNDPROC `0x000135cc`, then falling through `DefWindowProcW @264` without
  `BeginPaint` or GDI/DC imports. The next display frontier is the app
  WNDPROC/message-map branch that decides not to paint.
- WM_SIZE render diagnostics now annotate the render object/vtable/target and
  dimensions for the call at `0x0002d1a0`. A mounted monitor run with
  `tap 400 240`, `until 0x00058a04 180000 0`, `dump`, and
  `tracefile render` reached idle `GetMessageW @861` with an all-zero dump.
  The render milestones show `WM_SIZE` passing `800x480` to render object
  `0x3006b360`, vtable slot `+0xf0` target `0x0011ce60`, while the same object
  still reports `render_surface=0` and `render_enabled=0`. No
  `render_resize_entry`, `render_surface_create_call`, or
  `render_surface_store` milestone appears, so the current gap is the real
  lifecycle path that should call the resize/allocation slot `+0xf4`
  (`0x001033e4`), not missing WM_SIZE dimensions.
- CE source evidence rejected treating rooted ordinary file opens as
  EXE-directory-relative: `cnnclpth.h` says CE paths are absolute whether
  explicitly or implicitly rooted, and FSDMGR `InternalCreateFileW`
  canonicalizes before resolving the volume. A trace-only `0x59718`/`0x1ad94`
  readiness diagnostic identified the previous failing lookup text as
  `\res\values.dat`, and a follow-up formatter trace proved the source string
  was lost by wide `vswprintf("%s", wide_path)` handling. Do not paper over
  this by mounting app resources at `\res`; `e52e402` fixes the wide printf
  semantics and the real run now reads `\SDMMC Disk\INavi\res\values.dat`.
- CE FSDMGR source backs the removable mount attribute behavior now modeled by
  `mounts.toml`: `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\virtroot.cpp`
  `FindNextMountDir` sets `FILE_ATTRIBUTE_DIRECTORY`, then ORs
  `FILE_ATTRIBUTE_TEMPORARY` when `AFS_FLAG_PERMANENT` is absent and ORs
  `FILE_ATTRIBUTE_SYSTEM` for system mounts. The emulator's `removable = true`
  flag is this non-permanent mount behavior, so exact `\SDMMC Disk` and root
  enumeration now report `0x110` (`DIRECTORY|TEMPORARY`) while normal child
  directories remain `0x10`.
- The checked-in map-backed COREDLL ordinal path now covers the latest
  launch-demanded CRT/file tranche from `coredll.map`: `_wcsicmp @230`,
  `atoi @993`, `strcpy @1066`, `strtok @1073`, `fgets @1109`, and `_wfopen
  @1145`. Focused raw dispatch coverage verifies decimal parsing,
  per-thread `strtok` continuation, narrow/wide stdio file opens, `fgets`, and
  exact `\SDMMC Disk` file attributes/find data.
- A mounted monitor run after `_wfopen @1145` no longer stops on a COREDLL
  import ordinal. It opens and reads deeper map/SearchDB/MRData files plus
  `\SDMMC Disk\INavi\res\InaviMainConfig.bin`, then stops at
  `pc=0x3004a0a8`, `ra=0x3004a0a8`, `sp=0x7ffdf1f0`, `a0=0x0002000c`,
  `a1=0x00000401`; the framebuffer dump remains all zero. The current evidence
  points at a guest control-flow/callback frontier near the app query thunk
  rather than another missing COREDLL ordinal.
- `SystemParametersInfoW(SPI_GETOEMINFO)` now reads
  `HKLM\System\Emulator\SystemParametersInfo` by action id/alias, with generic
  CE fallbacks. The checked-in registry snapshot returns `iNavi GN 2010`, which
  moves the mounted iNavi resource selector from the old `mode=0` table miss to
  `mode=47`, finds record 47 in `values.dat`, reads the 1746-byte payload, and
  enters the resource field parser. The current real run still dumps an
  all-zero framebuffer, but the active stop is later: interrupt 20 at
  `pc=0`, `ra=0x0006bfb4`, `last_pc=0x0006bf8c` while parsing the second
  payload key. Monitor sessions now keep running after this stop so scripted
  `tracefile` and `dump` commands complete; `target\monitor_debugger_oeminfo.*`
  is the latest artifact set.
- The monitor is intentionally honest about statefulness: `continue`, `until`,
  `tap`, `dump`, `tracefile`, `checkpoint`, and `rewind` are usable, but
  `step` now reports that live instruction stepping needs persistent Unicorn
  CPU/RAM state instead of restarting from the image entry and pretending to
  single-step.
- The default bootstrap uses `regs.json` as backing storage for the fake CE
  registry API and creates base GWE, timer, audio, and memory-map state.
- The virtual Win32/CE framework and COREDLL dispatcher are connected to Unicorn
  import traps. SDK `mfcce400.dll` can execute from a relocated image through
  the current target startup and message-pump entry path. MFC imports are now
  SDK-DLL-only; commctrl, WINSOCK, OLE, and additional CE 4.2 ordinal behavior
  still need real subsystem-backed implementation as traces demand.
- Many COREDLL ordinals are classified and dispatchable but still stubbed by
  subsystem. Kernel/thread/time/sync, performance counter/frequency,
  memory/local/heap/virtual allocation,
  raw file buffer/find marshalling, first registry create/query/enum/delete
  behavior, first GWE class/HWND/RECT/text/window-long/focus/capture/z-order/
  timer/message pump/paint-update behavior, unplugged waveOut adapter ordinals,
  system-info/memory status, and first resource/string raw ordinals have real
  CE-referenced semantics; remaining ordinals still need to be burned down
  subsystem by subsystem.
- Rust remote server REST binding is implemented for the v2-compatible
  `/api/v1` status/frame/control routes. WebSocket control and audio upgrades
  are now implemented for JSON control frames and binary PCM streaming; live
  log streaming remains open.
- The mounted iNavi trace now gets beyond the earlier `values.dat` and map
  resource checks into RSImage/PNG resource loading. A trace-enabled monitor run
  with `tap 400 240` opened `\SDMMC Disk\INavi\res\FontResHigh.utf`,
  `resi_800x480.bin`, and `resmapi_800x480.bin`; `FindFirstFileW("\*")`
  reports the removable `SDMMC Disk` mount as `0x110`. Targeted RSImage probes
  at `0x00307d18`/`0x00307d44`/`0x00307d58`/`0x00307d84` show the stream
  callback is real `ReadFile` on `resi_800x480.bin`, with actual bytes matching
  requested bytes. The first PNG resource has a 28-byte record prefix followed
  by an 800x160 PNG; the later one is 800x320. No short-read path at
  `0x00307d74` was hit.
- The PNG decode/unfilter path is slow but valid rather than a broken
  trampoline: a long run parked in the trampoline for app origin `0x003447ac`,
  which disassembles as a normal PNG filter loop, then returns from the caller at
  `0x0030f384` with `v0=0x101`. Continuing after that return exits through the
  app's singleton/already-running branch, not through drawing. The preserved
  milestone trace `target\monitor_after_png_milestones.txt` shows
  `FindResourceW(#3585, #6) -> 0`, then `CreateMutexW(L"iNavi")` returning
  handle `0x11c` with `last_error=183`, `FindWindowW(title=L"iNavi") ->
  0x00020000`, `SetForegroundWindow`, `ReleaseMutex`, and finally encoded
  `TerminateProcess` from `0x0048fa90`. The framebuffer remains all zero.
- Monitor trace output now has a `tracefile milestones PATH` selector for the
  existing import milestone ring. `CreateMutexW`, `ReleaseMutex`, and
  `FindWindowW` milestones include decoded wide-string names/titles plus
  last-error/result details, so long resource/PNG runs do not lose the important
  singleton/window imports behind CRT noise.
- Admin `cargo flamegraph` confirmed startup time was dominated by emulator
  overhead, not the 2.7 GB file-memory issue. The fixed runtime now uses a
  shared static COREDLL export table for hot import dispatch/GetProcAddress
  paths, resolves import trace names directly from ordinal metadata, uses
  precomputed trampoline origin/stub maps plus page sets in the global Unicorn
  code hook, and indexes mapped PE/DLL code by page for hook instruction reads.
  The same 60 s host/tap bounded run moved from `pc=0x001704a4` to
  `pc=0x003426f0`, reaches real paint/DC/DIB imports and sparse framebuffer
  pixels (`target\host_mapped_code_index_progress_60s.ppm`: 301 red pixels,
  `(0,160)..(300,160)`), and stays memory-stable instead of returning to the
  multi-GB RAM spike. The opt-in `WINCE_EMU_FAST_START=1` path remains broken:
  it immediately reaches the thread-exit stub with no import counts, so do not
  enable it by default.
- The next flamegraph-guided startup fix removed the hottest
  `map_kernel_memory_allocations` cost. `MemorySystem` now exposes heap/virtual
  generation counters and the heap high-water mark, while Unicorn maps only
  heap spillover beyond the initially mapped 16 MiB arena and refreshes virtual
  mappings only when virtual state changes. A 30 s mounted host/tap run now
  reaches the old 60 s sparse-pixel frontier:
  `target\after_heap_mapper_30s.ppm` has the same 301 red pixels from
  `(0,160)..(300,160)`, while the stop advances to `pc=0x00339d9c` with
  `heap_live=7348/22215084B`, `ReadFile=15559`, and `CreateDIBSection=71`.
  The new 60 s run stops at `pc=0x0030f3c8`, `ra=0x002fd4cc`,
  `heap_live=7504/23541185B`, with `ReadFile=24712`,
  `CreateDIBSection=147`, and region churn visible (`CreateRectRgn=3866`,
  `CombineRgn=3863`, `DeleteObject=3865`). The framebuffer is still only the
  301-pixel Polyline line, so this is a real startup-speed improvement, not
  complete UI progress.
- Heap spillover mapping is now chunked in 1 MiB aligned regions instead of
  committing one Unicorn page at a time. A 30 s host/tap run reaches
  `pc=0x0034286c` with the same 301-pixel Polyline framebuffer and
  `ReadFile=15867`; a 60 s host/tap run advances to `pc=0x00b55150`,
  `ra=0x0030f384`, `heap_live=7530/23815449B`, `ReadFile=33759`,
  `CreateDIBSection=190`, and the same sparse framebuffer. The follow-up admin
  flamegraph (`target\startup_flamegraph_after_heap_chunk.svg`) no longer shows
  heap mapping in the filtered top frames. It runs far enough to hit the next
  real guest/UI fault: `READ_UNMAPPED` at `pc=0x0026f7e4`
  (`render_map_pointer_deref`), `addr=0x0000005c`, with
  `ReadFile=61825`, `CreateDIBSection=317`, and 401 red pixels spanning
  `(0,160)..(400,160)`.
- The CE file hot path no longer preloads existing host files into `Vec<u8>` just
  because the guest requested write access, and streamed reads no longer reopen
  the host file for every `ReadFile`. `OpenFile` now uses memory or live
  host-file backing, small host-backed reads use a bounded 64 KiB per-handle
  cache, larger `read_file_into` requests stream in 64 KiB chunks, and raw
  COREDLL `ReadFile` continues writing directly into guest memory. Full
  `cargo test --features unicorn,trace,win32-desktop` passes. A release
  host/tap monitor run to the current render-map fault wrote
  `target\file_io_hotpath_cached_boot_summary.txt` and
  `target\file_io_hotpath_cached_boot_files.txt`; the stop is still
  `pc=0x0026f7e4`, but the new file counters show
  `host_file_open_count=633`, `host_file_read_count=64995`,
  `host_file_read_bytes=3787819`, `memory_backed_open_count=2`, and
  `max_read_request=685080`, confirming the remaining startup delay is no
  longer caused by multi-hundred-MB file preloading.
- Added the next CE-backed GWE/dialog slices from the CE6 dialog manager and
  SDK headers. Raw `GetDialogBaseUnits` now returns the project-wide dialog
  unit base and raw `MapDialogRect` maps guest `RECT`s through the CE DLU
  formulas; raw `GetNextDlgTabItem` and `GetNextDlgGroupItem` now walk real
  dialog children, visibility/enabled state, `WS_TABSTOP`, and `WS_GROUP`
  group boundaries instead of falling through the generic ordinal path. The
  focused raw GWE tests cover dialog-unit conversion plus tab/group order,
  disabled controls, and previous/next cycling.
- Raw `IsDialogMessageW` now performs the first CE-backed dialog-message
  handling instead of only validating the `MSG` pointer and returning success.
  It consumes messages only when the target HWND is the dialog or a descendant,
  dispatches ordinary dialog-owned messages through the normal dispatch path,
  sends `WM_GETDLGCODE` for key handling, moves focus on TAB via the existing
  dialog tab-order helper, honors `VK_SHIFT`/`GetKeyState` for Shift+TAB
  reverse traversal, routes Escape as `IDCANCEL`, and routes Return to the
  focused pushbutton or the dialog's default pushbutton with `IDOK` fallback.
  GWE now answers `GetKeyState` from queued keydown/up state, exposes
  `GetAsyncKeyState`/`GetAsyncShiftFlags` with the first CE `KeyState*` and
  `KeyShift*` bit model, answers `WM_GETDLGCODE` for button/static/edit
  controls, and implements `DM_GETDEFID`/`DM_SETDEFID` over child pushbutton
  style state, so Enter from an edit control can reach the default button
  through generic dialog-manager behavior. Focused raw GWE coverage now proves
  dialog-owned dispatch, unrelated-HWND rejection, TAB and Shift+TAB focus
  traversal, `GetKeyState` high-bit state, async-key latch consumption, async
  shift flags, default-button dialog codes, and `DM_GETDEFID`/`DM_SETDEFID`
  transitions. The eVC MIPSII
  fixtures
  `052_modeless_dialog_isdialogmessage` and the strengthened
  `076_dialog_tab_enter_escape` also pass through the ignored `fixture_exes`
  harness with `WINCE_FIXTURE_FILTER`, proving the imported app-style path
  reaches guest dialog procedures for modeless `WM_COMMAND` and Return-key
  default-button command generation.
- Added the first CE-backed keyboard injection slice. Raw `PostKeybdMessage`
  now accepts the SDK six-argument shape and the wider internal GWE API-set
  shape, derives `WM_KEYDOWN`/`WM_KEYUP` from `KeyStateDownFlag`, preserves
  previous-down/key-up lParam transition bits, optionally queues character
  buffer entries as hardware-sourced `WM_CHAR`, and wakes the target thread
  through the normal GWE message queue. Raw `keybd_event` now targets the
  focused/active keyboard window and posts hardware-sourced key messages with
  scan-code/extended/up lParam bits. Focused raw GWE coverage proves
  `PostKeybdMessage` key/char/key-state behavior and `keybd_event` focus
  targeting; the new eVC MIPSII fixture `169_post_keybd_message` passes
  through the ignored `fixture_exes` harness, covering the imported
  `PostKeybdMessage` keydown/up path plus app-pump `TranslateMessage`
  character generation.
- Added the CE keyboard-target routing slice. GWE now tracks
  `m_hwndKeyboardTarget`-style keyboard targets per thread/message queue,
  clears targets when HWND subtrees are destroyed, hidden, or disabled through
  the existing focus/activation cleanup path, and targetless keyboard input
  now routes through that explicit target before focus/active fallback. Raw
  `SetKeyboardTarget`, `GetKeyboardTarget`, and
  `GetForegroundKeyboardTarget` now return real HWND state instead of generic
  stub results. Focused raw GWE coverage proves explicit target set/get,
  foreground target reporting, `keybd_event` routing to the keyboard target,
  fallback to focus after clearing, and invalid-HWND last-error behavior; the
  full raw GWE suite now has 78 passing tests.
- Added the next CE-backed sent-message reply slice. GWE `ReplyMessage` state
  now marks the active cross-thread sent transaction as `SMF_RESULT_READY`,
  records the early WNDPROC result, makes `InSendMessage` false for that send
  depth, preserves the early result when the receiver later returns from
  dispatch, and keeps nested send depths independent. The kernel wrapper routes
  the early reply through the same scheduler send-reply wake-candidate path as
  normal WNDPROC completion, timeout, and receiver destruction. Focused GWE
  tests cover early reply and nested-depth behavior, and
  `basic_subsystems` covers a sender waiter waking before the receiver
  dispatch returns. The smoke test's throwaway `WM_USER + 1` message was moved
  away from CE's `DM_SETDEFID` value so it no longer conflicts with real dialog
  message semantics.
- Added the CE-backed `SendMessageTimeout` transaction flag slice. Cross-thread
  sends created with a timeout now carry `SMF_TIMEOUT` in the GWE sent-message
  state, matching the CE `cmsgque.h` `smfTimeout`/`MessageTimeout` shape.
  Raw `SendMessageTimeout(..., timeout>0)` no longer executes an immediate
  virtual send in the caller thread; it queues the receiver-side sent message
  transaction and leaves the result pointer untouched until receiver dispatch
  or a Unicorn sender wait completes. Raw `timeout=0` behavior still creates
  and immediately expires the same transaction. Focused raw GWE coverage proves
  nonzero timeout queueing, `SMF_TIMEOUT` metadata, receiver retrieval, and
  dispatch completion. The raw syscall boundary now also follows CE
  `winuser.h`'s exposed `SMTO_NORMAL`-only flag surface: unsupported
  desktop-style `fuFlags` fail with `ERROR_INVALID_FLAGS`, leave the result
  pointer untouched, and do not queue a receiver-side send. Focused coverage:
  `coredll_raw_send_message_timeout_rejects_non_ce_flags_without_queueing`.
- Tightened the CE-backed `SendNotifyMessageW` transaction metadata. Different-
  thread notifications already used the receiver-side sent-message queue; they
  now also carry `SMF_SENDER_NO_WAIT | SMF_NOTIFY_MESSAGE`, matching the
  `cmsgque.h` `smfSenderNoWait`/`smfNotifyMessage` shape instead of being only
  a generic asynchronous send. Focused raw GWE coverage now asserts the queued
  notify transaction has no sender waiter and preserves both CE flags while
  same-thread notifications remain synchronous.
- Tightened raw `SendMessageW` at the syscall boundary. Different-thread raw
  sends now create the same sender/receiver sent-message transaction used by
  the Unicorn callout path instead of immediately executing the receiver
  window through the caller thread. `DefWindowProcW` was split away from that
  queueing path and remains direct default processing. Focused raw GWE coverage
  proves the queued `SendMessageW` exposes `QS_SENDMESSAGE`, marks receiver
  `InSendMessage` only after retrieval, completes through raw `DispatchMessageW`,
  and leaves `DefWindowProcW` immediate. The raw GWE suite now has 80 passing
  tests.
- Aligned raw `SendDlgItemMessageW` with the CE SDK wrapper shape documented in
  `winuser.h`: normal dialog-item messages now route through the same
  `SendMessageW` queueing helper after `GetDlgItem`, so a control/window owned
  by another thread receives a sent-message transaction instead of a caller-
  thread shortcut. Focused raw GWE coverage proves the child HWND, sender/
  receiver thread ids, `QS_SENDMESSAGE`, receiver `InSendMessage`, and dispatch
  result completion; button-check helper shortcuts remain covered. The raw GWE
  suite now has 81 passing tests.
- A post-send-queue mounted virtual/tap smoke probe using
  `D:\INAVI_Emulator\DUMPPLZ\Windows` wrote
  `target\sendmsg_queue_virtual_60s_*`. It stayed memory/file-I/O bounded
  (`host_open=206`, `host_read=33845/2085117B`, `mem_open=2`,
  `max_read=497178`) and produced a populated framebuffer
  (`1151398` nonzero RGB bytes), with real display presentation still present
  in the render trace. This 60 s run did not prove a new UI frontier: it
  stopped at `pc=0x003443d8` during repeated read-only opens/reads of
  `\SDMMC Disk\INavi\res\resmapi_800x480.bin`, with
  `sendsig=0/sendcand=0`, so the send-message slices remain foundational
  fidelity rather than visible post-splash success.
- The comparable 150 s virtual/tap probe wrote
  `target\sendmsg_queue_virtual_150s_*`. It also stayed bounded
  (`host_open=422`, `host_read=78935/2814860B`, `mem_open=2`,
  `max_read=497178`) and kept the framebuffer populated
  (`1151398` nonzero RGB bytes). It stopped at `pc=0x00b4bc00` with no
  send-reply activity (`sendsig=0/sendcand=0`) while the trace tail shows
  repeated `RSImage LoadPNG/Create` work, many `CreateDIBSection` calls, and
  `GetDC(hwnd=0x00020004)`/`ReleaseDC` around those image loads. The active
  short-run bottleneck is therefore not the new send-message transaction path;
  continue by tracing the generic resource/GDI/presentation path for those
  loaded surfaces.
- Added an indexed-DIB fidelity slice for CE GDI color tables. `BitmapObject`
  now stores RGBQUAD color tables, raw `SetDIBColorTable`/`GetDIBColorTable`
  read and write the selected bitmap table through guest memory, and the 8 bpp
  blit path resolves palette indices before writing RGB565 framebuffer pixels.
  The focused test creates an 8 bpp top-down DIBSection, installs red/green
  RGBQUAD entries, round-trips the table, and verifies the resulting `BitBlt`
  pixels. The follow-up direct-DIB slice now parses `BITMAPINFO` color tables
  for 1/4/8 bpp `DIB_RGB_COLORS` sources, so `StretchDIBits` and
  `SetDIBitsToDevice` can render indexed source pixels without requiring an
  explicit selected-bitmap `SetDIBColorTable` call. The raw GWE suite now has
  41 passing tests.
- Latest mounted iNavi host/tap validation after the dialog and DIB color-table
  slices wrote `target\long30_*` artifacts. The 30 s run stopped at
  `pc=0x003446ec` with memory still stable (`heap_live=7297/21843020B`),
  file I/O bounded (`host_open=156`, `host_read=25097/1921203B`,
  `memory_backed_open_count=2`, `max_read=497178`), and a sparse nonzero
  framebuffer body (`target\long30_probe.ppm`: 301 nonzero bytes). The render
  trace still reports no named render milestones, so this is real framebuffer
  movement from the guest path but not useful UI completion.
- Mounted validation after the embedded-BITMAPINFO indexed-DIB slice wrote
  `target\bitmapinfo_palette_*` artifacts. The 30 s host/tap run stopped at
  `pc=0x00b5019c`, stayed memory-stable (`heap_live=7192/21798813B`,
  `host_open=156`, `host_read=25079/1926075B`, `mem_open=2`,
  `max_read=497178`), and produced the same sparse guest-GDI red line:
  301 nonzero pixels from `(0,160)` through `(300,160)` in
  `target\bitmapinfo_palette_probe.ppm`. The render trace still says
  `inavi render milestones: none`, so the next blocker is later lifecycle/
  blit/surface progress, not bulk file I/O or the indexed color-table path.
- Rebuilt the debug executable with `CreateDIBSection` milestone details that
  include parsed DIB color-table entry counts, then reran the mounted 30 s
  host/tap probe (`target\dib_colors_fresh_*`). The app's real RSImage
  DIBSections now show indexed palette ingestion on the mounted path:
  800-wide 8 bpp DIBs report `colors=256`, and later 100x100/100x135 8 bpp
  resources report populated tables such as `colors=199`, `colors=156`,
  `colors=223`, `colors=236`, and `colors=249`. The framebuffer remains the
  same 301-pixel red line and the render trace still has no named render
  milestone, so the color-table port is active but not the remaining UI gate.
- Added the first CE-backed focus/activation window lifecycle slice. GWE now
  tracks an explicit active window separately from keyboard focus, clears it on
  destroy, and raw `SetFocus`, `SetActiveWindow`, `SetForegroundWindow`,
  activating `ShowWindow` commands, and `SetWindowPos` without
  `SWP_NOACTIVATE` route through kernel lifecycle helpers that queue
  `WM_ACTIVATE`, `WM_SETFOCUS`, and `WM_KILLFOCUS` for guest/MFC dispatch.
  `ShowWindow(SW_SHOWNOACTIVATE)`, `SW_SHOWMINNOACTIVE`, and `SW_SHOWNA`
  preserve no-activate behavior at the raw ordinal boundary. Focused coverage
  `coredll_raw_focus_and_activation_queue_ce_messages` passes, and the raw GWE
  suite now has 42 passing tests. Mounted validation wrote
  `target\focus_activation_*`: the 30 s host/tap run stayed memory-stable
  (`heap_live=7295/21831892B`, `host_read=24819/1924419B`) and kept the same
  301-pixel sparse red framebuffer line with no named render milestone. This
  is real window-subsystem fidelity, not complete UI output.
- Added the CE-backed `EnableWindow` lifecycle slice. Raw `EnableWindow` now
  routes through `CeKernel::enable_window`, keeps the CE previous-enabled
  return contract, mutates only live HWND state, posts `WM_CANCELMODE` before
  disabling, and posts `WM_ENABLE` on real enabled-state transitions while
  avoiding duplicate messages when the state is unchanged. Focused coverage
  `coredll_raw_enable_window_queues_ce_enable_messages` passes, the raw GWE
  suite now has 43 passing tests, and full
  `cargo test --features unicorn,trace,win32-desktop` passes. Mounted
  validation wrote `target\enable_window_*`: the 30 s host/tap run stayed
  memory-stable (`heap_live=7294/21830764B`,
  `host_read=24620/1918582B`) and preserved the same 301-pixel sparse red
  framebuffer line with no named render milestone. This is another generic
  window-lifecycle fidelity slice, not useful UI output yet.
- Added the raw `BringWindowToTop` z-order/activation slice from CE SDK/GWE API
  evidence. Raw ordinal 275 now routes through `CeKernel::bring_window_to_top`,
  reuses the existing HWND z-order engine as a `HWND_TOP`/no-move/no-size
  transition, activates the top-level target through the kernel lifecycle
  helper, and rejects invalid HWNDs. Focused coverage
  `coredll_raw_bring_window_to_top_updates_z_order_and_activation` passes, the
  raw GWE suite now has 44 passing tests, and full
  `cargo test --features unicorn,trace,win32-desktop` passes. Mounted
  validation wrote `target\bring_window_top_*`: the 30 s host/tap run stayed
  memory-stable (`heap_live=7293/21820764B`,
  `host_read=24620/1922561B`) and preserved the same 301-pixel sparse red
  framebuffer line with no named render milestone. A follow-up mounted run in
  virtual desktop mode wrote `target\virtual_after_bring_window_top_*` without
  showing the host presenter window; it stopped at `pc=0x00343750` with
  `heap_live=7200/21843325B`, `host_read=26122/1952147B`, the same 301-pixel
  red line, and still no named render milestone.
- Added the CE-backed disabled-ancestor enabled-state slice. Window creation
  now seeds direct enabled state from `WS_DISABLED`, `EnableWindow` keeps the
  direct enabled bit and `WS_DISABLED` style synchronized, and raw
  `IsWindowEnabled`, dialog tab/group traversal, and HWND point hit-testing now
  walk ancestor HWNDs so children under a disabled parent are effectively
  disabled without receiving child `WM_ENABLE` notifications. Focused coverage
  `coredll_raw_is_window_enabled_observes_disabled_ancestors` passes, the
  dialog-navigation fixture now checks disabled parents, the raw GWE suite now
  has 45 passing tests, and full
  `cargo test --features unicorn,trace,win32-desktop` passes. Mounted
  validation used virtual desktop mode per current workflow and wrote
  `target\disabled_ancestor_virtual_*`: the 30 s run stopped at
  `pc=0x00339d90`, stayed memory-stable (`heap_live=7304/21886404B`,
  `host_read=25878/1940731B`), preserved the same 301-pixel red line, and had
  no named render milestone.
- Added the matching CE-backed hidden-ancestor visibility slice. `ShowWindow`,
  `SetWindowPos(SWP_SHOWWINDOW/SWP_HIDEWINDOW)`, and `SetWindowLong(GWL_STYLE)`
  now keep direct visibility synchronized with `WS_VISIBLE`, while raw
  `IsWindowVisible` and point hit-testing treat children of hidden parents as
  effectively invisible without changing the child HWND. Focused coverage
  `coredll_raw_is_window_visible_observes_hidden_ancestors` passes, the raw
  GWE suite now has 46 passing tests, and full
  `cargo test --features unicorn,trace,win32-desktop` passes. Mounted
  validation again used virtual desktop mode and wrote
  `target\visibility_enabled_virtual_final_*`: the 30 s run stopped at
  `pc=0x00344780`, stayed memory-stable (`heap_live=7305/21887532B`,
  `host_read=26160/1961105B`), preserved the same 301-pixel red line, and had
  no named render milestone.
- Added the CE-backed `WM_WINDOWPOSCHANGED` payload slice. The SDK `WINDOWPOS`
  layout, GWE `IsLParamPtr(WM_WINDOWPOSCHANGED)`, and CE MFC
  `reinterpret_cast<WINDOWPOS*>(lParam)` dispatch path now have a matching raw
  emulator path: window move/size/create lifecycle posts allocate a stable
  guest heap payload, raw `GetMessageW`/`PeekMessageW` materialize the
  28-byte `WINDOWPOS` struct into guest memory, and raw/Unicorn
  `DispatchMessageW` return paths release the registered payload after
  consumption. Focused coverage
  `coredll_raw_windowposchanged_carries_guest_windowpos_payload` verifies the
  guest struct fields and heap release, the raw GWE suite now has 47 passing
  tests, and full `cargo test --features unicorn,trace,win32-desktop` passes.
  Mounted validation used virtual desktop mode to avoid the host black window
  and wrote `target\windowpos_virtual_*` plus
  `target\windowpos_virtual_60s_*`: the 60 s run reached RSImage
  `CreateDIBSection` work, stopped at `pc=0x00073684`, stayed memory-stable
  (`heap_live=6929/21276879B`, `host_read=7839/1759291B`), and produced a
  101-pixel red line from `(0,160)` through `(100,160)`, but still had no
  named render milestone or useful UI output. The reduced 30/60 s pixel extent
  versus the previous null-`lParam` run is recorded as a fidelity-cost
  observation, not UI progress.
- Added raw `WriteFile` result/last-error fidelity for host-backed file
  handles. `WriteFile` now clears thread last-error on success and reports
  `ERROR_ACCESS_DENIED` when the handle is valid but not writable, while
  existing invalid-handle errors are preserved. File-system coverage now proves
  an existing `GENERIC_READ | GENERIC_WRITE` host-backed file stays streamed
  and writes through at the current cursor without preloading the file, and raw
  COREDLL coverage verifies the guest `BOOL`, bytes-written pointer, last-error,
  and host contents for both writable and read-only handles. Validation:
  `cargo fmt`, focused raw/file tests, non-incremental
  `cargo check --features unicorn,trace,win32-desktop`, and full
  `cargo test --features unicorn,trace,win32-desktop` pass. Mounted validation
  wrote `target\writefile_lasterror_virtual_150s_*`; it stayed memory-stable
  (`heap_live=13697/13300954B`, `virtual_live=3/196608B`,
  `host_open=665`, `host_read=80198/4056903B`, `mem_open=3`,
  `max_read=685080`) and preserved the source `config.bin` SHA-256
  `1F04AE1349063D3A79F74733B233D8872F9A0D808309C33158DCF2EF9A86188A`.
- The 150 s virtual iNavi probe is now confirmed real UI progress rather than
  a black or fake frame: framebuffer `target\writefile_lasterror_virtual_150s.ppm`
  contains the iNavi SE splash art, produced by guest GDI memory-DC composition
  followed by a real screen `BitBlt` to HWND `0x00020008`. Later trace evidence
  shows additional offscreen DIB/StretchBlt/BitBlt work into an 800x54 memory
  surface and invalidation of hidden/effectively invisible child HWND
  `0x0002006c`, but no later display-surface blit or named render milestone.
- Added file-open access/disposition fields to the monitor file summary. The
  follow-up mounted probe `target\createfile_access_virtual_150s_*` shows
  `SDMMC Disk\iNaviData\config.bin` is opened with `req=0x40000000`
  (`GENERIC_WRITE`) and `pos=0x00000003` (`OPEN_EXISTING`), yet the write still
  reports zero bytes and the host file hash remains unchanged. Because the raw
  fixture proves writable host-backed handles write through, this is now
  evidence that the active mounted run is getting a non-writable host handle,
  likely from current host/sandbox permissions around the external SD-card
  dump, not from a guest read-only open.
- Added the first CE-backed GDI text/font query slice. Raw
  `CreateFontIndirectW` now copies the CE `LOGFONTW` fields needed by layout
  callers instead of storing only the guest pointer, DC text getters report
  `GetTextAlign`/`GetTextColor`, and raw `GetTextMetricsW`,
  `GetTextExtentExPointW`, and `GetTextFaceW` fill guest buffers with
  deterministic CE-shaped metrics/extent/face data for the selected font or
  stock `SYSTEM_FONT`. Focused coverage
  `coredll_raw_text_metrics_use_selected_logfont` and
  `coredll_raw_get_text_extent_ex_point_fills_fit_dx_and_size` pass, and the
  raw GWE/GDI suite now has 83 passing tests. This is query/layout fidelity;
  actual glyph rasterization and richer font enumeration/fallback remain open.
- Mounted validation after the text/font query slice wrote
  `target\text_metrics_virtual_60s_*`. The 60 s virtual/tap run stayed
  memory/file-I/O bounded (`heap_live=7445/22372164B`, `virtual_live=2/131072B`,
  `host_open=206`, `host_read=33843/2075303B`, `mem_open=2`,
  `max_read=497178`) and preserved the same populated framebuffer
  (`1151398` nonzero RGB bytes). It stopped at `pc=0x00b27b20` with
  `sendsig=0/sendcand=0`; the trace tail remains repeated `RSImage`
  `LoadPNG/Create`, `GetDC(hwnd=0x00020004)`, `CreateDIBSection`, and
  `ReleaseDC`, with no named render milestone. This narrows basic text metrics
  as a missing API class, but the active UI gate remains generic
  resource/GDI/presentation lifecycle after loaded surfaces.
- Added generic PC/RA region labels to Unicorn debug snapshots. Stop summaries
  now annotate `pc` and `ra` with mapped image/DLL/trap/heap/virtual regions,
  making bounded mounted runs immediately distinguish guest app loops from DLL,
  trampoline, and heap/virtual execution. A 30 s virtual trace run wrote
  `target\pc_region_virtual_30s_*` and showed the apparent mystery stop is
  still inside the iNavi image (`pc=0x00339c60(image:iNavi.exe+0x329c60)`),
  in an RGB-to-16bpp conversion loop. A no-trace release virtual run then
  reached the post-preload idle frontier quickly and stopped at
  `GetMessageW` (`pc=0x7fff0b60(ce-import-traps+0xb60)`,
  `ra=0x60024834(dll:mfcce400.dll+0x24834)`,
  `blocked_get_message=thread:1 hwnd=any`) with memory still bounded
  (`heap_live=13705/30071199B`, `host_open=665`,
  `host_read=80129/4059846B`). Current evidence says debug+trace startup is
  dominated by guest image conversion instrumentation, while release/no-trace
  reaches the scheduler/GWE message-wake frontier.
- Added CE-shaped `COMMTIMEOUTS` state to serial device handles. Raw
  `SetCommTimeouts` now reads and stores the five SDK DWORD fields on the
  opened `DeviceSession`, raw `GetCommTimeouts` round-trips them, and the
  Unicorn serial `ReadFile` bridge uses the configured total read timeout to
  park empty serial reads as finite scheduler waits. Default all-zero timeouts
  preserve the old blocking behavior, while `ReadIntervalTimeout=MAXDWORD` with
  zero total timeout becomes a nonblocking empty read. Focused coverage
  `serial_comm_timeouts_control_empty_read_parking` and
  `coredll_raw_comm_timeouts_round_trip_on_serial_handle` pass, along with the
  Unicorn wait scheduler suite.
- Mounted validation after the COM timeout slice wrote
  `target\comm_timeout_virtual_probe.*`. The previous 60 s startup frontier
  was a main-thread polling loop waiting on thread 6 while that worker was
  parked in an infinite serial read. With stored COM timeouts, the worker
  handles now exit (`WaitForSingleObjectThread` reports exit 0 for threads 5
  through 15), scheduler counters stay compact (`wait:68/19/0`, `sleep:46`,
  `block:88`, `wake:44`), and the run advances into real GDI/resource work:
  thousands of `CreateDIBSection`, `BitBlt`, `SelectObject`, and region calls
  compose into the 800x480 framebuffer. The new frontier is a guest null read
  at `pc=0x0002c264(image:iNavi.exe+0x1c264)`,
  `ra=0x0002c25c(image:iNavi.exe+0x1c25c)`, with only the main thread sleeping
  (`blocked_waits=[id=47/thr=1/kind=sleep/timeout=101]`). Memory and file I/O
  remain bounded (`heap_live=6499/20643900B`, `host_open=906`,
  `host_read=82486/6043980B`, `mem_open=7`, `max_read=685080`).

- Fixed the Unicorn serial `ReadFile` self-block timeout case. When the active
  guest thread parks on an empty serial read with a finite CE
  `COMMTIMEOUTS` read-total timeout and no suspended peer context is available,
  the bridge now completes the timeout on that same thread, writes zero bytes
  transferred, restores the saved callee registers, and resumes at the original
  return PC instead of leaving a stale `serial_read` waiter that can never be
  selected while it is still the active thread. Focused coverage
  `finite_serial_read_timeout_without_peer_completes_current_thread`, the full
  Unicorn wait scheduler suite, remote-serial tests, `cargo check`, and release
  build pass. Host Win32/tap validation wrote
  `target\host_serial_timeout_fix_*`: the final framebuffer is real map UI,
  the previous `id=58/thr=6/kind=serial_read/timeout=1000` leak is gone, and
  the remaining 180 s frontier is normal scheduler state
  (`blocked_get_message=thread:1`, finite worker sleeps, and one active timer)
  rather than the serial-read timeout leak.

- Fixed current-thread `GetMessageW` long-timer completion. The long wait path
  previously slept host time but did not advance the CE tick count, then called
  the short-timer helper which refused delays above the fast-forward threshold.
  It now advances `kernel.timers`, pumps GWE timers, and completes the parked
  current-thread `GetMessageW` from the resulting message queue. Focused
  coverage `current_get_message_long_timer_wait_returns_timer_message` and the
  existing `current_msg_wait_long_timer_writes_message_wait_result` pass, and
  the release host Win32/tap run `target\host_getmsg_timer_fix_*` confirms real
  progression: timer `4565` is delivered repeatedly at CE ticks `156239`,
  `186129`, `216014`, `245911`, `275799`, `305482`, and `335378`, GDI presents
  continue, and scheduler activity increases to `block:131/wake:64`. The new
  frontier is no longer a timer ANR; it is a guest control-flow fault at
  `pc=0x000e9d0c(image:iNavi.exe+0xd8d0c)` returning through `ra=0`.

- Fixed a real post-map scheduler corruption source by making the Unicorn
  scheduler timeslice hook skip MIPS branch/jump/call delay-slot PCs. The
  previous host Win32/tap frontier could suspend a guest worker at
  `0x0014e128`, the delay slot immediately before a later
  `READ_UNMAPPED addr=0x00000008` at `0x0014e12c`. Resuming a delay slot as a
  standalone PC loses the branch semantics, so scheduler switches now avoid
  any instruction whose previous instruction is a MIPS control transfer
  (`j`, `jal`, `jr`, `jalr`, normal/likely branches, and COP branch forms).
  Focused `timeslice` tests pass, release build passes, and the visible
  host Win32 validation `target\host_delayfix_180s_*` ran to the 180 s wall
  stop without the old `READ_UNMAPPED`/`ra=0` ANR. The final framebuffer is a
  real populated map UI with an app modal warning about GPS initialization
  abnormal behavior and reset recommendation (`Error Code: -14`). Memory and
  I/O stay bounded (`heap_live=14964/31470818B`, `host_read=83673/6454222B`,
  `mem_open=4`), and GWE cross-thread sends complete
  (`send:476 done:476 timeout:0 dead:0`). The current frontier is app/device
  fidelity around GPS/serial/system-state reporting, not map rendering or
  scheduler delay-slot corruption.

- Fixed the matching current-thread `WaitForMultipleObjects` finite-timeout
  bridge for the Unicorn syscall path. The bridge now lets a valid
  wait-any-only multiple wait whose handles are all waitable but not currently
  signaled complete on the active guest context when the finite timeout fits
  the remaining host wall budget: it sleeps host time, advances the CE tick
  count, returns `WAIT_TIMEOUT`, clears last error, and resumes at the saved
  return PC instead of registering an unselectable blocked waiter and stopping
  the emulator. The guard still leaves invalid handles, `wait_all`,
  zero-timeout polling, and immediately signaled objects on the existing raw
  kernel path. Focused coverage
  `current_multiple_wait_timeout_writes_wait_timeout_result` and the full
  Unicorn wait scheduler suite pass.

- Advanced the mounted route-search path through the first companion process
  handoff. Parked child CPUs now retain their own Unicorn state and
  `switch_to_next_parked_child_process` restores the child process identity
  (`GetModuleFileNameW(0)` now reports
  `\SDMMC Disk\INavi\happyway_win.exe`) when the parent reaches an encoded
  process exit. Focused coverage
  `switch_to_parked_child_restores_process_identity` passes. The required
  host run with `--remote-server 192.168.0.39:8765` no longer dies at the
  parent/child boundary; `happyway_win.exe` activates with process id 67 and
  continues executing.

- Added generic COREDLL narrow `vsprintf @1146` and NLS
  `IsValidLocale @209` support. `vsprintf` reads a guest `va_list` using the
  existing narrow printf formatter, and `IsValidLocale` accepts CE/Win32
  installed/supported flags for default and known LCIDs including Korean
  `0x0412`. Focused raw dispatch tests
  `coredll_raw_vsprintf_reads_guest_va_list` and
  `coredll_raw_is_valid_locale_accepts_korean_and_defaults` pass, along with
  `cargo check --features unicorn,trace,win32-desktop` and the release
  build. Mounted evidence: `target\route_drive_vsprintf1_*` stops moved from
  `COREDLL.dll@1146` to `COREDLL.dll@209`, and
  `target\route_drive_fast_locale1_*` moved past both imports.

- Current route-search validation reaches the real map UI, dismisses the
  bottom safety/notice bar, opens the real
  `목적지 / 현위치 안내 정보` search modal, and keeps the process alive after
  selecting/tapping rows. After parent handoff, however, remote hit-testing
  routes later touches to `happyway_win.exe` thread 3 (`hwnd=0x0002000c`)
  while the presented framebuffer still shows the parent iNavi modal. The new
  execution frontier is guest code inside `happyway_win.exe` at
  `pc=0x0003eac4(image:happyway_win.exe+0x2eac4)`, not a missing import.

- Fixed two generic route-search handoff issues. `TerminateProcess` on the
  current-process pseudo handle now destroys GWE windows owned by the current
  process before marking the pseudo process signaled, matching the cleanup path
  already used for launched child exits. The Win32 host presenter window is now
  a fixed-size wrapper around the 800x480 CE client area, so hovering the host
  window no longer advertises resize cursors. Focused coverage
  `terminate_current_process_destroys_owned_windows` passes, along with the
  parked-child identity test, `cargo check`, and release build.

- Fixed the `happyway_win.exe` large-heap frontier. The previous route run
  faulted at `happyway_win.exe+0x2eac4` writing inside a valid 16 MiB
  `HeapAlloc(HEAP_ZERO_MEMORY, 0x01000000)` block because heap spillover
  mapping could leave the tail of a newly allocated block unmapped. Unicorn
  heap growth is now mapped through the page-aware guest range mapper after
  import dispatch, and remap failures stop at the import boundary instead of
  resuming with a partial heap. Mounted host validation
  `target\route_drive_heapmap1_*` with `--remote-server 192.168.0.39:8765`
  moves past the former `WRITE_UNMAPPED` and renders the child
  `happyway_win.exe` iNavi SE splash while staying responsive at roughly
  170-180 MiB RSS.

- Advanced the host/live multiprocess scheduler around route startup. Parked
  process handoff now detects ready `GetMessage` and non-timeout blocked waits
  before idling the Win32 host presenter, and Unicorn can resume a saved
  blocked wait/GetMessage context when a parked CPU is re-entered with no
  running guest thread. The first broad version also treated timed-out parked
  waits as immediate readiness and caused a duplicate `happyway_win.exe`
  launch; that was tightened so only signaled/input/send/serial readiness
  preempts. Focused coverage
  `rotate_to_ready_parked_wait_process`,
  `rotate_to_ready_parked_kernel_wait_process`,
  `get_message_resume_from_no_running_thread_consumes_posted_input`, and the
  live-pump finite wait/sleep timeout tests pass. Mounted host validation with
  `--remote-server 192.168.0.39:8765` reaches the viable
  `happyway_win.exe` + `iSearch.exe` path without the duplicate companion.
  Current live evidence from `target\route_ready_wait2_170s.png` and the
  remote summaries: the app remains responsive under 300 MiB RSS, process
  traces cycle through `iNavi.exe`, `happyway_win.exe`, and `iSearch.exe`, but
  the route chrome is still not actionable because all 28 children under
  `hwnd=0x00020004` are guest-hidden while the app continues the
  `resmapi_800x480.bin` / `RSImage LoadPNG` startup path.

- Added stateful shell notification APIs at the raw COREDLL boundary.
  `SHNotificationAddI`, `SHNotificationUpdateI`, `SHNotificationRemoveI`, and
  `SHNotificationGetDataI` now decode the CE `SHNOTIFICATIONDATA` marshalled
  struct, key notifications by `CLSID + dwID`, preserve title/HTML/icon/
  duration/sink/lParam state, return CE-style Win32 error codes, copy stored
  data back into guest buffers, and post a CE-shaped `WM_NOTIFY`/`NMSHN`
  callback payload to the sink HWND for stored notification events. Focused
  coverage `shnotification_i_tracks_query_update_and_remove_state` passes.
  This keeps shell-visible notification data and callbacks honest without
  synthesizing taskbar bubble UI.

- Extended the raw shell notification callback payload to cover CE
  `SHNN_LINKSEL` string marshalling. The queued `NMSHN` heap block now grows to
  include an optional UTF-16 link string immediately after the 28-byte payload,
  writes `pszLink` to that receiver-local string, and releases the allocation
  through the existing `DispatchMessageW` payload cleanup. Focused coverage
  remains `shnotification_i_tracks_query_update_and_remove_state`.

- Added the matching CE `SHNN_DISMISS` callback marshalling helper. Dismiss
  callbacks now reuse the stored notification `lParam`, write the `fTimeout`
  flag into the `NMSHN` union DWORD, and keep the notification record alive
  until an explicit remove or window/process cleanup. Focused coverage remains
  `shnotification_i_tracks_query_update_and_remove_state`.

- Added shell notification lifetime cleanup on HWND destruction. The kernel
  destroy-window and process-window teardown paths now prune `Shell_NotifyIcon`
  records and `SHNotification*I` records whose sink window is being destroyed,
  so stale shell notification state does not survive normal CE window/process
  lifetime. Focused coverage
  `shell_window_destroy_removes_notify_icon_and_notification_state` passes.

- Added `host_progs\cmd.cpp`, a small Windows CE/eVC4-friendly command prompt
  utility. It uses a GUI `WinMain` shell instead of a console subsystem,
  supports `/C` and `/K`, implements basic desktop-cmd-like built-ins
  (`cd`, `dir`, `type`, `echo`, `copy`, `del`, `mkdir`, `rmdir`, `cls`,
  `pwd`, `exit`), and falls back to `CreateProcessW` for external commands.
  Manual eVC4 MIPSII validation succeeded with `CLMIPS.EXE` and `LINK.EXE`,
  producing `target\host_progs\cmd\cmd.exe`; the committed artifact is source
  only, with generated objects/exe under ignored `target\`.

## False Leads

- A process-directory fallback for rooted `CreateFileW` paths was tested and
  removed before commit. Windows CE loader code does search the current EXE
  directory for DLL/module names, but ordinary FSDMGR file opens canonicalize
  the supplied path and resolve it through the mount table/root filesystem.

## Regressions

- None yet.
