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
cause; no dump survives; no evidence implicates a particular shim or loader.

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
