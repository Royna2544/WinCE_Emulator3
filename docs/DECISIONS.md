# Decision log

One home for decisions and open questions. A subsystem page keeps a link to its
rows, not its own "Pending" list. Shared contracts still live only in the
[architecture anchor](ARCHITECTURE.md); a decision that changes one must update
the anchor and its consuming tests before implementation. Evidence is in the
[ledger](evidence/ledger.md).

Status values: **decided** (recorded with date and owner), **open** (needs a
decision before the named stage), **deferred** (not gating; revisit at the stage).

## Decided

| ID | Decision | Date | Effect |
| --- | --- | --- | --- |
| D-001 | Evidence lives in one repo ledger ([evidence/ledger.md](evidence/ledger.md)); design pages cite entries instead of repeating narrative. Chat-project validation notes are indexed there and remain in the separate project workspace; they have not yet been imported into this repo. | 2026-10-13 | New evidence is a new ledger entry, not copied prose. |
| D-002 | Run mode is **live-first**: real-time coupled to live I/O by default; never fast-forward past unobserved input. An opt-in **fast-idle** mode (bring-up and tests only) may advance guest time when no task is runnable and no external source is connected. Deterministic replay is a later layer. | 2026-10-13 | Anchor A5 and [clock-timers](subsystems/clock-timers.md) updated; Stage 6 no longer requires replay. |
| D-003 | Stages 3-6 (Task/object authority, continuations, outcomes/scheduler, clock) run on the fresh-engine-per-slice lifetime. Persistent engines become Stage 8, a later swap behind the CPU interface. | 2026-10-13 | The native translation-arena / commit-tail investigation no longer gates the scheduler rewrite. |
| D-004 | ~~The experimental persistent-owner source, Phase 0 changes and docs stay **uncommitted** in the D: working tree for now (GPG signing currently blocks commits).~~ **Superseded 2026-10-13 ~20:45 by user directive:** the working tree is checkpointed as **unsigned backup commits directly on `main`** (signing bypassed per command with `git -c commit.gpgsign=false`; hooks are not bypassed). The commits are grouped by area and are for backup only. They are not reviewed releases, and intermediate commits may not build on their own. The supervisor repeats this each 10-minute pass. Agents must not `reset`, `rebase` or rewrite these commits. | 2026-10-13 | A later 306-file source-matching archive in the ChatGPT project workspace (`validation/archives/task-pin-source-20261013.zip`, SHA-256 `1F3D5540…0E75`) now provides a recoverable copy of the experimental Rust/config/test source. The dirty D: tree still must not be reset; revisit a proper commit when signing works. |
| D-005 | Add a parallel, read-only app evidence lane ([app-evidence-lane.md](app-evidence-lane.md)); it never gates or forces architecture work. | 2026-10-13 | Plan "Parallel lanes". |
| D-006 | Docs style: spacing pass plus a status-at-a-glance table in [README](README.md); this file is the decision log. A fixed per-page template was not chosen. | 2026-10-13 | Prose otherwise unchanged. |
| D-007 | **One source directory, one disposable build directory** (user directive). All source work, including tests, expected-red controls and fixes, happens in this repo (`D:\GitHub\Wince_emulation_v3`). No copy mirrors, no `build-1`/`build-2`, no per-experiment target dirs. Exactly one Cargo target dir, which can be deleted at any time: the repo `target/`, or one fixed short `CARGO_TARGET_DIR` if MSVC C1041 forces it, recorded in AGENTS.md. Run evidence (frames, logs, jsonl, receipts) lives outside the build dir. Build-environment fixes, such as the `unicorn-engine-sys` CMake symlink step, land in the repo (for example a vendored `[patch.crates-io]` path dependency), not in a mirror. | 2026-10-13 | Supersedes the copy-mirror workflow in AGENTS.md and [artifact lifecycle](subsystems/artifact-lifecycle.md). Unique source in existing `C:\Temp\wce-*` mirrors is ported into the repo, then each mirror is retired with a receipt (ask the user before deleting). O-P4 is resolved in the repo source, not in a Temp tree. |
| D-008 | **Stage 2 O-M1 backing target:** each migrated committed guest page has one address-stable byte authority, pinned while mapped into a native owner with `mem_map_ptr`. An `AddressSpace` owns its private slabs and view records; a canonical VM/file-mapping object owns shared slabs referenced by each view. Private pages migrate first; shared views and image/heap families move separately without two writable authorities for one backing. A new canonical process-birth directory, allocation identities, exact native map pins, checked failures and independent-clone semantics precede *public Stage 2 acceptance*. The isolated 1/1 native slab diagnostic supports feasibility only; E-18 remains red, so this chooses a design, not an accepted implementation or a memory/boot-speed result. **Incremental scope:** a narrow single-address-space private slab change may turn internal E-18 green using a VM owner/allocation incarnation and explicit clone-unsupported failure; that remains a diagnostic milestone. Public two-process VA/Ex ownership, clone isolation and fresh-default production still require the canonical birth directory and full checked slice boundary, which Stage 3 extends for Tasks. | 2026-10-13 | [Anchor A4](ARCHITECTURE.md), [memory design/gates](subsystems/guest-memory.md) and [Stage 2 plan](PLAN.md) use this target. Unmigrated copy-based families retain explicit checked synchronization until migrated. O-M1 and Stage 2 remain open. |
| D-009 | The E-33 Windows TCG commit-tail arithmetic repair is a narrow **early safety exception** to the Stage 8 native-arena lane. After the E-32 host code-emission crash, a pristine one-MiB real-MIPS control reproduced `0xC0000005`; the vendored clamp made 1/2/8-MiB/default native controls green and passed broad suites. The user delegated Claude's steer to keep this one fix and its control, then stop the arena investigation. | 2026-10-14 | No arena-size change, crash-cause attribution, footprint acceptance, persistent-owner acceptance, or additional writable-mount app run follows. All other native arena/lifetime work stays deferred to Stage 8. See [E-33](evidence/ledger.md#e-33-windows-tcg-commit-tail-repair-2026-10-14-executed). |

Current status (2026-10-14): D-008's E-18-red statement records the decision-time
observation. E25 turned that control green only under an internal, test-only
private-slab opt-in with a fresh native owner
([broad validation receipt](../evidence/e25-private-slab-20261014/broad-validation.receipt.md)).
E35 then validated the public COREDLL private `VirtualAlloc` slab family
([validation receipt](../evidence/e35-public-virtualalloc-validation-20261014/validation.receipt.md)).
The O-P4 source-bound fresh-default owner and teardown code/test gate has now
passed ([receipt](../evidence/op4-fresh-default-20261014/validation.receipt.md));
this is not a generic checked-close API or a reopening of D-009's deferred
persistent arena lane. O-M1 shared views, real native
two-process same-VA bytes, bounded live validation and full Stage 2 acceptance
remain open.

## Open questions by owner page

### Tasks and scheduler ([page](subsystems/tasks-scheduler.md)) - needed before Stage 3-5

| ID | Question | Status |
| --- | --- | --- |
| O-T1 | Exact TaskDirectory API and compatibility for legacy fixtures with unknown creation owner (never fabricate a PID); durable child ProcessIdentity and primary TaskKey must survive launch-queue drain and public-handle closure through canonical completion. | open |
| O-T2 | Checked process/task incarnation and public PID/TID/handle-number allocation/exhaustion policy; failure must not publish an alias, task, launch or runnable frame. | open |
| O-T3 | Representation of native-frame payloads. | open |
| O-T4 | Scope of termination-in-progress and pending-suspend parity with CE6. | open |
| O-T5 | Priority-inheritance/ceiling coverage (shared with O-K6). | open |
| O-T6 | OEM default quantum configuration; extract from dumped kernel/OAL evidence, not a guessed 10 ms or the shim's 100 ms. | open |

### Clock and timers ([page](subsystems/clock-timers.md)) - needed before Stage 6

| ID | Question | Status |
| --- | --- | --- |
| O-C1 | Instruction / cycle / native-elapsed to GuestInstant conversion (no guessed MHz). | open |
| O-C2 | Checkpoint host-anchor rebasing. | open |
| O-C3 | Guest calendar epoch for system-time APIs. | open |
| O-C4 | Timer minimum, overrun and coalescing rules (needs dumped/source evidence for GWES SetTimer). | open |
| O-C5 | External-event timestamp assignment (arrival order vs recorded timestamps). | open |
| O-C6 | Acceptable host pacing bounds for live mode and the fast-idle opt-in conditions. | open |

### Kernel objects ([page](subsystems/kernel-objects.md)) - needed before Stage 3

| ID | Question | Status |
| --- | --- | --- |
| O-K1 | Per-process public handle tables vs compatibility global numbering. | open |
| O-K2 | Canonical object-reference representation. | open |
| O-K3 | Execution, public-alias and registered-wait pin/retirement bookkeeping **required in the first Stage 3 cut**, including exact release on cancellation and failed preparation. | open |
| O-K4 | Source-close duplication failure behavior: CE6 `handle.c` closes the source when its process is valid even if duplication/output fails; freeze null/bad-output, failed-duplicate and invalid-source-process controls before adopting the emulator policy. | open |
| O-K5 | Process termination's worker sweep and canonical child completion after the original process/thread public handles close. | open |
| O-K6 | Priority inheritance (shared with O-T5). | open |

### GWES ([page](subsystems/gwe-messaging.md))

| ID | Question | Status |
| --- | --- | --- |
| O-G1 | Canonical window-handle generation and lifetime. | open |
| O-G2 | Custom pointer-message marshalling policy for application-defined LPARAMs. | open |
| O-G3 | Which complete continuation family migrates first (ordinary GetMessage/send or window creation). | open |

### Devices and sensors ([page](subsystems/devices-sensors.md))

| ID | Question | Status |
| --- | --- | --- |
| O-D1 | Per-device vs per-open state placement. | open |
| O-D2 | Alias semantics for device handles. | open |
| O-D3 | RX/TX capacity and backpressure. | open |
| O-D4 | Host error propagation (partial write, serial error). | open |
| O-D5 | Operation preparation/commit for reads and IOCTLs. | open |
| O-D6 | Timeout conversion (shares O-C1). | open |
| O-D7 | Checkpoint treatment of live host endpoints. | open |
| O-D8 | Physical sensor timing scope (ideal model today). | open |

### Storage ([page](subsystems/storage.md))

| ID | Question | Status |
| --- | --- | --- |
| O-S1 | Open-context alias and cursor contract. | open |
| O-S2 | Writable buffer ownership and budget. | open |
| O-S3 | Host snapshot / rebind semantics for checkpoints. | open |
| O-S4 | Share, delete and lock behavior. | open |
| O-S5 | Mount-removal pins. | open |
| O-S6 | Error mapping parity. | open |
| O-S7 | Bounded asynchronous completion. | open |
| O-S8 | Symlink / junction containment policy. | open |

### Registry ([page](subsystems/registry.md))

| ID | Question | Status |
| --- | --- | --- |
| O-R1 | CE6 key, access and security semantics (locate the CE6 implementation and dumped coredll forwarding first). | open |
| O-R2 | Per-process handle visibility. | open |
| O-R3 | Deletion with open aliases. | open |
| O-R4 | Change notifications. | open |
| O-R5 | Hive persistence (commit point, flush semantics). | open |
| O-R6 | Decoder code pages. | open |
| O-R7 | Enumeration stability under mutation. | open |
| O-R8 | Storage/driver configuration publication. | open |

### Memory, CPU and cross-cutting

| ID | Question | Page | Status |
| --- | --- | --- | --- |
| O-M1 | D-008 selects AddressSpace-owned committed-page slabs as the backing design. E-16 turns the formerly red injected same-VA native file-view control green, but public same-VA `MapViewOfFile` remains open. E25 turned the formerly ignored E-18 private stale-byte control green under a test-only slab opt-in and fresh native owner ([corrected-source receipt](../evidence/e25-private-slab-20261014/broad-validation.receipt.md)); E35 validated the public private `VirtualAlloc` family and two-process same-VA metadata ([E35 receipt](../evidence/e35-public-virtualalloc-validation-20261014/validation.receipt.md)). O-P4's pinned-source fresh-default owner/teardown code gate also passed ([receipt](../evidence/op4-fresh-default-20261014/validation.receipt.md)), without a general checked-close API or live acceptance. Real native two-process A→B→A byte isolation, shared-view backing and full machine checkpoint relineage remain unproved. Less copying and lower memory remain unmeasured hypotheses. See [memory: O-M1 backing decision](subsystems/guest-memory.md). | [memory](subsystems/guest-memory.md) | E35 private-family and O-P4 code gates closed; broader O-M1/Stage 2 open |
| O-M2 | Authoritative reservation extent and allocation incarnation before any heap reuse. | [memory](subsystems/guest-memory.md) | open |
| O-P1 | Explicit stop classification (count / yield / deadline / error / debugger). E-34 preserves a checked JR continuation when a typed peer yield and native deadline coincide. Report sealing and failure-path controls remain separate hardening. | [CPU](subsystems/cpu-engine.md) | classification closed (E-34); hardening open (Stage 2/5) |
| O-P2 | Stable hook rebinding for persistent engines. | [CPU](subsystems/cpu-engine.md) | deferred (Stage 8) |
| O-P3 | Native translation-arena budget and commit-tail handler proof. | [CPU](subsystems/cpu-engine.md) | deferred (Stage 8) |
| O-X1 | Real-coredll / CE6 syscall boundary go/no-go (separate from the scheduler core). | [API boundary](subsystems/api-boundary.md) | deferred |
| O-P4 | **Where the fresh-engine lifetime lives.** `ProcessNativeEngine::new` now defaults to a new native owner for each slice; `WINCE_EMU_PERSISTENT_NATIVE_ENGINE=1` explicitly opts into the Stage 8 experiment, with the policy preserved through clone/park. Fresh finish/abort/drop closes the owning wrapper before callback boxes and pointer pins are released, then discards its native map ledger and engine-specific blank Context; successful finish retains only the exact-key CPU-only snapshot. Individual hooks are not removed first on this fresh path. The proof is specific to the pinned, statically linked vendored Unicorn 2.1.5 `uc_close`, which has only `UC_ERR_OK` exits; it does **not** add a generic checked-close API. The formerly ignored closed-owner Context/store control, 200-slice pointer-map/pin churn, finish-failure and typed-stop controls pass on frozen source; full release passed 1,730/0/14 ignored and no-feature 1,292/0/13 ignored ([receipt](../evidence/op4-fresh-default-20261014/validation.receipt.md)). No bounded app run or footprint comparison followed. Native A→B→A same-VA bytes, shared-view coherence and full Stage 2 remain separate gates. A writable-mount live run needs new human approval and fresh adjacent ZIP backups. | [CPU](subsystems/cpu-engine.md) | source-bound code/test gate passed; live/Stage 2 open |
| O-X2 | Preservation while commits are blocked (D-004). Until GPG signing works, take a non-destructive backup of the working tree whenever its state changes meaningfully: `git diff --binary HEAD` plus an archive of `git ls-files --others --exclude-standard`. Store it outside `target/` with a hash receipt. This never runs `stash`, `checkout` or `reset`. | [artifact lifecycle](subsystems/artifact-lifecycle.md) | open |
