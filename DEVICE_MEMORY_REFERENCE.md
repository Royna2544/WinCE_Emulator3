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

Conclusion:

- `A00FFC10` is enough for debug output.
- KITL likely needs valid `DEVICE_LOCATION` and transport-specific fields after
  the flags, not only additional flag bits.

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

## Preferred Next Experiments

1. Use `A00FFC10=0x11` as the standard debug-log unlock.
2. Determine BootArgs layout after `A00FFC10`, especially whether the next words
   match `DEVICE_LOCATION`.
3. Only then try KITL with a valid serial or ethernet device location.
4. Keep RTL8192CU enabled while tracing first failure; disable load-client keys
   only when a stable non-Wi-Fi boot is needed.
