# Known Bugs And Risks

Refreshed on 2026-10-17. This file lists unresolved behavior only; resolved entries are removed (history: docs/evidence/ledger.md and git).

## Open Issues

- **Nested CPU ownership outside the witnessed two-node shape remains stopped:**
  the earlier [real-MIPS census](evidence/stage3-nested-cpu-census-20261017/validation.receipt.md)
  counted A/B/C physical payloads 1/1/1, then reached a typed pre-entry
  `NoAttempt` because Kernel selected B while the active CPU owned C. At the
  later [connected source seal](evidence/stage3-connected-port-20261017/validation.receipt.md),
  the B-origin return instead retained the C-root/B-parked tree as
  `UnverifiedStopped`, with A's private continuation poisoned. The newer
  focused [witnessed handoff controls](evidence/stage3-rotated-handoff-20261017/a11-rotated-five-challenges.raw.log)
  execute later A/B/C guest slices for exactly one verified C-root/parked-B
  pair and preserve both original payload labels. A
  [no-witness nested E→D control](evidence/stage3-rotated-handoff-20261017/a12-no-witness-retention.raw.log)
  still stops with its whole tree retained and untouched siblings queued;
  malformed selected C metadata also fails closed. A B-only restore guard,
  relabeling C as B, or putting C in B's reserved slot remains unsound.
  Deeper trees, general exact-slot/leaf selection, recoverable machine-owned
  custody and in-session recovery remain open. No iNavi or mount run is
  attributed here.
- **Other post-native child handoffs still restore mutable process selection
  without general checked lineage:** the [proposed port boundary](docs/subsystems/tasks-scheduler.md#callback-scoped-machine-state-port-cut-proposed-not-implemented)
  identifies blind save/set/restore of selected module fields and
  `CurrentProcessState` across B's run. The selected-context stamp passed its
  [bounded preparatory gate](evidence/stage3-selector-context-stamp-20261017/validation.receipt.md).
  A [bounded pre-admission gate](evidence/stage3-deferred-context-preflight-20261017/validation.receipt.md)
  checked that stamp, exact live A owner/caller and nonzero staged PC before
  the then-aggregate pending-launch drain. A changed generation refuses with
  both original queued births still present; an unchanged-context control
  admits both, without creator attribution. At that earlier seal, focused
  **17/0**, 19-block no-default **1,548/0/2 ignored** and release
  **2,192/0/9 ignored** suites passed. The
  preflight does not check or safely restore selection after B runs, bind
  cloned Kernels to one machine lineage, or cover arbitrary mutation through
  public backing objects. The later bounded
  [P0 queue gate](evidence/stage3-pending-queue-p0-20261017/validation.receipt.md)
  adds in-place exact-birth claims. The bounded P1 gate adds a neutral Event
  offer with a selected-context check. The later combined monitor/P2 gate
  adds exact B-only producer claims and an original direct-`.exe` Shell
  trigger, but not general lineage-checked restoration for the deferred
  `CreateProcessW` handoff; raw
  `CreateProcessW` queues B before the post-native caller-birth check. In the
  tested canonical/private A path,
  refusal poisons its saved
  continuation and requires recovery, not resume; aliasless synthetic
  nonprivate attempts do not prove that poison or an exact `failure_operation`
  record. This is a source-audited risk, not an attributed live-app failure;
  the connected port now exists in a bounded source/test cut, but checked
  parent suspension, final thread/leaf authority and general post-B
  restoration remain open. A later [test-only real-MIPS A→B→C observation](evidence/stage3-post-child-selection-observation-20261017/validation.receipt.md)
  reached B's post-child/pre-parent-restore boundary with Kernel selecting C
  and the returned child CPU slot owning C's launch, not B's. The current
  [parked-process activation](src/emulator/unicorn.rs#L8229) can swap the
  active CPU payload during B's slice. A B-only restoration check or transfer
  of that returned payload into B's reservation would reject or mislabel this
  reached nested shape; it is not a safe preparatory fix. A newer focused
  witnessed two-node branch checks A's exact caller/selection before
  publishing B and C, but other child-return paths still use the legacy
  restoration path. The [a09 control](evidence/stage3-rotated-handoff-20261017/a09-selection-observation-green.raw.log)
  ends with an ordinary outer rotation selecting B/B after A was restored;
  it must not be described as a final A selection. General checked
  restoration remains O-T7 work. Neither observation attributes a live-app
  failure.
- **Independent launch service remains incomplete outside the bounded
  routes:** raw [`CeEventHasOccurred`](src/ce/coredll.rs#L23551) and
  [`ShellExecuteExW`](src/ce/coredll.rs#L34785) publish exact-birth launches
  in [`CeKernel`](src/ce/kernel.rs#L3431). P1 offers the oldest Event at a
  neutral host turn; the [combined monitor/P2 gate](evidence/stage3-launch-handoff-p2-combined-20261017/validation.receipt.md)
  adds a bounded monitor Event offer and exact B-only handoff for native
  `CreateProcessW` and original non-shortcut direct `.exe` Shell. `.lnk`,
  association, URL and other Shell routes may still queue without that
  immediate producer service. A requested-PC monitor command leaves E queued
  until a later ordinary `continue`; checkpoint save refuses pending Task
  authority. The `notify.dll` `NFY0:` static IOCTL path still gives no CE
  before/after-return launch timing. No live iNavi behavior is proven or
  attributed to this issue.
- **Shell output success still leaves alias and status semantics open:** the
  direct-EXE `hInstApp`/requested `hProcess` copy-fault controls passed under
  the [sealed source gate](evidence/stage3-shell-createprocess-handoff-20261017/validation.receipt.md),
  but an unwritable `hProcess` can leave its process alias undisclosed, with
  no executed retirement control. The shim still sets success-path
  `LastError=0`; bad-pointer CE behavior was not established. `TRUE` can
  precede loader admission. The later P2 gate services only the original
  direct-`.exe` route; other routes above remain open. None is attributed to
  a live iNavi failure.
- **`CreateProcessW` output correction leaves alias lifetime and loader status
  open:** [the raw shim](src/ce/coredll.rs#L29236) now returns `TRUE` for a
  published child even when optional `PROCESS_INFORMATION` is null or cannot
  receive all four words. The [sealed output gate](evidence/ce6-createprocess-output-contract-20261017/validation.receipt.md)
  passed 19-block no-default **1,550/0/2 ignored** and release-feature
  **2,194/0/9 ignored** suites; a06–a08 feature checks exited 0. Process/thread
  handle aliases are allocated before output copying. With null output the
  guest receives neither number, while canonical completion retains aliases;
  their retirement in that shape
  has no executed control and may retain completed records. The raw `TRUE`
  still precedes PE load and CPU admission. The existing forced success-path
  `LastError=0` is a known CE6 mismatch and is not validated by this gate.
  Neither output-copy failure nor partial guest-byte writes imply launch
  rollback. P0 replaced local queue draining, P1 added Event service, and
  bounded P2 now isolates witnessed B from older E for the tested native
  routes. Alias retirement and the Boolean-before-loader gap remain open;
  none is attributed to a live iNavi failure.
- **Process exit leaves general private VM and mapped-file views behind:**
  exact-birth [process completion](src/ce/kernel.rs#L3555) retires fixed
  heap/stack ranges, writable-image runs and heap state, but not ordinary
  private `VirtualAlloc` records or `FileViewSlab` allocations. It also does
  not remove `FileMappingObject.views` from the
  [handle table](src/ce/object.rs#L1756). Explicit
  [UnmapViewOfFile](src/ce/coredll.rs#L31147) releases a public birth-keyed
  view and its shared slab (or a legacy view's private allocation), whereas
  [CloseHandle](src/ce/kernel.rs#L6865) preserves a mapping object with live
  views. [Legacy injected/DIB views](src/ce/object.rs#L225) can carry only
  a numeric PID, without a process birth, so a future exit sweep must not
  attribute them to a recycled PID. Make exact-birth view/private-VM cleanup
  a separate tested cut; Cut 3a CPU quarantine neither causes nor repairs
  this Kernel lifetime gap, and must not claim general VM/view release.
- **Intermittent `0xc0000005` under the parallel default harness:** the
  release-feature native-failure broad attempt 02 reported
  `guest_thread_stack_tests::direct_visible_message_payload_windowpos_matches_raw_and_mips_reads`
  overflowing its test-thread stack, then exited `0xc0000005`
  ([raw log](evidence/stage2-writable-image-native-failure-20261016/broad-release-attempt02.raw.log)).
  In the earlier three stamped default-harness full release runs, one crashed
  and two passed (that native-failure attempt 02, its retry 04, and the
  [file-view attempt 01](evidence/stage2-writable-image-file-view-20261016/broad-release-attempt01.raw.log)).
  A fourth, later [VM-allocation full release run](evidence/stage2-writable-image-vm-allocation-20261016/validation.receipt.md)
  passed on changed source, giving one crash among four observed full-release
  attempts, not a stable reproduction-rate estimate.
  Three later separately stamped, default-harness release `--lib` reruns
  passed (**0/3** WINDOWPOS crashes); those library-only runs are not pooled
  with the earlier full-suite denominator
  ([receipt](evidence/stage2-windowpos-stack-20261016/lib-rate.validation.receipt.md)).
  The exact isolated attempt 01, serial broad attempt 03, and default broad
  retry 04 passed the same test. A later isolated direct-EXE probe passed
  at requested 512, 256 and 128 KiB test-thread stacks, then overflowed at
  64 KiB ([receipt](evidence/stage2-windowpos-stack-20261016/validation.receipt.md));
  this does not explain the earlier broad-run overflow, whose requested
  test-thread stack size was not explicitly stamped in the raw log. The test clones a
  full `CeKernel` for raw Get/Peek comparison and also exercises the shared
  native callout/payload path;
  which part consumes the stack is unknown. The existing release EXE's PE
  header reserves 1 MiB for its main stack, but source-level stack sizing
  and live-run exposure have not been established. Separately, the first
  full-release writable-image exit-gate attempt emitted one `FAILED` line for
  `mixed_native_deadline_visible_peer_yield_preserves_checked_jr_continuation`,
  then the library harness exited `0xc0000005` before a result block
  ([raw log](evidence/stage2-writable-image-exit-retire-20261016/broad-release-attempt01.raw.log)).
  That line does not locate the fault. The mixed-deadline test passed alone
  (**1/0/0**) and the next complete release attempt passed (**2,132/0/9**)
  ([receipt](evidence/stage2-writable-image-exit-retire-20261016/validation.receipt.md)).
  Neither incident is explained by those passes; a shared cause and test-only
  scope are unproved. Concurrent native instances under the test harness
  remain one possibility, but unsafe host backing handed to `mem_map_ptr`,
  pins or hooks could also expose a production memory-safety defect. A
  [matched library cohort](evidence/parallel-native-av-cohort-20261016/validation.receipt.md)
  observed 0/3 crashes with explicit serial harness threads and 0/3 with the
  default harness on one source seal; neither arm reproduces or explains the
  earlier full-suite failures. Native nonzero-timeout execution itself starts
  a timer worker, so one Rust emulation thread does not establish native
  single-thread safety. Its `emulation_done`/`stop_request` are unsynchronized
  native fields, but `restore_jit_state` is a no-op on this Windows build;
  concurrent JIT/W^X restoration is not a demonstrated Windows AV cause.
  A bounded opt-in timer/hook-stop rendezvous completed 48/48 fresh-owner
  iterations across three release orders with no watchdog or native failure
  ([receipt](evidence/native-timer-rendezvous-20261016/validation.receipt.md));
  this controlled non-reproduction neither explains the AVs nor proves the
  unsynchronized fields safe. The normal timeout path was not changed.
  Audit those lifetimes and process-global state; do not
  mask the WINDOWPOS stack demand by increasing the
  test stack; isolate and remove unnecessary large by-value or recursive paths
  before live acceptance.
- **Native memory performance remains a live-run gate:** a synthetic
  8,192-page (32-MiB) fixed heap originally took 78,689.550 ms to map and
  another 75.753 s to close its fresh owner. Fixed private heap/stack pages
  now use sparse 1-MiB host blocks and per-page pins/budget; the final
  release control mapped the same 8,192 pages in 32 calls and 5.056 ms,
  closing in 0.530 ms. The same-binary old-shape control omitted the old
  end-of-slice readback copy, so no full-slice speed ordering is established.
  This is not an iNavi speed, lag or RSS result
  ([mapper receipt](evidence/om2-native-map-profile-20261014/validation.receipt.md)).
- Public shared-view slabs and ordinary private `VirtualAlloc` slabs still
  have independent 4-KiB host pages and one `mem_map_ptr` per mapped page.
  They are deliberately outside the fixed heap/stack block migration; the
  saved iNavi evidence does not report a shared-view page count. Their
  real-run cost and any further grouping remain unresolved.
- Stage 8 (persistent owner, paused): E-32 crashed at 135.3 s and the 650
  trial at 129.1 s, both in TCG code emission, cause unattributed. E-33 fixed
  only the small-arena commit-tail arithmetic. Still open: VEH-handler versus
  owner-teardown race, x64 trampoline unwind metadata, the 1 GiB per-engine
  arena, E-12 map/unmap churn, in-hook register helpers that zero failed reads
  and drop write errors, and unaudited o32 float-return restores (ledger E-2
  to E-6, E-12, E-32, E-33).
- VM coherence still unproved across the general guest `CreateProcessW`/
  scheduler handoff, distinct public mappings at one VA, and full machine
  clone relineage. Earlier VM fixtures selected the child manually; the current
  source has an automatic post-native path in both modes, pending a final
  source/test gate. A same-PID DIB view without VA metadata can Flush but not
  Unmap. The former immediate CreateProcess view-sync failure path is
  historical; new post-native query/invalid-PE controls do not establish
  general failure recovery. `VirtualAllocEx`/`VirtualQueryEx` name exports are
  absent. History: ledger E-14 to E-41.
- Monitor checkpoint/rewind is deliberately unavailable while a CPU has
  execution custody or a pending launch or canonical Kernel Task record is
  retained, even if that record has stopped. The bounded
  [canonical-map lineage guard](evidence/stage3-checkpoint-orphan-frame-green-20261016/validation.receipt.md)
  prevents duplicate births and its seven formerly ignored controls now pass
  as ordinary tests in [both full suites](evidence/stage3-checkpoint-unignored-20261017/validation.receipt.md),
  but it does not relineage the whole machine. A
  D-011 stopped-exit hold therefore has no safe in-session rewind today;
  recovery requires quit, repair and restart. This is a safety limitation,
  not a successful live-checkpoint implementation.
- D-008 charges the whole anonymous mapping at first public `MapViewOfFile`,
  not at `CreateFileMapping` as documented for default `SEC_COMMIT` in
  [Microsoft's CE mapping reference](https://learn.microsoft.com/en-us/previous-versions/windows/embedded/ms923951%28v%3Dmsdn.10%29);
  CE6 device-specific accounting remains unmeasured. Raw/non-Unicorn alias
  propagation is also not rollback-atomic after a failed guest write; native
  pointer aliases share canonical bytes directly
  ([D-008 receipt](evidence/d008-shared-view-20261014/validation.receipt.md)).
- GMS's boot-reserved physical RAM and stable private-page commit use one
  budget, with four public controls passing both E-39 broad suites. Only
  stable private pages, D-008 stable shared-view pages, and E-44's committed
  fixed heap/stack slabs (including O-M2's sparse heap growth) enter the
  source-gated accounting path. Persistent native pin budget release is
  deferred until checked owner closure, while image and legacy-view memory
  are not charged by this budget. Copied heap spillover was removed in O-M2.
  It is not a complete guest-memory census
  ([GMS receipt](evidence/gms-device-report-20261014/validation.receipt.md),
  [O-M2 receipt](evidence/om2-heap-20261014/validation.receipt.md)).
- The normal production SD main-image loader and preloaded/runtime DLLs
  remain copy-backed; the sealed R/RX path excludes mutable image and trap
  pages. The historical writable-image expected red isolated stale loader
  bytes without readback, and the mapped-code R02 red isolated a stale
  same-slice instruction read. Opt-in synthetic VM adoption and native code
  reading now pass bounded real-MIPS/fresh-owner controls, but neither is
  production-path activation
  ([writable green](evidence/stage2-writable-image-green-20261016/validation.receipt.md),
  [code-read green](evidence/stage2-mutable-code-fallback-20261016/validation.receipt.md)).
  A synthetic second-run native `mem_map_ptr` collision now confirms pin
  retention through checked abort-close and refusal to reuse a staged
  continuation; a failed native owner close itself is not injected
  ([receipt](evidence/stage2-writable-image-native-failure-20261016/validation.receipt.md)).
  Legacy file-view seed/readback now refuses overlap with active or
  retired-but-pinned writable-image runs before byte mutation
  ([receipt](evidence/stage2-writable-image-file-view-20261016/validation.receipt.md)).
  The bounded VM-allocation gate also fences same-birth LegacySeed,
  private/shared reservations, pending holes, fixed heap and heap cursor,
  and native seed/reclaim/fault/map collisions; foreign births may still
  reuse a numeric VA
  ([receipt](evidence/stage2-writable-image-vm-allocation-20261016/validation.receipt.md)).
  Other direct host-write/restore/rebind and exact module-retirement
  fences, DLL unload/reload, full machine checkpoint relineage, O-M3 guest
  physical accounting and an approved matched app run remain open. The synthetic
  `Unclassified` image backing charges no guest pages and cannot establish
  production `PrivatePageBudget` treatment.
  Unconverted direct-import outputs can still fail closed on a
  cold, untouched stack page; a per-thread/slot 128-KiB contiguous low-water
  is conditional on an approved live run encountering that bypass. O-M2's
  exact stale different-birth serial/NMEA control passes, but no new iNavi
  memory, speed or map-readiness result has been measured
  ([O-M2 receipt](evidence/om2-heap-20261014/validation.receipt.md),
  [E-44 receipt](evidence/heap-stack-20261014/validation.receipt.md),
  [E-43 receipt](evidence/sealed-main-image-20261014/validation.receipt.md),
  [E-42 receipt](evidence/immutable-image-pages-20261014/validation.receipt.md)).
- Stage 3 task authority is not accepted. The passed bounded gates are listed
  in `PROGRESS.md` ("2026-10-15 Stage 3 source gates"). Still open:
  - Boot-main Toolhelp enumeration remains ignored (ledger E-8, E-11, E-26).
    The child-primary queue/trace-ring red is now green through the
    [canonical-primary source gate](evidence/stage3-primary-canonical-green-20261017/validation.receipt.md),
    and a later [bounded stopped-exit gate](evidence/stage3-primary-exit-birth-v4-20261017/validation.receipt.md)
    verifies that birth for tested child/boot exits. Other exit consumers and
    rollback/publication states are not thereby accepted.
  - CE caller wait/timeout behavior is not implemented. An old-MIPS
    direct-kernel real-handle continuation fails closed, the root process has
    no owned child-launch handoff for real-handle requests, and the
    raw direct current-process pseudo-handle calls keep a separate synchronous
    compatibility path. The decoded self-exit shape has a later bounded
    stopped-owner gate, not a general real-handle continuation.
  - Birth ownership covers the producers and exit paths gated in `PROGRESS.md`
    ("Stage 3 source gates"), each with its receipt. Still open:
    - Broadcast, HWND-owned timers, hotkeys, sent/paint/modal and other input
      producers, full exit capture retirement and implicit gesture ownership;
      their isolation under TID/PID reuse is unproved.
    - Modal pixels are neither prepared nor restored, and compatible retry is
      unproved; the modal-presence gate only refuses MessageBox/popup disposal
      ([design](docs/subsystems/gdi-framebuffer.md#d-011-modal-backing-presence-gate-bounded-source-test-cut)).
    - The synthetic first-slice child `Exited` A→B source gate now hoists the
      tested nonmodal foreign CPU tree, but real-MIPS `Exited`-classifier reachability,
      other active exits, requested parked siblings, already-exited skips and
      generic duplicate pruning remain unproved. Foreign CPUs without saved
      context and PC resumability remain unproved. The bounded
      [parked-survivor/saved-CPU-owner gate](evidence/stage3-survivor-resume-green-20261016/validation.receipt.md)
      covers only tested natural-parked and requested-active stopped exits;
      [first-slice receipt](evidence/stage3-first-slice-survivor-green-20261016/validation.receipt.md).
      This source-audited risk is not an attributed live-app failure.
    - **Post-native child custody remains incomplete outside the connected
      holding-cell path:** at its earlier seal, the
      [test-only `Parked` publication red](evidence/stage3-parked-publication-red-20261017/validation.receipt.md)
      ran A's `CreateProcessW` and B's fixed-stack first touch, then held a
      parent parked-queue read borrow around publication; B stayed Kernel-live
      while its CPU was lost. That artificial borrow was not demonstrated
      natural reentry or mounted-app behavior. The later
      [connected gate](evidence/stage3-connected-port-20261017/validation.receipt.md)
      retains B's original CPU as `UnverifiedStopped` on tested early and late
      queue conflicts, with no abandoned unbound tree. The later
      [no-custody admission fence](evidence/stage3-no-custody-admission-20261017/validation.receipt.md)
      refuses the tested public/direct child path before admission and guest
      execution, leaving the exact launch pending for machine-owned recovery.
      It does not provide a runnable B slot, cover all exit/classifier paths,
      or establish in-session recovery. Full O-T7 locator/leaf
      migration remains open; the historical red is not a current narrow
      `Some(custody)` failure.
    - At its earlier source seal, a [real-MIPS non-live expected-red](evidence/stage3-real-mips-first-slice-20261016/validation.receipt.md)
      executed A's actual `CreateProcessW` IAT call to launch B, then returned
      A through its saved startup RA. A exited `0x63`; B remained exact-birth
      Kernel-live and unsignaled, but slice-end rotation made B the local CPU
      before A was pruned. `run_pending_process_launches` then tested A's exited
      handle, recorded `CreateProcessChildParkSkipped(already_exited)` and dropped
      that local B CPU; the parent queue had zero owners instead of one. The
      historical expected-red was reproduced before the post-native launch
      cut, not an attributed live iNavi
      or live-pump failure. The stamped current-source
      [attempt 17](evidence/stage3-createprocess-deferred-20261017/attempt17-current-first-slice-red.raw.log)
      exited 101 because A had not reached the fixture's exit-once
      precondition (at `unicorn.rs:85571` in that stamped source); it neither
      reproduces nor fixes the
      earlier survivor-loss failure. That stale ignored fixture was retired;
      later-slice survivor ownership on the current route remains unproved.
    - If the parent parked queue is already borrowed, first-slice `Exited`
      refuses before Kernel exit and the fallback returns terminal `Backend`
      without a host panic, but it drops the local A+B CPU owners. This is
      ownership loss on a borrow-invariant violation, not recoverable parking;
      Stage 4 needs a machine-owned child owner. The separate `Completed`
      path formerly used unconditional `borrow_mut`. The
      [pre-deferral offline control](evidence/stage3-completed-reentry-red-20261017/validation.receipt.md)
      reproduced a `RefCell` panic and survivor-owner loss when a custom
      framebuffer reenters that borrowed queue; current framebuffer
      implementations have not been shown to do so. Stamped current-source
      [attempt 16](evidence/stage3-createprocess-deferred-20261017/attempt16-current-completed-red.raw.log)
      exited 101 before the restore/reentry precondition (`restore_dirty_calls`
      0 rather than 1); it neither reproduces nor fixes that panic. That stale
      ignored fixture was retired. A newer
      [direct-MIPS expected red](evidence/stage3-completed-borrow-20261017/attempt01-direct-completed-expected-red.raw.log)
      reached first-slice `Completed` with an empty descendant queue and a held
      parent read borrow and reproduced the old panic. The current source
      skips that unnecessary borrow; its
      [bounded receipt](evidence/stage3-completed-borrow-20261017/validation.receipt.md)
      records focused **1/0**, 19-block release **2,162/0/11 ignored**, and
      no-default **1,532/0/3 ignored**. Nonempty descendants instead used
      fallible borrow/reservation: at that earlier seal, a conflict after
      committed child exit effects could lose descendant CPU custody. The
      connected `Parked` controls do not establish a nonempty
      `Completed`/`Exited` survivor transition. No current-route nonempty survivor or live-pump later-slice
      control closes that gap. The later bounded
      [exact-primary stopped-exit gate](evidence/stage3-primary-exit-birth-v4-20261017/validation.receipt.md)
      covers its tested child/boot birth checks, not this descendant-publication
      or later-slice CPU-custody gap.
      The [ignored native-fault fixture](evidence/stage3-post-admission-native-fault-20261016/validation.receipt.md)
      at its earlier seal dropped an admitted synthetic child's only CPU after
      `READ_UNMAPPED`, with its exact Kernel birth still live and unsignaled.
      Later historical reds [18](evidence/stage3-createprocess-deferred-20261017/attempt18-current-native-fault-red.raw.log),
      [07](evidence/stage3-completed-borrow-20261017/attempt07-current-native-fault-red.raw.log)
      and the [pre-Cut-3a rerun](evidence/stage3-cpu-custody-current-red-20261017/validation.receipt.md)
      repeated that owner-loss result. The direct-helper ignored fixture has
      since been replaced by an ordinary real-MIPS Cut 3a control, earlier
      focused debug/release green and superseded producer-side a09 debug green: it retains
      the original faulted B CPU in a
      non-runnable machine slot, not the legacy parked tree. The
      [owner-sealed Cut 3a receipt](evidence/stage3-cpu-custody-quarantine-20261017/validation.receipt.md)
      passes a11 focused release **1/0**, 19-block a12 release
      **2,184/0/9 ignored**, a13 no-default **1,543/0/2 ignored**, and a14–a16
      feature checks; all six source hashes matched. This bounded result did
      not by itself repair the nonempty descendant publication conflict above,
      other parked/active owner-loss paths, or the then-public run wrapper's
      custody bypass. The later connected gate covers its tested `Parked`
      conflicts; the later no-custody fence refuses that tested public child
      admission but does not supply runnable recovery. A
      stamped a07 expected red reproduced a synthetic post-load/pre-native-entry
      error crossing the producer's `child_admitted` flag before poisoning and
      moving an unpoisoned CPU into quarantine. The test stopped at that
      assertion; by source inspection, the exit consumer would refuse its
      poison-bit preflight. A producer-side seal made a08 debug pre-entry
      and a09 debug real-MIPS fault controls green on superseded source
      SHA-256 `1AAE8A66…33090`; intermediate producer-only a10 focused
      release passed but is superseded. Current source makes the custody
      directory itself seal an admitted CPU on `Reserved`→`Quarantined` under
      Unicorn; the consumer poison check remains a backstop. This does not
      prove natural pre-entry reachability or general error-path closure. The two
      stale route-specific controls were retired; none is a live iNavi result.
    - The historical [scheduler-block self-reentry fence](evidence/stage3-scheduler-block-fence-20261017/validation.receipt.md)
      refused a preseeded busy block service but did not exclude raw
      code/import/memory-hook aliases or prove natural nested callbacks. The
      later connected port removes the named raw aliases, but it does not
      independently authorize an exact executing leaf.
      Source order permits modal framebuffer restoration to consume a wait and
      publish pixels before a hypothetical nested fault is observed.
      Whole-attempt access, partial-effects recovery, and full runnable/parked
      CPU custody remain open despite the narrow Cut 3a fault quarantine.
    - The historical [primary-identity red](evidence/stage3-primary-claim-red-20261017/validation.receipt.md)
      is fixed for the bounded Kernel query path by the
      [canonical birth gate](evidence/stage3-primary-canonical-green-20261017/validation.receipt.md).
      The [v4 stopped-exit gate](evidence/stage3-primary-exit-birth-v4-20261017/validation.receipt.md)
      also fails closed on missing primary authority in its tested shapes.
      Still open: other launch/handle-dependent exit consumers and
      unpublished-thread rollback without an explicit publication-state bit.
    - `parked_process_handle_exit_code` drops an already-exited parked CPU
      without inspecting its modal state, and an earlier external Kernel exit
      cannot be made atomic by the later finalizer preflight.
    - Failures after preflight are not whole-exit atomic. CE6 foreign-link
      topology, direct WndProc, receiver-side cross-process result writes and
      conditional system-origin device broadcasts are not established. No
      live-app or CE6 parity follows.
  - W-B1c does not establish physical arrival order across HTTP and desktop
    producers, ordering with NMEA/serial batch dispatch, CE6 focus/activation
    parity, cross-thread guest WNDPROC execution order or O-C5 guest-time
    assignment. Host `GetMessage(NULL)` wake without a HWND is separate.
    W-B1b removed hidden/off-point hint delivery and retention of input with
    no live target. Earlier E-38 and callback-drive live tap-delivery evidence
    predates both W-B1b and W-B1c routing; the next approved live run must
    verify taps still reach the intended iNavi window.
  - Opaque non-task wait slots (queue, socket, serial tokens) and duplicated
    native wait payloads are unpinned despite exact-birth publication. Wait-ID
    wrap/reuse and live modal continuation behavior remain unproved.
    Synchronous finite inline timeouts publish no scheduler wait. Raw
    synchronous/nonblocking MsgWait,
    queue/socket and serial (including overlapped) paths have no birth guard.
  - The guest return-stub hook has no end-to-end control, and pending-return
    KData PID follows mutable process selection, not the captured lease.
  - `SendMessageTimeout` global expiry can read as ready/success; the overloaded
    `SMF_TIMEOUT` flag still needs a separate semantic gate. Receiver-side
    `DestroyWindow` result writes, cross-process callback transfer, legacy
    numeric-only HWND ownership and the broader GWE send lifecycle remain
    ungated. The former ordinary queued-send receiver-TID/exit defect is a
    historical pre-repair source red: after B exited and B-prime reused its
    TID, B-prime took B's send without A's failure result
    ([receipt](evidence/2026-10-15-ordinary-queued-send-receiver-exit-reuse-red.md)).
  - **Reciprocal Send shape remains narrow:** the A11b next-lease replay is a
    [historical source red](evidence/scheduler-source-cut-a11-reciprocal-red/validation.next-lease.receipt.md);
    A12's post-format focused controls cover one birth-owned, same-process,
    plain S→T→S pair and its exact inner/outer return. A third concurrent
    sender, timed/result-pointer Send, early result/reply, second pending
    return or deeper nesting, and cross-process pairs are not supported by
    that handoff. An ineligible
    blocked Send fails closed with terminal `Backend` instead of replaying its
    import; this is a live-run risk, not CE parity or a reproduced iNavi
    deadlock. The checked callback SP is not a complete stack-allocation
    bound proof. A12's two broad source suites passed; live behavior and
    these unsupported shapes remain separate gates.
  - Live-run risk: a second concurrent cross-thread SendMessageTimeout now
    stops with an explicit backend error until task-owned SMT frames exist
    ([receipt](evidence/stage3-smt-occupied-20261015/validation.receipt.md)).
  - Aliasless Sleep-suspend, unpinned-worker lineage (fails closed), other
    parked-wait continuation, clone relineage, complete GMS reclamation and
    live-app behavior.

- Historical: build 354AA0FB failed at 260.174 s with PC 0 on pool worker 10
  (RA `0xe6de4`). Later builds ran 330 s cleanly, so it has not been
  reproduced and its cause is unattributed. PC 0 remains a hard failure.
- O-P1 hardening (outside the classification contract): fallible-tail effect
  salvage, genuine C query-error and competing-hook controls, and state
  consistency for other ready-resume families on a native-entry failure. An
  exploratory same-handle nested-start hook probe crashed with an AV of
  unknown cause and was removed from default tests. Receipts: ledger E-28 to
  E-31.
- `HeapFree` does not reuse addresses. O-M2 prevents foreign-only heap
  free/size/validate and cross-birth cursor charges, but failed prepared
  allocations can leave birth-local monotonic high-water/OOM holes. Original
  reservation footprint and allocation incarnation are still needed before
  safe reuse; no free list, cross-process page reuse or speculative unmapping
  has landed. `ORD_REMOTE_HEAP_*` currently ignores its target argument and
  acts under the selected birth; `GetClipboardDataAlloc` sizes a global
  source handle under the current birth, so those cross-birth paths remain
  unresolved ([O-M2 receipt](evidence/om2-heap-20261014/validation.receipt.md)).
- Host footprint: E-38 (fresh default) had a sampled RSS maximum of 430.7 MiB
  across 160 host samples, against BED2 at 555.6 MiB and persistent runs at
  about 1.5 GiB (ledger E-1, E-22/E-27/E-29, E-38). The read-only
  [historical E-38 validator](scripts/analysis/e38_validator.py)
  ([receipt](evidence/stage7-historical-validator-20261016/A01.receipt.md))
  also checks 166 successful frame polls/four hash segments and 1,714
  completed busy slices; frame hashes do not prove image or map content. The
  true continuous memory peak and current-build live readiness remain unknown.
  No app run has measured E-41 to O-M2. A cold TCG cache per fresh owner is a
  possible Stage 8 cost. Translation arenas and engine copies are unmeasured;
  virtual reservation is not resident RAM.
- Gesture FIFO is fixed, but a real tap still took about 1.29 s from successful
  HTTP response to first guest wndproc delivery in the prior 330 s input test,
  and 1.62 s in the bounded-cache repeat.
  Guest DOWN/UP handlers each returned in about 23 ms; delivery lag and frame
  readiness, not merely HTTP acceptance, are the remaining response gates.
  BED2's latest one-tap run observes0.397s before DOWN (earlier0.835s), still not responsive
  navigation or a controlled latency improvement.
- Full application boot timing remains unresolved because no map/interactive
  navigation phase has presented. The latest clean 300 s run reaches device
  opens at 31-35 s and positive GPS ReadFile, but splash remains unchanged.
  BED2 first splash is observed by+4.358s; failed650 by+4.800s with per-request timestamps;
  full readiness remains unproven and must be reported separately.
- iNavi route-flow completion remains open beyond process startup, shell
  readiness, real splash rendering, remote input delivery, and initial mapdata
  loading. The former "map presentation gate" was the emulator swallowing all
  host-delivered visible messages via escaped-callout false positives (fixed
  2026-07-04, `2fcc365f`/`dbaf4791`); post-fix runs advance into
  `resmapi_800x480.bin` map-skin streaming with the splash progress bar at
  full width. The remaining frontier is the map screen actually presenting
  after skin decode. COM7/MFS1/SMB1 opens and sustained GPS reads are proven
  on the current build; keep validating the remaining presentation gate.
- `happyway_win.exe` busy-waits inside its WM_ACTIVATE handler
  (`happyway_win.exe+0x28f8..0x2a18`) polling for cross-process state; it
  coexists with iNavi only because quantum rotation is unconditional. If
  startup ever wedges with happyway active and frozen counters, check that
  rotation still reaches iNavi.
- Scheduling is timing-sensitive and bimodal across identical builds
  (2026-10-12): `happyway_win` either gets 170-300 slices (fair round robin) or
  1-7 slices (starved while iNavi runs ahead). Reproduced with and without the
  Phase 0 hook changes, so it is a property of the wall-clock scheduling
  heuristics, not of one build. Metrics that depend on it (milestone times,
  slice splits) need several runs; compare busy-slice MIPS instead.
- Count=0 remains experimental after fixing CreateProcess replay and block-start
  sampling. The fixed 120 s run still gave the helper only six slices while
  iNavi received 844. Native Unicorn timeout stops also bypass the voluntary
  block delay-slot guard; saved contexts contain PC/GPR/HI/LO, not MIPS pending
  branch state. Preservation across an asynchronous stop is not yet proven.
- The former growing 5-30 s waits were sensor initialization retries (Sleep RA
  `0x000e69c0`), corrected by valid packed samples. TimerSystem mixes boot wall time
  with a sleep-driven virtual offset; a coherent central clock and idle
  fast-forward are not implemented. Do not attribute current live waits to
  non-live host-sleep paths without matching execution evidence.
- Watch for re-created zero-byte probe artifacts on the SD image
  (`INavi\traffictest.txt`, `INavi\routetest.txt`, `mapdata\soap.bin`,
  `mapdata\searchdb\oildata.dat`). `happyway_win.exe` embeds
  `traffictest`/`routetest` strings and boots dramatically slower when the
  marker files exist, so a returning zero-byte file means an emulator path is
  still materializing files on read-only existence probes. The 2026-07-04
  no-marker run did not re-create them through the splash and mapdata phases.
  Originals are preserved in `D:\INAVI_Emulator\_zero_byte_probe_artifacts_backup`.
- Scheduler/message semantics are not complete for every reentrant wait,
  cancellation, nested send, abort-if-hung, modal-loop, and cross-thread
  result-delivery combination.
- In E-22, `happyway_win` process 67/thread 3 revisited a completed WNDPROC
  return stub 2,527 times with no pending callout. Its recorded last return
  lacks caller context, so fabricating a recovery would be unsafe. Parent
  message delivery continued, and this is not proved to cause the splash
  stall. Preserve a child `WM_ACTIVATE` dispatch/park/resume regression that
  checks the next PC is the `DispatchMessageW` continuation (local
  `evidence/e22-live-20261013T154435Z/app.stdout.raw.txt`).
- Shell notification behavior still needs integrated validation for taskbar
  rendering/routing, guest COM callback lifecycle, stale-handle ownership, and
  remove/update races.
- Loader parity is strongest in the Unicorn runtime path. Raw/non-Unicorn
  loader behavior needs explicit audit before all load variants can be treated
  as equivalent.
- Mounted storage remains synthetic for several lower layers beyond covered
  public AFS mount registration: physical block-driver forwarding, external
  cache/filter DLLs, real mounted-volume availability/powerdown, broader ACL
  persistence/enforcement, utility DLL execution, callable FMD callbacks,
  hardware-backed FLS discovery, and lower-FSD `FsIoControl` forwarding.
- GDI/text/IME fidelity is still incomplete for rendered IME UI callbacks,
  caret timing, broader font/style fallback, glyph metrics outside fixture
  ranges, complex clipping, palette/device-color behavior, alpha/mask variants,
  and remaining DIB/bitmap lifetime edges.
- File-change notification reset/error propagation still has deeper edge cases
  beyond the currently covered prefix drains, no-more-items, undersized
  buffers, null-buffer ordering, all-zero no-data reset, and fault-drain cases.
- Popup/menu/modal behavior still needs broader live nested-modal routing,
  unusual cascade cancellation, timeout edges, and user-driven dispatch
  validation.
- Sensor emulation remains partial beyond the basic live paths. GPS/NMEA now
  reaches `COM7:` in the current live run, drains into guest serial RX, and is
  returned by real `ReadFile` calls; `MFS1:`/`SMB1:` also open and receive
  sensor IOCTLs. Parsed-position consumption, broader `WaitCommEvent` cadence,
  and richer accelerometer/magnetometer sample semantics still need trace
  evidence.
- Fresh splash-phase samples still spend time in `UID1:` probing and
  resource/map loading before `COM7:` opens, so sensor data cannot explain the
  earliest visible splash interval. The current post-COM7 path reaches GPS and
  sensor devices plus map-era child windows, but final map presentation remains
  gated while the framebuffer still shows the iNavi splash artwork.
- E-29's pre-retry-fix bounded 330 s run again stayed on the splash despite
  10 retained positive guest `COM7:` reads (2,784 B) and down/up touch return
  records.
  The full-screen splash window `0x00020008` remained visible over 23 hidden,
  update-pending controls. Service counters stopped changing by about +120 s
  while sampled host CPU kept increasing; the previous E-27 run also reached
  a service-counter plateau. A blocked TID 10 wait has one signaled component,
  but its any/all mode is absent from the trace. Neither CPU spin, missed wake,
  splash callback failure, nor loader failure is established; trace these
  identities/causes before altering policy (ledger E-29).
- Startup profiling still reaches real `happyway_win.exe` table/traffic
  preparation loops, but the scheduler no longer leaves an idle visible helper
  process ahead of a runnable visible parked UI process outside the live-pump
  branch. The remaining delay is not explained by unreadable MessageBox UI,
  remote input routing, or pre-COM7 sensor queuing.

## Build And Validation Risks

- The normal validation profile uses `--features unicorn,trace,win32-desktop`.
  No-feature `cargo test -j 1` is now a supported smoke profile for stub/raw
  runtime paths, but it is not a substitute for the Unicorn feature gate.
- `registry.reg` loading accepts REGEDIT text through UTF-8/lossy decoding and
  decodes typed `hex(2)`/`hex(7)` values as UTF-16. Non-UTF-8 ANSI quoted
  strings from future REGEDIT4 exports may need code-page-aware decoding.
- `git diff --check` may report CRLF warnings on existing files. Treat new
  non-CRLF whitespace findings as actionable.
