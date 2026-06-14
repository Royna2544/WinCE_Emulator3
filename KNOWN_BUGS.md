# Known Bugs And Risks

Regenerated on 2026-06-11 from current source and test evidence. Items here are unresolved issues, unverified behavior, or risk areas that should not be presented as complete.

## Open Issues

- `SendMessageTimeout` is not yet proven complete for broader reentrant waits and all scheduler wait combinations; zero-timeout cross-thread raw expiry, nonzero timeout stale-delivery cleanup, Unicorn block-prep refusal, below-threshold vs hung `SMTO_ABORTIFHUNG`, combined `SMTO_BLOCK | SMTO_ABORTIFHUNG` hung abort, same-thread abort-if-hung non-abort, early/nested `ReplyMessage`, and active-outer-timeout/nested-send lifetime coverage are covered.
- Shell notification behavior needs integrated validation for Explorer/taskbar `IShellNotificationCallback` dispatch through a real guest COM object lifecycle, taskbar-visible state, broader handle ownership, and remove/update races; the raw stale registered-taskbar, `SHNN_SHOW` window-only taskbar path, GUID-value OLE `CoCreateInstance` import writeback, local callback-acquisition queue path, unregistered callback-CLSID sink fallback, overlong taskbar-title clearing, mapped guest vtable callout, unmapped callback-pointer retry, and iconic remove/sink-cleanup icon lifetime paths are now covered.
- Mounted storage behavior still needs coverage for physical block-driver backing, broader external cache DLL/filter behavior, remaining specialized disk IOCTLs, real external-copy accelerator behavior, and real mounted-FSD `FsIoControl` hook forwarding beyond the host-backed unsupported stub; the `fsdmgr.dll` `FSINT_*`/`FSEXT_*` notification import surface, FSDMGR cache imports (`FSDMGR_CacheIoControl @3`, `FSDMGR_CachedRead @4`, `FSDMGR_CachedWrite @5`, `FSDMGR_CreateCache @6`, `FSDMGR_DeleteCache @9`, `FSDMGR_FlushCache @14`, `FSDMGR_InvalidateCache @24`, `FSDMGR_ResizeCache @30`, `FSDMGR_SyncCache @32`), direct sparse-sector disk read/write support through `FSDMGR_DiskIoControl @12`, `FSDMGR_ReadDisk @25`, `FSDMGR_ReadDiskEx @26`, `FSDMGR_WriteDisk @35`, and `FSDMGR_WriteDiskEx @36`, basic disk metadata/status IOCTLs, `GET_SECTOR_ADDR` validation/no-XIP unsupported behavior, `GETPMTIMINGS` zero timing snapshots, secure-wipe sector clearing and set-secure-wipe-flag validation/no-op behavior, copy-external-start `DISK_COPY_EXTERNAL` payload validation/unsupported no-touch behavior, `DELETE_SECTOR_INFO` sparse-sector clearing, `FSDMGR_CreateFileHandle @7`, `FSDMGR_CreateSearchHandle @8`, `FSDMGR_DeregisterVolume @10`, `FSDMGR_GetVolumeHandle @21`, `FSDMGR_GetVolumeName @22`, `FSDMGR_RegisterVolume @27`, `FSDMGR_GetMountFlags @37`, `STOREMGR_FsIoControlW @44` refresh/flush no-op and unsupported host-backed no-touch failure behavior, `FSCTL_COPY_EXTERNAL_START`/`COMPLETE` fixed `FILE_COPY_EXTERNAL` validation/unsupported no-touch behavior, file-handle `FSCTL_SET_FILE_CACHE(FileCacheDisableStandard)` no-op success plus unsupported enable/discard and volume-level rejection, FSDMGR first-change import owner regression, internal `FSINT_*` notification-handle owner/reset/info/close split, internal-vs-external wrong-handle close split, public direct `CloseHandle` owner rejection, direct AFS `hProc` owner close/duplicate rejection, raw AFS volume-handle owner/unmount/close signaling, volume-handle/FSDMGR-import `FSCTL_GET_VOLUME_INFO` metadata, FSDMGR mount name/AFS flag metadata, FSDMGR disk-pointer-to-HVOL mapping, FSD context-pointer file/search handle shims, CE null-cache fallback ID/status behavior, cross-mounted file-change isolation, same-parent and cross-parent mounted rename scoping, target-process duplicate ownership, write-close completion records, read-only-source cross-volume move copy/delete semantics, DeleteAndRename/PrestoChango direction, read-only/removable volume attributes, and system/hidden/permanent mount attributes are now covered for mounted roots.
- File-change notification reset/error propagation still has deeper edge cases, but the CE `NotifyReset` pending no-fit output-count write order, fitted-record count-pointer fault drain, partial caller-buffer fault copied-prefix drain, no-pending null-returned pointer order, and all-zero no-data reset are now covered along with prefix drains, no-more-items, undersized buffers, null-buffer guarded writes, and trailing-NUL record sizing.
- Loader parity is strongest in the Unicorn runtime path. Raw/non-Unicorn loader behavior should be explicitly audited before treating all `LoadLibraryEx` variants as equivalent.
- `StringCompress`/`StringDecompress` no longer blanket-fail for the CE
  `compr2.c` all-zero and shorter raw even/odd stream packet paths, including
  low-byte-only, high-byte-only, odd-length padded output, size-only queries,
  and compressed-half fail-closed coverage, but the opaque
  `CECompress`/`CEDecompress` payload branch is still not implemented.
- `ExtEscape` now covers the CE display-driver query/protected-error surface
  for `QUERYESCSUPPORT`, unsupported raw-framebuffer queries, invalid HDCs,
  privileged display escapes, and the zeroed CE-sized `DISPPERF_EXTESC_*`
  payload contract, but real driver escape payload handling for gamma,
  rotation, video protection, framebuffer access, and nonzero display-performance
  instrumentation counters is still not implemented.
- PE icon extraction now exists, but malformed resource tables, uncommon icon formats, live callback-selected `KernExtractIcons` behavior, broader non-PE fallback edges, and mask/alpha fidelity need more coverage; 1bpp indexed palette extraction/rendering, 24bpp `BI_RGB` stride-padded extraction/rendering, 24bpp extracted AND-mask transparency during `DrawIconEx(DI_NORMAL)`, 16bpp `BI_BITFIELDS`/RGB555 `RT_ICON` extraction/rendering, PE-extracted `DestroyIcon` owned-bitmap cleanup, raw exact integer `RT_GROUP_ICON` lookup, no-output `KernExtractIcons` failure behavior, and non-PE index-zero single-slot fallback preservation are now covered.
- GDI/text/input fidelity remains incomplete around rendered guideline/candidate UI callbacks, broader dictionary/private-profile IME candidate sources, caret timing, broader font-size/style selection/fallback, glyph metrics outside the CE fixture ranges, complex clipping, palette state beyond direct source palette indexes, remaining alpha orientation/clipping edges, additional overlay rendering variants, and remaining mask-format parity; the CE IMM NULL-HIMC status/composition paths, composition/candidate/status window placement round-trips, `ImmGet/SetCompositionFontW` `LOGFONTW` state, CE SIP panel registration/preference/location/input-attribute state, `ImmGetDefaultIMEWnd`/`DefaultImeWndGet` focused-window and caller-HWND proxy behavior, `ImmIsUIMessageW` forwarding for the `WM_IME_*` UI message family, `ImmGetHotKey` no-registered-hotkey output clearing, `ImmGetGuideLineW` no-guideline query/string-clearing behavior plus HIMCC-backed `GUIDELINE` level/index/string/private payload reads, `ImmLockIMC`/`ImmUnlockIMC` and `ImmLockIMCC`/`ImmUnlockIMCC` `INPUTCONTEXT`, `COMPOSITIONSTRING`, and `CANDIDATEINFO` lock buffers plus lock-count/size queries, `ImmUnlockIMC` readback for resized `COMPOSITIONSTRING`/`CANDIDATEINFO` IMCC payload mutations, `ImmGetCandidateListCountW`/`ImmGetCandidateListW` `CANDIDATELIST` payload/state, `ImmNotifyIME` candidate selection/page, composition-font, and status-window notification messages, TESTIME/`imm.h`-backed `ImmGetProperty` capability values plus TESTIME resource-backed `ImmGetIMEFileNameW`/`ImmGetDescriptionW` strings for IME HKLs, TESTIME zero-result `ImmGetConversionListW`, TESTIME `ImmEscapeW(IME_ESC_QUERY_SUPPORT)` self-query behavior, CE keyboard-layout loaded-list/`KLF_ACTIVATE`/`HKL_NEXT`/`HKL_PREV` transitions, TESTIME `dic.c` single-letter candidate generation for synthetic IME HKLs, TESTIME `regword.c` fake-word style/register/unregister state plus registered-word candidate inclusion and `ImmEnumRegisterWordW` callback enumeration, TESTIME `ImmGenerateMessage` `hMsgBuf` queue flushing, all-white image-list overlay-mask zero-bounds, non-rectangular overlay-mask paths, same-slot `ImageList_SetOverlayImage` bounds preservation, system image-list pseudo-icon selected-DIB body/overlay rendering, `ImageList_SetIconSize` changed zero/negative dimension behavior, hidden-after-`ImageList_BeginDrag` drag visibility, raw `ImageList_AddMasked(CLR_DEFAULT)` and `ImageList_LoadImage(CLR_DEFAULT)` upper-left sampling, raw bitmap-backed `AddMasked` mono-mask creation/rendering, raw `BitBlt`/`MaskBlt` negative destination extent mirroring for selected-DIB source-copy paths including `MaskBlt`'s null-mask shortcut, raw `StretchBlt` signed destination/source extent mirroring for selected-DIB draws, raw `CreateDIBPatternBrushPt` packed-DIB pattern brush creation/rendering, raw direct-DIB `StretchDIBits`/`SetDIBitsToDevice` selected-memory-DIB rendering plus bottom-up source scanline handling, indexed 2 bpp `DIB_RGB_COLORS` source tables, and `DIB_PAL_COLORS` source palette-index tables, raw `MaskBlt` negative destination extent mirroring for masked selected-DIB/framebuffer draws, raw `AlphaBlend` negative selected-DIB/framebuffer destination clipping, CE `BLT_ALPHASRCNEG` source-alpha/source-constant inversion, CE `BLT_ALPHADESTNEG` 32bpp destination-alpha inversion, CE `GetCharABCWidthsI` glyph-index/count ABI, CE nonzero-escapement `GetCharABCWidths` rejection, CE 16px Tahoma `GetCharABCWidths*` `fontdata.h` ABC table, default Tahoma `TEXTMETRICW` metadata bytes, plain 20px CE `NTFontMetrics` rows for Tahoma/Courier New/Symbol/Times New Roman/Wingdings/Verdana, CE `font.cpp::passOddSize` known-font realized `tmHeight` rows for `lfHeight` 0 and -24, plain 16px CE `NTExtentResults` width rows for Tahoma/Courier New/Symbol/Times New Roman/Wingdings/Verdana/Arial raster, CE `SetTextCharacterExtra` advance math in text extents/drawing, CE Arial raster `SetTextCharacterExtra` rejection, selected-font `DeleteObject` lifetime preservation, zeroed and nonzero `LOGFONTW` `CreateFontIndirectW`/`GetObjectW` round-trip behavior, and raw `ExtTextOutW` DC clip-region clipping are now covered.
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
  button, and direct-send WNDPROC cleanup now preserves live callouts until the
  saved PC reaches their return PC. The app still remains on the animated
  splash while loading resources/map data. Current evidence points at repeated
  file-position/resource loading around `iNavi.exe+0x642f8` (`SetFilePointer`,
  `COREDLL` ordinal 173) after map/resource files are opened. The guest now
  reaches `COM7:` plus `MFS1:`/`SMB1:` device initialization, and detached
  `drive97` proves posted-touch dispatch plus non-black splash rendering through
  the remote server. Normal map UI transition is still not observed.
- Hidden Happyway child scheduling remains incomplete. The real modal dialog
  can be dismissed and its framebuffer pixels restore correctly, but
  `happyway_win.exe` may remain parked with stale modal/close state while iNavi
  stays on the animated splash/resource-loading path.
- Sensor emulation is partially observed by the guest app. The remote endpoints
  queue GPS/IMU data, `serial_devices.json` can now mark a config-selected
  `remote_gps` serial port, and live `drive29` confirms queued NMEA drains when
  iNavi opens `COM7:`. Remaining risk: subsequent GPS posts can queue while the
  guest is sleeping/polling other workers, so the exact serial read/event
  cadence and parsed-position consumption still need proof.

## Build And Validation Risks

- The normal validation profile uses `--features unicorn,trace,win32-desktop`. No-feature test support needs an explicit decision and cfg audit before it can be treated as a required gate.
- `registry.reg` loading accepts REGEDIT text through UTF-8/lossy decoding and
  decodes typed `hex(2)`/`hex(7)` values as UTF-16. If a future REGEDIT4 export
  depends on a non-UTF-8 ANSI code page for quoted string values, those strings
  may need code-page-aware decoding.
- `git diff --check` may report CRLF warnings on existing files. Treat non-CRLF whitespace findings as actionable.

## Recently Closed From Source State

- `ExtractIconExW` no longer appears to be synthetic-only: current source reads PE resources, chooses an icon group, builds color/mask bitmaps, creates icon handles, and falls back to shell icons for index zero.
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
