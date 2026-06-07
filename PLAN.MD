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
| Guest DLL loader | `NK\KERNEL`, PE loader and SDK loader headers | startup preload path only | startup import DLL registration, CE-aware DLL search order, module metadata/refcounts, registered-export lookup, non-reclaiming `FreeLibrary` | runtime PE mapping from `LoadLibraryExW`, TLS, guest `DllMain` callouts | `dll_search`, `065_loadlibrary_getprocaddress_module`, raw module refcount tests | dumped DLLs from `D:\INAVI_Emulator\DUMPPLZ\Windows` are authoritative runtime bytes |
| COREDLL stub audit | COREDLL API sets and SDK headers | broad stub policy proved convenient but unsafe | static ordinal table, implemented/stubbed plan, fallback audit classification/logging, raw thread/caller/trap PC context | owning-module context and fail-loud audit mode | `raw_stub_audit_keeps_import_trap_context`, `tests\coredll_dispatch.rs` | shell/UI/process calls must not silently fake success |
| Shell launch/settings | `SHELL`, Explorer registry conventions | v2 shell path shape only | `SHGetSpecialFolderPath`, basic registry/shortcut-backed `ShellExecuteEx` launch chain | `SHGetFileInfo`, notify icons, recent docs, fuller namespace/policy | raw `ShellExecuteEx` association test | route UI still needs real shell/process/GWE semantics |
| GWE defaults | `GWE\INC`, `window.hpp`, CE `winuser.h`, MFC `wincore.cpp` | window broker behavior evidence | core window/message pump, partial defaults | fuller `DefWindowProcW`, `SendNotifyMessageW` edge cases | existing GWE fixtures | hidden chrome is guest-driven; do not force visibility |
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
  `safe no-op`, `safe failure`, or `must implement`.
- Implement `ShellExecuteEx` above `CreateProcessW`: normalize path, resolve CE
  shortcuts, query associations, expand command templates, locate executable,
  launch, apply show state, and return correct status.
- Expand registry-backed shell behavior for associations, icons, shell folders,
  startup items, active devices, storage profiles, app paths, recent docs, and
  UI policy/settings.
- Improve GWE defaults for focus, activation, non-client messages, hit testing,
  keyboard navigation, mouse capture, close/destroy, command routing, dialog and
  control notifications, and `SendNotifyMessageW` ordering.
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
