# Progress

Refreshed on 2026-06-20. This file records confirmed current facts only.
Older investigation ledgers were purged; use commits, logs, and targeted source
references for historical reconstruction.

## Current Snapshot

- Loader/runtime mapping reaches dynamic Unicorn DLL mapping with dependency
  loading, import patching, forwarders, rollback on failed loads, trampoline
  tracking, datafile/no-resolve handling, and CE-style attach/detach lifecycle
  behavior.
- Shell/resource behavior includes PE icon extraction, shell fallback icons,
  image lists, `DrawIconEx`, `SHGetFileInfo`, version-resource extraction, and
  several CE-compatible image-list validation and lifetime paths.
- File and storage behavior includes mounted host paths, volume-boundary checks,
  file attributes, find handles, change notifications, `CeGetFileNotificationInfo`,
  FSDMGR import shims, public AFS mount registration, synthetic disk metadata,
  sparse-sector backing, StoreMgr partition format no-ops, and selected
  FMD/file-lock/security descriptor surfaces.
- GWE/message behavior includes HWND/message-queue state, menu and message-box
  foundations, cross-thread send setup, timeout completion, nested reply paths,
  and scheduler wake integration for multiple wait classes.
- GDI/text/IME behavior covers many raw CE edge cases for DIBs, bitmaps,
  palettes, draw APIs, alpha/mask paths, selected font metrics, IME state, and
  TESTIME-backed dictionary behavior.
- CE `CreateDIBSection(DIB_PAL_COLORS)` now follows the GDIAPI helper's 8 bpp
  rule: 8 bpp DIBSections keep mutable color tables, while 1/2/4 bpp and
  truecolor requests fail during creation.
- Remote runtime behavior exposes status, frame capture, touch/key input,
  GPS/NMEA serial injection, debug handle snapshots, and audio/control WebSocket
  plumbing.
- Process-handle behavior includes opening launched process IDs as distinct
  handles plus raw `ReadProcessMemory`/`WriteProcessMemory` byte copies with
  CE-style handle validation and transferred-byte counts.
- Raw stack diagnostic behavior includes a minimal `GetCallStackSnapshot`
  surface for normal and extended frame buffers, skip handling, flag validation,
  and CE-style last-error reporting.
- Stub/raw no-feature validation now keeps visible receiver work aligned with
  the Unicorn runtime, including sent messages, queued visible messages, and
  dirty visible windows.
- Toolhelp process snapshots now expose the current process and launched child
  process entries through raw `THCreateSnapshot` memory blocks.
- Toolhelp thread snapshots now expose current and launched thread entries,
  owner process IDs, priorities, and state flags through raw `THCreateSnapshot`
  memory blocks.
- Toolhelp module snapshots now expose live loaded-module entries, image bases,
  sizes, refcounts, names, and guest paths through raw `THCreateSnapshot`
  memory blocks.
- Toolhelp heap snapshots now append live process heap lists and fixed heap
  allocation entries through raw `GetHeapSnapshot` memory blocks.
- Public `toolhelp.dll` imports now create/close snapshots, walk process,
  thread, module, heap-list, and heap-entry records, and read current-process
  memory through CE-shaped snapshot buffers.

## Latest Confirmed State

- The former no-hook crash at handle-like `pc=0x00001054` no longer reproduces
  after blocked scheduler contexts were removed from direct receiver-work
  eligibility. Debug handle output identifies `0x1054` as an event handle.
- Host wall-clock slice stops now purge stale blocked-wait and parked
  `GetMessage` records for the thread that just executed, avoiding impossible
  active-and-blocked thread snapshots at `THREAD_EXIT_STUB_ADDR`.
- Host live-pump idle polling no longer forces short 100 ms CPU slices while an
  active guest context is runnable.
- Current live runs reach real iNavi splash/resource state, hidden map child
  windows, map/resource file loading, `UID1:` probing, and eventually `COM7:`
  GPS startup. The 2026-06-21 post-drive sample opened `COM7:`, drained queued
  remote NMEA into the serial RX buffer, and accepted the guest `$PUBX`
  configuration writes.
- Remote taps are accepted by the live server and now route through the main
  iNavi splash/window stack after stale visible-message cleanup.
- Escaped visible-message WndProc callouts are now retired before the
  visible-dispatch gate decides whether pending callbacks block new input. The
  live `happyway_win` stale `OrphanedVisibleMessage` case now clears, leaving
  `pending wndproc returns: 0`.
- Current live taps route to the main iNavi splash/window stack with accepted
  remote key/touch posts; the framebuffer still shows the real iNavi splash
  artwork, so the remaining issue is after message dispatch has resumed.
- Receiver/window teardown now removes terminated send ids from active send
  stacks. The latest live `happyway_win` receiver-terminated send still leaves
  a completed result for the sender, but `active_stacks=0` and
  `depth_threads=0`.
- Current 2026-06-21 live samples show the splash phase persists after
  touch/key input is accepted and routed; the app continues resource/window
  setup (`resmapi_800x480.bin` memory-backed opens and hidden `afxwnd42u` child
  creation).
- Remote/live scheduling now rotates away from an active process with no
  running guest thread and no queued visible receiver work when another parked
  process is runnable. The latest sample advances iNavi through additional
  map/resource loads and accepts touch/key WndProc callouts on the real splash
  window instead of sitting only in the hidden `happyway_win` helper slice.
- Live-pump run entry now also prioritizes ready parked waits before starting a
  hidden active slice. This matches the after-slice policy and covers the
  observed state where hidden `happyway_win` had no receiver work while a
  visible iNavi thread was parked on a ready sleep wait.
- `YAS526B.dll` dump validation confirmed `MFS_IOControl` dispatches exact
  IOCTLs `0xb0000000`, `0xb0000004`, `0xb0000008`, `0xb000000c`, and
  `0xb0000010`. The virtual `MFS1:` shim now follows the dumped contract:
  writes validate/send selector-command input but do not mutate the next sample,
  and reads use input byte 0 as device selector plus byte 2 as count.
- Elevated `cargo flamegraph` startup samples still stop in the real
  `happyway_win.exe+0x7b56c` traffic/shared-memory fill loop; the live-pump
  visible-window readiness check no longer clones the full GWE window list on
  that hot path.
- `MultiByteToWideChar`/`WideCharToMultiByte` now skip host Windows conversion
  calls for zero-flag ASCII input under Korean ACP/UTF-8, preserving explicit
  length and null-terminated return counts.
- Raw `GetDeviceInformationByDeviceHandle` and
  `GetDeviceInformationByFileHandle` now resolve through coredll ordinals and
  write the CE `DEVMGR_DEVICE_INFORMATION` layout for live device handles,
  including legacy name, registry driver key, and `$device\...` name.
- Raw `FindFirstDevice` and `FindNextDevice` now resolve through coredll
  ordinals and enumerate live activated device handles with CE-style device
  search handles, including legacy-name, device-name, bus-name, parent, no
  match, and stale-handle behavior.
- Activated configured devices now capture registry `BusName`, expose it in
  CE `DEVMGR_DEVICE_INFORMATION` as `$bus\...`, and use that activation-time
  value for `DeviceSearchByBusName` wildcard matching.

## Recent Validation

- Targeted formatting and tests passed for the blocked-thread receiver-work
  slice, stale active-wait cleanup, host idle-slice guard, and related focused
  scheduler regressions.
- Release/live validation survived the prior crash window, served framebuffer
  snapshots, consumed remote touch/key input on the iNavi splash window, and
  confirmed remote GPS drains once the guest opens `COM7:`.
- Focused raw kernel ordinal validation covers duplicated process handles,
  shared process exit state, and process memory read/write copies.
- Focused raw kernel ordinal validation covers normal and extended
  `GetCallStackSnapshot` buffers plus skip and invalid-flag behavior.
- Focused raw kernel ordinal validation covers `THCreateSnapshot` success and
  invalid requested-process failure, thread-entry serialization, module-entry
  serialization, and `GetHeapSnapshot` heap-list and entry serialization.
- Focused raw kernel conversion validation covers Korean ACP conversion plus
  ASCII ACP fast-path sizing and explicit-length no-NUL behavior.
- Focused raw coredll/kernel validation covers device-information success for
  activated and file-open device handles plus null, short-buffer, and stale
  handle failures.
- Focused raw coredll/kernel validation covers `FindFirstDevice`/`FindNextDevice`
  device search output and cleanup semantics.
- Focused Unicorn validation covers the ready parked wait handoff and the
  hidden-helper live-pump handoff. Release build passed with existing warnings.
- Focused raw coredll/kernel validation covers activation-time `BusName`
  capture, `$bus\...` device-info output, and bus-name wildcard search.
- Full `cargo fmt --check` may still report unrelated pre-existing formatting
  drift in older files; treat new non-formatting whitespace findings as
  actionable.

## Working Rules

- Keep future entries concise and source-visible.
- Record completed behavior here only after code and validation support it.
- Move active work to `TODO.md`; move reproducible unresolved failures to
  `KNOWN_BUGS.md`.
