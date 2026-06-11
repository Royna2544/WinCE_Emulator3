# Current Engineering Plan

Regenerated on 2026-06-11 from the current `src/` and `tests/` implementation state. This file is the active roadmap; older historical status text has been removed.

## Implemented Baseline

- Runtime loader: MIPS PE parsing, import patching, runtime `LoadLibraryEx` mapping, dependency loading, forwarder resolution, trampoline patching, `DONT_RESOLVE_DLL_REFERENCES`, `LOAD_LIBRARY_AS_DATAFILE`, TLS directory parsing, and DLL lifecycle handling are implemented in the Unicorn path.
- Shell and file APIs: `ShellExecuteEx`, registry-backed file associations, shortcuts, recent-doc tracking, special folders, `SHGetFileInfo`, `ExtractIconExW`, resource-backed icon extraction, synthetic shell icons, image lists, selected DIB drawing, and icon rendering all have raw test coverage.
- Windowing and GWE: windows, message queues, default window proc behavior, modal menu/message paths, clipboard/caret support, accelerator translation, `SendMessageTimeout` blocking setup, timeout handling, and destroyed-target result writes are implemented to the current raw-test level.
- File system and storage: rooted host-backed paths, mount roots, directory enumeration, file attributes, change notification handles, `CeGetFileNotificationInfo`, duplicate coalescing, transient create/delete coalescing, and modified/delete coalescing are covered.
- Winsock and remote IO: CE-facing Winsock dispatch, isolated NAT addressing, TCP/UDP socket creation, bind/listen/connect/accept/send/recv/select paths, `FD_SETSIZE`/fd-set validation, nonblocking state, socket wait candidates, and scheduler wake integration are present.
- Core subsystems: thread handles, waits, events, semaphores, mutexes, TLS, critical sections, process handles, memory APIs, registry APIs, GDI object management, bitmap/DIB drawing, and scheduler readiness selection are established.

## Active Roadmap

1. Close the loader parity gaps outside the current happy path.
   - Add focused fixture variants for raw/non-Unicorn loader behavior where the Unicorn runtime has stronger coverage.
   - Keep validating dependency attach/detach ordering, forwarder failure modes, datafile loads, no-resolve loads, and invalid image edges.

2. Finish shell rendering parity.
   - Expand `ExtractIconExW` coverage across remaining PE icon formats, `RT_ICON` ordinal failure cases, and non-PE fallback behavior.
   - Add more `DrawIconEx` and image-list drawing cases for masks, additional stretched-size paths, framebuffer output, and invalid handle cleanup.

3. Finish modal UI and cross-thread message semantics.
   - Extend `SendMessageTimeout` beyond current timeout/destroyed-target cases into nested sends, reentrant waits, abort-if-hung edge cases, and `ReplyMessage` parity.
   - Keep menu and message-box behavior aligned with CE blocking, owner-window, and command-routing expectations.

4. Complete shell notification behavior.
   - Verify `Shell_NotifyIcon` and `SHNotification*` data lifetimes, update/remove paths, taskbar-visible state, and COM notification callback dispatch in integrated runtime scenarios.
   - Add negative tests for invalid callback pointers, stale handles, and process/thread ownership boundaries.

5. Harden mounted storage behavior.
   - Extend file-change notifications across mount roots, volume handles, rename churn, and FSDMGR-style ownership semantics.
   - Validate external SD-card behavior against the mounted iNavi layout, including writable vs read-only host directories.

6. Expand input, text, and GDI fidelity.
   - Add coverage for IME-facing keyboard input, caret timing, glyph metrics, font fallback, text clipping, palette selection, alpha/mask behavior, and device-context state restoration.

7. Advance the iNavi route/runtime frontier.
   - Continue from the current shell/process/window readiness point toward route search, destination entry, map rendering, and realistic navigation flow.
   - Preserve traceability with targeted progress logs, source references, and reproducible command lines when behavior changes.

## Validation Gates

- Run `cargo fmt` after Rust edits.
- Run `cargo check --features unicorn,trace,win32-desktop` for shared subsystem changes.
- Run targeted tests for the touched area, for example:
  - `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel`
  - `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  - `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`
  - `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems`
- Run mounted-layout or application-level checks when touching process startup, shell routing, storage, or UI scheduling behavior.

## Completion Criteria

Delete this roadmap only after the active roadmap is empty, source/tests reflect the final state, and the validation gates pass for the completed scope.

## WinCE 6.0 Reference Gap Appendix

Reference tree used: `C:\WINCE600`, sampled on 2026-06-11. The local source already covers many APIs in these areas; the missing work below is the remaining semantic and validation delta against the CE 6.0 headers and private implementation patterns.

### Shell API And Notification Gaps

Reference files:
- `C:\WINCE600\PUBLIC\COMMON\SDK\INC\shellapi.h`
- `C:\WINCE600\PUBLIC\COMMON\SDK\INC\shellsdk.h`
- `C:\WINCE600\PUBLIC\WCESHELLFE\OAK\TASKMAN\minserver.cpp`
- `C:\WINCE600\PUBLIC\WCESHELLFE\OAK\TASKMAN\mintask.cpp`

Missing or under-validated:
- `Shell_NotifyIcon` now rejects duplicate `NIM_ADD` records, honors `NIF_MESSAGE`, `NIF_ICON`, `NIF_TIP`, and `NIF_STATE` when capturing `NOTIFYICONDATA` members, requires the fixed CE `NOTIFYICONDATAW` footprint with its 64-WCHAR `szTip` buffer before mutating state, and follows the CE taskbar update rule that `NIM_MODIFY | NIF_ICON` with a null `hIcon` leaves the existing tray icon intact. Remaining gaps are integrated taskbar-visible state and broader tray-server routing validation against `minserver.cpp`, `mintask.cpp`, and the Explorer taskbar sources.
- `SHNotification*` now records integrated CE `IShellNotificationCallback` metadata for `OnShow`, `OnCommandSelected`, `OnLinkSelected`, and `OnDismiss`, including vtable offsets and typed arguments, and the raw `I` API no longer mistakes the marshalled `cbData` slot for a callback pointer. Remaining work is actual guest COM vtable invocation and failure cleanup around `CoCreateInstance`/callback failures.
- `SHNotificationUpdate` now preserves existing icons on `SHNUM_ICON` updates with null `hicon`, follows the CE update path by allowing stale `hwndSink` values in the incoming update struct while preserving the original sink, and moves records between inform and iconic notification lists when `SHNUM_PRIORITY` changes. `SHNotificationRemove` and sink window/process cleanup now purge pending notification callback records and priority-list entries for the removed `(CLSID, id)`, `SHNotificationGetData` follows the CE fixed-title-buffer behavior when `cbTitle` is zero, and `SHNotificationAddI` validates title/HTML presence by marshalled pointer nullness instead of string length. Remaining gaps are broader title/HTML allocation failures and broader remove/update races.
- `SHGetFileInfo` now covers the CE-supported flag set shape for `SHGFI_ICON`, `SHGFI_DISPLAYNAME`, `SHGFI_TYPENAME`, `SHGFI_ATTRIBUTES`, `SHGFI_SYSICONINDEX`, `SHGFI_SMALLICON`, and `SHGFI_USEFILEATTRIBUTES`, including CE `SFGAO_*` shell-attribute output for filesystem, folder, shortcut, read-only, and inaccessible network-folder cases, and returns the system image-list handle for both `SHGFI_ICON` and `SHGFI_SYSICONINDEX` queries. Remaining gaps are broader shell-folder PIDL/display-name parity and more edge cases around icon-cache lifetime.
- `ExtractIconExW` now extracts PE resources, reports actual PE `RT_GROUP_ICON` counts for `nIconIndex == -1`, fills successive large/small output-array slots for multi-group extraction, selects distinct large/small `RT_ICON` entries from multi-size groups, covers string-named `RT_GROUP_ICON` entries with ordinal `RT_ICON` payloads, resolves sparse integer `RT_GROUP_ICON` resource IDs when zero-based enumeration does not cover the requested index, preserves indexed 4bpp and 8bpp `RT_ICON` color tables, renders decoded 4bpp icon pixels, reports `ERROR_RESOURCE_NAME_NOT_FOUND` for malformed present group/icon resources and missing primary or secondary `RT_ICON` ordinals instead of falling back to synthetic shell icons, tolerates RT_ICON payloads that omit trailing AND-mask bytes by extracting a color-only icon, and preserves the synthetic one-icon fallback for non-PE shell-icon paths. Remaining gaps are broader PE format variants and non-PE fallback edge cases.
- `DrawIconEx` now paints bitmap-backed icons into framebuffers and selected memory DIBs, validates icon/HDC handles, scales framebuffer and selected-memory-DIB draws from the icon bitmap's native source dimensions when callers request a non-native destination size, honors bitmap-backed `DI_MASK` by drawing the icon mask bitmap instead of the color image, and covers a 1bpp mask-only framebuffer draw. `ImageList_Create` now rejects flags outside CE's `ILC_VALID` mask while still accepting CE-valid private/shared flags, and normalizes a zero CE color mask (`ILC_COLOR`) to `ILC_COLORDDB` before storing image-list flags. `ImageList_Add` and `ImageList_AddMasked` now reject bitmaps narrower than the image-list slot width instead of reporting success with zero appended strips, matching CE `ImageList::Add`. `ImageList_LoadImage` now follows CE by substituting the loaded bitmap height when callers pass `cx == 0`, so square-height bitmap strips split correctly, failing negative `cx` values through the image-list creation gate, carrying CE mask/color-depth flags from `crMask` and the loaded bitmap into the created image list, and using the unmasked `ImageList_Add(..., NULL)` path when `crMask == CLR_NONE`. `ImageList_GetIconSize` now validates both output pointers before writing either dimension, matching CE's reviewed-parameter guard. `ImageList_ReplaceIcon` now accepts only `-1` as the append sentinel and rejects indexes below `-1`, matching CE's Win95-compatible guard. `ImageList_Merge` now sizes the merged list from the CE union of the two image rectangles and combines mask/color creation flags like CE. `ImageList_Copy` now rejects flags outside CE's `ILCF_VALID` mask, rejects CE-unsupported cross-list copies, copies only between valid slots in the same image list for `ILCF_MOVE`, and swaps same-list slots for `ILCF_SWAP`. `ImageList_Draw` now uses CE's `CLR_DEFAULT` background wrapper default, and `ImageList_DrawIndirect` now enforces the exact CE `IMAGELISTDRAWPARAMS` size before recording or rendering a draw, resolves `CLR_DEFAULT` backgrounds to the image-list background color, defaults zero `cx`/`cy` after applying `xBitmap`/`yBitmap`, writes those normalized draw parameters and CE's final overlay-pass `i`/`x`/`y`/`cx`/`cy`/`fStyle` mutations back into the caller's `IMAGELISTDRAWPARAMS`, recognizes CE's private `ILD_BLENDMASK` shape so `ILD_BLEND75` enters the bitmap-backed blend path instead of drawing unblended pixels, supports CE `rgbFg == CLR_NONE` destination blending for bitmap-backed draw targets, applies CE `ILD_ROP`/`dwRop` raster operations for `ILD_MASK` and `ILD_IMAGE` selected-DIB/framebuffer draws, and still draws registered overlays when `ILD_MASK` is combined with `ILD_OVERLAYMASK`. `ImageList_CopyDitherImage` now preserves destination image metadata like CE, masks `fStyle` to overlay bits, and mutates bitmap-backed destination image/mask pixels when backing storage is available. `ImageList_Remove`/`ImageList_SetImageCount` now preserve CE overlay slot indexes for single-image removals and truncation while only `ImageList_Remove(-1)` clears overlays and other negative remove indexes fail, and `ImageList_SetImageCount` now preserves the existing count and returns failure when the backing allocation request cannot be satisfied. `ImageList_SetOverlayImage` now rejects image lists created without `ILC_MASK` like CE's missing-mask-DC path, enforces CE's four overlay slots, and computes CE-style overlay mask bounds/`ILD_IMAGE` metadata for drawing, `ImageList_SetIconSize` now rejects no-op size changes while clearing all images/overlays on real size changes, `ImageList_DragMove` now returns success even when no drag image is active, `ImageList_SetDragCursorImage` now treats a missing drag/dither image as a successful no-op, and `ImageList_GetDragImage`/`ImageList_DragEnter`/`ImageList_DragLeave` now preserve CE static drag point, hotspot, and lock state even when no drag image is active. Remaining gaps are broader mask-only format variants, image-list palette/dither blend and overlay rendering edge cases, and additional cleanup/error combinations.

### GWE, Menu, And Modal Message Gaps

Reference file:
- `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winuser.h`

Missing or under-validated:
- `TrackPopupMenuEx` now covers the core CE popup loop shape, including `TPM_NONOTIFY`, `TPM_RETURNCMD`, disabled/default items, submenu cascades, owner notifications, cancellation behavior, CE horizontal/vertical alignment flags, and top-level `TPMPARAMS.rcExclude` screen-rectangle avoidance. Remaining gaps are deeper rcExclude edge placement and broader live nested modal routing.
- `MessageBoxW` now validates CE-supported button, default-button, icon, `MB_SETFOREGROUND`, `MB_TOPMOST`, and `MB_RTLREADING` flag shapes while rejecting unsupported desktop-only or undefined bits; it also covers default return IDs, yes/no/cancel layouts, owner-window disabled state, and queued modal input. Remaining gaps are deeper modality, timeout/pump behavior, foreground/topmost z-order effects, and broader live user-driven dispatch.
- `SendMessageTimeout` currently covers important paths, including unsupported flag rejection, zero-timeout cross-thread expiry, nonzero cross-thread queueing, destroyed-target completion, hung-thread `SMTO_ABORTIFHUNG`, same-thread synchronous dispatch/result writes, early `ReplyMessage` result propagation, and nested timeout sends where an inner `ReplyMessage` completes without clearing the outer send. Remaining gaps are broader reentrant dispatch and broader `SMTO_NORMAL`, `SMTO_BLOCK`, and `SMTO_ABORTIFHUNG` combinations used by CE callers.
- `MsgWaitForMultipleObjectsEx` now covers CE's ignored desktop `MWMO_WAITALL` bit, `QS_TIMER`, `QS_SENDMESSAGE`, `QS_ALLINPUT` over posted messages, paint, and timers, `MWMO_INPUTAVAILABLE` new-vs-existing input behavior, and signaled-handle precedence over message wake. Remaining gaps are broader mixed multi-handle waits, cancellation edges, and live modal-loop interaction.

### File Notification And FSDMGR Gaps

Reference files:
- `C:\WINCE600\PRIVATE\WINCEOS\INC\fsnotify_api.h`
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\pathapi.cpp`
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\NOTIFY\fsnotify.cpp`

Missing or under-validated:
- CE distinguishes external caller handles from internal/kernel-owned handles through `FSEXT_*`, `FSINT_*`, `NotifyGetNextChange`, and `INT_NotifyGetNextChange`; `FindCloseChangeNotification` now consumes valid-but-wrong handle types before reporting `ERROR_INVALID_HANDLE`, matching the public `DUPLICATE_CLOSE_SOURCE` path. Remaining work should validate process ownership, duplicated handles, access failures, and internal-vs-external notification handle behavior.
- `FindFirstChangeNotificationW` now canonicalizes unrooted, `.`, and `..` watch paths before directory validation and watch registration, matching the CE `SafeGetCanonicalPathW` path into volume resolution. Root and non-root watches now honor the CE `WatchSubtree` boundary: non-recursive watches see only immediate children, while recursive watches see deeper descendants. `MoveFileW` file-change records now follow CE `NotifyMoveFileEx` boundaries, reporting same-parent moves as rename old/new and cross-parent moves as remove/add with file-vs-directory notify filters. Remaining coverage should expand into mounted-volume ownership.
- `CeGetFileNotificationInfo` now follows the CE `FILE_NOTIFY_CHANGE_CEGETINFO` detail-record gate for signal-only vs detailed watches, and `FindNextChangeNotification` now consumes one outstanding notification signal instead of clearing every queued detailed record. Remaining gaps are access-denied/error propagation, process-owned handle semantics, and more exhaustive buffer/reset edge coverage.
- FSDMGR move behavior now covers mounted storage boundaries for cross-volume file moves, cross-volume directory rejection, mount-point rename errors, and CE same-parent vs cross-parent notification action mapping. Read-only mounted host roots now report access-denied for raw create/copy/delete/attribute mutation attempts. Remaining volume-handle paths should be tested for system volumes, additional rename churn, and mounted-volume ownership.

### Loader And Runtime Gaps

Reference evidence:
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\TOOLS\ROMIMAGE\ROMIMAGE\pbtimebomb.cpp`
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\LOGGER\celoginstrumentation.c`
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\CORE\DLL\loader.cpp`
- `C:\WINCE600\PUBLIC\OSTEST\CESYSGEN\makefile`

Missing or under-validated:
- `LoadLibraryEx` now validates CE resource-only combinations such as `LOAD_LIBRARY_AS_DATAFILE | DONT_RESOLVE_DLL_REFERENCES`, rejects non-null `hFile` handles with `ERROR_INVALID_PARAMETER`, and promotes an already loaded `DONT_RESOLVE_DLL_REFERENCES` module to normal loaded-module flags when a later raw/normal load requests it without no-import flags. Keep expanding cases around resource-only handle behavior as more CE source consumers are found.
- Loader lifecycle should be checked against CE build-entry behavior: `DllEntry`, `_DllMainCRTStartup`, C++ static constructors/destructors, TLS callbacks, and process/thread attach-detach ordering.
- Runtime DLL mapping parity should be audited outside Unicorn so raw loader tests and Unicorn execution do not silently diverge.
- Forwarder resolution now includes a host fixture where a named export forwards to a target DLL ordinal, and malformed literal forwarder strings fail closed instead of being whitespace-normalized into valid targets. Runtime `LoadLibraryW` failures now release modules newly loaded during the failed attempt, matching CE `DoImports` cleanup of dependency blocks, final dynamic `FreeLibrary` now releases the module's dependency refs after detach planning, failed runtime DLL maps now unmap their current image reservation before returning failure, current-image resource/trap/trampoline/module state is committed only after fallible parse/forwarder/resource steps succeed, and `DllMain(DLL_PROCESS_ATTACH)` false returns now set `ERROR_DLL_INIT_FAILED` while releasing refs owned by the failed load attempt. Remaining gaps are deeper lifecycle failure variants and broader integration fixtures for loader failure paths.

### Winsock And Network Gaps

Reference evidence:
- `C:\WINCE600\PRIVATE\TEST\NET\WINSOCK\22`
- `C:\WINCE600\PRIVATE\SERVERS\WEBPROXY\CORE\session.cpp`
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\DEVICE\SERVICES\CORE\servacpt.cpp`

Missing or under-validated:
- Local Winsock already has core socket calls, `ioctlsocket`, `setsockopt`, `getsockopt`, and `select`; `select` now accepts CE caller `nfds` values of `0`, `-1`, and active counts while rejecting null fd-set triads, `fd_count > FD_SETSIZE`, invalid socket handles, and fd-set memory faults with Winsock errors. Mixed read/write/except set filtering, zero-ready recovery, graceful TCP peer-close read readiness, TCP half-close read/write behavior, TCP reset read readiness/error propagation, repeated listener `select`/`accept` loops, and `SO_ERROR` clearing after `getsockopt` are covered. Remaining work is broader CE stress parity across higher-volume socket churn and full webproxy/service accept-loop patterns.
- Add more tests based on CE server/stress patterns for higher-volume socket churn, concurrent accept/send/close transitions, and webproxy/service session lifetimes.

### Input, IME, Text, And GDI Gaps

Reference evidence:
- `C:\WINCE600\PUBLIC\WCESHELLFE\OAK\INPUTMETHODS`
- `C:\WINCE600\PUBLIC\COMMON\SDK\SAMPLES\TESTIME`
- `C:\WINCE600\PRIVATE\TEST\STRESS\STRESS\MODULES\FONT`
- `C:\WINCE600\PRIVATE\TEST\STRESS\STRESS\MODULES\DIB`
- `C:\WINCE600\PRIVATE\TEST\STRESS\STRESS\MODULES\ALPHA`

Missing or under-validated:
- IME-facing input paths need coverage for composition, selection/deselection, keyboard layout transitions, and SIP/input-method callbacks.
- Text and font behavior needs CE-style glyph metrics, fallback, clipping, transparent/opaque background modes, and selected font lifetime checks.
- Bitmap and alpha paths need more coverage for DIB orientation, masks, palette state, stretch/blit combinations, and alpha edge cases.
