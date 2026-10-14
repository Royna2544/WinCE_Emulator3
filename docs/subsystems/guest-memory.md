# Guest memory and AddressSpace

Status: current byte-boundary behavior is separate from the proposed ownership model.
Authority: [shared architecture](../ARCHITECTURE.md). Sequence: [migration plan](../PLAN.md).

## Current source and facts

- [memory.rs](../../src/ce/memory.rs) holds allocation/virtual metadata; emulator
  [memory.rs](../../src/emulator/memory.rs) defines region layout and permissions.
- [pe_loader.rs](../../src/emulator/pe_loader.rs) creates mapped blobs. Native mapping,
  spillover, shared-view synchronization and byte adapters live in
  [unicorn.rs](../../src/emulator/unicorn.rs); [native_engine.rs](../../src/emulator/native_engine.rs)
  now uses a fresh native owner per slice by default and retains persistent
  reuse only as a Stage 8 opt-in; it is not a replacement memory authority.
- Checked guest-backed raw/native initial/reallocation paths establish bytes before
  publication; metadata-only helpers still exist. Logical zero reserves one physical byte.
- Failed prepared candidates leave monotonic holes. Existing heap free does not prove
  address reuse, per-process ownership, immutable reserved extent or ABA protection.
- Four host-queued typed reservations materialize at delivery. Immediate guest-pointer
  publication must not bypass writable native backing or initialized payload bytes.
- Persisted spillover can mirror a global high-water mark into multiple CPU images;
  live heap metadata, blob capacity, native backing and host RSS are different measures.

### E-22 metadata-only authority slice (2026-10-14)

`HandleTable` now issues an immutable `ProcessBirthKey` from its canonical
birth record; `CeKernel::current_process_birth_key` checks the selected PID
against that directory rather than inventing PID 1. Each successful
`MemorySystem::virtual_alloc` records that key, an allocation incarnation
that is not reused within one `MemorySystem` lineage, and an explicit source
classification. COREDLL distinguishes ordinary private allocations, its
still-compatibility-only Ex and shared shims, file views, locale views, and
Toolhelp snapshot buffers. Four focused release metadata controls passed;
the combined release suite passed 1,647/0/5 ignored and the no-feature suite
passed 1,248/0/2 ignored on the later source ([ledger E-22](../evidence/ledger.md)).

E-22 was identity metadata, **not** AddressSpace ownership or slab backing.
At that point the allocation directory and cursor were global/base-keyed;
native `mem_map`/`initial_bytes` behavior was unchanged, and a derived clone can
reuse a later allocation incarnation under the same copied birth key.
Neither reserve-only native behavior nor E-18 private bytes, E-20 same-VA
private isolation, Ex targeting, shared views, clone lineage, or O-P4
fresh-default lifetime has passed its acceptance gate.

### E-23 owner-qualified private-VA metadata (2026-10-14)

The virtual-allocation directory now keys metadata by process birth and
base. Private allocation lookup, release, query visibility and pointer
probes reject a foreign owner's region; nonprivate legacy producers remain
globally visible for compatibility. Per-owner cursor metadata exists, but a
global floor and overlap fence still forbid live same-VA regions because
native mapping/reclaim is VA-only. Raw COREDLL Ex arms resolve their target,
allow the current process and fail closed for a remote process; these arms
are not published in the default export table. Two focused release controls
passed on source `4a1a9bfb9898a9f2a251212d6120794970266692`
([ledger E-23](../evidence/ledger.md)). The same source passed full release
1,649/0/5 ignored and no-feature 1,250/0/2 ignored; explicitly run ignored
E-18 and E-20 gates still failed at their stale-byte and child-same-VA
assertions. No slab backing or fresh-default owner policy was introduced.

### E-24 file-view mutation guard (2026-10-14)

E-23's green suites missed foreign mutation of legacy file views. E-24 now
requires the exact birth key for every `VirtualFree`/decommit and an owned
`FileViewLegacy` allocation before Unmap. The shared Flush/Unmap commit path
checks the view's process ID and any VA record's birth key before reading
guest bytes or changing mapping data. Child Free/Flush/Unmap rejection and
parent Flush/Unmap success have a public regression; three focused controls
passed. Full release passed 1,650/0/5 ignored and no-feature 1,251/0/2
ignored on source `59f7923b9e3b508705838b8da8f28a5f7aa3b14e`
([ledger E-24](../evidence/ledger.md)). E-18/E-20 explicitly remain red.
Legacy nonprivate read/query visibility remains, and a same-PID DIB view
without VA metadata may Flush but is conservatively refused by Unmap. Native
VA-only backing and the fresh-owner byte gap are unchanged on E-24 source.

### E-25 internal private-slab diagnostic (2026-10-14, bounded validation)

The internal `cfg(test)` slab opt-in now has a source-bound focused release
receipt: an unignored real-MIPS control stored `0x11223345` in canonical
private backing before native-owner closure, and a fresh owner's next MIPS
load read the same value. Two mapper rejection controls, one private-slab/file-
view overlap control, four existing native-memory controls and one retained-
pin control also passed ([E-25 focused receipt](../../evidence/e25-private-slab-20261014/README.md),
[ledger E-25](../evidence/ledger.md)). This is not the historical unchanged
E-18 fixture: its earlier red used legacy internal `virtual_alloc`, whereas
E-25 opted into a test-only slab; at that checkpoint public `VirtualAlloc`
still selected `LegacySeed`. E-35 later validated the public private slab
family ([E-35 receipt](../../evidence/e35-public-virtualalloc-validation-20261014/validation.receipt.md)).
The first full release run caught a synthetic-PID compatibility
regression (726/1/3 ignored); after its narrow birth-precondition fix, the
corrected source passed full release **1,664/0/4 ignored**, no-feature
**1,258/0/2 ignored**, a no-trace Unicorn+desktop release check and final
format check ([separate broad receipt](../../evidence/e25-private-slab-20261014/broad-validation.receipt.md)).
Real native two-process A→B→A same-VA bytes, shared-view coherence and full
machine checkpoint relineage remain open. E-25 by itself established no
O-M1/O-P4, Stage 2 or live-app acceptance; E-35 later closed the bounded
public private-family gate, and O-P4 later passed its pinned-source fresh-owner
teardown code/test gate
([receipt](../../evidence/op4-fresh-default-20261014/validation.receipt.md)).

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
- On the now-default fresh-engine path, `KernelMemoryMappings` is rederived per
  owner rather than carried across native-owner closure: old `virtual_pages`
  would otherwise request unmapping pages absent from the new engine. The
  remaining copy-based families need checked byte synchronization before closing
  the outgoing owner or rotating. Migrated private committed pages instead
  retain one owner-qualified backing, pinned through native mapping; a fresh
  engine remaps that same backing without private-page readback.
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
  The ignored two-slice MIPS control demonstrated that loss on the historical
  E-18 source for an internal `MemorySystem::virtual_alloc` page: the first
  owner's native RAM
  held guest-written `0x11223345`, but after `cfg(test)` idle-owner closure the
  fresh owner's load returned the unchanged `initial_bytes` seed
  `0x11223344`. Both slice PCs, normal stops and no-fault checks passed.
  This is an expected-red private-byte boundary, not a Context failure or a
  production fresh-default change ([ledger E-18](../evidence/ledger.md),
  local `evidence/stage2-o-m1-private-fresh-20261013/expected-red.receipt.md`).
  E-18 remained red on that legacy source; E-25's test-only opt-in made the
  diagnostic shape green, not public ordinary private pages. E-35 later
  validated public COREDLL private `VirtualAlloc` on stable backing, while
  real native two-process same-VA byte isolation remains a separate control.
  The selected host-backing design below removes private copy-in/readback
  rather than adding another copy-back path.
- Historical read-only E-18 source audit: `MemorySystem` allocations/lazy seeds were
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

## O-M1 same-VA file-view design gate (still open)

At the pre-E35 baseline the kernel had one virtual-address cursor and a
base-keyed allocation map. E35 moved the public private-VM family to
owner-qualified slab backing, but it did not establish public two-process
same-VA `MapViewOfFile`. `FileMappingView` records a PID but not a durable view
identity; handle-table lookup, flush and unmap use
base alone (`src/ce/object.rs:175-194,690-744`). E-16 captures the selected
process owner and exact live view for native seed/readback only; broader
native mapping, fault repair, public view allocation and lifetime still lack
an authoritative `AddressSpace` identity. A filter on one sync helper is not
a complete O-M1 fix.

The owner key must be the anchor's birth-qualified `ProcessIdentity`, not the
currently selected PID. At the E-20 baseline, `CeKernel` held a bare selected
PID and pending launches while each public process handle copied its mutable
`ProcessObject`. E-21 introduced one HandleTable-owned birth record for boot
PID1 and each child, independent of public alias lifetime, with a checked
incarnation and shared public-handle state. The next step is to capture that
identity at native-attempt start and bind VM owners to it. `next_process_id`
now uses checked arithmetic but still panics instead of returning a
guest-visible exhaustion failure; `set_current_process_id` still changes a
bare PID. Neither is a complete backing-owner contract. Stage 3 extends this
same directory for Task/object authority; checkpoint clone must establish a
new machine lineage and deep-copy private backing.

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
A fresh-owner two-slice guest-written private `VirtualAlloc` round-trip was an
executed expected-red gate on the historical E-18/E-19 sources. E-25 passed
the changed internal slab-opt-in shape; E35 subsequently validated the public
private family on stable backing. Neither establishes shared-view coherence
or native A→B→A isolation.

## O-M1 backing decision: AddressSpace-owned committed-page slabs (2026-10-13)

**Design selected; E35 validates the public private family, and O-P4 validates
the pinned-source fresh-owner code/test gate; shared views and Stage 2
acceptance remain open.** Per anchor A4, each
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
memory. E-18 was still red on the historical E-19 source
([ledger E-18/E-19](../evidence/ledger.md)); the changed E-25 internal opt-in
has the focused and corrected-source broad results above, not public backing
acceptance.

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

- E-21 now gives HandleTable a canonical birth directory for boot PID1 and
  child births: public PID plus checked incarnation, independent of public
  process handles. An alias stores only an immutable birth key; public exit,
  signaled, debugger, lookup and close resolve the canonical record. The
  execution pin survives last-handle close until actual completion; boot PID1
  has no invented public handle. This is not yet the anchor's full
  machine-lineage `ProcessIdentity`: VM/view/wait pins, selected-process
  compatibility state, clone relineage and guest-visible PID exhaustion still
  need migration. PID selection is not ownership. Stage 3 extends this same
  directory rather than adding another registry.
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
  Every fresh early return closes its owner before final pins drop under the
  pinned vendored 2.1.5 source proof; Drop is only a fail-safe and cannot
  publish success. If a future source/linkage change makes closure uncertain,
  retain/quarantine the pin and latch a terminal backend error instead of
  freeing host memory. Host
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
  proven transferable CPU snapshot. O-P4's ordinary closed-owner Context
  control and 200-slice mapped-pin churn passed on frozen source
  ([receipt](../../evidence/op4-fresh-default-20261014/validation.receipt.md));
  this is not shared-view or live-app acceptance.
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
  unmigrated copy-based family, check every sync and the source-bound owner
  teardown preconditions, and publish a successful continuation/VM state only
  after all checks pass. A
  failed sync poisons/closes that owner and forbids replay from an earlier
  `saved_context`; already-executed guest writes are not promised rollback or
  another full-RAM staging image.

### O-P4 source-bound fresh native owner (code/test gate passed)

The earlier general checked-close plan was deliberately narrowed after
auditing the exact statically linked vendored Unicorn 2.1.5 C source. Its
`uc_close` has only `UC_ERR_OK` exits; the high-level Rust wrapper still
discards the return from `uc_close`, so this is **not** a generic checked-close
API. The root manifest pins the wrapper to `=2.1.5`; the vendored sys crate
always builds the in-repo C and rejects `dynamic_linkage`. Any source, wrapper
or linkage change requires re-auditing the close/hook proof before relying on
it ([vendor patch notes](../../vendor/unicorn-engine-sys/PATCHES.md)).

For a successful default fresh slice, the lease captures the exact-key
CPU-only Context and checks mapping/byte synchronization before publication.
On finish, abort or unfinished Drop, it closes the **owning** wrapper while
the nonowning run wrapper, hook callback boxes and backing pins still live;
only then does it release callbacks
and pins. It does not call individual `remove_hook` first because the Rust
wrapper drops the callback box before the C hook-deletion call. The engine's
blank Context and map ledger never carry to the next fresh lease. A separate
explicit `WINCE_EMU_PERSISTENT_NATIVE_ENGINE=1` path preserves the Stage 8
experiment; reusable-hook teardown is not accepted by this fresh-only proof.

The formerly ignored closed-owner Context/store control, 200 fresh MIPS slices
with pointer-backed pages and checked unmap/owner closure, finish-failure pin
ordering and broad release/feature suites passed on frozen source
([O-P4 receipt](../../evidence/op4-fresh-default-20261014/validation.receipt.md)).
The receipt's failure seam is before publication, not evidence of a failing
vendored-C `uc_close`. Public private backing is separately covered by E35.
No live iNavi run, host-footprint comparison, native A→B→A isolation,
shared-view coherence or full Stage 2 acceptance follows from this gate.

### Connected migration and executable gates

The optional E-25 internal slab milestone, E35 public private-family gate and
O-P4 source-bound fresh-default code/test gate are executed, not still-proposed
steps ([E35 receipt](../../evidence/e35-public-virtualalloc-validation-20261014/validation.receipt.md),
[O-P4 receipt](../../evidence/op4-fresh-default-20261014/validation.receipt.md)).
They do not authorize shared-view, native two-process byte or live acceptance.
The remaining connected gates are:

1. Complete the Stage 2/3 identity boundary around the E-21 process-birth
   directory: selected-state, full machine checkpoint relineage and
   guest-visible exhaustion handling. Keep private, image, heap, shared-view
   and native overlap checks in one owner-qualified region authority; do not
   infer ownership from selected PID or a bare VA. Inventory locale/file
   views, Toolhelp snapshots, shared allocation and Ex export/targeting gaps.
   A clone must reject source-machine keys and keep its own private bytes;
   cloned shared aliases must agree with each other, not the original.
2. Extend E35's passing private-family controls to real native two-process
   A→B→A same-VA byte isolation, stale identity and second-page failure cases.
   Preserve public reserve/commit/decommit/protect/query, fresh-owner store,
   independent private-clone, pin-budget and 64-cycle checked-unmap controls;
   no later change may reintroduce a second writable authority. E-16 native
   view regressions must remain green.
3. Migrate shared file views with stable mapping backing and exact view
   identities. Exercise public two-process same-VA distinct files, same-file
   aliases, closed-handle live views, flush/unmap, lazy faults and foreign
   private/view collision. Then migrate image/heap blobs without leaving a
   second writable copy for any migrated family.
4. Treat O-P4's fresh-default code/test result as complete only for its
   pinned-source boundary; persistent reuse stays an explicit Stage 8 opt-in.
   A bounded read/write iNavi run requires **fresh human approval** and fresh
   adjacent ZIP backups of the original SD, ResidentFlash and emulator-root
   mount folders. Compare phase-matched host memory peak, busy-slice MIPS and
   fixed per-slice setup cost with BED2 E-1 and persistent E-22/E-27/E-29;
   no map readiness or speed/footprint gain is established by the suite.
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
