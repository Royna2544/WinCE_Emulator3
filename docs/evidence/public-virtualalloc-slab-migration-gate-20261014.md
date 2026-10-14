# Public `VirtualAlloc` slab migration: next dependency gate

Status: **plan only; no public backing change or test result is claimed here.**
This is a narrow update to [D-008](../DECISIONS.md), the [Stage 2 plan](../PLAN.md),
and the detailed [E25 design](e25-private-slab-design.md), not a replacement for
them. E25 proved guest-written bytes survive a fresh native owner only through
an internal `cfg(test)` slab opt-in. Public `VirtualAlloc` still selects
`LegacySeed` ([allocator](../../src/ce/memory.rs),
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
- Inject second-page map, fault-repair, protect, unmap, hook cleanup, and
  close failures. Assert no successful pointer/continuation/exit/rotation
  publication, no stale-VA reuse, and retained pins until proven unmap or
  close. Preserve the E16 owner-qualified view regressions.

## Gates that the transaction cannot bypass

The current [monitor checkpoint guard](../../src/main.rs) rejects slab-backed
save/restore, but generic `Clone` paths still exist and birth keys lack machine
lineage. Either make *every* slab-backed checkpoint route return typed
`Unsupported` before mutation, with negative save/restore tests, or implement
one machine-level deep-copy/relineage operation that rejects source keys in
the clone. A field-by-field clone is not the latter.

The current [native lease](../../src/emulator/native_engine.rs) can check hook
cleanup but its owning Unicorn wrapper discards `uc_close` status; pin
quarantine is diagnostic safety, not bounded public retirement. Before public
backing, provide a unique checked-close owner and inject close failures both
before and after possible native side effects, proving one close attempt, no
alias escape/double close, and no early callback or pin drop. On any mapping,
sync, or close failure, keep old metadata when still valid, poison the native
attempt if side effects are uncertain, retain its pins, and forbid a successful
continuation. Already-executed writes to a live slab are attributed to the
failed attempt, not promised rollback.

Only then remove the test-only selector, unignore the appropriate public
controls, and run focused, complete release, no-feature and alternate-feature
validation. Public shared-view coherence, full clone/relineage and O-P4
fresh-default acceptance remain distinct Stage 2 gates; another splash-only
bounded run would not close them.
