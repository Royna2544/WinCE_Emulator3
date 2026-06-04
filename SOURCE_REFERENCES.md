# SOURCE_REFERENCES

Bounded source references used to shape emulator behavior. These are evidence
anchors, not app-specific shortcuts.

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
  - Declares `UpdateWindow_I`; CE/MFC uses this as a synchronous paint forcing
    boundary. Rust raw `UpdateWindow` now validates pending update state by
    sending `WM_PAINT` through the window send path when an update region exists.
  - `CWindow` stores `m_rc` for the whole window and `m_rcClient` for the
    client area in screen coordinates; it declares `SetWindowPos_I`,
    `MoveWindow_I`, `GetWindowRect_I`, `GetClientRect_I`,
    `ClientToScreen_I`, and `ScreenToClient_I`.
  - Also declares `ShowWindow_I`, `UpdateWindow_I`, `GetParent_I`,
    `IsWindow_I`, `GetClassNameW_I`, and `EnableWindow_I`, which back the
    virtual HWND state, class/title text copying, visibility/enabled checks,
    parent lookup, and focus bookkeeping.
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

- GWE paint/update surface:
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\INC\gweapiset1.hpp`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\LIB\ARMV4I\RETAIL\coredll.def`, and
  `/mnt/c/Program Files (x86)/Windows CE Tools/wce420/STANDARDSDK_420/Include/Mipsii/winuser.h`
  - CE exposes `InvalidateRect`, `BeginPaint`, `EndPaint`, `UpdateWindow`,
    `GetUpdateRect`, `ValidateRect`, and `RedrawWindow` through the GWE API set
    and coredll ordinals `250`, `260`, `261`, `267`, `274`, `278`, and `286`.
  - The SDK `PAINTSTRUCT` layout is `hdc`, `fErase`, `rcPaint`, `fRestore`,
    `fIncUpdate`, and 32 reserved bytes; raw `BeginPaint` writes that shape.
  - `WM_PAINT` is `0x000F`; the virtual GWE subsystem generates it from a
    pending update region and clears the region through `BeginPaint` or
    `ValidateRect`.
  - Raw `RedrawWindow` now follows the same pending-paint model for rectangle
    and HRGN invalidation, update-region unioning, descendant invalidation, and
    `RDW_UPDATENOW`. Remaining gaps are tracked in `TODO.md`: partial validate
    subtraction, internal-paint-only requests, and full child clipping.

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
