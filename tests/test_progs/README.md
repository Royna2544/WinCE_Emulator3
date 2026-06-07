# WinCE MIPSII PE Fixture Sources

Small Windows CE / eVC4 MIPSII fixture sources for testing a user-mode WinCE/MIPS emulator.

These are intentionally tiny. Each fixture should test one layer or invariant before launching the real iNavi app.

## Intended target

- eMbedded Visual C++ 4.0
- Windows CE SDK with MIPSII target
- Console is not required; these are `WinMain` GUI-subsystem style fixtures.
- Build as MIPSII little-endian PE32.

## Fixture list

| Fixture | Purpose |
|---|---|
| `001_exit` | Minimal PE startup/exit return value. |
| `002_gettickcount` | Basic coredll imports: `GetTickCount`, `Sleep`, `QueryPerformanceCounter`. |
| `003_tls` | TLS isolation across main/worker guest threads. |
| `004_delay_slot_dynamic` | Runtime-generated MIPS code proving ordinary branch delay slot executes. |
| `005_branch_likely_dynamic` | Runtime-generated MIPS code proving branch-likely taken/not-taken annul behavior. |
| `006_setjmp_longjmp` | CRT `_setjmp` / `longjmp` non-local control flow. |
| `007_thread_event` | `CreateThread`, `CreateEvent`, `SetEvent`, `WaitForSingleObject`. |
| `008_file_io` | `CreateFileW`, `WriteFile`, `ReadFile`, `SetFilePointer`, `CloseHandle`. |
| `009_gdi_paint` | Basic GWE/GDI paint path: window, invalidate, `BeginPaint`, `FillRect`, `Ellipse`. |
| `010_resource_string` | PE resources: `LoadStringW`, `.rc` string table. |
| `011_api_storm` | Broad API storm across time, memory, strings, file, registry, thread/TLS, GWE, GDI, and messages. |
| `012_window_geometry` | Window lifetime, visibility, `GetWindowRect`, `GetClientRect`, `MoveWindow`, `SetWindowPos`. |
| `013_parent_child` | Parent/child ownership, child IDs, `GetParent`, `SetParent`, `GetWindow(GW_CHILD)`. |
| `014_z_order` | Top-level Z-order traversal and `SetWindowPos(HWND_TOP/HWND_BOTTOM)`. |
| `015_message_queue` | `PostMessageW`, `PeekMessageW`, `GetMessageW`, filtered message preservation/removal. |
| `016_send_message` | Synchronous `SendMessageW`, guest WNDPROC return value, and `GWL_USERDATA`. |
| `017_timer` | `SetTimer`, queued `WM_TIMER`, `KillTimer`, and sleep/tick interaction. |
| `018_focus_enable` | `EnableWindow`, `IsWindowEnabled`, `SetFocus`, and `GetFocus`. |
| `019_coordinate_map` | `ClientToScreen`, `ScreenToClient`, and `MapWindowPoints` across parent/child HWNDs. |
| `020_subsystem_smoke` | System/memory/heap/local/virtual allocation plus registry open/query/close. |
| `021_rect_math` | RECT helpers: `SetRect`, `CopyRect`, `EqualRect`, `OffsetRect`, `IntersectRect`, empty rects. |
| `164_object_transition_wake` | Event, semaphore, and mutex transitions wake blocked worker threads. |
| `165_thread_exit_wait_wake` | Thread handle exit signaling wakes waits and remains signaled for wait-any. |
| `166_msgwait_message_timer_wake` | Posted messages and timers wake `MsgWaitForMultipleObjectsEx`. |
| `167_sleep_infinite_resume` | Worker `Sleep(INFINITE)` self-suspends until `ResumeThread` makes it runnable again. |
| `168_sendmessage_timeout_zero_cross_thread` | `SendMessageTimeout(..., timeout=0)` to another thread expires without stale receiver delivery. |
| `169_post_keybd_message` | `PostKeybdMessage` posts hardware-sourced keydown/char/keyup messages and updates key state. |
| `171_loadlibrary_guest_dll` | Runtime `LoadLibraryW`, `GetProcAddress`, guest export calls, refcount reuse, process attach, and final process detach. |
| `172_loadlibrary_dependent_guest_dll` | Runtime recursive guest-DLL dependency loading, import patching, and dependency-order process attach. |
| `173_loadlibrary_tls_callback` | Runtime PE TLS callback invocation before DLL process attach and final detach. |
| `asm/` | Optional assembly/reference snippets for executable-section branch/delay tests. |
| `tools/make_raw_mips_blobs.py` | Generates raw MIPS little-endian blobs for CPU-only unit tests. |

## Return convention

Each fixture returns `0` on success unless noted. Nonzero return codes identify the failed behavior.
The emulator test harness should capture the process/entry return value or `ExitThread` code.

## Important policy

Do not make the emulator runtime depend on eVC4 or these sources.
Fixture EXEs are built automatically by the ignored `fixture_exes` integration test when the eVC4 toolchain is configured. Do not commit built EXEs; outputs belong under `target/wince-fixtures/mipsii/`.
