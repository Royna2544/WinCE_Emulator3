# Progress

Refreshed on 2026-10-17 (client KST date). This file records confirmed facts only.
Older investigation ledgers were purged; use commits, logs, and targeted source
references for historical reconstruction.

## 2026-10-17 unconnected machine-state port foundation (focused-only)

The new crate-private `MachineKernelPort` module holds the caller's Kernel and
framebuffer in one fallible short-loan cell and latches the first typed
terminal failure. The [partial validation receipt](evidence/stage3-machine-port-foundation-20261017/validation.receipt.md)
records a01 focused no-default and a02 focused release, each **3/0** (exit
0): owned observation and loan release, nonpanicking nested cross-resource
refusal, and first-failure precedence. There is no separate compile-fail
fixture. Broad and feature-matrix validation remain pending.

This module has no production caller. Native attempts and hooks still use
their prior Kernel/framebuffer paths; no attempt/leaf authority, hook
teardown lifetime, selected-process restoration or exit-event fence has
been connected. It is not Step 2 port acceptance or a live-app result. No
app or writable mount was run.

## 2026-10-17 preparatory selected-process context stamp (bounded source/test gate)

`CeKernel` now exposes a crate-private selected-context stamp made from the
canonical process birth and a checked generation. A stamp cannot be captured
without a registered selected birth. Every selected-process metadata/state
setter advances the generation before changing state; the two accepted exit
compatibility-state writes also advance it, while tested refused exits leave
it unchanged. Matching re-resolves the birth, so a PID reused through the
public handle table cannot validate an old stamp without a selector write;
generation overflow permanently poisons capture rather than wrapping.
The stamped [a01 focused no-default](evidence/stage3-selector-context-stamp-20261017/a01-focused-no-default.raw.log)
and [a02 focused release](evidence/stage3-selector-context-stamp-20261017/a02-focused-release.raw.log)
attempts each passed **5/0**. Same-source
[a03 broad release](evidence/stage3-selector-context-stamp-20261017/a03-broad-release.raw.log)
passed 19 blocks **2,189/0/10 ignored**. [A04 broad no-default](evidence/stage3-selector-context-stamp-20261017/a04-broad-no-default.raw.log)
passed 19 blocks **1,548/0/2 ignored**; a05–a07 feature checks exited 0.
All seven attempts kept unchanged source hashes
([receipt](evidence/stage3-selector-context-stamp-20261017/validation.receipt.md)).

This is provenance only: no production run/restore path consumes the stamp,
and it does not prove execution-live ownership, machine/checkpoint lineage or
safe restoration after A→B→A. Clone copies the generation, and direct
mutations of public backing objects are not all stamped. The callback-scoped
Kernel/framebuffer port, checked handoff token and O-T7 leaf/custody migration
remain separate work. No app or writable-mount run was made.

## 2026-10-17 test-only parked-publication expected red

The [bounded expected-red receipt](evidence/stage3-parked-publication-red-20261017/validation.receipt.md)
records a02's stamped focused ignored test (exit 101) on `unicorn.rs`
SHA-256 `C5F6A48CDEE38DA122EAC8960BD84E3F5A900BAE628CFD6995A4AC48BBD3EF38`.
Real-MIPS A executes `CreateProcessW` and B runs its post-native first slice,
executing fixed-stack `SW`/`LW`; the test verifies the stored value. A
test-only framebuffer gate holds a parent parked-queue read borrow only
during B's `Parked` publication. The production path returns typed
`Backend`; B remains exact-birth Kernel-live,
unsignaled and wait-unready, but the final custody assertion finds zero B
slots instead of one. The a01 fixture missed its intended expectation before
this a02 witness. This forced borrow conflict does not establish natural
reentry, a live-pump failure or an iNavi run. No production fix was made;
Cut 3a's non-runnable native-fault quarantine does not cover a runnable B at
the parked-publication boundary.

On the same test-only source, loopback-enabled a04 release passed 19 blocks
**2,184/0/10 ignored**, a05 no-default passed 19 blocks **1,543/0/2
ignored**, and a06–a08 feature checks exited 0. The initial sandboxed a03
release attempt had 45 failures concentrated in socket/remote tests, with
multiple explicit Windows `10013` errors; its identical-source loopback
rerun is the counted release result. An environment restriction is the
supported inference, not an attributed source regression.

## 2026-10-17 bounded Cut 3a non-runnable child CPU quarantine

An ordinary real-MIPS focused control passed under debug and release before
the pre-entry follow-up, then passed again in a09 debug on the superseded
producer-side source: A
executes `CreateProcessW`, retains its staged nonzero return after native
sync/finish, and B runs post-native. B's fixed-stack first-touch `SW`/`LW`
commits an exact-birth page, then a `READ_UNMAPPED` native fault leaves its
original CPU in one non-runnable, generation-checked machine custody slot,
not the legacy parked tree. B remains Kernel-live and unsignaled; its native
owner is closed and its unsafe saved continuation is cleared. A
`TerminateProcess` request alone retains the slot; same-Kernel checked
accepted exit retires it and the fixed-stack pin in this focused fault shape.
Stamped a07 expected red reproduced a synthetic post-load/pre-native-entry
error quarantining an unpoisoned CPU. It stopped at the poison-bit assertion;
the exit consumer would refuse that bit by source inspection, not an executed
consumer control.
A producer-side seal made the a08 synthetic pre-entry control green; a09's
real-MIPS fault control was also green in debug on the same `unicorn.rs`
SHA-256 `1AAE8A66ACFFFE3D8FE9CED10687C98A28879BF9267A1736D7B4689F21A33090`.
A10 focused release passed on an intermediate producer-only source, now
superseded. Current source makes `CpuCustody::quarantine_reserved` enforce the
terminal seal on `Reserved`→`Quarantined` under Unicorn; the exit consumer keeps
its poison-bit backstop. The [owner-sealed receipt](evidence/stage3-cpu-custody-quarantine-20261017/validation.receipt.md)
records a11 focused release **1/0**, a12 release **2,184/0/9 ignored** and
a13 no-default **1,543/0/2 ignored** across 19 blocks each, plus a14–a16
three feature checks exiting 0. All six source hashes matched the owner seal
before and after every counted attempt. The final-source real-MIPS control is
included in a12, not a separate final-source focused attempt.
The a07/a08 precondition is synthetic, not proof of natural reachability or
general error-path closure. The production run loop
owns custody, monitor save/restore refuse while it exists, and status/debug
publish slot identity and retained mapped-blob bytes. Those bytes are not a
measured host RSS peak. The Kernel-minted exit witness carries exact birth,
not independent machine lineage; its production use relies on the same
Kernel instance. General private VM and mapped-file-view exit cleanup,
runnable/parked custody, public-run bypass removal, a callback-scoped
Kernel/framebuffer port and live-app behavior remain open.

The pre-seal a03 broad release result (**2,183/0/9 ignored**) remains
development evidence only. No app or writable-mount run was made.

## 2026-10-17 bounded native-entry precondition

The [source/test receipt](evidence/stage3-native-entry-precondition-20261017/validation.receipt.md)
seals `unicorn.rs` SHA-256 `55642d7b415f8227b2c6f8def2d3cd1f9cfde46d2d325edced803ef66932ed3d`.
A same-host-thread guard now refuses a second native guest entry before private
CPU setup; the preseeded-busy real-MIPS control returned typed Backend without
guest execution, then the same CPU ran after the guard released. The initial
late-guard retry failure is preserved. Sealed focused release passed **1/0**;
complete release-feature and no-default suites passed **2,178/0/10 ignored**
and **1,539/0/2 ignored** over 19 blocks each, with three feature checks
exiting 0. This is a forced precondition control, not natural reentry or a
machine-owned leaf permit. The connected Kernel/framebuffer port remained
open at that seal; the later narrow Cut 3a quarantine is recorded above.
No app or mount run was made.

## 2026-10-17 bounded scheduler-block self-reentry fence

The [source/test receipt](evidence/stage3-scheduler-block-fence-20261017/validation.receipt.md)
seals `unicorn.rs` SHA-256 `f93d755705095d464bb7e91c9f67cddc75a3b54281a20166ae61941730221277`.
Its real-MIPS preseeded-busy scheduler service returned typed Backend with a
nonzero stop PC, unchanged selected PID and full virtual framebuffer, and a
closed native owner; the unbusy control passed. The same-source complete
release-feature and no-default suites passed **2,176/0/10 ignored** and
**1,539/0/2 ignored** over 19 blocks each, with three feature checks exit 0.
An initial sandboxed release attempt failed 45 socket/remote tests with
Windows `10013`; the identical source and command passed when loopback was
allowed. This is a scheduler-block-family diagnostic only: natural reentry,
other raw hook aliases, the whole-attempt port, and O-T7 CPU custody remain
unproved. No live app or writable mount was run.

## 2026-10-17 bounded exact-primary stopped-exit gate

The [v4 source/test receipt](evidence/stage3-primary-exit-birth-v4-20261017/validation.receipt.md)
seals `unicorn.rs` SHA-256
`1edeb353bfa618c760f02cf5397b3d3f4703c4a2a726225e496f237fd1916347`.
The tested stopped child/boot exit paths require a matching canonical primary
thread birth and CPU witness; a stopped boot exit lacking that authority
requests typed recovery, while an ordinary non-exit stop is not promoted to
an exit. Final-source focused boot recovery passed **1/0**; complete 19-block
release-feature and no-default suites passed **2,175/0/10 ignored** and
**1,539/0/2 ignored**. Feature checks for Unicorn alone, Unicorn plus
desktop, and the normal feature set all exited 0.

The earlier [a10 broad release attempt](evidence/stage3-primary-exit-birth-20261017/broad-release-a10.raw.log)
failed **1,144/16/7 ignored** (exit 101); it was not a gate pass. The later
[v2 focused guest-thread-stack selection](evidence/stage3-primary-exit-birth-v2-20261017/focused-guest-thread-stack-a01.raw.log)
passed **133/0/2 ignored**, but was not a complete broad gate. These are
separate source snapshots and remain in the validation lineage. Bounded
primary-exit checks do not establish general exit reconstruction, Task/Frame
authority, O-T7 CPU custody, CE6 parity or live iNavi behavior. No app or
writable mount was run.

## 2026-10-17 test-only live-pump fixed-stack fault-hook witness

The [bounded receipt](evidence/stage3-live-pump-fixed-stack-hook-20261017/validation.receipt.md)
seals test-only `unicorn.rs` SHA-256
`ff0bce3c3976c5f71bbcddbb2e35d182dd7d221552223529ed49b9eac720abd0`.
Two ordinary real-MIPS controls execute `SW`/`LW` on a previously uncommitted
fixed-stack page in `live_pump:true`, then verify the exact process-birth page
pin, stored bytes, progress registers and nonzero saved PC. One requests an
explicit PC stop (global code-hook mode); the other uses `stop_pc:None` and
the default hookless, count-limited mode. The production memory-fault repair
route is identified by source exclusivity, not a callback counter. Focused
release passed **2/0**, complete 19-block release **2,164/0/11 ignored** and
no-default **1,532/0/3 ignored**. No production pointer/loan or CPU-custody
change was made; callback reentry, child survival and live iNavi remain open.
No app, mount or GPS run occurred.

## 2026-10-17 direct Completed empty-descendant source gate

The [bounded receipt](evidence/stage3-completed-borrow-20261017/validation.receipt.md)
seals HEAD `0fb3bc0eaabb64fc153c1b62248cb2f3d6189561` and `unicorn.rs`
SHA-256 `0dcf5c5d04d0e632cbe405554bfc7029b7eea654070d4ca1fbdc3070ac136d06`.
The [stamped direct-MIPS expected red](evidence/stage3-completed-borrow-20261017/attempt01-direct-completed-expected-red.raw.log)
reached first-slice `Completed` after restoring a child-owned window while a
test framebuffer held a parent-queue read borrow. The old unconditional
`borrow_mut` panicked. The current source skips parent-queue publication when
the completed child has no descendants; its ordinary test checks one exact
exit, exit code `0x63`, a ready process wait, restored window state and no
host panic. For nonempty descendants the source uses fallible borrow and
capacity reservation, reporting typed terminal `Backend` on conflict after
exit effects have committed. This does not preserve descendant CPU custody
on that failure. Sealed focused attempt 05 passed **1/0**; 19-block release
attempt 04 passed **2,162/0/11 ignored** and no-default attempt 06 passed
**1,532/0/3 ignored**. Source hash stayed unchanged throughout those
attempts; HEAD changed during attempt 04 when the source was backed up.
[Ignored attempt 07 at that seal](evidence/stage3-completed-borrow-20261017/attempt07-current-native-fault-red.raw.log)
failed at child CPU custody (exit 101). No later-slice,
nonempty-descendant or live-app result is claimed. The two old ignored
A→B/held-borrow fixtures have been retired because they missed their
post-deferral preconditions; their historical evidence remains retained.

## 2026-10-17 deferred CreateProcessW post-native source gate

The [bounded source/test receipt](evidence/stage3-createprocess-deferred-20261017/validation.receipt.md)
seals `src/emulator/unicorn.rs` SHA-256
`4be7f3f25e4a51ad762d40b8064b2048c4593243dd991a6a5010444ef5eab69d`.
Successful guest `CreateProcessW` now stages the parent's nonzero return PC
and defers child launch until after the native attempt in both non-live and
live-pump modes; non-live rotation can choose the child before the parent
resumes. Final-source focused controls passed **5/0**, and the two-mode
file-view reseed control passed **1/0**. Complete 19-block suites passed
release-feature **2,161/0/13 ignored** and no-default **1,532/0/3 ignored**.
The file-view control checks canonical host mapping bytes after reseed, not a
guest-visible post-lease read.

Final-source ignored reruns [16](evidence/stage3-createprocess-deferred-20261017/attempt16-current-completed-red.raw.log)
and [17](evidence/stage3-createprocess-deferred-20261017/attempt17-current-first-slice-red.raw.log)
exited 101 at unmet preconditions of the old `Completed`/A-exit fixtures;
neither reproduces nor repairs those pre-deferral failures. Ignored
[attempt 18](evidence/stage3-createprocess-deferred-20261017/attempt18-current-native-fault-red.raw.log)
exited 101 when an admitted child's ordinary native fault lost its sole
CPU owner. The two stale fixtures were subsequently retired; the native-fault
fixture was later replaced by the narrow Cut 3a control above. Full machine-
owned runnable/parked CPU custody, the
callback-scoped Task/Kernel/framebuffer port, live-pump later-slice survivor
behavior and Stage 5 scheduling remain open. No live app or writable-mount
run occurred.

## 2026-10-17 counted checkpoint guard and CPU-custody controls

The seven formerly ignored `src/main.rs` checkpoint-lineage controls are now
ordinary tests. On the [unchanged source seal](evidence/stage3-checkpoint-unignored-20261017/validation.receipt.md),
complete 19-block suites passed no-default **1,532/0/3 ignored** and
release-feature **2,157/0/13 ignored**; all seven controls appear as `ok` in
both logs. Initial native-build debug-info and sandbox loopback failures were
separately recorded and resolved for these attempts. This verifies the strict
save/restore refusal, not live machine relineage or in-session rewind.

At that earlier pre-deferral seal, three
[ignored expected-red controls](evidence/stage3-completed-reentry-red-20261017/validation.receipt.md)
failed, each with a stamped, NUL-free focused log: a `Completed` child
with a held parent queue panicked on `RefCell::borrow_mut` and stranded its
survivor, an already-exited A skip dropped guest-created B's CPU, and an
ordinary native fault dropped an admitted child's CPU while its Kernel birth
stayed live. These were offline CPU-custody failures, not a live-pump or iNavi
result. No app or writable-mount run occurred.

## 2026-10-17 opt-in native timer boundary control

The [bounded timer rendezvous diagnostic](evidence/native-timer-rendezvous-20261016/validation.receipt.md)
passed 48/48 fresh-owner iterations (16 timer-first, 16 hook-first, 16
simultaneous release) under an external 60-second process-tree watchdog. Every
iteration returned from both native boundaries, reported `TIMEOUT=1`,
`COUNT_STOP=0`, PC `0x1000`, and closed its owner. The accepted build/test logs
are stamped, NUL-free and hash-sealed. One corrupt generated Unicorn rlib was
removed after exact-path/hash verification and rebuilt in the same `target/`;
the two older NUL-corrupt logs and a failed linker attempt remain untouched.
This does not explain either earlier full-suite AV or prove native race freedom.
The C probe is opt-in only; production timeout and `uc.c` behavior did not
change. No app or writable-mount run occurred.

## 2026-10-16 bounded Stage 2 writable-image source gates

The [VM foundation](evidence/stage2-writable-image-vm-20261016/validation.receipt.md)
adds exact birth/module-generation/run-keyed, stable writable-image backing in
`MemorySystem`, copied host access and native-access pins. Focused no-default
VM controls passed **3/3**. A separate [checkpoint control](evidence/stage2-writable-image-vm-20261016/test-main-checkpoint.validation.receipt.md)
passed **1/1**: monitor save/restore refuses when source or destination has
this backing, pending full machine relineage. Its physical status is
`Unclassified`; the new path charges no guest physical budget.

An [explicit opt-in main-image adoption gate](evidence/stage2-writable-image-green-20261016/validation.receipt.md)
passed **3/3** focused release controls, including a real MIPS write of
`0x51a7c0de` seen through canonical VM bytes and a fresh native owner with
zero image copy-in/readback in that synthetic path. The loader's final-run
manifest, live birth and VM lineage are checked; a later-run publication
failure preserves the original CPU blobs. The normal production SD app path
does **not** call adoption and remains copy-backed. Preloaded/runtime DLLs
remain separate work.

The [mutable-code read gate](evidence/stage2-mutable-code-fallback-20261016/validation.receipt.md)
passed **5/5** focused release controls: a live writable-image instruction is
read from native memory, while a failed native read does not fall back to a
stale mutable blob. The [complete source suites at that seal](evidence/stage2-mutable-code-fallback-20261016/broad-validation.receipt.md)
passed release **2,116/0/9 ignored** and no-default **1,515/0/3 ignored**
across 18 result blocks each.

The later [synthetic native-map failure gate](evidence/stage2-writable-image-native-failure-20261016/validation.receipt.md)
passed **1/1** focused release control: a real second-run `mem_map_ptr`
collision leaves both VM runs inaccessible while pinned; checked abort-close
retires the native slot, a retained clone keeps its pin, and a fresh owner
cannot reuse staged continuation/map metadata. Its serial-harness complete
suites passed release **2,117/0/9 ignored** and no-default **1,515/0/3
ignored**; one later default-harness full release retry also passed
**2,117/0/9 ignored**. An earlier default-harness attempt crashed on a
WINDOWPOS test-thread stack overflow, so those green runs do not establish
repeatable broad-run stability
([serial receipt](evidence/stage2-writable-image-native-failure-20261016/serial-broad-validation.receipt.md),
[retry](evidence/stage2-writable-image-native-failure-20261016/default-retry.addendum.receipt.md)).

The [writable-image/legacy file-view overlap gate](evidence/stage2-writable-image-file-view-20261016/validation.receipt.md)
now refuses overlapping legacy seed/readback before either direction mutates
bytes, including same/foreign PID and retired-but-pinned runs; adjacent
nonoverlapping views retain ordinary behavior. Focused release **5/5** and
no-default **1/1** passed. Its [stable-source complete suites](evidence/stage2-writable-image-file-view-20261016/broad-validation.receipt.md)
passed release **2,119/0/9 ignored** and no-default **1,516/0/3 ignored**
across 18 result blocks each with the default test harness.

The next [writable-image/VM-allocation overlap gate](evidence/stage2-writable-image-vm-allocation-20261016/validation.receipt.md)
preflights `LegacySeed` in both publication orders, same-birth stable
private/shared reservations and pending holes, fixed heap/heap cursor, and
native seed/reclaim/fault/map paths against active or retired-pinned image
ranges. Foreign-birth private backing may reuse the numeric VA. Focused
release **13/0/0** and no-default **8/0/0** passed; default-harness complete
suites passed release **2,127/0/9 ignored** and no-default **1,520/0/3
ignored** across 18 blocks each. Six Rust source hashes remained stable
through all four attempts despite a backup-only HEAD change during the first
focused run. Unicorn-only and Unicorn-plus-desktop feature checks also passed
on that stable source seal. These are source/test results, not an iNavi run,
a release EXE/PDB seal, a footprint or speed result, O-M3 physical-page
accounting, or full Stage 2 acceptance.

The [synthetic exact-exit/retained-pin gate](evidence/stage2-writable-image-exit-retire-20261016/validation.receipt.md)
passed focused release **18/0/0** and no-default **12/0/0**, with complete
retry release **2,132/0/9 ignored** and no-default **1,524/0/3 ignored**.
Only the verified birth retires after D-011 preflight; retired pins refuse
writes. Attempt 01's full release AV remains unexplained despite the retry.

Other direct host writes, restore/rebind and exact module-retirement fences,
DLL migration, checkpoint relineage and production activation remain open.
The [H-W1 host-write authority design](docs/subsystems/guest-memory.md#h-w1-host-origin-native-write-authority-design-record-not-implemented)
inventories current native writers; it is not implemented. The two earlier
default-harness native AV incidents remain under one unresolved bug. A
[matched six-run release library cohort](evidence/parallel-native-av-cohort-20261016/validation.receipt.md)
at stable source observed **0/3** crashes with explicit serial harness threads
and **0/3** with default harness threads; all six runs passed **1,133/0/6
ignored**. This does not explain the prior full-suite crashes or establish
production safety. The scoped native audit found a timer worker on nonzero
Unicorn timeouts, so one Rust emulation thread alone is not a single-thread
safety proof.

## 2026-10-15 Live-run preparation, historical reds and design refresh

The A12, A10 and finite-Sleep gates are in the Stage 3 list below. The
detailed per-section text before this condense is at
`git show 75dd5560:PROGRESS.md`.

- Live-run preparation (no app run). Approval
  `call_9b35e9d31fdf402ea082a3eb6a40ae16` started an adjacent backup, but the
  old `File.Replace` helper failed after the SD ZIP only (5,973,898,396 bytes,
  SHA-256 `6032F822…`; [disposition](evidence/pre-live-mount-backup-20261015T025500Z/aborted-approval-disposition.md)).
  No runner or guest launch followed. The approval is `withdrawn` in both
  byte-identical ledgers (seven rows, SHA-256 `53BAD1AB…`;
  [ledger](docs/evidence/ledger.md)), and that SD ZIP can't be reused. The
  repaired scripts were resealed as a04
  ([seal](evidence/stage3-live-evidence-20261015/prelaunch-repair-a04-source-seal.json)
  `C495AB53…`, patch `B9FBE5F1…`, offline gate `759017B6…`: 123/73/14/9). a04
  adds an independent 355-second watchdog worker. The user then chose to skip
  the a04 run and begin the scheduler cut (`call_49d1b609465a4e668da30a31361beb44`),
  so a04 is historical and not live-usable. A later run needs a fresh
  source/build seal and a new specific approval.
- Historical reds, superseded by the A10/A12 gates:
  - receiver-context Send a02–a04
    ([receipt](evidence/stage3-getmessage-blocked-send-context-20261015/validation.receipt.md));
  - the ordinary queued-send receiver-exit/TID-reuse Kernel red
    ([receipt](evidence/2026-10-15-ordinary-queued-send-receiver-exit-reuse-red.md));
  - the preliminary A11 run and the confirmed A11b next-lease reciprocal red,
    where T's inner Send replayed as send ID 3 / wait 4
    ([receipt](evidence/scheduler-source-cut-a11-reciprocal-red/validation.next-lease.receipt.md)).
- Design refresh (no source or run):
  - Stage 7 needs device- and interaction-grounded footprint and
    responsiveness budgets before calling anything "small" or "fast".
  - A proposed event-to-frame lineage gate is in
    [remote-observability](docs/subsystems/remote-observability.md).
  - The two-Sleeps-on-one-thread red was withdrawn: in CE6 a positive Sleep
    blocks its caller and dispatches no WNDPROC.

## 2026-10-16 and 2026-10-15 Stage 3 source gates

Each gate is bounded; none is full Stage 3 or live-app acceptance. Newest
first. Totals are passed/failed/ignored for release
`unicorn,trace,win32-desktop`, then no-feature. Linked receipts identify the
source and validation evidence; a source gate is not a live EXE/PDB or app-run
result. The per-slice text before this condense is at
`git show 75dd5560:PROGRESS.md`.
An offline [CE6 source/REA exit check](evidence/ce6-rea-exit-20261016/findings.md)
found per-process thread/handle cleanup and distinct dumped-COREDLL
`TerminateProcess`/`CloseHandle` wrappers. It did not locate the dumped
kernel's exit body or establish live child-process behavior. The current
parked CPU tree is not process-lifetime authority.
The bounded [wait/child-lifetime follow-up](evidence/ce6-rea-exit-20261016/findings.md#2026-10-16-bounded-waitchild-lifetime-follow-up)
also located dumped-COREDLL `WaitForSingleObject` and `GetExitCodeProcess`
kernel-call wrappers. Local CE6 source confirms `CreateProcess` waits for
loader completion, process waits use a pinned event, and requested
termination precedes accepted execution exit. This reinforces independent
child CPU custody; the dumped kernel process-exit body remains unidentified.
The separate [dumped-COREDLL REA thread-exit follow-up](evidence/ce6-rea-exit-20261016/findings.md)
located `ExitThread`'s KData-derived branch, `PrepareThreadExit` call and
kernel method `0xFFFFBBCE`, distinct from `TerminateThread`'s resolved-handle
method `0xFFFFB7DE`. It is static wrapper evidence, not a decoded ROM exit
body or live child-lifetime proof.
An offline [REA check of the shipped iNavi PE](evidence/ce6-rea-inavi-20261016/findings.md)
located static `FindWindowW`-guarded `CreateProcessW` branches for
`happyway_win.exe` and `iSearch.exe`, each retaining returned process/thread
information. This corroborates child ownership as a real app dependency but
does not establish branch execution, boot timing or UI progress.
- At its earlier seal, a [real-MIPS non-live expected-red](evidence/stage3-real-mips-first-slice-20261016/validation.receipt.md)
  executed an actual ordinal-493 `CreateProcessW` import in guest A.
  A exits once with `0x63`; B remains exact-birth Kernel-live and unsignaled,
  but `CreateProcessChildParkSkipped(already_exited)` drops the local CPU now
  switched to B, leaving the parent queue empty. Focused attempt 03 failed
  on B CPU reachability; the fixture was ignored for ordinary suites at that
  seal and was later retired after missing the post-deferral route. This is
  not the live-pump route or a live iNavi result.
- A [test-only first-slice native-fault expected-red](evidence/stage3-post-admission-native-fault-20261016/validation.receipt.md)
  executed a synthetic child to `READ_UNMAPPED` at its earlier seal. Its exact
  birth stayed Kernel-live, `STILL_ACTIVE` and unsignaled, but the error dropped
  the constructed CPU (parent queue zero versus one required). The final
  ignored fixture is source-sealed at `unicorn.rs` SHA-256
  `508101e2942f997472de99c1765905756104e06caf6313ac96717292e3250c79`;
  focused red exit 101 and normal selection one ignored/zero failed. No
  production custody repair, broad suite or mounted app run followed at that
  earlier seal; the later deferred-source gate is reported above.
- The [pre-CPU pending-launch red](evidence/stage3-pre-cpu-launch-retention-20261016/validation.receipt.md)
  showed ambiguous lazy file-view setup draining an already-published exact
  child launch while its Kernel birth stayed live. The bounded
  [green source cut](evidence/stage3-pre-cpu-launch-retention-green-20261016/validation.receipt.md)
  now restores the exact current launch plus its unprocessed tail on retryable
  errors before child CPU construction. The same-birth retry and malformed-PE
  tail controls pass **2/0**, two related pending-child controls pass **2/0**,
  and no-default and normal-feature checks pass. This does not retain a CPU
  after DLL-load/native/publication failure at that seal; the later narrow
  Cut 3a fault quarantine is reported above. No app or mount run followed.
- A [current-source, stamped primary-identity expected red](evidence/stage3-primary-claim-red-20261017/validation.receipt.md)
  explicitly reran the ignored `task_ownership_primary_identity_outlives_launch_queue_and_trace_ring`
  control and failed at its intended assertion (exit 101) with unchanged
  source. The post-native launcher drains Kernel's entire pending queue before
  child execution; after 129 later launch records roll the 128-entry trace
  ring, a still-live child's primary TID was no longer recognized. The
  [bounded canonical-primary fix](evidence/stage3-primary-canonical-green-20261017/validation.receipt.md)
  retains its exact primary birth on the canonical process instead of those
  ephemeral records. Five focused controls pass; stable broad release passed
  **2,169/0/10 ignored** and no-default **1,537/0/2 ignored**, 19 blocks each.
  The first broad release attempt was invalidated by mid-run validation-script
  churn and was rerun. The [dumped COREDLL REA check](evidence/ce6-rea-primary-thread-20261017/findings.md)
  supports durable identity but not complete exit semantics. Boot-main
  enumeration, broader exit reconstruction and CPU custody remained open at
  that gate; the later bounded stopped-exit gate is recorded above. No app ran.
- A [pre-Cut-3a ignored native-fault rerun](evidence/stage3-cpu-custody-current-red-20261017/validation.receipt.md)
  was expected red (exit 101): a real-MIPS child faulted on its first
  admitted native slice, remained Kernel-live and had zero parked CPU owners.
  The fixture's temporary directory was removed. This is not a live-app run.
- The [v2 offline preflight gate](evidence/stage3-preflight-v2-review-20261017/validation.receipt.md)
  passed **171/171** process-free source/build evidence controls and **83/83**
  runner guards. It requires a current-seal HEAD, executed focused test,
  clean/release log chain and wrapper-stamped hashes of the newly built
  target EXE/PDB matching preserved bytes. Historical a04 binaries lack that
  witness. No new rebuild, backups or mounted app run followed.
- The [live CPU checkpoint red](evidence/stage3-live-checkpoint-lineage-20261016/validation.receipt.md)
  copied boot PID 1/TID 1 execution births on save and restore. The separate
  [pristine-CPU Kernel reds](evidence/stage3-kernel-checkpoint-lineage-20261016/validation.receipt.md)
  showed that even an empty CPU wrapper cloned live boot and queued-child
  births. The bounded [fail-closed source gate](evidence/stage3-kernel-checkpoint-lineage-green-20261016/validation.receipt.md)
  now refuses monitor save/restore while canonical process/thread records,
  pending launches or CPU custody remain. Five former reds pass under release
  and no-default profiles; existing checkpoint selection passed **4/0 with
  5 ignored** in each at that earlier seal, and three feature checks passed.
  Positive copies use a
  synthetic fully retired Kernel only; live in-session rewind and full
  machine relineage remain absent. A D-011 recovery hold therefore requires
  repair and restart, not a cloned live checkpoint. No app/mount run followed.
- A later [orphaned-frame expected red](evidence/stage3-checkpoint-orphan-frame-20261016/validation.receipt.md)
  exposed a gap in that first guard: an exited, aliasless thread retained an
  active finite-Sleep frame while both filtered snapshot lists were empty.
  Save and restore each cloned its original task birth. The
  [canonical-map guard](evidence/stage3-checkpoint-orphan-frame-green-20261016/validation.receipt.md)
  now checks the underlying process/thread records on both sides. All seven
  lineage controls pass under release and no-default profiles; ordinary
  checkpoint selection passed **4/0/7 ignored**, serial no-feature passed
  **1,525/0/10 ignored**, raw Kernel integration passed **133/0**, and four
  feature checks passed at that earlier seal. The seven controls were later
  unignored and counted in the 2026-10-17 suites above. This is refusal, not
  frame cleanup or live rewind.
- The [combined boundary gate](evidence/stage3-custody-boundary-combined-20261016/validation.receipt.md)
  passed four normal-feature integration suites (**908/0**) and serial
  no-feature `cargo test -j 1` (**1,525/0/8 ignored**) on the same sealed
  four-file source. Its five stamped raw logs were hash-checked after closure.
  This predates the later canonical-map guard and is not full release-feature,
  machine-owned custody or live-app acceptance.
- Bounded first-slice child `Exited` survivor source gate
  ([receipt](evidence/stage3-first-slice-survivor-green-20261016/validation.receipt.md)):
  production-helper synthetic A→B controls move the original nonmodal B CPU,
  saved context and blocked wait to the parent queue after A's accepted exit;
  mismatched saved B ownership refuses before Kernel effects. A held parent
  queue borrow refuses A's exit before effects, then the production publication
  path returns a typed terminal error without a host panic. That error drops
  the local A+B CPU tree; it is not a recovery hold. The final source passed
  focused **3/0**, 19-block release **2,147/0/10 ignored** and no-default
  **1,524/0/3 ignored** suites, plus Unicorn-only and CI-feature checks.
  These controls do not run
  guest MIPS or establish first-slice classifier reachability, full Stage 3 or
  live-app behavior. No app or writable-mount run was made.
- Bounded parked-survivor and saved-CPU-owner source gate
  ([receipt](evidence/stage3-survivor-resume-green-20261016/validation.receipt.md)):
  natural parked and requested-active stopped exits preserve the original
  nonmodal foreign CPU tree, context and wait after exact-birth process
  retirement. A saved-context survivor must have a nonzero current TID
  matching its saved exact thread birth, and that thread must still be
  execution-live; otherwise exit refuses before effects. The parked record's
  representative TID is not CPU authority. Four focused release controls
  passed; the combined sealed source passed release **2,144/0/10 ignored**
  and no-default **1,524/0/3 ignored**, each across 19 result blocks.
  Earlier a09 passed on a superseded source. Other first-slice shapes, other
  active exits, requested parked siblings, already-exited cleanup, generic
  duplicate pruning, foreign CPUs without saved context and PC resumability
  remain unproved. No live app or writable-mount run was made.
- D-011 modal-presence refusal
  ([receipt](evidence/stage3-modal-exit-failclosed-20261016/validation.receipt.md)):
  exact-birth stopped exits fail closed before Kernel effects when the affected
  active/parked subtree contains a MessageBox or popup; modal-bearing parked
  duplicates are retained. The three expected-red controls turned green;
  broad suites passed 2,137/0/10 and 1,524/0/3 (19 blocks each).
- Decoded direct-kernel pseudo self-exit
  ([receipt](evidence/stage3-raw-decoded-pseudo-exit-20261016/decoded-pseudo-exit.receipt.md),
  [broad](evidence/stage3-raw-decoded-pseudo-exit-20261016/broad-validation.receipt.md)):
  only a witnessed live JALR, its delay slot and exact caller/owner births
  reach the shared D-011 finalizer, and forged or stale paths fail closed.
  2,107/0/9 and 1,511/0/3. Raw direct pseudo exit stays fixture-only.
- Native COREDLL pseudo self-exit
  ([receipt](evidence/stage3-native-pseudo-self-exit-20261016/validation.receipt.md)):
  the import stops MIPS without Kernel mutation in the hook; after owner
  close, exact witnesses gate D-011 preflight, Task/VM retirement and CPU
  sealing. 2,096/0/9 and 1,510/0/3.
- D-011 stopped host-exit framebuffer preflight
  ([receipt](evidence/stage3-host-exit-framebuffer-preflight-20261015/validation.receipt.md)):
  every saved backing is validated before any exit mutation. A failure returns
  typed recovery and holds the stopped CPU. At that historical seal the
  monitor allowed an earlier checkpoint restore; the later live-lineage
  guard now refuses such a clone, so repair and restart are required.
  2,094/0/9 and 1,510/0/3.
- Guest pseudo-exit caller birth
  ([receipt](evidence/stage3-pseudo-caller-birth-20261015/validation.receipt.md)):
  native and decoded self-exit require the live caller birth and exact
  process lease. 2,081/0/9 and 1,502/0/3.
- Host-exit exact window set
  ([receipt](evidence/stage3-host-exit-exact-window-20261015/validation.receipt.md)):
  only birth-selected live windows retire, without following foreign
  parent/owner links; an exact A->B SendReply waiter is woken.
  2,072/0/9 and 1,500/0/3. Guest `DestroyWindow` recursion is unchanged.
- Sender ownership
  ([receipt](evidence/stage3-sender-birth-20261015/validation.receipt.md)):
  the authenticated sender birth owns queued Send/Timeout through publication,
  exit cleanup and native re-entry. 2,067/0/9 and 1,495/0/3.
- Host-exit independent-window PID reuse
  ([receipt](evidence/stage3-host-teardown-20261015/validation.receipt.md)):
  window and backing selection uses frozen creation birth, not a recycled
  PID, and no-birth pseudo exit fails closed. 2,055/0/9 and 1,485/0/3.
- Common open edges for these exit gates: whole-exit atomicity after
  preflight, modal restoration, CE6 parity, full Stage 3 and live iNavi. The
  full per-gate text is at `git show e75df3ec:PROGRESS.md`.
- A13 same-birth orphaned `GetMessageW` source/test gate
  ([receipt](evidence/scheduler-source-cut-a13/validation.final.receipt.md)):
  the test-only expected red is preserved; saved-context recovery now leaves a
  matching birth-owned blocked payload parked, and a selected payload whose
  exact scheduler wait vanished fails before guest entry without taking the
  sender's reply/wait. Modal continuation and parked readiness require the
  exact live modal wait, including the real-button missing-wait controls.
  Final-source focused saved-context **7/7**, native orphan **1/1**, and modal
  **11/11** passed. The 18-group broad suites passed release **2,052/0/9
  ignored** and no-default **1,482/0/3 ignored**. This is not full Stage 3,
  live-modal acceptance, or an iNavi run.
- A12 strict reciprocal same-process Send source/test gate
  ([receipt](evidence/scheduler-source-cut-a12/validation.production.receipt.md)):
  post-format release **2,047/0/9 ignored** and no-default **1,482/0/3
  ignored** across 18 groups each; the no-replay and wrong-parent-birth
  controls are now normal tests. This does not support broader reciprocal
  shapes or establish live behavior.
- Connected A→B Send/GetMessage bounded source/test gate
  ([receipt](evidence/scheduler-source-cut-a10/validation.receipt.md)):
  A10 passed release **2,045/0/9 ignored** and no-default **1,482/0/3 ignored**
  across 18 groups each. A09's four-failure seal is historical, not a pass;
  neither result establishes full Stage 3 or live-app behavior.

- TaskFrame finite-Sleep scheduler source cut
  ([receipt](evidence/stage3-finite-sleep-frame-20261015/validation.receipt.md)):
  resealed HEAD `6626e74d` passed release **2,025/0/9 ignored** and
  no-feature **1,472/0/3 ignored** across 18 blocks each. Focused controls,
  the superseded diagnostic run and remaining boundaries are recorded above;
  this is not full Stage 3 or live-app acceptance.
- W-B1c mixed remote key/touch admission order
  ([receipt](evidence/stage3-mixed-input-order-20261015/validation.receipt.md)):
  the preserved expected red showed an earlier A-focused key reaching B after
  a later B-point DOWN changed focus in one drain. One typed, per-`CeRemote`
  sequenced FIFO now replaces the separate touch/key drains; tap expansion is
  contiguous, and active/hint keys resolve their live target at each event.
  Explicit HWND and W-B1b exact-birth consume/drop rules remain in force;
  message-time projection is unchanged. The key(A) → DOWN(B) → key(B) focused
  release control passed 1/0, and all 27 inherited W-B1b controls passed.
  Backup HEAD `0e06e2c5` matches the tested source; the a02 source seal was
  captured **after** the release run started, as its receipt states. Full
  release passed 1,992/0/9 and no-feature 1,452/0/3 across 18 result blocks
  each; Unicorn-only and desktop-without-trace checks exited 0. This proves
  local adapter admission order, not global HTTP/desktop or cross-device
  arrival order, CE6 focus policy, guest WNDPROC execution order or O-C5
  guest-time policy. No new iNavi run or tap-delivery result exists.
- W-B1b remote touch/key window-owner birth
  ([receipt](evidence/stage3-window-input-birth-20261015/validation.receipt.md)):
  stale explicit host HWNDs consume/drop input before focus, queue/status,
  timestamp, post-trace or wake effects; guest Get/Peek HWNDs remain retrieval
  filters. Coordinate-only touch uses live capture/delivery-time hit-test;
  keys use live focus/active ownership. Focused controls passed 27/27. The
  earlier a33 release and a34 no-default suites failed on an obsolete hidden
  integration fixture; a35 repaired that fixture and a36/a37 focused reruns
  passed before the fresh a38 source seal. Sealed a38 release passed
  1,988/0/9 and a39 no-feature 1,448/0/3 across 18 blocks each; a40/a41
  alternate-feature checks passed. Patch SHA-256 is `ED7A3B48…962D40`.
  Full exit capture retirement, implicit gesture ownership, host teardown,
  other producers and live iNavi remain open; W-B1c above is a separate
  local-order source gate.
- W-B1a window-owner birth
  ([receipt](evidence/stage3-window-owner-birth-20261015/validation.receipt.md)):
  windows retain one frozen creation owner; native creation ordinals 246/688/690
  preflight the captured caller, while explicit HWND posts, typed payloads,
  exact queue/wake/exit and CreateWindow callbacks use that owner. Five pre-fix
  reds, including in-slice MIPS self-termination, are preserved. The first
  broad release failed on stale typed-payload fixtures; corrected tests passed
  focused a16–a18, sealed release 1,967/0/9 and no-feature 1,427/0/3 across
  18 blocks each, plus Unicorn-only and desktop no-trace checks. The corrected
  base `0e686c9c` patch SHA-256 is `02429B95…E608DD`. Broadcast, HWND timers,
  hotkeys, sent/paint/modal paths, remote input, host teardown and live iNavi
  acceptance remain open.
- HWND-less thread-timer birth ownership
  ([receipt](evidence/stage3-thread-timer-birth-20261015/validation.receipt.md)):
  raw/native Set/Kill now carry the captured exact caller birth; timer keys,
  due publication, GWE queue/wake and pending clear use exact birth and a
  checked timer instance. A's queued and later-due timers cannot reach B
  after TID reuse; synthetic/older same-ID messages cannot clear B's pending
  bit. Mixed window/thread ordering remains tested. The expected-red a01 and
  test-only compile-error a05 are preserved; sealed release passed
  1,960/0/9 and no-feature 1,424/0/3 across 18 blocks each. Window timers,
  other numeric producers, clock policy and live acceptance remain open.
- Thread-directed GWE post/quit birth ownership
  ([receipt](evidence/stage3-gwe-thread-birth-20261015/validation.receipt.md)):
  `PostThreadMessageW`, `PostMessageW(hwnd=0)` and self `PostQuitMessage`
  now require a live exact birth before publication. Their posted FIFO entries,
  quit/status/timestamps, Get/Peek, wait wake/readiness and exact exit respect
  that birth; native stale callers and opportunistic window callouts cannot
  steal windowless posts. The pre-fix A→B control exposed inherited
  `QS_POSTMESSAGE` (`0x00080008`); the first unsealed full release attempt
  exposed two legacy fixture expectations and is preserved. The sealed a01
  source passed release 1,952/0/9 and no-feature 1,417/0/3 across 18 result
  blocks each. Other GWE producers and live acceptance remain open.
- Exact-birth wait ownership
  ([receipt](evidence/stage3-wait-owner-20261015/validation.receipt.md)):
  one `TaskWaitStore` now separates exact births from explicit legacy-unknown
  TIDs; production opaque/prepared wait publication verifies live birth and
  treats unexpected late failure as terminal backend error. Exact exit and
  dialog-HWND cleanup preserve a replacement task's waits. The a01 release
  attempt failed and remains preserved; a02 passed release 1,947/0/9 and
  no-feature 1,414/0/3 across 18 result blocks each. This is a source gate,
  not live-app acceptance.
- Occupied native SendMessageTimeout slot
  ([receipt](evidence/stage3-smt-occupied-20261015/validation.receipt.md)):
  an eligible second sender now stops with a backend error before raw
  dispatch can return or retain a result pointer. Real MIPS import-hook and
  compatibility controls passed; release 1,932/0/9 and no-feature
  1,402/0/3. Concurrent A+B task-owned frames remain open.
- Native parked SendMessageTimeout
  ([receipt](evidence/stage3-smt-birth-20261015/validation.receipt.md)):
  eligible cross-thread nonzero-timeout producer, re-entry and resume check
  captured birth/lease and the exact scheduler/native/GWE pair; stale cleanup
  abandons only the exact send ID. 1,929/0/9 (a02; a01 was green but its
  wrapper exited 1 after HEAD moved) and 1,402/0/3. Sealed at `d5738222` plus
  patch; committed unchanged as `7533b528`.
- Native guest exit
  ([receipt](evidence/stage3-native-exit-birth-20261015/validation.receipt.md)):
  `ExitThread` and the return-stub no-exit-candidate fallback check captured
  birth and process lease first. 1,923/0/9 and 1,399/0/3.
- Kernel sender exit
  ([receipt](evidence/stage3-send-exit-20261015/validation.receipt.md)): the
  first exit of an exact live birth removes its waits and queued sends; active
  sends detach the sender. 1,920/0/9 and 1,399/0/3.
- Parked SendMessage
  ([receipt](evidence/stage3-send-birth-20261015/validation.receipt.md)):
  cross-process native send checks sender birth before queueing and the exact
  pair before results or registers. 1,916/0/9 and 1,395/0/3 (both a02).
- Parked serial
  ([receipt](evidence/stage3-serial-birth-20261015/validation.receipt.md)):
  SerialRead/SerialCommEvent birth is checked before host RX polling,
  publication and completion. 1,908/0/9 and 1,395/0/3 (both a02), tested
  against a frozen `unicorn.rs` hash rather than a sealed HEAD plus patch.
- Parked queue/socket
  ([receipt](evidence/stage3-queue-socket-birth-20261015/validation.receipt.md)):
  MessageQueueIo/WinsockRead birth is checked before publication and host
  socket work. 1,901/0/9 and 1,395/0/3.
- Parked MsgWait
  ([receipt](evidence/stage3-msgwait-birth-20261015/validation.receipt.md)):
  birth is checked before timer pumping, inline completion and publication.
  Recorded 1,896/0/9 and 1,395/0/3, not raw-log verified (no raw logs kept).
- Parked Sleep/object wait
  ([receipt](evidence/stage3-parked-wait-20261015/validation.receipt.md)):
  stale A-to-B TID reuse cannot consume B's wake or restore A's registers.
  1,891/0/9 and 1,395/0/3.
- Stale-TID continuation and native stamp
  ([receipt](evidence/stage3-stale-tid-20261015/validation.receipt.md)):
  pending returns, suspended contexts, slice-end staging and the GetMessage
  bridge carry the captured birth in one `(TID, birth)` native cell.
  1,887/0/9 and 1,395/0/3.
- Prepared task wait
  ([receipt](evidence/stage3-waitprep-20261015/validation.receipt.md)): all
  targets are pinned before any mutation, then one publication or a full
  rollback; pin exhaustion returns `WAIT_FAILED`/`ERROR_NOT_ENOUGH_MEMORY`;
  opaque tokens stay unpinned. 1,883/0/9 and 1,395/0/3.
- CE WaitAny raw error
  ([receipt](evidence/stage3-wait-20261015/validation.receipt.md)): raw
  `WaitForMultipleObjects` keeps the kernel's last error, with CE6
  `schedule.c` controls. 1,875/0/9 and 1,391/0/3.
- O-K3 execution and wait pins
  ([receipt](evidence/stage3-pins-20261015/validation.receipt.md)): task
  records stay execution-live after the last alias closes, and registered
  waits hold birth-checked pins; the E-11 controls are green. 1,874/0/9 and
  1,390/0/3.
- TerminateProcess
  ([receipt](evidence/stage3-terminate-20261015/validation.receipt.md)): owned
  children terminate at preflight, at a self-call stop, and when parked or
  pending; exact-birth completion precedes the process-wait signal; an old-MIPS
  direct-kernel real-handle call fails instead of resuming unsafely.
  1,858/0/11 and 1,375/0/5.
- Still open across these gates: concurrent task-owned SMT frames, global
  expiry read as ready, receiver-side `DestroyWindow` result writes, an
  end-to-end return-stub control, KData PID from mutable process selection,
  raw queue/socket/serial/MsgWait paths, aliasless Sleep-suspend, opaque
  non-task pins, remaining numeric GWE producer-birth routing, native continuation payloads,
  wait-ID wrap/reuse and live modal behavior, caller wait parity and
  root-process real-handle completion
  (`TODO.md`, `KNOWN_BUGS.md`).

## 2026-10-14 Stage 2 source gates E-30 to E-44 and O-M2

Suite tallies, hashes and receipts are in `docs/evidence/ledger.md` (E-30 to
E-44, O-M2); the pre-condense narrative is in `git show ca9abe8f:PROGRESS.md`.

- Native attempt reports reach an optional host observer (E-30). O-P1 stop
  classification is closed; hardening remains (E-31, E-34).
- E-33 clamps the vendored Windows TCG commit tail to the reserved arena
  (D-009). It is not proven to be E-32's crash cause; E-32 (persistent owner)
  crashed at 135.3 s with 1.65 GB RSS on the splash.
- Public private `VirtualAlloc` uses owner-qualified stable pages (E-35). Fresh
  native owner per slice is the default; the persistent owner is an explicit
  Stage 8 opt-in (E-36).
- Fresh-owner view sync reads and seeds only natively mapped pages, and the
  generation mapper seeds only newly mapped pages (E-37).
- E-38, the only 330-second run on the fresh default (preserved E-37 EXE):
  exit 0 at 330 s, splash only, no map; sampled RSS maximum 430.7 MiB from
  160 host memory samples (BED2 555.6 MiB, persistent runs ~1.5 GiB);
  iNavi busy-slice 26.4 MIPS, inside E-9's 22–34; live heap reached 31.5 MB.
  The hash-pinned, read-only [historical validator](scripts/analysis/e38_validator.py)
  ([receipt](evidence/stage7-historical-validator-20261016/A01.receipt.md))
  rechecks 166 successful frame polls across four hash segments and 1,714
  completed busy slices. Frame hashes do not prove image or map content; the
  true continuous RSS peak and current-build live readiness remain unknown.
  That approval is spent.
- Real native two-process same-VA A→B→A isolation holds for public private
  pages and injected views (E-39). GlobalMemoryStatus uses one budget with
  the device boot reservation (E-40).
- Public named file-mapping views use canonical shared slabs, with
  `MappingBacking` Legacy|Slab one-way (E-41). Sparse anonymous migration was
  dropped: CE documents whole-section `SEC_COMMIT` (ms923951).
- Main-EXE non-writable pages are sealed from final post-trampoline bytes and
  pointer-mapped R/RX: real iNavi 1,379/2,168 pages, 5.39 MiB (E-42 selector,
  E-43).
- The initial heap arena and thread stacks are owner-qualified fixed slabs;
  heap commits at the allocation high-water, stack on fault or guarded host
  writes (E-44). Synthetic owned-blob copy fell from 21,049,344 B to 77,824 B.
- O-M2 birth-qualified heap cursor, handles, blocks, queued typed payloads and
  sparse fixed pages; copied spillover was removed. Final paired source suites
  passed release **1,803/0/17 ignored** and debug **1,341/0/13 ignored**;
  desktop no-trace check passed ([receipt](evidence/om2-heap-20261014/validation.receipt.md)).
- Open: safe heap address reuse, remote-heap target/cross-birth clipboard,
  unconverted direct stack outputs, writable image/DLL pages, Stage 3, and a
  combined live run (needs fresh approval). No O-M2 app speed/RSS is measured.

## 2026-10-12 to 2026-10-13 live-run and Stage 2 history (condensed)

Stage 2 facts are in ledger E-12 to E-29. The full pre-condense text is at
`git show 716ce0c4:PROGRESS.md`.

- Live runs of 330-600 s with GPS and taps exited 0 but stayed on the genuine
  splash. GPS bytes and taps reach the guest; parsing and map use are
  unproven. Persistent-owner memory peaked at about 1.49-1.55 GiB (E-22, E-27,
  E-29), and build 650 host-crashed at 129.1 s with 1.66 GiB RSS. E-38 is the
  only fresh-default run.
- A 600 s run showed a genuine Korean G-sensor error dialog (`Error Code: -15`)
  after `COM7:`, `MFS1:` and `SMB1:` opened. The growing 5/10/15/20 s sleeps
  are sensor-init retries: the Sleep caller at VA `0x000e69b8` multiplies a
  retry count by 5000 after `0x000e6754` fails. Decoding the YAS526B command
  protocol and Yamaha packed samples fixed that failure.
- Binary audit (read-only; no events forced):
  - The splash manager's show/destroy methods are `0x5e7a0`/`0x5e880`,
    WM_INITDIALOG startup is `0x2bc34`, the first initializer is `0x30e1c`
    (return `0x2bde4`) and pool worker 5 starts at `0xe6cd0`.
  - The second initializer `0x31970` (return `0x2be44`) creates the pool and
    dispatches jobs with no top-level wait. Owner+`0x39dd4` is the MDC object
    (vtable `0x4e57dc`, method `0x12e15c`); it calls engine virtual `+0x14c`
    and can sleep/wait and repeat.
  - MapInit returns normally on UI thread 1 with V0=1.
  - The old "command-flag readiness loop" lead was libpng cleanup
    (`png_free_data` at VA `0x00308298`).
- Fixed in that period, each with regressions:
  - the NotifyIcon payload overran 152 bytes (CE6 ordinal 481 marshals
    exactly 0x98);
  - the WNDPROC fallback restored only PC/RA/V0 (it now restores archived
    SP/GPR/HI/LO);
  - live-pump `CreateProcessW` replay;
  - historical child-lifetime traces reversed scheduler choices;
  - pre-run paths bypassed the CLI wall limit;
  - the unsafe raw-yield fallback was removed.
- The `C:\Temp` mirror and `target/` logs cited before the condense predate
  D-007 and are not current evidence.

## 2026-10-12 Session Facts (guest throughput)

Measured with `ce.slice_profile` (now logs `blocks`, `guest_insns`, `mips`;
`fc8c8384`) on 2-minute live runs, `--desktop virtual`, clean SD image, same
host. A bare Unicorn MIPS loop on this host does ~3200 MIPS unhooked, ~1900 with
a trivial block hook and ~400 with a trivial per-instruction hook, so hooks are
only slow when their body is.

- Root cause of the ~1 MIPS iNavi speed: the scheduler timeslice BLOCK hook in
  `run_with_unicorn` ran kernel/GWE/parked-process queries (visible receiver
  work, remote-control mutex, `Instant::elapsed`) on every translated block,
  about 15 us per block for iNavi (hundreds of hidden MFC child windows;
  `happyway_win` with none ran ~10x faster per block). Polling now only runs
  at amortized service points (every 1024 blocks, at most once per ms).
- The July "hookless A/B shows no gain" conclusion was a false lead: both arms
  kept that block hook. iNavi guest MIPS by variant: HEAD 1.05 average (4.6 in
  30-300 ms slices); amortized hook with the global code hook 7.3 average
  (12.9 busy); amortized hook plus hookless trampoline mode (now the default,
  `WINCE_EMU_GLOBAL_CODE_HOOK=1` restores the global hook) 22-34 busy versus
  13 busy with the global hook, ~6.5x more guest instructions per wall second
  than HEAD.
- The July "per-slice engine rebuild is 1.5%" figure only held for ~300 ms
  slices. Fixed per-slice cost (setup + hooks + snapshot + sync_out, 8-15 ms)
  was 3.6% / 14.9% / 25.5% of slice time at 302 / 143 / 65 ms average slices,
  and the cold translation cache each slice is not counted in it.
- Live-pump slices pass `instruction_limit = 1_000_000` to `emu_start`, which
  makes Unicorn install its own per-instruction counting hook and is the de
  facto round-robin tick (95% of slices in the fast runs ended at the count).
  `WINCE_EMU_LIVE_PUMP_RUN_SLICE_INSTRUCTIONS=0` removes it: iNavi busy slices
  reach 30-106 MIPS, but scheduling dynamics change (see KNOWN_BUGS.md), so
  the default stays 1M until the scheduler has a real time-based quantum.
- After the compute speedup wall time is dominated by guest-level waits, not
  compute: with unbounded slices iNavi runs ~600M instructions in ~20 s, then
  sits in waits of 15, 20, 25, 30 s (5 s growing steps) with no slices at all
  (104 of 127 s wall idle in one run); with the 1M count, 1 s slices starting
  at an import trap with ~2-6k guest instructions fill the same role. There is
  no centralized virtual scheduler clock. `TimerSystem` combines boot wall
  time with a sleep-driven virtual offset; coherent clock semantics and idle
  fast-forward remain unproven.
- Run-to-run scheduling is bimodal for the same build: `happyway_win` either
  gets 170-300 slices (fair round robin) or 1-7 slices (starved while iNavi
  runs). Single A/B runs of scheduling-sensitive metrics are therefore weak
  evidence; compare busy-slice MIPS and total guest instructions instead.
- Remote status/live snapshots are serviced in the main loop, including
  between rotations; framebuffer publication normally follows slices or
  scheduler changes. Import/file counters can be retained per-process CPU
  snapshots. Alternating frozen snapshots are not progress. Record capture
  timestamps and body/frame hashes alongside slice logs.
- The Jul-2026 release binary could not be rebuilt in an agent shell after the
  rustc 1.98.1 update: `unicorn-engine-sys` needs `LIBCLANG_PATH` and cmake
  symlink privilege (see AGENTS.md). Validation was run from a mirror build
  with the symlink step replaced by a copy: lib 576, bin 28, basic_subsystems
  130, coredll_dispatch 10, coredll_raw_gwe 495, coredll_raw_kernel 125,
  coredll_raw_memory_file 136, waveout 3, host_progs 2 all pass (the mirror
  needs `registry.reg`, `mounts.toml`, `serial_devices.json`, `host_progs/` in
  its root).

## 2026-07-04 Session Facts

- Map-gate root cause: escaped-callout recovery false-positived on the
  pre-push stop snapshot (a cross-thread visible-message callout copies its
  return pc from that same stop), so every host-delivered WM_SHOWWINDOW /
  WM_WINDOWPOSCHANGED / WM_ACTIVATE / WM_PAINT was consumed host-side ~1 ms
  after push without running the guest wndproc (`ce.gwe=debug` showed
  back-to-back push/escape pairs; 115+ per run). Fixed by gating recovery on
  a run slice completing after the newest push (`2fcc365f`); the same commit
  keeps staged callouts ahead of the run-entry ready-parked-wait rotation,
  which otherwise ping-pongs processes every ~2.6 ms without executing the
  staged slice.
- Quantum rotation is now unconditional at wall-clock quantum expiry
  (`dbaf4791`): the previous empty-receiver-queue gate let `happyway_win`
  busy-wait inside its WM_ACTIVATE handler (pc cycling `+0x28f8..0x2a18`,
  heap/message counters frozen) with one queued message it could not consume,
  pinning the CPU while iNavi's 43-message layout cascade starved.
- With real visible-message delivery, startup takes a different shape: the
  afxwnd42u children are created through actual dispatched messages, the
  splash progress bar advances to full width, and the run proceeds into
  `res\resmapi_800x480.bin` map-skin streaming (past the old
  InaviMainConfig.bin plateau). Post-fix runs show 0 escapes and 90+ genuine
  WNDPROC stub completions.
- Rotation churn could still starve a freshly staged visible callout (one
  hot core of silent main-loop rotation, no slices); all parked-process
  rotations now refuse to switch while a staged callout has not yet received
  a run slice (`265e751a`). With that veto, iNavi's 43-message child cascade
  drains completely and the skin decode advances past the prior stop
  (`resmapi` last_pos 1343058 -> 1415371+).
- The previously labeled command-flag region is libpng cleanup (corrected
  by the 2026-10-12 disassembly above), not a readiness condition.
  `happyway_win` runs a designed ~1 s retry cycle inside its wndproc: it
  polls `FindWindowW(NULL, "utis_parser")` (no such binary exists on the
  device image, so the poll fails forever on real hardware too), then
  self-posts msg 0x405 and Sleeps 1000 ms. Its permanently queued
  WM_ACTIVATE plus that cycle made every "pending message = priority"
  scheduler heuristic hand it the CPU.
- Scheduler fairness completed in three further fixes: priority parked
  handoff now requires a genuine wake event — completed send or signaled/
  timed-out wait — instead of any pending posted message (`814a6605`);
  the slice-end handoff records its rotation and every pre-run rotation
  branch defers to that decision until the chosen process actually starts a
  run attempt (`eb7065e1`, `c6f0552d`). Live verification: process slices
  went from 4700:37 (happyway:iNavi) to an exact 150:150 round-robin, and
  `resmapi_800x480.bin` decode advanced past 3.0 MB (previous best 1.4 MB)
  while the splash stays fully painted.

- Raw `GetOwnerProcess` now returns the current CE process id, and
  `GetCallerVMProcessId`/`GetDirectCallerProcessId` resolve through coredll
  ordinals with the same value (`c59d08ad`).
- Boot-time regression root cause: zero-byte marker files created on the SD
  image between 2026-06-11 and 2026-06-17 (`traffictest.txt`, `routetest.txt`,
  `soap.bin`, `oildata.dat`) enabled `happyway_win.exe` test modes
  (`traffictest`/`routetest` strings are embedded in that binary) and slowed
  boot from ~30 s to 20+ minutes in both single-shot and live-pump modes.
  Removing them restored full splash paint in ~90 s of a live run. Current
  builds do not re-create the files through splash/mapdata phases, so the
  probe-materialization bug that produced them appears already fixed.
- Slice profiling (`RUST_LOG=ce.slice_profile=debug`) shows the per-slice
  Unicorn engine rebuild is NOT the live-pump bottleneck: setup ~4 ms,
  hook install ~6 ms, blob read-back ~1 ms against ~1010 ms of guest
  execution per slice, with 25 blobs totalling ~43 MB. The dominant boot cost
  is genuine guest work: the hot PCs around `0x00339c00-0x00339da8` are a
  pixel-format conversion loop (RGB565/555 packing with `0xf81f`/`0x7c1f`
  transparency keys plus an alpha-plane pass), not a readiness spin.
- Live-pump handoff now applies CE quantum rotation: an active process that
  consumed its full host wall-clock slice rotates to a CPU-runnable parked
  process even while it keeps dirty visible windows, matching NK same-priority
  round-robin (`73bff641`). Previously a visible process with a dirty window
  and a runnable hidden helper starved the helper indefinitely (measured
  175:2 slice split over three minutes).
- The per-instruction code hook no longer clones the trampoline range vector
  on every executed instruction (`58ccb04d`).
- Debug snapshots published to the remote server can be minutes stale while
  the same process keeps the CPU, because publication happens on scheduler
  changes; use the run log or `ce.slice_profile`/`ce.quantum` debug targets
  for live-state truth.

## Current Snapshot

- Loader/runtime mapping reaches dynamic Unicorn DLL mapping with dependency
  loading, import patching, forwarders, rollback on failed loads, trampoline
  tracking, datafile/no-resolve handling, and CE-style attach/detach lifecycle
  behavior.
- Shell/resource behavior includes PE icon extraction, shell fallback icons,
  image lists, `DrawIconEx`, `SHGetFileInfo`, version-resource extraction, and
  several CE-compatible image-list validation and lifetime paths.
- File and storage behavior includes mounted host paths, volume-boundary checks,
  file attributes, find handles, change notifications, `CeGetFileNotificationInfo`,
  FSDMGR import shims, public AFS mount registration, synthetic disk metadata,
  sparse-sector backing, StoreMgr partition format no-ops, and selected
  FMD/file-lock/security descriptor surfaces.
- GWE/message behavior includes HWND/message-queue state, menu and message-box
  foundations, cross-thread send setup, timeout completion, nested reply paths,
  and scheduler wake integration for multiple wait classes.
- GDI/text/IME behavior covers many raw CE edge cases for DIBs, bitmaps,
  palettes, draw APIs, alpha/mask paths, selected font metrics, IME state, and
  TESTIME-backed dictionary behavior.
- CE `CreateDIBSection(DIB_PAL_COLORS)` now follows the GDIAPI helper's 8 bpp
  rule: 8 bpp DIBSections keep mutable color tables, while 1/2/4 bpp and
  truecolor requests fail during creation.
- Remote runtime behavior exposes status, frame capture, touch/key input,
  GPS/NMEA serial injection, debug handle snapshots, and audio/control WebSocket
  plumbing.
- Process-handle behavior includes opening launched process IDs as distinct
  handles plus raw `ReadProcessMemory`/`WriteProcessMemory` byte copies with
  CE-style handle validation and transferred-byte counts.
- Raw stack diagnostic behavior includes a minimal `GetCallStackSnapshot`
  surface for normal and extended frame buffers, skip handling, flag validation,
  and CE-style last-error reporting.
- Stub/raw no-feature validation now keeps visible receiver work aligned with
  the Unicorn runtime, including sent messages, queued visible messages, and
  dirty visible windows.
- Toolhelp process snapshots now expose the current process and launched child
  process entries through raw `THCreateSnapshot` memory blocks.
- Toolhelp thread snapshots now expose current and launched thread entries,
  owner process IDs, priorities, and state flags through raw `THCreateSnapshot`
  memory blocks.
- Toolhelp module snapshots now expose live loaded-module entries, image bases,
  sizes, refcounts, names, and guest paths through raw `THCreateSnapshot`
  memory blocks.
- Toolhelp heap snapshots now append live process heap lists and fixed heap
  allocation entries through raw `GetHeapSnapshot` memory blocks.
- Public `toolhelp.dll` imports now create/close snapshots, walk process,
  thread, module, heap-list, and heap-entry records, and read current-process
  memory through CE-shaped snapshot buffers.

## Earlier Runtime And Subsystem Facts

- The former no-hook crash at handle-like `pc=0x00001054` no longer reproduces
  after blocked scheduler contexts were removed from direct receiver-work
  eligibility. Debug handle output identifies `0x1054` as an event handle.
- On a successfully finalized `HookStop(_)`/`NativeDeadline` without a
  requested-PC stop, slice maintenance purges stale blocked-wait and parked
  `GetMessage` records for the executed thread. The tested ordinary/private
  entered-attempt errors retain their waits for diagnosis; a saved GetMessage
  wait must still be live in Kernel before
  the host idle-poll gate can use it. These narrow controls do not prove every
  wait/callback interleaving or full O-P1 closure.
- Host live-pump idle polling no longer forces short 100 ms CPU slices while an
  active guest context is runnable.
- Current live runs reach real iNavi splash/resource state, hidden map child
  windows, map/resource file loading, `UID1:` probing, and eventually `COM7:`
  GPS startup. The 2026-06-21 post-drive sample opened `COM7:`, drained queued
  remote NMEA into the serial RX buffer, and accepted the guest `$PUBX`
  configuration writes.
- Remote taps are accepted by the live server and now route through the main
  iNavi splash/window stack after stale visible-message cleanup.
- Remote live-pump servicing now drains queued remote-server controls from
  import traps, the scheduler block hook, and inline wait timeout paths. The
  2026-06-21 live run cleared `pendingControlMessages` to zero and posted a tap
  to splash window `0x00020008` as real `WM_LBUTTONDOWN`/`WM_MOUSEMOVE`
  messages.
- Escaped visible-message WndProc callouts are now retired before the
  visible-dispatch gate decides whether pending callbacks block new input. The
  live `happyway_win` stale `OrphanedVisibleMessage` case now clears, leaving
  `pending wndproc returns: 0`.
- Current live taps route to the main iNavi splash/window stack with accepted
  remote key/touch posts; the framebuffer still shows the real iNavi splash
  artwork, so the remaining issue is after message dispatch has resumed.
- Receiver/window teardown now removes terminated send ids from active send
  stacks. The latest live `happyway_win` receiver-terminated send still leaves
  a completed result for the sender, but `active_stacks=0` and
  `depth_threads=0`.
- Current 2026-06-21 live samples show the splash phase persists after
  touch/key input is accepted and routed; the app continues resource/window
  setup (`resmapi_800x480.bin` memory-backed opens and hidden `afxwnd42u` child
  creation).
- Remote/live scheduling now rotates away from an active process with no
  running guest thread and no queued visible receiver work when another parked
  process is runnable. The latest sample advances iNavi through additional
  map/resource loads and accepts touch/key WndProc callouts on the real splash
  window instead of sitting only in the hidden `happyway_win` helper slice.
- Live-pump run entry now also prioritizes ready parked waits before starting a
  hidden active slice. This matches the after-slice policy and covers the
  observed state where hidden `happyway_win` had no receiver work while a
  visible iNavi thread was parked on a ready sleep wait.
- Live-pump generic parked-process rotation now keeps a visible active process
  in front at both after-slice handoff and run entry. The 2026-06-21 fresh
  release run advanced beyond the earlier splash-only sample into real
  mapdata/SearchDB/mapinfo loading while keeping remote input accepted; the
  debug snapshot can still land on hidden `happyway_win` between slices.
- `YAS526B.dll` dump validation confirmed `MFS_IOControl` dispatches exact
  IOCTLs `0xb0000000`, `0xb0000004`, `0xb0000008`, `0xb000000c`, and
  `0xb0000010`. The virtual `MFS1:` shim now follows the dumped contract:
  writes validate/send selector-command input but do not mutate the next sample,
  and reads use input byte 0 as device selector plus byte 2 as count.
- Elevated `cargo flamegraph` startup samples still stop in the real
  `happyway_win.exe+0x7b56c` traffic/shared-memory fill loop; the live-pump
  visible-window readiness check no longer clones the full GWE window list on
  that hot path.
- `MultiByteToWideChar`/`WideCharToMultiByte` now skip host Windows conversion
  calls for zero-flag ASCII input under Korean ACP/UTF-8, preserving explicit
  length and null-terminated return counts.
- Raw `GetDeviceInformationByDeviceHandle` and
  `GetDeviceInformationByFileHandle` now resolve through coredll ordinals and
  write the CE `DEVMGR_DEVICE_INFORMATION` layout for live device handles,
  including legacy name, registry driver key, and `$device\...` name.
- Raw `FindFirstDevice` and `FindNextDevice` now resolve through coredll
  ordinals and enumerate live activated device handles with CE-style device
  search handles, including legacy-name, device-name, bus-name, parent, no
  match, and stale-handle behavior.
- Activated configured devices now capture registry `BusName`, expose it in
  CE `DEVMGR_DEVICE_INFORMATION` as `$bus\...`, and use that activation-time
  value for `DeviceSearchByBusName` wildcard matching.
- Current post-fix live device sample still has no open `COM7:` handle after
  queued remote GPS input; `queued_serial_bytes` grows because the guest has
  not opened the serial device in this run. `MFS1:` and `SMB1:` also remain
  unopened in the same sample.
- Fresh 2026-06-21 release run no longer logs mounted resource file opens as
  `ce.devices` "not configured" misses before falling back to host storage.
  `CreateFileW` now probes the device namespace only for names present in the
  configured device table; configured CE device names still open normally.
- Remote GPS is now explicitly routed to configured `COM7:` and live startup
  reaches that handle again. The 2026-06-21 release sample drained queued NMEA
  into `COM7:`, accepted `$PUBX` configuration writes, and returned GPS bytes
  through real guest `ReadFile` calls. The same run opened `MFS1:` and `SMB1:`
  and issued the dumped sensor IOCTL command values through the virtual devices.
- Serial `ReadFile` now models pending overlapped reads for empty serial RX:
  it records the caller buffer, resets/signals the supplied event, reports
  `ERROR_IO_PENDING`/`ERROR_IO_INCOMPLETE`, and completes through
  `GetOverlappedResult` once remote serial bytes arrive. Focused kernel and raw
  coredll tests cover the event wake and byte delivery path.
- Idle visible helper-process slices no longer keep CPU ownership ahead of a
  runnable visible parked UI process. The scheduler now applies the visible
  parked-process handoff outside live-pump-only paths; the 2026-06-21 release
  validation showed active execution returning to `iNavi.exe+0x317ea0` with
  `happyway_win` parked after the earlier `happyway_win.exe+0x47490` sample.
- The live code-hook stop path now uses the same larger visible parked-process
  predicate as the pending-timeslice decision. This covers helpers with a small
  visible window, such as `happyway_win`, instead of only stopping helpers with
  no visible surface.

## Recent Validation

- Targeted formatting and tests passed for the blocked-thread receiver-work
  slice, stale active-wait cleanup, host idle-slice guard, and related focused
  scheduler regressions.
- Release/live validation survived the prior crash window, served framebuffer
  snapshots, consumed remote touch/key input on the iNavi splash window, and
  confirmed remote GPS drains once the guest opens `COM7:`.
- Focused raw kernel ordinal validation covers duplicated process handles,
  shared process exit state, and process memory read/write copies.
- Focused raw kernel ordinal validation covers normal and extended
  `GetCallStackSnapshot` buffers plus skip and invalid-flag behavior.
- Focused raw kernel ordinal validation covers `THCreateSnapshot` success and
  invalid requested-process failure, thread-entry serialization, module-entry
  serialization, and `GetHeapSnapshot` heap-list and entry serialization.
- Focused raw kernel conversion validation covers Korean ACP conversion plus
  ASCII ACP fast-path sizing and explicit-length no-NUL behavior.
- Focused raw coredll/kernel validation covers device-information success for
  activated and file-open device handles plus null, short-buffer, and stale
  handle failures.
- Focused raw coredll/kernel validation covers `FindFirstDevice`/`FindNextDevice`
  device search output and cleanup semantics.
- Focused Unicorn validation covers the ready parked wait handoff and the
  hidden-helper live-pump handoff. Release build passed with existing warnings.
- Focused raw coredll/kernel validation covers activation-time `BusName`
  capture, `$bus\...` device-info output, and bus-name wildcard search.
- Focused device-name validation covers the configured-device precheck used by
  `CreateFileW`, including CE colon suffixes, `\\.\` prefixes, and ordinary
  mounted file paths. Release build with `--features unicorn` passed with
  existing warnings after cleaning the stale running executable.
- Focused serial validation covers remote GPS target selection, pending
  overlapped serial event signaling, and raw coredll
  `ReadFile`/`GetOverlappedResult` completion after remote NMEA arrives.
- Release/live driving with `--remote-server 192.168.0.39:8765` confirmed
  `queuedSerialBytes` dropped to zero after `COM7:` opened, followed by repeated
  `SerialRemoteDrain` and `ReadFile` records with NMEA payloads.
- Focused Unicorn scheduler validation covers the idle visible helper handoff
  to a visible parked process even when the handoff is not entered through the
  live-pump-only branch. Release build with `--features unicorn` passed with
  existing warnings.
- Full `cargo fmt --check` may still report unrelated pre-existing formatting
  drift in older files; treat new non-formatting whitespace findings as
  actionable.

## Working Rules

- Keep future entries concise and source-visible.
- Record completed behavior here only after code and validation support it.
- Move active work to `TODO.md`; move reproducible unresolved failures to
  `KNOWN_BUGS.md`.
