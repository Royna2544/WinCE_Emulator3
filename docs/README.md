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
| Installed baseline | BED2: clean bounded 330 s run, still on the decorative splash (ledger E-1) |
| App goal | BED2 first splash observed at +4.358 s; positive `COM7:` guest reads are proven, but parsed position, map presentation and responsive navigation remain open (E-1) |
| Architecture implemented | None of the proposed types exist. Landed so far: compatibility `TaskWaitStore`, Phase 0 hot-path hygiene (amortized block hook, hookless default, per-slice MIPS logging; ledger E-9). Both are partly uncommitted |
| Experiment | Persistent-owner 650 passed 1,618 tests, then host-crashed at 129.108 s with 1.62 GiB RSS (E-2). Source preserved in the D: checkout; deferred to Stage 8 |
| Plan position | Stage 0 evidence is diagnostic and ongoing; Stage 1 pages are written. E-21 migrated public process-birth aliases, not full Stage 2/3 Task ownership. Stages 3-6 run on fresh engines (D-003) |
| Red baselines | Task/handle ownership 13 controls, 1 pass, 12 expected red (E-8/E-11); historical E-7 native stop/error reds are green in the D: checkout, while E-18 private fresh-owner bytes are expected red |
| Decisions | D-001 to D-008 decided; D-008 selects the O-M1 backing design, not its implementation. Next gates: O-P1, O-M1 and O-P4 (Stage 2), then O-T1, O-K1 to O-K3 (Stage 3) |
| Source control | D-004 now permits an external supervisor's unsigned backup commits on `main`; those snapshots are not reviewed releases and do not replace frozen test evidence |
| Source of truth | One active source dir (this repo) and one Cargo target dir (D-007). The D working tree still contains the persistent-owner experiment; E-7 stop/error controls pass and O-P1 stop taxonomy is partial. O-P4 fresh-default/VM coherence remains open |
| Next action | Extend E-21's canonical birth record into owner-scoped private VM authority, then turn E-18's private-byte expected red green with exact pinned backing; prove closed-owner Context and public same-VA/shared VM before a fresh default. Complete O-P1 classification separately. Source map: [CPU page](subsystems/cpu-engine.md#stage-2-source-map-historical-fault-and-current-handoff) |

## Standalone boundaries

| Subsystem | Design page | Current source scope |
| --- | --- | --- |
| Runtime composition/control | [runtime control](subsystems/runtime-control.md) | main, desktop, monitor, CPU facade |
| Native CPU engine | [CPU](subsystems/cpu-engine.md) | emulator/unicorn, native_engine, cpu_mips, arch |
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
| Remote ingress/observation | [remote](subsystems/remote-observability.md) | remote_server, ce/remote, debug publication |
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
