# SOURCE_REFERENCES

Bounded source references used to shape emulator behavior. These are evidence
anchors, not app-specific shortcuts.

Artifact note: `target\` was cleared on 2026-06-04 to recover disk space.
Source references below remain authoritative anchors; any mentioned
`target\...` artifact names are historical unless regenerated.

Runtime DLL note: mounted iNavi execution should load DLL images from
`D:\INAVI_Emulator\DUMPPLZ\Windows`. SDK import libraries and CE/MFC source
trees remain behavior/reference evidence, not the primary runtime DLL source.

## Windows CE Core OS

- Scheduler and wait ownership:
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\schedule.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\syncobj.c`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winbase.h`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\pkfuncs.h`
  - CE waits are kernel scheduler decisions over signaled kernel objects, not
    local ad hoc return stubs at each API boundary.
  - Event, mutex, semaphore, thread, process, timer, message, device, and audio
    wait paths should converge through one scheduler-owned wait/wake model.
  - The Rust `Scheduler` subsystem now owns compact wait accounting for
    `WaitForSingleObject`, `WaitForMultipleObjects`,
    `MsgWaitForMultipleObjectsEx`, and Unicorn blocked-wait resume diagnostics.
    Parked Unicorn `WaitForSingleObject` waits now preserve start tick/timeout
    metadata and use CE-style `WAIT_TIMEOUT` resume for bounded waits, with
    object-signaled acquisition taking precedence. `NKWaitForMultipleObjects`
    rejects `fWaitAll` before dispatching to the lower wait helper, so v3 keeps
    that raw API behavior even though v2 had corroborating wait-all machinery.
    Full waiter queues and context-switch ownership remain the next scheduler
    port step.

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
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/DEVICE/DEVCORE/devfile.c`
  - `DM_DevReadFile`, `DM_DevWriteFile`, and `DM_DevDeviceIoControl` show the
    device-file split beneath Win32 file handles.

- COREDLL file cursor/size helpers:
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/CORE/DLL/apis.c`
  and
  `/mnt/c/Program Files (x86)/Windows CE Tools/wce420/STANDARDSDK_420/Include/Mipsii/winbase.h`
  - CE copy-file paths use `GetFileSize`, `SetFilePointer(FILE_BEGIN)`,
    `ReadFile`, `WriteFile`, and `FlushFileBuffers`-style handle behavior.
  - SDK signatures define the low/high file-pointer and high-size output
    pointer shapes mirrored by the raw dispatcher.

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
  - Raw/kernel parent `DestroyWindow` now walks the virtual descendant tree and
    sends `WM_DESTROY` to descendants before the parent, then performs final
    GWE cleanup. Unicorn direct guest-WNDPROC destroy callouts use the same
    descendant-before-parent target order and delay final root cleanup until
    the last guest `WM_DESTROY` callback returns. A virtual lifecycle order
    counter exists only to verify this child-first sequence in focused
    fixtures.
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
    wakeable only when `MWMO_INPUTAVAILABLE` is set.
  - `cmsgque.h` defines `smfSenderNoWait`,
    `smfSenderNoWaitIfDifferentThread`, and `smfNotifyMessage` for no-wait
    notification sends. Rust raw `SendNotifyMessageW` now preserves that CE
    split at the syscall boundary: same-thread targets use synchronous send,
    while different-thread targets enter the receiver-side sent-message queue
    without sender blocking.
  - GWE now keeps a receiver-side sent-message queue distinct from posted
    messages and paint requests. Retrieval prefers sent messages, marks
    `InSendMessage`, exposes `QS_SENDMESSAGE`, and records a send source.
    Raw and Unicorn `DispatchMessageW` paths now clear the receiver send
    context after dispatch returns. `cmsgque.h`'s `SendMsgEntry_t` fields
    (`pReceivedNext`, `pSentNext`, `pmsgqReply`, `smFlags`, HWND/message
    parameters, and `WndProcResult`) now map to explicit Rust sent-message
    transaction state with sender/receiver thread ids, flags, timeout metadata,
    an active receiver send stack, result-ready completion, and
    receiver-terminated completion when a target is destroyed. `cmsgque.h`'s
    `MessageTimeout` comment and `smfTimeout` flag now map to GWE timeout
    expiry: non-result-ready sent transactions compare the current tick against
    the message timestamp plus timeout, set `SMF_TIMEOUT|SMF_RESULT_READY`,
    leave a zero result, and leave receiver retrieval. Unicorn raw
    `SendMessageW`/`SendMessageTimeoutW` now uses that transaction state for
    same-process cross-thread guest WNDPROCs: the receiver thread becomes the
    active CE thread for the guest WNDPROC callout, the sender MIPS context is
    restored after the WNDPROC result is captured, and the result flows back to
    the sender and optional timeout result pointer. Full scheduler-owned sender
    parking, reply wakeups, reentrant cross-thread scheduling, and destroyed-
    target edge behavior remain open.
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
    arguments. Rust raw ordinals 252 and 253 now route through the virtual GWE
    visible/enabled HWND hit-test path; `ChildWindowFromPoint` treats the input
    point as parent-client coordinates and uses the existing
    client-to-screen transform before searching child windows.

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
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\wingdi.h`
  - CE exposes `CreateDIBSection`, `GetDIBColorTable`, `SetDIBColorTable`,
    `RGBQUAD`, `BITMAPINFOHEADER`, `BITMAPINFO`, `DIB_RGB_COLORS`, and indexed
    DIB color-table conventions through the public GDI header.
  - Rust now stores RGBQUAD color-table entries on bitmap objects selected into
    memory DCs, routes raw `SetDIBColorTable`/`GetDIBColorTable` through guest
    memory, and uses the selected bitmap table when 8 bpp blits write RGB565
    framebuffer pixels. Rust direct-DIB sources now also parse BITMAPINFO-
    supplied RGBQUAD/RGBTRIPLE tables for 1/4/8 bpp `DIB_RGB_COLORS`, so
    indexed `StretchDIBits`/`SetDIBitsToDevice` sources use their embedded
    palette. Remaining indexed-DIB work is broader palette/device-color
    behavior as trace evidence reaches it.

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
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/CORE/DLL/CRT/corelib1.def`
  and
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/INC/crt_ordinals.h`
  - Define common CRT math exports such as `sqrt @1060`, `pow @1051`, and MIPS
    helpers such as `__ll_div @2005`.

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
    message above GWE. Guest child destroy-message ordering is now chained
    through Unicorn WNDPROC callouts before final root cleanup; remaining
    lifecycle work is focused on synchronous-send ownership and destroyed-target
    behavior.

## Prior Emulator Reference

- Remote server API shape:
  `../WinCE_Emulator_v2/src/remote_server.cpp` and
  `../WinCE_Emulator_v2/src/ce_remote.h`
  - Remote routes and WebSocket control messages accept touch, key, location,
    NMEA, IMU, pause, resume, status, logs, frame, MJPEG, and audio endpoints.
  - `CeRemote` stores queued touch/key events, serial bytes, audio chunks, IMU
    state, audio client counts, and paused state.
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
  `../WinCE_Emulator_v2/src/synthetic_dll.cpp`, and
  `../WinCE_Emulator_v2/src/coredll_fs.cpp`
  - v2 exposed `SDMMC Disk` as a CE virtual root and mapped the main module
    under `\SDMMC Disk\...` when the host image lived beneath that root.
  - Root-relative probes under the SDMMC backing were supported, but `\`
    itself represented the CE namespace and should enumerate mount-point
    prefixes such as `SDMMC Disk` rather than the host filesystem root.

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
