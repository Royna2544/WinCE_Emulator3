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
