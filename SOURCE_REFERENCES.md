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
  - Host-presented probes of dumped `explorer.exe` using this same runtime DLL
    directory no longer fail on the old high-address trampoline from
    `0x00057108` to `0xffff832c`. After the COREDLL startup ordinal slice, the
    latest probe reaches the emulator sentinel (`pc=0x7ffffff0`) rather than a
    missing import trap.

## Windows CE Core OS

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
  - `SetWindowRgn(HWND, HRGN, BOOL)` consumes the region shape owned by GWE and
    only requests redraw when the third argument is nonzero. v3 now mirrors
    that boundary generically instead of invalidating every region change.

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
    `SHGetSpecialFolderPath`; v3 consults
    `HKLM\System\Explorer\Shell Folders` first and uses CE-shaped fallbacks
    when the dump lacks those values. `strsafe.h` anchors the `StringCch*`
    character-count and `StringCb*` byte-count distinction plus truncation
    HRESULTs. `winuser.h` anchors the icon/cursor signatures; v3 currently
    has lightweight synthetic stock icon handles and `DestroyIcon` validation
  rather than a full icon resource manager.

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
    signaled auto-reset object. `thread.c` defines `W32PrioMap` as CE
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
    Full COMMTIMEOUTS, `WaitCommEvent`, masks, purge/error state, host
    `win32_com`, and complete run-queue ownership remain open.

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
    leave a zero result, and leave receiver retrieval. Raw
    `SendMessageTimeout(..., timeout=0)` across threads now goes through the
    same sent-transaction path and expires immediately instead of running the
    receiver shortcut. The scheduler now has a send-reply blocked-wait kind
    keyed by sent-message id, mirroring the sender-side `pSentNext`/reply wait
    relationship: normal WNDPROC completion, timeout expiry, and receiver
    destruction enqueue send-reply wake candidates once the `WndProcResult`
    state is ready. Unicorn raw `SendMessageW`/`SendMessageTimeoutW` now uses
    that transaction state for same-process cross-thread guest WNDPROCs: the
    receiver thread becomes the active CE thread for the guest WNDPROC callout,
    the sender MIPS context is parked in a scheduler-backed `SendMessage`
    blocked wait, WNDPROC return and generic scheduler wake/resume restore that
    blocked record after the result is captured, and the result flows back to
    the sender and optional timeout result pointer. Reentrant cross-thread
    scheduling, `ReplyMessage` early release, and richer destroyed-target edge
    behavior remain open.
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
    the dialog's default pushbutton with `IDOK` fallback, and GWE now reports
    `DLGC_DEFPUSHBUTTON`/`DLGC_UNDEFPUSHBUTTON` plus
    `DM_GETDEFID`/`DM_SETDEFID` over child button style state. The same queued
    key state now backs the first raw `GetAsyncKeyState` and
    `GetAsyncShiftFlags` implementation.

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
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\wingdi.h`
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
