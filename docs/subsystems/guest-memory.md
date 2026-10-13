# Guest memory and AddressSpace

Status: current byte-boundary behavior is separate from the proposed ownership model.
Authority: [shared architecture](../ARCHITECTURE.md). Sequence: [migration plan](../PLAN.md).

## Current source and facts

- [memory.rs](../../src/ce/memory.rs) holds allocation/virtual metadata; emulator
  [memory.rs](../../src/emulator/memory.rs) defines region layout and permissions.
- [pe_loader.rs](../../src/emulator/pe_loader.rs) creates mapped blobs. Native mapping,
  spillover, shared-view synchronization and byte adapters live in
  [unicorn.rs](../../src/emulator/unicorn.rs); [native_engine.rs](../../src/emulator/native_engine.rs)
  is the quarantined owner/ledger experiment, not a replacement memory authority.
- Checked guest-backed raw/native initial/reallocation paths establish bytes before
  publication; metadata-only helpers still exist. Logical zero reserves one physical byte.
- Failed prepared candidates leave monotonic holes. Existing heap free does not prove
  address reuse, per-process ownership, immutable reserved extent or ABA protection.
- Four host-queued typed reservations materialize at delivery. Immediate guest-pointer
  publication must not bypass writable native backing or initialized payload bytes.
- Persisted spillover can mirror a global high-water mark into multiple CPU images;
  live heap metadata, blob capacity, native backing and host RSS are different measures.

## Target authority and permitted interfaces

- `CeMachine` owns mutation sequencing; each `AddressSpace` owns its process's regions,
  backing objects, protections and native mapping ledger. No CPU-local allocator authority.
- A caller's shared `TaskIdentity` resolves through the canonical lifecycle directory;
  ownership cannot come from a transient current PID or copied ThreadObject.
- Access also identifies the shared `FrameIdentity` when retaining a guest buffer
  across a wait/callback. Pointer value alone is not lifetime identity.
- Expose checked read/write/map/protect/unmap and prepared allocation operations with
  explicit ownership, extent and permission validation. Return typed failure, not success
  inferred from host writes that happen to bypass native page protections.
- Allocation publication, initialized bytes and native backing commit form one defined
  boundary. Partial writes to already-live output are not implicitly transactional.
- Shared mappings identify canonical backing explicitly; private RAM at the same VA in
  different AddressSpaces remains independent. Clone copies bytes, not mutable engines.
- Memory operations do not advance `GuestInstant`, sleep in `HostInstant`, select a
  Task or manufacture a scheduler wake. Those effects are authorized machine events.

## Ownership, lifetime and cleanup

- Retain an authoritative original reservation extent and allocation incarnation before
  implementing reuse or reclamation. Live logical size is not the reservation footprint.
- Reject a stale prepared token/frame even if another allocation has the same address.
  Never infer free pages from a high-water gap or another process's allocation snapshot.
- A run lease receives the correct outgoing mapping ledger and returns it only after
  checked reconciliation/synchronization; process rotation cannot attach it to incoming RAM.
- On the proposed fresh-engine path, `KernelMemoryMappings` is rederived per
  owner rather than carried across native-owner closure: old `virtual_pages`
  would otherwise request unmapping pages absent from the new engine. The
  native byte readback and canonical backing must complete before closing the
  outgoing owner or rotating to a different process.
- Reconcile exact current native map extents, removed pages, protections and runtime-added
  pages. Do not remap an existing region or retain a freed virtual page by omission.
- Preserve canonical shared-view ordering and host edits to parked blob bytes. Same blob
  length/name/count is insufficient evidence that native code/data are unchanged.
- Failure before pointer publication retains the old live allocation/queue/context.
  Classify invalid metadata/protection separately from identifiable allocation exhaustion.
- Native reconcile/write/sync failure makes the attempt unusable; never silently reuse
  a half-updated ledger. A destructor cannot turn an error into a successful commit.
- Keep mappings alive while any valid frame retains access. Teardown follows the machine's
  authoritative lifetime, not a diagnostic snapshot or a guessed callback stack address.

## Dependencies and failure boundaries

- Depends on canonical process/task/frame identity, loader region plans and backing
  lifetimes. CPU engine consumes this interface, not the raw allocator's internal maps.
- API shims request checked memory effects; they cannot directly expose unmapped heap
  candidates or treat unknown guest pointer payloads as host-owned typed allocations.
- Loader rollback uses a defined mapping transaction. CPU snapshot restore does not
  restore RAM; a machine checkpoint includes separately versioned memory/backing state.
- `RunOutcome` must report mapping/native errors explicitly, not count/deadline/yield.
  Unexplained PC zero and guest memory faults remain errors, not guessed termination.
- `SnapshotEpoch` binds reported byte/mapping owners. Non-atomic PID brackets and retained
  remote contexts cannot prove simultaneous ownership or attribute all host memory.

## Persistent experiment and unverified risks

The failed 650 persistent-owner run, native arena/commit-tail controls and
cross-owner same-VA/FPU diagnostic are in the [ledger](../evidence/ledger.md)
(E-2 to E-6, E-10). The E-10 synthetic persistent-owner result does not close
the Stage 2 fresh-engine private/shared-view gate. Design consequences:

- A read-only source audit found `sync_mapped_blobs_from_unicorn` at the slice
  end, but no corresponding readback of ordinary guest-written `VirtualAlloc`
  pages; their `initial_bytes` are only seeded into native memory on mapping.
  The ignored E-18 two-slice MIPS control now demonstrates that loss for an
  internal `MemorySystem::virtual_alloc` page: the first owner's native RAM
  held guest-written `0x11223345`, but after `cfg(test)` idle-owner closure the
  fresh owner's load returned the unchanged `initial_bytes` seed
  `0x11223344`. Both slice PCs, normal stops and no-fault checks passed.
  This is an expected-red private-byte boundary, not a Context failure or a
  production fresh-default change ([ledger E-18](../evidence/ledger.md),
  local `evidence/stage2-o-m1-private-fresh-20261013/expected-red.receipt.md`).
  Establish checked outgoing readback and owner-qualified canonical private
  backing before fresh becomes default; public COREDLL `VirtualAlloc` and
  cross-process private isolation still require separate controls.
- Read-only E-18 production audit: `MemorySystem` allocations/lazy seeds are
  global and base-keyed, mixing private `VirtualAlloc` and file-view backing;
  the native page ledger is VA-only. A safe readback gate needs captured owner,
  backing kind and allocation incarnation, exact mapped-page bytes before
  owner finish/rotation, and staged failure handling. Mapping any page can
  currently reseed the entire `initial_bytes` range, risking overwrite of a
  guest-written page; free/reallocation at the same VA can retain old native
  bytes. `self.saved_context` is published before byte synchronization, so
  context/byte commit ordering must become transactional. These are source-
  audit hazards and design requirements, not executed fixes or new failures.
- The historical E-14 two-PID, two-native-slice red showed foreign-view
  seeding and base-only readback at one injected guest VA. The narrow E-16
  source captures the native attempt's process owner for view seeding and
  carries exact mapping-handle/view identity through readback, including a
  closed handle with a live view. It rejects duplicate owner/base and stale
  view identity; an ambiguous global lazy seed fails closed. E-14 is now an
  unignored passing default regression, and focused closed-live/ambiguous-lazy
  and outgoing-before-rotation controls pass. The final O-M1 source also
  passed 1,631 release tests (0 failed, 3 ignored), both no-trace release
  checks and 1,239 no-feature tests (0 failed, 1 ignored). Exact frozen
  source/log identities are in local
  `evidence/stage2-o-m1-same-va-20261013/owner-qualified-native-view.receipt.md`
  ([ledger E-16](../evidence/ledger.md)). This proves injected native-view
  ownership only: public same-VA `MapViewOfFile`, global lazy page namespace,
  native lazy fault repair, private `VirtualAlloc` persistence across fresh
  owner closure, and app readiness remain open. The immediate CreateProcess
  sync-error latch is source-reviewed but lacks injected-failure execution.

- The crash site is TranslationBlock initialization, not FlatView/subpage
  construction; no evidence confirms the cause, so mapping topology remains a
  separate test boundary.
- Pinned native map/protect/unmap can split RAM regions and rebuild dispatch
  topology. Long-lived topology/cache lifetimes need direct tests, not
  fresh-engine assumptions (Stage 8).
- Native translation-arena reservation is host memory, separate from guest
  virtual memory ownership; report it apart from guest mapped/reserved bytes and
  host RSS.
- A native CPU-only Context excludes RAM/device/IRQ state. Do not assume a
  context clone pins or reconstructs FlatView mappings.
- The isolated Stage 0 controls did not exercise production mapping
  reconciliation, subpage paths or the application workload.

## O-M1 same-VA file-view design gate (proposed)

The current kernel has one `MemorySystem` virtual-address cursor and a base-keyed
allocation map (`src/ce/memory.rs:28-38,457-503`), so public `MapViewOfFile`
cannot yet allocate the same VA in two processes. `FileMappingView` records a
PID but not a durable view identity; handle-table lookup, flush and unmap use
base alone (`src/ce/object.rs:175-194,690-744`). E-16 captures the selected
process owner and exact live view for native seed/readback only; broader
native mapping, fault repair, public view allocation and lifetime still lack
an authoritative `AddressSpace` identity. A filter on one sync helper is not
a complete O-M1 fix.

The smallest transitional key may be an explicit `AddressSpaceKey` backed by
the currently incremented PID, captured when the CPU slice starts and passed through
the virtual-allocation and view paths. Keep `CeKernel.current_process_id` and
`CurrentProcessState` as existing selectors for now; do not present a PID as a
durable process incarnation or reuse-safe identity. Replace this transitional
key with canonical `ProcessIdentity` before allowing PID reuse.

Staged implementation boundary, with the E-14 real-backend expected-red
collision control now turned green for injected native-resident views under
E-16: make virtual cursors, allocations, lazy seeds,
queries and native page ledgers owner-scoped without changing current heap
callers; resolve views by
owner plus exact mapping/view identity, including closed handles with live
views (native readback covered by E-16); then pass the captured owner through
native map/reclaim/fault beyond the view sync now covered. Update
`VirtualQueryEx` to resolve its requested process or fail explicitly,
and propagate ambiguous/stale-view and unexpected native sync errors. An
unmapped lazy page may defer its write only when its owner-qualified canonical
seed is retained; it must not be reported as a successful write to native RAM.

Acceptance requires public two-process `MapViewOfFile` calls for distinct files
to obtain the same VA and survive native A-to-B-to-A read/write slices without
cross-file byte changes, including lazy fault repair and outgoing readback.
Separate controls must prove same-mapping alias visibility, flush and unmap
while a mapping handle is closed but its view remains live, owner-specific
`VirtualQueryEx`, and a foreign view not overwriting private RAM at the same VA.
A fresh-owner two-slice guest-written private `VirtualAlloc` round-trip is now
an executed expected-red gate under E-18. It must turn green with authoritative
byte backing; the E-16 view controls alone cannot establish it.

## O-M1 backing recommendation: host-owned slabs via `mem_map_ptr` (supervisor, 22:15)

Source facts (working tree, 2026-10-13):

- **Two writable truths.** Guest bytes reach a native engine by copy-in
  (`uc.mem_map` plus writes from blobs or `initial_bytes`). They leave by
  copy-back (`sync_mapped_blobs_from_unicorn`,
  `sync_file_mapping_views_from_unicorn`). For the length of a slice, host
  blobs and native RAM are both writable copies of the same guest page,
  which anchor A4 says to avoid.
- **E-18 is that gap.** Private `VirtualAlloc` pages have no copy-back, so a
  fresh owner reseeds from `initial_bytes`.
- **Copy-back is expensive.** Pinned Unicorn 2.1.5 has no dirty-page API, so
  copy-back has to rescan every mapped page per slice or add write hooks.
  Write hooks are the per-instruction cost Phase 0 removed (ledger E-9).
- **Slabs already work.** The ignored diagnostic
  `native_mem_map_ptr_boxed_slab_survives_splits_and_fresh_owner`
  (`native_engine.rs`) shows that a fixed host slab mapped with `mem_map_ptr`
  takes guest stores directly. It also survives a `mem_protect` split and is
  visible to a fresh owner with no copy.

Recommendation: decide O-M1 as **host-owned, page-aligned, address-stable
backing mapped with `mem_map_ptr`**, with the AddressSpace as its only owner.

- One truth per page. Native code and host shims read and write the same
  bytes. A fresh engine per slice maps the existing slabs, so nothing is
  reseeded and nothing is read back. E-18 closes structurally.
- Shared file views map the *same* slab into each process that has a view
  of it. Private pages map per-process slabs. Same-VA isolation follows from
  which slab is mapped, not from copy ordering (E-14 and E-16).
- Per-slice cost becomes mapping calls instead of copying the mapped
  footprint (hundreds of MiB for iNavi). That also makes the fresh lifetime
  (O-P4) cheaper.
- The `unsafe` stays in one module, with these invariants:
  - a slab never moves or resizes (`Box<[u8]>` or a page arena, never a
    growing `Vec`);
  - every engine that maps a slab is dropped or unmapped before the slab;
  - host access happens only while no `emu_start` is running, or from hooks
    through the existing bounded lease;
  - protection changes go through `mem_protect` on the mapping, never through
    a re-seed.
- Migrate in one connected change per backing family:
  1. Private `VirtualAlloc`: E-18 must turn green.
  2. File views: E-14 and E-16 must stay green.
  3. Image/heap blobs.

  Delete the matching copy-in and copy-back paths in the same change, so the
  old paths never linger beside the new ones.
- Promote the slab diagnostic to a normal test, because it is the
  foundation. Add the same-VA and fresh-owner controls on real Kernel
  allocations.

Prefer this to adding "checked outgoing private-byte readback before owner
finish". Readback keeps two truths and adds per-slice copy cost and failure
staging, all of which the slab design removes.

## Telemetry and executable migration gates

- Report per-AddressSpace mapped/reserved/committed bytes, blob length/capacity, live
  logical/physical heap and shared backing separately from native TCG and host RSS/private.
- Record mapping generations, allocation/frame incarnations, permission changes and
  checked failures under `SnapshotEpoch`; retain bounded events, not huge history dumps.
- Preserve current zero-size, ZERO, non-ZERO, grow-prefix/tail and failure-injection tests.
- Add same-VA private isolation, shared visibility, sparse spillover and external parked
  edits; a two-slice guest-written private `VirtualAlloc` byte round-trip; stale
  token/frame rejection; partial protect/unmap/remap; clone/checkpoint tests.
- Exercise native reads after publication and fresh/reused engine reconciliation, not
  metadata-only assertions. Long lifecycle controls must include post-owner-close sampling.
- AddressSpace/lifecycle authority precedes any heap reuse. Measured memory improvement,
  FlatView safety and ownership correctness are independent gates, not Phase 1 claims.
