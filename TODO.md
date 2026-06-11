# TODO

Regenerated on 2026-06-11 from current source and test coverage.

## Immediate Engineering Queue

- Add PE icon extraction edge tests for remaining PE resource variants and non-PE fallback paths.
- Expand `DrawIconEx`, `ImageList_DrawEx`, and `ImageList_DrawIndirect` tests for remaining mask/blend/overlay handling, stretched draws, framebuffer output, and invalid object lifetimes.
- Implement and test remaining `SendMessageTimeout` semantics: nested cross-thread sends, reentrant waits, `ReplyMessage`, abort-if-hung edge cases, and multi-wait interactions.
- Verify `Shell_NotifyIcon` and `SHNotification*` in an integrated Unicorn run, including COM callback dispatch, update/remove lifetimes, and invalid-handle behavior.
- Extend file-change notification coverage across mount roots, rename sequences, and FSDMGR-style volume handle ownership.
- Audit runtime loader behavior outside Unicorn so raw tests and runtime mapping behavior stay intentionally aligned.
- Continue iNavi route-flow work from the current process/window/shell readiness point into destination search and map interaction.

## Cleanup Queue

- Remove or use `IOCTL_NANDUUID_MICOM_RESET_STAGE` if it remains intentionally unused.
- Decide whether no-feature `cargo test` is a supported build profile. If yes, audit feature-gated Unicorn references and add it to validation.
- Keep `SOURCE_REFERENCES.md` tied to actual source behavior when implementing new slices.
- Avoid reintroducing long historical progress logs into roadmap files; use these files for current state only.

## Validation Queue

- For shell and GDI work: run `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel`.
- For GWE/message work: run `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe`.
- For file/storage work: run `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`.
- For shared scheduler/resource work: run `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems`.
- Before handing off a larger slice: run `cargo check --features unicorn,trace,win32-desktop` and `git diff --check`.
