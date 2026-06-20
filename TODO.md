# TODO

Refreshed on 2026-06-20. This file is limited to active, plan-aligned work.

## Immediate Engineering Queue

- Trace the current iNavi splash/map transition generically: identify the CE
  wait, event, message, window state, or readiness condition that should hide,
  destroy, or demote the owned splash popup above hidden map children.
- Extend `SendMessageTimeout` and message-wait coverage for broader reentrant
  waits, nested sends, timeout cleanup, `ReplyMessage`, abort-if-hung behavior,
  and live modal-loop interaction.
- Validate `Shell_NotifyIcon` and `SHNotification*` in integrated runtime runs,
  including taskbar-visible state, callback dispatch through real guest COM
  object lifetimes, stale handles, and remove/update races.
- Keep mounted-storage work focused on physical block-driver forwarding,
  cache/filter DLL behavior, FMD callbacks, utility DLL exports, mounted-volume
  availability, file-security ACL persistence, and lower-FSD forwarding.
- Continue GDI/text/IME parity where CE source or raw tests justify it:
  palette/device-color behavior, alpha/mask variants, clipping, font fallback,
  IME UI callbacks, caret timing, and remaining DIB/bitmap lifetime edges.

## Next

- Add raw/non-Unicorn loader fixture variants for loader paths where Unicorn
  runtime coverage is currently stronger.
- Expand PE icon/resource tests for uncommon formats, malformed resource tables,
  callback-selected `KernExtractIcons`, non-PE fallback edges, and cleanup
  lifetimes.
- Continue file-change notification reset/error propagation and mounted rename
  edge coverage where current behavior is still only partially proven.
- Prove device-notification consumer ordering and wake cadence in live startup
  scenarios.
- Verify remaining CE toolhelp enumeration walkers and edge flags against dump
  callers before treating process, module, thread, and heap layouts as fully
  complete.
- Replace the synthetic `GetCallStackSnapshot` frame with real saved-context
  unwinding when guest diagnostics require deeper stack fidelity.
- Decode the SMB380/G-sensor initialization contract from actual dump code or
  real caller traces before adding accelerometer command behavior.

## Cleanup Queue

- Keep `PROGRESS.md`, `TODO.md`, and `KNOWN_BUGS.md` short and current.
- Do not reintroduce historical ledgers into roadmap files.
- Keep `SOURCE_REFERENCES.md` as a compact index of active source evidence, not
  a running implementation diary.
- Keep registry fixtures and launch docs on `registry.reg`; temporary JSON
  parser regressions should use temporary fixtures only.
- Decide whether no-feature `cargo test` is a supported validation profile. If
  yes, audit feature-gated Unicorn references and add it to the validation
  queue.

## Validation Queue

- Shell/resource/GDI: `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel`
- GWE/message/menu/modal: `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
- File/storage: `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`
- Shared scheduler/runtime: `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems`
- Larger handoff: `cargo check --features unicorn,trace,win32-desktop` and
  `git diff --check`
