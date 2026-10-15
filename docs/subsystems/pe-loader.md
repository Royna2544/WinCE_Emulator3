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

## Stage 2 slice: sealed main-EXE image runs (source-bound gate passed)

The main EXE now takes a connected loader-to-native path, not just a page
selector. After main imports and MIPS trampolines are patched,
`final_main_image_runs` in [pe_loader.rs](../../src/emulator/pe_loader.rs)
consumes those **final** bytes into page-aligned `MappedBlob` runs. One-time
loader branch rewrites are sealed as executable bytes; they are not future
writes. `eligible_immutable_image_pages` chooses only complete pages inside
one readable, non-writable section; headers, mixed/partial pages and IAT thunk
pages stay copy-backed. Trampoline stub tails beyond `SizeOfImage` are not
eligible. Adjacent eligible pages with the same R or RX guest protection
become one sealed run. This E-43 slice covers the main EXE only: preloaded and
runtime DLLs, trap pages and mutable image pages remain copy-backed; at E-43
the initial heap and stack were copy-backed too, before E-44's separate fixed-
slab source gate. The fallback image runs remain RWX, so this change does not
assert a complete PE protection model.

`SealedImageBytes` and `BlobBytes` in
[pe_loader.rs](../../src/emulator/pe_loader.rs) retain one aligned host-RW
allocation per sealed run with no Rust mutable accessor. The original mapped
image used for occupancy planning is dropped before the final patched image
is built; partitioning allocates sealed runs while consuming that final image.
Peak loader memory still needs direct measurement. Unicorn requires the host
pointer to remain writable (`vendor/unicorn-engine-sys/include/unicorn/unicorn.h:1162-1164`);
only the guest mapping is R or RX. Host `uc_mem_write` can bypass guest
protection (`vendor/unicorn-engine-sys/uc.c:965-993`), so complete host-write
guard coverage remains a separate acceptance check.

| Path | Current source behavior and remaining gate |
| --- | --- |
| Final patch and publication ([pe_loader.rs](../../src/emulator/pe_loader.rs): `load_pe_image_with_dlls`, `final_main_image_runs`, `partition_main_image_runs`) | Final loader-patched branch bytes may be sealed. IAT thunk pages remain mutable; sealed and mutable runs replace the retained full-image blob. Section selection is by flags, not names. |
| Fresh/reused native owner ([unicorn.rs](../../src/emulator/unicorn.rs): `expected_sealed_image_runs`, `reconcile_persistent_native_memory_with_private`, `map_sealed_image_runs`; [native_engine.rs](../../src/emulator/native_engine.rs): `NativeMappingPins::pin_image`) | Exact process birth, image generation, base, extent, host pointer and permissions are recorded. Fresh owners pin before one `mem_map_ptr` per run; reused owners reject ledger/pin/protection drift. Reconciliation excludes sealed pages from ordinary `mem_map`. Native close/pin failure gates still require validation. |
| Copy-in and readback ([unicorn.rs](../../src/emulator/unicorn.rs): `sync_persisted_blobs_to_native`, `sync_mapped_blobs_from_unicorn`) | Both skip sealed runs; mutable blobs still take checked copy/readback paths. The synthetic gate must verify bytes and actual call counts, not infer savings from logical `blob_mb`. |
| Host writes and VM views ([unicorn.rs](../../src/emulator/unicorn.rs): `SealedHostWriteGuard`, `UnicornGuestMemory`, `sync_file_mapping_views_to_unicorn_guarded`) | Guarded writes reject sealed-range overlap, but the full set of direct and callback write sites is under audit. Do not treat guest RX alone as host-write protection. |
| Code readers and clone ([unicorn.rs](../../src/emulator/unicorn.rs): `MappedCodeIndex`, `cached_fast_start_indirect_transfer_sites`; [pe_loader.rs](../../src/emulator/pe_loader.rs): `BlobBytes::clone`) | Code readers use run pointers and the indirect-site cache includes sealed generation; clones share sealed `Rc` storage while Owned runs clone independently. Cross-run reads and generation churn remain regression gates. |
| Checkpoint ([main.rs](../../src/main.rs): `try_monitor_checkpoint_save`, `try_monitor_checkpoint_restore`) | Monitor save/restore explicitly returns `Unsupported` for sealed images until machine-memory restoration is proven. CPU-only native Context remains the precondition; memory-mode native snapshots need a separate identity/COW gate. |

The first synthetic `sealed_main_image_*` tests in
[unicorn.rs](../../src/emulator/unicorn.rs) now pass with production loader
selection, native pointer mapping, two owners, guest `WRITE_PROT`, a bounded
real MIPS slice and 200 fresh pin cycles. The focused
[native test log](../../evidence/sealed-main-image-20261014/test-unicorn-sealed-main-attempt-02.raw.log)
records 2 passed / 0 failed with the two profiles ignored by default; full
[Unicorn](../../evidence/sealed-main-image-20261014/test-full-unicorn-workspace-attempt-01.raw.log)
and [no-feature](../../evidence/sealed-main-image-20261014/test-full-no-feature-workspace-attempt-01.raw.log)
workspace runs also passed on that earlier source. The changed-page policy was
subsequently revised to seal final loader bytes, so these prior suites are not
final-source signoff. The final-source [E-43 receipt](../../evidence/sealed-main-image-20261014/validation.receipt.md)
records **1,761/0/17 ignored** across 17 blocks in release with
`unicorn,trace,win32-desktop`, paired **1,303/0/13 ignored** across 17 blocks
in debug with no features, and a successful `unicorn,win32-desktop` no-trace
check. Focused patched-branch, mutable-IAT and native-write overlap controls
also pass. This is a source-bound gate, not an iNavi app or writable-mount run.

The two opt-in 56-MiB profiles use the same sparse synthetic PE and production
loader/native setup. Only the section write bit differs: RX selects sealed
storage and RWX selects the copy-backed control. Their guest permissions are
therefore **not identical**, and these are setup-path microprofiles, not an
iNavi timing comparison. The first direct debug-test measurements were:

| Path | Retained Owned blob capacity | Fresh / reused setup | Fresh pointer map | Fresh blob write / outbound readback |
| --- | ---: | ---: | ---: | ---: |
| [RX sealed](../../evidence/sealed-main-image-20261014/profile-56mib-rx-attempt-01.raw.log) | 21,053,440 B | 24.930 / 34.974 ms | 1 call, 58,712,064 B | 21,053,440 / 21,053,440 B |
| [RWX copy](../../evidence/sealed-main-image-20261014/profile-56mib-rwx-attempt-01.raw.log) | 79,765,504 B | 17.810 / 29.566 ms | 0 | 79,765,504 / 79,765,504 B |

The RX setup microprofile was **slower** in both cases; this gate does not
establish a setup-speed improvement. RX also retains 58,712,064 bytes of
sealed backing outside the Owned-capacity column, so the smaller Owned figure
is not by itself a whole-process memory saving. In separate direct Windows
debug-test processes, 50-ms samples recorded maximum private bytes of
111,824,896 RX versus 179,552,256 RWX, and working sets of 115,429,376 RX
versus 173,776,896 RWX ([RX samples](../../evidence/sealed-main-image-20261014/profile-56mib-rx-os-attempt-01.metrics.txt),
[RWX samples](../../evidence/sealed-main-image-20261014/profile-56mib-rwx-os-attempt-01.metrics.txt)).
These are sampled maxima, not kernel-maintained peaks, release-build results
or iNavi measurements.

A later [frozen E-43 RX setup probe](../../evidence/sealed-main-image-20261014/profile-56mib-rx-pre-heap-stack-attempt-01.raw.log)
passed 1/0 and recorded 21,049,344 B retained Owned capacity and fresh
write/readback, with 33.246 ms fresh and 37.971 ms reused setup. It is one
debug synthetic pre-heap/stack-slab baseline, without a same-source RWX
control or any app-speed inference.

E-44's fixed initial heap/stack slabs leave the sealed main-image policy
unchanged. The post-slab synthetic RX release and debug controls both pass
1/0 with **77,824 B** owned blob capacity, fresh blob writes and fresh/reused
readback, versus **21,049,344 B** in the pre-slab debug control: an exact
**20,971,520-B (20-MiB)** logical blob-byte reduction. The post-slab debug
fresh/reused setup samples are 16.740/25.249 ms versus prior 33.246/37.971
ms; single samples and fixed-slab reservation outside the timed setup section
do not establish a speed or host-RSS gain
([E-44 receipt](../../evidence/heap-stack-20261014/validation.receipt.md)).

A read-only production-loader pass over the actual iNavi EXE and seven DLLs
now seals **1,379 of 2,168** total image pages, all **1,379 of 1,379**
non-writable-section candidates: **5,648,384 B (5.386719 MiB)**
([opt-in coverage and blob log](../../evidence/sealed-main-image-20261014/coverage-real-inavi-loader-attempt-06.raw.log)).
Of those sealed pages, **1,224** contain final bytes changed by loader
patching; the other 155 match the original mapping. The earlier 155-page,
0.605469-MiB [result](../../evidence/sealed-main-image-20261014/coverage-real-inavi-loader-attempt-02.raw.log)
is historical evidence for the superseded byte-equality filter, not current
coverage. The probe constructed only an owner-side
memory/blob plan; it did not create a native CPU, execute guest instructions,
or launch iNavi.

The E-43 load-only probe accounted for one initial owner's **26 mapped blobs,
36,679,680 B (34.980469 MiB)**. That includes the 16-MiB `ram:ce-heap-arena`
and 4-MiB `ram:guest-stack` backing, seven preloaded DLL images and their
trampolines, the main image runs, and small trap/KData blobs. The main image
has 13,344,768 B of retained runs, including the 5,648,384 B sealed above.
Initial `ram:heap-spillover` is zero; later spillover and runtime-loaded DLLs
cannot be inferred without guest execution. E-38's PID-1 mean `blob_mb` of
55.78 MiB is an older live profile of integer-truncated per-slice totals,
not an exact byte census or a matched performance comparison. No app memory,
setup-time, or frame-rate improvement is claimed from this load-only result.

ROM/XIP physical-budget treatment and `GlobalMemoryStatus` accounting are
undecided. A guest protection fault is still a native error, not a CE access
violation. Dynamic DLLs, production writable-image integration, CE exception
translation, the exhaustive direct native-write inventory and matched iNavi boot/frame
timing remain open; another writable-mount iNavi run requires fresh approval
and adjacent backups.

## Writable image and DLL source-cut plan (opt-in main-image gate only)

The normal production nonsealed main-image run is a mutable RWX `Owned` blob;
preloaded and runtime DLLs likewise publish whole-image `Owned` blobs. The
fresh native owner copies these bytes in and reads them back. A separate
[opt-in synthetic source gate](../../evidence/stage2-writable-image-green-20261016/validation.receipt.md)
now exposes final main-image run identity/load generation and consumes the
matched mutable runs into stable VM backing after checked live-birth/lineage
validation; its real-MIPS write survives a fresh native owner without image
copy-in/readback. The ordinary SD app loader does not invoke that seam.
Production migration must preserve
the finalized relocation/import/trampoline bytes while replacing **one**
copy-backed family at a time with AddressSpace-owned, stable private pages.
The sealed main-image R/RX runs and their generation/pin checks remain intact.

1. Prepare an exact `(process birth, module identity, load generation, page)`
   plan after final loader patching. Check region overlap, final protection,
   page provenance, budget decision and native-write destinations before
   publishing any module pointer or IAT binding. Keep the old live module
   untouched on a failed candidate; do not reuse a same-VA generation.
2. Complete writable **main-image** production integration as the first
   connected source/test cut.
   The VM must own bytes before execution; native owners pin/reconcile that
   backing, and neither a parked blob nor outbound readback may overwrite it.
   Preserve IAT writes through a checked VM/loader transaction. Validate
   actual MIPS store/load across fresh owners and A/B same-VA isolation, plus
   stale-key and partial-map failures, before removing the Owned copy from
   the production path. The bounded [mapped-code gate](../../evidence/stage2-mutable-code-fallback-20261016/validation.receipt.md)
   now reads mutable code from live native memory and refuses a stale mutable
   blob fallback if native reading fails; the real-MIPS same-slice control
   passed. The [synthetic second-run map failure](../../evidence/stage2-writable-image-native-failure-20261016/validation.receipt.md)
   now retains exact pins through checked abort-close without publishing a
   reusable continuation; it does not test a failed native close. The
   [legacy file-view overlap fence](../../evidence/stage2-writable-image-file-view-20261016/validation.receipt.md)
   refuses seed/readback collisions with active or retired pinned image runs.
   Keep static classification tied to proven immutable backing; fence other
   host-write and exact exit/module-retirement paths before production
   activation.
3. Move **preloaded DLLs**, then **runtime DLLs**, through the same authority.
   Dynamic mapping, import refresh, attach/detach, unload/reload and parked
   process restoration must reconcile exact generation/pins before returning
   to guest code. A live return frame prevents premature unmap; a failed
   attach or native map must not publish executable metadata or lose the old
   module. The current runtime path records `memory_map` before native map
   and its cleanup does not prove unmap success, so rollback and retained
   pins require a targeted failure test before this family migrates. The
   scoped `FreeLibrary` inventory did not find a matching blob/native unmap;
   test the actual unload/reload lifetime before claiming reclamation. Check
   direct `uc.mem_write`, restore and rebind sites explicitly;
   guest RX permissions alone do not constrain host writes.
4. Close [O-M3](../DECISIONS.md) with evidence of actual ROM/XIP versus RAM
   image provenance and charge timing before claiming guest-budget or live
   acceptance. For each proven case, test boot/load/write/unload
   `GlobalMemoryStatus` deltas and low-budget rollback. Preserve a separately
   reported host-RSS/native-map profile; do not call source tests an iNavi
   memory or speed improvement.

The configured iNavi EXE is an SD-mounted file, not a demonstrated XIP ROM
page. `\Windows` is a virtual mount without a host root; a DLL found in
`DUMPPLZ\Windows` through search paths is not therefore proven ROM-backed.
The app-directory DLL preload path also means one loader invocation can mix
module origins. Each module/page needs its own provenance rather than one
blanket DLL budget rule.

This is a production/DLL design sequence with one opt-in main-image source
gate, not a claim that the normal app path uses VM image backing or that
`AddressSpace` already owns every image page. The shared [memory plan](guest-memory.md)
defines byte/pin lifetime and the unresolved physical accounting boundary.

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
