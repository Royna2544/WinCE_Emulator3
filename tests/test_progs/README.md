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
| `011_rect_math` | RECT helpers: `SetRect`, `CopyRect`, `EqualRect`, `OffsetRect`, `IntersectRect`, empty rects. |
| `012_window_geometry` | Window lifetime, visibility, `GetWindowRect`, `GetClientRect`, `MoveWindow`, `SetWindowPos`. |
| `013_parent_child` | Parent/child ownership, child IDs, `GetParent`, `SetParent`, `GetWindow(GW_CHILD)`. |
| `014_z_order` | Top-level Z-order traversal and `SetWindowPos(HWND_TOP/HWND_BOTTOM)`. |
| `015_message_queue` | `PostMessageW`, `PeekMessageW`, `GetMessageW`, filtered message preservation/removal. |
| `016_send_message` | Synchronous `SendMessageW`, guest WNDPROC return value, and `GWL_USERDATA`. |
| `017_timer` | `SetTimer`, queued `WM_TIMER`, `KillTimer`, and sleep/tick interaction. |
| `018_focus_enable` | `EnableWindow`, `IsWindowEnabled`, `SetFocus`, and `GetFocus`. |
| `019_coordinate_map` | `ClientToScreen`, `ScreenToClient`, and `MapWindowPoints` across parent/child HWNDs. |
| `020_subsystem_smoke` | System/memory/heap/local/virtual allocation plus registry open/query/close. |
| `asm/` | Optional assembly/reference snippets for executable-section branch/delay tests. |
| `tools/make_raw_mips_blobs.py` | Generates raw MIPS little-endian blobs for CPU-only unit tests. |

## Return convention

Each fixture returns `0` on success unless noted. Nonzero return codes identify the failed behavior.
The emulator test harness should capture the process/entry return value or `ExitThread` code.

## Important policy

Do not make the emulator runtime depend on eVC4 or these sources.
Fixture EXEs are built automatically by the ignored `fixture_exes` integration test when the eVC4 toolchain is configured. Do not commit built EXEs; outputs belong under `target/wince-fixtures/mipsii/`.
