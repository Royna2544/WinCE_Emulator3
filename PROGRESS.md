# PROGRESS

## Confirmed

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
  - COREDLL, commctrl, winsock, and OLE import slots are patched to shim trap
    addresses when no loaded DLL export resolves them
  - MFC imports are not emulated by external stubs; they must resolve to loaded
    SDK DLL exports such as `mfcce400.dll`
  - external imports can resolve to loaded DLL exports before falling back to
    module-owned traps
  - COREDLL traps decode MIPS `a0`-`a3`, dispatch through the raw ordinal
    dispatcher, write `v0`, and retain a debug snapshot with PC/RA/SP/v0/v1/
    a0-a3/t9 plus memory-fault details on run failure
  - guest heap pages are mapped as a CE heap arena for APIs that allocate and
    populate memory during the same import call
  - non-COREDLL supported DLLs other than MFC currently use module-owned launch
    stubs with debug logs, not final API semantics
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
- Remote server socket/WebSocket binding is not implemented in Rust yet; the
  emulator-facing remote API state and dispatch behavior are present.
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

## False Leads

- A process-directory fallback for rooted `CreateFileW` paths was tested and
  removed before commit. Windows CE loader code does search the current EXE
  directory for DLL/module names, but ordinary FSDMGR file opens canonicalize
  the supplied path and resolve it through the mount table/root filesystem.

## Regressions

- None yet.
