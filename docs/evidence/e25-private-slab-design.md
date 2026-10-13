# E25 design record: private committed pages on process-owned native maps

Status: **plan for independent review; no implementation or acceptance is claimed.**
Date: 2026-10-14. Scope: Stage 2 O-M1, not scheduler policy or Stage 8 native-engine retention.
The normative contract is [Architecture A2/A4/A6](../ARCHITECTURE.md), sequenced by
[PLAN Stage 2](../PLAN.md) and [D-008](../DECISIONS.md). This record refines those
contracts without changing them. A code review must settle the specific API and
failure transitions below before source changes begin.

## Question and boundary

E-18 is an executed expected red: the first real MIPS slice writes
`0x11223345` to a private `VirtualAlloc` page, but a new native owner reloads
the stale seed `0x11223344` ([ledger E-18](ledger.md),
[`unicorn.rs:54850-54965`](../../src/emulator/unicorn.rs)). The current owner
retains native RAM; `MemorySystem` retains only `initial_bytes`. E-23/E-24 made
metadata and mutating APIs birth-aware without changing that byte authority
([ledger E-23/E-24](ledger.md)). E-20's public child same-VA allocation remains
an independent expected red ([`coredll.rs:70004+`](../../src/ce/coredll.rs)).

The first connected implementation is an **internal, opt-in E-18 diagnostic**:
one committed private allocation receives process-owned, byte-coherent host
slabs mapped into the running native owner. That allocation must replace its
private copy-seed/native-RAM behavior rather than add copy-back beside it.
Public `VirtualAlloc` remains on its explicit legacy seed backing until the
public prepare/map/commit and change transactions below are implemented. The
diagnostic may be accepted while the global VA fence and production
persistent-owner policy remain.
Public two-process same-VA, shared views, checkpoint clone and fresh-default
O-P4 are separate gates; a green E-18 alone must not close Stage 2 or imply
app/map readiness.

## Current consumer inventory (reviewed source lines)

| Authority/path | Current behavior and E25 implication |
| --- | --- |
| [`object.rs:239-253,686-755`](../../src/ce/object.rs) | `HandleTable` mints `ProcessBirthKey` and retains a birth after alias closure. Key is PID plus incarnation, **not machine/checkpoint lineage**. E25 must validate the key, not mint another process registry. |
| [`kernel.rs:2823-2835`](../../src/ce/kernel.rs) | Selection is still a mutable bare PID. `current_process_birth_key()` resolves it through the canonical directory. Capture the key once at native-attempt start; later child-launch selection must not change that attempt's VM owner. |
| [`memory.rs:29-44,133-161,486-636`](../../src/ce/memory.rs) | Global `next_virtual_ptr` floor and overlap fence coexist with `(birth,base)` metadata and per-owner cursor. `VirtualAllocation` stores a size, source, protection and copy seed, but no committed-page set or stable private bytes. `virtual_free` removes release metadata immediately; decommit is validation-only. `VirtualAllocationSource::Private` labels both E-18 and public calls, so it cannot select diagnostic slab backing by itself. This is the VM owner to migrate, not CPU-local state. |
| [`coredll.rs:5831-5953,29508-29644`](../../src/ce/coredll.rs) | Public `VirtualAlloc/Free/Query`, current-only Ex compatibility, Shared compatibility and no-op Protect consume this metadata. `virtual_alloc_raw` publishes before native mapping. Changing backing must introduce a checked pointer-publication boundary for public calls; do not silently treat Shared/Ex/file views as ordinary private. |
| [`coredll.rs:29864-30060`](../../src/ce/coredll.rs) | File views use `FileViewLegacy`, resizable mapping data, `initial_bytes`, VA/PID lookups and guest-byte sync. E24 guards foreign mutation, but this family remains an explicit copy boundary. It must not seed a foreign private page when same-VA is later allowed. |
| [`unicorn.rs:6214-6238,11068-11215`](../../src/emulator/unicorn.rs) | `UnicornMips` leases a process native engine, captures only PID, reclaims/reconciles VA-only state and copies persisted blobs. Private slabs must be excluded from blob copy and native-owned `mem_map` reconciliation. |
| [`unicorn.rs:12768-13011,13687-13760`](../../src/emulator/unicorn.rs) | `KernelMemoryMappings` has `HashSet<u32>` virtual pages; initial map and fault repair use `mem_map` plus seed bytes. `map_guest_range` skips a mapped VA without proving backing identity. E25 needs an exact `(owner,allocation incarnation,page/backing,protection)` pin ledger for private pages. Keep the legacy set only for unmigrated paths. |
| [`unicorn.rs:9501-9535,9921-9948,10324-10335,10596-10612`](../../src/emulator/unicorn.rs) | The import hook drops map errors and merely stops; the fault hook converts repair errors to `false`; `saved_context` is assigned before outgoing blob/view sync. E25 must latch a typed backend failure and not publish a successful continuation after mapping/sync failure. |
| [`native_engine.rs:23-143,194-235,311-398`](../../src/emulator/native_engine.rs) | The lease owns the native handle and optional mapping metadata; `finish` retains both on success, while unfinished/drop closes the owner. Exact page pins must follow the native handle through all paths. `ProcessNativeEngine::Clone` deliberately omits the live engine/ledger but copies an immutable CPU snapshot. |
| [`pe_loader.rs:24+`](../../src/emulator/pe_loader.rs), [`unicorn.rs:13239-13258`](../../src/emulator/unicorn.rs) | Image/heap blobs remain a distinct copy-backed family. E25 cannot make them a second writable truth for a migrated private page. |

### Primary evidence and limits

- Local CE6 source `C:\WINCE600\PRIVATE\WINCEOS\COREOS\NK\KERNEL\vm.c`:
  `DoVMAlloc` reserves null-address requests, scans an existing reservation
  for commit-only requests and commits pages separately (856-909);
  `VMDecommit` invalidates a page range (1598-1625), `VMRelease` releases
  its reservation (1639-1688), and `VMProtect` changes page permission
  (2916-2940). These are CE6 kernel semantics, not proof of iNavi's exact
  call pattern. `NKVirtualShareAlloc` uses the shared heap (3687-3705), so
  Shared compatibility is not private memory by declaration.
- The vendored native header
  [`unicorn.h:1143-1167`](../../vendor/unicorn-engine-sys/include/unicorn/unicorn.h)
  requires 4-KiB guest alignment/multiple and a writable host range of at
  least the mapped size for `uc_mem_map_ptr`; otherwise behavior is undefined.
  Unmap/protect are checked calls, and native regions may split after unmap
  ([`unicorn.h:1197-1248`](../../vendor/unicorn-engine-sys/include/unicorn/unicorn.h)).
- The isolated [`native_mem_map_ptr_boxed_slab_survives_splits_and_fresh_owner`](../../src/emulator/native_engine.rs)
  diagnostic (lines 753-890; [ledger O-M1](ledger.md)) showed real MIPS
  writes through a fixed `Box` and visibility under a fresh Unicorn owner.
  It did **not** exercise `MemorySystem`, API publication, failure injection,
  clones, concurrent host access or iNavi. E-16 only proves an injected
  owner-qualified file-view sync path, not public same-VA memory.
- The installed `unicorn-engine` 2.1.5 wrapper owns `Unicorn::new` handles
  through a private `Rc<UnsafeCell<UnicornInner>>`. Its final `Drop` calls
  `uc_close` but discards the result (`lib.rs:140-165,185-205`); there is no
  public consuming checked-close or ownership-transfer method. The public
  `get_handle()` and re-exported raw `uc_close` do not make a checked close
  safe on that owner: an ordinary subsequent `Drop` would close the same
  pointer again. `Unicorn::from_handle` creates a separate non-owning wrapper,
  not a transfer of the original owner. A future patched consuming close or
  separately raw-owned handle with proved callback/alias teardown is required
  before E25 can release native pins after verified closure.

## Authority, identity and minimal data model

`MemorySystem` (eventually its `AddressSpace` member) is the sole private-byte
authority. It may group records by `ProcessBirthKey`, but must not add a second
PID registry. The initial cut needs an explicit allocation backing variant,
for example `LegacySeed` versus `StablePrivateSlab`; source kind alone is not
an activation switch. Only an internal E-18 constructor opts into the slab
variant. Existing public `VirtualAlloc` and other producers remain
`LegacySeed`, and every mapper/fault/reclaim path dispatches on that actual
backing variant so neither a slab page nor a legacy page is silently mapped
through the other's route. A reservation has checked allocation incarnation,
original 64-KiB-granular extent, source, and per-4-KiB committed/protection state.
Committed private pages own fixed, page-sized host storage; a reservation
alone allocates no host page. A page identity is conceptually:

```text
PrivatePageId = (ProcessBirthKey, allocation_incarnation, guest_page)
PrivatePage = (PrivatePageId, stable_backing_id, fixed_4096_byte_storage, protection)
NativePrivatePin = (native_owner_generation, PrivatePageId,
                    stable_backing_id, mapped_protection, strong_backing_pin)
```

`stable_backing_id` is checked identity, not a raw pointer used as a key.
Use non-moving allocation for each committed segment; the exact storage type
is reviewable, but a resizable `Vec` is prohibited. Put `UnsafeCell`/native
pointer access behind one small module and one execution/access fence. The
native engine may write while emulating; no Rust `&mut [u8]`, `RefMut` or
ordinary host read is alive across `emu_start`. A bounded hook callback obtains
its own explicitly scoped access. Stable address alone does not make aliases
or `Send`/`Sync` sound. Do not clone a strong pin into checkpoint state.

The native ledger belongs to one `NativeEngineLease`, records an exact page
identity and holds a strong pin. It is **observed map state**, never a VM
allocator. The pin registry must remain a lease-owned field from pending map
through successful reuse or checked owner closure. Today's
`NativeEngineLease::take_metadata()` removes the mapping metadata at
`unicorn.rs:6224-6226`; the caller keeps it in a local hook `Rc` from line
7349 and restores a clone only at line 10330. That transfer is not safe for
pointer pins: a setup, map, hook or sync early return can bypass the restore.
Keep private pins in a separate lease-resident registry (hooks may borrow a
scoped handle, but cannot own the last pin), or replace the transfer with an
equivalent guard that proves the last pin survives until owner closure. On a
new owner, derive desired pages from its `AddressSpace` and map the *same*
canonical private slabs. On a reused owner, compare desired
identity/protection to its ledger and native region before declaring an
existing VA valid. A mapped VA with no exact matching record is a backend
collision, not a reason to skip. Existing `virtual_pages`/`ranges` can
continue for unmigrated families but cannot stand in for private identity.

## API and dataflow sketch (names provisional)

```text
VM.resolve_process_birth(selected_pid) -> Result<ProcessBirthKey>
VM.prepare_private(owner, request) -> Result<PreparedPrivateVm>
PreparedPrivateVm.{base, allocation_id, pages_to_commit, rollback_token}
NativeMapLease.reconcile_private(owner, desired_pages) -> Result<NativeMapDelta>
VM.commit_private(prepared, validated_native_delta) -> Result<PublishedVmPointer>
VM.prepare_private_change(owner, allocation_id, change) -> Result<PreparedVmChange>
NativeMapLease.apply_private_change(prepared_change) -> Result<NativeMapDelta>
VM.commit_private_change(prepared_change, validated_native_delta) -> Result<()>
```

1. Resolve the canonical birth once for the attempted process. Capture it
   with the native run lease; later `kernel.current_process_id` changes do not
   retarget its pages. Reject unknown/stale births before map or API success.
2. For `MEM_RESERVE|MEM_COMMIT`, prepare a reservation and exactly the
   committed 4-KiB pages. Initialize canonical slabs before exposure.
   `MEM_RESERVE` alone creates metadata without slab or native RAM;
   commit-only must target one existing same-owner reservation. Unsupported
   or not-yet-migrated flag combinations fail explicitly rather than report
   committed, accessible bytes that do not exist.
3. If the owner is active, map/protect all committed pages into that native
   lease and validate before committing metadata and returning a pointer.
   For an inactive owner, commit canonical backing and mapping intent; its
   first lease must reconcile and validate before guest entry. The imported
   COREDLL return register and `LastError` are set only after this boundary.
4. Initial map and unmapped-fault repair both resolve **that attempt's**
   `(birth, allocation incarnation, page)` and use `mem_map_ptr` with an exact
   pin. Do not replay whole `initial_bytes` across already live private pages.
   Guest writes then directly update canonical storage; no private readback
   occurs at slice end. Legacy views/blobs still perform checked copy sync.
5. Decommit, release and protect prepare an exact same-owner change. A mapped
   active owner must perform checked native unmap/protect before metadata
   commit or pointer reuse. An idle fresh owner has no ledger; a persistent
   idle owner must reconcile or close before old backing can be retired.
   A page released at one VA cannot be reallocated under the old incarnation.
6. At slice end, capture CPU state into a candidate, then check outgoing
   legacy blob/view sync and native close/finish. Publish saved Context,
   process exit, debugger stop, scheduler handoff and continuation only after
   these checks pass. Private slab bytes already contain guest writes; no
   rollback of writes to a live slab is promised.

The first E25 implementation exposes only the internal single-address-space
E-18 diagnostic path while public API transactions and same-VA work are
incomplete. The opt-in backing variant must not be inferred from
`VirtualAllocationSource::Private`; public `VirtualAlloc` stays `LegacySeed`.
The cut must still use real `mem_map_ptr`, exact pins/owner checks and checked
failure latches, and must be labelled **diagnostic**, not production backing
acceptance. Steps 2-5 become required before any public call returns a
slab-backed pointer; exposing a half-mapped public allocation is not an
acceptable shortcut.

## Native pin, failure and rollback protocol

| Boundary | Required transition on failure |
| --- | --- |
| Prepare/initialize | No published allocation or pointer. Keep a monotonic unusable VA hole if a failed native/host write might have dirtied it. Do not recycle an uncertain reservation. |
| Before `mem_map_ptr` | Install a pending entry in the lease-resident pin registry before the FFI call. Validate page alignment, extent, source, owner, incarnation, protection and absence of a conflicting native region. A hook-scoped handle cannot be the final strong owner. |
| Map succeeds | Mark the exact ledger entry live before guest entry. If later metadata commit fails, checked-unmap it or close/poison the native owner while retaining the pin; never leave a mapped pointer to freed host storage. |
| Map fails or native side effect uncertain | Latch `BackendError` with owner/page/native result. Close/poison that lease before dropping the pending pin. Do not turn the stop into a normal CE return, `UnknownNormalStop` or PC-zero continuation. |
| Fault repair fails | Preserve the typed error from the hook; request checked native stop and reject continuation. The current `.unwrap_or(false)` path is not sufficient. |
| Decommit/release/protect fails | Keep canonical old allocation/protection if no guest bytes changed. If native state may have partially changed, poison/close its owner and retain pins until closure; no successful API return or same-VA reuse. |
| Outgoing legacy sync / hook cleanup fails | No successful saved Context or VM continuation; close/poison owner. Stage, then commit CPU state only after synchronization. Existing live slab writes may remain and must be attributed to the failed attempt. |
| Owner closes normally | Native pointer mappings and engine-specific blank Context/ledger must die before their strong pins may be released. With the current opaque owning wrapper, native close success is unobservable: quarantine pins before discarding that owner, including the nominal-success path. A future checked-close owner may release them only after it proves closure. The AddressSpace retains private slabs while allocations live; a fresh owner maps those same slabs. |

Add an explicit abort/poison operation for every error return, including
pre-run setup, in-hook mapping and post-run sync. It must report checked hook
cleanup errors to the typed backend latch, retain or quarantine pins before
discarding any owner, and never advertise a native-close proof it does not
have. `NativeEngineLease::Drop` currently calls `close(false)` and ignores its
result; even a successful `abort_close()` hook-cleanup result does **not**
verify the wrapper's native `uc_close`. For the internal E-18 diagnostic,
quarantine the last strong pin for every discarded owner (normal, abort,
setup-error and idle rotation) until emulator host-process exit or another
proven-safe retirement boundary; measure retained bytes and cap diagnostic
owner rotations. This avoids a
use-after-free but deliberately permits a diagnostic-only leak; it cannot
justify public slab backing or fresh-default O-P4. A checked close requires
either a patched consuming wrapper method or a separate raw-handle ownership
design that performs one checked `uc_close` after all borrowed wrappers,
hooks and callbacks have been retired. Do not infer map
identity solely from `uc.mem_regions()`: it reports extent/protection, not
the backing pointer or allocation incarnation.

The current normal path rotates at `unicorn.rs:10604` before checked
`finish()` at line 10606. Successful decoded-process-exit paths return `Ok`
at lines 10429 and 10479 without `finish()`; the explicit debugger stop
persists handles before sync/finish at lines 10560-10573. All three exits
need the same staged publication and checked finish or checked abort/poison
boundary. No early `Ok`, process retirement, PID rotation or saved Context
may outlive a failed checked hook-cleanup boundary merely because `Drop`
eventually ran. Native-close success cannot yet be asserted on the current
owning wrapper, so diagnostic-only publication must keep discarded-owner pins
quarantined; public/O-P4 publication remains blocked.

## Clone and lineage gate

Current `MemorySystem`, `VirtualAllocation`, `CeKernel` and `UnicornMips`
derive `Clone` ([`memory.rs:29`](../../src/ce/memory.rs),
[`memory.rs:151`](../../src/ce/memory.rs),
[`kernel.rs:312`](../../src/ce/kernel.rs),
[`unicorn.rs:91`](../../src/emulator/unicorn.rs)). Copying the existing
`ProcessBirthKey` and allocation cursor can reuse future incarnations; a
shallow strong pin would additionally share mutable private bytes. Neither is
a checkpoint. `HandleTable` also derives `Clone` and its birth keys lack
machine lineage. The monitor independently clones CPU and Kernel when saving
at `main.rs:2264-2274`, then assigns cloned CPU, Kernel and framebuffer in
sequence when restoring at lines 2286-2295. The narrow E25 policy is a
checked machine-level `try_checkpoint_clone` **and** `try_checkpoint_restore`:
both return a typed **Unsupported** result for slab-backed source or live
destination state before pushing a checkpoint or assigning any field. The
current `Error` enum has no Unsupported variant; add a typed variant/result
and handle it inside the monitor loop without exiting it. Remove or replace
generic derived `Clone` on slab-bearing structures and independent machine
copy routes, including test-only direct clones, so no alternate shallow
`Clone` escape can bypass the gate. A panic, shared slab, partial save or
partial restore is not fail-closed.

Full Stage 2 acceptance requires one machine-level clone/relineage operation:
mint new machine/checkpoint lineage, remap birth keys, allocation/view IDs,
current/pending selection and CPU/parked keys together; deep-copy private
slabs, retain clone-local shared aliases, and reject source-machine keys in
the clone. Until then, E25 diagnostic state is explicitly non-checkpointable.

## Expected-red controls and acceptance recipe

1. Preserve and explicitly run E-18 red before implementation. After the
   slice, unignore it only when the first guest store is visible in canonical
   backing **before** owner closure and the fresh owner's real MIPS load gets
   `0x11223345`, with no fallback readback or old owner retained. Promote the
   isolated pointer-map diagnostic from ignored only after its behavior is
   exercised through production VM/native-lease plumbing.
2. Add focused tests for untouched zero page; writes on two committed pages;
   reserve-only no native page/slab; commit into an existing reservation;
   protection/decommit/recommit; release then same-VA new incarnation with no
   old bytes; exact owner mismatch; native already-mapped VA collision; and
   injected second-page map, unmap, protect and fault-repair failures. Verify
   old/partial metadata, pin counts, closed/poisoned owner and no successful
   saved continuation—not only API return values.
3. Add negative checkpoint tests: slab-backed save and restore each return
   `Unsupported` before any CPU, Kernel or framebuffer mutation, including
   restore of a pre-slab checkpoint into a live slab-backed machine. No
   independent field `Clone` escape may bypass the supported API. Later
   replace this gate with positive deep-copy/relineage and stale-source-key
   rejection tests.
4. Keep E-16 injected file-view controls green. Keep public E-20 same-VA
   ignored until public parent/child calls both obtain the same VA and real
   native A→B→A read/write slices prove independent bytes. Then test same-VA
   private versus foreign legacy view, public query/free isolation, Ex target
   resolution and closed-handle live views. A metadata-only E-20 pass is not
   enough to remove its ignore marker.
5. Keep O-P4 fresh-default ignored until private **and** shared/view outgoing
   bytes, exact-key Context transfer and closed-owner tests pass. Then run
   focused tests, complete release `unicorn,trace,win32-desktop`, no-feature
   and alternate/no-trace checks. Record source/config hashes and raw logs
   outside disposable `target/`.
6. Only after those gates, run several bounded phase-aligned iNavi controls
   with an explicit watchdog, frame/input-consumption evidence and original
   mounted-data protection. Splash, HTTP ACK and one clean 330-s exit are
   not map/navigation readiness. Report boot-to-first-frame/map times,
   busy MIPS and peak RSS/private for every live run.

## Instrumentation and measurement

At each attempt/snapshot, record bounded counts/bytes for each AddressSpace:
reserved extent, committed guest pages, allocated slab capacity, shared-view
capacity, native pointer-map page count, live pins, native owner count and
generation, protection changes, map/unmap/fault errors, and stale-key
rejections. Keep loader/heap blob length and capacity, native TCG arena,
host working set/private bytes, trace capacity, guest instructions and
setup/sync/guest-run timing separate. Do not retain full page dumps or log
guest payload bytes by default. The E-22 splash-only 330-s run peaked at
1,543.9 MiB working set and 1,554.4 MiB private bytes
([ledger E-22](ledger.md)); measure comparable phases before claiming a
footprint or latency gain. No fixed CE-sized host-memory threshold is inferred
from that one run.

## Bounded implementation sequence and stop gates

1. Independent review of this record against A2/A4, D-008, CE6 source,
   native header and the exact current call sites. Resolve the clone entry
   point and whether first delivery is diagnostic-only or public before code.
2. Add expected-red failure/identity fixtures, then an internal E-18-only
   opt-in backing variant, private page owner and lease-resident pin type with
   host-access fence. Public `VirtualAlloc` stays `LegacySeed`. Avoid broad
   shared/image/heap rewrites and avoid a second VM directory.
3. Thread captured `ProcessBirthKey` through native lease, initial mapping,
   import-hook map, fault repair, reclaim and slice-end checks; replace
   `mem_map`/seed only for the opt-in slab-backed private path with an exact
   `mem_map_ptr` ledger. Keep public and other legacy paths explicit.
4. Implement checked import/fault error latches, map rollback/poison, explicit
   abort/poison with conservative discarded-owner pin quarantine, and staged
   continuation/process-exit/stop/rotation. Do not treat hook cleanup as a
   checked native close. Keep public calls on their legacy backing in this
   cut; later implement a single-owner checked native-close path and their
   prepare/map/commit and free/protect/decommit transitions before publishing
   slab-backed success. Do not relax the global overlap fence until
   owner-qualified native mapping plus same-owner cross-family collision
   checks are executable.
5. Run the focused E-18/control matrix, then full suites/feature checks.
   Review diff and evidence, update ledger/subsystem/status docs in a later
   reviewed step, and preserve only useful receipts outside `target/`.

**Stop after any failed native identity/map/close check.** Do not paper over it
with a copy-back path, a fresh-default switch, a guessed PID, a reused VA or
a new test ignore. The next stage advances only when its exact red gate and
failure controls are green on the same source.
