# Progress

Refreshed on 2026-10-15 (client KST date). This file records confirmed facts only.
Older investigation ledgers were purged; use commits, logs, and targeted source
references for historical reconstruction.

## 2026-10-15 A11 reciprocal Send expected red (test only; no app run)

The first real-MIPS reciprocal S→T→S send control is an intentionally ignored,
test-only [expected red](evidence/scheduler-source-cut-a11-reciprocal-red/validation.receipt.md).
It was sealed at HEAD `cc1a6d101991e8c68fdeaed2f8ca8bd8d3be896d` with
`src/emulator/unicorn.rs` SHA-256
`4C1BFEEFE60D8F293A5CA8BA887069BE05D0E59EC5E7E854BA0D829162F75194`
and test-only patch SHA-256
`64C1BCFDBBB6378BBD5B946E08CF913E850F3BA29656E7FA050C378413E0DDED`.
The selected test exited 101 at the intended S callback boundary, after real
MIPS Sleep, S's outer Send, T's GetMessage callback entry and T's inner Send.
Both exact send IDs and scheduler/native waits remained present, but execution
stayed on T's saved PC rather than entering S's WNDPROC on S's stack. This is a
reproduced source-control failure, not a live-app deadlock or a fixture
precondition failure. No production repair or broad suite followed A11; it
does not change A10's passed bounded source/test gate or authorize an iNavi run.

## 2026-10-15 Connected A→B Send/GetMessage bounded source gate (no app run)

The ordinary same-process A→B `SendMessageW` into B's birth-verified blocked
`GetMessageW` now has a bounded A10 source/test gate. The exact
[source seal](evidence/scheduler-source-cut-a10/source-seal.md) pins HEAD
`d6013e7996271be811a95eb6215299089a8f7414` plus a two-file source/test
patch SHA-256 `AF497AE8FB4B9A0F19D76715FA94FA9E747FD8B21A8848832AAB80D0DB7F8C49`.
Both 18-group broad suites passed on that source: release-feature **2,045
passed, 0 failed, 9 ignored** and no-default **1,482 passed, 0 failed,
3 ignored**. HEAD, patch, Rust source and pinned toolchain hashes matched before
and after both suites and additional checks. Unicorn-only and desktop/no-trace
checks and the release build exited 0; selected EXE/PDB copies are SHA-preserved
in A10 evidence ([validation receipt](evidence/scheduler-source-cut-a10/validation.receipt.md)).

The original A→B expected reds and ordinary queued-send receiver-exit/TID-reuse
red are historical. A07 broad suites passed on prior source; A08 exposed
sender-restore corruption and its repair passed focused Send **30/30** and
Kernel **1/1** on the prior A09 source. The sealed A09 release-feature run
failed four existing same-process Send controls (**1,048 passed, 4 failed,
6 ignored**); no-default was not run
([A09 receipt](evidence/scheduler-source-cut-a09/validation.receipt.md)).
Strict extra-lease controls then showed that saved-context recovery could
revive B despite its separate blocked-GetMessage slot. A10 checks the exact
GetMessage/native wait and saved/live birth before recovery, rejects a known
stale saved birth before native restore, and resumes A on the next bounded
lease with its exact reply/wait. Only the runnable-receiver control checks a
later no-replay slice; the three finite-Sleep controls stop at A's continuation.
This is not full Stage 3 or live-iNavi acceptance. Reciprocal cross-Send,
cross-process parked-sender restore, legacy numeric-only HWND ownership,
post-wake MsgWait, `SMF_TIMEOUT` last-error semantics and same-birth orphaned
GetMessage payloads with a missing scheduler wait remain separate boundaries.

## 2026-10-15 TaskFrame finite-Sleep scheduler source cut (no app run)

The first machine-owned finite-Sleep `TaskFrameRef` source cut is resealed at
HEAD `6626e74d`. `ThreadRecord` owns its mutable frame phase; scheduler waits
and native register payloads carry exact-birth/frame locators. A sleeping
receiver's same-process sent message stays queued until real guest retrieval:
real-MIPS `GetMessageW`, `PeekMessageW(PM_REMOVE)` and `GetMessageWNoWait`
controls enter WNDPROC and return its `0x77` result to the blocked sender.
Pre-deadline idle reports `NoAttempt` instead of replaying S's send. Focused
controls passed finite-Sleep **17/17**, native same-process **4/4** and native
stale-HWND **1/1**; KData activation failures cancel only the new frame/wait,
and an old birth-owned HWND cannot borrow a replacement TID's frame.

The first release attempt was diagnostic (**1,032/2/6 ignored**): two hidden-
message fixtures lacked authentic saved-context births; they were repaired
without relaxing the production guard, then source was resealed. On the
resealed HEAD, full release passed **2,025/0/9 ignored** and no-default
**1,472/0/3 ignored** across 18 result blocks each. Unicorn-only and
Unicorn+desktop/no-trace checks, the release build, format/diff checks and
post-build source-hash verification passed
([receipt](evidence/stage3-finite-sleep-frame-20261015/validation.receipt.md)).
This is a source/test gate, not full Stage 3 or a live-app result. Reciprocal
cross-Send after receiver wake and post-wake MsgWait remain untested; neither
is a reproduced failure. The user skipped the a04 run; no new adjacent mount
backup or iNavi launch followed this scheduler cut.

## 2026-10-15 Historical receiver-context Send expected reds (no app run)

Before the connected cut, three ignored real-MIPS controls reached same-process
cross-thread send: ordinary A→B after B blocks in verified-birth
`GetMessageW`; a 64-instruction slice stop *inside* B's WNDPROC; and
A→displaced C after C and B both block in GetMessage. Each failed at the
native slice-end guard with
`Backend("native staged context has stale or unverified thread birth")` before
receiver-stack/return assertions. The [test-only receipt](evidence/stage3-getmessage-blocked-send-context-20261015/validation.receipt.md)
preserves distinct a02–a04 source patches, hashes and complete targeted raw
logs. Historical a01 has an exact patch and excerpt only; its complete raw
output was not retained and its waiter assertion was corrected in a02. The
file originally named `expected-red-a01.raw.log` was mislabeled a02 output;
its exact bytes are retained once under the a02 name, with the move and
temporary duplicate/removal recorded in the receipt. The
a03 report confirms a real count stop within B's WNDPROC; a04 proves the
target C was displaced before A sent, not that the ready-ID bridge selected
the wrong receiver. Source inspection shows the old in-hook shortcut derives
callback SP from A's live registers, but no red directly measured B's SP or
proved a reciprocal deadlock.

Plan-first review selected queue→birth-verified GetMessage resume at a slice
boundary instead of an in-hook TID/register relabel. At that pre-repair point,
the connected cut still needed frozen ordinary receiver birth, old-birth exit
retirement, exact send-ID completion and checked receiver/sender restoration.
An ignored
[receiver-exit/TID-reuse Kernel control](evidence/2026-10-15-ordinary-queued-send-receiver-exit-reuse-red.md)
failed as expected: B-prime took B's queued send while A had no failure
result. These receipts preserve historical expected-red evidence, not A10's
passing source/test status. No reciprocal fixture or iNavi launch was
established by them.

## 2026-10-15 Architecture and measurement design refresh (no source/run)

The common [Stage 7 plan](docs/PLAN.md) now requires reviewed, device- and
interaction-grounded footprint/responsiveness budgets before “small” or “fast”
acceptance; the two-second frame observer currently supplies bounds, not exact
visual lag. [App evidence](docs/app-evidence-lane.md) and
[verification](docs/subsystems/verification.md) point to the a04 script-only
seal and preserve its no-backup/no-run/no-approval limit. The raw-NMEA payload
is new, not an exact E-38 input match.

The proposed [event-to-frame lineage gate](docs/subsystems/remote-observability.md)
separates host ACK, machine adoption, guest dispatch/read, and a machine-sequenced
frame commit; a later changed image alone cannot prove input causation. The
[finite-Sleep Task/Frame plan](docs/subsystems/tasks-scheduler.md) now uses
CE-reachable sent-message deferral and GetMessage→WNDPROC→Sleep continuation
controls. The earlier two-simultaneous-Sleeps-on-one-thread expected red was
withdrawn after local CE6 source review: positive Sleep blocks its caller and
does not dispatch a WNDPROC. These are design/test specifications only; no
Rust/script change, new native test, backup or guest run followed this refresh.

The user then chose to **skip the a04 run and begin the scheduler source cut**
(reply to `call_49d1b609465a4e668da30a31361beb44`). No new mount backup or
guest launch was authorized by that choice. Once the cut changes source, a04
is historical evidence only; any later live attempt needs a fresh source/build
seal and a new specific approval covering its backups and bounded run.

## 2026-10-15 Live-run preparation and aborted backup (no app run)

The repaired pre-watchdog D: source had a [22-file script-only a02 seal](evidence/stage3-live-evidence-20261015/prelaunch-repair-a02-source-seal.json)
(SHA-256 `1DC1537217C621EBD1B8C2D5E682C50B9CFA6FA131D22CC3202F54C7C5651F97`)
and [build bridge](evidence/stage3-live-evidence-20261015/prelaunch-repair-a02-build-bridge.json)
(SHA-256 `A3746E1729186997CC684030E5E5BA772A1A3038132F49E5BAE9A04C6A21D580`).
The W-B1c-scoped patch is SHA-256
`8CF8CA60072D4F0ECDA8B21D5400A6B00F25344671D8082E374145F19CDAD18C`.
Its [offline script gate](evidence/stage3-live-evidence-20261015/prelaunch-repair-a02-offline-gate.raw.txt)
passed 123 preflight, 40 runner, 14 backup, 9 observer and atomic JSON overwrite
controls. Rust/Cargo and `.gitattributes` are unchanged from the prior clean
build; its preserved EXE/PDB pair and matching RSDS are carried forward, **not
rebuilt**. A02 and interim a01 are now historical and **not live-usable** because
the runner has changed again; the earlier
[21-file seal](evidence/stage3-live-evidence-20261015/final-source-build-seal-receipt.md)
and its 97/40/14/9 offline gate remain historical evidence. The pre-run Rust
telemetry gate passed 1,996/0/9 release and 1,456/0/3 no-default tests, plus
two alternate feature checks. The narrow guest stable-page budget and
per-server summary stamp remain source-tested, not live acceptance.

Approval `call_9b35e9d31fdf402ea082a3eb6a40ae16` started an adjacent ZIP
backup, but the old `File.Replace` helper failed after verifying only the SD
archive. The [aborted-backup disposition](evidence/pre-live-mount-backup-20261015T025500Z/aborted-approval-disposition.md)
records its 5,973,898,396 bytes and SHA-256
`6032F822551CFB5F28ED079CB5B41CB02B1585F8AFF16EA303B54A74B0C2C949`;
no ResidentFlash or EmulatorRoot ZIP, completed triplet, runner or guest launch
followed. The incomplete SD ZIP remains adjacent and cannot be reused. The
approval was withdrawn/void; a deny-only `withdrawn` row brought both
byte-identical ledgers to seven rows (SHA-256
`53BAD1ABCAEA7F723737014E182228922122951F1CFB0574773E49EE3DFD1A08`).
See the [ledger](docs/evidence/ledger.md) for the evidence chain and limits.
A new, distinct approval is required before any further backup or launch.
The current [a04 source seal](evidence/stage3-live-evidence-20261015/prelaunch-repair-a04-source-seal.json)
(SHA-256 `C495AB5324BABB1DCF252B8DCCB6A0B5EC4A58C3D04632E3728D39C6AFFA376C`),
[full patch](evidence/stage3-live-evidence-20261015/prelaunch-repair-a04-full-scoped.patch)
(SHA-256 `B9FBE5F1DA42453BE8213D2FD45B4553126B62F9641A2003FEB4D2FC26C35CCB`)
and [script-only build bridge](evidence/stage3-live-evidence-20261015/prelaunch-repair-a04-build-bridge.json)
(SHA-256 `3C84522E0D5DF2B6B0BA7BFD8DA8BB14747E4270BDB93BBA61C058D0F2B1ACFE`)
bind the unchanged clean-built EXE/PDB to the new scripts. Intermediate a03
is also superseded, not live-usable. The [a04 offline gate](evidence/stage3-live-evidence-20261015/prelaunch-repair-a04-offline-gate.raw.txt)
(SHA-256 `759017B61C36897C0FDCE658003AED41E450B0B573894245B2F496DDBE3627BD`)
passed 123/123 preflight, 73/73 runner, 14/14 backup, 9/9 summary and the
D: atomic JSON overwrite test. The runner adds an independent `WatchdogWorker`: its exact
process identity and heartbeat are checked before the launch request, and it
enforces a monotonic 355-second deadline from the post-mount scan, with
two-second RSS/raw-log probes and exact-identity cleanup on failure. The
existing detached monitor's 390-second guard remains. An atomic `launch-began`
marker precedes guest spawn; an empty WMI search after that marker is unresolved,
so the watchdog stays active and retries exact cleanup. Independent containment
assumes the D: evidence/mount volume makes I/O progress; a volume-wide stall
also stalls guest writes on these mounts. Claude's read-only and peer reviews
found no concrete source blocker, but did not prove a live dead-driver path or
authorize a run. No new backup or guest run followed the aborted attempt.

## 2026-10-15 Stage 3 source gates

Each gate is bounded; none is full Stage 3 or live-app acceptance. Newest
first. Totals are passed/failed/ignored for release
`unicorn,trace,win32-desktop`, then no-feature. Linked receipts identify the
source and validation evidence; a source gate is not a live EXE/PDB or app-run
result. The per-slice text before this condense is at
`git show d4263dde:PROGRESS.md`.
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
  exit 0 at 330 s, splash only, no map; RSS peak 430.7 MiB (BED2 555.6 MiB,
  persistent runs ~1.5 GiB); iNavi busy-slice 26.4 MIPS, inside E-9's 22–34; live heap
  reached 31.5 MB. That approval is spent.
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

## 2026-10-13 Parallel Continuation

- Stage 2 facts on the D: checkout, 2026-10-13/14. Details, receipts and suite
  tallies are in ledger E-12 to E-29; the pre-condense narrative is in
  `git show 3e4b52d6:PROGRESS.md`.
  - D-003 keeps Stage 2-6 on the fresh-engine lifetime; persistent owners are
    the Stage 8 swap.
  - The repaired-PC scalar fallback lost F0-F31/FCSR. It is fixed by carrying
    FPRs and FCSR in `MipsGuestContext`, with checked capture and restore onto
    a blank engine (E-12, E-13).
  - Native file-view sync is owner-qualified, and the injected two-PID same-VA
    view control is green (E-14, E-16).
  - Stop classification: `UnknownNormalStop`; typed `HookStop` reasons
    (`HostWallBudget`, `VisibleParkedPeerYield`, `RemoteControlService`,
    `ReadyParkedPeerYield`); `CountLimit` only from the vendored
    `UC_QUERY_COUNT_STOP` witness; native errors hard-fail (E-15, E-17, E-19,
    E-28).
  - At the E-25 checkpoint, `LegacySeed` lost guest-written private bytes on
    a fresh owner; only the `cfg(test)` slab opt-in made E-18 green. D-008
    selected AddressSpace-owned stable slabs mapped with `mem_map_ptr`.
  - Process birth keys and alias lifetime; owner-qualified private-VA metadata
    and file-view mutation. The public two-PID same-VA allocation was still
    red at that checkpoint (E-20 to E-24).
  - The 13 historical Task controls live in `tests/` (1 pass, 12 expected red)
    (E-26).
  - Bounded 330 s live runs exit 0 but stay on the splash. GPS bytes and taps
    reach the guest; parsing and map use are unproven. Persistent-owner memory
    peaks at about 1.49-1.55 GiB (E-22, E-27, E-29).
- D-007 now requires this D: checkout as the only active source and one Cargo
  target. The validated Unicorn 2.1.5 dependency was copied in-repo (567/567
  matching file hashes) and selected via Cargo's path patch. The initial
  focused native count, nonzero unmapped-error and live pre-run rotation
  controls passed here. After outgoing file-view readback was ordered before
  PID rotation and stale typed outcomes were cleared on preflight swaps, the
  serial full release suite passed **1,623 tests, 0 failures, 3 ignored** in
  this dirty persistent-owner checkout before the E-12 repair above. The
  separately run ignored closed-owner fixture reached its intended final
  assertion and failed because the
  default still retains the native owner; its preceding guest store and native
  no-timeout checks passed. O-P1/O-P4/O-M1 and a bounded app run remain open;
  this intermediate checkpoint is not fresh-engine Stage 2 acceptance.
  Exact log/binary hashes and the expected-red receipt are in local
  `evidence/stage2-e7-stop-20261013/post-order-and-retirement.receipt.md`.
  The 243 immediate target-root run-evidence files and 28 release EXE/PDB
  files were separately copied and hash-verified under local `evidence/`
  receipts. After explicit approval, those 271 originals were
  reverified and removed by exact paths; nested target directories remain intact.
  The root-only `/evidence/` ignore rule keeps large local artifacts ignored
  while `docs/evidence/ledger.md` is Git-visible.
- Persistent-owner build 650 host-crashed at 129.1 s with 1.66 GiB RSS after
  showing the splash by +4.800 s (ledger E-2).
- The 1 GiB per-engine translation arena keeps its commitment through hook
  churn (E-3, E-4).
- Small pre-init arenas crashed from a commit-tail arithmetic defect (E-5,
  E-6). It is fixed in the vendored source (E-33, D-009).
- The full-backend mapping/context control passed. The stop/error controls
  started red and are green since O-P1 (E-7, E-10).
- The Task/alias controls are the Stage 3 baseline (E-8, E-11, E-26).
- Stage 3 child-exit/worker sweep: real process waits signal only after
  exact-birth verified completion; stale worker contexts fail closed;
  completed child CPUs do not replay entry
  ([receipt](evidence/stage3-sweep-20261014/validation.receipt.md)).
- O-P1 count witness (diagnostic baseline 4E0A): positive-count and voluntary
  stops report expiry 0, a genuine deadline reports 1. Its EXE/PDB are kept as
  `native_outcome_baseline_1013.{exe,pdb}`.
- HeapReAlloc/LocalReAlloc/CRT realloc map, copy the retained prefix and zero
  only the grown tail before committing allocation metadata; IME resize uses
  the same boundary. Failure keeps the old allocation, and partly written
  destinations stay unused monotonic holes. Heap reuse and page reclamation
  remain open.
- Bundle A3BA00D6 330 s GPS/tap run: exit 0 at 330.239 s, peak RSS 566.0 MiB
  (no late-memory improvement), first splash by +4.631 s, 206 location
  responses and positive COM7 ReadFile payloads consumed. Taps arrive FIFO but
  take 1.578 s, and handlers return in about 22 ms. Every frame was the
  splash. The report is in the chat workspace
  `validation/bringup-1013-heap-drive-evidence.md`.
- That run executes 1,037,557,247 guest instructions across 3,406 slices;
  weighted busy-slice MIPS is 12.846 versus preceding 12.314. Guest work and
  late phase rates are comparable; no controlled speed improvement is claimed.
- Explicit nonzero PC stops preserve native registers, thread/process,
  pending callback state and ownership before thunk/callback repair or
  handoff. Import traps honor the stop before dispatch, and PC zero remains an
  error. Continuation and trace selection need the fresh matching stop and
  retained context, never a historical snapshot.
- Binary evidence (read-only): the splash manager show/destroy methods are at
  0x5e7a0/0x5e880, WM_INITDIALOG startup is at 0x2bc34, and the first large
  initializer is 0x30e1c (return 0x2bde4). Pool worker 5 starts at 0xe6cd0.
  A valid first cache-miss node does not establish whole-job completion. No
  completion event or app state is forced.
- Direct visible callbacks bypassed the deferred typed-payload serializer, and
  NotifyIcon output overran its 152-byte allocation. CE6 coredll ordinal 481
  marshals exactly 0x98 bytes, so the fix corrects the shared payload writers
  rather than enlarging the buffer. Sparse persisted heap pages use a
  contiguous mapping frontier.
- MapInit enters and returns normally on UI thread 1 with V0=1, on stack
  0x7ffdf558 (0x20 below the 0x5237 caller's 0x7ffdf578). The old WNDPROC
  fallback restored only PC/RA/V0. Authentic archived SP/GPR/HI/LO (including
  K0/K1) restoration now passes native callback and caller-epilogue
  regressions. Missing or ambiguous ownership fails closed.
- Callback-drive 330 s GPS/tap run (installed BC2E13E0): exit 0 at 330.236 s.
  0x5237 recovery restored UI thread 1's original stack at +2.796 s. Splash
  appeared by +4.683 s and COM7 by +27.293 s, and every later frame was the
  splash, not the map. 199 GPS responses and positive COM7 payloads were
  delivered, with parsed position unproven. Tap DOWN/UP had 0.835 s initial
  lag and about 22 ms per handler. Peak RSS/private was 492.6/503.2 MiB
  (A3: 566.0/568.0), not cache-controlled. Report:
  `validation/bringup-1013-callback-drive-evidence.md` (chat workspace).
- That run executes 1,434,175,567 guest instructions in 3,211 slices at
  weighted busy MIPS 14.071. iNavi contributes 1,002,008,727 at 13.140 MIPS;
  most extra aggregate work is happyway (432,073,151 versus prior 2,072,966),
  not a pure speed improvement or proof of startup completion.
- Binary audit identifies the second startup initializer at 0x31970, return
  0x2be44. It creates the pool and dispatches jobs, without a top-level wait.
  Owner+0x39dd4 is the concrete MDC object (vtable 0x4e57dc, method 0x12e15c),
  which invokes engine virtual +0x14c then can sleep/wait and repeat. A finite
  cache lookup is not whole-job completion. Actual matching invocation/return
  and current object reads are the next diagnostic boundary, not forced events.
- Checked initial guest allocation now reserves without live metadata, maps
  writable persistent RAM, explicitly zeros requested physical extent, runs
  checked initializers, then commits/publishes. All 36 immediate raw/native
  callers route through it; four host-queued typed reservations materialize
  at delivery. Logical-zero allocations still have one physical byte. Failure
  retains monotonic holes; no address reuse/page reclamation is introduced.
  CREATESTRUCT failure now tears down only the failed new window and returns
  zero, not raw HWND success; scratch lives through both create phases only.
  Clipboard/IME/array/cache publication and cleanup are checked without an
  invented rollback guarantee for already-live partial outputs.
- Native current/blocked GetMessage and SetWindowPos delivery now prepares
  known typed payloads before queue consumption/callback exposure. Sent
  messages use exact send identity across expiry; MSG output requires all
  28 bytes mapped RW. Unknown guest pointers stay untouched. Legacy wrappers
  lacking mapping context defer known payloads for a mapped successor; they
  are not claimed fully mapped. Failure preserves wait/FIFO/context; SetPos
  returns handled-zero rather than repeating an already-applied operation.
- Exact frozen source passes 17 focused tests and all 1,571 tests (one eVC4
  fixture ignored), release build, two no-trace checks and stub check, fmt and
  diff checks. Installed SHA256 354AA0FB...2E4327; prior BC executable/symbols
  are preserved as `pre_boundaries_1013.{exe,pdb}`. Logs:
  `target/bringup_1013_boundaries_fixed_*`. The initial compile-only Prot API
  mismatch log is preserved separately; no failing build was installed.
  Its requested 330 s GPS/tap/frame run `bringup_1013_boundaries_drive` FAILED
  early at 260.174 s, exit 1/no watchdog. PC becomes zero on pool worker 10,
  RA0x000e6de4/SP0x7feff1c8 after recorded ready-event wait resume (id4029,
  handle0xff4, result0). Suspended worker5 retains PC0x00321c98. Do not suppress
  this CPU error or call it a normal timeout; causality of the combined changes
  is not yet established. Unit/source acceptance does not establish live stability.
  Native restored continuation versus JALR target must be distinguished.
- That failed run still observes genuine splash by +3.411 s, COM7 by +27.877 s,
  actual positive GPS ReadFile, and FIFO tap DOWN/UP with clean returns. DOWN
  delay is 1.489 s, handlers24.373/22.080 ms; no map presentation. Peak sampled
  RSS/private509.3/520.6 MiB; the late cleanup dip is not a memory improvement.
  Workload differs (iSearch rather than happyway executes much more work), and
  duration is shorter than BC; no controlled performance claim is made.
  Full outcome/evidence report is in chat validation; executable354 remains
  identifiable and prior BC backup is preserved for explicit comparisons.
- Frozen BC exact diagnostics independently reach second-init 0x31970 on
  TID1/SP0x7ffdf578/RA0x2be44 and MDC engine return 0x12e180 on TID5, V0=1.
  Live object/vtable and saved-frame chain match the binary job binding.
  Current MDC body SP is 0x7ff9f1b0, not the older candidate 0x7ff9f190.
  Missed later stops in bounded monitor chunks do not prove lost continuation;
  remote debug publication is stale during interactive continuation. The
  owned monitor quit normally; terminal image is still genuine splash.

- Bounded exception diagnostics and six genuine native handoff controls pass
  1,580 tests (one unavailable eVC4 fixture ignored), release and three feature
  checks, fmt/diff. Freeze A206/89F9 builds executable528F776B...EC705B;
  it was not installed or live-run. Checked current GPR/IAT values are not
  claimed historical cause. Logs: `target/bringup_1013_fault_diag_*`.
- Two additional native caller-helper fixtures then reproduce generic defects:
  native delay block0xb000 normalized to source0x1100 causes interrupt20/PC0
  despite a valid IAT; completed-delay block0xc000 resumes its recipient cleanly
  but source replay increments callerV0 twice (0x105 rather than0x103).
  Separate failure logs are preserved. These do not yet prove the actual
  wait4029 app failure traversed that boundary.
- A minimal entry-only helper guard now defers interior scheduling instead of
  treating the original source PC as a saved continuation. Frozen Unicorn
  5152C897...A17CE5/types89F9 adds true/false native branch, entry, deferred
  replay and direct predicate controls; independent source review accepts it.
  Exact freeze passes 1,585 tests, one unavailable fixture ignored, release,
  three feature checks and fmt/diff. Installed12DFB7B4...0CAFF5 after all62
  Rust files matched; prior354 executable/symbols are preserved as
  `pre_safe_handoff_1013.{exe,pdb}`. Normal330s GPS/tap run completes330.268s,
  exit0/no watchdog/empty stderr, beyond the prior failure window. Authentic
  original5237 stack restoration and actual GPS payload transfer continue.
  This is bounded stability, not exact attribution of every prior live crash.
- Latest12DF run accepts208 GPS updates; retained ring proves148 positive
  COM7 reads/34,800B. Tap DOWN/UP are FIFO with0.777s first-delivery lag and
  22.628/22.575ms handlers. All observed/terminal frames remain splash, exactly
  matching BC. Capture started19.706s late, so first-splash timing is unavailable.
  PeakRSS/private556.031/571.316MiB; helper workload differs sharply from BC
  and354, and an extra GET-only memory observer was added. No controlled gain
  is claimed. Fullguest1,015,592,266/iNavi1,013,426,929 instructions;
  busy-weightedMIPS13.250/13.231. Logs:`target/bringup_1013_safe_handoff_drive_*`.
- Saved helper backing observations are77/78MiB RAM, while separately retained
  global live heap is30.085MiB. Ages and cached non-atomic PID brackets prevent
  exact simultaneous ownership or RSS attribution; capacity/native bytes and
  per-PID live heap are unavailable. No speculative reuse/reclamation landed.
- A subsequent native raw-yield/fresh-engine audit reproduces silent wrong-path
  execution for a not-taken branch at page-delay0xb000:3 controls pass,1 fails,
  sourceA676. Separately, native ctl_get_timeout returns configured duration,
  not expiry; production at that point used it as an expiry flag. A query-only
  correction was held because count-tick rotation/cleanup/restart depended on
  the false wall flag; the later connected D: correction is recorded above.
  Four counted replay tests over64 native boundaries
  pass; they do not cover full backend cleanup/process rotation.
  No new states/priorities/hooks or CPU-error suppression were introduced.
- The unsafe raw-yield fallback is now removed: its existing non-import and
  no-pending-return gates imply contextNone, so it never served a safe boundary.
  The earlier safe eligible-peer policy is preserved in a shared decision.
  Four genuine native both-outcome tests now defer actual helper interiors,
  execute the selected branch unaided, and freshly replay a safe destination
  with exactly-once effects. ExactEE18/89F9 freeze passes1,593 tests, one fixture
  ignored, release/three feature/fmt/diff checks; installedBED2F559...276E4F
  after all62 Rust files match. Prior12DF executable/symbols are preserved in
  `pre_safe_visible_yield_1013.{exe,pdb}`. No commit/push.
- BED2 normal330s run completes330.234s, exit0/no watchdog/empty stderr.
  Prelaunch frames observe genuine splash by4.358s, not full boot; a separate
  overlapping tail covers the final frame without extending the first watcher.
  Original5237SPf578 restoration remains intact.208GPS responses and a retained
  146-read/35,148B COM7 ring prove payload transfer; tap FIFO first callback
  takes0.397s, handlers22.746/22.898ms. All frames remain splash, not map.
  PeakRSS/private555.633/568.383MiB is essentially12DF556.031/571.316, not a
  memory gain. Profileguest1,009,828,063/iNavi1,007,664,687; busyMIPS13.126/13.107.
  Extra observers and variable scheduling prevent controlled performance claims.
  Artifacts:`target/bringup_1013_safe_visible_yield_drive_*`; detailed chat report
  `validation/bringup-1013-safe-visible-yield-drive-evidence.md`.
- Full terminal trace-all exposes no fault/probe marker only in its selected
  retained helper67 GetMessage snapshot, not all native attempts. Summary emits
  optional interrupt/invalid markers but omits memory faults; raw nativeResult
  and actual expiry were absent from that BED2 profile. The later diagnostic
  freeze logs both without changing timeout/error/rotation decisions. Full map and
  authoritative heap PID/reserved-footprint/incarnation remain unfinished.

## 2026-10-12 Parallel Continuation (baseline)

The bounded-file baseline: 1,527 tests passed, one eVC4 fixture ignored; both
no-trace Unicorn release checks pass. Exact mirror validation logs are
`target/bringup_1012_bounded_files_{tests,build,check_ci,check_unicorn}.log`.
That executable SHA256 is `C274BEE2...ADD1D1`; no commit/push was made.
The prior executable/symbols remain in `pre_bounded_files_1012.{exe,pdb}`.

- Corrected callback lifetime without expanding the stack grace: retain an
  outer direct send while a later same-thread, lower-stack callback is live.
  Real MIPS regressions verify nested return order and saved SP/S0/FP.
- Removed historical child-lifetime traces from pending-CreateProcess
  scheduling and preserve the selected slice-end rotation through run entry.
  The loop-head wall-limit check also bounds paths that execute no slice.
  A 600 s mounted run exited 0 at 600.45 s without an external watchdog.
- Decoded the YAS526B command/selector protocol and Yamaha B/C packed samples.
  The deterministic ideal sensor models calibration, rough/normal commands,
  bank ordering, packed 11-bit values, and coordinate signs. Nine focused
  sample/contract tests pass. The former invalid rough payload caused -15;
  no such modal appeared throughout the corrected 600 s run.
- Empty synchronous serial reads no longer add virtual time after spending
  the timeout in real host sleep. The focused tick/host-time regression
  passes. The broader hybrid clock remains an architectural gap.
- Bare visible-window thread relabeling is now restricted to the actual
  terminal thread stub, preserving worker contexts. The 300 s profile shows
  thread 6 continuing real slices and a successful COM7 ReadFile of 174
  injected NMEA bytes (not merely an empty host queue). It exited 0 at
  300.773 s with empty stderr and no PC-zero/guest-exception marker.
- Inactive/unmapped callback receivers are checked before removing a queued
  message. DOWN/UP FIFO and filtered non-head regressions pass. The subsequent
  330 s actual-input run delivered DOWN then UP through the real MFC wndproc,
  with clean returns: tap response 13:53:05.483680Z, DOWN push 06.771140Z,
  DOWN return 06.793756Z, UP push 06.861065Z, UP return 06.883623Z.
  About 1.29 s to first guest delivery is still a responsiveness issue.
- That run accepted 184 stationary location updates; the final retained file
  ring proves 124 positive COM7 reads totaling 28,536 B. Rings are incomplete
  history, not total injected/consumed bytes. Exit was 0 at 330.221 s, no
  watchdog or stderr, but no map transition. Sampled peak RSS was
  1,362,944,000 B/private 1,377,103,872 B. Boot2 frame timestamps were batch
  stamps, so its ~2.5-4.7 s splash bracket is approximate, not exact timing.
- Read-only ordinary files now use the existing bounded 64 KiB HostFile
  window rather than eager whole-file retention, with no path special cases
  or arbitrary RAM cap. Four new regressions plus converted cache tests cover
  byte/seek/EOF, duplicates, streaming, flush/truncation/reopen and closed-file
  lifetime; 22 file unit tests and all 136 raw memory/file tests pass. Writable
  private-memory promotion is intentionally unchanged.
- The bounded-window executable completed the matched 330 s live run in
  330.259 s, exit 0/no watchdog/empty stderr/no CPU-error markers. Sampled
  peak RSS is 589,615,104 B (562.3 MiB), private 601,038,848 B (573.2 MiB),
  versus the prior 330 s run's RSS 1,362,944,000 B (1,299.8 MiB): about 57%
  lower peak RSS. Phase-aligned samples are also lower. This is a bounded
  observational comparison, not an OS-cache-controlled benchmark or full
  allocation attribution; SD user data and host file cache were not reset.
- New per-request clocks observe black at +2.605 s and splash at +4.837 s
  (2 s polling cadence). COM7 opens at +27.897 s, MFS1 at +30.672 s, and
  actual GPS payload reads continue after sustained input. HTTP accepted 204
  location updates. Tap delivery again preserves DOWN then UP, but response
  to first guest callback is 1.617 s; both guest handlers return in ~22 ms.
  Pixels remain splash, with no map transition at the 330 s limit.
  Logs/result/memory/frame: `target/bringup_1012_bounded_files_drive_*`.
  The four SD zero-byte probe markers remain absent.
- Full bring-up is NOT complete: these runs remain on the genuine splash,
  without a map/interactive navigation transition. The 300 s run first
  opened COM7 at 30.936 s, MFS1 at 34.338 s, and SMB1 at 34.578 s. Its frame
  capture began late (+23.171 s), so that run cannot establish first-splash
  latency. Report: chat-workspace `validation/bringup-1012-boot-profile-evidence.md`.
- That profile sampled host working set up to 1,244,803,072 B (1,187.1 MiB)
  and private bytes up to 1,253,953,536 B (1,195.9 MiB). This is host memory,
  not CE guest heap or virtual reservation, and is not yet proof of a leak.
  Eager whole-file storage caching is an evidenced contributor candidate:
  31 reached read-only files total 263.30 MiB; all 44 mid-size candidates
  total 428.62 MiB, with incomplete access/ownership history. The bounded
  windows now materially lower live RSS as measured above. Remaining guest
  pages/native engine/trace capacity and any writable duplication need
  separate ownership measurements; do not infer a kernel-clone leak.
- Read-only residual-memory audit finds that ordinary process switches move
  CPU/blob owners; full cpu/kernel clones are optional monitor checkpoints,
  not this run. The allocator has a global monotonic heap cursor, and each
  process's `map_heap_spillover` persists pages up to that same shared high
  water mark. In the prior run, persisted blobs grew to roughly 93/81/81 MiB
  for PIDs 1/67/68 while live heap was about 31.6 MB. HeapFree removes live
  metadata but neither reuses addresses nor shrinks spillover backing.
  Measure per-process blob length/capacity and CE heap-reuse semantics before
  reclaiming pages or changing allocator ownership; no speculative fix landed.

Prior reproductions (fixed causes, retained for comparison):

- Normal remote GPS input exposed another control-flow failure in the prior
  validated executable (SHA256 `AF4CA1A7...E687DC2`): the 300 s input run
  exited 1 at 86.90 s, not by watchdog. Location `37.5,127.25` was accepted
  as three NMEA sentences; no touch was sent. Final PC/RA were zero and the
  emulator correctly reported a guest CPU exception. Before it, an outer
  SendMessageW callback was discarded while a nested visible callback still
  ran, then recovered from a historical FP-only record. Fixed by preserving
  the live callback ancestry above; the new actual-GPS run survives.
  Logs/frame/result: `target/bringup_1012_modal_drive_*`.
- A 600 s default-count live run reached a genuine Korean G-sensor error
  dialog (`Error Code: -15`) over the splash. The dialog says navigation can
  continue with GPS and offers OK. `COM7:`, `MFS1:`, and `SMB1:` opened;
  actual serial reads and sensor IOCTLs are present. No input was consumed
  before the run ended. Frame and timestamp/hash snapshots are under the
  chat workspace's `validation/captures/remote-20261012-121948Z`.
- That run stopped executing slices at 12:18:35Z but kept switching cached
  helper contexts and consuming a host core. PID67 was genuinely runnable
  (`can_run=true`, no ready wait/blocked send, PC `0x0001b4a0`). Historical
  `CreateProcessChildParked` trace records were still treated as unreturned
  parent API calls until child exit, reversing scheduler choices. Pre-run
  `continue` paths also bypassed the CLI wall limit. The external watchdog
  stopped PID6268 at 625.78 s, exit -1; this was not a clean guest completion.
  Log: `target/bringup_1012_default_stdout.log` (stderr empty).
- Disproved the old "command-flag readiness loop" lead: the flags
  0x4000/0x2000/0x100/0x80 belong to libpng allocation cleanup, not a
  splash-to-map handshake. `png_free_data` is VA `0x00308298`, image RVA
  `+0x2f8298`; the old `+0x2f7298` was a .text-relative disassembly offset.
  `png_sig_cmp` at VA `0x00307e74` and the PNG signature confirm the region.
- The growing 5/10/15/20 s sleeps are sensor initialization retries, not an
  unidentified helper/network wait. Sleep caller VA `0x000e69b8`, return
  `0x000e69c0`, multiplies object retry count by 5000 after failure of
  `0x000e6754`. The sensor read validation returns -15 after invalid
  rough-offset status; the command/packed-sample correction above closes
  this demonstrated failure. Do not interpret normal 50 ms sensor pacing
  arithmetic as a fresh SensorError merely because V0 was briefly negative.
- Fixed live-pump `CreateProcessW` replay: the child previously executed inside
  the parent import hook before its return registers/PC were committed. A child
  consuming the parent's native timeout left the parent on the completed import.
  Successful live launches now commit the return, stop the parent, and drain
  children after the outer `emu_start` and timeout check. Non-live execution
  retains its synchronous launch path.
- Fresh count=0 / native-timeout=100 ms reproduction dispatched the same helper
  launch at RA `0x00059ba0` 16 times, producing PIDs 67-82. The fixed 120 s run
  dispatched that site once, launched three distinct children (66-68), executed
  735,491,203 guest instructions, and exited normally at the requested limit.
  Logs: `target/parallel_1012_{baseline,fixed}_stdout.log`. The framebuffer is
  still the real splash; this does not establish map/route completion.
  A lower-noise 180 s repeat also exited 0 with only PIDs 1/66/67/68, executed
  1,280,494,534 instructions, and measured 84.56 busy-slice MIPS for iNavi.
  The helper received only one slice; fairness remains open. No explicit device
  open evidence was captured. The four zero-byte probe markers remain absent.
  Repeat artifacts: `target/parallel_1012_repeat_{stdout.log,frame.png}`.
- Corrected scheduler block sampling: prior TB starts were mistaken for prior
  instructions, rejecting ordinary branch targets/fallthrough blocks. The block
  path now checks the word at PC-4 for a possible delay slot. Separate block and
  instruction stop helpers preserve the sparse/global hook guards. A real
  Unicorn count=0 test crosses a page-end branch, executes its delay slot once,
  and stops before the target instruction.
- Explicit global-hook requests and arbitrary-PC diagnostics now override
  legacy sparse flags. Non-trace selector bindings are covered by feature checks.
- Exact source copies in `C:\Temp\wce-1012-parallel` passed 1,511 tests: lib 582,
  bin 28, basic 130, dispatch 10, GWE 495, kernel 125, memory/file 136, waveOut 3,
  host helpers 2. The eVC4 fixture runner is ignored because the MIPS compiler is
  unavailable. `cargo check --release` passes for `unicorn,win32-desktop` and
  `unicorn` without trace. Validation log: `target/parallel_1012_tests.log`.
- At the time, direct repository builds hit Unicorn's Windows symlink privilege
  error. The temporary mirror changed only its local dependency build to copy
  the archive and watch the correct source paths; global registry/system settings
  were not changed. The later in-repo vendor patch and focused D: build are
  recorded above. The 1M-instruction default remains while scheduling fairness
  and native-timeout branch-state preservation are validated. The then-validated
  executable/symbols were installed under `target/release/`; their later
  preservation and exact-path cleanup are recorded above.

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
