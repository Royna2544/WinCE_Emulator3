# Registry

Status: proposed registry boundary plan. Shared authority, identities, memory publication, external events and evidence come from [ARCHITECTURE](../ARCHITECTURE.md); [PLAN](../PLAN.md) gates implementation. Registry content is owned here; canonical handle references follow [kernel objects](kernel-objects.md), guest copies follow [guest memory](guest-memory.md), and driver/mount configuration consumers follow their subsystem plans.

## Current implementation

[registry.rs](../../src/ce/registry.rs) stores normalized keys/typed values and private HKey-to-path mappings in BTreeMaps. [config.rs](../../src/config.rs) loads [registry.reg](../../registry.reg) as the default seed: REGEDIT text is decoded through UTF-8/lossy conversion, while typed hex string values use UTF-16 decoding. JSON compatibility loading also remains. Configuration seed provenance is not a live CE hive.

The current model covers predefined roots, open/create/close, query/set/enumerate/delete and value serialization. Name normalization and deterministic BTree iteration are implementation choices; they are not proof of every CE locale/enumeration/security rule. Desired access/options are currently not a complete access-control model. Delete-key removes stored subtrees and associated path handles.

Query results distinguish required byte/character lengths, value type, data presence and ERROR_MORE_DATA. The raw coredll query/enumeration adapters check memory operations but write several outputs sequentially: earlier outputs may change before a later write fails. This is not an atomic output transaction. Size-only queries are distinct from unreadable buffers or missing values.

RegFlushKey currently succeeds as a no-op; it does not persist a CE hive. Several RegistryNotify* paths are successful no-ops, not actual asynchronous notification delivery. These limitations must stay visible. No claim is made that all registry-dependent device activation or shell behavior is implemented.

## Target authority and APIs

CeMachine sequences mutations and accepted external changes. Registry owns canonical content and key/value identity; public HKeys are typed references governed by canonical object/alias lifetime, not another independent Task registry. Predefined roots remain explicit namespace tokens. Do not infer key ownership from the process selected during a later query.

Proposed logical operations:

- Load a versioned/provenance-labelled seed, with an explicit decoder and supported types; separate seed configuration from durable mutable hive policy.
- Prepare open/create handles with validated path/access/owner before publishing outputs; abort unpublished candidates on output failure rather than leak live aliases.
- Query/enumerate a bounded typed value image and exact required lengths; preflight guest output extents under AddressSpace/FrameIdentity before publication.
- Read complete input bytes before set/delete mutation, validate type/encoding/reserved arguments and report API status separately from LastError where CE requires it.
- Duplicate/close/delete references with defined tombstone/open-key behavior; pin content for pending notifications/operations and invalidate stale incarnations/generations.
- Deliver only real subscribed change notifications through typed sequenced events and canonical frame/object references. Unsubscribe/close/exit cancels stale deliveries.
- Snapshot canonical content/handles independently on checkpoint, with SnapshotEpoch. Host persistence errors are visible and cannot turn into a fabricated successful flush.

Content changes must not manufacture app-specific readiness values, create guessed driver state or force a callback. Configuration consumers may read registry data; they do not acquire independent scheduling/clock authority.

## Lifetime, persistence and cancellation

Registry key content, an open HKey, a notification subscription and host backing have different lifetimes. Decide deletion semantics with surviving handles and recursive behavior from CE evidence; do not preserve the current path-string behavior merely by renaming it canonical identity. Bound subscription/event records and retire pins, not immortal deleted keys.

A persistence design must state commit point, ordering, crash/flush semantics, backing location and restore behavior. It must never rewrite the synced fixture or user host registry implicitly. Checkpoints clone logical machine state; undoing external persistence needs a separate authorized policy. Host I/O completion can be a typed event tagged with object/operation provenance and HostInstant; only the machine decides guest delivery/time assignment.

For failed guest writes, preserve the anchor's distinction between prepared publication and already-live partial outputs. Return honest errors; do not promise to restore arbitrary guest bytes or roll back a committed registry mutation unless the API transaction actually provides that guarantee. A canceled notification cannot be revived by matching only a numeric HKey.

## Evidence and current consumers

Primary ABI evidence: installed CE SDK `STANDARDSDK_420/Include/Mipsii/winreg.h` declares RegQueryValueExW's type/data/length outputs, RegSetValueExW's typed input and enumeration/close/flush APIs. This is header/ABI evidence, not proof of CE6 hive/security/notification implementation. The full CE6 registry implementation remains unidentified in this design; locate it and dumped coredll forwarding before introducing deeper parity claims.

Source consumers: coredll raw Registry APIs and high-level adapters; RuntimeConfig seed loading; Kernel driver activation/device keys, mounted-storage profile copying, shell/system values and change-notification shims. Every path must use the same status, encoding, lifetime and checked guest-memory boundary.

## Gates and unresolved choices

No new test was compiled or executed in this documentation lane. Existing parser/raw registry tests are current-model evidence, not target hive acceptance.

Required gates:

1. REGEDIT/typed UTF-16/binary/DWORD/multi-string decoder controls, malformed/truncated input and explicit unsupported encodings.
2. Size-only, exact-size/short buffer, optional type/count output, null/unwritable/cross-permission output and already-live prefix-failure behavior through actual raw APIs.
3. Case/name/ordering/Unicode/default-value semantics backed by CE evidence; typed status versus LastError parity.
4. Canonical alias/open-key deletion lifetime, stale handle/generation rejection and independent checkpoint mutations.
5. Real subscription delivery/cancel/close/exit, ordering and duplicate completion controls.
6. If persistence is approved: flush error, disk-full/partial commit, restart/reload and host-change policy in isolated temporary storage.

Open decisions (CE6 key/access/security semantics, per-process handle visibility, deletion with open aliases, notifications, hive persistence, decoder code pages, enumeration stability, storage/driver configuration publication): [DECISIONS.md](../DECISIONS.md) O-R1 to O-R8. No new handle model, notification implementation or host registry write is authorized by this plan.
