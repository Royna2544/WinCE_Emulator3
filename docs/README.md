# Windows CE emulator design documentation

Start with the [common architecture anchor](ARCHITECTURE.md), then the
[plan-first migration sequence](PLAN.md). Subsystem pages are design contracts,
not a claim of implemented architecture. Shared ownership/identity/time/outcome
definitions live only in the anchor; implementation follows reviewed gates.
Decisions and open questions are in [DECISIONS.md](DECISIONS.md); executed
evidence is in the [evidence ledger](evidence/ledger.md).

## Status at a glance

| Item | State |
| --- | --- |
| Preserved measured fallback | BED2: clean bounded 330 s run, still on the decorative splash (ledger E-1); it is not the current release executable |
| App goal | The latest launched E-38 fresh-owner run survived 330 s with COM7 reads and returned tap handlers, but stayed on the splash. Later scheduler and D-011 source cuts have not run iNavi; map and responsive navigation remain open |
| Architecture implemented | Process-birth identity, public private/shared slabs, sealed immutable main-image pages, fixed initial heap/stack and per-birth heap source gates exist with fresh owners by default. Opt-in synthetic writable-main-image ownership passes bounded source tests, but the ordinary SD-app loader remains copy-backed. One finite-Sleep Task/Frame path and bounded native/decoded pseudo self-exit paths are source-gated; raw direct pseudo handling remains fixture-only and fail-closed. General Task/Frame authority, production writable image/DLL ownership, remaining VM families and full Stage 2/3 acceptance are not complete |
| Experiment | Persistent-owner 650 passed 1,618 tests, then host-crashed at 129.108 s with 1.62 GiB RSS (E-2). Source preserved in the D: checkout; deferred to Stage 8 |
| Plan position | Stage 0 evidence is diagnostic and ongoing; Stage 1 pages are written. E-21 migrated public process-birth aliases, not full Stage 2/3 Task ownership. Stages 3-6 run on fresh engines (D-003) |
| Red baselines | Historical E-7 stop/error and E-18 private-byte reds became narrow greens; some E-8/E-11 Task/alias reds now have bounded source-gated greens. Remaining controls and canonical Task/Frame authority, Stage 2 VM coherence and live-app acceptance are open |
| Decisions | D-001 to D-012 decided; D-008's byte-authority design has bounded private/shared, immutable main-image and fixed/per-birth heap source gates, while writable image/DLL and clone lineage remain open. D-010 covers only the resealed finite-Sleep source gate; D-011 requires recovery before a stopped host exit with un-restorable saved pixels; D-012 keeps production EXE/DLL images copy-backed until O-M3 accounting is known; E-34 closes O-P1 stop classification |
| Source control | D-004 now permits an external supervisor's unsigned backup commits on `main`; those snapshots are not reviewed releases and do not replace frozen test evidence |
| Source of truth | One active source dir (this repo) and one disposable Cargo target dir (D-007). Fresh native owners are the default; persistent reuse is a Stage 8 opt-in. O-P1 classification is closed, but report hardening and Stage 2 VM coherence remain open |
| Live-run gate | The [a04 script-only seal](../evidence/stage3-live-evidence-20261015/prelaunch-repair-a04-source-seal.json), [bridge](../evidence/stage3-live-evidence-20261015/prelaunch-repair-a04-build-bridge.json) and [offline gate](../evidence/stage3-live-evidence-20261015/prelaunch-repair-a04-offline-gate.raw.txt) are historical after later scheduler and D-011 source cuts, not a launch seal for current code. The independent watchdog has a pre-spawn marker and retains unresolved empty-WMI cleanup; offline controls are not live integration proof. An earlier backup stopped after one verified SD ZIP without runner/guest launch; its approval is withdrawn and both ledgers have seven deny rows ([evidence](evidence/ledger.md)). A future run needs a new source/build seal and distinct approval |
| Next action | The bounded callback-scoped Kernel/framebuffer port is source-gated; now review and connect O-T7a's one host-turn selector and exact-birth CPU custody across boot, ordinary/Event/monitor entry, deferred child handoff and rotation before claiming a machine-owned lease. The process-scoped Cut 1 must precede exact thread/frame Cut 2. In the parallel Stage 2 lane, finish synthetic host-write/restore/rebind fences and O-M3 physical-page accounting before moving any production EXE/DLL off the copy path. Before any further backup or original-mount iNavi run, obtain a new distinct approval, make a complete fresh adjacent ZIP triplet and verify its inventory. Independent containment assumes D: volume I/O progress; a volume-wide stall also stalls guest writes on these mounts. The retained incomplete SD ZIP cannot be reused; the E-29 attempt trace remains a parallel evidence lane |

## Standalone boundaries

| Subsystem | Design page | Current source scope |
| --- | --- | --- |
| Runtime composition/control | [runtime control](subsystems/runtime-control.md) | main, desktop, monitor, CPU facade |
| Native CPU engine | [CPU](subsystems/cpu-engine.md) | emulator/unicorn, native_engine, cpu_mips, arch |
| Native post-stop tail custody | [tail capsule](subsystems/native-tail-capsule.md) | Deferred child handoff's still-open native owner, callbacks, pins and finish order (design only) |
| Guest memory/VM/heaps | [memory](subsystems/guest-memory.md) | ce/memory, emulator/memory, native byte adapters |
| PE/module loading | [loader](subsystems/pe-loader.md) | pe, pe_loader, dll_search, imports |
| CE ABI / host shims | [API boundary](subsystems/api-boundary.md) | coredll/ordinals, CRT, typed guest memory |
| Tasks, waits, scheduler | [tasks](subsystems/tasks-scheduler.md) | scheduler, task_wait, thread/native continuations |
| Kernel objects/handles | [objects](subsystems/kernel-objects.md) | kernel, object, handles/lifecycle |
| Clock/timers | [time](subsystems/clock-timers.md) | timer, quantum/wait progression |
| GWE/windows/messages | [GWES](subsystems/gwe-messaging.md) | gwe, keybd, dispatch/modal/callbacks |
| GDI/framebuffer | [graphics](subsystems/gdi-framebuffer.md) | GDI shims, framebuffer, desktop presenters |
| Files/mounts/shared views | [storage](subsystems/storage.md) | file, ObjectStore, mounts, mapping views |
| Registry/configuration | [registry](subsystems/registry.md) | registry, config, seeded registry.reg |
| Devices/serial/sensors | [devices](subsystems/devices-sensors.md) | devices, COM7/MFS1/SMB1, nled, device config |
| Audio | [audio](subsystems/audio.md) | audio/waveOut, host/WebSocket sinks |
| Guest networking | [network](subsystems/network.md) | winsock, host sockets/readiness |
| Resources/shell/COM | [resources](subsystems/resources-shell-com.md) | resource, shell, COM/OLE, helper libraries |
| Notification service | [notifications](subsystems/notification-service.md) | CeEventHasOccurred, NFY0 evidence, pending Event launches and UI handoff |
| Remote ingress/observation | [remote](subsystems/remote-observability.md) | remote_server, ce/remote, debug publication |
| Mounted live-run control | [live run](subsystems/live-run-control.md) | approval, mount backups, watchdog and evidence capture |
| Native-attempt diagnostics | [attempt observability](subsystems/native-attempt-observability.md) | CPU raw report, host trace writer, optional executed-event collector |
| Verification/bring-up | [verification](subsystems/verification.md) | raw/native fixtures, real app evidence |
| Artifacts/reproduction | [artifact lifecycle](subsystems/artifact-lifecycle.md) | one build directory, preserved evidence, cleanup receipts |

Other pages: [decision log](DECISIONS.md), [evidence ledger](evidence/ledger.md),
[app evidence lane](app-evidence-lane.md).

Pure math, Hangul/text and ordinal helpers follow their consuming API/graphics
contract; they do not get independent clocks, scheduling or mutable machine
authority. Native/GWE/host threads exchange typed commands and immutable views,
not extra writable copies of Task or guest RAM.

## Review and use

Every subsystem page identifies current versus target state, owning lifetime,
allowed interfaces/dependencies, error/cancellation boundaries, telemetry and
tests. Open choices live in the decision log; staged tests remain unexecuted
until recorded otherwise in the ledger. See PROGRESS/TODO/KNOWN_BUGS and
SOURCE_REFERENCES at the repository root for current evidence and primary local
CE6/device/native sources. Do not start a feature migration from an isolated
page without its common-anchor gate.
