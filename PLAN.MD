Reference C:\WINCE600 sources

# CE Guest DLL, Shell, and Winsock Fidelity Plan

## Summary

Use `C:\WINCE600` as the behavior reference and
`D:\INAVI_Emulator\DUMPPLZ\Windows` as the runtime DLL byte source. Implement
this as CE subsystem fidelity, not app-specific behavior: dynamic guest DLL
loading, explicit stub auditing, shell launch semantics, stronger GWE defaults,
and isolated guest networking.

## Implementation Ledger

| Subsystem | CE source reference | v2 corroboration | Implemented APIs | Open gaps | Fixture coverage | Latest iNavi evidence |
| --- | --- | --- | --- | --- | --- | --- |
| Guest DLL loader | `NK\KERNEL\loader.c`, `NK\KERNEL\thread.c`, PE loader and SDK loader headers | startup preload path only | startup import DLL registration, CE-aware DLL search order, live-mutable import trap table, static trap-base allocator with reserved dynamic range, live/persisted import trap-page refresh, runtime normal-code DLL map/relocate/import patching from `LoadLibraryW/LoadLibraryExW(flags=0)`, recursive dependency loading, runtime module export table ingestion, module metadata/refcounts, registered-export lookup, TLS callback metadata parsing and attach invocation, guest `DllMain(DLL_PROCESS_ATTACH)` callouts, final dynamic `FreeLibrary` guest `DllMain(DLL_PROCESS_DETACH)` callout before non-reclaiming unload-pending release | TLS detach callback ordering, forwarded exports, datafile/no-resolve loads, fuller runtime trampoline handling | `dll_search`, import-trap tests, `refresh_import_trap_page_blob_replaces_existing_blob`, `runtime_occupied_ranges_include_memory_regions_and_blobs`, `loaded_module_export_snapshots_skip_unload_pending_modules`, `external_table_accepts_runtime_loaded_module_exports`, `065_loadlibrary_getprocaddress_module`, `171_loadlibrary_guest_dll`, `172_loadlibrary_dependent_guest_dll`, `173_loadlibrary_tls_callback`, raw module refcount tests | dumped DLLs from `D:\INAVI_Emulator\DUMPPLZ\Windows` are authoritative runtime bytes |
| COREDLL stub audit | COREDLL API sets and SDK headers | broad stub policy proved convenient but unsafe | static ordinal table, implemented/stubbed plan, fallback audit classification/logging, raw thread/caller/trap PC context | owning-module context and fail-loud audit mode | `raw_stub_audit_keeps_import_trap_context`, `tests\coredll_dispatch.rs` | shell/UI/process calls must not silently fake success |
| Shell launch/settings | `SHELL`, Explorer registry conventions | v2 shell path shape only | `SHGetSpecialFolderPath`, basic registry/shortcut-backed `ShellExecuteEx` launch chain | `SHGetFileInfo`, notify icons, recent docs, fuller namespace/policy | raw `ShellExecuteEx` association test | route UI still needs real shell/process/GWE semantics |
| GWE defaults/input/send | `GWE\INC`, `window.hpp`, CE `winuser.h`, MFC `wincore.cpp`, CE input/clipboard/caret/IME headers | window broker behavior evidence | core window/message pump, partial defaults, minimal `TranslateMessage`, partial cross-thread send | fuller `DefWindowProcW`, `SendNotifyMessageW` edge cases, keyboard modifiers/layout, accelerators/menu shortcuts, `WM_SYSKEY*`, dead keys/IME handoff, clipboard ownership/formats, caret blink/position/focus integration, full `SendMessageTimeout` wait/result/flag/destruction/reentrancy semantics | existing GWE fixtures | hidden chrome is guest-driven; do not force visibility |
| Winsock | CE Winsock headers and socket wait behavior | none | direct host socket traps, CE-facing isolated local names/IPs | scheduler-backed blocking waits | winsock startup/loopback/isolation tests | host transport works, subnet identity is now CE-facing |

## Key Changes

- Add a real guest module manager for dynamically loaded MIPS DLLs. Track
  normalized module name, guest path, host path, base, size, exports by
  name/ordinal, dependencies, refcount, TLS callbacks, entrypoint, flags, and
  unload state.
- Extend PE runtime loading so guest DLLs are mapped, relocated, patched for
  COREDLL traps, resolved against other guest DLLs, and reused by normalized
  module identity.
- Replace broad COREDLL fallback behavior with audited explicit outcomes:
  `safe no-op`, `safe failure`, or `must implement`; every app-hit
  must-implement stub needs actionable callsite diagnostics and shell/UI paths
  should prefer explicit unsupported failure over plausible fake success.
- Implement `ShellExecuteEx` above `CreateProcessW`: normalize path, resolve CE
  shortcuts, query associations, expand command templates, locate executable,
  launch, apply show state, and return correct status. Finish the remaining
  fields/flags, working-directory behavior, precise shell errors, richer `.lnk`
  parsing, non-EXE document edge cases, and command-template substitutions.
- Expand registry-backed shell behavior for associations, icons, shell folders,
  startup items, active devices, storage profiles, app paths, recent docs, and
  UI policy/settings.
- Implement shell-visible UI APIs that currently still succeed too cheaply:
  `SHGetFileInfo`, `TrackPopupMenu*`, modal `MessageBoxW`, notification icons
  and notification data, shell namespace/storage presentation, file type
  metadata, and shell/file-change notifications.
- Improve GWE defaults for focus, activation, non-client messages, hit testing,
  keyboard navigation, mouse capture, close/destroy, command routing, dialog and
  control notifications, and `SendNotifyMessageW` ordering.
- Expand input and UI state fidelity beyond the current minimal
  `TranslateMessage`: keyboard modifiers/layout, accelerators and menu
  shortcuts, `WM_SYSKEY*`, dead-key/IME handoff, clipboard
  formats/ownership, caret creation/blink/position, and focus/capture
  integration.
- Complete cross-thread `SendMessageTimeout` semantics: wait for reply or
  timeout, write the result pointer on completion, honor `SMTO_*` flags, handle
  receiver/sender destruction, and perform required reentrant dispatch while
  waiting.
- Replace direct-only WINSOCK with a CE-facing isolated NAT model: gateway
  `10.0.0.1`, guest IP `10.0.0.2`, host sockets as the transport boundary, and
  blocking waits owned by the scheduler.

## Validation

- Add eVC fixtures under `tests\test_progs` for guest DLL loading, refcounts,
  exports, shell execution, shortcuts, window defaults, notifications, and
  Winsock waits.
- Add Rust tests for module normalization/refcounts, export lookup, shell
  registry parsing, stub audit classification, and isolated Winsock address
  translation.
- Recurring checks: `cargo fmt`, `cargo check --features unicorn,trace,win32-desktop`,
  `cargo test --features unicorn,trace,win32-desktop`, and a mounted iNavi run
  with `--remote-server 192.168.0.39:8765`.

## Queued Loader Gaps

These are not implemented yet and must remain queued until closed by focused
fixtures plus mounted iNavi evidence:

- Extend runtime guest-DLL fixtures as more lifecycle behavior lands.
  `tests\test_progs\171_loadlibrary_guest_dll` now proves the
  direct export-call path with one DLL containing `DllMain`, one named export,
  one ordinal export, and one EXE that calls `LoadLibraryW`, `GetProcAddress`,
  invokes both exports, asserts a single `DllMain(DLL_PROCESS_ATTACH)` call
  across handle reuse, verifies no detach on non-final `FreeLibrary`, and
  proves final `FreeLibrary` runs `DllMain(DLL_PROCESS_DETACH)` by writing a
  DLL detach marker into EXE-owned memory.
- Align non-Unicorn/raw `LoadLibraryW` / `LoadLibraryExW` behavior with the
  runtime path where practical. The raw helper currently returns COREDLL or
  already registered modules only; when it cannot map a new DLL, it should fail
  explicitly and consistently rather than diverging silently from the Unicorn
  syscall path.
- Keep the core-library vs guest-DLL boundary audited. Emulator-provided API
  modules must be intentionally classified; non-core libraries should load as
  guest PE DLLs or fail loudly; unresolved/unclassified imports need compact
  diagnostics so user MIPS libraries are not accidentally translated as host
  stubs.
- Extend the dependent-DLL runtime fixture as more lifecycle behavior lands.
  `tests\test_progs\172_loadlibrary_dependent_guest_dll` now proves DLL A
  importing from DLL B through the PE import table, recursive runtime
  dependency loading, guest-DLL-to-guest-DLL import resolution through the
  module manager, and one `DllMain(DLL_PROCESS_ATTACH)` call for both the
  dependency and dependent DLL.
- Confirm and implement TLS detach callback ordering on final `FreeLibrary`;
  guest `DllMain(..., DLL_PROCESS_DETACH, 0)` is now invoked before the module
  is marked unload-pending, and mapped address ranges remain reserved.
- Implement forwarded export resolution for `GetProcAddress` and import
  patching. Forwarders must resolve through already loaded modules or trigger
  CE search/load of the forwarded-to DLL; unresolved forwarders must fail
  loudly, not return a plausible fake pointer.
- Implement `DONT_RESOLVE_DLL_REFERENCES` semantics: map/reuse the module and
  expose exports without recursive import resolution, TLS callbacks, or
  `DllMain`, and clearly prevent execution paths that require unresolved IATs.
- Implement datafile/resource-style `LoadLibraryExW` modes for resource access
  without normal code execution, including `FindResourceW`/`LoadResource`
  behavior from the loaded resource module.
- Harden runtime trampoline handling for DLLs that relocate above ordinary
  MIPS jump reach, matching the startup trampoline machinery without
  corrupting guest code or reusing stale trampoline ranges.
- Add compact loader audit counters/log summary: runtime load attempts,
  successful maps, dependency loads, export lookups, forwarded exports, TLS
  callbacks, `DllMain` attach/detach calls, and loud loader failures.

## Queued Shell And UI Gaps

- Make must-implement stub hits test/actionable: ordinal, name, caller PC,
  owning module where available, thread id, and explicit unsupported result for
  shell/UI/process/loader paths.
- Implement `SHGetFileInfo`: display name, type name, attributes, icon index or
  icon handle behavior, folder/executable/file-extension icon mapping,
  observed `SHGFI_*` flags, and registry-backed fallback file types.
- Implement real popup menu behavior for `TrackPopupMenu*`: menu popup or
  equivalent UI, modal menu loop, pointer/key hit-testing, selection/highlight,
  disabled/checked/default item rendering, submenu behavior, `WM_COMMAND`
  routing, `TPM_RETURNCMD`, and cancellation.
- Implement real modal `MessageBoxW`: create/show modal dialog, button layout
  from flags, icon/title/text behavior, owner disable/reenable, nested message
  pump, Enter/Escape/default-button routing, and correct `ID*` return values.
- Finish shell special-folder behavior: log or classify fallback use, add a
  strict-vs-compat fallback policy, validate buffer and `fCreate` semantics,
  create missing folders when CE expects it, and confirm CSIDL coverage against
  observed calls.
- Implement notification APIs: `Shell_NotifyIcon`,
  `SHNotificationAddI`/`UpdateI`/`RemoveI`/`GetDataI`, callback message
  delivery, tooltip/title/icon state, process/window cleanup, and optional
  shell/taskbar rendering only when backed by real shell state.
- Finish file namespace and file-change notifications: shell namespace roots
  over mounts, storage-volume presentation, file type/display/icon metadata,
  change notification handles, shell change notifications, create/delete/
  rename/update refresh events, and storage insertion/removal notifications.

- User todos:
Create a basic cmd.exe like desktop Windows. under host_progs/cmd.cpp, Make sure it is compilable with eVC4 and compilable.
