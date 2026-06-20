# Current Engineering Plan

Refreshed on 2026-06-20. This file is the active roadmap only; historical
investigation detail belongs in logs, commits, or targeted source-reference
notes when it is still needed for implementation.

## Implemented Baseline

- Runtime loader: MIPS PE parsing, import patching, runtime DLL mapping,
  dependency loading, forwarder resolution, load flags, TLS parsing, CE-style
  attach/detach handling, and process/thread lifecycle hooks are implemented
  to the current test-backed level.
- Shell and resource APIs: shell execution, file associations, shortcuts,
  special folders, shell icons, image lists, PE icon/version resources, and
  selected icon-rendering paths have raw coverage.
- GWE and messaging: windows, message queues, modal/menu/message-box paths,
  clipboard/caret state, cross-thread send setup, timeout handling, and several
  scheduler wait paths are present.
- Storage: host-backed mounted paths, directory enumeration, file attributes,
  change notification handles, detailed notification records, synthetic disk
  metadata, FSDMGR import shims, and mounted path routing are covered.
- Remote IO and devices: CE-facing Winsock, remote HTTP control, framebuffer
  capture, touch/key injection, serial GPS input, and audio sink plumbing are
  wired into the current runtime.
- Core subsystems: handles, waits, events, semaphores, mutexes, TLS, critical
  sections, memory, registry, GDI objects, DIBs, palettes, text, IME, and the
  scheduler have broad raw-test coverage with known remaining gaps.

## Recent CE-Backed Progress

- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\TOOLHELP\toolhelp.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\kmisc.c`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\toolhelp.h`, and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\tlhelp32.h`: raw
  `THCreateSnapshot` now allocates and seeds a CE-shaped `THSNAP` buffer for
  process snapshots instead of returning `ERROR_NOT_SUPPORTED`. Coverage
  verifies header offsets, process-entry layout, current/launched process IDs,
  duplicate process-handle deduplication, wide executable names, and invalid
  process rejection.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\CORE\LMEM\th32heap.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\TOOLHELP\toolhelp.c`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\toolhelp.h`, and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\tlhelp32.h`: raw `GetHeapSnapshot` now
  appends CE-layout `TH32HEAPLIST`/`HEAPENTRY32` records to a `THSNAP` buffer
  instead of reporting unsupported. Coverage verifies process-heap filtering,
  heap-list counts/sizes, default-heap flags, fixed allocation entries,
  process/heap IDs, and invalid snapshot rejection.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\TOOLHELP\toolhelp.c`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\toolhelp.h`, and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\tlhelp32.h`: `TH32CS_SNAPTHREAD`
  snapshots now append CE-layout `THREADENTRY32` records after the process
  slice. Coverage verifies thread IDs, owner process IDs, current-process IDs,
  sizes, and current/launched thread coverage.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\TOOLHELP\toolhelp.c`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\toolhelp.h`, and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\tlhelp32.h`: `TH32CS_SNAPMODULE`
  snapshots now append CE-layout `MODULEENTRY32` records from live loaded
  modules. Coverage verifies module IDs, owner process IDs, base/size fields,
  refcounts, names, guest paths, and flags.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\thread.c` and related CE
  diagnostics: raw `GetCallStackSnapshot` now returns one synthetic in-process
  frame for normal and extended buffers, validates supported flags, and handles
  skipped frames without fabricating deeper stacks.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\kdriver.c` and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\INC\loader.h`: raw
  `LoadKernelLibrary` now resolves registered CE kernel libraries instead of
  returning `ERROR_NOT_SUPPORTED`, applies the CE `LOAD_LIBRARY_IN_KERNEL`,
  `MF_NO_THREAD_CALLS`, and `LLIB_NO_PAGING` load flags, increments the module
  refcount, preserves exported `IOControl` lookup, and reports CE-style missing
  or invalid library-name failures.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\pathapi.cpp` and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\mounttable.cpp`: public
  raw `IsSystemFile` now follows CE's system-volume/file-attribute contract
  instead of returning unconditional false. Coverage verifies system-mounted
  files, non-system mounted files, missing paths, and null path rejection.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\kmisc.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\MIPS\mdsched.c`, and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winnt.h`: raw `QueryInstructionSet` now
  writes the CE MIPSII instruction-set code and applies CE's compatibility
  checks instead of returning a placeholder zero. Coverage verifies
  query-current, exact MIPSII matches, incompatible ARM/MIPS requests, and
  invalid output pointer failure.
- `C:\WINCE600\PUBLIC\COMMON\OAK\INC\pwindbas.h`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\mwindbas.h`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\DRIVERS\OOMUI\oomui.cpp`: raw
  `GetSystemMemoryDivision` now fills CE's store-pages, RAM-pages, and
  page-size outputs instead of reporting unsupported. Coverage verifies the
  object-store page count against `GetStoreInformation`, the emulator RAM/page
  constants, null output rejection, and unmapped output failure.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\process.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\loader.c`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\mkfuncs.h`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\pkfuncs.h`: raw `PageOutModule` now
  follows CE's flag and process/module handle validation instead of reporting
  unsupported. The emulator treats valid page-out requests as a no-op because
  it has no discardable committed code pages yet; coverage verifies valid
  module/process requests, public two-argument module calls, invalid flags,
  current-process rejection, and invalid module/process handles.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\CORE\DLL\loader.cpp` and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\pkfuncs.h`: raw `LoadDriver` now mirrors
  CE's `LoadLibraryExW(file, NULL, MAKELONG(0, LLIB_NO_PAGING))` wrapper
  instead of reporting unsupported. Coverage verifies refcounting, no-paging
  load flags, normal export lookup, missing-driver failure, and invalid name
  pointer rejection.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\kmisc.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\TOOLHELP\toolhelp.c`, and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\tlhelp32.h`: raw `THCreateSnapshot`
  now appends CE-layout `THREADENTRY32` rows for `TH32CS_SNAPTHREAD` instead
  of returning only process rows. Coverage verifies process rows are still
  included for thread snapshots, current and launched child thread IDs,
  owner/current process IDs, and thread-entry sizing.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\kmisc.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\TOOLHELP\toolhelp.c`, and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\tlhelp32.h`: raw `THCreateSnapshot`
  now appends CE-layout `MODULEENTRY32` rows for `TH32CS_SNAPMODULE`.
  Coverage verifies module-count/header placement, module IDs/handles,
  process IDs, ref counts, image base/size, short module names, full guest
  paths, and flags.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\process.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\vm.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\TOOLHELP\toolhelp.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\SHELL\shell.c`, and
  `C:\WINCE600\PRIVATE\SHELL\SHELLPSL\HAVEAYGSHELL\shellpsl.cpp`: raw
  `ReadProcessMemory`/`WriteProcessMemory` now validate process handles and
  copy mapped guest memory bytes with CE-style optional byte-count reporting
  instead of returning `ERROR_NOT_SUPPORTED`. Coverage verifies reads, writes,
  invalid process handles, unmapped source failures, and byte-count reset on
  failure.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\process.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\kwin32.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\UTILS\CMD2\cmdexec.cxx`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\TOOLHELP\toolhelp.c`, and shell/task
  callers under `C:\WINCE600\PRIVATE\WINCEOS\COREOS\SHELL`: raw
  `OpenProcess` now opens launched process IDs as distinct process handles
  instead of only accepting the current pseudo-process. The opened handles
  preserve CE-style current-process pseudo behavior, reject unknown PIDs, remain
  independently closeable, and observe shared process exit/wait state when the
  launched process terminates.
- `C:\WINCE600\PUBLIC\COMMON\OAK\INC\pwindbas.h`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\extfile.h`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\mwindbas.h`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\mextfile.h`, and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\mounttable.cpp`:
  public raw `RegisterAFSName`/`RegisterAFSEx`/`DeregisterAFS*` now reserve,
  bind, flag, and remove CE-style AFS mount slots instead of reporting
  unsupported or unconditional success. Coverage verifies null-name rejection,
  duplicate-name index/error behavior, invalid API-set handles, hidden/system/
  permanent mount flags feeding the mounted namespace, duplicate volume binding,
  invalid deregistration indices, and successful unmount cleanup.
- `C:\WINCE600\PUBLIC\COMMON\DDK\INC\devload.h` and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\windev.h`: raw `GetDeviceKeys` now
  returns deterministic CE-style active and driver registry paths for enabled
  configured devices instead of reporting unsupported. Coverage verifies null
  argument rejection, unknown/disabled device lookup failures, length-only
  queries, `ERROR_MORE_DATA` size reporting, case-insensitive device matching,
  and successful `Drivers\Active\N` / `Drivers\BuiltIn\NAME` output.
- `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winbase.h`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\battery.h`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\battimpl.h`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\DRIVERS\BATTDRVR\MDD\battapi.c`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\DRIVERS\BATTDRVR\PDD\sbattif.c`:
  raw `GetSystemPowerStatusEx`/`GetSystemPowerStatusEx2` now fill CE-layout
  power-status structures with bounded `Ex2` byte-copy semantics, and the
  battery driver helper ordinals no longer report unsupported. Coverage
  verifies null status rejection, old/new struct offsets, `Ex2` return length,
  no-battery AC-powered metadata, level/change-notification helpers, lifetime
  info outputs, and time-change notification no-op behavior.
- `C:\WINCE600\PUBLIC\COMMON\OAK\INC\pwinuser.h` and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\SHELL\shell.c`: raw
  `GetForegroundInfo` now fills CE's foreground snapshot instead of reporting
  unsupported. Coverage verifies null output rejection, empty foreground
  state, active/focus/menu/keyboard-destination fields, IME/menu composition
  placeholders, and keyboard-layout handle reporting.
- `C:\WINCE600\PRIVATE\WINCEOS\DRIVERS\SERDEV\serial.c`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winbase.h`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\DRIVERS\SERIAL\SER_CARD\ser_card.c`, and
  `C:\WINCE600\PRIVATE\WINCEOS\COMM\TAPI\UNIMODEM\config.c`:
  raw `GetCommProperties` now returns CE-style serial `COMMPROP` data instead
  of reporting unsupported. Coverage verifies serial-handle validation, null
  output rejection, packet metadata, RS-232 provider identity, queue sizes,
  provider capability masks, settable baud/data/stop/parity masks, and empty
  current queue/provider-specific fields.
- `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winreg.h`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\DRIVERS\PM\MDD\pmsysreg.h`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\DRIVERS\PM\PDD\PDA\pwstates.h`:
  raw `CeFindFirstRegChange` now returns a CE-style waitable registry
  notification handle instead of reporting unsupported. The handle starts
  unsignaled, `CeFindNextRegChange` re-arms it by resetting the event, and
  `CeFindCloseRegChange` closes it through the kernel handle table. Coverage
  verifies valid predefined-key watches, invalid `HKEY` rejection, wait/reset
  behavior, and invalid/closed notification handles.
- `C:\WINCE600\PUBLIC\COMMON\DDK\INC\devload.h`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\windev.h`,
  `C:\WINCE600\PUBLIC\WCESHELLFE\OAK\CTLPNL\CPLMAIN\system.cpp`, and
  `C:\WINCE600\PRIVATE\TEST\STRESS\STRESS\MODULES\NET\BTIR\CLIENT\btirclient.cpp`:
  raw `EnumPnpIds`/`EnumDevices` now follow CE's double-null wide list and
  byte-sized buffer contract instead of reporting unsupported. Coverage
  verifies size queries, `ERROR_MORE_DATA`, null length-pointer rejection,
  empty PNP lists, and configured enabled device enumeration.
- `C:\WINCE600\PUBLIC\COMMON\SDK\INC\keybd.h`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\DRIVERS\KEYBD\LAYMGR\laymgr.cpp`,
  `C:\WINCE600\PUBLIC\WCESHELLFE\OAK\CTLPNL\CPLMAIN\keybd.cpp`, and
  `C:\WINCE600\PRIVATE\TEST\BASEOS\DRIVERS\KEYBOARD\BVT_API\KBDTEST\ratdelay.cpp`:
  raw `KeybdGetDeviceInfo` now reports CE keyboard metadata instead of
  returning unsupported. Coverage verifies VKey-to-Unicode capacity,
  autorepeat defaults, CE's selectable min/max delay/rate ranges, keyboard
  present/enabled/status flags, invalid IDs, null buffers, and
  `ERROR_INVALID_PARAMETER` reporting.
- `C:\WINCE600\PUBLIC\COMMON\SDK\INC\nled.h`,
  `C:\WINCE600\PLATFORM\DEVICEEMULATOR\SRC\DRIVERS\NLEDDRVR\snled.cpp`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\DRIVERS\NLEDDRVR\MDD\nleddrvr.cpp`, and
  `C:\WINCE600\PRIVATE\TEST\BASEOS\DRIVERS\LED\BVT\NLEDTEST\tests.cpp`:
  raw `NLedGetDeviceInfo`/`NLedSetDevice` now model the CE Device Emulator's
  two notification devices instead of returning unsupported or unconditional
  success. Coverage verifies count/supports/settings queries, invalid IDs,
  null/unmapped buffers, invalid LED numbers, `ERROR_INVALID_PARAMETER`
  reporting, and validated settings round-trips with CE-style total cycle
  recomputation.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\doc.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\MGDI\INC\dc.hpp`, and
  `C:\WINCE600\PUBLIC\DIRECTX\SDK\SAMPLES\DXMISC\DXVIEW\dxprint.c`:
  raw `SetAbortProc` now follows CE's HDC validation contract instead of the
  generic unsupported-printer stub. Bad/null/deleted HDCs return `SP_ERROR`
  with `ERROR_INVALID_HANDLE`, while valid DCs store the abort callback on DC
  state for later print/spooler parity work.
- `C:\WINCE600\PUBLIC\RDP\OAK\UIT\SHELL\WBTSHELL\wbttskbr.cpp`,
  `C:\WINCE600\PUBLIC\VOIP\OAK\PHONE\PHONEAPP\PHONECORE\hotkeys.cpp`, and
  `C:\WINCE600\PUBLIC\VOIP\OAK\PHONE\HOMESCREEN\hotkeys.cpp`:
  raw `RegisterHotKey`/`UnregisterHotKey` now keep CE-style window/thread
  hotkey registrations instead of hard-failing. Coverage verifies invalid
  window rejection, same-window/id replacement, `WM_HOTKEY` delivery on the
  registered chord, unregister cleanup, and missing-registration failure.
- `C:\WINCE600\PRIVATE\TEST\MULTIMEDIA\DIRECTX\DSHOW\COMMON\LIBS\IMAGING\imageload.cpp`,
  `C:\WINCE600\PRIVATE\TEST\MULTIMEDIA\DIRECTX\DSHOW\PLAYBACK\LOGGERS\videoframeinstrumenter.cpp`,
  and
  `C:\WINCE600\PUBLIC\COMMON\OAK\DRIVERS\CAPTURE\CAMERA\LAYERED\PDD_NULL\nullpdd.cpp`:
  `CreateBitmapFromPointer` is now implemented for CE-style BITMAPINFO inputs
  that alias caller-owned image memory with an explicit stride. Raw coverage
  verifies invalid parameter handling, top-down RGB565 BITMAPINFO metadata,
  padded source rows, selected memory-DC BitBlt behavior, and non-owned bitmap
  lifetime after `DeleteObject`.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\draw.cpp`:
  `GetPixelAlphaDIBTest` and `SetPixelAlphaDIBTest` are now covered for raw
  selected 32 bpp DIBs. `GetPixel` truncates stored alpha, and `SetPixel`
  realizes RGB while leaving the DIB alpha byte clear.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\draw.cpp`:
  `BltAlphaDIBTest` is now covered for BitBlt, StretchBlt, and MaskBlt
  selected 32 bpp DIB copy paths, preserving source alpha bytes through
  SRCCOPY-style DIB-to-DIB transfers.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\draw.cpp`:
  `BltAlphaDIBTest(EPatBlt)` is now covered for alpha-bearing solid and
  pattern brushes on selected 32 bpp DIBs; CE leaves the resulting alpha bytes
  undefined but requires the `PatBlt(PATCOPY)` calls to succeed.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\draw.cpp`:
  `BltAlphaDIBTest(ETransparentImage)` now follows CE's 32 bpp DIB behavior:
  transparent copies preserve RGB while truncating the destination alpha byte.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\draw.cpp`:
  `OddCreatecompatibleBitmapTest` is now covered for primary-display depth,
  compatible-DC stock monochrome depth, and CE's 1x1 monochrome fallback when
  `CreateCompatibleBitmap` receives a zero width or height.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\draw.cpp`:
  `CreateCompatibleBitmapDIBSECTIONTest` is now covered for primary-display
  compatible bitmaps: `GetObject` returns a `DIBSECTION` with CE-style RGB565
  `BI_BITFIELDS` masks.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\draw.cpp`:
  `CreateCompatibleToDibTest` is now covered for selected indexed DIB
  metadata: compatible bitmaps inherit the selected DIB color table so
  subsequent drawing/blits preserve the same palette semantics.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\draw.cpp`:
  `CreateBitmap0SizeTest` and `CreateBitmapNegSize` are now covered:
  zero-width/zero-height `CreateBitmap` calls return CE's 1x1 monochrome
  fallback, while negative dimensions still fail.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\draw.cpp`:
  `BitmapGetDIBColorTable` is now covered for indexed `CreateBitmap` DDBs:
  `GetDIBColorTable` reports CE's default grayscale table, while
  `SetDIBColorTable` still rejects non-DIBSECTION DDBs.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\draw.cpp`:
  `CreateBitmapBitsTest` is now covered for caller-provided initial bits:
  `CreateBitmap(..., lpBits)` copies the source bytes into owned bitmap
  storage instead of aliasing the caller buffer.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\draw.cpp`:
  `GetSetDIBColorTable` and the 2 bpp `GradientFill` setup path are now
  covered for indexed `CreateDIBSection(DIB_PAL_COLORS)` surfaces: 1/2/4/8 bpp
  DIBSections create successfully and round-trip mutable color tables, while
  truecolor `DIB_PAL_COLORS` still fails.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\draw.cpp`:
  `SetBitmapBitsOnePixel` is now covered beyond `CreateBitmap`: bottom-up
  DIBSECTION surfaces across 1/2/4/8/16/24/32 bpp and display-compatible
  bitmaps place the copied source bytes on CE's logical top row.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\draw.cpp`:
  `CreateBitmapSquares*` is now covered for `CreateBitmap(..., lpBits)` and
  `SetBitmapBits`: caller-provided DDB bits use CE's WORD-aligned source rows
  while emulator-owned bitmap storage can keep its DWORD-aligned stride.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\draw.cpp`:
  `BltFromMonochromeTest` is now covered for additional source creation
  variants: `CreateCompatibleBitmap` mono DDB sources remap through destination
  text/background colors, while 1 bpp DIBSECTION sources keep their own
  black/white color table.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\draw.cpp`:
  `BltToMonochromeTest` is now covered for additional BitBlt destination
  variants: `CreateCompatibleBitmap` mono DDB destinations map source pixels
  through the source background color, while 1 bpp DIBSECTION destinations map
  directly through their black/white palette.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\font.cpp`:
  `passNull2Font` is now covered for `EnumFontsW`,
  `EnumFontFamiliesW`, and `EnumFontFamiliesExW` raw validation: bad HDCs
  report `ERROR_INVALID_HANDLE`, null callbacks or null `LOGFONTW` inputs
  report `ERROR_INVALID_PARAMETER`, and valid non-Unicorn fallbacks succeed.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\dc.cpp`:
  `passNull2DC(ECreateCompatibleDC)` is now covered for the CE invalid-source
  HDC cases: `CreateCompatibleDC(NULL)` still creates a screen-compatible
  memory DC, while bad nonzero HDCs fail with `ERROR_INVALID_HANDLE`.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\dc.cpp`:
  `passNull2DC(EExtEscape)` is now covered for `QUERYESCSUPPORT` invalid-HDC
  edges: null and bad HDC inputs return unsupported and report
  `ERROR_INVALID_PARAMETER`, matching CE's parameter-first validation.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\brush.cpp`:
  `SimpleCreateSolidBrushTest` is now covered for the CE `CLR_INVALID`
  rejection path: `CreateSolidBrush(0xffffffff)` fails with
  `ERROR_INVALID_PARAMETER`, while valid colors still create brush handles.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\brush.cpp`:
  `CreatePenIndirectBadParamTest` is now covered for CE raw pen creation
  style validation: `CreatePen` and `CreatePenIndirect` accept only
  `PS_SOLID`, `PS_DASH`, and `PS_NULL`; unsupported desktop styles fail with
  `ERROR_INVALID_PARAMETER`.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\font.cpp`:
  `passNull2Font(ERemoveFontResource)` is now covered for raw
  `RemoveFontResourceW`: null paths fail with `ERROR_INVALID_PARAMETER`,
  missing paths fail with `ERROR_FILE_NOT_FOUND`, and existing mounted font
  paths still broadcast `WM_FONTCHANGE` on success.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\font.cpp`:
  `passNull2Font(EAddFontResource)` is now covered for raw
  `AddFontResourceW`: null paths fail with `ERROR_INVALID_PARAMETER`,
  missing and empty paths fail with `ERROR_FILE_NOT_FOUND`, and existing
  mounted font paths still broadcast `WM_FONTCHANGE` on success.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\font.cpp`:
  `MultipleAddRemoveTest` is now covered for raw font resources:
  `AddFontResourceW` records CE-style per-path references, the matching number
  of `RemoveFontResourceW` calls succeeds, and extra removes fail with
  `ERROR_FILE_NOT_FOUND`.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\font.cpp`:
  `SimpleAddRemoveFontPairs`/`SuperAddRemove` invalid-add paths are now
  represented for raw `AddFontResourceW`: readable files with unsupported font
  signatures fail with `ERROR_INVALID_DATA` instead of being registered.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\dc.cpp`:
  `ScrollDCBadParam` is now covered for raw `ScrollDC`: null/bad HDCs fail
  with `ERROR_INVALID_HANDLE`, diagonal scroll deltas fail with
  `ERROR_INVALID_PARAMETER`, and non-diagonal no-op scrolling still succeeds.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\dc.cpp`:
  `ProgramError` is now covered for raw DC lifecycle validation:
  `ReleaseDC(NULL, hdc)` rejects a previously deleted DC with
  `ERROR_INVALID_HANDLE` instead of treating the stale handle as released.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\dc.cpp`:
  `ScrollDCScrollTest` update geometry is now covered for raw `ScrollDC`:
  exposed edge rectangles are reported for one-axis scrolls, and
  `hrgnUpdate` is updated to the same simple region box.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\dc.cpp`:
  `SimpleScrollDCTest` pixel movement is now represented for raw
  framebuffer-backed `ScrollDC`: one-axis scrolls move screen pixels in CE's
  sign convention and clear the newly exposed edge.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\clip.cpp`:
  `TestGetClipBox1` empty-clip behavior is now explicitly covered for raw
  clip DC state: an empty `IntersectClipRect` produces `NULLREGION`, and
  `GetClipBox` returns the CE zero rectangle.
- `C:\WINCE600\PRIVATE\TEST\MULTIMEDIA\DIRECTX\DSHOW\PLAYBACK\LOGGERS\videoframerecognizer.cpp`
  and `videoframeinstrumenter.cpp`: the DirectShow logger pattern
  `LoadImage(NULL, file, IMAGE_BITMAP, 0, 0, LR_LOADFROMFILE)` is now covered
  for raw `LoadImageW`; existing bitmap files return read-only HBITMAP objects
  and missing paths report `ERROR_FILE_NOT_FOUND`.
- `C:\WINCE600\PUBLIC\COMMON\OAK\WTL\INCLUDE\atlframe.h` and
  `atlctrlw.h`: `LR_CREATEDIBSECTION` on `LoadImageW(IMAGE_BITMAP)` is now
  honored for raw file/resource bitmap loads, so `GetObjectW` can expose the
  CE DIBSECTION-sized object shape when callers request it.
- `C:\WINCE600\PUBLIC\VOIP\SDK\SAMPLES\VOIP\VOIPDEMO\SOURCE\voipdemo.cpp`,
  `C:\WINCE600\PUBLIC\SHELL\OAK\HPC\EXPLORER\TASKBAR\taskbar.cpp`, and
  `C:\WINCE600\PUBLIC\WCESHELLFE\OAK\CTLPNL\CPLMAIN\cplutils.cpp`: CE shell
  callers rely on `LoadImageW(IMAGE_ICON, 16, 16, LR_DEFAULTCOLOR)` selecting
  a concrete RT_ICON image from an RT_GROUP_ICON; raw `LoadImageW` now resolves
  module icon groups into bitmap-backed HICON objects for the requested size.
- `C:\WINCE600\PUBLIC\SHELL\OAK\OWNERDRAWLIB\odapp.cpp` and
  `C:\WINCE600\PUBLIC\COMMON\OAK\WTL\INCLUDE\atlmisc.h`: module cursor
  resources loaded by `LoadCursorW`/`LoadImageW(IMAGE_CURSOR)` now resolve
  RT_GROUP_CURSOR entries into real cursor objects with RT_CURSOR bitmaps and
  `GetIconInfo` hotspot metadata instead of returning the group resource handle.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\CORE\DLL\resource.cpp`,
  `C:\WINCE600\PUBLIC\SHELL\OAK\HPC\CESHELL\API\iconcache.cpp`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\NOTIFY\notifext.cxx`: raw
  `KernExtractIcons` now mirrors CE's callback-selected large/small
  RT_GROUP_ICON entries by choosing 32x32 and 16x16 resources instead of
  blindly extracting directory entries 0 and 1, while preserving CE's partial
  missing-resource behavior.

## Active Roadmap

1. Close loader parity gaps outside the current Unicorn happy path.
   - Add focused raw/non-Unicorn fixture variants where runtime coverage is
     stronger than raw coverage.
   - Keep dependency attach/detach ordering, forwarder failure modes,
     datafile/no-resolve loads, and invalid image edges source-backed.

2. Finish shell rendering parity.
   - Extend `ExtractIconExW`, `KernExtractIcons`, `DrawIconEx`, and image-list
     coverage across remaining PE formats, masks, palettes, stretch paths,
     framebuffer output, and invalid-handle cleanup.

3. Finish modal UI and cross-thread message semantics.
   - Extend `SendMessageTimeout`, `MsgWaitForMultipleObjectsEx`, nested sends,
     reentrant waits, abort-if-hung behavior, and `ReplyMessage` parity.
   - Keep menu and message-box behavior aligned with CE blocking,
     owner-window, topmost, cancellation, and command-routing expectations.

4. Complete shell notification behavior.
   - Validate `Shell_NotifyIcon` and `SHNotification*` lifetimes, taskbar
     routing, update/remove paths, callback dispatch, stale-handle handling,
     and remove/update races in integrated runtime scenarios.

5. Harden mounted storage behavior.
   - Continue physical block-driver, cache/filter, FMD, utility-DLL,
     file-security ACL, mounted-volume availability, and lower-FSD forwarding
     work beyond the current synthetic host-backed path.

6. Expand input, text, GDI, and drawing fidelity.
   - Add coverage for IME UI callbacks, caret timing, font fallback, glyph
     metrics, palette/device-color behavior, alpha/mask variants, DIB lifetime
     edges, clipping, and read-only bitmap rejection where CE evidence exists.

7. Advance the iNavi route/runtime frontier.
   - Continue from the current real splash/resource-loading state toward the
     map transition, route search, destination entry, and realistic navigation.
   - Trace the actual CE wait/event/message or app readiness transition that
     should hide, destroy, or demote the owned splash popup.
   - Keep runtime policy generic; do not add app-specific shortcuts.

## Validation Gates

- Run `cargo fmt` or targeted `rustfmt` after Rust edits.
- Run `cargo check --features unicorn,trace,win32-desktop` for shared changes.
- Run targeted tests for the touched area, for example:
  - `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel`
  - `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
  - `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`
  - `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems`
- Run mounted-layout or live application checks when touching process startup,
  shell routing, storage, remote input, or UI scheduling behavior.

## Completion Criteria

Delete this roadmap only after the active roadmap is empty, source/tests
reflect the final state, and validation passes for the completed scope.
