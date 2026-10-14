# Storage and mounted files

Status: subsystem design for review, not a new filesystem implementation. The [common anchor](../ARCHITECTURE.md) supplies CeMachine, ObjectIdentity, AddressSpace, FrameIdentity, time/event and SnapshotEpoch contracts. Follow [PLAN](../PLAN.md); coordinate file views with [guest memory](guest-memory.md), handle lifetime with [kernel objects](kernel-objects.md), and stream devices with [devices/sensors](devices-sensors.md).

## Current boundary

[HostFileSystem](../../src/ce/file.rs) translates configured guest roots to host paths and owns mounts, OpenFile/OpenFind records, cursors, backing, range locks and I/O statistics. [mounts.toml](../../mounts.toml) configures the object store and mounted roots. Kernel/coredll resolve file versus device handles and marshal guest buffers; a device session is not a mounted-file backing.

Read-only existing files use host handles with a bounded 64 KiB read window; larger transfers stream through bounded chunks rather than retain the whole file. Zero-length reads preserve preceding EOF state. Writable existing files in the current 1–128MiB range may use private full-file Vec buffers; other writable files use host handles. Private buffers become host-visible at flush/close, which invalidates read windows. This is current behavior, not a universal CE cache/coherence contract.

duplicate_open_file clones its OpenFile state/backing; private buffers and read windows can be copied and host handles use try_clone. Current logical cursors/alias visibility require a separate CE open-context audit. Public file mappings now use canonical VM-owned shared pages with birth-qualified per-process views in the focused D008 path; older DIB/injected views remain copy-backed until their object makes a one-way transition. Neither mapping backing is the same authority as every private OpenFile buffer. Ordinary process parking moves CPU owners; optional checkpoints can clone filesystem state/resources.

The first public view after all DIB views are deleted migrates the entire
legacy mapping byte store, including pages outside a tiny first view, into
the same charged physical-page budget as private slabs. Insufficient budget
leaves the legacy object unchanged; live DIB/public mixing and a new DIB on
an already-migrated object fail instead of creating two writable authorities.
File-backed pages are seeded from the host file when first needed, with any
preexisting DIB bytes overlaid before the old store is discarded. Focused
controls pass; broad D008 validation and live-app behavior are separate gates.

Known transaction limit: file read_into commits its logical cursor only after the guest sink succeeds, but a streamed failure can leave an already-written guest prefix. ReadFile's transferred-count write can fail after bytes/cursor effects. No complete byte rollback is implemented. Host flush errors prevent successful close; current broad raw error mapping does not prove precise CE error parity.

## Target authority, interfaces and lifetime

CeMachine sequences guest-visible effects; storage owns canonical file/open-context/mount backing and host-operation state. AddressSpace owns guest output and mapped views. Host adapters cannot select tasks, directly advance guest time or publish completion outside the machine event boundary.

Proposed logical operations, not finalized Rust signatures:

- Resolve a normalized guest path under a configured mount, enforce mount/access/creation rules and prepare an open candidate before publishing its handle.
- Resolve file aliases through canonical ObjectIdentity; pin open context and mounted volume for each operation. Closing one public alias does not invalidate a pinned operation or mapped backing.
- Read/seek/write through a declared open-context cursor/coherence contract; retain bounded staging/read windows and report actual transferred bytes/errors.
- Validate AddressSpace/frame, full output extents and count structures before completion publication. A host-read completion is not yet a successful guest ReadFile.
- Flush/close with explicit dirty-state, lock, alias, outstanding operation and view teardown rules. Cancel obsolete completions by operation/object/frame incarnation, not pointer equality.
- Manage shared file backing and per-process views with VM; inactive snapshots cannot overwrite canonical bytes. Unmap a view separately from closing the file/mapping object.
- Publish immutable provenance-labelled observations; SnapshotEpoch does not make independent endpoints atomic unless they share the same published snapshot.

Unmount/removal must distinguish namespace visibility from already-open volume references, pending I/O and mappings. Do not guess that removing a mount name permits immediate backing destruction. Checkpoint mutable state is independent; live host file changes/handles need an explicit snapshot/rebind policy, not a promise that cloning a Rust object rolls back the host disk.

## Host I/O and failure boundary

Host work may initially remain synchronous; converting it to typed accepted completion events is a reviewed later interface migration. Such events identify the actual object/operation, bytes or error and observed HostInstant. The machine assigns delivery under the anchor's clock mode; a storage worker neither sleeps the guest clock nor adds another elapsed offset.

Guest writes are preflighted before irreversible publication where possible. If bytes were already written or a host write partially completed, return an honest failure/partial-transfer result according to the audited API; do not invent atomic disk/output rollback, guessed replays or success counts. Cancellation prevents stale delivery but cannot undo an already-committed external write. Bound staging and retained history; distinguish file length, Vec capacity, mapped view bytes and host RSS.

A raw guest-memory adapter still has copied view bytes. Its Flush/Unmap
commits the requested source range into canonical pages and then propagates
overlapping bytes to same-birth guest aliases. If propagation fails, the API
reports failure, but the canonical page and earlier guest alias writes may
already have changed. This partial-effect limit does not apply to the native
in-slice pointer aliases, which map the same host page, and must not be
misreported as an atomic rollback or as independent file-buffer coherence.

Read-only existence/attribute/open probes must not create marker files or directories. Guest-authorized create/write/delete is a separate real API effect under the mounted configuration; investigation/cleanup authority does not extend to user SD contents. Path component checks do not by themselves prove immunity to host junction/symlink escapes; establish resolved-root policy explicitly.

## CE/source evidence

Primary local `C:/WINCE600/PRIVATE/WINCEOS/COREOS/STORAGE/FSDMGR/fileapi.cpp` provides separate read/write signatures, safe OVERLAPPED copying, access checks, mounted-volume EnterWithWait/Exit and delegated filesystem completion (133–234). Handle/open-context and mapping parity require connected evidence, not an assumption based on Rust clone.

`STORAGE/DISKCACHE/cacheapis.cpp` and `cache.cpp` implement bounded block/replacement cache contracts. The emulator 64 KiB window is its existing host-memory policy, not a device hardware cache inferred from registry CacheSize=0. Disk/FMD/AFS/FSDMGR lower-layer behavior and dumped drivers remain evidence gates; mounted host files do not prove physical flash/cache-driver emulation.

## Migration consumers, gates and pending decisions

Consumers: HostFileSystem open/read_into/read_at/write/seek/flush/close/duplicate/find/lock/mount; Kernel file/device dispatch, notifications and mapped views; coredll raw file/volume/store/AFS APIs; native read/overlapped continuations; VM shared-view synchronization; runtime mount/config and checkpoint paths.

Before connected changes, retain/execute focused file-window, cache invalidation, EOF/zero-read, sparse read, partial sink/count failure, writable flush and mount-path tests. Add canonical duplicate cursor/buffer visibility, last-alias close with pinned operations/views, removal/cancellation, host partial I/O, permission crossing and clone/host-mutation controls. Existing tests and historical runs are baseline evidence only; no new test execution occurred for this document.

Open decisions (open-context alias/cursor contract, writable buffer budget, host snapshot/rebind, share/delete/lock, mount-removal pins, error mapping, bounded asynchronous completion, symlink containment): [DECISIONS.md](../DECISIONS.md) O-S1 to O-S8. Resolve them before implementation; no filesystem rewrite or memory saving is claimed.
