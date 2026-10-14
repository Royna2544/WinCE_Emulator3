# Public `VirtualAlloc` slab migration: E-35 validation gate

Status: **E-35 public private-allocation-family gate validated on frozen
source.** The normal public-ordinal control first failed at the intended
canonical-backing assertion before the switch, then passed on the frozen
source; focused and complete feature-profile checks are recorded in the
[validation receipt](../../evidence/e35-public-virtualalloc-validation-20261014/validation.receipt.md).
This is a narrow update to [D-008](../DECISIONS.md), the [Stage 2 plan](../PLAN.md),
and the detailed [E25 design](e25-private-slab-design.md), not a replacement for
them. E25 proved guest-written bytes survive a fresh native owner only through
an internal `cfg(test)` slab opt-in. At that baseline, public `VirtualAlloc`
selected `LegacySeed`; the current COREDLL private path uses prepared slab
transactions ([allocator](../../src/ce/memory.rs),
[COREDLL dispatch](../../src/ce/coredll.rs)). The later [E29](ledger.md)
330-second app run ended on splash despite observed GPS `COM7:` transfers and
tap dequeue; it neither identifies a VM cause nor relaxes this gate. E-34
closed O-P1 stop classification, a separate CPU dependency; that does not
switch public memory backing by itself.

## Connected implementation validated for E-35 scope

The public COREDLL `VirtualAlloc`/current-target `VirtualAllocEx` private
family now prepares a detached, owner-qualified slab proposal. The native
adapter applies it before metadata or a success return is published. The
legacy allocator remains for unmigrated view/image/heap compatibility paths;
that is not a private-VM test-only selector. The connected boundary is:

1. A captured `ProcessBirthKey` and allocation incarnation qualify the
   reservation, 4-KiB page identities, and native mapping decisions. Private
   reservations may reuse the same VA across process births; overlap checks
   retain the boundaries for file views, image/heap regions, and unrelated
   native maps.
2. Reserve-only publishes no committed native page. Commit creates exact
   zeroed 4-KiB backing, including commit into an existing same-owner
   reservation. The active native owner maps and pins checked pages before
   publication; an inactive owner carries canonical bytes until reconciliation
   before guest entry. No successful public pointer is published after a
   checked native failure.
3. `VirtualFree` release/decommit and `VirtualProtect` use prepared checked
   native changes for the slab family. Query, pointer probes, and current-only
   Ex target checks use the owner-qualified private record. Shared views,
   image, and heap remain on their separate compatibility boundaries.

The public switch passed the E-35 private-family publication/change boundary.
It does not by itself establish every O-M1 or Stage 2 acceptance condition.

## Public controls and remaining proof

- The former ignored E20
  [`public_private_virtual_alloc_and_query_follow_process_rotation`](../../src/ce/coredll.rs)
  is non-ignored and exercises public parent/child same-VA metadata, Query,
  and Free. Real native A→B→A store/load isolation remains a distinct control;
  metadata success alone does not prove native separation.
- The normal, non-ignored
  [`public_virtual_alloc_ordinal_store_survives_fresh_native_owner`](../../src/emulator/unicorn.rs)
  control calls the public COREDLL ordinal, stores with MIPS, closes its
  completed owner, and loads through a fresh owner. Its pre-switch canonical
  backing assertion failed as expected; the frozen-source control then passed
  1/0/0, alongside the raw integration control.
- Unit and native controls now cover reserve-only/no native page, commit into
  the reservation, zero pages, two-page writes, protect, decommit/recommit,
  release, same-VA new incarnation, file-view collision, and current/remote Ex
  behavior. The frozen-source release suite passed 1,727/0/15 ignored and the
  no-feature suite passed 1,292/0/13 ignored, including the applicable older
  owner-qualified view regressions.
- Failure controls include public adapter map/protect/unmap rejection and a
  native test seam that simulates an unmap failure immediately before
  `uc.mem_unmap`; this proves retention accounting at that boundary, not an
  actual Unicorn-returned unmap error. Checked hook cleanup and
  engine-close injection belong to O-P4, not this E-35 public-VM gate.

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
checkpoint save/restore remain Stage 3 work. The negative guard and clone
isolation controls are included in the frozen-source release suite.

The persistent engine can outlive many VM allocations. E-35 tracks native
page pins by page and releases a pin only after that page's checked
`mem_unmap` succeeds. A failed unmap retains the affected pin and charges its
bytes against the same private-RAM budget as live committed pages. That budget
defaults to the dumped device's 195,350,528 physical-RAM bytes and is
configurable with `WINCE_EMU_GUEST_PHYSICAL_RAM_BYTES`. The 64-cycle checked
unmap test and a pre-FFI injected unmap failure test passed in the release
suite. On map/protect/unmap or sync failure, forbid successful
pointer or continuation publication, keep old metadata only where still valid,
and poison an attempt with uncertain native side effects. Already-executed
writes to a live slab are attributed to the failed attempt, not promised
rollback. The [native lease](../../src/emulator/native_engine.rs) still cannot
verify `uc_close` through its Unicorn wrapper; checked close and hook teardown
are deferred to O-P4, with conservative pin retention if close is uncertain.

The old test-only slab selector has been removed and the public controls are
non-ignored. Focused, complete release, no-feature and both no-trace checks
passed with source and raw-log hashes in the receipt. Public shared-view
coherence, real native two-process A→B→A same-VA bytes, and O-P4 fresh-default
acceptance remain distinct Stage 2 gates; full machine checkpoint relineage
is Stage 3. No E-35 app run occurred, and another splash-only bounded run
would not close those gates.
