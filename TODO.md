# TODO

Refreshed on 2026-06-20. This file is limited to active, plan-aligned work.

## Immediate Engineering Queue

- Trace the current iNavi splash/map transition generically through the
  post-mapdata startup path: the latest fresh run loads mapdata/SearchDB/mapinfo
  but still does not open `COM7:`, so identify the CE wait, event, message,
  window state, file result, or readiness condition gating serial startup.
- Trace why the real splash window keeps receiving repeated touch/key WndProc
  callouts without transitioning to the map UI after iNavi resumes from the
  hidden helper-process slice.
- Disassemble/sample the current hot guest PCs around `0x0030f54c`,
  `0x00339c8c`/`0x00339d90`, and later mapdata loaders against the real
  `iNavi.exe` to separate normal resource/image work from a repeated readiness
  wait.
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
- Replace the synthetic `GetCallStackSnapshot` frame with real saved-context
  unwinding when guest diagnostics require deeper stack fidelity.
- Decode the SMB380/G-sensor initialization contract from actual dump code or
  real caller traces before adding accelerometer command behavior.
- Continue live device traces until the app opens `COM7:` in the current
  startup path. Remote GPS is queued but not drained in the latest sample;
  keep watching for later `MFS1:`/`SMB1:` opens and validate their dumped-DLL
  contracts in live traces.
- Re-run the live startup after the device-information ordinal fix and confirm
  whether guest device probing now reaches `COM7:` earlier or still remains
  blocked behind the splash/readiness path.
- Re-run the live startup after the device-search ordinal fix and confirm
  whether CE Device Manager enumeration advances any GPS/storage probing paths.

## Cleanup Queue

- Keep `PROGRESS.md`, `TODO.md`, and `KNOWN_BUGS.md` short and current.
- Do not reintroduce historical ledgers into roadmap files.
- Keep `SOURCE_REFERENCES.md` as a compact index of active source evidence, not
  a running implementation diary.
- Keep registry fixtures and launch docs on `registry.reg`; temporary JSON
  parser regressions should use temporary fixtures only.

## Validation Queue

- Stub/raw smoke: `cargo test -j 1`
- Shell/resource/GDI: `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel`
- GWE/message/menu/modal: `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
- File/storage: `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`
- Shared scheduler/runtime: `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems`
- Larger handoff: `cargo check --features unicorn,trace,win32-desktop` and
  `git diff --check`
