# TODO

Regenerated on 2026-06-11 from current source and test coverage.

## Immediate Engineering Queue

- Add PE icon extraction edge tests for remaining PE resource variants and non-PE fallback paths; 1bpp indexed palette extraction/rendering, 24bpp `BI_RGB` stride-padded extraction/rendering, 24bpp extracted AND-mask transparency during `DrawIconEx(DI_NORMAL)`, 16bpp `BI_BITFIELDS`/RGB555 `RT_ICON` extraction/rendering, `DestroyIcon` cleanup of PE-extracted owned color/mask backing, raw `KernExtractIcons` exact integer `RT_GROUP_ICON` lookup, no-output failure behavior, and non-PE index-zero single-slot fallback preservation are now covered.
- Expand `DrawIconEx`, `ImageList_DrawEx`, and `ImageList_DrawIndirect` tests for remaining mask formats, palette variants, additional overlay rendering variants, stretched draws, framebuffer output, and additional cleanup/error lifetimes; the CE all-white overlay-mask zero-bounds, non-rectangular overlay-mask, same-slot `ImageList_SetOverlayImage` bounds-preservation, system image-list pseudo-icon selected-DIB body/overlay rendering, `ImageList_SetIconSize` changed zero/negative dimension branch, hidden-after-`ImageList_BeginDrag` drag-state behavior, raw `ImageList_AddMasked(CLR_DEFAULT)` and raw `ImageList_LoadImage(CLR_DEFAULT)` upper-left sampling, and raw bitmap-backed `AddMasked` mono-mask creation/rendering are now covered, raw `BitBlt`/`MaskBlt` now cover CE negative destination extent mirroring for source-copy selected-DIB draws including `MaskBlt`'s null-mask shortcut, raw `StretchBlt` now covers selected-DIB signed destination/source extent mirroring, raw `CreateDIBPatternBrushPt` now builds private bitmap-backed pattern brushes from packed DIB data, raw direct-DIB `StretchDIBits`/`SetDIBitsToDevice` now render into selected memory DIBs with CE bad-HDC/ROP4 validation, bottom-up source scanline handling, indexed 2 bpp `DIB_RGB_COLORS` source tables, and `DIB_PAL_COLORS` palette-index source tables, raw `MaskBlt` also covers masked selected-DIB/framebuffer signed-destination draws, and raw `AlphaBlend` now covers negative selected-DIB/framebuffer destination clipping, CE `BLT_ALPHASRCNEG` source-alpha/source-constant inversion, and CE `BLT_ALPHADESTNEG` 32bpp destination-alpha inversion.
- Implement and test remaining `SendMessageTimeout`/message-wait semantics: broader reentrant waits and multi-wait interactions beyond the already covered below-threshold/hung abort-if-hung split, combined `SMTO_BLOCK | SMTO_ABORTIFHUNG` hung abort, same-thread abort-if-hung non-abort, early/nested `ReplyMessage`, active-outer-timeout nested-send paths, and `MsgWaitForMultipleObjectsEx` message-wake indexing after multiple unsignaled handles.
- Extend remaining popup/menu placement coverage beyond the covered `TPMPARAMS.rcExclude` below-placement, right-side fallback, left-side fallback, and all-candidates-intersect fallback cases, especially live nested modal routing and unusual clamped-screen combinations.
- Verify `Shell_NotifyIcon` and `SHNotification*` in an integrated Unicorn run, including Explorer/taskbar `IShellNotificationCallback` dispatch through a real guest COM object lifecycle, broader remove/update race behavior, and runtime invalid-handle behavior beyond the raw stale-taskbar, raw `SHNN_SHOW` window-only, GUID-value OLE `CoCreateInstance` import writeback, local callback-acquisition queue, unregistered callback-CLSID sink fallback, overlong taskbar-title clearing, and mapped vtable callout paths.
- Extend `MessageBoxW` live modal coverage beyond raw default/queued input and the covered `MB_TOPMOST` transient `WS_EX_TOPMOST` style, especially foreground/topmost z-order behavior with live windows and timeout/pump edges.
- Extend file-change notification/storage coverage beyond the now-trapped `fsdmgr.dll` `FSINT_*`/`FSEXT_*` notification import surface, FSDMGR cache imports (`FSDMGR_CacheIoControl @3`, `FSDMGR_CachedRead @4`, `FSDMGR_CachedWrite @5`, `FSDMGR_CreateCache @6`, `FSDMGR_DeleteCache @9`, `FSDMGR_FlushCache @14`, `FSDMGR_InvalidateCache @24`, `FSDMGR_ResizeCache @30`, `FSDMGR_SyncCache @32`), direct sparse-sector `FSDMGR_DiskIoControl @12`, `FSDMGR_ReadDisk @25`, `FSDMGR_ReadDiskEx @26`, `FSDMGR_WriteDisk @35`, and `FSDMGR_WriteDiskEx @36` support, basic disk metadata/status IOCTLs, `GET_SECTOR_ADDR` validation/no-XIP unsupported behavior, `GETPMTIMINGS` zero timing snapshots, secure-wipe sector clearing and set-secure-wipe-flag validation/no-op behavior, copy-external-start `DISK_COPY_EXTERNAL` payload validation/unsupported no-touch behavior, `FSDMGR_CreateFileHandle @7`, `FSDMGR_CreateSearchHandle @8`, `FSDMGR_DeregisterVolume @10`, `FSDMGR_GetVolumeHandle @21`, `FSDMGR_GetVolumeName @22`, `FSDMGR_RegisterVolume @27`, `FSDMGR_GetMountFlags @37`, `STOREMGR_FsIoControlW @44`, FSDMGR first-change import owner regression, internal `FSINT_*` notification-handle owner/reset/info/close split, public direct `CloseHandle` owner rejection, direct AFS `hProc` owner close/duplicate rejection, raw AFS volume-handle owner/unmount/close signaling, volume-handle/FSDMGR-import `FSCTL_GET_VOLUME_INFO` metadata, FSDMGR `STOREMGR_FsIoControlW` refresh/flush no-op and unsupported host-backed no-touch failure behavior, `FSCTL_COPY_EXTERNAL_START`/`COMPLETE` fixed `FILE_COPY_EXTERNAL` validation/unsupported no-touch behavior, file-handle `FSCTL_SET_FILE_CACHE(FileCacheDisableStandard)` no-op success plus unsupported enable/discard and volume-level rejection, FSDMGR mount name/AFS flag metadata, FSDMGR disk-pointer-to-HVOL mapping, FSD context-pointer handle shims, CE null-cache fallback ID/status behavior, and `DELETE_SECTOR_INFO` sparse-sector clearing into physical block-driver backing, broader external cache DLL/filter behavior, remaining specialized disk IOCTLs, real external-copy accelerator behavior, and real mounted-FSD `FsIoControl` hook forwarding beyond the host-backed unsupported stub.
- Continue narrowing `CeGetFileNotificationInfo` reset/error propagation beyond the covered prefix drains, no-more-items, undersized buffers, pending no-fit output-count write order, fitted-record count-pointer fault drain, partial caller-buffer fault copied-prefix drain, no-pending null-returned pointer order, all-zero no-data reset, null-buffer guarded writes, and trailing-NUL record sizing.
- Extend GDI/text/IME coverage beyond the now-covered CE NULL-HIMC open/conversion/composition status resolution, CE IMM composition/candidate/status window form and point round-trips, CE `ImmGet/SetCompositionFontW` `LOGFONTW` state, CE SIP panel registration/preference/location/input-attribute state, CE `ImmGetDefaultIMEWnd`/`DefaultImeWndGet` focused-window and caller-HWND proxy behavior, CE `ImmIsUIMessageW` forwarding for the `WM_IME_*` UI message family, CE `ImmGetHotKey` no-registered-hotkey output clearing, CE `ImmGetGuideLineW` no-guideline query/string-clearing behavior and HIMCC-backed `GUIDELINE` level/index/string/private payload reads, CE `ImmLockIMC`/`ImmUnlockIMC` and `ImmLockIMCC`/`ImmUnlockIMCC` `INPUTCONTEXT`, `COMPOSITIONSTRING`, and `CANDIDATEINFO` lock buffers plus lock-count/size queries, CE `ImmUnlockIMC` readback for resized `COMPOSITIONSTRING`/`CANDIDATEINFO` IMCC payload mutations, CE `ImmGetCandidateListCountW`/`ImmGetCandidateListW` `CANDIDATELIST` payload/state, CE `ImmNotifyIME` candidate selection/page, composition-font, and status-window notification messages, TESTIME/`imm.h`-backed `ImmGetProperty` capability values plus TESTIME resource-backed `ImmGetIMEFileNameW`/`ImmGetDescriptionW` strings for IME HKLs, TESTIME zero-result `ImmGetConversionListW`, TESTIME `ImmEscapeW(IME_ESC_QUERY_SUPPORT)` self-query behavior, CE keyboard-layout loaded-list/`KLF_ACTIVATE`/`HKL_NEXT`/`HKL_PREV` transitions, TESTIME `dic.c` single-letter candidate generation for synthetic IME HKLs, TESTIME `regword.c` fake-word style/register/unregister state plus registered-word candidate inclusion and `ImmEnumRegisterWordW` callback enumeration, TESTIME `testime.reg` sample dictionary seeding for the active registry-value candidate path, TESTIME `ImmGenerateMessage` `hMsgBuf` queue flushing, CE `GetCharABCWidthsI` glyph-index/count ABI, CE nonzero-escapement `GetCharABCWidths` rejection, CE 16px Tahoma `GetCharABCWidths*` `fontdata.h` ABC table, default Tahoma `TEXTMETRICW` metadata bytes, plain 20px CE `NTFontMetrics` rows for Tahoma/Courier New/Symbol/Times New Roman/Wingdings/Verdana, CE `font.cpp::passOddSize` known-font realized `tmHeight` rows for `lfHeight` 0 and -24, plain 16px CE `NTExtentResults` width rows for Tahoma/Courier New/Times New Roman/Wingdings/Verdana/Arial raster, `SetTextCharacterExtra` positive and negative advance math in `GetTextExtentExPointW`, `DrawTextW`, and `ExtTextOutW`, CE Arial raster `SetTextCharacterExtra` rejection, selected-font `DeleteObject` lifetime preservation, zeroed and nonzero `LOGFONTW` `CreateFontIndirectW`/`GetObjectW` round-trip behavior, and `ExtTextOutW` DC clip-region clipping, especially rendered guideline/candidate UI callbacks, broader font-size/style selection/fallback, glyph fidelity outside the fixture ranges, complex clipping, and additional font lifetime edges.
- Keep TESTIME dictionary/private-profile work focused beyond the now-covered single-letter table, registered-word inclusion/enumeration, registry-value reads, sample `testime.reg` seed reconciliation, and `MAXCANDSTRNUM` oversized candidate rejection.
- Keep TESTIME private-profile parity moving beyond the now-covered uppercase-section visibility guard for registered-word candidate lookup and Unicorn `ImmEnumRegisterWordW` enumeration.
- Audit runtime loader behavior outside Unicorn so raw tests and runtime mapping behavior stay intentionally aligned.
- Continue iNavi route-flow work from the current process/window/shell readiness point into destination search and map interaction.
- Re-run the iNavi UI crawl on a host/remote launch after fixing the detached
  remote-server listener lifetime; validate that the new target-thread wake path
  lets top-right menu/search taps reach and advance the live iNavi UI.
- Fix the remote server accept-thread/lifetime regression before further
  unattended UI driving. Current `drive93` evidence: `RemoteServer::start`
  prints `remote server: http://192.168.0.39:8765`, the emulator process stays
  alive for at least 10 seconds, but `Get-NetTCPConnection -LocalPort 8765`
  reports no listener and HTTP requests are refused.
- Continue isolating the fast-start startup split. Without fast-start, a bounded
  iNavi CPU run exits via the CE current-process pseudo-handle (`0x42`) with
  code `3` at `mfcce400.dll+0xd674`; with `WINCE_EMU_FAST_START=1` it stays in
  startup. `WINCE_EMU_FAST_START_LIVE=1` now allows testing that behavior in
  live-pump mode, but the remote listener issue currently prevents UI driving.
- Continue the remote-driven iNavi startup investigation from the current
  resource/map-loading state: the Happyway `MessageBoxW` is dismissed through
  the real button, stale unowned modal waits no longer rotate the active
  process, visible-work handoff returns scheduling to iNavi after Happyway
  handles the modal button, post-dialog taps reach iNavi, and the app opens/reads
  mapdata, resource, font, and config files. Determine why the animated splash
  does not transition to the map UI, starting from the repeated `SetFilePointer`
  (`COREDLL` ordinal 173) path around `iNavi.exe+0x642f8` and the continuing
  `resi_800x480.bin`/mapdata reads.
- Continue the Happyway hidden-child scheduling investigation: modal dismissal
  and framebuffer restoration are fixed, and the active-process self-park
  duplicate is filtered, but `happyway_win.exe` can still remain parked with
  stale modal/close state while iNavi continues splash/resource loading.
- Continue sensor-consumption tracing after the config-driven remote GPS target
  fix: the live iNavi run now opens `COM7:` as the remote GPS target and opens
  `MFS1:`/`SMB1:` for motion sensors, but repeated location posts can still
  queue until the guest's next serial read/event path. Capture the concrete
  `ReadFile`/`WaitCommEvent` cadence and confirm whether map/UI code consumes
  parsed GPS position after startup.
- Decode the SMB380/G-sensor initialization contract from actual dump code or a
  real caller trace before adding accelerometer commands; do not use the
  unverified `0xb100...` family as SMB380 evidence.

## Cleanup Queue

- Keep registry fixtures and launch docs on `registry.reg`; any explicit
  legacy JSON parser regression should create a temporary JSON fixture instead
  of restoring the removed `regs.json`.
- Decide whether no-feature `cargo test` is a supported build profile. If yes, audit feature-gated Unicorn references and add it to validation.
- Keep `SOURCE_REFERENCES.md` tied to actual source behavior when implementing new slices.
- Avoid reintroducing long historical progress logs into roadmap files; use these files for current state only.

## Validation Queue

- For shell and GDI work: run `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel`.
- For GWE/message work: run `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe`.
- For file/storage work: run `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`.
- For shared scheduler/resource work: run `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems`.
- Before handing off a larger slice: run `cargo check --features unicorn,trace,win32-desktop` and `git diff --check`.
