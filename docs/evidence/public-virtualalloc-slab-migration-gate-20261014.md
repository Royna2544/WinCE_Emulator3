# Public `VirtualAlloc` slab migration: next dependency gate

Status: **E-35 implementation in progress; public backing is not yet validated.**
This is a narrow update to [D-008](../DECISIONS.md), the [Stage 2 plan](../PLAN.md),
and the detailed [E25 design](e25-private-slab-design.md), not a replacement for
them. E25 proved guest-written bytes survive a fresh native owner only through
an internal `cfg(test)` slab opt-in. At that baseline, public `VirtualAlloc`
selected `LegacySeed` ([allocator](../../src/ce/memory.rs),
[COREDLL dispatch](../../src/ce/coredll.rs)). The later [E29](ledger.md)
330-second app run ended on splash despite observed GPS `COM7:` transfers and
tap dequeue; it neither identifies a VM cause nor relaxes this gate. E-34
closed O-P1 stop classification, a separate CPU dependency; that does not
switch public memory backing by itself.

## Smallest connected first implementation

Implement one **private-allocation-family transaction** behind an explicit
test-only opt-in while the production public selector remains `LegacySeed`:

1. Capture the canonical `ProcessBirthKey` once at the public call/native
   attempt, then make reservation, allocation incarnation, exact region
   overlap, and native map/fault/reclaim decisions owner-qualified. Check
   collisions with legacy file views, image/heap regions, and existing native
   maps; removing the global VA fence alone would permit silent aliasing.
2. Prepare a private reservation and only its committed 4-KiB pages. For an
   active owner, initialize stable backing, acquire lease-resident pins, and
   validate every `mem_map_ptr` **before** publishing the allocation, guest
   pointer, return register, or success status. For an inactive owner, publish
   canonical bytes and mapping intent only; its first run must reconcile
   before guest entry. Support reserve-only and commit into an existing
   same-owner reservation in the transaction, not by treating every request
   as the E25 diagnostic's single `MEM_RESERVE|MEM_COMMIT` case.
3. Connect `VirtualFree`, decommit/recommit, and `VirtualProtect` to prepared,
   checked native changes for that same family. Keep shared views, image and
   heap on their explicit checked copy boundaries; do not infer private slab
   backing from `VirtualAllocationSource::Private` or migrate them implicitly.

The first slice can add the transaction and its controls without turning it
on for ordinary callers. A public slab-backed pointer is permitted only when
the whole private-family publication/change boundary below is green; a
one-line change to `MemorySystem::virtual_alloc` is not that boundary.

## Expected-red public controls to stage before enabling

- Keep the existing ignored E20
  [`public_private_virtual_alloc_and_query_follow_process_rotation`](../../src/ce/coredll.rs)
  red until parent and child can allocate one VA, then add real native
  A→B→A store/load slices proving independent bytes and owner-specific
  Query/Free. Its current child-allocation failure is only the first reached
  assertion, not a native isolation result.
- Add a **public-ordinal** counterpart to E25/E18: call COREDLL `VirtualAlloc`,
  write `0x11223345` with MIPS, close the completed owner, then read it through
  a fresh owner's MIPS load. Assert canonical bytes before closure, exact PCs,
  normal stop, and no fault; do not use the internal slab constructor or retain
  the old engine to make it pass.
- Add public reserve-only/no native page, commit-only into that reservation,
  untouched zero page, two-page writes, protect, decommit/recommit, release
  and same-VA new-incarnation controls. Include private-versus-foreign-view
  overlap and current-only/remote Ex target behavior; a metadata-only success
  is insufficient.
- Inject second-page map, fault-repair, protect, and unmap failures. Assert no
  successful pointer/continuation/exit/rotation publication, no stale-VA
  reuse, and retained pins after an uncertain unmap. Preserve the E16
  owner-qualified view regressions. Checked hook cleanup and engine-close
  injection belong to O-P4, not this E-35 public-VM gate.

## Gates that the transaction cannot bypass

E-35 uses the first checkpoint boundary: the monitor's save and restore routes
call the [typed guard](../../src/main.rs) before cloning or replacing state, and
return `Unsupported` when either the current machine or a saved snapshot has
slab-backed private VM. The main-binary tests cover rejected save, rejected
restore into a slab-backed destination, and rejected restore from a slab-backed
snapshot into a clean destination. `MemorySystem::clone` deep-copies private
page bytes; its isolation control belongs with the memory tests. These are
the E-35 negative checkpoint and copy boundaries, not full independent
checkpoint support. Machine-level birth-key relineage and enabled slab-backed
checkpoint save/restore remain Stage 3 work. Record green test evidence before
claiming this gate passed.

The persistent engine can outlive many VM allocations. E-35 therefore tracks
native page pins by page and releases a pin only after that page's checked
`mem_unmap` succeeds. A failed unmap retains the affected pin and charges its
bytes against the same device-derived private-RAM budget as live committed
pages; otherwise repeated guest free/recommit could grow host memory without
limit while appearing under budget. A 64-cycle commit/free/recommit control
must keep pin count equal to live pages, and a forced unmap failure must reduce
the remaining budget. On map/protect/unmap or sync failure, forbid successful
pointer or continuation publication, keep old metadata only where still valid,
and poison an attempt with uncertain native side effects. Already-executed
writes to a live slab are attributed to the failed attempt, not promised
rollback. The [native lease](../../src/emulator/native_engine.rs) still cannot
verify `uc_close` through its Unicorn wrapper; checked close and hook teardown
are deferred to O-P4, with conservative pin retention if close is uncertain.

Only then remove the test-only selector, unignore the appropriate public
controls, and run focused, complete release, no-feature and alternate-feature
validation. Public shared-view coherence, full clone/relineage and O-P4
fresh-default acceptance remain distinct Stage 2 gates; another splash-only
bounded run would not close them.
