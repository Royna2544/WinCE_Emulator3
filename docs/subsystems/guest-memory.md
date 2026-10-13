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

- `CeMachine` owns mutation sequencing; each `AddressSpace` owns its process's
  regions, private backing, view records and desired protections. Canonical
  shared backing belongs to a VM/file-mapping object. The selected native
  owner/lease holds its observed map ledger and pins; VM alone computes and
  reconciles it, and owner closure discards it. No CPU-local allocator authority.
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
- A run lease holds the exact observed native-owner mapping ledger. Checked
  reconciliation/synchronization precedes completion; fresh-owner close
  discards that ledger, while a persistent lease may retain it only for the
  same owner. Process rotation cannot attach it to incoming RAM or treat it
  as canonical VM state.
- On the proposed fresh-engine path, `KernelMemoryMappings` is rederived per
  owner rather than carried across native-owner closure: old `virtual_pages`
  would otherwise request unmapping pages absent from the new engine. The
  current copy-based families need checked byte synchronization before closing
  the outgoing owner or rotating. Migrated private committed pages instead
  retain one AddressSpace-owned backing, pinned through native mapping; a
  fresh engine remaps that same backing without private-page readback.
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
  E-18 stays red until ordinary private pages have owner-qualified canonical
  backing across fresh owners. The selected host-backing design below removes
  private copy-in/readback rather than adding another copy-back path; public
  COREDLL `VirtualAlloc` and cross-process isolation require separate controls.
- Read-only E-18 production audit: `MemorySystem` allocations/lazy seeds are
  global and base-keyed, mixing private `VirtualAlloc` and file-view backing;
  the native page ledger is VA-only. A copy-back interim would need captured
  owner, backing kind and allocation incarnation, exact mapped-page reads
  before owner finish/rotation, and staged failure handling. Mapping any page can
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

The owner key must be the anchor's birth-qualified `ProcessIdentity`, not the
currently selected PID. There is no existing canonical process-birth record:
`CeKernel` holds a bare selected PID and pending launches, while each public
process handle contains a copied `ProcessObject`. Introduce one durable,
machine-owned process-birth directory inside the HandleTable authority,
independent of public handle lifetime,
for boot PID1 and child birth; mint PID/incarnation with checked exhaustion and
capture the identity at native-attempt start. Current `next_process_id` uses
saturating increment and `set_current_process_id` changes a bare PID; neither
is safe as a backing owner. Stage 3 extends this same directory for Task/object
authority and makes process handles aliases, not a parallel long-lived registry.
Checkpoint clone establishes a new machine lineage and deep-copies private
backing.

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

## O-M1 backing decision: AddressSpace-owned committed-page slabs (2026-10-13)

**Design selected; implementation and acceptance open.** Per anchor A4, each
migrated page has one writable byte authority: stable host backing mapped into
the selected native engine with `mem_map_ptr`. An `AddressSpace` owns private
slabs and its view records; a canonical VM/file-mapping object owns shared
slabs referenced by all corresponding views, including closed-handle live
views. Ordinary private pages do not use per-slice `initial_bytes` copy-in or
copy-back after migration. Unmigrated file views and image/heap blobs retain
their existing checked copy boundary until each family moves in a connected
change; do not silently mix both authorities for one backing.

This choice is supported, not proved in production, by the explicitly selected
ignored `native_mem_map_ptr_boxed_slab_survives_splits_and_fresh_owner`
diagnostic (`native_engine.rs`): one fixed boxed slab took real MIPS stores,
survived protect/unmap splits and was visible to a fresh owner. It passed 1/1
on the E-18 source ([local receipt](../../evidence/om1-host-slab-20261013/focused-diagnostic.receipt.md)).
The vendored Unicorn pointer-map contract requires a writable, stable host
range for the mapping lifetime. The diagnostic did not exercise production
Kernel allocation, concurrent host access, clone, failed native calls or app
memory. E-18 is still red on the later E-19 source ([ledger E-18/E-19](../evidence/ledger.md)).

The local CE6 kernel source at
`C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\vm.c` directly confirms the
design distinction: `DoVMAlloc` adds `MEM_RESERVE` for a null address, scans
an existing reservation for commit-only requests and separately commits pages
(lines 856-909); `VMDecommit` invalidates a page range (1598-1625), while
`VMRelease` releases the reservation (1639-1688). `VMProtect` is a real
permission operation (2916-2940), and `NKVirtualSharedAlloc` uses the shared
heap rather than ordinary private allocation (3687-3705). These are source
semantics, not proof of the dumped application's exact API sequence; capture
that use and its failures before matching CE behavior in the emulator.

### Authority, identities and interface direction

- One new canonical machine-owned HandleTable birth directory mints the anchor's
  `ProcessIdentity` once at boot/child birth: public PID plus checked
  incarnation, scoped to a machine lineage. It lives independently of the
  current copied per-handle `ProcessObject`; a public handle stores only an
  immutable identity/key, never its own mutable process state or lifetime.
  Exit/signaled/debugger/lookup/close resolve one canonical record; active
  execution, VM/view and launch pins keep it alive after the last public
  handle closes. `open_process` resolves a live canonical birth, not an
  existing alias scan. Boot PID1 has no invented public handle. PID selection
  is not ownership.
  `next_process_id` currently saturates and can repeat; exhaustion must fail,
  not alias a prior birth. Stage 3 extends this same directory rather than
  adding another registry.
- Each address space owns its reservation cursor and allocation directory.
  An allocation has checked incarnation, original reserved extent, backing
  kind (`Private` or exact shared-mapping identity), protection and committed
  page set. A reallocation at the same VA gets a different identity. A
  `FileMappingView` gains a distinct view identity; public handle aliases and
  closed handles with live views cannot be resolved by base alone.
- API shims request checked reserve/commit/protect/decommit/release and scoped
  read/write operations. They do not own slabs or mint a PID. For an inactive
  process, publish initialized canonical backing and owner/permission metadata;
  its first execution/access lease maps and checks them before use. For an
  active owner, validate the native map before returning a pointer the guest
  may immediately dereference. The CPU captures the owner at primary-attempt
  start and passes it unchanged through
  map, fault, reclaim and sync, even if a child launch temporarily changes
  `kernel.current_process_id`. `VirtualQueryEx` resolves the requested process
  handle or fails; it cannot silently inspect the selected process.
- The native mapping lease records exact `(owner, allocation incarnation,
  backing identity, page offset, protection)` entries and holds a strong
  backing pin. Acquire a non-moving pin before `mem_map_ptr`, install the
  in-slice ledger entry before guest entry, and keep a local pin after map
  success until checked unmap or owner close even if metadata commit fails.
  Every early return/Drop closes the owner before its final pin drops. Host
  bytes need audited interior mutability and access guards; no live Rust byte
  slice or mutable reference may cross `emu_start`. A VA-only `virtual_pages`
  set or a source `Vec` pointer is not evidence of identity. Only the VM may
  reconcile this ledger; the CPU does not maintain a second virtual allocator.

### Backing, lifetime and failure boundary

- Keep 64-KiB reservation granularity separate from 4-KiB committed pages.
  Allocate fixed, page-multiple host segments only for committed pages; group
  contiguous pages of the same segment/protection where native mapping allows.
  A one-page commit must not allocate a full reservation by default. A segment
  never moves or resizes. `FileMappingObject.data` is currently a resizable
  `Vec<u8>` and cannot be passed directly to `mem_map_ptr` in the later view
  migration.
- Quarantine pointer mapping and host-byte access in one module. Native guest
  writes and Rust mutable references may not alias during `emu_start`; the
  execution lease fences ordinary host access, with only a bounded hook lease
  permitted inside a callback. No unchecked `Send`/`Sync` promise follows
  from a stable pointer. Protect changes use checked native `mem_protect`, not
  a re-seed.
- An active or idle engine pins each mapped segment until checked unmap or
  owner closure. `VirtualFree`, decommit or same-VA reuse cannot drop/rebind
  backing while a native pointer still maps it. On failed unmap, retain pins,
  fail the attempt and close its owner before release; never publish a
  half-unmapped state. Fresh `finish()` discards the engine-specific blank
  Context and ledger after checked CPU capture, while preserving only a
  proven transferable CPU snapshot. The existing manual-close fixture passes
  intermediate Context checks but still fails its final default-policy
  assertion; it is not O-P4 acceptance.
- Prepare and validate backing and metadata before pointer publication;
  validate a native map before active-guest access, or on the inactive owner's
  first native lease. If a pre-run map fails, abandon the candidate without
  changing the old live allocation. A fault-time native map/protect/unmap or
  identity failure becomes `BackendError`, closes the unusable lease and forbids a
  successful continuation from a previously captured CPU snapshot. Guest
  writes already made to a live slab are not promised rollback; record their
  attempt/owner and stop. Do not turn an error into a normal CE return or PC0.
- A checkpoint clone creates a new machine lineage and deep-copies private
  bytes. It remaps birth records, pending/current process selection, VM/view
  and CPU/parked owner keys in one checked machine operation; independent
  derived clones of `CeKernel`/`HandleTable`/`MemorySystem` cannot safely
  relineage a partial checkpoint. A source-machine key is stale in the clone.
  Shared aliases within that clone refer to one clone-local backing; no
  mutable slab is shared with the original. Image/heap and file-view copy
  paths remain explicit until their separate migration.
- At slice completion, stage the CPU snapshot and all outgoing bytes from any
  unmigrated copy-based family, check every sync and owner-close result, and
  publish a successful continuation/VM state only after all checks pass. A
  failed sync poisons/closes that owner and forbids replay from an earlier
  `saved_context`; already-executed guest writes are not promised rollback or
  another full-RAM staging image.

### Connected migration and executable gates

1. Confirm the dumped target's allocation/API sequence against the local CE6
   source distinction above. Introduce the one canonical HandleTable-owned
   process-birth directory, migrate all process-handle mutable state/aliases
   and checked selection to it, then move private allocation
   callers, owner-scoped cursors, incarnation and queries together; no unused
   parallel registry or implicit PID-1 fallback. Inventory locale and file
   views, Toolhelp snapshots, shared allocation and every public
   VirtualAlloc/Free/Protect/Query and Ex caller before defaulting to Private.
   Targeted Ex operations must resolve their process or fail closed; reserve-
   only then commit, partial decommit/recommit and host/native permissions need
   executable controls. One owner-scoped region/overlap directory must cover
   private, image, heap, shared-view and native regions; an explicit VA that
   overlaps another family fails before pointer publication, never silently
   skips an already mapped native page. Add overlap, release/reallocate and
   failed-publication controls before using identity for native mapping.
   Replace independent checkpoint clones with one machine-level relineage;
   prove original keys are rejected in the clone and clone-local aliases stay
   shared with each other but not with the original.
2. Migrate the private `VirtualAlloc` family in one change: stable committed
   backing, exact pinned native page ledger, owner-qualified startup
   reconciliation/map/fault/reclaim, checked failure latch, and removal of
   that family's whole-`initial_bytes` reseed/copy path. No migrated private
   page may reappear as native-owned `mem_map` RAM. Promote the isolated
   pointer-map diagnostic to a normal native control. Turn E-18 green without
   retaining the old engine; add
   public COREDLL two-slice store/load, two-PID same-VA private isolation,
   untouched/two-page writes, free/reallocate, decommit/protect, independent
   clone and injected second-page map/identity failures. E-16 view regressions
   must remain green. These tests, not this design record, prove the boundary.
3. Migrate shared file views with stable mapping backing and exact view
   identities. Exercise public two-process same-VA distinct files, same-file
   aliases, closed-handle live views, flush/unmap, lazy faults and foreign
   private/view collision. Then migrate image/heap blobs without leaving a
   second writable copy for any migrated family.
4. Only after private/shared bytes and exact closed-owner CPU Context pass,
   select O-P4 fresh-per-slice as production default. Repeat focused, full
   release, no-feature and alternate-feature suites, then bounded app runs.
   Keep the preserved BED2 fallback distinct from candidate builds.

Measure reserved versus committed guest bytes, slab capacity, native map
count, setup/sync time, engine count, host RSS/private and guest busy MIPS on
matched workloads. Less copying, smaller host footprint and faster boot are
hypotheses until phase-aligned runs establish them. A checked private
copy-back could be an explicit interim boundary, but it retains duplicate
writable storage, full-scan/write-tracking cost and failure staging; it is not
the selected O-M1 target or Stage 2 completion.

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
