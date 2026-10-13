# Devices, byte streams and sensors

Status: proposed stream-device and external-event plan. [ARCHITECTURE](../ARCHITECTURE.md) defines machine sequencing, canonical identity, checked GuestMemory, clocks and SnapshotEpoch; [PLAN](../PLAN.md) controls implementation. Mounted files belong to [storage](storage.md), task/wait lifetime to [tasks/scheduler](tasks-scheduler.md) and [kernel objects](kernel-objects.md), and time conversion to [clock/timers](clock-timers.md).

## Current boundary and model limits

[DeviceNamespace/DeviceSession](../../src/ce/devices.rs) loads [serial_devices.json](../../serial_devices.json), matches configured names/backends and creates per-open DCB/timeouts/masks/RX/TX/protocol state. Registry activation/key aliases and KernelObject::Device connect stream APIs to sessions. COM7 remote injection, host Win32 serial and IOCTL devices are distinct configured paths; mounted file reads are not device streams.

[devices/](../../src/ce/devices) contains accelerometer, Yamaha magnetometer, I2C, light, NANDUUID and PIC handlers. They model decoded stream-driver commands/state, not a board-level hardware emulator:

- Yamaha replies preserve command latching and the two-address/single-address wire formats. The explicit ideal-device model uses instantaneous conversions and no factory trim/noise; older two-address analog DAC gain is unestablished. Documented single-address compensation is not proof of the older analog hardware.
- Accelerometer configuration/packed readings do not establish physical impact/IRQ/noise simulation.
- I2C register transactions do not simulate electrical ACK, bus timing or interrupts; NANDUUID/PIC models likewise do not reproduce flash/MCU firmware timing.

Current remote bytes are queued separately, drained to a matching open device, then read by guest APIs. **HTTP GPS ACK proves acceptance only.** SerialRemoteDrain proves remote-to-session movement, not guest buffer delivery. A positive successful guest ReadFile/count and native field/byte consumption provide later evidence; empty host queues alone do not.

Current error gaps are source-visible: DeviceSession::read_file drains RX before Kernel's fallible guest sink writes; failed guest copy can lose consumed stream bytes. DeviceSession::write_file appends TX and returns requested length while ignoring the host serial write result. No target rollback/completion guarantee should be attributed to these paths.

## Target authority and ownership

CeMachine accepts typed, sequenced external input/completion events. Device adapters own host resources and protocol execution; canonical device/open-session objects own guest-visible state. Sessions, the configured physical device/protocol object, pending I/O and event handles have explicit separate lifetimes. Decide which driver state is per-device versus per-open from its real contract, not a generic assumption.

Canonical ObjectIdentity identifies device/session incarnation; pending operations retain their owning TaskIdentity/FrameIdentity and AddressSpace. Public aliases do not deep-copy mutable device state accidentally. Execution/public-alias/wait-operation references pin lifetime; close/removal/exit cancellation retires operations once. A borrowed Win32 serial handle cannot silently become a persistent callback or checkpoint-owned external connection.

Proposed logical interfaces:

- Open a configured namespace entry with validated access and explicit host-open failure, then publish the guest handle.
- Accept actual serial bytes/sensor samples with validated target, units/format and observed HostInstant; queue bounded typed events, never mutate CPU/task/time directly from transport threads.
- Prepare a read/IOCTL reply against live session/operation and output capacity; validate GuestMemory before consumption/publication where possible; commit exact bytes/status once.
- Report host partial write/error honestly; TX observation is not proof of external transmission. Preserve source data/operation state according to the reviewed API, not invented replay.
- Register serial readiness/WaitCommEvent/overlapped completion under canonical wait/frame identity, with real comm masks and API-specific deadline semantics.
- Apply purge, reset, close, device removal and process exit through one cancellation/lifetime boundary; reject stale completions rather than target a replacement handle/address.
- Snapshot logical queues/protocol state independently; live host endpoints need an explicit checkpoint rebind/replay policy, not copied real-world side effects.

Already-live output can have a partial-write failure; the plan does not invent byte rollback or re-enqueue bytes after arbitrary external side effects. Unknown IOCTLs/errors remain explicit unless real evidence establishes inert success.

## External time and completion

Typed event examples are real serial-byte arrival, a validated sensor-sample update, host-I/O completion or device removal. These are conceptual payloads, not new common enum definitions. Event order/timestamps come through the anchor's machine sequencer.

HostInstant observes host arrival/service time; GuestInstant governs guest waits/deadlines and conversion delivery. No sensor/serial adapter adds host elapsed time and a separate virtual sleep offset. Live mode cannot fast-forward over unknown future input; deterministic replay needs an established recorded event schedule. NMEA timestamp fields are protocol data, not direct permission to set the guest clock.

Immediate ideal sensor conversion is a declared model limitation. Adding realistic latency/IRQ behavior requires primary evidence, a guest-deadline/event design and cancellation tests; never invent completion signals to escape a startup loop.

## CE/device evidence

Local CE6 `PRIVATE/WINCEOS/COREOS/DEVICE/DEVCORE/devfile.c` separates device Read/Write/IOControl from mounted storage. DM_DevReadFile316–350 checks read access/dying device, calls the real open-context Read entry and pins/releases open references; Write and IOControl paths have their own errors/lifetime rules. This supports typed stream/session boundaries, not generic success for every driver.

True target binaries under `D:/INAVI_Emulator/DUMPPLZ/Windows` are evidence for YAS526B.dll, SMB380.dll, gio_i2c*.dll, NANDUUID.dll and pic_drv.dll command layouts; preserve exact observed IOCTL/selector/buffer contracts. Yamaha YAS529 reference material supports the documented single-address wire format, not full physical simulation. Source comments and prior disassembly notes must remain provenance/limits, not unsupported calibration or app-readiness claims.

## Consumers, gates and pending decisions

Consumers: namespace open/config; Kernel create/read/write/DeviceIoControl, host poll/remote drain, comm masks/timeouts/purge and pending overlapped records; coredll raw buffer/count/OVERLAPPED marshaling; native blocked serial/comm continuation/resume; remote transport and sensor sample validation; device/registry activation and close/removal; checkpoint paths.

No test execution occurred for this plan. Existing device protocol/raw/native controls are baseline evidence only; new architecture controls remain required:

1. Exact packed field/endianness/selector/capacity/zero-size/unknown-IOCTL controls against dumped caller shapes.
2. Failed/unwritable guest destination before RX consumption, cross-page permissions, count-output failure and honest documented partial behavior.
3. FIFO serial chunks, partial host writes/errors, comm masks/timeouts, overlapped signal versus actual GetOverlappedResult consumption, cancellation/removal and stale operation rejection.
4. Alias/session/device-state separation, simultaneous opens and checkpoint isolation without duplicate host consumption.
5. Typed arrival ordering and no double time in both live/replay modes; real guest native reads distinguish ACK, drain, copied bytes and parsed application use.
6. Sensor command-latched replies, saturation/underflow/overflow and declared ideal limits; timing/IRQ only after separate evidence gates.

Open decisions (per-device vs per-open state, alias semantics, RX/TX capacity, host error propagation, operation preparation/commit, timeout conversion, checkpoint host endpoints, physical sensor timing scope): [DECISIONS.md](../DECISIONS.md) O-D1 to O-D8. No new samples, events, forced callbacks or sensor hardware claims are introduced.
