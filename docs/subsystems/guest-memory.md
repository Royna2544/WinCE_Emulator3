# Guest memory and AddressSpace

Status: current byte-boundary behavior is separate from the proposed ownership model.
Authority: [shared architecture](../ARCHITECTURE.md). Sequence: [migration plan](../PLAN.md).

## Current source and facts

- [memory.rs](../../src/ce/memory.rs) holds allocation/virtual metadata; emulator
  [memory.rs](../../src/emulator/memory.rs) defines region layout and permissions.
- [pe_loader.rs](../../src/emulator/pe_loader.rs) creates mapped blobs. Native mapping,
  fixed heap/stack slabs, shared-view synchronization and byte
  adapters live in
  [unicorn.rs](../../src/emulator/unicorn.rs); [native_engine.rs](../../src/emulator/native_engine.rs)
  now uses a fresh native owner per slice by default and retains persistent
  reuse only as a Stage 8 opt-in; it is not a replacement memory authority.
- Checked guest-backed raw/native initial/reallocation paths establish bytes before
  publication; metadata-only helpers still exist. Logical zero reserves one physical byte.
- Failed prepared candidates leave birth-local monotonic holes. O-M2 keys heap
  high-water, handles and allocation metadata by process birth, rejecting
  foreign `HeapFree`/`HeapSize`; heap free still does not establish address
  reuse, immutable reservation extent or ABA protection.
- Four host-queued typed reservations materialize at delivery under the
  recipient birth. Retrieval alone does not retire a payload before dispatch;
  immediate guest-pointer publication must not bypass writable native backing
  or initialized bytes.
- O-M2 removed copied heap spillover. Live heap metadata, committed slab pages,
  native backing and host RSS remain different measures.

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
At E-25, real native two-process A→B→A same-VA bytes, shared-view coherence
and full machine checkpoint relineage remained open. E-25 by itself established no
O-M1/O-P4, Stage 2 or live-app acceptance; E-35 later closed the bounded
public private-family gate, and O-P4 later passed its pinned-source fresh-owner
teardown code/test gate
([receipt](../../evidence/op4-fresh-default-20261014/validation.receipt.md)).

### D008 public shared-view source gate (2026-10-14)

E-39 separately passed real native public-private and injected-file-view
A→B→A same-VA bytes. D008 now routes public `MapViewOfFile` through
birth-qualified view allocations and mapping-offset keyed stable shared
pages. A named `CreateFileMappingW` reopen in A/B maps the same canonical
page at the same guest VA across fresh native owners; A→B→A stores/loads and
an unflushed in-slice alias store/load passed the focused real-MIPS control.
This manually selects the queued child state, not guest `CreateProcessW` or
Scheduler handoff. The final frozen-source broad release/no-feature gates
passed 1,744/0/14 ignored and 1,297/0/13 ignored, respectively, each across
17 result blocks ([D008 receipt](../../evidence/d008-shared-view-20261014/validation.receipt.md)).
This is isolation at a shared VA selected by the emulator's per-owner cursor,
not CE same-VA placement semantics. No D008 iNavi run, footprint or speed
result exists.

`FileMappingObject` now has mutually exclusive `MappingBacking::Legacy` and
`Slab` variants. The first public view preflights and migrates the **entire**
legacy byte length into canonical 4-KiB pages, including bytes outside a
small or disjoint first view, and then drops the Vec. Every new page charges
the same physical-page budget as private slabs; insufficient budget returns
`ERROR_NOT_ENOUGH_MEMORY` without retiring Legacy or publishing VM metadata.
Live legacy DIB and public views cannot coexist; a later DIB request after
migration fails. File-backed pages are loaded lazily from the file, with a
whole-page file underlay beneath preexisting DIB bytes when migrating. The
focused DIB migration, disjoint-first-view, named child reopen, file-tail
seed/flush and low-budget controls pass.

The raw test adapter still owns copied guest bytes, unlike the native pointer
aliases. A raw Flush/Unmap commits the requested range into the canonical
page before writing overlapping aliases of the same process birth. If an
alias write fails, the API fails but canonical and earlier guest bytes may
already have changed; there is no atomic rollback. At D-008, image/heap and
older legacy-view backing, full machine checkpoint relineage, other native
failure edges and live map readiness remained separate gates.

Sparse migration was dropped: Microsoft's [CE `CreateFileMapping` reference](https://learn.microsoft.com/en-us/previous-versions/windows/embedded/ms923951%28v%3Dmsdn.10%29)
documents default `SEC_COMMIT` for the whole section. At D-008, the next
Stage 2 backing slice was immutable PE image pages, not a demand-zero file
view; E-43 later source-gated the final-byte sealed main-image family.

### E-44 fixed initial heap/stack slab source gate

The source reserves the initial 16-MiB `ProcessHeapArena` and
4-MiB `ThreadStack` as owner-qualified fixed ranges in `virtual_allocations`.
The production native mapper no longer creates `ram:ce-heap-arena` and
`ram:guest-stack` blobs. Heap pages commit at allocation; stack pages commit
lazily on a native fault. Native backing pins persist for the checked
owner/unmap lifetime, so logical decommit alone cannot release their budget.
Queued payload delivery uses the prepared heap path and rejects other fixed-
slab overlap instead of writing an unrelated persisted blob.

On the formatted six-file source snapshot, the serial release
`unicorn,trace,win32-desktop` all-target suite passed **1,785/0/17 ignored**
across 17 result blocks, paired debug no-feature passed **1,321/0/13
ignored** across 17, and the debug no-trace Unicorn/desktop check passed.
This is a **source-bound** gate, not live app acceptance. The audited startup
exact-output families (registry, time, geometry, find-data and module/path)
route through guarded hooks with whole-span preflight. Unconverted direct-
import outputs can still
fail closed on a cold, untouched stack page; the shared sparse `ThreadStack`
has no per-thread/slot low-water state. Only if an approved live run reaches
that bypass should a per-thread/slot 128-KiB committed low-water be considered,
tracked by slot and committing `[page(SP), slot_top)` contiguously. At E-44,
the global heap cursor and ownerless allocation metadata still needed
birth-qualified authority, and heap spillover remained copy-backed. Writable
image sections and DLLs were separate later slices. No E-44 iNavi, host RSS,
speed or map-readiness result exists.

Additional bounded controls passed under that source gate. A release synthetic
56-MiB sealed-RX setup profile passed 1/1 with 77,824 B owned blob capacity,
fresh blob writes and fresh/reused readback, compared with 21,049,344 B for
those fields in the E-43 pre-slab debug control: 20,971,520 B (20 MiB) fewer
logical blob bytes. Slab reservation lies outside the timed setup section;
cross-mode single samples cannot support a speed or RSS claim.
An exact-path blocked serial/NMEA control passed 1/1 after writing two
untouched stack pages and surviving a fresh native owner. It does not
independently prove stale different-birth rejection. A same-mode post-slab
debug profile also passed 1/1 with the same 77,824-B fields; its fresh/reused
setup samples were 16.740/25.249 ms versus old debug 33.246/37.971 ms, but
one sample each and out-of-timer slab reservation prohibit a speed claim
([E-44 receipt](../../evidence/heap-stack-20261014/validation.receipt.md)).

### O-M2 per-birth heap source gate (2026-10-14, executed)

Each `ProcessBirthKey` now owns its heap cursor, handles, `(birth, VA)` blocks,
prepared candidates and typed queued payloads. The fixed process-heap window
is `[0x30000000, 0x40000000)`: a 256-MiB sparse reservation below the
`0x50000000` `VirtualAlloc` floor. Only crossed 4-KiB pages commit; the global
copied heap-spillover path is removed. E-38 iNavi had reached 31,546,846 B
live heap (`evidence/e37-live-20261014T051228Z/app.stdout.raw.txt:13132`),
already beyond the E-44 16-MiB initial arena.

Raw Heap/Local/CRT and pointer probes use the selected process birth; host
message payloads capture the recipient window's birth. Known typed payloads
remain owner-qualified across Get/Peek retrieval until dispatch or exact
window/process teardown. B may use its own block at the same numeric VA, but
cannot query or mutate A's block; a VA allocated only in A is rejected in B.
Invalid `HeapFree`/`HeapSize` use
`ERROR_INVALID_PARAMETER`, matching the local CE6 `heap.c`/`rheap.cpp`
reference. Preflight validates the output destination before heap growth and
pointer publication. Growth past the top returns NULL with
`ERROR_NOT_ENOUGH_MEMORY` without a page charge; failed prepared allocations
may retain birth-local monotonic holes. A stale serial/NMEA completion under
another birth is fenced before RX consumption or stack writes.

The `ORD_REMOTE_HEAP_*` dispatches still skip their target-process argument
and delegate under the selected birth (`coredll.rs:5881-5911`); remote-target
heap semantics are not covered by this gate. `GetClipboardDataAlloc` reads a
globally stored clipboard handle but sizes it under the current birth
(`coredll.rs:20301-20304`), so cross-birth clipboard transfer is likewise
not established.

The serial release `unicorn,trace,win32-desktop` all-target suite, paired debug
no-feature all-target suite, and Unicorn/desktop no-trace check passed. Exact
focused controls, counts, hashes and logs are in the
[O-M2 receipt](../../evidence/om2-heap-20261014/validation.receipt.md).
This is a source gate: no O-M2 live iNavi run, host-RSS/speed gain, map
readiness, general heap address reuse or full Stage 2 acceptance follows.

### 1-MiB host-block native mapper (bounded source validation)

The opt-in 8,192-page/32-MiB fresh-owner profile measured 78.690 s mapping:
78.524 s in 8,192 Unicorn `mem_map_ptr` calls versus 0.164 s in Rust setup
and scans ([receipt](../../evidence/om2-native-map-profile-20261014/validation.receipt.md)).
An expected-red repeat instrumented that tail: native owner close took 75.753 s,
while pin/guard drop took 0.636 ms. The release before-change assertion saw
8,192 calls and failed as intended
([receipt](../../evidence/om2-native-map-profile-20261014/validation.receipt.md)).
The E-43 pre-slab copy setup used a different workload and debug profile,
so neither number is a controlled app-speed or RSS comparison.

The implementation reserves **1-MiB-sized host blocks** keyed by process
birth, allocation incarnation and 1-MiB-aligned guest block base, then
physically commits only guest-committed 4-KiB private **heap/stack** pages.
Host block bases need only Unicorn's 4-KiB alignment. Shared-view pins from
`stable_page_pins_for` remain independently backed; they are not rehomed or
grouped. A weak, non-owning pending-block fence prevents competing detached
first-cold-page proposals from obtaining two byte authorities. Canonical
`PrivatePageIdentity`, page-level budget charge, native pin and execution
guard remain exact; reserved holes have no guest-RAM charge. A clone reserves
new blocks and copies committed pages without sharing writable pointers.

Both fresh-owner and prepared in-slice heap/stack growth map maximal
contiguous, uniformly protected committed runs within one block through one
`mem_map_ptr` each. A later collision maps the valid prefix before returning
the ordered error. Every page is pinned before FFI; an uncertain map retains
the run's pins until checked native owner close. Vendored
`uc.c:1572-1715,1842-1911` may unmap an entire pointer run to split one
page; checked unmap now requires complete pre-split ledger and native-pin
coverage and verifies all surviving pages after FFI before releasing the
requested page pin. Failed or ambiguous unmap leaves the entire affected run
pinned/quarantined. VM remains the sole byte authority under A4; full machine
checkpoint relineage stays separate.

On one interim synthetic **release** source, the 8,192-page/32-MiB uniform
heap mapped as exactly **32 native regions/calls**: fresh-owner wall 4.427 ms
and native close 0.398 ms; prepared growth wall 4.741 ms and close 0.394 ms.
The same binary's ideal old-shape control (one 32-MiB pointer map plus a
32-MiB memcpy seed) took 3.133 ms map+copy, so batched canonical pages were
not faster than that copy control in this one sample. Exact 8,192 page
records/pins and 32-MiB guest-budget charge passed, along with sparse hole,
clone, first-cold competing proposal, mid-run collision, same-VA birth and
checked-unmap controls. The release timing preceded the final peer-safety
follow-up; paired broad gates on a fixed combined source remain pending.
Neither these synthetic measurements nor the E-43 debug copied-image receipt
establishes iNavi lag, host RSS, frame readiness or live-run acceptance.

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
  validated public COREDLL private `VirtualAlloc` on stable backing; E-39 later
  executed the real native two-process same-VA byte controls.
  The selected host-backing design below removes private copy-in/readback
  rather than adding another copy-back path.
- Historical read-only E-18 source audit: `MemorySystem` allocations/lazy seeds were
  global and base-keyed, mixing private `VirtualAlloc` and file-view backing;
  the native page ledger was VA-only. A copy-back interim would need captured
  owner, backing kind and allocation incarnation, exact mapped-page reads
  before owner finish/rotation, and staged failure handling. Mapping any page can
  at that point reseed the entire `initial_bytes` range, risking overwrite
  of a guest-written page; free/reallocation at the same VA could retain old
  native bytes. `self.saved_context` was published before byte synchronization,
  so context/byte commit ordering needed a transaction. These were historical
  source-audit hazards and design requirements, not claims about the D008 path.
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
  ([ledger E-16](../evidence/ledger.md)). At that E-16 checkpoint this proved
  injected native-view ownership only; public same-VA `MapViewOfFile`, global
  lazy-page scoping, native lazy fault repair, private `VirtualAlloc` persistence
  across fresh-owner closure, and app readiness were open. E35/E39/D008
  addressed only their separately measured subsets. The immediate CreateProcess
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

## O-M1 same-VA file-view design gate (partly executed)

At the pre-E35 baseline, public `MapViewOfFile` used a global cursor and a
copy-backed `FileViewLegacy`. E-16 proved native seed/readback for injected
views, E35 moved public private VM to owner-qualified pages, and E-39 proved
native public-private and injected-view A→B→A same-VA bytes. D008 now gives
public file views a per-birth VA cursor, exact `(birth, base)` view binding,
and one canonical page per `(mapping object, file offset)`. Fresh native
owners pointer-map those pages; the focused named-reopen A→B→A and same-slice
alias tests and frozen-source broad gates pass. This is a connected public
view change, not a completed AddressSpace, Task or machine-checkpoint
migration.

The public fixture uses named `CreateFileMappingW` reopen because the current
`DuplicateHandle` mapping path clones an object by value. It manually selects
a queued child, so it does not validate guest `CreateProcessW`/scheduler
handoff. The view record now carries `ProcessBirthKey`, but not a distinct
checked view incarnation; stale/reused view references and machine-lineage
clone behavior retain separate acceptance. `VirtualQueryEx` export/remote-
owner behavior and PID exhaustion also remain outside this focused gate.

Preserve the E-14/E-16 injected-view and E-39 private controls while proving
public **distinct** mappings at one VA cannot cross-contaminate, native lazy
fault repair and outgoing readback remain checked, closed-handle live views
retain their canonical page, and a foreign public view never overwrites
private RAM at the same VA. A raw adapter's copied aliases are a separate
boundary: failed propagation may leave canonical and prior guest-byte
effects, which must be reported rather than rolled back by assertion. These
requirements, full clone relineage, and remaining writable image/DLL
migration keep O-M1 and Stage 2 open after the D008, E-44 and O-M2 source
gates.

## O-M1 backing decision: AddressSpace-owned committed-page slabs (2026-10-13)

**Design selected; E35 validates the public private family, O-P4 validates
the pinned-source fresh-owner code/test gate, E-39 validates native private/
injected-view A→B→A bytes, and D008 public shared views passed focused and
frozen-source broad gates. Stage 2 acceptance remains open.** Per anchor A4, each
migrated page has one writable byte authority: stable host backing mapped into
the selected native engine with `mem_map_ptr`. An `AddressSpace` owns private
slabs and its view records; a canonical VM/file-mapping object owns shared
slabs referenced by all corresponding views, including closed-handle live
views. Ordinary private pages do not use per-slice `initial_bytes` copy-in or
copy-back after migration. Public views now use stable shared pages. At this
O-M1 decision, legacy DIB/injected views and image/heap blobs retained copy
boundaries; E-43 later sealed the final-byte main-image R/RX family, E-44
source-gated initial heap/stack slabs, and O-M2 removed copied heap
spillover. Do not silently mix both authorities for one backing.

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
  never moves or resizes. The old `FileMappingObject.data: Vec<u8>` could not
  serve as native pointer backing. D008 replaces that dual-authority risk
  with `MappingBacking::{Legacy(Vec<u8>), Slab}`: after complete byte migration
  and preflight, the Legacy Vec is dropped and later legacy writes fail closed.
  A live legacy DIB view blocks that transition.
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
  mutable slab is shared with the original. Remaining writable-image, DLL
  and legacy file-view copy paths remain explicit until their
  separate migration.
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
O-P4 alone established no live iNavi result, host-footprint comparison,
native A→B→A isolation, shared-view coherence or full Stage 2 acceptance;
later E-38/E-39 and D008 source controls address those questions separately
and do not retroactively broaden the O-P4 gate.

### Connected migration and executable gates

The optional E-25 internal slab milestone, E35 public private-family gate and
O-P4 source-bound fresh-default code/test gate are executed, not still-proposed
steps ([E35 receipt](../../evidence/e35-public-virtualalloc-validation-20261014/validation.receipt.md),
[O-P4 receipt](../../evidence/op4-fresh-default-20261014/validation.receipt.md)).
E-39 adds real native public-private/injected-view two-process bytes; D008's
focused controls and final frozen-source broad suites add public same-mapping
shared-page coherence ([receipt](../../evidence/d008-shared-view-20261014/validation.receipt.md)).
A D008 live-app run has not occurred. The remaining connected gates are:

1. Complete the Stage 2/3 identity boundary around the E-21 process-birth
   directory: selected-state, full machine checkpoint relineage and
   guest-visible exhaustion handling. Keep private, image, heap, shared-view
   and native overlap checks in one owner-qualified region authority; do not
   infer ownership from selected PID or a bare VA. Inventory locale/file
   views, Toolhelp snapshots, shared allocation and Ex export/targeting gaps.
   A clone must reject source-machine keys and keep its own private bytes;
   cloned shared aliases must agree with each other, not the original.
2. Retain E-39's passing real native two-process A→B→A same-VA byte isolation
   and extend it to stale identity and second-page failure cases.
   Preserve public reserve/commit/decommit/protect/query, fresh-owner store,
   independent private-clone, pin-budget and 64-cycle checked-unmap controls;
   no later change may reintroduce a second writable authority. E-16 native
   view regressions must remain green.
3. Extend D008's source-gated public shared-view family with public two-process
   same-VA **distinct** mappings, closed-handle live views,
   checked lazy faults, raw-adapter partial-effect controls and foreign
   private/view collision. The focused same-file named-reopen, same-slice alias,
   DIB transition and low-budget controls must stay green. E-43 sealed the
   final-byte main-image R/RX family; E-44 source-gated the initial heap/stack
   slabs; O-M2 then birth-qualified the fixed heap and removed copied
   spillover. Migrate the remaining writable image and DLL families without
   leaving a second writable copy for any migrated family. Safe heap address
   reuse, full clone lineage and Stage 2 acceptance remain separate gates.
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
- Add same-VA private isolation, shared visibility, sparse owner-local heap
  growth and external parked edits; a two-slice guest-written private
  `VirtualAlloc` byte round-trip; stale
  token/frame rejection; partial protect/unmap/remap; clone/checkpoint tests.
- Exercise native reads after publication and fresh/reused engine reconciliation, not
  metadata-only assertions. Long lifecycle controls must include post-owner-close sampling.
- AddressSpace/lifecycle authority precedes any heap reuse. Measured memory improvement,
  FlatView safety and ownership correctness are independent gates, not Phase 1 claims.
