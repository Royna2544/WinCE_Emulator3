# Known Bugs And Risks

Regenerated on 2026-06-11 from current source and test evidence. Items here are unresolved issues, unverified behavior, or risk areas that should not be presented as complete.

## Open Issues

- Raw `AlphaBlend` now covers selected CE `g_stcPPAlpha` 32bpp per-pixel-alpha rows; remaining alpha gaps are broader orientation, primary-surface, clipping, and palette/device-color cases beyond those table rows.
- `SendMessageTimeout`/message-wait behavior is not yet proven complete for broader reentrant waits, cancellation, live modal-loop interaction, and all scheduler wait combinations; zero-timeout cross-thread raw expiry, nonzero timeout stale-delivery cleanup, Unicorn block-prep refusal, parked timeout cleanup, direct/scheduler result-ready replay, below-threshold vs hung `SMTO_ABORTIFHUNG`, combined `SMTO_BLOCK | SMTO_ABORTIFHUNG` hung abort, same-thread abort-if-hung non-abort, early/nested `ReplyMessage`, active-outer-timeout/nested-send lifetime coverage, message-wake indexing after multiple unsignaled handles, unreadable `MsgWaitForMultipleObjectsEx` handle-array `WAIT_FAILED`/`ERROR_INVALID_PARAMETER` diagnostics, and invalid CE message-wait handle-array shapes are covered.
- Shell notification behavior needs integrated validation for Explorer/taskbar `IShellNotificationCallback` dispatch through a real guest COM object lifecycle, taskbar-visible state, broader handle ownership, and remove/update races; the raw failed `Shell_NotifyIcon` no-taskbar-post paths, stale registered-taskbar success path, live modify/delete taskbar-copy paths, `SHNN_SHOW` window-only taskbar path, GUID-value OLE `CoCreateInstance` import writeback, local callback-acquisition queue path, unregistered callback-CLSID sink fallback, overlong taskbar-title clearing, mapped guest vtable callout, unmapped callback-pointer retry, and iconic remove/sink-cleanup icon lifetime paths are now covered.
- Mounted storage behavior still needs coverage for physical block-driver backing, broader external cache DLL/filter behavior, real CE mounted-volume availability/powerdown/thread-wait reference behavior, broader file-security ACL storage/enforcement, remaining hardware-flash/callable FMD interface thunks/actual reserved-region storage/FLS region-table forwarding and specialized disk IOCTLs, real external-copy accelerator behavior, and real mounted-FSD `FsIoControl` hook forwarding beyond the host-backed unsupported stub; the `fsdmgr.dll` `FSINT_*`/`FSEXT_*` notification import surface, direct `FSDMGR_AdvertiseInterface @2` advertisement publication/removal, FSDMGR cache imports (`FSDMGR_CacheIoControl @3`, `FSDMGR_CachedRead @4`, `FSDMGR_CachedWrite @5`, `FSDMGR_CreateCache @6`, `FSDMGR_DeleteCache @9`, `FSDMGR_FlushCache @14`, `FSDMGR_InvalidateCache @24`, `FSDMGR_ResizeCache @30`, `FSDMGR_SyncCache @32`), direct sparse-sector disk read/write support through `FSDMGR_DiskIoControl @12`, `FSDMGR_ReadDisk @25`, `FSDMGR_ReadDiskEx @26`, `FSDMGR_WriteDisk @35`, and `FSDMGR_WriteDiskEx @36`, basic disk metadata/status IOCTLs, `GET_SECTOR_ADDR` validation/no-XIP unsupported behavior, `GETPMTIMINGS` zero timing snapshots, secure-wipe sector clearing and set-secure-wipe-flag validation/no-op behavior, copy-external-start `DISK_COPY_EXTERNAL` payload validation/unsupported no-touch behavior, exact-size `DELETE_SECTOR_INFO` validation and sparse-sector clearing, `FSDMGR_CreateFileHandle @7`, `FSDMGR_CreateSearchHandle @8`, `FSDMGR_DeregisterVolume @10`, `FSDMGR_GetVolumeHandle @21`, `FSDMGR_GetVolumeName @22`, `FSDMGR_RegisterVolume @27`, `FSDMGR_GetMountFlags @37`, `STOREMGR_FsIoControlW @44` refresh/flush no-op and unsupported host-backed no-touch failure behavior, `FSDMGR_AsyncEnterVolume @80`/`FSDMGR_AsyncExitVolume @81` one-shot lock-token lifetime validation, `FSDMGR_ParseSecurityDescriptor @82` security-attribute parsing, coredll/AFS `GetFileSecurityW`/`SetFileSecurityW` no-security-manager mounted-path routing, `FSCTL_COPY_EXTERNAL_START`/`COMPLETE` fixed `FILE_COPY_EXTERNAL` validation/unsupported no-touch behavior, file-handle `FSCTL_SET_FILE_CACHE(FileCacheDisableStandard)` no-op success plus unsupported enable/discard and volume-level rejection, FSDMGR first-change import owner regression, internal `FSINT_*` notification-handle owner/reset/info/close split, internal-vs-external wrong-handle close split, public direct `CloseHandle` owner rejection, direct AFS `hProc` owner close/duplicate rejection, raw AFS volume-handle owner/unmount/close signaling, volume-handle/FSDMGR-import `FSCTL_GET_VOLUME_INFO` metadata, FSDMGR mount name suffixing/AFS flag metadata, FSDMGR disk-pointer-to-HVOL mapping, FSD context-pointer file/search handle shims, CE null-cache fallback ID/status behavior, cross-mounted file-change isolation, same-parent and cross-parent mounted rename scoping, target-process duplicate ownership, write-close completion records, read-only-source cross-volume move copy/delete semantics, DeleteAndRename/PrestoChango direction, read-only/removable volume attributes, and system/hidden/permanent mount attributes are now covered for mounted roots.
- Public `LockFileEx`/`UnlockFileEx` now resolve at the CE coredll ordinals, validate real file handles plus readable `OVERLAPPED*` wrappers, enforce host-backed byte-range owner/conflict state for immediate callers, and release owned locks on file-handle close. Remaining file-lock gaps are CE-style blocking wait queues for non-immediate conflicts plus lower-FSD/filter forwarding.
- Direct synthetic-disk `IOCTL_DISK_FLUSH_CACHE` is covered as a successful no-op; remaining flush risk is limited to real hardware/cache-DLL forwarding behavior.
- Direct synthetic-disk `FSDMGR_GetDiskInfo @16` and `FSDMGR_GetDiskName @17` imports are covered; remaining disk metadata risk is limited to real physical block-driver geometry/name forwarding.
- Direct synthetic-disk `IOCTL_DISK_GETINFO` and legacy `DISK_IOCTL_GETINFO` are both covered for the CE `DISK_INFO` output surface; remaining GETINFO risk is limited to real physical block-driver forwarding.
- Direct synthetic-disk `IOCTL_DISK_SETINFO`/`DISK_IOCTL_SETINFO` is covered for validated `DISK_INFO` persistence into later GETINFO calls; real physical block-driver SETINFO forwarding remains open.
- File-handle `IOCTL_FILE_READ_SCATTER`/`IOCTL_FILE_WRITE_GATHER` are covered for synchronous page-multiple current-position and reserved offset-array transfers, including cachefilt-style ignored non-null `OVERLAPPED*` pointers; real lower-FSD/filter scatter/gather forwarding remains open.
- Direct synthetic-disk `IOCTL_DISK_FORMAT_VOLUME`/`IOCTL_DISK_SCAN_VOLUME` is covered for sparse-sector clearing and no-op scan success; real FATFS format/scan utility execution remains open.
- Direct synthetic-disk CE `fmd.h` IOCTLs are covered for XIP mode state, block lock/unlock state, a 56-byte `FMDInterface` ABI skeleton with null callback slots, sector-size state, total-block-bounded raw block writes, CE `ReservedReq` validation against the empty reserved table, `FlashRegion` table validation/storage, raw-block-size, and deterministic `FMDInfo` responses with the stored region count; real flash hardware interface, actual reserved-region storage/read/write, real FLS region-table forwarding, and callable FMD callback behavior remain open.
- Direct synthetic-disk standby, obsolete delete-cluster, and disk-level CD-ROM IOCTLs are covered as unsupported/no-touch; real standby power management, cluster deletion, and CD-ROM media behavior remain open behind physical drivers.
- Direct `FSDMGR_DeviceHandleToHDSK @11` and `FSDMGR_FormatVolume @15`/`FSDMGR_ScanVolume @31` imports are covered for the CE identity/no-utility-DLL paths; real format/scan utility DLL loading and disk-registry metadata remain open.
- Direct `FSDMGR_AdvertiseInterface @2` is covered for the shared coredll wrapper path, including GUID/name validation, FSDMGR empty-name root mapping, advertisement add/remove storage, configured block-device `IClass` publication/removal for `\StoreMgr\<device>`, explicit `GUID=name`, `%d`, and `%l` names with owner-scoped duplicate lifetime, coredll `EnumDeviceInterfaces @1874` enumeration of the tracked advertisement table with CE GUID/name/size/error behavior, CE message queues (`CreateMsgQueue`/`OpenMsgQueue`/`ReadMsgQueue`/`WriteMsgQueue`/`GetMsgQueueInfo`/`CloseMsgQueue`) with CE access-direction, reader/writer wait readiness, full-queue timeout, oversized-write, truncated-read, `MSGQUEUE_ALLOW_BROKEN` pipe-disconnect behavior, `MSGQUEUE_MSGALERT` priority/single-slot behavior, and Unicorn blocking `ReadMsgQueue`/`WriteMsgQueue` timeout parking, and coredll `RequestDeviceNotifications @1504`/`StopDeviceNotifications @1505` typed subscription handles with `DEVDETAIL` attach/detach delivery; real device-manager `fsdev_t` handles plus bus-backed `%b` interface names remain open.
- Message-queue import parking is implemented, but iNavi live runs still need
  proof that device-notification consumers observe startup attach events in the
  same order and wake cadence as CE while the owned splash popup remains visible.
- Direct CE `FSDMGR_GetRegistryFlag @18`, `FSDMGR_GetRegistryString @19`, and
  `FSDMGR_GetRegistryValue @20` imports are covered for the missing-registry
  fail-closed path and output clearing; real logical-disk registry-root lookup
  and utility/cache/filter DLL metadata forwarding remain open.
- Direct `FSDMGR_AsyncEnterVolume @80` and `FSDMGR_AsyncExitVolume @81` imports are covered for registered synthetic HVOL one-shot lock-token success/failure paths, including mismatched, duplicate, stale, and output-fault cleanup cases; CE mounted-volume availability, powerdown, and thread-exit wait reference behavior remains open.
- Direct `FSDMGR_ParseSecurityDescriptor @82` is covered for CE private descriptor-size extraction and validation, and coredll/AFS `GetFileSecurityW`/`SetFileSecurityW` now follow the no-security-manager mounted-path routing surface; broader file ACL persistence/enforcement remains open.
- File-change notification reset/error propagation still has deeper edge cases, but the CE `NotifyReset` pending no-fit output-count write order, fitted-record count-pointer fault drain, partial caller-buffer fault copied-prefix drain, no-pending null-returned pointer order, and all-zero no-data reset are now covered along with prefix drains, no-more-items, undersized buffers, null-buffer guarded writes, and trailing-NUL record sizing.
- Popup/menu behavior still needs broader live nested modal routing and unusual cascade cancellation coverage; the core `TrackPopupMenuEx` loop, owner notifications, keyboard/mouse selection, top-level CE screen clamping, `TPMPARAMS.rcExclude` placement fallbacks, tap-only submenu open, hover-timer submenu open, right-edge submenu flip paths, and child-submenu left/bottom screen clamping are now covered.
- Modal window behavior still needs timeout and broader live user-driven `MessageBoxW` validation; raw default/queued input, modal pump dispatch, owner focus/activation restoration after teardown, transient `MB_TOPMOST` `WS_EX_TOPMOST` capture, and the shared GWE topmost z-order group are now covered.
- Loader parity is strongest in the Unicorn runtime path. Raw/non-Unicorn loader behavior should be explicitly audited before treating all `LoadLibraryEx` variants as equivalent.
- `StringCompress`/`StringDecompress` no longer blanket-fail for the CE
  `compr2.c` all-zero, shorter raw even/odd stream, and emulator-owned opaque
  shrinkable-stream packet paths, including low-byte-only, high-byte-only,
  odd-length padded output, size-only queries, and compressed-half fail-closed
  coverage. Byte-for-byte private `CECompress`/`CEDecompress` engine parity
  remains unverified because the reviewed CE source tree exposes declarations
  and callers, but not the engine body; unknown external non-raw payloads still
  fail closed.
- `ExtEscape` now covers the CE display-driver query/protected-error surface
  for `QUERYESCSUPPORT` get/set gamma, get/set rotation, and `DISPPERF`
  queries, `GETRAWFRAMEBUFFER` queries, invalid HDCs, the full CE
  GDIAPI privileged display escape invalid-access list, and the zeroed CE-sized
  `DISPPERF_EXTESC_*` payload contract. Direct get/set gamma payloads now use
  the CE GPE `ddi_if.cpp` `cjIn`/`pvIn` ABI and `aablt.cpp` default/clamp
  range, and direct get/set screen-rotation payloads now use the CE
  VGAFLAT/SMI3DR packed supported-mask/current-mode ABI and `cjIn` mode value.
  DeviceEmulator `SETBACKLIGHT`/`GETBACKLIGHT` payloads now round-trip local
  BOOL state with the fixed four-byte buffer contract from `display_escapes.h`
  and `s3c2410x_lcd.cpp`. DeviceEmulator `CONTRASTCOMMAND` now follows the
  CE `pwingdi.h` `ContrastCmdInputParm` ABI and
  `s3c2410x_lcd.cpp::ContrastControlHelper` command semantics, including
  optional output, 0..15 clamping, the default-command return quirk, and the
  shared LCDCON3 high-nibble coupling with the backlight bit.
  DeviceEmulator `GETRAWFRAMEBUFFER` now returns the CE `gxdma.h`
  28-byte `RawFrameBufferInfo` metadata payload with RGB565 format, 16 bpp,
  the uncached framebuffer base address, byte/pitch strides, dimensions, and
  guest-readable RGB565 bytes behind the returned `pFramePointer`.
  The CE-sized `DISPPERF_EXTESC_*` payload contract now includes local nonzero
  GPE timing rows for raw `BitBlt`, `StretchBlt`, `PatBlt`, non-copy
  `MaskBlt`, CE `DrvTransparentBlt`-style `TransparentImage`, and explicit
  `ImageList_Draw*` `ILD_MASK`/`ILD_IMAGE` draw passes that CE routes through
  `StretchBlt_I`, direct-DIB `StretchDIBits`/`SetDIBitsToDevice` blit rows,
  CE `DrvAlphaBlend` `0xCCCC` blit rows with stretch accounting, CE VGAFLAT
  `ROP_LINE` timing rows for raw `LineTo`/`Polyline`, plus clear
  and unhandled behavior. Real driver escape payload handling for video
  protection and broader DISPPERF coverage outside those raw draw paths is
  still not implemented.
- PE icon extraction now exists, but malformed resource tables, uncommon icon formats, live callback-selected `KernExtractIcons` behavior, broader non-PE fallback edges, and mask/alpha fidelity need more coverage; 1bpp indexed palette extraction/rendering, 24bpp `BI_RGB` stride-padded extraction/rendering, 24bpp extracted AND-mask transparency during `DrawIconEx(DI_NORMAL)`, bitmap-backed `DrawIconEx(..., 0, 0, DI_NORMAL)` native-size selected-DIB drawing, 16bpp `BI_BITFIELDS`/RGB555 `RT_ICON` extraction/rendering, PE-extracted `DestroyIcon` owned-bitmap cleanup, raw exact integer `RT_GROUP_ICON` lookup, no-output `KernExtractIcons` failure behavior, independent requested large/small partial extraction with failed slots nulled, and non-PE index-zero single-slot fallback preservation are now covered.
- Raw `GetIconInfo` now distinguishes stock cursor handles from stock icon handles for CE `ICONINFO.fIcon` output, writes tracked bitmap-backed icon hotspot/mask/color fields, covers caller-owned bitmap deletion after `CreateIconIndirect`, covers the bitmap-backed `ImageList_GetIcon` consumer bridge, and verifies returned icon-owned bitmaps survive source `ImageList_Destroy`; broader live guest consumer paths still need validation.
- GDI/text/input fidelity remains incomplete around rendered guideline/candidate UI callbacks, broader dictionary/private-profile IME candidate sources, caret timing, broader font-size/style selection/fallback, glyph metrics outside the CE fixture ranges, complex clipping, broader palette/device-color behavior, remaining alpha orientation/clipping edges, additional overlay rendering variants, and remaining mask-format parity; the CE IMM NULL-HIMC status/composition paths, composition/candidate/status window placement round-trips, `ImmGet/SetCompositionFontW` `LOGFONTW` state, CE SIP panel registration/preference/location/input-attribute state, `ImmGetDefaultIMEWnd`/`DefaultImeWndGet` focused-window and caller-HWND proxy behavior, `ImmIsUIMessageW` forwarding for the `WM_IME_*` UI message family, `ImmGetHotKey` no-registered-hotkey output clearing, `ImmGetGuideLineW` no-guideline query/string-clearing behavior plus HIMCC-backed `GUIDELINE` level/index/string/private payload reads, `ImmLockIMC`/`ImmUnlockIMC` and `ImmLockIMCC` `INPUTCONTEXT`, `COMPOSITIONSTRING`, and `CANDIDATEINFO` lock buffers plus lock-count/size queries, `ImmUnlockIMC` readback for resized `COMPOSITIONSTRING`/`CANDIDATEINFO` IMCC payload mutations, `ImmGetCandidateListCountW`/`ImmGetCandidateListW` `CANDIDATELIST` payload/state, `ImmNotifyIME` candidate selection/page, composition-font, and status-window notification messages, TESTIME/`imm.h`-backed `ImmGetProperty` capability values plus TESTIME resource-backed `ImmGetIMEFileNameW`/`ImmGetDescriptionW` strings for IME HKLs, TESTIME zero-result `ImmGetConversionListW`, TESTIME `ImmEscapeW(IME_ESC_QUERY_SUPPORT)` self-query behavior, CE keyboard-layout loaded-list/`KLF_ACTIVATE`/`HKL_NEXT`/`HKL_PREV` transitions, CE `MapVirtualKeyW` scan-code mode behavior for LR modifier/common VK conversion and invalid mode errors, TESTIME `dic.c` single-letter candidate generation for synthetic IME HKLs, TESTIME `regword.c` fake-word style/register/unregister state plus registered-word candidate inclusion and `ImmEnumRegisterWordW` callback enumeration, TESTIME `ImmGenerateMessage` `hMsgBuf` queue flushing, all-white image-list overlay-mask zero-bounds, unmasked `ImageList_GetIcon` all-white mask initialization, non-rectangular overlay-mask paths, same-slot `ImageList_SetOverlayImage` bounds preservation, distinct CE large/small `SHGetFileInfo` system image-list handles, CE `SHGetFileInfo` icon-only `iIcon` writes plus untouched unrequested fields, system image-list pseudo-icon selected-DIB body/overlay rendering, `ImageList_SetIconSize` changed zero/negative dimension behavior, hidden-after-`ImageList_BeginDrag` drag visibility, raw `ImageList_AddMasked(CLR_DEFAULT)` and `ImageList_LoadImage(CLR_DEFAULT)` upper-left sampling, raw bitmap-backed `AddMasked` mono-mask creation/rendering, raw `BitBlt`/`MaskBlt` negative destination extent mirroring for selected-DIB source-copy paths including `MaskBlt`'s null-mask shortcut, raw `StretchBlt` signed destination/source extent mirroring for selected-DIB draws, raw `CreateDIBPatternBrushPt` packed-DIB pattern brush creation/rendering, raw direct-DIB `StretchDIBits`/`SetDIBitsToDevice` selected-memory-DIB rendering plus CE `WritableBitmapTest(EStretchDIBits/ESetDIBitsToDevice)` read-only selected-bitmap rejection and bottom-up source scanline handling, indexed 2 bpp `DIB_RGB_COLORS` source tables, and `DIB_PAL_COLORS` source palette-index tables, CE `GetDeviceCaps` bad-HDC/null-primary-HDC/invalid-index plus `DRIVERVERSION`, unlimited brush/pen/marker/font counts, rectangular `CLIPCAPS`, square-pixel `ASPECT*`, `SIZEPALETTE`, and `NUMRESERVED` behavior, CE `pal.cpp` palette entry invalid handle/parameter edges, `GetNearestColor` invalid-HDC behavior, non-paletted system-palette query behavior, stock default-palette selection plus readable stock default-palette `GetObjectW`/`GetPaletteEntries`/nearest-index behavior, CE `pal.cpp` 256-entry user-palette create/set/get/nearest-index flow, `GetCurrentObject(OBJ_PAL)` selected/default-palette and invalid-HDC/type behavior, `GetObjectType` CE `OBJ_DC`/`OBJ_MEMDC`/`OBJ_PAL` constants and invalid-handle behavior, `GetStockObject(-1)` invalid-parameter behavior, `SelectObject` invalid-HDC/null-object/bad-object behavior, raw `MaskBlt` negative destination extent mirroring for masked selected-DIB/framebuffer draws, raw `AlphaBlend` negative selected-DIB/framebuffer destination clipping, CE `WritableBitmapTest(EAlphaBlend)` read-only selected-bitmap rejection, CE `BLT_ALPHASRCNEG` source-alpha/source-constant inversion, CE `BLT_ALPHADESTNEG` 32bpp destination-alpha inversion, CE `GetCharABCWidthsI` glyph-index/count ABI, CE nonzero-escapement `GetCharABCWidths`/`GetCharABCWidthsI` rejection, CE 16px Tahoma `GetCharABCWidths*` `fontdata.h` ABC table, default Tahoma `TEXTMETRICW` metadata bytes, plain 20px CE `NTFontMetrics` rows for Tahoma/Courier New/Symbol/Times New Roman/Wingdings/Verdana, CE `font.cpp::passOddSize` known-font realized `tmHeight` rows for `lfHeight` 0 and -24, plain 16px CE `NTExtentResults` width rows for Tahoma/Courier New/Symbol/Times New Roman/Wingdings/Verdana/Arial raster, CE `SetTextCharacterExtra` advance math in text extents/drawing, CE `WritableBitmapTest(EDrawTextW/EExtTextOut)` read-only selected-bitmap rejection, CE Arial raster `SetTextCharacterExtra` rejection, selected-font `DeleteObject` lifetime preservation, zeroed and nonzero `LOGFONTW` `CreateFontIndirectW`/`GetObjectW` round-trip behavior, and raw `ExtTextOutW` DC clip-region clipping are now covered.
- GDI object-selection parity still needs broader cross-DC validation; the CE `SelectObject` simple-region `SIMPLEREGION` return/status clipping path, `SelectClipRgn` simple and complex copy/lifetime behavior plus complex `GetClipBox` status reporting, `clip.cpp` invalid-HDC/null-output validation for `IntersectClipRect`/`ExcludeClipRect`/`GetClipBox`, implicit-HDC-surface clipping for `IntersectClipRect`/`ExcludeClipRect`, `OffsetRgn` null/simple/complex status reporting, `EqualRgn` null-vs-wrong-handle error splitting, `PtInRegion`/`RectInRegion` invalid-region/null-rect error splitting, `GetRgnBox`/`CombineRgn` parameter-error edges, `GetRegionData` size/payload/error coverage, and null-region canonical bounds for create/set-region APIs are now covered.
- Gradient fill parity is currently limited to the CE-backed rectangle path: invalid HDC/pointer/count/mesh inputs, `GRADIENT_FILL_TRIANGLE` rejection, read-only selected-bitmap rejection from CE `WritableBitmapTest(EGradientFill)`, and `SHADEBLENDCAPS` alpha-plus-`SB_GRAD_RECT` reporting are covered, but any future non-rectangular gradient support needs concrete CE source or caller evidence first.
- Polygon/polyline parity now covers CE invalid-HDC, null-point, and small-count validation, including `Polyline` count-zero no-op and `Polygon` count-failure last-error behavior, selected-DIB `SetViewportOrgEx` origin translation, selected-DIB `SimpleClipRgnTest0` clip-region containment, and CE `ShapeColorTest(EPolygon)` two-point `R2_XORPEN` stroke behavior; broader stress-shape clipping, fill-mode, and pathological point-list fidelity remain open.
- Rectangle/ellipse/round-rect parity now covers CE invalid-HDC handling, raw origin/ext APIs now cover CE `SetWindowOrgEx`/`GetWindowOrgEx`/`OffsetViewportOrgEx`/`GetViewportOrgEx`/`Get*ExtEx` ordinals, raw `LineTo`/`Polyline`/`Polygon` plus `Rectangle`/`Ellipse`/`RoundRect` now apply selected-DIB viewport/window-origin translation, and selected-DIB `Rectangle`/`RoundRect` now cover CE `ShapeColorTest` `R2_XORPEN` outline behavior, but deeper stress-shape clipping and writable-bitmap fidelity remain open.
- Miscellaneous draw-API validation now covers CE `passNull2Draw` invalid-handle/invalid-parameter ordering for `RectVisible`, `FillRect`, `DrawFocusRect`, `DrawEdge`, `MoveToEx`, `LineTo`, `GetPixel`, `SetPixel`, `GetROP2`, `SetROP2`, `GetDIBColorTable`, `SetDIBColorTable`, `SetBitmapBits`, `PatBlt`, and `TransparentImage`; raw `SetBitmapBits` now allocates backing for pointerless `CreateBitmap(..., NULL)` handles before bounded writes like CE `CreateBitmapSquares*`/`SetBitmapBitsOnePixel` and rejects loaded/resource-style read-only bitmap backing like CE `WritableBitmapTest`, raw `DrawFocusRect` now covers CE framebuffer XOR toggling, raw `DrawEdge` covers CE `DrawEdgeTest1` invalid edge-type rejection, CE `DrawEdgeTest2/3` `BF_MIDDLE` center fill and `BF_FLAT` center preservation through real `GetPixel` reads, CE public partial-edge `BF_ADJUST` rectangle shrinking, and CE public `BF_DIAGONAL_END*` bounded visible diagonal rendering, raw `PatBlt` covers CE `PatBltBadRopTest` source-dependent ROP3 rejection, source-free pattern/destination ROP3 rendering, CE `TryShapes`/`PatBltSimple` zero and negative extent behavior, and CE `WritableBitmapTest(EBitBlt/EPatBlt/EStretchBlt/EMaskBlt/ETransparentImage/EGradientFill/EAlphaBlend/EStretchDIBits/ESetDIBitsToDevice/EDrawTextW/EExtTextOut)` read-only selected-bitmap rejection, raw `FillRect`, `InvertRect`, `SetPixel`, `LineTo`, `Polyline`, `Polygon`, `Rectangle`, `Ellipse`, `RoundRect`, `DrawFocusRect`, and `DrawEdge` now cover CE `WritableBitmapTest(EFillRect/EInvertRect/ESetPixel/ELineTo/EPolyline/EPolygon/ERectangle/EEllipse/ERoundRect/EDrawFocusRect/EDrawEdge)` read-only selected-bitmap rejection, raw `TransparentImage` now covers CE read-only selected-bitmap rejection, same-framebuffer black/white color-key rendering cases, CE `TransparentBltErrorTest` near-miss color-key rejection, CE `ClipBitBlt(ETransparentImage)` selected-DIB/framebuffer all-edge destination clipping coverage, and CE `TransparentBltBitmapTest` direct bitmap-handle sources, raw two-point `Polygon` now applies `R2_XORPEN` once instead of canceling itself with a reverse closing segment, and raw selected-DIB `Rectangle`/`RoundRect` outline pixels now apply `R2_XORPEN` over existing destination pixels. Deeper clip-region visibility, brush realization, broader draw-edge iteration flag rendering, viewport/current-position, additional ROP2 drawing interactions, DIB palette mutation, broader SetBitmapBits depth/format copy cases, remaining PatBlt raster-operation edges, remaining writable loaded-bitmap rejection across other draw APIs, and remaining TransparentBlt rendering/error cases remain open.
- CE `TransparentBltTransparencyTest`-style selected memory DC drawing into `CreateBitmap(..., NULL)` is now covered for the raw 1/2/4/8/16/24/32 bpp sweep: `SelectObject` materializes owned backing before `FillRect`/`SetPixel`, and `TransparentImage` observes those pixels through the source HDC. CE `TransparentImagePalTest(ETransparentImage)` duplicate color-table RGB behavior is covered for a 4 bpp selected-DIB source using the CE-accepted all-realized-RGB-transparent branch, CE `TransparentBltErrorTest` near-miss keys are covered for same-framebuffer HDC copies, and CE `ClipBitBlt(ETransparentImage)` selected-DIB/framebuffer all-edge destination clipping coverage is in place. Remaining transparent-blit clipping/error edge cases are still open.
- Stock-object parity still needs broader driver-specific edge validation, but CE `DeleteObject(GetStockObject(...))` no-op success, handle liveness, and public `BORDERX_PEN`/`BORDERY_PEN` stock pen lookup/type/select/delete coverage are now covered.
- DIB object parity still needs remaining compression variants plus remaining section-backed orientation/file-backed edge validation, but CE `CreateDIBSection` `GetObjectW` DIBSECTION-sized metadata output, BITMAP-sized fallback, 12-byte `BITMAPCOREHEADER`/RGBTRIPLE color-table creation, file-mapping section-backed bits with `dshSection`/`dsOffset` reporting, file-backed mapping initial-byte seeding and `FlushViewOfFile` write-through, shared-view synchronization, `DeleteObject` dirty-bit writeback/sibling-view synchronization, closed mapping-handle view lifetime, null-`ppvBits` creation, 24bpp `BI_BITFIELDS`-as-BGR behavior, direct-DIB `BI_RLE8`/`BI_RLE4` decoding for `StretchDIBits`/`SetDIBitsToDevice`, direct-DIB and DIB-section unsupported bit-depth rejection, bad nonzero HDC rejection, `SetDIBitsToDevice` null-DIB-payload validation ordering, direct-DIB read-only selected-bitmap rejection, null-HDC plus non-indexed/high-bpp `DIB_PAL_COLORS` rejection, and oversized indexed RGB `biClrUsed` rejection are now covered.
- iNavi route-flow completion remains open beyond process startup, shell readiness, and initial UI/window behavior.
- Detached host-launch remote-server listener lifetime is now covered by the
  optimized release path: `drive96` reproduced a false published listener on
  `127.0.0.1:8768`, then the state-owned listener plus startup `/api/v1/status`
  self-probe fix was validated by `drive97` on `127.0.0.1:8769`. The release
  run answered status/frame requests, drained a top-right tap into the active
  iNavi window as posted mouse messages, and captured real iNavi SE splash art.
- Non-fast-start Unicorn startup can take a different guest path where iNavi
  terminates the current process (`0x42`) with exit code `3` from
  `mfcce400.dll+0xd674`. Treat `WINCE_EMU_FAST_START_LIVE` as diagnostic only
  until the startup split is explained from guest code or loader behavior.
- The iNavi nearby-search G-sensor modal still indicates incomplete sensor
  initialization semantics. `YAS526B`, light-sensor, and per-I2C-bus command
  contracts have been verified, but the SMB380 command family is still
  unresolved and must come from real dump/caller evidence.
- Remote-driven iNavi startup now advances past the real `happyway_win` modal
  and keeps post-dialog taps routed to iNavi; a visible-work scheduler handoff
  also returns active state to iNavi after the Happyway modal thread handles the
  button, cross-thread visible-message handoff now skips unmapped guest WNDPROC
  targets instead of consuming/reposting them, direct-send WNDPROC cleanup
  now preserves live callouts until the saved PC reaches their return PC,
  orphaned WNDPROC return stubs can recover from the last valid return record,
  and escaped cross-thread visible-message WNDPROC callouts now restore the
  saved `ResumeImportAfterWndProc` context when the guest reaches their saved
  return PC instead of archiving a lossy orphan record. Host live-pump runs also
  pass the early
  MIPS import-thunk wall stop at `iNavi.exe+0x3554` /
  `COREDLL.dll@1047` (`memset`). The app still remains on the animated splash
  while loading resources/map data. Current traced evidence reaches
  `iNavi.exe+0x329da8` with later file/resource activity and lingering
  modal/winsock scheduler waits. The guest now reaches `COM7:` plus
  `MFS1:`/`SMB1:` device initialization, and detached `drive97` proves
  posted-touch dispatch plus non-black splash rendering through the remote
  server. A later full trace-mode run on `192.168.0.39:8765` stayed on the real
  iNavi SE splash while still advancing through `resmapi_800x480.bin`
  decompression at `iNavi.exe+0x2ff8xx`/`+0x2ff9xx`, and post-flamegraph
  no-trace runs now reach the later `iNavi.exe+0x329cxx` resource/map-loading
  region. Timer state is exposed through `/api/v1/debug/timers.txt`, but the
  normal map UI transition is still not observed. The old linear
  trampoline-origin helper no longer appears in the post-`4559d704`
  Windows-sudo flamegraph, and mapped-code indexing no longer clones every
  mapped blob byte vector per run slice; current sampled startup cost is Unicorn
  TCG translation/execution plus code-hook callbacks. Timer sampling while the
  splash is visible shows no pending timers. Raw import stack-context tracing
  now resolves the initial owned splash `ShowWindow(SW_SHOW)` to app-side stack
  candidates around `iNavi.exe+0x4d7a0`, with the concrete show-wrapper call at
  `0x0005e85c -> 0x0048e998` using `cmd=5`; no matching hide, destroy, or
  z-order demotion for `0x00020008` has been observed.
- Hidden Happyway child scheduling remains incomplete. The real modal dialog
  can be dismissed and its framebuffer pixels restore correctly, but
  `happyway_win.exe` may remain parked with stale modal/close state while iNavi
  stays on the animated splash/resource-loading path.
- Sensor emulation is partially observed by the guest app. The remote endpoints
  queue GPS/IMU data, `serial_devices.json` can now mark a config-selected
  `remote_gps` serial port, live `drive29` confirms queued NMEA drains when
  iNavi opens `COM7:`, and raw `GetCommModemStatus` now reports asserted
  CTS/DSR/RLSD for serial handles. Current live evidence shows `COM7:` remains
  open as the remote GPS target, but new REST GPS posts can sit queued while the
  guest is loading map data and not issuing further serial reads; the exact
  `ReadFile`/`WaitCommEvent` cadence and parsed-position consumption still need
  proof.

## Build And Validation Risks

- The normal validation profile uses `--features unicorn,trace,win32-desktop`. No-feature test support needs an explicit decision and cfg audit before it can be treated as a required gate.
- `registry.reg` loading accepts REGEDIT text through UTF-8/lossy decoding and
  decodes typed `hex(2)`/`hex(7)` values as UTF-16. If a future REGEDIT4 export
  depends on a non-UTF-8 ANSI code page for quoted string values, those strings
  may need code-page-aware decoding.
- `git diff --check` may report CRLF warnings on existing files. Treat non-CRLF whitespace findings as actionable.

## Recently Closed From Source State

- `ExtractIconExW` no longer appears to be synthetic-only: current source reads PE resources, chooses an icon group, builds color/mask bitmaps, creates icon handles, and falls back to shell icons for index zero.
- `GetFileVersionInfoSizeW`/`GetFileVersionInfoW` no longer report universal absence of version resources: current source reads integer `RT_VERSION/VS_VERSION_INFO` resources from PE files, validates `VS_FFI_SIGNATURE`, reports malformed data with `ERROR_INVALID_DATA`, and rewrites the copied `VERHEAD.wTotLen` to the bounded copy length.
- File-change notification coalescing now handles duplicate records, transient create/delete churn, and modified/delete collapse.
- Destroying a cross-thread `SendMessageTimeout` target now writes a zero result to `lpdwResult` for the completed destroyed-target case.
- System and hidden mounted-volume attributes are now source-backed: nested
  system-volume files inherit `FILE_ATTRIBUTE_SYSTEM`, hidden mounts are skipped
  from root enumeration, exact hidden mount probes keep `FILE_ATTRIBUTE_HIDDEN`,
  and `CeGetVolumeInfoW` exposes system/hidden volume attributes.
- CE input-method samples that pass `NULL` to `ImmGetOpenStatus`,
  `ImmSetOpenStatus`, `ImmGetConversionStatus`, and `ImmSetConversionStatus`
  are now covered by resolving the call through the current foreground keyboard
  target's HIMC instead of treating `HIMC == 0` as an invalid explicit handle.
- CE input-method composition probes that pass `NULL` to
  `ImmGetCompositionStringW(GCS_COMPSTR)` and
  `ImmSetCompositionStringW(SCS_SETSTR)` are now covered by resolving the call
  through the same foreground keyboard target HIMC.
- CE `COMPOSITIONFORM` and `CANDIDATEFORM` placement round-trips are now
  covered for `ImmGet/SetCompositionWindow` and `ImmGet/SetCandidateWindow`,
  including active `HIMC == NULL` resolution and candidate-index validation.
- CE `ImmNotifyIME` candidate notification actions now post `WM_IME_NOTIFY`
  open/change/close messages with candidate-list bit masks, while still
  preserving unsupported notification actions as successful no-ops.
- CE `ImmGetProperty` capability probes for IME HKLs now return TESTIME and
  `imm.h`-backed version, property, conversion, UI, select, set-composition,
  sentence, and private-data-size values, while non-IME HKLs still return zero.
- CE `ImmGet/SetStatusWindowPos` now round-trips the HIMC status-window point,
  and CE status-window `ImmNotifyIME` context updates now post open, close, and
  position-change `WM_IME_NOTIFY` messages.
- CE `ImmGetIMEFileNameW` and `ImmGetDescriptionW` now return TESTIME
  resource-backed strings for synthetic IME HKLs, while non-IME HKLs still
  return empty strings.
- TESTIME candidate generation now rejects combined built-in/registered-word/
  private-profile lists above the CE `MAXCANDSTRNUM` ceiling of 32.
- TESTIME private-profile lookup now hides registered-word entries whose
  readings contain lowercase characters, matching the sample stub's section
  guard.
- TESTIME registry-value private-profile candidate lookup is covered, and
  kernel boot now seeds the bundled sample `testime.reg` dictionary into the
  active registry-value enumeration shape.
