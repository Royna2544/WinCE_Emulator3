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
