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

- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\apicall.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\CORE\DLL\apis.c`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\psyscall.h`: raw API-set support now
  creates API-set handles, registers/query names in CE's 128-slot API table,
  returns ARM implicit-call trap addresses from `GetAPIAddress`, and verifies
  `CreateAPIHandle` objects back to their server data. Coverage verifies
  create/register/query, readiness, encoded method addresses, valid/invalid
  API handles, and bad guest method-table validation.
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
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\CORE\DLL\loader.cpp`: raw
  `IsPrimaryThread` now follows CE's per-process `g_dwMainThId` check instead
  of treating the first allocated emulator thread as globally primary.
  Coverage verifies the boot process, child-process primary thread identity
  after launch records are drained, and `IsProcessDying`/`IsExiting` reporting
  from the current process exit state.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\syncobj.c` and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\INC\syncobj.h`: raw
  `IsNamedEventSignaled` now opens the named event and returns its current
  signaled state without consuming auto-reset events or changing last-error on
  missing or invalid names. Coverage verifies missing events, set/reset state,
  repeated peeks, null names, and invalid guest pointers.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\compr2.c` and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\INC\kwin32.h`: raw
  `BinaryCompress`/`BinaryDecompress` now use the CE compression stream helper
  instead of reporting unsupported. Coverage verifies computed-only output
  sizing, stored binary packets, full and skipped decompression, all-zero
  buffers, short output buffers, invalid packets, and null output rejection.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\kdriver.c` and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\INC\loader.h`: raw
  `LoadKernelLibrary` now resolves registered CE kernel libraries instead of
  returning `ERROR_NOT_SUPPORTED`, applies the CE `LOAD_LIBRARY_IN_KERNEL`,
  `MF_NO_THREAD_CALLS`, and `LLIB_NO_PAGING` load flags, increments the module
  refcount, preserves exported `IOControl` lookup, and reports CE-style missing
  or invalid library-name failures.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\CORE\DLL\loader.cpp` and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\loader.c`: raw
  `LoadLibraryExW` now preserves CE no-import load flags on retained modules.
  `LOAD_LIBRARY_AS_DATAFILE` records both datafile and no-resolve state so
  `GetProcAddress` rejects the handle, while a later normal load still clears
  a prior pure `DONT_RESOLVE_DLL_REFERENCES` request.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\vm.c` and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\INC\vm.h`: raw virtual allocations
  with explicit unaligned addresses now reserve the full CE page range from
  `PAGEALIGN_DOWN(lpAddress)` through `PAGEALIGN_UP(lpAddress + size)`.
  Coverage verifies the rounded base, expanded allocation size, and accessible
  requested end address.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\process.c` and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\mkfuncs.h`: raw
  `GetModuleFileNameW` now accepts registered DLL module handles as well as
  the process module handle, returning the module guest path and preserving
  CE's `ERROR_INVALID_PARAMETER` failure for bad handles or empty buffers.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\CORE\DLL\loader.cpp`: raw
  `GetProcAddressInProcess` is now exported and follows CE's process/module
  validation plus `DoGetProcAddressA` lookup path for registered modules,
  including ordinal lookup and `ERROR_INVALID_PARAMETER` on missing modules,
  missing exports, invalid names, or invalid process handles.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\CORE\DLL\loader.cpp`: raw
  `GetProcName` now mirrors CE's `g_pszProgName` return path by exposing the
  current process module path as a cached guest wide string. Coverage verifies
  non-null return, stable repeated calls, last-error success, and cache
  invalidation when the process module path changes.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\process.c` and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\pkfuncs.h`: raw `CeGetModuleInfo` is now
  exported for `MINFO_MODULE_INFO`, and `GetModuleInformation` shares the same
  CE-style `PROCGetModInfo` validation for process handles, exact
  `MODULEINFO` sizing, and module-base lookup. Coverage verifies public and
  raw calls, image size/entry-point reporting, invalid process handles,
  unsupported info types, and oversized buffers.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\process.c` and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\pkfuncs.h`: raw `CeOpenFileHandle` is now
  exported and follows the `PROCGetModInfo`/`MINFO_FILE_HANDLE` path for
  host-backed loaded modules by returning a readable handle to the module's
  guest backing file. Coverage verifies returned file-handle readability,
  success last-error behavior, and invalid module rejection.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\process.c` and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winbase.h`: decoded old-MIPS
  `TerminateProcess(GetCurrentProcess(), code)` now marks the current process
  exited instead of resuming the thunk as a diagnostic-only success, matching
  CE's active-process `PROCTerminate` path and the `ExitProcess` macro.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\process.c`: raw
  `CheckRemoteDebuggerPresent` now follows CE's `PROCChekDbgr` validation by
  requiring both a valid process object and writable output pointer, returning
  no-debugger state for valid current or launched processes and
  `ERROR_INVALID_PARAMETER` for null/bad inputs.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\dbgapi.c`: raw
  `DebugActiveProcess`/`DebugActiveProcessStop` now track CE debugger
  attachment state instead of returning unsupported. Coverage verifies invalid
  and self-process rejection, duplicate attach `ERROR_ACCESS_DENIED`, debugger
  presence queries through duplicate handles, same-thread detach, and
  wrong-thread/repeated detach rejection.
- `C:\WINCE600\PUBLIC\COMMON\OAK\INC\diskio.h`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\DOSPART\part.cpp`, and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\DISKCACHE\cache.cpp`: store and
  partition handles now consume `IOCTL_DISK_DELETE_SECTORS`
  `DELETE_SECTOR_INFO` packets instead of falling through as unsupported.
  Coverage verifies CE packet-size validation, host-backed zero-byte success,
  and out-of-range partition requests returning `ERROR_INVALID_BLOCK` without
  touching the caller byte count.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\thread.c` and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winbase.h`: raw `CreateThread` now
  mirrors CE's `NKCreateThread` parameter gate by rejecting null start
  addresses, negative stack sizes, and flags other than `CREATE_SUSPENDED` and
  `STACK_SIZE_PARAM_IS_A_RESERVATION` before allocating a thread object.
  Coverage verifies invalid calls leave the caller thread-id slot untouched and
  do not consume the next CE thread ID.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\pathapi.cpp` and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\mounttable.cpp`: public
  raw `IsSystemFile` now follows CE's system-volume/file-attribute contract
  instead of returning unconditional false. Coverage verifies system-mounted
  files, non-system mounted files, missing paths, and null path rejection.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\fileapi.cpp` and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\fsioctl.h`: file-handle
  `DeviceIoControl(FSCTL_READ_OR_WRITE_SECURITY_DESCRIPTOR)` now follows CE
  FSDMGR's external-call rejection path, returning `ERROR_ACCESS_DENIED`
  before generic device dispatch and leaving caller byte counts untouched.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\fileapi.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\volumeapi.cpp`, and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\fsioctl.h`: file-handle
  `DeviceIoControl(FSCTL_GET_VOLUME_INFO)` now follows CE FSDMGR validation
  and returns the mounted or object-store `CE_VOLUME_INFO` record instead of
  falling through to generic device dispatch. Coverage verifies mounted-file
  volume names, byte counts, last error, and invalid parameter no-touch
  behavior.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\fsdmain.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\mounttable.cpp`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\extfile.h`: raw
  `CeRegisterFileSystemNotification` now follows CE's storage-manager wrapper
  by accepting and retaining the shell file-change callback pointer instead of
  returning unsupported. Coverage verifies callback registration, replacement,
  clearing by zero, and successful last-error behavior.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\volumeapi.cpp` and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\mextfile.h`: raw
  `AFS_RegisterFileSystemFunction` now validates mounted volume handles and
  retains the registered `SHELLFILECHANGEFUNC_t` callback for host-backed
  volumes instead of discarding it as a no-op. Coverage verifies registration,
  replacement, clearing by zero, invalid-handle failure, cross-process access
  denial, and cleanup when the volume handle closes.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\volumeapi.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\fsdmain.cpp`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\extfile.h`: raw `AFS_NotifyMountedFS`
  now validates mounted volume handles and records forwarded power/device
  notification flags for host-backed volumes instead of silently discarding the
  call. Coverage verifies power-on/off/device flags, repeated notifications,
  invalid-handle failure, cross-process access denial, and cleanup when the
  volume handle closes.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\volumeapi.cpp`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\extfile.h`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\mextfile.h`: raw
  `AFS_CloseAllFileHandles` now follows CE FSDMGR's mounted-volume validation
  path before returning the kernel-cleanup no-op success. Coverage verifies
  valid mounted volume handles, ignored process-handle payloads, invalid
  volume handles, and cross-process access denial.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\storemain.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\storedisk.cpp`, and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\storemgr.h`: raw StoreMgr
  `OpenStore`/`FindFirstStore`/`FindNextStore`/`FindCloseStore`/
  `GetStoreInfo` now expose mounted host-backed stores through CE-shaped
  `STOREINFO` records instead of being name-only exports. Coverage verifies
  leading-slash store names, ordered enumeration, store/device/profile fields,
  sector counts, read-only/removable flags, exhaustion, and closed-find-handle
  rejection.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\storemain.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\storedisk.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\partition.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\nullpart.cpp`, and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\storemgr.h`: raw StoreMgr
  `OpenPartition`/`FindFirstPartition`/`FindNextPartition`/
  `FindClosePartition`/`GetPartitionInfo` now expose a synthetic mounted
  partition for each mounted host-backed store. Coverage verifies CE-layout
  `PARTINFO` size and offsets, partition/file-system/volume names, sector
  counts, mounted attributes, whole-store FAT partition type, enumeration
  exhaustion, missing partitions, and closed partition-find-handle rejection.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\storemain.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\storedisk.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\partition.cpp`, and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\fsioctl.h`: partition-handle
  `DeviceIoControl` now follows StoreMgr's standard handle API mapping for
  safe mounted-volume FSCTLs. Coverage verifies `FSCTL_GET_VOLUME_INFO`
  returns CE-layout volume metadata from an opened partition handle, invalid
  volume-info inputs leave caller buffers untouched, and
  `FSCTL_REFRESH_VOLUME` succeeds as a zero-byte no-op.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\storemain.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\storedisk.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\partition.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\nullpart.cpp`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\fsdmgr.h`, and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\storemgr.h`: partition-handle
  `DeviceIoControl` now exposes CE-shaped cached disk metadata for mounted
  host-backed partitions. Coverage verifies `IOCTL_DISK_GETINFO`,
  `IOCTL_DISK_DEVICE_INFO`, `IOCTL_DISK_GETNAME`, and
  `IOCTL_DISK_INITIALIZED`/`DISK_IOCTL_INITIALIZED` plus
  `IOCTL_DISK_FLUSH_CACHE` through an opened partition handle, including byte
  counts, profile/device flags, mounted volume names, zero-byte status
  successes, lower disk-control `ERROR_NOT_SUPPORTED` forwarding, and
  insufficient-buffer rejection for disk names.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\storemain.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\storedisk.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\nullpart.cpp`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\fsdmgr.h`, and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\storemgr.h`: store-handle
  `DeviceIoControl` now exposes the cached disk paths CE uses before
  partition mounting. Coverage verifies `IOCTL_DISK_GETINFO` with shared
  in/out `DISK_INFO`, cached `IOCTL_DISK_DEVICE_INFO`, store
  `IOCTL_DISK_GETNAME`/`DISK_IOCTL_GETNAME`,
  `IOCTL_DISK_INITIALIZED`/`DISK_IOCTL_INITIALIZED`,
  `IOCTL_DISK_FLUSH_CACHE`, byte counts, mounted profile/device flags,
  zero-byte status successes, lower disk-control `ERROR_NOT_SUPPORTED`
  forwarding, and invalid cached-device-info/name no-touch behavior.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\storedisk.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\stgapi.cpp`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\diskio.h`: store and partition
  `DeviceIoControl(IOCTL_DISK_GET_STORAGEID)` now follow StoreMgr's cached
  storage-identification path. Coverage verifies the CE-layout
  `STORAGE_IDENTIFICATION` size/flags, byte counts, and invalid short-output
  rejection before any caller-buffer writes.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\fsdmgrapi.cpp` and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\mounttable.cpp`:
  FSDMGR helper imports now preserve CE's `FSDMGR_GetMountFlags` `LRESULT`
  shape by writing mount flags without clearing the caller's last-error on
  success. Coverage also locks down `FSDMGR_GetVolumeName` name-length return
  values and exact-sized buffer rejection before caller-buffer writes.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\CORE\DLL\resource.cpp` and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winver.h`: raw `VerQueryValueW` now
  walks CE `VERBLOCK` version-resource trees instead of returning unsupported,
  returning pointers inside the caller's `GetFileVersionInfoW` buffer and the
  raw CE `wValLen` value. Coverage verifies root `VS_FIXEDFILEINFO` queries,
  nested `StringFileInfo` lookups, zeroed length outputs, and missing-path
  `ERROR_RESOURCE_NAME_NOT_FOUND` behavior.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\CORE\DLL\resource.cpp`: raw
  `ExtractResource` now follows CE's datafile-load/resource-copy path for PE
  resources instead of returning unsupported, including `MAKEINTRESOURCE` and
  `#123` string identifiers, fixed heap copies, success last-error
  preservation, missing-resource failures, and invalid guest pointer rejection.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\storemain.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\stgapi.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\nullpart.cpp`, and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\storemgr.h`: raw StoreMgr management
  ordinals now validate store and partition handles/names instead of being
  inert exports. Coverage verifies safe host-backed semantics: mount and
  dismount of the already-mounted synthetic partition are no-op successes,
  store/partition layout mutations fail with `ERROR_GEN_FAILURE`, empty
  partition names fail with `ERROR_BAD_ARGUMENTS`, and missing partition
  deletion follows `ERROR_FILE_NOT_FOUND`.
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
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\filesystem.cpp`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winbase.h`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\mwinbase.h`: raw `LoadFSD`/`LoadFSDEx`
  now validate device handles, FSD names, sync/async flags, and required
  `FSD_MountDisk`/`FSD_UnmountDisk` driver exports instead of returning
  `ERROR_NOT_SUPPORTED`. Coverage verifies successful FSD driver retention
  with CE no-paging load flags, missing drivers, malformed FSD drivers,
  invalid device handles, invalid flags, and invalid name pointers.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\DEVICE\IORM\iorm.c`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winbase.h`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\mwinbase.h`, and
  `C:\WINCE600\PUBLIC\COMMON\DDK\INC\resmgr.h`: raw
  `ResourceCreateList`/`ResourceRequest`/`ResourceRequestEx`/
  `ResourceRelease`/`ResourceMarkAsShareable`/`ResourceDestroyList` now track
  CE-style IORM resource domains instead of reporting unsupported. Coverage
  verifies missing domains, duplicate and invalid list creation,
  release-to-validate behavior, default-unshareable and explicit-shareable
  request conflicts, exclusive claims, unsupported flag rejection, release
  refcounting, out-of-range failures, busy destroy rejection, missing-domain
  destroy behavior, and IORM supplemental export-table resolution.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\thread.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\kcalls.c`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winbase.h`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\mkfuncs.h`: raw
  `CeGetThreadQuantum`/`CeSetThreadQuantum` now validate thread handles and
  track per-thread quantum values instead of returning an unconditional
  default/success. Coverage verifies the CE default quantum, pseudo-current
  and real thread handles, set/get round-trips, high-bit quantum rejection,
  invalid get returning `MAXDWORD`, and invalid set failure.
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
- `C:\WINCE600\PRIVATE\WINCEOS\DRIVERS\DEVLOAD\devreg.cpp`,
  `C:\WINCE600\PUBLIC\COMMON\DDK\INC\devload.h`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\windev.h`: raw `OpenDeviceKey` now
  follows CE's active-key `Key` indirection instead of reporting unsupported,
  seeding configured devices into `Drivers\Active\N` and
  `Drivers\BuiltIn\NAME` registry keys when needed. Coverage verifies returned
  driver-key handles, null active-key pointers, missing active keys, and active
  keys missing the `Key` value.
- `C:\WINCE600\PUBLIC\COMMON\DDK\INC\devload.h`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\mwinbase.h`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\windev.h`: raw `ActivateDevice`,
  `ActivateDeviceEx`, `RegisterDevice`, `DeactivateDevice`, and
  `DeregisterDevice` now open and close configured emulator devices through
  CE-style device handles instead of reporting unsupported or unconditional
  success. Activation seeds `Drivers\Active\N` with `Key`, `Name`, `Hnd`, and
  `ClientInfo`, `ActivateDeviceEx` applies bounded `REGINI` entries to the
  driver key, and coverage verifies invalid parameters, missing devices,
  configured handle paths, active registry metadata, and closed-handle
  rejection.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\DEVICE\PMIF\pmif.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\DEVICE\NOPMIF\nopmif.c`, and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\pm.h`: raw `SetSystemPowerState`,
  `GetSystemPowerState`, `DevicePowerNotify`, `SetDevicePower`, and
  `GetDevicePower` now track emulator power-manager state instead of
  reporting unsupported. `SetPowerRequirement`/`ReleasePowerRequirement` and
  `RequestPowerNotifications`/`StopPowerNotifications` now return and validate
  typed handles, and `RegisterPowerRelationship`/`ReleasePowerRelationship`
  now retain parent/child names plus optional `POWER_CAPABILITIES` bytes.
  Requirements allow PM-only device names while updating configured device Dx
  state when possible, and notification registrations require real message
  queues. System state names/flags round-trip through the CE `DWORD` status
  APIs, configured devices expose default `D0` and mutable Dx state, and
  coverage verifies missing devices, invalid Dx values, null outputs, bad
  capability buffers, closed requirement/notification/relationship handles,
  and short system-state buffers.
- `C:\WINCE600\PUBLIC\SHELL\OAK\HPC\EXPLORER\AYGSHELLFUNCS\HAVEAYGSHELL\notification.cpp`
  and `C:\WINCE600\PUBLIC\SHELL\OAK\HPC\EXPLORER\TASKBAR\taskbarbubble.cpp`:
  `SHNotificationAddI` now retains title and HTML content according to CE's
  `SHNUM_TITLE`/`SHNUM_HTML` taskbar-bubble flags instead of treating every
  marshalled pointer as captured bubble state. Coverage verifies pointer
  presence still satisfies the add wrapper contract while retained content is
  flag-gated, HTML length queries distinguish no captured HTML from captured
  empty HTML, and an explicit `SHNUM_HTML` update can populate HTML later.
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
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\doc.cpp` and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\wingdi.h`: raw `StartDocW`,
  `StartPage`, `EndPage`, `EndDoc`, and `AbortDoc` now apply CE's print API
  validation before the emulator's unsupported-spooler fallback. Null/bad/
  deleted HDCs and `StartDocW(hdc, NULL)` return `SP_ERROR` with
  `ERROR_INVALID_HANDLE`; otherwise valid print operations continue to report
  `ERROR_NOT_SUPPORTED` until a spooler backend exists.
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
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\brush.cpp`:
  `passBrushNULL(ECreatePatternBrush)` is now covered for raw pattern-brush
  creation: null, stale, and non-bitmap handles fail with
  `ERROR_INVALID_HANDLE`, while valid bitmap pattern brushes keep private
  backing after the caller bitmap is deleted.
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
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\draw.cpp`,
  `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\verify.cpp`, and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\MGDI\INC\gdiobj.h`: raw
  `GetCurrentPositionEx` now follows CE DC-handle validation, reporting
  `ERROR_INVALID_HANDLE` for null/bad HDCs and `ERROR_INVALID_PARAMETER` for
  null or unwritable output points instead of silently succeeding or folding
  bad handles into parameter errors.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\text.cpp`:
  `passNull2Text(EDrawTextW)` is now represented for raw `DrawTextW` HDC
  validation: null and bad HDCs fail with `ERROR_INVALID_HANDLE`, while null
  text or rectangle pointers on a valid DC continue to fail with
  `ERROR_INVALID_PARAMETER`.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\dc.cpp`:
  `ProgramError` is now covered for raw DC lifecycle validation:
  `ReleaseDC(NULL, hdc)` rejects a previously deleted DC with
  `ERROR_INVALID_HANDLE` instead of treating the stale handle as released.
- `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\dc.cpp`:
  `ScrollDCScrollTest` update geometry is now covered for raw `ScrollDC`:
  exposed edge rectangles are reported for one-axis scrolls, and
  `hrgnUpdate` is updated to the same simple region box.
- `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winuser.h` and
  `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\dc.cpp`: raw `ScrollWindowEx`
  now applies the same CE update geometry to `hrgnUpdate` as it already wrote
  to `prcUpdate`, clearing the region on empty updates and rejecting invalid
  region handles before scrolling side effects. Coverage verifies simple,
  empty, and invalid update-region paths.
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
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\CORE\DLL\resource.cpp` and
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\shellapi.h`: raw `ExtractIconExW` and
  `KernExtractIcons` now validate file-name parameters as guest `LPCWSTR`
  paths instead of accepting atom-style low values. Coverage verifies invalid
  path pointers fail with `ERROR_INVALID_PARAMETER` without mutating large or
  small icon output slots.
- `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winuser.h`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\icon.h`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\WTL\INCLUDE\atlgdi.h`: raw
  `DrawIconEx` now honors `hbrFlickerFreeDraw` by validating the brush and
  pre-filling the resolved icon target rectangle before masked icon pixels are
  drawn. Coverage verifies masked bitmap-backed icons retain the brush-filled
  background for transparent pixels, opaque pixels still draw normally, and
  invalid brush handles fail with `ERROR_INVALID_HANDLE`.
- `C:\WINCE600\PUBLIC\COMMON\OAK\INC\mshellap.h`,
  `C:\WINCE600\PUBLIC\COMMON\SDK\INC\shellapi.h`, and
  `C:\WINCE600\PUBLIC\WCESHELLFE\OAK\TASKMAN\minserver.cpp`: raw
  `Shell_NotifyIcon` now respects CE's shell-service marshal size when present,
  capping optional fields beyond the 152-byte `NOTIFYICONDATAW` contract so
  stale extended state bytes are ignored for CE-sized trap calls. Coverage
  verifies add and modify calls with an extended caller buffer but a CE-sized
  service frame.
- `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winuser.h`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\WTL\INCLUDE\atlapp.h`, and CE broadcast
  callers such as `C:\WINCE600\PUBLIC\WCESHELLFE\OAK\CTLPNL\INTL2\intl.c`:
  raw `SendMessageTimeout(HWND_BROADCAST, ...)` now targets live top-level
  windows instead of failing as an invalid HWND. Same-thread targets dispatch
  synchronously, cross-thread targets queue timeout sends in CE Z-order,
  child/destroyed windows are skipped, and broadcast calls leave caller result
  storage untouched because no single window result exists. Coverage verifies
  top-level filtering, Z-order queueing, timeout metadata, and untouched
  result storage.
- `C:\WINCE600\PUBLIC\COMMON\SDK\INC\winuser.h` and the GWE monitor-handle
  model under `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\MGDI\INC`: raw
  `MonitorFromPoint`, `MonitorFromRect`, `MonitorFromWindow`, and
  `GetMonitorInfo` now honor CE's default-to-null/primary/nearest flags on the
  single emulated display and reject invalid monitor/window handles instead of
  unconditionally returning the primary pseudo-handle. Coverage verifies
  off-screen defaults, primary fallback, `MONITORINFO` layout, and invalid
  handle errors.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\CONTROLS\IMGCTL\imagelist.cpp`:
  raw `ImageList_GetImageInfo` now separates invalid image-list handles from
  invalid image indices/null output buffers and leaves the caller's
  `IMAGEINFO` storage untouched on failure, matching CE's validation-before-
  write path.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\GWE\CONTROLS\IMGCTL\imagelist.cpp`:
  raw `ImageList_SetDragCursorImage` now tracks the source cursor image list
  and index separately from the transient merged drag image so repeated calls
  with the same source cursor succeed as CE no-ops, even when the hotspot
  arguments differ. Coverage verifies the helper state and raw ordinal path
  keep the existing merged drag image instead of replacing it.
- `C:\WINCE600\PUBLIC\SHELL\OAK\HPC\EXPLORER\TASKBAR\taskbar.cpp`,
  `C:\WINCE600\PUBLIC\COMMON\OAK\WTL\INCLUDE\atlframe.h`, and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\CORE\DLL\resource.cpp`: raw
  `LoadImageW(IMAGE_ICON)` group-resource selection now keeps exact matches
  first but falls back to the nearest available RT_ICON size instead of the
  largest entry, preserving CE shell 16x16 icon requests when a group only
  contains intermediate and large images.
- Runtime live-pump/iNavi route frontier: when the active process is parked
  in a hidden `GetMessage` wait and a visible parked process has a ready wait
  unblock, `rotate_after_run_slice_handoff` now resumes the ready parked
  process before falling through to generic runnable-process rotation.
  Orphaned visible-window callouts now map process-slot-equivalent MFC
  wndprocs back into loaded guest DLL ranges and filter low-value paint/focus
  messages without forcing a guest callout. Coverage reproduces the hidden
  iSearch wait to visible iNavi handoff, verifies process/thread/module state
  after rotation, and checks stale child-thread recovery for mapped MFC
  wndprocs. Host-wall-clock live-pump stops now also rotate to runnable parked
  processes even when no message waiter or priority wait is present, preventing
  visible runnable work from being stranded behind the active process. Live
  pump handoff also keeps visible active route UI ahead of hidden CPU-runnable
  child helpers, only using the hidden-child fallback when the active process
  has no visible windows. The Unicorn timeslice hook now also stops immediately
  when live-pump execution
  has a hidden active process and a runnable visible parked process, allowing
  route UI work to progress even before the current PC reaches a normal safe
  scheduling boundary. CE high process-slot callbacks in the user/shared VM
  range are now preserved as receiver-process wndprocs instead of requiring a
  loaded-DLL alias, matching the `VM_SHARED_HEAP_BASE` through `VM_KMODE_BASE`
  range documented in `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\vm.c`.
  `SetWindowLongW(GWL_WNDPROC)` import thunks now also normalize CE process-slot
  callback values through the caller module before the raw coredll call, so
  route UI subclass procedures land on the loaded MFC/commctrl guest code that
  owns the slot offset. `SetClassLongW(GCL_WNDPROC)` and `RegisterClassW`
  import thunks use the same caller-module mapping before class state is
  committed, covering window classes that install CE process-slot callbacks at
  registration time. When both process-slot and shared DLL aliases can satisfy
  the same high-slot callback offset, mapped WNDPROC resolution now prefers the
  process-slot DLL image so callbacks stay in the receiver process view. Hidden
  active child processes with pending window-targeted messages are no longer
  treated as terminal exits before those messages are pumped, preserving the CE
  wait/message transition instead of handing control back to the parked parent
  too early. Return-stub handoffs now also preserve active processes that still
  own visible windows or receiver work, so child image launches cannot strand
  the parent route UI while rotating into the parked child process.
- `C:\WINCE600\PUBLIC\SHELL\OAK\HPC\EXPLORER\AYGSHELLFUNCS\HAVEAYGSHELL\notification.cpp`
  and `C:\WINCE600\PUBLIC\SHELL\OAK\HPC\EXPLORER\MAIN\utils.cpp`: raw
  `SHNotificationAddI`/`SHNotificationUpdateI` now follow CE TaskbarBubble
  icon ownership for emulator-managed icons by storing a copied HICON instead
  of the caller-owned handle, replacing updates with a fresh copy, and
  destroying the prior copied icon on update/remove. Coverage verifies add,
  query, non-null update, null-icon update, duplicate iconic add, and remove
  lifecycle behavior against copied handles.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\stgapi.cpp`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\storedisk.cpp`, and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\diskio.h`: mounted store and partition
  `DeviceIoControl` now route CE `SG_REQ` disk read/write IOCTLs through the
  synthetic FSDMGR sector store instead of falling through as unsupported.
  Coverage verifies `DISK_IOCTL_WRITE` through a store handle, `IOCTL_DISK_READ`
  through the opened partition handle, scatter-gather splitting, shared
  mounted-store identity, request status updates, zero byte-counts, and invalid
  SG request rejection.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\fsdmgrapi.cpp` and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\diskio.h`: mounted store and partition
  `DeviceIoControl(DISK_IOCTL_SETINFO/IOCTL_DISK_SETINFO)` now accepts the CE
  `DISK_INFO` packet and updates the synthetic disk metadata returned by later
  `GETINFO` calls, while short packets fail with `ERROR_INVALID_PARAMETER`
  without touching byte-count storage.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\syncobj.c` and
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\NOTIFY\fsnotify.cpp`: raw
  `GetEventData`/`SetEventData` now preserve the CE event `dwData` payload
  used by filesystem notification events instead of reporting unsupported or
  discarding writes. Coverage verifies zero-payload last-error clearing,
  nonzero payload reads without clobbering last-error, successful updates, and
  invalid event handles.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\kmisc.c`,
  `C:\WINCE600\PRIVATE\WINCEOS\COREOS\CORE\CORELIBC\CRTW32\STDIO\stioinit.c`,
  and `C:\WINCE600\PRIVATE\WINCEOS\UTILS\CMD2\cmd.cxx`: raw
  `GetStdioPathW`/`SetStdioPathW` now maintain CE per-process stdin/stdout/
  stderr path names instead of returning unsupported. Coverage verifies unset
  entries, explicit empty-string paths, null-path clearing, strict guest string
  pointer validation, short-buffer `ERROR_INSUFFICIENT_BUFFER` behavior that
  preserves the caller length, process isolation, and invalid descriptor
  rejection.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\CORE\DLL\resource.cpp`: raw
  `FindResourceW` now preserves CE's split between invalid resource modules
  and valid modules with missing resources, returning `ERROR_INVALID_HANDLE`
  only for bogus module handles and `ERROR_RESOURCE_NAME_NOT_FOUND` after a
  valid module/resource table search misses.
- `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\DOSPART\part.cpp` and
  `C:\WINCE600\PUBLIC\COMMON\OAK\INC\diskio.h`: mounted partition
  `IOCTL_DISK_SECURE_WIPE`/`IOCTL_DISK_SET_SECURE_WIPE_FLAG` now follow the CE
  partition-layer forwarding shape, accepting zero-byte caller input but
  reporting the host-backed lower disk driver's unsupported status instead of
  pretending the secure wipe completed.
- `C:\WINCE600\PUBLIC\COMMON\SDK\INC\fsioctl.h`: file-handle
  `DeviceIoControl(FSCTL_SET_EXTENDED_FLAGS/FSCTL_SET_ENCRYPTION/
  FSCTL_SET_SPARSE)` now recognizes the CE optional-attribute FSCTL contracts
  before generic device dispatch, validating DWORD encryption/extended-flag
  packets and sparse-file zero-byte requests before reporting the host-backed
  FSD's unsupported status.

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
   - `DeviceIoControl(FSCTL_GET_STREAM_INFORMATION)` now follows the CE
     `fsioctl.h` `FILE_STREAM_INFO` layout for open file handles, reporting
     the standard stream's attributes, CE FILETIMEs, logical size, and physical
     size while preserving short-buffer byte-count behavior. Coverage verifies
     the caller-initialized info-level packet shape used by CE callers.
   - File-handle `DeviceIoControl(FSCTL_GET_COMPRESSION/FSCTL_SET_COMPRESSION)`
     now uses the CE `fsioctl.h` control codes and `COMPRESSION_FORMAT_*`
     contract for host-backed uncompressed files. Coverage verifies
     `COMPRESSION_FORMAT_NONE` reporting, short output buffers, accepting
     no-op `NONE` updates, unsupported compressed formats, and invalid input
     packets.
   - File-handle `DeviceIoControl(FSCTL_QUERY_ALLOCATED_RANGES)` now consumes
     CE's `FILE_ALLOCATED_RANGE_BUFFER` layout from `fsioctl.h` and reports
     the requested intersection with the host-backed file extent as allocated.
     Coverage verifies EOF clipping, empty intersections, zero-length queries,
     short output buffers, invalid input sizing, and stale-handle rejection.
   - `CeFsIoControlW`/`AFS_FsIoControlW(FSCTL_STORAGE_MEDIA_CHANGE_EVENT)`
     now follows CE's mounted-volume media-change path for host-backed mounts:
     detach notifications succeed as ignored zero-byte events, while attach
     notifications report `StorageMediaAttachResultUnchanged`. Coverage
     verifies CE input/output sizing, AFS routing, and invalid event rejection.
  - File-handle `DeviceIoControl(FSCTL_SET_ZERO_DATA)` now consumes CE's
    `FILE_ZERO_DATA_INFORMATION` layout from `fsioctl.h` and zeros the
    requested existing host-backed file range without growing the file.
    Coverage verifies middle-range zeroing, EOF clipping, invalid range
    rejection, and read-only handle access denial.
  - `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\storedisk.cpp`,
    `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\FSDMGR\partition.cpp`,
    `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\DOSPART\part.cpp`, and
    `C:\WINCE600\PUBLIC\COMMON\OAK\INC\diskio.h`: mounted store and partition
    handles now recognize the remaining CE disk-maintenance IOCTLs needed by
    host-backed media. Store handles accept format/scan no-ops and power
    timing queries; partition handles add format no-ops, secure-wipe status
    controls, `IOCTL_DISK_GET_SECTOR_ADDR` validation, and
    `IOCTL_DISK_COPY_EXTERNAL_START` packet validation before reporting the
    unsupported lower block-driver operation; `IOCTL_DISK_COPY_EXTERNAL_COMPLETE`
    follows CE `DOSPART` fallthrough behavior and forwards to the lower driver
    without requiring a start-packet layout. Coverage verifies success
    byte-counts, CE `PowerTimings` sizing, invalid packet rejection, preserved
    sector-list inputs, and lower-driver `ERROR_NOT_SUPPORTED` forwarding.

6. Expand input, text, GDI, and drawing fidelity.
   - Add coverage for IME UI callbacks, caret timing, font fallback, glyph
     metrics, palette/device-color behavior, alpha/mask variants, DIB lifetime
     edges, clipping, and read-only bitmap rejection where CE evidence exists.
   - `C:\WINCE600\PUBLIC\COMMON\SDK\INC\wingdi.h` and the CE GDI invalid-HDC
     patterns in `C:\WINCE600\PRIVATE\TEST\GWES\GDI\GDIAPI\text.cpp`/`draw.cpp`:
     raw `GetFontData` and `GetOutlineTextMetricsW` now validate null/bad HDCs
     before falling back to their unsupported/no-outline valid-DC paths.

7. Advance the iNavi route/runtime frontier.
   - Continue from the current real splash/resource-loading state toward the
     map transition, route search, destination entry, and realistic navigation.
   - Continue after the ready parked-wait live-pump handoff by tracing the next
     app-level wait, event, or message transition blocking visible route UI
     progress.
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
