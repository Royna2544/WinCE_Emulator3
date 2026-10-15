# Mounted live-run control

Status: host-workflow design and source-only safety gate, with one **aborted
prelaunch backup** recorded below. It did not produce a complete three-mount
backup or launch the runner/guest. The pre-watchdog scripts had a
[22-file a02 seal](../../evidence/stage3-live-evidence-20261015/prelaunch-repair-a02-source-seal.json)
and [script-only build bridge](../../evidence/stage3-live-evidence-20261015/prelaunch-repair-a02-build-bridge.json);
the earlier [21-file preparation gate](../../evidence/stage3-live-evidence-20261015/final-source-build-seal-receipt.md)
is historical, not the current script identity. A01, a02 and intermediate a03
are superseded and not live-usable. The current [a04 source seal](../../evidence/stage3-live-evidence-20261015/prelaunch-repair-a04-source-seal.json),
[full patch](../../evidence/stage3-live-evidence-20261015/prelaunch-repair-a04-full-scoped.patch),
[script-only bridge](../../evidence/stage3-live-evidence-20261015/prelaunch-repair-a04-build-bridge.json)
and [offline gate](../../evidence/stage3-live-evidence-20261015/prelaunch-repair-a04-offline-gate.raw.txt)
bind the independent watchdog candidate to the unchanged clean-built EXE/PDB.
Claude's read-only and peer reviews found no concrete source blocker, but are
not a live dead-driver integration test or run approval. Neither gate
establishes full CE6 bring-up.

The [common anchor](../ARCHITECTURE.md) A1, A5 and A6 owns guest authority,
time and evidence semantics; the [migration plan](../PLAN.md) owns runtime
acceptance. This host control layer owns *permission-to-attempt*, preserved
inputs, process containment and observations. It cannot select a guest task,
advance guest time, force a window callback or declare a splash to be a map.
It is deliberately separate from `CeMachine` and from the guest's
`RunOutcome`.

## Boundary and identities

One proposed attempt has one unique run tag and one manifest. Its identity is
the manifest's SHA-256 together with the exact source seal, build receipt,
preserved EXE/PDB pair, configuration/app hashes, three new backup ZIP hashes,
approval citation and run caps. The [pure preflight core](../../scripts/inavi_run_preflight_core.ps1)
validates this structure; the [runner](../../scripts/inavi_bounded_e22.ps1)
re-reads and rehashes it under the launch monitor's lock. In the historical
a02 source-only gate, its seal enumerated 22 required source files and a
full W-B1c-scoped patch; the preserved release EXE and PDB share an RSDS
GUID+age and are outside disposable `target/`. The bridge binds the repaired
scripts to that unchanged [clean-built binary](../../evidence/stage3-live-evidence-20261015/telemetry-broad-a03-clean-build-receipt.md);
it is **not** a new Rust build. The a04 bridge supersedes a02's script
identity; the old 21-file seal remains historical.
The test-pass flags and patch identity in the build receipt are reviewed
attestations tied to preserved receipts, **not** cryptographic proof that the
runner executed those tests. An old EXE, a same-named PDB, a Git HEAD alone,
or a stale backup manifest is not an interchangeable input.

The run manifest also binds the exact registry, device configuration,
`mounts.toml`, and target `iNavi.exe` bytes. The monitor constructs the
emulator arguments itself: virtual desktop, mounted image and dumped runtime
DLL directory, 330-second guest run cap, one-million-instruction slice limit,
and the repository-mandated `192.168.0.39:8765` remote endpoint. The driver
rejects `WINCE_EMU_*` experiment variables for this matched profile. This
fixed command is a comparison protocol, not a claim that CE quantum or boot
timing is faithfully implemented.

## Human approval is outside the hash chain

Before *each* writable-mount launch, a human must review the exact proposed
scope and give a fresh, distinct one-run approval. Record its exact chat
question/message citation and observed UTC in a separate human-review
receipt, then place that citation and timestamp in the run manifest. The
operator must verify that the approval covers writes to the three original
mounts, the current binary/configuration, the intended GPS/tap drive and the
one bounded run. Where supervisory cross-check is requested, retain that
check as another review receipt before invoking the runner. No approval in
earlier run history or in this design page authorizes a new launch.

Approval `call_9b35e9d31fdf402ea082a3eb6a40ae16` was used to start a
prelaunch backup, then withdrawn/void after its script failed. It cannot
authorize another backup or launch. The [disposition](../../evidence/pre-live-mount-backup-20261015T025500Z/aborted-approval-disposition.md)
and `withdrawn` ledger row preserve the reason and one completed SD ZIP.
A **new** distinct approval is required before any further backup or launch.

The script checks manifest/caller citation equality, timestamp syntax,
backup-start time after the reported approval observation and ledger
non-reuse. It cannot authenticate the conversation, determine the message's
true author or prove that an operator did not copy an old citation with a new
timestamp. The current manifest has no mandatory `supervisorCheck` field; any
such authenticity check remains human evidence, not a property of the JSON
validator ([final offline gate](../../evidence/stage3-live-evidence-20261015/final-offline-safety-gate-receipt.md)).

## Recoverable writable-mount boundary

The [backup script](../../scripts/backup_rw_mounts.ps1) names exactly SD,
ResidentFlash and EmulatorRoot. It creates a uniquely tagged ZIP *beside*
each original folder without changing the original or an earlier archive.
It publishes an incomplete manifest first, inventories and hashes source
files, writes each `.partial` ZIP, verifies every archive entry against its
source, then publishes the renamed ZIP and its hash. Because the three ZIPs
are sequential rather than one atomic snapshot, it inventories and hashes
all three source trees again after all archives exist. Only then may its
schema-2 manifest say `complete=true`, with each row's post-backup inventory
and `postBackupInventoryVerified=true`. Failure leaves an incomplete manifest
and only its own unfinished partial is eligible for cleanup. These are source
contracts and offline fixture results, not proof of a **completed** real
three-mount backup ([freshness receipt](../../evidence/stage3-live-evidence-20261015/backup-manifest-offline-a02-freshness-receipt.md)).

One real attempt began under the now-withdrawn approval. The old backup
helper's existing-file `File.Replace` call failed while updating its manifest
after verifying the adjacent SD ZIP. Its recovery update failed the same way;
the manifest remains `complete=false`. The verified SD archive is
5,973,898,396 bytes, SHA-256
`6032F822551CFB5F28ED079CB5B41CB02B1585F8AFF16EA303B54A74B0C2C949`.
No ResidentFlash or EmulatorRoot ZIP followed, and this incomplete triplet
cannot satisfy preflight or be reused. The SD ZIP remains adjacent pending
the user's disposal choice; no runner or guest launch occurred
([failure](../../evidence/pre-live-mount-backup-20261015T025500Z/failure.json),
[disposition](../../evidence/pre-live-mount-backup-20261015T025500Z/aborted-approval-disposition.md)).
Both JSON overwrite helpers now pass offline creation, replacement and
locked-target failure/cleanup controls; this is not a second live backup
([green receipt](../../evidence/stage3-live-evidence-20261015/atomic-json-overwrite-green-receipt.md)).

Preflight requires the completed backup script's hash to match the sealed
source, the backup tag to match its `startedUtc` second, `startedUtc` to be
after the human-recorded approval time, all three adjacent archive names and
hashes to match the manifest, and the three mount roots to match the selected
configuration. Immediately before reservation, the detached monitor checks
for reparse points and compares **current file content hashes**, types,
counts, sizes, timestamps and attributes with the sealed post-backup source
inventories. It records scan bytes, file count and elapsed time. The
same-size/same-mtime replacement negative passed offline; actual scan cost is
unmeasured ([runner a06 receipt](../../evidence/stage3-live-evidence-20261015/runner-offline-a06-mount-content-receipt.md)).
Keep all three originals quiescent from backup through launch. A write after
the final scan, or a write during the sequential ZIP cycle that evades the
final comparison, remains an external time-of-check/time-of-use risk. The ZIPs
are recovery copies, not a promise to roll back guest writes automatically.

## One-use ledger and launch transaction

The local [use ledger](../../evidence/live-run-use.jsonl) and byte-identical
tracked [use ledger](../evidence/live-run-use.jsonl) are append-only deny
history. The monitor takes an exclusive local lock and verifies both histories
before rehashing, appending a `reserved` event, calling `Start-Process`, and
appending a `spent` event with the exact PID/start identity. It does not
release the lock between these steps. A partial dual append, changed ledger,
duplicate run tag, used approval or reused ZIP denies the next run; it is not
permission to retry launch. A successful `spent` must close its own matching
reservation. An unresolved reservation denies all future attempts until a
separately reviewed, hash-preserved **confirmed-no-spawn** investigation is
recorded as `reconciled`. Absence of `process.json` alone is not that proof,
and reconciliation does not make the old approval or ZIPs fresh again.

The six earlier writable-mount launches were backfilled honestly:
`legacy-deny` bars E22/E27's old shared ZIP triplet without inventing their
unknown approval IDs, and `historical-spent` bars the four later runs with
direct or explicitly inferred citation provenance. E32's run manifest
repeated E29's ZIP hashes, so its own fresh triplet is supported by the
separate backup manifest/analysis receipt; both triplets remain denied
([seed receipt](../../evidence/stage3-live-evidence-20261015/historical-ledger-seed-a01-receipt.md)).
The ledger proves recorded uniqueness and byte integrity at validation time,
not the historical processes' authenticity or approval authorship.

A seventh `withdrawn` terminal row denies this aborted approval, attempt ID
and its one verified SD ZIP hash without inventing a run manifest, other ZIPs
or process fields. Its reason, UTC and evidence path/hash are required; the
runner independently rehashes that evidence before accepting ledger history.
The local and tracked ledgers are byte-identical at SHA-256
`53BAD1ABCAEA7F723737014E182228922122951F1CFB0574773E49EE3DFD1A08`.
This is a **deny**, not a `reserved`/`spent` launch, and does not substitute
for a new human approval. Duplicate or conflicting use of its approval,
attempt or listed ZIP remains invalid or denied. The a02 offline suite added
three expected-red-then-green cross-row ZIP conflicts: a withdrawn ZIP cannot
also appear in an earlier/later historical row or another withdrawal
([receipt](../../evidence/stage3-live-evidence-20261015/withdrawn-zip-conflict-red-green-receipt.md)).

The runner candidate now starts a separate `WatchdogWorker` before the launch
request; the driver verifies its PID/start identity and fresh heartbeat first.
The monitor publishes a post-mount-scan marker. From that monotonic marker the
watchdog enforces a 355-second deadline independently of a stalled
post-`Start-Process` ledger or descriptor write, and checks exact-app RSS and
combined raw stdout/stderr size every two seconds against 2 GiB and 512 MiB
caps. It uses a verified spawned-process record or unique framebuffer-output
argument plus executable path and start window for exact cleanup, including
monitor cleanup; no process-name-wide kill is allowed. Worker exceptions
attempt exact cleanup and record failure. The detached monitor also retains
its 390-second/2-GiB/512-MiB guards, and the driver checks both workers'
heartbeats and exact identities. Once `Start-Process` returns, the monitor
holds the process object; a throwing post-spawn identity/spend/descriptor
commit closes it before propagating the error. It writes a pre-spend
`spawned-process.json` and then an active `process.json` after the spent
append. Before calling `Start-Process`, the monitor atomically publishes
`launch-began.json` after reservation. With the exact monitor stopped, absence
of the launch marker, process descriptors and spent row supports a no-spawn
proof;
an empty WMI search **after** the marker cannot prove app absence. In that
unresolved case the watchdog stays active and repeats exact sweeps under the
same approval instead of recording success. The driver's no-descriptor
finalizer attempts exact pre-descriptor cleanup even if the monitor is still
alive, and retains the independent watchdog when cleanup cannot be confirmed.
The earlier driver/monitor-only post-spawn gap is addressed in source and
offline controls, not yet in a live dead-driver integration run. Independent
containment assumes the D: evidence/mount volume makes I/O progress; while
the whole volume is stalled, guest writes to these original mounts stall too.
Simultaneous loss of all containment workers remains a residual risk.

## Observation and terminal evidence

The GET-only observer must be ready before launch. It waits for the exact
process descriptor without consuming capture slots, then measures a
420-second *launch-relative* horizon. A descriptor or first frame obtained
outside the first two seconds marks coverage incomplete but capture continues.
On two-second slots it requests frame first, then status and a bounded
64-KiB debug summary; it also samples file/remote-input traces, and less
frequently window/message/active views. Every request carries host receive
time, status/error and body hash; repeated debug-publication sequence is
labelled stale even when a fresh GET succeeds. Changed frame bodies are
retained within a bound. The summary's publisher stamp is server-local, and
status, frame and debug caches are still published separately: none provides
the common anchor's future atomic `SnapshotEpoch`
([observer receipt](../../evidence/stage3-live-evidence-20261015/observer-summary-validation.receipt.md),
[remote design](remote-observability.md)).

Input scheduling is relative to an observed running/`COM7:` status, not app
start: the present diagnostic plans 16 one-Hz raw RMC/GGA/VTG bursts from
input-ready +93 s and a tap at +153 s. POST ACK proves acceptance only.
Separate evidence must show guest `COM7:` bytes, input drain/post, genuine
WNDPROC entry/return, frame transition and navigation effect. The terminal
PPM, raw logs, monitor result, before/after mount inventories, observer
result and final `result.json` count only if actually produced and hashed;
filenames requested on the command line are not terminal artifacts.

## Fail-closed interpretation

| Phase or result | Classification and next action |
| --- | --- |
| Approval, seal, RSDS, config, backup, current mount or ledger check fails before reservation | No launch authority. Preserve the reason and repair inputs; obtain a new approval/backup where needed. |
| Prelaunch backup aborts and approval is withdrawn | Retain the failure, incomplete manifest and any verified ZIP as evidence; record a deny-only `withdrawn` row. Do not complete or reuse that triplet. Obtain a new approval before another backup. |
| Reservation exists but launch is not proved | Deny retry. Investigate exact monitor/process evidence and require independent confirmed-no-spawn reconciliation; do not infer it from a missing descriptor. |
| Spawn or post-spawn spend/descriptor commit fails | Attempt failure; close the held process and preserve partial ledger plus cleanup evidence. Treat uncertain termination as unresolved. |
| Either watchdog, RSS/log cap, driver/worker failure, guest CPU exception, unexplained PC zero or nonzero/early exit | Failed or incomplete workload, never a clean CE exit or map-readiness result. Preserve crash/native and containment evidence. |
| First-frame/horizon/GET or terminal-artifact coverage is incomplete | Observation failure even if the app exits zero; report the missing interval or artifact, not an inferred frame. |
| Runner exits zero with complete evidence | Only a bounded harness result. Inspect actual frame, input-consumption, native-outcome and phase-matched memory/lag evidence before making any boot or bring-up claim. |

The runner currently reduces several of these states to `earlyFailure` and a
nonzero harness exit. A future run receipt should retain the more specific
phase/reason, exact timestamps, process termination confirmation, source and
binary hashes, host RSS/private peaks, guest stable-page-budget use and
busy-slice MIPS/setup cost. The guest stable-page scalar excludes boot reserve
and unmigrated memory families; it is not total guest commit or host RSS.
Likewise a genuine splash, GPS ACK or tap ACK is not CE6 full boot, map
presentation, parsed position or interactive navigation
([verification](verification.md), [app evidence](../app-evidence-lane.md)).

This page defines the boundary to review before any live attempt. It does not
waive fresh one-run approval, the adjacent recovery backups, the mandated LAN
endpoint's exposure review, or the remaining scheduler/VM/clock migration
gates in the common plan.
