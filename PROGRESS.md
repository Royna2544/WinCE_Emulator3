# Progress

Regenerated on 2026-06-11 from the current implementation and test surface.

## Current Snapshot

- Runtime loader work has reached dynamic Unicorn DLL mapping with dependency loading, import patching, forwarders, trampoline tracking, datafile/no-resolve flags, and lifecycle calls.
- Shell icon work now includes `ExtractIconExW`, real PE resource icon extraction, shell fallback icons, `CreateIconIndirect`, `DrawIconEx`, image lists, bitmap-backed image-list drawing, `xBitmap` offsets, and `rgbBk` fill handling.
- File-change notifications now coalesce exact duplicates, transient create/delete pairs, and modified/delete sequences before signaling waiters.
- GWE message work includes cross-thread send setup, timeout marking, destroyed-window completion, and zero-result writes for destroyed `SendMessageTimeout` targets.
- Winsock has CE-facing dispatch for core socket operations with isolated NAT addressing, readiness checks, and scheduler wake candidate integration.
- Core CE subsystems remain broad and test-backed: handles, waits, events, TLS, critical sections, registry, files, memory, GDI resources, DIBs, windows, menus, clipboard, and scheduler selection.

## Recent Source-Visible Slices

- `src/ce/coredll.rs`: `ExtractIconExW` reads guest paths, validates files, extracts PE icon resources when available, falls back to shell icons for index zero, writes large/small icon outputs, and supports bitmap-backed icon rendering through `DrawIconEx`.
- `src/ce/kernel.rs`: file-change record append now coalesces pending records and signals only when pending notification data remains.
- `src/ce/gwe.rs` and `src/ce/coredll.rs`: destroyed-window handling exposes completed send-message result writes and flushes them to guest memory.
- `tests/coredll_raw_kernel.rs`: icon extraction, shell icon, and image-list drawing coverage is present.
- `tests/coredll_raw_memory_file.rs`: transient file-change notification churn coverage is present.
- `tests/coredll_raw_gwe.rs`: destroyed-target `SendMessageTimeout` result write coverage is present.

## Last Known Validation

- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe` passed for the GWE slice.
- `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file` passed for the file-change slice.
- `cargo check --features unicorn,trace,win32-desktop` passed after the recent code slices.
- `git diff --check` was clean except for expected CRLF warnings on existing files.

## Next Checkpoint

The next useful checkpoint is targeted validation after expanding shell icon/image-list edge coverage or completing the next `SendMessageTimeout` semantics slice.
