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
