# TODO

Refreshed on 2026-07-04. This file is limited to active, plan-aligned work.

## Immediate Engineering Queue

- Continue the live launch loop past `resmapi_800x480.bin` map-skin decode
  with real visible-message delivery (post `2fcc365f`/`dbaf4791`/`265e751a`):
  confirm the map screen presents, then re-validate `COM7:`/`MFS1:`/`SMB1:`
  opens, GPS drain, and touch-driven map navigation on this baseline.
- Scope the per-instruction code hook (the boot-speed lever, ~25 min skin
  decode today). Settled design:
  1. Move wall-clock stop checks, framebuffer tick, and remote drain into
     the scheduler timeslice BLOCK hook (it already captures
     `host_wall_clock_stop_timeslice_hook`, limits, and kernel); gate by the
     existing block counter. Block-boundary stops are delay-slot safe by TB
     construction — only defer when the block-start pc lies inside a
     trampoline range or the import-trap page.
  2. Install the code hook per-range instead of globally: trampoline ranges
     + import-trap page + 4 KB pages containing jr/jalr/sentinel sites
     (reuse `fast_start_indirect_transfer_sites`/`fast_start_hook_ranges`).
     Cache the per-blob site scan on `UnicornMips` keyed by blob name+len —
     scanning 43 MB every slice would cost tens of ms.
  3. Runtime DLL loads mid-slice: instead of adding hooks from inside the
     import hook, set the host wall-clock stop pending flag after a
     successful `runtime_load_guest_dll_from_path`; the slice ends at a safe
     pc and the next slice's fresh engine scans and hooks the new module's
     pages. This is the fix for the historical `WINCE_EMU_FAST_START`
     mfcce400 stall (its site enumeration never covered runtime loads).
  4. Gate the scoped mode behind `WINCE_EMU_SCOPED_HOOKS=1` for A/B until a
     2-minute single-shot boot proves the decode runs several times faster
     and the live loop stays correct, then flip the default and remove the
     gate.
- Investigate the 2026-07-04 silent desktop-run exit at 13:22:22Z (log ended
  mid-operation, no WER record; stderr was not captured for that run — the
  relaunched run captures both streams).
- Identify the completion condition iNavi's command-flag loop
  (`iNavi.exe+0x2f7298..0x2f8338`, slot flags 0x4000/0x2000/0x100/0x80)
  waits on before switching splash to map: sample the flag word writers,
  correlate with `happyway_win`'s ~1 s retry cycle (self-posted msg 0x405,
  import called with 0x3e8 — likely Sleep/timer), and check whether the
  retry is waiting on a device/file/socket the emulator leaves unavailable.
- Watch the SD image for re-created zero-byte probe artifacts during longer
  live runs (route/traffic phases); if any reappear, find and fix the
  emulator path that materializes files on read-only existence probes.
- Continue live driving after the quantum-rotation handoff fix and confirm
  whether `COM7:`, `MFS1:`, and `SMB1:` open again before the final map
  presentation gate; queued GPS still cannot drain until the guest opens
  `COM7:`.
- The hot guest PCs around `0x00339c00-0x00339da8` are confirmed pixel-format
  conversion work (RGB565/555 + alpha planes), not a readiness wait. If boot
  needs to go faster, the next perf lever is region-scoped code hooks
  (trampoline ranges + import-trap region only) so QEMU can build multi-
  instruction translation blocks for such loops; keep trampoline dispatch
  load-bearing per the fast-start lesson.
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
- Continue live device traces after `COM7:`, `MFS1:`, and `SMB1:` are open.
  GPS now drains and sensor IOCTLs are visible; keep validating parsed-position
  consumption and the remaining presentation gate with real traces.
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
