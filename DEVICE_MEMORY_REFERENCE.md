# Device Memory Reference

This note captures volatile EBOOT/CE memory findings for the iNavi GN 2010
AU13xx/MIPSII device. All `write` commands below are RAM-only and reset on SoC
reset unless explicitly noted otherwise.

## Serial Monitor

- Port: `COM12`
- Settings: `115200 8N1`
- Line ending: carriage return (`CR`)
- Important connection quirk: after opening the port, send one bare `CR` and
  wait for the `>` prompt before sending commands.

Useful EBOOT commands:

```text
help
help write
read [-8|16|32] <addr> [count]
write [-8|16|32] <addr> <val> [count]
dump <addr> <length>
jump <addr>
```

## Key Addresses

```text
A00FFC00        BootArgs uncached alias
800FFC00        BootArgs cached alias
AFC82450        Resident CE image startup target used by EBOOT jump
AFF88B34        Runtime OAL DBGPARAM mask observed in one CE boot
```

BootArgs header observed at `A00FFC00` / `800FFC00`:

```text
A00FFC00: 00000001
A00FFC04: 00000000
A00FFC08: 544F4F42  ; "BOOT"
A00FFC0C: 00000090
A00FFC10: 00000000  ; flag field before patch
```

The boot log confirms CE reads this block:

```text
BootArgs are at A00FFC00, SIG = 544f4f42
BOOTARG Sig is good
```

## Proven Kernel Debug Log Unlock

Minimal volatile patch:

```text
write -32 A00FFC10 00000011 1
jump AFC82450
```

Observed behavior:

- Enables kernel/OAL serial debug output on COM12.
- Produces early CE boot logs such as `+OEMInit`, interrupt init, driver load
  logs, `OALIoCtlGetDeviceInfo`, and exception dumps.
- Does not persist across SoC reset.

Interpretation:

```text
0x00000011 = OAL_KITL_FLAGS_ENABLED | OAL_KITL_FLAGS_POLL
```

This field appears to be at the start of an `OAL_KITL_ARGS`-like payload after
the 16-byte vendor `BOOT` header. It reliably unlocks serial debug logging, but
it has not yet proven KITL transport.

Reference log:

```text
target\eboot_kernel_debug_20260614_113230.log
target\eboot_enable_debug_20260614_122930.log
```

## KITL Status

Tried:

```text
write -32 A00FFC10 0000001D 1
jump AFC82450
```

Meaning if interpreted as CE KITL flags:

```text
0x0000001D = ENABLED | DHCP | VMINI | POLL
```

Observed behavior:

- Serial debug logs remained enabled.
- No visible `KITL`, `VMINI`, `KDBG`, `DHCP`, `BOOTME`, or kernel debugger
  startup strings appeared in the captured boot log.

Full Ethernet-style KITL block attempt:

```text
write -32 A00FFC10 0000001D 1  ; ENABLED | DHCP | VMINI | POLL
write -32 A00FFC14 00000000 1  ; devLoc.IfcType = Internal guess
write -32 A00FFC18 00000000 1  ; devLoc.BusNumber = 0
write -32 A00FFC1C B0900000 1  ; devLoc.LogicalLoc = LAN9118/EBOOT base-like value
write -32 A00FFC20 00000000 1  ; devLoc.PhysicalLoc = 0
write -32 A00FFC24 00000000 1  ; devLoc.Pin = 0
write -32 A00FFC28 56341202 1  ; MAC bytes 02:12:34:56
write -32 A00FFC2C 00009A78 1  ; MAC bytes 78:9A plus padding
write -32 A00FFC30 00000000 1  ; DHCP ipAddress
write -32 A00FFC34 00000000 1  ; DHCP ipMask
write -32 A00FFC38 00000000 1  ; DHCP ipRoute
jump AFC82450
```

Observed behavior:

- Serial debug logs remained enabled.
- No visible `KITL`, `VMINI`, `KDBG`, `DHCP`, `BOOTME`, `LAN9118`, or kernel
  debugger startup messages appeared.
- Normal CE boot continued, then RTL8192CU crashed as before.

Reference log:

```text
target\eboot_kitl_eth_attempt_20260614_123727.log
```

Conclusion:

- `A00FFC10` is enough for debug output.
- The attempts above used the vendor `BOOT` layout, not the standard CE
  `OAL_ARGS_HEADER` layout used by `OALArgsQuery(OAL_ARGS_QUERY_KITL)`.
- KITL likely needs a valid standard `ARGS` header and valid
  `DEVICE_LOCATION`, not only additional flag bits.
- `B0900000` is present near EBOOT data and may be Ethernet-related, but it is
  not sufficient as a CE KITL `DEVICE_LOCATION` for this image.

## Standard CE OAL Args Lead

Resident `nk.exe` disassembly shows an `OALArgsQuery`-style function at
`8FC89DC8` that checks:

```text
A00FFC00 == 0x53475241  ; OAL_ARGS_SIGNATURE 'SGRA' / "ARGS" in memory
A00FFC04 halfword == 1  ; OAL_ARGS_VERSION
A00FFC06 halfword == 1  ; BSP_ARGS_VERSION
```

Then it returns:

```text
type 1 -> A00FFC08  ; OAL_ARGS_QUERY_DEVID
type 2 -> A00FFC18  ; OAL_ARGS_QUERY_KITL
```

This matches CE source:

```text
C:\WINCE600\PLATFORM\COMMON\SRC\INC\oal_args.h
C:\WINCE600\PLATFORM\COMMON\SRC\INC\oal_kitl.h
C:\WINCE600\PLATFORM\SG2_VR5500\SRC\INC\args.h
C:\WINCE600\PLATFORM\SG2_VR5500\SRC\OAL\OALLIB\args.c
```

Known resident callers of this routine in `target\nk_exe.disasm`:

```text
8FC87C2C -> type 5
8FC8825C -> type 3
8FC88278 -> type 1
8FC88458 -> type 1
8FC88580 -> type 3
```

No direct resident `jal OALArgsQuery` with `type == 2` has been found in the
current `nk.exe` disassembly. That means the `A00FFC18` KITL pointer is a
source/disassembly-layout lead, not yet proof that this particular boot path
starts KITL from it.

The standard `BSP_ARGS` layout is:

```text
+0x00 OAL_ARGS_HEADER
+0x08 deviceId[16]
+0x18 OAL_KITL_ARGS
```

Candidate standard KITL experiment, if the device is back at the EBOOT prompt:

```text
write -32 A00FFC00 53475241 1  ; OAL_ARGS_SIGNATURE 'SGRA'
write -32 A00FFC04 00010001 1  ; oalVersion=1, bspVersion=1
write -32 A00FFC08 56414E49 1  ; "INAV"
write -32 A00FFC0C 494B2D49 1  ; "I-KI"
write -32 A00FFC10 00004C54 1  ; "TL\0\0"
write -32 A00FFC14 00000000 1
write -32 A00FFC18 0000001D 1  ; ENABLED | DHCP | VMINI | POLL
write -32 A00FFC1C 00000000 1  ; devLoc.IfcType candidate: Internal
write -32 A00FFC20 00000000 1  ; devLoc.BusNumber
write -32 A00FFC24 B0900000 1  ; devLoc.LogicalLoc candidate
write -32 A00FFC28 00000000 1  ; devLoc.PhysicalLoc
write -32 A00FFC2C 00000000 1  ; devLoc.Pin
write -32 A00FFC30 56341202 1  ; MAC bytes 02:12:34:56
write -32 A00FFC34 00009A78 1  ; MAC bytes 78:9A plus padding
write -32 A00FFC38 00000000 1  ; DHCP ipAddress
write -32 A00FFC3C 00000000 1  ; DHCP ipMask
write -32 A00FFC40 00000000 1  ; DHCP ipRoute
jump AFC82450
```

Risk/expectation:

- This replaces the vendor `"BOOT"` header, so the proven serial-debug unlock
  path may disappear for that boot.
- If the OAL uses the standard `ARGS` path for KITL, this is the first layout
  that actually gives it a valid `OALArgsQuery(OAL_ARGS_QUERY_KITL)` pointer.
- If no serial output appears, capture that too; it means the standard path did
  not also enable the UART debug print path.
- If KITL starts, look for `KITL`, `VMINI`, `KDBG`, `BOOTME`, `DHCP`, or
  `LAN9118` strings on serial or network traffic.

Result from `target\eboot_standard_args_kitl_attempt_20260614_125136.log`:

- EBOOT accepted all writes and verified:

```text
A00FFC00:53475241
A00FFC04:00010001
A00FFC08:56414E49
A00FFC0C:494B2D49
A00FFC10:00004C54
A00FFC14:00000000
```

- CE booted and printed:

```text
BootArgs are at A00FFC00, SIG = 56414e49
BOOTARG Sig is bad
```

- No visible `KITL`, `VMINI`, `KDBG`, `BOOTME`, `DHCP`, or `LAN9118` startup
  messages appeared.
- The `SIG = 56414e49` value is the word written at `A00FFC08` (`"INAV"`),
  which proves the vendor BootArgs checker reads its `"BOOT"` signature from
  `A00FFC08`, not `A00FFC00`.

Corrected hybrid experiment for the next EBOOT prompt:

```text
write -32 A00FFC00 53475241 1  ; Standard OAL_ARGS_SIGNATURE 'SGRA'
write -32 A00FFC04 00010001 1  ; oalVersion=1, bspVersion=1
write -32 A00FFC08 544F4F42 1  ; Preserve vendor "BOOT" signature
write -32 A00FFC0C 00000090 1  ; Preserve observed vendor BOOT word
write -32 A00FFC10 00000011 1  ; Preserve proven serial debug unlock
write -32 A00FFC14 00000000 1
write -32 A00FFC18 0000001D 1  ; Standard KITL flags: ENABLED|DHCP|VMINI|POLL
write -32 A00FFC1C 00000000 1  ; devLoc.IfcType candidate: Internal
write -32 A00FFC20 00000000 1  ; devLoc.BusNumber
write -32 A00FFC24 B0900000 1  ; devLoc.LogicalLoc candidate
write -32 A00FFC28 00000000 1  ; devLoc.PhysicalLoc
write -32 A00FFC2C 00000000 1  ; devLoc.Pin
write -32 A00FFC30 56341202 1  ; MAC bytes 02:12:34:56
write -32 A00FFC34 00009A78 1  ; MAC bytes 78:9A plus padding
write -32 A00FFC38 00000000 1  ; DHCP ipAddress
write -32 A00FFC3C 00000000 1  ; DHCP ipMask
write -32 A00FFC40 00000000 1  ; DHCP ipRoute
jump AFC82450
```

This hybrid layout tries to satisfy both parsers at once:

- Standard `OALArgsQuery` sees a valid `ARGS` header and can return KITL args
  at `A00FFC18`.
- Vendor boot/debug code still sees `"BOOT"` at `A00FFC08` and the known debug
  unlock at `A00FFC10`.

Result from `target\eboot_hybrid_args_boot_kitl_attempt_20260614_125912.log`:

- EBOOT accepted all writes and verified:

```text
A00FFC00:53475241
A00FFC04:00010001
A00FFC08:544F4F42
A00FFC0C:00000090
A00FFC10:00000011
A00FFC14:00000000
```

- CE booted with the vendor parser satisfied:

```text
BootArgs are at A00FFC00, SIG = 544f4f42
BOOTARG Sig is good
```

- Serial debug output remained enabled.
- No visible `KITL`, `VMINI`, `KDBG`, `BOOTME`, `DHCP`, or `LAN9118` startup
  messages appeared.
- RTL8192CU reproduced both known failures:

```text
PC=c0ce0a20(rtl8192cu.dll+0x00010a20) BVA=00000670
PC=c0ce3cd0(rtl8192cu.dll+0x00013cd0) BVA=00200214
```

Conclusion:

- The hybrid layout successfully preserves vendor debug logs.
- Simply adding a standard `ARGS` header and KITL args at `A00FFC18` is not
  enough to start visible KITL on this image.
- Next KITL work should focus on locating the actual KITL startup/device-table
  path or using a serial KITL candidate rather than continuing to vary these
  same Ethernet fields blindly.

## Resident KITL Linkage Check

Static checks after the hybrid attempt:

```text
D:\INAVI_Emulator\DUMPPLZ\Windows\nk.exe
target\nk_exe.disasm
D:\INAVI_Emulator\DUMPPLZ\dump_NAND_Part00.bin
D:\INAVI_Emulator\DUMPPLZ\dump_NAND_OpenStore.bin
```

Extraction manifest:

```text
D:\INAVI_Emulator\DUMPPLZ\Windows\_EXTRACT_MANIFEST_part00_boot.json
romhdr_offset = 0x002ff7f8
bias          = 0x8fc7e000
nummods       = 28
```

The manifest identifies `nk.exe` as the first resident module:

```text
name     = nk.exe
bytes    = 55808
rom_size = 79360
load     = 0x8FC7E000
PDB      = oal.pdb
```

Extracted part00 modules include kernel/OAL/user-mode core pieces and drivers
(`kernel.dll`, `coredll.dll`, `oalioctl.dll`, `device.dll`, `devmgr.dll`,
`ceddk.dll`, `ddi_au13xxlcd.dll`, `VSP.dll`, etc.), but no separate KITL/VMINI
module was extracted.

Findings:

- `ceconfig.h` and raw NAND text contain:

```text
#define CE_MODULES_VMINI 1
#define OEM_OEMMAIN_STATICKITL 1
```

- The actual resident `nk.exe` string set contains OAL/debug/BootArgs strings,
  including:

```text
+OALArgsQuery(%d)
-OALArgsQuery(pData = 0x%08x)
BootArgs are at %X, SIG = %x
BOOTARG Sig is good
BOOTARG Sig is bad
OALIoCtlGetDeviceInfo:
OALIoCtlDebugControl:
```

- The same resident image does not expose expected linked KITL strings such as:

```text
KITL
KDBG
VMINI
BOOTME
OALKitlInit
OEMKitlStartup
OALKitlStart
No supported KITL device
KITL Disabled
SerialRecv
```

- `target\nk_exe.disasm` has a standard `OALArgsQuery`-style function at
  `8FC89DC8`, but known direct callers are only:

```text
8FC87C2C -> type 5
8FC8825C -> type 3
8FC88278 -> type 1
8FC88458 -> type 1
8FC88580 -> type 3
```

- No direct resident call site has been found with
  `OALArgsQuery(OAL_ARGS_QUERY_KITL)` / `type == 2`.

Interpretation:

- The image contains standard OAL args query support.
- The dumped resident image does not currently look like it links the normal
  CE KITL startup library.
- The `ceconfig.h` macros alone are not enough to prove live KITL startup is
  present in this deployed image.
- This explains why valid-looking `A00FFC18` KITL args did not produce any
  `KITL`/`VMINI`/`KDBG` output.

Practical next direction:

- Treat COM12 kernel output as the available low-level debug channel.
- Use the stream driver/userland agent path for memory and trace control.
- If KITL remains desirable, find a full XIP/NK image region or symbols that
  include actual `OALKitlInit`/`OALKitlStart` code before more boot-arg writes.

Relevant CE layout from `C:\WINCE600\PLATFORM\COMMON\SRC\INC\oal_kitl.h`:

```c
typedef struct {
    UINT32 flags;
    DEVICE_LOCATION devLoc;
    union {
        struct {
            UINT32 baudRate;
            UINT32 dataBits;
            UINT32 stopBits;
            UINT32 parity;
        };
        struct {
            UINT16 mac[3];
            UINT32 ipAddress;
            UINT32 ipMask;
            UINT32 ipRoute;
        };
    };
} OAL_KITL_ARGS;
```

`DEVICE_LOCATION` from `pkfuncs.h`:

```c
typedef struct _DEVICE_LOCATION {
    DWORD IfcType;
    DWORD BusNumber;
    DWORD LogicalLoc;
    PVOID PhysicalLoc;
    DWORD Pin;
} DEVICE_LOCATION;
```

## Direct OAL Mask Patch Attempt

Tried before boot:

```text
read -32 AFF88B34 1
write -32 AFF88B34 0000FFFF 1
read -32 AFF88B34 1
jump AFC82450
```

Observed:

```text
AFF88B34: FFFFFAAF -> 0000FFFF
```

Result:

- Write stuck in EBOOT-visible RAM.
- No kernel debug output appeared after `jump AFC82450`.

Conclusion:

- The known runtime DBGPARAM mask is not the right pre-boot patch target.
- CE likely initializes or relocates the DBGPARAM table after handoff.
- Use `A00FFC10=0x11` for boot-time serial debug logs.

## RTL8192CU Crash Evidence

With debug logs unlocked, the RTL8192CU USB Wi-Fi driver crashes during NDIS
bind/adapter activity.

Observed early crash:

```text
Exception 'Access Violation' (2)
PC=c0ce0a20(rtl8192cu.dll+0x00010a20)
RA=c06b9bf8(ndis.dll+0x00019bf8)
BVA=00000670
```

Disassembly:

```asm
c0ce0a20: lw $17, 0x670($16)
```

Interpretation:

- `BVA=0x670` means `$16 == NULL`.
- NDIS/Realtek path is calling a routine with a null or uninitialized adapter
  context.

Observed later crash:

```text
Exception 'Access Violation' (3)
PC=c0ce3cd0(rtl8192cu.dll+0x00013cd0)
RA=c0ce3cbc(rtl8192cu.dll+0x00013cbc)
BVA=00200214
```

Disassembly around later crash:

```asm
c0ce3cbc: addiu $15, $20, 0x4
c0ce3cc0: sw    $2, 0x0($23)
c0ce3cc4: lw    $8, 0x4($15)
c0ce3cc8: sw    $8, 0x4($17)
c0ce3ccc: sw    $15, 0x0($17)
c0ce3cd0: sw    $17, 0x0($8)
c0ce3cd4: sw    $17, 0x4($15)
```

Interpretation:

- Queue/list insert path.
- Faulting store uses a corrupt queue tail pointer.
- This is consistent with earlier failed or partial adapter initialization.

Observed NDIS-side crash variant:

```text
Exception 'Access Violation' (3)
PC=c06ad964(ndis.dll+0x0000d964)
RA=c0ce1620(rtl8192cu.dll+0x00011620)
BVA=00720060
VM-active='PANManager.exe'
```

Interpretation:

- The Realtek driver can also hand a bad pointer/state back into NDIS.
- This reinforces that the failure is in the adapter bind/state lifecycle, not
  only one isolated Realtek queue helper.

Relevant registry load clients:

```text
HKLM\Drivers\USB\LoadClients\3034_20616\Default\Default\RTL8192CU
    "Dll"="RTL8192CU.dll"

HKLM\Drivers\USB\LoadClients\3034_33144\Default\Default\RTL8192CU
    "DLL"="RTL8192CU"
```

Disabling those values would avoid the driver path, but that is only a
stabilization step. It is not the final Wi-Fi fix because the same hardware once
loaded, scanned, connected, and browsed successfully.

Current likely root causes:

- Power/reset sequencing around the USB Wi-Fi chip.
- App timing between `PANManager.exe`, `WiFiManager.exe`, WZC, and NDIS bind.
- Runtime state corruption after a failed Realtek attach path.
- Stale WZC or Thinkware Wi-Fi state.

Additional reverse evidence:

- `RTL8192CU.dll` registers an NDIS miniport table in `DriverEntry`
  (`c0ce18ac`). With the CE `NDIS40_MINIPORT_CHARACTERISTICS` layout from
  `C:\WINCE600\PUBLIC\COMMON\DDK\INC\ndis.h`, the callback map is:

```text
CheckForHangHandler    c0cdf2bc
HaltHandler            c0ce09e8
InitializeHandler      c0ce1a18
QueryInformation       c0d5b10c
ResetHandler           c0ce2034
SendHandler            c0cdf7f4
SetInformationHandler  c0cdfb5c
ReturnPacketHandler    c0d07588
```

- Therefore the `PC=c0ce0a20, BVA=00000670` crash is in the Realtek
  `MiniportHalt` callback. NDIS is entering halt with a null
  `MiniportAdapterContext`, and Realtek dereferences it immediately:

```asm
c0ce0a0c: move  $16, $4
c0ce0a18: addiu $19, $zero, 1
c0ce0a1c: move  $4, $16
c0ce0a20: lw    $17, 0x670($16)
```

- A narrow experimental driver patch copy was created at:

```text
target\RTL8192CU_haltguard.dll
```

Patch details:

```text
Original DLL: D:\INAVI_Emulator\DUMPPLZ\Windows\RTL8192CU.dll
ImageBase:    c0cd0000
.text raw:    00000200
VA:           c0ce0a18
RVA:          00010a18
file offset:  0000fc18

original bytes:
01 00 13 24  25 20 00 02  70 06 11 8e

patched bytes:
b4 00 00 12  01 00 13 24  70 06 11 8e
```

Patched disassembly:

```asm
c0ce0a18: beqz  $16, 0xc0ce0cec
c0ce0a1c: addiu $19, $zero, 1
c0ce0a20: lw    $17, 0x670($16)
```

This only guards `MiniportHalt(NULL)`. It should prevent the first kernel crash
without disabling Wi-Fi or changing the normal non-null halt path. It does not
claim to fix the later transmit queue corruption at `c0ce3cd0`; that later
failure may become the next observable crash once the null-halt path is masked.

`WifiManager.exe` evidence:

- `WifiManager.exe` imports `wzcsapi.dll` and directly calls
  `WZCEnumInterfaces`, `WZCQueryInterfaceEx`, `WZCRefreshInterfaceEx`,
  `WZCSetInterfaceEx`, `WZCDeleteIntfObjEx`, and `WZCPassword2Key`.
- Its embedded log strings include `strLog.txt`,
  `IOCTL_NDIS_BIND_ADAPTER`, `IOCTL_NDIS_UNBIND_ADAPTER`,
  `IOCTL_NDIS_REBIND_ADAPTER`, `WZCRefreshInterfaceEx Error`,
  `WZCQueryInterfaceEx Error`, `RTL8192CU1`, and `RTL8712U0`.
- This supports the current theory that Wi-Fi on/off/search drives NDIS
  unbind/rebind and WZC refresh/query activity, which can hit the Realtek
  partial-init/halt bug.

## Preferred Next Experiments

1. Use `A00FFC10=0x11` as the standard debug-log unlock.
2. Determine BootArgs layout after `A00FFC10`, especially whether the next words
   match `DEVICE_LOCATION`.
3. Only then try KITL with a valid serial or ethernet device location.
4. Keep RTL8192CU enabled while tracing first failure; disable load-client keys
   only when a stable non-Wi-Fi boot is needed.
