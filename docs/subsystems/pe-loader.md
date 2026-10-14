# PE loader, modules and import metadata

Status: existing loader behavior and the target AddressSpace transaction are distinct.
Authority: [shared architecture](../ARCHITECTURE.md). Sequence: [migration plan](../PLAN.md).

## Current implementation

- [pe/mod.rs](../../src/pe/mod.rs) parses images; [pe_loader.rs](../../src/emulator/pe_loader.rs)
  constructs region/blob plans, relocations, resources, imports and MIPS trampolines.
- [dll_search.rs](../../src/emulator/dll_search.rs) resolves module paths;
  [imports.rs](../../src/emulator/imports.rs) holds traps/exports/forwarders.
- [cpu_mips.rs](../../src/emulator/cpu_mips.rs) generates interpreter-compatible helpers;
  [unicorn.rs](../../src/emulator/unicorn.rs) performs dynamic native DLL mapping,
  attach/detach callouts, unload cleanup and live import metadata refresh.
- Dumped device DLLs are the runtime source of truth. SDK headers/libs provide ABI
  evidence, not substitutes for an available target binary or guessed behavior.
- Current layout contains CE5-style addresses although the target is CE6. A loader
  address choice is implementation evidence, not a normative CE6 memory-layout contract.

## Next Stage 2 slice: sealed immutable image pages (design only)

The loader currently publishes each patched image as one copy-backed `MappedBlob`
and maps its whole region RWX. After relocation, cross-DLL imports, main IAT
and trampoline patching finish, select only complete 4-KiB pages whose PE
section characteristics are non-writable. Exclude mixed/header, writable
IAT/data, trampoline and runtime-loaded DLL pages from this first slice; do
not infer immutability from `.text`/`.rdata` names alone. The sealed final
bytes need one module-generation-qualified authority, pinned across each
fresh owner's `mem_map_ptr` lease with RX or R guest protection. Reconciliation
must skip the old map/copy/readback path for those pages and must not retain a
second writable full-image Vec. Clone and unload must respect pins and module
generation; the physical-budget treatment of ROM/XIP-style pages is still an
explicit decision, not an assumed `GlobalMemoryStatus` improvement.

Unicorn requires `uc_mem_map_ptr` backing to remain host-readable and
host-writable (`vendor/unicorn-engine-sys/include/unicorn/unicorn.h:1162-1164`).
Seal bytes through a Rust type that exposes no mutable access after publication;
only the guest mapping gets R or RX permissions. Do not host-`VirtualProtect`
the slab. Checked host writes must also reject any overlap with sealed pages,
because Unicorn's host `mem_write` can bypass guest R/RX permissions.

Checked post-load paths before publication (source anchors are the current
copy-backed implementation):

| Path | Required sealed-page behavior |
| --- | --- |
| Final patch/publication (`pe_loader.rs:132-257,320-338`; `unicorn.rs:6021-6041`) | Seal only after relocations, cross-DLL/main imports and trampolines; reject any patched-byte overlap not accounted for by eligibility. |
| Native reconcile/restore (`unicorn.rs:6436-6456,11909-12019,12039-12067`) | Never ordinary-`mem_map` or blob-`mem_write` a sealed page; pointer-map with an exact module-generation/page/pointer/perms identity and fail closed on reused-owner mismatch. |
| End-of-slice readback (`unicorn.rs:10808-10809,14598-14617`) | Skip sealed pages entirely; test zero readback attempts, unchanged final bytes and identical page pointer after a slice executes one. |
| Runtime DLL/import rebinding (`unicorn.rs:12849-12974`; `pe_loader.rs:397-414`) | Keep runtime DLLs copy-backed in this slice; refresh only the separate import-trap page and reject any attempted sealed-image overlap. Unload-pending is not checked native unmap/pin retirement. |
| Guest-pointer host writes (`unicorn.rs:50510-50578,50637-50650` and direct writes at `8347,16219,16710,17932,18028,19882,22728,22753,30983,31989`) | Check sealed-range disjointness before every host write, including prevalidated `VirtualProtect` output and fill; fail closed without changing another owner's bytes. Vendored `uc.c:965-993` shows host `uc_mem_write` temporarily bypasses guest protection. |
| VM/view seeding and readback (`unicorn.rs:14037,14253,14381-14475,14584`) | Assert disjoint ranges before host writes or view synchronization; do not let a legacy view alias a sealed image page. |
| Isolated probe (`unicorn.rs:32454-32489`) | Keep its synthetic code mapping in its own Unicorn engine; never borrow a live sealed page pin or write the source image. |
| Code/debug readers (`unicorn.rs:1112-1214,5943-5984,12157-12215,16423-16432`) | Read sealed authority without a retained full-image copy; make indirect-site cache generation-aware and preserve cross-page debug reads. |
| Clone/checkpoint (`ce/memory.rs:617-651`; `unicorn.rs:130-140`; `native_engine.rs:195-223`; `main.rs:2337-2369`) | Share immutable page `Rc`/pointer identity across lineages while writable image bytes remain independent. Until machine-level restore is proven, extend the monitor's explicit `Unsupported` gate for sealed images; test both lineages and writable isolation. |
| Native snapshot mode (`native_engine.rs:281-291`; `vendor/unicorn-engine-sys/uc.c:978-983`) | CPU-only Context is a precondition. Future memory-mode context or `uc_snapshot` must fail closed or pass a separate copy-on-write identity gate before sealed mappings are used. |

The storage candidate is one logical module blob with sorted page-aligned
`Sealed(Rc<aligned host-RW run>)` and `Mutable(Vec<u8>)` runs, not a second
full-image buffer. Consume the final patched Vec when partitioning it; share
sealed runs on CPU clone and deep-copy only mutable runs. Preserve full logical
extent for code-index, labels and cross-page reads; coalesce adjacent same-
permission sealed pages so setup does not issue a native map call per page.

Run expected-red section-eligibility and final-byte tests before changing the
loader. Native controls then execute one sealed page in two fresh owners with
the same pinned page identity and no per-owner blob write; a guest store must
raise native `WRITE_PROT` without changing bytes. Repeat 200 owner cycles and
map/finish failure cleanup with zero leaked lease pins. A synthetic roughly
56-MiB image setup benchmark can compare its own setup/copy/RSS counters; it
is not an iNavi timing result. Today a protection fault is a native error,
not a synthesized CE access violation. Dynamic DLLs, writable image/heap
pages, CE exception translation and matched iNavi setup/frame timing remain
separate gates; another writable-mount run needs fresh approval and backups.

## Target transaction and owner

- `CeMachine` sequences load/unload effects. An `AddressSpace` owns module mappings,
  native protections, symbols/import identities and executable generation.
- A module request carries the anchor's canonical `TaskIdentity`; a DLL lifecycle
  callback carries its shared `FrameIdentity`, not a guessed owner from current PID.
- Parse/resolve/relocate into a validated plan, preflight ownership/extents, then commit
  mapped bytes and metadata together. Do not publish executable pointers before backing.
- Address conflicts are checked against authoritative mappings, including sparse heap,
  stacks, shared views and runtime code—not just the last DLL base or blob count.
- Module identity and load generation distinguish an unload/reload at the same VA.
  A raw address cannot identify the current import/trampoline incarnation.
- Expose module load/unload, resolved export/import lookup and validated region plans.
  CPU engine consumes committed metadata; API shims do not mutate loader tables ad hoc.
- Loading does not independently select a Task, advance `GuestInstant`, or turn a
  `HostInstant` budget into guest lifecycle completion. Machine events own those decisions.

## Imports, native code and lifecycle frames

- IAT patches bind to an explicit ABI/export identity. Handle ordinals, names,
  forwarded exports, missing modules and no-resolve/datafile modes distinctly.
- A runtime-added module refreshes trap pages and trampoline metadata before execution.
  Native code cache/hook configuration must agree with the committed executable generation.
- Current scoped/hookless paths stop after newly mapped loader work so the next run can
  rebuild coverage. Removing that boundary requires executable proof, not cache assumptions.
- Helper raw PC, guest origin and validated call return PC are different values. Never
  normalize a helper interior into a saved continuation or repeat already-executed delay work.
- DllMain/attach/detach is real guest execution through a machine-owned frame. Preserve
  parent native CPU context and the frame's lifetime across waits or nested callbacks.
- Dynamic loader metadata must move with process parking and independent checkpoints.
  A cloned native CPU Context is not a clone of module bytes or AddressSpace mappings.
- Import-trap/debug observations carry `SnapshotEpoch`, actual Task/frame and module
  generation. A retained selected-process trace is not complete load/unload history.

## Failure, cleanup and dependencies

- Parse/relocation/resolve/map/write/protection/attach errors remain explicit. Define
  rollback for unpublished mappings and metadata; report incomplete teardown honestly.
- Do not leave an IAT pointing at an unloaded module, retain stale trampolines or let a
  failed attach return raw success merely because some bytes were mapped.
- Unload waits for valid code/frame references and follows machine lifetime authority.
  Do not reclaim guessed code ranges while a native frame still retains a return address.
- Native mapping failure produces an error `RunOutcome`, not deadline/count/normal exit.
  Preserve PC-zero and native-error handling; loader fallback must not hide a CPU fault.
- Depends on AddressSpace memory transactions, canonical identity, API/export definitions
  and guest callback execution. It does not own scheduler ready/wait queues or device state.
- Keep path resolution/mounted storage separate from guest module lifecycle and avoid
  behavior keyed on application-specific names, guessed callbacks or image presentation.

## Quarantined persistent-native evidence

The failed 650 run and the arena evidence are in the [ledger](../evidence/ledger.md)
(E-2 to E-6); no loader-specific root cause is proven.

- Runtime mapping/protect/unmap under persistent engines may rebuild FlatViews
  while cached code and native CPU state survive; fresh-engine acceptance cannot
  prove this path (Stage 8).
- Repeated scoped hook rebuilds can invalidate translations while retaining
  committed code storage. This is not measured loader memory attribution.
- The isolated controls exercised no loader/module churn or production
  reconciler; do not change native buffer/cache policy on their basis.

## Telemetry, migration and executable gates

- Record committed module identity/generation, mapped extents/protection, IAT/export
  resolution, lifecycle frame, rollback result and native coverage/cache invalidation.
- Tag events with the common epoch and sequencer; distinguish parse/plan/commit/attach,
  not a single ambiguous loaded flag. Bound logs and keep failed image provenance.
- Preserve static/dynamic/relocated high-address, ordinal/name/forwarder and malformed
  image tests, including actual MIPS calls through resolved IAT pointers.
- Add reused-engine load/unload/reload, same-VA generation rejection, partial-map failure,
  attach failure/nested callback, sparse-region overlap and parked-process isolation.
- Test host edits and new code through production mapping/import paths; metadata presence
  alone does not prove execution coverage or fresh native memory.
- Loader transaction extraction precedes removing the old rebuild boundary. Native
  lifetime/cache safety, module correctness and faster startup are separate gates.
