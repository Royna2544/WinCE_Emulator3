# SOURCE_REFERENCES

Bounded source references used to shape emulator behavior. These are evidence
anchors, not app-specific shortcuts.

Artifact note: `target\` was cleared on 2026-06-04 to recover disk space.
Source references below remain authoritative anchors; any mentioned
`target\...` artifact names are historical unless regenerated.

Runtime DLL note: mounted iNavi execution should load DLL images from
`D:\INAVI_Emulator\DUMPPLZ\Windows`. SDK import libraries and CE/MFC source
trees remain behavior/reference evidence, not the primary runtime DLL source.

## Runtime PE/DLL Loading

- Dumped runtime DLL bytes:
  `D:\INAVI_Emulator\DUMPPLZ\Windows\commctrl.dll`
  - The real target `commctrl.dll` has a base-relocation data directory at RVA
    `0x00076000` with size `0x7e7c`, while `SizeOfImage` is `0x0007f000` and
    no section owns that RVA. Mapped PE semantics still zero-fill all image
    memory below `SizeOfImage`, so parser reads through directory terminators
    and strings must treat unbacked mapped bytes as zero instead of rejecting
    the image.
  - Startup loader validation now maps dumped `mfcce400.dll` and dumped
    `commctrl.dll` from `--dll-search-dir`; COREDLL remains emulator-provided,
    while WINSOCK/OLE stay shimmed until subsystem-backed behavior is required
    by fixtures or trace evidence. Import patching resolves loaded external
    DLL exports before shim classification, and `commctrl.dll`/`commctrlce.dll`
    are not classified as emulator common-controls shims; they must come from
    the configured runtime DLL search paths.
  - Dumped `mfcce400.dll` imports `commctrl.dll` by ordinal through its IAT
    (`addr 0x000674d0` in `llvm-objdump -p`). When MFC is loaded before
    `commctrl.dll`, a single-pass loader leaves ordinal thunks such as
    `0x80000002` in the IAT, and MFC later executes that marker through
    `jalr`. v3 now keeps COREDLL trap patching intact, then does a second
    external import pass after all loaded dumped DLL exports are known, so
    cross-DLL imports by name and ordinal resolve to the loaded DLL export VA.
  - Runtime `LoadLibraryW` / `LoadLibraryExW(flags=0)` now uses the same
    dumped-DLL byte source and CE-aware search order at the Unicorn import-trap
    boundary. The first runtime slice maps/relocates normal MIPS DLL images,
    recursively loads non-emulator dependencies, patches COREDLL imports to
    trap slots, patches imports against already loaded guest DLL exports,
    refreshes the live and persisted import trap page, and registers exports
    with the kernel module table. TLS callbacks and
    `DllMain(DLL_PROCESS_ATTACH)` run before returning from normal loads, and
    final dynamic `FreeLibrary` enters guest TLS callbacks followed by
    `DllMain(DLL_PROCESS_DETACH)` before marking the module unload-pending.
    Forwarded exports are retained from the PE export directory and resolved
    through already-loaded guest modules or CE search/load of the forwarded-to
    DLL, including runtime `GetProcAddress` and import-patching paths. CE's
    loader paths treat export names as literal strings and preserve load
    failure state instead of repairing malformed targets, so v3 now rejects
    malformed forwarded-export strings with whitespace around the module or
    symbol before they can resolve through normal module/name normalization.
    CE `CORE\DLL\loader.cpp` unwinds libraries loaded while resolving an
    import block when `DoImports` fails; v3 now records runtime modules loaded
    during one `LoadLibraryW` attempt and marks only those new modules
    unload-pending if dependency load/import patching or lifecycle callout setup
    fails. The same file's `UnDoDepends` path frees imported dependency
    modules after module detach; v3 now retains direct dependency refs while
    loading and releases the reachable dependency chain after final dynamic
    `FreeLibrary` detach planning. Failed runtime DLL maps now unwind the
    current image's `MemoryMap`/Unicorn reservation before propagating late
    map, write, forwarded-export, trap-page, metadata, or resource failures,
    and v3 parses current-image resources before committing trap pages,
    trampoline metadata, module records, or resource handles. CE `CallDllMain`
    sets `ERROR_DLL_INIT_FAILED` when `DllMain(DLL_PROCESS_ATTACH)` returns
    false and then `DoImportAndCallDllMain` frees the failed module; v3 now
    records the same last-error shape and releases load-attempt module refs
    after a guest attach entry point returns false. The same CE `DoLoadLibrary`
    path increments an already loaded module's refcount and clears
    `DONT_RESOLVE_DLL_REFERENCES` when a later request has no no-import flags;
    v3 now mirrors that loaded-module promotion for raw and runtime reuse paths.
    CE `int_LoadLibraryExW` rejects non-null `hFile` with
    `ERROR_INVALID_PARAMETER`, which v3 now applies in raw and runtime
    `LoadLibraryExW` dispatch.
    CE `CORE\DLL\loader.cpp` also implements `DisableThreadLibraryCalls` by
    setting `MF_NO_THREAD_CALLS` on the resolved module, and its
    `ThreadNotifyDLLs` loop calls loaded DLL entrypoints for
    `DLL_THREAD_ATTACH`/`DLL_THREAD_DETACH` only when modules are imported,
    not loading, and not marked no-thread-calls. v3 now persists that
    loaded-module flag in raw coredll dispatch and uses it, together with
    no-import/datafile flags and load order, when Unicorn handles
    `ThreadAttachAllDLLs` and `ThreadDetachAllDLLs`.
    The same file's `ProcessDetach` loop drains loaded DLL refcounts by the
    current minimum positive refcount while walking load order and calls
    `DllMain(DLL_PROCESS_DETACH)` only when an imported module reaches zero;
    this keeps dependent modules detached before their higher-ref dependencies.
    v3 now uses that ordering for runtime `ProcessDetachAllDLLs` and releases
    the included per-process module refs after the guest lifecycle callouts
    complete, while still excluding no-import/datafile modules. Raw
    `ProcessDetachAllDLLs` cannot enter guest code, but it now uses the same
    imported-module eligibility to drain loaded-module refs and leaves
    no-import/datafile modules visible.
    `LoadLibraryExW(DONT_RESOLVE_DLL_REFERENCES)` now maps/reuses the runtime
    DLL and publishes ordinary exports without recursive dependency loading,
    import patching, TLS callbacks, `DllMain`, or detach callouts on final
    release. CE `loader.c` treats `LOAD_LIBRARY_AS_DATAFILE` as also setting
    `DONT_RESOLVE_DLL_REFERENCES`; v3 mirrors that for runtime datafile loads,
    maps the image for resource access, registers resource strings/data
    immediately with `kernel.resources`, and keeps code exports hidden from
    `GetProcAddress`. The raw/non-Unicorn helper cannot map new guest bytes,
    but it now mirrors the already-loaded-module reuse/refcount behavior for
    supported no-resolve/datafile flags, keeps datafile-flagged loaded-module
    exports hidden from raw `GetProcAddress` name/ordinal lookups, and fails
    missing raw loads explicitly.
    Runtime executable DLL loads now run the same MIPS Unicorn trampoline
    patcher before map/write and publish inline stub ranges into live
    full-code-hook metadata so high/relocated DLL branch/call sites can execute
    through generated stubs in the same run slice. Loader counters are now
    carried on `CeKernel` and surfaced through `UnicornDebugSnapshot` so mounted
    runs can compactly report load attempts, maps, dependencies, export
    lookups, forwarders, lifecycle calls, and loud failures.

- CE loader lifecycle anchors:
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\loader.c` and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\thread.c`;
  user-mode thread notification behavior is anchored in
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\CORE\DLL\loader.cpp`
  - `NKFreeLibrary` routes final unload through the module unlock/unload path
    with `fCallDllMain`; process/thread exit paths notify
    `DLL_PROCESS_DETACH` through `NKPSLNotify`. v3's runtime Unicorn import
    path now mirrors the observable final dynamic `FreeLibrary` contract by
    calling guest TLS callbacks and the DLL entrypoint with
    `DLL_PROCESS_DETACH` before marking the loaded module unload-pending and
    leaving its mapped address range reserved. The callback-before-entrypoint
    ordering is covered by the PE TLS eVC fixture.
  - Host-presented probes of dumped `explorer.exe` using this same runtime DLL
    directory no longer fail on the old high-address trampoline from
    `0x00057108` to `0xffff832c`. After the COREDLL startup ordinal slice, the
    latest probe reaches the emulator sentinel (`pc=0x7ffffff0`) rather than a
    missing import trap.

## Windows CE Core OS

- COREDLL stub audit diagnostics:
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\coredll.h`, CE API-set headers, and the
  generated COREDLL ordinal/export evidence remain the boundary for deciding
  whether an ordinal is implemented, a safe failure, or a must-implement stub.
  v3 now carries raw import-trap context through this boundary, including
  thread id, caller PC, trap PC, and caller-module attribution when the guest
  return address falls inside a mapped process image or runtime DLL blob. This
  is diagnostic evidence for subsystem ports; it must not become app-specific
  behavior or a reason to return plausible fake success for shell/UI/process
  loader calls. Must-implement raw fallbacks therefore record
  `ERROR_NOT_SUPPORTED` for the caller thread and use explicit failure-shaped
  return values until the specific CE API behavior is implemented.

- COREDLL CRT route-search import:
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\INC\crt_ordinals.h` and
  `C:\WINCE600\PUBLIC\COMMON\OAK\LIB\MIPSII\RETAIL\coredll.def`
  - CE maps `_ORDINAL_wcstoul` to ordinal 1083, and the MIPSII COREDLL export
    file lists `wcstoul @1083 PRIVATE`. v3 uses that ordinal for the raw
    COREDLL boundary reached by mounted iNavi route/current-location parsing.
    The implementation parses guest wide strings and writes `endptr` in
    two-byte character units.

- Clipboard allocation-copy API:
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winuser.h`
  - CE declares `SetClipboardData`, `GetClipboardData`, and
    `GetClipboardDataAlloc` together in the clipboard API group. v3 now routes
    raw `GetClipboardDataAlloc` through the same GWE clipboard format store as
    `GetClipboardData`, allocates a fresh local handle for emulator-tracked
    local clipboard data, copies guest bytes by the tracked local allocation
    size, and fails unknown source handles explicitly instead of inventing a
    size. `SetClipboardData(format, NULL)` is retained as a delayed-render
    format: a later `GetClipboardData`/`GetClipboardDataAlloc` queues
    `WM_RENDERFORMAT` to the clipboard owner and accepts the owner-supplied
    handle only while the owner thread is actively servicing the matching
    `WM_RENDERFORMAT`, without requiring the owner to reopen the clipboard;
    repeated reads while that owner render is already active do not queue duplicate
    `WM_RENDERFORMAT` sends, and an abandoned render callback clears the active
    render marker so a later read can request the delayed format again. Replacing
    clipboard contents with `EmptyClipboard` posts `WM_DESTROYCLIPBOARD` to the
    previous live owner, and destroying the owner/open clipboard window clears
    stale ownership/open-window state while dropping unresolved delayed
    formats. If the destroyed owner still has delayed formats, the destroy path
    sends `WM_RENDERALLFORMATS` first and preserves already-rendered data that
    remains in the clipboard store.

- Caret APIs:
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winuser.h`
  - CE declares `CreateCaret`, `DestroyCaret`, `ShowCaret`, `HideCaret`,
    `SetCaretPos`, blink-time accessors, and system-wide caret enable/disable
    together in the GWE-facing user API group. v3 now keeps the raw caret state
    in GWE and marks the visible caret's old/new client rectangle dirty when
    those calls make it appear, disappear, move, or change system-enable
    visibility. Framebuffer-backed raw dispatch also marks the translated
    screen rectangle dirty so host presentation can repaint the affected
    region without claiming full bitmap/blink rendering is complete.

- Shell shortcut APIs:
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\shellapi.h`,
  `C:\WINCE600\PUBLIC\SHELL\OAK\HPC\CESHELL\API\shortcut.cpp`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\LIB\MIPSII\RETAIL\coredll.def`
  - The SDK prototypes expose `SHCreateShortcut`, `SHCreateShortcutEx`, and
    `SHGetShortcutTarget`, while the MIPSII COREDLL export file maps the raw
    ordinals used by iNavi-era callers. CE's `Shortcut_Write` stores shortcut
    files as UTF-8 text with a BOM, decimal character count, `#`, and a quoted
    target executable plus optional arguments. v3 now implements the raw
    `SHCreateShortcut`, `SHGetShortcutTarget`, and `SHCreateShortcutEx` paths
    against mounted CE files with `CREATE_NEW` behavior, buffer-fit validation,
    and bounded unique-name generation for existing shortcut names. The
    `ShellExecuteEx` shortcut launch path now preserves the shortcut's stored
    argument tail and appends explicit `lpParameters` after it.

- Shell recent-document API:
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\shellapi.h`,
  `C:\WINCE600\PUBLIC\SHELL\OAK\HPC\CESHELL\API\api.cpp`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\LIB\MIPSII\RETAIL\coredll.def`
  - `shellapi.h` defines `SHARD_PATH`, `SHARD_PIDL`, and
    `SHAddToRecentDocs`. The HPC shell implementation resolves
    `CSIDL_RECENT`, clears the Recent folder when the item pointer is null,
    and for path-backed documents creates a `.lnk` named from the document
    file stem using the same CE shortcut writer. v3 now mirrors the
    `SHARD_PATH` and null-clear behavior through mounted CE filesystem calls,
    records recent-document state in `ShellSystem`, and explicitly fails PIDL
    input until namespace PIDL support exists.

- Shell file-change notification APIs:
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\shellsdk.h`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\shsdkstc.h`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winbase.h`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winnt.h`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\mwinbase.h`,
  `C:\WINCE600\PUBLIC\SHELL\OAK\HPC\CESHELL\UI\filechangemgr.cpp`, and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\pathapi.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\volumeapi.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\fileapi.cpp`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\LIB\MIPSII\RETAIL\coredll.def`
  - `shellsdk.h` exposes `SHChangeNotifyRegister`,
    `SHChangeNotifyDeregister`, and `SHChangeNotifyFree`; `shsdkstc.h`
    defines `SHCHANGENOTIFYENTRY`, `FILECHANGEINFO`, and
    `FILECHANGENOTIFY`. The HPC shell file-change manager maps filesystem
    notifications into `SHCNE_*` events for registered shell views. v3 now
    implements the raw `SHChangeNotifyRegisterI`, `SHFileNotifyRemoveI`, and
    `SHFileNotifyFreeI` ordinals as durable shell state with HWND cleanup and
    posts CE-style `WM_FILECHANGEINFO` path or PIDL payloads from matching
    filesystem mutations.
  - `winbase.h` declares `FindFirstChangeNotificationW`,
    `FindNextChangeNotification`, `CeGetFileNotificationInfo`, and
    `FindCloseChangeNotification`; `winnt.h` defines the
    `FILE_NOTIFY_CHANGE_*` masks, and `mwinbase.h` maps the public trap ids.
    FSDMGR `pathapi.cpp` canonicalizes the watched path, resolves the owning
    volume, and routes to `AFS_FindFirstChangeNotificationW`; `volumeapi.cpp`
    creates a notification event only for an existing directory and passes
    `NotifyFilter` through unchanged to `NotifyCreateEvent`; `fsnotify.cpp`
    stores that value in `NOTEVENTENTRY::dwFlags` and later matches generated
    known change bits with `pEvent->dwFlags & dwFlags`, while `fileapi.cpp`
    shows file writes notifying `FILE_NOTIFY_CHANGE_LAST_WRITE`.
    Rust now creates waitable directory-change handles for the public and
    direct AFS first-change ordinals, canonicalizes public
    `FindFirstChangeNotificationW` watch paths before directory validation and
    registration, preserves unknown caller filter bits instead of rejecting
    them up front, signals them from matching create/delete/rename/write/
    attribute directory events through the same known-bit gate, exposes pending action/name records through
    `CeGetFileNotificationInfo` using the standard
    `FILE_NOTIFY_INFORMATION` layout, including recursive directory-create,
    rename old/new, and directory-removal records, coalesces consecutive
    duplicate action/name records plus transient create/delete and
    modified/delete churn, and rearms/closes them through
    `FindNextChangeNotification`/`FindCloseChangeNotification`. Because
    `pathapi.cpp` carries the caller's `fWatchSubTree` value into the AFS/
    FSDMGR notification event, watches now preserve the same subtree boundary:
    non-recursive watches match immediate children only, while recursive
    watches match deeper descendants. `fsnotify.cpp` `NotifyMoveFileEx`
    compares source and destination parent paths, reports same-parent moves as
    `FILE_ACTION_RENAMED_OLD_NAME`/`FILE_ACTION_RENAMED_NEW_NAME`, and reports
    cross-parent moves as `FILE_ACTION_REMOVED`/`FILE_ACTION_ADDED`; v3 now
    applies that move-boundary mapping to raw file-change notifications,
    including the file-vs-directory notify-filter choice. In the
    `NotifyPathChangeEx` directory removal/rename branch, CE also signals a
    watcher on the directory being removed or renamed with a current-directory
    record (`FILE_ACTION_REMOVED`, `"\\"`); v3 now applies that exact record
    shape for self-watched directory removal and rename-old notifications.
    `PRIVATE\WINCEOS\COREOS\STORAGE\NOTIFY\fsnotify.cpp` `NotifyReset` drains
    only the records that fit the caller buffer, sets `ERROR_MORE_DATA` after a
    successful prefix copy, reports remaining bytes through `lpBytesAvailable`,
    re-signals the event while records remain, returns
    `ERROR_INSUFFICIENT_BUFFER` when the first record cannot fit, and returns
    `ERROR_NO_MORE_ITEMS` for data fetches with no pending notifications; v3 now
    mirrors those byte-count and re-signal semantics. CE computes each copied
    `FILE_NOTIFY_INFORMATION` length as header plus `FileNameLength` plus a
    trailing NUL WCHAR, DWORD-aligns that total, copies the NUL into the guest
    buffer, and leaves `FileNameLength` as the non-NUL byte count; v3 now uses
    that same sizing and copy shape. A null `lpBuffer` with a nonzero length
    large enough for the first record reaches the same guarded write path in CE
    and fails with `ERROR_INVALID_PARAMETER`, so v3 now lets the fit calculation
    proceed by length and reports the invalid guest pointer without draining
    pending records. When no records are pending, the CE `NotifyReset` data
    fetch path writes `0` to `lpBytesReturned` before `lpBytesAvailable` inside
    a guarded block, swallows output-pointer faults, and still reports
    `ERROR_NO_MORE_ITEMS`; v3 now mirrors the observable bad nonzero output
    pointer order and last-error result. The same `NotifyReset` path decrements
    one outstanding event count for no-buffer
    `FindNextChangeNotification` calls, so v3 keeps an outstanding signal count
    separate from detailed records instead of clearing all queued records on a
    single reset. `NotifyCloseChangeHandle` duplicates the caller's event
    handle with `DUPLICATE_CLOSE_SOURCE` before checking the event data, so v3
    now closes valid-but-wrong handle types before returning
    `ERROR_INVALID_HANDLE` from `FindCloseChangeNotification`. The same CE
    path depends on real `DuplicateHandle` ownership transfer instead of a
    handle-value alias; raw `DuplicateHandle` now validates the source handle,
    target pointer, and option mask, creates an independent local handle-table
    entry, preserves duplicated file-change notification handles after
    `DUPLICATE_CLOSE_SOURCE` consumes the source handle, and assigns duplicated
    notification handles to the requested target process so only that process
    can wait/reset/read/close the duplicate. `NotifyCreateEvent`
    stores the caller process in `NOTEVENTENTRY::hProc`, duplicates the
    internal notification handle into that process, and returns an event handle
    owned by that same process. `pathapi.cpp` supplies that process as
    `GetCallerVMProcessId()` for `FSEXT_FindFirstChangeNotificationW`,
    `GetCurrentProcessId()` for `FSINT_FindFirstChangeNotificationW`, and passes
    it through as `hProc` to `AFS_FindFirstChangeNotificationW`; v3 now records
    the creating process on public file-change notification handles, honors a
    nonzero direct AFS `hProc` owner, and rejects foreign-process wait,
    `FindNextChangeNotification`, `CeGetFileNotificationInfo`,
    `DuplicateHandle`, and `FindCloseChangeNotification` attempts.
    CE `NotifyCreateEvent` inserts events under the `NOTVOLENTRY` for the
    resolved volume, and `NotifyPathChangeEx`/`NotifyMoveFileEx` receive that
    same `hVolume` from FSDMGR before walking directory events. v3 now stores
    the resolved mounted root on each file-change notification handle and
    requires later changes to come from the same mounted volume unless the
    watch is the root namespace; same-parent mounted renames return CE old/new
    rename records to the owning volume, cross-parent mounted renames return
    CE remove/add records to the owning volume, same-child paths on other
    mounted volumes remain quiet, and recursive root watchers still report
    mounted-volume-prefixed old/new or remove/add paths such as
    `ResidentFlash\watch\old.bin` and
    `ResidentFlash\watch\src\move.bin`.
    `mounttable.cpp` also calls `NotifyPathChange` with `FILE_ACTION_ADDED` or
    `FILE_ACTION_REMOVED` for visible mount folders on the root notification
    handle; v3 mirrors that for root-directory waiters when guest roots are
    mounted or unmounted. Deeper FSDMGR volume-handle ownership and remaining
    owner/reset/close lifetime edges remain queued.
    FSDMGR `fileapi.cpp` calls `NotifyHandleChange(FILE_NOTIFY_CHANGE_LAST_WRITE)`
    after successful writes, and `fsnotify.cpp` marks that handle changed so
    `NotifyCloseHandle` emits `FILE_ACTION_CHANGE_COMPLETED` on close with the
    CE attribute/size/write/access/creation filter mask. v3 now tracks changed
    file handles, reports the close-completed detailed record, and keeps later
    remove coalescing from leaking stale write/completion churn.
    FSDMGR `FS_MoveFileW` also canonicalizes source/destination paths and
    compares their owning volumes: same-volume moves call `AFS_MoveFileW`,
    cross-volume file moves are emulated with `CopyFileW` plus source delete,
    and CE explicitly succeeds the `MoveFileW` call even if that source delete
    fails after the copy. Cross-volume directory moves fail with
    `ERROR_NOT_SAME_DEVICE`, source mount-point renames fail with
    `ERROR_ACCESS_DENIED`, and destination mount-point collisions fail with
    `ERROR_ALREADY_EXISTS`; v3 now mirrors those mounted-boundary cases for raw
    `MoveFileW`, including successful copy-without-delete moves off read-only
    mounted media and destination-only file-change notification records when
    the source remains.
    CE `FS_DeleteAndRenameFileW` and `FSDMGR_DeleteAndRenameFileW` implement
    the public `DeleteAndRenameFile(old, new)`/direct
    `AFS_PrestoChangoFileName(old, new)` shape by deleting the old path and
    moving the new path into that old name, requiring both paths to resolve to
    the same volume. FSDMGR then emits a destination delete notification
    followed by `NotifyMoveFile(source, destination)`, so v3 now routes both raw
    ordinals through a dedicated delete-and-rename helper instead of a
    delete-then-regular-`MoveFileW` shim.
    `pathapi.cpp` rejects direct mount-root mutations before dispatching to
    the filesystem: `FS_CreateDirectoryW` reports `ERROR_ALREADY_EXISTS` for a
    mount root, `FS_DeleteFileW` reports `ERROR_FILE_NOT_FOUND`, and
    `FS_RemoveDirectoryW`/`FS_SetFileAttributesW` report
    `ERROR_ACCESS_DENIED`. `fileapi.cpp`/`pathapi.cpp` also return
    `ERROR_ACCESS_DENIED` for storage access-check failures on mutating
    operations, and the `fsnotify.cpp` `NotifyPathChangeEx`/`NotifyMoveFileEx`
    paths are change notifications emitted after filesystem changes rather
    than for failed access checks. v3 now mirrors those mounted-root errors,
    reports access denied for raw create/copy/delete/directory/rename/
    set-attributes attempts against read-only mounted host roots, and verifies
    those failed mutations do not signal matching change-notification handles
    or enqueue detailed `CeGetFileNotificationInfo` records.
    `virtroot.cpp` skips `AFS_FLAG_HIDDEN` mount folders while enumerating the
    merged root directory and marks `AFS_FLAG_SYSTEM` mount folders with
    `FILE_ATTRIBUTE_SYSTEM`; `pathapi.cpp` also ORs `FILE_ATTRIBUTE_SYSTEM`
    into all file/directory attributes returned from a system volume, while
    exact hidden mount-root probes keep `FILE_ATTRIBUTE_HIDDEN`.
    `volumeapi.cpp` folds `AFS_FLAG_HIDDEN` and `AFS_FLAG_SYSTEM` into
    `CE_VOLUME_INFO.dwAttributes`, while `storemgr.h` defines the standard
    read-only and removable `CE_VOLUME_ATTRIBUTE_*` bits supplied by disk/store
    metadata. v3 now applies the system attribute to nested host-backed files
    and directories under system mounts and has raw coverage for hidden-root
    enumeration suppression, exact hidden mount attributes, system/hidden
    `CeGetVolumeInfoW` attributes, and read-only/removable volume attributes
    through both `CeGetVolumeInfoW` and `CeFsIoControlW(FSCTL_GET_VOLUME_INFO)`.

- Shell popup-menu APIs:
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winuser.h`,
  `C:\WINCE600\PUBLIC\SHELL\OAK\HPC\CESHELL\UI\defshellview.cpp`, and
  `C:\WINCE600\PUBLIC\SHELL\OAK\HPC\EXPLORER\TASKBAR\extrasmenu.cpp`
  - CE exposes `TPM_NONOTIFY`, `TPM_RETURNCMD`, and the menu-loop messages
    `WM_ENTERMENULOOP`, `WM_INITMENUPOPUP`, and `WM_EXITMENULOOP`. Shell code
    uses `TrackPopupMenu` both for blocking notification-style popups and for
    `TPM_RETURNCMD` command queries. v3 now records popup tracking and sends
    the CE owner notification sequence unless `TPM_NONOTIFY` is set, returns
    the enabled default command (or first enabled command, including commands
    inside enabled `MF_POPUP` submenus) for state-backed command queries,
    sends/records nested `WM_INITMENUPOPUP` notifications for the submenu path
    that produced that command, and covers the no-selectable-item cancellation
    shape by returning `0` without synthesizing `WM_COMMAND`. Framebuffer-backed
    raw `TrackPopupMenuEx` calls also paint popup and highlighted child-submenu
    surfaces from the current menu item state, including disabled/default/checked
    rows, separators, and submenu markers. Current-cursor row hit-testing now
    overrides the default/first-command path when the cursor lands on an
    enabled command row, enabled submenu row, or command row inside a rendered
    highlighted child submenu. v3 also consumes already queued owner-window
    `WM_KEYDOWN` Up/Down/Enter/Escape messages for popup navigation, skips
    disabled/separator rows, accepts the highlighted command or submenu command
    on Enter, opens highlighted submenu panes on Right-arrow for multi-level
    keyboard navigation and Enter selection, returns keyboard focus one pane
    upward on Left-arrow, closes a child pane when pointer movement returns
    to a parent row, and treats Escape inside a child pane as a one-level close,
    accepts queued `WM_CHAR` menu mnemonics from `&`-marked item text,
    stores the current row in menu `MF_HILITE` state with matching framebuffer
    highlight repainting, consumes `WM_MOUSEMOVE` over enabled rows to update that highlighted
    command before Enter, and returns `0` for Escape cancellation. Queued owner-window
    `WM_CANCELMODE` and outside-click messages also cancel, while click
    messages inside the popup select enabled rows from their screen-coordinate
    `lParam`; right-button popup selection is only consumed when
    `TPM_RIGHTBUTTON` is set. Queued posted/sent same-thread non-owner
    messages, owner-window non-menu messages, and generated owner `WM_PAINT`
    are now dispatched before default or later owner-window menu input resolves.
    CE `winuser.h` defines `TPM_CENTERALIGN`, `TPM_RIGHTALIGN`,
    `TPM_VCENTERALIGN`, and `TPM_BOTTOMALIGN`; v3 now applies those anchor
    transforms to top-level popup placement before painting and hit-testing.
    CE `winuser.h` also defines `TPMPARAMS.rcExclude` as a screen-coordinate
    rectangle to exclude while positioning `TrackPopupMenuEx`; v3 applies that
    rectangle after initial alignment, including the below-candidate placement,
    right-side fallback when the below/above placements still intersect the
    excluded screen rectangle after clamping, left-side fallback when the right
    candidate clamps back into the excluded region, and the final
    first-clamped-candidate fallback when every placement still intersects the
    excluded rectangle.
    A full nested modal menu pump and remaining live submenu cancellation/routing
    edge cases remain queued.

- Accelerator input APIs:
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winuser.h`
  - CE defines `FVIRTKEY`, `FSHIFT`, `FCONTROL`, `FALT`,
    `TranslateAcceleratorW`, and `WM_SYSKEYDOWN` in the same input/menu API
    surface. v3 now uses those modifier bits when matching raw
    `TranslateAcceleratorW` entries against `MSG` records and GWE key state,
    including the Alt/syskey path, instead of matching solely on the virtual
    key; non-`FVIRTKEY` ASCII accelerators now compare against the translated
    key character so shifted punctuation such as `!` can match the accelerator
    table entry.

- Keyboard translation APIs:
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winuser.h`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\imm.h`, and
  `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\main.cpp`
  - CE defines `TranslateMessage(CONST MSG *pMsg)`, `WM_CHAR`,
    `WM_SYSKEYDOWN`, `WM_SYSCHAR`, `VK_SHIFT`, and `VK_CAPITAL` in the same
    input API surface. v3 now routes raw `TranslateMessage` through GWE key
    state for the ASCII letter/digit/space path, control-character keys
    (`Backspace`, `Tab`, `Return`, `Escape`), Ctrl-letter and Ctrl-OEM control
    codes, numpad digits/operators, common OEM punctuation keys with Shift
    variants, Shift/CapsLock casing, and the syskey-to-`WM_SYSCHAR` message
    path. CE's keyboard-layout and IMM headers also expose
    `GetKeyboardLayout*`, `LoadKeyboardLayoutW`,
    `ActivateKeyboardLayout`, `ImmGetKeyboardLayout`, HIMC context,
    open/conversion-status, `ImmIsIME`, `ImmGetIMEFileNameW`, and
    `ImmDisableIME`; the CE GDI tests call `ImmDisableIME(0)` before drawing
    work so IME UI does not interfere. v3 now keeps an explicit GWE keyboard
    layout/KLID and minimal HIMC state for those raw probes, while still
    reporting no pending composition data. Broader layout-specific keymaps,
    dead keys, candidate/composition UI, and IME handoff remain queued.

- Old MIPS CE kernel-call encoding:
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\INC\nkmips.h`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\process.c`, and corroborating
  v2 notes/code in `..\WinCE_Emulator_v2\PROGRESS.md`,
  `..\WinCE_Emulator_v2\src\ce_kernel.cpp`, and
  `..\WinCE_Emulator_v2\src\synthetic_dll.cpp`
  - CE's old MIPS direct API-call encoding maps target `0xfffff3fa` to process
    API set method 2, `TerminateProcess`. `DeviceParser.exe` reaches this via
    `addiu t0,$zero,-0xc06; jalr t0`, with the process handle in `a0` and the
    exit code in `a1`. v3 decodes that pattern at the Unicorn/CE boundary even
    when Unicorn reports it as an interrupt/zero-PC stop, then routes it
    through the same current-process terminate path as raw COREDLL
    `TerminateProcess`.

- CE process and message scheduler frontier:
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\schedule.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\thread.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\process.c` where present, and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\cmsgque.h`
  - `CreateProcessW` must create independent waitable process/thread objects;
    a child that parks in its own wait/message loop is still `STILL_ACTIVE`.
    The mounted `target\route_drive_procfix1_*` trace now preserves that handle
    state for `happyway_win.exe` and `iSearch.exe`, but v3 still needs real
    parked child CPU ownership and handoff when the active process exits.
    Pending synchronous sends, including the observed thread-9
    `SendMessageW`, must resume through the same scheduler/GWE send-state
    model instead of being stranded at host process teardown.

- GWE message-queue order:
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\cmsgque.h`
  - CE's queue search order is posted messages, received message queue,
    sent-message stack, paint list, then quit. v3 mirrors this for
    `GetMessageW` and removing `PeekMessageW`, so ordinary posted mouse/timer/
    private messages are not starved behind received synchronous sends.
  - CE sent messages are processed as queue-internal work, not returned to the
    caller as ordinary `MSG` records. v3 now uses that boundary for the route
    helper IPC path: `GetMessageW` can enter the target guest WNDPROC for a
    ready sent message, complete the GWE transaction, then resume the original
    `GetMessageW` import so the receiver continues its normal message loop.

- GWE/GDI region and window-region behavior:
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\wingdi.h`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winuser.h`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\window.hpp`, and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\gweapiset1.hpp`
  - CE exposes region status as `NULLREGION`, `SIMPLEREGION`, and
    `COMPLEXREGION`; difference/intersection/union operations must therefore
    preserve multi-rectangle region shape where a single bounding box would
    make holes clickable/paintable. v3 keeps a bounding rect for old callers,
    but the authoritative region state is now a normalized rect list used by
    `CombineRgn`, point/rect tests, clipping status, `SetWindowRgn`, and
    `GetWindowRgn`.
  - Selected GDI clip regions must also draw through that rect list, not just
    through the bounding box. `FillRect`, polygon/polyline drawing, memory and
    display blits, stretch blits, and transparent-image composition now
    intersect every primitive with each selected region rect so
    `CombineRgn(RGN_DIFF)` holes remain unpainted on memory DIBs and display
    HDCs. Focused fixture:
    `coredll_raw_fill_rect_respects_complex_clip_holes_on_memory_dib`.
  - `SetWindowRgn(HWND, HRGN, BOOL)` consumes the region shape owned by GWE and
    only requests redraw when the third argument is nonzero. v3 now mirrors
    that boundary generically instead of invalidating every region change.
  - `SetWindowPos`/`MoveWindow` redraw behavior is controlled by the CE
    `SWP_NOREDRAW`/repaint flags rather than by app-specific knowledge. v3 now
    invalidates already-visible windows after move/size/z-order changes unless
    no-redraw was requested, preserving the normal `WM_PAINT` path for child
    bands and promoted controls.
  - CE SDK `winuser.h` defines `WS_CLIPSIBLINGS` and `WS_CLIPCHILDREN` as
    window style bits. v3 now applies those bits at paint-clip time:
    `WS_CLIPCHILDREN` subtracts visible child client regions from parent HDC
    painting, and child windows only clip against higher z-order siblings when
    they requested `WS_CLIPSIBLINGS`. Top-level windows still clip against
    higher z-order top-levels.

- GDI font resource loading:
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\wingdi.h` and
  `C:\WINCE600\PUBLIC\COMMON\OAK\LIB\MIPSII\RETAIL\coredll.def`
  - `wingdi.h` declares `AddFontResourceW(LPCWSTR)` as a count-returning GDI
    API, and dumped MultiTBT reaches COREDLL ordinal `893`
    (`AddFontResourceW`) with `\SDMMC Disk\TBT\ygo550.ttf`. v3 currently
    implements the CE boundary enough to validate the mounted guest file path
    and return one installed font on success, or zero with the normal last
    error on invalid/missing paths. Full font engine registration remains a
    later GDI/text fidelity item.

- MultiTBT runtime evidence:
  `D:\INAVI_Emulator\INAVI\TBT\MultiTBT.exe` and
  `D:\INAVI_Emulator\INAVI\mapdata\Manager.xml`
  - The mounted image set includes `TBT\MultiTBT.exe`, `TBTResData.bin`, and
    `ygo550.ttf`; `Manager.xml` records those files in the iNavi G3 install
    payload. A DUMPPLZ text search found only CE `TBTCORE` feature macros in
    `D:\INAVI_Emulator\DUMPPLZ\Windows\ceconfig.h`, not an observed in-app
    `CreateProcessW` launch of `MultiTBT.exe`. Treat v2 companion launch usage
    as harness evidence until a mounted trace proves a guest launch path.

- Kernel thread/scheduler stack evidence:
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\schedule.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\thread.c`, and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winbase.h`
  - CE thread creation/resume/wait behavior is scheduler owned. v3 still uses
    an emulator-managed guest stack reservation for mapped Unicorn worker
    contexts, but each resumed worker must have enough downward stack headroom
    for normal MIPS prologue stores before full CE stack guard/commit fidelity
    is implemented. The mounted `target\window_region_complex_virtual_150s_*`
    crash exposed this as a generic stack-slot layout bug; the follow-up
    4 MiB reserve keeps later worker slots inside mapped guest stack memory.

- Scheduler-backed message input waits:
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\schedule.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\cmsgque.h`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\GWEAPI\msgqueue.cpp`, and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winuser.h`
  - Remote REST touches and host mouse input must be routed into the same GWE
    message queue and scheduler wake path regardless of whether a diagnostic
    CPU wall-clock limit is active. v3 now drains those inputs from the normal
    Unicorn live tick before the optional wall-stop check, so no-wall host
    launches stay interactive while guest code is executing.
  - Host/presenter input must enter the same CE message queue and scheduler
    waiter path as guest/remote input. v3 now drains newly polled host input
    into the blocked raw `GetMessageW` thread/window when the host run loop is
    parked, which queues a scheduler message-wake candidate instead of
    returning from the syscall directly in host code.
  - `MsgWaitForMultipleObjectsEx` exposes `MWMO_INPUTAVAILABLE` and CE queue
    masks such as `QS_TIMER`; the message wait result is
    `WAIT_OBJECT_0 + nCount` when input wakes the call. v3 now applies that
    shape to raw timer waits by advancing only to timers due within the
    requested timeout and leaving later timers pending. The Unicorn import
    bridge uses the same result shape for current-thread timer wakes and
    timeout waits that fit the active host run budget, while over-budget waits
    remain scheduler-owned.
  - A CE thread has a single scheduler-owned blocked state. When the Unicorn
    bridge parks a guest thread on `GetMessageW`, `MsgWaitForMultipleObjectsEx`,
    or a blocking object wait, it must remove any stale saved wait for that
    same thread before registering the new one. When the UI thread parks on an
    empty `GetMessageW`, the bridge must still let other blocked guest threads
    whose finite timeouts mature inside the host run budget resume, instead of
    treating UI idle as a whole-process stop. v3 has two saved-context stores
    for this bridge while the run-queue port is incomplete, so stale cleanup
    must clear both vector-backed blocked waits and the separate blocked
    `GetMessageW` thread slot before registering the next empty-queue wait.
  - CE scheduling is preemptive across runnable threads, not cooperative only
    at imports and wait calls. Until v3 grows full saved-context run queues, the
    Unicorn bridge now has a conservative time-slice that swaps the active
    running guest context with the already-suspended peer context at bounded
    code-hook intervals, while avoiding import traps, trampoline pages, and
    pending WNDPROC returns. The same time-slice can now preempt the active
    running context into the suspended slot and resume a ready blocked waiter
    when no existing suspended peer would be overwritten. This is still a
    bridge, not the final CE run queue: v3 now adds a FIFO overflow queue behind
    the primary suspended slot for time-slice/ready-waiter preemption, so an
    active context can be preserved even when another runnable context is
    already suspended. Other guest-thread handoff paths still need to converge
    onto this queue before the bridge becomes a full CE ready-run model.

- GWE paint/update and MFC paint pumping:
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\cmsgque.h`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\window.hpp`,
  `C:\Program Files (x86)\Microsoft Visual Studio 8\VC\ce\atlmfc\src\mfc\thrdcore.cpp`,
  `C:\Program Files (x86)\Microsoft Visual Studio 8\VC\ce\atlmfc\src\mfc\wincore.cpp`,
  and `..\WinCE_Emulator_v2\src\synthetic_dll.cpp`
  - CE GWE tracks paint requests as queue-visible update state; `BeginPaint`
    consumes/validates the update, while erase state is separate from the paint
    request. MFC's message pump treats `WM_PAINT` as non-idle work and uses
    `UpdateWindow` during idle layout/update paths. v2 corroborated the
    emulator-shaped ordering of synchronous `WM_ERASEBKGND` before `WM_PAINT`
    in `UpdateWindow`, but v3 keeps the behavior generic: no forced app pixels
    and no hidden-child paint. The mounted `target\update_erase_virtual_*`
    trace confirms this ordering unlocks a real memory-DC-to-window-HDC
    `BitBlt` and a populated iNavi framebuffer.
  - The same CE paint-request shape means hidden windows should not keep
    queue-visible stale paint work from a previous visible state. MFC
    `wincore.cpp` `CWnd::SendMessageToDescendants` does still enumerate hidden
    permanent children for idle UI updates, so v3 must not skip the children;
    instead, GWE now clears a window's own pending update/erase state when it
    is hidden and clips surviving update rectangles when `SetWindowPos`
    changes client bounds. Mounted evidence
    `target\hide_update_clear_virtual_20s_*` confirms immediately hidden
    `AfxWnd42u` children no longer carry full-screen create-time dirty
    rectangles.
  - Direct raw/kernel `UpdateWindow` now uses effective HWND visibility through
    the same ancestor-aware model as `IsWindowVisible`, rather than only the
    target's direct `WS_VISIBLE` bit. This keeps CE paint forcing from sending
    `WM_PAINT` into a child whose parent is hidden; mounted evidence
    `target\update_effective_visibility_virtual_150s_*` confirms the current
    iNavi `0x0002006c` child remains a hidden offscreen-composition target,
    not a window v3 should force-paint.
  - Synthetic `WM_PAINT` selection for `GetMessageW`/`PeekMessageW` now uses
    the same effective ancestor-aware visibility for both unfiltered queues and
    explicit-HWND filters. This keeps filtered `PeekMessage(hwnd, WM_PAINT,
    WM_PAINT, ...)` from exposing paint for windows CE would consider hidden
    through an ancestor.
  - CE `window.hpp` models `m_hrgnUpdate` as `Invalid /\ Visible`, and
    `cmsgque.h` keeps paint requests as queue-visible work. v3 now lets hidden
    windows remember a simplified pending update rectangle for later visible
    presentation, but it does not set the changed `QS_PAINT` bit until the HWND
    is effectively visible. This keeps `MsgWaitForMultipleObjectsEx`/queue
    wakeups aligned with paint messages that `GetMessageW` can actually
    synthesize.
  - CE `window.hpp` also carries `fPendingSizeMove`, documented as waiting to
    send `WM_SIZE` and `WM_MOVE` when `ShowWindow` happens. v3 now preserves
    `WM_WINDOWPOSCHANGED` for hidden geometry changes, but defers direct-hidden
    `WM_MOVE`/`WM_SIZE` until a later direct show, rather than delivering size
    and move messages into an HWND that is still hidden.
  - CE `window.hpp` exposes `DestroyWindow_I`, `ShowWindow_I`,
    `SetWindowPos_I`, `InvalidateRect_I`, `UpdateWindow_I`, and
    `RedrawWindow_I` over the same window update-region state. v3 now treats a
    destroyed/hidden/moved visible window as exposing its old screen rectangle:
    surviving visible windows intersecting that rectangle are marked dirty
    with clipped client update bounds, so repaint happens through normal
    `WM_PAINT`/`BeginPaint` instead of leaving stale presenter pixels behind.
  - CE SDK `winuser.h` anchors `HWND_TOP`, `GW_HWNDFIRST`/`GW_HWNDNEXT`,
    `WindowFromPoint`, mouse messages, `WM_ACTIVATE`, `WM_SETFOCUS`, and
    `WM_KILLFOCUS`. Top-level windows created later should be frontmost until
    z-order changes move them, while child/owned-window relationships remain
    distinct. MFC `wincore.cpp` corroborates that mouse/activation paths expect
    the hit window to own focus before normal button handling; v3 now activates
    and focuses the hit HWND on remote `WM_LBUTTONDOWN` rather than leaving
    focus on an older overlapping top-level.

- Explorer/COREDLL startup ordinals:
  `C:\WINCE600\PUBLIC\COMMON\OAK\LIB\MIPSII\RETAIL\coredll.def`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\LIB\MIPSII\RETAIL\k.coredll.def`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\CORE\DLL\core_common.def`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\INC\crt_ordinals.h`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\syncobj.c`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\shellapi.h`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\strsafe.h`, and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winuser.h`
  - Dumped `explorer.exe` startup identified real COREDLL import needs rather
    than app-specific shortcuts. The ordinal sources map
    `__security_gen_cookie2 @2696`, `OpenEventW @1496`,
    `SHGetSpecialFolderPath @295`, `CopyFileW @164`, `StringCchCatW @1693`,
    `StringCbCatW @1694`, `wcsncmp @65`, and `DestroyIcon @725`.
  - `syncobj.c` anchors `OpenEventW` as an existing named event open with
    access validation. `shellapi.h` anchors the CE CSIDL values used by
    `SHGetSpecialFolderPath` (`Desktop`, `Programs`, `Personal`, `Favorites`,
    `Startup`, `Recent`, `Start Menu`, `DesktopDirectory`, `Fonts`, `AppData`,
    `Windows`, `Program Files`, and `Profile`), while HPC shell `api.cpp` and
    `NOUI\api.cpp` show callers masking `CSIDL_FLAG_CREATE` before forwarding
    creation intent to `SHGetSpecialFolderPath` and keeping virtual
    `CSIDL_DESKTOP` handling in `SHGetSpecialFolderLocation`. v3 consults
    `HKLM\System\Explorer\Shell Folders` first, uses CE-shaped fallbacks when
    the dump lacks those values, preserves the existing `CSIDL_SYSTEM`
    compatibility fallback, honors `fCreate`/`CSIDL_FLAG_CREATE` by creating
    the resolved root-relative or nested mounted folder, and rejects overlong
    resolved paths instead of truncating the fixed MAX_PATH output. HPC shell
    `recbin.cpp` uses
    `CSIDL_PROFILE`, while `shelldialogs.cpp` asks `CSIDL_DESKTOPDIRECTORY`
    with creation enabled; both call shapes are now covered by the raw tests.
    The same `shellapi.h` `SHFILEINFO`
    layout and `SHGFI_*` flags, plus HPC shell `api.cpp`, anchor the first
    `SHGetFileInfo` slices: v3 reads HKCR extension classes, writes display/
    type/attribute metadata, returns explicit generic icon handles/indexes,
    and follows the CE `iconcache.cpp`/`resource.h` default image-list order
    from `ceshapi_base.rc` for document, folder, application, storage-card,
    network-folder, and shortcut pseudo slots. Removable mount roots with
    `FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_TEMPORARY` use the storage-card
    branch and `Storage Card` type name, while temporary `\\share`,
    `\release`, and `\network` directories use the CE network-folder branch
    and `Network Folder` type name. `shelllistview.cpp`, `taskbar.cpp`, and
    `browse.cpp` call `SHGetFileInfo(TEXT(""), ..., SHGFI_SYSICONINDEX...)` to
    obtain the shared shell image list, so v3 now accepts empty-path
    system-image-list probes without trying to stat a file. Framebuffer- and
    selected-memory-DIB-backed raw `ImageList_Draw*` calls now paint clipped
    deterministic pseudo-icons for these image-list slots until real icon
    bitmap extraction is ported.
    HPC shell `api.cpp` also routes `SHGFI_ATTRIBUTES` through
    `IShellFolder::GetAttributesOf`, and `filefolder.cpp` maps filesystem
    attributes to `SFGAO_FILESYSTEM`, `SFGAO_FOLDER`, `SFGAO_LINK`, and
    `SFGAO_READONLY`; v3 mirrors that shell-attribute output instead of
    returning raw `FILE_ATTRIBUTE_*` values in `SHFILEINFO.dwAttributes`.
    `CESHELL\API\api.cpp` returns a system image-list handle whenever either
    `SHGFI_ICON` or `SHGFI_SYSICONINDEX` is requested, so raw `SHGetFileInfoW`
    now returns that list handle for icon-only queries while still populating
    `SHFILEINFO.hIcon`.
    File-backed `LoadImageW(..., IMAGE_BITMAP, LR_LOADFROMFILE)` now keeps the
    BMP pixel rows in CE heap-backed bitmap storage. Resource-backed
    `LoadImageW`/`LoadBitmapW` for RT_BITMAP DIB data also copies the resolved
    pixel rows into owned bitmap storage, preserving RGB masks and indexed
    color tables when present. Wide bitmap strips are split into per-index
    source rectangles when added to narrower image lists, and raw
    `ImageList_Draw*` blits those bitmap-backed image-list entries into the
    attached framebuffer or selected memory DIB before falling back to
    pseudo-icon rendering. CE `imagelist.cpp` also treats
    `ImageList_LoadImage(cx=0)` as `cx = bmHeight` before computing the strip
    count and leaves other invalid widths to `ImageList::Create`, so raw
    `ImageList_LoadImage` now uses the loaded bitmap height for zero-width
    requests instead of treating the entire bitmap width as one image, while
    negative widths fail through the image-list creation gate.
    The same CE path builds creation flags from `crMask != CLR_NONE` and
    `bm.bmBitsPixel & ILC_COLORMASK`, so raw `ImageList_LoadImage` now marks
    masked loads with `ILC_MASK` and records the loaded bitmap color depth on
    the image list. It calls `ImageList::Add` rather than
    `ImageList::AddMasked` when `crMask == CLR_NONE`, so raw
    `ImageList_LoadImage` now leaves unmasked entries with mask handle `0` and
    no transparent color. `ImageList_AddMasked` and the `ImageList_LoadImage` mask-color argument now
    preserve a transparent color for BMP-backed entries, so masked
    framebuffer draws skip matching source pixels while still rendering
    non-transparent pixels. `ImageList_Add` with a bitmap `hbmMask` now also
    reads that mask bitmap during framebuffer draws and skips non-black mask
    pixels, preserving the distinct mask-handle path from color-key masking.
    CE `ImageList::Add` first resolves the source bitmap with
    `GetObjectW_I`, rejects `bm.bmWidth < m_cx`, and copies image/mask pixels
    into the image-list-owned DCs. Raw `ImageList_Add`, `ImageList_AddMasked`,
    and `ImageList_Replace` now snapshot real source/mask bitmap pixels into
    owned backing bitmaps, so deleting the caller-owned source bitmap after an
    add does not invalidate later image-list draws; metadata-only pseudo
    handles remain preserved for existing non-rendering paths. CE
    `ImageList::Duplicate` copies the backing image bitmap with
    `CopyDIBBitmap`, optionally copies the mask with `CopyBitmap`, then copies
    count, allocation, color, and overlay metadata into the new list. Raw
    `ImageList_Duplicate` now deep-copies bitmap-backed image/mask entries
    into fresh owned bitmap storage while preserving pseudo-handle entries.
    CE `ImageList::Cleanup` deletes `m_hbmImage` and `m_hbmMask` through
    `ImageList::DeleteBitmap` before freeing the list. Raw `ImageList_Destroy`
    now releases cloned image-list-owned bitmap handles and their heap-backed
    bits while preserving caller-owned bitmap handles.
    CE `ImageList::Replace` and `ImageList::Remove` overwrite or remove image
    pixels from the list backing storage, with remove/set-count paths possibly
    calling `ReAllocBitmaps`; CE `ImageList::ReplaceIcon` allocates an image
    slot when needed and draws the icon color/mask into that slot. Local raw
    replace, replace-icon, remove, size reset, and image-count truncation now
    delete cloned per-entry backing bitmaps only after no remaining image-list
    entry references those handles.
    CE `ImageList::SetBkColor` updates `m_clrBk` and calls `ResetBkColor`
    for existing images; `ResetBkColor` applies mask-driven ROPs so mask-on
    background slots become black for `CLR_NONE`/black, white for white, or
    the selected background color. Raw `ImageList_SetBkColor` now mirrors that
    for bitmap-backed masked entries.
    CE `ImageList::GetIcon` allocates color and mono mask bitmaps, draws the
    image-list entry once with `ILD_MASK | flags` and once with
    `ILD_TRANSPARENT | flags`, then creates an icon from those bitmaps. Raw
    `ImageList_GetIcon` now does the same for bitmap-backed entries with real
    backing storage while preserving local pseudo handles for synthetic entries.
    CE `pcommctr.h::OVERLAYMASKTOINDEX` and `imagelist.cpp::DrawIndirect`
    limit overlay-mask drawing to `NUM_OVERLAY_IMAGES` entries, so synthetic
    shell/system image-list pseudo handles and pseudo rendering now ignore
    overlay nibbles above slot four instead of encoding or painting them.
    Because CE deletes the temporary `hbmMask` and `hbmColor` immediately after
    `CreateIconIndirect_I`, local `CreateIconIndirect` now copies readable
    caller bitmap backing into icon-owned storage, and `DestroyIcon` releases
    only bitmaps marked as icon-owned. This also lets `ImageList_GetIcon` and
    PE-extracted icons free their owned backing storage without deleting caller
    source bitmaps.
    The local
    resource image list now returns failure for invalid or undersized source
    bitmaps on both `ImageList_Add` and `ImageList_AddMasked` instead of
    reporting success with zero appended strips. CE `ImageList::ReplaceIcon` also rejects
    indexes below `-1` before appending or drawing, so local `ReplaceIcon`
    now keeps `-1` as the only append sentinel. CE `ImageList::GetIconSize`
    uses reviewed-parameter validation and returns failure when either output
    pointer is null before writing `cx` or `cy`, so raw `ImageList_GetIconSize`
    now avoids partial dimension writes on null outputs. CE `ImageList::Merge`
    builds `rc1`/`rc2`, uses `UnionRect` for the merged list dimensions, and
    combines mask/color creation flags from both lists, so local merged image
    lists now expose CE-sized metadata for offset and mixed-size merges.
    Bitmap-backed `ImageList_Draw*` also resolves `ILD_OVERLAYMASK` through
    `ImageList_SetOverlayImage` and composites the registered overlay image
    through the same transparent-color or mask-bitmap path. CE `commctrl.h`
    defines `IMAGELISTDRAWPARAMS` as a 56-byte structure, and `imagelist.cpp`
    rejects `ImageList_DrawIndirect` calls whose `cbSize` is not exactly
    `sizeof(IMAGELISTDRAWPARAMS)`; raw `ImageList_DrawIndirect` now applies
    that exact-size gate before recording draw state or rendering. CE's
    `ImageList::Draw` wrapper initializes `rgbBk` and `rgbFg` to
    `CLR_DEFAULT` before delegating to `DrawIndirect`, so raw `ImageList_Draw`
    now uses the image-list background-color default instead of forcing
    transparent drawing. The same CE path applies `xBitmap`/`yBitmap` to the source rectangle before
    defaulting zero `cx`/`cy`, and resolves `rgbBk == CLR_DEFAULT` to the
    image-list background color; v3 now normalizes raw draw parameters in
    that order and writes the mutated `cx`, `cy`, `rgbBk`, and `fStyle`
    fields back into the caller's `IMAGELISTDRAWPARAMS`, matching CE's
    first-stage in-place parameter updates. When `ILD_OVERLAYMASK` is present,
    CE then rewrites the same struct for the overlay pass (`i`, `x`, `y`,
    `cx`, `cy`, and `fStyle`, preserving only `ILD_MASK`, forcing
    `ILD_TRANSPARENT`, and adding the overlay metadata flags); v3 now writes
    those final overlay-pass values for raw `ImageList_DrawIndirect` too.
    `imagelist.cpp` also uses a private `ILD_BLENDMASK` value of
    `0x000E`, so `ILD_BLEND75` still enters the blend setup; the 16-bit and
    color-table blend helpers treat non-`ILD_BLEND50` blend styles as the 25%
    branch. Bitmap-backed image-list draws now use that CE mask shape instead
    of ignoring the `ILD_BLEND75` bit. For `rgbFg == CLR_NONE`, the CE 16-bit
    color-image blend path copies the destination into a work buffer and
    averages source pixels with existing destination pixels; bitmap-backed
    selected-DIB and framebuffer draws now model that destination-blend branch.
    When the same `rgbFg == CLR_NONE` blend setup is used with `ILD_MASK`,
    `imagelist.cpp` instead ORs the mask with the mono dither brush, forces
    `ILD_TRANSPARENT`, and reaches the mask draw's `SRCAND` path, so v3 now
    keeps mask-only blends on the dither/SRCAND branch rather than tinting mask
    pixels against the destination. The same
    `DrawIndirect` branch chooses `pimldp->dwRop` when `ILD_ROP` is combined
    with `ILD_MASK` or `ILD_IMAGE`, while `ILD_MASK` defaults to `SRCAND` for
    transparent draws and `SRCCOPY` otherwise; bitmap-backed image-list draws
    now carry `dwRop` and apply those CE raster-operation choices.
    CE `ImageList::CopyDitherImage` does not replace the destination image
    metadata; it computes the destination image rect, masks `fStyle` to
    `ILD_OVERLAYMASK`, draws the source into the destination image DC with
    `ILD_IMAGE`, optionally draws the source mask into the destination mask DC
    with `ILD_BLEND50 | ILD_MASK`, and resets the destination background color.
    v3 now records only the CE overlay style bits, preserves destination image
    records, mutates bitmap-backed destination image pixels, and updates
    destination mask pixels with CE's 8x8 `0xAAAAAAAA`/`0x55555555` 50% mono
    dither pattern ORed with the source mask before applying the final
    `SRCAND` step.
    `imagelist.cpp` treats only `ImageList_Remove(-1)` as remove-all; other
    negative indexes fall through the single-image branch and fail before any
    mutation. It clears `m_aOverlayIndexes` only on `ImageList_Remove(-1)`;
    its single-image `Remove` path calls `RemoveItemBitmap` and decrements
    `m_cImage` without rewriting overlay slots, and `SetImageCount` likewise
    only reallocates and updates `m_cImage`. v3 now rejects negative remove
    indexes below `-1`, preserves stale overlay indexes for single-image
    removal and truncation, and still clears overlays on remove-all.
    `SetImageCount` mutates `m_cImage` only after `ReAllocBitmaps` succeeds,
    and `ReAllocBitmaps` returns failure when the backing image/mask bitmap
    allocation fails; v3 now keeps the prior count and returns a raw
    invalid-parameter-shaped failure for impractical allocation requests.
    `ImageList::SetOverlayImage` also rejects
    lists whose mask DC is absent (`m_hdcMask == NULL`) and overlay slots
    outside CE's `1..=4` range after the `NUM_OVERLAY_IMAGES` check; v3 now
    returns failure for those unmasked or out-of-range overlay registrations
    instead of recording an overlay slot. The
    same CE function scans the mask bitmap for the black-pixel bounding box,
    leaves an all-white mask as a zero-width/zero-height overlay rooted at
    `(0, 0)`, and stores overlay x/y/dx/dy plus `ILD_IMAGE` only when the
    bounded area is fully opaque; v3 now records that metadata, keeps sparse
    non-rectangular overlay masks in the mask-driven draw path, and uses it
    when drawing overlays.
    During overlay drawing, CE preserves the caller's `ILD_MASK` bit, forces
    `ILD_TRANSPARENT`, applies the overlay metadata flags, and re-enters the
    draw path, so bitmap-backed overlay draws now continue into the overlay
    mask path instead of skipping overlays whenever `ILD_MASK` is set.
    `ImageList::SetIconSize` returns failure when the requested dimensions
    match the current size; otherwise it updates `m_cx`/`m_cy` and calls
    `ImageList::Remove(-1)`, so v3 now clears images and overlay slots when an
    image-list size change succeeds. `ImageList::DragMove` always returns
    `TRUE` after its optional visible-drag move block; v3 now reports success
    for no-active-drag `DragMove` calls while leaving drag state absent and
    only advances the stored drag point while an active drag image is visible.
    `ImageList::SetDragCursorImage` calls `MergeDragImages`, whose no-dither
    branch treats the missing drag-image setup as non-error, so v3 now returns
    success for no-active-drag cursor-image requests against valid image lists.
    `ImageList::GetDragImage` writes `s_DragContext.ptDrag` and
    `ptDragHotspot` before returning `s_DragContext.pimlDrag`; since CE
    initializes those fields to zero/null, v3 now returns null and zeroed
    point outputs before any drag context exists instead of surfacing a
    handle error. `ImageList::DragEnter` still records `hwndDC` and
    `ptDrag` when no drag image exists because it ignores the no-image
    `DragShowNolock(TRUE)` result, and `ImageList::DragLeave` clears that
    lock only for the matching window; v3 now preserves that no-active
    static drag point/lock state for subsequent `GetDragImage` calls.
    The `api.cpp` attribute-probe branch treats
    inaccessible UNC paths with access/network errors as
    `FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_TEMPORARY`, so raw
    `SHGetFileInfo` now returns the same network-folder type/icon metadata for
    missing UNC paths and writes `SFGAO_FILESYSTEM | SFGAO_FOLDER` when
    `SHGFI_ATTRIBUTES` is requested. The `api.cpp` unknown-file branch sets
    `ERROR_MOD_NOT_FOUND`, so raw `SHGetFileInfo` uses that failure shape for
    other missing non-`SHGFI_USEFILEATTRIBUTES` paths. The same `iconcache.cpp` `PathIsLink`
    loop and `GetType` branches anchor v3's `.lnk` `Shortcut` type name,
    bounded nested target-followed icon selection, and shortcut overlay pseudo
    `HICON` handles until real CE icon extraction/image-list behavior is
    ported. Raw user-created image lists also preserve overlay identity for
    `ImageList_GetIcon` through deterministic pseudo handles, matching the same
    overlay mapping used by `ImageList_Draw*`. It rejects unsupported/colliding flags such as
    `SHGFI_ICONLOCATION`,
    `SHGFI_ATTR_SPECIFIED`, `SHGFI_PIDL`, small-icon requests without
    icon/index output, and `SHGFI_ATTRIBUTES | SHGFI_USEFILEATTRIBUTES` with
    `ERROR_INVALID_FLAGS`. `strsafe.h` anchors the
    `StringCch*`
    character-count and `StringCb*` byte-count distinction plus truncation
    HRESULTs. `winuser.h` anchors the icon/cursor signatures; v3 currently
    has lightweight synthetic stock/shell pseudo icon handles plus
    `RT_GROUP_ICON` resource handles, and `DestroyIcon` now validates that
    namespace instead of accepting arbitrary nonzero/non-icon resource handles.
    Raw `CreateIconIndirect` also validates the CE `ICONINFO` mask/color bitmap
    handles, stores a state-backed synthetic icon object, and lets
    `DestroyIcon` release that tracked object.
    Raw `DrawIconEx` also validates the HDC/icon handle pair, paints a
    deterministic pseudo-icon into attached window framebuffers and selected
    memory DIBs for shell pseudo-icons, and draws bitmap-backed icons through
    the same bitmap renderer as image lists. Bitmap-backed framebuffer and
    selected-memory-DIB draws now scale from the icon bitmap's native source dimensions into
    caller-requested destination sizes, and bitmap-backed `DI_MASK` draws use
    the icon mask bitmap as the source instead of the color bitmap, including
    covered 1bpp mask-only framebuffer and selected-memory-DIB paths. CE
    `imagelist.cpp` uses `DrawIconEx_I(..., DI_NORMAL)` for image storage and
    `DrawIconEx_I(..., DI_MASK)` for mask storage when replacing an image-list
    icon. `pcommctr.h` defines CE's implemented image-list creation flag mask
    as `ILC_VALID`, excluding private placeholders such as `ILC_LARGESMALL`
    and `ILC_UNIQUE`; raw `ImageList_Create` now rejects flags outside that
    mask while preserving accepted private/shared flags on the image-list
    object. `imagelist.cpp` also converts a missing color-depth mask to
    `ILC_COLORDDB` for CE backward compatibility, so raw and direct
    image-list creation now store `ILC_COLORDDB` when callers pass
    `ILC_COLOR`/zero color bits. `commctrl.h` defines `ILCF_MOVE` and `ILCF_SWAP`, while
    `pcommctr.h` defines `ILCF_VALID` as `ILCF_SWAP`; CE `imagelist.cpp`
    rejects `ImageList_Copy` when the source and destination image-list
    pointers differ, requires valid source and destination image rectangles,
    and uses `ILCF_SWAP` only for a same-list slot exchange. Raw
    `ImageList_Copy` now matches that same-list-only behavior instead of
    treating flag zero as a cross-list move/removal path.
    Raw `ExtractIconExW` now supports `nIconIndex == -1` count probes,
    extracts PE `RT_GROUP_ICON`/`RT_ICON` resources into bitmap-backed icon
    handles, selects separate large/small icon entries from multi-size groups,
    counts and extracts string-named `RT_GROUP_ICON` group resources whose
    group directories reference ordinal `RT_ICON` payloads, resolves sparse
    integer `RT_GROUP_ICON` resource IDs when zero-based group enumeration does
    not cover the requested `nIconIndex`, fills successive large/small
    output-array slots, reports `ERROR_RESOURCE_NAME_NOT_FOUND`
    when a present group references malformed icon data or missing primary or
    secondary `RT_ICON` ordinal resources,
    tolerates present `RT_ICON` DIB payloads that have color pixels but omit
    trailing AND-mask bytes by creating a color-only icon, preserves color
    tables for covered 4bpp and 8bpp indexed `RT_ICON` payloads, and decodes
    4bpp high/low-nibble palette indexes when drawing extracted icons,
    and keeps the index-zero shell fallback for non-PE paths that writes the
    same synthetic `HICON` values that
    `SHGetFileInfo` would select, including CE shortcut overlays. CE
    `resource.cpp` `KernExtractIcons` loads the module as datafile, resolves
    `RT_GROUP_ICON` with `MAKEINTRESOURCE(nIconIndex)`, lets a callback choose
    large/small group-directory indices, then extracts the referenced
    `RT_ICON` resources. Raw `KernExtractIcons` now follows that integer
    group-resource lookup and copies selected `RT_ICON` payload bytes into
    guest heap outputs, rejects zero-based group enumeration when no matching
    integer `RT_GROUP_ICON` resource ID exists, and reports
    `ERROR_RESOURCE_NAME_NOT_FOUND` when neither large nor small output pointer
    is supplied; the non-Unicorn raw path uses the default `{0, 1}`
    group-entry selection because it cannot execute the CE callback.
    Missing paths fail with `ERROR_FILE_NOT_FOUND`; broader PE format variants
    and non-PE fallback edges remain queued.
  - `shellapi.h` defines `SHELLEXECUTEINFO`, `SEE_MASK_NOCLOSEPROCESS`,
    `nShow`, `hInstApp`, and `hProcess`. v3's raw `ShellExecuteEx` now
    preserves `nShow`, returns the queued process handle when
    `SEE_MASK_NOCLOSEPROCESS` is set, and feeds that show command into the
    child process entry context.
  - `shellapi.h` also defines `SHELLEXECUTEINFO.lpDirectory`, and
    `winbase.h` defines `CreateProcessW(..., LPWSTR pszCurDir, ...)`. v3 now
    preserves those explicit CE current-directory values in pending child
    launches, uses them for relative child executable lookup, falls back to the
    current CE process directory for relative `ShellExecuteEx` targets when
    `lpDirectory` is absent, and restores the effective current directory when
    parked processes are activated.
  - `PUBLIC\SHELL\OAK\FILES\shell.reg` uses registry association command
    templates such as `"\"%1\" %*"` for `exefile`, `explorer.exe -u%1` for
    `urlfile`, and `explorer.exe %1` for
    generic files. v3's `ShellExecuteEx` association path now treats `%*` as
    the explicit `lpParameters` insertion point and only appends parameters
    when the template did not include that placeholder, while the CE
    `urlfile` `-u%1` shape preserves the embedded target argument without
    inserting quotes between `-u` and the target; existing extensionless
    documents can use the CE `HKCR\file` command path, while missing
    extensionless targets, missing associated non-EXE documents, and
    association commands whose absolute executable target is missing still fail
    as file-not-found instead of queueing a plausible launch. Empty association
    command strings are treated as unusable associations and report
    `SE_ERR_NOASSOC`.
  - `winuser.h` anchors the public `MessageBoxW` signature, button groups,
    default-button flags, icon flags, `MB_SETFOREGROUND`, `MB_TOPMOST`,
    `MB_RTLREADING`, and `ID*` return codes. `OWNERDRAWLIB`
    `animlibbase.h` and `appalert.cpp` add the CE owner-draw `MB_YESALL` and
    `MB_CANCEL` alert modes, and `appalert.cpp` maps each alert mode onto
    left/center/right button labels before converting button messages into
    return IDs; v3's raw `MessageBoxW` validates that CE flag surface before
    creating transient dialog state, rejects unsupported desktop-only bits and
    undefined icon nibbles with `ERROR_INVALID_PARAMETER`, records the requested modal
    text/caption/style plus button IDs, those CE button slots/labels,
    default-button index, icon class, owner enabled-state, transient dialog and
    child-control HWNDs, active-dialog state, `MB_TOPMOST`'s CE
    `WS_EX_TOPMOST` extended style, and result, and now derives default return
    codes for both SDK and owner-draw button groups while closing the transient
    skeleton through `EndDialog`; framebuffer-backed raw calls also paint the
    dialog surface from the same caption, text, icon, and button-layout state.
    v3 also
    consumes already queued modal Enter/Escape key and character input, direct
    button and dialog-client hit-tested button-down/release input, dialog
    `WM_COMMAND`, dialog `WM_CLOSE`, and
    dialog `WM_SYSCOMMAND/SC_CLOSE` messages addressed to the transient
    dialog/buttons and routes them to the final `ID*` result, while dispatching
    queued posted and sent same-thread non-dialog messages before later modal
    dialog input resolves; generated non-dialog `WM_PAINT` is dispatched
    before default fallback selection, and generated dialog `WM_PAINT` is also
    dispatched so transient update regions are validated instead of being
    repeatedly reselected. The full blocking modal wait and broader live
    user-driven dispatch remain queued.
  - `shellapi.h` also anchors `Shell_NotifyIcon` and the `NOTIFYICONDATAW`
    fixed layout, including the 64-WCHAR `szTip` buffer, while
    `WCESHELLFE\OAK\TASKMAN\minserver.cpp` copies `sizeof(NOTIFYICONDATA)`
    before posting it to the taskbar thread, `mintask.cpp` consumes
    `WM_HANDLESHELLNOTIFYICON` with `wParam` as the `NIM_*` operation and
    `lParam` as the copied `PNOTIFYICONDATA`, and `minshell.h` defines that
    private taskbar message as `WM_USER + 0xBAD`. v3 now requires that fixed
    footprint and a readable `szTip[64]` buffer, stores notify icon
    add/modify/delete state in `ShellSystem`, validates owner HWNDs through
    GWE, posts the registered `uCallbackMessage` to `hWnd` with `wParam=uID`
    and `lParam` carrying the shell event, tracks the registered taskbar HWND,
    posts successful `Shell_NotifyIcon` operations to that taskbar with a
    heap-backed copied `NOTIFYICONDATAW` payload, releases the copied payload
    after `DispatchMessageW` handles `WM_HANDLESHELLNOTIFYICON`, and still
    reports success after mutating shell state when the registered taskbar
    HWND has gone stale and no private taskbar post can be queued, matching
    the sample callback's state-copy-before-taskbar-processing shape. The
    dispatch release guard now also checks the stored private payload type, so
    a spoofed taskbar private message cannot free an unrelated window-pos or
    shell-notification allocation.
    `HPC\EXPLORER\INC\taskbar.hxx`, `TASKBAR\taskbar.cpp`, and
    `TASKBAR\taskbarnotification.cpp` define `HHTBF_DESTROYICON`,
    `NotifyTagDestroyIcon`, and notify-item update/delete paths that destroy
    owned taskbar icons when replaced or removed; v3 records those would-destroy
    `HICON` handles on replacement, explicit delete, and owner-window cleanup.
    CE `SHNotificationRemoveII` removes iconic bubbles through
    `NIM_BUBBLE_DELETE`, which reaches `DeleteItem(..., TRUE)`; v3 records
    copied iconic notification icon destruction on explicit remove and sink
    window/process cleanup as well as timeout expiration.
    Stored `SHNotificationAddI`/`SHNotificationUpdateI` records also validate
    nonzero sink HWNDs against live GWE windows, so stale notification sinks
    fail before mutating shell notification state. CE `notification.cpp`
    rejects zero or unknown `SHNUM_*` update masks in `UpdateBubble`, only
    replaces an icon when `SHNUM_ICON` carries a non-null `hicon`, and destroys
    the old bubble icon before copying the replacement; v3 mirrors those raw
    update edges and records the replaced owned icon for destruction.
    Rich `SHNotification*` guest-COM invocation and taskbar rendering behavior
    remain queued instead of inventing shell UI.
  - The generated COREDLL ordinal table remains behavior data from these CE
    ordinal sources. v3 now caches ordinal-to-export lookup in the same
    precedence order as the old scan (`COREDLL_EXPORTS`, SDK-only exports, then
    supplemental CE-compatible entries); this is a host-side lookup
    optimization for import diagnostics/dynamic proc traps, not a behavior
    change to guest import resolution.

- File mapping and process IPC:
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\MAPFILE\mapfile.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\SHELL\MAPFILE\mapfile.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\schedule.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\handle.c`,
  CE SDK `winbase.h`, and v2 process/mapping fixtures as corroborating
  evidence only.
  - CE file mappings are objects with per-view lifetime. v3 now records
    explicit `FileMappingView` entries instead of treating a mapping as one
    reusable address. `MapViewOfFile` allocates a view, `FlushViewOfFile`
    updates shared backing and sibling views after flush, and
    `UnmapViewOfFile` removes/releases only that view. The remaining fidelity
    gap is full page aliasing/immediate cross-view coherence, richer access
    protection validation, and growing this into a dedicated `MappingSystem`
    as process IPC fixtures demand it.
  - CE process and thread handles are waitable kernel objects whose exit
    transitions signal waiters through the scheduler/handle model. v3 now
    records mounted process launch traces and signals process/thread handles
    when a child run completes. Rooted CE application paths are resolved
    through the mounted CE filesystem namespace before parent-directory module
    fallback, matching the existing FSDMGR-style namespace boundary rather
    than treating rooted names as host-relative strings.

- Scheduler and wait ownership:
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\schedule.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\syncobj.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\thread.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\handle.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\kcalls.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\INC\schedule.h`,
  `C:\WINCE600\PRIVATE\WINCEOS\DRIVERS\SERDEV\serial.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\DEVICE\DEVCORE\devfile.c`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\kfuncs.h`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winbase.h`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\pkfuncs.h`
  - CE waits are kernel scheduler decisions over signaled kernel objects, not
    local ad hoc return stubs at each API boundary.
  - `PulseEvent` semantics are a wait-time release over waiters that are
    currently registered on the event; v3 now records pulse-selected waiter
    ids so a resumed waiter can still receive the correct `WAIT_OBJECT_0`
    result after the event has been reset.
  - Bounded `Sleep` and finite waits are blocking scheduler states in CE, not
    unthrottled host busy loops. v3 still accelerates guest virtual time for
    current-thread bounded waits when no peer can run, but the host side now
    parks briefly so repeated guest timeout polling does not dominate startup
    wall time.
  - A running CE thread is not simultaneously parked in an older wait. The
    Unicorn bridge therefore removes stale saved wait contexts for a thread
    before registering that same thread's next `Sleep` or
    `WaitForSingleObject` wait; this cleans emulator bookkeeping without
    changing the kernel object's signaled state.
  - Event, mutex, semaphore, thread, process, timer, message, device, and audio
    wait paths should converge through one scheduler-owned wait/wake model.
  - The Rust `Scheduler` subsystem now owns compact wait accounting for
    `WaitForSingleObject`, `WaitForMultipleObjects`,
    `MsgWaitForMultipleObjectsEx`, and Unicorn blocked-wait resume diagnostics.
    Parked Unicorn `WaitForSingleObject` waits now preserve start tick/timeout
    metadata and use CE-style `WAIT_TIMEOUT` resume for bounded waits, with
    object-signaled acquisition taking precedence. `DoWaitForObjects` records
    each proxy with the waiting thread's current priority and `WAIT_OBJECT_0 +
    index`; CE priority comparisons treat lower numeric values as higher
    priority, so v3 blocked-wait selection now uses lower numeric priority
    first and FIFO ordering within equal priority. `LockWaitableObject`
    accepts process/thread handles (including current pseudo-handle mapping),
    event, mutex, and semaphore, and rejects other handle types as invalid
    waits; v3 now mirrors that for normal file/device/window/wave/mapping/
    find-file/critical-section handles. `NKWaitForMultipleObjects` rejects
    `fWaitAll` before dispatching to the lower wait helper, so v3 keeps that
    raw API behavior even though v2 had corroborating wait-all machinery.
    `NKWaitForMultipleObjects` also rejects zero handles and counts above
    `MAXIMUM_WAIT_OBJECTS` (`64` from `winnt.h`) before `DoWaitForObjects`;
    handle locking happens for every input handle before any object is waited,
    so a later invalid handle fails the call without consuming an earlier
    signaled auto-reset object. The Unicorn import bridge now preserves that
    ordering for finite current-thread waits: only after every handle is
    readable, waitable, and not signaled does it complete a valid wait-any
    `WaitForMultipleObjects` timeout on the active context with
    `WAIT_TIMEOUT`; `wait_all`, polling waits, invalid handles, and ready
    objects still fall through to the raw kernel path. `thread.c` defines
    `W32PrioMap` as CE
    priorities `248..255`, maps Win32 `SetThreadPriority` values `0..7`
    through that table, maps `GetThreadPriority` back to `0..7` while
    clamping real-time absolute priorities to `THREAD_PRIORITY_TIME_CRITICAL`,
    and exposes `CeGet/CeSetThreadPriority` as absolute `0..255` priority APIs.
    `schedule.h` defines `MAX_SUSPEND_COUNT` as `127`; `kcalls.c`
    `ThreadSuspend` returns the previous count, increments only below that cap,
    and returns `0xffffffff` for refused overflow, which `thread.c`
    `THRDSuspend` maps to `ERROR_SIGNAL_REFUSED`. `ThreadResume` returns the
    previous suspend or pending-suspend count and decrements only when nonzero.
    The first Unicorn multiple-wait parking bridge mirrors the same
    `DoWaitForObjects` proxy shape at the emulator boundary: one parked record
    owns all input handles, readiness scans every handle, and resume returns
    `WAIT_OBJECT_0 + idx` for the first ready handle. The first Unicorn
    `MsgWaitForMultipleObjectsEx` parking bridge uses the same handle-wait
    shape, with GWE queue input acting as the extra `WAIT_OBJECT_0 + count`
    wake slot; source anchors are
    `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\cmsgque.h`
    `MsgWaitForMultipleObjectsEx_E`/`_IWrapper`/`_I` and CE SDK
    `winuser.h` `MWMO_INPUTAVAILABLE`/queue-status flags.
    `kfuncs.h` defines `SYS_HANDLE_BASE == 64`, `SH_CURTHREAD == 1`, and
    `SH_CURPROC == 2`; `GetCurrentThread()`/`GetCurrentProcess()` return the
    resulting pseudo handles, while `GetCurrentThreadId()`/
    `GetCurrentProcessId()` read the KData system-handle slots. `handle.c`
    `LockHandleParam` converts those pseudo handles to the active process or
    current thread for APIs that lock handles, `schedule.c` `LockWaitableObject`
    accepts the current-process pseudo handle by waiting the process event and
    accepts current-thread waits, and `thread.c` `THRDGetCode`/`THRDGetTimes`
    map pseudo current handles where CE exposes thread/process status.
    `thread.c` `THRDSuspend`, `THRDResume`, `THRDSetPrio`, and
    `THRDSetPrio256` receive locked thread pointers after that pseudo-handle
    mapping, so raw current-thread priority and suspend/resume paths should
    update the active thread rather than rejecting the pseudo handle. CE mutex
    recursion follows `syncobj.c`/`kcalls.c`: `InitMutex` starts with
    `LockCount=1` when the creating thread requests initial ownership,
    recursive waits by the owner increment `LockCount` up to
    `MUTEX_MAXLOCKCNT == 0x7fff` from `syncobj.h`, and `MUTXRelease` returns
    `ERROR_NOT_OWNER` (`288` from `winerror.h`) when the current thread is not
    the owner. Releases decrement the recursive count until the final release
    clears ownership and wakes waiters. The first v3 scheduler registry now
    mirrors the CE proxy ownership shape by registering parked waits under a
    scheduler wait id plus each waited handle; live Unicorn resume selection
    uses that scheduler-owned wait metadata for CE priority/FIFO ordering while
    the saved CPU context remains in the Unicorn bridge for the next context
    switch slice.
    Object transitions now take the next CE-shaped step: successful
    `SetEvent`, `ReleaseSemaphore`, and final recursive `ReleaseMutex`
    enqueue the scheduler wait ids registered under that handle as pending wake
    candidates. Resume selection prefers those object-transition candidates,
    then rechecks and consumes the real object state through the existing wait
    path so auto-reset events, semaphore counts, and mutex ownership are not
    consumed by the bookkeeping itself. Full run-queue ownership, wait-all,
    timer/serial/audio wake integration, and scheduler-owned saved CPU
    contexts remain the next scheduler port steps. Thread and process handle
    signaling now follows the same waitable-object transition path:
    `mark_guest_thread_exited`, child-process completion, child initial-thread
    completion, and `TerminateProcess` enqueue scheduler wait ids registered
    under the corresponding real thread/process handles or the CE
    current-process pseudo handle after those handles are marked signaled.
    Wait acquisition and exit-code visibility remain owned by the existing
    thread/process handle state. The first GWE message-wait wake path is
    anchored to `GWE\INC\cmsgque.h` `MsgQueue::MsgWaitForMultipleObjectsEx_I`,
    `InputQueuePostMessage`, `PostMessageW_I`, `PostThreadMessageW`,
    `GetMessageW_I`, and the timer queue entries. v3 now records parked
    `MsgWaitForMultipleObjectsEx` waits in a per-thread scheduler
    message-wait queue; message, quit, sent-message, remote-input, and
    `WM_TIMER` posting transitions enqueue those wait ids as pending
    message-input candidates, then live resume rechecks the actual GWE queue
    status and wake mask before returning. The same GWE header declares
    `TimerEntry_t` with an owning `MsgQueue *m_pmsgqOwner` plus optional
    `HWND`, so a `SetTimer(NULL, id, ...)` expiration is a thread/message-queue
    timer rather than a window timer that can be ignored. v3 now stores the
    owner thread id on timers, posts due no-HWND `WM_TIMER` messages to that
    owner queue, and keeps `HWND` timers routed through their window owner when
    a window is known. The same header's `TimerQueuesRemoveSingleEvent(HWND,
    idTimer, MsgQueue*)` removal shape means numeric ids are not global; v3 now
    keys timers by owner thread/message queue, optional `HWND`, and id, so
    `KillTimer(hwnd,id)` removes only the matching window/thread timer and
    duplicate numeric ids can coexist across owners. The same timer entry API
    exposes `TimerQueuesRemoveAllMsgQueueOrHwnd` and
    `TimerQueueWindowDestroyedNotification`; v3 now removes timers for a
    destroyed HWND subtree while leaving no-HWND thread timers owned by the
    message queue. `TimerEntry_t` also carries `TIMERPROC m_tmprc` and
    `bool m_Callback`, and the queue API exposes `TestAndReset(...,
    TIMERPROC *pTimerProc, ...)`; v3 now preserves the guest-visible
    `SetTimer` callback pointer in `MSG.lParam` and the Unicorn
    `DispatchMessageW` bridge enters that callback with `(hwnd, WM_TIMER,
    timer-id, tick-count)`. The CE-internal `m_Callback` path whose timer
    entries bypass the normal message queue remains a future fidelity slice if
    trace evidence reaches it. CE virtual time is advanced inside the emulator timer
    system for sleeps/timer pumping instead of sleeping the host thread. The
    raw Unicorn `GetMessageW` bridge now keeps that timer pumping narrow: it
    only fast-forwards short, imminent timers up to 100 ms before queue
    retrieval, and lets longer future timers park as blocked message waits.
    This keeps CE's blocking `GetMessageW` shape for long periodic timers while
    preserving the current emulator bridge needed for near-term GUI-settling
    timers. Mounted evidence in
    `target\timer_cap_startup_tap_virtual_20s_*` shows the id-1000 7.5 s
    no-HWND timer now remains recorded as the next due timer instead of
    driving thousands of synthetic idle-update dispatches, while the real
    first-present framebuffer remains populated. The initial guest thread uses
    the CE current-thread pseudo-handle as its wait identity and the parked
    `GetMessageW` wait is registered in the scheduler; v3 still needs the
    fuller run-queue/real-time wake slice that resumes such long future timers
    after their real CE due time instead of either spinning or treating the
    parked state as completed UI progress.
    The follow-up `target\unicorn_realtime_timer_virtual_30s_*` probe moved
    that maturation into the live Unicorn import hook instead of the outer
    runner: if a long timer can become due inside the existing host wall-clock
    budget, v3 lets real time pass and then resumes the scheduler-owned
    `GetMessageW` wait before tearing down the Unicorn instance. That preserves
    the saved MIPS context and matches the CE source shape where the blocked
    message wait remains owned by the queue/scheduler until a timer/input wake
    occurs. The outer `run_cpu_loop` still is not a persistent CPU-state
    scheduler; fuller saved-thread context ownership remains a future port.
    The same GWE header declares
    blocking `GetMessageW_I` separately from
    `GetMessageWNoWait_I`, and documents paint requests as queue conditions
    observed by later `GetMessage` calls. v3 now registers parked Unicorn
    `GetMessageW` calls in the scheduler message-wait queue with the original
    HWND/min/max filters; message-input transitions enqueue them as candidates,
    and resume rechecks the actual filtered GWE message state before restoring
    the guest CPU context. `schedule.c` `NKSleep` calls `ThreadSleep`, adds one
    millisecond for bounded sleeps below `0xfffffffe`, handles very long waits
    in `MAX_TIMEOUT` chunks, maps `Sleep(INFINITE)` to thread suspend, maps
    `Sleep(0)` to `ThreadYield`, and implements `NKSleepTillTick` as
    `ThreadSleep(1)`. v3 now has the first worker-thread Unicorn bridge for
    bounded `Sleep(ms)` and `SleepTillTick`: bounded `Sleep` uses the CE
    `ms + 1` rule below `0xfffffffe`, `SleepTillTick` uses a one-tick timeout,
    the bridge registers a timeout-only scheduler wait, switches away from the
    sleeping worker when another saved context is available, and resumes the
    worker with a zero return after timeout expiry. `Sleep(0)` now follows
    the same `NKSleep` branch by recording a scheduler yield and swapping
    between the active guest context and the currently saved peer context when
    the Unicorn bridge has one available. `Sleep(INFINITE)` now follows the
    CE suspend branch far enough for v3 worker contexts: raw dispatch updates
    the current-thread suspend count, and Unicorn saves a worker CPU context
    until `ResumeThread` decrements the count from `1` to `0`, matching
    `kcalls.c::ThreadResume` making a blocked thread runnable. Full
    scheduler-owned run queues, pending PSL late-suspend, main-thread suspend
    blocking, and long-sleep chunking remain open. The first serial-read
    bridge is anchored to `SERDEV\serial.c`, CE comm `ReadFile` behavior through
    `DEVCORE\devfile.c`, and the same scheduler wait ownership rule: empty
    serial `ReadFile` calls can register a scheduler `SerialRead` wait by COM
    handle, remote serial injection queues candidate wait ids, and resumed
    reads complete by streaming device bytes into the original guest buffer.
    v3 now has the first host `win32_com` bridge for configured serial devices:
    it opens the named Windows COM port, applies nonblocking read timeouts,
    configures the host line state from CE DCB data, polls host RX before CE
    reads/parks, and forwards guest TX bytes to the host handle. Full
    COMMTIMEOUTS semantics, overlapped/event-backed waits, modem/error masks,
    richer host failure counters, and complete run-queue ownership remain open.
    The current Unicorn time-slice only alternates the active guest context
    with the single suspended peer context, so it is a temporary scheduler
    bridge toward that source-backed CE run-queue model rather than a
    replacement for it.

- Guest WNDPROC callout and MIPS kdata boundaries:
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\INC\mipskpg.h`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\MIPS\vmmips.c`, and
  `C:\Program Files (x86)\Microsoft Visual Studio 8\VC\ce\atlmfc\src\mfc\wincore.cpp`
  - CE user-kdata is data, not executable guest code. A return into
    `0x000052e8(user-kdata+0x2e8)` after a guest WNDPROC callout is therefore
    an emulator callout/return-context bug, not a valid guest target.
  - MFC expects WNDPROC dispatch to run in guest context and return through the
    normal caller stack. v3 now reserves a small WNDPROC call frame, restores
    the stack pointer on the WNDPROC return stub, and defers scheduler
    blocked-wait/get-message resumes until the WNDPROC return is complete so
    callee state is not interleaved with a different runnable thread.

- Registry API boundary:
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/NK/KERNEL/fscall.c`
  - `NKRegQueryValueExW` forwards to `FILESYS #25 - RegQueryValueExW`.
  - `NKRegSetValueExW` forwards to `FILESYS #26 - RegSetValueExW`.
  - `NKRegOpenKeyExW` forwards to `FILESYS #23 - RegOpenKeyExW`.
  - `NKRegCloseKey` uses `FSYSAPI_REGCLOSE`.

- Kernel-mode import signatures:
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/INC/kmodeentries.hpp`
  - Defines `NKRegOpenKeyExW_t` and `NKRegQueryValueExW_t` signatures.
  - Defines `CreateFileW_t`, `ReadFile_t`, `DeviceIoControl_t`, and
    `CloseHandle_t` signatures used by the virtual file/device facade.

- Device manager file API:
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\DEVICE\DEVCORE\devfile.c`
  and `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/DEVICE/DEVCORE/devfile.c`
  - `DM_DevReadFile`, `DM_DevWriteFile`, and `DM_DevDeviceIoControl` show the
    device-file split beneath Win32 file handles.
  - v3 keeps serial input beneath device-file handles: remote serial bytes are
    drained into the matching COM session before `ReadFile`/`ReadFileInto`, and
    scheduler wake candidates are keyed by the serial device handle rather than
    by an app-specific path.

- COREDLL file cursor/size helpers:
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/CORE/DLL/apis.c`
  and
  `/mnt/c/Program Files (x86)/Windows CE Tools/wce420/STANDARDSDK_420/Include/Mipsii/winbase.h`
  - CE copy-file paths use `GetFileSize`, `SetFilePointer(FILE_BEGIN)`,
    `ReadFile`, `WriteFile`, and `FlushFileBuffers`-style handle behavior.
  - SDK signatures define the low/high file-pointer and high-size output
    pointer shapes mirrored by the raw dispatcher.
  - Mounted iNavi evidence shows large map/SearchDB files are often opened
    read/write but used read-heavy. v3 must keep those existing files
    host-backed and readable without bulk preload; when the host denies
    write-through access, a read-only live host handle is preferable to
    failing `CreateFileW` or reverting to memory backing.

- FSDMGR path canonicalization and volume resolution:
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\CORE\INC\cnnclpth.h`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\CORE\DLL\apis.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\pathapi.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\mounttable.cpp`, and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\loader.c`
  - CE canonicalization treats paths without a leading slash as implicitly
    absolute, not current-directory relative.
  - FSDMGR `InternalCreateFileW` canonicalizes before resolving the owning
    volume through the mount table.
  - Loader same-directory probing is limited to executable/module loading and
    should not be copied into ordinary `CreateFileW` path translation.

- Kernel sync/wait:
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/NK/KERNEL/syncobj.c`
  and
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/NK/KERNEL/schedule.c`
  - Event/mutex objects have handle-close hooks and are waited through
    `NKWaitForSingleObject`.
  - `EVNTModify` calls `ForceEventModify`, accepts `EVENT_PULSE`,
    `EVENT_RESET`, and `EVENT_SET`, and sets last error on invalid event
    operations.

- COREDLL critical sections:
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/CORE/DLL/cscode.c`
  and
  `/mnt/c/Program Files (x86)/Windows CE Tools/wce420/STANDARDSDK_420/Include/Mipsii/winbase.h`
  - The MIPS CE `CRITICAL_SECTION` layout is `LockCount`, `OwnerThread`,
    `hCrit`, `needtrap`, and `dwContentions`.
  - `InitializeCriticalSection`, `EnterCriticalSection`,
    `TryEnterCriticalSection`, `LeaveCriticalSection`, and
    `DeleteCriticalSection` update those fields before kernel trap handling.

- COREDLL TLS and interlocked exports:
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/CORE/DLL/apis.c`
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\kmisc.c`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\kfuncs.h`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\pkfuncs.h`, and
  `/mnt/c/Program Files (x86)/Windows CE Tools/wce420/STANDARDSDK_420/Include/Mipsii/winbase.h`
  - `TlsGetValue` and `TlsSetValue` use `TLS_MINIMUM_AVAILABLE` and set
    `ERROR_INVALID_PARAMETER` for invalid slots; `TlsGetValue` sets
    `NO_ERROR` when a valid slot contains zero.
  - `TlsCall(type, slot)` routes `TLS_FUNCALLOC` and `TLS_FUNCFREE` through the
    kernel TLS allocator/free path. CE reserves slots `0..3`, allocates slots
    `4..63`, returns `TLS_OUT_OF_INDEXES` when exhausted, and clears freed TLS
    slots across process threads.
  - MIPS CE headers define the exported interlocked signatures and
    `InterlockedTestExchange`/`InterlockedCompareExchange` argument order.

- COREDLL heap/local/virtual memory:
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/CORE/LMEM/heap.c`,
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/CORE/LMEM/heap.h`,
  and
  `/mnt/c/Program Files (x86)/Windows CE Tools/wce420/STANDARDSDK_420/Include/Mipsii/winbase.h`
  - `SanitizeSize` maps zero-byte heap requests to one byte and rejects
    requests above `HEAP_MAX_ALLOC`.
  - `LocalAlloc` routes through the process heap and maps `LMEM_ZEROINIT` to
    `HEAP_ZERO_MEMORY`; `LocalFree` returns `NULL` on success and the original
    handle on failure.
  - `HeapAlloc`, `HeapReAlloc`, `HeapFree`, `HeapSize`, `HeapValidate`, and
    `GetProcessHeap` define the flag validation and return shapes mirrored by
    the virtual heap model.
  - `VirtualAlloc`/`VirtualFree` signatures and `MEM_*` flag shapes come from
    the CE MIPS SDK headers; the Rust model keeps page-granular virtual ranges
    until Unicorn mapping is connected.

- GWE message queue surface:
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/GWE/INC/cmsgque.h`
  - Declares `GetMessageW_I`, `GetMessageWNoWait_I`, `PeekMessageW_I`,
    `PostMessageW_I`, and `SendMessageW_*` queue entry points.
  - Raw dispatcher support now writes and reads CE-style `MSG` memory records
    for `GetMessageW`, `PeekMessageW`, and `DispatchMessageW`.
  - Paint requests are a separate queue/list from posted messages: `WM_PAINT`
    retrieval does not remove the paint request; processing paint cancels the
    request. Rust models this as synthetic paint from `update_pending`, not as
    an ordinary posted message.
  - Generated timers are queue work, but v3 now mirrors the observed CE/MFC
    pump requirement that existing sent/posted/quit/paint work is checked
    before raw `GetMessageW`/`PeekMessageW` generates additional due
    `WM_TIMER` entries. Timed-out sent-message transactions still expire
    before retrieval so `SendMessageTimeout` behavior remains visible.

- GWE window surface:
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/GWE/INC/window.hpp`
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\INC\gweapiset1.hpp`, and
  `/mnt/c/Program Files (x86)/Windows CE Tools/wce420/STANDARDSDK_420/Include/Mipsii/winuser.h`
  - Declares `SetWindowTextW_I`, `GetWindowTextW_I`, `SetWindowLongW_I`,
    `GetWindowLongW_I`, `DefWindowProcW_I`, and `DestroyWindow_I`.
  - `CWindow` tracks `fSentWmDestroy`; Rust virtual windows now carry the same
    lifecycle bit, and raw/kernel `DestroyWindow` sends `WM_DESTROY` before
    final HWND cleanup instead of deleting state directly. The current default
    `WM_CLOSE` shortcut records that same destroy-message observation before
    cleanup.
  - `CWindow` also tracks `fSentWmCreate`, and CE SDK/MFC code treats a
    `WM_CREATE` return of `-1` as create failure. Rust's Unicorn
    `CreateWindowExW` guest-WNDPROC callout now preserves that return contract
    by returning `NULL` and removing the just-created virtual HWND when guest
    `WM_CREATE` fails. CE MFC `_WIN32_WCE` sources (`wincore.cpp`,
    `dlgcore.cpp`) contain special first-message/dialog create glue, and a
    mounted iNavi probe showed that unconditional `WM_NCCREATE` injection at
    the `CreateWindowExW` import boundary regresses startup, so v3 does not
    synthesize `WM_NCCREATE` there by default.
  - Raw/kernel parent `DestroyWindow` now walks the virtual descendant tree and
    sends `WM_DESTROY` to descendants before the parent, then performs final
    GWE cleanup. Unicorn direct guest-WNDPROC destroy callouts use the same
    descendant-before-parent target order and delay final root cleanup until
    the last guest `WM_DESTROY` callback returns. A virtual lifecycle order
    counter exists only to verify this child-first sequence in focused
    fixtures.
  - CE SDK `winuser.h` defines `WM_NCHITTEST`, the `HT*` mouse-position return
    codes, `WM_SYSCOMMAND`, and `SC_CLOSE`. Rust raw and Unicorn-default
    `DefWindowProcW` now return CE hit-test codes from the stored window/client
    rectangles and route `WM_SYSCOMMAND/SC_CLOSE` through the default
    `WM_CLOSE` destroy path.
  - CE `window.hpp` exposes `fBeingDestroyed` alongside the sent-destroy
    lifecycle bits; Rust virtual windows now keep the HWND valid while a
    `DestroyWindow` subtree is inside guest `WM_DESTROY` callouts and only mark
    it final-dead after the callout chain completes. This lets reentrant
    `DestroyWindow` calls observe the in-flight lifecycle without deleting the
    same subtree twice.
  - Declares `UpdateWindow_I`; CE/MFC uses this as a synchronous paint forcing
    boundary. Rust raw `UpdateWindow` now validates pending update state by
    sending `WM_PAINT` through the window send path when an update region exists.
  - `window.hpp` declares `GetUpdateRect_I`, `GetUpdateRgn_I`, and
    `BeginPaint_I`, while `CWindow` stores `fHasUpdateRegion` and
    `fEraseBkgnd`. CE SDK `winuser.h` exposes the `bErase` arguments, and MFC
    paint code expects the OS paint/update path to manage `WM_ERASEBKGND`
    before `BeginPaint` reports `PAINTSTRUCT.fErase`. Rust raw
    `GetUpdateRect`/`GetUpdateRgn` now preserve the pending update bounds but
    synchronously send `WM_ERASEBKGND` with the HWND paint HDC and clear only
    the pending erase bit when `bErase` is true.
  - `CWindow` stores `m_rc` for the whole window and `m_rcClient` for the
    client area in screen coordinates; it declares `SetWindowPos_I`,
    `MoveWindow_I`, `GetWindowRect_I`, `GetClientRect_I`,
    `ClientToScreen_I`, and `ScreenToClient_I`.
  - CE SDK `winuser.h` defines `WINDOWPOS` as
    `HWND hwnd`, `HWND hwndInsertAfter`, `int x/y/cx/cy`, and `UINT flags`,
    and says `WM_WINDOWPOSCHANGED` points at that struct through `lParam`.
    GWE `cmsgque.h` classifies `WM_WINDOWPOSCHANGED` as an `IsLParamPtr`
    message, and CE MFC `wincore.cpp` dispatches the message-map handler by
    casting `lParam` directly to `WINDOWPOS*`. Rust now allocates a stable
    guest heap payload for queued `WM_WINDOWPOSCHANGED`, writes the CE struct
    during raw `GetMessageW`/`PeekMessageW` marshalling, and releases the
    registered payload when `DispatchMessageW`/guest WNDPROC return consumes
    it. CE MFC compiles out `WM_WINDOWPOSCHANGING` under `_WIN32_WCE`, so this
    slice intentionally covers the changed notification only.
  - CE `WINDOWPOS` carries show/hide and z-order metadata in `flags` even when
    the rectangle is unchanged. Rust raw/kernel `SetWindowPos` now keeps
    `WM_WINDOWPOSCHANGED` plus the pointer payload for show-only, hide-only,
    and z-order-only changes, while still queuing `WM_MOVE`/`WM_SIZE` only for
    actual geometry deltas. Mounted evidence
    `target\setwindowpos_showhide_virtual_150s_*` confirms the additional
    message traffic without adding app-specific painting or state forcing.
  - Monitor message snapshots now decode queued `WM_WINDOWPOSCHANGED`
    `lParam` pointers using the CE SDK `WINDOWPOS` layout. This is trace-only
    evidence plumbing: it does not alter GWE dispatch, but it lets mounted
    probes report `hwnd`, `hwndInsertAfter`, `x/y/cx/cy`, and `flags` beside
    opaque `MSG.lParam` values.
  - `window.hpp` and `gweapiset1.hpp` expose `BringWindowToTop_I`, and CE SDK
    `winuser.h` exposes `BringWindowToTop(HWND)` beside `GetWindow` and the
    `HWND_TOP`/`HWND_BOTTOM`/`HWND_TOPMOST` constants. Rust raw ordinal 275 now
    routes through the kernel window lifecycle boundary, reuses the virtual
    `SetWindowPos(HWND_TOP, SWP_NOMOVE|SWP_NOSIZE)` z-order path, and activates
    the top-level target.
  - Declares `GetWindowThreadProcessId_I`; Rust raw ordinal 292 now reports
    the HWND owner thread and optional process ID from the virtual GWE window
    table instead of returning a generic stub value.
  - Declares `IsChild_I`; Rust raw ordinal 277 now uses recursive parent-chain
    checks so direct children and descendants are reported through the virtual
    HWND tree.
  - Also declares `ShowWindow_I`, `UpdateWindow_I`, `GetParent_I`,
    `IsWindow_I`, `GetClassNameW_I`, and `EnableWindow_I`, which back the
    virtual HWND state, class/title text copying, visibility/enabled checks,
    parent lookup, and focus bookkeeping.
  - CE SDK `winuser.h` distinguishes a window's direct `WS_VISIBLE` style bit
    from effective ancestor visibility. Raw/kernel `ShowWindow` now uses the
    direct HWND visible state for `WM_SHOWWINDOW` and queues a changed
    `WINDOWPOS` payload for direct show/hide transitions even when
    `IsWindowVisible` remains false because an ancestor is hidden. Mounted
    evidence `target\showwindow_direct_visibility_virtual_150s_*` confirms the
    real app receives hide payloads with `SWP_HIDEWINDOW`, no-move/no-size,
    no-zorder, and no-activate flags without forcing any paint.
  - `window.hpp` declares `SetParent_I(HWND hwnd, HWND hwndParent)` and
    `GetParent_I(HWND hwnd)`, and CE SDK `winuser.h` exposes `SetParent`/
    `GetParent` beside `GW_CHILD` traversal and `WS_CHILD` style checks used
    heavily by CE MFC docking/control code. Rust raw `SetParent` now enters the
    kernel window lifecycle boundary, preserves the previous-parent return,
    rejects invalid handles and descendant-parent cycles, relinks the HWND into
    the new parent's sibling z-order, and reconciles descendant focus/explicit
    activation when the new ancestry is effectively hidden or disabled.
  - CE SDK `winuser.h` defines `WS_CHILD` and `GW_OWNER`; CE MFC
    `wincore.cpp::AfxGetParentOwner` explicitly uses
    `GetParent(hwnd)` for `WS_CHILD` windows and `GetWindow(hwnd, GW_OWNER)`
    for non-child windows. Rust raw `CreateWindowExW` now mirrors that split:
    `hWndParent` becomes the parent only for `WS_CHILD` creates, otherwise it
    becomes the owner while geometry remains screen-relative and
    `GW_OWNER` reports the owner HWND.
  - CE SDK `winuser.h` defines `CREATESTRUCTW.hMenu`,
    `CreateWindowExW(..., HWND hWndParent, HMENU hMenu, ...)`, and
    `DrawMenuBar(HWND)`. CE MFC `_WIN32_WCE`
    `wincore.cpp::PreCreateWindowEx` strips standalone menu handles from
    `CreateWindowExW` before creation because CE does not support that path
    directly there, then `PostCreateWindowEx` reattaches high-word menu
    handles with `SetMenu(hWnd, nIDorHMenu)`. Rust now stores optional HWND
    menu state, treats non-child `CreateWindowExW.hMenu` as attached menu
    state while preserving child `hMenu` as the control id, and exposes
    `SetMenu`/`GetMenu`/`DrawMenuBar` through raw COREDLL without drawing fake
    menu pixels.
  - CE GWE `window.hpp` declares `SetAssociatedMenu_I(HWND, HMENU)` and
    `GetAssociatedMenu_I(HWND)`, and `gweapiset1.hpp` exposes the same entries
    in the GWE API set. Rust raw ordinals `299` and `300` now use the same
    virtual HWND menu association as `SetMenu`/`GetMenu`, preserving live-HWND
    validation while avoiding host menu widgets or fake menu painting.
  - `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winuser.h` defines the CE menu API
    surface and flags used by the next virtual menu slice: `CreateMenu`,
    `CreatePopupMenu`, `AppendMenuW`, `InsertMenuW`, `RemoveMenu`,
    `DeleteMenu`, `GetSubMenu`, `EnableMenuItem`, `SetMenuItemInfoW`,
    `GetMenuItemInfoW`, `MF_BYPOSITION`, `MF_POPUP`, `MF_SEPARATOR`,
    `MF_ENABLED`, `MF_GRAYED`, `MF_CHECKED`, `MIIM_STATE`, `MIIM_ID`,
    `MIIM_SUBMENU`, `MIIM_CHECKMARKS`, `MIIM_TYPE`, `MIIM_DATA`, and the
    44-byte MIPS CE `MENUITEMINFOW` layout. CE MFC
    `C:\Program Files (x86)\Microsoft Visual Studio 8\VC\ce\atlmfc\src\mfc\winfrm.cpp`
    traverses menus with `GetMenu`, `GetSubMenu`, item counts/IDs, and
    `MENUITEMINFO` during frame/menu-bar handling. CE MFC
    `cmdtarg.cpp::CCmdUI::Enable`/`SetCheck` uses `EnableMenuItem` and
    `CheckMenuItem` with `MF_BYPOSITION` to update command UI state. Rust now
    keeps ordered virtual menu items with command IDs, popup submenu handles,
    type/state, checkmark bitmap handles, item data, and wide text through raw
    COREDLL menu ordinals; `EnableMenuItem` and by-position `CheckMenuItem`
    update that state without drawing fake menu UI. v2 corroborated that menu
    handles were a viable emulation path, but v3 keeps the state in CE-like
    Rust menu objects rather than host menu widgets.
  - `window.hpp` declares `IsWindowVisible_I`, and `CWindow::IsVisibleEnabled_I`
    checks `WS_VISIBLE`/`WS_DISABLED` style state. Rust now keeps direct
    visible state synchronized with `WS_VISIBLE` for `ShowWindow`,
    `SetWindowPos(SWP_SHOWWINDOW/SWP_HIDEWINDOW)`, and `SetWindowLong(GWL_STYLE)`;
    raw `IsWindowVisible` and point hit-testing walk ancestors so children of
    hidden parents are effectively invisible without mutating the child HWND.
  - CE SDK `winuser.h` exposes `EnableWindow`/`IsWindowEnabled` and defines
    `WM_ENABLE`; the same header defines `WM_CANCELMODE`. Rust raw
    `EnableWindow` now preserves the CE previous-enabled return shape, routes
    through `CeKernel::enable_window`, and queues `WM_CANCELMODE` before a
    disable transition plus `WM_ENABLE` when the enabled state actually
    changes. Initial `WS_DISABLED` windows and `EnableWindow` transitions now
    keep the virtual style bit and direct enabled bit in sync, while
    `IsWindowEnabled`, dialog traversal, and point hit-testing walk the parent
    chain so descendants of disabled windows are effectively disabled without
    receiving child `WM_ENABLE` notifications. Full synchronous message-send
    timing remains part of the broader GWE send/scheduler port.
  - `cmsgque.h` exposes `SetFocus_I`, `GetFocus_I`,
    `GetKeyboardTarget_I`, `GetActiveWindow_I`, `SetActiveWindow_I`, and
    `SetForegroundWindow_I`; `window.hpp` exposes `ShowWindow_I`,
    `EnableWindow_I`, `IsWindowVisible_I`, and `IsWindowEnabled_I`; CE SDK
    `winuser.h` defines `WM_SETFOCUS`, `WM_KILLFOCUS`, `WM_ACTIVATE`,
    `SWP_HIDEWINDOW`, and `SWP_NOACTIVATE`. CE MFC modal-dialog code disables
    owners around modal execution and restores active focus through normal
    window APIs. Rust now clears focus and explicit activation for a disabled
    or hidden target/descendant through the same message path already used by
    `SetFocus(NULL)` and `SetActiveWindow(NULL)`, rather than silently dropping
    the virtual state.
  - Declares `GetWindow_I(HWND hwndThis, UINT32 relation)`, the GWE API set
    exposes `m_pGetWindow`, and the CE Mipsii SDK defines `GW_HWNDFIRST`,
    `GW_HWNDLAST`, `GW_HWNDNEXT`, `GW_HWNDPREV`, `GW_OWNER`, and `GW_CHILD`
    as `0..5`. Raw ordinal 251 uses those command values over the virtual HWND
    tree for desktop-child, child, sibling, and owner-null traversal.

- GWE class/message surface:
  `/mnt/c/Program Files (x86)/Windows CE Tools/wce420/STANDARDSDK_420/Include/Mipsii/winuser.h`
  and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\cmsgque.h`
  - CE SDK headers define the `WNDCLASSW` memory shape, `HWND_BROADCAST`, and
    `WS_VISIBLE` values used by raw `RegisterClassW`, `GetClassInfoW`,
    `FindWindowW`, `PostMessageW`, and `ShowWindow` marshalling.
  - GWE queue declarations keep `GetMessageW` as the blocking message API;
    an empty queue is not modeled as a `FALSE` return because MFC treats that
    as `WM_QUIT`/thread exit.
  - `gweapiset1.hpp` and `cmsgque.h` also expose `GetMessageWNoWait` with the
    same `MSG*, HWND, min, max` argument shape as `GetMessageW`, but without a
    blocking wait path. Raw ordinal 863 now uses the GWE filtered retrieval
    path for nonblocking posted messages and queue-owned quit state.
  - `cmsgque.h` stores `PostedMsgQueueEntry_t::time`,
    `PostedMsgQueueEntry_t::MousePosAtPost`, and queue
    `m_ReadyTimeStamp`; it also declares `GetMessagePos_I`. CE SDK
    `winuser.h` exposes `GetMessagePos()` and
    `GetMessageQueueReadyTimeStamp(HWND hWnd)`. Rust now preserves screen
    mouse position metadata for mouse messages, records the last retrieved
    message position per receiving thread, and reports queue ready timestamps
    from posted, sent, and quit-state work rather than using the current timer
    tick as a stand-in.
  - `cmsgque.h` stores quit delivery as queue state (`msgqfGotWMQuitMessage`,
    `m_nExitCode`, and `mgefQuitMsg`) rather than as a normal posted-message
    entry. Rust `PostQuitMessage` now records per-thread quit state, and raw
    `GetMessageW`/`PeekMessageW` synthesize `WM_QUIT` from that state even when
    the caller supplies a nonmatching HWND or message filter.
  - CE SDK queue-status constants and `MsgWaitForMultipleObjectsEx` flags
    define the split between current queue input and changed queue input.
    Rust `GetQueueStatus` now reports current bits in the high word and
    changed-since-last-query bits in the low word, clearing only the requested
    changed bits after observation. Raw `MsgWaitForMultipleObjectsEx` consumes
    newly changed queue input by default and treats already-queued input as
    wakeable only when `MWMO_INPUTAVAILABLE` is set. CE `winuser.h` exposes
    `MWMO_INPUTAVAILABLE` as the only `MsgWaitForMultipleObjectsEx` flag in
    this target SDK, while MFC's CE `mtex.cpp` wait-all path loops around
    normal wait-any calls itself. v3 therefore treats desktop bit `0x0001` as
    unsupported/ignored message-wait metadata rather than converting raw
    `MsgWaitForMultipleObjectsEx` into kernel wait-all failure.
  - `cmsgque.h` defines `smfSenderNoWait`,
    `smfSenderNoWaitIfDifferentThread`, and `smfNotifyMessage` for no-wait
    notification sends. Rust raw `SendNotifyMessageW` now preserves that CE
    split at the syscall boundary: same-thread targets use synchronous send,
    while different-thread targets enter the receiver-side sent-message queue
    without sender blocking and carry `SMF_SENDER_NO_WAIT | SMF_NOTIFY_MESSAGE`
    in the queued transaction state. `winuser.h` exposes `HWND_BROADCAST`;
    v3's broadcast notify path now filters to live application top-level
    windows and applies the same direct/sent delivery split instead of using
    the posted-message broadcast helper.
  - GWE now keeps a receiver-side sent-message queue distinct from posted
    messages and paint requests. Retrieval prefers sent messages, marks
    `InSendMessage`, exposes `QS_SENDMESSAGE`, and records a send source.
    Filtered retrieval still applies the caller's HWND/min/max range before a
    receiver-side send is reported or removed, matching the queue filter shape
    in `GetMessageW`/`GetMessageWNoWait`; this keeps unrelated queued notify
    sends and mount `WM_DEVICECHANGE` broadcasts from satisfying modal or shell
    `WM_FILECHANGEINFO` peeks.
    Raw and Unicorn `DispatchMessageW` paths now clear the receiver send
    context after dispatch returns. Raw cross-thread `SendMessageW` now creates
    this sent-message transaction instead of running the receiver/default path
    immediately in the caller thread; raw `DefWindowProcW` remains direct
    default processing. CE SDK `winuser.h` documents `SendDlgItemMessage` as a
    wrapper implemented in terms of `GetDlgItem` and `SendMessage`, so raw
    `SendDlgItemMessageW` now reuses the same queueing helper for normal
    messages after resolving the child HWND. `cmsgque.h`'s `SendMsgEntry_t`
    fields
    (`pReceivedNext`, `pSentNext`, `pmsgqReply`, `smFlags`, HWND/message
    parameters, and `WndProcResult`) now map to explicit Rust sent-message
    transaction state with sender/receiver thread ids, flags, timeout metadata,
    an active receiver send stack, result-ready completion, and
    receiver-terminated completion when a target is destroyed. `cmsgque.h`'s
    `MessageTimeout` comment and `smfTimeout` flag now map to GWE timeout
    expiry: non-result-ready sent transactions compare the current tick against
    the message timestamp plus timeout, set `SMF_TIMEOUT|SMF_RESULT_READY`,
    leave a zero result, and are removed from receiver queues before retrieval.
    Raw
    `SendMessageTimeout(..., timeout=0)` across threads now goes through the
    same sent-transaction path and expires immediately instead of running the
    receiver shortcut. Raw `SendMessageTimeout(..., timeout>0)` across threads
    also creates a timeout-flagged sent transaction and leaves it queued for
    receiver retrieval rather than fabricating a synchronous result in the
    caller thread. CE SDK `winuser.h` exposes only `SMTO_NORMAL` for this
    target, and v3 now covers that public zero-flag path with a nonzero timeout
    send that queues, dispatches on the receiver thread, and writes through
    `lpdwResult`. The CE private Office `winresrc.h` also defines `SMTO_BLOCK`
    and `SMTO_ABORTIFHUNG`; v3 now accepts those two nonzero bits, rejects
    still-unknown `fuFlags` with `ERROR_INVALID_FLAGS` before queueing, and
    preserves the original accepted flag bits on the sent transaction so later
    blocking/reentrancy work can honor the exact caller contract. Same-thread
    raw `SendMessageTimeout` still uses the synchronous send path before the
    cross-thread hung check, so `SMTO_ABORTIFHUNG` does not abort a same-thread
    dispatch even when that thread's last-dispatch timestamp would satisfy the
    CE hung threshold. Cross-thread `SMTO_ABORTIFHUNG` now also covers the CE
    threshold boundary by queueing just below the 5-second hung cutoff and
    aborting without queueing once the receiver is considered hung. The Unicorn
    guest path then parks the sender context on that same transaction when a
    guest WNDPROC callout is possible; zero-timeout cross-thread calls are
    refused by Unicorn block preparation so the raw immediate-expiry path owns
    the CE case and no stale receiver send is queued by a parked waiter. When a
    parked `SendMessageTimeout` reaches its timeout before a receiver result,
    the Unicorn resume path now marks the same sent-message transaction timed
    out and consumes it before returning `ERROR_TIMEOUT`, so the receiver cannot
    dispatch stale work after the sender has resumed. The scheduler
    now has a send-reply blocked-wait kind
    keyed by sent-message id, mirroring the sender-side `pSentNext`/reply wait
    relationship: normal WNDPROC completion, timeout expiry, receiver
    destruction, and early `ReplyMessage`-style receiver release enqueue
    send-reply wake candidates once the `WndProcResult` state is ready.
    `cmsgque.h` documents `smfResultReady` as the reply event for a sent
    message, and v3 now preserves that result if the receiver later unwinds
    from dispatch. The same header's `MessageTimeout::cReference` note says the
    timeout record is kept alive for nested `SendMessageTimeout` calls; raw GWE
    coverage now verifies the equivalent state shape where an active outer send
    can time out, a nested sent message can still dispatch and complete
    independently, and the later outer dispatch unwind preserves the timeout
    result instead of overwriting it. The raw `DestroyWindow` path also flushes
    a receiver-terminated zero result into a pending `SendMessageTimeout` caller's
    `lpdwResult`. Unicorn raw `SendMessageW`/`SendMessageTimeoutW` now uses
    that transaction state for same-process cross-thread guest WNDPROCs: the
    receiver thread becomes the active CE thread for the guest WNDPROC callout,
    the sender MIPS context is parked in a scheduler-backed `SendMessage`
    blocked wait, WNDPROC return and generic scheduler wake/resume restore that
    blocked record after the result is captured, and the result flows back to
    the sender and optional timeout result pointer. Because CE's `SendMsgEntry_t`
    is the authoritative live send transaction, the runtime now clears stale
    cross-process send-yield debug snapshots once the corresponding GWE
    sent-message record has been completed or removed; the remote and monitor
    handoff loops then use current queue/wait state instead of re-routing on an
    orphaned snapshot from an earlier modal/send stop. Reentrant cross-thread
    scheduling, a public raw `ReplyMessage` export if the target import table
    exposes one, and broader nested destroyed-target edge behavior remain open.
  - CE `cmsgque.h` exposes `MsgWaitForMultipleObjectsEx_I` as a message-queue
    wait entrypoint with an owned handle list. Local raw dispatch now probes
    those handles with the no-record multiple-wait helper, preserving CE
    signaled-handle precedence while keeping scheduler telemetry attributed to
    the public message-wait call rather than to an extra internal
    `WaitForMultipleObjects` attempt.
  - CE SDK headers define `CREATESTRUCTW` as
    `lpCreateParams`, `hInstance`, `hMenu`, `hwndParent`, `cy`, `cx`, `y`, `x`,
    `style`, `lpszName`, `lpszClass`, and `dwExStyle`, and define
    `WM_CREATE` as `0x0001`. Unicorn raw `CreateWindowExW` uses that layout
    when delivering the create-time guest WNDPROC callout.
  - The CE SDK `winuser.h` message constants define `WM_MOVE`, `WM_SIZE`,
    `WM_SHOWWINDOW`, and `WM_WINDOWPOSCHANGED`. The virtual GWE kernel boundary
    now queues those lifecycle messages from raw show/move/resize calls so MFC
    frame/layout code can observe the same window-state transitions as the
    launch path advances.
  - The CE SDK `winuser.h` and GWE API set expose `SetFocus`,
    `SetActiveWindow`, `SetForegroundWindow`, `ShowWindow`, `SetWindowPos`,
    `WM_ACTIVATE`, `WM_SETFOCUS`, `WM_KILLFOCUS`, `WA_ACTIVE`,
    `WA_INACTIVE`, and `SWP_NOACTIVATE`. Rust now stores active-window state
    separately from focus and queues the focus/activation messages through the
    kernel lifecycle boundary. MFC `wincore.cpp`/`winfrm.cpp` consume these
    messages in frame/view activation and focus routing, so this remains guest
    WNDPROC-visible behavior rather than host-side shortcutting.
  - `winuser.h`, `gweapiset1.hpp`, and `window.hpp` expose
    `WindowFromPoint` and `ChildWindowFromPoint` with by-value `POINT`
    arguments. Rust raw ordinal 252 (`WindowFromPoint`) routes through the
    normal visible/enabled recursive HWND hit-test used by mouse input. Raw
    ordinal 253 (`ChildWindowFromPoint`) treats the input point as
    parent-client coordinates, converts through the parent client-to-screen
    transform, searches only immediate children in z-order, and returns hidden
    or disabled children. Microsoft CE `ChildWindowFromPoint` documentation
    states that hidden/disabled containing children are successful results and
    that parent is returned when the point is inside the parent but no child
    contains it; modern Win32 documentation corroborates the immediate-child
    search restriction.

- GWE dialog/control surface:
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\dlgmgr.h`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\INC\gweapiset1.hpp`, and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winuser.h`
  - CE exposes dialog manager entry points including
    `CreateDialogIndirectParamW_I`, `EndDialog_I`, `GetDlgItem_I`,
    `GetDlgCtrlID_I`, `SetDlgItemTextW_I`, `GetDlgItemTextW_I`,
    `SetDlgItemInt_I`, `GetDlgItemInt_I`, `CheckRadioButton_I`,
    `SendDlgItemMessageW_I`, `IsDialogMessageW_I`, `GetDialogBaseUnits_I`,
    `MapDialogRect_I`, `GetNextDlgTabItem_I`, and
    `GetNextDlgGroupItem_I`.
  - Rust now routes raw `SetDlgItemInt` and `GetDlgItemInt` through the
    existing dialog child lookup/window text state: integer setters write
    signed or unsigned decimal text to the child control, and integer getters
    parse that text and update the optional success flag. This is dialog
    manager behavior, not app-specific resource shaping.
  - Rust now also routes raw `GetDlgItem`, `SetDlgItemTextW`,
    `GetDlgItemTextW`, and `SendDlgItemMessageW` text messages through the
    same child-control HWND and guest-memory boundary. `SendDlgItemMessageW`
    forwards `WM_SETTEXT`, `WM_GETTEXT`, and `WM_GETTEXTLENGTH` to the child
    using the raw `SendMessageW` text path, while preserving the existing
    `BM_GETCHECK`/`BM_SETCHECK` button-state behavior. CE treats low pointer
    values as resource IDs/atoms, so focused tests use pointer-backed UTF-16
    buffers above `0xffff`.
  - Rust now routes raw `GetDialogBaseUnits`/`MapDialogRect` through CE-style
    dialog-unit conversion and raw `GetNextDlgTabItem`/
    `GetNextDlgGroupItem` through the real child HWND tree. `winuser.h`
    supplies the public `WS_TABSTOP`, `WS_GROUP`, and `WS_DISABLED` styles used
    by the traversal, and disabled ancestor checks now keep controls beneath a
    disabled dialog out of traversal without mutating the child HWND state.
  - `winuser.h` declares `IsDialogMessageW`, `GetKeyState`,
    `GetAsyncKeyState`, `WM_GETDLGCODE`, `VK_TAB`, `VK_SHIFT`, `VK_RETURN`,
    `VK_ESCAPE`, `IDOK`, `IDCANCEL`, `DM_GETDEFID`, `DM_SETDEFID`,
    `DC_HASDEFID`, button styles such as `BS_PUSHBUTTON` and
    `BS_DEFPUSHBUTTON`, and the `DLGC_*` dialog-code flags; `keybd.h` declares
    the CE `KeyState*` and `KeyShift*` flags returned by async keyboard state;
    `dlgmgr.h` exposes `IsDialogMessageW_I`, `CheckDefPushButton`,
    `IsDefPushButton`, and `IsUndefPushButton` paths that consult
    `WM_GETDLGCODE`. Rust raw `IsDialogMessageW` now consumes only messages for
    the dialog or its descendants, dispatches ordinary dialog-owned messages,
    moves focus on TAB/Shift+TAB through the existing dialog tab traversal using
    queued-key `GetKeyState(VK_SHIFT)`, and routes Enter/Escape as dialog
    commands without special-casing iNavi. Return uses a focused pushbutton or
    the dialog's default pushbutton with `IDOK` fallback, while Escape resolves
    an existing `IDCANCEL` pushbutton HWND before falling back to the command id
    alone. GWE now reports `DLGC_DEFPUSHBUTTON`/`DLGC_UNDEFPUSHBUTTON` plus
    `DM_GETDEFID`/`DM_SETDEFID` over child button style state. The same queued
    key state now backs the first raw `GetAsyncKeyState` and
    `GetAsyncShiftFlags` implementation.
  - `winuser.h` and `gweapiset1.hpp` declare the public/internal
    `PostKeybdMessage` shapes, `keybd_event`, `GetMessageSource`, and
    `MSGSRC_HARDWARE_KEYBOARD`; `keybd.h` defines `KeyStateDownFlag` and
    `KeyStatePrevDownFlag`. Rust raw ordinal 832 now queues hardware-sourced
    `WM_KEYDOWN`/`WM_KEYUP` plus optional character-buffer `WM_CHAR` messages
    through the normal GWE thread queue, including the compact and split
    pointer/capacity `PostKeybdMessage` character-buffer shapes, and raw
    `keybd_event` targets the focused/active keyboard window with CE-style
    lParam transition bits.
  - `gweapiset1.hpp` declares `SetKeyboardTarget`/`GetKeyboardTarget`, and
    `cmsgque.h` stores `m_hwndKeyboardTarget` beside `m_hwndFocus` and
    `m_hwndActiveWindow` on the message queue. Rust now stores keyboard
    targets per thread/message queue, exposes raw ordinals 710/711/1225, and
    routes `keybd_event`/targetless `PostKeybdMessage` through the explicit
    keyboard target before focus/active fallback.

- GWE paint/update surface:
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\INC\gweapiset1.hpp`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\LIB\ARMV4I\RETAIL\coredll.def`, and
  `/mnt/c/Program Files (x86)/Windows CE Tools/wce420/STANDARDSDK_420/Include/Mipsii/winuser.h`
  - CE exposes `InvalidateRect`, `BeginPaint`, `EndPaint`, `UpdateWindow`,
    `GetUpdateRgn`, `GetUpdateRect`, `ValidateRect`, and `RedrawWindow` through
    the GWE API set and coredll ordinals `250`, `260`, `261`, `267`, `273`,
    `274`, `278`, and `286`.
  - The SDK `PAINTSTRUCT` layout is `hdc`, `fErase`, `rcPaint`, `fRestore`,
    `fIncUpdate`, and 32 reserved bytes; raw `BeginPaint` writes that shape.
  - `WM_PAINT` is `0x000F`; the virtual GWE subsystem generates it from a
    pending update region and clears the region through `BeginPaint` or
    `ValidateRect`.
  - Raw `RedrawWindow` now follows the same pending-paint model for rectangle
    and HRGN invalidation, update-region unioning, descendant invalidation, and
    `RDW_UPDATENOW`. Raw `ValidateRect` and `RDW_VALIDATE` now subtract
    representable rectangular update bounds. Raw `GetUpdateRgn` copies the
    current pending update bounds into an existing HRGN and returns region
    status. Remaining gaps are tracked in `TODO.md`: complex update-region
    precision, erase-on-query behavior, internal-paint-only requests, and full
    child clipping.
  - Mounted `target\paint_priority_final_virtual_*` evidence now confirms this
    path reaches real `BeginPaint`/`EndPaint`; the remaining display frontier
    is generic GDI/DC presentation from memory-composed DIB surfaces to a
    screen/window HDC, not synthetic paint generation itself.
  - Direct `UpdateWindow` uses effective ancestor visibility before sending
    the synchronous erase/paint path. Focused raw coverage keeps a child under
    a hidden parent dirty until the parent becomes visible again, then verifies
    `UpdateWindow` clears the pending update through the normal paint path.

- Display surface boundary:
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\gpe.h` and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\gxinfo.h`
  - CE display drivers expose a `GPE` device with a primary `GPESurf`, screen
    width/height, stride, pixel format, and video-memory surface hooks.
  - `GXDeviceInfo` exposes a framebuffer pointer, stride, width, height, and
    bits-per-pixel. The Rust `Framebuffer` trait keeps only that generic
    byte-surface boundary; HWND/HDC/GDI behavior remains outside the trait.
  - The Rust `Presenter` and `Desktop` traits are host architecture boundaries
    layered around that generic byte-surface model. They deliberately do not
    define CE/MFC message, class, HWND, HDC, or GDI semantics.

- GDI DIB/color-table surface:
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\wingdi.h` and
  `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\da.cpp`
  - CE exposes `CreateDIBSection`, `GetDIBColorTable`, `SetDIBColorTable`,
    `RGBQUAD`, `BITMAPINFOHEADER`, `BITMAPINFO`, `DIB_RGB_COLORS`, and indexed
    DIB color-table conventions through the public GDI header.
  - The same header defines CE stock-object indexes for `GetStockObject`,
    including `WHITE_BRUSH == 0`, `BLACK_PEN == 7`, `SYSTEM_FONT == 13`, and
    `DEFAULT_PALETTE == 15`. Rust compatible/window DC state now uses those
    stock selections, plus a nondeletable default bitmap slot for memory DCs,
    so `SelectObject` returns restorable previous objects instead of `0` for
    newly created DCs.
  - Rust now stores RGBQUAD color-table entries on bitmap objects selected into
    memory DCs, routes raw `SetDIBColorTable`/`GetDIBColorTable` through guest
    memory, and uses the selected bitmap table when 8 bpp blits write RGB565
    framebuffer pixels. Rust direct-DIB sources now also parse BITMAPINFO-
    supplied RGBQUAD/RGBTRIPLE tables for 1/4/8 bpp `DIB_RGB_COLORS`, so
    indexed `StretchDIBits`/`SetDIBitsToDevice` sources use their embedded
    palette. Remaining indexed-DIB work is broader palette/device-color
    behavior as trace evidence reaches it.
  - CE `draw.cpp::passNull2Draw(EBitBlt)` and
    `draw.cpp::passNull2Draw(EStretchBlt)` expect null/bad destination and
    source DCs to fail with `ERROR_INVALID_HANDLE`. The same cases reject
    `MAKEROP4(PATCOPY, PATINVERT)` passed to `BitBlt`/`StretchBlt` with
    `ERROR_INVALID_PARAMETER`. Raw `BitBlt` and `StretchBlt` now enforce those
    validation edges before rendering.
  - CE `draw.cpp::gnvRop3Array`, `BitBltSuite`, `StretchBltSuite`, and
    `TestAllRops` exercise common source/destination ROP3 operations including
    `DSTINVERT`, `SRCINVERT`, `SRCCOPY`, `SRCPAINT`, `SRCAND`, `SRCERASE`,
    `MERGEPAINT`, `NOTSRCCOPY`, and `NOTSRCERASE` for `BitBlt` and
    `StretchBlt`. Raw blits now render those source/destination operations for
    selected-DIB and framebuffer paths. The same shared path now samples the
    selected brush for CE pattern ROP3 operations `MERGECOPY`, `PATCOPY`,
    `PATINVERT`, and `PATPAINT` across selected-DIB and framebuffer targets,
    and evaluates the ROP3 byte generically for the literal values in
    `gnvRop3Array`; broader ROP4 combinations remain future work.
  - CE `draw.cpp::passNull2Draw(EMaskBlt)` expects `MaskBlt` to fail null/bad
    destination DCs with `ERROR_INVALID_HANDLE`, fail null/bad source DCs with
    `ERROR_INVALID_HANDLE`, reject bad mask handles with `ERROR_INVALID_HANDLE`,
    and reject color masks or negative mask origins with
    `ERROR_INVALID_PARAMETER`. `MaskBltBadMaskWidth` also rejects 1 bpp masks
    whose origin/size cannot cover the requested blit rectangle, while
    `MaskBltTest` uses `MAKEROP4(DSTCOPY, SRCCOPY)` with a 1 bpp mask, and
    `TestAllRops(EMaskBlt)` iterates foreground/background ROP3 bytes with a
    two-pixel mask. Raw `MaskBlt` now implements those validation paths plus
    selected-DIB and framebuffer masked copies for that CE source-backed ROP4
    shape and generic ROP4 foreground/background byte evaluation.
  - CE `core\dll\apis.c::SystemParametersInfoW` routes
    `SPI_GETOEMINFO` and related device-info actions through
    `KernelIoControl(IOCTL_HAL_GET_DEVICE_INFO, ...)` before GWE handles the
    remaining `SystemParametersInfo` surface. Rust keeps explicit emulator SPI
    override values when present and falls back to the imported
    `HKLM\Ident\Name` registry value for `SPI_GETOEMINFO`, which preserves the
    device identity from `registry.reg`.
  - CE `draw.cpp::passNull2Draw(EAlphaBlend)` expects `AlphaBlend` null/bad
    destination and source DCs to fail with `ERROR_INVALID_HANDLE`.
    `AlphaBlendRandomTest` expects CE to reject nonzero `BlendFlags`,
    non-`AC_SRC_OVER` `BlendOp`, unsupported `AlphaFormat`, and
    `AC_SRC_ALPHA` on non-32bpp sources with `ERROR_INVALID_PARAMETER`.
    CE SDK `wingdi.h` exposes both `AC_SRC_ALPHA` and
    `AC_SRC_ALPHA_NONPREMULT`, while GWE `colortable.hpp` documents that GDI
    `AlphaBlend` defaults to premultiplied alpha and uses a separate negative
    flag for non-premultiplied color data.
    `AlphaBlendGoodRectTest` treats zero source or destination dimensions as
    successful no-op rectangles, while `AlphaBlendBadRectTest` expects negative
    dimensions and source rectangles outside the source surface to fail with
    `ERROR_INVALID_PARAMETER`.
    CE GPE `swblt.cpp` handles `BLT_STRETCH` by converting the source and
    destination extents into Bresenham accumulators (`rowXAccum`/`yAccum`) and
    repeating or advancing source pixels from that state; uneven stretches such
    as 2-to-3 or 3-to-5 therefore do not match simple floor-division sampling.
    `AlphaBlendConstAlphaTest`, `AlphaBlendPerPixelAlphaToPrimaryTest`, and
    `AlphaBlendPerPixelAlphaTest(..., TRUE/FALSE)` cover source-constant and
    top-down/bottom-up 32 bpp per-pixel alpha into primary or DIB surfaces.
    Raw `AlphaBlend` now validates those fields before rendering, accepts empty
    rectangles as successful no-ops, rejects negative dimensions and
    out-of-bounds selected-DIB source rectangles, keeps source-constant-alpha
    selected-DIB blending covered, treats `AC_SRC_ALPHA` source RGB as
    premultiplied, accepts `AC_SRC_ALPHA_NONPREMULT` for non-premultiplied
    source RGB, applies top-down and bottom-up 32 bpp per-pixel alpha between
    selected-memory DIBs, and applies source-constant plus top-down and
    bottom-up 32 bpp per-pixel alpha into framebuffer-backed window DCs, and
    clips negative framebuffer destination origins while preserving CE stretch
    source mapping to the visible pixel. Selected-DIB and framebuffer alpha
    stretch paths now use the same CE GPE Bresenham source-pixel
    repetition/advance pattern for uneven stretches.
  - CE GDIAPI device-attribute tests expect `GetBkMode(NULL/bad hdc)` to
    return `0` and `GetBkColor(NULL/bad hdc)` to return `CLR_INVALID`, both
    with `ERROR_INVALID_HANDLE`, so raw `GetBkMode` and `GetBkColor` now report
    those last-error states.
  - The same `da.cpp` `PassNull2da` and `AlphaCheckGetSetColor` paths expect
    `SetBkColor`/`GetBkColor` and `SetTextColor`/`GetTextColor` to return
    `CLR_INVALID` with `ERROR_INVALID_HANDLE` for null and bad HDCs, while
    valid `CLR_INVALID` color values round-trip through background/text color
    state with `ERROR_INVALID_PARAMETER`.
  - The `da.cpp` `PassNull2da`, `randSetStretchBltMode`,
    `RandSetTextCharacterExtraTest`, and `GetSetLayoutModeTest` paths define
    CE device-attribute sentinels: stretch mode APIs return `0` for invalid
    HDCs and invalid modes, text-character-extra APIs return `0x80000000` for
    invalid HDCs and `INT_MIN`, and layout APIs use `GDI_ERROR` only for
    invalid HDCs while successful layout state remains a non-sentinel value.
  - The same `PassNull2da` switch expects exported `SetViewportOrgEx` to return
    `FALSE` and set `ERROR_INVALID_HANDLE` for null and bad HDCs. The local CE
    export map currently exposes only `SetViewportOrgEx` from this origin/ext
    API family, so the raw implementation now validates the HDC before applying
    its existing no-op `(0,0)` viewport-origin behavior.
  - `brush.cpp` `passBrushNULL(ESetBrushOrgEx)` expects `SetBrushOrgEx` to
    return `FALSE` with `ERROR_INVALID_HANDLE` for null and bad HDCs, and
    `SimpleSetBrushOrgExTest` expects valid calls to return the previous brush
    origin through `lppt` while succeeding with a null output pointer.
  - `dc.cpp` `PassNull` and `SaveRestoreDCPairs` expect `SaveDC`/`RestoreDC`
    invalid HDCs to report `ERROR_INVALID_HANDLE`, `RestoreDC(hdc, 0)` to fail
    with `ERROR_INVALID_PARAMETER`, `RestoreDC(hdc, -1)` to restore one saved
    level at a time, and positive save levels to restore that absolute level
    while discarding newer saved states.

- GDI text/font query surface:
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\wingdi.h` and
  `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\text.cpp`
  - Defines CE `LOGFONTW`, `TEXTMETRICW`, `ExtTextOutW`,
    `GetTextExtentExPointW`, `GetTextMetricsW`, `SetTextAlign`,
    `GetTextAlign`, and `GetTextColor`; `GetTextExtentPointW` is a macro over
    `GetTextExtentExPointW`.
  - Rust now copies selected `LOGFONTW` fields into font objects and returns
    deterministic selected-font metrics, extent/fitting arrays, face names,
    text color, and text alignment at the raw COREDLL boundary. CE GDIAPI
    tests expect `GetTextFace(hdc, c, NULL)` to return the selected face-name
    length including the terminating NUL and `GetTextFace(hdc, -1, buffer)` to
    fail with `ERROR_INVALID_PARAMETER`, so raw `GetTextFaceW` now follows
    those parameter edges. The same CE test matrix expects
    `GetTextExtentExPoint` to fail with `ERROR_INVALID_PARAMETER` for a null
    output `SIZE`, null input text with a positive count, and negative counts,
    so raw `GetTextExtentExPointW` now follows those parameter edges.
    `passNull2Text` expects `SetTextAlign` and `GetTextAlign` to return
    `GDI_ERROR` with `ERROR_INVALID_HANDLE` for null and bad HDCs, so the raw
    alignment APIs now validate the HDC before reading or mutating DC state.
    `da.cpp::PassNull2da` expects `SetBkMode` and `GetBkMode` to return `0`
    with `ERROR_INVALID_HANDLE` for null and bad HDCs, so the raw background
    mode APIs now validate the HDC before reading or mutating DC state.
    `text.cpp` manual and color-fill text paths call `SetBkMode(OPAQUE)` or
    `SetBkMode(TRANSPARENT)` before `ExtTextOut`, so raw `ExtTextOutW` now
    applies the selected DC background mode when rendering text cells: opaque
    fills use `bk_color`, while transparent mode leaves non-glyph pixels
    untouched. `passNull2Text` expects `ExtTextOut` null/bad HDCs to fail with
    `ERROR_INVALID_HANDLE`, while null text with a positive count on a valid HDC
    fails with `ERROR_INVALID_PARAMETER`, so raw `ExtTextOutW` validates the HDC
    before checking the text pointer. Broader glyph rasterization remains a
    later GDI text step.

- MFC window layout behavior:
  `/mnt/c/Program Files (x86)/Microsoft Visual Studio 8/VC/ce/atlmfc/src/mfc/wincore.cpp`
  `/mnt/c/Program Files (x86)/Microsoft Visual Studio 8/VC/ce/atlmfc/src/mfc/winfrm.cpp`,
  `/mnt/c/Program Files (x86)/Microsoft Visual Studio 8/VC/ce/atlmfc/src/mfc/winutil.cpp`,
  and
  `/mnt/c/Program Files (x86)/Microsoft Visual Studio 8/VC/ce/atlmfc/src/mfc/thrdcore.cpp`
  - Layout and child reposition paths use `GetWindowRect`,
    `ScreenToClient`, `SetWindowPos`, and `GetClientRect`.
  - Subclassing/debug/text paths call `GetWindowLong`, `SetWindowLong`,
    `GetWindowTextLength`, `GetWindowText`, `GetClassName`, `DestroyWindow`,
    `GetParent`, and `SetFocus`.
  - CE MFC idle/modal/layout paths walk windows with `GetWindow(...,
    GW_CHILD)`, `GetWindow(..., GW_HWNDFIRST)`, `GW_HWNDNEXT`, `GW_HWNDPREV`,
    and `GW_OWNER`, including `WM_IDLEUPDATECMDUI` descendant/frame traversal.

- COREDLL resources:
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/CORE/DLL/resource.cpp`
  - `FindResourceW` searches MUI first and then base module resources.
  - `LoadResource` returns the data pointer computed from module base plus
    resource RVA.
  - `SizeofResource` returns the resource data size from the resource data
    entry.
  - `LoadStringW` locates the string-table segment `(id >> 4) + 1`, advances
    by counted UTF-16 strings, copies at most `nBufMax - 1` characters, and
    appends a null terminator.

- COM/OLE initialization reference:
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/DEVICE/SERVICES/CORE/servcom.cpp`
  - Services load `ole32.dll`, bind `CoInitializeEx` and `CoUninitialize`, and
    initialize COM before COM maintenance work.

- COREDLL multimedia ordinals:
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/CORE/DLL/core_common.def`,
  `/mnt/c/Program Files (x86)/Windows CE Tools/wce420/STANDARDSDK_420/Include/Mipsii/mmsystem.h`,
  and
  `/mnt/c/Program Files (x86)/Windows CE Tools/wce420/STANDARDSDK_420/Include/Mipsii/mmreg.h`
  - Lists waveOut exports including `waveOutSetVolume @382`,
    `waveOutClose @384`, `waveOutWrite @387`, `waveOutReset @390`, and
    `waveOutOpen @399`.
  - SDK headers define `WAVEHDR`, `WAVEFORMATEX`, `WAVEOUTCAPS`, `MMTIME`,
    `WHDR_*`, `TIME_*`, format masks, and capability flags used by the
    unplugged virtual waveOut adapter.
  - Converted into checked-in Rust constants and a static ordinal `match` in
    `src/ce/coredll_ordinals.rs`; `src/ce/coredll.rs` keeps parser helpers only
    for validation/reference work.

- COREDLL CRT/math exports:
  `C:\WINCE600\PUBLIC\COMMON\OAK\LIB\MIPSII\RETAIL\coredll.def`,
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/CORE/DLL/CRT/corelib1.def`
  and
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/INC/crt_ordinals.h`
  - Define common CRT math exports such as `_hypot @1023`, `sqrt @1060`,
    `pow @1051`, and MIPS helpers such as `__ll_div @2005`.
  - The same checked-in ordinal evidence includes narrow CRT helpers used by
    real companion DLLs, including `strcat @1063` and `strcpy @1066`.

- Mounted iNavi companion DLL bytes:
  `D:\INAVI_Emulator\INAVI\INavi\AuthLibrary.dll`,
  `D:\INAVI_Emulator\INAVI\INavi\TpSysAuth.dll`,
  `D:\INAVI_Emulator\INAVI\INavi\mMbcAuth.dll`,
  `D:\INAVI_Emulator\INAVI\INavi\tpeg_if_dll.dll`, and
  `D:\INAVI_Emulator\INAVI\INavi\tw_tpeg_if_dll.dll`
  - These are runtime inputs beside the main executable, not emulator shims.
    v3 now preloads sibling DLLs from the executable directory as a generic
    launch bridge while on-demand `LoadLibraryW` module mapping remains open.

## MFC CE Reference Source

- MFC process startup / show state:
  `/mnt/c/Program Files (x86)/Microsoft Visual Studio 8/VC/ce/atlmfc/src/mfc/appinit.cpp`,
  `/mnt/c/Program Files (x86)/Microsoft Visual Studio 8/VC/ce/atlmfc/src/mfc/docmgr.cpp`,
  and
  `/mnt/c/Program Files (x86)/Microsoft Visual Studio 8/VC/ce/atlmfc/src/mfc/winfrm.cpp`
  - `AfxWinInit` copies `hInstance`, `lpCmdLine`, and `nCmdShow` into the
    `CWinApp` state.
  - Later document/frame startup consumes `m_nCmdShow` when calling
    `ShowWindow`, so a zeroed entry `A3` incorrectly hides the main window.

- MFC thread pump:
  `/mnt/c/Program Files (x86)/Microsoft Visual Studio 8/VC/ce/atlmfc/src/mfc/thrdcore.cpp`
  - `AfxInternalPumpMessage` calls `GetMessage`.
  - `CWinThread::Run` uses `PeekMessage(..., PM_NOREMOVE)` for idle detection
    and loops through `PumpMessage`; a `FALSE` `GetMessage` return unwinds the
    pump as a quit condition, so an empty queue must block instead of returning
    false.
  - `AfxInternalPumpMessage` calls `AfxPreTranslateMessage` before
    `TranslateMessage`/`DispatchMessage`; menu/input traces need to distinguish
    raw message delivery from messages consumed during MFC pretranslation.
  - Its exception path calls `ValidateRect` for `WM_PAINT`, and idle detection
    excludes `WM_PAINT`, so paint messages must be tied to real update-region
    validation.

- MFC paint DC:
  `/mnt/c/Program Files (x86)/Microsoft Visual Studio 8/VC/ce/atlmfc/src/mfc/wingdi.cpp`
  - `CPaintDC` attaches the HDC returned by `BeginPaint` and calls `EndPaint`
    in its destructor.

- MFC window dispatch:
  `/mnt/c/Program Files (x86)/Microsoft Visual Studio 8/VC/ce/atlmfc/src/mfc/wincore.cpp`
  - `CWnd::WindowProc` calls message-map handling before `DefWindowProc`.
  - `CWnd::WalkPreTranslateTree` walks from the target HWND toward the main
    window through `GetParent`, and `CWnd::PreTranslateInput` forwards keyboard
    and mouse ranges to `IsDialogMessage`. Use this as evidence when comparing
    delivered mouse messages against later menu/action visibility changes.
  - Window creation flows through `AfxCtxCreateWindowEx`, `PreCreateWindowEx`,
    and `PostCreateWindowEx`.
  - CE `PreCreateWindowEx` registers a hybrid `WCE_` class whose WNDPROC is
    `DefWindowProcEx`; `DefWindowProcEx` is expected to run on the first
    create-time message, restore the saved old proc through `SetWindowLong`, run
    the MFC create hook, and delegate the same message to `AfxWndProc`.
  - `CWnd::DefWindowProc` and superclass paths call `CallWindowProc` for saved
    guest window procedures; the Unicorn import hook therefore enters the guest
    proc for CE `CallWindowProcW @285` instead of returning a raw stub value.
  - CE MFC post-processes `WM_DESTROY` by sending `WM_NCDESTROY`; Rust now
    covers the raw/kernel `WM_DESTROY` cleanup boundary and records
    `WM_NCDESTROY` when raw `SendMessageW` or a Unicorn guest-WNDPROC return
    actually delivers it. Rust does not add an OS-side automatic
    `WM_NCDESTROY` send because this CE MFC source path explicitly fakes the
    message above GWE. CE MFC `atlosapice.h` defines that fake message as
    `WM_APP - 1`, so v3 uses `0x7fff` rather than the desktop
    `WM_NCDESTROY` value. Guest child destroy-message ordering is now chained
    through Unicorn WNDPROC callouts before final root cleanup; remaining
    lifecycle work is focused on synchronous-send ownership and destroyed-target
    behavior.

## Prior Emulator Reference

- v2 diagnostic companion launcher:
  `..\wince_emulator_v2\tools\autodrive_inavi.ps1` and
  `..\wince_emulator_v2\README.md`
  - The autodrive harness starts `TBT\MultiTBT.exe` from the SDMMC backing when
    present unless `-NoCompanion` is set, and `-CompanionTarget` can override
    or add explicit companions. The companion uses the same emulator binary,
    registry, SDMMC path, serial map, DLL search dirs, and a headless/limited
    instruction run, with stdout/stderr captured beside the parent run.
  - v2 documents this as route-search diagnostic support and notes that no
    guest `CreateProcessW` launch for `MultiTBT.exe` had been observed. Rust
    v3 therefore exposes an explicit generic `--companion-image` /
    `--companion-target` launcher for parity, but keeps automatic app-specific
    startup and final shared behavior out of the emulator core until CE-like
    process/mapping/window IPC is implemented.

- Remote server API shape:
  `../WinCE_Emulator_v2/src/remote_server.cpp` and
  `../WinCE_Emulator_v2/src/ce_remote.h`
  - Remote routes and WebSocket control messages accept touch, key, location,
    NMEA, IMU, pause, resume, status, logs, frame, MJPEG, and audio endpoints.
  - `CeRemote` stores queued touch/key events, serial bytes, audio chunks, IMU
    state, audio client counts, and paused state.
  - v2's `/api/v1/logs/recent?lines=N` and control WebSocket `"logs"` response
    both read `runtime.recentRemoteLogLines(lines)`; Rust v3 mirrors the same
    shape by publishing the bounded `CeRemote` recent-log ring through
    `RemoteServer`.
  - v2's control WebSocket answers `"status"` in-place from
    `runtime.remoteStatusJson()`; Rust v3 now returns the latest published
    v2-shaped status without queueing a synthetic control message.
  - `materializeRemoteAudioChunkLocked` and `CeAudio::liveSlice` tie remote
    audio to the host playback cursor, so the Rust websocket sink models
    host-time client cursors and partial-chunk late joins instead of a single
    global audio drain.
  - Rust now splits audio delivery into registered sink adapters:
    `HostAudioSink`, `WebSocketAudioSink`, and debug-only `LoggingAudioSink`;
    only the websocket/remote queue is connected, while host playback remains
    deliberately unplugged even though the Windows host boundary is represented
    with the `windows` crate.

- CE file namespace / SDMMC mount precedent:
  `../WinCE_Emulator_v2/README.md`,
  `../WinCE_Emulator_v2/src/synthetic_dll.cpp`,
  `../WinCE_Emulator_v2/src/coredll_fs.cpp`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winbase.h`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\fsioctl.h`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\storemgr.h`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\extfile.h`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\mextfile.h`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\pathapi.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\volumeapi.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\CORE\DLL\apis.c`,
  `C:\WINCE600\PUBLIC\SHELL\OAK\HPC\CESHELL\API\shfileop.cpp`,
  `C:\WINCE600\PUBLIC\SHELL\OAK\HPC\CESHELL\API\recbin.cpp`, and
  `C:\WINCE600\PUBLIC\SHELL\OAK\HPC\CESHELL\UI\shelldialogs.cpp`
  - v2 exposed `SDMMC Disk` as a CE virtual root and mapped the main module
    under `\SDMMC Disk\...` when the host image lived beneath that root.
  - Root-relative probes under the SDMMC backing were supported, but `\`
    itself represented the CE namespace. FSDMGR `pathapi.cpp`
    `InternalFindFirstFileW` sends root-directory searches through
    `ROOTFS_FindFirstFileW`, while `fsdmain.cpp` `STOREMGR_GetOidInfoEx`
    reports mount folders as directory objects and adds
    `FILE_ATTRIBUTE_TEMPORARY` for non-permanent mounts. Rust now merges visible
    mount folders and object-store root entries for root `FindFirstFileW`
    enumeration, with mount metadata taking precedence over same-named
    object-store directories.
  - `winbase.h` declares `GetDiskFreeSpaceExW` with three `ULARGE_INTEGER`
    outputs, while `extfile.h`/`mextfile.h` declare and trap
    `AFS_GetDiskFreeSpace` as the sector/cluster FSD boundary. FSDMGR
    `pathapi.cpp` resolves a path to a volume, calls `AFS_GetDiskFreeSpace`,
    and multiplies clusters by sectors/bytes-per-sector for the Ex byte
    totals; the CE shell file-operation/recycle-bin/storage-dialog code calls
    `GetDiskFreeSpaceEx` when presenting copy/delete/storage capacity
    decisions. Rust now routes mounted CE paths to their configured mount
    total/free byte counts while null/object-store paths keep using the
    configured object-store capacity, and reports matching `AFS_*`
    sector/cluster values for lower-level callers. The same `extfile.h`
    signatures and FSDMGR `pathapi.cpp` call sites anchor the direct AFS
    create/remove-directory, get/set-attributes, create-file, delete,
    move/presto-chango rename, find-first, and first-change-notification
    ordinals; Rust currently routes full guest paths through the existing CE
    namespace. `fsioctl.h` defines `FSCTL_GET_VOLUME_INFO`, `storemgr.h`
    defines `CE_VOLUME_INFO` and its store/RAMFS/removable flags, FSDMGR
    `volumeapi.cpp` validates the info-level input and fixed output size
    before returning volume metadata, and COREDLL `apis.c` uses
    `CeFsIoControlW(... FSCTL_GET_VOLUME_INFO ...)` for RAMFS-aware file
    copying. Rust now reports matching mounted-root/object-store
    `CE_VOLUME_INFO` through `CeFsIoControlW`, direct `AFS_FsIoControlW`, and
    `CeGetVolumeInfoW`; full mount-table volume handles and broader FSCTL
    forwarding remain queued fidelity gaps.

- CE file write syscall and error surface:
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winbase.h`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winerror.h`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\fsdmgr.h`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\fileapi.cpp`, and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\fscall.c`
  - `winbase.h` declares `WriteFile(HANDLE, LPCVOID, DWORD, LPDWORD,
    LPOVERLAPPED)`, so the raw COREDLL path must return a `BOOL` and write
    `lpNumberOfBytesWritten` when supplied.
  - `winerror.h` defines `ERROR_ACCESS_DENIED` as `5L`.
  - `fsdmgr.h` and FSDMGR `fileapi.cpp` route `WriteFile` through the
    filesystem handle implementation, and NK `fscall.c` bridges `FSWriteFile`
    through the file-handle call path. Rust keeps existing host-backed file
    handles open, writes through writable handles, reports bytes written, and
    uses `ERROR_ACCESS_DENIED` for valid non-writable handles.

- CE process entry / module-name precedent:
  `../WinCE_Emulator_v2/src/main.cpp` and
  `../WinCE_Emulator_v2/src/coredll_named_dispatch.cpp`
  - v2 entered the main image with `A0=hInstance`, `A1=0`,
    `A2=guestCommandLine`, and `A3=1`.
  - v2 `GetModuleFileNameW` resolves the main module path when the requested
    module handle is the process module base.

- CE enabled-window precedent:
  `../WinCE_Emulator_v2/src/coredll_window.cpp`
  - v2 returned the previous enabled state from `EnableWindow` and treated the
    enabled bit as part of the virtual HWND state. Rust keeps CE source as the
    behavior authority, but this corroborates the previous-state return and
    enabled-state ownership path.

- CE message wait / scheduler wake authority:
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\schedule.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\syncobj.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\GWE\GWEAPI\msgqueue.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\INC\cmsgque.h`, and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winuser.h`
  - The current release/no-trace iNavi frontier is `GetMessageW` blocked on
    thread 1 with no synthetic app state. Continue the port by routing timer,
    posted input, serial/audio/process, and synchronous-send wakeups through
    scheduler-owned wait state and GWE message queues as CE does, rather than
    resuming blocked message calls from ad hoc subsystem paths.
  - Current-thread `GetMessageW` waits must still advance CE timer state and
    pump GWE timers before returning a queued `WM_TIMER` message. Rust now uses
    the same scheduler/GWE queue readiness check after long timer delays that
    it already used for short timer fast-forward.

- CE serial timeout authority:
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winbase.h`,
  `C:\WINCE600\PRIVATE\WINCEOS\DRIVERS\SERDEV\serial.c`, and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\DEVICE\DEVCORE\devfile.c`
  - `winbase.h` defines `COMMTIMEOUTS` as five DWORD fields:
    `ReadIntervalTimeout`, `ReadTotalTimeoutMultiplier`,
    `ReadTotalTimeoutConstant`, `WriteTotalTimeoutMultiplier`, and
    `WriteTotalTimeoutConstant`.
  - CE `serial.c` exposes `GetCommTimeouts` and `SetCommTimeouts` as comm API
    wrappers over the device/file-handle boundary, and `devfile.c` routes
    device `ReadFile` through the device manager rather than normal filesystem
    file data.
  - Rust stores timeout state on each opened serial `DeviceSession`, round-trips
    it through raw COREDLL `GetCommTimeouts`/`SetCommTimeouts`, and uses finite
    read-total timeouts to complete empty serial `ReadFile` waits with zero
    bytes instead of treating every empty serial read as an infinite wait.
  - The Unicorn serial `ReadFile` bridge also handles the CE timeout case when
    the current thread has no suspended peer to run: finite empty reads complete
    on the original thread with zero bytes transferred, while the wait
    registration is removed from both the local blocked-wait list and the
    kernel scheduler queue.

- CE CRT narrow varargs formatting authority:
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\INC\crt_ordinals.h`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\LIB\MIPSII\RETAIL\coredll.def`, and
  the generated `src\ce\coredll_ordinals.rs`
  - The MIPSII COREDLL export table maps `sprintf` to ordinal `719`,
    `vsprintf` to ordinal `1146`, `_vsnprintf` to ordinal `1147`, and the
    wide variants to the corresponding `swprintf`/`vswprintf` ordinals.
  - Rust keeps `vsprintf` on the raw COREDLL syscall boundary and reads the
    guest MIPS `va_list` as a pointer to DWORD arguments, matching the existing
    `_vsnprintf`/wide varargs behavior rather than special-casing the mounted
    `happyway_win.exe` call site.

- CE locale validity authority:
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winnls.h`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winbase.h`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\LIB\MIPSII\RETAIL\coredll.def`
  - COREDLL exports `IsValidLocale` as ordinal `209`.
  - SDK NLS headers define the `LCID_INSTALLED` and `LCID_SUPPORTED` flag
    shape used by `IsValidLocale(LCID, DWORD)`. Mounted evidence from
    `happyway_win.exe` calls `IsValidLocale(0x0412, LCID_INSTALLED)`, so Rust
    now treats Korean `0x0412` and normal default/known LCIDs as valid under
    those CE/Win32 flags while rejecting invalid flag values.

- CE process/window lifetime and heap mapping evidence:
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\schedule.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\GWE\GWEAPI\msgqueue.cpp`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winbase.h`, and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\memory.h`
  - CE process termination must signal process waiters and release thread/GWE
    ownership; Rust now applies the same owned-window cleanup used for launched
    child exits when the current-process pseudo handle is terminated.
  - CE heap APIs return committed guest memory for the requested allocation
    range. Rust's Unicorn backend now maps heap spillover pages with the same
  page-aware range mapper used for virtual allocations so large
  `HeapAlloc` results are writable all the way to their tail.

- CE shell notification data authority:
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\shsdkstc.h`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\shellsdk.h`,
  `C:\WINCE600\PRIVATE\SHELL\SHELLPSL\HAVEAYGSHELL\api.cpp`, and
  `C:\WINCE600\PRIVATE\SHELL\SHELLPSL\HAVEAYGSHELL\shellpsl.cpp`,
  `C:\WINCE600\PUBLIC\SHELL\OAK\HPC\EXPLORER\AYGSHELLFUNCS\HAVEAYGSHELL\notification.cpp`,
  and
  `C:\WINCE600\PUBLIC\SHELL\OAK\HPC\EXPLORER\AYGSHELLFUNCS\HAVEAYGSHELL\bubble.cpp`,
  plus the taskbar bubble implementation at
  `C:\WINCE600\PUBLIC\SHELL\OAK\HPC\EXPLORER\TASKBAR\bubble.cpp` and
  `C:\WINCE600\PUBLIC\SHELLSDK\SDK\INC\shellsdkguids.h`
  - `shsdkstc.h` defines `SHNOTIFICATIONDATA` as the 56-byte CE struct keyed
    by `CLSID` and `dwID`, with title/HTML strings carried through marshalled
    pointers in the `I` API set signatures.
  - `shellsdk.h` defines `SHNN_SHOW`, `SHNN_DISMISS`, `SHNN_LINKSEL`, and the
    `NMSHN` callback payload. `shellpsl.cpp` marshals `NMSHN` into the target
    process and sends `WM_NOTIFY` with `wParam=hdr.idFrom` and
    `lParam=NMSHN*`; for link-selection notifications it appends the selected
    link string after the `NMSHN` allocation and points `pszLink` at that
    receiver-local string. CE bubble dismiss callbacks carry the timeout
    boolean through `NMSHN.fTimeout` in the same union field.
  - `api.cpp` maps `SHNotificationAddI`/`UpdateI`/`RemoveI`/`GetDataI` to the
    shell API set and returns Win32 error codes (`ERROR_SUCCESS`,
    `ERROR_INVALID_PARAMETER`, `ERROR_INVALID_DATA`) rather than BOOL success;
    the PSL signatures pass the full `SHNOTIFICATIONDATA` for add and update.
  - `notification.cpp` stores notification data in the taskbar/bubble lists
    and copies the persisted struct/title/HTML fields back through
    `GetNotificationData`. Rust now preserves that app-visible data in
    `ShellSystem`, can post the window-based `WM_NOTIFY`/`NMSHN` sink callback
    for stored notification events, rejects add/update records with nonzero
    dead sink HWNDs, rejects zero/unknown update masks, keeps the previous icon
    on null `SHNUM_ICON` updates, preserves the CE add-vs-update duration split
    where add defaults zero duration but update stores `pndNew->csDuration`
    literally, and leaves add-time `grfFlags`, `hwndSink`, and `lParam`
    unchanged because `UpdateBubble` only assigns the masked bubble fields.
    Rust also marshals optional `SHNN_LINKSEL` link strings into the receiver
    `NMSHN` allocation, carries `SHNN_DISMISS` `fTimeout`, and prunes
    window-bound notify icon, notification, and change-notification records
    when the sink HWND is destroyed directly or removed during process-exit
    cleanup. Explicit `SHNotificationRemoveI` and sink-window cleanup also
    purge pending callback records for that `(CLSID, dwID)`, matching the CE
    taskbar model where callbacks are delivered from live bubble records rather
    than from detached stale queue entries. CE `GetNotificationData` ignores
    `cbTitle` and assumes `pszTitle` has the fixed taskbar-label capacity
    (`CCHMAXTBLABEL == MAX_PATH`), so raw `SHNotificationGetDataI` now writes a
    bounded fixed-title copy even when the marshalled `cbTitle` argument is
    zero. `SHNotificationAddII` validates title/HTML content by pointer
    presence (`pszHTML == NULL` and both title/HTML null), not by string
    length, so raw `SHNotificationAddI` now accepts a non-null empty HTML string
    for `SHNP_INFORM` while still rejecting a null HTML pointer.
    `CTaskBar::UpdateBubble` handles `SHNUM_PRIORITY` changes by removing the
    record from its old priority list, assigning `m_npPriority`, then adding it
    to the iconic tray list or inform bubble list, so Rust now keeps separate
    inform/iconic notification key lists synchronized with add, update, remove,
    expiration, and sink-window cleanup.
    taskbar `bubble.cpp` sends `SHNN_SHOW` through the window sink from
    `PopUp` without calling `GetCallbackInterface`, while link, dismiss, and
    non-cancel command paths attempt `IShellNotificationCallback` COM methods
    before also notifying the sink window. Rust now keeps `SHNN_SHOW` as a
    window-notification-only path, records the callback method, CE vtable
    offset, and typed arguments for the link/dismiss/command COM candidates
    from non-null notification CLSIDs, can enter a pending runtime guest vtable
    method when the saved interface pointer is readable, restores non-null
    pending callback records to the front of the queue when transient guest
    vtable dispatch or unmapped callback-interface pointer reads cannot yet be
    entered, and posts the command-selection `WM_COMMAND` sink message. The raw
    `I` API `cbData`
    argument is the marshalled
    `SHNOTIFICATIONDATA` byte count and not a callback pointer, so v3 no longer
    stores it as callback state. `bubble.cpp::GetCallbackInterface` acquires
    the callback with `CoCreateInstance(m_tbiiBubble.clsid, NULL, CLSCTX_ALL,
    IID_IShellNotificationCallback, &m_pishnc)`, and the local OLE import path
    now routes `CoCreateInstance` through the emulator COM registry using the
    CLSID/IID GUID bytes read from guest memory, with `ppv` zeroing/writeback
    and the registered interface token returned as the acquired pointer when
    present. `shellsdkguids.h` defines `IID_IShellNotificationCallback` as
    `DEFINE_OLEGUID(..., 0x000214C0L, 0, 0)`, so Rust stores the CE memory-order
    GUID bytes and uses them when a notification with only a CLSID queues a
    link/dismiss/command callback. If the local COM registry has that CLSID, the
    queue now records the acquired `IShellNotificationCallback*` token instead
    of a null callback pointer. A Unicorn regression now maps a guest COM
    interface pointer/vtable and verifies the callback dispatcher enters the
    selected `IShellNotificationCallback` method with the CE-style MIPS
    `this`/argument registers, return stub, stack adjustment, and pending return
    bookkeeping. Integrated Explorer/taskbar validation with a real guest COM
    object lifecycle and visual bubble/taskbar rendering remain queued
    separately.
    CE `UpdateTimedNotificationIcons` walks iconic bubble notifications in the
    taskbar list, sets `HHTBF_DESTROYICON` on expired items, then calls
    `DeleteItem(..., TRUE)`, whose taskbar path destroys `m_hIcon` only when
    that flag is present. Rust now records destruction of the copied
    notification icon when an iconic `SHNotification` expires, is explicitly
    removed, or is cleaned up with its sink window/process.

- CE Winsock exception-readiness authority:
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winsock.h` and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winsock2.h`
  - The CE Winsock headers define `SO_ERROR` as the socket error status queried
    by `getsockopt`, `MSG_OOB` as out-of-band data, and the `select` signature
    with an in/out `exceptfds` set. Rust now keeps `SO_ERROR`-backed exception
    readiness separate from stateful `MSG_OOB` readiness: an OOB send marks the
    connected peer as exception-ready, `select` retains that peer in
    `exceptfds` on both raw imports and parked Unicorn scheduler replay,
    `getsockopt(SO_ERROR)` still reports zero for pure OOB readiness, and
    `recv(..., MSG_OOB)` clears the OOB-ready bit after reading.
