# Build, evidence and intermediate artifact lifecycle

Common anchor: [ARCHITECTURE.md](../ARCHITECTURE.md), A6.
Implementation order: [PLAN.md](../PLAN.md). This is a host workflow boundary,
not guest memory management or authorization to discard user data.

## Ownership and classification

Source/dirty work, synced project references and user SD data are protected.
Build caches are generated but may be in use by another editor/compiler.
Installed/fallback/failed executables, matching symbols and unique run evidence
are reproduction artifacts, not disposable merely because they are old.
Diagnostic logs, capture manifests and staged tests name exact source/binary
hashes, clocks, configuration, outcomes and completion/missing-artifact state.
Record image RSDS GUID+age and verify the PDB actually loaded during
symbolization; a matching filename or hash recorded separately is not enough
to prevent an adjacent generic-name PDB from mislabeling a crash address.

Target layout ([D-007](../DECISIONS.md)): one source dir (this repo) and one
Cargo target dir intended to be disposable after retained artifacts are
classified. Installed, fallback and failed binaries with their PDBs, plus run
evidence, should live outside the target dir so its eventual deletion loses
nothing. At the 2026-10-13 inventory, repo `target/` (about 16 GB) mixed
build output with run evidence (`*.ppm`, `*_live.log`, `bringup_*` jsonl). The 243
immediate top-level evidence files were copied and SHA-256 verified under
`evidence/legacy-target-root-20261013/` (1,207,435,200 B); 28 immediate
`target/release` EXE/PDB files were likewise copied and verified under
`evidence/legacy-target-release-20261013/` (2,903,482,880 B). Each directory
contains a manifest and receipt. After explicit user approval, those 271
original files were rehashed against their preserved copies and removed by
exact, nonrecursive paths. Nested build/evidence directories are still
unclassified, so the whole target is
**not yet certified disposable**. The local `evidence/` directory is ignored
by Git; its portable summary belongs in the repository ledger.

Historical copy-based mirrors under `C:/Temp/wce-*` included the four
`wce-1013-stage2-*` paths retired below. Other candidates, including
`wce-1012-parallel` with target `t`, `cross-*`, `tail-*` and `veh-*`, still
require their own source/evidence inventory before retirement. The validated
Unicorn 2.1.5 dependency was copied into
`vendor/unicorn-engine-sys` and matched 567/567 file hashes against that mirror;
`Cargo.toml` now selects it. The later post-order serial full release suite
passed 1,623 tests (0 failed, 3 ignored) in the D: target on dirty persistent-
owner source; other feature profiles and the live run remain pending. Retire
each further mirror only with a receipt and separate approval. Cargo hardlinks
connect installed artifacts to cache entries;
summed file lengths are not physical reclaim.

The five inventoried legacy paths have a verified unique-source recovery ZIP
and receipt under `evidence/legacy-mirror-unique-source-20261013.*`: 26 selected
source/config/test/document files plus a manifest, 680,869 ZIP bytes, SHA-256
`45EAF380…80BFF33`. Every archived stream matched its original hash and length.
This is not a full copy of all build targets or proof that every historical
binary is preserved. After separate explicit approval, the five exact
inventoried paths (`C:/Temp/wce-1013-stage2-fresh`, its `-target`,
`C:/Temp/wce-1013-stage2-churn`, `C:/Temp/wce-1013-stage2-t`, and the
`scheduler-core-fresh` Git worktree) were retired. The worktree branch was
preserved; all five paths were checked absent afterward. The exact 129-line
fresh-HEAD red-test patch and selected fresh-HEAD/Stage-8 EXE/PDB pairs were
also preserved under D: `evidence/` and their hashes reverified after removal.
Unselected compiler outputs and binaries are not recoverable from these
archives, though rebuildable where source was retained. See local
`evidence/stage2-e7-stop-20261013/post-order-and-retirement.receipt.md`.
No other `C:/Temp` path, D: source/target/evidence or SD data was removed by
that operation; other mirror retirement still needs its own scope and approval.

## Scoped interfaces and safety

An `ArtifactManifest` records exact absolute target, owner, category, content
hash/length, retained twin or recovery archive and reason for disposal.
Cleanup only consumes an explicit reviewed whitelist. Verify resolved paths
remain in the intended artifact directory; refuse junction/symlink traversal
and broad roots. Revalidate content/active use immediately before mutation.
Do not enumerate a whole drive or pass computed delete commands between shells.

Preserve installed BED2 plus `pre_scheduler_core_1013` fallback, failed 650
plus matching PDB, its source/native dependency and actual crash evidence.
Unique earlier reproductions require verified recoverable archival first.
Never equate a same-volume move with reclaimed disk space or a hardlink entry
removal with released data. Cleanup failure leaves evidence/recovery intact.

## Current cleanup evidence

Five exact disposable files were archived and hash-verified on 2026-10-13:
two obsolete standalone test executable/PDB pairs and one byte-identical
cache-entry trace whose final twin remains. Source/harnesses were retained.
The original artifact files totalled 9,522,161 logical bytes; the verified ZIP
is 2,794,447 bytes. No source file was removed.
This is not a measured physical-disk saving. All five are recoverable from
the chat validation archive and its receipt. Active caches and unique evidence
were not removed. No synced source or SD file was modified by cleanup.

The later Stage 3 Task-pin red-test mirror was separately archived to the
chat project's `validation/archives/task-pin-source-20261013.zip`: all 306
source-file streams were SHA-256 verified against the original. Only its
exact idle `C:/Temp/wce-taskpin-1013-a7` source copy and
`C:/Temp/wce-taskpin-t-a7` disposable Cargo target were then removed.
The ZIP is recoverable; compiled output requires rebuilding. See ledger E-11
and the chat `20261013-task-pin-expected-red-receipt.md`.

## Plan and verification gates

Before a larger cleanup: prove ownership/idle state, inspect exact cache
subtree, separate unique evidence and preserve rebuild inputs. For archives,
verify every extracted stream's original hash/extent before removing originals;
write a receipt even for partial cleanup. Validate installed/fallback/failure
hashes afterwards. Rotate bounded traces/captures only under documented retention
rules; terminal snapshots cannot replace all earlier execution observations.
Artifact work never overrides the architecture/runtime acceptance gates.
