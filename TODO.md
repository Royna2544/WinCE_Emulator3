# TODO

Refreshed on 2026-10-17. This file is limited to active, plan-aligned work.

ACK to Claude (supervisor), 2026-10-17 native timeout steer: the bounded
[opt-in timer-boundary control](evidence/native-timer-rendezvous-20261016/validation.receipt.md)
passed 48/48 fresh-owner iterations with stamped, NUL-free logs. The C probe
is diagnostic only; the earlier full-suite AVs remain unattributed. On Windows
`restore_jit_state` is a no-op (`vendor/unicorn-engine-sys/uc.c:38-74`), so
concurrent JIT/W^X restoration is not the demonstrated mechanism. Keep the
production timeout and `uc.c` unchanged without a separate user decision;
timeout zero would remove the default 1-second cap and change
`Query::TIMEOUT`/O-P1/E-34 classification. The Claude bridge now accepts
supervision messages; older unconfirmed-send notes below are historical.

ACK to Claude (supervisor), 2026-10-16 05:00 KST WINDOWPOS steer: the
AV-versus-stack-overflow lead remains unconfirmed. After the file-view broad
seal, three separately stamped default-harness release `--lib` trials passed
without a WINDOWPOS crash (**0/3**); the earlier full-suite default-harness
sequence had **1/3** crash. These are different denominators and do not
establish cause or safety; a later VM-allocation full release run also passed
on changed source
([library reruns](evidence/stage2-windowpos-stack-20261016/lib-rate.validation.receipt.md),
[stack probe](evidence/stage2-windowpos-stack-20261016/validation.receipt.md),
[latest full suite](evidence/stage2-writable-image-vm-allocation-20261016/validation.receipt.md)).
The direct Claude bridge ACK is still queued behind an earlier
`sent_unconfirmed` message, so this is the in-repo ACK without bypassing it.

ACK to Claude (supervisor), 2026-10-16 Stage 2 sequencing: the writable-main-
image red and mapped-code R02 now have bounded synthetic green controls. The
later code-read fallback guard also passed focused and complete source suites
([final broad receipt](evidence/stage2-mutable-code-fallback-20261016/broad-validation.receipt.md));
R01 was compile-only and never a behavioral red. No third unpaired red was
opened. Claude also identified an ignored-test tally convention error in the
final broad receipt; its corrected overall totals are 2,116/0/9 release and
1,515/0/3 no-default, without changing raw logs. A bridge ACK remains pending
because an earlier message is `sent_unconfirmed`; this records the status
without bypassing the queue.

ACK to Claude (supervisor), 2026-10-16 decoded-exit seal audit: the broad
receipt now discloses missing raw-log command/exit stamps, and
`head-at-seal.txt` plus `source-at-seal.patch` preserve hashed, reverse-checked
source provenance. E-38 has no affirmative production decoded-PC0-exit marker;
that is not proof the path never ran. The bridge refused a new reply while an
earlier send remained unconfirmed, so this records the ACK without bypassing
its queue.

ACK to Claude (supervisor): the A13 exact-modal-wait selector finding was
accepted and covered in the bounded
[A13 source/test gate](evidence/scheduler-source-cut-a13/validation.final.receipt.md).
The bridge still has an earlier unconfirmed message, so this records the ACK
without bypassing that delivery queue.

ACK to Claude (supervisor), 2026-10-15 19:57 KST steer: A13 seal verification
received. The reused-PID host-teardown expected-red was repaired in a bounded
gate and its test enabled before opening another red. Every new validation
attempt's raw log is named in the
[receipt](evidence/stage3-host-teardown-20261015/validation.receipt.md). The
unused numeric backing-store selector was removed and the legacy no-birth
teardown branch now fails closed. The bridge's earlier delivery remains
unconfirmed, so this ACK is recorded here without bypassing its queue.

## Immediate engineering queue

History and suite tallies live in `docs/evidence/ledger.md`; open defects are in
`KNOWN_BUGS.md`. Delete an item when it lands; do not annotate it.

0. Complete writable-image production safety gates before enabling adoption
   for the SD app. VM backing, opt-in real-MIPS fresh-owner behavior,
   mapped-code fallback, the synthetic second-run native `mem_map_ptr`
   failure/abort-close control, the legacy file-view seed/readback fence,
   and the VM-allocation/heap/native-mapper collision fences pass their
   bounded source gates
   ([native failure](evidence/stage2-writable-image-native-failure-20261016/validation.receipt.md),
   [file-view broad](evidence/stage2-writable-image-file-view-20261016/broad-validation.receipt.md),
   [VM allocation](evidence/stage2-writable-image-vm-allocation-20261016/validation.receipt.md));
   the normal app loader is still copy-backed. Next, fence other direct
   host-write/restore/rebind paths and exact module-retirement races with
   synthetic controls. The user chose to keep production images on this
   copy-backed path until O-M3 accounting is known; only then migrate the
   main EXE, preloaded DLLs and runtime DLLs as separate exact-birth/
   module-generation cuts with failure/unload tests.
   Settle [O-M3](docs/DECISIONS.md) per-page physical provenance and
   `PrivatePageBudget` timing before guest-budget or live acceptance; do not
   treat the synthetic `Unclassified`/zero-charge path as that decision.
   Matched iNavi timing requires a separate run approval and fresh adjacent
   mount backups.
   Diagnose the intermittent native `0xc0000005` seen in two different
   default-harness full-suite tests without assuming one cause or test-only
   scope. The requested 128-KiB isolated WINDOWPOS stack passed and 64 KiB
   overflowed; a later matched release-library cohort observed **0/3** crashes
   with serial harness threads and **0/3** with default harness threads on one
   stable source seal. Neither that non-reproduction nor the mixed-deadline
   isolated pass/full retry is a fix. Audit process-global state, `mem_map_ptr`
   pin lifetime and native timeout-worker interactions; do not inflate the
   test stack as a substitute for root cause
   ([cohort](evidence/parallel-native-av-cohort-20261016/validation.receipt.md),
   [WINDOWPOS probe](evidence/stage2-windowpos-stack-20261016/validation.receipt.md),
   [exit retry](evidence/stage2-writable-image-exit-retire-20261016/validation.receipt.md)).
1. Continue the user-authorized Stage 3 migration. Passed bounded gates are
   listed in `PROGRESS.md` ("2026-10-16 and 2026-10-15 Stage 3 source gates"); open defects
   are in `KNOWN_BUGS.md`. The [test-only real-MIPS A→B→C observation](evidence/stage3-post-child-selection-observation-20261017/validation.receipt.md)
   shows C selected and occupying the returned child CPU slot at B's
   post-child boundary. Do not add a B-only post-restore guard or quarantine
   that C payload under B's reservation; carry checked selection restoration
   and the rotated subtree into O-T7's slot/leaf design. The follow-on
   [enabled CPU census](evidence/stage3-nested-cpu-census-20261017/validation.receipt.md)
   reproduced exact-owner `NoAttempt` non-progress with all three CPUs
   retained at its earlier seal. The later
   [connected cut](evidence/stage3-connected-port-20261017/validation.receipt.md)
   retained the mismatched C-root/B-parked tree as stopped at its seal. A
   newer [bounded witnessed two-node gate](evidence/stage3-rotated-handoff-20261017/validation.receipt.md)
   publishes B and C under their own legacy-queue labels and executes later
   A/B/C slices. Its final-source release/no-default 19-block suites passed
   **2,231/0/9 ignored** and **1,564/0/2 ignored**; both feature checks exited 0.
   Its [a09 selection observation](evidence/stage3-rotated-handoff-20261017/a09-selection-observation-green.raw.log)
   ends with ordinary outer rotation selecting B after exact A restoration;
   it does not leave A finally selected. The
   [a21 final-source no-witness control](evidence/stage3-rotated-handoff-20261017/a21-final-no-witness.raw.log)
   still retains the whole nested E→D tree as `UnverifiedStopped`, with
   untouched outer siblings queued. This is bounded legacy-queue progress,
   not a runnable machine slot or in-session recovery. An inherited B
   directory is checked against the saved A directory; later guest mutation
   can fail closed and remains a general handoff limit. Next, in
   order:
   - Complete exit-effect authority beyond exact-birth window/backing selection,
     sender cleanup and the bounded
     [exact-window-set gate](evidence/stage3-host-exit-exact-window-20261015/validation.receipt.md)
     and [pseudo-caller gate](evidence/stage3-pseudo-caller-birth-20261015/validation.receipt.md):
     [D-011's bounded framebuffer gate](evidence/stage3-host-exit-framebuffer-preflight-20261015/validation.receipt.md)
     now stops before mutation when a saved backing cannot restore. The bounded
     [native pseudo self-exit gate](evidence/stage3-native-pseudo-self-exit-20261016/validation.receipt.md)
     source-tested CPU stop, native owner closure and exact child/boot Task/VM
     completion after preflight. The bounded
     [decoded pseudo self-exit gate](evidence/stage3-raw-decoded-pseudo-exit-20261016/decoded-pseudo-exit.receipt.md)
     now requires an attempt-local executed JALR/delay-slot witness and exact
     stopped caller/owner before that same finalizer. Raw direct pseudo
     compatibility remains fixture-only and fail-closed; no raw executor is
     proposed. The bounded
     [modal-presence refusal gate](evidence/stage3-modal-exit-failclosed-20261016/validation.receipt.md)
     now fails closed for affected MessageBox/popup state in tested stopped
     exit shapes. Next add checked modal-backing restore preparation
     ([design](docs/subsystems/gdi-framebuffer.md#d-011-modal-backing-presence-gate-bounded-source-test-cut))
     and handle failures
     after successful preflight without claiming whole-exit
     atomicity, and settle foreign-link topology under O-G4.
     The bounded [parked-survivor/saved-CPU-owner gate](evidence/stage3-survivor-resume-green-20261016/validation.receipt.md)
     now preserves tested nonmodal foreign CPU descendants on natural parked
     and requested-active exits, and refuses a foreign saved context without
     its matching execution-live current thread birth before exit effects.
     The bounded [first-slice `Exited` source gate](evidence/stage3-first-slice-survivor-green-20261016/validation.receipt.md)
     adds synthetic production-helper A→B preservation and a terminal
     held-parent-queue-borrow control; it does not execute the guest classifier.
     Continue the [machine-owned custody design](docs/subsystems/tasks-scheduler.md#next-custody-and-reachability-design-gate-proposed-not-implemented)
     and [O-T7](docs/DECISIONS.md) consumer inventory: a
     [test-only direct-MIPS held-queue expected red](evidence/stage3-parked-publication-red-20261017/validation.receipt.md)
     reached post-native B `Parked` publication at its earlier seal and showed
     a typed queue conflict dropping its only CPU owner while B remained
     Kernel-live and unsignaled. The later connected `Some(custody)` early and
     late conflict controls retain B's CPU. The later
     [no-custody admission fence](evidence/stage3-no-custody-admission-20261017/validation.receipt.md)
     stops the tested public/direct path before guest child execution, leaving
     its exact launch pending for machine-owned recovery. Other publication
     and runnable custody paths remain open. The artificial read borrow is
     not natural reentry or a live-pump result. The earlier ordinary
     post-admission first-slice error was separately narrowed by Cut 3a
     quarantine. At its earlier source seal, the
     [real-MIPS non-live expected-red](evidence/stage3-real-mips-first-slice-20261016/validation.receipt.md)
     proved a third route: after A's guest launch and exit, the
     already-exited-A skip dropped live B's CPU. Retain that receipt as
     historical evidence; its ignored fixture was retired after it missed
     the post-native deferred route. A separate later-slice survivor control
     still needs a reachable exact-owner fixture. The
     [native-fault expected-red](evidence/stage3-post-admission-native-fault-20261016/validation.receipt.md)
     that proved an ordinary first-slice error left a still-live child
     without CPU custody is historical. An ordinary real-MIPS Cut 3a
     replacement passed focused debug/release controls at an earlier seal; its
     superseded producer-side a09 debug rerun passed; its final owner-sealed
     broad gate is recorded below. The
     [bounded pre-CPU pending-launch retention gate](evidence/stage3-pre-cpu-launch-retention-green-20261016/validation.receipt.md)
     preserves published launches only until CPU construction; it does not
     own an admitted child through DLL load, native execution or error.
     The [primary-identity red](evidence/stage3-primary-claim-red-20261017/validation.receipt.md)
     is now covered by the [canonical-primary query gate](evidence/stage3-primary-canonical-green-20261017/validation.receipt.md).
     The later bounded [exact-primary stopped-exit gate](evidence/stage3-primary-exit-birth-v4-20261017/validation.receipt.md)
     now verifies the canonical birth for its tested child/boot exit shapes;
     the v4 broad release and no-default suites pass. Next prove unpublished
     rollback/publication state and the remaining exit consumers without
     promoting a recycled numeric TID or an old launch/handle witness. This
     remains separate from O-T7 admitted-CPU custody and general Task/Frame
     authority.
     The [bounded live-checkpoint guard](evidence/stage3-checkpoint-orphan-frame-green-20261016/validation.receipt.md)
     refuses retained Task/launch and CPU custody on save and restore; it is
     a safety stop, not atomic relineage. Real monitor rewind is unavailable
     and D-011 refusal currently needs repair/restart. Design the whole-machine
     relineage/recovery transaction before restoring that feature.
     Its seven formerly ignored controls are now ordinary counted tests in
     [both full suites](evidence/stage3-checkpoint-unignored-20261017/validation.receipt.md);
     keep the strict fence while CPU custody is the priority. The
     [three pre-deferral source reds](evidence/stage3-completed-reentry-red-20261017/validation.receipt.md)
     reproduced `Completed` callback borrow/panic, already-exited-A survivor
     loss, and admitted-child native-fault loss offline. Stamped final-source
     ignored reruns [16](evidence/stage3-createprocess-deferred-20261017/attempt16-current-completed-red.raw.log)
     and [17](evidence/stage3-createprocess-deferred-20261017/attempt17-current-first-slice-red.raw.log)
     exited 101 at unmet fixture preconditions under post-native deferred
     launch, not at the old failure sites: they neither reproduce nor fix
     those two failures. Their ignored fixture implementations were retired;
     the historical logs remain. A new direct first-slice MIPS control
     reaches `Completed` with an empty descendant queue and a held parent
     borrow ([red attempt](evidence/stage3-completed-borrow-20261017/attempt01-direct-completed-expected-red.raw.log));
     the current source skips that unnecessary borrow. Its
     [bounded receipt](evidence/stage3-completed-borrow-20261017/validation.receipt.md)
     seals focused **1/0**, 19-block release **2,162/0/11 ignored**, and
     no-default **1,532/0/3 ignored**. Next separately test nonempty
     descendant publication, later-slice A→B survival and live-pump
     reachability. The nonempty `Completed`/`Exited` queue conflict belongs to
     an earlier seal and remains unproven on the connected route. The bounded
     connected `Some(custody)` **Parked** early/late conflict controls retain
     B's CPU as non-runnable, not as a recoverable execution slot.
     [Attempt 18](evidence/stage3-createprocess-deferred-20261017/attempt18-current-native-fault-red.raw.log)
     first showed the admitted child's ordinary native fault losing its sole
     CPU owner. [Final-source attempt 07](evidence/stage3-completed-borrow-20261017/attempt07-current-native-fault-red.raw.log)
     still exited 101 at that earlier assertion. The ignored direct-helper
     fixture is now replaced by an ordinary real-MIPS Cut 3a control; do not
     treat the earlier red as the current narrow producer result. The
     [latest current-source rerun](evidence/stage3-cpu-custody-current-red-20261017/validation.receipt.md)
     again failed at zero parked child CPU owners after the canonical-primary
     query fix at its earlier seal; it is not a mounted-app result.
     The current source stages a successful `CreateProcessW` parent return PC
     and defers the child launch until after native return in both modes,
     removing the former immediate in-hook child call. The bounded
     [source/test gate](evidence/stage3-createprocess-deferred-20261017/validation.receipt.md)
     passed focused **5/0**, two-mode file-view reseed **1/0**, and 19-block
     release-feature **2,161/0/13 ignored** and no-default **1,532/0/3
     ignored** suites. The file-view control did not check guest-visible
     post-lease reads. This cut does not supply CPU slots, a callback-scoped
     machine-state port or Stage 5 fairness.
     A [test-only live-pump fixed-stack witness](evidence/stage3-live-pump-fixed-stack-hook-20261017/validation.receipt.md)
     now passes real-MIPS first-touch `SW`/`LW` in explicit-PC/global-hook and
     default hookless/count-limited modes (focused **2/0**, release
     **2,164/0/11 ignored**, no-default **1,532/0/3 ignored**). It attributes
     the page commit to the production memory-fault route by source
     exclusivity, not an independent callback counter. At that earlier seal,
     an isolated fault-hook `RefCell` wrapper could not be sound while the
     native call retained `&mut CeKernel` and the other raw hooks bypassed it.
     The later [connected port source/test gate](evidence/stage3-connected-port-20261017/validation.receipt.md)
     removes named raw aliases across native setup/tail, all hook families
     and the recursive child path, with short fallible loans and no loan
     across guest reentry. It still lacks exact thread/leaf authority,
     checked post-B restoration and an exit-event fence; it is not full Step 2
     or O-T7 acceptance. The
     [historical pre-port Step 2 inventory](docs/PLAN.md) counted 112
     Kernel and nine framebuffer named-alias use-lines on the sealed
     `FB855EEB…92E381` source, plus the live host, monitor, child, exit and
     presentation boundaries. The earlier [pointer delta audit](evidence/stage3-machine-port-inventory-20261017/findings.md)
     belongs to its older source and includes a then-native-fault CPU-drop
     path; both are historical inventory, not the connected implementation.
     The [bounded scheduler-block fence](evidence/stage3-scheduler-block-fence-20261017/validation.receipt.md)
     routes that callback's 19 Kernel and two framebuffer dereferences
     through one guarded raw adapter and rejects a preseeded busy service
     point. Its fresh per-attempt flag is shared by no other hook family.
     The earlier source had one native entry, sequential import/block hooks
     and deferred child execution; there was no natural overlap to justify
     a synthetic shared-flag gate. The subsequent
     [native-entry precondition](evidence/stage3-native-entry-precondition-20261017/validation.receipt.md)
     refuses a second same-host-thread native entry before private CPU setup;
     its forced-busy control retries on the same CPU, and sealed broad suites
     pass. It is neither natural reentry nor a machine-owned leaf permit.
     Continue to audit host-callback lifetime, selected-birth/leaf authority,
     modal partial effects, A-to-B handoff and full runnable CPU custody;
     the old busy flag and entry guard alone were not the connected port.
     The preparatory crate-private selected-context stamp pairs canonical
     birth with a checked generation. Focused no-default and release attempts
     each passed **5/0**; same-source broad release passed 19 blocks
     **2,189/0/10 ignored** ([a01](evidence/stage3-selector-context-stamp-20261017/a01-focused-no-default.raw.log),
     [a02](evidence/stage3-selector-context-stamp-20261017/a02-focused-release.raw.log),
     [a03](evidence/stage3-selector-context-stamp-20261017/a03-broad-release.raw.log)).
     Broad no-default passed 19 blocks **1,548/0/2 ignored** and three
     feature checks exited 0 on unchanged source
     ([receipt](evidence/stage3-selector-context-stamp-20261017/validation.receipt.md)).
     The [bounded pre-admission gate](evidence/stage3-deferred-context-preflight-20261017/validation.receipt.md)
     consumed that stamp, at its earlier seal, before
     the then-aggregate pending-launch drain: it checks A's selected
     context, exact live parent/caller and nonzero staged PC. Its test-only
     real-MIPS A `CreateProcessW` plus older raw `CeEventHasOccurred` launch
     is red-to-green on a changed selected generation: typed refusal leaves
     both queued births in FIFO order. In the tested canonical/private A path,
     its saved continuation is poisoned and requires recovery, not rollback
     or resume; aliasless synthetic nonprivate attempts do not prove that
     poison or an exact `failure_operation` record. Final-source a13 focused
     `deferred_` passed **17/0**, a14 no-default **1,548/0/2 ignored** and a15
     release **2,192/0/9 ignored** over 19 blocks each; a16 parent-resume
     passed **1/0** and three feature checks exited 0. Unchanged-context
     mixed and existing non-live/both-mode controls pass. At that seal the
     mixed control intentionally confirmed both launches admitted together;
     creator attribution was not checked. Raw `CreateProcessW` has already
     queued B before the post-native caller-birth check. The later bounded
     [P0 in-place queue gate](evidence/stage3-pending-queue-p0-20261017/validation.receipt.md)
     replaces the local take/drain with exact-birth claims and same-slot
     pre-CPU return while preserving this E+B admission order. Its runner
     observers see unattempted siblings after claim and post-admission, just
     before the native slice, and a nested real-MIPS control preserves outer
     sibling order; none is a live run.
     The bounded [P1 Event-service gate](evidence/stage3-event-trigger-p1-20261017/validation.receipt.md)
     now offers one exact Event birth per neutral host-loop opportunity without
     waiting for another `CreateProcessW`; a04/a05/a06/a07/a08/a10 controls
     pass. Final-source 19-block a11 no-default **1,558/0/2 ignored** and a12
     release-feature **2,210/0/9 ignored** suites passed; a13–a15 feature
     checks exited 0. A09 was a corrected expected-error test-harness
     assertion, not a production failure. Those were the separate
     debugger-stop, monitor and live-app limits at P1's seal. The later
     [combined monitor/P2 gate](evidence/stage3-launch-handoff-p2-combined-20261017/validation.receipt.md)
     carries an attempt-local producer witness, converts the mixed control to
     B-only and adds the original direct-`.exe` Shell trigger. Continue with
     other Shell routes, checked post-B parent restoration and the connected
     callback-scoped port; do not infer `NFY0:` timing or live-app acceptance.
     The sealed [raw `CreateProcessW` output-Boolean gate](evidence/ce6-createprocess-output-contract-20261017/validation.receipt.md)
     passed A03 raw-kernel **135/0**, 19-block A04 no-default **1,550/0/2
     ignored**, A05 release-feature **2,194/0/9 ignored**, and a06–a08 feature
     checks exit 0. The separate [sealed Shell output gate](evidence/stage3-shell-createprocess-handoff-20261017/validation.receipt.md)
     prevents a failed direct-EXE `hInstApp` or requested `hProcess` copy from
     reporting `FALSE` after queueing a live child: focused **2/0**,
     raw-kernel **137/0**, 19-block no-default **1,552/0/2 ignored** and
     release **2,196/0/9 ignored**, with three feature checks exiting 0.
     At that seal it did not admit that child, prove bad-pointer CE parity or
     provide a Shell trigger; P1 and the later bounded P2 direct-`.exe` cut
     are separate gates.
     The bounded [REA/Shell finding](evidence/ce6-rea-shell-trigger-20261017/findings.md)
     supports one dumped-COREDLL `ShellExecuteEx`→`CreateProcessW` branch and
     a `CeEventHasOccurred`→`DeviceIoControl` boundary routed through `NFY0:`
     in local CE6 source; it does not establish notification launch timing.
     The later P2 source implements only the witnessed original direct-`.exe`
     route. Audit
     lifetime/retirement
     of process/thread aliases created when optional output is null or unwritable,
     and preserve the separate `TRUE`-before-loader gap. Output-copy failure
     is not permission to roll back an already-published child. The shim's
     success-path `LastError=0` remains a known CE6 mismatch outside this gate.
     The [connected port and custody gate](evidence/stage3-connected-port-20261017/validation.receipt.md)
     passes its bounded historical source/test seal. The later
     [witnessed two-node seal](evidence/stage3-rotated-handoff-20261017/validation.receipt.md)
     checks A restoration before publishing exact
     B/C payloads, but other post-child routes remain unproved. Next
     authenticate the final executing thread and exact leaf permit (including
     handoff), generalize checked restoration and add exit-event fencing. The bounded
     [public no-custody fence](evidence/stage3-no-custody-admission-20261017/validation.receipt.md)
     now refuses the tested direct admission bypass before guest child
     execution; the public parent is poisoned and the exact launch stays
     pending for machine-owned recovery, not proven same-session retry.
     Convert retained unverified and legacy runnable/parked trees into
     checked executable slots with exact locator migration, then prove
     later-slice progress beyond the witnessed two-node case and
     recovery/relineage. The historical parked-publication red is preserved;
     current tested `Some(custody)` conflicts retain B but do not provide a
     runnable hold or close every child/exit path. Public backing-object
     mutation and cloned Kernel lineage remain open.
     The bounded [Cut 3a](docs/subsystems/tasks-scheduler.md#cut-3a-quarantine-resource-lifetime-bounded-source-test-gate)
     [receipt](evidence/stage3-cpu-custody-quarantine-20261017/validation.receipt.md)
     seals one source: a11 focused release **1/0**, 19-block a12 release
     **2,184/0/9 ignored**, a13 no-default **1,543/0/2 ignored**, and three
     feature checks exit 0. All six source hashes match. Its real-MIPS
     control now executes A's `CreateProcessW` and B's fixed-stack first touch
     plus native fault; the original B CPU stays in a non-runnable exact-birth
     slot instead of being dropped or parked in the legacy tree. A's staged
     return survives native sync/finish. The main run loop owns the slot,
     status/debug show its identity and retained mapped-blob bytes (not RSS),
     and checkpoint save/restore refuse it. A `TerminateProcess` request
     leaves B live; same-Kernel accepted exit retires its slot and fixed stack
     in the tested native-fault shape. Stamped a07 expected red reproduced a
     synthetic post-load/pre-native-entry unpoisoned-quarantine failure;
     producer-side sealing passed a08 debug pre-entry and a09 debug real-MIPS
     fault on one superseded `unicorn.rs` SHA-256 `1AAE8A66…33090`. The
     intermediate producer-only a10 focused release passed but is superseded.
     Current source makes `CpuCustody::quarantine_reserved` seal the CPU under
     Unicorn on `Reserved`→`Quarantined`; the exit consumer keeps its poison
     backstop. Natural reachability and general error-path closure remain
     unproved. The pre-seal a03 broad **2,183/0/9 ignored** remains
     development-only. Next migrate runnable/parked CPU families without
     bypassing the directory. No general
     private VM/file-view exit cleanup or runnable leaf permit follows.
     Separately, test and repair the
     [process-exit VM/view leak](KNOWN_BUGS.md#open-issues) with exact-birth
     cleanup, including legacy PID-only view ambiguity; do not fold it into
     3a or mistake slot retirement for Kernel allocation retirement.
     Audit the five remaining unchecked native `V0` return writes in the
     runtime loader, MessageBox and popup-menu paths (`unicorn.rs:13324`,
     `:23483`, `:23494`, `:23744`, `:23754`). After the new checked import
     return-write guard, separate its error slot/report operation from the
     generic `immediate_process_sync_error` and `synchronize native process
     mapping` label (`unicorn.rs:15092`); an import-register failure must not
     be reported as a mapping-sync failure.
     Keep the original direct-helper red as historical evidence. The new
     Parked-publication red is an ownership contract control, not permission
     for a queue-only production patch. The bounded connected port and
     `Some(custody)` gate retain B's original CPU in tested early/late
     conflicts, but only as stopped custody. Continue full runnable/parked
     slot and exact leaf-permit migration.
     Stage further expected reds before replacing parked Box ownership with
     checked locators; do not add another pending owner vector. A controlled
     `Exited`-classifier fixture and separate live-pump deferred-launch/
     later-exit fixture remain.
     Extend the [exact-birth survivor design](docs/subsystems/tasks-scheduler.md#exact-birth-parked-cpu-survivors-on-process-exit-design-and-bounded-cuts)
     to other active exits, requested parked siblings,
     already-exited skips and generic duplicate pruning. Prove foreign CPUs
     without saved context and PC resumability separately; no ROM kernel exit
     body was identified by the [CE6/REA check](evidence/ce6-rea-exit-20261016/findings.md),
     and this is not live-app proof.
     Give the already-exited parked-CPU cleanup branch a typed retention or
     quarantine rule for residual modal state; it has no earlier exit to
     preflight at that point.
     Direct WndProc paths, receiver-side cross-process result writes and
     conditional system-origin device-change broadcasts remain separate.
     Full exit capture retirement and implicit gesture ownership need their
     own gates; so do HWND-timer producer ownership, hotkeys, sent/paint/modal
     and other numeric GWE producers.
   - Cross-process parked-sender restore, reciprocal shapes beyond A12's exact
     two-send case, legacy numeric-only HWNDs, post-wake MsgWait/sent
     processing and `SMF_TIMEOUT` last-error semantics remain separate gates,
     not reproduced live-app failures.
   - Native continuation payload ownership, task-owned concurrent SMT frames,
     the SMT global expiry-as-ready ambiguity and receiver-side
     `DestroyWindow` result writes.
   - CE caller wait/timeout behavior, a safe old-MIPS direct-kernel
     real-handle continuation and root-process real-handle completion.
   - Unpinned intra-slice worker lineage, aliasless Sleep-suspend and generic
     parked-wait continuation.
   - An end-to-end return-stub hook control; pending-return KData PID from the
     captured lease, not mutable process selection.
   - Raw synchronous/nonblocking queue/socket/MsgWait and serial (including
     overlapped) paths, opaque non-task pins, synchronous finite inline
     timeouts and native wait payload lifetimes, all before claiming caller
     wait parity.
   - Wait-ID wrap/reuse and live modal behavior.
   - Migrate the remaining fixture calls to the legacy `register_blocked_waiter`
     helper, then make that entry test-support only.
   The historical E-8/E-11/E-26 reds stay the acceptance baseline. Full Stage 3
   and live behavior remain unaccepted.

## Parallel lanes (never gate the queue above)

- E-43's sealed main-image, E-44's fixed initial heap/stack, and O-M2's
  birth-qualified heap **source** gates passed. O-M2 includes an exact stale-
  birth serial/NMEA control; full host-write inventory, memory-mode checkpoint
  relineage, pin-budget treatment and live app behavior remain open
  ([E-43 receipt](evidence/sealed-main-image-20261014/validation.receipt.md),
  [E-44 receipt](evidence/heap-stack-20261014/validation.receipt.md),
  [O-M2 receipt](evidence/om2-heap-20261014/validation.receipt.md)).
- If a later approved live run reaches a direct-import cold-stack bypass,
  consider per-thread/slot 128-KiB committed low-water tracked by slot, with
  contiguous `[page(SP), slot_top)` commit. The current shared sparse
  `ThreadStack` has no per-slot low-water state; do not add this speculatively.
- Splash stall (app evidence lane): run the E-29 post-120 s trace as one
  instrumentation change plus one run using `NativeAttemptReport`
  (`docs/subsystems/native-attempt-observability.md`). Record the
  any/all mode of the TID 10 wait; wait id 833 needs a boot-forward trace. Each
  writable-mount live run needs fresh user approval and a verified backup
  (`scripts/backup_rw_mounts.ps1`).
- Live run: none approved. a02 and a04 are historical and not live-usable;
  the user chose the scheduler cut over a04
  (`call_49d1b609465a4e668da30a31361beb44`). Approval
  `call_9b35e9d31fdf402ea082a3eb6a40ae16` was withdrawn after an SD-only
  backup failure, and that ZIP must not be reused
  ([disposition](evidence/pre-live-mount-backup-20261015T025500Z/aborted-approval-disposition.md)).
  Keep the incomplete archive until the user decides on disposal. History:
  [a04 seal](evidence/stage3-live-evidence-20261015/prelaunch-repair-a04-source-seal.json),
  [offline gate](evidence/stage3-live-evidence-20261015/prelaunch-repair-a04-offline-gate.raw.txt)
  and PROGRESS.md. The [v2 offline preflight](evidence/stage3-preflight-v2-review-20261017/validation.receipt.md)
  is source-tested, but no fresh output-hash-stamped rebuild exists yet. A
  future run needs, in order:
  - a fresh, distinct approval whose chat citation is checked;
  - a new SHA-verified adjacent backup triplet, with the writable folders
    quiescent and the inventory validated;
  - a freshly source/build-sealed EXE/PDB (llvm libclang) and the
    `192.168.0.39:8765` endpoint.

  The a04 watchdog (independent worker, 355 s deadline) is the base for that
  rebuild. Report input ACK, drain/post, guest WNDPROC and frame separately,
  plus phase-aligned busy-slice MIPS, per-slice setup cost, host RSS/private
  memory and the narrow guest page budget. Source, backup and terminal
  evidence and any monitor failure are separate acceptance gates.
- Host idle polling must not decide from debug snapshots. Derive it from
  canonical wait state (`docs/evidence/stale-snapshot-host-routing-design-20261014.md`).
- Before claiming exact input-to-visual lag, implement the proposed
  lineage-checked ingress→guest-work→frame-commit gate in
  `docs/subsystems/remote-observability.md`; until then report poll brackets,
  not a causal response time.
- Child `WM_ACTIVATE` stale-return stub (A-6): the controls pass (ledger E-27).
  Any recovery change needs identity-matched live evidence.
- Stage 8 native lane is paused. The E-33 commit-tail clamp landed as the D-009
  exception; do not expand it.

## Older investigation items

- Follow the second startup initializer at `0x31970` (return `0x2be44`) with the
  exact-stop debugger after authentic callback restoration.
- Revalidate the native GetMessage/sent/SetWindowPos boundary.
- Bind the live MDC object and stack chain through method `0x12e15c`, engine
  return `0x12e180` and pool return `0xe6da4`.
- Before any heap address reuse, prove the original reservation footprint,
  checked allocation incarnation and stale-pointer rejection. O-M2 retains
  birth-local monotonic holes rather than reclaiming them.
- Profile boot, frame and input latency plus host memory, phase-aligned. Reduce
  tap-delivery lag using queue and slice evidence.
- Continue the live launch loop past the `resmapi_800x480.bin` map-skin decode
  until the map screen presents.
- Throughput, after Stages 5-6: a time-based quantum replacing the 1M
  instruction count, then virtual time with idle fast-forward. Always report
  per-slice guest MIPS (`ce.slice_profile`).
- Launch wrappers must capture the process exit code and both streams (the
  2026-07-04 13:22:22Z silent desktop-run exit is still unexplained).
- Watch the SD image for re-created zero-byte probe artifacts on long runs.
- Re-measure the map-skin decode: hot PCs `0x00339c00-0x00339da8` are pixel
  conversion, not a readiness wait.
- Broaden coverage where CE source or raw tests justify it: SendMessageTimeout
  and reentrant message waits, Shell_NotifyIcon/SHNotification integration,
  lower mounted-storage layers, and GDI/text/IME parity.

## Next

- Add raw/non-Unicorn loader fixture variants for loader paths where Unicorn
  runtime coverage is currently stronger.
- Expand PE icon/resource tests for uncommon formats, malformed resource tables,
  callback-selected `KernExtractIcons`, non-PE fallback edges, and cleanup
  lifetimes.
- Continue file-change notification reset/error propagation and mounted rename
  edge coverage where current behavior is still only partially proven.
- Prove device-notification consumer ordering and wake cadence in live startup
  scenarios.
- Replace the synthetic `GetCallStackSnapshot` frame with real saved-context
  unwinding when guest diagnostics require deeper stack fidelity.
- Decode the SMB380/G-sensor initialization contract from actual dump code or
  real caller traces before adding accelerometer command behavior.
- Continue live device traces after `COM7:`, `MFS1:`, and `SMB1:` are open.
  GPS now drains and sensor IOCTLs are visible; keep validating parsed-position
  consumption and the remaining presentation gate with real traces.
- Re-run the live startup after the device-information ordinal fix and confirm
  whether guest device probing now reaches `COM7:` earlier or still remains
  blocked behind the splash/readiness path.
- Re-run the live startup after the device-search ordinal fix and confirm
  whether CE Device Manager enumeration advances any GPS/storage probing paths.

## Cleanup Queue

- Clear the 21 release-build warnings in one change, between gates. The
  inventory is in `evidence/stage3-thread-timer-birth-20261015/build-a01.raw.log`.
  Dead code: `ArchHooks`/`ArmArch`/`ArmGuestContext`/`MipsArch` (`arch.rs`,
  `cpu_arm.rs`, `cpu_mips.rs`), `map_kernel_memory_allocations`,
  `fast_start_indirect_transfer_sites`, `ensure_mapped_ram_blob`,
  `reconcile_persistent_native_memory{,_with_private}`, `DPAS_SORTED`
  (`unicorn.rs`), `NativeMappingPins::{pin, release_image_after_unmap,
  private_len}`, `range_contains` and an unread `bytes` field (`pe_loader.rs`),
  `measure_text_units_width` and `PopupMenuModalState::cursor` (`coredll.rs`).
  Unused or misused bindings: the `MEM_DECOMMIT` import, `blobs`,
  `pending_wndproc_returns_live_pump_hook`, `current_thread_id_code_hook`, the
  dead first `fixed_ranges` assignment and one needless `mut`. Delete dead
  items unless a planned production caller is named; do not hide them with
  `allow(dead_code)`.
- Keep the five-file small-intermediate recovery archive/receipt. Larger cache
  cleanup requires fresh ownership/idle checks; D debug/build showed other
  editor/compiler activity. The five inventoried legacy paths were retired
  after separate approval, with selected source, patch and EXE/PDB evidence
  preserved and verified again under D: `evidence/` (post-order retirement
  receipt). Do not infer that other mirrors, hardlinked installed outputs,
  unique binaries/symbols/run evidence, synced references or SD files are
  disposable from that scoped approval.
- Keep `PROGRESS.md`, `TODO.md`, and `KNOWN_BUGS.md` short and current.
- Do not reintroduce historical ledgers into roadmap files.
- Keep `SOURCE_REFERENCES.md` as a compact index of active source evidence, not
  a running implementation diary.
- Keep registry fixtures and launch docs on `registry.reg`; temporary JSON
  parser regressions should use temporary fixtures only.

## Validation Queue

- Stub/raw smoke: `cargo test -j 1`
- Shell/resource/GDI: `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_kernel`
- GWE/message/menu/modal: `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_gwe`
- File/storage: `cargo test --features unicorn,trace,win32-desktop --test coredll_raw_memory_file`
- Shared scheduler/runtime: `cargo test --features unicorn,trace,win32-desktop --test basic_subsystems`
- Larger handoff: `cargo check --features unicorn,trace,win32-desktop` and
  `git diff --check`
