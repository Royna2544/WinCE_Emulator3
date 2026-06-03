# TODO

## Immediate

- Continue the launch path after the first synthetic `WM_PAINT` dispatch by
  expanding CE-referenced GDI/surface drawing and blit behavior beyond the
  first solid `FillRect` framebuffer path, then verify those pixels through the
  generic presenter boundary. Do not treat the timeout-running paint loop as
  GUI success.
- Continue the iNavi render-surface path with targeted diagnostics around the
  resize/surface allocation gate, not app-state forcing. Confirmed host/tap
  evidence: `render_size_entry` receives `800x480`, but the path never reaches
  `render_surface_create_call`/`render_surface_store` at
  `0x00104904`/`0x00104910`; `WM_PAINT` later calls render entry
  `0x0010518c`, which returns immediately because `render_surface=0` and
  `render_enabled=0`. Next evidence should identify the branch/input state that
  skips the allocation block around `0x00104878..0x00104954`.
- Continue the post-time iNavi path from the new wall-clock diagnostic frontier.
  The latest mounted run now gets past the earlier export-index
  `GetPaletteEntries` trap via real palette/DC state, preserves SDK CRT
  ordinals such as `memset @1047` and `swprintf @1097` before export-index
  fallback, returns heap-backed `RegisterGesture @2724` state, and writes
  `GetSystemTime @25`. With sampled Unicorn code tracing and mapped-code
  instruction reads, a 90,000 ms mounted no-tap run now returns in roughly 27 s
  at an idle `GetMessageW @861` `blocked_get_message` snapshot instead of
  timing out in app-side date/geometry logic. The visible top-level
  `wce_solution_inavi` HWND is `800x480`, and the `Afx:10000:b:0:40000006:0`
  child HWND exists, but the framebuffer dump is still all zero. Use the idle
  frontier to keep probing WNDPROC/paint/GDI behavior before expecting guest
  drawing.
- Continue from the new post-jump-table exit frontier. The latest release
  mounted run gets past `__nes @2047`, `__litofp @2032`, `__ll_div @2005`,
  `GetTimeZoneInformation @27`, `SetForegroundWindow @702`,
  `InputDebugCharW @595`, and the previous trampoline corruption of the iNavi
  halfword jump table at `0x000ebbf0`. The `ADBSetAccountProperties @1943`
  frontier now returns `FALSE`/`ERROR_NOT_SUPPORTED` and the app proceeds to an
  encoded `TerminateProcess` exit (`caller=0x0048fa90`, process `0x42`,
  `exit_code=0`). The framebuffer dump `target\inavi-release-adb1943.ppm` is
  still all zero. WNDPROC return trampoline-origin tracing now decodes the
  shutdown path as `0x56d0` entering `0x0004390c`, then an app-side `0x5236`
  send at `0x00043e30`/`0x00043e38`; the main `wce_solution_inavi` WNDPROC maps
  that to `WM_CLOSE`. Disassemble the branch path through `0x0004390c` and
  determine which preceding CE/MFC resource, window, or service result is
  causing the app to shut down before useful drawing.
- Replace launch-stub behavior for commctrl, WINSOCK, and OLE imports with
  real subsystem-backed implementations as import traces demand. Keep MFC on
  the loaded SDK DLL path only; do not add emulator MFC stubs.
- Continue burning down COREDLL ordinals subsystem by subsystem, replacing
  stubbed ordinal plan entries with CE/MFC/SDK-referenced semantics. Next
  likely tranche: `BitBlt`, `PatBlt`, `StretchDIBits`, `SetDIBitsToDevice`,
  basic shape/text drawing, and memory-DC bitmap surfaces into or through the
  virtual framebuffer; PE-backed resource icon/bitmap loading beyond the
  string-resource path,
  COM/OLE API dispatch when ole32 imports are connected, more GWE menu/dialog/
  control raw pointer marshalling, broader file attributes/directory metadata,
  and timer/system-time structs.
- Continue tracing after CE `CreateWindowExW` now delivers the source-backed
  create-time `WM_CREATE` callout and CE `CallWindowProcW` enters guest
  window-procedure targets. The latest bounded snapshot still reaches SDK MFC
  default/idle handling and then an empty-queue `GetMessageW` diagnostic; the
  former ordinal-1036 `longjmp`/`pc=0` crash is no longer the current stop.
  Raw `GetWindow` sibling/child traversal is now connected for the observed
  MFC `GetWindow @251` calls. Virtual show/move/size lifecycle messages are
  queued for raw `ShowWindow`, `SetWindowPos`, `MoveWindow`, and visible
  top-level `CreateWindowExW`; the mounted bounded rerun confirmed
  `\SDMMC Disk\iNaviData` succeeds and creates `WCE_Solution_iNavi` plus an
  MFC child window. The latest rerun gets past the previous
  `GetPaletteEntries` trap, SDK CRT ordinal normalization bug, and
  `RegisterGesture @2724` pointer-return path, and `GetSystemTime @25`; the
  current wall-clock-bounded post-time run names the next frontier as repeated
  startup CRT/import activity before visible drawing. Continue replacing raw
  COREDLL/GDI/DC
  behavior with CE-referenced semantics that advance the path toward target
  framebuffer drawing.
- Use the new guest-WNDPROC return ring to compare creation-time sequencing
  against CE/MFC expectations. The latest diagnostic shows create/show/size/
  paint/idle messages returning `0`, `WM_PAINT` not reaching `BeginPaint`, MFC
  dispatch through `AfxWndProcBase` (`0x6004eba8`), and `Solution_iNavi`
  registered with target WNDPROC `0x000135cc`. A `--tap 400,240` idle-frontier
  run now confirms queued `WM_LBUTTONDOWN`/`WM_LBUTTONUP` delivery and drain
  through the active HWND, but still produces an all-zero framebuffer. After
  correcting Unicorn paint validation semantics, the app WNDPROC still routes
  top-level `WM_PAINT` to `DefWindowProcW` without `BeginPaint` or drawing
  imports. Continue with a targeted probe of the `0x000135cc` app
  WNDPROC/message-map branch, `SetWindowLongW`/superclass state, and
  first-message creation ordering before adding more lifecycle messages.
- Continue connecting SDK CE 4.2 Mipsii COREDLL CRT ordinals from `coredll.lib`
  as the launch trace demands.
- Add focused import-trap tests for Unicorn `_setjmp`/`longjmp` register/PC
  restoration once the fixture harness is wired to the existing
  `tests/test_progs/006_setjmp_longjmp` program.
- Implement CRT `_msize`/`realloc`/operator delete ordinals from SDK evidence so
  MFC/CRT heap paths do not rely only on Local/Heap reallocation aliases.
- Extend `cemath` as real guest imports demand more CRT/floating-point helpers.
- Extend subsystem smoke tests as each shim is connected to guest import traps.
- Add import-trap argument/result marshalling tests that exercise the new raw
  heap/file/find/message/resource ordinals through decoded guest MIPS
  registers.
- Continue PE resource directory integration beyond string tables so
  `FindResourceW`, `LoadResource`, and `SizeofResource` can consume mapped
  icon/bitmap/dialog/menu data rather than only test-registered virtual
  resources and PE-backed strings.
- Keep resource lookup evidence current. `FindResource(W)` for `RT_STRING` now
  falls back from an individual string id to its containing string block, which
  removed the observed `#3867` string-resource miss in a real host/tap run.
  The older `0x0e01` main-EXE RT_STRING miss remains explained by the EXE
  having no RT_STRING table; continue resource-module/MFC fallback
  investigation only if current traces demand it.
- When GWE/DC behavior is ready, adapt window state to the generic `Desktop`
  trait boundary without replacing CE/MFC message, class, or window semantics
  with host-window shortcuts.

## Next

- Extend `--monitor` from a bounded-run command loop into a persistent
  Unicorn debugger session. Needed pieces: retain live Unicorn CPU/register/
  memory state across commands, expose memory examine/write commands, and add
  explicit snapshot/restore checkpoints for rewind without corrupting CE kernel
  state.
- Extend bounded run tooling beyond the current snapshot import ring if more
  structured trace context is needed.
- Trace why the now-consumed `--tap 400,240` messages do not trigger useful
  paint, child-window, or custom-message drawing behavior. The next useful
  evidence is the exact WNDPROC/superclass path and any GDI/DC/resource imports
  following the delivered mouse down/up.
- Add an HTTP/WebSocket transport over the Rust `CeRemote` API state when the
  host runtime is ready for remote UI/audio streaming; audio transport should
  honor the sink's per-client cursors and flush-marked chunks immediately.
- Add ordinal/decorated-name evidence from the Windows CE 4.2 Mipsii SDK import
  libraries, alongside the source references already recorded.
- Persist host-backed registry writes separately from the source dump.
- Add real serial backend support for `win32_com` devices.
- Bridge selected virtual Win32/CE APIs to host Win32 APIs where that preserves
  real guest semantics.

## Later

- Keep host presentation/streaming of framebuffer snapshots wired through
  `Presenter` implementations as guest drawing paths start writing meaningful
  pixels.
- Add real low-latency host playback draining behind `HostAudioSink`; current
  waveOut work copies guest PCM into registered sinks and `main` registers the
  Windows `winmm` host-sink boundary, but the host backend still retains chunks
  instead of owning a full playback queue.
- Implement socket behavior for WINSOCK imports.

## Parked

- App-specific fixes are parked unless backed by guest execution evidence.
