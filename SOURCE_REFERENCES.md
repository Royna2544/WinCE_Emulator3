# Source References

Refreshed on 2026-10-13. Keep this file as a compact index of active evidence.
Do not expand it into a historical progress log.

## Local Reference Roots

- Windows CE 6.0 source/header tree: `C:\WINCE600`
- Prior emulator notes, when needed for comparison only:
  `..\WinCE_Emulator_v2`
- Checked-in registry/device fixtures:
  - `registry.reg`
  - `serial_devices.json`
  - `mounts.toml`

## Active Evidence Map

- Scheduler-core ownership and CE6 scheduling:
  - Common proposed contracts and migration gates: `docs/ARCHITECTURE.md`,
    `docs/PLAN.md`, `docs/subsystems/`. These are plans, not source parity proof.
  - CE6 `NK/KERNEL/handle.c` HNDLDuplicate774-829 shares PHDATA rather than
    copies; NKDuplicateHandle840-891 closes source when requested even on
    errors. `schedule.c`660-684 locks wait targets through wait execution.
    CE6 `thread.c`1016-1025 NKOpenThread duplicates the canonical thread
    handle into the active process; `thread.c`808-813 THRDGetProcID returns
    `pth->pprcOwner->dwId`, not the selected process. Two raw API expected-red
    controls confirm the emulator currently violates both boundaries.
  - `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\kcalls.c` KCNextThread
    (1200-1303): lower numeric priority, expired equal-priority quantum to tail,
    unexpired/zero-quantum preemption retention. `schedule.c` NKSleep(227-251)
    and NKWaitForMultipleObjects(644-656): zero sleep yields, finite rounding,
    and CE6 WaitAll rejection. First storage migration deliberately preserves
    current deadline/API behavior; these are next-stage parity contracts.
  - Pinned Unicorn2.1.5 `uc.c` native hook deletion(2036) and CPU Context APIs;
    `qemu/target/mips/unicorn.c` uc_init limits CPU context at end_reset_fields.
    Rust `unicorn-engine-2.1.5/src/lib.rs` from_handle/owner Drop(151-230),
    shallow Rc Clone(244), hook removal(1171), Context APIs(1182-1220).
  - Live migration: `src/emulator/native_engine.rs`, `unicorn.rs`,
    `src/ce/task_wait.rs`, `scheduler.rs`, per-thread cancellation in kernel.rs.
    Experimental650 failed; installed BED2 does not contain this migration.
  - Pinned Unicorn2.1.5 `qemu/accel/tcg/translate-all.c`:64-bit default1GiB
    arena, Windows VirtualAlloc reserve/nominal4MiB commitment, code_gen_alloc,
    tb_flush/tcg_region_reset_all; `uc.c` TCG_BUFFER_SIZE control2949.
    Hook invalidation does not rewind arena; flush does not decommit. Cache
    size must be set before native initialization; a later setter is not proof
    that allocated arena size changed. Six isolated Stage0 cases queried
    1,073,741,824B initial size; three hook-churn cases retained about243MiB
    sampled RSS until owner close. This is bounded fixture evidence, not
    allocation attribution for the failed app run.
  - Failed650 GUID-matched image/PDB (RSDS5C58B88E-4CAE-4ADB-8F71-147604275719,
    age1): WER RVA0x85e757 resolves to tb_gen_code_mipsel/translate-all.c1764.
    Exact disassembly writes0xffff to TB+0x50 after tcg_tb_alloc; exec-all.h
    jmp_reset_offset[2] matches. Earlier subpage_init labeling used adjacent
    generic BED2 PDB. A renamed image embeds the generic PDB basename; its
    adjacent PDB can be loaded without GUID/age validation. Validate the
    image RSDS GUID+age and the PDB actually loaded, then use the matched650
    pair. Corrected site is not cause.
  - Native Windows code_gen_buffer_handler translate-all.c883-898: nominal
    4MiB commit; tail branch computes base_end-base, not remaining arena.
    Isolated pre-init1/2MiB cases faulted during query at GUID-matched
    tcg_out8 RVA0xA2D31;8MiB/default tiny controls passed with actual arena
    queries. The identical code-emission fault is consistent with this handler
    arithmetic, but no dump supplies the fault address/handler return, and
    terminal650 fault address/handler results remain unavailable. A later
    native matrix supplied direct repaired-handler tail evidence; any
    production-size change still requires full-backend/app controls.
    Chat `validation/20261013-stage0-native-commit-tail-differential.md`
    records a separate copied dependency: pristine-handler C SHA256
    675E42DFD5E09A70679DD7325E57BFFDDEFE235038738A6DBE2BC74A62C77F50
    versus bounded-handler C SHA256
    645EE381E9C2149E7A94B91BF255818E5D04900A8CDDC4038FDA533BCD2AABDF.
    Both use the same post-uc_ctl read accessor/fixture; only handler range
    arithmetic differs. Repaired1/2/8MiB/default smokes passed; pristine
    1/2MiB and the8MiB tail probe AVed. Repaired8MiB translated into
    committed second-half pages before a TB flush. That first differential
    had no direct VEH fault offset/request or650 linkage. Later isolated
    2x2 Win64 matrix telemetry directly measured a second fault at arena
    offset4,194,328, request4,194,280B and successful commit in24/24 owned
    controls; a leaf-tail-jump thunk passed seven more controls with matching
    counters. Evidence: chat `validation/native-tail-veh-abi-matrix-1013/REPORT.md`
    and `validation/native-tail-leaf-thunk-1013/README.md`. The pinned mirror
    above remains unchanged and no production-size setting was changed. A
    synthetic logical4MiB+1 final-byte touch, using Windows VirtualAlloc
    page rounding and VirtualQuery state, passed twice with a direct one-byte
    clipped request and no commitment beyond the rounded reservation; normal
    TCG translation does not reach the guarded final page. Chat evidence:
    `validation/native-tail-page-rounding-1013/README.md`. A separate
    single-threaded leaf-thunk lifecycle fixture passed two fresh owned runs
    of eight owner pairs with successful handler removal, former closure/
    arena release and a surviving owner's clipped tail; in-flight concurrent
    callback teardown remains open. Chat
    `validation/native-tail-veh-lifecycle-1013/README.md`.

- Native MIPS branch handoff and helper replay:
  - Unicorn2.1.5 `qemu/target/mips/unicorn.c`, PC register write changes PC/M16
    but not pending branch fields; `cpu.h` TB flags and `translate.c`
    pending-delay reconstruction/`clear_branch_hflags` govern completion.
  - Local generated branch/JAL helper sequences: `src/emulator/cpu_mips.rs`
    `normal_branch_stub_words` and `jal_stub_words`; scheduling eligibility
    and genuine native recipient/caller regressions in `unicorn.rs`.
- Heap allocation/reallocation:
  - `C:\WINCE600\PRIVATE\WINCEOS\COREOS\CORE\LMEM\heap.c` wrappers and
    `rheap.cpp` free-block search/reuse (1583-1704), allocation zeroing (1639),
    grown-tail zeroing (1943-1945), copy before old-block free (1955).
  - Local prepared reallocation and guest mapping effects:
    `src\ce\memory.rs`, `coredll.rs`, `crt.rs`, `src\emulator\unicorn.rs`.
- CE notification wire size:
  - True device `D:\INAVI_Emulator\DUMPPLZ\Windows\coredll.dll`, export 481,
    RVA 0x4dd18: VA 0x4005dd48 loads the explicit 0x98-byte marshal size.
  - SDK `STANDARDSDK_420\Include\Mipsii\shellapi.h`, wide notification
    typedef: six 32-bit fields plus 64 UTF-16 characters (152 bytes).
- Loader and PE behavior:
  - CE loader/import/resource headers and implementation references under
    `C:\WINCE600`
  - Local implementation under `src\emulator\`, `src\pe\mod.rs`, and loader-facing
    tests under `tests\`
- Shell, image-list, icon, and notification behavior:
  - CE shell/taskbar/resource sources under `C:\WINCE600\PUBLIC`
  - Local implementation under `src\ce\shell.rs`, `src\ce\coredll.rs`, and
    raw kernel/GWE tests
- GWE, menu, modal, message, and scheduler behavior:
  - CE GWE, window, menu, and message-loop references under `C:\WINCE600`
  - Local implementation under `src\ce\gwe.rs`, `src\ce\thread.rs`, and
    `src\emulator\unicorn.rs`
- Storage and FSDMGR behavior:
  - Bounded CE disk-cache block/replacement implementation:
    `C:\WINCE600\PRIVATE\WINCEOS\COREOS\STORAGE\DISKCACHE\cacheapis.cpp`
    and `cache.cpp`; the Rust HostFile path uses its existing 64 KiB window,
    not a hardware-cache size inferred from registry `CacheSize=0`.
  - CE storage, AFS, FSDMGR, disk, FMD, and file-notification references under
    `C:\WINCE600`
  - Local implementation under `src\ce\kernel.rs`, `src\ce\coredll.rs`, and
    memory/file tests
- GDI, DIB, palette, text, and IME behavior:
  - CE GDIAPI, draw, font, palette, IMM, and TESTIME references under
    `C:\WINCE600`
  - Local implementation under `src\ce\resource.rs`, `framebuffer.rs`, `coredll.rs`, and raw
    GWE/kernel tests
- Runtime iNavi investigation:
  - Device command/wire evidence: dumped `YAS526B.dll` under
    `D:\INAVI_Emulator\DUMPPLZ\Windows`, actual MIPS caller command/selector
    transactions, and Yamaha YAS529 single-address format reference.
    Implementation/explicit ideal-device limits: `src\ce\devices\magnetometer.rs`.
  - Current evidence should come from reproducible live runs, debug endpoints,
    target logs, and committed code changes.
  - Keep emulator policy generic; source references should describe CE/runtime
    behavior rather than app-specific shortcuts.

## Maintenance Rules

- Add a reference only when it supports active implementation or validation.
- Prefer precise file paths, function names, structs, constants, and observed
  call shapes.
- Remove references once they become stale, speculative, or disconnected from
  the active roadmap.
