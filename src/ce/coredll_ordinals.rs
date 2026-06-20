use std::{collections::BTreeMap, sync::OnceLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoredllOrdinalDef {
    pub name: &'static str,
    pub ordinal: u32,
}

pub const COREDLL_EXPORTS: &[CoredllOrdinalDef; 1530] = &[
    CoredllOrdinalDef {
        name: "??2@YAPAXI@Z",
        ordinal: 1095,
    },
    CoredllOrdinalDef {
        name: "??2@YAPAXIABUnothrow_t@std@@@Z",
        ordinal: 1646,
    },
    CoredllOrdinalDef {
        name: "??3@YAXPAX@Z",
        ordinal: 1094,
    },
    CoredllOrdinalDef {
        name: "??3@YAXPAXABUnothrow_t@std@@@Z",
        ordinal: 1662,
    },
    CoredllOrdinalDef {
        name: "??_U@YAPAXI@Z",
        ordinal: 1456,
    },
    CoredllOrdinalDef {
        name: "??_U@YAPAXIABUnothrow_t@std@@@Z",
        ordinal: 1661,
    },
    CoredllOrdinalDef {
        name: "??_V@YAXPAX@Z",
        ordinal: 1457,
    },
    CoredllOrdinalDef {
        name: "??_V@YAXPAXABUnothrow_t@std@@@Z",
        ordinal: 1663,
    },
    CoredllOrdinalDef {
        name: "?DefaultImcGet@@YAKXZ",
        ordinal: 1218,
    },
    CoredllOrdinalDef {
        name: "?DefaultImeWndGet@@YAPAUHWND__@@XZ",
        ordinal: 1219,
    },
    CoredllOrdinalDef {
        name: "?ImmGetUIClassName@@YAPAGXZ",
        ordinal: 1223,
    },
    CoredllOrdinalDef {
        name: "?ImmProcessKey@@YAKPAUHWND__@@IJKI@Z",
        ordinal: 1220,
    },
    CoredllOrdinalDef {
        name: "?ImmSetActiveContext@@YAHPAUHWND__@@KH@Z",
        ordinal: 806,
    },
    CoredllOrdinalDef {
        name: "?ImmTranslateMessage@@YAHPAUHWND__@@IIJHIIPAH@Z",
        ordinal: 1221,
    },
    CoredllOrdinalDef {
        name: "?_Nomemory@std@@YAXXZ",
        ordinal: 1660,
    },
    CoredllOrdinalDef {
        name: "?_Xlen@std@@YAXXZ",
        ordinal: 1658,
    },
    CoredllOrdinalDef {
        name: "?_Xran@std@@YAXXZ",
        ordinal: 1659,
    },
    CoredllOrdinalDef {
        name: "?__set_inconsistency@@YAP6AXXZP6AXXZ@Z",
        ordinal: 1555,
    },
    CoredllOrdinalDef {
        name: "__security_gen_cookie",
        ordinal: 1875,
    },
    CoredllOrdinalDef {
        name: "__security_gen_cookie2",
        ordinal: 2696,
    },
    CoredllOrdinalDef {
        name: "?_inconsistency@@YAXXZ",
        ordinal: 1558,
    },
    CoredllOrdinalDef {
        name: "?_query_new_handler@@YAP6AHI@ZXZ",
        ordinal: 1618,
    },
    CoredllOrdinalDef {
        name: "?_query_new_mode@@YAHXZ",
        ordinal: 1649,
    },
    CoredllOrdinalDef {
        name: "?_set_new_handler@@YAP6AHI@ZP6AHI@Z@Z",
        ordinal: 1650,
    },
    CoredllOrdinalDef {
        name: "?_set_new_mode@@YAHH@Z",
        ordinal: 1648,
    },
    CoredllOrdinalDef {
        name: "?nothrow@std@@3Unothrow_t@1@B",
        ordinal: 1647,
    },
    CoredllOrdinalDef {
        name: "?set_new_handler@@YAP6AXXZP6AXXZ@Z",
        ordinal: 1619,
    },
    CoredllOrdinalDef {
        name: "?set_terminate@std@@YAP6AXXZP6AXXZ@Z",
        ordinal: 1552,
    },
    CoredllOrdinalDef {
        name: "?set_unexpected@std@@YAP6AXXZP6AXXZ@Z",
        ordinal: 1553,
    },
    CoredllOrdinalDef {
        name: "?terminate@std@@YAXXZ",
        ordinal: 1556,
    },
    CoredllOrdinalDef {
        name: "?unexpected@std@@YAXXZ",
        ordinal: 1557,
    },
    CoredllOrdinalDef {
        name: "AFS_CloseAllFileHandles",
        ordinal: 655,
    },
    CoredllOrdinalDef {
        name: "AFS_CreateDirectoryW",
        ordinal: 644,
    },
    CoredllOrdinalDef {
        name: "AFS_CreateFileW",
        ordinal: 648,
    },
    CoredllOrdinalDef {
        name: "AFS_DeleteFileW",
        ordinal: 649,
    },
    CoredllOrdinalDef {
        name: "AFS_FindFirstChangeNotificationW",
        ordinal: 1685,
    },
    CoredllOrdinalDef {
        name: "AFS_FindFirstFileW",
        ordinal: 651,
    },
    CoredllOrdinalDef {
        name: "AFS_GetDiskFreeSpace",
        ordinal: 656,
    },
    CoredllOrdinalDef {
        name: "AFS_GetFileAttributesW",
        ordinal: 646,
    },
    CoredllOrdinalDef {
        name: "AFS_MoveFileW",
        ordinal: 650,
    },
    CoredllOrdinalDef {
        name: "AFS_NotifyMountedFS",
        ordinal: 657,
    },
    CoredllOrdinalDef {
        name: "AFS_PrestoChangoFileName",
        ordinal: 654,
    },
    CoredllOrdinalDef {
        name: "AFS_RegisterFileSystemFunction",
        ordinal: 652,
    },
    CoredllOrdinalDef {
        name: "AFS_RemoveDirectoryW",
        ordinal: 645,
    },
    CoredllOrdinalDef {
        name: "AFS_SetFileAttributesW",
        ordinal: 647,
    },
    CoredllOrdinalDef {
        name: "AFS_Unmount",
        ordinal: 643,
    },
    CoredllOrdinalDef {
        name: "A_SHAFinal",
        ordinal: 1791,
    },
    CoredllOrdinalDef {
        name: "A_SHAInit",
        ordinal: 1789,
    },
    CoredllOrdinalDef {
        name: "A_SHAUpdate",
        ordinal: 1790,
    },
    CoredllOrdinalDef {
        name: "AbortDoc",
        ordinal: 955,
    },
    CoredllOrdinalDef {
        name: "AccessibilitySoundSentryEvent",
        ordinal: 1540,
    },
    CoredllOrdinalDef {
        name: "ActivateDevice",
        ordinal: 1179,
    },
    CoredllOrdinalDef {
        name: "ActivateDeviceEx",
        ordinal: 1494,
    },
    CoredllOrdinalDef {
        name: "ActivateKeyboardLayout",
        ordinal: 1766,
    },
    CoredllOrdinalDef {
        name: "ActivateService",
        ordinal: 1508,
    },
    CoredllOrdinalDef {
        name: "AddEventAccess",
        ordinal: 558,
    },
    CoredllOrdinalDef {
        name: "AddFontResourceW",
        ordinal: 893,
    },
    CoredllOrdinalDef {
        name: "AddTrackedItem",
        ordinal: 578,
    },
    CoredllOrdinalDef {
        name: "AdjustWindowRectEx",
        ordinal: 887,
    },
    CoredllOrdinalDef {
        name: "AdvertiseInterface",
        ordinal: 1687,
    },
    CoredllOrdinalDef {
        name: "AllKeys",
        ordinal: 1453,
    },
    CoredllOrdinalDef {
        name: "AllocPhysMem",
        ordinal: 1486,
    },
    CoredllOrdinalDef {
        name: "AppendMenuW",
        ordinal: 842,
    },
    CoredllOrdinalDef {
        name: "AttachDebugger",
        ordinal: 157,
    },
    CoredllOrdinalDef {
        name: "AudioUpdateFromRegistry",
        ordinal: 376,
    },
    CoredllOrdinalDef {
        name: "BeginDeferWindowPos",
        ordinal: 1157,
    },
    CoredllOrdinalDef {
        name: "BeginPaint",
        ordinal: 260,
    },
    CoredllOrdinalDef {
        name: "BinaryCompress",
        ordinal: 593,
    },
    CoredllOrdinalDef {
        name: "BinaryDecompress",
        ordinal: 594,
    },
    CoredllOrdinalDef {
        name: "BitBlt",
        ordinal: 903,
    },
    CoredllOrdinalDef {
        name: "BringWindowToTop",
        ordinal: 275,
    },
    CoredllOrdinalDef {
        name: "CacheRangeFlush",
        ordinal: 1765,
    },
    CoredllOrdinalDef {
        name: "CacheSync",
        ordinal: 577,
    },
    CoredllOrdinalDef {
        name: "CallNextHookEx",
        ordinal: 1204,
    },
    CoredllOrdinalDef {
        name: "CallWindowProcW",
        ordinal: 285,
    },
    CoredllOrdinalDef {
        name: "CeChangeDatabaseLCID",
        ordinal: 340,
    },
    CoredllOrdinalDef {
        name: "CeClearUserNotification",
        ordinal: 474,
    },
    CoredllOrdinalDef {
        name: "CeCreateDatabase",
        ordinal: 315,
    },
    CoredllOrdinalDef {
        name: "CeCreateDatabaseEx",
        ordinal: 1190,
    },
    CoredllOrdinalDef {
        name: "CeCreateDatabaseEx2",
        ordinal: 1468,
    },
    CoredllOrdinalDef {
        name: "CeDeleteDatabase",
        ordinal: 318,
    },
    CoredllOrdinalDef {
        name: "CeDeleteDatabaseEx",
        ordinal: 1193,
    },
    CoredllOrdinalDef {
        name: "CeDeleteRecord",
        ordinal: 320,
    },
    CoredllOrdinalDef {
        name: "CeEnumDBVolumes",
        ordinal: 1165,
    },
    CoredllOrdinalDef {
        name: "CeEventHasOccurred",
        ordinal: 479,
    },
    CoredllOrdinalDef {
        name: "CeFindFirstDatabase",
        ordinal: 313,
    },
    CoredllOrdinalDef {
        name: "CeFindFirstDatabaseEx",
        ordinal: 1196,
    },
    CoredllOrdinalDef {
        name: "CeFindNextDatabase",
        ordinal: 314,
    },
    CoredllOrdinalDef {
        name: "CeFindNextDatabaseEx",
        ordinal: 1189,
    },
    CoredllOrdinalDef {
        name: "CeFlushDBVol",
        ordinal: 1217,
    },
    CoredllOrdinalDef {
        name: "CeFreeNotification",
        ordinal: 1226,
    },
    CoredllOrdinalDef {
        name: "CeGenRandom",
        ordinal: 1601,
    },
    CoredllOrdinalDef {
        name: "CeGetCallerTrust",
        ordinal: 1395,
    },
    CoredllOrdinalDef {
        name: "CeGetCurrentTrust",
        ordinal: 1357,
    },
    CoredllOrdinalDef {
        name: "CeGetDBInformationByHandle",
        ordinal: 1473,
    },
    CoredllOrdinalDef {
        name: "CeGetFileNotificationInfo",
        ordinal: 1798,
    },
    CoredllOrdinalDef {
        name: "CeGetRandomSeed",
        ordinal: 1443,
    },
    CoredllOrdinalDef {
        name: "CeGetThreadPriority",
        ordinal: 622,
    },
    CoredllOrdinalDef {
        name: "CeGetThreadQuantum",
        ordinal: 1245,
    },
    CoredllOrdinalDef {
        name: "CeGetUserNotification",
        ordinal: 1354,
    },
    CoredllOrdinalDef {
        name: "CeGetUserNotificationHandles",
        ordinal: 1353,
    },
    CoredllOrdinalDef {
        name: "CeGetUserNotificationPreferences",
        ordinal: 478,
    },
    CoredllOrdinalDef {
        name: "CeHandleAppNotifications",
        ordinal: 477,
    },
    CoredllOrdinalDef {
        name: "CeLogData",
        ordinal: 1451,
    },
    CoredllOrdinalDef {
        name: "CeLogGetZones",
        ordinal: 1681,
    },
    CoredllOrdinalDef {
        name: "CeLogReSync",
        ordinal: 1467,
    },
    CoredllOrdinalDef {
        name: "CeLogSetZones",
        ordinal: 1452,
    },
    CoredllOrdinalDef {
        name: "CeMapArgumentArray",
        ordinal: 1446,
    },
    CoredllOrdinalDef {
        name: "CeModuleJit",
        ordinal: 53,
    },
    CoredllOrdinalDef {
        name: "CeMountDBVol",
        ordinal: 1164,
    },
    CoredllOrdinalDef {
        name: "CeOidGetInfo",
        ordinal: 312,
    },
    CoredllOrdinalDef {
        name: "CeOidGetInfoEx",
        ordinal: 1195,
    },
    CoredllOrdinalDef {
        name: "CeOidGetInfoEx2",
        ordinal: 1472,
    },
    CoredllOrdinalDef {
        name: "CeOpenDatabase",
        ordinal: 317,
    },
    CoredllOrdinalDef {
        name: "CeOpenDatabaseEx",
        ordinal: 1192,
    },
    CoredllOrdinalDef {
        name: "CeOpenDatabaseEx2",
        ordinal: 1469,
    },
    CoredllOrdinalDef {
        name: "CeReadRecordProps",
        ordinal: 321,
    },
    CoredllOrdinalDef {
        name: "CeReadRecordPropsEx",
        ordinal: 1194,
    },
    CoredllOrdinalDef {
        name: "CeRegisterFileSystemNotification",
        ordinal: 331,
    },
    CoredllOrdinalDef {
        name: "CeRemoveFontResource",
        ordinal: 894,
    },
    CoredllOrdinalDef {
        name: "CeResyncFilesys",
        ordinal: 1425,
    },
    CoredllOrdinalDef {
        name: "CeRunAppAtEvent",
        ordinal: 476,
    },
    CoredllOrdinalDef {
        name: "CeRunAppAtTime",
        ordinal: 475,
    },
    CoredllOrdinalDef {
        name: "CeSeekDatabase",
        ordinal: 319,
    },
    CoredllOrdinalDef {
        name: "CeSeekDatabaseEx",
        ordinal: 1470,
    },
    CoredllOrdinalDef {
        name: "CeSetDatabaseInfo",
        ordinal: 316,
    },
    CoredllOrdinalDef {
        name: "CeSetDatabaseInfoEx",
        ordinal: 1191,
    },
    CoredllOrdinalDef {
        name: "CeSetDatabaseInfoEx2",
        ordinal: 1471,
    },
    CoredllOrdinalDef {
        name: "CeSetExtendedPdata",
        ordinal: 1455,
    },
    CoredllOrdinalDef {
        name: "CeSetPowerOnEvent",
        ordinal: 1688,
    },
    CoredllOrdinalDef {
        name: "CeSetProcessVersion",
        ordinal: 1775,
    },
    CoredllOrdinalDef {
        name: "CeSetThreadPriority",
        ordinal: 621,
    },
    CoredllOrdinalDef {
        name: "CeSetThreadQuantum",
        ordinal: 1244,
    },
    CoredllOrdinalDef {
        name: "CeSetUserNotification",
        ordinal: 473,
    },
    CoredllOrdinalDef {
        name: "CeSetUserNotificationEx",
        ordinal: 1352,
    },
    CoredllOrdinalDef {
        name: "CeUnmountDBVol",
        ordinal: 1197,
    },
    CoredllOrdinalDef {
        name: "CeWriteRecordProps",
        ordinal: 322,
    },
    CoredllOrdinalDef {
        name: "CeZeroPointer",
        ordinal: 1781,
    },
    CoredllOrdinalDef {
        name: "ChangeDisplaySettingsEx",
        ordinal: 1611,
    },
    CoredllOrdinalDef {
        name: "CharLowerBuffW",
        ordinal: 222,
    },
    CoredllOrdinalDef {
        name: "CharLowerW",
        ordinal: 221,
    },
    CoredllOrdinalDef {
        name: "CharNextW",
        ordinal: 226,
    },
    CoredllOrdinalDef {
        name: "CharPrevW",
        ordinal: 225,
    },
    CoredllOrdinalDef {
        name: "CharUpperBuffW",
        ordinal: 223,
    },
    CoredllOrdinalDef {
        name: "CharUpperW",
        ordinal: 224,
    },
    CoredllOrdinalDef {
        name: "CheckMenuItem",
        ordinal: 848,
    },
    CoredllOrdinalDef {
        name: "CheckMenuRadioItem",
        ordinal: 849,
    },
    CoredllOrdinalDef {
        name: "CheckPassword",
        ordinal: 182,
    },
    CoredllOrdinalDef {
        name: "CheckRadioButton",
        ordinal: 684,
    },
    CoredllOrdinalDef {
        name: "ChildWindowFromPoint",
        ordinal: 253,
    },
    CoredllOrdinalDef {
        name: "ClearCommBreak",
        ordinal: 107,
    },
    CoredllOrdinalDef {
        name: "ClearCommError",
        ordinal: 108,
    },
    CoredllOrdinalDef {
        name: "ClientToScreen",
        ordinal: 254,
    },
    CoredllOrdinalDef {
        name: "ClipCursor",
        ordinal: 731,
    },
    CoredllOrdinalDef {
        name: "CloseAllDeviceHandles",
        ordinal: 244,
    },
    CoredllOrdinalDef {
        name: "CloseAllFileHandles",
        ordinal: 242,
    },
    CoredllOrdinalDef {
        name: "CloseAllServiceHandles",
        ordinal: 1511,
    },
    CoredllOrdinalDef {
        name: "CloseClipboard",
        ordinal: 669,
    },
    CoredllOrdinalDef {
        name: "CloseEnhMetaFile",
        ordinal: 956,
    },
    CoredllOrdinalDef {
        name: "CloseHandle",
        ordinal: 553,
    },
    CoredllOrdinalDef {
        name: "CloseMsgQueue",
        ordinal: 1533,
    },
    CoredllOrdinalDef {
        name: "CloseProcOE",
        ordinal: 589,
    },
    CoredllOrdinalDef {
        name: "ComThreadBaseFunc",
        ordinal: 1240,
    },
    CoredllOrdinalDef {
        name: "CombineRgn",
        ordinal: 968,
    },
    CoredllOrdinalDef {
        name: "CompactAllHeaps",
        ordinal: 54,
    },
    CoredllOrdinalDef {
        name: "CompareFileTime",
        ordinal: 18,
    },
    CoredllOrdinalDef {
        name: "CompareStringW",
        ordinal: 198,
    },
    CoredllOrdinalDef {
        name: "ConnectDebugger",
        ordinal: 633,
    },
    CoredllOrdinalDef {
        name: "ContinueDebugEvent",
        ordinal: 504,
    },
    CoredllOrdinalDef {
        name: "ConvertDefaultLocale",
        ordinal: 210,
    },
    CoredllOrdinalDef {
        name: "CopyFileW",
        ordinal: 164,
    },
    CoredllOrdinalDef {
        name: "CopyRect",
        ordinal: 96,
    },
    CoredllOrdinalDef {
        name: "CountClipboardFormats",
        ordinal: 674,
    },
    CoredllOrdinalDef {
        name: "CreateAPIHandle",
        ordinal: 636,
    },
    CoredllOrdinalDef {
        name: "CreateAPISet",
        ordinal: 559,
    },
    CoredllOrdinalDef {
        name: "CreateAcceleratorTableW",
        ordinal: 92,
    },
    CoredllOrdinalDef {
        name: "CreateBitmap",
        ordinal: 901,
    },
    CoredllOrdinalDef {
        name: "CreateBitmapFromPointer",
        ordinal: 946,
    },
    CoredllOrdinalDef {
        name: "CreateCaret",
        ordinal: 658,
    },
    CoredllOrdinalDef {
        name: "CreateCompatibleBitmap",
        ordinal: 902,
    },
    CoredllOrdinalDef {
        name: "CreateCompatibleDC",
        ordinal: 910,
    },
    CoredllOrdinalDef {
        name: "CreateCrit",
        ordinal: 616,
    },
    CoredllOrdinalDef {
        name: "CreateCursor",
        ordinal: 722,
    },
    CoredllOrdinalDef {
        name: "CreateDCW",
        ordinal: 909,
    },
    CoredllOrdinalDef {
        name: "CreateDIBPatternBrushPt",
        ordinal: 929,
    },
    CoredllOrdinalDef {
        name: "CreateDIBSection",
        ordinal: 90,
    },
    CoredllOrdinalDef {
        name: "CreateDeviceHandle",
        ordinal: 245,
    },
    CoredllOrdinalDef {
        name: "CreateDialogIndirectParamW",
        ordinal: 688,
    },
    CoredllOrdinalDef {
        name: "CreateDirectoryW",
        ordinal: 160,
    },
    CoredllOrdinalDef {
        name: "CreateEnhMetaFileW",
        ordinal: 957,
    },
    CoredllOrdinalDef {
        name: "CreateEventW",
        ordinal: 495,
    },
    CoredllOrdinalDef {
        name: "CreateFileForMapping",
        ordinal: 552,
    },
    CoredllOrdinalDef {
        name: "CreateFileForMappingW",
        ordinal: 1167,
    },
    CoredllOrdinalDef {
        name: "CreateFileMappingW",
        ordinal: 548,
    },
    CoredllOrdinalDef {
        name: "CreateFileW",
        ordinal: 168,
    },
    CoredllOrdinalDef {
        name: "CreateFontIndirectW",
        ordinal: 895,
    },
    CoredllOrdinalDef {
        name: "CreateIconIndirect",
        ordinal: 723,
    },
    CoredllOrdinalDef {
        name: "CreateLocaleView",
        ordinal: 1466,
    },
    CoredllOrdinalDef {
        name: "CreateMenu",
        ordinal: 851,
    },
    CoredllOrdinalDef {
        name: "CreateMsgQueue",
        ordinal: 1529,
    },
    CoredllOrdinalDef {
        name: "CreateMutexW",
        ordinal: 555,
    },
    CoredllOrdinalDef {
        name: "CreatePalette",
        ordinal: 947,
    },
    CoredllOrdinalDef {
        name: "CreatePatternBrush",
        ordinal: 925,
    },
    CoredllOrdinalDef {
        name: "CreatePen",
        ordinal: 926,
    },
    CoredllOrdinalDef {
        name: "CreatePenIndirect",
        ordinal: 930,
    },
    CoredllOrdinalDef {
        name: "CreatePopupMenu",
        ordinal: 852,
    },
    CoredllOrdinalDef {
        name: "CreateProcessW",
        ordinal: 493,
    },
    CoredllOrdinalDef {
        name: "CreateRectRgn",
        ordinal: 980,
    },
    CoredllOrdinalDef {
        name: "CreateRectRgnIndirect",
        ordinal: 969,
    },
    CoredllOrdinalDef {
        name: "CreateSemaphoreW",
        ordinal: 1238,
    },
    CoredllOrdinalDef {
        name: "CreateServiceHandle",
        ordinal: 1512,
    },
    CoredllOrdinalDef {
        name: "CreateSolidBrush",
        ordinal: 931,
    },
    CoredllOrdinalDef {
        name: "CreateStaticMapping",
        ordinal: 1539,
    },
    CoredllOrdinalDef {
        name: "CreateThread",
        ordinal: 492,
    },
    CoredllOrdinalDef {
        name: "CreateWindowExW",
        ordinal: 246,
    },
    CoredllOrdinalDef {
        name: "CryptAcquireContextW",
        ordinal: 126,
    },
    CoredllOrdinalDef {
        name: "CryptContextAddRef",
        ordinal: 154,
    },
    CoredllOrdinalDef {
        name: "CryptCreateHash",
        ordinal: 137,
    },
    CoredllOrdinalDef {
        name: "CryptDecrypt",
        ordinal: 136,
    },
    CoredllOrdinalDef {
        name: "CryptDeriveKey",
        ordinal: 129,
    },
    CoredllOrdinalDef {
        name: "CryptDestroyHash",
        ordinal: 140,
    },
    CoredllOrdinalDef {
        name: "CryptDestroyKey",
        ordinal: 130,
    },
    CoredllOrdinalDef {
        name: "CryptDuplicateHash",
        ordinal: 156,
    },
    CoredllOrdinalDef {
        name: "CryptDuplicateKey",
        ordinal: 155,
    },
    CoredllOrdinalDef {
        name: "CryptEncrypt",
        ordinal: 135,
    },
    CoredllOrdinalDef {
        name: "CryptEnumProviderTypesW",
        ordinal: 152,
    },
    CoredllOrdinalDef {
        name: "CryptEnumProvidersW",
        ordinal: 153,
    },
    CoredllOrdinalDef {
        name: "CryptExportKey",
        ordinal: 133,
    },
    CoredllOrdinalDef {
        name: "CryptGenKey",
        ordinal: 128,
    },
    CoredllOrdinalDef {
        name: "CryptGenRandom",
        ordinal: 143,
    },
    CoredllOrdinalDef {
        name: "CryptGetDefaultProviderW",
        ordinal: 151,
    },
    CoredllOrdinalDef {
        name: "CryptGetHashParam",
        ordinal: 146,
    },
    CoredllOrdinalDef {
        name: "CryptGetKeyParam",
        ordinal: 132,
    },
    CoredllOrdinalDef {
        name: "CryptGetProvParam",
        ordinal: 148,
    },
    CoredllOrdinalDef {
        name: "CryptGetUserKey",
        ordinal: 144,
    },
    CoredllOrdinalDef {
        name: "CryptHashData",
        ordinal: 139,
    },
    CoredllOrdinalDef {
        name: "CryptHashSessionKey",
        ordinal: 138,
    },
    CoredllOrdinalDef {
        name: "CryptImportKey",
        ordinal: 134,
    },
    CoredllOrdinalDef {
        name: "CryptProtectData",
        ordinal: 1599,
    },
    CoredllOrdinalDef {
        name: "CryptReleaseContext",
        ordinal: 127,
    },
    CoredllOrdinalDef {
        name: "CryptSetHashParam",
        ordinal: 147,
    },
    CoredllOrdinalDef {
        name: "CryptSetKeyParam",
        ordinal: 131,
    },
    CoredllOrdinalDef {
        name: "CryptSetProvParam",
        ordinal: 149,
    },
    CoredllOrdinalDef {
        name: "CryptSetProviderExW",
        ordinal: 150,
    },
    CoredllOrdinalDef {
        name: "CryptSetProviderW",
        ordinal: 145,
    },
    CoredllOrdinalDef {
        name: "CryptSignHashW",
        ordinal: 141,
    },
    CoredllOrdinalDef {
        name: "CryptUnprotectData",
        ordinal: 1600,
    },
    CoredllOrdinalDef {
        name: "CryptVerifySignatureW",
        ordinal: 142,
    },
    CoredllOrdinalDef {
        name: "DBCanonicalize",
        ordinal: 233,
    },
    CoredllOrdinalDef {
        name: "DDKReg_GetIsrInfo",
        ordinal: 1669,
    },
    CoredllOrdinalDef {
        name: "DDKReg_GetPciInfo",
        ordinal: 1670,
    },
    CoredllOrdinalDef {
        name: "DDKReg_GetWindowInfo",
        ordinal: 1668,
    },
    CoredllOrdinalDef {
        name: "DeactivateDevice",
        ordinal: 1180,
    },
    CoredllOrdinalDef {
        name: "DebugActiveProcess",
        ordinal: 505,
    },
    CoredllOrdinalDef {
        name: "DebugNotify",
        ordinal: 642,
    },
    CoredllOrdinalDef {
        name: "DecompressBinaryBlock",
        ordinal: 1776,
    },
    CoredllOrdinalDef {
        name: "DefDlgProcW",
        ordinal: 689,
    },
    CoredllOrdinalDef {
        name: "DefWindowProcW",
        ordinal: 264,
    },
    CoredllOrdinalDef {
        name: "DeferWindowPos",
        ordinal: 1158,
    },
    CoredllOrdinalDef {
        name: "DeleteAndRenameFile",
        ordinal: 183,
    },
    CoredllOrdinalDef {
        name: "DeleteCriticalSection",
        ordinal: 3,
    },
    CoredllOrdinalDef {
        name: "DeleteDC",
        ordinal: 911,
    },
    CoredllOrdinalDef {
        name: "DeleteEnhMetaFile",
        ordinal: 958,
    },
    CoredllOrdinalDef {
        name: "DeleteFileW",
        ordinal: 165,
    },
    CoredllOrdinalDef {
        name: "DeleteMenu",
        ordinal: 850,
    },
    CoredllOrdinalDef {
        name: "DeleteObject",
        ordinal: 912,
    },
    CoredllOrdinalDef {
        name: "DeleteTrackedItem",
        ordinal: 579,
    },
    CoredllOrdinalDef {
        name: "DeregisterAFS",
        ordinal: 335,
    },
    CoredllOrdinalDef {
        name: "DeregisterAFSName",
        ordinal: 339,
    },
    CoredllOrdinalDef {
        name: "DeregisterDevice",
        ordinal: 236,
    },
    CoredllOrdinalDef {
        name: "DeregisterService",
        ordinal: 1510,
    },
    CoredllOrdinalDef {
        name: "DestroyAcceleratorTable",
        ordinal: 93,
    },
    CoredllOrdinalDef {
        name: "DestroyCaret",
        ordinal: 659,
    },
    CoredllOrdinalDef {
        name: "DestroyCursor",
        ordinal: 724,
    },
    CoredllOrdinalDef {
        name: "DestroyIcon",
        ordinal: 725,
    },
    CoredllOrdinalDef {
        name: "DestroyMenu",
        ordinal: 844,
    },
    CoredllOrdinalDef {
        name: "DestroyWindow",
        ordinal: 265,
    },
    CoredllOrdinalDef {
        name: "DeviceIoControl",
        ordinal: 179,
    },
    CoredllOrdinalDef {
        name: "DevicePowerNotify",
        ordinal: 1588,
    },
    CoredllOrdinalDef {
        name: "DialogBoxIndirectParamW",
        ordinal: 690,
    },
    CoredllOrdinalDef {
        name: "DisableCaretSystemWide",
        ordinal: 666,
    },
    CoredllOrdinalDef {
        name: "DisableThreadLibraryCalls",
        ordinal: 1232,
    },
    CoredllOrdinalDef {
        name: "DispatchMessageW",
        ordinal: 859,
    },
    CoredllOrdinalDef {
        name: "DrawEdge",
        ordinal: 932,
    },
    CoredllOrdinalDef {
        name: "DrawFocusRect",
        ordinal: 933,
    },
    CoredllOrdinalDef {
        name: "DrawFrameControl",
        ordinal: 987,
    },
    CoredllOrdinalDef {
        name: "DrawIconEx",
        ordinal: 726,
    },
    CoredllOrdinalDef {
        name: "DrawMenuBar",
        ordinal: 856,
    },
    CoredllOrdinalDef {
        name: "DrawTextW",
        ordinal: 945,
    },
    CoredllOrdinalDef {
        name: "DumpFileSystemHeap",
        ordinal: 341,
    },
    CoredllOrdinalDef {
        name: "DumpKCallProfile",
        ordinal: 510,
    },
    CoredllOrdinalDef {
        name: "DuplicateHandle",
        ordinal: 1535,
    },
    CoredllOrdinalDef {
        name: "Ellipse",
        ordinal: 934,
    },
    CoredllOrdinalDef {
        name: "EmptyClipboard",
        ordinal: 677,
    },
    CoredllOrdinalDef {
        name: "EnableCaretSystemWide",
        ordinal: 667,
    },
    CoredllOrdinalDef {
        name: "EnableEUDC",
        ordinal: 986,
    },
    CoredllOrdinalDef {
        name: "EnableHardwareKeyboard",
        ordinal: 825,
    },
    CoredllOrdinalDef {
        name: "EnableMenuItem",
        ordinal: 847,
    },
    CoredllOrdinalDef {
        name: "EnableWindow",
        ordinal: 287,
    },
    CoredllOrdinalDef {
        name: "EndDeferWindowPos",
        ordinal: 1159,
    },
    CoredllOrdinalDef {
        name: "EndDialog",
        ordinal: 691,
    },
    CoredllOrdinalDef {
        name: "EndDoc",
        ordinal: 959,
    },
    CoredllOrdinalDef {
        name: "EndPage",
        ordinal: 960,
    },
    CoredllOrdinalDef {
        name: "EndPaint",
        ordinal: 261,
    },
    CoredllOrdinalDef {
        name: "EnterCriticalSection",
        ordinal: 4,
    },
    CoredllOrdinalDef {
        name: "EnumCalendarInfoW",
        ordinal: 206,
    },
    CoredllOrdinalDef {
        name: "EnumClipboardFormats",
        ordinal: 675,
    },
    CoredllOrdinalDef {
        name: "EnumDateFormatsW",
        ordinal: 208,
    },
    CoredllOrdinalDef {
        name: "EnumDevices",
        ordinal: 124,
    },
    CoredllOrdinalDef {
        name: "EnumDeviceInterfaces",
        ordinal: 1874,
    },
    CoredllOrdinalDef {
        name: "EnumDisplayDevices",
        ordinal: 1778,
    },
    CoredllOrdinalDef {
        name: "EnumDisplayMonitors",
        ordinal: 1526,
    },
    CoredllOrdinalDef {
        name: "EnumDisplaySettings",
        ordinal: 1777,
    },
    CoredllOrdinalDef {
        name: "EnumFontFamiliesW",
        ordinal: 965,
    },
    CoredllOrdinalDef {
        name: "EnumFontsW",
        ordinal: 966,
    },
    CoredllOrdinalDef {
        name: "EnumPnpIds",
        ordinal: 123,
    },
    CoredllOrdinalDef {
        name: "EnumPropsEx",
        ordinal: 1500,
    },
    CoredllOrdinalDef {
        name: "EnumServices",
        ordinal: 1517,
    },
    CoredllOrdinalDef {
        name: "EnumSystemCodePagesW",
        ordinal: 220,
    },
    CoredllOrdinalDef {
        name: "EnumSystemLocalesW",
        ordinal: 219,
    },
    CoredllOrdinalDef {
        name: "EnumTimeFormatsW",
        ordinal: 207,
    },
    CoredllOrdinalDef {
        name: "EnumWindows",
        ordinal: 291,
    },
    CoredllOrdinalDef {
        name: "EqualRect",
        ordinal: 97,
    },
    CoredllOrdinalDef {
        name: "EqualRgn",
        ordinal: 91,
    },
    CoredllOrdinalDef {
        name: "EscapeCommFunction",
        ordinal: 109,
    },
    CoredllOrdinalDef {
        name: "EventModify",
        ordinal: 494,
    },
    CoredllOrdinalDef {
        name: "ExcludeClipRect",
        ordinal: 970,
    },
    CoredllOrdinalDef {
        name: "ExitThread",
        ordinal: 6,
    },
    CoredllOrdinalDef {
        name: "ExtCreateRegion",
        ordinal: 1617,
    },
    CoredllOrdinalDef {
        name: "ExtEscape",
        ordinal: 1182,
    },
    CoredllOrdinalDef {
        name: "ExtTextOutW",
        ordinal: 896,
    },
    CoredllOrdinalDef {
        name: "ExtractIconExW",
        ordinal: 727,
    },
    CoredllOrdinalDef {
        name: "ExtractResource",
        ordinal: 573,
    },
    CoredllOrdinalDef {
        name: "FileSystemPowerFunction",
        ordinal: 241,
    },
    CoredllOrdinalDef {
        name: "FileTimeToLocalFileTime",
        ordinal: 21,
    },
    CoredllOrdinalDef {
        name: "FileTimeToSystemTime",
        ordinal: 20,
    },
    CoredllOrdinalDef {
        name: "FillRect",
        ordinal: 935,
    },
    CoredllOrdinalDef {
        name: "FillRgn",
        ordinal: 927,
    },
    CoredllOrdinalDef {
        name: "FilterTrackedItem",
        ordinal: 585,
    },
    CoredllOrdinalDef {
        name: "FindClose",
        ordinal: 180,
    },
    CoredllOrdinalDef {
        name: "FindCloseChangeNotification",
        ordinal: 1684,
    },
    CoredllOrdinalDef {
        name: "FindFirstChangeNotificationW",
        ordinal: 1682,
    },
    CoredllOrdinalDef {
        name: "FindFirstFileExW",
        ordinal: 1235,
    },
    CoredllOrdinalDef {
        name: "FindFirstFileW",
        ordinal: 167,
    },
    CoredllOrdinalDef {
        name: "FindNextChangeNotification",
        ordinal: 1683,
    },
    CoredllOrdinalDef {
        name: "FindNextFileW",
        ordinal: 181,
    },
    CoredllOrdinalDef {
        name: "FindResource",
        ordinal: 531,
    },
    CoredllOrdinalDef {
        name: "FindResourceW",
        ordinal: 532,
    },
    CoredllOrdinalDef {
        name: "FindWindowW",
        ordinal: 286,
    },
    CoredllOrdinalDef {
        name: "FlushFileBuffers",
        ordinal: 175,
    },
    CoredllOrdinalDef {
        name: "FlushInstructionCache",
        ordinal: 508,
    },
    CoredllOrdinalDef {
        name: "FlushViewOfFile",
        ordinal: 551,
    },
    CoredllOrdinalDef {
        name: "FlushViewOfFileMaybe",
        ordinal: 1215,
    },
    CoredllOrdinalDef {
        name: "FoldStringW",
        ordinal: 218,
    },
    CoredllOrdinalDef {
        name: "ForcePageout",
        ordinal: 540,
    },
    CoredllOrdinalDef {
        name: "FormatMessageW",
        ordinal: 234,
    },
    CoredllOrdinalDef {
        name: "FreeIntChainHandler",
        ordinal: 1476,
    },
    CoredllOrdinalDef {
        name: "FreeLibrary",
        ordinal: 529,
    },
    CoredllOrdinalDef {
        name: "FreeLibraryAndExitThread",
        ordinal: 1216,
    },
    CoredllOrdinalDef {
        name: "FreePhysMem",
        ordinal: 1487,
    },
    CoredllOrdinalDef {
        name: "GetACP",
        ordinal: 186,
    },
    CoredllOrdinalDef {
        name: "GetAPIAddress",
        ordinal: 32,
    },
    CoredllOrdinalDef {
        name: "GetActiveWindow",
        ordinal: 706,
    },
    CoredllOrdinalDef {
        name: "GetAssociatedMenu",
        ordinal: 300,
    },
    CoredllOrdinalDef {
        name: "GetAsyncKeyState",
        ordinal: 826,
    },
    CoredllOrdinalDef {
        name: "GetAsyncShiftFlags",
        ordinal: 834,
    },
    CoredllOrdinalDef {
        name: "GetBkColor",
        ordinal: 913,
    },
    CoredllOrdinalDef {
        name: "GetBkMode",
        ordinal: 914,
    },
    CoredllOrdinalDef {
        name: "GetCPInfo",
        ordinal: 188,
    },
    CoredllOrdinalDef {
        name: "GetCRTFlags",
        ordinal: 1228,
    },
    CoredllOrdinalDef {
        name: "GetCRTStorageEx",
        ordinal: 1227,
    },
    CoredllOrdinalDef {
        name: "GetCallStackSnapshot",
        ordinal: 1760,
    },
    CoredllOrdinalDef {
        name: "GetCallerProcess",
        ordinal: 607,
    },
    CoredllOrdinalDef {
        name: "GetCallerProcessIndex",
        ordinal: 641,
    },
    CoredllOrdinalDef {
        name: "GetCapture",
        ordinal: 707,
    },
    CoredllOrdinalDef {
        name: "GetCaretBlinkTime",
        ordinal: 665,
    },
    CoredllOrdinalDef {
        name: "GetCaretPos",
        ordinal: 663,
    },
    CoredllOrdinalDef {
        name: "GetCharABCWidths",
        ordinal: 1779,
    },
    CoredllOrdinalDef {
        name: "GetCharWidth32",
        ordinal: 1664,
    },
    CoredllOrdinalDef {
        name: "GetClassInfoW",
        ordinal: 878,
    },
    CoredllOrdinalDef {
        name: "GetClassLong",
        ordinal: 881,
    },
    CoredllOrdinalDef {
        name: "GetClassLongW",
        ordinal: 879,
    },
    CoredllOrdinalDef {
        name: "GetClassNameW",
        ordinal: 283,
    },
    CoredllOrdinalDef {
        name: "GetClientRect",
        ordinal: 249,
    },
    CoredllOrdinalDef {
        name: "GetClipBox",
        ordinal: 971,
    },
    CoredllOrdinalDef {
        name: "GetClipCursor",
        ordinal: 732,
    },
    CoredllOrdinalDef {
        name: "GetClipRgn",
        ordinal: 972,
    },
    CoredllOrdinalDef {
        name: "GetClipboardData",
        ordinal: 672,
    },
    CoredllOrdinalDef {
        name: "GetClipboardDataAlloc",
        ordinal: 681,
    },
    CoredllOrdinalDef {
        name: "GetClipboardFormatNameW",
        ordinal: 676,
    },
    CoredllOrdinalDef {
        name: "GetClipboardOwner",
        ordinal: 670,
    },
    CoredllOrdinalDef {
        name: "GetCommMask",
        ordinal: 110,
    },
    CoredllOrdinalDef {
        name: "GetCommModemStatus",
        ordinal: 111,
    },
    CoredllOrdinalDef {
        name: "GetCommProperties",
        ordinal: 112,
    },
    CoredllOrdinalDef {
        name: "GetCommState",
        ordinal: 113,
    },
    CoredllOrdinalDef {
        name: "GetCommTimeouts",
        ordinal: 114,
    },
    CoredllOrdinalDef {
        name: "GetCommandLineW",
        ordinal: 1231,
    },
    CoredllOrdinalDef {
        name: "GetCurrencyFormatW",
        ordinal: 205,
    },
    CoredllOrdinalDef {
        name: "GetCurrentFT",
        ordinal: 29,
    },
    CoredllOrdinalDef {
        name: "GetCurrentObject",
        ordinal: 915,
    },
    CoredllOrdinalDef {
        name: "GetCurrentPermissions",
        ordinal: 612,
    },
    CoredllOrdinalDef {
        name: "GetCurrentPositionEx",
        ordinal: 1653,
    },
    CoredllOrdinalDef {
        name: "GetCursor",
        ordinal: 733,
    },
    CoredllOrdinalDef {
        name: "GetCursorPos",
        ordinal: 734,
    },
    CoredllOrdinalDef {
        name: "GetDC",
        ordinal: 262,
    },
    CoredllOrdinalDef {
        name: "GetDCEx",
        ordinal: 1185,
    },
    CoredllOrdinalDef {
        name: "GetDIBColorTable",
        ordinal: 1665,
    },
    CoredllOrdinalDef {
        name: "GetDateFormatW",
        ordinal: 203,
    },
    CoredllOrdinalDef {
        name: "GetDesktopWindow",
        ordinal: 1397,
    },
    CoredllOrdinalDef {
        name: "GetDeviceByIndex",
        ordinal: 1236,
    },
    CoredllOrdinalDef {
        name: "GetDeviceCaps",
        ordinal: 916,
    },
    CoredllOrdinalDef {
        name: "GetDeviceKeys",
        ordinal: 125,
    },
    CoredllOrdinalDef {
        name: "GetDevicePower",
        ordinal: 1679,
    },
    CoredllOrdinalDef {
        name: "GetDialogBaseUnits",
        ordinal: 694,
    },
    CoredllOrdinalDef {
        name: "GetDiskFreeSpaceExW",
        ordinal: 184,
    },
    CoredllOrdinalDef {
        name: "GetDlgCtrlID",
        ordinal: 693,
    },
    CoredllOrdinalDef {
        name: "GetDlgItem",
        ordinal: 692,
    },
    CoredllOrdinalDef {
        name: "GetDlgItemInt",
        ordinal: 695,
    },
    CoredllOrdinalDef {
        name: "GetDlgItemTextW",
        ordinal: 687,
    },
    CoredllOrdinalDef {
        name: "GetDoubleClickTime",
        ordinal: 888,
    },
    CoredllOrdinalDef {
        name: "GetEventData",
        ordinal: 1527,
    },
    CoredllOrdinalDef {
        name: "GetExitCodeProcess",
        ordinal: 519,
    },
    CoredllOrdinalDef {
        name: "GetExitCodeThread",
        ordinal: 518,
    },
    CoredllOrdinalDef {
        name: "GetFSHeapInfo",
        ordinal: 603,
    },
    CoredllOrdinalDef {
        name: "GetFileAttributesExW",
        ordinal: 1237,
    },
    CoredllOrdinalDef {
        name: "GetFileAttributesW",
        ordinal: 166,
    },
    CoredllOrdinalDef {
        name: "GetFileInformationByHandle",
        ordinal: 174,
    },
    CoredllOrdinalDef {
        name: "GetFileSize",
        ordinal: 172,
    },
    CoredllOrdinalDef {
        name: "GetFileTime",
        ordinal: 176,
    },
    CoredllOrdinalDef {
        name: "GetFileVersionInfoSizeW",
        ordinal: 1461,
    },
    CoredllOrdinalDef {
        name: "GetFileVersionInfoW",
        ordinal: 1460,
    },
    CoredllOrdinalDef {
        name: "GetFocus",
        ordinal: 705,
    },
    CoredllOrdinalDef {
        name: "GetForegroundInfo",
        ordinal: 1224,
    },
    CoredllOrdinalDef {
        name: "GetForegroundKeyboardLayoutHandle",
        ordinal: 1802,
    },
    CoredllOrdinalDef {
        name: "GetForegroundKeyboardTarget",
        ordinal: 1225,
    },
    CoredllOrdinalDef {
        name: "GetForegroundWindow",
        ordinal: 701,
    },
    CoredllOrdinalDef {
        name: "GetHeapSnapshot",
        ordinal: 52,
    },
    CoredllOrdinalDef {
        name: "GetIdleTime",
        ordinal: 608,
    },
    CoredllOrdinalDef {
        name: "GetKPhys",
        ordinal: 581,
    },
    CoredllOrdinalDef {
        name: "GetKeyState",
        ordinal: 860,
    },
    CoredllOrdinalDef {
        name: "GetKeyboardLayout",
        ordinal: 1229,
    },
    CoredllOrdinalDef {
        name: "GetKeyboardLayoutList",
        ordinal: 1767,
    },
    CoredllOrdinalDef {
        name: "GetKeyboardLayoutNameW",
        ordinal: 1160,
    },
    CoredllOrdinalDef {
        name: "GetKeyboardStatus",
        ordinal: 827,
    },
    CoredllOrdinalDef {
        name: "GetKeyboardTarget",
        ordinal: 711,
    },
    CoredllOrdinalDef {
        name: "GetKeyboardType",
        ordinal: 1771,
    },
    CoredllOrdinalDef {
        name: "GetLastError",
        ordinal: 516,
    },
    CoredllOrdinalDef {
        name: "GetLocalTime",
        ordinal: 23,
    },
    CoredllOrdinalDef {
        name: "GetLocaleInfoW",
        ordinal: 200,
    },
    CoredllOrdinalDef {
        name: "GetMenuItemInfoW",
        ordinal: 854,
    },
    CoredllOrdinalDef {
        name: "GetMessagePos",
        ordinal: 862,
    },
    CoredllOrdinalDef {
        name: "GetMessageQueueReadyTimeStamp",
        ordinal: 1477,
    },
    CoredllOrdinalDef {
        name: "GetMessageSource",
        ordinal: 872,
    },
    CoredllOrdinalDef {
        name: "GetMessageW",
        ordinal: 861,
    },
    CoredllOrdinalDef {
        name: "GetMessageWNoWait",
        ordinal: 863,
    },
    CoredllOrdinalDef {
        name: "GetModuleFileNameW",
        ordinal: 537,
    },
    CoredllOrdinalDef {
        name: "GetModuleHandleW",
        ordinal: 1177,
    },
    CoredllOrdinalDef {
        name: "GetModuleInformation",
        ordinal: 1721,
    },
    CoredllOrdinalDef {
        name: "GetMonitorInfo",
        ordinal: 1525,
    },
    CoredllOrdinalDef {
        name: "GetMouseMovePoints",
        ordinal: 820,
    },
    CoredllOrdinalDef {
        name: "GetMsgQueueInfo",
        ordinal: 1532,
    },
    CoredllOrdinalDef {
        name: "GetNearestColor",
        ordinal: 952,
    },
    CoredllOrdinalDef {
        name: "GetNearestPaletteIndex",
        ordinal: 948,
    },
    CoredllOrdinalDef {
        name: "GetNextDlgGroupItem",
        ordinal: 697,
    },
    CoredllOrdinalDef {
        name: "GetNextDlgTabItem",
        ordinal: 696,
    },
    CoredllOrdinalDef {
        name: "GetNumberFormatW",
        ordinal: 204,
    },
    CoredllOrdinalDef {
        name: "GetOEMCP",
        ordinal: 187,
    },
    CoredllOrdinalDef {
        name: "GetObjectType",
        ordinal: 917,
    },
    CoredllOrdinalDef {
        name: "GetObjectW",
        ordinal: 918,
    },
    CoredllOrdinalDef {
        name: "GetOpenClipboardWindow",
        ordinal: 680,
    },
    CoredllOrdinalDef {
        name: "GetOpenFileNameW",
        ordinal: 488,
    },
    CoredllOrdinalDef {
        name: "GetOwnerProcess",
        ordinal: 606,
    },
    CoredllOrdinalDef {
        name: "GetPaletteEntries",
        ordinal: 949,
    },
    CoredllOrdinalDef {
        name: "GetParent",
        ordinal: 269,
    },
    CoredllOrdinalDef {
        name: "GetPasswordActive",
        ordinal: 239,
    },
    CoredllOrdinalDef {
        name: "GetPasswordStatus",
        ordinal: 1538,
    },
    CoredllOrdinalDef {
        name: "GetPixel",
        ordinal: 936,
    },
    CoredllOrdinalDef {
        name: "GetPriorityClipboardFormat",
        ordinal: 679,
    },
    CoredllOrdinalDef {
        name: "GetPrivateCallbacks",
        ordinal: 1400,
    },
    CoredllOrdinalDef {
        name: "GetProcAddrBits",
        ordinal: 602,
    },
    CoredllOrdinalDef {
        name: "GetProcAddressA",
        ordinal: 1230,
    },
    CoredllOrdinalDef {
        name: "GetProcAddressW",
        ordinal: 530,
    },
    CoredllOrdinalDef {
        name: "GetProcFromPtr",
        ordinal: 600,
    },
    CoredllOrdinalDef {
        name: "GetProcName",
        ordinal: 624,
    },
    CoredllOrdinalDef {
        name: "GetProcessHeap",
        ordinal: 50,
    },
    CoredllOrdinalDef {
        name: "GetProcessIDFromIndex",
        ordinal: 1727,
    },
    CoredllOrdinalDef {
        name: "GetProcessIndexFromID",
        ordinal: 640,
    },
    CoredllOrdinalDef {
        name: "GetProcessVersion",
        ordinal: 536,
    },
    CoredllOrdinalDef {
        name: "GetProp",
        ordinal: 1498,
    },
    CoredllOrdinalDef {
        name: "GetQueueStatus",
        ordinal: 1420,
    },
    CoredllOrdinalDef {
        name: "GetRealTime",
        ordinal: 570,
    },
    CoredllOrdinalDef {
        name: "GetRegionData",
        ordinal: 973,
    },
    CoredllOrdinalDef {
        name: "GetRgnBox",
        ordinal: 974,
    },
    CoredllOrdinalDef {
        name: "GetROP2",
        ordinal: 1990,
    },
    CoredllOrdinalDef {
        name: "GetRomFileBytes",
        ordinal: 576,
    },
    CoredllOrdinalDef {
        name: "GetRomFileInfo",
        ordinal: 575,
    },
    CoredllOrdinalDef {
        name: "GetSaveFileNameW",
        ordinal: 489,
    },
    CoredllOrdinalDef {
        name: "GetScrollInfo",
        ordinal: 282,
    },
    CoredllOrdinalDef {
        name: "GetServiceByIndex",
        ordinal: 1513,
    },
    CoredllOrdinalDef {
        name: "GetServiceHandle",
        ordinal: 1518,
    },
    CoredllOrdinalDef {
        name: "GetStdioPathW",
        ordinal: 1149,
    },
    CoredllOrdinalDef {
        name: "GetStockObject",
        ordinal: 919,
    },
    CoredllOrdinalDef {
        name: "GetStoreInformation",
        ordinal: 323,
    },
    CoredllOrdinalDef {
        name: "GetStringTypeExW",
        ordinal: 217,
    },
    CoredllOrdinalDef {
        name: "GetStringTypeW",
        ordinal: 216,
    },
    CoredllOrdinalDef {
        name: "GetSubMenu",
        ordinal: 855,
    },
    CoredllOrdinalDef {
        name: "GetSysColor",
        ordinal: 889,
    },
    CoredllOrdinalDef {
        name: "GetSysColorBrush",
        ordinal: 937,
    },
    CoredllOrdinalDef {
        name: "GetSystemDefaultLCID",
        ordinal: 213,
    },
    CoredllOrdinalDef {
        name: "GetSystemDefaultLangID",
        ordinal: 211,
    },
    CoredllOrdinalDef {
        name: "GetSystemInfo",
        ordinal: 542,
    },
    CoredllOrdinalDef {
        name: "GetSystemMemoryDivision",
        ordinal: 336,
    },
    CoredllOrdinalDef {
        name: "GetSystemMetrics",
        ordinal: 885,
    },
    CoredllOrdinalDef {
        name: "GetSystemPaletteEntries",
        ordinal: 950,
    },
    CoredllOrdinalDef {
        name: "GetSystemPowerState",
        ordinal: 1581,
    },
    CoredllOrdinalDef {
        name: "GetSystemTime",
        ordinal: 25,
    },
    CoredllOrdinalDef {
        name: "GetTempFileNameW",
        ordinal: 1234,
    },
    CoredllOrdinalDef {
        name: "GetTempPathW",
        ordinal: 162,
    },
    CoredllOrdinalDef {
        name: "GetTextAlign",
        ordinal: 1655,
    },
    CoredllOrdinalDef {
        name: "GetTextColor",
        ordinal: 920,
    },
    CoredllOrdinalDef {
        name: "GetTextExtentExPointW",
        ordinal: 897,
    },
    CoredllOrdinalDef {
        name: "GetTextFaceW",
        ordinal: 967,
    },
    CoredllOrdinalDef {
        name: "GetTextMetricsW",
        ordinal: 898,
    },
    CoredllOrdinalDef {
        name: "GetThreadContext",
        ordinal: 1148,
    },
    CoredllOrdinalDef {
        name: "GetThreadPriority",
        ordinal: 515,
    },
    CoredllOrdinalDef {
        name: "GetThreadTimes",
        ordinal: 1186,
    },
    CoredllOrdinalDef {
        name: "GetTickCount",
        ordinal: 535,
    },
    CoredllOrdinalDef {
        name: "GetTimeFormatW",
        ordinal: 202,
    },
    CoredllOrdinalDef {
        name: "GetTimeZoneInformation",
        ordinal: 27,
    },
    CoredllOrdinalDef {
        name: "GetUpdateRect",
        ordinal: 274,
    },
    CoredllOrdinalDef {
        name: "GetUpdateRgn",
        ordinal: 273,
    },
    CoredllOrdinalDef {
        name: "GetUserDefaultLCID",
        ordinal: 215,
    },
    CoredllOrdinalDef {
        name: "GetUserDefaultLangID",
        ordinal: 212,
    },
    CoredllOrdinalDef {
        name: "GetUserDirectory",
        ordinal: 1686,
    },
    CoredllOrdinalDef {
        name: "GetUserNameExW",
        ordinal: 1503,
    },
    CoredllOrdinalDef {
        name: "GetVersionEx",
        ordinal: 17,
    },
    CoredllOrdinalDef {
        name: "GetVersionExW",
        ordinal: 717,
    },
    CoredllOrdinalDef {
        name: "GetWindow",
        ordinal: 251,
    },
    CoredllOrdinalDef {
        name: "GetWindowDC",
        ordinal: 270,
    },
    CoredllOrdinalDef {
        name: "GetWindowLongW",
        ordinal: 259,
    },
    CoredllOrdinalDef {
        name: "GetWindowRect",
        ordinal: 248,
    },
    CoredllOrdinalDef {
        name: "GetWindowRgn",
        ordinal: 1399,
    },
    CoredllOrdinalDef {
        name: "GetWindowTextLengthW",
        ordinal: 276,
    },
    CoredllOrdinalDef {
        name: "GetWindowTextW",
        ordinal: 257,
    },
    CoredllOrdinalDef {
        name: "GetWindowTextWDirect",
        ordinal: 1454,
    },
    CoredllOrdinalDef {
        name: "GetWindowThreadProcessId",
        ordinal: 292,
    },
    CoredllOrdinalDef {
        name: "GiveKPhys",
        ordinal: 582,
    },
    CoredllOrdinalDef {
        name: "GlobalAddAtomW",
        ordinal: 1519,
    },
    CoredllOrdinalDef {
        name: "GlobalDeleteAtom",
        ordinal: 1520,
    },
    CoredllOrdinalDef {
        name: "GlobalFindAtomW",
        ordinal: 1521,
    },
    CoredllOrdinalDef {
        name: "GlobalMemoryStatus",
        ordinal: 88,
    },
    CoredllOrdinalDef {
        name: "GradientFill",
        ordinal: 1763,
    },
    CoredllOrdinalDef {
        name: "GwesPowerDown",
        ordinal: 1722,
    },
    CoredllOrdinalDef {
        name: "GwesPowerOffSystem",
        ordinal: 296,
    },
    CoredllOrdinalDef {
        name: "GwesPowerUp",
        ordinal: 1723,
    },
    CoredllOrdinalDef {
        name: "HeapAlloc",
        ordinal: 46,
    },
    CoredllOrdinalDef {
        name: "HeapCreate",
        ordinal: 44,
    },
    CoredllOrdinalDef {
        name: "HeapDestroy",
        ordinal: 45,
    },
    CoredllOrdinalDef {
        name: "HeapFree",
        ordinal: 49,
    },
    CoredllOrdinalDef {
        name: "HeapReAlloc",
        ordinal: 47,
    },
    CoredllOrdinalDef {
        name: "HeapSize",
        ordinal: 48,
    },
    CoredllOrdinalDef {
        name: "HeapValidate",
        ordinal: 51,
    },
    CoredllOrdinalDef {
        name: "HideCaret",
        ordinal: 660,
    },
    CoredllOrdinalDef {
        name: "ImageList_Add",
        ordinal: 738,
    },
    CoredllOrdinalDef {
        name: "ImageList_AddMasked",
        ordinal: 739,
    },
    CoredllOrdinalDef {
        name: "ImageList_BeginDrag",
        ordinal: 740,
    },
    CoredllOrdinalDef {
        name: "ImageList_Copy",
        ordinal: 767,
    },
    CoredllOrdinalDef {
        name: "ImageList_CopyDitherImage",
        ordinal: 741,
    },
    CoredllOrdinalDef {
        name: "ImageList_Create",
        ordinal: 742,
    },
    CoredllOrdinalDef {
        name: "ImageList_Destroy",
        ordinal: 743,
    },
    CoredllOrdinalDef {
        name: "ImageList_DragEnter",
        ordinal: 744,
    },
    CoredllOrdinalDef {
        name: "ImageList_DragLeave",
        ordinal: 745,
    },
    CoredllOrdinalDef {
        name: "ImageList_DragMove",
        ordinal: 746,
    },
    CoredllOrdinalDef {
        name: "ImageList_DragShowNolock",
        ordinal: 747,
    },
    CoredllOrdinalDef {
        name: "ImageList_Draw",
        ordinal: 748,
    },
    CoredllOrdinalDef {
        name: "ImageList_DrawEx",
        ordinal: 749,
    },
    CoredllOrdinalDef {
        name: "ImageList_DrawIndirect",
        ordinal: 750,
    },
    CoredllOrdinalDef {
        name: "ImageList_Duplicate",
        ordinal: 768,
    },
    CoredllOrdinalDef {
        name: "ImageList_EndDrag",
        ordinal: 751,
    },
    CoredllOrdinalDef {
        name: "ImageList_GetBkColor",
        ordinal: 752,
    },
    CoredllOrdinalDef {
        name: "ImageList_GetDragImage",
        ordinal: 753,
    },
    CoredllOrdinalDef {
        name: "ImageList_GetIcon",
        ordinal: 754,
    },
    CoredllOrdinalDef {
        name: "ImageList_GetIconSize",
        ordinal: 755,
    },
    CoredllOrdinalDef {
        name: "ImageList_GetImageCount",
        ordinal: 756,
    },
    CoredllOrdinalDef {
        name: "ImageList_GetImageInfo",
        ordinal: 757,
    },
    CoredllOrdinalDef {
        name: "ImageList_LoadImage",
        ordinal: 758,
    },
    CoredllOrdinalDef {
        name: "ImageList_Merge",
        ordinal: 759,
    },
    CoredllOrdinalDef {
        name: "ImageList_Remove",
        ordinal: 760,
    },
    CoredllOrdinalDef {
        name: "ImageList_Replace",
        ordinal: 761,
    },
    CoredllOrdinalDef {
        name: "ImageList_ReplaceIcon",
        ordinal: 762,
    },
    CoredllOrdinalDef {
        name: "ImageList_SetBkColor",
        ordinal: 763,
    },
    CoredllOrdinalDef {
        name: "ImageList_SetDragCursorImage",
        ordinal: 764,
    },
    CoredllOrdinalDef {
        name: "ImageList_SetIconSize",
        ordinal: 765,
    },
    CoredllOrdinalDef {
        name: "ImageList_SetImageCount",
        ordinal: 769,
    },
    CoredllOrdinalDef {
        name: "ImageList_SetOverlayImage",
        ordinal: 766,
    },
    CoredllOrdinalDef {
        name: "ImmAssociateContext",
        ordinal: 770,
    },
    CoredllOrdinalDef {
        name: "ImmAssociateContextEx",
        ordinal: 1205,
    },
    CoredllOrdinalDef {
        name: "ImmConfigureIMEW",
        ordinal: 771,
    },
    CoredllOrdinalDef {
        name: "ImmCreateContext",
        ordinal: 1198,
    },
    CoredllOrdinalDef {
        name: "ImmCreateIMCC",
        ordinal: 772,
    },
    CoredllOrdinalDef {
        name: "ImmDestroyContext",
        ordinal: 1199,
    },
    CoredllOrdinalDef {
        name: "ImmDestroyIMCC",
        ordinal: 773,
    },
    CoredllOrdinalDef {
        name: "ImmDisableIME",
        ordinal: 1206,
    },
    CoredllOrdinalDef {
        name: "ImmEnableIME",
        ordinal: 1541,
    },
    CoredllOrdinalDef {
        name: "ImmEnumRegisterWordW",
        ordinal: 774,
    },
    CoredllOrdinalDef {
        name: "ImmEscapeW",
        ordinal: 775,
    },
    CoredllOrdinalDef {
        name: "ImmGenerateMessage",
        ordinal: 776,
    },
    CoredllOrdinalDef {
        name: "ImmGetCandidateListCountW",
        ordinal: 778,
    },
    CoredllOrdinalDef {
        name: "ImmGetCandidateListW",
        ordinal: 777,
    },
    CoredllOrdinalDef {
        name: "ImmGetCandidateWindow",
        ordinal: 779,
    },
    CoredllOrdinalDef {
        name: "ImmGetCompositionFontW",
        ordinal: 780,
    },
    CoredllOrdinalDef {
        name: "ImmGetCompositionStringW",
        ordinal: 781,
    },
    CoredllOrdinalDef {
        name: "ImmGetCompositionWindow",
        ordinal: 782,
    },
    CoredllOrdinalDef {
        name: "ImmGetContext",
        ordinal: 783,
    },
    CoredllOrdinalDef {
        name: "ImmGetConversionListW",
        ordinal: 784,
    },
    CoredllOrdinalDef {
        name: "ImmGetConversionStatus",
        ordinal: 785,
    },
    CoredllOrdinalDef {
        name: "ImmGetDefaultIMEWnd",
        ordinal: 786,
    },
    CoredllOrdinalDef {
        name: "ImmGetDescriptionW",
        ordinal: 787,
    },
    CoredllOrdinalDef {
        name: "ImmGetGuideLineW",
        ordinal: 788,
    },
    CoredllOrdinalDef {
        name: "ImmGetHotKey",
        ordinal: 813,
    },
    CoredllOrdinalDef {
        name: "ImmGetIMCCLockCount",
        ordinal: 789,
    },
    CoredllOrdinalDef {
        name: "ImmGetIMCCSize",
        ordinal: 790,
    },
    CoredllOrdinalDef {
        name: "ImmGetIMCLockCount",
        ordinal: 791,
    },
    CoredllOrdinalDef {
        name: "ImmGetIMEFileNameW",
        ordinal: 1207,
    },
    CoredllOrdinalDef {
        name: "ImmGetImeMenuItemsW",
        ordinal: 1211,
    },
    CoredllOrdinalDef {
        name: "ImmGetKeyboardLayout",
        ordinal: 1769,
    },
    CoredllOrdinalDef {
        name: "ImmGetOpenStatus",
        ordinal: 792,
    },
    CoredllOrdinalDef {
        name: "ImmGetProperty",
        ordinal: 793,
    },
    CoredllOrdinalDef {
        name: "ImmGetRegisterWordStyleW",
        ordinal: 794,
    },
    CoredllOrdinalDef {
        name: "ImmGetStatusWindowPos",
        ordinal: 1200,
    },
    CoredllOrdinalDef {
        name: "ImmGetVirtualKey",
        ordinal: 1210,
    },
    CoredllOrdinalDef {
        name: "ImmIsIME",
        ordinal: 1209,
    },
    CoredllOrdinalDef {
        name: "ImmIsUIMessageW",
        ordinal: 796,
    },
    CoredllOrdinalDef {
        name: "ImmLockIMC",
        ordinal: 797,
    },
    CoredllOrdinalDef {
        name: "ImmLockIMCC",
        ordinal: 798,
    },
    CoredllOrdinalDef {
        name: "ImmNotifyIME",
        ordinal: 800,
    },
    CoredllOrdinalDef {
        name: "ImmReSizeIMCC",
        ordinal: 801,
    },
    CoredllOrdinalDef {
        name: "ImmRegisterWordW",
        ordinal: 802,
    },
    CoredllOrdinalDef {
        name: "ImmReleaseContext",
        ordinal: 803,
    },
    CoredllOrdinalDef {
        name: "ImmRequestMessageW",
        ordinal: 1242,
    },
    CoredllOrdinalDef {
        name: "ImmSIPanelState",
        ordinal: 804,
    },
    CoredllOrdinalDef {
        name: "ImmSetCandidateWindow",
        ordinal: 807,
    },
    CoredllOrdinalDef {
        name: "ImmSetCompositionFontW",
        ordinal: 808,
    },
    CoredllOrdinalDef {
        name: "ImmSetCompositionStringW",
        ordinal: 809,
    },
    CoredllOrdinalDef {
        name: "ImmSetCompositionWindow",
        ordinal: 810,
    },
    CoredllOrdinalDef {
        name: "ImmSetConversionStatus",
        ordinal: 811,
    },
    CoredllOrdinalDef {
        name: "ImmSetHotKey",
        ordinal: 812,
    },
    CoredllOrdinalDef {
        name: "ImmSetImeWndIMC",
        ordinal: 1222,
    },
    CoredllOrdinalDef {
        name: "ImmSetOpenStatus",
        ordinal: 814,
    },
    CoredllOrdinalDef {
        name: "ImmSetStatusWindowPos",
        ordinal: 815,
    },
    CoredllOrdinalDef {
        name: "ImmSimulateHotKey",
        ordinal: 816,
    },
    CoredllOrdinalDef {
        name: "ImmUnlockIMC",
        ordinal: 817,
    },
    CoredllOrdinalDef {
        name: "ImmUnlockIMCC",
        ordinal: 818,
    },
    CoredllOrdinalDef {
        name: "ImmUnregisterWordW",
        ordinal: 819,
    },
    CoredllOrdinalDef {
        name: "InSendMessage",
        ordinal: 1419,
    },
    CoredllOrdinalDef {
        name: "InflateRect",
        ordinal: 98,
    },
    CoredllOrdinalDef {
        name: "InitLocale",
        ordinal: 8,
    },
    CoredllOrdinalDef {
        name: "InitializeCriticalSection",
        ordinal: 2,
    },
    CoredllOrdinalDef {
        name: "InputDebugCharW",
        ordinal: 595,
    },
    CoredllOrdinalDef {
        name: "InsertMenuW",
        ordinal: 841,
    },
    CoredllOrdinalDef {
        name: "Int_CloseHandle",
        ordinal: 1762,
    },
    CoredllOrdinalDef {
        name: "Int_CreateEventW",
        ordinal: 1761,
    },
    CoredllOrdinalDef {
        name: "InterlockedCompareExchange",
        ordinal: 1492,
    },
    CoredllOrdinalDef {
        name: "InterlockedDecrement",
        ordinal: 11,
    },
    CoredllOrdinalDef {
        name: "InterlockedExchange",
        ordinal: 12,
    },
    CoredllOrdinalDef {
        name: "InterlockedExchangeAdd",
        ordinal: 1491,
    },
    CoredllOrdinalDef {
        name: "InterlockedIncrement",
        ordinal: 10,
    },
    CoredllOrdinalDef {
        name: "InterlockedTestExchange",
        ordinal: 9,
    },
    CoredllOrdinalDef {
        name: "InterruptDisable",
        ordinal: 629,
    },
    CoredllOrdinalDef {
        name: "InterruptDone",
        ordinal: 628,
    },
    CoredllOrdinalDef {
        name: "InterruptInitialize",
        ordinal: 627,
    },
    CoredllOrdinalDef {
        name: "InterruptMask",
        ordinal: 1797,
    },
    CoredllOrdinalDef {
        name: "IntersectClipRect",
        ordinal: 975,
    },
    CoredllOrdinalDef {
        name: "IntersectRect",
        ordinal: 99,
    },
    CoredllOrdinalDef {
        name: "InvalidateRect",
        ordinal: 250,
    },
    CoredllOrdinalDef {
        name: "InvalidateRgn",
        ordinal: 1615,
    },
    CoredllOrdinalDef {
        name: "InvertRect",
        ordinal: 1770,
    },
    CoredllOrdinalDef {
        name: "IsAPIReady",
        ordinal: 30,
    },
    CoredllOrdinalDef {
        name: "IsBadCodePtr",
        ordinal: 521,
    },
    CoredllOrdinalDef {
        name: "IsBadPtr",
        ordinal: 601,
    },
    CoredllOrdinalDef {
        name: "IsBadReadPtr",
        ordinal: 522,
    },
    CoredllOrdinalDef {
        name: "IsBadWritePtr",
        ordinal: 523,
    },
    CoredllOrdinalDef {
        name: "IsChild",
        ordinal: 277,
    },
    CoredllOrdinalDef {
        name: "IsClipboardFormatAvailable",
        ordinal: 678,
    },
    CoredllOrdinalDef {
        name: "IsDBCSLeadByte",
        ordinal: 191,
    },
    CoredllOrdinalDef {
        name: "IsDBCSLeadByteEx",
        ordinal: 192,
    },
    CoredllOrdinalDef {
        name: "IsDialogMessageW",
        ordinal: 698,
    },
    CoredllOrdinalDef {
        name: "IsEncryptionPermitted",
        ordinal: 613,
    },
    CoredllOrdinalDef {
        name: "IsExiting",
        ordinal: 159,
    },
    CoredllOrdinalDef {
        name: "IsPrimaryThread",
        ordinal: 610,
    },
    CoredllOrdinalDef {
        name: "IsProcessDying",
        ordinal: 1213,
    },
    CoredllOrdinalDef {
        name: "IsProcessorFeaturePresent",
        ordinal: 1758,
    },
    CoredllOrdinalDef {
        name: "IsRectEmpty",
        ordinal: 100,
    },
    CoredllOrdinalDef {
        name: "IsSystemFile",
        ordinal: 1680,
    },
    CoredllOrdinalDef {
        name: "IsValidCodePage",
        ordinal: 185,
    },
    CoredllOrdinalDef {
        name: "IsValidLocale",
        ordinal: 209,
    },
    CoredllOrdinalDef {
        name: "IsWindow",
        ordinal: 271,
    },
    CoredllOrdinalDef {
        name: "IsWindowEnabled",
        ordinal: 288,
    },
    CoredllOrdinalDef {
        name: "IsWindowVisible",
        ordinal: 886,
    },
    CoredllOrdinalDef {
        name: "KernExtractIcons",
        ordinal: 574,
    },
    CoredllOrdinalDef {
        name: "KernelIoControl",
        ordinal: 557,
    },
    CoredllOrdinalDef {
        name: "KernelLibIoControl",
        ordinal: 1489,
    },
    CoredllOrdinalDef {
        name: "KeybdGetDeviceInfo",
        ordinal: 828,
    },
    CoredllOrdinalDef {
        name: "KeybdInitStates",
        ordinal: 829,
    },
    CoredllOrdinalDef {
        name: "KeybdVKeyToUnicode",
        ordinal: 830,
    },
    CoredllOrdinalDef {
        name: "KillAllOtherThreads",
        ordinal: 605,
    },
    CoredllOrdinalDef {
        name: "KillTimer",
        ordinal: 876,
    },
    CoredllOrdinalDef {
        name: "LCMapStringW",
        ordinal: 199,
    },
    CoredllOrdinalDef {
        name: "LeaveCritSec",
        ordinal: 597,
    },
    CoredllOrdinalDef {
        name: "LeaveCriticalSection",
        ordinal: 5,
    },
    CoredllOrdinalDef {
        name: "LineTo",
        ordinal: 1652,
    },
    CoredllOrdinalDef {
        name: "LoadAcceleratorsW",
        ordinal: 94,
    },
    CoredllOrdinalDef {
        name: "LoadAnimatedCursor",
        ordinal: 1493,
    },
    CoredllOrdinalDef {
        name: "LoadBitmapW",
        ordinal: 873,
    },
    CoredllOrdinalDef {
        name: "LoadCursorW",
        ordinal: 683,
    },
    CoredllOrdinalDef {
        name: "LoadDriver",
        ordinal: 626,
    },
    CoredllOrdinalDef {
        name: "LoadFSD",
        ordinal: 237,
    },
    CoredllOrdinalDef {
        name: "LoadFSDEx",
        ordinal: 1421,
    },
    CoredllOrdinalDef {
        name: "LoadIconW",
        ordinal: 728,
    },
    CoredllOrdinalDef {
        name: "LoadImageW",
        ordinal: 730,
    },
    CoredllOrdinalDef {
        name: "LoadIntChainHandler",
        ordinal: 1475,
    },
    CoredllOrdinalDef {
        name: "LoadKernelLibrary",
        ordinal: 1671,
    },
    CoredllOrdinalDef {
        name: "LoadKeyboardLayoutW",
        ordinal: 1768,
    },
    CoredllOrdinalDef {
        name: "LoadLibraryExW",
        ordinal: 1241,
    },
    CoredllOrdinalDef {
        name: "LoadLibraryW",
        ordinal: 528,
    },
    CoredllOrdinalDef {
        name: "LoadMenuW",
        ordinal: 846,
    },
    CoredllOrdinalDef {
        name: "LoadResource",
        ordinal: 533,
    },
    CoredllOrdinalDef {
        name: "LoadStringW",
        ordinal: 874,
    },
    CoredllOrdinalDef {
        name: "LocalAlloc",
        ordinal: 33,
    },
    CoredllOrdinalDef {
        name: "LocalAllocInProcess",
        ordinal: 41,
    },
    CoredllOrdinalDef {
        name: "LocalFileTimeToFileTime",
        ordinal: 22,
    },
    CoredllOrdinalDef {
        name: "LocalFree",
        ordinal: 36,
    },
    CoredllOrdinalDef {
        name: "LocalFreeInProcess",
        ordinal: 42,
    },
    CoredllOrdinalDef {
        name: "LocalReAlloc",
        ordinal: 34,
    },
    CoredllOrdinalDef {
        name: "LocalSize",
        ordinal: 35,
    },
    CoredllOrdinalDef {
        name: "LocalSizeInProcess",
        ordinal: 43,
    },
    CoredllOrdinalDef {
        name: "LockPages",
        ordinal: 1161,
    },
    CoredllOrdinalDef {
        name: "MD5Final",
        ordinal: 1794,
    },
    CoredllOrdinalDef {
        name: "MD5Init",
        ordinal: 1792,
    },
    CoredllOrdinalDef {
        name: "MD5Update",
        ordinal: 1793,
    },
    CoredllOrdinalDef {
        name: "MainThreadBaseFunc",
        ordinal: 14,
    },
    CoredllOrdinalDef {
        name: "MapCallerPtr",
        ordinal: 1602,
    },
    CoredllOrdinalDef {
        name: "MapDialogRect",
        ordinal: 699,
    },
    CoredllOrdinalDef {
        name: "MapPtrToProcWithSize",
        ordinal: 1603,
    },
    CoredllOrdinalDef {
        name: "MapPtrToProcess",
        ordinal: 598,
    },
    CoredllOrdinalDef {
        name: "MapPtrUnsecure",
        ordinal: 599,
    },
    CoredllOrdinalDef {
        name: "MapViewOfFile",
        ordinal: 549,
    },
    CoredllOrdinalDef {
        name: "MapVirtualKeyW",
        ordinal: 831,
    },
    CoredllOrdinalDef {
        name: "MapWindowPoints",
        ordinal: 284,
    },
    CoredllOrdinalDef {
        name: "MaskBlt",
        ordinal: 904,
    },
    CoredllOrdinalDef {
        name: "MessageBeep",
        ordinal: 857,
    },
    CoredllOrdinalDef {
        name: "MessageBoxW",
        ordinal: 858,
    },
    CoredllOrdinalDef {
        name: "MonitorFromPoint",
        ordinal: 1522,
    },
    CoredllOrdinalDef {
        name: "MonitorFromRect",
        ordinal: 1523,
    },
    CoredllOrdinalDef {
        name: "MonitorFromWindow",
        ordinal: 1524,
    },
    CoredllOrdinalDef {
        name: "MoveFileW",
        ordinal: 163,
    },
    CoredllOrdinalDef {
        name: "MoveToEx",
        ordinal: 1651,
    },
    CoredllOrdinalDef {
        name: "MoveWindow",
        ordinal: 272,
    },
    CoredllOrdinalDef {
        name: "MsgWaitForMultipleObjectsEx",
        ordinal: 871,
    },
    CoredllOrdinalDef {
        name: "MultiByteToWideChar",
        ordinal: 196,
    },
    CoredllOrdinalDef {
        name: "NKDbgPrintfW",
        ordinal: 545,
    },
    CoredllOrdinalDef {
        name: "NKTerminateThread",
        ordinal: 623,
    },
    CoredllOrdinalDef {
        name: "NKvDbgPrintfW",
        ordinal: 568,
    },
    CoredllOrdinalDef {
        name: "NLedGetDeviceInfo",
        ordinal: 839,
    },
    CoredllOrdinalDef {
        name: "NLedSetDevice",
        ordinal: 840,
    },
    CoredllOrdinalDef {
        name: "NotSystemParametersInfoI",
        ordinal: 1787,
    },
    CoredllOrdinalDef {
        name: "NotifyForceCleanboot",
        ordinal: 513,
    },
    CoredllOrdinalDef {
        name: "NotifyWinUserSystem",
        ordinal: 716,
    },
    CoredllOrdinalDef {
        name: "OffsetRect",
        ordinal: 101,
    },
    CoredllOrdinalDef {
        name: "OffsetRgn",
        ordinal: 976,
    },
    CoredllOrdinalDef {
        name: "OpenClipboard",
        ordinal: 668,
    },
    CoredllOrdinalDef {
        name: "OpenDeviceKey",
        ordinal: 1396,
    },
    CoredllOrdinalDef {
        name: "OpenEventW",
        ordinal: 1496,
    },
    CoredllOrdinalDef {
        name: "OpenMsgQueue",
        ordinal: 1536,
    },
    CoredllOrdinalDef {
        name: "OpenProcess",
        ordinal: 509,
    },
    CoredllOrdinalDef {
        name: "OtherThreadsRunning",
        ordinal: 604,
    },
    CoredllOrdinalDef {
        name: "OutputDebugStringW",
        ordinal: 541,
    },
    CoredllOrdinalDef {
        name: "PPSHRestart",
        ordinal: 638,
    },
    CoredllOrdinalDef {
        name: "PSLNotify",
        ordinal: 7,
    },
    CoredllOrdinalDef {
        name: "PageOutModule",
        ordinal: 1780,
    },
    CoredllOrdinalDef {
        name: "PatBlt",
        ordinal: 938,
    },
    CoredllOrdinalDef {
        name: "PeekMessageW",
        ordinal: 864,
    },
    CoredllOrdinalDef {
        name: "PegClearUserNotification",
        ordinal: 468,
    },
    CoredllOrdinalDef {
        name: "PegCreateDatabase",
        ordinal: 304,
    },
    CoredllOrdinalDef {
        name: "PegDeleteDatabase",
        ordinal: 307,
    },
    CoredllOrdinalDef {
        name: "PegDeleteRecord",
        ordinal: 309,
    },
    CoredllOrdinalDef {
        name: "PegFindFirstDatabase",
        ordinal: 302,
    },
    CoredllOrdinalDef {
        name: "PegFindNextDatabase",
        ordinal: 303,
    },
    CoredllOrdinalDef {
        name: "PegGetUserNotificationPreferences",
        ordinal: 472,
    },
    CoredllOrdinalDef {
        name: "PegHandleAppNotifications",
        ordinal: 471,
    },
    CoredllOrdinalDef {
        name: "PegOidGetInfo",
        ordinal: 301,
    },
    CoredllOrdinalDef {
        name: "PegOpenDatabase",
        ordinal: 306,
    },
    CoredllOrdinalDef {
        name: "PegReadRecordProps",
        ordinal: 310,
    },
    CoredllOrdinalDef {
        name: "PegRemoveFontResource",
        ordinal: 899,
    },
    CoredllOrdinalDef {
        name: "PegRunAppAtEvent",
        ordinal: 470,
    },
    CoredllOrdinalDef {
        name: "PegRunAppAtTime",
        ordinal: 469,
    },
    CoredllOrdinalDef {
        name: "PegSeekDatabase",
        ordinal: 308,
    },
    CoredllOrdinalDef {
        name: "PegSetDatabaseInfo",
        ordinal: 305,
    },
    CoredllOrdinalDef {
        name: "PegSetUserNotification",
        ordinal: 467,
    },
    CoredllOrdinalDef {
        name: "PegWriteRecordProps",
        ordinal: 311,
    },
    CoredllOrdinalDef {
        name: "PerformCallBack4",
        ordinal: 1448,
    },
    CoredllOrdinalDef {
        name: "PlayEnhMetaFile",
        ordinal: 961,
    },
    CoredllOrdinalDef {
        name: "PlaySoundW",
        ordinal: 378,
    },
    CoredllOrdinalDef {
        name: "Polygon",
        ordinal: 939,
    },
    CoredllOrdinalDef {
        name: "Polyline",
        ordinal: 940,
    },
    CoredllOrdinalDef {
        name: "PostKeybdMessage",
        ordinal: 832,
    },
    CoredllOrdinalDef {
        name: "PostMessageW",
        ordinal: 865,
    },
    CoredllOrdinalDef {
        name: "PostQuitMessage",
        ordinal: 866,
    },
    CoredllOrdinalDef {
        name: "PostThreadMessageW",
        ordinal: 290,
    },
    CoredllOrdinalDef {
        name: "PowerOffSystem",
        ordinal: 617,
    },
    CoredllOrdinalDef {
        name: "PowerPolicyNotify",
        ordinal: 1764,
    },
    CoredllOrdinalDef {
        name: "PrintTrackedItem",
        ordinal: 580,
    },
    CoredllOrdinalDef {
        name: "ProcessDetachAllDLLs",
        ordinal: 572,
    },
    CoredllOrdinalDef {
        name: "ProfileCaptureStatus",
        ordinal: 1800,
    },
    CoredllOrdinalDef {
        name: "ProfileStart",
        ordinal: 82,
    },
    CoredllOrdinalDef {
        name: "ProfileStartEx",
        ordinal: 1801,
    },
    CoredllOrdinalDef {
        name: "ProfileStop",
        ordinal: 83,
    },
    CoredllOrdinalDef {
        name: "ProfileSyscall",
        ordinal: 569,
    },
    CoredllOrdinalDef {
        name: "PtInRect",
        ordinal: 102,
    },
    CoredllOrdinalDef {
        name: "PtInRegion",
        ordinal: 977,
    },
    CoredllOrdinalDef {
        name: "PurgeComm",
        ordinal: 115,
    },
    CoredllOrdinalDef {
        name: "QASetWindowsJournalHook",
        ordinal: 821,
    },
    CoredllOrdinalDef {
        name: "QAUnhookWindowsJournalHook",
        ordinal: 822,
    },
    CoredllOrdinalDef {
        name: "QueryAPISetID",
        ordinal: 490,
    },
    CoredllOrdinalDef {
        name: "QueryInstructionSet",
        ordinal: 1677,
    },
    CoredllOrdinalDef {
        name: "QueryPerformanceCounter",
        ordinal: 538,
    },
    CoredllOrdinalDef {
        name: "QueryPerformanceFrequency",
        ordinal: 539,
    },
    CoredllOrdinalDef {
        name: "RaiseException",
        ordinal: 543,
    },
    CoredllOrdinalDef {
        name: "Random",
        ordinal: 80,
    },
    CoredllOrdinalDef {
        name: "ReadFile",
        ordinal: 170,
    },
    CoredllOrdinalDef {
        name: "ReadFileWithSeek",
        ordinal: 243,
    },
    CoredllOrdinalDef {
        name: "ReadMsgQueue",
        ordinal: 1530,
    },
    CoredllOrdinalDef {
        name: "ReadProcessMemory",
        ordinal: 506,
    },
    CoredllOrdinalDef {
        name: "ReadRegistryFromOEM",
        ordinal: 1153,
    },
    CoredllOrdinalDef {
        name: "RealizePalette",
        ordinal: 953,
    },
    CoredllOrdinalDef {
        name: "RectInRegion",
        ordinal: 978,
    },
    CoredllOrdinalDef {
        name: "RectVisible",
        ordinal: 981,
    },
    CoredllOrdinalDef {
        name: "Rectangle",
        ordinal: 941,
    },
    CoredllOrdinalDef {
        name: "RectangleAnimation",
        ordinal: 294,
    },
    CoredllOrdinalDef {
        name: "RedrawWindow",
        ordinal: 1672,
    },
    CoredllOrdinalDef {
        name: "RefreshKernelAlarm",
        ordinal: 587,
    },
    CoredllOrdinalDef {
        name: "RegCloseKey",
        ordinal: 455,
    },
    CoredllOrdinalDef {
        name: "RegCopyFile",
        ordinal: 465,
    },
    CoredllOrdinalDef {
        name: "RegCreateKeyExW",
        ordinal: 456,
    },
    CoredllOrdinalDef {
        name: "RegDeleteKeyW",
        ordinal: 457,
    },
    CoredllOrdinalDef {
        name: "RegDeleteValueW",
        ordinal: 458,
    },
    CoredllOrdinalDef {
        name: "RegEnumKeyExW",
        ordinal: 460,
    },
    CoredllOrdinalDef {
        name: "RegEnumValueW",
        ordinal: 459,
    },
    CoredllOrdinalDef {
        name: "RegFlushKey",
        ordinal: 1152,
    },
    CoredllOrdinalDef {
        name: "RegOpenKeyExW",
        ordinal: 461,
    },
    CoredllOrdinalDef {
        name: "RegOpenProcessKey",
        ordinal: 1542,
    },
    CoredllOrdinalDef {
        name: "RegQueryInfoKeyW",
        ordinal: 462,
    },
    CoredllOrdinalDef {
        name: "RegQueryValueExW",
        ordinal: 463,
    },
    CoredllOrdinalDef {
        name: "RegReplaceKey",
        ordinal: 1479,
    },
    CoredllOrdinalDef {
        name: "RegRestoreFile",
        ordinal: 466,
    },
    CoredllOrdinalDef {
        name: "RegSaveKey",
        ordinal: 1478,
    },
    CoredllOrdinalDef {
        name: "RegSetValueExW",
        ordinal: 464,
    },
    CoredllOrdinalDef {
        name: "RegisterAFSEx",
        ordinal: 1490,
    },
    CoredllOrdinalDef {
        name: "RegisterAFSName",
        ordinal: 338,
    },
    CoredllOrdinalDef {
        name: "RegisterAPISet",
        ordinal: 635,
    },
    CoredllOrdinalDef {
        name: "RegisterClassW",
        ordinal: 95,
    },
    CoredllOrdinalDef {
        name: "RegisterClipboardFormatW",
        ordinal: 673,
    },
    CoredllOrdinalDef {
        name: "RegisterDbgZones",
        ordinal: 546,
    },
    CoredllOrdinalDef {
        name: "RegisterDesktop",
        ordinal: 1507,
    },
    CoredllOrdinalDef {
        name: "RegisterDevice",
        ordinal: 235,
    },
    CoredllOrdinalDef {
        name: "RegisterHotKey",
        ordinal: 835,
    },
    CoredllOrdinalDef {
        name: "RegisterPowerRelationship",
        ordinal: 1609,
    },
    CoredllOrdinalDef {
        name: "RegisterSIPanel",
        ordinal: 293,
    },
    CoredllOrdinalDef {
        name: "RegisterService",
        ordinal: 1509,
    },
    CoredllOrdinalDef {
        name: "RegisterTaskBar",
        ordinal: 892,
    },
    CoredllOrdinalDef {
        name: "RegisterTaskBarEx",
        ordinal: 1506,
    },
    CoredllOrdinalDef {
        name: "RegisterTrackedItem",
        ordinal: 584,
    },
    CoredllOrdinalDef {
        name: "RegisterWindowMessageW",
        ordinal: 891,
    },
    CoredllOrdinalDef {
        name: "ReinitLocale",
        ordinal: 1799,
    },
    CoredllOrdinalDef {
        name: "ReleaseCapture",
        ordinal: 709,
    },
    CoredllOrdinalDef {
        name: "ReleaseDC",
        ordinal: 263,
    },
    CoredllOrdinalDef {
        name: "ReleaseMutex",
        ordinal: 556,
    },
    CoredllOrdinalDef {
        name: "ReleasePowerRelationship",
        ordinal: 1610,
    },
    CoredllOrdinalDef {
        name: "ReleasePowerRequirement",
        ordinal: 1584,
    },
    CoredllOrdinalDef {
        name: "ReleaseSemaphore",
        ordinal: 1239,
    },
    CoredllOrdinalDef {
        name: "RemoteHeapAlloc",
        ordinal: 1604,
    },
    CoredllOrdinalDef {
        name: "RemoteHeapFree",
        ordinal: 1606,
    },
    CoredllOrdinalDef {
        name: "RemoteHeapReAlloc",
        ordinal: 1605,
    },
    CoredllOrdinalDef {
        name: "RemoteHeapSize",
        ordinal: 1607,
    },
    CoredllOrdinalDef {
        name: "RemoteLocalAlloc",
        ordinal: 37,
    },
    CoredllOrdinalDef {
        name: "RemoteLocalFree",
        ordinal: 40,
    },
    CoredllOrdinalDef {
        name: "RemoteLocalReAlloc",
        ordinal: 38,
    },
    CoredllOrdinalDef {
        name: "RemoteLocalSize",
        ordinal: 39,
    },
    CoredllOrdinalDef {
        name: "RemoveDirectoryW",
        ordinal: 161,
    },
    CoredllOrdinalDef {
        name: "RemoveFontResourceW",
        ordinal: 900,
    },
    CoredllOrdinalDef {
        name: "RemoveMenu",
        ordinal: 843,
    },
    CoredllOrdinalDef {
        name: "RemoveProp",
        ordinal: 1499,
    },
    CoredllOrdinalDef {
        name: "RequestDeviceNotifications",
        ordinal: 1504,
    },
    CoredllOrdinalDef {
        name: "RequestPowerNotifications",
        ordinal: 1585,
    },
    CoredllOrdinalDef {
        name: "ResourceCreateList",
        ordinal: 1612,
    },
    CoredllOrdinalDef {
        name: "ResourceRelease",
        ordinal: 1614,
    },
    CoredllOrdinalDef {
        name: "ResourceRequest",
        ordinal: 1613,
    },
    CoredllOrdinalDef {
        name: "RestoreDC",
        ordinal: 907,
    },
    CoredllOrdinalDef {
        name: "ResumeThread",
        ordinal: 500,
    },
    CoredllOrdinalDef {
        name: "RoundRect",
        ordinal: 942,
    },
    CoredllOrdinalDef {
        name: "SHAddToRecentDocs",
        ordinal: 483,
    },
    CoredllOrdinalDef {
        name: "SHChangeNotifyRegisterI",
        ordinal: 1805,
    },
    CoredllOrdinalDef {
        name: "SHCloseAppsI",
        ordinal: 1788,
    },
    CoredllOrdinalDef {
        name: "SHCreateExplorerInstance",
        ordinal: 1163,
    },
    CoredllOrdinalDef {
        name: "SHCreateShortcut",
        ordinal: 484,
    },
    CoredllOrdinalDef {
        name: "SHCreateShortcutEx",
        ordinal: 1488,
    },
    CoredllOrdinalDef {
        name: "SHDoneButtonI",
        ordinal: 1782,
    },
    CoredllOrdinalDef {
        name: "SHFileNotifyFreeI",
        ordinal: 1804,
    },
    CoredllOrdinalDef {
        name: "SHFileNotifyRemoveI",
        ordinal: 1803,
    },
    CoredllOrdinalDef {
        name: "SHGetAppKeyAssocI",
        ordinal: 1783,
    },
    CoredllOrdinalDef {
        name: "SHGetFileInfo",
        ordinal: 482,
    },
    CoredllOrdinalDef {
        name: "SHGetShortcutTarget",
        ordinal: 485,
    },
    CoredllOrdinalDef {
        name: "SHGetSpecialFolderPath",
        ordinal: 295,
    },
    CoredllOrdinalDef {
        name: "SHLoadDIBitmap",
        ordinal: 487,
    },
    CoredllOrdinalDef {
        name: "SHNotificationAddI",
        ordinal: 1806,
    },
    CoredllOrdinalDef {
        name: "SHNotificationGetDataI",
        ordinal: 1809,
    },
    CoredllOrdinalDef {
        name: "SHNotificationRemoveI",
        ordinal: 1808,
    },
    CoredllOrdinalDef {
        name: "SHNotificationUpdateI",
        ordinal: 1807,
    },
    CoredllOrdinalDef {
        name: "SHSetAppKeyWndAssocI",
        ordinal: 1784,
    },
    CoredllOrdinalDef {
        name: "SHSetNavBarTextI",
        ordinal: 1785,
    },
    CoredllOrdinalDef {
        name: "SHShowOutOfMemory",
        ordinal: 486,
    },
    CoredllOrdinalDef {
        name: "SHSipPreferenceI",
        ordinal: 1786,
    },
    CoredllOrdinalDef {
        name: "SaveDC",
        ordinal: 908,
    },
    CoredllOrdinalDef {
        name: "ScreenToClient",
        ordinal: 255,
    },
    CoredllOrdinalDef {
        name: "ScrollDC",
        ordinal: 985,
    },
    CoredllOrdinalDef {
        name: "ScrollWindowEx",
        ordinal: 289,
    },
    CoredllOrdinalDef {
        name: "SelectClipRgn",
        ordinal: 979,
    },
    CoredllOrdinalDef {
        name: "SelectObject",
        ordinal: 921,
    },
    CoredllOrdinalDef {
        name: "SelectPalette",
        ordinal: 954,
    },
    CoredllOrdinalDef {
        name: "SendDlgItemMessageW",
        ordinal: 685,
    },
    CoredllOrdinalDef {
        name: "SendInput",
        ordinal: 823,
    },
    CoredllOrdinalDef {
        name: "SendMessageTimeout",
        ordinal: 1495,
    },
    CoredllOrdinalDef {
        name: "SendMessageW",
        ordinal: 868,
    },
    CoredllOrdinalDef {
        name: "SendNotifyMessageW",
        ordinal: 869,
    },
    CoredllOrdinalDef {
        name: "ServiceAddPort",
        ordinal: 1515,
    },
    CoredllOrdinalDef {
        name: "ServiceClosePort",
        ordinal: 1759,
    },
    CoredllOrdinalDef {
        name: "ServiceIoControl",
        ordinal: 1514,
    },
    CoredllOrdinalDef {
        name: "ServiceUnbindPorts",
        ordinal: 1516,
    },
    CoredllOrdinalDef {
        name: "SetACP",
        ordinal: 189,
    },
    CoredllOrdinalDef {
        name: "SetAbortProc",
        ordinal: 962,
    },
    CoredllOrdinalDef {
        name: "SetActiveWindow",
        ordinal: 703,
    },
    CoredllOrdinalDef {
        name: "SetAssociatedMenu",
        ordinal: 299,
    },
    CoredllOrdinalDef {
        name: "SetBitmapBits",
        ordinal: 1725,
    },
    CoredllOrdinalDef {
        name: "SetBkColor",
        ordinal: 922,
    },
    CoredllOrdinalDef {
        name: "SetBkMode",
        ordinal: 923,
    },
    CoredllOrdinalDef {
        name: "SetBrushOrgEx",
        ordinal: 943,
    },
    CoredllOrdinalDef {
        name: "SetCapture",
        ordinal: 708,
    },
    CoredllOrdinalDef {
        name: "SetCaretBlinkTime",
        ordinal: 664,
    },
    CoredllOrdinalDef {
        name: "SetCaretPos",
        ordinal: 662,
    },
    CoredllOrdinalDef {
        name: "SetClassLong",
        ordinal: 882,
    },
    CoredllOrdinalDef {
        name: "SetClassLongW",
        ordinal: 880,
    },
    CoredllOrdinalDef {
        name: "SetCleanRebootFlag",
        ordinal: 615,
    },
    CoredllOrdinalDef {
        name: "SetClipboardData",
        ordinal: 671,
    },
    CoredllOrdinalDef {
        name: "SetCommBreak",
        ordinal: 116,
    },
    CoredllOrdinalDef {
        name: "SetCommMask",
        ordinal: 117,
    },
    CoredllOrdinalDef {
        name: "SetCommState",
        ordinal: 118,
    },
    CoredllOrdinalDef {
        name: "SetCommTimeouts",
        ordinal: 119,
    },
    CoredllOrdinalDef {
        name: "SetCurrentUser",
        ordinal: 1501,
    },
    CoredllOrdinalDef {
        name: "SetCursor",
        ordinal: 682,
    },
    CoredllOrdinalDef {
        name: "SetCursorPos",
        ordinal: 736,
    },
    CoredllOrdinalDef {
        name: "SetDIBColorTable",
        ordinal: 1666,
    },
    CoredllOrdinalDef {
        name: "SetDIBitsToDevice",
        ordinal: 1726,
    },
    CoredllOrdinalDef {
        name: "SetDaylightTime",
        ordinal: 547,
    },
    CoredllOrdinalDef {
        name: "SetDbgZone",
        ordinal: 618,
    },
    CoredllOrdinalDef {
        name: "SetDevicePower",
        ordinal: 1678,
    },
    CoredllOrdinalDef {
        name: "SetDlgItemInt",
        ordinal: 700,
    },
    CoredllOrdinalDef {
        name: "SetDlgItemTextW",
        ordinal: 686,
    },
    CoredllOrdinalDef {
        name: "SetEndOfFile",
        ordinal: 178,
    },
    CoredllOrdinalDef {
        name: "SetEventData",
        ordinal: 1528,
    },
    CoredllOrdinalDef {
        name: "SetExceptionHandler",
        ordinal: 583,
    },
    CoredllOrdinalDef {
        name: "SetFileAttributesW",
        ordinal: 169,
    },
    CoredllOrdinalDef {
        name: "SetFilePointer",
        ordinal: 173,
    },
    CoredllOrdinalDef {
        name: "SetFileTime",
        ordinal: 177,
    },
    CoredllOrdinalDef {
        name: "SetFocus",
        ordinal: 704,
    },
    CoredllOrdinalDef {
        name: "SetForegroundWindow",
        ordinal: 702,
    },
    CoredllOrdinalDef {
        name: "SetGwesOOMEvent",
        ordinal: 590,
    },
    CoredllOrdinalDef {
        name: "SetGwesPowerHandler",
        ordinal: 632,
    },
    CoredllOrdinalDef {
        name: "SetHandleOwner",
        ordinal: 625,
    },
    CoredllOrdinalDef {
        name: "SetHardwareWatch",
        ordinal: 634,
    },
    CoredllOrdinalDef {
        name: "SetInterruptEvent",
        ordinal: 158,
    },
    CoredllOrdinalDef {
        name: "SetKMode",
        ordinal: 630,
    },
    CoredllOrdinalDef {
        name: "SetKernelAlarm",
        ordinal: 586,
    },
    CoredllOrdinalDef {
        name: "SetKeyboardTarget",
        ordinal: 710,
    },
    CoredllOrdinalDef {
        name: "SetLastError",
        ordinal: 517,
    },
    CoredllOrdinalDef {
        name: "SetLocalTime",
        ordinal: 24,
    },
    CoredllOrdinalDef {
        name: "SetLocaleInfoW",
        ordinal: 201,
    },
    CoredllOrdinalDef {
        name: "SetLowestScheduledPriority",
        ordinal: 609,
    },
    CoredllOrdinalDef {
        name: "SetMenuItemInfoW",
        ordinal: 853,
    },
    CoredllOrdinalDef {
        name: "SetOEMCP",
        ordinal: 190,
    },
    CoredllOrdinalDef {
        name: "SetOOMEvent",
        ordinal: 1462,
    },
    CoredllOrdinalDef {
        name: "SetObjectOwner",
        ordinal: 984,
    },
    CoredllOrdinalDef {
        name: "SetPaletteEntries",
        ordinal: 951,
    },
    CoredllOrdinalDef {
        name: "SetParent",
        ordinal: 268,
    },
    CoredllOrdinalDef {
        name: "SetPassword",
        ordinal: 238,
    },
    CoredllOrdinalDef {
        name: "SetPasswordActive",
        ordinal: 240,
    },
    CoredllOrdinalDef {
        name: "SetPasswordStatus",
        ordinal: 1537,
    },
    CoredllOrdinalDef {
        name: "SetPixel",
        ordinal: 944,
    },
    CoredllOrdinalDef {
        name: "SetPowerOffHandler",
        ordinal: 631,
    },
    CoredllOrdinalDef {
        name: "SetPowerRequirement",
        ordinal: 1583,
    },
    CoredllOrdinalDef {
        name: "SetProcPermissions",
        ordinal: 611,
    },
    CoredllOrdinalDef {
        name: "SetProp",
        ordinal: 1497,
    },
    CoredllOrdinalDef {
        name: "SetROP2",
        ordinal: 928,
    },
    CoredllOrdinalDef {
        name: "SetRealTime",
        ordinal: 571,
    },
    CoredllOrdinalDef {
        name: "SetRect",
        ordinal: 103,
    },
    CoredllOrdinalDef {
        name: "SetRectEmpty",
        ordinal: 104,
    },
    CoredllOrdinalDef {
        name: "SetRectRgn",
        ordinal: 982,
    },
    CoredllOrdinalDef {
        name: "SetScrollInfo",
        ordinal: 279,
    },
    CoredllOrdinalDef {
        name: "SetScrollPos",
        ordinal: 280,
    },
    CoredllOrdinalDef {
        name: "SetScrollRange",
        ordinal: 281,
    },
    CoredllOrdinalDef {
        name: "SetStdioPathW",
        ordinal: 1150,
    },
    CoredllOrdinalDef {
        name: "SetSysColors",
        ordinal: 890,
    },
    CoredllOrdinalDef {
        name: "SetSystemDefaultLCID",
        ordinal: 214,
    },
    CoredllOrdinalDef {
        name: "SetSystemMemoryDivision",
        ordinal: 337,
    },
    CoredllOrdinalDef {
        name: "SetSystemPowerState",
        ordinal: 1582,
    },
    CoredllOrdinalDef {
        name: "SetSystemTime",
        ordinal: 26,
    },
    CoredllOrdinalDef {
        name: "SetTextAlign",
        ordinal: 1654,
    },
    CoredllOrdinalDef {
        name: "SetTextColor",
        ordinal: 924,
    },
    CoredllOrdinalDef {
        name: "SetThreadContext",
        ordinal: 502,
    },
    CoredllOrdinalDef {
        name: "SetThreadPriority",
        ordinal: 514,
    },
    CoredllOrdinalDef {
        name: "SetTimeZoneBias",
        ordinal: 614,
    },
    CoredllOrdinalDef {
        name: "SetTimeZoneInformation",
        ordinal: 28,
    },
    CoredllOrdinalDef {
        name: "SetTimer",
        ordinal: 875,
    },
    CoredllOrdinalDef {
        name: "SetUserData",
        ordinal: 1502,
    },
    CoredllOrdinalDef {
        name: "SetUserDefaultLCID",
        ordinal: 1795,
    },
    CoredllOrdinalDef {
        name: "SetWindowOrgEx",
        ordinal: 1984,
    },
    CoredllOrdinalDef {
        name: "GetWindowOrgEx",
        ordinal: 1985,
    },
    CoredllOrdinalDef {
        name: "GetWindowExtEx",
        ordinal: 1986,
    },
    CoredllOrdinalDef {
        name: "OffsetViewportOrgEx",
        ordinal: 1987,
    },
    CoredllOrdinalDef {
        name: "GetViewportOrgEx",
        ordinal: 1988,
    },
    CoredllOrdinalDef {
        name: "GetViewportExtEx",
        ordinal: 1989,
    },
    CoredllOrdinalDef {
        name: "SetViewportOrgEx",
        ordinal: 983,
    },
    CoredllOrdinalDef {
        name: "SetWDevicePowerHandler",
        ordinal: 1178,
    },
    CoredllOrdinalDef {
        name: "SetWindowLongW",
        ordinal: 258,
    },
    CoredllOrdinalDef {
        name: "SetWindowPos",
        ordinal: 247,
    },
    CoredllOrdinalDef {
        name: "SetWindowRgn",
        ordinal: 1398,
    },
    CoredllOrdinalDef {
        name: "SetWindowTextW",
        ordinal: 256,
    },
    CoredllOrdinalDef {
        name: "SetWindowsHookExW",
        ordinal: 1202,
    },
    CoredllOrdinalDef {
        name: "SetupComm",
        ordinal: 120,
    },
    CoredllOrdinalDef {
        name: "ShellExecuteEx",
        ordinal: 480,
    },
    CoredllOrdinalDef {
        name: "ShellModalEnd",
        ordinal: 712,
    },
    CoredllOrdinalDef {
        name: "Shell_NotifyIcon",
        ordinal: 481,
    },
    CoredllOrdinalDef {
        name: "ShowCaret",
        ordinal: 661,
    },
    CoredllOrdinalDef {
        name: "ShowCursor",
        ordinal: 737,
    },
    CoredllOrdinalDef {
        name: "ShowStartupWindow",
        ordinal: 1810,
    },
    CoredllOrdinalDef {
        name: "ShowWindow",
        ordinal: 266,
    },
    CoredllOrdinalDef {
        name: "SignalStarted",
        ordinal: 639,
    },
    CoredllOrdinalDef {
        name: "SizeofResource",
        ordinal: 534,
    },
    CoredllOrdinalDef {
        name: "Sleep",
        ordinal: 496,
    },
    CoredllOrdinalDef {
        name: "SleepTillTick",
        ordinal: 1534,
    },
    CoredllOrdinalDef {
        name: "StartDocW",
        ordinal: 963,
    },
    CoredllOrdinalDef {
        name: "StartPage",
        ordinal: 964,
    },
    CoredllOrdinalDef {
        name: "StopDeviceNotifications",
        ordinal: 1505,
    },
    CoredllOrdinalDef {
        name: "StopPowerNotifications",
        ordinal: 1586,
    },
    CoredllOrdinalDef {
        name: "StretchBlt",
        ordinal: 905,
    },
    CoredllOrdinalDef {
        name: "StretchDIBits",
        ordinal: 1667,
    },
    CoredllOrdinalDef {
        name: "StringCbCatA",
        ordinal: 1710,
    },
    CoredllOrdinalDef {
        name: "StringCbCatExA",
        ordinal: 1712,
    },
    CoredllOrdinalDef {
        name: "StringCbCatExW",
        ordinal: 1696,
    },
    CoredllOrdinalDef {
        name: "StringCbCatNA",
        ordinal: 1753,
    },
    CoredllOrdinalDef {
        name: "StringCbCatNExA",
        ordinal: 1755,
    },
    CoredllOrdinalDef {
        name: "StringCbCatNExW",
        ordinal: 1747,
    },
    CoredllOrdinalDef {
        name: "StringCbCatNW",
        ordinal: 1745,
    },
    CoredllOrdinalDef {
        name: "StringCbCatW",
        ordinal: 1694,
    },
    CoredllOrdinalDef {
        name: "StringCbCopyA",
        ordinal: 1706,
    },
    CoredllOrdinalDef {
        name: "StringCbCopyExA",
        ordinal: 1708,
    },
    CoredllOrdinalDef {
        name: "StringCbCopyExW",
        ordinal: 1692,
    },
    CoredllOrdinalDef {
        name: "StringCbCopyNA",
        ordinal: 1751,
    },
    CoredllOrdinalDef {
        name: "StringCbCopyNW",
        ordinal: 1743,
    },
    CoredllOrdinalDef {
        name: "StringCbCopyW",
        ordinal: 1690,
    },
    CoredllOrdinalDef {
        name: "StringCbLengthA",
        ordinal: 1757,
    },
    CoredllOrdinalDef {
        name: "StringCbLengthW",
        ordinal: 1749,
    },
    CoredllOrdinalDef {
        name: "StringCbPrintfA",
        ordinal: 1716,
    },
    CoredllOrdinalDef {
        name: "StringCbPrintfExA",
        ordinal: 1718,
    },
    CoredllOrdinalDef {
        name: "StringCbPrintfExW",
        ordinal: 1702,
    },
    CoredllOrdinalDef {
        name: "StringCbPrintfW",
        ordinal: 1700,
    },
    CoredllOrdinalDef {
        name: "StringCbVPrintfA",
        ordinal: 1714,
    },
    CoredllOrdinalDef {
        name: "StringCbVPrintfExA",
        ordinal: 1720,
    },
    CoredllOrdinalDef {
        name: "StringCbVPrintfExW",
        ordinal: 1704,
    },
    CoredllOrdinalDef {
        name: "StringCbVPrintfW",
        ordinal: 1698,
    },
    CoredllOrdinalDef {
        name: "StringCchCatA",
        ordinal: 1709,
    },
    CoredllOrdinalDef {
        name: "StringCchCatExA",
        ordinal: 1711,
    },
    CoredllOrdinalDef {
        name: "StringCchCatExW",
        ordinal: 1695,
    },
    CoredllOrdinalDef {
        name: "StringCchCatNA",
        ordinal: 1752,
    },
    CoredllOrdinalDef {
        name: "StringCchCatNExA",
        ordinal: 1754,
    },
    CoredllOrdinalDef {
        name: "StringCchCatNExW",
        ordinal: 1746,
    },
    CoredllOrdinalDef {
        name: "StringCchCatNW",
        ordinal: 1744,
    },
    CoredllOrdinalDef {
        name: "StringCchCatW",
        ordinal: 1693,
    },
    CoredllOrdinalDef {
        name: "StringCchCopyA",
        ordinal: 1705,
    },
    CoredllOrdinalDef {
        name: "StringCchCopyExA",
        ordinal: 1707,
    },
    CoredllOrdinalDef {
        name: "StringCchCopyExW",
        ordinal: 1691,
    },
    CoredllOrdinalDef {
        name: "StringCchCopyNA",
        ordinal: 1750,
    },
    CoredllOrdinalDef {
        name: "StringCchCopyNW",
        ordinal: 1742,
    },
    CoredllOrdinalDef {
        name: "StringCchCopyW",
        ordinal: 1689,
    },
    CoredllOrdinalDef {
        name: "StringCchLengthA",
        ordinal: 1756,
    },
    CoredllOrdinalDef {
        name: "StringCchLengthW",
        ordinal: 1748,
    },
    CoredllOrdinalDef {
        name: "StringCchPrintfA",
        ordinal: 1715,
    },
    CoredllOrdinalDef {
        name: "StringCchPrintfExA",
        ordinal: 1717,
    },
    CoredllOrdinalDef {
        name: "StringCchPrintfExW",
        ordinal: 1701,
    },
    CoredllOrdinalDef {
        name: "StringCchPrintfW",
        ordinal: 1699,
    },
    CoredllOrdinalDef {
        name: "StringCchVPrintfA",
        ordinal: 1713,
    },
    CoredllOrdinalDef {
        name: "StringCchVPrintfExA",
        ordinal: 1719,
    },
    CoredllOrdinalDef {
        name: "StringCchVPrintfExW",
        ordinal: 1703,
    },
    CoredllOrdinalDef {
        name: "StringCchVPrintfW",
        ordinal: 1697,
    },
    CoredllOrdinalDef {
        name: "StringCompress",
        ordinal: 591,
    },
    CoredllOrdinalDef {
        name: "StringDecompress",
        ordinal: 592,
    },
    CoredllOrdinalDef {
        name: "SubtractRect",
        ordinal: 105,
    },
    CoredllOrdinalDef {
        name: "SuspendThread",
        ordinal: 499,
    },
    CoredllOrdinalDef {
        name: "SystemIdleTimerReset",
        ordinal: 837,
    },
    CoredllOrdinalDef {
        name: "SystemMemoryLow",
        ordinal: 720,
    },
    CoredllOrdinalDef {
        name: "SystemParametersInfoW",
        ordinal: 89,
    },
    CoredllOrdinalDef {
        name: "SystemStarted",
        ordinal: 1,
    },
    CoredllOrdinalDef {
        name: "SystemTimeToFileTime",
        ordinal: 19,
    },
    CoredllOrdinalDef {
        name: "THCreateSnapshot",
        ordinal: 511,
    },
    CoredllOrdinalDef {
        name: "THGrow",
        ordinal: 512,
    },
    CoredllOrdinalDef {
        name: "TakeCritSec",
        ordinal: 596,
    },
    CoredllOrdinalDef {
        name: "TerminateProcess",
        ordinal: 544,
    },
    CoredllOrdinalDef {
        name: "TerminateThread",
        ordinal: 491,
    },
    CoredllOrdinalDef {
        name: "ThreadAttachAllDLLs",
        ordinal: 561,
    },
    CoredllOrdinalDef {
        name: "ThreadBaseFunc",
        ordinal: 13,
    },
    CoredllOrdinalDef {
        name: "ThreadDetachAllDLLs",
        ordinal: 562,
    },
    CoredllOrdinalDef {
        name: "ThreadExceptionExit",
        ordinal: 1474,
    },
    CoredllOrdinalDef {
        name: "TlsCall",
        ordinal: 520,
    },
    CoredllOrdinalDef {
        name: "TlsGetValue",
        ordinal: 15,
    },
    CoredllOrdinalDef {
        name: "TlsSetValue",
        ordinal: 16,
    },
    CoredllOrdinalDef {
        name: "TouchCalibrate",
        ordinal: 877,
    },
    CoredllOrdinalDef {
        name: "TrackPopupMenuEx",
        ordinal: 845,
    },
    CoredllOrdinalDef {
        name: "TranslateAcceleratorW",
        ordinal: 838,
    },
    CoredllOrdinalDef {
        name: "TranslateCharsetInfo",
        ordinal: 1166,
    },
    CoredllOrdinalDef {
        name: "TranslateMessage",
        ordinal: 870,
    },
    CoredllOrdinalDef {
        name: "TransmitCommChar",
        ordinal: 121,
    },
    CoredllOrdinalDef {
        name: "TransparentImage",
        ordinal: 906,
    },
    CoredllOrdinalDef {
        name: "TryEnterCriticalSection",
        ordinal: 1233,
    },
    CoredllOrdinalDef {
        name: "TurnOffProfiling",
        ordinal: 620,
    },
    CoredllOrdinalDef {
        name: "TurnOnProfiling",
        ordinal: 619,
    },
    CoredllOrdinalDef {
        name: "U_rclose",
        ordinal: 567,
    },
    CoredllOrdinalDef {
        name: "U_rlseek",
        ordinal: 566,
    },
    CoredllOrdinalDef {
        name: "U_ropen",
        ordinal: 563,
    },
    CoredllOrdinalDef {
        name: "U_rread",
        ordinal: 564,
    },
    CoredllOrdinalDef {
        name: "U_rwrite",
        ordinal: 565,
    },
    CoredllOrdinalDef {
        name: "UnhookWindowsHookEx",
        ordinal: 1203,
    },
    CoredllOrdinalDef {
        name: "UnionRect",
        ordinal: 106,
    },
    CoredllOrdinalDef {
        name: "UnlockPages",
        ordinal: 1162,
    },
    CoredllOrdinalDef {
        name: "UnmapViewOfFile",
        ordinal: 550,
    },
    CoredllOrdinalDef {
        name: "UnregisterClassW",
        ordinal: 884,
    },
    CoredllOrdinalDef {
        name: "UnregisterFunc1",
        ordinal: 1156,
    },
    CoredllOrdinalDef {
        name: "UnregisterHotKey",
        ordinal: 836,
    },
    CoredllOrdinalDef {
        name: "UpdateNLSInfo",
        ordinal: 1447,
    },
    CoredllOrdinalDef {
        name: "UpdateNLSInfoEx",
        ordinal: 1796,
    },
    CoredllOrdinalDef {
        name: "UpdateWindow",
        ordinal: 267,
    },
    CoredllOrdinalDef {
        name: "ValidateRect",
        ordinal: 278,
    },
    CoredllOrdinalDef {
        name: "ValidateRgn",
        ordinal: 1616,
    },
    CoredllOrdinalDef {
        name: "VerQueryValueW",
        ordinal: 1459,
    },
    CoredllOrdinalDef {
        name: "VerifyAPIHandle",
        ordinal: 637,
    },
    CoredllOrdinalDef {
        name: "VirtualAlloc",
        ordinal: 524,
    },
    CoredllOrdinalDef {
        name: "VirtualCopy",
        ordinal: 560,
    },
    CoredllOrdinalDef {
        name: "VirtualFree",
        ordinal: 525,
    },
    CoredllOrdinalDef {
        name: "VirtualProtect",
        ordinal: 526,
    },
    CoredllOrdinalDef {
        name: "VirtualQuery",
        ordinal: 527,
    },
    CoredllOrdinalDef {
        name: "VirtualSetAttributes",
        ordinal: 1724,
    },
    CoredllOrdinalDef {
        name: "WNetAddConnection3W",
        ordinal: 444,
    },
    CoredllOrdinalDef {
        name: "WNetCancelConnection2W",
        ordinal: 445,
    },
    CoredllOrdinalDef {
        name: "WNetCloseEnum",
        ordinal: 453,
    },
    CoredllOrdinalDef {
        name: "WNetConnectionDialog1W",
        ordinal: 446,
    },
    CoredllOrdinalDef {
        name: "WNetDisconnectDialog",
        ordinal: 447,
    },
    CoredllOrdinalDef {
        name: "WNetDisconnectDialog1W",
        ordinal: 448,
    },
    CoredllOrdinalDef {
        name: "WNetEnumResourceW",
        ordinal: 454,
    },
    CoredllOrdinalDef {
        name: "WNetGetConnectionW",
        ordinal: 449,
    },
    CoredllOrdinalDef {
        name: "WNetGetUniversalNameW",
        ordinal: 450,
    },
    CoredllOrdinalDef {
        name: "WNetGetUserW",
        ordinal: 451,
    },
    CoredllOrdinalDef {
        name: "WNetOpenEnumW",
        ordinal: 452,
    },
    CoredllOrdinalDef {
        name: "WaitCommEvent",
        ordinal: 122,
    },
    CoredllOrdinalDef {
        name: "WaitForDebugEvent",
        ordinal: 503,
    },
    CoredllOrdinalDef {
        name: "WaitForMultipleObjects",
        ordinal: 498,
    },
    CoredllOrdinalDef {
        name: "WaitForSingleObject",
        ordinal: 497,
    },
    CoredllOrdinalDef {
        name: "WideCharToMultiByte",
        ordinal: 197,
    },
    CoredllOrdinalDef {
        name: "WindowFromPoint",
        ordinal: 252,
    },
    CoredllOrdinalDef {
        name: "WriteDebugLED",
        ordinal: 1155,
    },
    CoredllOrdinalDef {
        name: "WriteFile",
        ordinal: 171,
    },
    CoredllOrdinalDef {
        name: "WriteFileWithSeek",
        ordinal: 718,
    },
    CoredllOrdinalDef {
        name: "WriteMsgQueue",
        ordinal: 1531,
    },
    CoredllOrdinalDef {
        name: "WriteProcessMemory",
        ordinal: 507,
    },
    CoredllOrdinalDef {
        name: "WriteRegistryToOEM",
        ordinal: 1154,
    },
    CoredllOrdinalDef {
        name: "_CountLeadingOnes",
        ordinal: 1625,
    },
    CoredllOrdinalDef {
        name: "_CountLeadingOnes64",
        ordinal: 1626,
    },
    CoredllOrdinalDef {
        name: "_CountLeadingSigns",
        ordinal: 1627,
    },
    CoredllOrdinalDef {
        name: "_CountLeadingSigns64",
        ordinal: 1628,
    },
    CoredllOrdinalDef {
        name: "_CountLeadingZeros",
        ordinal: 1629,
    },
    CoredllOrdinalDef {
        name: "_CountLeadingZeros64",
        ordinal: 1630,
    },
    CoredllOrdinalDef {
        name: "_CountOneBits",
        ordinal: 1631,
    },
    CoredllOrdinalDef {
        name: "_CountOneBits64",
        ordinal: 1632,
    },
    CoredllOrdinalDef {
        name: "_HUGE",
        ordinal: 1181,
    },
    CoredllOrdinalDef {
        name: "_InitStdioLib",
        ordinal: 1151,
    },
    CoredllOrdinalDef {
        name: "_MulHigh",
        ordinal: 1636,
    },
    CoredllOrdinalDef {
        name: "_MulUnsignedHigh",
        ordinal: 1637,
    },
    CoredllOrdinalDef {
        name: "_RtlCheckStack",
        ordinal: 2001,
    },
    CoredllOrdinalDef {
        name: "_XcptFilter",
        ordinal: 1645,
    },
    CoredllOrdinalDef {
        name: "__C_specific_handler",
        ordinal: 87,
    },
    CoredllOrdinalDef {
        name: "__CxxFrameHandler",
        ordinal: 1550,
    },
    CoredllOrdinalDef {
        name: "__CxxThrowException",
        ordinal: 1551,
    },
    CoredllOrdinalDef {
        name: "__d_to_ll",
        ordinal: 2019,
    },
    CoredllOrdinalDef {
        name: "__d_to_ull",
        ordinal: 2021,
    },
    CoredllOrdinalDef {
        name: "__dpadd",
        ordinal: 2023,
    },
    CoredllOrdinalDef {
        name: "__dpcmp",
        ordinal: 2041,
    },
    CoredllOrdinalDef {
        name: "__dpdiv",
        ordinal: 2029,
    },
    CoredllOrdinalDef {
        name: "__dpmul",
        ordinal: 2027,
    },
    CoredllOrdinalDef {
        name: "__dpsub",
        ordinal: 2025,
    },
    CoredllOrdinalDef {
        name: "__dptofp",
        ordinal: 2039,
    },
    CoredllOrdinalDef {
        name: "__dptoli",
        ordinal: 2034,
    },
    CoredllOrdinalDef {
        name: "__dptoul",
        ordinal: 2035,
    },
    CoredllOrdinalDef {
        name: "__eqd",
        ordinal: 2050,
    },
    CoredllOrdinalDef {
        name: "__eqs",
        ordinal: 2044,
    },
    CoredllOrdinalDef {
        name: "__f_to_ll",
        ordinal: 2018,
    },
    CoredllOrdinalDef {
        name: "__f_to_ull",
        ordinal: 2020,
    },
    CoredllOrdinalDef {
        name: "__fpadd",
        ordinal: 2022,
    },
    CoredllOrdinalDef {
        name: "__fpcmp",
        ordinal: 2040,
    },
    CoredllOrdinalDef {
        name: "__fpdiv",
        ordinal: 2028,
    },
    CoredllOrdinalDef {
        name: "__fpmul",
        ordinal: 2026,
    },
    CoredllOrdinalDef {
        name: "__fpsub",
        ordinal: 2024,
    },
    CoredllOrdinalDef {
        name: "__fptodp",
        ordinal: 2038,
    },
    CoredllOrdinalDef {
        name: "__fptoli",
        ordinal: 2030,
    },
    CoredllOrdinalDef {
        name: "__fptoul",
        ordinal: 2031,
    },
    CoredllOrdinalDef {
        name: "__ged",
        ordinal: 2051,
    },
    CoredllOrdinalDef {
        name: "__ges",
        ordinal: 2045,
    },
    CoredllOrdinalDef {
        name: "__gtd",
        ordinal: 2052,
    },
    CoredllOrdinalDef {
        name: "__gts",
        ordinal: 2046,
    },
    CoredllOrdinalDef {
        name: "__led",
        ordinal: 2049,
    },
    CoredllOrdinalDef {
        name: "__les",
        ordinal: 2043,
    },
    CoredllOrdinalDef {
        name: "__litodp",
        ordinal: 2036,
    },
    CoredllOrdinalDef {
        name: "__litofp",
        ordinal: 2032,
    },
    CoredllOrdinalDef {
        name: "__ll_bit_extract",
        ordinal: 2007,
    },
    CoredllOrdinalDef {
        name: "__ll_bit_insert",
        ordinal: 2008,
    },
    CoredllOrdinalDef {
        name: "__ll_div",
        ordinal: 2005,
    },
    CoredllOrdinalDef {
        name: "__ll_lshift",
        ordinal: 2003,
    },
    CoredllOrdinalDef {
        name: "__ll_mul",
        ordinal: 2004,
    },
    CoredllOrdinalDef {
        name: "__ll_rem",
        ordinal: 2006,
    },
    CoredllOrdinalDef {
        name: "__ll_rshift",
        ordinal: 2002,
    },
    CoredllOrdinalDef {
        name: "__ll_to_d",
        ordinal: 2010,
    },
    CoredllOrdinalDef {
        name: "__ll_to_f",
        ordinal: 2009,
    },
    CoredllOrdinalDef {
        name: "__ltd",
        ordinal: 2048,
    },
    CoredllOrdinalDef {
        name: "__lts",
        ordinal: 2042,
    },
    CoredllOrdinalDef {
        name: "__ned",
        ordinal: 2053,
    },
    CoredllOrdinalDef {
        name: "__nes",
        ordinal: 2047,
    },
    CoredllOrdinalDef {
        name: "__strgtold12",
        ordinal: 1089,
    },
    CoredllOrdinalDef {
        name: "__ull_bit_extract",
        ordinal: 2014,
    },
    CoredllOrdinalDef {
        name: "__ull_bit_insert",
        ordinal: 2015,
    },
    CoredllOrdinalDef {
        name: "__ull_div",
        ordinal: 2012,
    },
    CoredllOrdinalDef {
        name: "__ull_rem",
        ordinal: 2013,
    },
    CoredllOrdinalDef {
        name: "__ull_rshift",
        ordinal: 2011,
    },
    CoredllOrdinalDef {
        name: "__ull_to_d",
        ordinal: 2017,
    },
    CoredllOrdinalDef {
        name: "__ull_to_f",
        ordinal: 2016,
    },
    CoredllOrdinalDef {
        name: "__ultodp",
        ordinal: 2037,
    },
    CoredllOrdinalDef {
        name: "__ultofp",
        ordinal: 2033,
    },
    CoredllOrdinalDef {
        name: "_abs64",
        ordinal: 1621,
    },
    CoredllOrdinalDef {
        name: "_atodbl",
        ordinal: 996,
    },
    CoredllOrdinalDef {
        name: "_atoflt",
        ordinal: 997,
    },
    CoredllOrdinalDef {
        name: "_atoi64",
        ordinal: 1418,
    },
    CoredllOrdinalDef {
        name: "_byteswap_uint64",
        ordinal: 1622,
    },
    CoredllOrdinalDef {
        name: "_byteswap_ulong",
        ordinal: 1623,
    },
    CoredllOrdinalDef {
        name: "_byteswap_ushort",
        ordinal: 1624,
    },
    CoredllOrdinalDef {
        name: "_cabs",
        ordinal: 998,
    },
    CoredllOrdinalDef {
        name: "_chgsign",
        ordinal: 1000,
    },
    CoredllOrdinalDef {
        name: "_clearfp",
        ordinal: 1001,
    },
    CoredllOrdinalDef {
        name: "_controlfp",
        ordinal: 1002,
    },
    CoredllOrdinalDef {
        name: "_copysign",
        ordinal: 1003,
    },
    CoredllOrdinalDef {
        name: "_ecvt",
        ordinal: 1008,
    },
    CoredllOrdinalDef {
        name: "_fcloseall",
        ordinal: 1119,
    },
    CoredllOrdinalDef {
        name: "_fcvt",
        ordinal: 1011,
    },
    CoredllOrdinalDef {
        name: "_fileno",
        ordinal: 1124,
    },
    CoredllOrdinalDef {
        name: "_finite",
        ordinal: 1012,
    },
    CoredllOrdinalDef {
        name: "_fltused",
        ordinal: 1093,
    },
    CoredllOrdinalDef {
        name: "_flushall",
        ordinal: 1123,
    },
    CoredllOrdinalDef {
        name: "_fpclass",
        ordinal: 1015,
    },
    CoredllOrdinalDef {
        name: "_fpieee_flt",
        ordinal: 1016,
    },
    CoredllOrdinalDef {
        name: "_fpreset",
        ordinal: 1017,
    },
    CoredllOrdinalDef {
        name: "_frnd",
        ordinal: 1020,
    },
    CoredllOrdinalDef {
        name: "_fsqrt",
        ordinal: 1021,
    },
    CoredllOrdinalDef {
        name: "_gcvt",
        ordinal: 1022,
    },
    CoredllOrdinalDef {
        name: "_getstdfilex",
        ordinal: 1100,
    },
    CoredllOrdinalDef {
        name: "_getws",
        ordinal: 1138,
    },
    CoredllOrdinalDef {
        name: "_hypot",
        ordinal: 1023,
    },
    CoredllOrdinalDef {
        name: "_isctype",
        ordinal: 1417,
    },
    CoredllOrdinalDef {
        name: "_isnan",
        ordinal: 1024,
    },
    CoredllOrdinalDef {
        name: "_isnanf",
        ordinal: 1633,
    },
    CoredllOrdinalDef {
        name: "_isunordered",
        ordinal: 1634,
    },
    CoredllOrdinalDef {
        name: "_isunorderedf",
        ordinal: 1635,
    },
    CoredllOrdinalDef {
        name: "_itoa",
        ordinal: 1025,
    },
    CoredllOrdinalDef {
        name: "_itow",
        ordinal: 1026,
    },
    CoredllOrdinalDef {
        name: "_j0",
        ordinal: 1027,
    },
    CoredllOrdinalDef {
        name: "_j1",
        ordinal: 1028,
    },
    CoredllOrdinalDef {
        name: "_jn",
        ordinal: 1029,
    },
    CoredllOrdinalDef {
        name: "_ld12tod",
        ordinal: 1087,
    },
    CoredllOrdinalDef {
        name: "_ld12tof",
        ordinal: 1088,
    },
    CoredllOrdinalDef {
        name: "_logb",
        ordinal: 1035,
    },
    CoredllOrdinalDef {
        name: "_lrotl",
        ordinal: 1037,
    },
    CoredllOrdinalDef {
        name: "_lrotr",
        ordinal: 1038,
    },
    CoredllOrdinalDef {
        name: "_ltoa",
        ordinal: 1039,
    },
    CoredllOrdinalDef {
        name: "_ltow",
        ordinal: 1040,
    },
    CoredllOrdinalDef {
        name: "_memccpy",
        ordinal: 1042,
    },
    CoredllOrdinalDef {
        name: "_memicmp",
        ordinal: 1045,
    },
    CoredllOrdinalDef {
        name: "_msize",
        ordinal: 1049,
    },
    CoredllOrdinalDef {
        name: "_nextafter",
        ordinal: 1050,
    },
    CoredllOrdinalDef {
        name: "_purecall",
        ordinal: 1092,
    },
    CoredllOrdinalDef {
        name: "_putws",
        ordinal: 1139,
    },
    CoredllOrdinalDef {
        name: "_rotl",
        ordinal: 1055,
    },
    CoredllOrdinalDef {
        name: "_rotl64",
        ordinal: 1638,
    },
    CoredllOrdinalDef {
        name: "_rotr",
        ordinal: 1056,
    },
    CoredllOrdinalDef {
        name: "_rotr64",
        ordinal: 1639,
    },
    CoredllOrdinalDef {
        name: "_scalb",
        ordinal: 1057,
    },
    CoredllOrdinalDef {
        name: "_setjmp",
        ordinal: 2000,
    },
    CoredllOrdinalDef {
        name: "_setmode",
        ordinal: 1187,
    },
    CoredllOrdinalDef {
        name: "_snprintf",
        ordinal: 729,
    },
    CoredllOrdinalDef {
        name: "_snwprintf",
        ordinal: 1096,
    },
    CoredllOrdinalDef {
        name: "_statusfp",
        ordinal: 1062,
    },
    CoredllOrdinalDef {
        name: "_strdup",
        ordinal: 1409,
    },
    CoredllOrdinalDef {
        name: "_stricmp",
        ordinal: 1410,
    },
    CoredllOrdinalDef {
        name: "_strlwr",
        ordinal: 1415,
    },
    CoredllOrdinalDef {
        name: "_strnicmp",
        ordinal: 1411,
    },
    CoredllOrdinalDef {
        name: "_strnset",
        ordinal: 1412,
    },
    CoredllOrdinalDef {
        name: "_strrev",
        ordinal: 1413,
    },
    CoredllOrdinalDef {
        name: "_strset",
        ordinal: 1414,
    },
    CoredllOrdinalDef {
        name: "_strupr",
        ordinal: 1416,
    },
    CoredllOrdinalDef {
        name: "_swab",
        ordinal: 1074,
    },
    CoredllOrdinalDef {
        name: "_ultoa",
        ordinal: 1079,
    },
    CoredllOrdinalDef {
        name: "_ultow",
        ordinal: 1080,
    },
    CoredllOrdinalDef {
        name: "_vsnprintf",
        ordinal: 1147,
    },
    CoredllOrdinalDef {
        name: "_vsnwprintf",
        ordinal: 1132,
    },
    CoredllOrdinalDef {
        name: "_wcsdup",
        ordinal: 74,
    },
    CoredllOrdinalDef {
        name: "_wcsicmp",
        ordinal: 230,
    },
    CoredllOrdinalDef {
        name: "_wcslwr",
        ordinal: 231,
    },
    CoredllOrdinalDef {
        name: "_wcsnicmp",
        ordinal: 229,
    },
    CoredllOrdinalDef {
        name: "_wcsnset",
        ordinal: 67,
    },
    CoredllOrdinalDef {
        name: "_wcsrev",
        ordinal: 70,
    },
    CoredllOrdinalDef {
        name: "_wcsset",
        ordinal: 71,
    },
    CoredllOrdinalDef {
        name: "_wcsupr",
        ordinal: 232,
    },
    CoredllOrdinalDef {
        name: "_wfdopen",
        ordinal: 1117,
    },
    CoredllOrdinalDef {
        name: "_wfopen",
        ordinal: 1145,
    },
    CoredllOrdinalDef {
        name: "_wfreopen",
        ordinal: 1201,
    },
    CoredllOrdinalDef {
        name: "_wtol",
        ordinal: 78,
    },
    CoredllOrdinalDef {
        name: "_wtoll",
        ordinal: 79,
    },
    CoredllOrdinalDef {
        name: "_y0",
        ordinal: 1084,
    },
    CoredllOrdinalDef {
        name: "_y1",
        ordinal: 1085,
    },
    CoredllOrdinalDef {
        name: "_yn",
        ordinal: 1086,
    },
    CoredllOrdinalDef {
        name: "abs",
        ordinal: 988,
    },
    CoredllOrdinalDef {
        name: "acos",
        ordinal: 989,
    },
    CoredllOrdinalDef {
        name: "asin",
        ordinal: 990,
    },
    CoredllOrdinalDef {
        name: "atan",
        ordinal: 991,
    },
    CoredllOrdinalDef {
        name: "atan2",
        ordinal: 992,
    },
    CoredllOrdinalDef {
        name: "atof",
        ordinal: 995,
    },
    CoredllOrdinalDef {
        name: "atoi",
        ordinal: 993,
    },
    CoredllOrdinalDef {
        name: "atol",
        ordinal: 994,
    },
    CoredllOrdinalDef {
        name: "calloc",
        ordinal: 1346,
    },
    CoredllOrdinalDef {
        name: "ceil",
        ordinal: 999,
    },
    CoredllOrdinalDef {
        name: "ceilf",
        ordinal: 1640,
    },
    CoredllOrdinalDef {
        name: "clearerr",
        ordinal: 1127,
    },
    CoredllOrdinalDef {
        name: "cos",
        ordinal: 1004,
    },
    CoredllOrdinalDef {
        name: "cosh",
        ordinal: 1005,
    },
    CoredllOrdinalDef {
        name: "difftime",
        ordinal: 1006,
    },
    CoredllOrdinalDef {
        name: "div",
        ordinal: 1007,
    },
    CoredllOrdinalDef {
        name: "exp",
        ordinal: 1009,
    },
    CoredllOrdinalDef {
        name: "fabs",
        ordinal: 1010,
    },
    CoredllOrdinalDef {
        name: "fabsf",
        ordinal: 1641,
    },
    CoredllOrdinalDef {
        name: "fclose",
        ordinal: 1118,
    },
    CoredllOrdinalDef {
        name: "feof",
        ordinal: 1125,
    },
    CoredllOrdinalDef {
        name: "ferror",
        ordinal: 1126,
    },
    CoredllOrdinalDef {
        name: "fflush",
        ordinal: 1122,
    },
    CoredllOrdinalDef {
        name: "fgetc",
        ordinal: 1108,
    },
    CoredllOrdinalDef {
        name: "fgetpos",
        ordinal: 1128,
    },
    CoredllOrdinalDef {
        name: "fgets",
        ordinal: 1109,
    },
    CoredllOrdinalDef {
        name: "fgetwc",
        ordinal: 1140,
    },
    CoredllOrdinalDef {
        name: "fgetws",
        ordinal: 1143,
    },
    CoredllOrdinalDef {
        name: "floor",
        ordinal: 1013,
    },
    CoredllOrdinalDef {
        name: "floorf",
        ordinal: 1642,
    },
    CoredllOrdinalDef {
        name: "fmod",
        ordinal: 1014,
    },
    CoredllOrdinalDef {
        name: "fmodf",
        ordinal: 1643,
    },
    CoredllOrdinalDef {
        name: "fopen",
        ordinal: 1113,
    },
    CoredllOrdinalDef {
        name: "fprintf",
        ordinal: 1115,
    },
    CoredllOrdinalDef {
        name: "fputc",
        ordinal: 1110,
    },
    CoredllOrdinalDef {
        name: "fputs",
        ordinal: 1111,
    },
    CoredllOrdinalDef {
        name: "fputwc",
        ordinal: 1141,
    },
    CoredllOrdinalDef {
        name: "fputws",
        ordinal: 1144,
    },
    CoredllOrdinalDef {
        name: "fread",
        ordinal: 1120,
    },
    CoredllOrdinalDef {
        name: "free",
        ordinal: 1018,
    },
    CoredllOrdinalDef {
        name: "frexp",
        ordinal: 1019,
    },
    CoredllOrdinalDef {
        name: "fscanf",
        ordinal: 1114,
    },
    CoredllOrdinalDef {
        name: "fseek",
        ordinal: 1130,
    },
    CoredllOrdinalDef {
        name: "fsetpos",
        ordinal: 1129,
    },
    CoredllOrdinalDef {
        name: "ftell",
        ordinal: 1131,
    },
    CoredllOrdinalDef {
        name: "fwprintf",
        ordinal: 867,
    },
    CoredllOrdinalDef {
        name: "fwrite",
        ordinal: 1121,
    },
    CoredllOrdinalDef {
        name: "fwscanf",
        ordinal: 735,
    },
    CoredllOrdinalDef {
        name: "getchar",
        ordinal: 1104,
    },
    CoredllOrdinalDef {
        name: "gets",
        ordinal: 1106,
    },
    CoredllOrdinalDef {
        name: "getwchar",
        ordinal: 1136,
    },
    CoredllOrdinalDef {
        name: "iswctype",
        ordinal: 193,
    },
    CoredllOrdinalDef {
        name: "keybd_event",
        ordinal: 833,
    },
    CoredllOrdinalDef {
        name: "labs",
        ordinal: 1030,
    },
    CoredllOrdinalDef {
        name: "ldexp",
        ordinal: 1031,
    },
    CoredllOrdinalDef {
        name: "ldiv",
        ordinal: 1032,
    },
    CoredllOrdinalDef {
        name: "log",
        ordinal: 1033,
    },
    CoredllOrdinalDef {
        name: "log10",
        ordinal: 1034,
    },
    CoredllOrdinalDef {
        name: "longjmp",
        ordinal: 1036,
    },
    CoredllOrdinalDef {
        name: "lstrcmpW",
        ordinal: 227,
    },
    CoredllOrdinalDef {
        name: "lstrcmpiW",
        ordinal: 228,
    },
    CoredllOrdinalDef {
        name: "malloc",
        ordinal: 1041,
    },
    CoredllOrdinalDef {
        name: "mbstowcs",
        ordinal: 76,
    },
    CoredllOrdinalDef {
        name: "memchr",
        ordinal: 31,
    },
    CoredllOrdinalDef {
        name: "memcmp",
        ordinal: 1043,
    },
    CoredllOrdinalDef {
        name: "memcpy",
        ordinal: 1044,
    },
    CoredllOrdinalDef {
        name: "memmove",
        ordinal: 1046,
    },
    CoredllOrdinalDef {
        name: "memset",
        ordinal: 1047,
    },
    CoredllOrdinalDef {
        name: "mixerClose",
        ordinal: 1598,
    },
    CoredllOrdinalDef {
        name: "mixerGetControlDetails",
        ordinal: 1589,
    },
    CoredllOrdinalDef {
        name: "mixerGetDevCaps",
        ordinal: 1591,
    },
    CoredllOrdinalDef {
        name: "mixerGetID",
        ordinal: 1590,
    },
    CoredllOrdinalDef {
        name: "mixerGetLineControls",
        ordinal: 1592,
    },
    CoredllOrdinalDef {
        name: "mixerGetLineInfo",
        ordinal: 1593,
    },
    CoredllOrdinalDef {
        name: "mixerGetNumDevs",
        ordinal: 1594,
    },
    CoredllOrdinalDef {
        name: "mixerMessage",
        ordinal: 1596,
    },
    CoredllOrdinalDef {
        name: "mixerOpen",
        ordinal: 1595,
    },
    CoredllOrdinalDef {
        name: "mixerSetControlDetails",
        ordinal: 1597,
    },
    CoredllOrdinalDef {
        name: "modf",
        ordinal: 1048,
    },
    CoredllOrdinalDef {
        name: "mouse_event",
        ordinal: 824,
    },
    CoredllOrdinalDef {
        name: "pow",
        ordinal: 1051,
    },
    CoredllOrdinalDef {
        name: "printf",
        ordinal: 1102,
    },
    CoredllOrdinalDef {
        name: "putchar",
        ordinal: 1105,
    },
    CoredllOrdinalDef {
        name: "puts",
        ordinal: 1107,
    },
    CoredllOrdinalDef {
        name: "putwchar",
        ordinal: 1137,
    },
    CoredllOrdinalDef {
        name: "qsort",
        ordinal: 1052,
    },
    CoredllOrdinalDef {
        name: "rand",
        ordinal: 1053,
    },
    CoredllOrdinalDef {
        name: "realloc",
        ordinal: 1054,
    },
    CoredllOrdinalDef {
        name: "scanf",
        ordinal: 1101,
    },
    CoredllOrdinalDef {
        name: "setvbuf",
        ordinal: 1608,
    },
    CoredllOrdinalDef {
        name: "sin",
        ordinal: 1058,
    },
    CoredllOrdinalDef {
        name: "sinh",
        ordinal: 1059,
    },
    CoredllOrdinalDef {
        name: "sndPlaySoundW",
        ordinal: 377,
    },
    CoredllOrdinalDef {
        name: "sprintf",
        ordinal: 719,
    },
    CoredllOrdinalDef {
        name: "sqrt",
        ordinal: 1060,
    },
    CoredllOrdinalDef {
        name: "sqrtf",
        ordinal: 1644,
    },
    CoredllOrdinalDef {
        name: "srand",
        ordinal: 1061,
    },
    CoredllOrdinalDef {
        name: "sscanf",
        ordinal: 653,
    },
    CoredllOrdinalDef {
        name: "strcat",
        ordinal: 1063,
    },
    CoredllOrdinalDef {
        name: "strchr",
        ordinal: 1064,
    },
    CoredllOrdinalDef {
        name: "strcmp",
        ordinal: 1065,
    },
    CoredllOrdinalDef {
        name: "strcpy",
        ordinal: 1066,
    },
    CoredllOrdinalDef {
        name: "strcspn",
        ordinal: 1067,
    },
    CoredllOrdinalDef {
        name: "strlen",
        ordinal: 1068,
    },
    CoredllOrdinalDef {
        name: "strncat",
        ordinal: 1069,
    },
    CoredllOrdinalDef {
        name: "strncmp",
        ordinal: 1070,
    },
    CoredllOrdinalDef {
        name: "strncpy",
        ordinal: 1071,
    },
    CoredllOrdinalDef {
        name: "strpbrk",
        ordinal: 1406,
    },
    CoredllOrdinalDef {
        name: "strrchr",
        ordinal: 1407,
    },
    CoredllOrdinalDef {
        name: "strspn",
        ordinal: 1408,
    },
    CoredllOrdinalDef {
        name: "strstr",
        ordinal: 1072,
    },
    CoredllOrdinalDef {
        name: "strtod",
        ordinal: 1403,
    },
    CoredllOrdinalDef {
        name: "strtok",
        ordinal: 1073,
    },
    CoredllOrdinalDef {
        name: "strtol",
        ordinal: 1404,
    },
    CoredllOrdinalDef {
        name: "strtoul",
        ordinal: 1405,
    },
    CoredllOrdinalDef {
        name: "swprintf",
        ordinal: 1097,
    },
    CoredllOrdinalDef {
        name: "swscanf",
        ordinal: 1098,
    },
    CoredllOrdinalDef {
        name: "tan",
        ordinal: 1075,
    },
    CoredllOrdinalDef {
        name: "tanh",
        ordinal: 1076,
    },
    CoredllOrdinalDef {
        name: "tolower",
        ordinal: 1090,
    },
    CoredllOrdinalDef {
        name: "toupper",
        ordinal: 1091,
    },
    CoredllOrdinalDef {
        name: "towlower",
        ordinal: 194,
    },
    CoredllOrdinalDef {
        name: "towupper",
        ordinal: 195,
    },
    CoredllOrdinalDef {
        name: "ungetc",
        ordinal: 1112,
    },
    CoredllOrdinalDef {
        name: "ungetwc",
        ordinal: 1142,
    },
    CoredllOrdinalDef {
        name: "vfprintf",
        ordinal: 1116,
    },
    CoredllOrdinalDef {
        name: "vfwprintf",
        ordinal: 721,
    },
    CoredllOrdinalDef {
        name: "vprintf",
        ordinal: 1103,
    },
    CoredllOrdinalDef {
        name: "vsprintf",
        ordinal: 1146,
    },
    CoredllOrdinalDef {
        name: "vswprintf",
        ordinal: 1099,
    },
    CoredllOrdinalDef {
        name: "vwprintf",
        ordinal: 1135,
    },
    CoredllOrdinalDef {
        name: "waveInAddBuffer",
        ordinal: 406,
    },
    CoredllOrdinalDef {
        name: "waveInClose",
        ordinal: 403,
    },
    CoredllOrdinalDef {
        name: "waveInGetDevCaps",
        ordinal: 401,
    },
    CoredllOrdinalDef {
        name: "waveInGetErrorText",
        ordinal: 402,
    },
    CoredllOrdinalDef {
        name: "waveInGetID",
        ordinal: 411,
    },
    CoredllOrdinalDef {
        name: "waveInGetNumDevs",
        ordinal: 400,
    },
    CoredllOrdinalDef {
        name: "waveInGetPosition",
        ordinal: 410,
    },
    CoredllOrdinalDef {
        name: "waveInMessage",
        ordinal: 412,
    },
    CoredllOrdinalDef {
        name: "waveInOpen",
        ordinal: 413,
    },
    CoredllOrdinalDef {
        name: "waveInPrepareHeader",
        ordinal: 404,
    },
    CoredllOrdinalDef {
        name: "waveInReset",
        ordinal: 409,
    },
    CoredllOrdinalDef {
        name: "waveInStart",
        ordinal: 407,
    },
    CoredllOrdinalDef {
        name: "waveInStop",
        ordinal: 408,
    },
    CoredllOrdinalDef {
        name: "waveInUnprepareHeader",
        ordinal: 405,
    },
    CoredllOrdinalDef {
        name: "waveOutBreakLoop",
        ordinal: 391,
    },
    CoredllOrdinalDef {
        name: "waveOutClose",
        ordinal: 384,
    },
    CoredllOrdinalDef {
        name: "waveOutGetDevCaps",
        ordinal: 380,
    },
    CoredllOrdinalDef {
        name: "waveOutGetErrorText",
        ordinal: 383,
    },
    CoredllOrdinalDef {
        name: "waveOutGetID",
        ordinal: 397,
    },
    CoredllOrdinalDef {
        name: "waveOutGetNumDevs",
        ordinal: 379,
    },
    CoredllOrdinalDef {
        name: "waveOutGetPitch",
        ordinal: 393,
    },
    CoredllOrdinalDef {
        name: "waveOutGetPlaybackRate",
        ordinal: 395,
    },
    CoredllOrdinalDef {
        name: "waveOutGetPosition",
        ordinal: 392,
    },
    CoredllOrdinalDef {
        name: "waveOutGetVolume",
        ordinal: 381,
    },
    CoredllOrdinalDef {
        name: "waveOutMessage",
        ordinal: 398,
    },
    CoredllOrdinalDef {
        name: "waveOutOpen",
        ordinal: 399,
    },
    CoredllOrdinalDef {
        name: "waveOutPause",
        ordinal: 388,
    },
    CoredllOrdinalDef {
        name: "waveOutPrepareHeader",
        ordinal: 385,
    },
    CoredllOrdinalDef {
        name: "waveOutReset",
        ordinal: 390,
    },
    CoredllOrdinalDef {
        name: "waveOutRestart",
        ordinal: 389,
    },
    CoredllOrdinalDef {
        name: "waveOutSetPitch",
        ordinal: 394,
    },
    CoredllOrdinalDef {
        name: "waveOutSetPlaybackRate",
        ordinal: 396,
    },
    CoredllOrdinalDef {
        name: "waveOutSetVolume",
        ordinal: 382,
    },
    CoredllOrdinalDef {
        name: "waveOutUnprepareHeader",
        ordinal: 386,
    },
    CoredllOrdinalDef {
        name: "waveOutWrite",
        ordinal: 387,
    },
    CoredllOrdinalDef {
        name: "wcscat",
        ordinal: 58,
    },
    CoredllOrdinalDef {
        name: "wcschr",
        ordinal: 59,
    },
    CoredllOrdinalDef {
        name: "wcscmp",
        ordinal: 60,
    },
    CoredllOrdinalDef {
        name: "wcscpy",
        ordinal: 61,
    },
    CoredllOrdinalDef {
        name: "wcscspn",
        ordinal: 62,
    },
    CoredllOrdinalDef {
        name: "wcslen",
        ordinal: 63,
    },
    CoredllOrdinalDef {
        name: "wcsncat",
        ordinal: 64,
    },
    CoredllOrdinalDef {
        name: "wcsncmp",
        ordinal: 65,
    },
    CoredllOrdinalDef {
        name: "wcsncpy",
        ordinal: 66,
    },
    CoredllOrdinalDef {
        name: "wcspbrk",
        ordinal: 68,
    },
    CoredllOrdinalDef {
        name: "wcsrchr",
        ordinal: 69,
    },
    CoredllOrdinalDef {
        name: "wcsspn",
        ordinal: 72,
    },
    CoredllOrdinalDef {
        name: "wcsstr",
        ordinal: 73,
    },
    CoredllOrdinalDef {
        name: "wcstod",
        ordinal: 1081,
    },
    CoredllOrdinalDef {
        name: "wcstok",
        ordinal: 77,
    },
    CoredllOrdinalDef {
        name: "wcstol",
        ordinal: 1082,
    },
    CoredllOrdinalDef {
        name: "wcstombs",
        ordinal: 75,
    },
    CoredllOrdinalDef {
        name: "wcstoul",
        ordinal: 1083,
    },
    CoredllOrdinalDef {
        name: "wprintf",
        ordinal: 1134,
    },
    CoredllOrdinalDef {
        name: "wscanf",
        ordinal: 1133,
    },
    CoredllOrdinalDef {
        name: "wsprintfW",
        ordinal: 56,
    },
    CoredllOrdinalDef {
        name: "wvsprintfW",
        ordinal: 57,
    },
];

pub const COREDLL_EXPORT_INDEX: &[Option<CoredllOrdinalDef>; 1764] = &[
    None,
    Some(CoredllOrdinalDef {
        name: "SystemMemoryLow",
        ordinal: 720,
    }),
    Some(CoredllOrdinalDef {
        name: "InitializeCriticalSection",
        ordinal: 2,
    }),
    Some(CoredllOrdinalDef {
        name: "DeleteCriticalSection",
        ordinal: 3,
    }),
    Some(CoredllOrdinalDef {
        name: "EnterCriticalSection",
        ordinal: 4,
    }),
    Some(CoredllOrdinalDef {
        name: "LeaveCriticalSection",
        ordinal: 5,
    }),
    Some(CoredllOrdinalDef {
        name: "ExitThread",
        ordinal: 6,
    }),
    Some(CoredllOrdinalDef {
        name: "ThreadExceptionExit",
        ordinal: 1474,
    }),
    Some(CoredllOrdinalDef {
        name: "PSLNotify",
        ordinal: 7,
    }),
    Some(CoredllOrdinalDef {
        name: "InitLocale",
        ordinal: 8,
    }),
    Some(CoredllOrdinalDef {
        name: "ReinitLocale",
        ordinal: 1799,
    }),
    Some(CoredllOrdinalDef {
        name: "IsProcessDying",
        ordinal: 1213,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "InterlockedIncrement",
        ordinal: 10,
    }),
    Some(CoredllOrdinalDef {
        name: "InterlockedDecrement",
        ordinal: 11,
    }),
    Some(CoredllOrdinalDef {
        name: "InterlockedExchange",
        ordinal: 12,
    }),
    Some(CoredllOrdinalDef {
        name: "InterlockedExchangeAdd",
        ordinal: 1491,
    }),
    Some(CoredllOrdinalDef {
        name: "InterlockedCompareExchange",
        ordinal: 1492,
    }),
    Some(CoredllOrdinalDef {
        name: "InterlockedTestExchange",
        ordinal: 9,
    }),
    Some(CoredllOrdinalDef {
        name: "InterlockedIncrement",
        ordinal: 10,
    }),
    Some(CoredllOrdinalDef {
        name: "InterlockedDecrement",
        ordinal: 11,
    }),
    Some(CoredllOrdinalDef {
        name: "InterlockedExchange",
        ordinal: 12,
    }),
    Some(CoredllOrdinalDef {
        name: "InterlockedExchangeAdd",
        ordinal: 1491,
    }),
    Some(CoredllOrdinalDef {
        name: "InterlockedCompareExchange",
        ordinal: 1492,
    }),
    Some(CoredllOrdinalDef {
        name: "ThreadBaseFunc",
        ordinal: 13,
    }),
    Some(CoredllOrdinalDef {
        name: "MainThreadBaseFunc",
        ordinal: 14,
    }),
    Some(CoredllOrdinalDef {
        name: "ComThreadBaseFunc",
        ordinal: 1240,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateLocaleView",
        ordinal: 1466,
    }),
    Some(CoredllOrdinalDef {
        name: "TlsGetValue",
        ordinal: 15,
    }),
    Some(CoredllOrdinalDef {
        name: "TlsSetValue",
        ordinal: 16,
    }),
    Some(CoredllOrdinalDef {
        name: "GetVersionEx",
        ordinal: 17,
    }),
    Some(CoredllOrdinalDef {
        name: "GetVersionExW",
        ordinal: 717,
    }),
    Some(CoredllOrdinalDef {
        name: "CompareFileTime",
        ordinal: 18,
    }),
    Some(CoredllOrdinalDef {
        name: "SystemTimeToFileTime",
        ordinal: 19,
    }),
    Some(CoredllOrdinalDef {
        name: "FileTimeToSystemTime",
        ordinal: 20,
    }),
    Some(CoredllOrdinalDef {
        name: "FileTimeToLocalFileTime",
        ordinal: 21,
    }),
    Some(CoredllOrdinalDef {
        name: "LocalFileTimeToFileTime",
        ordinal: 22,
    }),
    Some(CoredllOrdinalDef {
        name: "GetLocalTime",
        ordinal: 23,
    }),
    Some(CoredllOrdinalDef {
        name: "SetLocalTime",
        ordinal: 24,
    }),
    Some(CoredllOrdinalDef {
        name: "GetSystemTime",
        ordinal: 25,
    }),
    Some(CoredllOrdinalDef {
        name: "SetSystemTime",
        ordinal: 26,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "GetTimeZoneInformation",
        ordinal: 27,
    }),
    Some(CoredllOrdinalDef {
        name: "SetTimeZoneInformation",
        ordinal: 28,
    }),
    Some(CoredllOrdinalDef {
        name: "GetCurrentFT",
        ordinal: 29,
    }),
    Some(CoredllOrdinalDef {
        name: "IsAPIReady",
        ordinal: 30,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "GetAPIAddress",
        ordinal: 32,
    }),
    Some(CoredllOrdinalDef {
        name: "CeEventHasOccurred",
        ordinal: 479,
    }),
    Some(CoredllOrdinalDef {
        name: "GetCRTFlags",
        ordinal: 1228,
    }),
    Some(CoredllOrdinalDef {
        name: "CeZeroPointer",
        ordinal: 1781,
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "CeModuleJit",
        ordinal: 53,
    }),
    Some(CoredllOrdinalDef {
        name: "LocalAlloc",
        ordinal: 33,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "LocalReAlloc",
        ordinal: 34,
    }),
    Some(CoredllOrdinalDef {
        name: "LocalSize",
        ordinal: 35,
    }),
    Some(CoredllOrdinalDef {
        name: "LocalFree",
        ordinal: 36,
    }),
    Some(CoredllOrdinalDef {
        name: "RemoteHeapAlloc",
        ordinal: 1604,
    }),
    Some(CoredllOrdinalDef {
        name: "RemoteHeapReAlloc",
        ordinal: 1605,
    }),
    Some(CoredllOrdinalDef {
        name: "RemoteHeapFree",
        ordinal: 1606,
    }),
    Some(CoredllOrdinalDef {
        name: "RemoteHeapSize",
        ordinal: 1607,
    }),
    Some(CoredllOrdinalDef {
        name: "RemoteLocalAlloc",
        ordinal: 37,
    }),
    Some(CoredllOrdinalDef {
        name: "RemoteLocalReAlloc",
        ordinal: 38,
    }),
    Some(CoredllOrdinalDef {
        name: "RemoteLocalSize",
        ordinal: 39,
    }),
    Some(CoredllOrdinalDef {
        name: "RemoteLocalFree",
        ordinal: 40,
    }),
    Some(CoredllOrdinalDef {
        name: "LocalAllocInProcess",
        ordinal: 41,
    }),
    Some(CoredllOrdinalDef {
        name: "LocalFreeInProcess",
        ordinal: 42,
    }),
    Some(CoredllOrdinalDef {
        name: "LocalSizeInProcess",
        ordinal: 43,
    }),
    Some(CoredllOrdinalDef {
        name: "HeapCreate",
        ordinal: 44,
    }),
    Some(CoredllOrdinalDef {
        name: "HeapDestroy",
        ordinal: 45,
    }),
    Some(CoredllOrdinalDef {
        name: "HeapAlloc",
        ordinal: 46,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "HeapReAlloc",
        ordinal: 47,
    }),
    Some(CoredllOrdinalDef {
        name: "HeapSize",
        ordinal: 48,
    }),
    Some(CoredllOrdinalDef {
        name: "HeapFree",
        ordinal: 49,
    }),
    Some(CoredllOrdinalDef {
        name: "GetProcessHeap",
        ordinal: 50,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "HeapValidate",
        ordinal: 51,
    }),
    Some(CoredllOrdinalDef {
        name: "GetHeapSnapshot",
        ordinal: 52,
    }),
    Some(CoredllOrdinalDef {
        name: "CompactAllHeaps",
        ordinal: 54,
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "wsprintfW",
        ordinal: 56,
    }),
    Some(CoredllOrdinalDef {
        name: "wvsprintfW",
        ordinal: 57,
    }),
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "StringCchCopyW",
        ordinal: 1689,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbCopyW",
        ordinal: 1690,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchCopyExW",
        ordinal: 1691,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbCopyExW",
        ordinal: 1692,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchCopyNW",
        ordinal: 1742,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbCopyNW",
        ordinal: 1743,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchCatW",
        ordinal: 1693,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbCatW",
        ordinal: 1694,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchCatExW",
        ordinal: 1695,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbCatExW",
        ordinal: 1696,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchCatNW",
        ordinal: 1744,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbCatNW",
        ordinal: 1745,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchCatNExW",
        ordinal: 1746,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbCatNExW",
        ordinal: 1747,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchVPrintfW",
        ordinal: 1697,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbVPrintfW",
        ordinal: 1698,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchPrintfW",
        ordinal: 1699,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbPrintfW",
        ordinal: 1700,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchPrintfExW",
        ordinal: 1701,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbPrintfExW",
        ordinal: 1702,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchVPrintfExW",
        ordinal: 1703,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbVPrintfExW",
        ordinal: 1704,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchLengthW",
        ordinal: 1748,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbLengthW",
        ordinal: 1749,
    }),
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "Random",
        ordinal: 80,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "ProfileStart",
        ordinal: 82,
    }),
    Some(CoredllOrdinalDef {
        name: "ProfileStop",
        ordinal: 83,
    }),
    Some(CoredllOrdinalDef {
        name: "ProfileCaptureStatus",
        ordinal: 1800,
    }),
    Some(CoredllOrdinalDef {
        name: "ProfileStartEx",
        ordinal: 1801,
    }),
    Some(CoredllOrdinalDef {
        name: "CeLogData",
        ordinal: 1451,
    }),
    Some(CoredllOrdinalDef {
        name: "CeLogSetZones",
        ordinal: 1452,
    }),
    Some(CoredllOrdinalDef {
        name: "CeLogGetZones",
        ordinal: 1681,
    }),
    Some(CoredllOrdinalDef {
        name: "CeLogReSync",
        ordinal: 1467,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "GlobalMemoryStatus",
        ordinal: 88,
    }),
    Some(CoredllOrdinalDef {
        name: "SystemParametersInfoW",
        ordinal: 89,
    }),
    Some(CoredllOrdinalDef {
        name: "CeGetRandomSeed",
        ordinal: 1443,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateDIBSection",
        ordinal: 90,
    }),
    Some(CoredllOrdinalDef {
        name: "EqualRgn",
        ordinal: 91,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateAcceleratorTableW",
        ordinal: 92,
    }),
    Some(CoredllOrdinalDef {
        name: "DestroyAcceleratorTable",
        ordinal: 93,
    }),
    Some(CoredllOrdinalDef {
        name: "LoadAcceleratorsW",
        ordinal: 94,
    }),
    Some(CoredllOrdinalDef {
        name: "CopyRect",
        ordinal: 96,
    }),
    Some(CoredllOrdinalDef {
        name: "EqualRect",
        ordinal: 97,
    }),
    Some(CoredllOrdinalDef {
        name: "InflateRect",
        ordinal: 98,
    }),
    Some(CoredllOrdinalDef {
        name: "IntersectRect",
        ordinal: 99,
    }),
    Some(CoredllOrdinalDef {
        name: "IsRectEmpty",
        ordinal: 100,
    }),
    Some(CoredllOrdinalDef {
        name: "OffsetRect",
        ordinal: 101,
    }),
    Some(CoredllOrdinalDef {
        name: "PtInRect",
        ordinal: 102,
    }),
    Some(CoredllOrdinalDef {
        name: "SetRect",
        ordinal: 103,
    }),
    Some(CoredllOrdinalDef {
        name: "SetRectEmpty",
        ordinal: 104,
    }),
    Some(CoredllOrdinalDef {
        name: "SubtractRect",
        ordinal: 105,
    }),
    Some(CoredllOrdinalDef {
        name: "UnionRect",
        ordinal: 106,
    }),
    Some(CoredllOrdinalDef {
        name: "ClearCommBreak",
        ordinal: 107,
    }),
    Some(CoredllOrdinalDef {
        name: "ClearCommError",
        ordinal: 108,
    }),
    Some(CoredllOrdinalDef {
        name: "EscapeCommFunction",
        ordinal: 109,
    }),
    Some(CoredllOrdinalDef {
        name: "GetCommMask",
        ordinal: 110,
    }),
    Some(CoredllOrdinalDef {
        name: "GetCommModemStatus",
        ordinal: 111,
    }),
    Some(CoredllOrdinalDef {
        name: "GetCommProperties",
        ordinal: 112,
    }),
    Some(CoredllOrdinalDef {
        name: "GetCommState",
        ordinal: 113,
    }),
    Some(CoredllOrdinalDef {
        name: "GetCommTimeouts",
        ordinal: 114,
    }),
    Some(CoredllOrdinalDef {
        name: "PurgeComm",
        ordinal: 115,
    }),
    Some(CoredllOrdinalDef {
        name: "SetCommBreak",
        ordinal: 116,
    }),
    Some(CoredllOrdinalDef {
        name: "SetCommMask",
        ordinal: 117,
    }),
    Some(CoredllOrdinalDef {
        name: "SetCommState",
        ordinal: 118,
    }),
    Some(CoredllOrdinalDef {
        name: "SetCommTimeouts",
        ordinal: 119,
    }),
    Some(CoredllOrdinalDef {
        name: "SetupComm",
        ordinal: 120,
    }),
    Some(CoredllOrdinalDef {
        name: "TransmitCommChar",
        ordinal: 121,
    }),
    Some(CoredllOrdinalDef {
        name: "WaitCommEvent",
        ordinal: 122,
    }),
    Some(CoredllOrdinalDef {
        name: "EnumPnpIds",
        ordinal: 123,
    }),
    Some(CoredllOrdinalDef {
        name: "EnumDevices",
        ordinal: 124,
    }),
    Some(CoredllOrdinalDef {
        name: "GetDeviceKeys",
        ordinal: 125,
    }),
    Some(CoredllOrdinalDef {
        name: "OpenDeviceKey",
        ordinal: 1396,
    }),
    Some(CoredllOrdinalDef {
        name: "DDKReg_GetWindowInfo",
        ordinal: 1668,
    }),
    Some(CoredllOrdinalDef {
        name: "DDKReg_GetIsrInfo",
        ordinal: 1669,
    }),
    Some(CoredllOrdinalDef {
        name: "DDKReg_GetPciInfo",
        ordinal: 1670,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptAcquireContextW",
        ordinal: 126,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptReleaseContext",
        ordinal: 127,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptGenKey",
        ordinal: 128,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptDeriveKey",
        ordinal: 129,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptDestroyKey",
        ordinal: 130,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptSetKeyParam",
        ordinal: 131,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptGetKeyParam",
        ordinal: 132,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptExportKey",
        ordinal: 133,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptImportKey",
        ordinal: 134,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptEncrypt",
        ordinal: 135,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptDecrypt",
        ordinal: 136,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptCreateHash",
        ordinal: 137,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptHashSessionKey",
        ordinal: 138,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptHashData",
        ordinal: 139,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptDestroyHash",
        ordinal: 140,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptSignHashW",
        ordinal: 141,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptVerifySignatureW",
        ordinal: 142,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptGenRandom",
        ordinal: 143,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptGetUserKey",
        ordinal: 144,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptSetProviderW",
        ordinal: 145,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptGetHashParam",
        ordinal: 146,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptSetHashParam",
        ordinal: 147,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptGetProvParam",
        ordinal: 148,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptSetProvParam",
        ordinal: 149,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptSetProviderExW",
        ordinal: 150,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptGetDefaultProviderW",
        ordinal: 151,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptEnumProviderTypesW",
        ordinal: 152,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptEnumProvidersW",
        ordinal: 153,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptContextAddRef",
        ordinal: 154,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptDuplicateKey",
        ordinal: 155,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptDuplicateHash",
        ordinal: 156,
    }),
    Some(CoredllOrdinalDef {
        name: "IsEncryptionPermitted",
        ordinal: 613,
    }),
    Some(CoredllOrdinalDef {
        name: "AttachDebugger",
        ordinal: 157,
    }),
    None,
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "SetInterruptEvent",
        ordinal: 158,
    }),
    Some(CoredllOrdinalDef {
        name: "CeSetPowerOnEvent",
        ordinal: 1688,
    }),
    Some(CoredllOrdinalDef {
        name: "IsExiting",
        ordinal: 159,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateDirectoryW",
        ordinal: 160,
    }),
    Some(CoredllOrdinalDef {
        name: "RemoveDirectoryW",
        ordinal: 161,
    }),
    Some(CoredllOrdinalDef {
        name: "GetTempPathW",
        ordinal: 162,
    }),
    Some(CoredllOrdinalDef {
        name: "MoveFileW",
        ordinal: 163,
    }),
    Some(CoredllOrdinalDef {
        name: "CopyFileW",
        ordinal: 164,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "DeleteFileW",
        ordinal: 165,
    }),
    Some(CoredllOrdinalDef {
        name: "GetFileAttributesW",
        ordinal: 166,
    }),
    Some(CoredllOrdinalDef {
        name: "FindFirstFileW",
        ordinal: 167,
    }),
    Some(CoredllOrdinalDef {
        name: "FindFirstFileExW",
        ordinal: 1235,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateFileW",
        ordinal: 168,
    }),
    Some(CoredllOrdinalDef {
        name: "SetFileAttributesW",
        ordinal: 169,
    }),
    Some(CoredllOrdinalDef {
        name: "ReadFile",
        ordinal: 170,
    }),
    Some(CoredllOrdinalDef {
        name: "WriteFile",
        ordinal: 171,
    }),
    Some(CoredllOrdinalDef {
        name: "GetFileSize",
        ordinal: 172,
    }),
    Some(CoredllOrdinalDef {
        name: "SetFilePointer",
        ordinal: 173,
    }),
    Some(CoredllOrdinalDef {
        name: "GetFileInformationByHandle",
        ordinal: 174,
    }),
    Some(CoredllOrdinalDef {
        name: "FlushFileBuffers",
        ordinal: 175,
    }),
    Some(CoredllOrdinalDef {
        name: "GetFileTime",
        ordinal: 176,
    }),
    Some(CoredllOrdinalDef {
        name: "SetFileTime",
        ordinal: 177,
    }),
    Some(CoredllOrdinalDef {
        name: "SetEndOfFile",
        ordinal: 178,
    }),
    Some(CoredllOrdinalDef {
        name: "FindClose",
        ordinal: 180,
    }),
    Some(CoredllOrdinalDef {
        name: "FindNextFileW",
        ordinal: 181,
    }),
    Some(CoredllOrdinalDef {
        name: "DeleteAndRenameFile",
        ordinal: 183,
    }),
    Some(CoredllOrdinalDef {
        name: "GetDiskFreeSpaceExW",
        ordinal: 184,
    }),
    Some(CoredllOrdinalDef {
        name: "GetFileAttributesExW",
        ordinal: 1237,
    }),
    Some(CoredllOrdinalDef {
        name: "GetStoreInformation",
        ordinal: 323,
    }),
    Some(CoredllOrdinalDef {
        name: "PegOidGetInfo",
        ordinal: 301,
    }),
    Some(CoredllOrdinalDef {
        name: "CeOidGetInfo",
        ordinal: 312,
    }),
    Some(CoredllOrdinalDef {
        name: "CeOidGetInfoEx",
        ordinal: 1195,
    }),
    Some(CoredllOrdinalDef {
        name: "CeOidGetInfoEx2",
        ordinal: 1472,
    }),
    Some(CoredllOrdinalDef {
        name: "FindFirstChangeNotificationW",
        ordinal: 1682,
    }),
    Some(CoredllOrdinalDef {
        name: "FindNextChangeNotification",
        ordinal: 1683,
    }),
    Some(CoredllOrdinalDef {
        name: "FindCloseChangeNotification",
        ordinal: 1684,
    }),
    Some(CoredllOrdinalDef {
        name: "CeGetFileNotificationInfo",
        ordinal: 1798,
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "CheckPassword",
        ordinal: 182,
    }),
    Some(CoredllOrdinalDef {
        name: "SetPassword",
        ordinal: 238,
    }),
    Some(CoredllOrdinalDef {
        name: "GetPasswordActive",
        ordinal: 239,
    }),
    Some(CoredllOrdinalDef {
        name: "SetPasswordActive",
        ordinal: 240,
    }),
    Some(CoredllOrdinalDef {
        name: "SetPasswordStatus",
        ordinal: 1537,
    }),
    Some(CoredllOrdinalDef {
        name: "GetPasswordStatus",
        ordinal: 1538,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateMsgQueue",
        ordinal: 1529,
    }),
    Some(CoredllOrdinalDef {
        name: "ReadMsgQueue",
        ordinal: 1530,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "WriteMsgQueue",
        ordinal: 1531,
    }),
    Some(CoredllOrdinalDef {
        name: "GetMsgQueueInfo",
        ordinal: 1532,
    }),
    Some(CoredllOrdinalDef {
        name: "CloseMsgQueue",
        ordinal: 1533,
    }),
    Some(CoredllOrdinalDef {
        name: "OpenMsgQueue",
        ordinal: 1536,
    }),
    Some(CoredllOrdinalDef {
        name: "IsValidCodePage",
        ordinal: 185,
    }),
    Some(CoredllOrdinalDef {
        name: "GetACP",
        ordinal: 186,
    }),
    Some(CoredllOrdinalDef {
        name: "GetOEMCP",
        ordinal: 187,
    }),
    Some(CoredllOrdinalDef {
        name: "GetCPInfo",
        ordinal: 188,
    }),
    Some(CoredllOrdinalDef {
        name: "SetACP",
        ordinal: 189,
    }),
    Some(CoredllOrdinalDef {
        name: "SetOEMCP",
        ordinal: 190,
    }),
    Some(CoredllOrdinalDef {
        name: "IsDBCSLeadByte",
        ordinal: 191,
    }),
    Some(CoredllOrdinalDef {
        name: "IsDBCSLeadByteEx",
        ordinal: 192,
    }),
    Some(CoredllOrdinalDef {
        name: "MultiByteToWideChar",
        ordinal: 196,
    }),
    Some(CoredllOrdinalDef {
        name: "WideCharToMultiByte",
        ordinal: 197,
    }),
    Some(CoredllOrdinalDef {
        name: "CompareStringW",
        ordinal: 198,
    }),
    Some(CoredllOrdinalDef {
        name: "LCMapStringW",
        ordinal: 199,
    }),
    Some(CoredllOrdinalDef {
        name: "GetLocaleInfoW",
        ordinal: 200,
    }),
    Some(CoredllOrdinalDef {
        name: "SetLocaleInfoW",
        ordinal: 201,
    }),
    Some(CoredllOrdinalDef {
        name: "GetTimeFormatW",
        ordinal: 202,
    }),
    Some(CoredllOrdinalDef {
        name: "GetDateFormatW",
        ordinal: 203,
    }),
    Some(CoredllOrdinalDef {
        name: "GetNumberFormatW",
        ordinal: 204,
    }),
    Some(CoredllOrdinalDef {
        name: "GetCurrencyFormatW",
        ordinal: 205,
    }),
    Some(CoredllOrdinalDef {
        name: "EnumCalendarInfoW",
        ordinal: 206,
    }),
    Some(CoredllOrdinalDef {
        name: "EnumTimeFormatsW",
        ordinal: 207,
    }),
    Some(CoredllOrdinalDef {
        name: "EnumDateFormatsW",
        ordinal: 208,
    }),
    Some(CoredllOrdinalDef {
        name: "IsValidLocale",
        ordinal: 209,
    }),
    Some(CoredllOrdinalDef {
        name: "ConvertDefaultLocale",
        ordinal: 210,
    }),
    Some(CoredllOrdinalDef {
        name: "GetSystemDefaultLangID",
        ordinal: 211,
    }),
    Some(CoredllOrdinalDef {
        name: "GetUserDefaultLangID",
        ordinal: 212,
    }),
    Some(CoredllOrdinalDef {
        name: "GetSystemDefaultLCID",
        ordinal: 213,
    }),
    Some(CoredllOrdinalDef {
        name: "SetSystemDefaultLCID",
        ordinal: 214,
    }),
    Some(CoredllOrdinalDef {
        name: "GetUserDefaultLCID",
        ordinal: 215,
    }),
    Some(CoredllOrdinalDef {
        name: "SetUserDefaultLCID",
        ordinal: 1795,
    }),
    Some(CoredllOrdinalDef {
        name: "GetStringTypeW",
        ordinal: 216,
    }),
    Some(CoredllOrdinalDef {
        name: "GetStringTypeExW",
        ordinal: 217,
    }),
    Some(CoredllOrdinalDef {
        name: "FoldStringW",
        ordinal: 218,
    }),
    Some(CoredllOrdinalDef {
        name: "EnumSystemLocalesW",
        ordinal: 219,
    }),
    Some(CoredllOrdinalDef {
        name: "EnumSystemCodePagesW",
        ordinal: 220,
    }),
    Some(CoredllOrdinalDef {
        name: "CharLowerW",
        ordinal: 221,
    }),
    Some(CoredllOrdinalDef {
        name: "CharLowerBuffW",
        ordinal: 222,
    }),
    Some(CoredllOrdinalDef {
        name: "CharUpperBuffW",
        ordinal: 223,
    }),
    Some(CoredllOrdinalDef {
        name: "CharUpperW",
        ordinal: 224,
    }),
    Some(CoredllOrdinalDef {
        name: "CharPrevW",
        ordinal: 225,
    }),
    Some(CoredllOrdinalDef {
        name: "CharNextW",
        ordinal: 226,
    }),
    Some(CoredllOrdinalDef {
        name: "lstrcmpW",
        ordinal: 227,
    }),
    Some(CoredllOrdinalDef {
        name: "lstrcmpiW",
        ordinal: 228,
    }),
    Some(CoredllOrdinalDef {
        name: "DBCanonicalize",
        ordinal: 233,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "IsDBCSLeadByte",
        ordinal: 191,
    }),
    Some(CoredllOrdinalDef {
        name: "MultiByteToWideChar",
        ordinal: 196,
    }),
    Some(CoredllOrdinalDef {
        name: "WideCharToMultiByte",
        ordinal: 197,
    }),
    Some(CoredllOrdinalDef {
        name: "CompareStringW",
        ordinal: 198,
    }),
    Some(CoredllOrdinalDef {
        name: "GetSystemDefaultLCID",
        ordinal: 213,
    }),
    Some(CoredllOrdinalDef {
        name: "GetStringTypeExW",
        ordinal: 217,
    }),
    Some(CoredllOrdinalDef {
        name: "CharLowerW",
        ordinal: 221,
    }),
    Some(CoredllOrdinalDef {
        name: "CharUpperW",
        ordinal: 224,
    }),
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "FormatMessageW",
        ordinal: 234,
    }),
    Some(CoredllOrdinalDef {
        name: "RegisterDevice",
        ordinal: 235,
    }),
    Some(CoredllOrdinalDef {
        name: "DeregisterDevice",
        ordinal: 236,
    }),
    Some(CoredllOrdinalDef {
        name: "LoadFSD",
        ordinal: 237,
    }),
    Some(CoredllOrdinalDef {
        name: "LoadFSDEx",
        ordinal: 1421,
    }),
    Some(CoredllOrdinalDef {
        name: "ActivateDevice",
        ordinal: 1179,
    }),
    Some(CoredllOrdinalDef {
        name: "ActivateDeviceEx",
        ordinal: 1494,
    }),
    Some(CoredllOrdinalDef {
        name: "DeactivateDevice",
        ordinal: 1180,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "RegOpenProcessKey",
        ordinal: 1542,
    }),
    Some(CoredllOrdinalDef {
        name: "CeResyncFilesys",
        ordinal: 1425,
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "ResourceCreateList",
        ordinal: 1612,
    }),
    Some(CoredllOrdinalDef {
        name: "ResourceRequest",
        ordinal: 1613,
    }),
    Some(CoredllOrdinalDef {
        name: "ResourceRelease",
        ordinal: 1614,
    }),
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "GetSystemPowerState",
        ordinal: 1581,
    }),
    Some(CoredllOrdinalDef {
        name: "SetSystemPowerState",
        ordinal: 1582,
    }),
    Some(CoredllOrdinalDef {
        name: "SetPowerRequirement",
        ordinal: 1583,
    }),
    Some(CoredllOrdinalDef {
        name: "ReleasePowerRequirement",
        ordinal: 1584,
    }),
    Some(CoredllOrdinalDef {
        name: "RequestPowerNotifications",
        ordinal: 1585,
    }),
    Some(CoredllOrdinalDef {
        name: "StopPowerNotifications",
        ordinal: 1586,
    }),
    Some(CoredllOrdinalDef {
        name: "DevicePowerNotify",
        ordinal: 1588,
    }),
    Some(CoredllOrdinalDef {
        name: "RegisterPowerRelationship",
        ordinal: 1609,
    }),
    Some(CoredllOrdinalDef {
        name: "ReleasePowerRelationship",
        ordinal: 1610,
    }),
    Some(CoredllOrdinalDef {
        name: "SetDevicePower",
        ordinal: 1678,
    }),
    Some(CoredllOrdinalDef {
        name: "GetDevicePower",
        ordinal: 1679,
    }),
    Some(CoredllOrdinalDef {
        name: "PowerPolicyNotify",
        ordinal: 1764,
    }),
    Some(CoredllOrdinalDef {
        name: "ActivateService",
        ordinal: 1508,
    }),
    Some(CoredllOrdinalDef {
        name: "RegisterService",
        ordinal: 1509,
    }),
    Some(CoredllOrdinalDef {
        name: "DeregisterService",
        ordinal: 1510,
    }),
    Some(CoredllOrdinalDef {
        name: "CloseAllServiceHandles",
        ordinal: 1511,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateServiceHandle",
        ordinal: 1512,
    }),
    Some(CoredllOrdinalDef {
        name: "GetServiceByIndex",
        ordinal: 1513,
    }),
    Some(CoredllOrdinalDef {
        name: "ServiceIoControl",
        ordinal: 1514,
    }),
    Some(CoredllOrdinalDef {
        name: "ServiceAddPort",
        ordinal: 1515,
    }),
    Some(CoredllOrdinalDef {
        name: "ServiceUnbindPorts",
        ordinal: 1516,
    }),
    Some(CoredllOrdinalDef {
        name: "EnumServices",
        ordinal: 1517,
    }),
    Some(CoredllOrdinalDef {
        name: "GetServiceHandle",
        ordinal: 1518,
    }),
    Some(CoredllOrdinalDef {
        name: "ServiceClosePort",
        ordinal: 1759,
    }),
    Some(CoredllOrdinalDef {
        name: "SignalStarted",
        ordinal: 639,
    }),
    Some(CoredllOrdinalDef {
        name: "CeRegisterFileSystemNotification",
        ordinal: 331,
    }),
    Some(CoredllOrdinalDef {
        name: "RegisterAFSEx",
        ordinal: 1490,
    }),
    Some(CoredllOrdinalDef {
        name: "DeregisterAFS",
        ordinal: 335,
    }),
    Some(CoredllOrdinalDef {
        name: "RegisterAFSName",
        ordinal: 338,
    }),
    Some(CoredllOrdinalDef {
        name: "DeregisterAFSName",
        ordinal: 339,
    }),
    Some(CoredllOrdinalDef {
        name: "GetSystemMemoryDivision",
        ordinal: 336,
    }),
    Some(CoredllOrdinalDef {
        name: "SetSystemMemoryDivision",
        ordinal: 337,
    }),
    Some(CoredllOrdinalDef {
        name: "DumpFileSystemHeap",
        ordinal: 341,
    }),
    Some(CoredllOrdinalDef {
        name: "FileSystemPowerFunction",
        ordinal: 241,
    }),
    Some(CoredllOrdinalDef {
        name: "CloseAllFileHandles",
        ordinal: 242,
    }),
    Some(CoredllOrdinalDef {
        name: "ReadFileWithSeek",
        ordinal: 243,
    }),
    Some(CoredllOrdinalDef {
        name: "WriteFileWithSeek",
        ordinal: 718,
    }),
    Some(CoredllOrdinalDef {
        name: "IsSystemFile",
        ordinal: 1680,
    }),
    Some(CoredllOrdinalDef {
        name: "RequestDeviceNotifications",
        ordinal: 1504,
    }),
    Some(CoredllOrdinalDef {
        name: "StopDeviceNotifications",
        ordinal: 1505,
    }),
    Some(CoredllOrdinalDef {
        name: "AdvertiseInterface",
        ordinal: 1687,
    }),
    Some(CoredllOrdinalDef {
        name: "GetDeviceByIndex",
        ordinal: 1236,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "CreateWindowExW",
        ordinal: 246,
    }),
    Some(CoredllOrdinalDef {
        name: "SetWindowPos",
        ordinal: 247,
    }),
    Some(CoredllOrdinalDef {
        name: "GetWindowRect",
        ordinal: 248,
    }),
    Some(CoredllOrdinalDef {
        name: "GetClientRect",
        ordinal: 249,
    }),
    Some(CoredllOrdinalDef {
        name: "InvalidateRect",
        ordinal: 250,
    }),
    Some(CoredllOrdinalDef {
        name: "GetWindow",
        ordinal: 251,
    }),
    Some(CoredllOrdinalDef {
        name: "WindowFromPoint",
        ordinal: 252,
    }),
    Some(CoredllOrdinalDef {
        name: "ChildWindowFromPoint",
        ordinal: 253,
    }),
    Some(CoredllOrdinalDef {
        name: "ClientToScreen",
        ordinal: 254,
    }),
    Some(CoredllOrdinalDef {
        name: "ScreenToClient",
        ordinal: 255,
    }),
    Some(CoredllOrdinalDef {
        name: "SetWindowTextW",
        ordinal: 256,
    }),
    Some(CoredllOrdinalDef {
        name: "GetWindowTextW",
        ordinal: 257,
    }),
    Some(CoredllOrdinalDef {
        name: "SetWindowLongW",
        ordinal: 258,
    }),
    Some(CoredllOrdinalDef {
        name: "GetWindowLongW",
        ordinal: 259,
    }),
    Some(CoredllOrdinalDef {
        name: "BeginPaint",
        ordinal: 260,
    }),
    Some(CoredllOrdinalDef {
        name: "EndPaint",
        ordinal: 261,
    }),
    Some(CoredllOrdinalDef {
        name: "GetDCEx",
        ordinal: 1185,
    }),
    Some(CoredllOrdinalDef {
        name: "DefWindowProcW",
        ordinal: 264,
    }),
    Some(CoredllOrdinalDef {
        name: "DestroyWindow",
        ordinal: 265,
    }),
    Some(CoredllOrdinalDef {
        name: "ShowWindow",
        ordinal: 266,
    }),
    Some(CoredllOrdinalDef {
        name: "UpdateWindow",
        ordinal: 267,
    }),
    Some(CoredllOrdinalDef {
        name: "SetParent",
        ordinal: 268,
    }),
    Some(CoredllOrdinalDef {
        name: "GetParent",
        ordinal: 269,
    }),
    Some(CoredllOrdinalDef {
        name: "IsWindow",
        ordinal: 271,
    }),
    Some(CoredllOrdinalDef {
        name: "MoveWindow",
        ordinal: 272,
    }),
    Some(CoredllOrdinalDef {
        name: "GetUpdateRgn",
        ordinal: 273,
    }),
    Some(CoredllOrdinalDef {
        name: "GetUpdateRect",
        ordinal: 274,
    }),
    Some(CoredllOrdinalDef {
        name: "BringWindowToTop",
        ordinal: 275,
    }),
    Some(CoredllOrdinalDef {
        name: "GetWindowTextLengthW",
        ordinal: 276,
    }),
    Some(CoredllOrdinalDef {
        name: "IsChild",
        ordinal: 277,
    }),
    Some(CoredllOrdinalDef {
        name: "ValidateRect",
        ordinal: 278,
    }),
    Some(CoredllOrdinalDef {
        name: "GetClassNameW",
        ordinal: 283,
    }),
    Some(CoredllOrdinalDef {
        name: "MapWindowPoints",
        ordinal: 284,
    }),
    Some(CoredllOrdinalDef {
        name: "CallWindowProcW",
        ordinal: 285,
    }),
    Some(CoredllOrdinalDef {
        name: "FindWindowW",
        ordinal: 286,
    }),
    Some(CoredllOrdinalDef {
        name: "EnableWindow",
        ordinal: 287,
    }),
    Some(CoredllOrdinalDef {
        name: "IsWindowEnabled",
        ordinal: 288,
    }),
    Some(CoredllOrdinalDef {
        name: "ScrollWindowEx",
        ordinal: 289,
    }),
    Some(CoredllOrdinalDef {
        name: "PostThreadMessageW",
        ordinal: 290,
    }),
    Some(CoredllOrdinalDef {
        name: "EnumWindows",
        ordinal: 291,
    }),
    Some(CoredllOrdinalDef {
        name: "GetWindowThreadProcessId",
        ordinal: 292,
    }),
    Some(CoredllOrdinalDef {
        name: "BeginDeferWindowPos",
        ordinal: 1157,
    }),
    Some(CoredllOrdinalDef {
        name: "DeferWindowPos",
        ordinal: 1158,
    }),
    Some(CoredllOrdinalDef {
        name: "EndDeferWindowPos",
        ordinal: 1159,
    }),
    Some(CoredllOrdinalDef {
        name: "GetDesktopWindow",
        ordinal: 1397,
    }),
    Some(CoredllOrdinalDef {
        name: "SetWindowRgn",
        ordinal: 1398,
    }),
    Some(CoredllOrdinalDef {
        name: "GetWindowRgn",
        ordinal: 1399,
    }),
    Some(CoredllOrdinalDef {
        name: "GetWindowTextWDirect",
        ordinal: 1454,
    }),
    Some(CoredllOrdinalDef {
        name: "AccessibilitySoundSentryEvent",
        ordinal: 1540,
    }),
    Some(CoredllOrdinalDef {
        name: "ChangeDisplaySettingsEx",
        ordinal: 1611,
    }),
    Some(CoredllOrdinalDef {
        name: "InvalidateRgn",
        ordinal: 1615,
    }),
    Some(CoredllOrdinalDef {
        name: "ValidateRgn",
        ordinal: 1616,
    }),
    Some(CoredllOrdinalDef {
        name: "RedrawWindow",
        ordinal: 1672,
    }),
    Some(CoredllOrdinalDef {
        name: "RegisterSIPanel",
        ordinal: 293,
    }),
    Some(CoredllOrdinalDef {
        name: "RectangleAnimation",
        ordinal: 294,
    }),
    Some(CoredllOrdinalDef {
        name: "GwesPowerOffSystem",
        ordinal: 296,
    }),
    Some(CoredllOrdinalDef {
        name: "SetAssociatedMenu",
        ordinal: 299,
    }),
    Some(CoredllOrdinalDef {
        name: "GetAssociatedMenu",
        ordinal: 300,
    }),
    Some(CoredllOrdinalDef {
        name: "GwesPowerDown",
        ordinal: 1722,
    }),
    Some(CoredllOrdinalDef {
        name: "GwesPowerUp",
        ordinal: 1723,
    }),
    Some(CoredllOrdinalDef {
        name: "ShowStartupWindow",
        ordinal: 1810,
    }),
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "PegFindFirstDatabase",
        ordinal: 302,
    }),
    Some(CoredllOrdinalDef {
        name: "PegFindNextDatabase",
        ordinal: 303,
    }),
    Some(CoredllOrdinalDef {
        name: "PegCreateDatabase",
        ordinal: 304,
    }),
    Some(CoredllOrdinalDef {
        name: "PegSetDatabaseInfo",
        ordinal: 305,
    }),
    Some(CoredllOrdinalDef {
        name: "PegOpenDatabase",
        ordinal: 306,
    }),
    Some(CoredllOrdinalDef {
        name: "PegDeleteDatabase",
        ordinal: 307,
    }),
    Some(CoredllOrdinalDef {
        name: "PegSeekDatabase",
        ordinal: 308,
    }),
    Some(CoredllOrdinalDef {
        name: "PegDeleteRecord",
        ordinal: 309,
    }),
    Some(CoredllOrdinalDef {
        name: "PegReadRecordProps",
        ordinal: 310,
    }),
    Some(CoredllOrdinalDef {
        name: "PegWriteRecordProps",
        ordinal: 311,
    }),
    Some(CoredllOrdinalDef {
        name: "CeFindFirstDatabase",
        ordinal: 313,
    }),
    Some(CoredllOrdinalDef {
        name: "CeFindNextDatabase",
        ordinal: 314,
    }),
    Some(CoredllOrdinalDef {
        name: "CeCreateDatabase",
        ordinal: 315,
    }),
    Some(CoredllOrdinalDef {
        name: "CeCreateDatabaseEx",
        ordinal: 1190,
    }),
    Some(CoredllOrdinalDef {
        name: "CeSetDatabaseInfo",
        ordinal: 316,
    }),
    Some(CoredllOrdinalDef {
        name: "CeSetDatabaseInfoEx",
        ordinal: 1191,
    }),
    Some(CoredllOrdinalDef {
        name: "CeOpenDatabase",
        ordinal: 317,
    }),
    Some(CoredllOrdinalDef {
        name: "CeOpenDatabaseEx",
        ordinal: 1192,
    }),
    Some(CoredllOrdinalDef {
        name: "CeDeleteDatabase",
        ordinal: 318,
    }),
    Some(CoredllOrdinalDef {
        name: "CeReadRecordProps",
        ordinal: 321,
    }),
    Some(CoredllOrdinalDef {
        name: "CeSeekDatabase",
        ordinal: 319,
    }),
    Some(CoredllOrdinalDef {
        name: "CeFindFirstDatabaseEx",
        ordinal: 1196,
    }),
    Some(CoredllOrdinalDef {
        name: "CeFindNextDatabaseEx",
        ordinal: 1189,
    }),
    Some(CoredllOrdinalDef {
        name: "CeCreateDatabaseEx2",
        ordinal: 1468,
    }),
    Some(CoredllOrdinalDef {
        name: "CeSetDatabaseInfoEx2",
        ordinal: 1471,
    }),
    Some(CoredllOrdinalDef {
        name: "CeOpenDatabaseEx2",
        ordinal: 1469,
    }),
    Some(CoredllOrdinalDef {
        name: "CeDeleteDatabaseEx",
        ordinal: 1193,
    }),
    Some(CoredllOrdinalDef {
        name: "CeSeekDatabaseEx",
        ordinal: 1470,
    }),
    Some(CoredllOrdinalDef {
        name: "CeDeleteRecord",
        ordinal: 320,
    }),
    Some(CoredllOrdinalDef {
        name: "CeReadRecordPropsEx",
        ordinal: 1194,
    }),
    Some(CoredllOrdinalDef {
        name: "CeMountDBVol",
        ordinal: 1164,
    }),
    Some(CoredllOrdinalDef {
        name: "CeEnumDBVolumes",
        ordinal: 1165,
    }),
    Some(CoredllOrdinalDef {
        name: "CeWriteRecordProps",
        ordinal: 322,
    }),
    Some(CoredllOrdinalDef {
        name: "CeUnmountDBVol",
        ordinal: 1197,
    }),
    Some(CoredllOrdinalDef {
        name: "CeFlushDBVol",
        ordinal: 1217,
    }),
    Some(CoredllOrdinalDef {
        name: "CeGetDBInformationByHandle",
        ordinal: 1473,
    }),
    Some(CoredllOrdinalDef {
        name: "CeFreeNotification",
        ordinal: 1226,
    }),
    Some(CoredllOrdinalDef {
        name: "CeChangeDatabaseLCID",
        ordinal: 340,
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "AudioUpdateFromRegistry",
        ordinal: 376,
    }),
    Some(CoredllOrdinalDef {
        name: "sndPlaySoundW",
        ordinal: 377,
    }),
    Some(CoredllOrdinalDef {
        name: "PlaySoundW",
        ordinal: 378,
    }),
    Some(CoredllOrdinalDef {
        name: "waveOutGetNumDevs",
        ordinal: 379,
    }),
    Some(CoredllOrdinalDef {
        name: "waveOutGetDevCaps",
        ordinal: 380,
    }),
    Some(CoredllOrdinalDef {
        name: "waveOutGetVolume",
        ordinal: 381,
    }),
    Some(CoredllOrdinalDef {
        name: "waveOutSetVolume",
        ordinal: 382,
    }),
    Some(CoredllOrdinalDef {
        name: "waveOutGetErrorText",
        ordinal: 383,
    }),
    Some(CoredllOrdinalDef {
        name: "waveOutClose",
        ordinal: 384,
    }),
    Some(CoredllOrdinalDef {
        name: "waveOutPrepareHeader",
        ordinal: 385,
    }),
    Some(CoredllOrdinalDef {
        name: "waveOutUnprepareHeader",
        ordinal: 386,
    }),
    Some(CoredllOrdinalDef {
        name: "waveOutWrite",
        ordinal: 387,
    }),
    Some(CoredllOrdinalDef {
        name: "waveOutPause",
        ordinal: 388,
    }),
    Some(CoredllOrdinalDef {
        name: "waveOutRestart",
        ordinal: 389,
    }),
    Some(CoredllOrdinalDef {
        name: "waveOutReset",
        ordinal: 390,
    }),
    Some(CoredllOrdinalDef {
        name: "waveOutBreakLoop",
        ordinal: 391,
    }),
    Some(CoredllOrdinalDef {
        name: "waveOutGetPosition",
        ordinal: 392,
    }),
    Some(CoredllOrdinalDef {
        name: "waveOutGetPitch",
        ordinal: 393,
    }),
    Some(CoredllOrdinalDef {
        name: "waveOutSetPitch",
        ordinal: 394,
    }),
    Some(CoredllOrdinalDef {
        name: "waveOutGetPlaybackRate",
        ordinal: 395,
    }),
    Some(CoredllOrdinalDef {
        name: "waveOutSetPlaybackRate",
        ordinal: 396,
    }),
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "waveOutGetID",
        ordinal: 397,
    }),
    Some(CoredllOrdinalDef {
        name: "waveOutMessage",
        ordinal: 398,
    }),
    Some(CoredllOrdinalDef {
        name: "waveOutOpen",
        ordinal: 399,
    }),
    Some(CoredllOrdinalDef {
        name: "waveInGetNumDevs",
        ordinal: 400,
    }),
    Some(CoredllOrdinalDef {
        name: "waveInGetDevCaps",
        ordinal: 401,
    }),
    Some(CoredllOrdinalDef {
        name: "waveInGetErrorText",
        ordinal: 402,
    }),
    Some(CoredllOrdinalDef {
        name: "waveInClose",
        ordinal: 403,
    }),
    Some(CoredllOrdinalDef {
        name: "waveInPrepareHeader",
        ordinal: 404,
    }),
    Some(CoredllOrdinalDef {
        name: "waveInUnprepareHeader",
        ordinal: 405,
    }),
    Some(CoredllOrdinalDef {
        name: "waveInAddBuffer",
        ordinal: 406,
    }),
    Some(CoredllOrdinalDef {
        name: "waveInStart",
        ordinal: 407,
    }),
    Some(CoredllOrdinalDef {
        name: "waveInStop",
        ordinal: 408,
    }),
    Some(CoredllOrdinalDef {
        name: "waveInReset",
        ordinal: 409,
    }),
    Some(CoredllOrdinalDef {
        name: "waveInGetPosition",
        ordinal: 410,
    }),
    Some(CoredllOrdinalDef {
        name: "waveInGetID",
        ordinal: 411,
    }),
    Some(CoredllOrdinalDef {
        name: "waveInMessage",
        ordinal: 412,
    }),
    Some(CoredllOrdinalDef {
        name: "waveInOpen",
        ordinal: 413,
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "mixerGetControlDetails",
        ordinal: 1589,
    }),
    Some(CoredllOrdinalDef {
        name: "mixerGetDevCaps",
        ordinal: 1591,
    }),
    Some(CoredllOrdinalDef {
        name: "mixerGetID",
        ordinal: 1590,
    }),
    Some(CoredllOrdinalDef {
        name: "mixerGetLineControls",
        ordinal: 1592,
    }),
    Some(CoredllOrdinalDef {
        name: "mixerGetLineInfo",
        ordinal: 1593,
    }),
    Some(CoredllOrdinalDef {
        name: "mixerGetNumDevs",
        ordinal: 1594,
    }),
    Some(CoredllOrdinalDef {
        name: "mixerMessage",
        ordinal: 1596,
    }),
    Some(CoredllOrdinalDef {
        name: "mixerOpen",
        ordinal: 1595,
    }),
    Some(CoredllOrdinalDef {
        name: "mixerSetControlDetails",
        ordinal: 1597,
    }),
    Some(CoredllOrdinalDef {
        name: "mixerClose",
        ordinal: 1598,
    }),
    Some(CoredllOrdinalDef {
        name: "WNetAddConnection3W",
        ordinal: 444,
    }),
    Some(CoredllOrdinalDef {
        name: "WNetCancelConnection2W",
        ordinal: 445,
    }),
    Some(CoredllOrdinalDef {
        name: "WNetConnectionDialog1W",
        ordinal: 446,
    }),
    Some(CoredllOrdinalDef {
        name: "WNetDisconnectDialog",
        ordinal: 447,
    }),
    Some(CoredllOrdinalDef {
        name: "WNetDisconnectDialog1W",
        ordinal: 448,
    }),
    Some(CoredllOrdinalDef {
        name: "WNetGetConnectionW",
        ordinal: 449,
    }),
    Some(CoredllOrdinalDef {
        name: "WNetGetUniversalNameW",
        ordinal: 450,
    }),
    Some(CoredllOrdinalDef {
        name: "WNetGetUserW",
        ordinal: 451,
    }),
    Some(CoredllOrdinalDef {
        name: "WNetOpenEnumW",
        ordinal: 452,
    }),
    Some(CoredllOrdinalDef {
        name: "WNetCloseEnum",
        ordinal: 453,
    }),
    Some(CoredllOrdinalDef {
        name: "WNetEnumResourceW",
        ordinal: 454,
    }),
    Some(CoredllOrdinalDef {
        name: "GetUserNameExW",
        ordinal: 1503,
    }),
    Some(CoredllOrdinalDef {
        name: "RegCloseKey",
        ordinal: 455,
    }),
    Some(CoredllOrdinalDef {
        name: "RegCreateKeyExW",
        ordinal: 456,
    }),
    Some(CoredllOrdinalDef {
        name: "RegDeleteKeyW",
        ordinal: 457,
    }),
    Some(CoredllOrdinalDef {
        name: "RegDeleteValueW",
        ordinal: 458,
    }),
    Some(CoredllOrdinalDef {
        name: "RegEnumValueW",
        ordinal: 459,
    }),
    Some(CoredllOrdinalDef {
        name: "RegEnumKeyExW",
        ordinal: 460,
    }),
    Some(CoredllOrdinalDef {
        name: "RegOpenKeyExW",
        ordinal: 461,
    }),
    Some(CoredllOrdinalDef {
        name: "RegQueryInfoKeyW",
        ordinal: 462,
    }),
    Some(CoredllOrdinalDef {
        name: "RegQueryValueExW",
        ordinal: 463,
    }),
    Some(CoredllOrdinalDef {
        name: "RegSetValueExW",
        ordinal: 464,
    }),
    Some(CoredllOrdinalDef {
        name: "RegFlushKey",
        ordinal: 1152,
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "RegCopyFile",
        ordinal: 465,
    }),
    Some(CoredllOrdinalDef {
        name: "RegRestoreFile",
        ordinal: 466,
    }),
    Some(CoredllOrdinalDef {
        name: "RegSaveKey",
        ordinal: 1478,
    }),
    Some(CoredllOrdinalDef {
        name: "RegReplaceKey",
        ordinal: 1479,
    }),
    Some(CoredllOrdinalDef {
        name: "SetCurrentUser",
        ordinal: 1501,
    }),
    Some(CoredllOrdinalDef {
        name: "SetUserData",
        ordinal: 1502,
    }),
    Some(CoredllOrdinalDef {
        name: "GetUserDirectory",
        ordinal: 1686,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptProtectData",
        ordinal: 1599,
    }),
    Some(CoredllOrdinalDef {
        name: "CryptUnprotectData",
        ordinal: 1600,
    }),
    Some(CoredllOrdinalDef {
        name: "CeGenRandom",
        ordinal: 1601,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "PegSetUserNotification",
        ordinal: 467,
    }),
    Some(CoredllOrdinalDef {
        name: "PegClearUserNotification",
        ordinal: 468,
    }),
    Some(CoredllOrdinalDef {
        name: "PegRunAppAtTime",
        ordinal: 469,
    }),
    Some(CoredllOrdinalDef {
        name: "PegRunAppAtEvent",
        ordinal: 470,
    }),
    Some(CoredllOrdinalDef {
        name: "PegHandleAppNotifications",
        ordinal: 471,
    }),
    Some(CoredllOrdinalDef {
        name: "PegGetUserNotificationPreferences",
        ordinal: 472,
    }),
    Some(CoredllOrdinalDef {
        name: "CeSetUserNotification",
        ordinal: 473,
    }),
    Some(CoredllOrdinalDef {
        name: "CeClearUserNotification",
        ordinal: 474,
    }),
    Some(CoredllOrdinalDef {
        name: "CeRunAppAtTime",
        ordinal: 475,
    }),
    Some(CoredllOrdinalDef {
        name: "CeRunAppAtEvent",
        ordinal: 476,
    }),
    Some(CoredllOrdinalDef {
        name: "CeHandleAppNotifications",
        ordinal: 477,
    }),
    Some(CoredllOrdinalDef {
        name: "CeGetUserNotificationPreferences",
        ordinal: 478,
    }),
    Some(CoredllOrdinalDef {
        name: "CeSetUserNotificationEx",
        ordinal: 1352,
    }),
    Some(CoredllOrdinalDef {
        name: "CeGetUserNotificationHandles",
        ordinal: 1353,
    }),
    Some(CoredllOrdinalDef {
        name: "CeGetUserNotification",
        ordinal: 1354,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "Shell_NotifyIcon",
        ordinal: 481,
    }),
    Some(CoredllOrdinalDef {
        name: "SHAddToRecentDocs",
        ordinal: 483,
    }),
    Some(CoredllOrdinalDef {
        name: "SHCreateExplorerInstance",
        ordinal: 1163,
    }),
    Some(CoredllOrdinalDef {
        name: "SHDoneButtonI",
        ordinal: 1782,
    }),
    Some(CoredllOrdinalDef {
        name: "SHGetAppKeyAssocI",
        ordinal: 1783,
    }),
    Some(CoredllOrdinalDef {
        name: "SHSetAppKeyWndAssocI",
        ordinal: 1784,
    }),
    Some(CoredllOrdinalDef {
        name: "SHSetNavBarTextI",
        ordinal: 1785,
    }),
    Some(CoredllOrdinalDef {
        name: "SHSipPreferenceI",
        ordinal: 1786,
    }),
    Some(CoredllOrdinalDef {
        name: "NotSystemParametersInfoI",
        ordinal: 1787,
    }),
    Some(CoredllOrdinalDef {
        name: "SHCloseAppsI",
        ordinal: 1788,
    }),
    Some(CoredllOrdinalDef {
        name: "SHNotificationAddI",
        ordinal: 1806,
    }),
    Some(CoredllOrdinalDef {
        name: "SHNotificationUpdateI",
        ordinal: 1807,
    }),
    Some(CoredllOrdinalDef {
        name: "SHNotificationRemoveI",
        ordinal: 1808,
    }),
    Some(CoredllOrdinalDef {
        name: "SHNotificationGetDataI",
        ordinal: 1809,
    }),
    Some(CoredllOrdinalDef {
        name: "ShellExecuteEx",
        ordinal: 480,
    }),
    Some(CoredllOrdinalDef {
        name: "SHCreateShortcut",
        ordinal: 484,
    }),
    Some(CoredllOrdinalDef {
        name: "SHGetShortcutTarget",
        ordinal: 485,
    }),
    Some(CoredllOrdinalDef {
        name: "SHCreateShortcutEx",
        ordinal: 1488,
    }),
    Some(CoredllOrdinalDef {
        name: "SHShowOutOfMemory",
        ordinal: 486,
    }),
    Some(CoredllOrdinalDef {
        name: "SHLoadDIBitmap",
        ordinal: 487,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "SHGetFileInfo",
        ordinal: 482,
    }),
    Some(CoredllOrdinalDef {
        name: "SHGetSpecialFolderPath",
        ordinal: 295,
    }),
    Some(CoredllOrdinalDef {
        name: "GetOpenFileNameW",
        ordinal: 488,
    }),
    Some(CoredllOrdinalDef {
        name: "GetSaveFileNameW",
        ordinal: 489,
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "PerformCallBack4",
        ordinal: 1448,
    }),
    Some(CoredllOrdinalDef {
        name: "QueryAPISetID",
        ordinal: 490,
    }),
    Some(CoredllOrdinalDef {
        name: "TerminateThread",
        ordinal: 491,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateThread",
        ordinal: 492,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateProcessW",
        ordinal: 493,
    }),
    Some(CoredllOrdinalDef {
        name: "EventModify",
        ordinal: 494,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateEventW",
        ordinal: 495,
    }),
    Some(CoredllOrdinalDef {
        name: "OpenEventW",
        ordinal: 1496,
    }),
    Some(CoredllOrdinalDef {
        name: "GetEventData",
        ordinal: 1527,
    }),
    Some(CoredllOrdinalDef {
        name: "SetEventData",
        ordinal: 1528,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "Sleep",
        ordinal: 496,
    }),
    Some(CoredllOrdinalDef {
        name: "WaitForSingleObject",
        ordinal: 497,
    }),
    Some(CoredllOrdinalDef {
        name: "WaitForMultipleObjects",
        ordinal: 498,
    }),
    Some(CoredllOrdinalDef {
        name: "SuspendThread",
        ordinal: 499,
    }),
    Some(CoredllOrdinalDef {
        name: "ResumeThread",
        ordinal: 500,
    }),
    Some(CoredllOrdinalDef {
        name: "GetThreadContext",
        ordinal: 1148,
    }),
    Some(CoredllOrdinalDef {
        name: "SetThreadContext",
        ordinal: 502,
    }),
    Some(CoredllOrdinalDef {
        name: "WaitForDebugEvent",
        ordinal: 503,
    }),
    Some(CoredllOrdinalDef {
        name: "ContinueDebugEvent",
        ordinal: 504,
    }),
    Some(CoredllOrdinalDef {
        name: "DebugActiveProcess",
        ordinal: 505,
    }),
    None,
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "ReadProcessMemory",
        ordinal: 506,
    }),
    Some(CoredllOrdinalDef {
        name: "WriteProcessMemory",
        ordinal: 507,
    }),
    Some(CoredllOrdinalDef {
        name: "FlushInstructionCache",
        ordinal: 508,
    }),
    Some(CoredllOrdinalDef {
        name: "CeSetProcessVersion",
        ordinal: 1775,
    }),
    Some(CoredllOrdinalDef {
        name: "OpenProcess",
        ordinal: 509,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "GetModuleInformation",
        ordinal: 1721,
    }),
    Some(CoredllOrdinalDef {
        name: "DumpKCallProfile",
        ordinal: 510,
    }),
    Some(CoredllOrdinalDef {
        name: "THCreateSnapshot",
        ordinal: 511,
    }),
    Some(CoredllOrdinalDef {
        name: "NotifyForceCleanboot",
        ordinal: 513,
    }),
    Some(CoredllOrdinalDef {
        name: "SetThreadPriority",
        ordinal: 514,
    }),
    Some(CoredllOrdinalDef {
        name: "GetThreadPriority",
        ordinal: 515,
    }),
    Some(CoredllOrdinalDef {
        name: "CeSetThreadPriority",
        ordinal: 621,
    }),
    Some(CoredllOrdinalDef {
        name: "CeGetThreadPriority",
        ordinal: 622,
    }),
    Some(CoredllOrdinalDef {
        name: "CeSetThreadQuantum",
        ordinal: 1244,
    }),
    Some(CoredllOrdinalDef {
        name: "CeGetThreadQuantum",
        ordinal: 1245,
    }),
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "GetLastError",
        ordinal: 516,
    }),
    Some(CoredllOrdinalDef {
        name: "SetLastError",
        ordinal: 517,
    }),
    Some(CoredllOrdinalDef {
        name: "GetExitCodeThread",
        ordinal: 518,
    }),
    Some(CoredllOrdinalDef {
        name: "GetExitCodeProcess",
        ordinal: 519,
    }),
    Some(CoredllOrdinalDef {
        name: "TlsCall",
        ordinal: 520,
    }),
    Some(CoredllOrdinalDef {
        name: "IsBadCodePtr",
        ordinal: 521,
    }),
    Some(CoredllOrdinalDef {
        name: "IsBadReadPtr",
        ordinal: 522,
    }),
    Some(CoredllOrdinalDef {
        name: "IsBadWritePtr",
        ordinal: 523,
    }),
    Some(CoredllOrdinalDef {
        name: "VirtualAlloc",
        ordinal: 524,
    }),
    Some(CoredllOrdinalDef {
        name: "VirtualFree",
        ordinal: 525,
    }),
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "VirtualProtect",
        ordinal: 526,
    }),
    Some(CoredllOrdinalDef {
        name: "VirtualQuery",
        ordinal: 527,
    }),
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "LoadLibraryW",
        ordinal: 528,
    }),
    Some(CoredllOrdinalDef {
        name: "LoadLibraryExW",
        ordinal: 1241,
    }),
    Some(CoredllOrdinalDef {
        name: "FreeLibrary",
        ordinal: 529,
    }),
    Some(CoredllOrdinalDef {
        name: "GetProcAddressW",
        ordinal: 530,
    }),
    Some(CoredllOrdinalDef {
        name: "FindResource",
        ordinal: 531,
    }),
    Some(CoredllOrdinalDef {
        name: "FindResourceW",
        ordinal: 532,
    }),
    Some(CoredllOrdinalDef {
        name: "LoadResource",
        ordinal: 533,
    }),
    Some(CoredllOrdinalDef {
        name: "LoadStringW",
        ordinal: 874,
    }),
    Some(CoredllOrdinalDef {
        name: "SizeofResource",
        ordinal: 534,
    }),
    Some(CoredllOrdinalDef {
        name: "VerQueryValueW",
        ordinal: 1459,
    }),
    Some(CoredllOrdinalDef {
        name: "GetFileVersionInfoW",
        ordinal: 1460,
    }),
    Some(CoredllOrdinalDef {
        name: "GetFileVersionInfoSizeW",
        ordinal: 1461,
    }),
    Some(CoredllOrdinalDef {
        name: "GetTickCount",
        ordinal: 535,
    }),
    Some(CoredllOrdinalDef {
        name: "GetProcessVersion",
        ordinal: 536,
    }),
    Some(CoredllOrdinalDef {
        name: "GetModuleFileNameW",
        ordinal: 537,
    }),
    Some(CoredllOrdinalDef {
        name: "GetModuleHandleW",
        ordinal: 1177,
    }),
    Some(CoredllOrdinalDef {
        name: "QueryPerformanceCounter",
        ordinal: 538,
    }),
    Some(CoredllOrdinalDef {
        name: "QueryPerformanceFrequency",
        ordinal: 539,
    }),
    Some(CoredllOrdinalDef {
        name: "ForcePageout",
        ordinal: 540,
    }),
    Some(CoredllOrdinalDef {
        name: "GetThreadTimes",
        ordinal: 1186,
    }),
    Some(CoredllOrdinalDef {
        name: "OutputDebugStringW",
        ordinal: 541,
    }),
    Some(CoredllOrdinalDef {
        name: "GetSystemInfo",
        ordinal: 542,
    }),
    Some(CoredllOrdinalDef {
        name: "QueryInstructionSet",
        ordinal: 1677,
    }),
    Some(CoredllOrdinalDef {
        name: "IsProcessorFeaturePresent",
        ordinal: 1758,
    }),
    Some(CoredllOrdinalDef {
        name: "RaiseException",
        ordinal: 543,
    }),
    Some(CoredllOrdinalDef {
        name: "TerminateProcess",
        ordinal: 544,
    }),
    Some(CoredllOrdinalDef {
        name: "NKDbgPrintfW",
        ordinal: 545,
    }),
    Some(CoredllOrdinalDef {
        name: "RegisterDbgZones",
        ordinal: 546,
    }),
    Some(CoredllOrdinalDef {
        name: "SetDaylightTime",
        ordinal: 547,
    }),
    Some(CoredllOrdinalDef {
        name: "GetCallStackSnapshot",
        ordinal: 1760,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "PageOutModule",
        ordinal: 1780,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "CreateFileMappingW",
        ordinal: 548,
    }),
    Some(CoredllOrdinalDef {
        name: "MapViewOfFile",
        ordinal: 549,
    }),
    Some(CoredllOrdinalDef {
        name: "UnmapViewOfFile",
        ordinal: 550,
    }),
    Some(CoredllOrdinalDef {
        name: "FlushViewOfFile",
        ordinal: 551,
    }),
    Some(CoredllOrdinalDef {
        name: "FlushViewOfFileMaybe",
        ordinal: 1215,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateFileForMapping",
        ordinal: 552,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateFileForMappingW",
        ordinal: 1167,
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "CloseHandle",
        ordinal: 553,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateMutexW",
        ordinal: 555,
    }),
    Some(CoredllOrdinalDef {
        name: "ReleaseMutex",
        ordinal: 556,
    }),
    Some(CoredllOrdinalDef {
        name: "KernelIoControl",
        ordinal: 557,
    }),
    Some(CoredllOrdinalDef {
        name: "KernelLibIoControl",
        ordinal: 1489,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateStaticMapping",
        ordinal: 1539,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "MapCallerPtr",
        ordinal: 1602,
    }),
    Some(CoredllOrdinalDef {
        name: "MapPtrToProcWithSize",
        ordinal: 1603,
    }),
    Some(CoredllOrdinalDef {
        name: "FreeLibraryAndExitThread",
        ordinal: 1216,
    }),
    Some(CoredllOrdinalDef {
        name: "GetProcAddressA",
        ordinal: 1230,
    }),
    Some(CoredllOrdinalDef {
        name: "GetCommandLineW",
        ordinal: 1231,
    }),
    Some(CoredllOrdinalDef {
        name: "DisableThreadLibraryCalls",
        ordinal: 1232,
    }),
    Some(CoredllOrdinalDef {
        name: "TryEnterCriticalSection",
        ordinal: 1233,
    }),
    Some(CoredllOrdinalDef {
        name: "GetTempFileNameW",
        ordinal: 1234,
    }),
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "CreateSemaphoreW",
        ordinal: 1238,
    }),
    Some(CoredllOrdinalDef {
        name: "ReleaseSemaphore",
        ordinal: 1239,
    }),
    Some(CoredllOrdinalDef {
        name: "CeMapArgumentArray",
        ordinal: 1446,
    }),
    Some(CoredllOrdinalDef {
        name: "CeSetExtendedPdata",
        ordinal: 1455,
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "AddEventAccess",
        ordinal: 558,
    }),
    Some(CoredllOrdinalDef {
        name: "VirtualCopy",
        ordinal: 560,
    }),
    None,
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "VirtualSetAttributes",
        ordinal: 1724,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "AllocPhysMem",
        ordinal: 1486,
    }),
    Some(CoredllOrdinalDef {
        name: "FreePhysMem",
        ordinal: 1487,
    }),
    Some(CoredllOrdinalDef {
        name: "SleepTillTick",
        ordinal: 1534,
    }),
    Some(CoredllOrdinalDef {
        name: "DuplicateHandle",
        ordinal: 1535,
    }),
    Some(CoredllOrdinalDef {
        name: "DeviceIoControl",
        ordinal: 179,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "LockPages",
        ordinal: 1161,
    }),
    Some(CoredllOrdinalDef {
        name: "UnlockPages",
        ordinal: 1162,
    }),
    Some(CoredllOrdinalDef {
        name: "U_ropen",
        ordinal: 563,
    }),
    Some(CoredllOrdinalDef {
        name: "U_rread",
        ordinal: 564,
    }),
    Some(CoredllOrdinalDef {
        name: "U_rwrite",
        ordinal: 565,
    }),
    Some(CoredllOrdinalDef {
        name: "U_rlseek",
        ordinal: 566,
    }),
    Some(CoredllOrdinalDef {
        name: "U_rclose",
        ordinal: 567,
    }),
    Some(CoredllOrdinalDef {
        name: "UpdateNLSInfo",
        ordinal: 1447,
    }),
    Some(CoredllOrdinalDef {
        name: "UpdateNLSInfoEx",
        ordinal: 1796,
    }),
    Some(CoredllOrdinalDef {
        name: "NKvDbgPrintfW",
        ordinal: 568,
    }),
    Some(CoredllOrdinalDef {
        name: "ProfileSyscall",
        ordinal: 569,
    }),
    Some(CoredllOrdinalDef {
        name: "GetRealTime",
        ordinal: 570,
    }),
    Some(CoredllOrdinalDef {
        name: "SetRealTime",
        ordinal: 571,
    }),
    Some(CoredllOrdinalDef {
        name: "ExtractResource",
        ordinal: 573,
    }),
    Some(CoredllOrdinalDef {
        name: "KernExtractIcons",
        ordinal: 574,
    }),
    Some(CoredllOrdinalDef {
        name: "GetRomFileInfo",
        ordinal: 575,
    }),
    Some(CoredllOrdinalDef {
        name: "GetRomFileBytes",
        ordinal: 576,
    }),
    Some(CoredllOrdinalDef {
        name: "CacheSync",
        ordinal: 577,
    }),
    Some(CoredllOrdinalDef {
        name: "CacheRangeFlush",
        ordinal: 1765,
    }),
    Some(CoredllOrdinalDef {
        name: "AddTrackedItem",
        ordinal: 578,
    }),
    Some(CoredllOrdinalDef {
        name: "DeleteTrackedItem",
        ordinal: 579,
    }),
    Some(CoredllOrdinalDef {
        name: "PrintTrackedItem",
        ordinal: 580,
    }),
    Some(CoredllOrdinalDef {
        name: "GetKPhys",
        ordinal: 581,
    }),
    Some(CoredllOrdinalDef {
        name: "GiveKPhys",
        ordinal: 582,
    }),
    Some(CoredllOrdinalDef {
        name: "SetExceptionHandler",
        ordinal: 583,
    }),
    Some(CoredllOrdinalDef {
        name: "RegisterTrackedItem",
        ordinal: 584,
    }),
    Some(CoredllOrdinalDef {
        name: "FilterTrackedItem",
        ordinal: 585,
    }),
    Some(CoredllOrdinalDef {
        name: "SetKernelAlarm",
        ordinal: 586,
    }),
    Some(CoredllOrdinalDef {
        name: "RefreshKernelAlarm",
        ordinal: 587,
    }),
    Some(CoredllOrdinalDef {
        name: "SetGwesOOMEvent",
        ordinal: 590,
    }),
    Some(CoredllOrdinalDef {
        name: "SetOOMEvent",
        ordinal: 1462,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCompress",
        ordinal: 591,
    }),
    Some(CoredllOrdinalDef {
        name: "StringDecompress",
        ordinal: 592,
    }),
    Some(CoredllOrdinalDef {
        name: "BinaryCompress",
        ordinal: 593,
    }),
    Some(CoredllOrdinalDef {
        name: "BinaryDecompress",
        ordinal: 594,
    }),
    Some(CoredllOrdinalDef {
        name: "DecompressBinaryBlock",
        ordinal: 1776,
    }),
    Some(CoredllOrdinalDef {
        name: "InputDebugCharW",
        ordinal: 595,
    }),
    Some(CoredllOrdinalDef {
        name: "MapPtrToProcess",
        ordinal: 598,
    }),
    Some(CoredllOrdinalDef {
        name: "MapPtrUnsecure",
        ordinal: 599,
    }),
    Some(CoredllOrdinalDef {
        name: "GetProcFromPtr",
        ordinal: 600,
    }),
    Some(CoredllOrdinalDef {
        name: "IsBadPtr",
        ordinal: 601,
    }),
    Some(CoredllOrdinalDef {
        name: "GetProcAddrBits",
        ordinal: 602,
    }),
    Some(CoredllOrdinalDef {
        name: "GetFSHeapInfo",
        ordinal: 603,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "GetOwnerProcess",
        ordinal: 606,
    }),
    Some(CoredllOrdinalDef {
        name: "GetCallerProcess",
        ordinal: 607,
    }),
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "GetIdleTime",
        ordinal: 608,
    }),
    Some(CoredllOrdinalDef {
        name: "SetLowestScheduledPriority",
        ordinal: 609,
    }),
    Some(CoredllOrdinalDef {
        name: "IsPrimaryThread",
        ordinal: 610,
    }),
    Some(CoredllOrdinalDef {
        name: "SetProcPermissions",
        ordinal: 611,
    }),
    Some(CoredllOrdinalDef {
        name: "CeGetCurrentTrust",
        ordinal: 1357,
    }),
    Some(CoredllOrdinalDef {
        name: "CeGetCallerTrust",
        ordinal: 1395,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "GetCurrentPermissions",
        ordinal: 612,
    }),
    Some(CoredllOrdinalDef {
        name: "SetTimeZoneBias",
        ordinal: 614,
    }),
    Some(CoredllOrdinalDef {
        name: "SetCleanRebootFlag",
        ordinal: 615,
    }),
    Some(CoredllOrdinalDef {
        name: "PowerOffSystem",
        ordinal: 617,
    }),
    Some(CoredllOrdinalDef {
        name: "SetDbgZone",
        ordinal: 618,
    }),
    Some(CoredllOrdinalDef {
        name: "TurnOnProfiling",
        ordinal: 619,
    }),
    Some(CoredllOrdinalDef {
        name: "TurnOffProfiling",
        ordinal: 620,
    }),
    Some(CoredllOrdinalDef {
        name: "GetProcName",
        ordinal: 624,
    }),
    Some(CoredllOrdinalDef {
        name: "SetHandleOwner",
        ordinal: 625,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "LoadDriver",
        ordinal: 626,
    }),
    Some(CoredllOrdinalDef {
        name: "LoadIntChainHandler",
        ordinal: 1475,
    }),
    Some(CoredllOrdinalDef {
        name: "FreeIntChainHandler",
        ordinal: 1476,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "LoadKernelLibrary",
        ordinal: 1671,
    }),
    Some(CoredllOrdinalDef {
        name: "InterruptInitialize",
        ordinal: 627,
    }),
    Some(CoredllOrdinalDef {
        name: "InterruptMask",
        ordinal: 1797,
    }),
    Some(CoredllOrdinalDef {
        name: "InterruptDone",
        ordinal: 628,
    }),
    Some(CoredllOrdinalDef {
        name: "InterruptDisable",
        ordinal: 629,
    }),
    Some(CoredllOrdinalDef {
        name: "SetKMode",
        ordinal: 630,
    }),
    Some(CoredllOrdinalDef {
        name: "SetPowerOffHandler",
        ordinal: 631,
    }),
    Some(CoredllOrdinalDef {
        name: "SetGwesPowerHandler",
        ordinal: 632,
    }),
    Some(CoredllOrdinalDef {
        name: "ConnectDebugger",
        ordinal: 633,
    }),
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "SetHardwareWatch",
        ordinal: 634,
    }),
    Some(CoredllOrdinalDef {
        name: "RegisterAPISet",
        ordinal: 635,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateAPIHandle",
        ordinal: 636,
    }),
    Some(CoredllOrdinalDef {
        name: "VerifyAPIHandle",
        ordinal: 637,
    }),
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "PPSHRestart",
        ordinal: 638,
    }),
    Some(CoredllOrdinalDef {
        name: "GetProcessIndexFromID",
        ordinal: 640,
    }),
    Some(CoredllOrdinalDef {
        name: "GetProcessIDFromIndex",
        ordinal: 1727,
    }),
    Some(CoredllOrdinalDef {
        name: "GetCallerProcessIndex",
        ordinal: 641,
    }),
    Some(CoredllOrdinalDef {
        name: "DebugNotify",
        ordinal: 642,
    }),
    Some(CoredllOrdinalDef {
        name: "ReadRegistryFromOEM",
        ordinal: 1153,
    }),
    Some(CoredllOrdinalDef {
        name: "WriteRegistryToOEM",
        ordinal: 1154,
    }),
    Some(CoredllOrdinalDef {
        name: "WriteDebugLED",
        ordinal: 1155,
    }),
    Some(CoredllOrdinalDef {
        name: "AFS_Unmount",
        ordinal: 643,
    }),
    Some(CoredllOrdinalDef {
        name: "AFS_CreateDirectoryW",
        ordinal: 644,
    }),
    Some(CoredllOrdinalDef {
        name: "AFS_RemoveDirectoryW",
        ordinal: 645,
    }),
    Some(CoredllOrdinalDef {
        name: "AFS_GetFileAttributesW",
        ordinal: 646,
    }),
    Some(CoredllOrdinalDef {
        name: "AFS_SetFileAttributesW",
        ordinal: 647,
    }),
    Some(CoredllOrdinalDef {
        name: "AFS_CreateFileW",
        ordinal: 648,
    }),
    Some(CoredllOrdinalDef {
        name: "AFS_DeleteFileW",
        ordinal: 649,
    }),
    Some(CoredllOrdinalDef {
        name: "AFS_MoveFileW",
        ordinal: 650,
    }),
    Some(CoredllOrdinalDef {
        name: "AFS_FindFirstFileW",
        ordinal: 651,
    }),
    Some(CoredllOrdinalDef {
        name: "AFS_RegisterFileSystemFunction",
        ordinal: 652,
    }),
    Some(CoredllOrdinalDef {
        name: "AFS_PrestoChangoFileName",
        ordinal: 654,
    }),
    Some(CoredllOrdinalDef {
        name: "AFS_CloseAllFileHandles",
        ordinal: 655,
    }),
    Some(CoredllOrdinalDef {
        name: "AFS_GetDiskFreeSpace",
        ordinal: 656,
    }),
    Some(CoredllOrdinalDef {
        name: "AFS_NotifyMountedFS",
        ordinal: 657,
    }),
    Some(CoredllOrdinalDef {
        name: "AFS_FindFirstChangeNotificationW",
        ordinal: 1685,
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "CreateCaret",
        ordinal: 658,
    }),
    Some(CoredllOrdinalDef {
        name: "DestroyCaret",
        ordinal: 659,
    }),
    Some(CoredllOrdinalDef {
        name: "HideCaret",
        ordinal: 660,
    }),
    Some(CoredllOrdinalDef {
        name: "ShowCaret",
        ordinal: 661,
    }),
    Some(CoredllOrdinalDef {
        name: "SetCaretPos",
        ordinal: 662,
    }),
    Some(CoredllOrdinalDef {
        name: "GetCaretPos",
        ordinal: 663,
    }),
    Some(CoredllOrdinalDef {
        name: "SetCaretBlinkTime",
        ordinal: 664,
    }),
    Some(CoredllOrdinalDef {
        name: "GetCaretBlinkTime",
        ordinal: 665,
    }),
    Some(CoredllOrdinalDef {
        name: "DisableCaretSystemWide",
        ordinal: 666,
    }),
    Some(CoredllOrdinalDef {
        name: "EnableCaretSystemWide",
        ordinal: 667,
    }),
    Some(CoredllOrdinalDef {
        name: "OpenClipboard",
        ordinal: 668,
    }),
    Some(CoredllOrdinalDef {
        name: "CloseClipboard",
        ordinal: 669,
    }),
    Some(CoredllOrdinalDef {
        name: "GetClipboardOwner",
        ordinal: 670,
    }),
    Some(CoredllOrdinalDef {
        name: "SetClipboardData",
        ordinal: 671,
    }),
    Some(CoredllOrdinalDef {
        name: "GetClipboardData",
        ordinal: 672,
    }),
    Some(CoredllOrdinalDef {
        name: "RegisterClipboardFormatW",
        ordinal: 673,
    }),
    Some(CoredllOrdinalDef {
        name: "CountClipboardFormats",
        ordinal: 674,
    }),
    Some(CoredllOrdinalDef {
        name: "EnumClipboardFormats",
        ordinal: 675,
    }),
    Some(CoredllOrdinalDef {
        name: "GetClipboardFormatNameW",
        ordinal: 676,
    }),
    Some(CoredllOrdinalDef {
        name: "EmptyClipboard",
        ordinal: 677,
    }),
    Some(CoredllOrdinalDef {
        name: "IsClipboardFormatAvailable",
        ordinal: 678,
    }),
    Some(CoredllOrdinalDef {
        name: "GetPriorityClipboardFormat",
        ordinal: 679,
    }),
    Some(CoredllOrdinalDef {
        name: "GetOpenClipboardWindow",
        ordinal: 680,
    }),
    Some(CoredllOrdinalDef {
        name: "GetClipboardDataAlloc",
        ordinal: 681,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateDialogIndirectParamW",
        ordinal: 688,
    }),
    Some(CoredllOrdinalDef {
        name: "DialogBoxIndirectParamW",
        ordinal: 690,
    }),
    Some(CoredllOrdinalDef {
        name: "CheckRadioButton",
        ordinal: 684,
    }),
    Some(CoredllOrdinalDef {
        name: "SendDlgItemMessageW",
        ordinal: 685,
    }),
    Some(CoredllOrdinalDef {
        name: "SetDlgItemTextW",
        ordinal: 686,
    }),
    Some(CoredllOrdinalDef {
        name: "GetDlgItemTextW",
        ordinal: 687,
    }),
    Some(CoredllOrdinalDef {
        name: "DefDlgProcW",
        ordinal: 689,
    }),
    Some(CoredllOrdinalDef {
        name: "EndDialog",
        ordinal: 691,
    }),
    Some(CoredllOrdinalDef {
        name: "GetDlgItem",
        ordinal: 692,
    }),
    Some(CoredllOrdinalDef {
        name: "GetDlgCtrlID",
        ordinal: 693,
    }),
    Some(CoredllOrdinalDef {
        name: "GetDialogBaseUnits",
        ordinal: 694,
    }),
    Some(CoredllOrdinalDef {
        name: "GetDlgItemInt",
        ordinal: 695,
    }),
    Some(CoredllOrdinalDef {
        name: "GetNextDlgTabItem",
        ordinal: 696,
    }),
    Some(CoredllOrdinalDef {
        name: "GetNextDlgGroupItem",
        ordinal: 697,
    }),
    Some(CoredllOrdinalDef {
        name: "IsDialogMessageW",
        ordinal: 698,
    }),
    Some(CoredllOrdinalDef {
        name: "MapDialogRect",
        ordinal: 699,
    }),
    Some(CoredllOrdinalDef {
        name: "SetDlgItemInt",
        ordinal: 700,
    }),
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "GetForegroundWindow",
        ordinal: 701,
    }),
    Some(CoredllOrdinalDef {
        name: "SetForegroundWindow",
        ordinal: 702,
    }),
    Some(CoredllOrdinalDef {
        name: "SetActiveWindow",
        ordinal: 703,
    }),
    Some(CoredllOrdinalDef {
        name: "SetFocus",
        ordinal: 704,
    }),
    Some(CoredllOrdinalDef {
        name: "GetFocus",
        ordinal: 705,
    }),
    Some(CoredllOrdinalDef {
        name: "GetActiveWindow",
        ordinal: 706,
    }),
    Some(CoredllOrdinalDef {
        name: "GetCapture",
        ordinal: 707,
    }),
    Some(CoredllOrdinalDef {
        name: "SetCapture",
        ordinal: 708,
    }),
    Some(CoredllOrdinalDef {
        name: "ReleaseCapture",
        ordinal: 709,
    }),
    Some(CoredllOrdinalDef {
        name: "SetKeyboardTarget",
        ordinal: 710,
    }),
    Some(CoredllOrdinalDef {
        name: "GetKeyboardTarget",
        ordinal: 711,
    }),
    Some(CoredllOrdinalDef {
        name: "ShellModalEnd",
        ordinal: 712,
    }),
    Some(CoredllOrdinalDef {
        name: "GetForegroundInfo",
        ordinal: 1224,
    }),
    Some(CoredllOrdinalDef {
        name: "GetForegroundKeyboardTarget",
        ordinal: 1225,
    }),
    Some(CoredllOrdinalDef {
        name: "GetForegroundKeyboardLayoutHandle",
        ordinal: 1802,
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "NotifyWinUserSystem",
        ordinal: 716,
    }),
    Some(CoredllOrdinalDef {
        name: "ExtractIconExW",
        ordinal: 727,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateIconIndirect",
        ordinal: 723,
    }),
    Some(CoredllOrdinalDef {
        name: "DestroyIcon",
        ordinal: 725,
    }),
    Some(CoredllOrdinalDef {
        name: "DrawIconEx",
        ordinal: 726,
    }),
    Some(CoredllOrdinalDef {
        name: "LoadIconW",
        ordinal: 728,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "DestroyCursor",
        ordinal: 724,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateCursor",
        ordinal: 722,
    }),
    Some(CoredllOrdinalDef {
        name: "SetCursor",
        ordinal: 682,
    }),
    Some(CoredllOrdinalDef {
        name: "LoadCursorW",
        ordinal: 683,
    }),
    Some(CoredllOrdinalDef {
        name: "ClipCursor",
        ordinal: 731,
    }),
    Some(CoredllOrdinalDef {
        name: "GetClipCursor",
        ordinal: 732,
    }),
    Some(CoredllOrdinalDef {
        name: "GetCursor",
        ordinal: 733,
    }),
    Some(CoredllOrdinalDef {
        name: "GetCursorPos",
        ordinal: 734,
    }),
    Some(CoredllOrdinalDef {
        name: "SetCursorPos",
        ordinal: 736,
    }),
    Some(CoredllOrdinalDef {
        name: "ShowCursor",
        ordinal: 737,
    }),
    Some(CoredllOrdinalDef {
        name: "LoadAnimatedCursor",
        ordinal: 1493,
    }),
    Some(CoredllOrdinalDef {
        name: "LoadImageW",
        ordinal: 730,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_Add",
        ordinal: 738,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_AddMasked",
        ordinal: 739,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_BeginDrag",
        ordinal: 740,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_CopyDitherImage",
        ordinal: 741,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_Create",
        ordinal: 742,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_Destroy",
        ordinal: 743,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_DragEnter",
        ordinal: 744,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_DragLeave",
        ordinal: 745,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_DragMove",
        ordinal: 746,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_DragShowNolock",
        ordinal: 747,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_Draw",
        ordinal: 748,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_DrawEx",
        ordinal: 749,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_DrawIndirect",
        ordinal: 750,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_EndDrag",
        ordinal: 751,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_GetBkColor",
        ordinal: 752,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_GetDragImage",
        ordinal: 753,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_GetIcon",
        ordinal: 754,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_GetIconSize",
        ordinal: 755,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_GetImageCount",
        ordinal: 756,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_GetImageInfo",
        ordinal: 757,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_LoadImage",
        ordinal: 758,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_Merge",
        ordinal: 759,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_Remove",
        ordinal: 760,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_Replace",
        ordinal: 761,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_ReplaceIcon",
        ordinal: 762,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_SetBkColor",
        ordinal: 763,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_SetDragCursorImage",
        ordinal: 764,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_SetIconSize",
        ordinal: 765,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_SetOverlayImage",
        ordinal: 766,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_Copy",
        ordinal: 767,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_Duplicate",
        ordinal: 768,
    }),
    Some(CoredllOrdinalDef {
        name: "ImageList_SetImageCount",
        ordinal: 769,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetContext",
        ordinal: 783,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetConversionStatus",
        ordinal: 785,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmNotifyIME",
        ordinal: 800,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmDisableIME",
        ordinal: 1206,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmEnableIME",
        ordinal: 1541,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmReleaseContext",
        ordinal: 803,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmSetConversionStatus",
        ordinal: 811,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetCompositionStringW",
        ordinal: 781,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmIsIME",
        ordinal: 1209,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetKeyboardLayout",
        ordinal: 1769,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmAssociateContext",
        ordinal: 770,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetOpenStatus",
        ordinal: 792,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmSIPanelState",
        ordinal: 804,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmEscapeW",
        ordinal: 775,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmCreateContext",
        ordinal: 1198,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmDestroyContext",
        ordinal: 1199,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmConfigureIMEW",
        ordinal: 771,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmCreateIMCC",
        ordinal: 772,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmDestroyIMCC",
        ordinal: 773,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmEnumRegisterWordW",
        ordinal: 774,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGenerateMessage",
        ordinal: 776,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetCandidateListW",
        ordinal: 777,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetCandidateListCountW",
        ordinal: 778,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetCandidateWindow",
        ordinal: 779,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetCompositionFontW",
        ordinal: 780,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetCompositionWindow",
        ordinal: 782,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetConversionListW",
        ordinal: 784,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetDefaultIMEWnd",
        ordinal: 786,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetDescriptionW",
        ordinal: 787,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetGuideLineW",
        ordinal: 788,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetIMCCLockCount",
        ordinal: 789,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetIMCCSize",
        ordinal: 790,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetIMCLockCount",
        ordinal: 791,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetProperty",
        ordinal: 793,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetRegisterWordStyleW",
        ordinal: 794,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmIsUIMessageW",
        ordinal: 796,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmLockIMC",
        ordinal: 797,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmLockIMCC",
        ordinal: 798,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmReSizeIMCC",
        ordinal: 801,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmRegisterWordW",
        ordinal: 802,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "ImmSetCandidateWindow",
        ordinal: 807,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmSetCompositionFontW",
        ordinal: 808,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmSetCompositionStringW",
        ordinal: 809,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmSetCompositionWindow",
        ordinal: 810,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmSetOpenStatus",
        ordinal: 814,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmSetStatusWindowPos",
        ordinal: 815,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetStatusWindowPos",
        ordinal: 1200,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmSimulateHotKey",
        ordinal: 816,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmUnlockIMC",
        ordinal: 817,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmUnlockIMCC",
        ordinal: 818,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmUnregisterWordW",
        ordinal: 819,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmAssociateContextEx",
        ordinal: 1205,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetIMEFileNameW",
        ordinal: 1207,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetVirtualKey",
        ordinal: 1210,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetImeMenuItemsW",
        ordinal: 1211,
    }),
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "ImmSetImeWndIMC",
        ordinal: 1222,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "ImmRequestMessageW",
        ordinal: 1242,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmSetHotKey",
        ordinal: 812,
    }),
    Some(CoredllOrdinalDef {
        name: "ImmGetHotKey",
        ordinal: 813,
    }),
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "GetMouseMovePoints",
        ordinal: 820,
    }),
    Some(CoredllOrdinalDef {
        name: "SendInput",
        ordinal: 823,
    }),
    Some(CoredllOrdinalDef {
        name: "mouse_event",
        ordinal: 824,
    }),
    Some(CoredllOrdinalDef {
        name: "QASetWindowsJournalHook",
        ordinal: 821,
    }),
    Some(CoredllOrdinalDef {
        name: "QAUnhookWindowsJournalHook",
        ordinal: 822,
    }),
    Some(CoredllOrdinalDef {
        name: "EnableHardwareKeyboard",
        ordinal: 825,
    }),
    Some(CoredllOrdinalDef {
        name: "GetAsyncKeyState",
        ordinal: 826,
    }),
    Some(CoredllOrdinalDef {
        name: "GetKeyboardStatus",
        ordinal: 827,
    }),
    Some(CoredllOrdinalDef {
        name: "KeybdGetDeviceInfo",
        ordinal: 828,
    }),
    Some(CoredllOrdinalDef {
        name: "KeybdInitStates",
        ordinal: 829,
    }),
    Some(CoredllOrdinalDef {
        name: "KeybdVKeyToUnicode",
        ordinal: 830,
    }),
    Some(CoredllOrdinalDef {
        name: "MapVirtualKeyW",
        ordinal: 831,
    }),
    Some(CoredllOrdinalDef {
        name: "PostKeybdMessage",
        ordinal: 832,
    }),
    Some(CoredllOrdinalDef {
        name: "keybd_event",
        ordinal: 833,
    }),
    Some(CoredllOrdinalDef {
        name: "GetAsyncShiftFlags",
        ordinal: 834,
    }),
    Some(CoredllOrdinalDef {
        name: "SetWindowsHookExW",
        ordinal: 1202,
    }),
    Some(CoredllOrdinalDef {
        name: "UnhookWindowsHookEx",
        ordinal: 1203,
    }),
    Some(CoredllOrdinalDef {
        name: "CallNextHookEx",
        ordinal: 1204,
    }),
    Some(CoredllOrdinalDef {
        name: "RegisterHotKey",
        ordinal: 835,
    }),
    Some(CoredllOrdinalDef {
        name: "UnregisterHotKey",
        ordinal: 836,
    }),
    Some(CoredllOrdinalDef {
        name: "UnregisterFunc1",
        ordinal: 1156,
    }),
    Some(CoredllOrdinalDef {
        name: "AllKeys",
        ordinal: 1453,
    }),
    Some(CoredllOrdinalDef {
        name: "GetKeyboardType",
        ordinal: 1771,
    }),
    Some(CoredllOrdinalDef {
        name: "GetKeyboardLayoutList",
        ordinal: 1767,
    }),
    Some(CoredllOrdinalDef {
        name: "LoadKeyboardLayoutW",
        ordinal: 1768,
    }),
    Some(CoredllOrdinalDef {
        name: "GetKeyboardLayout",
        ordinal: 1229,
    }),
    Some(CoredllOrdinalDef {
        name: "GetKeyboardLayoutNameW",
        ordinal: 1160,
    }),
    Some(CoredllOrdinalDef {
        name: "ActivateKeyboardLayout",
        ordinal: 1766,
    }),
    Some(CoredllOrdinalDef {
        name: "SystemIdleTimerReset",
        ordinal: 837,
    }),
    Some(CoredllOrdinalDef {
        name: "TranslateAcceleratorW",
        ordinal: 838,
    }),
    Some(CoredllOrdinalDef {
        name: "NLedGetDeviceInfo",
        ordinal: 839,
    }),
    Some(CoredllOrdinalDef {
        name: "NLedSetDevice",
        ordinal: 840,
    }),
    Some(CoredllOrdinalDef {
        name: "InsertMenuW",
        ordinal: 841,
    }),
    Some(CoredllOrdinalDef {
        name: "AppendMenuW",
        ordinal: 842,
    }),
    Some(CoredllOrdinalDef {
        name: "RemoveMenu",
        ordinal: 843,
    }),
    Some(CoredllOrdinalDef {
        name: "DestroyMenu",
        ordinal: 844,
    }),
    Some(CoredllOrdinalDef {
        name: "TrackPopupMenuEx",
        ordinal: 845,
    }),
    Some(CoredllOrdinalDef {
        name: "LoadMenuW",
        ordinal: 846,
    }),
    Some(CoredllOrdinalDef {
        name: "EnableMenuItem",
        ordinal: 847,
    }),
    Some(CoredllOrdinalDef {
        name: "CheckMenuItem",
        ordinal: 848,
    }),
    Some(CoredllOrdinalDef {
        name: "CheckMenuRadioItem",
        ordinal: 849,
    }),
    Some(CoredllOrdinalDef {
        name: "DeleteMenu",
        ordinal: 850,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateMenu",
        ordinal: 851,
    }),
    Some(CoredllOrdinalDef {
        name: "CreatePopupMenu",
        ordinal: 852,
    }),
    Some(CoredllOrdinalDef {
        name: "SetMenuItemInfoW",
        ordinal: 853,
    }),
    Some(CoredllOrdinalDef {
        name: "GetMenuItemInfoW",
        ordinal: 854,
    }),
    Some(CoredllOrdinalDef {
        name: "GetSubMenu",
        ordinal: 855,
    }),
    Some(CoredllOrdinalDef {
        name: "DrawMenuBar",
        ordinal: 856,
    }),
    Some(CoredllOrdinalDef {
        name: "MessageBeep",
        ordinal: 857,
    }),
    Some(CoredllOrdinalDef {
        name: "MessageBoxW",
        ordinal: 858,
    }),
    Some(CoredllOrdinalDef {
        name: "DispatchMessageW",
        ordinal: 859,
    }),
    Some(CoredllOrdinalDef {
        name: "GetKeyState",
        ordinal: 860,
    }),
    Some(CoredllOrdinalDef {
        name: "GetMessageW",
        ordinal: 861,
    }),
    Some(CoredllOrdinalDef {
        name: "GetMessagePos",
        ordinal: 862,
    }),
    Some(CoredllOrdinalDef {
        name: "GetMessageWNoWait",
        ordinal: 863,
    }),
    Some(CoredllOrdinalDef {
        name: "PeekMessageW",
        ordinal: 864,
    }),
    Some(CoredllOrdinalDef {
        name: "PostMessageW",
        ordinal: 865,
    }),
    Some(CoredllOrdinalDef {
        name: "PostQuitMessage",
        ordinal: 866,
    }),
    Some(CoredllOrdinalDef {
        name: "SendMessageW",
        ordinal: 868,
    }),
    Some(CoredllOrdinalDef {
        name: "SendNotifyMessageW",
        ordinal: 869,
    }),
    Some(CoredllOrdinalDef {
        name: "TranslateMessage",
        ordinal: 870,
    }),
    Some(CoredllOrdinalDef {
        name: "MsgWaitForMultipleObjectsEx",
        ordinal: 871,
    }),
    Some(CoredllOrdinalDef {
        name: "GetMessageSource",
        ordinal: 872,
    }),
    Some(CoredllOrdinalDef {
        name: "InSendMessage",
        ordinal: 1419,
    }),
    Some(CoredllOrdinalDef {
        name: "GetQueueStatus",
        ordinal: 1420,
    }),
    Some(CoredllOrdinalDef {
        name: "SendMessageTimeout",
        ordinal: 1495,
    }),
    Some(CoredllOrdinalDef {
        name: "GetMessageQueueReadyTimeStamp",
        ordinal: 1477,
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "LoadBitmapW",
        ordinal: 873,
    }),
    Some(CoredllOrdinalDef {
        name: "SetTimer",
        ordinal: 875,
    }),
    Some(CoredllOrdinalDef {
        name: "KillTimer",
        ordinal: 876,
    }),
    Some(CoredllOrdinalDef {
        name: "TouchCalibrate",
        ordinal: 877,
    }),
    Some(CoredllOrdinalDef {
        name: "GetClassInfoW",
        ordinal: 878,
    }),
    Some(CoredllOrdinalDef {
        name: "GetClassLongW",
        ordinal: 879,
    }),
    Some(CoredllOrdinalDef {
        name: "SetClassLongW",
        ordinal: 880,
    }),
    Some(CoredllOrdinalDef {
        name: "GetClassLong",
        ordinal: 881,
    }),
    Some(CoredllOrdinalDef {
        name: "SetClassLong",
        ordinal: 882,
    }),
    Some(CoredllOrdinalDef {
        name: "RegisterClassW",
        ordinal: 95,
    }),
    Some(CoredllOrdinalDef {
        name: "UnregisterClassW",
        ordinal: 884,
    }),
    Some(CoredllOrdinalDef {
        name: "GetSystemMetrics",
        ordinal: 885,
    }),
    Some(CoredllOrdinalDef {
        name: "IsWindowVisible",
        ordinal: 886,
    }),
    Some(CoredllOrdinalDef {
        name: "GetDC",
        ordinal: 262,
    }),
    Some(CoredllOrdinalDef {
        name: "GetWindowDC",
        ordinal: 270,
    }),
    Some(CoredllOrdinalDef {
        name: "ReleaseDC",
        ordinal: 263,
    }),
    Some(CoredllOrdinalDef {
        name: "AdjustWindowRectEx",
        ordinal: 887,
    }),
    Some(CoredllOrdinalDef {
        name: "GetDoubleClickTime",
        ordinal: 888,
    }),
    Some(CoredllOrdinalDef {
        name: "GetSysColor",
        ordinal: 889,
    }),
    Some(CoredllOrdinalDef {
        name: "SetSysColors",
        ordinal: 890,
    }),
    Some(CoredllOrdinalDef {
        name: "RegisterWindowMessageW",
        ordinal: 891,
    }),
    Some(CoredllOrdinalDef {
        name: "RegisterTaskBar",
        ordinal: 892,
    }),
    Some(CoredllOrdinalDef {
        name: "RegisterTaskBarEx",
        ordinal: 1506,
    }),
    Some(CoredllOrdinalDef {
        name: "RegisterDesktop",
        ordinal: 1507,
    }),
    Some(CoredllOrdinalDef {
        name: "SetProp",
        ordinal: 1497,
    }),
    Some(CoredllOrdinalDef {
        name: "GetProp",
        ordinal: 1498,
    }),
    Some(CoredllOrdinalDef {
        name: "RemoveProp",
        ordinal: 1499,
    }),
    Some(CoredllOrdinalDef {
        name: "EnumPropsEx",
        ordinal: 1500,
    }),
    Some(CoredllOrdinalDef {
        name: "GlobalAddAtomW",
        ordinal: 1519,
    }),
    Some(CoredllOrdinalDef {
        name: "GlobalDeleteAtom",
        ordinal: 1520,
    }),
    Some(CoredllOrdinalDef {
        name: "GlobalFindAtomW",
        ordinal: 1521,
    }),
    Some(CoredllOrdinalDef {
        name: "AddFontResourceW",
        ordinal: 893,
    }),
    Some(CoredllOrdinalDef {
        name: "CeRemoveFontResource",
        ordinal: 894,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateFontIndirectW",
        ordinal: 895,
    }),
    Some(CoredllOrdinalDef {
        name: "ExtTextOutW",
        ordinal: 896,
    }),
    Some(CoredllOrdinalDef {
        name: "GetTextExtentExPointW",
        ordinal: 897,
    }),
    Some(CoredllOrdinalDef {
        name: "GetTextMetricsW",
        ordinal: 898,
    }),
    Some(CoredllOrdinalDef {
        name: "PegRemoveFontResource",
        ordinal: 899,
    }),
    Some(CoredllOrdinalDef {
        name: "RemoveFontResourceW",
        ordinal: 900,
    }),
    Some(CoredllOrdinalDef {
        name: "SetTextAlign",
        ordinal: 1654,
    }),
    Some(CoredllOrdinalDef {
        name: "GetTextAlign",
        ordinal: 1655,
    }),
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "GetCharWidth32",
        ordinal: 1664,
    }),
    Some(CoredllOrdinalDef {
        name: "GetCharABCWidths",
        ordinal: 1779,
    }),
    None,
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "CreateBitmap",
        ordinal: 901,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateCompatibleBitmap",
        ordinal: 902,
    }),
    Some(CoredllOrdinalDef {
        name: "SetBitmapBits",
        ordinal: 1725,
    }),
    Some(CoredllOrdinalDef {
        name: "BitBlt",
        ordinal: 903,
    }),
    Some(CoredllOrdinalDef {
        name: "MaskBlt",
        ordinal: 904,
    }),
    Some(CoredllOrdinalDef {
        name: "StretchBlt",
        ordinal: 905,
    }),
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "TransparentImage",
        ordinal: 906,
    }),
    Some(CoredllOrdinalDef {
        name: "StretchDIBits",
        ordinal: 1667,
    }),
    Some(CoredllOrdinalDef {
        name: "SetDIBitsToDevice",
        ordinal: 1726,
    }),
    Some(CoredllOrdinalDef {
        name: "RestoreDC",
        ordinal: 907,
    }),
    Some(CoredllOrdinalDef {
        name: "SaveDC",
        ordinal: 908,
    }),
    Some(CoredllOrdinalDef {
        name: "ExtEscape",
        ordinal: 1182,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateDCW",
        ordinal: 909,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateCompatibleDC",
        ordinal: 910,
    }),
    Some(CoredllOrdinalDef {
        name: "DeleteDC",
        ordinal: 911,
    }),
    Some(CoredllOrdinalDef {
        name: "DeleteObject",
        ordinal: 912,
    }),
    Some(CoredllOrdinalDef {
        name: "GetBkColor",
        ordinal: 913,
    }),
    Some(CoredllOrdinalDef {
        name: "GetBkMode",
        ordinal: 914,
    }),
    Some(CoredllOrdinalDef {
        name: "GetCurrentObject",
        ordinal: 915,
    }),
    Some(CoredllOrdinalDef {
        name: "GetDeviceCaps",
        ordinal: 916,
    }),
    Some(CoredllOrdinalDef {
        name: "GetObjectType",
        ordinal: 917,
    }),
    Some(CoredllOrdinalDef {
        name: "GetObjectW",
        ordinal: 918,
    }),
    Some(CoredllOrdinalDef {
        name: "GetStockObject",
        ordinal: 919,
    }),
    Some(CoredllOrdinalDef {
        name: "GetTextColor",
        ordinal: 920,
    }),
    Some(CoredllOrdinalDef {
        name: "SelectObject",
        ordinal: 921,
    }),
    Some(CoredllOrdinalDef {
        name: "SetBkColor",
        ordinal: 922,
    }),
    Some(CoredllOrdinalDef {
        name: "SetBkMode",
        ordinal: 923,
    }),
    Some(CoredllOrdinalDef {
        name: "SetTextColor",
        ordinal: 924,
    }),
    Some(CoredllOrdinalDef {
        name: "GetDIBColorTable",
        ordinal: 1665,
    }),
    Some(CoredllOrdinalDef {
        name: "SetDIBColorTable",
        ordinal: 1666,
    }),
    Some(CoredllOrdinalDef {
        name: "EnumDisplaySettings",
        ordinal: 1777,
    }),
    Some(CoredllOrdinalDef {
        name: "EnumDisplayDevices",
        ordinal: 1778,
    }),
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "CreatePatternBrush",
        ordinal: 925,
    }),
    Some(CoredllOrdinalDef {
        name: "CreatePen",
        ordinal: 926,
    }),
    Some(CoredllOrdinalDef {
        name: "FillRgn",
        ordinal: 927,
    }),
    Some(CoredllOrdinalDef {
        name: "SetROP2",
        ordinal: 928,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "CreateDIBPatternBrushPt",
        ordinal: 929,
    }),
    Some(CoredllOrdinalDef {
        name: "CreatePenIndirect",
        ordinal: 930,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateSolidBrush",
        ordinal: 931,
    }),
    Some(CoredllOrdinalDef {
        name: "DrawEdge",
        ordinal: 932,
    }),
    Some(CoredllOrdinalDef {
        name: "DrawFocusRect",
        ordinal: 933,
    }),
    Some(CoredllOrdinalDef {
        name: "Ellipse",
        ordinal: 934,
    }),
    Some(CoredllOrdinalDef {
        name: "FillRect",
        ordinal: 935,
    }),
    Some(CoredllOrdinalDef {
        name: "GetPixel",
        ordinal: 936,
    }),
    Some(CoredllOrdinalDef {
        name: "GetSysColorBrush",
        ordinal: 937,
    }),
    Some(CoredllOrdinalDef {
        name: "PatBlt",
        ordinal: 938,
    }),
    Some(CoredllOrdinalDef {
        name: "InvertRect",
        ordinal: 1770,
    }),
    Some(CoredllOrdinalDef {
        name: "Polygon",
        ordinal: 939,
    }),
    Some(CoredllOrdinalDef {
        name: "Polyline",
        ordinal: 940,
    }),
    Some(CoredllOrdinalDef {
        name: "Rectangle",
        ordinal: 941,
    }),
    Some(CoredllOrdinalDef {
        name: "RoundRect",
        ordinal: 942,
    }),
    Some(CoredllOrdinalDef {
        name: "SetBrushOrgEx",
        ordinal: 943,
    }),
    Some(CoredllOrdinalDef {
        name: "SetPixel",
        ordinal: 944,
    }),
    Some(CoredllOrdinalDef {
        name: "MoveToEx",
        ordinal: 1651,
    }),
    Some(CoredllOrdinalDef {
        name: "LineTo",
        ordinal: 1652,
    }),
    Some(CoredllOrdinalDef {
        name: "GetCurrentPositionEx",
        ordinal: 1653,
    }),
    Some(CoredllOrdinalDef {
        name: "DrawTextW",
        ordinal: 945,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateBitmapFromPointer",
        ordinal: 946,
    }),
    Some(CoredllOrdinalDef {
        name: "CreatePalette",
        ordinal: 947,
    }),
    Some(CoredllOrdinalDef {
        name: "GetNearestPaletteIndex",
        ordinal: 948,
    }),
    Some(CoredllOrdinalDef {
        name: "GetPaletteEntries",
        ordinal: 949,
    }),
    Some(CoredllOrdinalDef {
        name: "GetSystemPaletteEntries",
        ordinal: 950,
    }),
    Some(CoredllOrdinalDef {
        name: "SetPaletteEntries",
        ordinal: 951,
    }),
    Some(CoredllOrdinalDef {
        name: "GetNearestColor",
        ordinal: 952,
    }),
    Some(CoredllOrdinalDef {
        name: "RealizePalette",
        ordinal: 953,
    }),
    Some(CoredllOrdinalDef {
        name: "SelectPalette",
        ordinal: 954,
    }),
    Some(CoredllOrdinalDef {
        name: "GradientFill",
        ordinal: 1763,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "AbortDoc",
        ordinal: 955,
    }),
    Some(CoredllOrdinalDef {
        name: "CloseEnhMetaFile",
        ordinal: 956,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateEnhMetaFileW",
        ordinal: 957,
    }),
    Some(CoredllOrdinalDef {
        name: "DeleteEnhMetaFile",
        ordinal: 958,
    }),
    Some(CoredllOrdinalDef {
        name: "EndDoc",
        ordinal: 959,
    }),
    Some(CoredllOrdinalDef {
        name: "EndPage",
        ordinal: 960,
    }),
    Some(CoredllOrdinalDef {
        name: "PlayEnhMetaFile",
        ordinal: 961,
    }),
    Some(CoredllOrdinalDef {
        name: "SetAbortProc",
        ordinal: 962,
    }),
    Some(CoredllOrdinalDef {
        name: "StartDocW",
        ordinal: 963,
    }),
    Some(CoredllOrdinalDef {
        name: "StartPage",
        ordinal: 964,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "EnumFontFamiliesW",
        ordinal: 965,
    }),
    Some(CoredllOrdinalDef {
        name: "EnumFontsW",
        ordinal: 966,
    }),
    Some(CoredllOrdinalDef {
        name: "GetTextFaceW",
        ordinal: 967,
    }),
    Some(CoredllOrdinalDef {
        name: "TranslateCharsetInfo",
        ordinal: 1166,
    }),
    Some(CoredllOrdinalDef {
        name: "CombineRgn",
        ordinal: 968,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateRectRgnIndirect",
        ordinal: 969,
    }),
    Some(CoredllOrdinalDef {
        name: "ExcludeClipRect",
        ordinal: 970,
    }),
    Some(CoredllOrdinalDef {
        name: "GetClipBox",
        ordinal: 971,
    }),
    Some(CoredllOrdinalDef {
        name: "GetClipRgn",
        ordinal: 972,
    }),
    Some(CoredllOrdinalDef {
        name: "GetRegionData",
        ordinal: 973,
    }),
    Some(CoredllOrdinalDef {
        name: "GetRgnBox",
        ordinal: 974,
    }),
    Some(CoredllOrdinalDef {
        name: "IntersectClipRect",
        ordinal: 975,
    }),
    Some(CoredllOrdinalDef {
        name: "OffsetRgn",
        ordinal: 976,
    }),
    Some(CoredllOrdinalDef {
        name: "PtInRegion",
        ordinal: 977,
    }),
    Some(CoredllOrdinalDef {
        name: "RectInRegion",
        ordinal: 978,
    }),
    Some(CoredllOrdinalDef {
        name: "SelectClipRgn",
        ordinal: 979,
    }),
    Some(CoredllOrdinalDef {
        name: "CreateRectRgn",
        ordinal: 980,
    }),
    Some(CoredllOrdinalDef {
        name: "RectVisible",
        ordinal: 981,
    }),
    Some(CoredllOrdinalDef {
        name: "SetRectRgn",
        ordinal: 982,
    }),
    Some(CoredllOrdinalDef {
        name: "ExtCreateRegion",
        ordinal: 1617,
    }),
    Some(CoredllOrdinalDef {
        name: "SetViewportOrgEx",
        ordinal: 983,
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "SetObjectOwner",
        ordinal: 984,
    }),
    Some(CoredllOrdinalDef {
        name: "ScrollDC",
        ordinal: 985,
    }),
    Some(CoredllOrdinalDef {
        name: "EnableEUDC",
        ordinal: 986,
    }),
    Some(CoredllOrdinalDef {
        name: "DrawFrameControl",
        ordinal: 987,
    }),
    Some(CoredllOrdinalDef {
        name: "SetScrollInfo",
        ordinal: 279,
    }),
    Some(CoredllOrdinalDef {
        name: "SetScrollPos",
        ordinal: 280,
    }),
    Some(CoredllOrdinalDef {
        name: "SetScrollRange",
        ordinal: 281,
    }),
    Some(CoredllOrdinalDef {
        name: "GetScrollInfo",
        ordinal: 282,
    }),
    None,
    Some(CoredllOrdinalDef {
        name: "StringCchCopyA",
        ordinal: 1705,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbCopyA",
        ordinal: 1706,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchCopyExA",
        ordinal: 1707,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbCopyExA",
        ordinal: 1708,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchCopyNA",
        ordinal: 1750,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbCopyNA",
        ordinal: 1751,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchCatA",
        ordinal: 1709,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbCatA",
        ordinal: 1710,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchCatExA",
        ordinal: 1711,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbCatExA",
        ordinal: 1712,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchCatNA",
        ordinal: 1752,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbCatNA",
        ordinal: 1753,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchCatNExA",
        ordinal: 1754,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbCatNExA",
        ordinal: 1755,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchVPrintfA",
        ordinal: 1713,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbVPrintfA",
        ordinal: 1714,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchPrintfA",
        ordinal: 1715,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbPrintfA",
        ordinal: 1716,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchPrintfExA",
        ordinal: 1717,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbPrintfExA",
        ordinal: 1718,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchVPrintfExA",
        ordinal: 1719,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbVPrintfExA",
        ordinal: 1720,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCchLengthA",
        ordinal: 1756,
    }),
    Some(CoredllOrdinalDef {
        name: "StringCbLengthA",
        ordinal: 1757,
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "MonitorFromPoint",
        ordinal: 1522,
    }),
    Some(CoredllOrdinalDef {
        name: "MonitorFromRect",
        ordinal: 1523,
    }),
    Some(CoredllOrdinalDef {
        name: "MonitorFromWindow",
        ordinal: 1524,
    }),
    Some(CoredllOrdinalDef {
        name: "GetMonitorInfo",
        ordinal: 1525,
    }),
    Some(CoredllOrdinalDef {
        name: "EnumDisplayMonitors",
        ordinal: 1526,
    }),
    Some(CoredllOrdinalDef {
        name: "A_SHAInit",
        ordinal: 1789,
    }),
    Some(CoredllOrdinalDef {
        name: "A_SHAUpdate",
        ordinal: 1790,
    }),
    Some(CoredllOrdinalDef {
        name: "A_SHAFinal",
        ordinal: 1791,
    }),
    Some(CoredllOrdinalDef {
        name: "MD5Init",
        ordinal: 1792,
    }),
    Some(CoredllOrdinalDef {
        name: "MD5Update",
        ordinal: 1793,
    }),
    Some(CoredllOrdinalDef {
        name: "MD5Final",
        ordinal: 1794,
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "GetStdioPathW",
        ordinal: 1149,
    }),
    Some(CoredllOrdinalDef {
        name: "SetStdioPathW",
        ordinal: 1150,
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(CoredllOrdinalDef {
        name: "abs",
        ordinal: 988,
    }),
    Some(CoredllOrdinalDef {
        name: "acos",
        ordinal: 989,
    }),
    Some(CoredllOrdinalDef {
        name: "asin",
        ordinal: 990,
    }),
    Some(CoredllOrdinalDef {
        name: "atan",
        ordinal: 991,
    }),
    Some(CoredllOrdinalDef {
        name: "atan2",
        ordinal: 992,
    }),
    Some(CoredllOrdinalDef {
        name: "ceil",
        ordinal: 999,
    }),
    Some(CoredllOrdinalDef {
        name: "cos",
        ordinal: 1004,
    }),
    Some(CoredllOrdinalDef {
        name: "cosh",
        ordinal: 1005,
    }),
    Some(CoredllOrdinalDef {
        name: "div",
        ordinal: 1007,
    }),
    Some(CoredllOrdinalDef {
        name: "exp",
        ordinal: 1009,
    }),
    Some(CoredllOrdinalDef {
        name: "fabs",
        ordinal: 1010,
    }),
    Some(CoredllOrdinalDef {
        name: "floor",
        ordinal: 1013,
    }),
    Some(CoredllOrdinalDef {
        name: "fmod",
        ordinal: 1014,
    }),
    Some(CoredllOrdinalDef {
        name: "frexp",
        ordinal: 1019,
    }),
    Some(CoredllOrdinalDef {
        name: "labs",
        ordinal: 1030,
    }),
    Some(CoredllOrdinalDef {
        name: "ldexp",
        ordinal: 1031,
    }),
    Some(CoredllOrdinalDef {
        name: "ldiv",
        ordinal: 1032,
    }),
    Some(CoredllOrdinalDef {
        name: "log",
        ordinal: 1033,
    }),
    Some(CoredllOrdinalDef {
        name: "log10",
        ordinal: 1034,
    }),
    Some(CoredllOrdinalDef {
        name: "modf",
        ordinal: 1048,
    }),
    Some(CoredllOrdinalDef {
        name: "pow",
        ordinal: 1051,
    }),
    Some(CoredllOrdinalDef {
        name: "sin",
        ordinal: 1058,
    }),
    Some(CoredllOrdinalDef {
        name: "sinh",
        ordinal: 1059,
    }),
    Some(CoredllOrdinalDef {
        name: "sqrt",
        ordinal: 1060,
    }),
    Some(CoredllOrdinalDef {
        name: "tan",
        ordinal: 1075,
    }),
    Some(CoredllOrdinalDef {
        name: "tanh",
        ordinal: 1076,
    }),
    Some(CoredllOrdinalDef {
        name: "__ll_rshift",
        ordinal: 2002,
    }),
    Some(CoredllOrdinalDef {
        name: "__ll_lshift",
        ordinal: 2003,
    }),
    Some(CoredllOrdinalDef {
        name: "__ll_mul",
        ordinal: 2004,
    }),
    Some(CoredllOrdinalDef {
        name: "__ll_div",
        ordinal: 2005,
    }),
    Some(CoredllOrdinalDef {
        name: "__ll_rem",
        ordinal: 2006,
    }),
    Some(CoredllOrdinalDef {
        name: "__ull_rshift",
        ordinal: 2011,
    }),
    Some(CoredllOrdinalDef {
        name: "__ull_div",
        ordinal: 2012,
    }),
    Some(CoredllOrdinalDef {
        name: "__ull_rem",
        ordinal: 2013,
    }),
    Some(CoredllOrdinalDef {
        name: "__fpadd",
        ordinal: 2022,
    }),
    Some(CoredllOrdinalDef {
        name: "__dpadd",
        ordinal: 2023,
    }),
    Some(CoredllOrdinalDef {
        name: "__fpsub",
        ordinal: 2024,
    }),
    Some(CoredllOrdinalDef {
        name: "__dpsub",
        ordinal: 2025,
    }),
    Some(CoredllOrdinalDef {
        name: "__fpmul",
        ordinal: 2026,
    }),
    Some(CoredllOrdinalDef {
        name: "__dpmul",
        ordinal: 2027,
    }),
    Some(CoredllOrdinalDef {
        name: "__fpdiv",
        ordinal: 2028,
    }),
    Some(CoredllOrdinalDef {
        name: "__dpdiv",
        ordinal: 2029,
    }),
    Some(CoredllOrdinalDef {
        name: "__fptoli",
        ordinal: 2030,
    }),
    Some(CoredllOrdinalDef {
        name: "__fptoul",
        ordinal: 2031,
    }),
    Some(CoredllOrdinalDef {
        name: "__litofp",
        ordinal: 2032,
    }),
    Some(CoredllOrdinalDef {
        name: "__ultofp",
        ordinal: 2033,
    }),
    Some(CoredllOrdinalDef {
        name: "__dptoli",
        ordinal: 2034,
    }),
    Some(CoredllOrdinalDef {
        name: "__dptoul",
        ordinal: 2035,
    }),
    Some(CoredllOrdinalDef {
        name: "__litodp",
        ordinal: 2036,
    }),
    Some(CoredllOrdinalDef {
        name: "__ultodp",
        ordinal: 2037,
    }),
    Some(CoredllOrdinalDef {
        name: "__fptodp",
        ordinal: 2038,
    }),
    Some(CoredllOrdinalDef {
        name: "__dptofp",
        ordinal: 2039,
    }),
    Some(CoredllOrdinalDef {
        name: "__fpcmp",
        ordinal: 2040,
    }),
    Some(CoredllOrdinalDef {
        name: "__dpcmp",
        ordinal: 2041,
    }),
    Some(CoredllOrdinalDef {
        name: "__lts",
        ordinal: 2042,
    }),
    Some(CoredllOrdinalDef {
        name: "__les",
        ordinal: 2043,
    }),
    Some(CoredllOrdinalDef {
        name: "__eqs",
        ordinal: 2044,
    }),
    Some(CoredllOrdinalDef {
        name: "__ges",
        ordinal: 2045,
    }),
    Some(CoredllOrdinalDef {
        name: "__gts",
        ordinal: 2046,
    }),
    Some(CoredllOrdinalDef {
        name: "__nes",
        ordinal: 2047,
    }),
    Some(CoredllOrdinalDef {
        name: "__ltd",
        ordinal: 2048,
    }),
    Some(CoredllOrdinalDef {
        name: "__led",
        ordinal: 2049,
    }),
    Some(CoredllOrdinalDef {
        name: "__eqd",
        ordinal: 2050,
    }),
    Some(CoredllOrdinalDef {
        name: "__ged",
        ordinal: 2051,
    }),
    Some(CoredllOrdinalDef {
        name: "__gtd",
        ordinal: 2052,
    }),
    Some(CoredllOrdinalDef {
        name: "__ned",
        ordinal: 2053,
    }),
];

pub fn is_current_map_export(export: &CoredllOrdinalDef) -> bool {
    COREDLL_EXPORTS
        .iter()
        .any(|current| current.name == export.name && current.ordinal == export.ordinal)
}

pub fn current_static_export_count() -> usize {
    COREDLL_EXPORTS.len()
}

pub const ORD_SYSTEM_MEMORY_LOW: u32 = 720;
pub const ORD_WCSDUP: u32 = 74;
pub const ORD_WCSTOMBS: u32 = 75;
pub const ORD_MBSTOWCS: u32 = 76;
pub const ORD_WTOL: u32 = 78;
pub const ORD_WCSRCHR: u32 = 69;
pub const ORD_WCSSTR: u32 = 73;
pub const ORD_WCSICMP: u32 = 230;
pub const ORD_WCSNICMP: u32 = 229;
pub const ORD_WCSNCMP: u32 = 65;
pub const ORD_WCSCHR: u32 = 59;
pub const ORD_WCSLEN: u32 = 63;
pub const ORD_WCSCPY: u32 = 61;
pub const ORD_WCSNCPY: u32 = 66;
pub const ORD_WCSPBRK: u32 = 68;
pub const ORD_WCSTOUL: u32 = 1083;
pub const ORD_MALLOC: u32 = 1041;
pub const ORD_MSIZE: u32 = 1049;
pub const ORD_REALLOC: u32 = 1054;
pub const ORD_MEMCMP: u32 = 1043;
pub const ORD_MEMCPY: u32 = 1044;
pub const ORD_MEMMOVE: u32 = 1046;
pub const ORD_MEMSET: u32 = 1047;
pub const ORD_QSORT: u32 = 1052;
pub const ORD_ATOF: u32 = 995;
pub const ORD_ATOI: u32 = 993;
pub const ORD_TOLOWER: u32 = 1090;
pub const ORD_TOUPPER: u32 = 1091;
pub const ORD_STRCAT: u32 = 1063;
pub const ORD_STRCPY: u32 = 1066;
pub const ORD_STRTOK: u32 = 1073;
pub const ORD_STRTOUL: u32 = 1405;
pub const ORD_STRUPR: u32 = 1416;
pub const ORD_OPERATOR_DELETE: u32 = 1094;
pub const ORD_OPERATOR_NEW: u32 = 1095;
pub const ORD_OPERATOR_NEW_ARRAY: u32 = 1456;
pub const ORD_OPERATOR_DELETE_ARRAY: u32 = 1457;
pub const ORD_OPERATOR_NEW_ARRAY_NOTHROW: u32 = 1661;
pub const ORD_OPERATOR_DELETE_ARRAY_NOTHROW: u32 = 1663;
pub const ORD_SWPRINTF: u32 = 1097;
pub const ORD_VSWPRINTF: u32 = 1099;
pub const ORD_SPRINTF: u32 = 719;
pub const ORD_VSPRINTF: u32 = 1146;
pub const ORD_SNPRINTF: u32 = 729;
pub const ORD_VSNPRINTF: u32 = 1147;
pub const ORD_SNWPRINTF: u32 = 1096;
pub const ORD_VSNWPRINTF: u32 = 1132;
pub const ORD_PRINTF: u32 = 1102;
pub const ORD_FGETS: u32 = 1109;
pub const ORD_FOPEN: u32 = 1113;
pub const ORD_WFOPEN: u32 = 1145;
pub const ORD_FCLOSE: u32 = 1118;
pub const ORD_FREAD: u32 = 1120;
pub const ORD_FWRITE: u32 = 1121;
pub const ORD_FFLUSH: u32 = 1122;
pub const ORD_FEOF: u32 = 1125;
pub const ORD_FERROR: u32 = 1126;
pub const ORD_FSEEK: u32 = 1130;
pub const ORD_FTELL: u32 = 1131;
pub const ORD_RAND: u32 = 1053;
pub const ORD_SRAND: u32 = 1061;
pub const ORD_SECURITY_GEN_COOKIE: u32 = 1875;
pub const ORD_SECURITY_GEN_COOKIE2: u32 = 2696;
pub const ORD_FREE: u32 = 1018;
pub const ORD_LONGJMP: u32 = 1036;
pub const ORD_SETJMP: u32 = 2000;
pub const ORD_INITIALIZE_CRITICAL_SECTION: u32 = 2;
pub const ORD_DELETE_CRITICAL_SECTION: u32 = 3;
pub const ORD_ENTER_CRITICAL_SECTION: u32 = 4;
pub const ORD_LEAVE_CRITICAL_SECTION: u32 = 5;
pub const ORD_EXIT_THREAD: u32 = 6;
pub const ORD_THREAD_EXCEPTION_EXIT: u32 = 1474;
pub const ORD_PSLNOTIFY: u32 = 7;
pub const ORD_INIT_LOCALE: u32 = 8;
pub const ORD_REINIT_LOCALE: u32 = 1799;
pub const ORD_IS_PROCESS_DYING: u32 = 1213;
pub const ORD_DIRECT_HANDLE_CALL: u32 = 2552;
pub const ORD_INTERLOCKED_INCREMENT: u32 = 10;
pub const ORD_INTERLOCKED_DECREMENT: u32 = 11;
pub const ORD_INTERLOCKED_EXCHANGE: u32 = 12;
pub const ORD_INTERLOCKED_EXCHANGE_ADD: u32 = 1491;
pub const ORD_INTERLOCKED_COMPARE_EXCHANGE: u32 = 1492;
pub const ORD_INTERLOCKED_TEST_EXCHANGE: u32 = 9;
pub const ORD_THREAD_BASE_FUNC: u32 = 13;
pub const ORD_MAIN_THREAD_BASE_FUNC: u32 = 14;
pub const ORD_COM_THREAD_BASE_FUNC: u32 = 1240;
pub const ORD_CREATE_LOCALE_VIEW: u32 = 1466;
pub const ORD_TLS_GET_VALUE: u32 = 15;
pub const ORD_TLS_SET_VALUE: u32 = 16;
pub const ORD_GET_VERSION_EX: u32 = 17;
pub const ORD_GET_VERSION_EX_W: u32 = 717;
pub const ORD_COMPARE_FILE_TIME: u32 = 18;
pub const ORD_SYSTEM_TIME_TO_FILE_TIME: u32 = 19;
pub const ORD_FILE_TIME_TO_SYSTEM_TIME: u32 = 20;
pub const ORD_FILE_TIME_TO_LOCAL_FILE_TIME: u32 = 21;
pub const ORD_LOCAL_FILE_TIME_TO_FILE_TIME: u32 = 22;
pub const ORD_GET_LOCAL_TIME: u32 = 23;
pub const ORD_SET_LOCAL_TIME: u32 = 24;
pub const ORD_GET_SYSTEM_TIME: u32 = 25;
pub const ORD_SET_SYSTEM_TIME: u32 = 26;
pub const ORD_GET_SYSTEM_TIME_AS_FILE_TIME: u32 = 2536;
pub const ORD_GET_TIME_ZONE_INFORMATION: u32 = 27;
pub const ORD_SET_TIME_ZONE_INFORMATION: u32 = 28;
pub const ORD_GET_CURRENT_FT: u32 = 29;
pub const ORD_IS_APIREADY: u32 = 30;
pub const ORD_WAIT_FOR_APIREADY: u32 = 2562;
pub const ORD_GET_APIADDRESS: u32 = 32;
pub const ORD_CE_EVENT_HAS_OCCURRED: u32 = 479;
pub const ORD_GET_CRTFLAGS: u32 = 1228;
pub const ORD_CE_ZERO_POINTER: u32 = 1781;
pub const ORD_FORCE_PIXEL_DOUBLING: u32 = 1829;
pub const ORD_IS_FORCE_PIXEL_DOUBLING: u32 = 1830;
pub const ORD_INITIALIZE_USR_COREDLL_CALLBACKS: u32 = 2561;
pub const ORD_GET_HANDLE_SERVER_ID: u32 = 2575;
pub const ORD_CE_SET_DIRECT_CALL: u32 = 2542;
pub const ORD_CE_GET_RAW_TIME: u32 = 2537;
pub const ORD_GET_USER_KDATA: u32 = 2528;
pub const ORD_CE_GET_RAW_TIME_OFFSET: u32 = 2529;
pub const ORD_CONVERT_THREAD_TO_FIBER: u32 = 1480;
pub const ORD_GET_CURRENT_FIBER: u32 = 1481;
pub const ORD_GET_FIBER_DATA: u32 = 1482;
pub const ORD_CREATE_FIBER: u32 = 1483;
pub const ORD_DELETE_FIBER: u32 = 1484;
pub const ORD_SWITCH_TO_FIBER: u32 = 1485;
pub const ORD_CE_REVERT_TO_SELF: u32 = 1926;
pub const ORD_CE_ACCESS_CHECK: u32 = 1927;
pub const ORD_CE_PRIVILEGE_CHECK: u32 = 1928;
pub const ORD_CE_CREATE_TOKEN_FROM_ACCOUNT: u32 = 1929;
pub const ORD_CE_CONVERT_STR_TO_SD: u32 = 1930;
pub const ORD_CE_CONVERT_SDTO_STR: u32 = 1931;
pub const ORD_CE_CREATE_TOKEN: u32 = 1932;
pub const ORD_CE_IMPERSONATE_TOKEN: u32 = 1933;
pub const ORD_CE_IMPERSONATE_CURRENT_PROCESS: u32 = 1935;
pub const ORD_CE_GET_OWNER_SID: u32 = 2567;
pub const ORD_CE_GET_GROUP_SID: u32 = 2568;
pub const ORD_CE_MODULE_JIT: u32 = 53;
pub const ORD_LOCAL_ALLOC: u32 = 33;
pub const ORD_LOCAL_ALLOC_TRACE: u32 = 2602;
pub const ORD_LOCAL_RE_ALLOC: u32 = 34;
pub const ORD_LOCAL_SIZE: u32 = 35;
pub const ORD_LOCAL_FREE: u32 = 36;
pub const ORD_REMOTE_HEAP_ALLOC: u32 = 1604;
pub const ORD_REMOTE_HEAP_RE_ALLOC: u32 = 1605;
pub const ORD_REMOTE_HEAP_FREE: u32 = 1606;
pub const ORD_REMOTE_HEAP_SIZE: u32 = 1607;
pub const ORD_REMOTE_LOCAL_ALLOC: u32 = 37;
pub const ORD_REMOTE_LOCAL_RE_ALLOC: u32 = 38;
pub const ORD_REMOTE_LOCAL_SIZE: u32 = 39;
pub const ORD_REMOTE_LOCAL_FREE: u32 = 40;
pub const ORD_LOCAL_ALLOC_IN_PROCESS: u32 = 41;
pub const ORD_LOCAL_FREE_IN_PROCESS: u32 = 42;
pub const ORD_LOCAL_SIZE_IN_PROCESS: u32 = 43;
pub const ORD_HEAP_CREATE: u32 = 44;
pub const ORD_HEAP_DESTROY: u32 = 45;
pub const ORD_HEAP_ALLOC: u32 = 46;
pub const ORD_HEAP_ALLOC_TRACE: u32 = 1999;
pub const ORD_HEAP_RE_ALLOC: u32 = 47;
pub const ORD_HEAP_SIZE: u32 = 48;
pub const ORD_HEAP_FREE: u32 = 49;
pub const ORD_GET_PROCESS_HEAP: u32 = 50;
pub const ORD_CE_HEAP_CREATE: u32 = 1836;
pub const ORD_HEAP_VALIDATE: u32 = 51;
pub const ORD_GET_HEAP_SNAPSHOT: u32 = 52;
pub const ORD_COMPACT_ALL_HEAPS: u32 = 54;
pub const ORD_HEAP_COMPACT: u32 = 1884;
pub const ORD_G_H_PROCESS_HEAP: u32 = 2543;
pub const ORD_CE_REMOTE_HEAP_CREATE: u32 = 2544;
pub const ORD_CE_REMOTE_HEAP_TRANSLATE_POINTER: u32 = 2545;
pub const ORD_CE_GET_MODULE_HEAP_INFO: u32 = 2713;
pub const ORD_HEAP_DUMP: u32 = 55;
pub const ORD_WSPRINTF_W: u32 = 56;
pub const ORD_WVSPRINTF_W: u32 = 57;
pub const ORD_RTL_DISPATCH_EXCEPTION: u32 = 2548;
pub const ORD_RTL_UNWIND_ONE_FRAME: u32 = 2549;
pub const ORD_ADD_VECTORED_EXCEPTION_HANDLER: u32 = 2546;
pub const ORD_REMOVE_VECTORED_EXCEPTION_HANDLER: u32 = 2547;
pub const ORD_STRING_CCH_COPY_W: u32 = 1689;
pub const ORD_STRING_CB_COPY_W: u32 = 1690;
pub const ORD_STRING_CCH_COPY_EX_W: u32 = 1691;
pub const ORD_STRING_CB_COPY_EX_W: u32 = 1692;
pub const ORD_STRING_CCH_COPY_NW: u32 = 1742;
pub const ORD_STRING_CB_COPY_NW: u32 = 1743;
pub const ORD_STRING_CCH_CAT_W: u32 = 1693;
pub const ORD_STRING_CB_CAT_W: u32 = 1694;
pub const ORD_STRING_CCH_CAT_EX_W: u32 = 1695;
pub const ORD_STRING_CB_CAT_EX_W: u32 = 1696;
pub const ORD_STRING_CCH_CAT_NW: u32 = 1744;
pub const ORD_STRING_CB_CAT_NW: u32 = 1745;
pub const ORD_STRING_CCH_CAT_NEX_W: u32 = 1746;
pub const ORD_STRING_CB_CAT_NEX_W: u32 = 1747;
pub const ORD_STRING_CCH_VPRINTF_W: u32 = 1697;
pub const ORD_STRING_CB_VPRINTF_W: u32 = 1698;
pub const ORD_STRING_CCH_PRINTF_W: u32 = 1699;
pub const ORD_STRING_CB_PRINTF_W: u32 = 1700;
pub const ORD_STRING_CCH_PRINTF_EX_W: u32 = 1701;
pub const ORD_STRING_CB_PRINTF_EX_W: u32 = 1702;
pub const ORD_STRING_CCH_VPRINTF_EX_W: u32 = 1703;
pub const ORD_STRING_CB_VPRINTF_EX_W: u32 = 1704;
pub const ORD_STRING_CCH_LENGTH_W: u32 = 1748;
pub const ORD_STRING_CB_LENGTH_W: u32 = 1749;
pub const ORD_STRING_CCH_COPY_NEX_W: u32 = 1868;
pub const ORD_STRING_CB_COPY_NEX_W: u32 = 1869;
pub const ORD_RANDOM: u32 = 80;
pub const ORD_DEBUG_BREAK: u32 = 81;
pub const ORD_PROFILE_START: u32 = 82;
pub const ORD_PROFILE_STOP: u32 = 83;
pub const ORD_PROFILE_CAPTURE_STATUS: u32 = 1800;
pub const ORD_PROFILE_START_EX: u32 = 1801;
pub const ORD_CE_LOG_DATA: u32 = 1451;
pub const ORD_CE_LOG_SET_ZONES: u32 = 1452;
pub const ORD_CE_LOG_GET_ZONES: u32 = 1681;
pub const ORD_CE_LOG_RE_SYNC: u32 = 1467;
pub const ORD_NPXNPHANDLER: u32 = 81;
pub const ORD_GLOBAL_MEMORY_STATUS: u32 = 88;
pub const ORD_SYSTEM_PARAMETERS_INFO_W: u32 = 89;
pub const ORD_CE_GET_RANDOM_SEED: u32 = 1443;
pub const ORD_CREATE_DIBSECTION: u32 = 90;
pub const ORD_EQUAL_RGN: u32 = 91;
pub const ORD_CREATE_ACCELERATOR_TABLE_W: u32 = 92;
pub const ORD_DESTROY_ACCELERATOR_TABLE: u32 = 93;
pub const ORD_LOAD_ACCELERATORS_W: u32 = 94;
pub const ORD_COPY_RECT: u32 = 96;
pub const ORD_EQUAL_RECT: u32 = 97;
pub const ORD_INFLATE_RECT: u32 = 98;
pub const ORD_INTERSECT_RECT: u32 = 99;
pub const ORD_IS_RECT_EMPTY: u32 = 100;
pub const ORD_OFFSET_RECT: u32 = 101;
pub const ORD_PT_IN_RECT: u32 = 102;
pub const ORD_SET_RECT: u32 = 103;
pub const ORD_SET_RECT_EMPTY: u32 = 104;
pub const ORD_SUBTRACT_RECT: u32 = 105;
pub const ORD_UNION_RECT: u32 = 106;
pub const ORD_CLEAR_COMM_BREAK: u32 = 107;
pub const ORD_CLEAR_COMM_ERROR: u32 = 108;
pub const ORD_ESCAPE_COMM_FUNCTION: u32 = 109;
pub const ORD_GET_COMM_MASK: u32 = 110;
pub const ORD_GET_COMM_MODEM_STATUS: u32 = 111;
pub const ORD_GET_COMM_PROPERTIES: u32 = 112;
pub const ORD_GET_COMM_STATE: u32 = 113;
pub const ORD_GET_COMM_TIMEOUTS: u32 = 114;
pub const ORD_PURGE_COMM: u32 = 115;
pub const ORD_SET_COMM_BREAK: u32 = 116;
pub const ORD_SET_COMM_MASK: u32 = 117;
pub const ORD_SET_COMM_STATE: u32 = 118;
pub const ORD_SET_COMM_TIMEOUTS: u32 = 119;
pub const ORD_SETUP_COMM: u32 = 120;
pub const ORD_TRANSMIT_COMM_CHAR: u32 = 121;
pub const ORD_WAIT_COMM_EVENT: u32 = 122;
pub const ORD_ENUM_PNP_IDS: u32 = 123;
pub const ORD_ENUM_DEVICES: u32 = 124;
pub const ORD_GET_DEVICE_KEYS: u32 = 125;
pub const ORD_OPEN_DEVICE_KEY: u32 = 1396;
pub const ORD_DDKREG_GET_WINDOW_INFO: u32 = 1668;
pub const ORD_DDKREG_GET_ISR_INFO: u32 = 1669;
pub const ORD_DDKREG_GET_PCI_INFO: u32 = 1670;
pub const ORD_CRYPT_ACQUIRE_CONTEXT_W: u32 = 126;
pub const ORD_CRYPT_RELEASE_CONTEXT: u32 = 127;
pub const ORD_CRYPT_GEN_KEY: u32 = 128;
pub const ORD_CRYPT_DERIVE_KEY: u32 = 129;
pub const ORD_CRYPT_DESTROY_KEY: u32 = 130;
pub const ORD_CRYPT_SET_KEY_PARAM: u32 = 131;
pub const ORD_CRYPT_GET_KEY_PARAM: u32 = 132;
pub const ORD_CRYPT_EXPORT_KEY: u32 = 133;
pub const ORD_CRYPT_IMPORT_KEY: u32 = 134;
pub const ORD_CRYPT_ENCRYPT: u32 = 135;
pub const ORD_CRYPT_DECRYPT: u32 = 136;
pub const ORD_CRYPT_CREATE_HASH: u32 = 137;
pub const ORD_CRYPT_HASH_SESSION_KEY: u32 = 138;
pub const ORD_CRYPT_HASH_DATA: u32 = 139;
pub const ORD_CRYPT_DESTROY_HASH: u32 = 140;
pub const ORD_CRYPT_SIGN_HASH_W: u32 = 141;
pub const ORD_CRYPT_VERIFY_SIGNATURE_W: u32 = 142;
pub const ORD_CRYPT_GEN_RANDOM: u32 = 143;
pub const ORD_CRYPT_GET_USER_KEY: u32 = 144;
pub const ORD_CRYPT_SET_PROVIDER_W: u32 = 145;
pub const ORD_CRYPT_GET_HASH_PARAM: u32 = 146;
pub const ORD_CRYPT_SET_HASH_PARAM: u32 = 147;
pub const ORD_CRYPT_GET_PROV_PARAM: u32 = 148;
pub const ORD_CRYPT_SET_PROV_PARAM: u32 = 149;
pub const ORD_CRYPT_SET_PROVIDER_EX_W: u32 = 150;
pub const ORD_CRYPT_GET_DEFAULT_PROVIDER_W: u32 = 151;
pub const ORD_CRYPT_ENUM_PROVIDER_TYPES_W: u32 = 152;
pub const ORD_CRYPT_ENUM_PROVIDERS_W: u32 = 153;
pub const ORD_CRYPT_CONTEXT_ADD_REF: u32 = 154;
pub const ORD_CRYPT_DUPLICATE_KEY: u32 = 155;
pub const ORD_CRYPT_DUPLICATE_HASH: u32 = 156;
pub const ORD_IS_ENCRYPTION_PERMITTED: u32 = 613;
pub const ORD_ATTACH_DEBUGGER: u32 = 157;
pub const ORD_ATTACH_HDSTUB: u32 = 1955;
pub const ORD_ATTACH_OS_AXS_T0: u32 = 1956;
pub const ORD_ATTACH_OS_AXS_T1: u32 = 1982;
pub const ORD_CAPTURE_DUMP_FILE_ON_DEVICE: u32 = 1960;
pub const ORD_REPORT_FAULT: u32 = 1964;
pub const ORD_SET_INTERRUPT_EVENT: u32 = 158;
pub const ORD_CE_SET_POWER_ON_EVENT: u32 = 1688;
pub const ORD_IS_EXITING: u32 = 159;
pub const ORD_CREATE_DIRECTORY_W: u32 = 160;
pub const ORD_REMOVE_DIRECTORY_W: u32 = 161;
pub const ORD_GET_TEMP_PATH_W: u32 = 162;
pub const ORD_MOVE_FILE_W: u32 = 163;
pub const ORD_COPY_FILE_W: u32 = 164;
pub const ORD_COPY_FILE_EX_W: u32 = 1958;
pub const ORD_DELETE_FILE_W: u32 = 165;
pub const ORD_GET_FILE_ATTRIBUTES_W: u32 = 166;
pub const ORD_FIND_FIRST_FILE_W: u32 = 167;
pub const ORD_FIND_FIRST_FILE_EX_W: u32 = 1235;
pub const ORD_CREATE_FILE_W: u32 = 168;
pub const ORD_SET_FILE_ATTRIBUTES_W: u32 = 169;
pub const ORD_READ_FILE: u32 = 170;
pub const ORD_WRITE_FILE: u32 = 171;
pub const ORD_GET_FILE_SIZE: u32 = 172;
pub const ORD_SET_FILE_POINTER: u32 = 173;
pub const ORD_GET_FILE_INFORMATION_BY_HANDLE: u32 = 174;
pub const ORD_FLUSH_FILE_BUFFERS: u32 = 175;
pub const ORD_GET_FILE_TIME: u32 = 176;
pub const ORD_SET_FILE_TIME: u32 = 177;
pub const ORD_SET_END_OF_FILE: u32 = 178;
pub const ORD_FIND_CLOSE: u32 = 180;
pub const ORD_FIND_NEXT_FILE_W: u32 = 181;
pub const ORD_DELETE_AND_RENAME_FILE: u32 = 183;
pub const ORD_GET_DISK_FREE_SPACE_EX_W: u32 = 184;
pub const ORD_GET_FILE_ATTRIBUTES_EX_W: u32 = 1237;
pub const ORD_GET_STORE_INFORMATION: u32 = 323;
pub const ORD_PEG_OID_GET_INFO: u32 = 301;
pub const ORD_CE_OID_GET_INFO: u32 = 312;
pub const ORD_CE_OID_GET_INFO_EX: u32 = 1195;
pub const ORD_CE_OID_GET_INFO_EX2: u32 = 1472;
pub const ORD_FIND_FIRST_CHANGE_NOTIFICATION_W: u32 = 1682;
pub const ORD_FIND_NEXT_CHANGE_NOTIFICATION: u32 = 1683;
pub const ORD_FIND_CLOSE_CHANGE_NOTIFICATION: u32 = 1684;
pub const ORD_CE_GET_FILE_NOTIFICATION_INFO: u32 = 1798;
pub const ORD_READ_FILE_SCATTER: u32 = 1831;
pub const ORD_WRITE_FILE_GATHER: u32 = 1832;
pub const ORD_CE_FS_IO_CONTROL_W: u32 = 1965;
pub const ORD_LOCK_FILE_EX: u32 = 1968;
pub const ORD_UNLOCK_FILE_EX: u32 = 1969;
pub const ORD_CE_GET_VOLUME_INFO_W: u32 = 1978;
pub const ORD_CE_CERT_VERIFY: u32 = 2509;
pub const ORD_CE_OPEN_MODULE_BY_POLICY: u32 = 2699;
pub const ORD_CE_POLICY_CHECK: u32 = 2700;
pub const ORD_CE_OPEN_POLICY: u32 = 2701;
pub const ORD_CE_POLICY_CHECK_BY_HANDLE: u32 = 2702;
pub const ORD_CE_GET_POLICY_INFO: u32 = 2703;
pub const ORD_CE_CLOSE_POLICY: u32 = 2704;
pub const ORD_SET_FILE_SECURITY_W: u32 = 2711;
pub const ORD_GET_FILE_SECURITY_W: u32 = 2712;
pub const ORD_OPEN_STORE: u32 = 2577;
pub const ORD_DISMOUNT_STORE: u32 = 2578;
pub const ORD_FORMAT_STORE: u32 = 2579;
pub const ORD_FIND_FIRST_STORE: u32 = 2580;
pub const ORD_FIND_NEXT_STORE: u32 = 2581;
pub const ORD_FIND_CLOSE_STORE: u32 = 2582;
pub const ORD_GET_STORE_INFO: u32 = 2583;
pub const ORD_CREATE_PARTITION: u32 = 2584;
pub const ORD_CREATE_PARTITION_EX: u32 = 2585;
pub const ORD_DELETE_PARTITION: u32 = 2586;
pub const ORD_OPEN_PARTITION: u32 = 2587;
pub const ORD_MOUNT_PARTITION: u32 = 2588;
pub const ORD_DISMOUNT_PARTITION: u32 = 2589;
pub const ORD_RENAME_PARTITION: u32 = 2590;
pub const ORD_SET_PARTITION_ATTRIBUTES: u32 = 2591;
pub const ORD_GET_PARTITION_INFO: u32 = 2592;
pub const ORD_FORMAT_PARTITION: u32 = 2593;
pub const ORD_FORMAT_PARTITION_EX: u32 = 2594;
pub const ORD_FIND_FIRST_PARTITION: u32 = 2595;
pub const ORD_FIND_NEXT_PARTITION: u32 = 2596;
pub const ORD_FIND_CLOSE_PARTITION: u32 = 2597;
pub const ORD_CHECK_PASSWORD: u32 = 182;
pub const ORD_SET_PASSWORD: u32 = 238;
pub const ORD_GET_PASSWORD_ACTIVE: u32 = 239;
pub const ORD_SET_PASSWORD_ACTIVE: u32 = 240;
pub const ORD_SET_PASSWORD_STATUS: u32 = 1537;
pub const ORD_GET_PASSWORD_STATUS: u32 = 1538;
pub const ORD_CREATE_MSG_QUEUE: u32 = 1529;
pub const ORD_READ_MSG_QUEUE: u32 = 1530;
pub const ORD_READ_MSG_QUEUE_EX: u32 = 2538;
pub const ORD_WRITE_MSG_QUEUE: u32 = 1531;
pub const ORD_GET_MSG_QUEUE_INFO: u32 = 1532;
pub const ORD_CLOSE_MSG_QUEUE: u32 = 1533;
pub const ORD_OPEN_MSG_QUEUE: u32 = 1536;
pub const ORD_IS_VALID_CODE_PAGE: u32 = 185;
pub const ORD_GET_ACP: u32 = 186;
pub const ORD_GET_OEMCP: u32 = 187;
pub const ORD_GET_CPINFO: u32 = 188;
pub const ORD_SET_ACP: u32 = 189;
pub const ORD_SET_OEMCP: u32 = 190;
pub const ORD_IS_DBCSLEAD_BYTE: u32 = 191;
pub const ORD_IS_DBCSLEAD_BYTE_EX: u32 = 192;
pub const ORD_ISWCTYPE: u32 = 193;
pub const ORD_MULTI_BYTE_TO_WIDE_CHAR: u32 = 196;
pub const ORD_WIDE_CHAR_TO_MULTI_BYTE: u32 = 197;
pub const ORD_COMPARE_STRING_W: u32 = 198;
pub const ORD_LCMAP_STRING_W: u32 = 199;
pub const ORD_GET_LOCALE_INFO_W: u32 = 200;
pub const ORD_SET_LOCALE_INFO_W: u32 = 201;
pub const ORD_GET_TIME_FORMAT_W: u32 = 202;
pub const ORD_GET_DATE_FORMAT_W: u32 = 203;
pub const ORD_GET_NUMBER_FORMAT_W: u32 = 204;
pub const ORD_GET_CURRENCY_FORMAT_W: u32 = 205;
pub const ORD_ENUM_CALENDAR_INFO_W: u32 = 206;
pub const ORD_ENUM_TIME_FORMATS_W: u32 = 207;
pub const ORD_ENUM_DATE_FORMATS_W: u32 = 208;
pub const ORD_IS_VALID_LOCALE: u32 = 209;
pub const ORD_CONVERT_DEFAULT_LOCALE: u32 = 210;
pub const ORD_GET_SYSTEM_DEFAULT_LANG_ID: u32 = 211;
pub const ORD_GET_USER_DEFAULT_LANG_ID: u32 = 212;
pub const ORD_GET_SYSTEM_DEFAULT_LCID: u32 = 213;
pub const ORD_SET_SYSTEM_DEFAULT_LCID: u32 = 214;
pub const ORD_GET_USER_DEFAULT_LCID: u32 = 215;
pub const ORD_SET_USER_DEFAULT_LCID: u32 = 1795;
pub const ORD_GET_STRING_TYPE_W: u32 = 216;
pub const ORD_GET_STRING_TYPE_EX_W: u32 = 217;
pub const ORD_FOLD_STRING_W: u32 = 218;
pub const ORD_ENUM_SYSTEM_LOCALES_W: u32 = 219;
pub const ORD_ENUM_SYSTEM_CODE_PAGES_W: u32 = 220;
pub const ORD_CHAR_LOWER_W: u32 = 221;
pub const ORD_CHAR_LOWER_BUFF_W: u32 = 222;
pub const ORD_CHAR_UPPER_BUFF_W: u32 = 223;
pub const ORD_CHAR_UPPER_W: u32 = 224;
pub const ORD_CHAR_PREV_W: u32 = 225;
pub const ORD_CHAR_NEXT_W: u32 = 226;
pub const ORD_LSTRCMP_W: u32 = 227;
pub const ORD_LSTRCMPI_W: u32 = 228;
pub const ORD_DBCANONICALIZE: u32 = 233;
pub const ORD_GET_NLS_TABLES: u32 = 1886;
pub const ORD_GET_SYSTEM_DEFAULT_UILANGUAGE: u32 = 1317;
pub const ORD_GET_USER_DEFAULT_UILANGUAGE: u32 = 1318;
pub const ORD_SET_USER_DEFAULT_UILANGUAGE: u32 = 1319;
pub const ORD_ENUM_UILANGUAGES_W: u32 = 1320;
pub const ORD_FORMAT_MESSAGE_W: u32 = 234;
pub const ORD_REGISTER_DEVICE: u32 = 235;
pub const ORD_DEREGISTER_DEVICE: u32 = 236;
pub const ORD_LOAD_FSD: u32 = 237;
pub const ORD_LOAD_FSDEX: u32 = 1421;
pub const ORD_ACTIVATE_DEVICE: u32 = 1179;
pub const ORD_ACTIVATE_DEVICE_EX: u32 = 1494;
pub const ORD_DEACTIVATE_DEVICE: u32 = 1180;
pub const ORD_GET_DEVICE_HANDLE_FROM_CONTEXT: u32 = 1961;
pub const ORD_REG_OPEN_PROCESS_KEY: u32 = 1542;
pub const ORD_CE_RESYNC_FILESYS: u32 = 1425;
pub const ORD_GET_DEVICE_INFORMATION_BY_DEVICE_HANDLE: u32 = 1870;
pub const ORD_GET_DEVICE_INFORMATION_BY_FILE_HANDLE: u32 = 1871;
pub const ORD_FIND_FIRST_DEVICE: u32 = 1872;
pub const ORD_FIND_NEXT_DEVICE: u32 = 1873;
pub const ORD_ENUM_DEVICE_INTERFACES: u32 = 1874;
pub const ORD_REL_UDRIVER_PROC_IO_CONTROL: u32 = 2576;
pub const ORD_RESOURCE_CREATE_LIST: u32 = 1612;
pub const ORD_RESOURCE_REQUEST: u32 = 1613;
pub const ORD_RESOURCE_RELEASE: u32 = 1614;
pub const ORD_RESOURCE_REQUEST_EX: u32 = 1833;
pub const ORD_RESOURCE_MARK_AS_SHAREABLE: u32 = 1834;
pub const ORD_RESOURCE_DESTROY_LIST: u32 = 1835;
pub const ORD_GET_SYSTEM_POWER_STATE: u32 = 1581;
pub const ORD_SET_SYSTEM_POWER_STATE: u32 = 1582;
pub const ORD_SET_POWER_REQUIREMENT: u32 = 1583;
pub const ORD_RELEASE_POWER_REQUIREMENT: u32 = 1584;
pub const ORD_REQUEST_POWER_NOTIFICATIONS: u32 = 1585;
pub const ORD_STOP_POWER_NOTIFICATIONS: u32 = 1586;
pub const ORD_DEVICE_POWER_NOTIFY: u32 = 1588;
pub const ORD_REGISTER_POWER_RELATIONSHIP: u32 = 1609;
pub const ORD_RELEASE_POWER_RELATIONSHIP: u32 = 1610;
pub const ORD_SET_DEVICE_POWER: u32 = 1678;
pub const ORD_GET_DEVICE_POWER: u32 = 1679;
pub const ORD_POWER_POLICY_NOTIFY: u32 = 1764;
pub const ORD_ACTIVATE_SERVICE: u32 = 1508;
pub const ORD_REGISTER_SERVICE: u32 = 1509;
pub const ORD_DEREGISTER_SERVICE: u32 = 1510;
pub const ORD_CLOSE_ALL_SERVICE_HANDLES: u32 = 1511;
pub const ORD_CREATE_SERVICE_HANDLE: u32 = 1512;
pub const ORD_GET_SERVICE_BY_INDEX: u32 = 1513;
pub const ORD_SERVICE_IO_CONTROL: u32 = 1514;
pub const ORD_SERVICE_ADD_PORT: u32 = 1515;
pub const ORD_SERVICE_UNBIND_PORTS: u32 = 1516;
pub const ORD_ENUM_SERVICES: u32 = 1517;
pub const ORD_GET_SERVICE_HANDLE: u32 = 1518;
pub const ORD_SERVICE_CLOSE_PORT: u32 = 1759;
pub const ORD_SIGNAL_STARTED: u32 = 639;
pub const ORD_CE_REGISTER_FILE_SYSTEM_NOTIFICATION: u32 = 331;
pub const ORD_REGISTER_AFSEX: u32 = 1490;
pub const ORD_DEREGISTER_AFS: u32 = 335;
pub const ORD_REGISTER_AFSNAME: u32 = 338;
pub const ORD_DEREGISTER_AFSNAME: u32 = 339;
pub const ORD_GET_SYSTEM_MEMORY_DIVISION: u32 = 336;
pub const ORD_SET_SYSTEM_MEMORY_DIVISION: u32 = 337;
pub const ORD_DUMP_FILE_SYSTEM_HEAP: u32 = 341;
pub const ORD_FILE_SYSTEM_POWER_FUNCTION: u32 = 241;
pub const ORD_CLOSE_ALL_FILE_HANDLES: u32 = 242;
pub const ORD_READ_FILE_WITH_SEEK: u32 = 243;
pub const ORD_WRITE_FILE_WITH_SEEK: u32 = 718;
pub const ORD_IS_SYSTEM_FILE: u32 = 1680;
pub const ORD_REQUEST_DEVICE_NOTIFICATIONS: u32 = 1504;
pub const ORD_STOP_DEVICE_NOTIFICATIONS: u32 = 1505;
pub const ORD_ADVERTISE_INTERFACE: u32 = 1687;
pub const ORD_GET_DEVICE_BY_INDEX: u32 = 1236;
pub const ORD_USER_CALL_WINDOW_PROC: u32 = 2867;
pub const ORD_CREATE_WINDOW_EX_W: u32 = 246;
pub const ORD_SET_WINDOW_POS: u32 = 247;
pub const ORD_GET_WINDOW_RECT: u32 = 248;
pub const ORD_GET_CLIENT_RECT: u32 = 249;
pub const ORD_INVALIDATE_RECT: u32 = 250;
pub const ORD_GET_WINDOW: u32 = 251;
pub const ORD_WINDOW_FROM_POINT: u32 = 252;
pub const ORD_CHILD_WINDOW_FROM_POINT: u32 = 253;
pub const ORD_CLIENT_TO_SCREEN: u32 = 254;
pub const ORD_SCREEN_TO_CLIENT: u32 = 255;
pub const ORD_SET_WINDOW_TEXT_W: u32 = 256;
pub const ORD_GET_WINDOW_TEXT_W: u32 = 257;
pub const ORD_SET_WINDOW_LONG_W: u32 = 258;
pub const ORD_GET_WINDOW_LONG_W: u32 = 259;
pub const ORD_BEGIN_PAINT: u32 = 260;
pub const ORD_END_PAINT: u32 = 261;
pub const ORD_GET_DCEX: u32 = 1185;
pub const ORD_DEF_WINDOW_PROC_W: u32 = 264;
pub const ORD_DESTROY_WINDOW: u32 = 265;
pub const ORD_SHOW_WINDOW: u32 = 266;
pub const ORD_UPDATE_WINDOW: u32 = 267;
pub const ORD_SET_PARENT: u32 = 268;
pub const ORD_GET_PARENT: u32 = 269;
pub const ORD_IS_WINDOW: u32 = 271;
pub const ORD_MOVE_WINDOW: u32 = 272;
pub const ORD_GET_UPDATE_RGN: u32 = 273;
pub const ORD_GET_UPDATE_RECT: u32 = 274;
pub const ORD_BRING_WINDOW_TO_TOP: u32 = 275;
pub const ORD_GET_WINDOW_TEXT_LENGTH_W: u32 = 276;
pub const ORD_IS_CHILD: u32 = 277;
pub const ORD_VALIDATE_RECT: u32 = 278;
pub const ORD_GET_CLASS_NAME_W: u32 = 283;
pub const ORD_MAP_WINDOW_POINTS: u32 = 284;
pub const ORD_CALL_WINDOW_PROC_W: u32 = 285;
pub const ORD_FIND_WINDOW_W: u32 = 286;
pub const ORD_ENABLE_WINDOW: u32 = 287;
pub const ORD_IS_WINDOW_ENABLED: u32 = 288;
pub const ORD_SCROLL_WINDOW_EX: u32 = 289;
pub const ORD_POST_THREAD_MESSAGE_W: u32 = 290;
pub const ORD_ENUM_WINDOWS: u32 = 291;
pub const ORD_GET_WINDOW_THREAD_PROCESS_ID: u32 = 292;
pub const ORD_BEGIN_DEFER_WINDOW_POS: u32 = 1157;
pub const ORD_DEFER_WINDOW_POS: u32 = 1158;
pub const ORD_END_DEFER_WINDOW_POS: u32 = 1159;
pub const ORD_GET_DESKTOP_WINDOW: u32 = 1397;
pub const ORD_SET_WINDOW_RGN: u32 = 1398;
pub const ORD_GET_WINDOW_RGN: u32 = 1399;
pub const ORD_GET_WINDOW_TEXT_WDIRECT: u32 = 1454;
pub const ORD_ACCESSIBILITY_SOUND_SENTRY_EVENT: u32 = 1540;
pub const ORD_CHANGE_DISPLAY_SETTINGS_EX: u32 = 1611;
pub const ORD_INVALIDATE_RGN: u32 = 1615;
pub const ORD_VALIDATE_RGN: u32 = 1616;
pub const ORD_REDRAW_WINDOW: u32 = 1672;
pub const ORD_REGISTER_SIPANEL: u32 = 293;
pub const ORD_RECTANGLE_ANIMATION: u32 = 294;
pub const ORD_GWES_POWER_OFF_SYSTEM: u32 = 296;
pub const ORD_SET_ASSOCIATED_MENU: u32 = 299;
pub const ORD_GET_ASSOCIATED_MENU: u32 = 300;
pub const ORD_GWES_POWER_DOWN: u32 = 1722;
pub const ORD_GWES_POWER_UP: u32 = 1723;
pub const ORD_SHOW_STARTUP_WINDOW: u32 = 1810;
pub const ORD_GET_GWE_API_SET_TABLES: u32 = 1867;
pub const ORD_IMM_DLL_ENTRY: u32 = 2598;
pub const ORD_ANIMATE_RECTS: u32 = 2707;
pub const ORD_CE_CALL_USER_PROC: u32 = 2606;
pub const ORD_PEG_FIND_FIRST_DATABASE: u32 = 302;
pub const ORD_PEG_FIND_NEXT_DATABASE: u32 = 303;
pub const ORD_PEG_CREATE_DATABASE: u32 = 304;
pub const ORD_PEG_SET_DATABASE_INFO: u32 = 305;
pub const ORD_PEG_OPEN_DATABASE: u32 = 306;
pub const ORD_PEG_DELETE_DATABASE: u32 = 307;
pub const ORD_PEG_SEEK_DATABASE: u32 = 308;
pub const ORD_PEG_DELETE_RECORD: u32 = 309;
pub const ORD_PEG_READ_RECORD_PROPS: u32 = 310;
pub const ORD_PEG_WRITE_RECORD_PROPS: u32 = 311;
pub const ORD_CE_FIND_FIRST_DATABASE: u32 = 313;
pub const ORD_CE_FIND_NEXT_DATABASE: u32 = 314;
pub const ORD_CE_CREATE_DATABASE: u32 = 315;
pub const ORD_CE_CREATE_DATABASE_EX: u32 = 1190;
pub const ORD_CE_SET_DATABASE_INFO: u32 = 316;
pub const ORD_CE_SET_DATABASE_INFO_EX: u32 = 1191;
pub const ORD_CE_OPEN_DATABASE: u32 = 317;
pub const ORD_CE_OPEN_DATABASE_EX: u32 = 1192;
pub const ORD_CE_DELETE_DATABASE: u32 = 318;
pub const ORD_CE_READ_RECORD_PROPS: u32 = 321;
pub const ORD_CE_SEEK_DATABASE: u32 = 319;
pub const ORD_CE_FIND_FIRST_DATABASE_EX: u32 = 1196;
pub const ORD_CE_FIND_NEXT_DATABASE_EX: u32 = 1189;
pub const ORD_CE_CREATE_DATABASE_EX2: u32 = 1468;
pub const ORD_CE_SET_DATABASE_INFO_EX2: u32 = 1471;
pub const ORD_CE_OPEN_DATABASE_EX2: u32 = 1469;
pub const ORD_CE_DELETE_DATABASE_EX: u32 = 1193;
pub const ORD_CE_SEEK_DATABASE_EX: u32 = 1470;
pub const ORD_CE_DELETE_RECORD: u32 = 320;
pub const ORD_CE_READ_RECORD_PROPS_EX: u32 = 1194;
pub const ORD_CE_MOUNT_DBVOL: u32 = 1164;
pub const ORD_CE_ENUM_DBVOLUMES: u32 = 1165;
pub const ORD_CE_WRITE_RECORD_PROPS: u32 = 322;
pub const ORD_CE_UNMOUNT_DBVOL: u32 = 1197;
pub const ORD_CE_FLUSH_DBVOL: u32 = 1217;
pub const ORD_CE_GET_DBINFORMATION_BY_HANDLE: u32 = 1473;
pub const ORD_CE_FREE_NOTIFICATION: u32 = 1226;
pub const ORD_CE_CHANGE_DATABASE_LCID: u32 = 340;
pub const ORD_CE_ADD_DATABASE_PROPS: u32 = 1892;
pub const ORD_CE_ADD_SYNC_PARTNER: u32 = 1893;
pub const ORD_CE_ATTACH_CUSTOM_TRACKING_DATA: u32 = 1894;
pub const ORD_CE_BEGIN_SYNC_SESSION: u32 = 1895;
pub const ORD_CE_BEGIN_TRANSACTION: u32 = 1896;
pub const ORD_CE_CREATE_DATABASE_WITH_PROPS: u32 = 1897;
pub const ORD_CE_CREATE_SESSION: u32 = 1898;
pub const ORD_CE_END_SYNC_SESSION: u32 = 1899;
pub const ORD_CE_END_TRANSACTION: u32 = 1900;
pub const ORD_CE_FIND_NEXT_CHANGED_RECORD: u32 = 1901;
pub const ORD_CE_GET_CHANGED_RECORD_CNT: u32 = 1902;
pub const ORD_CE_GET_CHANGED_RECORDS: u32 = 1903;
pub const ORD_CE_GET_CUSTOM_TRACKING_DATA: u32 = 1904;
pub const ORD_CE_GET_DATABASE_PROPS: u32 = 1905;
pub const ORD_CE_GET_DATABASE_SESSION: u32 = 1906;
pub const ORD_CE_GET_PROP_CHANGE_INFO: u32 = 1907;
pub const ORD_CE_GET_RECORD_CHANGE_INFO: u32 = 1908;
pub const ORD_CE_MARK_RECORD: u32 = 1909;
pub const ORD_CE_MOUNT_DBVOL_EX: u32 = 1910;
pub const ORD_CE_OPEN_DATABASE_IN_SESSION: u32 = 1911;
pub const ORD_CE_OPEN_STREAM: u32 = 1912;
pub const ORD_CE_PURGE_TRACKING_DATA: u32 = 1913;
pub const ORD_CE_PURGE_TRACKING_GENERATIONS: u32 = 1914;
pub const ORD_CE_REMOVE_DATABASE_PROPS: u32 = 1915;
pub const ORD_CE_REMOVE_DATABASE_TRACKING: u32 = 1916;
pub const ORD_CE_REMOVE_SYNC_PARTNER: u32 = 1917;
pub const ORD_CE_SET_SESSION_OPTION: u32 = 1918;
pub const ORD_CE_STREAM_READ: u32 = 1919;
pub const ORD_CE_STREAM_SAVE_CHANGES: u32 = 1920;
pub const ORD_CE_STREAM_SEEK: u32 = 1921;
pub const ORD_CE_STREAM_SET_SIZE: u32 = 1922;
pub const ORD_CE_STREAM_WRITE: u32 = 1923;
pub const ORD_CE_TRACK_DATABASE: u32 = 1924;
pub const ORD_CE_TRACK_PROPERTY: u32 = 1925;
pub const ORD_CE_GET_REPL_CHANGE_MASK: u32 = 324;
pub const ORD_CE_SET_REPL_CHANGE_MASK: u32 = 325;
pub const ORD_CE_GET_REPL_CHANGE_BITS_EX: u32 = 326;
pub const ORD_CE_SET_REPL_CHANGE_BITS_EX: u32 = 327;
pub const ORD_CE_CLEAR_REPL_CHANGE_BITS_EX: u32 = 328;
pub const ORD_CE_GET_REPL_OTHER_BITS_EX: u32 = 329;
pub const ORD_CE_SET_REPL_OTHER_BITS_EX: u32 = 330;
pub const ORD_CE_REGISTER_REPL_NOTIFICATION: u32 = 332;
pub const ORD_REPL_OPEN_SYNC: u32 = 1543;
pub const ORD_REPL_CHECKPOINT: u32 = 1544;
pub const ORD_REPL_CLOSE_SYNC: u32 = 1545;
pub const ORD_REPL_GET_SYNC_STATE: u32 = 1546;
pub const ORD_REPL_CHANGE_SYNC_SETTINGS: u32 = 1547;
pub const ORD_REPL_FIND_NEXT_CHANGE: u32 = 1548;
pub const ORD_REPL_GET_OID_STATUS: u32 = 1549;
pub const ORD_RAS_DIAL: u32 = 342;
pub const ORD_RAS_HANGUP: u32 = 343;
pub const ORD_RAS_HANG_UP: u32 = 344;
pub const ORD_RAS_ENUM_ENTRIES: u32 = 345;
pub const ORD_RAS_GET_ENTRY_DIAL_PARAMS: u32 = 346;
pub const ORD_RAS_SET_ENTRY_DIAL_PARAMS: u32 = 347;
pub const ORD_RAS_GET_ENTRY_PROPERTIES: u32 = 348;
pub const ORD_RAS_SET_ENTRY_PROPERTIES: u32 = 349;
pub const ORD_RAS_VALIDATE_ENTRY_NAME: u32 = 350;
pub const ORD_RAS_DELETE_ENTRY: u32 = 351;
pub const ORD_RAS_RENAME_ENTRY: u32 = 352;
pub const ORD_RAS_ENUM_CONNECTIONS: u32 = 353;
pub const ORD_RAS_GET_CONNECT_STATUS: u32 = 354;
pub const ORD_RAS_GET_ENTRY_DEV_CONFIG: u32 = 355;
pub const ORD_RAS_SET_ENTRY_DEV_CONFIG: u32 = 356;
pub const ORD_RAS_IOCONTROL: u32 = 357;
pub const ORD_RAS_ENUM_DEVICES_W: u32 = 1424;
pub const ORD_RAS_GET_PROJECTION_INFO_W: u32 = 1458;
pub const ORD_RAS_GET_LINK_STATISTICS: u32 = 1463;
pub const ORD_RAS_GET_DISP_PHONE_NUM_W: u32 = 1464;
pub const ORD_RAS_DEV_CONFIG_DIALOG_EDIT_W: u32 = 1465;
pub const ORD_RAS_GET_EAP_USER_DATA: u32 = 1673;
pub const ORD_RAS_SET_EAP_USER_DATA: u32 = 1674;
pub const ORD_RAS_GET_EAP_CONNECTION_DATA: u32 = 1675;
pub const ORD_RAS_SET_EAP_CONNECTION_DATA: u32 = 1676;
pub const ORD_AFDSOCKET: u32 = 2612;
pub const ORD_AFDSELECT: u32 = 2613;
pub const ORD_LINE_CLOSE: u32 = 358;
pub const ORD_LINE_DEALLOCATE_CALL: u32 = 359;
pub const ORD_LINE_DROP: u32 = 360;
pub const ORD_LINE_GET_DEV_CAPS: u32 = 361;
pub const ORD_LINE_GET_DEV_CONFIG: u32 = 362;
pub const ORD_LINE_GET_TRANSLATE_CAPS: u32 = 363;
pub const ORD_LINE_INITIALIZE: u32 = 364;
pub const ORD_LINE_MAKE_CALL: u32 = 365;
pub const ORD_LINE_NEGOTIATE_APIVERSION: u32 = 366;
pub const ORD_LINE_OPEN: u32 = 367;
pub const ORD_LINE_SET_DEV_CONFIG: u32 = 368;
pub const ORD_LINE_SET_STATUS_MESSAGES: u32 = 369;
pub const ORD_LINE_SHUTDOWN: u32 = 370;
pub const ORD_LINE_TRANSLATE_ADDRESS: u32 = 371;
pub const ORD_LINE_GET_ID: u32 = 372;
pub const ORD_LINE_TRANSLATE_DIALOG: u32 = 373;
pub const ORD_LINE_CONFIG_DIALOG_EDIT: u32 = 374;
pub const ORD_LINE_ADD_PROVIDER: u32 = 375;
pub const ORD_LINE_SET_CURRENT_LOCATION: u32 = 1168;
pub const ORD_LINE_ACCEPT: u32 = 1246;
pub const ORD_LINE_ADD_TO_CONFERENCE: u32 = 1247;
pub const ORD_LINE_ANSWER: u32 = 1248;
pub const ORD_LINE_BLIND_TRANSFER: u32 = 1249;
pub const ORD_LINE_COMPLETE_TRANSFER: u32 = 1250;
pub const ORD_LINE_DEV_SPECIFIC: u32 = 1251;
pub const ORD_LINE_DIAL: u32 = 1252;
pub const ORD_LINE_FORWARD: u32 = 1253;
pub const ORD_LINE_GENERATE_DIGITS: u32 = 1254;
pub const ORD_LINE_GENERATE_TONE: u32 = 1255;
pub const ORD_LINE_GET_ADDRESS_CAPS: u32 = 1256;
pub const ORD_LINE_GET_ADDRESS_ID: u32 = 1257;
pub const ORD_LINE_GET_ADDRESS_STATUS: u32 = 1258;
pub const ORD_LINE_GET_APP_PRIORITY: u32 = 1259;
pub const ORD_LINE_GET_CALL_INFO: u32 = 1260;
pub const ORD_LINE_GET_CALL_STATUS: u32 = 1261;
pub const ORD_LINE_GET_CONF_RELATED_CALLS: u32 = 1262;
pub const ORD_LINE_GET_ICON: u32 = 1263;
pub const ORD_LINE_GET_LINE_DEV_STATUS: u32 = 1264;
pub const ORD_LINE_GET_MESSAGE: u32 = 1265;
pub const ORD_LINE_GET_NEW_CALLS: u32 = 1266;
pub const ORD_LINE_GET_NUM_RINGS: u32 = 1267;
pub const ORD_LINE_GET_PROVIDER_LIST: u32 = 1268;
pub const ORD_LINE_GET_STATUS_MESSAGES: u32 = 1269;
pub const ORD_LINE_HANDOFF: u32 = 1270;
pub const ORD_LINE_HOLD: u32 = 1271;
pub const ORD_LINE_INITIALIZE_EX: u32 = 1272;
pub const ORD_LINE_MONITOR_DIGITS: u32 = 1273;
pub const ORD_LINE_MONITOR_MEDIA: u32 = 1274;
pub const ORD_LINE_NEGOTIATE_EXT_VERSION: u32 = 1275;
pub const ORD_LINE_PICKUP: u32 = 1276;
pub const ORD_LINE_PREPARE_ADD_TO_CONFERENCE: u32 = 1277;
pub const ORD_LINE_REDIRECT: u32 = 1278;
pub const ORD_LINE_RELEASE_USER_USER_INFO: u32 = 1279;
pub const ORD_LINE_REMOVE_FROM_CONFERENCE: u32 = 1280;
pub const ORD_LINE_SEND_USER_USER_INFO: u32 = 1281;
pub const ORD_LINE_SET_APP_PRIORITY: u32 = 1282;
pub const ORD_LINE_SET_CALL_PARAMS: u32 = 1283;
pub const ORD_LINE_SET_CALL_PRIVILEGE: u32 = 1284;
pub const ORD_LINE_SET_MEDIA_MODE: u32 = 1285;
pub const ORD_LINE_SET_NUM_RINGS: u32 = 1286;
pub const ORD_LINE_SET_TERMINAL: u32 = 1287;
pub const ORD_LINE_SET_TOLL_LIST: u32 = 1288;
pub const ORD_LINE_SETUP_CONFERENCE: u32 = 1289;
pub const ORD_LINE_SETUP_TRANSFER: u32 = 1290;
pub const ORD_LINE_SWAP_HOLD: u32 = 1291;
pub const ORD_LINE_UNHOLD: u32 = 1292;
pub const ORD_PHONE_CLOSE: u32 = 1293;
pub const ORD_PHONE_CONFIG_DIALOG: u32 = 1294;
pub const ORD_PHONE_DEV_SPECIFIC: u32 = 1295;
pub const ORD_PHONE_GET_DEV_CAPS: u32 = 1296;
pub const ORD_PHONE_GET_GAIN: u32 = 1297;
pub const ORD_PHONE_GET_HOOK_SWITCH: u32 = 1298;
pub const ORD_PHONE_GET_ICON: u32 = 1299;
pub const ORD_PHONE_GET_ID: u32 = 1300;
pub const ORD_PHONE_GET_MESSAGE: u32 = 1301;
pub const ORD_PHONE_GET_RING: u32 = 1302;
pub const ORD_PHONE_GET_STATUS: u32 = 1303;
pub const ORD_PHONE_GET_STATUS_MESSAGES: u32 = 1304;
pub const ORD_PHONE_GET_VOLUME: u32 = 1305;
pub const ORD_PHONE_INITIALIZE_EX: u32 = 1306;
pub const ORD_PHONE_NEGOTIATE_APIVERSION: u32 = 1307;
pub const ORD_PHONE_NEGOTIATE_EXT_VERSION: u32 = 1308;
pub const ORD_PHONE_OPEN: u32 = 1309;
pub const ORD_PHONE_SET_GAIN: u32 = 1310;
pub const ORD_PHONE_SET_HOOK_SWITCH: u32 = 1311;
pub const ORD_PHONE_SET_RING: u32 = 1312;
pub const ORD_PHONE_SET_STATUS_MESSAGES: u32 = 1313;
pub const ORD_PHONE_SET_VOLUME: u32 = 1314;
pub const ORD_PHONE_SHUTDOWN: u32 = 1315;
pub const ORD_LINE_START_DTMF: u32 = 2608;
pub const ORD_LINE_STOP_DTMF: u32 = 2609;
pub const ORD_AUDIO_UPDATE_FROM_REGISTRY: u32 = 376;
pub const ORD_SND_PLAY_SOUND_W: u32 = 377;
pub const ORD_PLAY_SOUND_W: u32 = 378;
pub const ORD_WAVE_OUT_GET_NUM_DEVS: u32 = 379;
pub const ORD_WAVE_OUT_GET_DEV_CAPS: u32 = 380;
pub const ORD_WAVE_OUT_GET_VOLUME: u32 = 381;
pub const ORD_WAVE_OUT_SET_VOLUME: u32 = 382;
pub const ORD_WAVE_OUT_GET_ERROR_TEXT: u32 = 383;
pub const ORD_WAVE_OUT_CLOSE: u32 = 384;
pub const ORD_WAVE_OUT_PREPARE_HEADER: u32 = 385;
pub const ORD_WAVE_OUT_UNPREPARE_HEADER: u32 = 386;
pub const ORD_WAVE_OUT_WRITE: u32 = 387;
pub const ORD_WAVE_OUT_PAUSE: u32 = 388;
pub const ORD_WAVE_OUT_RESTART: u32 = 389;
pub const ORD_WAVE_OUT_RESET: u32 = 390;
pub const ORD_WAVE_OUT_BREAK_LOOP: u32 = 391;
pub const ORD_WAVE_OUT_GET_POSITION: u32 = 392;
pub const ORD_WAVE_OUT_GET_PITCH: u32 = 393;
pub const ORD_WAVE_OUT_SET_PITCH: u32 = 394;
pub const ORD_WAVE_OUT_GET_PLAYBACK_RATE: u32 = 395;
pub const ORD_WAVE_OUT_SET_PLAYBACK_RATE: u32 = 396;
pub const ORD_WAVE_OUT_GET_PROPERTY: u32 = 1813;
pub const ORD_WAVE_OUT_SET_PROPERTY: u32 = 1814;
pub const ORD_WAVE_OUT_GET_ID: u32 = 397;
pub const ORD_WAVE_OUT_MESSAGE: u32 = 398;
pub const ORD_WAVE_OUT_OPEN: u32 = 399;
pub const ORD_WAVE_IN_GET_NUM_DEVS: u32 = 400;
pub const ORD_WAVE_IN_GET_DEV_CAPS: u32 = 401;
pub const ORD_WAVE_IN_GET_ERROR_TEXT: u32 = 402;
pub const ORD_WAVE_IN_CLOSE: u32 = 403;
pub const ORD_WAVE_IN_PREPARE_HEADER: u32 = 404;
pub const ORD_WAVE_IN_UNPREPARE_HEADER: u32 = 405;
pub const ORD_WAVE_IN_ADD_BUFFER: u32 = 406;
pub const ORD_WAVE_IN_START: u32 = 407;
pub const ORD_WAVE_IN_STOP: u32 = 408;
pub const ORD_WAVE_IN_RESET: u32 = 409;
pub const ORD_WAVE_IN_GET_POSITION: u32 = 410;
pub const ORD_WAVE_IN_GET_ID: u32 = 411;
pub const ORD_WAVE_IN_MESSAGE: u32 = 412;
pub const ORD_WAVE_IN_OPEN: u32 = 413;
pub const ORD_WAVE_IN_GET_PROPERTY: u32 = 1815;
pub const ORD_WAVE_IN_SET_PROPERTY: u32 = 1816;
pub const ORD_ACM_DRIVER_ADD: u32 = 414;
pub const ORD_ACM_DRIVER_CLOSE: u32 = 415;
pub const ORD_ACM_DRIVER_DETAILS: u32 = 416;
pub const ORD_ACM_DRIVER_ENUM: u32 = 417;
pub const ORD_ACM_DRIVER_ID: u32 = 418;
pub const ORD_ACM_DRIVER_MESSAGE: u32 = 419;
pub const ORD_ACM_DRIVER_OPEN: u32 = 420;
pub const ORD_ACM_DRIVER_PRIORITY: u32 = 421;
pub const ORD_ACM_DRIVER_REMOVE: u32 = 422;
pub const ORD_ACM_FILTER_DETAILS: u32 = 424;
pub const ORD_ACM_FILTER_ENUM: u32 = 425;
pub const ORD_ACM_FILTER_TAG_DETAILS: u32 = 426;
pub const ORD_ACM_FILTER_TAG_ENUM: u32 = 427;
pub const ORD_ACM_FORMAT_DETAILS: u32 = 429;
pub const ORD_ACM_FORMAT_ENUM: u32 = 430;
pub const ORD_ACM_FORMAT_SUGGEST: u32 = 431;
pub const ORD_ACM_FORMAT_TAG_DETAILS: u32 = 432;
pub const ORD_ACM_FORMAT_TAG_ENUM: u32 = 433;
pub const ORD_ACM_STREAM_CLOSE: u32 = 434;
pub const ORD_ACM_STREAM_CONVERT: u32 = 435;
pub const ORD_ACM_STREAM_MESSAGE: u32 = 436;
pub const ORD_ACM_STREAM_OPEN: u32 = 437;
pub const ORD_ACM_STREAM_PREPARE_HEADER: u32 = 438;
pub const ORD_ACM_STREAM_RESET: u32 = 439;
pub const ORD_ACM_STREAM_SIZE: u32 = 440;
pub const ORD_ACM_STREAM_UNPREPARE_HEADER: u32 = 441;
pub const ORD_ACM_GET_VERSION: u32 = 442;
pub const ORD_ACM_METRICS: u32 = 443;
pub const ORD_ACM_FORMAT_CHOOSE: u32 = 428;
pub const ORD_ACM_FILTER_CHOOSE: u32 = 423;
pub const ORD_MIXER_GET_CONTROL_DETAILS: u32 = 1589;
pub const ORD_MIXER_GET_DEV_CAPS: u32 = 1591;
pub const ORD_MIXER_GET_ID: u32 = 1590;
pub const ORD_MIXER_GET_LINE_CONTROLS: u32 = 1592;
pub const ORD_MIXER_GET_LINE_INFO: u32 = 1593;
pub const ORD_MIXER_GET_NUM_DEVS: u32 = 1594;
pub const ORD_MIXER_MESSAGE: u32 = 1596;
pub const ORD_MIXER_OPEN: u32 = 1595;
pub const ORD_MIXER_SET_CONTROL_DETAILS: u32 = 1597;
pub const ORD_MIXER_CLOSE: u32 = 1598;
pub const ORD_WNET_ADD_CONNECTION3_W: u32 = 444;
pub const ORD_WNET_CANCEL_CONNECTION2_W: u32 = 445;
pub const ORD_WNET_CONNECTION_DIALOG1_W: u32 = 446;
pub const ORD_WNET_DISCONNECT_DIALOG: u32 = 447;
pub const ORD_WNET_DISCONNECT_DIALOG1_W: u32 = 448;
pub const ORD_WNET_GET_CONNECTION_W: u32 = 449;
pub const ORD_WNET_GET_UNIVERSAL_NAME_W: u32 = 450;
pub const ORD_WNET_GET_USER_W: u32 = 451;
pub const ORD_WNET_OPEN_ENUM_W: u32 = 452;
pub const ORD_WNET_CLOSE_ENUM: u32 = 453;
pub const ORD_WNET_ENUM_RESOURCE_W: u32 = 454;
pub const ORD_GET_USER_NAME_EX_W: u32 = 1503;
pub const ORD_REG_CLOSE_KEY: u32 = 455;
pub const ORD_REG_CREATE_KEY_EX_W: u32 = 456;
pub const ORD_REG_DELETE_KEY_W: u32 = 457;
pub const ORD_REG_DELETE_VALUE_W: u32 = 458;
pub const ORD_REG_ENUM_VALUE_W: u32 = 459;
pub const ORD_REG_ENUM_KEY_EX_W: u32 = 460;
pub const ORD_REG_OPEN_KEY_EX_W: u32 = 461;
pub const ORD_REG_QUERY_INFO_KEY_W: u32 = 462;
pub const ORD_REG_QUERY_VALUE_EX_W: u32 = 463;
pub const ORD_REG_SET_VALUE_EX_W: u32 = 464;
pub const ORD_REG_FLUSH_KEY: u32 = 1152;
pub const ORD_CE_REG_TEST_SET_VALUE_W: u32 = 2504;
pub const ORD_CE_REG_GET_INFO: u32 = 2505;
pub const ORD_CE_REG_GET_NOTIFICATION_INFO: u32 = 2506;
pub const ORD_CE_FIND_FIRST_REG_CHANGE: u32 = 1950;
pub const ORD_CE_FIND_NEXT_REG_CHANGE: u32 = 1951;
pub const ORD_CE_FIND_CLOSE_REG_CHANGE: u32 = 1952;
pub const ORD_REG_COPY_FILE: u32 = 465;
pub const ORD_REG_RESTORE_FILE: u32 = 466;
pub const ORD_REG_SAVE_KEY: u32 = 1478;
pub const ORD_REG_REPLACE_KEY: u32 = 1479;
pub const ORD_SET_CURRENT_USER: u32 = 1501;
pub const ORD_SET_USER_DATA: u32 = 1502;
pub const ORD_GET_USER_DIRECTORY: u32 = 1686;
pub const ORD_CRYPT_PROTECT_DATA: u32 = 1599;
pub const ORD_CRYPT_UNPROTECT_DATA: u32 = 1600;
pub const ORD_CE_GEN_RANDOM: u32 = 1601;
pub const ORD_GET_DEVICE_UNIQUE_ID: u32 = 1993;
pub const ORD_PEG_SET_USER_NOTIFICATION: u32 = 467;
pub const ORD_PEG_CLEAR_USER_NOTIFICATION: u32 = 468;
pub const ORD_PEG_RUN_APP_AT_TIME: u32 = 469;
pub const ORD_PEG_RUN_APP_AT_EVENT: u32 = 470;
pub const ORD_PEG_HANDLE_APP_NOTIFICATIONS: u32 = 471;
pub const ORD_PEG_GET_USER_NOTIFICATION_PREFERENCES: u32 = 472;
pub const ORD_CE_SET_USER_NOTIFICATION: u32 = 473;
pub const ORD_CE_CLEAR_USER_NOTIFICATION: u32 = 474;
pub const ORD_CE_RUN_APP_AT_TIME: u32 = 475;
pub const ORD_CE_RUN_APP_AT_EVENT: u32 = 476;
pub const ORD_CE_HANDLE_APP_NOTIFICATIONS: u32 = 477;
pub const ORD_CE_GET_USER_NOTIFICATION_PREFERENCES: u32 = 478;
pub const ORD_CE_SET_USER_NOTIFICATION_EX: u32 = 1352;
pub const ORD_CE_GET_USER_NOTIFICATION_HANDLES: u32 = 1353;
pub const ORD_CE_GET_USER_NOTIFICATION: u32 = 1354;
pub const ORD_CE_GET_NOTIFICATION_THREAD_ID: u32 = 1823;
pub const ORD_SHELL_NOTIFY_ICON: u32 = 481;
pub const ORD_SHADD_TO_RECENT_DOCS: u32 = 483;
pub const ORD_SHCREATE_EXPLORER_INSTANCE: u32 = 1163;
pub const ORD_SHDONE_BUTTON_I: u32 = 1782;
pub const ORD_SHGET_APP_KEY_ASSOC_I: u32 = 1783;
pub const ORD_SHSET_APP_KEY_WND_ASSOC_I: u32 = 1784;
pub const ORD_SHSET_NAV_BAR_TEXT_I: u32 = 1785;
pub const ORD_SHSIP_PREFERENCE_I: u32 = 1786;
pub const ORD_NOT_SYSTEM_PARAMETERS_INFO_I: u32 = 1787;
pub const ORD_SHCLOSE_APPS_I: u32 = 1788;
pub const ORD_SHFILE_NOTIFY_REMOVE_I: u32 = 1803;
pub const ORD_SHFILE_NOTIFY_FREE_I: u32 = 1804;
pub const ORD_SHCHANGE_NOTIFY_REGISTER_I: u32 = 1805;
pub const ORD_SHNOTIFICATION_ADD_I: u32 = 1806;
pub const ORD_SHNOTIFICATION_UPDATE_I: u32 = 1807;
pub const ORD_SHNOTIFICATION_REMOVE_I: u32 = 1808;
pub const ORD_SHNOTIFICATION_GET_DATA_I: u32 = 1809;
pub const ORD_SHELL_EXECUTE_EX: u32 = 480;
pub const ORD_SHCREATE_SHORTCUT: u32 = 484;
pub const ORD_SHGET_SHORTCUT_TARGET: u32 = 485;
pub const ORD_SHCREATE_SHORTCUT_EX: u32 = 1488;
pub const ORD_SHSHOW_OUT_OF_MEMORY: u32 = 486;
pub const ORD_SHLOAD_DIBITMAP: u32 = 487;
pub const ORD_SHLOAD_INDIRECT_STRING: u32 = 1977;
pub const ORD_SHGET_FILE_INFO: u32 = 482;
pub const ORD_SHGET_SPECIAL_FOLDER_PATH: u32 = 295;
pub const ORD_GET_OPEN_FILE_NAME_W: u32 = 488;
pub const ORD_GET_SAVE_FILE_NAME_W: u32 = 489;
pub const ORD_DPA_CREATE: u32 = 1837;
pub const ORD_DPA_CREATE_EX: u32 = 1838;
pub const ORD_DPA_CLONE: u32 = 1839;
pub const ORD_DPA_DELETE_ALL_PTRS: u32 = 1840;
pub const ORD_DPA_DELETE_PTR: u32 = 1841;
pub const ORD_DPA_DESTROY: u32 = 1842;
pub const ORD_DPA_DESTROY_CALLBACK: u32 = 1843;
pub const ORD_DPA_ENUM_CALLBACK: u32 = 1844;
pub const ORD_DPA_GET_PTR: u32 = 1845;
pub const ORD_DPA_GET_PTR_INDEX: u32 = 1846;
pub const ORD_DPA_GROW: u32 = 1847;
pub const ORD_DPA_INSERT_PTR: u32 = 1848;
pub const ORD_DPA_SEARCH: u32 = 1849;
pub const ORD_DPA_SET_PTR: u32 = 1850;
pub const ORD_DPA_SORT: u32 = 1851;
pub const ORD_DSA_CREATE: u32 = 1852;
pub const ORD_DSA_CLONE: u32 = 1853;
pub const ORD_DSA_DELETE_ALL_ITEMS: u32 = 1854;
pub const ORD_DSA_DELETE_ITEM: u32 = 1855;
pub const ORD_DSA_DESTROY: u32 = 1856;
pub const ORD_DSA_DESTROY_CALLBACK: u32 = 1857;
pub const ORD_DSA_ENUM_CALLBACK: u32 = 1858;
pub const ORD_DSA_GET_ITEM: u32 = 1859;
pub const ORD_DSA_GET_ITEM_PTR: u32 = 1860;
pub const ORD_DSA_GROW: u32 = 1861;
pub const ORD_DSA_INSERT_ITEM: u32 = 1862;
pub const ORD_DSA_SEARCH: u32 = 1863;
pub const ORD_DSA_SET_ITEM: u32 = 1864;
pub const ORD_DSA_SET_RANGE: u32 = 1865;
pub const ORD_DSA_SORT: u32 = 1866;
pub const ORD_PERFORM_CALL_BACK4: u32 = 1448;
pub const ORD_QUERY_APISET_ID: u32 = 490;
pub const ORD_TERMINATE_THREAD: u32 = 491;
pub const ORD_CREATE_THREAD: u32 = 492;
pub const ORD_CREATE_PROCESS_W: u32 = 493;
pub const ORD_EVENT_MODIFY: u32 = 494;
pub const ORD_CREATE_EVENT_W: u32 = 495;
pub const ORD_OPEN_EVENT_W: u32 = 1496;
pub const ORD_GET_EVENT_DATA: u32 = 1527;
pub const ORD_SET_EVENT_DATA: u32 = 1528;
pub const ORD_IS_NAMED_EVENT_SIGNALED: u32 = 1981;
pub const ORD_SLEEP: u32 = 496;
pub const ORD_WAIT_FOR_SINGLE_OBJECT: u32 = 497;
pub const ORD_WAIT_FOR_MULTIPLE_OBJECTS: u32 = 498;
pub const ORD_SUSPEND_THREAD: u32 = 499;
pub const ORD_RESUME_THREAD: u32 = 500;
pub const ORD_GET_THREAD_CONTEXT: u32 = 1148;
pub const ORD_SET_THREAD_CONTEXT: u32 = 502;
pub const ORD_WAIT_FOR_DEBUG_EVENT: u32 = 503;
pub const ORD_CONTINUE_DEBUG_EVENT: u32 = 504;
pub const ORD_DEBUG_ACTIVE_PROCESS: u32 = 505;
pub const ORD_DEBUG_ACTIVE_PROCESS_STOP: u32 = 1991;
pub const ORD_DEBUG_SET_PROCESS_KILL_ON_EXIT: u32 = 1992;
pub const ORD_CE_GET_MODULE_INFO: u32 = 1994;
pub const ORD_CHECK_REMOTE_DEBUGGER_PRESENT: u32 = 2507;
pub const ORD_CE_OPEN_FILE_HANDLE: u32 = 2511;
pub const ORD_READ_PROCESS_MEMORY: u32 = 506;
pub const ORD_WRITE_PROCESS_MEMORY: u32 = 507;
pub const ORD_FLUSH_INSTRUCTION_CACHE: u32 = 508;
pub const ORD_CE_SET_PROCESS_VERSION: u32 = 1775;
pub const ORD_OPEN_PROCESS: u32 = 509;
pub const ORD_OPEN_THREAD: u32 = 2551;
pub const ORD_GET_MODULE_INFORMATION: u32 = 1721;
pub const ORD_DUMP_KCALL_PROFILE: u32 = 510;
pub const ORD_THCREATE_SNAPSHOT: u32 = 511;
pub const ORD_NOTIFY_FORCE_CLEANBOOT: u32 = 513;
pub const ORD_SET_THREAD_PRIORITY: u32 = 514;
pub const ORD_GET_THREAD_PRIORITY: u32 = 515;
pub const ORD_CE_SET_THREAD_PRIORITY: u32 = 621;
pub const ORD_CE_GET_THREAD_PRIORITY: u32 = 622;
pub const ORD_CE_SET_THREAD_QUANTUM: u32 = 1244;
pub const ORD_CE_GET_THREAD_QUANTUM: u32 = 1245;
pub const ORD_GET_THREAD_ID: u32 = 2558;
pub const ORD_GET_PROCESS_ID_OF_THREAD: u32 = 2559;
pub const ORD_GET_LAST_ERROR: u32 = 516;
pub const ORD_SET_LAST_ERROR: u32 = 517;
pub const ORD_GET_EXIT_CODE_THREAD: u32 = 518;
pub const ORD_GET_EXIT_CODE_PROCESS: u32 = 519;
pub const ORD_TLS_CALL: u32 = 520;
pub const ORD_IS_BAD_CODE_PTR: u32 = 521;
pub const ORD_IS_BAD_READ_PTR: u32 = 522;
pub const ORD_IS_BAD_WRITE_PTR: u32 = 523;
pub const ORD_VIRTUAL_ALLOC: u32 = 524;
pub const ORD_VIRTUAL_FREE: u32 = 525;
pub const ORD_VIRTUAL_ALLOC_EX: u32 = 2563;
pub const ORD_VIRTUAL_FREE_EX: u32 = 2564;
pub const ORD_VIRTUAL_PROTECT: u32 = 526;
pub const ORD_VIRTUAL_QUERY: u32 = 527;
pub const ORD_VIRTUAL_QUERY_EX: u32 = 2540;
pub const ORD_VIRTUAL_PROTECT_EX: u32 = 2541;
pub const ORD_LOAD_LIBRARY_W: u32 = 528;
pub const ORD_LOAD_LIBRARY_EX_W: u32 = 1241;
pub const ORD_FREE_LIBRARY: u32 = 529;
pub const ORD_GET_PROC_ADDRESS_W: u32 = 530;
pub const ORD_FIND_RESOURCE: u32 = 531;
pub const ORD_FIND_RESOURCE_W: u32 = 532;
pub const ORD_LOAD_RESOURCE: u32 = 533;
pub const ORD_LOAD_STRING_W: u32 = 874;
pub const ORD_SIZEOF_RESOURCE: u32 = 534;
pub const ORD_VER_QUERY_VALUE_W: u32 = 1459;
pub const ORD_GET_FILE_VERSION_INFO_W: u32 = 1460;
pub const ORD_GET_FILE_VERSION_INFO_SIZE_W: u32 = 1461;
pub const ORD_GET_TICK_COUNT: u32 = 535;
pub const ORD_GET_PROCESS_VERSION: u32 = 536;
pub const ORD_GET_MODULE_FILE_NAME_W: u32 = 537;
pub const ORD_GET_MODULE_HANDLE_W: u32 = 1177;
pub const ORD_QUERY_PERFORMANCE_COUNTER: u32 = 538;
pub const ORD_QUERY_PERFORMANCE_FREQUENCY: u32 = 539;
pub const ORD_FORCE_PAGEOUT: u32 = 540;
pub const ORD_GET_THREAD_TIMES: u32 = 1186;
pub const ORD_OUTPUT_DEBUG_STRING_W: u32 = 541;
pub const ORD_GET_SYSTEM_INFO: u32 = 542;
pub const ORD_QUERY_INSTRUCTION_SET: u32 = 1677;
pub const ORD_IS_PROCESSOR_FEATURE_PRESENT: u32 = 1758;
pub const ORD_RAISE_EXCEPTION: u32 = 543;
pub const ORD_TERMINATE_PROCESS: u32 = 544;
pub const ORD_NKDBG_PRINTF_W: u32 = 545;
pub const ORD_REGISTER_DBG_ZONES: u32 = 546;
pub const ORD_SET_DAYLIGHT_TIME: u32 = 547;
pub const ORD_GET_CALL_STACK_SNAPSHOT: u32 = 1760;
pub const ORD_GET_THREAD_CALL_STACK: u32 = 1811;
pub const ORD_PAGE_OUT_MODULE: u32 = 1780;
pub const ORD_GET_PROC_ADDRESS_IN_PROCESS: u32 = 2599;
pub const ORD_CREATE_FILE_MAPPING_W: u32 = 548;
pub const ORD_MAP_VIEW_OF_FILE: u32 = 549;
pub const ORD_UNMAP_VIEW_OF_FILE: u32 = 550;
pub const ORD_FLUSH_VIEW_OF_FILE: u32 = 551;
pub const ORD_FLUSH_VIEW_OF_FILE_MAYBE: u32 = 1215;
pub const ORD_CREATE_FILE_FOR_MAPPING: u32 = 552;
pub const ORD_CREATE_FILE_FOR_MAPPING_W: u32 = 1167;
pub const ORD_CE_OPEN_CALLER_BUFFER: u32 = 2569;
pub const ORD_CE_CLOSE_CALLER_BUFFER: u32 = 2570;
pub const ORD_CE_ALLOC_ASYNCHRONOUS_BUFFER: u32 = 2571;
pub const ORD_CE_FREE_ASYNCHRONOUS_BUFFER: u32 = 2572;
pub const ORD_CE_FLUSH_ASYNCHRONOUS_BUFFER: u32 = 2607;
pub const ORD_CE_ALLOC_DUPLICATE_BUFFER: u32 = 2573;
pub const ORD_CE_FREE_DUPLICATE_BUFFER: u32 = 2574;
pub const ORD_CLOSE_HANDLE: u32 = 553;
pub const ORD_CREATE_MUTEX_W: u32 = 555;
pub const ORD_RELEASE_MUTEX: u32 = 556;
pub const ORD_KERNEL_IO_CONTROL: u32 = 557;
pub const ORD_KERNEL_LIB_IO_CONTROL: u32 = 1489;
pub const ORD_CREATE_STATIC_MAPPING: u32 = 1539;
pub const ORD_DELETE_STATIC_MAPPING: u32 = 1826;
pub const ORD_MAP_CALLER_PTR: u32 = 1602;
pub const ORD_MAP_PTR_TO_PROC_WITH_SIZE: u32 = 1603;
pub const ORD_FREE_LIBRARY_AND_EXIT_THREAD: u32 = 1216;
pub const ORD_GET_PROC_ADDRESS_A: u32 = 1230;
pub const ORD_GET_COMMAND_LINE_W: u32 = 1231;
pub const ORD_DISABLE_THREAD_LIBRARY_CALLS: u32 = 1232;
pub const ORD_TRY_ENTER_CRITICAL_SECTION: u32 = 1233;
pub const ORD_GET_TEMP_FILE_NAME_W: u32 = 1234;
pub const ORD_CE_GET_CANONICAL_PATH_NAME_W: u32 = 1957;
pub const ORD_MATCHES_WILDCARD_MASK: u32 = 1959;
pub const ORD_CREATE_SEMAPHORE_W: u32 = 1238;
pub const ORD_RELEASE_SEMAPHORE: u32 = 1239;
pub const ORD_CE_MAP_ARGUMENT_ARRAY: u32 = 1446;
pub const ORD_CE_SET_EXTENDED_PDATA: u32 = 1455;
pub const ORD_GET_PROCESS_ID: u32 = 2560;
pub const ORD_CREATE_WATCH_DOG_TIMER: u32 = 2530;
pub const ORD_OPEN_WATCH_DOG_TIMER: u32 = 2531;
pub const ORD_START_WATCH_DOG_TIMER: u32 = 2532;
pub const ORD_STOP_WATCH_DOG_TIMER: u32 = 2533;
pub const ORD_REFRESH_WATCH_DOG_TIMER: u32 = 2534;
pub const ORD_ADD_EVENT_ACCESS: u32 = 558;
pub const ORD_CREATE_APISET: u32 = 559;
pub const ORD_VIRTUAL_COPY: u32 = 560;
pub const ORD_VIRTUAL_COPY_EX: u32 = 2565;
pub const ORD_VIRTUAL_ALLOC_COPY_EX: u32 = 2566;
pub const ORD_VIRTUAL_SET_PAGE_FLAGS: u32 = 1183;
pub const ORD_SET_RAMMODE: u32 = 1184;
pub const ORD_SET_STORE_QUEUE_BASE: u32 = 1212;
pub const ORD_VIRTUAL_SET_ATTRIBUTES: u32 = 1724;
pub const ORD_CE_VIRTUAL_SHARED_ALLOC: u32 = 1812;
pub const ORD_ALLOC_PHYS_MEM: u32 = 1486;
pub const ORD_FREE_PHYS_MEM: u32 = 1487;
pub const ORD_SLEEP_TILL_TICK: u32 = 1534;
pub const ORD_DUPLICATE_HANDLE: u32 = 1535;
pub const ORD_DEVICE_IO_CONTROL: u32 = 179;
pub const ORD_FORWARD_DEVICE_IO_CONTROL: u32 = 2605;
pub const ORD_LOCK_PAGES: u32 = 1161;
pub const ORD_UNLOCK_PAGES: u32 = 1162;
pub const ORD_U_ROPEN: u32 = 563;
pub const ORD_U_RREAD: u32 = 564;
pub const ORD_U_RWRITE: u32 = 565;
pub const ORD_U_RLSEEK: u32 = 566;
pub const ORD_U_RCLOSE: u32 = 567;
pub const ORD_UPDATE_NLSINFO: u32 = 1447;
pub const ORD_UPDATE_NLSINFO_EX: u32 = 1796;
pub const ORD_NKV_DBG_PRINTF_W: u32 = 568;
pub const ORD_PROFILE_SYSCALL: u32 = 569;
pub const ORD_GET_REAL_TIME: u32 = 570;
pub const ORD_SET_REAL_TIME: u32 = 571;
pub const ORD_EXTRACT_RESOURCE: u32 = 573;
pub const ORD_KERN_EXTRACT_ICONS: u32 = 574;
pub const ORD_GET_ROM_FILE_INFO: u32 = 575;
pub const ORD_GET_ROM_FILE_BYTES: u32 = 576;
pub const ORD_CACHE_SYNC: u32 = 577;
pub const ORD_CACHE_RANGE_FLUSH: u32 = 1765;
pub const ORD_ADD_TRACKED_ITEM: u32 = 578;
pub const ORD_DELETE_TRACKED_ITEM: u32 = 579;
pub const ORD_PRINT_TRACKED_ITEM: u32 = 580;
pub const ORD_GET_KPHYS: u32 = 581;
pub const ORD_GIVE_KPHYS: u32 = 582;
pub const ORD_SET_EXCEPTION_HANDLER: u32 = 583;
pub const ORD_REGISTER_TRACKED_ITEM: u32 = 584;
pub const ORD_FILTER_TRACKED_ITEM: u32 = 585;
pub const ORD_SET_KERNEL_ALARM: u32 = 586;
pub const ORD_REFRESH_KERNEL_ALARM: u32 = 587;
pub const ORD_SET_GWES_OOMEVENT: u32 = 590;
pub const ORD_SET_OOMEVENT: u32 = 1462;
pub const ORD_STRING_COMPRESS: u32 = 591;
pub const ORD_STRING_DECOMPRESS: u32 = 592;
pub const ORD_BINARY_COMPRESS: u32 = 593;
pub const ORD_BINARY_DECOMPRESS: u32 = 594;
pub const ORD_DECOMPRESS_BINARY_BLOCK: u32 = 1776;
pub const ORD_INPUT_DEBUG_CHAR_W: u32 = 595;
pub const ORD_MAP_PTR_TO_PROCESS: u32 = 598;
pub const ORD_MAP_PTR_UNSECURE: u32 = 599;
pub const ORD_GET_PROC_FROM_PTR: u32 = 600;
pub const ORD_IS_BAD_PTR: u32 = 601;
pub const ORD_GET_PROC_ADDR_BITS: u32 = 602;
pub const ORD_GET_FSHEAP_INFO: u32 = 603;
pub const ORD_PREPARE_THREAD_EXIT: u32 = 605;
pub const ORD_GET_OWNER_PROCESS: u32 = 606;
pub const ORD_GET_CALLER_PROCESS: u32 = 607;
pub const ORD_GET_CALLER_VMPROCESS_ID: u32 = 2603;
pub const ORD_GET_DIRECT_CALLER_PROCESS_ID: u32 = 2604;
pub const ORD_GET_IDLE_TIME: u32 = 608;
pub const ORD_SET_LOWEST_SCHEDULED_PRIORITY: u32 = 609;
pub const ORD_IS_PRIMARY_THREAD: u32 = 610;
pub const ORD_SET_PROC_PERMISSIONS: u32 = 611;
pub const ORD_CE_GET_CURRENT_TRUST: u32 = 1357;
pub const ORD_CE_GET_CALLER_TRUST: u32 = 1395;
pub const ORD_CE_GET_PROCESS_TRUST: u32 = 2510;
pub const ORD_GET_CURRENT_PERMISSIONS: u32 = 612;
pub const ORD_SET_TIME_ZONE_BIAS: u32 = 614;
pub const ORD_SET_CLEAN_REBOOT_FLAG: u32 = 615;
pub const ORD_POWER_OFF_SYSTEM: u32 = 617;
pub const ORD_SET_DBG_ZONE: u32 = 618;
pub const ORD_TURN_ON_PROFILING: u32 = 619;
pub const ORD_TURN_OFF_PROFILING: u32 = 620;
pub const ORD_GET_PROC_NAME: u32 = 624;
pub const ORD_SET_HANDLE_OWNER: u32 = 625;
pub const ORD_SET_HANDLE_OWNER_WORKAROUND: u32 = 2550;
pub const ORD_LOAD_DRIVER: u32 = 626;
pub const ORD_LOAD_INT_CHAIN_HANDLER: u32 = 1475;
pub const ORD_FREE_INT_CHAIN_HANDLER: u32 = 1476;
pub const ORD_INT_CHAIN_HANDLER_IO_CONTROL: u32 = 2614;
pub const ORD_LOAD_KERNEL_LIBRARY: u32 = 1671;
pub const ORD_INTERRUPT_INITIALIZE: u32 = 627;
pub const ORD_INTERRUPT_MASK: u32 = 1797;
pub const ORD_INTERRUPT_DONE: u32 = 628;
pub const ORD_INTERRUPT_DISABLE: u32 = 629;
pub const ORD_SET_KMODE: u32 = 630;
pub const ORD_SET_POWER_OFF_HANDLER: u32 = 631;
pub const ORD_SET_GWES_POWER_HANDLER: u32 = 632;
pub const ORD_CONNECT_DEBUGGER: u32 = 633;
pub const ORD_CONNECT_HDSTUB: u32 = 1953;
pub const ORD_CONNECT_OS_AXS_T0: u32 = 1954;
pub const ORD_CONNECT_OS_AXS_T1: u32 = 1983;
pub const ORD_SET_HARDWARE_WATCH: u32 = 634;
pub const ORD_REGISTER_APISET: u32 = 635;
pub const ORD_CREATE_APIHANDLE: u32 = 636;
pub const ORD_VERIFY_APIHANDLE: u32 = 637;
pub const ORD_REGISTER_DIRECT_METHODS: u32 = 2555;
pub const ORD_LOCK_APIHANDLE: u32 = 2553;
pub const ORD_UNLOCK_APIHANDLE: u32 = 2554;
pub const ORD_SET_APIERROR_HANDLER: u32 = 2611;
pub const ORD_PPSHRESTART: u32 = 638;
pub const ORD_GET_PROCESS_INDEX_FROM_ID: u32 = 640;
pub const ORD_GET_PROCESS_IDFROM_INDEX: u32 = 1727;
pub const ORD_GET_CALLER_PROCESS_INDEX: u32 = 641;
pub const ORD_DEBUG_NOTIFY: u32 = 642;
pub const ORD_READ_REGISTRY_FROM_OEM: u32 = 1153;
pub const ORD_WRITE_REGISTRY_TO_OEM: u32 = 1154;
pub const ORD_WRITE_DEBUG_LED: u32 = 1155;
pub const ORD_AFS_UNMOUNT: u32 = 643;
pub const ORD_AFS_CREATE_DIRECTORY_W: u32 = 644;
pub const ORD_AFS_REMOVE_DIRECTORY_W: u32 = 645;
pub const ORD_AFS_GET_FILE_ATTRIBUTES_W: u32 = 646;
pub const ORD_AFS_SET_FILE_ATTRIBUTES_W: u32 = 647;
pub const ORD_AFS_CREATE_FILE_W: u32 = 648;
pub const ORD_AFS_DELETE_FILE_W: u32 = 649;
pub const ORD_AFS_MOVE_FILE_W: u32 = 650;
pub const ORD_AFS_FIND_FIRST_FILE_W: u32 = 651;
pub const ORD_AFS_REGISTER_FILE_SYSTEM_FUNCTION: u32 = 652;
pub const ORD_AFS_PRESTO_CHANGO_FILE_NAME: u32 = 654;
pub const ORD_AFS_CLOSE_ALL_FILE_HANDLES: u32 = 655;
pub const ORD_AFS_GET_DISK_FREE_SPACE: u32 = 656;
pub const ORD_AFS_NOTIFY_MOUNTED_FS: u32 = 657;
pub const ORD_AFS_FIND_FIRST_CHANGE_NOTIFICATION_W: u32 = 1685;
pub const ORD_AFS_FS_IO_CONTROL_W: u32 = 1966;
pub const ORD_AFS_SET_FILE_SECURITY_W: u32 = 2709;
pub const ORD_AFS_GET_FILE_SECURITY_W: u32 = 2710;
pub const ORD_CLEAR_EVENT_LOG_W: u32 = 1818;
pub const ORD_REPORT_EVENT_W: u32 = 1819;
pub const ORD_REGISTER_EVENT_SOURCE_W: u32 = 1820;
pub const ORD_DEREGISTER_EVENT_SOURCE: u32 = 1821;
pub const ORD_OPEN_EVENT_LOG_W: u32 = 1970;
pub const ORD_CLOSE_EVENT_LOG: u32 = 1971;
pub const ORD_BACKUP_EVENT_LOG_W: u32 = 1972;
pub const ORD_LOCK_EVENT_LOG: u32 = 1973;
pub const ORD_UN_LOCK_EVENT_LOG: u32 = 1974;
pub const ORD_READ_EVENT_LOG_RAW: u32 = 1975;
pub const ORD_VERIFY_USER: u32 = 1827;
pub const ORD_LASSRELOAD_CONFIG: u32 = 1828;
pub const ORD_CREATE_ENROLLMENT_CONFIG_DIALOG: u32 = 1976;
pub const ORD_LASSGET_VALUE: u32 = 2515;
pub const ORD_VERIFY_USER_ASYNC: u32 = 2516;
pub const ORD_LASSGET_RESULT: u32 = 2517;
pub const ORD_LASSCLOSE: u32 = 2518;
pub const ORD_REQUEST_BLUETOOTH_NOTIFICATIONS: u32 = 1995;
pub const ORD_STOP_BLUETOOTH_NOTIFICATIONS: u32 = 1996;
pub const ORD_REGISTER_BLUETOOTH_COMPORT: u32 = 2600;
pub const ORD_DEREGISTER_BLUETOOTH_COMPORT: u32 = 2601;
pub const ORD_CREATE_CARET: u32 = 658;
pub const ORD_DESTROY_CARET: u32 = 659;
pub const ORD_HIDE_CARET: u32 = 660;
pub const ORD_SHOW_CARET: u32 = 661;
pub const ORD_SET_CARET_POS: u32 = 662;
pub const ORD_GET_CARET_POS: u32 = 663;
pub const ORD_SET_CARET_BLINK_TIME: u32 = 664;
pub const ORD_GET_CARET_BLINK_TIME: u32 = 665;
pub const ORD_DISABLE_CARET_SYSTEM_WIDE: u32 = 666;
pub const ORD_ENABLE_CARET_SYSTEM_WIDE: u32 = 667;
pub const ORD_OPEN_CLIPBOARD: u32 = 668;
pub const ORD_CLOSE_CLIPBOARD: u32 = 669;
pub const ORD_GET_CLIPBOARD_OWNER: u32 = 670;
pub const ORD_SET_CLIPBOARD_DATA: u32 = 671;
pub const ORD_GET_CLIPBOARD_DATA: u32 = 672;
pub const ORD_REGISTER_CLIPBOARD_FORMAT_W: u32 = 673;
pub const ORD_COUNT_CLIPBOARD_FORMATS: u32 = 674;
pub const ORD_ENUM_CLIPBOARD_FORMATS: u32 = 675;
pub const ORD_GET_CLIPBOARD_FORMAT_NAME_W: u32 = 676;
pub const ORD_EMPTY_CLIPBOARD: u32 = 677;
pub const ORD_IS_CLIPBOARD_FORMAT_AVAILABLE: u32 = 678;
pub const ORD_GET_PRIORITY_CLIPBOARD_FORMAT: u32 = 679;
pub const ORD_GET_OPEN_CLIPBOARD_WINDOW: u32 = 680;
pub const ORD_GET_CLIPBOARD_DATA_ALLOC: u32 = 681;
pub const ORD_CREATE_DIALOG_INDIRECT_PARAM_W: u32 = 688;
pub const ORD_DIALOG_BOX_INDIRECT_PARAM_W: u32 = 690;
pub const ORD_CHECK_RADIO_BUTTON: u32 = 684;
pub const ORD_SEND_DLG_ITEM_MESSAGE_W: u32 = 685;
pub const ORD_SET_DLG_ITEM_TEXT_W: u32 = 686;
pub const ORD_GET_DLG_ITEM_TEXT_W: u32 = 687;
pub const ORD_DEF_DLG_PROC_W: u32 = 689;
pub const ORD_END_DIALOG: u32 = 691;
pub const ORD_GET_DLG_ITEM: u32 = 692;
pub const ORD_GET_DLG_CTRL_ID: u32 = 693;
pub const ORD_GET_DIALOG_BASE_UNITS: u32 = 694;
pub const ORD_GET_DLG_ITEM_INT: u32 = 695;
pub const ORD_GET_NEXT_DLG_TAB_ITEM: u32 = 696;
pub const ORD_GET_NEXT_DLG_GROUP_ITEM: u32 = 697;
pub const ORD_IS_DIALOG_MESSAGE_W: u32 = 698;
pub const ORD_MAP_DIALOG_RECT: u32 = 699;
pub const ORD_SET_DLG_ITEM_INT: u32 = 700;
pub const ORD_SET_DIALOG_AUTO_SCROLL_BAR: u32 = 2705;
pub const ORD_SET_WINDOW_POS_ON_ROTATE: u32 = 2706;
pub const ORD_GET_FOREGROUND_WINDOW: u32 = 701;
pub const ORD_SET_FOREGROUND_WINDOW: u32 = 702;
pub const ORD_SET_ACTIVE_WINDOW: u32 = 703;
pub const ORD_SET_FOCUS: u32 = 704;
pub const ORD_GET_FOCUS: u32 = 705;
pub const ORD_GET_ACTIVE_WINDOW: u32 = 706;
pub const ORD_GET_CAPTURE: u32 = 707;
pub const ORD_SET_CAPTURE: u32 = 708;
pub const ORD_RELEASE_CAPTURE: u32 = 709;
pub const ORD_SET_KEYBOARD_TARGET: u32 = 710;
pub const ORD_GET_KEYBOARD_TARGET: u32 = 711;
pub const ORD_SHELL_MODAL_END: u32 = 712;
pub const ORD_GET_FOREGROUND_INFO: u32 = 1224;
pub const ORD_GET_FOREGROUND_KEYBOARD_TARGET: u32 = 1225;
pub const ORD_GET_FOREGROUND_KEYBOARD_LAYOUT_HANDLE: u32 = 1802;
pub const ORD_BATTERY_DRVR_GET_LEVELS: u32 = 297;
pub const ORD_BATTERY_DRVR_SUPPORTS_CHANGE_NOTIFICATION: u32 = 298;
pub const ORD_BATTERY_GET_LIFE_TIME_INFO: u32 = 713;
pub const ORD_BATTERY_NOTIFY_OF_TIME_CHANGE: u32 = 714;
pub const ORD_GET_SYSTEM_POWER_STATUS_EX: u32 = 715;
pub const ORD_GET_SYSTEM_POWER_STATUS_EX2: u32 = 1358;
pub const ORD_NOTIFY_WIN_USER_SYSTEM: u32 = 716;
pub const ORD_EXTRACT_ICON_EX_W: u32 = 727;
pub const ORD_CREATE_ICON_INDIRECT: u32 = 723;
pub const ORD_DESTROY_ICON: u32 = 725;
pub const ORD_DRAW_ICON_EX: u32 = 726;
pub const ORD_LOAD_ICON_W: u32 = 728;
pub const ORD_GET_ICON_INFO: u32 = 1822;
pub const ORD_DESTROY_CURSOR: u32 = 724;
pub const ORD_CREATE_CURSOR: u32 = 722;
pub const ORD_SET_CURSOR: u32 = 682;
pub const ORD_LOAD_CURSOR_W: u32 = 683;
pub const ORD_CLIP_CURSOR: u32 = 731;
pub const ORD_GET_CLIP_CURSOR: u32 = 732;
pub const ORD_GET_CURSOR: u32 = 733;
pub const ORD_GET_CURSOR_POS: u32 = 734;
pub const ORD_SET_CURSOR_POS: u32 = 736;
pub const ORD_SHOW_CURSOR: u32 = 737;
pub const ORD_LOAD_ANIMATED_CURSOR: u32 = 1493;
pub const ORD_LOAD_IMAGE_W: u32 = 730;
pub const ORD_IMAGE_LIST_ADD: u32 = 738;
pub const ORD_IMAGE_LIST_ADD_MASKED: u32 = 739;
pub const ORD_IMAGE_LIST_BEGIN_DRAG: u32 = 740;
pub const ORD_IMAGE_LIST_COPY_DITHER_IMAGE: u32 = 741;
pub const ORD_IMAGE_LIST_CREATE: u32 = 742;
pub const ORD_IMAGE_LIST_DESTROY: u32 = 743;
pub const ORD_IMAGE_LIST_DRAG_ENTER: u32 = 744;
pub const ORD_IMAGE_LIST_DRAG_LEAVE: u32 = 745;
pub const ORD_IMAGE_LIST_DRAG_MOVE: u32 = 746;
pub const ORD_IMAGE_LIST_DRAG_SHOW_NOLOCK: u32 = 747;
pub const ORD_IMAGE_LIST_DRAW: u32 = 748;
pub const ORD_IMAGE_LIST_DRAW_EX: u32 = 749;
pub const ORD_IMAGE_LIST_DRAW_INDIRECT: u32 = 750;
pub const ORD_IMAGE_LIST_END_DRAG: u32 = 751;
pub const ORD_IMAGE_LIST_GET_BK_COLOR: u32 = 752;
pub const ORD_IMAGE_LIST_GET_DRAG_IMAGE: u32 = 753;
pub const ORD_IMAGE_LIST_GET_ICON: u32 = 754;
pub const ORD_IMAGE_LIST_GET_ICON_SIZE: u32 = 755;
pub const ORD_IMAGE_LIST_GET_IMAGE_COUNT: u32 = 756;
pub const ORD_IMAGE_LIST_GET_IMAGE_INFO: u32 = 757;
pub const ORD_IMAGE_LIST_LOAD_IMAGE: u32 = 758;
pub const ORD_IMAGE_LIST_MERGE: u32 = 759;
pub const ORD_IMAGE_LIST_REMOVE: u32 = 760;
pub const ORD_IMAGE_LIST_REPLACE: u32 = 761;
pub const ORD_IMAGE_LIST_REPLACE_ICON: u32 = 762;
pub const ORD_IMAGE_LIST_SET_BK_COLOR: u32 = 763;
pub const ORD_IMAGE_LIST_SET_DRAG_CURSOR_IMAGE: u32 = 764;
pub const ORD_IMAGE_LIST_SET_ICON_SIZE: u32 = 765;
pub const ORD_IMAGE_LIST_SET_OVERLAY_IMAGE: u32 = 766;
pub const ORD_IMAGE_LIST_COPY: u32 = 767;
pub const ORD_IMAGE_LIST_DUPLICATE: u32 = 768;
pub const ORD_IMAGE_LIST_SET_IMAGE_COUNT: u32 = 769;
pub const ORD_IMM_GET_CONTEXT: u32 = 783;
pub const ORD_IMM_GET_CONVERSION_STATUS: u32 = 785;
pub const ORD_IMM_NOTIFY_IME: u32 = 800;
pub const ORD_IMM_DISABLE_IME: u32 = 1206;
pub const ORD_IMM_ENABLE_IME: u32 = 1541;
pub const ORD_IMM_RELEASE_CONTEXT: u32 = 803;
pub const ORD_IMM_SET_CONVERSION_STATUS: u32 = 811;
pub const ORD_IMM_GET_COMPOSITION_STRING_W: u32 = 781;
pub const ORD_IMM_IS_IME: u32 = 1209;
pub const ORD_IMM_GET_KEYBOARD_LAYOUT: u32 = 1769;
pub const ORD_IMM_ASSOCIATE_CONTEXT: u32 = 770;
pub const ORD_IMM_GET_OPEN_STATUS: u32 = 792;
pub const ORD_IMM_SIPANEL_STATE: u32 = 804;
pub const ORD_IMM_ESCAPE_W: u32 = 775;
pub const ORD_IMM_CREATE_CONTEXT: u32 = 1198;
pub const ORD_IMM_DESTROY_CONTEXT: u32 = 1199;
pub const ORD_IMM_CONFIGURE_IMEW: u32 = 771;
pub const ORD_IMM_CREATE_IMCC: u32 = 772;
pub const ORD_IMM_DESTROY_IMCC: u32 = 773;
pub const ORD_IMM_ENUM_REGISTER_WORD_W: u32 = 774;
pub const ORD_IMM_GENERATE_MESSAGE: u32 = 776;
pub const ORD_IMM_GET_CANDIDATE_LIST_W: u32 = 777;
pub const ORD_IMM_GET_CANDIDATE_LIST_COUNT_W: u32 = 778;
pub const ORD_IMM_GET_CANDIDATE_WINDOW: u32 = 779;
pub const ORD_IMM_GET_COMPOSITION_FONT_W: u32 = 780;
pub const ORD_IMM_GET_COMPOSITION_WINDOW: u32 = 782;
pub const ORD_IMM_GET_CONVERSION_LIST_W: u32 = 784;
pub const ORD_IMM_GET_DEFAULT_IMEWND: u32 = 786;
pub const ORD_IMM_GET_DESCRIPTION_W: u32 = 787;
pub const ORD_IMM_GET_GUIDE_LINE_W: u32 = 788;
pub const ORD_IMM_GET_IMCCLOCK_COUNT: u32 = 789;
pub const ORD_IMM_GET_IMCCSIZE: u32 = 790;
pub const ORD_IMM_GET_IMCLOCK_COUNT: u32 = 791;
pub const ORD_IMM_GET_PROPERTY: u32 = 793;
pub const ORD_IMM_GET_REGISTER_WORD_STYLE_W: u32 = 794;
pub const ORD_IMM_IS_UIMESSAGE_W: u32 = 796;
pub const ORD_IMM_LOCK_IMC: u32 = 797;
pub const ORD_IMM_LOCK_IMCC: u32 = 798;
pub const ORD_IMM_RE_SIZE_IMCC: u32 = 801;
pub const ORD_IMM_REGISTER_WORD_W: u32 = 802;
pub const ORD_IMM_SET_ACTIVE_CONTEXT: u32 = 806;
pub const ORD_IMM_SET_CANDIDATE_WINDOW: u32 = 807;
pub const ORD_IMM_SET_COMPOSITION_FONT_W: u32 = 808;
pub const ORD_IMM_SET_COMPOSITION_STRING_W: u32 = 809;
pub const ORD_IMM_SET_COMPOSITION_WINDOW: u32 = 810;
pub const ORD_IMM_SET_OPEN_STATUS: u32 = 814;
pub const ORD_IMM_SET_STATUS_WINDOW_POS: u32 = 815;
pub const ORD_IMM_GET_STATUS_WINDOW_POS: u32 = 1200;
pub const ORD_IMM_SIMULATE_HOT_KEY: u32 = 816;
pub const ORD_IMM_UNLOCK_IMC: u32 = 817;
pub const ORD_IMM_UNLOCK_IMCC: u32 = 818;
pub const ORD_IMM_UNREGISTER_WORD_W: u32 = 819;
pub const ORD_IMM_ASSOCIATE_CONTEXT_EX: u32 = 1205;
pub const ORD_IMM_GET_IMEFILE_NAME_W: u32 = 1207;
pub const ORD_IMM_GET_VIRTUAL_KEY: u32 = 1210;
pub const ORD_IMM_GET_IME_MENU_ITEMS_W: u32 = 1211;
pub const ORD_DEFAULT_IMC_GET: u32 = 1218;
pub const ORD_DEFAULT_IME_WND_GET: u32 = 1219;
pub const ORD_IMM_PROCESS_KEY: u32 = 1220;
pub const ORD_IMM_TRANSLATE_MESSAGE: u32 = 1221;
pub const ORD_IMM_SET_IME_WND_IMC: u32 = 1222;
pub const ORD_IMM_GET_UICLASS_NAME: u32 = 1223;
pub const ORD_IMM_REQUEST_MESSAGE_W: u32 = 1242;
pub const ORD_IMM_SET_HOT_KEY: u32 = 812;
pub const ORD_IMM_GET_HOT_KEY: u32 = 813;
pub const ORD_IMM_ACTIVATE_LAYOUT: u32 = 1979;
pub const ORD_IMM_SEND_NOTIFICATION: u32 = 1980;
pub const ORD_IMM_GET_IMC_INFO: u32 = 2610;
pub const ORD_GET_MOUSE_MOVE_POINTS: u32 = 820;
pub const ORD_SEND_INPUT: u32 = 823;
pub const ORD_MOUSE_EVENT: u32 = 824;
pub const ORD_QASET_WINDOWS_JOURNAL_HOOK: u32 = 821;
pub const ORD_QAUNHOOK_WINDOWS_JOURNAL_HOOK: u32 = 822;
pub const ORD_ENABLE_HARDWARE_KEYBOARD: u32 = 825;
pub const ORD_GET_ASYNC_KEY_STATE: u32 = 826;
pub const ORD_GET_KEYBOARD_STATUS: u32 = 827;
pub const ORD_KEYBD_GET_DEVICE_INFO: u32 = 828;
pub const ORD_KEYBD_INIT_STATES: u32 = 829;
pub const ORD_KEYBD_VKEY_TO_UNICODE: u32 = 830;
pub const ORD_MAP_VIRTUAL_KEY_W: u32 = 831;
pub const ORD_POST_KEYBD_MESSAGE: u32 = 832;
pub const ORD_KEYBD_EVENT: u32 = 833;
pub const ORD_GET_ASYNC_SHIFT_FLAGS: u32 = 834;
pub const ORD_SET_WINDOWS_HOOK_EX_W: u32 = 1202;
pub const ORD_UNHOOK_WINDOWS_HOOK_EX: u32 = 1203;
pub const ORD_CALL_NEXT_HOOK_EX: u32 = 1204;
pub const ORD_REGISTER_HOT_KEY: u32 = 835;
pub const ORD_UNREGISTER_HOT_KEY: u32 = 836;
pub const ORD_UNREGISTER_FUNC1: u32 = 1156;
pub const ORD_ALL_KEYS: u32 = 1453;
pub const ORD_GET_KEYBOARD_TYPE: u32 = 1771;
pub const ORD_GET_KEYBOARD_LAYOUT_LIST: u32 = 1767;
pub const ORD_LOAD_KEYBOARD_LAYOUT_W: u32 = 1768;
pub const ORD_GET_KEYBOARD_LAYOUT: u32 = 1229;
pub const ORD_GET_KEYBOARD_LAYOUT_NAME_W: u32 = 1160;
pub const ORD_ACTIVATE_KEYBOARD_LAYOUT: u32 = 1766;
pub const ORD_SYSTEM_IDLE_TIMER_RESET: u32 = 837;
pub const ORD_TRANSLATE_ACCELERATOR_W: u32 = 838;
pub const ORD_NLED_GET_DEVICE_INFO: u32 = 839;
pub const ORD_NLED_SET_DEVICE: u32 = 840;
pub const ORD_INSERT_MENU_W: u32 = 841;
pub const ORD_APPEND_MENU_W: u32 = 842;
pub const ORD_REMOVE_MENU: u32 = 843;
pub const ORD_DESTROY_MENU: u32 = 844;
pub const ORD_TRACK_POPUP_MENU_EX: u32 = 845;
pub const ORD_LOAD_MENU_W: u32 = 846;
pub const ORD_ENABLE_MENU_ITEM: u32 = 847;
pub const ORD_CHECK_MENU_ITEM: u32 = 848;
pub const ORD_CHECK_MENU_RADIO_ITEM: u32 = 849;
pub const ORD_DELETE_MENU: u32 = 850;
pub const ORD_CREATE_MENU: u32 = 851;
pub const ORD_CREATE_POPUP_MENU: u32 = 852;
pub const ORD_SET_MENU_ITEM_INFO_W: u32 = 853;
pub const ORD_GET_MENU_ITEM_INFO_W: u32 = 854;
pub const ORD_GET_SUB_MENU: u32 = 855;
pub const ORD_DRAW_MENU_BAR: u32 = 856;
pub const ORD_SET_MENU: u32 = 2725;
pub const ORD_GET_MENU: u32 = 2726;
pub const ORD_MESSAGE_BEEP: u32 = 857;
pub const ORD_MESSAGE_BOX_W: u32 = 858;
pub const ORD_DISPATCH_MESSAGE_W: u32 = 859;
pub const ORD_GET_KEY_STATE: u32 = 860;
pub const ORD_GET_MESSAGE_W: u32 = 861;
pub const ORD_GET_MESSAGE_POS: u32 = 862;
pub const ORD_GET_MESSAGE_WNO_WAIT: u32 = 863;
pub const ORD_PEEK_MESSAGE_W: u32 = 864;
pub const ORD_POST_MESSAGE_W: u32 = 865;
pub const ORD_POST_QUIT_MESSAGE: u32 = 866;
pub const ORD_SEND_MESSAGE_W: u32 = 868;
pub const ORD_SEND_NOTIFY_MESSAGE_W: u32 = 869;
pub const ORD_TRANSLATE_MESSAGE: u32 = 870;
pub const ORD_MSG_WAIT_FOR_MULTIPLE_OBJECTS_EX: u32 = 871;
pub const ORD_GET_MESSAGE_SOURCE: u32 = 872;
pub const ORD_IN_SEND_MESSAGE: u32 = 1419;
pub const ORD_GET_QUEUE_STATUS: u32 = 1420;
pub const ORD_SEND_MESSAGE_TIMEOUT: u32 = 1495;
pub const ORD_GET_MESSAGE_QUEUE_READY_TIME_STAMP: u32 = 1477;
pub const ORD_ENABLE_GESTURES: u32 = 2899;
pub const ORD_DISABLE_GESTURES: u32 = 2900;
pub const ORD_QUERY_GESTURES: u32 = 2910;
pub const ORD_CLOSE_GESTURE_INFO_HANDLE: u32 = 2924;
pub const ORD_GET_GESTURE_INFO: u32 = 2925;
pub const ORD_GET_GESTURE_EXTRA_ARGUMENTS: u32 = 2926;
pub const ORD_GESTURE: u32 = 2927;
pub const ORD_REGISTER_GESTURE: u32 = 2724;
pub const ORD_REGISTER_DEFAULT_GESTURE_HANDLER: u32 = 2928;
pub const ORD_GET_WINDOW_AUTO_GESTURE: u32 = 2869;
pub const ORD_SET_WINDOW_AUTO_GESTURE: u32 = 2870;
pub const ORD_GET_ANIMATE_MESSAGE_INFO: u32 = 2871;
pub const ORD_LOAD_BITMAP_W: u32 = 873;
pub const ORD_SET_TIMER: u32 = 875;
pub const ORD_KILL_TIMER: u32 = 876;
pub const ORD_TOUCH_CALIBRATE: u32 = 877;
pub const ORD_GET_CLASS_INFO_W: u32 = 878;
pub const ORD_GET_CLASS_LONG_W: u32 = 879;
pub const ORD_SET_CLASS_LONG_W: u32 = 880;
pub const ORD_GET_CLASS_LONG: u32 = 881;
pub const ORD_SET_CLASS_LONG: u32 = 882;
pub const ORD_REGISTER_CLASS_W: u32 = 95;
pub const ORD_UNREGISTER_CLASS_W: u32 = 884;
pub const ORD_GET_SYSTEM_METRICS: u32 = 885;
pub const ORD_IS_WINDOW_VISIBLE: u32 = 886;
pub const ORD_GET_DC: u32 = 262;
pub const ORD_GET_WINDOW_DC: u32 = 270;
pub const ORD_RELEASE_DC: u32 = 263;
pub const ORD_ADJUST_WINDOW_RECT_EX: u32 = 887;
pub const ORD_GET_DOUBLE_CLICK_TIME: u32 = 888;
pub const ORD_GET_SYS_COLOR: u32 = 889;
pub const ORD_SET_SYS_COLORS: u32 = 890;
pub const ORD_REGISTER_WINDOW_MESSAGE_W: u32 = 891;
pub const ORD_REGISTER_TASK_BAR: u32 = 892;
pub const ORD_REGISTER_TASK_BAR_EX: u32 = 1506;
pub const ORD_REGISTER_DESKTOP: u32 = 1507;
pub const ORD_SET_PROP: u32 = 1497;
pub const ORD_GET_PROP: u32 = 1498;
pub const ORD_REMOVE_PROP: u32 = 1499;
pub const ORD_ENUM_PROPS_EX: u32 = 1500;
pub const ORD_GLOBAL_ADD_ATOM_W: u32 = 1519;
pub const ORD_GLOBAL_DELETE_ATOM: u32 = 1520;
pub const ORD_GLOBAL_FIND_ATOM_W: u32 = 1521;
pub const ORD_ADD_FONT_RESOURCE_W: u32 = 893;
pub const ORD_CE_REMOVE_FONT_RESOURCE: u32 = 894;
pub const ORD_CREATE_FONT_INDIRECT_W: u32 = 895;
pub const ORD_EXT_TEXT_OUT_W: u32 = 896;
pub const ORD_GET_TEXT_EXTENT_EX_POINT_W: u32 = 897;
pub const ORD_GET_TEXT_METRICS_W: u32 = 898;
pub const ORD_PEG_REMOVE_FONT_RESOURCE: u32 = 899;
pub const ORD_REMOVE_FONT_RESOURCE_W: u32 = 900;
pub const ORD_SET_TEXT_ALIGN: u32 = 1654;
pub const ORD_GET_TEXT_ALIGN: u32 = 1655;
pub const ORD_SET_TEXT_CHARACTER_EXTRA: u32 = 1962;
pub const ORD_GET_TEXT_CHARACTER_EXTRA: u32 = 1963;
pub const ORD_GET_CHAR_WIDTH32: u32 = 1664;
pub const ORD_GET_CHAR_ABCWIDTHS: u32 = 1779;
pub const ORD_GET_CHAR_ABCWIDTHS_I: u32 = 1887;
pub const ORD_GET_FONT_DATA: u32 = 1888;
pub const ORD_GET_OUTLINE_TEXT_METRICS_W: u32 = 1889;
pub const ORD_ADD_FONT_MEM_RESOURCE_EX: u32 = 2513;
pub const ORD_REMOVE_FONT_MEM_RESOURCE_EX: u32 = 2514;
pub const ORD_CREATE_BITMAP: u32 = 901;
pub const ORD_CREATE_COMPATIBLE_BITMAP: u32 = 902;
pub const ORD_SET_BITMAP_BITS: u32 = 1725;
pub const ORD_BIT_BLT: u32 = 903;
pub const ORD_MASK_BLT: u32 = 904;
pub const ORD_STRETCH_BLT: u32 = 905;
pub const ORD_GET_STRETCH_BLT_MODE: u32 = 1824;
pub const ORD_SET_STRETCH_BLT_MODE: u32 = 1825;
pub const ORD_TRANSPARENT_IMAGE: u32 = 906;
pub const ORD_STRETCH_DIBITS: u32 = 1667;
pub const ORD_SET_DIBITS_TO_DEVICE: u32 = 1726;
pub const ORD_RESTORE_DC: u32 = 907;
pub const ORD_SAVE_DC: u32 = 908;
pub const ORD_EXT_ESCAPE: u32 = 1182;
pub const ORD_CREATE_DCW: u32 = 909;
pub const ORD_CREATE_COMPATIBLE_DC: u32 = 910;
pub const ORD_DELETE_DC: u32 = 911;
pub const ORD_DELETE_OBJECT: u32 = 912;
pub const ORD_GET_BK_COLOR: u32 = 913;
pub const ORD_GET_BK_MODE: u32 = 914;
pub const ORD_GET_CURRENT_OBJECT: u32 = 915;
pub const ORD_GET_DEVICE_CAPS: u32 = 916;
pub const ORD_GET_OBJECT_TYPE: u32 = 917;
pub const ORD_GET_OBJECT_W: u32 = 918;
pub const ORD_GET_STOCK_OBJECT: u32 = 919;
pub const ORD_GET_TEXT_COLOR: u32 = 920;
pub const ORD_SELECT_OBJECT: u32 = 921;
pub const ORD_SET_BK_COLOR: u32 = 922;
pub const ORD_SET_BK_MODE: u32 = 923;
pub const ORD_SET_TEXT_COLOR: u32 = 924;
pub const ORD_GET_DIBCOLOR_TABLE: u32 = 1665;
pub const ORD_SET_DIBCOLOR_TABLE: u32 = 1666;
pub const ORD_ENUM_DISPLAY_SETTINGS: u32 = 1777;
pub const ORD_ENUM_DISPLAY_DEVICES: u32 = 1778;
pub const ORD_SET_LAYOUT: u32 = 1890;
pub const ORD_GET_LAYOUT: u32 = 1891;
pub const ORD_CREATE_PATTERN_BRUSH: u32 = 925;
pub const ORD_CREATE_PEN: u32 = 926;
pub const ORD_FILL_RGN: u32 = 927;
pub const ORD_SET_ROP2: u32 = 928;
pub const ORD_GET_ROP2: u32 = 1990;
pub const ORD_SET_WINDOW_ORG_EX: u32 = 1984;
pub const ORD_GET_WINDOW_ORG_EX: u32 = 1985;
pub const ORD_GET_WINDOW_EXT_EX: u32 = 1986;
pub const ORD_OFFSET_VIEWPORT_ORG_EX: u32 = 1987;
pub const ORD_GET_VIEWPORT_ORG_EX: u32 = 1988;
pub const ORD_GET_VIEWPORT_EXT_EX: u32 = 1989;
pub const ORD_CREATE_DIBPATTERN_BRUSH_PT: u32 = 929;
pub const ORD_CREATE_PEN_INDIRECT: u32 = 930;
pub const ORD_CREATE_SOLID_BRUSH: u32 = 931;
pub const ORD_DRAW_EDGE: u32 = 932;
pub const ORD_DRAW_FOCUS_RECT: u32 = 933;
pub const ORD_ELLIPSE: u32 = 934;
pub const ORD_FILL_RECT: u32 = 935;
pub const ORD_GET_PIXEL: u32 = 936;
pub const ORD_GET_SYS_COLOR_BRUSH: u32 = 937;
pub const ORD_PAT_BLT: u32 = 938;
pub const ORD_INVERT_RECT: u32 = 1770;
pub const ORD_POLYGON: u32 = 939;
pub const ORD_POLYLINE: u32 = 940;
pub const ORD_RECTANGLE: u32 = 941;
pub const ORD_ROUND_RECT: u32 = 942;
pub const ORD_SET_BRUSH_ORG_EX: u32 = 943;
pub const ORD_SET_PIXEL: u32 = 944;
pub const ORD_MOVE_TO_EX: u32 = 1651;
pub const ORD_LINE_TO: u32 = 1652;
pub const ORD_GET_CURRENT_POSITION_EX: u32 = 1653;
pub const ORD_DRAW_TEXT_W: u32 = 945;
pub const ORD_CREATE_BITMAP_FROM_POINTER: u32 = 946;
pub const ORD_CREATE_PALETTE: u32 = 947;
pub const ORD_GET_NEAREST_PALETTE_INDEX: u32 = 948;
pub const ORD_GET_PALETTE_ENTRIES: u32 = 949;
pub const ORD_GET_SYSTEM_PALETTE_ENTRIES: u32 = 950;
pub const ORD_SET_PALETTE_ENTRIES: u32 = 951;
pub const ORD_GET_NEAREST_COLOR: u32 = 952;
pub const ORD_REALIZE_PALETTE: u32 = 953;
pub const ORD_SELECT_PALETTE: u32 = 954;
pub const ORD_GRADIENT_FILL: u32 = 1763;
pub const ORD_ALPHA_BLEND: u32 = 1883;
pub const ORD_ABORT_DOC: u32 = 955;
pub const ORD_CLOSE_ENH_META_FILE: u32 = 956;
pub const ORD_CREATE_ENH_META_FILE_W: u32 = 957;
pub const ORD_DELETE_ENH_META_FILE: u32 = 958;
pub const ORD_END_DOC: u32 = 959;
pub const ORD_END_PAGE: u32 = 960;
pub const ORD_PLAY_ENH_META_FILE: u32 = 961;
pub const ORD_SET_ABORT_PROC: u32 = 962;
pub const ORD_START_DOC_W: u32 = 963;
pub const ORD_START_PAGE: u32 = 964;
pub const ORD_ENUM_FONT_FAMILIES_EX_W: u32 = 1885;
pub const ORD_ENUM_FONT_FAMILIES_W: u32 = 965;
pub const ORD_ENUM_FONTS_W: u32 = 966;
pub const ORD_GET_TEXT_FACE_W: u32 = 967;
pub const ORD_TRANSLATE_CHARSET_INFO: u32 = 1166;
pub const ORD_COMBINE_RGN: u32 = 968;
pub const ORD_CREATE_RECT_RGN_INDIRECT: u32 = 969;
pub const ORD_EXCLUDE_CLIP_RECT: u32 = 970;
pub const ORD_GET_CLIP_BOX: u32 = 971;
pub const ORD_GET_CLIP_RGN: u32 = 972;
pub const ORD_GET_REGION_DATA: u32 = 973;
pub const ORD_GET_RGN_BOX: u32 = 974;
pub const ORD_INTERSECT_CLIP_RECT: u32 = 975;
pub const ORD_OFFSET_RGN: u32 = 976;
pub const ORD_PT_IN_REGION: u32 = 977;
pub const ORD_RECT_IN_REGION: u32 = 978;
pub const ORD_SELECT_CLIP_RGN: u32 = 979;
pub const ORD_CREATE_RECT_RGN: u32 = 980;
pub const ORD_RECT_VISIBLE: u32 = 981;
pub const ORD_SET_RECT_RGN: u32 = 982;
pub const ORD_EXT_CREATE_REGION: u32 = 1617;
pub const ORD_SET_VIEWPORT_ORG_EX: u32 = 983;
pub const ORD_SET_OBJECT_OWNER: u32 = 984;
pub const ORD_SCROLL_DC: u32 = 985;
pub const ORD_ENABLE_EUDC: u32 = 986;
pub const ORD_DRAW_FRAME_CONTROL: u32 = 987;
pub const ORD_SET_SCROLL_INFO: u32 = 279;
pub const ORD_SET_SCROLL_POS: u32 = 280;
pub const ORD_SET_SCROLL_RANGE: u32 = 281;
pub const ORD_GET_SCROLL_INFO: u32 = 282;
pub const ORD_STRING_CCH_COPY_A: u32 = 1705;
pub const ORD_STRING_CB_COPY_A: u32 = 1706;
pub const ORD_STRING_CCH_COPY_EX_A: u32 = 1707;
pub const ORD_STRING_CB_COPY_EX_A: u32 = 1708;
pub const ORD_STRING_CCH_COPY_NA: u32 = 1750;
pub const ORD_STRING_CB_COPY_NA: u32 = 1751;
pub const ORD_STRING_CCH_CAT_A: u32 = 1709;
pub const ORD_STRING_CB_CAT_A: u32 = 1710;
pub const ORD_STRING_CCH_CAT_EX_A: u32 = 1711;
pub const ORD_STRING_CB_CAT_EX_A: u32 = 1712;
pub const ORD_STRING_CCH_CAT_NA: u32 = 1752;
pub const ORD_STRING_CB_CAT_NA: u32 = 1753;
pub const ORD_STRING_CCH_CAT_NEX_A: u32 = 1754;
pub const ORD_STRING_CB_CAT_NEX_A: u32 = 1755;
pub const ORD_STRING_CCH_VPRINTF_A: u32 = 1713;
pub const ORD_STRING_CB_VPRINTF_A: u32 = 1714;
pub const ORD_STRING_CCH_PRINTF_A: u32 = 1715;
pub const ORD_STRING_CB_PRINTF_A: u32 = 1716;
pub const ORD_STRING_CCH_PRINTF_EX_A: u32 = 1717;
pub const ORD_STRING_CB_PRINTF_EX_A: u32 = 1718;
pub const ORD_STRING_CCH_VPRINTF_EX_A: u32 = 1719;
pub const ORD_STRING_CB_VPRINTF_EX_A: u32 = 1720;
pub const ORD_STRING_CCH_LENGTH_A: u32 = 1756;
pub const ORD_STRING_CB_LENGTH_A: u32 = 1757;
pub const ORD_GET_OVERLAPPED_RESULT: u32 = 1188;
pub const ORD_MONITOR_FROM_POINT: u32 = 1522;
pub const ORD_MONITOR_FROM_RECT: u32 = 1523;
pub const ORD_MONITOR_FROM_WINDOW: u32 = 1524;
pub const ORD_GET_MONITOR_INFO: u32 = 1525;
pub const ORD_ENUM_DISPLAY_MONITORS: u32 = 1526;
pub const ORD_A_SHAINIT: u32 = 1789;
pub const ORD_A_SHAUPDATE: u32 = 1790;
pub const ORD_A_SHAFINAL: u32 = 1791;
pub const ORD_MD5_INIT: u32 = 1792;
pub const ORD_MD5_UPDATE: u32 = 1793;
pub const ORD_MD5_FINAL: u32 = 1794;
pub const ORD_GET_STDIO_PATH_W: u32 = 1149;
pub const ORD_SET_STDIO_PATH_W: u32 = 1150;
pub const ORD_REGISTRY_GET_DWORD: u32 = 2615;
pub const ORD_REGISTRY_GET_STRING: u32 = 2616;
pub const ORD_REGISTRY_SET_DWORD: u32 = 2617;
pub const ORD_REGISTRY_SET_STRING: u32 = 2618;
pub const ORD_REGISTRY_TEST_EXCHANGE_DWORD: u32 = 2619;
pub const ORD_REGISTRY_NOTIFY_APP: u32 = 2620;
pub const ORD_REGISTRY_NOTIFY_WINDOW: u32 = 2621;
pub const ORD_REGISTRY_NOTIFY_MSG_QUEUE: u32 = 2622;
pub const ORD_REGISTRY_NOTIFY_CALLBACK: u32 = 2623;
pub const ORD_REGISTRY_CLOSE_NOTIFICATION: u32 = 2624;
pub const ORD_REGISTRY_STOP_NOTIFICATION: u32 = 2625;
pub const ORD_REGISTRY_BATCH_NOTIFICATION: u32 = 2626;
pub const ORD_REGISTRY_DELETE_VALUE: u32 = 2627;
pub const ORD_ABS: u32 = 988;
pub const ORD_ACOS: u32 = 989;
pub const ORD_ASIN: u32 = 990;
pub const ORD_ATAN: u32 = 991;
pub const ORD_ATAN2: u32 = 992;
pub const ORD_CEIL: u32 = 999;
pub const ORD_COS: u32 = 1004;
pub const ORD_COSH: u32 = 1005;
pub const ORD_DIV: u32 = 1007;
pub const ORD_EXP: u32 = 1009;
pub const ORD_FABS: u32 = 1010;
pub const ORD_FLOOR: u32 = 1013;
pub const ORD_FMOD: u32 = 1014;
pub const ORD_FMODF: u32 = 1643;
pub const ORD_FREXP: u32 = 1019;
pub const ORD_HYPOT: u32 = 1023;
pub const ORD_LABS: u32 = 1030;
pub const ORD_LDEXP: u32 = 1031;
pub const ORD_LDIV: u32 = 1032;
pub const ORD_LOG: u32 = 1033;
pub const ORD_LOG10: u32 = 1034;
pub const ORD_MODF: u32 = 1048;
pub const ORD_POW: u32 = 1051;
pub const ORD_SIN: u32 = 1058;
pub const ORD_SINH: u32 = 1059;
pub const ORD_SQRT: u32 = 1060;
pub const ORD_TAN: u32 = 1075;
pub const ORD_TANH: u32 = 1076;
pub const ORD_LL_RSHIFT: u32 = 2002;
pub const ORD_LL_LSHIFT: u32 = 2003;
pub const ORD_LL_MUL: u32 = 2004;
pub const ORD_LL_DIV: u32 = 2005;
pub const ORD_LL_REM: u32 = 2006;
pub const ORD_ULL_RSHIFT: u32 = 2011;
pub const ORD_ULL_DIV: u32 = 2012;
pub const ORD_ULL_REM: u32 = 2013;
pub const ORD_F_TO_LL: u32 = 2018;
pub const ORD_D_TO_LL: u32 = 2019;
pub const ORD_F_TO_ULL: u32 = 2020;
pub const ORD_D_TO_ULL: u32 = 2021;
pub const ORD_FPADD: u32 = 2022;
pub const ORD_DPADD: u32 = 2023;
pub const ORD_FPSUB: u32 = 2024;
pub const ORD_DPSUB: u32 = 2025;
pub const ORD_FPMUL: u32 = 2026;
pub const ORD_DPMUL: u32 = 2027;
pub const ORD_FPDIV: u32 = 2028;
pub const ORD_DPDIV: u32 = 2029;
pub const ORD_FPTOLI: u32 = 2030;
pub const ORD_FPTOUL: u32 = 2031;
pub const ORD_LITOFP: u32 = 2032;
pub const ORD_ULTOFP: u32 = 2033;
pub const ORD_DPTOLI: u32 = 2034;
pub const ORD_DPTOUL: u32 = 2035;
pub const ORD_LITODP: u32 = 2036;
pub const ORD_ULTODP: u32 = 2037;
pub const ORD_FPTODP: u32 = 2038;
pub const ORD_DPTOFP: u32 = 2039;
pub const ORD_FPCMP: u32 = 2040;
pub const ORD_DPCMP: u32 = 2041;
pub const ORD_LTS: u32 = 2042;
pub const ORD_LES: u32 = 2043;
pub const ORD_EQS: u32 = 2044;
pub const ORD_GES: u32 = 2045;
pub const ORD_GTS: u32 = 2046;
pub const ORD_NES: u32 = 2047;
pub const ORD_LTD: u32 = 2048;
pub const ORD_LED: u32 = 2049;
pub const ORD_EQD: u32 = 2050;
pub const ORD_GED: u32 = 2051;
pub const ORD_GTD: u32 = 2052;
pub const ORD_NED: u32 = 2053;

// CRT narrow string
pub const ORD_MEMCHR: u32 = 31;
pub const ORD_ATOL: u32 = 994;
pub const ORD_STRLEN: u32 = 1068;
pub const ORD_STRCMP: u32 = 1065;
pub const ORD_STRCHR: u32 = 1064;
pub const ORD_STRCSPN: u32 = 1067;
pub const ORD_STRNCMP: u32 = 1070;
pub const ORD_STRNCPY: u32 = 1071;
pub const ORD_STRSTR: u32 = 1072;
pub const ORD_STRNCAT: u32 = 1069;
pub const ORD_STRSPN: u32 = 1408;
pub const ORD_STRPBRK: u32 = 1406;
pub const ORD_STRRCHR: u32 = 1407;
pub const ORD_STRDUP: u32 = 1409;
pub const ORD_STRICMP: u32 = 1410;
pub const ORD_STRNICMP: u32 = 1411;
pub const ORD_STRNSET: u32 = 1412;
pub const ORD_STRREV: u32 = 1413;
pub const ORD_STRSET: u32 = 1414;
pub const ORD_STRLWR: u32 = 1415;
pub const ORD_STRTOL: u32 = 1404;
pub const ORD_STRTOD: u32 = 1403;
pub const ORD_ATOI64: u32 = 1418;
pub const ORD_ITOA: u32 = 1025;
pub const ORD_ITOW: u32 = 1026;
pub const ORD_LTOA: u32 = 1039;
pub const ORD_LTOW: u32 = 1040;
pub const ORD_ULTOA: u32 = 1079;
pub const ORD_ULTOW: u32 = 1080;
pub const ORD_MEMCCPY: u32 = 1042;
pub const ORD_MEMICMP: u32 = 1045;
pub const ORD_SWAB: u32 = 1074;
pub const ORD_ISCTYPE: u32 = 1417;
// CRT wide string extras
pub const ORD_WCSCAT: u32 = 58;
pub const ORD_WCSCMP: u32 = 60;
pub const ORD_WCSCSPN: u32 = 62;
pub const ORD_WCSNCAT: u32 = 64;
pub const ORD_WCSNSET: u32 = 67;
pub const ORD_WCSREV: u32 = 70;
pub const ORD_WCSSET: u32 = 71;
pub const ORD_WCSSPN: u32 = 72;
pub const ORD_WCSTOK: u32 = 77;
pub const ORD_WTOLL: u32 = 79;
pub const ORD_TOWLOWER: u32 = 194;
pub const ORD_TOWUPPER: u32 = 195;
pub const ORD_WCSLWR: u32 = 231;
pub const ORD_WCSUPR: u32 = 232;
pub const ORD_WCSTOD: u32 = 1081;
pub const ORD_WCSTOL: u32 = 1082;
// Bit / integer ops
pub const ORD_ROTL: u32 = 1055;
pub const ORD_ROTR: u32 = 1056;
pub const ORD_LROTL: u32 = 1037;
pub const ORD_LROTR: u32 = 1038;
pub const ORD_BYTESWAP_USHORT: u32 = 1624;
pub const ORD_BYTESWAP_ULONG: u32 = 1623;
pub const ORD_BYTESWAP_UINT64: u32 = 1622;
pub const ORD_COUNT_LEADING_ONES: u32 = 1625;
pub const ORD_COUNT_LEADING_ONES64: u32 = 1626;
pub const ORD_COUNT_LEADING_SIGNS: u32 = 1627;
pub const ORD_COUNT_LEADING_SIGNS64: u32 = 1628;
pub const ORD_COUNT_LEADING_ZEROS: u32 = 1629;
pub const ORD_COUNT_LEADING_ZEROS64: u32 = 1630;
pub const ORD_COUNT_ONE_BITS: u32 = 1631;
pub const ORD_COUNT_ONE_BITS64: u32 = 1632;
pub const ORD_MUL_HIGH: u32 = 1636;
pub const ORD_MUL_UNSIGNED_HIGH: u32 = 1637;
pub const ORD_ROTL64: u32 = 1638;
pub const ORD_ROTR64: u32 = 1639;
pub const ORD_ABS64: u32 = 1621;
// Float math extras
pub const ORD_CEILF: u32 = 1640;
pub const ORD_FABSF: u32 = 1641;
pub const ORD_FLOORF: u32 = 1642;
pub const ORD_SQRTF: u32 = 1644;
pub const ORD_FINITE: u32 = 1012;
pub const ORD_ISNAN: u32 = 1024;
pub const ORD_ISNANF: u32 = 1633;
pub const ORD_ISUNORDERED: u32 = 1634;
pub const ORD_ISUNORDEREDF: u32 = 1635;
pub const ORD_FRND: u32 = 1020;
pub const ORD_FSQRT: u32 = 1021;
pub const ORD_FPIEEE_FLT: u32 = 1016;
pub const ORD_FPCLASS: u32 = 1015;
pub const ORD_COPYSIGN: u32 = 1003;
pub const ORD_CHGSIGN: u32 = 1000;
pub const ORD_LOGB: u32 = 1035;
pub const ORD_SCALB: u32 = 1057;
pub const ORD_NEXTAFTER: u32 = 1050;
pub const ORD_DIFFTIME: u32 = 1006;
pub const ORD_CABS: u32 = 998;
pub const ORD_J0: u32 = 1027;
pub const ORD_J1: u32 = 1028;
pub const ORD_JN: u32 = 1029;
pub const ORD_Y0: u32 = 1084;
pub const ORD_Y1: u32 = 1085;
pub const ORD_YN: u32 = 1086;
pub const ORD_ECVT: u32 = 1008;
pub const ORD_FCVT: u32 = 1011;
pub const ORD_GCVT: u32 = 1022;
pub const ORD_ATODBL: u32 = 996;
pub const ORD_ATOFLT: u32 = 997;
pub const ORD_CLEARFP: u32 = 1001;
pub const ORD_CONTROLFP: u32 = 1002;
pub const ORD_STATUSFP: u32 = 1062;
pub const ORD_FPRESET: u32 = 1017;
pub const ORD_LD12TOD: u32 = 1087;
pub const ORD_LD12TOF: u32 = 1088;
pub const ORD_STRGTOLD12: u32 = 1089;
// MIPS 64-bit helpers
pub const ORD_LL_BIT_EXTRACT: u32 = 2007;
pub const ORD_LL_BIT_INSERT: u32 = 2008;
pub const ORD_LL_TO_F: u32 = 2009;
pub const ORD_LL_TO_D: u32 = 2010;
pub const ORD_ULL_BIT_EXTRACT: u32 = 2014;
pub const ORD_ULL_BIT_INSERT: u32 = 2015;
pub const ORD_ULL_TO_F: u32 = 2016;
pub const ORD_ULL_TO_D: u32 = 2017;
// Internal / kernel stubs
pub const ORD_SYSTEM_STARTED: u32 = 1;
pub const ORD_CLOSE_ALL_DEVICE_HANDLES: u32 = 244;
pub const ORD_CREATE_DEVICE_HANDLE: u32 = 245;
pub const ORD_THGROW: u32 = 512;
pub const ORD_THREAD_ATTACH_ALL_DLLS: u32 = 561;
pub const ORD_THREAD_DETACH_ALL_DLLS: u32 = 562;
pub const ORD_PROCESS_DETACH_ALL_DLLS: u32 = 572;
pub const ORD_CLOSE_PROC_OE: u32 = 589;
pub const ORD_TAKE_CRIT_SEC: u32 = 596;
pub const ORD_LEAVE_CRIT_SEC: u32 = 597;
pub const ORD_OTHER_THREADS_RUNNING: u32 = 604;
pub const ORD_CREATE_CRIT: u32 = 616;
pub const ORD_NK_TERMINATE_THREAD: u32 = 623;
pub const ORD_SSCANF: u32 = 653;
pub const ORD_VFWPRINTF: u32 = 721;
pub const ORD_FWSCANF: u32 = 735;
pub const ORD_FWPRINTF: u32 = 867;
pub const ORD_PURECALL: u32 = 1092;
pub const ORD_FLTUSED: u32 = 1093;
pub const ORD_SWSCANF: u32 = 1098;
pub const ORD_GETSTDFILEX: u32 = 1100;
pub const ORD_SCANF: u32 = 1101;
pub const ORD_VPRINTF: u32 = 1103;
pub const ORD_GETCHAR: u32 = 1104;
pub const ORD_PUTCHAR: u32 = 1105;
pub const ORD_GETS: u32 = 1106;
pub const ORD_PUTS: u32 = 1107;
pub const ORD_FGETC: u32 = 1108;
pub const ORD_FPUTC: u32 = 1110;
pub const ORD_FPUTS: u32 = 1111;
pub const ORD_UNGETC: u32 = 1112;
pub const ORD_FSCANF: u32 = 1114;
pub const ORD_FPRINTF: u32 = 1115;
pub const ORD_VFPRINTF: u32 = 1116;
pub const ORD_WFDOPEN: u32 = 1117;
pub const ORD_FCLOSEALL: u32 = 1119;
pub const ORD_FLUSHALL: u32 = 1123;
pub const ORD_FILENO: u32 = 1124;
pub const ORD_CLEARERR: u32 = 1127;
pub const ORD_FGETPOS: u32 = 1128;
pub const ORD_FSETPOS: u32 = 1129;
pub const ORD_WSCANF: u32 = 1133;
pub const ORD_WPRINTF: u32 = 1134;
pub const ORD_VWPRINTF: u32 = 1135;
pub const ORD_GETWCHAR: u32 = 1136;
pub const ORD_PUTWCHAR: u32 = 1137;
pub const ORD_GETWS: u32 = 1138;
pub const ORD_PUTWS: u32 = 1139;
pub const ORD_FGETWC: u32 = 1140;
pub const ORD_FPUTWC: u32 = 1141;
pub const ORD_UNGETWC: u32 = 1142;
pub const ORD_FGETWS: u32 = 1143;
pub const ORD_FPUTWS: u32 = 1144;
pub const ORD_INITSTDIOLIB: u32 = 1151;
pub const ORD_HUGE: u32 = 1181;
pub const ORD_SETMODE: u32 = 1187;
pub const ORD_WFREOPEN: u32 = 1201;
pub const ORD_GET_CRT_STORAGE_EX: u32 = 1227;
pub const ORD_CALLOC: u32 = 1346;
pub const ORD_GET_PRIVATE_CALLBACKS: u32 = 1400;
pub const ORD_SETVBUF: u32 = 1608;
pub const ORD_QUERY_NEW_HANDLER: u32 = 1618;
pub const ORD_SET_NEW_HANDLER: u32 = 1619;
pub const ORD_OPERATOR_NEW_NOTHROW: u32 = 1646;
pub const ORD_OPERATOR_DELETE_NOTHROW: u32 = 1662;
pub const ORD_CXX_FRAME_HANDLER: u32 = 1550;
pub const ORD_CXX_THROW_EXCEPTION: u32 = 1551;
pub const ORD_SET_TERMINATE: u32 = 1552;
pub const ORD_SET_UNEXPECTED: u32 = 1553;
pub const ORD_SET_INCONSISTENCY: u32 = 1555;
pub const ORD_STD_TERMINATE: u32 = 1556;
pub const ORD_STD_UNEXPECTED: u32 = 1557;
pub const ORD_STD_INCONSISTENCY: u32 = 1558;
pub const ORD_XCPT_FILTER: u32 = 1645;
pub const ORD_NOTHROW_OBJ: u32 = 1647;
pub const ORD_SET_NEW_MODE: u32 = 1648;
pub const ORD_QUERY_NEW_MODE: u32 = 1649;
pub const ORD_SET_NEW_HANDLER2: u32 = 1650;
pub const ORD_STD_XLEN: u32 = 1658;
pub const ORD_STD_XRAN: u32 = 1659;
pub const ORD_STD_NOMEMORY: u32 = 1660;
pub const ORD_INT_CREATE_EVENT_W: u32 = 1761;
pub const ORD_INT_CLOSE_HANDLE: u32 = 1762;
pub const ORD_RTLCHECKSTACK: u32 = 2001;
pub const ORD_C_SPECIFIC_HANDLER: u32 = 87;
pub const ORD_SET_WDEVICE_POWER_HANDLER: u32 = 1178;

pub const SDK_ORDINALS: &[CoredllOrdinalDef; 27] = &[
    CoredllOrdinalDef {
        name: "wcsrchr",
        ordinal: ORD_WCSRCHR,
    },
    CoredllOrdinalDef {
        name: "_wcsdup",
        ordinal: ORD_WCSDUP,
    },
    CoredllOrdinalDef {
        name: "_wtol",
        ordinal: ORD_WTOL,
    },
    CoredllOrdinalDef {
        name: "_wcsicmp",
        ordinal: ORD_WCSICMP,
    },
    CoredllOrdinalDef {
        name: "_wcsnicmp",
        ordinal: ORD_WCSNICMP,
    },
    CoredllOrdinalDef {
        name: "wcsncpy",
        ordinal: ORD_WCSNCPY,
    },
    CoredllOrdinalDef {
        name: "malloc",
        ordinal: ORD_MALLOC,
    },
    CoredllOrdinalDef {
        name: "memcpy",
        ordinal: ORD_MEMCPY,
    },
    CoredllOrdinalDef {
        name: "memmove",
        ordinal: ORD_MEMMOVE,
    },
    CoredllOrdinalDef {
        name: "memset",
        ordinal: ORD_MEMSET,
    },
    CoredllOrdinalDef {
        name: "atoi",
        ordinal: ORD_ATOI,
    },
    CoredllOrdinalDef {
        name: "strcat",
        ordinal: ORD_STRCAT,
    },
    CoredllOrdinalDef {
        name: "strcpy",
        ordinal: ORD_STRCPY,
    },
    CoredllOrdinalDef {
        name: "strtok",
        ordinal: ORD_STRTOK,
    },
    CoredllOrdinalDef {
        name: "??3@YAXPAX@Z",
        ordinal: ORD_OPERATOR_DELETE,
    },
    CoredllOrdinalDef {
        name: "??2@YAPAXI@Z",
        ordinal: ORD_OPERATOR_NEW,
    },
    CoredllOrdinalDef {
        name: "??_U@YAPAXI@Z",
        ordinal: ORD_OPERATOR_NEW_ARRAY,
    },
    CoredllOrdinalDef {
        name: "??_V@YAXPAX@Z",
        ordinal: ORD_OPERATOR_DELETE_ARRAY,
    },
    CoredllOrdinalDef {
        name: "??_U@YAPAXIABUnothrow_t@std@@@Z",
        ordinal: ORD_OPERATOR_NEW_ARRAY_NOTHROW,
    },
    CoredllOrdinalDef {
        name: "??_V@YAXPAXABUnothrow_t@std@@@Z",
        ordinal: ORD_OPERATOR_DELETE_ARRAY_NOTHROW,
    },
    CoredllOrdinalDef {
        name: "swprintf",
        ordinal: ORD_SWPRINTF,
    },
    CoredllOrdinalDef {
        name: "printf",
        ordinal: ORD_PRINTF,
    },
    CoredllOrdinalDef {
        name: "fgets",
        ordinal: ORD_FGETS,
    },
    CoredllOrdinalDef {
        name: "_wfopen",
        ordinal: ORD_WFOPEN,
    },
    CoredllOrdinalDef {
        name: "free",
        ordinal: ORD_FREE,
    },
    CoredllOrdinalDef {
        name: "longjmp",
        ordinal: ORD_LONGJMP,
    },
    CoredllOrdinalDef {
        name: "_setjmp",
        ordinal: ORD_SETJMP,
    },
];

pub const SUPPLEMENTAL_ORDINALS: &[CoredllOrdinalDef; 97] = &[
    // Originally registered
    CoredllOrdinalDef {
        name: "GetSystemTimeAsFileTime",
        ordinal: ORD_GET_SYSTEM_TIME_AS_FILE_TIME,
    },
    CoredllOrdinalDef {
        name: "GetProcessId",
        ordinal: ORD_GET_PROCESS_ID,
    },
    CoredllOrdinalDef {
        name: "GetThreadId",
        ordinal: ORD_GET_THREAD_ID,
    },
    CoredllOrdinalDef {
        name: "RegisterGesture",
        ordinal: ORD_REGISTER_GESTURE,
    },
    CoredllOrdinalDef {
        name: "SetMenu",
        ordinal: ORD_SET_MENU,
    },
    CoredllOrdinalDef {
        name: "GetMenu",
        ordinal: ORD_GET_MENU,
    },
    CoredllOrdinalDef {
        name: "CeFsIoControlW",
        ordinal: ORD_CE_FS_IO_CONTROL_W,
    },
    CoredllOrdinalDef {
        name: "AFS_FsIoControlW",
        ordinal: ORD_AFS_FS_IO_CONTROL_W,
    },
    CoredllOrdinalDef {
        name: "AFS_SetFileSecurityW",
        ordinal: ORD_AFS_SET_FILE_SECURITY_W,
    },
    CoredllOrdinalDef {
        name: "AFS_GetFileSecurityW",
        ordinal: ORD_AFS_GET_FILE_SECURITY_W,
    },
    CoredllOrdinalDef {
        name: "SetFileSecurityW",
        ordinal: ORD_SET_FILE_SECURITY_W,
    },
    CoredllOrdinalDef {
        name: "GetFileSecurityW",
        ordinal: ORD_GET_FILE_SECURITY_W,
    },
    CoredllOrdinalDef {
        name: "LockFileEx",
        ordinal: ORD_LOCK_FILE_EX,
    },
    CoredllOrdinalDef {
        name: "UnlockFileEx",
        ordinal: ORD_UNLOCK_FILE_EX,
    },
    CoredllOrdinalDef {
        name: "CeGetVolumeInfoW",
        ordinal: ORD_CE_GET_VOLUME_INFO_W,
    },
    CoredllOrdinalDef {
        name: "RegistryGetDword",
        ordinal: ORD_REGISTRY_GET_DWORD,
    },
    CoredllOrdinalDef {
        name: "RegistryGetString",
        ordinal: ORD_REGISTRY_GET_STRING,
    },
    CoredllOrdinalDef {
        name: "RegistrySetDword",
        ordinal: ORD_REGISTRY_SET_DWORD,
    },
    CoredllOrdinalDef {
        name: "RegistrySetString",
        ordinal: ORD_REGISTRY_SET_STRING,
    },
    CoredllOrdinalDef {
        name: "RegistryDeleteValue",
        ordinal: ORD_REGISTRY_DELETE_VALUE,
    },
    CoredllOrdinalDef {
        name: "RegistryTestExchangeDword",
        ordinal: ORD_REGISTRY_TEST_EXCHANGE_DWORD,
    },
    CoredllOrdinalDef {
        name: "RegistryNotifyWindow",
        ordinal: ORD_REGISTRY_NOTIFY_WINDOW,
    },
    CoredllOrdinalDef {
        name: "RegistryNotifyMsgQueue",
        ordinal: ORD_REGISTRY_NOTIFY_MSG_QUEUE,
    },
    CoredllOrdinalDef {
        name: "CeFindFirstRegChange",
        ordinal: ORD_CE_FIND_FIRST_REG_CHANGE,
    },
    CoredllOrdinalDef {
        name: "CeFindNextRegChange",
        ordinal: ORD_CE_FIND_NEXT_REG_CHANGE,
    },
    CoredllOrdinalDef {
        name: "CeFindCloseRegChange",
        ordinal: ORD_CE_FIND_CLOSE_REG_CHANGE,
    },
    CoredllOrdinalDef {
        name: "BatteryGetLifeTimeInfo",
        ordinal: ORD_BATTERY_GET_LIFE_TIME_INFO,
    },
    CoredllOrdinalDef {
        name: "BatteryNotifyOfTimeChange",
        ordinal: ORD_BATTERY_NOTIFY_OF_TIME_CHANGE,
    },
    CoredllOrdinalDef {
        name: "BatteryDrvrGetLevels",
        ordinal: ORD_BATTERY_DRVR_GET_LEVELS,
    },
    CoredllOrdinalDef {
        name: "BatteryDrvrSupportsChangeNotification",
        ordinal: ORD_BATTERY_DRVR_SUPPORTS_CHANGE_NOTIFICATION,
    },
    CoredllOrdinalDef {
        name: "WaitForAPIReady",
        ordinal: ORD_WAIT_FOR_APIREADY,
    },
    CoredllOrdinalDef {
        name: "EnableGestures",
        ordinal: ORD_ENABLE_GESTURES,
    },
    CoredllOrdinalDef {
        name: "DisableGestures",
        ordinal: ORD_DISABLE_GESTURES,
    },
    // Registry notification lifecycle (dispatched no-ops)
    CoredllOrdinalDef {
        name: "RegistryCloseNotification",
        ordinal: ORD_REGISTRY_CLOSE_NOTIFICATION,
    },
    CoredllOrdinalDef {
        name: "RegistryStopNotification",
        ordinal: ORD_REGISTRY_STOP_NOTIFICATION,
    },
    CoredllOrdinalDef {
        name: "RegistryBatchNotification",
        ordinal: ORD_REGISTRY_BATCH_NOTIFICATION,
    },
    // Gesture/dialog CE-specific helpers (dispatched no-ops)
    CoredllOrdinalDef {
        name: "SetWindowAutoGesture",
        ordinal: ORD_SET_WINDOW_AUTO_GESTURE,
    },
    CoredllOrdinalDef {
        name: "GetWindowAutoGesture",
        ordinal: ORD_GET_WINDOW_AUTO_GESTURE,
    },
    CoredllOrdinalDef {
        name: "SetDialogAutoScrollBar",
        ordinal: ORD_SET_DIALOG_AUTO_SCROLL_BAR,
    },
    CoredllOrdinalDef {
        name: "SetWindowPosOnRotate",
        ordinal: ORD_SET_WINDOW_POS_ON_ROTATE,
    },
    // DPA — Dynamic Pointer Array container
    CoredllOrdinalDef {
        name: "DPA_Create",
        ordinal: ORD_DPA_CREATE,
    },
    CoredllOrdinalDef {
        name: "DPA_CreateEx",
        ordinal: ORD_DPA_CREATE_EX,
    },
    CoredllOrdinalDef {
        name: "DPA_Clone",
        ordinal: ORD_DPA_CLONE,
    },
    CoredllOrdinalDef {
        name: "DPA_DeleteAllPtrs",
        ordinal: ORD_DPA_DELETE_ALL_PTRS,
    },
    CoredllOrdinalDef {
        name: "DPA_DeletePtr",
        ordinal: ORD_DPA_DELETE_PTR,
    },
    CoredllOrdinalDef {
        name: "DPA_Destroy",
        ordinal: ORD_DPA_DESTROY,
    },
    CoredllOrdinalDef {
        name: "DPA_DestroyCallback",
        ordinal: ORD_DPA_DESTROY_CALLBACK,
    },
    CoredllOrdinalDef {
        name: "DPA_EnumCallback",
        ordinal: ORD_DPA_ENUM_CALLBACK,
    },
    CoredllOrdinalDef {
        name: "DPA_GetPtr",
        ordinal: ORD_DPA_GET_PTR,
    },
    CoredllOrdinalDef {
        name: "DPA_GetPtrIndex",
        ordinal: ORD_DPA_GET_PTR_INDEX,
    },
    CoredllOrdinalDef {
        name: "DPA_Grow",
        ordinal: ORD_DPA_GROW,
    },
    CoredllOrdinalDef {
        name: "DPA_InsertPtr",
        ordinal: ORD_DPA_INSERT_PTR,
    },
    CoredllOrdinalDef {
        name: "DPA_Search",
        ordinal: ORD_DPA_SEARCH,
    },
    CoredllOrdinalDef {
        name: "DPA_SetPtr",
        ordinal: ORD_DPA_SET_PTR,
    },
    CoredllOrdinalDef {
        name: "DPA_Sort",
        ordinal: ORD_DPA_SORT,
    },
    // DSA — Dynamic Structure Array container
    CoredllOrdinalDef {
        name: "DSA_Clone",
        ordinal: ORD_DSA_CLONE,
    },
    CoredllOrdinalDef {
        name: "DSA_Create",
        ordinal: ORD_DSA_CREATE,
    },
    CoredllOrdinalDef {
        name: "DSA_DeleteAllItems",
        ordinal: ORD_DSA_DELETE_ALL_ITEMS,
    },
    CoredllOrdinalDef {
        name: "DSA_DeleteItem",
        ordinal: ORD_DSA_DELETE_ITEM,
    },
    CoredllOrdinalDef {
        name: "DSA_Destroy",
        ordinal: ORD_DSA_DESTROY,
    },
    CoredllOrdinalDef {
        name: "DSA_DestroyCallback",
        ordinal: ORD_DSA_DESTROY_CALLBACK,
    },
    CoredllOrdinalDef {
        name: "DSA_EnumCallback",
        ordinal: ORD_DSA_ENUM_CALLBACK,
    },
    CoredllOrdinalDef {
        name: "DSA_GetItem",
        ordinal: ORD_DSA_GET_ITEM,
    },
    CoredllOrdinalDef {
        name: "DSA_GetItemPtr",
        ordinal: ORD_DSA_GET_ITEM_PTR,
    },
    CoredllOrdinalDef {
        name: "DSA_Grow",
        ordinal: ORD_DSA_GROW,
    },
    CoredllOrdinalDef {
        name: "DSA_InsertItem",
        ordinal: ORD_DSA_INSERT_ITEM,
    },
    CoredllOrdinalDef {
        name: "DSA_Search",
        ordinal: ORD_DSA_SEARCH,
    },
    CoredllOrdinalDef {
        name: "DSA_SetItem",
        ordinal: ORD_DSA_SET_ITEM,
    },
    CoredllOrdinalDef {
        name: "DSA_SetRange",
        ordinal: ORD_DSA_SET_RANGE,
    },
    CoredllOrdinalDef {
        name: "DSA_Sort",
        ordinal: ORD_DSA_SORT,
    },
    // GWE internal — returns zero/false
    CoredllOrdinalDef {
        name: "GetGweApiSetTables",
        ordinal: ORD_GET_GWE_API_SET_TABLES,
    },
    // String safe-copy helpers
    CoredllOrdinalDef {
        name: "StringCchCopyNExW",
        ordinal: ORD_STRING_CCH_COPY_NEX_W,
    },
    CoredllOrdinalDef {
        name: "StringCbCopyNExW",
        ordinal: ORD_STRING_CB_COPY_NEX_W,
    },
    // GDI extras
    CoredllOrdinalDef {
        name: "GetIconInfo",
        ordinal: ORD_GET_ICON_INFO,
    },
    CoredllOrdinalDef {
        name: "AlphaBlend",
        ordinal: ORD_ALPHA_BLEND,
    },
    CoredllOrdinalDef {
        name: "GetStretchBltMode",
        ordinal: ORD_GET_STRETCH_BLT_MODE,
    },
    CoredllOrdinalDef {
        name: "SetStretchBltMode",
        ordinal: ORD_SET_STRETCH_BLT_MODE,
    },
    CoredllOrdinalDef {
        name: "GetLayout",
        ordinal: ORD_GET_LAYOUT,
    },
    CoredllOrdinalDef {
        name: "SetLayout",
        ordinal: ORD_SET_LAYOUT,
    },
    CoredllOrdinalDef {
        name: "AnimateRects",
        ordinal: ORD_ANIMATE_RECTS,
    },
    CoredllOrdinalDef {
        name: "GetTextCharacterExtra",
        ordinal: ORD_GET_TEXT_CHARACTER_EXTRA,
    },
    CoredllOrdinalDef {
        name: "SetTextCharacterExtra",
        ordinal: ORD_SET_TEXT_CHARACTER_EXTRA,
    },
    // Font utilities
    CoredllOrdinalDef {
        name: "EnumFontFamiliesExW",
        ordinal: ORD_ENUM_FONT_FAMILIES_EX_W,
    },
    CoredllOrdinalDef {
        name: "GetNLSTables",
        ordinal: ORD_GET_NLS_TABLES,
    },
    CoredllOrdinalDef {
        name: "GetCharABCWidthsI",
        ordinal: ORD_GET_CHAR_ABCWIDTHS_I,
    },
    CoredllOrdinalDef {
        name: "GetFontData",
        ordinal: ORD_GET_FONT_DATA,
    },
    CoredllOrdinalDef {
        name: "GetOutlineTextMetricsW",
        ordinal: ORD_GET_OUTLINE_TEXT_METRICS_W,
    },
    // Process/thread utilities
    CoredllOrdinalDef {
        name: "OpenThread",
        ordinal: ORD_OPEN_THREAD,
    },
    CoredllOrdinalDef {
        name: "GetProcessIdOfThread",
        ordinal: ORD_GET_PROCESS_ID_OF_THREAD,
    },
    // Power management
    CoredllOrdinalDef {
        name: "GetSystemPowerStatusEx",
        ordinal: ORD_GET_SYSTEM_POWER_STATUS_EX,
    },
    CoredllOrdinalDef {
        name: "GetSystemPowerStatusEx2",
        ordinal: ORD_GET_SYSTEM_POWER_STATUS_EX2,
    },
    // Language/locale utilities
    CoredllOrdinalDef {
        name: "GetSystemDefaultUILanguage",
        ordinal: ORD_GET_SYSTEM_DEFAULT_UILANGUAGE,
    },
    CoredllOrdinalDef {
        name: "GetUserDefaultUILanguage",
        ordinal: ORD_GET_USER_DEFAULT_UILANGUAGE,
    },
    CoredllOrdinalDef {
        name: "EnumUILanguagesW",
        ordinal: ORD_ENUM_UILANGUAGES_W,
    },
    // Async I/O
    CoredllOrdinalDef {
        name: "GetOverlappedResult",
        ordinal: ORD_GET_OVERLAPPED_RESULT,
    },
    // Memory tracing variants
    CoredllOrdinalDef {
        name: "LocalAllocTrace",
        ordinal: ORD_LOCAL_ALLOC_TRACE,
    },
    CoredllOrdinalDef {
        name: "HeapAllocTrace",
        ordinal: ORD_HEAP_ALLOC_TRACE,
    },
];

pub fn lookup(ordinal: u32) -> Option<&'static CoredllOrdinalDef> {
    static BY_ORDINAL: OnceLock<BTreeMap<u32, &'static CoredllOrdinalDef>> = OnceLock::new();
    BY_ORDINAL
        .get_or_init(|| {
            let mut by_ordinal = BTreeMap::new();
            for export in COREDLL_EXPORTS {
                by_ordinal.entry(export.ordinal).or_insert(export);
            }
            for export in SDK_ORDINALS {
                by_ordinal.entry(export.ordinal).or_insert(export);
            }
            for export in SUPPLEMENTAL_ORDINALS {
                by_ordinal.entry(export.ordinal).or_insert(export);
            }
            by_ordinal
        })
        .get(&ordinal)
        .copied()
}

pub fn lookup_export_index(index: u32) -> Option<&'static CoredllOrdinalDef> {
    COREDLL_EXPORT_INDEX
        .get(index as usize)
        .and_then(Option::as_ref)
}
