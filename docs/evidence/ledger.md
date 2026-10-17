# Evidence ledger

Status: evidence only. This file records what was observed, with run
conditions and limits. It does not define design; contracts live in the
[architecture anchor](../ARCHITECTURE.md) and sequencing in the
[plan](../PLAN.md). Other pages cite entries here (for example "ledger E-2")
instead of repeating the narrative. Add new evidence as a new entry; do not
copy it into design pages.

Conventions: "executed" means compiled and run with the result recorded here;
"staged" means a test design exists but was not run. Hashes are abbreviated
(`first8…last6`). Durations are guest-run wall seconds unless noted. An entry
never upgrades itself into acceptance: acceptance is defined only by the plan's
stage gates.

## E-1 Installed baseline BED2

- Executable `BED2F559…276E4F` completed a bounded 330.234 s GPS/tap run
  (exit 0, no watchdog). Its two-second host samples peaked at RSS
  582,623,232 B and private 595,992,576 B; these are sampled peaks, not a
  complete allocation breakdown.
- The frame stayed on the decorative iNavi splash. It is not a full CE boot or
  an interactive map, so it is a safe fallback, not an accepted runtime.
- The first observed splash frame was at app +4.358 s; the last observed
  image change was +24.752 s, and the terminal 800×480 frame was still the
  splash. The retained files snapshot contained 146 positive `COM7:` reads
  totaling 35,148 B; one delivered tap reached the guest DOWN handler
  0.397 s after HTTP ACK, without a frame change. These are one-run observations,
  not parsed-position, navigation responsiveness or controlled latency proof.
- Preserved with its symbols; the `pre_scheduler_core_1013` fallback is kept
  beside it (see [artifact lifecycle](../subsystems/artifact-lifecycle.md)).

## E-2 Persistent-owner 650 run (failed)

Build `65054E88…877F5` (first scheduler-core migration with a process-owned
native engine). Exact executable, matching PDB and source are preserved; the
source remains uncommitted in D: and is also recoverable from the verified
chat-project source archive recorded under E-11 (see
[DECISIONS D-004](../DECISIONS.md)).

| Fact | Value |
| --- | --- |
| Unit suite | 1,618 passed, 0 failed, 1 fixture ignored; build/feature/fmt/diff checks passed |
| Normal 330 s workload | host access violation `0xC0000005` at 129.108 s; no watchdog; empty stderr |
| Memory | peak RSS 1,662.59 MiB (1,743,351,808 B), peak private 1,663.40 MiB; RSS 373.5 MiB at +5 s, 1,662.6 MiB at +128 s |
| Crash site | WER RVA `0x85e757`, `tb_gen_code_mipsel`, translate-all.c:1764, TranslationBlock jump-reset initialization (GUID-matched symbols and disassembly) |
| Earlier mislabel | a subpage label came from the adjacent BED2 PDB and was wrong; always verify the PDB GUID+age actually loaded |
| Artifacts | no terminal PPM and no full trace were produced despite result filenames |

App-side observations from the same run: first decorative splash at +4.800 s;
last successful splash frame at +128.892 s (0.217 s before the crash); input
helper started 16.075 s after the app; the "+93 s" GPS feed was app +109.108 s;
16 GPS ACKs, but the only retained files-full response showed 56 zero-byte
COM7 reads, so positive guest consumption is unproven; the tap failed after the
app died. The splash is HWND `0x20008`, class `afx:10000:3:0:b5000:0`; authentic
caller `0x5237` (SP `0x7ffdf578`) was recovered, with no later `0x5237`/wParam
`0xa`, `0x56d0` or splash teardown in retained logs. Observer processes
27984/15496 were stopped by the parent around 02:30 and exited -1 without a
graceful finish record.

Limits: the memory growth correlates with the crash but does not establish
cause; no evidence implicates a particular shim or loader. A matching
1,027,645-byte Windows minidump was recovered on 2026-10-14 and copied beside
the already-preserved 650 EXE/PDB as
`evidence/legacy-target-release-20261013/wince_emulation_v3.scheduler_core_failed_1013.dmp`
(SHA-256 `DFD12E4351A5084326FA65FF49717D4E256040C6B2E5B5BBFB7CA872DC62A23F`).
Its existence updates the historical “no dump survives” limit; it has not
established E-2's cause.

## E-3 Translation-arena retention mechanism (source-confirmed)

- Pinned Unicorn 2.1.5, 64-bit TCG: default arena 1 GiB per engine; Windows
  reserves it and commits touched pages in roughly 4 MiB chunks.
- Hook deletion invalidates recorded TB regions but does not rewind arena
  allocation. A full TB flush resets the region cursor but does not decommit.
- Therefore persistent owners plus hook churn can retain translated-code
  commitment that fresh-engine destruction used to release.
- Status: mechanism read from source, not measured attribution of E-2.

## E-4 Stage 0 native-owner controls (executed, isolated fixture)

Six separately compiled controls: stable, repeated lease, repeated hook scope,
mapping churn (aligned 4 KiB topology), combined churn, real in-block churn.
2,048 counted attempts of 768 MIPS instructions each; actual timeout query 0;
checked final PC/counter; initial arena queried as 1,073,741,824 B in all six.

| Case family | Sampled retained RSS |
| --- | --- |
| stable, repeated lease, mapping-only | about 7 MiB |
| hook-containing (hook-only, combined, in-block) | about 243 MiB until owner close, then released |

Limits: one bounded owner workload; no production mapping reconciler, Task/FPU
transfer, backend stop/error matrix or app run; the fixture time includes
native timeout service, so it says nothing about application throughput or
cache speed; it does not identify which hook retains memory.

## E-5 Pre-init arena budget differential (executed, diagnostic)

Four fresh processes requested 8 MiB, default, 1 MiB and 2 MiB before native
initialization. 1 MiB and 2 MiB exited `0xC0000005` during the first size query
(matched RSDS/PDB: `tcg_out8`, RVA `0xA2D31`) and never returned a size.
8 MiB and default passed a four-instruction MIPS smoke (after correcting the
fixture's last-PC expectation to `0x100c`); actual sizes 8,388,608 B and
1,073,741,824 B. The pinned Windows 4 MiB commit handler's tail calculation is
a source defect candidate consistent with the small-reservation fault. No dump
gives the fault address. This is separate from the E-2 site.

## E-6 Commit-tail repair controls (executed, copied source, isolated)

- Copied-source bounds repair passed the 1/2/8 MiB/default tiny controls; an
  8 MiB distinct-PC probe committed second-half pages before any TB flush. The
  matching pristine handler AVed on 1/2 MiB and before tail observation.
- An in-handler telemetry variant stack-overflowed and is excluded.
- Win64 2x2 handler matrix: 24/24 owned controls passed; telemetry cells
  recorded the clipped second-half VEH request (fault offset 4,194,328;
  remaining/request 4,194,280 B) and a successful commit.
- A leaf-tail-jump thunk passed seven fresh-process small/tail controls with
  matching direct tail counters.
- A synthetic logical 4 MiB+1 final-byte touch passed twice: one-byte clipped
  commit, the next host page left free. TCG's guard prevents normal translation
  there, so this is not a translated-tail budget gate.
- Sequential leaf-thunk lifecycle: two runs closed/recreated eight owner pairs
  each, verified handler and mapping removal, and translated through the
  survivor's clipped tail.

Open: concurrent VEH/teardown with an in-flight callback, normal translated-page
boundaries, exact handler request/return in the full backend, x64 trampoline
unwind safety, a real-app budget. No production native source or cache setting
was changed.

## E-7 Full-backend Stage 2 controls (executed)

- Mapping/context control passed twice in owned release unit-test processes:
  three counted slices on the same engine, guest code loaded a newly mapped
  page, native F0 survived re-entry, the value persisted, the page was removed
  before the third slice; actual timeout query 0. One synthetic page/owner only.
- Expected red (preserved as failing controls): a counted stop with actual
  `Query::TIMEOUT` = 0 fabricates a host-wall record; a nonzero
  `READ_UNMAPPED` native fault returns Ok under a positive configured timeout.
- `ctl_get_timeout` returns the configured duration, not actual expiry.
- Staged and unexecuted: in-slice FPU wait→resume→exit; native wait/exit FPU.

The two failures above are the preserved **fresh-HEAD red baseline**, not the
current D: source result. In the sole D: checkout, the connected stop/error
fix built in the repo `target/` with `unicorn,trace,win32-desktop` release
features; four isolated final tests passed (count/no fabricated deadline,
nonzero `READ_UNMAPPED` hard failure, typed service handoff, main-loop
rotation). The final library/main test EXE SHA-256 values were
`8BC19FE0…9DCDC3` / `BA66B07F…38C54D`; exact source hashes, PIDs, stdout,
stderr and the private red test patch are in local
`evidence/stage2-e7-stop-20261013/receipt.md`. This proves only those controls
on the then-current dirty persistent-owner checkout. O-P1's complete reason
taxonomy, O-P4 fresh default, O-M1 guest-byte/view ownership, other feature
profiles and a bounded app run remain open. Later source edits require their own
validation; the E-7 executable hash does not identify them.

After outgoing file-view readback moved before PID rotation and stale typed
outcomes were cleared on preflight swaps, the sole D: checkout's serial full
release suite exited 0 with **1,623 passed, 0 failed, 3 ignored**. This was
before the later E-13 repaired-PC change. Its log
SHA-256 is `5DB8C9DF…AFA530C181B9`; the library test EXE SHA-256 is
`8543991F…EF72ADE1`. The exact command, full hashes and logs are in local
`evidence/stage2-e7-stop-20261013/post-order-and-retirement.receipt.md`.
This is a later result on dirty persistent-owner source, not fresh-engine
acceptance, a complete stop taxonomy or a bounded Windows CE app run.

Two earlier ignored isolated native probes used the focused-control library
EXE cited above. A
Box-backed `mem_map_ptr` slab survived guest stores, partial protection,
unmapping and new-owner remap (exit 0, 1 pass); this is native API viability,
not the production VM strategy. An earlier closed-owner Context fixture failed
at a terminal-PC/count assertion (`0x100c` versus `0x1010`, exit 101), after
its restored FPU/HI/LO/PC/T0 checks; the closure assertion was then unreached
(`native-probes.receipt.md`). After adjustment, the ignored fixture was run
separately under a 45-second watchdog and exited 101 in 0.02 seconds at its
intended final assertion: the current default does not close the native owner.
Its preceding guest store and native no-timeout checks passed. This is the
expected-red O-P4 gate, not a full-suite failure or fresh-owner acceptance;
the later raw logs and exact scope are in the post-order receipt above.

## E-8 Task and handle ownership baseline (executed, expected red)

Eight actual-Kernel controls in a source-matching copy mirror: one
close-source preservation pass, seven expected failures (alias priority/suspend/
exit coherence, creation owner, primary identity, trace-independent primary
lookup, checkpoint isolation). Two raw-API controls also failed as predicted:
`OpenThread` returns the existing public handle instead of a new alias;
`GetProcessIdOfThread` returned the selected PID 1 instead of child creation
PID 66. Total: 10 controls, 1 pass, 9 failures. Later assertions in several red
cases were not reached; last-public-handle and wait-pin controls are still to
add. This is a red baseline, not a passed gate.

## E-9 Hot-path throughput and slicing (2026-10-12, executed)

Two-minute live runs, `--desktop virtual`, clean SD image, same host, using the
`ce.slice_profile` fields `guest_insns`/`mips` (commit `fc8c8384`). Full
conditions and numbers: [PROGRESS.md](../../PROGRESS.md), 2026-10-12 section.

- Raw Unicorn MIPS loop on this host: about 3,200 MIPS unhooked, 1,900 with a
  trivial block hook, 400 with a trivial per-instruction hook.
- iNavi: 1.05 MIPS at the pre-change `HEAD` (a per-block hook that queried
  kernel/GWE state, about 15 µs per block); 7.3 average with polling amortized;
  22-34 MIPS in busy slices with hookless trampolines (13 with the global hook).
- Fixed per-slice cost: 3.6% / 14.9% / 25.5% of slice time at average slices of
  302 / 143 / 65 ms.
- `emu_start(count = 1_000_000)` is the de facto round-robin tick and installs
  Unicorn's per-instruction counting hook; count 0 reached 30-106 MIPS but
  exposed a respawn storm (with a 100 ms native timeout, repeated
  `CreateProcessW(happyway_win.exe)` in 3 of 4 runs, PIDs 67-82). A later
  session traced it to the live-pump `CreateProcessW` replay (child executed
  inside the parent's import hook before the return was committed), fixed it,
  and measured 84.56 busy-slice MIPS with count 0 ([PROGRESS.md](../../PROGRESS.md)).
- After the speedup, wall time was dominated by guest waits (15, 20, 25, 30 s in
  5 s steps, and 1 s idle slices at an import trap). A later session identified
  the growing 5/10/15/20 s sleeps as sensor-initialization retries (caller VA
  `0x000e69b8`, retry count x 5000 ms) and fixed the sensor command/packed-sample
  path ([PROGRESS.md](../../PROGRESS.md)). The 1 s import-trap idle slices have
  not been re-attributed.
- Scheduling is bimodal for one build (`happyway_win` 170-300 slices versus 1-7).
- Debug-endpoint counters stayed stale for minutes and are not a progress metric.

## E-10 Cross-owner same-VA/FPU control (2026-10-13, executed)

Two fresh, bounded release-test processes (PIDs 35920 and 28296, exits 0)
exercised two distinct `UnicornMips` native owners at the same guest VA
`0x20000000`. The parent and child retained independent page values
`0x11223344` / `0xa1b2c3d4` and F0 markers `0x7300` / `0x7400` over six
counted 20-instruction slices and four explicit selected/parked swaps. Guest
LW/MFC1 stores, owner, PC, F0 and page checks passed; actual native
`Query::TIMEOUT` was 0. The fixture source was `F90427EB…3C782` in a private
copy mirror; executable `76F53DF9…3B0` and matching PDB `F4F023F6…B9C3` were
preserved. No watchdog fired; observed RSS was about 99 MiB.

Limits: synthetic PEs and test-created pages, explicit slice selection and
manual fulfillment of queued launch; no real Kernel `VirtualAlloc`, native
wait/exit continuation, long churn or application frame. This ran against the
experimental persistent-owner code, so it is a **parallel Stage 8 diagnostic**
of the same-VA/FPU shape, not acceptance of the Stage 2 fresh-engine VM gate
under [D-003](../DECISIONS.md). A Kernel-owned `VirtualAlloc` fault/release
fixture is staged and reviewed but has not compiled or run.

## E-11 Task lifetime and alias-wake baseline (2026-10-13, executed, expected red)

Three no-Unicorn release controls in a private source-matching copy ran in
one bounded owned process (PID 2960, 0.01 s, exit 101): **0 passed, 3 failed
at the intended assertions**. Closing the final public handle lost an
unsignaled live task snapshot; exiting through one alias queued zero rather
than one waiter indexed on a sibling alias; and a registered wait did not
retain an exited target after both public aliases closed. The test-only
fixture was `5B450F24…E7DC`; the compiled test executable was
`84557AD5…D2186`. No D production source or installed binary changed.

These add to the earlier ten controls in E-8 (one pass, nine expected red),
giving thirteen distinct Task/alias controls with one pass and twelve red.
This is not full wait acquisition, genuine native worker exit, stale-key
rejection, failed-publish cleanup, clone isolation or Stage 3 acceptance.

## E-12 Persistent-owner full-backend FPU/churn diagnostic (2026-10-13, executed)

Two bounded release-test processes used a synthetic MIPS PE and the full
`UnicornMips` backend with the experimental persistent owner. The stable
case completed 128 counted slices with guest loop/page progress (PID 33408,
2.243 s, 70,832,128 B peak RSS), but final F0 was 0 instead of the seeded
`0x6300`. The map/unmap attempt (PID 35672, 0.214 s, 55,222,272 B peak RSS)
failed at its **first mapped phase**; 64 cycles were not completed. Both were
test failures, not acceptance passes.

A short `cfg(test)`-instrumented replay (PID 35436, 0.509 s) established the
loss path: the native context was captured with `(TID 1, PC 0x402020)` and
F0 `0x6300`, but post-run repair saved scalar PC `0x401078`. The next
`restore_if_exact` missed that key and the blank/GPR-only fallback left F0
at 0. The next exact restore preserved 0; the first failing safe boundary
still read the mapped guest page correctly. Executable/PDB pairs and full
logs are archived and stream-hash verified in the separate ChatGPT project
workspace. This proves the pre-repair key/post-repair PC mismatch on this
path, **not** all stop paths or a causal account of the 650 app crash.

Under [D-003](../DECISIONS.md), these pre-repair E-12 runs are Stage 8
diagnostics, not the fresh-engine Stage 2 VM/FPU gate or a scheduler-core
acceptance result. The page mapping used test-owned RAM, not guest
`VirtualAlloc`. The lifetime-independent scalar loss was subsequently
reproduced and repaired under the focused E-13 control below.

## E-13 Repaired-PC fresh-owner FPU continuation (2026-10-13, executed)

The deterministic full-backend fixture ran two native NOPs in a synthetic
trampoline, captured raw context, used the production restart-PC helper,
closed the original native owner, then resumed on a fresh owner through the
mismatched-key scalar fallback. It seeded full-width F0 and FCSR, checked
HI/LO and PC, and verified host registers plus guest `mfc1`/`cfc1` readback.
The native attempt used a 100 ms timeout and two-instruction count; this is
not an application run.

Before repair the explicitly run ignored control failed as expected (exit 1,
one failure): after PC and HI/LO passed, observed `(F0,FCSR)` was `(0,0)`
instead of `(1234605616436508552,1)`. Red executable SHA-256
`D8BB0B21…4DE2B`; pre-repair `unicorn.rs`/`cpu_mips.rs` hashes
`E3381CF7…3F39B2` / `DF111899…1AE962`. The earlier bindgen failure without
`LIBCLANG_PATH` was setup-only, not this red result.

The repair extends `MipsGuestContext` with 64-bit F0-F31 and FCSR, uses
checked primary/post-repair capture and checked scalar restore onto a blank
engine when the native key differs, and preserves primary native-result error
precedence. The final focused release run passed (exit 0, one passed, 690
filtered), including guest FPU readback. Two checked-context controls passed
on that same final executable (two passed, 689 filtered): valid FCSR `0x1005`
round-tripped; enabled pending cause `0x1080` returned `UC_ERR_EXCEPTION`.
Final executable SHA-256 `66B6319E…154711`; final `unicorn.rs`/`cpu_mips.rs`
hashes `9914D621…6F041C` / `708E25D2…0F25C62`. Format and scoped diff
checks passed. Full commands, hashes and preserved binaries are in local
`evidence/e12-repaired-pc-20261013.receipt.md`.

This proves the repaired-PC mismatched-key loss is fixed in this focused
fresh-owner fixture, not closed-owner exact-key native Context portability,
guest `VirtualAlloc` byte coherence, callback/o32 floating returns,
CP0_STATUS FP-mode transfer, full scheduler behavior or iNavi readiness.
Legacy in-hook register helpers still suppress some read/write errors. Full
release/feature suites and bounded app runs were separate gates at that point.

Post-E-12 serial validation on the same source and final focused-test
executable passed the full `unicorn,trace,win32-desktop` release suite
(**1,626 passed, 0 failed, 3 ignored**), Unicorn-only and
Unicorn+desktop/no-trace release checks, and the no-feature suite
(**1,239 passed, 0 failed, 1 ignored**). All exited 0; `cargo fmt --all --
--check` and `git diff --check` also passed. Full-suite log SHA-256
`BC8FF5FB…FEBDA2`; no-feature log `2C2CEFC2…3F0F7C`.
The exact commands, full hashes and logs are in local
`evidence/e12-repaired-pc-green-20261013/post-e12-validation.receipt.md`.
These results validate those profiles on dirty persistent-owner source, not
fresh-engine default ownership, closed-owner exact-key native Context,
O-M1 private/shared byte coherence, complete O-P1 classification, Stage 2/3,
bounded live application stability or map/input readiness. The isolated
ignored closed-owner/pointer probes were not part of the default suite.

## E-14 Two-process same-VA file-view ownership (2026-10-13, expected red)

The ignored Stage 2 O-M1 control in `src/emulator/unicorn.rs` ran five real
MIPS load/increment/store instructions on each of two process CPUs. Two
distinct pagefile-backed `FileMappingObject` views were injected at one guest
VA under PID 1 and a queued child PID; injection is necessary because the
current global virtual allocator cannot expose the same VA through public
`MapViewOfFile`. The post-format focused release run exited **101** at its
intended ownership assertion (one failed, 691 filtered), after both native
slices completed. It is not a fixture setup error.

In hex, the observed `(parent T1, parent mapping after parent, child mapping
after parent, child T1, parent mapping after child, child mapping after child)`
was `(55667789, 55667789, 55667788, 55667789, 55667789, 55667788)`; required
was `(11223345, 11223345, 55667788, 55667789, 11223345, 55667789)`.
All-view native seeding gave the parent the child's bytes; base-only readback
then applied the child's write to the parent mapping, leaving the child's
canonical bytes unchanged. The test was left ignored at this red baseline;
E-16 records the later checked fix that unignored it.

Post-format SHA-256: `unicorn.rs`
`E6E27273514557E271AC1A73C7DE9ECD1E1078AD8756EEDD1A021522B3E16FB0`;
focused EXE `7C630A2ADF9D59A99C104C15E5CB24092DA11270141EABE90CF8B7F09CC9B826`;
raw log `799FF159EB2FD1F0095A4DC1E41233043E8F618B71681B8D82BFB9C17CA61F42`.
The exact command, paths, file sizes and full raw output are in local
`evidence/stage2-o-m1-same-va-20261013/expected-red.receipt.md`. This proves
only the injected same-VA native-view collision: public same-VA MapView,
private `VirtualAlloc` readback, fresh owner closure, same-mapping aliases,
file flush/unmap, and application readiness remain open.
This is the historical red baseline; the same fixture was later unignored and
passed on the owner-qualified native-view source recorded in E-16.

## E-15 Narrow O-P1 native stop taxonomy (2026-10-13, executed)

On the sole D: source, `NativeSliceOutcome::UnknownNormalStop` replaced the
unproven successful `ServiceBoundary` residual. A positive configured count
and native `Ok` are not an independent count-exhaustion witness. The scheduler
block hook now records `HookStop(HostWallBudget)` only after an elapsed host
budget, a safe restart PC, successful checked PC write and successful
`emu_stop` request. Its failed control operations instead return
`BackendError` after primary native result/query/context precedence; the
failure precedence is unit-tested, but real PC-write/stop-failure injection
has not run. Six other host-wall debug-record writers remain untyped and
normally classify as `UnknownNormalStop`; restart-PC repair still reads the
legacy debug marker. Successful Unknown, HookStop and NativeDeadline outcomes
remain eligible for the existing service handoff, so the one-million-
instruction tick and rotation policy were not changed.

Seven focused release controls passed individually on the first O-P1 source
(one pass in
each run): actual MIPS host-budget hook stop with `Query::TIMEOUT = 0`,
hook-control error precedence, counted unknown boundary, unmapped native
error, explicit PC stop, actual native deadline with `Query::TIMEOUT = 1`,
and service-boundary handoff policy. Format and diff checks passed. That source
SHA-256 was `5CCAD8FF…92E5` for `types.rs` and `F1103483…46C73` for
`unicorn.rs`; focused executable SHA-256 was `482BB864…F3A55E`.
Commands, full hashes and raw logs are in local
`evidence/op1-stop-taxonomy-20261013/focused-controls.receipt.md`.

A subsequent source review found that a failed hook-control operation could
still be masked by the decoded guest-exit branch. The decoded-exit readiness
predicate now requires the hook-control-error slot to be empty in both
branches. On this follow-up source, the two `host_budget` focused controls
passed (2 passed, 0 failed); `unicorn.rs` SHA-256 was `CD693878…25F8` and
the focused executable was `A64D52D4…7F79B`. The exact one-line change,
command and hashes are in local
`evidence/op1-stop-taxonomy-20261013/encoded-exit-mask.receipt.md`. No test
injects a real native PC-write/stop failure together with an encoded exit;
this remains an end-to-end gap.

The final follow-up source then passed the complete
`unicorn,trace,win32-desktop` release suite: **1,628 passed, 0 failed,
4 ignored**. The fourth ignored control is the expected-red E-14 same-VA
fixture; it did not turn green. Raw log SHA-256
`8946D0736B3B0D8816C0017B7B8DCDC24B436EBBFE80A7CEBA559DE1FB51E041`
is local `evidence/op1-stop-taxonomy-20261013/full-release-suite-final.log`.
This supersedes the earlier E-13 full-suite count as the latest release
result, but does not convert any ignored gate into a pass.
Both no-trace release checks then exited 0 on the same source:
`--features unicorn` (log SHA-256 `E2EE30F4…EE67E`) and
`--features unicorn,win32-desktop` (`5CFDDB76…AAA95`). The no-feature
`cargo test --locked -j1` suite passed **1,239 passed, 0 failed, 1 ignored**
(log SHA-256 `4F448F59…51560`). Raw logs are local
`evidence/op1-stop-taxonomy-20261013/{check-unicorn-final.log,check-win32-final.log,no-feature-suite-final.log}`.
The final-source identity, commands, full hashes and serial-run scope are in
local `evidence/op1-stop-taxonomy-20261013/post-op1-validation.receipt.md`.
These results apply to `types.rs` SHA-256 `5CCAD8FF…92E5` and `unicorn.rs`
SHA-256 `CD693878…25F8`, before any later O-M1 memory edits.

This is focused O-P1 progress, not a complete classification or Stage 2
acceptance. Native count-stop witness, remaining hook reasons, typed
restart-PC repair, fresh-owner/VM gates and a bounded live workload remain
separate gates. The earlier E-13 suite counts are
historical and must not be attributed to this O-P1 source.

## E-16 Owner-qualified native file-view sync (2026-10-13, narrow green)

The final O-P1 source reproduced E-14's ignored red after both real MIPS
slices: observed hex tuple `(55667789,55667789,55667788,55667789,55667789,55667788)`
versus required `(11223345,11223345,55667788,55667789,11223345,55667789)`.
The O-M1 edit captured the outgoing native process owner before view seeding,
seeded only that owner's views, and retained exact mapping-handle/view identity
for native readback. It rejects duplicate owner/base or stale views and fails
closed for an ambiguous global lazy seed; a deferred unmapped write requires
the exact canonical seed, while mapped native write errors propagate. The
immediate CreateProcess hook also latches launch/view-sync errors and requests
a checked stop, but no end-to-end injected CreateProcess sync failure ran.

On O-M1 `unicorn.rs` SHA-256
`0467E73EB55479681B1DC9BC74E464EEE65F051B4DD32FC131DC7C55E3DF8D89`,
the formerly ignored E-14 two-PID native control passed as a default test;
two new closed-live exact-readback/ambiguous-lazy controls passed, as did the
outgoing-before-rotation regression. The full release suite passed
**1,631 passed, 0 failed, 3 ignored** (log SHA-256 `BC8A085A…CB302C`).
Unicorn-only and Unicorn+desktop/no-trace release checks exited 0; the
no-feature suite passed **1,239 passed, 0 failed, 1 ignored**. Exact commands,
source/executable hashes, all focused/broad raw logs and limits are in local
`evidence/stage2-o-m1-same-va-20261013/owner-qualified-native-view.receipt.md`
(receipt SHA-256 `10939F07…4D14E`). These results belong to that frozen O-M1
source, not any later source revision.

The fixture still injects views at an already mapped VA because public
`MapViewOfFile` uses a global virtual allocator and cannot create this
collision. The lazy control proves safe rejection, not a per-owner lazy
namespace or native fault repair. Public same-VA allocation, private
guest-written `VirtualAlloc` persistence across fresh owners, same-mapping
aliases, closed-handle flush/unmap, `VirtualQueryEx` owner resolution,
O-P4, Stage 2 and bounded live application readiness remain open.

## E-17 Visible parked-peer yield attribution (2026-10-13, narrow green)

After the frozen E-16 O-M1 source, the hookless live-pump scheduler block hook
gained `HookStop(VisibleParkedPeerYield)` for a visible, runnable parked peer
only after it selected a safe restart PC and both the checked native PC write
and `emu_stop` request succeeded. Control-transfer failures use the existing
backend-error slot. This yield creates no host-wall debug marker and does not
claim a count limit, native deadline or remote-input cause. The scheduler's
successful service-handoff policy and one-million-instruction count are
unchanged.

The real-backend fixture directly constructs a visible resumable peer and
bypasses only preflight rotation so the parent takes a native slice. It passed
three serial focused runs with `Query::TIMEOUT = 0`, typed outcome, native and
saved safe PC, no host-wall marker and peer handoff. Seven nearby focused
controls also passed. No failing Unicorn PC-write/stop API call was injected;
the peer fixture is not a public CreateProcess test.

On frozen `types.rs` SHA-256 `6EA882B5…D14DE` and `unicorn.rs` SHA-256
`F1D45245…FD38`, the full release suite passed **1,632 passed, 0 failed,
3 ignored** (raw log SHA-256 `6BD9C340…3DA2`). Unicorn-only and
Unicorn+desktop/no-trace release checks exited 0; the no-feature suite passed
**1,239 passed, 0 failed, 1 ignored**. Exact commands, full source/binary/log
hashes and limits are in local
`evidence/op1-visible-peer-20261013/post-visible-peer-validation.receipt.md`
(receipt SHA-256 `0E270930…BFE03`); focused repetitions are in the same
directory's `focused-controls.receipt.md`. These totals do not transfer to
later source edits.

Global and sparse hook writers still lack independent typed provenance; an
actual native count-exhaustion witness and typed restart-PC repair remain
open. O-P1 is incomplete, as are O-P4, public same-VA/lazy/private O-M1,
Stage 2 and bounded live-app acceptance.

## E-18 Private virtual bytes across fresh owner (2026-10-13, expected red)

The ignored `native_private_virtual_write_survives_closed_owner_next_slice`
fixture uses the internal `MemorySystem::virtual_alloc` API to reserve and
commit one private page, seeded with `0x11223344`. Five real MIPS instructions
load, increment and store the word. The first saved PC, T1=`0x11223345`,
normal native stop and no memory fault passed; a read-only test helper also
verified `0x11223345` in the retained native owner's RAM. A separate
`cfg(test)` helper then closed that idle owner and its blank context/ledger,
retaining the CPU snapshot only. This simulates one fresh-owner boundary for
the fixture; production still retains its idle owner by default.

One real MIPS load on the new native owner advanced the expected PC with
`UnknownNormalStop` and no fault, but read `0x11223344` instead of the guest-
written `0x11223345`. The explicitly run ignored release test exited 101 at
this final byte assertion (one expected failure, 697 filtered), not during
setup, native Context restore or PC handling. The new owner was seeded from
unchanged private `VirtualAllocation.initial_bytes`; a CPU-only Context
cannot carry the missing RAM bytes. Frozen test-only source SHA-256:
`native_engine.rs` `32339A6C…106FC`, `unicorn.rs` `9027D0DC…45E53`;
raw final log SHA-256 `68D7DB5A…64EBB`. Exact command, full hashes,
observations and limits are in local
`evidence/stage2-o-m1-private-fresh-20261013/expected-red.receipt.md`
(receipt SHA-256 `B9657C08…691D7`). This source is later than E-17; its
default full release suite subsequently passed **1,632 passed, 0 failed,
4 ignored** (raw log SHA-256 `50310507…D622`). The fourth ignored control
is E-18 itself; the explicit expected-red result is not counted as a pass.
This is a distinct later test-only source from E-17's 1,632/0/3 result.
Unicorn-only and Unicorn+desktop/no-trace release checks exited 0 (logs
`check-unicorn-final.log` SHA-256 `C419E2C8…605C3` and
`check-unicorn-win32-final.log` `6035F983…A107`). The no-feature suite passed
**1,239 passed, 0 failed, 1 ignored** (log
`default-no-feature-suite-final.log` SHA-256 `28075D1B…8506C`). These logs
are local beside the E-18 receipt. The ignored E-18 control remains expected
red when run explicitly; these green default/profile results do not accept
fresh-owner private-byte coherence or Stage 2. Final-source identity, exact
commands, full hashes and format/diff checks are in local
`evidence/stage2-o-m1-private-fresh-20261013/post-e18-validation.receipt.md`
(receipt SHA-256 `EA66CA0F…7F630`).

The fixture does not call public COREDLL `VirtualAlloc`, prove cross-process
same-VA isolation or shared views, or enable the fresh-owner production
policy. O-M1 private-page readback/authoritative backing, O-P4 closed-owner
default, Stage 2 and a bounded live app remain open. Retaining the old owner
or suppressing failed native reads would not satisfy this control.

## E-19 Global code-hook deferred stop reasons (2026-10-13, narrow green)

The global per-instruction code hook now latches the first reason it observes:
`RemoteControlService` when it drains queued remote control, or
`HostWallBudget` after an elapsed host limit. The pending reason travels to
one of three safe-PC completion paths (JR thunk, trampoline origin or ordinary
safe instruction). The hook publishes `HookStop(reason)` only after checked
native PC write and `emu_stop` both succeed; control-transfer failure records
a backend error instead. A debug host-wall record can accompany either
reason, so that marker alone is not evidence of actual host-budget expiry.

Two real-backend focused controls passed together in three serial runs
(2 passed, 0 failed each). Both checked typed reason, saved/native safe PC,
and actual native `Query::TIMEOUT = 0`; the remote control used a test-queued
message and the host-budget case used a paused framebuffer service point.
This proves the exercised straight-line/deferred-stop shapes, not an actual
network delivery, every branch/delay-slot shape, or a fault-injected PC-write/
stop failure. Focused raw logs SHA-256 were `63166691…E918`,
`CAE66AFD…8362` and `976F8269…6E94` in local
`evidence/op1-global-code-hook-20261013/`.

On frozen `types.rs` SHA-256 `00856084…80521` and `unicorn.rs` SHA-256
`6560E1E3…A0F2`, the full release suite passed **1,634 passed, 0 failed,
4 ignored** (raw log SHA-256 `B56F3E54…CE30`). The fourth ignored control is
E-18's private-byte expected red, not a pass. Unicorn-only and
Unicorn+desktop/no-trace release checks exited 0 (raw log SHA-256
`8458A187…3943` / `FCF30D1A…BC61`); the no-feature suite passed
**1,239 passed, 0 failed, 1 ignored** (log `3A25F566…07C1`). An explicit
rerun of E-18 on this source still exited 101 at the intended stale-byte
assertion (`0x11223345` written, `0x11223344` read; raw log SHA-256
`70AE4B88…206A`). That deliberate ignored-test run is not a normal-suite
failure. All logs are local in `evidence/op1-global-code-hook-20261013/`;
exact commands, full source/log hashes and limits are in
`evidence/op1-global-code-hook-20261013/post-global-stop-validation.receipt.md`
(receipt SHA-256 `C9EE9661…5FB3`). E-16's injected same-VA green and E-18's
private-byte red remain separate; this O-P1 slice does not change production
fresh-owner policy or close Stage 2.

The hookless/sparse remote and ready-peer paths, native count-exhaustion
witness, typed restart-PC repair, O-P4/O-M1 and bounded live-app gates remain
open. The first reason observed by this hook wins when both arise before
the safe stop; the type is not a post-hoc inference from elapsed time or
proof of which real-world condition happened first.

## E-20 Process birth and private VA owner controls (2026-10-13, expected red)

Two new ignored, test-only controls expose distinct Stage 2 ownership gaps.
`pending_child_birth_survives_last_public_process_handle_close` queues a
child, closes its last public process handle while its pending launch and
primary thread handle remain, then expects a PID lookup to reopen it. The
focused run exited 1 at `open_process_handle(child_pid) == None`: process
birth is still discoverable only through public handle aliases. The other
control, `public_private_virtual_alloc_and_query_follow_process_rotation`,
uses public COREDLL `VirtualAlloc` and `CreateProcessW`; its focused run
exited 1 because the selected child could not allocate the same private VA
(`0x52000000`) already held by the parent. Later parent-free and both query
assertions were not reached. Neither test executes the child or validates
native private bytes. They remain ignored in default suites, not accepted.

On `kernel.rs` SHA-256 `78B2A04E…B0A6B` and `coredll.rs` SHA-256
`D2020A00…F9926558`, the default release library suite passed **697 passed,
0 failed, 5 ignored**; the full release suite exited 0 with **1,634 passed,
0 failed, 6 ignored** across binaries. One additional ignored binary requires
the external eVC4 MIPSII toolchain. The focused reds were each one failed,
zero passed. Raw logs were not saved for E-20; commands, full source hashes,
observed failures and limits are in local
`evidence/e20-process-vm-owner-20261013.receipt.md` (receipt SHA-256
`449B6FD9…7CFA86`). A first `VirtualAllocEx`
draft stopped at an unresolved static ordinal and is not the same-VA red.
The target EXE directly imports `VirtualAlloc`, `VirtualFree`, and three file
mapping functions; the ROM Ex-family export-index gap is separate, and
dynamic imports are not ruled out. E-18 private-byte coherence, O-P4 fresh
default, full O-M1, Stage 2, and live iNavi acceptance all remain open.

## E-21 Canonical process birth and alias lifetime (2026-10-13, narrow green)

The E-20 pending-child last-handle control is now unignored and green.
`HandleTable` owns one birth record for boot PID1 and each child, independent
of public process-handle aliases. Public aliases identify a PID/incarnation;
exit, signal, debugger and execution lifetime live in that record. Five
focused release controls each passed 1/0: pending-child PID reopen after last
alias close, exit/wake after original-handle close, terminated PID external
reopen prevention, selected-child pseudo termination signaling aliases, and
completed-birth final-alias retirement/distinct rebirth. A registered wait is
removed before the test asserts final retirement; Stage 3 wait-target pins
are not claimed. `TerminateProcess` requests/signals, while actual child
completion releases the execution pin. An independent review found no
remaining narrow-patch blocker after the termination and test corrections.

On `object.rs` SHA-256 `C65F7125…CF46D0` and `kernel.rs` SHA-256
`D893BC67…2D176`, the final full release suite passed **1,639 passed,
0 failed, 5 ignored** across binaries. The no-feature suite passed **1,244
passed, 0 failed, 2 ignored** after an initial access-denied debug lock was
retried with permission; both no-trace release feature checks exited 0. No
raw logs were saved for E-21. Exact commands, full hashes, focused cases,
permission limitation and source boundaries are in local
`evidence/e21-process-birth-20261013.receipt.md` (receipt SHA-256
`5F9F16B6…A0A681`). Explicit E-18 and public same-VA E-20 reruns on this
source still exited 1 at their intended stale-byte (`0x11223344` versus
`0x11223345`) and child-allocation-null assertions, respectively. The
selected-process compatibility snapshot, worker
sweep, checked guest-visible allocation failure, wait pins and clone lineage
remain open. E-20's *other* red, public same-VA private allocation, remains
ignored; E-18 private bytes, O-P4 fresh default, Stage 2 and bounded iNavi
acceptance also remain open. No live app run occurred on this source.

## E-22 VM identity metadata and pre-live preparation (2026-10-14, narrow green)

`ProcessBirthKey` is now read from the E-21 canonical birth directory for
the selected process; an unregistered selected PID has no VM owner key.
`MemorySystem::virtual_alloc` records this key, a checked allocation
incarnation not reused within one `MemorySystem` lineage, and an explicit
producer kind. COREDLL marks private, Ex-compat, file-view, locale-view,
Toolhelp-snapshot and shared-compat allocations separately. Four focused
release metadata tests passed in three filters (2/2, 1/1, 1/1); their
terminal output was observed but not separately saved as raw logs. The four
source SHA-256 prefixes were `object.rs` `4F5982D9`, `kernel.rs`
`5F056945`, `memory.rs` `C597DFC0`, `coredll.rs` `AA4529C1`.

On the later combined sparse-stop and VM-metadata source, the full release
suite passed **1,647 passed, 0 failed, 5 ignored** and the no-feature suite
passed **1,248 passed, 0 failed, 2 ignored**. Raw logs are local
`evidence/op1-sparse-stop-20261014/full-release-suite-attempt2.log` and
`evidence/op1-sparse-stop-20261014/no-feature-suite.log`. Explicit ignored
gate reruns remained expected red: E-18 guest MIPS wrote `0x11223345`, but
the fresh owner loaded stale `0x11223344`; E-20 public child same-VA
`VirtualAlloc` returned `Handle(0)`. Their raw outputs are local
`evidence/op1-sparse-stop-20261014/e18-private-fresh-expected-red.log` and
`evidence/op1-sparse-stop-20261014/e20-private-va-expected-red.log`.

This slice leaves the global base-keyed VA directory, native copy-seeded
bytes, reserve-only native behavior, Ex target semantics and derived-clone
lineage unchanged. In particular, a cloned `MemorySystem` can later mint
the same incarnation under its copied birth key; it is not a machine-unique
identity. No slab backing, public two-PID private isolation, O-P4 fresh
default, Stage 2 or live-app acceptance is claimed.

Separately, user-authorized ZIP backups beside the three original mounted
data folders were created and verified before another guest run; exact
archive hashes and cleanup of one invalid partial ZIP are in local
`evidence/e22-pre-live-mounted-data-20261013.receipt.md`. The bounded
read/write script `scripts/inavi_bounded_e22.ps1` was then run against the
original mounts. Local `evidence/e22-live-20261013T154435Z/result.json`
records emulator exit 0 at its configured 330 s limit (330.502 s elapsed),
no monitor trip, no stderr CPU-exception/PC-zero text, and a terminal frame.
The observer completed its 420 s horizon and saved four changed frames; its
last observed and terminal images remain iNavi splash, not the map. A newly
constructed raw-NMEA feed sent 16 once-per-second HTTP requests starting at
input-ready +93 s, plus a tap at +153 s; all were accepted by the remote API.
API ACKs alone would not prove guest delivery, but retained
`observer-captures/files-full-0065.txt` records positive `COM7:` `ReadFile`
transfers with the injected `$GPRMC` bytes (174/348 bytes on handle 5204).
`observer-captures/messages-0165.txt` records tap down/up delivery and
window-procedure returns. Neither trace establishes parsed GPS position,
touch-driven navigation, or map readiness. The earlier exact NMEA payload
and frequency were not retained, so this is not an input-matched BED2
comparison. `mounted-changes.jsonl` reports two directory and two file
timestamp changes; both file lengths and SHA-256 values were unchanged, with
no added or removed entries. The originals and adjacent verified ZIP backups
remain in place. Bounded survival does not close E-18, E-20, Stage 2 or
splash-to-map acceptance.

Memory, from the same run's `memory.jsonl` (two-second host samples, PID
43456):
- Working set was 888 MiB at 41 s, 1,009 MiB at 60 s and 1,519 MiB at
  328.6 s.
- Sampled peaks were 1,543.9 MiB working set and 1,554.4 MiB private bytes.

That is about 2.7× BED2's sampled peak (E-1: 582,623,232 B RSS) and close to
the failed 650 run's 1,662.59 MiB (E-2). This run survived 330 s, but the
source still keeps the persistent native owner by default, and the growth
curve is consistent with the E-3 arena-retention mechanism. This is a
sampled correlation, not attribution. It is the memory baseline that the
fresh-default lifetime (O-P4) must be measured against.

## E-23 owner-qualified private-VA metadata (2026-10-14, default suites green)

On source `4a1a9bfb9898a9f2a251212d6120794970266692`, private virtual
allocation metadata is keyed by `(ProcessBirthKey, base)`; allocation
lookup, release, query visibility and pointer probes now reject another
process's private region. Owner cursor metadata was added, but its global
floor and a global overlap fence still prevent two live private allocations
at the same guest VA because the native page map is VA-only. Nonprivate
legacy producers retain global compatibility visibility. The known Ex
ordinal arms resolve a target and reject foreign processes, while the
default COREDLL export table does not publish those arms.

Two focused release controls passed (2/2 total), including the public raw
COREDLL self/foreign Ex-arm checks; exact commands and source identity are
in local `evidence/stage2-e23-owner-va-20261013/focused-owner-va.md`.
The same source passed the full release Unicorn/trace/desktop suite with
**1,649 passed, 0 failed, 5 ignored**, and the no-feature suite with
**1,250 passed, 0 failed, 2 ignored**. Raw logs are local
`evidence/stage2-e23-owner-va-20261013/full-release.log` and
`no-feature.log`. The two explicitly run ignored gates remain expected red:
E-18's fresh owner read stale `0x11223344` after guest-written `0x11223345`,
and E-20's public child same-VA `VirtualAlloc` returned `Handle(0)` at the
native overlap fence. Their logs are local `e18-expected-red.log` and
`e20-expected-red.log` in the same evidence folder. No O-P4 fresh default,
public remote Ex operation, Stage 2 or application acceptance is claimed.

Post-suite independent review found a legacy ownership escape on this E-23
snapshot: cross-owner `VirtualFree` can erase a parent's `FileViewLegacy`
metadata, and base-only `UnmapViewOfFile` can commit/remove that parent's
view. The green suites did not cover those cases. E-24 below corrected the
foreign mutation paths; the E-23 snapshot and validation remain historical.

## E-24 owner-qualified file-view mutation (2026-10-14, corrected)

On tested source `59f7923b9e3b508705838b8da8f28a5f7aa3b14e`, every
`VirtualFree`/decommit requires its exact `(ProcessBirthKey, base)` owner.
`UnmapViewOfFile` requires current-owner `FileViewLegacy` metadata before
committing/removing a view. The shared commit path used by Flush/Unmap
rejects a foreign process ID or foreign birth key before guest-byte read or
mapping mutation; legacy nonprivate read/query visibility remains global.
An independent review and the new public regression confirm that child
Free, Flush and Unmap all fail without changing the parent's view, metadata
or bytes, while parent Flush and Unmap succeed. Three focused release
controls passed; see local
`evidence/stage2-e24-owner-fileview-20261013/focused-owner-fileview.md`.

The same source passed the full release suite **1,650/0/5 ignored** and
no-feature suite **1,251/0/2 ignored**. Explicit ignored E-18/E-20 reruns
still failed at their intended stale private byte (`0x11223344` versus
`0x11223345`) and child same-VA `VirtualAlloc` (`Handle(0)`) assertions.
Exact commands and log hashes are in local
`evidence/stage2-e24-owner-fileview-20261013/post-e24-validation.receipt.md`.
A same-PID DIB section view without `VirtualAllocation` metadata can still
Flush through the same-PID commit path but is conservatively refused by
Unmap. Native VA backing remains global, and no fresh-default, memory
improvement, Stage 2 or app/map acceptance is claimed; the E-22 live run
predates this source.

## E-25 internal private-slab fresh-owner diagnostic (2026-10-14, bounded validation)

The source-bound focused receipt for the earlier source is local
[`evidence/e25-private-slab-20261014/README.md`](../../evidence/e25-private-slab-20261014/README.md)
(SHA-256 `978F1BAD8E1D9E8EF14479AE06E767A7BCFFC3B23FD287B2E7F2DF2AB6B58E46`).
It used the D: source, release profile, `unicorn` feature and
`LIBCLANG_PATH=C:\Program Files\doxygen\bin`. Source SHA-256 values at that
receipt were `unicorn.rs` `BFD6097EE12992980ED7CC9E3CC3E0FD68DE7A0C64F8A79D2D019F455B8866DE`,
`memory.rs` `F3F2F226E7523D094EA21B85BEE826F917E5B40849EC78800B05A8FB53D1E077`,
and `native_engine.rs` `346D2E3DAD0141E7A69BF1FEADC7B429861DEAFC318CA00E0097AE492B9AA18F`.
Those hashes and all four raw-log hashes below were independently verified
against the files named in the receipt.

| Focused raw log (same evidence directory) | Observed result | SHA-256 |
| --- | --- | --- |
| `native-private.log` | 3/0: unignored E-18 shape plus unowned-native-RAM and cloned/released-pin mapper rejection | `85B93514D1A65290FFB0E47A1F52B4921BF4F1DEF43EE95F38C57B7A8C89A540` |
| `view-overlap.log` | 1/0: same- and foreign-PID file-view/private-slab conflicts rejected before sync | `4E5CEA3DF68782D4A007916C4206AC59C6E671AF677BEC6752BE694979F0049C` |
| `persistent-native-memory.log` | 4/0 existing native-memory controls | `17435602923C28155ACC8A36B53746AE479F3DD889FC4983B7C5ACCBCF3877EF` |
| `native-mapping-pin.log` | 1/0: pin survives metadata transfer and unchecked owner close | `B1D6F9E7D0AC46F81D174DFC8D9E455648E17547CB5AC9BA8F0FCEDF0A01EA5C` |

The unignored `native_private_virtual_write_survives_closed_owner_next_slice`
fixture uses only the `cfg(test)` private-slab opt-in. Five real MIPS
instructions store `0x11223345`; canonical backing contains that value
before completed native-owner closure. A second real MIPS slice under a fresh
owner loads `0x11223345` at the expected PC with no memory fault. The raw log
prints `first_written=0x11223345 second_read=0x11223345
initial=0x11223344`. The historical E-18 red used the legacy internal
`virtual_alloc` path on an older frozen source; this is not a red-to-green
comparison of an unchanged fixture and source.

The first locked full release run on that focused source exposed a regression:
the synthetic-PID `slice_end_selected_hidden_helper_executes_before_visible_parent_preflight`
fixture failed with `native attempt has no live process birth for pid 67`
(726 passed, 1 failed, 3 ignored; raw `full-release.log` SHA-256
`FA3735B87FE84B56D277FF09F8883445137D4B67FB034FAD8F5F8A15102001CF`).
The corrected source limits that birth precondition to diagnostic slabs;
the separate focused birth-fix controls both passed 1/1, and the rerun of
`cargo test --locked -j1 --release --features unicorn,trace,win32-desktop`
passed **1,664/0/4 ignored** across targets (raw `full-release-final.log`
SHA-256 `3FA9DC90FAADA1F75B01221B75D07A9BDBC729786B97E7141C24283E53573B9F`).
The corrected `unicorn.rs` SHA-256 is `2D698B1DDA6FCC0CD891A71FE04819476F46C0719DFED8905830B0FCFED9C660`;
source/config hashes and both attempts are kept separately in the
[broad-validation receipt](../../evidence/e25-private-slab-20261014/broad-validation.receipt.md)
(SHA-256 `3488E8DCA2FB5040F99B7E143D47DA15452CC4BE6D4E3DC25F4068067DA1F1A5`).
On that corrected source, the locked no-feature suite passed **1,258/0/2
ignored** (raw `no-feature-final.log` SHA-256
`DF1D7DB0C2D1CB9520836BA62BDC230386B70820509D6FD1E577A609E7492194`),
the no-trace Unicorn+desktop release check exited 0 (raw
`alternate-no-trace-check.log` SHA-256
`2283096CBA5A9841002A65ACC5EF893FCE93D8AC15BC7F44A10BB1134E4484F2`),
and the final format check exited 0. These are bounded build/test results,
not a live application run.

Public `VirtualAlloc` remains `LegacySeed`; this
internal diagnostic does not establish public private-VA semantics, E-20
two-process same-VA isolation, shared-view coherence, checked native close,
independent checkpoint clone/relineage, O-P4 fresh-default ownership,
host-memory improvement or bounded iNavi/map readiness. Hook-time map-error
injection, a bounded discarded-owner pin-retention policy, and negative
checkpoint save/restore/direct-Clone controls remain open; a focused pass
cannot close those gates or Stage 2. The reused-owner existing-page fast path
still relies on the ledger/pin-bag invariant without an exact pin lookup;
failure-injection hardening remains open. Legacy outgoing blob sync can
partially update earlier blob snapshots before a later read error: the
diagnostic poisons that attempt, but transactional outgoing legacy VM state is
not proven. The isolated pointer-map diagnostic
`native_mem_map_ptr_boxed_slab_survives_splits_and_fresh_owner` remains
ignored/unrun on this E-25 receipt, not a promoted production control.

## E-26 D-checkout Stage 3 expected-red baseline (2026-10-14, executed)

Under D-007, the thirteen historical E-8/E-11 Task controls were ported from
the read-only ChatGPT project validation fixtures into this sole D: source
checkout. Only headers and expected-red `#[ignore]` annotations changed;
assertions were preserved, and the existing `tests/support/mod.rs` was
reused. The earlier E-8/E-11 receipts remain **private-mirror historical
evidence**; this is a separate current D-source run, with exact fixture and
raw-log hashes in the two local receipts.

The focused no-feature default test-target commands and format check exited
0: **1 active duplicate-close pass, 12 ignored, 0 failed**. Three separate
`--ignored --nocapture` commands then each exited 1 after successful
compilation: seven ownership, two raw-API identity and three execution-pin
controls all failed at their intended assertions (**0 passed, 12 failed**);
no expected-red case unexpectedly passed or failed from setup/compile.
The active positive control was filtered out of those explicit-red runs.
See the [port-and-compile receipt](../../evidence/e26-stage3-red-20261014/port-and-compile.receipt.md)
(SHA-256 `4033964DF4EC994390E430AF7E4587B4DFF8B60E104A9E8580E514E233134A69`)
and [executed-red receipt](../../evidence/e26-stage3-red-20261014/executed-red.receipt.md)
(SHA-256 `CFB251900ED5D0B7F9AA65A8D4B5B0BEB663FA298CBEDC33E9675CED9C2EF75F`).
No production scheduler source changed, no full suite was rerun for this
port, and these red boundaries do not pass S3-A or establish Stage 3
acceptance.

## E-27 second bounded iNavi run on corrected E-25 runtime source (2026-10-14 analysis)

The second user-authorized bounded run used the original read/write mounts
with adjacent pre-live ZIP backups. Its 2026-10-13 UTC harness directory is
local `evidence/e22-live-20261013T182442Z/`; the source-bound
[`analysis.receipt.md`](../../evidence/e22-live-20261013T182442Z/analysis.receipt.md)
has SHA-256 `A74FBE7FE81366DD67831511D13081351A8152B714744B85FAC7B71CC5D96053`.
The harness built a release executable SHA-256
`90A944B949A166171ADFB5EB28414038790144CB51868CC1F97C57160E3A6F4F`
from the D: checkout after E-25 corrected validation, without intervening
runtime-source edits. The three key source hashes checked during analysis
match E-25's corrected receipt. The run manifest did not capture a full
source-tree digest or Git commit, so the executable and checked key files
are the supported source boundary, not full-tree forensic proof.

`result.json` records normal exit 0 after 330.4300087 s at its configured
330 s limit, no RSS monitor trip and empty stderr. The 420 s observer
completed and saved three changed frames. Genuine splash appeared by
input-ready +2.978 s; the last splash JPEG hash began at +28.987 s and
repeated for 151 successful polls through +328.974 s, without an observed
map transition. Sixteen raw-NMEA HTTP posts and one +153 s tap were accepted.
The retained guest `COM7:` trace has three positive `ReadFile` transfers of
174/348/174 B (696 B total) with previews matching injected `$GPRMC` payloads;
the tap down/up reached PID 1/TID 1 window `0x00020008` and has WNDPROC
return records by the +159.00 s capture. This proves input transfer and
message delivery, not parsed GPS, exact touch-handler latency or navigation.

Raw stdout contains 2,518 orphaned return-stub warning occurrences for child
TID 3 and the same call SP. Their archived global `last_return` reports a
`DispatchMessageW`/`WM_ACTIVATE` return, but is not identity-matched to each
encounter; this is not a count of distinct callbacks or proof of the splash
cause. E-25's narrow real-MIPS child park/resume fixture passed separately
but did not reproduce live warning churn. Among 160 host-memory samples,
working set peaked at **1,561,530,368 B (1,489.2 MiB)** and private bytes
at **1,576,525,824 B (1,503.5 MiB)**. Those are still large and their
small difference from the earlier E-22 run is not a measured improvement
from unmatched runs. The mounted inventory stayed at 3,689 entries; four
entries changed timestamps only, with both affected files retaining length
and SHA-256 and no reported additions/removals. E-25's test-only slab did
not become public `VirtualAlloc`, fresh-default ownership, Stage 2 or
splash/map acceptance because this app run survived its limit.

A later E-27 **test-only** real-MIPS discriminator added two controls beside
the unchanged child park/resume fixture; the three focused release tests and a
no-feature compile check passed. Its healthy bounded loop reuses one synthetic
`MSG` for three callback entries and continuations; it does not model three
queue dequeues. After a child park/resume, the test removed exactly one
pending return and observed two fresh orphan-warning events on separate native
attempts, each with PC/RA at the same return stub, lowered call SP, no pending
record and frozen guest dispatch/entry/continuation counts `(1,1,0)`.
This is a controlled stale-stub replay signature, not an identity match to
the 2,518 live warnings, a scheduler root cause or splash progress. No new
app run followed. Commands, source/test-binary/log SHA-256 values and the
interrupted pre-witness compile are in the
[discriminator receipt](../../evidence/e27-orphan-discriminator-20261014/receipt.md)
(SHA-256 `D66F8E33B9FF27B388A287EF5B48E9D83185E95BF436A754E74781F4BB537D12`).

## E-28 Native count-stop witness and typed adapter (2026-10-14, executed)

An additive local Unicorn 2.1.5 C query now exposes whether the current
`uc_emu_start` accepted a count-hook stop request. It is reset per start and
queried on the same lease immediately after the primary native call, with
`Query::TIMEOUT`; it is **not** an exclusive cause or retired-instruction
count. The Rust adapter classifies an uncontested successful request as
`NativeSliceOutcome::CountLimit` and treats query failure as a hard error.
`CountLimit` remains a service boundary under the existing rotation policy;
the one-million-instruction count, CE quantum and task selection did not
change. The [native C seam receipt](../../evidence/e28-count-witness-audit-20261014/native-c-seam.receipt.md)
and [vendor patch inventory](../../vendor/unicorn-engine-sys/PATCHES.md)
record the exact local patch and its upstream status. Focused real-MIPS
count/timeout/private-owner/service-boundary controls and two public typed
one-/two-instruction integration controls passed
([adapter receipt](../../evidence/e28-count-witness-audit-20261014/adapter-slice.receipt.md)).

On that count-adapter source, locked full release features passed **1,671/0/16 ignored**
and no-feature passed **1,259/0/14 ignored**; no-trace release and format
checks exited 0. Those counts predate the focused follow-ups below; complete
logs and selected source SHA-256 values are in the
[broad validation receipt](../../evidence/e28-count-witness-validation-20261014/broad-validation.receipt.md).
An ARM Thumb direct-native control subsequently passed stop reversal inside
an IT block and reset on the next top-level run; all three integration tests
passed. One exploratory same-handle nested-start hook probe exited with
Windows `0xC0000005` before a query assertion and was removed from the
default test. This is not evidence that nested-start ambiguity is handled;
actual nested execution remains unvalidated. A
[read-only risk audit](../../evidence/e28-count-witness-audit-20261014/nested-count-risk.receipt.md)
identifies plausible Rust callback reentrancy and shared C per-start state,
without identifying the crash cause; the
[edge receipt](../../evidence/e28-count-witness-audit-20261014/edge-controls.receipt.md)
preserves the observed result.

The [finish-order plan](../../evidence/e28-run-outcome-20261014/finish-order-test-plan.receipt.md)
then led to a real-MIPS intended red: a parent hook-finish `RESOURCE` failure
was reported after the active PID had rotated to a parked child. The narrow
fix checks `uc.finish()` before handoff. Three focused controls pass: a
one-shot finish failure stays with parent PID 1 and discards its native
owner, outgoing view bytes reconcile, the child remains parked, and the
successful sibling still rotates
([regression receipt](../../evidence/e28-run-outcome-20261014/finish-order-regression.receipt.md)).
That later source passed the locked full release suite **1,674/0/16 ignored**
([release log](../../evidence/e28-run-outcome-20261014/broad-validation/release-unicorn-trace-desktop.raw.log));
locked no-feature passed **1,259/0/14 ignored**
([no-feature log](../../evidence/e28-run-outcome-20261014/broad-validation/no-feature.raw.log)).
The Unicorn+desktop/no-trace release check, format and diff checks also
exited 0; exact source/log hashes are in the
[post-order validation receipt](../../evidence/e28-run-outcome-20261014/broad-validation/validation.receipt.md).
O-P1 still needs one immutable owner/frame-qualified final attempt report,
forced query-error and competing-hook controls, and
preserved native-query provenance in the diagnostic-private error path.
O-P1, fresh-owner Stage 2, E26 S3-A and live-app acceptance remain open.

### E-28 detached diagnostic report transport (2026-10-14, focused)

The next source slice adds `AttemptExecution { report, result }` to the CPU
boundary. Its owned report captures a checked host-process-local diagnostic
ordinal, starting PID/optional birth/TID/PC, raw primary native/count/timeout/
context observations, typed hook request and raw stop PC before deferred child
work, plus an outgoing native owner and final observed outcome. A preflight
rotation or rejected private attempt reports `NoAttempt` with no invented
native evidence. The frame remains `LegacyUnbound`; neither the ordinal nor
the selected CPU's later state authorizes a Task transition.

Real-MIPS parent-to-child, checked-finish, preflight, private native-error and
decoded-exit finish-error controls passed. The decoder now checks outgoing
view sync and hook teardown before terminating; one injected finish failure
left the process alive. See the [report controls](../../evidence/e28-run-outcome-20261014/detached-report-expected-red-plan.receipt.md),
[decoded-exit control](../../evidence/e28-run-outcome-20261014/decoded-exit-ordering.focused.receipt.md)
and [consumer audit](../../evidence/e28-run-outcome-20261014/main-report-consumer-seam.receipt.md).
No expected-red report run was captured; the test assertions and
implementation landed together. The app loop still discards this report via
its compatibility `Result<()>` wrapper, and the scheduler still reads the
mutable selected-CPU outcome. An earlier broad run on this source was
interrupted for error-precedence repair. The subsequent completed
[then-current report-source validation](../../evidence/e28-run-outcome-20261014/detached-report-current-source-validation.receipt.md)
passed **1,682/0/16 ignored** release and **1,262/0/14 ignored** no-feature,
with three check profiles, format and diff checks exiting 0. Three focused
real-MIPS [query-adapter error controls](../../evidence/e28-run-outcome-20261014/query-adapter-error-controls.receipt.md)
passed after the real native query returned; their one-shot Rust substitution
is not a C `uc_query` failure or nested-start proof. Genuine C query failure,
competing hooks, full fallible-tail/partial-effect validation, Task/Frame
authority and Stage 2 remain open. A subsequent bounded app run is E-29;
it is not runtime acceptance.

### E-28 ordinary checked-finish failed retry (2026-10-14, red to green)

An executed [real-MIPS expected-red control](../../evidence/e28-run-outcome-20261014/ordinary-finish-retry-expected-red.receipt.md)
showed an ordinary checked-finish `BackendError` after guest counter `0x3006`
but then a successful second native entry with a new diagnostic attempt ID
and counter `0x3010`. The correction poisons the outgoing CPU after an
entered-attempt error and clears provisional saved context. Its retry now
fails preflight as `NoAttempt`, without a second ID or guest counter change;
the formerly ignored focused release test passed **1/0**. The
[post-fix validation](../../evidence/e28-run-outcome-20261014/ordinary-finish-retry-green.receipt.md)
passed **1,683/0/16 ignored** release and **1,262/0/14 ignored** no-feature,
with feature/release-no-trace checks, format and diff checks exiting 0.
First-attempt guest effects remain, without rollback. This closes only the
ordinary failed-retry control, not all fallible tails, O-P1, Stage 2 or
canonical Task/Frame authority. E-29 used the **pre-fix** executable and
does not validate this correction.

### E-28 typed-stop and run-state wait boundaries (2026-10-14, focused)

A [real-MIPS expected red](../../evidence/e28-run-outcome-20261014/typed-stop-wait-cleanup-expected-red.receipt.md)
registered a same-thread `Sleep` wait, then used a test-only Rust `RESOURCE`
substitution after the actual C `COUNT_STOP=0` query. A host-wall debug marker
was still present on the resulting `NativeError`; the old marker-driven
cleanup wrongly consumed the Kernel wait. Cleanup now requires a finalized
successful `HookStop(_)` or `NativeDeadline` after outgoing view sync and
checked `uc.finish`, with no requested-PC stop. The debug marker is retained
as evidence, not cleanup authority. The host idle source guard also checks
that a saved GetMessage wait ID is still live in Kernel; its binary-target
focused test has not run on this source boundary.

A separate [private error red](../../evidence/e28-run-outcome-20261014/private-wait-error-salvage-expected-red.receipt.md)
then showed Kernel retaining the wait while the CPU lost its transferred
`blocked_wait_threads` record. Private entered-attempt error exits now drain
the transient run-state handles back to the poisoned CPU without publishing
context or rotation. On the [focused green source](../../evidence/e28-run-outcome-20261014/typed-stop-and-private-wait-green.receipt.md),
four real-MIPS controls passed: ordinary/private error paths preserved both
wait records and ordinary/private successful host-budget stops pruned them.
This is **4/0**, not a full-suite result for that later source. The Rust
adapter substitution is not a failing C `uc_query`.

An additional [pre-entry fence expected red](../../evidence/e28-run-outcome-20261014/private-preentry-wait-salvage-expected-red.receipt.md)
showed `NoAttempt` with Kernel wait intact but CPU wait lost before `emu_start`
when a private-page execution fence rejected setup. Its
[two-phase repair design](../evidence/e28-preentry-handle-salvage-design.md)
covers state moved before/after hook construction. The first handles-only
source passed [the non-ready wait control](../../evidence/e28-run-outcome-20261014/private-preentry-nonready-wait-focused-green.receipt.md)
**1/0**, but a [ready-worker expected red](../../evidence/e28-run-outcome-20261014/private-preentry-ready-wait-expected-red.receipt.md)
showed the stronger failure: a selectable worker wait was consumed and the
worker was selected before a later fence error returned `NoAttempt`, leaving
no saved PC. A separate [orphaned-WNDPROC expected red](../../evidence/e28-run-outcome-20261014/private-preentry-orphaned-wndproc-expected-red.receipt.md)
showed that the same rejected setup also lost the separately moved return
queue. The combined source restores that queue on pre-entry errors and moves
ready resume after fallible hook/fence setup. Its
[focused release validation](../../evidence/e28-run-outcome-20261014/private-preentry-ready-and-orphaned-focused-green.receipt.md)
passed the two fence controls **2/0** and the executed-wait filter **5/0**,
with format and diff checks. The later
[ready GetMessage/private-slab integration](../../evidence/e28-run-outcome-20261014/private-ready-get-message-slab-focused-green.receipt.md)
passed **1/0**, with those two filters rerun **2/0** and **5/0** on the same
rebuilt binary. Its guard-active claim rests on checked source order rather
than a runtime guard-state probe. Other ready-resume families, preflight/tail
errors, and state consistency after ready-resume followed by a native-entry
failure remain untested; no rollback semantics are claimed.
These controls do not close
Task/Frame authority, O-P1, Stage 2, S3-A, or live-app map readiness. E-29
used an earlier executable and cannot validate them.

On the final source with the ready GetMessage fixture, the
[locked broad validation](../../evidence/e28-run-outcome-20261014/final-current-source-validation.receipt.md)
passed release **1,691/0/16 ignored** and no-feature **1,263/0/14 ignored**,
plus both Unicorn no-trace checks, format, and scoped diff checks. The
binary-target live-wait guard test passed within the release main suite.
The no-feature/dev check logs note denied incremental-cache finalization,
but both commands exited 0. These tests do not qualify the observational
`LegacyUnbound` report as canonical scheduler authority or update E-29.

## E-29 third bounded iNavi run after detached-report validation (2026-10-14 analysis)

The third user-authorized original-mount read/write run is retained under
`evidence/e22-live-20261013T211230Z/` with a
[binary-identified analysis receipt](../../evidence/e22-live-20261013T211230Z/analysis.receipt.md).
Its pre-retry-fix release executable SHA-256 was `8A8B9C7E07C5DF4B40281A4B882CBCA0FA212DD7A04B9183EE93457AFB3E094A`;
the manifest lacks a complete source-tree digest. It exited 0 at the 330 s
host cap after 330.3949337 s, with no 2,048 MiB RSS-monitor trip or stderr
text. This is bounded host survival, not guest completion.

The observer captured genuine iNavi SE splash by input-ready +2.970 s; its
settled splash image repeated for 151 successful polls from +28.946 through
+329.030 s, without an observed map. Sixteen raw-NMEA posts and a +153 s
tap received HTTP 200. The final retained guest file trace has ten positive
`COM7:` `ReadFile` transfers totaling 2,784 B with injected `$GPRMC`
previews. The tap's down/up messages were dequeued by PID 1/TID 1 with
WNDPROC returns by roughly +160 s. These do not prove parsed position or
touch-driven navigation.

Among 160 host samples, peak RSS/private were 1,601,482,752 /
1,615,622,144 B (1,527.3/1,540.8 MiB); last sampled CPU was 255.3125 s.
The nominally similar E-27 330 s run had 1,489.2/1,503.5 MiB peaks and
223.375 s sampled CPU. Different executable hashes, uncontrolled host load,
and lack of an established exact historical NMEA match preclude attribution
of those differences. E-29 logged 655 orphaned return-stub warnings ending
near +94 s, versus E-27's 2,518 through its cap; warning count is not a
callback count or root-cause proof. E-29's service counters plateaued by
about +120 s while host CPU rose, but E-27 also plateaued by about +100 s.
The visible full-screen splash overlay and a blocked TID 10 wait with one
signaled component warrant an identity-bound trace; the wait-any/all mode is
not saved. The mounted inventory remained at 3,689 entries, with only four
timestamp changes and unchanged hashes/lengths for both touched files. The
[pre-run current-state ZIP manifest](../../evidence/pre-live-mount-backup-20261013T210234Z/manifest.json)
records three adjacent verified archives. O-P1, Stage 2, S3-A and map
acceptance remain open.

## E-30 Host report consumer and E-31 O-P1 red baseline (2026-10-14, executed)

E-30 connected both live and monitor host loops to the owned outgoing
`AttemptExecution` before ordinary error handling, with optional observers
(`None` in production). Two no-feature tests first failed at zero observed
reports, then passed; the final E-30 source passed release 1,691/0/16 ignored
and no-feature 1,265/0/14 ignored. This is not the bounded E-29 trace or
canonical Task/Frame authority. Exact commands, hashes and logs are in the
[E-30 receipt](../../evidence/e30-native-attempt-observer-20261014/validation.receipt.md).

E-31 extracted the first native-deadline JR repair predicate without changing
its production behavior. A typed visible-peer-yield plus deadline pure
control failed at the intended marker-based assertion and is now ignored
pending a real-MIPS mixed-stop/raw-and-saved-PC fixture; the second post-run
JR repair is not covered by that predicate test. Final-source release passed
1,691/0/17 ignored and no-feature 1,265/0/14 ignored; both no-trace checks,
format and diff checks passed. The [red](../../evidence/e31-native-deadline-jr-repair-20261014/expected-red.receipt.md)
and [validation](../../evidence/e31-native-deadline-jr-repair-20261014/validation.receipt.md)
retain source, executable and log hashes. O-P1 remains open; no app run or
mounted-file change occurred.

## E-32 E-31-source bounded iNavi diagnostic and TCG crash (2026-10-14, executed)

The one newly approved writable-mount run rebuilt E-31 source into executable
`D3F07DEF…BE0B89`, with `RUST_LOG=warn,ce.slice_profile=debug`, a 330-s app
cap and 2,048-MiB RSS guard. Three newly adjacent ZIP backups passed
source-entry SHA checks before launch. The host exited early with
`0xC0000005` after 135.282 s without watchdog action; the tap was not sent.
Splash persisted through the final successful +130.994-s frame poll, while
121 completed native slices from outgoing PIDs 1/67/68 were logged after
+120 s. Sixteen NMEA ACKs and 11 retained positive COM7 reads/2,784 B do
not prove parsing. Sampled RSS peaked at 1,655,271,424 B. Mounted contents
were unchanged by hash, with four timestamp/metadata changes. Matching
EXE/PDB/dump show a host write fault in Unicorn TCG code emission at module
RVA `0x89618c`; the missing arena bounds prevent assigning the known Windows
VEH commit-tail defect as the cause. Neither O-P1 nor Stage 2 nor map
acceptance is closed. [Full E-32 receipt](../../evidence/e22-live-20261014T002535Z/analysis.receipt.md).

## E-33 Windows TCG commit-tail repair (2026-10-14, executed)

The pristine vendored Windows handler's final-commit branch left a full
4-MiB request even when less reservation remained. The new one-MiB
pre-init real-MIPS test exited `0xC0000005` against that handler, then passed
after a local `min(4 MiB, right - base)` repair. An isolated 2-MiB test and
the combined 1/2/8-MiB/default four-case test passed with the patch. Full
release passed **1,695/0/17 ignored** and no-feature **1,265/0/14 ignored**;
alternate checks, release build, format and diff checks passed. Default
arena size was unchanged; the new test does not fault at the final tail of a
larger arena. Neither the missing E-32 arena bounds nor E-2's app fault is
resolved by this isolated differential. No app rerun occurred, and the E-32
one-run RW approval is exhausted. The
[E-33 repair receipt](../../evidence/e33-windows-tcg-commit-tail-20261014/validation.receipt.md)
retains source/test/build hashes and the exact limitations.

## E-34 O-P1 mixed-stop JR classification (2026-10-14, executed)

The new real-MIPS fixture first failed with genuine simultaneous native
`TIMEOUT=true` and typed `VisibleParkedPeerYield`: raw PC was the checked JR
`0x00401010`, but post-run repairs rewrote saved/native/snapshot PC to the
import trap and added a false host-wall debug marker. The first JR repair now
uses the typed reason rather than marker presence, and the second import-page
repair refuses typed stops; marker reads only maintain debug history. The
active E-31 pure test and E-34 fixture now pass: both stop facts remain, all
four PCs stay at the JR, no marker is added, and resume dispatches the real
import once. Final-source release passed **1,697/0/16 ignored**, no-feature
**1,265/0/14 ignored**; both no-trace checks, normal release build, format and
diff checks passed. The [E-34 receipt](../../evidence/e34-op1-mixed-jr-20261014/validation.receipt.md)
records exact source/log/artifact hashes and raw logs. **O-P1 stop
classification is closed**, excluding report-tail sealing/effect salvage,
genuine C query-error and competing-hook controls, nested-start ambiguity,
and other ready-resume failure consistency. Stage 2/fresh-default, broader
scheduler and iNavi map acceptance remain open; no app run or mount write
occurred.

## E-35 Public private-VM slab family (2026-10-14, executed)

The normal, non-ignored public COREDLL `VirtualAlloc` ordinal first failed at
its canonical-backing assertion before the switch (expected red, 0/1). The
current private family now prepares owner/birth- and incarnation-qualified
stable 4-KiB pages; checked native map/protect/unmap applies before public
allocation or change success is published. Reserve-only, commit, decommit,
recommit, release, protect, Query, pointer probes, current-target Ex and
two-process same-VA metadata controls are non-ignored. The old `cfg(test)`
slab selector is gone. Shared views, image and heap are not silently migrated.

The frozen-source public fresh-native-owner ordinal and raw integration
controls each passed **1/0/0**. Full release passed **1,727/0/15 ignored**
(`full-release-final.raw.log`, `C753DB02…5E5304`); no-feature passed
**1,292/0/13 ignored** (`no-feature-final.raw.log`, `9C6A0B88…4CC8F9`).
Both release no-trace checks and the release build exited 0. Nine exact source
file hashes were rechecked unchanged afterward; the release EXE hash begins
`5DF5B748`. Touched-file format and scoped diff checks passed. The
[E-35 receipt](../../evidence/e35-public-virtualalloc-validation-20261014/validation.receipt.md)
has commands, full hashes, all raw logs and the earlier evolving-source
diagnostic; the [gate note](public-virtualalloc-slab-migration-gate-20261014.md)
defines this bounded scope.

Negative monitor checkpoint save/restore guards and independent
`MemorySystem::clone` page bytes passed, but full machine checkpoint
relineage remains Stage 3. Persistent native pins are keyed by private page
and released only after checked `mem_unmap`; a 64-cycle churn control and a
forced pre-FFI unmap-failure control passed with budget retention. That seam
is not a real Unicorn-returned unmap error. E-35's public private-family
gate is closed; O-P4 checked `uc_close`/hook teardown and fresh-default,
real native two-process A→B→A same-VA byte isolation, shared-view slab
coherence and Stage 2 acceptance remain open. No iNavi run or writable-mount
operation occurred.

## E-36 O-P4 source-bound fresh-default native owner (2026-10-14, executed)

`ProcessNativeEngine::new` now chooses one fresh owner per native slice;
`WINCE_EMU_PERSISTENT_NATIVE_ENGINE=1` explicitly opts into the separate
Stage 8 experiment. A fresh finish, abort or unfinished-lease drop closes the
unique owning Unicorn wrapper while the nonowning run wrapper, callback boxes
and pointer-backed pins remain alive, then releases them. It does not call
individual `remove_hook` first. The pinned vendored Unicorn 2.1.5 C
`uc_close` has only successful exits, and the patched sys crate statically
builds that in-repo C. This is a source-specific closure proof, not a general
checked-close API; dependency, linkage or vendored-source changes require
reaudit. No closed owner's blank Context, map ledger or pins enter a new lease;
only exact-key CPU state can transfer.

The formerly ignored E-7 closed-owner Context/MIPS continuation control is
normal and green; the 200-slice pointer-backed guest-store test alternated
checked unmap and close-while-mapped without retained pins or inherited maps.
Focused finish-failure, public private-byte, count-stop, typed-stop and mixed
JR controls passed. Frozen-source full release passed **1,730/0/14 ignored**
(`full-release-final.raw.log`, `28F25760…283AE9`), no-feature passed
**1,292/0/13 ignored** (`full-no-feature.raw.log`, `399499AA…A81F06`);
both no-trace checks and release build exited 0. The preserved EXE/PDB and
all exact source/log hashes are in the
[O-P4 receipt](../../evidence/op4-fresh-default-20261014/validation.receipt.md).
An earlier diagnostic full run had nine distinct library failures from tests
that inspected an owner after normal fresh closure; the final tests read the
captured per-attempt report and retained their PC/TIMEOUT/outcome assertions.

The frozen-source O-P4 gate is closed, **not** full Stage 2 or live
acceptance. No post-switch iNavi run, memory peak, busy-slice MIPS, fixed
per-slice setup-cost comparison or map-readiness measurement was made. A run
on the original writable mounts requires fresh human approval and fresh
adjacent ZIP backups; previous single-run approvals are spent. Real native
two-process A→B→A same-VA bytes, shared-view slabs and Stage 3 checkpoint
relineage remain open.

## E-37 Fresh-default view and legacy-page sync (2026-10-14, executed)

The one approved O-P4 iNavi launch used a preserved fresh-default EXE and fresh
adjacent, SHA-verified ZIP backups of the three original writable mounts. It
exited 1 after **2.9344481 s**, before NMEA or tap delivery, with
`Backend("read process 67 mapped view 0x50000000: READ_UNMAPPED")`. Its two
memory samples and 37 busy slices are startup diagnostics, not a 330-second
memory, throughput, splash/map or Stage 2 result. The single-run approval is
spent; the [live receipt](../../evidence/e22-live-20261014T041105Z/analysis.receipt.md)
retains the exact backup, binary, input and output evidence.

The fresh-owner outgoing view sync now reads only native mapped intersections,
leaving untouched canonical mapping bytes intact. The incoming sync writes
only mapped spans and fails when an unmapped gap lacks matching canonical lazy
seed bytes. A separate first-import generation sweep now seeds only newly
mapped virtual pages; it cannot replay stale `initial_bytes` over a guest
store to a page fault-mapped earlier in that slice. Direct native tests cover
unmapped/partial readback, partial inbound positive and stale-seed negative,
and a real MIPS guest store before the generation sweep. Public controls cover
`MapViewOfFile` across fresh slices, a guest store around its first subsequent
import, and a successful `CreateProcessW` with `live_pump: true` through the
deferred launch/re-seed path. The public CreateProcess control does not by
itself prove partial inbound mapping: the generic mapper fills remaining pages
before re-seed; the direct positive/negative tests discriminate that case.

Frozen-source full release passed **1,739/0/14 ignored**, no-feature
**1,294/0/13 ignored**; both no-trace checks and release build exited 0.
The [E-37 receipt](../../evidence/op4-fresh-view-readback-20261014/validation.receipt.md)
has source, raw-log, preserved EXE/PDB hashes and test-scope limits. This is a
source-bound sync gate only; no iNavi re-run occurred after the fixes. Another
original-mount launch requires fresh human approval and fresh adjacent verified
ZIP backups. GMS, real native E-20 A→B→A bytes, shared-view slab coherence and
full Stage 2/live acceptance remain separate.

## E-38 Preserved E-37 iNavi live comparison (2026-10-14, executed)

A separate human approval (`call_93cb71e3f4dc432c84262f0a340f3091`) covered
**one** run of the frozen E-37 release EXE on the original writable mounts,
after fresh adjacent SHA-verified ZIP backups. That launch exited 0 at the
330-second host cap (330.3920454 s), with no watchdog trip or guest CPU error;
the observer completed. The E-37 source-bound view fix therefore cleared the
earlier O-P4 fatal mapped-view readback in this one live attempt. The settled
iNavi SE splash persisted through the end; **no map** was observed.

The 160 host samples peaked at **430.676 MiB RSS** (+299.737 s) and
**444.953 MiB private** (+326.484 s). The RSS peak is 22.5% below BED2 E-1
and about 71–72% below the persistent E-22/E-27/E-29 peaks over similarly
bounded runs; this is descriptive, not a causal or input-matched result.
Sixteen raw-NMEA POSTs and one tap received ACKs; retained guest traces
additionally show nine positive COM7 reads (2,262 B) and tap down/up WNDPROC
returns. They do not prove GPS parsing or navigation. The 1,714 busy native
slice profiles give **17.120 weighted MIPS** overall and **7.278 ms
setup+hooks per busy slice**, but the persistent runs lack matching profiles,
so a throughput or fixed-cost gain cannot be asserted. This run-only MIPS
measure excludes setup/hooks; busy slices averaged **58.398 ms** in `run_ms`
and **68.159 ms** across the five tracked fields, with setup+hooks at
**10.68%** of tracked time. E-9's same-field hookless range was **22–34
MIPS for busy iNavi slices** on two-minute runs. E-38's corresponding PID 1
figure is **26.359 MIPS, within that range**; the 17.120 figure includes
helper processes, chiefly PID 67 at 15.060 MIPS, and cannot be compared to
E-9's iNavi-only range. Neither phase/input parity nor a cold-TCG cost is
established; that remains a paused Stage 8 question. Four mount-inventory
entries changed timestamps only, with file hashes unchanged. All backups and
the full [E-38 receipt](../../evidence/e37-live-20261014T051228Z/analysis.receipt.md)
remain on D:. This single-run approval is spent; map-readiness and full Stage
2 acceptance remain open.

## E-39 E-20 native two-process same-VA A→B→A bytes (2026-10-14, executed)

Two real MIPS/Unicorn regressions passed with fresh native owners across
A→B→A. The public private-VM control calls COREDLL `VirtualAlloc` for distinct
process births at `0x52000000`: A writes `0x11223344`, B writes `0x55667788`,
then A reloads, increments and stores only its own canonical slab bytes. The
file-view control likewise proves owner-qualified native seeding/readback at
one guest VA for separate file mappings, with exact birth reports and closed
fresh owners, but injects the two views because the legacy public view
allocator still has a global cursor. Both controls manually select the queued
child and restore the parent; they do not prove guest `CreateProcessW`/scheduler
handoff, public same-VA `MapViewOfFile`, named reopen, same-mapping aliases or
shared-view slab coherence. No production fix, iNavi launch or mount write was
part of this gate.

On `unicorn.rs` SHA-256 `C36AEEB5300230804E8A170E34523E77DCF036239B19A1493D35B4163142B04F`,
both focused controls passed **1/0**. The frozen-source full release suite
passed **1,742/0/14 ignored** (raw log SHA-256
`624AEB38A9A693FF019E1605E08B1FBDE8140A21710A05F817688551CDA1A288`),
and no-feature passed **1,296/0/13 ignored** (raw log SHA-256
`C0412636E84AEC4B17924C79C08FD7D708FF84E46356747564DBC7F58B246FAE`).
The same broad source includes four passing public `GlobalMemoryStatus` tests,
but that change has a separate gate. The [E-20 native receipt](../../evidence/e20-native-same-va-20261014/validation.receipt.md)
binds the exact commands, source/test-executable/log hashes and limits.

## E-40 GlobalMemoryStatus private-page budget (2026-10-14, executed)

The device-dump defaults now drive `GlobalMemoryStatus`, while one
boot-reserved private-page budget governs both reported available physical
bytes and public `VirtualAlloc` commit admission. Four focused public API
controls passed **4/0/0**: exact defaults, guest-RAM overrides, private-page
delta/release, and a small-budget exhaustion/recovery path. After the
test-only setter gate, all four also passed in the combined-source full
release (**1,742/0/14 ignored**) and no-feature (**1,296/0/13 ignored**)
suites recorded at E-39. This is a source gate, not a new iNavi launch.

Only stable private pages are charged today; heap, image and legacy file-view
backing do not yet reduce reported available physical memory. The
[GMS receipt](../../evidence/gms-device-report-20261014/validation.receipt.md)
records commands, hashes, the separate E-20 source present at broad gate,
and this accounting limit. Shared-view and image/heap migration must either
charge the same budget or explicitly justify a different accounting rule.

## E-41 D-008 public shared-view slab source gate (2026-10-14, executed)

Public named `CreateFileMappingW`/`MapViewOfFile` now publishes one canonical,
address-stable, 4-KiB shared-page slab rather than retaining a second
writable Legacy byte copy. A real-MIPS A→B→A fixture reopens the named object,
manually selects two process births, and confirms alias byte coherence across
fresh native owners, including an unflushed in-slice alias read. The equal
guest VAs demonstrate isolation **at a shared VA** produced by the emulator's
per-owner cursor; they do not assert a CE same-VA guarantee or prove guest
`CreateProcessW` handoff. DIB migration, file-backed underlay, budget-OOM
rollback, same-birth raw Flush propagation, and explicit refusal of a new DIB
view after slab transition have focused controls. Shared pages now charge the
existing private-page budget, but that still is not a full memory census.

On the same independently rechecked nine-file source/test hash set, including
`memory.rs` SHA-256 `CF7CF08F6D80D9D4183ADE958A4BD92B5CCBB298660D724693B3AA774D3884B6`,
serial full release passed **1,744/0/14 ignored** across 17 blocks (raw log
SHA-256 `E373A3712D5BB21C89DDD4BF05245A09346506CC2FB004B0BCE1422E87E02F6C`),
and no-feature passed **1,297/0/13 ignored** across 17 blocks (raw log SHA-256
`DA43812484166EED583C8F2C894B130DD3D1CFE905F7BC0AD90168E64939B69B`).
The [D-008 receipt](../../evidence/d008-shared-view-20261014/validation.receipt.md)
records the exact commands, focused controls, source and executable hashes,
and earlier failed/partial logs. A pre-fix debug run caught an offset
underflow hidden by release; only the final same-source pair is signoff.

This closes a **source-bound D-008 shared-view gate**, not live iNavi, map
readiness, full Stage 2, image/heap accounting, Task authority or handle
lifetime. Named reopen still uses global handle numbers, not per-process
handle transfer. File-backed view coherence with `ReadFile`/`WriteFile` and
external edits remains excluded; raw adapter alias writes are not
rollback-atomic after an error. First public mapping migrates and charges the
whole existing anonymous Legacy backing at `MapViewOfFile`, while [Microsoft's CE mapping reference](https://learn.microsoft.com/en-us/previous-versions/windows/embedded/ms923951%28v%3Dmsdn.10%29)
documents default whole-section `SEC_COMMIT` at creation for CE 4.2/5;
CE6 device-specific accounting is unmeasured. No iNavi launch or writable
mount operation occurred in E-41.

## E-42 Immutable PE image-page eligibility source gate (2026-10-14, executed)

A pure loader helper selects complete 4-KiB image pages from PE section
characteristics, returning R or RX permissions and borrowed slices of supplied
image bytes. It excludes headers, writable/execute-only sections, partial or
mixed pages, patch overlaps and the trampoline tail beyond `SizeOfImage`;
invalid extents fail closed. One synthetic test manually patched a supplied
`Vec` before selection. This proves byte borrowing from that input, **not**
that actual relocation, import and trampoline patches ran before publication.

The expected-red control failed to compile at the missing helper. On
`src/emulator/pe_loader.rs` SHA-256
`30791E94E6F47DD0A8B4B2DA0F1CA42253F4B07519B48CC12C2A083C0F68016A`,
final no-feature focused and debug Unicorn-feature focused runs each passed
**5/0**; the full no-feature suite passed **1,302/0/13 ignored** across 17
blocks. The [E-42 receipt](../../evidence/immutable-image-pages-20261014/validation.receipt.md)
holds the four retained passing-log hashes and commands. A later test
overwrote the expected-red raw log; its original bytes are gone, so that red
attempt is historical observation rather than retained log evidence. No full
release-feature suite is claimed for E-42.

At E-42, production image mapping still used RWX with the old copy path.
Sealed segmented storage, clone isolation, typed native pins, host-write and
readback guards, native write protection, 200-slice churn, memory-budget
treatment and a synthetic 56-MiB setup benchmark remained separate gates.
Later E-43 source work changed that mapping path; E-42 itself made no iNavi
run, writable-mount operation, timing, map-readiness or Stage 2 claim.

## E-43 Sealed main-EXE final-byte source gate (2026-10-14, executed)

The production loader now seals final import- and trampoline-patched main-EXE
bytes in aligned R/RX runs. One-time rewritten branch pages are eligible;
IAT thunk pages stay copy-backed, and the stub tail beyond `SizeOfImage` is
excluded. Generation-qualified native pins and fresh/reused-owner ledger
checks retain backing identity; focused native-write overlap, guest write-
protection, patched-branch execution, mutable IAT, file-view and malformed-
blob controls passed. This is a **source-bound** gate, not a general proof that
every host write path or memory-mode checkpoint is safe.

An opt-in, read-only production-loader pass over the E-38-manifest-matching
iNavi EXE plus seven hashed DLLs sealed **1,379 of 2,168** total image pages
and **1,379 of 1,379** non-writable-section candidates: **5,648,384 B
(5.386719 MiB)**. Of those, **1,224** contain bytes changed by final loader
patching. The earlier 155-page result belonged to the superseded byte-
equality filter. The load-only initial-owner census was **26 mapped blobs,
36,679,680 B (34.980469 MiB)** including 16 MiB heap-arena and 4 MiB stack
backing; runtime heap spillover and later DLLs were not observed. No native
guest execution or writable-mount write occurred in the probe.

On the rechecked source snapshot (`pe_loader.rs` SHA-256
`ED5D78E2…0F6284`, `unicorn.rs` `FEF83F33…BA4083`), the serial release
`unicorn,trace,win32-desktop` suite passed **1,761/0/17 ignored** across 17
result blocks (raw log `A33BE1E6…840A9A`), paired debug no-feature passed
**1,303/0/13 ignored** across 17 blocks (raw log `6F4AFDE3…FFEF27`), and the
debug `unicorn,win32-desktop` no-trace check finished. Earlier compile/test
reds and a libclang-unavailable coverage attempt are diagnostics, not signoff.
The [E-43 receipt](../../evidence/sealed-main-image-20261014/validation.receipt.md)
records exact source/input/log hashes, per-blob-name bytes, focused controls,
and limits. No new iNavi launch, app speed/footprint result, map readiness,
`PrivatePageBudget` decision, or full Stage 2 acceptance is claimed; the E-38
single-run approval is spent.

## E-44 Fixed initial heap/stack slab source gate (2026-10-14, executed)

The initial 16-MiB heap arena and 4-MiB guest stack now reserve owner-qualified
fixed ranges in `virtual_allocations`, with heap commit at allocation and
stack commit on native fault rather than 20 MiB of initial `ram:` owner blobs.
Native pins retain committed pages through checked owner/unmap closure;
queued payloads use checked heap delivery and reject unrelated fixed-slab
overlap. Audited registry, time, geometry, find-data and module/path startup
outputs have guarded whole-span preflight. Unconverted direct-import outputs
can still fail closed on an untouched stack page.

On the formatted six-file source snapshot (`memory.rs` SHA-256
`9907CE18…CEF2A7`, `kernel.rs` `B48C703A…948A09`, `coredll.rs`
`72870167…49A2E4`, `crt.rs` `E625FDAF…A20073`, `pe_loader.rs`
`ED5D78E2…0F6284`, `unicorn.rs` `D883EEB2…20B084`), serial release
`unicorn,trace,win32-desktop` all targets passed **1,785/0/17 ignored** across
17 blocks (16 nonempty); paired debug no-feature passed **1,321/0/13 ignored**
across 17 (14 nonempty). The debug Unicorn/desktop no-trace check passed.
The first release attempt stopped at **803/28/4**: 24 fixture assumptions
(21 old heap-base scratch/view, two stack blob seeds, one budget) and four
wait fixtures missing birth context. Focused 24/0 and 4/0 exact reruns passed;
production guards were not weakened. A later COREDLL test-only formatting
change required the final paired broad reruns. The [E-44 receipt](../../evidence/heap-stack-20261014/validation.receipt.md)
pins full source/log hashes and diagnostic lineage.

An exact blocked serial/NMEA path test passed 1/0 across two untouched stack
pages and a fresh owner, but did not independently test stale different-birth
rejection. Synthetic 56-MiB sealed-RX post-slab release and debug controls
each passed 1/0; owned blob capacity, fresh writes and fresh/reused readback
were **77,824 B** versus the E-43 pre-slab debug **21,049,344 B**, an exact
**20,971,520-B (20-MiB)** logical reduction. Same-mode debug fresh/reused
setup samples were 16.740/25.249 ms versus prior 33.246/37.971 ms, but one
sample each and slab reservation outside the timer preclude a speed claim.
No E-44 live iNavi run, writable-mount write, host RSS, map readiness or
full Stage 2 acceptance follows. O-M2 global heap metadata, spillover,
writable image/DLL backing and pin-budget lifetime remain separate work.

## O-M2 Per-birth heap source gate (2026-10-14, executed)

O-M2 replaced the global heap cursor and ownerless allocation metadata with
`ProcessBirthKey`-qualified cursor, handles, `(birth, VA)` blocks and prepared
allocations. Typed queued payloads capture the recipient window's birth. The
fixed process-heap reservation is now `[0x30000000, 0x40000000)` (256 MiB),
with 4-KiB pages committed only as touched; the copied-spillover path is
removed. Birth-local monotonic holes remain after failed prepared allocations.

On the exact 12-file source/test hashes in the [O-M2 receipt](../../evidence/om2-heap-20261014/validation.receipt.md), the serial release
`unicorn,trace,win32-desktop` all-target gate passed **1,803/0/17 ignored**
across 16 result blocks ([raw log](../../evidence/om2-heap-20261014/release-unicorn-trace-desktop-attempt-07.raw.log),
SHA-256 `1B54366E4E894A448B71D86FE98BA3AAE44CD8F99BAF8550FB9ACAB75356B5D2`).
The paired debug no-feature all-target gate passed **1,341/0/13 ignored**
across 16 blocks ([raw log](../../evidence/om2-heap-20261014/debug-no-default-attempt-02.raw.log),
SHA-256 `DF1317C9D1AC85CBC9AA7E37D0F449BE4217C2159F276D5CC631F07DB87A2E1B`).
The Unicorn/desktop no-trace check passed ([raw log](../../evidence/om2-heap-20261014/check-unicorn-desktop-no-trace-attempt-02.raw.log),
SHA-256 `4780471D9C6B47D295B037CFBE834011706637A9FEAEE81C23B3D0F591F04AB3`);
scoped Rust formatting and `git diff --check` passed. Earlier numbered
compile/fixture failures and the final test-only rerun lineage are preserved
with SHA-256 hashes in the receipt; only attempts 07/02/02 sign off the final
snapshot.

The broad tests cover same-VA A/B ownership, foreign-only heap rejection,
>32-MiB native MIPS growth, top-boundary OOM without charge, queued typed
payload retention/cleanup, and stale A-born serial/NMEA completion under B
without RX or stack mutation. The same-VA raw text-pointer fixture checks
that A's size and bytes persist while B owns its own allocation at that
numeric VA. A valid but impossible in-place realloc reports
`ERROR_NOT_ENOUGH_MEMORY`, matching local CE6 `rheap.cpp:1947-1963`.

This is source-only. Safe heap address reuse needs original footprint and
incarnation checks; `ORD_REMOTE_HEAP_*` target routing and cross-birth
clipboard handle ownership remain open. Writable image/DLL backing, full
clone relineage, direct cold-stack imports, checked pin-budget closure,
Stage 3 and a freshly approved bounded live run remain separate. No O-M2
iNavi launch, host RSS, busy-slice MIPS, map readiness or speed gain is
established.

## W-B1a Window-owner birth source gate (2026-10-15, executed)

Five pre-fix expected-red attempts exposed stale explicit HWND post and native
creation through ordinals 246/688/690; the fifth ran MIPS `TerminateThread`
then `CreateWindowExW` in the same slice. The W-B1a implementation freezes one
Window owner, checks native caller birth before creation effects, and uses the
stored owner for explicit HWND FIFO/wake, typed recipient/payload retirement
and the CreateWindow callback process. The first broad release a15 stopped at
975/2/6 because two stale typed-payload fixtures failed; a third adjacent
fixture was corrected too, without restoring a numeric-PID fallback. Focused
a16–a18 then passed.

The corrected source is base `0e686c9c` plus
`sealed-code-source-a19.patch` (SHA-256
`02429B95862509D26FEBA3C1E62976D164FE9F7EEF6DE975D0FC197646E608DD`);
post-test file hashes matched. Full release a19 passed **1,967/0/9** and
no-feature a20 **1,427/0/3**, each across 18 result blocks; Unicorn-only and
Unicorn/desktop no-trace checks a21–a22 passed. The
[receipt](../../evidence/stage3-window-owner-birth-20261015/validation.receipt.md)
holds all raw logs, exact commands, attempt hashes and limits. Broadcast,
HWND timers, hotkeys, sent/paint/modal paths, remote input, host teardown,
full Stage 3 and live iNavi acceptance remain open.

## W-B1b Remote touch/key window-owner source gate (2026-10-15, executed)

Preserved test-first controls exposed stale explicit HWND touch/key reaching a
reused TID, stale capture/focus and filter/hint routing, and coordinate-only
events being lost instead of hit-testing the live window. Early tests that
treated a guest filter or host hint as an explicit target were corrected and
are not acceptance oracles. The corrected
input-only path checks each selected Window's frozen birth before focus,
input-state, FIFO/status/time/post-trace or wake effects. Explicit stale host
targets consume/drop without replay; coordinate touch resolves live capture
or delivery-time hit-test, blocked-wait/modal HWNDs remain fallback hints, and
keys resolve live focus/active ownership rather than mouse capture. Focused
controls passed **27/27**.

The earlier a14 release pass was diagnostic-only because it still used those
invalid routing oracles. Broad a33 release and diagnostic a34 no-default both exited 101 on the old
`blocked_get_message_wait_wakes_when_remote_input_is_drained` fixture: its
hidden, zero-rect legacy HWND could no longer be a valid input target. The
a35 test-only repair used a visible, enabled Birth-owned window containing
the touch point; named a36/a37 focused checks passed before the fresh a38
source seal. Those failed broad attempts are retained, not counted as passes.

The pre-run `seal-a38-source.md` SHA-256 is
`08080D5382DF17C8825940EF05C294AE13F4C5451652CA49AC691D09477C4995`.
Its base `413947b3` binary patch `seal-a38-source.patch` SHA-256 is
`ED7A3B48DD9272F9E139DB883BCE5093661E443AEE7BE0684D35E576FA962D40`;
the sealed kernel, GWE and integration-test file hashes remained unchanged
through the final matrix despite the later backup HEAD. Release a38 passed
**1,988/0/9** and no-default a39 **1,448/0/3**, each across 18 result
blocks; Unicorn-only a40 and Unicorn/desktop no-trace a41 checks passed.
The [receipt](../../evidence/stage3-window-input-birth-20261015/validation.receipt.md)
holds raw attempts, commands, hashes and limits. Mixed touch/key ordering,
full exit capture retirement, implicit gesture ownership, owner-filtered host
teardown, remaining producers, CE6 implementation parity, full Stage 3 and
live iNavi acceptance remain open.

## Live-run preparation: telemetry, clean build and offline safety (2026-10-15, historical source-only gate)

After the W-B1c source gate, a pre-run Rust
[seal](../../evidence/stage3-live-evidence-20261015/telemetry-broad-a03-pre-run-seal.md)
(SHA-256 `9D28B43737E94ED902009019A4E145DEE38BD35815222F22CA65A06D4DBA0BF2`)
and scoped patch (SHA-256
`042C126BA55B6521E5607D24518FE050831E2D8AC47A2FEC5A35179EEE5934F7`)
bound the debug-summary per-server publication stamp and narrow live guest
stable-page budget field. Full release passed **1,996/0/9** and no-default
**1,456/0/3** across 18 result blocks each; Unicorn-only and desktop-without-
trace checks exited 0. The [validation receipt](../../evidence/stage3-live-evidence-20261015/telemetry-broad-a03-validation-receipt.md)
holds the raw-log hashes and unchanged post-test source hashes. This scalar is
not total guest commit or host RSS; the stamp is not a shared cross-endpoint
snapshot epoch. The user-supplied `D:\GitHub\llvm-proj\build\bin\libclang.dll`
was absent, so the gates used `C:\Program Files\doxygen\bin\libclang.dll`.

A package-scoped `cargo clean --release -p wince_emulation_v3` removed 449
disposable build files (9.6 GiB), then a real clean release build produced a
preserved EXE (SHA-256
`1984310B427BF5E719EB2BDDEB7871134C0EDC923544271B32D962A6FF6AC513`)
and PDB (SHA-256
`B7C0C7C403CB93CAB2AE214D65FA48414B8524E7AC784FFB857A13C92FD28AD8`)
under `evidence/stage3-live-evidence-20261015/`. Their RSDS GUID+age matches:
`1673843D-3539-4BBE-8265-C11B88F6B4C1:1`. The
[clean-build receipt](../../evidence/stage3-live-evidence-20261015/telemetry-broad-a03-clean-build-receipt.md)
distinguishes this build from an earlier no-op artifact reuse.

The final 21-file [source seal](../../evidence/stage3-live-evidence-20261015/final-source-seal.json)
has SHA-256 `9BC784FD86060C429889160702234080682199E0D08C7D4A39FC7CF367845ECB`;
its scoped patch is `D2B7696A4F5E814E2B059BCA4419227138C6EF369FF05E5B0D71EFC3D1CA1C27`.
The [final build receipt](../../evidence/stage3-live-evidence-20261015/final-build-receipt.json)
has SHA-256 `B29AC4E45AB58C986A15D27CBDB4191D533AA4219E9EF86156A6D51634830B28`.
Separate [offline safety controls](../../evidence/stage3-live-evidence-20261015/final-offline-safety-gate-receipt.md)
passed **97/97 preflight, 40/40 runner, 14/14 backup and 9/9 observer** without
running the effectful entry point or touching real mounts. The two historical-
use ledgers were seeded with six deny rows and are byte-identical (SHA-256
`90B83E40BE6550BFEFACF36538BF77DEB8EE41C905E83953C75CCE8F57C333A3`).
The [final seal receipt](../../evidence/stage3-live-evidence-20261015/final-source-build-seal-receipt.md)
records the source-to-binary checks and independent review. Its build-receipt
test flags are cross-checked attestations, not cryptographic execution proof.

No iNavi guest, real writable-mount backup, mount write or live observer ran
for this historical gate. At that time, a fresh, distinct one-run chat approval
was pending and would require human authenticity review. A future approved run
must make fresh adjacent ZIPs after approval, recheck mount content and
preserved artifacts, keep the
mounts quiescent, and disclose LAN exposure at the mandated
`192.168.0.39:8765` endpoint. Offline controls do not eliminate a simultaneous
driver/monitor death immediately after spawn or external writes after the
last mount scan. Splash/map readiness, boot time, guest response lag and the
current build's live memory footprint are still unmeasured.

## Aborted prelaunch backup, withdrawn approval and watchdog source candidate (2026-10-15, no app run)

Approval `call_9b35e9d31fdf402ea082a3eb6a40ae16` was verified at
02:50:21Z for one backup/run workflow. The adjacent backup began at 02:55:00Z
and stopped at 03:00:01Z when the old `File.Replace` helper passed an empty
backup-file path while updating the existing manifest; its recovery update
failed the same way. The [failure record](../../evidence/pre-live-mount-backup-20261015T025500Z/failure.json)
(SHA-256 `8D2B9F4291F55F3396F2FE1C4E17D6CBD3D029C0DFA69B55AED4C30E697EDF29`)
and [disposition](../../evidence/pre-live-mount-backup-20261015T025500Z/aborted-approval-disposition.md)
retain the scope. `manifest.json` remained `complete=false`; only the SD ZIP
was verified and published adjacent to its original folder:
`D:\INAVI_Emulator\INAVI-pre-live-20261015T025500Z.zip`, 5,973,898,396 bytes,
SHA-256 `6032F822551CFB5F28ED079CB5B41CB02B1585F8AFF16EA303B54A74B0C2C949`.
No ResidentFlash or EmulatorRoot ZIP, complete triplet, runner reservation or
guest launch followed. This workflow made no guest write to the original
mounts; independent writers were not audited. The SD ZIP is retained pending
the user's disposal choice and cannot be reused for a future run.

The approval was withdrawn/void. A deny-only `withdrawn` row records its
approval reference, attempt ID, UTC, reason, the one actual ZIP hash and a
hash-preserved failure-evidence path—no invented run manifest or process.
Both append-only [local](../../evidence/live-run-use.jsonl) and
[tracked](live-run-use.jsonl) ledgers now have seven byte-identical rows,
SHA-256 `53BAD1ABCAEA7F723737014E182228922122951F1CFB0574773E49EE3DFD1A08`.
The repaired runner independently rehashes withdrawal evidence when validating
the ledger. Reusing that approval or ZIP remains denied; a fresh approval is
required before **any** further backup or launch.

The backup and runner JSON overwrite defects were captured by
[expected-red controls](../../evidence/stage3-live-evidence-20261015/atomic-json-overwrite-expected-red-receipt.md)
and then closed by the [green offline control](../../evidence/stage3-live-evidence-20261015/atomic-json-overwrite-green-receipt.md).
The now-historical [22-file a02 source seal](../../evidence/stage3-live-evidence-20261015/prelaunch-repair-a02-source-seal.json)
has SHA-256 `1DC1537217C621EBD1B8C2D5E682C50B9CFA6FA131D22CC3202F54C7C5651F97`;
its full W-B1c-scoped [patch](../../evidence/stage3-live-evidence-20261015/prelaunch-repair-a02-full-scoped.patch)
has SHA-256 `8CF8CA60072D4F0ECDA8B21D5400A6B00F25344671D8082E374145F19CDAD18C`.
The [script-only build bridge](../../evidence/stage3-live-evidence-20261015/prelaunch-repair-a02-build-bridge.json)
has SHA-256 `A3746E1729186997CC684030E5E5BA772A1A3038132F49E5BAE9A04C6A21D580`.
Its [raw offline gate](../../evidence/stage3-live-evidence-20261015/prelaunch-repair-a02-offline-gate.raw.txt)
(SHA-256 `3843D86AC1495D9A29D64CA284ECA51A63A3B05FD4700E6D7E963F1CD761D268`)
records **123/123 preflight, 40/40 runner, 14/14 backup, 9/9 observer** and
green atomic overwrite controls. The previously sealed 11 Rust/Cargo files
and `.gitattributes` match the clean-build source; the same preserved EXE/PDB
and RSDS are carried forward, **without a Rust rebuild**. The earlier 21-file
seal and 97/40/14/9 controls above remain historical; both a01 and a02 are
superseded and **not live-usable** after the independent watchdog source
change. The
[withdrawn-ZIP cross-row receipt](../../evidence/stage3-live-evidence-20261015/withdrawn-zip-conflict-red-green-receipt.md)
preserves an expected-red log (SHA-256
`DB936FA73ABD15C757259A59A6B683FE29105D34B36A62838A5455CAE5DE2505`):
three false accepts shared a withdrawn ZIP with earlier/later historical rows
or another withdrawal. A02 rejects those conflicts with `LEDGER_INVALID` and
requires withdrawal evidence to contain exactly path and hash. Build-bridge
pass flags are attestations linked to preserved Rust and script receipts, not
a new binary test execution or live-app acceptance. Claude's a02 review was
unavailable at that session limit; this is not a waived review or new run
authorization.
The [a02 source/build receipt](../../evidence/stage3-live-evidence-20261015/prelaunch-repair-a02-source-build-receipt.md)
records the per-file/binary hashes, actual-ledger and synthetic-bridge checks,
and the remaining prelaunch conditions together.

The later a04 runner candidate closes the observed post-spawn/pre-descriptor
containment gap with an independent `WatchdogWorker`. The driver starts and
identity/heartbeat-checks that worker before publishing `launch-request.json`.
After the monitor records the post-mount-scan marker, the watchdog has a
355-second monotonic deadline even if a subsequent ledger or descriptor write
stalls. It probes exact-app RSS and combined raw logs at two-second intervals,
attempts exact monitor/app cleanup on deadline or worker error, and leaves the
existing detached monitor's 390-second guard intact. The driver's missing-
descriptor finalizer also performs exact cleanup while the monitor is alive;
an unconfirmed cleanup leaves the independent watchdog active. An atomic
`launch-began.json` marker is flushed after reservation and before
`Start-Process`. If it exists, an empty WMI search is not proof that no app
spawned: the watchdog retains the unresolved attempt and repeats exact cleanup
sweeps. A confirmed no-spawn requires the exact monitor stopped, marker and
process descriptors absent, and no spent ledger row. No broad process-name
kill is used. Independent containment assumes the D: evidence/mount volume
makes I/O progress; while the whole volume is stalled, guest writes to these
original mounts stall too.

The current [a04 source seal](../../evidence/stage3-live-evidence-20261015/prelaunch-repair-a04-source-seal.json)
is SHA-256 `C495AB5324BABB1DCF252B8DCCB6A0B5EC4A58C3D04632E3728D39C6AFFA376C`,
the [full W-B1c-scoped patch](../../evidence/stage3-live-evidence-20261015/prelaunch-repair-a04-full-scoped.patch)
is SHA-256 `B9FBE5F1DA42453BE8213D2FD45B4553126B62F9641A2003FEB4D2FC26C35CCB`,
and the [script-only build bridge](../../evidence/stage3-live-evidence-20261015/prelaunch-repair-a04-build-bridge.json)
is SHA-256 `3C84522E0D5DF2B6B0BA7BFD8DA8BB14747E4270BDB93BBA61C058D0F2B1ACFE`.
The [offline gate](../../evidence/stage3-live-evidence-20261015/prelaunch-repair-a04-offline-gate.raw.txt)
is SHA-256 `759017B61C36897C0FDCE658003AED41E450B0B573894245B2F496DDBE3627BD`:
**123/123 preflight, 73/73 runner, 14/14 backup, 9/9 summary** and D:
atomic JSON overwrite passed. A01, a02 and intermediate a03 are historical,
not live-usable. Rust/Cargo source and the preserved EXE/PDB remain unchanged;
the bridge is not a new Rust build, live containment test or app result.
Claude's read-only and peer reviews found no concrete source blocker, but did
not prove a live dead-driver integration path or grant a new one-run approval.
There has been no second real backup, runner reservation, guest launch or
guest write from this workflow.

## A12 reciprocal Send source gate (2026-10-15, executed)

The historical
[A11b expected red](../../evidence/scheduler-source-cut-a11-reciprocal-red/validation.next-lease.receipt.md)
replayed T's inner `SendMessageW` on the next native lease, creating a third
send and duplicate wait before S's callback. The bounded A12
[source seal](../../evidence/scheduler-source-cut-a12/source-seal.md) and
[production validation receipt](../../evidence/scheduler-source-cut-a12/validation.production.receipt.md)
pin the formatted `unicorn.rs` SHA-256
`4A08AFB9FA4AE0072E145DA1588673997D5BBD3D79700B661A2B79EB45E90E07`.
Post-format focused native same-process 6/6, native-send-to 4/4 and
saved-context 6/6 passed. The two 18-group broad suites passed release
2,047/0/9 ignored and no-default 1,482/0/3 ignored; format and diff checks,
alternate feature checks and a release build passed. The normal real-MIPS
positive control runs S→T→S on exact birth/stack, returns inner `0x88` and
outer `0x77` once, retires only their matching sends/waits and detects no
replay. A wrong-parent-birth negative fails before guest entry with original
sends/waits and parent intact. The separate
[focused history](../../evidence/scheduler-source-cut-a12/validation.focused.receipt.md)
records pre-format tests and a diagnostic fixture-assertion correction; it is
not the final source seal. This is one same-process, birth-owned, plain
two-send case. Third sender, timed/result-pointer, deeper pending-callback
and cross-process variants remain unsupported and may fail closed with
terminal `Backend`; no iNavi, writable mount, LAN server or Stage 3/live
acceptance follows.

## A13 orphaned GetMessage source gate (2026-10-15, executed)

The test-only [expected red](../../evidence/scheduler-source-cut-a13-red/validation.receipt.md)
showed a same-birth blocked `GetMessageW` payload being revived from saved
context after its scheduler wait was removed. The bounded
[production receipt](../../evidence/scheduler-source-cut-a13/validation.final.receipt.md)
seals `unicorn.rs` SHA-256
`E12BEEE86A6A9BC2D429052FF79B110C794F8A079F93DFD4594F30669B65BD9F`.
Its [source patch](../../evidence/scheduler-source-cut-a13/source-at-seal.patch)
against the A12 source commit `ef23090d` is SHA-256
`E1EBE0C94D44F3F926A32107C8499DDA62AF2396ECB28DE74C56F7B4B3DCBFED`
and reverse-apply checks against the sealed source. The payload now always
carries an exact birth; matching saved-context promotion is suppressed, and
a selected payload lacking its exact wait fails before guest entry without
consuming the sender reply/wait. Exact live modal-wait ownership also gates
parked readiness and modal continuation; the real-button positive and
missing-wait negative controls are source tests, not live-modal acceptance.
Focused saved-context 7/7, integrated native orphan 1/1 and modal 11/11
passed. Both 18-group broad suites passed release **2,052/0/9 ignored** and
no-default **1,482/0/3 ignored**; Unicorn-only and desktop-without-trace
checks, formatting and diff checks exited 0. No iNavi app, writable mount,
remote LAN server or full Stage 3 acceptance is part of this gate.

## Bounded host-exit PID-reuse source gate (2026-10-15, executed)

The test-only
[expected-red receipt](../../evidence/stage3-host-teardown-red/validation.receipt.md)
preserves the failure in which B's exit after numeric PID reuse destroyed a
stranded prior-birth A independent HWND. The bounded
[production receipt](../../evidence/stage3-host-teardown-20261015/validation.receipt.md)
records the enabled normal control, the full numbered validation attempts and
the final source patch. Verified host process exit now selects independent
live windows and captured backing stores by immutable window creation process
birth; current-process pseudo-exit without a canonical birth rejects before
window cleanup, exit signaling or waiter wakes. After fixture correction to
register synthetic process births and create exact-birth windows, the final
18-group suites passed release **2,055/0/9 ignored** and no-default
**1,485/0/3 ignored**. Earlier broad attempts failed on synthetic fixtures;
they are preserved diagnostics, not final passes. This gate does not
birth-filter recursive GWE foreign parent/owner traversal or numeric
sent-message sender cleanup. There was no iNavi run, writable-mount backup or
guest write, LAN debug session, full Stage 3 acceptance or CE6 parity claim.

## Sender-ownership source gate (2026-10-15, executed offline)

The [expected-red receipt](../../evidence/stage3-sender-birth-20261015/red.receipt.md)
preserves the test-first queued, active and raw stale-caller failures with
attempt-numbered hashes. Its first active attempt failed fixture setup; later
attempts reached the bug. The [final receipt](../../evidence/stage3-sender-birth-20261015/validation.receipt.md)
seals the six tested source/test files at Git HEAD
`e27a4cddb2fe30fc8280f06492a48ea189a65ca9`, with a HEAD manifest and
reversible source patch from `b00272ca`. The native library pass preceded the
now-committed non-hung integration-test timing correction; the final full
suites include it. The receipt lists SHA-256 for the tested files and all
numbered raw logs, and records the
three enabled original red controls. Independent producer/fixture/native
reviews preceded the final validation. Full offline suites passed release
**2,067/0/9 ignored** and no-default **1,495/0/3 ignored** across 18 groups
each; native-feature library tests passed **1,074/0/6 ignored**. Earlier
native, legacy-fixture and near-threshold failures remain in that receipt,
not overwritten by the final pass.

This bounded gate validates exact sender-birth ownership of queued
Send/Timeout and pre-entry guards for the identified guest no-wait producers.
The stranded prior-birth send is synthetic safety evidence, not normal exit
behavior. Direct WndProc dispatch, conditional system-origin storage/device
broadcasts, receiver-side cross-process result writes, and (at that sender
seal) recursive foreign GWE links remained open. CE6 parity, full Stage 3 and
live iNavi remain open. There was no
emulator run, writable-mount operation, LAN session, footprint or latency
measurement.

## Host-exit exact-window-set source gate (2026-10-15, executed offline)

The [final receipt](../../evidence/stage3-host-exit-exact-window-20261015/validation.receipt.md)
preserves the parent/owner and SendReply expected reds, attempt lineage,
reversible source patch, tested file hashes and complete final-suite logs.
The tested source at `32ef955c` selects live host-exit HWNDs only by frozen
creation process birth, even when synthetic parent/owner links cross births;
Kernel uses the actually retired set for payload, shell and timer cleanup.
An exact A→B sender waiting on a retired B receiver HWND is woken once.
Normal guest `DestroyWindow` keeps its existing recursion/callback path.
Final 18-block offline suites passed release **2,072/0/9 ignored** and
no-default **1,500/0/3 ignored**; Unicorn-only and desktop-without-trace
release checks passed. The supplied `C:\Program Files\LLVM\bin` resolved
the native libclang dependency. This is synthetic identity-safety evidence,
not CE6 permission for foreign links. Pseudo-exit caller authority, fallible
whole-exit effects, CE6 topology, full Stage 3 and live iNavi remain open.
There was no emulator run, writable-mount operation, LAN session, footprint
or input-to-frame latency measurement.

## Guest pseudo-exit caller-birth source gate (2026-10-15, executed offline)

The [final receipt](../../evidence/stage3-pseudo-caller-birth-20261015/validation.receipt.md)
seals the tested HEAD, source/test hashes, all numbered raw attempts and
their results. Behavioral reds established raw stale/missing caller, native
classification and decoded missing-caller defects; a fixture compile error
and a zero-test filter attempt were retained but not counted as reds. The
corrected guest COREDLL path requires captured live caller birth and the
native process lease before pseudo or real-handle dispatch. The birth-
qualified pseudo operation signals only the selected exact process birth;
decoded self-PID is classified before public-handle lookup, preventing a
foreign-handle collision. Focused tests include a tiny-MIPS positive import.
Final 18-block suites passed release **2,081/0/9 ignored** and no-default
**1,502/0/3 ignored**; Unicorn-only and desktop-without-trace checks passed.
The tiny-MIPS missing-birth negative is refused before guest entry, so an
actual stale-A/B MIPS import trap remains unproved. At this earlier seal,
immediate native CPU stop, canonical Task retirement, the root return-stub
path and D-011 stopped-exit framebuffer preflight/recovery remained open.
CE6 parity, full Stage 3 and live iNavi also remained open. No emulator run,
writable-mount operation or LAN session occurred.

## D-011 stopped host-exit framebuffer preflight (2026-10-15, executed offline)

The [validation receipt](../../evidence/stage3-host-exit-framebuffer-preflight-20261015/validation.receipt.md)
identifies the final A33 source seal, hashes and full numbered raw logs. A01
preserved a 2/2 behavioral red for un-restorable saved backing. The initial
scheduler fixture A12 was 2 pass/2 fail; its corrected A18 rerun was 4/4.
A24 then preserved a selected-owner behavioral red (0/1): the stopped boot
CPU returned ordinary `false` when the selected process no longer matched its
saved owner. A25 made that control 1/1 green; A26 reran scheduler controls
4/4. A32 then preserved a monitor red: after recovery refusal, a newly saved
checkpoint could be restored to clear the hold. A33 made its focused control
green 1/1 by blocking new checkpoint saves during recovery. The final
18-block suites passed A34 no-default **1,510/0/3 ignored** and A35 release
`unicorn,trace,win32-desktop` **2,094/0/9 ignored**. A36/A37 alternate native
feature checks and A38 format/diff check passed.

The bounded gate validates all exact-birth saved window backings before any
stopped-exit effect; a malformed, incompatible or missing destination returns
`HostExitRecoveryRequired` before mutating windows, backing, sends, waits or
process state. Controls include already-destroyed HWND backing and a later
failing store after an earlier compatible one. A stopped boot CPU whose saved
owner differs from the selected process returns
`HostExitAuthorityRecoveryRequired` instead of ordinary `false`. Scheduler
controls keep active, parked and pending owners for recovery rather than
running another slice. The monitor holds later manual
continue/until/tap and blocks new checkpoint saves until an earlier
checkpoint is restored. Compatible retry succeeds in the
bounded controls. This does not prove canonical Task/VM retirement after a
root pseudo signal or whole-exit atomicity after successful preflight. No
live iNavi run, writable-mount operation or LAN session occurred.
The later live-checkpoint lineage refusal gate below supersedes the proposed
in-session restore recovery: an earlier live checkpoint cannot safely be
restored by the present monitor.

## Stage 2 writable main-image VM and mutable-code source gates (2026-10-16, executed offline)

The historical [writable-image expected red](../../evidence/stage2-writable-image-red-20261016/validation.receipt.md)
showed a real MIPS store visible to one native owner but absent from the
loader blob and a fresh owner when blob readback was deliberately omitted.
The separate [mapped-code R02 red](../../evidence/stage2-mutable-code-read-20261016/validation.receipt.md)
showed a same-slice live instruction overwritten by guest MIPS but decoded
from stale mutable blob bytes; R01 was compile-only. These are bounded
source-fixture failures, not live iNavi observations.

The [VM foundation](../../evidence/stage2-writable-image-vm-20261016/validation.receipt.md)
passed 3/3 focused no-default tests: stable aligned backing keyed by process
birth, module base, load generation and run base; exact native pins and
retirement/reuse fences; independent VM clone bytes. The
[monitor guard](../../evidence/stage2-writable-image-vm-20261016/test-main-checkpoint.validation.receipt.md)
passed 1/1 focused no-default test and refuses checkpoint save/restore when
either machine has writable-image backing. An
[opt-in main-image adoption gate](../../evidence/stage2-writable-image-green-20261016/validation.receipt.md)
passed 3/3 focused release controls. Its real-MIPS `0x51a7c0de` store persists
through canonical VM bytes to a fresh native owner with zero image
copy-in/readback in that synthetic path; other controls reject mismatched
final image runs/second load attempt and retire a partially published later
run without dropping the original CPU blobs. Attempts 01–04 in that focused
directory lacked native Cargo output and are explicitly uncredited; attempt
05 is the captured test proof.

The later [mapped-code fallback gate](../../evidence/stage2-mutable-code-fallback-20261016/validation.receipt.md)
passed 5/5 focused release controls: mutable instructions come from live
native bytes and a failed native read no longer falls through to stale blob
bytes. Its [final broad receipt](../../evidence/stage2-mutable-code-fallback-20261016/broad-validation.receipt.md)
seals HEAD `18c6c9268fcedc050a2a53778b4ad720b56e98ba` plus five stable
source-file hashes and complete captured raw output. Across all 18 result
blocks per profile, release passed **2,116/0/9 ignored** and no-default
**1,515/0/3 ignored**. Earlier 8/2 ignored subtotals excluded one ignored
test in a zero-passed block of each suite; the receipt was corrected without
altering the raw logs.

The ordinary production SD main-image loader remains copy-backed and DLL
families are not migrated. The synthetic backing reports physical status
`Unclassified` and charges zero guest-budget bytes; it cannot close O-M3 or
authorize production image physical-memory accounting.

The later [synthetic native-map failure gate](../../evidence/stage2-writable-image-native-failure-20261016/validation.receipt.md)
passed its final focused release control **1/1**. A test-only collision after
pinning the second writable main-image run forces a real `mem_map_ptr` error;
both runs remain inaccessible while pinned. Checked abort-close retires the
native slot and staged continuation, and the last retained pin clone fences
bytes until dropped. This tests a map failure, **not** a failure of native
owner close. Its first default-harness broad release attempt crashed before
any test-result block on a WINDOWPOS test-thread overflow and is uncredited
([failed broad receipt](../../evidence/stage2-writable-image-native-failure-20261016/broad-validation.receipt.md)).
The [serial-harness complete suites](../../evidence/stage2-writable-image-native-failure-20261016/serial-broad-validation.receipt.md)
then passed release **2,117/0/9 ignored** and no-default **1,515/0/3
ignored**, and a later [default-harness full release retry](../../evidence/stage2-writable-image-native-failure-20261016/default-retry.addendum.receipt.md)
passed **2,117/0/9 ignored**. Those passes do not explain the intermittent
test-thread crash.

The [writable-image/legacy file-view overlap gate](../../evidence/stage2-writable-image-file-view-20261016/validation.receipt.md)
passed focused release **5/5** and no-default **1/1**: exact guest-range
preflight rejects same/foreign PID partial overlap before legacy seed or
readback mutates VM, native or file bytes. It fences retired runs while their
last pin/guard remains, and accepts adjacent nonoverlapping sync. Its
[complete default-harness suites](../../evidence/stage2-writable-image-file-view-20261016/broad-validation.receipt.md)
used stable Rust source hashes and passed 18-block release **2,119/0/9
ignored** and no-default **1,516/0/3 ignored**. The release HEAD advanced
only across a `KNOWN_BUGS.md` documentation backup; this is not a source
drift claim.

The subsequent [writable-image/VM-allocation overlap gate](../../evidence/stage2-writable-image-vm-allocation-20261016/validation.receipt.md)
passed focused release **13/0/0** and no-default **8/0/0** controls. It
preflights global VA-only LegacySeed in both publication orders, same-birth
stable private/shared reservations, pending holes, fixed heap arenas and
monotonic heap cursor, while permitting a foreign-birth private page to reuse
the numeric VA. Native seed, stale reclaim, fault repair and low-level map
paths reject image collisions before ledger or byte mutation. This includes
a forged stale ledger as a defensive control, not a reachable ordinary VM
publication claim. Default-harness complete suites passed release
**2,127/0/9 ignored** and no-default **1,520/0/3 ignored** across 18 blocks
each; six Rust source hashes matched through the four attempts despite a
backup-only HEAD advance during the first focused run. Unicorn-only and
Unicorn-plus-desktop feature checks also passed on the same source seal.

The [synthetic exact-process-exit/retained-pin gate](../../evidence/stage2-writable-image-exit-retire-20261016/validation.receipt.md)
passed focused release **18/0/0** and no-default **12/0/0** controls. After
verified exact-birth completion and D-011 framebuffer preflight, only that
birth's image runs retire; retained pins reject host writes but preserve
diagnostic readback after owner close. Default-harness complete retry suites
passed release **2,132/0/9 ignored** and no-default **1,524/0/3 ignored**;
alternate Unicorn feature checks passed. The first full release attempt had
one named `FAILED` line, then exited `0xc0000005` without a result block. A
passing isolated named test and complete retry do not explain that access
violation or prove its cause. The native fixture covers after-owner-close
retirement, not exit during an active mapped engine.

The intermittent native `0xc0000005` incidents remain under one unresolved
bug entry. A [matched serial/default release-library cohort](../../evidence/parallel-native-av-cohort-20261016/validation.receipt.md)
at unchanged Rust source ran three attempts in each harness arm; all six
passed **1,133/0/6 ignored**, observing **0/3** crashes per arm. These
library-only results do not reproduce or explain either prior full-suite AV.
Native nonzero-timeout execution starts a timer worker; a single Rust
emulation thread does not prove native single-thread safety. The
WINDOWPOS-specific stack probe remains a narrower observation:
an [isolated direct-EXE probe](../../evidence/stage2-windowpos-stack-20261016/validation.receipt.md)
passed at requested 128 KiB but overflowed at 64 KiB, while
[three later default release library reruns](../../evidence/stage2-windowpos-stack-20261016/lib-rate.validation.receipt.md)
had **0/3** crashes, versus **1/3** in an earlier, separate full-suite
default-harness sequence. The VM-allocation release pass is a fourth,
later full-suite observation on changed source, not part of that earlier
three-run cohort. Neither denominator establishes cause or production stack
safety.

Other direct host-write/restore/rebind and exact module-retirement
fences, machine checkpoint relineage, DLL migration and production activation
remain separate gates. No app, mount, LAN, RSS, throughput, map-readiness or
full Stage 2 result follows.

## CE6 process-exit offline REA/source check (2026-10-16, observed)

Exact-hash device `nk.exe`, `kernel.dll` and `coredll.dll` were inspected
without running them; REA records and source hashes are retained in the
[finding](../../evidence/ce6-rea-exit-20261016/findings.md). The bounded
exported-COREDLL wrapper observations and local CE6 process/thread source
support per-process exit ownership. The dumped kernel's exit body was not
identified, and no emulator source gate or live-app run follows from this
static check. All three temporary NTFS analysis copies were removed after
their evidence bundles were exported.

## CE6 wait/child-lifetime REA/source follow-up (2026-10-16, observed)

The [bounded follow-up](../../evidence/ce6-rea-exit-20261016/findings.md#2026-10-16-bounded-waitchild-lifetime-follow-up)
reopened exact-SHA dumped `kernel.dll` and `coredll.dll` through temporary
NTFS copies, exported 12 REA records per target, verified bundle hashes and
removed both exact scratch copies. Dumped COREDLL's `WaitForSingleObject`
wrapper calls `0xffffbb9a`; `GetExitCodeProcess` resolves its handle before
calling `0xffffba8e` and conditionally releases the temporary resolution.
The bounded kernel string/callsite pass identified a secure-stack-failure
thread path, **not** `PROCTerminate`; the ROM exit body remains unknown. Local
CE6 source shows child loader-completion signaling, process-event wait pins,
exit signaling after resource cleanup but before final object/thread teardown,
and termination-request/actual-exit separation.
The implication for O-T7 is independent child CPU custody across creator
exit and alias closure. Static wrappers plus source are not a ROM-to-source
function mapping or emulator/live-app acceptance.

## Bounded parked-CPU survivor and saved-CPU-owner source gate (2026-10-16, executed offline)

The [preserved natural-parked/active duplicate expected reds](../../evidence/stage3-parked-survivor-red-20261016/validation.receipt.md)
showed independently live foreign B CPU trees being lost when the exiting A
tree was removed. The [combined sealed receipt](../../evidence/stage3-survivor-resume-green-20261016/validation.receipt.md)
records the exact source and numbered attempts. At HEAD `6aeeb21c`,
`unicorn.rs` SHA-256 was
`D172BF79A5A5990B707BC980C22ACC1C63C74B49475BCE15BF1C88DEE15D751B`.
In the tested natural-parked and requested-active stopped-exit paths, exact
birth and live Kernel ownership are preflighted before effects; after
accepted process retirement, original nonmodal foreign CPU boxes, saved
contexts, waits and descendants are moved into the survivor queue rather
than discarded or cloned. A foreign saved-context CPU must have a nonzero
current TID matching its saved exact thread birth, and that birth must remain
execution-live. Three negative ownership controls refuse exit before effects;
a live representative-worker control confirms that the parked record's
numeric representative TID is not CPU authority. Focused release passed
**4/0**. Complete 19-block suites passed release **2,144/0/10 ignored** and
no-default **1,524/0/3 ignored**. Earlier passing a09 on source `286A3FDF…`
and the a06 alias-close regression are intermediate/superseded, not credited
to this seal.

At that seal, first-slice child `Exited`, other active exits, requested parked siblings,
already-exited cleanup, generic duplicate pruning, foreign CPU nodes without
saved context and PC resumability remain separate gates. The static
[CE6/REA check](../../evidence/ce6-rea-exit-20261016/findings.md) did not
locate the dumped kernel exit body. No iNavi app, writable mount, LAN,
footprint, responsiveness or full Stage 3 result follows.

## Bounded first-slice child Exited survivor source gate (2026-10-16, executed offline)

The [preserved expected-red receipt](../../evidence/stage3-first-slice-survivor-red-20261016/validation.receipt.md)
records two synthetic production-helper controls: successful first-slice A
exit lost a nested live B CPU, and malformed saved B ownership was accepted.
The subsequent [source-gate receipt](../../evidence/stage3-first-slice-survivor-green-20261016/validation.receipt.md)
identifies the final source hash and validation attempts. On its bounded A→B
shape, the helper preflights B, holds parent publication capacity before
Kernel completion, and moves the original B CPU, context and wait after
accepted A exit. It refuses mismatched saved B birth before effects. A third
control deliberately holds a parent queue borrow: A's exit is refused before
Kernel effects and the fallback returns typed terminal `Backend` without a
host panic. This conflict drops the local A+B CPU tree, so it is not a
recoverable hold or the target's infallible publication boundary.

The current normal callsites at `unicorn.rs:13530` and `:14611` do not hold
that parent queue borrow. Existing concrete framebuffers do not reenter the
queue during exit completion; a custom reentrant implementation is not
supported by this cut. At that seal, the separate `Completed` queue borrow and
an exact live primary-thread-birth gate remained open. These tests do not execute a
guest MIPS first slice, establish classifier reachability or iNavi behavior.
The [CE6/REA finding](../../evidence/ce6-rea-exit-20261016/findings.md)
is static ownership context, not a decompilation of the dumped kernel exit
body. No app or writable-mount run was made.

## Real-MIPS nested first-slice survivor expected-red (2026-10-16, executed offline)

The [four-attempt receipt](../../evidence/stage3-real-mips-first-slice-20261016/validation.receipt.md)
preserves the on-disk MIPS PE/IAT fixture, stamped raw logs and hashes. In
attempt 03 A calls the real import trap for `CreateProcessW`, launches B, and
returns through its saved startup RA with `0x63`. B is separately exact-birth
Kernel-live and unsignaled; after B is activated and A is pruned, the outer A
launch sees `CreateProcessChildParkSkipped(already_exited)` and discards the
local CPU now belonging to B. The asserted parent CPU queue is zero versus
one required owner. At that seal the final source test was ignored
expected-red; the normal focused selection passed with one ignored. The
fixture was later retired after missing post-deferral preconditions. The
historical natural route uses
`parked_prune_exit`, not the synthetic `initial_child_exit` arm, and covers
only `live_pump=false`. No production repair or app/mount run occurred.

## Post-admission first-slice native-fault custody red (2026-10-16, executed offline)

The [five-attempt receipt](../../evidence/stage3-post-admission-native-fault-20261016/validation.receipt.md)
seals a test-only ignored synthetic MIPS child fixture. The child enters a
native slice and faults on `lw` at unmapped `0x2000_0000`; the trace requires
`READ_UNMAPPED`, while its exact Kernel birth remains live, `STILL_ACTIVE`
and unsignaled. The current error route returns without publishing or
retaining its CPU: parent custody queue **0**, expected **1**. Final
`unicorn.rs` SHA-256 is
`508101e2942f997472de99c1765905756104e06caf6313ac96717292e3250c79`;
attempt 04 is the source-sealed expected red (exit 101), attempt 05 confirms
normal focused selection has one ignored/zero failures. Future-green checks
also require the faulting PC and nonzero saved PC. The attempt-local scratch
directory was verified absent after the panic. No production path, app or
writable mount was run or changed; no broad suite was run on this test-only
addition.

## Pre-CPU published-launch retention (2026-10-16, bounded source gate)

The [preserved expected red](../../evidence/stage3-pre-cpu-launch-retention-20261016/validation.receipt.md)
made outgoing lazy file-view validation fail before path resolution or child
CPU construction. It left the child's exact Kernel birth live and unsignaled
but drained its pending launch (zero retained versus one required). The
[green receipt](../../evidence/stage3-pre-cpu-launch-retention-green-20261016/validation.receipt.md)
seals a bounded source cut: pre-CPU file-view, birth, path, PE-inspection and
CPU-constructor errors restore the current exact launch ahead of its tail.
The same-birth retry and malformed-PE tail controls passed 2/0, two related
pending-child controls passed 2/0, and normal/no-default feature checks
passed. Post-construction DLL-load, native and publication errors still need
machine-owned CPU custody; no app or writable mount was run.

## Live checkpoint lineage refusal (2026-10-16, bounded source gate)

The [live CPU expected reds](../../evidence/stage3-live-checkpoint-lineage-20261016/validation.receipt.md)
showed that monitor save/restore cloned a live CPU while retaining its exact
process birth. The separate [pristine-CPU Kernel reds](../../evidence/stage3-kernel-checkpoint-lineage-20261016/validation.receipt.md)
showed that boot, active Task and queued-child births could likewise be
duplicated without a live CPU. The [green receipt](../../evidence/stage3-kernel-checkpoint-lineage-green-20261016/validation.receipt.md)
seals a fail-closed monitor guard: save and restore refuse live CPU custody,
pending launches or canonical Kernel process/thread records; restore checks
both source and destination CPUs. Five former reds passed under release and
no-default profiles, and the existing checkpoint selection passed 4/0 with
five ignored under both profiles. Positive selection is now synthetic and
fully retired; it does not demonstrate live machine relineage. Once D-011
requires recovery, the monitor cannot currently rewind a live machine in
session; quit, repair and restart remain necessary. No app or writable mount
was run.

A later [orphaned-frame expected red](../../evidence/stage3-checkpoint-orphan-frame-20261016/validation.receipt.md)
proved that filtered process/thread snapshots were an incomplete refusal
predicate: an exited, aliasless thread could retain a finite-Sleep frame
while both lists appeared empty, and save/restore copied its original birth.
The [canonical-map green receipt](../../evidence/stage3-checkpoint-orphan-frame-green-20261016/validation.receipt.md)
seals a direct process/thread-record guard. The seven lineage controls passed
under release and no-default profiles; at that earlier seal ordinary
checkpoint selection passed 4/0/7 ignored, serial no-feature passed
**1,525/0/10 ignored**, raw Kernel integration **133/0**, and four feature
checks passed. It does not clean up
the synthetic orphan, make live rewind safe, or establish whole-machine
checkpoint relineage. No app or mount run occurred.

The later [counted checkpoint receipt](../../evidence/stage3-checkpoint-unignored-20261017/validation.receipt.md)
unignored those seven `src/main.rs` controls without changing other tested
source. Its complete 19-block no-default suite passed **1,532/0/3 ignored**
and release-feature suite **2,157/0/13 ignored**; all seven controls were
counted green. Failed native-build and sandbox-loopback attempts remain
separately hash-preserved. This does not lift the live save/restore fence.
Three [pre-deferral offline expected reds](../../evidence/stage3-completed-reentry-red-20261017/validation.receipt.md)
then reproduced the `Completed` held-queue `RefCell` panic/survivor loss,
already-exited-A dropping B's original CPU, and admitted-child native-fault
CPU loss. At that seal all three were ignored; no app or writable mount was
run. The two stale route-specific
fixture implementations were subsequently retired; the native-fault
custody control was later replaced by the ordinary Cut 3a focused witness.

Separately, the earlier [combined boundary regression](../../evidence/stage3-custody-boundary-combined-20261016/validation.receipt.md)
used the same four-file source seal for five stamped attempts. The four
normal-feature integration suites passed **908/0**, and serial no-feature
`cargo test -j 1` passed **1,525/0/8 ignored**. Each closed raw-log hash was
verified against the receipt. This earlier seal predates the direct-map
checkpoint guard and does not execute the ignored post-admission owner-loss
reds, a full release-feature suite, or the mounted app.

## Deferred CreateProcessW post-native source gate (2026-10-17, executed)

The [bounded receipt](../../evidence/stage3-createprocess-deferred-20261017/validation.receipt.md)
seals `src/emulator/unicorn.rs` SHA-256 `4BE7F3F2…EAB69D`. Successful guest
`CreateProcessW` now stages the parent's nonzero return PC and defers child
launch until after the native attempt in both non-live and live-pump modes.
Non-live normal rotation may choose B before A resumes. On that source, the
focused non-live selection passed **5/0**, and one two-mode file-view reseed
control passed **1/0**. Complete release-feature and no-default suites each
finished 19 result blocks: **2,161/0/13 ignored** and **1,532/0/3 ignored**,
respectively. The file-view control checked canonical host mapping bytes
after reseed, not a guest-visible post-lease read.

Three ignored final-source reruns have distinct meanings:
[attempt 16](../../evidence/stage3-createprocess-deferred-20261017/attempt16-current-completed-red.raw.log)
and [attempt 17](../../evidence/stage3-createprocess-deferred-20261017/attempt17-current-first-slice-red.raw.log)
exited 101 at unmet pre-deferral fixture preconditions, so neither
reproduces nor repairs the old `Completed`/A-exit failures.
[Attempt 18](../../evidence/stage3-createprocess-deferred-20261017/attempt18-current-native-fault-red.raw.log)
exited 101 at the sole-child-CPU assertion after an ordinary native fault;
that custody failure remains current. CPU slots, a callback-scoped
machine-state port, live-pump later-slice survivor behavior, Stage 5 policy
and live iNavi acceptance are unproved. No app or writable mount was run.

## Direct first-slice Completed empty-descendant gate (2026-10-17, executed offline)

The [bounded receipt](../../evidence/stage3-completed-borrow-20261017/validation.receipt.md)
seals HEAD `0fb3bc0eaabb64fc153c1b62248cb2f3d6189561`, `unicorn.rs`
SHA-256 `0dcf5c5d04d0e632cbe405554bfc7029b7eea654070d4ca1fbdc3070ac136d06`,
and an empty source patch. The stamped
[attempt 01 expected red](../../evidence/stage3-completed-borrow-20261017/attempt01-direct-completed-expected-red.raw.log)
executed a direct pseudo-exit MIPS child through window-backing restoration
and first-slice `Completed`. A test framebuffer held a parent-queue read
borrow; the old unconditional `borrow_mut` panicked even though the child had
no parked descendants. The current source skips that empty publication and
uses `try_borrow_mut`/`try_reserve` for nonempty descendants. A nonempty
conflict is reported as typed terminal `Backend` after committed exit effects
and can still lose descendant CPU custody.

[Attempt 05 focused](../../evidence/stage3-completed-borrow-20261017/attempt05-sealed-focused.raw.log)
passed **1/0** (closed-log SHA-256
`8c2d284d7ad9d3588e2923cd86c816df29e5725886536c9866eba56049197adb`).
[Attempt 04 release](../../evidence/stage3-completed-borrow-20261017/attempt04-final-broad-release.raw.log)
passed 19 blocks, **2,162/0/11 ignored** (SHA-256
`1572c11aae63bff3f1c10b970316ecfdf8da14864b95e2277d2abd73787bc48d`),
and [attempt 06 no-default](../../evidence/stage3-completed-borrow-20261017/attempt06-final-broad-no-default.raw.log)
passed 19 blocks, **1,532/0/3 ignored** (SHA-256
`0c6e6ae48b478f1db5f26ebad4cf20e0afc60b518df5e53bf835421358df7428`).
Attempt 03 used an earlier source hash and is superseded. Attempt 04 crossed
a backup-commit HEAD change, but `unicorn.rs` SHA stayed unchanged before
and after; attempts 05–07 used the final HEAD and the same source hash.
The two old ignored `Completed`/A→B fixture implementations were retired
because A13 reruns 16/17 missed their post-deferral preconditions; their
earlier red logs remain historical evidence. The admitted-child native-fault
control was still ignored and red at that seal:
[attempt 07](../../evidence/stage3-completed-borrow-20261017/attempt07-current-native-fault-red.raw.log)
exited 101 at the child-CPU-custody assertion (SHA-256
`52df312df7b39ba3ba58bcbbc786604f7f480dc8bf2354dcb2e0702609b387e4`).
This is not a nonempty-survivor, later-slice or live-app result. No app or
writable mount was run.

## Live-pump fixed-stack fault-hook witness (2026-10-17, test-only)

The [bounded receipt](../../evidence/stage3-live-pump-fixed-stack-hook-20261017/validation.receipt.md)
seals test-only `unicorn.rs` SHA-256
`ff0bce3c3976c5f71bbcddbb2e35d182dd7d221552223529ed49b9eac720abd0`.
Real MIPS `SW`/`LW` first-touches an uncommitted fixed-stack page under
`live_pump:true`; the exact-birth page pin, stored bytes, progress registers
and nonzero saved PC are checked. One ordinary test uses an explicit PC stop
and global code hook; the second uses `stop_pc:None` with default hookless
count-limited execution. The production memory-fault repair route is inferred
from source exclusivity, not counted callback entry. Focused release passed
**2/0**; [broad release attempt 03](../../evidence/stage3-live-pump-fixed-stack-hook-20261017/attempt03-broad-release.raw.log)
passed 19 blocks **2,164/0/11 ignored** (closed-log SHA-256
`1cd2c2f9bff7a3861390e8a447939d2dfbe6fe39b02ce4a468418fae7f3e2c1b`);
[no-default attempt 04](../../evidence/stage3-live-pump-fixed-stack-hook-20261017/attempt04-broad-no-default.raw.log)
passed 19 blocks **1,532/0/3 ignored** (SHA-256
`1a0ce9a1aa6f59761768c62593e7ff2a85842a0d57ae072e4affe12e537b0df8`).
No production Kernel/framebuffer loan, callback reentry or CPU-custody fix
was made. No iNavi, GPS or writable-mount run occurred.

## Pending-launch primary-identity red (2026-10-17, process-free)

The [stamped current-source receipt](../../evidence/stage3-primary-claim-red-20261017/validation.receipt.md)
reruns the existing ignored Stage 3 prerequisite explicitly. The one control
failed at its intended still-live child primary-TID assertion after 129 newer
launch records evicted the 128-entry diagnostic ring (`EXIT_CODE=101`, closed
raw-log SHA-256 `fd4a26674c12bd582993a196ddc7222af72e13b509fd597ca5fb65cd2ed4ff41`).
The source HEAD and three selected file hashes were unchanged during the
attempt. Production drains Kernel's entire pending launch queue before child
execution, so primary lookup can fall back to that diagnostic ring. This is
evidence for a connected exact-birth Kernel/launcher claim cut, not its fix or
a guest-app observation. No iNavi or mount run occurred.

## Shipped iNavi process-launch REA check (2026-10-16, static artifact)

The [hash-preserved REA finding](../../evidence/ce6-rea-inavi-20261016/findings.md)
uses the original MIPS PE SHA-256 and 30 retained REA Evidence records. A
direct `CreateProcessW` branch checks a `happyway_win` window, builds
`%s\happyway_win.exe` and stores four return words; a second checks `iSearch`,
builds `%s\iSearch.exe` and likewise stores the result. This corroborates
child-process custody as a shipped-app requirement, but neither branch was
executed in this check. Ghidra's MIPS calling convention and MIPS III decoding
remain assumptions/limitations. The original, mounts and emulator were not
run or modified; the temporary exact-hash NTFS analysis copy was removed.

## Dumped COREDLL thread-exit REA follow-up (2026-10-16, static artifact)

The [retained REA finding](../../evidence/ce6-rea-exit-20261016/findings.md)
and [raw evidence bundle](../../evidence/ce6-rea-exit-20261016/coredll-exitthread-followup-rea-evidence.json)
use an exact SHA-verified analysis copy of the dumped CE6 MIPS `coredll.dll`.
`ExitThread` checks a KData-derived value, sets an `IsExiting` flag on one
branch, calls `PrepareThreadExit`, performs branch-dependent cleanup, and
invokes the `0xFFFFBBCE` kernel method with the exit code. `TerminateThread`
uses a separate resolved-handle method at `0xFFFFB7DE`, and
`PrepareThreadExit` calls `0xFFFFBAB2`. The named-function search found no
`ExitProcess` wrapper in its bounded result. These are static wrapper facts,
not a decoded ROM kernel exit body, a proven main-thread condition, or live
child-lifetime behavior. The original binary was not modified; the temporary
analysis copy was removed.

## Opt-in native timer rendezvous (2026-10-16, executed diagnostic)

The [bounded receipt](../../evidence/native-timer-rendezvous-20261016/validation.receipt.md)
records 48/48 fresh-owner iterations across timer-first, hook-first and
simultaneous-release orders (16 each). Each returned from both C boundaries,
reported native `TIMEOUT=1`, `COUNT_STOP=0`, PC `0x1000`, and closed the owner
under a 60-second external process-tree watchdog. Accepted build and test raw
logs are stamped, NUL-free and hash-sealed. Two earlier zero-corrupt logs and
one failed-link log remain preserved without success claims. The probe is
opt-in; no production timeout or C path changed. This non-reproduction neither
explains earlier full-suite AVs nor establishes race freedom, iNavi timing or
footprint.

## Canonical primary and v2 prelaunch gates (2026-10-17)

The [current-source primary red](../../evidence/stage3-primary-claim-red-20261017/validation.receipt.md)
failed after pending-launch drain and diagnostic-ring eviction. The bounded
[canonical-primary gate](../../evidence/stage3-primary-canonical-green-20261017/validation.receipt.md)
now stores an exact primary thread birth on each canonical process and passes
five focused controls, plus stable 19-block release **2,169/0/10 ignored** and
no-default **1,537/0/2 ignored**. Its first broad release attempt had a
mid-run validation-wrapper source change and is explicitly invalidated. The
[dumped COREDLL REA evidence](../../evidence/ce6-rea-primary-thread-20261017/findings.md)
supports durable main-thread identity but is static, not a live-app witness.
Exit reconstruction, full CPU custody and boot-main enumeration remain open.
The [pre-Cut-3a native-fault red](../../evidence/stage3-cpu-custody-current-red-20261017/validation.receipt.md)
failed with a Kernel-live admitted child and no parked CPU owner; it is
process-free and does not implicate the real iNavi image.

The [v2 offline preflight receipt](../../evidence/stage3-preflight-v2-review-20261017/validation.receipt.md)
passes **171/171** source/build-chain controls and **83/83** runner guards.
It rejects historical a04 provenance and requires wrapper-owned post-build
EXE/PDB hash stamps, a matching current HEAD and executed focused tests.
No new clean rebuild or mounted run was performed.

## Bounded exact-primary stopped-exit gate (2026-10-17, executed)

The [synthetic expected-red receipt](../../evidence/stage3-primary-exit-birth-20261017/expected-red.receipt.md)
records same-process primary-TID reuse and missing-primary authority before
the source cut. The subsequent stamped
[a10 broad release attempt](../../evidence/stage3-primary-exit-birth-20261017/broad-release-a10.raw.log)
failed its library block at **1,144/16/7 ignored**, exit 101, on earlier
`unicorn.rs` SHA-256 `782DE852…30685D`; it is preserved failure evidence,
not a broad gate. After the boot-owner and synthetic stopped-CPU fixture
corrections, the [v2 focused guest-thread-stack rerun](../../evidence/stage3-primary-exit-birth-v2-20261017/focused-guest-thread-stack-a01.raw.log)
passed **133/0/2 ignored** on `unicorn.rs` SHA-256 `CE560FE0…003833`.
Its before/after file hashes matched despite an intervening HEAD change;
focused success alone was not broad acceptance.

The final [v4 receipt](../../evidence/stage3-primary-exit-birth-v4-20261017/validation.receipt.md)
seals `unicorn.rs` SHA-256 `1EDEB353…916347`. A focused stopped-boot
missing-primary recovery/non-exit control passed **1/0**. Complete 19-block
release-feature and no-default suites passed **2,175/0/10 ignored** and
**1,539/0/2 ignored**; checks for Unicorn, Unicorn plus desktop, and the
normal feature set exited 0. These controls gate exact canonical primary
birth and stopped CPU authority for their tested child/boot exit shapes;
they do not establish every exit route, Task/Frame authority, O-T7 CPU
custody, live-pump later-slice survival or CE6 parity. No app, GPS or writable
mount was run.

## Bounded scheduler-block self-reentry fence (2026-10-17)

The [sealed receipt](../../evidence/stage3-scheduler-block-fence-20261017/validation.receipt.md)
records `unicorn.rs` SHA-256 `F93D7557…221277`. Its real-MIPS scheduler
service with a **preseeded** busy flag refused with typed Backend/nonzero PC,
unchanged selected PID/framebuffer and closed native owner. Complete
loopback-enabled release-feature and no-default suites passed
**2,176/0/10 ignored** and **1,539/0/2 ignored** over 19 blocks each; all
three Unicorn feature checks exited 0. The initial sandboxed release attempt
failed 45 socket/remote cases with Windows `10013` and remains preserved;
the same sealed source passed when loopback was allowed. This guard catches
scheduler-block self-reentry only. Natural or cross-hook reentry, modal
partial-effects recovery, whole-attempt port and O-T7 CPU custody are open.
No live app or mount run was made.

## Bounded native-entry precondition (2026-10-17)

The [sealed receipt](../../evidence/stage3-native-entry-precondition-20261017/validation.receipt.md)
records `unicorn.rs` SHA-256 `55642d7b…32ed3d`. A same-host-thread guard
refuses native guest reentry before private CPU setup. The real-MIPS
**preseeded** busy control verifies typed refusal and same-CPU retry; the
initial late-guard retry failure remains preserved. Sealed focused release
passed **1/0**; complete release-feature and no-default suites passed
**2,178/0/10 ignored** and **1,539/0/2 ignored** over 19 blocks each, with
three feature checks exiting 0. This guard does not establish natural
reentry, a machine-owned leaf permit, a callback-scoped Kernel/framebuffer
port, or custody after a child native fault at that guard's seal. The earlier
scheduler-block
receipt's suggested shared import/block flag is superseded by the current
source audit: those hook families execute sequentially in the sole native
entry. No live app or mount run was made.

## Cut 3a non-runnable admitted-child CPU quarantine (2026-10-17, bounded source/test gate)

An ordinary real-MIPS guest A calls `CreateProcessW`; its nonzero return stays
staged after the parent native attempt's clean sync/finish while B is launched
post-native. B first touches its fixed stack with `SW`/`LW`, then triggers
`READ_UNMAPPED`. Earlier focused debug/release controls passed; superseded
producer-side a09 debug passed: the original B CPU is
retained in one exact-birth, generation-checked, non-runnable machine custody
slot, not the legacy parked tree; its native owner is closed and unsafe saved
context cleared while B remains Kernel-live and unsignaled. A
`TerminateProcess` request alone does not retire it. Same-Kernel checked
accepted exit retires the slot and the fixed-stack pin in that focused path.
Stamped a07 expected red reproduced a synthetic post-load/pre-native-entry
unpoisoned quarantine and stopped at that test assertion. Source inspection
shows the exit consumer would refuse its poison preflight; that consumer
refusal was not executed. Producer-side
sealing made a08 debug pre-entry and a09 debug real-MIPS fault controls green
on superseded `unicorn.rs` SHA-256
`1AAE8A66ACFFFE3D8FE9CED10687C98A28879BF9267A1736D7B4689F21A33090`.
Intermediate producer-only a10 focused release passed but is superseded.
Current source makes `CpuCustody::quarantine_reserved` seal the CPU under
Unicorn on `Reserved`→`Quarantined`, retaining the exit consumer's poison
backstop. The [owner-sealed receipt](../../evidence/stage3-cpu-custody-quarantine-20261017/validation.receipt.md)
passes a11 focused release **1/0**, 19-block a12 release
**2,184/0/9 ignored**, a13 no-default **1,543/0/2 ignored**, and a14–a16
three feature checks exiting 0; all six source hashes matched before and
after each counted attempt. The final-source real-MIPS control is in a12, not
a separate final-source focused run. This forced precondition does not prove
natural reachability or general error-path
closure. The main run loop owns
the directory; monitor save/restore refuse it; status/debug publish keys,
birth, state and retained mapped-blob bytes. These bytes are not a measured
host RSS peak. The Kernel-minted exit witness carries exact birth only, so
the production consumer relies on same-Kernel coupling, not a cross-machine
lineage proof. General private VM and mapped-file-view exit cleanup remain
separate [open bugs](../../KNOWN_BUGS.md). Runnable/parked custody, public-run
bypass removal, the callback-scoped Kernel/framebuffer port, full O-T7 and
live-app acceptance remain open.

The pre-seal a03 release broad **2,183/0/9 ignored** remains development
evidence, not this final source gate. No app or writable-mount run was made.

## Post-native Parked publication custody expected red (2026-10-17, test-only)

The [bounded receipt](../../evidence/stage3-parked-publication-red-20261017/validation.receipt.md)
preserves a01's fixture-expectation miss and the a02 stamped, focused ignored
test. A02's closed raw log has SHA-256
`CF1BDE452EA360E0C747C19968AF34897B9AA5D43D5D4C978FF551F9C74354E3`
and `unicorn.rs` SHA-256
`C5F6A48CDEE38DA122EAC8960BD84E3F5A900BAE628CFD6995A4AC48BBD3EF38`.
Real guest A executes `CreateProcessW`; the post-native B CPU executes
fixed-stack `SW`/`LW`, with the stored value verified, and classifies
`Parked`. Only at B's publication, a
test-only framebuffer gate holds the parent parked queue's read borrow.
Production returns typed `Backend`, while B stays exact-birth Kernel-live,
unsignaled and wait-unready. The final expected-red assertion fails at zero
machine-owned B slots versus one; the legacy parked queue is also empty.
This is a forced queue conflict, not natural reentry, a live-pump result or
a live iNavi failure. No production ownership code changed. Cut 3a's
non-runnable fault quarantine does not repair this runnable/parked boundary;
full custody remains behind the connected callback-scoped port and leaf
permit. No app or writable mount was run.
The same-source loopback-enabled a04 release suite passed 19 blocks,
**2,184/0/10 ignored**; a05 no-default passed 19 blocks,
**1,543/0/2 ignored**; three feature checks a06–a08 exited 0. A03's
sandboxed release attempt had 45 failures concentrated in socket/remote
tests with multiple explicit Windows `10013` errors; a04's identical command
passed when loopback was permitted, supporting an environment explanation.
The receipt preserves all closed log hashes and matching source stamps.
The test-only one-shot seam and ignored red fixture were later retired from
current `unicorn.rs`, whose SHA-256 is
`B6C7FC7C89517F390D4309974F384B37B733EAC97B0250421F4DA03C984DC300`.
The patch and raw logs remain the historical witness. This removal is not a
production custody fix; the runnable/parked publication bug remains open.

## Selected-process context provenance stamp (2026-10-17, bounded source/test gate)

The [stamped a01 no-default attempt](../../evidence/stage3-selector-context-stamp-20261017/a01-focused-no-default.raw.log)
ran `cargo test --lib selected_process_context_stamp -j 1` and passed
**5/0**; its closed raw log is SHA-256
`6BCFB4A9F3B33E1F969DDA24CBB636D515BB5EDE5C026556EA4CA5DC44FDBBFC`.
The [a02 focused release attempt](../../evidence/stage3-selector-context-stamp-20261017/a02-focused-release.raw.log)
also passed **5/0** (closed log SHA-256
`67CF6AC0A92F6DFC3B936E21A90DC445228B8EA9F072C20D7A7D5AA02AB2C05B`).
The [a03 broad release attempt](../../evidence/stage3-selector-context-stamp-20261017/a03-broad-release.raw.log)
passed 19 blocks **2,189/0/10 ignored** (closed log SHA-256
`1282B80FDF2A1167F0CE615EFE51A9BC146A0DCA66E4B1AE50974506924B2F91`).
The [a04 broad no-default attempt](../../evidence/stage3-selector-context-stamp-20261017/a04-broad-no-default.raw.log)
passed 19 blocks **1,548/0/2 ignored** (closed log SHA-256
`293D8AFBB084E13299B1AC07485BC30C0D8CA529CF89899E0C77B53D99ADB9FC`).
Feature checks [a05 unicorn](../../evidence/stage3-selector-context-stamp-20261017/a05-check-unicorn.raw.log),
[a06 unicorn+desktop](../../evidence/stage3-selector-context-stamp-20261017/a06-check-unicorn-desktop.raw.log)
and [a07 normal](../../evidence/stage3-selector-context-stamp-20261017/a07-check-normal.raw.log)
each exited 0; their closed log SHA-256 hashes are respectively
`199CFABD789039B0C455AF38CFF40DDB9212BEF53819C0D66E08747D5BF4E6F5`,
`C0AF659B29D372E9864EECFCBD6D368DD3EDB5B01EB4428DDD002C2D342FF8BB`
and `D350DD19458E334DB248E052DCC3D4125FA2F3CAA8D855F91DBA689728648F7E`.
`kernel.rs` was SHA-256
`756636D2866FD8F6C656591EED66D7BFB5D97F61FCE46809A17E45D1D890B1D6`
before and after all seven attempts, with unchanged hashes for the six other
stamped source files. The [validation receipt](../../evidence/stage3-selector-context-stamp-20261017/validation.receipt.md)
ties them to one source seal. The source-only gate introduces a crate-private stamp
of selected canonical birth plus checked generation. Tests cover all selected
metadata/state setters, accepted versus refused exit compatibility writes,
A→B→A selection, missing birth, PID replacement through public
`HandleTable` without a selector write, and permanent provenance poisoning
on generation overflow.

This is not a production restoration gate. No handoff consumer checks the
stamp; cloned Kernel lineage, direct backing-object mutation, execution-live
ownership, a callback-scoped port and an exact parent-suspension token remain
open. No live iNavi or writable-mount run occurred.

## Unconnected machine-state port foundation (2026-10-17, development evidence)

The [partial receipt](../../evidence/stage3-machine-port-foundation-20261017/validation.receipt.md)
preserves the a01 stamped `cargo test -j 1 machine_port_` attempt: three
library tests passed, zero failed, exit 0; other binaries were filtered.
The closed [raw log](../../evidence/stage3-machine-port-foundation-20261017/a01-focused-no-default.raw.log)
has SHA-256 `5F9C070161500EC44AF98EBE5CCEF995A7EC501E98644D07C355AA02A337EEC5`.
The same-source [a02 focused release log](../../evidence/stage3-machine-port-foundation-20261017/a02-focused-release.raw.log)
records `cargo test --release --features unicorn,trace,win32-desktop --lib
machine_port -j 1`: **3/0/0**, exit 0, SHA-256
`4964F7C55C4E1D94B2A5FB2AB867952C8BBC3C7187FF889BD6BE9EA02352DFB9`.
The [a03 broad release log](../../evidence/stage3-machine-port-foundation-20261017/a03-broad-release.raw.log)
records `cargo test --release --features unicorn,trace,win32-desktop -j 1`:
19 blocks **2,192/0/10 ignored**, exit 0, closed SHA-256
`BA54DF0289153B5686012F1319BA2DE641F12295A8B75E662AAFCB7F17BAAA7B`.
Its source hashes match before and after, although backup commits moved
HEAD from `fa7de51ab790af3c0f72467265f171c396055413` to
`a105546f326181cee410383f7cc4fa80b83eb4f6` during that attempt.
The preserved parked-publication red was still ignored in this pre-retirement
a03 broad suite. Its test-only fixture has since been removed from current
source without a custody repair. This broad pass is development validation,
not port acceptance.
Before/after source hashes matched for the new
`src/emulator/machine_kernel_port.rs`
(`89DACB64AFA8ACAC394E8CF9DC809D649E1D81D2A7164888F1B7D3E9332BA2F0`),
`src/emulator/mod.rs`
(`200E384F3E7138A9A39E9F1FF6160AED69B411FEF12AB09E689E8CCB945E48BB`),
and unchanged Kernel/Unicorn
files. The preserved combined source patch has SHA-256
`B9695971CE6233ACEB7140E58FC8DCF10362549B4FBE97EA14C9617345691A55`.

The source only introduces a crate-private joint Kernel/framebuffer loan
cell, typed busy/attempt failure and first-terminal latch. The focused tests
exercise owned observation after loan release, nested cross-resource busy
refusal without panic, and terminal precedence. There is no independent
compile-fail fixture, production caller, attempt/leaf authority, native-hook
or child-helper migration, callback teardown proof, selection restoration,
event fence or CPU-custody effect. Broad no-default and feature-matrix
checks remain pending; this is not Step 2 or live iNavi acceptance. The next
connected cut needs a production consumer for the selected-context stamp;
the parked red remains evidence to preserve; its ignored fixture is no
longer active in current source. The runnable/parked custody bug stays open.

## Chat-project validation notes (not in this repository)

The pages cite the following notes by name. Their originals are available in
the separate ChatGPT project workspace under `validation/`; they have not yet
been imported into this repository. Until then, this ledger is the portable
summary, not a substitute for the full run receipts. If importing the notes,
place them in `docs/evidence/notes/` and link each matching entry.

| Note | Summarized in |
| --- | --- |
| `20261013-stage0-arena-initialization-differential.md` | E-5 |
| `20261013-stage0-native-commit-tail-differential.md` | E-6 |
| `native-tail-veh-abi-matrix-1013/REPORT.md` | E-6 |
| `native-tail-leaf-thunk-1013/README.md` | E-6 |
| `native-tail-page-rounding-1013/README.md` | E-6 |
| `native-tail-veh-lifecycle-1013/README.md` | E-6 |
| `20261013-leaf-full-backend-integration-gate.md` (design only) | E-6 |
| `stage2-full-backend-1013/mapping-context-1` | E-7 |
| `20261013-stage2-full-backend-mapping-stop-context-controls.md` | E-7 |
| `run-outcome-consumer-map-1013.md` (design only) | E-7 |
| `20261013-task-ownership-expected-red-baseline.md` | E-8 |
| `20261013-task-identity-api-expected-red-baseline.md` | E-8 |
| `20261013-task-first-production-slice-audit.md` (design only) | [tasks](../subsystems/tasks-scheduler.md) |
| `20261013-stage2-cross-process-same-va-fpu-control.md` | E-10 |
| `20261013-stage2-kernel-virtual-page-staged.md` (design only) | E-10 |
| `20261013-task-pin-expected-red-receipt.md` | E-11 |
| `task_pin_expected_red_1013.rs` (test-only fixture) | E-11 |
| `20261013-task-directory-decisions-proposal.md` (design only) | [tasks](../subsystems/tasks-scheduler.md) |
| `20261013-fresh-engine-vm-coherence-review.md` (design only) | [memory](../subsystems/guest-memory.md) |
| `20261013-fresh-default-policy-design.md` (read-only source audit; no tests run) | [CPU](../subsystems/cpu-engine.md), [memory](../subsystems/guest-memory.md), [O-P4/O-M1](../DECISIONS.md) |
| `20261013-stage8-full-backend-churn-control.md` | E-12 |
| `20261013-fresh-engine-source-delta-audit.md` (read-only porting map) | [plan](../PLAN.md) |
| `validation/paint`, `validation/end-paint` | [GDI](../subsystems/gdi-framebuffer.md) |
