# Current Engineering Plan

Regenerated on 2026-06-11 from the current `src/` and `tests/` implementation state. This file is the active roadmap; older historical status text has been removed.

## Implemented Baseline

- Runtime loader: MIPS PE parsing, import patching, runtime `LoadLibraryEx` mapping, dependency loading, forwarder resolution, trampoline patching, `DONT_RESOLVE_DLL_REFERENCES`, `LOAD_LIBRARY_AS_DATAFILE`, TLS directory parsing, and DLL lifecycle handling are implemented in the Unicorn path.
- Shell and file APIs: `ShellExecuteEx`, registry-backed file associations, shortcuts, recent-doc tracking, special folders, `SHGetFileInfo`, `ExtractIconExW`, resource-backed icon extraction, synthetic shell icons, image lists, selected DIB drawing, and icon rendering all have raw test coverage.
- Windowing and GWE: windows, message queues, default window proc behavior, modal menu/message paths, clipboard/caret support, accelerator translation, `SendMessageTimeout` blocking setup, timeout handling, and destroyed-target result writes are implemented to the current raw-test level.
- File system and storage: rooted host-backed paths, mount roots, directory enumeration, file attributes, change notification handles, `CeGetFileNotificationInfo`, duplicate coalescing, transient create/delete coalescing, and modified/delete coalescing are covered.
- Winsock and remote IO: CE-facing Winsock dispatch, isolated NAT addressing, TCP/UDP socket creation, bind/listen/connect/accept/send/recv/select paths, nonblocking state, socket wait candidates, and scheduler wake integration are present.
- Core subsystems: thread handles, waits, events, semaphores, mutexes, TLS, critical sections, process handles, memory APIs, registry APIs, GDI object management, bitmap/DIB drawing, and scheduler readiness selection are established.

## Active Roadmap

1. Close the loader parity gaps outside the current happy path.
   - Add focused fixture variants for raw/non-Unicorn loader behavior where the Unicorn runtime has stronger coverage.
   - Keep validating dependency attach/detach ordering, forwarder failure modes, datafile loads, no-resolve loads, and invalid image edges.

2. Finish shell rendering parity.
   - Expand `ExtractIconExW` coverage across PE icon formats, malformed resource tables, ordinal resource names, multi-size icon groups, and non-PE fallback behavior.
   - Add more `DrawIconEx` and image-list drawing cases for masks, stretched sizes, selected DIBs, framebuffer output, and invalid handle cleanup.

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
- `Shell_NotifyIcon` should be validated against the CE tray-server callback model from `minserver.cpp` and `mintask.cpp`, including `NIM_ADD`, `NIM_MODIFY`, `NIM_DELETE`, `NIF_MESSAGE`, `NIF_ICON`, `NIF_TIP`, callback message delivery, tooltip truncation, duplicate `(hwnd,uID)` behavior, and stale modify/delete handling.
- `SHNotification*` has local COM callback dispatch support, but still needs integrated validation for the CE `IShellNotificationCallback` methods: `OnShow`, `OnCommandSelected`, `OnLinkSelected`, and `OnDismiss`.
- `SHNotificationUpdate` should be checked against CE update-mask behavior, data replacement rules, callback pointer lifetime, and remove/update races.
- `SHGetFileInfo` should be expanded across the exact CE-supported flag set: `SHGFI_ICON`, `SHGFI_DISPLAYNAME`, `SHGFI_TYPENAME`, `SHGFI_ATTRIBUTES`, `SHGFI_SYSICONINDEX`, `SHGFI_SMALLICON`, and `SHGFI_USEFILEATTRIBUTES`.
- `ExtractIconExW` now extracts PE resources, but needs CE-reference coverage for `nIconIndex == -1` counts, multi-group ordering, large/small output arrays, malformed group/icon resources, and non-PE fallback.

### GWE, Menu, And Modal Message Gaps

Reference file:
- `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winuser.h`

Missing or under-validated:
- `TrackPopupMenuEx` should be validated against all CE flags and structs exposed in `winuser.h`: alignment flags, `TPM_NONOTIFY`, `TPM_RETURNCMD`, `TPMPARAMS.rcExclude`, disabled/default items, submenu cascades, owner notifications, and cancellation behavior.
- `MessageBoxW` should be validated across CE flags and return values: icon flags, default buttons, yes/no/cancel layouts, modality, owner-window disabled state, timeout/pump behavior, and foreground behavior.
- `SendMessageTimeout` currently covers important paths but still needs parity for nested sends, reentrant dispatch, `ReplyMessage`, `QS_SENDMESSAGE` interaction during modal waits, unsupported flag rejection, and all `SMTO_NORMAL`, `SMTO_BLOCK`, and `SMTO_ABORTIFHUNG` combinations used by CE callers.
- `MsgWaitForMultipleObjectsEx` should be cross-checked against the CE queue-status masks, especially `QS_SENDMESSAGE`, `QS_ALLINPUT`, timer readiness, posted message readiness, and mixed kernel-handle waits.

### File Notification And FSDMGR Gaps

Reference files:
- `C:\WINCE600\PRIVATE\WINCEOS\INC\fsnotify_api.h`
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\pathapi.cpp`
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\NOTIFY\fsnotify.cpp`

Missing or under-validated:
- CE distinguishes external caller handles from internal/kernel-owned handles through `FSEXT_*`, `FSINT_*`, `NotifyGetNextChange`, and `INT_NotifyGetNextChange`; the emulator should validate notification handle ownership, process ownership, duplicated handles, invalid handles, and close behavior.
- `FindFirstChangeNotificationW` should be validated through CE path canonicalization and mount-volume resolution semantics, not only simple rooted paths.
- `CeGetFileNotificationInfo` should be extended for buffer-size reporting, `lpBytesReturned`, `lpBytesAvailable`, empty queues, reset semantics, subtree watches, rename old/new pairs, and access-denied/error propagation.
- FSDMGR volume-handle paths should be tested for mounted storage, system volumes, read-only host roots, cross-volume renames, and watch-subtree boundaries.

### Loader And Runtime Gaps

Reference evidence:
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\TOOLS\ROMIMAGE\ROMIMAGE\pbtimebomb.cpp`
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\LOGGER\celoginstrumentation.c`
- `C:\WINCE600\PUBLIC\OSTEST\CESYSGEN\makefile`

Missing or under-validated:
- `LoadLibraryEx` should continue to validate CE resource-only combinations such as `LOAD_LIBRARY_AS_DATAFILE | DONT_RESOLVE_DLL_REFERENCES`.
- Loader lifecycle should be checked against CE build-entry behavior: `DllEntry`, `_DllMainCRTStartup`, C++ static constructors/destructors, TLS callbacks, and process/thread attach-detach ordering.
- Runtime DLL mapping parity should be audited outside Unicorn so raw loader tests and Unicorn execution do not silently diverge.
- Forwarder resolution should include malformed forwarder strings, ordinal forwarders, dependency unload, and failure cleanup.

### Winsock And Network Gaps

Reference evidence:
- `C:\WINCE600\PRIVATE\TEST\NET\WINSOCK\22`
- `C:\WINCE600\PRIVATE\SERVERS\WEBPROXY\CORE\session.cpp`
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\DEVICE\SERVICES\CORE\servacpt.cpp`

Missing or under-validated:
- Local Winsock already has core socket calls, `ioctlsocket`, `setsockopt`, `getsockopt`, and `select`; remaining work is CE stress parity: `FD_SETSIZE` limits, mixed read/write/except sets, repeated zero/error select recovery, peer close behavior, reset/error propagation, and long-running server loops.
- Add tests based on CE server patterns where `select` is called with `nfds` values of `0`, `-1`, and active socket counts.
- Validate UDP datagram source-address behavior, TCP half-close/reset semantics, and `SO_ERROR` clearing after `getsockopt`.

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
